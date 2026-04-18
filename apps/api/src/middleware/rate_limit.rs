use dashmap::DashMap;
use governor::{clock::DefaultClock, state::direct::NotKeyed, state::InMemoryState, Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

/// A per-user rate limiter that uses a Token Bucket algorithm.
#[derive(Debug, Clone)]
pub struct UserRateLimiter {
    limiters: Arc<
        DashMap<
            String,
            Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
        >,
    >,
    quota: Quota,
}

impl UserRateLimiter {
    /// Create a new limiter with a specific quota (e.g. 5 requests per minute with a burst of 10).
    pub fn new(requests_per_minute: u32, burst: u32) -> Self {
        Self {
            limiters: Arc::new(DashMap::new()),
            quota: Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap())
                .allow_burst(NonZeroU32::new(burst).unwrap()),
        }
    }

    /// Check if a user is allowed to perform an action.
    /// Returns true if allowed, false if rate limited.
    pub fn check(&self, user_id: &str) -> bool {
        let limiter = self
            .limiters
            .entry(user_id.to_string())
            .or_insert_with(|| Arc::new(RateLimiter::direct(self.quota)))
            .value()
            .clone();

        limiter.check().is_ok()
    }
}
