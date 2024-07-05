use anyhow::Result;

use crm_metadata::{AppConfig, MetadataService};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load().expect("MetadataService load failed");
    let addr = config.server.port;
    let addr = format!("[::1]:{}", addr).parse().unwrap();
    println!("MetadataService listening on {}", addr);

    let svc = MetadataService::new(config).into_server();
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
