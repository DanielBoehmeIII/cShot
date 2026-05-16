# Prompt 74 — Multi-Agent Audio Pipeline

Design a multi-agent architecture for cShot. Agents for transient design, timbre shaping, genre matching, sound repair, metadata labeling, quality evaluation, pack organization, personalization, and search. Research orchestration systems, blackboard architectures, distributed cognition, model routing, and ensemble systems.

---

## 1. Why Multi-Agent?

### The Monolith Problem

```
Current cShot pipeline (single model → single output):

  Prompt → [Generation Model] → Sound → [Repair Chain] → Output

  Problems:
    1. One model does everything → jack of all trades, master of none
    2. No specialization → kicks and snares use the same pipeline
    3. No feedback loops → generation doesn't learn from repair
    4. No orchestration → can't route to different models per task
    5. No memory → each generation is isolated from the past
    6. No collaboration → agents can't share information

  The monolith hits a quality ceiling.
  Specialized agents can surpass it by dominating their domain.
```

### The Multi-Agent Vision

```
Instead of one model doing everything:

  ┌─────────────────────────────────────────────────────────────────────┐
  │                                                                     │
  │  Prompt → [Orchestrator] → Specialized Agents → Ensemble → Output  │
  │                              │                                      │
  │                              ▼                                      │
  │                    Each agent has:                                  │
  │                      - A single domain (transient, timbre, etc.)    │
  │                      - A specialized model/algorithm                │
  │                      - A quality metric for its domain              │
  │                      - Memory of past generations                   │
  │                      - Communication channel to other agents        │
  │                                                                     │
  │  The orchestrator:                                                   │
  │    - Routes requests to the right agents                            │
  │    - Manages agent communication                                    │
  │    - Evaluates ensemble quality                                     │
  │    - Iterates until quality threshold met                           │
  │    - Learns routing patterns over time                              │
  │                                                                     │
└─────────────────────────────────────────────────────────────────────┘

Benefits:
  ✓ Specialization: each agent masters ONE thing
  ✓ Composability: agents can be added, removed, upgraded independently
  ✓ Parallelism: independent agents run concurrently
  ✓ Explainability: each agent's contribution is visible
  ✓ Evolution: improve one agent without touching others
  ✓ Distribution: agents can run on different hardware
```

---

## 2. Agent Architecture

### Agent Types

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  CORE GENERATION AGENTS:                                           │
│                                                                     │
│  1. TRANSIENT AGENT                                                │
│     Domain: Attack shape, decay profile, transient character       │
│     Model: Fine-tuned diffusion model (specialized for transients) │
│     Input: "punchy attack, 8ms rise time, sharp peak"             │
│     Output: Transient envelope + timing parameters                 │
│     Quality metric: Transient-to-noise ratio, attack accuracy      │
│                                                                     │
│  2. TIMBRE AGENT                                                   │
│     Domain: Harmonic content, spectral shape, tonal character      │
│     Model: Neural synthesizer (differentiable harmonic model)      │
│     Input: "warm, analog, slight saturation, mid-forward"          │
│     Output: Harmonic profile + spectral envelope                    │
│     Quality metric: Spectral centroid accuracy, harmonic matching  │
│                                                                     │
│  3. GENRE AGENT                                                    │
│     Domain: Genre-appropriate character, style conventions         │
│     Model: Genre classifier → conditioned latent sampler           │
│     Input: "trap, 140bpm, aggressive"                              │
│     Output: Genre embedding + style parameters                     │
│     Quality metric: Genre classifier confidence                    │
│                                                                     │
│  4. BODY/TAIL AGENT                                                │
│     Domain: Sustain, decay tail, room character, body feel         │
│     Model: Neural reverb + body synthesizer                        │
│     Input: "short decay, tight body, no room"                     │
│     Output: Decay envelope + body spectrum                         │
│     Quality metric: Decay match, body frequency accuracy           │
│                                                                     │
│  POST-PROCESSING AGENTS:                                           │
│                                                                     │
│  5. REPAIR AGENT                                                   │
│     Domain: Fix generation artifacts, normalize, clean up          │
│     Model: Audio restoration network + DSP chain                   │
│     Input: Raw generation output                                    │
│     Output: Cleaned, normalized sound                               │
│     Quality metric: Artifact reduction, SNR improvement             │
│                                                                     │
│  6. MIX PLACEMENT AGENT                                            │
│     Domain: EQ carve, dynamic range, spectral positioning          │
│     Model: Differentiable EQ network + loudness optimizer          │
│     Input: "forward in mix, -12 LUFS, slight high shelf"          │
│     Output: Processed sound with mix-ready characteristics        │
│     Quality metric: LUFS target, spectral balance score            │
│                                                                     │
│  KNOWLEDGE AGENTS:                                                 │
│                                                                     │
│  7. METADATA AGENT                                                 │
│     Domain: Labeling, tagging, description generation              │
│     Model: CLAP classifier + LLM for description                   │
│     Input: Generated sound audio                                    │
│     Output: Type, genre, mood, character tags, description         │
│     Quality metric: Tag accuracy, description quality              │
│                                                                     │
│  8. QUALITY EVALUATOR                                              │
│     Domain: Overall sound quality assessment                       │
│     Model: SoundScore model (regression on perceptual metrics)     │
│     Input: Generated sound audio + all agent outputs               │
│     Output: Quality score (0-100) + improvement suggestions        │
│     Quality metric: Correlation with human ratings                 │
│                                                                     │
│  ORCHESTRATION AGENTS:                                             │
│                                                                     │
│  9. PACK ORGANIZER                                                 │
│     Domain: Pack structure, cohesion, naming, folder hierarchy     │
│     Model: Embedding-based cohesion engine                         │
│     Input: Set of generated sounds + theme                         │
│     Output: Organized pack structure + naming                      │
│     Quality metric: Pack cohesion score, naming consistency        │
│                                                                     │
│  10. PERSONALIZATION AGENT                                         │
│      Domain: User taste modeling, preference prediction            │
│      Model: User embedding + collaborative filtering               │
│      Input: User history, current prompt, generated sound          │
│      Output: Personal preference score (0-1) + taste-adjusted      │
│      Quality metric: Prediction accuracy (export vs skip)          │
│                                                                     │
│  11. SEARCH AGENT                                                  │
│      Domain: Semantic search, similarity retrieval                 │
│      Model: FAISS index + embedding router                         │
│      Input: Query (text/sound/hybrid), filters                     │
│      Output: Top-K relevant sounds + relevance scores              │
│      Quality metric: NDCG@10, Recall@100                           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Agent Communication Protocol

```
Each agent communicates via a shared blackboard.

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  BLACKBOARD — Central shared state                                  │
│                                                                     │
│  {
│    "request_id": "gen_abc123",
│    "prompt": "punchy trap kick 140bpm aggressive",
│    "status": "in_progress",
│    "agents_used": [],
│    "intermediate_results": {},
│    "final_output": null,
│    "quality_scores": {},
│    "agent_comments": {}
│  }
│                                                                     │
│  Communication flow:                                                │
│    1. Orchestrator writes request to blackboard                    │
│    2. Agents read relevant parts, write their outputs             │
│    3. Agents can read OTHER agents' outputs (collaboration)       │
│    4. Orchestrator monitors blackboard, manages flow              │
│    5. When all agents complete, orchestrator assembles output     │
│                                                                     │
│  Agent contract:                                                    │
│    Each agent must:                                                 │
│      - Declare its inputs (what it reads from blackboard)          │
│      - Declare its outputs (what it writes to blackboard)          │
│      - Provide a confidence score for its output                   │
│      - Handle failures gracefully (write error, don't crash)       │
│      - Complete within a time budget (configurable per agent)      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Agent Pipeline Example

```
Generation of "punchy trap kick 140bpm":

  1. Orchestrator receives prompt
     └── Parses: type=kick, genre=trap, bpm=140, style=aggressive

  2. Orchestrator spawns parallel agents:
     ├── Genre Agent → genre_embedding = [0.8 trap, 0.1 drill, 0.1 house]
     ├── Transient Agent → transient_params = {attack:8ms, peak:-2dB, decay:120ms}
     └── Timbre Agent → timbre_params = {warmth:0.7, brightness:0.5, body:0.8}

  3. Orchestrator spawns dependent agents:
     └── Body/Tail Agent → tail_params = {decay:300ms, body_freq:120Hz}
         (depends on transient + timbre agents output)

  4. Orchestrator synthesizes from agent outputs:
     ├── Transient + Timbre + Body → raw audio buffer
     └── Passes raw audio to Repair Agent

  5. Repair Agent → cleaned audio buffer

  6. Orchestrator spawns quality agents:
     ├── Quality Evaluator → SoundScore: 82/100
     ├── Mix Placement Agent → adjusted EQ, target -12 LUFS
     └── Metadata Agent → tags: ["punchy", "trap", "aggressive", "kick"]

  7. Orchestrator checks quality threshold (target > 75):
     ├── Score 82 > 75 → proceed
     └── If failed: iterate with adjusted parameters

  8. Personalization Agent → taste_score = 0.78 (user likes this type)

  9. Orchestrator assembles final output:
     ├── audio: WAV buffer
     ├── metadata: {...}
     ├── quality: {soundscore: 82, taste_match: 0.78}
     └── provenance: {agents: [...], model_ids: [...], timing: {...}}

  Total time: ~3 seconds (parallel agents = 1.5s, serial = 1.5s)
```

---

## 3. Orchestration Systems

### Orchestrator Design

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Orchestrator — The brain of the multi-agent system                 │
│                                                                     │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Request Router                                              │    │
│  │  • Parse incoming request (type, genre, style, constraints)  │    │
│  │  • Determine which agents to invoke                         │    │
│  │  • Determine agent execution order (parallel vs serial)     │    │
│  │  • Assign time budgets per agent                            │    │
│  │                                                             │    │
│  │  Routing rules:                                             │    │
│  │    "generate kick"       → transient + timbre + body        │    │
│  │    "repair sound"        → repair + mix + quality           │    │
│  │    "find similar"        → search + personalization         │    │
│  │    "create pack"         → genre + transient + timbre +     │    │
│  │                             body × N + pack_organizer       │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                     │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Execution Manager                                           │    │
│  │  • Maintain agent execution graph (DAG)                     │    │
│  │  • Execute independent agents in parallel                   │    │
│  │  • Execute dependent agents sequentially                    │    │
│  │  • Handle timeouts (kill slow agents, fall back)            │    │
│  │  • Handle failures (retry, substitute, degrade)             │    │
│  │  • Track execution timing for optimization                  │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                     │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Quality Controller                                          │    │
│  │  • Define quality threshold per request type                │    │
│  │  • Evaluate ensemble output against threshold               │    │
│  │  • If below threshold: identify weak agents, iterate        │    │
│  │  • Iteration: adjust parameters, re-run weak agents         │    │
│  │  • Max iterations: 3 (prevent infinite loops)               │    │
│  │  • Escalation: if 3 iterations fail, route to fallback      │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                     │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Memory Manager                                              │    │
│  │  • Store generation history (request → agent outputs)       │    │
│  │  • Retrieve similar past generations for warm-start         │    │
│  │  • Learn routing patterns: "this prompt → these agents"    │    │
│  │  • Learn parameter patterns: "this prompt → these params"  │    │
│  │  • Short-term memory: current session (for iteration)       │    │
│  │  • Long-term memory: all generations (for learning)         │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Execution Strategies

```
Strategy 1 — Sequential Pipeline:
  Agent A → Agent B → Agent C → ...
  
  Use when: strict dependencies, no parallel gain
  Example: raw_audio → repair → mix → quality
  Latency: sum of all agent latencies
  Best for: post-processing chains

Strategy 2 — Parallel Fan-Out:
  Agent A ─→ Agent B1 ──┐
           ─→ Agent B2 ──┤──→ Agent C
           ─→ Agent B3 ──┘
  
  Use when: independent sub-tasks
  Example: genre + transient + timbre run simultaneously
  Latency: max(B1, B2, B3) + A + C
  Best for: initial sound generation

Strategy 3 — Ensemble Voting:
  Agent 1 → result1 ──┐
  Agent 2 → result2 ──┤──→ Vote → final_result
  Agent 3 → result3 ──┘
  
  Use when: multiple approaches to same task
  Example: 3 different transient models, vote on best
  Latency: max(Agent1, Agent2, Agent3)
  Best for: critical quality decisions

Strategy 4 — Blackboard Collaboration:
  Multiple agents read/write shared state iteratively.
  
  Agent A writes → B reads A, writes → C reads A+B, writes → 
  A reads B+C, refines... → converge on solution
  
  Use when: complex problems needing multiple perspectives
  Example: "make this snare fit an orchestral mix"
  Latency: N iterations × max(agent latencies)
  Best for: complex creative problems

Strategy 5 — Mixture of Experts Routing:
  Router → selects top-K agents → combines outputs
  
  Router trained to predict which agents perform best per input.
  "For this prompt, transient_agent_v2 and timbre_agent_v3 work best"
  
  Use when: many specialized agent variants
  Example: different transient models for different genres
  Latency: router + selected agents
  Best for: quality-maximizing scenarios
```

### Dynamic Orchestration

```
Not all generations need all agents.

Orchestrator dynamically selects agents based on:

  ┌──────────────────┬────────────────────────────────────────────┐
  │ Request Type     │ Active Agents                               │
  ├──────────────────┼────────────────────────────────────────────┤
  │ Quick generation │ Transient + Timbre + Repair + Quality       │
  │ (default)        │ (minimal — fast)                            │
  ├──────────────────┼────────────────────────────────────────────┤
  │ High quality     │ All generation agents + Repair + Mix +     │
  │ (HQ mode)        │ Quality × 2 + Metadata                     │
  │                  │ (max quality — slower)                      │
  ├──────────────────┼────────────────────────────────────────────┤
  │ Experimental     │ Transient(random)+ Timbre(random) + Genre  │
  │ (discovery)      │ + Quality (permissive)                     │
  │                  │ (creativity — less quality filtering)       │
  ├──────────────────┼────────────────────────────────────────────┤
  │ Repair only      │ Repair + Mix + Quality                     │
  │ (uploaded audio) │ (no generation — just polish)              │
  ├──────────────────┼────────────────────────────────────────────┤
  │ Batch generation │ Genre + Transient + Timbre × N +           │
  │ (pack mode)      │ Pack Organizer + Metadata                  │
  │                  │ (max throughput — batched)                 │
  ├──────────────────┼────────────────────────────────────────────┤
  │ Search           │ Search + Personalization                   │
  │ (query)          │ (no generation — retrieval only)           │
  └──────────────────┴────────────────────────────────────────────┘

  Orchestrator also learns user preferences:
    - User frequently regenerates → add quality agent with stricter threshold
    - User exports most sounds → reduce agent count (they like raw output)
    - User favors experimental → add random variance to generation agents
```

---

## 4. Memory Systems

### Agent Memory Types

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  MEMORY TYPE 1 — Working Memory (per generation)                   │
│                                                                     │
│  Scope: Single generation request.                                 │
│  Contents: Prompt, intermediate results, agent outputs, quality    │
│  Lifetime: Until generation completes (+ 5min for debug)           │
│  Storage: In-memory (Redis, TTL=300s)                              │
│                                                                     │
│  Used for:                                                         │
│    - Agent communication via blackboard                            │
│    - Iteration (try → evaluate → adjust → try again)              │
│    - Logging and observability                                    │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  MEMORY TYPE 2 — Session Memory (per user session)                │
│                                                                     │
│  Scope: Single user session (typically 1 hour).                   │
│  Contents: Last 20 generations, user interactions per gen          │
│  Lifetime: Session duration (+ 24h for session resume)             │
│  Storage: Redis, TTL=86400s                                        │
│                                                                     │
│  Used for:                                                         │
│    - "Undo" and re-generation                                      │
│    - Session-aware personalization ("you've been making dark kicks")│
│    - Consistent generation character within session                │
│    - "Generate 5 more like this" — uses same params                │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  MEMORY TYPE 3 — User Memory (per user)                           │
│                                                                     │
│  Scope: Entire user history.                                       │
│  Contents: Taste embedding, preference vectors, export patterns    │
│  Lifetime: Permanent (until user deletes account)                  │
│  Storage: PostgreSQL (structured) + Vector DB (embeddings)         │
│                                                                     │
│  Used for:                                                         │
│    - Personalization: "this user prefers punchy kicks"             │
│    - Routing optimization: "this user's requests → these agents"   │
│    - Quality calibration: "this user's quality threshold = 75"    │
│    - Cold-start mitigation: similar users → proxy preferences     │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  MEMORY TYPE 4 — Agent Memory (per agent, cross-user)             │
│                                                                     │
│  Scope: All generations across all users.                          │
│  Contents: Prompt→param mappings, success rates, failure modes    │
│  Lifetime: Permanent                                               │
│  Storage: PostgreSQL + ML model weights                            │
│                                                                     │
│  Used for:                                                         │
│    - Learning: "prompt pattern X → agent params Y"                │
│    - Quality improvement: "these params produce better results"    │
│    - Failure prediction: "this prompt type → high failure rate"   │
│    - Agent improvement: fine-tune agent models on successful gens  │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  MEMORY TYPE 5 — Global Knowledge Base (cross-user, curated)      │
│                                                                     │
│  Scope: All generations + external knowledge.                      │
│  Contents: Genre profiles, trend data, production techniques       │
│  Lifetime: Permanent (updated periodically)                        │
│  Storage: Vector DB + Document store                               │
│                                                                     │
│  Used for:                                                         │
│    - Genre agent: "trap production characteristics"                │
│    - Trend awareness: "2025 trap → more distorted 808s"           │
│    - Cultural reference: "Lex Luger drums → 2010 trap"            │
│    - Quality baseline: "this genre's quality distribution"        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Memory Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Memory Service                                                │  │
│  │                                                               │  │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐  │  │
│  │  │ Working Memory │  │ Session Memory │  │  User Memory   │  │  │
│  │  │ (Redis)        │  │ (Redis)        │  │ (PostgreSQL)   │  │  │
│  │  │ TTL: 300s      │  │ TTL: 86400s    │  │ Permanent      │  │  │
│  │  │ 50MB           │  │ 500MB          │  │ 10GB           │  │  │
│  │  └────────────────┘  └────────────────┘  └────────────────┘  │  │
│  │                                                               │  │
│  │  ┌────────────────┐  ┌────────────────┐                       │  │
│  │  │ Agent Memory   │  │ Knowledge Base │                       │  │
│  │  │ (PostgreSQL)   │  │ (Vector DB)    │                       │  │
│  │  │ Permanent      │  │ Permanent      │                       │  │
│  │  │ 100GB          │  │ 50GB           │                       │  │
│  │  └────────────────┘  └────────────────┘                       │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  Agent → Memory API:                                                │
│    read_working(req_id, key)              → value                  │
│    write_working(req_id, key, value)       → OK                    │
│    read_session(user_id, key)             → value                  │
│    write_session(user_id, key, value)      → OK                    │
│    read_user(user_id, key)                → value                  │
│    write_user(user_id, key, value)         → OK                    │
│    query_agent_memory(agent_id, prompt)    → similar params        │
│    query_knowledge_base(domain, query)     → relevant info         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 5. Evaluation Loops

### Per-Generation Evaluation

```
Every generation is evaluated at multiple levels:

Level 1 — Agent-Level Evaluation:
  Each agent self-reports: confidence score (0.0-1.0)
  "I'm 87% confident this transient is correct"

Level 2 — Quality Evaluator:
  Independent evaluation agent scores the output.
  SoundScore: 0-100
  Sub-scores: transient_quality, timbre_quality, mix_readiness, etc.

Level 3 — User Feedback (implicit):
  User action = implicit evaluation signal:
    Export:     +1.0 (strong positive)
    Favorite:   +0.8 (positive)
    Regenerate: -0.3 (weak negative)
    Skip:       -0.1 (neutral/weak negative)
    Delete:     -0.8 (strong negative)
    Report:     -1.0 (extreme negative — system failure)

Level 4 — User Feedback (explicit):
  Thumbs up/down: binary signal
  Rating: 1-5 stars
  Comment: qualitative signal (→ NL analysis)

Level 5 — Downstream Success:
  Was the sound used in a finished track?
  Was the pack purchased?
  Did the user return to generate more?
  These are delayed signals but the most valuable.
```

### Learning Loops

```
Loop 1 — Real-Time Agent Adjustment:
  User exports a sound → agent params that produced it get +weight
  User skips a sound → agent params get -weight
  Effect: agents gradually align with user preferences within session
  
  Timescale: seconds (within session)

Loop 2 — Session-Level Learning:
  After session ends, analyze: which agent configurations 
  produced the highest export rates?
  Update session patterns for next session.
  
  Timescale: hours (session to session)

Loop 3 — User-Level Learning:
  After N generations, compute user's taste embedding.
  Update personalization agent.
  Adjust quality thresholds per user.
  
  Timescale: days (compounds with usage)

Loop 4 — Agent-Level Learning:
  Aggregate all user feedback across all generations.
  Fine-tune agent models on high-rated outputs.
  (Transient agent trained on best transients)
  
  Timescale: weeks (requires thousands of ratings)

Loop 5 — System-Level Learning:
  Analyze which agent configurations produce best outcomes.
  Adjust orchestrator routing rules.
  Add/remove agents based on utility.
  
  Timescale: months (orchestrator evolution)
```

### Quality Gates

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Quality gate hierarchy per generation:                            │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  GATE 0: Prompt Parsing                                      │  │
│  │  Can the orchestrator parse the user's intent?               │  │
│  │  Fail: "I don't understand that prompt" → suggest alternatives│  │
│  └──────────────────────────────────────────────────────────────┘  │
│                    │                                                │
│  ┌─────────────────▼──────────────────────────────────────────┐  │  │
│  │  GATE 1: Generation Success                                │  │  │
│  │  Did all agents complete without error?                    │  │  │
│  │  Fail: partial output with degraded experience             │  │  │
│  └────────────────────────────────────────────────────────────┘  │  │
│                    │                                                │
│  ┌─────────────────▼──────────────────────────────────────────┐  │  │
│  │  GATE 2: Technical Quality                                 │  │  │
│  │  No clipping, no DC offset, valid sample rate              │  │  │
│  │  Fail: reprocess through repair agent                     │  │  │
│  └────────────────────────────────────────────────────────────┘  │  │
│                    │                                                │
│  ┌─────────────────▼──────────────────────────────────────────┐  │  │
│  │  GATE 3: Perceptual Quality (SoundScore)                   │  │  │
│  │  Score > threshold (default: 70, user-adjusted)            │  │  │
│  │  Fail: iterate with adjusted params (max 3 iterations)     │  │  │
│  └────────────────────────────────────────────────────────────┘  │  │
│                    │                                                │
│  ┌─────────────────▼──────────────────────────────────────────┐  │  │
│  │  GATE 4: Prompt Alignment                                  │  │  │
│  │  Does the output match the prompt intent?                  │  │  │
│  │  Measured by: CLAP score between prompt and sound          │  │  │
│  │  Fail: regenerate with stronger conditioning               │  │  │
│  └────────────────────────────────────────────────────────────┘  │  │
│                    │                                                │
│  ┌─────────────────▼──────────────────────────────────────────┐  │  │
│  │  GATE 5: Personalization                                   │  │  │
│  │  Does the output match the user's taste profile?           │  │  │
│  │  Measured by: taste alignment score                        │  │  │
│  │  Fail: re-rank results, don't discard (user may want fresh) │  │  │
│  └────────────────────────────────────────────────────────────┘  │  │
│                    │                                                │
│                    ▼                                                │
│            Deliver to user                                          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 6. Latency Optimization

### Latency Budget

```
Target: < 3 seconds for a standard generation (25th-75th percentile)

Per-agent latency targets:
  ┌─────────────────────────┬──────────┬──────────┬──────────────┐
  │ Agent                   │ P50      │ P99      │ Parallel?    │
  ├─────────────────────────┼──────────┼──────────┼──────────────┤
  │ Genre Agent             │ 200ms    │ 500ms    │ ✅ Yes       │
  │ Transient Agent         │ 400ms    │ 1000ms   │ ✅ Yes       │
  │ Timbre Agent            │ 400ms    │ 1000ms   │ ✅ Yes       │
  │ Body/Tail Agent         │ 300ms    │ 800ms    │ ❌ No        │
  │ Synthesis (gather)      │ 50ms     │ 100ms    │ N/A          │
  │ Repair Agent            │ 300ms    │ 1000ms   │ ❌ No        │
  │ Mix Placement Agent     │ 200ms    │ 500ms    │ ❌ No        │
  │ Quality Evaluator       │ 200ms    │ 500ms    │ ✅ Yes       │
  │ Metadata Agent          │ 100ms    │ 300ms    │ ✅ Yes       │
  │ Personalization Agent   │ 50ms     │ 100ms    │ ✅ Yes       │
  ├─────────────────────────┼──────────┼──────────┼──────────────┤
  │ Total (parallel path)   │ 400ms    │ 1000ms   │              │
  │ Total (serial path)     │ 750ms    │ 2400ms   │              │
  │ Total (with overhead)   │ ~1.5s    │ ~3.5s    │              │
  └─────────────────────────┴──────────┴──────────┴──────────────┘

  Standard generation target: < 2s wall clock
  High quality target: < 5s wall clock
```

### Optimization Strategies

```
Strategy 1 — Agent Warm-Start:
  Problem: First generation after cold start is slow (model loading).
  Solution: Pre-warm agents in background pool.
    ~Agent Pool~: 5 instances of each agent running idle
    New request: pick warm agent → 0ms cold start
    Cost: memory (500MB per agent instance)

Strategy 2 — Cached Routing:
  Problem: Similar prompts trigger the same agent pipeline.
  Solution: Cache prompt → agent_params mappings.
    80% of prompts are variations of common patterns.
    Cache hit: skip agent inference, use cached params.
    30% latency reduction.

Strategy 3 — Speculative Execution:
  Problem: Sequential agents wait for predecessor.
  Solution: Run ALL agents speculatively in parallel.
    Orchestrator predicts which agents will be needed.
    Runs them all simultaneously with predicted inputs.
    When actual dependencies resolve, most agents already done.
    50% latency reduction for serial paths.

Strategy 4 — Adaptive Quality Threshold:
  Problem: Quality gate iterations add 2-3x latency.
  Solution: Adjust threshold based on context.
    Quick mode (default): threshold=70, max 1 iteration
    HQ mode: threshold=85, max 3 iterations
    Batch mode: threshold=65 (pack sounds can be curated later)
    User-requested: use their historical threshold

Strategy 5 — Agent Model Distillation:
  Problem: Full-size models are slow.
  Solution: Distill agents into faster variants.
    Full model: 1B params, 500ms latency
    Distilled: 100M params, 100ms latency, 95% quality
    Use distilled for quick generations, full for HQ mode
    Distilled + full ensemble for critical quality decisions

Strategy 6 — Request Batching:
  Problem: Individual requests waste GPU utilization.
  Solution: Batch similar requests through agents.
    Transient agent: batch 8 requests → 1 inference → 8 results
    8x throughput, same latency per sound
    Optimal for pack generation
```

### Deployment Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Edge (CDN):                                                       │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │  • Static assets                                            │   │
│  │  • Sound cache (recent generations)                         │   │
│  │  • User session state                                       │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Application Servers:                                              │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │  • Orchestrator (routing, execution, quality control)      │   │
│  │  • Memory service (working + session)                       │   │
│  │  • Personalization agent (CPU, fast)                       │   │
│  │  • Metadata agent (CPU, fast)                              │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  GPU Cluster (Inference):                                          │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ │   │
│  │  │ Transient │ │  Timbre   │ │   Body    │ │  Repair   │ │   │
│  │  │ Agent     │ │  Agent    │ │  Agent    │ │  Agent    │ │   │
│  │  │ (T4 × 4)  │ │  (T4 × 4) │ │  (T4 × 2) │ │  (T4 × 2) │ │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘ │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐                │   │
│  │  │  Genre    │ │  Quality  │ │  Mix      │                │   │
│  │  │  Agent    │ │  Agent    │ │  Agent    │                │   │
│  │  │  (CPU)    │ │  (T4 × 2) │ │  (T4 × 2) │                │   │
│  │  └───────────┘ └───────────┘ └───────────┘                │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Vector Database:                                                  │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │  • Search agent (FAISS)                                    │   │
│  │  • Knowledge base (genre profiles, cultural refs)          │   │
│  │  • User taste embeddings                                   │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Data Layer:                                                       │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │  • PostgreSQL (user data, agent memory, metadata)          │   │
│  │  • Redis (working memory, session memory, cache)           │   │
│  │  • S3 (audio files, generation outputs)                    │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 7. Implementation Roadmap

```
Phase 1 — Agent Foundation (2 months):
  ✓ Define agent interface and communication protocol
  ✓ Build blackboard system (Redis-backed)
  ✓ Implement orchestrator (request routing + execution graph)
  ✓ Build 3 core agents: Transient, Timbre, Quality Evaluator
  ✓ Integration: prompt → agents → output in < 3 seconds

Phase 2 — Agent Expansion (1 month):
  ✓ Genre Agent
  ✓ Body/Tail Agent
  ✓ Repair Agent
  ✓ Mix Placement Agent
  ✓ Metadata Agent
  ✓ Personalization Agent
  ✓ Search Agent

Phase 3 — Intelligence Layer (2 months):
  ✓ Memory systems (all 5 types)
  ✓ Learning loops (all 5 levels)
  ✓ Quality gates (all 5 levels)
  ✓ Dynamic orchestration (learn routing patterns)
  ✓ Feedback integration (implicit + explicit)

Phase 4 — Optimization (1 month):
  ✓ Agent warm-start pool
  ✓ Cached routing
  ✓ Speculative execution
  ✓ Adaptive quality thresholds
  ✓ Model distillation for fast path
  ✓ Request batching for pack generation

Phase 5 — Evolution (ongoing):
  ✓ A/B test single-model vs multi-agent (target: 15% quality improvement)
  ✓ Add/remove agents based on utility metrics
  ✓ Fine-tune agents on user feedback monthly
  ✓ Distill ensemble into single fast model for edge deployment

Total timeline: ~6 months to full multi-agent system
```

---

## 8. Summary

```
Multi-Agent Audio Pipeline

  Core insight:
    One model doing everything hits a quality ceiling.
    Specialized agents collaborating surpass it.

  Architecture:
    11 agents across 4 categories:
      Generation: Transient, Timbre, Genre, Body/Tail
      Post-processing: Repair, Mix Placement
      Knowledge: Metadata, Quality Evaluator, Pack Organizer
      Orchestration: Personalization, Search

    Communication: Blackboard pattern (shared state via Redis)
    Orchestration: Dynamic DAG execution with parallel fan-out
    Memory: 5-tier (working → session → user → agent → global)
    Evaluation: 5-level (agent → quality → implicit → explicit → downstream)

  Latency:
    Standard: < 2s wall clock (parallel agents)
    High quality: < 5s (more iterations)
    Pack mode: < 30s for 20 sounds (batched)

  Key advantage:
    Each agent masters ONE thing.
    Agents can be added, removed, upgraded independently.
    The system improves by improving any single agent.
    The orchestrator learns which agents work best for which requests.

  Multi-agent isn't just an architecture choice.
  It's a scalability strategy for quality improvement.
```

