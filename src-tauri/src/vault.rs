// ── Vault State Machine ─────────────────────────────────────
// Purpose: Manage the in-memory state of the vault, including
// the decrypted vault key, loaded entries, and auto-lock timing.
//
// Security invariants:
// 1. The vault key is stored in memory only while the vault is unlocked.
// 2. All entries are decrypted into memory on unlock and re-encrypted on save.
// 3. The vault key is zeroed on lock.
// 4. Entry secrets are zeroed on lock.
// 5. Auto-lock triggers after a configurable period of inactivity.
//
// Thread safety: The vault state is wrapped in Mutex<Option<VaultState>>
// and managed by Tauri's state system.

use std::time::Instant;

use zeroize::Zeroize;

use crate::crypto;
use crate::errors::VaultError;
use crate::models::*;
use crate::storage;

/// In-memory state of an unlocked vault.
pub struct VaultState {
    /// The decrypted vault encryption key (256-bit).
    /// SECURITY: This is the crown jewel. Zeroed on lock/drop.
    vault_key: Vec<u8>,
    /// All entries (decrypted in memory).
    entries: Vec<VaultEntry>,
    /// Timestamp of the last activity for auto-lock.
    last_activity: Instant,
    /// Auto-lock timeout in seconds.
    auto_lock_seconds: u64,
}

impl VaultState {
    /// Create a new vault state with the decrypted vault key and entries.
    pub fn new(vault_key: Vec<u8>, entries: Vec<VaultEntry>, auto_lock_seconds: u64) -> Self {
        VaultState {
            vault_key,
            entries,
            last_activity: Instant::now(),
            auto_lock_seconds,
        }
    }

    /// Get the vault key (for encryption operations).
    pub fn vault_key(&self) -> &[u8] {
        &self.vault_key
    }

    /// Get a reference to all entries.
    pub fn entries(&self) -> &[VaultEntry] {
        &self.entries
    }

    /// Get a mutable reference to all entries.
    pub fn entries_mut(&mut self) -> &mut Vec<VaultEntry> {
        &mut self.entries
    }

    /// Record activity (reset auto-lock timer).
    pub fn record_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check if the vault has timed out due to inactivity.
    pub fn is_timed_out(&self) -> bool {
        self.last_activity.elapsed().as_secs() >= self.auto_lock_seconds
    }

    /// Get the time remaining before auto-lock (in seconds).
    pub fn time_remaining(&self) -> u64 {
        let elapsed = self.last_activity.elapsed().as_secs();
        if elapsed >= self.auto_lock_seconds {
            0
        } else {
            self.auto_lock_seconds - elapsed
        }
    }

    /// Zero out the vault key and all entry secrets.
    pub fn zeroize(&mut self) {
        self.vault_key.zeroize();
        for entry in &mut self.entries {
            entry.zeroize();
        }
        self.entries.clear();
    }
}

impl Drop for VaultState {
    fn drop(&mut self) {
        self.zeroize();
    }
}

// ── Public Vault Operations ─────────────────────────────────

/// Default auto-lock timeout: 5 minutes (300 seconds).
/// Trade-off: Shorter timeouts (60s) are more secure but annoying.
/// Longer (600s+) reduce security. 300s is a reasonable balance.
const DEFAULT_AUTO_LOCK_SECONDS: u64 = 300;

/// Create a new vault with the given master password.
///
/// This generates a new vault key, derives a wrapping key from the
/// master password, encrypts the vault key, and saves it to disk.
pub fn create_vault(
    app_data_dir: &std::path::PathBuf,
    master_password: &str,
) -> Result<(), VaultError> {
    // 1. Generate salt and vault key.
    let salt = crypto::generate_salt();
    let vault_key = crypto::generate_vault_key();

    // 2. Derive wrapping key from master password.
    let mut wrapping_key = crypto::derive_key(master_password, &salt)?;

    // 3. Encrypt vault key with wrapping key.
    let wrapping_nonce = crypto::generate_nonce();
    let encrypted_vault_key = crypto::encrypt_b64(
        &wrapping_key,
        &wrapping_nonce,
        &crypto::b64_encode(&vault_key),
    )?;

    // Zeroize wrapping key immediately — no longer needed.
    crypto::zeroize_buffer(&mut wrapping_key);

    // 4. Create and save empty vault file.
    let vault = VaultFile {
        version: 1,
        salt: crypto::b64_encode(&salt),
        wrapping_nonce: crypto::b64_encode(&wrapping_nonce),
        encrypted_vault_key,
        entries: Vec::new(),
    };

    storage::save_vault(app_data_dir, &vault)
}

/// Unlock the vault with the master password.
///
/// Decrypts the vault key, then decrypts all entries.
/// Returns the vault state with all data in memory.
pub fn unlock_vault(
    app_data_dir: &std::path::PathBuf,
    master_password: &str,
) -> Result<VaultState, VaultError> {
    // 1. Load encrypted vault file.
    let vault_file = storage::load_vault(app_data_dir)?
        .ok_or(VaultError::VaultNotInitialized)?;

    // 2. Decode salt and derive wrapping key.
    let salt = crypto::b64_decode(&vault_file.salt)?;
    let mut wrapping_key = crypto::derive_key(master_password, &salt)?;

    // 3. Decrypt the vault key.
    let wrapping_nonce = crypto::b64_decode(&vault_file.wrapping_nonce)?;
    let vault_key_b64 = crypto::decrypt_b64(&wrapping_key, &wrapping_nonce, &vault_file.encrypted_vault_key)?;
    let vault_key = crypto::b64_decode(&vault_key_b64)?;

    // Zeroize wrapping key immediately.
    crypto::zeroize_buffer(&mut wrapping_key);

    // 4. Decrypt all entries.
    let mut entries = Vec::with_capacity(vault_file.entries.len());
    for encrypted_entry in &vault_file.entries {
        let nonce = crypto::b64_decode(&encrypted_entry.nonce)?;
        let payload_json = crypto::decrypt_b64(&vault_key, &nonce, &encrypted_entry.ciphertext)?;
        let payload: EntryPayload = serde_json::from_str(&payload_json)
            .map_err(|e| VaultError::Serialization(format!("Failed to parse entry: {e}")))?;

        entries.push(VaultEntry::from_parts(encrypted_entry, payload));
    }

    Ok(VaultState::new(vault_key, entries, DEFAULT_AUTO_LOCK_SECONDS))
}

/// Save the current vault state to disk.
///
/// Re-encrypts ALL entries with the vault key.
/// Each entry gets a fresh nonce on every save.
pub fn save_state(app_data_dir: &std::path::PathBuf, state: &VaultState) -> Result<(), VaultError> {
    let vault_file = storage::load_vault(app_data_dir)?
        .ok_or(VaultError::VaultNotInitialized)?;

    let vault_key = state.vault_key();

    // Re-encrypt all entries with fresh nonces.
    let mut encrypted_entries = Vec::with_capacity(state.entries().len());
    for entry in state.entries() {
        let payload = EntryPayload {
            name: entry.name.clone(),
            entry_type: entry.entry_type.clone(),
            description: entry.description.clone(),
            secret: entry.secret.clone(),
            category: entry.category.clone(),
        };
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| VaultError::Serialization(format!("Serialize error: {e}")))?;

        let nonce = crypto::generate_nonce();
        let ciphertext = crypto::encrypt_b64(vault_key, &nonce, &payload_json)?;

        encrypted_entries.push(EncryptedEntry {
            id: entry.id.clone(),
            nonce: crypto::b64_encode(&nonce),
            ciphertext,
            created_at: entry.created_at.clone(),
            updated_at: entry.updated_at.clone(),
            last_used_at: entry.last_used_at.clone(),
        });
    }

    let updated_vault = VaultFile {
        version: vault_file.version,
        salt: vault_file.salt,
        wrapping_nonce: vault_file.wrapping_nonce,
        encrypted_vault_key: vault_file.encrypted_vault_key,
        entries: encrypted_entries,
    };

    storage::save_vault(app_data_dir, &updated_vault)
}


