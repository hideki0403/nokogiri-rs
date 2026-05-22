use crate::{config::CONFIG, core::{request::{ip_check, resolver}, summary::def::SummarizeArguments}};
use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use parse_size::parse_size;
use reqwest::{
    Client,
    Method,
    Response,
    cookie::Jar,
    header::{
        self,
        HeaderMap,
        HeaderValue,
    },
    redirect::Policy,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use std::{env, fmt, sync::Arc, time::Duration};
use url::Url;

pub static COOKIE_JAR: Lazy<Arc<Jar>> = Lazy::new(|| Arc::new(Jar::default()));

pub static CLIENT: Lazy<ClientWithMiddleware> = Lazy::new(|| {
    let response_timeout = Duration::from_millis(CONFIG.general.response_timeout);
    let client = Client::builder()
        .user_agent(UserAgentList::Default.to_string())
        .redirect(Policy::none())
        .timeout(Duration::from_millis(CONFIG.general.operation_timeout))
        .read_timeout(response_timeout)
        .connect_timeout(response_timeout)
        .cookie_provider(Arc::clone(&COOKIE_JAR))
        .dns_resolver(Arc::new(resolver::CustomDnsResolver))
        .build()
        .unwrap();

    ClientBuilder::new(client).with(ip_check::BlockNonGlobalIpMiddleware).build()
});

pub static CONTENT_LENGTH_LIMIT: Lazy<usize> = Lazy::new(|| match parse_size(&CONFIG.general.content_length_limit) {
    Ok(size) => size as usize,
    Err(e) => {
        tracing::error!("Invalid content length limit in config: {}. Using default 10 MB.", e);
        10 * 1024 * 1024
    }
});

#[derive(Debug, Clone, Default, PartialEq)]
pub enum UserAgentList {
    #[default]
    Default,
    TwitterBot,
    Chrome,
}

impl fmt::Display for UserAgentList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ua_string = match self {
            UserAgentList::Default => format!(
                "Mozilla/5.0 (compatible; {}; {}) SummalyBot/1.0 {}/{}",
                env::consts::OS,
                env::consts::ARCH,
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ),
            UserAgentList::TwitterBot => "Twitterbot/1.0".to_string(),
            UserAgentList::Chrome => {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36".to_string()
            }
        };
        write!(f, "{ua_string}")
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
    pub response: Response,
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

        tracing::trace!("Received {} bytes", received_bytes.len());
        String::from_utf8(received_bytes).ok()
    }

    pub fn ttl(&self) -> u64 {
        self.response
            .headers()
            .get("Cache-Control")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| {
                s.split(',').find_map(|part| {
                    let part = part.trim();
                    if let Some(age_str) = part.strip_prefix("max-age=") {
                        age_str.parse::<u64>().ok()
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(300)
    }

    pub fn content_type(&self) -> Option<String> {
        self.response
            .headers()
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
}

impl From<Response> for ResponseWrapper {
    fn from(response: Response) -> Self {
        Self::new(response)
    }
}

pub async fn send(
    method: Method,
    url: &str,
    options: Option<&RequestOptions>,
) -> Result<Response> {
    let max_redirects = CONFIG.general.max_redirect_hops as usize;
    let mut current_url = Url::parse(url).map_err(|e| anyhow!("Invalid URL: {e}"))?;
    let mut redirects = 0usize;

    let mut headers = if let Some(opt) = options {
        build_headers(opt)?
    } else {
        HeaderMap::new()
    };

    loop {
        let response = CLIENT
            .request(method.clone(), current_url.clone())
            .headers(headers.clone())
            .send()
            .await?;

        if !response.status().is_redirection() {
            return Ok(response);
        }

        if max_redirects == 0 {
            return Ok(response);
        }

        if redirects >= max_redirects {
            anyhow::bail!("Redirect limit exceeded");
        }

        let location = match response.headers().get(header::LOCATION).and_then(|v| v.to_str().ok()) {
            Some(value) => value,
            None => return Ok(response),
        };

        let next_url = match response.url().join(location).or_else(|_| Url::parse(location)) {
            Ok(next) => next,
            Err(e) => {
                anyhow::bail!("Invalid redirect URL: {e}");
            }
        };

        if !matches!(next_url.scheme(), "http" | "https") {
            anyhow::bail!("Unsupported redirect scheme: {}", next_url.scheme());
        }

        if let Err(err) = ip_check::enforce_non_global_ip_block(&next_url).await {
            anyhow::bail!(err);
        }

        if next_url.origin() != current_url.origin() {
            headers.remove(header::AUTHORIZATION);
            headers.remove(header::COOKIE);
            headers.remove(header::PROXY_AUTHORIZATION);
        }

        current_url = next_url;
        redirects += 1;
    }
}

fn build_headers(options: &RequestOptions) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    let accept = options.accept_mime.as_deref().unwrap_or("text/html,application/xhtml+xml");
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_str(accept).map_err(|e| anyhow!("Invalid Accept header value: {e}"))?,
    );

    let lang = options.lang.as_ref().unwrap_or(&CONFIG.general.default_lang);
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_str(lang).map_err(|e| anyhow!("Invalid Accept-Language header value: {e}"))?,
    );

    if options.user_agent != UserAgentList::Default {
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_str(&options.user_agent.to_string())
                .map_err(|e| anyhow!("Invalid User-Agent header value: {e}"))?,
        );
    } else if let Some(ua) = &options.user_agent_string {
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_str(ua).map_err(|e| anyhow!("Invalid User-Agent header value: {e}"))?,
        );
    }

    if let Some(custom_headers) = &options.headers {
        headers.extend(custom_headers.clone());
    }

    Ok(headers)
}
