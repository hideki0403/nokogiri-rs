use crate::core::{
    request::{self, RequestOptions},
    summary::{
        def::{SummalyHandler, SummarizeArguments, SummaryResultWithMetadata},
        summarize,
    },
};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

static DOMAIN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.*\.)?(?:youtube(-nocookie)?\.com|youtu.be)$").unwrap());

pub struct YoutubeHandler;

#[async_trait]
impl SummalyHandler for YoutubeHandler {
    fn id(&self) -> &str {
        "youtube"
    }

    fn test(&self, url: &Url) -> bool {
        DOMAIN_REGEX.is_match(url.host_str().unwrap_or(""))
    }

    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata> {
        let mut options: RequestOptions = args.into();
        options.user_agent = request::UserAgentList::TwitterBot;

        let url = &args.url;
        let response = request::get(url.as_str(), &options).await.ok()?;
        let ttl = &response.ttl();
        let summarized = summarize::generic_summarize(url, response.text().await?, args).await?;

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: *ttl,
        })
    }
}
