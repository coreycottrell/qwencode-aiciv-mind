//! # api_key — API key authentication provider
//!
//! The simplest auth provider: reads an API key from one of three sources
//! (in priority order):
//!
//! 1. Environment variable (e.g., `OPENAI_API_KEY`)
//! 2. Credential storage (file-based)
//! 3. Explicit key passed at construction
//!
//! Returns an `Authorization: Bearer {key}` header (prefix is configurable).

use async_trait::async_trait;

use crate::storage::CredentialStorage;
use crate::types::ProviderConfig;
use crate::{AuthError, AuthProvider};

/// API key authentication provider.
///
/// Resolves an API key from environment, storage, or an explicit value,
/// then formats it as an HTTP Authorization header.
pub struct ApiKeyProvider {
    config: ProviderConfig,
    storage: Option<CredentialStorage>,
    explicit_key: Option<String>,
}

impl ApiKeyProvider {
    /// Create a new provider with the given configuration.
    ///
    /// `storage` is optional — if `None`, only env var and explicit key are checked.
    pub fn new(
        config: ProviderConfig,
        storage: Option<CredentialStorage>,
        explicit_key: Option<String>,
    ) -> Self {
        Self {
            config,
            storage,
            explicit_key,
        }
    }

    /// Create a minimal provider with just a name and explicit key.
    ///
    /// Useful for tests and simple configurations.
    pub fn with_key(name: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig {
                name: name.into(),
                env_var: None,
                header_prefix: "Bearer".to_string(),
            },
            storage: None,
            explicit_key: Some(key.into()),
        }
    }

    /// Create a provider that reads from an environment variable.
    pub fn from_env(name: impl Into<String>, env_var: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig {
                name: name.into(),
                env_var: Some(env_var.into()),
                header_prefix: "Bearer".to_string(),
            },
            storage: None,
            explicit_key: None,
        }
    }

    /// Resolve the API key from the configured sources.
    ///
    /// Priority: env var > storage > explicit key.
    fn resolve_key(&self) -> Result<Option<String>, AuthError> {
        // 1. Check environment variable
        if let Some(ref var_name) = self.config.env_var {
            if let Ok(val) = std::env::var(var_name) {
                if !val.is_empty() {
                    tracing::debug!(
                        provider = %self.config.name,
                        source = "env",
                        var = %var_name,
                        "API key resolved from environment"
                    );
                    return Ok(Some(val));
                }
            }
        }

        // 2. Check credential storage
        if let Some(ref storage) = self.storage {
            if let Some(cred) = storage.read(&self.config.name)? {
                tracing::debug!(
                    provider = %self.config.name,
                    source = "storage",
                    "API key resolved from credential storage"
                );
                return Ok(Some(cred.secret));
            }
        }

        // 3. Use explicit key
        if let Some(ref key) = self.explicit_key {
            tracing::debug!(
                provider = %self.config.name,
                source = "explicit",
                "API key resolved from explicit configuration"
            );
            return Ok(Some(key.clone()));
        }

        Ok(None)
    }
}

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    async fn auth_header(&self) -> Result<Option<String>, AuthError> {
        match self.resolve_key()? {
            Some(key) => {
                let header = format!("{} {}", self.config.header_prefix, key);
                Ok(Some(header))
            }
            None => Ok(None),
        }
    }

    fn provider_name(&self) -> &str {
        &self.config.name
    }
}
