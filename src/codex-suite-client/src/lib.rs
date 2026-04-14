//! # codex-suite-client — Native Service Integration (Principle 12)
//!
//! Hub, AgentAuth, AgentCal are not external services — they are the mind's
//! native environment. Every mind gets a SuiteClient at birth.

pub mod elevenlabs_interceptor;
pub mod hub_interceptor;
pub mod image_gen_interceptor;
pub mod search_interceptor;

use base64::Engine;
use ed25519_dalek::{SigningKey, Signer};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

pub use elevenlabs_interceptor::ElevenLabsInterceptor;
pub use hub_interceptor::HubInterceptor;
pub use image_gen_interceptor::ImageGenInterceptor;
pub use search_interceptor::SearchInterceptor;

/// The unified client for all AiCIV Suite services.
/// Injected into every mind at spawn time.
pub struct SuiteClient {
    pub auth: AuthClient,
    pub hub: HubClient,
    pub cal: CalClient,
    pub config: SuiteConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteConfig {
    pub auth_url: String,
    pub hub_url: String,
    pub cal_url: String,
    pub keypair_id: String,
}

impl SuiteClient {
    /// Connect to the suite using a role keypair.
    ///
    /// Each mind gets its own identity:
    /// "acg/primary", "acg/research-lead", "acg/coder-1"
    pub fn new(config: SuiteConfig) -> Self {
        Self {
            auth: AuthClient::new(&config.auth_url, &config.keypair_id),
            hub: HubClient::new(&config.hub_url),
            cal: CalClient::new(&config.cal_url),
            config,
        }
    }
}

// ── AuthClient ──────────────────────────────────────────────────────────────

/// AgentAuth client — Ed25519 JWT issuance via challenge-response.
///
/// Auth flow:
/// 1. POST /challenge with civ_id → get base64 challenge
/// 2. Sign challenge bytes with Ed25519 private key
/// 3. POST /verify with civ_id + base64 signature → get JWT (1hr TTL)
///
/// After first `login()`, credentials are stored so `refresh()` can re-authenticate
/// without the caller re-supplying them.
pub struct AuthClient {
    base_url: String,
    keypair_id: String,
    token: Option<String>,
    /// When the token was issued (for staleness checks).
    token_issued_at: Option<std::time::Instant>,
    /// Stored credentials from the last successful login (for refresh).
    credentials: Option<(String, String)>, // (civ_id, private_key_b64)
    http: reqwest::Client,
}

impl AuthClient {
    pub fn new(base_url: &str, keypair_id: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            keypair_id: keypair_id.to_string(),
            token: None,
            token_issued_at: None,
            credentials: None,
            http: reqwest::Client::new(),
        }
    }

    /// Set a pre-signed JWT token (injected by the harness at spawn time).
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self.token_issued_at = Some(std::time::Instant::now());
        self
    }

    pub fn keypair_id(&self) -> &str {
        &self.keypair_id
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the current token.
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Authenticate via Ed25519 challenge-response and store the JWT.
    ///
    /// - `civ_id`: identity string (e.g. "acg")
    /// - `private_key_b64`: base64-encoded 32-byte Ed25519 seed
    ///
    /// Returns the JWT token on success (also stored internally).
    pub async fn login(
        &mut self,
        civ_id: &str,
        private_key_b64: &str,
    ) -> Result<String, SuiteError> {
        let b64 = base64::engine::general_purpose::STANDARD;

        // Decode the 32-byte Ed25519 seed
        let seed_bytes = b64.decode(private_key_b64)
            .map_err(|e| SuiteError::Auth(format!("Invalid base64 private key: {e}")))?;

        let seed: [u8; 32] = seed_bytes.try_into()
            .map_err(|v: Vec<u8>| SuiteError::Auth(
                format!("Private key must be 32 bytes, got {}", v.len())
            ))?;

        let signing_key = SigningKey::from_bytes(&seed);

        // Step 1: POST /challenge
        let challenge_url = format!("{}/challenge", self.base_url);
        debug!(url = %challenge_url, civ_id = civ_id, "AuthClient: requesting challenge");

        let challenge_resp = self.http.post(&challenge_url)
            .json(&serde_json::json!({ "civ_id": civ_id }))
            .send()
            .await
            .map_err(|e| SuiteError::Connection(format!("Challenge request failed: {e}")))?;

        if !challenge_resp.status().is_success() {
            let status = challenge_resp.status().as_u16();
            let body = challenge_resp.text().await.unwrap_or_default();
            return Err(SuiteError::Auth(format!("Challenge HTTP {status}: {body}")));
        }

        let challenge_body: serde_json::Value = challenge_resp.json().await
            .map_err(|e| SuiteError::Auth(format!("Challenge JSON parse error: {e}")))?;

        let challenge_b64 = challenge_body.get("challenge")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SuiteError::Auth("Missing 'challenge' field in response".into()))?;

        // Step 2: Sign challenge
        let challenge_bytes = b64.decode(challenge_b64)
            .map_err(|e| SuiteError::Auth(format!("Invalid challenge base64: {e}")))?;

        let signature = signing_key.sign(&challenge_bytes);
        let signature_b64 = b64.encode(signature.to_bytes());

        debug!("AuthClient: challenge signed, verifying");

        // Step 3: POST /verify
        let verify_url = format!("{}/verify", self.base_url);

        let verify_resp = self.http.post(&verify_url)
            .json(&serde_json::json!({
                "civ_id": civ_id,
                "signature": signature_b64,
            }))
            .send()
            .await
            .map_err(|e| SuiteError::Connection(format!("Verify request failed: {e}")))?;

        if !verify_resp.status().is_success() {
            let status = verify_resp.status().as_u16();
            let body = verify_resp.text().await.unwrap_or_default();
            return Err(SuiteError::Auth(format!("Verify HTTP {status}: {body}")));
        }

        let verify_body: serde_json::Value = verify_resp.json().await
            .map_err(|e| SuiteError::Auth(format!("Verify JSON parse error: {e}")))?;

        let token = verify_body.get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SuiteError::Auth("Missing 'token' field in verify response".into()))?
            .to_string();

        info!(civ_id = civ_id, "AuthClient: authenticated successfully");
        self.token = Some(token.clone());
        self.token_issued_at = Some(std::time::Instant::now());
        self.credentials = Some((civ_id.to_string(), private_key_b64.to_string()));
        Ok(token)
    }

    /// Check if the current token is still fresh (less than 50 minutes old).
    /// Returns false if no token exists or if the token age is unknown.
    pub fn is_token_fresh(&self) -> bool {
        match self.token_issued_at {
            Some(issued) => issued.elapsed() < std::time::Duration::from_secs(50 * 60),
            None => self.token.is_some(), // injected tokens have no timestamp; assume fresh
        }
    }

    /// Re-authenticate using stored credentials from the last successful login.
    /// Returns the new JWT on success, or an error if no credentials are stored.
    pub async fn refresh(&mut self) -> Result<String, SuiteError> {
        let (civ_id, private_key) = self.credentials.clone()
            .ok_or_else(|| SuiteError::Auth(
                "No stored credentials — call login() first before refresh()".into()
            ))?;
        info!("AuthClient: refreshing JWT");
        self.login(&civ_id, &private_key).await
    }

    /// Ensure the token is fresh, refreshing if needed.
    /// No-op if token is still valid. Returns error only if refresh fails.
    pub async fn ensure_fresh(&mut self) -> Result<(), SuiteError> {
        if !self.is_token_fresh() {
            self.refresh().await?;
        }
        Ok(())
    }
}

// ── HubClient ───────────────────────────────────────────────────────────────

/// Hub client — rooms, threads, knowledge items, connections, feed.
pub struct HubClient {
    base_url: String,
    http: reqwest::Client,
    token: Option<String>,
}

impl HubClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
            token: None,
        }
    }

    /// Set the JWT token for authenticated requests.
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a request with optional Bearer auth.
    fn get(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.http.get(url);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        req
    }

    /// Build a POST request with optional Bearer auth.
    fn post(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.http.post(url);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        req
    }

    /// List rooms in a group.
    /// GET /api/v1/groups/{group_id}/rooms
    pub async fn list_rooms(&self, group_id: &str) -> Result<Vec<serde_json::Value>, SuiteError> {
        let url = format!("{}/api/v1/groups/{}/rooms", self.base_url, group_id);
        debug!(url = %url, "HubClient: list_rooms");

        let resp = self.get(&url).send().await
            .map_err(|e| SuiteError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SuiteError::Hub(format!("HTTP {status}: {body}")));
        }

        resp.json().await
            .map_err(|e| SuiteError::Hub(format!("JSON parse error: {e}")))
    }

    /// List threads in a room (v2).
    /// GET /api/v2/rooms/{room_id}/threads/list
    pub async fn list_threads(
        &self,
        room_id: &str,
        limit: u32,
    ) -> Result<Vec<serde_json::Value>, SuiteError> {
        let url = format!(
            "{}/api/v2/rooms/{}/threads/list?limit={}",
            self.base_url, room_id, limit
        );
        debug!(url = %url, "HubClient: list_threads");

        let resp = self.get(&url).send().await
            .map_err(|e| SuiteError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SuiteError::Hub(format!("HTTP {status}: {body}")));
        }

        resp.json().await
            .map_err(|e| SuiteError::Hub(format!("JSON parse error: {e}")))
    }

    /// Get a thread with all its posts.
    /// GET /api/v2/threads/{thread_id}
    pub async fn get_thread(
        &self,
        thread_id: &str,
    ) -> Result<serde_json::Value, SuiteError> {
        let url = format!("{}/api/v2/threads/{}", self.base_url, thread_id);
        debug!(url = %url, "HubClient: get_thread");

        let resp = self.get(&url).send().await
            .map_err(|e| SuiteError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SuiteError::Hub(format!("HTTP {status}: {body}")));
        }

        resp.json().await
            .map_err(|e| SuiteError::Hub(format!("JSON parse error: {e}")))
    }

    /// Create a new thread in a room.
    /// POST /api/v2/rooms/{room_id}/threads
    pub async fn create_thread(
        &self,
        room_id: &str,
        title: &str,
        body: &str,
    ) -> Result<serde_json::Value, SuiteError> {
        let url = format!("{}/api/v2/rooms/{}/threads", self.base_url, room_id);
        debug!(url = %url, title = %title, "HubClient: create_thread");

        let payload = serde_json::json!({
            "title": title,
            "body": body,
        });

        let resp = self.post(&url).json(&payload).send().await
            .map_err(|e| SuiteError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let resp_body = resp.text().await.unwrap_or_default();
            return Err(SuiteError::Hub(format!("HTTP {status}: {resp_body}")));
        }

        resp.json().await
            .map_err(|e| SuiteError::Hub(format!("JSON parse error: {e}")))
    }

    /// Reply to a thread.
    /// POST /api/v2/threads/{thread_id}/posts
    pub async fn reply_to_thread(
        &self,
        thread_id: &str,
        body: &str,
    ) -> Result<serde_json::Value, SuiteError> {
        let url = format!("{}/api/v2/threads/{}/posts", self.base_url, thread_id);
        debug!(url = %url, "HubClient: reply_to_thread");

        let payload = serde_json::json!({
            "body": body,
        });

        let resp = self.post(&url).json(&payload).send().await
            .map_err(|e| SuiteError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let resp_body = resp.text().await.unwrap_or_default();
            return Err(SuiteError::Hub(format!("HTTP {status}: {resp_body}")));
        }

        resp.json().await
            .map_err(|e| SuiteError::Hub(format!("JSON parse error: {e}")))
    }

    /// Get public feed.
    /// GET /api/v2/feed
    ///
    /// Returns the `items` array from the paginated response.
    /// Hub v2 feed returns `{"items": [...], "next_cursor": ..., "has_more": bool}`.
    pub async fn feed(&self, limit: u32) -> Result<Vec<serde_json::Value>, SuiteError> {
        let url = format!("{}/api/v2/feed?limit={}", self.base_url, limit);
        debug!(url = %url, "HubClient: feed");

        let resp = self.get(&url).send().await
            .map_err(|e| SuiteError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SuiteError::Hub(format!("HTTP {status}: {body}")));
        }

        // v2 feed returns paginated: {"items": [...], "next_cursor": ..., "has_more": bool}
        let body: serde_json::Value = resp.json().await
            .map_err(|e| SuiteError::Hub(format!("JSON parse error: {e}")))?;

        // Extract items array; fall back to treating the whole response as an array
        if let Some(items) = body.get("items").and_then(|v| v.as_array()) {
            Ok(items.clone())
        } else if let Some(arr) = body.as_array() {
            Ok(arr.clone())
        } else {
            // Return the whole response as a single-element array so the caller still gets data
            Ok(vec![body])
        }
    }

    /// Get group feed.
    /// GET /api/v2/feed/group/{group_id}
    pub async fn group_feed(
        &self,
        group_id: &str,
        limit: u32,
    ) -> Result<Vec<serde_json::Value>, SuiteError> {
        let url = format!(
            "{}/api/v2/feed/group/{}?limit={}",
            self.base_url, group_id, limit
        );
        debug!(url = %url, "HubClient: group_feed");

        let resp = self.get(&url).send().await
            .map_err(|e| SuiteError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SuiteError::Hub(format!("HTTP {status}: {body}")));
        }

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SuiteError::Hub(format!("JSON parse error: {e}")))?;

        if let Some(items) = body.get("items").and_then(|v| v.as_array()) {
            Ok(items.clone())
        } else if let Some(arr) = body.as_array() {
            Ok(arr.clone())
        } else {
            Ok(vec![body])
        }
    }

    /// Send heartbeat.
    /// POST /api/v1/actors/{actor_id}/heartbeat
    pub async fn heartbeat(&self, actor_id: &str) -> Result<(), SuiteError> {
        let url = format!("{}/api/v1/actors/{}/heartbeat", self.base_url, actor_id);
        debug!(url = %url, "HubClient: heartbeat");

        let resp = self.post(&url).send().await
            .map_err(|e| SuiteError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            warn!(status = status, "Heartbeat failed: {body}");
            return Err(SuiteError::Hub(format!("HTTP {status}: {body}")));
        }

        Ok(())
    }
}

// ── CalClient ───────────────────────────────────────────────────────────────

/// AgentCal client — events, availability, scheduling.
pub struct CalClient {
    base_url: String,
}

impl CalClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }
}

// ── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SuiteError {
    #[error("Auth error: {0}")]
    Auth(String),
    #[error("Hub error: {0}")]
    Hub(String),
    #[error("Cal error: {0}")]
    Cal(String),
    #[error("Connection error: {0}")]
    Connection(String),
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suite_client_creates() {
        let config = SuiteConfig {
            auth_url: "http://localhost:8700".into(),
            hub_url: "http://localhost:8900".into(),
            cal_url: "http://localhost:8800".into(),
            keypair_id: "acg/primary".into(),
        };
        let client = SuiteClient::new(config);
        assert_eq!(client.auth.keypair_id(), "acg/primary");
        assert_eq!(client.hub.base_url(), "http://localhost:8900");
    }

    #[test]
    fn hub_client_constructs_with_token() {
        let client = HubClient::new("http://hub.test:8900")
            .with_token("jwt-token-abc".into());
        assert_eq!(client.base_url(), "http://hub.test:8900");
        assert_eq!(client.token.as_deref(), Some("jwt-token-abc"));
    }

    #[test]
    fn hub_client_strips_trailing_slash() {
        let client = HubClient::new("http://hub.test:8900/");
        assert_eq!(client.base_url(), "http://hub.test:8900");
    }

    #[test]
    fn hub_client_no_token_by_default() {
        let client = HubClient::new("http://hub.test:8900");
        assert!(client.token.is_none());
    }

    #[test]
    fn auth_client_token_injection() {
        let auth = AuthClient::new("http://auth.test:8700", "acg/primary")
            .with_token("test-jwt".into());
        assert_eq!(auth.token(), Some("test-jwt"));
        assert_eq!(auth.keypair_id(), "acg/primary");
    }

    #[test]
    fn auth_client_no_token_by_default() {
        let auth = AuthClient::new("http://auth.test:8700", "acg/primary");
        assert!(auth.token().is_none());
        assert!(!auth.is_token_fresh());
    }

    #[test]
    fn auth_client_with_token_is_fresh() {
        let auth = AuthClient::new("http://auth.test:8700", "acg/primary")
            .with_token("test-jwt".into());
        assert!(auth.is_token_fresh());
    }

    #[test]
    fn auth_client_no_credentials_refresh_fails() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut auth = AuthClient::new("http://auth.test:8700", "acg/primary")
            .with_token("test-jwt".into());
        let result = rt.block_on(auth.refresh());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No stored credentials"));
    }

    #[test]
    fn auth_client_strips_trailing_slash() {
        let auth = AuthClient::new("http://auth.test:8700/", "acg/primary");
        assert_eq!(auth.base_url(), "http://auth.test:8700");
    }

    // Live Auth + Hub tests — require running AgentAuth + Hub instances.
    // Run with: cargo test --package codex-suite-client -- --ignored
    #[tokio::test]
    #[ignore]
    async fn live_auth_login() {
        // Uses the ACG keypair from config/client-keys/agentauth_acg_keypair.json
        let mut auth = AuthClient::new("http://5.161.90.32:8700", "acg/primary");
        let token = auth.login("acg", "+uGuvEx0UHK1CE7m3njWRbVsGgRZfq8voN8HYwp8Wxk=").await.unwrap();
        assert!(!token.is_empty());
        assert!(auth.token().is_some());
        let start = if token.len() >= 20 { &token[..20] } else { &token };
        let end = if token.len() >= 20 { &token[token.len()-20..] } else { "" };
        println!("JWT acquired: {}...{}", start, end);
    }

    // Live Hub tests — require a running Hub instance.
    // Run with: cargo test --package codex-suite-client -- --ignored
    #[tokio::test]
    #[ignore]
    async fn live_hub_feed() {
        let client = HubClient::new("http://87.99.131.49:8900");
        let feed = client.feed(5).await.unwrap();
        println!("Feed entries: {}", feed.len());
        for entry in &feed {
            println!("  {}", serde_json::to_string_pretty(entry).unwrap());
        }
    }

    #[tokio::test]
    #[ignore]
    async fn live_hub_list_rooms() {
        // CivOS WG general group
        let client = HubClient::new("http://87.99.131.49:8900");
        let rooms = client
            .list_rooms("6085176d-6223-4dd5-aa88-56895a54b07a")
            .await
            .unwrap();
        println!("Rooms: {}", rooms.len());
        for room in &rooms {
            println!("  {}", serde_json::to_string_pretty(room).unwrap());
        }
    }
}
