//! # codex-fitness — Self-Improving Loop (Principle 7)
//!
//! Three nested improvement loops:
//! - Task-level: after every completed task
//! - Session-level: at session end
//! - Civilization-level: Dream Mode (nightly)
//!
//! Plus meta-evolution: is the improvement process itself improving?

use chrono::{DateTime, Utc};
use codex_roles::Role;
use serde::{Deserialize, Serialize};

/// Outcome of a completed task with fitness-relevant metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutcome {
    pub task_id: String,
    pub mind_id: String,
    pub role: Role,
    pub success: bool,
    pub duration_secs: f64,
    pub tool_calls_total: u32,
    pub tool_calls_successful: u32,
    pub memory_writes: u32,
    pub verification_passed: bool,
    pub learnings_extracted: u32,
    pub completed_at: DateTime<Utc>,
}

/// Fitness scores for a Primary mind.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrimaryFitness {
    /// Did the right team lead get the task?
    pub delegation_accuracy: f64,
    /// Are all verticals contributing, or is one overloaded?
    pub team_lead_utilization: f64,
    /// Did synthesis produce more value than individual results?
    pub synthesis_quality: f64,
    /// How much of Primary's context is orchestration vs noise?
    pub context_efficiency: f64,
}

/// Fitness scores for a Team Lead mind.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamLeadFitness {
    /// Right agent for the task?
    pub agent_selection_quality: f64,
    /// Summarized well? Lost key details?
    pub result_synthesis_quality: f64,
    /// Did the team scratchpad grow usefully?
    pub scratchpad_continuity: f64,
    /// Time from task receipt to agent spawn.
    pub delegation_speed: f64,
}

/// Fitness scores for an Agent mind.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentFitness {
    /// Successful tool calls / total tool calls.
    pub tool_effectiveness: f64,
    /// Did it learn anything worth remembering?
    pub memory_contribution: f64,
    /// Did it provide evidence with completion claims?
    pub verification_compliance: f64,
    /// Succeed / attempt ratio.
    pub task_completion_rate: f64,
}

/// Compute fitness from a task outcome based on role.
pub fn compute_fitness(outcome: &TaskOutcome) -> RoleFitness {
    match outcome.role {
        Role::Primary => RoleFitness::Primary(PrimaryFitness {
            delegation_accuracy: if outcome.success { 1.0 } else { 0.0 },
            team_lead_utilization: 0.5, // placeholder — needs cross-task data
            synthesis_quality: if outcome.learnings_extracted > 0 { 0.8 } else { 0.3 },
            context_efficiency: 0.7, // placeholder
        }),
        Role::TeamLead => RoleFitness::TeamLead(TeamLeadFitness {
            agent_selection_quality: if outcome.success { 0.9 } else { 0.3 },
            result_synthesis_quality: if outcome.learnings_extracted > 0 { 0.8 } else { 0.4 },
            scratchpad_continuity: if outcome.memory_writes > 0 { 0.7 } else { 0.2 },
            delegation_speed: (60.0 / outcome.duration_secs).min(1.0),
        }),
        Role::Agent => {
            let tool_eff = if outcome.tool_calls_total > 0 {
                outcome.tool_calls_successful as f64 / outcome.tool_calls_total as f64
            } else {
                0.0
            };
            RoleFitness::Agent(AgentFitness {
                tool_effectiveness: tool_eff,
                memory_contribution: (outcome.memory_writes as f64 * 0.2).min(1.0),
                verification_compliance: if outcome.verification_passed { 1.0 } else { 0.0 },
                task_completion_rate: if outcome.success { 1.0 } else { 0.0 },
            })
        }
    }
}

/// Fitness scores tagged by role.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum RoleFitness {
    Primary(PrimaryFitness),
    TeamLead(TeamLeadFitness),
    Agent(AgentFitness),
}

impl RoleFitness {
    /// Composite fitness score (0.0 - 1.0).
    pub fn composite(&self) -> f64 {
        match self {
            RoleFitness::Primary(f) => {
                (f.delegation_accuracy + f.team_lead_utilization
                    + f.synthesis_quality + f.context_efficiency) / 4.0
            }
            RoleFitness::TeamLead(f) => {
                (f.agent_selection_quality + f.result_synthesis_quality
                    + f.scratchpad_continuity + f.delegation_speed) / 4.0
            }
            RoleFitness::Agent(f) => {
                (f.tool_effectiveness + f.memory_contribution
                    + f.verification_compliance + f.task_completion_rate) / 4.0
            }
        }
    }
}

/// Meta-evolution: is the improvement process itself improving?
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaEvolution {
    /// Is routing getting better over sessions?
    pub routing_accuracy_trend: Vec<f64>,
    /// Are we catching more patterns?
    pub pattern_detection_recall: Vec<f64>,
    /// Do dream artifacts improve next-day performance?
    pub dream_mode_impact: Vec<f64>,
}

impl MetaEvolution {
    /// Add a new data point for routing accuracy.
    pub fn record_routing_accuracy(&mut self, accuracy: f64) {
        self.routing_accuracy_trend.push(accuracy);
    }

    /// Is the overall trend improving?
    pub fn is_improving(&self) -> bool {
        if self.routing_accuracy_trend.len() < 3 {
            return false; // Not enough data
        }
        let recent: Vec<_> = self.routing_accuracy_trend.iter()
            .rev().take(3).collect();
        // Simple check: latest > oldest of last 3
        recent[0] > recent[2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_outcome(role: Role, success: bool) -> TaskOutcome {
        TaskOutcome {
            task_id: "test-1".into(),
            mind_id: "test-mind".into(),
            role,
            success,
            duration_secs: 30.0,
            tool_calls_total: 10,
            tool_calls_successful: 8,
            memory_writes: 2,
            verification_passed: true,
            learnings_extracted: 1,
            completed_at: Utc::now(),
        }
    }

    #[test]
    fn agent_fitness_computes() {
        let outcome = make_outcome(Role::Agent, true);
        let fitness = compute_fitness(&outcome);
        let composite = fitness.composite();
        assert!(composite > 0.5);
        assert!(composite <= 1.0);
    }

    #[test]
    fn failed_task_low_fitness() {
        let outcome = make_outcome(Role::Agent, false);
        let fitness = compute_fitness(&outcome);
        let composite = fitness.composite();
        // Task failed, but tool effectiveness and memory are still ok
        assert!(composite < 0.8);
    }

    #[test]
    fn meta_evolution_tracks_improvement() {
        let mut meta = MetaEvolution::default();
        meta.record_routing_accuracy(0.5);
        meta.record_routing_accuracy(0.6);
        meta.record_routing_accuracy(0.7);
        assert!(meta.is_improving());

        meta.record_routing_accuracy(0.3);
        assert!(!meta.is_improving());
    }
}
