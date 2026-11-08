use clap::{ArgGroup, Parser, ValueEnum};
use std::{num::NonZeroUsize, path::PathBuf, time::Duration};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum Dataset {
    Checkout,
    Profile,
    Search,
}

#[derive(Parser, Debug)]
#[command(about = "Publish synthetic HTTP logs to an existing RabbitMQ input")]
#[command(group(ArgGroup::new("limit").required(true).args(["count", "duration"])))]
pub(crate) struct Args {
    #[arg(long)]
    pub(crate) count: Option<NonZeroUsize>,
    #[arg(long, value_parser = positive_float)]
    pub(crate) duration: Option<f64>,
    #[arg(long, default_value = "100", value_parser = positive_float)]
    pub(crate) rate: f64,
    #[arg(long, num_args = 1.., default_value = "1024")]
    pub(crate) sizes: Vec<NonZeroUsize>,
    #[arg(long, num_args = 1.., default_values_t = [Dataset::Checkout, Dataset::Profile, Dataset::Search], value_enum)]
    pub(crate) datasets: Vec<Dataset>,
    #[arg(long)]
    pub(crate) malformed_every: Option<NonZeroUsize>,
    #[arg(long)]
    pub(crate) incompatible_every: Option<NonZeroUsize>,
    #[arg(long, default_value_t = 1)]
    pub(crate) seed: u64,
    #[arg(long, default_value = "config.toml")]
    pub(crate) config: PathBuf,
}

fn positive_float(s: &str) -> Result<f64, String> {
    let value: f64 = s.parse().map_err(|_| "expected a number")?;
    if value.is_finite()
        && value > 0.0
        && Duration::try_from_secs_f64(value).is_ok()
    {
        Ok(value)
    } else {
        Err("must be finite, positive, and representable as a duration".into())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/loggen_cli.rs"]
mod tests;
