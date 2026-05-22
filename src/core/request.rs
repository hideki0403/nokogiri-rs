use anyhow::{Error, Result};
use reqwest::{
    Method,
    header::HeaderMap,
};
use std::fmt::Write;
use url::Url;

mod ip_check;
mod resolver;
mod internal;
pub mod robotstxt;

pub use internal::RequestOptions;
pub use internal::UserAgentList;
pub use internal::ResponseWrapper;

pub async fn get(url: &str, options: &RequestOptions) -> Result<ResponseWrapper> {
    let response = internal::send(Method::GET, url, Some(options)).await;
    if let Err(e) = &response {
        let mut chain = e.chain();
        let mut message = String::new();
        if let Some(first) = chain.next() {
            let _ = write!(&mut message, "{first}");
        }
        for cause in chain {
            let _ = write!(&mut message, "\n  caused by: {cause}");
        }
        tracing::error!("Failed to fetch (URL: {}): {}", url, message);
    }

    Ok(response?.into())
}

pub async fn head(url: &str) -> Result<HeaderMap> {
    let response = internal::send(Method::HEAD, url, None).await?;
    Ok(response.headers().clone())
}

pub fn add_cookie(url: &Url, cookie_str: &str) {
    internal::COOKIE_JAR.add_cookie_str(cookie_str, url);
}
