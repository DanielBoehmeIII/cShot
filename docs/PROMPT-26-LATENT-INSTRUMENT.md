# Prompt 26 — Latent Space Instrument Design

cShot becomes a playable instrument — a synthesizer made of semantic audio intelligence.

---

## 1. Interface Paradigms

### 1.1 The Latent Instrument Concept

```
Traditional synth:  Oscillators → Filter → Envelope → VCA → Effects
cShot instrument:   Semantic intent → Latent navigation → Physical rendering → Perceptual refinement


Instead of knobs controlling voltage,
knobs control meaning.

Instead of patch cables routing signal,
gestures navigate sonic possibility space.
```

### 1.2 Instrument Metaphors

| Metaphor | Description | Best For |
|----------|-------------|----------|
| **X/Y Pad** | 2D latent slice, navigate with finger | Live exploration, morphing |
| **Sphere** | 3D latent globe, surround sound | Immersive spatial performance |
| **Z-Axis slider** | Third dimension (e.g., intensity/depth) | Expressive layering |
| **Radial menu** | Genre/perceptual type selection | Quick preset navigation |
| **Gesture path** | Draw trajectory through latent space | Choreographed sound journeys |
| **Voice** | Semantic voice commands → latent position | Hands-free, eyes-free |
| **Touch** | Pressure/speed/position → multi-param control | Tactile performance |
| **Sequencer** | Time-mapped latent waypoints | Generative arrangements |
| **Graph** | Node graph of latent space | Visual programming |

---

## 2. Spatial Audio Maps

### 2.1 Latent Space as a Navigable World

```python
class LatentSoundscape:
    """A navigable 2D/3D space of sounds."""
    
    def __init__(self, library_sounds=None):
        self.sounds = library_sounds or []
        self.positions = {}  # sound_id -> (x, y) in 2D space
        self.terrain = None  # perceptual height map
        
    def build_map(self, projection='umap'):
        """Project sounds into 2D navigable space."""
        embeddings = [encode_dna(s) for s in self.sounds]
        
        if projection == 'umap':
            coords = UMAP(n_components=2).fit_transform(embeddings)
        elif projection == 'tsne':
            coords = TSNE(n_components=2).fit_transform(embeddings)
        elif projection == 'pca':
            coords = PCA(n_components=2).fit_transform(embeddings)
        
        for i, sound_id in enumerate(self.sounds):
            self.positions[sound_id] = coords[i]
        
        # Build perceptual terrain
        self.terrain = self._build_terrain()
        
    def _build_terrain(self):
        """Build a height map from perceptual features."""
        # "Height" = perceptual intensity (complexity, energy, brightness)
        terrain = np.zeros((100, 100))
        for sound_id, (x, y) in self.positions.items():
            sound = self.get_sound(sound_id)
            features = extract_perceptual_features(sound)
            height = features.get('complexity', 0.5)
            
            # Gaussian splat
            ix, iy = int((x + 1) / 2 * 99), int((y + 1) / 2 * 99)
            for dx in range(-5, 6):
                for dy in range(-5, 6):
                    if 0 <= ix+dx < 100 and 0 <= iy+dy < 100:
                        dist = np.sqrt(dx**2 + dy**2)
                        terrain[ix+dx, iy+dy] += height * np.exp(-dist**2 / 4)
        
        return terrain / np.max(terrain)
    
    def get_at_position(self, x, y):
        """Get the sound at a navigable position (interpolates)."""
        # Find nearest sounds
        distances = []
        for sound_id, (sx, sy) in self.positions.items():
            dist = np.sqrt((x - sx)**2 + (y - sy)**2)
            distances.append((dist, sound_id))
        
        distances.sort(key=lambda d: d[0])
        
        # Interpolate top 3 nearest
        nearest = distances[:3]
        total_weight = sum(1 / (d[0] + 0.001) for d, _ in nearest)
        
        params = {}
        for dist, sound_id in nearest:
            weight = (1 / (dist + 0.001)) / total_weight
            sound = self.get_sound(sound_id)
            dna = sound.dna
            for gene_name, gene_value in dna.items():
                if gene_name not in params:
                    params[gene_name] = 0
                params[gene_name] += weight * gene_value
        
        return self.synthesize_from_params(params)
```

### 2.2 Visual Feedback

```
The latent space instrument shows:
  - Sound clusters as "constellations" (brightness = density)
  - "Hot zones" where user has spent time
  - "Gravitational wells" around saved/favorited sounds
  - Trails showing navigation history
  - "Wind" showing trending directions in the space

Visual style: cosmic/astronomical — sounds as stars, 
  latent paths as shooting stars, clusters as nebulae
```

---

## 3. Performance Systems

### 3.1 Gesture-to-Sound Mapping

```python
class GestureMapper:
    """Map performance gestures to latent parameters."""
    
    def __init__(self):
        self.gesture_maps = {
            'horizontal_sweep': {
                'direction': 'x_axis',
                'maps_to': ['spectral_centroid', 'brightness'],
                'range': (0, 1),
                'curve': 'linear',
            },
            'vertical_sweep': {
                'direction': 'y_axis',
                'maps_to': ['punch', 'temporal_centroid'],
                'range': (0, 1),
                'curve': 'exponential',
            },
            'circle': {
                'radius': {
                    'maps_to': ['stereo_width', 'reverb_amount'],
                    'range': (0, 1),
                },
                'speed': {
                    'maps_to': ['modulation_rate', 'vibrato_depth'],
                    'range': (0, 0.5),
                },
            },
            'pinch': {
                'distance': {
                    'maps_to': ['spectral_flatness', 'harmonicity'],
                    'range': (0.2, 0.8),
                },
            },
            'tap': {
                'velocity': {
                    'maps_to': ['excitation_velocity', 'transient_sharpness'],
                    'range': (0.2, 1.0),
                },
                'position': {
                    'maps_to': ['formant_position', 'pan'],
                    'range': (-1, 1),
                },
            },
            'hold': {
                'duration': {
                    'maps_to': ['decay_time', 'release_time'],
                    'range': (0.1, 5.0),
                },
                'pressure': {
                    'maps_to': ['saturation_amount', 'compression_ratio'],
                    'range': (0, 1),
                },
            },
        }
    
    def apply_gesture(self, gesture_type, gesture_data, current_params):
        """Apply gesture to modify current parameters."""
        mapping = self.gesture_maps.get(gesture_type)
        if not mapping:
            return current_params
        
        new_params = current_params.copy()
        for target_param, map_info in mapping.items():
            if target_param == 'direction':
                continue
            if target_param in gesture_data:
                value = gesture_data[target_param]
                raw = map_info['range'][0] + value * (map_info['range'][1] - map_info['range'][0])
                for param in map_info['maps_to']:
                    if param in new_params:
                        new_params[param] = np.clip(raw, map_info['range'][0], map_info['range'][1])
        
        return new_params
```

### 3.2 Expressive Controllers

| Controller | Gesture | Affects |
|-----------|---------|---------|
| MPE touch surface | Slide up/down | Brightness → darkness |
| MPE pressure | Press harder | Punch, weight, intensity |
| XY pad | Orbit | Travel latent space |
| Accelerometer | Tilt device | Stereo pan, reverb tilt |
| Breath controller | Blow harder | Parameter intensity |
| Foot pedal | Press | Filter cutoff, volume swell |
| Rotary encoder | Spin | Browse latent dimensions |
| Multi-touch | 2-finger rotate | Stereo width, modulation |
| Camera/IR | Hand position | Latent space navigation |

### 3.3 Performance Modes

```python
class PerformanceMode:
    """Different playing modes for the latent instrument."""
    
    @staticmethod
    def explore_mode(navigator, gesture_input):
        """Free exploration: user wanders latent space."""
        while True:
            x, y = gesture_input.get_xy()
            sound = navigator.get_at_position(x, y)
            navigator.output(sound)
            yield sound
    
    @staticmethod
    def trajectory_mode(navigator, gesture_input):
        """User draws trajectory, sound follows path."""
        path = []
        while True:
            if gesture_input.is_touching():
                x, y = gesture_input.get_xy()
                path.append((x, y))
                
                # Generate sound at current position
                sound = navigator.get_at_position(x, y)
                navigator.output(sound)
            elif path:
                # Loop trajectory
                for x, y in path:
                    sound = navigator.get_at_position(x, y)
                    navigator.output(sound)
                    time.sleep(0.05)
            
            yield
    
    @staticmethod
    def morph_mode(navigator, gesture_input):
        """Two-finger: morph between two positions."""
        while True:
            if gesture_input.n_touches() == 2:
                pos_a = gesture_input.get_touch(0)
                pos_b = gesture_input.get_touch(1)
                
                # Interpolate between positions
                sound_a = navigator.get_at_position(*pos_a)
                sound_b = navigator.get_at_position(*pos_b)
                
                blend = gesture_input.get_pinch_amount()
                morphed = interpolate_sounds(sound_a, sound_b, alpha=blend)
                navigator.output(morphed)
            
            yield
    
    @staticmethod
    def rhythm_mode(navigator, gesture_input):
        """Tap to trigger, gesture to shape each hit."""
        while True:
            if gesture_input.was_tapped():
                x, y = gesture_input.get_tap_position()
                velocity = gesture_input.get_tap_velocity()
                sound = navigator.get_at_position(x, y)
                sound = apply_velocity(sound, velocity)
                navigator.trigger(sound)  # one-shot trigger
            
            yield
```

---

## 4. Latent Instrument Architectures

### 4.1 Core Architecture

```python
class LatentInstrument:
    """The cShot playable latent space instrument."""
    
    def __init__(self):
        self.soundscape = LatentSoundscape()
        self.gesture = GestureMapper()
        self.current_position = np.array([0.0, 0.0])
        self.current_params = {}
        self.history = []
        self.record_mode = False
        
    def play(self, gesture):
        """Single play iteration — called at audio rate."""
        # Map gesture to latent movement
        delta = self.gesture.gesture_to_movement(gesture)
        self.current_position += delta * 0.01
        self.current_position = np.clip(self.current_position, -1, 1)
        
        # Get sound at position
        sound = self.soundscape.get_at_position(*self.current_position)
        
        # Apply expressive gesture modifiers
        params = extract_dna(sound)
        params = self.gesture.apply_gesture(
            gesture.type, gesture.data, params
        )
        
        # Record trajectory if recording
        if self.record_mode:
            self.history.append((self.current_position.copy(), params.copy()))
        
        # Render
        audio = synthesize_from_params(params)
        
        return audio
    
    def morph_to(self, target_position, duration_ms=500):
        """Smoothly move to target position over duration."""
        start = self.current_position.copy()
        n_steps = int(duration_ms / (64 / 44100 * 1000))  # ~1.45ms per block
        
        for step in range(n_steps):
            alpha = step / n_steps
            alpha = smoothstep(alpha)  # ease in/out
            pos = (1 - alpha) * start + alpha * target_position
            self.current_position = pos
            
            yield self.play(NullGesture())  # generate audio block
    
    def record_performance(self, duration_seconds=30):
        """Record a performance for later reconstruction."""
        self.record_mode = True
        self.history = []
    
    def playback_performance(self):
        """Play back a recorded performance."""
        for pos, params in self.history:
            self.current_position = pos
            audio = synthesize_from_params(params)
            yield audio
```

### 4.2 Instrument Presets

```python
instrument_presets = {
    'orbital': {
        'description': 'Latent space with gravity wells around favorite sounds',
        'physics': {
            'gravity': 0.5,      # pull toward favorite sounds
            'friction': 0.1,     # slow down movement
            'momentum': 0.3,     # keep moving in same direction
            'repulsion': 0.2,    # push away from explored areas
        },
        'visual': 'nebula_with_stars',
        'mapping': 'xy_pad',
    },
    'constellation': {
        'description': 'Discrete points (sounds) connected by lines (morphs)',
        'physics': {
            'snap_to_nearest': True,
            'transition_time_ms': 200,
        },
        'visual': 'star_chart',
        'mapping': 'tap_to_select',
    },
    'wormhole': {
        'description': 'Extreme jumps between distant latent regions',
        'physics': {
            'warp_factor': 10,   # speed multiplier
            'tunnel_effect': True, # visual warp
        },
        'visual': 'hyperspace',
        'mapping': 'drag_to_warp',
    },
    'ocean': {
        'description': 'Slow, continuous drift through latent space',
        'physics': {
            'current': (0.01, 0.02),  # constant drift direction
            'wave_amplitude': 0.1,     # oscillation
            'wave_frequency': 0.5,     # Hz
        },
        'visual': 'waves',
        'mapping': 'tilt_to_navigate',
    },
    'garden': {
        'description': 'Plant sounds at positions, watch them grow and interact',
        'physics': {
            'growth_rate': 0.01,
            'interaction_radius': 0.2,
            'crossover_probability': 0.001,
        },
        'visual': 'growing_plants',
        'mapping': 'plant_and_tend',
    },
}
```

---

## 5. Emotion-Performance Mapping

```python
class EmotionalPerformanceMap:
    """Map performance gestures to emotional trajectories."""
    
    def __init__(self):
        # Emotional state as position in VAP space
        self.current_emotion = np.array([0.0, 0.0, 0.0])  # V, A, P
        self.target_emotion = np.array([0.0, 0.0, 0.0])
        
        # Gesture → emotional intent
        self.gesture_mood_map = {
            'aggressive_tapping':    np.array([-0.5, 0.8, 0.7]),
            'gentle_caress':         np.array([0.6, -0.3, -0.2]),
            'fast_sweep':            np.array([0.3, 0.7, 0.5]),
            'slow_drift':            np.array([0.4, -0.5, 0.1]),
            'erratic_jitter':        np.array([-0.3, 0.6, -0.1]),
            'deep_press':            np.array([0.2, 0.3, 0.9]),
            'circular_motion':       np.array([0.5, 0.2, 0.3]),
            'still_hold':            np.array([0.3, -0.6, 0.4]),
        }
        
    def update_from_gesture(self, gesture_type, gesture_data):
        """Update emotional target based on gesture."""
        if gesture_type in self.gesture_mood_map:
            mood_delta = self.gesture_mood_map[gesture_type]
            
            # Intensity scales with gesture velocity/pressure
            intensity = gesture_data.get('velocity', 0.5)
            self.target_emotion += mood_delta * intensity * 0.1
            self.target_emotion = np.clip(self.target_emotion, -1, 1)
    
    def get_current_emotional_state(self):
        """Get smoothed emotional state."""
        # Smooth interpolation
        self.current_emotion += (self.target_emotion - self.current_emotion) * 0.05
        return self.current_emotion
```

---

## 6. DAW Integration (VST3/AU)

```python
class LatentInstrumentPlugin:
    """DAW plugin that hosts the latent instrument."""
    
    # Parameters exposed to DAW automation
    parameters = [
        ('latent_x', -1.0, 1.0),
        ('latent_y', -1.0, 1.0),
        ('morph_speed', 0.0, 1.0),
        ('brightness', 0.0, 1.0),
        ('punch', 0.0, 1.0),
        ('texture', 0.0, 1.0),
        ('stereo_width', 0.0, 1.0),
        ('reverb', 0.0, 1.0),
        ('emotion_valence', -1.0, 1.0),
        ('emotion_arousal', -1.0, 1.0),
        ('emotion_power', -1.0, 1.0),
        ('performance_mode', 0, 5),  # index
    ]
    
    def process_block(self, midi_events, audio_buffer):
        """Process MIDI + automation to generate audio."""
        for event in midi_events:
            if event.type == NOTE_ON:
                # Generate one-shot at current latent position
                sound = self.latent_instrument.play(
                    self.get_current_gesture(event)
                )
                # Write to audio output
                self.output_sound(sound, event.velocity, audio_buffer)
            
            elif event.type == PITCH_BEND:
                # Morph in latent space
                self.latent_instrument.morph_to(
                    self.latent_instrument.current_position 
                    + np.array([event.value * 0.1, 0]),
                    duration_ms=50
                )
            
            elif event.type == CC:
                # Map CC to latent parameters
                cc_mappings = {
                    1: 'latent_x',
                    2: 'latent_y',
                    3: 'morph_speed',
                    74: 'brightness',  # common filter cutoff CC
                }
                param = cc_mappings.get(event.controller)
                if param == 'latent_x':
                    self.latent_instrument.current_position[0] = (event.value / 127) * 2 - 1
```
