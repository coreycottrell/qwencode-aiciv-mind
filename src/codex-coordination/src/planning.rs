//! # PlanningGate — Go Slow to Go Fast (Principle 3)
//!
//! Every task passes through a planning gate. Gate depth scales with complexity.
//! Trivial tasks get a memory check. Complex tasks spawn planning sub-minds.

use serde::{Deserialize, Serialize};

/// Task complexity levels — determines planning gate depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskComplexity {
    /// Memory check only. < 1s. "Have I done this before?"
    Trivial,
    /// Memory check + brief plan. 2-5s. "3 steps, low risk."
    Simple,
    /// Memory check + competing hypotheses. 10-30s. "Two approaches, testing A."
    Medium,
    /// Spawn a planning sub-mind. 30s-5m. "Needs its own context."
    Complex,
    /// Spawn multiple competing planners. 1-10m. "Never seen this before."
    Novel,
}

/// What the planning gate decided.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanningDecision {
    /// Execute immediately. Memory check passed.
    Execute { memory_hit: Option<String> },
    /// Execute with this plan.
    ExecuteWithPlan { plan: Vec<String> },
    /// Need to spawn a planner sub-mind.
    SpawnPlanner { reason: String },
    /// Need to spawn competing planners.
    SpawnCompetingPlanners { approaches: Vec<String> },
}

/// The planning gate that intercepts tasks before execution.
pub struct PlanningGate {
    /// Threshold above which we spawn sub-minds for planning.
    spawn_threshold: TaskComplexity,
}

impl PlanningGate {
    pub fn new() -> Self {
        Self {
            spawn_threshold: TaskComplexity::Complex,
        }
    }

    /// Evaluate a task's complexity and return a planning decision.
    ///
    /// In production, this makes a lightweight LLM call (M2.7) to classify
    /// the task and check memory for prior solutions.
    pub fn evaluate(
        &self,
        description: &str,
        memory_results: &[String],
    ) -> PlanningDecision {
        let complexity = self.classify_complexity(description);

        match complexity {
            TaskComplexity::Trivial => {
                let hit = memory_results.first().cloned();
                PlanningDecision::Execute { memory_hit: hit }
            }
            TaskComplexity::Simple => {
                PlanningDecision::ExecuteWithPlan {
                    plan: vec![format!("Execute: {}", description)],
                }
            }
            TaskComplexity::Medium => {
                PlanningDecision::ExecuteWithPlan {
                    plan: vec![
                        format!("Approach A: direct implementation of '{}'", description),
                        "Verify with red team after completion".into(),
                    ],
                }
            }
            TaskComplexity::Complex => {
                PlanningDecision::SpawnPlanner {
                    reason: format!("Task '{}' exceeds complexity threshold", description),
                }
            }
            TaskComplexity::Novel => {
                PlanningDecision::SpawnCompetingPlanners {
                    approaches: vec![
                        "Conservative: minimal changes".into(),
                        "Aggressive: full redesign".into(),
                        "Hybrid: targeted refactor".into(),
                    ],
                }
            }
        }
    }

    /// Classify task complexity. In production, this is an M2.7 LLM call.
    /// For now, use heuristics based on description length and keywords.
    fn classify_complexity(&self, description: &str) -> TaskComplexity {
        let words: Vec<&str> = description.split_whitespace().collect();
        let len = words.len();

        // Keyword-based classification (placeholder for M2.7 classification)
        let complex_keywords = ["redesign", "architect", "migrate", "rewrite", "integrate"];
        let has_complex_keyword = words.iter()
            .any(|w| complex_keywords.contains(&w.to_lowercase().as_str()));

        if has_complex_keyword {
            return TaskComplexity::Complex;
        }

        match len {
            0..=5 => TaskComplexity::Trivial,
            6..=15 => TaskComplexity::Simple,
            16..=40 => TaskComplexity::Medium,
            41..=80 => TaskComplexity::Complex,
            _ => TaskComplexity::Novel,
        }
    }
}

impl Default for PlanningGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial_task_executes_immediately() {
        let gate = PlanningGate::new();
        let decision = gate.evaluate("fix typo", &[]);
        assert!(matches!(decision, PlanningDecision::Execute { .. }));
    }

    #[test]
    fn trivial_with_memory_hit() {
        let gate = PlanningGate::new();
        let decision = gate.evaluate("fix typo", &["Did this before: change line 5".into()]);
        match decision {
            PlanningDecision::Execute { memory_hit } => {
                assert!(memory_hit.is_some());
            }
            _ => panic!("Expected Execute"),
        }
    }

    #[test]
    fn complex_task_spawns_planner() {
        let gate = PlanningGate::new();
        let decision = gate.evaluate(
            "redesign the entire authentication system to use Ed25519 keypairs with JWKS rotation",
            &[],
        );
        assert!(matches!(decision, PlanningDecision::SpawnPlanner { .. }));
    }

    #[test]
    fn medium_task_gets_plan() {
        let gate = PlanningGate::new();
        let decision = gate.evaluate(
            "add depth scoring to the memory consolidation pipeline with citation tracking",
            &[],
        );
        assert!(matches!(decision, PlanningDecision::ExecuteWithPlan { .. }));
    }
}
