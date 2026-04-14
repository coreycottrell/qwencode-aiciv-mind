//! The core Mind struct — identity, memory, planning, LLM, fitness.
//!
//! Phase 1a: isolated mind with local memory, Ollama, scratchpad, fitness.
//! Phase 1b: SuiteClient integration, Hub dual-write, Envelope signing.

use cortex_memory::{MemoryQuery, NewMemory as CortexNewMemory};
use crate::identity::{Manifest, Role};
use crate::memory::MindMemory;
use crate::scratchpad::Scratchpad;
use crate::fitness::FitnessTracker;
use crate::llm::{OllamaClient, LlmError};
use crate::delegation::DelegationRules;
use crate::planning::PlanningGate;

/// Result of executing a task.
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub content: String,
    pub success: bool,
    pub memory_id: Option<String>,
    pub fitness_score: f64,
}

/// The persistent AI mind.
pub struct Mind {
    pub manifest: Manifest,
    pub memory: MindMemory,
    pub scratchpad: Scratchpad,
    pub fitness: FitnessTracker,
    pub llm: OllamaClient,
    pub delegation: DelegationRules,
    pub root_dir: std::path::PathBuf,
}

impl Mind {
    /// Create a new mind from a manifest.
    pub async fn new(manifest: Manifest, root_dir: &std::path::Path) -> anyhow::Result<Self> {
        let root_dir = root_dir.to_path_buf();

        // Initialize persistence layers
        let db_path = root_dir
            .join("data")
            .join("memory")
            .join(format!("{}.db", manifest.identity));
        std::fs::create_dir_all(db_path.parent().unwrap()).ok();
        let memory = MindMemory::new(db_path.to_str().unwrap()).await?;

        let scratchpad = Scratchpad::new(&root_dir, &manifest.identity);
        let fitness = FitnessTracker::new(&root_dir, &manifest.identity);
        let llm = OllamaClient::from_env();
        let delegation = DelegationRules::new(manifest.role, &manifest.vertical);

        Ok(Self {
            manifest,
            memory,
            scratchpad,
            fitness,
            llm,
            delegation,
            root_dir,
        })
    }

    /// The core think loop — receive task, plan, execute, verify, persist.
    pub async fn think(&self, task: &str) -> Result<TaskResult, LlmError> {
        // Step 1: Search memory for prior similar tasks
        let prior_results = self.memory
            .search(&MemoryQuery {
                text: Some(task.chars().take(80).collect()),
                mind_id: None,
                category: None,
                tier: None,
                min_depth: Some(0.2),
                limit: Some(3),
            })
            .await
            .unwrap_or_default();

        let has_prior = !prior_results.is_empty();
        let best_relevance = prior_results
            .first()
            .map(|r| r.relevance)
            .unwrap_or(0.0);

        // Step 2: Planning gate
        let plan = PlanningGate::assess(task, has_prior, best_relevance).await;

        // If trivial and we have a prior match, replay
        if plan.complexity == crate::planning::TaskComplexity::Trivial
            && plan.prior_result.is_some()
        {
            let content = plan.prior_result.unwrap();
            return Ok(TaskResult {
                content: content.clone(),
                success: true,
                memory_id: None,
                fitness_score: 0.5,
            });
        }

        // Step 3: Build prompt
        let scratchpad_recent = self.scratchpad.read_recent(500);
        let system_prompt = self.build_system_prompt();
        let mut user_prompt = format!("Task: {task}");

        // Add prior memory context
        if !prior_results.is_empty() {
            user_prompt.push_str("\n\nPrior relevant findings:");
            for r in &prior_results {
                user_prompt.push_str(&format!(
                    "\n[relevance {:.2}] {}: {}",
                    r.relevance, r.memory.title, 
                    r.memory.content.chars().take(300).collect::<String>()
                ));
            }
        }

        // Add scratchpad context
        if !scratchpad_recent.is_empty() {
            user_prompt.push_str(&format!("\n\nRecent scratchpad:\n{scratchpad_recent}"));
        }

        // Step 4: Execute via LLM
        let llm_response = self.llm.chat(&system_prompt, &user_prompt).await?;

        // Step 5: Write memory
        let memory_id = self.persist_result(task, &llm_response.content).await;

        // Step 6: Update scratchpad
        self.scratchpad.append(&format!(
            "Task: {}\nResult: {}",
            task.chars().take(80).collect::<String>(),
            llm_response.content.chars().take(200).collect::<String>()
        ));

        // Step 7: Record fitness
        let had_errors = llm_response.retries > 0;
        let memory_written = memory_id.is_some();
        self.fitness.record(task, true, had_errors, &llm_response.content, memory_written);

        let score = self.fitness.average();

        Ok(TaskResult {
            content: llm_response.content,
            success: true,
            memory_id,
            fitness_score: score,
        })
    }

    fn build_system_prompt(&self) -> String {
        let principles_str = if self.manifest.principles.is_empty() {
            "- Memory IS architecture\n- System > symptom\n- Go slow to go fast".to_string()
        } else {
            self.manifest.principles.iter().map(|p| format!("- {p}")).collect::<Vec<_>>().join("\n")
        };

        let anti_patterns_str = if self.manifest.anti_patterns.is_empty() {
            "None yet".to_string()
        } else {
            self.manifest.anti_patterns.iter().map(|p| format!("- {p}")).collect::<Vec<_>>().join("\n")
        };

        let tool_restriction = match self.manifest.role {
            Role::Primary => "You can ONLY coordinate and delegate. You CANNOT execute tools directly.",
            Role::TeamLead => "You can ONLY delegate to Agents in your vertical. You execute via delegation.",
            Role::Agent => "You execute tools and do the actual work. You CANNOT spawn children or delegate.",
        };

        format!(
            "You are {}, a {} mind.\n\
            Vertical: {}\n\
            Specialty: {}\n\
            Growth stage: {}\n\
            Session count: {}\n\n\
            Principles:\n{}\n\n\
            Anti-patterns (things NOT to do):\n{}\n\n\
            Rules:\n{}\n\n\
            Be concise. Lead with outcomes.",
            self.manifest.identity,
            self.manifest.role,
            self.manifest.vertical,
            self.manifest.specialty.as_deref().unwrap_or("general"),
            self.manifest.growth_stage,
            self.manifest.session_count,
            principles_str,
            anti_patterns_str,
            tool_restriction,
        )
    }

    async fn persist_result(&self, task: &str, result: &str) -> Option<String> {
        let mem = CortexNewMemory {
            mind_id: self.manifest.identity.clone(),
            role: self.manifest.role.to_string(),
            vertical: Some(self.manifest.vertical.clone()),
            category: cortex_memory::MemoryCategory::Learning,
            title: task.chars().take(80).collect(),
            content: result.to_string(),
            evidence: vec![format!("task-response-{}", chrono::Utc::now().timestamp())],
            tier: cortex_memory::MemoryTier::Working,
            session_id: None,
            task_id: None,
        };

        self.memory.store(mem).await.ok()
    }

    /// Increment session count and save manifest.
    pub fn end_session(&mut self) {
        self.manifest.increment_session();
        let manifest_path = self.root_dir.join("manifests").join(format!("{}.json", self.manifest.identity));
        if let Err(e) = self.manifest.save(&manifest_path) {
            tracing::error!(error = %e, "Failed to save manifest");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Manifest, Role};
    use tempfile::TempDir;

    #[tokio::test]
    async fn mind_creates_persistence_dirs() {
        let dir = TempDir::new().unwrap();
        let manifest = Manifest::new("test-mind", Role::Agent, "test");
        let mind = Mind::new(manifest, dir.path()).await.unwrap();

        assert!(dir.path().join("data/memory/test-mind.db").exists());
        assert!(dir.path().join("scratchpads/test-mind").exists());
        // Fitness file is created lazily on first record
        let _ = mind;
    }

    #[tokio::test]
    async fn mind_system_prompt_includes_identity() {
        let dir = TempDir::new().unwrap();
        let mut manifest = Manifest::new("qwen-lead", Role::TeamLead, "qwen");
        manifest.principles.push("Test principle".to_string());
        manifest.anti_patterns.push("Bad pattern".to_string());
        let mind = Mind::new(manifest, dir.path()).await.unwrap();

        let prompt = mind.build_system_prompt();
        assert!(prompt.contains("qwen-lead"));
        assert!(prompt.contains("team_lead"));
        assert!(prompt.contains("Test principle"));
        assert!(prompt.contains("Bad pattern"));
        assert!(prompt.contains("qwen"));
    }
}
