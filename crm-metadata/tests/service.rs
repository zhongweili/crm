use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use crm_metadata::{
    pb::{metadata_client::MetadataClient, MaterializeRequest},
    AppConfig, MetadataService,
};
use futures::StreamExt;
use tokio::time::sleep;
use tonic::{transport::Server, Request};

#[tokio::test]
async fn materialize_should_work() -> Result<()> {
    let addr = start_server().await?;
    let mut client = MetadataClient::connect(format!("http://{addr}")).await?;

    let stream = tokio_stream::iter(vec![
        MaterializeRequest { id: 1 },
        MaterializeRequest { id: 2 },
        MaterializeRequest { id: 3 },
    ]);

    let req = Request::new(stream);
    let response = client.materialize(req).await?.into_inner();

    let ret: Vec<_> = response.then(|res| async { res.unwrap() }).collect().await;

    assert_eq!(ret.len(), 3);

    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    // Start the server
    let conf = AppConfig::load()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], conf.server.port));

    let svc = MetadataService::new(conf).into_server();
    tokio::spawn(async move {
        let _ = Server::builder().add_service(svc).serve(addr).await;
    });

    sleep(Duration::from_micros(1)).await;
    Ok(addr)
}
