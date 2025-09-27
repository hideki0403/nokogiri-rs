use async_trait::async_trait;
use url::Url;
use crate::core::{request, summary::{def::{SummalyHandler, SummaryResultWithMetadata}, summarize}};

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
        let (html, ttl) = request::get(url.as_str()).await.ok()?;
        let summarized = summarize::generic_summarize(url, html).await?;

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: ttl,
        })
    }
}
