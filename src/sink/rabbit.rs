use lapin::{BasicProperties, Confirmation, options::BasicPublishOptions};

use crate::sink::Sink;

pub struct RabbitSink {
    pub channel: lapin::Channel,
    pub exchange: String,
    pub key: String,
}

impl Sink for RabbitSink {
    async fn publish(&self, body: &[u8]) -> Result<(), ()> {
        let confirmation = self
            .channel
            .basic_publish(
                self.exchange.clone().into(),
                self.key.clone().into(),
                BasicPublishOptions {
                    mandatory: true,
                    ..Default::default()
                },
                body,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2),
            )
            .await
            .map_err(|_| ())?
            .await
            .map_err(|_| ())?;

        match confirmation {
            Confirmation::Ack(None) => Ok(()),
            _ => Err(()),
        }
    }
}
