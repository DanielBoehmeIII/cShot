# Prompt 49 — Design the Alpha User Flow

The alpha experience for cShot's first 10 real users: producers, sound designers, and experimental musicians.

---

## 1. User Personas (Alpha Invitees)

| Persona | Background | What They Care About | What They'll Break |
|---------|-----------|---------------------|-------------------|
| Alex (beatmaker) | Makes trap/hip-hop beats in Ableton. Has Splice subscription | Speed, punch, mix-readiness. "Will this save me time?" | Weak kicks, bad tags, slow generation |
| Jordan (sound designer) | Game audio, uses Reaper + FMOD. Needs 50 variations | Variation quality, batch workflow, export format | Silent outputs, clipping, weird artifacts |
| Sam (hobbyist) | Learning production in GarageBand. Intimidated by synths | Simplicity, good defaults. "Does it sound pro?" | Confusing UI, crashes, bad defaults |
| Maria (electronic producer) | Makes techno/house. Max/MSP power user. Deep understanding of audio | Latency, uniqueness, "Is this actually useful or a toy?" | Generation that sounds like existing Splice content |
| David (Ableton power user) | Teaches production on YouTube. Has 10k+ sample library | "Can I show this to my students?" Quality bar is high | Anything that sounds cheap or half-baked |

---

## 2. Onboarding Flow

### First Launch (Cold Start)

```
Screen appears: Dark, minimal, centered text input.
No splash screen, no logo animation, no onboarding overlay.

What user sees:
┌─────────────────────────────────────────────────────┐
│                                                     │
│                                                     │
│                                                     │
│          ┌─────────────────────────────────┐        │
│          │  Describe the sound you want... │        │
│          └─────────────────────────────────┘        │
│                                                     │
│          [  ⚡ Generate  ]  [ 📁 Upload   ]         │
│                                                     │
│                                                     │
│                                                     │
│        ┌──────────────────────────────────┐          │
│        │    Your generated sound          │          │
│        │    will appear here              │          │
│        │                                  │          │
│        └──────────────────────────────────┘          │
│                                                     │
│                                                     │
└─────────────────────────────────────────────────────┘
```

No instructions. No tutorial. The input says "Describe the sound you want..." — that's the instruction.

### First 5 Seconds

| Time | User Action | System Response | Feeling |
|------|-------------|----------------|---------|
| 0s | Opens app | Instant window, cursor in input | "Clean, fast" |
| 1s | Types "kick" | Text appears in prompt | "Responsive" |
| 2s | Presses Enter | Input dims, spinner appears on generation area | "It's working" |
| 3s | — | Spinner shows "generating..." text | "How long?" |
| 5s | — | Sound appears: waveform, type badge "KICK", play button | "Oh, it's ready" |
| 6s | Clicks waveform | Sound plays — punchy, clean, usable | "Wait, that's actually good" |
| 7s | Clicks play again | Plays again instantly (cached) | "Fast" |
| 8s | Clicks heart | Heart fills, toast: "Added to favorites" | "Nice, saved" |
| 9s | Clicks export | Native save dialog, saves to desktop | "Professional" |
| 10s | Drags WAV into DAW | It plays. It fits. | "This is useful" |

### The Magic Moment

The first time a user hears a generated sound that makes them stop and go "huh, that's actually good" — that's the magic moment. It should happen within 10 seconds of opening the app.

Everything in the alpha is designed to clear the path to this moment:
- No account required
- No settings to configure
- No model downloads
- No "first, let me explain how AI works"
- Just: type, click, hear, smile

---

## 3. First Upload Flow

```
1. User drags a WAV/MP3 file onto the prompt area
   OR clicks the upload button

2. File appears as a "reference" tag in the prompt bar:
   ┌──────────────────────────────────────┐
   │  │📎 reference_kick.wav│  Type prompt...│
   └──────────────────────────────────────┘

3. File is analyzed (brief spinner, <500ms):
   "Analyzed: 124 BPM, F# minor, punchy"

4. User types prompt with reference present:
   "tight snare that fits this track"

5. Generate: system uses reference as conditioning
   → output is a snare that sits in the same mix

6. User previews, adjusts prompt, regenerates
   "snare with more crack" → new generation
```

---

## 4. Generation & Preview Flow

```
┌─────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────┐   │
│  │  punchy trap kick 140bpm                        │   │
│  └─────────────────────────────────────────────────┘   │
│  [ ⚡ Generate ]                                       │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │ ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐          │  │
│  │ │wavform│  │wavform│  │wavform│  │wavform│          │  │
│  │ │KICK   │  │SNARE  │  │HAT    │  │PERC   │          │  │
│  │ │0.42s  │  │0.31s  │  │0.18s  │  │0.55s  │          │  │
│  │ │★      │  │☆      │  │★      │  │☆      │          │  │
│  │ └──────┘  └──────┘  └──────┘  └──────┘          │  │
│  │ ┌──────┐  ┌──────┐                               │  │
│  │ │wavform│  │wavform│                               │  │
│  │ │FX     │  │SNARE  │                               │  │
│  │ │0.89s  │  │0.29s  │                               │  │
│  │ │☆      │  │☆      │                               │  │
│  │ └──────┘  └──────┘                               │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  Selected Sound Detail:                                │
│  ┌──────────────────────────────────────────────┐      │
│  │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━     │      │
│  │  ● Playing                                    │      │
│  │  KICK · 0.42s · punchy · bright              │      │
│  │  [ ★ Favorite ] [ ⬇ Export ] [ ↻ Variants ] │      │
│  └──────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────┘
```

---

## 5. Save/Export Flow

```
Save (favorite):
  Click ★ → heart fills → sound added to favorites
  Toast: "Added to favorites" (2s, auto-dismiss)
  Undo available in toast: "Undo"
  
Export:
  Click ⬇ → native save dialog appears
  Default: ~/Desktop/{sound_type}_{date}.wav
  User can rename and choose location
  On save: toast "Exported to Desktop/kick_2025-01-15.wav"
  Opens Finder on macOS (optional)
```

---

## 6. Feedback Collection

### In-App Feedback

```
After 3rd generation, show subtle prompt:
  "How's the sound quality?"
  [ 😕 ] [ 😐 ] [ 😊 ] [ 😍 ]
  (3 emoji rating — fast, no friction)

After 1st export:
  "Would you use this in a track?"
  [No] [Maybe] [Yes] [Already did!]

After 5 minutes of use:
  "What's one thing we should improve?"
  [Free text, single line, optional]
  [Dismiss]

After 10 generations (or end of session):
  "Thanks for trying cShot alpha!"
  "Your feedback directly shapes the next version."
  [Share your email for updates] [Join Discord] [Done]
```

### Automated Telemetry (opt-in)

```typescript
interface TelemetryEvent {
  event: 'generation_start' | 'generation_end' | 'preview' | 'export' | 'favorite' | 'error';
  timestamp: string;
  duration_ms?: number;
  sound_type?: string;
  model_name?: string;
  success?: boolean;
  error_message?: string;
}

// Stored locally, uploaded on explicit consent
// No audio data, no prompts — just timing and feature usage
```

---

## 7. Failure Recovery

| Failure | What User Sees | Recovery Path |
|---------|---------------|---------------|
| Generation fails | Red toast: "Generation failed. Check connection." + [Retry] button | Click retry → regenerates. Auto-retry once |
| Export fails | Toast: "Couldn't save file. Try a different location." | User picks new location |
| Audio doesn't play | Nothing (silent failure) | User clicks again. If 2nd fail, toast: "Preview unavailable" |
| App crashes | N/A (app closes) | On next launch: "cShot didn't close properly. Restore session?" → regenerates last prompt |
| Network error (API) | Toast: "Network error. Check your connection." + [Retry] | Retry button |
| Model download fails | Toast: "Model download failed." + [Retry] | Retry with resume |
| Disk full | Toast: "Not enough disk space. Free up space and try again." | User frees space, retries |

---

## 8. First 5 Minutes: What User Should Feel

```
Minute 0-1: "This looks clean and fast. I know what to do."
  - App opens instantly
  - Single input, cursor blinking
  - No clutter, no settings, no accounts

Minute 1-2: "I typed a prompt and got a sound. That was easy."
  - Generation takes 2-5 seconds
  - Sound appears with waveform and type badge
  - Click to hear it immediately

Minute 2-3: "The sound is actually good. Like, usable good."
  - Audio quality surprises them
  - They click play again to verify
  - They think about what they'd use this for

Minute 3-4: "I can save this and take it into my DAW."
  - Favorite click feels satisfying
  - Export is one click → file on desktop
  - They imagine the workflow

Minute 4-5: "I want to try more. What else can I make?"
  - They type a new prompt
  - They try uploading a reference
  - They generate variants
  - They explore
```

### Emotional Journey Map

```
Time    ──────────────────────────────────────────────►
        │      │      │      │      │      │      │
       0      1      2      3      4      5      6+ mins
       
Emotion:
  Curious ──► Hopeful ──► Surprised ──► Delighted ──► Productive
    │          │          │          │          │
    │          │          │          │          │
  "What's  "Will it  "Wait,   "I can   "I'm
   this?"   work?"    that's   save     making
                      good"    this"    music"
```

---

## 9. Alpha-Specific Restrictions

| Restriction | Reason | Communication |
|-------------|--------|---------------|
| Max 10 generations per session | API cost control (ElevenLabs) | "You've used 8/10 generations. Generate more after restart." |
| Limited prompt length (100 chars) | Focus, prevent abuse | Input simply stops accepting text |
| WAV only export | Simplify testing | Only format offered |
| No cloud sync | Alpha scope | "Local only for now." |
| No library browser (v1) | Cut scope | Favorites shown as horizontal scroll |
| Keyboard shortcuts limited | Cut scope | Space + Enter only |

---

## 10. What Users Should NOT See in Alpha

```
✗ Loading spinners longer than 5 seconds
✗ Error messages with technical details ("segmentation fault")
✗ Empty states without guidance ("No sounds yet")
✗ Settings or configuration screens
✗ Account creation or login
✗ Model version numbers
✗ API keys or configuration
✗ "Beta" or "Alpha" watermarks
✗ Console logs or debug output
✗ Unresponsive UI during generation
```

---

## 11. Post-Session Data Collection

After a user session, collect and log:

```json
{
  "session_id": "uuid",
  "duration_minutes": 8.5,
  "generations": 7,
  "exports": 3,
  "favorites": 4,
  "variants_requested": 2,
  "uploads": 1,
  "sound_types_generated": ["kick", "snare", "hat"],
  "generation_times_ms": [3200, 4100, 2800, 5100, 3700, 2900, 3300],
  "errors": [],
  "ratings": [4, 5, 3],
  "feedback_text": "The kicks are great, snares need more crack",
  "prompts": [
    "punchy kick",
    "tight snare",
    "bright hat",
    "deeper kick with more sub"
  ],
  "device_info": {
    "os": "macOS 14.2",
    "memory_gb": 16,
    "cpu": "Apple M1"
  }
}
```

This data tells you what users actually do, not what they say they do.

---

## 12. Summary

The alpha flow is: **Type → Hear → Save → Show a friend.**

Everything else is noise. The alpha app has one screen, one input, one output area, and three buttons (play, favorite, export). Users complete the core loop in under 10 seconds and the magic moment happens in under 30 seconds.

The alpha is not the product. It's a test of the core premise: *"Can I get a usable sound from a text prompt faster than searching my library?"* If the answer is yes for 10 users, build the real product. If the answer is no, figure out why before building more.
