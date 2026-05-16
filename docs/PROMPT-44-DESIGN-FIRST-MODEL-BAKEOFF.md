# Prompt 44 — Design the First Model Bakeoff

A controlled experiment to compare which model/API is best for one-shot sound generation.

---

## 1. Experiment Overview

| Field | Value |
|-------|-------|
| Goal | Determine which generation tool produces the best one-shot sounds for cShot |
| Contenders | ElevenLabs SFX API, Stable Audio Open 1.0, AudioLDM 2, Stable Audio API |
| Total sounds | 400 (4 models × 20 prompts × 5 seeds) |
| Duration | 2 days of automated generation + 2 days of evaluation |
| Budget | ~$30 (ElevenLabs + Stable Audio API costs. Open models are free) |
| Output | Ranked model scores + recommendation + decision gate |

---

## 2. Input Prompts (20)

### Kick Drums (5 prompts)

```
1. "punchy kick drum 140bpm, short attack, sub-bass body"
2. "808 kick deep boomy hip hop"
3. "tight electronic kick, clicky attack, short decay"
4. "rock kick drum, natural, beater attack, medium body"
5. "experimental kick, distorted, lo-fi, 80bpm"
```

### Snares & Claps (5 prompts)

```
6. "crack snare drum 100bpm, tight, bright, metal shell"
7. "trap snare, layered clap, short, punchy, 140bpm"
8. "rimshot, wooden, dry, acoustic"
9. "deep clap, reverb, wide, house music"
10. "military snare, marching, crisp, snappy"
```

### Hi-Hats & Percussion (5 prompts)

```
11. "closed hi-hat, tight, bright, 130bpm"
12. "open hi-hat, wash, sizzle, electronic"
13. "shaker loop, subtle, high-end, organic"
14. "cowbell, metallic, cutting, dance music"
15. "tambourine hit, bright, jingle, acoustic"
```

### FX & Tonal (5 prompts)

```
16. "riser sound effect, tension build, electronic, 4 bars"
17. "sub bass hit, deep, 808, long release"
18. "glitch glitchy impact, stutter, digital, short"
19. "orchestral hit, cinematic, dramatic, brass and strings"
20. "reverse cymbal, swells, wash, build-up"
```

---

## 3. Reference Audio Snippets (for conditioning/model input)

For each category, include one professional reference file:
- `ref_kick.wav` — Professional kick from Splice (CC0 licensed)
- `ref_snare.wav` — Professional snare
- `ref_hat.wav` — Professional hi-hat
- `ref_perc.wav` — Professional percussion
- `ref_fx.wav` — Professional FX

Reference files serve two purposes:
1. Baseline comparison target (FAD calculation)
2. Audio-conditioned generation (for models that support it)

---

## 4. Generation Protocol

### Per Model

```python
for prompt in prompts:              # 20 prompts
    for seed in range(5):           # 5 seeds per prompt
        generate(
            prompt=prompt,
            seed=seed,
            duration=2.0,           # 2 seconds max for one-shots
            model=model_name,
            params=default_params   # Standard settings per model
        )
        save_as(f"{model}/{category}_{prompt_id}_seed{seed}.wav")
```

### Per-Model Parameters

| Model | Params |
|-------|--------|
| ElevenLabs SFX | duration_seconds=2.0, prompt_influence=0.5 |
| Stable Audio Open | steps=100, cfg_scale=7, sampler=dpmpp-3m-sde, audio_end_in_s=2.0 |
| Stable Audio API | steps=100, cfg_scale=7, audio_end_in_s=2.0 |
| AudioLDM 2 | steps=200, guidance_scale=3.5, audio_length_in_s=2.0 |

---

## 5. File Naming System

```
{model}/{category}_{prompt_id}_seed{seed}_{timestamp}.wav

Examples:
elevenlabs/kick_01_seed0_20250115_103022.wav
stableaudio/kick_01_seed0_20250115_103105.wav
audioldm2/kick_01_seed0_20250115_103145.wav
stableaudioapi/kick_01_seed0_20250115_103222.wav

Directory structure:
bakeoff_001/
├── results.json           # All scores, model metadata
├── elevenlabs/
├── stableaudio/
├── audioldm2/
├── stableaudioapi/
├── references/            # Professional reference files
└── listening_test/        # Randomized pairs for blind test
```

---

## 6. Automated Metrics

### Objective Measurements

| Metric | Tool | What It Measures | Target |
|--------|------|-----------------|--------|
| FAD (Fréchet Audio Distance) | `audiocraft` FAD implementation | Distribution match to reference one-shots | Lower = better |
| CLAP Score | CLAP model (laion/clap) | Text-to-audio alignment | Higher = better |
| SNR | librosa | Signal quality, noise floor | >20dB |
| Peak clipping % | librosa | Distortion from clipping | <1% |
| Duration accuracy | librosa | Actual vs. requested duration | ±20% |
| RMS consistency | librosa | Volume consistency across generations | Std dev <3dB |
| Zero-crossing rate | librosa | Noise floor, silent output detection | >0.01 |
| Spectral centroid | librosa | Brightness match to prompt expectation | Variable |

### Scoring Formula

```python
def objective_score(sound, prompt, reference):
    scores = {}
    
    # FAD (lower is better, invert to 0-10 scale)
    fad = compute_fad(sound, reference_dataset)
    scores['fad'] = max(0, 10 - fad * 10)
    
    # CLAP alignment (higher is better, 0-1 → 0-10)
    clap = compute_clap_score(sound, prompt)
    scores['clap'] = clap * 10
    
    # Audio quality
    snr = compute_snr(sound)
    scores['snr'] = min(10, snr / 3)  # 30dB SNR = 10
    
    clipping = compute_clipping_percent(sound)
    scores['clipping'] = max(0, 10 - clipping * 100)  # 0% clipping = 10
    
    # Duration match
    dur_ratio = actual_duration(sound) / requested_duration
    scores['duration'] = max(0, 10 - abs(1 - dur_ratio) * 20)
    
    # Weighted composite (0-10)
    weights = {
        'fad': 0.30,
        'clap': 0.25,
        'snr': 0.15,
        'clipping': 0.15,
        'duration': 0.15
    }
    
    total = sum(scores[k] * weights[k] for k in weights)
    return total, scores
```

---

## 7. Listening Test Design

### Blind A/B Test

```
Format: Web-based listening test (self-hosted or Google Form)

Instructions:
"You will hear two sounds generated from the same text prompt.
Please answer the following questions for each pair."

Questions per pair:
  Q1: Which sound sounds more like the prompt description?
      [Sound A] [Sound B] [No difference]
  
  Q2: Which sound is higher quality (cleaner, less artifacts)?
      [Sound A] [Sound B] [No difference]
  
  Q3: Which sound would you use in a track?
      [Sound A] [Sound B] [Neither] [Both]
  
  Q4: Rate Sound A — "This sounds like a usable one-shot"
      [1-5 scale, 1=completely unusable, 5=professional quality]
  
  Q5: Rate Sound B — same scale
      [1-5 scale]
```

### Pair Generation

```python
# Each listener hears 20 pairs, balanced:
pair_types = [
    ("elevenlabs", "stableaudio", 5),    # 5 pairs
    ("elevenlabs", "audioldm2", 5),       # 5 pairs
    ("elevenlabs", "stableaudioapi", 5),  # 5 pairs
    ("stableaudio", "audioldm2", 3),      # 3 pairs
    ("stableaudio", "stableaudioapi", 2), # 2 pairs
]
# Total: 20 pairs per listener
# Each pair: same prompt, different model, same seed
# A/B order randomized per listener
```

### Listener Requirements

```
- Minimum 10 listeners
- Must have music production or sound design experience
- Must use studio monitors or professional headphones
- No time limit but typical session: 15-20 minutes
- Calibration track played before test starts (reference kick)
```

---

## 8. Results Table Template

```
┌─────────────────────┬──────────┬──────────┬──────────┬──────────┬──────────┐
│ Metric              │ Eleven   │ StAud    │ StAudAP  │ AuLDM2   │ Winner   │
├─────────────────────┼──────────┼──────────┼──────────┼──────────┼──────────┤
│ Objective Score     │ 8.2      │ 7.1      │ 7.4      │ 6.0      │ Eleven   │
│   FAD (lower=better)│ 0.31     │ 0.52     │ 0.48     │ 0.89     │ Eleven   │
│   CLAP Score        │ 0.78     │ 0.71     │ 0.73     │ 0.62     │ Eleven   │
│   SNR (dB)          │ 28.3     │ 24.1     │ 25.0     │ 19.2     │ Eleven   │
│   Clipping %        │ 0.0%     │ 0.5%     │ 0.2%     │ 2.1%     │ Eleven   │
│   Duration accuracy │ 92%      │ 78%      │ 81%      │ 65%      │ Eleven   │
├─────────────────────┼──────────┼──────────┼──────────┼──────────┼──────────┤
│ Listening Score     │ 8.5      │ 7.0      │ 7.2      │ 5.5      │ Eleven   │
│   Q1 (prompt match) │ 8.2      │ 7.0      │ 7.3      │ 5.8      │ Eleven   │
│   Q2 (quality)      │ 8.8      │ 7.2      │ 7.1      │ 5.2      │ Eleven   │
│   Q3 (use in track) │ 82% yes  │ 64% yes  │ 66% yes  │ 38% yes  │ Eleven   │
│   Q4-Q5 (1-5 avg)   │ 4.2      │ 3.5      │ 3.6      │ 2.8      │ Eleven   │
├─────────────────────┼──────────┼──────────┼──────────┼──────────┼──────────┤
│ Latency (avg, sec)  │ 3.2      │ 18.5     │ 7.8      │ 12.4     │ Eleven   │
│ Cost per 1000 gens  │ ~$100    │ ~$0      │ ~$50     │ ~$0      │ (varies) │
│ Local feasible?     │ No       │ Yes      │ No       │ Yes      │ AuLDM2   │
└─────────────────────┴──────────┴──────────┴──────────┴──────────┴──────────┘
```

---

## 9. Category-Specific Analysis

Break down results by sound category to understand model strengths:

```
Kick Drums (prompts 1-5):
  ElevenLabs: Best punch and sub-bass. Most mix-ready.
  Stable Audio: Good but sometimes too long/ambient.
  AudioLDM 2: Weak low-end. Often sounds like a thud.
  
Snares & Claps (prompts 6-10):
  ElevenLabs: Best transient detail. Crack and snap are clear.
  Stable Audio: Good but sometimes washed out.
  AudioLDM 2: Often misses the transient entirely.
  
Hi-Hats & Percussion (prompts 11-15):
  ElevenLabs: Clean, bright, realistic.
  Stable Audio: Good but can be muffled.
  AudioLDM 2: Decent, simpler sounds work better.
  
FX & Tonal (prompts 16-20):
  ElevenLabs: Excellent risers, impacts, cinematic hits.
  Stable Audio: Good for ambient/texture FX.
  AudioLDM 2: Environmental sounds are its strength.
```

---

## 10. Decision Gate

After the bakeoff, the experiment produces a clear recommendation:

```
If ElevenLabs > 8.0 and cost acceptable:
  → Use ElevenLabs for prototype + MVP
  → Self-host Stable Audio Open as fallback
  → Defer custom model training

If Stable Audio Open > 8.0 (via fine-tuning):
  → Invest in self-hosting infrastructure
  → Build ONNX Runtime pipeline
  → Fine-tune on one-shot dataset

If AudioLDM 2 > 7.5:
  → Use AudioLDM 2 (MIT license advantage)
  → Build upsampling pipeline (16kHz → 44.1kHz)
  → Fine-tune for one-shot quality

If all models < 6.0:
  → Reconsider the product premise
  → Wait for better models
  → Focus on DSP-based generation + reference manipulation
```

### Expected Outcome

Based on current (2025) landscape, the expected winner is **ElevenLabs Text-to-Sound-Effects**. It was built specifically for this use case, while the other models are general-purpose audio generators. The experiment validates or disproves this assumption.

---

## 11. Experiment Automation Script

```python
#!/usr/bin/env python3
"""
cShot Model Bakeoff Runner

Usage:
  python bakeoff.py                          # Run all models
  python bakeoff.py --models elevenlabs       # Run one model
  python bakeoff.py --dry-run                # Print prompts, don't generate
  python bakeoff.py --eval-only              # Only run evaluation
"""

import argparse
import json
import subprocess
from pathlib import Path
from datetime import datetime

BAKEOFF_DIR = Path("bakeoff_001")
PROMPTS = [...]  # 20 prompts from §2
SEEDS = [42, 123, 456, 789, 1024]
CATEGORIES = ["kick", "snare", "hat", "perc", "fx"]

MODELS = {
    "elevenlabs": {
        "script": "run_elevenlabs.py",
        "params": {"duration_seconds": 2.0, "prompt_influence": 0.5}
    },
    "stableaudio": {
        "script": "run_stableaudio_open.py",
        "params": {"steps": 100, "cfg_scale": 7, "audio_end_in_s": 2.0}
    },
    "audioldm2": {
        "script": "run_audioldm2.py",
        "params": {"steps": 200, "guidance_scale": 3.5, "audio_length_in_s": 2.0}
    },
    "stableaudioapi": {
        "script": "run_stableaudio_api.py",
        "params": {"steps": 100, "cfg_scale": 7, "audio_end_in_s": 2.0}
    }
}

def generate(model_name, prompt_id, prompt, seed, params):
    model_dir = BAKEOFF_DIR / model_name
    model_dir.mkdir(parents=True, exist_ok=True)
    
    filename = f"{CATEGORIES[prompt_id // 5]}_{prompt_id:02d}_seed{seed}.wav"
    filepath = model_dir / filename
    
    # Call model-specific generation script
    result = subprocess.run([
        "python", MODELS[model_name]["script"],
        "--prompt", prompt,
        "--seed", str(seed),
        "--output", str(filepath),
        "--params", json.dumps(params)
    ], capture_output=True, text=True)
    
    return {
        "model": model_name,
        "prompt_id": prompt_id,
        "seed": seed,
        "file": str(filepath),
        "success": result.returncode == 0,
        "duration_ms": parse_duration(result.stdout),
        "error": result.stderr if result.returncode != 0 else None
    }

def evaluate():
    """Run all evaluation metrics"""
    results = []
    for model_name in MODELS:
        model_dir = BAKEOFF_DIR / model_name
        for wav in model_dir.glob("*.wav"):
            scores = compute_metrics(wav)
            results.append(scores)
    return results

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--models", nargs="+", choices=list(MODELS.keys()))
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--eval-only", action="store_true")
    args = parser.parse_args()
    
    models = args.models or list(MODELS.keys())
    
    if not args.eval_only:
        generation_log = []
        for model_name in models:
            print(f"\n=== Model: {model_name} ===")
            for prompt_id, prompt in enumerate(PROMPTS):
                for seed in SEEDS:
                    if args.dry_run:
                        print(f"  Would generate: {model_name} / prompt_{prompt_id} / seed_{seed}")
                        continue
                    result = generate(model_name, prompt_id, prompt, seed, MODELS[model_name]["params"])
                    generation_log.append(result)
                    print(f"  [{model_name}] prompt_{prompt_id} seed_{seed}: {'OK' if result['success'] else 'FAIL'} ({result.get('duration_ms', 0)}ms)")
        
        if not args.dry_run:
            with open(BAKEOFF_DIR / "generation_log.json", "w") as f:
                json.dump(generation_log, f, indent=2)
    
    if not args.dry_run:
        print("\n=== Evaluating ===")
        results = evaluate()
        with open(BAKEOFF_DIR / "results.json", "w") as f:
            json.dump(results, f, indent=2)
        
        # Generate summary
        summary = aggregate_results(results)
        print("\n=== Summary ===")
        for model, scores in sorted(summary.items()):
            print(f"  {model}: {scores['composite']:.2f}/10")

if __name__ == "__main__":
    main()
```

---

## 12. Deliverables

After the bakeoff, produce:

1. `results.json` — Full numeric results per sound
2. `summary.md` — Ranked model scores and winner
3. `listening_test_results.md` — Blind test analysis
4. `recommendation.md` — Which model to build around and why
5. `category_analysis.md` — Model strengths per sound type
6. `generation_log.json` — Full generation record with timings

The bakeoff tells cShot what to build on. Don't guess — measure.
