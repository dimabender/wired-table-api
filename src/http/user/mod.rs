mod dto;

use crate::{
    entity::user,
    error::ApiError,
    http::{middleware::AuthUser, user::dto::MeResponse},
    state::AppState,
};
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use sea_orm::EntityTrait;

async fn me(auth: AuthUser, State(state): State<AppState>) -> Result<Json<MeResponse>, ApiError> {
    let user = user::Entity::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "user not found"))?;

    Ok(Json(user.into()))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/me", get(me))
}
