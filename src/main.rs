use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "spire-agent", version, about = "Agent CLI for Spire")]
struct Cli {
    // No subcommands yet; keep the CLI minimal.
}

fn main() {
    let _cli = Cli::parse();
    if let Err(err) = Cli::command().print_long_help() {
        eprintln!("Failed to render help: {err}");
        std::process::exit(1);
    }
    println!();
}
