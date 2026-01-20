-- Migration 002: Create cards table
-- This table stores flashcards with FSRS scheduling data

CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY NOT NULL,
    front TEXT NOT NULL,
    back TEXT NOT NULL,
    source_id TEXT,
    -- FSRS fields (backward compatible with SM-2 ease_factor)
    ease_factor REAL NOT NULL DEFAULT 2.5,
    interval INTEGER NOT NULL DEFAULT 0,
    repetitions INTEGER NOT NULL DEFAULT 0,
    -- FSRS specific fields
    stability REAL NOT NULL DEFAULT 0.0,
    difficulty REAL NOT NULL DEFAULT 5.0,
    lapses INTEGER NOT NULL DEFAULT 0,
    -- Scheduling
    next_review TEXT NOT NULL,
    last_review TEXT,
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Foreign key
    FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE SET NULL
);

-- Index for efficient review queue queries
CREATE INDEX IF NOT EXISTS idx_cards_next_review ON cards(next_review);

-- Index for source lookups
CREATE INDEX IF NOT EXISTS idx_cards_source ON cards(source_id);
