mod rabbit;

pub use rabbit::RabbitSink;

pub trait Sink {
    async fn publish(&self, body: &[u8]) -> Result<(), ()>;
}
