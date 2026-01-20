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

  let audioContext: AudioContext | null = null;
  let mediaStream: MediaStream | null = null;
  let workletNode: AudioWorkletNode | null = null;
  let workletLoaded = false;

  async function startVoiceSession() {
    try {
      connectionState = 'connecting';
      const ws = getVoiceWebSocket();

      ws.onStateChange((state) => {
        connectionState = state;
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
    onEndSession?.();
    connectionState = 'disconnected';
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
    <div class="session-controls">
      <div class="recording-container">
        <button
          class="record-btn"
          class:recording={isRecording}
          class:speaking={isSpeaking}
          onclick={toggleRecording}
        >
          {isRecording ? 'Stop' : 'Record'}
        </button>
        {#if isRecording}
          <div class="level-meter">
            <div class="level-fill" style="width: {audioLevel * 100}%"></div>
          </div>
          <span class="status-text">
            {isSpeaking ? 'Listening...' : 'Waiting for speech...'}
          </span>
        {/if}
      </div>
      <button class="end-btn" onclick={endVoiceSession}>
        End Session
      </button>
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
  }

  .record-btn:hover {
    transform: scale(1.05);
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

  @keyframes pulse-shadow {
    0%, 100% {
      box-shadow: 0 0 0 4px rgba(220, 53, 69, 0.3);
    }
    50% {
      box-shadow: 0 0 0 8px rgba(220, 53, 69, 0.1);
    }
  }
</style>
