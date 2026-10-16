"""Grunnbildene og dybden til filmens malerier som males over motoren.

Eieren maler over et skjermbilde fra motoren (docs/prompter-film-2.md), så
maleriet har samme hus og perspektiv som verden. Filmen projiserer maleriet
tilbake på verden fra samme sted, etter avstanden motoren selv målte, så det
ligger på murene og bakken mens kameraet flyr.

  python3 tools/film/paintover.py base    # grunnbildene, 2560 × 1440
  python3 tools/film/paintover.py depth   # dybden, 640 × 360

Stedene står i `web/film/opening.json` (`paintover`). Dybden skrives
som `assets/film/paintover/<navn>.depth.png`: avstanden langs kameraets akse
i centimeter, høy byte i rødt og lav i grønt (himmelen 65535).
"""

import json
import math
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets/film/paintover"
SNAPSHOT = ROOT / "target/release/snapshot"


def shots():
    script = json.loads((ROOT / "web/film/opening.json").read_text())
    return script.get("paintover", [])


def run(lines, size, extra=()):
    with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as f:
        f.write("\n".join(lines) + "\n")
    subprocess.run([str(SNAPSHOT), "--size", size, *extra, "--film-seq", f.name], check=True, cwd=ROOT)


def line(out, p):
    x, y, z = p["eye"]
    return f"{out} {x} {y} {z} {p['yaw']} {p['pitch']} {p['fov']} 0 {1 if p.get('night') else 0} 0.5"


def srgb_to_linear(c):
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def decode(png, fov):
    """Avstanden fra `--debug depth` (16 m, 1 m og resten i hver sin kanal)
    som avstand langs kameraets akse."""
    a = srgb_to_linear(np.asarray(Image.open(png).convert("RGB")).astype(np.float64) / 255.0)
    coarse = np.round(a[..., 0] * 16)
    metre = np.round(a[..., 1] * 16)
    d = coarse * 16 + metre + a[..., 2]
    sky = coarse >= 16
    h, w = d.shape
    tan_v = math.tan(math.radians(fov) / 2)
    tan_h = tan_v * w / h
    u = (np.arange(w) + 0.5) / w * 2 - 1
    v = (np.arange(h) + 0.5) / h * 2 - 1
    ray = np.sqrt(1 + (u[None, :] * tan_h) ** 2 + (v[:, None] * tan_v) ** 2)
    cm = np.clip(np.round(d / ray * 100), 0, 65534)
    cm[sky] = 65535
    return cm.astype(np.uint32)


def main(what):
    OUT.mkdir(parents=True, exist_ok=True)
    if what == "base":
        (OUT / "base").mkdir(exist_ok=True)
        run([line(OUT / "base" / f"{p['id']}.png", p) for p in shots()], "2560x1440")
        for p in shots():
            png = OUT / "base" / f"{p['id']}.png"
            Image.open(png).convert("RGB").save(png.with_suffix(".jpg"), quality=90)
            png.unlink()
    elif what == "depth":
        with tempfile.TemporaryDirectory() as tmp:
            run([line(Path(tmp) / f"{p['id']}.png", p) for p in shots()], "640x360", ["--no-smaa", "--debug", "depth"])
            for p in shots():
                cm = decode(Path(tmp) / f"{p['id']}.png", p["fov"])
                rgb = np.stack([cm >> 8, cm & 255, np.zeros_like(cm)], -1).astype(np.uint8)
                Image.fromarray(rgb).save(OUT / f"{p['id']}.depth.png")
                print(p["id"], f"{cm[cm < 65535].min() / 100:.2f}–{cm[cm < 65535].max() / 100:.2f} m")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "")
