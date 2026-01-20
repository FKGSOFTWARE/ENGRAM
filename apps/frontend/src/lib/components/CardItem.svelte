<script lang="ts">
  import type { Card } from '@engram/shared';

  interface Props {
    card: Card;
    onEdit?: (card: Card) => void;
    onDelete?: (id: string) => void;
  }

  let { card, onEdit, onDelete }: Props = $props();

  let showBack = $state(false);

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffDays = Math.ceil((date.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));

    if (diffDays < 0) return 'Due now';
    if (diffDays === 0) return 'Due today';
    if (diffDays === 1) return 'Due tomorrow';
    return `Due in ${diffDays} days`;
  }
</script>

<div class="card-item">
  <button class="card-content" type="button" onclick={() => (showBack = !showBack)}>
    <div class="card-front">
      <span class="label">Front</span>
      <p>{card.front}</p>
    </div>
    {#if showBack}
      <div class="card-back">
        <span class="label">Back</span>
        <p>{card.back}</p>
      </div>
    {/if}
  </button>

  <div class="card-meta">
    <span class="due-date">{formatDate(card.next_review)}</span>
    <span class="stats">
      EF: {card.ease_factor.toFixed(2)} | Int: {card.interval}d | Rep: {card.repetitions}
    </span>
  </div>

  <div class="card-actions">
    {#if onEdit}
      <button class="btn-icon" onclick={() => onEdit?.(card)} aria-label="Edit card">
        ✏️
      </button>
    {/if}
    {#if onDelete}
      <button class="btn-icon btn-danger" onclick={() => onDelete?.(card.id)} aria-label="Delete card">
        🗑️
      </button>
    {/if}
  </div>
</div>

<style>
  .card-item {
    background: var(--surface, #fff);
    border: 1px solid var(--border, #e0e0e0);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 0.75rem;
  }

  .card-content {
    cursor: pointer;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    font: inherit;
    color: inherit;
  }

  .card-front,
  .card-back {
    margin-bottom: 0.5rem;
  }

  .label {
    font-size: 0.75rem;
    color: var(--text-secondary, #666);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .card-front p,
  .card-back p {
    margin: 0.25rem 0 0;
    font-size: 1rem;
  }

  .card-back {
    padding-top: 0.5rem;
    border-top: 1px dashed var(--border, #e0e0e0);
    margin-top: 0.5rem;
  }

  .card-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-secondary, #666);
    margin-top: 0.75rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border, #e0e0e0);
  }

  .card-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .btn-icon {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.25rem;
    font-size: 1rem;
    opacity: 0.7;
    transition: opacity 0.2s;
  }

  .btn-icon:hover {
    opacity: 1;
  }

  .btn-danger:hover {
    color: var(--danger, #dc3545);
  }
</style>
