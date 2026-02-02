use std::time::Duration;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

use crate::fetch_jwt::fetch_jwt;
use crate::fetch_x509::fetch_x509;
use crate::healthcheck::healthcheck;

#[derive(Parser)]
#[command(name = "spire-agent", version, about = "Agent CLI for Spire")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Api(ApiArgs),
    Healthcheck(HealthcheckArgs),
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
    #[arg(
        long = "socketPath",
        alias = "socket-path",
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
        long = "format",
        value_name = "value",
        global = true,
        help = "deprecated; use -output"
    )]
    format: Option<String>,
    #[command(subcommand)]
    command: ApiCommand,
}

#[derive(Parser)]
struct HealthcheckArgs {
    #[arg(long = "shallow", help = "Perform a less stringent health check")]
    shallow: bool,
    #[arg(
        long = "socket-path",
        alias = "socketPath",
        value_name = "string",
        default_value = "/tmp/spire-agent/public/api.sock",
        help = "Path to the SPIRE Agent API socket (default \"/tmp/spire-agent/public/api.sock\")"
    )]
    socket_path: String,
    #[arg(long = "verbose", help = "Print verbose information")]
    verbose: bool,
}

#[derive(Subcommand)]
enum ApiCommand {
    Fetch(FetchArgs),
    Watch,
}

#[derive(Parser)]
struct FetchArgs {
    #[command(subcommand)]
    command: Option<FetchCommand>,
}

#[derive(Subcommand)]
enum FetchCommand {
    X509(FetchX509Args),
    Jwt(FetchJwtArgs),
}

#[derive(Parser, Default)]
struct FetchX509Args {
    #[arg(long = "silent", help = "Suppress stdout")]
    silent: bool,
    #[arg(
        long = "write",
        value_name = "string",
        help = "Write SVID data to the specified path (optional; only available for pretty output format)"
    )]
    write: Option<String>,
}

#[derive(Parser, Default)]
struct FetchJwtArgs {
    #[arg(
        long = "audience",
        value_name = "value",
        value_delimiter = ',',
        help = "comma separated list of audience values"
    )]
    audience: Vec<String>,
    #[arg(
        long = "spiffeID",
        alias = "spiffe-id",
        value_name = "string",
        help = "SPIFFE ID subject (optional)"
    )]
    spiffe_id: Option<String>,
}

fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    let (value, unit) = split_duration(s)?;

    match unit {
        "" | "s" => Ok(Duration::from_secs(value)),
        "ns" => Ok(Duration::from_nanos(value)),
        "us" | "µs" => Ok(Duration::from_micros(value)),
        "ms" => Ok(Duration::from_millis(value)),
        "m" => Ok(Duration::from_secs(value.saturating_mul(60))),
        "h" => Ok(Duration::from_secs(value.saturating_mul(60 * 60))),
        _ => anyhow::bail!("invalid duration"),
    }
}

fn split_duration(s: &str) -> Result<(u64, &str)> {
    if s.is_empty() {
        anyhow::bail!("invalid duration");
    }

    let split_idx = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let (value_str, unit) = s.split_at(split_idx);
    if value_str.is_empty() {
        anyhow::bail!("invalid duration");
    }

    let value: u64 = value_str.parse().context("invalid duration")?;
    Ok((value, unit))
}

pub async fn run() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Api(ApiArgs {
            command: ApiCommand::Fetch(FetchArgs { command }),
            socket_path,
            timeout,
            output,
            format,
        })) => {
            let timeout = match parse_duration(&timeout) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error parsing timeout: {e}");
                    std::process::exit(1);
                }
            };
            let output = format.unwrap_or(output);

            match command.unwrap_or(FetchCommand::X509(FetchX509Args::default())) {
                FetchCommand::X509(FetchX509Args { silent, write }) => {
                    if output != "pretty" {
                        eprintln!("Error: only pretty output is supported for api fetch x509");
                        std::process::exit(1);
                    }
                    if let Err(e) =
                        fetch_x509(&socket_path, timeout, silent, write.as_deref()).await
                    {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
                FetchCommand::Jwt(FetchJwtArgs {
                    audience,
                    spiffe_id,
                }) => {
                    if audience.is_empty() {
                        eprintln!("audience must be specified");
                        std::process::exit(1);
                    }
                    if let Err(e) =
                        fetch_jwt(&socket_path, timeout, audience, spiffe_id, &output).await
                    {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
        Some(Commands::Api(ApiArgs {
            command: ApiCommand::Watch,
            ..
        })) => {
            // Intentionally no-op for now.
        }
        Some(Commands::Healthcheck(HealthcheckArgs {
            shallow,
            socket_path,
            verbose,
        })) => {
            if let Err(e) = healthcheck(&socket_path, shallow, verbose).await {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        ApiArgs, ApiCommand, Cli, Commands, FetchArgs, FetchCommand, FetchJwtArgs, parse_duration,
    };
    use clap::Parser;

    #[test]
    fn parse_duration_supports_units() {
        assert_eq!(parse_duration("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("250ms").unwrap(), Duration::from_millis(250));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("10").unwrap(), Duration::from_secs(10));
    }

    #[test]
    fn parse_duration_rejects_invalid() {
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("1xs").is_err());
    }

    #[test]
    fn api_fetch_defaults_to_x509() {
        let cli = Cli::try_parse_from(["spire-agent", "api", "fetch"]).unwrap();
        match cli.command {
            Some(Commands::Api(ApiArgs {
                command: ApiCommand::Fetch(FetchArgs { command: None }),
                ..
            })) => {}
            _ => panic!("unexpected parse result"),
        }
    }

    #[test]
    fn api_fetch_jwt_parses_audience_and_spiffe_id() {
        let cli = Cli::try_parse_from([
            "spire-agent",
            "api",
            "fetch",
            "jwt",
            "--audience",
            "example.org,spiffe.io",
            "--spiffeID",
            "spiffe://example.org/workload",
        ])
        .unwrap();

        match cli.command {
            Some(Commands::Api(ApiArgs {
                command:
                    ApiCommand::Fetch(FetchArgs {
                        command:
                            Some(FetchCommand::Jwt(FetchJwtArgs {
                                audience,
                                spiffe_id,
                            })),
                    }),
                ..
            })) => {
                assert_eq!(audience, vec!["example.org", "spiffe.io"]);
                assert_eq!(spiffe_id.as_deref(), Some("spiffe://example.org/workload"));
            }
            _ => panic!("unexpected parse result"),
        }
    }
}
