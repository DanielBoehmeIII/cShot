#!/usr/bin/env python3
"""cShot Gradio UI — prompt, generate, preview, rate, export."""
import sys
from pathlib import Path

# Ensure cshot root is in path
REPO_ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(REPO_ROOT))

try:
    import gradio as gr
except ImportError:
    print("Install gradio: pip install gradio")
    sys.exit(1)

from gen.prompt import parse_prompt, generate_single_from_prompt, _seed_from_prompt, _write_metadata
from gen.io import write_wav
from gen.polish import polish_file
from gen.rating import cmd_rate
from gen.rank import score_file
from gen.rating import _load_ratings
import numpy as np
import random
import tempfile
import json
import time


def generate_audio(prompt_text, seed_input):
    """Generate audio from prompt and return WAV path + info."""
    if not prompt_text.strip():
        return None, "Please enter a prompt"

    parsed = parse_prompt(prompt_text)

    try:
        if seed_input and seed_input.strip():
            seed = int(seed_input)
        else:
            seed = _seed_from_prompt(prompt_text)

        np.random.seed(seed % 2**32)
        samples = generate_single_from_prompt(parsed)

        tmp = tempfile.NamedTemporaryFile(suffix=".wav", delete=False)
        tmp_path = Path(tmp.name)
        tmp.close()
        write_wav(tmp_path, samples)
        _write_metadata(tmp_path, parsed, seed, 0, 0)

        info = (
            f"Family: {parsed['family']}  |  Profile: {parsed['default_profile']}  |  "
            f"Adjectives: {', '.join(parsed['adjectives']) or 'none'}  |  Seed: {seed}\n"
            f"Duration: {len(samples)/44100:.2f}s"
        )
        return str(tmp_path), info
    except Exception as e:
        return None, f"Error: {e}"


def rate_audio(wav_path, rating):
    """Rate a generated audio file."""
    if not wav_path:
        return "Generate a file first"
    try:
        p = Path(wav_path)
        cmd_rate(type("args", (), {"file": str(p), "rating": rating, "notes": ""})())
        return f"Rated as {rating}"
    except Exception as e:
        return f"Error: {e}"


def export_audio(wav_path, export_name):
    """Export with producer-friendly name."""
    if not wav_path:
        return "Generate a file first", ""
    src = Path(wav_path)
    name = export_name.strip() or src.stem
    out_dir = Path("outputs/ui_exports")
    out_dir.mkdir(parents=True, exist_ok=True)
    dest = out_dir / f"{name}.wav"
    import shutil
    shutil.copy2(src, dest)
    return f"Exported → {dest}", str(dest)


# Build Gradio UI
with gr.Blocks(title="cShot — One-Shot Generator", theme=gr.themes.Soft()) as demo:
    gr.Markdown("# cShot — AI One-Shot Generator")
    gr.Markdown("Type a prompt, generate, rate, export.")

    with gr.Row():
        prompt_input = gr.Textbox(label="Prompt", placeholder="e.g. dark soft piano stab",
                                  scale=3)
        seed_input = gr.Textbox(label="Seed (optional)", placeholder="42", scale=1)
        gen_btn = gr.Button("Generate", variant="primary", scale=1)

    with gr.Row():
        audio_output = gr.Audio(label="Generated", type="filepath")
        info_output = gr.Textbox(label="Info", lines=3)

    gen_btn.click(generate_audio, inputs=[prompt_input, seed_input],
                  outputs=[audio_output, info_output])

    with gr.Row():
        fav_btn = gr.Button("★ Favorite")
        good_btn = gr.Button("✓ Good")
        bad_btn = gr.Button("✗ Bad")
        trash_btn = gr.Button("🗑 Trash")
        status_output = gr.Textbox(label="Status", lines=1)

    fav_btn.click(rate_audio, inputs=[audio_output, gr.State("favorite")], outputs=status_output)
    good_btn.click(rate_audio, inputs=[audio_output, gr.State("good")], outputs=status_output)
    bad_btn.click(rate_audio, inputs=[audio_output, gr.State("bad")], outputs=status_output)
    trash_btn.click(rate_audio, inputs=[audio_output, gr.State("trash")], outputs=status_output)

    with gr.Row():
        export_name = gr.Textbox(label="Export filename", placeholder="my_dark_piano", scale=2)
        export_btn = gr.Button("Export", scale=1)
        export_output = gr.Textbox(label="Export status", lines=1)

    export_btn.click(export_audio, inputs=[audio_output, export_name],
                     outputs=[export_output, gr.State()])

    gr.Markdown("### Examples")
    gr.Examples(
        examples=[
            ["dark soft piano stab", ""],
            ["bright hard piano stab", ""],
            ["punchy kick 808", "42"],
            ["warm synth pluck", ""],
            ["aggressive distorted bass", ""],
            ["cinematic impact fx", ""],
            ["clean nylon guitar", ""],
            ["lo-fi dusty keys", ""],
        ],
        inputs=[prompt_input, seed_input],
    )

if __name__ == "__main__":
    demo.launch()
