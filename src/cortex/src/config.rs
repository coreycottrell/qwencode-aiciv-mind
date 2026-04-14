//! Cortex configuration — parsed from `config/config.toml`.
//!
//! Replaces hardcoded model names, URLs, and thresholds with a TOML config file.
//! Falls back to environment variables, then to compiled defaults.

use codex_llm::ollama::ModelRouter;
use serde::Deserialize;
use std::path::Path;

/// Top-level Cortex configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct CortexConfig {
    /// Model provider definitions (ollama_local, ollama_cloud, etc.)
    #[serde(default)]
    pub model_providers: ModelProviders,

    /// Coordination settings — model mappings, thresholds, dream schedule.
    #[serde(default)]
    pub coordination: CoordinationConfig,

    /// AiCIV Suite integration URLs.
    #[serde(default)]
    pub suite: SuiteConfig,
}

/// Named model providers.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModelProviders {
    pub ollama_local: Option<ProviderConfig>,
    pub ollama_cloud: Option<ProviderConfig>,
}

/// A single model provider.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub base_url: String,
    #[allow(dead_code)]
    pub wire_api: Option<String>,
}

/// Coordination engine settings.
#[derive(Debug, Clone, Deserialize)]
pub struct CoordinationConfig {
    /// Model for Primary minds.
    #[serde(default = "default_primary_model")]
    pub primary_model: String,
    /// Model for TeamLead minds.
    #[serde(default = "default_team_lead_model")]
    pub team_lead_model: String,
    /// Model for Agent minds.
    #[serde(default = "default_agent_model")]
    pub agent_model: String,
    /// Lightweight model for red team, memory scoring.
    #[serde(default = "default_lightweight_model")]
    pub lightweight_model: String,
    /// Model for dream consolidation.
    #[serde(default = "default_dream_model")]
    pub dream_model: String,
    /// Model for memory extraction.
    #[serde(default = "default_memory_extraction_model")]
    pub memory_extraction_model: String,

    /// Local fallback models (no API key needed).
    #[serde(default)]
    pub local_fallback: LocalFallbackConfig,

    /// Planning complexity threshold.
    #[serde(default = "default_planning_threshold")]
    pub planning_spawn_threshold: String,

    /// Pattern repetition threshold before spawning specialist.
    #[serde(default = "default_pattern_threshold")]
    pub pattern_repetition_threshold: u32,
    /// Seconds stuck before spawning fresh context.
    #[serde(default = "default_blocking_threshold")]
    pub blocking_threshold_secs: u64,
    /// Context pressure percentage before overflow.
    #[serde(default = "default_context_pressure")]
    pub context_pressure_threshold: f64,

    /// Scratchpad rotation interval in hours.
    #[serde(default = "default_rotation_hours")]
    pub rotation_interval_hours: u32,

    /// Dream mode schedule.
    #[serde(default = "default_dream_start")]
    pub dream_start_time: String,
    #[serde(default = "default_dream_end")]
    pub dream_end_time: String,
    /// Memory depth score below which entries are archive candidates.
    #[serde(default = "default_archive_threshold")]
    pub memory_archive_threshold: f64,
}

/// Local fallback model assignments.
#[derive(Debug, Clone, Deserialize)]
pub struct LocalFallbackConfig {
    #[serde(default = "default_local_primary")]
    pub primary_model: String,
    #[serde(default = "default_local_team_lead")]
    pub team_lead_model: String,
    #[serde(default = "default_local_agent")]
    pub agent_model: String,
    #[serde(default = "default_local_lightweight")]
    pub lightweight_model: String,
}

impl Default for LocalFallbackConfig {
    fn default() -> Self {
        Self {
            primary_model: default_local_primary(),
            team_lead_model: default_local_team_lead(),
            agent_model: default_local_agent(),
            lightweight_model: default_local_lightweight(),
        }
    }
}

/// AiCIV Suite integration endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct SuiteConfig {
    #[serde(default = "default_auth_url")]
    pub auth_url: String,
    #[serde(default = "default_hub_url")]
    pub hub_url: String,
    #[serde(default = "default_cal_url")]
    pub cal_url: String,
    #[serde(default = "default_keypair")]
    pub default_keypair: String,
}

impl Default for SuiteConfig {
    fn default() -> Self {
        Self {
            auth_url: default_auth_url(),
            hub_url: default_hub_url(),
            cal_url: default_cal_url(),
            default_keypair: default_keypair(),
        }
    }
}

impl SuiteConfig {
    /// Build an `AuthClient` from this config.
    pub fn auth_client(&self) -> codex_suite_client::AuthClient {
        codex_suite_client::AuthClient::new(&self.auth_url, &self.default_keypair)
    }

    /// Build a `HubClient` from this config.
    ///
    /// If `HUB_JWT_TOKEN` env var is set, the client is authenticated.
    /// Otherwise, the client works for unauthenticated endpoints (feed, read).
    pub fn hub_client(&self) -> codex_suite_client::HubClient {
        let mut client = codex_suite_client::HubClient::new(&self.hub_url);
        if let Ok(token) = std::env::var("HUB_JWT_TOKEN") {
            if !token.is_empty() {
                tracing::info!("HubClient: using HUB_JWT_TOKEN for authentication");
                client = client.with_token(token);
            }
        }
        client
    }

    /// Build an authenticated `HubClient` using Ed25519 challenge-response.
    ///
    /// Logs into AgentAuth, gets a JWT, and injects it into the HubClient.
    /// Requires `civ_id` and `private_key_b64` (32-byte Ed25519 seed, base64).
    pub async fn authenticated_hub_client(
        &self,
        civ_id: &str,
        private_key_b64: &str,
    ) -> Result<codex_suite_client::HubClient, codex_suite_client::SuiteError> {
        let mut auth = self.auth_client();
        let token = auth.login(civ_id, private_key_b64).await?;
        Ok(codex_suite_client::HubClient::new(&self.hub_url).with_token(token))
    }

    /// Build a `HubInterceptor` from this config, ready for ThinkLoop injection.
    pub fn hub_interceptor(&self) -> codex_suite_client::HubInterceptor {
        codex_suite_client::HubInterceptor::new(self.hub_client())
    }

    /// Build an authenticated `HubInterceptor` using Ed25519 challenge-response.
    pub async fn authenticated_hub_interceptor(
        &self,
        civ_id: &str,
        private_key_b64: &str,
    ) -> Result<codex_suite_client::HubInterceptor, codex_suite_client::SuiteError> {
        let client = self.authenticated_hub_client(civ_id, private_key_b64).await?;
        Ok(codex_suite_client::HubInterceptor::new(client))
    }
}

// ── Default value functions ─────────────────────────────────────────────────

fn default_primary_model() -> String { "devstral-small-2:24b".into() }
fn default_team_lead_model() -> String { "devstral-small-2:24b".into() }
fn default_agent_model() -> String { "devstral-small-2:24b".into() }
fn default_lightweight_model() -> String { "minimax-m2.7".into() }
fn default_dream_model() -> String { "devstral-small-2:24b".into() }
fn default_memory_extraction_model() -> String { "minimax-m2.7".into() }

fn default_local_primary() -> String { "qwen2.5:7b".into() }
fn default_local_team_lead() -> String { "qwen2.5:7b".into() }
fn default_local_agent() -> String { "qwen2.5:7b".into() }
fn default_local_lightweight() -> String { "phi3:mini".into() }

fn default_planning_threshold() -> String { "complex".into() }
fn default_pattern_threshold() -> u32 { 3 }
fn default_blocking_threshold() -> u64 { 120 }
fn default_context_pressure() -> f64 { 85.0 }
fn default_rotation_hours() -> u32 { 3 }
fn default_dream_start() -> String { "01:00".into() }
fn default_dream_end() -> String { "04:00".into() }
fn default_archive_threshold() -> f64 { 0.1 }

fn default_auth_url() -> String { "http://5.161.90.32:8700".into() }
fn default_hub_url() -> String { "http://87.99.131.49:8900".into() }
fn default_cal_url() -> String { "http://localhost:8800".into() }
fn default_keypair() -> String { "acg/primary".into() }

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            primary_model: default_primary_model(),
            team_lead_model: default_team_lead_model(),
            agent_model: default_agent_model(),
            lightweight_model: default_lightweight_model(),
            dream_model: default_dream_model(),
            memory_extraction_model: default_memory_extraction_model(),
            local_fallback: LocalFallbackConfig::default(),
            planning_spawn_threshold: default_planning_threshold(),
            pattern_repetition_threshold: default_pattern_threshold(),
            blocking_threshold_secs: default_blocking_threshold(),
            context_pressure_threshold: default_context_pressure(),
            rotation_interval_hours: default_rotation_hours(),
            dream_start_time: default_dream_start(),
            dream_end_time: default_dream_end(),
            memory_archive_threshold: default_archive_threshold(),
        }
    }
}

impl CortexConfig {
    /// Load config from a TOML file. Falls back to defaults if file doesn't exist.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            tracing::warn!(path = %path.display(), "Config file not found, using defaults");
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Io(path.display().to_string(), e.to_string()))?;

        let config: CortexConfig = toml::from_str(&contents)
            .map_err(|e| ConfigError::Parse(path.display().to_string(), e.to_string()))?;

        tracing::info!(path = %path.display(), "Loaded config");
        Ok(config)
    }

    /// Search for config.toml by walking up from the given directory.
    pub fn find_and_load(start_dir: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let mut dir = start_dir.as_ref().to_path_buf();
        loop {
            let candidate = dir.join("config").join("config.toml");
            if candidate.exists() {
                return Self::load(&candidate);
            }
            // Also check config.toml directly in the directory
            let direct = dir.join("config.toml");
            if direct.exists() {
                return Self::load(&direct);
            }
            if !dir.pop() {
                break;
            }
        }
        tracing::warn!("No config.toml found, using defaults");
        Ok(Self::default())
    }

    /// Build a `ModelRouter` from this config.
    ///
    /// If OLLAMA_API_KEY is set, uses cloud provider settings.
    /// Otherwise, uses local fallback models.
    pub fn model_router(&self) -> ModelRouter {
        let api_key = std::env::var("OLLAMA_API_KEY").ok().filter(|k| !k.is_empty());

        if let Some(key) = api_key {
            // Cloud mode — use cloud provider URL + coordination models
            let base_url = self.model_providers.ollama_cloud
                .as_ref()
                .map(|p| p.base_url.clone())
                .unwrap_or_else(|| "https://api.ollama.com".into());

            ModelRouter {
                primary_model: self.coordination.primary_model.clone(),
                team_lead_model: self.coordination.team_lead_model.clone(),
                agent_model: self.coordination.agent_model.clone(),
                lightweight_model: self.coordination.lightweight_model.clone(),
                base_url,
                api_key: Some(key),
            }
        } else {
            // Local mode — use local provider URL + fallback models
            let base_url = self.model_providers.ollama_local
                .as_ref()
                .map(|p| p.base_url.clone())
                .unwrap_or_else(|| "http://localhost:11434".into());

            ModelRouter {
                primary_model: self.coordination.local_fallback.primary_model.clone(),
                team_lead_model: self.coordination.local_fallback.team_lead_model.clone(),
                agent_model: self.coordination.local_fallback.agent_model.clone(),
                lightweight_model: self.coordination.local_fallback.lightweight_model.clone(),
                base_url,
                api_key: None,
            }
        }
    }
}

impl Default for CortexConfig {
    fn default() -> Self {
        Self {
            model_providers: ModelProviders::default(),
            coordination: CoordinationConfig::default(),
            suite: SuiteConfig::default(),
        }
    }
}

/// Config loading errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Cannot read config file {0}: {1}")]
    Io(String, String),
    #[error("Cannot parse config file {0}: {1}")]
    Parse(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_config() {
        let toml_str = r#"
[model_providers.ollama_local]
name = "Ollama Local"
base_url = "http://localhost:11434"
wire_api = "native"

[model_providers.ollama_cloud]
name = "Ollama Cloud"
base_url = "https://api.ollama.com"
wire_api = "native"

[coordination]
primary_model = "devstral-small-2:24b"
team_lead_model = "devstral-small-2:24b"
agent_model = "devstral-small-2:24b"
lightweight_model = "minimax-m2.7"
dream_model = "devstral-small-2:24b"
memory_extraction_model = "minimax-m2.7"
planning_spawn_threshold = "complex"
pattern_repetition_threshold = 3
blocking_threshold_secs = 120
context_pressure_threshold = 85.0
rotation_interval_hours = 3
dream_start_time = "01:00"
dream_end_time = "04:00"
memory_archive_threshold = 0.1

[coordination.local_fallback]
primary_model = "qwen2.5:7b"
team_lead_model = "qwen2.5:7b"
agent_model = "qwen2.5:7b"
lightweight_model = "phi3:mini"

[suite]
auth_url = "http://5.161.90.32:8700"
hub_url = "http://87.99.131.49:8900"
cal_url = "http://localhost:8800"
default_keypair = "acg/primary"
"#;

        let config: CortexConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.coordination.primary_model, "devstral-small-2:24b");
        assert_eq!(config.coordination.lightweight_model, "minimax-m2.7");
        assert_eq!(config.coordination.local_fallback.primary_model, "qwen2.5:7b");
        assert_eq!(config.coordination.dream_start_time, "01:00");
        assert_eq!(config.coordination.pattern_repetition_threshold, 3);
        assert_eq!(config.coordination.blocking_threshold_secs, 120);
        assert_eq!(config.coordination.context_pressure_threshold, 85.0);
        assert_eq!(config.suite.auth_url, "http://5.161.90.32:8700");
        assert_eq!(config.suite.default_keypair, "acg/primary");

        let cloud = config.model_providers.ollama_cloud.unwrap();
        assert_eq!(cloud.base_url, "https://api.ollama.com");
    }

    #[test]
    fn defaults_when_empty() {
        let config: CortexConfig = toml::from_str("").unwrap();
        assert_eq!(config.coordination.primary_model, "devstral-small-2:24b");
        assert_eq!(config.coordination.local_fallback.lightweight_model, "phi3:mini");
        assert_eq!(config.suite.hub_url, "http://87.99.131.49:8900");
    }

    #[test]
    fn partial_override() {
        let toml_str = r#"
[coordination]
primary_model = "llama3.3:70b"
"#;
        let config: CortexConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.coordination.primary_model, "llama3.3:70b");
        // Unset fields keep defaults
        assert_eq!(config.coordination.agent_model, "devstral-small-2:24b");
        assert_eq!(config.coordination.lightweight_model, "minimax-m2.7");
    }

    #[test]
    fn model_router_local_mode() {
        let config: CortexConfig = toml::from_str(r#"
[model_providers.ollama_local]
name = "Local"
base_url = "http://localhost:11434"

[coordination.local_fallback]
primary_model = "qwen2.5:7b"
lightweight_model = "phi3:mini"
"#).unwrap();

        // No OLLAMA_API_KEY set → local mode
        // (Can't fully test without env manipulation, but struct builds correctly)
        let router = config.model_router();
        // Without API key env var, should use local fallback
        assert!(router.api_key.is_none());
        assert_eq!(router.primary_model, "qwen2.5:7b");
    }

    #[test]
    fn missing_file_returns_defaults() {
        let config = CortexConfig::load("/nonexistent/path/config.toml").unwrap();
        assert_eq!(config.coordination.primary_model, "devstral-small-2:24b");
    }
}
