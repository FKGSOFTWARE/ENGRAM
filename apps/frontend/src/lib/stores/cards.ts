import { writable, derived } from 'svelte/store';
import type { Card, CreateCard, UpdateCard } from '@engram/shared';
import { db, type LocalCard, getAllCards, getDueCards } from '$lib/db';
import { api } from '$lib/api/client';

// Store state
interface CardsState {
  cards: LocalCard[];
  loading: boolean;
  error: string | null;
}

function createCardsStore() {
  const { subscribe, set, update } = writable<CardsState>({
    cards: [],
    loading: false,
    error: null
  });

  return {
    subscribe,

    // Load cards from IndexedDB
    async loadFromLocal() {
      update((s) => ({ ...s, loading: true, error: null }));
      try {
        const cards = await getAllCards();
        update((s) => ({ ...s, cards, loading: false }));
      } catch (e) {
        update((s) => ({
          ...s,
          loading: false,
          error: e instanceof Error ? e.message : 'Failed to load cards'
        }));
      }
    },

    // Sync with server (if available)
    async syncWithServer() {
      update((s) => ({ ...s, loading: true, error: null }));
      try {
        const serverCards = await api.listCards();
        // Clear and repopulate IndexedDB
        await db.cards.clear();
        for (const card of serverCards) {
          await db.cards.add({ ...card, _synced: true });
        }
        const cards = await getAllCards();
        update((s) => ({ ...s, cards, loading: false }));
      } catch (e) {
        // If server is unavailable, just load from local
        const cards = await getAllCards();
        update((s) => ({
          ...s,
          cards,
          loading: false,
          error: 'Offline - using local data'
        }));
      }
    },

    async create(data: CreateCard) {
      const localId = crypto.randomUUID();
      try {
        // Create locally first
        const now = new Date().toISOString();
        const localCard: LocalCard = {
          id: localId,
          front: data.front,
          back: data.back,
          source_id: data.source_id ?? null,
          ease_factor: 2.5,
          interval: 0,
          repetitions: 0,
          next_review: now,
          created_at: now,
          updated_at: now,
          _synced: false
        };
        await db.cards.add(localCard);

        // Add to store immediately for responsive UI
        update((s) => ({ ...s, cards: [...s.cards, localCard] }));

        // Try to sync with server
        try {
          const serverCard = await api.createCard(data);
          // Use a transaction to atomically replace local card with server version
          await db.transaction('rw', db.cards, async () => {
            await db.cards.delete(localId);
            await db.cards.add({ ...serverCard, _synced: true });
          });
          // Update store with server card
          update((s) => ({
            ...s,
            cards: s.cards.map((c) => (c.id === localId ? { ...serverCard, _synced: true } : c))
          }));
        } catch (syncError) {
          // Offline - local card is already in store, just update error state
          update((s) => ({
            ...s,
            error: 'Offline - card saved locally, will sync when online'
          }));
        }
      } catch (e) {
        update((s) => ({
          ...s,
          error: e instanceof Error ? e.message : 'Failed to create card'
        }));
      }
    },

    async update(id: string, data: UpdateCard) {
      try {
        const now = new Date().toISOString();
        await db.cards.update(id, { ...data, updated_at: now, _synced: false });

        // Try to sync with server
        try {
          const serverCard = await api.updateCard(id, data);
          await db.cards.update(id, { ...serverCard, _synced: true });
        } catch {
          // Offline - local update is enough
        }

        const cards = await getAllCards();
        update((s) => ({ ...s, cards }));
      } catch (e) {
        update((s) => ({
          ...s,
          error: e instanceof Error ? e.message : 'Failed to update card'
        }));
      }
    },

    async delete(id: string) {
      try {
        // Soft delete locally
        await db.cards.update(id, { _deleted: true, _synced: false });

        // Try to sync with server
        try {
          await api.deleteCard(id);
          await db.cards.delete(id);
        } catch {
          // Offline - keep soft deleted
        }

        update((s) => ({ ...s, cards: s.cards.filter((c) => c.id !== id) }));
      } catch (e) {
        update((s) => ({
          ...s,
          error: e instanceof Error ? e.message : 'Failed to delete card'
        }));
      }
    },

    clearError() {
      update((s) => ({ ...s, error: null }));
    }
  };
}

export const cardsStore = createCardsStore();

// Derived store for card count
export const cardCount = derived(cardsStore, ($cards) => $cards.cards.length);
