<script lang="ts">
  import { onMount } from 'svelte';
  import { cardsStore, cardCount } from '$lib/stores/cards';
  import { getDueCards } from '$lib/db';

  let dueCount = $state(0);

  onMount(async () => {
    // Sync with server to get any cards added via API, then load locally
    await cardsStore.syncWithServer();
    const dueCards = await getDueCards(100);
    dueCount = dueCards.length;
  });
</script>

<svelte:head>
  <title>Engram - Voice Flashcards</title>
</svelte:head>

<div class="home">
  <h1>Welcome to Engram</h1>
  <p class="subtitle">Voice-powered spaced repetition learning</p>

  <div class="stats">
    <div class="stat-card">
      <span class="stat-value">{$cardCount}</span>
      <span class="stat-label">Total Cards</span>
    </div>
    <div class="stat-card">
      <span class="stat-value">{dueCount}</span>
      <span class="stat-label">Due for Review</span>
    </div>
  </div>

  <div class="actions">
    {#if dueCount > 0}
      <a href="/review" class="btn btn-primary">
        Start Review ({dueCount} cards)
      </a>
    {:else}
      <a href="/review" class="btn btn-secondary">
        No cards due
      </a>
    {/if}
    <a href="/cards" class="btn btn-secondary">
      Manage Cards
    </a>
  </div>

  <div class="quick-add">
    <h2>Quick Add Card</h2>
    <form onsubmit={(e) => {
      e.preventDefault();
      const form = e.target as HTMLFormElement;
      const front = (form.elements.namedItem('front') as HTMLInputElement).value;
      const back = (form.elements.namedItem('back') as HTMLInputElement).value;
      if (front && back) {
        cardsStore.create({ front, back });
        form.reset();
      }
    }}>
      <div class="form-group">
        <label for="front">Front (Question)</label>
        <textarea id="front" name="front" rows="2" required></textarea>
      </div>
      <div class="form-group">
        <label for="back">Back (Answer)</label>
        <textarea id="back" name="back" rows="2" required></textarea>
      </div>
      <button type="submit" class="btn btn-primary">Add Card</button>
    </form>
  </div>
</div>

<style>
  .home {
    max-width: 800px;
    margin: 0 auto;
    text-align: center;
  }

  h1 {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 1.25rem;
    margin-bottom: 2rem;
  }

  .stats {
    display: flex;
    gap: 1.5rem;
    justify-content: center;
    margin-bottom: 2rem;
  }

  .stat-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1.5rem 2rem;
    min-width: 150px;
  }

  .stat-value {
    display: block;
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--primary);
  }

  .stat-label {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
    margin-bottom: 3rem;
  }

  .btn {
    display: inline-block;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 500;
    text-decoration: none;
    border-radius: 8px;
    border: none;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-primary {
    background: var(--primary);
    color: white;
  }

  .btn-primary:hover {
    background: var(--primary-dark);
  }

  .btn-secondary {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover {
    background: var(--surface-alt);
  }

  .quick-add {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 2rem;
    text-align: left;
  }

  .quick-add h2 {
    font-size: 1.25rem;
    margin-bottom: 1rem;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    font-weight: 500;
    margin-bottom: 0.5rem;
  }

  .form-group textarea {
    width: 100%;
    padding: 0.75rem;
    font-size: 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--background);
    color: var(--text);
    resize: vertical;
    font-family: inherit;
  }

  .form-group textarea:focus {
    outline: none;
    border-color: var(--primary);
  }

  .quick-add .btn {
    width: 100%;
  }
</style>
