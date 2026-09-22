use crate::{error::ApiError, http::middleware::AuthUser};
use axum::{
    Router,
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
};
use dashmap::DashMap;
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<String, (u32, Instant)>>,
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(limit: u32, window: Duration) -> Self {
        let limiter = Self {
            buckets: Arc::new(DashMap::new()),
            limit,
            window,
        };
        limiter.spawn_cleanup();
        limiter
    }

    fn check(&self, key: &str) -> bool {
        let now = Instant::now();

        match self.buckets.get_mut(key) {
            Some(mut entry) => {
                if now.duration_since(entry.1) > self.window {
                    *entry = (1, now);
                    true
                } else {
                    entry.0 += 1;
                    entry.0 <= self.limit
                }
            }
            None => {
                self.buckets.insert(key.to_string(), (1, now));
                true
            }
        }
    }

    fn spawn_cleanup(&self) {
        let buckets = self.buckets.clone();
        let window = self.window;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(window);
            loop {
                interval.tick().await;
                let now = Instant::now();
                buckets.retain(|_, (_, last_seen)| now.duration_since(*last_seen) <= window);
            }
        });
    }
}

async fn rate_limit_by_ip(
    State(limiter): State<RateLimiter>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if limiter.check(&addr.ip().to_string()) {
        Ok(next.run(req).await)
    } else {
        Err(ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "too many requests",
        ))
    }
}

async fn rate_limit_by_user(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let user_id = req
        .extensions()
        .get::<AuthUser>()
        .map(|u| u.user_id.to_string())
        .unwrap_or_else(|| "anonymous".into());

    if limiter.check(&user_id) {
        Ok(next.run(req).await)
    } else {
        Err(ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "too many requests",
        ))
    }
}

pub trait RateLimitExt<S> {
    fn rate_limit_ip(self, limit: u32, window: Duration) -> Self;

    fn rate_limit_user(self, limit: u32, window: Duration) -> Self;
}

impl<S> RateLimitExt<S> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn rate_limit_ip(self, limit: u32, window: Duration) -> Self {
        let limiter = RateLimiter::new(limit, window);
        self.route_layer(middleware::from_fn_with_state(limiter, rate_limit_by_ip))
    }

    fn rate_limit_user(self, limit: u32, window: Duration) -> Self {
        let limiter = RateLimiter::new(limit, window);
        self.route_layer(middleware::from_fn_with_state(limiter, rate_limit_by_user))
    }
}
