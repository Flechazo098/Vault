// ── Vault Store (Singleton Composable) ──────────────────────
// Purpose: Centralized state management for the vault UI.
// This composable provides reactive state and methods that call
// the Tauri Rust backend via `invoke`.
//
// IMPORTANT: refs are defined at MODULE level, so every component
// that calls useVault() shares the SAME reactive state.
//
// Security: The store does NOT cache decrypted secrets longer than
// necessary. Secrets are fetched on-demand for copy/view operations.

import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { VaultStatus, EntrySummary, EntryInput } from '../types';

// ── Module-level reactive state (singleton) ─────────────
const vaultInitialized = ref(false);
const vaultLocked = ref(true);
const entries = ref<EntrySummary[]>([]);
const categories = ref<string[]>([]);

const isReady = computed(() => !vaultLocked.value && vaultInitialized.value);
const entryCount = computed(() => entries.value.length);

/// Reactive vault store — returns the singleton instance.
export function useVault() {

  // ── Lifecycle ──────────────────────────────────────────

  /// Fetch the current vault status from the backend.
  async function fetchStatus(): Promise<VaultStatus> {
    const status = await invoke<VaultStatus>('get_vault_status');
    vaultInitialized.value = status.initialized;
    vaultLocked.value = status.locked;
    return status;
  }

  /// Create a new vault with a master password.
  async function createVault(masterPassword: string): Promise<void> {
    await invoke('create_vault', { masterPassword });
    vaultInitialized.value = true;
  }

  /// Unlock the vault with the master password.
  async function unlockVault(masterPassword: string): Promise<void> {
    const result = await invoke<EntrySummary[]>('unlock_vault', {
      masterPassword,
    });
    entries.value = result;
    vaultLocked.value = false;
    await refreshCategories();
  }

  /// Lock the vault and zeroize in-memory secrets.
  async function lockVault(): Promise<void> {
    await invoke('lock_vault');
    vaultLocked.value = true;
    entries.value = [];
    categories.value = [];
  }

  /// Check if auto-lock should trigger.
  async function checkAutoLock(): Promise<boolean> {
    const timedOut = await invoke<boolean>('check_auto_lock');
    if (timedOut) {
      vaultLocked.value = true;
      entries.value = [];
      categories.value = [];
    }
    return timedOut;
  }

  /// Get time remaining before auto-lock (seconds).
  async function getAutoLockRemaining(): Promise<number> {
    return await invoke<number>('get_auto_lock_remaining');
  }

  // ── Entry Operations ───────────────────────────────────

  /// Refresh the entries list from the backend.
  async function refreshEntries(): Promise<void> {
    entries.value = await invoke<EntrySummary[]>('get_entries');
  }

  /// Refresh the categories list.
  async function refreshCategories(): Promise<void> {
    categories.value = await invoke<string[]>('get_categories');
  }

  /// Create a new entry.
  async function createEntry(input: EntryInput): Promise<EntrySummary> {
    const entry = await invoke<EntrySummary>('create_entry', { input });
    entries.value.push(entry);
    await refreshCategories();
    return entry;
  }

  /// Update an existing entry.
  async function updateEntry(
    id: string,
    input: EntryInput,
  ): Promise<EntrySummary> {
    const updated = await invoke<EntrySummary>('update_entry', { id, input });
    const idx = entries.value.findIndex((e) => e.id === id);
    if (idx !== -1) {
      entries.value[idx] = updated;
    }
    await refreshCategories();
    return updated;
  }

  /// Delete an entry by ID.
  async function deleteEntry(id: string): Promise<void> {
    await invoke('delete_entry', { id });
    entries.value = entries.value.filter((e) => e.id !== id);
    await refreshCategories();
  }

  /// View a secret (on-demand, transient).
  async function viewSecret(id: string): Promise<string> {
    const secret = await invoke<string>('view_secret', { id });
    // Update local last_used_at to reflect the backend change immediately.
    const entry = entries.value.find((e) => e.id === id);
    if (entry) {
      entry.last_used_at = new Date().toISOString();
    }
    return secret;
  }

  /// Copy a secret to clipboard (with auto-clear).
  async function copySecret(id: string): Promise<void> {
    await invoke('copy_secret', { id });
    // Update local last_used_at to reflect the backend change immediately.
    const entry = entries.value.find((e) => e.id === id);
    if (entry) {
      entry.last_used_at = new Date().toISOString();
    }
  }

  // ── Import / Export ───────────────────────────────────

  /// Export all entries as an encrypted JSON string.
  /// Returns the JSON string that should be saved to a .vault-export file.
  async function exportVault(exportPassword: string): Promise<string> {
    return await invoke<string>('export_vault', { exportPassword });
  }

  /// Import entries from an encrypted export file on disk.
  /// Single Rust command reads, decrypts, and persists — no intermediate JS step.
  async function importFromFile(
    filePath: string,
    exportPassword: string,
  ): Promise<void> {
    const updatedEntries = await invoke<EntrySummary[]>('import_from_file', {
      filePath,
      exportPassword,
    });
    entries.value = updatedEntries;
    await refreshCategories();
  }

  // ── Exports ────────────────────────────────────────────
  return {
    // State
    vaultInitialized,
    vaultLocked,
    entries,
    categories,
    isReady,
    entryCount,

    // Lifecycle
    fetchStatus,
    createVault,
    unlockVault,
    lockVault,
    checkAutoLock,
    getAutoLockRemaining,

    // Entries
    refreshEntries,
    refreshCategories,
    createEntry,
    updateEntry,
    deleteEntry,
    viewSecret,
    copySecret,

    // Import / Export
    exportVault,
    importFromFile,
  };
}
