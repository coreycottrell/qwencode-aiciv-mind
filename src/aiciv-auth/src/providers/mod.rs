//! # providers — Authentication provider implementations
//!
//! Each provider implements the `AuthProvider` trait from the crate root.
//! Currently ships with `ApiKeyProvider`; more providers (OAuth, AgentAuth
//! Ed25519 signing) will be added as the platform evolves.

pub mod api_key;

pub use api_key::ApiKeyProvider;
