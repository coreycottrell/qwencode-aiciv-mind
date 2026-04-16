//! # types — Auth tokens, credentials, and provider configuration
//!
//! Shared types for the aiciv-auth crate. These are the data structures
//! that flow between providers, storage, and consumers.

use serde::{Deserialize, Serialize};

/// A resolved authentication token ready to be used in an HTTP header.
///
/// This is the output of an AuthProvider — the thing that actually goes
/// on the wire. Consumers never need to know how it was obtained.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// The header value (e.g., "Bearer sk-abc123").
    pub header_value: String,
    /// Which provider issued this token, for diagnostics.
    pub provider_name: String,
}

/// A stored credential for a named provider.
///
/// This is what CredentialStorage persists to disk. It holds the raw
/// secret material (API key, refresh token, etc.) plus metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// The provider this credential belongs to (e.g., "openai", "anthropic").
    pub provider: String,
    /// The secret value (API key, token, etc.).
    pub secret: String,
    /// Optional label for human identification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Configuration for a single auth provider instance.
///
/// Tells the provider where to look for credentials and how to
/// format the auth header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name (e.g., "openai", "ollama", "anthropic").
    pub name: String,
    /// Environment variable to check for the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env_var: Option<String>,
    /// Header prefix (defaults to "Bearer" if not set).
    #[serde(default = "default_header_prefix")]
    pub header_prefix: String,
}

fn default_header_prefix() -> String {
    "Bearer".to_string()
}
