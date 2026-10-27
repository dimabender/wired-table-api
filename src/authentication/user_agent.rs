use axum::http::{HeaderMap, header::USER_AGENT};

const USER_AGENT_MAX_LEN: usize = 512;

pub fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(USER_AGENT)?.to_str().ok()?.trim();

    if value.is_empty() {
        return None;
    }

    Some(value.chars().take(USER_AGENT_MAX_LEN).collect())
}
