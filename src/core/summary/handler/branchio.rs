use async_trait::async_trait;
use url::Url;
use crate::core::{request, summary::{def::{SummalyHandler, SummaryResult, SummaryResultWithMetadata}, summarize}};

pub struct BranchioHandler;

#[async_trait]
impl SummalyHandler for BranchioHandler {
    fn id(&self) -> &str {
        "branchio"
    }

    fn test(&self, url: &Url) -> bool {
        let domain = url.domain().unwrap_or("");
        domain == "spotify.link" || domain.ends_with(".app.link")
    }

    async fn summarize(&self, url: &Url) -> Option<SummaryResultWithMetadata> {
        let mut fixed_url = url.clone();
        fixed_url.set_query(Some("$web_only=true"));

        let (html, ttl) = request::get(fixed_url.as_str()).await.ok()?;
        let summarized = summarize::generic_summarize(&fixed_url, html).await?;

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: ttl,
        })
    }
}
