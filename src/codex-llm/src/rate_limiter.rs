//! Rate limit monitor + circuit breaker for Ollama Cloud.
//!
//! Ollama Cloud uses GPU-time-based limits (not RPM/TPM). Limits are opaque:
//! - 5-hour session reset + 7-day weekly reset
//! - HTTP 429 on exhaustion, no X-RateLimit headers
//! - Pro tier: subscription-based, ~0.7% weekly usage observed at moderate load
//!
//! Since we can't predict limits, we monitor and react:
//! - Track every request (model, tokens, status, latency)
//! - Circuit breaker trips on consecutive failures (429/500/502)
//! - Usage dashboard for self-monitoring via `ollama_usage` tool

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

// ── Configuration ────────────────────────────────────────────────────────────

/// Number of consecutive error responses that trips the circuit breaker.
const BREAKER_THRESHOLD: u32 = 5;

/// Cooldown period in seconds when circuit breaker is open.
const COOLDOWN_SECS: u64 = 60;

/// Estimated cost per million input tokens (M2.7 pricing).
const COST_PER_M_INPUT: f64 = 0.30;

/// Estimated cost per million output tokens (M2.7 pricing).
const COST_PER_M_OUTPUT: f64 = 1.20;

// ── Types ────────────────────────────────────────────────────────────────────

/// A single request record logged to JSONL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRecord {
    pub timestamp: String,
    pub model: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub status_code: u16,
    pub latency_ms: u64,
    /// Whether the circuit breaker was open when this was recorded.
    pub breaker_open: bool,
}

/// Circuit breaker state.
#[derive(Debug, Clone)]
enum BreakerState {
    Closed,
    Open { until: chrono::DateTime<Utc> },
}

/// Internal mutable state behind the Arc<Mutex<>>.
#[derive(Debug)]
struct Inner {
    /// Consecutive error count (429, 500, 502, 503, 504).
    consecutive_errors: u32,
    /// Circuit breaker state.
    breaker: BreakerState,
    /// Rolling counters for the current day.
    today_date: String,
    today_requests: u64,
    today_tokens_in: u64,
    today_tokens_out: u64,
    today_errors: u64,
    today_429s: u64,
    /// Total latency (for average calculation).
    today_latency_sum: u64,
}

impl Inner {
    fn new() -> Self {
        Self {
            consecutive_errors: 0,
            breaker: BreakerState::Closed,
            today_date: Utc::now().format("%Y-%m-%d").to_string(),
            today_requests: 0,
            today_tokens_in: 0,
            today_tokens_out: 0,
            today_errors: 0,
            today_429s: 0,
            today_latency_sum: 0,
        }
    }

    /// Roll over counters if the date changed.
    fn maybe_rollover(&mut self) {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        if today != self.today_date {
            self.today_date = today;
            self.today_requests = 0;
            self.today_tokens_in = 0;
            self.today_tokens_out = 0;
            self.today_errors = 0;
            self.today_429s = 0;
            self.today_latency_sum = 0;
        }
    }
}

// ── RateLimiter ──────────────────────────────────────────────────────────────

/// Rate limit monitor + circuit breaker.
///
/// Thread-safe (Arc<Mutex<>>). Clone is cheap.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<Inner>>,
    metrics_dir: PathBuf,
}

impl RateLimiter {
    /// Create a new rate limiter. `metrics_dir` is where `ollama-usage.jsonl` is written.
    pub fn new(metrics_dir: PathBuf) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
            metrics_dir,
        }
    }

    /// Check if the circuit breaker allows a request.
    ///
    /// Returns `Ok(())` if the request can proceed.
    /// Returns `Err(secs_remaining)` if the breaker is open.
    pub async fn check(&self) -> Result<(), u64> {
        let mut inner = self.inner.lock().await;
        inner.maybe_rollover();

        match &inner.breaker {
            BreakerState::Closed => Ok(()),
            BreakerState::Open { until } => {
                let now = Utc::now();
                if now >= *until {
                    // Cooldown expired — half-open: allow the request, reset on success
                    info!("Circuit breaker cooldown expired — half-open, allowing request");
                    inner.breaker = BreakerState::Closed;
                    inner.consecutive_errors = 0;
                    Ok(())
                } else {
                    let remaining = (*until - now).num_seconds().max(0) as u64;
                    Err(remaining)
                }
            }
        }
    }

    /// Record a completed request. Updates counters, logs to JSONL, manages breaker.
    pub async fn record(
        &self,
        model: &str,
        tokens_in: u32,
        tokens_out: u32,
        status_code: u16,
        latency_ms: u64,
    ) {
        let mut inner = self.inner.lock().await;
        inner.maybe_rollover();

        let is_error = matches!(status_code, 429 | 500 | 502 | 503 | 504);
        let breaker_was_open = matches!(inner.breaker, BreakerState::Open { .. });

        // Update counters
        inner.today_requests += 1;
        inner.today_tokens_in += tokens_in as u64;
        inner.today_tokens_out += tokens_out as u64;
        inner.today_latency_sum += latency_ms;

        if is_error {
            inner.today_errors += 1;
            inner.consecutive_errors += 1;

            if status_code == 429 {
                inner.today_429s += 1;
            }

            // Trip the circuit breaker if threshold reached
            if inner.consecutive_errors >= BREAKER_THRESHOLD
                && !matches!(inner.breaker, BreakerState::Open { .. })
            {
                let until = Utc::now()
                    + chrono::Duration::seconds(COOLDOWN_SECS as i64);
                warn!(
                    consecutive = inner.consecutive_errors,
                    cooldown_secs = COOLDOWN_SECS,
                    "Circuit breaker OPEN — pausing all Ollama requests"
                );
                inner.breaker = BreakerState::Open { until };
            }
        } else {
            // Success — reset consecutive error counter
            inner.consecutive_errors = 0;
            // If we were in half-open state (breaker just closed), confirm closed
            if breaker_was_open {
                info!("Circuit breaker confirmed CLOSED — request succeeded after cooldown");
            }
        }

        // Build record
        let record = RequestRecord {
            timestamp: Utc::now().to_rfc3339(),
            model: model.to_string(),
            tokens_in,
            tokens_out,
            status_code,
            latency_ms,
            breaker_open: matches!(inner.breaker, BreakerState::Open { .. }),
        };

        // Drop lock before file I/O
        drop(inner);

        // Append to JSONL (best-effort, don't fail the request)
        if let Err(e) = self.append_record(&record) {
            warn!(error = %e, "Failed to write usage record to JSONL");
        }
    }

    /// Get a usage summary for the current day.
    pub async fn usage_summary(&self) -> UsageSummary {
        let inner = self.inner.lock().await;

        let breaker_state = match &inner.breaker {
            BreakerState::Closed => "closed".to_string(),
            BreakerState::Open { until } => {
                let now = Utc::now();
                if now >= *until {
                    "half-open (cooldown expired)".to_string()
                } else {
                    let secs = (*until - now).num_seconds().max(0);
                    format!("OPEN ({secs}s remaining)")
                }
            }
        };

        let avg_latency = if inner.today_requests > 0 {
            inner.today_latency_sum / inner.today_requests
        } else {
            0
        };

        let error_rate = if inner.today_requests > 0 {
            (inner.today_errors as f64 / inner.today_requests as f64) * 100.0
        } else {
            0.0
        };

        let est_cost_input = (inner.today_tokens_in as f64 / 1_000_000.0) * COST_PER_M_INPUT;
        let est_cost_output = (inner.today_tokens_out as f64 / 1_000_000.0) * COST_PER_M_OUTPUT;

        UsageSummary {
            date: inner.today_date.clone(),
            total_requests: inner.today_requests,
            total_tokens_in: inner.today_tokens_in,
            total_tokens_out: inner.today_tokens_out,
            total_errors: inner.today_errors,
            total_429s: inner.today_429s,
            error_rate_pct: (error_rate * 10.0).round() / 10.0,
            avg_latency_ms: avg_latency,
            estimated_cost_usd: ((est_cost_input + est_cost_output) * 10000.0).round() / 10000.0,
            breaker_state,
            consecutive_errors: inner.consecutive_errors,
        }
    }

    /// Append a record to the JSONL file.
    fn append_record(&self, record: &RequestRecord) -> std::io::Result<()> {
        use std::io::Write;

        std::fs::create_dir_all(&self.metrics_dir)?;
        let path = self.metrics_dir.join("ollama-usage.jsonl");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let line = serde_json::to_string(record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writeln!(file, "{}", line)?;
        Ok(())
    }
}

// ── Usage Summary ────────────────────────────────────────────────────────────

/// Summary of today's Ollama usage.
#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub date: String,
    pub total_requests: u64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub total_errors: u64,
    pub total_429s: u64,
    pub error_rate_pct: f64,
    pub avg_latency_ms: u64,
    pub estimated_cost_usd: f64,
    pub breaker_state: String,
    pub consecutive_errors: u32,
}

impl std::fmt::Display for UsageSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# Ollama Usage — {}", self.date)?;
        writeln!(f)?;
        writeln!(f, "## Requests")?;
        writeln!(f, "- Total: {}", self.total_requests)?;
        writeln!(f, "- Errors: {} ({:.1}%)", self.total_errors, self.error_rate_pct)?;
        writeln!(f, "- 429 (rate limited): {}", self.total_429s)?;
        writeln!(f, "- Avg latency: {}ms", self.avg_latency_ms)?;
        writeln!(f)?;
        writeln!(f, "## Tokens")?;
        writeln!(f, "- Input: {}", self.total_tokens_in)?;
        writeln!(f, "- Output: {}", self.total_tokens_out)?;
        writeln!(f, "- Total: {}", self.total_tokens_in + self.total_tokens_out)?;
        writeln!(f, "- Est. cost: ${:.4}", self.estimated_cost_usd)?;
        writeln!(f)?;
        writeln!(f, "## Circuit Breaker")?;
        writeln!(f, "- State: {}", self.breaker_state)?;
        writeln!(f, "- Consecutive errors: {}", self.consecutive_errors)?;
        writeln!(f, "- Threshold: {} errors → {}s cooldown", BREAKER_THRESHOLD, COOLDOWN_SECS)?;
        Ok(())
    }
}

// ── Tool Schema + Handler (for ThinkLoop integration) ────────────────────────

/// Returns the tool schema for `ollama_usage`.
pub fn ollama_usage_tool_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "ollama_usage",
        "description": "Check Ollama Cloud API usage: total requests today, tokens consumed, estimated cost, error rate, and circuit breaker state. Use this to monitor your own resource consumption.",
        "parameters": {
            "type": "object",
            "properties": {},
            "required": []
        }
    })
}

/// Handle the `ollama_usage` tool call. Returns formatted usage summary.
pub async fn handle_ollama_usage(limiter: Option<&RateLimiter>) -> String {
    match limiter {
        Some(lim) => {
            let summary = lim.usage_summary().await;
            summary.to_string()
        }
        None => "Rate limiter not configured — usage tracking unavailable.".to_string(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_limiter(dir: &Path) -> RateLimiter {
        RateLimiter::new(dir.to_path_buf())
    }

    #[tokio::test]
    async fn breaker_stays_closed_on_success() {
        let dir = tempfile::tempdir().unwrap();
        let lim = test_limiter(dir.path());

        for _ in 0..10 {
            lim.record("test-model", 100, 50, 200, 500).await;
        }

        assert!(lim.check().await.is_ok());
        let summary = lim.usage_summary().await;
        assert_eq!(summary.total_requests, 10);
        assert_eq!(summary.total_errors, 0);
        assert_eq!(summary.breaker_state, "closed");
    }

    #[tokio::test]
    async fn breaker_trips_on_consecutive_errors() {
        let dir = tempfile::tempdir().unwrap();
        let lim = test_limiter(dir.path());

        // 5 consecutive 429s should trip the breaker
        for _ in 0..5 {
            lim.record("test-model", 100, 0, 429, 200).await;
        }

        let result = lim.check().await;
        assert!(result.is_err());
        let summary = lim.usage_summary().await;
        assert!(summary.breaker_state.starts_with("OPEN"));
        assert_eq!(summary.total_429s, 5);
    }

    #[tokio::test]
    async fn success_resets_consecutive_errors() {
        let dir = tempfile::tempdir().unwrap();
        let lim = test_limiter(dir.path());

        // 4 errors, then a success
        for _ in 0..4 {
            lim.record("test-model", 100, 0, 500, 200).await;
        }
        lim.record("test-model", 100, 50, 200, 500).await;

        // Should still be closed (success reset the counter)
        assert!(lim.check().await.is_ok());
        let summary = lim.usage_summary().await;
        assert_eq!(summary.consecutive_errors, 0);
        assert_eq!(summary.breaker_state, "closed");
    }

    #[tokio::test]
    async fn mixed_error_codes_count() {
        let dir = tempfile::tempdir().unwrap();
        let lim = test_limiter(dir.path());

        // Mix of retryable errors
        lim.record("m", 100, 0, 429, 200).await;
        lim.record("m", 100, 0, 500, 200).await;
        lim.record("m", 100, 0, 502, 200).await;
        lim.record("m", 100, 0, 503, 200).await;
        lim.record("m", 100, 0, 504, 200).await;

        let result = lim.check().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn token_tracking() {
        let dir = tempfile::tempdir().unwrap();
        let lim = test_limiter(dir.path());

        lim.record("m", 1000, 500, 200, 100).await;
        lim.record("m", 2000, 800, 200, 200).await;

        let summary = lim.usage_summary().await;
        assert_eq!(summary.total_tokens_in, 3000);
        assert_eq!(summary.total_tokens_out, 1300);
        assert_eq!(summary.avg_latency_ms, 150);
    }

    #[tokio::test]
    async fn jsonl_written() {
        let dir = tempfile::tempdir().unwrap();
        let lim = test_limiter(dir.path());

        lim.record("test-model", 100, 50, 200, 500).await;

        let path = dir.path().join("ollama-usage.jsonl");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("test-model"));
        assert!(content.contains("\"status_code\":200"));
    }

    #[tokio::test]
    async fn usage_summary_display() {
        let dir = tempfile::tempdir().unwrap();
        let lim = test_limiter(dir.path());

        lim.record("m", 500000, 200000, 200, 1000).await;

        let summary = lim.usage_summary().await;
        let text = summary.to_string();
        assert!(text.contains("Ollama Usage"));
        assert!(text.contains("Circuit Breaker"));
        assert!(text.contains("closed"));
    }

    #[tokio::test]
    async fn error_rate_calculation() {
        let dir = tempfile::tempdir().unwrap();
        let lim = test_limiter(dir.path());

        // 8 successes, 2 errors = 20% error rate
        for _ in 0..8 {
            lim.record("m", 100, 50, 200, 100).await;
        }
        lim.record("m", 100, 0, 429, 100).await;
        lim.record("m", 100, 0, 500, 100).await;

        let summary = lim.usage_summary().await;
        assert_eq!(summary.total_requests, 10);
        assert_eq!(summary.total_errors, 2);
        assert!((summary.error_rate_pct - 20.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn estimated_cost() {
        let dir = tempfile::tempdir().unwrap();
        let lim = test_limiter(dir.path());

        // 1M input + 1M output = $0.30 + $1.20 = $1.50
        lim.record("m", 1_000_000, 1_000_000, 200, 100).await;

        let summary = lim.usage_summary().await;
        assert!((summary.estimated_cost_usd - 1.50).abs() < 0.01);
    }
}
