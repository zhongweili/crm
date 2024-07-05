use tonic::Status;

use crate::{
    pb::{send_request::Msg, InAppMessage, SendRequest, SendResponse},
    NotificationService,
};

use super::{to_ts, Sender};

impl Sender for InAppMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        svc.sender
            .send(Msg::InApp(self))
            .await
            .map_err(|e| Status::internal(format!("Failed to send in-app message: {}", e)))?;
        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<InAppMessage> for Msg {
    fn from(msg: InAppMessage) -> Self {
        Msg::InApp(msg)
    }
}

impl From<InAppMessage> for SendRequest {
    fn from(msg: InAppMessage) -> Self {
        SendRequest {
            msg: Some(msg.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::pb::InAppMessage;

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
