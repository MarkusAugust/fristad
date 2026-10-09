"""Portalen ved sørporten, modellert i Blender (bpy) og eksportert til
assets/models/town/. Kjør:

    python3 tools/models/portal.py [portal_ring portal_runes portal_swirl portal_shard]

Steinringen (`portal_ring`): en rund portal av hogde kilesteiner på en
plattform av heller, med to støttesteiner ved foten og innhogde runer på
forsiden av hver stein. Den står alltid ved sørporten, sovende.

Når du har gått hjem fra kjelleren med H, våkner den: runene gløder
(`portal_runes`, de samme runene som lysende innlegg), virvelen
(`portal_swirl`: spiralarmer i fiolett og turkis over en mørk kjerne, med
origo midt i ringen) dreies og pulserer, og krystallsplinter
(`portal_shard`) svever rundt ringen. Økta dreier og skalerer dem
(`Session::portal_poses`).

Akser som i town.py: −Y fram, Z opp, origo midt på grunnflaten (virvelen
og splinten: origo i midten)."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import clear  # noqa: E402
from mathutils import Vector, noise  # noqa: E402

import town  # noqa: E402
from town import PIECES, beam, block, finish, layer, plain  # noqa: E402
from walls import link  # noqa: E402

OUT = "assets/models/town"

# Ringen: midten, indre og ytre radius, og dybden (langs y).
HC = 1.62
RI = 1.12
RO = 1.58
DEPTH = 0.5
STONES = 17
# Runene sitter på forsiden (y = −DEPTH/2), på midtradien.
RUNE_R = (RI + RO) / 2


def mats():
    class Mats:
        pass

    m = Mats()
    m.stone = layer("stone", (0.56, 0.55, 0.53), 0.5)
    m.dark = layer("stone", (0.42, 0.41, 0.4), 0.5)
    m.flag = layer("concrete", (0.6, 0.58, 0.55), 0.5)
    m.moss = layer("sod", (0.5, 0.62, 0.42), 0.5)
    m.cut = layer("stone", (0.14, 0.13, 0.13), 0.5)
    return m


def wedge(name, a0, a1, r0, r1, depth, mat, at_y=0.0, rough=0.012, seed=0):
    """En kilestein i ringen: fra vinkel a0 til a1 (om ringmidten), radius
    r0 til r1, `depth` dyp langs y, med avfasede kanter og litt ujevn
    overflate."""
    bm = bmesh.new()
    n = 6
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
    bmesh.ops.bevel(bm, geom=bm.edges[:], offset=0.025, segments=2, affect="EDGES")
    off = Vector((seed * 1.7, seed * 2.9, seed * 0.7))
    for v in bm.verts:
        d = noise.noise(v.co * 6.0 + off) * rough
        c = Vector((0.0, v.co.y, HC))
        out = (v.co - c)
        out.y = 0.0
        if out.length > 1e-4:
            v.co += out.normalized() * d
    ob = link(name, bm, mat)
    town.uv_box(ob)
    return ob


def glyph(name, cx, cz, angle, size, mat, rng, y, w=0.022, h=0.012):
    """En rune av streker (som de eldre runene: en stav og greiner), dreid
    `angle` om y, med midten i (cx, cz) på flaten y."""
    s = size
    stem = [((0.0, -0.5), (0.0, 0.5))]
    branches = [
        [((0.0, 0.5), (0.35, 0.2)), ((0.0, 0.1), (0.35, -0.2))],  # ansuz
        [((0.0, 0.5), (0.35, 0.25)), ((0.35, 0.25), (0.0, 0.0)), ((0.0, 0.0), (0.35, -0.5))],  # raido
        [((0.0, 0.35), (0.3, 0.0)), ((0.0, -0.35), (0.3, 0.0))],  # thurisaz-ish
        [((-0.3, 0.5), (0.3, -0.5)), ((0.3, 0.5), (-0.3, -0.5))],  # gebo + stav
        [((0.0, 0.0), (0.32, 0.4)), ((0.0, 0.0), (-0.32, 0.4))],  # algiz
        [((0.0, 0.5), (0.3, 0.3)), ((0.0, 0.2), (0.3, 0.0))],  # fehu
        [((-0.3, 0.3), (0.3, -0.3))],  # nauthiz
        [((0.0, 0.5), (0.3, 0.2)), ((0.3, 0.2), (0.0, -0.1)), ((0.0, -0.1), (0.3, -0.5))],  # berkano-ish
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
        glyph(f"rune{i}", cx, cz, a - math.pi / 2, 0.24, mat, rng, y, w, h)


def ring_stones(m):
    rng = random.Random(7)
    gap = 0.012
    step = 2 * math.pi / STONES
    for i in range(STONES):
        a0 = i * step - math.pi / 2 + gap
        a1 = (i + 1) * step - math.pi / 2 - gap
        # Annenhver stein stikker litt lenger ut, som i en ekte bue.
        r1 = RO + (0.05 if i % 2 else 0.0) + rng.uniform(-0.015, 0.015)
        wedge(f"voussoir{i}", a0, a1, RI + rng.uniform(-0.01, 0.01), r1, DEPTH + rng.uniform(-0.02, 0.03), m.stone if i % 3 else m.dark, seed=i)
    # Sluttsteinen øverst: større, med en innhogd sirkel.
    wedge("keystone", math.pi / 2 - step * 0.62, math.pi / 2 + step * 0.62, RI - 0.06, RO + 0.14, DEPTH + 0.1, m.dark, at_y=0.0, seed=99)


def platform(m):
    """Plattformen: heller lagt i forband, et trinn foran, og to
    støttesteiner som holder ringen ved foten."""
    rng = random.Random(11)
    W, D, H = 4.0, 2.0, 0.16
    block("plinth", (W, D, H), m.dark, at=(0, 0, H / 2 - 0.04), bevel=0.03)
    nx, ny = 6, 3
    for ix in range(nx):
        for iy in range(ny):
            w = W / nx - 0.025
            d = D / ny - 0.025
            x = -W / 2 + (ix + 0.5) * W / nx + (0.5 * W / nx if iy % 2 else 0) * 0
            y = -D / 2 + (iy + 0.5) * D / ny
            block(f"flag{ix}_{iy}", (w, d, 0.05), m.flag, at=(x, y, H - 0.04 + 0.025 + rng.uniform(-0.006, 0.006)), rot=(rng.uniform(-0.01, 0.01), rng.uniform(-0.01, 0.01), rng.uniform(-0.02, 0.02)), bevel=0.012)
    # Trinnet foran.
    block("step", (2.6, 0.45, 0.1), m.flag, at=(0, -D / 2 - 0.22, 0.05 - 0.04), bevel=0.02)
    # Støttesteinene: tunge, skrå blokker på hver side av foten.
    for sx in (-1, 1):
        block(f"buttress{sx}", (0.55, DEPTH + 0.3, 0.9), m.stone, at=(sx * 1.05, 0, 0.45 + H - 0.08), rot=(0, sx * 0.35, 0), bevel=0.05, segments=2)
        block(f"buttress_cap{sx}", (0.5, DEPTH + 0.36, 0.12), m.dark, at=(sx * 1.18, 0, 0.92 + H - 0.08), rot=(0, sx * 0.35, 0), bevel=0.03)
    # Mose i fugene nederst og på toppen av støttesteinene.
    for sx in (-1, 1):
        block(f"moss{sx}", (0.42, DEPTH + 0.2, 0.03), m.moss, at=(sx * 1.2, 0, 0.99 + H - 0.08), rot=(0, sx * 0.35, 0), bevel=0.012)


def portal_ring():
    clear()
    PIECES.clear()
    m = mats()
    platform(m)
    ring_stones(m)
    # Runene hogd inn: mørke innlegg i flaten foran og bak.
    runes(m.cut, -DEPTH / 2 - 0.002, w=0.026, h=0.01)
    # Kolliderne settes i byen (town::portal): plattformen, sidene og toppen
    # av ringen; åpningen er fri, så virvelen kan siktes på.
    finish(f"{OUT}/portal_ring.glb")


def portal_runes():
    clear()
    PIECES.clear()
    glow = plain("magic_rune", (0.12, 0.32, 0.6), 0.4)
    runes(glow, -DEPTH / 2 - 0.009, w=0.03, h=0.012)
    finish(f"{OUT}/portal_runes.glb")


def spiral_arm(name, a0, mat, y, turns=1.15, n=40, w0=0.015, w1=0.2, r0=0.04, r1=1.1):
    """En spiralarm i xz-planet: et bånd som vider seg ut og vrir seg."""
    bm = bmesh.new()
    left, right = [], []
    for k in range(n + 1):
        t = k / n
        r = r0 + (r1 - r0) * t
        a = a0 + turns * 2 * math.pi * t
        w = w0 + (w1 - w0) * math.sin(t * math.pi * 0.9) ** 0.7
        # Retningen langs armen og ut fra den.
        da = a + 0.01
        p = Vector((math.cos(a) * r, y, math.sin(a) * r))
        q = Vector((math.cos(da) * (r + 0.001), y, math.sin(da) * (r + 0.001)))
        along = (q - p).normalized()
        side = along.cross(Vector((0, 1, 0))).normalized()
        left.append(bm.verts.new(p + side * w / 2))
        right.append(bm.verts.new(p - side * w / 2))
    for k in range(n):
        bm.faces.new((left[k], left[k + 1], right[k + 1], right[k]))
    ob = link(name, bm, mat)
    town.uv_box(ob)
    sol = ob.modifiers.new("t", "SOLIDIFY")
    sol.thickness = 0.008
    return ob


def disc(name, r, y, mat, n=48):
    bm = bmesh.new()
    c = bm.verts.new((0, y, 0))
    ring = [bm.verts.new((math.cos(2 * math.pi * k / n) * r, y, math.sin(2 * math.pi * k / n) * r)) for k in range(n)]
    for k in range(n):
        bm.faces.new((c, ring[k], ring[(k + 1) % n]))
    ob = link(name, bm, mat)
    town.uv_box(ob)
    sol = ob.modifiers.new("t", "SOLIDIFY")
    sol.thickness = 0.006
    return ob


def annulus(name, r0, r1, y, mat, n=64):
    bm = bmesh.new()
    inner = [bm.verts.new((math.cos(2 * math.pi * k / n) * r0, y, math.sin(2 * math.pi * k / n) * r0)) for k in range(n)]
    outer = [bm.verts.new((math.cos(2 * math.pi * k / n) * r1, y, math.sin(2 * math.pi * k / n) * r1)) for k in range(n)]
    for k in range(n):
        j = (k + 1) % n
        bm.faces.new((inner[k], outer[k], outer[j], inner[j]))
    ob = link(name, bm, mat)
    town.uv_box(ob)
    sol = ob.modifiers.new("t", "SOLIDIFY")
    sol.thickness = 0.006
    return ob


def portal_swirl():
    """Virvelen, i lag forfra og bakover: et hvitt øye med glorie, en indre
    virvel som vrir seg motsatt vei, tynne lyse streker, brede armer i
    fiolett, magenta, dypblått og turkis, en lysende kant innenfor steinene,
    og en mørk kjerne bakerst som skinner gjennom mellom armene. Origo i
    midten; økta dreier to slike hver sin vei."""
    clear()
    PIECES.clear()
    rng = random.Random(5)
    core = plain("magic_portal_core", (0.02, 0.006, 0.05), 0.4)
    shades = [
        plain("magic_portal_violet", (0.16, 0.03, 0.4), 0.4),
        plain("magic_portal_magenta", (0.32, 0.03, 0.3), 0.4),
        plain("magic_portal_deep", (0.04, 0.05, 0.32), 0.4),
        plain("magic_portal_teal", (0.03, 0.24, 0.42), 0.4),
    ]
    streak = plain("magic_portal_streak", (0.3, 0.45, 0.7), 0.4)
    rim = plain("magic_portal_rim", (0.12, 0.2, 0.55), 0.4)
    halo = plain("magic_portal_halo", (0.16, 0.2, 0.45), 0.4)
    eye = plain("magic_portal_eye", (0.45, 0.5, 0.65), 0.4)
    disc("core", 1.1, 0.05, core)
    annulus("rim", 1.0, 1.1, 0.02, rim)
    for k in range(10):
        spiral_arm(f"arm{k}", k / 10 * 2 * math.pi + rng.uniform(-0.1, 0.1), shades[k % 4], 0.0 - 0.003 * k, turns=1.25 + rng.uniform(-0.1, 0.15), w1=rng.uniform(0.1, 0.2), r1=1.04)
    for k in range(16):
        spiral_arm(f"streak{k}", rng.uniform(0, 2 * math.pi), streak, -0.035 - 0.002 * (k % 4), turns=1.5 + rng.uniform(-0.2, 0.2), w0=0.004, w1=rng.uniform(0.012, 0.03), r0=rng.uniform(0.15, 0.35), r1=rng.uniform(0.7, 1.02), n=48)
    for k in range(6):
        spiral_arm(f"inner{k}", k / 6 * 2 * math.pi + 0.3, shades[(k + 1) % 4], -0.06 - 0.003 * k, turns=-0.85, w1=0.09, r1=0.62)
    disc("halo", 0.26, -0.085, halo, n=32)
    disc("eye", 0.11, -0.095, eye, n=24)
    for o in list(bpy.data.objects):
        if o.type == "MESH" and o.modifiers:
            bpy.context.view_layer.objects.active = o
            for md in list(o.modifiers):
                bpy.ops.object.modifier_apply(modifier=md.name)
    finish(f"{OUT}/portal_swirl.glb")


def portal_shard():
    """En krystallsplint: et langstrakt, skjevt oktaeder som gløder."""
    clear()
    PIECES.clear()
    glow = plain("magic_shard", (0.15, 0.35, 0.6), 0.4)
    bm = bmesh.new()
    pts = [(0, 0, 0.16), (0, 0, -0.12), (0.045, 0, 0.0), (-0.04, 0.0, 0.01), (0, 0.045, -0.01), (0, -0.04, 0.02)]
    vs = [bm.verts.new(p) for p in pts]
    bmesh.ops.convex_hull(bm, input=vs)
    link("shard", bm, glow)
    town.uv_box(bpy.data.objects["shard"])
    finish(f"{OUT}/portal_shard.glb")


MODELS = {"portal_ring": portal_ring, "portal_runes": portal_runes, "portal_swirl": portal_swirl, "portal_shard": portal_shard}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for n in names:
        MODELS[n]()
