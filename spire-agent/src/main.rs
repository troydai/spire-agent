mod commands;
mod fetch_jwt;
mod fetch_x509;
mod healthcheck;
mod rpc;
mod grpc;

#[tokio::main]
async fn main() {
    commands::run().await;
}
