"""Smia og Apoteket i den malte stilen (docs/ny-stil.md), bygd med
byggesettet (kit.py). Kjør:

    python3 tools/models/workshops.py

Gir for hvert hus utsida (`forge.glb`, `apothecary.glb`, tegnes alltid),
innsida (`forge_interior.glb`, `apothecary_interior.glb`, tegnes bare når
du er nær) og `<navn>_inside.json` med kolliderne, rampene, lysene, lykta
og pipa for begge.

Smia (Blender: x til høyre, −y mot torget):
- Steinmurer på tre sider og en bindingsverksfasade mot torget med et stort
  åpent verksted mellom to grove stolper. Teglsaltak med mønet langs
  fasaden, og ei klumpete pipe fra essa gjennom bakre takflate.
- Inne: essa med glør og hette, blåsebelgen, ambolten på en kubbe,
  kjølekaret, verktøyveggen, arbeidsbenken med skrustikke og et stativ
  med ferdige våpen. Bakdøra går ut i bakgården.
- Bakgården innenfor et flettverksgjerde: kullbinge, vedstabel under et
  lite skur, slipestein og høner.
Essa, ambolten og kjølekaret er med i utsida, så verkstedet ikke står tomt
sett fra torget (innsida tegnes bare nær).

Apoteket:
- Steinsokkel, bindingsverk i to etasjer med hvit puss, overetasjen krager
  ut mot torget. Skifertak, og ei pipe fra destillerovnen bak.
- Første etasje: salgsrommet med salgsluke mot torget (skodda slått ned
  til et bord), disken, hyller med krukker og flasker og urtebunter fra
  bjelkene; trappa opp langs østveggen. Laboratoriet i bakrommet med
  destillerovn, kolber, mortere, bord og bokhylle.
- Urteloftet oppe under sperrene: tørkestativ, urter fra hanebjelkene,
  kurver, sekker og apotekerens seng."""

import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402,F401
import bmesh  # noqa: E402
import kit  # noqa: E402
import town  # noqa: E402
from common import clear  # noqa: E402
from kit import CARVED, DARK_BEAM, IRON_BAND, MOULDING, ROPE, barrel, box, candle, door, fire, half_timbered, planks, stairs, stick, stool, table, window  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from town import collide, layer, light, piece, plain  # noqa: E402

OUT = "assets/models/town"


# --- Felles småting -------------------------------------------------------------


def revolve(name, prof, mat, n=8, at=(0, 0, 0)):
    """Et omdreiningslegeme fra profilen [(r, z), …] nedenfra og opp, lukket
    (radius 0 gir en spiss). Uv i meter rundt og oppover."""
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    rings = []
    for r, z in prof:
        if r < 1e-4:
            rings.append([bm.verts.new((at[0], at[1], at[2] + z))])
        else:
            rings.append([bm.verts.new((at[0] + r * math.cos(2 * math.pi * i / n), at[1] + r * math.sin(2 * math.pi * i / n), at[2] + z)) for i in range(n)])
    circ = 2 * math.pi * max(r for r, _ in prof)
    for lo, hi in zip(rings, rings[1:]):
        for i in range(n):
            j = (i + 1) % n
            if len(lo) == 1 and len(hi) == 1:
                continue
            if len(lo) == 1:
                vs, ks = (lo[0], hi[j], hi[i]), (i + 0.5, j if j else n, i)
            elif len(hi) == 1:
                vs, ks = (lo[i], lo[j], hi[0]), (i, j if j else n, i + 0.5)
            else:
                vs, ks = (lo[i], lo[j], hi[j], hi[i]), (i, j if j else n, j if j else n, i)
            f = bm.faces.new(vs)
            for lp, k in zip(f.loops, ks):
                lp[uvl].uv = (k / n * circ, lp.vert.co.z)
    for ring, top in ((rings[0], False), (rings[-1], True)):
        if len(ring) > 1:
            f = bm.faces.new(ring if top else ring[::-1])
            for lp in f.loops:
                lp[uvl].uv = (lp.vert.co.x, lp.vert.co.y)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return kit._object(name, bm, mat)


def turned(ob, origin, rot):
    """Drei en del om `origin` (rotasjonsmatrise 4×4)."""
    o = Vector(origin)
    ob.matrix_world = Matrix.Translation(o) @ rot @ Matrix.Translation(-o) @ ob.matrix_world
    return ob


def ramp_to(x0, x1, y_out, y_in, z_out, z_in):
    """En rampe over et trinn fra y_out (ute, z_out) til y_in (inne, z_in),
    i hvilken retning som helst langs y."""
    town.HULLS.append([(x, y_out, z_out - 0.3) for x in (x0, x1)] + [(x, y_out, z_out) for x in (x0, x1)] + [(x, y_in, z_in + 0.02) for x in (x0, x1)] + [(x, y_in, z_out - 0.3) for x in (x0, x1)])


def jar(x, y, z, r, h, mat, n=7, neck=True):
    """Krukke: buker ut og har en hals og en kant."""
    if neck:
        prof = [(r * 0.75, 0.0), (r, h * 0.35), (r * 0.92, h * 0.75), (r * 0.6, h * 0.86), (r * 0.66, h), (0.0, h)]
    else:
        prof = [(r * 0.85, 0.0), (r, h * 0.5), (r * 0.95, h), (0.0, h)]
    return revolve("jar", prof, mat, n=n, at=(x, y, z))


def bottle(x, y, z, r, h, mat, n=6):
    """Flaske: rund kropp og smal hals med en kork."""
    revolve("bottle", [(r * 0.8, 0.0), (r, h * 0.15), (r, h * 0.55), (r * 0.35, h * 0.72), (r * 0.3, h * 0.92), (0.0, h * 0.92)], mat, n=n, at=(x, y, z))
    box("cork", (x - r * 0.25, y - r * 0.25, z + h * 0.9), (x + r * 0.25, y + r * 0.25, z + h), layer("leather", (0.8, 0.6, 0.45)), bevel=0)


def flask(x, y, z, r, mat, n=8):
    """Rundkolbe med lang hals."""
    revolve("flask", [(r * 0.4, 0.0), (r * 0.9, r * 0.35), (r, r * 0.9), (r * 0.7, r * 1.6), (r * 0.22, r * 1.95), (r * 0.2, r * 3.2), (0.0, r * 3.2)], mat, n=n, at=(x, y, z))


def herbs(x, y, z_top, ln, mat, rng, tie):
    """En urtebunt som henger opp ned fra en snor: tykk nede, bundet oppe."""
    stick("herb_string", (x, y, z_top), (x, y, z_top - 0.12), 0.015, 0.015, tie, bevel=0)
    r = rng.uniform(0.07, 0.1)
    b = revolve("herbs", [(0.0, -ln), (r * 0.8, -ln * 0.9), (r, -ln * 0.55), (r * 0.45, -0.06), (0.02, 0.0), (0.0, 0.0)], mat, n=5, at=(x, y, z_top - 0.12))
    turned(b, (x, y, z_top), Matrix.Rotation(rng.uniform(-0.12, 0.12), 4, "X") @ Matrix.Rotation(rng.uniform(-0.12, 0.12), 4, "Y"))


def books(x0, x1, y, z, depth, rng, along="x", lean_last=True):
    """En rad bøker mellom x0 og x1 (eller langs y), ulike høyder og farger."""
    covers = [layer("leather", c) for c in ((0.75, 0.35, 0.25), (0.45, 0.3, 0.2), (0.35, 0.45, 0.35), (0.3, 0.32, 0.5), (0.8, 0.65, 0.4))]
    u = x0
    while u < x1 - 0.05:
        t = rng.uniform(0.04, 0.09)
        h = rng.uniform(0.2, 0.32)
        m = rng.choice(covers)
        if along == "x":
            box("book", (u, y - depth / 2, z), (u + t, y + depth / 2, z + h), m, bevel=0)
        else:
            box("book", (y - depth / 2, u, z), (y + depth / 2, u + t, z + h), m, bevel=0)
        u += t + 0.005


def shelf_unit(p, x0, x1, y_wall, depth, z0, height, rows, along="x", out=-1):
    """En reol mot en vegg: to gavler, hyllebord og en bakplate. `along`
    er retningen langs veggen, `out` hvilken vei hyllene vender (±1 på
    tvers). Gir høyden til hver hylle."""
    zs = [z0 + 0.08 + (height - 0.12) * k / rows for k in range(rows + 1)]
    yi = y_wall + out * depth

    def b(name, a0, a1, c0, c1, za, zb, mat):
        if along == "x":
            box(name, (a0, min(c0, c1), za), (a1, max(c0, c1), zb), mat, bevel=0.01)
        else:
            box(name, (min(c0, c1), a0, za), (max(c0, c1), a1, zb), mat, bevel=0.01)

    for e in (x0, x1 - 0.05):
        b("shelf_side", e, e + 0.05, y_wall, yi, z0, z0 + height, p.timber)
    for z in zs:
        b("shelf_board", x0 + 0.05, x1 - 0.05, y_wall, yi, z - 0.035, z, p.floor)
    b("shelf_back", x0 + 0.05, x1 - 0.05, y_wall, y_wall + out * 0.02, z0, z0 + height, layer("cladding", (0.7, 0.6, 0.5)))
    if along == "x":
        collide(((x0 + x1) / 2, y_wall + out * depth / 2, z0 + height / 2), (x1 - x0, depth, height))
    else:
        collide((y_wall + out * depth / 2, (x0 + x1) / 2, z0 + height / 2), (depth, x1 - x0, height))
    return zs[:-1], yi


def hen(x, y, yaw, color, peck=False):
    """Ei høne: kropp, hale, hals med hode, kam, nebb og bein, enkle og
    litt overdrevne former."""
    feather = layer("wool", color)
    red = layer("plaster_red", (1.0, 0.55, 0.5))
    yellow = layer("plaster", (1.0, 0.75, 0.3))
    parts = []
    parts.append(box("hen_body", (-0.11, -0.16, 0.13), (0.11, 0.15, 0.33), feather, bevel=0.06))
    tail = box("hen_tail", (-0.06, 0.08, 0.26), (0.06, 0.2, 0.45), feather, bevel=0)
    turned(tail, (0, 0.12, 0.3), Matrix.Rotation(-0.5, 4, "X"))
    parts.append(tail)
    if peck:
        hz, hy = 0.12, -0.27
    else:
        hz, hy = 0.4, -0.17
    parts.append(box("hen_neck", (-0.055, -0.2, 0.25), (0.055, -0.08, 0.36 if not peck else 0.3), feather, bevel=0))
    parts.append(box("hen_head", (-0.06, hy - 0.07, hz - 0.05), (0.06, hy + 0.06, hz + 0.06), feather, bevel=0.025))
    parts.append(box("hen_comb", (-0.015, hy - 0.06, hz + 0.05), (0.015, hy + 0.05, hz + 0.11), red, bevel=0))
    parts.append(box("hen_wattle", (-0.015, hy - 0.06, hz - 0.1), (0.015, hy - 0.02, hz - 0.04), red, bevel=0))
    parts.append(box("hen_beak", (-0.02, hy - 0.12, hz - 0.015), (0.02, hy - 0.06, hz + 0.02), yellow, bevel=0))
    for sx in (-1, 1):
        parts.append(box("hen_leg", (sx * 0.05 - 0.012, -0.01, 0.0), (sx * 0.05 + 0.012, 0.015, 0.15), yellow, bevel=0))
        parts.append(box("hen_foot", (sx * 0.05 - 0.03, -0.06, 0.0), (sx * 0.05 + 0.03, 0.03, 0.015), yellow, bevel=0))
    for ob in parts:
        kit._place(ob, Vector((x, y, 0.0)), yaw)


def lantern(p, x, y, z_top, z):
    """Ei lykt i en kjetting fra en bjelke, med hornruter og lys."""
    stick("lamp_chain", (x, y, z_top), (x, y, z + 0.2), 0.015, 0.015, p.iron, bevel=0)
    box("lamp_top", (x - 0.13, y - 0.13, z + 0.12), (x + 0.13, y + 0.13, z + 0.2), p.iron, bevel=0.02)
    box("lamp_horn", (x - 0.1, y - 0.1, z - 0.14), (x + 0.1, y + 0.1, z + 0.12), plain("glow_horn", (0.3, 0.19, 0.08), 0.5), bevel=0.0)
    for sx in (-1, 1):
        for sy in (-1, 1):
            box("lamp_post", (x + sx * 0.1 - 0.012, y + sy * 0.1 - 0.012, z - 0.16), (x + sx * 0.1 + 0.012, y + sy * 0.1 + 0.012, z + 0.13), p.iron, bevel=0)
    box("lamp_base", (x - 0.13, y - 0.13, z - 0.2), (x + 0.13, y + 0.13, z - 0.14), p.iron, bevel=0.01)


# --- Smia -----------------------------------------------------------------------

# Mål (m). Murene er 0,55 tykke, fasaden mot torget 0,3.
FW = 9.0
FY0, FY1 = -3.6, 2.4  # forkant og bakkant av huset
FT = 0.55
FZ = 0.12  # gulvet (steinheller)
# Den malte ambolten (sheet2model fra assets/textures/props/prop2/anvil.jpeg).
ANVIL = "assets/models/props/anvil.glb"


def import_glb(path, at, yaw=0.0):
    """Ta inn en ferdig glb (y opp, fronten mot +z) og sett den med foten i
    `at` (Blender-akser), dreid `yaw` om z. Teksturen følger med i eksporten."""
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)
    new = [o for o in bpy.data.objects if o not in before]
    meshes = []
    for o in new:  # løs fra eventuelle tomme noder, med verdensmatrisen beholdt
        if o.type == "MESH":
            mw = o.matrix_world.copy()
            o.parent = None
            o.matrix_world = Matrix.Translation(at) @ Matrix.Rotation(yaw, 4, "Z") @ mw
            town.ensure_uv(o.data)
            town.PIECES.append(o)
            meshes.append(o)
    for o in new:
        if o.type != "MESH":
            bpy.data.objects.remove(o)
    return meshes
FE = 3.45  # takfoten
FPITCH = 42.0
FIX0, FIX1 = -FW / 2 + FT, FW / 2 - FT  # innsida av gavlmurene
FIY1 = FY1 - FT  # innsida av bakmuren
# Det åpne verkstedet mot torget, og de to stolpene midt i.
OPEN0, OPEN1 = -2.8, 3.6
OPEN_TOP = 2.75
POSTS = (-0.7, 1.5)
BACK_DOOR = 2.6
YARD1 = 5.3  # bakkant av bakgården
FENCE_X = FW / 2 + 0.05
# Essa langs bakmuren til venstre.
HX, HY = -2.15, FIY1 - 0.62


def forge_outside(p, rng):
    """Utsida: murene, fasaden med det åpne verkstedet, gulvet, taket, pipa,
    skiltet, lykta, essa, ambolten og kjølekaret, og bakgården."""
    # Gavlmurene og bakmuren av stein, fra fundamentet under bakken.
    left = kit.wall_frame((-FW / 2 + FT / 2, FY0), (-FW / 2 + FT / 2, FY1))
    right = kit.wall_frame((FW / 2 - FT / 2, FY0), (FW / 2 - FT / 2, FY1))
    back = kit.wall_frame((FIX0, FY1 - FT / 2), (FIX1, FY1 - FT / 2))
    win_l = [(1.0 - FY0 - 0.5, 1.0 - FY0 + 0.5, 1.2, 2.2)]
    win_r = [(-1.7 - FY0 - 0.5, -1.7 - FY0 + 0.5, 1.2, 2.2)]
    door_b = [(BACK_DOOR - FIX0 - 0.55, BACK_DOOR - FIX0 + 0.55, FZ, FZ + 2.15)]
    for name, (frame, L), holes in (("wall_left", left, win_l), ("wall_right", right, win_r), ("wall_back", back, door_b)):
        kit.slab_grid(name, frame, L, -0.2, FE, FT, holes, p.wall_stone, bevel=0.03)
        kit.wall_colliders(frame, L, -0.2, FE, FT, holes)
    window(p, left[0], 1.0 - FY0, 1.7, 1.0, 1.0, 1, thick=FT, rng=rng, inside=True)
    window(p, right[0], -1.7 - FY0, 1.7, 1.0, 1.0, -1, thick=FT, rng=rng, inside=True)
    door(p, back[0], BACK_DOOR - FIX0, FZ, 1.1, 2.15, 1, thick=FT)
    # Murkronen: en svill av tømmer oppå murene, under takfoten.
    for a, b in (((-FW / 2 + FT / 2, FY0 - 0.05), (-FW / 2 + FT / 2, FY1 + 0.05)), ((FW / 2 - FT / 2, FY0 - 0.05), (FW / 2 - FT / 2, FY1 + 0.05)), ((-FW / 2, FY1 - FT / 2), (FW / 2, FY1 - FT / 2))):
        stick("wall_plate", (a[0], a[1], FE + 0.1), (b[0], b[1], FE + 0.1), 0.5, 0.22, p.timber, ("timber", rng.randrange(4)), seed=a[0])
    # Store hjørnesteiner, annenhver lang langs gavlen og langs fasaden.
    for sx in (-1, 1):
        for sy, yy in ((-1, FY0), (1, FY1)):
            for k in range(3):
                z = 0.05 + k * 1.1
                ly, lx = (0.75, 0.45) if k % 2 == 0 else (0.45, 0.75)
                xo, xi = sx * (FW / 2 + 0.05), sx * (FW / 2 - lx)
                yo, yi = yy + sy * 0.05, yy - sy * ly
                box("quoin", (min(xo, xi), min(yo, yi), z), (max(xo, xi), max(yo, yi), z + 0.45), p.stone, bevel=0.05)

    # Fasaden mot torget: bindingsverk med det store åpne verkstedet og et
    # lite vindu til venstre.
    fy = FY0 + 0.15
    opens = [((OPEN0 + OPEN1) / 2 - FIX0, FZ, OPEN_TOP, OPEN1 - OPEN0), (-3.45 - FIX0, 1.25, 2.15, 0.7)]
    fr, _ = half_timbered(p, "wall_front", (FIX0, fy), (FIX1, fy), FZ, FE, -1, opens, rng)
    window(p, fr, -3.45 - FIX0, 1.7, 0.7, 0.9, -1, rng=rng, inside=True, in_frame=True)
    # Overliggeren over åpningen: en tykk, litt skjev bjelke på to stolper
    # med knekter.
    stick("lintel", (OPEN0 - 0.35, FY0 - 0.08, OPEN_TOP + 0.17), (OPEN1 + 0.35, FY0 - 0.08, OPEN_TOP + 0.22), 0.36, 0.38, p.timber, ("timber", 2), bevel=0.05)
    for k, x in enumerate(POSTS):
        stick("post", (x, FY0 + 0.05, FZ), (x + rng.uniform(-0.05, 0.05), FY0 + 0.05, OPEN_TOP), 0.32, 0.32, p.timber, ("timber", k), bevel=0.05)
        box("post_stone", (x - 0.26, FY0 - 0.21, -0.1), (x + 0.26, FY0 + 0.31, FZ + 0.06), p.stone, bevel=0.05)
        collide((x, FY0 + 0.05, 1.4), (0.36, 0.36, 2.8))
        for sx in (-1, 1):
            stick("knee", (x, FY0 + 0.0, OPEN_TOP - 0.6), (x + sx * 0.55, FY0 + 0.0, OPEN_TOP - 0.02), 0.18, 0.2, p.timber, ("timber", 3), bevel=0.03)
    for x in (OPEN0, OPEN1):
        for sx in (-1, 1):
            if (x == OPEN0 and sx < 0) or (x == OPEN1 and sx > 0):
                continue
            stick("knee", (x + sx * 0.12, FY0 + 0.0, OPEN_TOP - 0.55), (x + sx * 0.55, FY0 + 0.0, OPEN_TOP - 0.02), 0.18, 0.2, p.timber, ("timber", 1), bevel=0.03)

    # Gulvet: steinheller, og rampene over kanten foran og ved bakdøra.
    box("floor", (FIX0, FY0 + 0.02, -0.2), (FIX1, FIY1, FZ), p.flag, bevel=0.0)
    collide((0.0, (FY0 + FIY1) / 2, FZ - 0.15), (FIX1 - FIX0, FIY1 - FY0, 0.3))
    box("sill_stone", (OPEN0 + 0.05, FY0 - 0.35, -0.15), (OPEN1 - 0.05, FY0 + 0.02, FZ - 0.02), p.stone, bevel=0.04)
    ramp_to(OPEN0, OPEN1, FY0 - 0.7, FY0 + 0.1, 0.0, FZ)
    box("back_step", (BACK_DOOR - 0.65, FY1 - 0.02, -0.15), (BACK_DOOR + 0.65, FY1 + 0.45, FZ - 0.03), p.stone, bevel=0.04)
    ramp_to(BACK_DOOR - 0.6, BACK_DOOR + 0.6, FY1 + 0.75, FY1 - FT - 0.1, 0.0, FZ)

    # Taket: tegl, med mønet langs fasaden, og gavlene murt av samme stein
    # som veggene helt opp under mønet (eieren: «stein hele veien opp»).
    ridge = kit.gable_roof(p, -FW / 2, FW / 2, FY0, FY1, FE + 0.28, FPITCH, overhang=0.85, gable_over=0.55, sag=0.28)
    for x, out in ((-FW / 2 + FT / 2, -1), (FW / 2 - FT / 2, 1)):
        kit.gable_wall(p, x, FY0, FY1, FE, ridge, FPITCH, out, rng, thick=FT, mat=p.wall_stone, timber=False)
    # Bindbjelkene på tvers inne (synes gjennom åpningen).
    for x in (-3.45, 0.4, 3.2):
        stick("tie_beam", (x, FY0 + 0.2, FE - 0.05), (x, FIY1 + 0.25, FE - 0.05), 0.24, 0.26, p.timber, ("timber", rng.randrange(4)), seed=x)
    # Essa: murt av stein, med glør og en hette som går over i pipa.
    yc = (FY0 + FY1) / 2
    tan = math.tan(math.radians(FPITCH))
    roof_at = lambda y: FE + 0.28 + ((FY1 - FY0) / 2 - abs(y - yc)) * tan
    hood_top = roof_at(HY + 0.2) - 0.45
    box("hearth", (HX - 0.95, HY - 0.62, -0.1), (HX + 0.95, FIY1 + 0.02, 0.85), p.stone, bevel=0.06)
    box("hearth_rim", (HX - 1.0, HY - 0.68, 0.85), (HX + 1.0, FIY1 + 0.02, 0.97), p.soot, bevel=0.03)
    box("coal_bed", (HX - 0.6, HY - 0.42, 0.9), (HX + 0.6, HY + 0.38, 0.99), layer("rock", (0.18, 0.16, 0.15)), bevel=0.02)
    glow = plain("fire_coals", (1.0, 0.42, 0.12), 0.4)
    for k in range(9):
        cx = HX + rng.uniform(-0.38, 0.38)
        cy = HY + rng.uniform(-0.25, 0.22)
        box("ember", (cx - 0.09, cy - 0.08, 0.97), (cx + 0.09, cy + 0.08, 1.02 + rng.uniform(0, 0.04)), glow, bevel=0)
    # Hetta: smalner oppover i tre trinn, med en jernlist nederst.
    for k in range(3):
        t0, t1 = k / 3, (k + 1) / 3
        z0 = 1.85 + (hood_top - 1.85) * t0
        z1 = 1.85 + (hood_top - 1.85) * t1
        w = 1.9 - 0.95 * t0
        d = 1.25 - 0.55 * t0
        box("hood", (HX - w / 2, FIY1 - d, z0), (HX + w / 2, FIY1, z1 + 0.02), p.soot if k else p.stone, bevel=0.05)
    stick("hood_band", (HX - 0.98, FIY1 - 1.27, 1.88), (HX + 0.98, FIY1 - 1.27, 1.88), 0.06, 0.1, p.iron, bevel=0.0)
    kit.chimney(p, HX, FIY1 - 0.35, hood_top - 0.3, ridge + 0.9, w=1.0, smoke=1.0, lean=-4.0)
    fire(p, HX, HY - 0.02, 0.96, size=0.55, light_it=False)
    light((HX, HY - 0.3, 1.3), (1.0, 0.45, 0.15), 3.2, 7.5)
    collide((HX, (HY - 0.68 + FIY1) / 2, 0.55), (2.0, FIY1 - HY + 0.68, 1.1))
    # Blåsebelgen mellom essa og gavlmuren: to bord med lær mellom.
    bx, by = FIX0 + 0.42, HY + 0.1
    box("bellows_bottom", (bx - 0.32, by - 0.55, 0.62), (bx + 0.32, by + 0.55, 0.68), p.floor, bevel=0.01)
    stick("bellows_top", (bx, by - 0.55, 0.74), (bx, by + 0.55, 0.98), 0.64, 0.06, p.floor, bevel=0.01)
    box("bellows_leather", (bx - 0.28, by - 0.4, 0.68), (bx + 0.28, by + 0.5, 0.95), p.door_dark, bevel=0.1)
    stick("bellows_nozzle", (bx + 0.1, by - 0.5, 0.75), (HX - 0.85, HY - 0.15, 0.9), 0.08, 0.08, p.iron, bevel=0.02)
    for sy in (-1, 1):
        stick("bellows_leg", (bx, by + sy * 0.4, FZ), (bx, by + sy * 0.4, 0.62), 0.1, 0.1, p.timber, ("timber", 0), bevel=0.02)
    stick("bellows_pole", (bx, by + 0.5, 1.02), (bx, by + 0.5, FE - 0.1), 0.06, 0.06, p.timber, ("timber", 2), bevel=0.0)
    collide((bx, by, 0.5), (0.7, 1.2, 1.0))
    # Ambolten på en kubbe: den malte modellen fra eierens ark (sheet2model,
    # `sheet2model/examples/fristad/anvil.json`) når den finnes, ellers
    # byggesettets.
    ax, ay = -1.2, -1.15
    dark = layer("iron", (0.45, 0.43, 0.42), 0.4)
    if os.path.exists(ANVIL):
        import_glb(ANVIL, (ax, ay, FZ))
    else:
        revolve("stump", [(0.3, 0.0), (0.34, 0.05), (0.31, 0.5), (0.0, 0.5)], layer("logs"), n=8, at=(ax, ay, FZ - 0.02))
        revolve("stump_top", [(0.29, 0.0), (0.0, 0.0)], p.floor, n=8, at=(ax, ay, FZ + 0.485))
        box("anvil_foot", (ax - 0.2, ay - 0.15, FZ + 0.48), (ax + 0.2, ay + 0.15, FZ + 0.6), dark, bevel=0.025)
        box("anvil_waist", (ax - 0.1, ay - 0.08, FZ + 0.6), (ax + 0.1, ay + 0.08, FZ + 0.74), dark, bevel=0.02)
        box("anvil_face", (ax - 0.3, ay - 0.1, FZ + 0.74), (ax + 0.24, ay + 0.1, FZ + 0.88), dark, bevel=0.025)
        horn = revolve("anvil_horn", [(0.075, 0.0), (0.04, 0.17), (0.0, 0.3)], dark, n=6, at=(0, 0, 0))
        turned(horn, (0, 0, 0), Matrix.Translation((ax + 0.22, ay, FZ + 0.82)) @ Matrix.Rotation(math.pi / 2, 4, "Y"))
    collide((ax, ay, FZ + 0.45), (0.75, 0.7, 0.9))
    # Kjølekaret langs gavlmuren: et trau av planker med vann.
    qx, qy0, qy1 = FIX0 + 0.38, -2.9, -1.6
    box("trough_bottom", (qx - 0.33, qy0, 0.18), (qx + 0.33, qy1, 0.26), p.floor, bevel=0.01)
    for sx in (-1, 1):
        box("trough_side", (qx + sx * 0.33 - 0.04, qy0, 0.18), (qx + sx * 0.33 + 0.04, qy1, 0.72), p.boards, bevel=0.015)
    for yy in (qy0, qy1):
        box("trough_end", (qx - 0.37, yy - 0.05, 0.18), (qx + 0.37, yy + 0.05, 0.76), p.boards, bevel=0.015)
        box("trough_leg", (qx - 0.4, yy + (0.14 if yy == qy0 else -0.26), FZ - 0.02), (qx + 0.4, yy + (0.26 if yy == qy0 else -0.14), 0.19), p.timber, bevel=0.02)
    box("trough_water", (qx - 0.29, qy0 + 0.05, 0.6), (qx + 0.29, qy1 - 0.05, 0.62), plain("water_trough", (0.08, 0.1, 0.09), 0.08), bevel=0)
    stick("trough_band", (qx - 0.38, qy0 + 0.3, 0.6), (qx + 0.38, qy0 + 0.3, 0.6), 0.04, 0.06, p.iron, bevel=0)
    collide((qx, (qy0 + qy1) / 2, 0.4), (0.8, qy1 - qy0 + 0.1, 0.8))

    # Skiltet: en smijernsarm fra hjørnestolpen med en ambolt i silhuett.
    sx0, sy0, sz = OPEN1 + 0.45, FY0 - 0.2, 2.55
    stick("sign_arm", (sx0, sy0, sz), (sx0, sy0 - 1.15, sz), 0.07, 0.07, p.iron, bevel=0.01)
    stick("sign_strut", (sx0, sy0, sz - 0.55), (sx0, sy0 - 0.75, sz - 0.02), 0.05, 0.05, p.iron, bevel=0)
    stick("sign_curl", (sx0, sy0 - 1.15, sz), (sx0, sy0 - 1.22, sz + 0.08), 0.05, 0.05, p.iron, bevel=0)
    for yy in (sy0 - 0.4, sy0 - 1.0):
        stick("sign_chain", (sx0, yy, sz), (sx0, yy, sz - 0.25), 0.02, 0.02, p.iron, bevel=0)
    for dx in (-0.025, 0.025):
        box("sign_anvil", (sx0 + dx - 0.03, sy0 - 1.2, sz - 0.45), (sx0 + dx + 0.03, sy0 - 0.25, sz - 0.25), dark, bevel=0.015)
    box("sign_anvil_waist", (sx0 - 0.03, sy0 - 0.85, sz - 0.65), (sx0 + 0.03, sy0 - 0.55, sz - 0.45), dark, bevel=0.01)
    box("sign_anvil_foot", (sx0 - 0.03, sy0 - 1.0, sz - 0.78), (sx0 + 0.03, sy0 - 0.4, sz - 0.65), dark, bevel=0.01)
    # Lykta ved verkstedet.
    town.wall_lantern(p, -2.4, FY0, 2.45, -1)
    # Ei benk og et våpenstativ ute ved fasaden.
    kit.bench(p, -3.4, FY0 - 0.55, 0.0, 1.5)

    forge_yard(p, rng)
    return ridge


def forge_yard(p, rng):
    """Bakgården: flettverksgjerde med stolper, kullbinge, vedstabel under
    et lite skur, slipestein og høner."""
    wattle = layer("wattle")
    corners = [(-FENCE_X, FY1 - 0.05), (-FENCE_X, YARD1), (FENCE_X, YARD1), (FENCE_X, FY1 - 0.05)]
    for (ax_, ay_), (bx_, by_) in zip(corners, corners[1:]):
        L = math.hypot(bx_ - ax_, by_ - ay_)
        n = max(1, round(L / 1.8))
        for k in range(n):
            t0, t1 = k / n, (k + 1) / n
            p0 = (ax_ + (bx_ - ax_) * t0, ay_ + (by_ - ay_) * t0)
            p1 = (ax_ + (bx_ - ax_) * t1, ay_ + (by_ - ay_) * t1)
            h = 1.15 + rng.uniform(-0.06, 0.06)
            if abs(bx_ - ax_) > abs(by_ - ay_):
                box("fence_panel", (min(p0[0], p1[0]), p0[1] - 0.05, 0.1), (max(p0[0], p1[0]), p0[1] + 0.05, h), wattle, bevel=0.02)
            else:
                box("fence_panel", (p0[0] - 0.05, min(p0[1], p1[1]), 0.1), (p0[0] + 0.05, max(p0[1], p1[1]), h), wattle, bevel=0.02)
            stick("fence_post", (p1[0], p1[1], -0.15), (p1[0] + rng.uniform(-0.04, 0.04), p1[1], h + 0.2), 0.12, 0.12, p.timber, ("timber", rng.randrange(4)), bevel=0.03)
        collide(((ax_ + bx_) / 2, (ay_ + by_) / 2, 0.65), (abs(bx_ - ax_) + 0.14, abs(by_ - ay_) + 0.14, 1.3))
    stick("fence_post", (corners[0][0], corners[0][1] + 0.1, -0.15), (corners[0][0], corners[0][1] + 0.1, 1.35), 0.12, 0.12, p.timber, ("timber", 0), bevel=0.03)

    # Kullbingen mot bakmuren til venstre: tre plankevegger og en haug kull.
    kx0, kx1, ky0, ky1 = -4.2, -2.4, FY1 + 0.05, FY1 + 1.5
    for lo, hi in (((kx0, ky0, 0.0), (kx0 + 0.08, ky1, 0.9)), ((kx1 - 0.08, ky0, 0.0), (kx1, ky1, 0.9)), ((kx0, ky0, 0.0), (kx1, ky0 + 0.08, 1.0))):
        box("coal_bin", lo, hi, p.boards, bevel=0.02)
    for x in (kx0 + 0.04, kx1 - 0.04):
        stick("coal_bin_post", (x, ky1, -0.1), (x, ky1, 0.98), 0.1, 0.1, p.timber, ("timber", 2), bevel=0.02)
    box("coal_bin_front", (kx0, ky1 - 0.04, 0.0), (kx1, ky1 + 0.04, 0.35), p.boards, bevel=0.015)
    coal = layer("rock", (0.16, 0.15, 0.15), 0.6)
    revolve("coal_heap", [(0.82, 0.0), (0.7, 0.35), (0.4, 0.62), (0.0, 0.72)], coal, n=8, at=((kx0 + kx1) / 2, (ky0 + ky1) / 2 + 0.05, 0.0))
    for k in range(5):
        cx = rng.uniform(kx0 + 0.3, kx1 - 0.3)
        box("coal_lump", (cx - 0.09, ky1 + 0.08 + k * 0.06, 0.0), (cx + 0.09, ky1 + 0.2 + k * 0.06, 0.1), coal, bevel=0)
    stick("coal_shovel", (kx1 - 0.3, ky1 - 0.1, 0.4), (kx1 + 0.15, ky1 + 0.35, 1.3), 0.04, 0.04, p.timber, ("timber", 0), bevel=0)
    box("coal_shovel_blade", (kx1 - 0.45, ky1 - 0.3, 0.05), (kx1 - 0.15, ky1 + 0.0, 0.42), p.iron, bevel=0.0)
    collide(((kx0 + kx1) / 2, (ky0 + ky1) / 2, 0.5), (kx1 - kx0, ky1 - ky0, 1.0))

    # Vedstabelen langs gjerdet til høyre, under et lite skråtak.
    wx, wy0, wy1 = FENCE_X - 0.55, FY1 + 0.7, YARD1 - 0.5
    # Stabelen: kubbene ligger langs gjerdet, så laftelaget viser sidene
    # av dem; noen løse kubber på toppen og foran.
    # Selve stabelen er den frittstående modellen `woodpile`, satt på
    # bakken av byen (`town.prop`), så den ikke svever der tomta skrår.
    town.prop("woodpile", wx, (wy0 + wy1) / 2, math.pi / 2)
    for yy in (wy0 - 0.15, wy1 + 0.15):
        stick("shed_post", (wx - 0.45, yy, -0.1), (wx - 0.45, yy, 1.55), 0.12, 0.12, p.timber, ("timber", 1), bevel=0.02)
        stick("shed_post", (wx + 0.45, yy, -0.1), (wx + 0.45, yy, 1.8), 0.12, 0.12, p.timber, ("timber", 3), bevel=0.02)
        stick("shed_beam", (wx - 0.55, yy, 1.55), (wx + 0.55, yy, 1.8), 0.1, 0.12, p.timber, ("timber", 2), bevel=0.02)
    roof = box("shed_roof", (-0.7, wy0 - 0.45, -0.05), (0.7, wy1 + 0.45, 0.05), layer("shingles"), bevel=0.02)
    turned(roof, (0, 0, 0), Matrix.Translation((wx, 0, 1.75)) @ Matrix.Rotation(-math.atan2(0.25, 1.1), 4, "Y"))
    collide((wx, (wy0 + wy1) / 2, 0.7), (1.0, wy1 - wy0 + 0.4, 1.4))
    # Hoggestabben med øksa.
    cx, cy = 1.6, YARD1 - 1.4
    revolve("chop_block", [(0.3, 0.0), (0.33, 0.05), (0.3, 0.55), (0.0, 0.56)], layer("logs"), n=8, at=(cx, cy, -0.02))
    stick("axe_handle", (cx + 0.05, cy, 0.56), (cx + 0.5, cy - 0.3, 0.95), 0.045, 0.045, p.timber, ("timber", 0), bevel=0)
    box("axe_head", (cx - 0.08, cy - 0.04, 0.5), (cx + 0.1, cy + 0.04, 0.68), p.iron, bevel=0.01)
    for k in range(4):
        a = rng.uniform(0, 6.28)
        stick("chip", (cx + math.cos(a) * 0.5, cy + math.sin(a) * 0.5, 0.03), (cx + math.cos(a) * 0.75, cy + math.sin(a) * 0.55, 0.03), 0.09, 0.05, layer("logs"), bevel=0)
    collide((cx, cy, 0.3), (0.65, 0.65, 0.6))

    # Slipesteinen i en ramme med sveiv og et vanntrau under.
    gx, gy = -1.0, YARD1 - 1.3
    wheel = revolve("grindstone", [(0.0, -0.08), (0.36, -0.08), (0.4, -0.04), (0.4, 0.04), (0.36, 0.08), (0.0, 0.08)], layer("stone", (0.85, 0.8, 0.72)), n=14, at=(0, 0, 0))
    turned(wheel, (0, 0, 0), Matrix.Translation((gx, gy, 0.72)) @ Matrix.Rotation(math.pi / 2, 4, "Y"))
    for sx in (-1, 1):
        for sy in (-1, 1):
            stick("grind_leg", (gx + sx * 0.16, gy + sy * 0.45, 0.0), (gx + sx * 0.16, gy + sy * 0.12, 0.72), 0.09, 0.09, p.timber, ("timber", 1), bevel=0.02)
    box("grind_trough", (gx - 0.14, gy - 0.32, 0.18), (gx + 0.14, gy + 0.32, 0.4), p.boards, bevel=0.02)
    stick("grind_axle", (gx - 0.32, gy, 0.72), (gx + 0.3, gy, 0.72), 0.04, 0.04, p.iron, bevel=0)
    stick("grind_crank", (gx - 0.32, gy, 0.72), (gx - 0.32, gy - 0.05, 0.48), 0.035, 0.035, p.iron, bevel=0)
    stick("grind_handle", (gx - 0.32, gy - 0.05, 0.48), (gx - 0.5, gy - 0.05, 0.48), 0.04, 0.04, p.timber, ("timber", 0), bevel=0)
    collide((gx, gy, 0.5), (0.6, 1.0, 1.0))

    # Hønene: noen går og noen hakker.
    colors = [(0.95, 0.92, 0.85), (0.75, 0.42, 0.22), (0.55, 0.33, 0.2), (0.95, 0.92, 0.85), (0.3, 0.25, 0.22)]
    spots = [(-0.2, FY1 + 1.3, 0.4, False), (0.6, FY1 + 2.0, 2.2, True), (-2.4, YARD1 - 0.9, -0.8, True), (2.6, FY1 + 2.6, 3.6, False), (-1.8, FY1 + 2.3, 1.2, False)]
    for (x, y, yaw, peck), c in zip(spots, colors):
        hen(x, y, yaw, c, peck)
    # Vannfatet til hønene.
    revolve("hen_dish", [(0.2, 0.0), (0.22, 0.08), (0.18, 0.08), (0.17, 0.03), (0.0, 0.03)], layer("plaster_red", (0.7, 0.5, 0.4)), n=8, at=(0.2, FY1 + 2.6, 0.0))


def forge_inside(p, rng):
    """Innsida av Smia: verktøyveggen, arbeidsbenken, våpenstativet, det
    som ligger rundt essa og ambolten, og lykta."""
    dark = layer("iron", (0.45, 0.43, 0.42), 0.4)
    bright = layer("iron", (1.0, 0.97, 0.92), 0.4)
    ax, ay = -1.2, -1.15
    # Hammer og tang på ambolten, og et glødende emne.
    stick("hammer_handle", (ax - 0.05, ay - 0.04, FZ + 0.92), (ax - 0.45, ay - 0.3, FZ + 0.92), 0.035, 0.035, p.timber, ("timber", 0), bevel=0)
    box("hammer_head", (ax - 0.09, ay - 0.1, FZ + 0.88), (ax + 0.0, ay + 0.04, FZ + 0.96), dark, bevel=0.01)
    for s in (-1, 1):
        stick("tongs", (ax + 0.05, ay + 0.04 + s * 0.015, FZ + 0.9), (ax + 0.52, ay + 0.08 + s * 0.04, FZ + 0.9), 0.02, 0.02, dark, bevel=0)
    box("hot_iron", (ax - 0.05, ay - 0.02, FZ + 0.88), (ax + 0.12, ay + 0.03, FZ + 0.905), plain("fire_iron", (1.0, 0.5, 0.15), 0.4), bevel=0)
    # Ildgreip og raker i essa.
    for k in range(3):
        stick("poker", (HX + 0.3 + k * 0.12, HY - 0.15, 0.98), (HX + 0.75 + k * 0.1, HY - 0.95, 1.12), 0.025, 0.025, dark, bevel=0)
    # Kullbøtta foran essa.
    with piece("coal_pail", (HX + 1.2, HY - 0.75, FZ)):
        revolve("coal_pail", [(0.18, 0.0), (0.22, 0.32), (0.0, 0.32)], p.boards, n=8, at=(HX + 1.2, HY - 0.75, FZ))
        revolve("coal_pail_top", [(0.19, 0.0), (0.12, 0.06), (0.0, 0.08)], layer("rock", (0.16, 0.15, 0.15)), n=8, at=(HX + 1.2, HY - 0.75, FZ + 0.32))
        collide((HX + 1.2, HY - 0.75, FZ + 0.2), (0.45, 0.45, 0.4))

    # Verktøyveggen på bakmuren: en list med tenger, hammere og filer, og
    # hestesko på spiker.
    wy = FIY1
    with piece("tool_wall", (0.3, wy, FZ)):
        stick("tool_rail", (-1.0, wy - 0.05, 1.95), (1.6, wy - 0.05, 1.95), 0.08, 0.1, p.timber, ("timber", 2), bevel=0.02)
        stick("tool_rail", (-1.0, wy - 0.05, 1.25), (1.6, wy - 0.05, 1.25), 0.08, 0.1, p.timber, ("timber", 1), bevel=0.02)
        x = -0.85
        k = 0
        while x < 1.45:
            kind = k % 4
            if kind == 0:  # tang
                for s in (-1, 1):
                    stick("tongs", (x, wy - 0.12, 1.92), (x + s * 0.05, wy - 0.12, 1.32), 0.02, 0.02, dark, bevel=0)
            elif kind == 1:  # hammer
                stick("tool", (x, wy - 0.12, 1.92), (x, wy - 0.12, 1.42), 0.035, 0.035, p.timber, ("timber", 0), bevel=0)
                box("tool_head", (x - 0.08, wy - 0.16, 1.38), (x + 0.08, wy - 0.08, 1.46), dark, bevel=0.01)
            elif kind == 2:  # fil og meisel
                stick("file", (x, wy - 0.11, 1.9), (x, wy - 0.11, 1.5), 0.03, 0.012, bright, bevel=0)
            else:  # sag
                box("saw", (x - 0.03, wy - 0.12, 1.4), (x + 0.03, wy - 0.1, 1.9), bright, bevel=0)
            x += 0.22
            k += 1
        for k in range(7):
            hx_ = -0.9 + k * 0.36
            z = 1.05 - (k % 2) * 0.12
            horse = revolve("horseshoe", [(0.07, -0.01), (0.07, 0.01), (0.0, 0.01)], bright, n=6, at=(0, 0, 0))
            turned(horse, (0, 0, 0), Matrix.Translation((hx_, wy - 0.03, z)) @ Matrix.Rotation(math.pi / 2, 4, "X"))
    # Jernstenger lent mot muren.
    with piece("iron_bars_leaning", (1.84, wy - 0.15, FZ)):
        for k in range(5):
            stick("iron_bar", (1.7 + k * 0.07, wy - 0.25, FZ), (1.75 + k * 0.07, wy - 0.06, 1.5), 0.035, 0.035, dark, bevel=0)

    # Arbeidsbenken langs høyre gavlmur, med skrustikke og en klinge.
    bx0, bx1, by0, by1 = FIX1 - 0.75, FIX1, -1.2, 0.8
    with piece("forge_workbench", ((bx0 + bx1) / 2, (by0 + by1) / 2, FZ)):
        box("bench_top", (bx0, by0, 0.88), (bx1, by1, 0.98), p.floor, bevel=0.02)
        for xx in (bx0 + 0.08, bx1 - 0.12):
            for yy in (by0 + 0.1, by1 - 0.1):
                stick("bench_leg", (xx, yy, FZ), (xx, yy, 0.88), 0.1, 0.1, p.timber, ("timber", 1), bevel=0.02)
        box("bench_shelf", (bx0 + 0.04, by0 + 0.05, 0.3), (bx1 - 0.04, by1 - 0.05, 0.35), p.floor, bevel=0.01)
        box("vise", (bx0 + 0.02, by0 + 0.2, 0.98), (bx0 + 0.24, by0 + 0.45, 1.18), dark, bevel=0.02)
        stick("vise_bar", (bx0 - 0.04, by0 + 0.12, 1.1), (bx0 - 0.04, by0 + 0.53, 1.1), 0.03, 0.03, dark, bevel=0)
        stick("blade", (bx0 + 0.3, -0.3, 0.995), (bx0 + 0.35, 0.5, 0.995), 0.06, 0.01, bright, bevel=0)
        box("ingot", (bx0 + 0.3, by1 - 0.35, 0.98), (bx0 + 0.55, by1 - 0.15, 1.03), dark, bevel=0.01)
        for k in range(3):
            box("sack", (bx0 + 0.1, by0 + 0.2 + k * 0.5, 0.35), (bx1 - 0.1, by0 + 0.6 + k * 0.5, 0.6), layer("linen", (0.75, 0.68, 0.55)), bevel=0.08)
        collide(((bx0 + bx1) / 2, (by0 + by1) / 2, 0.5), (bx1 - bx0, by1 - by0, 1.0))
    # Hylle over benken med hjelmer og skjoldbuler.
    with piece("helmet_shelf", (FIX1 - 0.16, (by0 + by1) / 2, FZ)):
        box("wall_shelf", (FIX1 - 0.32, by0, 1.75), (FIX1, by1, 1.8), p.floor, bevel=0.01)
        for k in range(3):
            y = by0 + 0.35 + k * 0.62
            revolve("helmet", [(0.15, 0.0), (0.15, 0.12), (0.11, 0.22), (0.0, 0.26)], bright if k != 1 else dark, n=8, at=(FIX1 - 0.17, y, 1.8))
            box("helmet_nasal", (FIX1 - 0.34, y - 0.015, 1.83), (FIX1 - 0.3, y + 0.015, 1.98), bright, bevel=0)

    # Våpenstativet foran til høyre: sverd og økser til salgs.
    rx, ry0, ry1 = FIX1 - 0.4, -3.1, -1.8
    with piece("forge_weapon_rack", (rx, (ry0 + ry1) / 2, FZ)):
        for yy in (ry0, ry1):
            stick("rack_post", (rx, yy, FZ), (rx, yy, 1.35), 0.09, 0.09, p.timber, ("timber", 2), bevel=0.02)
        stick("rack_rail", (rx, ry0 - 0.05, 1.2), (rx, ry1 + 0.05, 1.2), 0.08, 0.1, p.timber, ("timber", 3), bevel=0.02)
        stick("rack_foot", (rx, ry0 - 0.05, FZ + 0.12), (rx, ry1 + 0.05, FZ + 0.12), 0.25, 0.08, p.timber, ("timber", 1), bevel=0.02)
        for k in range(4):
            y = ry0 + 0.2 + k * 0.28
            if k % 2 == 0:
                stick("sword_blade", (rx, y, FZ + 0.18), (rx, y, 1.05), 0.012, 0.06, bright, bevel=0)
                box("sword_guard", (rx - 0.02, y - 0.12, 1.05), (rx + 0.02, y + 0.12, 1.09), dark, bevel=0)
                stick("sword_grip", (rx, y, 1.09), (rx, y, 1.3), 0.035, 0.035, p.door_dark, bevel=0)
                box("sword_pommel", (rx - 0.035, y - 0.035, 1.3), (rx + 0.035, y + 0.035, 1.36), dark, bevel=0.01)
            else:
                stick("axe_haft", (rx, y, FZ + 0.18), (rx, y, 1.4), 0.04, 0.04, p.timber, ("timber", 0), bevel=0)
                box("axe_head", (rx - 0.02, y - 0.02, 1.2), (rx + 0.02, y + 0.2, 1.38), bright, bevel=0.005)
        collide((rx, (ry0 + ry1) / 2, 0.7), (0.4, ry1 - ry0 + 0.2, 1.4))
    # Tønne med jernstenger og ei kasse i hjørnet foran til venstre.
    with piece("barrel_bars", (FIX0 + 0.4, -0.95, FZ)):
        barrel(p, FIX0 + 0.4, -0.95, FZ, r=0.3, h=0.8)
        for k in range(4):
            stick("iron_bar", (FIX0 + 0.33 + k * 0.05, -0.95, FZ + 0.4), (FIX0 + 0.31 + k * 0.05, -0.9, FZ + 1.4), 0.03, 0.03, dark, bevel=0)
        collide((FIX0 + 0.4, -0.95, FZ + 0.4), (0.65, 0.65, 0.8))
    with piece("forge_crate", (FIX0 + 0.4, 0.275, FZ)):
        box("crate", (FIX0 + 0.1, 0.0, FZ), (FIX0 + 0.7, 0.55, FZ + 0.55), p.boards, bevel=0.03)
        collide((FIX0 + 0.4, 0.27, FZ + 0.27), (0.6, 0.55, 0.55))
    # Lykta i bindbjelken midt i rommet, og lys over arbeidsbenken.
    with piece("chain_lantern_forge", (0.4, -0.6, 2.15)):
        lantern(p, 0.4, -0.6, FE - 0.18, 2.35)
        light((0.4, -0.6, 2.3), (1.0, 0.66, 0.36), 1.3, 6.5)
    light((FIX1 - 0.6, -0.2, 2.0), (1.0, 0.7, 0.42), 0.8, 4.0)


# --- Apoteket -------------------------------------------------------------------

AW, AD, AT = 8.0, 7.0, 0.3
AF0 = 0.7  # gulvet i første etasje
AC0 = 3.22  # undersida av bjelkelaget
AF1 = 3.44  # gulvet oppe
AE = AF1 + 2.2  # takfoten (overetasjen under takskjegget)
AJ = 0.4  # utkragingen mot torget
APITCH = 50.0
AX0, AX1 = -AW / 2 + AT / 2, AW / 2 - AT / 2
AY0, AY1 = -AD / 2 + AT / 2, AD / 2 - AT / 2
AUY0 = AY0 - AJ
AJETTY_T = 0.06  # gulvbordene over utkragingen ligger oppå bjelkeendene
AIX0, AIX1, AIY0, AIY1 = AX0 + AT / 2, AX1 - AT / 2, AY0 + AT / 2, AY1 - AT / 2
ADOOR = 1.9
AHATCH = -1.85
# Trappa opp langs østveggen: fra y −2,3 (nederst) til 1,6 (oppe), og
# trapphullet i bjelkelaget.
# Trappa langs østveggen, etter reglene i kit.py (en meter fritt gulv
# ved foten, trapphullet fra der hoderommet krever det).
SX0, SX1, SY0 = 2.55, 3.65, AIY0 + kit.LANDING + kit.WALL_PAD
SY1 = SY0 + kit.stair_run(AF0, AF1)
AUNDER = AC0 - 0.24  # undersida av bjelkene over trappa
HX0, HX1, HY1 = 2.42, AIX1, SY1
HY0 = kit.stair_well(SY0, SY1, AF0, AF1, kit.stair_steps(AF0, AF1), AUNDER)
# Skilleveggen mot laboratoriet, med døra.
PY = 0.4
PDOOR = 0.7
# Destillerovnen i laboratoriet, og pipa over.
OVX, OVY = 1.75, AIY1 - 0.45


def apothecary_outside(p, rng):
    """Utsida: sokkel, trinn, vegger med salgsluke, utkraget overetasje,
    skifertak, pipe, skilt, lykt, og det som står ute ved luka og døra."""
    sx, sy = AW / 2 + 0.1, AD / 2 + 0.1
    box("plinth", (-sx, -sy, -0.2), (sx, sy, AF0), p.stone, bevel=0.06)
    box("step", (ADOOR - 0.8, -sy - 0.75, -0.2), (ADOOR + 0.8, -sy + 0.05, 0.28), p.stone, bevel=0.05)
    box("step", (ADOOR - 0.75, -sy - 0.38, 0.2), (ADOOR + 0.75, -sy + 0.05, AF0 - 0.02), p.stone, bevel=0.05)
    ramp_to(ADOOR - 0.6, ADOOR + 0.6, -sy - 0.95, AIY0 + 0.05, 0.0, AF0)

    fw = (AF0 + 0.95, AF0 + 2.05)
    hatch = (AF0 + 0.85, AF0 + 1.95)
    front = [(ADOOR - AX0, AF0, AF0 + 2.2, 1.15), (AHATCH - AX0, hatch[0], hatch[1], 1.5)]
    fr, _ = half_timbered(p, "wall_front", (AX0, AY0), (AX1, AY0), AF0, AC0, -1, front, rng)
    back = [(-2.6 - AX0, fw[0], fw[1], 0.9), (0.0 - AX0, fw[0], fw[1], 0.9)]
    bk, _ = half_timbered(p, "wall_back", (AX0, AY1), (AX1, AY1), AF0, AC0, 1, back, rng)
    left = [(-1.6 - AY0, fw[0], fw[1], 0.9), (2.1 - AY0, fw[0], fw[1], 0.9)]
    lf, _ = half_timbered(p, "wall_left", (AX0, AY0), (AX0, AY1), AF0, AC0, 1, left, rng, own=(False, False))
    right = [(-0.2 - AY0, fw[0] + 0.25, fw[1], 0.7)]
    rt, _ = half_timbered(p, "wall_right", (AX1, AY0), (AX1, AY1), AF0, AC0, -1, right, rng, own=(False, False))
    for frame, opens, out in ((bk, back, 1), (lf, left, 1), (rt, right, -1)):
        for s, za, zb, w in opens:
            window(p, frame, s, (za + zb) / 2, w, zb - za, out, rng=rng, inside=True, in_frame=True)
    door(p, fr, ADOOR - AX0, AF0, 1.15, 2.2, -1, in_frame=True)
    hatch_front(p, rng, hatch)

    # Bjelkelaget: bjelkeender under utkragingen foran, og et bord langs.
    zc = (AC0 + AF1) / 2
    x = AX0 + 0.3
    while x < AX1:
        stick("joist_end", (x, AUY0 - 0.05, (AC0 + AF1 - AJETTY_T) / 2), (x, AY0 + AT / 2 - 0.05, (AC0 + AF1 - AJETTY_T) / 2), 0.2, AF1 - AJETTY_T - AC0, p.timber, ("timber", rng.randrange(4)), seed=x, bevel=0.0)
        x += 0.7
    stick("jetty_board", (AX0 - 0.1, AUY0 - 0.05, zc + 0.05), (AX1 + 0.1, AUY0 - 0.05, zc + 0.05), 0.08, 0.3, p.trim, ("trim", CARVED))
    for x in (AX0, AX1):
        box("floor_band", (x - AT / 2 - 0.02, AY0 - 0.02, AC0), (x + AT / 2 + 0.02, AY1 + 0.02, AF1 - 0.012), p.timber)
    box("floor_band", (AX0 - AT / 2, AY1 - AT / 2 - 0.02, AC0), (AX1 + AT / 2, AY1 + AT / 2 + 0.02, AF1 - 0.012), p.timber)

    # Overetasjen: urteloftet med knevegger og vinduer.
    uw = (AF1 + 0.7, AF1 + 1.7)
    ufront = [(x - AX0, uw[0], uw[1], 0.8) for x in (-2.4, 0.4)]
    ufr, _ = half_timbered(p, "upper_front", (AX0, AUY0), (AX1, AUY0), AF1, AE, -1, ufront, rng)
    uback = [(-1.0 - AX0, uw[0], uw[1], 0.8)]
    ubk, _ = half_timbered(p, "upper_back", (AX0, AY1), (AX1, AY1), AF1, AE, 1, uback, rng)
    ulf, _ = half_timbered(p, "upper_left", (AX0, AUY0), (AX0, AY1), AF1, AE, 1, [(0.0 - AUY0, uw[0], uw[1] + 0.3, 0.8)], rng, own=(False, False))
    urt, _ = half_timbered(p, "upper_right", (AX1, AUY0), (AX1, AY1), AF1, AE, -1, [(0.4 - AUY0, uw[0], uw[1] + 0.3, 0.8)], rng, own=(False, False))
    for frame, opens, out in ((ufr, ufront, -1), (ubk, uback, 1), (ulf, [(0.0 - AUY0, uw[0], uw[1] + 0.3, 0.8)], 1), (urt, [(0.4 - AUY0, uw[0], uw[1] + 0.3, 0.8)], -1)):
        for s, za, zb, w in opens:
            window(p, frame, s, (za + zb) / 2, w, zb - za, out, rng=rng, inside=True, in_frame=True)

    # Taket og gavlene, med et lite lukevindu øverst i gavlene.
    ridge = kit.gable_roof(p, AX0 - AT / 2, AX1 + AT / 2, AUY0 - AT / 2, AY1 + AT / 2, AE + 0.28, APITCH, overhang=0.75)
    yc = (AUY0 + AY1) / 2
    for x, out in ((AX0, -1), (AX1, 1)):
        kit.gable_wall(p, x, AUY0 - AT / 2, AY1 + AT / 2, AE, ridge, APITCH, out, rng)
        gf = kit.wall_frame((x, AUY0), (x, AY1))[0]
        window(p, gf, yc - AUY0, AE + 1.3, 0.6, 0.7, -out, shutters=False, rng=rng, inside=True)
    # Pipa fra destillerovnen, gjennom bakre takflate.
    tan = math.tan(math.radians(APITCH))
    half = (AY1 + AT / 2 - (AUY0 - AT / 2)) / 2
    roof_at = lambda y: AE + 0.28 + (half - abs(y - yc)) * tan
    kit.chimney(p, OVX, OVY + 0.05, roof_at(OVY) - 0.9, ridge + 0.6, w=0.85, smoke=0.6, lean=5.0)

    # Skiltet: morter og støter i smijern på en arm ved hjørnet.
    ax, ay, az = AX1 - 0.15, AY0 - AT / 2, AC0 - 0.35
    stick("sign_arm", (ax, ay, az), (ax, ay - 1.1, az), 0.06, 0.07, p.iron, bevel=0.01)
    stick("sign_strut", (ax, ay, az - 0.5), (ax, ay - 0.75, az - 0.02), 0.05, 0.05, p.iron, bevel=0)
    stick("sign_chain", (ax, ay - 0.75, az), (ax, ay - 0.75, az - 0.22), 0.02, 0.02, p.iron, bevel=0)
    copper = layer("plaster_white", (0.95, 0.5, 0.28), 0.3)
    revolve("sign_mortar", [(0.12, 0.0), (0.2, 0.05), (0.24, 0.26), (0.18, 0.26), (0.13, 0.08), (0.0, 0.08)], copper, n=8, at=(ax, ay - 0.75, az - 0.52))
    stick("sign_pestle", (ax - 0.06, ay - 0.72, az - 0.42), (ax + 0.14, ay - 0.78, az - 0.12), 0.07, 0.07, copper, bevel=0.02)
    town.wall_lantern(p, ADOOR + 1.0, AY0 - AT / 2, AF0 + 2.3, -1)
    # Plantekrukker ved døra.
    clay = layer("plaster_red", (0.85, 0.55, 0.42))
    green = layer("sod", (0.75, 1.0, 0.6))
    for k, (x, y) in enumerate(((ADOOR + 1.25, -sy - 0.45), (ADOOR + 1.55, -sy - 0.95), (ADOOR - 1.2, -sy - 0.4))):
        r = 0.2 + 0.03 * k
        jar(x, y, 0.0, r, r * 1.5, clay, n=8, neck=False)
        revolve("plant", [(r * 0.85, 0.0), (r * 1.25, r * 0.6), (r * 0.7, r * 1.6), (0.0, r * 2.0)], green, n=6, at=(x, y, r * 1.35))
        collide((x, y, 0.35), (r * 2 + 0.05, r * 2 + 0.05, 0.7))
    # Urtebunter til tørk på en stang under utkragingen.
    tie = p.trim
    bar_z = AC0 - 0.12
    stick("herb_bar", (-3.4, AY0 - AT / 2 - 0.3, bar_z), (-0.4, AY0 - AT / 2 - 0.3, bar_z), 0.05, 0.05, p.timber, ("timber", 1), bevel=0)
    for x in (-3.3, -0.5):
        stick("herb_bar_hook", (x, AY0 - AT / 2, bar_z), (x, AY0 - AT / 2 - 0.32, bar_z), 0.03, 0.03, p.iron, bevel=0)
    herbmats = [layer("sod", (0.8, 0.95, 0.55)), layer("thatch", (0.75, 0.8, 0.45)), layer("sod", (0.95, 0.75, 0.55))]
    for k in range(8):
        herbs(-3.2 + k * 0.38, AY0 - AT / 2 - 0.3, bar_z - 0.02, rng.uniform(0.28, 0.4), herbmats[k % 3], rng, tie)
    return ridge


def hatch_front(p, rng, hatch):
    """Salgsluka: overskodda er slått opp og holdt med en stang, underskodda
    er slått ned til et bord på to knekter, med krukker og en vekt."""
    z0, z1 = hatch
    w = 1.5
    fy = AY0 - AT / 2
    # Karmen.
    for sx in (-1, 1):
        stick("hatch_jamb", (AHATCH + sx * (w / 2 + 0.07), fy - 0.06, z0 - 0.05), (AHATCH + sx * (w / 2 + 0.07), fy - 0.06, z1 + 0.05), 0.14, 0.14, p.trim, ("trim", DARK_BEAM), bevel=0.02)
    stick("hatch_head", (AHATCH - w / 2 - 0.2, fy - 0.07, z1 + 0.1), (AHATCH + w / 2 + 0.2, fy - 0.07, z1 + 0.1), 0.18, 0.16, p.trim, ("trim", CARVED), bevel=0.02)
    # Bordet (underskodda) på knekter, flukt med terskelen inne.
    top = z0 + 0.02
    box("hatch_board", (AHATCH - w / 2, fy - 0.62, top - 0.06), (AHATCH + w / 2, fy + 0.05, top), p.cladding, bevel=0.015)
    box("hatch_sill", (AHATCH - w / 2, fy - 0.02, top - 0.06), (AHATCH + w / 2, AY0 + AT / 2 + 0.05, top), p.floor, bevel=0.01)
    for sx in (-1, 1):
        stick("hatch_bracket", (AHATCH + sx * (w / 2 - 0.12), fy - 0.03, top - 0.55), (AHATCH + sx * (w / 2 - 0.12), fy - 0.55, top - 0.07), 0.06, 0.07, p.timber, ("timber", 2), bevel=0.015)
        stick("hatch_chain", (AHATCH + sx * (w / 2 - 0.05), fy - 0.6, top), (AHATCH + sx * (w / 2 + 0.02), fy - 0.03, z1 - 0.1), 0.015, 0.015, p.iron, bevel=0)
    # Overskodda slått opp som et lite tak, på en stang.
    lid = box("hatch_lid", (AHATCH - w / 2 - 0.05, -0.55, -0.03), (AHATCH + w / 2 + 0.05, 0.0, 0.03), p.cladding, bevel=0.015)
    turned(lid, (0, 0, 0), Matrix.Translation((0, fy - 0.08, z1 + 0.04)) @ Matrix.Rotation(-0.42, 4, "X"))
    for sx in (-1, 1):
        stick("hatch_prop", (AHATCH + sx * (w / 2 - 0.1), fy - 0.55, top), (AHATCH + sx * (w / 2 - 0.1), fy - 0.5, z1 + 0.2), 0.03, 0.03, p.timber, ("timber", 0), bevel=0)
    collide((AHATCH, fy - 0.3, top - 0.35), (w, 0.65, 0.8))
    # Varene på bordet.
    clay = layer("plaster_red", (0.85, 0.55, 0.42))
    white = layer("plaster_white", (0.9, 0.88, 0.8))
    blue = layer("plaster_white", (0.45, 0.6, 0.85))
    green_glass = plain("glass_green", (0.08, 0.2, 0.1), 0.1)
    xs = [AHATCH - 0.6 + k * 0.2 for k in range(7)]
    for k, x in enumerate(xs):
        y = fy - 0.35 + rng.uniform(-0.08, 0.08)
        if k == 3:
            continue
        if k % 3 == 0:
            bottle(x, y, top, 0.05, rng.uniform(0.2, 0.26), green_glass)
        else:
            jar(x, y, top, rng.uniform(0.05, 0.07), rng.uniform(0.13, 0.2), [clay, white, blue][k % 3], n=6)
    # Skålvekta midt på bordet.
    brass = layer("iron", (0.95, 0.75, 0.4), 0.35)
    stick("scale_post", (AHATCH, fy - 0.35, top), (AHATCH, fy - 0.35, top + 0.35), 0.025, 0.025, brass, bevel=0)
    stick("scale_beam", (AHATCH - 0.16, fy - 0.35, top + 0.35), (AHATCH + 0.16, fy - 0.35, top + 0.35), 0.02, 0.02, brass, bevel=0)
    for sx in (-1, 1):
        stick("scale_string", (AHATCH + sx * 0.15, fy - 0.35, top + 0.35), (AHATCH + sx * 0.15, fy - 0.35, top + 0.12), 0.006, 0.006, brass, bevel=0)
        revolve("scale_pan", [(0.07, 0.0), (0.0, -0.03)], brass, n=7, at=(AHATCH + sx * 0.15, fy - 0.35, top + 0.12))


def apothecary_ground(p, rng):
    """Første etasje inne: gulv, bjelkelag med trapphull, skilleveggen,
    salgsrommet og laboratoriet."""
    # Gulvet og kollideren.
    planks(p, "floor0", (AIX0, AIY0, AF0 - 0.08), (AIX1, AIY1, AF0 + 0.025))
    collide((0, 0, AF0 - 0.2), (AIX1 - AIX0, AIY1 - AIY0, 0.4))
    # Bjelkelaget og bordene under gulvet oppe, med trapphullet.
    kit.floor_frame(p, "floor1", (AIX0, AIY0), (AIX1, AIY1), (HX0, HY0, HX1, HY1), AC0, AF1)
    x = AIX0 + 0.45
    while x < AIX1 - 0.1:
        if HX0 - 0.1 < x:
            for y0, y1 in ((AIY0, HY0), (HY1, AIY1)):
                stick("joist", (x, y0, AC0 - 0.12), (x, y1, AC0 - 0.12), 0.18, 0.24, p.timber, ("timber", rng.randrange(4)), seed=x)
        else:
            stick("joist", (x, AIY0, AC0 - 0.12), (x, AIY1, AC0 - 0.12), 0.18, 0.24, p.timber, ("timber", rng.randrange(4)), seed=x)
        x += 0.85
    for yy in (HY0, HY1):
        stick("trimmer", (HX0 - 0.1, yy, AC0 - 0.12), (AIX1, yy, AC0 - 0.12), 0.2, 0.24, p.timber, ("timber", 2))
    stick("trimmer", (HX0 - 0.05, AIY0, AC0 - 0.12), (HX0 - 0.05, AIY1, AC0 - 0.12), 0.2, 0.24, p.timber, ("timber", 1))
    stairs(p, SX0, SX1, SY0, SY1, AF0, AF1, along="y", rails=(SX0 - 0.03,), room=(AIY0, AIY1), under=AUNDER, well=(HY0, HY1))

    # Skilleveggen mot laboratoriet, med døråpning og karm.
    pw = 0.16
    fp, L = kit.wall_frame((AIX0, PY), (HX0 - 0.12, PY))
    hole = [(PDOOR - AIX0 - 0.5, PDOOR - AIX0 + 0.5, AF0, AF0 + 2.15)]
    kit.slab_grid("partition", fp, L, AF0, AC0 - 0.24, pw, hole, p.plaster)
    kit.wall_colliders(fp, L, AF0, AC0, pw, hole)
    for side in (-1, 1):
        stick("pdoor_post", (PDOOR + side * 0.56, PY, AF0), (PDOOR + side * 0.56, PY, AF0 + 2.25), 0.12, 0.24, p.timber, ("timber", 1))
    stick("pdoor_head", (PDOOR - 0.68, PY, AF0 + 2.21), (PDOOR + 0.68, PY, AF0 + 2.21), 0.16, 0.26, p.trim, ("trim", DARK_BEAM))
    for s in (0.08, L - 0.08):
        stick("ppost", kit.at_wall(fp, s, 0, AF0), kit.at_wall(fp, s, 0, AC0 - 0.24), 0.2, 0.22, p.timber, ("timber", 3))
    stick("pplate", (AIX0, PY, AC0 - 0.3), (HX0 - 0.12, PY, AC0 - 0.3), 0.2, 0.14, p.timber, ("timber", 0))

    sales_room(p, rng)
    laboratory(p, rng)
    light((-1.0, -1.2, AC0 - 0.6), (1.0, 0.7, 0.42), 1.1, 6.0)


def sales_room(p, rng):
    """Salgsrommet: disken, hyllene med krukker og flasker bak den, krakk og
    urtebunter fra bjelkene."""
    clay = layer("plaster_red", (0.85, 0.55, 0.42))
    white = layer("plaster_white", (0.9, 0.88, 0.8))
    blue = layer("plaster_white", (0.45, 0.6, 0.85))
    ochre = layer("plaster", (0.95, 0.75, 0.45))
    green_glass = plain("glass_green", (0.08, 0.2, 0.1), 0.1)
    brown_glass = plain("glass_brown", (0.2, 0.1, 0.05), 0.1)
    pots = [clay, white, blue, ochre]
    # Reolen langs vestveggen, fra fasaden til skilleveggen.
    with piece("apothecary_shelf", (AIX0 + 0.18, (AIY0 + PY - 0.07) / 2, AF0)):
        zs, yi = shelf_unit(p, AIY0 + 0.05, PY - 0.12, AIX0, 0.36, AF0, 2.35, 5, along="y", out=1)
        for r, z in enumerate(zs):
            u = AIY0 + 0.2
            # Ikke foran salgsluka (den sitter i fasaden, ikke på vestveggen).
            while u < PY - 0.3:
                x = AIX0 + 0.2
                if r > 0 and rng.random() < 0.2:
                    books(u, u + 0.35, x, z, 0.22, rng, along="y")
                    u += 0.42
                    continue
                if rng.random() < 0.35:
                    rr = rng.uniform(0.04, 0.055)
                    bottle(x, u + rr, z, rr, rng.uniform(0.17, 0.3), rng.choice((green_glass, brown_glass)))
                else:
                    rr = rng.uniform(0.05, 0.08)
                    jar(x, u + rr, z, rr, min(rng.uniform(0.12, 0.28), 0.36), rng.choice(pots), n=6)
                u += rr * 2 + rng.uniform(0.03, 0.07)
    # Disken på tvers foran reolen, med vekt, morter, bok og pengeskrin.
    cx0, cx1, cy0, cy1 = -1.2, -0.62, -2.45, -0.3
    with piece("apothecary_counter", ((cx0 + cx1) / 2, (cy0 + cy1) / 2, AF0)):
        box("counter", (cx0, cy0, AF0), (cx1, cy1, AF0 + 0.98), p.cladding, bevel=0.03)
        box("counter_top", (cx0 - 0.05, cy0 - 0.08, AF0 + 0.98), (cx1 + 0.1, cy1 + 0.08, AF0 + 1.06), p.floor, bevel=0.02)
        stick("counter_trim", (cx1 + 0.02, cy0, AF0 + 0.9), (cx1 + 0.02, cy1, AF0 + 0.9), 0.05, 0.1, p.trim, ("trim", IRON_BAND), bevel=0)
        for k in range(4):
            y = cy0 + 0.2 + k * (cy1 - cy0 - 0.4) / 3
            stick("counter_post", (cx1 + 0.03, y, AF0), (cx1 + 0.03, y, AF0 + 0.9), 0.08, 0.08, p.timber, ("timber", k % 4), bevel=0.02)
        collide(((cx0 + cx1) / 2, (cy0 + cy1) / 2, AF0 + 0.53), (cx1 - cx0 + 0.15, cy1 - cy0 + 0.16, 1.06))
        top = AF0 + 1.06
        brass = layer("iron", (0.95, 0.75, 0.4), 0.35)
        stick("scale_post", (-0.95, -1.9, top), (-0.95, -1.9, top + 0.42), 0.03, 0.03, brass, bevel=0)
        stick("scale_beam", (-0.95, -2.12, top + 0.42), (-0.95, -1.68, top + 0.42), 0.02, 0.02, brass, bevel=0)
        for sy in (-1, 1):
            stick("scale_string", (-0.95, -1.9 + sy * 0.2, top + 0.42), (-0.95, -1.9 + sy * 0.2, top + 0.16), 0.006, 0.006, brass, bevel=0)
            revolve("scale_pan", [(0.08, 0.0), (0.0, -0.035)], brass, n=7, at=(-0.95, -1.9 + sy * 0.2, top + 0.16))
        stone = layer("stone", (0.9, 0.88, 0.82))
        revolve("mortar", [(0.08, 0.0), (0.12, 0.04), (0.13, 0.13), (0.1, 0.13), (0.07, 0.06), (0.0, 0.06)], stone, n=8, at=(-0.9, -1.4, top))
        stick("pestle", (-0.9, -1.4, top + 0.08), (-0.84, -1.35, top + 0.28), 0.04, 0.04, p.floor, bevel=0.01)
        box("book_open", (-1.1, -1.0, top), (-0.7, -0.7, top + 0.04), layer("linen", (0.95, 0.9, 0.78)), bevel=0.01)
        box("book_cover", (-1.12, -1.02, top), (-0.68, -0.68, top + 0.015), layer("leather", (0.6, 0.3, 0.2)), bevel=0)
        box("cash_box", (-1.1, -0.6, top), (-0.8, -0.38, top + 0.14), p.door_dark, bevel=0.02)
        box("cash_band", (-1.11, -0.61, top + 0.08), (-0.79, -0.37, top + 0.1), brass, bevel=0)
        candle(p, -0.8, -2.25, top, light_it=True)
    # Kunder kan sitte på en krakk ved døra, og sekker med tørkede urter.
    with piece("stool", (0.8, -2.6, AF0)):
        stool(p, 0.8, -2.6, AF0)
        collide((0.8, -2.6, AF0 + 0.25), (0.4, 0.4, 0.5))
    with piece("herb_sacks", (-2.7, AIY0 + 0.31, AF0)):
        for k, (x, y) in enumerate(((-2.95, AIY0 + 0.3), (-2.45, AIY0 + 0.32))):
            box("sack", (x - 0.24, y - 0.2, AF0), (x + 0.24, y + 0.2, AF0 + 0.6), layer("linen", (0.85, 0.78, 0.62)), bevel=0.12)
            revolve("sack_top", [(0.16, 0.0), (0.08, 0.1), (0.0, 0.1)], layer("sod", (0.75, 0.9, 0.5)), n=6, at=(x, y, AF0 + 0.6))
        collide((-2.7, AIY0 + 0.31, AF0 + 0.3), (1.0, 0.5, 0.6))
    # Urtebunter fra bjelkene over salgsrommet.
    herbmats = [layer("sod", (0.8, 0.95, 0.55)), layer("thatch", (0.75, 0.8, 0.45)), layer("sod", (0.95, 0.75, 0.55)), layer("thatch", (0.9, 0.6, 0.5))]
    x = AIX0 + 0.45
    j = 0
    while x < HX0 - 0.3:
        for y in (-2.6, -1.9, -1.2, -0.5):
            if rng.random() < 0.55:
                herbs(x, y + rng.uniform(-0.1, 0.1), AC0 - 0.24, rng.uniform(0.25, 0.42), herbmats[j % 4], rng, p.trim)
                j += 1
        x += 0.85
    # Stige-hylle over døra innenfor fasaden med glass.
    with piece("door_shelf_jars", ((AIX0 + 0.3) / 2, AIY0 + 0.125, AF0 + 2.35)):
        box("door_shelf", (AIX0 + 0.5, AIY0, AF0 + 2.35), (-0.2, AIY0 + 0.25, AF0 + 2.4), p.floor, bevel=0.01)
        for k in range(9):
            x = AIX0 + 0.62 + k * 0.38
            if abs(x - AHATCH) < 0.95:
                continue
            jar(x, AIY0 + 0.13, AF0 + 2.4, 0.06, 0.18, pots[k % 4], n=6)


def laboratory(p, rng):
    """Laboratoriet i bakrommet: destillerovn med kobberkolbe og rør til
    forlaget, arbeidsbenk med kolber, bord med mortere og bøker, og
    bokhylla på vestveggen."""
    copper = layer("plaster_white", (0.95, 0.5, 0.28), 0.3)
    # Teglen i ovnen: laget «chimney» (det finnes ikke noe «brick»-lag).
    brick = layer("chimney", (0.9, 0.62, 0.48))
    glass = plain("glass_flask", (0.1, 0.16, 0.12), 0.08)
    green_glass = plain("glass_green", (0.08, 0.2, 0.1), 0.1)
    brown_glass = plain("glass_brown", (0.2, 0.1, 0.05), 0.1)
    # Destillerovnen i hjørnet: murt av tegl, med ildhull, og pipa opp.
    ox0, ox1, oy0, oy1 = OVX - 0.55, OVX + 0.55, OVY - 0.5, AIY1
    box("oven", (ox0, oy0, AF0), (ox1, oy1, AF0 + 0.95), brick, bevel=0.04)
    box("oven_top", (ox0 - 0.05, oy0 - 0.05, AF0 + 0.95), (ox1 + 0.05, oy1, AF0 + 1.03), p.stone, bevel=0.03)
    box("oven_mouth", (OVX - 0.22, oy0 - 0.02, AF0 + 0.18), (OVX + 0.22, oy0 + 0.06, AF0 + 0.5), p.soot, bevel=0.0)
    glow = plain("fire_coals", (1.0, 0.42, 0.12), 0.4)
    box("oven_embers", (OVX - 0.18, oy0 - 0.03, AF0 + 0.2), (OVX + 0.18, oy0 + 0.03, AF0 + 0.3), glow, bevel=0.01)
    # Hetta smalner oppover og går inn i bjelkelaget (pipa står oppe).
    box("oven_hood", (OVX - 0.55, OVY - 0.5, AF0 + 1.75), (OVX + 0.55, oy1, AF0 + 2.05), brick, bevel=0.03)
    box("oven_hood", (OVX - 0.42, OVY - 0.32, AF0 + 2.05), (OVX + 0.42, oy1, AC0), brick, bevel=0.03)
    for sx in (-1, 1):
        stick("hood_bracket", (OVX + sx * 0.5, oy1 - 0.02, AF0 + 1.25), (OVX + sx * 0.5, OVY - 0.45, AF0 + 1.74), 0.07, 0.08, p.timber, ("timber", 2), bevel=0.01)
    light((OVX, oy0 - 0.3, AF0 + 0.5), (1.0, 0.48, 0.18), 1.3, 4.5)
    # Kobberkolben (alembikken) på ovnen og røret til forlaget på benken.
    revolve("alembic", [(0.16, 0.0), (0.24, 0.12), (0.25, 0.28), (0.18, 0.42), (0.08, 0.52), (0.07, 0.62), (0.0, 0.64)], copper, n=10, at=(OVX, OVY - 0.05, AF0 + 1.03))
    revolve("alembic_head", [(0.1, 0.0), (0.13, 0.08), (0.08, 0.16), (0.0, 0.17)], copper, n=8, at=(OVX, OVY - 0.05, AF0 + 1.64))
    stick("still_pipe", (OVX - 0.08, OVY - 0.05, AF0 + 1.72), (OVX - 1.15, OVY - 0.05, AF0 + 1.25), 0.04, 0.04, copper, bevel=0)
    # Benken langs bakveggen.
    bx0, bx1, by0, by1 = -1.55, OVX - 0.62, AIY1 - 0.7, AIY1
    with piece("lab_bench", ((bx0 + bx1) / 2, (by0 + by1) / 2, AF0)):
        box("bench_top", (bx0, by0, AF0 + 0.85), (bx1, by1, AF0 + 0.93), p.floor, bevel=0.02)
        for xx in (bx0 + 0.08, bx1 - 0.08):
            for yy in (by0 + 0.08, by1 - 0.08):
                stick("bench_leg", (xx, yy, AF0), (xx, yy, AF0 + 0.85), 0.08, 0.08, p.timber, ("timber", 1), bevel=0.02)
        box("bench_shelf", (bx0 + 0.04, by0 + 0.04, AF0 + 0.25), (bx1 - 0.04, by1 - 0.04, AF0 + 0.29), p.floor, bevel=0.01)
        collide(((bx0 + bx1) / 2, (by0 + by1) / 2, AF0 + 0.47), (bx1 - bx0, by1 - by0, 0.94))
        z = AF0 + 0.93
        flask(OVX - 1.2, OVY - 0.05, z, 0.11, glass, n=8)
        flask(-0.9, AIY1 - 0.35, z, 0.08, green_glass)
        flask(-0.5, AIY1 - 0.25, z, 0.07, glass)
        for k in range(5):
            bottle(-1.4 + k * 0.12, AIY1 - 0.55, z, 0.04, 0.16 + 0.02 * (k % 3), [green_glass, brown_glass, glass][k % 3])
        # Et lite stativ med reagensglass, og en spiritusbrenner.
        box("rack", (-0.25, AIY1 - 0.6, z), (0.15, AIY1 - 0.48, z + 0.1), p.floor, bevel=0.01)
        for k in range(4):
            stick("tube", (-0.2 + k * 0.1, AIY1 - 0.54, z + 0.02), (-0.2 + k * 0.1, AIY1 - 0.54, z + 0.22), 0.03, 0.03, glass, bevel=0)
        for k in range(3):
            box("bench_crock", (0.3 + k * 0.22, AIY1 - 0.2, z), (0.45 + k * 0.22, AIY1 - 0.05, z + 0.2), layer("plaster_red", (0.85, 0.55, 0.42)), bevel=0.04)
    # Hylle over benken med flere flasker.
    with piece("lab_shelf", ((bx0 + bx1) / 2, AIY1 - 0.14, AF0 + 1.75)):
        box("lab_shelf", (bx0, AIY1 - 0.28, AF0 + 1.75), (bx1, AIY1, AF0 + 1.8), p.floor, bevel=0.01)
        for k in range(9):
            x = bx0 + 0.15 + k * 0.32
            if x > bx1 - 0.1:
                break
            if k % 2:
                bottle(x, AIY1 - 0.14, AF0 + 1.8, 0.05, 0.22, [green_glass, brown_glass][k % 4 // 2])
            else:
                jar(x, AIY1 - 0.14, AF0 + 1.8, 0.07, 0.18, layer("plaster_white", (0.9, 0.88, 0.8)), n=6)

    # Bordet midt i rommet med mortere, en bok, et lys og en hodeskalle-
    # lignende krukke.
    tx, ty = -1.0, 1.55
    with piece("lab_table", (tx, ty, AF0)):
        table(p, tx, ty, AF0, 1.4, 0.75)
        stone = layer("stone", (0.9, 0.88, 0.82))
        for k, (dx, dy, r) in enumerate(((-0.45, -0.15, 0.12), (-0.1, 0.18, 0.09), (0.45, -0.1, 0.07))):
            revolve("mortar", [(r * 0.65, 0.0), (r, r * 0.35), (r * 1.05, r * 1.0), (r * 0.8, r * 1.0), (r * 0.55, r * 0.5), (0.0, r * 0.5)], stone if k != 1 else layer("iron", (0.95, 0.75, 0.4), 0.35), n=8, at=(tx + dx, ty + dy, AF0 + 0.78))
            stick("pestle", (tx + dx, ty + dy, AF0 + 0.78 + r * 0.6), (tx + dx + 0.05, ty + dy + 0.04, AF0 + 0.78 + r * 2.6), 0.035, 0.035, p.floor, bevel=0.01)
        box("lab_book", (tx + 0.05, ty - 0.35, AF0 + 0.78), (tx + 0.42, ty - 0.08, AF0 + 0.82), layer("linen", (0.95, 0.9, 0.78)), bevel=0.01)
        candle(p, tx + 0.6, ty + 0.25, AF0 + 0.78, light_it=True)
        for k in range(5):
            a = rng.uniform(0, 6.28)
            box("herb_leaf", (tx - 0.2 + math.cos(a) * 0.15, ty + 0.1 + math.sin(a) * 0.1, AF0 + 0.78), (tx - 0.12 + math.cos(a) * 0.15, ty + 0.15 + math.sin(a) * 0.1, AF0 + 0.79), layer("sod", (0.8, 0.95, 0.55)), bevel=0)
    with piece("stool", (tx - 0.2, ty - 0.75, AF0)):
        stool(p, tx - 0.2, ty - 0.75, AF0)
        collide((tx - 0.2, ty - 0.75, AF0 + 0.25), (0.4, 0.4, 0.5))

    # Bokhylla på vestveggen i laboratoriet.
    with piece("bookcase_lab", (AIX0 + 0.17, (PY + AIY1 + 0.1) / 2, AF0)):
        zs, yi = shelf_unit(p, PY + 0.15, AIY1 - 0.05, AIX0, 0.34, AF0, 2.3, 5, along="y", out=1)
        for z in zs:
            books(PY + 0.25, AIY1 - 0.2, AIX0 + 0.18, z, 0.24, rng, along="y")
    # Urtebunter fra bjelkene her også.
    herbmats = [layer("sod", (0.8, 0.95, 0.55)), layer("thatch", (0.75, 0.8, 0.45))]
    for k, (x, y) in enumerate(((-2.0, 1.0), (-1.15, 1.2), (-0.3, 0.9), (0.55, 1.9), (-2.75, 2.5), (-0.3, 2.3))):
        herbs(x, y, AC0 - 0.24, rng.uniform(0.22, 0.36), herbmats[k % 2], rng, p.trim)
    light((-0.5, 2.0, AC0 - 0.7), (1.0, 0.72, 0.45), 0.9, 5.0)


def apothecary_upper(p, rng):
    """Urteloftet: gulvet over utkragingen, sperrene, rekkverket rundt
    trapphullet, tørkestativ, kurver, sekker, seng og urter."""
    uy0 = AUY0 + AT / 2
    planks(p, "floor_jetty", (AIX0, uy0, AF1 - AJETTY_T), (AIX1, AIY0, AF1))
    collide(((AIX0 + AIX1) / 2, (uy0 + AIY0) / 2, AF1 - 0.12), (AIX1 - AIX0, AIY0 - uy0, 0.25))
    # Sperrene og hanebjelkene.
    tan = math.tan(math.radians(APITCH))
    yc = (AUY0 + AY1) / 2
    half = (AY1 + AT / 2 - (AUY0 - AT / 2)) / 2
    ridge_in = AE + 0.28 + half * tan - 0.65
    zc = AE + 1.2
    cw = half - (zc - AE) / tan - 0.25
    x = AIX0 + 0.35
    collars = []
    while x < AIX1:
        for side in (-1, 1):
            stick("rafter", (x, yc + side * (half - 0.3), AE + 0.05), (x, yc, ridge_in), 0.15, 0.2, p.timber, ("timber", rng.randrange(4)), seed=x * side)
        stick("collar_tie", (x, yc - cw, zc), (x, yc + cw, zc), 0.14, 0.18, p.timber, ("timber", rng.randrange(4)), seed=x + 9)
        collars.append(x)
        x += 1.5
    # Rekkverket rundt trapphullet.
    kit.well_rail(p, (((HX0 - 0.06, HY0 - 0.06), (HX0 - 0.06, HY1)), ((HX0 - 0.06, HY0 - 0.06), (AIX1, HY0 - 0.06))), AF1)
    # Pipa fra ovnen går gjennom loftet.
    box("chimney_inside", (OVX - 0.4, OVY - 0.35, AF1), (OVX + 0.45, AIY1, ridge_in - 1.0), p.stone, bevel=0.04)
    collide((OVX + 0.02, (OVY - 0.35 + AIY1) / 2, AF1 + 1.5), (0.85, AIY1 - OVY + 0.35, 3.0))

    # Benker og kister langs kneveggene (der taket går lavt), så du ikke
    # står med hodet i takflaten.
    herbmats = [layer("sod", (0.8, 0.95, 0.55)), layer("thatch", (0.75, 0.8, 0.45)), layer("sod", (0.95, 0.75, 0.55)), layer("thatch", (0.9, 0.6, 0.5))]
    for y0, y1 in ((uy0, uy0 + 0.75), (AIY1 - 0.75, AIY1)):
        x0, x1 = AIX0, HX0 - 0.15 if y0 < 0 else OVX - 0.5
        dry = piece("drying_bed_front" if y0 < 0 else "drying_bed_back", ((x0 + x1) / 2, (y0 + y1) / 2, AF1))
        dry.__enter__()
        box("drying_bed", (x0 + 0.05, y0, AF1 + 0.4), (x1, y1, AF1 + 0.5), p.boards, bevel=0.02)
        for xx in (x0 + 0.15, (x0 + x1) / 2, x1 - 0.1):
            for yy in (y0 + 0.08, y1 - 0.08):
                stick("drying_leg", (xx, yy, AF1), (xx, yy, AF1 + 0.4), 0.07, 0.07, p.timber, ("timber", 1), bevel=0)
        # Urter lagt utover til tørk.
        xx = x0 + 0.3
        k = 0
        while xx < x1 - 0.2:
            box("drying_herbs", (xx - 0.18, y0 + 0.1, AF1 + 0.5), (xx + 0.18, y1 - 0.1, AF1 + 0.56), herbmats[k % 4], bevel=0.02)
            xx += 0.45
            k += 1
        collide(((x0 + x1) / 2, (y0 + y1) / 2, AF1 + 0.6), (x1 - x0, y1 - y0, 1.2))
        dry.__exit__(None, None, None)
    # Tørkestativ midt på loftet: rammer med stenger og bunter.
    for k, x in enumerate((-1.8, 0.4)):
        y0r, y1r = yc - 1.1, yc + 1.1
        rack = piece(f"drying_rack_{k}", (x, yc, AF1))
        rack.__enter__()
        for yy in (y0r, y1r):
            stick("rack_post", (x, yy, AF1), (x, yy, AF1 + 1.8), 0.09, 0.09, p.timber, ("timber", k), bevel=0.02)
            stick("rack_foot", (x - 0.35, yy, AF1 + 0.05), (x + 0.35, yy, AF1 + 0.05), 0.09, 0.1, p.timber, ("timber", 2), bevel=0.02)
        for z in (AF1 + 1.15, AF1 + 1.75):
            stick("rack_bar", (x, y0r, z), (x, y1r, z), 0.05, 0.05, p.timber, ("timber", 3), bevel=0)
            yy = y0r + 0.2
            j = 0
            while yy < y1r - 0.1:
                herbs(x, yy, z - 0.02, rng.uniform(0.2, 0.32), herbmats[(j + k) % 4], rng, p.trim)
                yy += 0.3
                j += 1
        collide((x, yc, AF1 + 0.9), (0.3, 2.3, 1.8))
        rack.__exit__(None, None, None)
    # Urter fra hanebjelkene.
    for x in collars:
        for t in (-0.6, 0.0, 0.6):
            if abs(x - (-1.8)) < 0.4 or abs(x - 0.4) < 0.4:
                continue
            herbs(x, yc + t * cw, zc - 0.09, rng.uniform(0.25, 0.38), herbmats[int((x + t) * 7) % 4], rng, p.trim)
    # Apotekerens seng og ei kiste ved den vestre gavlen.
    bx, by = AIX0 + 0.55, yc + 0.0
    wool = layer("wool", (0.45, 0.55, 0.7))
    # Senga i plassen mellom tørkebedene ved gavlen.
    with piece("bed_apothecary", (bx, by, AF1)):
        box("bed_frame", (bx - 0.45, by - 1.0, AF1), (bx + 0.45, by + 1.0, AF1 + 0.32), p.timber, bevel=0.03)
        box("bed_straw", (bx - 0.4, by - 0.95, AF1 + 0.32), (bx + 0.4, by + 0.95, AF1 + 0.44), p.straw, bevel=0.04)
        box("bed_blanket", (bx - 0.42, by - 0.95, AF1 + 0.4), (bx + 0.42, by + 0.35, AF1 + 0.5), wool, bevel=0.04)
        box("bed_pillow", (bx - 0.3, by + 0.5, AF1 + 0.42), (bx + 0.3, by + 0.9, AF1 + 0.56), layer("linen"), bevel=0.06)
        collide((bx, by, AF1 + 0.3), (0.9, 2.0, 0.6))
    # Kurver og sekker ved trapphullet.
    basket = layer("wattle", (1.0, 0.9, 0.75))
    for k, (x, y) in enumerate(((1.2, uy0 + 1.25), (1.75, uy0 + 1.1), (0.9, 2.0))):
        with piece(f"herb_basket_{k}", (x, y, AF1)):
            revolve("basket", [(0.17, 0.0), (0.24, 0.3), (0.0, 0.3)], basket, n=8, at=(x, y, AF1))
            revolve("basket_herbs", [(0.22, 0.0), (0.12, 0.08), (0.0, 0.1)], herbmats[k % 4], n=7, at=(x, y, AF1 + 0.3))
            collide((x, y, AF1 + 0.2), (0.5, 0.5, 0.4))
    with piece("sacks_two", (-1.25, AIY1 - 1.07, AF1)):
        for k, (x, y) in enumerate(((-1.5, AIY1 - 1.1), (-1.0, AIY1 - 1.05))):
            box("sack", (x - 0.24, y - 0.2, AF1), (x + 0.24, y + 0.2, AF1 + 0.6), layer("linen", (0.85, 0.78, 0.62)), bevel=0.12)
        collide((-1.25, AIY1 - 1.07, AF1 + 0.3), (1.0, 0.45, 0.6))
    light((-1.5, yc, AF1 + 1.9), (1.0, 0.7, 0.42), 1.0, 6.0)
    light((1.6, yc, AF1 + 1.9), (1.0, 0.7, 0.42), 0.8, 5.0)


# --- Bygging --------------------------------------------------------------------


def build_one(name, outside, insides, paint):
    """Utsida og innsida hver for seg; kolliderne, lysene og røyken fra
    begge skrives til én `<navn>_inside.json` til slutt."""
    rng = random.Random(1350 + len(name))
    real_write = town.write_inside
    town.write_inside = lambda path: None
    clear()
    town.PIECES.clear()
    town.LIT = True
    p = paint()
    outside(p, rng)
    town.finish(f"{OUT}/{name}.glb", max_edge=100.0)
    clear()
    town.PIECES.clear()
    p = paint()
    for f in insides:
        f(p, rng)
    town.finish(f"{OUT}/{name}_interior.glb", max_edge=100.0)
    town.write_inside = real_write
    real_write(f"{OUT}/{name}.glb")
    # Møblene: stedene i `assets/layout/inside/<navn>.json`, og hvert møbel
    # som egen modell.
    town.write_contents(name)


def build():
    which = sys.argv[1:] or ["forge", "apothecary"]
    if "forge" in which:
        build_one("forge", forge_outside, [forge_inside], lambda: kit.Paint(plaster="plaster_old", timber="timber", roof="tiles"))
    if "apothecary" in which:
        build_one("apothecary", apothecary_outside, [apothecary_ground, apothecary_upper], lambda: kit.Paint(plaster="plaster_white", timber="timber", roof="slate"))


if __name__ == "__main__":
    build()
