# Week 35 — VST/Plugin Feasibility Architecture

## Goal
A VST3/AU plugin that can request one-shot generation from the local cShot engine.

## Architecture

```
┌─────────────────────────┐
│   DAW (Ableton/FL)      │
│   ┌─────────────────┐   │
│   │  cShot VST       │   │
│   │  - UI: prompt    │   │
│   │  - Preview btn   │   │
│   │  - Render btn    │   │
│   └────────┬─────────┘   │
└────────────┼─────────────┘
             │ IPC (local socket / HTTP)
             ▼
┌─────────────────────────┐
│  cShot Engine (Python)  │
│  - CLI backend          │
│  - Audio generation     │
│  - Returns WAV via IPC  │
└─────────────────────────┘
```

## IPC Bridge

### Option A: Local HTTP Server (Recommended)
- Python Flask/FastAPI server runs on localhost:8765
- VST plugin sends HTTP POST with prompt params
- Server generates audio, returns WAV bytes
- Simple, proven, cross-platform

### Option B: ZeroMQ / Named Pipes
- Lower latency for streaming
- More complex setup
- Overkill for one-shot generation

## API Design

```python
POST /api/generate
{
  "prompt": "dark 808 kick",
  "seed": 42,
  "duration_ms": 500
}
→ 200 OK
Content-Type: audio/wav
[WAV binary data]

POST /api/preview
{
  "prompt": "dark 808 kick",
  "duration_ms": 200
}
→ 200 OK
Content-Type: audio/wav
[Short preview WAV]
```

## VST Scaffold (Rust)

Use `vst-rs` crate:
```rust
struct CShotPlugin {
    client: reqwest::Client,
    base_url: String,
}

impl Plugin for CShotPlugin {
    fn new(_host: &HostInfo) -> Self { ... }
    fn get_info(&self) -> Info { ... }
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) { ... }
}
```

## Next Steps
1. Build local HTTP server wrapping gen/cli.py
2. Verify: `curl -X POST localhost:8765/api/generate -d '{"prompt":"kick"}' > out.wav`
3. Scaffold VST plugin with UI (prompt text box + generate button)
4. Connect VST to local server
5. Test in Ableton/FL Studio
