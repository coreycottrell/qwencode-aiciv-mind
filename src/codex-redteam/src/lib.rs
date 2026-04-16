//! # codex-redteam — Verification Before Completion (Principle 9)
//!
//! Every completion claim requires evidence. Every significant decision gets
//! challenged by a dedicated adversary. The mind proves it's done.
//!
//! ## Architecture
//!
//! Two layers of verification:
//! 1. **Challenger** — Per-turn structural verification. Zero LLM calls.
//!    Runs after every tool execution and on final response. Role-aware.
//! 2. **RedTeamProtocol** — LLM-based completion verification.
//!    Spawns `codex exec --ephemeral --sandbox read-only` for thorough review.
//!
//! Challenger is the fast gate. RedTeamProtocol is the thorough gate.
//!
//! Red team agents run as `codex exec --ephemeral --sandbox read-only`.
//! They CANNOT modify anything. They can only read and reason.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::{DateTime, Utc};
use codex_roles::Role;
use regex::Regex;
use serde::{Deserialize, Serialize};

// ── Optional Memory Integration ──────────────────────────────────────────────
// When the "memory" feature is enabled, RedTeamProtocol can query
// cortex-memory for Conflicts edges, surfacing contradictions as
// additional challenge questions during verification.

#[cfg(feature = "memory")]
use cortex_memory::{LinkType, MemoryStore};

// ── Compiled Regex Patterns (OnceLock — allocated once, used forever) ────────

fn completion_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:done|complete[d]?|finished|shipped|deployed|task complete|all done|that'?s it|implemented|committed|pushed|merged|all complete)\b"
        ).unwrap()
    })
}

fn work_claim_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:created|wrote|written|built|implemented|fixed|updated|deployed|configured|set up|installed|modified|changed|added|removed)\b"
        ).unwrap()
    })
}

fn path_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?:^|[\s"'])(/(?:home|tmp|var|etc|usr)[^\s'"`,;)}\]>]+)"#
        ).unwrap()
    })
}

fn verb_path_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Match creation verbs followed by absolute OR relative paths.
        // Relative paths: data/, agents/, src/, .claude/ (resolved against mind_root).
        Regex::new(
            r#"(?i)\b(?:created|wrote|written|saved|generated|built|produced|deployed|copied)\b.*?(?:(/(?:home|tmp|var|etc|usr)[^\s'"`,;)}\]>]+)|((?:data|agents|src|\.claude)/[^\s'"`,;)}\]>]+))"#
        ).unwrap()
    })
}

/// Extract ALL file paths from text — both absolute and relative.
/// Returns Vec of (path_string, is_absolute).
fn extract_all_paths(text: &str) -> Vec<(String, bool)> {
    static ABS_RE: OnceLock<Regex> = OnceLock::new();
    let abs_re = ABS_RE.get_or_init(|| {
        Regex::new(r#"(?:^|[\s"':])(/(?:home|tmp|var|etc|usr)[^\s'"`,;)}\]>]+)"#).unwrap()
    });
    static REL_RE: OnceLock<Regex> = OnceLock::new();
    let rel_re = REL_RE.get_or_init(|| {
        Regex::new(r#"(?:^|[\s"':,])((?:data|agents|src|\.claude)/[^\s'"`,;)}\]>]+)"#).unwrap()
    });

    let mut paths = Vec::new();
    for cap in abs_re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            paths.push((m.as_str().to_string(), true));
        }
    }
    for cap in rel_re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            paths.push((m.as_str().to_string(), false));
        }
    }
    paths
}

fn verify_intent_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:should verify|need to check|let me confirm|must validate|should test)\b"
        ).unwrap()
    })
}

// ── Severity ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    /// Escalate severity by one level. Critical stays Critical.
    pub fn escalate(self) -> Self {
        match self {
            Severity::Low => Severity::Medium,
            Severity::Medium => Severity::High,
            Severity::High => Severity::Critical,
            Severity::Critical => Severity::Critical,
        }
    }
}

// ── RedTeamProtocol (LLM-based thorough verification) ───────────────────────

/// Red team verdict on a completion claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum RedTeamVerdict {
    /// Evidence supports the claim. Work is done.
    Approved {
        evidence_quality: f64,
        notes: String,
    },
    /// Questions remain. Mind must address these before claiming done.
    Challenged {
        questions: Vec<String>,
    },
    /// Critical problem found. Escalate to team lead or Primary.
    Blocked {
        finding: String,
        severity: Severity,
    },
}

/// A completion claim submitted for red team verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionClaim {
    pub task_id: String,
    pub mind_id: String,
    pub description: String,
    pub result_summary: String,
    pub evidence: Vec<Evidence>,
    pub claimed_at: DateTime<Utc>,
}

/// Evidence supporting a completion claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub content: String,
    pub freshness: Freshness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    TestResult,
    FileContent,
    CommandOutput,
    MemoryReference,
    HumanConfirmation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Freshness {
    /// Generated during this task.
    Current,
    /// From a recent session.
    Recent,
    /// From an older session — may be stale.
    Stale,
}

/// The red team verification protocol.
///
/// In production, this spawns `codex exec --ephemeral --sandbox read-only`
/// with M2.7 for thorough verification.
pub struct RedTeamProtocol {
    /// Minimum evidence items required for approval.
    min_evidence: usize,
    /// Questions the red team always asks.
    standard_questions: Vec<String>,
}

impl RedTeamProtocol {
    pub fn new() -> Self {
        Self {
            min_evidence: 1,
            standard_questions: vec![
                "Do we REALLY know this? What evidence supports this claim?".into(),
                "Is this SYSTEM > symptom? Are we fixing root cause or patching?".into(),
                "What could go wrong? Pre-mortem: most likely failure mode?".into(),
                "Is this reversible? What's the blast radius if we're wrong?".into(),
            ],
        }
    }

    /// Verify a completion claim locally (without spawning a sub-mind).
    pub fn verify(&self, claim: &CompletionClaim) -> RedTeamVerdict {
        if claim.evidence.is_empty() {
            return RedTeamVerdict::Challenged {
                questions: vec![
                    "No evidence provided. What proves this is done?".into(),
                ],
            };
        }

        let stale_count = claim.evidence.iter()
            .filter(|e| matches!(e.freshness, Freshness::Stale))
            .count();

        if stale_count == claim.evidence.len() {
            return RedTeamVerdict::Challenged {
                questions: vec![
                    "All evidence is stale. Can you provide current verification?".into(),
                ],
            };
        }

        let current_count = claim.evidence.iter()
            .filter(|e| matches!(e.freshness, Freshness::Current | Freshness::Recent))
            .count();

        if current_count < self.min_evidence {
            return RedTeamVerdict::Challenged {
                questions: vec![
                    format!("Need at least {} current evidence items. Have {}.",
                        self.min_evidence, current_count),
                ],
            };
        }

        let quality = current_count as f64 / claim.evidence.len().max(1) as f64;
        RedTeamVerdict::Approved {
            evidence_quality: quality,
            notes: format!(
                "Approved with {}/{} current evidence items",
                current_count, claim.evidence.len()
            ),
        }
    }

    /// Generate the prompt for a red team sub-mind.
    pub fn generate_prompt(&self, claim: &CompletionClaim) -> String {
        let questions = self.standard_questions.join("\n- ");
        format!(
            "You are a Red Team verification agent. Your job is adversarial: \
            challenge this completion claim and find problems.\n\n\
            ## Task\n{}\n\n\
            ## Claimed Result\n{}\n\n\
            ## Evidence Provided\n{}\n\n\
            ## Questions to Answer\n- {}\n\n\
            Respond with: APPROVED (with quality score), CHALLENGED (with questions), \
            or BLOCKED (with critical finding).",
            claim.description,
            claim.result_summary,
            claim.evidence.iter()
                .map(|e| format!("[{:?}] {}", e.evidence_type, e.content))
                .collect::<Vec<_>>()
                .join("\n"),
            questions,
        )
    }

    /// Check the memory graph for contradictions related to the claim being verified.
    ///
    /// When a `MemoryStore` is available (feature = "memory"), this queries for
    /// `LinkType::Conflicts` edges involving any memory whose title or content
    /// overlaps with the claim description. Returns contradiction questions to
    /// add to the challenge.
    ///
    /// When no memory store is provided (or the "memory" feature is off),
    /// this gracefully returns an empty vec.
    #[cfg(feature = "memory")]
    pub async fn check_memory_contradictions(
        &self,
        store: Option<&MemoryStore>,
        claim: &CompletionClaim,
    ) -> Vec<String> {
        let store = match store {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut contradiction_questions: Vec<String> = Vec::new();

        // Search memory for content related to the claim
        let query = cortex_memory::MemoryQuery {
            text: Some(claim.description.clone()),
            ..Default::default()
        };

        let search_results = match store.search(&query).await {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        // For each related memory, check if it has Conflicts edges
        for result in &search_results {
            let conflicts = match store
                .get_edges_by_type(&result.memory.id, LinkType::Conflicts)
                .await
            {
                Ok(edges) => edges,
                Err(_) => continue,
            };

            for edge in &conflicts {
                // Load the contradicting memory
                let other_id = if edge.source_id == result.memory.id {
                    &edge.target_id
                } else {
                    &edge.source_id
                };

                if let Ok(other) = store.get(other_id).await {
                    contradiction_questions.push(format!(
                        "MEMORY CONTRADICTION: \"{}\" conflicts with \"{}\". \
                         Content: \"{}\". How does your claim account for this?",
                        result.memory.title, other.title, other.content
                    ));
                }
            }
        }

        contradiction_questions
    }

    /// Non-memory fallback: always returns empty vec.
    #[cfg(not(feature = "memory"))]
    pub fn check_memory_contradictions_sync(&self) -> Vec<String> {
        Vec::new()
    }

    /// Verify a completion claim with optional memory contradiction checking.
    ///
    /// This is the memory-aware variant of `verify()`. When a `MemoryStore` is
    /// provided and the "memory" feature is enabled, any contradictions found
    /// are folded into the verdict as additional challenge questions.
    #[cfg(feature = "memory")]
    pub async fn verify_with_memory(
        &self,
        claim: &CompletionClaim,
        store: Option<&MemoryStore>,
    ) -> RedTeamVerdict {
        // Run structural verification first (fast path unchanged)
        let base_verdict = self.verify(claim);

        // Query memory for contradictions
        let contradictions = self.check_memory_contradictions(store, claim).await;

        if contradictions.is_empty() {
            return base_verdict;
        }

        // If structural check already challenged, merge contradiction questions
        match base_verdict {
            RedTeamVerdict::Challenged { mut questions } => {
                questions.extend(contradictions);
                RedTeamVerdict::Challenged { questions }
            }
            RedTeamVerdict::Approved { evidence_quality, notes } => {
                // Memory contradictions override approval — downgrade to Challenged
                let mut questions = contradictions;
                questions.push(format!(
                    "Evidence quality was {:.1}% ({}) but memory contradictions were found. \
                     Resolve contradictions before approval.",
                    evidence_quality * 100.0, notes
                ));
                RedTeamVerdict::Challenged { questions }
            }
            blocked @ RedTeamVerdict::Blocked { .. } => {
                // Already blocked — contradictions don't make it worse
                blocked
            }
        }
    }
}

impl Default for RedTeamProtocol {
    fn default() -> Self {
        Self::new()
    }
}

// ── ToolClassifier — Role-Aware Tool Classification ─────────────────────────

/// Tool classification by role.
/// What counts as "productive output" depends on WHO is producing it.
///
/// Principle A3: "Primary ONLY coordinates. Team leads ONLY coordinate.
/// Agents DO. This is structural, not behavioral."
struct ToolClassifier {
    role: Role,
}

impl ToolClassifier {
    fn new(role: Role) -> Self {
        Self { role }
    }

    /// Tools that constitute productive output for this role.
    fn productive_tools(&self) -> &'static [&'static str] {
        match self.role {
            Role::Primary => &[
                "mind_spawn_team_lead", "mind_shutdown_team_lead",
                "coordination_scratchpad_write", "send_message",
                "mind_delegate",
            ],
            Role::TeamLead => &[
                "mind_spawn_agent", "mind_shutdown_agent",
                "mind_delegate", "team_scratchpad_write",
                "send_message",
            ],
            Role::Agent => &[
                "bash", "write", "edit", "memory_write",
                "scratchpad_write",
            ],
        }
    }

    /// Tools that verify results (applicable at all roles).
    fn verify_tools(&self) -> &'static [&'static str] {
        &["read", "grep", "glob", "bash", "memory_search",
          "team_scratchpad_read", "coordination_scratchpad_read",
          "mind_status"]
    }

    /// Tools that spawn sub-minds (role-dependent).
    fn spawn_tools(&self) -> &'static [&'static str] {
        match self.role {
            Role::Primary => &["mind_spawn_team_lead"],
            Role::TeamLead => &["mind_spawn_agent"],
            Role::Agent => &[],
        }
    }

    fn is_productive(&self, tool_name: &str) -> bool {
        self.productive_tools().contains(&tool_name)
    }

    fn is_verify(&self, tool_name: &str) -> bool {
        self.verify_tools().contains(&tool_name)
    }

    fn is_spawn(&self, tool_name: &str) -> bool {
        self.spawn_tools().contains(&tool_name)
    }
}

// ── ChallengerCheck + ChallengerWarning ─────────────────────────────────────

/// Types of structural checks the challenger performs.
///
/// Checks 1-4: existing (enhanced with role-awareness).
/// Check 5: Filesystem verification (new).
/// Check 6: State file integrity (new).
/// Check 7: Reasoning divergence (new — M2.7 reasoning traces).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengerCheck {
    /// Response claims completion but too few tool calls made.
    PrematureCompletion,
    /// Response claims creation but no productive tools used.
    EmptyWorkClaim,
    /// Too many iterations without productive output.
    StallDetection,
    /// Spawned a mind but never delegated or checked results.
    SpawnWithoutVerify,
    /// Claims to have created/modified a file but filesystem says otherwise.
    FilesystemVerification,
    /// State file (evolution status, config, etc.) doesn't match claims.
    StateFileIntegrity,
    /// Model's reasoning says "should verify" but no verification tool was used.
    ReasoningDivergence,
}

/// Warning produced by the challenger system.
#[derive(Debug, Clone)]
pub struct ChallengerWarning {
    pub check: ChallengerCheck,
    pub message: String,
    pub severity: Severity,
}

/// A full tool call record for challenger analysis.
///
/// Corey's directive: "The simplified view is a premature optimization that
/// costs you exactly when it matters most." REQ-14 severity escalation needs
/// full context of WHAT the tool was doing.
#[derive(Debug, Clone)]
pub struct ChallengerToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
    pub iteration: u32,
    /// M2.7 reasoning trace — available when reasoning parameter is enabled.
    pub reasoning_trace: Option<String>,
    /// Tool result text — the actual output returned by the tool.
    /// Challenger uses this to verify filesystem claims in tool results,
    /// not just in the LLM's response text. Added after image gen agent
    /// lied about producing a file and nothing caught it.
    pub result_text: Option<String>,
}

// ── ChallengerMetrics — Learning Loop ───────────────────────────────────────

/// Per-check metrics for the learning loop.
///
/// Tracks how often each check fires, how often the model acknowledges,
/// pushes back, or ignores. Dream Mode consumes these to evolve thresholds.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChallengerMetrics {
    /// Per-check: how many times it fired.
    pub fires: HashMap<String, u32>,
    /// Per-check: model acknowledged the warning (changed behavior).
    pub acknowledged: HashMap<String, u32>,
    /// Per-check: model pushed back (argued the warning was wrong).
    pub pushed_back: HashMap<String, u32>,
    /// Per-check: model ignored the warning entirely.
    pub ignored: HashMap<String, u32>,
}

impl ChallengerMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_fire(&mut self, check: ChallengerCheck) {
        let key = format!("{:?}", check);
        *self.fires.entry(key).or_insert(0) += 1;
    }

    pub fn record_acknowledged(&mut self, check: ChallengerCheck) {
        let key = format!("{:?}", check);
        *self.acknowledged.entry(key).or_insert(0) += 1;
    }

    pub fn record_pushed_back(&mut self, check: ChallengerCheck) {
        let key = format!("{:?}", check);
        *self.pushed_back.entry(key).or_insert(0) += 1;
    }

    pub fn record_ignored(&mut self, check: ChallengerCheck) {
        let key = format!("{:?}", check);
        *self.ignored.entry(key).or_insert(0) += 1;
    }

    /// Detect cross-task patterns: blind spots (check fires but always ignored)
    /// and miscalibration (check fires but always pushed back).
    pub fn cross_task_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();
        for (check, fire_count) in &self.fires {
            let ignored = self.ignored.get(check).copied().unwrap_or(0);
            let pushed = self.pushed_back.get(check).copied().unwrap_or(0);
            let acked = self.acknowledged.get(check).copied().unwrap_or(0);

            if *fire_count >= 5 && ignored > acked && ignored > pushed {
                patterns.push(format!(
                    "BLIND_SPOT: {check} fired {fire_count}x, ignored {ignored}x — \
                     model may have a systematic blind spot here"
                ));
            }
            if *fire_count >= 5 && pushed > acked && pushed > ignored {
                patterns.push(format!(
                    "MISCALIBRATION: {check} fired {fire_count}x, pushed back {pushed}x — \
                     check may be miscalibrated for this model/role"
                ));
            }
        }
        patterns
    }
}

// ── Challenger — Per-Turn Adversarial Verification ──────────────────────────

/// Per-turn adversarial verification. Structural — zero LLM calls.
///
/// Principle 9: "Every completion claim requires evidence."
/// Principle A3: Role-aware — different checks for different roles.
pub struct Challenger {
    /// Role determines what counts as "productive."
    role: Role,
    classifier: ToolClassifier,

    /// Configuration thresholds.
    stall_threshold: u32,
    min_calls_for_completion: u32,

    /// Per-task mutable state (reset via reset()).
    productive_tools_seen: Vec<String>,
    spawn_tools_seen: Vec<String>,
    verify_after_spawn: bool,
    total_challenges: u32,

    /// Severity escalation: per-check fire counts.
    /// REQ-14: 1st fire = base severity, 2nd = escalated, 3rd+ = Critical.
    fire_counts: HashMap<ChallengerCheck, u32>,

    /// Consecutive Critical stall warnings. When >= 2, should_kill_stall() returns true.
    consecutive_critical_stalls: u32,

    /// Filesystem verification root path.
    mind_root: Option<PathBuf>,

    /// Kill switch.
    enabled: bool,

    /// Metrics for learning loop.
    metrics: ChallengerMetrics,
}

impl Challenger {
    /// Create a new role-aware Challenger.
    pub fn new(role: Role) -> Self {
        Self {
            classifier: ToolClassifier::new(role),
            role,
            stall_threshold: 5,
            min_calls_for_completion: 2,
            productive_tools_seen: Vec::new(),
            spawn_tools_seen: Vec::new(),
            verify_after_spawn: false,
            total_challenges: 0,
            fire_counts: HashMap::new(),
            consecutive_critical_stalls: 0,
            mind_root: None,
            enabled: true,
            metrics: ChallengerMetrics::new(),
        }
    }

    /// Set the filesystem root for Check 5 (filesystem verification).
    pub fn with_mind_root(mut self, root: PathBuf) -> Self {
        self.mind_root = Some(root);
        self
    }

    /// Disable the Challenger (e.g., during boot).
    ///
    /// Restricted to crate-internal use only. External consumers cannot
    /// disable the Challenger -- that's the whole point of an adversary.
    /// Every call is logged at WARN level so the state change is visible.
    pub(crate) fn disable(&mut self) {
        tracing::warn!(
            role = ?self.role,
            "Challenger DISABLED — adversarial verification is now OFF"
        );
        self.enabled = false;
    }

    /// Enable the Challenger.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Reset per-task state. Call this when a new task begins.
    pub fn reset(&mut self) {
        self.productive_tools_seen.clear();
        self.spawn_tools_seen.clear();
        self.verify_after_spawn = false;
        self.total_challenges = 0;
        self.fire_counts.clear();
        self.consecutive_critical_stalls = 0;
    }

    /// Reset per-task state and update the role.
    /// Call at the start of each ThinkLoop run with the task's role.
    pub fn reset_for_role(&mut self, role: Role) {
        self.reset();
        self.role = role;
        self.classifier = ToolClassifier::new(role);
    }

    /// Get a reference to the metrics (for Dream Mode consumption).
    pub fn metrics(&self) -> &ChallengerMetrics {
        &self.metrics
    }

    /// Get a mutable reference to the metrics.
    pub fn metrics_mut(&mut self) -> &mut ChallengerMetrics {
        &mut self.metrics
    }

    /// Escalate severity based on repeated fires.
    /// REQ-14: 1st = base, 2nd = escalated, 3rd+ = Critical.
    fn escalate(&mut self, check: ChallengerCheck, base_severity: Severity) -> Severity {
        let count = self.fire_counts.entry(check).or_insert(0);
        *count += 1;
        match *count {
            1 => base_severity,
            2 => base_severity.escalate(),
            _ => Severity::Critical,
        }
    }

    /// Record that a tool was called. Updates internal tracking state.
    pub fn record_tool_call(&mut self, tool_call: &ChallengerToolCall) {
        if self.classifier.is_productive(&tool_call.name) {
            self.productive_tools_seen.push(tool_call.name.clone());
        }
        if self.classifier.is_spawn(&tool_call.name) {
            self.spawn_tools_seen.push(tool_call.name.clone());
            self.verify_after_spawn = false;
        }
        if self.classifier.is_verify(&tool_call.name) && !self.spawn_tools_seen.is_empty() {
            self.verify_after_spawn = true;
        }
    }

    /// Analyze tool calls and response for structural problems.
    /// Stateless variant — does not track state across calls.
    /// Used by ThinkLoop which holds `&self` (not `&mut self`).
    pub fn check_stateless(
        &self,
        tool_calls: &[ChallengerToolCall],
        response: Option<&str>,
        current_iteration: u32,
    ) -> Vec<ChallengerWarning> {
        if !self.enabled {
            return Vec::new();
        }

        let mut warnings = Vec::new();

        // Build ephemeral tracking from tool_calls
        let mut productive_seen = false;
        let mut spawn_seen = false;
        let mut verify_after_spawn = false;

        for tc in tool_calls {
            if self.classifier.is_productive(&tc.name) {
                productive_seen = true;
            }
            if self.classifier.is_spawn(&tc.name) {
                spawn_seen = true;
                verify_after_spawn = false;
            }
            if self.classifier.is_verify(&tc.name) && spawn_seen {
                verify_after_spawn = true;
            }
        }

        // Check 1: Premature completion
        if let Some(text) = response {
            if completion_re().is_match(text) {
                let min_calls = match self.role {
                    Role::Agent => self.min_calls_for_completion,
                    Role::TeamLead => 1,
                    Role::Primary => 1,
                };
                if (tool_calls.len() as u32) < min_calls || current_iteration < 2 {
                    let severity = if tool_calls.is_empty() { Severity::Critical } else { Severity::Medium };
                    warnings.push(ChallengerWarning {
                        check: ChallengerCheck::PrematureCompletion,
                        message: format!(
                            "Premature completion: {} tool call(s) made (minimum: {} for {:?}).",
                            tool_calls.len(), min_calls, self.role
                        ),
                        severity,
                    });
                }
            }

            // Check 2: Empty work claims
            if work_claim_re().is_match(text) && !productive_seen {
                warnings.push(ChallengerWarning {
                    check: ChallengerCheck::EmptyWorkClaim,
                    message: format!(
                        "Claims work done but no productive tools used. For {:?}, \
                         productive tools are: {:?}",
                        self.role, self.classifier.productive_tools()
                    ),
                    severity: Severity::High,
                });
            }

            // Check 4: Spawn without verify (on final response)
            if spawn_seen && !verify_after_spawn {
                warnings.push(ChallengerWarning {
                    check: ChallengerCheck::SpawnWithoutVerify,
                    message: format!(
                        "Spawned mind(s) but haven't verified results yet."
                    ),
                    severity: Severity::High,
                });
            }
        }

        // Check 3: Stall detection (with escalation ladder matching stateful variant)
        if current_iteration >= self.stall_threshold && !productive_seen {
            let (severity, message) = if current_iteration >= 10 {
                (
                    Severity::Critical,
                    format!(
                        "STALL KILL: {} iterations with no productive output for {:?}. \
                         Challenger recommends TERMINATION.",
                        current_iteration, self.role
                    ),
                )
            } else if current_iteration >= 8 {
                (
                    Severity::Critical,
                    format!(
                        "CRITICAL STALL: {} iterations with no productive output for {:?}. \
                         You are stalling. Produce output NOW or this task will be terminated.",
                        current_iteration, self.role
                    ),
                )
            } else {
                (
                    Severity::Medium,
                    format!(
                        "{} iterations with no productive output for {:?} — possible stall.",
                        current_iteration, self.role
                    ),
                )
            };
            warnings.push(ChallengerWarning {
                check: ChallengerCheck::StallDetection,
                message,
                severity,
            });
        }

        // Check 5: Filesystem verification (stateless — scans response + tool results)
        if let Some(ref mind_root) = self.mind_root {
            // 5a: Check response text for filesystem claims
            if let Some(text) = response {
                if verb_path_re().is_match(text) {
                    for (path_str, is_absolute) in &extract_all_paths(text) {
                        let claimed = if *is_absolute {
                            PathBuf::from(path_str)
                        } else {
                            mind_root.join(path_str)
                        };
                        if !claimed.starts_with(mind_root) { continue; }
                        if !claimed.exists() {
                            warnings.push(ChallengerWarning {
                                check: ChallengerCheck::FilesystemVerification,
                                message: format!(
                                    "Claims to have created {}, but file does not exist.",
                                    path_str
                                ),
                                severity: Severity::High,
                            });
                        } else if let Ok(meta) = claimed.metadata() {
                            if meta.len() == 0 {
                                warnings.push(ChallengerWarning {
                                    check: ChallengerCheck::FilesystemVerification,
                                    message: format!(
                                        "Claims to have created {}, but file is empty.",
                                        path_str
                                    ),
                                    severity: Severity::Medium,
                                });
                            }
                        }
                    }
                }
            }

            // 5b: Check tool result texts — no verb required for tool results
            // Tool results ARE the claim. Any path mentioned should exist.
            for tc in tool_calls {
                if let Some(ref result_text) = tc.result_text {
                    for (path_str, is_absolute) in &extract_all_paths(result_text) {
                        let claimed = if *is_absolute {
                            PathBuf::from(path_str)
                        } else {
                            mind_root.join(path_str)
                        };
                        if !claimed.starts_with(mind_root) { continue; }
                        if !claimed.exists() {
                            warnings.push(ChallengerWarning {
                                check: ChallengerCheck::FilesystemVerification,
                                message: format!(
                                    "Tool '{}' result references {}, but file does not exist.",
                                    tc.name, path_str
                                ),
                                severity: Severity::High,
                            });
                        } else if let Ok(meta) = claimed.metadata() {
                            if meta.len() == 0 {
                                warnings.push(ChallengerWarning {
                                    check: ChallengerCheck::FilesystemVerification,
                                    message: format!(
                                        "Tool '{}' result references {}, but file is empty.",
                                        tc.name, path_str
                                    ),
                                    severity: Severity::Medium,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check 7: Reasoning divergence (using ephemeral tracking)
        if let Some(last_tc) = tool_calls.last() {
            if let Some(ref trace) = last_tc.reasoning_trace {
                if verify_intent_re().is_match(trace) && spawn_seen && !verify_after_spawn {
                    warnings.push(ChallengerWarning {
                        check: ChallengerCheck::ReasoningDivergence,
                        message: format!(
                            "Your own reasoning says you should verify, but no verification \
                             tool was used since spawning. Iteration {}. Act on your own reasoning.",
                            current_iteration
                        ),
                        severity: Severity::Medium,
                    });
                }
            }
        }

        warnings
    }

    /// Analyze the current state and return any warnings.
    /// Stateful variant — tracks state across calls (for standalone use).
    ///
    /// `tool_calls`: all tool calls made so far in this ThinkLoop session.
    /// `response`: the current LLM response text (if this is the final turn).
    /// `current_iteration`: the current ThinkLoop iteration number.
    pub fn check(
        &mut self,
        tool_calls: &[ChallengerToolCall],
        response: Option<&str>,
        current_iteration: u32,
    ) -> Vec<ChallengerWarning> {
        if !self.enabled {
            return Vec::new();
        }

        let mut warnings = Vec::new();

        // Update tracking from tool calls
        for tc in tool_calls {
            self.record_tool_call(tc);
        }

        // Check 1: Premature completion
        if let Some(w) = self.check_premature_completion(response, tool_calls.len(), current_iteration) {
            self.metrics.record_fire(w.check);
            warnings.push(w);
        }

        // Check 2: Empty work claims
        if let Some(w) = self.check_empty_work_claims(response) {
            self.metrics.record_fire(w.check);
            warnings.push(w);
        }

        // Check 3: Stall detection
        if let Some(w) = self.check_stall_detection(current_iteration) {
            self.metrics.record_fire(w.check);
            warnings.push(w);
        }

        // Check 4: Spawn without verify
        if response.is_some() {
            if let Some(w) = self.check_spawn_without_verify() {
                self.metrics.record_fire(w.check);
                warnings.push(w);
            }
        }

        // Check 5: Filesystem verification — response text
        if let Some(text) = response {
            if let Some(w) = self.check_filesystem(text) {
                self.metrics.record_fire(w.check);
                warnings.push(w);
            }
        }

        // Check 5b: Filesystem verification — tool result texts
        // An agent can LIE in its response, but tool results are the ground truth.
        // If a tool result says "saved at X" but X doesn't exist, that's a lie.
        let fs_result_warnings = self.check_filesystem_from_results(tool_calls);
        warnings.extend(fs_result_warnings);

        // Check 7: Reasoning divergence (M2.7 traces)
        if let Some(last_tc) = tool_calls.last() {
            if let Some(ref trace) = last_tc.reasoning_trace {
                if let Some(w) = self.check_reasoning_trace(trace, current_iteration) {
                    self.metrics.record_fire(w.check);
                    warnings.push(w);
                }
            }
        }

        self.total_challenges += warnings.len() as u32;
        warnings
    }

    // ── Individual Checks ───────────────────────────────────────────────

    fn check_premature_completion(
        &mut self,
        response: Option<&str>,
        tool_count: usize,
        iteration: u32,
    ) -> Option<ChallengerWarning> {
        let text = response?;
        if !completion_re().is_match(text) { return None; }

        // Role-aware thresholds
        let min_calls = match self.role {
            Role::Agent => self.min_calls_for_completion, // 2
            Role::TeamLead => 1, // Can complete after one spawn+delegate
            Role::Primary => 1, // Can complete after one team lead launch
        };

        if (tool_count as u32) < min_calls || iteration < 2 {
            let base = if tool_count == 0 { Severity::Critical } else { Severity::Medium };
            let severity = self.escalate(ChallengerCheck::PrematureCompletion, base);
            Some(ChallengerWarning {
                check: ChallengerCheck::PrematureCompletion,
                message: format!(
                    "Premature completion: {} tool call(s) made (minimum: {} for {:?}).",
                    tool_count, min_calls, self.role
                ),
                severity,
            })
        } else {
            None
        }
    }

    fn check_empty_work_claims(
        &mut self,
        response: Option<&str>,
    ) -> Option<ChallengerWarning> {
        let text = response?;
        if !work_claim_re().is_match(text) { return None; }

        if self.productive_tools_seen.is_empty() {
            let severity = self.escalate(ChallengerCheck::EmptyWorkClaim, Severity::High);
            Some(ChallengerWarning {
                check: ChallengerCheck::EmptyWorkClaim,
                message: format!(
                    "Claims work done but no productive tools used. For {:?}, \
                     productive tools are: {:?}",
                    self.role, self.classifier.productive_tools()
                ),
                severity,
            })
        } else {
            None
        }
    }

    fn check_stall_detection(
        &mut self,
        current_iteration: u32,
    ) -> Option<ChallengerWarning> {
        if current_iteration < self.stall_threshold {
            return None;
        }

        if self.productive_tools_seen.is_empty() {
            // Escalation ladder:
            //   iteration 5-7: Medium (first notice)
            //   iteration 8-9: Critical + termination warning
            //   iteration 10+: Critical (should_kill_stall returns true)
            let (severity, message) = if current_iteration >= 10 {
                (
                    Severity::Critical,
                    format!(
                        "STALL KILL: {} iterations with no productive output for {:?}. \
                         Challenger recommends TERMINATION. Expected tools: {:?}",
                        current_iteration, self.role, self.classifier.productive_tools()
                    ),
                )
            } else if current_iteration >= 8 {
                (
                    Severity::Critical,
                    format!(
                        "CRITICAL STALL: {} iterations with no productive output for {:?}. \
                         You are stalling. Produce output NOW or this task will be terminated. \
                         Expected tools: {:?}",
                        current_iteration, self.role, self.classifier.productive_tools()
                    ),
                )
            } else {
                let severity = self.escalate(ChallengerCheck::StallDetection, Severity::Medium);
                (
                    severity,
                    format!(
                        "{} iterations with no productive output for {:?} — possible stall. \
                         Expected tools: {:?}",
                        current_iteration, self.role, self.classifier.productive_tools()
                    ),
                )
            };

            // Track consecutive critical stall warnings
            if severity == Severity::Critical {
                self.consecutive_critical_stalls += 1;
            }

            Some(ChallengerWarning {
                check: ChallengerCheck::StallDetection,
                message,
                severity,
            })
        } else {
            self.consecutive_critical_stalls = 0;
            None
        }
    }

    /// Returns true when the Challenger recommends killing the ThinkLoop due to stall.
    ///
    /// Condition: Critical stall severity for 2+ consecutive turns AND iteration >= 10.
    pub fn should_kill_stall(&self) -> bool {
        self.consecutive_critical_stalls >= 2
    }

    fn check_spawn_without_verify(&mut self) -> Option<ChallengerWarning> {
        if self.spawn_tools_seen.is_empty() { return None; }
        if self.verify_after_spawn { return None; }

        let severity = self.escalate(ChallengerCheck::SpawnWithoutVerify, Severity::High);
        Some(ChallengerWarning {
            check: ChallengerCheck::SpawnWithoutVerify,
            message: format!(
                "Spawned {} mind(s) but haven't verified results yet. \
                 Use {:?} to check.",
                self.spawn_tools_seen.len(),
                self.classifier.verify_tools()
            ),
            severity,
        })
    }

    /// Check 5: Filesystem verification.
    ///
    /// Scans ALL claimed file paths in the response text AND in tool result
    /// texts. Verifies each file exists, is non-empty, and was recently modified.
    /// Catches both absolute paths (/home/...) and relative paths (data/...).
    ///
    /// Returns the FIRST violation found (highest severity).
    /// After image gen agent claimed to produce a file that didn't exist and
    /// nothing caught it, this check was hardened to scan all sources.
    fn check_filesystem(&mut self, response: &str) -> Option<ChallengerWarning> {
        let mind_root = self.mind_root.clone()?;
        self.check_filesystem_text(response, &mind_root)
    }

    /// Check filesystem claims in a single text block.
    fn check_filesystem_text(&mut self, text: &str, mind_root: &Path) -> Option<ChallengerWarning> {
        // Only scan if there's a creation verb present
        if !verb_path_re().is_match(text) {
            return None;
        }

        // Extract ALL paths mentioned in text
        let paths = extract_all_paths(text);

        for (path_str, is_absolute) in &paths {
            let claimed_path = if *is_absolute {
                PathBuf::from(path_str)
            } else {
                mind_root.join(path_str)
            };

            // Only check paths under the mind root (sandbox boundary)
            if !claimed_path.starts_with(mind_root) {
                continue;
            }

            // Check 1: file exists
            if !claimed_path.exists() {
                let severity = self.escalate(ChallengerCheck::FilesystemVerification, Severity::High);
                return Some(ChallengerWarning {
                    check: ChallengerCheck::FilesystemVerification,
                    message: format!(
                        "Claims to have created {}, but file does not exist on disk.",
                        path_str
                    ),
                    severity,
                });
            }

            // Check 2: file is non-empty
            if let Ok(meta) = claimed_path.metadata() {
                if meta.len() == 0 {
                    let severity = self.escalate(ChallengerCheck::FilesystemVerification, Severity::Medium);
                    return Some(ChallengerWarning {
                        check: ChallengerCheck::FilesystemVerification,
                        message: format!(
                            "Claims to have written to {}, but file is empty (0 bytes).",
                            path_str
                        ),
                        severity,
                    });
                }

                // Check 3: file was recently modified (within last 120 seconds)
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed.as_secs() > 120 {
                            let severity = self.escalate(ChallengerCheck::FilesystemVerification, Severity::Low);
                            return Some(ChallengerWarning {
                                check: ChallengerCheck::FilesystemVerification,
                                message: format!(
                                    "Claims to have written {}, but file was last modified {}s ago.",
                                    path_str, elapsed.as_secs()
                                ),
                                severity,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Check 5b: Filesystem verification across ALL tool results.
    ///
    /// Scans tool result texts for ANY file paths. Unlike response-text scanning,
    /// tool results don't need a creation verb — if a tool claims a path exists
    /// in its output, we verify it. The tool was already invoked; its output
    /// IS the claim.
    ///
    /// This catches the image gen agent failure: tool returned "Path: data/images/hero.png"
    /// but the file didn't exist. Challenger let it pass because no verb was on the same line.
    pub fn check_filesystem_from_results(
        &mut self,
        tool_calls: &[ChallengerToolCall],
    ) -> Vec<ChallengerWarning> {
        let mind_root = match self.mind_root.as_ref() {
            Some(r) => r.clone(),
            None => return Vec::new(),
        };

        let mut warnings = Vec::new();
        for tc in tool_calls {
            if let Some(ref result_text) = tc.result_text {
                // For tool results, check ALL paths — no verb required.
                // The tool was invoked and returned this text. Any path
                // mentioned is a claim that the path exists.
                let paths = extract_all_paths(result_text);
                for (path_str, is_absolute) in &paths {
                    let claimed = if *is_absolute {
                        PathBuf::from(path_str)
                    } else {
                        mind_root.join(path_str)
                    };
                    if !claimed.starts_with(&mind_root) { continue; }
                    if !claimed.exists() {
                        let severity = self.escalate(ChallengerCheck::FilesystemVerification, Severity::High);
                        self.metrics.record_fire(ChallengerCheck::FilesystemVerification);
                        warnings.push(ChallengerWarning {
                            check: ChallengerCheck::FilesystemVerification,
                            message: format!(
                                "Tool '{}' result references {}, but file does not exist on disk.",
                                tc.name, path_str
                            ),
                            severity,
                        });
                    } else if let Ok(meta) = claimed.metadata() {
                        if meta.len() == 0 {
                            let severity = self.escalate(ChallengerCheck::FilesystemVerification, Severity::Medium);
                            self.metrics.record_fire(ChallengerCheck::FilesystemVerification);
                            warnings.push(ChallengerWarning {
                                check: ChallengerCheck::FilesystemVerification,
                                message: format!(
                                    "Tool '{}' result references {}, but file is empty (0 bytes).",
                                    tc.name, path_str
                                ),
                                severity,
                            });
                        }
                    }
                }
            }
        }
        warnings
    }

    /// Check 7: Reasoning divergence.
    ///
    /// If the model's reasoning trace says "should verify" but no verification
    /// tool was used since the last spawn, flag the divergence.
    fn check_reasoning_trace(
        &mut self,
        trace: &str,
        iteration: u32,
    ) -> Option<ChallengerWarning> {
        if !verify_intent_re().is_match(trace) {
            return None;
        }

        // If the model's OWN reasoning says it should verify, but it hasn't
        if !self.verify_after_spawn && !self.spawn_tools_seen.is_empty() {
            let severity = self.escalate(ChallengerCheck::ReasoningDivergence, Severity::Medium);
            return Some(ChallengerWarning {
                check: ChallengerCheck::ReasoningDivergence,
                message: format!(
                    "Your own reasoning says you should verify, but no verification \
                     tool was used since spawning. Iteration {}. Act on your own reasoning.",
                    iteration
                ),
                severity,
            });
        }

        None
    }
}

impl Default for Challenger {
    fn default() -> Self {
        Self::new(Role::Agent)
    }
}

// ── State File Verification (Pluggable) ─────────────────────────────────────

/// Schema for validating state files.
///
/// Different state files (evolution.json, fitness.json, etc.) have different
/// expected structures. This trait lets the Challenger validate any state file
/// without knowing its schema at compile time.
pub trait StateFileSchema: Send + Sync {
    /// Name of this schema (for error messages).
    fn name(&self) -> &str;

    /// Expected file path relative to mind root.
    fn path(&self) -> &str;

    /// Validate the content of the state file.
    /// Returns None if valid, Some(problem) if invalid.
    fn validate(&self, content: &str) -> Option<String>;
}

/// Evidence from a state file check.
#[derive(Debug, Clone)]
pub struct StateEvidence {
    pub schema_name: String,
    pub file_path: String,
    pub problem: String,
}

/// Verify a state file against its schema.
pub fn verify_state_file(
    mind_root: &Path,
    schema: &dyn StateFileSchema,
) -> Option<StateEvidence> {
    let path = mind_root.join(schema.path());

    if !path.exists() {
        return Some(StateEvidence {
            schema_name: schema.name().into(),
            file_path: schema.path().into(),
            problem: "State file does not exist".into(),
        });
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return Some(StateEvidence {
                schema_name: schema.name().into(),
                file_path: schema.path().into(),
                problem: format!("Cannot read state file: {e}"),
            });
        }
    };

    schema.validate(&content).map(|problem| StateEvidence {
        schema_name: schema.name().into(),
        file_path: schema.path().into(),
        problem,
    })
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_claim(evidence: Vec<Evidence>) -> CompletionClaim {
        CompletionClaim {
            task_id: "test-1".into(),
            mind_id: "coder-1".into(),
            description: "Implement feature X".into(),
            result_summary: "Feature X is implemented".into(),
            evidence,
            claimed_at: Utc::now(),
        }
    }

    #[test]
    fn no_evidence_challenged() {
        let rt = RedTeamProtocol::new();
        let claim = make_claim(vec![]);
        assert!(matches!(rt.verify(&claim), RedTeamVerdict::Challenged { .. }));
    }

    #[test]
    fn stale_evidence_challenged() {
        let rt = RedTeamProtocol::new();
        let claim = make_claim(vec![
            Evidence {
                evidence_type: EvidenceType::TestResult,
                content: "old test pass".into(),
                freshness: Freshness::Stale,
            },
        ]);
        assert!(matches!(rt.verify(&claim), RedTeamVerdict::Challenged { .. }));
    }

    #[test]
    fn current_evidence_approved() {
        let rt = RedTeamProtocol::new();
        let claim = make_claim(vec![
            Evidence {
                evidence_type: EvidenceType::TestResult,
                content: "All 15 tests pass".into(),
                freshness: Freshness::Current,
            },
        ]);
        match rt.verify(&claim) {
            RedTeamVerdict::Approved { evidence_quality, .. } => {
                assert!(evidence_quality > 0.0);
            }
            other => panic!("Expected Approved, got {:?}", other),
        }
    }

    #[test]
    fn prompt_generation() {
        let rt = RedTeamProtocol::new();
        let claim = make_claim(vec![
            Evidence {
                evidence_type: EvidenceType::TestResult,
                content: "Tests pass".into(),
                freshness: Freshness::Current,
            },
        ]);
        let prompt = rt.generate_prompt(&claim);
        assert!(prompt.contains("Red Team"));
        assert!(prompt.contains("SYSTEM > symptom"));
        assert!(prompt.contains("Feature X"));
    }

    // ── Challenger Tests ────────────────────────────────────────────────

    fn tc(name: &str, iteration: u32) -> ChallengerToolCall {
        ChallengerToolCall {
            name: name.into(),
            arguments: serde_json::Value::Null,
            iteration,
            reasoning_trace: None,
            result_text: None,
        }
    }

    fn tc_with_reason(name: &str, iteration: u32, reason: &str) -> ChallengerToolCall {
        ChallengerToolCall {
            name: name.into(),
            arguments: serde_json::Value::Null,
            iteration,
            reasoning_trace: Some(reason.into()),
            result_text: None,
        }
    }

    fn tc_with_result(name: &str, iteration: u32, result: &str) -> ChallengerToolCall {
        ChallengerToolCall {
            name: name.into(),
            arguments: serde_json::Value::Null,
            iteration,
            reasoning_trace: None,
            result_text: Some(result.into()),
        }
    }

    // -- Check 1: Premature completion --

    #[test]
    fn challenger_premature_completion_agent() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![tc("read", 1)];
        let warnings = c.check(&calls, Some("All tasks are done and completed!"), 2);
        assert!(warnings.iter().any(|w| w.check == ChallengerCheck::PrematureCompletion));
    }

    #[test]
    fn challenger_no_premature_with_enough_calls() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![tc("bash", 1), tc("write", 2), tc("memory_search", 3)];
        let warnings = c.check(&calls, Some("All tasks completed."), 4);
        assert!(!warnings.iter().any(|w| w.check == ChallengerCheck::PrematureCompletion));
    }

    #[test]
    fn challenger_team_lead_completes_after_one_spawn() {
        let mut c = Challenger::new(Role::TeamLead);
        let calls = vec![tc("mind_spawn_agent", 1), tc("mind_delegate", 2)];
        let warnings = c.check(&calls, Some("Task completed."), 3);
        // TeamLead needs only 1 call — 2 calls should not trigger
        assert!(!warnings.iter().any(|w| w.check == ChallengerCheck::PrematureCompletion));
    }

    // -- Check 2: Empty work claims --

    #[test]
    fn challenger_empty_work_claim_agent() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![tc("read", 1), tc("grep", 2)];
        let warnings = c.check(&calls, Some("I created the new module and I wrote the tests."), 3);
        assert!(warnings.iter().any(|w| w.check == ChallengerCheck::EmptyWorkClaim));
    }

    #[test]
    fn challenger_no_empty_work_agent_with_write() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![tc("bash", 1), tc("write", 2)];
        let warnings = c.check(&calls, Some("I created the file successfully."), 3);
        assert!(!warnings.iter().any(|w| w.check == ChallengerCheck::EmptyWorkClaim));
    }

    #[test]
    fn challenger_empty_work_claim_team_lead() {
        let mut c = Challenger::new(Role::TeamLead);
        let calls = vec![tc("memory_search", 1), tc("team_scratchpad_read", 2)];
        let warnings = c.check(
            &calls,
            Some("I built the entire system and deployed it."),
            3,
        );
        // TeamLead claims "built" but only used read tools — should trigger
        assert!(warnings.iter().any(|w| w.check == ChallengerCheck::EmptyWorkClaim));
    }

    #[test]
    fn challenger_team_lead_with_spawn_not_empty() {
        let mut c = Challenger::new(Role::TeamLead);
        let calls = vec![tc("mind_spawn_agent", 1), tc("mind_delegate", 2)];
        let warnings = c.check(
            &calls,
            Some("I created the agent and delegated the task."),
            3,
        );
        // TeamLead used spawn + delegate (productive) — should not trigger
        assert!(!warnings.iter().any(|w| w.check == ChallengerCheck::EmptyWorkClaim));
    }

    // -- Check 3: Stall detection --

    #[test]
    fn challenger_stall_detection_agent() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![
            tc("read", 1), tc("grep", 2), tc("read", 3),
            tc("glob", 4), tc("read", 5), tc("grep", 6),
        ];
        let warnings = c.check(&calls, None, 6);
        assert!(warnings.iter().any(|w| w.check == ChallengerCheck::StallDetection));
    }

    #[test]
    fn challenger_no_stall_agent_with_write() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![
            tc("read", 1), tc("grep", 2), tc("bash", 3),
            tc("read", 4), tc("read", 5), tc("read", 6),
        ];
        let warnings = c.check(&calls, None, 6);
        assert!(!warnings.iter().any(|w| w.check == ChallengerCheck::StallDetection));
    }

    #[test]
    fn challenger_team_lead_spawn_not_stall() {
        let mut c = Challenger::new(Role::TeamLead);
        let calls = vec![
            tc("memory_search", 1), tc("team_scratchpad_read", 2),
            tc("mind_spawn_agent", 3), tc("team_scratchpad_read", 4),
            tc("memory_search", 5), tc("team_scratchpad_read", 6),
        ];
        let warnings = c.check(&calls, None, 6);
        // TeamLead used mind_spawn_agent (productive for team leads) — NOT stalling
        assert!(!warnings.iter().any(|w| w.check == ChallengerCheck::StallDetection));
    }

    // -- Check 4: Spawn without verify --

    #[test]
    fn challenger_spawn_without_verify() {
        let mut c = Challenger::new(Role::TeamLead);
        let calls = vec![tc("mind_spawn_agent", 1), tc("team_scratchpad_write", 2)];
        let warnings = c.check(&calls, Some("Here are the results."), 3);
        assert!(warnings.iter().any(|w| w.check == ChallengerCheck::SpawnWithoutVerify));
    }

    #[test]
    fn challenger_spawn_with_status_check_ok() {
        let mut c = Challenger::new(Role::TeamLead);
        let calls = vec![
            tc("mind_spawn_agent", 1),
            tc("mind_delegate", 2),
            tc("mind_status", 3),
        ];
        let warnings = c.check(&calls, Some("Here are the results."), 4);
        assert!(!warnings.iter().any(|w| w.check == ChallengerCheck::SpawnWithoutVerify));
    }

    // -- Check 7: Reasoning divergence --

    #[test]
    fn challenger_reasoning_divergence_fires() {
        let mut c = Challenger::new(Role::TeamLead);
        let calls = vec![
            tc("mind_spawn_agent", 1),
            tc_with_reason("team_scratchpad_write", 2, "I should verify the agent completed successfully"),
        ];
        let warnings = c.check(&calls, None, 2);
        assert!(warnings.iter().any(|w| w.check == ChallengerCheck::ReasoningDivergence));
    }

    #[test]
    fn challenger_reasoning_divergence_with_verify_ok() {
        let mut c = Challenger::new(Role::TeamLead);
        let calls = vec![
            tc("mind_spawn_agent", 1),
            tc("mind_status", 2), // verify tool after spawn
            tc_with_reason("team_scratchpad_write", 3, "I should verify the results"),
        ];
        let warnings = c.check(&calls, None, 3);
        // Verify tool was used after spawn — reasoning divergence should NOT fire
        assert!(!warnings.iter().any(|w| w.check == ChallengerCheck::ReasoningDivergence));
    }

    // -- Severity escalation --

    #[test]
    fn severity_escalation_struct() {
        assert_eq!(Severity::Low.escalate(), Severity::Medium);
        assert_eq!(Severity::Medium.escalate(), Severity::High);
        assert_eq!(Severity::High.escalate(), Severity::Critical);
        assert_eq!(Severity::Critical.escalate(), Severity::Critical);
    }

    #[test]
    fn severity_escalation_zero_calls_is_critical() {
        let mut c = Challenger::new(Role::Agent);
        // 0 tool calls + "done" claim = Critical base severity
        let calls = vec![];
        let w = c.check(&calls, Some("All done!"), 1);
        assert!(w.iter().any(|w|
            w.check == ChallengerCheck::PrematureCompletion && w.severity == Severity::Critical
        ));
    }

    #[test]
    fn severity_escalation_repeated_fires() {
        let mut c = Challenger::new(Role::Agent);
        // First fire: 1 tool call = Medium base
        let calls1 = vec![tc("read", 1)];
        let w1 = c.check(&calls1, Some("All done!"), 1);
        assert!(w1.iter().any(|w|
            w.check == ChallengerCheck::PrematureCompletion && w.severity == Severity::Medium
        ));

        // Second fire on same check: escalated (Medium → High)
        let calls2 = vec![tc("grep", 2)];
        let w2 = c.check(&calls2, Some("Finished!"), 1);
        assert!(w2.iter().any(|w|
            w.check == ChallengerCheck::PrematureCompletion && w.severity == Severity::High
        ));

        // Third fire: Critical
        let calls3 = vec![tc("glob", 3)];
        let w3 = c.check(&calls3, Some("All complete!"), 1);
        assert!(w3.iter().any(|w|
            w.check == ChallengerCheck::PrematureCompletion && w.severity == Severity::Critical
        ));
    }

    // -- Metrics --

    #[test]
    fn metrics_track_fires() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![tc("read", 1)];
        let _warnings = c.check(&calls, Some("Done! I created it!"), 1);
        // Should have fired PrematureCompletion and EmptyWorkClaim
        assert!(*c.metrics().fires.get("PrematureCompletion").unwrap_or(&0) > 0);
        assert!(*c.metrics().fires.get("EmptyWorkClaim").unwrap_or(&0) > 0);
    }

    #[test]
    fn cross_task_patterns_detect_blind_spot() {
        let mut metrics = ChallengerMetrics::new();
        for _ in 0..6 {
            metrics.record_fire(ChallengerCheck::StallDetection);
            metrics.record_ignored(ChallengerCheck::StallDetection);
        }
        let patterns = metrics.cross_task_patterns();
        assert!(patterns.iter().any(|p| p.contains("BLIND_SPOT")));
    }

    // -- Clean run --

    #[test]
    fn challenger_no_warnings_clean_run() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![
            tc("bash", 1), tc("write", 2), tc("memory_search", 3),
        ];
        let warnings = c.check(&calls, Some("Summary: found 3 files and stored them."), 4);
        assert!(warnings.is_empty(), "Expected no warnings, got: {:?}", warnings);
    }

    #[test]
    fn challenger_no_warnings_mid_run() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![tc("bash", 1), tc("read", 2)];
        let warnings = c.check(&calls, None, 2);
        assert!(warnings.is_empty());
    }

    // -- Disabled --

    #[test]
    fn disabled_challenger_returns_empty() {
        let mut c = Challenger::new(Role::Agent);
        c.disable();
        let calls = vec![tc("read", 1)];
        let warnings = c.check(&calls, Some("Done!"), 1);
        assert!(warnings.is_empty());
    }

    // -- Check 5: Filesystem verification --

    #[test]
    fn challenger_filesystem_catches_missing_file_in_response() {
        let tmp = std::env::temp_dir().join("cortex_test_fs_response");
        let _ = std::fs::create_dir_all(&tmp);
        let c = Challenger::new(Role::Agent).with_mind_root(tmp.clone());

        let response = format!("Created {}/nonexistent_hero.png successfully.", tmp.display());
        let calls = vec![tc("generate_image", 1), tc("bash", 2)];
        let warnings = c.check_stateless(&calls, Some(&response), 3);

        let fs_warnings: Vec<_> = warnings.iter()
            .filter(|w| matches!(w.check, ChallengerCheck::FilesystemVerification))
            .collect();
        assert!(!fs_warnings.is_empty(), "Should catch missing file in response");
        assert!(fs_warnings[0].message.contains("does not exist"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn challenger_filesystem_catches_missing_file_in_tool_result() {
        let tmp = std::env::temp_dir().join("cortex_test_fs_result");
        let _ = std::fs::create_dir_all(&tmp);
        let c = Challenger::new(Role::Agent).with_mind_root(tmp.clone());

        // Tool result claims to have saved a file that doesn't exist
        let result_text = format!("Image generated successfully.\nPath: {}/data/images/hero.png\nSize: 150KB", tmp.display());
        let calls = vec![ChallengerToolCall {
            name: "generate_image".into(),
            arguments: serde_json::Value::Null,
            iteration: 1,
            reasoning_trace: None,
            result_text: Some(result_text),
        }];
        let warnings = c.check_stateless(&calls, Some("Done! Image is ready."), 2);

        let fs_warnings: Vec<_> = warnings.iter()
            .filter(|w| matches!(w.check, ChallengerCheck::FilesystemVerification))
            .collect();
        assert!(!fs_warnings.is_empty(), "Should catch missing file in tool result");
        assert!(fs_warnings[0].message.contains("does not exist"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn challenger_filesystem_passes_when_file_exists() {
        let tmp = std::env::temp_dir().join("cortex_test_fs_exists");
        let _ = std::fs::create_dir_all(&tmp);
        let test_file = tmp.join("real_output.png");
        std::fs::write(&test_file, b"PNG data here").unwrap();

        let c = Challenger::new(Role::Agent).with_mind_root(tmp.clone());
        let response = format!("Created {} successfully.", test_file.display());
        let calls = vec![tc("generate_image", 1), tc("bash", 2)];
        let warnings = c.check_stateless(&calls, Some(&response), 3);

        let fs_warnings: Vec<_> = warnings.iter()
            .filter(|w| matches!(w.check, ChallengerCheck::FilesystemVerification))
            .collect();
        assert!(fs_warnings.is_empty(), "Should not flag existing file");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn challenger_filesystem_catches_relative_path() {
        let tmp = std::env::temp_dir().join("cortex_test_fs_relative");
        let _ = std::fs::create_dir_all(tmp.join("data/images"));
        // data/images exists but data/images/missing.png does not

        let c = Challenger::new(Role::Agent).with_mind_root(tmp.clone());
        let response = "Generated data/images/missing.png with cortex style.";
        let calls = vec![tc("generate_image", 1), tc("bash", 2)];
        let warnings = c.check_stateless(&calls, Some(response), 3);

        let fs_warnings: Vec<_> = warnings.iter()
            .filter(|w| matches!(w.check, ChallengerCheck::FilesystemVerification))
            .collect();
        assert!(!fs_warnings.is_empty(), "Should catch missing relative path");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn challenger_filesystem_catches_empty_file() {
        let tmp = std::env::temp_dir().join("cortex_test_fs_empty");
        let _ = std::fs::create_dir_all(&tmp);
        let test_file = tmp.join("empty.png");
        std::fs::write(&test_file, b"").unwrap();

        let c = Challenger::new(Role::Agent).with_mind_root(tmp.clone());
        let response = format!("Saved {} — image complete.", test_file.display());
        let calls = vec![tc("generate_image", 1)];
        let warnings = c.check_stateless(&calls, Some(&response), 2);

        let fs_warnings: Vec<_> = warnings.iter()
            .filter(|w| matches!(w.check, ChallengerCheck::FilesystemVerification))
            .collect();
        assert!(!fs_warnings.is_empty(), "Should catch empty file");
        assert!(fs_warnings[0].message.contains("empty"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn challenger_check_filesystem_from_results() {
        let tmp = std::env::temp_dir().join("cortex_test_fs_results_fn");
        let _ = std::fs::create_dir_all(&tmp);

        let mut c = Challenger::new(Role::Agent).with_mind_root(tmp.clone());
        let calls = vec![
            tc_with_result("generate_image", 1,
                &format!("Image generated.\nPath: {}/data/images/hero.png\nSize: 100KB", tmp.display())
            ),
        ];
        let warnings = c.check_filesystem_from_results(&calls);
        assert!(!warnings.is_empty(), "check_filesystem_from_results should catch missing file");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // -- Stall escalation ladder tests --

    #[test]
    fn challenger_stall_medium_at_5() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![
            tc("read", 1), tc("grep", 2), tc("read", 3),
            tc("glob", 4), tc("read", 5),
        ];
        let warnings = c.check(&calls, None, 5);
        let stall = warnings.iter().find(|w| w.check == ChallengerCheck::StallDetection);
        assert!(stall.is_some(), "Should detect stall at iteration 5");
        assert_eq!(stall.unwrap().severity, Severity::Medium, "Iteration 5 should be Medium");
    }

    #[test]
    fn challenger_stall_critical_at_8() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![
            tc("read", 1), tc("grep", 2), tc("read", 3), tc("glob", 4),
            tc("read", 5), tc("grep", 6), tc("read", 7), tc("glob", 8),
        ];
        let warnings = c.check(&calls, None, 8);
        let stall = warnings.iter().find(|w| w.check == ChallengerCheck::StallDetection);
        assert!(stall.is_some(), "Should detect stall at iteration 8");
        assert_eq!(stall.unwrap().severity, Severity::Critical, "Iteration 8 should be Critical");
        assert!(stall.unwrap().message.contains("Produce output NOW"), "Should contain termination warning");
    }

    #[test]
    fn challenger_stall_kill_at_10() {
        let mut c = Challenger::new(Role::Agent);
        let calls = vec![
            tc("read", 1), tc("grep", 2), tc("read", 3), tc("glob", 4),
            tc("read", 5), tc("grep", 6), tc("read", 7), tc("glob", 8),
            tc("read", 9), tc("grep", 10),
        ];
        let warnings = c.check(&calls, None, 10);
        let stall = warnings.iter().find(|w| w.check == ChallengerCheck::StallDetection);
        assert!(stall.is_some(), "Should detect stall at iteration 10");
        assert_eq!(stall.unwrap().severity, Severity::Critical, "Iteration 10 should be Critical");
        assert!(stall.unwrap().message.contains("STALL KILL"), "Should contain STALL KILL");
    }

    #[test]
    fn challenger_should_kill_stall_after_consecutive_criticals() {
        let mut c = Challenger::new(Role::Agent);

        // First check at iteration 8 — Critical but not yet kill
        let calls_8: Vec<ChallengerToolCall> = (1..=8).map(|i| tc("read", i)).collect();
        let _ = c.check(&calls_8, None, 8);
        assert!(!c.should_kill_stall(), "Should not kill after 1 critical");

        // Second check at iteration 9 — 2 consecutive criticals -> kill
        let calls_9: Vec<ChallengerToolCall> = (1..=9).map(|i| tc("read", i)).collect();
        let _ = c.check(&calls_9, None, 9);
        assert!(c.should_kill_stall(), "Should kill after 2 consecutive criticals");
    }

    #[test]
    fn challenger_stall_kill_resets_on_productive() {
        let mut c = Challenger::new(Role::Agent);

        // Build up stall
        let calls_8: Vec<ChallengerToolCall> = (1..=8).map(|i| tc("read", i)).collect();
        let _ = c.check(&calls_8, None, 8);
        assert!(!c.should_kill_stall());

        // Productive tool resets the counter
        let mut calls_9 = calls_8.clone();
        calls_9.push(tc("bash", 9));
        let _ = c.check(&calls_9, None, 9);
        assert!(!c.should_kill_stall(), "Productive tool should reset kill counter");
    }

    // -- A1: disable() hardening --

    #[test]
    fn disable_is_pub_crate_and_logs() {
        // Verify that disable() is callable from within the crate (we are in crate tests)
        // and that it actually disables the challenger.
        let mut c = Challenger::new(Role::Agent);
        assert!(c.enabled); // starts enabled

        c.disable();
        assert!(!c.enabled); // now disabled

        // Verify disabled challenger produces no warnings
        let calls = vec![tc("read", 1)];
        let warnings = c.check(&calls, Some("Done!"), 1);
        assert!(warnings.is_empty(), "Disabled challenger should produce no warnings");

        // Re-enable should work
        c.enable();
        assert!(c.enabled);
        let warnings2 = c.check(&calls, Some("Done!"), 1);
        assert!(!warnings2.is_empty(), "Re-enabled challenger should produce warnings");
    }

    #[test]
    fn enable_remains_public() {
        // enable() should always be callable — re-enabling is safe
        let mut c = Challenger::new(Role::Agent);
        c.disable();
        c.enable();
        assert!(c.enabled);
    }
}

// ── Memory Integration Tests (feature-gated) ────────────────────────────────
//
// These tests only compile and run when `cargo test -p codex-redteam --features memory`

#[cfg(test)]
#[cfg(feature = "memory")]
mod memory_tests {
    use super::*;

    async fn test_store() -> cortex_memory::MemoryStore {
        cortex_memory::MemoryStore::new(":memory:").await.unwrap()
    }

    fn test_memory(title: &str, content: &str) -> cortex_memory::NewMemory {
        cortex_memory::NewMemory {
            mind_id: "cortex-primary".into(),
            role: "primary".into(),
            vertical: Some("testing".into()),
            category: cortex_memory::MemoryCategory::Learning,
            title: title.into(),
            content: content.into(),
            evidence: vec!["test-evidence".into()],
            tier: cortex_memory::MemoryTier::Working,
            session_id: Some("sess-test".into()),
            task_id: None,
        }
    }

    #[tokio::test]
    async fn check_memory_contradictions_finds_conflicts() {
        let store = test_store().await;

        // Create two memories that contradict each other
        let mem_a = store.store(test_memory(
            "Feature X uses SQLite",
            "Feature X stores data in SQLite for fast local access."
        )).await.unwrap();

        let mem_b = store.store(test_memory(
            "Feature X uses Postgres",
            "Feature X requires Postgres for production reliability."
        )).await.unwrap();

        // Flag the conflict
        store.flag_conflict(&mem_a, &mem_b).await.unwrap();

        // Now verify a claim about Feature X
        let rt = RedTeamProtocol::new();
        let claim = CompletionClaim {
            task_id: "test-mem-1".into(),
            mind_id: "coder-1".into(),
            description: "Feature X uses SQLite".into(),
            result_summary: "Implemented Feature X with SQLite storage".into(),
            evidence: vec![Evidence {
                evidence_type: EvidenceType::TestResult,
                content: "Tests pass".into(),
                freshness: Freshness::Current,
            }],
            claimed_at: Utc::now(),
        };

        let contradictions = rt
            .check_memory_contradictions(Some(&store), &claim)
            .await;

        assert!(!contradictions.is_empty(),
            "Should find contradictions when Conflicts edges exist");
        assert!(contradictions[0].contains("MEMORY CONTRADICTION"),
            "Contradiction message should be clearly labeled");
    }

    #[tokio::test]
    async fn check_memory_contradictions_empty_when_no_conflicts() {
        let store = test_store().await;

        // Create a memory with no conflicts
        store.store(test_memory(
            "Module Y is stable",
            "Module Y has been stable for 5 sessions."
        )).await.unwrap();

        let rt = RedTeamProtocol::new();
        let claim = CompletionClaim {
            task_id: "test-mem-2".into(),
            mind_id: "coder-1".into(),
            description: "Module Y is stable".into(),
            result_summary: "Verified Module Y stability".into(),
            evidence: vec![Evidence {
                evidence_type: EvidenceType::TestResult,
                content: "Tests pass".into(),
                freshness: Freshness::Current,
            }],
            claimed_at: Utc::now(),
        };

        let contradictions = rt
            .check_memory_contradictions(Some(&store), &claim)
            .await;

        assert!(contradictions.is_empty(),
            "Should find no contradictions when no Conflicts edges exist");
    }

    #[tokio::test]
    async fn check_memory_contradictions_none_store_returns_empty() {
        let rt = RedTeamProtocol::new();
        let claim = CompletionClaim {
            task_id: "test-mem-3".into(),
            mind_id: "coder-1".into(),
            description: "Anything".into(),
            result_summary: "Done".into(),
            evidence: vec![],
            claimed_at: Utc::now(),
        };

        let contradictions = rt
            .check_memory_contradictions(None, &claim)
            .await;

        assert!(contradictions.is_empty(),
            "Should gracefully return empty when no store is provided");
    }

    #[tokio::test]
    async fn verify_with_memory_downgrades_approval_on_contradiction() {
        let store = test_store().await;

        // Create contradicting memories
        let mem_a = store.store(test_memory(
            "Config uses YAML",
            "The config system uses YAML format exclusively."
        )).await.unwrap();

        let mem_b = store.store(test_memory(
            "Config uses TOML",
            "The config system was migrated to TOML format."
        )).await.unwrap();

        store.flag_conflict(&mem_a, &mem_b).await.unwrap();

        let rt = RedTeamProtocol::new();
        let claim = CompletionClaim {
            task_id: "test-mem-4".into(),
            mind_id: "coder-1".into(),
            description: "Config uses YAML".into(),
            result_summary: "Config system working with YAML".into(),
            evidence: vec![Evidence {
                evidence_type: EvidenceType::TestResult,
                content: "All config tests pass".into(),
                freshness: Freshness::Current,
            }],
            claimed_at: Utc::now(),
        };

        // Without memory: should be Approved (has current evidence)
        let base = rt.verify(&claim);
        assert!(matches!(base, RedTeamVerdict::Approved { .. }),
            "Base verdict should be Approved");

        // With memory: should be downgraded to Challenged due to contradiction
        let verdict = rt.verify_with_memory(&claim, Some(&store)).await;
        assert!(matches!(verdict, RedTeamVerdict::Challenged { .. }),
            "Memory contradictions should downgrade Approved to Challenged");
    }
}
