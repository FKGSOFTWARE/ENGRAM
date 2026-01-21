import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type ReviewMode = 'manual' | 'oral' | 'conversational';

interface Settings {
  dailyCardLimit: number;
  voiceEnabled: boolean;
  theme: 'light' | 'dark' | 'system';
  llmProvider: 'gemini' | 'openai' | 'anthropic';
  apiKey: string | null;
  reviewMode: ReviewMode;
}

const defaultSettings: Settings = {
  dailyCardLimit: 20,
  voiceEnabled: true,
  theme: 'system',
  llmProvider: 'gemini',
  apiKey: null,
  reviewMode: 'manual'
};

function createSettingsStore() {
  // Load non-sensitive settings from localStorage
  const stored = browser ? localStorage.getItem('engram-settings') : null;
  // Load API key from sessionStorage (more secure - cleared on browser close)
  const storedApiKey = browser ? sessionStorage.getItem('engram-api-key') : null;

  const initial: Settings = stored
    ? { ...defaultSettings, ...JSON.parse(stored), apiKey: storedApiKey }
    : { ...defaultSettings, apiKey: storedApiKey };

  const { subscribe, set, update } = writable<Settings>(initial);

  // Persist to storage on changes
  if (browser) {
    subscribe((value) => {
      // Store non-sensitive settings in localStorage
      const { apiKey, ...nonSensitiveSettings } = value;
      localStorage.setItem('engram-settings', JSON.stringify(nonSensitiveSettings));
      // Store API key in sessionStorage (cleared when browser closes)
      if (apiKey) {
        sessionStorage.setItem('engram-api-key', apiKey);
      } else {
        sessionStorage.removeItem('engram-api-key');
      }
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

    setReviewMode(mode: ReviewMode) {
      update((s) => ({ ...s, reviewMode: mode }));
    },

    reset() {
      set(defaultSettings);
    }
  };
}

export const settingsStore = createSettingsStore();
