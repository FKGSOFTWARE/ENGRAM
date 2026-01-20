// Types shared between frontend and backend

export interface Card {
  id: string;
  front: string;
  back: string;
  source_id: string | null;
  ease_factor: number;
  interval: number;
  repetitions: number;
  next_review: string; // ISO date string
  created_at: string;
  updated_at: string;
}

export interface CreateCard {
  front: string;
  back: string;
  source_id?: string;
}

export interface UpdateCard {
  front?: string;
  back?: string;
}

export type ReviewRating = 'again' | 'hard' | 'good' | 'easy';

export interface SubmitReview {
  card_id: string;
  rating: ReviewRating;
  user_answer?: string;
}

export interface Review {
  id: string;
  card_id: string;
  rating: number;
  user_answer: string | null;
  llm_evaluation: string | null;
  reviewed_at: string;
}

export type SourceType = 'manual' | 'text' | 'pdf' | 'url';

export interface Source {
  id: string;
  source_type: SourceType;
  title: string | null;
  url: string | null;
  content_hash: string | null;
  created_at: string;
}

// Voice session types (for Phase 4)
export type SessionState =
  | 'idle'
  | 'presenting_card'
  | 'waiting_for_answer'
  | 'evaluating'
  | 'showing_feedback';

export interface VoiceSessionMessage {
  type:
    | 'start_session'
    | 'audio_chunk'
    | 'end_audio'
    | 'command'
    | 'text_answer'
    | 'rate_card'
    | 'next_card';
  data?: unknown;
  card_limit?: number;
  answer?: string;
  rating?: ReviewRating;
}

export interface VoiceSessionResponse {
  type:
    | 'card_presented'
    | 'audio_chunk'
    | 'evaluation'
    | 'session_state'
    | 'session_started'
    | 'session_ended'
    | 'session_complete'
    | 'state_changed'
    | 'error';
  data?: unknown;
}
