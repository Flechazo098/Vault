<script setup lang="ts">
defineProps<{
  categories: string[];
  selectedCategory: string | null;
  entryCounts: Record<string, number>;
}>();

const emit = defineEmits<{
  'select-category': [value: string | null];
}>();
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-section">
      <h3 class="sidebar-label">{{ $t('sidebar.label_categories') }}</h3>
      <nav class="sidebar-nav">
        <button
          :class="['sidebar-item', { active: selectedCategory === null }]"
          @click="emit('select-category', null)"
        >
          <span class="item-dot all-dot"></span>
          <span class="item-name">{{ $t('sidebar.all_entries') }}</span>
        </button>

        <button
          v-for="cat in categories"
          :key="cat"
          :class="['sidebar-item', { active: selectedCategory === cat }]"
          @click="emit('select-category', cat)"
        >
          <span class="item-dot"></span>
          <span class="item-name">{{ cat }}</span>
        </button>

        <div v-if="categories.length === 0" class="sidebar-empty">
          <p>{{ $t('sidebar.empty') }}</p>
        </div>
      </nav>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 200px;
  min-width: 200px;
  background-color: #f0ede8;
  border-right: 1px solid #e5e2dc;
  padding: 1rem 0.75rem;
  overflow-y: auto;
  flex-shrink: 0;
}

.sidebar-section {
  margin-bottom: 1.5rem;
}

.sidebar-label {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #8a8580;
  padding: 0 0.5rem;
  margin-bottom: 0.5rem;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.sidebar-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.45rem 0.5rem;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  color: #5a5652;
  background: transparent;
  transition: all 0.12s ease;
  width: 100%;
  text-align: left;
}

.sidebar-item:hover {
  background-color: #e5e0da;
  color: #2c2a28;
}

.sidebar-item.active {
  background-color: #d9d4cc;
  color: #1f1d1b;
  font-weight: 600;
}

.item-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: #b5b0a8;
  flex-shrink: 0;
}

.item-dot.all-dot {
  background-color: #7a6f66;
}

.sidebar-item.active .item-dot {
  background-color: #635a52;
}

.item-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-empty {
  padding: 0.5rem;
}

.sidebar-empty p {
  font-size: 0.8rem;
  color: #a5a09a;
}
</style>
