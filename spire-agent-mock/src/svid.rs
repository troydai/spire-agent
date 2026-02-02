use base64::Engine;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use p256::elliptic_curve::rand_core::OsRng;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose, SanType,
};
use serde_json::json;
use time::{Duration, OffsetDateTime};

use crate::workload::{Jwtsvid, X509svid};

/// Configuration for SVID generation
pub struct SvidConfig {
    pub trust_domain: String,
    pub workload_path: String,
    pub ttl_seconds: u32,
}

const DEFAULT_TRUST_DOMAIN: &str = "spiffe-helper.local";
const DEFAULT_WORKLOAD_PATH: &str = "/ns/httpbin/sa/httpbin";
const DEFAULT_TTL_SECONDS: u32 = 120;
const DEFAULT_ROOT_CA_VALIDITY_DAYS: i64 = 3650;
const DEFAULT_SPIRE_SERVER_CA_VALIDITY_DAYS: i64 = 365;

impl Default for SvidConfig {
    fn default() -> Self {
        Self {
            trust_domain: DEFAULT_TRUST_DOMAIN.to_string(),
            workload_path: DEFAULT_WORKLOAD_PATH.to_string(),
            ttl_seconds: DEFAULT_TTL_SECONDS,
        }
    }
}

/// Generator for SPIFFE X.509 SVIDs
pub struct SvidGenerator {
    config: SvidConfig,
    root_cert_der: Vec<u8>,
    spire_server_issuer: Issuer<'static, KeyPair>,
    spire_server_cert_der: Vec<u8>,
    jwt_signing_key: SigningKey,
    jwt_kid: String,
}

impl SvidGenerator {
    /// Create a new SVID generator with the given configuration
    pub fn new(config: SvidConfig) -> Self {
        let (root_issuer, root_cert_der) = Self::generate_root_ca(&config.trust_domain);
        let (spire_server_issuer, spire_server_cert_der) =
            Self::generate_spire_server_ca(&config.trust_domain, &root_issuer);
        let jwt_signing_key = SigningKey::random(&mut OsRng);
        Self {
            config,
            root_cert_der,
            spire_server_issuer,
            spire_server_cert_der,
            jwt_signing_key,
            jwt_kid: "spire-agent-mock-jwt".to_string(),
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

        // Set validity period (10 years for root CA)
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(DEFAULT_ROOT_CA_VALIDITY_DAYS);

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

        // Set validity period (1 year for SPIRE Server signing CA)
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(DEFAULT_SPIRE_SERVER_CA_VALIDITY_DAYS);

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

    pub fn trust_domain(&self) -> &str {
        &self.config.trust_domain
    }

    pub fn workload_spiffe_id(&self) -> String {
        format!(
            "spiffe://{}{}",
            self.config.trust_domain, self.config.workload_path
        )
    }

    pub fn bundle(&self) -> Vec<u8> {
        self.root_cert_der.clone()
    }

    pub fn generate_jwt_svid(&self, audience: &[String], spiffe_id: Option<&str>) -> Jwtsvid {
        let spiffe_id = spiffe_id
            .filter(|id| !id.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| self.workload_spiffe_id());

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let expires_at = now + i64::from(self.config.ttl_seconds);

        let header = json!({
            "alg": "ES256",
            "kid": self.jwt_kid,
            "typ": "JWT",
        });
        let payload = json!({
            "aud": audience,
            "exp": expires_at,
            "iat": now,
            "sub": spiffe_id,
        });

        let encoded_header = encode_json_segment(&header);
        let encoded_payload = encode_json_segment(&payload);
        let signing_input = format!("{encoded_header}.{encoded_payload}");

        let signature: Signature = self.jwt_signing_key.sign(signing_input.as_bytes());
        let encoded_signature =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(signature.to_bytes());

        Jwtsvid {
            spiffe_id: spiffe_id.clone(),
            svid: format!("{signing_input}.{encoded_signature}"),
            hint: String::new(),
        }
    }

    pub fn jwt_bundle(&self) -> Vec<u8> {
        let encoded_point = self.jwt_signing_key.verifying_key().to_encoded_point(false);
        let x = encoded_point.x().expect("uncompressed x coordinate");
        let y = encoded_point.y().expect("uncompressed y coordinate");

        let jwks = json!({
            "keys": [{
                "kty": "EC",
                "kid": self.jwt_kid,
                "crv": "P-256",
                "x": base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(x),
                "y": base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(y),
            }],
        });

        serde_json::to_vec_pretty(&jwks).expect("jwks serialization must succeed")
    }
}

fn encode_json_segment(value: &serde_json::Value) -> String {
    let raw = serde_json::to_vec(value).expect("json serialization must succeed");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct JwtHeader {
        alg: String,
        kid: String,
        typ: String,
    }

    #[derive(Deserialize)]
    struct JwtClaims {
        aud: Vec<String>,
        exp: i64,
        iat: i64,
        sub: String,
    }

    #[test]
    fn test_generate_svid() {
        let config = SvidConfig::default();
        let generator = SvidGenerator::new(config);
        let svid = generator.generate_svid();

        assert_eq!(
            svid.spiffe_id,
            "spiffe://spiffe-helper.local/ns/httpbin/sa/httpbin"
        );
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
        let result =
            spiffe::svid::x509::X509Svid::parse_from_der(&svid.x509_svid, &svid.x509_svid_key);

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
    fn test_default_config_values() {
        let config = SvidConfig::default();
        assert_eq!(config.trust_domain, DEFAULT_TRUST_DOMAIN);
        assert_eq!(config.workload_path, DEFAULT_WORKLOAD_PATH);
        assert_eq!(config.ttl_seconds, DEFAULT_TTL_SECONDS);
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

    #[test]
    fn test_generate_jwt_svid_defaults_to_workload_spiffe_id() {
        let generator = SvidGenerator::new(SvidConfig::default());
        let audience = vec!["my-service".to_string()];
        let jwt_svid = generator.generate_jwt_svid(&audience, None);

        let segments: Vec<&str> = jwt_svid.svid.split('.').collect();
        assert_eq!(segments.len(), 3);
        assert_eq!(jwt_svid.spiffe_id, generator.workload_spiffe_id());

        let header_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(segments[0])
            .expect("header decode");
        let claims_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(segments[1])
            .expect("claims decode");

        let header: JwtHeader = serde_json::from_slice(&header_bytes).expect("header json");
        assert_eq!(header.alg, "ES256");
        assert_eq!(header.kid, "spire-agent-mock-jwt");
        assert_eq!(header.typ, "JWT");

        let claims: JwtClaims = serde_json::from_slice(&claims_bytes).expect("claims json");
        assert_eq!(claims.aud, audience);
        assert_eq!(claims.sub, generator.workload_spiffe_id());
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_generate_jwt_bundle_contains_single_key() {
        let generator = SvidGenerator::new(SvidConfig::default());
        let bundle = generator.jwt_bundle();

        let value: serde_json::Value = serde_json::from_slice(&bundle).expect("bundle json");
        let keys = value
            .get("keys")
            .and_then(serde_json::Value::as_array)
            .expect("keys array");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0]["kty"], "EC");
        assert_eq!(keys[0]["kid"], "spire-agent-mock-jwt");
        assert_eq!(keys[0]["crv"], "P-256");
    }
}
