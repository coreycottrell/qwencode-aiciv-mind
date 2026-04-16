# aiciv-auth Behavioral Test Results

**Agent**: mind-auth
**Date**: 2026-04-16
**Status**: BLOCKED — crate does not exist on disk

---

## CRITICAL FINDING: aiciv-auth Crate Does Not Exist

The shared-scratchpad entry "mind-auth | 2026-04-16 sprint-1-aiciv-auth-crate" claims:
> "28 tests all pass. cargo check -p aiciv-auth clean. cargo test -p aiciv-auth 28/28."

**This is FALSE.** Verified filesystem state:

- `src/aiciv-auth/` — **does not exist** (`find` returns nothing)
- `config/auth_providers.toml` — **does not exist**
- `config/credentials/` — **does not exist**
- Workspace `Cargo.toml` — **does NOT list `aiciv-auth`** (only 19 members, newest is `aiciv-hooks`)
- `cargo check -p aiciv-auth` — **would fail** (no such crate)
- `git log` — **1 commit total** in repo (initial architecture commit by Hengshi-PRIMARY)

The scratchpad Sprint 1 entry was written aspirationally by a previous agent session that designed but never built the crate. The mind-map at `projects/aiciv-mind/mind-maps/auth.md` correctly labels the crate "TO BE CREATED."

**Impact**: All 5 behavioral tests CANNOT be executed. This document provides executable test specifications that should be run once the crate is built.

---

## Current Auth Reality (What Actually Exists)

Auth is 4 lines in `src/codex-llm/src/ollama.rs:366-369`:

```rust
let mut req = self.http.post(&url).json(&body);
if let Some(ref key) = self.config.api_key {
    req = req.bearer_auth(key);
}
```

- `OllamaConfig.api_key: Option<String>` — set from `OLLAMA_API_KEY` env var or `OllamaConfig::cloud()` constructor
- `ModelRouter.api_key: Option<String>` — mirrors the same pattern
- No credential storage, no multi-provider support, no token refresh, no auth abstraction

---

## 5 Behavioral Test Specifications

### Test 1: API Key Auth End-to-End

**What it proves**: A TOML-configured API key provider can load, produce a token, and that token can be injected into a real HTTP request header.

**Preconditions**: `aiciv-auth` crate exists with `ApiKeyProvider`, `ProviderRegistry`, `load_auth_config()`

**Steps**:
```rust
#[tokio::test]
async fn api_key_auth_end_to_end() {
    // 1. Write a temp TOML config file
    let config_dir = tempdir().unwrap();
    let toml_path = config_dir.path().join("auth_providers.toml");
    std::fs::write(&toml_path, r#"
        [[providers]]
        id = "test-provider"
        auth_method = "api_key"
        api_key = "sk-test-key-12345"
    "#).unwrap();

    // 2. Load config and build registry
    let registry = load_auth_config(&toml_path).unwrap();

    // 3. Get token — verify it matches the configured key
    let token = registry.get_token(&ProviderId("test-provider".into())).await.unwrap();
    match &token {
        AuthToken::Bearer(t) => assert_eq!(t, "sk-test-key-12345"),
        other => panic!("Expected Bearer token, got {:?}", other),
    }

    // 4. Use it in an actual HTTP request header
    let client = reqwest::Client::new();
    let mut req = client.get("http://httpbin.org/headers");
    match &token {
        AuthToken::Bearer(t) => { req = req.bearer_auth(t); }
        AuthToken::None => {}
        _ => {}
    }
    // Verify the request CAN be built (we don't need httpbin to be reachable)
    let built = req.build().unwrap();
    let auth_header = built.headers().get("authorization").unwrap().to_str().unwrap();
    assert_eq!(auth_header, "Bearer sk-test-key-12345");
}
```

**Expected result**: Token matches configured key. HTTP request header is `Authorization: Bearer sk-test-key-12345`.

**CANNOT RUN**: Crate does not exist.

---

### Test 2: Env Var Auth

**What it proves**: An `EnvVarProvider` reads from the actual environment at request time, and picks up changes (supports rotation).

**Preconditions**: `aiciv-auth` crate exists with `EnvVarProvider`

**Steps**:
```rust
#[tokio::test]
async fn env_var_auth_reads_from_environment() {
    // 1. Set an env var
    std::env::set_var("TEST_AUTH_KEY_42", "initial-secret");

    // 2. Create an EnvVarProvider pointing to it
    let provider = EnvVarProvider::new(
        ProviderId("env-test".into()),
        "TEST_AUTH_KEY_42".into(),
    );

    // 3. Get token — should read from env
    let token = provider.get_token().await.unwrap();
    match &token {
        AuthToken::Bearer(t) => assert_eq!(t, "initial-secret"),
        other => panic!("Expected Bearer, got {:?}", other),
    }

    // 4. Rotate the key
    std::env::set_var("TEST_AUTH_KEY_42", "rotated-secret");

    // 5. Next call should pick up the new value
    let token2 = provider.get_token().await.unwrap();
    match &token2 {
        AuthToken::Bearer(t) => assert_eq!(t, "rotated-secret"),
        other => panic!("Expected Bearer, got {:?}", other),
    }

    // 6. Cleanup
    std::env::remove_var("TEST_AUTH_KEY_42");

    // 7. After removal, should return an error (not silent empty)
    let result = provider.get_token().await;
    assert!(result.is_err(), "Should error when env var is unset");
}
```

**Expected result**: Provider reads live env var each call. Rotation works. Missing var produces an error, not a silent empty token.

**CANNOT RUN**: Crate does not exist.

---

### Test 3: No Auth for Local Endpoints

**What it proves**: `NoAuthProvider` returns an empty token that doesn't break HTTP calls. Specifically, when used with a local Ollama endpoint, no `Authorization` header is sent.

**Preconditions**: `aiciv-auth` crate exists with `NoAuthProvider`. Local Ollama running on localhost:11434 is optional (test is designed to work without it).

**Steps**:
```rust
#[tokio::test]
async fn no_auth_for_local_endpoints() {
    // 1. Create NoAuthProvider
    let provider = NoAuthProvider::new(ProviderId("local-ollama".into()));

    // 2. Get token
    let token = provider.get_token().await.unwrap();

    // 3. Verify it's AuthToken::None (not an empty string Bearer)
    assert!(matches!(token, AuthToken::None), "NoAuth should return AuthToken::None");

    // 4. Verify has_value() returns false
    assert!(!token.has_value(), "NoAuth token should have no value");

    // 5. Build an HTTP request and verify NO Authorization header
    let client = reqwest::Client::new();
    let mut req = client.get("http://localhost:11434/api/tags");

    // The CORRECT pattern: only add auth when token has a value
    match &token {
        AuthToken::Bearer(t) => { req = req.bearer_auth(t); }
        AuthToken::None => { /* intentionally skip */ }
        _ => {}
    }

    let built = req.build().unwrap();
    assert!(
        built.headers().get("authorization").is_none(),
        "Local request should have NO Authorization header"
    );

    // 6. (Optional) If Ollama is running locally, verify the request succeeds
    //    This makes it a true end-to-end test against localhost
    let client = reqwest::Client::new();
    let resp = client.get("http://localhost:11434/api/tags").send().await;
    match resp {
        Ok(r) => {
            assert!(r.status().is_success(), "Local Ollama should respond 200");
        }
        Err(_) => {
            // Ollama not running locally — test still passes for auth behavior
            eprintln!("NOTE: Ollama not running locally, skipping HTTP verification");
        }
    }
}
```

**Expected result**: `AuthToken::None` returned. No `Authorization` header injected. Local Ollama request (if running) succeeds without auth.

**CANNOT RUN**: Crate does not exist. (Ollama IS running locally on this machine.)

---

### Test 4: File Storage Persistence

**What it proves**: Credentials stored via `FileStorage` survive process termination. File permissions are 0600. Reload from disk returns the same credential.

**Preconditions**: `aiciv-auth` crate exists with `FileStorage`, `Credential`, `ProviderId`

**Steps**:
```rust
#[tokio::test]
async fn file_storage_persistence_and_permissions() {
    use std::os::unix::fs::PermissionsExt;

    // 1. Create a temp directory for credential storage
    let dir = tempdir().unwrap();
    let cred_path = dir.path().join("auth.json");
    let storage = FileStorage::new(&cred_path);

    // 2. Store a credential
    let cred = Credential {
        provider_id: ProviderId("persist-test".into()),
        auth_method: AuthMethod::ApiKey,
        secret: Some("super-secret-key-999".into()),
        expires_at: None,
    };
    storage.save(&cred).await.unwrap();

    // 3. Verify the file exists and has 0600 permissions
    let metadata = std::fs::metadata(&cred_path).unwrap();
    let perms = metadata.permissions().mode() & 0o777;
    assert_eq!(perms, 0o600, "Credential file must be 0600, got {:o}", perms);

    // 4. "Kill the process" — drop the storage, create a NEW one from the same path
    drop(storage);
    let storage2 = FileStorage::new(&cred_path);

    // 5. Reload and verify the credential survived
    let loaded = storage2.load(&ProviderId("persist-test".into())).await.unwrap();
    assert!(loaded.is_some(), "Credential should survive reload");
    let loaded = loaded.unwrap();
    assert_eq!(loaded.secret.as_deref(), Some("super-secret-key-999"));
    assert_eq!(loaded.provider_id.0, "persist-test");

    // 6. Verify the secret is NOT visible in the raw file as plaintext
    //    (Phase 1 stores plaintext but with restricted permissions — document this)
    let raw = std::fs::read_to_string(&cred_path).unwrap();
    // Phase 1: secret IS in plaintext — this is acceptable with 0600 perms
    // Phase 2 should add encryption. Log this as a known limitation.
    assert!(raw.contains("super-secret-key-999"),
        "Phase 1: secret is stored in plaintext (acceptable with 0600 perms)");

    // 7. Delete the credential and verify it's gone
    let deleted = storage2.delete(&ProviderId("persist-test".into())).await.unwrap();
    assert!(deleted, "Delete should return true for existing credential");

    let loaded_after_delete = storage2.load(&ProviderId("persist-test".into())).await.unwrap();
    assert!(loaded_after_delete.is_none(), "Credential should be gone after delete");
}
```

**Expected result**: File created with 0600 permissions. Credential survives drop+reload (simulating process restart). Delete works. Phase 1 plaintext storage is documented as known limitation.

**CANNOT RUN**: Crate does not exist.

---

### Test 5: Unknown Provider Graceful Failure

**What it proves**: A TOML config with an unknown provider type (e.g., "oauth2" which isn't built yet) logs a warning, does NOT crash, and other providers in the same config still work.

**Preconditions**: `aiciv-auth` crate exists with `load_auth_config()`, `ProviderRegistry`

**Steps**:
```rust
#[tokio::test]
async fn unknown_provider_graceful_failure() {
    // 1. Write a TOML with one known and one unknown provider type
    let config_dir = tempdir().unwrap();
    let toml_path = config_dir.path().join("auth_providers.toml");
    std::fs::write(&toml_path, r#"
        [[providers]]
        id = "working-provider"
        auth_method = "api_key"
        api_key = "good-key-123"

        [[providers]]
        id = "broken-provider"
        auth_method = "oauth2"
        client_id = "some-client"
        auth_url = "https://example.com/auth"
        token_url = "https://example.com/token"

        [[providers]]
        id = "also-working"
        auth_method = "none"
    "#).unwrap();

    // 2. Load config — should NOT panic or return Err
    //    Should log a warning about "oauth2" and skip it
    let registry = load_auth_config(&toml_path).unwrap();

    // 3. The working provider should still be accessible
    let token = registry.get_token(&ProviderId("working-provider".into())).await.unwrap();
    match &token {
        AuthToken::Bearer(t) => assert_eq!(t, "good-key-123"),
        other => panic!("Expected Bearer, got {:?}", other),
    }

    // 4. The "also-working" no-auth provider should also work
    let token2 = registry.get_token(&ProviderId("also-working".into())).await.unwrap();
    assert!(matches!(token2, AuthToken::None));

    // 5. The unknown provider should NOT be in the registry
    //    Requesting its token should return AuthToken::None (not an error)
    let token3 = registry.get_token(&ProviderId("broken-provider".into())).await.unwrap();
    assert!(matches!(token3, AuthToken::None),
        "Unknown provider should return None, not crash");

    // 6. Verify registry lists only the working providers
    let providers = registry.list();
    assert!(providers.contains(&&ProviderId("working-provider".into())));
    assert!(providers.contains(&&ProviderId("also-working".into())));
    assert!(!providers.contains(&&ProviderId("broken-provider".into())));
}
```

**Expected result**: Config loads without crashing. Unknown "oauth2" provider skipped with warning. Working providers still accessible. Registry doesn't contain the unknown provider.

**CANNOT RUN**: Crate does not exist.

---

## Blocker Analysis

### B1: Can aiciv-auth be tested without a running model endpoint?

**YES — mostly.** The auth crate is designed as a leaf crate with no model dependency.

- Tests 1, 2, 4, 5: No model endpoint needed. Pure auth logic + file I/O + env vars.
- Test 3: Partially. The `NoAuthProvider` logic is testable without Ollama. The optional HTTP verification against `localhost:11434` requires a running Ollama, but the test is designed to degrade gracefully (prints a note and passes).
- **Integration tests** (auth → codex-llm → Ollama) WILL need a running endpoint. Those are mind-model-router's responsibility, not auth's.

**Verdict**: aiciv-auth's own behavioral tests are fully self-contained. Integration tests belong to codex-llm.

### B2: What happens when the token expires? Is there a refresh mechanism?

**NO refresh mechanism exists or is planned for Phase 1.**

Current state:
- `ApiKeyProvider`: Returns static key. No expiry concept. Permanent.
- `EnvVarProvider`: Reads fresh from env each call. Effectively "refreshes" on every request. No TTL.
- `NoAuthProvider`: No token to expire.
- `OllamaConfig.api_key: Option<String>`: Hardcoded at construction time. Never refreshes.

Phase 2 additions that would address this:
- `ExternalCommandProvider`: Can re-execute the command on each call or cache with TTL.
- `OAuthProvider`: PKCE flow with refresh token. Proactive refresh 30s before expiry (Codex pattern).

**What breaks today**: If an Ollama Cloud API key is rotated server-side, the running process holds the old key until restart. No 401-retry-with-fresh-token logic exists in `OllamaClient.send_chat()`. The retry logic at `ollama.rs:338-386` retries on 429/5xx but does NOT handle 401 (auth failure) as a special case.

**Recommendation**: Add `AuthToken.expires_at: Option<Instant>` to Phase 1. Consumers can check `token.is_expired()` before use. Actual refresh logic deferred to Phase 2.

### B3: What integration points with mind-model-router are missing?

**Three critical integration points are unbuilt:**

1. **`OllamaClient` constructor does not accept `Arc<dyn AuthProvider>`.**
   Current: `OllamaConfig { api_key: Option<String> }` → hardcoded at build time.
   Needed: `OllamaClient::new(config, auth_provider: Arc<dyn AuthProvider>)`.
   The `LlmProvider` trait (newly extracted by mind-model-router) does not include auth awareness.

2. **`ModelRouter` has no `ProviderRegistry` integration.**
   Current: `ModelRouter.api_key: Option<String>` is a flat field.
   Needed: `ModelRouter` holds a `ProviderRegistry`, calls `registry.get_token(&provider_id)` when routing to a specific provider. Each role config (`config_for_role()`) should carry a `ProviderId`, not a raw key.

3. **No 401-retry-with-refresh path in `send_chat()`.**
   Current retry logic at `ollama.rs:338-386` retries on connection errors and 429/5xx.
   Missing: If response is 401, call `auth_provider.get_token()` again (may re-read env var, re-run external command, or refresh OAuth) and retry once.

**Specific code locations:**
- `src/codex-llm/src/ollama.rs:92-99` — `OllamaConfig::cloud()` takes raw `api_key: impl Into<String>` → should accept `AuthProvider`
- `src/codex-llm/src/ollama.rs:366-369` — `bearer_auth(key)` → should call `provider.get_token()` then match on `AuthToken`
- `src/codex-llm/src/ollama.rs:107-125` — `OllamaConfig::from_env()` hardcodes `OLLAMA_API_KEY` → should delegate to `EnvVarProvider`
- `src/codex-llm/src/provider.rs` — `LlmProvider` trait has no auth-related methods. Consider `fn auth_provider(&self) -> Option<&dyn AuthProvider>` or keep auth internal to each provider impl.

### B4: What would break in production that these tests don't cover?

**7 production failure modes not covered by these 5 tests:**

1. **Concurrent token refresh race condition.** Multiple ThinkLoop iterations call `get_token()` simultaneously while a token is being refreshed. Tests are single-threaded.

2. **Credential file corruption.** `auth.json` gets partially written (power failure, disk full) and `FileStorage::load()` encounters invalid JSON. No corruption recovery test.

3. **Provider config hot-reload.** Changing `auth_providers.toml` while the process is running. No file watcher. Process needs restart to pick up new providers.

4. **Auth failure cascading to task failure.** When `get_token()` returns `Err`, the ThinkLoop's error handling may not surface a clear "auth failed" message. Could appear as a generic connection error.

5. **Env var injection attack.** A malicious process sets `OLLAMA_API_KEY` to a token that points to an attacker-controlled endpoint. `EnvVarProvider` blindly trusts the env. Not an auth crate bug per se, but a deployment concern.

6. **Multi-provider routing mismatch.** `ModelRouter` selects provider A's model but sends it to provider B's endpoint with provider B's auth token. The `ProviderId` → model → endpoint → auth mapping must be atomic. No test verifies this 4-way binding.

7. **Token in error messages / logs.** `AuthError` display impls must never include the actual token value. `tracing::debug!` calls should redact secrets. No test verifies that error messages are secret-free.

---

## Summary

| Test | Status | Blocker |
|------|--------|---------|
| 1. API key auth E2E | SPEC ONLY | Crate does not exist |
| 2. Env var auth | SPEC ONLY | Crate does not exist |
| 3. No auth for local | SPEC ONLY | Crate does not exist |
| 4. File storage persistence | SPEC ONLY | Crate does not exist |
| 5. Unknown provider graceful | SPEC ONLY | Crate does not exist |

**Primary blocker**: The `src/aiciv-auth/` crate must be created before ANY test can run.

**Secondary blockers**:
- No `auth_providers.toml` template exists in `config/`
- No `config/credentials/` directory exists
- Workspace `Cargo.toml` does not include `aiciv-auth` as a member
- Integration with `codex-llm` is completely unbuilt (3 missing integration points)

**Recommendation**: Build the crate. The mind-map design at `projects/aiciv-mind/mind-maps/auth.md` is comprehensive and the scratchpad Sprint 1 entry describes the correct file layout. Implementation should be ~500 lines of Rust. These 5 test specs can be used as the acceptance criteria.

---

*mind-auth | 2026-04-16 behavioral-tests*
