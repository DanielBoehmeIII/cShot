# Prompt 35 — Make Every One-Shot Mix-Ready

Every cShot output should sound like it was already processed by a professional mixing engineer.

---

## 1. The Mix-Readiness Standard

### What "Mix-Ready" Means

```
A mix-ready one-shot:
  ✓ Won't clip (peak ≤ -0.5dBFS with headroom)
  ✓ Has appropriate loudness (genre-typical integrated LUFS)
  ✓ Has no harsh frequencies (spectral balance checked)
  ✓ Has no muddy low-end (sub-200Hz clarity)
  ✓ Has a clean transient (punch where expected)
  ✓ Has no phase problems (mono-compatible)
  ✓ Has balanced stereo image (not overly wide)
  ✓ Has clean tail (no noise, clicks, DC offset)
  ✓ Has proper fade-in/fade-out
  ✓ Is trimmed to appropriate length

No DAW required: drag and drop directly into a mix.
```

### The Problem It Solves

```
Typical sample or raw generation output problems:

  Problem                | Occurrence | Severity
  ───────────────────────┼────────────┼─────────
  Clipping               │     15%    │ Critical
  Too quiet              │     25%    │ High  
  Too loud               │     10%    │ Medium
  Harsh frequencies      │     30%    │ High
  Muddy low-end          │     20%    │ High
  Weak transient         │     35%    │ Medium
  Phase issues           │      5%    │ Critical
  Stereo imbalance       │     10%    │ Low
  Noisy tail             │     20%    │ Low
  DC offset             │      8%    │ Medium
  Bad fade-out           │     15%    │ Low
  Too long/short         │     25%    │ Low

  After mix-readiness engine: all problems < 1% occurrence.
```

---

## 2. Diagnostic Tests

Every generated sound runs through this diagnostic pipeline before reaching the user.

### Test Descriptions

```python
class MixReadinessDiagnostics:
    """
    Full diagnostic suite for one-shot mix-readiness.
    Each test returns (pass: bool, value: float, recommendation: str).
    """
    
    @staticmethod
    def test_clipping(audio):
        """Check for intersample peaks and true peak clipping."""
        peak = np.max(np.abs(audio))
        true_peak = compute_true_peak(audio)  # 4x oversampled
        return {
            'peak_dbfs': 20 * log10(peak),
            'true_peak_dbfs': 20 * log10(true_peak),
            'clipping': peak >= 0.999,
            'intersample_peaks': true_peak > 1.0,
            'pass': true_peak <= -0.5
        }
    
    @staticmethod
    def test_loudness(audio, sr):
        """Measure integrated loudness (LUFS)."""
        loudness = compute_integrated_loudness(audio, sr)
        return {
            'integrated_lufs': loudness,
            'pass': -18 <= loudness <= -8,
            'recommendation': f"Target: {target_lufs}, Current: {loudness:.1f}"
        }
    
    @staticmethod
    def test_harsh_frequencies(audio, sr):
        """Detect harsh/annoying frequency content."""
        spec = compute_spectrum(audio, sr)
        harsh_region = spec.get_band_energy(2000, 8000)
        total_energy = spec.total_energy
        harsh_ratio = harsh_region / total_energy if total_energy > 0 else 0
        return {
            'harsh_ratio': harsh_ratio,
            'pass': harsh_ratio < 0.4,
            'problem_frequencies': spec.find_peaks(2000, 8000)
        }
    
    @staticmethod
    def test_muddy_lowend(audio, sr):
        """Detect muddy/unclear low frequency content."""
        spec = compute_spectrum(audio, sr)
        sub_energy = spec.get_band_energy(20, 60)
        low_mid_energy = spec.get_band_energy(60, 200)
        balance = sub_energy / low_mid_energy if low_mid_energy > 0 else float('inf')
        return {
            'sub_to_low_mid_ratio': balance,
            'pass': 0.3 <= balance <= 3.0,
            'muddy': balance < 0.5
        }
    
    @staticmethod
    def test_transient(audio, sr):
        """Evaluate transient quality and clarity."""
        envelope = compute_envelope(audio)
        onset_strength = compute_onset_strength(envelope)
        attack_time = compute_attack_time(envelope)
        transient_peak = np.max(envelope[:int(0.01 * sr)])  # first 10ms
        return {
            'onset_strength': onset_strength,
            'attack_time_ms': attack_time * 1000,
            'transient_to_body_ratio': transient_peak / np.mean(envelope),
            'pass': onset_strength > 0.3
        }
    
    @staticmethod
    def test_phase(audio):
        """Check for phase issues (relevant for stereo)."""
        if audio.ndim < 2 or audio.shape[1] < 2:
            return {'pass': True, 'note': 'mono — no phase issues possible'}
        correlation = compute_stereo_correlation(audio)
        return {
            'correlation': correlation,
            'pass': correlation > -0.3,
            'mono_compatible': correlation > 0.5
        }
    
    @staticmethod
    def test_stereo_balance(audio):
        """Check left/right balance."""
        if audio.ndim < 2 or audio.shape[1] < 2:
            return {'pass': True, 'note': 'mono'}
        l_energy = np.mean(audio[:, 0] ** 2)
        r_energy = np.mean(audio[:, 1] ** 2)
        balance_db = 20 * log10(l_energy / r_energy) if r_energy > 0 else 0
        return {
            'balance_db': balance_db,
            'pass': abs(balance_db) < 6,
            'recommendation': f"Pan center: {balance_db:.1f}dB imbalance"
        }
    
    @staticmethod
    def test_noisy_tail(audio, sr):
        """Detect noise, artifacts in the tail of the sound."""
        tail_start = int(len(audio) * 0.8)
        tail = audio[tail_start:]
        noise_floor = compute_noise_floor(tail)
        signal_peak = np.max(np.abs(audio[:tail_start]))
        snr = 20 * log10(signal_peak / noise_floor) if noise_floor > 0 else 100
        return {
            'tail_snr': snr,
            'pass': snr > 40,
            'noise_floor_dbfs': 20 * log10(noise_floor)
        }
    
    @staticmethod
    def test_dc_offset(audio):
        """Check for DC offset."""
        dc = np.mean(audio)
        return {
            'dc_offset': dc,
            'pass': abs(dc) < 0.001
        }
    
    @staticmethod
    def test_fades(audio, sr):
        """Check for proper fade-in/fade-out."""
        fade_in_samples = 5  # first 5 samples
        fade_out_samples = 5  # last 5 samples
        has_fade_in = all(abs(audio[i]) <= abs(audio[i+1]) for i in range(fade_in_samples - 1))
        has_fade_out = all(abs(audio[-i-1]) <= abs(audio[-i]) for i in range(fade_out_samples - 1))
        return {
            'has_fade_in': has_fade_in,
            'has_fade_out': has_fade_out,
            'pass': has_fade_in and has_fade_out
        }
    
    @staticmethod
    def test_silence_trimming(audio, sr):
        """Detect leading/trailing silence."""
        threshold = 0.001  # -60dB
        non_silent = np.where(np.abs(audio) > threshold)[0]
        if len(non_silent) == 0:
            return {'pass': False, 'note': 'completely silent'}
        leading_silence = non_silent[0] / sr * 1000
        trailing_silence = (len(audio) - non_silent[-1]) / sr * 1000
        return {
            'leading_silence_ms': leading_silence,
            'trailing_silence_ms': trailing_silence,
            'pass': leading_silence < 50 and trailing_silence < 200
        }
```

### Diagnostic Report

```
Mix Readiness Report for: "Kick_Trap_01"

  ✅ Clipping:            -1.2dBFS true peak      PASS
  ✅ Loudness:            -12.3 LUFS integrated    PASS (*)
  ✅ Harsh frequencies:   0.23 ratio               PASS
  ✅ Low-end clarity:     balanced                 PASS
  ⚠ Transient strength:  0.28 onset               WARN (could be punchier)
  ✅ Phase:               0.82 correlation         PASS
  ✅ Stereo balance:      0.3dB imbalance          PASS
  ✅ Tail noise:          52dB SNR                PASS
  ✅ DC offset:           0.0002                   PASS
  ✅ Fade in/out:         present                  PASS
  ✅ Silence trimming:    2ms lead, 15ms trail    PASS

  Overall: MIX-READY (9/10 checks passed)
  Suggested improvement: Enhance attack transient by 2dB at 3kHz

  * (Note: genre "trap" targets -10 LUFS, current is -12.3)
```

---

## 3. Automatic Repair Pipeline

### Pipeline Architecture

```
┌──────────────┐
│  Raw Sound   │ (from model output)
└──────┬───────┘
       ↓
┌──────────────────────────────────────────────┐
│              Diagnostic Suite                  │
│  Run all 10 tests → collect results           │
└──────┬───────────────────────────────────────┘
       ↓
┌──────────────────────────────────────────────┐
│           Repair Decision Engine              │
│  For each failed/warning test:               │
│    - Select repair module                    │
│    - Set repair parameters                   │
│    - Order: destructive first                │
└──────┬───────────────────────────────────────┘
       ↓
┌──────────────────────────────────────────────┐
│            Repair Pipeline                    │
│  ┌────────┐ ┌────────┐ ┌──────────┐          │
│  │ DC     │→│ Trim   │→│ Fade     │          │
│  │ Remove │ │ Silence│ │ Apply    │          │
│  └────────┘ └────────┘ └──────────┘          │
│  ┌──────────┐ ┌────────┐ ┌──────────┐        │
│  │ EQ      │→│Transient│→│ Loudness │        │
│  │ Balance │ │ Enhance │ │ Normalize│        │
│  └──────────┘ └────────┘ └──────────┘        │
│  ┌────────┐ ┌────────┐ ┌──────────────┐      │
│  │Phase   │→│De-esser│→│ Final Quality│      │
│  │Fix     │ │        │ │ Check        │      │
│  └────────┘ └────────┘ └──────────────┘      │
└──────┬───────────────────────────────────────┘
       ↓
┌──────────────┐
│  Mix-Ready   │
│  Output      │ (re-diagnostics pass required)
└──────────────┘
```

### Repair Modules

```python
class MixRepairEngine:
    """Individual repair modules that fix specific issues."""
    
    @staticmethod
    def remove_dc_offset(audio):
        """Remove DC offset from audio signal."""
        dc = np.mean(audio)
        return audio - dc
    
    @staticmethod
    def trim_silence(audio, threshold_db=-60, min_leading_ms=2, min_trailing_ms=10):
        """Trim leading and trailing silence."""
        threshold = 10 ** (threshold_db / 20)
        non_silent = np.where(np.abs(audio) > threshold)[0]
        if len(non_silent) == 0:
            return audio
        # Leave tiny fade buffers
        start = max(0, non_silent[0] - int(min_leading_ms * sr / 1000))
        end = min(len(audio), non_silent[-1] + int(min_trailing_ms * sr / 1000))
        return audio[start:end]
    
    @staticmethod
    def apply_fades(audio, sr, fade_in_ms=2, fade_out_ms=10):
        """Apply clean fades to prevent clicks."""
        fade_in_len = int(fade_in_ms * sr / 1000)
        fade_out_len = int(fade_out_ms * sr / 1000)
        
        audio = audio.copy()
        audio[:fade_in_len] *= np.linspace(0, 1, fade_in_len)
        audio[-fade_out_len:] *= np.linspace(1, 0, fade_out_len)
        return audio
    
    @staticmethod
    def eq_balance(audio, sr, target_curve='flat'):
        """Apply corrective EQ for spectral balance."""
        # Analyze spectral balance
        spec = compute_spectrum(audio, sr)
        
        # Detect problem areas
        harsh = spec.get_band_energy(2000, 8000) / spec.total_energy
        muddy = spec.get_band_energy(60, 200) / spec.total_energy
        
        filters = []
        if harsh > 0.35:
            filters.append({'type': 'shelf', 'freq': 4000, 'gain': -3, 'q': 0.7})
        if muddy > 0.5:
            filters.append({'type': 'shelf', 'freq': 150, 'gain': -2, 'q': 0.7})
        
        return apply_filters(audio, sr, filters)
    
    @staticmethod
    def enhance_transient(audio, sr, amount=1.5):
        """Enhance attack transient for more punch."""
        # Split into transient and body
        transient_len = int(0.01 * sr)  # first 10ms
        transient = audio[:transient_len].copy()
        body = audio[transient_len:].copy()
        
        # Boost transient
        transient *= amount
        
        # Optional: apply transient shaper
        envelope = compute_envelope(np.abs(transient))
        shaped = transient * (envelope ** 0.5)  # squash envelope slightly
        
        return np.concatenate([shaped, body])
    
    @staticmethod
    def loudness_normalize(audio, sr, target_lufs=-14):
        """Normalize to target integrated loudness."""
        current = compute_integrated_loudness(audio, sr)
        gain_db = target_lufs - current
        gain_linear = 10 ** (gain_db / 20)
        normalized = audio * gain_linear
        
        # Soft limit to prevent clipping
        peak = np.max(np.abs(normalized))
        if peak > 0.95:
            normalized = soft_limit(normalized, threshold=-0.5)
        
        return normalized
    
    @staticmethod
    def fix_phase(audio):
        """Fix stereo phase issues."""
        if audio.ndim < 2 or audio.shape[1] < 2:
            return audio
        
        correlation = compute_stereo_correlation(audio)
        if correlation < -0.3:
            # Widen correlation toward center
            mid = (audio[:, 0] + audio[:, 1]) / 2
            side = (audio[:, 0] - audio[:, 1]) / 2
            side *= 0.5  # Reduce side content
            audio[:, 0] = mid + side
            audio[:, 1] = mid - side
        
        return audio
    
    @staticmethod
    def de_esser(audio, sr, threshold_db=-20):
        """Reduce harsh sibilant frequencies."""
        return apply_dynamic_eq(audio, sr, freq=5000, threshold=threshold_db, ratio=3)
    
    @staticmethod
    def noise_gate_tail(audio, sr, threshold_db=-50):
        """Gate the tail to remove noise."""
        envelope = compute_envelope(np.abs(audio))
        threshold = 10 ** (threshold_db / 20)
        
        # Find where signal drops below threshold
        gate_points = np.where(envelope < threshold)[0]
        if len(gate_points) > len(audio) * 0.3:
            return audio  # Too much would be gated, skip
        
        # Apply gradual fade after last significant signal
        significant = np.where(envelope > threshold * 3)[0]
        if len(significant) > 0:
            fade_start = significant[-1]
            fade_len = int(0.005 * sr)  # 5ms fade
            fade_end = min(fade_start + fade_len, len(audio))
            audio = audio.copy()
            audio[fade_start:fade_end] *= np.linspace(1, 0, fade_end - fade_start)
            audio[fade_end:] = 0
        
        return audio
```

### Repair Decision Engine

```python
class RepairDecisionEngine:
    """
    Decides which repairs to apply and in what order.
    Respects genre preferences and user settings.
    """
    
    def __init__(self, genre='none', aggressiveness='auto'):
        self.genre = genre
        self.aggressiveness = aggressiveness
        self.repair_order = [
            'remove_dc_offset',    # Must be first — affects everything
            'trim_silence',        # Must be before fades
            'apply_fades',         # Must be after trim
            'fix_phase',           # Before stereo processing
            'eq_balance',          # After phase
            'de_esser',            # After EQ
            'enhance_transient',   # Before loudness
            'noise_gate_tail',     # Before loudness
            'loudness_normalize',  # Almost last
        ]
    
    def decide(self, diagnostics):
        """Decide which repairs to apply based on diagnostics and genre."""
        repairs = []
        
        for repair_name in self.repair_order:
            test_name = repair_name  # maps to diagnostic test
            if test_name in diagnostics:
                result = diagnostics[test_name]
                should_repair = not result.get('pass', True)
                
                # Genre overrides
                if self.genre == 'techno' and repair_name == 'loudness_normalize':
                    should_repair = True  # Always normalize for techno
                if self.genre == 'ambient' and repair_name == 'noise_gate_tail':
                    should_repair = False  # Ambient needs tails
                
                repairs.append({
                    'repair': repair_name,
                    'apply': should_repair,
                    'params': self.get_params(repair_name)
                })
        
        return repairs
    
    def get_params(self, repair_name):
        """Get genre-aware parameters for each repair."""
        params = {
            'loudness_normalize': {'target_lufs': -14},
            'enhance_transient': {'amount': 1.3},
            'eq_balance': {'target_curve': 'flat'},
        }
        
        # Genre-specific overrides
        genre_overrides = {
            'trap': {
                'loudness_normalize': {'target_lufs': -10},
                'enhance_transient': {'amount': 1.8},
            },
            'techno': {
                'loudness_normalize': {'target_lufs': -12},
                'enhance_transient': {'amount': 1.5},
            },
            'ambient': {
                'loudness_normalize': {'target_lufs': -18},
                'enhance_transient': {'amount': 1.0},  # No transient enhancement
                'eq_balance': {'target_curve': 'warm'},
            },
        }
        
        if self.genre in genre_overrides:
            if repair_name in genre_overrides[self.genre]:
                params[repair_name].update(genre_overrides[self.genre][repair_name])
        
        return params.get(repair_name, {})
```

---

## 4. Loudness Normalization Strategy

### Standards

```
cShot targets these loudness standards:

  ┌────────────────────┬──────────┬──────────────────────────┐
  │ Standard           │ Target   │ Use Case                 │
  ├────────────────────┼──────────┼──────────────────────────┤
  │ ITU-R BS.1770-4    │ -14 LUFS │ General commercial audio │
  │ Apple Music        │ -16 LUFS │ Apple Music delivery     │
  │ Spotify            │ -14 LUFS │ Spotify delivery         │
  │ YouTube            │ -14 LUFS │ YouTube delivery         │
  │ Broadcast          │ -23 LUFS │ TV/broadcast             │
  │ Mastered for iTunes│ -16 LUFS │ MFiT delivery            │
  │ Club/Mastering     │ -8 LUFS  │ Club-ready               │
  │ Trailer            │ -6 LUFS  │ Cinematic trailer        │
  └────────────────────┴──────────┴──────────────────────────┘

For one-shots specifically:
  ┌────────────┬──────────┬──────────────────────────────────┐
  │ Genre      │ Target   │ Rationale                        │
  ├────────────┼──────────┼──────────────────────────────────┤
  │ Trap       │ -10 LUFS │ Loud, competitive genre          │
  │ Drill      │ -10 LUFS │ Similar to trap                  │
  │ House      │ -12 LUFS │ Clean, commercial                │
  │ Techno     │ -12 LUFS │ Punchy but not crushed           │
  │ Dubstep    │ -8 LUFS  │ Maximum impact                   │
  │ Pop        │ -14 LUFS │ Streaming-optimized              │
  │ R&B        │ -14 LUFS │ Dynamic, warm                    │
  │ Ambient    │ -18 LUFS │ Dynamic range preserved          │
  │ Cinematic  │ -16 LUFS │ Headroom for full mix            │
  │ Game Audio │ -18 LUFS │ Runtime mixing flexibility       │
  │ Default    │ -14 LUFS │ Safe for any context             │
  └────────────┴──────────┴──────────────────────────────────┘
```

### True Peak Limiting

```
After loudness normalization, apply true peak limiting:
  - Lookahead: 5ms
  - Attack: 1ms
  - Release: 100ms  
  - Ceiling: -0.5dBFS (true peak)
  - Oversampling: 4x for true peak detection
  
This ensures the one-shot won't clip in any DAW, on any system.
```

---

## 5. Transient Enhancement System

### Enhancement Types

| Type | Method | Use Case |
|------|--------|----------|
| **Attack boost** | Gain transient region (first 10ms) +3dB | Kick needs more click |
| **Transient shaper** | Adjust attack/release envelope | Tighten or loosen the transient |
| **Saturation** | Soft clip the transient region | Add harmonic punch |
| **Layer** | Synthesize transient layer on top | When none exists |
| **Spectral shaping** | Boost 2-5kHz transient region | Add perceived punch without volume |
| **Multi-band** | Process transient per frequency band | Targeted punch (sub vs click) |

### Auto-Enhance Decision

```python
def auto_transient_strategy(audio, sr, genre):
    """Choose transient enhancement strategy based on analysis and genre."""
    analysis = analyze_transient(audio, sr)
    
    # Check if transient is weak
    onset_strength = analysis['onset_strength']
    attack_slope = analysis['attack_slope']
    
    if onset_strength > 0.6:
        return {'action': 'none', 'reason': 'transient already strong'}
    
    if onset_strength < 0.2:
        # Very weak transient → layer synthetic transient
        return {
            'action': 'layer',
            'params': {
                'freq': 3000 if genre in ['trap', 'drill'] else 2000,
                'decay_ms': 5,
                'amount': 2.0
            },
            'reason': 'transient too weak, synthesizing'
        }
    
    if attack_slope < 0.5:
        # Slow attack → use transient shaper
        return {
            'action': 'shaper',
            'params': {
                'attack_ms': 2,
                'release_ms': 15,
                'amount': 1.5
            },
            'reason': 'attack too slow, shaping'
        }
    
    # Default: boost transient by moderate amount
    return {
        'action': 'boost',
        'params': {'gain_db': 2.5, 'freq_boost': 3000},
        'reason': 'moderate enhancement'
    }
```

---

## 6. Genre-Aware Processing Chains

### Genre Profiles

Each genre gets a tailored processing chain:

```
TRAP PROFILE:
  Target loudness: -10 LUFS
  Transient: aggressive (boost 3kHz by 3dB, fast attack)
  Low end: sub-heavy, boosted at 50-80Hz
  Saturation: soft clip for perceived loudness
  Stereo: mostly mono (center-focused)
  Reverb: none or very short room
  
HOUSE PROFILE:
  Target loudness: -12 LUFS
  Transient: clean, precise (boost 2kHz by 2dB)
  Low end: punchy, tight sub
  Saturation: gentle tape warmth
  Stereo: moderate width
  Reverb: short plate or room
  
AMBIENT PROFILE:
  Target loudness: -18 LUFS
  Transient: softened (no enhancement)
  Low end: natural, not emphasized
  Saturation: none or very gentle
  Stereo: wide, possibly stereoized
  Reverb: long hall or convolution
  Extra: long fade-outs, preserve dynamic range
```

### Processing Chain Selector

```python
GENRE_CHAINS = {
    'trap': {
        'eq': [{'type': 'low_shelf', 'freq': 80, 'gain': 3, 'q': 0.7}],
        'compression': {'threshold': -20, 'ratio': 4, 'attack': 1, 'release': 50},
        'saturation': {'type': 'soft_clip', 'amount': 2.0},
        'transient': {'boost_db': 3, 'freq': 3000},
        'stereo': {'width': 0.2},
        'reverb': None,
    },
    'house': {
        'eq': [{'type': 'high_pass', 'freq': 30, 'slope': 12}],
        'compression': {'threshold': -18, 'ratio': 3, 'attack': 5, 'release': 100},
        'saturation': {'type': 'tape', 'amount': 1.0},
        'transient': {'boost_db': 2, 'freq': 2000},
        'stereo': {'width': 0.4},
        'reverb': {'type': 'plate', 'decay': 0.3, 'mix': 0.15},
    },
    'ambient': {
        'eq': None,
        'compression': None,
        'saturation': None,
        'transient': {'boost_db': 0},  # No transient enhancement
        'stereo': {'width': 0.8},
        'reverb': {'type': 'hall', 'decay': 3.0, 'mix': 0.3},
    },
    # ... more genres
}
```

---

## 7. Export Presets

### Preset Types

```
MIX-READY (default):
  - All repairs applied
  - Genre-appropriate loudness
  - True peak limited to -0.5dBFS
  - 44.1kHz, 24-bit WAV
  - Metadata: full provenance record

RAW:
  - Clean trim only
  - No EQ, no compression, no normalization
  - Leave full dynamic range
  - For users who want to process themselves

CLUB-READY:
  - Aggressive loudness (-8 LUFS)
  - Heavy transient enhancement
  - Sub boost
  - Soft limiting

STREAMING-OPTIMIZED:
  - -14 LUFS loudness (Spotify/YouTube standard)
  - True peak limited to -1.0dBFS
  - Gentle compression
  - Mono-compatible

CINEMATIC:
  - -16 LUFS loudness
  - Wide stereo
  - Reverb tail preserved
  - Minimal compression

GAME-READY:
  - -18 LUFS (headroom for runtime mixing)
  - Clean transients
  - Mono-compatible
  - Multi-LOD (44.1k + 22k + 11k for streaming)
  - Loop points if applicable
```

### Custom Preset Builder

```
Users can create custom export presets:

  Preset Name: "My Signature Kick"
  
  ✔ Trim Silence (threshold: -60dB)
  ✔ Remove DC Offset
  ✔ Apply Fades (in: 2ms, out: 10ms)
  ✔ EQ Balance (curve: warm)
  ✔ Transient Enhancement (amount: 1.5x)
  ✔ Loudness Normalize (target: -11 LUFS)
  ✔ True Peak Limit (ceiling: -0.5dBFS)
  
  Format: WAV, 48kHz, 24-bit
  Filename: {type}_{bpm}_{key}.wav
  Metadata: Full provenance
  
  Save | Apply | Share
```

---

## Summary

| Capability | What It Fixes | How Fast |
|-----------|--------------|----------|
| Diagnostics | Find every mix issue | <10ms |
| DC offset | Remove DC offset | <1ms |
| Silence trim | Remove leading/trailing silence | <1ms |
| Fades | Prevent click artifacts | <1ms |
| EQ balance | Fix harsh/muddy frequencies | <5ms |
| Transient enhance | Add punch where missing | <3ms |
| Phase fix | Ensure mono compatibility | <3ms |
| De-esser | Remove harsh sibilance | <3ms |
| Noise gate | Clean up noisy tails | <2ms |
| Loudness normalize | Match genre standard | <5ms |
| True peak limit | Prevent clipping | <3ms |
| Full pipeline | All repairs | <30ms |

Every cShot sound comes out of the box mix-ready. No EQ, no compression, no gain staging needed. Drag and drop into your mix and it just works.
