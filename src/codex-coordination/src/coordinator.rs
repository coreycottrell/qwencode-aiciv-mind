//! # CoordinatorLoop — The Main Event Loop
//!
//! The fractal coordination engine's top-level orchestrator.
//! Integrates MindManager, InputMux, PlanningGate, and TriggerEngine.

use codex_roles::Vertical;
use tracing::{info, warn};

use crate::input_mux::InputMux;
use crate::mind_manager::{CoordinationError, MindManager};
use crate::planning::{PlanningDecision, PlanningGate};
use crate::triggers::{RotationTrigger, TriggerEngine};
use crate::types::*;

/// The top-level coordination loop.
///
/// In production, this wraps Codex's app server and manages the hierarchy
/// of ThreadManagers (one per persistent mind).
pub struct CoordinatorLoop {
    /// Manages all active minds.
    pub mind_manager: MindManager,
    /// Routes inputs to the correct mind.
    pub input_mux: InputMux,
    /// Scales planning depth with task complexity.
    pub planning_gate: PlanningGate,
    /// Detects patterns and triggers automatic spawning.
    pub trigger_engine: TriggerEngine,
    /// 3-hour scratchpad rotation.
    pub rotation_trigger: RotationTrigger,
}

impl CoordinatorLoop {
    /// Initialize a new coordinator with Primary mind.
    pub fn new(agents_dir: String, scratchpads_dir: String) -> Self {
        let mut mind_manager = MindManager::new(agents_dir, scratchpads_dir);
        let primary_id = mind_manager.init_primary();

        Self {
            mind_manager,
            input_mux: InputMux::new(primary_id),
            planning_gate: PlanningGate::new(),
            trigger_engine: TriggerEngine::new(),
            rotation_trigger: RotationTrigger::new(3), // 3-hour rotation
        }
    }

    /// Process an external input through the full pipeline.
    ///
    /// 1. InputMux routes to correct mind
    /// 2. PlanningGate evaluates complexity
    /// 3. Task is delegated or executed
    pub fn process_input(&mut self, input: ExternalInput) -> Result<ProcessResult, CoordinationError> {
        // Step 1: Route via InputMux
        let routing = self.input_mux.route(&input);

        let target_id = match routing {
            RoutingDecision::Direct(id) => {
                info!(target = %id, "InputMux → direct route");
                id
            }
            RoutingDecision::Escalate => {
                let primary = self.input_mux.primary_id().clone();
                info!("InputMux → escalated to Primary");
                primary
            }
            RoutingDecision::Drop { reason } => {
                info!(reason = reason, "InputMux → dropped");
                return Ok(ProcessResult::Dropped { reason });
            }
        };

        // Step 2: Planning gate
        let planning = self.planning_gate.evaluate(&input.content, &[]);

        match planning {
            PlanningDecision::Execute { memory_hit } => {
                info!(memory_hit = ?memory_hit, "Planning → execute immediately");
                let task = self.mind_manager.delegate(
                    self.input_mux.primary_id(),
                    &target_id,
                    &input.content,
                )?;
                Ok(ProcessResult::Delegated {
                    task_id: task.id,
                    target: target_id,
                })
            }
            PlanningDecision::ExecuteWithPlan { plan } => {
                info!(steps = plan.len(), "Planning → execute with plan");
                let enriched = format!("{}\n\nPlan:\n{}", input.content,
                    plan.iter().enumerate()
                        .map(|(i, s)| format!("{}. {}", i + 1, s))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                let task = self.mind_manager.delegate(
                    self.input_mux.primary_id(),
                    &target_id,
                    &enriched,
                )?;
                Ok(ProcessResult::Delegated {
                    task_id: task.id,
                    target: target_id,
                })
            }
            PlanningDecision::SpawnPlanner { reason } => {
                warn!(reason = reason, "Planning → need planner sub-mind");
                Ok(ProcessResult::NeedsPlanner { reason })
            }
            PlanningDecision::SpawnCompetingPlanners { approaches } => {
                warn!(count = approaches.len(), "Planning → competing planners");
                Ok(ProcessResult::NeedsCompetingPlanners { approaches })
            }
        }
    }

    /// Spawn a team lead for a vertical.
    pub fn spawn_team_lead(
        &mut self,
        vertical: Vertical,
        objective: &str,
    ) -> Result<MindId, CoordinationError> {
        self.mind_manager.spawn_team_lead(vertical, objective)
    }

    /// Check and process spawn triggers.
    pub fn check_triggers(&mut self) -> Vec<SpawnTriggerResult> {
        let mut results = Vec::new();

        // Check blocking
        let blocking_triggers = self.trigger_engine.check_blocking();
        for trigger in blocking_triggers {
            results.push(SpawnTriggerResult {
                trigger,
                action: "spawn_fresh_context".into(),
            });
        }

        // Check rotation
        if self.rotation_trigger.is_due() {
            results.push(SpawnTriggerResult {
                trigger: crate::triggers::SpawnTrigger::Scheduled {
                    trigger_name: "scratchpad_rotation".into(),
                },
                action: "rotate_scratchpads".into(),
            });
        }

        results
    }

    /// Get coordination state snapshot.
    pub fn state(&self) -> CoordinationState {
        self.mind_manager.coordination_state()
    }
}

/// Result of processing an input through the pipeline.
#[derive(Debug)]
pub enum ProcessResult {
    /// Task was delegated to a mind.
    Delegated { task_id: String, target: MindId },
    /// Input was dropped (noise).
    Dropped { reason: String },
    /// Needs a planner sub-mind (too complex for immediate execution).
    NeedsPlanner { reason: String },
    /// Needs competing planners (novel task).
    NeedsCompetingPlanners { approaches: Vec<String> },
}

/// Result of checking spawn triggers.
#[derive(Debug)]
pub struct SpawnTriggerResult {
    pub trigger: crate::triggers::SpawnTrigger,
    pub action: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_coordinator() -> CoordinatorLoop {
        CoordinatorLoop::new("agents".into(), "scratchpads".into())
    }

    #[test]
    fn coordinator_initializes_with_primary() {
        let coord = make_coordinator();
        let state = coord.state();
        assert_eq!(state.minds.len(), 1);
        assert_eq!(state.minds[0].role, codex_roles::Role::Primary);
    }

    #[test]
    fn spawn_and_delegate() {
        let mut coord = make_coordinator();
        let lead_id = coord.spawn_team_lead(Vertical::Research, "Research stuff").unwrap();

        let input = ExternalInput {
            source: InputSource::Hub {
                room: "protocol".into(),
                thread: None,
            },
            content: "analyze this".into(),
            priority: InputPriority::Normal,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        };

        let result = coord.process_input(input).unwrap();
        assert!(matches!(result, ProcessResult::Delegated { .. }));
    }

    #[test]
    fn human_input_escalates_to_primary() {
        let mut coord = make_coordinator();
        // Need at least Primary to delegate to itself
        let input = ExternalInput {
            source: InputSource::Human { name: "Corey".into() },
            content: "hello".into(),
            priority: InputPriority::Normal,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        };

        // This will escalate to Primary and try to delegate Primary → Primary
        // which works (Primary can self-assign)
        let result = coord.process_input(input);
        // The delegation from primary to primary should work
        assert!(result.is_ok());
    }
}
