# Prompt 21 — Treat One-Shots as Physical Objects

cShot models one-shots as physical acoustic objects, not audio files.

---

## 1. The Physics of One-Shots

### 1.1 What Physically Creates a "Good" Kick?

A kick drum is a physical system: a membrane (head) coupled to an air cavity (shell) with a mechanical impactor (beater).

```
Physical Kick Anatomy:
  Membrane (head):     tensioned circular membrane, struck by beater
  Air cavity (shell):  Helmholtz resonator, adds low-frequency body
  Beater:              felt/wood/plastic, determines attack transient
  Port hole:           tuned leakage for pitch control
  Mechanical coupling: head → air → shell → floor (tactile feel)

Good kick = 
  1. Fast attack transient (beater impact, <5ms rise)
  2. Pitch drop (head stretches → pitch falls, "thump")
  3. Resonant body (shell resonance, 40-100Hz)
  4. Controlled decay (damping from head tension + port)
  5. Subsonic extension (room coupling, <40Hz felt not heard)
```

### 1.2 Why Certain Snares Feel Realistic

```
Snare Anatomy:
  Top head:    thin, resonant, struck by stick
  Bottom head: thin, sensitive, snares rest on it
  Snares:      spiral wire coils that buzz against bottom head
  Shell:       metal or wood, determines fundamental character

Realistic snare = 
  1. Stick impact (sharp transient, wood-on-mylar)
  2. Head resonance (ringy, 200-400Hz fundamental)
  3. Snare buzz (wire rattle, chaotic, micro-delays)
  4. Shell character (metal = bright, wood = warm)
  5. Air movement (pressure wave from port hole)
```

### 1.3 Why Impacts Feel Cinematic

```
Cinematic Impact Physics:
  Source:        large object, high velocity, hard surface
  Transient:     massive broadband impulse (all frequencies simultaneously)
  Body:          low-frequency thump from object resonance (50-150Hz)
  Debris:        high-frequency particle scattering (breaking/crumbling)
  Reverb:        large space (cathedral/warehouse) for scale
  Tactile:       infrasonic component (10-30Hz, felt in chest)
  Psychoacoustic:precedence effect → perceived size and distance

cinematic_punch ∝ log(mass × velocity² × surface_hardness)
```

### 1.4 Material Perception from Sound

Humans identify materials from sound alone with surprising accuracy:

| Material | Acoustic Signature |
|----------|-------------------|
| Wood | Broadband impact, mid-focused resonance with harmonics, fast decay |
| Metal | Bright, long-ringing harmonics, high spectral centroid, slow decay |
| Glass | Very bright, pure tones, rapid crack, shattering sustain |
| Plastic | Dull thud, low-mid focus, little high-frequency content, quick decay |
| Stone | Heavy thud, very low resonance, minimal decay, massive feel |
| Organic | Irregular spectrum, non-harmonic partials, textured noise |
| Water | Splash: rising pitch, chaotic bubbles, wide stereo |
| Fabric | Minimal attack, muted highs, compressed envelope |

Material acoustics = f(density, stiffness, internal damping, shape, excitation type)

---

## 2. Material-Aware Sound Model

### 2.1 Physical Parameter Space

```python
class PhysicalMaterial:
    """Physical properties of a sound-producing object."""
    
    def __init__(self, material_type='custom'):
        self.density = 1.0           # g/cm³ (0.1-20)
        self.stiffness = 1.0         # Young's modulus GPa (0.01-1000)
        self.internal_damping = 0.01  # loss factor (0.001-0.5)
        self.shape_factor = 0.5      # 0=plate, 1=sphere
        self.thickness = 0.01        # meters (0.001-0.1)
        self.size = 0.3              # meters (0.01-10)
        self.surface_hardness = 0.7  # 0=soft, 1=hard
        self.roughness = 0.3         # surface texture (0-1)
        
class PhysicalExcitation:
    """How the object is excited."""
    
    def __init__(self, excitation_type='strike'):
        self.type = excitation_type  # strike, scrape, pluck, bow, blow, shake
        self.velocity = 5.0          # m/s (0.1-50)
        self.mass = 0.1              # kg of striking object (0.001-10)
        self.contact_area = 0.001    # m² (0.0001-0.1)
        self.contact_hardness = 0.8  # 0=soft mallet, 1=hard stick
        self.angle = 90              # degrees (0-90, normal to surface)
        
class PhysicalEnvironment:
    """Acoustic environment."""
    
    def __init__(self):
        self.room_size = 10          # meters (1-100)
        self.reverberation = 0.3     # absorption coefficient (0-1)
        self.air_density = 1.2       # kg/m³
        self.temperature = 20        # Celsius
```

### 2.2 Physical Synthesis Engine

```python
class PhysicalSynthesizer:
    """Generate one-shots from physical parameters."""
    
    def synthesize(self, material, excitation, environment):
        components = {}
        
        # 1. Transient (impact impulse)
        components['transient'] = self.synthesize_impact(
            velocity=excitation.velocity,
            mass=excitation.mass,
            contact_hardness=excitation.contact_hardness,
            surface_hardness=material.surface_hardness,
            roughness=material.roughness
        )
        
        # 2. Resonant body (modal sum)
        components['resonance'] = self.synthesize_resonance(
            density=material.density,
            stiffness=material.stiffness,
            damping=material.internal_damping,
            shape=material.shape_factor,
            thickness=material.thickness,
            size=material.size,
            excitation_position=excitation.contact_area / material.size
        )
        
        # 3. Material noise (surface texture)
        components['texture'] = self.synthesize_material_noise(
            roughness=material.roughness,
            excitation_type=excitation.type,
            material_type=type(material).__name__
        )
        
        # 4. Environmental coupling
        components['environment'] = self.synthesize_environment(
            room_size=environment.room_size,
            absorption=environment.reverberation,
            air_density=environment.air_density
        )
        
        # Mix components
        audio = self.mix_physical_components(components)
        return audio
    
    def synthesize_impact(self, velocity, mass, contact_hardness, 
                           surface_hardness, roughness):
        """Physical impact transient."""
        # Impact force profile
        peak_force = velocity * mass * (contact_hardness + surface_hardness) / 2
        contact_time = 1 / (peak_force ** 0.3) * 0.001  # seconds
        attack = 0.1 * contact_time
        decay = 3 * contact_time
        
        # Envelope (Hann window shape)
        n = int((attack + decay) * 44100)
        envelope = np.sin(np.pi * np.arange(n) / n) ** 1.5
        
        # Broadband noise shaped by contact characteristics
        noise = white_noise(n)
        noise = lowpass(noise, 5000 + 15000 * contact_hardness)
        noise = highpass(noise, 100 * (1 - roughness))
        
        # Impact roughness (micro-impulses from surface irregularities)
        if roughness > 0.3:
            n_micro = int(roughness * 20)
            for _ in range(n_micro):
                delay = int(random() * n * 0.2)
                amp = random() * roughness * 0.3
                noise[delay:] += amp * noise[:-delay] if delay > 0 else 0
        
        return noise * envelope
    
    def synthesize_resonance(self, density, stiffness, damping,
                              shape, thickness, size, excitation_position):
        """Modal resonance using finite-difference or modal synthesis."""
        # Compute modal frequencies for the object
        modes = self.compute_modal_frequencies(
            density, stiffness, shape, thickness, size
        )
        
        # Each mode = frequency, decay, amplitude
        resonance = np.zeros(int(44100 * 2))  # 2 second buffer
        t = np.arange(len(resonance)) / 44100
        
        for freq, amp, decay_rate in modes:
            if freq > 20000:
                continue
            # Mode onset delayed by propagation from excitation point
            onset_delay = excitation_position * size / 344
            delay_samples = int(onset_delay * 44100)
            if delay_samples >= len(resonance):
                continue
            
            mode_signal = amp * np.exp(-decay_rate * t) * np.sin(2 * np.pi * freq * t)
            resonance[delay_samples:delay_samples + len(mode_signal)] += mode_signal
        
        return resonance
    
    def compute_modal_frequencies(self, density, stiffness, shape, thickness, size):
        """Compute resonant modes for a physical object.
        
        Uses analytical solutions for basic shapes:
        - Circular membrane (drum head)
        - Rectangular plate (metal sheet)
        - Sphere (bell)
        - Bar (xylophone key)
        - Helmholtz resonator (drum shell)
        """
        modes = []
        
        # Clamped circular membrane modes (Bessel function zeros)
        # f_mn = (alpha_mn / (2*pi*a)) * sqrt(T/sigma)
        # where alpha_mn are Bessel function zeros
        bessel_zeros = [2.405, 3.832, 5.136, 5.520, 6.380, 7.016]
        tension = stiffness * thickness  # approximate
        areal_density = density * thickness
        
        for n_mode, alpha in enumerate(bessel_zeros[:6]):
            freq = (alpha / (2 * np.pi * size)) * np.sqrt(tension / areal_density)
            amp = 1.0 / (n_mode + 1)  # higher modes quieter
            decay = damping * freq * 0.001  # frequency-dependent damping
            modes.append((freq, amp, decay))
        
        return modes
    
    def synthesize_material_noise(self, roughness, excitation_type, material_type):
        """Surface texture and material-specific noise."""
        duration = 0.5 if excitation_type == 'strike' else 2.0
        n = int(duration * 44100)
        
        # Material-specific noise profiles
        noise_profiles = {
            'metal': {'brightness': 0.9, 'granularity': 0.3, 'modulation': 0.6},
            'wood':  {'brightness': 0.5, 'granularity': 0.5, 'modulation': 0.3},
            'glass': {'brightness': 0.95, 'granularity': 0.2, 'modulation': 0.1},
            'plastic': {'brightness': 0.3, 'granularity': 0.7, 'modulation': 0.2},
            'stone': {'brightness': 0.2, 'granularity': 0.6, 'modulation': 0.1},
        }
        profile = noise_profiles.get(material_type, 
                                      {'brightness': 0.5, 'granularity': 0.5, 'modulation': 0})
        
        noise = white_noise(n)
        noise = lowpass(noise, 2000 + 18000 * profile['brightness'])
        
        # Granular modulation for realistic texture
        if profile['granularity'] > 0.2:
            grain_env = granular_envelope(n, grain_size_ms=5, 
                                          density=profile['granularity'])
            noise = noise * grain_env
        
        return noise * roughness * 0.3
    
    def mix_physical_components(self, components):
        """Mix physical components with realistic ratios."""
        # Typical physical energy distribution
        mix = (
            components['transient'] * 0.3 +
            components['resonance'] * 0.5 +
            components['texture'] * 0.1 +
            components['environment'] * 0.1
        )
        return normalize(mix)
```

---

## 3. Procedural Impact System

### 3.1 Impact Parameterization

```python
class ImpactDesigner:
    """Design impacts from physical parameters."""
    
    impact_presets = {
        'cinematic_boom': {
            'mass': 100,      # kg
            'velocity': 20,   # m/s
            'contact_hardness': 0.9,
            'surface_hardness': 0.8,
            'size': 5,        # meters
            'room_size': 50,  # cathedral
            'material': 'stone',
            'low_freq_boost': 12,  # dB
        },
        'punchy_kick': {
            'mass': 0.5,
            'velocity': 8,
            'contact_hardness': 0.7,
            'surface_hardness': 0.6,
            'size': 0.4,
            'room_size': 5,
            'material': 'wood',
            'low_freq_boost': 6,
        },
        'glassy_impact': {
            'mass': 0.1,
            'velocity': 15,
            'contact_hardness': 0.95,
            'surface_hardness': 0.95,
            'size': 0.2,
            'room_size': 8,
            'material': 'glass',
            'low_freq_boost': 0,
        },
        'metallic_hit': {
            'mass': 2,
            'velocity': 10,
            'contact_hardness': 0.85,
            'surface_hardness': 0.9,
            'size': 0.5,
            'room_size': 10,
            'material': 'metal',
            'low_freq_boost': 3,
        },
        'sub_bass_impact': {
            'mass': 200,
            'velocity': 5,
            'contact_hardness': 0.3,
            'surface_hardness': 0.2,
            'size': 10,
            'room_size': 10,
            'material': 'organic',
            'low_freq_boost': 18,
        },
    }
    
    def morph_impact(self, preset_a, preset_b, amount):
        """Morph between two impact designs."""
        morphed = {}
        for key in preset_a:
            morphed[key] = lerp(preset_a[key], preset_b[key], amount)
        return morphed
```

### 3.2 Impact Morphing Space

```
Impact types form a continuous space:
  
  Material hardness:  soft (0) ───────────────────── hard (1)
  Object mass:        small (0) ────────────────────── large (1)
  Impact velocity:    slow (0) ────────────────────── fast (1)
  Surface texture:    smooth (0) ────────────────── rough (1)
  Room size:          small (0) ───────────────────── large (1)
  
  Navigating this space = exploring all possible physical impacts
  
  Preset positions:
    Punchy kick:      (0.6, 0.3, 0.5, 0.4, 0.2)
    Cinematic boom:   (0.8, 0.9, 0.8, 0.3, 0.9)
    Glass shatter:    (0.95, 0.1, 0.9, 0.1, 0.3)
    Metallic clang:   (0.85, 0.4, 0.6, 0.2, 0.4)
    Explosion:        (0.5, 1.0, 0.9, 0.5, 0.8)
    Subsonic thud:    (0.2, 0.8, 0.3, 0.6, 0.3)
```

---

## 4. Hybrid Neural + Physical Architecture

### 4.1 Architecture

```python
class HybridPhysicalNeuralSynth(nn.Module):
    """Neural frontend predicts physical params; physics engine renders."""
    
    def __init__(self):
        self.physical_engine = PhysicalSynthesizer()
        
        # Neural physical parameter predictor
        self.param_predictor = nn.Sequential(
            nn.Linear(512, 256),  # input: text/latent embedding
            nn.GELU(),
            nn.Linear(256, 128),
            nn.GELU(),
            nn.Linear(128, 32)    # output: 32 physical parameters
        )
        
        # Neural residual corrector (fixes physics engine artifacts)
        self.residual_net = nn.Sequential(
            nn.Conv1d(1, 16, 3, padding=1),
            nn.GELU(),
            nn.Conv1d(16, 32, 3, padding=1),
            nn.GELU(),
            nn.Conv1d(32, 16, 3, padding=1),
            nn.GELU(),
            nn.Conv1d(16, 1, 3, padding=1)
        )
        
        # Neural perceptual optimizer (closed-loop correction)
        self.perceptual_optimizer = PerceptualOptimizer()
        
    def forward(self, latent_code, n_iterations=3):
        # Stage 1: Predict physical parameters
        phys_params = self.param_predictor(latent_code)
        
        # Stage 2: Physics engine renders base sound
        materials = self.params_to_materials(phys_params[:16])
        excitation = self.params_to_excitation(phys_params[16:24])
        environment = self.params_to_environment(phys_params[24:32])
        
        base_audio = self.physical_engine.synthesize(materials, excitation, environment)
        
        # Stage 3: Neural refinement (perceptual correction loop)
        audio = base_audio
        for _ in range(n_iterations):
            residual = self.residual_net(audio.unsqueeze(0)).squeeze(0)
            audio = audio + residual * 0.1
            
            # Perceptual check and adjust
            perceptual_score = self.perceptual_optimizer.evaluate(audio, latent_code)
            if perceptual_score > 0.9:
                break
            
            # Feedback: adjust physics params
            grad = self.perceptual_optimizer.compute_gradient(audio, latent_code)
            phys_params = phys_params + 0.1 * grad[:32]
        
        return audio
```

### 4.2 Training

```
Training data: 
  - Real recordings with analyzed physical parameters
  - Synthetic pairs (random physical params → physics engine output)
  - Human-rated material realism

Losses:
  - Physical parameter L1 (supervised from analyzed recordings)
  - Adversarial realism (discriminator judges "real" vs synthesized)
  - Perceptual similarity (embedding distance to real recordings)
  - Material classification (auxiliary task: predict material type)
```

---

## 5. Applications

| Application | How It Works |
|-------------|--------------|
| **Material browser** | Filter sounds by material property (wooden kick, metallic snare) |
| **Physical morphing** | Gradually change density/stiffness/size → sound morphs naturally |
| **Impossible materials** | Set physical parameters to non-existent values → novel sounds |
| **Hybrid design** | "Make a kick that sounds like stone but resonates like glass" |
| **Realistic layering** | Physics-aware layering: combine multiple physical objects |
| **Material transfer** | Take resonance of one object, impact of another |
| **Cinematic design** | Physically accurate explosions, crashes, impacts |
| **Foley generation** | Procedural foley from physical parameters |
