# Prompt 37 — Design the cShot Interface

Not a sample browser. A sound laboratory. A futuristic instrument. A creative co-pilot.

---

## 1. Design Philosophy

### The Feeling

```
cShot should feel like:
  - Standing at a lab bench, not browsing a website
  - Playing an instrument, not filling out a form
  - Exploring a universe, not clicking through folders

Visual language:
  - Dark, deep, infinite background
  - Glowing, responsive elements
  - Subtle particle/energy effects for generation
  - Waveforms as living organisms
  - Sound as matter you can sculpt

Every interaction should feel like you're working with physical material,
not digital files. Grabbing, shaping, stretching, combining.
```

### Anti-Patterns (What cShot is NOT)

```
✗ Not a search results page (Splice)
✗ Not a form with parameters (early VSTs)
✗ Not a spreadsheet of files (Finder)
✗ Not a flat grid of thumbnails (generic browser)
✗ Not skeuomorphic (we don't need fake wood paneling)
✗ Not minimal to the point of being abstract
```

---

## 2. Screen Layout

### Main Screen (Default State)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ★ cShot                                                            ≡    │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ [ Prompt: "punchy kick, 140bpm, C minor...              ] [⚡ Gen] │ │
│ │ [🔊 Reference drag zone or mic input]                            │ │
│ └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│ ┌────────────────────────────────────────────────────────────────────┐  │
│ │ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐          │  │
│ │ │  1   │ │  2   │ │  3   │ │  4   │ │  5   │ │  6   │          │  │
│ │ │ █▁▃▇▆ │ │ █▁▃▇▆ │ │ █▁▃▇▆ │ │ █▁▃▇▆ │ │ █▁▃▇▆ │ │ █▁▃▇▆ │    │  │
│ │ │▁▂▃▄▅▆▇│ │▁▂▃▄▅▆▇│ │▁▂▃▄▅▆▇│ │▁▂▃▄▅▆▇│ │▁▂▃▄▅▆▇│ │▁▂▃▄▅▆▇│    │  │
│ │ │ kick  │ │ snare│ │ clap │ │ hat  │ │ perc │ │ fx   │          │  │
│ │ │ ★ ♫ → │ │ ★ ♫ → │ │ ★ ♫ → │ │ ★ ♫ → │ │ ★ ♫ → │ │ ★ ♫ → │    │  │
│ │ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘          │  │
│ │ [More variants...]  [New generation]                             │  │
│ └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│ ┌──────────────────────────────────────┬───────────────────────────────┐│
│ │ Sound Detail Panel                   │ Variation Tree               ││
│ │ ┌────────────────────────────────┐  │    ┌────┐                     ││
│ │ │ Waveform / Spectrogram Viewer  │  │ ┌──→│ S1 │──┐                 ││
│ │ │ [glowing visualization]       │  │ │   └────┘  │                 ││
│ │ └────────────────────────────────┘  │ │   ┌────┐ │                 ││
│ │ ┌──────┬─────────────────────────┐  │ ├──→│ S2 │→┤─ ─ ─ ─ ┐        ││
│ │ │ Tags │ kick, dark, 140bpm, Cm  │  │ │   └────┘ │         │        ││
│ │ ├──────┼─────────────────────────┤  │ │   ┌────┐ │         │        ││
│ │ │ DNA  │ [radar/spider chart]    │  │ └──→│ S3 │─┘         │        ││
│ │ ├──────┼─────────────────────────┤  │     └────┘           │        ││
│ │ │ Hist │ [Variant lineage]       │  │     ┌────┐           │        ││
│ │ ├──────┼─────────────────────────┤  │ └───→│ S4 │──────────┘        ││
│ │ │ Safe │ 🟢 Commercial-safe 97%  │  │      └────┘                   ││
│ │ └──────┴─────────────────────────┘  │                               ││
│ └──────────────────────────────────────┴───────────────────────────────┘│
│                                                                          │
│ ┌────────────────────────────────────────────────────────────────────┐  │
│ │ Latent Space Navigator  [⊙⊙⊙⊙⊙⊙]  Temp [───●──]  Seed [424242]    │  │
│ │ [Coords: (0.42, -0.13, 0.87)]  [Navigate...]                      │  │
│ └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│ Status: 2.3s gen  |  Model v1.2.3  |  140 BPM / C# minor  |  🔊  ───●─ │
└──────────────────────────────────────────────────────────────────────────┘
```

### Screen Zones

```
1. TOP BAR — Global controls
   - Logo / app name
   - Mode toggle (Beginner/Expert)
   - Settings
   - Library browser toggle
   - Project save/load
   - Cloud sync status

2. PROMPT ZONE — Input
   - Prompt text box (auto-expanding)
   - Reference audio drag zone
   - Voice input button (speak your prompt)
   - Recent prompts dropdown
   - Quick presets: "Kick", "Snare", "Hat", "Clap", "Perc", "Bass", "FX"

3. SOUND GRID — Results
   - 6 primary slots (customizable: 4-12)
   - Waveform thumbnail per sound
   - Type tag + auto-tags
   - Quick action buttons (★ favorite, ♫ play, → variants)
   - Generation progress indicator (while loading)
   - Drag source for DAW export
   - Multi-select for batch operations

4. DETAIL PANEL — Selected sound
   - Waveform display (zoomable, scrollable)
   - Spectrogram overlay (toggleable)
   - Sound DNA radar chart (punch, warmth, brightness, space, texture)
   - Auto-tags (editable)
   - Provenance summary
   - Safety/commercial score
   - Parameter controls (ADSR, tone, pitch, etc.)
   - Quick actions: export, favorite, variant, delete

5. VARIATION TREE — Lineage
   - Graph visualization of sound family
   - Parent → child → variant relationships
   - Highlight current selection
   - Click to preview any node
   - Branch, merge, mutate controls
   - Compare mode (A/B selected variants)

6. LATENT CONTROLS — Advanced
   - Latent space coordinate display (2D/3D)
   - Navigation joystick / pad
   - Temperature, CFG scale, seed
   - Model selector (if multiple)
   - Interpolation slider (between two sounds)
   - Random walk / explore mode

7. STATUS BAR — System state
   - Generation status and timing
   - Model version
   - DAW context (BPM, key)
   - Audio level meter
   - Connection status (local/cloud)
   - Recent notifications
```

---

## 3. Interaction Flow

### Primary Flow: Generate a Sound

```
Step 1: ENTER PROMPT
  - Click prompt box
  - Type: "punchy trap kick with short decay"
  - Auto-complete appears (from history + suggestions)
  - OR: drag reference audio to reference zone
  - OR: click microphone icon and speak

Step 2: GENERATE
  - Press Enter or click ⚡ Generate
  - Sound grid shows: [▮▮▮▮▮▮▮▮▮▮] 0% → 100%
  - Generation takes 2-5 seconds
  - First slot fills with result
  - Subtle glow/energy animation during generation

Step 3: PREVIEW
  - Auto-preview plays (or wait for click)
  - Click sound slot → instant playback
  - Waveform animates with playback position
  - Sound continues until: click stop, click different sound, or complete

Step 4: ITERATE
  - Like it? → ★ favorite (adds to library)
  - Want variations? → → variants (generates 6 more)
  - Too similar to each other? → adjust temperature
  - Want different type? → click type tag and regenerate

Step 5: EXPORT
  - Drag sound slot to DAW (if plugin mode)
  - Or: click ↓ export → choose format and location
  - Or: add to pack builder for batch export
```

### Secondary Flow: Work with Reference

```
Step 1: IMPORT REFERENCE
  - Drag audio file to reference zone
  - Or: click "Upload" and select file
  - Or: record directly (click microphone → record → stop)
  
Step 2: ANALYZE
  - cShot analyzes: BPM, key, spectral content, transient profile
  - Shows analysis results: "128 BPM, F# minor, punchy transient"
  - Auto-fills prompt with detected characteristics
  
Step 3: GENERATE-NEAR
  - Click "Generate Similar" → sounds in ballpark of reference
  - Click "Generate Opposite" → contrasting sounds
  - Click "Extract DNA" → analyze and suggest new directions
  - Click "Morph" → transition from reference to generated
```

### Tertiary Flow: Explore Latent Space

```
Step 1: OPEN LATENT NAVIGATOR
  - Toggle latent panel (Ctrl+L in Expert mode)
  - Shows 2D/3D projection of current latent region
  
Step 2: NAVIGATE
  - Click/drag in latent space → real-time preview
  - Sound morphs continuously as you drag
  - "Interesting" regions highlighted with glow
  - Can drop pins at interesting coordinates
  
Step 3: CAPTURE
  - Click "Capture" → current latent position saved as a sound
  - Added to sound grid as new slot
  - Can return to this position later

Step 4: PATH RECORDING
  - Click "Record" → move through latent space
  - Path recorded as series of sounds
  - Export as sample pack or evolving texture
```

---

## 4. Visual Language

### Color Palette

```
Background: #0A0A0F (near-black with blue tilt)
Surface:    #14141F (dark panel)
Surface 2:  #1E1E2E (slightly lighter)
Border:     #2A2A3F (subtle separation)

Primary:    #6C5CE7 (purple — generation energy)
Secondary:  #00D2D3 (cyan — sound/audio)
Accent:     #FDCB6E (amber — favorite/warning)
Success:    #00B894 (green — commercial-safe)
Error:      #D63031 (red — issues)
Text:       #DFE6E9 (light)
Text Dim:   #636E72 (muted)

Waveform:   Cyan gradient (#00D2D3 → #6C5CE7)
Glow:       #6C5CE7 at 30% opacity
Grid line:  #2A2A3F at 50% opacity
```

### Typography

```
Headings:    JetBrains Mono (bold, uppercase, letter-spaced)
Labels:      JetBrains Mono (regular)
Values:      JetBrains Mono (tabular figures for numbers)
Body:        Inter (for any prose/descriptions)

Hierarchy:
  - Sound type tag: 10px uppercase, bold, letter-spaced, dim
  - Sound name: 14px medium
  - Prompt text: 16px light
  - Parameter value: 12px mono, accent color
  - Waveform labels: 9px mono, dim
```

### Animation Language

```
Generation:
  - Pulsing glow on active generation slot
  - Energy particles flowing into the slot
  - Smooth progress fill, not a spinner

Preview playback:
  - Waveform highlight follows playback position
  - Subtle pulse on transient hits
  - Playhead as a glowing vertical line

Transitions:
  - Sound grid: smooth fade-in on generation complete
  - Panels: slide in/out, 200ms ease
  - Modal: backdrop blur + scale-in
  - Navigation: instant, no animations on core UI

Hover/Active:
  - Sound slot: slight scale (1.02), border glow
  - Buttons: color shift, subtle grow
  - Drag: slot becomes "floating," shadow follows cursor
  - Sliders: value readout follows handle
```

### Iconography

```
Custom icon set, monoline style, 1.5px stroke:
  - Generate: ⚡ lightning bolt
  - Favorite: ★ star (filled/unfilled)
  - Play: ▶ play triangle
  - Variants: ↻ branching arrow
  - Export: ↓ download arrow
  - Tag: # hash
  - DNA: 🧬 helix
  - Latent: ⊙ target
  - History: ↶ clock
  - Settings: ≡ menu
  - Reference: 🔊 speaker waves
  - Safety: 🛡 shield
  - Interpolate: ⟷ double arrow
  - Compare: ⊞ overlapping squares
```

---

## 5. Sound Preview Behavior

### Playback System

```
Trigger latency: <5ms from click to audible output
Buffer: 4096 samples pre-loaded per slot (always ready)
Crossfade: 10ms linear crossfade between different sounds

Interaction models:
  Click:       Play from start. Click again → stop. Click different → crossfade.
  Double-click: Play and loop. Double-click again → stop loop.
  Shift+click:  Add to compare list (A/B mode).
  Right-click:  Context menu (provenance, export, tag, delete).
  Drag:         Initiate drag-to-export (to DAW or file system).

While playing:
  - Waveform animates with playback position
  - Playhead is a glowing vertical line
  - Sound type tag pulses subtly
  - Audio level meter in status bar shows output
  - Loop indicator shown if looping
```

### Preview Queue

```
Batch preview mode:
  - Click "Play All" button above grid
  - Sounds play sequentially with configurable gap (1-5s)
  - Current slot highlighted with moving border
  - Can skip (→ key) or pause (Space)
  - Shows "Previewing 3/6" indicator
  
  Auto-stop after 3 full cycles (configurable)
```

### MIDI Preview

```
In plugin mode (with MIDI input):
  - Sounds map to MIDI notes (C3-B3 for 6-slot grid)
  - Velocity maps to preview volume
  - Pitch bend bends the sound ±12 semitones
  - Aftertouch → filter cutoff
  - Mod wheel → effect send
  
  Sound plays as long as MIDI note is held.
  (For one-shots: play full sound on note-on, 
   allow retrigger on each note-on)
```

---

## 6. Keyboard Shortcuts

### Global Shortcuts

```
Space        Play/pause preview of selected sound
Enter        Generate from current prompt
Escape       Clear selection / close modal / stop playback
Ctrl+P       Focus prompt box
Ctrl+Enter   Generate with current prompt (from anywhere)
Ctrl+,       Open settings

Ctrl+1-6     Select sound slot 1-6
Ctrl+S       Save current project
Ctrl+O       Open project
Ctrl+E       Export selected sound(s)
Ctrl+Z       Undo last generation/action
Ctrl+Shift+Z Redo

F1           Toggle beginner/expert mode
F2           Rename selected sound
F5           Regenerate selected sound (new seed)
F11         Fullscreen toggle
```

### Navigation

```
Tab          Next control/panel
Shift+Tab    Previous control/panel
→ / ←        Navigate sound grid horizontally
↑ / ↓        Navigate sound grid vertically
Home         Select first sound
End          Select last sound
Ctrl+F       Search/filter library
```

### Sound-Specific

```
F            Toggle favorite
V            Generate variants (of selected)
D            Download / export
T            Edit tags
Delete/Backspace Remove selected sound from grid
Ctrl+C       Copy sound to internal clipboard (WAV on paste)
Ctrl+V       Paste sound from clipboard

/            Quick tag filter (focus tag search)
?            Show keyboard shortcuts reference
```

### Latent Controls (Expert Mode)

```
L            Toggle latent controls panel
Ctrl+↑/↓     Adjust temperature
Ctrl+←/→     Adjust CFG scale
R            Random walk in latent space
P            Pin current latent position
[ / ]        Cycle through model versions
```

---

## 7. Beginner vs Expert Modes

### Beginner Mode

```
What's visible:
  - Prompt bar (large, friendly placeholder text)
  - 4 sound slots (not 6)
  - Simple action bar per sound: ♫ play, ★ favorite, ↓ export
  - Basic waveform view (no spectrogram)
  - Generation progress as simple percentage
  - Big, obvious buttons

What's hidden:
  - Latent controls completely hidden
  - Variation tree hidden (replaced by "More like this" button)
  - No parameter knobs (DSP controls hidden)
  - No spectrogram option
  - No provenance details (just safe/not-safe badge)
  - No multi-select
  - No batch operations
  - No keyboard shortcuts (but shown as tooltips)

What's simplified:
  - Tags: auto-generated only, not editable
  - Export: one button, default format
  - Context: shown as simple text ("140 BPM · C# minor")
  - Safety: just a green checkmark

Beginner tutorial:
  - First launch: overlay with 3 tips
    1. "Type what you want to hear"
    2. "Click ♫ to preview"
    3. "Drag to your DAW to use"
  - Can dismiss permanently
  - Accessible from help menu
```

### Expert Mode

```
What's added:
  - Full prompt bar with modifiers (/bpm=140, /key=Fm, /seed=42)
  - 6-12 sound slot grid (configurable rows/columns)
  - Full parameter panel per sound (ADSR, filter, pitch, fx chain)
  - Latent navigator (2D/3D space visualization)
  - Variation tree (full graph)
  - Spectrogram overlay on waveform
  - Sound DNA radar chart
  - Full provenance card
  - Safety confidence breakdown
  - Multi-select with batch operations
  - Custom export presets
  - Scriptable via MIDI CC, OSC, or CLI
  - Editable auto-tags
  - Interpolation between any two sounds
  - Raw waveform editing (trim, fade, normalize)
```

### Adaptive Mode

```
cShot can detect user expertise:

  Signals:
    - How often you adjust parameters
    - Whether you use keyboard shortcuts
    - If you open latent controls
    - How many sounds you generate per session
    - If you edit tags or accept defaults
    
  Adaptation:
    - After 5 sessions without latent controls → stays Beginner
    - After 3 sessions with parameter tweaking → suggests Expert
    - User can always override in settings
    - Per-session mode memory (switch for specific tasks)
```

---

## 8. Micro-Interactions

### Generation Animation

```
When user clicks Generate:
  1. Prompt bar pulse once (acknowledgment)
  2. First empty slot shows progress ring:
     - Outer ring fills 0→100% over generation time
     - Inner shows "2.3s" elapsed time
     - Subtle particle glow around slot
  3. On complete:
     - Slot glows brighter for 500ms
     - Waveform renders left-to-right (100ms animation)
     - Tags appear with typewriter effect (200ms)
     - Quick actions fade in (100ms)
     - Auto-preview starts (100ms after complete)
```

### Drag-to-Export

```
When user starts dragging:
  1. Sound slot shrinks slightly (0.95x)
  2. Cursor changes to drag icon
  3. Slot becomes "floating" with shadow
  4. If dragging to DAW:
     - DAW highlights drop zone (if available)
     - File format indicator appears
  5. On drop:
     - Slot returns to normal
     - Brief checkmark animation
     - "Exported: kick_140bpm_C#m.wav" toast (2s)
     - Sound added to recent exports list
```

### Favorite Animation

```
Click ★:
  1. Star scales up (1.5x) and glows
  2. Particle burst (golden, 300ms)
  3. Star fills with accent color
  4. "Added to favorites" toast (1.5s)
  5. Favorites counter increments

Unfavorite (click filled ★):
  1. Star dims and scales down
  2. Reverse particle effect (300ms)
  3. "Removed from favorites" toast (1.5s)
```

---

## Summary

| Element | Beginner Mode | Expert Mode |
|---------|--------------|-------------|
| Prompt bar | Simple text | Text + modifiers |
| Sound grid | 4 slots | 6-12 slots |
| Waveform | Basic | + Spectrogram overlay |
| Parameters | Hidden | Full ADSR + fx chain |
| Latent controls | Hidden | Full navigator |
| Variation tree | "More like this" button | Full graph |
| Provenance | Safe badge | Full card |
| Tags | Auto only | Editable |
| Export | Default settings | Custom presets |
| Shortcuts | Tooltips shown | Full keyboard control |

The interface should disappear when you're working. Every element earns its place. If it doesn't help you make a sound faster, it doesn't belong.
