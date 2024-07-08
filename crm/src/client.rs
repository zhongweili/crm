use anyhow::Result;
use crm::pb::{
    crm_client::CrmClient, RecallRequestBuilder, RemindRequestBuilder, WelcomeRequestBuilder,
};
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig},
    Request,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let pem = include_str!("../../fixtures/rootCA.pem");
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(pem))
        .domain_name("localhost");
    let channel = Channel::from_static("https://[::1]:50000")
        .tls_config(tls)?
        .connect()
        .await?;

    let token = include_str!("../../fixtures/token").trim();
    let token: MetadataValue<_> = format!("Bearer {}", token).parse()?;

    let mut client = CrmClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });

    let req_of_welcome = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(98u32)
        .content_ids([1u32, 2, 3])
        .build()?;

    let resp_of_welcome = client
        .welcome(Request::new(req_of_welcome))
        .await?
        .into_inner();
    println!("Response of welcome: {:?}", resp_of_welcome);

    let req_of_recall = RecallRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .last_visit_interval(37u32)
        .content_ids([1u32, 2, 3])
        .build()?;

    let resp_of_recall = client
        .recall(Request::new(req_of_recall))
        .await?
        .into_inner();
    println!("Response of recall: {:?}", resp_of_recall);

    let req_of_remind = RemindRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .last_visit_interval(37u32)
        .build()?;

    let resp_of_remind = client
        .remind(Request::new(req_of_remind))
        .await?
        .into_inner();
    println!("Response of remind: {:?}", resp_of_remind);

    Ok(())
}
