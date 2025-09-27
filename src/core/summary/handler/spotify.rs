use async_trait::async_trait;
use scraper::Html;
use url::Url;
use crate::core::{request, summary::{def::{Player, SummalyHandler, SummarizeHandler, SummaryResultWithMetadata}, summarize::{self, GenericSummarizeHandler}}};

pub struct SpotifyHandler;

#[async_trait]
impl SummalyHandler for SpotifyHandler {
    fn id(&self) -> &str {
        "spotify"
    }

    fn test(&self, url: &Url) -> bool {
        url.domain().unwrap_or("") == "open.spotify.com"
    }

    async fn summarize(&self, url: &Url) -> Option<SummaryResultWithMetadata> {
        let response = request::get_with_options(url.as_str(), &Some(request::RequestOptions {
            user_agent: Some(request::UserAgentList::TwitterBot),
            ..Default::default()
        })).await.ok()?;

        let body = response.text().await.ok()?;
        let summarized = summarize::execute_summarize(url, body, &SpotifySummarizeHandler).await?;

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

    async fn oembed(&self, url: &Url, href: Option<String>) -> Option<Player> {
        GenericSummarizeHandler.oembed(url, href).await
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

