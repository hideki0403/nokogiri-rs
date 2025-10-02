use std::{env, error::Error, sync::Arc, time::Duration};
use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::{cookie::Jar, header::HeaderMap, redirect::Policy, Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error as ReqwestMiddlewareError};
use http_acl_reqwest::{HttpAcl, HttpAclMiddleware};
use hyper_util::client::legacy::Error as HyperUtilError;
use url::Url;
use parse_size::parse_size;
use crate::{config::CONFIG, core::{cache, summary::def::SummarizeArguments}};

mod resolver;

pub static COOKIE_JAR: Lazy<Arc<Jar>> = Lazy::new(|| Arc::new(Jar::default()));

pub static CLIENT: Lazy<ClientWithMiddleware> = Lazy::new(|| {
    let acl = HttpAcl::builder()
        .ip_acl_default(true)
        .port_acl_default(true)
        .host_acl_default(true)
        .non_global_ip_ranges(CONFIG.security.block_non_global_ips)
        .build();

    let middleware = HttpAclMiddleware::new(acl);
    let response_timeout = Duration::from_millis(CONFIG.general.response_timeout);
    let client = Client::builder()
        .user_agent(UserAgentList::Default.to_string())
        .redirect(Policy::limited(CONFIG.general.max_redirect_hops as usize))
        .timeout(Duration::from_millis(CONFIG.general.operation_timeout))
        .read_timeout(response_timeout)
        .connect_timeout(response_timeout)
        .cookie_provider(Arc::clone(&COOKIE_JAR))
        .dns_resolver(middleware.with_dns_resolver(Arc::new(resolver::CustomDnsResolver::default())))
        .build()
        .unwrap();

    ClientBuilder::new(client)
        .with(middleware)
        .build()
});

pub static CONTENT_LENGTH_LIMIT: Lazy<usize> = Lazy::new(|| {
    match parse_size(&CONFIG.general.content_length_limit) {
        Ok(size) => size as usize,
        Err(e) => {
            tracing::error!("Invalid content length limit in config: {}. Using default 10 MB.", e);
            10 * 1024 * 1024
        }
    }
});

#[derive(Debug, Clone, Default, PartialEq)]
pub enum UserAgentList {
    #[default]
    Default,
    TwitterBot,
    Chrome,
}

impl UserAgentList {
    pub fn to_string(&self) -> String {
        match self {
            UserAgentList::Default => format!("Mozilla/5.0 (compatible; {} {}) SummalyBot/1.0 {}/{}", env::consts::OS, env::consts::ARCH, env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            UserAgentList::TwitterBot => "Twitterbot/1.0".to_string(),
            UserAgentList::Chrome => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct RequestOptions {
    pub user_agent: UserAgentList,
    pub accept_mime: Option<String>,
    pub headers: Option<HeaderMap>,
    pub lang: Option<String>,
    pub user_agent_string: Option<String>,
    // pub follow_redirects: Option<bool>,
    // pub response_timeout: Option<u64>,
    // pub operation_timeout: Option<u64>,
    // pub content_length_limit: Option<usize>,
    // pub content_length_required: Option<bool>,
}

impl From<&SummarizeArguments> for RequestOptions {
    fn from(args: &SummarizeArguments) -> Self {
        RequestOptions {
            lang: args.lang.clone(),
            user_agent_string: args.user_agent.clone(),
            // follow_redirects: args.follow_redirects,
            // response_timeout: args.response_timeout,
            // operation_timeout: args.operation_timeout,
            // content_length_limit: args.content_length_limit,
            // content_length_required: args.content_length_required,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct ResponseWrapper {
    pub response: Response
}

impl ResponseWrapper {
    pub fn new(response: Response) -> Self {
        Self { response }
    }

    pub async fn text(mut self) -> Option<String> {
        if *CONTENT_LENGTH_LIMIT == 0 {
            tracing::debug!("Content length limit is disabled, reading entire response body");
            return self.response.text().await.ok();
        }

        let mut received_bytes = Vec::new();
        let mut received_size = 0;

        while let Some(chunk) = self.response.chunk().await.transpose() {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Failed to read chunk from response body: {}", e);
                    return None;
                }
            };

            received_size += chunk.len();
            if received_size > *CONTENT_LENGTH_LIMIT {
                tracing::warn!("Response body exceeded the content length limit of {:?} bytes", *CONTENT_LENGTH_LIMIT);
                return None;
            }
            received_bytes.extend_from_slice(&chunk);
        }

        tracing::debug!("Received {} bytes", received_bytes.len());
        String::from_utf8(received_bytes).ok()
    }

    pub fn ttl(&self) -> u64 {
        self.response.headers()
            .get("Cache-Control")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| {
                s.split(',')
                .find_map(|part| {
                    let part = part.trim();
                    if part.starts_with("max-age=") {
                        part[8..].parse::<u64>().ok()
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(300)
    }
}

impl From<Response> for ResponseWrapper {
    fn from(response: Response) -> Self {
        Self::new(response)
    }
}

pub async fn get(url: &str, options: &RequestOptions) -> Result<ResponseWrapper> {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", options.accept_mime.as_deref().unwrap_or("text/html,application/xhtml+xml").parse().unwrap());

    let lang = options.lang
        .as_ref()
        .unwrap_or(&CONFIG.general.default_lang);

    headers.insert("Accept-Language", lang.parse().unwrap());

    if &options.user_agent != &UserAgentList::Default {
        headers.insert("User-Agent", options.user_agent.to_string().parse().unwrap());
    } else if let Some(ua) = &options.user_agent_string {
        headers.insert("User-Agent", ua.parse().unwrap());
    }

    if let Some(custom_headers) = &options.headers {
        headers.extend(custom_headers.clone());
    }

    let request = CLIENT
        .get(url)
        .headers(headers);

    let response = request.send().await;
    if let Err(e) = &response {
        let is_ignore_error = 'err: {
            let ReqwestMiddlewareError::Reqwest(inner) = e else { break 'err false };
            let Some(hyper_err) = inner.source().and_then(|s| s.downcast_ref::<HyperUtilError>()) else { break 'err false };
            if let Some(source) = hyper_err.source() {
                source.to_string() == "tcp connect error"
            } else {
                false
            }
        };

        if is_ignore_error {
            tracing::info!("Failed to resolve host for '{}'. The resolved IP address may have been blocked by ACL.", url);
        } else {
            let mut root_cause: &dyn std::error::Error = &e;
            while let Some(source) = root_cause.source() {
                root_cause = source;
            }
            tracing::error!("Failed to fetch '{}' -> {}", url, root_cause);
        }
    }

    Ok(response?.into())
}

pub async fn head(url: &str) -> Result<HeaderMap> {
    let response = CLIENT.head(url).send().await?;
    Ok(response.headers().clone())
}

pub fn add_cookie(url: &Url, cookie_str: &str) {
    COOKIE_JAR.add_cookie_str(cookie_str, &url);
}

pub async fn is_allowed_scraping(url: &Url) -> bool {
    let domain = match url.host_str() {
        Some(d) => d,
        None => return false,
    };

    let txt = match cache::get_robotstxt_cache(domain) {
        Some(cached) => {
            tracing::debug!("Robots.txt cache hit for domain: {}", domain);
            cached
        },
        None => {
            let robots_url = match url.join("/robots.txt") {
                Ok(u) => u,
                Err(e) => {
                    tracing::debug!("Failed to construct robots.txt URL for '{}': {}", url, e);
                    return false;
                }
            };

            let response = match get(robots_url.as_str(), &RequestOptions::default()).await {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::debug!("Failed to fetch robots.txt from '{}': {}", robots_url, e);
                    cache::set_robotstxt_cache(domain, "");
                    return true;
                }
            };

            let content = match response.text().await {
                Some(x) => x,
                None => {
                    tracing::debug!("Failed to read robots.txt content from '{}'", robots_url);
                    cache::set_robotstxt_cache(domain, "");
                    return true;
                }
            };

            cache::set_robotstxt_cache(domain, &content);
            content
        }
    };

    texting_robots::Robot::new("SummalyBot", &txt.as_bytes())
        .map_or(true, |robot| robot.allowed(url.path()))
}
