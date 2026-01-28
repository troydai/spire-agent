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
    #[command(subcommand)]
    command: ApiCommand,
}

#[derive(Subcommand)]
enum ApiCommand {
    Fetch(FetchArgs),
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
