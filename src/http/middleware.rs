use crate::{authentication::token::verify_access_token, error::ApiError, state::AppState};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, header, request::Parts},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let header_value = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "missing authorization header"))?;

    let token = header_value
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "invalid authorization header"))?;

    let claims = verify_access_token(&state.secret, token)
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "invalid or expired token"))?;

    request.extensions_mut().insert(AuthUser {
        user_id: claims.user_id,
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
