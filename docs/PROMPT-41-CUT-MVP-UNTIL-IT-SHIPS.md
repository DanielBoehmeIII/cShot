# Prompt 41 — Cut the MVP Until It Ships

Take cShot's existing MVP spec (Prompts 39-40) and aggressively reduce it to the absolute minimum that still proves the core loop.

---

## 1. The Core Loop (Everything Exists to Serve This)

```
reference audio → prompt → generated one-shot → preview → save/export
```

If a feature doesn't directly touch this loop, it gets cut. If it touches it indirectly, it gets cut. If it's nice-to-have, it gets cut.

---

## 2. What Stays

| Item | Why It Stays | Minimum Viable Form |
|------|-------------|---------------------|
| Prompt input | Entry point of the loop | Single text input, Enter to submit, no history, no autocomplete |
| Generation trigger | The entire point | One button. Click → wait → get sound. No config, no options |
| One result slot | Proves generation works | Single slot, not 6. Generate replaces it. No grid |
| Waveform thumbnail | Shows something happened | 80-point SVG rendered from float array. Pre-computed on backend |
| Instant preview | Core feedback mechanism | Click waveform → play. Web Audio API. Single sound in memory |
| WAV export | Gets sound into DAW | One button → saves to ~/Desktop/{name}.wav. No dialog, no options |
| Auto-tagging (basic) | Makes sound feel real | Sound type (kick/snare/hat/other) + 2-3 analysis tags |
| Favorites (basic) | Gives feeling of ownership | Heart toggle. Persisted in flat JSON file. No library view |
| Reference upload | Differentiator from text-only | Drag a WAV onto the prompt bar. One file. No analysis shown |

---

## 3. What Gets Cut

| Feature | Existing Reference | Why Cut |
|---------|-------------------|---------|
| 6-slot sound grid | Prompt 39 §3, Day 9 | Single slot is enough to prove generation. Grid is nice but adds layout state complexity |
| "More like this" variants | Prompt 39 §3, Day 12 | Core loop doesn't need iteration on iteration. Generate → judge → export is enough |
| Library browser | Prompt 39, Day 19 | Library of one session isn't meaningful. Favorites live in a JSON file |
| Search/filter | Prompt 39, Day 19 | Nothing to search. Cut entirely |
| Settings page | Prompt 39, Day 24 | Defaults are fine. No settings needed |
| Onboarding overlay | Prompt 39, Day 26 | The app has one input and one button. It's self-evident |
| Keyboard shortcuts | Prompt 39, Day 13 | Cut. Unnecessary complexity. Mouse-only is fine for prototype |
| Error boundaries | Prompt 39, Day 22 | Basic error handling only. No ErrorBoundary components |
| Status bar | Prompt 39, Day 14 | Cut. Unnecessary chrome |
| Detail panel | Prompt 39, Day 14 | Cut. Tags and metadata on the single slot card |
| Tag editor | Prompt 39, Day 18 | Auto-tags only. No user tag editing |
| Export history | Prompt 39, Day 11 | Cut. Export happens, you get a file. That's it |
| Dark/light theme | Prompt 39, Day 25 | Dark only. One theme |
| Database (SQLite) | Prompt 39, Day 15 | Replace with flat JSON file. Favorites and tags persist, but don't need a DB |
| Content-addressed storage | Prompt 39, §8 | Cut. Save to a flat directory with UUID filenames |
| Audio cache | Prompt 39, §8 | Cut. Single sound in memory. No need for LRU |
| Generation queue | Prompt 39, §9 | Cut. Single generation, blocking UI (with spinner) |
| Progress events | Prompt 39, §9 | Cut. Single spinner. Don't need stage-by-stage progress |
| Sound type classifier (ML) | Prompt 39, Day 6 | Rule-based. Spectral centroid + zero-crossing rate + duration thresholds |
| BPM/key detection | Prompt 39, Day 20 | Cut. Not needed for prototype |
| Batch export | Prompt 39, §5 | Cut. Single export only |
| Drag-to-export | Prompt 39, §11 | Cut. Button-based export only |
| Auto-update | Prompt 39, Day 27 | Cut. Manual download for prototype |
| Code signing | Prompt 39, Day 27 | Cut. Not shipping to general public |
| Multi-platform build | Prompt 39, Day 27 | Ship on one platform (developer's primary OS) |
| Help page | Prompt 39, Day 26 | Cut. No help needed for 3-step app |
| Demo video | Prompt 39, Day 29 | Cut. Show a friend in person |

---

## 4. What Can Be Faked

| Fake | What It Simulates | How to Fake It |
|------|-------------------|----------------|
| Model inference | Real AI generation | Pre-generate 20 WAV files from different prompts. Pick the closest match based on keyword overlap. Run real DSP (pitch shift, filter, time-stretch) to make it feel unique per prompt |
| Text embedding | Understanding the prompt | Keyword-to-config mapping: "kick" → low shelf boost + short decay envelope, "snare" → mid-band noise burst, "bright" → high shelf, "dark" → low-pass filter |
| Sound type classification | ML classifier | Derive from the keyword mapping. If prompt says "kick", type = kick. If ambiguous, measure spectral centroid |
| Generation progress | Real-time progress bar | Fake it. After click, animate a progress ring for 2-3 seconds, then reveal the sound |
| "AI is generating" | Model is running | Show spinning indicator. The DSP pipeline actually runs during this time |
| Waveform rendering | Live analysis | Pre-compute waveform data for pre-generated files. For DSP output, compute on the fly (fast enough without GPU) |

---

## 5. What Must Be Real

| Real Component | Why It Must Be Real | Minimum Viable Implementation |
|----------------|---------------------|------------------------------|
| Audio file upload | User provides their own audio | Tauri file picker (wav only). Copy file to app directory. Decode to f32 buffer |
| DSP transformation | Proves cShot can process audio | Trim silence, normalize peak, apply envelopes. Rust with hound crate |
| WAV export | User gets a usable file | hound crate writes 44.1kHz/24-bit/mono WAV to user-specified location |
| Instant preview | User hears the result immediately | Web Audio API playback. Float32 array sent from Rust via IPC |
| Prompt parsing | Connects text to sound generation | Simple keyword → parameters mapping (50 keywords, lookup table) |
| Favorites persistence | User keeps their work | Flat JSON file (~/cShot/favorites.json). Read on load, write on toggle |

---

## 6. What Validates the Idea

| Validation Question | How the Prototype Answers It | Success Signal |
|---------------------|------------------------------|----------------|
| "Can I get a usable sound from a text prompt?" | User types → gets a sound → plays it → exports it | Sound is recognizable as the requested type. User exports it and drags into DAW |
| "Is this faster than searching Splice?" | Time from prompt to preview is measured | Under 10 seconds from hitting Enter to hearing audio |
| "Does the sound fit in a mix?" | User drops exported WAV into their DAW alongside their track | Sound sits in the mix without sounding obviously synthetic or broken |
| "Would I use this in a real project?" | User's own reaction | User favs the sound, exports it, continues working with it later |
| "Is the reference audio making it better?" | User uploads a reference clip, generates a related sound | Generated sound is clearly related to the reference (same ballpark timbre, pitch, energy) |
| "Do I want to do this again?" | User generates more than once | Return rate: user generates 5+ times in a session |

---

## 7. The Cut-Down Technical Spec

### Frontend

```
Stack: React + TypeScript + Vite + Tauri v2
Pages: 1 (single page, no routing)
Components:
  - PromptBar (text input + upload button + generate button)
  - SoundCard (waveform thumbnail + type badge + play button + fav button + export button)
  - LoadingSpinner (while generating)
State: React useState + useEffect (no Zustand, no store library)
Audio: Web Audio API, single AudioContext, one buffer at a time
```

### Backend (Rust via Tauri)

```
Commands:
  - generate_sound(prompt: String, reference_path: Option<String>) → SoundResult
  - get_audio_data(sound_id: String) → Vec<f32>
  - export_wav(sound_id: String, path: String) → ExportResult
  - toggle_favorite(sound_id: String) → bool
  - get_favorites() → Vec<SoundMetadata>
  - upload_reference() → String (returns file path)
```

### Audio Processing (Rust)

```
Pipeline:
  1. Parse prompt → keyword map (simple lookup)
  2. If reference provided: analyze reference (duration, RMS, spectral centroid)
  3. Load nearest pre-generated WAV (or apply DSP to reference)
  4. Apply prompt-based DSP: EQ shelves, envelope shaping, pitch shift if needed
  5. Trim silence, normalize peak to -1dBFS
  6. Save to ~/cShot/audio/{uuid}.wav
  7. Return metadata: id, type, duration, waveform_data
```

### Persistence

```
File: ~/cShot/favorites.json
Format:
{
  "favorites": ["uuid1", "uuid2"],
  "sounds": {
    "uuid1": {
      "prompt": "punchy kick",
      "type": "kick",
      "duration_ms": 423,
      "created_at": "2025-01-15T10:30:00Z"
    }
  }
}
```

### File Structure

```
~/cShot/
├── audio/          # WAV files, flat directory
│   ├── {uuid}.wav
│   └── ...
├── favorites.json  # Persisted favorites
└── references/     # Uploaded reference audio
    └── {uuid}.wav
```

---

## 8. Comparison: Before vs. After

| Dimension | Prompt 39 MVP | This Cut |
|-----------|---------------|----------|
| Components | 25+ | 3 |
| Rust modules | 15+ | 4 |
| Frontend pages | 4 | 1 |
| Database | SQLite with migrations | Single JSON file |
| State management | Zustand stores (4) | React useState |
| Generation slots | 6 | 1 |
| Model inference | ONNX Runtime, CUDA, GPU | Pre-generated WAVs + DSP |
| Settings | Full settings page | Zero settings |
| Onboarding | Overlay + tips + help | Nothing |
| Export options | Dialog with format choices | Save to ~/Desktop |
| Build targets | macOS + Windows + Linux | Developer's OS only |
| **Estimated build time** | 30 days | **3 days** |

---

## 9. Summary

The aggressively cut MVP is a single-screen desktop app where you:

1. Type "punchy kick 140" or drag in a reference kick
2. Wait 2-3 seconds (fake progress spinner)
3. Hear a sound, see a waveform
4. Click heart to save, click export to get a WAV
5. Open it in your DAW
6. Decide: "Is this useful?"

Everything else — the grid, the library, the settings, the model — is validation-dependent. Build the 3-day prototype. Put it in front of one producer. If they smile, keep going. If they don't, the idea needs to change before you invest more.

The prototype is not the product. It's a question: *"Would you use this?"*
