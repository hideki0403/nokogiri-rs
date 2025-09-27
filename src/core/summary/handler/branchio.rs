use async_trait::async_trait;
use url::Url;
use crate::core::summary::{def::{SummalyHandler, SummaryResult}, summarize};

pub struct BranchioHandler;

#[async_trait]
impl SummalyHandler for BranchioHandler {
    fn id(&self) -> &str {
        "branchio"
    }

    fn test(&self, url: &Url) -> bool {
        let domain = url.domain().unwrap_or("");
        domain == "spotify.link" || domain.ends_with(".app.link")
    }

    async fn summarize(&self, url: &Url) -> Option<SummaryResult> {
        let mut fixed_url = url.clone();
        fixed_url.set_query(Some("$web_only=true"));

        let response = summarize::fetch(&fixed_url).await;
        if let Some(html) = response {
            summarize::generic_summarize(&fixed_url, html).await
        } else {
            None
        }
    }
}
