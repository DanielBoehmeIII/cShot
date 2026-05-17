#!/usr/bin/env python3
"""cShot Gradio UI — prompt, generate, pack build, rate, export."""
import sys, json, tempfile, time, random, shutil
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(REPO_ROOT))

try:
    import gradio as gr
except ImportError:
    print("Install gradio: pip install gradio")
    sys.exit(1)

import numpy as np
from gen.prompt import parse_prompt, generate_single_from_prompt, _seed_from_prompt, _write_metadata
from gen.io import write_wav
from gen.polish import polish_file
from gen.rating import cmd_rate
from gen.rank import score_file
from gen.rating import _load_ratings
from gen.pack import cmd_pack
from gen.genre import GENRE_PROFILES

TEMP_FILES = []


def _cleanup():
    for f in TEMP_FILES:
        try:
            Path(f).unlink()
        except:
            pass


def generate_audio(prompt_text, seed_input):
    if not prompt_text.strip():
        return None, "Enter a prompt", ""
    parsed = parse_prompt(prompt_text)
    try:
        seed = int(seed_input) if seed_input.strip() else _seed_from_prompt(prompt_text)
        np.random.seed(seed % 2**32)
        samples = generate_single_from_prompt(parsed)
        tmp = Path(tempfile.mktemp(suffix=".wav"))
        TEMP_FILES.append(str(tmp))
        write_wav(tmp, samples)
        _write_metadata(tmp, parsed, seed, 0, 0)
        info = (f"Family: {parsed['family']} | Profile: {parsed['default_profile']} | "
                f"Adjectives: {', '.join(parsed['adjectives']) or 'none'} | Seed: {seed}\n"
                f"Duration: {len(samples)/44100:.2f}s")
        return str(tmp), info, str(tmp)
    except Exception as e:
        return None, f"Error: {e}", ""


def rate_audio_ui(wav_path, rating):
    if not wav_path:
        return "Generate a file first"
    try:
        p = Path(wav_path)
        if not p.exists():
            return f"File not found: {wav_path}"
        cmd_rate(type("args", (), {"file": str(p), "rating": rating, "notes": ""})())
        return f"Rated as {rating}"
    except Exception as e:
        return f"Error: {e}"


def export_audio_ui(wav_path, export_name):
    if not wav_path:
        return "Generate a file first", ""
    src = Path(wav_path)
    if not src.exists():
        return f"File not found", ""
    name = export_name.strip() or src.stem
    out_dir = Path("outputs/ui_exports")
    out_dir.mkdir(parents=True, exist_ok=True)
    dest = out_dir / f"{name}.wav"
    shutil.copy2(src, dest)
    return f"Exported → {dest}", str(dest)


def generate_pack_ui(theme, count, genre):
    if not theme.strip():
        return "Enter a theme", ""
    try:
        count_int = int(count) if count.strip() else 60
    except ValueError:
        return "Invalid count", ""
    if genre and genre != "none":
        prompt = f"{genre} {theme}"
    else:
        prompt = theme
    out_dir = REPO_ROOT / "outputs" / f"ui_pack_{int(time.time())}"
    out_dir.mkdir(parents=True, exist_ok=True)
    fake_args = type("Args", (), {"prompt": prompt.split(), "count": count_int, "out": str(out_dir)})()
    cmd_pack(fake_args)
    wavs = list(out_dir.rglob("*.wav"))
    return f"Generated {len(wavs)} files → {out_dir}", str(out_dir)


def search_ref_ui(query):
    if not query.strip():
        return "Enter a search query"
    from gen.search_ref import cmd_search_ref
    import io, contextlib
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        try:
            cmd_search_ref(type("args", (), {"query": query.split()}))
        except SystemExit:
            pass
    return buf.getvalue() or "No results"


# ─── Build UI ───
with gr.Blocks(title="cShot — One-Shot Generator", theme=gr.themes.Soft()) as demo:
    gr.Markdown("# cShot — AI One-Shot Generator")
    gr.Markdown("Type a prompt, generate, rate, export. Or build a full pack.")

    with gr.Tabs():
        # ── Tab 1: Single Generation ──
        with gr.TabItem("Generate"):
            with gr.Row():
                prompt_input = gr.Textbox(label="Prompt", placeholder="e.g. dark soft piano stab", scale=3)
                seed_input = gr.Textbox(label="Seed (optional)", placeholder="42", scale=1)
                gen_btn = gr.Button("Generate", variant="primary", scale=1)
            with gr.Row():
                audio_output = gr.Audio(label="Generated", type="filepath")
                info_output = gr.Textbox(label="Info", lines=4)
            with gr.Row():
                fav_btn = gr.Button("★ Favorite")
                good_btn = gr.Button("✓ Good")
                bad_btn = gr.Button("✗ Bad")
                trash_btn = gr.Button("🗑 Trash")
                status_output = gr.Textbox(label="Status", lines=1)
            fav_btn.click(rate_audio_ui, inputs=[audio_output, gr.State("favorite")], outputs=status_output)
            good_btn.click(rate_audio_ui, inputs=[audio_output, gr.State("good")], outputs=status_output)
            bad_btn.click(rate_audio_ui, inputs=[audio_output, gr.State("bad")], outputs=status_output)
            trash_btn.click(rate_audio_ui, inputs=[audio_output, gr.State("trash")], outputs=status_output)
            with gr.Row():
                export_name = gr.Textbox(label="Export filename", placeholder="my_sound", scale=2)
                export_btn = gr.Button("Export", scale=1)
                export_output = gr.Textbox(label="Export status", lines=1)
            export_btn.click(export_audio_ui, inputs=[audio_output, export_name], outputs=[export_output, gr.State()])
            gen_btn.click(generate_audio, inputs=[prompt_input, seed_input], outputs=[audio_output, info_output, gr.State()])
            gr.Examples(
                examples=[
                    ["dark soft piano stab", ""],
                    ["bright hard piano stab", "42"],
                    ["punchy kick 808", ""],
                    ["warm synth pluck", ""],
                    ["aggressive distorted bass", ""],
                    ["cinematic impact fx", ""],
                    ["clean nylon guitar", ""],
                    ["lo-fi dusty keys", ""],
                ],
                inputs=[prompt_input, seed_input],
            )

        # ── Tab 2: Pack Builder ──
        with gr.TabItem("Pack Builder"):
            gr.Markdown("### Build a themed one-shot pack")
            with gr.Row():
                pack_theme = gr.Textbox(label="Pack Theme", placeholder="e.g. dark rnb one shots", scale=2)
                pack_count = gr.Textbox(label="Target Count", value="60", scale=1)
                pack_genre = gr.Dropdown(
                    choices=["none"] + sorted(GENRE_PROFILES.keys()),
                    value="none", label="Genre", scale=1
                )
            pack_btn = gr.Button("Generate Pack", variant="primary")
            with gr.Row():
                pack_output = gr.Textbox(label="Pack Result", lines=2)
                pack_preview = gr.File(label="Pack Folder")
            pack_btn.click(generate_pack_ui, inputs=[pack_theme, pack_count, pack_genre],
                          outputs=[pack_output, pack_preview])

        # ── Tab 3: Rate History ──
        with gr.TabItem("Ratings"):
            gr.Markdown("### Your Ratings")
            refresh_btn = gr.Button("Refresh")
            ratings_display = gr.Textbox(label="Summary", lines=10)
            def show_ratings():
                from gen.rating import cmd_ratings_summary, cmd_favorites
                import io, contextlib
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    try:
                        cmd_ratings_summary(type("args", (), {"action": "summary"}))
                    except SystemExit:
                        pass
                return buf.getvalue()
            refresh_btn.click(show_ratings, outputs=ratings_display)

    gr.Markdown("### Quick Reference")
    gr.Markdown("Adjectives: bright, dark, warm, soft, hard, punchy, distorted, clean, wide, narrow, lo-fi, airy, vintage, analog, digital, dusty, glossy, crunchy, huge, tiny, dry, wet, aggressive, mellow, edgy, crisp, expensive, smooth, rough")

if __name__ == "__main__":
    import atexit
    atexit.register(_cleanup)
    demo.launch()
