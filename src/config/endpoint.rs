use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Endpoint {
    pub url_env: String,
    pub exchange: String,
    pub routing_key: String,
}

impl Endpoint {
    pub fn validate(&self, section: &str) -> Result<(), String> {
        if self.url_env.trim().is_empty() || self.routing_key.trim().is_empty()
        {
            return Err(format!(
                "{section} url_env and routing_key must not be empty"
            ));
        }
        Ok(())
    }

    pub fn url(&self) -> Result<String, String> {
        env::var(&self.url_env)
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
