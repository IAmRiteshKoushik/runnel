use std::path::Path;

use config::{Config, File, FileFormat};
use serde::Deserialize;

use crate::config::{endpoint::Endpoint, redact::Redaction};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub source: Endpoint,
    pub sink: Endpoint,
    pub redaction: Redaction,
}

impl AppConfig {
    fn validate(&self) -> Result<(), String> {
        self.source.validate("source")?;
        self.sink.validate("sink")?;
        if !self.source.exchange.is_empty() {
            return Err(
                "source exchange must be empty; routing_key names the queue"
                    .into(),
            );
        }
        if self.redaction.path.is_empty()
            || self.redaction.path.iter().any(|p| p.trim().is_empty())
        {
            return Err(
                "redaction path must contain nonempty object keys".into()
            );
        }
        Ok(())
    }
}

pub fn load(path: &Path) -> Result<AppConfig, String> {
    let config: AppConfig = read(path)?
        .try_deserialize()
        .map_err(|_| format!("invalid configuration in {}", path.display()))?;

    config.validate()?;
    Ok(config)
}

fn read(path: &Path) -> Result<config::Config, String> {
    Config::builder()
        .add_source(File::from(path).format(FileFormat::Toml))
        .build()
        .map_err(|error| {
            format!("cannot load config {}: {error}", path.display())
        })
}
