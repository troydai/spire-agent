use std::time::Duration;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

use crate::fetch_x509::fetch_x509;

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
        long = "socket-path",
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

pub async fn run() {
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
            write,
            ..
        })) => {
            let timeout = match parse_duration(&timeout) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error parsing timeout: {e}");
                    std::process::exit(1);
                }
            };

            if let Err(e) = fetch_x509(&socket_path, timeout, silent, write.as_deref()).await {
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
