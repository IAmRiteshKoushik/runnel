use lapin::{
    BasicProperties, Confirmation, Connection, ConnectionProperties, options::*,
};
use runnel::{config, sanitization};
use std::{path::PathBuf, time::Duration};

// A source owns settlement; a sink only reports whether publication succeeded.
trait Source {
    type Delivery;
    async fn receive(&self) -> Result<Option<Self::Delivery>, ()>;
    fn body(delivery: &Self::Delivery) -> &[u8];
    async fn settle(delivery: Self::Delivery) -> Result<(), ()>;
}
trait Sink {
    async fn publish(&self, body: &[u8]) -> Result<(), ()>;
}
struct RabbitSource {
    channel: lapin::Channel,
    queue: String,
}
impl Source for RabbitSource {
    type Delivery = lapin::message::BasicGetMessage;
    async fn receive(&self) -> Result<Option<Self::Delivery>, ()> {
        self.channel
            .basic_get(self.queue.clone().into(), BasicGetOptions::default())
            .await
            .map_err(|_| ())
    }
    fn body(delivery: &Self::Delivery) -> &[u8] {
        &delivery.data
    }
    async fn settle(delivery: Self::Delivery) -> Result<(), ()> {
        delivery
            .ack(BasicAckOptions::default())
            .await
            .map(|_| ())
            .map_err(|_| ())
    }
}
struct RabbitSink {
    channel: lapin::Channel,
    exchange: String,
    key: String,
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
async fn run(config: config::Config, source_url: String, sink_url: String) {
    loop {
        if pipeline(&config, &source_url, &sink_url).await.is_err() {
            eprintln!("pipeline stopped; retrying connection");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}
async fn pipeline(
    config: &config::Config,
    source_url: &str,
    sink_url: &str,
) -> Result<(), ()> {
    let source_connection =
        Connection::connect(source_url, ConnectionProperties::default())
            .await
            .map_err(|_| ())?;
    let sink_connection =
        Connection::connect(sink_url, ConnectionProperties::default())
            .await
            .map_err(|_| ())?;
    let source = RabbitSource {
        channel: source_connection.create_channel().await.map_err(|_| ())?,
        queue: config.source.routing_key.clone(),
    };
    let sink_channel =
        sink_connection.create_channel().await.map_err(|_| ())?;
    sink_channel
        .confirm_select(ConfirmSelectOptions::default())
        .await
        .map_err(|_| ())?;
    let sink = RabbitSink {
        channel: sink_channel,
        exchange: config.sink.exchange.clone(),
        key: config.sink.routing_key.clone(),
    };
    let path: Vec<&str> =
        config.redaction.path.iter().map(String::as_str).collect();
    loop {
        let Some(delivery) = source.receive().await? else {
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        };
        let body =
            match sanitization::redact(RabbitSource::body(&delivery), &path) {
                Ok(body) => body,
                Err(_) => {
                    // Keep the original unacknowledged until an operator intervenes.
                    eprintln!("event could not be sanitized; pipeline paused");
                    std::future::pending::<()>().await;
                    unreachable!();
                }
            };
        sink.publish(&body).await?;
        RabbitSource::settle(delivery).await?;
    }
}
#[tokio::main]
async fn main() {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config.toml"));
    let result = config::load(&path).and_then(|config| {
        Ok((config.source.url()?, config.sink.url()?, config))
    });
    match result {
        Ok((source_url, sink_url, config)) => {
            run(config, source_url, sink_url).await
        }
        Err(error) => {
            eprintln!("configuration error: {error}");
            std::process::exit(1);
        }
    }
}
