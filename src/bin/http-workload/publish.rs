use super::{cli::Args, generate::message};
use lapin::{
    BasicProperties, Confirmation, Connection, ConnectionProperties,
    options::{BasicPublishOptions, ConfirmSelectOptions},
};
use rand::{SeedableRng, rngs::StdRng};
use runnel::config::Endpoint;
use std::time::{Duration, Instant};

pub(crate) async fn run(
    args: Args,
    source: Endpoint,
    url: &str,
) -> Result<(usize, Duration), &'static str> {
    // Avoid including connection details or credentials in errors.
    let connection = Connection::connect(url, ConnectionProperties::default())
        .await
        .map_err(|_| "connection error")?;
    let channel = connection
        .create_channel()
        .await
        .map_err(|_| "channel error")?;
    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await
        .map_err(|_| "could not enable publisher confirms")?;
    let mut rng = StdRng::seed_from_u64(args.seed);
    let start = Instant::now();
    let mut sent = 0usize;
    loop {
        if args.count.is_some_and(|count| sent >= count.get())
            || args.duration.is_some_and(|duration| {
                start.elapsed().as_secs_f64() >= duration
            })
        {
            break;
        }
        let target = sent as f64 / args.rate;
        if args.duration.is_some_and(|duration| target >= duration) {
            break;
        }
        let delay = target - start.elapsed().as_secs_f64();
        if delay > 0.0 {
            tokio::time::sleep(
                Duration::try_from_secs_f64(delay)
                    .map_err(|_| "rate interval too large")?,
            )
            .await;
        }
        let number = sent + 1;
        let malformed = args
            .malformed_every
            .is_some_and(|n| number.is_multiple_of(n.get()));
        let incompatible = args
            .incompatible_every
            .is_some_and(|n| number.is_multiple_of(n.get()));
        let body = message(
            &mut rng,
            args.datasets[sent % args.datasets.len()],
            args.sizes[sent % args.sizes.len()].get(),
            malformed,
            incompatible,
        );
        let confirmation = channel
            .basic_publish(
                source.exchange.clone().into(),
                source.routing_key.clone().into(),
                BasicPublishOptions {
                    mandatory: true,
                    ..Default::default()
                },
                &body,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2),
            )
            .await
            .map_err(|_| "publish error")?
            .await
            .map_err(|_| "confirmation error")?;
        match confirmation {
            Confirmation::Ack(None) => sent += 1,
            Confirmation::Ack(Some(_)) | Confirmation::Nack(_) => {
                return Err("message was not routed or was rejected");
            }
            Confirmation::NotRequested => {
                return Err("publisher confirmation missing");
            }
        }
    }
    connection
        .close(200, "done".into())
        .await
        .map_err(|_| "connection close error")?;
    Ok((sent, start.elapsed()))
}
