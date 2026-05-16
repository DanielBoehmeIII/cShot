# Prompt 52 — Diagnose Bad Generations

A failure taxonomy for bad cShot generations. Classify every failure mode, diagnose its cause, and prescribe the repair.

---

## 1. Failure Taxonomy

### F-01: Muddy

**Perception:** Sound lacks clarity. Low-mid buildup. Transients are buried. Sounds like a blanket is over the speaker.

**Likely Cause:**
- Model generates with too much spectral overlap in 200-800Hz range
- No high-pass filter post-generation
- Reference audio itself is muddy → model mimics the flaw
- Prompt requested "warm" or "dark" without specifying clarity

**Detection Method:**
- Spectral flatness in low-mids >0.7
- Spectral centroid <800Hz with crest factor <8
- Energy ratio (200-800Hz) / (800-4000Hz) > 2.0
- Zero crossing rate <0.08 with duration >300ms

**User-Facing Explanation:**
> "This sound is a bit muddy — the low-mid frequencies are masking the clarity. Try adding 'bright' or 'clean' to your prompt, or use the Clarity control in post-processing."

**Automated Repair:**
```rust
pub fn repair_muddy(audio: &mut [f32], sample_rate: u32) {
    // 1. High-pass filter at 60Hz (gentle slope)
    // 2. Low-mid cut: -3dB shelf at 400Hz, Q=0.7
    // 3. Presence boost: +2dB at 3kHz, Q=1.0
    // 4. Transient enhancement: increase onset gain by 2dB
    // All adjustments are subtle — never more than ±3dB
}
```

**Model/Prompt Improvement:**
- Add "clarity" as a prompt parameter
- Train on cleaner samples — filter out muddy training data
- Post-generation EQ as default processing
- Detect muddy pre-generation by analyzing the latent vector

---

### F-02: Weak Transient

**Perception:** The attack is soft. The sound doesn't cut through. Feels like it was played from another room.

**Likely Cause:**
- Model averages out transients during generation (common diffusion artifact)
- Generation parameters favor smoothness over attack
- Reference audio has compressed/dulled transient
- Sample rate conversion introduced smearing

**Detection Method:**
- Onset strength <0.3 (normalized)
- Attack time >20ms for percussive sounds
- Crest factor <8 for kick/snare/clap
- Spectral flux across onset <0.15

**User-Facing Explanation:**
> "The attack on this sound is weak — it won't cut through a mix. We can sharpen the transient automatically, or add 'sharp' or 'crack' to your prompt."

**Automated Repair:**
```rust
pub fn repair_transient(audio: &mut [f32], sample_rate: u32) {
    // 1. Detect onset position
    // 2. Split audio at onset (transient + tail)
    // 3. Apply transient shaper:
    //    - Boost transient gain by +3dB
    //    - Shorten transient sustain by 40%
    //    - Add 1ms attack, 5ms hold, 10ms release envelope
    // 4. Recombine with crossfade (2ms)
    // Safe limit: never boost transient more than +6dB
}
```

**Model/Prompt Improvement:**
- Train with transient-preserving loss function
- Add "transient strength" as a controllable parameter
- Post-generation transient shaper for all percussive sounds
- Pre-condition diffusion process with transient cues

---

### F-03: Too Long

**Perception:** Sound overstays its usefulness. A kick that rings for 2 seconds. A hat that doesn't close. Too much tail.

**Likely Cause:**
- Model generated with excessive reverb/decay
- No tail detection or trim applied
- Prompt didn't specify duration cues (e.g., "tight", "short")
- Normalization amplified the tail noise floor

**Detection Method:**
- Duration exceeds expected range for detected sound type:
  - Kick: >1.5s
  - Snare: >1.0s
  - Hi-hat: >0.5s
  - Clap: >1.0s
  - Perc: >2.0s
- Tail energy (last 20%) >10% of total energy
- Envelope doesn't reach -60dB within expected time

**User-Facing Explanation:**
> "This sound is longer than expected for a [kick]. The tail might conflict with other elements in your mix. We can trim it automatically, or add 'tight' to your prompt."

**Automated Repair:**
```rust
pub fn repair_too_long(audio: &[f32], sample_rate: u32) -> Vec<f32> {
    // 1. Find the point where envelope drops below -50dB
    // 2. Trim to that point + 50ms buffer
    // 3. Apply 5ms fade-out at new endpoint
    // 4. Verify duration is within expected range
    // If still too long, apply more aggressive gate (threshold: -40dB)
}
```

**Model/Prompt Improvement:**
- Type-specific duration targets in model conditioning
- Automatic trim with type-aware thresholds
- Train duration classifier to reject long samples
- Add "duration_seconds" as a generation parameter

---

### F-04: Too Short

**Perception:** Sound cuts off abruptly. The decay is unnatural. Feels truncated.

**Likely Cause:**
- Model generated truncated output (API bug)
- Tail was incorrectly gated by post-processing
- Prompt implied a shorter sound
- Sample rate mismatch caused incorrect duration math

**Detection Method:**
- Duration below expected range for detected type
- Envelope doesn't reach -20dB at endpoint (sudden cutoff)
- High-frequency energy cuts off before low-frequency (phase issues)
- File ends at a non-zero sample (click artifact)

**User-Facing Explanation:**
> "This sound cuts off earlier than expected. The natural decay was truncated. We can heal the tail automatically."

**Automated Repair:**
```rust
pub fn repair_too_short(audio: &[f32], sample_rate: u32, sound_type: &str) -> Vec<f32> {
    // 1. Detect the cutoff point (sudden drop in envelope)
    // 2. Generate synthetic tail:
    //    - Capture last 50ms of audio
    //    - Model as exponential decay
    //    - Extend by predicted duration for this sound type
    // 3. Crossfade: natural → synthetic over 10ms
    // 4. Apply fade-out on synthetic tail
    // 5. Verify no audible join
}
```

**Model/Prompt Improvement:**
- Validate output duration before returning to user
- Minimum duration floor per sound type
- Reject and regenerate if output is below threshold
- Fix API truncation bugs

---

### F-05: Noisy

**Perception:** Background hiss, static, or artifacts. Sounds like a low-bitrate MP3. Lack of clarity.

**Likely Cause:**
- Model generated with high noise floor
- Compression artifacts from low-bitrate model output
- Reference audio was noisy → model reproduces noise
- Spectral flatness is too high (noise-like instead of tonal)

**Detection Method:**
- Noise floor > -50dB during "silent" sections
- Spectral flatness >0.7 across all bands
- High-frequency energy ratio >0.3 (hiss indicator)
- Harmonic ratio <0.3 (too much noise vs. tone)

**User-Facing Explanation:**
> "There's audible noise in this generation — a hiss or static quality. We can clean it up automatically, or try a different prompt to avoid noise amplification."

**Automated Repair:**
```rust
pub fn repair_noisy(audio: &mut [f32], sample_rate: u32) {
    // 1. Spectral noise gate: estimate noise floor from quietest 10%
    // 2. Apply spectral subtraction (gentle, max -6dB reduction)
    // 3. Low-pass filter above 16kHz (remove ultrasonic noise)
    // 4. Apply expander: 1:1.5 ratio below -40dB threshold
    // Safe limit: never remove more than -6dB of any frequency band
}
```

**Model/Prompt Improvement:**
- Add denoising post-processing step
- Train with noise-augmented data for robustness
- Lower temperature / sampling noise in generation
- Detect and reject high-noise outputs before returning

---

### F-06: Distorted

**Perception:** Clipping, crackling, buzzy, or overdriven. The waveform shows flat-topped peaks.

**Likely Cause:**
- Model generated with peaks >0dBFS (before normalization)
- Post-processing normalization added gain into clipping
- Reference audio was clipped → model learned to clip
- Latent space interpolation caused phase cancellation → peak buildup

**Detection Method:**
- Peak amplitude > 0.99 (before normalization)
- More than 0.1% of samples at ±1.0
- Third harmonic distortion ratio >0.05
- Crest factor <4 (too compressed + clipped)

**User-Facing Explanation:**
> "This sound is distorted — the waveform has been clipped. We can reduce the distortion automatically, or try lowering the 'intensity' in your prompt."

**Automated Repair:**
```rust
pub fn repair_distorted(audio: &mut [f32], sample_rate: u32) {
    // 1. Detect clipping: count samples at ±1.0
    // 2. If <5% of samples clipped:
    //    - Apply soft-clip reconstruction (interpolate clipped regions)
    //    - Reduce gain by 3dB, then re-normalize
    // 3. If >=5% of samples clipped:
    //    - Sound is beyond repair → flag for regeneration
    //    - Return error: "Too distorted to repair, please regenerate"
}
```

**Model/Prompt Improvement:**
- Enforce -3dB headroom in model output
- Headroom normalization (target -1dBFS, not 0dBFS)
- Detect clipping pre-normalization and warn
- Train with headroom constraint in loss function

---

### F-07: Not Prompt-Aligned

**Perception:** The generated sound doesn't match what the user typed. User asked for "kick" and got a snare. User asked for "bright" and got dark.

**Likely Cause:**
- Model misinterprets prompt (CLAP/text encoder failure)
- Prompt contains ambiguous language
- Reference audio overpowers the prompt conditioning
- Generation seed + prompt combination produces unexpected result

**Detection Method:**
- Embedding similarity between prompt and generated audio <0.3 (CLAP score)
- Sound type classifier disagrees with prompt keywords
- Spectral centroid differs by >2 octaves from prompt-predicted centroid
- User behavior: immediate regeneration without preview (withhold play)

**User-Facing Explanation:**
> "This sound doesn't seem to match what you asked for. You typed '[prompt]' but the result sounds more like a [detected_type]. Try being more specific, or add 'exactly' to your prompt."

**Automated Repair:**
```rust
pub fn check_prompt_alignment(prompt: &str, features: &SignalFeatures) -> AlignmentScore {
    // 1. Extract keywords from prompt
    // 2. Predict expected feature ranges for each keyword
    // 3. Compare actual features against expected ranges
    // 4. Return score + suggested fix
    //
    // Example:
    //   Prompt: "bright kick"
    //   Expected: centroid > 3000Hz, sound_type = kick
    //   Actual: centroid = 1200Hz, sound_type = snare
    //   Score: 0.2 — low alignment
    //   Suggestion: "Try 'punchy kick drum' instead"
}
```

**Model/Prompt Improvement:**
- Improve prompt encoder with music-domain fine-tuning
- Add negative prompting ("not a snare, not dark")
- Show prompt interpretation to user before generating
- Implement classifier-free guidance scale for prompt adherence

---

### F-08: Not Reference-Aligned

**Perception:** User uploaded a reference track for context, but the generated sound doesn't fit the reference. Wrong key, wrong timbre, wrong energy.

**Likely Cause:**
- Reference embedding is not being used effectively by the model
- Reference audio is too different from the requested sound type
- Model ignores reference conditioning in favor of prompt
- Reference is too long/complex for the model to parse

**Detection Method:**
- Embedding similarity between reference and generated <0.25
- Spectral centroid difference >2x between reference and generated
- Key/frequency mismatch: reference tonal center vs. generated

**User-Facing Explanation:**
> "This generation doesn't match your reference audio well. Try using a reference that's closer to what you want, or describe the relationship more clearly (e.g., 'a snare that fits this track')."

**Automated Repair:**
- Not repairable automatically — the generation itself is wrong
- Option: re-generate with stronger reference conditioning weight
- Option: blend reference timbre with generated sound

**Model/Prompt Improvement:**
- Increase reference conditioning weight in model
- Improve reference audio embedding extraction
- Add reference similarity score to generation metadata
- Train on reference-conditioned pairs

---

### F-09: Boring

**Perception:** Technically correct but uninspired. Sounds generic, like a default preset. No character.

**Likely Cause:**
- Model averages toward the mean of training data (safest prediction)
- Low temperature / sampling diversity
- Prompt is too generic ("kick" vs "glitchy kick with sub harmonics")
- No reference or style conditioning to add character

**Detection Method:**
- High similarity to training set prototypes (nearest neighbor distance <0.1)
- Low spectral variance across time (too static)
- All features in the 40-60th percentile of their ranges (middle of everything)
- User rated low but didn't give specific complaint

**User-Facing Explanation:**
> "This sound is technically fine but lacks character. Try adding descriptive words like 'glitchy', 'layered', 'aggressive', or 'vintage' to your prompt for more unique results."

**Automated Repair:**
```rust
pub fn repair_boring(audio: &[f32], sample_rate: u32, style_hint: &str) -> Vec<f32> {
    // 1. Analyze which dimensions are "average"
    // 2. Add controlled variation:
    //    - Saturation (1-2% THD, subtle character)
    //    - Filter movement (gentle 0.5dB sweep)
    //    - Stereo widening (if applicable)
    //    - Harmonic excitation (add 2nd/3rd harmonics)
    // 3. Never make it weird — only add character, not artifacts
}
```

**Model/Prompt Improvement:**
- Increase generation diversity (higher temperature, top-k sampling)
- Add "creativity" slider to control diversity vs. accuracy
- Train on more diverse, characterful samples
- Implement style transfer from reference sounds

---

### F-10: Unusable in Mix

**Perception:** The sound sits wrong in every context. Too much low end for a snare. Too bright for a kick. Wrong frequency balance for any role.

**Likely Cause:**
- Sound type is ambiguous — doesn't fill a clear role
- Frequency content spans too wide (sounds like everything at once)
- Dynamic range is inappropriate for the sound type
- Masking profile conflicts with common mix arrangements

**Detection Method:**
- Sound type classifier confidence <0.5 (ambiguous type)
- Energy is spread across all 6 bands with no dominant band (>15% each)
- Crest factor is mismatched with type (high crest on pad, low crest on kick)
- Spectral centroid falls between type-expected ranges

**User-Facing Explanation:**
> "This sound is hard to place in a mix — it doesn't clearly fit any one role. Try specifying what you want more precisely (e.g., 'sub kick, nothing above 100Hz')."

**Automated Repair:**
- Not repairable automatically — regenerate with clearer prompt
- Feature suggestion: show the sound's frequency profile vs. mix role targets
- Guided mode: "What role should this fill?" → Kick/Snare/Hat/Bass/FX/Perc → then shape

**Model/Prompt Improvement:**
- Train type-specific models (one for kicks, one for snares, etc.)
- Add mix-role conditioning to generation
- Generate with frequency mask constraints
- Post-generation EQ to match type profile

---

### F-11: Wrong Genre

**Perception:** User asked for trap and got lo-fi. Asked for techno and got pop.

**Likely Cause:**
- Prompt didn't specify genre strongly enough
- Model defaults to its training distribution (most common genres)
- Reference audio genre overpowers prompt genre
- Genre boundaries are fuzzy in the model's latent space

**Detection Method:**
- Genre classifier output disagrees with prompt genre keyword
- Feature profile matches different genre template
- User feedback: "this doesn't sound like [genre]"

**User-Facing Explanation:**
> "This doesn't sound like the genre you asked for. Try making the genre the first word in your prompt (e.g., 'trap snare, bright' instead of 'snare, bright, trap')."

**Automated Repair:**
- Not repairable — regenerate with genre-weighted prompt
- Option: apply genre-specific EQ/compression template
- Option: show genre match score so user knows to regenerate

**Model/Prompt Improvement:**
- Genre classifier at generation time to steer latent vector
- Genre-conditioned model (LoRA adapters per genre)
- Show genre prediction before generation for user validation
- Collect genre preference data from user ratings

---

### F-12: Too Similar to Source

**Perception:** (When reference audio is used) The generated sound is essentially a copy of the reference with minimal variation. Not useful — user wanted something related but different.

**Likely Cause:**
- Reference conditioning weight is too high
- Model doesn't have enough "creativity" budget when reference is present
- Reference audio already sounds like what user requested

**Detection Method:**
- Cross-correlation between reference and generated >0.8
- Spectral distance <0.1 (nearly identical)
- User audits both and says "these are the same"

**User-Facing Explanation:**
> "The generated sound is very close to your reference — almost a copy. Try setting 'inspired by' rather than 'match this' by adding 'different from reference' or 'variation of' to your prompt."

**Automated Repair:**
- Regenerate with lower reference weight
- Add controlled variation to distinguish from reference
- Suggest different seed or higher creativity setting

**Model/Prompt Improvement:**
- Add reference influence slider (0-100%) in UI
- Train dissimilarity objective (minimize reference similarity while maintaining relevance)
- Detect near-copy and regenerate automatically
- Log copy rate as quality metric

---

### F-13: Too Generic

**Perception:** Sounds like every other AI-generated version of this sound. No unique character. You've heard this exact sound before.

**Likely Cause:**
- Model predicts the mode of the distribution (most common variation)
- Seed doesn't create enough variation for this prompt
- Training data has low diversity for this sound type
- Temperature/diversity parameters are too conservative

**Detection Method:**
- Nearest neighbor distance to training set <0.05
- Output is within 5% percentile range of all features (cocktail-party effect)
- Multiple generations of same prompt produce nearly identical results

**User-Facing Explanation:**
> "This is a very generic version of [sound]. Every generation of this prompt sounds similar. Try adding unusual words to your prompt (e.g., 'glitchy', 'modular', 'processed') or lower the 'accuracy' slider."

**Automated Repair:**
- Regenerate with higher temperature
- Apply randomization to post-processing parameters
- Blend with noise, then denoise (adds subtle variation)
- Randomize EQ curve within type-safe range

**Model/Prompt Improvement:**
- Implement diversity penalty (MMD) in generation
- Track output diversity and flag when variance drops
- Prompt user to try different prompt styles
- Build latent space with better separation

---

## 2. Failure Detection Pipeline

```
Generated Audio
    │
    ▼
┌──────────────────────┐
│ Quick Checks (<1ms)   │
│ • Duration range      │  F-03, F-04
│ • Peak amplitude      │  F-06
│ • DC offset           │  (pre-emptive)
│ • Sample rate         │  (pre-emptive)
├──────────────────────┤
│ Feature Extraction    │
│ (SignalFeatures)      │  ~3ms
├──────────────────────┤
│ Rule-Based Detection  │
│ • Spectral centroid   │  F-01, F-07
│ • Crest factor        │  F-02, F-06
│ • Spectral flatness   │  F-05
│ • Onset strength      │  F-02
│ • Energy distribution │  F-10
│ • Duration by type    │  F-03, F-04
│ • Noise floor         │  F-05
├──────────────────────┤
│ ML-Based Detection    │
│ • CLAP alignment      │  F-07, F-08    ~20ms
│ • Style classifier    │  F-11
│ • Novelty detector    │  F-09, F-13
│ • Copy detector       │  F-12
├──────────────────────┤
│ Decision              │
│ • No issues → pass    │
│ • Minor issues → auto-repair │
│ • Major issues → flag for user │
│ • Critical → auto-regenerate │
└──────────────────────┘
```

---

## 3. Repair Priority Matrix

```
Failure     | Severity | Auto-Repair? | Detection Cost | Repair Cost | User Awareness
────────────|──────────|──────────────|────────────────|─────────────|──────────────
F-01 Muddy  | Medium   | Yes          | Low            | Low         | Show "Clarity Boost applied"
F-02 Weak T | High     | Yes          | Low            | Low         | Show "Transient Sharpened"
F-03 Long   | Medium   | Yes          | Low            | Low         | Show "Trimmed to [duration]"
F-04 Short  | Medium   | Yes          | Low            | Medium      | Show "Tail extended"
F-05 Noisy  | Medium   | Yes          | Low            | Low         | Show "Noise reduced"
F-06 Dist   | High     | Conditional  | Low            | Medium      | Show "Distortion repaired"
F-07 Align  | Critical | No           | Medium         | N/A         | Suggest prompt change
F-08 Ref Al | Critical | No           | Medium         | N/A         | Suggest reference change
F-09 Boring | Low      | Conditional  | High           | Low         | Offer "Make interesting"
F-10 Mix    | High     | No           | Medium         | N/A         | Suggest type refinement
F-11 Genre  | Medium   | No           | Medium         | N/A         | Suggest genre emphasis
F-12 Copy   | Medium   | Yes          | High           | Medium      | Auto-regenerate
F-13 Generic| Low      | Yes          | High           | Low         | Offer "More variety"
```

---

## 4. User-Facing Error Messages

```typescript
const FAILURE_MESSAGES: Record<string, FailureMessage> = {
  'muddy': {
    icon: '🌫️',
    title: 'Sound is a bit muddy',
    description: 'The low-mid frequencies are masking the clarity.',
    suggestion: 'Try adding "bright" or "clean" to your prompt.',
    action: 'Apply Clarity Boost',
    autoFixable: true,
  },
  'weak_transient': {
    icon: '💨',
    title: 'Attack could be sharper',
    description: 'The transient isn\'t strong enough to cut through a mix.',
    suggestion: 'Try adding "sharp", "crack", or "punchy" to your prompt.',
    action: 'Sharpen Transient',
    autoFixable: true,
  },
  'distorted': {
    icon: '🔊',
    title: 'Sound is clipping',
    description: 'The waveform has been driven past the clean limit.',
    suggestion: 'Try reducing intensity in your prompt or regenerating.',
    action: 'Try to Repair',
    autoFixable: true, // conditional on severity
  },
  'not_aligned': {
    icon: '🎯',
    title: 'Doesn\'t match your prompt',
    description: 'The result sounds different from what you described.',
    suggestion: 'Try being more specific, or add "exactly" to your prompt.',
    action: 'Regenerate',
    autoFixable: false,
  },
  'boring': {
    icon: '😐',
    title: 'A bit generic',
    description: 'Technically correct but lacks character.',
    suggestion: 'Try descriptive words like "glitchy", "aggressive", or "vintage".',
    action: 'Make More Interesting',
    autoFixable: true,
  },
};
```

---

## 5. Failure Log & Analytics

```sql
-- Every detected failure is logged
CREATE TABLE failure_log (
    id              TEXT PRIMARY KEY,
    sound_id        TEXT NOT NULL,
    failure_codes   TEXT NOT NULL,           -- JSON array: ["F-01", "F-02"]
    severity        TEXT NOT NULL,           -- 'low', 'medium', 'high', 'critical'
    detection_method TEXT,                    -- 'rule', 'model', 'user_report'
    auto_repair_applied INTEGER DEFAULT 0,
    auto_repair_success INTEGER,             -- NULL if not applied
    user_action      TEXT,                    -- 'kept', 'regenerated', 'exported_anyway', 'deleted'
    created_at       TEXT NOT NULL,
    FOREIGN KEY (sound_id) REFERENCES sounds(id)
);

-- Failure rate by type
CREATE VIEW failure_rates AS
SELECT
    failure_codes,
    COUNT(*) AS occurrences,
    AVG(CASE WHEN user_action = 'exported_anyway' THEN 1.0 ELSE 0.0 END) AS export_rate_after_failure,
    AVG(CASE WHEN auto_repair_success = 1 THEN 1.0 ELSE 0.0 END) AS repair_success_rate
FROM failure_log
GROUP BY failure_codes
ORDER BY occurrences DESC;

-- Weekly failure trends
CREATE VIEW weekly_failures AS
SELECT
    DATE(created_at) AS week,
    failure_codes,
    COUNT(*) AS count
FROM failure_log
GROUP BY week, failure_codes
ORDER BY week DESC;
```

---

## 6. Summary

Every failure mode is classified, detectable, and has a prescribed response. Rule-based detection catches the common issues (muddy, weak, long, short, noisy, distorted) in <5ms. ML-based detection catches alignment and style issues in <30ms. Auto-repair fixes minor failures transparently. Major failures trigger user-facing messages with actionable suggestions. The log tracks every failure, its repair, and the user's response, building a database of what actually matters to users.
