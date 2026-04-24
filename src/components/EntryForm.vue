<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { ENTRY_TYPES, PRESET_CATEGORIES, type EntryInput, type EntrySummary } from '../types';

const props = defineProps<{
  initial?: EntrySummary;
}>();

const emit = defineEmits<{
  submit: [value: EntryInput];
  cancel: [];
}>();

const form = reactive<EntryInput>({
  name: '',
  entry_type: 'API Key',
  description: '',
  secret: '',
  category: 'General',
});

const showNewCategory = ref(false);
const customCategory = ref('');
const isEditing = ref(false);

onMounted(() => {
  if (props.initial) {
    isEditing.value = true;
    form.name = props.initial.name;
    form.entry_type = props.initial.entry_type;
    form.description = props.initial.description;
    form.category = props.initial.category;
    // Secret is NOT pre-filled for editing — user must re-enter if changing.
  }
});

function handleCategorySelect(cat: string) {
  if (cat === '__new__') {
    showNewCategory.value = true;
  } else {
    form.category = cat;
  }
}

function addCustomCategory() {
  if (customCategory.value.trim()) {
    form.category = customCategory.value.trim();
  }
  showNewCategory.value = false;
  customCategory.value = '';
}

function handleSubmit() {
  if (!form.name.trim()) return;

  const input: EntryInput = {
    name: form.name.trim(),
    entry_type: form.entry_type,
    description: form.description.trim(),
    secret: form.secret,
    category: form.category,
  };

  if (isEditing.value && !input.secret) {
    // For edits, if secret is empty, we need to keep the old secret.
    // The parent component should handle this by preserving the old value.
  }

  emit('submit', input);
}
</script>

<template>
  <div class="entry-form">
    <div class="form-header">
      <h2 class="form-title">{{ isEditing ? $t('entry_form.title_edit') : $t('entry_form.title_new') }}</h2>
      <button class="btn-ghost" @click="emit('cancel')">{{ $t('entry_form.btn_cancel') }}</button>
    </div>

    <form @submit.prevent="handleSubmit" class="form-body">
      <div class="field">
        <label for="entry-name">{{ $t('entry_form.label_name') }}</label>
        <input
          id="entry-name"
          v-model="form.name"
          type="text"
          :placeholder="$t('entry_form.placeholder_name')"
          autocomplete="off"
          spellcheck="false"
          autofocus
          required
        />
      </div>

      <div class="field-row">
        <div class="field">
          <label for="entry-type">{{ $t('entry_form.label_type') }}</label>
          <select id="entry-type" v-model="form.entry_type">
            <option v-for="t in ENTRY_TYPES" :key="t" :value="t">{{ t }}</option>
          </select>
        </div>

        <div class="field">
          <label for="entry-category">{{ $t('entry_form.label_category') }}</label>
          <select
            v-if="!showNewCategory"
            id="entry-category"
            :value="form.category"
            @change="handleCategorySelect(($event.target as HTMLSelectElement).value)"
          >
            <option v-for="cat in PRESET_CATEGORIES" :key="cat" :value="cat">{{ cat }}</option>
            <option disabled>──────────</option>
            <option value="__new__">{{ $t('entry_form.new_category') }}</option>
          </select>
          <div v-else class="inline-input">
            <input
              v-model="customCategory"
              type="text"
              :placeholder="$t('entry_form.placeholder_category')"
              @keyup.enter="addCustomCategory"
              @blur="addCustomCategory"
            />
          </div>
        </div>
      </div>

      <div class="field">
        <label for="entry-description">{{ $t('entry_form.label_description') }}</label>
        <textarea
          id="entry-description"
          v-model="form.description"
          :placeholder="$t('entry_form.placeholder_description')"
          rows="2"
        ></textarea>
      </div>

      <div class="field">
        <label for="entry-secret">
          {{ isEditing ? $t('entry_form.label_secret_edit') : $t('entry_form.label_secret') }}
        </label>
        <textarea
          id="entry-secret"
          v-model="form.secret"
          :placeholder="isEditing ? $t('entry_form.placeholder_secret_edit') : $t('entry_form.placeholder_secret')"
          rows="3"
          spellcheck="false"
          :required="!isEditing"
        ></textarea>
      </div>

      <div class="form-footer">
        <button type="button" class="btn-secondary" @click="emit('cancel')">{{ $t('entry_form.btn_cancel') }}</button>
        <button type="submit" class="btn-primary" :disabled="!form.name.trim()">
          {{ isEditing ? $t('entry_form.btn_save') : $t('entry_form.btn_create') }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.entry-form {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.form-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem 1rem;
  border-bottom: 1px solid #f0ede8;
}

.form-title {
  font-size: 1.1rem;
  font-weight: 600;
}

.form-body {
  flex: 1;
  padding: 1.25rem 1.5rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.field-row {
  display: flex;
  gap: 0.75rem;
}

.field-row .field {
  flex: 1;
}

.inline-input {
  display: flex;
  gap: 0.35rem;
}

.inline-input input {
  flex: 1;
}

.form-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding-top: 0.75rem;
  border-top: 1px solid #f0ede8;
  margin-top: auto;
}
</style>
