use std::fmt;

use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    messages: Vec<MessageItem>,
}

#[derive(Serialize, Debug)]
pub struct MessageItem {
    field: Option<String>,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            messages: vec![MessageItem {
                field: None,
                message: message.into(),
            }],
        }
    }

    pub fn from_items(status: StatusCode, messages: Vec<MessageItem>) -> Self {
        Self { status, messages }
    }
}

impl MessageItem {
    pub fn new(field: Option<String>, message: String) -> Self {
        Self { field, message }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    messages: Vec<MessageItem>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                messages: self.messages,
            }),
        )
            .into_response()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rej: JsonRejection) -> Self {
        Self::new(rej.status(), rej.body_text())
    }
}

impl From<garde::Report> for ApiError {
    fn from(report: garde::Report) -> Self {
        let messages: Vec<MessageItem> = report
            .iter()
            .map(|(path, e)| MessageItem::new(Some(path.to_string()), e.to_string()))
            .collect();
        Self::from_items(StatusCode::UNPROCESSABLE_ENTITY, messages)
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let texts: Vec<&str> = self.messages.iter().map(|m| m.message.as_str()).collect();
        write!(f, "{}", texts.join(", "))
    }
}
