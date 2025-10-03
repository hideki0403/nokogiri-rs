use crate::core::{
    request::{self, RequestOptions},
    summary::{
        def::{SummalyHandler, SummarizeArguments, SummaryResult, SummaryResultWithMetadata},
        utility::text_clamp,
    },
};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use url::Url;

static PAGE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^https?:\/\/(?:(?<lang>.*).)?wikipedia\.org\/wiki\/(?<title>.*?)(?:(#|\?|\/).*)?$").unwrap());

pub struct WikipediaHandler;

#[async_trait]
impl SummalyHandler for WikipediaHandler {
    fn id(&self) -> &str {
        "wikipedia"
    }

    fn test(&self, url: &Url) -> bool {
        let host = url.host_str().unwrap_or("");
        host == "wikipedia.org" || host.ends_with(".wikipedia.org")
    }

    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata> {
        let url = &args.url;
        let caps = PAGE_REGEX.captures(url.as_str())?;
        let lang = caps.name("lang").map_or("en", |m| m.as_str());
        let title = caps.name("title")?.as_str();

        let mut options: RequestOptions = args.into();
        options.accept_mime = Some("application/json".to_string());

        let response = request::get(
            format!("https://{lang}.wikipedia.org/api/rest_v1/page/summary/{title}").as_str(),
            &options,
        )
        .await;

        let response = match response {
            Ok(resp) => {
                if !resp.response.status().is_success() {
                    return None;
                }
                resp
            }
            Err(_) => return None,
        };

        let response = response.text().await;
        let page_content = serde_json::from_str::<WikipediaApiResponse>(response.as_ref()?).ok()?;
        let result = SummaryResult {
            title: text_clamp(&page_content.title, 100),
            description: Some(text_clamp(&page_content.extract, 300)),
            icon: Some("https://wikipedia.org/static/favicon/wikipedia.ico".to_string()),
            sitename: Some("Wikipedia".to_string()),
            thumbnail: Some(format!("https://wikipedia.org/static/images/project-logos/{lang}wiki.png")),
            ..Default::default()
        };

        Some(SummaryResultWithMetadata {
            summary: result,
            cache_ttl: 604800,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct WikipediaApiResponse {
    pub title: String,
    pub extract: String,
}
