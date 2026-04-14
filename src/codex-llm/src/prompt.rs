//! Prompt construction — builds the system prompt and context for each mind role.
//!
//! ## AGENTS.md Injection
//!
//! Cortex follows Codex's AGENTS.md pattern: directory-scoped instruction files
//! that customize a mind's behavior. When `agents_dir` is set, the PromptBuilder
//! searches for:
//!
//! ```text
//! {agents_dir}/primary/AGENTS.md        → Primary mind
//! {agents_dir}/team-leads/{vertical}/AGENTS.md → Team Lead for vertical
//! {agents_dir}/agents/{agent_type}/AGENTS.md   → Agent type
//! ```
//!
//! If found, the file content is injected after the role-specific system prompt.

use std::path::{Path, PathBuf};
use codex_roles::Role;
use tracing::debug;

use crate::ollama::ChatMessage;

/// Builds prompts tailored to a mind's role and context.
pub struct PromptBuilder {
    role: Role,
    mind_id: String,
    vertical: Option<String>,
    extra_context: Vec<String>,
    agents_dir: Option<PathBuf>,
    agents_md_content: Option<String>,
}

impl PromptBuilder {
    pub fn new(role: Role, mind_id: impl Into<String>) -> Self {
        Self {
            role,
            mind_id: mind_id.into(),
            vertical: None,
            extra_context: Vec::new(),
            agents_dir: None,
            agents_md_content: None,
        }
    }

    pub fn vertical(mut self, vertical: impl Into<String>) -> Self {
        self.vertical = Some(vertical.into());
        self
    }

    pub fn add_context(mut self, context: impl Into<String>) -> Self {
        self.extra_context.push(context.into());
        self
    }

    /// Set the directory to search for AGENTS.md files.
    ///
    /// The builder will look for role-specific AGENTS.md files:
    /// - Primary: `{dir}/primary/AGENTS.md`
    /// - TeamLead: `{dir}/team-leads/{vertical}/AGENTS.md`
    /// - Agent: `{dir}/agents/{mind_id}/AGENTS.md`
    pub fn agents_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        // Attempt to load the AGENTS.md for this role
        self.agents_md_content = self.find_agents_md(&dir);
        self.agents_dir = Some(dir);
        self
    }

    /// Directly inject AGENTS.md content (when loaded from elsewhere).
    pub fn agents_md(mut self, content: impl Into<String>) -> Self {
        self.agents_md_content = Some(content.into());
        self
    }

    /// Search for the AGENTS.md file matching this role.
    fn find_agents_md(&self, dir: &Path) -> Option<String> {
        let candidates = match self.role {
            Role::Primary => vec![
                dir.join("primary").join("AGENTS.md"),
                dir.join("primary.agents.md"),
            ],
            Role::TeamLead => {
                let vert = self.vertical.as_deref().unwrap_or("general");
                vec![
                    dir.join("team-leads").join(vert).join("AGENTS.md"),
                    dir.join(format!("team-lead-{vert}.agents.md")),
                ]
            }
            Role::Agent => {
                vec![
                    dir.join("agents").join(&self.mind_id).join("AGENTS.md"),
                    dir.join(format!("agent-{}.agents.md", self.mind_id)),
                ]
            }
        };

        for path in &candidates {
            if let Ok(content) = std::fs::read_to_string(path) {
                debug!(path = %path.display(), role = ?self.role, "Loaded AGENTS.md");
                return Some(content);
            }
        }

        debug!(role = ?self.role, "No AGENTS.md found");
        None
    }

    /// Build the system prompt for this mind.
    pub fn system_prompt(&self) -> String {
        let role_section = match self.role {
            Role::Primary => PRIMARY_SYSTEM.to_string(),
            Role::TeamLead => {
                let vert = self.vertical.as_deref().unwrap_or("general");
                TEAM_LEAD_SYSTEM.replace("{VERTICAL}", vert)
            }
            Role::Agent => AGENT_SYSTEM.to_string(),
        };

        let mut prompt = format!(
            "# Cortex Mind: {}\n\
             ## Role: {:?}\n\n\
             {TOOL_USE_PREAMBLE}\n\n\
             {role_section}",
            self.mind_id, self.role
        );

        // Inject AGENTS.md content if available
        if let Some(ref agents_md) = self.agents_md_content {
            prompt.push_str("\n\n## Instructions (AGENTS.md)\n");
            prompt.push_str(agents_md);
        }

        for ctx in &self.extra_context {
            prompt.push_str("\n\n## Additional Context\n");
            prompt.push_str(ctx);
        }

        prompt
    }

    /// Build the initial message sequence (system + user task).
    pub fn build_messages(&self, task: &str) -> Vec<ChatMessage> {
        vec![
            ChatMessage::system(self.system_prompt()),
            ChatMessage::user(task.to_string()),
        ]
    }
}

/// Tool calling preamble injected into ALL system prompts.
/// Placed early so the model internalizes tool-calling format before seeing schemas.
const TOOL_USE_PREAMBLE: &str = "\
## How to Use Tools (MANDATORY)

You have tools available as structured function calls. The system handles execution.

RULES:
1. When you want to perform an action, EMIT A TOOL CALL. Do not describe it in prose.
2. WRONG: \"I would use the read tool to read /tmp/foo.txt\"
   RIGHT: (emit a read tool call with file_path=\"/tmp/foo.txt\")
3. You may call multiple tools in sequence. After each tool result, decide your next action.
4. Do NOT claim your task is complete until you have evidence from tool results.
5. If a tool returns an error, read the error message, fix the issue, and retry — do not give up after one attempt.
6. Use EXACT parameter names as shown in tool descriptions. Do not invent aliases.

## Completion Standard

Do NOT say \"done\", \"complete\", or \"finished\" unless ALL of the following are true:
- You have called at least one tool and received its result
- The tool result confirms your action succeeded
- You have addressed the full scope of the task, not just one sub-part

If the task has multiple steps, complete ALL steps before claiming completion.";

const PRIMARY_SYSTEM: &str = "\
You are the PRIMARY mind — Conductor of Conductors.

## Identity
You orchestrate team leads. You NEVER execute work directly.
You spawn team leads, delegate tasks, synthesize results, and make decisions.

## Available Actions
- Spawn team leads for specific verticals
- Delegate tasks to team leads
- Check status of active minds
- Send messages to other minds
- Search memory for relevant context

## Constraints
- NEVER use bash, file_read, file_write, or any execution tools
- ONLY use coordination tools (mind_spawn_team_lead, mind_delegate, mind_status, send_message)
- Route ALL work through team leads — no exceptions
- When genuinely uncertain about routing, ask for clarification

## Decision Making
For each input:
1. Search memory for relevant context
2. Determine which vertical owns this work
3. Spawn or delegate to the appropriate team lead
4. Monitor progress and synthesize results
";

const TEAM_LEAD_SYSTEM: &str = "\
You are a TEAM LEAD mind for the {VERTICAL} vertical.

## Identity
You coordinate agents within your vertical. You delegate work, synthesize results,
and report outcomes to Primary.

## Tool Use — CRITICAL
You have tools available as function calls. When you want to perform an action,
call the appropriate tool function. DO NOT write code blocks or pseudo-code.
The system will execute your tool calls and return results.

For example, to spawn an agent, call the `spawn_agent` tool directly.
To search memory, call `memory_search`. To delegate work, call `delegate_to_agent`.

ALWAYS use tool calls for actions. NEVER describe what you would do — just do it.

## Available Tools
- `spawn_agent` — Create a new agent mind for a specific task
- `delegate_to_agent` — Send a task to an existing agent
- `shutdown_agent` — Shut down an agent when its work is complete
- `memory_search` — Search for relevant context in memory
- `memory_write` — Save important findings to memory

## Constraints
- NEVER use bash, file tools, or web tools directly
- Delegate all execution to agents via tool calls
- Synthesize agent results into concise summaries for Primary
- Maintain team scratchpad with current state

## Workflow
1. Receive task from Primary
2. Break into sub-tasks for agents
3. Call `spawn_agent` for each needed agent
4. Call `delegate_to_agent` to assign work
5. Synthesize results and report back to Primary
";

const AGENT_SYSTEM: &str = "\
You are an AGENT mind — the hands of the civilization.

## Identity
You execute real work: read files, write code, run commands, search the web.
You have full tool access within your sandbox.

## Available Actions
- All standard tools: bash, read, write, glob, grep
- Memory search and write
- Report results back to your team lead

## Constraints
- Work within your workspace sandbox — no escaping to system directories
- Complete your assigned task thoroughly
- Provide evidence for your findings
- Extract learnings from your work
- Report results and learnings when done

## Workflow
1. Receive task from team lead
2. Search memory for relevant prior work
3. Execute the task using available tools
4. Verify your work
5. Report results with evidence and learnings
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_prompt() {
        let builder = PromptBuilder::new(Role::Primary, "cortex-primary");
        let prompt = builder.system_prompt();
        assert!(prompt.contains("PRIMARY mind"));
        assert!(prompt.contains("Conductor of Conductors"));
        assert!(prompt.contains("cortex-primary"));
    }

    #[test]
    fn team_lead_prompt_with_vertical() {
        let builder = PromptBuilder::new(Role::TeamLead, "research-lead")
            .vertical("research");
        let prompt = builder.system_prompt();
        assert!(prompt.contains("TEAM LEAD"));
        assert!(prompt.contains("research"));
    }

    #[test]
    fn agent_prompt() {
        let builder = PromptBuilder::new(Role::Agent, "coder-1");
        let prompt = builder.system_prompt();
        assert!(prompt.contains("AGENT mind"));
        assert!(prompt.contains("hands of the civilization"));
    }

    #[test]
    fn extra_context_appended() {
        let builder = PromptBuilder::new(Role::Agent, "researcher-1")
            .add_context("Previous findings: XYZ");
        let prompt = builder.system_prompt();
        assert!(prompt.contains("Previous findings: XYZ"));
    }

    #[test]
    fn build_messages() {
        let builder = PromptBuilder::new(Role::Agent, "coder-1");
        let msgs = builder.build_messages("Write a hello world program");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "system");
        assert_eq!(msgs[1].role, "user");
        assert!(msgs[1].content.as_ref().unwrap().contains("hello world"));
    }

    #[test]
    fn agents_md_injection() {
        let builder = PromptBuilder::new(Role::Agent, "coder-1")
            .agents_md("You specialize in Rust systems programming.");
        let prompt = builder.system_prompt();
        assert!(prompt.contains("Instructions (AGENTS.md)"));
        assert!(prompt.contains("Rust systems programming"));
    }

    #[test]
    fn tool_use_preamble_in_all_roles() {
        for (role, id) in [
            (Role::Primary, "primary"),
            (Role::TeamLead, "lead"),
            (Role::Agent, "agent"),
        ] {
            let builder = PromptBuilder::new(role, id);
            let prompt = builder.system_prompt();
            assert!(
                prompt.contains("How to Use Tools (MANDATORY)"),
                "TOOL_USE_PREAMBLE missing for {:?}",
                role
            );
            assert!(
                prompt.contains("EMIT A TOOL CALL"),
                "Tool call instruction missing for {:?}",
                role
            );
            assert!(
                prompt.contains("Completion Standard"),
                "Completion standard missing for {:?}",
                role
            );
        }
    }

    #[test]
    fn agents_md_before_extra_context() {
        let builder = PromptBuilder::new(Role::Agent, "coder-1")
            .agents_md("AGENTS.md content first")
            .add_context("Extra context second");
        let prompt = builder.system_prompt();
        let agents_pos = prompt.find("AGENTS.md content first").unwrap();
        let extra_pos = prompt.find("Extra context second").unwrap();
        assert!(agents_pos < extra_pos, "AGENTS.md should come before extra context");
    }

    #[test]
    fn agents_dir_no_files() {
        // Point to a nonexistent directory — should not crash, just no injection
        let builder = PromptBuilder::new(Role::Primary, "primary")
            .agents_dir("/tmp/nonexistent-cortex-test-dir");
        let prompt = builder.system_prompt();
        assert!(!prompt.contains("Instructions (AGENTS.md)"));
        assert!(prompt.contains("PRIMARY mind"));
    }

    #[test]
    fn agents_dir_loads_file() {
        // Create a temp directory with an AGENTS.md
        let dir = std::env::temp_dir().join("cortex-agents-test");
        let primary_dir = dir.join("primary");
        std::fs::create_dir_all(&primary_dir).unwrap();
        std::fs::write(
            primary_dir.join("AGENTS.md"),
            "You are the conductor. Never write code directly.",
        ).unwrap();

        let builder = PromptBuilder::new(Role::Primary, "cortex-primary")
            .agents_dir(&dir);
        let prompt = builder.system_prompt();
        assert!(prompt.contains("You are the conductor"));
        assert!(prompt.contains("Never write code directly"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn agents_dir_team_lead_vertical() {
        let dir = std::env::temp_dir().join("cortex-agents-tl-test");
        let tl_dir = dir.join("team-leads").join("research");
        std::fs::create_dir_all(&tl_dir).unwrap();
        std::fs::write(
            tl_dir.join("AGENTS.md"),
            "Focus on multi-angle research with competing hypotheses.",
        ).unwrap();

        let builder = PromptBuilder::new(Role::TeamLead, "research-lead")
            .vertical("research")
            .agents_dir(&dir);
        let prompt = builder.system_prompt();
        assert!(prompt.contains("multi-angle research"));
        assert!(prompt.contains("competing hypotheses"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
