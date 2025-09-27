use once_cell::sync::Lazy;
use crate::{config::CONFIG, core::cache};

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
    let cache = cache::get_summarize_cache(url.as_str());
    if let Some(cached) = cache {
        tracing::debug!("Cache hit for URL: {}", url);
        return serde_json::from_str(&cached).ok();
    }

    for handler in ACTIVE_HANDLERS.iter() {
        if handler.test(url) {
            tracing::debug!("Using handler: {}", handler.id());
            let summarized = handler.summarize(url).await?;
            let serialized = serde_json::to_string(&summarized.summary).ok()?;
            cache::set_summarize_cache(url.as_str(), &serialized, &summarized.cache_ttl);
            return Some(summarized.summary);
        }
    }
    None
}
