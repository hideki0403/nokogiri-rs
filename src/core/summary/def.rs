use scraper::Html;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use url::Url;

/* Summary */

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Player {
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub allow: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SummaryResult {
    pub title: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub sitename: Option<String>,
    pub player: Player,
    pub sensitive: Option<bool>,
    pub activity_pub: Option<String>,
    /// The @ handle of a fediverse user (https://blog.joinmastodon.org/2024/07/highlighting-journalism-on-mastodon/)
    pub fediverse_creator: Option<String>,
    pub large_card: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct SummaryResultWithMetadata {
    pub summary: SummaryResult,
    pub cache_ttl: u64, // in seconds
}

#[async_trait]
pub trait SummalyHandler: Send + Sync {
    fn id(&self) -> &str;
    fn test(&self, url: &Url) -> bool;
    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata>;
}

/* Summarize */

#[async_trait]
pub trait SummarizeHandler: Send + Sync {
    fn title(&self, url: &Url, html: &Html) -> Option<String>;
    fn icon(&self, url: &Url, html: &Html) -> Option<Url>;
    async fn icon_exists(&self, url: &Option<Url>) -> bool;
    fn description(&self, url: &Url, html: &Html) -> Option<String>;
    fn sitename(&self, url: &Url, html: &Html) -> Option<String>;
    fn thumbnail(&self, url: &Url, html: &Html) -> Option<String>;
    fn extract_oembed_url(&self, url: &Url, html: &Html) -> Option<String>;
    async fn oembed(&self, url: &Url, href: Option<String>, args: &SummarizeArguments) -> Option<Player>;
    fn player(&self, url: &Url, html: &Html, is_summary_large_image: bool) -> Option<Player>;
    fn sensitive(&self, url: &Url, html: &Html) -> Option<bool>;
    fn activity_pub(&self, url: &Url, html: &Html) -> Option<String>;
    fn fediverse_creator(&self, url: &Url, html: &Html) -> Option<String>;
    fn summary_large_image(&self, url: &Url, html: &Html) -> bool;
}

pub struct SummarizeArguments {
    pub url: Url,
    pub lang: Option<String>,
    pub follow_redirects: Option<bool>,
    pub user_agent: Option<String>,
    pub response_timeout: Option<u64>,
    pub operation_timeout: Option<u64>,
    pub content_length_limit: Option<usize>,
    pub content_length_required: Option<bool>,
}

/* oEmbed */

#[derive(Debug, Deserialize)]
pub struct OEmbedData {
    pub r#type: String,
    pub version: String,
    pub html: String,
    pub width: u32,
    pub height: u32,
}
