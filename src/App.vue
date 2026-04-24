<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useVault } from './stores/vault';
import { useLocale } from './composables/useLocale';
import UnlockView from './views/UnlockView.vue';
import DashboardView from './views/DashboardView.vue';

const vault = useVault();
const { initLocale } = useLocale();

// ── Auto-lock polling ──────────────────────────────────────
// Poll the backend every 10 seconds to check for inactivity timeout.
// Trade-off: 10s interval is a balance between responsiveness and
// CPU usage. Shorter intervals (1s) would lock faster but waste cycles.
let autoLockInterval: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  initLocale();
  await vault.fetchStatus();

  // Start auto-lock polling when unlocked.
  autoLockInterval = setInterval(async () => {
    if (!vault.vaultLocked.value) {
      const timedOut = await vault.checkAutoLock();
      if (timedOut) {
        // Vault was auto-locked — UI will react via vaultLocked ref.
        // No console logging of security events in production.
      }
    }
  }, 10_000);
});

onUnmounted(() => {
  if (autoLockInterval) {
    clearInterval(autoLockInterval);
  }
});
</script>

<template>
  <div class="app-root">
    <Transition name="fade" mode="out-in">
      <UnlockView
        v-if="vault.vaultLocked.value"
        :vault="vault"
      />
      <DashboardView
        v-else-if="vault.isReady.value"
        :vault="vault"
      />
    </Transition>
  </div>
</template>

<style scoped>
.app-root {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>