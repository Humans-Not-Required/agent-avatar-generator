use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, Responder};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_REQUESTS: u32 = 200;

pub struct RateLimiter {
    window: Duration,
    state: Mutex<HashMap<IpAddr, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            state: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, ip: IpAddr) -> RateResult {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(poisoned) => poisoned.into_inner(),
        };

        let now = Instant::now();
        let entry = state.entry(ip).or_insert((now, 0));

        if now.duration_since(entry.0) > self.window {
            *entry = (now, 0);
        }

        entry.1 += 1;

        if entry.1 > MAX_REQUESTS {
            let remaining_secs = self.window.as_secs().saturating_sub(now.duration_since(entry.0).as_secs());
            RateResult {
                allowed: false,
                limit: MAX_REQUESTS,
                remaining: 0,
                retry_after: Some(remaining_secs),
            }
        } else {
            RateResult {
                allowed: true,
                limit: MAX_REQUESTS,
                remaining: MAX_REQUESTS - entry.1,
                retry_after: None,
            }
        }
    }
}

pub struct RateResult {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub retry_after: Option<u64>,
}

pub struct RateGuard {
    pub limit: u32,
    pub remaining: u32,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateGuard {
    type Error = serde_json::Value;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let limiter = req.rocket().state::<RateLimiter>().unwrap();
        let ip = req
            .client_ip()
            .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

        let result = limiter.check(ip);

        if result.allowed {
            Outcome::Success(RateGuard {
                limit: result.limit,
                remaining: result.remaining,
            })
        } else {
            let body = serde_json::json!({
                "error": "Rate limit exceeded",
                "retry_after_secs": result.retry_after.unwrap_or(60),
                "limit": result.limit,
                "remaining": 0,
            });
            Outcome::Error((Status::TooManyRequests, body))
        }
    }
}

/// Wrapper that attaches rate limit headers to any response.
pub struct RateLimited<T> {
    pub inner: T,
    pub limit: u32,
    pub remaining: u32,
}

impl<'r, T: Responder<'r, 'static>> Responder<'r, 'static> for RateLimited<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let mut response = self.inner.respond_to(req)?;
        response.set_raw_header("X-RateLimit-Limit", self.limit.to_string());
        response.set_raw_header("X-RateLimit-Remaining", self.remaining.to_string());
        Ok(response)
    }
}
