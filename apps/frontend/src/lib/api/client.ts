import type { Card, CreateCard, UpdateCard, SubmitReview } from '@engram/shared';

const API_BASE = '/api';

class ApiClient {
  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
    const response = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers
      }
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || `HTTP ${response.status}`);
    }

    // Handle 204 No Content
    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
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
