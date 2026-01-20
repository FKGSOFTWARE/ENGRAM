"""
REST API client for communicating with the ENGRAM Rust backend.

Handles all API calls for cards, reviews, and LLM evaluation.
"""

import logging
from typing import Any, Optional

import httpx

logger = logging.getLogger(__name__)


class EngramAPIClient:
    """HTTP client for ENGRAM backend API."""

    def __init__(self, base_url: str, timeout: float = 30.0):
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self._client: Optional[httpx.AsyncClient] = None

    async def _get_client(self) -> httpx.AsyncClient:
        """Get or create HTTP client."""
        if self._client is None:
            self._client = httpx.AsyncClient(
                base_url=self.base_url,
                timeout=self.timeout,
                headers={"Content-Type": "application/json"},
            )
        return self._client

    async def close(self) -> None:
        """Close the HTTP client."""
        if self._client:
            await self._client.aclose()
            self._client = None

    async def get_due_cards(
        self,
        deck_id: Optional[str] = None,
        limit: int = 20,
    ) -> list[dict[str, Any]]:
        """
        Get cards due for review.

        Args:
            deck_id: Optional deck ID to filter by
            limit: Maximum number of cards to return

        Returns:
            List of card objects with id, front, back, etc.
        """
        client = await self._get_client()

        params = {"limit": limit}
        if deck_id:
            params["deck_id"] = deck_id

        try:
            response = await client.get("/api/review/due", params=params)
            response.raise_for_status()
            return response.json()
        except httpx.HTTPStatusError as e:
            logger.error(f"Failed to get due cards: {e}")
            raise
        except Exception as e:
            logger.error(f"API error getting due cards: {e}")
            raise

    async def get_card(self, card_id: str) -> dict[str, Any]:
        """
        Get a single card by ID.

        Args:
            card_id: The card's UUID

        Returns:
            Card object with all fields
        """
        client = await self._get_client()

        try:
            response = await client.get(f"/api/cards/{card_id}")
            response.raise_for_status()
            return response.json()
        except httpx.HTTPStatusError as e:
            logger.error(f"Failed to get card {card_id}: {e}")
            raise

    async def submit_review(
        self,
        card_id: str,
        rating: int,
        response_time_ms: Optional[int] = None,
    ) -> dict[str, Any]:
        """
        Submit a review for a card.

        Args:
            card_id: The card's UUID
            rating: Rating 0-3 (Again, Hard, Good, Easy)
            response_time_ms: Optional time to respond in milliseconds

        Returns:
            Updated review state
        """
        client = await self._get_client()

        payload = {
            "card_id": card_id,
            "rating": rating,
        }
        if response_time_ms is not None:
            payload["response_time_ms"] = response_time_ms

        try:
            response = await client.post("/api/review", json=payload)
            response.raise_for_status()
            return response.json()
        except httpx.HTTPStatusError as e:
            logger.error(f"Failed to submit review for {card_id}: {e}")
            raise

    async def evaluate_answer(
        self,
        card_id: str,
        user_answer: str,
        prompt: Optional[str] = None,
    ) -> dict[str, Any]:
        """
        Request LLM evaluation of user's answer.

        Args:
            card_id: The card's UUID
            user_answer: The user's transcribed answer
            prompt: Optional custom evaluation prompt

        Returns:
            Evaluation result with rating and feedback
        """
        client = await self._get_client()

        payload = {
            "card_id": card_id,
            "user_answer": user_answer,
        }
        if prompt:
            payload["prompt"] = prompt

        try:
            response = await client.post("/api/review/evaluate", json=payload)
            response.raise_for_status()
            return response.json()
        except httpx.HTTPStatusError as e:
            logger.error(f"Failed to evaluate answer for {card_id}: {e}")
            raise

    async def get_decks(self) -> list[dict[str, Any]]:
        """
        Get all available decks.

        Returns:
            List of deck objects
        """
        client = await self._get_client()

        try:
            response = await client.get("/api/decks")
            response.raise_for_status()
            return response.json()
        except httpx.HTTPStatusError as e:
            logger.error(f"Failed to get decks: {e}")
            raise

    async def health_check(self) -> bool:
        """
        Check if the backend API is healthy.

        Returns:
            True if healthy, False otherwise
        """
        client = await self._get_client()

        try:
            response = await client.get("/health")
            return response.status_code == 200
        except Exception:
            return False
