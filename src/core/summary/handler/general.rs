use async_trait::async_trait;
use url::Url;
use crate::core::summary::{def::{SummalyHandler, SummaryResult}, summarize};

pub struct GeneralHandler;

#[async_trait]
impl SummalyHandler for GeneralHandler {
    fn id(&self) -> &str {
        "general"
    }

    fn test(&self, _url: &Url) -> bool {
        true
    }

    async fn summarize(&self, url: &Url) -> Option<SummaryResult> {
        let response = summarize::fetch(url).await;
        if let Some(html) = response {
            summarize::generic_summarize(url, html).await
        } else {
            None
        }
    }
}
