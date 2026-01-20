<script lang="ts">
  import type { Card, ReviewRating } from '@engram/shared';

  interface Props {
    card: Card;
    showingAnswer: boolean;
    onShowAnswer: () => void;
    onRate: (rating: ReviewRating) => void;
  }

  let { card, showingAnswer, onShowAnswer, onRate }: Props = $props();

  const ratings: { rating: ReviewRating; label: string; color: string }[] = [
    { rating: 'again', label: 'Again', color: '#dc3545' },
    { rating: 'hard', label: 'Hard', color: '#fd7e14' },
    { rating: 'good', label: 'Good', color: '#28a745' },
    { rating: 'easy', label: 'Easy', color: '#007bff' }
  ];
</script>

<div class="review-card">
  <div class="card-face front">
    <span class="label">Question</span>
    <p class="content">{card.front}</p>
  </div>

  {#if showingAnswer}
    <div class="card-face back">
      <span class="label">Answer</span>
      <p class="content">{card.back}</p>
    </div>

    <div class="rating-buttons">
      <p class="rating-prompt">How well did you know this?</p>
      <div class="buttons">
        {#each ratings as { rating, label, color }}
          <button
            class="rating-btn"
            style="--btn-color: {color}"
            onclick={() => onRate(rating)}
          >
            {label}
          </button>
        {/each}
      </div>
    </div>
  {:else}
    <button class="show-answer-btn" onclick={onShowAnswer}>
      Show Answer
    </button>
  {/if}
</div>

<style>
  .review-card {
    max-width: 600px;
    margin: 0 auto;
    padding: 2rem;
  }

  .card-face {
    background: var(--surface, #fff);
    border: 1px solid var(--border, #e0e0e0);
    border-radius: 12px;
    padding: 2rem;
    margin-bottom: 1.5rem;
    text-align: center;
  }

  .label {
    font-size: 0.75rem;
    color: var(--text-secondary, #666);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .content {
    font-size: 1.5rem;
    margin: 1rem 0 0;
    line-height: 1.4;
  }

  .back {
    background: var(--surface-alt, #f8f9fa);
  }

  .show-answer-btn {
    width: 100%;
    padding: 1rem 2rem;
    font-size: 1.25rem;
    background: var(--primary, #007bff);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .show-answer-btn:hover {
    background: var(--primary-dark, #0056b3);
  }

  .rating-buttons {
    text-align: center;
  }

  .rating-prompt {
    color: var(--text-secondary, #666);
    margin-bottom: 1rem;
  }

  .buttons {
    display: flex;
    gap: 0.75rem;
    justify-content: center;
    flex-wrap: wrap;
  }

  .rating-btn {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    background: var(--btn-color);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: opacity 0.2s, transform 0.1s;
    min-width: 80px;
  }

  .rating-btn:hover {
    opacity: 0.9;
    transform: translateY(-2px);
  }

  .rating-btn:active {
    transform: translateY(0);
  }
</style>
