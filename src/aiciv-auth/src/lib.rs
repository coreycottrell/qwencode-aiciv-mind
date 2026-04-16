//! # aiciv-auth — Provider Authentication for aiciv-mind
//!
//! Composable authentication providers for LLM APIs, MCP servers, and
//! external services. Every provider implements the `AuthProvider` trait,
//! which yields an HTTP Authorization header value on demand.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐     ┌──────────────────┐
//! │ AuthProvider │◄────│  ApiKeyProvider   │  (env var / storage / explicit)
//! │   trait      │     └──────────────────┘
//! │              │     ┌──────────────────┐
//! │              │◄────│  (future: OAuth)  │
//! │              │     └──────────────────┘
//! │              │     ┌──────────────────┐
//! │              │◄────│  (future: Ed25519)│
//! └─────────────┘     └──────────────────┘
//!         │
//!         ▼
//!  auth_header() → Option<"Bearer sk-...">
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use aiciv_auth::providers::ApiKeyProvider;
//! use aiciv_auth::AuthProvider;
//!
//! # async fn example() -> Result<(), aiciv_auth::AuthError> {
//! let provider = ApiKeyProvider::with_key("openai", "sk-test123");
//! let header = provider.auth_header().await?;
//! assert_eq!(header, Some("Bearer sk-test123".to_string()));
//! # Ok(())
//! # }
//! ```

pub mod providers;
pub mod storage;
pub mod types;

use async_trait::async_trait;

// Re-export key types for convenience
pub use providers::ApiKeyProvider;
pub use storage::CredentialStorage;
pub use types::{AuthToken, Credential, ProviderConfig};

/// Errors that can occur during authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// A required API key was not found in any configured source.
    #[error("no API key found for provider '{provider}': checked env var ({env_var:?}), storage, and explicit config")]
    MissingKey {
        provider: String,
        env_var: Option<String>,
    },

    /// Credential storage I/O error.
    #[error("storage error at '{path}': {detail}")]
    StorageError { path: String, detail: String },

    /// An unexpected internal error.
    #[error("auth error: {0}")]
    Internal(String),
}

/// The core authentication trait.
///
/// Every auth provider implements this. Consumers call `auth_header()` to get
/// the value for the HTTP `Authorization` header (or `None` if no auth is needed).
///
/// Providers are `Send + Sync` so they can be shared across async tasks.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get the authorization header value for a request.
    ///
    /// Returns `Some("Bearer sk-...")` if credentials are available,
    /// or `None` if this provider has no credentials configured.
    async fn auth_header(&self) -> Result<Option<String>, AuthError>;

    /// Provider name for diagnostics and logging.
    fn provider_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AuthProvider trait tests ─────────────────────────────────────────

    #[tokio::test]
    async fn api_key_provider_returns_bearer_header() {
        let provider = ApiKeyProvider::with_key("openai", "sk-test-key-123");
        let header = provider.auth_header().await.unwrap();
        assert_eq!(header, Some("Bearer sk-test-key-123".to_string()));
    }

    #[tokio::test]
    async fn api_key_provider_name_matches() {
        let provider = ApiKeyProvider::with_key("anthropic", "key");
        assert_eq!(provider.provider_name(), "anthropic");
    }

    #[tokio::test]
    async fn api_key_provider_no_key_returns_none() {
        // No env var, no storage, no explicit key
        let config = ProviderConfig {
            name: "missing".to_string(),
            env_var: None,
            header_prefix: "Bearer".to_string(),
        };
        let provider = ApiKeyProvider::new(config, None, None);
        let header = provider.auth_header().await.unwrap();
        assert_eq!(header, None);
    }

    #[tokio::test]
    async fn api_key_provider_env_var_fallback() {
        // Set an env var and verify it is picked up
        let var_name = "AICIV_AUTH_TEST_KEY_12345";
        unsafe {
            std::env::set_var(var_name, "env-secret-value");
        }
        let provider = ApiKeyProvider::from_env("test-provider", var_name);
        let header = provider.auth_header().await.unwrap();
        assert_eq!(header, Some("Bearer env-secret-value".to_string()));
        // Clean up
        unsafe {
            std::env::remove_var(var_name);
        }
    }

    #[tokio::test]
    async fn env_var_takes_priority_over_explicit_key() {
        let var_name = "AICIV_AUTH_PRIORITY_TEST_67890";
        unsafe {
            std::env::set_var(var_name, "from-env");
        }
        let config = ProviderConfig {
            name: "priority-test".to_string(),
            env_var: Some(var_name.to_string()),
            header_prefix: "Bearer".to_string(),
        };
        let provider = ApiKeyProvider::new(config, None, Some("from-explicit".to_string()));
        let header = provider.auth_header().await.unwrap();
        // Env var should win over explicit
        assert_eq!(header, Some("Bearer from-env".to_string()));
        unsafe {
            std::env::remove_var(var_name);
        }
    }

    // ── CredentialStorage tests ─────────────────────────────────────────

    #[test]
    fn storage_write_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());

        let cred = Credential {
            provider: "openai".to_string(),
            secret: "sk-abc123".to_string(),
            label: Some("production key".to_string()),
        };
        storage.write(&cred).unwrap();

        let loaded = storage.read("openai").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.provider, "openai");
        assert_eq!(loaded.secret, "sk-abc123");
        assert_eq!(loaded.label.as_deref(), Some("production key"));
    }

    #[test]
    fn storage_read_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());

        let result = storage.read("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn storage_delete() {
        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());

        let cred = Credential {
            provider: "ollama".to_string(),
            secret: "key-to-delete".to_string(),
            label: None,
        };
        storage.write(&cred).unwrap();
        assert!(storage.read("ollama").unwrap().is_some());

        storage.delete("ollama").unwrap();
        assert!(storage.read("ollama").unwrap().is_none());
    }

    #[test]
    fn storage_delete_nonexistent_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());
        // Should not error
        storage.delete("ghost").unwrap();
    }

    #[test]
    fn storage_list_providers() {
        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());

        for name in &["anthropic", "openai", "ollama"] {
            storage
                .write(&Credential {
                    provider: name.to_string(),
                    secret: format!("key-{name}"),
                    label: None,
                })
                .unwrap();
        }

        let mut providers = storage.list_providers().unwrap();
        providers.sort();
        assert_eq!(providers, vec!["anthropic", "ollama", "openai"]);
    }

    #[test]
    fn storage_read_all() {
        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());

        storage
            .write(&Credential {
                provider: "a".to_string(),
                secret: "secret-a".to_string(),
                label: None,
            })
            .unwrap();
        storage
            .write(&Credential {
                provider: "b".to_string(),
                secret: "secret-b".to_string(),
                label: None,
            })
            .unwrap();

        let all = storage.read_all().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all["a"].secret, "secret-a");
        assert_eq!(all["b"].secret, "secret-b");
    }

    #[cfg(unix)]
    #[test]
    fn storage_sets_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());

        storage
            .write(&Credential {
                provider: "secure".to_string(),
                secret: "top-secret".to_string(),
                label: None,
            })
            .unwrap();

        let path = dir.path().join("secure.json");
        let perms = std::fs::metadata(&path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o600);
    }

    #[test]
    fn storage_list_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());
        let providers = storage.list_providers().unwrap();
        assert!(providers.is_empty());
    }

    #[test]
    fn storage_list_nonexistent_dir() {
        let storage = CredentialStorage::new("/tmp/aiciv-auth-test-nonexistent-dir-98765");
        let providers = storage.list_providers().unwrap();
        assert!(providers.is_empty());
    }

    // ── Integration: ApiKeyProvider + CredentialStorage ──────────────────

    #[tokio::test]
    async fn api_key_from_storage() {
        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());

        storage
            .write(&Credential {
                provider: "stored-provider".to_string(),
                secret: "stored-secret".to_string(),
                label: None,
            })
            .unwrap();

        let read_storage = CredentialStorage::new(dir.path());
        let config = ProviderConfig {
            name: "stored-provider".to_string(),
            env_var: None,
            header_prefix: "Bearer".to_string(),
        };
        let provider = ApiKeyProvider::new(config, Some(read_storage), None);
        let header = provider.auth_header().await.unwrap();
        assert_eq!(header, Some("Bearer stored-secret".to_string()));
    }

    #[tokio::test]
    async fn storage_key_used_when_env_empty() {
        let dir = tempfile::tempdir().unwrap();
        let storage = CredentialStorage::new(dir.path());

        storage
            .write(&Credential {
                provider: "fallback-test".to_string(),
                secret: "from-storage".to_string(),
                label: None,
            })
            .unwrap();

        // Set env var to empty string — should fall through to storage
        let var_name = "AICIV_AUTH_EMPTY_VAR_TEST_54321";
        unsafe {
            std::env::set_var(var_name, "");
        }

        let read_storage = CredentialStorage::new(dir.path());
        let config = ProviderConfig {
            name: "fallback-test".to_string(),
            env_var: Some(var_name.to_string()),
            header_prefix: "Bearer".to_string(),
        };
        let provider = ApiKeyProvider::new(config, Some(read_storage), None);
        let header = provider.auth_header().await.unwrap();
        assert_eq!(header, Some("Bearer from-storage".to_string()));

        unsafe {
            std::env::remove_var(var_name);
        }
    }

    // ── Type serialization tests ────────────────────────────────────────

    #[test]
    fn credential_roundtrip_json() {
        let cred = Credential {
            provider: "test".to_string(),
            secret: "s3cret".to_string(),
            label: Some("my key".to_string()),
        };
        let json = serde_json::to_string(&cred).unwrap();
        let parsed: Credential = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.provider, "test");
        assert_eq!(parsed.secret, "s3cret");
        assert_eq!(parsed.label.as_deref(), Some("my key"));
    }

    #[test]
    fn credential_without_label() {
        let json = r#"{"provider":"x","secret":"y"}"#;
        let cred: Credential = serde_json::from_str(json).unwrap();
        assert_eq!(cred.provider, "x");
        assert_eq!(cred.secret, "y");
        assert!(cred.label.is_none());
    }

    #[test]
    fn provider_config_default_prefix() {
        let json = r#"{"name":"openai"}"#;
        let config: ProviderConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.header_prefix, "Bearer");
    }

    #[test]
    fn auth_token_roundtrip() {
        let token = AuthToken {
            header_value: "Bearer abc".to_string(),
            provider_name: "test".to_string(),
        };
        let json = serde_json::to_string(&token).unwrap();
        let parsed: AuthToken = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.header_value, "Bearer abc");
        assert_eq!(parsed.provider_name, "test");
    }
}
