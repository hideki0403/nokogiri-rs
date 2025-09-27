use std::{error::Error, sync::Arc};
use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::{cookie::Jar, header::HeaderMap, Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error as ReqwestMiddlewareError};
use http_acl_reqwest::{http_acl::IpNet, HttpAcl, HttpAclMiddleware};
use hyper_util::client::legacy::Error as HyperUtilError;
use url::Url;
use crate::config::CONFIG;

mod resolver;

pub static COOKIE_JAR: Lazy<Arc<Jar>> = Lazy::new(|| Arc::new(Jar::default()));

pub static CLIENT: Lazy<ClientWithMiddleware> = Lazy::new(|| {
    let allowed_private_ips: Vec<IpNet> = CONFIG.security.allowed_private_ips
        .iter()
        .filter_map(|ip| {
            let x = ip.parse();
            if let Err(e) = &x {
                tracing::error!("Failed to parse allowed_private_ip '{}': {}", ip, e);
            }
            x.ok()
        })
        .collect();

    let acl = HttpAcl::builder()
        .ip_acl_default(true)
        .port_acl_default(true)
        .host_acl_default(true)
        .non_global_ip_ranges(false)
        .allowed_ip_ranges(allowed_private_ips)
        .unwrap()
        .build();

    let middleware = HttpAclMiddleware::new(acl);

    // TODO: options
    let client = Client::builder()
        .user_agent(UserAgentList::Default.to_string())
        .cookie_provider(Arc::clone(&COOKIE_JAR))
        .dns_resolver(middleware.with_dns_resolver(Arc::new(resolver::CustomDnsResolver::default())))
        .build()
        .unwrap();

    ClientBuilder::new(client)
        .with(middleware)
        .build()
});

#[derive(Debug, Clone)]
pub enum UserAgentList {
    Default,
    TwitterBot,
    Chrome,
}

impl UserAgentList {
    pub fn to_string(&self) -> String {
        match self {
            UserAgentList::Default => format!("Mozilla/5.0 (compatible; {}/{})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            UserAgentList::TwitterBot => "Twitterbot/1.0".to_string(),
            UserAgentList::Chrome => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct RequestOptions {
    pub user_agent: Option<UserAgentList>,
    pub accept_mime: Option<String>,
    pub headers: Option<HeaderMap>,
}

pub async fn get(url: &str) -> Result<String> {
    let response = get_with_options(url, &None).await?;
    Ok(response.text().await?)
}

pub async fn get_with_options(url: &str, options: &Option<RequestOptions>) -> Result<Response> {
    // TODO: リクエストのキャッシュ
    let default_options = RequestOptions::default();
    let options = options.as_ref().unwrap_or(&default_options);

    let mut headers = HeaderMap::new();
    headers.insert("Accept", options.accept_mime.as_deref().unwrap_or("text/html,application/xhtml+xml").parse().unwrap());

    if let Some(ua) = &options.user_agent {
        headers.insert("User-Agent", ua.to_string().parse().unwrap());
    }

    if let Some(custom_headers) = &options.headers {
        headers.extend(custom_headers.clone());
    }

    let request = CLIENT
        .get(url)
        .headers(headers);

    let response = request.send().await;
    if let Err(e) = &response {
        tracing::error!("Failed to fetch '{}' -> {}", url, e);

        match e {
            ReqwestMiddlewareError::Middleware(_) => {
            }
            ReqwestMiddlewareError::Reqwest(err) => {
                if let Some(hyper_err) = err.source().and_then(|s| s.downcast_ref::<HyperUtilError>()) {
                    hyper_err.source().and_then(|e| e.downcast_ref::<std::io::Error>()).map(|io_err| {
                        if io_err.kind() == std::io::ErrorKind::NotConnected {
                            tracing::error!("Connection refused: {}", io_err);
                        }
                    });
                }
            }
        }
    }

    Ok(response?)
}

pub async fn head(url: &str) -> Result<HeaderMap> {
    let response = CLIENT.head(url).send().await?;
    Ok(response.headers().clone())
}

pub fn add_cookie(url: &Url, cookie_str: &str) {
    COOKIE_JAR.add_cookie_str(cookie_str, &url);
}
