import type { VoiceSessionMessage, VoiceSessionResponse, SessionState } from '@engram/shared';

type MessageHandler = (message: VoiceSessionResponse) => void;
type StateChangeHandler = (state: 'connecting' | 'connected' | 'disconnected' | 'error') => void;

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
    const host = window.location.host;
    voiceWs = new VoiceWebSocket(`${protocol}//${host}/api/ws`);
  }
  return voiceWs;
}
