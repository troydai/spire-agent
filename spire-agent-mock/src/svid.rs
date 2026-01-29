use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair, KeyUsagePurpose, SanType,
};
use time::{Duration, OffsetDateTime};

use crate::workload::X509svid;

/// Configuration for SVID generation
pub struct SvidConfig {
    pub trust_domain: String,
    pub workload_path: String,
    pub ttl_seconds: u32,
}

impl Default for SvidConfig {
    fn default() -> Self {
        Self {
            trust_domain: "example.org".to_string(),
            workload_path: "/workload".to_string(),
            ttl_seconds: 30,
        }
    }
}

/// Generator for SPIFFE X.509 SVIDs
pub struct SvidGenerator {
    config: SvidConfig,
    root_cert_der: Vec<u8>,
    spire_server_issuer: Issuer<'static, KeyPair>,
    spire_server_cert_der: Vec<u8>,
}

impl SvidGenerator {
    /// Create a new SVID generator with the given configuration
    pub fn new(config: SvidConfig) -> Self {
        let (root_issuer, root_cert_der) = Self::generate_root_ca(&config.trust_domain);
        let (spire_server_issuer, spire_server_cert_der) =
            Self::generate_spire_server_ca(&config.trust_domain, &root_issuer);
        Self {
            config,
            root_cert_der,
            spire_server_issuer,
            spire_server_cert_der,
        }
    }

    /// Generate a root CA certificate for the trust domain
    fn generate_root_ca(trust_domain: &str) -> (Issuer<'static, KeyPair>, Vec<u8>) {
        let mut params = CertificateParams::default();

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, format!("{} Root CA", trust_domain));
        dn.push(DnType::OrganizationName, trust_domain);
        params.distinguished_name = dn;

        // CA certificate settings
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];

        // Set validity period (1 year for root CA)
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(365);

        // Add SPIFFE trust domain as URI SAN
        let trust_domain_uri = format!("spiffe://{}", trust_domain);
        params.subject_alt_names = vec![SanType::URI(trust_domain_uri.parse().unwrap())];

        // Generate key pair
        let key_pair = KeyPair::generate().unwrap();

        // Generate self-signed CA certificate
        let ca_cert = params.self_signed(&key_pair).unwrap();
        let ca_cert_der = ca_cert.der().to_vec();

        let issuer = Issuer::new(params, key_pair);

        (issuer, ca_cert_der)
    }

    /// Generate a SPIRE Server signing CA certificate signed by the root CA
    fn generate_spire_server_ca(
        trust_domain: &str,
        root_issuer: &Issuer<'static, KeyPair>,
    ) -> (Issuer<'static, KeyPair>, Vec<u8>) {
        let mut params = CertificateParams::default();

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(
            DnType::CommonName,
            format!("{} SPIRE Server CA", trust_domain),
        );
        dn.push(DnType::OrganizationName, trust_domain);
        params.distinguished_name = dn;

        // SPIRE Server signing CA settings
        params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];

        // Set validity period (1 month for SPIRE Server signing CA)
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(30);

        // Add SPIFFE trust domain as URI SAN
        let trust_domain_uri = format!("spiffe://{}", trust_domain);
        params.subject_alt_names = vec![SanType::URI(trust_domain_uri.parse().unwrap())];

        // Generate key pair
        let key_pair = KeyPair::generate().unwrap();

        // Generate SPIRE Server signing certificate signed by root
        let spire_server_cert = params.signed_by(&key_pair, root_issuer).unwrap();
        let spire_server_cert_der = spire_server_cert.der().to_vec();

        let issuer = Issuer::new(params, key_pair);

        (issuer, spire_server_cert_der)
    }

    /// Generate a new X.509 SVID
    pub fn generate_svid(&self) -> X509svid {
        let spiffe_id = format!(
            "spiffe://{}{}",
            self.config.trust_domain, self.config.workload_path
        );

        // Create workload certificate parameters
        let mut params = CertificateParams::default();

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, &spiffe_id);
        dn.push(DnType::OrganizationName, &self.config.trust_domain);
        params.distinguished_name = dn;

        // Leaf certificate (not a CA) - use ExplicitNoCa to ensure Basic Constraints
        // extension is included (required by SPIFFE spec)
        params.is_ca = IsCa::ExplicitNoCa;
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        params.extended_key_usages = vec![
            ExtendedKeyUsagePurpose::ServerAuth,
            ExtendedKeyUsagePurpose::ClientAuth,
        ];

        // Set validity period based on TTL
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::seconds(self.config.ttl_seconds.into());

        // SPIFFE ID as URI SAN - this is required by SPIFFE spec
        params.subject_alt_names = vec![SanType::URI(spiffe_id.parse().unwrap())];

        // Generate key pair for the workload
        let key_pair = KeyPair::generate().unwrap();

        // Sign with CA
        let cert = params
            .signed_by(&key_pair, &self.spire_server_issuer)
            .unwrap();

        // Certificate chain (per SPIFFE Workload API): leaf cert + intermediates only.
        // Root CAs belong in the bundle.
        let mut cert_chain = cert.der().to_vec();
        cert_chain.extend_from_slice(&self.spire_server_cert_der);

        X509svid {
            spiffe_id,
            x509_svid: cert_chain,
            x509_svid_key: key_pair.serialize_der(),
            bundle: self.root_cert_der.clone(),
            hint: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_svid() {
        let config = SvidConfig::default();
        let generator = SvidGenerator::new(config);
        let svid = generator.generate_svid();

        assert_eq!(svid.spiffe_id, "spiffe://example.org/workload");
        assert!(!svid.x509_svid.is_empty());
        assert!(!svid.x509_svid_key.is_empty());
        assert!(!svid.bundle.is_empty());
    }

    #[test]
    fn test_svid_can_be_parsed_by_spiffe_crate() {
        let config = SvidConfig::default();
        let generator = SvidGenerator::new(config);
        let svid = generator.generate_svid();

        // Try to parse with spiffe crate - this is what the client does
        let result = spiffe::svid::x509::X509Svid::parse_from_der(
            &svid.x509_svid,
            &svid.x509_svid_key,
        );

        match &result {
            Ok(parsed) => {
                println!("Parsed SPIFFE ID: {}", parsed.spiffe_id());
            }
            Err(e) => {
                println!("Parse error: {:?}", e);
            }
        }

        assert!(result.is_ok(), "Failed to parse SVID: {:?}", result.err());
    }

    #[test]
    fn test_custom_config() {
        let config = SvidConfig {
            trust_domain: "test.domain".to_string(),
            workload_path: "/my/service".to_string(),
            ttl_seconds: 60,
        };
        let generator = SvidGenerator::new(config);
        let svid = generator.generate_svid();

        assert_eq!(svid.spiffe_id, "spiffe://test.domain/my/service");
    }
}
