use serde::Deserialize;
use std::{fs, path::Path};

/// RabbitMQ settings shared by the source and sink.
#[derive(Debug, Deserialize)]
pub struct Endpoint {
    pub url_env: String,
    pub exchange: String,
    pub routing_key: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub source: Endpoint,
    pub sink: Endpoint,
}

#[derive(Deserialize)]
struct SourceConfig {
    source: Endpoint,
}

/// Load both endpoints for a pipeline.
pub fn load(path: &Path) -> Result<Config, String> {
    let contents = read(path)?;
    let config: Config = toml::from_str(&contents)
        .map_err(|_| format!("invalid configuration in {}", path.display()))?;
    config.source.validate("source")?;
    config.sink.validate("sink")?;
    Ok(config)
}

/// Read only the source section, leaving sink validation to its consumer.
pub fn load_source(path: &Path) -> Result<Endpoint, String> {
    let contents = read(path)?;
    let config: SourceConfig = toml::from_str(&contents).map_err(|_| {
        format!("invalid source configuration in {}", path.display())
    })?;
    config.source.validate("source")?;
    Ok(config.source)
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| {
        format!("cannot read config {}: {error}", path.display())
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

#[cfg(test)]
#[path = "../tests/unit/config.rs"]
mod tests;
