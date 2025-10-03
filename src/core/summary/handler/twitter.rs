use crate::core::{
    request,
    summary::def::{SummalyHandler, SummarizeArguments, SummaryResult, SummaryResultWithMetadata},
};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use url::Url;

static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^https?:\/\/((www|mobile)\.)?(twitter|x)\.com\/\w+\/status\/(?<id>\d+)([\/?#].*)?$").unwrap());

pub struct TwitterHandler;

#[async_trait]
impl SummalyHandler for TwitterHandler {
    fn id(&self) -> &str {
        "twitter"
    }

    fn test(&self, url: &Url) -> bool {
        URL_REGEX.is_match(url.as_str())
    }

    async fn summarize(&self, args: &SummarizeArguments) -> Option<SummaryResultWithMetadata> {
        let url = &args.url;
        let id = URL_REGEX.captures(url.as_str())?.name("id")?.as_str();
        let response = request::get(
            format!("https://cdn.syndication.twimg.com/tweet-result?id={id}&token=x&lang=en").as_str(),
            &args.into(),
        )
        .await;

        if response.is_err() {
            return None;
        }
        let response = response.unwrap().text().await;

        let is_twitter = url.domain().unwrap_or("").to_lowercase().contains("twitter");
        let tweet = serde_json::from_str::<TweetData>(response.as_ref()?).ok()?;
        let data_available = match &tweet.__typename {
            Some(t) => t == "Tweet",
            None => false,
        };

        let mut result = SummaryResult {
            icon: if is_twitter {
                Some("https://abs.twimg.com/favicons/twitter.2.ico".to_string())
            } else {
                Some("https://x.com/favicon.ico".to_string())
            },
            sitename: if is_twitter {
                Some("Twitter".to_string())
            } else {
                Some("X".to_string())
            },
            sensitive: tweet.possibly_sensitive,
            ..Default::default()
        };

        if data_available {
            let Some(user) = &tweet.user else {
                tracing::info!("Tweet user data is missing for id: {}", id);
                return None;
            };

            let (Some(username), Some(screen_name)) = (&user.name, &user.screen_name) else {
                tracing::info!("Tweet user name or screen_name is missing for id: {}", id);
                return None;
            };

            let mut tweet_text = tweet.text.clone().unwrap_or_default();

            if let Some(entities) = &tweet.entities {
                entities.urls.iter().flatten().for_each(|x| {
                    if let (Some(url), Some(display_url)) = (&x.url, &x.display_url) {
                        tweet_text = tweet_text.replace(url, display_url);
                    }
                });

                entities.media.iter().flatten().for_each(|m| {
                    if let Some(url) = &m.url {
                        tweet_text = tweet_text.replace(url, "");
                    }
                });
            }

            let thumbnail = tweet
                .video
                .as_ref()
                .and_then(|v| v.poster.clone())
                .or_else(|| tweet.photos.as_ref().and_then(|p| p.first()).and_then(|p| p.url.clone()))
                .or_else(|| user.profile_image_url_https.clone().map(|url| url.replace("_normal.", ".")));

            result.title = format!("{username} (@{screen_name})");
            result.description = Some(tweet_text.trim().to_string());
            result.thumbnail = thumbnail;
        } else {
            result.title = if is_twitter { "Twitter".to_string() } else { "X".to_string() };
        }

        Some(SummaryResultWithMetadata {
            summary: result,
            cache_ttl: 3600,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct TweetData {
    pub __typename: Option<String>,
    pub text: Option<String>,
    pub user: Option<TweetUser>,
    pub entities: Option<TweetEntities>,
    pub photos: Option<Vec<TweetPhoto>>,
    pub video: Option<TweetVideo>,
    pub possibly_sensitive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TweetUser {
    pub name: Option<String>,
    pub screen_name: Option<String>,
    pub profile_image_url_https: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TweetEntities {
    pub urls: Option<Vec<TwitterUrl>>,
    pub media: Option<Vec<TwitterUrl>>,
}

#[derive(Debug, Deserialize)]
pub struct TwitterUrl {
    pub display_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TweetPhoto {
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TweetVideo {
    pub poster: Option<String>,
}
