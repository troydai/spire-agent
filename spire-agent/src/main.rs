use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{CommandFactory, Parser, Subcommand};
use const_oid::db::rfc5280::ID_CE_BASIC_CONSTRAINTS;
use der::{Decode, Reader};
use hyper_util::rt::TokioIo;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use x509_cert::Certificate;

mod workload;

use workload::X509svidRequest;
use workload::spiffe_workload_api_client::SpiffeWorkloadApiClient;

#[derive(Parser)]
#[command(name = "spire-agent", version, about = "Agent CLI for Spire")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Api(ApiArgs),
}

#[derive(Parser)]
struct ApiArgs {
    #[arg(
        long = "output",
        value_name = "value",
        default_value = "pretty",
        global = true,
        help = "Desired output format (pretty, json); default: pretty."
    )]
    output: String,
    #[arg(long = "silent", global = true, help = "Suppress stdout")]
    silent: bool,
    #[arg(
        long = "socketPath",
        value_name = "string",
        default_value = "/tmp/spire-agent/public/api.sock",
        global = true,
        help = "Path to the SPIRE Agent API Unix domain socket (default \"/tmp/spire-agent/public/api.sock\")"
    )]
    socket_path: String,
    #[arg(
        long = "timeout",
        value_name = "value",
        default_value = "5s",
        global = true,
        help = "Time to wait for a response (default 5s)"
    )]
    timeout: String,
    #[arg(
        long = "write",
        value_name = "string",
        global = true,
        help = "Write SVID data to the specified path (optional; only available for pretty output format)"
    )]
    write: Option<String>,
    #[command(subcommand)]
    command: ApiCommand,
}

#[derive(Subcommand)]
enum ApiCommand {
    Fetch(FetchArgs),
    Watch,
}

#[derive(Parser)]
struct FetchArgs {
    #[command(subcommand)]
    command: FetchCommand,
}

#[derive(Subcommand)]
enum FetchCommand {
    X509,
}

fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if let Some(secs) = s.strip_suffix('s') {
        let secs: u64 = secs.parse().context("invalid duration")?;
        return Ok(Duration::from_secs(secs));
    }
    if let Some(ms) = s.strip_suffix("ms") {
        let ms: u64 = ms.parse().context("invalid duration")?;
        return Ok(Duration::from_millis(ms));
    }
    // Default to seconds if no suffix
    let secs: u64 = s.parse().context("invalid duration")?;
    Ok(Duration::from_secs(secs))
}

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

async fn fetch_x509(socket_path: &str, timeout: Duration, silent: bool) -> Result<()> {
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
                    exts.iter()
                        .find(|ext| ext.extn_id == ID_CE_BASIC_CONSTRAINTS)
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Api(ApiArgs {
            command:
                ApiCommand::Fetch(FetchArgs {
                    command: FetchCommand::X509,
                }),
            socket_path,
            timeout,
            silent,
            ..
        })) => {
            let timeout = match parse_duration(&timeout) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error parsing timeout: {e}");
                    std::process::exit(1);
                }
            };

            if let Err(e) = fetch_x509(&socket_path, timeout, silent).await {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Api(ApiArgs {
            command: ApiCommand::Watch,
            ..
        })) => {
            // Intentionally no-op for now.
        }
        None => {
            if let Err(err) = Cli::command().print_long_help() {
                eprintln!("Failed to render help: {err}");
                std::process::exit(1);
            }
            println!();
        }
    }
}
