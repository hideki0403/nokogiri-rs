use axum::routing::{MethodRouter, get};

pub fn handler() -> MethodRouter {
    get(|| async { "User-agent: *\nDisallow: /" })
}