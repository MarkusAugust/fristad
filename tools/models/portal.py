"""Portalen ved sørporten i den malte stilen, modellert i Blender (bpy) og
eksportert til assets/models/town/. Kjør:

    python3.13 tools/models/portal.py [portal_ring portal_runes portal_swirl portal_shard]

Steinringen (`portal_ring`): en rund portal av grove kilesteiner på en
plattform av heller, med to støttesteiner ved foten og runer hogd inn på
forsiden av hver stein. Steinene er malt i lagene `wall_stone`, `ruin` og
`flagstone`; bare silhuetten er geometri. Ringen stiger opp av bakken
(`PORTAL_RISE` i økta: den flyttes 3,4 m ned først), så alt må ligge over
y = −0,2 i modellen.

Når du har gått hjem fra kjelleren med H, våkner den: runene gløder
(`portal_runes`: de samme runene som lysende innlegg, materialet `magic…`
gløder fiolett hele døgnet), de to virvelskivene (`portal_swirl`: spiralarmer
i fiolett og turkis over en mørk kjerne, origo i midten, i xz-planet med
dybden langs y) dreies hver sin vei og pulserer om `PORTAL_CENTER`, og
splintene (`portal_shard`: små fasetterte steiner med glødende kant, origo
i midten) svever rundt ringen. Økta setter dem (`Session::portal_poses`).

Akser som i town.py: −Y fram (mot gata), Z opp, origo midt på
grunnflaten. Kolliderne står i `town::PORTAL_SOLIDS`: plattformen 4 × 2 m,
sidene ved x = ±1,3 og toppen ved y = 3,0; åpningen er fri."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import clear, new_object  # noqa: E402
from mathutils import Vector, noise  # noqa: E402

import town  # noqa: E402
from town import PIECES, beam, block, finish, layer, plain  # noqa: E402

OUT = "assets/models/town"

# Ringen: midten over plattformen, indre og ytre radius, og dybden (langs y).
# Må stemme med town::PORTAL_CENTER og PORTAL_RADIUS.
HC = 1.62
RI = 1.12
RO = 1.58
DEPTH = 0.5
STONES = 15
RUNE_R = (RI + RO) / 2


def link(name, bm, mat):
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    ob = new_object(name, bm, [mat])
    town.uv_box(ob)
    PIECES.append(ob)
    return ob


def mats():
    class Mats:
        pass

    m = Mats()
    m.stone = layer("wall_stone", (0.9, 0.88, 0.84), 0.65)
    m.dark = layer("ruin", (0.78, 0.76, 0.72), 0.65)
    m.flag = layer("flagstone", (0.84, 0.82, 0.78), 0.65)
    m.moss = layer("sod", (0.7, 0.8, 0.55), 0.6)
    m.cut = layer("ruin", (0.3, 0.28, 0.27), 0.65)
    return m


def wedge(name, a0, a1, r0, r1, depth, mat, at_y=0.0, rough=0.02, seed=0):
    """En kilestein i ringen: fra vinkel a0 til a1 (om ringmidten), radius
    r0 til r1, `depth` dyp langs y, med avfasede kanter og litt ujevn
    overflate (grov stein, som i ruinen)."""
    bm = bmesh.new()
    n = 4
    rows = []
    for (r, y) in ((r0, -depth / 2), (r1, -depth / 2), (r1, depth / 2), (r0, depth / 2)):
        rows.append([bm.verts.new((math.cos(a0 + (a1 - a0) * k / n) * r, y + at_y, HC + math.sin(a0 + (a1 - a0) * k / n) * r)) for k in range(n + 1)])
    for i in range(4):
        a, b = rows[i], rows[(i + 1) % 4]
        for k in range(n):
            bm.faces.new((a[k], a[k + 1], b[k + 1], b[k]))
    bm.faces.new([rows[0][0], rows[3][0], rows[2][0], rows[1][0]])
    bm.faces.new([rows[0][n], rows[1][n], rows[2][n], rows[3][n]])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bmesh.ops.bevel(bm, geom=bm.edges[:], offset=0.035, segments=1, affect="EDGES")
    off = Vector((seed * 1.7, seed * 2.9, seed * 0.7))
    for v in bm.verts:
        d = noise.noise(v.co * 5.0 + off) * rough
        c = Vector((0.0, v.co.y, HC))
        out = v.co - c
        out.y = 0.0
        if out.length > 1e-4:
            v.co += out.normalized() * d
    return link(name, bm, mat)


def glyph(name, cx, cz, angle, size, mat, rng, y, w=0.022, h=0.012):
    """En rune av streker (en stav og greiner), dreid `angle` om y, med
    midten i (cx, cz) på flaten y."""
    s = size
    stem = [((0.0, -0.5), (0.0, 0.5))]
    branches = [
        [((0.0, 0.5), (0.35, 0.2)), ((0.0, 0.1), (0.35, -0.2))],
        [((0.0, 0.5), (0.35, 0.25)), ((0.35, 0.25), (0.0, 0.0)), ((0.0, 0.0), (0.35, -0.5))],
        [((0.0, 0.35), (0.3, 0.0)), ((0.0, -0.35), (0.3, 0.0))],
        [((-0.3, 0.5), (0.3, -0.5)), ((0.3, 0.5), (-0.3, -0.5))],
        [((0.0, 0.0), (0.32, 0.4)), ((0.0, 0.0), (-0.32, 0.4))],
        [((0.0, 0.5), (0.3, 0.3)), ((0.0, 0.2), (0.3, 0.0))],
        [((-0.3, 0.3), (0.3, -0.3))],
        [((0.0, 0.5), (0.3, 0.2)), ((0.3, 0.2), (0.0, -0.1)), ((0.0, -0.1), (0.3, -0.5))],
    ]
    strokes = stem + rng.choice(branches)
    ca, sa = math.cos(angle), math.sin(angle)
    for k, ((x0, z0), (x1, z1)) in enumerate(strokes):
        p0 = (cx + (x0 * ca - z0 * sa) * s, y, cz + (x0 * sa + z0 * ca) * s)
        p1 = (cx + (x1 * ca - z1 * sa) * s, y, cz + (x1 * sa + z1 * ca) * s)
        beam(f"{name}_{k}", p0, p1, w, h, mat, roll=math.pi / 2)


def runes(mat, y, w=0.022, h=0.012):
    """Én rune på forsiden av hver kilestein, dreid så den står langs radien."""
    rng = random.Random(1350)
    for i in range(STONES):
        a = (i + 0.5) / STONES * 2 * math.pi - math.pi / 2
        cx, cz = math.cos(a) * RUNE_R, HC + math.sin(a) * RUNE_R
        glyph(f"rune{i}", cx, cz, a - math.pi / 2, 0.26, mat, rng, y, w, h)


def ring_stones(m):
    rng = random.Random(7)
    gap = 0.014
    step = 2 * math.pi / STONES
    for i in range(STONES):
        a0 = i * step - math.pi / 2 + gap
        a1 = (i + 1) * step - math.pi / 2 - gap
        # Annenhver stein stikker litt lenger ut, som i en ekte bue.
        r1 = RO + (0.06 if i % 2 else 0.0) + rng.uniform(-0.02, 0.02)
        wedge(f"voussoir{i}", a0, a1, RI + rng.uniform(-0.012, 0.012), r1, DEPTH + rng.uniform(-0.03, 0.04), m.stone if i % 3 else m.dark, seed=i)
    # Sluttsteinen øverst: større.
    wedge("keystone", math.pi / 2 - step * 0.62, math.pi / 2 + step * 0.62, RI - 0.06, RO + 0.16, DEPTH + 0.1, m.dark, seed=99)


def platform(m):
    """Plattformen: heller lagt i forband, et trinn foran, og to
    støttesteiner som holder ringen ved foten."""
    rng = random.Random(11)
    W, D, H = 4.0, 2.0, 0.16
    block("plinth", (W, D, H), m.dark, at=(0, 0, H / 2 - 0.04), bevel=0.03)
    nx, ny = 5, 3
    for ix in range(nx):
        for iy in range(ny):
            w = W / nx - 0.03
            d = D / ny - 0.03
            x = -W / 2 + (ix + 0.5) * W / nx
            y = -D / 2 + (iy + 0.5) * D / ny
            block(f"flag{ix}_{iy}", (w, d, 0.05), m.flag, at=(x, y, H - 0.04 + 0.025 + rng.uniform(-0.008, 0.008)), rot=(rng.uniform(-0.01, 0.01), rng.uniform(-0.01, 0.01), rng.uniform(-0.02, 0.02)), bevel=0.012)
    # Trinnet foran.
    block("step", (2.6, 0.45, 0.1), m.flag, at=(0, -D / 2 - 0.22, 0.05 - 0.04), bevel=0.02)
    # Støttesteinene: tunge, skrå blokker på hver side av foten.
    for sx in (-1, 1):
        block(f"buttress{sx}", (0.55, DEPTH + 0.3, 0.9), m.stone, at=(sx * 1.05, 0, 0.45 + H - 0.08), rot=(0, sx * 0.35, 0), bevel=0.05, segments=2)
        block(f"buttress_cap{sx}", (0.5, DEPTH + 0.36, 0.12), m.dark, at=(sx * 1.18, 0, 0.92 + H - 0.08), rot=(0, sx * 0.35, 0), bevel=0.03)
        block(f"moss{sx}", (0.42, DEPTH + 0.2, 0.03), m.moss, at=(sx * 1.2, 0, 0.99 + H - 0.08), rot=(0, sx * 0.35, 0), bevel=0.012)


def portal_ring():
    clear()
    PIECES.clear()
    m = mats()
    platform(m)
    ring_stones(m)
    # Runene hogd inn: mørke innlegg i flaten foran.
    runes(m.cut, -DEPTH / 2 - 0.002, w=0.028, h=0.01)
    finish(f"{OUT}/portal_ring.glb", max_edge=1.0)


def portal_runes():
    clear()
    PIECES.clear()
    glow = plain("magic_rune", (0.55, 0.3, 1.0), 0.4)
    runes(glow, -DEPTH / 2 - 0.012, w=0.032, h=0.014)
    finish(f"{OUT}/portal_runes.glb", ao_samples=16, max_edge=3.0)


def spiral_arm(name, a0, mat, y, turns=1.15, n=36, w0=0.015, w1=0.2, r0=0.04, r1=1.1):
    """En spiralarm i xz-planet: et bånd som vider seg ut og vrir seg."""
    bm = bmesh.new()
    left, right = [], []
    for k in range(n + 1):
        t = k / n
        r = r0 + (r1 - r0) * t
        a = a0 + turns * 2 * math.pi * t
        w = w0 + (w1 - w0) * math.sin(t * math.pi * 0.9) ** 0.7
        p = Vector((math.cos(a) * r, y, math.sin(a) * r))
        q = Vector((math.cos(a + 0.01) * (r + 0.001), y, math.sin(a + 0.01) * (r + 0.001)))
        side = (q - p).normalized().cross(Vector((0, 1, 0))).normalized()
        left.append(bm.verts.new(p + side * w / 2))
        right.append(bm.verts.new(p - side * w / 2))
    for k in range(n):
        bm.faces.new((left[k], left[k + 1], right[k + 1], right[k]))
    bmesh.ops.solidify(bm, geom=bm.faces[:] + bm.edges[:] + bm.verts[:], thickness=0.008)
    return link(name, bm, mat)


def disc(name, r, y, mat, n=40):
    bm = bmesh.new()
    c = bm.verts.new((0, y, 0))
    ring = [bm.verts.new((math.cos(2 * math.pi * k / n) * r, y, math.sin(2 * math.pi * k / n) * r)) for k in range(n)]
    for k in range(n):
        bm.faces.new((c, ring[k], ring[(k + 1) % n]))
    bmesh.ops.solidify(bm, geom=bm.faces[:] + bm.edges[:] + bm.verts[:], thickness=0.006)
    return link(name, bm, mat)


def annulus(name, r0, r1, y, mat, n=48):
    bm = bmesh.new()
    inner = [bm.verts.new((math.cos(2 * math.pi * k / n) * r0, y, math.sin(2 * math.pi * k / n) * r0)) for k in range(n)]
    outer = [bm.verts.new((math.cos(2 * math.pi * k / n) * r1, y, math.sin(2 * math.pi * k / n) * r1)) for k in range(n)]
    for k in range(n):
        j = (k + 1) % n
        bm.faces.new((inner[k], outer[k], outer[j], inner[j]))
    bmesh.ops.solidify(bm, geom=bm.faces[:] + bm.edges[:] + bm.verts[:], thickness=0.006)
    return link(name, bm, mat)


def portal_swirl():
    """Virvelskiva, i lag forfra og bakover: et lyst øye med glorie, en indre
    virvel som vrir seg motsatt vei, lyse streker, brede armer i fiolett,
    magenta, dypblått og turkis, en lysende kant innenfor steinene, og en
    mørk kjerne bakerst. Origo i midten; økta dreier to slike hver sin vei
    og skalerer dem når portalen åpner seg."""
    clear()
    PIECES.clear()
    rng = random.Random(5)
    core = plain("magic_portal_core", (0.03, 0.01, 0.07), 0.4)
    shades = [
        plain("magic_portal_violet", (0.3, 0.08, 0.7), 0.4),
        plain("magic_portal_magenta", (0.55, 0.08, 0.5), 0.4),
        plain("magic_portal_deep", (0.08, 0.1, 0.55), 0.4),
        plain("magic_portal_teal", (0.06, 0.4, 0.65), 0.4),
    ]
    streak = plain("magic_portal_streak", (0.5, 0.65, 0.95), 0.4)
    rim = plain("magic_portal_rim", (0.25, 0.35, 0.85), 0.4)
    halo = plain("magic_portal_halo", (0.3, 0.35, 0.7), 0.4)
    eye = plain("magic_portal_eye", (0.75, 0.8, 0.95), 0.4)
    disc("core", 1.1, 0.05, core)
    annulus("rim", 1.0, 1.1, 0.02, rim)
    for k in range(8):
        spiral_arm(f"arm{k}", k / 8 * 2 * math.pi + rng.uniform(-0.1, 0.1), shades[k % 4], 0.0 - 0.003 * k, turns=1.25 + rng.uniform(-0.1, 0.15), w1=rng.uniform(0.12, 0.22), r1=1.04)
    for k in range(10):
        spiral_arm(f"streak{k}", rng.uniform(0, 2 * math.pi), streak, -0.035 - 0.002 * (k % 4), turns=1.5 + rng.uniform(-0.2, 0.2), w0=0.004, w1=rng.uniform(0.012, 0.03), r0=rng.uniform(0.15, 0.35), r1=rng.uniform(0.7, 1.02), n=40)
    for k in range(5):
        spiral_arm(f"inner{k}", k / 5 * 2 * math.pi + 0.3, shades[(k + 1) % 4], -0.06 - 0.003 * k, turns=-0.85, w1=0.09, r1=0.62)
    disc("halo", 0.26, -0.085, halo, n=24)
    disc("eye", 0.11, -0.095, eye, n=16)
    finish(f"{OUT}/portal_swirl.glb", ao_samples=16, max_edge=3.0)


def portal_shard():
    """En splint: en liten fasettert stein (ruin) med en glødende kant av
    magi langs den ene siden. Origo i midten."""
    clear()
    PIECES.clear()
    m = mats()
    glow = plain("magic_shard", (0.5, 0.3, 1.0), 0.4)
    bm = bmesh.new()
    pts = [(0, 0, 0.17), (0, 0, -0.13), (0.05, 0, 0.0), (-0.045, 0.0, 0.01), (0, 0.05, -0.01), (0, -0.045, 0.02), (0.03, 0.03, 0.08), (-0.03, -0.02, -0.07)]
    vs = [bm.verts.new(p) for p in pts]
    bmesh.ops.convex_hull(bm, input=vs)
    link("shard", bm, m.dark)
    # Den glødende kanten: en tynn fasett langs ryggen.
    bm = bmesh.new()
    pts = [(0, -0.052, 0.14), (0, -0.056, -0.1), (0.02, -0.05, 0.02), (-0.02, -0.05, 0.03), (0, -0.04, 0.0)]
    vs = [bm.verts.new(p) for p in pts]
    bmesh.ops.convex_hull(bm, input=vs)
    link("edge", bm, glow)
    finish(f"{OUT}/portal_shard.glb", ao_samples=16, max_edge=3.0)


MODELS = {"portal_ring": portal_ring, "portal_runes": portal_runes, "portal_swirl": portal_swirl, "portal_shard": portal_shard}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for n in names:
        MODELS[n]()
