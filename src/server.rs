use crate::config;
use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing,
};
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::Span;
use uuid::Uuid;

mod middleware;
mod route;

// Error handling
pub struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where E: Into<anyhow::Error>
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let request_id = Uuid::new_v4();
        tracing::error!(request_id = %request_id, "{}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error (RequestID: {request_id})"),
        )
            .into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

// Server setup
pub async fn listen() {
    let conf = &config::CONFIG;

    let app = Router::new()
        .route("/", route::index::handler())
        .route("/robots.txt", route::robots::handler())
        .route("/url", routing::get(route::url::handler))
        .layer(axum::middleware::from_fn(middleware::logger::request_logger))
        .layer(
            TraceLayer::new_for_http().on_response(|response: &Response, latency: Duration, _: &Span| {
                if let Some(request_logger) = response.extensions().get::<middleware::logger::RequestLogger>() {
                    tracing::info!(
                        parent: None,
                        "{} {} {} ({:.1}ms)",
                        request_logger.method,
                        response.status().as_str(),
                        request_logger.uri,
                        latency.as_secs_f64() * 1000.0
                    );
                }
            }),
        )
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

    let addr = format!("{}:{}", conf.server.host, conf.server.port);
    let listener = TcpListener::bind(&addr).await;
    if let Err(err) = listener {
        tracing::error!("Failed to bind to {}: {}", addr, err);
        return;
    }

    tracing::info!("Server listening on http://{}", addr);

    let server = axum::serve(listener.unwrap(), app).await;
    if let Err(err) = server {
        tracing::error!("Server error: {}", err);
    }
}
