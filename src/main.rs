use lapin::{
    BasicProperties, Confirmation, Connection, ConnectionProperties, options::*,
};
use runnel::{
    config,
    pipeline::{self, Error, Sink, Source},
};
use std::{path::PathBuf, time::Duration};

struct RabbitSource {
    channel: lapin::Channel,
    queue: String,
}
impl Source for RabbitSource {
    type Delivery = lapin::message::BasicGetMessage;
    async fn receive(&self) -> Result<Option<Self::Delivery>, Error> {
        self.channel
            .basic_get(self.queue.clone().into(), BasicGetOptions::default())
            .await
            .map_err(|_| Error::Receive)
    }
    fn body(delivery: &Self::Delivery) -> &[u8] {
        &delivery.data
    }
    async fn settle(delivery: Self::Delivery) -> Result<(), Error> {
        delivery
            .ack(BasicAckOptions::default())
            .await
            .map(|_| ())
            .map_err(|_| Error::Settle)
    }
}
struct RabbitSink {
    channel: lapin::Channel,
    exchange: String,
    key: String,
}
impl Sink for RabbitSink {
    async fn publish(&self, body: &[u8]) -> Result<(), Error> {
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
            .map_err(|_| Error::Publish)?
            .await
            .map_err(|_| Error::Confirm)?;
        match confirmation {
            Confirmation::Ack(None) => Ok(()),
            _ => Err(Error::Unroutable),
        }
    }
}
async fn run(config: config::Config, source_url: String, sink_url: String) {
    loop {
        if let Err(error) =
            connect_and_run(&config, &source_url, &sink_url).await
        {
            eprintln!("{error}; retrying connection");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}
async fn connect_and_run(
    config: &config::Config,
    source_url: &str,
    sink_url: &str,
) -> Result<(), Error> {
    let source_connection =
        Connection::connect(source_url, ConnectionProperties::default())
            .await
            .map_err(|_| Error::SourceConnect)?;
    let sink_connection =
        Connection::connect(sink_url, ConnectionProperties::default())
            .await
            .map_err(|_| Error::SinkConnect)?;
    let source = RabbitSource {
        channel: source_connection
            .create_channel()
            .await
            .map_err(|_| Error::SourceChannel)?,
        queue: config.source.routing_key.clone(),
    };
    let sink_channel = sink_connection
        .create_channel()
        .await
        .map_err(|_| Error::SinkChannel)?;
    sink_channel
        .confirm_select(ConfirmSelectOptions::default())
        .await
        .map_err(|_| Error::ConfirmSelect)?;
    let sink = RabbitSink {
        channel: sink_channel,
        exchange: config.sink.exchange.clone(),
        key: config.sink.routing_key.clone(),
    };
    let path: Vec<&str> =
        config.redaction.path.iter().map(String::as_str).collect();
    loop {
        match pipeline::process_one(&source, &sink, &path).await {
            Ok(true) => {}
            Ok(false) => tokio::time::sleep(Duration::from_millis(100)).await,
            Err(Error::Sanitize) => {
                eprintln!("event could not be sanitized; pipeline paused");
                std::future::pending::<()>().await;
            }
            Err(error) => return Err(error),
        }
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
