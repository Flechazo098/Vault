<script setup lang="ts">
defineProps<{
  title: string;
  message: string;
  confirmLabel?: string;
  danger?: boolean;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();
</script>

<template>
  <div class="overlay" @click.self="emit('cancel')">
    <div class="dialog">
      <h3 class="dialog-title">{{ title }}</h3>
      <p class="dialog-message">{{ message }}</p>
      <div class="dialog-actions">
        <button class="btn-secondary" @click="emit('cancel')">{{ $t('confirm.btn_cancel') }}</button>
        <button
          :class="['btn', danger ? 'btn-danger' : 'btn-primary']"
          @click="emit('confirm')"
        >
          {{ confirmLabel || $t('confirm.btn_confirm') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(44, 42, 40, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.dialog {
  width: 380px;
  padding: 1.5rem;
  background: #ffffff;
  border-radius: 12px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.1);
}

.dialog-title {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.dialog-message {
  font-size: 0.9rem;
  color: #5a5652;
  margin-bottom: 1.25rem;
  line-height: 1.5;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
