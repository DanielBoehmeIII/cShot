# Prompt 67 — Design the First Plugin Version

Design the first DAW plugin version of cShot. Fastest path to a usable plugin that lets producers generate and drag sounds without leaving their DAW.

---

## 1. Plugin Format Comparison

### VST3

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Market share | 85%+ | Universal. Every major DAW supports VST3 (Ableton, FL Studio, Cubase, Reaper, Bitwig, Studio One) |
| SDK maturity | Excellent | Steinberg VST3 SDK, well-documented, stable API |
| UI framework | Any | Can embed any UI technology (webview, OpenGL, custom) |
| Audio/MIDI I/O | Full | Audio input/output, MIDI input/output, sidechain |
| Parameter automation | Full | All parameters automatable in DAW |
| Cross-platform | Yes | Windows, macOS (Linux through some DAWs) |
| Development effort | Medium | Well-understood format, good tooling |
| Distribution | Easy | Plugins folder, no installer needed for basic version |

### AU (Audio Unit)

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Market share | ~40% (macOS only) | All Mac DAWs support AU. Required for Logic Pro, GarageBand |
| SDK maturity | Good | Apple's AU SDK, stable |
| UI framework | Any | Same as VST3 |
| Audio/MIDI I/O | Full | Matches VST3 capability |
| Cross-platform | macOS only | Dealbreaker on its own |
| Development effort | Medium | Similar to VST3 |
| Distribution | Easy | Component folder, code-signing recommended |

### CLAP

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Market share | ~5% (growing) | Newer format. Supported by Bitwig, Reaper, Cubase (beta), FL Studio (via wrapper) |
| SDK maturity | Good | Open source, clean API, MIT-licensed |
| UI framework | Any | Same as VST3/AU |
| Audio/MIDI I/O | Full | Well-designed, modern API |
| Cross-platform | Yes | Designed for it |
| Development effort | Low | Cleanest API of the three |
| Distribution | Easy | Same as VST3 |
| Verdict | **Future format.** Too early to be primary, but worth dual-shipping |

### Standalone Bridge (Tauri + LoopMidi/REAPER ReaRoute)

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Development effort | Lowest | No plugin SDK needed. Tauri app communicates via virtual MIDI + audio loopback |
| User experience | Poor | Extra setup, routing, latency. Not "it just works" |
| Integration | Minimal | No DAW context (BPM, key, transport). Manual routing |
| Verdict | **Prototype only.** Proves the concept, not the product |

### Web-to-DAW Workflow (Current)

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Development effort | None (already done) | Current cShot app exports WAV, user drags into DAW |
| User experience | Acceptable | One extra step: export → switch to DAW → drag |
| Integration | None | No DAW context awareness |
| Verdict | **Keep as fallback.** Plugin is for when you want zero friction |

### Recommendation

```
Fastest path to usable plugin:
  1. Ship VST3 first (covers 85% of market, all major DAWs)
  2. Ship AU simultaneously if possible (same SDK pattern, covers Logic)
  3. Add CLAP as secondary target (future-proofing)
  4. Keep standalone web-to-DAW as fallback (already works)
  
  Skip:
    - AAX (Pro Tools) — different SDK, small market for one-shots
    - Standalone bridge (LoopMidi) — poor UX
    - Webview in plugin (too complex for v1)
```

---

## 2. Plugin UI Architecture

### v1.0 Plugin: Minimal Viable Plugin

```
┌─────────────────────────────────────────────────────────┐
│  cShot  [≡]                                              │
│                                                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │  [🔍 Search or describe a sound...          ]    │    │
│  │  [⚡ Generate]                                   │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │  ┌──────────────────────┐                        │   │
│  │  │  Waveform            │                        │   │
│  │  │  ━━━━━━━━━━━━━      │   Score: 72            │   │
│  │  │                      │                        │   │
│  │  │  Kick · 412ms        │                        │   │
│  │  │                      │                        │   │
│  │  │  [▶] [♥] [⤓]        │                        │   │
│  │  └──────────────────────┘                        │   │
│  │                                                   │   │
│  │  ┌──────────────────────┐                        │   │
│  │  │  History ▼           │                        │   │
│  │  │  punchy kick         │                        │   │
│  │  │  tight snare 140bpm  │                        │   │
│  │  │  deep 808            │                        │   │
│  │  └──────────────────────┘                        │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  [Drag me to track]     BPM: 140  Key: F#m              │
│                                                          │
│  Connection: ● Connected | User: producer_a              │
└─────────────────────────────────────────────────────────┘
```

### v1.0 Features

| Feature | Description | Priority |
|---------|-------------|----------|
| Login/library access | Authenticate with cShot account, access favorites and history | Required |
| Generate one-shot | Text prompt → generate → preview in plugin | Required |
| Preview sounds | Click to play generated sound (low latency, local audio buffer) | Required |
| Drag to DAW | Drag waveform from plugin to DAW track → creates audio clip | Required |
| MIDI trigger preview | Trigger preview via MIDI note (play generated sound at MIDI pitch) | Required |
| BPM awareness | Read DAW transport BPM, auto-insert into prompt | High |
| Key awareness | Read DAW project key, auto-insert into prompt | High |
| Favorites | Heart toggle, show favorited sounds | High |
| Recent generations | Show last 20 generated sounds in history list | High |
| SoundScore display | Show quality score on each generated sound | Medium |
| Reference upload | Drag reference into plugin for context-aware generation | Medium |

### v1.0 Non-Goals

```
✗ Real-time generation (generate → play in <5s)
✗ MIDI note output (generate sounds as MIDI)
✗ Audio input processing (no sidechain)
✗ Multi-output (stereo out only)
✗ Plugin preset system
✗ Full library browser (use history + favorites)
✗ Collaboration features
✗ Pack generation in plugin
✗ Complex parameter automation
```

### UI Technology Decision

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| **OpenGL + IMGUI** | Fast, no dependencies, full control | Custom UI code, no web tech, big effort | ✗ Too much work |
| **Webview (CEF/WebView2)** | Reuse existing React UI, familiar | Heavy (~100MB), slow startup, macOS complexities | ✗ Not worth it for v1 |
| **VST3 with embedded webview** | Best of both worlds | Complex integration, debugging nightmare | ✗ Phase 2 |
| **Native plugin UI (Rust + egui)** | Light (~5MB), fast, Rust-native | New UI code, but can be simple for v1 | ✓ Best choice |
| **JUCE** | Battle-tested, all formats, all platforms | C++, big framework, LGPL licensing | ✗ Different language |

**Recommendation: Use `egui` (Rust immediate-mode GUI) for v1 plugin UI.**

`egui` is:
- Pure Rust — reuse audio DSP and model gateway code from Tauri app
- Lightweight (~5MB binary)
- Cross-platform (Windows, macOS, Linux)
- Simple to develop (immediate-mode — just draw each frame)
- Already has a VST3 example (`egui-base` + `nih-plug` integration)
- No external dependencies (no webview, no OpenGL setup)

---

## 3. Plugin Architecture

### v1.0 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    DAW Host                                       │
│                                                                   │
│  ┌───────────────────────────────────────────────────────────┐   │
│  │                  cShot VST3 Plugin                          │   │
│  │                                                              │   │
│  │  ┌────────────────────────────────────────────────────┐     │   │
│  │  │  UI Layer (egui)                                    │     │   │
│  │  │  • Prompt input, generate button                    │     │   │
│  │  │  • Waveform display                                 │     │   │
│  │  │  • Play/fav/export controls                         │     │   │
│  │  │  • History list                                     │     │   │
│  │  └────────────────────┬───────────────────────────────┘     │   │
│  │                        │                                     │   │
│  │  ┌────────────────────▼───────────────────────────────┐     │   │
│  │  │  Audio Engine (Rust)                                │     │   │
│  │  │  • Audio buffer management                          │     │   │
│  │  │  • Preview playback (local buffer)                  │     │   │
│  │  │  • Drag-to-DAW audio output                         │     │   │
│  │  │  • Waveform thumbnail computation                   │     │   │
│  │  └────────────────────┬───────────────────────────────┘     │   │
│  │                        │                                     │   │
│  │  ┌────────────────────▼───────────────────────────────┐     │   │
│  │  │  Network Layer (reqwest)                            │     │   │
│  │  │  • Authentication (JWT)                             │     │   │
│  │  │  • Model Gateway API calls                          │     │   │
│  │  │  • Library sync (favorites, history)                │     │   │
│  │  └────────────────────────────────────────────────────┘     │   │
│  └───────────────────────────────────────────────────────────┘   │
│                                                                   │
│  Audio/MIDI I/O:                                                  │
│  ┌───────────────────────────────────────────────────────────┐   │
│  │  • Audio output: Generated sound plays through plugin     │   │
│  │  • MIDI input: Note triggers preview (with pitch bend)    │   │
│  │  • No audio input (v1)                                    │   │
│  └───────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Rust Dependencies

```toml
[dependencies]
# Plugin framework
nih-plug = { version = "0.5", features = ["vst3"] }

# UI
egui = "0.27"
egui-base = "0.27"  # nih-plug integration for egui

# Audio
hound = "3"                      # WAV read/write
symphonia = "0.5"                # Audio decoding (for imported references)

# Network
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Auth
base64 = "0.22"
ring = "0.17"                    # JWT verification

# Async (for network calls)
tokio = { version = "1", features = ["rt", "macros"] }

# Cache
lru = "0.12"

# Utilities
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Plugin Structure

```
cshot-plugin/
├── Cargo.toml
├── src/
│   ├── main.rs                    # nih-plug entry point
│   ├── params.rs                  # Plugin parameters (exposed to DAW)
│   ├── editor.rs                  # egui UI setup
│   │
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── prompt.rs              # Prompt input + generate button
│   │   ├── waveform.rs            # Waveform display widget
│   │   ├── controls.rs            # Play/fav/export buttons
│   │   ├── history.rs             # Recent generations list
│   │   ├── login.rs               # Login screen
│   │   └── theme.rs               # Dark theme styling
│   │
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── buffer.rs              # Audio buffer management
│   │   ├── playback.rs            # Preview playback engine
│   │   └── output.rs              # Audio output to DAW
│   │
│   ├── net/
│   │   ├── mod.rs
│   │   ├── auth.rs                # Authentication
│   │   ├── api.rs                 # Model Gateway API client
│   │   └── sync.rs                # Library sync
│   │
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── local.rs               # Local SQLite cache
│   │   └── cache.rs               # LRU audio cache
│   │
│   └── utils/
│       ├── mod.rs
│       └── waveform.rs            # Waveform thumbnail computation
```

---

## 4. Key Plugin Features

### Drag-to-DAW

The most important plugin feature. User drags waveform → audio file appears on DAW track.

```rust
// Implementation: Standard VST3 drag-and-drop
// On drag start:
// 1. Export current sound to temp WAV file
// 2. Create IDataPackage with file URL
// 3. DAW handles the rest

impl Editor for CshotEditor {
    fn on_drag_start(&mut self, sound_id: &str) {
        // Export to temp file
        let temp_path = temp_dir().join(format!("cshot_{}.wav", sound_id));
        if !temp_path.exists() {
            let audio = self.audio_cache.get(sound_id).unwrap();
            export_wav(audio, 44100, 24, &temp_path).unwrap();
        }
        
        // Create file URL
        let file_url = Url::from_file_path(&temp_path).unwrap();
        
        // Initiate drag (VST3 drag-drop API)
        // DAW receives file URL, creates audio clip on drop
    }
}
```

### MIDI Trigger Preview

```
MIDI Input → Plugin: If DAW has a MIDI track routed to cShot:
  - Note On → Start preview of current sound at MIDI note pitch
  - Note Off → Stop preview (for sustained sounds), or let ring
  - Pitch Bend → Apply pitch bend to preview
  - Velocity → Map to gain (soft/hard hit)
  
  This lets producers:
  - Trigger cShot sounds from their MIDI controller
  - Preview sounds at different pitches (kick at C1 vs C2)
  - Layer cShot sounds with their existing instruments
  - Incorporate cShot into their existing performance workflow
```

### BPM/Key Awareness

```rust
impl Plugin for CshotPlugin {
    fn process(&mut self, buffer: &mut Buffer, context: &mut ProcessContext) -> ProcessStatus {
        // Read DAW transport
        let transport = context.transport();
        
        if let Some(tempo) = transport.tempo() {
            self.current_bpm = Some(tempo as u32);
            
            // Auto-update prompt chip
            self.ui_state.bpm_chip = Some(tempo as u32);
            
            // Use BPM for generation if user has "auto-BPM" enabled
            if self.settings.auto_bpm {
                self.pending_generation.bpm = Some(tempo as u32);
            }
        }
        
        if let Some(time_sig) = transport.time_signature() {
            self.current_time_sig = Some(time_sig);
        }
        
        // Future: Read project key from DAW metadata
        // (Not all DAWs expose this via VST3 API)
        
        ProcessStatus::Normal
    }
}
```

### Library Access

```rust
pub struct LibrarySync {
    local_db: SqliteConnection,     // Local SQLite cache
    api_client: ApiClient,          // Cloud API
    sync_status: SyncStatus,
}

impl LibrarySync {
    pub async fn sync(&mut self) -> Result<()> {
        match self.sync_status {
            SyncStatus::NotLoggedIn => {
                // Show login screen
            }
            SyncStatus::LoggedIn { user_id, token } => {
                // 1. Pull latest favorites from cloud
                let cloud_favorites = self.api_client.get_favorites(&token).await?;
                
                // 2. Merge with local
                for fav in cloud_favorites {
                    self.local_db.upsert_favorite(fav)?;
                }
                
                // 3. Push local changes to cloud
                let local_changes = self.local_db.get_changes_since(last_sync)?;
                self.api_client.sync_favorites(local_changes, &token).await?;
                
                // 4. Update sync status
                self.last_sync = Utc::now();
            }
        }
        Ok(())
    }
}
```

---

## 5. Plugin Distribution

### VST3 Packaging

```
Platform-specific packaging:

macOS:
  - Build: cargo build --release
  - Output: libcshot_plugin.dylib
  - Package as: cshot_plugin.vst3/Contents/MacOS/libcshot_plugin.dylig
  - Code sign: codesign --force --sign "Developer ID" cshot_plugin.vst3
  - Install: ~/Library/Audio/Plug-Ins/VST3/cshot_plugin.vst3/

Windows:
  - Build: cargo build --release
  - Output: cshot_plugin.dll  
  - Package as: cshot_plugin.vst3/Contents/x86_64-win/cshot_plugin.dll
  - Install: C:\Program Files\Common Files\VST3\cshot_plugin.vst3\

Distribution:
  - Direct download from cshot website (.zip with .vst3 folder inside)
  - No installer needed (user unzips to VST3 folder)
  - Auto-update: Check version on launch, download update
  - License verification: JWT token validated on startup
```

### Installation Flow

```
1. User downloads cshot-plugin-v1.0.0.zip from cshot.ai/download
2. Unzips → cshot_plugin.vst3 folder
3. User copies to VST3 folder (or installer does it)
4. Opens DAW → Rescan plugins → cShot appears in instrument list
5. Drags cShot to MIDI or instrument track
6. Plugin window opens → Login screen
7. User logs in with cShot account (or creates one)
8. Ready to generate

Total time from download to first generation: ~2 minutes
Total friction: Login (30 seconds) + scanning (varies by DAW)
```

---

## 6. Plugin Version Roadmap

### v1.0 — MVP Plugin (3 months)

```
Focus: Generation + drag-to-DAW. Minimum viable.

Ships:
  ✓ VST3 (and AU on macOS)
  ✓ Login with cShot account
  ✓ Text prompt → generate → preview
  ✓ Drag waveform to DAW track
  ✓ MIDI trigger preview (note on/off)
  ✓ DAW BPM auto-detect
  ✓ SoundScore display
  ✓ Favorites (heart toggle)
  ✓ Recent generations history (20 sounds)
  ✓ Dark theme (matches cShot standalone)
  
Tech: nih-plug + egui + reqwest
Size: ~5MB binary
```

### v1.5 — Production Plugin (6 months)

```
Focus: Workflow integration, quality of life.

Ships:
  ✓ AU validation (stable on Logic Pro)
  ✓ CLAP format support
  ✓ DAW key awareness
  ✓ Reference upload (drag reference into plugin)
  ✓ High-level controls (punch, body, weight, snap, air)
  ✓ Batch generate (generate 5 variations)
  ✓ Auto-fill with BPM + key chips
  ✓ Keyboard shortcuts inside plugin
  ✓ Sound comparison (A/B last two generations)
  ✓ Plugin preset saving (per-session defaults)
```

### v2.0 — Power User Plugin (12 months)

```
Focus: Deep DAW integration, personalization.

Ships:
  ✓ Personal sonic identity integration
  ✓ Context-aware suggestions (based on arrangement)
  ✓ Pack generation (generate a full kit from one prompt)
  ✓ Export to audio track (automatically creates audio clip on timeline)
  ✓ MIDI output (also output MIDI alongside audio)
  ✓ Advanced parameter automation per sound
  ✓ Multi-output (separate outputs for kick/snare/hat)
  ✓ Offline cache (recent favorites available without internet)
  ✓ Collaboration features (shared packs in plugin)
```

---

## 7. Plugin vs Standalone: What Stays

```
What stays in standalone only:
  - Full library browser (favorites grid, search, filtering)
  - Batch pack export (multi-sound, named, organized)
  - Reference analysis visualization
  - Sound evolution timeline
  - Collaboration features (pack creation, commenting)
  - Settings and configuration
  - Taste profile visualization
  - Export history
  - Keyboard shortcut customization

What moves to plugin (v1.0):
  - Generate from text prompt
  - Preview generated sound
  - Drag to DAW
  - Favorites toggle
  - Recent history (20 latest)
  - DAW BPM detection
  - MIDI trigger
  - SoundScore display

What stays in both (synced):
  - User account
  - Favorites (synced via cloud)
  - Generation history (synced via cloud)
  - Taste profile (synced via cloud)
  - Settings (per-device, not synced)
```

---

## 8. Plugin Development Timeline

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1-2 | nih-plug setup | Empty plugin loads in DAW, shows egui window |
| 3-4 | UI shell | Prompt input, generate button, waveform placeholder |
| 5-6 | Network layer | Login flow, API client, token management |
| 7-8 | Generation | Full generate → preview → drag flow works |
| 9-10 | MIDI + BPM | MIDI trigger, BPM auto-detect |
| 11 | Polish | SoundScore, favorites, history, dark theme |
| 12 | Package + Ship | VST3/AU builds, signing, distribution, documentation |

---

## 9. Summary

```
cShot Plugin v1.0 — Key Decisions:

  1. VST3 first (85% market share), AU shipped simultaneously.
     CLAP in v1.5. Skip AAX and standalone bridge.

  2. Built with nih-plug + egui in Rust. Reuses Tauri app's
     audio DSP and model gateway code. Same codebase.

  3. Core features: Generate, preview, drag to DAW, MIDI trigger,
     BPM auto-detect. No library browser, no collaboration.

  4. Drag-to-DAW is the killer feature: user drags waveform
     from plugin → audio clip appears on DAW track. Zero friction.

  5. Standalone app continues alongside plugin. Plugin is for
     the "generate in place" workflow. Standalone is for
     exploration, packs, libraries, and collaboration.

  6. 12-week build from start to shipping binary.
     Smallest possible plugin that proves the concept.
```
