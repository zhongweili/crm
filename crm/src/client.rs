use anyhow::{Ok, Result};
use crm::pb::{user_service_client::UserServiceClient, CreateUserRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    let dst = "http://[::1]:50051";
    let mut client = UserServiceClient::connect(dst).await?;

    let request = Request::new(CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    });

    let response = client.create_user(request).await?;
    println!("User created: {:?}", response.into_inner());

    Ok(())
}
