use clap::{CommandFactory, Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Api(ApiArgs {
            command:
                ApiCommand::Fetch(FetchArgs {
                    command: FetchCommand::X509,
                }),
            ..
        })) => {
            // Intentionally no-op for now.
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
