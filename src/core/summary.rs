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

static LANG_REGEX: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"^[a-zA-Z]{2,3}-[a-zA-Z]{2,3}$").unwrap());

pub async fn summary(args: SummarizeArguments) -> Option<def::SummaryResult> {
    let url = &args.url;
    let lang = args.lang.clone();

    if lang.is_none() || !LANG_REGEX.is_match(&lang.unwrap()) {
        tracing::error!("Invalid language code: {}", args.lang.unwrap_or("None".to_string()));
        return None;
    }

    let mut lang = args.lang.clone();
    if let Some(l) = lang.clone() &&
        l == "ja-KS"
    {
        lang = Some("ja-JP".to_string());
    }

    let cache = cache::get_summarize_cache(url.as_str(), lang.clone());
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
                    cache::set_summarize_cache(url.as_str(), lang.clone(), &serialized, &s.cache_ttl.clamp(300, 86400));
                    Some(s.summary)
                }
                None => {
                    cache::set_summarize_cache(url.as_str(), lang.clone(), "null", &300);
                    None
                }
            };

            return summary;
        }
    }
    None
}
