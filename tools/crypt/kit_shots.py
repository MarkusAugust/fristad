"""Bildene av bitene i byggesettet inne i prøvenivået, til godkjenningssiden
(`tools/qa/review.py` legger dem under studiobildene, og «Se i 3D» viser
biten blant de andre bitene i nivået).

For hver modell i byggesettet velges et sted i prøvenivået der den står
(`snapshot --dump-level`), og kameraet settes foran den: midt i cella på
skrå for cellene, foran muren for halvkantene, på skrå inn i hjørnet for
kvarthjørnene. Bildene skrives til `target/qa/review/kit/<modell>.png`, og
bitene i nivået til `target/qa/review/kit/nivaa_test.json`.

    python3 tools/crypt/kit_shots.py [modell ...]
"""

import json
import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
OUT = os.path.join(ROOT, "target", "qa", "review", "kit")
SNAP = os.path.join(ROOT, "target", "release", "snapshot")


def local(yaw, x, z):
    """Et punkt i bitens rom (x, z) dreid til nivåets rom (som `compose::transform`)."""
    s, c = math.sin(yaw), math.cos(yaw)
    return (c * x + s * z, -s * x + c * z)


def camera(kind, inst):
    """(øyet, yaw i grader, pitch i grader) for å se biten."""
    px, py, pz = inst["pos"]
    yaw = inst.get("yaw", 0.0)
    if kind in ("cell", "nedgang"):
        # Nedgangen har hullet mot +z: kameraet står på den andre siden.
        k = -1.3 if kind == "nedgang" else 1.3
        d = local(yaw, k, k)
        dx, dz = -d[0], -d[1]
        return (px + d[0], py + 1.6, pz + d[1]), math.degrees(math.atan2(-dx, -dz)), 12.0
    if kind == "rotunde":
        return (px, py + 1.6, pz + 4.2), 0.0, 18.0
    if kind == "edge":
        d = local(yaw, 0.0, 3.0)
    elif kind == "corner":
        d = local(yaw, 2.2, 2.2)
    elif kind == "torch":
        d = local(yaw, 0.0, 1.8)
        return (px + d[0], py, pz + d[1]), math.degrees(math.atan2(d[0], d[1])), -5.0
    else:
        d = local(yaw, 2.0, 2.0)
    return (px + d[0], py + 1.6, pz + d[1]), math.degrees(math.atan2(d[0], d[1])), 8.0


def main():
    only = sys.argv[1:]
    os.makedirs(OUT, exist_ok=True)
    level = os.path.join(OUT, "nivaa_test.json")
    subprocess.run([SNAP, "--level-doc", "test", "--dump-level", level], cwd=ROOT, check=True)
    instances = json.load(open(level))
    manifest = json.load(open(os.path.join(ROOT, "assets", "models", "crypt", "kit", "grav.json")))
    kinds = {}
    for p in manifest["pieces"]:
        kind = "rotunde" if p.get("footprint") else "nedgang" if any(m["kind"] == "down" for m in p.get("mounts", [])) else {"prop": "torch"}.get(p["kind"], p["kind"])
        for m in [p["model"]] + [v["model"] for v in p.get("variants", [])]:
            kinds.setdefault(m, kind)
    lines = []
    for model, kind in kinds.items():
        if only and model not in only:
            continue
        found = [i for i in instances if i["model"] == model]
        if not found:
            print(f"  {model}: står ikke i prøvenivået")
            continue
        # Et sted litt inne i nivået (ikke det første i hjørnet).
        inst = found[len(found) // 2]
        eye, yaw, pitch = camera(kind, inst)
        lines.append(f"{os.path.join(OUT, model + '.png')} {eye[0]:.2f},{eye[1]:.2f},{eye[2]:.2f} {yaw:.1f} {pitch:.1f}")
    shots = os.path.join(OUT, "shots.txt")
    with open(shots, "w") as f:
        f.write("\n".join(lines) + "\n")
    subprocess.run([SNAP, "--level-doc", "test", "--level", "1", "--film", "--size", "640x448", "--shots", shots], cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
