#!/usr/bin/env python3
"""cShot Web UI — make, from-song, from-sample, preview, curate, export."""
import json
import shutil
import sys
import tempfile
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(REPO_ROOT))

try:
    import gradio as gr
except ImportError:
    print("Install gradio: pip install gradio")
    sys.exit(1)

from gen.kit_spec import infer_spec_from_prompt
from gen.kit_engine import generate_kit, setup_kit_export, compute_kit_coherence
from gen.quality_gate import run_all_gates
from gen.rating import _load_ratings, _save_rating
from gen.io import read_wav
from gen.polish import polish_file
from gen.rank import score_file

OUTPUTS = REPO_ROOT / "outputs" / "ui"
OUTPUTS.mkdir(parents=True, exist_ok=True)


def ui_make(prompt, count):
    if not prompt.strip():
        return None, "Enter a prompt", "", gr.update(visible=False)
    kit_dir = OUTPUTS / f"make_{prompt.replace(' ', '_')[:20]}_{int(time.time())}"
    spec = infer_spec_from_prompt(prompt)
    total = sum(v for k, v in vars(spec.categories).items() if isinstance(v, int))
    if total > 0 and total != count:
        ratio = count / total
        for field in vars(spec.categories):
            c = getattr(spec.categories, field)
            if isinstance(c, int):
                setattr(spec.categories, field, max(1, int(round(c * ratio))))
    spec.total_target = count
    t0 = time.time()
    generated = generate_kit(spec, kit_dir, polish=True)
    elapsed = time.time() - t0
    wavs = sorted(kit_dir.rglob("*.wav"))
    gate = run_all_gates(wavs)
    for p, r in gate["results"].items():
        if not r["pass"]:
            Path(p).unlink(missing_ok=True)
    remaining = sorted(kit_dir.rglob("*.wav"))
    coherence = compute_kit_coherence(kit_dir)
    setup_kit_export(kit_dir, spec, coherence)
    msg = f"Generated {len(remaining)}/{generated} sounds in {elapsed:.1f}s | Coherence: {coherence.get('overall_coherence', 0):.3f}"
    audio_paths = [str(w) for w in remaining[:20]]
    return kit_dir, msg, f"Kit ready at {kit_dir}", gr.update(visible=True, value=kit_dir)


def ui_from_song(song_file, count):
    if song_file is None:
        return None, "Upload a song file", "", gr.update(visible=False)
    from gen.kit_engine import cmd_kit_from_song
    song_path = Path(song_file.name) if hasattr(song_file, 'name') else Path(song_file)
    kit_dir = OUTPUTS / f"from_song_{song_path.stem[:20]}_{int(time.time())}"
    import types
    args = types.SimpleNamespace(song=str(song_path), count=count, out=str(kit_dir), style="inspired")
    cmd_kit_from_song(args)
    wavs = sorted(kit_dir.rglob("*.wav"))
    msg = f"Generated {len(wavs)} sounds from {song_path.name}"
    audio_paths = [str(w) for w in wavs[:20]]
    return kit_dir, msg, str(kit_dir), gr.update(visible=True, value=str(kit_dir))


def ui_from_sample(sample_file, count):
    if sample_file is None:
        return None, "Upload a sample file", "", gr.update(visible=False)
    from gen.kit_engine import cmd_kit_from_sample
    sample_path = Path(sample_file.name) if hasattr(sample_file, 'name') else Path(sample_file)
    kit_dir = OUTPUTS / f"from_sample_{sample_path.stem[:20]}_{int(time.time())}"
    import types
    args = types.SimpleNamespace(sample=str(sample_path), count=count, out=str(kit_dir))
    cmd_kit_from_sample(args)
    wavs = sorted(kit_dir.rglob("*.wav"))
    msg = f"Generated {len(wavs)} sounds from {sample_path.name}"
    audio_paths = [str(w) for w in wavs[:20]]
    return kit_dir, msg, str(kit_dir), gr.update(visible=True, value=str(kit_dir))


def ui_listen(kit_dir_path):
    if not kit_dir_path:
        return "Select a kit directory first", "", ""
    kit_dir = Path(kit_dir_path)
    wavs = sorted(kit_dir.rglob("*.wav"))
    if not wavs:
        return "No WAV files found", "", ""
    ratings = _load_ratings()
    scored = []
    for w in wavs:
        s = score_file(w, ratings)
        s["path"] = str(w.relative_to(kit_dir))
        scored.append(s)
    scored.sort(key=lambda x: x.get("score", 0), reverse=True)
    top = scored[:5]
    msg = f"{len(wavs)} files in kit"
    fav_count = sum(1 for r in ratings if r.get("rating") == "favorite")
    return msg, f"Favorites: {fav_count}", json.dumps([s["path"] for s in top], indent=2)


def ui_export_favorites(kit_dir_path):
    if not kit_dir_path:
        return "No kit directory"
    kit_dir = Path(kit_dir_path)
    export_dir = kit_dir / "_favorites"
    export_dir.mkdir(exist_ok=True)
    count = 0
    for r in _load_ratings():
        if r["rating"] == "favorite":
            fp = REPO_ROOT / r["file"]
            if fp.exists() and fp.suffix == ".wav":
                shutil.copy2(fp, export_dir / fp.name)
                count += 1
    return f"Exported {count} favorites to {export_dir}"


with gr.Blocks(title="cShot", theme=gr.themes.Soft()) as demo:
    gr.Markdown("# cShot — One-Shot Kit Generator")
    gr.Markdown("Create custom one-shot kits from descriptions, songs, or samples.")

    kit_state = gr.State()

    with gr.Tab("Make from Description"):
        with gr.Row():
            prompt_input = gr.Textbox(label="Kit description", placeholder="e.g. dark rnb one shot kit", scale=3)
            count_slider = gr.Slider(10, 200, value=40, label="Kit size", step=10)
        make_btn = gr.Button("Generate Kit", variant="primary")
        make_output = gr.Textbox(label="Result")
        make_dir = gr.Textbox(label="Kit directory", visible=False)
        listen_make = gr.Button("Listen to Kit", visible=False)

    with gr.Tab("From Song"):
        with gr.Row():
            song_input = gr.File(label="Upload song", file_types=[".wav", ".mp3", ".flac", ".ogg"])
            song_count = gr.Slider(10, 100, value=40, label="Kit size", step=10)
        song_btn = gr.Button("Generate from Song", variant="primary")
        song_output = gr.Textbox(label="Result")
        song_dir = gr.Textbox(label="Kit directory", visible=False)

    with gr.Tab("From Sample"):
        with gr.Row():
            sample_input = gr.File(label="Upload sample", file_types=[".wav"])
            sample_count = gr.Slider(10, 60, value=20, label="Kit size", step=5)
        sample_btn = gr.Button("Generate from Sample", variant="primary")
        sample_output = gr.Textbox(label="Result")
        sample_dir = gr.Textbox(label="Kit directory", visible=False)

    with gr.Tab("Listen & Curate"):
        listen_info = gr.Textbox(label="Kit info")
        listen_favs = gr.Textbox(label="Favorites")
        listen_top = gr.Textbox(label="Top sounds")
        listen_btn = gr.Button("Refresh")
        export_btn = gr.Button("Export Favorites", variant="secondary")
        export_output = gr.Textbox(label="Export result")

    make_btn.click(ui_make, [prompt_input, count_slider], [kit_state, make_output, make_dir, listen_make])
    listen_make.click(ui_listen, [make_dir], [listen_info, listen_favs, listen_top])
    song_btn.click(ui_from_song, [song_input, song_count], [kit_state, song_output, song_dir, listen_make])
    sample_btn.click(ui_from_sample, [sample_input, sample_count], [kit_state, sample_output, sample_dir, listen_make])
    listen_btn.click(ui_listen, [kit_state], [listen_info, listen_favs, listen_top])
    export_btn.click(ui_export_favorites, [kit_state], [export_output])

if __name__ == "__main__":
    demo.launch()
