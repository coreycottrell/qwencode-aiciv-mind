//! Behavioral tests for aiciv-hooks.
//!
//! These are end-to-end tests that prove hooks work the way a real AiCIV
//! would use them: external shell scripts reading JSON from stdin, making
//! decisions, and returning responses that the dispatcher acts on.

use std::sync::Arc;
use std::time::Duration;

use aiciv_hooks::dispatcher::{Decision, HookDispatcher};
use aiciv_hooks::handler::ExternalCommandHandler;
use aiciv_hooks::types::{HookEvent, HookEventType};
use aiciv_hooks::config::HooksSettings;

/// Path to test scripts, relative to the crate root.
fn script_path(name: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/tests/scripts/{}", manifest_dir, name)
}

// =============================================================================
// Test 1: External command hook BLOCKS a dangerous tool call
// =============================================================================
//
// Scenario: A shell script reads JSON from stdin, sees tool_name="bash" with
// command containing "rm -rf", and returns {"should_block":true}.
// The dispatcher must block the operation and return the reason.

#[tokio::test]
async fn test_1_external_hook_blocks_dangerous_command() {
    let handler = ExternalCommandHandler::new(script_path("safety_checker.sh"))
        .with_timeout(Duration::from_secs(5));

    let mut dispatcher = HookDispatcher::new();
    dispatcher.register(HookEventType::PreToolUse, Arc::new(handler));

    // Fire a dangerous bash command
    let event = HookEvent::PreToolUse {
        session_id: "behavioral-test-1".into(),
        tool_name: "bash".into(),
        tool_input: serde_json::json!({
            "command": "rm -rf /important/data"
        }),
    };

    let decision = dispatcher.fire_blocking(&event).await;

    // MUST be blocked
    assert!(
        decision.is_blocked(),
        "dangerous 'rm -rf' command should be blocked by safety_checker.sh"
    );

    if let Decision::Block { reason } = decision {
        assert!(
            reason.contains("rm -rf"),
            "block reason should mention 'rm -rf', got: {reason}"
        );
    }
}

// =============================================================================
// Test 2: External command hook APPROVES a safe command
// =============================================================================
//
// Scenario: Same safety_checker.sh, but with a safe "ls -la" command.
// The hook reads the input, finds no "rm -rf", and returns should_block=false.

#[tokio::test]
async fn test_2_external_hook_approves_safe_command() {
    let handler = ExternalCommandHandler::new(script_path("safety_checker.sh"))
        .with_timeout(Duration::from_secs(5));

    let mut dispatcher = HookDispatcher::new();
    dispatcher.register(HookEventType::PreToolUse, Arc::new(handler));

    // Fire a safe command
    let event = HookEvent::PreToolUse {
        session_id: "behavioral-test-2".into(),
        tool_name: "bash".into(),
        tool_input: serde_json::json!({
            "command": "ls -la /home"
        }),
    };

    let decision = dispatcher.fire_blocking(&event).await;

    // MUST be allowed
    assert!(
        !decision.is_blocked(),
        "safe 'ls -la' command should be approved by safety_checker.sh"
    );
}

// =============================================================================
// Test 3: Hook timeout falls back to fail-open
// =============================================================================
//
// Scenario: A hook runs "sleep 30" which exceeds the 200ms timeout.
// With fail_open=true (default), the dispatcher should NOT hang and should
// fall back to allow.
//
// CRITICAL BEHAVIOR: An AiCIV must never hang waiting for a broken hook.
// Fail-open means: if the hook is broken, let the operation through rather
// than blocking the entire agent.

#[tokio::test]
async fn test_3_timeout_falls_back_to_fail_open() {
    // The sleep command will run for 30 seconds, but timeout is 200ms.
    // fail_open=true (default) means timeout -> Ack -> no block.
    let handler = ExternalCommandHandler::new("sleep".into())
        .with_args(vec!["30".into()])
        .with_timeout(Duration::from_millis(200))
        .with_fail_open(true); // fail_open: if hook fails/times out, allow

    let mut dispatcher = HookDispatcher::new();
    dispatcher.register(HookEventType::PreToolUse, Arc::new(handler));

    let event = HookEvent::PreToolUse {
        session_id: "behavioral-test-3".into(),
        tool_name: "bash".into(),
        tool_input: serde_json::json!({"command": "echo hello"}),
    };

    let start = std::time::Instant::now();
    let decision = dispatcher.fire_blocking(&event).await;
    let elapsed = start.elapsed();

    // MUST complete quickly (within 2 seconds, well under the 30s sleep)
    assert!(
        elapsed < Duration::from_secs(2),
        "dispatcher should not hang; elapsed {:?}",
        elapsed
    );

    // MUST allow (fail-open mode)
    assert!(
        !decision.is_blocked(),
        "timeout with fail_open=true should fall back to allow"
    );
}

// =============================================================================
// Test 4: Multiple hooks chain — approve, approve, BLOCK
// =============================================================================
//
// Scenario: 3 hooks registered on PreToolUse. First two approve (approver.sh),
// third blocks (blocker.sh). The chain must stop at the block and return the
// block reason from hook 3.
//
// This tests the dispatcher's sequential execution and short-circuit behavior.

#[tokio::test]
async fn test_4_multiple_hooks_chain_block_at_third() {
    let approver1 = ExternalCommandHandler::new(script_path("approver.sh"))
        .with_timeout(Duration::from_secs(5));
    let approver2 = ExternalCommandHandler::new(script_path("approver.sh"))
        .with_timeout(Duration::from_secs(5));
    let blocker = ExternalCommandHandler::new(script_path("blocker.sh"))
        .with_timeout(Duration::from_secs(5));

    let mut dispatcher = HookDispatcher::new();
    dispatcher.register(HookEventType::PreToolUse, Arc::new(approver1));
    dispatcher.register(HookEventType::PreToolUse, Arc::new(approver2));
    dispatcher.register(HookEventType::PreToolUse, Arc::new(blocker));

    let event = HookEvent::PreToolUse {
        session_id: "behavioral-test-4".into(),
        tool_name: "bash".into(),
        tool_input: serde_json::json!({"command": "echo test"}),
    };

    let decision = dispatcher.fire_blocking(&event).await;

    // MUST be blocked (third hook blocks)
    assert!(
        decision.is_blocked(),
        "chain should be blocked by the third hook"
    );

    if let Decision::Block { reason } = decision {
        assert!(
            reason.contains("chain hook 3"),
            "block reason should come from the third hook, got: {reason}"
        );
    }

    // Also verify fire() collects all 3 responses (no short-circuit in fire mode)
    let responses = dispatcher.fire(&event).await;
    assert_eq!(
        responses.len(),
        3,
        "fire() should collect responses from all 3 hooks"
    );
}

// =============================================================================
// Test 5: Tool-name filtering — hook only fires for specific tools
// =============================================================================
//
// Scenario: A blocking hook is registered for "edit" tool ONLY.
// When a "bash" event fires, the hook should NOT fire (no block).
// When an "edit" event fires, the hook SHOULD fire (block).
//
// This is critical for AiCIV: you want a safety hook that only checks
// file-write tools, not every tool invocation.

#[tokio::test]
async fn test_5_tool_name_filtering() {
    // Register blocker.sh only for "edit" tool
    let blocker = ExternalCommandHandler::new(script_path("blocker.sh"))
        .with_timeout(Duration::from_secs(5));

    let mut dispatcher = HookDispatcher::new();
    dispatcher.register_with_filter(
        HookEventType::PreToolUse,
        Arc::new(blocker),
        Some(vec!["edit".into()]),
    );

    // PART A: Fire a "bash" event — hook should NOT fire (filter doesn't match)
    let bash_event = HookEvent::PreToolUse {
        session_id: "behavioral-test-5a".into(),
        tool_name: "bash".into(),
        tool_input: serde_json::json!({"command": "echo safe"}),
    };

    let decision = dispatcher.fire_blocking(&bash_event).await;
    assert!(
        !decision.is_blocked(),
        "blocker registered for 'edit' should NOT fire for 'bash' event"
    );

    let responses = dispatcher.fire(&bash_event).await;
    assert_eq!(
        responses.len(),
        0,
        "no responses expected when tool filter doesn't match"
    );

    // PART B: Fire an "edit" event — hook SHOULD fire (filter matches)
    let edit_event = HookEvent::PreToolUse {
        session_id: "behavioral-test-5b".into(),
        tool_name: "edit".into(),
        tool_input: serde_json::json!({
            "file_path": "/tmp/test.rs",
            "old_string": "foo",
            "new_string": "bar"
        }),
    };

    let decision = dispatcher.fire_blocking(&edit_event).await;
    assert!(
        decision.is_blocked(),
        "blocker registered for 'edit' SHOULD fire for 'edit' event"
    );

    let responses = dispatcher.fire(&edit_event).await;
    assert_eq!(
        responses.len(),
        1,
        "exactly 1 response expected when tool filter matches"
    );
}

// =============================================================================
// BONUS: Config-driven dispatcher test (end-to-end from JSON config)
// =============================================================================
//
// Proves the full pipeline: JSON config → HooksSettings → HookDispatcher
// → fire event → get correct behavior. This is how a real AiCIV would
// configure hooks — via a JSON settings file.

#[tokio::test]
async fn test_bonus_config_driven_dispatcher() {
    let config_json = format!(
        r#"{{
            "hooks": [
                {{
                    "event": "pre_tool_use",
                    "command": "{}",
                    "tool_names": ["bash"],
                    "timeout_ms": 5000,
                    "required": true
                }}
            ]
        }}"#,
        script_path("safety_checker.sh")
    );

    let settings = HooksSettings::from_json(&config_json).unwrap();
    let dispatcher = HookDispatcher::from_settings(&settings);

    // Dangerous bash command should be blocked
    let dangerous = HookEvent::PreToolUse {
        session_id: "config-test".into(),
        tool_name: "bash".into(),
        tool_input: serde_json::json!({"command": "rm -rf /"}),
    };
    let decision = dispatcher.fire_blocking(&dangerous).await;
    assert!(decision.is_blocked(), "config-driven hook should block rm -rf");

    // read tool should not trigger hook (tool_names filter = ["bash"])
    let read_event = HookEvent::PreToolUse {
        session_id: "config-test".into(),
        tool_name: "read".into(),
        tool_input: serde_json::json!({"path": "/tmp/x"}),
    };
    let decision = dispatcher.fire_blocking(&read_event).await;
    assert!(!decision.is_blocked(), "read tool should bypass bash-only hook");
}
