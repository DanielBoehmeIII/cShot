# Prompt 25 — Real-Time AI Sound Design

cShot feels immediate and playable.

---

## 1. Real-Time Architecture

### 1.1 Latency Budget

```
Perception of "instant":
  < 5ms:   Feels like hardware (analog synth)
  < 10ms:  Feels like native plugin
  < 20ms:  Feels like a well-optimized DAW
  < 50ms:  Noticeable but acceptable
  < 100ms: Annoying, breaks flow
  > 100ms: Unusable for real-time

cShot RT targets:
  Parameter tweak → audio change:  < 5ms
  New generation (quick mode):     < 20ms
  New generation (standard):       < 50ms
  Latent navigation:               < 16ms (1 frame at 60fps)
  Morph between sounds:            < 10ms transition
```

### 1.2 Pipeline Architecture

```python
class RealTimePipeline:
    """Low-latency generation pipeline."""
    
    def __init__(self):
        # Pre-warm everything
        self.dsp_engine = DSPEngine(sample_rate=44100, buffer_size=64)
        self.neural_frontend = ONNXSession("frontend.onnx", providers=['CUDAExecutionProvider', 'CPUExecutionProvider'])
        self.cache = AudioCache(max_size=256)
        
        # Dual-buffer for seamless parameter changes
        self.current_params = None
        self.next_params = None
        self.audio_buffer = np.zeros(44100 * 2)  # 2-second ring buffer
        self.write_pos = 0
        self.read_pos = 0
        
    def start(self):
        """Start real-time audio thread."""
        self.running = True
        self.audio_thread = threading.Thread(target=self._audio_loop, daemon=True)
        self.audio_thread.start()
    
    def _audio_loop(self):
        """Real-time audio generation loop."""
        while self.running:
            # Generate next block (64 samples)
            if self.next_params is not None:
                # Smooth parameter transition over block
                block = self.dsp_engine.render_block_transition(
                    self.current_params, self.next_params, 64
                )
                self.current_params = self.next_params
                self.next_params = None
            else:
                block = self.dsp_engine.render_block(self.current_params, 64)
            
            # Write to ring buffer
            self.audio_buffer[self.write_pos:self.write_pos + 64] = block
            self.write_pos = (self.write_pos + 64) % len(self.audio_buffer)
            
            # Yield to audio driver (ALSA/ASIO/CoreAudio)
            time.sleep(64 / 44100)  # ~1.45ms
    
    def set_parameter(self, param, value):
        """Set a parameter with sample-accurate transition."""
        if self.next_params is None:
            self.next_params = copy(self.current_params)
        self.next_params[param] = value
    
    def generate_new_sound(self, prompt, quality='quick'):
        """Start generating new sound without interrupting output."""
        # Current sound continues playing
        old_params = self.current_params
        
        # Generate new params in background
        future = ThreadPoolExecutor().submit(self._generate_params, prompt, quality)
        
        # When ready, schedule crossfade
        def crossfade_callback(new_params):
            self._schedule_crossfade(old_params, new_params, duration_ms=20)
        
        future.add_done_callback(crossfade_callback)
```

### 1.3 Caching Strategy

```python
class AudioCache:
    """Multi-level cache for instant playback."""
    
    def __init__(self, max_size=256):
        self.l1_cache = {}   # parameter hash -> audio (in memory, hot)
        self.l2_cache = {}   # parameter hash -> file path (on disk, warm)
        self.max_size = max_size
        self.access_counts = Counter()
        
    def get(self, params):
        key = hash_params(params)
        self.access_counts[key] += 1
        
        if key in self.l1_cache:
            return self.l1_cache[key]
        
        if key in self.l2_cache:
            audio = load_audio(self.l2_cache[key])
            self.l1_cache[key] = audio  # promote to L1
            self._evict_if_needed()
            return audio
        
        return None
    
    def put(self, params, audio):
        key = hash_params(params)
        self.l1_cache[key] = audio
        self._evict_if_needed()
    
    def prewarm(self, params_list):
        """Pre-generate and cache likely next sounds."""
        for params in params_list:
            key = hash_params(params)
            if key not in self.l1_cache and key not in self.l2_cache:
                audio = generate_sound(params, quality='draft')
                self.l1_cache[key] = audio
    
    def _evict_if_needed(self):
        if len(self.l1_cache) > self.max_size:
            # Evict least-accessed
            least_accessed = min(self.access_counts, key=self.access_counts.get)
            del self.l1_cache[least_accessed]
            del self.access_counts[least_accessed]
```

---

## 2. Streaming Inference

### 2.1 Progressive Refinement

```python
class ProgressiveGenerator:
    """Generate low-quality instantly, refine over time."""
    
    def __init__(self):
        self.quality_levels = {
            'draft': {
                'latency_ms': 1,
                'dsp_resolution': 'low',     # fewer oscillators, lower quality
                'neural_model': None,          # no neural processing
                'block_size': 256,
            },
            'quick': {
                'latency_ms': 5,
                'dsp_resolution': 'medium',
                'neural_model': 'tiny.onnx',   # 1M param model
                'block_size': 128,
            },
            'standard': {
                'latency_ms': 20,
                'dsp_resolution': 'high',
                'neural_model': 'standard.onnx', # 5M param
                'block_size': 64,
            },
            'premium': {
                'latency_ms': 200,
                'dsp_resolution': 'ultra',
                'neural_model': 'full.onnx',    # 15M param
                'block_size': 32,
            },
        }
        
    def generate_progressive(self, prompt, callback):
        """Generate sound that starts low-quality and refines."""
        
        def stage_draft():
            audio = self._generate_at_level(prompt, 'draft')
            callback(audio, quality='draft')
            
            # Schedule next stage
            Timer(0.01, stage_quick).start()
        
        def stage_quick():
            audio = self._generate_at_level(prompt, 'quick')
            callback(audio, quality='quick')
            Timer(0.05, stage_standard).start()
        
        def stage_standard():
            audio = self._generate_at_level(prompt, 'standard')
            callback(audio, quality='standard')
            Timer(0.2, stage_premium).start()
        
        def stage_premium():
            audio = self._generate_at_level(prompt, 'premium')
            callback(audio, quality='premium')
        
        # Start the cascade
        stage_draft()
```

### 2.2 Streamable Latent Space

```python
class StreamingLatentNavigator:
    """Navigate latent space with real-time audio output."""
    
    def __init__(self):
        self.current_latent = None
        self.target_latent = None
        self.interpolation_progress = 0.0
        self.interpolation_speed = 0.01  # per sample
        
    def set_target(self, target_latent):
        """Set a new target latent position."""
        self.target_latent = target_latent
        self.interpolation_progress = 0.0
        
    def get_next_block(self, block_size):
        """Get next audio block with smooth latent interpolation."""
        if self.target_latent is None:
            return self.dsp_engine.render_block(self.current_params, block_size)
        
        # Interpolate latent
        alpha = self.interpolation_progress
        current_latent = (1 - alpha) * self.current_latent + alpha * self.target_latent
        
        # Decode latent to DSP params (tiny NN, microsecond scale)
        params = self.latent_to_params(current_latent)
        
        # Render
        block = self.dsp_engine.render_block(params, block_size)
        
        # Advance interpolation
        self.interpolation_progress += self.interpolation_speed * block_size
        if self.interpolation_progress >= 1.0:
            self.current_latent = self.target_latent
            self.target_latent = None
        
        return block
```

---

## 3. Latency Reduction Strategies

| Strategy | Latency Saved | Complexity | Tradeoff |
|----------|--------------|------------|----------|
| **Pre-computation** | 100-500ms | Low | Predict and pre-generate likely sounds |
| **Progressive refinement** | 200ms perceived | Medium | Draft quality first, refine |
| **Model quantization (INT8)** | 2-5x speedup | Medium | ~1% quality loss |
| **TensorRT compilation** | 2-4x speedup | Medium | Hardware-specific |
| **CUDA graphs** | 0.5-2ms per inference | High | Fixed model graph |
| **Speculative generation** | 50-200ms | High | Generate multiple candidates ahead |
| **Flash Attention** | 2-3x for transformer | Medium | New attention kernel |
| **Pruning (50% sparsity)** | 2x speedup | High | Structured sparsity |
| **Knowledge distillation** | 5-10x | Very high | Train smaller model |
| **C++ DSP core** | 10-100x vs Python | Medium | Development complexity |
| **Zero-copy buffers** | 0.1-0.5ms | Low | Avoid memory copies |
| **Lock-free queues** | 0.05-0.2ms | Medium | Thread synchronization |

### 3.1 Optimal Configuration

```
Target hardware: M1 MacBook Pro / RTX 4060 laptop / Phone

DSP engine:           C++ SIMD (NEON/SSE)   —  <1ms per block
Neural frontend:      INT8 ONNX             —  2ms per inference
Neural refinement:    skipped on first pass —  0ms initially, 5ms deferred
Latent→params:        Tiny MLP (50K params) —  <0.1ms
Audio buffering:      64 samples            —  1.45ms latency
Crossfade:            20ms overlap          —  seamless transitions

Total perceived latency: <5ms for parameter tweaks
                         <20ms for new generation (draft)
                         <50ms for full quality (progressive)
```

---

## 4. Adaptive Quality System

```python
class AdaptiveQualitySystem:
    """Automatically adjust quality based on context."""
    
    def __init__(self):
        self.performance_monitor = PerformanceMonitor()
        self.quality_level = 'standard'
        
    def adapt_to_context(self, context):
        """Choose quality level based on current context."""
        
        # User is performing (playing live)
        if context.is_performance:
            return 'quick'
        
        # User is tweaking parameters rapidly
        if context.parameter_change_rate > 5:  # >5 changes per second
            return 'draft'  # show draft during tweaking
            # Will auto-upgrade to standard when tweaking stops
        
        # User is listening critically
        if context.listening_mode or context.eyes_off_screen:
            return 'premium'
        
        # Battery/thermal awareness
        if context.battery_level < 20 or context.cpu_temperature > 85:
            return 'quick'
        
        # DAW integration (must match audio buffer)
        if context.daw_buffer_size:
            if context.daw_buffer_size <= 64:
                return 'quick'
            elif context.daw_buffer_size <= 256:
                return 'standard'
            else:
                return 'premium'
        
        return self.quality_level
    
    def on_tweak_start(self):
        """User started tweaking — downgrade quality for responsiveness."""
        self.quality_level = 'draft'
    
    def on_tweak_end(self):
        """User stopped tweaking — upgrade quality after brief delay."""
        def upgrade():
            time.sleep(0.3)  # wait for tweaking to settle
            self.quality_level = 'standard'
        Thread(target=upgrade).start()
```

---

## 5. Threading & Real-Time Safety

```python
class RealTimeSafeEngine:
    """Lock-free, real-time-safe audio engine."""
    
    def __init__(self):
        self.param_queue = lockfree.Queue()
        self.audio_output = ringbuffer.RingBuffer(size=44100 * 2)
        self.dsp = DSPEngine()
        
        # Real-time thread (SCHED_FIFO priority)
        self.rt_thread = threading.Thread(target=self._rt_loop, daemon=True)
        self.rt_thread.name = "cShot-audio"
        
    def _rt_loop(self):
        """Real-time audio thread — no allocations, no blocking."""
        # Set real-time priority
        os.sched_setscheduler(0, os.SCHED_FIFO, os.sched_param(80))
        
        while True:
            # Process pending parameter changes
            while not self.param_queue.empty():
                param, value = self.param_queue.get()
                self.dsp.set_param(param, value)
            
            # Generate next block
            block = self.dsp.render_block(64)
            self.audio_output.write(block)
    
    def set_param(self, param, value):
        """Thread-safe parameter change (called from UI thread)."""
        self.param_queue.put((param, value))
    
    def get_audio(self, n_samples):
        """Get audio for audio driver callback."""
        return self.audio_output.read(n_samples)
```
