use async_trait::async_trait;
use url::Url;
use crate::{config::CONFIG, core::{request, summary::{def::{SummalyHandler, SummarizeArguments, SummaryResultWithMetadata}, summarize}}};

pub struct GeneralHandler;

#[async_trait]
impl SummalyHandler for GeneralHandler {
    fn id(&self) -> &str {
        "general"
    }

    fn test(&self, _url: &Url) -> bool {
        true
    }

    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata> {
        let url = &args.url;
        if !&CONFIG.config.ignore_robots_txt && !request::is_allowed_scraping(url).await {
            tracing::info!("Scraping disallowed by robots.txt: {}", url);
            return None;
        }

        let response = request::get(&url.as_str(), &args.into()).await.ok()?;
        let ttl = &response.ttl();
        let summarized = summarize::generic_summarize(&url, response.text().await?, args).await?;

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: *ttl,
        })
    }
}
