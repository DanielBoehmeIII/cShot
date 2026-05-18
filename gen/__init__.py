SAMPLE_RATE = 44100

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
TAURI_DIR = REPO_ROOT / "src-tauri"
PACKS_DIR = TAURI_DIR / "Packs"
SPANISH_GUITAR_DIR = TAURI_DIR / "Spanish Guitar" / "Spanish Guitar"
SUPPORTED_EXTS = {'.wav', '.wave', '.wv', '.aif', '.aiff', '.aifc', '.flac', '.ogg'}
