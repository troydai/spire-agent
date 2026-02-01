use std::time::Duration;

use anyhow::{Context, Result};
use hyper_util::rt::TokioIo;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint, Uri};
use tonic::Status;
use tower::service_fn;

use crate::workload::X509svidRequest;
use crate::workload::spiffe_workload_api_client::SpiffeWorkloadApiClient;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(1);
const SECURITY_HEADER_KEY: &str = "workload.spiffe.io";
const SECURITY_HEADER_VALUE: &str = "true";

type WorkloadClient = SpiffeWorkloadApiClient<
    tonic::service::interceptor::InterceptedService<
        Channel,
        fn(tonic::Request<()>) -> Result<tonic::Request<()>, Status>,
    >,
>;

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

async fn connect_channel(socket_path: &str) -> Result<Channel> {
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

    Ok(channel)
}

async fn connect_workload_client(socket_path: &str) -> Result<WorkloadClient> {
    let channel = connect_channel(socket_path).await?;
    Ok(SpiffeWorkloadApiClient::with_interceptor(
        channel,
        add_security_header,
    ))
}

fn add_security_header(mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
    insert_security_header(&mut request);
    Ok(request)
}

fn insert_security_header<T>(request: &mut tonic::Request<T>) {
    request.metadata_mut().insert(
        SECURITY_HEADER_KEY,
        MetadataValue::from_static(SECURITY_HEADER_VALUE),
    );
}
