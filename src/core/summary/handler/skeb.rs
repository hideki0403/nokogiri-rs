use async_trait::async_trait;
use axum::http::HeaderMap;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize};
use url::Url;
use crate::core::{request::{self, RequestOptions}, summary::{def::{SummalyHandler, SummarizeArguments, SummaryResult, SummaryResultWithMetadata}, utility::text_clamp}};

static COOKIE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"document\.cookie\s?=\s?"(?<cookie>.*)";"#).unwrap()
});

static ACCEPTABLE_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https:\/\/([a-z0-9-]+\.)?skeb\.jp\/@(?<user>\w+)(\/works\/(?<work>[0-9]+))?\/?$").unwrap()
});

static REQUEST_OPTIONS: Lazy<RequestOptions> = Lazy::new(|| RequestOptions {
    user_agent: request::UserAgentList::Chrome,
    accept_mime: Some("application/json".to_string()),
    headers: Some({
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer null".parse().unwrap());
        headers
    }),
    ..Default::default()
});

pub struct SkebHandler;

impl SkebHandler {
    async fn api_caller<T: DeserializeOwned>(&self, url: &str) -> Option<T> {
        let u = Url::parse(&url).ok()?;
        let mut response = request::get(url, &REQUEST_OPTIONS).await.ok()?.response;

        let is_too_many_requests = response.status() == StatusCode::TOO_MANY_REQUESTS;
        let retry_after_zero = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .map(|s| s == "0")
            .unwrap_or(false);

        if is_too_many_requests && retry_after_zero {
            tracing::debug!("Skeb cookie check detected, adding cookie...");
            let body = response.text().await.ok()?;
            let cookie = COOKIE_REGEX.captures(&body)
                .and_then(|caps| caps.name("cookie").map(|m| m.as_str().to_string()))?;

            request::add_cookie(&u, &cookie);
            response = request::get(url, &REQUEST_OPTIONS).await.ok()?.response;
        }

        response.json::<T>().await.ok()
    }
}

#[async_trait]
impl SummalyHandler for SkebHandler {
    fn id(&self) -> &str {
        "skeb"
    }

    fn test(&self, url: &Url) -> bool {
        ACCEPTABLE_URL_REGEX.is_match(url.as_str())
    }

    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata> {
        let caps = ACCEPTABLE_URL_REGEX.captures(args.url.as_str())?;
        let user = caps.name("user")?.as_str();
        let work = caps.name("work").map(|m| m.as_str());

        println!("Fetching Skeb summary for user: {}, work: {:?}", user, work);

        let summary = if let Some(work_id) = work {
            let response = self.api_caller::<SkebWorkResponse>(&format!("https://skeb.jp/api/users/{user}/works/{work_id}")).await?;
            let clampped_description = match &response.body.clone() {
                Some(desc) => text_clamp(&desc.replace("\n", ""), 12),
                None => "Untitled".to_string(),
            };

            SkebSummary {
                title: format!("{} by {}", clampped_description, response.creator.name),
                description: response.body.and_then(|x| Some(text_clamp(&x, 300))),
                og_image: response.og_image_url,
                nsfw: response.nsfw,
            }
        } else {
            let response = self.api_caller::<SkebUserResponse>(&format!("https://skeb.jp/api/users/{user}")).await?;
            SkebSummary {
                title: format!("{} (@{})", response.name, response.screen_name),
                description: response.description.and_then(|x| Some(text_clamp(&x, 300))),
                og_image: response.og_image_url,
                nsfw: false,
            }
        };

        let summarized = SummaryResult {
            title: format!("{} | Skeb", summary.title),
            description: summary.description,
            icon: Some("https://fcdn.skeb.jp/assets/v1/commons/favicon.ico".to_string()),
            sitename: Some("Skeb".to_string()),
            thumbnail: summary.og_image,
            sensitive: Some(summary.nsfw),
            large_card: Some(true),
            ..Default::default()
        };

        Some(SummaryResultWithMetadata {
            summary: summarized,
            cache_ttl: 3600,
        })
    }
}

struct SkebSummary {
    title: String,
    description: Option<String>,
    og_image: Option<String>,
    nsfw: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct SkebUserResponse {
    pub name: String,
    pub screen_name: String,
    pub description: Option<String>,
    pub og_image_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct SkebWorkResponse {
    pub creator: SkebWorkCreator,
    pub body: Option<String>,
    pub og_image_url: Option<String>,
    pub nsfw: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct SkebWorkCreator {
    pub name: String,
}
