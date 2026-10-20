"""Buen Breadcrumb-bueskytteren holder i venstre hånd (eieren 20. oktober:
«det ser rart ut at den angriper med buen på ryggen og tomme hender»).
Kjør:

    python3.13 tools/models/bow.py

En reflekskbue av mørk eik med lærviklet grep og en streng av lin. Origo
`BELLY` bak grepet (der lemmene ville møttes), buen langs Z (lemmene opp og ned), buken (fronten bort fra
strengen) mot +X og strengen mot −X. Spillet setter den etter beinet «bow»
i bueskytteren (`mob3d.bow_bone`, `mob_rigs`), som har buens rom og står `BELLY` bak midten av neven. Rundt
800 trekanter, teksturlag i meter.

Akser som i town.py: Z opp."""

import math
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402,F401
import bmesh  # noqa: E402
from common import clear, new_object  # noqa: E402

import town  # noqa: E402
from town import PIECES, finish, layer  # noqa: E402

OUT = "assets/models/props"
HALF = 0.56  # halve lengden langs Z
BELLY = 0.075  # hvor langt buen bøyer seg fram midt på
RECURVE = 0.035  # tuppene som bøyer seg bakover
RINGS = 28
SIDES = 8


def axis(z):
    """Buens midtlinje: x som funksjon av z."""
    u = abs(z) / HALF
    return BELLY * math.cos(0.5 * math.pi * u) - RECURVE * u ** 6


def width(z):
    """Halve tykkelsen (fram, sidelengs): tykkest ved grepet, smalner mot tuppene."""
    u = abs(z) / HALF
    return 0.016 * (1 - 0.55 * u) + 0.004, 0.022 * (1 - 0.6 * u) + 0.005


def tube(name, zs, centre, radii, mat):
    """Et rør langs Z gjennom punktene `centre(z)` med ellipsen `radii(z)`,
    lukket i endene."""
    bm = bmesh.new()
    rings = []
    for z in zs:
        cx = centre(z)
        rx, ry = radii(z)
        rings.append([bm.verts.new((cx + rx * math.cos(a), ry * math.sin(a), z)) for a in (k * math.tau / SIDES for k in range(SIDES))])
    for a, b in zip(rings, rings[1:]):
        for k in range(SIDES):
            bm.faces.new((a[k], a[(k + 1) % SIDES], b[(k + 1) % SIDES], b[k]))
    bm.faces.new(rings[0][::-1])
    bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    ob = new_object(name, bm, [mat])
    town.uv_box(ob)
    PIECES.append(ob)
    return ob


def bow():
    clear()
    PIECES.clear()
    wood = layer("oak_planed", (0.42, 0.3, 0.22), 0.55)
    grip = layer("leather", (0.45, 0.28, 0.18), 0.65)
    linen = layer("linen", (0.86, 0.82, 0.72), 0.8)
    zs = [-HALF + 2 * HALF * k / RINGS for k in range(RINGS + 1)]
    tube("limbs", zs, axis, width, wood)
    # Grepet: lær viklet rundt midten, litt tykkere enn treet.
    gz = [-0.075 + 0.15 * k / 6 for k in range(7)]
    tube("grip", gz, axis, lambda z: (width(z)[0] + 0.006, width(z)[1] + 0.006), grip)
    # Strengen: fra tupp til tupp bak grepet, rett.
    tip = HALF - 0.02
    x0, x1 = axis(-tip) - 0.004, axis(tip) - 0.004
    tube("string", [-tip + 2 * tip * k / 8 for k in range(9)], lambda z: x0 + (x1 - x0) * (z + tip) / (2 * tip), lambda z: (0.0025, 0.0025), linen)
    finish(f"{OUT}/bow.glb", ao_samples=8, max_edge=0.3)


if __name__ == "__main__":
    bow()
