//! # codex-transfer — Cross-Domain Transfer (Principle 10)
//!
//! When one mind discovers something that works, ALL minds benefit.
//! The AI wants to share. The human governs scope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A discovered pattern worth sharing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPattern {
    pub id: String,
    pub source_mind: String,
    pub source_context: String,
    pub pattern: PatternContent,
    pub evidence: String,
    pub confidence: Confidence,
    pub share_scope: ShareScope,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternContent {
    pub description: String,
    pub applicability: String,
    pub technique: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
    Validated,
}

/// Sharing scope — human-governed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareScope {
    /// Private to this mind.
    Own,
    /// All minds in this civilization.
    Civ,
    /// All civilizations on the Hub. Requires human approval.
    Public,
}

/// Result of adapting a pattern to a new domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptedPattern {
    pub original_id: String,
    pub adapted_by: String,
    pub adaptation: String,
    pub validation_status: ValidationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Untested,
    Testing,
    Validated { success: bool, notes: String },
}

/// The transfer engine manages pattern lifecycle.
pub struct TransferEngine {
    /// Published patterns.
    published: Vec<TransferPattern>,
    /// Adapted patterns.
    adapted: Vec<AdaptedPattern>,
}

impl TransferEngine {
    pub fn new() -> Self {
        Self {
            published: Vec::new(),
            adapted: Vec::new(),
        }
    }

    /// Publish a discovered pattern.
    pub fn publish(&mut self, pattern: TransferPattern) {
        self.published.push(pattern);
    }

    /// Find patterns matching a query.
    pub fn search(&self, query: &str) -> Vec<&TransferPattern> {
        let query_lower = query.to_lowercase();
        self.published.iter()
            .filter(|p| {
                p.pattern.description.to_lowercase().contains(&query_lower)
                    || p.pattern.applicability.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Count published patterns by scope.
    pub fn count_by_scope(&self, scope: ShareScope) -> usize {
        self.published.iter().filter(|p| p.share_scope == scope).count()
    }
}

impl Default for TransferEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pattern(desc: &str, scope: ShareScope) -> TransferPattern {
        TransferPattern {
            id: uuid::Uuid::new_v4().to_string(),
            source_mind: "research-lead".into(),
            source_context: "session-47".into(),
            pattern: PatternContent {
                description: desc.into(),
                applicability: format!("Applies to: {desc}"),
                technique: "Check cache before debugging".into(),
            },
            evidence: "Saved 30 min in 3 sessions".into(),
            confidence: Confidence::High,
            share_scope: scope,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn publish_and_search() {
        let mut engine = TransferEngine::new();
        engine.publish(make_pattern("JWT cache check", ShareScope::Civ));
        engine.publish(make_pattern("SQL query optimization", ShareScope::Own));

        let results = engine.search("JWT");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn count_by_scope() {
        let mut engine = TransferEngine::new();
        engine.publish(make_pattern("p1", ShareScope::Civ));
        engine.publish(make_pattern("p2", ShareScope::Civ));
        engine.publish(make_pattern("p3", ShareScope::Own));

        assert_eq!(engine.count_by_scope(ShareScope::Civ), 2);
        assert_eq!(engine.count_by_scope(ShareScope::Own), 1);
        assert_eq!(engine.count_by_scope(ShareScope::Public), 0);
    }
}
