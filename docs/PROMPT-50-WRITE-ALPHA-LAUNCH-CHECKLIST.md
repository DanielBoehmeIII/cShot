# Prompt 50 — Write the Alpha Launch Checklist

Everything that must be true before showing cShot to the first 10 users.

---

## 1. Product Readiness

### Core Loop
- [ ] User can type a prompt and press Enter
- [ ] Generation completes within 10 seconds
- [ ] Generated sound plays on click
- [ ] Waveform thumbnail displays correctly
- [ ] Sound type is detected and displayed
- [ ] Sound can be favorited (heart toggle)
- [ ] Sound can be exported as WAV to user-chosen location
- [ ] Exported WAV opens in DAW/Audacity and plays correctly
- [ ] Reference audio can be uploaded (drag-and-drop or file picker)
- [ ] Generation with reference produces a related sound

### UI/UX
- [ ] App opens in <2 seconds (cold start)
- [ ] No splash screen or startup delay
- [ ] Prompt input is auto-focused on launch
- [ ] Generate button is disabled during generation
- [ ] Loading spinner shows during generation
- [ ] Error states show user-friendly messages (no stack traces)
- [ ] Empty states show helpful guidance
- [ ] Favorites persist after app restart
- [ ] All text is readable (no overflow, no clipping)
- [ ] Window resizes gracefully (minimum 800×600)
- [ ] Dark theme is consistent across all UI elements
- [ ] Cursor changes to pointer on clickable elements

### Performance
- [ ] Cold start: <2 seconds to interactive
- [ ] Generation: <10 seconds (API-dependent)
- [ ] Click-to-play: <50ms
- [ ] Export: <500ms
- [ ] Memory: <200MB idle
- [ ] CPU: <10% idle
- [ ] No frame drops during animation (loading spinner, transitions)

### Edge Cases
- [ ] Empty prompt → show validation
- [ ] Very long prompt (>100 chars) → handled gracefully
- [ ] Very short prompt (1 word) → generates reasonable sound
- [ ] Rapid clicking generate → debounced
- [ ] Rapid clicking play → handles correctly (stops previous)
- [ ] Export to read-only directory → clear error message
- [ ] Export to path with special characters → works
- [ ] App quit during generation → no corruption
- [ ] App reopen after force quit → loads correctly

---

## 2. Technical Readiness

### Build & Packaging
- [ ] App builds successfully: `cargo tauri build`
- [ ] Binary size is reasonable (<50MB before model downloads)
- [ ] Installer works (DMG on macOS, EXE/MSI on Windows, AppImage on Linux)
- [ ] Install on clean machine works (no missing dependencies)
- [ ] App opens from Applications folder / Start menu
- [ ] App icon displays correctly in dock/taskbar
- [ ] App name displays correctly in menu bar
- [ ] Window title is "cShot"
- [ ] Uninstall removes all app files

### Platform Support (Ship to developer's primary OS first)
- [ ] Works on macOS (Intel + Apple Silicon)
- [ ] Works on Windows 10/11
- [ ] Works on Linux (Ubuntu 22.04+ / Fedora)
- [ ] Rosetta 2 compatibility (macOS Intel on Apple Silicon)
- _If only shipping one platform, others are documented as "not yet tested"_

### Security
- [ ] No API keys hardcoded in binary
- [ ] API keys stored in OS keychain or config file
- [ ] No secrets in git history
- [ ] All network calls are HTTPS
- [ ] File system access is sandboxed to app directories
- [ ] No arbitrary code execution from user input
- [ ] Audio files are validated before processing

### Logging & Debugging
- [ ] App logs to file: `~/cShot/logs/app.log`
- [ ] Logs are rotated (max 10MB)
- [ ] No PII in logs
- [ ] Crash reporter captures stack traces
- [ ] Log level is configurable (file or env var)
- [ ] Debug mode available (hidden flag)

### Backup & Recovery
- [ ] Favorites JSON is written atomically (write to temp, rename)
- [ ] Corrupted favorites file doesn't crash app
- [ ] App handles missing audio files gracefully
- [ ] App can recover from partial write

---

## 3. Model/API Readiness

### ElevenLabs SFX API (Alpha Model)
- [ ] API key is configured and working
- [ ] Rate limits are understood (requests/minute, tokens/minute)
- [ ] Error handling for 429 (rate limit) — show retry-after message
- [ ] Error handling for 500 (server error) — show "try again" message
- [ ] Error handling for 401 (auth) — show "check API key" message
- [ ] Timeout configured (15 seconds max)
- [ ] Retry logic: 1 retry on timeout/server error
- [ ] Cost tracking: log cost per generation
- [ ] Budget limit: max 50 generations/day for alpha
- [ ] Audio format handling: model returns WAV/MP3, app converts correctly

### Fallback (DSP-Only Mode)
- [ ] App works without network (uses pre-generated + DSP)
- [ ] Seamless switch: API available → use API, API down → use DSP fallback
- [ ] User is notified: "Using offline mode" (subtle)
- [ ] DSP fallback produces recognizable one-shots

### Generation Quality Checks
- [ ] 10 test prompts produce acceptable sounds
- [ ] 5 reference uploads produce related generations
- [ ] No silent outputs (check: duration >50ms, RMS > -60dB)
- [ ] No clipping outputs (check: peak <= -0.1dBFS after normalization)
- [ ] Output diversity: same prompt + different seeds → different sounds
- [ ] <10% generation failure rate

---

## 4. Audio Quality Checks

### Per-Generation Checks
- [ ] Duration: 50ms – 5000ms (reject outside range)
- [ ] Not silent: RMS > -60dB
- [ ] No DC offset: average sample value < 0.001
- [ ] No clipping: <1% of samples at ±1.0
- [ ] Peak normalized: within 0.5dB of target (-1dBFS)

### Export Quality
- [ ] WAV file opens in Ableton Live
- [ ] WAV file opens in FL Studio
- [ ] WAV file opens in Logic Pro
- [ ] WAV file opens in Audacity
- [ ] WAV file opens in Windows Media Player / QuickTime
- [ ] Sample rate: 44100Hz ± 1Hz
- [ ] Bit depth: 24-bit (or 16-bit as fallback)
- [ ] Channels: mono
- [ ] File duration matches metadata
- [ ] No audible artifacts (clicks, pops, truncation)

### Reference Upload Quality
- [ ] WAV files import correctly (8/16/24/32-bit, 22050-96000Hz)
- [ ] MP3 files import correctly (128/192/320kbps)
- [ ] FLAC files import correctly
- [ ] AIFF files import correctly
- [ ] Files >10MB show processing indicator
- [ ] Files >1 minute are truncated with warning
- [ ] Corrupted files show clear error

---

## 5. Bug Checks (Manual Test Script)

### Flow 1: Basic Generation (run 5 times)
- [ ] Enter → spinner → waveform → play → sound is good
- [ ] Time each generation (should be <10s)
- [ ] No crashes
- [ ] Sound type is correct (kick prompt → kick sound)

### Flow 2: Reference Upload (run 3 times)
- [ ] Drag WAV → processes → shows reference tag
- [ ] Generate with reference → sound relates to reference
- [ ] Upload without prompt → prompt pre-filled from analysis

### Flow 3: Favorites (run 3 times)
- [ ] Click heart → heart fills
- [ ] Click heart again → heart empties
- [ ] Close app → reopen → favorites persist

### Flow 4: Export (run 3 times)
- [ ] Click export → save dialog appears
- [ ] Choose location → file appears at location
- [ ] Open file in DAW → plays correctly

### Flow 5: Edge Cases
- [ ] Empty prompt → validation message
- [ ] Generate twice in rapid succession → second replaces first
- [ ] Click play → click play again quickly → no double-play
- [ ] Close app during generation → no crash
- [ ] Delete audio cache → app recovers gracefully
- [ ] Remove network → app shows offline mode

### Flow 6: Long Session
- [ ] Generate 20 sounds in a row
- [ ] Memory usage stays stable
- [ ] No degradation in performance
- [ ] All 20 sounds play correctly

---

## 6. Legal/Licensing Checks

### App
- [ ] App name "cShot" doesn't infringe on existing trademarks
- [ ] Open-source licenses compiled (Tauri, React, hound, etc.)
- [ ] Third-party assets have correct attribution
- [ ] Privacy policy exists (even for alpha)
- [ ] Terms of use exist (even for alpha)

### API
- [ ] ElevenLabs API terms allow commercial use (for future)
- [ ] API usage complies with rate limits
- [ ] User data is not sent to API beyond prompt + audio
- [ ] API responses are not re-sold or redistributed

### Generated Audio
- [ ] Alpha testers understand generated audio ownership
- [ ] Terms clarify: user owns generated content
- [ ] Terms clarify: cShot is not liable for generated content
- [ ] Terms clarify: model may produce similar outputs for different users

### User Data
- [ ] No audio data is uploaded without explicit consent
- [ ] Telemetry is opt-in (not default)
- [ ] User can delete all local data (clear library option)
- [ ] Privacy policy explains what data is collected

---

## 7. Feedback Forms

### In-App (Auto-Triggered)
- [ ] After 3rd generation: emoji rating (😕 😐 😊 😍)
- [ ] After 1st export: "Would you use this in a track?" (No/Maybe/Yes)
- [ ] After 5 min: "One thing to improve?" (free text, optional)
- [ ] After 10 gens / end of session: feedback prompt + Discord invite

### Post-Session Email (if user provides email)
- [ ] "Thanks for trying cShot alpha — here's what we learned from your session"
- [ ] "What would make you use cShot every day?" (3 question survey)

### Tester Onboarding Email
- [ ] Download link
- [ ] Setup instructions (API key if needed)
- [ ] What to try (suggested prompts)
- [ ] Known issues
- [ ] How to report bugs (Discord / email / GitHub)
- [ ] Feedback form link

### Bug Report Template

```
**What were you doing?**
[e.g., generating a kick drum with prompt "punchy kick 140"]

**What happened?**
[e.g., app crashed, no sound, wrong sound type]

**What did you expect?**
[e.g., a punchy kick drum]

**Screenshot / screen recording:** [link]

**System:** macOS 14.2, M1 Pro, 16GB RAM

**App version:** v0.1.0-alpha

**Steps to reproduce:**
1. Open app
2. Type "punchy kick 140"
3. Press Enter
4. App crashes

**Logs:** [paste from ~/cShot/logs/app.log]
```

---

## 8. Onboarding Materials

### Quick-Start Guide (One Page)

```
# cShot Alpha — Quick Start

**1. Generate a sound**
Type what you want → Press Enter

Examples:
  "punchy kick drum 140bpm"
  "bright trap snare"
  "deep 808 sub hit"
  "shaker loop 120bpm"

**2. Preview**
Click the waveform → Hear the sound instantly

**3. Save it**
❤️ Favorite — save for later
⬇ Export — save as WAV (use in any DAW)

**4. Upload a reference**
Drag a WAV/MP3 onto the prompt bar → Type what you want → Generate

**Tips:**
  - Be descriptive: "dark ambient riser" works better than "riser"
  - Include BPM: "140bpm" helps the model understand the timing
  - Try variations: generate the same prompt multiple times for different results
  - Reference helps: upload a kick from your track to generate sounds that fit

**Known issues:**
  - Generation takes 2-10 seconds (API-dependent)
  - Internet connection required for generation
  - WAV export only (alpha)
```

### Suggested Prompts (For Testers)

```
Kick drums:
  "punchy kick 140bpm"
  "deep 808 sub kick"
  "tight electronic kick, clicky"
  "rock kick, natural, beater attack"

Snares & claps:
  "crack snare, tight, bright"
  "trap snare with clap layer"
  "rimshot, wooden, dry"
  "deep clap, reverb, wide"

Hi-hats:
  "closed hi-hat, tight, bright"
  "open hi-hat, wash"
  "shaker, organic, high"
  "ride cymbal bell, cutting"

FX:
  "riser, tension, electronic"
  "sub hit, deep, long release"
  "impact, cinematic, dramatic"
  "reverse cymbal, swell"
```

---

## 9. Demo Sounds (Pre-Generated)

Ship the app with 5 pre-generated demo sounds so the grid is never empty on first launch:

```
1. punchy_kick.wav — Classic kick, 0.4s
2. crack_snare.wav — Tight snare, 0.3s
3. bright_hat.wav — Closed hat, 0.15s
4. deep_clap.wav — Wide clap, 0.35s
5. sub_bass.wav — 808 sub hit, 1.2s

These are generated, processed, and embedded in the app bundle.
On first launch, they appear in the grid with "demo" tag.
On first generation, demos are replaced.
```

---

## 10. Success Metrics (Must-Have for Alpha)

### Quantitative

| Metric | Target | Minimum |
|--------|--------|---------|
| Generation success rate | >95% | >90% |
| Average generation time | <5s | <10s |
| Click-to-play latency | <20ms | <50ms |
| Export success rate | >98% | >95% |
| App crash rate | 0% | <5% of sessions |
| Users who generate >3 times | >80% | >60% |
| Users who export at least once | >70% | >50% |
| Users who favorite at least once | >60% | >40% |
| Rated 4+ on "would you use this?" | >50% | >30% |

### Qualitative

| Question | Target | Reading |
|----------|--------|---------|
| "Was the generation fast enough?" | >80% yes | Latency is acceptable |
| "Was the sound quality usable?" | >70% yes | Audio quality is good enough |
| "Would you use cShot in your workflow?" | >50% yes | Core premise is validated |
| "What's the most surprising thing?" | Positive responses | Magic moment is happening |
| "What's the worst thing?" | Trackable patterns | Clear improvement priorities |

### Must-Fix Before Alpha

Any tester encountering these means the alpha is not ready:

```
CRITICAL (blocking):
  ❌ App crashes on generation
  ❌ No sound comes out (silent generation)
  ❌ Exported WAV is corrupted
  ❌ App doesn't open on tester's machine

HIGH (fix within 24 hours):
  ❌ Generation takes >20 seconds
  ❌ Play button doesn't work
  ❌ Favorites don't persist
  ❌ Upload doesn't work
  ❌ Obvious audio artifacts (clicks, pops, distortion)

MEDIUM (fix for next tester cohort):
  ❌ Sound type is always wrong
  ❌ Tags make no sense
  ❌ Waveform doesn't match audio
  ❌ Export filename is unhelpful
```

---

## 11. Launch Day Runbook

### Pre-Launch (24 hours before)

```
□ Build release binary for target platform
□ Run full test script (§5) — all pass
□ Verify demo sounds are embedded
□ Verify API key is active and has balance
□ Verify feedback forms are working
□ Verify logs are being captured
□ Send tester invite emails with download link + instructions
□ Set up Discord channel for real-time support
□ Set up GitHub issue label "alpha-feedback"
```

### Launch

```
□ Post download link in Discord
□ Share "First, try typing 'punchy kick 140bpm'"
□ Monitor logs for errors
□ Monitor API usage and cost
□ Be available for questions (monitor Discord)
```

### Post-Launch (48 hours)

```
□ Check all tester feedback
□ Fix critical/high bugs found
□ Review success metrics (§10)
□ Decide: continue to MVP or pivot based on feedback
□ Send thank-you email to testers
□ Share what you learned (public blog post or Discord)
```

---

## 12. Final Check: Would You Show This to Your Mom?

The ultimate alpha readiness test: would you install this on your mom's computer without fear?

```
□ Yes — it's stable, it won't crash, it's usable
□ Yes — the UI is clear enough she'd figure it out
□ Yes — generated sounds are good enough to impress her
□ Yes — there's nothing embarrassing in the app

If any answer is "no", fix it before inviting testers.
```

---

## 13. Summary: What Must Be True Before Launch

1. **The core loop works.** Type → Generate → Hear → Export. Every time. Under 10 seconds.
2. **It doesn't crash.** Zero tolerance for crashes in alpha.
3. **The quality surprises people.** If the first sound doesn't impress, nothing else matters.
4. **You can hear the feedback immediately.** In-app ratings and free-text. No friction.
5. **You know what to do next.** The metrics tell you whether to build the MVP or change direction.

Everything on this checklist serves these five things. Check the boxes. Ship the alpha. Learn.
