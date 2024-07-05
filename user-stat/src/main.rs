use anyhow::Result;
use user_stat::UserStatsService;

use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50051".parse()?;

    println!("UserService listening on {}", addr);

    let svc = UserStatsService::new().await.into_server();
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
