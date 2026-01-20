import type { VoiceSessionMessage, VoiceSessionResponse, SessionState } from '@engram/shared';

type MessageHandler = (message: VoiceSessionResponse) => void;
type StateChangeHandler = (state: 'connecting' | 'connected' | 'disconnected' | 'error') => void;

// Voice service endpoint configuration
// In production, these would be proxied through the same host
// For development, the Python voice service runs separately
const VOICE_SERVICE_PORT = import.meta.env.VITE_VOICE_SERVICE_PORT || '8001';
const USE_PYTHON_VOICE = import.meta.env.VITE_USE_PYTHON_VOICE === 'true';

export class VoiceWebSocket {
  private ws: WebSocket | null = null;
  private messageHandlers: Set<MessageHandler> = new Set();
  private stateHandlers: Set<StateChangeHandler> = new Set();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  constructor(private url: string) {}

  connect(): void {
    this.notifyStateChange('connecting');

    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.notifyStateChange('connected');
      };

      this.ws.onmessage = (event) => {
        try {
          const message: VoiceSessionResponse = JSON.parse(event.data);
          this.messageHandlers.forEach((handler) => handler(message));
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      this.ws.onclose = () => {
        this.notifyStateChange('disconnected');
        this.attemptReconnect();
      };

      this.ws.onerror = () => {
        this.notifyStateChange('error');
      };
    } catch (e) {
      console.error('Failed to create WebSocket:', e);
      this.notifyStateChange('error');
    }
  }

  disconnect(): void {
    this.reconnectAttempts = this.maxReconnectAttempts; // Prevent reconnection
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  send(message: VoiceSessionMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  sendAudioChunk(chunk: ArrayBuffer): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(chunk);
    }
  }

  sendEndAudio(): void {
    this.send({ type: 'end_audio' } as VoiceSessionMessage);
  }

  startSession(cardLimit = 10): void {
    this.send({ type: 'start_session', card_limit: cardLimit } as VoiceSessionMessage);
  }

  submitTextAnswer(answer: string): void {
    this.send({ type: 'text_answer', answer } as VoiceSessionMessage);
  }

  rateCard(rating: 'again' | 'hard' | 'good' | 'easy'): void {
    const ratingMap = { again: 0, hard: 1, good: 2, easy: 3 };
    this.send({ type: 'rate_card', rating: ratingMap[rating] } as VoiceSessionMessage);
  }

  nextCard(): void {
    this.send({ type: 'next_card' } as VoiceSessionMessage);
  }

  onMessage(handler: MessageHandler): () => void {
    this.messageHandlers.add(handler);
    return () => this.messageHandlers.delete(handler);
  }

  onStateChange(handler: StateChangeHandler): () => void {
    this.stateHandlers.add(handler);
    return () => this.stateHandlers.delete(handler);
  }

  private notifyStateChange(state: 'connecting' | 'connected' | 'disconnected' | 'error'): void {
    this.stateHandlers.forEach((handler) => handler(state));
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

    setTimeout(() => {
      console.log(`Reconnecting... attempt ${this.reconnectAttempts}`);
      this.connect();
    }, delay);
  }
}

// Singleton instance (created when needed)
let voiceWs: VoiceWebSocket | null = null;

export function getVoiceWebSocket(): VoiceWebSocket {
  if (!voiceWs) {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';

    // Determine voice service URL based on configuration
    let wsUrl: string;
    if (USE_PYTHON_VOICE) {
      // Connect to Python voice service (Pipecat with STT/TTS)
      const hostname = window.location.hostname;
      wsUrl = `${protocol}//${hostname}:${VOICE_SERVICE_PORT}/ws/voice/stream`;
    } else {
      // Connect to Rust backend WebSocket (text-only evaluation)
      const host = window.location.host;
      wsUrl = `${protocol}//${host}/api/ws`;
    }

    voiceWs = new VoiceWebSocket(wsUrl);
  }
  return voiceWs;
}

/**
 * Reset the voice WebSocket singleton (useful for configuration changes)
 */
export function resetVoiceWebSocket(): void {
  if (voiceWs) {
    voiceWs.disconnect();
    voiceWs = null;
  }
}
