"""
cShot — custom one-shot kits from songs, samples, genres, and vibes.

Product commands:
  cshot make "dark rnb one shot kit"     Generate a complete kit from a description
  cshot from-song song.wav               Generate a kit from a song analysis
  cshot from-sample sample.wav           Generate a mini kit from a reference sample

Producer tools:
  cshot listen <dir>                     Interactive listening session
  cshot rate <file> --rating good|bad    Rate a generated sound
  cshot favorites                        List all favorited files
  cshot rank <dir>                       Rank files by quality
  cshot taste                            Show your learned taste profile

Advanced:
  cshot lab <command>                    Access research and development commands
"""

import argparse
import sys

from gen.scanning import cmd_scan, cmd_profiles
from gen.commands import cmd_oneshot, cmd_batch, cmd_qa, cmd_compare, cmd_all
from gen.refinement import (
    cmd_analyze_output, cmd_compare_to_references, cmd_diagnose,
    cmd_refine, cmd_audit_report,
)
from gen.health import cmd_dataset_health
from gen.listen import cmd_listen, cmd_listening_report
from gen.rating import cmd_rate, cmd_ratings_summary, cmd_favorites
from gen.clustering import cmd_cluster_references
from gen.cluster_report import cmd_cluster_report
from gen.cluster_gen import cmd_cluster_gen, cmd_cluster_qa
from gen.pitch import cmd_detect_pitch
from gen.piano import cmd_analyze_piano
from gen.synth import cmd_analyze_synth
from gen.tonal_qa import cmd_tonal_qa
from gen.piano_gen import cmd_piano_gen, cmd_piano_audit, PIANO_CLASSES
from gen.synth_gen import cmd_synth_gen, cmd_synth_refine, SYNTH_PROFILES
from gen.guitar_gen import cmd_guitar_gen, cmd_guitar_qa, GUITAR_PROFILES
from gen.bass_gen import cmd_bass_gen, cmd_bass_qa, BASS_PROFILES
from gen.fx_gen import cmd_fx_gen, cmd_fx_qa, FX_PROFILES
from gen.prompt import cmd_prompt, cmd_prompt_refine, cmd_mvp_audit, cmd_compare_prompt, cmd_contrast_test, cmd_regenerate
from gen.presets import cmd_save_preset, cmd_preset_list, cmd_preset_generate
from gen.polish import cmd_polish
from gen.kit_spec import cmd_kit_spec, KitSpec, infer_spec_from_prompt, GENRE_DNA_BIAS, KIT_SPEC_TEMPLATES
from gen.kit_engine import (
    cmd_kit_from_pack_dna, cmd_kit_audit, cmd_kit_from_folder,
    cmd_kit_from_sample, cmd_plan_kit, cmd_kit_from_song,
    cmd_mini_kit_from_song,
    cmd_genre_dna, cmd_list_genre_dna, cmd_kit_rank,
    cmd_kit_repair, cmd_kit_variations, cmd_kit_naming,
    cmd_kit_from_description, cmd_kit_export, cmd_kit_similarity,
    cmd_more_like_kit, cmd_merge_kits, cmd_kit_preset,
    cmd_kit_factory, cmd_export_kit,
)
from gen.pack import cmd_pack, cmd_pack_audit, cmd_theme
from gen.similar import cmd_similar, cmd_variations
from gen.genre import cmd_genre, GENRE_PROFILES
from gen.rank import cmd_rank, cmd_top
from gen.taste import (
    cmd_taste_profile, cmd_taste_prompt_history,
    cmd_learn_from_folder, cmd_export_cshotpack, cmd_import_cshotpack,
)
from gen.make import cmd_make
from gen.demo import cmd_demo
from gen.onboard import cmd_onboard
from gen.quality_gate import cmd_gate
from gen.curate import cmd_curate_pack
from gen.feedback import cmd_feedback_pack
from gen.export_daw import cmd_export_daw
from gen.marketplace import cmd_marketplace_serve
from gen.quality_pass import cmd_quality_pass
from gen.demo_kits import cmd_build_demo_kits
from gen.beta import cmd_beta_round
from gen.search_ref import cmd_search_ref
from gen.retrieval import cmd_retrieve, cmd_like, cmd_kit_from_folder_retrieval
from gen.blend import cmd_blend
from gen.refine_feedback import cmd_refine_feedback
from gen.pack_census import cmd_pack_census, cmd_pack_census_quick
from gen.semantics import cmd_semantics, cmd_semantics_report
from gen.pack_clustering import cmd_pack_cluster
from gen.pack_dna import cmd_pack_dna, cmd_pack_dna_report
from gen.recreate import cmd_recreate, cmd_recreate_folder, cmd_recreate_audit
from gen.style_embed import (
    cmd_embed, cmd_embed_folder, cmd_pack_style,
    cmd_style_transfer, cmd_variation_chain, cmd_pack_style_viz,
)
from gen.hybrid import cmd_cross_blend, cmd_hybrid, cmd_mutate
from gen.dataset_intel import cmd_find_duplicates, cmd_quality_rank, cmd_producer_fingerprint, cmd_imitate
from gen.model_audio import (
    cmd_encode, cmd_decode, cmd_interpolate, cmd_latent_mutate,
    cmd_rag_generate, cmd_contrastive_pairs, cmd_diffuse,
)
from gen.product import cmd_make_pack, cmd_verticality_audit
from gen.song import cmd_song_dna, cmd_section_dna, cmd_song_palette


def main():
    parser = argparse.ArgumentParser(
        description="cShot — custom one-shot kits from songs, samples, genres, and vibes"
    )
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # ═══════════════════════════════════════════════════════
    #  PRODUCT COMMANDS
    # ═══════════════════════════════════════════════════════

    # ── make ──
    p_make = subparsers.add_parser(
        "make",
        help="Generate a complete one-shot kit from a description",
        description="One-command producer mode: generate, polish, rank, and export a complete one-shot kit from a natural language description.",
    )
    p_make.add_argument("prompt", nargs="+", help="Kit description (e.g. 'dark rnb one shot kit')")
    p_make.add_argument("--count", "-n", type=int, default=100, help="Target file count")
    p_make.add_argument("--out", "-o", help="Output directory")
    p_make.add_argument("--strategy", "-s", choices=["dsp", "reference"], default="reference",
                        help="Generation strategy: dsp (synthetic) or reference (reference-conditioned, default)")

    # ── from-song ──
    p_from_song = subparsers.add_parser(
        "from-song",
        help="Generate a kit from a song analysis",
        description="Analyze a song and generate a custom one-shot kit matching its sonic profile, tempo, key, and mood.",
    )
    p_from_song.add_argument("song", help="Audio file (WAV, MP3, etc.)")
    p_from_song.add_argument("--count", "-n", type=int, default=80, help="Number of files to generate")
    p_from_song.add_argument("--style", "-s", choices=["strict", "inspired", "wild"], default="inspired",
                             help="How closely to match the source song (default: inspired)")
    p_from_song.add_argument("--out", "-o", help="Output directory")
    p_from_song.add_argument("--strategy", choices=["dsp", "reference"], default="reference",
                             help="Generation strategy: dsp or reference (default)")

    # ── from-sample ──
    p_from_sample = subparsers.add_parser(
        "from-sample",
        help="Generate a mini kit from a reference sample",
        description="Analyze a single sample and generate complementary sounds around it to create a coherent mini kit.",
    )
    p_from_sample.add_argument("sample", help="Reference .wav file")
    p_from_sample.add_argument("--count", "-n", type=int, default=30, help="Number of files to generate")
    p_from_sample.add_argument("--kit-size", type=int, help="Alias for --count")
    p_from_sample.add_argument("--out", "-o", help="Output directory")
    p_from_sample.add_argument("--strategy", choices=["dsp", "reference"], default="reference",
                               help="Generation strategy: dsp or reference (default)")

    # ── retrieve ──
    p_retrieve = subparsers.add_parser(
        "retrieve",
        help="Search reference library by text description",
        description="Search the full reference library using natural language and find the best matching sounds by semantic similarity.",
    )
    p_retrieve.add_argument("query", nargs="+", help="Search description (e.g. 'dark rnb clap')")
    p_retrieve.add_argument("--n", type=int, default=20, help="Number of results (default: 20)")

    # ── like ──
    p_like = subparsers.add_parser(
        "like",
        help="Find sounds similar to a reference audio file",
        description="Analyze any audio file and find the most similar sounds in the reference library using feature embeddings.",
    )
    p_like.add_argument("path", help="Path to audio file")
    p_like.add_argument("--n", type=int, default=20, help="Number of results (default: 20)")

    # ── listen ──
    p_listen = subparsers.add_parser(
        "listen",
        help="Interactive listening session",
        description="Rate and take notes on generated sounds with interactive playback.",
    )
    p_listen.add_argument("input_dir", help="Directory of .wav files to rate")
    p_listen.add_argument("--notes", "-n", help="Path to listening_notes.json")

    # ── rate ──
    p_rate = subparsers.add_parser(
        "rate",
        help="Rate a generated sound",
        description="Rate a generated file as good, bad, favorite, or trash.",
    )
    p_rate.add_argument("file", help="Path to the .wav file to rate")
    p_rate.add_argument("--rating", "-r", choices=["good", "bad", "favorite", "trash"], required=True, help="Rating")
    p_rate.add_argument("--notes", "-n", help="Optional notes about this file")

    # ── favorites ──
    subparsers.add_parser(
        "favorites",
        help="List all favorited files",
        description="Show all files you've marked as favorites.",
    )

    # ── rank ──
    p_rank = subparsers.add_parser(
        "rank",
        help="Rank files by quality",
        description="Rank all WAV files in a directory by quality score, combining audio features with your ratings.",
    )
    p_rank.add_argument("input_dir", help="Directory of WAV files to rank")

    # ── ratings ──
    p_ratings = subparsers.add_parser(
        "ratings",
        help="Show rating summary",
        description="Display a summary of all your ratings.",
    )
    p_ratings.add_argument("action", nargs="?", choices=["summary"], default="summary", help="Rating action")

    # ── taste ──
    p_taste = subparsers.add_parser(
        "taste",
        help="Show your learned taste profile",
        description="Display the taste profile cShot has learned from your ratings and favorites.",
    )
    p_taste.add_argument("--rebuild", action="store_true", help="Rebuild profile from latest ratings")

    # ── taste-prompt-history ──
    subparsers.add_parser(
        "taste-prompt-history",
        help="Show prompt-level rating history",
        description="Display which prompts you've favorited or trashed most.",
    )

    # ── learn-from-folder ──
    p_lff = subparsers.add_parser(
        "learn-from-folder",
        help="Build a producer DNA profile from a folder",
        description="Analyze a folder of your favorite WAV files and build a personal DNA profile for generation.",
    )
    p_lff.add_argument("folder", help="Folder of WAV files to learn from")
    p_lff.add_argument("--out", "-o", help="Output path for DNA JSON")

    # ── export-cshotpack ──
    p_export = subparsers.add_parser(
        "export-cshotpack",
        help="Export a kit as .cshotpack",
        description="Package a kit directory into portable .cshotpack format with manifest and optional DNA.",
    )
    p_export.add_argument("kit_dir", help="Kit directory to export")
    p_export.add_argument("--out", "-o", help="Output .cshotpack path")

    # ── import-cshotpack ──
    p_import = subparsers.add_parser(
        "import-cshotpack",
        help="Import a .cshotpack file",
        description="Extract a .cshotpack into a kit directory.",
    )
    p_import.add_argument("pack_file", help="Path to .cshotpack file")
    p_import.add_argument("--out", "-o", help="Output directory")

    # ── marketplace-serve ──
    p_ms = subparsers.add_parser(
        "marketplace-serve",
        help="Generate marketplace page for browsing kits",
        description="Scan generated kits and create an HTML marketplace page with previews.",
    )
    p_ms.add_argument("--kits-dir", help="Directory containing kits (default: outputs/kits)")
    p_ms.add_argument("--out", "-o", help="Output HTML path")

    # ── beta-round ──
    p_br = subparsers.add_parser(
        "beta-round",
        help="Generate beta test kits for producers",
        description="Generate and package kits for producer beta testing with feedback forms.",
    )
    p_br.add_argument("--round", "-r", type=int, default=1, help="Beta round number")
    p_br.add_argument("--count", "-n", type=int, default=40, help="Sounds per kit")
    p_br.add_argument("--producers", "-p", help="Producer indices to include (e.g. '1,3,5')")
    p_br.add_argument("--out", "-o", help="Output base directory")

    # ── quality-pass ──
    p_qp = subparsers.add_parser(
        "quality-pass",
        help="Run a focused quality pass on a sound family",
        description="Generate, gate, rank, and curate sounds for a specific family (drums, bass, tonal, fx).",
    )
    p_qp.add_argument("family", choices=["drums", "bass", "tonal", "fx"], help="Sound family to focus on")
    p_qp.add_argument("--count", "-n", type=int, default=100, help="Sounds to generate (default: 100)")
    p_qp.add_argument("--keep", "-k", type=int, default=25, help="Number to keep after curation (default: 25)")
    p_qp.add_argument("--out", "-o", help="Output directory")

    # ── export-daw ──
    p_daw = subparsers.add_parser(
        "export-daw",
        help="Export a kit for DAW (Ableton/FL Studio)",
        description="Organize a kit with proper folder structure, naming, and key/BPM tags for your DAW.",
    )
    p_daw.add_argument("kit_dir", help="Kit directory to export")
    p_daw.add_argument("--daw", choices=["ableton", "fl"], default="ableton", help="Target DAW (default: ableton)")
    p_daw.add_argument("--out", "-o", help="Output directory")
    p_daw.add_argument("--zipped", "-z", action="store_true", help="Create ZIP archive")

    # ── feedback-pack ──
    p_fb = subparsers.add_parser(
        "feedback-pack",
        help="Package a kit for producer feedback",
        description="Export a kit with rating sheets and feedback form for producer testing.",
    )
    p_fb.add_argument("kit_dir", help="Kit directory to package")
    p_fb.add_argument("--name", "-n", default="producer", help="Producer name")
    p_fb.add_argument("--out", "-o", help="Output directory")

    # ── build-demo-kits ──
    p_bdk = subparsers.add_parser(
        "build-demo-kits",
        help="Build the 5 flagship demo kits",
        description="Generate, curate, and package 5 demo kits: Dark RnB, Alien Trap, PluggnB Keys, Cinematic Impacts, Experimental Textures.",
    )
    p_bdk.add_argument("--count", "-n", type=int, default=30, help="Sounds per kit (default: 30)")
    p_bdk.add_argument("--out", "-o", help="Output base directory")

    # ── curate-pack ──
    p_curate = subparsers.add_parser(
        "curate-pack",
        help="Over-generate and auto-curate a kit",
        description="Generate extra sounds, run quality gates, rank, and keep only the best ones.",
    )
    p_curate.add_argument("prompt", nargs="+", help="Kit description")
    p_curate.add_argument("--target", "-t", type=int, default=40, help="Target number of sounds to keep (default: 40)")
    p_curate.add_argument("--overgenerate", "-o", type=float, default=3.0, help="Overgenerate multiplier (default: 3x)")
    p_curate.add_argument("--no-gate", action="store_true", help="Skip quality gate")
    p_curate.add_argument("--out", help="Output directory")

    # ── gate ──
    p_gate = subparsers.add_parser(
        "gate",
        help="Run quality gates on a kit",
        description="Check a kit for weak transients, clipping, long tails, muddiness, and near-duplicates.",
    )
    p_gate.add_argument("input_dir", help="Directory of WAV files to check")
    p_gate.add_argument("--fix", action="store_true", help="Auto-remove files that fail gates")

    # ── demo ──
    p_demo = subparsers.add_parser(
        "demo",
        help="Generate a quick demo kit",
        description="Fast demo mode: generate a small kit in under 2 minutes to demonstrate cShot's capabilities.",
    )
    p_demo.add_argument("prompt", nargs="+", help="Kit description (e.g. 'dark rnb one shot kit')")
    p_demo.add_argument("--count", "-n", type=int, default=20, help="Number of sounds (default: 20)")
    p_demo.add_argument("--out", "-o", help="Output directory")

    # ── onboard ──
    p_onboard = subparsers.add_parser(
        "onboard",
        help="Interactive onboarding wizard",
        description="Guided step-by-step wizard: choose genre, input type, kit size, generate, listen, and export.",
    )
    p_onboard.add_argument("--out", "-o", help="Output directory")

    # ═══════════════════════════════════════════════════════
    #  LAB COMMANDS (research and development)
    # ═══════════════════════════════════════════════════════

    p_lab = subparsers.add_parser(
        "lab",
        help="Access advanced research and development commands",
        description="Advanced research and development commands for audio generation, analysis, and experimentation.",
    )
    lab_sub = p_lab.add_subparsers(dest="lab_command", help="Lab command")

    # ── Scan ──
    p = lab_sub.add_parser("scan", help="Scan reference folders, compute per-file features")
    p.add_argument("--output", "-o", help="Output path for reference_analysis.json")

    # ── Profiles ──
    p = lab_sub.add_parser("profiles", help="Build class profiles from analysis")
    p.add_argument("--analysis", "-a", help="Path to reference_analysis.json")
    p.add_argument("--output", "-o", help="Output path for class_profiles.json")

    # ── Oneshot ──
    p = lab_sub.add_parser("oneshot", help="Generate one-shots for a class")
    p.add_argument("class_name", help="Sound class (kick, snare, clap, etc.)")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p.add_argument("--output-dir", "-o", help="Output directory")
    p.add_argument("--out", help="Exact output path for single file")
    p.add_argument("--profiles", "-p", help="Path to class_profiles.json")

    # ── Batch ──
    p = lab_sub.add_parser("batch", help="Batch generate many samples of a class")
    p.add_argument("--class", "-c", dest="class_name", required=True, help="Sound class")
    p.add_argument("--count", "-n", type=int, default=20, help="Number of samples")
    p.add_argument("--out", "-o", default="outputs", help="Output directory")
    p.add_argument("--profiles", "-p", help="Path to class_profiles.json")

    # ── QA ──
    p = lab_sub.add_parser("qa", help="Generate QA audit with manifest and feature report")
    p.add_argument("--output-dir", "-o", help="Output directory")
    p.add_argument("--samples", "-n", type=int, default=10, help="Samples per class")
    p.add_argument("--profiles", "-p", help="Path to class_profiles.json")

    # ── Compare ──
    p = lab_sub.add_parser("compare", help="Compare generated vs reference features")
    p.add_argument("--generated", "-g", help="Generated feature_report.json path")
    p.add_argument("--reference", "-r", help="Reference analysis path")
    p.add_argument("--output", "-o", help="Output path")

    # ── All ──
    lab_sub.add_parser("all", help="Run full pipeline: scan -> profiles -> qa -> compare")

    # ── Prompt ──
    p = lab_sub.add_parser("prompt", help="Generate from natural language prompt")
    p.add_argument("prompt", nargs="+", help="Natural language description")
    p.add_argument("--count", "-n", type=int, default=1, help="Number of samples")
    p.add_argument("--out", "-o", help="Output path")
    p.add_argument("--seed", "-s", type=int, help="Fixed seed")
    p.add_argument("--name-template", "-t", help="Filename template")

    # ── Regenerate ──
    p = lab_sub.add_parser("regenerate", help="Regenerate a file from its metadata JSON")
    p.add_argument("--metadata", "-m", required=True, help="Path to metadata JSON file")
    p.add_argument("--out", "-o", help="Output path")

    # ── Prompt Refine ──
    p = lab_sub.add_parser("prompt-refine", help="Diagnose and refine prompt-based generation")
    p.add_argument("input_dir", help="Directory of generated files")
    p.add_argument("--prompt", help="Original prompt")
    p.add_argument("--out", "-o", help="Output directory")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of refined samples")

    # ── Compare Prompt ──
    p = lab_sub.add_parser("compare-prompt", help="Compare feature reports between two dirs")
    p.add_argument("dir_a", help="First directory")
    p.add_argument("dir_b", help="Second directory")

    # ── Contrast Test ──
    p = lab_sub.add_parser("contrast-test", help="Run pairwise contrast tests")
    p.add_argument("--family", "-f", help="Generator family")
    p.add_argument("--profile", "-p", help="Profile within family")
    p.add_argument("--count", "-n", type=int, default=5, help="Samples per adjective")
    p.add_argument("--out", "-o", help="Output directory")
    p.add_argument("--pairs", nargs="+", help="Adjective pairs to test")

    # ── MVP Audit ──
    p = lab_sub.add_parser("mvp-audit", help="Full 100-file QA audit")
    p.add_argument("--out", "-o", default="outputs/mvp_audit", help="Output directory")

    # ── Analyze Output ──
    p = lab_sub.add_parser("analyze-output", help="Compute features for generated output directory")
    p.add_argument("input_dir", help="Directory of generated .wav files")

    # ── Compare to References ──
    p = lab_sub.add_parser("compare-to-references", help="Compare generated outputs against reference profiles")
    p.add_argument("input_dir", help="Directory containing generated_analysis.json")
    p.add_argument("--target", "-t", help="Target reference class")

    # ── Diagnose ──
    p = lab_sub.add_parser("diagnose", help="Why generated sounds differ from references")
    p.add_argument("input_dir", help="Directory containing generated_analysis.json")
    p.add_argument("--target", "-t", help="Target reference class")

    # ── Refine ──
    p = lab_sub.add_parser("refine", help="Auto-suggest and apply parameter changes")
    p.add_argument("--target", "-t", help="Target sound class")
    p.add_argument("--from", dest="from_dir", required=True, help="Directory with diagnosis_result.json")
    p.add_argument("--out", "-o", default="outputs/refined", help="Output directory")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of refined samples")

    # ── Audit Report ──
    p = lab_sub.add_parser("audit-report", help="Generate HTML or Markdown audit report")
    p.add_argument("input_dir", help="Directory containing analysis files")
    p.add_argument("--target", "-t", help="Target class name")
    p.add_argument("--format", "-f", choices=["html", "markdown"], default="html", help="Report format")

    # ── Cluster References ──
    p = lab_sub.add_parser("cluster-references", help="Auto-cluster reference library")
    p.add_argument("--analysis", "-a", help="Path to reference_analysis.json")
    p.add_argument("--output", "-o", help="Output path")
    p.add_argument("--n-clusters", "-k", type=int, default=12, help="Number of clusters")
    p.add_argument("--pca-dims", "-p", type=int, default=10, help="PCA dimensions")

    # ── Cluster Gen ──
    p = lab_sub.add_parser("cluster-gen", help="Generate samples targeting a cluster")
    p.add_argument("--cluster-id", "-k", type=int, required=True, help="Cluster ID")
    p.add_argument("--clusters", help="Path to reference_clusters.json")
    p.add_argument("--analysis", help="Path to reference_analysis.json")
    p.add_argument("--out", "-o", default="outputs/cluster_gen", help="Output directory")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of samples")

    # ── Cluster QA ──
    p = lab_sub.add_parser("cluster-qa", help="QA generated cluster outputs")
    p.add_argument("input_dir", help="Directory of generated .wav files")
    p.add_argument("--cluster-id", "-k", type=int, help="Target cluster ID")
    p.add_argument("--clusters", help="Path to reference_clusters.json")
    p.add_argument("--output", "-o", help="Output path")

    # ── Cluster Report ──
    lab_sub.add_parser("cluster-report", help="Generate cluster visualization report")

    # ── Detect Pitch ──
    p = lab_sub.add_parser("detect-pitch", help="Detect pitch, MIDI note, and key")
    p.add_argument("input", help="Audio file or directory")
    p.add_argument("--output", "-o", help="Output JSON path")

    # ── Analyze Piano ──
    p = lab_sub.add_parser("analyze-piano", help="Analyze piano one-shot characteristics")
    p.add_argument("input", help="Audio file or directory")
    p.add_argument("--output", "-o", help="Output JSON path")

    # ── Analyze Synth ──
    p = lab_sub.add_parser("analyze-synth", help="Analyze synth stab characteristics")
    p.add_argument("input", help="Audio file or directory")
    p.add_argument("--output", "-o", help="Output JSON path")

    # ── Tonal QA ──
    p = lab_sub.add_parser("tonal-qa", help="QA tonal samples for pitch, decay, harmonic structure")
    p.add_argument("input_dir", help="Directory of generated .wav files")
    p.add_argument("--target", "-t", required=True, help="Target reference class")
    p.add_argument("--output", "-o", help="Output JSON path")

    # ── Piano Gen ──
    p = lab_sub.add_parser("piano-gen", help="Generate piano/keys one-shots")
    p.add_argument("profile", help=f"Piano profile ({', '.join(sorted(PIANO_CLASSES.keys()))}, or 'all')")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p.add_argument("--out", "-o", default="outputs/piano", help="Output directory")

    # ── Piano Audit ──
    p = lab_sub.add_parser("piano-audit", help="Piano audit report")
    p.add_argument("input_dir", help="Directory of generated piano samples")
    p.add_argument("--output", "-o", help="Output path")

    # ── Synth Gen ──
    p = lab_sub.add_parser("synth-gen", help="Generate synth one-shots")
    p.add_argument("profile", help=f"Synth profile ({', '.join(sorted(SYNTH_PROFILES.keys()))}, or 'all')")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p.add_argument("--out", "-o", default="outputs/synth", help="Output directory")
    p.add_argument("--detune", type=float, help="Detune amount in cents")
    p.add_argument("--filter-env", choices=["opening", "closing", "slightly_opening"], help="Filter envelope shape")
    p.add_argument("--attack-ms", type=float, help="Attack time in ms")
    p.add_argument("--decay-ms", type=float, help="Decay time in ms")
    p.add_argument("--sustain", type=float, help="Sustain level (0-1)")
    p.add_argument("--stereo", type=float, help="Stereo width (0-1)")
    p.add_argument("--saturation", type=float, help="Saturation drive (0-1)")
    p.add_argument("--chord", type=int, choices=[1, 2, 3, 4], help="Chord density")
    p.add_argument("--osc", help="Oscillator mix override, e.g. 'saw=0.5,square=0.3,sine=0.2'")

    # ── Synth Refine ──
    p = lab_sub.add_parser("synth-refine", help="Diagnose and refine synth samples")
    p.add_argument("input_dir", help="Directory of generated synth .wav files")
    p.add_argument("--target", "-t", default="stab", help="Target synth profile name")
    p.add_argument("--out", "-o", help="Output directory")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of refined samples")

    # ── Guitar Gen ──
    p = lab_sub.add_parser("guitar-gen", help="Generate guitar/plucked one-shots")
    p.add_argument("profile", help=f"Guitar profile ({', '.join(sorted(GUITAR_PROFILES.keys()))}, or 'all')")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p.add_argument("--out", "-o", default="outputs/guitar", help="Output directory")

    # ── Guitar QA ──
    p = lab_sub.add_parser("guitar-qa", help="QA guitar stabs")
    p.add_argument("input_dir", help="Directory of generated guitar .wav files")
    p.add_argument("--output", "-o", help="Output JSON path")

    # ── Bass Gen ──
    p = lab_sub.add_parser("bass-gen", help="Generate bass one-shots")
    p.add_argument("profile", help=f"Bass profile ({', '.join(sorted(BASS_PROFILES.keys()))}, all, or hybrid)")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p.add_argument("--out", "-o", default="outputs/bass", help="Output directory")
    p.add_argument("--clusters", help="Path to reference_clusters.json")
    p.add_argument("--drive", type=float, help="Drive/saturation (0-1)")
    p.add_argument("--growl", type=float, help="Growl/filter resonance (0-1)")
    p.add_argument("--glide", type=float, help="Pitch glide (-1 to 1)")
    p.add_argument("--sub-balance", type=float, help="Sub/body balance (0-1)")

    # ── Bass QA ──
    p = lab_sub.add_parser("bass-qa", help="QA bass sounds")
    p.add_argument("input_dir", help="Directory of generated bass .wav files")
    p.add_argument("--output", "-o", help="Output JSON path")

    # ── FX Gen ──
    p = lab_sub.add_parser("fx-gen", help="Generate FX/impact/texture one-shots")
    p.add_argument("profile", help=f"FX profile ({', '.join(sorted(FX_PROFILES.keys()))}, or 'all')")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p.add_argument("--out", "-o", default="outputs/fx", help="Output directory")

    # ── FX QA ──
    p = lab_sub.add_parser("fx-qa", help="QA FX sounds")
    p.add_argument("input_dir", help="Directory of generated FX .wav files")
    p.add_argument("--output", "-o", help="Output JSON path")

    # ── Dataset Health ──
    lab_sub.add_parser("dataset-health", help="Run dataset health checks")

    # ── Listening Report ──
    p = lab_sub.add_parser("listening-report", help="Show listening summary with rankings")
    p.add_argument("input_dir", help="Directory containing listening_notes.json")
    p.add_argument("--notes", "-n", help="Path to listening_notes.json")

    # ── Pack Census ──
    p = lab_sub.add_parser("pack-census", help="Full pack census")
    p.add_argument("--output", "-o", help="Output directory for census files")
    lab_sub.add_parser("pack-census-quick", help="Quick pack census: list packs and file counts")

    # ── Semantics ──
    lab_sub.add_parser("semantics", help="Annotate pack index with semantic tags")
    lab_sub.add_parser("semantics-report", help="Print semantic tag report")

    # ── Pack Cluster ──
    p = lab_sub.add_parser("pack-cluster", help="Cluster pack sounds by audio similarity")
    p.add_argument("--n-clusters", "-k", type=int, default=25, help="Number of clusters")
    p.add_argument("--pca-dims", "-p", type=int, default=15, help="PCA dimensions")

    # ── Pack DNA ──
    lab_sub.add_parser("pack-dna", help="Compute pack DNA fingerprints")
    lab_sub.add_parser("pack-dna-report", help="Print detailed pack DNA report")

    # ── Recreate ──
    p = lab_sub.add_parser("recreate", help="Recreate a source sample with variations")
    p.add_argument("input", help="Source .wav file")
    p.add_argument("--count", "-n", type=int, default=5, help="Number of variations")
    p.add_argument("--out", "-o", default="outputs/recreate", help="Output directory")
    p = lab_sub.add_parser("recreate-folder", help="Recreate all samples in a folder")
    p.add_argument("folder", help="Folder of audio files")
    p.add_argument("--count", "-n", type=int, default=5, help="Variations per source")
    p.add_argument("--out", "-o", default="outputs/recreate_folder", help="Output directory")
    p.add_argument("--max-files", "-m", type=int, default=20, help="Max files to process")
    p = lab_sub.add_parser("recreate-audit", help="Audit recreation outputs")
    p.add_argument("input_dir", nargs="?", default="outputs/recreate", help="Directory to audit")
    p.add_argument("--output", "-o", help="Output path")

    # ── Embed ──
    p = lab_sub.add_parser("embed", help="Compute style embedding for a sample")
    p.add_argument("input", help="Audio file to analyze")
    p = lab_sub.add_parser("embed-folder", help="Style-embed all files in a folder")
    p.add_argument("folder", help="Folder of audio files")
    lab_sub.add_parser("pack-style", help="Compute pack style centroids")

    # ── Style Transfer ──
    p = lab_sub.add_parser("style-transfer", help="Transfer style from source toward target pack")
    p.add_argument("input", help="Source audio file")
    p.add_argument("--style", "-s", dest="style_pack", required=True, help="Target pack name")
    p.add_argument("--count", "-n", type=int, default=5, help="Number of outputs")
    p.add_argument("--out", "-o", default="outputs/style_transfer", help="Output directory")
    p.add_argument("--variation-radius", "-r", type=float, default=0.3, help="Variation radius")

    # ── Variation Chain ──
    p = lab_sub.add_parser("variation-chain", help="Generate variation chain from tight to extreme")
    p.add_argument("source", help="Source audio file")
    p.add_argument("--count", "-n", type=int, default=5, help="Number of variations")
    p.add_argument("--radius-start", type=float, default=0.1, help="Starting variation radius")
    p.add_argument("--radius-end", type=float, default=1.0, help="Ending variation radius")
    p.add_argument("--out", "-o", default="outputs/variations", help="Output directory")

    # ── Pack Style Viz ──
    lab_sub.add_parser("pack-style-viz", help="Generate pack style space visualization")

    # ── Cross Blend ──
    p = lab_sub.add_parser("cross-blend", help="Cross-pack blend two samples")
    p.add_argument("sample_a", help="First sample")
    p.add_argument("sample_b", help="Second sample")
    p.add_argument("--ratios", help="Comma-separated blend ratios")
    p.add_argument("--out", "-o", default="outputs/cross_blend", help="Output directory")

    # ── Hybrid ──
    p = lab_sub.add_parser("hybrid", help="Hybrid sound design")
    p.add_argument("hybrid_type", help="Hybrid type (piano_texture, synth_guitar, 808_reese, kick_snare)")
    p.add_argument("--count", "-n", type=int, default=5, help="Number of hybrids")
    p.add_argument("--out", "-o", default="outputs/hybrid", help="Output directory")

    # ── Mutate ──
    p = lab_sub.add_parser("mutate", help="Apply spectral/temporal/harmonic mutations")
    p.add_argument("input", help="Source audio file")
    p.add_argument("--ops", default="spectral,temporal,harmonic,transient", help="Comma-separated mutation ops")
    p.add_argument("--amount", type=float, default=0.3, help="Mutation amount (0-1)")
    p.add_argument("--count", "-n", type=int, default=5, help="Number of mutations")
    p.add_argument("--out", "-o", default="outputs/mutations", help="Output directory")

    # ── Find Duplicates ──
    p = lab_sub.add_parser("find-duplicates", help="Detect near-duplicate samples")
    p.add_argument("--threshold", type=float, default=0.95, help="Similarity threshold")
    lab_sub.add_parser("quality-rank", help="Rank all pack sounds by production quality")
    lab_sub.add_parser("producer-fingerprint", help="Compute producer style fingerprints")

    # ── Imitate ──
    p = lab_sub.add_parser("imitate", help="Generate sounds that belong in a target pack")
    p.add_argument("pack_name", help="Target pack name")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of imitations")
    p.add_argument("--out", "-o", default="outputs/imitations", help="Output directory")

    # ── Interpolate ──
    p = lab_sub.add_parser("interpolate", help="Interpolate between two audio files in latent space")
    p.add_argument("input_a", help="First audio file")
    p.add_argument("input_b", help="Second audio file")
    p.add_argument("--steps", "-n", type=int, default=8, help="Number of interpolation steps")
    p.add_argument("--out", "-o", default="outputs/interpolate", help="Output directory")

    # ── Latent Mutate ──
    p = lab_sub.add_parser("latent-mutate", help="Apply controlled latent mutations")
    p.add_argument("input", help="Source audio file")
    p.add_argument("--mutate", "-m", nargs="+", choices=["brightness", "darkness", "texture", "aggression", "width", "punch"],
                   help="Mutation types")
    p.add_argument("--amount", "-a", type=float, default=0.5, help="Mutation intensity")
    p.add_argument("--count", "-n", type=int, default=5, help="Number of mutations")
    p.add_argument("--out", "-o", default="outputs/latent_mutations", help="Output directory")

    # ── Encode / Decode ──
    p = lab_sub.add_parser("encode", help="Encode audio to compact latent representation")
    p.add_argument("input", help="Audio file to encode")
    p.add_argument("--output", "-o", help="Output latent JSON path")
    p = lab_sub.add_parser("decode", help="Decode latent representation back to audio")
    p.add_argument("input", help="Latent JSON file to decode")
    p.add_argument("--output", "-o", help="Output WAV path")

    # ── RAG Generate ──
    p = lab_sub.add_parser("rag-generate", help="Retrieval-augmented generation")
    p.add_argument("input", help="Source audio file")
    p.add_argument("--n-refs", type=int, default=5, help="Number of reference neighbors")
    p.add_argument("--count", "-n", type=int, default=5, help="Number to generate")
    p.add_argument("--out", "-o", default="outputs/rag", help="Output directory")

    # ── Contrastive Pairs ──
    lab_sub.add_parser("contrastive-pairs", help="Generate contrastive pair dataset")

    # ── Diffuse ──
    p = lab_sub.add_parser("diffuse", help="DSP diffusion prototype")
    p.add_argument("input", help="Source audio file")
    p.add_argument("--steps", type=int, default=10, help="Number of diffusion steps")
    p.add_argument("--noise-schedule", choices=["linear", "cosine"], default="linear")
    p.add_argument("--out", "-o", default="outputs/diffusion", help="Output directory")

    # ── Make Pack ──
    p = lab_sub.add_parser("make-pack", help="One-click: recreate entire pack")
    p.add_argument("pack_dir", help="Source pack directory")
    p.add_argument("--count", "-n", type=int, default=50, help="Number of new sounds")
    p.add_argument("--out", "-o", default="outputs/producer_pack", help="Output directory")
    p.add_argument("--style-profile", help="Optional style profile JSON")

    # ── Verticality Audit ──
    p = lab_sub.add_parser("verticality-audit", help="Final verticality audit")
    p.add_argument("--out", "-o", default="outputs/verticality_audit", help="Output directory")

    # ─── Song Analysis ──
    p = lab_sub.add_parser("song-dna", help="Analyze songs and export song_dna.json")
    p.add_argument("inputs", nargs="+", help="Song file(s) or directory")
    p.add_argument("--output", "-o", help="Output path")
    p = lab_sub.add_parser("section-dna", help="Extract section-level DNA")
    p.add_argument("inputs", nargs="+", help="Song file(s) or directory")
    p = lab_sub.add_parser("song-palette", help="Extract sonic palette from songs")
    p.add_argument("inputs", nargs="+", help="Song file(s) or directory")
    p.add_argument("--output", "-o", help="Output path")

    # ── Genre ──
    p = lab_sub.add_parser("genre", help="Genre-aware generation")
    p.add_argument("name", help="Genre name (trap, drill, rage, ambient, etc.)")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p.add_argument("--out", "-o", default="outputs/genre", help="Output directory")

    # ── Similar ──
    p = lab_sub.add_parser("similar", help="Generate variations near a reference")
    p.add_argument("ref", help="Reference .wav file")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of variations")
    p.add_argument("--out", "-o", default="outputs/similar", help="Output directory")

    # ── Variations ──
    p = lab_sub.add_parser("variations", help="Generate variation cloud")
    p.add_argument("ref", help="Reference .wav file")
    p.add_argument("--spread", choices=["tight", "medium", "wide"], default="medium", help="Spread")
    p.add_argument("--count", "-n", type=int, default=10, help="Number of variations")
    p.add_argument("--out", "-o", default="outputs/variations", help="Output directory")

    # ── Blend ──
    p = lab_sub.add_parser("blend", help="Blend two audio samples")
    p.add_argument("sample_a", help="First .wav file")
    p.add_argument("sample_b", help="Second .wav file")
    p.add_argument("--blend", type=float, default=0.5, help="Blend ratio")
    p.add_argument("--mode", choices=["mix", "envelope"], default="mix", help="Blend mode")
    p.add_argument("--out", "-o", help="Output path")

    # ── Pack ──
    p = lab_sub.add_parser("pack", help="Generate a themed one-shot pack")
    p.add_argument("prompt", nargs="+", help="Theme description")
    p.add_argument("--count", "-n", type=int, default=100, help="Total files")
    p.add_argument("--out", "-o", required=True, help="Output pack directory")

    # ── Pack Audit ──
    p = lab_sub.add_parser("pack-audit", help="Audit a pack for quality issues")
    p.add_argument("pack_dir", help="Pack directory to audit")

    # ── Theme ──
    p = lab_sub.add_parser("theme", help="Generate a themed pack")
    p.add_argument("theme", nargs="+", help="Theme name")
    p.add_argument("--out", "-o", help="Output directory")

    # ── Top ──
    p = lab_sub.add_parser("top", help="Show top N ranked files")
    p.add_argument("input_dir", help="Directory of WAV files")
    p.add_argument("--n", type=int, default=10, help="Number of top files to show")

    # ── Prompt History ──
    lab_sub.add_parser("prompt-history", help="Show prompt rating history")

    # ── Save Preset ──
    p = lab_sub.add_parser("save-preset", help="Save a generation preset")
    p.add_argument("name", help="Preset name")
    p.add_argument("--from", "-f", dest="from_meta", required=True, help="Path to metadata JSON")

    # ── Preset ──
    p = lab_sub.add_parser("preset", help="Manage presets")
    ps = p.add_subparsers(dest="preset_action")
    ps.add_parser("list", help="List all saved presets")
    pg = ps.add_parser("generate", help="Generate from a preset")
    pg.add_argument("name", help="Preset name to generate")
    pg.add_argument("--out", "-o", help="Output directory")
    pg.add_argument("--count", "-n", type=int, default=1, help="Number of samples")

    # ── Polish ──
    p = lab_sub.add_parser("polish", help="Polish audio: trim, fade, normalize")
    p.add_argument("input", help="WAV file or directory")
    p.add_argument("--target-db", type=float, default=-1.0, help="Peak normalization target")
    p.add_argument("--trim-db", type=float, default=-60.0, help="Silence threshold")
    p.add_argument("--fade-in-ms", type=float, default=3.0, help="Fade-in length")
    p.add_argument("--fade-out-ms", type=float, default=5.0, help="Fade-out length")

    # ── Search Ref ──
    p = lab_sub.add_parser("search-ref", help="Search reference library")
    p.add_argument("query", nargs="+", help="Search query")

    # ── Refine Feedback ──
    p = lab_sub.add_parser("refine-feedback", help="Natural language refinement")
    p.add_argument("file", help="Audio file to refine")
    p.add_argument("feedback", nargs="+", help="Natural language feedback")

    # ── Kit Spec ──
    p = lab_sub.add_parser("kit-spec", help="Generate a KitSpec blueprint")
    p.add_argument("prompt", nargs="+", help="Kit description")
    p.add_argument("--out", "-o", help="Output path")
    p = lab_sub.add_parser("plan-kit", help="Show kit category plan without generating")
    p.add_argument("prompt", nargs="+", help="Kit description")
    p = lab_sub.add_parser("kit-from-pack-dna", help="Generate kit from pack DNA")
    p.add_argument("pack_name", help="Pack name (partial match)")
    p.add_argument("--count", "-n", type=int, default=50, help="Files to generate")
    p.add_argument("--out", "-o", help="Output directory")
    p = lab_sub.add_parser("kit-from-folder", help="Generate kit from a reference folder")
    p.add_argument("folder", help="Reference folder path")
    p.add_argument("--count", "-n", type=int, default=60, help="Files to generate")
    p.add_argument("--out", "-o", help="Output directory")
    p.add_argument("--strategy", "-s", choices=["dsp", "retrieval"], default="dsp",
                   help="Generation strategy: dsp (default) or retrieval (reference-transform)")
    p = lab_sub.add_parser("kit-from-description", help="Generate kit from description")
    p.add_argument("prompt", nargs="+", help="Kit description")
    p.add_argument("--count", "-n", type=int, default=60, help="Files to generate")
    p.add_argument("--out", "-o", help="Output directory")
    p = lab_sub.add_parser("mini-kit-from-song", help="Generate focused 13-file mini kit from a song")
    p.add_argument("song", help="Audio file")
    p.add_argument("--out", "-o", help="Output directory")
    p = lab_sub.add_parser("kit-audit", help="Audit a kit folder")
    p.add_argument("kit_folder", help="Kit directory")
    p = lab_sub.add_parser("kit-rank", help="Rank kit files best to worst")
    p.add_argument("kit_folder", help="Kit directory")
    p = lab_sub.add_parser("kit-repair", help="Auto-regenerate weak kit files")
    p.add_argument("kit_folder", help="Kit directory")
    p = lab_sub.add_parser("kit-variations", help="Generate kit variations")
    p.add_argument("kit_folder", help="Kit directory")
    p.add_argument("--mode", choices=["tight", "medium", "wild"], default="medium")
    p.add_argument("--out", "-o", help="Output directory")
    p = lab_sub.add_parser("kit-name", help="Apply producer-friendly naming")
    p.add_argument("kit_folder", help="Kit directory")
    p.add_argument("--mood", "-m", default="dark", help="Mood for naming")
    p.add_argument("--key", "-k", default="", help="Key signature")
    p = lab_sub.add_parser("kit-export", help="Final export: polish, manifest, README")
    p.add_argument("kit_folder", help="Kit directory")
    p = lab_sub.add_parser("kit-similarity", help="Check similarity risk")
    p.add_argument("kit_folder", help="Kit directory")
    p.add_argument("--refs", nargs="+", required=True, help="Reference directories")
    p.add_argument("--threshold", type=float, default=0.85, help="Similarity threshold")
    p = lab_sub.add_parser("more-like-kit", help="Generate sibling kit")
    p.add_argument("kit_folder", help="Source kit directory")
    p.add_argument("--count", "-n", type=int, default=60, help="Files to generate")
    p.add_argument("--out", "-o", help="Output directory")
    p = lab_sub.add_parser("merge-kits", help="Merge two kits")
    p.add_argument("kit_a", help="First kit directory")
    p.add_argument("kit_b", help="Second kit directory")
    p.add_argument("--out", "-o", help="Output directory")
    p = lab_sub.add_parser("kit-preset", help="Manage kit presets")
    kp_sub = p.add_subparsers(dest="kit_preset_action")
    kp_save = kp_sub.add_parser("save", help="Save kit as preset")
    kp_save.add_argument("--name", help="Preset name")
    kp_save.add_argument("--source", dest="source_dir", required=True, help="Source kit directory")
    kp_sub.add_parser("list", help="List all kit presets")
    kp_gen = kp_sub.add_parser("generate", help="Generate from preset")
    kp_gen.add_argument("--from", dest="from_name", required=True, help="Preset name")
    kp_gen.add_argument("--count", "-n", type=int, default=60, help="Files")
    kp_gen.add_argument("--out", "-o", help="Output directory")
    p = lab_sub.add_parser("kit-factory", help="Generate multiple kits from prompts file")
    p.add_argument("prompts_file", help="Text file with one prompt per line")
    p.add_argument("--count-per-kit", "-n", type=int, default=60, help="Files per kit")
    p.add_argument("--out", "-o", help="Output base directory")
    p = lab_sub.add_parser("export-kit", help="Final export alias")
    p.add_argument("kit_folder", help="Kit directory")
    p = lab_sub.add_parser("genre-dna", help="List or generate genre DNA profiles")
    gd_sub = p.add_subparsers(dest="genre_dna_action")
    gd_sub.add_parser("list", help="List all genre DNA profiles")
    gd_gen = gd_sub.add_parser("generate", help="Generate from genre profile")
    gd_gen.add_argument("genre", help="Genre name")
    gd_gen.add_argument("--count", "-n", type=int, default=60, help="Files to generate")
    gd_gen.add_argument("--out", "-o", help="Output directory")

    # ═══════════════════════════════════════════════════════
    #  PARSE AND DISPATCH
    # ═══════════════════════════════════════════════════════

    args = parser.parse_args()

    if args.command is None:
        parser.print_help()
        sys.exit(1)

    # ── Product commands ──
    if args.command == "make":
        cmd_make(args)
    elif args.command == "from-song":
        cmd_kit_from_song(args)
    elif args.command == "from-sample":
        if hasattr(args, "kit_size") and args.kit_size:
            args.count = args.kit_size
        cmd_kit_from_sample(args)
    elif args.command == "listen":
        cmd_listen(args)
    elif args.command == "rate":
        cmd_rate(args)
    elif args.command == "favorites":
        cmd_favorites(args)
    elif args.command == "rank":
        cmd_rank(args)
    elif args.command == "ratings":
        cmd_ratings_summary(args)
    elif args.command == "taste":
        cmd_taste_profile(args)
    elif args.command == "taste-prompt-history":
        cmd_taste_prompt_history(args)
    elif args.command == "learn-from-folder":
        cmd_learn_from_folder(args)
    elif args.command == "export-cshotpack":
        cmd_export_cshotpack(args)
    elif args.command == "import-cshotpack":
        cmd_import_cshotpack(args)
    elif args.command == "marketplace-serve":
        cmd_marketplace_serve(args)
    elif args.command == "gate":
        cmd_gate(args)
    elif args.command == "build-demo-kits":
        cmd_build_demo_kits(args)
    elif args.command == "curate-pack":
        cmd_curate_pack(args)
    elif args.command == "quality-pass":
        cmd_quality_pass(args)
    elif args.command == "export-daw":
        cmd_export_daw(args)
    elif args.command == "feedback-pack":
        cmd_feedback_pack(args)
    elif args.command == "beta-round":
        cmd_beta_round(args)
    elif args.command == "demo":
        cmd_demo(args)
    elif args.command == "onboard":
        cmd_onboard(args)
    elif args.command == "retrieve":
        cmd_retrieve(args)
    elif args.command == "like":
        cmd_like(args)

    # ── Lab commands ──
    elif args.command == "lab":
        if args.lab_command is None:
            p_lab.print_help()
            sys.exit(1)

        lc = args.lab_command

        # Map lab subcommand to handler
        lab_dispatch = {
            "scan": cmd_scan,
            "profiles": cmd_profiles,
            "oneshot": cmd_oneshot,
            "batch": cmd_batch,
            "qa": cmd_qa,
            "compare": cmd_compare,
            "all": cmd_all,
            "prompt": cmd_prompt,
            "regenerate": cmd_regenerate,
            "prompt-refine": cmd_prompt_refine,
            "compare-prompt": cmd_compare_prompt,
            "contrast-test": cmd_contrast_test,
            "mvp-audit": cmd_mvp_audit,
            "analyze-output": cmd_analyze_output,
            "compare-to-references": cmd_compare_to_references,
            "diagnose": cmd_diagnose,
            "refine": cmd_refine,
            "audit-report": cmd_audit_report,
            "cluster-references": cmd_cluster_references,
            "cluster-gen": cmd_cluster_gen,
            "cluster-qa": cmd_cluster_qa,
            "cluster-report": cmd_cluster_report,
            "detect-pitch": cmd_detect_pitch,
            "analyze-piano": cmd_analyze_piano,
            "analyze-synth": cmd_analyze_synth,
            "tonal-qa": cmd_tonal_qa,
            "piano-gen": cmd_piano_gen,
            "piano-audit": cmd_piano_audit,
            "synth-gen": cmd_synth_gen,
            "synth-refine": cmd_synth_refine,
            "guitar-gen": cmd_guitar_gen,
            "guitar-qa": cmd_guitar_qa,
            "bass-gen": cmd_bass_gen,
            "bass-qa": cmd_bass_qa,
            "fx-gen": cmd_fx_gen,
            "fx-qa": cmd_fx_qa,
            "dataset-health": cmd_dataset_health,
            "listening-report": cmd_listening_report,
            "pack-census": cmd_pack_census,
            "pack-census-quick": cmd_pack_census_quick,
            "semantics": cmd_semantics,
            "semantics-report": cmd_semantics_report,
            "pack-cluster": cmd_pack_cluster,
            "pack-dna": cmd_pack_dna,
            "pack-dna-report": cmd_pack_dna_report,
            "recreate": cmd_recreate,
            "recreate-folder": cmd_recreate_folder,
            "recreate-audit": cmd_recreate_audit,
            "embed": cmd_embed,
            "embed-folder": cmd_embed_folder,
            "pack-style": cmd_pack_style,
            "style-transfer": cmd_style_transfer,
            "variation-chain": cmd_variation_chain,
            "pack-style-viz": cmd_pack_style_viz,
            "cross-blend": cmd_cross_blend,
            "hybrid": cmd_hybrid,
            "mutate": cmd_mutate,
            "find-duplicates": cmd_find_duplicates,
            "quality-rank": cmd_quality_rank,
            "producer-fingerprint": cmd_producer_fingerprint,
            "imitate": cmd_imitate,
            "interpolate": cmd_interpolate,
            "latent-mutate": cmd_latent_mutate,
            "encode": cmd_encode,
            "decode": cmd_decode,
            "rag-generate": cmd_rag_generate,
            "contrastive-pairs": cmd_contrastive_pairs,
            "diffuse": cmd_diffuse,
            "make-pack": cmd_make_pack,
            "verticality-audit": cmd_verticality_audit,
            "song-dna": cmd_song_dna,
            "section-dna": cmd_section_dna,
            "song-palette": cmd_song_palette,
            "genre": cmd_genre,
            "similar": cmd_similar,
            "variations": cmd_variations,
            "blend": cmd_blend,
            "pack": cmd_pack,
            "pack-audit": cmd_pack_audit,
            "theme": cmd_theme,
            "top": cmd_top,
            "prompt-history": cmd_prompt_history,
            "save-preset": cmd_save_preset,
            "polish": cmd_polish,
            "search-ref": cmd_search_ref,
            "refine-feedback": cmd_refine_feedback,
            "kit-spec": cmd_kit_spec,
            "plan-kit": cmd_plan_kit,
            "kit-from-pack-dna": cmd_kit_from_pack_dna,
            "kit-from-folder": lambda a: cmd_kit_from_folder_retrieval(a) if getattr(a, 'strategy', 'dsp') == 'retrieval' else cmd_kit_from_folder(a),
            "kit-from-description": cmd_kit_from_description,
            "mini-kit-from-song": cmd_mini_kit_from_song,
            "kit-audit": cmd_kit_audit,
            "kit-rank": cmd_kit_rank,
            "kit-repair": cmd_kit_repair,
            "kit-variations": cmd_kit_variations,
            "kit-name": cmd_kit_naming,
            "kit-export": cmd_kit_export,
            "kit-similarity": cmd_kit_similarity,
            "more-like-kit": cmd_more_like_kit,
            "merge-kits": cmd_merge_kits,
            "kit-factory": cmd_kit_factory,
            "export-kit": cmd_export_kit,
        }

        if lc in lab_dispatch:
            lab_dispatch[lc](args)
        elif lc == "preset":
            if hasattr(args, "preset_action"):
                if args.preset_action == "list":
                    cmd_preset_list(args)
                elif args.preset_action == "generate":
                    cmd_preset_generate(args)
                else:
                    print("Usage: cshot lab preset list|generate <name>")
            else:
                print("Usage: cshot lab preset list|generate <name>")
        elif lc == "genre-dna":
            if hasattr(args, "genre_dna_action"):
                if args.genre_dna_action == "list":
                    cmd_list_genre_dna(args)
                elif args.genre_dna_action == "generate":
                    cmd_genre_dna(args)
                else:
                    print("Usage: cshot lab genre-dna list|generate <genre>")
            else:
                print("Usage: cshot lab genre-dna list|generate <genre>")
        elif lc == "kit-preset":
            if hasattr(args, "kit_preset_action"):
                cmd_kit_preset(args)
            else:
                print("Usage: cshot lab kit-preset save|list|generate")
        else:
            print(f"Unknown lab command: {lc}")
            p_lab.print_help()
            sys.exit(1)
    else:
        print(f"Unknown command: {args.command}")
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
