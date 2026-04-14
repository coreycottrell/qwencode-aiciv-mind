//! # Boot Context — Session Continuity for Cortex Minds
//!
//! When a mind starts up, it needs to know:
//! - Who am I? (identity from AGENTS.md / config)
//! - What was I doing? (last handoff)
//! - What do I know? (recent memories)
//! - What's on my desk? (scratchpad)
//!
//! BootContext gathers all of this and injects it into the first system prompt.

use codex_memory::MemoryStore;
use codex_roles::Role;
use std::path::{Path, PathBuf};
use tracing::info;

/// Truncate a string to at most `max` bytes, landing on a valid UTF-8 char boundary.
fn safe_truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Gathered boot context for a mind waking up.
#[derive(Debug, Clone, Default)]
pub struct BootContext {
    /// Identity text from AGENTS.md (role-specific instructions).
    pub identity: Option<String>,
    /// Last session handoff (what was I doing when I stopped?).
    pub last_handoff: Option<String>,
    /// Current scratchpad content.
    pub scratchpad: Option<String>,
    /// Recent memories (summaries).
    pub recent_memories: Vec<String>,
}

impl BootContext {
    /// Load boot context for a mind.
    ///
    /// Searches for:
    /// - `agents/{role}/AGENTS.md` or `agents/AGENTS.md`
    /// - `data/handoffs/{mind_id}/` (most recent file)
    /// - `data/scratchpad/{mind_id}.md`
    /// - Recent memories from MemoryStore
    pub async fn load(
        project_root: &Path,
        mind_id: &str,
        role: Role,
        memory: Option<&MemoryStore>,
    ) -> Self {
        let mut ctx = BootContext::default();

        // 1. Identity from AGENTS.md
        ctx.identity = load_agents_md(project_root, role);

        // 2. Last handoff
        ctx.last_handoff = load_latest_handoff(project_root, mind_id);

        // 3. Scratchpad
        ctx.scratchpad = load_scratchpad(project_root, mind_id);

        // 4. Recent memories
        if let Some(store) = memory {
            ctx.recent_memories = load_recent_memories(store, mind_id).await;
        }

        let parts = [
            ctx.identity.is_some(),
            ctx.last_handoff.is_some(),
            ctx.scratchpad.is_some(),
            !ctx.recent_memories.is_empty(),
        ];
        info!(
            mind_id = mind_id,
            identity = parts[0],
            handoff = parts[1],
            scratchpad = parts[2],
            memories = ctx.recent_memories.len(),
            "Boot context loaded"
        );

        ctx
    }

    /// Format boot context as a system prompt injection.
    pub fn to_system_prompt(&self) -> String {
        let mut sections = Vec::new();

        if let Some(ref identity) = self.identity {
            sections.push(format!("## Identity\n\n{identity}"));
        }

        if let Some(ref handoff) = self.last_handoff {
            sections.push(format!("## Last Session Handoff\n\n{handoff}"));
        }

        if let Some(ref scratchpad) = self.scratchpad {
            sections.push(format!("## Current Scratchpad\n\n{scratchpad}"));
        }

        if !self.recent_memories.is_empty() {
            let mem_text = self.recent_memories.join("\n");
            sections.push(format!("## Recent Memories\n\n{mem_text}"));
        }

        if sections.is_empty() {
            return String::new();
        }

        format!("# Boot Context\n\n{}", sections.join("\n\n---\n\n"))
    }
}

/// Load AGENTS.md for the given role.
fn load_agents_md(project_root: &Path, role: Role) -> Option<String> {
    let role_dir = match role {
        Role::Primary => "primary",
        Role::TeamLead => "team-lead",
        Role::Agent => "agent",
    };

    // Try role-specific first, then generic
    let candidates = [
        project_root.join("agents").join(role_dir).join("AGENTS.md"),
        project_root.join("agents").join("AGENTS.md"),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            info!(path = %path.display(), "Loaded AGENTS.md");
            return Some(content);
        }
    }
    None
}

/// Load the most recent handoff file for a mind.
fn load_latest_handoff(project_root: &Path, mind_id: &str) -> Option<String> {
    let handoff_dir = project_root.join("data").join("handoffs").join(mind_id);
    if !handoff_dir.is_dir() {
        return None;
    }

    let mut entries: Vec<_> = std::fs::read_dir(&handoff_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .collect();

    // Sort by filename (timestamps sort lexicographically)
    entries.sort_by_key(|e| e.file_name());

    if let Some(latest) = entries.last() {
        if let Ok(content) = std::fs::read_to_string(latest.path()) {
            info!(path = %latest.path().display(), "Loaded handoff");
            return Some(content);
        }
    }
    None
}

/// Load the scratchpad for a mind.
fn load_scratchpad(project_root: &Path, mind_id: &str) -> Option<String> {
    let today = chrono::Utc::now().format("%Y-%m-%d");
    let path = project_root
        .join("data")
        .join("scratchpad")
        .join(format!("{mind_id}-{today}.md"));
    if let Ok(content) = std::fs::read_to_string(&path) {
        info!(path = %path.display(), lines = content.lines().count(), "Loaded scratchpad");
        Some(content)
    } else {
        None
    }
}

/// Load recent memories from the store.
async fn load_recent_memories(store: &MemoryStore, mind_id: &str) -> Vec<String> {
    use codex_memory::MemoryQuery;

    let query = MemoryQuery {
        text: Some(mind_id.to_string()),
        limit: Some(5),
        ..Default::default()
    };

    match store.search(&query).await {
        Ok(results) => {
            results.iter().map(|r| {
                format!("- **{}** (depth: {:.2}): {}", r.memory.title, r.memory.depth_score, r.memory.content)
            }).collect()
        }
        Err(_) => Vec::new(),
    }
}

/// Roll over stale scratchpads at daemon boot.
///
/// Scans `scratchpad_dir` for files that DON'T match today's date,
/// moves them to `scratchpad_dir/archive/`. Returns the count archived.
pub fn rollover_scratchpads(scratchpad_dir: &Path) -> std::io::Result<usize> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let archive_dir = scratchpad_dir.join("archive");

    let entries: Vec<_> = std::fs::read_dir(scratchpad_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".md") && !name.contains(&today) && e.path().is_file()
        })
        .collect();

    if entries.is_empty() {
        return Ok(0);
    }

    std::fs::create_dir_all(&archive_dir)?;
    let mut count = 0;
    for entry in &entries {
        let dest = archive_dir.join(entry.file_name());
        if std::fs::rename(entry.path(), &dest).is_ok() {
            count += 1;
        }
    }

    if count > 0 {
        info!(archived = count, "Scratchpad daily rollover — archived stale scratchpads");
    }
    Ok(count)
}

/// Write a session handoff to disk.
///
/// Called when a ThinkLoop session completes — captures what the mind was doing
/// so the next session can pick up where this one left off.
pub fn write_handoff(
    project_root: &Path,
    mind_id: &str,
    session_id: &str,
    task: &str,
    response: &str,
    iterations: u32,
    tool_calls: u32,
    completed: bool,
) -> std::io::Result<PathBuf> {
    let handoff_dir = project_root.join("data").join("handoffs").join(mind_id);
    std::fs::create_dir_all(&handoff_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let filename = format!("{timestamp}.json");
    let path = handoff_dir.join(&filename);

    let handoff = serde_json::json!({
        "session_id": session_id,
        "mind_id": mind_id,
        "completed_at": chrono::Utc::now().to_rfc3339(),
        "task": task,
        "response_summary": if response.len() > 500 {
            format!("{}...", safe_truncate(response, 500))
        } else {
            response.to_string()
        },
        "iterations": iterations,
        "tool_calls": tool_calls,
        "completed": completed,
    });

    std::fs::write(&path, serde_json::to_string_pretty(&handoff).unwrap())?;
    info!(path = %path.display(), "Handoff written");
    Ok(path)
}

/// Record a fitness outcome to the fitness log.
pub fn record_fitness(
    project_root: &Path,
    mind_id: &str,
    outcome: &codex_fitness::TaskOutcome,
) -> std::io::Result<()> {
    let fitness_dir = project_root.join("data").join("fitness");
    std::fs::create_dir_all(&fitness_dir)?;

    let path = fitness_dir.join(format!("{mind_id}.jsonl"));

    let fitness = codex_fitness::compute_fitness(outcome);
    let record = serde_json::json!({
        "task_id": outcome.task_id,
        "mind_id": outcome.mind_id,
        "role": outcome.role,
        "completed_at": outcome.completed_at.to_rfc3339(),
        "composite_score": fitness.composite(),
        "fitness": fitness,
    });

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{}", serde_json::to_string(&record).unwrap())?;
    info!(path = %path.display(), score = fitness.composite(), "Fitness recorded");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn boot_context_empty() {
        let ctx = BootContext::default();
        assert!(ctx.to_system_prompt().is_empty());
    }

    #[test]
    fn boot_context_with_identity() {
        let ctx = BootContext {
            identity: Some("You are Cortex, a fractal mind.".into()),
            ..Default::default()
        };
        let prompt = ctx.to_system_prompt();
        assert!(prompt.contains("# Boot Context"));
        assert!(prompt.contains("## Identity"));
        assert!(prompt.contains("fractal mind"));
    }

    #[test]
    fn boot_context_full() {
        let ctx = BootContext {
            identity: Some("Primary conductor".into()),
            last_handoff: Some(r#"{"task": "research"}"#.into()),
            scratchpad: Some("- TODO: finish dream cycle".into()),
            recent_memories: vec!["- **Pattern**: fractal delegation".into()],
        };
        let prompt = ctx.to_system_prompt();
        assert!(prompt.contains("## Identity"));
        assert!(prompt.contains("## Last Session Handoff"));
        assert!(prompt.contains("## Current Scratchpad"));
        assert!(prompt.contains("## Recent Memories"));
    }

    #[test]
    fn write_handoff_creates_file() {
        let tmp = TempDir::new().unwrap();
        let path = write_handoff(
            tmp.path(),
            "researcher",
            "session-001",
            "Analyze code",
            "Found 3 patterns in the codebase",
            4,
            3,
            true,
        ).unwrap();

        assert!(path.exists());
        let content: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&path).unwrap()
        ).unwrap();
        assert_eq!(content["mind_id"], "researcher");
        assert_eq!(content["iterations"], 4);
        assert_eq!(content["completed"], true);
    }

    #[test]
    fn load_latest_handoff_finds_newest() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("data").join("handoffs").join("test-mind");
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("20260401-120000.json"), r#"{"old": true}"#).unwrap();
        std::fs::write(dir.join("20260402-120000.json"), r#"{"new": true}"#).unwrap();

        let result = load_latest_handoff(tmp.path(), "test-mind").unwrap();
        assert!(result.contains("new"));
    }

    #[test]
    fn load_scratchpad_reads_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("data").join("scratchpad");
        std::fs::create_dir_all(&dir).unwrap();
        let today = chrono::Utc::now().format("%Y-%m-%d");
        std::fs::write(dir.join(format!("researcher-{today}.md")), "# Notes\n- item 1").unwrap();

        let result = load_scratchpad(tmp.path(), "researcher").unwrap();
        assert!(result.contains("item 1"));
    }

    #[test]
    fn load_agents_md_finds_role_specific() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("agents").join("agent");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("AGENTS.md"), "You are an agent.").unwrap();

        let result = load_agents_md(tmp.path(), Role::Agent).unwrap();
        assert!(result.contains("agent"));
    }

    #[test]
    fn load_agents_md_falls_back_to_generic() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("agents");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("AGENTS.md"), "Generic instructions.").unwrap();

        let result = load_agents_md(tmp.path(), Role::Primary).unwrap();
        assert!(result.contains("Generic"));
    }

    #[test]
    fn record_fitness_creates_jsonl() {
        let tmp = TempDir::new().unwrap();
        let outcome = codex_fitness::TaskOutcome {
            task_id: "task-1".into(),
            mind_id: "researcher".into(),
            role: codex_roles::Role::Agent,
            success: true,
            duration_secs: 7.0,
            tool_calls_total: 3,
            tool_calls_successful: 3,
            memory_writes: 1,
            verification_passed: true,
            learnings_extracted: 1,
            completed_at: chrono::Utc::now(),
        };

        record_fitness(tmp.path(), "researcher", &outcome).unwrap();

        let path = tmp.path().join("data").join("fitness").join("researcher.jsonl");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        let record: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        assert!(record["composite_score"].as_f64().unwrap() > 0.5);
    }
}
