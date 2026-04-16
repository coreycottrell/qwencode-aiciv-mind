//! # storage — File-based credential storage
//!
//! Reads and writes credentials as JSON files with restrictive permissions.
//! Each provider gets its own file: `{base_path}/{provider_name}.json`.
//!
//! On Unix, files are created with mode 0600 (owner read/write only).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::types::Credential;
use crate::AuthError;

/// File-based credential storage.
///
/// Stores one JSON file per provider under a configurable base directory.
/// The directory is created on first write if it does not exist.
pub struct CredentialStorage {
    base_path: PathBuf,
}

impl CredentialStorage {
    /// Create a new storage instance rooted at `base_path`.
    ///
    /// The directory does not need to exist yet — it will be created
    /// on the first `write` call.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Read a credential for the given provider name.
    ///
    /// Returns `Ok(None)` if the file does not exist.
    /// Returns `Err` if the file exists but cannot be read or parsed.
    pub fn read(&self, provider: &str) -> Result<Option<Credential>, AuthError> {
        let path = self.credential_path(provider);
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(&path).map_err(|e| AuthError::StorageError {
            path: path.display().to_string(),
            detail: e.to_string(),
        })?;
        let cred: Credential =
            serde_json::from_str(&contents).map_err(|e| AuthError::StorageError {
                path: path.display().to_string(),
                detail: format!("invalid JSON: {e}"),
            })?;
        Ok(Some(cred))
    }

    /// Write a credential for the given provider name.
    ///
    /// Creates the base directory if it does not exist.
    /// On Unix, sets file permissions to 0600.
    pub fn write(&self, credential: &Credential) -> Result<(), AuthError> {
        // Ensure directory exists
        if !self.base_path.exists() {
            std::fs::create_dir_all(&self.base_path).map_err(|e| AuthError::StorageError {
                path: self.base_path.display().to_string(),
                detail: format!("cannot create directory: {e}"),
            })?;
        }

        let path = self.credential_path(&credential.provider);
        let json = serde_json::to_string_pretty(credential).map_err(|e| {
            AuthError::StorageError {
                path: path.display().to_string(),
                detail: format!("serialization failed: {e}"),
            }
        })?;

        std::fs::write(&path, &json).map_err(|e| AuthError::StorageError {
            path: path.display().to_string(),
            detail: e.to_string(),
        })?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&path, perms).map_err(|e| AuthError::StorageError {
                path: path.display().to_string(),
                detail: format!("cannot set permissions: {e}"),
            })?;
        }

        tracing::debug!(provider = %credential.provider, path = %path.display(), "credential written");
        Ok(())
    }

    /// Delete a credential for the given provider name.
    ///
    /// Returns `Ok(())` if the file was deleted or did not exist.
    pub fn delete(&self, provider: &str) -> Result<(), AuthError> {
        let path = self.credential_path(provider);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| AuthError::StorageError {
                path: path.display().to_string(),
                detail: e.to_string(),
            })?;
            tracing::debug!(provider = %provider, "credential deleted");
        }
        Ok(())
    }

    /// List all stored provider names.
    ///
    /// Scans the base directory for `.json` files and returns the stem
    /// of each as a provider name.
    pub fn list_providers(&self) -> Result<Vec<String>, AuthError> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }
        let mut providers = Vec::new();
        let entries =
            std::fs::read_dir(&self.base_path).map_err(|e| AuthError::StorageError {
                path: self.base_path.display().to_string(),
                detail: e.to_string(),
            })?;
        for entry in entries {
            let entry = entry.map_err(|e| AuthError::StorageError {
                path: self.base_path.display().to_string(),
                detail: e.to_string(),
            })?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    providers.push(stem.to_string());
                }
            }
        }
        providers.sort();
        Ok(providers)
    }

    /// Read all stored credentials into a map.
    pub fn read_all(&self) -> Result<HashMap<String, Credential>, AuthError> {
        let providers = self.list_providers()?;
        let mut map = HashMap::new();
        for name in providers {
            if let Some(cred) = self.read(&name)? {
                map.insert(name, cred);
            }
        }
        Ok(map)
    }

    /// Get the file path for a provider's credential.
    fn credential_path(&self, provider: &str) -> PathBuf {
        self.base_path.join(format!("{provider}.json"))
    }

    /// Get the base path this storage is rooted at.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}
