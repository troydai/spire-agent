use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "spire-agent", version, about = "Agent CLI for Spire")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Help,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Help) | None => {
            if let Err(err) = Cli::command().print_long_help() {
                eprintln!("Failed to render help: {err}");
                std::process::exit(1);
            }
            println!();
        }
    }
}
