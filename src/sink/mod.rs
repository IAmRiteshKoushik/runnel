mod rabbit;

pub use rabbit::RabbitSink;

pub trait Sink {
    fn publish(
        &self,
        body: &[u8],
    ) -> impl Future<Output = Result<(), ()>> + Send;
}
