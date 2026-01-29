use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use const_oid::db::rfc5280::ID_CE_BASIC_CONSTRAINTS;
use der::{Decode, Reader};
use hyper_util::rt::TokioIo;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use x509_cert::Certificate;

use crate::workload::spiffe_workload_api_client::SpiffeWorkloadApiClient;
use crate::workload::X509svidRequest;

fn format_utc_time(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M:%S +0000 UTC").to_string()
}

fn parse_x509_time(time: &x509_cert::time::Time) -> DateTime<Utc> {
    match time {
        x509_cert::time::Time::UtcTime(ut) => {
            let unix = ut.to_unix_duration().as_secs() as i64;
            DateTime::from_timestamp(unix, 0).unwrap_or_default()
        }
        x509_cert::time::Time::GeneralTime(gt) => {
            let unix = gt.to_unix_duration().as_secs() as i64;
            DateTime::from_timestamp(unix, 0).unwrap_or_default()
        }
    }
}

/// Parse a DER certificate chain (concatenated DER certificates)
fn parse_cert_chain(der_bytes: &[u8]) -> Result<Vec<Certificate>> {
    let mut certs = Vec::new();
    let mut reader = der::SliceReader::new(der_bytes).context("failed to create DER reader")?;

    while !reader.is_finished() {
        let cert = reader
            .decode::<Certificate>()
            .context("failed to parse certificate")?;
        certs.push(cert);
    }

    Ok(certs)
}

pub async fn fetch_x509(socket_path: &str, timeout: Duration, silent: bool) -> Result<()> {
    let start = Instant::now();

    // Connect to Unix socket
    let socket_path = socket_path.to_string();
    let channel = Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(move |_: Uri| {
            let path = socket_path.clone();
            async move {
                let stream = tokio::net::UnixStream::connect(path).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
        .context("failed to connect to socket")?;

    let mut client = SpiffeWorkloadApiClient::new(channel);

    // Create request with timeout
    let request = tonic::Request::new(X509svidRequest {});

    // Fetch X509 SVID (streaming RPC, we just need the first response)
    let response = tokio::time::timeout(timeout, client.fetch_x509svid(request))
        .await
        .context("request timed out")?
        .context("failed to fetch x509 svid")?;

    let mut stream = response.into_inner();

    // Get the first message from the stream
    let first_response = tokio::time::timeout(timeout, stream.message())
        .await
        .context("timed out waiting for response")?
        .context("failed to receive message")?
        .context("empty response from server")?;

    let elapsed = start.elapsed();

    if silent {
        return Ok(());
    }

    let svids = &first_response.svids;
    println!(
        "Received {} svid after {:.6}ms\n",
        svids.len(),
        elapsed.as_secs_f64() * 1000.0
    );

    for svid in svids {
        println!("SPIFFE ID:\t\t{}", svid.spiffe_id);

        // Parse the certificate chain
        let certs = parse_cert_chain(&svid.x509_svid)?;

        if let Some(leaf) = certs.first() {
            let validity = &leaf.tbs_certificate.validity;
            let not_before = parse_x509_time(&validity.not_before);
            let not_after = parse_x509_time(&validity.not_after);
            println!("SVID Valid After:\t{}", format_utc_time(not_before));
            println!("SVID Valid Until:\t{}", format_utc_time(not_after));
        }

        // Print intermediate and CA certificates
        let mut intermediate_num = 1;
        let mut ca_num = 1;

        for cert in certs.iter().skip(1) {
            let validity = &cert.tbs_certificate.validity;
            let not_before = parse_x509_time(&validity.not_before);
            let not_after = parse_x509_time(&validity.not_after);

            // Check if it's a CA certificate
            let is_ca = cert
                .tbs_certificate
                .extensions
                .as_ref()
                .and_then(|exts| {
                    exts.iter().find(|ext| ext.extn_id == ID_CE_BASIC_CONSTRAINTS)
                })
                .and_then(|ext| {
                    x509_cert::ext::pkix::BasicConstraints::from_der(ext.extn_value.as_bytes()).ok()
                })
                .is_some_and(|bc| bc.ca);

            if is_ca {
                println!(
                    "CA #{} Valid After:\t{}",
                    ca_num,
                    format_utc_time(not_before)
                );
                println!(
                    "CA #{} Valid Until:\t{}",
                    ca_num,
                    format_utc_time(not_after)
                );
                ca_num += 1;
            } else {
                println!(
                    "Intermediate #{} Valid After:\t{}",
                    intermediate_num,
                    format_utc_time(not_before)
                );
                println!(
                    "Intermediate #{} Valid Until:\t{}",
                    intermediate_num,
                    format_utc_time(not_after)
                );
                intermediate_num += 1;
            }
        }
    }

    Ok(())
}
