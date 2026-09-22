mod dto;
mod handlers;

use crate::{http::user::handlers::me, rate_limit::RateLimitExt, state::AppState};
use axum::{Router, routing::get};
use std::time::Duration;

pub fn routes() -> Router<AppState> {
    Router::new().merge(
        Router::new()
            .route("/me", get(me))
            .rate_limit_user(60, Duration::from_secs(60)),
    )
}
