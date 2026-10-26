use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{Rng, rng};
use sha2::{Digest, Sha256};

pub const SESSION_IDLE_TTL_DAYS: i64 = 30;
pub const SESSION_ABSOLUTE_TTL_DAYS: i64 = 90;
pub const SESSION_REFRESH_THRESHOLD_DAYS: i64 = 15;

pub fn generate_session_token() -> (String, String) {
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
