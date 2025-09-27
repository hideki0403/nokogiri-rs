use once_cell::sync::Lazy;
use scraper::Selector;

pub fn s(selector: &'static str) -> Selector {
    Selector::parse(selector).unwrap()
}

// Twitter Card
pub static META_TWITTER_CARD_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="twitter:card"]"#));
pub static META_TWITTER_CARD_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="twitter:card"]"#));

// Title
pub static META_OG_TITLE_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="og:title"]"#));
pub static META_TWITTER_TITLE_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="twitter:title"]"#));
pub static META_TWITTER_TITLE_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="twitter:title"]"#));
pub static TITLE: Lazy<Selector> = Lazy::new(|| s("title"));

// Image
pub static META_OG_IMAGE_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="og:image"]"#));
pub static META_TWITTER_IMAGE_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="twitter:image"]"#));
pub static META_TWITTER_IMAGE_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="twitter:image"]"#));
pub static LINK_IMAGE_SRC_REL: Lazy<Selector> = Lazy::new(|| s(r#"link[rel="image_src"]"#));
pub static LINK_APPLE_TOUCH_ICON_REL: Lazy<Selector> = Lazy::new(|| s(r#"link[rel="apple-touch-icon"]"#));

// Player - url
pub static META_TWITTER_PLAYER_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="twitter:player"]"#));
pub static META_TWITTER_PLAYER_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="twitter:player"]"#));
pub static META_OG_VIDEO_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="og:video"]"#));
pub static META_OG_VIDEO_SECURE_URL_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="og:video:secure_url"]"#));
pub static META_OG_VIDEO_URL_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="og:video:url"]"#));

// Player - width
pub static META_TWITTER_PLAYER_WIDTH_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="twitter:player:width"]"#));
pub static META_TWITTER_PLAYER_WIDTH_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="twitter:player:width"]"#));
pub static META_OG_VIDEO_WIDTH_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="og:video:width"]"#));

// Player - height
pub static META_TWITTER_PLAYER_HEIGHT_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="twitter:player:height"]"#));
pub static META_TWITTER_PLAYER_HEIGHT_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="twitter:player:height"]"#));
pub static META_OG_VIDEO_HEIGHT_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="og:video:height"]"#));

// Description
pub static META_OG_DESCRIPTION_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="og:description"]"#));
pub static META_TWITTER_DESCRIPTION_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="twitter:description"]"#));
pub static META_TWITTER_DESCRIPTION_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="twitter:description"]"#));
pub static META_DESCRIPTION_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="description"]"#));

// Sitename
pub static META_OG_SITE_NAME_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="og:site_name"]"#));
pub static META_APPLICATION_NAME_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="application-name"]"#));

// Favicon
pub static LINK_ICON_REL: Lazy<Selector> = Lazy::new(|| s(r#"link[rel="icon"]"#));
pub static LINK_SHORTCUT_ICON_REL: Lazy<Selector> = Lazy::new(|| s(r#"link[rel="shortcut icon"]"#));

// ActivityPub
pub static LINK_ALTERNATE_ACTIVITYJSON_TYPE: Lazy<Selector> = Lazy::new(|| s(r#"link[rel="alternate"][type="application/activity+json"]"#));

// FediverseCreator
pub static META_FEDIVERSE_CREATOR_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="fediverse:creator"]"#));

// Sensitive
pub static META_MIXI_CONTENT_RATING_PROPERTY: Lazy<Selector> = Lazy::new(|| s(r#"meta[property="mixi:content_rating"]"#));
pub static META_RATING_NAME: Lazy<Selector> = Lazy::new(|| s(r#"meta[name="rating"]"#));

// oEmbed
pub static LINK_JSON_OEMBED_TYPE: Lazy<Selector> = Lazy::new(|| s(r#"link[type="application/json+oembed"]"#));

// iframe
pub static IFRAME: Lazy<Selector> = Lazy::new(|| s("iframe"));