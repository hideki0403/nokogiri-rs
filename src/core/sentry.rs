use crate::config;
use once_cell::sync::Lazy;
use sentry::types::Dsn;

pub static SENTRY_DSN: Lazy<Option<Dsn>> = Lazy::new(|| {
    let conf = &config::CONFIG;
    if let Some(sentry) = &conf.sentry &&
        let Some(dsn) = &sentry.dsn &&
        let Ok(sentry_dsn) = sentry::IntoDsn::into_dsn(dsn.clone())
    {
        sentry_dsn
    } else {
        None
    }
});

pub fn is_sentry_enabled() -> bool {
    SENTRY_DSN.is_some()
}
