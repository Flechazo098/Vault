// ── Internationalization Setup ──────────────────────────────────
// Uses vue-i18n with lazy-loaded locale messages.
// Detects system language on startup, defaults to English.

import { createI18n } from 'vue-i18n';
import en from './locales/en.json';
import zhCN from './locales/zh-CN.json';

type MessageSchema = typeof en;

// Detect preferred language: navigator.language returns "zh-CN", "en-US", etc.
function getDefaultLocale(): string {
  const lang = navigator.language;
  if (lang.startsWith('zh')) return 'zh-CN';
  return 'en';
}

const i18n = createI18n<[MessageSchema], 'en' | 'zh-CN'>({
  legacy: false, // Use Composition API mode
  locale: getDefaultLocale(),
  fallbackLocale: 'en',
  messages: {
    en,
    'zh-CN': zhCN,
  },
});

export default i18n;
