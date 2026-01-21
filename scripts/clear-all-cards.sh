#!/bin/bash
# Clear all flashcards from the database
#
# Usage: ./scripts/clear-all-cards.sh
#
# After running, refresh the home page to see the changes.
# The frontend syncs with the backend on page load.

API_URL="${API_URL:-http://localhost:3001/api/cards}"

echo "Fetching all cards from $API_URL..."

# Get all card IDs
CARD_IDS=$(curl -s "$API_URL" | jq -r '.[].id' 2>/dev/null)

if [ -z "$CARD_IDS" ]; then
  echo "No cards found."
  exit 0
fi

# Count cards
COUNT=$(echo "$CARD_IDS" | wc -l)
echo "Found $COUNT card(s). Deleting..."

# Delete each card
for ID in $CARD_IDS; do
  curl -s -X DELETE "$API_URL/$ID" > /dev/null
  echo "Deleted: $ID"
done

echo ""
echo "Done! Cleared all $COUNT card(s)."
