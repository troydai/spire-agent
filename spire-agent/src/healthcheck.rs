use std::time::Duration;

use anyhow::{Context, Result};
use crate::rpc::{connect_channel, connect_workload_client};
use crate::workload::X509svidRequest;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(1);

pub async fn healthcheck(socket_path: &str, shallow: bool, verbose: bool) -> Result<()> {
    check_workload_health(socket_path, DEFAULT_TIMEOUT).await?;
    if verbose {
        println!("Workload API health check: ok");
    }

    if !shallow {
        if verbose {
            println!("Workload API X509-SVID check: starting");
        }
        check_x509_svid(socket_path, DEFAULT_TIMEOUT).await?;
        if verbose {
            println!("Workload API X509-SVID check: ok");
        }
    }

    if verbose {
        println!("Agent health: ok");
    }
    println!("Agent is healthy.");
    Ok(())
}

async fn check_workload_health(socket_path: &str, timeout: Duration) -> Result<()> {
    tokio::time::timeout(timeout, connect_channel(socket_path))
        .await
        .context("request timed out")?
        .context("health check request failed")?;
    Ok(())
}

async fn check_x509_svid(socket_path: &str, timeout: Duration) -> Result<()> {
    let mut client = connect_workload_client(socket_path).await?;
    let request = tonic::Request::new(X509svidRequest {});

    let response = tokio::time::timeout(timeout, client.fetch_x509svid(request))
        .await
        .context("request timed out")?
        .context("failed to fetch x509 svid")?;

    let mut stream = response.into_inner();
    tokio::time::timeout(timeout, stream.message())
        .await
        .context("timed out waiting for response")?
        .context("failed to receive message")?
        .context("empty response from server")?;

    Ok(())
}