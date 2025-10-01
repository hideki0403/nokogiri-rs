use async_trait::async_trait;
use url::Url;
use scraper::Html;
use htmlentity::{self, entity::ICodedDataTrait};
use crate::core::{request, summary::{def::*, selector, utility::{resolve_absolute_url, select_attr, select_text, text_clamp, url_exists_check}}};

pub async fn resolve_oembed(url: &Url, href: Option<String>, args: &SummarizeArguments) -> Option<Player> {
    if href.is_none() {
        return None;
    }

    let href = resolve_absolute_url(url, &href.unwrap())?;
    let response = request::get(&href, &args.into()).await.ok()?;
    let oembed = serde_json::from_str::<OEmbedData>(&response.text().await?.as_str()).ok()?;

    if oembed.version != "1.0" && oembed.r#type != "video" && oembed.r#type != "rich" {
        tracing::debug!("oembed type is not video or rich: {}", oembed.r#type);
        return None;
    }

    let oembed_html = oembed.html.trim();
    if !oembed_html.starts_with("<iframe ") || !oembed_html.ends_with("</iframe>") {
        tracing::debug!("invalid iframe in oembed html");
        return None;
    }

    let document = Html::parse_fragment(oembed_html);
    let mut iframes = vec![];

    for iframe in document.select(&selector::IFRAME) {
        iframes.push(iframe);
    }

    // iframe must be exactly one
    if iframes.len() != 1 {
        tracing::debug!("iframe count is not 1 in oembed html: {}", iframes.len());
        return None;
    }

    let iframe = &iframes[0].value();
    let iframe_url = iframe.attr("src")?;
    let iframe_url = Url::parse(iframe_url).ok()?;

    if iframe_url.scheme() != "https" {
        tracing::debug!("iframe url scheme is not https: {}", iframe_url);
        return None;
    }

    let width = iframe.attr("width").and_then(|w| w.parse::<u32>().ok()).or(Some(oembed.width));
    let mut height = iframe.attr("height").and_then(|h| h.parse::<u32>().ok()).or(Some(oembed.height));

    if height.is_none() {
        return None;
    } else if height.unwrap() > 1024 {
        height = Some(1024);
    }

    const ALLOWED_PERMISSION_POLICY: [&str; 6] = [
        "autoplay",
		"clipboard-write",
		"fullscreen",
		"encrypted-media",
		"picture-in-picture",
		"web-share",
    ];

    const IGNORED_PERMISSION_POLICY: [&str; 2] = [
        "accelerometer",
        "gyroscope",
    ];

    let permissions = iframe.attr("allow").unwrap_or("").split(";")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !IGNORED_PERMISSION_POLICY.contains(s))
        .collect::<Vec<&str>>();

    if permissions.iter().any(|p| !ALLOWED_PERMISSION_POLICY.contains(p)) {
        tracing::debug!("iframe has disallowed permission policy: {:?}", permissions);
        return None;
    }

    Some(Player {
        url: Some(iframe_url.to_string()),
        width,
        height,
        allow: permissions.iter().map(|s| s.to_string()).collect(),
    })
}

pub fn resolve_player(url: &Url, html: &Html, search_with_twitter_player: bool) -> Option<Player> {
    let mut player_url = if search_with_twitter_player {
        select_attr(html, "content", &[&selector::META_TWITTER_PLAYER_NAME, &selector::META_TWITTER_PLAYER_PROPERTY])
    } else {
        None
    };

    if player_url.is_none() {
        player_url = select_attr(html, "content", &[&selector::META_OG_VIDEO_PROPERTY, &selector::META_OG_VIDEO_SECURE_URL_PROPERTY, &selector::META_OG_VIDEO_URL_PROPERTY]);
    }

    if player_url.is_none() {
        return None;
    }

    let player_width = select_attr(html, "content", &[&selector::META_TWITTER_PLAYER_WIDTH_NAME, &selector::META_TWITTER_PLAYER_WIDTH_PROPERTY, &selector::META_OG_VIDEO_WIDTH_PROPERTY])
        .and_then(|x| x.parse::<u32>().ok());

    let player_height = select_attr(html, "content", &[&selector::META_TWITTER_PLAYER_HEIGHT_NAME, &selector::META_TWITTER_PLAYER_HEIGHT_PROPERTY, &selector::META_OG_VIDEO_HEIGHT_PROPERTY])
        .and_then(|x| x.parse::<u32>().ok());

    Some(Player {
        url: player_url.and_then(|u| resolve_absolute_url(url, &u)),
        width: player_width,
        height: player_height,
        allow: ["autoplay".to_string(), "encrypted-media".to_string(), "fullscreen".to_string()].to_vec(),
    })
}

pub async fn generic_summarize(url: &Url, str_html: String, args: &SummarizeArguments) -> Option<SummaryResult> {
    execute_summarize(url, str_html, args, &GenericSummarizeHandler).await
}

pub async fn execute_summarize(url: &Url, str_html: String, args: &SummarizeArguments, handler: &dyn SummarizeHandler) -> Option<SummaryResult> {
    let html = Html::parse_document(str_html.as_str());

    let title = handler.title(url, &html);
    if title.is_none() {
        tracing::debug!("Title not found");
        return None;
    }

    let title = match htmlentity::entity::decode(title.unwrap().as_bytes()).to_string() {
        Ok(x) => text_clamp(&x, 100),
        Err(_) => return None,
    };

    let is_large_summary_image = handler.summary_large_image(url, &html);
    let mut image = handler.thumbnail(url, &html);

    if image.is_some() {
        image = resolve_absolute_url(url, &image.unwrap());
    }

    let favicon = handler.icon(url, &html);
    let oembed_href = handler.extract_oembed_url(url, &html);
    let (oembed, favicon_available) = tokio::join!(handler.oembed(url, oembed_href, args), handler.icon_exists(&favicon));

    let player = oembed
        .or_else(|| handler.player(url, &html, is_large_summary_image))
        .unwrap_or(Player {
            url: None,
            width: None,
            height: None,
            allow: vec![],
        });

    let mut description = handler.description(url, &html);

    if description.is_some() {
        description = match htmlentity::entity::decode(description.unwrap().as_bytes()).to_string() {
            Ok(x) => Some(text_clamp(&x, 300)),
            Err(_) => None,
        };
    }

    let mut sitename = handler.sitename(url, &html);

    if sitename.is_none() {
        sitename = match url.domain() {
            Some(domain) => Some(domain.to_string()),
            None => None,
        };
    }

    let activity_pub = handler.activity_pub(url, &html);
    let fediverse_creator = handler.fediverse_creator(url, &html);
    let sensitive = handler.sensitive(url, &html);

    Some(SummaryResult {
        title,
        icon: if favicon_available { Some(favicon.unwrap().to_string()) } else { None },
        description,
        sitename,
        thumbnail: image,
        player,
        sensitive,
        activity_pub,
        fediverse_creator,
        large_card: Some(is_large_summary_image),
    })
}

pub struct GenericSummarizeHandler;

#[async_trait]
impl SummarizeHandler for GenericSummarizeHandler {
    fn title(&self, _url: &Url, html: &Html) -> Option<String> {
        select_attr(&html, "content", &[&selector::META_OG_TITLE_PROPERTY, &selector::META_TWITTER_TITLE_NAME, &selector::META_TWITTER_TITLE_PROPERTY])
            .or_else(|| select_text(&html, &selector::TITLE))
    }

    fn icon(&self, url: &Url, html: &Html) -> Option<Url> {
        let f = match select_attr(&html, "href", &[&selector::LINK_ICON_REL, &selector::LINK_SHORTCUT_ICON_REL]) {
            Some(favicon) => favicon,
            None => "/favicon.ico".to_string(),
        };

        let f = resolve_absolute_url(url, &f)?;
        Url::parse(&f).ok()
    }

    async fn icon_exists(&self, url: &Option<Url>) -> bool {
        url_exists_check(&url).await
    }

    fn description(&self, _url: &Url, html: &Html) -> Option<String> {
        select_attr(&html, "content", &[&selector::META_OG_DESCRIPTION_PROPERTY, &selector::META_TWITTER_DESCRIPTION_NAME, &selector::META_TWITTER_DESCRIPTION_PROPERTY, &selector::META_DESCRIPTION_NAME])
            .and_then(|d| match htmlentity::entity::decode(d.as_bytes()).to_string() {
                Ok(x) => Some(text_clamp(&x, 300)),
                Err(_) => None,
            })
    }

    fn sitename(&self, url: &Url, html: &Html) -> Option<String> {
        if let Some(name) = select_attr(&html, "content", &[&selector::META_OG_SITE_NAME_PROPERTY, &selector::META_APPLICATION_NAME_NAME]) {
            match htmlentity::entity::decode(name.as_bytes()).to_string() {
                Ok(x) => Some(x),
                Err(_) => None,
            }
        } else {
            match url.domain() {
                Some(domain) => Some(domain.to_string()),
                None => None,
            }
        }
    }

    fn thumbnail(&self, url: &Url, html: &Html) -> Option<String> {
        select_attr(&html, "content", &[&selector::META_OG_IMAGE_PROPERTY, &selector::META_TWITTER_IMAGE_NAME, &selector::META_TWITTER_IMAGE_PROPERTY])
            .or_else(|| select_attr(&html, "href", &[&selector::LINK_IMAGE_SRC_REL, &selector::LINK_APPLE_TOUCH_ICON_REL]))
            .and_then(|img| resolve_absolute_url(url, &img))
    }

    fn extract_oembed_url(&self, _url: &Url, html: &Html) -> Option<String> {
        select_attr(&html, "href", &[&selector::LINK_JSON_OEMBED_TYPE])
    }

    async fn oembed(&self, url: &Url, href: Option<String>, args: &SummarizeArguments) -> Option<Player> {
        resolve_oembed(url, href, args).await
    }

    fn player(&self, url: &Url, html: &Html, is_summary_large_image: bool) -> Option<Player> {
        resolve_player(url, html, is_summary_large_image)
    }

    fn sensitive(&self, _url: &Url, html: &Html) -> Option<bool> {
        if let Some(s) = select_attr(&html, "content", &[&selector::META_MIXI_CONTENT_RATING_PROPERTY]) {
            Some(s == "true" || s == "1")
        } else if let Some(s) = select_attr(&html, "content", &[&selector::META_RATING_NAME]) {
            let x = s.to_uppercase();
            Some(x == "ADULT" || x == "RTA-5042-1996-1400-1577-RTA")
        } else {
            None
        }
    }

    fn activity_pub(&self, _url: &Url, html: &Html) -> Option<String> {
        select_attr(&html, "href", &[&selector::LINK_ALTERNATE_ACTIVITYJSON_TYPE])
    }

    fn fediverse_creator(&self, _url: &Url, html: &Html) -> Option<String> {
        select_attr(&html, "content", &[&selector::META_FEDIVERSE_CREATOR_NAME])
    }

    fn summary_large_image(&self, _url: &Url, html: &Html) -> bool {
        let x = select_attr(&html, "content", &[&selector::META_TWITTER_CARD_NAME, &selector::META_TWITTER_CARD_PROPERTY]);
        x.is_some_and(|v| v == "summary_large_image")
    }
}
