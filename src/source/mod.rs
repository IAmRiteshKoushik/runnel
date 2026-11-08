mod rabbit;

pub use rabbit::RabbitSource;

pub trait Source {
    type Delivery;

    fn body(delivery: &Self::Delivery) -> &[u8];

    async fn receive(&self) -> Result<Option<Self::Delivery>, ()>;
    async fn settle(delivery: Self::Delivery) -> Result<(), ()>;
}
