<script lang="ts">
  import type { Snippet } from 'svelte';
  import { settingsStore } from '$lib/stores/settings';
  import { browser } from '$app/environment';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  // Apply theme
  $effect(() => {
    if (browser) {
      const theme = $settingsStore.theme;
      if (theme === 'system') {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        document.documentElement.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
      } else {
        document.documentElement.setAttribute('data-theme', theme);
      }
    }
  });
</script>

<div class="app">
  <nav class="nav">
    <a href="/" class="logo">Engram</a>
    <div class="nav-links">
      <a href="/review">Review</a>
      <a href="/cards">Cards</a>
      <a href="/ingest">Import</a>
      <a href="/settings">Settings</a>
    </div>
  </nav>

  <main class="main">
    {@render children()}
  </main>
</div>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(html) {
    --primary: #4f46e5;
    --primary-dark: #4338ca;
    --secondary: #6b7280;
    --success: #22c55e;
    --warning: #f59e0b;
    --danger: #ef4444;
    --surface: #ffffff;
    --surface-alt: #f9fafb;
    --background: #f3f4f6;
    --text: #111827;
    --text-secondary: #6b7280;
    --border: #e5e7eb;
  }

  :global(html[data-theme='dark']) {
    --primary: #818cf8;
    --primary-dark: #6366f1;
    --surface: #1f2937;
    --surface-alt: #374151;
    --background: #111827;
    --text: #f9fafb;
    --text-secondary: #9ca3af;
    --border: #374151;
  }

  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: var(--background);
    color: var(--text);
    line-height: 1.5;
  }

  .app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .nav {
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    padding: 1rem 2rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .logo {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--primary);
    text-decoration: none;
  }

  .nav-links {
    display: flex;
    gap: 1.5rem;
  }

  .nav-links a {
    color: var(--text-secondary);
    text-decoration: none;
    font-weight: 500;
    transition: color 0.2s;
  }

  .nav-links a:hover {
    color: var(--primary);
  }

  .main {
    flex: 1;
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
  }
</style>
