// ── Data Models ─────────────────────────────────────────────
// Purpose: Define the on-disk vault format and in-memory entry
// representation. The on-disk format is versioned to support
// future upgrades (key rotation, algorithm changes).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

// ── Storage Format (versioned) ──────────────────────────────
// Version 1: Initial format.
//   - Vault key wrapped with key derived from master password
//   - Each entry encrypted individually with AES-256-GCM
//   - Per-entry unique nonce

/// The on-disk vault file structure.
/// Stored as a single JSON file in the app data directory.
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultFile {
    /// Storage format version. Enables forward-compatible upgrades.
    pub version: u32,
    /// Salt used for Argon2 key derivation (base64).
    /// Unique per vault, generated on creation.
    pub salt: String,
    /// Nonce used when encrypting the vault_key (base64).
    pub wrapping_nonce: String,
    /// The vault encryption key, encrypted with the master-password-derived key (base64).
    pub encrypted_vault_key: String,
    /// Encrypted entries.
    pub entries: Vec<EncryptedEntry>,
}

/// An entry as stored on disk — all sensitive fields are encrypted.
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedEntry {
    /// Unique identifier (UUID v4). Not encrypted — needed for lookup.
    pub id: String,
    /// Nonce used for AES-256-GCM encryption of this entry's payload (base64).
    pub nonce: String,
    /// AES-256-GCM ciphertext of the serialized EntryPayload (base64).
    pub ciphertext: String,
    /// Timestamps are not encrypted — they're metadata, not secrets.
    pub created_at: String,
    pub updated_at: String,
    pub last_used_at: Option<String>,
}

/// The actual sensitive data for an entry, encrypted as a JSON blob.
#[derive(Debug, Serialize, Deserialize)]
pub struct EntryPayload {
    pub name: String,
    pub entry_type: String,
    pub description: String,
    pub secret: String,
    pub category: String,
}

impl Zeroize for EntryPayload {
    fn zeroize(&mut self) {
        self.secret.zeroize();
    }
}

// ── In-Memory Representation ────────────────────────────────
// These are the decrypted forms used within the application.
// They are kept in memory only as long as necessary.

/// A fully decrypted vault entry.
/// Security note: the `secret` field holds plaintext in memory.
/// The vault ensures these are zeroed on drop or vault lock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub name: String,
    pub entry_type: String,
    pub description: String,
    /// The decrypted secret. Transient in memory.
    pub secret: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_used_at: Option<String>,
}

impl VaultEntry {
    /// Create an in-memory entry from an `EncryptedEntry` + decrypted payload.
    pub fn from_parts(encrypted: &EncryptedEntry, payload: EntryPayload) -> Self {
        VaultEntry {
            id: encrypted.id.clone(),
            name: payload.name,
            entry_type: payload.entry_type,
            description: payload.description,
            secret: payload.secret,
            category: payload.category,
            created_at: encrypted.created_at.clone(),
            updated_at: encrypted.updated_at.clone(),
            last_used_at: encrypted.last_used_at.clone(),
        }
    }
}

/// Zeroize implementation: on `zeroize()` call, overwrite the secret
/// in memory. This is invoked automatically when VaultState is dropped.
impl Zeroize for VaultEntry {
    fn zeroize(&mut self) {
        self.secret.zeroize();
    }
}

// ── Frontend-facing DTOs ────────────────────────────────────
// These are sent to the frontend and deliberately exclude the secret
// except when explicitly requested (on-demand copy/view).

/// Safe representation of an entry sent to the frontend (no secret).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySummary {
    pub id: String,
    pub name: String,
    pub entry_type: String,
    pub description: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_used_at: Option<String>,
}

impl From<&VaultEntry> for EntrySummary {
    fn from(e: &VaultEntry) -> Self {
        EntrySummary {
            id: e.id.clone(),
            name: e.name.clone(),
            entry_type: e.entry_type.clone(),
            description: e.description.clone(),
            category: e.category.clone(),
            created_at: e.created_at.clone(),
            updated_at: e.updated_at.clone(),
            last_used_at: e.last_used_at.clone(),
        }
    }
}

/// Request payload for creating or updating an entry (from frontend).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryInput {
    pub name: String,
    pub entry_type: String,
    pub description: String,
    pub secret: String,
    pub category: String,
}

// ── Vault Status ────────────────────────────────────────────
/// Status information sent to the frontend on connect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStatus {
    pub initialized: bool,
    pub locked: bool,
    pub entry_count: usize,
}

// ── Import / Export ─────────────────────────────────────────
/// A single encrypted entry in an export bundle.
/// Each entry is independently encrypted with its own nonce.
/// The key is derived from the export password via Argon2id.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportedEntry {
    /// Per-entry nonce (base64).
    pub nonce: String,
    /// AES-256-GCM ciphertext of the serialized ImportedEntryPayload (base64).
    pub ciphertext: String,
}

/// The exported vault bundle — a standalone encrypted JSON file.
///
/// Security properties:
/// - Uses its own Argon2id-derived key (NOT the vault key)
/// - Per-entry random nonces
/// - AES-256-GCM authenticated encryption
/// - The export password is never stored, only used for key derivation
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportBundle {
    /// Format version for forward compatibility.
    pub version: u32,
    /// Argon2 salt (base64) — unique per export.
    pub salt: String,
    /// Encrypted entries.
    pub entries: Vec<ExportedEntry>,
}

/// The plaintext payload inside each exported entry ciphertext.
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportedEntryPayload {
    pub name: String,
    pub entry_type: String,
    pub description: String,
    pub secret: String,
    pub category: String,
}

// ── Helpers ─────────────────────────────────────────────────
pub fn now_iso8601() -> String {
    Utc::now().to_rfc3339()
}
