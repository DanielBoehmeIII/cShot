# Week 36 — Cloud Feasibility Architecture

## Goal
Optional cloud version: upload song/sample → queue → render → download.

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Web Client  │────▶│  API Server  │────▶│  Job Queue   │
│  (React)     │     │  (FastAPI)   │     │  (Redis/RQ)  │
└──────────────┘     └──────┬───────┘     └──────┬───────┘
                            │                     │
                            ▼                     ▼
                     ┌──────────────┐     ┌──────────────┐
                     │  File Store  │     │  Worker(s)   │
                     │  (S3/Local)  │     │  (cShot gen) │
                     └──────────────┘     └──────────────┘
```

## Components

### API Server
- FastAPI with JWT auth
- Endpoints:
  - `POST /api/kits` — create generation job
  - `GET /api/kits/{id}` — check status
  - `GET /api/kits/{id}/download` — download ZIP
- File upload: multipart, max 50MB

### Job Queue
- Redis + RQ (simple) or Celery (scalable)
- Each job: `{"prompt": "...", "count": 40, "user_id": "..."}`
- Status: queued → generating → completed → failed

### Workers
- Python processes running `gen/cli.py` commands
- Each worker handles one job at a time
- Output stored to file store
- Worker pool: 1-10 depending on load

### File Store
- Local filesystem for MVP
- S3-compatible for production
- Kit retention: 7 days, then cleanup

### Storage Limits (MVP)
- Free tier: 5 kits/month, 20 sounds per kit
- Paid tier: unlimited
- File retention: 7 days

## Prototype Job Runner

```python
# Simple job runner using subprocess
import subprocess, uuid, json
from pathlib import Path

def run_generation_job(prompt: str, count: int) -> dict:
    job_id = str(uuid.uuid4())[:8]
    out_dir = Path(f"/tmp/cshot_jobs/{job_id}")
    out_dir.mkdir(parents=True)

    result = subprocess.run(
        ["python3", "gen.py", "make", prompt,
         "--count", str(count),
         "--out", str(out_dir)],
        capture_output=True, text=True
    )

    wav_files = list(out_dir.rglob("*.wav"))
    return {
        "job_id": job_id,
        "status": "completed" if result.returncode == 0 else "failed",
        "files": len(wav_files),
        "output_dir": str(out_dir),
        "log": result.stdout[-500:],
    }
```

## Cost Estimate (per 1000 jobs)
- CPU time: ~3 seconds per 20-sound kit
- 1000 kits = ~50 minutes CPU
- $0.50-2.00 on cloud (varies by provider)
- Storage: negligible (WAVs deleted after 7 days)

## Next Steps
1. Build the API server (FastAPI)
2. Implement job queue with Redis
3. Deploy worker on a single VPS
4. Test: upload song via curl → poll status → download kit
