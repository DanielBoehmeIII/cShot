# Prompt 46 — Design the Preview Experience

The fastest possible preview system for generated one-shots — click, hear, decide, move on.

---

## 1. User Actions

| Action | Latency Target | Implementation |
|--------|---------------|----------------|
| Click sound → hear it | <10ms from click to audio | Pre-cached AudioBuffer, Web Audio API |
| Keyboard trigger | <5ms key → audio | Key handler + pre-loaded buffer |
| Switch between variants | <10ms crossfade | 10ms gain ramp between sources |
| Loop audition | <1ms loop point → seamless | Web Audio API loop flag |
| Preview loudness normalization | Instant | All sounds pre-normalized to -1dBFS on backend |
| Waveform thumbnail | Render in <16ms | 80-point pre-computed SVG |
| Favorite/delete | <50ms UI update | Optimistic update, no async wait |
| Drag/export | <100ms file write | Pre-exported WAV or on-demand with progress |

---

## 2. Frontend Architecture

### Component Tree

```
PreviewSystem
├── AudioEngine            # Singleton, manages AudioContext + buffer cache
│   ├── AudioContext       # Single shared context
│   ├── BufferCache        # LRU cache of decoded AudioBuffers
│   ├── ActiveSource       # Currently playing source node
│   └── MasterGain         # Output gain node
│
├── WaveformDisplay        # SVG-based waveform
│   ├── Thumbnail          # 80-point miniature (sound slot)
│   └── FullView           # Detailed waveform (selected sound)
│
├── PlaybackController     # Play/stop/loop state
│   ├── TransportBar       # Play, stop, loop toggle, position
│   └── KeyboardHandler    # Space, numbers, arrows
│
└── SoundSlot              # Individual sound card
    ├── WaveformThumbnail  # 80-point mini waveform
    ├── PlayIndicator      # Playing/paused animation
    ├── FavoriteButton     # Heart toggle
    └── ExportButton       # Download trigger
```

### AudioEngine Singleton

```typescript
// src/lib/AudioEngine.ts

class AudioEngine {
  private ctx: AudioContext | null = null;
  private bufferCache: Map<string, AudioBuffer> = new Map();
  private activeSource: AudioBufferSourceNode | null = null;
  private masterGain: GainNode | null = null;
  private currentId: string | null = null;
  private isLooping: boolean = false;

  // Max cache: 50MB of audio buffers (~30 seconds of 44.1kHz mono)
  private static MAX_CACHE_BYTES = 50 * 1024 * 1024;
  private currentCacheBytes: number = 0;

  init(): void {
    // Lazy init on first user interaction
    this.ctx = new AudioContext({ sampleRate: 44100 });
    this.masterGain = this.ctx.createGain();
    this.masterGain.gain.value = 0.8; // Master volume
    this.masterGain.connect(this.ctx.destination);
  }

  async loadBuffer(soundId: string, audioData: Float32Array): Promise<AudioBuffer> {
    if (this.bufferCache.has(soundId)) {
      return this.bufferCache.get(soundId)!;
    }
    const buffer = this.ctx!.createBuffer(1, audioData.length, 44100);
    buffer.getChannelData(0).set(audioData);

    // Evict if over cache limit
    const bytes = audioData.length * 4;
    while (this.currentCacheBytes + bytes > AudioEngine.MAX_CACHE_BYTES) {
      const oldest = this.bufferCache.keys().next().value!;
      this.currentCacheBytes -= (this.bufferCache.get(oldest)!.getChannelData(0).length * 4);
      this.bufferCache.delete(oldest);
    }
    this.bufferCache.set(soundId, buffer);
    this.currentCacheBytes += bytes;
    return buffer;
  }

  async play(soundId: string, startTime: number = 0): Promise<void> {
    this.stop();
    if (!this.ctx) this.init();
    const buffer = this.bufferCache.get(soundId);
    if (!buffer) return;

    const source = this.ctx!.createBufferSource();
    source.buffer = buffer;
    source.loop = this.isLooping;
    source.connect(this.masterGain!);
    source.start(0, startTime);
    this.activeSource = source;
    this.currentId = soundId;
  }

  stop(): void {
    if (this.activeSource) {
      try { this.activeSource.stop(); } catch {}
      this.activeSource.disconnect();
      this.activeSource = null;
    }
    this.currentId = null;
  }

  toggleLoop(): void {
    this.isLooping = !this.isLooping;
    if (this.activeSource) {
      this.activeSource.loop = this.isLooping;
    }
  }

  getPlaybackPosition(): number {
    if (!this.ctx || !this.activeSource) return 0;
    return this.ctx.currentTime - (this.activeSource as any).startTime;
  }

  setVolume(db: number): void {
    if (this.masterGain) {
      this.masterGain.gain.value = Math.pow(10, db / 20);
    }
  }

  // Crossfade between two sounds
  async crossfade(fromId: string | null, toId: string, durationMs: number = 10): Promise<void> {
    if (!fromId || fromId === toId) {
      return this.play(toId);
    }
    const crossfadeSec = durationMs / 1000;
    const currentTime = this.ctx!.currentTime;

    // Fade out current
    if (this.masterGain) {
      this.masterGain.gain.setValueAtTime(this.masterGain.gain.value, currentTime);
      this.masterGain.gain.linearRampToValueAtTime(0, currentTime + crossfadeSec);
    }
    setTimeout(() => {
      this.stop();
      this.play(toId);
      // Fade in new
      if (this.masterGain) {
        this.masterGain.gain.setValueAtTime(0, this.ctx!.currentTime);
        this.masterGain.gain.linearRampToValueAtTime(0.8, this.ctx!.currentTime + crossfadeSec);
      }
    }, durationMs);
  }

  clearCache(): void {
    this.bufferCache.clear();
    this.currentCacheBytes = 0;
  }
}

export const audioEngine = new AudioEngine(); // Singleton
```

---

## 3. Audio Caching Strategy

### Layers

```
Layer 1: Memory (AudioBuffer)
  - Recently played sounds (last 20)
  - ~35MB max (20 × ~1.7MB for 10s mono)
  - Instant playback (<1ms)
  - Eviction: LRU

Layer 2: Local File (WAV on disk)
  - All generated sounds
  - ~/cShot/audio/{uuid}.wav
  - Load via invoke('get_audio_data') → Float32Array
  - ~5-10ms read + decode

Layer 3: Cache Miss
  - Regenerate from model (slow path)
  - Display "loading" state
  - Only if file was deleted from disk
```

### Cache Pre-warming

```typescript
// On app start, pre-warm cache with:
// 1. Last played sound
// 2. Favorited sounds
// 3. Most recent generation batch

async function prewarmCache(): Promise<void> {
  const recentIds = await invoke('get_recent_sounds', { limit: 5 });
  const favoriteIds = await invoke('get_favorite_ids');
  const toLoad = [...new Set([...recentIds, ...favoriteIds])];
  
  // Load in parallel
  await Promise.all(toLoad.map(async (id) => {
    const audioData = await invoke('get_audio_data', { sound_id: id });
    await audioEngine.loadBuffer(id, new Float32Array(audioData));
  }));
}
```

---

## 4. Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Space | Play/stop selected sound |
| Enter | Generate from prompt |
| ← → | Select previous/next sound in grid |
| ↑ ↓ | Navigate sound categories (if grouped) |
| 1-6 | Select sound slot 1-6 |
| Cmd+E | Export selected sound |
| F / Cmd+F | Favorite toggle |
| L | Toggle loop |
| Cmd+Z | Undo last generation (restore previous grid) |
| Esc | Clear selection / stop playback |
| ? | Show shortcuts overlay |

### Implementation

```typescript
// src/hooks/useKeyboard.ts

import { useEffect } from 'react';
import { audioEngine } from '../lib/AudioEngine';

type KeyAction = 
  | { type: 'play_stop' }
  | { type: 'select_slot'; index: number }
  | { type: 'navigate'; direction: 'prev' | 'next' }
  | { type: 'export' }
  | { type: 'favorite' }
  | { type: 'toggle_loop' }
  | { type: 'generate' };

interface KeyboardConfig {
  onAction: (action: KeyAction) => void;
  isEnabled: boolean;
}

export function useKeyboard({ onAction, isEnabled }: KeyboardConfig): void {
  useEffect(() => {
    if (!isEnabled) return;
    
    const handler = (e: KeyboardEvent) => {
      const isCmd = e.metaKey || e.ctrlKey;
      
      switch (e.code) {
        case 'Space':
          e.preventDefault();
          onAction({ type: 'play_stop' });
          break;
        case 'Enter':
          onAction({ type: 'generate' });
          break;
        case 'ArrowLeft':
          onAction({ type: 'navigate', direction: 'prev' });
          break;
        case 'ArrowRight':
          onAction({ type: 'navigate', direction: 'next' });
          break;
        case 'KeyE':
          if (isCmd) onAction({ type: 'export' });
          break;
        case 'KeyF':
          onAction({ type: 'favorite' });
          break;
        case 'KeyL':
          onAction({ type: 'toggle_loop' });
          break;
        default:
          // Digit 1-6
          if (e.code.startsWith('Digit') && !isCmd) {
            const digit = parseInt(e.code.replace('Digit', ''));
            if (digit >= 1 && digit <= 6) {
              onAction({ type: 'select_slot', index: digit - 1 });
            }
          }
      }
    };
    
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [onAction, isEnabled]);
}
```

---

## 5. Waveform UI

### Thumbnail Component (80 points)

```typescript
// src/components/grid/WaveformThumbnail.tsx

interface WaveformThumbnailProps {
  data: number[];           // 80 normalized float values [-1, 1]
  width?: number;           // default: 120
  height?: number;          // default: 32
  color?: string;           // default: accent purple
  isPlaying?: boolean;
  progress?: number;        // 0-1 playback position
}

export function WaveformThumbnail({ data, width = 120, height = 32, color, isPlaying, progress }: WaveformThumbnailProps) {
  // SVG viewBox: 0 0 120 32
  // Draw mirrored waveform: top half = positive, bottom half = negative
  // Playing state: animated scan line + color shift
  // Progress: filled portion changes opacity
}
```

### SVG Rendering Strategy

```
For 80 data points in a 120×32 SVG viewBox:
  - Each point spans 1.5px horizontally
  - Vertical: map [-1, 1] to [0, height] with center at height/2
  - Mirror: draw positive up, negative down from center
  - Playing: animated gradient sweep left-to-right
  - Hover: slight scale up + glow effect
```

### Full Waveform View (Selected Sound)

```typescript
// src/components/detail/WaveformViewer.tsx

interface WaveformViewerProps {
  data: number[];           // Full resolution waveform (1000+ points)
  sampleRate: number;
  duration: number;
  isPlaying: boolean;
  onSeek: (position: number) => void;
}

// Interactive: click to seek, drag to select region
// Zoom: scroll to zoom in/out
// Shows: waveform + envelope overlay + play head
```

---

## 6. Latency Targets

| Action | Target | Acceptable | Unacceptable |
|--------|--------|------------|-------------|
| Click → audio start | <10ms | <50ms | >100ms |
| Keyboard → audio start | <5ms | <30ms | >80ms |
| Sound switch (crossfade) | <20ms | <50ms | >100ms |
| Waveform render | <16ms (1 frame) | <33ms (2 frames) | >50ms |
| Favorite toggle UI | <16ms | <50ms | >100ms |
| Export dialog show | <100ms | <300ms | >500ms |
| Export file write | <50ms | <200ms | >500ms |

### Achieving Sub-10ms Playback

```
1. Pre-decode: AudioBuffer created on generation complete, held in memory
2. Single AudioContext: Created once, reused (no cold-start latency)
3. Buffer source: Web Audio API source nodes start with zero overhead
4. No format conversion: Float32Array directly to AudioBuffer
5. Pre-warm: Recent + favorited sounds loaded on app start
6. Zero-copy IPC: Audio data sent as raw Float32Array (no base64 in Tauri IPC)
```

---

## 7. Failure States

| Failure | Symptom | Handling |
|---------|---------|----------|
| AudioContext blocked | No sound, browser autoplay policy | Show "Click to enable audio" overlay. Resumes on first click |
| Buffer not loaded | Click does nothing | Show loading spinner on slot. Load buffer. Retry on fail |
| Decode error | No sound | Show error toast "Failed to decode audio". Regenerate |
| File not found on disk | Sound missing | Show "Sound file missing" indicator. Offer regeneration |
| Rapid clicking | Multiple sounds at once | Debounce play calls (100ms). Always stop previous before new |
| AudioContext lost | All sound stops | Auto-recover: recreate AudioContext, reload cache |
| Memory pressure | Browser kills AudioContext | Listen for context state change. Recreate. Reduce cache size |

### Recovery Logic

```typescript
audioEngine.ctx.onstatechange = () => {
  if (audioEngine.ctx!.state === 'closed') {
    // AudioContext was closed (e.g., memory pressure)
    audioEngine.ctx = null;
    audioEngine.bufferCache.clear();
    showToast('Audio engine reset. Click to re-enable.');
  }
};
```

---

## 8. Implementation Order

```
Phase 1 — Prototype (build first):
  1. AudioEngine singleton with play/stop
  2. Float32Array → AudioBuffer loading
  3. Basic WaveformThumbnail SVG
  4. Space bar play/stop
  5. Single sound in/out of cache

Phase 2 — MVP (add second):
  6. Crossfade between sounds
  7. Loop toggle
  8. Full keyboard shortcut support
  9. BufferCache with LRU eviction
  10. Waveform playhead animation
  11. Cache pre-warming

Phase 3 — Post-MVP (add last):
  12. Full waveform viewer with seek/zoom
  13. Drag handle for loop region
  14. A/B comparison (swap between two sounds instantly)
  15. Volume envelope overlay
  16. Export region selection
```

---

## 9. Summary

The preview system is the most latency-sensitive part of cShot. Every millisecond between click and audio erodes the feeling of magic. The entire architecture — from pre-decoded AudioBuffers to keyboard shortcuts to the SVG waveform — is designed to make playback feel instantaneous.

Target: **Click → hear in <10ms.** This is achievable with Web Audio API and pre-loaded buffers in a Tauri WebView.
