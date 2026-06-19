import { ref } from 'vue';
import en from '../locales/en.json';
import pl from '../locales/pl.json';
import { invoke } from '@tauri-apps/api/core';

const translations: Record<string, any> = { en, pl };

export const currentLanguage = ref('en');
export const currentTheme = ref('dark');
export const isLanguageLoaded = ref(false);

export async function initI18n() {
  try {
    const settings: any = await invoke('get_settings');
    if (settings) {
      if (settings.language && translations[settings.language]) {
        currentLanguage.value = settings.language;
      }
      if (settings.theme) {
        currentTheme.value = settings.theme;
        applyThemeClass(settings.theme);
      }
    }
  } catch (e) {
  } finally {
    isLanguageLoaded.value = true;
  }
}

export async function setLanguage(lang: string) {
  if (translations[lang]) {
    currentLanguage.value = lang;
    await saveSettings();
  }
}

export async function setTheme(theme: string) {
  currentTheme.value = theme;
  applyThemeClass(theme);
  await saveSettings();
}

async function saveSettings() {
  try {
    await invoke('save_settings', { 
      settings: { 
        language: currentLanguage.value,
        theme: currentTheme.value
      } 
    });
  } catch (e) {}
}

function applyThemeClass(theme: string) {
  document.documentElement.className = '';
  if (theme !== 'dark') {
    document.documentElement.classList.add(`${theme}-theme`);
  }
}

export function t(key: string, params?: Record<string, string>): string {
  const keys = key.split('.');
  let value = translations[currentLanguage.value];
  for (const k of keys) {
    if (value === undefined) break;
    value = value[k];
  }
  if (typeof value === 'string' && params) {
    for (const [k, v] of Object.entries(params)) {
      value = value.replace(`{${k}}`, v);
    }
  }
  return value || key;
}
