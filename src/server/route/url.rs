use axum::{extract::Query, http::HeaderMap, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::Deserialize;
use url::Url;
use urlencoding::decode;
use crate::{config::CONFIG, core::summary::{def::SummarizeArguments, summary}, server::AppResult};

#[derive(Deserialize, Debug)]
pub struct ReqParams {
    url: Option<String>,
    lang: Option<String>,
    #[serde(rename = "userAgent")]
    user_agent: Option<String>,
    // #[serde(rename = "followRedirects")]
    // follow_redirects: Option<bool>,
    // #[serde(rename = "responseTimeout")]
    // response_timeout: Option<u64>,
    // #[serde(rename = "operationTimeout")]
    // operation_timeout: Option<u64>,
    // #[serde(rename = "contentLengthLimit")]
    // content_length_limit: Option<usize>,
    // #[serde(rename = "contentLengthRequired")]
    // content_length_required: Option<bool>,
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

    let arguments = SummarizeArguments {
        url: url.clone(),
        lang: params.lang,
        user_agent: params.user_agent,
        // follow_redirects: params.follow_redirects,
        // response_timeout: params.response_timeout,
        // operation_timeout: params.operation_timeout,
        // content_length_limit: params.content_length_limit,
        // content_length_required: params.content_length_required,
    };

    let summary = summary(arguments).await;
    if summary.is_none() {
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "Failed to summarize the URL").into_response());
    }

    let mut headers = HeaderMap::new();
    headers.insert("Cache-Control", "public, max-age=604800".parse().unwrap());

    let response = Json(summary.unwrap()).into_response();
    Ok((headers, response).into_response())
}
