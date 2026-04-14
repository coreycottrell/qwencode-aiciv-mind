//! Metric types for Cortex monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single metric data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// When this metric was recorded.
    pub timestamp: DateTime<Utc>,
    /// Which mind recorded this metric.
    pub mind_id: String,
    /// Metric name (e.g., "thinkloop_iterations").
    pub name: String,
    /// Metric value.
    pub value: f64,
    /// Optional labels for filtering.
    pub labels: Vec<(String, String)>,
}

impl MetricPoint {
    pub fn new(mind_id: &str, name: &str, value: f64) -> Self {
        Self {
            timestamp: Utc::now(),
            mind_id: mind_id.to_string(),
            name: name.to_string(),
            value,
            labels: Vec::new(),
        }
    }

    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.push((key.to_string(), value.to_string()));
        self
    }
}

/// ThinkLoop performance metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThinkLoopMetrics {
    /// Number of iterations in this turn.
    pub iterations: u32,
    /// Number of tool calls made.
    pub tool_calls: u32,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the task completed naturally.
    pub completed: bool,
    /// Whether it was killed for stalling.
    pub stall_killed: bool,
    /// Number of Challenger warnings.
    pub challenger_warnings: u32,
    /// Model used for this turn.
    pub model: String,
}

impl ThinkLoopMetrics {
    pub fn avg_iteration_time(&self) -> f64 {
        if self.iterations == 0 { return 0.0; }
        self.duration_ms as f64 / self.iterations as f64
    }

    pub fn tool_call_rate(&self) -> f64 {
        if self.duration_ms == 0 { return 0.0; }
        self.tool_calls as f64 / (self.duration_ms as f64 / 1000.0)
    }
}

/// Per-tool performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    pub tool_name: String,
    pub call_count: u32,
    pub success_count: u32,
    pub error_count: u32,
    pub total_latency_ms: u64,
}

impl ToolMetrics {
    pub fn new(tool_name: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            call_count: 0,
            success_count: 0,
            error_count: 0,
            total_latency_ms: 0,
        }
    }

    pub fn record_success(&mut self, latency_ms: u64) {
        self.call_count += 1;
        self.success_count += 1;
        self.total_latency_ms += latency_ms;
    }

    pub fn record_error(&mut self, latency_ms: u64) {
        self.call_count += 1;
        self.error_count += 1;
        self.total_latency_ms += latency_ms;
    }

    pub fn avg_latency_ms(&self) -> f64 {
        if self.call_count == 0 { return 0.0; }
        self.total_latency_ms as f64 / self.call_count as f64
    }

    pub fn success_rate(&self) -> f64 {
        if self.call_count == 0 { return 0.0; }
        self.success_count as f64 / self.call_count as f64
    }
}

/// Delegation performance metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DelegationMetrics {
    /// Number of agents spawned.
    pub agents_spawned: u32,
    /// Number of tasks delegated.
    pub tasks_delegated: u32,
    /// Number of tasks completed successfully.
    pub tasks_completed: u32,
    /// Average delegation duration (ms).
    pub avg_duration_ms: f64,
}

/// Memory usage metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Number of memory searches.
    pub searches: u32,
    /// Number of memory writes.
    pub writes: u32,
    /// Number of cache hits (found in working memory).
    pub cache_hits: u32,
    /// Average depth score of accessed memories.
    pub avg_depth_score: f64,
}

/// Challenger system metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChallengerMetrics {
    /// Total warnings fired.
    pub total_warnings: u32,
    /// Warnings by severity.
    pub by_severity: std::collections::HashMap<String, u32>,
    /// Warnings by check type.
    pub by_check: std::collections::HashMap<String, u32>,
}

/// Model usage metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Which model was used.
    pub model_name: String,
    /// Number of API calls.
    pub api_calls: u32,
    /// Total latency across all calls (ms).
    pub total_latency_ms: u64,
    /// Total tokens used (approximate).
    pub total_tokens: u64,
    /// Number of errors.
    pub errors: u32,
}

/// Aggregate snapshot of all metrics for a mind.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MindMetricsSnapshot {
    pub mind_id: String,
    pub timestamp: DateTime<Utc>,
    pub thinkloop: ThinkLoopMetrics,
    pub tools: std::collections::HashMap<String, ToolMetrics>,
    pub delegation: DelegationMetrics,
    pub memory: MemoryMetrics,
    pub challenger: ChallengerMetrics,
    pub models: std::collections::HashMap<String, ModelMetrics>,
}
