use scraper::Html;
use std::ops::Deref;
use url::Url;

use crate::core::request;

pub fn select_attr(html: &Html, attr: &str, selectors: &[&(dyn Deref<Target = scraper::Selector> + Sync)]) -> Option<String> {
    for selector in selectors {
        if let Some(element) = html.select(selector).next() &&
            let Some(value) = element.value().attr(attr)
        {
            return Some(value.to_string());
        }
    }

    None
}

pub fn select_text(html: &Html, selector: &scraper::Selector) -> Option<String> {
    html.select(selector).next().map(|element| element.text().collect())
}

pub fn text_clamp(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if i >= max_len {
                break;
            }
            result.push(c);
        }
        result.push_str("...");
        result
    }
}

pub fn resolve_absolute_url(base: &Url, relative: &str) -> Option<String> {
    match base.join(relative) {
        Ok(url) => Some(url.to_string()),
        Err(_) => None,
    }
}

pub async fn url_exists_check(url: &Option<Url>) -> bool {
    if url.is_none() {
        return false;
    }
    request::head(url.as_ref().unwrap().as_str()).await.is_ok()
}
