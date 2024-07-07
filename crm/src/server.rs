use std::mem;

use anyhow::Result;
use crm::{config::AppConfig, CrmService};
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let mut conf = AppConfig::load().expect("Failed to load configuration");

    let tls = mem::take(&mut conf.server.tls);

    let addr = conf.server.port;
    let addr = format!("[::1]:{}", addr).parse().unwrap();
    info!("Server is listening on {}", addr);

    let server = CrmService::try_new(conf).await?.into_server();

    if let Some(tls) = tls {
        let identity = Identity::from_pem(tls.cert, tls.key);
        Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(server)
            .serve(addr)
            .await?;
    } else {
        Server::builder().add_service(server).serve(addr).await?;
    }
    Ok(())
}
