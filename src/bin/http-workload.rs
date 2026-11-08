use std::{
    num::NonZeroUsize,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use clap::{ArgGroup, Parser, ValueEnum};
use fake::{
    Fake,
    faker::{
        address::en::StreetName, internet::en::SafeEmail, lorem::en::Sentence, name::en::Name,
    },
};
use lapin::{
    BasicProperties, Confirmation, Connection, ConnectionProperties,
    options::{BasicPublishOptions, ConfirmSelectOptions},
};
use rand::{SeedableRng, rngs::StdRng};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Dataset {
    Checkout,
    Profile,
    Search,
}

#[derive(Parser, Debug)]
#[command(about = "Publish synthetic HTTP logs to an existing RabbitMQ input")]
#[command(group(ArgGroup::new("limit").required(true).args(["count", "duration"])))]
struct Args {
    #[arg(long)]
    count: Option<NonZeroUsize>,
    #[arg(long, value_parser = positive_float)]
    duration: Option<f64>,
    #[arg(long, default_value = "100", value_parser = positive_float)]
    rate: f64,
    #[arg(long, num_args = 1.., default_value = "1024")]
    sizes: Vec<NonZeroUsize>,
    #[arg(long, num_args = 1.., default_values_t = [Dataset::Checkout, Dataset::Profile, Dataset::Search], value_enum)]
    datasets: Vec<Dataset>,
    #[arg(long)]
    malformed_every: Option<NonZeroUsize>,
    #[arg(long)]
    incompatible_every: Option<NonZeroUsize>,
    #[arg(long, default_value_t = 1)]
    seed: u64,
    #[arg(long, default_value = "amqp://guest:guest@localhost:5672/%2F")]
    url: String,
    #[arg(long, default_value = "")]
    exchange: String,
    #[arg(long, default_value = "runnel.input")]
    routing_key: String,
}

fn positive_float(s: &str) -> Result<f64, String> {
    let value: f64 = s.parse().map_err(|_| "expected a number")?;
    if value.is_finite() && value > 0.0 && Duration::try_from_secs_f64(value).is_ok() {
        Ok(value)
    } else {
        Err("must be finite, positive, and representable as a duration".into())
    }
}

fn message(
    rng: &mut StdRng,
    dataset: Dataset,
    size: usize,
    malformed: bool,
    incompatible: bool,
) -> Vec<u8> {
    if malformed {
        return b"{\"request\":".to_vec();
    }
    let email: String = SafeEmail().fake_with_rng(rng);
    let name: String = Name().fake_with_rng(rng);
    let sentence: String = Sentence(3..6).fake_with_rng(rng);
    let (method, path, body, status, response) = match dataset {
        Dataset::Checkout => (
            "POST",
            "/api/orders",
            json!({"email":email,"card":"4111111111111111","items":[{"sku":"SKU-1234","quantity":2}]}),
            201,
            json!({"order_id":sentence,"total":"49.99"}),
        ),
        Dataset::Profile => (
            "PUT",
            "/api/profile",
            json!({"name":name,"email":email,"address":StreetName().fake_with_rng::<String, _>(rng)}),
            200,
            json!({"updated":true}),
        ),
        Dataset::Search => (
            "POST",
            "/api/search",
            json!({"query":sentence,"filters":{"category":"books"}}),
            200,
            json!({"results":[{"id":1,"title":name}]}),
        ),
    };
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let request = if incompatible {
        json!("not an object")
    } else {
        json!({"method":method,"path":path,"headers":{"content-type":"application/json","authorization":format!("Bearer {email}")},"body":body,"remote_ip":"192.0.2.1"})
    };
    let mut record = json!({"timestamp_unix":timestamp,"request":request,"response":{"status":status,"headers":{"content-type":"application/json"},"body":response},"duration_ms":42,"padding":""});
    let mut encoded = serde_json::to_vec(&record).expect("JSON value serializes");
    if encoded.len() < size {
        record["padding"] = Value::String("x".repeat(size - encoded.len()));
        encoded = serde_json::to_vec(&record).expect("JSON value serializes");
    }
    encoded
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match run(args).await {
        Ok((sent, elapsed)) => {
            println!("Published {sent} messages in {:.2}s", elapsed.as_secs_f64())
        }
        Err(error) => {
            eprintln!("RabbitMQ publication failed: {error}");
            std::process::exit(1);
        }
    }
}

async fn run(args: Args) -> Result<(usize, Duration), &'static str> {
    // Avoid including connection details or credentials in errors.
    let connection = Connection::connect(&args.url, ConnectionProperties::default())
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
            || args
                .duration
                .is_some_and(|duration| start.elapsed().as_secs_f64() >= duration)
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
                Duration::try_from_secs_f64(delay).map_err(|_| "rate interval too large")?,
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
                args.exchange.clone().into(),
                args.routing_key.clone().into(),
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
            Confirmation::NotRequested => return Err("publisher confirmation missing"),
        }
    }
    connection
        .close(200, "done".into())
        .await
        .map_err(|_| "connection close error")?;
    Ok((sent, start.elapsed()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn datasets_have_http_details_and_requested_size() {
        let mut rng = StdRng::seed_from_u64(1);
        for dataset in [Dataset::Checkout, Dataset::Profile, Dataset::Search] {
            let bytes = message(&mut rng, dataset, 8192, false, false);
            assert!(bytes.len() >= 8192);
            let event: Value = serde_json::from_slice(&bytes).unwrap();
            assert!(event["request"]["headers"].is_object());
            assert!(event["request"]["body"].is_object());
            assert!(event["response"]["headers"].is_object());
            assert!(event["response"]["body"].is_object());
        }
    }

    #[test]
    fn invalid_cases_are_distinct() {
        let mut rng = StdRng::seed_from_u64(1);
        assert!(
            serde_json::from_slice::<Value>(&message(
                &mut rng,
                Dataset::Checkout,
                1024,
                true,
                true
            ))
            .is_err()
        );
        let event: Value =
            serde_json::from_slice(&message(&mut rng, Dataset::Checkout, 1024, false, true))
                .unwrap();
        assert!(event["request"].is_string());
    }

    #[test]
    fn cli_validates_limits_and_positive_values() {
        for args in [
            vec!["tool"],
            vec!["tool", "--count", "0"],
            vec!["tool", "--duration", "NaN"],
            vec!["tool", "--count", "1", "--rate", "0"],
            vec!["tool", "--count", "1", "--sizes", "0"],
            vec!["tool", "--count", "1", "--duration", "1"],
        ] {
            assert!(Args::try_parse_from(args).is_err());
        }
    }
}
