use crate::{entity::user, error::ApiError, http::middleware::AuthUser, state::AppState};
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use sea_orm::{EntityTrait, entity::prelude::DateTimeUtc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl From<user::Model> for MeResponse {
    fn from(model: user::Model) -> Self {
        Self {
            id: model.id,
            username: model.username,
            email: model.email,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

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
