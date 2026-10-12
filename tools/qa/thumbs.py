"""Små bilder av hver modell til katalogen i redigeringen
(`docs/redigering.md`): ett studiobilde fra hjørnet per modell, som jpg i
`assets/catalog/thumbs/<navn>.jpg` (kopiert til `dist/catalog/` av
`tools/build-web.sh`). Bitene av kryptnivåene får ikke bilde.

    python3 tools/qa/thumbs.py [navn ...]     # uten navn: de som mangler
"""
import os
import subprocess
import sys
import tempfile

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "assets", "catalog", "thumbs")
VIEW = "corner:-35:14:1.25:0.4"


def models():
    base = os.path.join(ROOT, "assets", "models")
    for d, _, files in os.walk(base):
        for f in sorted(files):
            name = f[:-4]
            if not f.endswith(".glb") or (name.startswith("crypt_") and name[6:7].isdigit()):
                continue
            yield name, os.path.join(d, f)


def main(names):
    os.makedirs(OUT, exist_ok=True)
    if names:
        todo = [(n, p) for n, p in models() if n in names]
    else:
        todo = [(n, p) for n, p in models() if not os.path.exists(os.path.join(OUT, n + ".jpg"))]
    env = dict(os.environ, FRISTAD_STUDIO_SIZE="320x224")
    with tempfile.TemporaryDirectory() as tmp:
        for k, (name, path) in enumerate(todo):
            prefix = os.path.join(tmp, name)
            r = subprocess.run(["python3.13", os.path.join(ROOT, "tools", "qa", "studio.py"), path, prefix, VIEW], env=env, capture_output=True, text=True)
            png = prefix + "_corner.png"
            if not os.path.exists(png):
                print(f"{name}: ingen bilde\n{r.stderr[-400:]}", file=sys.stderr)
                continue
            Image.open(png).convert("RGB").save(os.path.join(OUT, name + ".jpg"), quality=82)
            print(f"[{k + 1}/{len(todo)}] {name}", flush=True)


if __name__ == "__main__":
    main(sys.argv[1:])
