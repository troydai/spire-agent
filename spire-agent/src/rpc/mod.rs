use anyhow::{Context, Result};
use hyper_util::rt::TokioIo;
use tonic::Status;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use crate::grpc::spiffe_workload_api_client::SpiffeWorkloadApiClient;

const SECURITY_HEADER_KEY: &str = "workload.spiffe.io";
const SECURITY_HEADER_VALUE: &str = "true";

pub type WorkloadClient = SpiffeWorkloadApiClient<
    tonic::service::interceptor::InterceptedService<
        Channel,
        fn(tonic::Request<()>) -> Result<tonic::Request<()>, Status>,
    >,
>;

pub async fn connect_channel(socket_path: &str) -> Result<Channel> {
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

pub async fn connect_workload_client(socket_path: &str) -> Result<WorkloadClient> {
    let channel = connect_channel(socket_path).await?;
    Ok(SpiffeWorkloadApiClient::with_interceptor(
        channel,
        add_security_header,
    ))
}

// interceptor adding security header
fn add_security_header(mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
    request.metadata_mut().insert(
        SECURITY_HEADER_KEY,
        MetadataValue::from_static(SECURITY_HEADER_VALUE),
    );
    Ok(request)
}