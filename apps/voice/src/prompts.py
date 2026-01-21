"""
LLM evaluation prompts for voice-based answer assessment.

These prompts guide the LLM to evaluate spoken answers against
expected flashcard content, accounting for natural speech variations.
"""


def build_evaluation_prompt(
    question: str,
    expected_answer: str,
    user_answer: str,
    context: str | None = None,
) -> str:
    """
    Build an evaluation prompt for LLM assessment.

    Args:
        question: The flashcard question/front
        expected_answer: The expected answer/back
        user_answer: The user's transcribed spoken answer
        context: Optional additional context

    Returns:
        Formatted prompt string for LLM evaluation
    """
    context_section = f"\nAdditional Context: {context}" if context else ""

    return f"""You are evaluating a spoken answer to a flashcard question. Consider that the answer was transcribed from speech, so minor variations in wording, filler words, or small transcription errors should be tolerated.

Question: {question}

Expected Answer: {expected_answer}

User's Spoken Answer: {user_answer}
{context_section}
Evaluate the user's answer and provide:

1. A rating from 0-3:
   - 0 (Again): Answer is incorrect or shows no understanding
   - 1 (Hard): Answer is partially correct but has significant gaps
   - 2 (Good): Answer is correct with minor omissions or variations
   - 3 (Easy): Answer is complete and demonstrates clear understanding

2. Brief feedback (1-2 sentences) explaining your rating. Be encouraging but accurate.

Consider:
- Semantic equivalence (synonyms, paraphrasing are acceptable)
- Key concepts must be present for higher ratings
- Transcription artifacts (um, uh, repeated words) should be ignored
- Order of information can vary

Respond in JSON format:
{{
    "rating": <0-3>,
    "feedback": "<brief feedback>",
    "key_points_covered": ["<point1>", "<point2>", ...],
    "key_points_missing": ["<point1>", "<point2>", ...]
}}"""


def build_hint_prompt(
    question: str,
    expected_answer: str,
    previous_attempts: list[str] | None = None,
) -> str:
    """
    Build a prompt for generating progressive hints.

    Args:
        question: The flashcard question/front
        expected_answer: The expected answer/back
        previous_attempts: Optional list of previous wrong answers

    Returns:
        Formatted prompt for hint generation
    """
    attempts_section = ""
    if previous_attempts:
        attempts_list = "\n".join(f"- {a}" for a in previous_attempts)
        attempts_section = f"\nPrevious attempts:\n{attempts_list}"

    return f"""Generate a helpful hint for this flashcard question. The hint should guide the learner without giving away the complete answer.

Question: {question}

Expected Answer: {expected_answer}
{attempts_section}
Generate a hint that:
- Provides a starting point or memory cue
- References related concepts if helpful
- Gets progressively more specific with each attempt
- Never reveals the complete answer directly

Respond in JSON format:
{{
    "hint": "<the hint>",
    "difficulty_level": "<easy|medium|hard>"
}}"""


def build_explanation_prompt(
    question: str,
    expected_answer: str,
    user_answer: str,
) -> str:
    """
    Build a prompt for detailed explanation after incorrect answer.

    Args:
        question: The flashcard question/front
        expected_answer: The expected answer/back
        user_answer: The user's incorrect answer

    Returns:
        Formatted prompt for explanation generation
    """
    return f"""The user gave an incorrect answer to a flashcard question. Provide a clear, educational explanation.

Question: {question}

Expected Answer: {expected_answer}

User's Answer: {user_answer}

Provide an explanation that:
1. Acknowledges what they got right (if anything)
2. Clearly explains the correct answer
3. Highlights the key difference between their answer and the correct one
4. Offers a memorable way to remember the correct information

Keep the explanation concise but thorough. Use simple language suitable for spoken delivery.

Respond in JSON format:
{{
    "explanation": "<the explanation>",
    "mnemonic": "<optional memory aid>",
    "related_concepts": ["<concept1>", "<concept2>"]
}}"""


def build_followup_question_prompt(
    original_question: str,
    original_answer: str,
    user_performance: str,
) -> str:
    """
    Build a prompt for generating follow-up questions.

    Args:
        original_question: The original flashcard question
        original_answer: The expected answer
        user_performance: Description of how user performed

    Returns:
        Formatted prompt for follow-up question generation
    """
    return f"""Based on the user's performance on this flashcard, generate a relevant follow-up question to reinforce learning.

Original Question: {original_question}
Original Answer: {original_answer}
User Performance: {user_performance}

Generate a follow-up question that:
- Tests a related but different aspect of the same topic
- Helps reinforce the knowledge just reviewed
- Is appropriate for voice-based Q&A (can be answered verbally)
- Builds on what was just learned

Respond in JSON format:
{{
    "followup_question": "<the question>",
    "expected_answer": "<brief expected answer>",
    "reasoning": "<why this followup helps>"
}}"""


# =============================================================================
# ORAL MODE PROMPTS
# =============================================================================


def build_oral_feedback_prompt(
    question: str,
    expected_answer: str,
    user_answer: str,
) -> str:
    """
    Build a prompt for oral mode structured feedback.

    Oral mode provides direct, efficient feedback with rating announcement.
    No personality or small talk - just clear, concise evaluation.

    Args:
        question: The flashcard question/front
        expected_answer: The expected answer/back
        user_answer: The user's transcribed spoken answer

    Returns:
        Formatted prompt for oral feedback generation
    """
    return f"""Evaluate this flashcard answer and provide structured feedback for spoken delivery.

Question: {question}
Expected Answer: {expected_answer}
Student's Answer: {user_answer}

Be direct and clear. Your feedback will be spoken aloud.

Evaluate using these criteria:
- 0 (Again): Incorrect or no understanding shown
- 1 (Hard): Partially correct with significant gaps
- 2 (Good): Correct with minor variations
- 3 (Easy): Complete and demonstrates clear understanding

Consider:
- Semantic equivalence (synonyms, paraphrasing are acceptable)
- Speech transcription artifacts (um, uh) should be ignored
- Key concepts must be present for higher ratings

Respond in JSON format:
{{
    "rating": <0-3>,
    "is_correct": <true/false>,
    "spoken_feedback": "<feedback for TTS - be concise, state correctness, give the right answer if wrong, announce rating as Again/Hard/Good/Easy>"
}}

Example spoken_feedback formats:
- "Correct. Rating: Good."
- "Correct! The answer is X. Rating: Easy."
- "Incorrect. The answer is X. Rating: Again."
- "Partially correct. You mentioned X but missed Y. Rating: Hard."
"""


# =============================================================================
# CONVERSATIONAL MODE PROMPTS (Feynman Style)
# =============================================================================


def build_conversational_question_prompt(
    question: str,
    card_number: int,
    total_cards: int,
) -> str:
    """
    Build a prompt for presenting a question in conversational Feynman style.

    Args:
        question: The flashcard question to present
        card_number: Current card number in session
        total_cards: Total cards in session

    Returns:
        Formatted prompt for question presentation
    """
    return f"""You are Richard Feynman teaching a student with flashcards. Present this question naturally.

Question to ask: "{question}"

This is card {card_number} of {total_cards}.

Be curious, engaging, maybe add why this is interesting. Keep it brief (1-2 sentences + the question).
Don't be over the top - be genuinely curious and warm, like a friendly teacher.

Examples of good intros:
- "Ah, this is a fun one! [question]"
- "Now here's something interesting... [question]"
- "Okay, let me ask you this: [question]"
- "Here's a good one: [question]"

Output ONLY the text to speak, no JSON. Do not include stage directions or meta-commentary."""


def build_conversational_evaluation_prompt(
    question: str,
    expected_answer: str,
    user_answer: str,
) -> str:
    """
    Build a prompt for conversational Feynman-style evaluation.

    The response should feel like a natural conversation with a brilliant,
    approachable teacher. Rating is determined but not announced.

    Args:
        question: The flashcard question/front
        expected_answer: The expected answer/back
        user_answer: The user's transcribed spoken answer

    Returns:
        Formatted prompt for conversational evaluation
    """
    return f"""You are Richard Feynman responding to a student's answer. Be warm, curious, and teach naturally.

Question: {question}
Expected Answer: {expected_answer}
Student said: "{user_answer}"

Respond as Feynman would:
- Be genuinely curious about their thinking
- If wrong, explain WHY in an intuitive way
- Use simple analogies if helpful
- Be encouraging but honest
- Keep it conversational and natural
- End with a natural transition phrase

Important:
- Do NOT announce the rating - it happens silently
- Focus on teaching and understanding
- Keep response brief (2-4 sentences for correct, 3-5 for incorrect)
- Make it feel like a real conversation

Respond in JSON format:
{{
    "rating": <0-3>,
    "is_correct": <true/false>,
    "spoken_feedback": "<what to say aloud - natural, teaching-focused>",
    "teaching_note": "<optional deeper explanation or insight>"
}}

Rating guide (internal only, don't mention):
- 0: No understanding
- 1: Partial understanding
- 2: Good understanding with minor gaps
- 3: Excellent, complete understanding"""


def build_session_intro_prompt(total_cards: int) -> str:
    """
    Build a prompt for generating a warm session greeting.

    Args:
        total_cards: Number of cards in the session

    Returns:
        Formatted prompt for session intro
    """
    return f"""Generate a warm 1-2 sentence greeting to start a flashcard review session.

Session info: {total_cards} cards to review

Be encouraging and casual, like a friendly tutor. Keep it brief and natural.
Don't be overly enthusiastic or use too many exclamation points.

Examples:
- "Hey there! Let's work through these cards together."
- "Alright, ready to review some cards? Let's do it."
- "Good to see you. We've got {total_cards} cards today - let's get started."

Output ONLY the greeting text, no JSON or meta-commentary."""


def build_session_outro_prompt(
    correct_count: int,
    total_count: int,
    accuracy: float,
) -> str:
    """
    Build a prompt for generating an encouraging session conclusion.

    Args:
        correct_count: Number of correct answers
        total_count: Total cards reviewed
        accuracy: Accuracy percentage (0.0 to 1.0)

    Returns:
        Formatted prompt for session outro
    """
    accuracy_pct = int(accuracy * 100)

    return f"""Generate an encouraging 1-2 sentence conclusion for a completed review session.

Session results:
- {correct_count} out of {total_count} correct
- {accuracy_pct}% accuracy

Be positive regardless of score. Acknowledge their effort.
Keep it brief and genuine - not over the top.

For lower scores: Focus on effort and improvement
For higher scores: Acknowledge the good work without being excessive

Examples:
- "Nice work! You got through all of them. See you next time."
- "Good session. Keep at it and you'll see those concepts stick."
- "Well done today - {accuracy_pct}% is solid. Take a break, you've earned it."

Output ONLY the conclusion text, no JSON or meta-commentary."""
