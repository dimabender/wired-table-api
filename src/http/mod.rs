mod authentication;
mod health;
pub mod middleware;
mod user;

use crate::state::AppState;
use axum::{Router, middleware::from_fn_with_state};

pub fn router(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .nest("/user", user::routes())
        .route_layer(from_fn_with_state(state.clone(), middleware::require_auth));

    Router::new()
        .nest("/authentication", authentication::routes())
        .merge(protected)
        .merge(health::routes())
}
