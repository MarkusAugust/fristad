"""Gatelivet i den malte stilen (docs/ny-stil.md), med byggesettet (kit.py).
Kjør:

    python3 tools/models/streetlife.py [well chest ...]

Erstatter modellene fra props.py og street.py med samme navn, mål og
origo, så byen (`town.rs`) og kjelleren ikke må endres:

- `market_cross`: torgkorset med ringkors på et skaft, sokkel og tre
  åttekantede trinn, og en oppslagstavle.
- `market_stall`, `market_stall_b`: markedsboder med disk og varer, den
  ene med bordtak og frynsekappe, den andre med stripet seilduk.
- `cart`: tohjulskjerre med store eikehjul, skjæker og last.
- `barrel`, `crate`: tønne og kasse.
- `lantern_post`: lyktestolpe med smijernslykt (hornruter `glow_horn`,
  flamme `glow`); byen setter lyset 2,68 m oppe.
- `training_dummy`, `weapon_rack`: Gorveks øvingsplass.
- `well`: brønnen med kum, stolper og tak; `well_winch` (origo i aksen
  langs x, valsen har radius `WELL_DRUM` 0,08), `well_rope` (1 m ned fra
  origo, skaleres) og `well_bucket` (origo øverst på bøyla) flyttes av
  økta, med aksen 1,55 m oppe og tauet 9 cm foran (town.rs, `WELL_*`).
- `woodpile`: vedstabel mellom to staker med tak og hoggestabbe.
- `signpost`: veiskiltet til krypten (pila peker langs lokal +X).
- `chest`, `chest_lid`: kista i kjelleren; lokket har origo i hengslet
  (`crypt::CHEST_HINGE`, bakre øvre kant).

Akser som i town.py: X til høyre, −Y fram, Z opp, origo midt på
grunnflaten. Ingen kollidere her: byen setter dem selv."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
import kit  # noqa: E402
import town  # noqa: E402
from common import clear  # noqa: E402
from kit import box, stick  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from town import PIECES, layer, plain  # noqa: E402

OUT = "assets/models/town"
CARVED, DARK_BEAM, MOULDING, SILL, BRICKS, IRON_BAND, ROUGH_PLANK, ROPE = range(8)


class Mats:
    def __init__(self):
        self.timber = layer("timber")
        self.grey = layer("timber_grey")
        self.boards = layer("weatherboard")
        self.trim = layer("trim")
        self.iron = layer("iron", rough=0.45)
        self.stone = layer("wall_stone")
        self.flag = layer("flagstone")
        self.logs = layer("logs")
        self.thatch = layer("thatch")
        self.straw = layer("straw_floor")
        self.door = layer("door")
        self.shingles = layer("shingles")
        self.linen = layer("linen")
        self.ends = layer("weatherboard", (1.0, 0.86, 0.66))
        self.dark = layer("timber", (0.25, 0.2, 0.18))


# --- Hjelpere -----------------------------------------------------------------


def lathe(name, prof, mat, n=8, at=(0, 0, 0), cap=(True, True), phase=0.0, top=None, closed=False, strip=None, su=1.0, sv=1.0):
    """Dreid form om z: profilen er (r, z) nedenfra. Uv på sidene: u rundt
    (meter ved største radius), v langs profilen. `top` gir flatene som
    vender opp et eget materiale. `closed` lukker profilen (ringer).
    `strip` = ("trim", k) eller ("timber", k) legger en stripe av arket
    over profilen."""
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    rings = []
    for r, z in prof:
        if r <= 1e-5:
            rings.append([bm.verts.new((0, 0, z))])
        else:
            rings.append([bm.verts.new((r * math.cos(phase + 2 * math.pi * i / n), r * math.sin(phase + 2 * math.pi * i / n), z)) for i in range(n)])
    cum = [0.0]
    pairs = list(zip(range(len(prof)), range(1, len(prof))))
    if closed:
        pairs.append((len(prof) - 1, 0))
    for a, b in pairs:
        cum.append(cum[-1] + math.hypot(prof[b][0] - prof[a][0], prof[b][1] - prof[a][1]))
    total = max(cum[-1], 1e-4)
    rmax = max(r for r, _ in prof)
    circ = 2 * math.pi * rmax
    if strip:
        if strip[0] == "trim":
            v0, v1 = 1 - (strip[1] + 1) / 8 + 0.006, 1 - strip[1] / 8 - 0.006
            scale = 0.125 / total
        else:
            v0, v1 = strip[1] * 0.25 + 0.02, strip[1] * 0.25 + 0.23
            scale = 1.0
    for k, (a, b) in enumerate(pairs):
        ra, rb = rings[a], rings[b]
        for i in range(n):
            j = i + 1
            if len(ra) == 1 and len(rb) == 1:
                continue
            if len(ra) == 1:
                vs = [ra[0], rb[j % n], rb[i]]
                idx = [(i + 0.5, k), (j, k + 1), (i, k + 1)]
            elif len(rb) == 1:
                vs = [ra[i], ra[j % n], rb[0]]
                idx = [(i, k), (j, k), (i + 0.5, k + 1)]
            else:
                vs = [ra[i], ra[j % n], rb[j % n], rb[i]]
                idx = [(i, k), (j, k), (j, k + 1), (i, k + 1)]
            try:
                f = bm.faces.new(vs)
            except ValueError:
                continue
            for lp, (ii, kk) in zip(f.loops, idx):
                u = ii / n * circ
                if strip:
                    t = cum[kk] / total
                    lp[uvl].uv = (u * scale * su, v0 + t * (v1 - v0))
                else:
                    lp[uvl].uv = (u * su, cum[kk] * sv)
    if not closed:
        for ring, z_up, want in ((rings[0], False, cap[0]), (rings[-1], True, cap[1])):
            if len(ring) > 2 and want:
                f = bm.faces.new(ring if z_up else ring[::-1])
                for lp in f.loops:
                    lp[uvl].uv = (lp.vert.co.x * su, lp.vert.co.y * su)
    # Ingen recalc: rekkefølgen gir normalen til høyre for profilen (ut
    # når profilen går oppover, opp når den går innover).
    ob = kit._object(name, bm, [mat, top] if top else mat)
    if top:
        for p in ob.data.polygons:
            if p.normal.z > 0.6:
                p.material_index = 1
    ob.location = at
    return ob


def since(i):
    return PIECES[i:]


def move(obs, at=(0, 0, 0), yaw=0.0, pitch=0.0, roll=0.0):
    """Flytt ferdige deler: dreid om x (roll), y (pitch) og z (yaw), så flyttet."""
    bpy.context.view_layer.update()
    m = Matrix.Translation(Vector(at)) @ Matrix.Rotation(yaw, 4, "Z") @ Matrix.Rotation(pitch, 4, "Y") @ Matrix.Rotation(roll, 4, "X")
    for o in obs:
        o.matrix_world = m @ o.matrix_world


def scale_uv(ob, su, sv=None):
    sv = su if sv is None else sv
    for d in ob.data.uv_layers.active.data:
        d.uv = (d.uv[0] * su, d.uv[1] * sv)
    return ob


def swap_uv(ob):
    for d in ob.data.uv_layers.active.data:
        d.uv = (d.uv[1], d.uv[0])
    return ob


def ring(name, r, w, h, mat, n=16, at=(0, 0, 0), strip=None):
    """En ring (bånd) om z med radius r, bredde w ut og høyde h."""
    return lathe(name, [(r - w / 2, -h / 2), (r + w / 2, -h / 2), (r + w / 2, h / 2), (r - w / 2, h / 2)], mat, n=n, at=at, closed=True, strip=strip)


def band(name, r, h, mat, n=16, at=(0, 0, 0), k=IRON_BAND, rr=None):
    """Et jernbånd rundt en tønne: en tynn sylinderflate med stripen fra
    trim-arket."""
    rr = r if rr is None else rr
    return lathe(name, [(r, -h / 2), (rr + 0.012, 0.0), (r, h / 2)], mat, n=n, at=at, cap=(False, False), strip=("trim", k))


def slab(name, pts, thick, mat, uv_axes=("x", "y"), uvs=1.0):
    """En flat plate fra et polygon (x, y) med tykkelse ned fra z = 0 i
    lokale akser."""
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    top = [bm.verts.new((x, y, 0.0)) for x, y in pts]
    bot = [bm.verts.new((x, y, -thick)) for x, y in pts]
    bm.faces.new(top)
    bm.faces.new(bot[::-1])
    n = len(pts)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((bot[i], bot[j], top[j], top[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        for lp in f.loops:
            c = lp.vert.co
            if abs(f.normal.z) > 0.5:
                lp[uvl].uv = (c.x * uvs, c.y * uvs) if uv_axes[0] == "x" else (c.y * uvs, c.x * uvs)
            else:
                lp[uvl].uv = ((c.x + c.y) * uvs, c.z * uvs)
    return kit._object(name, bm, mat)


def letters(text, at, size, mat, back=False):
    """Bokstaver fra Blenders skrift, gjort om til mesh, med forsida mot −Y
    (eller +Y med `back`)."""
    bpy.ops.object.text_add(location=(0, 0, 0))
    ob = bpy.context.active_object
    ob.data.body = text
    ob.data.align_x = "CENTER"
    ob.data.align_y = "CENTER"
    ob.data.size = size
    ob.data.extrude = 0.008
    ob.data.resolution_u = 2
    bpy.ops.object.convert(target="MESH")
    ob = bpy.context.active_object
    ob.rotation_euler = (math.pi / 2, 0, math.pi if back else 0)
    ob.location = at
    ob.data.materials.clear()
    ob.data.materials.append(mat)
    kit.ensure_uv(ob.data)
    town.uv_box(ob)
    PIECES.append(ob)
    return ob


def pyramid(name, w, h, mat, at, n=4, phase=math.pi / 4):
    return lathe(name, [(w / math.sqrt(2), 0.0), (0.0, h)], mat, n=n, at=at, phase=phase)


# --- Små ting som går igjen ---------------------------------------------------


def barrel_at(m, at=(0, 0, 0), r=0.32, h=0.86, n=12, lying=False, yaw=0.0):
    """Tønne med kraftig buk, staver (timber-laget på høykant), lokk og to
    jernbånd med nagler (trim-arket)."""
    i = len(PIECES)
    prof = [(r * 0.8, 0.0), (r * 0.94, h * 0.2), (r, h * 0.5), (r * 0.94, h * 0.8), (r * 0.8, h)]
    body = lathe("barrel", prof, m.timber, n=n, cap=(True, False))
    # Stavene står på høykant: bjelkene i laget går rundt, så u og v byttes.
    swap_uv(body)
    scale_uv(body, 1.0, 2.0)
    lathe("barrel_lid", [(r * 0.8, h), (r * 0.74, h - 0.03), (0.0, h - 0.03)], m.ends, n=n, cap=(False, False))
    for t in (0.14, 0.86):
        rr = r * (0.8 + 0.2 * math.sin(math.pi * t) ** 0.6)
        band("barrel_band", rr + 0.006, 0.07, m.trim, n=n, at=(0, 0, h * t))
    obs = since(i)
    if lying:
        move(obs, (0, 0, -h / 2))
        move(obs, (at[0], at[1], at[2] + r), yaw=yaw, pitch=math.pi / 2)
    else:
        move(obs, at, yaw=yaw)
    return obs


def crate_at(m, at=(0, 0, 0), s=0.72, yaw=0.0, lid=True):
    """Kasse av bord med tykke hjørnelekter, skråstag foran og bak."""
    i = len(PIECES)
    h = s
    core = box("crate", (-s / 2 + 0.03, -s / 2 + 0.03, 0.0), (s / 2 - 0.03, s / 2 - 0.03, h - 0.02), m.boards, bevel=0.01)
    scale_uv(core, 1.4)
    e = 0.075
    for sx in (-1, 1):
        for sy in (-1, 1):
            stick("crate_post", (sx * (s / 2 - e / 2), sy * (s / 2 - e / 2), 0.0), (sx * (s / 2 - e / 2), sy * (s / 2 - e / 2), h), e, e, m.timber, ("timber", (sx + sy + 2) % 4), bevel=0.015)
    for z in (e / 2, h - e / 2):
        for sy in (-1, 1):
            stick("crate_rail", (-s / 2 + e, sy * (s / 2 - e / 2), z), (s / 2 - e, sy * (s / 2 - e / 2), z), e, e * 0.8, m.timber, ("timber", 1), bevel=0.012)
        for sx in (-1, 1):
            stick("crate_rail", (sx * (s / 2 - e / 2), -s / 2 + e, z), (sx * (s / 2 - e / 2), s / 2 - e, z), e * 0.8, e, m.timber, ("timber", 2), bevel=0.012)
    for sy in (-1, 1):
        stick("crate_brace", (-s / 2 + e, sy * (s / 2 - 0.02), e), (s / 2 - e, sy * (s / 2 - 0.02), h - e), 0.06, 0.035, m.timber, ("timber", 3), bevel=0.01)
    if lid:
        lid_ob = box("crate_lid", (-s / 2 + 0.02, -s / 2 + 0.02, h - 0.02), (s / 2 - 0.02, s / 2 - 0.02, h + 0.015), m.boards, bevel=0.01)
        scale_uv(lid_ob, 1.4)
    obs = since(i)
    move(obs, at, yaw=yaw)
    return obs


def sack_at(m, at, r=0.18, h=0.45, mat=None, yaw=0.0, tilt=0.0):
    """En fylt sekk med knyting på toppen."""
    i = len(PIECES)
    lathe("sack", [(r * 0.7, 0.0), (r, h * 0.15), (r * 1.05, h * 0.45), (r * 0.85, h * 0.75), (r * 0.35, h * 0.9), (r * 0.25, h), (r * 0.35, h * 1.12), (0.0, h * 1.14)], mat or m.linen, n=10, cap=(True, False), su=1.0)
    ring("sack_tie", r * 0.28, 0.025, 0.03, m.trim, n=8, at=(0, 0, h * 0.94), strip=("trim", ROPE))
    obs = since(i)
    move(obs, at, yaw=yaw, roll=tilt)
    return obs


# --- Torget -------------------------------------------------------------------


def market_cross():
    m = Mats()
    # Tre åttekantede trinn med avfaset kant: hellene på toppen, mur på sidene.
    for (r, z0, z1) in ((1.3, -0.2, 0.2), (1.0, 0.2, 0.42), (0.74, 0.42, 0.64)):
        lathe("step", [(r, z0), (r, z1 - 0.05), (r - 0.05, z1), (0.0, z1)], m.stone, n=8, phase=math.pi / 8, top=m.flag)
    # Sokkelen: en firkantet kloss med skrå topp og en list nede.
    box("plinth_foot", (-0.46, -0.46, 0.64), (0.46, 0.46, 0.78), m.flag, bevel=0.04)
    box("plinth", (-0.38, -0.38, 0.78), (0.38, 0.38, 1.22), m.stone, bevel=0.03)
    lathe("plinth_top", [(0.54, 1.22), (0.54, 1.27), (0.24, 1.42), (0.0, 1.42)], m.flag, n=4, phase=math.pi / 4)
    # Skaftet smalner av oppover, med en krage.
    lathe("shaft", [(0.2, 1.38), (0.15, 3.6), (0.0, 3.6)], m.stone, n=8, phase=math.pi / 8)
    ring("collar", 0.2, 0.1, 0.12, m.flag, n=8, at=(0, 0, 3.0))
    # Ringkorset: armer som vider seg ut i endene, og ringen rundt krysset.
    zc = 3.88
    box("cross_v", (-0.13, -0.12, 3.5), (0.13, 0.12, 4.12), m.stone, bevel=0.03)
    box("cross_h", (-0.5, -0.1, zc - 0.12), (0.5, 0.1, zc + 0.12), m.stone, bevel=0.03)
    for sx in (-1, 1):
        box("cross_end", (sx * 0.5 - 0.07, -0.14, zc - 0.19), (sx * 0.5 + 0.07, 0.14, zc + 0.19), m.stone, bevel=0.04)
    box("cross_top", (-0.19, -0.14, 4.12), (0.19, 0.14, 4.3), m.stone, bevel=0.04)
    hoop = ring("cross_ring", 0.3, 0.09, 0.14, m.flag, n=16)
    move([hoop], (0, 0, zc), roll=math.pi / 2)
    # Oppslagstavla på skaftet, med lapper og to jernbånd.
    i = len(PIECES)
    box("notice_board", (-0.3, -0.05, -0.35), (0.3, 0.0, 0.35), m.grey, bevel=0.015)
    for z in (-0.22, 0.22):
        box("notice_strap", (-0.33, -0.065, z - 0.03), (0.33, -0.04, z + 0.03), m.iron, bevel=0.006)
    paper = layer("linen", (1.0, 1.0, 0.95))
    for x, z, w, h, a in ((-0.12, 0.03, 0.2, 0.26, 0.05), (0.13, -0.05, 0.18, 0.2, -0.08), (0.06, 0.2, 0.14, 0.1, 0.1)):
        ob = box("notice", (-w / 2, -0.06, -h / 2), (w / 2, -0.05, h / 2), paper, bevel=0.0)
        move([ob], (x, 0, z), pitch=a)
    move(since(i), (0, -0.17, 2.05))
    # Jernringen til å binde dyr, foran på sokkelen.
    tie = ring("tie_ring", 0.09, 0.025, 0.025, m.iron, n=10)
    move([tie], (0, -0.4, 0.93), roll=math.pi / 2)
    box("tie_plate", (-0.06, -0.4, 0.98), (0.06, -0.38, 1.08), m.iron, bevel=0.01)


def stall_frame(m, rng):
    """Stolpene, disken og bakhylla som begge bodene har."""
    W, D = 2.6, 1.24
    yf, yb = -0.62, 0.62
    for sx in (-1, 1):
        for y, top in ((yf, 2.12), (yb, 2.5)):
            stick("stall_post", (sx * 1.22, y, -0.1), (sx * 1.22 + rng.uniform(-0.02, 0.02), y, top), 0.17, 0.17, m.timber, ("timber", rng.randrange(4)), bevel=0.035)
        # Knekter under takbjelken.
        stick("stall_knee", (sx * 1.22, yf, 1.7), (sx * 0.85, yf, 2.06), 0.1, 0.1, m.timber, ("timber", 2), bevel=0.02)
    for y, top in ((yf, 2.12), (yb, 2.5)):
        stick("stall_plate", (-1.38, y, top), (1.38, y, top), 0.17, 0.2, m.timber, ("timber", 1), bevel=0.035)
    # Disken: forpanel av bord, tykk plate som stikker ut, og sider.
    front = box("counter_front", (-1.15, yf - 0.02, 0.0), (1.15, yf + 0.06, 0.88), m.boards, bevel=0.015)
    scale_uv(front, 1.2)
    box("counter_top", (-1.32, yf - 0.16, 0.88), (1.32, -0.12, 0.98), m.grey, bevel=0.03)
    for sx in (-1, 1):
        box("counter_side", (sx * 1.15 - 0.04, yf, 0.0), (sx * 1.15 + 0.04, -0.16, 0.88), m.boards, bevel=0.012)
    for x in (-0.6, 0.0, 0.6):
        stick("counter_batten", (x, yf - 0.035, 0.05), (x, yf - 0.035, 0.85), 0.09, 0.04, m.timber, ("timber", 3), bevel=0.012)
    # Bakhylla.
    box("shelf", (-1.1, 0.25, 0.6), (1.1, 0.6, 0.66), m.grey, bevel=0.02)
    for sx in (-1, 1):
        stick("shelf_leg", (sx * 1.0, 0.42, 0.0), (sx * 1.0, 0.42, 0.6), 0.09, 0.09, m.timber, ("timber", 0), bevel=0.015)
    return yf, yb


def scallops(name, x0, x1, y, z, n, depth, mats):
    """Frynsekappe langs fronten: flapper med rund bue nede, annenhver
    farge."""
    for k in range(n):
        xa = x0 + (x1 - x0) * k / n
        xb = x0 + (x1 - x0) * (k + 1) / n
        pts = [(xa, 0.0), (xb, 0.0)]
        for t in range(1, 6):
            a = math.pi * t / 6
            pts.append((xb - (xb - xa) * (1 - math.cos(a)) / 2, -depth * 0.55 - depth * 0.45 * math.sin(a)))
        pts.append((xa, -depth * 0.55))
        ob = slab(name, [(px, pz) for px, pz in pts], 0.02, mats[k % len(mats)])
        # Platen ligger i xy: reis den opp i xz-planet ved fronten.
        move([ob], (0, y, z), roll=math.pi / 2)


def market_stall():
    m = Mats()
    rng = random.Random(5)
    yf, yb = stall_frame(m, rng)
    # Bordtaket: en tykk plate av liggende bord fra fronten og opp bakover.
    i = len(PIECES)
    roof = box("stall_roof", (-1.45, -1.06, -0.06), (1.45, 0.82, 0.06), m.boards, bevel=0.03)
    scale_uv(roof, 1.1)
    slope = math.atan2(0.38, 1.24)
    move(since(i), (0, -0.12, 2.36), roll=slope)
    for x in (-1.45, 1.45):
        stick("roof_edge", (x, -1.06, 2.36 - 0.94 * math.sin(slope) + 0.06), (x, 0.82, 2.36 + 0.94 * math.sin(slope) + 0.06), 0.08, 0.1, m.timber, ("timber", 0), bevel=0.02)
    red = layer("linen", (0.78, 0.2, 0.12))
    cream = layer("linen", (1.0, 0.95, 0.8))
    scallops("valance", -1.45, 1.45, -1.08, 2.0, 7, 0.32, [red, cream])
    # Varer: ostehjul, kasser, sekker og en tønne.
    cheese = layer("linen", (1.0, 0.72, 0.3))
    for x, z, r in ((-0.95, 0.98, 0.17), (-0.95, 1.1, 0.15), (-0.6, 0.98, 0.16)):
        lathe("cheese", [(r * 0.92, z), (r, z + 0.03), (r, z + 0.08), (r * 0.92, z + 0.11), (0.0, z + 0.11)], cheese, n=10, at=(x, -0.42, 0), cap=(True, False))
    crate_at(m, (0.05, -0.38, 0.98), s=0.36, yaw=0.15, lid=False)
    apples = layer("linen", (0.85, 0.22, 0.12))
    for k in range(5):
        a = k * 1.25
        lathe("apple", [(0.0, 0.0), (0.05, 0.02), (0.055, 0.06), (0.03, 0.09), (0.0, 0.085)], apples, n=6, at=(0.05 + math.cos(a) * 0.08, -0.38 + math.sin(a) * 0.08, 1.31 - 0.05 * (k % 2)))
    sack_at(m, (0.7, -0.4, 0.98), r=0.15, h=0.32)
    sack_at(m, (0.98, -0.32, 0.98), r=0.13, h=0.28, yaw=1.0)
    crate_at(m, (-0.6, 0.42, 0.66), s=0.4, yaw=-0.1)
    barrel_at(m, (0.55, 0.43, 0.66), r=0.18, h=0.42, n=10)
    barrel_at(m, (1.6, 0.25, 0.0), r=0.28, h=0.72, n=12)


def market_stall_b():
    m = Mats()
    rng = random.Random(6)
    yf, yb = stall_frame(m, rng)
    # Seilduken: stripet, spent fra bakbjelken over frontbjelken og ut, og
    # henger litt mellom stolpene.
    red = layer("linen", (0.72, 0.16, 0.12))
    cream = layer("linen", (1.0, 0.96, 0.82))
    nx, ny = 8, 5
    x0, x1 = -1.48, 1.48
    ya, yb2 = 0.72, -1.1
    za, zb = 2.72, 2.32
    for side, t in ((0, 0.0), (1, -0.02)):
        bm = bmesh.new()
        uvl = bm.loops.layers.uv.verify()
        g = {}
        for i in range(nx + 1):
            for j in range(ny + 1):
                u, v = i / nx, j / ny
                x = x0 + (x1 - x0) * u
                y = ya + (yb2 - ya) * v
                z = za + (zb - za) * v - 0.1 * math.sin(math.pi * v) * (0.6 + 0.4 * math.sin(math.pi * u)) - 0.05 * math.sin(math.pi * u * 2) ** 2 * v
                if v > 0.72:
                    # Ytterst foran knekker duken ned over frontbjelken.
                    z -= (v - 0.72) * 0.5
                g[(i, j)] = bm.verts.new((x, y, z + t))
        for i in range(nx):
            for j in range(ny):
                q = [g[(i, j)], g[(i + 1, j)], g[(i + 1, j + 1)], g[(i, j + 1)]]
                f = bm.faces.new(q[::-1] if side == 0 else q)
                f.material_index = i % 2
                for lp in f.loops:
                    c = lp.vert.co
                    lp[uvl].uv = (c.x * 2.0, c.y * 2.0)
        kit._object("awning", bm, [red, cream])
    scallops("valance", x0, x1, yb2 + 0.01, zb - 0.28 * 0.5 + 0.01, 8, 0.3, [red, cream])
    # Tøyruller, ei kurv med brød og en kasse.
    for k, (x, tint) in enumerate(((-1.0, (0.3, 0.42, 0.85)), (-0.72, (0.35, 0.6, 0.3)), (-0.44, (0.95, 0.75, 0.3)))):
        ob = lathe("bolt", [(0.0, -0.32), (0.1, -0.32), (0.11, 0.0), (0.1, 0.32), (0.0, 0.32)], layer("wool", tint), n=10)
        move([ob], (x, -0.4, 1.09), roll=math.pi / 2)
    i = len(PIECES)
    lathe("basket", [(0.17, 0.0), (0.26, 0.16), (0.24, 0.16), (0.15, 0.03), (0.0, 0.03)], m.straw, n=12, cap=(True, False))
    ring("basket_rim", 0.255, 0.04, 0.04, m.trim, n=12, at=(0, 0, 0.16), strip=("trim", ROPE))
    bread = layer("weatherboard", (1.0, 0.78, 0.5))
    for k, (dx, dy) in enumerate(((0.0, 0.0), (0.1, 0.06), (-0.09, 0.07), (0.05, -0.1), (-0.08, -0.07))):
        ob = lathe("loaf", [(0.0, 0.0), (0.075, 0.01), (0.08, 0.05), (0.05, 0.09), (0.0, 0.1)], bread, n=8)
        ob.scale = (1.3, 1.0, 0.9)
        move([ob], (dx, dy, 0.1 + 0.03 * (k % 2)), yaw=k * 0.7)
    move(since(i), (0.45, -0.4, 0.98))
    crate_at(m, (1.0, -0.36, 0.98), s=0.3, yaw=-0.2, lid=False)
    for k in range(3):
        lathe("apple", [(0.0, 0.0), (0.05, 0.02), (0.055, 0.06), (0.03, 0.09), (0.0, 0.085)], layer("linen", (0.5, 0.75, 0.2)), n=6, at=(1.0 + (k - 1) * 0.08, -0.36, 1.25))
    crate_at(m, (0.5, 0.42, 0.66), s=0.42, yaw=0.1)
    sack_at(m, (-0.55, 0.42, 0.66), r=0.17, h=0.4)
    sack_at(m, (-1.65, -0.1, 0.0), r=0.22, h=0.55, yaw=0.5)


# --- Kjerra -------------------------------------------------------------------


def wheel(m, y, R=0.55, z=0.55):
    """Eikehjul i xz-planet ved y: nav, åtte eiker, felg og jernring."""
    i = len(PIECES)
    lathe("hub", [(0.0, -0.13), (0.09, -0.13), (0.12, -0.05), (0.12, 0.05), (0.09, 0.13), (0.0, 0.13)], m.timber, n=8)
    ring("felloe", R - 0.06, 0.1, 0.1, m.timber, n=16)
    band("tyre", R - 0.005, 0.1, m.trim, n=16)
    for k in range(8):
        a = 2 * math.pi * k / 8 + math.pi / 8
        stick("spoke", (0.1 * math.cos(a), 0.1 * math.sin(a), 0), ((R - 0.1) * math.cos(a), (R - 0.1) * math.sin(a), 0), 0.06, 0.045, m.timber, ("timber", k % 4), bevel=0.012)
    move(since(i), (0, y, z), roll=math.pi / 2)


def cart():
    m = Mats()
    L, W = 2.1, 1.1
    z0 = 0.62
    # Lasteplanet: tykk bunn, sidebrett med staker og en bakfjel.
    floor = box("cart_floor", (-L / 2, -W / 2, z0 + 0.12), (L / 2, W / 2, z0 + 0.2), m.grey, bevel=0.02)
    scale_uv(floor, 1.2)
    for sy in (-1, 1):
        stick("cart_beam", (-L / 2 - 0.05, sy * 0.35, z0 + 0.06), (L / 2 + 0.05, sy * 0.35, z0 + 0.06), 0.12, 0.12, m.timber, ("timber", 1), bevel=0.025)
        side = box("cart_side", (-L / 2, sy * W / 2 - 0.035, z0 + 0.2), (L / 2, sy * W / 2 + 0.035, z0 + 0.55), m.boards, bevel=0.015)
        scale_uv(side, 1.3)
        stick("cart_rail", (-L / 2 - 0.04, sy * (W / 2 + 0.03), z0 + 0.56), (L / 2 + 0.04, sy * (W / 2 + 0.03), z0 + 0.56), 0.08, 0.07, m.timber, ("timber", 2), bevel=0.02)
        for k in range(4):
            x = -L / 2 + 0.06 + (L - 0.12) * k / 3
            stick("cart_stake", (x, sy * (W / 2 + 0.04), z0 + 0.08), (x, sy * (W / 2 + 0.04), z0 + 0.62), 0.07, 0.06, m.timber, ("timber", k % 4), bevel=0.015)
    box("cart_back", (-L / 2 - 0.04, -W / 2 + 0.03, z0 + 0.2), (-L / 2 + 0.03, W / 2 - 0.03, z0 + 0.5), m.boards, bevel=0.015)
    # Skjækene fram og en tverrslå, og støttebeinet.
    for sy in (-1, 1):
        stick("cart_shaft", (L / 2 - 0.3, sy * 0.35, z0 + 0.08), (L / 2 + 1.12, sy * 0.24, z0 - 0.02), 0.09, 0.09, m.timber, ("timber", 0), bevel=0.02)
    stick("cart_bar", (L / 2 + 0.95, -0.3, z0), (L / 2 + 0.95, 0.3, z0), 0.07, 0.07, m.timber, ("timber", 3), bevel=0.015)
    stick("cart_prop", (L / 2 + 0.75, 0.0, z0 - 0.02), (L / 2 + 0.85, 0.0, 0.0), 0.07, 0.07, m.timber, ("timber", 2), bevel=0.015)
    # Akselen og hjulene.
    stick("axle", (0, -W / 2 - 0.22, 0.55), (0, W / 2 + 0.22, 0.55), 0.1, 0.1, m.timber, ("timber", 1), bevel=0.02)
    for sy in (-1, 1):
        stick("axle_block", (-0.25, sy * 0.35, 0.6), (0.25, sy * 0.35, 0.6), 0.12, 0.12, m.timber, ("timber", 2), bevel=0.02)
        wheel(m, sy * (W / 2 + 0.13))
    # Lasten: ei tønne på siden, en kasse og to sekker.
    barrel_at(m, (-0.55, 0.1, z0 + 0.2), r=0.24, h=0.62, n=10, lying=True, yaw=0.2)
    crate_at(m, (0.25, 0.18, z0 + 0.2), s=0.42, yaw=0.25)
    sack_at(m, (0.55, -0.25, z0 + 0.2), r=0.17, h=0.45, yaw=0.3, tilt=0.15)
    sack_at(m, (0.05, -0.28, z0 + 0.2), r=0.16, h=0.4, yaw=1.2, tilt=-0.2)


def barrel():
    m = Mats()
    barrel_at(m, r=0.33, h=0.86, n=14)


def crate():
    m = Mats()
    crate_at(m, s=0.72)


# --- Lykta --------------------------------------------------------------------


def lantern_post():
    m = Mats()
    horn = plain("glow_horn", (0.3, 0.19, 0.08), 0.5)
    flame = plain("glow", (1.0, 0.72, 0.38), 0.4)
    wax = layer("linen", (1.0, 0.95, 0.8))
    # Foten av hogd stein, og stolpen med avfasede kanter og en krage.
    lathe("lamp_foot", [(0.3, -0.15), (0.3, 0.14), (0.22, 0.26), (0.13, 0.3), (0.0, 0.3)], m.stone, n=8, phase=math.pi / 8, top=m.flag)
    stick("lamp_pole", (0, 0, 0.2), (0, 0, 2.38), 0.18, 0.18, m.timber, ("timber", 1), bevel=0.04, roll=0.0)
    box("lamp_collar", (-0.13, -0.13, 2.3), (0.13, 0.13, 2.42), m.timber, bevel=0.025)
    box("lamp_band", (-0.105, -0.105, 0.55), (0.105, 0.105, 0.63), m.iron, bevel=0.01)
    # Knekter av smijern som bærer lykta, med snirkler.
    for a in range(4):
        d = Vector((math.cos(a * math.pi / 2), math.sin(a * math.pi / 2), 0))
        p0 = d * 0.09 + Vector((0, 0, 2.18))
        p1 = d * 0.2 + Vector((0, 0, 2.46))
        stick("lamp_strut", tuple(p0), tuple(p1), 0.025, 0.025, m.iron, bevel=0)
        stick("lamp_curl", tuple(p1), tuple(d * 0.25 + Vector((0, 0, 2.4))), 0.022, 0.022, m.iron, bevel=0)
    # Lykta: bunn, fire hjørnestolper, hornruter, tak og ring.
    z0, h, s = 2.46, 0.38, 0.17
    box("lamp_base", (-s - 0.03, -s - 0.03, z0), (s + 0.03, s + 0.03, z0 + 0.05), m.iron, bevel=0.012)
    box("lamp_head", (-s - 0.02, -s - 0.02, z0 + h), (s + 0.02, s + 0.02, z0 + h + 0.04), m.iron, bevel=0.01)
    for sx in (-1, 1):
        for sy in (-1, 1):
            box("lamp_corner", (sx * s - 0.022, sy * s - 0.022, z0 + 0.04), (sx * s + 0.022, sy * s + 0.022, z0 + h), m.iron, bevel=0.006)
    for k in range(4):
        a = k * math.pi / 2
        d = Vector((math.cos(a), math.sin(a), 0))
        if k % 2 == 0:
            box("lamp_horn", (d.x * s - 0.004, -s + 0.02, z0 + 0.05), (d.x * s + 0.004, s - 0.02, z0 + h), horn, bevel=0)
            box("lamp_bar", (d.x * (s + 0.008) - 0.008, -s, z0 + h * 0.55 - 0.012), (d.x * (s + 0.008) + 0.008, s, z0 + h * 0.55 + 0.012), m.iron, bevel=0.003)
        else:
            box("lamp_horn", (-s + 0.02, d.y * s - 0.004, z0 + 0.05), (s - 0.02, d.y * s + 0.004, z0 + h), horn, bevel=0)
            box("lamp_bar", (-s, d.y * (s + 0.008) - 0.008, z0 + h * 0.55 - 0.012), (s, d.y * (s + 0.008) + 0.008, z0 + h * 0.55 + 0.012), m.iron, bevel=0.003)
    lathe("lamp_candle", [(0.04, z0 + 0.05), (0.04, z0 + 0.15), (0.0, z0 + 0.15)], wax, n=8)
    lathe("lamp_flame", [(0.0, z0 + 0.15), (0.035, z0 + 0.2), (0.025, z0 + 0.25), (0.0, z0 + 0.3)], flame, n=6)
    pyramid("lamp_roof", 2 * s + 0.12, 0.2, m.iron, (0, 0, z0 + h + 0.04))
    ring("lamp_ring", 0.045, 0.018, 0.018, m.iron, n=8)
    move([PIECES[-1]], (0, 0, z0 + h + 0.28), roll=math.pi / 2)


# --- Øvingsplassen ------------------------------------------------------------


def training_dummy():
    m = Mats()
    rng = random.Random(9)
    stick("dummy_post", (0, 0, -0.2), (0, 0, 1.78), 0.16, 0.16, m.timber, ("timber", 1), bevel=0.035)
    stick("dummy_arm", (-0.66, 0, 1.38), (0.6, 0, 1.38), 0.12, 0.12, m.timber, ("timber", 2), bevel=0.03)
    # Kroppen: en halmbylt surret med tau, og et sekkehode.
    lathe("dummy_body", [(0.17, 0.78), (0.28, 0.86), (0.33, 1.08), (0.34, 1.32), (0.27, 1.52), (0.15, 1.6), (0.0, 1.62)], m.thatch, n=10, cap=(True, False))
    for z, r in ((0.9, 0.3), (1.15, 0.345), (1.4, 0.32)):
        ring("dummy_rope", r, 0.04, 0.045, m.trim, n=10, at=(0, 0, z), strip=("trim", ROPE))
    sack = layer("linen", (0.95, 0.85, 0.7))
    lathe("dummy_head", [(0.06, 1.58), (0.15, 1.64), (0.18, 1.74), (0.15, 1.86), (0.09, 1.92), (0.12, 1.98), (0.0, 1.96)], sack, n=8, cap=(False, False))
    ring("dummy_neck", 0.09, 0.035, 0.04, m.trim, n=8, at=(0, 0, 1.62), strip=("trim", ROPE))
    # Øynene er to svarte kryss malt på sekken.
    soot = layer("timber", (0.08, 0.06, 0.05))
    for sx in (-1, 1):
        for a in (0.7, -0.7):
            ob = box("dummy_eye", (-0.04, -0.008, -0.009), (0.04, 0.0, 0.009), soot, bevel=0)
            move([ob], (sx * 0.065, -0.175, 1.77), pitch=a)
    # Skjoldet på den venstre armen, med jernkant og bule.
    i = len(PIECES)
    paint = layer("weatherboard", (0.75, 0.3, 0.22))
    lathe("dummy_shield", [(0.0, -0.025), (0.3, -0.025), (0.3, 0.025), (0.0, 0.025)], paint, n=14)
    ring("dummy_rim", 0.3, 0.035, 0.06, m.iron, n=14)
    lathe("dummy_boss", [(0.08, 0.025), (0.07, 0.06), (0.0, 0.08)], m.iron, n=8, cap=(False, False))
    move(since(i), (-0.72, 0, 1.28), pitch=-math.pi / 2)
    # Tre skråstøtter i bakken.
    for k in range(3):
        a = k * 2.1 + 0.4
        stick("dummy_foot", (0, 0, 0.5), (math.cos(a) * 0.45, math.sin(a) * 0.45, 0.0), 0.08, 0.08, m.timber, ("timber", k), bevel=0.02)
    # Et par hogg i stolpen.
    for k in range(3):
        ob = box("dummy_cut", (-0.06, -0.085, -0.01), (0.06, -0.075, 0.01), soot, bevel=0)
        move([ob], (0, 0, 0.3 + k * 0.13 + rng.uniform(-0.03, 0.03)), pitch=rng.uniform(-0.4, 0.4))


def weapon_rack():
    m = Mats()
    rng = random.Random(11)
    # To bukker med en tverrslå oppe og to lister nede med hull til skaftene.
    for sx in (-1, 1):
        for sy in (-1, 1):
            stick("rack_leg", (sx * 0.95, sy * 0.3, 0.0), (sx * 0.95, sy * 0.04, 1.55), 0.1, 0.1, m.timber, ("timber", (sx + sy + 2) % 4), bevel=0.025)
        stick("rack_tie", (sx * 0.95, -0.22, 0.5), (sx * 0.95, 0.22, 0.5), 0.07, 0.07, m.timber, ("timber", 3), bevel=0.015)
    stick("rack_top", (-1.12, 0, 1.5), (1.12, 0, 1.5), 0.12, 0.12, m.timber, ("timber", 1), bevel=0.03)
    for sy in (-1, 1):
        stick("rack_low", (-1.08, sy * 0.2, 0.45), (1.08, sy * 0.2, 0.45), 0.09, 0.08, m.timber, ("timber", 2), bevel=0.02)
    leather = layer("timber", (0.5, 0.32, 0.22))
    for k in range(7):
        x = -0.72 + k * 0.24
        kind = k % 3
        if kind == 0:
            # Spyd: skaft og bladformet jernspiss.
            stick("spear", (x, 0.2, 0.0), (x, 0.0, 2.05), 0.04, 0.04, m.grey, bevel=0.0)
            ob = lathe("spear_head", [(0.0, 0.0), (0.035, 0.05), (0.04, 0.12), (0.0, 0.3)], m.iron, n=4)
            ob.scale = (1.0, 0.35, 1.0)
            move([ob], (x, 0.0, 2.02), roll=-0.098)
        elif kind == 1:
            # Sverd: bred klinge, parerstang, grep med lær og knapp.
            i = len(PIECES)
            box("blade", (-0.035, -0.008, 0.0), (0.035, 0.008, 0.85), m.iron, bevel=0.004)
            tip = lathe("blade_tip", [(0.0, -0.1), (0.05, 0.0)], m.iron, n=4, phase=math.pi / 4, cap=(False, False))
            tip.scale = (1.0, 0.3, 1.0)
            box("guard", (-0.13, -0.025, 0.85), (0.13, 0.025, 0.9), m.iron, bevel=0.01)
            box("grip", (-0.022, -0.022, 0.9), (0.022, 0.022, 1.07), leather, bevel=0.008)
            box("pommel", (-0.04, -0.035, 1.07), (0.04, 0.035, 1.13), m.iron, bevel=0.015)
            move(since(i), (x, -0.12, 0.42), roll=0.1)
        else:
            # Øks: skaft og et skjeggøksblad.
            stick("haft", (x, 0.2, 0.0), (x, 0.02, 1.45), 0.045, 0.045, m.grey, bevel=0.0)
            pts = [(0.0, -0.05), (0.12, -0.12), (0.2, -0.16), (0.2, 0.1), (0.1, 0.05), (0.0, 0.05)]
            ob = slab("axe_head", pts, 0.025, m.iron)
            move([ob], (x, 0.04, 1.33), roll=math.pi / 2 - 0.12)
    # Skjold lent mot bukkene i hver ende.
    for sx, tint in ((-1, (0.3, 0.42, 0.8)), (1, (0.85, 0.7, 0.25))):
        i = len(PIECES)
        lathe("rack_shield", [(0.0, -0.025), (0.32, -0.025), (0.32, 0.025), (0.0, 0.025)], layer("weatherboard", tint), n=14)
        ring("rack_rim", 0.32, 0.035, 0.06, m.iron, n=14)
        lathe("rack_boss", [(0.08, 0.025), (0.07, 0.06), (0.0, 0.08)], m.iron, n=8, cap=(False, False))
        move(since(i), (sx * 1.2, -0.12 + rng.uniform(-0.03, 0.03), 0.34), yaw=sx * 0.2, roll=math.pi / 2 - 0.25)


# --- Brønnen ------------------------------------------------------------------


def sag_roof(name, x0, x1, y_ridge, y_eave, z_ridge, z_eave, mat, thick=0.1, sag=0.06, n=6):
    """Ett takfall fra mønet (y_ridge) til takfoten (y_eave) med svai."""
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    g = {}
    run = math.hypot(y_eave - y_ridge, z_eave - z_ridge)
    for i in range(n + 1):
        u = i / n
        x = x0 + (x1 - x0) * u
        dip = sag * math.sin(math.pi * u)
        for j, t in enumerate((0.0, 0.6, 1.0)):
            y = y_ridge + (y_eave - y_ridge) * t
            z = z_ridge + (z_eave - z_ridge) * t - dip * (t * 1.2 if t > 0 else 0.3) + (0.04 if t == 1.0 else 0)
            g[(i, j, 0)] = bm.verts.new((x, y, z))
            g[(i, j, 1)] = bm.verts.new((x, y, z - thick))
    for i in range(n):
        for j in range(2):
            bm.faces.new([g[(i, j, 0)], g[(i + 1, j, 0)], g[(i + 1, j + 1, 0)], g[(i, j + 1, 0)]])
            bm.faces.new([g[(i, j, 1)], g[(i, j + 1, 1)], g[(i + 1, j + 1, 1)], g[(i + 1, j, 1)]])
        for j in (0, 2):
            bm.faces.new([g[(i, j, 0)], g[(i, j, 1)], g[(i + 1, j, 1)], g[(i + 1, j, 0)]])
    for i in (0, n):
        for j in range(2):
            bm.faces.new([g[(i, j, 0)], g[(i, j + 1, 0)], g[(i, j + 1, 1)], g[(i, j, 1)]])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        for lp in f.loops:
            c = lp.vert.co
            lp[uvl].uv = (c.x, -math.hypot(c.y - y_ridge, c.z - z_ridge))
    ob = kit._object(name, bm, mat)
    kit._bevel(ob, 0.025)
    return ob


def well():
    m = Mats()
    R, r, H = 0.98, 0.62, 0.8
    # Kummen: ring av murstein med en tykk krage av heller på toppen, og
    # mørkt vann nede i sjakta.
    lathe("curb", [(r, H - 0.05), (r, -0.25), (R - 0.04, -0.25), (R, -0.05), (R, H - 0.05)], m.stone, n=16, phase=math.pi / 16, cap=(False, False), sv=1.0)
    lathe("curb_cap", [(r - 0.04, H - 0.06), (R + 0.08, H - 0.06), (R + 0.1, H + 0.06), (R + 0.04, H + 0.12), (r - 0.04, H + 0.12)], m.flag, n=16, phase=math.pi / 16, closed=True)
    lathe("well_water", [(r + 0.01, 0.05), (0.0, 0.05)], plain("water_well", (0.03, 0.05, 0.05), 0.05), n=16, cap=(False, False))
    # Hellene rundt.
    lathe("well_flags", [(R + 0.46, -0.08), (R + 0.46, -0.04), (R + 0.42, 0.04), (R, 0.05)], m.flag, n=12, cap=(False, False), su=1.0)
    # Stolpene i kummen, med knekter opp til takbjelken.
    for sx in (-1, 1):
        stick("well_post", (sx * 0.75, 0.0, 0.2), (sx * 0.75, 0.0, 2.82), 0.2, 0.2, m.timber, ("timber", 1 + sx), bevel=0.04)
        box("post_shoe", (sx * 0.75 - 0.15, -0.15, H + 0.1), (sx * 0.75 + 0.15, 0.15, H + 0.22), m.stone, bevel=0.03)
        stick("well_knee", (sx * 0.75, 0.0, 1.92), (sx * 0.4, 0.0, 2.22), 0.1, 0.12, m.timber, ("timber", 2), bevel=0.02)
        # Lagrene for vinda: jernbeslag på innsida av stolpene i 1,55 m.
        box("bearing", (sx * 0.75 - 0.12, -0.13, 1.47), (sx * 0.75 + 0.12, 0.13, 1.63), m.iron, bevel=0.02)
    stick("well_plate", (-1.0, 0.0, 2.32), (1.0, 0.0, 2.32), 0.18, 0.2, m.timber, ("timber", 0), bevel=0.035)
    # Saltaket med mønet langs x, svai og mønebord.
    for sy in (-1, 1):
        sag_roof("well_roof", -1.18, 1.18, sy * 0.02, sy * 0.92, 2.86, 2.24, m.shingles, thick=0.1, sag=0.07)
        stick("well_rafter", (-0.95, sy * 0.06, 2.62), (-0.95, sy * 0.8, 2.12), 0.08, 0.1, m.timber, ("timber", 3), bevel=0.015)
        stick("well_rafter", (0.95, sy * 0.06, 2.62), (0.95, sy * 0.8, 2.12), 0.08, 0.1, m.timber, ("timber", 3), bevel=0.015)
    stick("well_ridge", (-1.24, 0.0, 2.88), (1.24, 0.0, 2.88), 0.16, 0.12, m.timber, ("timber", 1), bevel=0.03)
    # En bøtte til på kanten.
    i = len(PIECES)
    bucket_body(m)
    move(since(i), (-0.62, -0.62, H + 0.12 + 0.42), yaw=0.6)


def bucket_body(m, water=True):
    """Bøtta: staver i en kjegle, to jernbånd og vann. Origo der bøyla
    henger (0,13 m over kanten); kanten i z −0,13, bunnen i −0,42."""
    rim, bottom = -0.13, -0.42
    body = lathe("bucket", [(0.0, bottom), (0.13, bottom), (0.17, rim)], m.timber, n=12, cap=(False, False))
    swap_uv(body)
    scale_uv(body, 1.0, 3.0)
    lathe("bucket_in", [(0.15, rim), (0.115, bottom + 0.03), (0.0, bottom + 0.03)], m.ends, n=12, cap=(False, False))
    lathe("bucket_lip", [(0.17, rim), (0.15, rim)], m.ends, n=12, cap=(False, False))
    for z, rr in ((bottom + 0.06, 0.14), (rim - 0.06, 0.162)):
        band("bucket_band", rr + 0.006, 0.05, m.trim, n=12, at=(0, 0, z))
    if water:
        lathe("bucket_water", [(0.15, rim - 0.04), (0.0, rim - 0.04)], plain("water_well", (0.03, 0.05, 0.05), 0.05), n=12, cap=(False, False))
    for sx in (-1, 1):
        box("bucket_ear", (sx * 0.17 - 0.02, -0.03, rim - 0.06), (sx * 0.17 + 0.02, 0.03, rim + 0.02), m.iron, bevel=0.006)


def well_winch():
    m = Mats()
    # Valsen (radius 0,08 som WELL_DRUM) med tauet kveilet i bånd.
    drum = lathe("drum", [(0.08, -0.62), (0.08, 0.62)], m.logs, n=10)
    move([drum], pitch=math.pi / 2)
    for x in (-0.6, 0.6):
        ob = ring("drum_band", 0.083, 0.012, 0.05, m.iron, n=10)
        move([ob], (x, 0, 0), pitch=math.pi / 2)
    for k in range(10):
        x = -0.2 + k * 0.036
        ob = lathe("coil", [(0.08, -0.018), (0.098, -0.009), (0.098, 0.009), (0.08, 0.018)], m.trim, n=10, cap=(False, False), strip=("trim", ROPE))
        move([ob], (x, 0, 0), pitch=math.pi / 2)
    # Tappene gjennom lagrene og sveiven på østsida.
    stick("axle", (-0.92, 0, 0), (-0.62, 0, 0), 0.05, 0.05, m.iron, bevel=0.01)
    stick("axle", (0.62, 0, 0), (0.98, 0, 0), 0.05, 0.05, m.iron, bevel=0.01)
    stick("crank_arm", (0.98, 0, 0.02), (0.98, 0, -0.32), 0.06, 0.035, m.iron, bevel=0.01)
    stick("crank_grip", (0.96, 0, -0.32), (1.17, 0, -0.32), 0.055, 0.055, m.grey, bevel=0.015)


def well_rope():
    m = Mats()
    stick("rope", (0, 0, 0), (0, 0, -1.0), 0.028, 0.028, m.trim, ("trim", ROPE), bevel=0.008)


def well_bucket():
    m = Mats()
    bucket_body(m)
    # Bøyla av jern opp til tauet, og knuten.
    rim = -0.13
    pts = [Vector((math.cos(math.pi * k / 8) * 0.175, 0, rim + math.sin(math.pi * k / 8) * (-rim))) for k in range(9)]
    for a, b in zip(pts, pts[1:]):
        stick("bail", tuple(a), tuple(b), 0.018, 0.018, m.iron, bevel=0)
    lathe("knot", [(0.0, -0.035), (0.032, -0.02), (0.032, 0.02), (0.0, 0.035)], m.trim, n=6, strip=("trim", ROPE))


# --- Vedstabelen --------------------------------------------------------------


def woodpile():
    m = Mats()
    rng = random.Random(73)
    L, D, H = 1.8, 0.5, 1.2
    # Stakene i endene, og en kjerne bak kubbeendene.
    for sx in (-1, 1):
        for sy in (-1, 1):
            stick("pile_stake", (sx * (L / 2 + 0.06), sy * 0.2, -0.2), (sx * (L / 2 + 0.06) + rng.uniform(-0.02, 0.02), sy * 0.2, H + 0.18), 0.1, 0.1, m.timber, ("timber", rng.randrange(4)), bevel=0.02)
    core = box("pile_core", (-L / 2, -D / 2 + 0.06, 0.0), (L / 2, D / 2 - 0.04, H), m.logs, bevel=0.02)
    swap_uv(core)
    # Kubbeendene foran: sekskantede stubber i forskutte rader.
    z = 0.1
    row = 0
    while z < H - 0.05:
        x = -L / 2 + 0.1 + (0.09 if row % 2 else 0.0)
        while x < L / 2 - 0.08:
            r = rng.uniform(0.085, 0.105)
            ob = lathe("log_end", [(r * 0.95, 0.0), (r, 0.12), (0.0, 0.12)], m.logs, n=6, phase=rng.uniform(0, 1), cap=(False, True), top=m.ends)
            move([ob], (x, -D / 2 + 0.1 + rng.uniform(-0.03, 0.0), z + rng.uniform(-0.01, 0.01)), roll=math.pi / 2)
            x += 0.2
        z += 0.185
        row += 1
    # Taket: bord på skrå over stabelen.
    roof = box("pile_roof", (-L / 2 - 0.25, -D / 2 - 0.3, -0.04), (L / 2 + 0.25, D / 2 + 0.2, 0.04), m.boards, bevel=0.02)
    move([roof], (0, 0, H + 0.24), roll=-0.2)
    # Hoggestabben med øksa.
    lathe("chop_block", [(0.24, 0.0), (0.26, 0.05), (0.24, 0.48), (0.0, 0.48)], m.logs, n=8, at=(L / 2 + 0.52, -0.2, 0), top=m.ends)
    stick("axe_haft", (L / 2 + 0.5, -0.2, 0.46), (L / 2 + 0.66, -0.35, 1.08), 0.045, 0.045, m.grey, bevel=0.0)
    pts = [(0.0, -0.05), (0.1, -0.08), (0.17, -0.11), (0.17, 0.07), (0.08, 0.05), (0.0, 0.05)]
    ob = slab("axe_head", pts, 0.025, m.iron)
    move([ob], (L / 2 + 0.5, -0.2, 0.58), yaw=-0.5, roll=math.pi / 2 + 0.3)


# --- Veiskiltet ---------------------------------------------------------------


def signpost():
    m = Mats()
    stick("sign_post", (0, 0, -0.35), (0, 0, 2.5), 0.15, 0.15, m.timber, ("timber", 1), bevel=0.035)
    pyramid("sign_cap", 0.26, 0.16, m.shingles, (0, 0, 2.5))
    box("sign_collar", (-0.1, -0.1, 2.44), (0.1, 0.1, 2.52), m.iron, bevel=0.01)
    # Pila: en tykk tavle med spiss mot +X og en svalehale bak.
    z, h, L, d = 2.05, 0.34, 1.25, 0.07
    pts = [(0.02, -h / 2), (L - 0.24, -h / 2), (L - 0.24, -h / 2 - 0.05), (L, 0.0), (L - 0.24, h / 2 + 0.05), (L - 0.24, h / 2), (0.02, h / 2), (0.12, 0.0)]
    ob = slab("sign_board", pts, d, m.grey)
    move([ob], (0, -d / 2, z), roll=math.pi / 2)
    for dz in (-0.1, 0.1):
        box("sign_strap", (-0.1, -d / 2 - 0.02, z + dz - 0.025), (0.24, d / 2 + 0.02, z + dz + 0.025), m.iron, bevel=0.008)
    burnt = layer("timber", (0.07, 0.05, 0.04))
    for back in (False, True):
        y = (d / 2 + 0.002) * (1 if back else -1)
        letters("KRYPTEN", (0.6, y, z), 0.15, burnt, back=back)
        sx = 1.0
        box("sign_cross_v", (sx - 0.014, y - 0.006, z - 0.065), (sx + 0.014, y + 0.006, z + 0.065), m.iron, bevel=0.003)
        box("sign_cross_h", (sx - 0.045, y - 0.006, z + 0.01), (sx + 0.045, y + 0.006, z + 0.04), m.iron, bevel=0.003)
    # Steiner rundt foten.
    rng = random.Random(4)
    for k in range(4):
        a = k * 1.6 + 0.3
        ob = lathe("sign_stone", [(0.1, -0.05), (0.12, 0.03), (0.07, 0.09), (0.0, 0.1)], m.stone, n=5, phase=rng.uniform(0, 1))
        ob.scale = (1.0 + rng.uniform(0, 0.4), 0.8, 0.8)
        move([ob], (math.cos(a) * 0.2, math.sin(a) * 0.2, 0.0), yaw=a)


# --- Kista --------------------------------------------------------------------

CHEST_W, CHEST_D, CHEST_H = 1.0, 0.56, 0.46


def chest_body():
    m = Mats()
    W, D, H = CHEST_W, CHEST_D, CHEST_H
    # Kassa av planker: timber-laget, tre bord i høyden.
    body = box("chest_box", (-W / 2, -D / 2, 0.04), (W / 2, D / 2, H), m.timber, bevel=0.02)
    scale_uv(body, 1.0, 0.75 / (H - 0.04))
    box("chest_plinth", (-W / 2 - 0.02, -D / 2 - 0.02, 0.0), (W / 2 + 0.02, D / 2 + 0.02, 0.07), m.grey, bevel=0.015)
    # Jernvinkler på hjørnene, bånd rundt kassa og store nagler.
    for sx in (-1, 1):
        for sy in (-1, 1):
            x, y = sx * W / 2, sy * D / 2
            box("chest_corner_a", (min(x, x - sx * 0.12) - 0.01, y - 0.014, 0.02), (max(x, x - sx * 0.12) + 0.01, y + 0.014, H + 0.005), m.iron, bevel=0.006)
            box("chest_corner_b", (x - 0.014, min(y, y - sy * 0.12) - 0.01, 0.02), (x + 0.014, max(y, y - sy * 0.12) + 0.01, H + 0.005), m.iron, bevel=0.006)
    for bx in (-0.3, 0.3):
        for sy in (-1, 1):
            box("chest_band", (bx - 0.035, sy * D / 2 - 0.012, 0.04), (bx + 0.035, sy * D / 2 + 0.012, H), m.iron, bevel=0.005)
            for bz in (0.13, 0.26, 0.39):
                box("chest_rivet", (bx - 0.016, sy * (D / 2 + 0.012) - 0.01, bz - 0.016), (bx + 0.016, sy * (D / 2 + 0.012) + 0.01, bz + 0.016), m.iron, bevel=0.0)
    # Håndtak på endene og låsplata foran.
    for sx in (-1, 1):
        box("handle_plate", (sx * W / 2 - 0.012, -0.08, H * 0.55), (sx * W / 2 + 0.012, 0.08, H * 0.72), m.iron, bevel=0.005)
        pts = [Vector((sx * (W / 2 + 0.01), math.cos(math.pi * k / 6) * 0.08, H * 0.6 - math.sin(math.pi * k / 6) * 0.07)) for k in range(7)]
        for a, b in zip(pts, pts[1:]):
            stick("handle", tuple(a), tuple(b), 0.022, 0.022, m.iron, bevel=0)
    box("lock_plate", (-0.08, -D / 2 - 0.02, H - 0.2), (0.08, -D / 2 + 0.005, H - 0.02), m.iron, bevel=0.012)
    box("keyhole", (-0.012, -D / 2 - 0.026, H - 0.15), (0.012, -D / 2 - 0.018, H - 0.09), layer("iron", (0.08, 0.08, 0.08)), bevel=0.003)
    for sx in (-1, 1):
        for sy in (-1, 1):
            box("chest_foot", (sx * (W / 2 - 0.02) - 0.06, sy * (D / 2 - 0.02) - 0.06, 0.0), (sx * (W / 2 - 0.02) + 0.06, sy * (D / 2 - 0.02) + 0.06, 0.04), m.iron, bevel=0.008)


def chest_lid():
    """Lokket: buet av planker over en halv ellipse fra hengslet (origo,
    bakre øvre kant, +Y) til forkanten (y = −D), med jernbånd over buen,
    nagler og en hasp som henger ned foran."""
    m = Mats()
    W, D = CHEST_W, CHEST_D
    R = D / 2
    H = R * 0.62
    n = 8

    def arc(t, grow=0.0):
        a = math.pi * t
        return (-R + math.cos(a) * (R + grow), math.sin(a) * (H + grow))

    def shell(name, xa, xb, grow0, grow1, mat, sv=1.0, strip=None):
        bm = bmesh.new()
        uvl = bm.loops.layers.uv.verify()
        rows = []
        for k in range(n + 1):
            y0, z0 = arc(k / n, grow0)
            y1, z1 = arc(k / n, grow1)
            rows.append([bm.verts.new((xa, y0, z0)), bm.verts.new((xb, y0, z0)), bm.verts.new((xb, y1, z1)), bm.verts.new((xa, y1, z1))])
        cum = [0.0]
        for k in range(n):
            a, b = rows[k][3].co, rows[k + 1][3].co
            cum.append(cum[-1] + (b - a).length)
        for k in range(n):
            a, b = rows[k], rows[k + 1]
            for q, uvs in (((a[3], a[2], b[2], b[3]), "out"), ((a[0], b[0], b[1], a[1]), "in"), ((a[0], a[3], b[3], b[0]), "side"), ((a[1], b[1], b[2], a[2]), "side")):
                f = bm.faces.new(q)
                for lp in f.loops:
                    c = lp.vert.co
                    kk = k if lp.vert in a else k + 1
                    lp[uvl].uv = (c.x, cum[kk] * sv) if uvs != "side" else (c.y, c.z)
        for row in (rows[0], rows[-1]):
            f = bm.faces.new(row)
            for lp in f.loops:
                lp[uvl].uv = (lp.vert.co.x, lp.vert.co.z)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        return kit._object(name, bm, mat)

    shell("lid", -W / 2, W / 2, -0.035, 0.0, m.timber, sv=2.5)
    # Endestykkene: halve ellipser i yz-planet.
    for sx in (-1, 1):
        pts = [arc(t / 12, -0.03) for t in range(13)]
        bm = bmesh.new()
        uvl = bm.loops.layers.uv.verify()
        xa, xb = sx * (W / 2 - 0.035), sx * (W / 2 - 0.005)
        fa = [bm.verts.new((xa, y, z)) for y, z in pts]
        fb = [bm.verts.new((xb, y, z)) for y, z in pts]
        bm.faces.new(fa)
        bm.faces.new(fb[::-1])
        for j in range(len(pts)):
            k = (j + 1) % len(pts)
            bm.faces.new((fa[j], fa[k], fb[k], fb[j]))
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        for f in bm.faces:
            for lp in f.loops:
                c = lp.vert.co
                lp[uvl].uv = (c.y, c.z * 2.5)
        kit._object("lid_end", bm, m.timber)
    for bx in (-0.3, 0.3):
        shell("lid_band", bx - 0.035, bx + 0.035, 0.0, 0.012, m.iron)
        for k in range(1, 8, 2):
            y, z = arc(k / 8, 0.018)
            box("lid_rivet", (bx - 0.016, y - 0.016, z - 0.016), (bx + 0.016, y + 0.016, z + 0.016), m.iron, bevel=0.0)
    # Jernkant langs forkanten og haspen ned over låsplata.
    box("lid_lip", (-W / 2 - 0.01, -D - 0.015, -0.01), (W / 2 + 0.01, -D + 0.02, 0.03), m.iron, bevel=0.006)
    box("hasp", (-0.04, -D - 0.03, -0.12), (0.04, -D - 0.012, 0.02), m.iron, bevel=0.008)
    for bx in (-0.3, 0.3):
        stick("hinge", (bx - 0.06, 0.012, 0.0), (bx + 0.06, 0.012, 0.0), 0.03, 0.03, m.iron, bevel=0)


MODELS = {
    "market_cross": market_cross,
    "market_stall": market_stall,
    "market_stall_b": market_stall_b,
    "cart": cart,
    "barrel": barrel,
    "crate": crate,
    "lantern_post": lantern_post,
    "training_dummy": training_dummy,
    "weapon_rack": weapon_rack,
    "well": well,
    "well_winch": well_winch,
    "well_rope": well_rope,
    "well_bucket": well_bucket,
    "woodpile": woodpile,
    "signpost": signpost,
    "chest": chest_body,
    "chest_lid": chest_lid,
}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        town.LIT = False
        MODELS[name]()
        town.finish(f"{OUT}/{name}.glb", max_edge=100.0)
