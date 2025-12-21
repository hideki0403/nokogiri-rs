use sentry::SentryFutureExt;
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod core;
mod resource;
mod server;

fn main() {
    let conf = &config::CONFIG;

    // Logging setup
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let level = conf
            .debug
            .as_ref()
            .and_then(|d| d.log_level.as_ref())
            .and_then(|s| s.parse::<LevelFilter>().ok())
            .unwrap_or(if cfg!(debug_assertions) { LevelFilter::DEBUG } else { LevelFilter::INFO });

        let filter_str = format!("{}={}", env!("CARGO_PKG_NAME").replace("-", "_"), level);
        tracing_subscriber::EnvFilter::new(filter_str)
    });

    // Sentry setup
    if core::sentry::is_sentry_enabled() {
        println!("Sentry logging is enabled");

        let _guard = sentry::init(sentry::ClientOptions {
            dsn: core::sentry::SENTRY_DSN.clone(),
            release: sentry::release_name!(),
            ..Default::default()
        });

        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(filter)
            .with(sentry::integrations::tracing::layer())
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    // Start server
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
        run().bind_hub(sentry::Hub::current()).await;
    });
}

async fn run() {
    server::listen().await;
}
