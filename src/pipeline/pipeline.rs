use std::time::Duration;

use lapin::{Connection, ConnectionProperties, options::ConfirmSelectOptions};

use crate::{
    config::AppConfig,
    pipeline,
    sink::{RabbitSink, Sink},
    source::{RabbitSource, Source},
};

pub async fn pipeline(
    config: &AppConfig,
    source_url: &str,
    sink_url: &str,
) -> Result<(), ()> {
    let source_connection =
        Connection::connect(source_url, ConnectionProperties::default())
            .await
            .map_err(|_| ())?;
    let source = RabbitSource {
        channel: source_connection.create_channel().await.map_err(|_| ())?,
        queue: config.source.routing_key.clone(),
    };

    let sink_connection =
        Connection::connect(sink_url, ConnectionProperties::default())
            .await
            .map_err(|_| ())?;
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

    // Read redaction paths
    let path: Vec<&str> =
        config.redaction.path.iter().map(String::as_str).collect();

    loop {
        let Some(delivery) = source.receive().await? else {
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        };
        let body = match pipeline::redact::redact(
            RabbitSource::body(&delivery),
            &path,
        ) {
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
