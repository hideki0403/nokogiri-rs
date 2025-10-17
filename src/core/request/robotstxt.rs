use crate::core::{
    cache,
    request::{self, RequestOptions},
};
use reqwest::StatusCode;
use url::Url;

struct RobotsTxt {
    content: Option<String>,
    cached: bool,
    failed: bool,
    disallowed: bool,
}

impl RobotsTxt {
    fn failed(disallowed: bool) -> Self {
        Self {
            content: None,
            cached: false,
            failed: true,
            disallowed,
        }
    }

    fn success(content: String, cached: bool) -> Self {
        Self {
            content: Some(content),
            cached,
            failed: false,
            disallowed: false,
        }
    }
}

pub async fn is_allowed_scraping(url: &Url) -> bool {
    let domain = match url.host_str() {
        Some(d) => d,
        None => return false,
    };

    let result = fetch(domain, url).await;
    if result.failed || result.content.is_none() {
        if !result.disallowed {
            cache::set_robotstxt_cache(domain, "");
        }

        return !result.disallowed;
    }

    let txt = result.content.unwrap();
    let parsed = texting_robots::Robot::new("SummalyBot", txt.as_bytes());

    if !result.cached {
        let x = if parsed.is_ok() { txt } else { "".to_string() };
        cache::set_robotstxt_cache(domain, &x);
    }

    parsed.map_or(true, |robot| robot.allowed(url.path()))
}

async fn fetch(domain: &str, url: &Url) -> RobotsTxt {
    if let Some(cached) = cache::get_robotstxt_cache(domain) {
        tracing::debug!("Robots.txt cache hit for domain: {}", domain);
        return RobotsTxt::success(cached, true);
    }

    let robots_url = match url.join("/robots.txt") {
        Ok(u) => u,
        Err(e) => {
            tracing::debug!("Failed to construct robots.txt URL for '{}': {}", url, e);
            return RobotsTxt::failed(false);
        }
    };

    let response = match request::get(robots_url.as_str(), &RequestOptions::default()).await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::debug!("Failed to fetch robots.txt from '{}': {}", robots_url, e);
            return RobotsTxt::failed(false);
        }
    };

    let status = response.response.status();
    if !status.is_success() {
        tracing::debug!("Non-success status code for robots.txt from '{}': {}", robots_url, status);
        return RobotsTxt::failed(status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS);
    }

    if response.content_type().is_none_or(|x| !x.starts_with("text/plain")) {
        tracing::debug!("Invalid content type for robots.txt from '{}'", robots_url);
        return RobotsTxt::failed(false);
    }

    let content = match response.text().await {
        Some(x) => x,
        None => {
            tracing::debug!("Failed to read robots.txt content from '{}'", robots_url);
            return RobotsTxt::failed(false);
        }
    };

    RobotsTxt::success(content, false)
}
