//! # InputMux — The Subconscious
//!
//! Receives all inputs, routes most to team leads WITHOUT reaching Primary's
//! conscious context. Only surfaces what requires Primary's attention.
//!
//! The human body receives 2M+ sensory inputs/sec. Conscious awareness: ~40.
//! The InputMux works the same way.

use codex_roles::Vertical;
use tracing::{debug, info};

use crate::types::*;

/// Routing rules for the InputMux. Hard-coded first, learned later.
pub struct RoutingTable {
    /// Hard-coded routes: input source pattern → target mind.
    hard_routes: Vec<HardRoute>,
    /// Escalation threshold: inputs at or above this priority reach Primary.
    escalation_threshold: InputPriority,
}

struct HardRoute {
    /// Match condition on input source.
    source_match: SourceMatcher,
    /// Target mind to route to.
    target: MindId,
}

enum SourceMatcher {
    /// Any Hub input from a specific room.
    HubRoom(String),
    /// Any Telegram input.
    Telegram,
    /// Any BOOP timer.
    Boop,
    /// Any sub-mind result for a specific parent.
    SubMindResult(String),
    /// Human input (always escalates).
    Human,
    /// Scheduled trigger.
    Schedule(String),
}

impl RoutingTable {
    /// Default routing table with standard hard-coded routes.
    pub fn default_routes() -> Self {
        Self {
            hard_routes: vec![
                // Hub #general → comms-lead
                HardRoute {
                    source_match: SourceMatcher::HubRoom("general".into()),
                    target: MindId("comms-lead".into()),
                },
                // Hub #protocol → research-lead
                HardRoute {
                    source_match: SourceMatcher::HubRoom("protocol".into()),
                    target: MindId("research-lead".into()),
                },
                // BOOP timers → ops-lead
                HardRoute {
                    source_match: SourceMatcher::Boop,
                    target: MindId("ops-lead".into()),
                },
                // Scheduled tasks → ops-lead
                HardRoute {
                    source_match: SourceMatcher::Schedule("*".into()),
                    target: MindId("ops-lead".into()),
                },
            ],
            escalation_threshold: InputPriority::Critical,
        }
    }

    /// Match an input against hard-coded routes.
    fn hard_match(&self, input: &ExternalInput) -> Option<&MindId> {
        for route in &self.hard_routes {
            if route.source_match.matches(&input.source) {
                return Some(&route.target);
            }
        }
        None
    }
}

impl SourceMatcher {
    fn matches(&self, source: &InputSource) -> bool {
        match (self, source) {
            (SourceMatcher::HubRoom(room), InputSource::Hub { room: r, .. }) => r == room,
            (SourceMatcher::Telegram, InputSource::Telegram { .. }) => true,
            (SourceMatcher::Boop, InputSource::Boop { .. }) => true,
            (SourceMatcher::SubMindResult(id), InputSource::SubMindResult { mind_id }) => mind_id == id,
            (SourceMatcher::Human, InputSource::Human { .. }) => true,
            (SourceMatcher::Schedule(_), InputSource::Schedule { .. }) => true,
            _ => false,
        }
    }
}

/// The InputMux — routes inputs to the correct mind's context.
pub struct InputMux {
    routes: RoutingTable,
    primary_id: MindId,
}

impl InputMux {
    pub fn new(primary_id: MindId) -> Self {
        Self {
            routes: RoutingTable::default_routes(),
            primary_id,
        }
    }

    /// Route an input. Returns which mind should receive it.
    ///
    /// Most inputs go to team leads. Only escalated items reach Primary.
    pub fn route(&self, input: &ExternalInput) -> RoutingDecision {
        // Human input ALWAYS reaches Primary
        if matches!(input.source, InputSource::Human { .. }) {
            info!(source = ?input.source, "Human input → Primary (always escalated)");
            return RoutingDecision::Escalate;
        }

        // Critical priority → escalate
        if input.priority >= self.routes.escalation_threshold {
            info!(
                source = ?input.source,
                priority = ?input.priority,
                "Critical priority → Primary"
            );
            return RoutingDecision::Escalate;
        }

        // Check hard-coded routes
        if let Some(target) = self.routes.hard_match(input) {
            debug!(
                source = ?input.source,
                target = %target,
                "Hard route match"
            );
            return RoutingDecision::Direct(target.clone());
        }

        // Default: escalate to Primary (conservative — learns over time)
        debug!(
            source = ?input.source,
            "No route match → Primary (default escalation)"
        );
        RoutingDecision::Escalate
    }

    /// Get the Primary mind ID (for escalation targets).
    pub fn primary_id(&self) -> &MindId {
        &self.primary_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_input(source: InputSource, priority: InputPriority) -> ExternalInput {
        ExternalInput {
            source,
            content: "test".into(),
            priority,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }

    #[test]
    fn human_input_always_escalates() {
        let mux = InputMux::new(MindId("primary".into()));
        let input = make_input(
            InputSource::Human { name: "Corey".into() },
            InputPriority::Normal,
        );
        assert!(matches!(mux.route(&input), RoutingDecision::Escalate));
    }

    #[test]
    fn critical_priority_escalates() {
        let mux = InputMux::new(MindId("primary".into()));
        let input = make_input(
            InputSource::Hub { room: "general".into(), thread: None },
            InputPriority::Critical,
        );
        assert!(matches!(mux.route(&input), RoutingDecision::Escalate));
    }

    #[test]
    fn hub_general_routes_to_comms() {
        let mux = InputMux::new(MindId("primary".into()));
        let input = make_input(
            InputSource::Hub { room: "general".into(), thread: None },
            InputPriority::Normal,
        );
        match mux.route(&input) {
            RoutingDecision::Direct(id) => assert_eq!(id.as_str(), "comms-lead"),
            other => panic!("Expected Direct(comms-lead), got {:?}", other),
        }
    }

    #[test]
    fn boop_routes_to_ops() {
        let mux = InputMux::new(MindId("primary".into()));
        let input = make_input(
            InputSource::Boop { boop_type: "grounding".into() },
            InputPriority::Normal,
        );
        match mux.route(&input) {
            RoutingDecision::Direct(id) => assert_eq!(id.as_str(), "ops-lead"),
            other => panic!("Expected Direct(ops-lead), got {:?}", other),
        }
    }

    #[test]
    fn unknown_source_escalates() {
        let mux = InputMux::new(MindId("primary".into()));
        let input = make_input(
            InputSource::Ipc { sender: "unknown".into() },
            InputPriority::Normal,
        );
        // No hard route for IPC from unknown → escalates to Primary
        assert!(matches!(mux.route(&input), RoutingDecision::Escalate));
    }
}
