# Prompt 13 — Invent a Latent Space for One-Shots

cShot models one-shots as trajectories through latent acoustic space.

---

## 1. The One-Shot as a Trajectory

A one-shot is not a static object — it is an **event unfolding in time**. cShot's latent space captures this by modeling each one-shot as a trajectory through a latent manifold.

### 1.1 Trajectory Model

```
One-shot = {
    attack_trajectory:  [z_0, z_1, ..., z_T]   (first 50ms, high temporal resolution)
    body_trajectory:    [z_T, z_T+1, ..., z_S]  (sustain/body, lower resolution)
    release_trajectory: [z_S, z_S+1, ..., z_E]  (decay/release)
    global_code:        z_global                 (static attributes: genre, type, key)
}
```

Each `z_i` is a point in a 128-D latent space. The trajectory is the sequence of latent states the sound passes through.

### 1.2 Why Trajectories?

| Property | Static Embedding | Trajectory |
|----------|-----------------|------------|
| Transient shape | Lost | Encoded in path curvature at onset |
| Temporal evolution | Collapsed | Preserved as sequential structure |
| Sound-as-process | Invisible | Explicitly modeled |
| Interpolation | Linear (artifact-prone) | Along learned geodesics |
| Variation | Single latent perturbation | Trajectory warping (more expressive) |

---

## 2. Emergent Latent Dimensions

Through contrastive pretraining on 1M+ one-shots, we expect these natural axes to emerge:

### 2.1 Spectral Dimensions

| Axis | What It Encodes | Observable In Latent Space |
|------|-----------------|---------------------------|
| Spectral Centroid | High vs low frequency center | Linear direction, monotonic change in centroid |
| Brightness | HF content ratio | Orthogonal to centroid, captures HF detail |
| Harmonicity | Tonal vs noisy content | Boundary between harmonic/inharmonic clusters |
| Spectral Width | Bandwidth of energy | Spread along frequency axis |
| Formant Structure | Spectral peak locations | Localized clusters in spectral subspace |

### 2.2 Temporal Dimensions

| Axis | What It Encodes | Observable In Latent Space |
|------|-----------------|---------------------------|
| Attack Sharpness | Transient onset speed | Changes in trajectory curvature at start |
| Decay Rate | How fast sound dies | Slope of trajectory toward origin |
| Envelope Shape | ADSR morphology | Overall path shape through latent space |
| Temporal Centroid | Where energy concentrates in time | Position along time-axis of trajectory |
| Onset Density | Number of micro-transients | High-frequency jitter along trajectory |

### 2.3 Perceptual Dimensions

| Axis | What It Encodes |
|------|-----------------|
| Punch | Combined sharpness + weight (diagonal direction) |
| Warmth | Low spectral centroid + harmonic density |
| Texture | Spectral irregularity + noise content |
| Width | Stereo divergence of L/R trajectories |
| Depth | Reverb tail length in trajectory decay |

### 2.4 Semantic Dimensions

| Axis | What It Encodes |
|------|-----------------|
| Genre Tendency | Cluster density near genre prototypes |
| Instrument Type | Region in latent space (drum, synth, foley, etc.) |
| Production Era | Temporal drift in latent style space |
| Emotional Valence | Aligned with VAP coordinate from Prompt 12 |
| Complexity | Trajectory length / fractal dimension |

---

## 3. Latent Space Architecture

### 3.1 Encoder

```python
class OneShotEncoder(nn.Module):
    """Maps raw audio to latent trajectory + global code."""
    
    def __init__(self):
        # Multi-resolution frontend
        self.mel_encoder = MelEncoder()           # 128-band mel -> feature map
        self.wave_encoder = WaveEncoder()         # raw waveform -> feature map
        
        # Temporal encoder (processes time dimension)
        self.temporal_encoder = nn.TransformerEncoder(
            num_layers=6, d_model=512, nhead=8
        )
        
        # Projection heads
        self.trajectory_proj = nn.Linear(512, 128)   # per-timestep latent
        self.global_proj = nn.Sequential(
            nn.AdaptiveAvgPool1d(1),
            nn.Flatten(),
            nn.Linear(512, 128)
        )
        
    def forward(self, audio):
        mel_feats = self.mel_encoder(audio)      # [B, T, 256]
        wave_feats = self.wave_encoder(audio)    # [B, T, 256]
        combined = torch.cat([mel_feats, wave_feats], dim=-1)  # [B, T, 512]
        
        # Temporal encoding with attention
        encoded = self.temporal_encoder(combined)  # [B, T, 512]
        
        # Trajectory (per-timestep latent codes)
        trajectory = self.trajectory_proj(encoded)  # [B, T, 128]
        
        # Global code (pooled across time)
        global_code = self.global_proj(encoded)     # [B, 128]
        
        return trajectory, global_code
```

### 3.2 Decoder (Generator)

```python
class OneShotDecoder(nn.Module):
    """Maps latent trajectory + global code back to audio."""
    
    def __init__(self):
        self.trajectory_encoder = nn.TransformerEncoder(
            num_layers=4, d_model=128, nhead=4
        )
        self.neural_vocoder = NeuralVocoder()  # e.g., HiFi-GAN or diffusion
        self.dsp_control = DSPParameterDecoder()  # predicts synth params
        
    def forward(self, trajectory, global_code, mode='hybrid'):
        # Condition on global code
        T, D = trajectory.shape
        global_expanded = global_code.unsqueeze(0).expand(T, -1)
        conditioned = torch.cat([trajectory, global_expanded], dim=-1)
        
        # Process through decoder
        decoded = self.trajectory_encoder(conditioned)
        
        if mode == 'neural':
            return self.neural_vocoder(decoded)
        elif mode == 'hybrid':
            dsp_params = self.dsp_control(decoded)
            neural_residual = self.neural_vocoder(decoded)
            return self.render_dsp_plus_residual(dsp_params, neural_residual)
```

### 3.3 Training Objectives

| Loss | Weight | Purpose |
|------|--------|---------|
| Reconstruction (Mel + Waveform) | 1.0 | Faithful reconstruction |
| KL Divergence (VAE variant) | 0.1 | Smooth latent space |
| Contrastive (SimCLR) | 0.5 | Disentangle meaningful dimensions |
| Trajectory Smoothness | 0.01 | Encourage smooth latent paths |
| Perceptual Loss (from Prompt 11) | 0.5 | Align with human perception |
| Semantic Consistency | 0.3 | Same-label sounds cluster |
| Cycle Consistency | 0.2 | encode(decode(z)) ≈ z |

---

## 4. Navigation Systems

### 4.1 Latent Arithmetic

```python
# Vector arithmetic in latent space
kick_dark = encode("dark_kick.wav")
kick_bright = encode("bright_kick.wav")
snare = encode("snare.wav")

# Find "darkness" direction
darkness_vector = kick_dark.z_global - kick_bright.z_global

# Apply to snare
snare_darkened = decode(snare.z_global + darkness_vector * 0.5)

# Sound analogies
# "snare that punches like this kick"
snare_with_kick_punch = decode(
    snare.z_global - snare.attack_trajectory[0] + kick.attack_trajectory[0]
)
```

### 4.2 Axis Traversal

```python
def traverse_axis(sound, axis='punch', amount=1.0, n_steps=10):
    """Move sound along a perceptual axis in latent space."""
    z_global, trajectory = encode(sound)
    
    # Find the direction vector for this axis
    # (precomputed via linear probe or PCA on labeled data)
    direction = latent_axes[axis]  # shape: [128]
    
    results = []
    for alpha in linspace(-amount, amount, n_steps):
        z_moved = z_global + alpha * direction
        # Optionally also warp trajectory
        trajectory_moved = trajectory + alpha * direction.unsqueeze(0)
        audio = decode(trajectory_moved, z_moved)
        results.append(audio)
    return results
```

### 4.3 Geodesic Interpolation

```python
def interpolate_sounds(sound_a, sound_b, n_steps=10):
    """Interpolate along latent geodesic (not straight line)."""
    traj_a, global_a = encode(sound_a)
    traj_b, global_b = encode(sound_b)
    
    # Project to lower-dim for geodesic (2D UMAP for path planning)
    umap_coords_a = umap_project(global_a)
    umap_coords_b = umap_project(global_b)
    
    # Find geodesic path (avoiding low-density regions)
    path_2d = plan_geodesic(umap_coords_a, umap_coords_b)
    
    # Map back to latent space
    path_z = umap_inverse(path_2d)
    
    results = []
    for z in path_z:
        # Reconstruct trajectory from waypoint
        trajectory = reconstruct_trajectory(z, traj_a, traj_b)
        audio = decode(trajectory, z)
        results.append(audio)
    return results
```

### 4.4 Style Transfer

```python
def transfer_style(content_sound, style_sound, alpha=0.5):
    """Transfer temporal style while preserving content."""
    content_traj, content_global = encode(content_sound)
    style_traj, style_global = encode(style_sound)
    
    # Content = global code (spectral identity)
    # Style = trajectory dynamics (temporal behavior)
    
    new_global = content_global  # keep content
    new_trajectory = lerp(content_traj, style_traj, alpha)  # blend style
    
    return decode(new_trajectory, new_global)
```

---

## 5. Variation Generation

### 5.1 Latent Perturbation

```python
def generate_variations(sound, n=10, variation_scale=0.3):
    z_global, trajectory = encode(sound)
    variations = []
    for _ in range(n):
        noise = torch.randn_like(z_global) * variation_scale
        # Anisotropic noise (less in important directions)
        noise = noise * importance_scaling(z_global)
        z_var = z_global + noise
        variations.append(decode(trajectory, z_var))
    return variations
```

### 5.2 Trajectory Warping

```python
def warp_trajectory(sound, warp_type='stretch', amount=1.5):
    """Warp the temporal trajectory without changing global code."""
    traj, global_code = encode(sound)
    
    if warp_type == 'stretch':
        # Slow down / speed up latent path
        warped = time_stretch(traj, amount)
    elif warp_type == 'amplify':
        # Make trajectory more pronounced
        mean_traj = traj.mean(dim=0, keepdim=True)
        warped = mean_traj + (traj - mean_traj) * amount
    elif warp_type == 'smooth':
        # Reduce trajectory jitter
        warped = gaussian_filter(traj, sigma=amount)
    elif warp_type == 'repeat':
        # Loop section of trajectory
        warped = repeat_section(traj, amount)
    
    return decode(warped, global_code)
```

### 5.3 Generative Flow on Latent Manifold

```python
class LatentFlow(nn.Module):
    """Learn the density of one-shots in latent space for sampling."""
    
    def __init__(self):
        self.flow = ContinuousNormalizingFlow(
            dim=128, hidden_dims=[512, 512, 256]
        )
        
    def sample(self, n=1, condition=None):
        z = self.flow.sample(n)  # sample from learned density
        # Decode using a learned prior trajectory
        trajectory = learned_trajectory_prior(z)
        return decode(trajectory, z)
```

---

## 6. Latent Clustering

### 6.1 Hierarchical Clustering

```
Level 1: SUPERCLASS (4 clusters)
    - Percussive (kicks, snares, hats, claps)
    - Tonal (keys, pads, basses, leads)
    - Texture (risers, impacts, atmospheres)
    - Foley (noise, organic, found sound)

Level 2: CLASS (16 clusters)
    - Kick, Snare, Hi-hat, Clap, Tom
    - Bass, Pad, Lead, Key, Pluck
    - Riser, Impact, Atmosphere, Sweep
    - Foley, Glitch

Level 3: SUBCLASS (64 clusters)
    - Deep Kick, Punchy Kick, 808 Kick, Acoustic Kick...
    - Open Hat, Closed Hat, Ride, Crash...
```

### 6.2 Online Clustering

As new one-shots are added, they are assigned to clusters via nearest-centroid in latent space. Clusters can split when variance exceeds threshold.

### 6.3 Perceptual Cluster Validation

For each cluster, compute:
- **Intra-cluster perceptual variance** (are cluster members really similar?)
- **Inter-cluster perceptual separation** (are clusters really different?)
- **Cluster coherence** (do cluster members share labels?)

---

## 7. Control Interface

The latent navigation will be exposed in cShot as:

| Control | Maps To |
|---------|---------|
| Macro knobs (1-8) | Learned latent axes (PCA directions) |
| Micro knobs (1-8) | Trajectory warp parameters |
| XY Pad | 2D latent slice (selectable axes) |
| Morph slider | Linear or geodesic interpolation |
| Variation dial | Perturbation scale |
| Style transfer | Source → target semantic mapping |
| Latent browser | 2D map of entire library |
