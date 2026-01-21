<script lang="ts">
  import { settingsStore } from '$lib/stores/settings';
</script>

<svelte:head>
  <title>Settings - Engram</title>
</svelte:head>

<div class="settings-page">
  <h1>Settings</h1>

  <section class="settings-section">
    <h2>Review Settings</h2>

    <div class="setting-item">
      <label for="dailyLimit">
        <span class="setting-label">Daily Card Limit</span>
        <span class="setting-desc">Maximum cards to review per session</span>
      </label>
      <input
        type="number"
        id="dailyLimit"
        min="1"
        max="100"
        value={$settingsStore.dailyCardLimit}
        onchange={(e) => settingsStore.setDailyLimit(parseInt((e.target as HTMLInputElement).value))}
      />
    </div>

    <div class="setting-item">
      <label for="voiceEnabled">
        <span class="setting-label">Voice Mode</span>
        <span class="setting-desc">Enable voice-based review sessions</span>
      </label>
      <button
        class="toggle-btn"
        class:active={$settingsStore.voiceEnabled}
        onclick={() => settingsStore.toggleVoice()}
      >
        {$settingsStore.voiceEnabled ? 'Enabled' : 'Disabled'}
      </button>
    </div>

  </section>

  <section class="settings-section">
    <h2>Appearance</h2>

    <div class="setting-item">
      <label>
        <span class="setting-label">Theme</span>
        <span class="setting-desc">Choose your preferred color scheme</span>
      </label>
      <div class="theme-buttons">
        <button
          class="theme-btn"
          class:active={$settingsStore.theme === 'light'}
          onclick={() => settingsStore.setTheme('light')}
        >
          Light
        </button>
        <button
          class="theme-btn"
          class:active={$settingsStore.theme === 'dark'}
          onclick={() => settingsStore.setTheme('dark')}
        >
          Dark
        </button>
        <button
          class="theme-btn"
          class:active={$settingsStore.theme === 'system'}
          onclick={() => settingsStore.setTheme('system')}
        >
          System
        </button>
      </div>
    </div>
  </section>

  <section class="settings-section">
    <h2>AI Provider</h2>

    <div class="setting-item">
      <label>
        <span class="setting-label">LLM Provider</span>
        <span class="setting-desc">Choose the AI provider for answer evaluation</span>
      </label>
      <div class="provider-buttons">
        <button
          class="provider-btn"
          class:active={$settingsStore.llmProvider === 'gemini'}
          onclick={() => settingsStore.setLlmProvider('gemini')}
        >
          Gemini
        </button>
        <button
          class="provider-btn"
          class:active={$settingsStore.llmProvider === 'openai'}
          onclick={() => settingsStore.setLlmProvider('openai')}
        >
          OpenAI
        </button>
        <button
          class="provider-btn"
          class:active={$settingsStore.llmProvider === 'anthropic'}
          onclick={() => settingsStore.setLlmProvider('anthropic')}
        >
          Anthropic
        </button>
      </div>
    </div>

    <div class="setting-item">
      <label for="apiKey">
        <span class="setting-label">API Key (Optional)</span>
        <span class="setting-desc">Use your own API key instead of server default</span>
      </label>
      <input
        type="password"
        id="apiKey"
        placeholder="Enter your API key"
        value={$settingsStore.apiKey ?? ''}
        onchange={(e) => settingsStore.setApiKey((e.target as HTMLInputElement).value || null)}
      />
    </div>
  </section>

  <section class="settings-section">
    <h2>Data</h2>
    <div class="setting-item">
      <label>
        <span class="setting-label">Reset Settings</span>
        <span class="setting-desc">Restore all settings to defaults</span>
      </label>
      <button class="reset-btn" onclick={() => settingsStore.reset()}>
        Reset to Defaults
      </button>
    </div>
  </section>
</div>

<style>
  .settings-page {
    max-width: 600px;
    margin: 0 auto;
  }

  h1 {
    font-size: 1.75rem;
    margin-bottom: 2rem;
  }

  .settings-section {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .settings-section h2 {
    font-size: 1.125rem;
    margin-bottom: 1rem;
    color: var(--text-secondary);
  }

  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 0;
    border-bottom: 1px solid var(--border);
  }

  .setting-item:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .setting-label {
    display: block;
    font-weight: 500;
  }

  .setting-desc {
    display: block;
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-top: 0.25rem;
  }

  input[type='number'],
  input[type='password'] {
    width: 120px;
    padding: 0.5rem;
    font-size: 1rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--background);
    color: var(--text);
  }

  input[type='password'] {
    width: 200px;
  }

  .toggle-btn {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--background);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .toggle-btn.active {
    background: var(--primary);
    color: white;
    border-color: var(--primary);
  }

  .theme-buttons,
  .provider-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .theme-btn,
  .provider-btn {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--background);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .theme-btn.active,
  .provider-btn.active {
    background: var(--primary);
    color: white;
    border-color: var(--primary);
  }

  .reset-btn {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    border: 1px solid var(--danger);
    border-radius: 6px;
    background: transparent;
    color: var(--danger);
    cursor: pointer;
  }

  .reset-btn:hover {
    background: var(--danger);
    color: white;
  }
</style>
