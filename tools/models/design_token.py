"""Designtokenet fiendene slipper (eieren 13. oktober: «synlige designtokens
som droppes, og plukkes opp når man går over dem»). Kjør:

    python3.13 tools/models/design_token.py

En sekskantet brikke, 16 cm over hjørnene og 2,2 cm tykk, som står på kant:
en kant av smidd jern (`layer_iron`) rundt en glødende flate på hver side
(`magic_paint`: gløder hele døgnet, i fargen fra instansen, så hvert token
får sin LO-TEK-farge; `mobs::TOKEN_COLORS`). Litt fasett i flaten, og et
lite hull midt i, så den leses som en brikke og ikke en mynt. Origo i
midten; spillet snurrer den om loddlinja og lar den sveve.

Akser som i town.py: Z opp, flatene vender mot ±Y."""

import math
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402,F401
import bmesh  # noqa: E402
from common import clear, new_object  # noqa: E402

import town  # noqa: E402
from town import PIECES, finish, layer, plain  # noqa: E402

OUT = "assets/models/props"
R_OUT = 0.08  # ytre radius (til hjørnene)
R_IN = 0.063  # flaten innenfor kanten
T = 0.011  # halv tykkelse på kanten
FACE = 0.0065  # flaten ligger litt inne i kanten
HOLE = 0.012  # hullet i midten


def hexagon(r, y, rot=math.pi / 6):
    return [(r * math.cos(rot + k * math.pi / 3), y, r * math.sin(rot + k * math.pi / 3)) for k in range(6)]


def link(name, bm, mat):
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    ob = new_object(name, bm, [mat])
    town.uv_box(ob)
    PIECES.append(ob)
    return ob


def token():
    clear()
    PIECES.clear()
    iron = layer("iron", (0.55, 0.52, 0.5), 0.45)
    glow = plain("magic_paint_token", (1.0, 1.0, 1.0), 0.35)
    # Kanten: en sekskantet ring, lukket, med fas på ytterkanten.
    bm = bmesh.new()
    rings = []
    for r, y in ((R_IN, -T), (R_OUT - 0.004, -T), (R_OUT, -T + 0.004), (R_OUT, T - 0.004), (R_OUT - 0.004, T), (R_IN, T)):
        rings.append([bm.verts.new(p) for p in hexagon(r, y)])
    for a, b in zip(rings, rings[1:] + rings[:1]):
        for k in range(6):
            bm.faces.new((a[k], a[(k + 1) % 6], b[(k + 1) % 6], b[k]))
    link("rim", bm, iron)
    # Flatene: en svakt hvelvet sekskant på hver side, med et lite hull.
    for side in (-1, 1):
        bm = bmesh.new()
        outer = [bm.verts.new(p) for p in hexagon(R_IN + 0.0005, side * FACE)]
        mid = [bm.verts.new(p) for p in hexagon(R_IN * 0.55, side * (FACE + 0.002))]
        inner = [bm.verts.new(p) for p in hexagon(HOLE, side * (FACE + 0.0015))]
        for a, b in ((outer, mid), (mid, inner)):
            for k in range(6):
                bm.faces.new((a[k], a[(k + 1) % 6], b[(k + 1) % 6], b[k]))
        link(f"face{side}", bm, glow)
    # Hullet gjennom: en kort sekskantet hylse av jern.
    bm = bmesh.new()
    a = [bm.verts.new(p) for p in hexagon(HOLE, -FACE - 0.0015)]
    b = [bm.verts.new(p) for p in hexagon(HOLE, FACE + 0.0015)]
    for k in range(6):
        bm.faces.new((a[k], a[(k + 1) % 6], b[(k + 1) % 6], b[k]))
    link("hole", bm, iron)
    finish(f"{OUT}/token.glb", ao_samples=16, max_edge=3.0)


if __name__ == "__main__":
    token()
