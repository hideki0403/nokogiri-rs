use axum::routing::{MethodRouter, get};

pub fn handler() -> MethodRouter {
    get(|| async {
        let version = env!("CARGO_PKG_VERSION");
        format!("nokogiri v{version} - https://github.com/hideki0403/nokogiri-rs")
    })
}
