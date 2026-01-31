use anyhow::Result;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tokio_stream::Stream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::svid::{SvidConfig, SvidGenerator};
use crate::workload;

use workload::spiffe_workload_api_server::SpiffeWorkloadApi;
use workload::spiffe_workload_api_server::SpiffeWorkloadApiServer;
use workload::{
    JwtBundlesRequest, JwtBundlesResponse, JwtsvidRequest, JwtsvidResponse, ValidateJwtsvidRequest,
    ValidateJwtsvidResponse, X509BundlesRequest, X509BundlesResponse, X509svidRequest,
    X509svidResponse,
};

pub async fn start_server(socket_path: &Path, rotation_interval: Duration) -> Result<()> {
    let uds = UnixListener::bind(socket_path)?;
    let uds_stream = UnixListenerStream::new(uds);
    let service = MockWorkloadApi::with_rotation_interval(rotation_interval);

    Server::builder()
        .add_service(SpiffeWorkloadApiServer::with_interceptor(
            service,
            verify_security_header,
        ))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}

const SECURITY_HEADER_KEY: &str = "workload.spiffe.io";
const SECURITY_HEADER_VALUE: &str = "true";

fn verify_security_header(request: Request<()>) -> Result<Request<()>, Status> {
    let values: Vec<_> = request
        .metadata()
        .get_all(SECURITY_HEADER_KEY)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect();

    if values.len() == 1 && values[0] == SECURITY_HEADER_VALUE {
        return Ok(request);
    }

    Err(Status::invalid_argument(
        "security header missing from request",
    ))
}

struct MockWorkloadApi {
    svid_generator: Arc<SvidGenerator>,
    rotation_interval: Duration,
}

impl MockWorkloadApi {
    pub fn with_rotation_interval(rotation_interval: Duration) -> Self {
        let config = SvidConfig::default();
        Self {
            svid_generator: Arc::new(SvidGenerator::new(config)),
            rotation_interval,
        }
    }
}

#[tonic::async_trait]
impl SpiffeWorkloadApi for MockWorkloadApi {
    type FetchX509SVIDStream = Pin<Box<dyn Stream<Item = Result<X509svidResponse, Status>> + Send>>;

    type FetchX509BundlesStream =
        Pin<Box<dyn Stream<Item = Result<X509BundlesResponse, Status>> + Send>>;

    type FetchJWTBundlesStream =
        Pin<Box<dyn Stream<Item = Result<JwtBundlesResponse, Status>> + Send>>;

    async fn fetch_x509svid(
        &self,
        _request: Request<X509svidRequest>,
    ) -> Result<Response<Self::FetchX509SVIDStream>, Status> {
        println!("Received FetchX509SVID request");

        let svid_generator = Arc::clone(&self.svid_generator);
        let rotation_interval = self.rotation_interval;

        let stream = async_stream::stream! {
            loop {
                let x509_svid = svid_generator.generate_svid();
                let spiffe_id = x509_svid.spiffe_id.clone();

                let response = X509svidResponse {
                    svids: vec![x509_svid],
                    crl: vec![],
                    federated_bundles: std::collections::HashMap::new(),
                };

                println!("Sending X509SVID: {}", spiffe_id);
                yield Ok(response);

                // Wait for the rotation interval before sending the next certificate
                tokio::time::sleep(rotation_interval).await;
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }

    async fn fetch_x509_bundles(
        &self,
        _request: Request<X509BundlesRequest>,
    ) -> Result<Response<Self::FetchX509BundlesStream>, Status> {
        println!("Received FetchX509Bundles request");
        Err(Status::unimplemented("not implemented"))
    }

    async fn fetch_jwtsvid(
        &self,
        _request: Request<JwtsvidRequest>,
    ) -> Result<Response<JwtsvidResponse>, Status> {
        println!("Received FetchJWTSVID request");
        Err(Status::unimplemented("not implemented"))
    }

    async fn fetch_jwt_bundles(
        &self,
        _request: Request<JwtBundlesRequest>,
    ) -> Result<Response<Self::FetchJWTBundlesStream>, Status> {
        println!("Received FetchJWTBundles request");
        Err(Status::unimplemented("not implemented"))
    }

    async fn validate_jwtsvid(
        &self,
        _request: Request<ValidateJwtsvidRequest>,
    ) -> Result<Response<ValidateJwtsvidResponse>, Status> {
        println!("Received ValidateJWTSVID request");
        Err(Status::unimplemented("not implemented"))
    }
}
