"""
cShot Generator — Reference-informed one-shot synthesis + Audio Refinement Lab.

Usage:
  python3 gen.py scan                              Scan reference folders, write reference_analysis.json
  python3 gen.py profiles                          Build class profiles from analysis
  python3 gen.py oneshot <class> [--out FILE]      Generate one-shot (--out for exact path)
  python3 gen.py batch --class <c> --count N --out DIR  Batch generate N samples to directory
  python3 gen.py qa                                Generate QA audit: all classes x 10, manifest + feature report
  python3 gen.py compare                           Compare generated vs reference features
  python3 gen.py all                               Run full pipeline: scan → profiles → qa → compare

  # Audio Refinement Lab — repeatable scientific sound-design loop
  python3 gen.py analyze-output <dir>              Compute features for generated output directory
  python3 gen.py compare-to-references <dir> --t c  Compare generated outputs against reference profiles
  python3 gen.py diagnose <dir> --target c          Diagnose why generated sounds differ from references
  python3 gen.py refine --target c --from <dir> --out <dir>  Auto-refine parameters based on diagnosis
  python3 gen.py audit-report <dir> --format html   Generate HTML/Markdown audit report with tables
  python3 gen.py cluster-references                  Auto-cluster reference library using PCA + K-Means
  python3 gen.py cluster-gen --cluster-id N          Generate samples toward a reference cluster's statistics
  python3 gen.py cluster-qa <dir>                    QA: verify generated samples map to target cluster
  python3 gen.py cluster-report                      Generate cluster visualization report (class map, neighbors, confusion)
  python3 gen.py detect-pitch <file_or_dir>           Detect pitch, MIDI note, and musical key
  python3 gen.py analyze-piano <file_or_dir>          Analyze piano characteristics (attack, decay, resonance)
  python3 gen.py analyze-synth <file_or_dir>          Analyze synth characteristics (oscillator, filter, detune, stereo)
  python3 gen.py tonal-qa <dir> --target <class>      QA tonal samples preserve pitch, decay, harmonic structure
  python3 gen.py piano-gen <profile> [--out dir]      Generate piano/keys from named profile
  python3 gen.py piano-audit <dir>                    Piano audit report with metrics and scores
  python3 gen.py synth-gen <profile> [options]        Generate synths (stab/pluck/pad/chord/lead/bass)
  python3 gen.py synth-refine <dir> [--out dir]       Diagnose and refine synth samples
  python3 gen.py guitar-gen <profile> [--out dir]     Generate guitar/plucked one-shots
  python3 gen.py guitar-qa <dir>                      QA guitar stabs (pitch, transient, decay, body)
  python3 gen.py bass-gen <profile> [options]         Generate bass one-shots (808/reese/distorted/pluck/fm/hybrid)
  python3 gen.py bass-qa <dir>                        QA bass (low-end, richness, pitch, clipping)
  python3 gen.py fx-gen <profile> [--out dir]         Generate FX/impact/texture one-shots
  python3 gen.py fx-qa <dir>                          QA FX (energy curve, size, spectral motion)
  python3 gen.py prompt <desc> [--count N]            Generate from natural language prompt
  python3 gen.py prompt-refine <dir>                  Diagnose + refine prompt-based generation
  python3 gen.py mvp-audit                            Final MVP audit: 100 files across 20 categories
  python3 gen.py dataset-health                      Run dataset health checks on reference library
  python3 gen.py listen <dir>                        Interactive listening session (rate files, add notes)
  python3 gen.py listening-report <dir>              Show listening summary with rankings

Examples:
  python3 gen.py oneshot clap --out outputs/clap.wav
  python3 gen.py batch --class clap --count 20 --out outputs/claps/
  python3 gen.py all
  python3 gen.py analyze-output outputs/claps_v3/
  python3 gen.py compare-to-references outputs/claps_v3/ --target clap
  python3 gen.py diagnose outputs/claps_v3/ --target clap
  python3 gen.py refine --target clap --from outputs/claps_v3/ --out outputs/claps_v4/
  python3 gen.py audit-report outputs/claps_v3/ --format markdown
"""

import argparse
import sys

from gen.scanning import cmd_scan, cmd_profiles
from gen.commands import cmd_oneshot, cmd_batch, cmd_qa, cmd_compare, cmd_all
from gen.refinement import (
    cmd_analyze_output,
    cmd_compare_to_references,
    cmd_diagnose,
    cmd_refine,
    cmd_audit_report,
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
from gen.pack import cmd_pack, cmd_pack_audit
from gen.similar import cmd_similar, cmd_variations
from gen.genre import cmd_genre, GENRE_PROFILES
from gen.refine_feedback import cmd_refine_feedback


def main():
    parser = argparse.ArgumentParser(description="cShot Generator — Reference-informed one-shot synthesis")
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # scan
    p_scan = subparsers.add_parser("scan", help="Scan reference folders, compute per-file features")
    p_scan.add_argument("--output", "-o", help="Output path for reference_analysis.json")

    # profiles
    p_profiles = subparsers.add_parser("profiles", help="Build class profiles from analysis")
    p_profiles.add_argument("--analysis", "-a", help="Path to reference_analysis.json")
    p_profiles.add_argument("--output", "-o", help="Output path for class_profiles.json")

    # oneshot
    p_oneshot = subparsers.add_parser("oneshot", help="Generate one-shots for a class")
    p_oneshot.add_argument("class_name", help="Sound class (kick, snare, clap, closed_hat, open_hat, 808, bass_stab, impact_fx, synth_stab, guitar_stab, or 'all')")
    p_oneshot.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p_oneshot.add_argument("--output-dir", "-o", help="Output directory (default: generated_audit)")
    p_oneshot.add_argument("--out", help="Exact output path for single file (implies count=1)")
    p_oneshot.add_argument("--profiles", "-p", help="Path to class_profiles.json")

    # batch
    p_batch = subparsers.add_parser("batch", help="Batch generate many samples of a class")
    p_batch.add_argument("--class", "-c", dest="class_name", required=True, help="Sound class to generate")
    p_batch.add_argument("--count", "-n", type=int, default=20, help="Number of samples to generate")
    p_batch.add_argument("--out", "-o", default="outputs", help="Output directory (default: outputs/)")
    p_batch.add_argument("--profiles", "-p", help="Path to class_profiles.json")

    # qa
    p_qa = subparsers.add_parser("qa", help="Generate QA audit with manifest and feature report")
    p_qa.add_argument("--output-dir", "-o", help="Output directory (default: generated_audit)")
    p_qa.add_argument("--samples", "-n", type=int, default=10, help="Samples per class")
    p_qa.add_argument("--profiles", "-p", help="Path to class_profiles.json")

    # compare
    p_compare = subparsers.add_parser("compare", help="Compare generated vs reference features")
    p_compare.add_argument("--generated", "-g", help="Generated feature_report.json path")
    p_compare.add_argument("--reference", "-r", help="Reference analysis path")
    p_compare.add_argument("--output", "-o", help="Output path for nearest_reference_matches.json")

    # all
    p_all = subparsers.add_parser("all", help="Run full pipeline: scan → profiles → qa → compare")

    # ── Audio Refinement Lab ──

    # analyze-output
    p_ao = subparsers.add_parser("analyze-output", help="Compute features for generated output directory")
    p_ao.add_argument("input_dir", help="Directory of generated .wav files")

    # compare-to-references
    p_cr = subparsers.add_parser("compare-to-references", help="Compare generated outputs against reference class profiles")
    p_cr.add_argument("input_dir", help="Directory containing generated_analysis.json")
    p_cr.add_argument("--target", "-t", help="Target reference class (e.g. clap, kick, closed_hat)")

    # diagnose
    p_diag = subparsers.add_parser("diagnose", help="Why generated sounds differ from target references")
    p_diag.add_argument("input_dir", help="Directory containing generated_analysis.json")
    p_diag.add_argument("--target", "-t", help="Target reference class (auto-detected from comparison if omitted)")

    # refine
    p_ref = subparsers.add_parser("refine", help="Auto-suggest and apply parameter/DSP changes based on diagnosis")
    p_ref.add_argument("--target", "-t", help="Target sound class")
    p_ref.add_argument("--from", dest="from_dir", required=True, help="Directory with diagnosis_result.json")
    p_ref.add_argument("--out", "-o", default="outputs/refined", help="Output directory for refined samples")
    p_ref.add_argument("--count", "-n", type=int, default=10, help="Number of refined samples")

    # audit-report
    p_ar = subparsers.add_parser("audit-report", help="Generate HTML or Markdown audit report")
    p_ar.add_argument("input_dir", help="Directory containing generated_analysis.json / comparison / diagnosis")
    p_ar.add_argument("--target", "-t", help="Target class name")
    p_ar.add_argument("--format", "-f", choices=["html", "markdown"], default="html", help="Report format (default: html)")

    #     ── Listening Workflow ──
    p_listen = subparsers.add_parser("listen", help="Interactive listening session: rate and take notes on generated files")
    p_listen.add_argument("input_dir", help="Directory of .wav files to rate")
    p_listen.add_argument("--notes", "-n", help="Path to listening_notes.json (default: <dir>/listening_notes.json)")

    p_lr = subparsers.add_parser("listening-report", help="Show summary of listening notes with rankings")
    p_lr.add_argument("input_dir", help="Directory containing listening_notes.json")
    p_lr.add_argument("--notes", "-n", help="Path to listening_notes.json (default: <dir>/listening_notes.json)")

    #     ──     Reference Clustering ──
    p_clust = subparsers.add_parser("cluster-references", help="Auto-cluster reference files using PCA + K-Means")
    p_clust.add_argument("--analysis", "-a", help="Path to reference_analysis.json")
    p_clust.add_argument("--output", "-o", help="Output path for cluster assignments JSON")
    p_clust.add_argument("--n-clusters", "-k", type=int, default=12, help="Number of clusters for K-Means")
    p_clust.add_argument("--pca-dims", "-p", type=int, default=10, help="PCA dimensions before clustering")

    p_cg = subparsers.add_parser("cluster-gen", help="Generate samples targeting a reference cluster's statistics")
    p_cg.add_argument("--cluster-id", "-k", type=int, required=True, help="Cluster ID to target")
    p_cg.add_argument("--clusters", help="Path to reference_clusters.json")
    p_cg.add_argument("--analysis", help="Path to reference_analysis.json")
    p_cg.add_argument("--out", "-o", default="outputs/cluster_gen", help="Output directory")
    p_cg.add_argument("--count", "-n", type=int, default=10, help="Number of samples")

    p_cqa = subparsers.add_parser("cluster-qa", help="QA: verify generated outputs map to target cluster, not rivals")
    p_cqa.add_argument("input_dir", help="Directory of generated .wav files (from cluster-gen)")
    p_cqa.add_argument("--cluster-id", "-k", type=int, help="Target cluster ID (auto-detected from log if omitted)")
    p_cqa.add_argument("--clusters", help="Path to reference_clusters.json")
    p_cqa.add_argument("--output", "-o", help="Output path for QA results JSON")

    # ── Pitch Detection ──
    p_dp = subparsers.add_parser("detect-pitch", help="Detect pitch, MIDI note, and musical key for audio files")
    p_dp.add_argument("input", help="Audio file or directory of .wav files")
    p_dp.add_argument("--output", "-o", help="Output JSON path for results")

    # ── Piano Analysis ──
    p_pa = subparsers.add_parser("analyze-piano", help="Analyze piano one-shot characteristics (attack, decay, brightness, resonance)")
    p_pa.add_argument("input", help="Audio file or directory of .wav files")
    p_pa.add_argument("--output", "-o", help="Output JSON path for results")

    # ── Synth Analysis ──
    p_sa = subparsers.add_parser("analyze-synth", help="Analyze synth stab characteristics (oscillator, filter, detune, stereo)")
    p_sa.add_argument("input", help="Audio file or directory of .wav files")
    p_sa.add_argument("--output", "-o", help="Output JSON path for results")

    # ── Tonal QA ──
    p_tqa = subparsers.add_parser("tonal-qa", help="QA: verify generated tonal samples preserve pitch, decay, harmonic structure")
    p_tqa.add_argument("input_dir", help="Directory of generated .wav files")
    p_tqa.add_argument("--target", "-t", required=True, help="Target reference class (e.g., synth_stab, clap)")
    p_tqa.add_argument("--output", "-o", help="Output JSON path for results")

    # ── Piano Generation ──
    p_pg = subparsers.add_parser("piano-gen", help="Generate piano / keys one-shots from named profiles")
    p_pg.add_argument("profile", help=f"Piano profile ({', '.join(sorted(PIANO_CLASSES.keys()))}, or 'all')")
    p_pg.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p_pg.add_argument("--out", "-o", default="outputs/piano", help="Output directory")

    p_pa = subparsers.add_parser("piano-audit", help="Piano audit report: evaluate generated piano samples")
    p_pa.add_argument("input_dir", help="Directory of generated piano samples")
    p_pa.add_argument("--output", "-o", help="Output JSON path for report")

    # ── Synth Generation ──
    os = ["saw", "square", "sine", "noise"]
    p_sg = subparsers.add_parser("synth-gen", help="Generate synth one-shots (stab/pluck/pad/chord/lead/bass)")
    p_sg.add_argument("profile", help=f"Synth profile ({', '.join(sorted(SYNTH_PROFILES.keys()))}, or 'all')")
    p_sg.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p_sg.add_argument("--out", "-o", default="outputs/synth", help="Output directory")
    p_sg.add_argument("--detune", type=float, help="Detune amount in cents (override)")
    p_sg.add_argument("--filter-env", choices=["opening", "closing", "slightly_opening"], help="Filter envelope shape")
    p_sg.add_argument("--attack-ms", type=float, help="Attack time in ms")
    p_sg.add_argument("--decay-ms", type=float, help="Decay time in ms")
    p_sg.add_argument("--sustain", type=float, help="Sustain level (0-1)")
    p_sg.add_argument("--stereo", type=float, help="Stereo width (0-1)")
    p_sg.add_argument("--saturation", type=float, help="Saturation drive (0-1)")
    p_sg.add_argument("--chord", type=int, choices=[1, 2, 3, 4], help="Chord density (1=note, 2=power, 3=triad, 4=7th)")
    p_sg.add_argument("--osc", help=f"Oscillator mix override, e.g. 'saw=0.5,square=0.3,sine=0.2'")

    p_sr = subparsers.add_parser("synth-refine", help="Diagnose and refine synth samples")
    p_sr.add_argument("input_dir", help="Directory of generated synth .wav files")
    p_sr.add_argument("--target", "-t", default="stab", help="Target synth profile name")
    p_sr.add_argument("--out", "-o", help="Output directory for refined samples")
    p_sr.add_argument("--count", "-n", type=int, default=10, help="Number of refined samples")

    # ── Guitar Generation ──
    p_gg = subparsers.add_parser("guitar-gen", help="Generate guitar/plucked one-shots (nylon/muted/bright/dark/processed/reversed/chopped)")
    p_gg.add_argument("profile", help=f"Guitar profile ({', '.join(sorted(GUITAR_PROFILES.keys()))}, or 'all')")
    p_gg.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p_gg.add_argument("--out", "-o", default="outputs/guitar", help="Output directory")

    p_gq = subparsers.add_parser("guitar-qa", help="QA: verify generated guitar stabs (pitch, transient, decay, body)")
    p_gq.add_argument("input_dir", help="Directory of generated guitar .wav files")
    p_gq.add_argument("--output", "-o", help="Output JSON path for results")

    # ── Bass Generation ──
    p_bg = subparsers.add_parser("bass-gen", help="Generate bass one-shots (808/reese/distorted/pluck/fm/hybrid)")
    p_bg.add_argument("profile", help=f"Bass profile ({', '.join(sorted(BASS_PROFILES.keys()))}, all, or hybrid)")
    p_bg.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p_bg.add_argument("--out", "-o", default="outputs/bass", help="Output directory")
    p_bg.add_argument("--clusters", help="Path to reference_clusters.json (for hybrid mode)")
    p_bg.add_argument("--drive", type=float, help="Drive/saturation (0-1)")
    p_bg.add_argument("--growl", type=float, help="Growl/filter resonance (0-1)")
    p_bg.add_argument("--glide", type=float, help="Pitch glide (-1 to 1)")
    p_bg.add_argument("--sub-balance", type=float, help="Sub/body balance (0-1)")

    p_bq = subparsers.add_parser("bass-qa", help="QA: verify generated bass (low-end, richness, pitch, clipping)")
    p_bq.add_argument("input_dir", help="Directory of generated bass .wav files")
    p_bq.add_argument("--output", "-o", help="Output JSON path for results")

    # ── FX Generation ──
    p_fg = subparsers.add_parser("fx-gen", help="Generate FX/impact/texture one-shots")
    p_fg.add_argument("profile", help=f"FX profile ({', '.join(sorted(FX_PROFILES.keys()))}, or 'all')")
    p_fg.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    p_fg.add_argument("--out", "-o", default="outputs/fx", help="Output directory")

    p_fq = subparsers.add_parser("fx-qa", help="QA: verify generated FX (energy curve, size, spectral motion)")
    p_fq.add_argument("input_dir", help="Directory of generated FX .wav files")
    p_fq.add_argument("--output", "-o", help="Output JSON path for results")

    # ── Prompt-to-Sound ──
    p_pr = subparsers.add_parser("prompt", help="Generate from natural language prompt (e.g. 'dark soft piano stab')")
    p_pr.add_argument("prompt", nargs="+", help="Natural language description")
    p_pr.add_argument("--count", "-n", type=int, default=1, help="Number of samples")
    p_pr.add_argument("--out", "-o", help="Output path (file.wav for single, directory for multiple)")
    p_pr.add_argument("--seed", "-s", type=int, help="Fixed seed for deterministic generation")
    p_pr.add_argument("--name-template", "-t",
                      help="Filename template. Variables: {prompt}, {family}, {profile}, {seed}, {n}, {adj}. "
                           "Default: prompt_{prompt}_{n:03d}.wav")

    # ── Regenerate from metadata ──
    p_reg = subparsers.add_parser("regenerate", help="Regenerate a file from its metadata JSON sidecar")
    p_reg.add_argument("--metadata", "-m", required=True, help="Path to metadata JSON file")
    p_reg.add_argument("--out", "-o", help="Output path (default uses timestamp)")


    p_prr = subparsers.add_parser("prompt-refine", help="Diagnose and refine a prompt-based generation")
    p_prr.add_argument("input_dir", help="Directory of generated files")
    p_prr.add_argument("--prompt", help="Original prompt (auto-detected from dir name)")
    p_prr.add_argument("--out", "-o", help="Output directory for refined files")
    p_prr.add_argument("--count", "-n", type=int, default=10, help="Number of refined samples")

    # ── Compare prompt outputs ──
    p_cp = subparsers.add_parser("compare-prompt", help="Compare feature reports between two generated directories")
    p_cp.add_argument("dir_a", help="First directory of .wav files")
    p_cp.add_argument("dir_b", help="Second directory of .wav files")

    # ── Contrast test ──
    p_ct = subparsers.add_parser("contrast-test", help="Run pairwise contrast tests to verify adjectives")
    p_ct.add_argument("--family", "-f", help="Generator family (synth-gen, piano-gen, bass-gen)")
    p_ct.add_argument("--profile", "-p", help="Profile within family (pluck, stab, acoustic)")
    p_ct.add_argument("--count", "-n", type=int, default=5, help="Samples per adjective")
    p_ct.add_argument("--out", "-o", help="Output directory")
    p_ct.add_argument("--pairs", nargs="+", help="Adjective pairs to test, e.g. bright dark soft punchy")

    # ── MVP Audit ──
    p_mvp = subparsers.add_parser("mvp-audit", help="Final MVP audit: generate 100 files across 20 categories with contrast tests")
    p_mvp.add_argument("--out", "-o", default="outputs/mvp_audit", help="Output directory")

    # ── Rating System ──
    p_rate = subparsers.add_parser("rate", help="Rate a generated file")
    p_rate.add_argument("file", help="Path to the .wav file to rate")
    p_rate.add_argument("--rating", "-r", choices=["good", "bad", "favorite", "trash"], required=True, help="Rating")
    p_rate.add_argument("--notes", "-n", help="Optional notes about this file")

    p_ratings = subparsers.add_parser("ratings", help="Show rating summary or manage ratings")
    p_ratings.add_argument("action", nargs="?", choices=["summary"], default="summary", help="Rating action")

    p_fav = subparsers.add_parser("favorites", help="List all favorited files")

    # ── Presets ──
    p_save = subparsers.add_parser("save-preset", help="Save a generation preset from metadata")
    p_save.add_argument("name", help="Preset name")
    p_save.add_argument("--from", "-f", dest="from_meta", required=True, help="Path to metadata JSON")

    p_plist = subparsers.add_parser("preset", help="Manage presets")
    p_plist_sub = p_plist.add_subparsers(dest="preset_action", help="Preset action")
    p_plist_list = p_plist_sub.add_parser("list", help="List all saved presets")
    p_plist_gen = p_plist_sub.add_parser("generate", help="Generate from a preset")
    p_plist_gen.add_argument("name", help="Preset name to generate")
    p_plist_gen.add_argument("--out", "-o", help="Output directory")
    p_plist_gen.add_argument("--count", "-n", type=int, default=1, help="Number of samples")

    # ── Polish / Export Quality ──
    p_polish = subparsers.add_parser("polish", help="Polish audio: trim, fade, normalize, validate")
    p_polish.add_argument("input", help="WAV file or directory of WAV files")
    p_polish.add_argument("--target-db", type=float, default=-1.0,
                          help="Peak normalization target in dB (default: -1.0, options: 0, -1, -3, -6)")
    p_polish.add_argument("--trim-db", type=float, default=-60.0, help="Silence threshold in dB (default: -60)")
    p_polish.add_argument("--fade-in-ms", type=float, default=3.0, help="Fade-in length in ms")
    p_polish.add_argument("--fade-out-ms", type=float, default=5.0, help="Fade-out length in ms")

    # ── Pack Generation ──
    p_pack = subparsers.add_parser("pack", help="Generate a themed one-shot pack")
    p_pack.add_argument("prompt", nargs="+", help="Theme description (e.g. 'dark rnb one shots')")
    p_pack.add_argument("--count", "-n", type=int, default=100, help="Total files to generate")
    p_pack.add_argument("--out", "-o", required=True, help="Output pack directory")

    # ── Pack Audit ──
    p_audit = subparsers.add_parser("pack-audit", help="Audit a pack for quality issues")
    p_audit.add_argument("pack_dir", help="Pack directory to audit")

    # ── Make Similar ──
    p_sim = subparsers.add_parser("similar", help="Generate variations similar to a reference sample")
    p_sim.add_argument("reference", help="Path to reference .wav file")
    p_sim.add_argument("--count", "-n", type=int, default=20, help="Number of variations")
    p_sim.add_argument("--out", "-o", help="Output directory")

    # ── Make Variations ──
    p_var = subparsers.add_parser("variations", help="Generate a variation cloud from a reference sample")
    p_var.add_argument("reference", help="Path to reference .wav file")
    p_var.add_argument("--count", "-n", type=int, default=30, help="Number of variations")
    p_var.add_argument("--spread", choices=["low", "medium", "high"], default="medium", help="Variation spread")
    p_var.add_argument("--out", "-o", help="Output directory")

    # ── Refine Feedback ──
    p_rf = subparsers.add_parser("refine-feedback", help="Refine a file using natural language feedback")
    p_rf.add_argument("file", help="Path to the .wav file to refine")
    p_rf.add_argument("feedback", help="Natural language feedback (e.g. 'less harsh, more warm, shorter tail')")
    p_rf.add_argument("--out", "-o", help="Output path for refined file")

    # ── Genre ──
    p_genre = subparsers.add_parser("genre", help="Generate sounds for a specific genre")
    p_genre.add_argument("genre", help=f"Genre name ({', '.join(sorted(GENRE_PROFILES.keys()))})")
    p_genre.add_argument("--count", "-n", type=int, default=20, help="Number of samples")
    p_genre.add_argument("--out", "-o", help="Output directory")

    args = parser.parse_args()

    if args.command == "scan":
        cmd_scan(args)
    elif args.command == "profiles":
        cmd_profiles(args)
    elif args.command == "oneshot":
        cmd_oneshot(args)
    elif args.command == "batch":
        cmd_batch(args)
    elif args.command == "qa":
        cmd_qa(args)
    elif args.command == "compare":
        cmd_compare(args)
    elif args.command == "analyze-output":
        cmd_analyze_output(args)
    elif args.command == "compare-to-references":
        cmd_compare_to_references(args)
    elif args.command == "diagnose":
        cmd_diagnose(args)
    elif args.command == "refine":
        cmd_refine(args)
    elif args.command == "audit-report":
        cmd_audit_report(args)
    elif args.command == "cluster-references":
        cmd_cluster_references(args)
    elif args.command == "cluster-gen":
        cmd_cluster_gen(args)
    elif args.command == "cluster-qa":
        cmd_cluster_qa(args)
    elif args.command == "detect-pitch":
        cmd_detect_pitch(args)
    elif args.command == "analyze-piano":
        cmd_analyze_piano(args)
    elif args.command == "analyze-synth":
        cmd_analyze_synth(args)
    elif args.command == "tonal-qa":
        cmd_tonal_qa(args)
    elif args.command == "piano-gen":
        cmd_piano_gen(args)
    elif args.command == "piano-audit":
        cmd_piano_audit(args)
    elif args.command == "synth-gen":
        cmd_synth_gen(args)
    elif args.command == "synth-refine":
        cmd_synth_refine(args)
    elif args.command == "guitar-gen":
        cmd_guitar_gen(args)
    elif args.command == "guitar-qa":
        cmd_guitar_qa(args)
    elif args.command == "bass-gen":
        cmd_bass_gen(args)
    elif args.command == "bass-qa":
        cmd_bass_qa(args)
    elif args.command == "fx-gen":
        cmd_fx_gen(args)
    elif args.command == "fx-qa":
        cmd_fx_qa(args)
    elif args.command == "prompt":
        cmd_prompt(args)
    elif args.command == "prompt-refine":
        cmd_prompt_refine(args)
    elif args.command == "compare-prompt":
        cmd_compare_prompt(args)
    elif args.command == "contrast-test":
        cmd_contrast_test(args)
    elif args.command == "mvp-audit":
        cmd_mvp_audit(args)
    elif args.command == "cluster-report":
        cmd_cluster_report(args)
    elif args.command == "listen":
        cmd_listen(args)
    elif args.command == "listening-report":
        cmd_listening_report(args)
    elif args.command == "dataset-health":
        cmd_dataset_health(args)
    elif args.command == "rate":
        cmd_rate(args)
    elif args.command == "ratings":
        cmd_ratings_summary(args)
    elif args.command == "favorites":
        cmd_favorites(args)
    elif args.command == "regenerate":
        cmd_regenerate(args)
    elif args.command == "save-preset":
        cmd_save_preset(args)
    elif args.command == "preset":
        if args.preset_action == "list":
            cmd_preset_list(args)
        elif args.preset_action == "generate":
            cmd_preset_generate(args)
        else:
            print("Usage: cshot preset list|generate <name>")
    elif args.command == "polish":
        cmd_polish(args)
    elif args.command == "pack":
        cmd_pack(args)
    elif args.command == "pack-audit":
        cmd_pack_audit(args)
    elif args.command == "similar":
        cmd_similar(args)
    elif args.command == "variations":
        cmd_variations(args)
    elif args.command == "refine-feedback":
        cmd_refine_feedback(args)
    elif args.command == "genre":
        cmd_genre(args)
    elif args.command == "all":
        cmd_all(args)
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
