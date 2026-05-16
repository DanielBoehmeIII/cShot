# Prompt 31 — Design cShot as a DAW-Native System

cShot should not be a website. It should live inside the tools producers already use.

---

## 1. Design Philosophy

A producer should never leave their DAW to use cShot. Generation, audition, tagging, and export must feel like native DAW features — not a separate app running in a browser window.

### Principles

```
1. Zero-context-switch: Never tab out of the DAW
2. Latency empathy: Everything under 5ms response; generation runs async
3. DAW idioms: Follow each DAW's UI conventions, not cShot's
4. Drag-and-drop first: Output is a WAV on the timeline, period
5. Project-aware: Read tempo, key, time signature, arrangement from the session
6. Offline-capable: Core generation works without internet
7. Lightweight: Plugin CPU/RAM footprint must be negligible when idle
```

### Supported DAWs

| DAW | Plugin Format | Bridge Strategy | Priority |
|-----|--------------|-----------------|----------|
| Ableton Live | VST3, AU, Max for Live | Max device for deep integration | P0 |
| FL Studio | VST3 | Native plugin with Patcher support | P0 |
| Logic Pro | AU | Native AU + MIDI FX plugin | P1 |
| Pro Tools | AAX | AAX Native | P1 |
| Cubase/Nuendo | VST3 | Native VST3 | P2 |
| Studio One | VST3, AAX | Native | P2 |
| Reaper | VST3, JSFX | Native + ReaScript API | P2 |
| Bitwig | VST3, CLAP | CLAP for deeper integration | P2 |

---

## 2. VST3/AU Architecture

### Plugin Types

cShot ships as two plugin variants:

#### 2.1 cShot Generator (Instrument Plugin)

```
Purpose: Generate one-shots from within the DAW
Type: VSTi / AUi (instrument)
IO: 0 audio in, stereo out
MIDI: Note input for triggering (C1-C8 mapped to sounds)
```

#### 2.2 cShot Processor (Audio Effect)

```
Purpose: Transform/analyze existing audio in the session
Type: VST3 / AU (effect)
IO: Stereo in, stereo out
Sidechain: Optional reference input
Use cases: Resample selected clip, analyze stem, match EQ
```

### Plugin Internal Architecture

```
┌─────────────────────────────────────────────────────┐
│                  cShot VST3/AU Host                  │
├─────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────┐   │
│  │            UI Layer (Dear ImGui)             │   │
│  │  - Prompt input                             │   │
│  │  - Sound grid                               │   │
│  │  - Waveform/spectrogram preview             │   │
│  │  - Parameter controls                       │   │
│  │  - Variant tree (simplified)                │   │
│  │  - Export panel                             │   │
│  └─────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────┐   │
│  │         Plugin State Manager                 │   │
│  │  - DAW sync (tempo, key, transport)         │   │
│  │  - Session metadata                         │   │
│  │  - Project path resolution                  │   │
│  │  - Undo/redo history                        │   │
│  └─────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────┐   │
│  │           Audio Engine Bridge                 │   │
│  │  - Ring buffer for real-time comms          │   │
│  │  - Sample-accurate timing                   │   │
│  │  - Zero-copy buffer sharing                 │   │
│  │  - Asynchronous job submission              │   │
│  └─────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────┐   │
│  │            IPC / Local Server                 │   │
│  │  - Communicates with standalone service     │   │
│  │  - Shared memory for audio data             │   │
│  │  - REST-ish over Unix socket / named pipe   │   │
│  │  - Request queuing and priority             │   │
│  └─────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────┐   │
│  │         Model Registry (lightweight)         │   │
│  │  - Cached local models                      │   │
│  │  - Model version pinning                    │   │
│  │  - Fallback to CPU if no GPU                │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### Real-Time Safety

```
Audio thread (must never block):
  - Buffer swapping (lock-free ring buffer)
  - Parameter smoothing
  - MIDI note handling
  - Transport sync

Non-audio thread:
  - Model inference
  - File I/O
  - Network requests
  - UI rendering
  - Analysis jobs

Communication:
  - Atomic parameter flags
  - Lock-free SPSC queues for audio data
  - Timestamped command buffers
  - Priority: transport > MIDI > generation > UI > analysis
```

---

## 3. Standalone App Architecture

The standalone is cShot when you don't want to open a DAW — or when you want the full interface.

```
┌──────────────────────────────────────────────────────────┐
│                 cShot Standalone App                      │
├──────────────────────────────────────────────────────────┤
│  ┌──────────┐ ┌──────────────────┐ ┌────────────────┐   │
│  │ Browser  │ │   Sound Grid     │ │  Export Panel  │   │
│  │ /Upload  │ │   + Preview      │ │  + Pack Builder│   │
│  └──────────┘ └──────────────────┘ └────────────────┘   │
│  ┌──────────────────────────────────────────────────┐   │
│  │              Variation Tree Canvas                │   │
│  └──────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────┐   │
│  │         Waveform / Spectrogram / DNA View         │   │
│  └──────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────┐   │
│  │            Local Sample Browser                    │   │
│  └──────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────┐   │
│  │            Settings / Library Manager             │   │
│  └──────────────────────────────────────────────────┘   │
├──────────────────────────────────────────────────────────┤
│  Backend Services:                                       │
│  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌─────────────┐  │
│  │ Inference │ │ Analysis │ │ Search │ │ File Manager│  │
│  │ Engine    │ │ Pipeline │ │ Index  │ │             │  │
│  └──────────┘ └──────────┘ └────────┘ └─────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌───────────────────────┐   │
│  │ Model    │ │ Job      │ │ Local SQLite /        │   │
│  │ Registry │ │ Queue    │ │ Sled Database         │   │
│  └──────────┘ └──────────┘ └───────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

### Tech Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| UI Framework | Tauri (Rust) + React | Native performance, cross-platform, web UI tools |
| Audio I/O | CPAL (Rust) | Low-latency cross-platform audio |
| Audio Graph | custom DSP chain in Rust | Full control, no overhead |
| Model Runtime | ONNX Runtime / llama.cpp | Local inference, GPU acceleration |
| Database | SQLite via Diesel (Rust) | Embedded, zero-config |
| Sample Storage | Local filesystem + content-addressed store | Portable, verifiable |
| UI Components | React Flow / XYFlow | For variation tree graph |
| Plotting | custom WebGL waveform renderer | High-DPI, real-time |

---

## 4. DAW Bridge Design

The bridge connects the plugin UI to the generation backend. It handles sync, state, and audio transfer.

### Bridge Protocol

```
Plugin ←→ Bridge Service (same machine)

Transport: Unix domain socket (macOS/Linux) / Named pipe (Windows)
Serialization: FlatBuffers (zero-copy reads for audio)
Auth: Process-ID verification (only local DAW process can connect)

Message Types:
  ┌────────────────────┬────────────────────────────────┐
  │ Type               │ Payload                        │
  ├────────────────────┼────────────────────────────────┤
  │ TempoSync          │ BPM, time signature, position  │
  │ KeySync            │ Key, scale                     │
  │ TransportSync      │ Playing/stopped/recording      │
  │ GenerateRequest    │ Prompt, params, context        │
  │ GenerateResponse   │ Sound ID, metadata, waveform   │
  │ PreviewRequest     │ Sound ID, start/stop           │
  │ AudioData          │ Float32 samples, sample rate   │
  │ AnalysisRequest    │ Audio buffer, requested metrics│
  │ AnalysisResponse   │ Key, BPM, spectral data, etc.  │
  │ ExportRequest      │ Sound ID, format, path         │
  │ ExportResponse     │ Export status, file path       │
  │ DragDropRequest    │ Sound ID, format               │
  │ ModelStatus        │ What models loaded/ready       │
  │ Error              │ Error code, message, severity  │
  └────────────────────┴────────────────────────────────┘
```

### DAW-Specific Bridges

#### 4.1 Ableton Live (Max for Live)

```
Max Device:
  - [live.object] for tempo, key, transport
  - [live.path] for track/device navigation
  - [live.observer] for project changes
  - Custom JS for drag-and-drop (filepath injection)
  - [dict] objects for JSON IPC with bridge
  
Deep Integration:
  - Reads Live's Arrangement view for empty slots
  - Places generated sounds on new tracks
  - Creates warped audio clips matching project tempo
  - Can map cShot parameters to Live's macro knobs
  - Sends MIDI notes to trigger generation via Live's sequencer
```

#### 4.2 FL Studio

```
Wrapper:
  - VST3 plugin with FL Studio-specific parameter exposing
  - Patcher integration for signal flow
  - Edison integration for drag-out
  
Deep Integration:
  - Reads project tempo and key from FL Studio API
  - Can route generated audio to mixer tracks
  - Writes to FL Studio's browser/crushed audio folders
  - Supports channel rack preview
```

#### 4.3 Logic Pro

```
Wrapper:
  - AU instrument + AU MIDI FX plugin
  - MIDI FX plugin translates chord/note data for context
  
Deep Integration:
  - Reads global tempo, key signature track
  - Respects Logic's project management (saves under project bundle)
  - Uses Audio Unit's built-in preset management
  - Supports Logic's Flex Time/Flex Pitch integration
```

#### 4.4 Pro Tools

```
Wrapper:
  - AAX Native plugin
  - Avid Audio Engine integration
  
Deep Integration:
  - Reads session tempo map, key signature
  - Integrates with Pro Tools' clip-based workflow
  - Writes rendered files to session's Audio Files folder
  - Supports Elastic Audio for time-stretching
```

### Bridge Lifecycle

```
DAW Opens Project
  → Plugin initializes
  → Bridge service starts (or connects to existing)
  → Handshake: DAW type, version, project path
  → Initial sync: tempo, key, transport
  
Plugin UI Opens
  → Request model status from bridge
  → Populate sound grid (cached or empty)
  → Ready state (idle, low CPU)

User Generates
  → Plugin sends GenerateRequest (prompt + context)
  → Bridge queues job, returns immediately
  → Background: inference → audio write → preview prepare
  → Bridge sends GenerateResponse with sound ID
  → Plugin updates sound grid

User Drags Out
  → Plugin sends DragDropRequest (sound ID)
  → Bridge writes WAV to DAW's drag-drop temp location
  → DAW picks up file automatically

User Closes Project
  → Plugin sends project close signal
  → Bridge flushes cache, saves metadata
  → Plugin unloads

DAW Closes
  → Plugin destructor called
  → Bridge receives disconnect
  → Optional: stays alive for other sessions
  → Timeout → clean shutdown
```

---

## 5. Local File Management

### Storage Layout

```
~/cShot/
├── library/                          # Managed sample library
│   ├── index.db                      # SQLite database (metadata, tags, embeddings)
│   ├── audio/                        # Content-addressed WAV files
│   │   └── {sha256_prefix}/
│   │       └── {sha256}.wav
│   ├── thumbnails/                   # Waveform PNG thumbnails
│   └── models/                       # Cached model files (ONNX, etc.)
│       ├── generator_v1.onnx
│       ├── classifier_v2.onnx
│       └── embedder_v1.onnx
├── projects/                         # User's cShot projects
│   └── {project_name}/
│       ├── project.cshot             # JSON project file
│       ├── sounds/                   # Generated sounds for this project
│       ├── exports/                  # Exported WAV files
│       └── provenance.db            # Per-project provenance chain
├── packs/                            # User-created sample packs
│   └── {pack_name}/
│       ├── pack.json                 # Pack metadata
│       └── audio/                    # Pack content (copied/referenced)
├── favorites.db                      # Cross-project favorites
├── config.toml                       # User preferences
└── bridge.sock                       # Unix domain socket (runtime)
```

### Content-Addressed Storage

```
Why:
  - Deduplication: same sound generated twice → one file
  - Integrity: hash verification prevents corruption
  - Portable: no path dependencies
  - Mergeable: two libraries can merge without conflict
  
Schema:
  Hash: SHA-256 of Float32 audio samples
  Format: 44.1kHz, 16-bit, mono (for one-shots)
  Path: /library/audio/{hash[0:2]}/{hash[2:4]}/{hash}.wav
  
Metadata (SQLite):
  CREATE TABLE sounds (
    id TEXT PRIMARY KEY,             -- UUID
    hash TEXT UNIQUE NOT NULL,       -- SHA-256 of audio
    original_filename TEXT,
    duration REAL,                   -- seconds
    sample_rate INTEGER,
    channels INTEGER,
    bit_depth INTEGER,
    file_size INTEGER,
    created_at TEXT,                 -- ISO 8601
    source TEXT,                     -- 'generated', 'imported', 'resampled'
    model_version TEXT,
    prompt TEXT,
    params_json TEXT,                -- generation parameters
    provenance_chain TEXT            -- JSON array of ancestor IDs
  );
  
  CREATE TABLE tags (
    sound_id TEXT REFERENCES sounds(id),
    tag TEXT,
    confidence REAL,                 -- 0.0-1.0 for auto-tags
    source TEXT,                     -- 'auto', 'user', 'model'
    created_at TEXT,
    PRIMARY KEY (sound_id, tag, source)
  );
  
  CREATE TABLE embeddings (
    sound_id TEXT PRIMARY KEY REFERENCES sounds(id),
    vector BLOB,                     -- 768 float32 embedding
    model_version TEXT,
    created_at TEXT
  );
```

---

## 6. Export Workflow

### Drag-and-Drop Export

```
User Action:
  1. Hover sound in grid
  2. Begin drag (mouse down + move)
  3. Drag to DAW track / sampler / browser
  4. Release

Behind the Scenes:
  1. Plugin detects drag start
  2. Sends DragDropRequest to bridge
  3. Bridge writes temp WAV to DAW-specific location:
     - Live: ~/Music/Ableton/User Library/DropAudio/
     - FL Studio: FL Studio's "Packs" temp dir
     - Logic: NSPasteboard with file promise
     - Pro Tools: Session's Audio Files folder
  4. DAW picks up file from drag-drop target
  5. Plugin receives confirmation
  6. Optionally: add to export history
```

### Direct Export (File Menu)

```
User Action:
  1. Select sound(s)
  2. Click "Export" (or Cmd+E)
  3. Choose: single / batch
  4. Set options in export panel:
     - Format: WAV, AIFF, FLAC, MP3
     - Sample rate: 44.1k, 48k, 96k
     - Bit depth: 16, 24, 32
     - Mono/Stereo
     - Normalization: peak / loudness / none
     - Filename template: {name}_{bpm}_{key}_{variant}
     - Export location
  5. Click Export
  6. Progress bar for batch
  7. "Reveal in Finder/Explorer" option
```

### Pack Builder Export

```
Export multiple sounds as a coherent sample pack:

  1. Select multiple sounds (Shift/Cmd+click)
  2. Click "Create Pack" or drag to Pack Builder panel
  3. Pack Builder organizes:
     - Auto-names: Kick_01.wav, Snare_03.wav, etc.
     - Creates folder structure: Kick/, Snare/, Hat/, Percussion/
     - Generates pack.json with metadata
     - Optional: demo audio preview
  4. Export as:
     - Folder with organized WAVs
     - ZIP archive (for sharing)
     - Ableton Pack (.alp)
     - Battery Kit (.kit)
     - Kontakt Instrument
     - NN-XT preset
  5. Auto-tags each file with rough genre/category metadata
```

### DAW-Specific Export Optimizations

| DAW | Export Feature | Implementation |
|-----|---------------|----------------|
| Ableton | Warped clip export | Generate + compute warp markers for project BPM |
| Ableton | Simpler preset | Export with .adv preset file, macros for envelope/shaping |
| FL Studio | Channel preset | Export as .fst with basic envelope routing |
| FL Studio | Direct-to-step sequencer | Place sound directly in channel rack |
| Logic | Quick Sampler instrument | Export as .exs with zone mapping across keys |
| Logic | Drum Machine Designer | Export as DMD kit piece with tuning metadata |
| Pro Tools | Clip group integration | Create clip groups that follow session grid |
| Pro Tools | Elastic Audio markers | Pre-analyze transient markers for time-stretching |

---

## 7. Plugin UX

### UI Design Principles

```
1. Dark, minimal, high-density — like a pro tool, not a consumer app
2. Keyboard-first — every action has a shortcut
3. Resizable — from tiny strip to full-screen
4. DAW-consistent — match each DAW's font, colors, widget style
5. Progressive disclosure — basic mode hides latent controls
```

### Layout (Default State)

```
┌─────────────────────────────────────────────────────────────┐
│ [Prompt...                                  ] [Generate]    │
│ ┌───────┬───────┬───────┬───────┬───────┬───────┬──────┐  │
│ │Sound 1│Sound 2│Sound 3│Sound 4│Sound 5│Sound 6│More │  │
│ │█▁▃▇▆▄▂│█▁▃▇▆▄▂│█▁▃▇▆▄▂│█▁▃▇▆▄▂│█▁▃▇▆▄▂│█▁▃▇▆▄▂│ ... │  │
│ │tag1 t2│tag1 t3│tag2 t4│tag1 t2│tag3 t5│tag1 t6│     │  │
│ └───────┴───────┴───────┴───────┴───────┴───────┴──────┘  │
│ [♫ Play: Spacebar] [★ Favorite] [↻ Variants] [↓ Export]   │
│ ┌─────────────────────────────────────────────────────┐   │
│ │ Waveform / Spectrogram Viewer (selected sound)      │   │
│ └─────────────────────────────────────────────────────┘   │
│ [Parameters: Attack] [Decay] [Tone] [Pitch] [FX...]       │
└─────────────────────────────────────────────────────────────┘
```

### Interaction Flow

```
State: Empty / First Open
  1. User sees prompt bar with placeholder: 
     "Describe the sound you want... or drag in a reference"
  2. Sound grid shows 6 empty slots with "Generate" placeholders
  3. Context bar shows detected project info: 
     "Live session · 128 BPM · G minor"
  
State: Prompt Entered
  1. User types: "punchy 808 kick with short decay"
  2. Auto-complete suggests: "punchy kick", "808 kick", "short decay kick"
  3. Hit Enter → [Generate] → sounds appear in grid (async)
  4. Sound grid fills left-to-right, top generation slot highlighted
  5. First sound auto-previews (or waits, user preference)
  
State: Sound Selected
  1. Waveform/spectrogram shows for selected sound
  2. Tags appear below (auto-detected)
  3. Parameter knobs populate with current sound's settings
  4. User can tweak → real-time reprocess (or regenerate, depending on param)
  5. Quick actions bar visible: ★ ↻ ↓ 🎯 (favorite, variants, export, similar)
  
State: Exploring Variants
  1. User clicks ↻ → variant tree expands as dropdown or side panel
  2. Shows: Current sound as root, 4-6 variants branching off
  3. Click variant → previews immediately, replaces selection
  4. User can mark favorites, branch deeper
  5. Full variation tree available in standalone app
  
State: Export
  1. User clicks ↓ or drags sound to DAW timeline
  2. Toast: "Exported: Kick_128bpm_Gm.wav"
  3. Sound added to export history
  4. Undo available (Cmd+Z removes last export)
```

### Keyboard Shortcuts

```
Global:
  Space        Play/stop preview
  Enter        Generate from prompt
  Escape       Clear selection / close panel

Navigation:
  ← → ↑ ↓     Navigate sound grid
  Tab          Focus next control
  Shift+Tab    Focus previous control
  Cmd+1-9      Jump to sound slot

Actions:
  F            Favorite/unfavorite sound
  V            Generate variants
  E            Export sound
  D            Download / reveal in finder
  S            Tag sound (opens tag editor)
  Cmd+C        Copy sound to clipboard (generates WAV on paste)
  Cmd+Z        Undo last action
  Cmd+Shift+Z  Redo

Playback:
  Shift+Space  Play from cursor in waveform
  Cmd+Space    Toggle loop preview
  ,/.          Previous/next sound in grid

Modifiers:
  Shift+Click  Add to multi-select
  Cmd+Drag     Drag copy (for DAW drag-out)
  Alt+Click    Quick-favorite without entering detail view

Advanced:
  L            Toggle latent controls panel
  T            Toggle variation tree
  P            Toggle pack builder
  /            Focus search/tag filter
  ?            Show keyboard shortcuts help
```

### Beginner vs Expert Modes

```
Beginner Mode:
  - Single prompt bar (no advanced parameters)
  - 4 sound slots instead of 6
  - Simple buttons: Generate, ♫, ★, ↓
  - No latent controls visible
  - No variation tree (simplified "more like this" button)
  - Auto-preview first result
  - Tooltips on everything
  - "Quick start" overlay on first launch

Expert Mode:
  - Full prompt bar with modifiers (/bpm=140 /key=Fm /genre=trap)
  - 6+ sound slots (customizable grid)
  - All parameter controls visible (ADSR, pitch, filter, fx chain)
  - Latent controls (temperature, truncation, seed, interpolation)
  - Variation tree with full lineage
  - Custom export presets
  - No tooltips, denser layout
  - Scriptable (expose via MIDI CC / OSC)
  - Multi-selection and batch operations
  - Keyboard-only workflow possible

Mode Switching:
  - Explicit toggle in settings
  - Or: "intelligent mode" — detect DAW track context and show relevant complexity
  - Beginner graduates naturally as they use more features
```

### Plugin Size States

```
Mini (collapsed):    320x48 px
  - Prompt bar only, 1 slot preview
  - For keeping on track while working

Small:               480x200 px
  - Prompt bar + 2x2 sound grid
  - Minimal parameters

Medium (default):    800x500 px
  - Full layout as above
  - For dedicated generation work

Large (expanded):    1200x800 px
  - Full variation tree visible
  - Spectrogram detail
  - Batch controls

Full-screen:         Matches DAW window
  - Standalone-level interface
  - All panels visible
```

### Sound Preview Behavior

```
Immediate Mode (default):
  - Click sound → instant preview (sub-5ms start)
  - Preview plays through plugin audio output
  - Cursor shows playback position in waveform
  - Click again → stop
  - Click different sound → cross-fade (10ms) to new sound
  
Trigger Mode:
  - Sound plays via MIDI note input
  - Each grid slot mapped to consecutive MIDI notes (C3-B3)
  - Velocity maps to preview volume
  - Pitch bend maps to coarse pitch (for tonal sounds)
  
Loop Mode:
  - Sound loops while previewing
  - Loop points auto-detected (or set at zero crossings)
  - Useful for sustained sounds, atmospheres
  
Batch Preview Mode:
  - Play through all sounds in grid sequentially
  - Configurable gap (1-5 seconds)
  - "Walking" highlight on current slot
  - Good for A/B comparison
```

---

## 8. Context Awareness

The plugin reads DAW context and adjusts generation accordingly.

### What cShot Reads from the DAW Session

```
Tempo & Time Signature:
  - Affects decay length (kick decay = 1/4 note)
  - Affects reverb tail (damped at tempo-synced rate)
  - Affects transient placement (grid-aligned)

Key & Scale:
  - Tunes tonal content to project key
  - Kick fundamental → root note
  - Snare body → avoid conflicting frequencies
  - Hi-hat pitch → harmonic series of key

Arrangement:
  - Reads track names ("Kick", "Snare", "808", etc.)
  - Detects which frequency ranges are crowded
  - Adjusts new sounds to avoid masking
  
Mix Context (sidechain input):
  - Analyzes existing mix spectrum
  - Identifies frequency masking potential
  - Recommends complementary frequency placement
  - Detects overall dynamic range and matches

Project Metadata:
  - Genre tags (if available)
  - Project name
  - Time spent on project
  - Recent actions/undo history
```

### Context-Aware Generation Example

```
User is producing a trap beat at 140 BPM in C minor.
Kick track already has a punchy layered kick.
cShot detects: 
  - 140 BPM → short decay needed
  - C minor → tune to C fundamental
  - Existing kick → make new kick complement, not compete
  - "Kick" track exists → offering snares and hats instead

cShot output:
  "Working on a trap beat at 140 BPM in C minor.
   Your kick is punchy and mid-heavy.
   
   I recommend:
   1. A sub-heavy 808 with long tail (140 BPM = ~428ms 1/4 note)
   2. A tight snare with 200Hz resonance (complementing kick's 60Hz)
   3. A hat pattern with short, crisp transients
   
   Want me to generate all three as a matched set?"
```

---

## Summary

| Capability | Plugin | Standalone |
|-----------|--------|------------|
| Generate one-shots | ✅ Full | ✅ Full |
| DAW context awareness | ✅ Deep | ❌ (no DAW) |
| Drag-and-drop export | ✅ Native | ✅ File system |
| Variation tree | ✅ Simplified | ✅ Full |
| Sample browser | ⚠️ Basic | ✅ Full |
| Pack building | ⚠️ Export only | ✅ Full |
| MIDI triggering | ✅ Full | ❌ |
| Offline operation | ✅ Full | ✅ Full |
| Cloud sync | ❌ | ✅ Optional |
| Full latent controls | ⚠️ Expert mode | ✅ Always |

The plugin is for *in-DAW generation*. The standalone is for *sound design exploration*. A producer will use both: explore in standalone, generate in-plugin.
