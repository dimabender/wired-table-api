use crate::{
    authentication::token::{REFRESH_TOKEN_TTL_DAYS, generate_refresh_token, issues_access_token},
    entity::session,
    error::ApiError,
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveValue::Set, ConnectionTrait, EntityTrait};
use uuid::Uuid;

pub struct IssuedTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub session: session::Model,
}

pub async fn issue_session_tokens<C: ConnectionTrait>(
    db: &C,
    secret: &[u8],
    user_id: Uuid,
) -> Result<IssuedTokens, ApiError> {
    let (refresh_token, refresh_hash) = generate_refresh_token();

    let session = session::ActiveModel {
        user_id: Set(user_id),
        token: Set(refresh_hash),
        expires_at: Set(Utc::now() + Duration::days(REFRESH_TOKEN_TTL_DAYS)),
        ..Default::default()
    };

    let session = session::Entity::insert(session)
        .exec_with_returning(db)
        .await?;

    let access_token = issues_access_token(secret, session.id, user_id)?;

    Ok(IssuedTokens {
        access_token,
        refresh_token,
        session: session,
    })
}
