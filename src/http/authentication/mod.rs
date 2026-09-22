mod dto;
mod handlers;

use crate::{
    http::authentication::handlers::{login, logout, refresh, register},
    rate_limit::RateLimitExt,
    state::AppState,
};
use axum::{Router, routing::post};
use std::time::Duration;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(
            Router::new()
                .route("/register", post(register))
                .rate_limit_ip(5, Duration::from_mins(60)),
        )
        .merge(
            Router::new()
                .route("/login", post(login))
                .rate_limit_ip(5, Duration::from_mins(15)),
        )
        .merge(
            Router::new()
                .route("/refresh", post(refresh))
                .rate_limit_ip(20, Duration::from_mins(20)),
        )
        .merge(
            Router::new()
                .route("/logout", post(logout))
                .rate_limit_ip(10, Duration::from_mins(10)),
        )
}
