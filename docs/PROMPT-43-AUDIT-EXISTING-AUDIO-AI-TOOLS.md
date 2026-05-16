# Prompt 43 — Audit Existing Audio AI Tools

Comprehensive audit of current tools, APIs, models, and libraries for generating short audio one-shots.

---

## 1. Models (Open Source, Self-Hostable)

### Stable Audio Open 1.0 (Stability AI)

| Dimension | Assessment |
|-----------|------------|
| Quality | 8/10 — Generates clean stereo audio at 44.1kHz. Best for sound effects, field recordings, percussion. Weak on realistic vocals. |
| Latency | ~10-30s on A100, ~60-120s on consumer GPU. Too slow for real-time but acceptable for generation-on-demand |
| Controllability | 7/10 — Text prompt + negative prompt + duration (up to 47s). CFG scale control. No fine-grained timbre control |
| Licensing | Stable Audio Community License. Commercial use requires separate license from Stability AI. Not truly free for commercial products |
| Format | Stereo 44.1kHz, up to 47s |
| Architecture | Latent diffusion (DiT) with T5 text encoder. 3 components: autoencoder + T5 + transformer diffusion |
| Local feasibility | High — can run on 24GB VRAM (FP16). CPU inference possible but very slow |
| One-shot usefulness | 7/10 — Generates good individual hits. Tends toward longer samples; needs post-processing to trim to one-shot length |
| Python package | `stable-audio-tools` + `diffusers` |

### AudioLDM 2 (Surrey/Microsoft)

| Dimension | Assessment |
|-----------|------------|
| Quality | 7/10 — Good for sound effects and environmental sounds. Slightly less clean than Stable Audio for music elements |
| Latency | ~5-15s on A100, ~30-60s on consumer GPU. Slightly faster than Stable Audio |
| Controllability | 7/10 — Text prompt + negative prompt + inference steps + guidance scale. Audio length in seconds |
| Licensing | MIT License (cvssp/audioldm2 on HuggingFace). Truly free for commercial use |
| Format | Mono 16kHz (base model). Upsampling available |
| Architecture | LDM with CLAP + Flan-T5 text encoders + GPT2 language model + UNet |
| Local feasibility | High — smaller model size (~1.1B params). Runs on 16GB VRAM |
| One-shot usefulness | 6/10 — Output is 16kHz mono, needs upsampling for professional use. Better for ambient/atmosphere than punchy one-shots |
| Python package | `diffusers` (built-in pipeline) |

### AudioCraft / MusicGen (Meta)

| Dimension | Assessment |
|-----------|------------|
| Quality | 8/10 — MusicGen is excellent for melodic content. AudioCraft is the framework; includes MusicGen, AudioGen, EnCodec |
| Latency | ~8-20s on A100 for MusicGen. EnCodec compression is very fast |
| Controllability | 6/10 — Text prompt + melody conditioning (MusicGen). Less controllable for specific one-shot timbres |
| Licensing | CC-BY-NC 4.0 (non-commercial). MIT for EnCodec. Commercial use requires Meta license |
| Format | Mono 32kHz (MusicGen). Stereo in development |
| Architecture | EnCodec tokenizer + autoregressive transformer (LM) + codec decoder |
| Local feasibility | Medium — MusicGen requires ~16GB VRAM. EnCodec is very lightweight |
| One-shot usefulness | 5/10 — Designed for longer musical passages, not short one-shots. Can be adapted but not ideal |
| Python package | `audiocraft` |

### Bark (Suno)

| Dimension | Assessment |
|-----------|------------|
| Quality | 6/10 — Designed for text-to-speech but can generate non-speech sounds |
| Latency | ~10-30s on GPU. Slow |
| Controllability | 4/10 — Primarily speech. Non-speech generation is unpredictable |
| Licensing | MIT License |
| Format | Mono 24kHz |
| Architecture | GPT-style transformer + EnCodec |
| Local feasibility | High — small model |
| One-shot usefulness | 2/10 — Not suitable for one-shot sound generation |
| Python package | `bark` |

### Riffusion

| Dimension | Assessment |
|-----------|------------|
| Quality | 5/10 — Spectrogram-based, quality is below diffusion models. Novel approach but not production-ready |
| Latency | ~5-10s on GPU |
| Controllability | 5/10 — Text prompt, relatively limited |
| Licensing | MIT License |
| Format | Spectrogram → audio conversion |
| Architecture | Fine-tuned Stable Diffusion on spectrograms |
| Local feasibility | High — runs on modest GPU |
| One-shot usefulness | 4/10 — Interesting for experimentation, not reliable enough for cShot |
| Python package | `riffusion` |

---

## 2. APIs (Cloud, Pay-per-use)

### ElevenLabs Text-to-Sound-Effects

| Dimension | Assessment |
|-----------|------------|
| Quality | 9/10 — Best-in-class sound effect generation. Clean, punchy, mix-ready. Surprisingly good for one-shots |
| Latency | ~2-5s per generation. Fast |
| Controllability | 7/10 — Text prompt + duration + negative prompt. Limited parameter control |
| Pricing | Pay-as-you-go. ~$0.10/generation at standard tier. $11/mo for 100k characters (generous for one-shots) |
| Licensing | Generated sounds owned by user. Terms allow commercial use |
| API access | REST API. Simple POST request. Returns audio buffer |
| Local feasibility | Not possible — cloud-only |
| One-shot usefulness | 9/10 — Generates short, punchy, mix-ready sounds. Best option for cShot prototype |
| Notes | Newer product, rapidly improving. Best quality-per-dollar for one-shots |

### Stability AI API (Stable Audio)

| Dimension | Assessment |
|-----------|------------|
| Quality | 8/10 — Same model as open-source but on better hardware. Clean stereo output |
| Latency | ~5-10s. Slower than ElevenLabs |
| Controllability | 7/10 — Text prompt + duration + CFG scale |
| Pricing | Credit-based. ~$0.05-0.10/generation depending on duration |
| Licensing | Standard API terms. Generated content rights to user |
| API access | REST API |
| Local feasibility | Also available as open-source model (above) |
| One-shot usefulness | 7/10 — Good quality but tends toward longer samples |
| Notes | Can self-host the open model for no API costs (if you have GPU) |

### Hugging Face Inference API

| Dimension | Assessment |
|-----------|------------|
| Quality | Varies by model (7-8/10) |
| Latency | ~10-30s. Queue-based, unpredictable |
| Controllability | Varies by model |
| Pricing | $0.09/hr for serverless. Dedicated endpoints from $0.50/hr |
| Licensing | Varies by model |
| API access | REST API, WebSocket for streaming |
| Local feasibility | N/A — but most models can be self-hosted |
| One-shot usefulness | 6/10 — Good for experimentation, not reliable enough for production |
| Notes | Best for model comparison/testing, not for product integration |

### Google MediaPipe Audio (Edge)

| Dimension | Assessment |
|-----------|------------|
| Quality | 4/10 — Audio classification and processing, not generation |
| Latency | Real-time |
| Controllability | N/A |
| Pricing | Free |
| Licensing | Apache 2.0 |
| API access | Python, JS, Android, iOS SDKs |
| Local feasibility | Yes — designed for on-device |
| One-shot usefulness | 1/10 — Not a generation tool |
| Notes | Useful for audio analysis features (classification, detection) |

---

## 3. Libraries (Non-AI, DSP)

### librosa

| Dimension | Value |
|-----------|-------|
| Purpose | Audio analysis and feature extraction |
| Quality | Industry standard |
| Language | Python |
| Licensing | ISC License |
| One-shot usefulness | 9/10 — Spectral features, onset detection, pitch, chroma. Essential for analysis |
| Recommendation | Use for model comparison experiments, auto-tagging feature extraction |

### torchaudio

| Dimension | Value |
|-----------|-------|
| Purpose | Audio I/O and transforms for PyTorch |
| Quality | Industry standard |
| Language | Python |
| Licensing | BSD |
| One-shot usefulness | 8/10 — Integration with PyTorch models, audio transforms, resampling |
| Recommendation | Required for any PyTorch-based audio model work |

### pedalboard (Spotify)

| Dimension | Value |
|-----------|-------|
| Purpose | Audio effects (EQ, compression, reverb, etc.) |
| Quality | 9/10 — Professional quality, same engine as Spotify's desktop app |
| Language | Python (C++ under the hood) |
| Licensing | GPL-3.0 |
| One-shot usefulness | 10/10 — Apply EQ, compression, reverb to generated one-shots. Essential for mix-readiness |
| Recommendation | Use for post-processing polish pipeline |

### pysox

| Dimension | Value |
|-----------|-------|
| Purpose | Audio file conversion and basic processing |
| Quality | 8/10 — Wraps SoX, battle-tested |
| Language | Python |
| Licensing | LGPL-3.0 |
| One-shot usefulness | 7/10 — Trim, convert, resample, concatenate |
| Recommendation | Alternative to custom Rust processing for Python prototype |

### hound (Rust) / symphonia (Rust)

| Dimension | Value |
|-----------|-------|
| Purpose | WAV read/write (hound) / audio decoding (symphonia) |
| Quality | 9/10 — Pure Rust, no system dependencies |
| Language | Rust |
| Licensing | Apache 2.0 / MIT |
| One-shot usefulness | 10/10 — Required for cShot's Rust backend |
| Recommendation | Already selected for cShot stack |

---

## 4. Comparison Matrix

| Tool | Quality | Latency | Cost | License | Local | One-Shot Fit | cShot Phase |
|------|---------|---------|------|---------|-------|-------------|-------------|
| **ElevenLabs SFX** | 9/10 | 2-5s | $0.10/gen | Commercial OK | No | 9/10 | Prototype |
| **Stable Audio Open** | 8/10 | 10-30s | Free (self) | Community | Yes (24GB) | 7/10 | MVP |
| **Stable Audio API** | 8/10 | 5-10s | $0.05-0.10 | Commercial OK | No | 7/10 | MVP |
| **AudioLDM 2** | 7/10 | 5-15s | Free | MIT | Yes (16GB) | 6/10 | Research |
| **MusicGen** | 8/10 | 8-20s | Free | CC-BY-NC | Yes (16GB) | 5/10 | Not for one-shots |
| **Bark** | 6/10 | 10-30s | Free | MIT | Yes (8GB) | 2/10 | Not suitable |
| **librosa** | 10/10 (analysis) | N/A | Free | ISC | Yes | 9/10 | Analysis pipeline |
| **pedalboard** | 9/10 (FX) | Real-time | Free | GPL-3.0 | Yes | 10/10 | Post-processing |

---

## 5. Recommended Test Order for cShot

### Phase 1: Prototype (Weeks 21-25)

```
1. ElevenLabs Text-to-Sound-Effects API
   Why: Best quality, fastest, most reliable for one-shots.
   Cost: ~$10 for first 100 generations. Acceptable for prototype.
   Integration: Simple REST API call from Rust backend via reqwest.
   Risk: API dependency, ongoing cost. Acceptable for validation.
```

### Phase 2: MVP (Post Prototype)

```
2. Stable Audio Open 1.0 (self-hosted)
   Why: Free per-generation, open model, can fine-tune.
   Hardware: RTX 3090/4090 (24GB VRAM) minimum.
   Integration: ONNX Runtime or direct Python subprocess.
   Risk: GPU requirement limits user base.

3. AudioLDM 2 as fallback
   Why: MIT license, smaller, runs on more hardware.
   Integration: ONNX Runtime via Python→Rust bridge.
   Risk: Lower quality, 16kHz output needs upsampling.
```

### Phase 3: Production

```
4. Fine-tuned custom model
   Why: Optimized for one-shot generation specifically.
   Data: CC0 one-shot dataset + user feedback.
   Architecture: Small diffusion model (50-200M params).
   Deployment: ONNX Runtime in Tauri.

5. Hybrid: Cloud API + local fallback
   Why: Best quality on any hardware.
   Architecture: Try cloud first, fall back to local model.
```

---

## 6. Tool-Specific Integration Notes

### ElevenLabs Integration (Prototype)

```
POST https://api.elevenlabs.io/v1/sound-effects/convert
Headers:
  - xi-api-key: {key}
  - Content-Type: application/json

Body:
{
  "text": "punchy trap kick drum 140bpm",
  "duration_seconds": 1.0,
  "prompt_influence": 0.5
}

Response: Binary audio data (WAV/MP3)
Time: ~2-5 seconds
Cost: ~$0.10 per generation
```

### Stable Audio Open Integration (Self-Hosted)

```python
# Called from Rust via subprocess or Python bridge
import torch
from diffusers import StableAudioPipeline

pipe = StableAudioPipeline.from_pretrained(
    "stabilityai/stable-audio-open-1.0",
    torch_dtype=torch.float16
)
pipe = pipe.to("cuda")

audio = pipe(
    prompt="short punchy kick drum",
    num_inference_steps=100,
    audio_end_in_s=2.0,
    num_waveforms_per_prompt=3,
).audios[0]

# audio is stereo Float32 tensor at 44.1kHz
# Convert to mono, trim, normalize, save as WAV
```

---

## 7. Summary: What cShot Should Use

| Need | Tool | Why |
|------|------|-----|
| **Prototype generation** | ElevenLabs SFX API | Best quality, fastest, simple REST API. $10 to validate the idea |
| **Self-hosted fallback** | Stable Audio Open 1.0 | Free, high quality, open model. Good path to production |
| **Analysis/features** | librosa (Python) | All the audio features you need for auto-tagging |
| **Post-processing FX** | pedalboard (Python) | Professional quality EQ, compression for mix-readiness |
| **WAV I/O (Rust)** | hound crate | Already in stack. Simple, fast, pure Rust |
| **Experimental/model testing** | AudioLDM 2 | MIT license, good for comparing against ElevenLabs |
| **Not for cShot** | MusicGen, Bark, Riffusion | Wrong use case — designed for music/speech, not one-shots |

### Final Recommendation

**Start with ElevenLabs Text-to-Sound-Effects API for the prototype.** It requires zero ML infrastructure, produces the best-quality one-shots, costs ~$10 for validation, and the REST interface means you can integrate it in a day. If the prototype validates, invest in self-hosting Stable Audio Open or fine-tuning a custom model.

The risk is not the API cost. The risk is building ML infrastructure before validating that anyone wants AI-generated one-shots.
