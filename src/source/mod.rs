mod rabbit;

pub use rabbit::RabbitSource;

pub trait Source {
    type Delivery;

    fn body(delivery: &Self::Delivery) -> &[u8];
    fn receive(
        &self,
    ) -> impl Future<Output = Result<Option<Self::Delivery>, ()>> + Send;
    fn settle(
        delivery: Self::Delivery,
    ) -> impl Future<Output = Result<(), ()>> + Send;
}
