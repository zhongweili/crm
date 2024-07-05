use tonic::Status;

use crate::{
    pb::{send_request::Msg, SendRequest, SendResponse, SmsMessage},
    NotificationService,
};

use super::{to_ts, Sender};

impl Sender for SmsMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        svc.sender
            .send(Msg::Sms(self))
            .await
            .map_err(|e| Status::internal(format!("Failed to send SMS: {}", e)))?;
        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<SmsMessage> for Msg {
    fn from(sms: SmsMessage) -> Self {
        Msg::Sms(sms)
    }
}

impl From<SmsMessage> for SendRequest {
    fn from(sms: SmsMessage) -> Self {
        SendRequest {
            msg: Some(sms.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{faker::phone_number::en::PhoneNumber, Fake};
    use uuid::Uuid;

    use crate::pb::SmsMessage;

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
}
