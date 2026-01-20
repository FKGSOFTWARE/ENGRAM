import { writable, derived } from 'svelte/store';
import type { Card, ReviewRating, SubmitReview } from '@engram/shared';
import { db, type LocalCard, getDueCards, recordReview, type LocalReview } from '$lib/db';
import { api } from '$lib/api/client';

interface ReviewState {
  queue: LocalCard[];
  currentIndex: number;
  loading: boolean;
  error: string | null;
  showingAnswer: boolean;
}

function createReviewStore() {
  const { subscribe, set, update } = writable<ReviewState>({
    queue: [],
    currentIndex: 0,
    loading: false,
    error: null,
    showingAnswer: false
  });

  return {
    subscribe,

    async loadQueue(limit = 10) {
      update((s) => ({ ...s, loading: true, error: null }));
      try {
        // Try server first
        try {
          const serverCards = await api.getNextCards(limit);
          update((s) => ({
            ...s,
            queue: serverCards.map((c) => ({ ...c, _synced: true })),
            currentIndex: 0,
            loading: false,
            showingAnswer: false
          }));
        } catch {
          // Fallback to local
          const localCards = await getDueCards(limit);
          update((s) => ({
            ...s,
            queue: localCards,
            currentIndex: 0,
            loading: false,
            showingAnswer: false
          }));
        }
      } catch (e) {
        update((s) => ({
          ...s,
          loading: false,
          error: e instanceof Error ? e.message : 'Failed to load review queue'
        }));
      }
    },

    showAnswer() {
      update((s) => ({ ...s, showingAnswer: true }));
    },

    async submitReview(rating: ReviewRating, userAnswer?: string) {
      let currentCard: LocalCard | undefined;
      const unsubscribe = subscribe((s) => {
        currentCard = s.queue[s.currentIndex];
      });
      unsubscribe();

      if (!currentCard) return;

      try {
        const reviewData: SubmitReview = {
          card_id: currentCard.id,
          rating,
          user_answer: userAnswer
        };

        // Record locally
        const localReview: LocalReview = {
          id: crypto.randomUUID(),
          card_id: currentCard.id,
          rating: ['again', 'hard', 'good', 'easy'].indexOf(rating),
          user_answer: userAnswer ?? null,
          llm_evaluation: null,
          reviewed_at: new Date().toISOString(),
          _synced: false
        };
        await recordReview(localReview);

        // Update card scheduling locally (simplified SM-2)
        const now = new Date();
        let newInterval: number;
        let newEaseFactor = currentCard.ease_factor;
        let newRepetitions = currentCard.repetitions;

        switch (rating) {
          case 'again':
            newInterval = 1;
            newRepetitions = 0;
            newEaseFactor = Math.max(1.3, newEaseFactor - 0.2);
            break;
          case 'hard':
            newInterval = Math.max(1, Math.floor(currentCard.interval * 0.8));
            newRepetitions++;
            break;
          case 'good':
            if (newRepetitions === 0) newInterval = 1;
            else if (newRepetitions === 1) newInterval = 6;
            else newInterval = Math.floor(currentCard.interval * newEaseFactor);
            newRepetitions++;
            break;
          case 'easy':
            if (newRepetitions === 0) newInterval = 4;
            else if (newRepetitions === 1) newInterval = 10;
            else newInterval = Math.floor(currentCard.interval * newEaseFactor * 1.3);
            newRepetitions++;
            newEaseFactor = Math.min(3.0, newEaseFactor + 0.1);
            break;
        }

        const nextReview = new Date(now.getTime() + newInterval * 24 * 60 * 60 * 1000);
        await db.cards.update(currentCard.id, {
          interval: newInterval,
          ease_factor: newEaseFactor,
          repetitions: newRepetitions,
          next_review: nextReview.toISOString(),
          _synced: false
        });

        // Try to sync with server
        try {
          await api.submitReview(reviewData);
        } catch {
          // Offline - local update is enough
        }

        // Move to next card
        update((s) => ({
          ...s,
          currentIndex: s.currentIndex + 1,
          showingAnswer: false
        }));
      } catch (e) {
        update((s) => ({
          ...s,
          error: e instanceof Error ? e.message : 'Failed to submit review'
        }));
      }
    },

    reset() {
      set({
        queue: [],
        currentIndex: 0,
        loading: false,
        error: null,
        showingAnswer: false
      });
    }
  };
}

export const reviewStore = createReviewStore();

// Derived stores
export const currentCard = derived(reviewStore, ($review) =>
  $review.queue[$review.currentIndex]
);

export const reviewProgress = derived(reviewStore, ($review) => ({
  current: $review.currentIndex + 1,
  total: $review.queue.length,
  completed: $review.currentIndex >= $review.queue.length
}));

export const hasCardsToReview = derived(
  reviewStore,
  ($review) => $review.queue.length > 0 && $review.currentIndex < $review.queue.length
);
