import type { Card, CreateCard, UpdateCard, SubmitReview } from '@engram/shared';

const API_BASE = '/api';
const DEFAULT_TIMEOUT_MS = 30000; // 30 seconds

class ApiClient {
  private async request<T>(
    path: string,
    options: RequestInit = {},
    timeoutMs = DEFAULT_TIMEOUT_MS
  ): Promise<T> {
    // Create abort controller for timeout
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

    try {
      const response = await fetch(`${API_BASE}${path}`, {
        ...options,
        signal: controller.signal,
        headers: {
          'Content-Type': 'application/json',
          ...options.headers
        }
      });

      if (!response.ok) {
        const contentType = response.headers.get('content-type');
        let error: string;
        if (contentType?.includes('application/json')) {
          const json = await response.json();
          error = json.message || json.error || `HTTP ${response.status}`;
        } else {
          error = (await response.text()) || `HTTP ${response.status}`;
        }
        throw new Error(error);
      }

      // Handle 204 No Content
      if (response.status === 204) {
        return undefined as T;
      }

      return response.json();
    } catch (e) {
      if (e instanceof Error && e.name === 'AbortError') {
        throw new Error(`Request timed out after ${timeoutMs}ms`);
      }
      throw e;
    } finally {
      clearTimeout(timeoutId);
    }
  }

  // Cards
  async listCards(): Promise<Card[]> {
    return this.request<Card[]>('/cards');
  }

  async getCard(id: string): Promise<Card> {
    return this.request<Card>(`/cards/${id}`);
  }

  async createCard(data: CreateCard): Promise<Card> {
    return this.request<Card>('/cards', {
      method: 'POST',
      body: JSON.stringify(data)
    });
  }

  async updateCard(id: string, data: UpdateCard): Promise<Card> {
    return this.request<Card>(`/cards/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data)
    });
  }

  async deleteCard(id: string): Promise<void> {
    return this.request<void>(`/cards/${id}`, {
      method: 'DELETE'
    });
  }

  // Review
  async getNextCards(limit = 10): Promise<Card[]> {
    return this.request<Card[]>(`/review/next?limit=${limit}`);
  }

  async submitReview(data: SubmitReview): Promise<Card> {
    return this.request<Card>('/review/submit', {
      method: 'POST',
      body: JSON.stringify(data)
    });
  }
}

export const api = new ApiClient();
