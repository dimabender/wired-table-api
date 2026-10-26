use axum::http::{HeaderValue, Method, header};
use std::env;
use tower_http::cors::CorsLayer;

pub fn cors_layer() -> CorsLayer {
    let origins = env::var("CLIENT_URL").expect("CLIENT_URL must be set");

    let origins: Vec<HeaderValue> = origins
        .split(',')
        .map(|origin| {
            origin
                .trim()
                .parse()
                .expect("CLIENT_URL must contain valid origins")
        })
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(true)
}
