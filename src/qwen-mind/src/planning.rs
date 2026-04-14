//! Planning gate — proportional complexity assessment.
//! Principle 3 — Go Slow To Go Fast.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskComplexity {
    /// Already solved — replay prior result
    Trivial,
    /// 3 steps or fewer, no dependencies
    Simple,
    /// Multiple approaches, needs hypothesis evaluation
    Medium,
    /// Needs dedicated planning sub-mind
    Complex,
    /// Novel — spawn competing planners
    Novel,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub complexity: TaskComplexity,
    pub steps: Vec<String>,
    pub prior_result: Option<String>,
    pub confidence: f64,
}

impl Plan {
    pub fn replay(prior: &str) -> Self {
        Self {
            complexity: TaskComplexity::Trivial,
            steps: vec!["Replay prior result".to_string()],
            prior_result: Some(prior.to_string()),
            confidence: 0.9,
        }
    }

    pub fn immediate() -> Self {
        Self {
            complexity: TaskComplexity::Trivial,
            steps: vec!["Execute directly".to_string()],
            prior_result: None,
            confidence: 0.7,
        }
    }

    pub fn steps(steps: Vec<String>) -> Self {
        Self {
            complexity: TaskComplexity::Simple,
            steps,
            prior_result: None,
            confidence: 0.6,
        }
    }

    pub fn with_hypotheses(hypotheses: Vec<String>) -> Self {
        let steps = vec![
            format!("Evaluate approaches: {}", hypotheses.join(", ")),
            "Select best approach based on evidence".to_string(),
            "Execute selected approach".to_string(),
        ];
        Self {
            complexity: TaskComplexity::Medium,
            steps,
            prior_result: None,
            confidence: 0.5,
        }
    }
}

pub struct PlanningGate;

impl PlanningGate {
    /// Assess task complexity based on a prompt-based evaluation.
    ///
    /// In Phase 1a, this uses a simple heuristic. In Phase 2,
    /// this would use an LLM call for nuanced assessment.
    pub async fn assess(task: &str, has_prior_match: bool, prior_relevance: f64) -> Plan {
        // If we have a highly relevant prior result, replay it
        if has_prior_match && prior_relevance > 0.8 {
            return Plan::replay(&format!("Prior result at relevance {prior_relevance:.2}"));
        }

        // Simple heuristics for Phase 1a:
        let word_count = task.split_whitespace().count();
        let has_multiple_goals = task.matches(|c: char| c == ',' || c == ';').count() > 2;
        let has_subtasks = task.to_lowercase().contains("then")
            || task.to_lowercase().contains("after")
            || task.to_lowercase().contains("first");

        if word_count < 10 && !has_multiple_goals {
            Plan::immediate()
        } else if word_count < 50 && !has_multiple_goals && !has_subtasks {
            Plan::steps(vec![
                "Analyze task requirements".to_string(),
                "Execute solution".to_string(),
                "Verify result".to_string(),
            ])
        } else if has_multiple_goals || has_subtasks {
            Plan::with_hypotheses(vec![
                "Sequential approach".to_string(),
                "Parallel approach".to_string(),
            ])
        } else {
            Plan::steps(vec![
                "Research and plan".to_string(),
                "Implement solution".to_string(),
                "Test and verify".to_string(),
            ])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn trivial_task_replays_prior() {
        let plan = PlanningGate::assess("read file.txt", true, 0.9).await;
        assert_eq!(plan.complexity, TaskComplexity::Trivial);
        assert!(plan.prior_result.is_some());
    }

    #[tokio::test]
    async fn short_task_is_immediate() {
        let plan = PlanningGate::assess("read file.txt", false, 0.0).await;
        assert_eq!(plan.complexity, TaskComplexity::Trivial);
    }

    #[tokio::test]
    async fn medium_task_gets_hypotheses() {
        let plan = PlanningGate::assess(
            "Read the config, check if the server is running, restart if needed, and verify it came back up",
            false,
            0.0,
        )
        .await;
        assert_eq!(plan.complexity, TaskComplexity::Medium);
        assert!(!plan.steps.is_empty());
    }
}
