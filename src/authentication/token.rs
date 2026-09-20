use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const ACCESS_TOKEN_TTL_MINUTES: i64 = 15;
pub const REFRESH_TOKEN_TTL_DAYS: i64 = 30;

pub fn generate_refresh_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    rng().fill_bytes(&mut bytes);

    let raw = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hash_token(&raw);

    (raw, hash)
}

pub fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());

    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

#[derive(Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub exp: usize,
}

pub fn issues_access_token(
    secret: &[u8],
    session_id: Uuid,
    user_id: Uuid,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (Utc::now() + Duration::minutes(ACCESS_TOKEN_TTL_MINUTES)).timestamp() as usize;
    let claims = AccessTokenClaims {
        session_id,
        user_id,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
}

pub fn verify_access_token(secret: &[u8], token: &str) -> Option<AccessTokenClaims> {
    decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims)
}
