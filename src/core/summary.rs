use crate::{
    config::CONFIG,
    core::{cache, summary::def::SummarizeArguments},
};
use once_cell::sync::Lazy;

pub mod def;
pub mod handler;
pub mod selector;
pub mod summarize;
pub mod utility;

static ACTIVE_HANDLERS: Lazy<Vec<&'static dyn def::SummalyHandler>> = Lazy::new(|| {
    handler::HANDLERS
        .iter()
        .filter(|handler| !CONFIG.plugins.disabled.contains(&handler.id().to_string()))
        .copied()
        .collect()
});

pub async fn summary(args: SummarizeArguments) -> Option<def::SummaryResult> {
    let url = &args.url;
    let cache = cache::get_summarize_cache(url.as_str(), args.lang.clone());
    if let Some(cached) = cache {
        tracing::debug!("Cache hit for URL: {}", url);
        return serde_json::from_str(&cached).ok();
    }

    for handler in ACTIVE_HANDLERS.iter() {
        if handler.test(url) {
            tracing::debug!("Using handler: {}", handler.id());

            let summary = match handler.summarize(&args).await {
                Some(mut s) => {
                    if s.summary.url.is_none() {
                        s.summary.url = Some(url.as_str().to_string());
                    }

                    let serialized = serde_json::to_string(&s.summary).ok()?;
                    cache::set_summarize_cache(url.as_str(), args.lang.clone(), &serialized, &s.cache_ttl.clamp(300, 86400));
                    Some(s.summary)
                }
                None => {
                    cache::set_summarize_cache(url.as_str(), args.lang.clone(), "null", &300);
                    None
                }
            };

            return summary;
        }
    }
    None
}
