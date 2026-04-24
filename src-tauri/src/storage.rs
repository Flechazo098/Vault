// ── Storage Module ──────────────────────────────────────────
// Purpose: Handle reading and writing the encrypted vault file.
// Security: All data persisted is already encrypted by the crypto
// module. This module deals only with serialization and I/O —
// it never sees plaintext secrets.
//
// The vault file is stored at: {app_data_dir}/vault.json
// Using Tauri's app data directory ensures proper sandboxing.

use std::path::PathBuf;

use crate::errors::VaultError;
use crate::models::VaultFile;

/// Get the path to the vault file in the app data directory.
pub fn vault_file_path(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("vault.json")
}

/// Check if a vault file exists.
pub fn vault_exists(app_data_dir: &PathBuf) -> bool {
    vault_file_path(app_data_dir).exists()
}

/// Load the vault file from disk.
/// Returns `None` if the file doesn't exist (not yet initialized).
pub fn load_vault(app_data_dir: &PathBuf) -> Result<Option<VaultFile>, VaultError> {
    let path = vault_file_path(app_data_dir);

    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(&path)
        .map_err(|e| VaultError::Io(format!("Failed to read vault file: {e}")))?;

    let vault: VaultFile = serde_json::from_str(&contents)
        .map_err(|e| VaultError::Serialization(format!("Failed to parse vault file: {e}")))?;

    Ok(Some(vault))
}

/// Save the vault file to disk.
/// Overwrites the existing file atomically by writing to a temp file
/// then renaming (best-effort on all platforms).
/// Creates the app data directory if it doesn't exist.
pub fn save_vault(app_data_dir: &PathBuf, vault: &VaultFile) -> Result<(), VaultError> {
    let path = vault_file_path(app_data_dir);

    // Ensure the app data directory exists before writing.
    std::fs::create_dir_all(app_data_dir)
        .map_err(|e| VaultError::Io(format!("Failed to create app data dir: {e}")))?;

    // Serialize to JSON bytes first.
    let json = serde_json::to_string_pretty(vault)
        .map_err(|e| VaultError::Serialization(format!("Failed to serialize vault: {e}")))?;

    // Write to a temporary file, then rename for crash safety.
    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, &json)
        .map_err(|e| VaultError::Io(format!("Failed to write vault file: {e}")))?;

    std::fs::rename(&temp_path, &path)
        .map_err(|e| VaultError::Io(format!("Failed to rename vault file: {e}")))?;

    Ok(())
}

/// Delete the vault file from disk.
/// Used when resetting the vault.
#[allow(dead_code)]
pub fn delete_vault(app_data_dir: &PathBuf) -> Result<(), VaultError> {
    let path = vault_file_path(app_data_dir);
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| VaultError::Io(format!("Failed to delete vault file: {e}")))?;
    }
    Ok(())
}
