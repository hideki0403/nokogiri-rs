use crate::core::{
    request::{self, RequestOptions},
    summary::{
        def::{Player, SummalyHandler, SummarizeArguments, SummarizeHandler, SummaryResultWithMetadata},
        summarize::{self, GenericSummarizeHandler},
    },
};
use async_trait::async_trait;
use scraper::Html;
use url::Url;

pub struct SpotifyHandler;

#[async_trait]
impl SummalyHandler for SpotifyHandler {
    fn id(&self) -> &str {
        "spotify"
    }

    fn test(&self, url: &Url) -> bool {
        url.domain().unwrap_or("") == "open.spotify.com"
    }

    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata> {
        let url = &args.url;
        let mut options: RequestOptions = args.into();
        options.user_agent = request::UserAgentList::TwitterBot;

        let response = request::get(url.as_str(), &options).await.ok()?;
        let summarized = summarize::execute_summarize(url, response.text().await?, args, &SpotifySummarizeHandler).await?;

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: 86400, // 1 day
        })
    }
}

pub struct SpotifySummarizeHandler;

#[async_trait]
impl SummarizeHandler for SpotifySummarizeHandler {
    fn title(&self, url: &Url, html: &Html) -> Option<String> {
        GenericSummarizeHandler.title(url, html)
    }

    fn icon(&self, url: &Url, html: &Html) -> Option<Url> {
        GenericSummarizeHandler.icon(url, html)
    }

    async fn icon_exists(&self, _url: &Option<Url>) -> bool {
        true
    }

    fn description(&self, url: &Url, html: &Html) -> Option<String> {
        GenericSummarizeHandler.description(url, html)
    }

    fn sitename(&self, _url: &Url, _html: &Html) -> Option<String> {
        Some("Spotify".to_string())
    }

    fn thumbnail(&self, url: &Url, html: &Html) -> Option<String> {
        GenericSummarizeHandler.thumbnail(url, html)
    }

    fn extract_oembed_url(&self, url: &Url, html: &Html) -> Option<String> {
        GenericSummarizeHandler.extract_oembed_url(url, html)
    }

    async fn oembed(&self, url: &Url, href: Option<String>, args: &SummarizeArguments) -> Option<Player> {
        GenericSummarizeHandler.oembed(url, href, args).await
    }

    fn player(&self, _url: &Url, _html: &Html, _is_summary_large_image: bool) -> Option<Player> {
        None
    }

    fn sensitive(&self, _url: &Url, _html: &Html) -> Option<bool> {
        None
    }

    fn activity_pub(&self, _url: &Url, _html: &Html) -> Option<String> {
        None
    }

    fn fediverse_creator(&self, _url: &Url, _html: &Html) -> Option<String> {
        None
    }

    fn summary_large_image(&self, _url: &Url, _html: &Html) -> bool {
        false
    }
}
