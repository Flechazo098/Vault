// ── Tauri Commands ──────────────────────────────────────────
// Purpose: Expose vault operations to the Vue frontend via Tauri's IPC.
//
// Security design:
// - Each command checks for vault lock state before proceeding.
// - The `copy_secret` command returns the secret AND manages clipboard
//   auto-clear via a spawned task.
// - The `view_secret` command returns the secret temporarily (frontend
//   must not cache it).
// - No command logs or persists plaintext secrets.
// - Activity is recorded on each command to support auto-lock.

use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};

use crate::errors::VaultError;
use crate::models::*;
use crate::storage;
use crate::vault;

/// Global application state holding the in-memory vault.
/// When `None`, the vault is locked or not yet initialized.
pub struct AppState {
    pub vault: Mutex<Option<vault::VaultState>>,
}

// ── Initialization & Status ─────────────────────────────────

/// Check the vault status: initialized, locked, entry count.
#[tauri::command]
pub fn get_vault_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<VaultStatus, VaultError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    let initialized = storage::vault_exists(&app_data_dir);
    let vault_guard = state.vault.lock().unwrap();

    Ok(VaultStatus {
        initialized,
        locked: vault_guard.is_none(),
        entry_count: vault_guard.as_ref().map(|v| v.entries().len()).unwrap_or(0),  
    })
}

/// Create a new vault with the given master password.
/// Enforces minimum password length on the backend (frontend can be bypassed).
/// Trade-off: 8 character minimum is a baseline; users can choose longer.
#[tauri::command]
pub fn create_vault(
    app: AppHandle,
    master_password: String,
) -> Result<(), VaultError> {
    if master_password.len() < 8 {
        return Err(VaultError::InvalidPassword);
    }

    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    vault::create_vault(&app_data_dir, &master_password)
}

/// Unlock the vault with the master password.
/// On success, the vault state is loaded into memory.
#[tauri::command]
pub fn unlock_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    master_password: String,
) -> Result<Vec<EntrySummary>, VaultError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    let vault_state = vault::unlock_vault(&app_data_dir, &master_password)?;
    let entries: Vec<EntrySummary> = vault_state.entries().iter().map(EntrySummary::from).collect();

    let mut vault_guard = state.vault.lock().unwrap();
    *vault_guard = Some(vault_state);

    Ok(entries)
}

/// Lock the vault: zeroize all in-memory secrets.
#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) -> Result<(), VaultError> {
    let mut vault_guard = state.vault.lock().unwrap();
    if let Some(vault_state) = vault_guard.take() {
        // vault_state is dropped here, which triggers zeroize via Drop.
        drop(vault_state);
    }
    Ok(())
}

/// Check auto-lock status: is the vault timed out?
#[tauri::command]
pub fn check_auto_lock(state: State<'_, AppState>) -> Result<bool, VaultError> {
    let mut vault_guard = state.vault.lock().unwrap();
    match vault_guard.as_mut() {
        Some(vault_state) => {
            if vault_state.is_timed_out() {
                // Auto-lock: zeroize and remove state.
                *vault_guard = None;
                Ok(true)
            } else {
                vault_state.record_activity();
                Ok(false)
            }
        }
        None => Ok(false), // Already locked.
    }
}

/// Get time remaining before auto-lock (seconds).
#[tauri::command]
pub fn get_auto_lock_remaining(state: State<'_, AppState>) -> Result<u64, VaultError> {
    let vault_guard = state.vault.lock().unwrap();
    match vault_guard.as_ref() {
        Some(vault_state) => Ok(vault_state.time_remaining()),
        None => Ok(0),
    }
}

// ── Entry Operations ────────────────────────────────────────

/// Get all entries (summaries without secrets).
/// Records activity for auto-lock tracking.
#[tauri::command]
pub fn get_entries(state: State<'_, AppState>) -> Result<Vec<EntrySummary>, VaultError> {
    let mut vault_guard = state.vault.lock().unwrap();
    let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
    vault_state.record_activity();
    let entries: Vec<EntrySummary> = vault_state.entries().iter().map(EntrySummary::from).collect();
    Ok(entries)
}

/// Get a single entry by ID (without secret).
/// Records activity for auto-lock tracking.
#[tauri::command]
pub fn get_entry(state: State<'_, AppState>, id: String) -> Result<EntrySummary, VaultError> {
    let mut vault_guard = state.vault.lock().unwrap();
    let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
    vault_state.record_activity();
    let entry = vault_state
        .entries()
        .iter()
        .find(|e| e.id == id)
        .ok_or(VaultError::EntryNotFound)?;
    Ok(EntrySummary::from(entry))
}

/// View a secret temporarily (returns the decrypted secret).
/// The frontend must NOT persist this value.
/// Security: The secret is returned as a plain string over IPC.
/// The frontend should clear its reference after displaying/copying.
/// Updates last_used_at and persists to disk.
#[tauri::command]
pub fn view_secret(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<String, VaultError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    let secret = {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
        vault_state.record_activity();
        let entry = vault_state
            .entries_mut()
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or(VaultError::EntryNotFound)?;

        // Update last_used_at.
        entry.last_used_at = Some(crate::models::now_iso8601());
        entry.secret.clone()
    };

    // Persist the last_used_at update to disk.
    {
        let vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_ref().ok_or(VaultError::VaultLocked)?;
        vault::save_state(&app_data_dir, vault_state)?;
    }

    Ok(secret)
}

/// Copy a secret to the clipboard and schedule auto-clear.
/// Auto-clear happens after 15 seconds.
/// Updates last_used_at and persists to disk.
#[tauri::command]
pub fn copy_secret(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), VaultError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    let secret = {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
        vault_state.record_activity();
        let entry = vault_state
            .entries_mut()
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or(VaultError::EntryNotFound)?;

        // Update last_used_at.
        entry.last_used_at = Some(crate::models::now_iso8601());
        entry.secret.clone()
    };

    // Persist the last_used_at update to disk.
    {
        let vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_ref().ok_or(VaultError::VaultLocked)?;
        vault::save_state(&app_data_dir, vault_state)?;
    }

    // Copy to clipboard.
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| VaultError::Clipboard(e.to_string()))?;
    clipboard
        .set_text(secret.clone())
        .map_err(|e| VaultError::Clipboard(e.to_string()))?;

    // ════════════════════════════════════════════════════════════════
    // Clipboard confusion technique (security)
    // ════════════════════════════════════════════════════════════════
    //
    // Timeline:
    //   T+0s     — Copy real secret → user can Ctrl+V immediately
    //   T+5s     — Overwrite clipboard with empty text
    //              → user has 5s to paste; Win+V captures "" not secret
    //   T+15s    — EmptyClipboard() as final safety net
    //
    // Why this works:
    //   - 5 seconds is enough for the user to alt-tab and Ctrl+V
    //   - Win+V clipboard history will show "empty content" because
    //     the overwrite happens well before they browse history
    //   - No WinRT / STA requirements — arboard + Win32 work from
    //     any thread
    //   - The original `clipboard` handle is already dropped at this
    //     point, so no conflicts opening a new handle in the thread
    std::thread::spawn(move || {
        // Phase 1 (5s): Give user time to paste the real secret.
        std::thread::sleep(std::time::Duration::from_secs(5));

        // Overwrite clipboard with empty text (arboard, always safe).
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text("");
        }

        // Phase 2 (10s more = 15s total): Final safety clear.
        std::thread::sleep(std::time::Duration::from_secs(10));

        // Win32 EmptyClipboard — works from any thread, no STA needed.
        #[cfg(target_os = "windows")]
        unsafe {
            if windows::Win32::System::DataExchange::OpenClipboard(None).is_ok() {
                let _ = windows::Win32::System::DataExchange::EmptyClipboard();
                let _ = windows::Win32::System::DataExchange::CloseClipboard();
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text("");
            }
        }
    });

    Ok(())
}


/// Create a new entry.
#[tauri::command]
pub fn create_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    input: EntryInput,
) -> Result<EntrySummary, VaultError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    let now = crate::models::now_iso8601();
    let entry = VaultEntry {
        id: uuid::Uuid::new_v4().to_string(),
        name: input.name,
        entry_type: input.entry_type,
        description: input.description,
        secret: input.secret,
        category: input.category,
        created_at: now.clone(),
        updated_at: now,
        last_used_at: None,
    };

    let summary = EntrySummary::from(&entry);

    {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
        vault_state.entries_mut().push(entry);
        vault_state.record_activity();
    }

    // Persist to disk.
    {
        let vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_ref().ok_or(VaultError::VaultLocked)?;
        vault::save_state(&app_data_dir, vault_state)?;
    }

    Ok(summary)
}

/// Update an existing entry.
#[tauri::command]
pub fn update_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: EntryInput,
) -> Result<EntrySummary, VaultError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    let now = crate::models::now_iso8601();

    let summary = {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
        let entry = vault_state
            .entries_mut()
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or(VaultError::EntryNotFound)?;

        entry.name = input.name;
        entry.entry_type = input.entry_type;
        entry.description = input.description;
        // Security: Only update secret if a new value was provided.
        // An empty string means "keep the current secret".
        if !input.secret.is_empty() {
            entry.secret = input.secret;
        }
        entry.category = input.category;
        entry.updated_at = now.clone();

        // Clone summary before recording activity to avoid borrow conflict.
        let summary = EntrySummary::from(&*entry);
        vault_state.record_activity();
        summary
    };

    // Persist to disk.
    {
        let vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_ref().ok_or(VaultError::VaultLocked)?;
        vault::save_state(&app_data_dir, vault_state)?;
    }

    Ok(summary)
}

/// Delete an entry by ID.
#[tauri::command]
pub fn delete_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), VaultError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
        let original_len = vault_state.entries().len();
        vault_state.entries_mut().retain(|e| e.id != id);
        if vault_state.entries().len() == original_len {
            return Err(VaultError::EntryNotFound);
        }
        vault_state.record_activity();
    }

    // Persist to disk.
    {
        let vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_ref().ok_or(VaultError::VaultLocked)?;
        vault::save_state(&app_data_dir, vault_state)?;
    }

    Ok(())
}

/// Get all unique categories from entries.
#[tauri::command]
pub fn get_categories(state: State<'_, AppState>) -> Result<Vec<String>, VaultError> {
    let mut vault_guard = state.vault.lock().unwrap();
    let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
    vault_state.record_activity();
    let mut categories: Vec<String> = vault_state
        .entries()
        .iter()
        .map(|e| e.category.clone())
        .collect();
    categories.sort();
    categories.dedup();
    Ok(categories)
}

// ── Import / Export ─────────────────────────────────────────

/// Export all entries as an encrypted JSON bundle.
///
/// Security:
/// - The export uses its OWN key derived from the export password (NOT the vault key)
/// - Each entry is individually encrypted with AES-256-GCM + fresh random nonce
/// - The export password is hashed with Argon2id before use
/// - The exported file reveals nothing about the vault's encryption
/// - Import regenerates all UUIDs, so there's no traceable link to the source vault
#[tauri::command]
pub fn export_vault(
    _app: AppHandle,
    state: State<'_, AppState>,
    export_password: String,
) -> Result<String, VaultError> {
    // 1. Check vault is unlocked.
    let entries: Vec<VaultEntry>;
    {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
        vault_state.record_activity();
        entries = vault_state.entries().to_vec();
    }

    // 2. Generate salt + derive export key.
    let salt = crate::crypto::generate_salt();
    let mut export_key = crate::crypto::derive_key(&export_password, &salt)?;

    // 3. Encrypt each entry with the export key.
    let mut exported_entries = Vec::with_capacity(entries.len());
    for entry in &entries {
        let payload = ImportedEntryPayload {
            name: entry.name.clone(),
            entry_type: entry.entry_type.clone(),
            description: entry.description.clone(),
            secret: entry.secret.clone(),
            category: entry.category.clone(),
        };
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| VaultError::Serialization(format!("Export serialize: {e}")))?;

        let nonce = crate::crypto::generate_nonce();
        let ciphertext = crate::crypto::encrypt_b64(&export_key, &nonce, &payload_json)?;

        exported_entries.push(ExportedEntry {
            nonce: crate::crypto::b64_encode(&nonce),
            ciphertext,
        });
    }

    // 4. Zeroize export key — no longer needed.
    crate::crypto::zeroize_buffer(&mut export_key);

    // 5. Build and serialize the export bundle.
    let bundle = ExportBundle {
        version: 1,
        salt: crate::crypto::b64_encode(&salt),
        entries: exported_entries,
    };

    serde_json::to_string_pretty(&bundle)
        .map_err(|e| VaultError::Serialization(format!("Export serialize: {e}")))
}

/// Import entries from an encrypted export bundle.
///
/// Security:
/// - Decrypts each entry using the key derived from the export password
/// - Each imported entry gets a brand-new UUID (no cross-vault ID linkage)
/// - Timestamps are set to the current time (preserving original timing is not useful)
/// - Failed decryption of any single entry fails the entire import (tamper resistance)
#[tauri::command]
pub fn import_entries(
    app: AppHandle,
    state: State<'_, AppState>,
    exported_json: String,
    export_password: String,
) -> Result<Vec<EntrySummary>, VaultError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    // 1. Parse the export bundle.
    let bundle: ExportBundle = serde_json::from_str(&exported_json)
        .map_err(|e| VaultError::Serialization(format!("Invalid export file: {e}")))?;

    if bundle.version != 1 {
        return Err(VaultError::Encryption(format!(
            "Unsupported export version: {}", bundle.version
        )));
    }

    if bundle.entries.is_empty() {
        return Err(VaultError::Encryption("Export file contains no entries".into()));
    }

    // 2. Derive the export key.
    let salt = crate::crypto::b64_decode(&bundle.salt)?;
    let mut export_key = crate::crypto::derive_key(&export_password, &salt)?;

    // 3. Decrypt each entry.
    let mut new_entries = Vec::with_capacity(bundle.entries.len());
    let now = crate::models::now_iso8601();

    for exp_entry in &bundle.entries {
        let nonce = crate::crypto::b64_decode(&exp_entry.nonce)?;
        let payload_json = crate::crypto::decrypt_b64(&export_key, &nonce, &exp_entry.ciphertext)
            .map_err(|_| VaultError::Decryption(
                "Failed to decrypt export — wrong password or tampered file".into()
            ))?;

        let payload: ImportedEntryPayload = serde_json::from_str(&payload_json)
            .map_err(|e| VaultError::Serialization(format!("Import payload parse: {e}")))?;

        new_entries.push(VaultEntry {
            id: uuid::Uuid::new_v4().to_string(),
            name: payload.name,
            entry_type: payload.entry_type,
            description: payload.description,
            secret: payload.secret,
            category: payload.category,
            created_at: now.clone(),
            updated_at: now.clone(),
            last_used_at: None,
        });
    }

    // 4. Zeroize export key.
    crate::crypto::zeroize_buffer(&mut export_key);

    // 5. Add entries to the vault state and persist.
    let summaries: Vec<EntrySummary>;
    {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
        vault_state.entries_mut().extend(new_entries);
        vault_state.record_activity();

        // Build summaries before releasing the lock.
        summaries = vault_state.entries().iter().map(EntrySummary::from).collect();

        // Persist AFTER extending entries, while lock is held for vault_key.
        vault::save_state(&app_data_dir, vault_state)?;
    }

    Ok(summaries)
}

// ── File I/O helpers (used by frontend import/export) ──

#[tauri::command]
pub fn write_export_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("Failed to write file: {e}"))
}

/// Single combined command: read export file from disk, decrypt, and import.
/// This avoids two separate IPC calls (read + import) and reduces failure points.
#[tauri::command]
pub fn import_from_file(
    app: AppHandle,
    state: State<'_, AppState>,
    file_path: String,
    export_password: String,
) -> Result<Vec<EntrySummary>, VaultError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        VaultError::Storage(format!("Failed to get app data dir: {e}"))
    })?;

    // 1. Read file.
    let exported_json = std::fs::read_to_string(&file_path).map_err(|e| {
        VaultError::Storage(format!("Failed to read import file '{file_path}': {e}"))
    })?;

    // 2. Parse the export bundle.
    let bundle: ExportBundle = serde_json::from_str(&exported_json)
        .map_err(|e| VaultError::Serialization(format!("Invalid export file: {e}")))?;

    if bundle.version != 1 {
        return Err(VaultError::Encryption(format!(
            "Unsupported export version: {}", bundle.version
        )));
    }

    if bundle.entries.is_empty() {
        return Err(VaultError::Encryption("Export file contains no entries".into()));
    }

    // 3. Derive the export key.
    let salt = crate::crypto::b64_decode(&bundle.salt)?;
    let mut export_key = crate::crypto::derive_key(&export_password, &salt)?;

    // 4. Decrypt each entry.
    let mut new_entries = Vec::with_capacity(bundle.entries.len());
    let now = crate::models::now_iso8601();

    for exp_entry in &bundle.entries {
        let nonce = crate::crypto::b64_decode(&exp_entry.nonce)?;
        let payload_json = crate::crypto::decrypt_b64(&export_key, &nonce, &exp_entry.ciphertext)
            .map_err(|_| VaultError::Decryption(
                "Failed to decrypt export — wrong password or tampered file".into()
            ))?;

        let payload: ImportedEntryPayload = serde_json::from_str(&payload_json)
            .map_err(|e| VaultError::Serialization(format!("Import payload parse: {e}")))?;

        new_entries.push(VaultEntry {
            id: uuid::Uuid::new_v4().to_string(),
            name: payload.name,
            entry_type: payload.entry_type,
            description: payload.description,
            secret: payload.secret,
            category: payload.category,
            created_at: now.clone(),
            updated_at: now.clone(),
            last_used_at: None,
        });
    }

    // 5. Zeroize export key.
    crate::crypto::zeroize_buffer(&mut export_key);

    // 6. Add entries to the vault state and persist.
    let summaries: Vec<EntrySummary>;
    {
        let mut vault_guard = state.vault.lock().unwrap();
        let vault_state = vault_guard.as_mut().ok_or(VaultError::VaultLocked)?;
        vault_state.entries_mut().extend(new_entries);
        vault_state.record_activity();

        summaries = vault_state.entries().iter().map(EntrySummary::from).collect();

        vault::save_state(&app_data_dir, vault_state)?;
    }

    Ok(summaries)
}
