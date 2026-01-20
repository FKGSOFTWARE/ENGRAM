<script lang="ts">
  import { onMount } from 'svelte';
  import CardItem from '$lib/components/CardItem.svelte';
  import { cardsStore } from '$lib/stores/cards';
  import type { Card } from '@engram/shared';

  let showAddForm = $state(false);
  let editingCard = $state<Card | null>(null);
  let frontInput = $state('');
  let backInput = $state('');

  onMount(() => {
    cardsStore.loadFromLocal();
  });

  function handleAdd() {
    if (frontInput && backInput) {
      cardsStore.create({ front: frontInput, back: backInput });
      frontInput = '';
      backInput = '';
      showAddForm = false;
    }
  }

  function handleEdit(card: Card) {
    editingCard = card;
    frontInput = card.front;
    backInput = card.back;
  }

  function handleSaveEdit() {
    if (editingCard && frontInput && backInput) {
      cardsStore.update(editingCard.id, { front: frontInput, back: backInput });
      editingCard = null;
      frontInput = '';
      backInput = '';
    }
  }

  function handleCancelEdit() {
    editingCard = null;
    frontInput = '';
    backInput = '';
  }

  function handleDelete(id: string) {
    if (confirm('Are you sure you want to delete this card?')) {
      cardsStore.delete(id);
    }
  }
</script>

<svelte:head>
  <title>Cards - Engram</title>
</svelte:head>

<div class="cards-page">
  <header class="page-header">
    <h1>Your Cards</h1>
    <button class="btn btn-primary" onclick={() => (showAddForm = !showAddForm)}>
      {showAddForm ? 'Cancel' : '+ Add Card'}
    </button>
  </header>

  {#if $cardsStore.error}
    <div class="error">{$cardsStore.error}</div>
  {/if}

  {#if showAddForm || editingCard}
    <div class="card-form">
      <h2>{editingCard ? 'Edit Card' : 'New Card'}</h2>
      <div class="form-group">
        <label for="front">Front (Question)</label>
        <textarea
          id="front"
          rows="3"
          bind:value={frontInput}
          placeholder="What is the capital of France?"
        ></textarea>
      </div>
      <div class="form-group">
        <label for="back">Back (Answer)</label>
        <textarea
          id="back"
          rows="3"
          bind:value={backInput}
          placeholder="Paris"
        ></textarea>
      </div>
      <div class="form-actions">
        {#if editingCard}
          <button class="btn btn-primary" onclick={handleSaveEdit}>Save Changes</button>
          <button class="btn btn-secondary" onclick={handleCancelEdit}>Cancel</button>
        {:else}
          <button class="btn btn-primary" onclick={handleAdd}>Add Card</button>
        {/if}
      </div>
    </div>
  {/if}

  {#if $cardsStore.loading}
    <div class="loading">Loading cards...</div>
  {:else if $cardsStore.cards.length === 0}
    <div class="empty">
      <p>No cards yet. Add your first card to get started!</p>
    </div>
  {:else}
    <div class="card-list">
      {#each $cardsStore.cards as card (card.id)}
        <CardItem
          {card}
          onEdit={handleEdit}
          onDelete={handleDelete}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .cards-page {
    max-width: 800px;
    margin: 0 auto;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }

  .page-header h1 {
    font-size: 1.75rem;
  }

  .error {
    background: #fee2e2;
    color: var(--danger);
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
  }

  .card-form {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 2rem;
  }

  .card-form h2 {
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

  .form-actions {
    display: flex;
    gap: 0.75rem;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 500;
    border-radius: 8px;
    border: none;
    cursor: pointer;
  }

  .btn-primary {
    background: var(--primary);
    color: white;
  }

  .btn-secondary {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
  }

  .loading,
  .empty {
    text-align: center;
    padding: 3rem;
    color: var(--text-secondary);
  }

  .card-list {
    display: flex;
    flex-direction: column;
  }
</style>
