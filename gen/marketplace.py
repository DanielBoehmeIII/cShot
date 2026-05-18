"""Week 31 — Pack Marketplace: browse, preview, download generated kits."""
import json
import shutil
import sys
from pathlib import Path
from zipfile import ZipFile

from gen import REPO_ROOT


def _scan_kits(kits_dir: Path) -> list[dict]:
    """Scan kits directory and return metadata for all kits."""
    kits = []
    if not kits_dir.exists():
        return kits

    for kit_dir in sorted(kits_dir.iterdir()):
        if not kit_dir.is_dir():
            continue
        wav_files = sorted(kit_dir.rglob("*.wav"))
        if not wav_files:
            continue

        manifest_path = kit_dir / "manifest.json"
        manifest = {}
        if manifest_path.exists():
            manifest = json.load(open(manifest_path))

        coherence_path = kit_dir / "kit_audit.json"
        coherence = {}
        if coherence_path.exists():
            coherence = json.load(open(coherence_path))

        top_dir = kit_dir / "_top"
        previews = []
        if top_dir.exists():
            for w in sorted(top_dir.iterdir())[:5]:
                previews.append(str(w))

        kits.append({
            "name": kit_dir.name,
            "path": str(kit_dir),
            "total_files": len(wav_files),
            "manifest": manifest,
            "coherence": coherence.get("overall_coherence", coherence),
            "previews": previews,
            "categories": list(set(w.parent.name for w in wav_files if w.parent != kit_dir)),
        })

    return kits


def cmd_marketplace_serve(args):
    """Generate a marketplace HTML page from generated kits."""
    kits_dir = Path(args.kits_dir) if args.kits_dir else REPO_ROOT / "outputs" / "kits"
    out_path = Path(args.out) if args.out else REPO_ROOT / "docs" / "marketplace" / "index.html"

    kits = _scan_kits(kits_dir)

    if not kits:
        print(f"No kits found in {kits_dir}")
        print("Generate some kits first: cshot make 'dark rnb one shot kit'")
        return

    print(f"Marketplace: {len(kits)} kits found in {kits_dir}")
    print()

    cards = []
    for k in kits:
        name = k["name"].replace("_", " ").title()
        coherence = k.get("coherence", {})
        if isinstance(coherence, dict):
            coh_str = f"{coherence.get('overall_coherence', 0):.2f}"
        else:
            coh_str = f"{coherence:.2f}"
        cat_list = ", ".join(k["categories"][:6])
        preview_html = ""
        for p in k["previews"][:3]:
            preview_html += f'<audio controls style="width:100%;height:32px;"><source src="../../{p}" type="audio/wav"></audio>\n'

        cards.append(f"""
  <div class="kit-card">
    <h3>{name}</h3>
    <div class="kit-meta">{k['total_files']} sounds | Coherence: {coh_str}</div>
    <div class="kit-cats">{cat_list}</div>
    <div class="previews">{preview_html}</div>
    <a class="download-btn" href="#" onclick="alert('Download: {k['path']}')">Download</a>
  </div>""")

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>cShot Marketplace</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0a0a0a; color: #e0e0e0; padding: 40px; }}
  h1 {{ font-size: 2em; margin-bottom: 8px; background: linear-gradient(135deg, #a855f7, #3b82f6); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }}
  .subtitle {{ color: #888; margin-bottom: 32px; }}
  .grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 24px; }}
  .kit-card {{ background: #141414; border: 1px solid #222; border-radius: 12px; padding: 24px; }}
  .kit-card h3 {{ color: #e0e0e0; margin-bottom: 8px; }}
  .kit-meta {{ color: #777; font-size: 0.85em; margin-bottom: 8px; }}
  .kit-cats {{ color: #6366f1; font-size: 0.8em; margin-bottom: 12px; }}
  .previews {{ margin-bottom: 12px; }}
  .previews audio {{ margin-bottom: 4px; }}
  .download-btn {{ display: inline-block; padding: 8px 24px; background: #6366f1; color: #fff; text-decoration: none; border-radius: 6px; font-size: 0.9em; }}
  .count {{ color: #555; margin-bottom: 24px; }}
</style>
</head>
<body>
  <h1>cShot Marketplace</h1>
  <p class="subtitle">Generated one-shot kits</p>
  <p class="count">{len(kits)} kits available</p>
  <div class="grid">
    {''.join(cards)}
  </div>
</body>
</html>"""

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(html)
    print(f"Marketplace page: {out_path}")
    print(f"Open in browser to browse {len(kits)} kits")
