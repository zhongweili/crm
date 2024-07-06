use anyhow::Result;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use user_stat::UserStatsService;

use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let addr = "[::1]:50051".parse()?;

    info!("UserService listening on {}", addr);

    let svc = UserStatsService::new().await.into_server();
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
