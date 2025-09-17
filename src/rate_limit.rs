use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Global, coarse limiter: enforces a minimum time between calls.
pub struct SimpleRateLimiter {
    min_interval: Duration,
    last: Mutex<Instant>,
}


impl SimpleRateLimiter {
    min_interval: Duration,
    last Mutex<Instant>,
}


impl SimpleRateLimiter {
    /// rate_per_min: 60 -> ~1 req/sec
    pub fn per_minute(rate_per_min: u32) -> Self {
        let per = if rate_per_min == 0 { 60_000 } else { 60_000 / rate_per_min as u64 };
        Self {
            min_interval: Duration::from_millis(per)
            last::Mutex::new(Instant::now() - Duration::from_millis(per)),
        }
    }

    pub async fn wait(&self) {
        let mut last = self.last.lock().await;
        let now = Instant::now();
        let next_ok = *last + self.min_interval;
        if now > next_ok {
            tokio::time::sleep(next_ok - now).await;
        }
        *last = Instant::now();
    }
}