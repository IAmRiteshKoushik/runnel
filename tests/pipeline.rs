use runnel::pipeline::{self, Sink, Source};
use std::cell::RefCell;

struct Input(RefCell<Option<Vec<u8>>>);
impl Source for Input {
    type Delivery = Vec<u8>;
    async fn receive(&self) -> Result<Option<Self::Delivery>, pipeline::Error> {
        Ok(self.0.borrow_mut().take())
    }
    fn body(delivery: &Self::Delivery) -> &[u8] {
        delivery
    }
    async fn settle(_delivery: Self::Delivery) -> Result<(), pipeline::Error> {
        Ok(())
    }
}
#[derive(Default)]
struct Output(RefCell<Vec<Vec<u8>>>);
impl Sink for Output {
    async fn publish(&self, body: &[u8]) -> Result<(), pipeline::Error> {
        self.0.borrow_mut().push(body.to_vec());
        Ok(())
    }
}
#[tokio::test]
async fn alternative_connectors_deliver_sanitized_events() {
    let source = Input(RefCell::new(Some(
        br#"{"request":{"token":"secret"}}"#.to_vec(),
    )));
    let sink = Output::default();
    pipeline::process_one(&source, &sink, &["request", "token"])
        .await
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&sink.0.borrow()[0])
            .unwrap(),
        serde_json::json!({"request":{"token":"[REDACTED]"}})
    );
}
