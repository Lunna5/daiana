//! High-performance Token Bucket Rate Limiter with sub-millisecond precision.

use std::time::Instant;

/// Token-bucket rate limiter for pacing inbound packets and preventing burst abuse.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_rate_per_sec: f64,
    last_refill: Instant,
    consecutive_violations: u32,
}

impl TokenBucket {
    /// Creates a new [`TokenBucket`].
    ///
    /// - `rate_per_sec`: Number of permitted tokens (packets) per second (0 to disable rate limiting).
    /// - `burst_capacity`: Maximum burst size accumulated during idle periods.
    pub fn new(rate_per_sec: u32, burst_capacity: u32) -> Self {
        let cap = burst_capacity.max(rate_per_sec) as f64;
        Self {
            capacity: cap,
            tokens: cap,
            refill_rate_per_sec: rate_per_sec as f64,
            last_refill: Instant::now(),
            consecutive_violations: 0,
        }
    }

    /// Attempts to consume 1 token.
    ///
    /// Returns `true` if permitted, or `false` if the rate limit is exceeded.
    pub fn try_consume(&mut self) -> bool {
        self.try_consume_tokens(1.0)
    }

    /// Attempts to consume a specified amount of tokens.
    pub fn try_consume_tokens(&mut self, amount: f64) -> bool {
        if self.refill_rate_per_sec <= 0.0 {
            return true;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;

        // Refill tokens continuously proportional to elapsed time
        self.tokens = (self.tokens + elapsed * self.refill_rate_per_sec).min(self.capacity);

        if self.tokens >= amount {
            self.tokens -= amount;
            self.consecutive_violations = 0;
            true
        } else {
            self.consecutive_violations = self.consecutive_violations.saturating_add(1);
            false
        }
    }

    /// Returns the current number of consecutive dropped packets due to rate limit exhaustion.
    pub fn consecutive_violations(&self) -> u32 {
        self.consecutive_violations
    }

    /// Resets the consecutive violations counter.
    pub fn reset_violations(&mut self) {
        self.consecutive_violations = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_token_bucket_disabled_when_zero_rate() {
        let mut limiter = TokenBucket::new(0, 0);
        for _ in 0..1000 {
            assert!(limiter.try_consume());
        }
        assert_eq!(limiter.consecutive_violations(), 0);
    }

    #[test]
    fn test_token_bucket_burst_and_exhaustion() {
        let mut limiter = TokenBucket::new(10, 10);

        // Consume initial burst capacity
        for _ in 0..10 {
            assert!(limiter.try_consume());
        }

        // 11th should be denied
        assert!(!limiter.try_consume());
        assert_eq!(limiter.consecutive_violations(), 1);

        // Consecutive violations increment
        assert!(!limiter.try_consume());
        assert_eq!(limiter.consecutive_violations(), 2);
    }

    #[test]
    fn test_token_bucket_refill_over_time() {
        let mut limiter = TokenBucket::new(20, 20);

        // Exhaust capacity
        for _ in 0..20 {
            assert!(limiter.try_consume());
        }
        assert!(!limiter.try_consume());

        // Wait for refill (100ms should replenish ~2 tokens at 20/sec)
        sleep(Duration::from_millis(120));

        assert!(limiter.try_consume());
        assert_eq!(limiter.consecutive_violations(), 0);
    }
}
