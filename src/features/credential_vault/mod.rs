//! Encrypted credential vault.
//!
//! Stores site credentials encrypted at rest (AES-256-GCM, key derived from
//! `THALORA_MASTER_PASSWORD` with Argon2id; see [`crypto`]). Secrets are only
//! ever decrypted for `browser_fill_credential`, which types them into a page
//! of the credential's own origin without returning them to the model.
//!
//! The vault lives on the real filesystem (not the page-facing VFS): it is
//! the user's own store and no page or JS API can reach it.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

mod crypto;

/// Credential tuple: (service, username, decrypted password, extra metadata).
pub type CredentialInfo = (String, String, String, HashMap<String, String>);

/// A stored credential; the password is encrypted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    pub service: String,
    pub username: String,
    pub encrypted_password: String,
    pub additional_data: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// On-disk vault contents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct VaultData {
    credentials: HashMap<String, CredentialEntry>,
}

/// Encrypted credential store, persisted to a JSON file (or memory only).
#[derive(Debug, Clone)]
pub struct CredentialVault {
    file: Option<PathBuf>,
    data: VaultData,
}

impl CredentialVault {
    /// Open (or create) a vault backed by `file`.
    pub fn open<P: AsRef<Path>>(file: P) -> Result<Self> {
        let file = file.as_ref().to_path_buf();
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = if file.exists() {
            serde_json::from_str(&fs::read_to_string(&file)?)?
        } else {
            VaultData::default()
        };
        Ok(Self {
            file: Some(file),
            data,
        })
    }

    /// Open the vault at `THALORA_VAULT_PATH`, or `~/.thalora/credentials.json`.
    pub fn open_default() -> Result<Self> {
        if let Some(path) = std::env::var_os("THALORA_VAULT_PATH") {
            return Self::open(path);
        }
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Self::open(home.join(".thalora").join("credentials.json"))
    }

    /// A vault that is never persisted.
    pub fn in_memory() -> Self {
        Self {
            file: None,
            data: VaultData::default(),
        }
    }

    /// Write the vault atomically (temp file + rename), readable only by
    /// the owner on Unix.
    fn save(&self) -> Result<()> {
        let Some(file) = &self.file else {
            return Ok(());
        };
        let tmp = file.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(&self.data)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?;
        }
        fs::rename(&tmp, file)?;
        Ok(())
    }

    /// Store a credential under `key`; the password is encrypted.
    pub fn store_credentials(
        &mut self,
        key: &str,
        service: &str,
        username: &str,
        password: &str,
        additional_data: HashMap<String, String>,
    ) -> Result<()> {
        let entry = CredentialEntry {
            service: service.to_string(),
            username: username.to_string(),
            encrypted_password: crypto::encrypt_password(password)?,
            additional_data,
            created_at: Utc::now(),
            last_used: None,
            tags: vec![],
        };
        self.data.credentials.insert(key.to_string(), entry);
        self.save()
    }

    /// Decrypt the credential stored under `key` and mark it used.
    pub fn get_credentials(&mut self, key: &str) -> Result<Option<CredentialInfo>> {
        let Some(entry) = self.data.credentials.get_mut(key) else {
            return Ok(None);
        };
        let password = crypto::decrypt_password(&entry.encrypted_password)?;
        entry.last_used = Some(Utc::now());
        let info = (
            entry.service.clone(),
            entry.username.clone(),
            password,
            entry.additional_data.clone(),
        );
        self.save()?;
        Ok(Some(info))
    }

    /// Metadata for the credential stored under `key` (never the secret).
    pub fn entry(&self, key: &str) -> Option<&CredentialEntry> {
        self.data.credentials.get(key)
    }

    /// All stored (key, service, username) triples, without secrets.
    pub fn list(&self) -> Vec<(&str, &str, &str)> {
        let mut entries: Vec<_> = self
            .data
            .credentials
            .iter()
            .map(|(k, c)| (k.as_str(), c.service.as_str(), c.username.as_str()))
            .collect();
        entries.sort();
        entries
    }

    /// Remove the credential stored under `key`. Returns whether it existed.
    pub fn remove(&mut self, key: &str) -> Result<bool> {
        let removed = self.data.credentials.remove(key).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn with_master_password() {
        // SAFETY: tests touching the env are serialized.
        unsafe {
            std::env::set_var(
                "THALORA_MASTER_PASSWORD",
                "test-master-password-that-is-long-enough-123",
            )
        };
    }

    #[test]
    #[serial]
    fn stores_and_decrypts() {
        with_master_password();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("credentials.json");
        let mut vault = CredentialVault::open(&path).unwrap();
        vault
            .store_credentials("gh", "github", "me", "s3cret", HashMap::new())
            .unwrap();

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert!(!on_disk.contains("s3cret"), "password stored in plaintext");

        let mut reopened = CredentialVault::open(&path).unwrap();
        let (service, user, password, _) = reopened.get_credentials("gh").unwrap().unwrap();
        assert_eq!((service.as_str(), user.as_str()), ("github", "me"));
        assert_eq!(password, "s3cret");
        assert!(reopened.entry("gh").unwrap().last_used.is_some());
        assert_eq!(reopened.list(), vec![("gh", "github", "me")]);
        assert!(reopened.remove("gh").unwrap());
        assert!(reopened.get_credentials("gh").unwrap().is_none());
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn vault_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        with_master_password();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("credentials.json");
        let mut vault = CredentialVault::open(&path).unwrap();
        vault
            .store_credentials("gh", "github", "me", "pw", HashMap::new())
            .unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
