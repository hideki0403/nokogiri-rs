use crate::core::{
    request::{self},
    summary::{
        def::{SummalyHandler, SummarizeArguments, SummaryResultWithMetadata},
        summarize,
    },
};
use async_trait::async_trait;
use url::Url;

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

    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata> {
        let mut fixed_url = args.url.clone();
        fixed_url.set_query(Some("$web_only=true"));

        let response = request::get(fixed_url.as_str(), &args.into()).await.ok()?;
        let ttl = &response.ttl();
        let summarized = summarize::generic_summarize(&fixed_url, response.text().await?, args).await?;

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: *ttl,
        })
    }
}
