mod server;
mod svid;
mod workload;

use anyhow::Result;
use clap::Parser;
use server::start_server;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author = "Troy Dai", version, about = "A mock implementation of the Workload API", long_about = None)]
struct Args {
    /// Unix Domain Socket path to listen on
    #[arg(short, long, default_value = "/tmp/agent.sock")]
    socket_path: PathBuf,
    /// X.509 SVID rotation interval in seconds
    #[arg(long = "x509-internal", default_value_t = 30)]
    x509_rotation_interval_seconds: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    ensure_path(&args.socket_path)?;

    println!(
        "SPIRE Agent Mock listening on uds://{}",
        &args.socket_path.display()
    );

    start_server(
        &args.socket_path,
        Duration::from_secs(args.x509_rotation_interval_seconds),
    )
    .await?;

    Ok(())
}

fn ensure_path(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}
