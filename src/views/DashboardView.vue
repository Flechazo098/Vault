<script setup lang="ts">
import { ref, computed } from 'vue';
import type { EntrySummary, EntryInput } from '../types';
import AppSidebar from '../components/AppSidebar.vue';
import EntryList from '../components/EntryList.vue';
import EntryDetail from '../components/EntryDetail.vue';
import EntryForm from '../components/EntryForm.vue';
import TopBar from '../components/TopBar.vue';
import ConfirmDialog from '../components/ConfirmDialog.vue';

const props = defineProps<{
  vault: ReturnType<typeof import('../stores/vault').useVault>;
}>();

// ── State ──────────────────────────────────────────────
const selectedCategory = ref<string | null>(null);
const selectedEntryId = ref<string | null>(null);
const searchQuery = ref('');
const showCreateForm = ref(false);
const editingEntry = ref<EntrySummary | null>(null);
const confirmDelete = ref<EntrySummary | null>(null);

// ── Computed ───────────────────────────────────────────
const filteredEntries = computed(() => {
  let result = props.vault.entries.value;

  if (selectedCategory.value) {
    result = result.filter((e) => e.category === selectedCategory.value);
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase().trim();
    result = result.filter(
      (e) =>
        e.name.toLowerCase().includes(q) ||
        e.description.toLowerCase().includes(q) ||
        e.entry_type.toLowerCase().includes(q),
    );
  }

  return result;
});

const selectedEntry = computed(() => {
  if (!selectedEntryId.value) return null;
  return (
    props.vault.entries.value.find((e) => e.id === selectedEntryId.value) ??
    null
  );
});

// ── Methods ────────────────────────────────────────────
function selectEntry(entry: EntrySummary) {
  selectedEntryId.value = entry.id;
}

function selectCategory(category: string | null) {
  selectedCategory.value = category;
}

function onCreateEntry(input: EntryInput) {
  props.vault.createEntry(input);
  showCreateForm.value = false;
}

function onStartEdit(entry: EntrySummary) {
  editingEntry.value = entry;
}

function onUpdateEntry(input: EntryInput) {
  if (editingEntry.value) {
    props.vault.updateEntry(editingEntry.value.id, input);
    editingEntry.value = null;
  }
}

function onDeleteEntry(entry: EntrySummary) {
  confirmDelete.value = entry;
}

async function confirmDeleteEntry() {
  if (confirmDelete.value) {
    await props.vault.deleteEntry(confirmDelete.value.id);
    if (selectedEntryId.value === confirmDelete.value.id) {
      selectedEntryId.value = null;
    }
    confirmDelete.value = null;
  }
}

function onLock() {
  props.vault.lockVault();
}
</script>

<template>
  <div class="dashboard">
    <!-- Top Bar -->
    <TopBar
      :search-query="searchQuery"
      @update:search-query="searchQuery = $event"
      @lock="onLock"
      @create-new="showCreateForm = true"
    />

    <!-- Main Content -->
    <div class="dashboard-body">
      <!-- Sidebar -->
      <AppSidebar
        :categories="vault.categories.value"
        :selected-category="selectedCategory"
        :entry-counts="{}"
        @select-category="selectCategory"
      />

      <!-- Entry List -->
      <EntryList
        :entries="filteredEntries"
        :selected-id="selectedEntryId"
        @select-entry="selectEntry"
      />

      <!-- Detail Panel -->
      <div class="detail-panel">
        <Transition name="slide" mode="out-in">
          <EntryDetail
            v-if="selectedEntry && !editingEntry && !showCreateForm"
            :key="'detail-' + selectedEntry.id"
            :entry="selectedEntry"
            :vault="vault"
            @edit="onStartEdit"
            @delete="onDeleteEntry"
          />
          <EntryForm
            v-else-if="editingEntry"
            :key="'edit-' + editingEntry.id"
            :initial="editingEntry"
            @submit="onUpdateEntry"
            @cancel="editingEntry = null"
          />
          <EntryForm
            v-else-if="showCreateForm"
            key="create"
            @submit="onCreateEntry"
            @cancel="showCreateForm = false"
          />
          <div v-else class="detail-empty">
            <div class="empty-icon">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 5v14M5 12h14"/>
              </svg>
            </div>
            <p>{{ $t('dashboard.empty_title') }}</p>
          </div>
        </Transition>
      </div>
    </div>

    <!-- Confirm Delete Dialog -->
    <ConfirmDialog
      v-if="confirmDelete"
      :title="$t('confirm.delete_title')"
      :message="$t('confirm.delete_message', { name: confirmDelete.name })"
      :confirm-label="$t('confirm.btn_delete')"
      :danger="true"
      @confirm="confirmDeleteEntry"
      @cancel="confirmDelete = null"
    />
  </div>
</template>

<style scoped>
.dashboard {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #f6f4f0;
}

.dashboard-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.detail-panel {
  flex: 1;
  min-width: 360px;
  background-color: #ffffff;
  border-left: 1px solid #e5e2dc;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.detail-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #b5b0a8;
  gap: 0.75rem;
}

.detail-empty p {
  font-size: 0.9rem;
  color: #b5b0a8;
}

.empty-icon {
  opacity: 0.5;
}
</style>
