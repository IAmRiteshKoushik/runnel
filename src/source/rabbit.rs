use lapin::options::{BasicAckOptions, BasicGetOptions};

use crate::source::Source;

pub struct RabbitSource {
    pub channel: lapin::Channel,
    pub queue: String,
}

impl Source for RabbitSource {
    type Delivery = lapin::message::BasicGetMessage;

    fn body(delivery: &Self::Delivery) -> &[u8] {
        &delivery.data
    }

    async fn receive(&self) -> Result<Option<Self::Delivery>, ()> {
        self.channel
            .basic_get(self.queue.clone().into(), BasicGetOptions::default())
            .await
            .map_err(|_| ())
    }

    async fn settle(delivery: Self::Delivery) -> Result<(), ()> {
        delivery
            .ack(BasicAckOptions::default())
            .await
            .map(|_| ())
            .map_err(|_| ())
    }
}
