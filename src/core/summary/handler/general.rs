use async_trait::async_trait;
use url::Url;
use crate::{config::CONFIG, core::{request, summary::{def::{SummalyHandler, SummaryResultWithMetadata}, summarize}}};

pub struct GeneralHandler;

#[async_trait]
impl SummalyHandler for GeneralHandler {
    fn id(&self) -> &str {
        "general"
    }

    fn test(&self, _url: &Url) -> bool {
        true
    }

    async fn summarize(&self, url: &Url) -> Option<SummaryResultWithMetadata> {
        if !&CONFIG.config.ignore_robots_txt && !request::is_allowed_scraping(url).await {
            tracing::info!("Scraping disallowed by robots.txt: {}", url);
            return None;
        }

        let (html, ttl) = request::get(url.as_str()).await.ok()?;
        let summarized = summarize::generic_summarize(url, html).await?;

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: ttl,
        })
    }
}
