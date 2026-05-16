# cShot Beta Release Notes

## What cShot Does

cShot is a local-first, AI-powered one-shot sound generator for music producers and sound designers. Describe a sound, generate it instantly, preview it, and export to your DAW — all in under 10 seconds.

### Core Workflow

```
describe → generate → preview → export
```

### Key Features

- **AI Sound Generation** — Type a prompt like "punchy kick 140bpm" and get a WAV one-shot in seconds
- **Reference Conditioning** — Drag in a WAV reference to condition the generation
- **Variants** — Automatically generate 5 variants of every sound
- **Sound Quality Score** — Every sound gets a quality score based on spectral content, dynamics, and perceptual metrics
- **Repair Chain** — Fix clipping, trim silence, normalize, brighten/darken, add punch, shorten
- **Library Management** — Search, filter, favorite, export all generated sounds
- **Comparison Mode** — Compare original vs variants side-by-side
- **Version Tree** — Visualize parent/child relationships between sounds
- **Sound Design Recipes** — Pre-built and custom recipe templates for quick generation
- **Sample Import** — Import existing WAV/MP3 samples, analyze and auto-tag them
- **Folder Import** — Bulk import folders with safety limits and duplicate detection
- **Duplicate Detection** — Find exact duplicates by file hash
- **Smart Defaults** — Session memory learns your preferences over time
- **Pack Builder** — Group sounds into packs with cohesion analysis
- **Export to Desktop** — One-click WAV export with semantic filenames
- **Integrity Tools** — Scan for missing/orphan files, repair metadata
- **Privacy Controls** — Clear session memory, recent prompts, exported data
- **Local-First** — All data stored locally. No cloud accounts required

### Provider Support

- ElevenLabs SFX (primary, requires API key)
- mock-dsp (always-available fallback, no API key needed)
- Rate limiting detection and automatic fallback chain
- User-friendly error messages for all failure modes

## Experimental Features

The following features are functional but may have rough edges:

- **Sound morphing / interpolation** — Basic morphing between two sounds works but quality varies
- **Sonic cohesion metrics** — Pack cohesion analysis provides directional guidance but isn't production-grade
- **Imported sample analysis** — Auto-tagging of imported samples is basic (pattern-based, not ML)
- **Mono only** — All generated and processed audio is mono. Stereo support is planned
- **Provider embedding** — Mock embeddings only. Real embedding providers are not integrated

## What Users Should Test

1. **Core generation**: Type prompts for different sound types (kick, snare, hi-hat, clap, perc, bass, fx)
2. **Reference upload**: Drag a WAV file and generate a variant based on it
3. **Variants**: Generate variants and compare them in comparison mode
4. **Export**: Export sounds to Desktop and verify the WAV files play correctly
5. **Library**: Search, filter by type, favorite sounds, delete sounds
6. **Repair**: Apply normalize, trim silence, fade, brighten, darken, punch
7. **Recipes**: Browse built-in recipes, create custom recipes, generate from recipes
8. **Import**: Import a WAV or MP3 file, verify it's analyzed correctly
9. **Folder import**: Import a folder of samples with the dry-run preview
10. **Packs**: Create a pack, add sounds, view cohesion, export
11. **Cleanup**: Clear generated sounds, clear failed sounds, scan integrity
12. **Privacy**: Clear session memory, clear recent prompts
13. **Keyboard**: Enter to generate, Escape to stop playback

## Known Limitations

| Limitation | Impact | Status |
|------------|--------|--------|
| Mono audio only | No stereo/spatial sounds | Planned for post-beta |
| Mock DSP for fallback | Sound quality is lower with mock-dsp | Always available |
| No VST3/AU plugin | Must export and import to DAW | Post-beta |
| Single-screen UI | Advanced workflows may feel constrained | Intentional for beta |
| Basic auto-tagging | Tags may be imprecise for some sounds | Improvement planned |
| No multi-language prompts | English only | Post-beta |
| Large library performance | 500+ sounds may slow initial load | Optimization planned |
| No undo for delete | Deleted sounds are gone permanently | Coming soon |

## How to Install and Run

### Prerequisites

- Node.js 18+
- Rust toolchain (for building from source)
- An ElevenLabs API key (optional — for ElevenLabs provider)

### Setup

```bash
# Install dependencies
npm install

# Set up API keys (optional)
echo "ELEVENLABS_API_KEY=your_key_here" > .env

# Development
npm run tauri dev

# Production build
npm run tauri build
```

### Configuration

Create a `.env` file in the project root:

```
ELEVENLABS_API_KEY=sk_your_elevenlabs_key
```

Or use `~/.cshot.env` for user-level configuration.

## How to Report Feedback

- **GitHub Issues**: Report bugs and feature requests at [cshot/issues](https://github.com/anomalyco/cshot/issues)
- **In-App Feedback**: Use the feedback form in the tools view
- **Email**: feedback@cshot.app

## What's Coming Next

- **VST3/AU Plugin** — Generate sounds directly in your DAW
- **Real-Time Generation** — Sub-second generation for immediate preview
- **Stereo Support** — Stereo field control and spatial sounds
- **Improved Auto-Tagging** — ML-based tag recommendations
- **Cloud Sync** — Optional sync of recipes and favorites across devices
- **Marketplace** — Share and sell sound design recipes
- **Fine-Tuning** — Personalize the generation model to your taste
- **Sound Graph** — Visual discovery through sonic similarity
- **Collaborative Sessions** — Real-time collaborative sound design
