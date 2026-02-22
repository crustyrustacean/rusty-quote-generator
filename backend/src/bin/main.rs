// src/main.rs

// dependencies
use rusty_quote_generator_server::configuration::get_configuration;
use rusty_quote_generator_server::startup::Application;
use rusty_quote_generator_server::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("actix-web-starter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    application.run_until_stopped().await?;

    Ok(())
}
