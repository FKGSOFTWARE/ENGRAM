-- Migration 003: Create reviews table
-- This table stores review history for analytics and LLM evaluation results

CREATE TABLE IF NOT EXISTS reviews (
    id TEXT PRIMARY KEY NOT NULL,
    card_id TEXT NOT NULL,
    rating INTEGER NOT NULL,
    -- Review details
    user_answer TEXT,
    llm_evaluation TEXT,
    response_time_ms INTEGER,
    -- Session tracking
    session_id TEXT,
    -- Timestamp
    reviewed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Foreign key
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

-- Index for card review history
CREATE INDEX IF NOT EXISTS idx_reviews_card ON reviews(card_id);

-- Index for session grouping
CREATE INDEX IF NOT EXISTS idx_reviews_session ON reviews(session_id);

-- Index for analytics queries
CREATE INDEX IF NOT EXISTS idx_reviews_date ON reviews(reviewed_at);
