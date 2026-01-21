<script lang="ts">
  import { onMount } from 'svelte';
  import ReviewCard from '$lib/components/ReviewCard.svelte';
  import VoiceControls from '$lib/components/VoiceControls.svelte';
  import VoiceReview from '$lib/components/VoiceReview.svelte';
  import ConversationalReview from '$lib/components/ConversationalReview.svelte';
  import { reviewStore, currentCard, reviewProgress, hasCardsToReview } from '$lib/stores/review';
  import { settingsStore } from '$lib/stores/settings';
  import type { ReviewRating } from '@engram/shared';

  type ReviewMode = 'manual' | 'oral' | 'conversational';

  // Current view state
  let selectedMode = $state<ReviewMode | null>(null);
  let sessionActive = $state(false);

  // Mode switch confirmation
  let showSwitchConfirm = $state(false);
  let pendingSwitchDirection = $state<'prev' | 'next' | null>(null);

  // Online status for voice mode indicator
  let isOnline = $state(true);

  $effect(() => {
    if (typeof window !== 'undefined') {
      isOnline = navigator.onLine;
      const handleOnline = () => isOnline = true;
      const handleOffline = () => isOnline = false;
      window.addEventListener('online', handleOnline);
      window.addEventListener('offline', handleOffline);
      return () => {
        window.removeEventListener('online', handleOnline);
        window.removeEventListener('offline', handleOffline);
      };
    }
  });

  // Mode definitions with icons and descriptions
  const modes = [
    {
      id: 'manual' as ReviewMode,
      name: 'Manual',
      icon: '📝',
      tagline: 'Classic flashcards',
      description: 'Click to reveal, rate yourself',
      color: '#6366f1'
    },
    {
      id: 'oral' as ReviewMode,
      name: 'Oral',
      icon: '🎤',
      tagline: 'Speak your answers',
      description: 'Voice input, AI evaluation',
      color: '#10b981'
    },
    {
      id: 'conversational' as ReviewMode,
      name: 'Conversational',
      icon: '💬',
      tagline: 'Learn with a tutor',
      description: 'Feynman-style teaching',
      color: '#f59e0b'
    }
  ];

  function selectMode(mode: ReviewMode) {
    selectedMode = mode;
    sessionActive = true;

    // Load queue for manual mode
    if (mode === 'manual') {
      reviewStore.loadQueue($settingsStore.dailyCardLimit);
    }
  }

  function requestSwitchMode(direction: 'prev' | 'next') {
    if (!selectedMode) return;

    // Show confirmation if session is active and user has started reviewing
    if (sessionActive) {
      pendingSwitchDirection = direction;
      showSwitchConfirm = true;
    } else {
      performSwitchMode(direction);
    }
  }

  function performSwitchMode(direction: 'prev' | 'next') {
    if (!selectedMode) return;
    const currentIndex = modes.findIndex(m => m.id === selectedMode);
    let newIndex: number;

    if (direction === 'prev') {
      newIndex = currentIndex === 0 ? modes.length - 1 : currentIndex - 1;
    } else {
      newIndex = currentIndex === modes.length - 1 ? 0 : currentIndex + 1;
    }

    selectMode(modes[newIndex].id);
    showSwitchConfirm = false;
    pendingSwitchDirection = null;
  }

  function confirmSwitch() {
    if (pendingSwitchDirection) {
      performSwitchMode(pendingSwitchDirection);
    }
  }

  function cancelSwitch() {
    showSwitchConfirm = false;
    pendingSwitchDirection = null;
  }

  function backToSelection() {
    selectedMode = null;
    sessionActive = false;
  }

  function handleRate(rating: ReviewRating) {
    reviewStore.submitReview(rating);
  }

  function handleSessionComplete(stats: { cards_reviewed: number; correct_count: number; accuracy: number }) {
    console.log('Session completed:', stats);
  }

  function getCurrentModeInfo() {
    return modes.find(m => m.id === selectedMode);
  }
</script>

<svelte:head>
  <title>Review - Engram</title>
</svelte:head>

<div class="review-page">
  {#if !sessionActive}
    <!-- Mode Selection Screen -->
    <div class="mode-selection">
      <header class="selection-header">
        <h1>Choose Your Review Style</h1>
        <p>How would you like to study today?</p>
      </header>

      <div class="mode-cards">
        {#each modes as mode}
          {@const isVoiceMode = mode.id === 'oral' || mode.id === 'conversational'}
          {@const isDisabled = isVoiceMode && !isOnline}
          <button
            class="mode-card"
            class:disabled={isDisabled}
            style="--mode-color: {mode.color}"
            onclick={() => !isDisabled && selectMode(mode.id)}
            disabled={isDisabled}
          >
            <div class="mode-icon">{mode.icon}</div>
            <h2 class="mode-name">{mode.name}</h2>
            <p class="mode-tagline">{mode.tagline}</p>
            <p class="mode-description">{mode.description}</p>
            {#if isVoiceMode && !isOnline}
              <div class="offline-indicator">
                <span class="offline-icon">&#9888;</span>
                <span>Requires internet connection</span>
              </div>
            {:else}
              <div class="mode-arrow">→</div>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {:else}
    <!-- Active Review Session -->
    <div class="review-session">
      <!-- Side Navigation - Previous Mode -->
      <button
        class="mode-nav mode-nav-prev"
        onclick={() => requestSwitchMode('prev')}
        title="Switch to {modes[(modes.findIndex(m => m.id === selectedMode) - 1 + modes.length) % modes.length].name}"
      >
        <span class="nav-icon">‹</span>
        <span class="nav-label">{modes[(modes.findIndex(m => m.id === selectedMode) - 1 + modes.length) % modes.length].icon}</span>
      </button>

      <!-- Main Review Content -->
      <div class="review-content">
        <header class="review-header">
          <button class="back-btn" onclick={backToSelection}>
            ← Back
          </button>
          <div class="current-mode" style="--mode-color: {getCurrentModeInfo()?.color}">
            <span class="mode-emoji">{getCurrentModeInfo()?.icon}</span>
            <span class="mode-title">{getCurrentModeInfo()?.name} Mode</span>
          </div>
          {#if selectedMode === 'manual' && $hasCardsToReview}
            <div class="progress">
              <span>{$reviewProgress.current} / {$reviewProgress.total}</span>
            </div>
          {:else}
            <div class="progress-placeholder"></div>
          {/if}
        </header>

        <div class="review-area">
          {#if selectedMode === 'oral'}
            <VoiceReview onSessionComplete={handleSessionComplete} />
          {:else if selectedMode === 'conversational'}
            <ConversationalReview onSessionComplete={handleSessionComplete} />
          {:else}
            <!-- Manual Mode -->
            {#if $reviewStore.loading}
              <div class="status-message">
                <div class="loading-spinner"></div>
                <span>Loading cards...</span>
              </div>
            {:else if $reviewStore.error}
              <div class="status-message error">{$reviewStore.error}</div>
            {:else if $reviewProgress.completed}
              <div class="completed">
                <div class="completed-icon">✓</div>
                <h2>Session Complete!</h2>
                <p>You've reviewed all {$reviewProgress.total} cards.</p>
                <div class="completed-actions">
                  <button class="btn btn-primary" onclick={() => reviewStore.loadQueue()}>
                    Review More
                  </button>
                  <button class="btn btn-secondary" onclick={backToSelection}>
                    Change Mode
                  </button>
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
                <div class="empty-icon">📚</div>
                <h2>No cards due for review</h2>
                <p>Come back later or add more cards.</p>
                <a href="/cards" class="btn btn-primary">Add Cards</a>
              </div>
            {/if}
          {/if}
        </div>
      </div>

      <!-- Side Navigation - Next Mode -->
      <button
        class="mode-nav mode-nav-next"
        onclick={() => requestSwitchMode('next')}
        title="Switch to {modes[(modes.findIndex(m => m.id === selectedMode) + 1) % modes.length].name}"
      >
        <span class="nav-label">{modes[(modes.findIndex(m => m.id === selectedMode) + 1) % modes.length].icon}</span>
        <span class="nav-icon">›</span>
      </button>
    </div>
  {/if}

  <!-- Mode Switch Confirmation Modal -->
  {#if showSwitchConfirm}
    <div class="modal-overlay" onclick={cancelSwitch}>
      <div class="modal-content" onclick={(e) => e.stopPropagation()}>
        <h3>Switch modes?</h3>
        <p>Your current progress will be lost if you switch to another review mode.</p>
        <div class="modal-actions">
          <button class="btn btn-secondary" onclick={cancelSwitch}>Cancel</button>
          <button class="btn btn-primary" onclick={confirmSwitch}>Switch Mode</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .review-page {
    min-height: calc(100vh - 120px);
  }

  /* Mode Selection Screen */
  .mode-selection {
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  .selection-header {
    text-align: center;
    margin-bottom: 3rem;
  }

  .selection-header h1 {
    font-size: 2rem;
    font-weight: 700;
    margin-bottom: 0.5rem;
    background: linear-gradient(135deg, var(--primary), #8b5cf6);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .selection-header p {
    color: var(--text-secondary);
    font-size: 1.125rem;
  }

  .mode-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 1.5rem;
  }

  .mode-card {
    position: relative;
    background: var(--surface);
    border: 2px solid var(--border);
    border-radius: 20px;
    padding: 2rem;
    text-align: center;
    cursor: pointer;
    transition: all 0.3s ease;
    overflow: hidden;
  }

  .mode-card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 4px;
    background: var(--mode-color);
    transform: scaleX(0);
    transition: transform 0.3s ease;
  }

  .mode-card:hover {
    border-color: var(--mode-color);
    transform: translateY(-4px);
    box-shadow: 0 12px 24px -8px rgba(0, 0, 0, 0.15);
  }

  .mode-card:hover::before {
    transform: scaleX(1);
  }

  .mode-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    filter: grayscale(0.2);
    transition: filter 0.3s;
  }

  .mode-card:hover .mode-icon {
    filter: grayscale(0);
    transform: scale(1.1);
  }

  .mode-name {
    font-size: 1.5rem;
    font-weight: 700;
    margin-bottom: 0.25rem;
    color: var(--text);
  }

  .mode-tagline {
    font-size: 1rem;
    font-weight: 500;
    color: var(--mode-color);
    margin-bottom: 0.5rem;
  }

  .mode-description {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }

  .mode-arrow {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background: var(--background);
    border-radius: 50%;
    font-size: 1.25rem;
    color: var(--text-secondary);
    transition: all 0.3s;
  }

  .mode-card:hover .mode-arrow {
    background: var(--mode-color);
    color: white;
    transform: translateX(4px);
  }

  /* Active Review Session */
  .review-session {
    display: flex;
    min-height: calc(100vh - 120px);
    position: relative;
  }

  .mode-nav {
    position: fixed;
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    padding: 0.75rem 0.5rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s;
    z-index: 10;
    opacity: 0.7;
  }

  .mode-nav:hover {
    opacity: 1;
    background: var(--primary);
    color: white;
    border-color: var(--primary);
  }

  .mode-nav-prev {
    left: 1rem;
    border-top-left-radius: 20px;
    border-bottom-left-radius: 20px;
  }

  .mode-nav-next {
    right: 1rem;
    border-top-right-radius: 20px;
    border-bottom-right-radius: 20px;
  }

  .nav-icon {
    font-size: 1.5rem;
    line-height: 1;
    font-weight: 300;
  }

  .nav-label {
    font-size: 1.25rem;
  }

  .review-content {
    flex: 1;
    max-width: 800px;
    margin: 0 auto;
    padding: 1rem 4rem;
    width: 100%;
  }

  .review-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border);
  }

  .back-btn {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s;
  }

  .back-btn:hover {
    background: var(--background);
    color: var(--text);
  }

  .current-mode {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: var(--surface);
    border: 2px solid var(--mode-color);
    border-radius: 24px;
  }

  .mode-emoji {
    font-size: 1.25rem;
  }

  .mode-title {
    font-weight: 600;
    color: var(--text);
  }

  .progress {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .progress-placeholder {
    width: 60px;
  }

  .review-area {
    min-height: 400px;
  }

  /* Status Messages */
  .status-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 4rem 2rem;
    color: var(--text-secondary);
  }

  .status-message.error {
    color: var(--danger);
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Completed State */
  .completed {
    text-align: center;
    padding: 4rem 2rem;
  }

  .completed-icon {
    width: 80px;
    height: 80px;
    background: var(--success);
    color: white;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2.5rem;
    margin: 0 auto 1.5rem;
  }

  .completed h2 {
    font-size: 1.75rem;
    margin-bottom: 0.5rem;
  }

  .completed p {
    color: var(--text-secondary);
    margin-bottom: 2rem;
  }

  .completed-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
  }

  /* Empty State */
  .empty {
    text-align: center;
    padding: 4rem 2rem;
  }

  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
    opacity: 0.8;
  }

  .empty h2 {
    font-size: 1.5rem;
    margin-bottom: 0.5rem;
  }

  .empty p {
    color: var(--text-secondary);
    margin-bottom: 1.5rem;
  }

  /* Buttons */
  .btn {
    display: inline-block;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 500;
    text-decoration: none;
    border-radius: 8px;
    border: none;
    cursor: pointer;
    transition: all 0.2s;
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
    background: var(--background);
  }

  /* Offline indicator */
  .offline-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: rgba(255, 193, 7, 0.1);
    border-radius: 20px;
    font-size: 0.75rem;
    color: var(--warning, #ffc107);
  }

  .offline-icon {
    font-size: 0.875rem;
  }

  .mode-card.disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .mode-card.disabled:hover {
    transform: none;
    border-color: var(--border);
    box-shadow: none;
  }

  .mode-card.disabled:hover::before {
    transform: scaleX(0);
  }

  /* Modal */
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fadeIn 0.2s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal-content {
    background: var(--surface);
    border-radius: 16px;
    padding: 2rem;
    max-width: 400px;
    width: 90%;
    text-align: center;
    animation: slideUp 0.2s ease;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .modal-content h3 {
    font-size: 1.25rem;
    margin-bottom: 0.5rem;
  }

  .modal-content p {
    color: var(--text-secondary);
    margin-bottom: 1.5rem;
  }

  .modal-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .mode-cards {
      grid-template-columns: 1fr;
    }

    .mode-nav {
      padding: 0.5rem;
    }

    .mode-nav-prev {
      left: 0.5rem;
    }

    .mode-nav-next {
      right: 0.5rem;
    }

    .nav-label {
      display: none;
    }

    .review-content {
      padding: 1rem 3rem;
    }

    .review-header {
      flex-wrap: wrap;
      gap: 0.5rem;
    }

    .progress-placeholder {
      display: none;
    }
  }
</style>
