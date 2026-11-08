use config::{Config, FileFormat};
use serde::Deserialize;
use std::path::Path;

/// RabbitMQ settings shared by the source and sink.
#[derive(Debug, Deserialize)]
pub struct Endpoint {
    pub url_env: String,
    pub exchange: String,
    pub routing_key: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub source: Endpoint,
    pub sink: Endpoint,
    pub redaction: Redaction,
}

#[derive(Debug, Deserialize)]
pub struct Redaction {
    pub path: Vec<String>,
}

#[derive(Deserialize)]
struct SourceConfig {
    source: Endpoint,
}

/// Load both endpoints for a pipeline.
pub fn load(path: &Path) -> Result<AppConfig, String> {
    let config: AppConfig = read(path)?
        .try_deserialize()
        .map_err(|_| format!("invalid configuration in {}", path.display()))?;
    config.source.validate("source")?;
    config.sink.validate("sink")?;
    if !config.source.exchange.is_empty() {
        return Err(
            "source exchange must be empty; routing_key names the queue".into(),
        );
    }
    if config.redaction.path.is_empty()
        || config.redaction.path.iter().any(|p| p.trim().is_empty())
    {
        return Err("redaction path must contain nonempty object keys".into());
    }
    Ok(config)
}

/// Read only the source section, leaving sink validation to its consumer.
pub fn load_source(path: &Path) -> Result<Endpoint, String> {
    let config: SourceConfig = read(path)?.try_deserialize().map_err(|_| {
        format!("invalid source configuration in {}", path.display())
    })?;
    config.source.validate("source")?;
    Ok(config.source)
}

fn read(path: &Path) -> Result<config::Config, String> {
    Config::builder()
        .add_source(config::File::from(path).format(FileFormat::Toml))
        .build()
        .map_err(|error| {
            format!("cannot load config {}: {error}", path.display())
        })
}

impl Endpoint {
    fn validate(&self, section: &str) -> Result<(), String> {
        if self.url_env.trim().is_empty() || self.routing_key.trim().is_empty()
        {
            return Err(format!(
                "{section} url_env and routing_key must not be empty"
            ));
        }
        Ok(())
    }
    /// Resolve the broker URL without including its value in diagnostics.
    pub fn url(&self) -> Result<String, String> {
        std::env::var(&self.url_env)
            .ok()
            .filter(|url| !url.trim().is_empty())
            .ok_or_else(|| {
                format!(
                    "environment variable {} is missing or empty",
                    self.url_env
                )
            })
    }
}
