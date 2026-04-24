// ── Module Declarations ─────────────────────────────────────
pub mod commands;
pub mod crypto;
pub mod errors;
pub mod models;
pub mod storage;
pub mod vault;

use std::sync::Mutex;

use commands::AppState;

/// Application entry point.
///
/// Registers all Tauri commands and manages the global vault state.
/// The `AppState` is initialized as `None` (locked) on app start.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            vault: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            // ── Vault lifecycle ────────────────────────────
            commands::get_vault_status,
            commands::create_vault,
            commands::unlock_vault,
            commands::lock_vault,
            commands::check_auto_lock,
            commands::get_auto_lock_remaining,
            // ── Entry operations ───────────────────────────
            commands::get_entries,
            commands::get_entry,
            commands::view_secret,
            commands::copy_secret,
            commands::create_entry,
            commands::update_entry,
            commands::delete_entry,
            commands::get_categories,
            // ── Import / Export ───────────────────────────
            commands::export_vault,
            commands::import_entries,
            commands::import_from_file,
            commands::write_export_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
