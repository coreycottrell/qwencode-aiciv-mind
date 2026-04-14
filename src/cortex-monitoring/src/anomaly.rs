//! Anomaly detection for Cortex metrics.
//!
//! Detects when metrics deviate from expected ranges and triggers alerts.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Anomaly alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub mind_id: String,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Anomaly detector with configurable thresholds.
pub struct AnomalyDetector {
    mind_id: String,
    alerts_dir: std::path::PathBuf,
    thresholds: HashMap<String, ThresholdConfig>,
    alerts: Vec<Alert>,
}

#[derive(Debug, Clone)]
pub struct ThresholdConfig {
    pub warning: f64,
    pub critical: f64,
}

impl AnomalyDetector {
    pub fn new(mind_id: &str, alerts_dir: &Path) -> Self {
        let mut detector = Self {
            mind_id: mind_id.to_string(),
            alerts_dir: alerts_dir.to_path_buf(),
            thresholds: HashMap::new(),
            alerts: Vec::new(),
        };

        // Default thresholds
        detector.set_threshold("thinkloop_iterations", ThresholdConfig { warning: 10.0, critical: 15.0 });
        detector.set_threshold("thinkloop_duration_ms", ThresholdConfig { warning: 30000.0, critical: 60000.0 });
        detector.set_threshold("thinkloop_avg_iteration_time_ms", ThresholdConfig { warning: 10000.0, critical: 20000.0 });
        detector.set_threshold("thinkloop_challenger_warnings", ThresholdConfig { warning: 3.0, critical: 5.0 });
        detector.set_threshold("tool_latency_ms", ThresholdConfig { warning: 5000.0, critical: 15000.0 });
        detector.set_threshold("challenger_warning", ThresholdConfig { warning: 5.0, critical: 10.0 });

        detector
    }

    pub fn set_threshold(&mut self, name: &str, config: ThresholdConfig) {
        self.thresholds.insert(name.to_string(), config);
    }

    /// Check a metric value against thresholds. Returns alerts if breached.
    pub fn check(&mut self, metric: &str, value: f64) -> Vec<Alert> {
        let mut new_alerts = Vec::new();

        if let Some(threshold) = self.thresholds.get(metric) {
            if value >= threshold.critical {
                let alert = Alert {
                    timestamp: chrono::Utc::now(),
                    mind_id: self.mind_id.clone(),
                    metric: metric.to_string(),
                    value,
                    threshold: threshold.critical,
                    severity: AlertSeverity::Critical,
                    message: format!("CRITICAL: {} = {:.1} (threshold: {:.1})", metric, value, threshold.critical),
                };
                new_alerts.push(alert);
            } else if value >= threshold.warning {
                let alert = Alert {
                    timestamp: chrono::Utc::now(),
                    mind_id: self.mind_id.clone(),
                    metric: metric.to_string(),
                    value,
                    threshold: threshold.warning,
                    severity: AlertSeverity::Warning,
                    message: format!("WARNING: {} = {:.1} (threshold: {:.1})", metric, value, threshold.warning),
                };
                new_alerts.push(alert);
            }
        }

        for alert in &new_alerts {
            self.alerts.push(alert.clone());
        }

        new_alerts
    }

    /// Get recent alerts.
    pub fn recent_alerts(&self, count: usize) -> &[Alert] {
        let start = self.alerts.len().saturating_sub(count);
        &self.alerts[start..]
    }

    /// Get alert summary by severity.
    pub fn summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for alert in &self.alerts {
            let key = format!("{:?}", alert.severity);
            *summary.entry(key).or_insert(0) += 1;
        }
        summary
    }
}
