# cShot Demo Walkthrough

**Duration:** 3 minutes
**Goal:** Show a producer going from idea to usable sound in under 10 seconds.

---

## Setup

1. Open cShot (dark UI, single prompt bar centered)
2. Cursor is in the input field — ready to type

---

## Flow 1: Basic Generation (60 seconds)

### Step 1: Type a prompt
Type: `punchy kick 140`

Press Enter.

### Step 2: Generation
- Loading spinner appears: "generating your sound..."
- After 2-5 seconds, SoundCard appears with:
  - Waveform SVG (purple/cyan gradient)
  - Type badge: "KICK"
  - Duration: ~420ms
  - Score badge: 87 (green)

### Step 3: Preview
Click the waveform area.

- Sound plays instantly through speakers/headphones
- Waveform animates with a playback cursor
- Play button becomes Stop button

### Step 4: Favorite
Click the heart icon (♡ → ♥).

- Heart fills gold
- Footer updates: "kick · 1 faved · 5 variants"

### Step 5: Export
Click "Export" button.

- Toast appears: "Exported to Desktop"
- WAV file saved to ~/Desktop

### Step 6: Variants
Below the SoundCard, 5 variant cards appear:

| Index | Variant | Character |
|-------|---------|-----------|
| 2 | Trimmed | Shorter, tighter |
| 3 | Repitched | Slightly higher |
| 4 | Reversed | Backwards transient |
| 5 | Saturated | Gritty, distorted |
| 6 | Shortened | Staccato |

Click any variant to preview. Favorite or export individual variants.

---

## Flow 2: Reference Upload (60 seconds)

### Step 1: Upload a reference
Click the "WAV" button next to the prompt input, or drag a WAV file.

### Step 2: Reference analysis
- ReferenceCard appears with waveform, duration, sample rate, format
- Reference is tagged with filename in teal
- Prompt placeholder changes to "describe the variant..."

### Step 3: Generate from reference
Type: `same character, snappier attack`

Press Enter.

- New sound inherits reference character
- Comparison: play reference → play generated → hear the relationship

### Step 4: Clear reference
Click "clear" on ReferenceCard.

- Reference removed
- Prompt returns to normal mode

---

## Flow 3: Library Browsing (60 seconds)

### Step 1: Switch to Library
Click "Library" tab.

### Step 2: Browse sounds
- All generated sounds listed with waveform thumbnails
- Search bar: type `kick` to filter
- Type filter dropdown: select "Kick" to narrow

### Step 3: Manage sounds
- Play any sound from library
- Favorite/unfavorite
- Export individual sounds
- Delete unwanted sounds (trash icon)

### Step 4: Return to Generate
Click "Generate" tab.

---

## Suggested Demo Script

### 30-Second Pitch Demo
```
1. Type "punchy kick 140" → Enter (3 sec gen)
2. Click waveform → hear the kick
3. Click heart → favorite it
4. Click Export → file on desktop
5. "That's it. 5 seconds. Mix-ready. Unique to you."
```

### 3-Minute Full Demo
```
1. Generate: "punchy kick 140" → preview → fav → export
2. Generate: "crack snare, tight, bright" → preview → export
3. Generate: "closed hi-hat, tight, bright" → preview → export
4. Open library → show all 3 sounds organized
5. "Three sounds. One kit. 60 seconds."
```

### Reference Demo
```
1. Upload a kick reference WAV
2. Type "snappier, less sub" → generate
3. Play reference → play generated → show the relationship
4. "cShot understands your reference and generates variations."
```

---

## What to Emphasize

| Point | Say This |
|-------|----------|
| Speed | "5 seconds from idea to hearing a sound" |
| Uniqueness | "You're not finding a sample. You're creating one." |
| Mix-readiness | "Normalized, trimmed, faded. Drop it into your DAW as-is." |
| No browsing | "No scrolling through 50,000 kicks. Describe and done." |
| Variants | "One prompt → 6 distinct variations instantly." |
| Reference | "Upload your track. cShot generates sounds that fit." |

## Known Demo Limitations

| Area | Current Behavior | Future Improvement |
|------|-----------------|-------------------|
| Generation speed | 2-5 seconds (mock DSP) | Target <1s |
| Sound quality | Basic DSP synthesis | Real model inference |
| Variant count | 5 variants | Unlimited with controls |
| Reference handling | Manual upload | Drag-drop from DAW |
| Export dialog | No format selection | Format options dialog |
