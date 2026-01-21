#!/bin/bash
# Add 5 test flashcards for voice review testing
#
# Usage: ./scripts/add-test-cards.sh
#
# After running, refresh the home page to see the cards.
# The frontend syncs with the backend on page load.

API_URL="${API_URL:-http://localhost:3001/api/cards}"

echo "Adding 5 test cards to $API_URL..."

# Card 1: Simple fact
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{"front": "What is the capital of France?", "back": "Paris"}' \
  | jq -r '"Created: \(.front)"' 2>/dev/null || echo "Created card 1"

# Card 2: Science question
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{"front": "What is the chemical symbol for water?", "back": "H2O"}' \
  | jq -r '"Created: \(.front)"' 2>/dev/null || echo "Created card 2"

# Card 3: Math concept
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{"front": "What is the square root of 144?", "back": "12"}' \
  | jq -r '"Created: \(.front)"' 2>/dev/null || echo "Created card 3"

# Card 4: History question
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{"front": "In what year did World War II end?", "back": "1945"}' \
  | jq -r '"Created: \(.front)"' 2>/dev/null || echo "Created card 4"

# Card 5: Language/vocabulary
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{"front": "What does the word \"ephemeral\" mean?", "back": "Lasting for a very short time; short-lived or transitory"}' \
  | jq -r '"Created: \(.front)"' 2>/dev/null || echo "Created card 5"

echo ""
echo "Done! Added 5 test cards."
