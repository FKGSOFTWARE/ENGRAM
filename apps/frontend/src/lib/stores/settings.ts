import { writable } from 'svelte/store';
import { browser } from '$app/environment';

interface Settings {
  dailyCardLimit: number;
  voiceEnabled: boolean;
  theme: 'light' | 'dark' | 'system';
  llmProvider: 'gemini' | 'openai' | 'anthropic';
  apiKey: string | null;
}

const defaultSettings: Settings = {
  dailyCardLimit: 20,
  voiceEnabled: true,
  theme: 'system',
  llmProvider: 'gemini',
  apiKey: null
};

function createSettingsStore() {
  // Load from localStorage if available
  const stored = browser ? localStorage.getItem('engram-settings') : null;
  const initial: Settings = stored ? { ...defaultSettings, ...JSON.parse(stored) } : defaultSettings;

  const { subscribe, set, update } = writable<Settings>(initial);

  // Persist to localStorage on changes
  if (browser) {
    subscribe((value) => {
      localStorage.setItem('engram-settings', JSON.stringify(value));
    });
  }

  return {
    subscribe,

    setDailyLimit(limit: number) {
      update((s) => ({ ...s, dailyCardLimit: Math.max(1, Math.min(100, limit)) }));
    },

    toggleVoice() {
      update((s) => ({ ...s, voiceEnabled: !s.voiceEnabled }));
    },

    setTheme(theme: Settings['theme']) {
      update((s) => ({ ...s, theme }));
    },

    setLlmProvider(provider: Settings['llmProvider']) {
      update((s) => ({ ...s, llmProvider: provider }));
    },

    setApiKey(key: string | null) {
      update((s) => ({ ...s, apiKey: key }));
    },

    reset() {
      set(defaultSettings);
    }
  };
}

export const settingsStore = createSettingsStore();
