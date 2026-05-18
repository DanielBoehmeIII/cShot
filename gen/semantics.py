"""
Week 2 — Folder Semantics
Infer semantic meaning from folder names and filenames.
Build semantic tags and latent "sound descriptors" for every file.
"""

import json
import re
import time
from pathlib import Path

from gen import REPO_ROOT

SOUND_CHARACTER_TAGS = {
    "dark", "punchy", "warm", "bright", "distorted", "clean",
    "hard", "soft", "airy", "metallic", "deep", "crisp", "fat",
    "thin", "raw", "processed", "smooth", "gritty", "edgy",
    "aggressive", "gentle", "cold", "dry", "wet", "heavy", "light",
    "tight", "loose", "boomy", "harsh", "shimmer", "thick",
    "hollow", "sharp", "dull", "rich", "lush", "epic", "intimate",
    "calm", "tense", "sweet", "shallow", "sad", "happy",
    "digital", "analog", "vintage", "modern", "glitch",
    "acoustic", "electronic", "layered", "reversed", "compressed",
    "saturated", "scary", "mysterious", "powerful", "subtle",
    "complex", "simple", "noisy", "tone", "round", "clicky",
    "snappy", "slap", "rubber", "wooden", "glass", "brass",
    "string", "mallet", "bell", "resonant", "muffled", "dampened",
    "ambient", "texture", "drone", "noise", "space", "room",
    "close", "wide", "stereo", "mono", "phaser", "flanger",
    "chorus", "reverb", "delay", "echo", "filter", "wah",
    "fm", "reese", "growl", "sine", "square", "saw", "noise",
}

INSTRUMENT_TAGS = {
    "kick", "snare", "clap", "hat", "hihat", "hi-hat", "hh", "oh",
    "808", "sub", "bass", "synth", "stab", "lead", "pad", "pluck",
    "chord", "piano", "keys", "keyboard", "guitar", "vocal", "vox",
    "voice", "fx", "effect", "impact", "riser", "crash", "cymbal",
    "ride", "tom", "perc", "percussion", "rim", "rimshot", "shaker",
    "foley", "loop", "noise", "texture", "ambient", "drone",
    "brass", "string", "strings", "woodwind", "flute", "horn",
    "organ", "reed", "bell", "mallet", "marimba", "vibraphone",
    "xylophone", "glockenspiel", "chime", "tambourine", "cowbell",
    "clave", "block", "stick", "clap", "snap", "click", "tap",
    "splash", "china", "swell", "sweep", "drop", "whoosh",
    "reverse", "glitch", "transition", "lift", "build", "hit",
    "sting", "boom", "thud", "thump", "smack", "crack", "pop",
    "punch", "bang", "blast", "blast", "break", "fill", "roll",
    "groove", "beat", "rhythm", "pattern",
}

PRODUCTION_STYLE_TAGS = {
    "oneshot", "one-shot", "one_shot", "loop", "processed", "raw",
    "acoustic", "electronic", "analog", "digital", "vintage", "modern",
    "distorted", "clean", "compressed", "saturated", "reversed",
    "layered", "glitch", "hybrid", "polished", "unprocessed",
    "live", "sampled", "synthetic", "organic", "hybrid",
    "dry", "wet", "close", "room", "studio", "bedroom",
    "pro", "professional", "demo", "design", "sound design",
}

EMOTION_TAGS = {
    "dark", "bright", "warm", "cold", "aggressive", "soft",
    "smooth", "harsh", "punchy", "tight", "airy", "thick",
    "thin", "heavy", "light", "deep", "rich", "hollow",
    "sharp", "dull", "hard", "edgy", "sweet", "sad", "happy",
    "tense", "calm", "epic", "intimate", "lush", "scary",
    "mysterious", "powerful", "subtle", "complex", "simple",
    "noisy", "playful", "serious", "dramatic", "gentle",
    "soothing", "anxious", "hopeful", "dark", "brooding",
}

NUMERIC_PATTERN = re.compile(r'\d+')
SEPARATOR_PATTERN = re.compile(r'[_\-\s+]+')


def tokenize_name(name: str) -> list[str]:
    """Split a filename into meaningful tokens."""
    name = re.sub(r'\.\w+$', '', name)
    name = SEPARATOR_PATTERN.sub(' ', name)
    tokens = name.lower().split()
    result = []
    for t in tokens:
        result.append(t)
        subtokens = re.findall(r'[a-z]+|[A-Z][a-z]*', t)
        if len(subtokens) > 1:
            result.extend(s.lower() for s in subtokens)
    return [t for t in result if t and not NUMERIC_PATTERN.fullmatch(t)]


def extract_semantic_tags(name: str, folder_name: str, pack_name: str) -> dict:
    """Extract semantic tags from a filename and its folder/pack context."""
    name_tokens = tokenize_name(name)
    folder_tokens = tokenize_name(folder_name)
    pack_tokens = tokenize_name(pack_name)

    all_tokens = list(set(name_tokens + folder_tokens + pack_tokens))

    sound_character = []
    instruments = []
    production_style = []
    emotion = []

    for token in all_tokens:
        if token in SOUND_CHARACTER_TAGS:
            sound_character.append(token)
        if token in INSTRUMENT_TAGS:
            instruments.append(token)
        if token in PRODUCTION_STYLE_TAGS:
            production_style.append(token)
        if token in EMOTION_TAGS and token not in sound_character:
            emotion.append(token)

    descriptors = []

    vowel_patterns = {
        r'\b(.*?)808\b': '808_style',
        r'\b(.*?)reese\b': 'reese_style',
        r'\b(.*?)stab\b': 'stab_style',
        r'\b(.*?)pluck\b': 'pluck_style',
        r'\b(.*?)lead\b': 'lead_style',
        r'\b(.*?)pad\b': 'pad_style',
    }

    for pattern, desc in vowel_patterns.items():
        m = re.search(pattern, name.lower() + ' ' + folder_name.lower())
        if m:
            descriptors.append(desc)

    if re.search(r'(?<!\w)ch(?!\w)', name.lower()):
        if 'open' not in name.lower():
            instruments.append('closed_hat')
    if re.search(r'(?<!\w)oh(?!\w)', name.lower()):
        if 'hat' not in name.lower():
            instruments.append('open_hat')

    tags = {
        "sound_character": sorted(set(sound_character)),
        "instruments": sorted(set(instruments)),
        "production_style": sorted(set(production_style)),
        "emotion": sorted(set(emotion)),
        "descriptors": sorted(set(descriptors)),
    }

    tags["tag_count"] = sum(len(v) for v in tags.values())
    tags["primary_instrument"] = tags["instruments"][0] if tags["instruments"] else None
    tags["primary_character"] = tags["sound_character"][0] if tags["sound_character"] else None

    return tags


def guess_sonic_family(tags: dict, category: str, family: str) -> str:
    """Infer a higher-level sonic family from tags + category + family."""
    inst = tags.get("instruments", [])
    char = tags.get("sound_character", [])
    desc = tags.get("descriptors", [])

    if category in ("kick", "snare", "clap", "closed_hat", "open_hat", "cymbal", "percussion"):
        return f"percussive_{category}"

    if "pad" in inst or "ambient" in char or "drone" in family.lower():
        return "ambient_texture"

    if "bass" in inst or "808" in inst or "sub" in char or "reese" in desc:
        sub_type = "reese" if "reese" in desc else "808" if "808" in inst else "sub"
        return f"bass_{sub_type}"

    if "synth" in inst or "stab" in inst or "lead" in inst:
        return f"tonal_synth"

    if "fx" in inst or "impact" in char or "riser" in inst:
        return "fx_impact"

    if "piano" in inst or "keys" in inst:
        return "tonal_piano"

    if "guitar" in inst:
        return "tonal_guitar"

    if "vocal" in inst or "vox" in inst:
        return "vocal"

    if "loop" in family.lower():
        return "loop"

    if "texture" in family.lower() or "noise" in char:
        return "texture"

    return f"other_{category}"


def cmd_semantics(args):
    """WEEK 2: Annotate pack_index.json with semantic tags from folder/filename analysis."""
    census_dir = REPO_ROOT / "gen" / "census"
    index_path = census_dir / "pack_index.json"

    if not index_path.exists():
        print("Error: run 'pack-census' first to generate pack_index.json")
        return

    with open(index_path) as f:
        census = json.load(f)

    files = census.get("files", {})
    print(f"Annotating {len(files)} files with semantic tags...")

    annotated = 0
    for rel_path, entry in files.items():
        filename = entry.get("filename", "")
        family = entry.get("family", "")
        pack = entry.get("pack", "")
        category = entry.get("category", "other")

        name = Path(rel_path).stem
        folder_name = Path(rel_path).parent.name if Path(rel_path).parent.name != "." else family

        tags = extract_semantic_tags(name, folder_name, pack)
        sonic_family = guess_sonic_family(tags, category, family)

        entry["semantic_tags"] = tags
        entry["sonic_family"] = sonic_family
        annotated += 1

    output = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "source": str(index_path),
        "total_annotated": annotated,
        "files": files,
    }

    output_path = census_dir / "pack_semantics.json"
    from gen.pack_census import SafeEncoder
    output_path.write_text(json.dumps(output, indent=2, cls=SafeEncoder))
    print(f"Wrote {output_path} ({annotated} files annotated)")

    tag_stats = {}
    for entry in files.values():
        tags = entry.get("semantic_tags", {})
        for tag_type, tag_list in tags.items():
            if isinstance(tag_list, list):
                for tag in tag_list:
                    key = f"{tag_type}.{tag}"
                    tag_stats[key] = tag_stats.get(key, 0) + 1

    stats_path = census_dir / "semantic_tag_stats.json"
    with open(stats_path, "w") as f:
        json.dump({
            "generated_at": output["generated_at"],
            "total_tags": sum(tag_stats.values()),
            "unique_tags": len(tag_stats),
            "tag_counts": dict(sorted(tag_stats.items(), key=lambda x: -x[1])),
        }, f, indent=2)
    print(f"Wrote {stats_path} ({len(tag_stats)} unique tags)")

    sonic_families = {}
    for entry in files.values():
        sf = entry.get("sonic_family", "unknown")
        sonic_families[sf] = sonic_families.get(sf, 0) + 1

    print(f"\nSonic families ({len(sonic_families)}):")
    for sf in sorted(sonic_families.keys()):
        print(f"  {sf}: {sonic_families[sf]}")

    return output


def cmd_semantics_report(args):
    """Print a human-readable semantic report."""
    census_dir = REPO_ROOT / "gen" / "census"
    index_path = census_dir / "pack_semantics.json"

    if not index_path.exists():
        print("Error: run 'cshot semantics' first")
        return

    with open(index_path) as f:
        data = json.load(f)

    files = data.get("files", {})
    print(f"Semantic Report — {len(files)} files annotated\n")

    tag_stats = {}
    sonic_families = {}
    primary_instruments = {}

    for entry in files.values():
        tags = entry.get("semantic_tags", {})
        for tag_type, tag_list in tags.items():
            if isinstance(tag_list, list):
                for tag in tag_list:
                    tag_stats[f"{tag_type}.{tag}"] = tag_stats.get(f"{tag_type}.{tag}", 0) + 1

        sf = entry.get("sonic_family", "unknown")
        sonic_families[sf] = sonic_families.get(sf, 0) + 1

        pi = tags.get("primary_instrument")
        if pi:
            primary_instruments[pi] = primary_instruments.get(pi, 0) + 1

    print("Top 30 Semantic Tags:")
    print(f"{'Tag':<35} {'Count':>6}")
    print("-" * 42)
    for tag, count in sorted(tag_stats.items(), key=lambda x: -x[1])[:30]:
        print(f"  {tag:<33} {count:>6}")

    print(f"\nSonic Families ({len(sonic_families)}):")
    for sf, count in sorted(sonic_families.items(), key=lambda x: -x[1]):
        print(f"  {sf:<25} {count}")

    print(f"\nPrimary Instruments ({len(primary_instruments)}):")
    for inst, count in sorted(primary_instruments.items(), key=lambda x: -x[1])[:15]:
        print(f"  {inst:<20} {count}")
