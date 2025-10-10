use rand::Rng;

/// Compute sleep duration in milliseconds for attempt number (1-based).
/// base_ms: starting delay (e.g., 300)
/// factor:  growth rate (e.g., 2.0)
/// jitter_ms: add random 0..=jitter_ms
pub fn backoff_ms(attempt: u32, base_ms: u64, factor: f64, jitter_ms: u64) -> u64 {
    let pow = factor.powi((attempt.saturating_sub(1)) as i32);
    let core = (base_ms as f64 * pow).round() as u64;
    let jitter = if jitter_ms > 0 {
        rand::thread_rng().gen_range(0..=jitter_ms)
    } else { 0 };
    core + jitter
}
