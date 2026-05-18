"""
Phase 9 — Model-Assisted Audio (Weeks 35-38)
Autoencoder, Retrieval-Augmented Generation, Contrastive Learning, Diffusion protoype.
"""

import json
import math
import random
import time
from collections import defaultdict
from pathlib import Path
import numpy as np
from scipy import signal as sp_signal

from gen import REPO_ROOT, SAMPLE_RATE
from gen.io import read_audio_safe, write_wav
from gen.pack_census import SafeEncoder
from gen.features import compute_features, compute_mfccs
from gen.style_embed import (
    compute_style_fingerprint, embed_distance, STYLE_DIMENSIONS,
    CENSUS_DIR,
)
from gen.recreate import (
    analyze_source, find_nearest_neighbors, infer_generator,
    build_target_profile, _call_generator,
)

CENSUS_DIR = REPO_ROOT / "gen" / "census"


# Week 35: Latent Audio Autoencoder (DSP-based compression/decompression)
# Stores envelope shape, per-band spectral envelopes, transient profile,
# and style fingerprint for recognizable reconstruction.


def _subsample_envelope(samples: np.ndarray, n_points: int = 64) -> list[float]:
    """Extract a smooth amplitude envelope at n_points resolution."""
    env = np.abs(samples)
    window = max(1, len(env) // n_points)
    smoothed = np.array([np.max(env[i:i+window]) for i in range(0, len(env), window)])
    if len(smoothed) < n_points:
        smoothed = np.pad(smoothed, (0, n_points - len(smoothed)))
    else:
        smoothed = smoothed[:n_points]
    peak = np.max(smoothed)
    if peak > 0:
        smoothed = smoothed / peak
    return smoothed.tolist()


def _band_envelopes(samples: np.ndarray, sr: int = SAMPLE_RATE, n_points: int = 32) -> dict:
    """Extract per-band energy envelopes (sub, bass, low-mid, high-mid, presence)."""
    nyquist = sr / 2 - 10
    bands = [(20, 80), (80, 250), (250, 2000), (2000, 6000), (6000, nyquist)]
    band_names = ["sub", "bass", "low_mid", "high_mid", "presence"]
    n_fft = 2048
    hop = n_fft // 2
    n_frames = max(1, (len(samples) - n_fft) // hop)
    result = {}
    for name, (lo, hi) in zip(band_names, bands):
        energies = []
        for f in range(n_frames):
            start = f * hop
            frame = samples[start:start+n_fft]
            if len(frame) < n_fft:
                frame = np.pad(frame, (0, n_fft - len(frame)))
            spec = np.abs(np.fft.rfft(frame)) ** 2
            freqs = np.fft.rfftfreq(n_fft, 1.0/sr)
            lo_idx = max(0, np.searchsorted(freqs, lo) - 1)
            hi_idx = min(len(spec), np.searchsorted(freqs, hi) + 1)
            e = float(np.sum(spec[lo_idx:hi_idx])) if hi_idx > lo_idx else 0
            energies.append(e)
        if not energies:
            result[name] = [0.0] * n_points
            continue
        arr = np.array(energies)
        total = np.sum(arr)
        if total > 0:
            arr = arr / total
        window = max(1, len(arr) // n_points)
        sub = [float(np.mean(arr[i:i+window])) for i in range(0, len(arr), window)]
        if len(sub) < n_points:
            sub = sub + [0.0] * (n_points - len(sub))
        result[name] = sub[:n_points]
    return result


def _detect_transient_profile(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Extract transient onset times, strengths, and count."""
    frame_size = 512
    hop_size = 256
    n_frames = max(1, (len(samples) - frame_size) // hop_size)
    prev_spec = np.zeros(frame_size // 2)
    fluxes = []
    for f in range(n_frames):
        start = f * hop_size
        frame = samples[start:start+frame_size]
        if len(frame) < frame_size:
            frame = np.pad(frame, (0, frame_size - len(frame)))
        spec = np.abs(np.fft.rfft(frame))
        half = min(len(spec), len(prev_spec))
        flux = float(np.sum(np.maximum(0, spec[:half] - prev_spec[:half])))
        fluxes.append(flux)
        prev_spec = spec[:half]
    if not fluxes:
        return {"count": 0, "times_ms": [], "strengths": []}
    arr = np.array(fluxes)
    threshold = np.mean(arr) + np.std(arr) * 2.5
    times = []
    strengths = []
    in_onset = False
    for j in range(len(arr)):
        if arr[j] > threshold and not in_onset:
            times.append(float(j * hop_size / sr * 1000))
            strengths.append(min(1.0, arr[j] / max(np.max(arr), 1e-10)))
            in_onset = True
        elif arr[j] < threshold * 0.3:
            in_onset = False
    return {"count": len(times), "times_ms": times[:32], "strengths": strengths[:32]}


def encode_latent(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Encode audio into a rich latent representation with envelope/spectral/transient data."""
    feats = compute_features(samples, sr)
    mfccs = [feats.get(f"mfcc_{i+1}", 0) for i in range(13)]
    style = compute_style_fingerprint(samples, sr)
    envelope = _subsample_envelope(samples, 64)
    band_env = _band_envelopes(samples, sr, 32)
    trans = _detect_transient_profile(samples, sr)

    return {
        "n_samples": len(samples),
        "sr": sr,
        "duration_ms": feats["duration_ms"],
        "mfcc": [round(v, 6) for v in mfccs],
        "style": style,
        "pitch_hz": feats.get("pitch_hz", 0),
        "pitch_confidence": feats.get("pitch_confidence", 0),
        "hpr": feats.get("hpr", 0.5),
        "centroid": feats.get("spectral_centroid", 0),
        "bandwidth": feats.get("spectral_bandwidth", 0),
        "crest": feats.get("crest_factor", 0),
        "attack_ms": feats.get("attack_ms", 0),
        "decay_ms": feats.get("decay_length_ms", 0),
        "rms": feats.get("rms", 0),
        "envelope": [round(v, 6) for v in envelope],
        "band_envelopes": {k: [round(v, 6) for v in vals] for k, vals in band_env.items()},
        "transients": trans,
    }


def decode_latent(latent: dict) -> np.ndarray:
    """Decode latent back to audio using envelope + spectral + transient reconstruction."""
    sr = latent.get("sr", SAMPLE_RATE)
    dur_ms = latent.get("duration_ms", 500)
    num_samples = int(sr * dur_ms / 1000.0)
    num_samples = min(num_samples, int(latent.get("n_samples", num_samples)))

    pitch = latent.get("pitch_hz", 220)
    hpr = latent.get("hpr", 0.5)
    centroid = latent.get("centroid", 2000)
    bandwidth = latent.get("bandwidth", 1000)
    attack_ms = latent.get("attack_ms", 5)
    decay_ms = latent.get("decay_ms", 200)
    rms_val = latent.get("rms", 0.1)
    envelope = latent.get("envelope", [])
    band_env = latent.get("band_envelopes", {})
    trans = latent.get("transients", {"count": 0, "times_ms": [], "strengths": []})

    samples = np.zeros(num_samples)

    # ── Build amplitude envelope from sub-sampled points ──
    if envelope and len(envelope) >= 4:
        env_points = np.array(envelope)
        amp_env = np.interp(
            np.linspace(0, 1, num_samples),
            np.linspace(0, 1, len(env_points)),
            env_points,
        )
    else:
        t = np.arange(num_samples) / sr
        attack_n = max(1, int(attack_ms / 1000.0 * sr))
        decay_n = max(1, int(decay_ms / 1000.0 * sr))
        amp_env = np.ones(num_samples)
        if attack_n < num_samples:
            amp_env[:attack_n] = np.linspace(0.01, 1.0, attack_n)
        decay_end = min(attack_n + decay_n, num_samples)
        if decay_end > attack_n:
            amp_env[attack_n:decay_end] = np.linspace(1.0, 0.1, decay_end - attack_n)
        amp_env[decay_end:] = np.linspace(0.1, 0.01, num_samples - decay_end)

    # ── Reconstruct harmonic component ──
    if hpr > 0.1 and pitch > 20:
        bw_ratio = min(bandwidth / max(centroid, 1), 2.0)
        num_partials = max(2, int(3 + bw_ratio * 4))
        t_arr = np.arange(num_samples) / sr
        harmonic = np.zeros(num_samples)
        for h in range(1, num_partials + 1):
            partial_pitch = pitch * h
            if partial_pitch > sr / 2:
                break
            harmonic_amp = 0.6 / h ** (0.7 + bw_ratio * 0.3)
            harm = np.sin(2 * math.pi * partial_pitch * t_arr) * harmonic_amp
            harmonic += harm
        harmonic *= amp_env
        samples += harmonic * hpr

    # ── Reconstruct noise/percussive component ──
    noise_amt = max(0.05, 1.0 - hpr * 1.2)
    if noise_amt > 0.05:
        np.random.seed(42)
        if band_env:
            noise = np.random.randn(num_samples).astype(np.float32)
            for band_name in ["sub", "bass", "low_mid", "high_mid", "presence"]:
                be = band_env.get(band_name, [0.0] * 32)
                if max(be) < 0.01:
                    continue
                be_env = np.interp(
                    np.linspace(0, 1, num_samples),
                    np.linspace(0, 1, len(be)),
                    be,
                )
                nyquist = sr / 2 - 10
                lo, hi = {"sub": (20, 80), "bass": (80, 250),
                          "low_mid": (250, 2000), "high_mid": (2000, 6000),
                          "presence": (6000, nyquist)}[band_name]
                sos = sp_signal.butter(4, [lo, hi], 'bandpass', fs=sr, output='sos')
                band_noise = sp_signal.sosfilt(sos, noise) * be_env * noise_amt * 0.3
                samples += band_noise
        else:
            n_env = np.exp(-5.0 * t_arr) * noise_amt * 0.3
            noise = np.random.randn(num_samples).astype(np.float32) * n_env
            samples += noise

    # ── Add transients ──
    for t_ms, strength in zip(trans.get("times_ms", [])[:16], trans.get("strengths", [])[:16]):
        t_idx = int(t_ms / 1000.0 * sr)
        if t_idx >= num_samples:
            continue
        transient_len = min(int(0.05 * sr), num_samples - t_idx)
        if transient_len <= 0:
            continue
        trans_env = np.exp(-np.arange(transient_len) * 30.0 / sr) * strength * 0.4
        np.random.seed(hash(t_ms) % 2**32)
        trans_noise = np.random.randn(transient_len) * trans_env
        end = t_idx + transient_len
        samples[t_idx:end] += trans_noise

    # ── Color with centroid — filter to match target spectral shape ──
    if centroid > 100:
        sos = sp_signal.butter(2, centroid / (sr/2), 'lowpass' if centroid < 3000 else 'highpass',
                               output='sos')
        samples = sp_signal.sosfilt(sos, samples)

    # ── Apply final envelope and normalize ──
    samples = samples * amp_env
    target_rms = max(rms_val, 0.01)
    current_rms = float(np.sqrt(np.mean(samples ** 2))) + 1e-10
    samples = samples * (target_rms / current_rms)
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


# Week 36: Latent Interpolation

def interpolate_latents(latent_a: dict, latent_b: dict, t: float) -> dict:
    """Linearly interpolate between two latent representations.
    t=0 -> pure A, t=1 -> pure B.
    """
    result = {}
    for key in ["n_samples", "sr"]:
        result[key] = int(latent_a[key] * (1 - t) + latent_b[key] * t)
    for key in ["duration_ms", "pitch_hz", "pitch_confidence", "hpr",
                "centroid", "bandwidth", "crest", "attack_ms", "decay_ms", "rms"]:
        a = latent_a.get(key, 0)
        b = latent_b.get(key, 0)
        if isinstance(a, (int, float)):
            result[key] = a * (1 - t) + b * t
        else:
            result[key] = a
    for key in ["mfcc", "envelope"]:
        a_list = latent_a.get(key, [])
        b_list = latent_b.get(key, [])
        max_len = max(len(a_list), len(b_list))
        a_padded = a_list + [0.0] * (max_len - len(a_list))
        b_padded = b_list + [0.0] * (max_len - len(b_list))
        result[key] = [a * (1 - t) + b * t for a, b in zip(a_padded, b_padded)]
    band_keys_a = latent_a.get("band_envelopes", {})
    band_keys_b = latent_b.get("band_envelopes", {})
    result["band_envelopes"] = {}
    all_bands = set(band_keys_a.keys()) | set(band_keys_b.keys())
    for bk in all_bands:
        a_list = band_keys_a.get(bk, [])
        b_list = band_keys_b.get(bk, [])
        max_len = max(len(a_list), len(b_list))
        a_padded = a_list + [0.0] * (max_len - len(a_list))
        b_padded = b_list + [0.0] * (max_len - len(b_list))
        result["band_envelopes"][bk] = [a * (1 - t) + b * t for a, b in zip(a_padded, b_padded)]
    for key in ["style"]:
        a_dict = latent_a.get(key, {})
        b_dict = latent_b.get(key, {})
        result[key] = {}
        all_k = set(a_dict.keys()) | set(b_dict.keys())
        for k in all_k:
            av = a_dict.get(k, 0.5)
            bv = b_dict.get(k, 0.5)
            if isinstance(av, (int, float)):
                result[key][k] = av * (1 - t) + bv * t
            else:
                result[key][k] = av
    for key in ["transients"]:
        a_t = latent_a.get(key, {"count": 0, "times_ms": [], "strengths": []})
        b_t = latent_b.get(key, {"count": 0, "times_ms": [], "strengths": []})
        n_times = max(len(a_t.get("times_ms", [])), len(b_t.get("times_ms", [])))
        times = []
        strengths = []
        for i in range(n_times):
            at = a_t["times_ms"][i] if i < len(a_t["times_ms"]) else a_t["times_ms"][-1] if a_t["times_ms"] else 0
            bt = b_t["times_ms"][i] if i < len(b_t["times_ms"]) else b_t["times_ms"][-1] if b_t["times_ms"] else 0
            times.append(at * (1 - t) + bt * t)
            as_ = a_t["strengths"][i] if i < len(a_t["strengths"]) else 0
            bs_ = b_t["strengths"][i] if i < len(b_t["strengths"]) else 0
            strengths.append(as_ * (1 - t) + bs_ * t)
        result["transients"] = {"count": len(times), "times_ms": times, "strengths": strengths}
    return result


def cmd_interpolate(args):
    """Interpolate between two audio files in latent space."""
    path_a = Path(args.input_a)
    path_b = Path(args.input_b)
    steps = args.steps
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    result_a = read_audio_safe(path_a, mono=True)
    result_b = read_audio_safe(path_b, mono=True)
    if not result_a or not result_b:
        print("Error: could not read one or both input files")
        return

    samples_a, sr_a = result_a
    samples_b, sr_b = result_b
    sr = min(sr_a, sr_b)
    chunk_a = samples_a[:int(sr * 1.0)]
    chunk_b = samples_b[:int(sr * 1.0)]

    print(f"Latent Interpolation")
    print(f"{'='*60}")
    print(f"A: {path_a.name} ({len(chunk_a)} samples)")
    print(f"B: {path_b.name} ({len(chunk_b)} samples)")
    print(f"Steps: {steps}")
    print(f"Output: {out_dir}")
    print()

    latent_a = encode_latent(chunk_a, sr)
    latent_b = encode_latent(chunk_b, sr)

    print(f"Latent A:")
    print(f"  pitch={latent_a['pitch_hz']:.1f}Hz centroid={latent_a['centroid']:.0f}Hz hpr={latent_a['hpr']:.2f}")
    print(f"  attack={latent_a['attack_ms']:.1f}ms decay={latent_a['decay_ms']:.1f}ms")
    print(f"  transients={latent_a['transients']['count']}")
    print()
    print(f"Latent B:")
    print(f"  pitch={latent_b['pitch_hz']:.1f}Hz centroid={latent_b['centroid']:.0f}Hz hpr={latent_b['hpr']:.2f}")
    print(f"  attack={latent_b['attack_ms']:.1f}ms decay={latent_b['decay_ms']:.1f}ms")
    print(f"  transients={latent_b['transients']['count']}")
    print()

    for i in range(steps + 1):
        t = i / steps
        interp = interpolate_latents(latent_a, latent_b, t)
        decoded = decode_latent(interp)
        out_name = f"interp_{i:03d}_t{t:.2f}.wav"
        write_wav(out_dir / out_name, decoded)
        print(f"  [{i:3d}/{steps}] t={t:.2f}  pitch={interp['pitch_hz']:.0f}Hz  centroid={interp['centroid']:.0f}Hz  → {out_name}")

    print(f"\nDone — {steps + 1} frames → {out_dir}")


# Week 37: Latent Mutation

MUTATION_PARAMS = {
    "brightness": {"key": "centroid", "range": (-3000, 3000), "desc": "Spectral centroid shift (Hz)"},
    "darkness": {"key": "centroid", "range": (-3000, 3000), "desc": "Spectral centroid shift (Hz), inverted"},
    "texture": {"key": "hpr", "range": (-0.4, 0.4), "desc": "Tonal/noise balance shift"},
    "aggression": {"key": "transients.count", "range": (-5, 5), "desc": "Transient count change"},
    "width": {"key": "bandwidth", "range": (-500, 500), "desc": "Spectral bandwidth shift (Hz)"},
    "punch": {"key": "attack_ms", "range": (-10, 10), "desc": "Attack time change (ms)"},
}


def mutate_latent(latent: dict, mutations: dict) -> dict:
    """Apply controlled mutations to a latent representation.
    mutations = {"brightness": 0.5, "texture": -0.3, ...}
    Values are in [-1, 1] where 0 = no change.
    """
    result = dict(latent)
    for mut_name, amount in mutations.items():
        if mut_name not in MUTATION_PARAMS:
            continue
        param = MUTATION_PARAMS[mut_name]
        key = param["key"]
        lo, hi = param["range"]
        if key == "transients.count":
            current = result.get("transients", {}).get("count", 0)
            new_val = max(0, int(current + amount * hi))
            result["transients"]["count"] = new_val
            result["transients"]["times_ms"] = result["transients"].get("times_ms", [])[:new_val]
            result["transients"]["strengths"] = result["transients"].get("strengths", [])[:new_val]
        elif key == "attack_ms":
            current = result.get("attack_ms", 5)
            new_val = max(0.1, current + amount * hi)
            result["attack_ms"] = new_val
        elif key == "centroid":
            current = result.get("centroid", 3000)
            shift = amount * (hi if amount > 0 else lo)
            new_val = max(50, current + shift)
            result["centroid"] = new_val
            if mut_name == "darkness":
                result["centroid"] = max(50, current - shift)
        elif key == "hpr":
            current = result.get("hpr", 0.5)
            new_val = max(0.0, min(1.0, current + amount * hi))
            result["hpr"] = new_val
        elif key == "bandwidth":
            current = result.get("bandwidth", 1000)
            new_val = max(100, current + amount * hi)
            result["bandwidth"] = new_val
    return result


def cmd_latent_mutate(args):
    """Apply controlled latent mutations to an audio file."""
    source_path = Path(args.input)
    count = args.count
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    result = read_audio_safe(source_path, mono=True)
    if result is None:
        print("Error: cannot read source")
        return

    samples, sr = result
    chunk = samples[:int(sr * 1.0)]
    latent = encode_latent(chunk, sr)

    mut_types = args.mutate if args.mutate else ["brightness", "texture", "aggression", "width", "punch"]

    print(f"Latent Mutation")
    print(f"{'='*60}")
    print(f"Source: {source_path.name}")
    print(f"Mutate: {', '.join(mut_types)}")
    print(f"Amount: {args.amount}")
    print(f"Count:  {count}")
    print(f"Output: {out_dir}")
    print()
    print(f"Base Latent:")
    print(f"  centroid={latent['centroid']:.0f}Hz  hpr={latent['hpr']:.2f}  bw={latent['bandwidth']:.0f}Hz")
    print(f"  attack={latent['attack_ms']:.1f}ms  trans={latent['transients']['count']}")
    print()

    for i in range(count):
        mutations = {}
        for mt in mut_types:
            sign = 1 if i % 2 == 0 else -1
            amount = args.amount * sign * (0.5 + random.random() * 0.5)
            mutations[mt] = amount

        mutated = mutate_latent(latent, mutations)
        decoded = decode_latent(mutated)
        out_name = f"latent_mutated_{'+'.join(mut_types)}_{i+1:03d}.wav"
        write_wav(out_dir / out_name, decoded)

        changes = []
        for mt in mut_types:
            if mt == "brightness":
                changes.append(f"cent={mutated['centroid']:.0f}")
            elif mt == "texture":
                changes.append(f"hpr={mutated['hpr']:.2f}")
            elif mt == "punch":
                changes.append(f"atk={mutated['attack_ms']:.1f}")
        print(f"  [{i+1}/{count}] {out_name}  {', '.join(changes)}")

    print(f"\nDone — {count} latent mutations → {out_dir}")


def cmd_encode(args):
    source_path = Path(args.input)
    result = read_audio_safe(source_path, mono=True)
    if result is None:
        print("Error: cannot read")
        return
    samples, sr = result
    latent = encode_latent(samples, sr)
    latent["source"] = str(source_path)
    output_path = Path(getattr(args, 'output', source_path.with_suffix('.latent.json')))
    with open(output_path, "w") as f:
        json.dump(latent, f, indent=2)
    print(f"Encoded {source_path.name} → {output_path.name} ({len(json.dumps(latent))} bytes)")


def cmd_decode(args):
    latent_path = Path(args.input)
    with open(latent_path) as f:
        latent = json.load(f)
    samples = decode_latent(latent)
    output_path = Path(getattr(args, 'output', latent_path.with_suffix('.wav')))
    write_wav(output_path, samples)
    print(f"Decoded {latent_path.name} → {output_path.name} ({len(samples)} samples)")


# Week 36: Retrieval-Augmented Generation

def cmd_rag_generate(args):
    source_path = Path(args.input)
    n_refs = getattr(args, 'n_refs', 5)
    count = getattr(args, 'count', 5)
    out_dir = Path(getattr(args, 'out', 'outputs/rag'))
    out_dir.mkdir(parents=True, exist_ok=True)

    analysis = analyze_source(source_path)
    neighbors = find_nearest_neighbors(analysis, 8)
    route = infer_generator(analysis, neighbors)

    ref_embeddings = []
    for n in neighbors[:n_refs]:
        ref_path = REPO_ROOT / n["file_path"]
        ref_result = read_audio_safe(ref_path, mono=True)
        if ref_result:
            ref_fp = compute_style_fingerprint(ref_result[0], SAMPLE_RATE)
            ref_embeddings.append(ref_fp)

    avg_ref = {}
    for dim in STYLE_DIMENSIONS:
        vals = [r[dim] for r in ref_embeddings if dim in r]
        avg_ref[dim] = float(np.mean(vals)) if vals else 0.5

    print(f"RAG generation with {len(ref_embeddings)} references")
    generated = []
    for i in range(count):
        target = build_target_profile(analysis, route)
        target["style_profile"] = avg_ref
        for dim in STYLE_DIMENSIONS:
            if dim in ("saturation", "punch", "brightness", "air"):
                ref_val = avg_ref.get(dim, 0.5)
                noise = random.gauss(0, 0.05)
                if dim == "saturation":
                    target["drive"] = max(0, min(1, ref_val + noise))
                elif dim == "brightness":
                    target["brightness"] = max(0, min(1, ref_val + noise))
                elif dim == "punch":
                    target["attack_strength"] = max(0, min(1, ref_val + noise))
        samples, error = _call_generator(
            route["generator_family"], route["generator_profile"],
            target, i, out_dir,
        )
        if samples is not None:
            out_name = f"rag_{source_path.stem}_{i+1:03d}.wav"
            write_wav(out_dir / out_name, samples)
            generated.append(str(out_dir / out_name))
            print(f"  [{i+1}/{count}] {out_name}")

    print(f"RAG generated {len(generated)} files in {out_dir}")


# Week 37: Contrastive Learning Simulator

def cmd_contrastive_pairs(args):
    index_path = CENSUS_DIR / "pack_index.json"
    with open(index_path) as f:
        idx = json.load(f)
    files = list(idx.get("files", {}).items())
    pairs = {"similar": [], "different": []}
    for _ in range(min(500, len(files))):
        a_path, a_entry = random.choice(files)
        b_path, b_entry = random.choice(files)
        if "error" in a_entry or "error" in b_entry:
            continue
        a_fp = a_entry.get("style_embedding", {})
        b_fp = b_entry.get("style_embedding", {})
        if not a_fp or not b_fp:
            continue
        dist = embed_distance(a_fp, b_fp)
        same_pack = a_entry.get("pack") == b_entry.get("pack")
        same_cat = a_entry.get("category") == b_entry.get("category")
        if same_pack or same_cat:
            pairs["similar"].append({
                "a": a_path, "b": b_path, "distance": dist,
                "a_pack": a_entry.get("pack"), "b_pack": b_entry.get("pack"),
            })
        if not same_pack and not same_cat and dist > 0.3:
            pairs["different"].append({
                "a": a_path, "b": b_path, "distance": dist,
                "a_pack": a_entry.get("pack"), "b_pack": b_entry.get("pack"),
            })
    result = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "n_similar": len(pairs["similar"]),
        "n_different": len(pairs["different"]),
        "pairs": pairs,
    }
    output_path = CENSUS_DIR / "contrastive_pairs.json"
    with open(output_path, "w") as f:
        json.dump(result, f, indent=2, cls=SafeEncoder)
    print(f"Contrastive pairs: {len(pairs['similar'])} similar, {len(pairs['different'])} different")


# Week 38: Diffusion Prototype (DSP-based iterative refinement)

def cmd_diffuse(args):
    source_path = Path(args.input)
    steps = getattr(args, 'steps', 10)
    noise_schedule = getattr(args, 'noise_schedule', "linear")
    out_dir = Path(getattr(args, 'out', 'outputs/diffusion'))
    out_dir.mkdir(parents=True, exist_ok=True)

    result = read_audio_safe(source_path, mono=True)
    if result is None:
        print("Error: cannot read")
        return
    samples, sr = result
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak

    for step in range(steps):
        t = step / steps
        if noise_schedule == "linear":
            noise_level = 1.0 - t
        else:
            noise_level = math.cos(t * math.pi / 2)
        noise = np.random.randn(len(samples)).astype(np.float32) * noise_level * 0.3
        noisy = samples + noise

        sos_lp = sp_signal.butter(2, max(100, int(8000 * t)), 'lowpass',
                                   fs=sr, output='sos')
        cleaned = sp_signal.sosfilt(sos_lp, noisy)
        cleaned = cleaned * (1.0 - noise_level * 0.2)
        samples = cleaned

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    out_path = out_dir / f"diffused_{source_path.stem}_steps{steps}.wav"
    write_wav(out_path, samples)
    print(f"Diffused {source_path.name} → {out_path.name} ({steps} steps)")
