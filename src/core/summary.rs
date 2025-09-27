use once_cell::sync::Lazy;
use crate::config::CONFIG;

pub mod handler;
pub mod def;
pub mod selector;
pub mod utility;
pub mod summarize;

static ACTIVE_HANDLERS: Lazy<Vec<&'static dyn def::SummalyHandler>> =
    Lazy::new(|| {
        handler::HANDLERS
            .iter()
            .filter(|handler| !CONFIG.plugins.disabled.contains(&handler.id().to_string()))
            .map(|handler| *handler)
            .collect()
    });

pub async fn summary(url: &url::Url) -> Option<def::SummaryResult> {
    for handler in ACTIVE_HANDLERS.iter() {
        if handler.test(url) {
            tracing::debug!("Using handler: {}", handler.id());
            return handler.summarize(url).await;
        }
    }
    None
}
