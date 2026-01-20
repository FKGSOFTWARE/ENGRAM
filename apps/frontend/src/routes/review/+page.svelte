<script lang="ts">
  import { onMount } from 'svelte';
  import ReviewCard from '$lib/components/ReviewCard.svelte';
  import VoiceControls from '$lib/components/VoiceControls.svelte';
  import { reviewStore, currentCard, reviewProgress, hasCardsToReview } from '$lib/stores/review';
  import { settingsStore } from '$lib/stores/settings';
  import type { ReviewRating } from '@engram/shared';

  onMount(() => {
    reviewStore.loadQueue($settingsStore.dailyCardLimit);
  });

  function handleRate(rating: ReviewRating) {
    reviewStore.submitReview(rating);
  }
</script>

<svelte:head>
  <title>Review - Engram</title>
</svelte:head>

<div class="review-page">
  <header class="review-header">
    <h1>Review Session</h1>
    {#if $hasCardsToReview}
      <div class="progress">
        <span>{$reviewProgress.current} / {$reviewProgress.total}</span>
        <div class="progress-bar">
          <div
            class="progress-fill"
            style="width: {($reviewProgress.current / $reviewProgress.total) * 100}%"
          ></div>
        </div>
      </div>
    {/if}
  </header>

  {#if $reviewStore.loading}
    <div class="loading">Loading cards...</div>
  {:else if $reviewStore.error}
    <div class="error">{$reviewStore.error}</div>
  {:else if $reviewProgress.completed}
    <div class="completed">
      <h2>🎉 Session Complete!</h2>
      <p>You've reviewed all {$reviewProgress.total} cards.</p>
      <div class="completed-actions">
        <button class="btn btn-primary" onclick={() => reviewStore.loadQueue()}>
          Review More
        </button>
        <a href="/" class="btn btn-secondary">Back Home</a>
      </div>
    </div>
  {:else if $currentCard}
    {#if $settingsStore.voiceEnabled}
      <VoiceControls />
    {/if}

    <ReviewCard
      card={$currentCard}
      showingAnswer={$reviewStore.showingAnswer}
      onShowAnswer={() => reviewStore.showAnswer()}
      onRate={handleRate}
    />
  {:else}
    <div class="empty">
      <h2>No cards due for review</h2>
      <p>Come back later or add more cards.</p>
      <a href="/cards" class="btn btn-primary">Add Cards</a>
    </div>
  {/if}
</div>

<style>
  .review-page {
    max-width: 800px;
    margin: 0 auto;
  }

  .review-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .review-header h1 {
    font-size: 1.5rem;
    margin-bottom: 1rem;
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 1rem;
    justify-content: center;
  }

  .progress span {
    font-weight: 500;
    color: var(--text-secondary);
  }

  .progress-bar {
    width: 200px;
    height: 8px;
    background: var(--border);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--primary);
    transition: width 0.3s;
  }

  .loading,
  .error,
  .empty,
  .completed {
    text-align: center;
    padding: 3rem;
  }

  .error {
    color: var(--danger);
  }

  .empty h2,
  .completed h2 {
    font-size: 1.5rem;
    margin-bottom: 0.5rem;
  }

  .empty p,
  .completed p {
    color: var(--text-secondary);
    margin-bottom: 1.5rem;
  }

  .completed-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
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
</style>
