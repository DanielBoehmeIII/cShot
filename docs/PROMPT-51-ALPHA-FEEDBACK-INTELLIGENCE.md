# Prompt 51 — Turn Alpha Feedback Into Product Intelligence

Design the feedback system for cShot's first alpha users. Every alpha session should teach cShot what makes a one-shot useful.

---

## 1. What We Need to Capture

| Signal | Why It Matters | Capture Method | Priority |
|--------|---------------|----------------|----------|
| Which sounds users keep | Reveals what's actually useful vs. novel | Favorite event (★ click) | Critical |
| Which sounds users reject | Reveals what's not working | Implicit: no favorite, no export, no replay | Critical |
| Prompt wording | Reveals how users describe sounds | String log on generation | Critical |
| Reference audio type | Reveals what users want to match | File metadata + analysis | High |
| Generation settings | Reveals which params matter | Param snapshot on generate | High |
| Preview behavior | Reveals listening patterns | Play count, play duration, replay timing | High |
| Export behavior | Reveals true intent to use | Export event with target path | Critical |
| User ratings | Reveals perceived quality | 4-point emoji scale | Critical |
| Written feedback | Reveals unexpected insights | Free text at triggers | High |
| Repeat usage | Reveals retention/worth | Session tracking | Critical |

---

## 2. Feedback Database Schema

```sql
-- Core events table: every user action is a row
CREATE TABLE events (
    id            TEXT PRIMARY KEY,           -- uuid
    session_id    TEXT NOT NULL,              -- uuid per app launch
    event_type    TEXT NOT NULL,              -- see event_type enum
    timestamp     TEXT NOT NULL,              -- ISO 8601
    duration_ms   INTEGER,                   -- how long the action took
    success       INTEGER DEFAULT 1,          -- 0/1
    error_message TEXT,                       -- if success=0
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Generation-specific detail
CREATE TABLE generations (
    id            TEXT PRIMARY KEY,
    session_id    TEXT NOT NULL,
    event_id      TEXT NOT NULL UNIQUE,
    prompt        TEXT NOT NULL,
    prompt_length INTEGER,
    prompt_word_count INTEGER,
    reference_id  TEXT,                       -- FK to references, if used
    model_name    TEXT,
    model_version TEXT,
    seed          INTEGER,
    duration_ms   INTEGER,                    -- generation time
    cost_credits  REAL,                       -- API cost tracking
    retry_count   INTEGER DEFAULT 0,
    success       INTEGER DEFAULT 1,
    error_code    TEXT,
    created_at    TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (event_id) REFERENCES events(id)
);

-- Generated sound metadata
CREATE TABLE sounds (
    id              TEXT PRIMARY KEY,
    generation_id   TEXT NOT NULL,
    file_path       TEXT,                     -- local path to cached audio
    duration_seconds REAL,
    sample_rate     INTEGER,
    bit_depth       INTEGER,
    file_size_bytes INTEGER,
    peak_dbfs       REAL,
    rms_dbfs        REAL,
    crest_factor    REAL,
    spectral_centroid REAL,
    sound_type      TEXT,                     -- from auto-tagging: kick, snare, etc.
    tags_json       TEXT,                     -- full tag set from auto-tagger
    embedding_blob  BLOB,                    -- feature embedding for similarity
    created_at      TEXT NOT NULL,
    FOREIGN KEY (generation_id) REFERENCES generations(id)
);

-- Preview/interaction tracking
CREATE TABLE interactions (
    id            TEXT PRIMARY KEY,
    sound_id      TEXT NOT NULL,
    session_id    TEXT NOT NULL,
    interaction_type TEXT NOT NULL,           -- preview, replay, scrub, export, favorite, unfavorite
    timestamp     TEXT NOT NULL,
    preview_duration_ms INTEGER,             -- how long they listened
    preview_complete INTEGER DEFAULT 0,      -- did they listen to the whole sound?
    times_played  INTEGER DEFAULT 0,         -- cumulative count
    context       TEXT,                       -- what screen/state they were in
    FOREIGN KEY (sound_id) REFERENCES sounds(id),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- User ratings (submitted explicitly)
CREATE TABLE ratings (
    id            TEXT PRIMARY KEY,
    sound_id      TEXT NOT NULL,
    session_id    TEXT NOT NULL,
    rating        INTEGER NOT NULL,          -- 1-4 mapped from emoji
    rating_type   TEXT NOT NULL,             -- 'quality', 'usefulness', 'fit'
    context       TEXT,                      -- what triggered this rating prompt
    created_at    TEXT NOT NULL,
    FOREIGN KEY (sound_id) REFERENCES sounds(id),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Written feedback
CREATE TABLE feedback (
    id            TEXT PRIMARY KEY,
    session_id    TEXT NOT NULL,
    feedback_type TEXT NOT NULL,             -- 'free_text', 'bug_report', 'feature_request'
    question      TEXT,                      -- the prompt that was shown
    answer        TEXT NOT NULL,
    context       TEXT,                      -- what they were doing when prompted
    rating        INTEGER,                   -- optional attached rating
    created_at    TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Reference audio uploads
CREATE TABLE references_audio (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    original_filename TEXT,
    file_path       TEXT,
    duration_seconds REAL,
    sample_rate     INTEGER,
    bpm             REAL,                    -- estimated
    key             TEXT,                    -- estimated musical key
    spectral_profile TEXT,                   -- JSON, for comparison with generated
    embedding_blob  BLOB,                   -- for similarity search
    created_at      TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Sessions (one per app launch)
CREATE TABLE sessions (
    id              TEXT PRIMARY KEY,
    started_at      TEXT NOT NULL,
    ended_at        TEXT,
    duration_seconds INTEGER,
    generation_count INTEGER DEFAULT 0,
    export_count    INTEGER DEFAULT 0,
    favorite_count  INTEGER DEFAULT 0,
    preview_count   INTEGER DEFAULT 0,
    upload_count    INTEGER DEFAULT 0,
    feedback_count  INTEGER DEFAULT 0,
    total_play_time_seconds INTEGER DEFAULT 0,
    device_os       TEXT,
    device_arch     TEXT,
    app_version     TEXT,
    api_version     TEXT,
    session_rating  INTEGER                  -- end-of-session overall rating
);

-- Indexes for analytics queries
CREATE INDEX idx_events_session ON events(session_id);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_generations_prompt ON generations(prompt);
CREATE INDEX idx_generations_session ON generations(session_id);
CREATE INDEX idx_interactions_sound ON interactions(sound_id);
CREATE INDEX idx_ratings_sound ON ratings(sound_id);
CREATE INDEX idx_feedback_type ON feedback(feedback_type);
CREATE INDEX idx_sessions_date ON sessions(started_at);
```

### Event Type Enum

```rust
pub enum EventType {
    AppLaunch,
    AppQuit,
    GenerationStart,
    GenerationComplete,
    GenerationFailed,
    PreviewPlay,
    PreviewStop,
    PreviewReplay,
    ExportStart,
    ExportComplete,
    ExportFailed,
    FavoriteAdd,
    FavoriteRemove,
    ReferenceUpload,
    ReferenceAnalyzed,
    RatingSubmitted,
    FeedbackSubmitted,
    SettingsChanged,
    ErrorOccurred,
    SessionEnd,
}
```

---

## 3. Event Tracking Implementation

### Rust Backend (Tauri Commands)

```rust
#[derive(Serialize)]
pub struct TrackEvent {
    pub event_type: EventType,
    pub session_id: String,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: String,
}

#[tauri::command]
async fn track_event(
    state: State<'_, AppState>,
    event: TrackEvent,
) -> Result<(), String> {
    let db = state.db.lock().await;
    
    db.execute(
        "INSERT INTO events (id, session_id, event_type, timestamp, metadata) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            Uuid::new_v4().to_string(),
            event.session_id,
            event.event_type.as_str(),
            event.timestamp,
            event.metadata.map(|m| m.to_string()),
        ],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn get_session_id(state: State<'_, AppState>) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();
    
    state.db.lock().await.execute(
        "INSERT INTO sessions (id, started_at, app_version) VALUES (?1, ?2, ?3)",
        params![session_id, Utc::now().to_rfc3339(), env!("CARGO_PKG_VERSION")],
    ).map_err(|e| e.to_string())?;
    
    Ok(session_id)
}
```

### Frontend (TypeScript)

```typescript
// /src/lib/analytics.ts
import { invoke } from '@tauri-apps/api/core';

interface EventPayload {
  event_type: string;
  session_id: string;
  metadata?: Record<string, unknown>;
}

class Analytics {
  private sessionId: string | null = null;
  private queue: EventPayload[] = [];
  private flushing = false;

  async init() {
    this.sessionId = await invoke<string>('get_session_id');
    // Flush any queued events
    this.flush();
    // Set up periodic flush (every 5 seconds)
    setInterval(() => this.flush(), 5000);
  }

  async track(eventType: string, metadata?: Record<string, unknown>) {
    if (!this.sessionId) {
      this.queue.push({ event_type: eventType, session_id: '', metadata });
      return;
    }
    
    const event: EventPayload = {
      event_type: eventType,
      session_id: this.sessionId,
      metadata,
    };
    
    // Fire and forget — never block the UI
    invoke('track_event', { event }).catch((err) => {
      console.warn('Failed to track event:', err);
    });
  }

  private async flush() {
    if (this.queue.length === 0) return;
    const events = [...this.queue];
    this.queue = [];
    
    for (const event of events) {
      await invoke('track_event', { event }).catch(() => {});
    }
  }

  // Convenience methods
  trackGenerationStart(prompt: string, referenceId?: string) {
    this.track('GenerationStart', { prompt, promptLength: prompt.length, referenceId });
  }

  trackGenerationComplete(durationMs: number, soundId: string, success: boolean) {
    this.track('GenerationComplete', { durationMs, soundId, success });
  }

  trackPreview(soundId: string, durationMs: number) {
    this.track('PreviewPlay', { soundId, durationMs });
  }

  trackExport(soundId: string, format: string) {
    this.track('ExportComplete', { soundId, format });
  }

  trackFavorite(soundId: string, added: boolean) {
    this.track(added ? 'FavoriteAdd' : 'FavoriteRemove', { soundId });
  }

  trackError(context: string, error: string) {
    this.track('ErrorOccurred', { context, error });
  }
}

export const analytics = new Analytics();
```

### Privacy & Consent

```typescript
// First launch dialog
const CONSENT_KEY = 'cshot_telemetry_consent';

export function getTelemetryConsent(): boolean | null {
  return localStorage.getItem(CONSENT_KEY) as boolean | null;
}

export function setTelemetryConsent(consent: boolean) {
  localStorage.setItem(CONSENT_KEY, JSON.stringify(consent));
  if (consent) {
    analytics.init();
  }
}

// What we NEVER track:
// - Audio content (no WAV uploads)
// - Screen recordings
// - Keystroke-level data (just prompts)
// - File system structure
// - Other running applications
```

---

## 4. Rating UI

### Placement Strategy

```
Trigger          | Rating Question               | UI Treatment                  | Frequency
─────────────────|───────────────────────────────|───────────────────────────────|─────────────────
After 3rd gen    | "How's the sound quality?"    | 4 emoji, subtle, bottom-right | Once per session
After 1st export | "Would you use this?"         | 3 option, modal-lite          | Once per session
After 5th gen    | "Better or worse than Splice?"| 5-star comparison             | Every 5 gens
After 10th gen   | "What should we improve?"     | Single-line text input        | Once per session
End of session   | "Rate cShot overall"          | 1-10 slider                   | Once per session
On unfavorite    | "Why didn't this work?"       | Quick select: "Wrong type / Bad quality / Not useful / Other" | On action
```

### Rating Component

```tsx
// /src/components/RatingPrompt.tsx
interface RatingPromptProps {
  question: string;
  type: 'emoji' | 'stars' | 'thumbs' | 'slider' | 'text';
  options?: string[];
  onSubmit: (value: number | string) => void;
  onDismiss: () => void;
  position?: 'bottom-right' | 'modal' | 'inline';
}

export function RatingPrompt({ question, type, options, onSubmit, onDismiss, position = 'bottom-right' }: RatingPromptProps) {
  // Render the appropriate rating UI
  // Always include dismiss (X) button
  // Never block interaction — non-modal by default
  // Animate in/out with fade
  // Store dismissed state in localStorage so it doesn't repeat
}
```

### Emoji-to-Numeric Mapping

```typescript
const EMOJI_MAP = {
  '😕': 1,  // Bad
  '😐': 2,  // Okay
  '😊': 3,  // Good
  '😍': 4,  // Amazing
} as const;
```

### Writing Feedback to DB

```rust
#[tauri::command]
async fn submit_feedback(
    state: State<'_, AppState>,
    session_id: String,
    feedback_type: String,
    question: String,
    answer: String,
    context: Option<String>,
) -> Result<(), String> {
    state.db.lock().await.execute(
        "INSERT INTO feedback (id, session_id, feedback_type, question, answer, context, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            Uuid::new_v4().to_string(),
            session_id,
            feedback_type,
            question,
            answer,
            context,
            Utc::now().to_rfc3339(),
        ],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}
```

---

## 5. Listening-Test Questions

Structured questions for deeper qualitative research. These are NOT shown in-app — they go in the post-session email or scheduled 15-minute call.

### Core Questions (Every Tester)

```
1. Play the 3 sounds you generated. Which one would you actually use?
   → Why that one? What makes it usable?

2. Play the 1 you would NOT use.
   → Why not? What's wrong with it?

3. What were you trying to make when you typed [prompt]?
   → Did the result match what you imagined?

4. Where would this sound sit in your mix?
   → Lead, layer, fill, accent, texture?

5. Compare your favorite generation to a similar sound from your library.
   → Which is better? Why?

6. Would you pay $X/month for this?
   → Why or why not?

7. What's the ONE thing that would make cShot indispensable to you?
```

### Diagnostic Questions (For Bad Generations)

```
1. What's wrong with this sound?
   → Pick: muddy / weak / distorted / boring / too long / too short / wrong genre

2. Does this sound AI-generated to you?
   → What gives it away?

3. What prompt would you have written to get what you actually wanted?
   → Helps us understand prompt engineering gaps
```

### Comparative Questions

```
1. Which sounds better — A or B? (blind A/B of two generation settings)

2. Can you tell which one used a reference audio?

3. Which prompt produced the better result?
   → "punchy kick" vs. "punchy kick drum 140bpm layered"
```

---

## 6. Product Learning Dashboard

### Schema for Aggregated Views

```sql
-- Materialized daily aggregation
CREATE VIEW daily_metrics AS
SELECT
    DATE(s.started_at) AS day,
    COUNT(DISTINCT s.id) AS sessions,
    COUNT(DISTINCT g.id) AS generations,
    COUNT(DISTINCT i.id) FILTER (WHERE i.interaction_type = 'export') AS exports,
    COUNT(DISTINCT i.id) FILTER (WHERE i.interaction_type = 'favorite') AS favorites,
    AVG(g.duration_ms) AS avg_gen_time_ms,
    SUM(CASE WHEN g.success = 0 THEN 1 ELSE 0 END) * 1.0 / COUNT(g.id) AS failure_rate,
    AVG(r.rating) AS avg_rating,
    COUNT(DISTINCT f.id) AS feedback_count,
    AVG(s.duration_seconds) AS avg_session_duration
FROM sessions s
LEFT JOIN generations g ON g.session_id = s.id
LEFT JOIN interactions i ON i.session_id = s.id
LEFT JOIN ratings r ON r.session_id = s.id
LEFT JOIN feedback f ON f.session_id = s.id
GROUP BY day;

-- Top-performing prompts
CREATE VIEW top_prompts AS
SELECT
    g.prompt,
    COUNT(*) AS generation_count,
    SUM(CASE WHEN i.interaction_type = 'export' THEN 1 ELSE 0 END) AS export_count,
    SUM(CASE WHEN i.interaction_type = 'favorite' THEN 1 ELSE 0 END) AS favorite_count,
    AVG(r.rating) AS avg_rating,
    AVG(g.duration_ms) AS avg_gen_time
FROM generations g
LEFT JOIN interactions i ON i.sound_id IN (
    SELECT id FROM sounds WHERE generation_id = g.id
)
LEFT JOIN ratings r ON r.sound_id IN (
    SELECT id FROM sounds WHERE generation_id = g.id
)
GROUP BY g.prompt
HAVING generation_count >= 3
ORDER BY export_count DESC;

-- Sound type performance
CREATE VIEW sound_type_metrics AS
SELECT
    snd.sound_type,
    COUNT(*) AS generation_count,
    SUM(CASE WHEN i.interaction_type = 'export' THEN 1 ELSE 0 END) AS export_count,
    SUM(CASE WHEN i.interaction_type = 'favorite' THEN 1 ELSE 0 END) AS favorite_count,
    AVG(r.rating) AS avg_rating,
    AVG(snd.duration_seconds) AS avg_duration,
    AVG(snd.rms_dbfs) AS avg_loudness,
    AVG(snd.crest_factor) AS avg_punch
FROM sounds snd
LEFT JOIN interactions i ON i.sound_id = snd.id
LEFT JOIN ratings r ON r.sound_id = snd.id
GROUP BY snd.sound_type
ORDER BY export_count DESC;

-- Failure analysis
CREATE VIEW failure_analysis AS
SELECT
    g.error_code,
    COUNT(*) AS count,
    AVG(g.duration_ms) AS avg_time_before_failure,
    g.prompt
FROM generations g
WHERE g.success = 0
GROUP BY g.error_code, g.prompt
ORDER BY count DESC;

-- Prompt effectiveness (export rate by prompt characteristics)
CREATE VIEW prompt_effectiveness AS
SELECT
    CASE
        WHEN g.prompt LIKE '%bpm%' THEN 'has_bpm'
        WHEN g.prompt LIKE '%kick%' OR g.prompt LIKE '%snare%' OR g.prompt LIKE '%hat%' THEN 'drum'
        WHEN g.prompt LIKE '%bass%' OR g.prompt LIKE '%sub%' THEN 'bass'
        WHEN g.prompt LIKE '%ambient%' OR g.prompt LIKE '%pad%' OR g.prompt LIKE '%texture%' THEN 'texture'
        WHEN g.prompt LIKE '%fx%' OR g.prompt LIKE '%riser%' OR g.prompt LIKE '%impact%' THEN 'fx'
        ELSE 'other'
    END AS prompt_category,
    COUNT(*) AS count,
    SUM(CASE WHEN i.interaction_type = 'export' THEN 1 ELSE 0 END) AS exports,
    AVG(r.rating) AS avg_rating
FROM generations g
LEFT JOIN interactions i ON i.sound_id IN (SELECT id FROM sounds WHERE generation_id = g.id)
LEFT JOIN ratings r ON r.sound_id IN (SELECT id FROM sounds WHERE generation_id = g.id)
GROUP BY prompt_category
ORDER BY exports DESC;
```

### Dashboard Views

```
WEEKLY REVIEW DASHBOARD
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

OVERVIEW (last 7 days)
  Sessions:          47
  Generations:       312
  Exports:           89 (28.5% export rate)
  Favorites:         134 (42.9% favorite rate)
  Avg rating:        2.8/4.0
  Failure rate:      8.3%
  Avg gen time:      4.2s

TOP PROMPTS (by export count)
  1. "punchy kick 140bpm"        — 12 exports, 3.6 ★
  2. "tight trap snare"          — 8 exports, 3.2 ★
  3. "deep 808 sub"              — 7 exports, 3.8 ★
  4. "bright open hat"           — 5 exports, 3.0 ★
  5. "dark ambient pad"          — 4 exports, 2.5 ★

SOUND TYPE PERFORMANCE
  Kick     — 128 gens, 42 exports (32.8%), 3.4 ★
  Snare    — 74 gens,  18 exports (24.3%), 2.8 ★
  Hat      — 52 gens,  16 exports (30.8%), 3.1 ★
  Bass     — 31 gens,   9 exports (29.0%), 3.5 ★
  FX       — 18 gens,   3 exports (16.7%), 2.2 ★
  Perc     — 9 gens,    1 exports (11.1%), 2.0 ★

FAILURE ANALYSIS
  timeout         — 15 (48.4%) — API timeout
  rate_limit      — 8  (25.8%) — hit API rate limit
  empty_response  — 4  (12.9%) — model returned silence
  bad_reference   — 3  (9.7%)  — reference format not supported
  auth_error      — 1  (3.2%)  — API key expired

PROMPT CATEGORY EFFECTIVENESS
  has_bpm    — 2.8★, 34.2% export rate  ← BPM in prompt = higher export
  drum       — 3.2★, 31.5% export rate
  bass       — 3.5★, 29.0% export rate
  texture    — 2.1★, 12.5% export rate
  fx         — 2.2★, 16.7% export rate

USER SEGMENTS (by generation count)
  Power users (20+ gens/session)   — 8 users, 4.1 avg rating
  Regular (5-19 gens/session)      — 22 users, 2.9 avg rating
  Casual (<5 gens/session)         — 17 users, 1.8 avg rating

  → Power users rate higher → initial friction may be hiding value
  → Casual users need better first-generation quality
```

### Dashboard Implementation (Tauri + React)

```typescript
// /src/lib/dashboard.ts
interface DashboardData {
  overview: {
    sessions: number;
    generations: number;
    exports: number;
    exportRate: number;
    favorites: number;
    favoriteRate: number;
    avgRating: number;
    failureRate: number;
    avgGenTimeMs: number;
  };
  topPrompts: Array<{
    prompt: string;
    count: number;
    exports: number;
    avgRating: number;
  }>;
  soundTypePerformance: Array<{
    type: string;
    generations: number;
    exports: number;
    exportRate: number;
    avgRating: number;
  }>;
  failureAnalysis: Array<{
    errorCode: string;
    count: number;
    percentage: number;
  }>;
}

// Rust command to query the dashboard
#[tauri::command]
async fn get_dashboard(
    state: State<'_, AppState>,
    days: i32,
) -> Result<DashboardData, String> {
    // Run aggregate queries against SQLite
    // Return pre-computed dashboard data
}
```

---

## 7. Weekly Iteration Process

### Monday — Review & Prioritize

```
1. Review dashboard (5 min)
   - Export rate up or down?
   - Any new top prompts?
   - Failure rate trending?

2. Read raw feedback (15 min)
   - All free-text from the week
   - Group by theme
   - Flag urgent complaints

3. Pick 3 things to improve
   - One quality issue (e.g., "snares are weak")
   - One UX issue (e.g., "generation is too slow")
   - One growth issue (e.g., "users don't know what to prompt")

4. Write this week's experiment plan
   - "This week we test: longer prompts produce better results?"
   - "This week we fix: snare transient sharpness"
   - "This week we ship: prompt suggestion chips"
```

### Tuesday-Thursday — Build & Ship

```
Each improvement should:
1. Be shippable in 1-2 days (alpha velocity > quality)
2. Have a measurable success metric
3. Be communicated to testers

Example sprint:
  Tue AM: Fix snare transient — adjust DSP post-processing
  Tue PM: Ship alpha v0.1.1 with fix
  Wed AM: Add prompt suggestion chips below input
  Wed PM: Ship alpha v0.1.2
  Thu AM: Check dashboard — are snare exports up?
  Thu PM: If yes, keep. If no, try something else.
```

### Friday — Analyze & Share

```
1. Run this week's experiment analysis
   - Was our hypothesis correct?
   - What did we learn?

2. Write alpha update for Discord/testers
   - "This week we shipped: X, Y, Z"
   - "Based on your feedback, we fixed: X"
   - "Next week we're exploring: X"

3. Update the learning log
   - One file: ALPHA_LEARNINGS.md
   - One entry per week
   - Capture: what we tried, what happened, what we learned
```

### Learning Log Template

```markdown
# Alpha Learning Log

## Week 1 (2025-01-20)
**Experiment:** Does adding BPM to prompts improve quality?
**Result:** Yes — prompts with BPM have 34% export rate vs 22% without.
**Action:** Auto-suggest BPM in prompt field. Show "140bpm" chip.

**Experiment:** Are users finding the generation too slow?
**Result:** Average rating for speed: 2.1/4. No — they want better, not faster.
**Action:** Keep generation time target at <5s. Focus on quality.

**Surprise:** Users keep generating hats even though they rate them lower.
**Hypothesis:** Hats are harder to get right, so users iterate more.
**Action:** Profile hat generations to find failure patterns.

## Week 2 (2025-01-27)
...
```

---

## 8. Feedback Pipeline Architecture

```
User Action
    │
    ▼
┌─────────────────┐
│ Frontend Event   │  TypeScript, non-blocking
│ analytics.track()│  Queued, batched every 5s
└────────┬────────┘
         │ invoke('track_event')
         ▼
┌─────────────────┐
│ Tauri IPC        │  Rust command
│ track_event()    │  Inserts into SQLite
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ SQLite Database   │  ~/cShot/data/feedback.db
│ events            │
│ generations       │  All local, user-owned
│ interactions      │
│ ratings           │
│ feedback          │
│ sessions          │
└────────┬────────┘
         │ (opt-in sync)
         ▼
┌─────────────────┐
│ Sync Service     │  Optional, user-consented
│ Uploads to S3    │  Only aggregate + metadata
│ No audio files   │  No prompt→sound pairs
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Dashboard        │  Local or web-based
│ Weekly review    │  Queries + visualizations
│ Insights         │
└─────────────────┘
```

---

## 9. Summary

The feedback system turns every user action into a data point. The schema captures generations, interactions, ratings, and written feedback. The rating UI is fast, non-blocking, and strategically timed. The dashboard surfaces what matters: what people actually keep. The weekly process ensures every alpha session teaches cShot what makes a one-shot useful.
