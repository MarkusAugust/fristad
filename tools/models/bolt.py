"""Markøren den korrupte inputen (og Toast-frosken) spytter (`mobs::Bolt`,
planen del 13e: «ranged attacks er vanskelige å se»). Kjør:

    python3.13 tools/models/bolt.py

En klump av giftgrønt lys, ujevn som spytt, med en lys kjerne og en hale
som snor seg bakover i fiolett og blir tynnere. Alt er `magic…` (gløder hele
døgnet i sin egen farge), så den synes i mørket i krypten. Rundt 500
trekanter. Origo midt i klumpen, fronten (fartsretningen) mot +X; spillet
dreier den etter farten og ruller den om lengdeaksen.

Akser som i town.py: Z opp."""

import math
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402,F401
import bmesh  # noqa: E402
from common import clear, new_object  # noqa: E402

import town  # noqa: E402
from town import PIECES, finish, plain  # noqa: E402

OUT = "assets/models/props"
R = 0.11  # klumpens radius
TAIL = 0.7  # halens lengde bak klumpen
RINGS = 9
SIDES = 8


def link(name, bm, mat):
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    ob = new_object(name, bm, [mat])
    town.uv_box(ob)
    PIECES.append(ob)
    return ob


def lump(r, seed, squash=1.0):
    """En ujevn kule: ikosaeder delt to ganger, dyttet ut og inn."""
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=2, radius=r)
    for v in bm.verts:
        x, y, z = v.co
        k = 1.0 + 0.16 * math.sin(7.0 * x / r + seed) * math.cos(5.0 * y / r - seed) + 0.08 * math.sin(11.0 * z / r + 2 * seed)
        # Strukket litt framover, flatere bak der halen tar over.
        stretch = 1.18 if x > 0 else squash
        v.co = (x * k * stretch, y * k, z * k)
    return bm


def tail(phase, r0, mat, name):
    """En hale som snor seg bakover og smalner til en spiss."""
    bm = bmesh.new()
    rings = []
    for i in range(RINGS):
        t = i / (RINGS - 1)
        x = -R * 0.5 - t * TAIL
        rad = r0 * (1.0 - t) ** 1.4 + 0.004
        wob = 0.035 * t * math.sin(phase + t * 7.0)
        cy, cz = wob * math.cos(phase), wob * math.sin(phase)
        rings.append([bm.verts.new((x, cy + rad * math.cos(a), cz + rad * math.sin(a))) for a in (k * math.tau / SIDES for k in range(SIDES))])
    for a, b in zip(rings, rings[1:]):
        for k in range(SIDES):
            bm.faces.new((a[k], a[(k + 1) % SIDES], b[(k + 1) % SIDES], b[k]))
    # Lokket foran (inne i klumpen).
    bm.faces.new(rings[0][::-1])
    link(name, bm, mat)


def bolt():
    clear()
    PIECES.clear()
    shell = plain("magic_bolt_green", (0.35, 1.0, 0.18), 0.4)
    core = plain("magic_bolt_core", (0.85, 1.0, 0.7), 0.4)
    violet = plain("magic_bolt_violet", (0.55, 0.18, 1.0), 0.4)
    link("shell", lump(R, 0.7, squash=0.8), shell)
    # Den lyse kjernen stikker ut foran, der klumpen er tynnest.
    bm = lump(R * 0.62, 2.1)
    bmesh.ops.translate(bm, verts=bm.verts[:], vec=(R * 0.42, 0.0, 0.0))
    link("core", bm, core)
    tail(0.0, R * 0.75, shell, "tail")
    tail(2.4, R * 0.45, violet, "wisp")
    finish(f"{OUT}/bolt.glb", ao_samples=8, max_edge=3.0)


if __name__ == "__main__":
    bolt()
