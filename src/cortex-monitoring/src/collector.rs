//! Metrics collector — thread-safe ring buffer + JSONL persistence.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::metrics::*;

/// Thread-safe metrics collector for a single mind.
pub struct MetricsCollector {
    mind_id: String,
    metrics_dir: PathBuf,
    /// Ring buffer for real-time access (last 1000 points).
    buffer: Arc<Mutex<VecDeque<MetricPoint>>>,
    /// Max buffer size.
    max_buffer: usize,
}

impl MetricsCollector {
    pub fn new(mind_id: &str, metrics_dir: &Path) -> Self {
        Self {
            mind_id: mind_id.to_string(),
            metrics_dir: metrics_dir.to_path_buf(),
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            max_buffer: 1000,
        }
    }

    /// Record a metric point.
    pub async fn record(&self, point: MetricPoint) {
        let mut buf = self.buffer.lock().await;
        if buf.len() >= self.max_buffer {
            buf.pop_front();
        }
        buf.push_back(point);
    }

    /// Record a ThinkLoop completion.
    pub async fn record_thinkloop(&self, m: &ThinkLoopMetrics) {
        self.record(MetricPoint::new(&self.mind_id, "thinkloop_iterations", m.iterations as f64)
            .with_label("model", &m.model)).await;
        self.record(MetricPoint::new(&self.mind_id, "thinkloop_tool_calls", m.tool_calls as f64)).await;
        self.record(MetricPoint::new(&self.mind_id, "thinkloop_duration_ms", m.duration_ms as f64)).await;
        self.record(MetricPoint::new(&self.mind_id, "thinkloop_completed", if m.completed { 1.0 } else { 0.0 })).await;
        self.record(MetricPoint::new(&self.mind_id, "thinkloop_stall_killed", if m.stall_killed { 1.0 } else { 0.0 })).await;
        self.record(MetricPoint::new(&self.mind_id, "thinkloop_challenger_warnings", m.challenger_warnings as f64)).await;
        self.record(MetricPoint::new(&self.mind_id, "thinkloop_avg_iteration_time_ms", m.avg_iteration_time())).await;

        // Persist to JSONL
        self.persist_thinkloop(m);
    }

    /// Record a tool call result.
    pub async fn record_tool(&self, tool_name: &str, success: bool, latency_ms: u64) {
        self.record(MetricPoint::new(&self.mind_id, "tool_call", if success { 1.0 } else { 0.0 })
            .with_label("tool", tool_name)
            .with_label("success", &success.to_string())).await;
        self.record(MetricPoint::new(&self.mind_id, "tool_latency_ms", latency_ms as f64)
            .with_label("tool", tool_name)).await;
    }

    /// Record a delegation event.
    pub async fn record_delegation(&self, event: &str, duration_ms: u64, success: bool) {
        self.record(MetricPoint::new(&self.mind_id, "delegation_event", if success { 1.0 } else { 0.0 })
            .with_label("event", event)).await;
        self.record(MetricPoint::new(&self.mind_id, "delegation_latency_ms", duration_ms as f64)
            .with_label("event", event)).await;
    }

    /// Record a Challenger warning.
    pub async fn record_challenger(&self, check: &str, severity: &str) {
        self.record(MetricPoint::new(&self.mind_id, "challenger_warning", 1.0)
            .with_label("check", check)
            .with_label("severity", severity)).await;
    }

    /// Get recent metrics from the ring buffer.
    pub async fn recent(&self, count: usize) -> Vec<MetricPoint> {
        let buf = self.buffer.lock().await;
        buf.iter().rev().take(count).cloned().collect()
    }

    /// Get metrics by name from the ring buffer.
    pub async fn by_name(&self, name: &str) -> Vec<MetricPoint> {
        let buf = self.buffer.lock().await;
        buf.iter().filter(|m| m.name == name).cloned().collect()
    }

    /// Persist ThinkLoop metrics to JSONL (synchronous — no async drop issues).
    fn persist_thinkloop(&self, m: &ThinkLoopMetrics) {
        let file_path = self.metrics_dir.join("thinkloop.jsonl");

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("[monitoring] Failed to create metrics dir: {e}");
            }
        }

        let entry = serde_json::json!({
            "timestamp": chrono::Utc::now(),
            "mind_id": self.mind_id,
            "iterations": m.iterations,
            "tool_calls": m.tool_calls,
            "duration_ms": m.duration_ms,
            "completed": m.completed,
            "stall_killed": m.stall_killed,
            "challenger_warnings": m.challenger_warnings,
            "model": m.model,
            "avg_iteration_time_ms": m.avg_iteration_time(),
        });

        let line = serde_json::to_string(&entry).unwrap_or_default();
        use std::io::Write;
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
        {
            Ok(mut file) => {
                if let Err(e) = writeln!(file, "{line}") {
                    eprintln!("[monitoring] Failed to write metrics: {e}");
                }
            }
            Err(e) => {
                eprintln!("[monitoring] Failed to open metrics file {}: {e}", file_path.display());
            }
        }
    }
}
