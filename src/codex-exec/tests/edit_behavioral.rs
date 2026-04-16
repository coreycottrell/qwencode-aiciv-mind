//! Behavioral tests for the Edit tool.
//!
//! These are END-TO-END tests that create real files on disk, call the
//! EditTool handler directly, and verify the file system state afterwards.
//! They test what a user would actually experience, not internal logic.

use codex_exec::registry::ToolHandler;
use codex_exec::tools::edit::EditTool;
use std::path::PathBuf;
use tokio::fs;

/// Create a temp file with the given name and content, return its path.
async fn create_test_file(name: &str, content: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("aiciv-mind-behavioral-edit");
    fs::create_dir_all(&dir).await.unwrap();
    let path = dir.join(name);
    fs::write(&path, content).await.unwrap();
    path
}

// ============================================================
// TEST 1: Happy Path — single-line edit in a Python file
// ============================================================
// Create a real Python file with a function, edit one line,
// verify only that line changed and everything else is intact.
#[tokio::test]
async fn behavioral_01_happy_path_single_line_edit() {
    let content = r#"def greet(name):
    """Greet someone."""
    message = "Hello, " + name
    print(message)
    return message

def farewell(name):
    """Say goodbye."""
    print("Goodbye, " + name)
"#;

    let path = create_test_file("happy_path.py", content).await;
    let tool = EditTool;

    // Edit: change "Hello" to "Hi" in the greeting function
    let result = tool
        .execute(serde_json::json!({
            "file_path": path.to_str().unwrap(),
            "old_string": "    message = \"Hello, \" + name",
            "new_string": "    message = \"Hi, \" + name"
        }))
        .await;

    // Verify success
    assert!(result.success, "Expected success, got error: {:?}", result.error);
    assert!(result.output.contains("Replaced 1 occurrence"), "Output: {}", result.output);

    // Read back and verify exact file state
    let after = fs::read_to_string(&path).await.unwrap();

    // The changed line
    assert!(after.contains("    message = \"Hi, \" + name"), "Changed line missing");
    // The OLD line must be gone
    assert!(!after.contains("    message = \"Hello, \" + name"), "Old line still present");
    // Everything else must be untouched
    assert!(after.contains("def greet(name):"), "Function header changed");
    assert!(after.contains("    \"\"\"Greet someone.\"\"\""), "Docstring changed");
    assert!(after.contains("    print(message)"), "print line changed");
    assert!(after.contains("    return message"), "return line changed");
    assert!(after.contains("def farewell(name):"), "farewell function changed");
    assert!(after.contains("    print(\"Goodbye, \" + name)"), "farewell body changed");

    // Line count must be identical (we replaced, not inserted/deleted)
    assert_eq!(
        content.lines().count(),
        after.lines().count(),
        "Line count changed! Before: {}, After: {}",
        content.lines().count(),
        after.lines().count()
    );

    // Cleanup
    let _ = fs::remove_file(&path).await;
}

// ============================================================
// TEST 2: Multiline edit — replace 3 lines in the middle of 10
// ============================================================
// Verify that surrounding lines stay completely intact.
#[tokio::test]
async fn behavioral_02_multiline_edit_middle_of_file() {
    let content = "line 1: header\n\
                   line 2: imports\n\
                   line 3: constants\n\
                   line 4: old logic A\n\
                   line 5: old logic B\n\
                   line 6: old logic C\n\
                   line 7: validation\n\
                   line 8: output\n\
                   line 9: cleanup\n\
                   line 10: footer\n";

    let path = create_test_file("multiline_edit.txt", content).await;
    let tool = EditTool;

    // Replace lines 4-6 (3 lines) with new content
    let result = tool
        .execute(serde_json::json!({
            "file_path": path.to_str().unwrap(),
            "old_string": "line 4: old logic A\nline 5: old logic B\nline 6: old logic C",
            "new_string": "line 4: NEW logic A\nline 5: NEW logic B\nline 6: NEW logic C"
        }))
        .await;

    assert!(result.success, "Expected success, got error: {:?}", result.error);

    let after = fs::read_to_string(&path).await.unwrap();
    let lines: Vec<&str> = after.lines().collect();

    // Lines before the edit (1-3) must be untouched
    assert_eq!(lines[0], "line 1: header", "Line 1 changed");
    assert_eq!(lines[1], "line 2: imports", "Line 2 changed");
    assert_eq!(lines[2], "line 3: constants", "Line 3 changed");

    // Edited lines (4-6)
    assert_eq!(lines[3], "line 4: NEW logic A", "Line 4 not edited");
    assert_eq!(lines[4], "line 5: NEW logic B", "Line 5 not edited");
    assert_eq!(lines[5], "line 6: NEW logic C", "Line 6 not edited");

    // Lines after the edit (7-10) must be untouched
    assert_eq!(lines[6], "line 7: validation", "Line 7 changed");
    assert_eq!(lines[7], "line 8: output", "Line 8 changed");
    assert_eq!(lines[8], "line 9: cleanup", "Line 9 changed");
    assert_eq!(lines[9], "line 10: footer", "Line 10 changed");

    // Total line count preserved
    assert_eq!(lines.len(), 10, "Line count changed: {}", lines.len());

    let _ = fs::remove_file(&path).await;
}

// ============================================================
// TEST 3: Ambiguous match rejection — 5 TODOs, no replace_all
// ============================================================
// Verify it refuses with a helpful error showing line numbers.
#[tokio::test]
async fn behavioral_03_ambiguous_match_shows_line_numbers() {
    let content = "# TODO: fix authentication\n\
                   def login():\n\
                       pass  # TODO: implement\n\
                   \n\
                   # TODO: add rate limiting\n\
                   def api_call():\n\
                       pass  # TODO: add retry logic\n\
                   \n\
                   # TODO: write tests\n";

    let path = create_test_file("ambiguous_todo.py", content).await;
    let tool = EditTool;

    let result = tool
        .execute(serde_json::json!({
            "file_path": path.to_str().unwrap(),
            "old_string": "TODO",
            "new_string": "DONE"
        }))
        .await;

    // Must FAIL
    assert!(!result.success, "Should have rejected ambiguous match");

    let err = result.error.as_deref().unwrap_or("");

    // Must tell the user how many matches
    assert!(err.contains("5 times"), "Error should say '5 times', got: {err}");

    // Must show line numbers so the user knows WHERE
    assert!(err.contains("line 1"), "Missing line 1 reference in: {err}");
    assert!(err.contains("line 3"), "Missing line 3 reference in: {err}");
    assert!(err.contains("line 5"), "Missing line 5 reference in: {err}");
    assert!(err.contains("line 7"), "Missing line 7 reference in: {err}");
    assert!(err.contains("line 9"), "Missing line 9 reference in: {err}");

    // Must suggest fix: more context or replace_all
    assert!(
        err.contains("replace_all") || err.contains("more") || err.contains("context"),
        "Error should suggest how to fix, got: {err}"
    );

    // File must be UNCHANGED
    let after = fs::read_to_string(&path).await.unwrap();
    assert_eq!(after, content, "File was modified despite error!");

    let _ = fs::remove_file(&path).await;
}

// ============================================================
// TEST 4: Replace-all — same 5 TODOs, replace_all=true
// ============================================================
// Verify all 5 instances are replaced.
#[tokio::test]
async fn behavioral_04_replace_all_changes_every_occurrence() {
    let content = "# TODO: fix authentication\n\
                   def login():\n\
                       pass  # TODO: implement\n\
                   \n\
                   # TODO: add rate limiting\n\
                   def api_call():\n\
                       pass  # TODO: add retry logic\n\
                   \n\
                   # TODO: write tests\n";

    let path = create_test_file("replace_all_todo.py", content).await;
    let tool = EditTool;

    let result = tool
        .execute(serde_json::json!({
            "file_path": path.to_str().unwrap(),
            "old_string": "TODO",
            "new_string": "DONE",
            "replace_all": true
        }))
        .await;

    assert!(result.success, "Expected success, got error: {:?}", result.error);
    assert!(
        result.output.contains("5 occurrences"),
        "Output should mention 5 occurrences, got: {}",
        result.output
    );

    let after = fs::read_to_string(&path).await.unwrap();

    // No TODOs remaining
    assert_eq!(
        after.matches("TODO").count(),
        0,
        "Still have TODO in file: {after}"
    );

    // Exactly 5 DONEs
    assert_eq!(
        after.matches("DONE").count(),
        5,
        "Expected 5 DONE, got {}: {after}",
        after.matches("DONE").count()
    );

    // Non-TODO content untouched
    assert!(after.contains("def login():"), "login function lost");
    assert!(after.contains("def api_call():"), "api_call function lost");
    assert!(after.contains("pass"), "pass statements lost");

    let _ = fs::remove_file(&path).await;
}

// ============================================================
// TEST 5: File doesn't exist — helpful error, no panic
// ============================================================
// Verify the error message is actionable (says what file, not a stack trace).
#[tokio::test]
async fn behavioral_05_nonexistent_file_helpful_error() {
    let bogus_path = "/tmp/aiciv-mind-behavioral-edit/this-file-does-not-exist-at-all.rs";

    // Make sure it really doesn't exist
    let _ = fs::remove_file(bogus_path).await;

    let tool = EditTool;
    let result = tool
        .execute(serde_json::json!({
            "file_path": bogus_path,
            "old_string": "fn main()",
            "new_string": "fn start()"
        }))
        .await;

    // Must fail, not panic
    assert!(!result.success, "Should fail for missing file");

    let err = result.error.as_deref().unwrap_or("");

    // Must mention the file path so user knows WHICH file failed
    assert!(
        err.contains(bogus_path),
        "Error should include the file path, got: {err}"
    );

    // Must say it failed to read (not a cryptic OS error)
    assert!(
        err.contains("Failed to read"),
        "Error should say 'Failed to read', got: {err}"
    );

    // Must NOT be empty or a panic message
    assert!(!err.is_empty(), "Error message is empty");
    assert!(!err.contains("panic"), "Error contains 'panic': {err}");
    assert!(!err.contains("stack backtrace"), "Error is a stack trace: {err}");
}
