use tracing_subscriber::filter::LevelFilter;

mod core;
mod config;
mod resource;
mod server;

#[tokio::main]
async fn main() {
    let conf = &config::CONFIG;

    // Logging setup
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let level = conf
        .debug
        .as_ref()
        .and_then(|d| d.log_level.as_ref())
        .and_then(|s| s.parse::<LevelFilter>().ok())
        .unwrap_or(if cfg!(debug_assertions) { LevelFilter::DEBUG } else { LevelFilter::INFO });

        let filter_str = format!("{},selectors=off,html5ever=off", level);
        tracing_subscriber::EnvFilter::new(filter_str)
    });

    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Sentry setup
    if let Some(sentry) = &conf.sentry &&
        let Some(dsn) = &sentry.dsn &&
        let Ok(sentry_dsn) = sentry::IntoDsn::into_dsn(dsn.clone())
    {
        tracing::info!("Sentry logging is enabled");
        let _guard = sentry::init(sentry::ClientOptions {
            dsn: sentry_dsn,
            release: sentry::release_name!(),
            ..Default::default()
        });
    }

    // Start server
    server::listen().await;
}
