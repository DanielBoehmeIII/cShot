# Prompt 87 — Teach Users Sound Design

cShot as a learning tool. Every interaction teaches sound design. Users don't just generate sounds — they understand how sounds are built.

---

## 1. Learning Objectives

### Sound Design Curriculum

```
Level 1 — Foundations (what is a one-shot?)
  • What is a waveform? (visual + audible)
  • What is a transient? (the "attack" of a sound)
  • What is a spectral profile? (frequency content)
  • Understanding envelope: ADSR

Level 2 — Core Skills (how to shape sound)
  • Transient shaping (making sounds punchier/softer)
  • Equalization (changing frequency content)
  • Saturation (adding harmonics, warmth, distortion)
  • Compression (controlling dynamics)
  • Noise and texture (adding character)

Level 3 — Genre Conventions (what sounds belong where)
  • Kick design by genre (trap vs. house vs. lo-fi)
  • Snare design by genre
  • Hi-hat patterns and texture
  • FX design (impacts, risers, downlifters)

Level 4 — Advanced Production (mix-ready sounds)
  • Layering (combining multiple sounds)
  • Sample selection (why certain samples work together)
  • Mix placement (how a sound fits in a full mix)
  • Reference tracking (comparing to pro productions)

Level 5 — Mastery (creating your own sonic identity)
  • Developing personal sound signature
  • Breaking genre conventions intentionally
  • Sound design for emotional impact
  • Building sample packs with cohesion
```

---

## 2. Explainable Controls

### Transparent DSP

```
Traditional approach:
  [Drive: ████░░░░░] [Mix: ██████░░░░]
  → User turns knobs, doesn't know what they do

cShot approach:
  "Saturation adds harmonic overtones to your sound.
   More drive = more harmonics = richer, grittier sound.

   Current setting: drive=4.2
   Effect: adding 3rd and 5th harmonics at -12dB below fundamental
   → This will make your kick cut through a dense mix"

   ┌─────────────────────────────────────────────┐
   │  SPECTRUM: BEFORE                    AFTER  │
   │                                             │
   │  ████▒▒▒▒░░░░      ██████████▒▒▒▒▒▒░░░░    │
   │  60Hz 200Hz  1k   60Hz 200Hz 500Hz 1k  4k  │
   │                                             │
   │  New harmonics: 180Hz (3rd), 300Hz (5th)   │
   └─────────────────────────────────────────────┘
```

### Explainable Control System

```rust
/// Every parameter in cShot has an explanation attached.
/// Shown on hover, focus, or when user clicks the "?" icon.

struct ExplainableParameter {
    /// The actual DSP parameter
    dsp_parameter: DSPParameter,

    /// Human-readable label
    label: String,                      // "Drive"

    /// What this parameter does (1 sentence)
    one_liner: String,                  // "Adds harmonic overtones"

    /// Detailed explanation (2-3 sentences)
    explanation: String,                // "Saturation adds harmonics above...\nMore drive = more harmonics..."

    /// Visual explanation data
    visual_aid: Option<VisualAid>,     // Before/after spectrogram, waveform overlay

    /// Related concepts (for "learn more" links)
    related_concepts: Vec<String>,      // ["Harmonics", "Distortion", "Clipping"]

    /// Current effect on the sound (computed from audio)
    computed_effect: String,            // "Adding 3rd harmonic at 180Hz, -15dB"
}

struct VisualAid {
    aid_type: VisualAidType,  // SpectrogramOverlay, WaveformOverlay, FrequencyResponse
    before_data: Vec<f32>,    // Audio analysis before
    after_data: Vec<f32>,     // Audio analysis after (estimated or computed)
    highlight_region: Option<(f32, f32)>,  // x-axis range to highlight
    annotation: String,       // "Notice the new harmonics here"
}
```

### Concept Graph

```python
# Every sound design concept is linked to related concepts.
# Forms a knowledge graph for the AI tutor.

CONCEPT_GRAPH = {
    "transient": {
        "description": "The initial attack of a sound, before it settles into its sustain",
        "level": 1,
        "related": ["attack_time", "envelope", "transient_shaper"],
        "visual:": "waveform_highlight_first_10ms",
        "listen_for": "the 'click' or 'hit' at the very beginning",
    },
    "envelope": {
        "description": "How a sound's volume changes over time (Attack, Decay, Sustain, Release)",
        "level": 1,
        "related": ["attack_time", "decay_time", "sustain_level", "release_time"],
        "visual": "adsr_envelope_overlay",
    },
    "saturation": {
        "description": "Adding harmonic overtones by non-linear waveshaping",
        "level": 2,
        "related": ["harmonics", "distortion", "tape_saturation", "tube_warmth"],
        "prerequisites": ["spectrum", "harmonics"],
        "listen_for": "the sound becoming 'fuller' or 'grittier'",
    },
    "frequency_spectrum": {
        "description": "The distribution of energy across bass, mids, and treble",
        "level": 1,
        "related": ["spectrogram", "spectral_centroid", "eq"],
        "visual": "spectrogram",
    },
    "compression": {
        "description": "Reducing dynamic range by attenuating loud parts",
        "level": 2,
        "related": ["threshold", "ratio", "attack", "release", "makeup_gain"],
        "prerequisites": ["envelope", "dynamics"],
        "listen_for": "the sound becoming more 'pressed' or 'controlled'",
    },
    # ... 100+ concepts
}
```

---

## 3. Before/After Lessons

### Interactive Comparison

```
┌──────────────────────────────────────────────────────────┐
│  LESSON: What Does EQ Do?                                │
│                                                          │
│  "EQ (Equalization) lets you boost or cut specific        │
│   frequency ranges. Let's hear what happens when we       │
│   cut the low end of this kick."                          │
│                                                          │
│  ┌─────────────────────┐  ┌─────────────────────┐        │
│  │  BEFORE (no EQ)      │  │  AFTER (cut 200Hz)  │        │
│  │                      │  │                      │        │
│  │  [Waveform]          │  │  [Waveform]          │        │
│  │  ████▓▓▓▓▒▒▒▒░░░░   │  │  ██▓▓▓▓▒▒▒▒░░░░░░   │        │
│  │                      │  │                      │        │
│  │  [Spectrogram]       │  │  [Spectrogram]       │        │
│  │  ░░░░░░░░░░░░        │  │  ░░░░░░░░░░░░        │        │
│  │  ████████▒▒░░ 200Hz  │  │  ██████▒▒░░░░ 200Hz  │        │
│  │  ████████████ 60Hz   │  │  ████████████ 60Hz   │        │
│  │                      │  │                      │        │
│  │  [► Play Before]     │  │  [► Play After]      │        │
│  └─────────────────────┘  └─────────────────────┘        │
│                                                          │
│  What changed:                                           │
│  ✓ The kick sounds "tighter" — less mud in the low mids  │
│  ✓ The sub bass (60Hz) is preserved — still hits hard    │
│  ✓ The kick will sit better in a busy mix                 │
│                                                          │
│  Try it yourself: Adjust the slider below                │
│  [200Hz Cut: ████████░░░░░░░░]                           │
│  (Hear the change in real-time as you drag)              │
│                                                          │
│  [Next: What happens when we BOOST the low end? →]       │
└──────────────────────────────────────────────────────────┘
```

### Lesson Format

```python
@dataclass
class SoundDesignLesson:
    id: str
    title: str
    level: int                     # 1-5
    prerequisites: List[str]       # lesson IDs
    concepts_taught: List[str]     # concept IDs

    duration_minutes: int
    interactive: bool              # can user tweak parameters?

    # Content
    explanation: str               # Main lesson text
    audio_example_before: str      # Sound ID for "before"
    audio_example_after: str       # Sound ID for "after"

    # Interactive component
    controls: List[LessonControl]  # Sliders, buttons the user can manipulate

    # Assessment
    quiz_question: str
    quiz_options: List[str]
    correct_answer: int

    # Metadata
    tags: List[str]                # ["eq", "kick", "beginner"]
    related_lessons: List[str]

# Example lessons
LESSONS = [
    SoundDesignLesson(
        id="eq_basics",
        title="What Does EQ Do?",
        level=1,
        prerequisites=[],
        concepts_taught=["frequency_spectrum", "eq"],
        duration_minutes=5,
        interactive=True,
        explanation="EQ lets you shape the frequency content...",
        controls=[
            LessonControl(
                type="slider",
                label="Low Cut Frequency",
                dsp_processor="eq",
                dsp_parameter="low_cut_frequency",
                range=(20, 500),
                default=100,
                unit="Hz",
                realtime=True,
            )
        ],
        quiz_question="What happens when you cut 200Hz on a kick?",
        quiz_options=[
            "It gets louder",
            "It sounds tighter, less muddy",
            "It disappears entirely",
            "It adds more bass",
        ],
        correct_answer=1,
    ),

    SoundDesignLesson(
        id="transient_basics",
        title="Why Do Some Sounds Hit Harder?",
        level=1,
        prerequisites=[],
        concepts_taught=["transient", "attack_time", "transient_shaper"],
        duration_minutes=7,
        interactive=True,
        controls=[
            LessonControl(
                type="slider",
                label="Attack Gain",
                dsp_processor="transient_shaper",
                dsp_parameter="attack_gain",
                range=(-12, 12),
                default=0,
                unit="dB",
                realtime=True,
            ),
        ],
        quiz_question="What does increasing attack gain do?",
        quiz_options=[
            "Makes the sound longer",
            "Makes the initial hit louder and punchier",
            "Removes all low frequencies",
            "Adds reverb to the sound",
        ],
        correct_answer=1,
    ),
]
```

---

## 4. Guided Challenges

### Challenge Format

```
CHALLENGE: "Make This Kick Punchier"

  Goal:       Apply processing to make a dull kick sound punchy
  Level:      Beginner
  Duration:   3-5 minutes
  Concepts:   Transient shaping, EQ, compression

  ┌──────────────────────────────────────────────────┐
  │  YOUR TURN:                                      │
  │                                                  │
  │  Here's a kick that sounds flat and boring.      │
  │  Use the tools below to make it punchier.         │
  │                                                  │
  │  [► Listen to the original]                      │
  │                                                  │
  │  Available tools (drag to use):                  │
  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐   │
  │  │ EQ     │ │Trans.  │ │Comp.   │ │Satur.  │   │
  │  │        │ │Shaper  │ │        │ │        │   │
  │  └────────┘ └────────┘ └────────┘ └────────┘   │
  │                                                  │
  │  Hints (click to reveal):                        │
  │  [Hint 1] Try boosting the attack              │
  │  [Hint 2] A little saturation goes a long way  │
  │  [Hint 3] Cut some low-mid mud (200-400Hz)     │
  │                                                  │
  │  [Check: Does it sound punchy?]                  │
  │                                                  │
  │  Feedback:                                       │
  │  ✓ Attack transient boosted (+3.2dB) ✓           │
  │  ✓ Low-mid mud reduced ✓                         │
  │  ✗ Try adding light saturation for more "crunch" │
  │                                                  │
  │  [Mark Complete] [Try Again] [Show Solution]     │
  └──────────────────────────────────────────────────┘
```

### Challenge Library

```python
CHALLENGES = [
    {
        "id": "make_kick_punchy",
        "title": "Make This Kick Punchy",
        "level": 1,
        "source_sound": "flat_kick_01",
        "target_characteristic": "punchy",
        "allowed_tools": ["eq", "transient_shaper", "saturator", "compressor"],
        "success_criteria": {
            "transient_energy_increase": 0.2,       # +20% transient energy
            "spectral_centroid_in_range": (100, 400), # not too dark, not too bright
            "low_mud_reduction": 0.1,               # -10% energy at 200-400Hz
            "loudness_change_max": 3.0,             # not more than 3dB louder
        },
    },
    {
        "id": "design_trap_hihat",
        "title": "Design a Trap Hi-Hat",
        "level": 2,
        "source_sound": "raw_noise_burst",
        "target_characteristic": "crisp trap hi-hat",
        "allowed_tools": ["eq", "transient_shaper", "saturator", "filter"],
        "success_criteria": {
            "high_freq_energy_min": 0.3,             # enough high end
            "duration_in_range": (0.05, 0.15),       # 50-150ms
            "noise_tonal_ratio_min": 0.7,             # noise-dominant
            "spectral_centroid_min": 8000,           # bright
        },
    },
    {
        "id": "cinematic_impact",
        "title": "Build a Cinematic Impact",
        "level": 3,
        "source_sound": "sub_kick_01",
        "additional_sounds": ["noise_swell", "cymbal_crash"],
        "target_characteristic": "epic cinematic impact",
        "allowed_tools": ["eq", "compressor", "saturator", "reverb", "layerer"],
        "success_criteria": {
            "duration_in_range": (1.0, 3.0),
            "spectral_fullness_min": 0.6,
            "stereo_width_min": 0.5,
            "loudness_range": (-18, -10),
        },
    },
]
```

### Adaptive Difficulty

```python
class ChallengeAdapter:
    """Adapts challenge difficulty based on user skill."""

    def __init__(self):
        self.user_skill = UserSkillModel()

    def get_next_challenge(self, user_id: str) -> Challenge:
        skill_level = self.user_skill.get_level(user_id)
        recent_errors = self.user_skill.get_recent_errors(user_id, n=5)
        mastered_concepts = self.user_skill.get_mastered_concepts(user_id)

        # Filter available challenges
        candidates = [c for c in CHALLENGES
                      if c.level <= skill_level + 1
                      and c.level >= skill_level - 1
                      and all(p in mastered_concepts for p in c.prerequisites)]

        # Prefer challenges that address recent errors
        if recent_errors:
            error_concepts = [e.concept for e in recent_errors]
            candidates.sort(key=lambda c: sum(
                1 for concept in c.concepts_taught if concept in error_concepts
            ), reverse=True)

        return candidates[0] if candidates else None

    def evaluate_attempt(self, user_id: str, challenge_id: str,
                         result: ChallengeResult):
        """Update user skill model based on attempt."""

        # Update concept mastery
        challenge = next(c for c in CHALLENGES if c.id == challenge_id)
        for concept in challenge.concepts_taught:
            if result.passed:
                self.user_skill.record_success(user_id, concept)
            else:
                self.user_skill.record_error(user_id, concept,
                                             result.failure_reason)

        # Check for level-up
        if self.user_skill.should_level_up(user_id):
            self.user_skill.set_level(user_id, self.user_skill.get_level(user_id) + 1)
            return {"event": "level_up", "new_level": self.user_skill.get_level(user_id) + 1}

        return {"event": "progress", "next_challenge": self.get_next_challenge(user_id)}
```

---

## 5. AI Tutor Mode

### Tutor Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    AI TUTOR                               │
│                                                          │
│  Functions:                                              │
│  1. Answer questions about sound design                  │
│  2. Suggest what to try next                             │
│  3. Explain why a sound sounds the way it does           │
│  4. Demo processing chains                              │
│  5. Quiz on concepts                                    │
│                                                          │
│  Triggered by:                                           │
│  • User clicks "?" or "Explain" button                   │
│  • User types "why does this kick sound muddy?"          │
│  • User has been stuck on a challenge for >5 min         │
│  • User exports a sound — "Want to understand this? →"  │
│  • User asks in the prompt bar (prefixed with "?")       │
└─────────────────────────────────────────────────────────┘
```

### Natural Language Q&A

```python
class SoundDesignTutor:
    """
    LLM-powered tutor specifically for sound design.
    Context: current sound, current processing, user skill level.
    """

    def __init__(self, llm_backend: LLMBackend):
        self.llm = llm_backend

    def answer_question(self, question: str, context: TutorContext) -> str:
        # Build prompt with context
        prompt = f"""
You are a sound design tutor for the cShot app.
The user is working on a {context.sound_type} sound.
Current processing: {context.processing_chain}
User skill level: {context.skill_level}/5
User's recent challenges: {context.recent_challenges}

The user asks: "{question}"

Answer briefly (2-3 sentences) and practically.
Include specific parameter suggestions if applicable.
If the user seems confused, suggest a concrete next step.
"""
        return self.llm.generate(prompt)

    def explain_sound(self, audio: AudioBuffer, analysis: AudioAnalysis) -> str:
        """Explain why this sound sounds the way it does."""
        prompt = f"""
Analyze this one-shot audio and explain its character:

Duration: {analysis.duration_ms}ms
Spectral centroid: {analysis.spectral_centroid}Hz
Transient energy: {analysis.transient_energy_ratio:.2f}
Loudness: {analysis.loudness_lufs} LUFS
Peak: {analysis.peak_db} dBFS
Estimated type: {analysis.estimated_type}

What makes this sound {analysis.estimated_type}?
Describe its envelope, frequency content, and character
in terms a music producer would understand (2-3 sentences).
Suggest one thing they could try to change it.
"""
        return self.llm.generate(prompt)

    def suggest_next_step(self, context: TutorContext) -> str:
        """Suggest what the user should try next."""
        prompt = f"""
The user has been working on a {context.sound_type} for {context.session_duration_min} minutes.
They have applied: {context.processing_chain}
But they said: "{context.user_frustration}"

Suggest ONE specific thing they should try next.
Be concrete (include parameter values).
Format: "Try [action] with [value]. This will [effect]."
"""
        return self.llm.generate(prompt)


@dataclass
class TutorContext:
    sound_type: str
    processing_chain: List[str]
    skill_level: int
    recent_challenges: List[str]
    session_duration_min: int
    user_frustration: Optional[str]
```

### Tutor Modalities

```
1. Tooltip Explanations
   Hover over any parameter → tooltip explains what it does
   "Attack: How quickly the compressor responds to loud sounds.
    Fast attack (0.1ms) catches transients. Slow attack (10ms) lets them through."

2. Visual Overlays
   Show spectrogram annotations
   "The red area is the transient — notice how it's concentrated in the first 10ms"
   "The blue area is the sustain — this is where the tone lives"

3. "Explain This Sound" Button
   One-click analysis: "This is a trap kick with a boomy 60Hz sub,
   a papery attack around 3kHz, and a short 200ms decay. It would
   work well in a trap or drill beat."

4. Recipe Suggestions
   "You made a dark 808 kick. Would you like to try:
   • Making it punchier (add transient shaper)
   • Making it subbier (boost 40Hz)
   • Turning it into a 909 kick (change envelope)"

5. Challenge Mode
   "Let's test your knowledge: Make THIS kick sound like a 909.
   Hints available if you get stuck."
```

---

## 6. Interactive Waveform/Spectrogram Education

### Learning Visualization

```
┌─────────────────────────────────────────────────────────┐
│  WAVEFORM VIEW (learn mode)                               │
│                                                          │
│  ████▓▓▓▓░░░░░░░░░░░░░░▒▒▒▒▒▒░░░░░░░░░░░░               │
│  │    │    │    │    │    │    │    │    │               │
│  ATTACK  DECAY          SUSTAIN        RELEASE           │
│  0-10ms  10-100ms       100-800ms      800-1000ms        │
│  ─────── ────────────   ───────────    ───────────       │
│  "hit"   "body"         "tone"         "tail"            │
│                                                          │
│  [Hover to hear just this section]                       │
│  [Click to learn more about envelopes]                   │
│                                                          │
├─────────────────────────────────────────────────────────┤
│  SPECTROGRAM VIEW (learn mode)                            │
│                                                          │
│  16kHz ──▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  Air/Hi-hat range      │
│   8kHz ──▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░  Presence/Attack        │
│   4kHz ──██████████████▓▓▓▓░░░░░░  Harmonic body         │
│   2kHz ──██████████████████▓▓░░░░  Upper mids            │
│   1kHz ──████████████████████▒░░░  Low mids              │
│   500Hz ─█████████████████████▓░░  Low end body         │
│   200Hz ─██████████████████████▒░  Bass                  │
│    60Hz ─███████████████████████░  Sub bass              │
│                                                          │
│          0ms  200ms 400ms 600ms 800ms                    │
│                                                          │
│  Key regions highlighted:                                 │
│  ■ Sub bass (20-100Hz): Kick fundamental                 │
│  ■ Low end (100-300Hz): Body, warmth                     │
|  ■ Mids (300-2000Hz): Tone, character                    │
│  ■ High mids (2k-6kHz): Attack, presence                 │
│  ■ Highs (6kHz+): Air, sizzle                            │
│                                                          │
│  [Hover to solo this frequency range]                    │
│  [Click to learn about frequency bands]                  │
└─────────────────────────────────────────────────────────┘
```

### Interactive Learning Mode

```python
class InteractiveLearnMode:
    """Educational overlay for waveform/spectrogram views."""

    def __init__(self):
        self.active = False
        self.highlighted_region = None
        self.annotations = []

    def enable(self):
        """Turn on learning mode."""
        self.active = True
        # Show region labels
        # Show frequency band labels
        # Show tooltip triggers

    def on_hover(self, position: (float, float), domain: str):
        """User hovers over a region → show explanation."""
        if domain == "waveform":
            time_ms = self.pixel_to_time(position.x)
            region = self.get_waveform_region(time_ms)
            return self.get_region_explanation(region)

        elif domain == "spectrogram":
            freq_hz, time_ms = self.pixel_to_spectral(position)
            band = self.get_frequency_band(freq_hz)
            return self.get_band_explanation(band, time_ms)

    def get_region_explanation(self, region: WaveformRegion) -> str:
        explanations = {
            "attack": "This is the ATTACK — the first 10-50ms of the sound.\n"
                      "This is what gives the sound its 'hit' or 'punch'.\n"
                      "Try adjusting the transient shaper to change this.",
            "decay": "This is the DECAY — how the sound settles after the attack.\n"
                     "Faster decay = tighter sound. Slower decay = more body.\n"
                     "Short decay is typical for kicks. Long decay for cymbals.",
            "sustain": "This is the SUSTAIN — the body of the sound.\n"
                       "Not all one-shots have sustain (kicks don't, pads do).\n"
                       "This is where the tonal character lives.",
            "release": "This is the RELEASE — how the sound fades out.\n"
                       "Long release = ambient. Short release = tight.\n"
                       "Compression can affect how this feels.",
        }
        return explanations.get(region, "")

    def get_band_explanation(self, band: FrequencyBand, time_ms: float) -> str:
        explanations = {
            "sub_bass": "SUB BASS (20-100Hz): The 'feel' frequencies.\n"
                        "Felt more than heard. Kick fundamentals live here.\n"
                        "Too much = boomy. Too little = thin.",
            "bass": "BASS (100-300Hz): The body and warmth.\n"
                    "This is where kick 'body' and bass notes sit.\n"
                    "Too much = muddy. Too little = hollow.",
            "low_mids": "LOW MIDS (300-800Hz): The 'boxy' range.\n"
                        "Can make a kick sound 'honky' if overemphasized.\n"
                        "Often reduced on kicks for a cleaner sound.",
            "mids": "MIDS (800Hz-2kHz): The tonal character.\n"
                    "This is where the sound's 'voice' sits.\n"
                    "Important for snares and melodic elements.",
            "high_mids": "HIGH MIDS (2k-6kHz): Attack and presence.\n"
                         "This is the 'snap' of a snare or the 'click' of a kick.\n"
                         "Boosting here = more articulation.",
            "highs": "HIGHS (6kHz+): Air and sizzle.\n"
                     "Hi-hats and cymbals live here.\n"
                     "Too much = harsh. Too little = dull.",
        }
        return explanations.get(band, "")
```

---

## 7. Complete Learning System

```
┌─────────────────────────────────────────────────────────────────────┐
│                      CSHOT LEARNING SYSTEM                           │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    PASSIVE LEARNING                          │    │
│  │                                                             │    │
│  │  • Tooltips on every parameter (what it does, why it       │    │
│  │    matters, related concepts)                               │    │
│  │  • Visual overlays on waveform/spectrogram (region labels,  │    │
│  │    frequency band labels)                                   │    │
│  │  • "Explain this sound" button (AI analysis of current      │    │
│  │    sound's character)                                       │    │
│  │  • Processing chain visualizer (see the signal flow)        │    │
│  └──────────────────────────┬──────────────────────────────────┘    │
│                             │                                       │
│  ┌──────────────────────────┴──────────────────────────────────┐    │
│  │                    ACTIVE LEARNING                           │    │
│  │                                                             │    │
│  │  • Before/after comparisons (hear the difference)           │    │
│  │  • Interactive lessons (follow-along with real controls)    │    │
│  │  • Guided challenges (apply concepts to real sounds)        │    │
│  │  • AI tutor (ask questions, get explanations)               │    │
│  └──────────────────────────┬──────────────────────────────────┘    │
│                             │                                       │
│  ┌──────────────────────────┴──────────────────────────────────┐    │
│  │                    ASSESSMENT                                │    │
│  │                                                             │    │
│  │  • Knowledge checks (multiple choice after each lesson)     │    │
│  │  • Practical challenges (did you achieve the goal?)         │    │
│  │  • Skill tracking (what concepts have you mastered?)        │    │
│  │  • Adaptive difficulty (challenge matches your level)       │    │
│  └──────────────────────────┬──────────────────────────────────┘    │
│                             │                                       │
│  ┌──────────────────────────┴──────────────────────────────────┐    │
│  │                    KNOWLEDGE GRAPH                           │    │
│  │                                                             │    │
│  │  100+ sound design concepts linked in a DAG                 │    │
│  │  Prerequisites → mastery paths                              │    │
│  │  Each concept has: explanation, visual, audio example       │    │
│  │  User's progress tracked per concept                        │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Summary

1. **Explainable controls**: Every parameter has a one-liner, detailed explanation, visual aid, and computed effect — shown on hover/focus
2. **Before/after lessons**: Interactive side-by-side comparisons with real-time parameter sliders — users hear the effect of each change
3. **Guided challenges**: Goal-oriented tasks with hint system, success criteria, automated feedback, and adaptive difficulty
4. **AI tutor**: LLM-powered Q&A about sound design, context-aware (current sound, processing chain, user skill level)
5. **Interactive waveform/spectrogram education**: Hover-to-hear regions, labeled frequency bands, click-to-learn annotations
6. **Knowledge graph**: 100+ linked sound design concepts with mastery tracking, prerequisites, and personalized learning paths
