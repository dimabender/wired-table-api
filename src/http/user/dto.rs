use sea_orm::entity::prelude::DateTimeUtc;
use serde::Serialize;
use uuid::Uuid;

use crate::entity::user;

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
