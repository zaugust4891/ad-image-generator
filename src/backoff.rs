use rand::Rng;
#[allow(dead_code)]
pub fn backoff_ms(attempt: u32, base_ms: u64, factor: f64, jitter_ms: u64) -> u64 {
    let pow = factor.powi((attempt.saturating_sub(1)) as i32);
    let core = (base_ms as f64 * pow).round() as u64;
    let jitter = if jitter_ms > 0 { rand::rng().random_range(0..=jitter_ms) } else { 0 };
    core + jitter
}
