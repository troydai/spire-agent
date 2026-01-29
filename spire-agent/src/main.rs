mod commands;
mod fetch_x509;
mod workload;

#[tokio::main]
async fn main() {
    commands::run().await;
}
