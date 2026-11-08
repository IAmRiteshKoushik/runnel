mod cli;
mod generate;
mod publish;

use clap::Parser;
use cli::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let source =
        runnel::config::load_source(&args.config).unwrap_or_else(|error| {
            eprintln!("Configuration error: {error}");
            std::process::exit(1);
        });
    let url = source.url().unwrap_or_else(|error| {
        eprintln!("Configuration error: {error}");
        std::process::exit(1);
    });
    match publish::run(args, source, &url).await {
        Ok((sent, elapsed)) => {
            println!(
                "Published {sent} messages in {:.2}s",
                elapsed.as_secs_f64()
            )
        }
        Err(error) => {
            eprintln!("RabbitMQ publication failed: {error}");
            std::process::exit(1);
        }
    }
}
