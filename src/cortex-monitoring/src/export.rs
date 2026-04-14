//! Metrics exporter — reads persisted JSONL and produces summaries.

use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkLoopEntry {
    pub timestamp: DateTime<Utc>,
    pub mind_id: String,
    pub iterations: u32,
    pub tool_calls: u32,
    pub duration_ms: u64,
    pub completed: bool,
    pub stall_killed: bool,
    pub challenger_warnings: u32,
    pub model: String,
    pub avg_iteration_time_ms: f64,
}

/// Exports metrics summaries from persisted JSONL files.
pub struct MetricsExporter {
    metrics_dir: std::path::PathBuf,
}

impl MetricsExporter {
    pub fn new(metrics_dir: &Path) -> Self {
        Self {
            metrics_dir: metrics_dir.to_path_buf(),
        }
    }

    /// Read all ThinkLoop entries from JSONL.
    pub async fn read_thinkloop(&self) -> Vec<ThinkLoopEntry> {
        let file_path = self.metrics_dir.join("thinkloop.jsonl");
        let mut entries = Vec::new();

        if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
            for line in content.lines() {
                if let Ok(entry) = serde_json::from_str::<ThinkLoopEntry>(line) {
                    entries.push(entry);
                }
            }
        }

        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        entries
    }

    /// Generate a summary report from all entries.
    pub async fn summary_report(&self) -> String {
        let entries = self.read_thinkloop().await;
        if entries.is_empty() {
            return "No metrics recorded yet.".to_string();
        }

        let total_turns = entries.len();
        let total_iters: u32 = entries.iter().map(|e| e.iterations).sum();
        let total_tools: u32 = entries.iter().map(|e| e.tool_calls).sum();
        let completed: usize = entries.iter().filter(|e| e.completed).count();
        let stall_killed: usize = entries.iter().filter(|e| e.stall_killed).count();
        let total_warnings: u32 = entries.iter().map(|e| e.challenger_warnings).sum();
        let total_ms: u64 = entries.iter().map(|e| e.duration_ms).sum();
        let avg_ms = total_ms as f64 / total_turns as f64;
        let avg_iters = total_iters as f64 / total_turns as f64;
        let avg_tools = total_tools as f64 / total_turns as f64;

        // Model breakdown
        let mut model_counts = std::collections::HashMap::new();
        for e in &entries {
            *model_counts.entry(&e.model).or_insert(0) += 1;
        }

        let mut report = format!(
            "╔══════════════════════════════════════════════════╗\n\
             ║         CORTEX METRICS SUMMARY                   ║\n\
             ╠══════════════════════════════════════════════════╣\n\
             ║ Total turns:        {:>5}                        ║\n\
             ║ Total iterations:   {:>5} (avg {:.1}/turn)        ║\n\
             ║ Total tool calls:   {:>5} (avg {:.1}/turn)        ║\n\
             ║ Completed:          {:>5} ({:.0}%)               ║\n\
             ║ Stall killed:       {:>5} ({:.0}%)               ║\n\
             ║ Challenger warns:   {:>5}                        ║\n\
             ║ Avg turn duration:  {:>5.0}ms                    ║\n\
             ║                                                  ║",
            total_turns, total_iters, avg_iters, total_tools, avg_tools,
            completed, completed as f64 / total_turns as f64 * 100.0,
            stall_killed, stall_killed as f64 / total_turns as f64 * 100.0,
            total_warnings, avg_ms,
        );

        if !model_counts.is_empty() {
            report.push_str("\n╠══════════════════════════════════════════════════╣\n");
            report.push_str("║ Model usage:                                     ║\n");
            for (model, count) in &model_counts {
                report.push_str(&format!("║   {:<30} {:>5} turns              ║\n", model, count));
            }
        }

        report.push_str("╚══════════════════════════════════════════════════╝\n");
        report
    }
}
