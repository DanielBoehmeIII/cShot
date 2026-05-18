"""Reference retrieval — text search, audio similarity, and song-to-kit matching.

Commands:
  cshot retrieve "dark rnb clap" --n 20
  cshot like path/to/sample.wav --n 20
  cshot kit-from-song song.wav --strategy retrieval
  cshot kit-from-folder src-tauri/Packs/Drums --strategy retrieval
"""

import json
import sqlite3
import time
from pathlib import Path
from typing import Optional

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.io import read_audio_safe
from gen.reference_db import (
    compute_all_features, read_audio_file, compute_envelope_contour,
    compute_zcr_librosa, compute_loudness, compute_lufs,
    _safe_librosa_stft, _compute_all_features_from_stft,
    estimate_key_from_mfcc_chroma, compute_semantic_tags,
    compute_pitch_key,
)

REF_DB_DIR = REPO_ROOT / "reference_db"
EMBED_PATH = REF_DB_DIR / "reference_embeddings.npy"
EMBED_INDEX_PATH = REF_DB_DIR / "reference_embeddings_index.json"
DB_PATH = REF_DB_DIR / "reference_db.sqlite"


def _load_embeddings() -> tuple[np.ndarray, list[str]]:
    if not EMBED_PATH.exists():
        raise FileNotFoundError(f"Embeddings not found at {EMBED_PATH}. Run reference scan first.")
    embeddings = np.load(str(EMBED_PATH))
    with open(EMBED_INDEX_PATH) as f:
        index_data = json.load(f)
    return embeddings, index_data["files"]


def _get_db_connection() -> sqlite3.Connection:
    if not DB_PATH.exists():
        raise FileNotFoundError(f"Database not found at {DB_PATH}. Run reference scan first.")
    conn = sqlite3.connect(str(DB_PATH))
    conn.row_factory = sqlite3.Row
    return conn


def text_to_embedding_query(query: str, dim: int = 41) -> np.ndarray:
    """Convert a text query to an approximate embedding vector for search.
    Uses semantic keyword scoring on chroma/spectral features.
    """
    q = query.lower()
    vec = np.zeros(dim, dtype=np.float32)

    low_keywords = ['kick', '808', 'bass', 'sub', 'low', 'boom', 'thump']
    mid_keywords = ['snare', 'clap', 'snap', 'perc', 'rim', 'tom', 'stab']
    high_keywords = ['hat', 'hihat', 'hi-hat', 'cymbal', 'crash', 'ride', 'shaker', 'bright', 'top']
    tonal_keywords = ['synth', 'keys', 'piano', 'guitar', 'chord', 'bell', 'melodic', 'pad']
    dark_keywords = ['dark', 'deep', 'heavy', 'warm', 'muffled', 'ambient', 'atmospheric']
    bright_keywords = ['bright', 'crisp', 'sharp', 'clean', 'airy', 'sparkle']
    percussive_keywords = ['drum', 'hit', 'transient', 'punch', 'crack', 'snap', 'click']
    texture_keywords = ['fx', 'texture', 'noise', 'riser', 'sweep', 'ambient', 'atmospheric', 'pad']

    spectral_centroid_target = 3000.0  # default mid
    if any(k in q for k in low_keywords):
        spectral_centroid_target = 400.0
    elif any(k in q for k in high_keywords):
        spectral_centroid_target = 7000.0
    if any(k in q for k in dark_keywords):
        spectral_centroid_target *= 0.5
    if any(k in q for k in bright_keywords):
        spectral_centroid_target *= 1.5

    spectral_rolloff_target = spectral_centroid_target * 1.8
    spectral_bandwidth_target = spectral_centroid_target * 0.6

    # MFCC rough targets
    if spectral_centroid_target < 1000:
        mfcc_bias = [-10, -5, -2, 0, 2, 3, 2, 1, 0, -1, -2, -1, 0]
    elif spectral_centroid_target > 5000:
        mfcc_bias = [10, 8, 5, 3, 0, -2, -3, -2, -1, 0, 1, 0, 0]
    else:
        mfcc_bias = [0] * 13
    if any(k in q for k in dark_keywords):
        mfcc_bias = [v - 3 for v in mfcc_bias]
    if any(k in q for k in bright_keywords):
        mfcc_bias = [v + 3 for v in mfcc_bias]

    for i in range(13):
        vec[i] = mfcc_bias[i]

    # Chroma: pitch class targets
    chroma_idx = {
        'c': 0, 'c#': 1, 'db': 1, 'd': 2, 'd#': 3, 'eb': 3, 'e': 4,
        'f': 5, 'f#': 6, 'gb': 6, 'g': 7, 'g#': 8, 'ab': 8, 'a': 9,
        'a#': 10, 'bb': 10, 'b': 11,
    }
    for word in q.split():
        word = word.strip(' ,.#-')
        if word in chroma_idx:
            ci = chroma_idx[word]
            for j in range(12):
                vec[13 + j] = 0.1
            vec[13 + ci] = 0.5
            break

    # Spectral features
    vec[25] = spectral_centroid_target / 20000.0  # norm centroid
    vec[26] = spectral_rolloff_target / 20000.0
    vec[27] = spectral_bandwidth_target / 10000.0
    vec[28] = 0.005 if spectral_centroid_target < 1000 else 0.5  # flatness

    # ZCR
    vec[29] = 0.05 if spectral_centroid_target < 1000 else 0.3

    # HPR
    if any(k in q for k in tonal_keywords):
        vec[30] = 0.7
    elif any(k in q for k in percussive_keywords):
        vec[30] = 0.2
    else:
        vec[30] = 0.5

    # Envelope
    attack_keywords = ['punch', 'snap', 'click', 'crack', 'kick', 'snare', 'clap', 'rim']
    long_keywords = ['pad', 'ambient', 'long', 'release', 'tail', 'reverb', 'wash', 'riser', 'loop']
    if any(k in q for k in attack_keywords):
        vec[31] = 5.0 / 200.0  # fast attack
    elif any(k in q for k in long_keywords):
        vec[31] = 80.0 / 200.0  # slow attack
    else:
        vec[31] = 0.25
    vec[32] = 0.3  # decay

    # Onset density
    if any(k in q for k in ['loop', 'rhythm', 'pattern']):
        vec[33] = 0.5
    elif any(k in q for k in ['fx', 'pad', 'riser', 'texture']):
        vec[33] = 0.05
    elif any(k in q for k in percussive_keywords):
        vec[33] = 0.3
    else:
        vec[33] = 0.2

    # Pitch
    if any(k in q for k in low_keywords):
        vec[35] = 60.0 / 1000.0  # pitch ~60Hz
    elif any(k in q for k in tonal_keywords):
        vec[35] = 440.0 / 1000.0  # pitch ~440Hz
    elif any(k in q for k in high_keywords):
        vec[35] = 800.0 / 1000.0
    else:
        vec[35] = 200.0 / 1000.0
    vec[36] = 0.5  # pitch confidence

    # Loudness
    loud_keywords = ['loud', 'impact', 'boom', 'heavy', 'punch', 'crash']
    quiet_keywords = ['quiet', 'soft', 'gentle', 'ambient', 'pad', 'texture']
    if any(k in q for k in loud_keywords):
        vec[37] = 0.3
    elif any(k in q for k in quiet_keywords):
        vec[37] = 0.05
    else:
        vec[37] = 0.15

    # Texture
    if any(k in q for k in texture_keywords):
        vec[38] = 1.0  # rms variation
        vec[39] = 0.1  # sub energy
        vec[40] = 0.3  # uniformity
    elif any(k in q for k in ['sub', '808', 'bass']):
        vec[38] = 0.3
        vec[39] = 0.5
        vec[40] = 0.7
    else:
        vec[38] = 0.5
        vec[39] = 0.2
        vec[40] = 0.5

    return vec


def cosine_similarity(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    dot = np.dot(a, b.T)
    norm_a = np.linalg.norm(a, axis=1, keepdims=True) if a.ndim > 1 else np.linalg.norm(a)
    norm_b = np.linalg.norm(b, axis=1, keepdims=True) if b.ndim > 1 else np.linalg.norm(b)
    return dot / (norm_a * norm_b.T + 1e-10)


def retrieve_by_text(query: str, n: int = 20) -> list[dict]:
    """Search reference database by text query."""
    q = query.lower()
    embeddings, files = _load_embeddings()
    query_vec = text_to_embedding_query(q, embeddings.shape[1])

    scores = cosine_similarity(query_vec.reshape(1, -1), embeddings).flatten()

    # Get semantic tag match bonus
    conn = _get_db_connection()
    c = conn.cursor()

    query_words = set(q.split())
    results = []
    for idx in np.argsort(scores)[::-1]:
        if len(results) >= n * 2:
            break
        file_path = files[idx]
        embed_score = float(scores[idx])
        c.execute("SELECT semantic_tags, category, pack, duration_ms, "
                  "spectral_centroid, spectral_rolloff, pitch_hz, key, "
                  "hpr, envelope_shape, attack_ms, onset_count, "
                  "rms_dbfs, lufs_integrated, spectral_flatness "
                  "FROM reference_sounds WHERE file_path = ?", (file_path,))
        row = c.fetchone()
        if row is None:
            continue
        tags = (row[0] or "").lower()
        category = (row[1] or "").lower()
        pack = (row[2] or "").lower()

        # Tag bonus
        tag_bonus = 0.0
        for tw in query_words:
            if tw in tags or tw in category or tw in pack:
                tag_bonus += 0.15
        # Category match bonus (e.g., "kick" matching category containing "kick")
        for tw in query_words:
            if tw in category:
                tag_bonus += 0.2
            if tw in pack:
                tag_bonus += 0.1

        combined = embed_score + tag_bonus
        results.append({
            "file_path": file_path,
            "file_name": Path(file_path).name,
            "category": row[1],
            "pack": row[2],
            "similarity": round(float(combined), 4),
            "embed_score": round(float(embed_score), 4),
            "tag_bonus": round(float(tag_bonus), 4),
            "duration_ms": row[3],
            "spectral_centroid": row[4],
            "pitch_hz": row[6],
            "key": row[7],
            "hpr": row[8],
            "envelope_shape": row[9],
        })

    conn.close()
    results.sort(key=lambda x: x["similarity"], reverse=True)
    return results[:n]


def retrieve_by_audio(audio_path: str, n: int = 20) -> list[dict]:
    """Search reference database by audio similarity."""
    abs_path = Path(audio_path).resolve()
    if not abs_path.exists():
        raise FileNotFoundError(f"Audio file not found: {audio_path}")
    samples = read_audio_file(abs_path)
    if samples is None:
        raise ValueError(f"Cannot read audio file: {audio_path}")
    if len(samples) < 100:
        raise ValueError(f"Audio file too short: {audio_path}")

    feats = compute_all_features(samples, SAMPLE_RATE, abs_path)

    # Build embedding vector (same layout as reference_db.py compute_embeddings)
    mfcc = [feats.get(f"mfcc_{i+1}", 0) for i in range(13)]
    chroma = [feats.get(f"chroma_{i}", 0) for i in range(12)]
    spectral = [feats.get("spectral_centroid", 0) / 20000.0,
                feats.get("spectral_bandwidth", 0) / 10000.0,
                feats.get("spectral_rolloff", 0) / 20000.0,
                feats.get("spectral_flatness", 0)]
    zcr = [feats.get("zcr", 0)]
    hpr = [feats.get("hpr", 0.5)]
    attack_decay = [feats.get("attack_ms", 50) / 200.0, feats.get("decay_ms", 100) / 500.0]
    onset_den = [feats.get("onset_count", 0) / 20.0, feats.get("onset_density", 0)]
    pitch_hz_conf = [feats.get("pitch_hz", 200) / 1000.0, feats.get("pitch_confidence", 0)]
    rms = [feats.get("rms", 0.1)]
    texture = [feats.get("rms_variation", 0.5), feats.get("sub_energy_ratio", 0.2), feats.get("uniformity", 0.5)]
    query_vec = np.array(mfcc + chroma + spectral + zcr + hpr + attack_decay + onset_den + pitch_hz_conf + rms + texture,
                         dtype=np.float32)

    embeddings, files = _load_embeddings()
    scores = cosine_similarity(query_vec.reshape(1, -1), embeddings).flatten()

    conn = _get_db_connection()
    c = conn.cursor()
    results = []
    for idx in np.argsort(scores)[::-1]:
        if len(results) >= n:
            break
        file_path = files[idx]
        score = float(scores[idx])
        c.execute("SELECT category, pack, duration_ms, spectral_centroid, "
                  "spectral_rolloff, pitch_hz, key, hpr, envelope_shape, "
                  "attack_ms, semantic_tags "
                  "FROM reference_sounds WHERE file_path = ?", (file_path,))
        row = c.fetchone()
        if row is None:
            continue
        results.append({
            "file_path": file_path,
            "file_name": Path(file_path).name,
            "category": row[0],
            "pack": row[1],
            "similarity": score,
            "duration_ms": row[2],
            "spectral_centroid": row[3],
            "pitch_hz": row[5],
            "key": row[6],
            "hpr": row[7],
            "envelope_shape": row[8],
        })

    conn.close()
    return results


def retrieve_by_song(song_path: str, n: int = 60) -> list[dict]:
    """Analyze a song and retrieve matching references for kit construction."""
    from gen.song import analyze_song
    from gen.features import compute_features, detect_tempo

    samples = read_audio_file(Path(song_path))
    if samples is None:
        raise ValueError(f"Cannot read song: {song_path}")

    song_feats = compute_all_features(samples, SAMPLE_RATE, Path(song_path))
    bpm, bpm_conf = 120.0, 0.0
    try:
        from gen.features import detect_tempo as dt
        bpm, bpm_conf = dt(samples, SAMPLE_RATE)
    except Exception:
        pass

    # Use ensemble retrieval: split the song into segments, get per-segment refs
    seg_n = min(len(samples) // (SAMPLE_RATE // 2), 20)
    if seg_n < 2:
        return retrieve_by_audio(song_path, n)

    all_refs = []
    for i in range(seg_n):
        start = int(i * len(samples) / seg_n)
        end = int((i + 1) * len(samples) / seg_n)
        seg = samples[start:end]
        seg_feats = compute_all_features(seg, SAMPLE_RATE, Path(song_path))
        mfcc = [seg_feats.get(f"mfcc_{i+1}", 0) for i in range(13)]
        chroma = [seg_feats.get(f"chroma_{i}", 0) for i in range(12)]
        spectral = [seg_feats.get("spectral_centroid", 0) / 20000.0,
                    seg_feats.get("spectral_bandwidth", 0) / 10000.0,
                    seg_feats.get("spectral_rolloff", 0) / 20000.0,
                    seg_feats.get("spectral_flatness", 0)]
        zcr = [seg_feats.get("zcr", 0)]
        hpr = [seg_feats.get("hpr", 0.5)]
        attack_decay = [seg_feats.get("attack_ms", 50) / 200.0, seg_feats.get("decay_ms", 100) / 500.0]
        onset_den = [seg_feats.get("onset_count", 0) / 20.0, seg_feats.get("onset_density", 0)]
        pitch_hz_conf = [seg_feats.get("pitch_hz", 200) / 1000.0, seg_feats.get("pitch_confidence", 0)]
        rms = [seg_feats.get("rms", 0.1)]
        texture = [seg_feats.get("rms_variation", 0.5), seg_feats.get("sub_energy_ratio", 0.2), seg_feats.get("uniformity", 0.5)]
        vec = np.array(mfcc + chroma + spectral + zcr + hpr + attack_decay + onset_den + pitch_hz_conf + rms + texture,
                       dtype=np.float32)

        embeddings, files = _load_embeddings()
        scores = cosine_similarity(vec.reshape(1, -1), embeddings).flatten()
        conn = _get_db_connection()
        c = conn.cursor()
        for idx in np.argsort(scores)[::-1][:max(5, n // seg_n)]:
            fp = files[idx]
            c.execute("SELECT category, pack, semantic_tags FROM reference_sounds WHERE file_path = ?", (fp,))
            row = c.fetchone()
            if row:
                all_refs.append({
                    "file_path": fp,
                    "file_name": Path(fp).name,
                    "category": row[0],
                    "pack": row[1],
                    "similarity": float(scores[idx]),
                })
        conn.close()

    all_refs.sort(key=lambda x: x["similarity"], reverse=True)
    seen = set()
    unique = []
    for r in all_refs:
        if r["file_path"] not in seen:
            seen.add(r["file_path"])
            unique.append(r)
    return unique[:n]


def retrieve_by_folder(folder_path: str, n: int = 60) -> list[dict]:
    """Analyze a reference folder and find matching sounds across all packs."""
    folder = Path(folder_path)
    if not folder.exists():
        raise FileNotFoundError(f"Folder not found: {folder_path}")

    audio_files = []
    for ext in ['.wav', '.wave', '.aif', '.aiff', '.flac', '.mp3', '.wv']:
        audio_files.extend(folder.rglob(f"*{ext}"))
        audio_files.extend(folder.rglob(f"*{ext.upper()}"))
    audio_files = [f for f in audio_files if f.is_file()]

    if not audio_files:
        raise ValueError(f"No audio files found in {folder_path}")

    # Compute aggregate profile
    profile_vec = np.zeros(41, dtype=np.float32)
    count = 0
    for fpath in audio_files[:50]:
        samples = read_audio_file(fpath)
        if samples is None or len(samples) < 100:
            continue
        feats = compute_all_features(samples, SAMPLE_RATE, fpath)
        mfcc = [feats.get(f"mfcc_{i+1}", 0) for i in range(13)]
        chroma = [feats.get(f"chroma_{i}", 0) for i in range(12)]
        spectral = [feats.get("spectral_centroid", 0) / 20000.0,
                    feats.get("spectral_bandwidth", 0) / 10000.0,
                    feats.get("spectral_rolloff", 0) / 20000.0,
                    feats.get("spectral_flatness", 0)]
        zcr = [feats.get("zcr", 0)]
        hpr = [feats.get("hpr", 0.5)]
        attack_decay = [feats.get("attack_ms", 50) / 200.0, feats.get("decay_ms", 100) / 500.0]
        onset_den = [feats.get("onset_count", 0) / 20.0, feats.get("onset_density", 0)]
        pitch_hz_conf = [feats.get("pitch_hz", 200) / 1000.0, feats.get("pitch_confidence", 0)]
        rms = [feats.get("rms", 0.1)]
        texture_v = [feats.get("rms_variation", 0.5), feats.get("sub_energy_ratio", 0.2), feats.get("uniformity", 0.5)]
        vec = np.array(mfcc + chroma + spectral + zcr + hpr + attack_decay + onset_den + pitch_hz_conf + rms + texture_v,
                       dtype=np.float32)
        profile_vec += vec
        count += 1

    if count == 0:
        raise ValueError("Could not read any audio files in folder")

    profile_vec /= count
    embeddings, files = _load_embeddings()
    scores = cosine_similarity(profile_vec.reshape(1, -1), embeddings).flatten()

    conn = _get_db_connection()
    c = conn.cursor()
    results = []
    for idx in np.argsort(scores)[::-1]:
        if len(results) >= n:
            break
        fp = files[idx]
        c.execute("SELECT category, pack, duration_ms, spectral_centroid, "
                  "pitch_hz, key, envelope_shape, semantic_tags "
                  "FROM reference_sounds WHERE file_path = ?", (fp,))
        row = c.fetchone()
        if row:
            results.append({
                "file_path": fp,
                "file_name": Path(fp).name,
                "category": row[0],
                "pack": row[1],
                "similarity": float(scores[idx]),
                "duration_ms": row[2],
                "spectral_centroid": row[3],
                "pitch_hz": row[4],
                "envelope_shape": row[6],
            })
    conn.close()
    return results


def cmd_retrieve(args):
    query = " ".join(args.query)
    n = getattr(args, 'n', 20)
    results = retrieve_by_text(query, n)
    print(f"\nSearch: '{query}' — Top {len(results)} results")
    print(f"{'='*80}")
    print(f"{'#':>3} {'File':<40} {'Category':<15} {'Pack':<20} {'Sim':>6}")
    print(f"{'-'*3} {'-'*40} {'-'*15} {'-'*20} {'-'*6}")
    for i, r in enumerate(results[:n]):
        print(f"{i+1:>3} {r['file_name']:<40} {r['category']:<15} {r['pack']:<20} {r['similarity']:.4f}")


def cmd_like(args):
    path = args.path
    n = getattr(args, 'n', 20)
    results = retrieve_by_audio(path, n)
    print(f"\nSimilar to: '{path}' — Top {len(results)} results")
    print(f"{'='*80}")
    print(f"{'#':>3} {'File':<40} {'Category':<15} {'Pack':<20} {'Sim':>6}")
    print(f"{'-'*3} {'-'*40} {'-'*15} {'-'*20} {'-'*6}")
    for i, r in enumerate(results[:n]):
        print(f"{i+1:>3} {r['file_name']:<40} {r['category']:<15} {r['pack']:<20} {r['similarity']:.4f}")


def cmd_kit_from_song_retrieval(args):
    from gen.kit_engine import generate_kit_from_refs
    path = args.song
    out = Path(getattr(args, 'out', f"outputs/reference_kit_from_song/{Path(path).stem}"))
    n = getattr(args, 'count', 60)
    strategy = getattr(args, 'strategy', 'retrieval')

    print(f"Analyzing song: {path}")
    refs = retrieve_by_song(path, n=n * 2)
    print(f"Retrieved {len(refs)} matching references")

    # Categorize refs for kit construction
    from gen.reference_transform import transform_references_to_kit
    out.mkdir(parents=True, exist_ok=True)
    result = transform_references_to_kit(refs, out, target_count=n)
    print(f"Kit generated: {out}")
    print(f"  Files: {result.get('generated', 0)}")
    print(f"  Coherence: {result.get('coherence', 0):.3f}")


def cmd_kit_from_folder_retrieval(args):
    folder = args.folder
    out = Path(getattr(args, 'out', f"outputs/reference_kit_from_folder/{Path(folder).name}"))
    n = getattr(args, 'count', 60)
    strategy = getattr(args, 'strategy', 'retrieval')

    refs = retrieve_by_folder(folder, n=n * 2)
    print(f"Retrieved {len(refs)} matching references from folder analysis")

    from gen.reference_transform import transform_references_to_kit
    out.mkdir(parents=True, exist_ok=True)
    result = transform_references_to_kit(refs, out, target_count=n)
    print(f"Kit generated: {out}")
    print(f"  Files: {result.get('generated', 0)}")
    print(f"  Coherence: {result.get('coherence', 0):.3f}")


if __name__ == "__main__":
    import argparse
    p = argparse.ArgumentParser(description="cShot Reference Retrieval")
    sp = p.add_subparsers(dest="cmd")
    r = sp.add_parser("retrieve", help="Search by text")
    r.add_argument("query", nargs="+", help="Search query")
    r.add_argument("--n", type=int, default=20, help="Number of results")
    l = sp.add_parser("like", help="Search by audio similarity")
    l.add_argument("path", help="Path to audio file")
    l.add_argument("--n", type=int, default=20, help="Number of results")
    args = p.parse_args()
    if args.cmd == "retrieve":
        cmd_retrieve(args)
    elif args.cmd == "like":
        cmd_like(args)
    else:
        p.print_help()
