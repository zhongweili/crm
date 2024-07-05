use tonic::Status;

use crate::{
    pb::{send_request::Msg, EmailMessage, SendRequest, SendResponse},
    NotificationService,
};

use super::{to_ts, Sender};

impl Sender for EmailMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        svc.sender
            .send(Msg::Email(self))
            .await
            .map_err(|e| Status::internal(format!("Failed to send email: {}", e)))?;

        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<EmailMessage> for Msg {
    fn from(value: EmailMessage) -> Self {
        Msg::Email(value)
    }
}

impl From<EmailMessage> for SendRequest {
    fn from(value: EmailMessage) -> Self {
        SendRequest {
            msg: Some(value.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{faker::internet::en::SafeEmail, Fake};
    use uuid::Uuid;

    use crate::pb::EmailMessage;

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
}
