mod commands;
mod fetch_x509;
mod healthcheck;
mod workload;

#[tokio::main]
async fn main() {
    commands::run().await;
}
