# Prompt 99 — cShot v1 Product Spec

## Product Overview

### Product Thesis

cShot is a desktop application that generates unique, production-ready one-shot audio samples from natural language text prompts. It replaces the traditional sample library browsing workflow (Splice, Loopcloud, sample packs) with instant AI generation, eliminating the 30-60% of music production time spent browsing samples.

### Category Definition

**Semantic Sound Creation Platform** — a new category that combines:
- Natural language understanding (describe sounds semantically)
- Generative audio AI (create unique samples on demand)
- Producer workflow tools (organize, export, use in DAW)

Not a "sample generator" — those imply finite, pre-made outputs. Not an "AI music tool" — those generate finished songs. cShot generates raw materials: the atomic sounds that producers assemble into music.

---

### Target Users

| Persona | Role | Age | Skill | Key Job | Priority |
|---|---|---|---|---|---|
| Alex | Beatmaker / Hip-hop producer | 18-35 | Intermediate | Get a good kick/snare/808 in <10 seconds | Primary |
| Jordan | Sound designer (game/post) | 25-45 | Expert | Generate 50 variations of an impact with precise control | Secondary |
| Sam | Hobbyist producer | 14-25 | Beginner | Make cool sounds without knowing sound design | Tertiary |

**Primary persona deeper dive — Alex:**
- Uses FL Studio or Ableton Live
- Spends $200-500/year on sample packs (Splice, Loopmasters)
- Has 50GB+ of samples, still can't find the right kick
- Makes beats 3-5 nights a week
- Shares beats on YouTube, SoundCloud, BeatStars
- Willing to pay $15-30/month for tools that save time
- Frustrated by: browsing, folder management, "close enough" samples

---

### Main Jobs-to-Be-Done

1. **"Get the right kick for this track RIGHT NOW"** — Generate a kick that fits the current beat tonally, rhythmically, and energetically
2. **"Explore variations of a sound I like"** — Take a reference sound and produce 6+ meaningful variations
3. **"Build a custom drum kit for a project"** — Generate multiple one-shots (kick, snare, hat, 808) that work together
4. **"Export a sound and drop it into my DAW"** — Get a production-ready WAV file in the right format, at the right path
5. **"Find that sound I made last week"** — Locate a previous generation by prompt, tag, or similarity
6. **"Describe what I want and hear it"** — Express a sound concept in natural language and hear the result
7. **"Make a unique sound that no one else has"** — Generate a truly original one-shot for signature sound identity

---

### Core Workflows

#### Workflow 1: Single Generation (Primary)
1. Open cShot
2. Type prompt in prompt bar (e.g., "punchy 808 kick, sub-bass tail, tuned to G")
3. Press Enter or click Generate
4. 6 sound slots fill with waveform thumbnails (4-10 seconds)
5. Click any slot to hear the sound (instant playback)
6. Click a slot to select → Detail panel opens with waveform, metrics, SoundScore
7. Click Export → Choose format → File saves to disk
8. Drop WAV into DAW. Done.

#### Workflow 2: Reference-Based Generation (Power User)
1. Drag a WAV file into the ReferenceDropZone
2. cShot analyzes: waveform, spectral profile, key parameters
3. Type prompt describing what to change: "Snappier attack, less sub, key of D"
4. 6 variations appear, modified from the reference
5. Select, export, use.

#### Workflow 3: Pack Creation
1. Generate a kick → save to pack "Night Run Kit"
2. Generate a snare → add to same pack
3. Generate a hi-hat → add to same pack
4. Generate an 808 → add to same pack
5. Pack view shows all sounds together
6. "Export All" or individual export
7. Pack is saved in library for reuse

#### Workflow 4: Library Browsing
1. Open Library view
2. Browse by: date, pack, tag, sound type, model
3. Search by prompt text (full-text search)
4. Click a sound → waveform preview, metadata, SoundScore
5. Re-export to different format
6. Delete, tag, rate, or add to pack

---

### Screens

#### Screen 1: Main Generation View (Home)

```
┌──────────────────────────────────────────────────────────────┐
│ [Logo] cShot                                    [Settings]   │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  [🔍] "punchy 808 kick, round sub-bass tail, tuned... │  │
│  │  [Generate]  [🎤 Upload Reference]                     │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ ┌──────┐ │  │ ┌──────┐ │  │ ┌──────┐ │  │ ┌──────┐ │   │
│  │ │ wave │ │  │ │ wave │ │  │ │ wave │ │  │ │ wave │ │   │
│  │ └──────┘ │  │ └──────┘ │  │ └──────┘ │  │ └──────┘ │   │
│  │ 87 score │  │ 82 score │  │ 91 score │  │ 78 score │   │
│  │ Kick #1  │  │ Kick #2  │  │ Kick #3  │  │ Kick #4  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│  ┌──────────┐  ┌──────────┐                                 │
│  │ ┌──────┐ │  │ ┌──────┐ │                                 │
│  │ │ wave │ │  │ │ wave │ │                                 │
│  │ └──────┘ │  │ └──────┘ │                                 │
│  │ 85 score │  │ 88 score │                                 │
│  │ Kick #5  │  │ Kick #6  │                                 │
│  └──────────┘  └──────────┘                                 │
│                                                              │
│  [Export Selected]  [Add to Pack]  [↻ Regenerate]           │
├──────────────────────────────────────────────────────────────┤
│ Status: Ready | Library: 142 sounds | v1.0.0                 │
└──────────────────────────────────────────────────────────────┘
```

**Elements:**
- Top bar: Logo, app title, settings gear icon, account menu
- Prompt bar: Text input with semantic autocomplete, microphone icon for future voice input
- Action buttons: Generate (primary CTA), Upload Reference (secondary)
- Sound grid: 2×3 grid of sound cards
  - Each card: Waveform thumbnail (SVG), SoundScore badge (colored), sound name/label
  - Click to play (waveform animates during playback)
  - Click to select (highlighted border)
  - Right-click: Export, Add to Pack, Delete, Rate, Tag
- Bottom action bar: Context-sensitive actions for selected sound
- Status bar: Generation status, library count, version

#### Screen 2: Detail Panel (Overlay / Side Panel)

```
┌──────────────────────────────────────────────────────────────┐
│ Back to Grid                                          [✕]   │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                                                        │  │
│  │              WAVEFORM (full width)                     │  │
│  │              [zoom controls]                           │  │
│  │                                                        │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              SPECTRAL DISPLAY                           │  │
│  │              (frequency over time)                      │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐                        │
│  │Punch │ │ Body │ │Clarity│ │Uniq. │                        │
│  │  92  │ │  78  │ │  88   │ │  85  │                        │
│  └──────┘ └──────┘ └──────┘ └──────┘                        │
│                     Overall: 86                               │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ Metadata                                                │  │
│  │ Duration: 0.42s  |  Sample Rate: 44.1kHz  |  24-bit    │  │
│  │ RMS: -8.4dB  |  Peak: -1.0dB  |  Crest: 14.2dB        │  │
│  │ Transient: 23ms  |  Spectral Centroid: 2.1kHz         │  │
│  │ Prompt: "punchy 808 kick..."                          │  │
│  │ Model: ElevenLabs SFX v2  |  Seed: 847291             │  │
│  │ Created: 2025-05-15 22:34:12  |  ID: 8a3f...          │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  [Export]  [↻ Regenerate]  [Add to Pack]  [Tag]  [Rate ★★★] │
└──────────────────────────────────────────────────────────────┘
```

#### Screen 3: Library View

```
┌──────────────────────────────────────────────────────────────┐
│ Library                                              [Grid]  │
├──────────────────────────────────────────────────────────────┤
│ [🔍 Search prompts, tags...]  [Filter by type] [Sort by]   │
│                                                              │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│ │ ┌────┐   │ │ ┌────┐   │ │ ┌────┐   │ │ ┌────┐   │        │
│ │ │wave│   │ │ │wave│   │ │ │wave│   │ │ │wave│   │        │
│ │ └────┘   │ │ └────┘   │ │ └────┘   │ │ └────┘   │        │
│ │ 92 score │ │ 87 score │ │ 79 score │ │ 90 score │        │
│ │ "punchy..│ │ "tight.. │ │ "round.. │ │ "aggr..  │        │
│ │ Pack: NRK│ │ Pack: NRK│ │ Pack: -- │ │ Pack: DRM│        │
│ │ May 15   │ │ May 14   │ │ May 14   │ │ May 13   │        │
│ └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
│ ... (pagination)                                             │
│                                                              │
│ Packs: Night Run Kit (4)  |  Dream Kit (3)  |  + New Pack   │
├──────────────────────────────────────────────────────────────┤
│ 142 sounds | 8 packs | 2.1GB used                            │
└──────────────────────────────────────────────────────────────┘
```

#### Screen 4: Export Dialog (Modal)

```
┌──────────────────────────────────────────────────────┐
│ Export Sound                                         │
├──────────────────────────────────────────────────────┤
│                                                      │
│ Format:  ○ WAV  ● AIFF  ○ FLAC  ○ MP3              │
│                                                      │
│ Bit Depth:  ○ 16-bit  ● 24-bit  ○ 32-bit float     │
│                                                      │
│ Sample Rate:  ○ 44.1kHz  ● 48kHz  ○ 96kHz          │
│                                                      │
│ Normalize:  ● Yes (peak -1.0dB)  ○ No               │
│                                                      │
│ Fade In:  ● 5ms  ○ None                             │
│ Fade Out:  ● 10ms  ○ None                           │
│                                                      │
│ Export to:  /Users/alex/Music/cShot/        [Browse] │
│                                                      │
│ Filename:  cShot_kick_G_punchy_808.wav              │
│                                                      │
│ [Preview]                         [Cancel] [Export]  │
└──────────────────────────────────────────────────────┘
```

---

### Audio Pipeline

```
User Prompt → Text Encoding → Model Inference → DSP Post-Processing → Preview + Export

1. TEXT ENCODING (local ONNX)
   - CLAP-style text encoder (768-dim embedding)
   - Runs on CPU, <100ms
   - Producer vocabulary optimized ("punchy", "subby", "crack", "body", "snap")

2. MODEL INFERENCE (cloud or local ONNX)
   - Cloud: ElevenLabs SFX API or Stable Audio Open API (4-8s)
   - Local: AudioLDM 2 fine-tuned, INT8 quantized, ONNX Runtime (10-30s)
   - Generates raw float32 audio at 44.1kHz

3. DSP POST-PROCESSING (Rust, always local)
   - Trim leading/trailing silence (< -60dB threshold, 100ms hold)
   - Normalize peak to -1.0dB (true peak, not sample peak)
   - Fade in 5ms (linear ramp)
   - Fade out 10ms (linear ramp)
   - Analysis: RMS, peak, crest factor, spectral centroid, transient onset time
   - SoundScore: ONNX quality model (punch, body, clarity, uniqueness, overall)

4. STORAGE
   - Content-addressed (SHA-256 of audio)
   - Write metadata to SQLite
   - Generate waveform thumbnail (SVG path data)
```

---

### Library System

**Storage location:** `~/.cshot/` (configurable)

**Organization:**
- All sounds stored as flat content-addressed files in `audio/` directory
- SQLite database (`metadata.db`) indexes everything
- Packs are logical groupings (many-to-many: one sound can be in multiple packs)

**Search capabilities:**
- Full-text search on prompt text
- Filter by: pack, model, date range, sound type (auto-classified), rating
- Sort by: date, SoundScore, duration, random
- Similarity search: select a sound → "Find similar" → FAISS vector search on UShOt embeddings

**Tags:**
- Free-form user tags
- Auto-tags from prompt analysis (e.g., "kick", "808", "punchy", "sub-bass")
- Filterable and searchable

---

### Export System

**Supported formats:**
| Format | Bit depths | Sample rates | Use case |
|---|---|---|---|
| WAV | 16, 24, 32-float | 44.1k, 48k, 96k | Default, universal DAW compatibility |
| AIFF | 16, 24 | 44.1k, 48k, 96k | Apple ecosystem (Logic Pro) |
| FLAC | 16, 24 | 44.1k, 48k, 96k | Lossless archival, smaller than WAV |
| MP3 | 320kbps CBR | 44.1k, 48k | Quick preview, sharing |

**Export options:**
- Normalize: Yes (default) / No
- Fade: On (default) / Off
- Include sidecar metadata JSON: Yes / No (default)
- Add to DAW browser folder: Optional

**Naming convention:**
`cShot_{type}_{key}_{descriptor}_{seed}.wav`
Example: `cShot_kick_G_punchy_808_a8f3.wav`

---

### Feedback Loop

**In-app feedback:**
- SoundScore is displayed automatically (quality assessment)
- Rating: 1-5 stars on any sound (persisted, searchable)
- Export tracking: when a sound is exported, that's positive signal
- Regeneration: when a user regenerates, something wasn't right — log the context

**Telemetry (opt-in):**
- Generation count, latency, model used (anonymized)
- Export rate (what percentage of generations are good enough to export)
- Regeneration rate (what percentage need a retry)
- Prompt length distribution
- Feature usage (% using reference upload, library search, etc.)

**Loop closure:**
- High-export sounds are candidates for model fine-tuning data
- Low-SoundScore generations are analyzed for failure patterns (clipping, silence, wrong type)
- Churned users are surveyed: "What was missing?"

---

### Onboarding

**First launch flow:**
1. Welcome screen: "Stop browsing. Start making."
2. Quick tutorial (3 screens, 15 seconds each):
   - "Type what you hear"
   - "Hear instant variations"
   - "Export to your DAW"
3. Sample prompt: pre-filled "Punchy 808 kick, round sub-bass tail, tuned to G"
4. One-click generate — immediately shows the value
5. After first export: "You just saved 45 minutes of browsing. Welcome to cShot."

**First 7 days as a guided experience:**
- Day 1: "Try generating 3 different kicks" → triggers reference workflow suggestion
- Day 2: "Try uploading a reference" → triggers variation workflow
- Day 3: "Create your first pack" → triggers pack creation flow
- Day 4: "Explore your library" → triggers library discovery
- Day 5: "Try a different model" → triggers model comparison
- Day 7: "Check your stats" → shows personal generation statistics

---

### Pricing Hypothesis

| Tier | Price | Limits | Features |
|---|---|---|---|
| Free | $0 | 30 generations/month, export watermark-free, no reference upload | Basic generation, WAV export |
| Producer | $15/mo | 500 generations/month, all formats, reference upload, SoundScore, packs | Core product |
| Pro | $30/mo | 3000 generations/month, local inference, batch export, priority support | Power users |
| Studio | $50/mo | Unlimited, commercial license, plugin access, cloud sync | Professionals |

**Annual plan:** 20% discount on all tiers

**Free tier rationale:** Low enough to try, high enough to want more. The 30-gen limit shows the core value without giving away infinite generation. Producers hit the limit in 1-2 sessions and upgrade.

---

### Success Metrics (v1)

| Metric | Target (Month 6) | Why |
|---|---|---|
| MAU | 10,000 | Validated product-market fit |
| Paid conversion | 8-12% | Healthy freemium funnel |
| MRR | $15-25K | Sustainable revenue |
| Generations/user/day | 10+ | Habit formation |
| Export rate | 25%+ | Generations are useful = satisfaction |
| NPS | 40+ | Strong recommendation intent |
| Churn (monthly) | <5% | Stickiness |
| P95 generation latency | <10s | Speed promise kept |
| Reference upload usage | 20%+ of users | Power feature adoption |

---

### Non-Goals (v1)

- No multi-user collaboration
- No cloud library sync
- No DAW plugin (VST3/AU in Phase 2)
- No song/loop generation
- No marketplace/community
- No mobile app
- No web app
- No real-time audio processing
- No advanced audio editing (EQ, compression, reverb)
- No MIDI generation or sequencing
- No stem separation
- No batch processing
- No API for developers
- No team/organization accounts
- No SSO or enterprise auth
