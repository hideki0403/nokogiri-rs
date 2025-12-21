use crate::core::{
    request::{self, RequestOptions},
    summary::{
        def::{SummalyHandler, SummarizeArguments, SummaryResultWithMetadata},
        summarize,
    },
};
use async_trait::async_trait;
use url::Url;

pub struct RedditHandler;

#[async_trait]
impl SummalyHandler for RedditHandler {
    fn id(&self) -> &str {
        "reddit"
    }

    fn test(&self, url: &Url) -> bool {
        let host = url.host_str().unwrap_or("");
        host == "reddit.com" || host.ends_with(".reddit.com") || host == "redd.it"
    }

    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata> {
        let mut options: RequestOptions = args.into();
        options.user_agent = request::UserAgentList::TwitterBot;

        let url = &args.url;
        let response = request::get(url.as_str(), &options).await.ok()?;
        let summarized = summarize::generic_summarize(url, response.text().await?, args).await?;

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: 3600, // 1 hour
        })
    }
}
