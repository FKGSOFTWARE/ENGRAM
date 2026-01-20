<script lang="ts">
  import { getVoiceWebSocket } from '$lib/api/websocket';

  interface Props {
    onStartSession?: () => void;
    onEndSession?: () => void;
  }

  let { onStartSession, onEndSession }: Props = $props();

  let connectionState = $state<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected');
  let isRecording = $state(false);
  let audioContext: AudioContext | null = null;
  let mediaStream: MediaStream | null = null;
  let workletNode: AudioWorkletNode | null = null;

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

  async function startRecording() {
    try {
      mediaStream = await navigator.mediaDevices.getUserMedia({
        audio: {
          sampleRate: 24000,
          channelCount: 1,
          echoCancellation: true,
          noiseSuppression: true
        }
      });

      audioContext = new AudioContext({ sampleRate: 24000 });

      // Load audio worklet for processing
      // Note: This is a placeholder - actual worklet implementation needed
      const source = audioContext.createMediaStreamSource(mediaStream);

      isRecording = true;

      // For now, use ScriptProcessor as fallback
      const processor = audioContext.createScriptProcessor(4096, 1, 1);
      processor.onaudioprocess = (e) => {
        if (!isRecording) return;
        const inputData = e.inputBuffer.getChannelData(0);
        const pcmData = new Int16Array(inputData.length);
        for (let i = 0; i < inputData.length; i++) {
          pcmData[i] = Math.max(-32768, Math.min(32767, inputData[i] * 32768));
        }
        const ws = getVoiceWebSocket();
        ws.sendAudioChunk(pcmData.buffer);
      };

      source.connect(processor);
      processor.connect(audioContext.destination);
    } catch (e) {
      console.error('Failed to start recording:', e);
    }
  }

  function stopRecording() {
    isRecording = false;
    if (mediaStream) {
      mediaStream.getTracks().forEach((track) => track.stop());
      mediaStream = null;
    }
    if (audioContext) {
      audioContext.close();
      audioContext = null;
    }
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
      🎤 Start Voice Session
    </button>
  {:else if connectionState === 'connecting'}
    <button class="voice-btn connecting" disabled>
      Connecting...
    </button>
  {:else if connectionState === 'connected'}
    <div class="session-controls">
      <button
        class="record-btn"
        class:recording={isRecording}
        onclick={toggleRecording}
      >
        {isRecording ? '⏹️ Stop' : '🎙️ Record'}
      </button>
      <button class="end-btn" onclick={endVoiceSession}>
        End Session
      </button>
    </div>
  {:else}
    <button class="voice-btn error" onclick={startVoiceSession}>
      ⚠️ Connection Error - Retry
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
    background: var(--primary, #007bff);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .voice-btn:hover:not(:disabled) {
    background: var(--primary-dark, #0056b3);
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
    gap: 1rem;
  }

  .record-btn {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    background: var(--success, #28a745);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .record-btn.recording {
    background: var(--danger, #dc3545);
    animation: pulse 1s infinite;
  }

  .end-btn {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    background: var(--secondary, #6c757d);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.7;
    }
  }
</style>
