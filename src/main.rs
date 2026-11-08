use std::{path::PathBuf, time::Duration};

use runnel::{
    config::{self, AppConfig},
    pipeline::pipeline,
};

async fn run(config: AppConfig, source_url: String, sink_url: String) {
    loop {
        if pipeline(&config, &source_url, &sink_url).await.is_err() {
            eprintln!("pipeline stopped; retrying connection");
            tokio::time::sleep(Duration::from_secs(2)).await;
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
