use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;
use crate::core::{request::{self, RequestOptions}, summary::{def::{Player, SummalyHandler, SummarizeArguments, SummarizeHandler, SummaryResultWithMetadata}, selector, summarize::{self, GenericSummarizeHandler}}};

static DOMAIN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(www\.)?((amazon(\.co|com)?(\.[a-z]{2})?|amzn\.[a-z]{2,4}))$").unwrap()
});

static SELECTOR_ID_ADULT_WARNING: Lazy<Selector> = Lazy::new(|| selector::s("#adultWarning"));

pub struct AmazonHandler;

#[async_trait]
impl SummalyHandler for AmazonHandler {
    fn id(&self) -> &str {
        "amazon"
    }

    fn test(&self, url: &Url) -> bool {
        let host = match url.host_str() {
            Some(h) => h,
            None => return false,
        };

        DOMAIN_REGEX.is_match(host)
    }

    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata> {
        let mut options: RequestOptions = args.into();
        options.user_agent = request::UserAgentList::TwitterBot;

        let response = request::get(args.url.as_str(), &options).await.ok()?;

        let body = response.text().await?;
        let summarized = summarize::execute_summarize(&args.url, body, args, &AmazonSummarizeHandler).await?;

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: 3600,
        })
    }
}

pub struct AmazonSummarizeHandler;

#[async_trait]
impl SummarizeHandler for AmazonSummarizeHandler {
    fn title(&self, url: &Url, html: &Html) -> Option<String> {
        GenericSummarizeHandler.title(url, html)
    }

    fn icon(&self, _url: &Url, _html: &Html) -> Option<Url> {
        Url::parse("https://www.amazon.com/favicon.ico").ok()
    }

    async fn icon_exists(&self, _url: &Option<Url>) -> bool {
        true
    }

    fn description(&self, url: &Url, html: &Html) -> Option<String> {
        GenericSummarizeHandler.description(url, html)
    }

    fn sitename(&self, _url: &Url, _html: &Html) -> Option<String> {
        Some("Amazon".to_string())
    }

    fn thumbnail(&self, url: &Url, html: &Html) -> Option<String> {
        GenericSummarizeHandler.thumbnail(url, html)
    }

    fn extract_oembed_url(&self, _url: &Url, _html: &Html) -> Option<String> {
        None
    }

    async fn oembed(&self, _url: &Url, _href: Option<String>, _args: &SummarizeArguments) -> Option<Player> {
        None
    }

    fn player(&self, url: &Url, html: &Html, is_summary_large_image: bool) -> Option<Player> {
        GenericSummarizeHandler.player(url, html, is_summary_large_image)
    }

    fn sensitive(&self, _url: &Url, html: &Html) -> Option<bool> {
        Some(html.select(&SELECTOR_ID_ADULT_WARNING).next().is_some())
    }

    fn activity_pub(&self, _url: &Url, _html: &Html) -> Option<String> {
        None
    }

    fn fediverse_creator(&self, _url: &Url, _html: &Html) -> Option<String> {
        None
    }

    fn summary_large_image(&self, _url: &Url, _html: &Html) -> bool {
        true
    }
}
