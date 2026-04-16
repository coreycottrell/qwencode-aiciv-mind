# Mind-Map: mind-auth Domain

**Owner**: mind-auth
**Crate**: `src/aiciv-auth/` (TO BE CREATED)
**Lines**: 0 (greenfield)
**Status**: Phase 3 — Design complete, implementation pending
**Last Updated**: 2026-04-16

---

## 1. What Exists Today — Current Auth State

### Inline Auth in codex-llm/ollama.rs

There is NO dedicated auth module. Authentication is embedded directly in `OllamaClient`:

```rust
// codex-llm/src/ollama.rs, lines 33-34, 364-365
pub struct OllamaConfig {
    pub api_key: Option<String>,  // When set, sent as Authorization: Bearer <key>
}

// In send_chat():
if let Some(ref key) = self.config.api_key {
    req = req.bearer_auth(key);
}
```

**What works:**
- Single `OLLAMA_API_KEY` env var → Bearer token injection
- Cloud vs local detection (key present = cloud mode, absent = local)
- Per-role model configs carry their own `api_key: Option<String>`

**What's missing:**
- No credential storage (keys only in env vars or hardcoded in config)
- No token refresh (bearer tokens are assumed eternal)
- No multi-provider auth (cannot auth to OpenAI, Anthropic, OpenRouter simultaneously)
- No OAuth flows (cannot do PKCE, device code, or service account impersonation)
- No MCP server auth (MCP servers may require OAuth)
- No AgentAuth Ed25519 signing (ACG's identity system uses Ed25519 keypairs)

### qwen-mind/llm.rs — Same Pattern, Duplicated

```rust
// qwen-mind/src/llm.rs (simpler OllamaClient)
// Same api_key: Option<String> + bearer_auth() pattern
// No rate limiter, no circuit breaker
```

### Environment Variables (Current Auth Surface)

| Env Var | Used By | Purpose |
|---------|---------|---------|
| `OLLAMA_API_KEY` | `OllamaConfig::from_env()`, `ModelRouter::from_env()` | Bearer token for Ollama Cloud |
| `OLLAMA_BASE_URL` | `OllamaConfig::from_env()` | Endpoint URL (defaults to cloud if key set) |
| `CORTEX_PRIMARY_MODEL` | `ModelRouter` | Per-role model override |
| `CORTEX_TEAM_LEAD_MODEL` | `ModelRouter` | Per-role model override |
| `CORTEX_AGENT_MODEL` | `ModelRouter` | Per-role model override |
| `CORTEX_LIGHTWEIGHT_MODEL` | `ModelRouter` | Per-role model override |

**No env vars for:** OpenAI key, Anthropic key, OpenRouter key, MCP OAuth client secrets, AgentAuth identity keys.

---

## 2. Source Analysis — What We Learn From

### Codex Login Crate (codex-upstream/codex-rs/login/, ~8K lines)

**Structure:**
```
login/src/
├── lib.rs                    # Re-exports
├── auth/
│   ├── mod.rs               # Module root
│   ├── manager.rs           # AuthManager — central orchestrator (~1,500 lines)
│   ├── storage.rs           # AuthStorageBackend trait + File/Keyring/Ephemeral impls
│   ├── default_client.rs    # HTTP client factory
│   ├── external_bearer.rs   # External command-based bearer token refresh
│   ├── error.rs             # Auth error types
│   └── util.rs              # JWT parsing utilities
├── provider_auth.rs         # Provider-scoped auth managers
├── token_data.rs            # JWT token parsing and claims
├── device_code_auth.rs      # OAuth device code flow
├── pkce.rs                  # OAuth PKCE challenge generation
├── server.rs                # Local HTTP server for OAuth callback
└── auth_env_telemetry.rs    # Auth telemetry collection
```

**Key patterns to adopt:**

1. **`CodexAuth` enum — discriminated auth modes:**
   ```rust
   pub enum CodexAuth {
       ApiKey(ApiKeyAuth),
       Chatgpt(ChatgptAuth),
       ChatgptAuthTokens(ChatgptAuthTokens),
   }
   ```
   We generalize this: `AuthMethod { ApiKey, BearerToken, OAuth, Ed25519Signature }`

2. **`AuthStorageBackend` trait — composable storage:**
   ```rust
   pub(super) trait AuthStorageBackend: Debug + Send + Sync {
       fn load(&self) -> std::io::Result<Option<AuthDotJson>>;
       fn save(&self, auth: &AuthDotJson) -> std::io::Result<()>;
       fn delete(&self) -> std::io::Result<bool>;
   }
   ```
   With impls: `FileAuthStorage`, `KeyringAuthStorage`, `EphemeralAuthStorage`

3. **`AuthCredentialsStoreMode` — user-selectable storage strategy:**
   ```rust
   pub enum AuthCredentialsStoreMode {
       File,      // Persist in auth.json
       Keyring,   // System keyring
       Auto,      // Keyring with file fallback
       Ephemeral, // Memory only
   }
   ```

4. **Provider-scoped auth** (`provider_auth.rs`):
   ```rust
   pub fn auth_manager_for_provider(
       auth_manager: Option<Arc<AuthManager>>,
       provider: &ModelProviderInfo,
   ) -> Option<Arc<AuthManager>>
   ```
   Each model provider can have its own auth manager. This is EXACTLY what we need for multi-provider routing.

5. **External bearer token refresh** (`external_bearer.rs`):
   Providers can specify an external command that produces bearer tokens. This decouples auth entirely from the harness — any auth scheme becomes a shell command.

**What NOT to adopt:**
- OpenAI/ChatGPT-specific OAuth flows (vendor lock-in)
- `codex_keyring_store` dependency (heavy, platform-specific; we start with file storage)
- JWT claims parsing for ChatGPT plan types (irrelevant to us)
- Telemetry collection (not our focus)

### Gemini CLI Auth (from module map analysis)

**Key files** (TypeScript, `packages/core/src/services/`):
- `auth-provider.ts` — base auth provider interface
- `google-auth-provider.ts` — Google OAuth
- `oauth-provider.ts` — generic OAuth 2.0
- `oauth-token-storage.ts` — encrypted token persistence
- `sa-impersonation-provider.ts` — service account impersonation
- `mcp-oauth-provider.ts` — MCP OAuth (for remote MCP servers)
- `keychainService.ts` — system keychain
- `fileKeychain.ts` — file-based fallback
- `apiKeyCredentialStorage.ts` — API key storage
- `environmentSanitization.ts` — sanitizes env vars

**Key pattern**: Auth providers are **composable and selectable per-connection**. A single CLI session can talk to:
- Google API (OAuth)
- MCP server A (API key)
- MCP server B (OAuth + PKCE)
- Local model (no auth)

All through the SAME auth provider interface. The caller never knows which auth method is used.

**Independence score: HIGH** — Auth is a gatekeeping module with clear inputs (credentials/config) and outputs (authenticated HTTP headers). It doesn't need to understand the agent loop, tool engine, or memory system.

---

## 3. Architecture Design — aiciv-auth Crate

### Design Principles

1. **Provider-agnostic**: Same `AuthProvider` trait whether talking to Ollama, OpenAI, Anthropic, OpenRouter, or a custom endpoint
2. **Composable providers**: Different auth methods compose through a single trait. A provider registry maps provider IDs to auth methods.
3. **Credential isolation**: Keys stored in `config/credentials/` with `0600` permissions, never in code, never logged
4. **Zero vendor lock-in**: No hardcoded provider URLs or auth flows. Everything is configurable.
5. **Fail-safe**: Auth failures produce clear errors with remediation steps (e.g., "OLLAMA_API_KEY not set — run `aiciv auth add ollama`"), never silent fallbacks
6. **External command escape hatch**: Any auth method can delegate to an external command (Codex pattern), making the harness infinitely extensible

### Crate Structure

```
src/aiciv-auth/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Module root, re-exports AuthProvider trait + ProviderRegistry
│   ├── types.rs                  # AuthToken, Credential, ProviderConfig, AuthMethod enum
│   ├── provider.rs               # AuthProvider trait definition
│   ├── registry.rs               # ProviderRegistry — maps provider IDs to auth providers
│   ├── providers/
│   │   ├── mod.rs                # Provider implementations index
│   │   ├── api_key.rs            # ApiKeyProvider — static API key → Bearer header
│   │   ├── env_var.rs            # EnvVarProvider — reads key from env var at request time
│   │   ├── oauth.rs              # OAuthProvider — OAuth 2.0 + PKCE flow (Phase 2)
│   │   ├── external_command.rs   # ExternalCommandProvider — shell command produces token
│   │   └── none.rs               # NoAuthProvider — for local endpoints (Ollama localhost)
│   ├── storage/
│   │   ├── mod.rs                # CredentialStorage trait
│   │   ├── file.rs               # FileStorage — JSON file with 0600 permissions
│   │   └── memory.rs             # MemoryStorage — ephemeral, for testing/CI
│   └── config.rs                 # Load provider auth config from TOML/JSON
```

### Core Types (`types.rs`)

```rust
use serde::{Deserialize, Serialize};

/// Identifies a model provider (used as key in ProviderRegistry)
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderId(pub String);

/// How a provider authenticates requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// No authentication (local Ollama, localhost endpoints)
    None,
    /// Static API key sent as Bearer token
    ApiKey,
    /// API key read from environment variable at request time
    EnvVar { var_name: String },
    /// OAuth 2.0 with PKCE (for providers that require it)
    OAuth {
        client_id: String,
        auth_url: String,
        token_url: String,
        scopes: Vec<String>,
    },
    /// External command that produces a bearer token on stdout
    ExternalCommand { command: String, args: Vec<String> },
}

/// An authenticated token ready to be injected into HTTP headers
#[derive(Debug, Clone)]
pub enum AuthToken {
    /// Bearer token → Authorization: Bearer <token>
    Bearer(String),
    /// Custom header → <name>: <value>
    CustomHeader { name: String, value: String },
    /// No auth needed
    None,
}

/// Stored credential for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub provider_id: ProviderId,
    pub auth_method: AuthMethod,
    /// The actual secret (API key, refresh token, etc.)
    /// NEVER logged, NEVER displayed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// When this credential expires (None = never)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Provider configuration (loaded from config file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAuthConfig {
    pub provider_id: ProviderId,
    pub auth_method: AuthMethod,
    /// Header name for the auth token (default: "Authorization")
    #[serde(default = "default_header_name")]
    pub header_name: String,
    /// Header value prefix (default: "Bearer ")
    #[serde(default = "default_header_prefix")]
    pub header_prefix: String,
}

fn default_header_name() -> String { "Authorization".into() }
fn default_header_prefix() -> String { "Bearer ".into() }
```

### Core Trait (`provider.rs`)

```rust
use async_trait::async_trait;
use crate::types::{AuthToken, ProviderId};

/// The single interface for getting authenticated HTTP headers.
/// Every auth method implements this trait.
#[async_trait]
pub trait AuthProvider: Send + Sync + std::fmt::Debug {
    /// Get the auth token for the next request.
    /// May refresh tokens, read env vars, run external commands, etc.
    async fn get_token(&self) -> Result<AuthToken, AuthError>;

    /// Provider ID this auth is for
    fn provider_id(&self) -> &ProviderId;

    /// Whether this provider's credentials are currently valid
    async fn is_valid(&self) -> bool;

    /// Human-readable description of what auth method is in use
    fn description(&self) -> String;
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("No credentials configured for provider {0}")]
    NoCredentials(ProviderId),

    #[error("Credential expired for provider {0} — refresh required")]
    Expired(ProviderId),

    #[error("Environment variable {var} not set — required for provider {provider}")]
    EnvVarMissing { var: String, provider: ProviderId },

    #[error("External auth command failed: {0}")]
    ExternalCommandFailed(String),

    #[error("OAuth flow failed: {0}")]
    OAuthFailed(String),

    #[error("Credential storage error: {0}")]
    StorageError(String),
}
```

### Provider Registry (`registry.rs`)

```rust
use std::collections::HashMap;
use std::sync::Arc;
use crate::provider::AuthProvider;
use crate::types::ProviderId;

/// Central registry mapping provider IDs to their auth providers.
/// mind-model-router registers providers here; aiciv-auth manages credentials.
pub struct ProviderRegistry {
    providers: HashMap<ProviderId, Arc<dyn AuthProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: HashMap::new() }
    }

    /// Register an auth provider for a model provider
    pub fn register(&mut self, provider: Arc<dyn AuthProvider>) {
        self.providers.insert(provider.provider_id().clone(), provider);
    }

    /// Get auth for a specific provider
    pub fn get(&self, id: &ProviderId) -> Option<&Arc<dyn AuthProvider>> {
        self.providers.get(id)
    }

    /// Get auth token for a provider (convenience)
    pub async fn get_token(&self, id: &ProviderId) -> Result<AuthToken, AuthError> {
        match self.providers.get(id) {
            Some(provider) => provider.get_token().await,
            None => Ok(AuthToken::None), // unregistered = no auth (local)
        }
    }

    /// List all registered providers
    pub fn list(&self) -> Vec<&ProviderId> {
        self.providers.keys().collect()
    }
}
```

### Provider Implementations

#### `providers/api_key.rs` — Static API Key (Phase 1, simplest)

```rust
/// Static API key stored in credential storage.
/// Reads once at construction, returns Bearer token forever.
pub struct ApiKeyProvider {
    provider_id: ProviderId,
    api_key: String,
    header_prefix: String,  // Default: "Bearer "
}
```
- No refresh, no expiry, no external calls
- Used for: Ollama Cloud, OpenRouter, most OpenAI-compat providers
- This replaces the current `api_key: Option<String>` in OllamaConfig

#### `providers/env_var.rs` — Environment Variable (Phase 1)

```rust
/// Reads API key from env var at request time (not cached).
/// Supports rotation: change the env var, next request uses new key.
pub struct EnvVarProvider {
    provider_id: ProviderId,
    var_name: String,
    header_prefix: String,
}
```
- Used for: CI/CD, dynamic key rotation, dev environments
- Replaces current `env::var("OLLAMA_API_KEY")` pattern

#### `providers/none.rs` — No Auth (Phase 1)

```rust
/// No authentication. For local endpoints.
pub struct NoAuthProvider {
    provider_id: ProviderId,
}
```
- Always returns `AuthToken::None`
- Used for: local Ollama (localhost:11434), local LiteLLM proxy

#### `providers/external_command.rs` — External Command (Phase 1)

```rust
/// Runs a shell command to get a bearer token.
/// The command MUST print a token to stdout and exit 0.
/// Tokens are cached until expiry (if parseable) or for 5 minutes.
pub struct ExternalCommandProvider {
    provider_id: ProviderId,
    command: String,
    args: Vec<String>,
    cached_token: Arc<Mutex<Option<CachedToken>>>,
}
```
- This is the **escape hatch** — any auth scheme becomes a shell command
- Used for: AgentAuth Ed25519 signing, custom OAuth flows, cloud provider CLI auth
- Pattern borrowed from Codex's `external_bearer.rs`

#### `providers/oauth.rs` — OAuth 2.0 + PKCE (Phase 2)

```rust
/// Full OAuth 2.0 + PKCE flow.
/// Opens browser for consent, receives callback, stores tokens, refreshes.
pub struct OAuthProvider {
    provider_id: ProviderId,
    client_id: String,
    auth_url: String,
    token_url: String,
    scopes: Vec<String>,
    storage: Arc<dyn CredentialStorage>,
    // Token refresh state
    tokens: Arc<AsyncMutex<Option<OAuthTokens>>>,
}
```
- Phase 2 — not needed for initial deployment
- Used for: Google API, MCP servers requiring OAuth, future provider integrations
- Pattern borrowed from both Codex (`pkce.rs`, `server.rs`) and Gemini CLI (`oauth-provider.ts`)

### Credential Storage (`storage/`)

#### `storage/mod.rs` — Trait

```rust
#[async_trait]
pub trait CredentialStorage: Send + Sync + std::fmt::Debug {
    /// Load credential for a provider
    async fn load(&self, provider_id: &ProviderId) -> Result<Option<Credential>, StorageError>;
    /// Save credential
    async fn save(&self, credential: &Credential) -> Result<(), StorageError>;
    /// Delete credential
    async fn delete(&self, provider_id: &ProviderId) -> Result<bool, StorageError>;
    /// List all stored provider IDs
    async fn list(&self) -> Result<Vec<ProviderId>, StorageError>;
}
```

#### `storage/file.rs` — JSON File Storage (Phase 1)

```rust
/// Stores credentials in config/credentials/auth.json with 0600 permissions.
/// Format: { "providers": { "ollama-cloud": { ... }, "openrouter": { ... } } }
pub struct FileStorage {
    path: PathBuf,
}
```
- File permissions: `0600` (owner read/write only) on Unix
- Path: `config/credentials/auth.json` (gitignored)
- Secrets encrypted at rest with a machine key (Phase 2; plaintext for Phase 1 with restricted perms)

#### `storage/memory.rs` — Ephemeral Storage

```rust
/// In-memory only. For testing and CI.
pub struct MemoryStorage {
    credentials: Arc<Mutex<HashMap<ProviderId, Credential>>>,
}
```

### Configuration (`config.rs`)

```rust
/// Provider auth config loaded from config/auth_providers.toml
///
/// Example:
/// ```toml
/// [[providers]]
/// id = "ollama-cloud"
/// auth_method = "env_var"
/// var_name = "OLLAMA_API_KEY"
///
/// [[providers]]
/// id = "openrouter"
/// auth_method = "env_var"
/// var_name = "OPENROUTER_API_KEY"
///
/// [[providers]]
/// id = "local-ollama"
/// auth_method = "none"
///
/// [[providers]]
/// id = "custom-endpoint"
/// auth_method = "external_command"
/// command = "/usr/local/bin/get-token"
/// args = ["--provider", "custom"]
/// ```
pub fn load_auth_config(path: &Path) -> Result<Vec<ProviderAuthConfig>, ConfigError>;
```

---

## 4. Integration Points with Other Agents

### mind-model-router (PRIMARY consumer)

**Current state**: `OllamaConfig.api_key: Option<String>` is passed to `OllamaClient`, which does `req.bearer_auth(key)`.

**Target state**: `OllamaClient` (and future `OpenAiCompatClient`) receives an `Arc<dyn AuthProvider>` instead of a raw API key.

```rust
// BEFORE (codex-llm/ollama.rs):
if let Some(ref key) = self.config.api_key {
    req = req.bearer_auth(key);
}

// AFTER:
let token = self.auth_provider.get_token().await?;
match token {
    AuthToken::Bearer(t) => { req = req.bearer_auth(&t); }
    AuthToken::CustomHeader { name, value } => { req = req.header(&name, &value); }
    AuthToken::None => {} // no auth needed
}
```

**Integration sequence:**
1. aiciv-auth provides `AuthProvider` trait + `ProviderRegistry`
2. mind-model-router's `ModelRouter` holds a `ProviderRegistry`
3. When routing a request to a provider, `ModelRouter` calls `registry.get_token(&provider_id)`
4. The returned `AuthToken` is injected into the HTTP request

**Interface Note for mind-model-router:**
- I will expose: `AuthProvider` trait, `AuthToken` enum, `ProviderRegistry`, `ProviderId`
- You consume: `registry.get_token(&provider_id) -> Result<AuthToken>`
- Migration path: Keep `api_key: Option<String>` working as `EnvVarProvider("OLLAMA_API_KEY")` wrapper — zero breaking changes for Phase 1

### mind-mcp (MCP server auth)

**What mind-mcp needs:**
- OAuth 2.0 + PKCE for remote MCP servers that require user consent
- API key auth for simple MCP servers
- Per-server auth configuration

**What aiciv-auth provides:**
- `OAuthProvider` (Phase 2) for MCP servers requiring OAuth
- `ApiKeyProvider` or `EnvVarProvider` for simple MCP servers
- `ProviderRegistry` scoped per MCP server (each server = its own ProviderId)

### mind-coordination (type definitions)

**Types I need added to codex-types:**
- `ProviderId(String)` — identifies a model provider
- `AuthMethod` enum — None, ApiKey, EnvVar, OAuth, ExternalCommand

**Alternative**: Self-contained types in aiciv-auth (no codex-types dependency). **I recommend self-contained** for Phase 1 — auth is a leaf crate with no reason to pull in the coordination layer. Types can be promoted to codex-types later if multiple crates need them.

### mind-hooks (auth event hooks)

**Proposed hooks:**
- `AuthRefreshed { provider_id, method }` — credential was refreshed
- `AuthFailed { provider_id, error }` — auth attempt failed

These are informational only (not blocking hooks). Phase 2.

---

## 5. Build Plan — Phased Implementation

### Phase 1: Foundation (Immediate — unblocks mind-model-router)

**Goal**: Replace `api_key: Option<String>` with `AuthProvider` trait without breaking anything.

1. Create `src/aiciv-auth/` crate skeleton
   - `Cargo.toml` with deps: `async-trait`, `serde`, `serde_json`, `tokio`, `tracing`, `thiserror`, `chrono`
   - `lib.rs` re-exporting core types and traits
   - `types.rs`, `provider.rs`, `registry.rs`

2. Implement 3 providers:
   - `NoAuthProvider` (trivial, ~20 lines)
   - `ApiKeyProvider` (static key, ~40 lines)
   - `EnvVarProvider` (reads env var per-request, ~50 lines)

3. Implement `FileStorage` for credential persistence
   - JSON file at `config/credentials/auth.json`
   - Unix 0600 permissions
   - Load/save/delete/list operations

4. Write unit tests for all providers + storage

5. **DO NOT modify codex-llm** — that's mind-model-router's crate. Instead, publish the trait and let them integrate.

**Estimated size**: ~400-500 lines of Rust

### Phase 2: External Commands + OAuth (Medium-term)

6. Implement `ExternalCommandProvider`
   - Subprocess execution with timeout
   - Token caching with TTL
   - Stderr capture for error reporting

7. Implement `OAuthProvider`
   - PKCE challenge/verifier generation
   - Local HTTP server for OAuth callback
   - Token persistence + refresh
   - Browser launch for consent

8. Auth config file loading (`config/auth_providers.toml`)

**Estimated size**: ~600-800 additional lines

### Phase 3: Advanced (Long-term)

9. **Ed25519 signing** — AgentAuth identity keys for inter-civ authentication
10. **Token rotation** — automatic key rotation with zero-downtime
11. **Auth health dashboard** — expiry warnings, failure rates, provider status
12. **Keychain integration** — OS keychain for secure credential storage

---

## 6. Dependency Graph

```
aiciv-auth (NEW — leaf crate, minimal dependencies)
├── async-trait         # AuthProvider trait
├── serde + serde_json  # Credential serialization
├── tokio               # Async runtime (subprocess, file I/O)
├── tracing             # Structured logging
├── thiserror           # Error types
├── chrono              # Token expiry timestamps
└── reqwest (Phase 2)   # OAuth token exchange HTTP calls

Does NOT depend on:
├── codex-types         # Self-contained types (no coordination layer dependency)
├── codex-llm           # Auth is consumed BY codex-llm, not the reverse
├── codex-exec          # No tool execution
├── codex-coordination  # No orchestration
└── codex-memory        # No memory access
```

**Why leaf crate**: Auth should have the FEWEST dependencies possible. Every dependency is an attack surface. Auth is consumed by other crates; it consumes nothing from the workspace.

---

## 7. How This Enables Multi-Provider Routing

This is the CRITICAL unlock. Without aiciv-auth, mind-model-router cannot implement the `LlmProvider` trait abstraction from their mind-map (Section 4).

**The flow:**

```
User request
    │
    ├─ ModelRouter selects provider (by role, complexity, or fallback)
    │   → ProviderId("ollama-cloud") or ProviderId("openrouter") or ProviderId("local")
    │
    ├─ ModelRouter calls ProviderRegistry.get_token(provider_id)
    │   → AuthToken::Bearer("sk-or-...") or AuthToken::None
    │
    ├─ LlmProvider.chat() injects AuthToken into HTTP request
    │   → req.bearer_auth(token)
    │
    └─ Response flows back through ThinkLoop
```

**Without aiciv-auth**: Every new provider requires hardcoding another env var check in codex-llm. With 5 providers, that's 5 different auth paths in the LLM client. With aiciv-auth: one call to `get_token()`, every provider works the same way.

---

## 8. Configuration Example

```toml
# config/auth_providers.toml

# Ollama Cloud — reads API key from OLLAMA_API_KEY env var
[[providers]]
id = "ollama-cloud"
auth_method = "env_var"
var_name = "OLLAMA_API_KEY"

# OpenRouter — reads from OPENROUTER_API_KEY
[[providers]]
id = "openrouter"
auth_method = "env_var"
var_name = "OPENROUTER_API_KEY"

# Local Ollama — no auth
[[providers]]
id = "local-ollama"
auth_method = "none"

# LiteLLM proxy — static API key
[[providers]]
id = "litellm"
auth_method = "api_key"
# Key stored in config/credentials/auth.json, not in this file

# Custom provider with external auth command
[[providers]]
id = "agentauth-signed"
auth_method = "external_command"
command = "aiciv-auth-sign"
args = ["--key", "config/client-keys/role-keys/acg/primary/private.pem"]
```

---

## 9. Anti-Patterns to Avoid

| Anti-Pattern | Why It's Bad | What to Do Instead |
|-------------|-------------|-------------------|
| Hardcoding provider URLs in auth | Vendor lock-in | Auth is credential-only; URLs belong in model-router config |
| Storing secrets in config files checked into git | Security breach | Use `config/credentials/` (gitignored) with 0600 perms |
| Logging auth tokens | Security breach | Use `tracing::debug!` with `%provider_id` only, NEVER log the token value |
| Making auth synchronous | Blocks the ThinkLoop | All auth ops are `async` — token refresh, file I/O, external commands |
| Coupling to a specific HTTP client | Limits composability | `AuthProvider` returns `AuthToken`, not `reqwest::RequestBuilder` |
| Refreshing tokens on every request | Performance waste | Cache tokens, refresh only on expiry or 401 response |

---

## 10. Comparison: Our Design vs. Sources

| Feature | Codex Login | Gemini CLI | aiciv-auth (ours) |
|---------|-------------|-----------|-------------------|
| **Primary auth** | OpenAI API key / ChatGPT OAuth | Google OAuth | ANY provider via config |
| **Provider abstraction** | `CodexAuth` enum (3 variants) | Per-service auth providers | `AuthProvider` trait (open-ended) |
| **Storage** | File + Keyring + Ephemeral | File + Keychain | File + Memory (Phase 1), Keychain (Phase 3) |
| **External command** | Yes (BearerTokenRefresher) | No | Yes (ExternalCommandProvider) |
| **OAuth** | PKCE + Device Code | PKCE + SA Impersonation | PKCE (Phase 2) |
| **Multi-provider** | Single provider per session | Yes (per MCP server) | Yes (ProviderRegistry) |
| **Vendor lock-in** | OpenAI-locked | Google-locked | **NONE** — config-driven |
| **Lines of code** | ~8,000 | ~2,000 | ~500 (Phase 1), ~1,300 (Phase 2) |

**Our advantage**: Codex assumes OpenAI. Gemini CLI assumes Google. We assume NOTHING. Every provider is configured the same way. This is what makes aiciv-mind a true platform.

---

*mind-auth | 2026-04-16*
