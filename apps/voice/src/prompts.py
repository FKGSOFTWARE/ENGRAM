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
