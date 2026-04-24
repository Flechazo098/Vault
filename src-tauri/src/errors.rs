// ── Error Types ──────────────────────────────────────────────
// Purpose: Centralized error handling for the vault application.
// Security: Error messages deliberately avoid exposing internal
// state (e.g., key length, salt format, plaintext contents).
//
// Trade-off: We map internal errors to user-facing strings rather
// than letting them propagate. This prevents information leakage
// through error messages while still enabling debugging via logs.

use serde::Serialize;

/// Unified error type for all vault operations.
/// Implements `Into<tauri::InvokeError>` so Tauri commands can return `Result<T, VaultError>`.
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("Invalid master password")]
    InvalidPassword,

    #[error("Vault is locked. Please unlock first.")]
    VaultLocked,

    #[error("Vault not initialized. Create a new vault first.")]
    VaultNotInitialized,

    #[error("Entry not found")]
    EntryNotFound,

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Inactivity timeout reached")]
    InactivityTimeout,
}

// ── Serialization for Tauri ─────────────────────────────────
// We serialize errors as JSON so the frontend can display them.
impl From<VaultError> for tauri::ipc::InvokeError {
    fn from(e: VaultError) -> Self {
        tauri::ipc::InvokeError::from(e.to_string())
    }
}

/// Serializable error payload sent to the frontend.
#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub message: String,
}

impl From<&VaultError> for ErrorPayload {
    fn from(e: &VaultError) -> Self {
        ErrorPayload {
            message: e.to_string(),
        }
    }
}
