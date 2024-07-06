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

#[cfg(feature = "test_utils")]
mod test_utils {
    use fake::{
        faker::{internet::en::SafeEmail, phone_number::en::PhoneNumber},
        Fake,
    };
    use uuid::Uuid;

    use crate::pb::{EmailMessage, InAppMessage, SmsMessage};

    impl EmailMessage {
        pub fn fake() -> Self {
            EmailMessage {
                message_id: Uuid::new_v4().to_string(),
                sender: SafeEmail().fake(),
                recipients: vec![SafeEmail().fake()],
                subject: "Test Subject".to_string(),
                body: "Test Body".to_string(),
            }
        }
    }

    impl SmsMessage {
        pub fn fake() -> Self {
            SmsMessage {
                message_id: Uuid::new_v4().to_string(),
                sender: PhoneNumber().fake(),
                recipients: vec![PhoneNumber().fake()],
                body: "Test Body".to_string(),
            }
        }
    }

    impl InAppMessage {
        pub fn fake() -> Self {
            InAppMessage {
                message_id: Uuid::new_v4().to_string(),
                device_id: Uuid::new_v4().to_string(),
                title: "Test Title".to_string(),
                body: "Test Body".to_string(),
            }
        }
    }
}
