//! # Spawn Triggers — Dynamic Agent Spawning (Principle 4)
//!
//! Triggers that fire automatically when patterns demand more minds.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::types::MindId;

/// Events that can trigger spawning new minds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpawnTrigger {
    /// Same problem type seen 3+ times → spawn specialist.
    PatternRepetition {
        pattern_id: String,
        occurrences: usize,
        threshold: usize,
    },
    /// Task complexity exceeds planning gate threshold.
    TaskComplexityExceeded {
        task_description: String,
    },
    /// Multiple valid approaches exist → spawn parallel thinkers.
    CompetingHypotheses {
        hypotheses: Vec<String>,
    },
    /// Mind stuck for too long → spawn fresh context.
    BlockingDetected {
        mind_id: MindId,
        stuck_duration: u64, // seconds
        retry_count: usize,
    },
    /// Task crosses domain boundaries → route to or spawn domain mind.
    DomainBoundary {
        from_vertical: String,
        to_vertical: String,
    },
    /// Completion claimed, needs adversarial verification.
    VerificationNeed {
        task_id: String,
    },
    /// Context window approaching capacity → spawn overflow mind.
    ContextPressure {
        mind_id: MindId,
        usage_percent: f64,
    },
    /// Time-based or event-based scheduled trigger.
    Scheduled {
        trigger_name: String,
    },
}

/// Manages spawn trigger evaluation.
pub struct TriggerEngine {
    /// Pattern repetition counts.
    pattern_counts: std::collections::HashMap<String, usize>,
    /// Blocking detection: mind_id → last activity timestamp.
    last_activity: std::collections::HashMap<MindId, DateTime<Utc>>,
    /// Blocking threshold in seconds.
    blocking_threshold_secs: i64,
    /// Pattern threshold (how many occurrences before triggering).
    pattern_threshold: usize,
    /// Context pressure threshold (percent).
    context_pressure_threshold: f64,
}

impl TriggerEngine {
    pub fn new() -> Self {
        Self {
            pattern_counts: std::collections::HashMap::new(),
            last_activity: std::collections::HashMap::new(),
            blocking_threshold_secs: 120, // 2 minutes
            pattern_threshold: 3,
            context_pressure_threshold: 85.0,
        }
    }

    /// Record a pattern occurrence. Returns a trigger if threshold exceeded.
    pub fn observe_pattern(&mut self, pattern_id: &str) -> Option<SpawnTrigger> {
        let count = self.pattern_counts
            .entry(pattern_id.to_string())
            .or_insert(0);
        *count += 1;

        if *count >= self.pattern_threshold {
            Some(SpawnTrigger::PatternRepetition {
                pattern_id: pattern_id.to_string(),
                occurrences: *count,
                threshold: self.pattern_threshold,
            })
        } else {
            None
        }
    }

    /// Record activity for a mind. Returns blocking trigger if stuck.
    pub fn observe_activity(&mut self, mind_id: &MindId) {
        self.last_activity.insert(mind_id.clone(), Utc::now());
    }

    /// Check all minds for blocking. Returns triggers for stuck minds.
    pub fn check_blocking(&self) -> Vec<SpawnTrigger> {
        let now = Utc::now();
        let threshold = Duration::seconds(self.blocking_threshold_secs);

        self.last_activity.iter()
            .filter_map(|(mind_id, last)| {
                if now - *last > threshold {
                    Some(SpawnTrigger::BlockingDetected {
                        mind_id: mind_id.clone(),
                        stuck_duration: (now - *last).num_seconds() as u64,
                        retry_count: 0,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check context pressure for a mind.
    pub fn check_context_pressure(
        &self,
        mind_id: &MindId,
        usage_percent: f64,
    ) -> Option<SpawnTrigger> {
        if usage_percent >= self.context_pressure_threshold {
            Some(SpawnTrigger::ContextPressure {
                mind_id: mind_id.clone(),
                usage_percent,
            })
        } else {
            None
        }
    }
}

impl Default for TriggerEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages the 3-hour scratchpad rotation cycle (Addendum A5).
pub struct RotationTrigger {
    interval: Duration,
    last_rotation: DateTime<Utc>,
}

impl RotationTrigger {
    pub fn new(interval_hours: i64) -> Self {
        Self {
            interval: Duration::hours(interval_hours),
            last_rotation: Utc::now(),
        }
    }

    /// Check if rotation is due. Returns true if scratchpads should rotate.
    pub fn is_due(&self) -> bool {
        Utc::now() - self.last_rotation >= self.interval
    }

    /// Mark rotation as completed.
    pub fn mark_rotated(&mut self) {
        self.last_rotation = Utc::now();
    }

    /// Time until next rotation.
    pub fn time_until_next(&self) -> Duration {
        let elapsed = Utc::now() - self.last_rotation;
        if elapsed >= self.interval {
            Duration::zero()
        } else {
            self.interval - elapsed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_triggers_after_threshold() {
        let mut engine = TriggerEngine::new();
        assert!(engine.observe_pattern("jwt_debug").is_none());
        assert!(engine.observe_pattern("jwt_debug").is_none());
        let trigger = engine.observe_pattern("jwt_debug");
        assert!(trigger.is_some());
        match trigger.unwrap() {
            SpawnTrigger::PatternRepetition { occurrences, .. } => {
                assert_eq!(occurrences, 3);
            }
            _ => panic!("Expected PatternRepetition"),
        }
    }

    #[test]
    fn context_pressure_triggers() {
        let engine = TriggerEngine::new();
        let mind_id = MindId("research-lead".into());
        assert!(engine.check_context_pressure(&mind_id, 50.0).is_none());
        assert!(engine.check_context_pressure(&mind_id, 90.0).is_some());
    }

    #[test]
    fn rotation_trigger() {
        let trigger = RotationTrigger::new(3);
        assert!(!trigger.is_due()); // Just created, not due yet
    }
}
