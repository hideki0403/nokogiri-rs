use axum::{body::Body, http::Request, middleware::Next, response::Response};

#[derive(Clone)]
pub struct RequestLogger {
    pub uri: String,
    pub method: String,
}

pub async fn request_logger(request: Request<Body>, next: Next) -> Response {
    let request_logger = RequestLogger {
        uri: request.uri().to_string(),
        method: request.method().to_string(),
    };

    let mut response = next.run(request).await;
    response.extensions_mut().insert(request_logger);
    response
}
