<script setup lang="ts">
import type { EntrySummary } from '../types';

defineProps<{
  entries: EntrySummary[];
  selectedId: string | null;
}>();

const emit = defineEmits<{
  'select-entry': [value: EntrySummary];
}>();

function formatDate(iso: string): string {
  if (!iso) return '';
  const d = new Date(iso);
  return d.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
  });
}

function entryInitial(name: string): string {
  return name.charAt(0).toUpperCase();
}
</script>

<template>
  <div class="entry-list">
    <div class="list-header">
      <span class="list-count">{{ $t('entry_list.count_plural', { count: entries.length }) }}</span>
    </div>

    <div class="list-items">
      <button
        v-for="entry in entries"
        :key="entry.id"
        :class="['entry-item', { selected: selectedId === entry.id }]"
        @click="emit('select-entry', entry)"
      >
        <div class="entry-avatar">{{ entryInitial(entry.name) }}</div>
        <div class="entry-info">
          <span class="entry-name">{{ entry.name }}</span>
          <span class="entry-meta">
            {{ entry.entry_type }}
            <span v-if="entry.category"> · {{ entry.category }}</span>
          </span>
        </div>
        <span class="entry-date">{{ formatDate(entry.updated_at) }}</span>
      </button>

      <div v-if="entries.length === 0" class="empty-state">
        <p>{{ $t('entry_list.empty') }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.entry-list {
  width: 280px;
  min-width: 280px;
  background-color: #faf9f7;
  border-right: 1px solid #e5e2dc;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.list-header {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #e5e2dc;
  flex-shrink: 0;
}

.list-count {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: #8a8580;
}

.list-items {
  flex: 1;
  overflow-y: auto;
  padding: 0.35rem 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.entry-item {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  padding: 0.6rem 0.65rem;
  border-radius: 6px;
  background: transparent;
  transition: all 0.1s ease;
  text-align: left;
  width: 100%;
}

.entry-item:hover {
  background-color: #edeae5;
}

.entry-item.selected {
  background-color: #e5e0da;
}

.entry-avatar {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background-color: #e0dbd4;
  color: #5a5652;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.8rem;
  font-weight: 600;
  flex-shrink: 0;
}

.entry-item.selected .entry-avatar {
  background-color: #7a6f66;
  color: #ffffff;
}

.entry-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.entry-name {
  font-size: 0.85rem;
  font-weight: 600;
  color: #2c2a28;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entry-meta {
  font-size: 0.75rem;
  color: #8a8580;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entry-date {
  font-size: 0.7rem;
  color: #b5b0a8;
  flex-shrink: 0;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem 1rem;
}

.empty-state p {
  font-size: 0.85rem;
  color: #b5b0a8;
}
</style>
