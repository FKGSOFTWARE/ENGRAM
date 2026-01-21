<script lang="ts">
  import { getVoiceWebSocket } from '$lib/api/websocket';

  interface Props {
    onStartSession?: () => void;
    onEndSession?: () => void;
  }

  let { onStartSession, onEndSession }: Props = $props();

  let connectionState = $state<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected');
  let isRecording = $state(false);
  let isSpeaking = $state(false);
  let audioLevel = $state(0);

  // Server response state
  let transcription = $state<string | null>(null);
  let evaluation = $state<{ is_correct: boolean; rating: number; feedback: string; expected_answer: string; user_answer: string } | null>(null);
  let currentCard = $state<{ id: string; front: string } | null>(null);
  let totalCards = $state(0);
  let currentCardIndex = $state(0);  // 1-based index of current card being reviewed
  let processingAudio = $state(false);
  let errorMessage = $state<string | null>(null);
  let sessionComplete = $state(false);

  let audioContext: AudioContext | null = null;
  let mediaStream: MediaStream | null = null;
  let workletNode: AudioWorkletNode | null = null;
  let workletLoaded = false;

  async function startVoiceSession() {
    try {
      connectionState = 'connecting';
      // Reset state
      transcription = null;
      evaluation = null;
      currentCard = null;
      errorMessage = null;
      sessionComplete = false;

      const ws = getVoiceWebSocket();

      ws.onStateChange((state) => {
        connectionState = state;
        // Auto-start session when connected
        if (state === 'connected') {
          ws.startSession();
        }
      });

      // Handle server messages
      ws.onMessage((message) => {
        switch (message.type) {
          case 'session_started':
            // Session has begun, cards are loaded
            totalCards = message.total_cards ?? 0;
            currentCardIndex = 0;
            break;
          case 'card_presented':
            currentCardIndex++;  // Increment when new card is presented
            currentCard = {
              id: message.card_id ?? '',
              front: message.front ?? ''
            };
            // Clear previous results for new card
            transcription = null;
            evaluation = null;
            errorMessage = null;
            break;
          case 'transcription':
            transcription = message.text ?? null;
            processingAudio = false;
            break;
          case 'evaluation':
            evaluation = {
              is_correct: message.is_correct ?? false,
              rating: message.rating ?? 0,
              feedback: message.feedback ?? '',
              expected_answer: message.expected_answer ?? '',
              user_answer: message.user_answer ?? ''
            };
            break;
          case 'card_rated':
            // Card was rated, next card will be presented
            break;
          case 'session_complete':
            sessionComplete = true;
            currentCard = null;
            break;
          case 'error':
            errorMessage = message.message ?? null;
            processingAudio = false;
            break;
        }
      });

      ws.connect();
      onStartSession?.();
    } catch (e) {
      console.error('Failed to start voice session:', e);
      connectionState = 'error';
    }
  }

  function endVoiceSession() {
    const ws = getVoiceWebSocket();
    ws.disconnect();
    stopRecording();
    processingAudio = false;
    transcription = null;
    evaluation = null;
    currentCard = null;
    errorMessage = null;
    sessionComplete = false;
    totalCards = 0;
    currentCardIndex = 0;
    onEndSession?.();
    connectionState = 'disconnected';
  }

  function nextCard() {
    const ws = getVoiceWebSocket();
    ws.nextCard();
    // Clear state for next card
    transcription = null;
    evaluation = null;
    errorMessage = null;
  }

  function rateCard(rating: 'again' | 'hard' | 'good' | 'easy') {
    const ws = getVoiceWebSocket();
    ws.rateCard(rating);
  }

  async function loadAudioWorklet(ctx: AudioContext): Promise<boolean> {
    if (workletLoaded) return true;

    try {
      await ctx.audioWorklet.addModule('/audio-processor.worklet.js');
      workletLoaded = true;
      return true;
    } catch (e) {
      console.warn('AudioWorklet not supported, falling back to ScriptProcessor:', e);
      return false;
    }
  }

  async function startRecording() {
    try {
      processingAudio = false; // Reset processing state

      // Request microphone access with optimal settings
      mediaStream = await navigator.mediaDevices.getUserMedia({
        audio: {
          sampleRate: 16000, // Match Whisper's expected sample rate
          channelCount: 1,
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true
        }
      });

      // Create AudioContext with target sample rate
      audioContext = new AudioContext({ sampleRate: 16000 });

      // Try to use AudioWorklet (modern API)
      const workletSupported = await loadAudioWorklet(audioContext);

      const source = audioContext.createMediaStreamSource(mediaStream);
      isRecording = true;

      if (workletSupported) {
        // Modern AudioWorklet path
        workletNode = new AudioWorkletNode(audioContext, 'audio-processor', {
          processorOptions: {
            bufferSize: 4096,
            vadThreshold: 0.01,
            vadEnabled: true
          }
        });

        // Handle messages from the worklet
        workletNode.port.onmessage = (event) => {
          if (!isRecording) return;

          const { type, buffer, isSpeech, rms } = event.data;

          if (type === 'audio') {
            // Update UI state
            isSpeaking = isSpeech;
            audioLevel = Math.min(1, rms * 10); // Normalize for display

            // Send audio chunk via WebSocket
            const ws = getVoiceWebSocket();
            ws.sendAudioChunk(buffer);
          } else if (type === 'silence') {
            isSpeaking = false;
            audioLevel = 0;
          }
        };

        source.connect(workletNode);
        workletNode.connect(audioContext.destination);
      } else {
        // AudioWorklet not supported - show error
        throw new Error('Your browser does not support AudioWorklet. Please use a modern browser (Chrome 66+, Firefox 76+, Safari 14.1+, Edge 79+) for voice features.');
      }
    } catch (e) {
      console.error('Failed to start recording:', e);
      isRecording = false;
    }
  }

  function stopRecording() {
    isRecording = false;
    isSpeaking = false;
    audioLevel = 0;
    processingAudio = true; // Show processing state while waiting for transcription

    // Stop the worklet
    if (workletNode) {
      workletNode.port.postMessage({ type: 'stop' });
      workletNode.disconnect();
      workletNode = null;
    }

    // Stop media tracks
    if (mediaStream) {
      mediaStream.getTracks().forEach((track) => track.stop());
      mediaStream = null;
    }

    // Close audio context
    if (audioContext) {
      audioContext.close();
      audioContext = null;
    }

    // Notify server that audio ended
    const ws = getVoiceWebSocket();
    ws.sendEndAudio();
  }

  function toggleRecording() {
    if (isRecording) {
      stopRecording();
    } else {
      startRecording();
    }
  }
</script>

<div class="voice-controls">
  {#if connectionState === 'disconnected'}
    <button class="voice-btn" onclick={startVoiceSession}>
      Start Voice Session
    </button>
  {:else if connectionState === 'connecting'}
    <button class="voice-btn connecting" disabled>
      Connecting...
    </button>
  {:else if connectionState === 'connected'}
    <div class="session-container">
      <!-- Current Card Display -->
      {#if currentCard}
        <div class="card-display">
          <div class="card-progress">Card {currentCardIndex} of {totalCards}</div>
          <div class="card-front">{currentCard.front}</div>
        </div>
      {/if}

      <!-- Session Complete -->
      {#if sessionComplete}
        <div class="session-complete">
          <span class="complete-icon">✓</span>
          <span>Session Complete!</span>
        </div>
      {/if}

      <!-- Error Display -->
      {#if errorMessage}
        <div class="error-message">
          <span class="error-icon">!</span>
          <span>{errorMessage}</span>
        </div>
      {/if}

      <!-- Recording Controls -->
      <div class="session-controls">
        <div class="recording-container">
          <button
            class="record-btn"
            class:recording={isRecording}
            class:speaking={isSpeaking}
            class:processing={processingAudio}
            onclick={toggleRecording}
            disabled={processingAudio || sessionComplete}
          >
            {#if processingAudio}
              <span class="spinner"></span>
            {:else if isRecording}
              Stop
            {:else}
              Record
            {/if}
          </button>
          {#if isRecording}
            <div class="level-meter">
              <div class="level-fill" style="width: {audioLevel * 100}%"></div>
            </div>
            <span class="status-text recording-status">
              {isSpeaking ? 'Listening...' : 'Waiting for speech...'}
            </span>
          {:else if processingAudio}
            <span class="status-text processing-status">Processing...</span>
          {:else if !transcription && !evaluation && currentCard}
            <span class="status-text hint-text">Click to record your answer</span>
          {/if}
        </div>
        <button class="end-btn" onclick={endVoiceSession}>
          End Session
        </button>
      </div>

      <!-- Transcription Display -->
      {#if transcription}
        <div class="transcription">
          <div class="transcription-label">You said:</div>
          <div class="transcription-text">"{transcription}"</div>
        </div>
      {/if}

      <!-- Evaluation Display -->
      {#if evaluation}
        {@const ratingNames = ['again', 'hard', 'good', 'easy'] as const}
        {@const suggestedRating = ratingNames[evaluation.rating] || 'good'}
        <div class="evaluation" class:correct={evaluation.is_correct} class:incorrect={!evaluation.is_correct}>
          <div class="evaluation-header">
            <span class="evaluation-icon">{evaluation.is_correct ? '✓' : '✗'}</span>
            <span class="evaluation-result">{evaluation.is_correct ? 'Correct!' : 'Incorrect'}</span>
          </div>
          {#if evaluation.expected_answer}
            <div class="expected-answer">
              <span class="answer-label">Expected:</span> {evaluation.expected_answer}
            </div>
          {/if}
          {#if evaluation.feedback}
            <div class="evaluation-feedback">{evaluation.feedback}</div>
          {/if}
          <div class="rating-section">
            <div class="rating-label">Rate this card (suggested: {suggestedRating}):</div>
            <div class="rating-buttons">
              <button class="rating-btn again" class:suggested={evaluation.rating === 0} onclick={() => rateCard('again')}>Again</button>
              <button class="rating-btn hard" class:suggested={evaluation.rating === 1} onclick={() => rateCard('hard')}>Hard</button>
              <button class="rating-btn good" class:suggested={evaluation.rating === 2} onclick={() => rateCard('good')}>Good</button>
              <button class="rating-btn easy" class:suggested={evaluation.rating === 3} onclick={() => rateCard('easy')}>Easy</button>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <button class="voice-btn error" onclick={startVoiceSession}>
      Connection Error - Retry
    </button>
  {/if}
</div>

<style>
  .voice-controls {
    display: flex;
    justify-content: center;
    padding: 1rem;
    width: 100%;
  }

  .session-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
    width: 100%;
    max-width: 500px;
  }

  .voice-btn {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    background: var(--accent-light, #e94560);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s, transform 0.1s;
  }

  .voice-btn:hover:not(:disabled) {
    background: var(--accent-dark, #d63d55);
    transform: translateY(-1px);
  }

  .voice-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .voice-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .voice-btn.connecting {
    background: var(--warning, #ffc107);
    color: #000;
  }

  .voice-btn.error {
    background: var(--danger, #dc3545);
  }

  /* Card Display */
  .card-display {
    background: var(--bg-secondary, #16213e);
    border: 1px solid var(--border, #333);
    border-radius: 12px;
    padding: 1.5rem;
    width: 100%;
    text-align: center;
  }

  .card-progress {
    font-size: 0.75rem;
    color: var(--text-secondary, #a0a0a0);
    margin-bottom: 0.75rem;
  }

  .card-front {
    font-size: 1.25rem;
    color: var(--text-primary, #e8e8e8);
    font-weight: 500;
  }

  /* Session Complete */
  .session-complete {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem 1.5rem;
    background: var(--success, #28a745);
    color: white;
    border-radius: 8px;
    font-weight: 600;
  }

  .complete-icon {
    font-size: 1.25rem;
  }

  /* Error Display */
  .error-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: rgba(220, 53, 69, 0.1);
    border: 1px solid var(--danger, #dc3545);
    border-radius: 8px;
    color: var(--danger, #dc3545);
    width: 100%;
  }

  .error-icon {
    width: 20px;
    height: 20px;
    background: var(--danger, #dc3545);
    color: white;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    font-size: 0.875rem;
  }

  .session-controls {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .recording-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .record-btn {
    width: 80px;
    height: 80px;
    font-size: 0.875rem;
    font-weight: 600;
    background: var(--success, #28a745);
    color: white;
    border: none;
    border-radius: 50%;
    cursor: pointer;
    transition: background 0.2s, transform 0.1s, box-shadow 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .record-btn:hover:not(:disabled) {
    transform: scale(1.05);
  }

  .record-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
    transform: none;
  }

  .record-btn.recording {
    background: var(--danger, #dc3545);
    box-shadow: 0 0 0 4px rgba(220, 53, 69, 0.3);
    animation: pulse-shadow 1.5s infinite;
  }

  .record-btn.speaking {
    box-shadow: 0 0 0 6px rgba(40, 167, 69, 0.5);
  }

  .record-btn.recording.speaking {
    box-shadow: 0 0 0 6px rgba(220, 53, 69, 0.5);
  }

  .record-btn.processing {
    background: var(--warning, #ffc107);
    color: #000;
  }

  /* Spinner */
  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid rgba(0, 0, 0, 0.2);
    border-top-color: #000;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .level-meter {
    width: 100px;
    height: 6px;
    background: var(--bg-tertiary, #2a2a3e);
    border-radius: 3px;
    overflow: hidden;
  }

  .level-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--success, #28a745), var(--warning, #ffc107));
    border-radius: 3px;
    transition: width 0.05s ease-out;
  }

  .status-text {
    font-size: 0.75rem;
    color: var(--text-secondary, #a0a0a0);
    text-align: center;
  }

  .processing-status {
    color: var(--warning, #ffc107);
  }

  .hint-text {
    color: var(--text-tertiary, #666);
  }

  .end-btn {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    background: var(--bg-secondary, #16213e);
    color: var(--text-primary, #e8e8e8);
    border: 1px solid var(--border, #333);
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .end-btn:hover {
    background: var(--bg-tertiary, #2a2a3e);
  }

  /* Transcription */
  .transcription {
    background: var(--bg-secondary, #16213e);
    border: 1px solid var(--border, #333);
    border-radius: 8px;
    padding: 1rem;
    width: 100%;
  }

  .transcription-label {
    font-size: 0.75rem;
    color: var(--text-secondary, #a0a0a0);
    margin-bottom: 0.25rem;
  }

  .transcription-text {
    font-size: 1rem;
    color: var(--text-primary, #e8e8e8);
    font-style: italic;
  }

  /* Evaluation */
  .evaluation {
    background: var(--bg-secondary, #16213e);
    border: 2px solid var(--border, #333);
    border-radius: 12px;
    padding: 1rem;
    width: 100%;
  }

  .evaluation.correct {
    border-color: var(--success, #28a745);
  }

  .evaluation.incorrect {
    border-color: var(--danger, #dc3545);
  }

  .evaluation-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .evaluation-icon {
    font-size: 1.5rem;
  }

  .evaluation.correct .evaluation-icon {
    color: var(--success, #28a745);
  }

  .evaluation.incorrect .evaluation-icon {
    color: var(--danger, #dc3545);
  }

  .evaluation-result {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text-primary, #e8e8e8);
  }

  .expected-answer {
    font-size: 0.875rem;
    color: var(--text-secondary, #a0a0a0);
    margin-bottom: 0.5rem;
    padding: 0.5rem;
    background: var(--bg-tertiary, #2a2a3e);
    border-radius: 4px;
  }

  .answer-label {
    font-weight: 600;
    color: var(--text-primary, #e8e8e8);
  }

  .evaluation-feedback {
    font-size: 0.875rem;
    color: var(--text-secondary, #a0a0a0);
    margin-bottom: 1rem;
    line-height: 1.4;
  }

  .rating-section {
    border-top: 1px solid var(--border, #333);
    padding-top: 1rem;
  }

  .rating-label {
    font-size: 0.75rem;
    color: var(--text-secondary, #a0a0a0);
    margin-bottom: 0.5rem;
  }

  .rating-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .rating-btn {
    flex: 1;
    padding: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    border: 1px solid var(--border, #333);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    background: var(--bg-tertiary, #2a2a3e);
    color: var(--text-primary, #e8e8e8);
  }

  .rating-btn:hover {
    transform: translateY(-1px);
  }

  .rating-btn.suggested {
    border-width: 2px;
  }

  .rating-btn.again {
    border-color: #dc3545;
  }

  .rating-btn.again.suggested {
    background: rgba(220, 53, 69, 0.2);
  }

  .rating-btn.hard {
    border-color: #fd7e14;
  }

  .rating-btn.hard.suggested {
    background: rgba(253, 126, 20, 0.2);
  }

  .rating-btn.good {
    border-color: #28a745;
  }

  .rating-btn.good.suggested {
    background: rgba(40, 167, 69, 0.2);
  }

  .rating-btn.easy {
    border-color: #17a2b8;
  }

  .rating-btn.easy.suggested {
    background: rgba(23, 162, 184, 0.2);
  }

  @keyframes pulse-shadow {
    0%, 100% {
      box-shadow: 0 0 0 4px rgba(220, 53, 69, 0.3);
    }
    50% {
      box-shadow: 0 0 0 8px rgba(220, 53, 69, 0.1);
    }
  }
</style>
