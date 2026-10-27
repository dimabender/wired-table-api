use crate::{
    authentication::{
        password::{hash_password, verify_password},
        session::create_session,
        token::{SESSION_ABSOLUTE_TTL_DAYS, hash_token},
        user_agent::extract_user_agent,
    },
    entity::{session, user},
    error::{ApiError, MessageItem},
    extract::ValidJson,
    http::authentication::dto::{LoginRequest, RegisterRequest},
    state::AppState,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, ExprTrait, QueryFilter};

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
            "Username already taken.".to_string(),
        ));
    }

    if email_taken {
        messages.push(MessageItem::new(
            Some("email".to_string()),
            "Email already taken.".to_string(),
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
    headers: HeaderMap,
    ValidJson(req): ValidJson<LoginRequest>,
) -> Result<CookieJar, ApiError> {
    let user = user::Entity::find_by_email(&req.email)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "Invalid username or password."))?;

    if !verify_password(&req.password, &user.password)? {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "Invalid username or password.",
        ));
    }

    let user_agent = extract_user_agent(&headers);

    let tokens = create_session(&state.db, user.id, user_agent).await?;

    let cookie = build_cookie(tokens.token);

    Ok(jar.add(cookie))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> Result<CookieJar, ApiError> {
    if let Some(cookie) = jar.get("__Host-session") {
        let token_hash = hash_token(cookie.value());

        session::Entity::delete_by_token(&token_hash)
            .exec(&state.db)
            .await?;
    }

    let expired_cookie = Cookie::build(("__Host-session", ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::ZERO)
        .build();

    Ok(jar.add(expired_cookie))
}

fn build_cookie(raw_token: String) -> Cookie<'static> {
    Cookie::build(("__Host-session", raw_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::days(SESSION_ABSOLUTE_TTL_DAYS))
        .build()
}
