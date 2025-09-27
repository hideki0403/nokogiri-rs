use axum::{extract::Query, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::Deserialize;
use url::Url;
use urlencoding::decode;
use crate::{config::CONFIG, core::summary::summary, server::AppResult};

#[derive(Deserialize, Debug)]
pub struct ReqParams {
    url: Option<String>,
    #[serde(rename = "secretKey")]
    secret_key: Option<String>,
}

pub async fn handler(Query(params): Query<ReqParams>) -> AppResult<impl IntoResponse> {
    let url_str = params.url;
    if url_str.is_none() {
        return Ok((StatusCode::BAD_REQUEST, "Missing 'url' parameter").into_response());
    }

    let secret_key = &CONFIG.security.secret_key;
    if !secret_key.is_empty() {
        let provided_key = params.secret_key;
        if provided_key.is_none() || provided_key.unwrap() != *secret_key {
            return Ok((StatusCode::UNAUTHORIZED, "Invalid secret key").into_response());
        }
    }

    let url_string = url_str.unwrap();
    let decoded_url = decode(&url_string);
    if decoded_url.is_err() {
        return Ok((StatusCode::BAD_REQUEST, "URL Decode failed").into_response());
    }

    let url = Url::parse(decoded_url.unwrap().as_ref());
    if url.is_err() {
        return Ok((StatusCode::BAD_REQUEST, "Invalid URL").into_response());
    }
    let url = url.unwrap();

    let scheme = url.scheme();
    if !matches!(scheme, "http" | "https") {
        return Ok((StatusCode::BAD_REQUEST, "Only http and https are supported").into_response());
    }

    let summary = summary(&url).await;
    if summary.is_none() {
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "Failed to summarize the URL").into_response());
    }

    Ok(Json(summary.unwrap()).into_response())
}
