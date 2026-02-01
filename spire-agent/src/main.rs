mod commands;
mod fetch_x509;
mod healthcheck;
mod rpc;
mod workload;

#[tokio::main]
async fn main() {
    commands::run().await;
}
