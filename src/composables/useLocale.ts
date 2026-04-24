// ── Locale Switching Composable ────────────────────────────────
// Provides reactive locale state and a toggle function.
// Persists the user's choice to localStorage.

import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const STORAGE_KEY = 'vault-locale';

export function useLocale() {
  const { locale, t } = useI18n();

  // The current locale code.
  const currentLocale = computed(() => locale.value);

  // The display label for the other available locale.
  const alternateLabel = computed(() =>
    locale.value === 'en' ? '中文' : 'English',
  );

  // The alternate locale code.
  const alternateCode = computed(() =>
    locale.value === 'en' ? 'zh-CN' : 'en',
  );

  /// Switch to the other locale and persist the choice.
  function toggleLocale() {
    const next = alternateCode.value;
    locale.value = next;
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // localStorage may be unavailable (e.g. in some webviews).
    }
  }

  /// Initialize locale from stored preference or system language.
  function initLocale() {
    try {
      const stored = localStorage.getItem(STORAGE_KEY) as 'en' | 'zh-CN' | null;
      if (stored && (stored === 'en' || stored === 'zh-CN')) {
        locale.value = stored;
        return;
      }
    } catch {
      // Ignore localStorage errors.
    }

    // Fall back to browser language detection.
    const lang = navigator.language;
    if (lang.startsWith('zh')) {
      locale.value = 'zh-CN';
    }
  }

  return {
    currentLocale,
    alternateLabel,
    alternateCode,
    toggleLocale,
    initLocale,
    t,
  };
}
