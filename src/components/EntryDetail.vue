<script setup lang="ts">
import { ref, onUnmounted } from 'vue';
import type { EntrySummary } from '../types';

const props = defineProps<{
  entry: EntrySummary;
  vault: ReturnType<typeof import('../stores/vault').useVault>;
}>();

const emit = defineEmits<{
  edit: [value: EntrySummary];
  delete: [value: EntrySummary];
}>();

const showSecret = ref(false);
const secretValue = ref('');
const loadingSecret = ref(false);
const copied = ref(false);

async function toggleSecret() {
  if (showSecret.value) {
    showSecret.value = false;
    secretValue.value = '';
    return;
  }

  loadingSecret.value = true;
  try {
    secretValue.value = await props.vault.viewSecret(props.entry.id);
    showSecret.value = true;
  } catch (err) {
    console.error('Failed to view secret:', err);
  } finally {
    loadingSecret.value = false;
  }
}

async function copyToClipboard() {
  try {
    await props.vault.copySecret(props.entry.id);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch (err) {
    console.error('Failed to copy secret:', err);
  }
}

function formatDate(iso: string): string {
  if (!iso) return '—';
  return new Date(iso).toLocaleString();
}

function maskSecret(secret: string): string {
  if (secret.length <= 8) return '•'.repeat(secret.length);
  return secret.substring(0, 4) + '••••' + secret.substring(secret.length - 4);
}

// Cleanup: zeroize secret when component unmounts to prevent
// lingering sensitive data in JavaScript memory.
onUnmounted(() => {
  showSecret.value = false;
  secretValue.value = '';
});
</script>

<template>
  <div class="detail">
    <!-- Header -->
    <div class="detail-header">
      <div class="detail-title-row">
        <h2 class="detail-title">{{ entry.name }}</h2>
        <div class="detail-actions">
          <button class="btn-icon" :title="$t('entry_detail.tooltip_edit')" @click="emit('edit', entry)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/>
            </svg>
          </button>
          <button class="btn-icon" :title="$t('entry_detail.tooltip_delete')" @click="emit('delete', entry)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
            </svg>
          </button>
        </div>
      </div>
      <div class="detail-meta">
        <span class="badge">{{ entry.entry_type }}</span>
        <span v-if="entry.category" class="badge badge-category">{{ entry.category }}</span>
      </div>
    </div>

    <!-- Content -->
    <div class="detail-body">
      <!-- Description -->
      <div class="detail-section">
        <label>{{ $t('entry_detail.label_description') }}</label>
        <p class="description-text">{{ entry.description || $t('entry_detail.no_description') }}</p>
      </div>

      <!-- Secret -->
      <div class="detail-section">
        <label>{{ $t('entry_detail.label_secret') }}</label>
        <div class="secret-field">
          <div class="secret-display">
            <span v-if="showSecret" class="secret-value">{{ secretValue }}</span>
            <span v-else class="secret-masked">{{ maskSecret('sk-example-secret-key-here-12345') }}</span>
          </div>
          <div class="secret-actions">
            <button class="btn-ghost btn-sm" @click="toggleSecret" :disabled="loadingSecret">
              {{ loadingSecret ? '…' : showSecret ? $t('entry_detail.btn_hide') : $t('entry_detail.btn_show') }}
            </button>
            <button class="btn-ghost btn-sm" @click="copyToClipboard">
              {{ copied ? $t('entry_detail.btn_copied') : $t('entry_detail.btn_copy') }}
            </button>
          </div>
        </div>
        <p v-if="copied" class="copy-hint">{{ $t('entry_detail.copy_hint') }}</p>
      </div>

      <!-- Metadata -->
      <div class="detail-section detail-metadata">
        <div class="metadata-row">
          <span class="metadata-label">{{ $t('entry_detail.label_created') }}</span>
          <span class="metadata-value">{{ formatDate(entry.created_at) }}</span>
        </div>
        <div class="metadata-row">
          <span class="metadata-label">{{ $t('entry_detail.label_updated') }}</span>
          <span class="metadata-value">{{ formatDate(entry.updated_at) }}</span>
        </div>
        <div class="metadata-row">
          <span class="metadata-label">{{ $t('entry_detail.label_last_used') }}</span>
          <span class="metadata-value">{{ entry.last_used_at ? formatDate(entry.last_used_at) : $t('entry_detail.never') }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.detail {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.detail-header {
  padding: 1.25rem 1.5rem 1rem;
  border-bottom: 1px solid #f0ede8;
}

.detail-title-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.5rem;
}

.detail-title {
  font-size: 1.2rem;
  font-weight: 600;
  color: #1f1d1b;
  word-break: break-word;
}

.detail-actions {
  display: flex;
  gap: 0.15rem;
  flex-shrink: 0;
}

.detail-meta {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}

.badge {
  display: inline-block;
  padding: 0.2rem 0.5rem;
  font-size: 0.75rem;
  font-weight: 500;
  color: #5a5652;
  background-color: #f0ede8;
  border-radius: 4px;
}

.badge-category {
  background-color: #e8e5e0;
}

.detail-body {
  flex: 1;
  padding: 1.25rem 1.5rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.detail-section {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.description-text {
  font-size: 0.9rem;
  color: #5a5652;
  line-height: 1.6;
}

.secret-field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem;
  background-color: #faf9f7;
  border: 1px solid #e5e2dc;
  border-radius: 8px;
}

.secret-display {
  font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 0.85rem;
  word-break: break-all;
}

.secret-value {
  color: #2c2a28;
}

.secret-masked {
  color: #b5b0a8;
  letter-spacing: 0.05em;
}

.secret-actions {
  display: flex;
  gap: 0.35rem;
}

.copy-hint {
  font-size: 0.75rem;
  color: #7a6f66;
  font-style: italic;
}

.detail-metadata {
  padding-top: 0.75rem;
  border-top: 1px solid #f0ede8;
  gap: 0.5rem;
}

.metadata-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
}

.metadata-label {
  color: #8a8580;
}

.metadata-value {
  color: #5a5652;
  font-weight: 500;
}

.btn-sm {
  font-size: 0.8rem;
  padding: 0.25rem 0.55rem;
}
</style>
