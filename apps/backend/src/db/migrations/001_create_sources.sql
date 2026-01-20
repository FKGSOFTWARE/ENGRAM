-- Migration 001: Create sources table
-- This table stores the origin of flashcard content (manual entry, URL, PDF, etc.)

CREATE TABLE IF NOT EXISTS sources (
    id TEXT PRIMARY KEY NOT NULL,
    source_type TEXT NOT NULL,
    title TEXT,
    url TEXT,
    content_hash TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for finding sources by type
CREATE INDEX IF NOT EXISTS idx_sources_type ON sources(source_type);
