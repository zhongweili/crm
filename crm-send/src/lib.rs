use std::{pin::Pin, sync::Arc};

pub use config::AppConfig;
use futures::Stream;
use pb::{notification_server::Notification, send_request::Msg, SendRequest, SendResponse};

use tokio::sync::mpsc::Sender;
use tonic::{async_trait, Request, Response, Status, Streaming};

mod abi;
mod config;
pub mod pb;

#[allow(unused)]
#[derive(Clone)]
pub struct NotificationService {
    inner: Arc<NotificationServiceInner>,
}

#[allow(dead_code)]
pub struct NotificationServiceInner {
    config: AppConfig,
    sender: Sender<Msg>,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<SendResponse, Status>> + Send>>;

#[async_trait]
impl Notification for NotificationService {
    type SendStream = ResponseStream;

    async fn send(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> ServiceResult<ResponseStream> {
        let stream = request.into_inner();
        self.send(stream).await
    }
}
