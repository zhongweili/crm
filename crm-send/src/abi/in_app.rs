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
