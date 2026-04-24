<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { zxcvbn } from '@zxcvbn-ts/core';

const props = defineProps<{
  vault: ReturnType<typeof import('../stores/vault').useVault>;
}>();

const { t } = useI18n();

const mode = ref<'unlock' | 'create'>('unlock');
const masterPassword = ref('');
const confirmPassword = ref('');
const errorMessage = ref('');
const loading = ref(false);

// ── Password strength (zxcvbn) ────────────────────────────
// Score 0-4: 0=too guessable, 1=very guessable, 2=somewhat,
//            3=safely unguessable, 4=very unguessable.
// We require score >= 2 for vault creation.
const passwordScore = computed(() => {
  if (!masterPassword.value) return -1;
  return zxcvbn(masterPassword.value).score;
});

const strengthClass = computed(() => {
  const s = passwordScore.value;
  if (s < 0) return '';
  if (s <= 1) return 'weak';
  if (s === 2) return 'fair';
  return 'strong';
});

const strengthLabels = ['strength.none', 'strength.weak', 'strength.fair', 'strength.strong', 'strength.perfect'] as const;

const strengthLabel = computed(() => {
  const s = passwordScore.value;
  if (s < 0) return '';
  const idx = Math.min(s, 4);
  return t(strengthLabels[idx]);
});

onMounted(async () => {
  const status = await props.vault.fetchStatus();
  if (!status.initialized) {
    mode.value = 'create';
  }
});

async function handleSubmit() {
  errorMessage.value = '';
  loading.value = true;

  try {
    if (mode.value === 'create') {
      if (masterPassword.value.length < 8) {
        throw new Error(t('unlock.error_min_length'));
      }
      if (passwordScore.value < 2) {
        throw new Error(t('unlock.error_weak_password'));
      }
      if (masterPassword.value !== confirmPassword.value) {
        throw new Error(t('unlock.error_mismatch'));
      }
      await props.vault.createVault(masterPassword.value);
      // Now unlock with the new password.
      await props.vault.unlockVault(masterPassword.value);
    } else {
      await props.vault.unlockVault(masterPassword.value);
    }
    // Clear password from memory after use.
    masterPassword.value = '';
    confirmPassword.value = '';
  } catch (err: unknown) {
    masterPassword.value = '';
    confirmPassword.value = '';
    errorMessage.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}

function switchMode() {
  mode.value = mode.value === 'unlock' ? 'create' : 'unlock';
  errorMessage.value = '';
  masterPassword.value = '';
  confirmPassword.value = '';
}
</script>

<template>
  <div class="unlock-view">
    <div class="unlock-card">
      <!-- Logo / Brand -->
      <div class="brand">
        <div class="brand-icon">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
        </div>
        <h1 class="brand-title">{{ $t('app.name') }}</h1>
        <p class="brand-subtitle" v-html="$t('app.tagline')"></p>
      </div>

      <!-- Mode indicator -->
      <div class="mode-tabs">
        <button
          :class="['mode-tab', { active: mode === 'unlock' }]"
          @click="switchMode"
        >
          {{ $t('unlock.tab_unlock') }}
        </button>
        <button
          :class="['mode-tab', { active: mode === 'create' }]"
          @click="switchMode"
        >
          {{ $t('unlock.tab_create') }}
        </button>
      </div>

      <!-- Form -->
      <form @submit.prevent="handleSubmit" class="unlock-form">
        <div class="field">
          <label for="master-password">{{ $t('unlock.label_password') }}</label>
          <input
            id="master-password"
            v-model="masterPassword"
            type="password"
            :placeholder="mode === 'create' ? $t('unlock.placeholder_create_password') : $t('unlock.placeholder_password')"
            autocomplete="off"
            spellcheck="false"
            autofocus
          />
        </div>

        <!-- Password strength meter (create mode only, require ≥ "Fair") -->
        <div v-if="mode === 'create' && masterPassword.length > 0" class="strength-meter">
          <div class="strength-bar">
            <div
              :class="['strength-fill', strengthClass]"
              :style="{ width: (passwordScore + 1) * 20 + '%' }"
            ></div>
          </div>
          <span :class="['strength-label', strengthClass]">{{ strengthLabel }}</span>
        </div>

        <div v-if="mode === 'create'" class="field">
          <label for="confirm-password">{{ $t('unlock.label_confirm') }}</label>
          <input
            id="confirm-password"
            v-model="confirmPassword"
            type="password"
            :placeholder="$t('unlock.placeholder_confirm')"
            autocomplete="off"
            spellcheck="false"
          />
        </div>

        <p v-if="errorMessage" class="error">{{ errorMessage }}</p>

        <button type="submit" class="btn-primary submit-btn" :disabled="loading">
          {{ loading ? $t('unlock.btn_processing') : mode === 'unlock' ? $t('unlock.btn_unlock') : $t('unlock.btn_create') }}
        </button>
      </form>

      <p class="hint">
        {{ mode === 'unlock' ? $t('unlock.hint_unlock') : $t('unlock.hint_create') }}
      </p>
    </div>
  </div>
</template>

<style scoped>
.unlock-view {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f6f4f0;
}

.unlock-card {
  width: 380px;
  padding: 3rem 2rem 2rem;
  background: #ffffff;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.04);
}

.brand {
  text-align: center;
  margin-bottom: 2rem;
}

.brand-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: 14px;
  background-color: #f0ede8;
  color: #7a6f66;
  margin-bottom: 1rem;
}

.brand-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: #1f1d1b;
  margin-bottom: 0.25rem;
}

.brand-subtitle {
  font-size: 0.85rem;
  color: #8a8580;
}

.mode-tabs {
  display: flex;
  background-color: #f0ede8;
  border-radius: 8px;
  padding: 3px;
  margin-bottom: 1.5rem;
}

.mode-tab {
  flex: 1;
  padding: 0.5rem;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  color: #5a5652;
  background: transparent;
  transition: all 0.15s ease;
}

.mode-tab.active {
  background-color: #ffffff;
  color: #2c2a28;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.unlock-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.field {
  display: flex;
  flex-direction: column;
}

.submit-btn {
  width: 100%;
  padding: 0.65rem;
  font-size: 0.95rem;
  margin-top: 0.5rem;
}

.submit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error {
  font-size: 0.85rem;
  color: #c44a4a;
  text-align: center;
  padding: 0.4rem 0.6rem;
  background: #fdf0f0;
  border-radius: 4px;
}

/* ── Password strength meter ─────────────────────── */
.strength-meter {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: -0.25rem;
}

.strength-bar {
  flex: 1;
  height: 4px;
  background: #e8e4e0;
  border-radius: 2px;
  overflow: hidden;
}

.strength-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.2s ease, background 0.2s ease;
}

.strength-fill.weak {
  background: #c44a4a;
  width: 40%;
}

.strength-fill.fair {
  background: #d49c3d;
  width: 60%;
}

.strength-fill.strong {
  background: #4a9c6f;
  width: 100%;
}

.strength-label {
  font-size: 0.75rem;
  white-space: nowrap;
  min-width: 4.5rem;
  text-align: right;
}

.strength-label.weak {
  color: #c44a4a;
}

.strength-label.fair {
  color: #d49c3d;
}

.strength-label.strong {
  color: #4a9c6f;
}

.hint {
  text-align: center;
  font-size: 0.75rem;
  color: #a5a09a;
  margin-top: 1.5rem;
  line-height: 1.5;
}
</style>
