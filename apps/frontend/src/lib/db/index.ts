import Dexie, { type EntityTable } from 'dexie';
import type { Card, Review, Source } from '@engram/shared';

// Extend Card type with sync metadata
export interface LocalCard extends Card {
  _synced?: boolean;
  _deleted?: boolean;
}

export interface LocalReview extends Review {
  _synced?: boolean;
}

export interface LocalSource extends Source {
  _synced?: boolean;
}

const db = new Dexie('engram') as Dexie & {
  cards: EntityTable<LocalCard, 'id'>;
  reviews: EntityTable<LocalReview, 'id'>;
  sources: EntityTable<LocalSource, 'id'>;
};

db.version(1).stores({
  cards: 'id, front, back, source_id, next_review, created_at, _synced, _deleted',
  reviews: 'id, card_id, reviewed_at, _synced, [card_id+reviewed_at]',
  sources: 'id, source_type, created_at, _synced'
});

export { db };

// Helper functions for card operations
export async function getAllCards(): Promise<LocalCard[]> {
  return db.cards.filter((card) => !card._deleted).toArray();
}

export async function getCard(id: string): Promise<LocalCard | undefined> {
  const card = await db.cards.get(id);
  return card?._deleted ? undefined : card;
}

export async function createCard(card: LocalCard): Promise<string> {
  return db.cards.add(card);
}

export async function updateCard(id: string, updates: Partial<LocalCard>): Promise<void> {
  await db.cards.update(id, { ...updates, updated_at: new Date().toISOString() });
}

export async function deleteCard(id: string): Promise<void> {
  // Soft delete for sync
  await db.cards.update(id, { _deleted: true, _synced: false });
}

export async function getDueCards(limit = 10): Promise<LocalCard[]> {
  const now = new Date().toISOString();
  return db.cards
    .filter((card) => !card._deleted && card.next_review <= now)
    .limit(limit)
    .toArray();
}

export async function recordReview(review: LocalReview): Promise<string> {
  return db.reviews.add(review);
}

// Sync helpers (for future server sync)
export async function getUnsyncedCards(): Promise<LocalCard[]> {
  return db.cards.filter((card) => !card._synced).toArray();
}

export async function markCardSynced(id: string): Promise<void> {
  await db.cards.update(id, { _synced: true });
}
