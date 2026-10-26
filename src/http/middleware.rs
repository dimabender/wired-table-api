use crate::{
    authentication::token::{SESSION_IDLE_TTL_DAYS, SESSION_REFRESH_THRESHOLD_DAYS, hash_token},
    entity::session,
    error::ApiError,
    state::AppState,
};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, IntoActiveModel};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

pub async fn require_auth(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = jar
        .get("__Host-session")
        .map(|c| c.value())
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "Missing session."))?;

    let token_hash = hash_token(token);

    let session = session::Entity::find_by_token(&token_hash)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "Session validate failed."))?;

    let now = Utc::now();
    if session.expires_at <= now || session.absolute_expires_at <= now {
        let _ = session::Entity::delete_by_id(session.id)
            .exec(&state.db)
            .await;
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "Session expired."));
    }

    let new_expires_at =
        (now + Duration::days(SESSION_IDLE_TTL_DAYS)).min(session.absolute_expires_at);
    if session.expires_at - now < Duration::days(SESSION_REFRESH_THRESHOLD_DAYS)
        && new_expires_at > session.expires_at
    {
        let mut active = session.clone().into_active_model();
        active.expires_at = Set(new_expires_at);
        active.update(&state.db).await?;
    }

    request.extensions_mut().insert(AuthUser {
        user_id: session.user_id,
        session_id: session.id,
    });

    Ok(next.run(request).await)
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<AuthUser>().cloned().ok_or_else(|| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "AuthUser extension missing — is require_auth layer applied?",
            )
        })
    }
}
