mod api;
mod core;
use api::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start_api_server().await;
    Ok(())
}
