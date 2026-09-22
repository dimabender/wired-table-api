use crate::{
    authentication::{
        password::{hash_password, verify_password},
        session::{IssuedTokens, issue_session_tokens},
        token::{REFRESH_TOKEN_TTL_DAYS, hash_token},
    },
    entity::{session, user},
    error::{ApiError, MessageItem},
    extract::ValidJson,
    http::authentication::dto::{LoginRequest, LoginResponse, RefreshResponse, RegisterRequest},
    state::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, ExprTrait, QueryFilter,
    TransactionTrait,
};

pub async fn register(
    State(state): State<AppState>,
    ValidJson(req): ValidJson<RegisterRequest>,
) -> Result<StatusCode, ApiError> {
    let matches = user::Entity::find()
        .filter(
            user::Column::Username
                .eq(&req.username)
                .or(user::Column::Email.eq(&req.email)),
        )
        .all(&state.db)
        .await?;

    let username_taken = matches.iter().any(|u| u.username == req.username);
    let email_taken = matches.iter().any(|u| u.email == req.email);

    let mut messages: Vec<MessageItem> = Vec::new();

    if username_taken {
        messages.push(MessageItem::new(
            Some("username".to_string()),
            "username already taken".to_string(),
        ));
    }

    if email_taken {
        messages.push(MessageItem::new(
            Some("email".to_string()),
            "email already taken".to_string(),
        ));
    }

    if !messages.is_empty() {
        return Err(ApiError::from_items(StatusCode::CONFLICT, messages));
    }

    let password = hash_password(&req.password)?;

    let user = user::ActiveModel {
        username: Set(req.username),
        email: Set(req.email),
        password: Set(password),
        ..Default::default()
    };

    user::Entity::insert(user).exec(&state.db).await?;

    Ok(StatusCode::CREATED)
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    ValidJson(req): ValidJson<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), ApiError> {
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&req.username))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "invalid username or password"))?;

    if !verify_password(&req.password, &user.password)? {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid username or password",
        ));
    }

    let tokens = issue_session_tokens(&state.db, &state.secret, user.id).await?;

    let refresh_cookie = build_refresh_cookie(tokens.refresh_token);

    Ok((
        jar.add(refresh_cookie),
        Json(LoginResponse {
            access_token: tokens.access_token,
        }),
    ))
}

pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<RefreshResponse>), ApiError> {
    let raw_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "no refresh token"))?;

    let token_hash = hash_token(&raw_token);

    let current_session = session::Entity::find()
        .filter(session::Column::Token.eq(&token_hash))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "invalid refresh token"))?;

    if current_session.revoked_at.is_some() {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "token reuse detected, all sessions revoked",
        ));
    }

    if current_session.expires_at < Utc::now() {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "refresh token expired",
        ));
    }

    let user_id = current_session.user_id;
    let secret = state.secret.clone();

    let tokens = state
        .db
        .transaction::<_, IssuedTokens, ApiError>(|txn| {
            Box::pin(async move {
                let tokens = issue_session_tokens(txn, &secret, user_id).await?;

                let mut old_session: session::ActiveModel = current_session.into();
                old_session.revoked_at = Set(Some(Utc::now()));
                old_session.replaced_by = Set(Some(tokens.session.id));
                old_session.update(txn).await?;

                Ok(tokens)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Connection(db_err) => ApiError::from(db_err),
            sea_orm::TransactionError::Transaction(db_err) => ApiError::from(db_err),
        })?;

    let refresh_cookie = build_refresh_cookie(tokens.refresh_token);

    Ok((
        jar.add(refresh_cookie),
        Json(RefreshResponse {
            access_token: tokens.access_token,
        }),
    ))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> Result<CookieJar, ApiError> {
    if let Some(cookie) = jar.get("refresh_token") {
        let token_hash = hash_token(cookie.value());

        if let Some(session) = session::Entity::find()
            .filter(session::Column::Token.eq(&token_hash))
            .one(&state.db)
            .await?
        {
            if session.revoked_at.is_none() {
                let mut session: session::ActiveModel = session.into();
                session.revoked_at = Set(Some(Utc::now()));
                session.update(&state.db).await?;
            }
        }
    }

    let expired_cookie = Cookie::build(("refresh_token", ""))
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .path("/authentication")
        .max_age(time::Duration::ZERO)
        .build();

    Ok(jar.add(expired_cookie))
}

fn build_refresh_cookie(raw_token: String) -> Cookie<'static> {
    Cookie::build(("refresh_token", raw_token))
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .path("/authentication")
        .max_age(time::Duration::days(REFRESH_TOKEN_TTL_DAYS))
        .build()
}
