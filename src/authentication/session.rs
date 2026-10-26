use crate::{
    authentication::token::{
        SESSION_ABSOLUTE_TTL_DAYS, SESSION_IDLE_TTL_DAYS, generate_session_token,
    },
    entity::session,
    error::ApiError,
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveValue::Set, ConnectionTrait, EntityTrait};
use uuid::Uuid;

pub struct IssuedSession {
    pub token: String,
    pub session: session::Model,
}

pub async fn create_session<C: ConnectionTrait>(
    db: &C,
    user_id: Uuid,
    user_agent: Option<String>,
) -> Result<IssuedSession, ApiError> {
    let (token, token_hash) = generate_session_token();
    let now = Utc::now();

    let session = session::ActiveModel {
        user_id: Set(user_id),
        token: Set(token_hash),
        expires_at: Set(now + Duration::days(SESSION_IDLE_TTL_DAYS)),
        absolute_expires_at: Set(now + Duration::days(SESSION_ABSOLUTE_TTL_DAYS)),
        user_agent: Set(user_agent),
        ..Default::default()
    };

    let session = session::Entity::insert(session)
        .exec_with_returning(db)
        .await?;

    Ok(IssuedSession { token, session })
}
