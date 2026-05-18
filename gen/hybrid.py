"""
Phase 7 — Hybridization (Weeks 27-30)
Cross-pack blending, hybrid sound design, mutation engine, evolutionary generation.
"""

import json
import math
import random
import time
from pathlib import Path
import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.io import read_audio_safe, write_wav
from gen.pack_census import SafeEncoder
from gen.features import compute_features
from gen.style_embed import compute_style_fingerprint, embed_distance, STYLE_DIMENSIONS
from gen.recreate import (
    analyze_source, find_nearest_neighbors, infer_generator,
    build_target_profile, _call_generator,
)
from gen.pack_census import compute_lufs, compute_crest_factor

CENSUS_DIR = REPO_ROOT / "gen" / "census"


# Week 27: Cross-Pack Blending

def cmd_cross_blend(args):
    source_a = Path(args.sample_a)
    source_b = Path(args.sample_b)
    out_dir = Path(getattr(args, 'out', 'outputs/cross_blend'))
    out_dir.mkdir(parents=True, exist_ok=True)

    result_a = read_audio_safe(source_a, mono=True)
    result_b = read_audio_safe(source_b, mono=True)
    if result_a is None or result_b is None:
        print("Error: cannot read samples")
        return

    samples_a, sr_a = result_a
    samples_b, sr_b = result_b

    max_len = max(len(samples_a), len(samples_b))
    if len(samples_a) < max_len:
        samples_a = np.pad(samples_a, (0, max_len - len(samples_a)))
    if len(samples_b) < max_len:
        samples_b = np.pad(samples_b, (0, max_len - len(samples_b)))

    ratios = [0.0, 0.25, 0.5, 0.75, 1.0] if not getattr(args, 'ratios', None) else [float(r) for r in args.ratios.split(',')]

    analysis_a = analyze_source(source_a)
    analysis_b = analyze_source(source_b)
    neighbors_a = find_nearest_neighbors(analysis_a, 4) if analysis_a else []
    neighbors_b = find_nearest_neighbors(analysis_b, 4) if analysis_b else []

    pack_a = neighbors_a[0]["pack"] if neighbors_a else "unknown"
    pack_b = neighbors_b[0]["pack"] if neighbors_b else "unknown"

    print(f"Cross-pack blend: {source_a.name} ({pack_a}) × {source_b.name} ({pack_b})")
    generated = []
    for r in ratios:
        blended = samples_a * (1.0 - r) + samples_b * r
        peak = np.max(np.abs(blended))
        if peak > 0:
            blended = blended / peak * 0.95
        out_name = f"blend_{source_a.stem}_x_{source_b.stem}_ratio{r:.2f}.wav"
        out_path = out_dir / out_name
        write_wav(out_path, blended)
        fp = compute_style_fingerprint(blended, sr_a)
        generated.append({
            "file": str(out_path),
            "ratio": r,
            "pack_a": pack_a,
            "pack_b": pack_b,
            "style": fp,
        })
        print(f"  ratio={r:.2f}: {out_name}")

    meta_path = out_dir / f"cross_blend_report.json"
    with open(meta_path, "w") as f:
        json.dump({"sources": [str(source_a), str(source_b)], "packs": [pack_a, pack_b],
                    "results": generated}, f, indent=2)
    print(f"Written {len(generated)} blends to {out_dir}")


# Week 28: Hybrid Sound Design

HYBRID_REGISTRY = {
    "piano_texture": ("piano", "fx"),
    "synth_guitar": ("synth", "guitar"),
    "808_reese": ("bass", "bass"),
    "kick_snare": ("drum", "drum"),
}


def cmd_hybrid(args):
    hybrid_type = args.hybrid_type
    count = getattr(args, 'count', 5)
    out_dir = Path(getattr(args, 'out', 'outputs/hybrid'))
    out_dir.mkdir(parents=True, exist_ok=True)

    if hybrid_type == "piano_texture":
        template_a = REPO_ROOT / "Packs/dark_rnb/keys/piano_gen_acoustic_001.wav"
        template_b = REPO_ROOT / "Packs/trap_god_kit/fx/trap_god_kit_impact_distorted_001.wav"
    elif hybrid_type == "synth_guitar":
        template_a = REPO_ROOT / "Packs/dark_rnb/synth/synth_gen_stab_001.wav"
        template_b = REPO_ROOT / "Packs/dark_rnb/guitar/guitar_gen_nylon_001.wav"
    elif hybrid_type == "808_reese":
        template_a = REPO_ROOT / "Packs/dark_rnb/bass/bass_gen_808_001.wav"
        template_b = REPO_ROOT / "Packs/trap_god_kit/bass/trap_god_kit_reese_punchy_001.wav"
    elif hybrid_type == "kick_snare":
        template_a = REPO_ROOT / "Packs/trap_god_kit/drums/trap_god_kit_kick_hard_001.wav"
        template_b = REPO_ROOT / "Packs/trap_god_kit/drums/trap_god_kit_snare_hard_001.wav"
    else:
        print(f"Unknown hybrid: {hybrid_type}")
        return

    analysis_a = analyze_source(template_a)
    analysis_b = analyze_source(template_b)
    neighbors_a = find_nearest_neighbors(analysis_a, 4)
    neighbors_b = find_nearest_neighbors(analysis_b, 4)
    route_a = infer_generator(analysis_a, neighbors_a)
    route_b = infer_generator(analysis_b, neighbors_b)

    print(f"Hybrid: {hybrid_type} ({route_a['generator_family']}/{route_a['generator_profile']} + {route_b['generator_family']}/{route_b['generator_profile']})")

    generated = []
    for i in range(count):
        target_a = build_target_profile(analysis_a, route_a)
        target_b = build_target_profile(analysis_b, route_b)
        blend_r = random.uniform(0.3, 0.7)
        target = {}
        for k in target_a:
            if isinstance(target_a[k], (int, float)):
                target[k] = target_a[k] * (1.0 - blend_r) + target_b.get(k, target_a[k]) * blend_r
            else:
                target[k] = target_a[k] if random.random() > 0.5 else target_b.get(k, target_a[k])

        chosen_route = route_a if random.random() > 0.5 else route_b
        result, error = _call_generator(chosen_route["generator_family"],
                                        chosen_route["generator_profile"],
                                        target, i, out_dir)
        if result is not None:
            out_name = f"hybrid_{hybrid_type}_{i+1:03d}.wav"
            out_path = out_dir / out_name
            write_wav(out_path, result)
            generated.append(str(out_path))
            print(f"  [{i+1}/{count}] {out_name}")

    print(f"Generated {len(generated)} hybrids in {out_dir}")


# Week 29: Mutation Engine

MUTATION_OPS = ["spectral", "temporal", "harmonic", "transient"]


def mutate_spectral(samples: np.ndarray, amount: float = 0.3, sr: int = SAMPLE_RATE) -> np.ndarray:
    n = len(samples)
    spec = np.fft.rfft(samples)
    freqs = np.fft.rfftfreq(n, 1/sr)
    for i in range(len(spec)):
        stretch = 1.0 + (random.random() - 0.5) * amount * 0.5
        spec[i] *= stretch
        phase_jitter = random.gauss(0, amount * 0.3)
        spec[i] *= math.cos(phase_jitter) + 1j * math.sin(phase_jitter)
    out = np.fft.irfft(spec, n=n)
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


def mutate_temporal(samples: np.ndarray, amount: float = 0.3) -> np.ndarray:
    n = len(samples)
    out = np.zeros(n)
    time_warp = 1.0 + (random.random() - 0.5) * amount * 0.4
    for i in range(n):
        src_idx = int(i * time_warp)
        if src_idx < n:
            out[i] = samples[src_idx]
        else:
            out[i] = samples[-(src_idx - n + 1)] if (src_idx - n + 1) < n else 0
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


def mutate_harmonic(samples: np.ndarray, amount: float = 0.3, sr: int = SAMPLE_RATE) -> np.ndarray:
    n = len(samples)
    spec = np.fft.rfft(samples)
    for i in range(2, len(spec), 2):
        spec[i] *= (1.0 + random.gauss(0, amount * 0.3))
    out = np.fft.irfft(spec, n=n)
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


def mutate_transient(samples: np.ndarray, amount: float = 0.3, sr: int = SAMPLE_RATE) -> np.ndarray:
    n = len(samples)
    env = np.abs(samples)
    peak_val = np.max(env)
    if peak_val < 0.01:
        return samples
    threshold = peak_val * 0.3
    out = samples.copy()
    for i in range(1, n - 1):
        if env[i] > env[i - 1] and env[i] >= env[i + 1] and env[i] >= threshold:
            spread = int(0.003 * sr)
            for j in range(max(0, i - spread), min(n, i + spread)):
                boost = 1.0 + amount * 0.5 * (1.0 - abs(j - i) / spread)
                out[j] *= boost
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


MUTATION_FUNCS = {
    "spectral": mutate_spectral,
    "temporal": mutate_temporal,
    "harmonic": mutate_harmonic,
    "transient": mutate_transient,
}


def cmd_mutate(args):
    source_path = Path(args.input)
    ops = getattr(args, 'ops', "spectral,temporal,harmonic,transient").split(',')
    amount = getattr(args, 'amount', 0.3)
    count = getattr(args, 'count', 5)
    out_dir = Path(getattr(args, 'out', 'outputs/mutations'))
    out_dir.mkdir(parents=True, exist_ok=True)

    result = read_audio_safe(source_path, mono=True)
    if result is None:
        print("Error: cannot read source")
        return
    samples, sr = result

    print(f"Mutating {source_path.name} with ops={ops}, amount={amount}")
    generated = []
    for i in range(count):
        mutated = samples.copy()
        for op in ops:
            op = op.strip()
            if op in MUTATION_FUNCS:
                mutated = MUTATION_FUNCS[op](mutated, amount * random.uniform(0.5, 1.5), sr)
        out_name = f"mutated_{source_path.stem}_{'+'.join(ops)}_{i+1:03d}.wav"
        out_path = out_dir / out_name
        write_wav(out_path, mutated)
        generated.append(str(out_path))
        print(f"  [{i+1}/{count}] {out_name}")

    print(f"Generated {len(generated)} mutations in {out_dir}")


# Week 30: Evolutionary Generation

class Evolver:
    def __init__(self, source_path: Path):
        self.source_path = source_path
        self.population = []
        self.favorites = []
        self.analysis = analyze_source(source_path) if source_path.exists() else None
        self.neighbors = find_nearest_neighbors(self.analysis, 8) if self.analysis else []
        self.route = infer_generator(self.analysis, self.neighbors) if self.analysis else None

    def generate_initial(self, n: int = 10) -> list:
        results = []
        for i in range(n):
            target = build_target_profile(self.analysis, self.route)
            samples, err = _call_generator(
                self.route["generator_family"], self.route["generator_profile"],
                target, i, Path("/tmp"))
            if samples is not None:
                results.append({"samples": samples, "score": 0.5, "generation": 0, "id": i})
        self.population = results
        return results

    def crossover(self, parent_a: dict, parent_b: dict) -> np.ndarray:
        a, b = parent_a["samples"], parent_b["samples"]
        min_len = min(len(a), len(b))
        split = random.randint(min_len // 4, min_len * 3 // 4)
        child = np.concatenate([a[:split], b[split:]])
        child = child[:min_len]
        peak = np.max(np.abs(child))
        if peak > 0:
            child = child / peak * 0.95
        return child

    def mutate_population(self, mutation_rate: float = 0.2):
        for i in range(len(self.population)):
            if random.random() < mutation_rate:
                op = random.choice(list(MUTATION_FUNCS.keys()))
                self.population[i]["samples"] = MUTATION_FUNCS[op](
                    self.population[i]["samples"], random.uniform(0.1, 0.5))
                self.population[i]["generation"] += 1

    def evolve(self, generations: int = 5, pop_size: int = 10) -> list:
        self.generate_initial(pop_size)
        all_time_best = None
        for gen in range(generations):
            self.mutate_population(0.3)
            scored = sorted(self.population, key=lambda x: x["score"], reverse=True)
            survivors = scored[:max(2, pop_size // 2)]
            children = []
            for _ in range(pop_size - len(survivors)):
                a = random.choice(survivors)
                b = random.choice(survivors)
                child_samples = self.crossover(a, b)
                children.append({"samples": child_samples, "score": 0.5, "generation": gen + 1, "id": len(self.population) + len(children)})
            if survivors:
                all_time_best = survivors[0]
            self.population = survivors + children
        return self.population
