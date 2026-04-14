//! Fitness tracking — evidence-based scoring + downstream citation.
//!
//! Phase 1: evidence-based score only.
//! Phase 2: combined evidence + citation score.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessEntry {
    pub timestamp: String,
    pub score: f64,
    pub evidence_score: f64,
    pub citation_score: f64,
    pub task_summary: String,
    pub completion: f64,
    pub no_errors: f64,
    pub specificity: f64,
    pub memory_written: f64,
    pub citation_count: i64,
}

pub struct FitnessTracker {
    file: PathBuf,
}

impl FitnessTracker {
    pub fn new(root_dir: &Path, mind_id: &str) -> Self {
        let dir = root_dir.join("fitness");
        std::fs::create_dir_all(&dir).ok();
        Self {
            file: dir.join(format!("{mind_id}.jsonl")),
        }
    }

    /// Record a fitness entry for a completed task.
    ///
    /// Phase 1: score = evidence_score (citation_score is 0 until citations accumulate)
    pub fn record(
        &self,
        task_summary: &str,
        completion: bool,
        had_errors: bool,
        result_text: &str,
        wrote_memory: bool,
    ) {
        // Completion component
        let completion_val = if completion { 1.0 } else { 0.0 };

        // No-errors component
        let no_errors_val = if had_errors { 0.0 } else { 1.0 };

        // Specificity: concrete details in result
        let specificity_val = compute_specificity(result_text);

        // Memory written component
        let memory_val = if wrote_memory { 1.0 } else { 0.0 };

        // Evidence score: weighted combination
        let evidence_score =
            0.35 * completion_val
            + 0.20 * no_errors_val
            + 0.25 * specificity_val
            + 0.20 * memory_val;

        // Citation score: 0 in Phase 1 (no citation data yet)
        let citation_score = 0.0;

        // Combined: 70% evidence, 30% citation
        let score = 0.7 * evidence_score + 0.3 * citation_score;

        let entry = FitnessEntry {
            timestamp: Utc::now().to_rfc3339(),
            score,
            evidence_score,
            citation_score,
            task_summary: task_summary.to_string(),
            completion: completion_val,
            no_errors: no_errors_val,
            specificity: specificity_val,
            memory_written: memory_val,
            citation_count: 0,
        };

        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)
            .unwrap();
        if let Ok(json) = serde_json::to_string(&entry) {
            writeln!(f, "{json}").ok();
        }
    }

    pub fn history(&self) -> Vec<FitnessEntry> {
        if !self.file.exists() {
            return Vec::new();
        }
        let text = std::fs::read_to_string(&self.file).unwrap_or_default();
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }

    pub fn average(&self) -> f64 {
        let history = self.history();
        if history.is_empty() {
            return 0.0;
        }
        history.iter().map(|e| e.score).sum::<f64>() / history.len() as f64
    }
}

/// Compute specificity score from result text.
/// Looks for concrete details: numbers, file paths, code blocks, named entities.
fn compute_specificity(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    let mut score: f64 = 0.0;

    // Length: longer results tend to be more specific (up to a point)
    let len = text.len();
    if len > 500 {
        score += 0.3;
    } else if len > 100 {
        score += 0.2;
    } else if len > 20 {
        score += 0.1;
    }

    // Numbers in text (suggests concrete findings)
    if text.chars().any(|c| c.is_ascii_digit()) {
        score += 0.2;
    }

    // File paths (suggests specific locations)
    if text.contains('/') || text.contains('\\') {
        score += 0.2;
    }

    // Code blocks (suggests concrete examples)
    if text.contains("```") {
        score += 0.2;
    }

    // Bullet points or numbered lists (suggests structured findings)
    if text.contains("\n- ") || text.contains("\n* ") {
        score += 0.1;
    }

    score.min(1.0_f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn record_and_average() {
        let dir = TempDir::new().unwrap();
        let tracker = FitnessTracker::new(dir.path(), "test-mind");

        tracker.record("test task", true, false, "Found 3 files with the issue:\n- src/foo.rs\n- src/bar.rs\n```rust\nfn test() {}\n```", true);
        tracker.record("another task", true, false, "Looked at the code.", true);
        tracker.record("failed task", false, true, "", false);

        let history = tracker.history();
        assert_eq!(history.len(), 3);

        let avg = tracker.average();
        assert!(avg > 0.0 && avg < 1.0);

        // First entry should have higher specificity than third (empty result)
        assert!(history[0].specificity > history[2].specificity);
        // Second entry is short text, still better than empty
        assert!(history[1].specificity >= history[2].specificity);
    }

    #[test]
    fn empty_history() {
        let dir = TempDir::new().unwrap();
        let tracker = FitnessTracker::new(dir.path(), "test-mind");
        assert_eq!(tracker.average(), 0.0);
        assert!(tracker.history().is_empty());
    }
}
