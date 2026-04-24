<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';
import { useLocale } from '../composables/useLocale';
import { useVault } from '../stores/vault';

const vault = useVault();
const { t } = useI18n();

defineProps<{
  searchQuery: string;
}>();

const emit = defineEmits<{
  'update:search-query': [value: string];
  lock: [];
  'create-new': [];
}>();

const { alternateLabel, toggleLocale } = useLocale();

// ── Import / Export modal state ─────────────────────────
const showModal = ref(false);
const modalMode = ref<'export' | 'import'>('export');

// Export fields
const exportPassword = ref('');
const exportConfirm = ref('');
const exportError = ref('');

// Import fields
const importPassword = ref('');
const importError = ref('');

const processing = ref(false);

function openExport() {
  modalMode.value = 'export';
  exportPassword.value = '';
  exportConfirm.value = '';
  exportError.value = '';
  showModal.value = true;
}

function openImport() {
  modalMode.value = 'import';
  importPassword.value = '';
  importError.value = '';
  showModal.value = true;
}

function closeModal() {
  showModal.value = false;
  processing.value = false;
}

function handleOverlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('modal-overlay')) {
    closeModal();
  }
}

async function handleExport() {
  exportError.value = '';

  // Validate
  if (exportPassword.value.length < 8) {
    exportError.value = t('export.error_weak');
    return;
  }
  if (exportPassword.value !== exportConfirm.value) {
    exportError.value = t('export.error_mismatch');
    return;
  }

  processing.value = true;
  try {
    // 1. Get encrypted JSON from Rust backend
    const jsonStr = await vault.exportVault(exportPassword.value);

    // 2. Ask user where to save
    const filePath = await save({
      filters: [{ name: 'Vault Export', extensions: ['vault-encrypted'] }],
      defaultPath: 'vault-export.vault-encrypted',
    });
    if (!filePath) {
      processing.value = false;
      return; // user cancelled
    }

    // 3. Write via Rust command
    await invoke('write_export_file', { path: filePath, content: jsonStr });

    closeModal();
    console.info('[export]', t('export.success_message'));
  } catch (err: any) {
    exportError.value = t('export.error_failed', { error: String(err) });
  } finally {
    processing.value = false;
  }
}

async function handleImport() {
  importError.value = '';

  if (importPassword.value.length < 1) {
    importError.value = t('import.error_failed', { error: 'Password is required' });
    return;
  }

  processing.value = true;
  try {
    // 1. Ask user to pick a file
    const selected = await open({
      filters: [{ name: 'Vault Export', extensions: ['vault-encrypted'] }],
      multiple: false,
    });
    if (!selected) {
      processing.value = false;
      return; // user cancelled
    }

    // 2. Import file directly via single Rust command (read + decrypt + save)
    await vault.importFromFile(selected, importPassword.value);
    console.log('[import] done, entry count:', vault.entries.value.length);

    closeModal();
    console.info('[import]', t('import.success_message', { count: vault.entryCount.value }));
  } catch (err: any) {
    const msg = String(err);
    if (msg.includes('wrong password') || msg.includes('decrypt') || msg.includes('Decryption')) {
      importError.value = t('import.error_wrong_password');
    } else {
      importError.value = t('import.error_failed', { error: msg });
    }
  } finally {
    processing.value = false;
  }
}
</script>

<template>
  <div class="topbar">
    <div class="topbar-left">
      <div class="brand-mark">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
      </div>
      <span class="brand-name">{{ $t('app.name') }}</span>
    </div>

    <div class="topbar-center">
      <div class="search-wrapper">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
        <input
          type="text"
          class="search-input"
          :value="searchQuery"
          @input="emit('update:search-query', ($event.target as HTMLInputElement).value)"
          :placeholder="$t('topbar.search_placeholder')"
          spellcheck="false"
        />
      </div>
    </div>

    <div class="topbar-right">
      <button class="btn-ghost btn-sm locale-btn" @click="toggleLocale" :title="$t('locale.label')">
        {{ alternateLabel }}
      </button>
      <div class="topbar-divider"></div>
      <button class="btn-ghost btn-sm" @click="openImport" :title="$t('import.btn_import_title')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7,10 12,15 17,10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        <span>{{ $t('import.btn_import') }}</span>
      </button>
      <button class="btn-ghost btn-sm" @click="openExport" :title="$t('export.btn_export_title')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="17,8 12,3 7,8"/>
          <line x1="12" y1="3" x2="12" y2="15"/>
        </svg>
        <span>{{ $t('export.btn_export') }}</span>
      </button>
      <div class="topbar-divider"></div>
      <button class="btn-ghost btn-sm" @click="emit('create-new')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 5v14M5 12h14"/>
        </svg>
        <span>{{ $t('topbar.btn_new') }}</span>
      </button>
      <button class="btn-ghost btn-sm" @click="emit('lock')" :title="$t('topbar.btn_lock_title')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
        <span>{{ $t('topbar.btn_lock') }}</span>
      </button>
    </div>
  </div>

  <!-- ── Import / Export Modal ──────────────────────────── -->
  <Teleport to="body">
    <div v-if="showModal" class="modal-overlay" @click="handleOverlayClick">
      <div class="modal-card">
        <!-- Header -->
        <div class="modal-header">
          <h3>{{ modalMode === 'export' ? $t('export.modal_title') : $t('import.modal_title') }}</h3>
          <button class="btn-ghost btn-icon" @click="closeModal" :title="$t('confirm.btn_cancel')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <!-- Body: Export mode -->
        <div v-if="modalMode === 'export'" class="modal-body">
          <p class="modal-desc">{{ $t('export.modal_desc') }}</p>

          <div class="form-group">
            <label>{{ $t('export.label_password') }}</label>
            <input
              type="password"
              class="form-input"
              :placeholder="$t('export.placeholder_password')"
              v-model="exportPassword"
              autocomplete="new-password"
              spellcheck="false"
            />
          </div>

          <div class="form-group">
            <label>{{ $t('export.label_confirm') }}</label>
            <input
              type="password"
              class="form-input"
              :placeholder="$t('export.placeholder_confirm')"
              v-model="exportConfirm"
              autocomplete="new-password"
              spellcheck="false"
            />
          </div>

          <p v-if="exportError" class="form-error">{{ exportError }}</p>
        </div>

        <!-- Body: Import mode -->
        <div v-if="modalMode === 'import'" class="modal-body">
          <p class="modal-desc">{{ $t('import.modal_desc') }}</p>

          <div class="form-group">
            <label>{{ $t('import.label_password') }}</label>
            <input
              type="password"
              class="form-input"
              :placeholder="$t('import.placeholder_password')"
              v-model="importPassword"
              autocomplete="off"
              spellcheck="false"
            />
          </div>

          <p v-if="importError" class="form-error">{{ importError }}</p>
        </div>

        <!-- Footer -->
        <div class="modal-footer">
          <button class="btn-ghost" @click="closeModal">{{ $t('confirm.btn_cancel') }}</button>
          <button
            class="btn-primary"
            :disabled="processing"
            @click="modalMode === 'export' ? handleExport() : handleImport()"
          >
            <span v-if="processing" class="spinner"></span>
            {{ modalMode === 'export' ? $t('export.btn_export_action') : $t('import.btn_import_action') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  height: 48px;
  padding: 0 1rem;
  background-color: #ffffff;
  border-bottom: 1px solid #e5e2dc;
  gap: 1rem;
  flex-shrink: 0;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 180px;
}

.brand-mark {
  display: flex;
  align-items: center;
  color: #7a6f66;
}

.brand-name {
  font-weight: 600;
  font-size: 0.95rem;
  color: #2c2a28;
  letter-spacing: 0.02em;
}

.topbar-center {
  flex: 1;
  display: flex;
  justify-content: center;
}

.search-wrapper {
  position: relative;
  width: 100%;
  max-width: 360px;
}

.search-icon {
  position: absolute;
  left: 0.65rem;
  top: 50%;
  transform: translateY(-50%);
  color: #a5a09a;
  pointer-events: none;
}

.search-input {
  padding-left: 2.25rem;
  height: 32px;
  font-size: 0.85rem;
  background-color: #f0ede8;
  border-color: transparent;
  border-radius: 6px;
}

.search-input:focus {
  background-color: #ffffff;
  border-color: #d5d0ca;
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  min-width: 180px;
  justify-content: flex-end;
}

.topbar-divider {
  width: 1px;
  height: 20px;
  background-color: #e5e2dc;
  margin: 0 0.15rem;
}

.locale-btn {
  font-size: 0.8rem;
  color: #8a8580;
  min-width: 40px;
  justify-content: center;
}

.locale-btn:hover {
  color: #5a5652;
}

.btn-sm {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.35rem 0.65rem;
  font-size: 0.85rem;
}

/* ── Modal ──────────────────────────────────────────── */

.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(0, 0, 0, 0.35);
}

.modal-card {
  background: #ffffff;
  border-radius: 12px;
  width: 420px;
  max-width: 90vw;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem 0.5rem;
}

.modal-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: #2c2a28;
}

.btn-icon {
  padding: 0.25rem;
  border: none;
  background: none;
  color: #8a8580;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-icon:hover {
  color: #2c2a28;
  background: #f0ede8;
}

.modal-body {
  padding: 0.75rem 1.25rem 1rem;
}

.modal-desc {
  margin: 0 0 1rem;
  font-size: 0.85rem;
  color: #6b6660;
  line-height: 1.4;
}

.form-group {
  margin-bottom: 0.75rem;
}

.form-group label {
  display: block;
  font-size: 0.8rem;
  font-weight: 500;
  color: #5a5652;
  margin-bottom: 0.25rem;
}

.form-input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  font-size: 0.85rem;
  border: 1px solid #d5d0ca;
  border-radius: 6px;
  background: #ffffff;
  color: #2c2a28;
  outline: none;
  transition: border-color 0.15s;
  box-sizing: border-box;
}

.form-input:focus {
  border-color: #7a6f66;
}

.form-error {
  margin: 0.25rem 0 0;
  font-size: 0.8rem;
  color: #d95c4a;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0.75rem 1.25rem 1rem;
}

.btn-ghost {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.45rem 0.85rem;
  font-size: 0.85rem;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: #5a5652;
  cursor: pointer;
  transition: all 0.12s;
}

.btn-ghost:hover {
  background: #f0ede8;
  border-color: #e5e2dc;
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.45rem 1rem;
  font-size: 0.85rem;
  border: none;
  border-radius: 6px;
  background: #2c2a28;
  color: #ffffff;
  cursor: pointer;
  font-weight: 500;
  transition: background 0.12s;
}

.btn-primary:hover {
  background: #1a1816;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #ffffff;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
