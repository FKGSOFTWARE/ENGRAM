/**
 * AudioWorklet Processor for ENGRAM Voice Recording
 *
 * This processor captures audio from the microphone, converts it to PCM Int16,
 * and sends it to the main thread for transmission via WebSocket.
 *
 * Features:
 * - Real-time audio processing in a separate thread
 * - Float32 to Int16 PCM conversion
 * - Voice activity detection (basic energy-based)
 * - Configurable buffer size
 */

class AudioProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();

    // Configuration from options
    this.bufferSize = options?.processorOptions?.bufferSize || 4096;
    this.vadThreshold = options?.processorOptions?.vadThreshold || 0.01;
    this.vadEnabled = options?.processorOptions?.vadEnabled ?? true;

    // Store sampleRate from AudioWorkletGlobalScope (available as global in worklet context)
    // Fallback to common sample rates if not available
    this.sampleRate = typeof sampleRate !== 'undefined' ? sampleRate : 48000;

    // Internal buffer for accumulating samples
    this.buffer = new Float32Array(this.bufferSize);
    this.bufferIndex = 0;

    // State
    this.isRecording = true;
    this.silentFrames = 0;
    this.maxSilentFrames = 30; // ~640ms at 128 samples/frame, 48kHz

    // Handle messages from main thread
    this.port.onmessage = (event) => {
      if (event.data.type === 'stop') {
        this.isRecording = false;
      } else if (event.data.type === 'start') {
        this.isRecording = true;
      } else if (event.data.type === 'config') {
        if (event.data.vadThreshold !== undefined) {
          this.vadThreshold = event.data.vadThreshold;
        }
        if (event.data.vadEnabled !== undefined) {
          this.vadEnabled = event.data.vadEnabled;
        }
      }
    };
  }

  /**
   * Calculate the RMS (Root Mean Square) energy of audio samples.
   * Used for basic voice activity detection.
   */
  calculateRMS(samples) {
    let sum = 0;
    for (let i = 0; i < samples.length; i++) {
      sum += samples[i] * samples[i];
    }
    return Math.sqrt(sum / samples.length);
  }

  /**
   * Convert Float32 samples (-1 to 1) to Int16 PCM (-32768 to 32767)
   */
  floatToInt16(float32Array) {
    const int16Array = new Int16Array(float32Array.length);
    for (let i = 0; i < float32Array.length; i++) {
      // Clamp and convert
      const sample = Math.max(-1, Math.min(1, float32Array[i]));
      int16Array[i] = sample < 0 ? sample * 0x8000 : sample * 0x7fff;
    }
    return int16Array;
  }

  /**
   * Process audio input.
   * Called by the AudioWorklet infrastructure with 128 samples per channel.
   */
  process(inputs, outputs, parameters) {
    // Check if we should continue processing
    if (!this.isRecording) {
      return true; // Keep processor alive but don't process
    }

    // Get input channel (mono)
    const input = inputs[0];
    if (!input || !input[0]) {
      return true;
    }

    const inputChannel = input[0];

    // Accumulate samples into buffer
    for (let i = 0; i < inputChannel.length; i++) {
      this.buffer[this.bufferIndex++] = inputChannel[i];

      // When buffer is full, process and send
      if (this.bufferIndex >= this.bufferSize) {
        this.processBuffer();
        this.bufferIndex = 0;
      }
    }

    return true; // Keep processor running
  }

  /**
   * Process the accumulated buffer and send to main thread
   */
  processBuffer() {
    // Calculate energy for VAD
    const rms = this.calculateRMS(this.buffer);
    const isSpeech = rms > this.vadThreshold;

    // Track silence for end-of-speech detection
    if (isSpeech) {
      this.silentFrames = 0;
    } else {
      this.silentFrames++;
    }

    // Skip sending if VAD is enabled and we've had too many silent frames
    if (this.vadEnabled && this.silentFrames > this.maxSilentFrames) {
      // Send silence indicator
      this.port.postMessage({
        type: 'silence',
        duration: this.silentFrames * (this.bufferSize / this.sampleRate),
      });
      return;
    }

    // Convert to Int16 PCM
    const pcmData = this.floatToInt16(this.buffer);

    // Send to main thread
    this.port.postMessage(
      {
        type: 'audio',
        buffer: pcmData.buffer,
        isSpeech,
        rms,
      },
      [pcmData.buffer] // Transfer ownership for efficiency
    );
  }
}

// Register the processor
registerProcessor('audio-processor', AudioProcessor);
