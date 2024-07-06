use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use crm_send::{
    pb::{
        notification_client::NotificationClient, EmailMessage, InAppMessage, SendRequest,
        SmsMessage,
    },
    AppConfig, NotificationService,
};
use futures::StreamExt;
use tokio::time::sleep;
use tonic::{transport::Server, Request};

#[tokio::test]
async fn send_should_work() -> Result<()> {
    let addr = start_server().await?;
    let mut client = NotificationClient::connect(format!("http://{}", addr)).await?;

    let stream = tokio_stream::iter(vec![
        SendRequest {
            msg: Some(EmailMessage::fake().into()),
        },
        SendRequest {
            msg: Some(SmsMessage::fake().into()),
        },
        SendRequest {
            msg: Some(InAppMessage::fake().into()),
        },
    ]);

    let req = Request::new(stream);
    let response = client.send(req).await?.into_inner();

    let ret: Vec<_> = response.then(|res| async { res.unwrap() }).collect().await;

    assert_eq!(ret.len(), 3);
    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    // Start the server
    let conf = AppConfig::load()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], conf.server.port));

    let svc = NotificationService::new(conf).into_server();
    tokio::spawn(async move {
        let _ = Server::builder().add_service(svc).serve(addr).await;
    });

    sleep(Duration::from_micros(1)).await;
    Ok(addr)
}
