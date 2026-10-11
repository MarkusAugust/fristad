"""Bymuren i den malte stilen (docs/ny-stil.md), bygd med byggesettet
(kit.py). Erstatter walls.py. Kjør:

    python3 tools/models/fortress.py [town_wall wall_tower gatehouse]

Mursegmentet: kraftig mur av bymurstein (`wall_stone`) med skrå fot som
buler litt ut, en gesims på kragsteiner, brystvern med murtinder og
skyteskår (halve murtinder i endene, så to segmenter møtes i en hel),
bjelkehull og en lav kant på innsida av murgangen.

Hjørnetårnet: rundt (16 kanter, litt skjevt) med skrå fot, gesims,
skytespalter i tre høyder, en ring av kragsteiner under en utkraget
skyteganger av bord med bjelkestolper, og et høyt kjegletak av spon med
takskjegg som bretter seg ut, og et smijernsspir.

Porthuset: to firkantede flanketårn med skrå fot, hjørnestein og
valmtak (pyramide) med takskjegg, en midtblokk med rundbuet portrom man
kan gå gjennom (tønnehvelv, buestein med sluttstein), hevet portgitter,
to eikefløyer slått inn mot veggene, bjelkeender gjennom muren, brystvern
med murtinder og et lite pulttak av spon over vaktgangen.

Samme navn, mål og origo som walls.py. Akser som i town.py: −Y fram, Z opp,
origo midt på grunnflaten. Mursegmentet har utsida mot +Y (byen snur det så
utsida vender ut), og porthuset har utsida mot −Y. Kolliderne setter byen
(`town::wall`)."""

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
from sacred import transform, voussoirs  # noqa: E402
from town import layer  # noqa: E402

OUT = "assets/models/town"
PIECES = town.PIECES


class Mats:
    def __init__(self):
        self.stone = layer("wall_stone")
        self.dark_stone = layer("wall_stone", (0.86, 0.84, 0.8))
        self.cap = layer("flagstone", (0.95, 0.93, 0.9))
        self.ruin = layer("ruin", (0.95, 0.95, 0.92))
        self.roof = layer("shingles")
        self.tiles = layer("tiles", (0.92, 0.9, 0.9))
        self.timber = layer("timber")
        self.trim = layer("trim")
        self.boards = layer("weatherboard", (0.9, 0.86, 0.8))
        self.door = layer("door")
        self.door_dark = layer("door_b")
        self.iron = layer("iron", (0.6, 0.58, 0.56), 0.45)
        self.soot = layer("stone", (0.12, 0.11, 0.1))


# --- Former -------------------------------------------------------------------


def mesh(name, rings, mat, closed=True, cap_lo=True, cap_hi=True, recalc=True):
    """Flater mellom ringer av punkter (alle like lange). `closed` lukker hver
    ring rundt; endene lukkes med en mangekant."""
    bm = bmesh.new()
    vs = [[bm.verts.new(p) for p in r] if len(r) > 1 else [bm.verts.new(r[0])] for r in rings]
    for A, B in zip(vs, vs[1:]):
        n = max(len(A), len(B))
        m = n if closed else n - 1
        for i in range(m):
            j = (i + 1) % n
            if len(A) == 1:
                bm.faces.new((A[0], B[i], B[j]))
            elif len(B) == 1:
                bm.faces.new((A[i], A[j], B[0]))
            else:
                bm.faces.new((A[i], A[j], B[j], B[i]))
    if cap_lo and len(vs[0]) > 2:
        bm.faces.new(vs[0][::-1])
    if cap_hi and len(vs[-1]) > 2:
        bm.faces.new(vs[-1])
    # Ringene går mot klokka sett ovenfra og profilet nedenfra, så
    # normalene peker ut av seg selv; lukkede former rettes i tillegg.
    if recalc:
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return kit._object(name, bm, mat)


SEAM = math.pi / 4


def round_uv(ob, roof_apex=None, rref=None):
    """Uv rundt en loddrett akse i origo: u = vinkel × radius i hjørnet
    (meter langs omkretsen), v = høyden, eller nedover takflaten fra
    spissen. Flater som ligger flatt får (x, y). Sømmen ligger i 45°, der
    flatene på hver side vender hver sin vei, så motorens grove detaljnivå
    (som slår sammen hjørner som vender samme vei) aldri blander dem."""
    me = ob.data
    bm = bmesh.new()
    bm.from_mesh(me)
    uvl = bm.loops.layers.uv.verify()
    tau = 2 * math.pi
    for f in bm.faces:
        # Flater som motoren regner som opp eller ned (største komponent i
        # z) får plan uv, så sømmen ikke går gjennom dem.
        n = [round(v * 127) for v in f.normal]
        if abs(n[2]) >= max(abs(n[0]), abs(n[1])):
            for lp in f.loops:
                lp[uvl].uv = (lp.vert.co.x, lp.vert.co.y)
            continue
        c = f.calc_center_median()
        ac = (math.atan2(c.y, c.x) - SEAM) % tau
        for lp in f.loops:
            p = lp.vert.co
            r = math.hypot(p.x, p.y)
            a = (math.atan2(p.y, p.x) - SEAM) % tau if r > 1e-4 else ac
            if a - ac > math.pi:
                a -= tau
            elif ac - a > math.pi:
                a += tau
            # Én fast radius for hele formen: ellers skjærer den skrå foten
            # og taket teksturen på skrå (u ville vokse med radien).
            rr = rref if rref else r
            if roof_apex is None:
                lp[uvl].uv = (a * rr, p.z)
            else:
                lp[uvl].uv = (a * rr, -math.hypot(r, roof_apex - p.z))
    bm.to_mesh(me)
    bm.free()


def engine_uv(skip=()):
    """Uv i meter etter flatens hovedakse, valgt som motoren velger retningen
    til hjørnene i det grove detaljnivået (`Mesh::clustered`: normalen i
    motorens akser, den siste av like store komponenter). Da får hjørner som
    slås sammen samme uv, og store flater strekkes ikke på avstand. Gjelder
    delene med flisbare lag lagt rett på (ikke dreide former, bjelker og tak)."""

    bpy.context.view_layer.update()
    for ob in PIECES:
        if ob.type != "MESH" or any(ob.name.startswith(s) for s in skip):
            continue
        names = {m.name.split(".")[0] for m in ob.data.materials if m}
        if not names & BOXED:
            continue
        mw = ob.matrix_world
        me = ob.data
        bm = bmesh.new()
        bm.from_mesh(me)
        uvl = bm.loops.layers.uv.verify()
        nm = mw.to_3x3().inverted().transposed()
        for f in bm.faces:
            n = (nm @ f.normal).normalized()
            q = [round(v * 127) for v in (n.x, n.z, -n.y)]
            best = 0
            for k in (1, 2):
                if abs(q[k]) >= abs(q[best]):
                    best = k
            for lp in f.loops:
                w = mw @ lp.vert.co
                lp[uvl].uv = [(w.y, w.z), (w.x, w.y), (w.x, w.z)][best]
        bm.to_mesh(me)
        bm.free()


BOXED = {"layer_wall_stone", "layer_flagstone", "layer_stone", "layer_ruin", "layer_weatherboard"}


def lathe(name, prof, n, mat, rng=None, jitter=0.0, a0=0.0, roof_apex=None, cap=True, rref=None):
    """Dreid form rundt z: profilet (r, z) nedenfra. Med `jitter` flyttes hver
    ring litt (håndlaget og skjevt)."""
    rings = []
    for k, (r, z) in enumerate(prof):
        if r < 1e-6:
            rings.append([(0.0, 0.0, z)])
            continue
        ring = []
        for i in range(n):
            a = a0 + 2 * math.pi * i / n
            rr = r + (rng.uniform(-jitter, jitter) if rng and jitter else 0.0)
            ring.append((rr * math.cos(a), rr * math.sin(a), z))
        rings.append(ring)
    ob = mesh(name, rings, mat, cap_lo=cap, cap_hi=cap, recalc=cap)
    round_uv(ob, roof_apex, rref)
    return ob


def slit(p, c, out, h=0.7, w=0.1):
    """Skytespalte: en mørk, smal flate rett utenpå muren, med en overligger
    og et brett av stein. `out` er murens normal (x, y)."""
    o = Vector((out[0], out[1], 0)).normalized()
    c = Vector(c)
    yaw = math.atan2(o.y, o.x)
    obs = [
        box("slit", (-0.06, -w / 2, -h / 2), (0.02, w / 2, h / 2), p.soot, bevel=0),
        box("slit_head", (-0.03, -w / 2 - 0.16, h / 2), (0.1, w / 2 + 0.16, h / 2 + 0.16), p.cap, bevel=0.03),
        box("slit_sill", (-0.03, -w / 2 - 0.12, -h / 2 - 0.12), (0.12, w / 2 + 0.12, -h / 2), p.cap, bevel=0.03),
    ]
    transform(obs, Matrix.Translation(c) @ Matrix.Rotation(yaw, 4, "Z"))


def hip_roof(p, name, hw, hd, z0, rise, over, mat, kick=0.35, thick=0.3, at=(0, 0)):
    """Valmtak (pyramide) over et rektangel ±hw × ±hd med takfoten i z0:
    takskjegget stikker `over` ut og bretter seg opp (`kick`), flatene
    henger litt (bratt øverst). Under: en sofitt inn til muren."""
    cx, cy = at
    ex, ey = hw + over, hd + over

    def rect(sx, sy, z):
        return [(cx - sx, cy - sy, z), (cx + sx, cy - sy, z), (cx + sx, cy + sy, z), (cx - sx, cy + sy, z)]

    top = z0 + rise
    rings = [
        rect(hw - 0.05, hd - 0.05, z0 - thick),
        rect(ex, ey, z0 - thick + kick * 0.5),
        rect(ex, ey, z0 + kick * 0.5),
        rect(hw, hd, z0 + rise * 0.16),
        rect(hw * 0.45, hd * 0.45, z0 + rise * 0.62),
        [(cx, cy, top)],
    ]
    ob = mesh(name, rings, mat)
    me = ob.data
    bm = bmesh.new()
    bm.from_mesh(me)
    uvl = bm.loops.layers.uv.verify()
    for f in bm.faces:
        n = f.normal
        if abs(n.z) > 0.97:
            for lp in f.loops:
                lp[uvl].uv = (lp.vert.co.x, lp.vert.co.y)
            continue
        # u langs takfoten, v nedover fallet (sponet renner ned).
        ax = 0 if abs(n.x) > abs(n.y) else 1
        for lp in f.loops:
            q = lp.vert.co
            along = q.y - cy if ax == 0 else q.x - cx
            down = abs(q.x - cx) if ax == 0 else abs(q.y - cy)
            lp[uvl].uv = (along, -(down * 1.0 + (top - q.z) * 0.6))
    bm.to_mesh(me)
    bm.free()
    return top


# --- Mursegmentet -------------------------------------------------------------

WL = 6.24  # lengden langs x
WALK = 5.45  # murgangen
P_IN, P_OUT = 0.36, 0.98  # brystvernet, på tvers
MERLON_Z = 6.55


def bulge(x):
    """Hvor mye murflata buler ut midt på segmentet (0 i endene)."""
    return 0.07 * math.cos(math.pi * x / WL)


def wall_face_y(x, z):
    """Utsida av murkjernen i (x, z): skrå fot, så nesten loddrett."""
    if z < 0.2:
        return 1.12
    if z < 1.6:
        return 1.12 - (z - 0.2) / 1.4 * 0.3
    if z < 3.4:
        return 0.82 + bulge(x) * (z - 1.6) / 1.8
    return 0.82 + bulge(x) * (WALK - z) / (WALK - 3.4) - 0.0


def town_wall():
    p = Mats()
    rng = random.Random(21)
    xs = [-WL / 2, -WL / 4, 0.0, WL / 4, WL / 2]
    # Murkjernen: tverrsnittet trukket ut langs x i fem skiver; utsida buler.
    rings = []
    for x in xs:
        prof = [(-0.98, -0.8), (wall_face_y(x, -0.8), -0.8), (wall_face_y(x, 0.2), 0.2), (wall_face_y(x, 1.6), 1.6), (wall_face_y(x, 3.4), 3.4), (0.82, WALK), (-0.8, WALK), (-0.8, 1.2), (-0.92, 0.25)]
        rings.append([(x, y, z) for y, z in prof])
    body = mesh("wall_body", rings, p.stone, closed=True)
    town.uv_box(body)
    # Gesimsen over kragsteinene, og selve kragsteinene.
    box("course", (-WL / 2, 0.7, WALK - 0.24), (WL / 2, 1.06, WALK + 0.02), p.cap, bevel=0.04)
    n = 6
    for k in range(n):
        x = -WL / 2 + WL * (k + 0.5) / n
        box("corbel", (x - 0.17, 0.75, WALK - 0.62), (x + 0.17, 1.04, WALK - 0.24), p.stone, bevel=0.04)
        box("corbel", (x - 0.15, 0.75, WALK - 0.86), (x + 0.15, 0.94, WALK - 0.62), p.stone, bevel=0.04)
    # Brystvernet: den lave muren under skyteskårene, og murtindene.
    box("parapet", (-WL / 2, P_IN, WALK), (WL / 2, P_OUT, MERLON_Z), p.stone, bevel=0.03)
    box("parapet_top", (-WL / 2, P_IN - 0.05, MERLON_Z - 0.02), (WL / 2, P_OUT + 0.06, MERLON_Z + 0.1), p.cap, bevel=0.03)
    step = WL / 4
    mw = 0.55  # halv bredde på en murtind
    for k in range(5):
        xc = -WL / 2 + step * k
        x0, x1 = max(xc - mw, -WL / 2), min(xc + mw, WL / 2)
        end = k in (0, 4)
        h = 0.95 if end else 0.95 + rng.uniform(-0.08, 0.06)
        lean = 0.0 if end else rng.uniform(-0.035, 0.035)
        m = box("merlon", (x0, P_IN + 0.02, MERLON_Z), (x1, P_OUT - 0.02, MERLON_Z + h), p.stone, bevel=0.03)
        c = box("merlon_cap", (x0 - (0 if k == 0 else 0.05), P_IN - 0.04, MERLON_Z + h), (x1 + (0 if k == 4 else 0.05), P_OUT + 0.05, MERLON_Z + h + 0.16), p.cap, bevel=0.04)
        if lean:
            pivot = Vector((xc, 0.6, MERLON_Z))
            transform([m, c], Matrix.Translation(pivot) @ Matrix.Rotation(lean, 4, "Y") @ Matrix.Translation(-pivot))
        if not end:
            # Skytespalte i murtinda, ut og inn.
            box("slit", (xc - 0.05, P_OUT - 0.01, MERLON_Z + 0.18), (xc + 0.05, P_OUT + 0.0, MERLON_Z + 0.78), p.soot, bevel=0)
            box("slit", (xc - 0.05, P_IN + 0.0, MERLON_Z + 0.18), (xc + 0.05, P_IN + 0.01, MERLON_Z + 0.78), p.soot, bevel=0)
    # Den lave kanten på innsida av murgangen.
    box("kerb", (-WL / 2, -0.8, WALK), (WL / 2, -0.52, WALK + 0.38), p.cap, bevel=0.03)
    # Bjelkehull og en vannspyer.
    for k, (x, z) in enumerate([(-1.9, 2.5), (0.3, 2.6), (2.0, 2.45), (-0.9, 4.3), (1.4, 4.4)]):
        y = wall_face_y(x, z)
        box("putlog", (x - 0.11, y - 0.03, z - 0.11), (x + 0.11, y + 0.02, z + 0.11), p.soot, bevel=0)
    for x in (-WL / 4,):
        stick("spout", (x, P_OUT - 0.2, WALK + 0.1), (x, P_OUT + 0.5, WALK + 0.02), 0.2, 0.16, p.cap, bevel=0.03)
    # Store, ujevne steiner i den skrå foten ute.
    for k in range(3):
        x = -WL / 2 + WL * (k + 0.25 + rng.uniform(0, 0.5)) / 3
        w, h = rng.uniform(0.7, 1.0), rng.uniform(0.4, 0.55)
        b = box("foot_stone", (-w / 2, -0.25, -h / 2), (w / 2, 0.25, h / 2), p.dark_stone, bevel=0.1)
        transform([b], Matrix.Translation((x, 1.12, rng.uniform(-0.1, 0.1))) @ Matrix.Rotation(rng.uniform(-0.15, 0.15), 4, "Y") @ Matrix.Rotation(rng.uniform(-0.3, 0.3), 4, "Z") @ Matrix.Rotation(-0.3, 4, "X"))
    engine_uv()
    return p


# --- Hjørnetårnet -------------------------------------------------------------

TR = 3.4  # radien på tårnet
T_N = 16


def tower_r(z, prof):
    """Radien på tårnkroppen i høyden z (lineært mellom profilpunktene)."""
    for (r0, z0), (r1, z1) in zip(prof, prof[1:]):
        if z0 <= z <= z1:
            return r0 + (r1 - r0) * (z - z0) / (z1 - z0)
    return prof[-1][0]


def wall_tower():


    p = Mats()
    rng = random.Random(22)
    R = TR
    top = 9.4
    a0 = 0.0  # hjørnene i 2πk/n, flatene midt imellom (sømmen i 45°)
    # Kroppen: skrå fot, så loddrett, litt skjev.
    prof = [(R + 0.75, -0.8), (R + 0.7, 0.3), (R + 0.12, 2.2), (R, 4.8), (R - 0.04, 7.4), (R - 0.06, top)]
    lathe("tower_body", prof, T_N, p.stone, rng=rng, jitter=0.03, a0=a0, rref=R)
    # Gesimsen over foten.
    lathe("tower_course", [(R + 0.18, 2.1), (R + 0.22, 2.34), (R + 0.02, 2.5)], T_N, p.cap, a0=a0, cap=False)
    # Skytespalter i tre høyder, forskjøvet rundt, midt på flatene.
    cf = math.cos(math.pi / T_N)
    for z, k0 in ((3.6, 0), (5.9, 2), (8.0, 1)):
        for k in range(k0, T_N, 4):
            a = 2 * math.pi * (k + 0.5) / T_N
            r = tower_r(z, prof) * cf + 0.03
            slit(p, (r * math.cos(a), r * math.sin(a), z), (math.cos(a), math.sin(a)), h=0.85)
    # Kragsteinene under skytegangen, i hjørnene.
    for k in range(T_N):
        a = 2 * math.pi * k / T_N
        rr = R - 0.1
        obs = [box("corbel", (0.0, -0.17, -0.3), (0.4, 0.17, 0.0), p.stone, bevel=0.04), box("corbel", (0.0, -0.15, -0.6), (0.24, 0.15, -0.3), p.stone, bevel=0.04)]
        transform(obs, Matrix.Translation((rr * math.cos(a), rr * math.sin(a), top + 0.0)) @ Matrix.Rotation(a, 4, "Z"))
    # Skytegangen: et gulv over kragsteinene, et belte av bord og stolper.
    H0, H1 = top, top + 1.45
    lathe("hoard_floor", [(R + 0.36, H0 - 0.08), (R + 0.36, H0 + 0.12)], T_N, p.cap, a0=a0)
    hb = lathe("hoard", [(R + 0.3, H0 + 0.12), (R + 0.3, H1)], T_N, p.boards, a0=a0, cap=False)
    sol = hb.modifiers.new("thick", "SOLIDIFY")
    sol.thickness = 0.08
    sol.offset = 0.0
    bpy.context.view_layer.objects.active = hb
    bpy.ops.object.modifier_apply(modifier=sol.name)
    for k in range(T_N):
        a = 2 * math.pi * k / T_N
        rr = R + 0.38
        c = (rr * math.cos(a), rr * math.sin(a))
        stick("hoard_post", (c[0], c[1], H0 + 0.1), (c[0], c[1], H1 + 0.02), 0.17, 0.17, p.timber, ("timber", k % 4), seed=k)
        # Mørke luker (skyteåpninger) på annenhver flate.
        if k % 2 == 0:
            a2 = 2 * math.pi * (k + 0.5) / T_N
            r2 = (R + 0.3) * cf + 0.05
            ob = box("hatch", (-0.02, -0.3, -0.24), (0.02, 0.3, 0.24), p.soot, bevel=0)
            transform([ob], Matrix.Translation((r2 * math.cos(a2), r2 * math.sin(a2), H0 + 0.85)) @ Matrix.Rotation(a2, 4, "Z"))
            sill = box("hatch_sill", (-0.02, -0.38, -0.05), (0.12, 0.38, 0.03), p.timber, bevel=0.01)
            transform([sill], Matrix.Translation((r2 * math.cos(a2), r2 * math.sin(a2), H0 + 0.58)) @ Matrix.Rotation(a2, 4, "Z"))
    lathe("hoard_plate", [(R + 0.22, H1), (R + 0.5, H1), (R + 0.5, H1 + 0.16), (R - 0.4, H1 + 0.16)], T_N, p.timber, a0=a0, cap=False)
    # Kjegletaket: takskjegget bretter seg ut, flaten henger litt.
    z0 = H1 + 0.1
    apex = 18.4
    E = R + 1.3
    roof_prof = [
        (R + 0.25, z0 - 0.02),
        (E, z0 - 0.36),
        (E, z0 - 0.14),
        (R + 0.62, z0 + 0.8),
        (R * 0.62, z0 + (apex - z0) * 0.42),
        (R * 0.28, z0 + (apex - z0) * 0.75),
        (0.0, apex),
    ]
    lathe("tower_roof", roof_prof, T_N, p.roof, a0=a0, roof_apex=apex, cap=False, rref=R * 0.85)
    # Spiret: en kule og en spiss av smijern, med en liten vimpel.
    lathe("finial", [(0.14, apex - 0.3), (0.06, apex + 0.1), (0.13, apex + 0.3), (0.13, apex + 0.42), (0.04, apex + 0.55), (0.03, apex + 1.4), (0.0, apex + 1.6)], 6, p.iron)
    box("vane", (0.04, -0.012, apex + 0.85), (0.62, 0.012, apex + 1.2), p.iron, bevel=0)
    # Noen løse steiner i foten.
    for k in range(5):
        a = rng.uniform(0, 2 * math.pi)
        w = rng.uniform(0.5, 0.75)
        b = box("foot_stone", (-0.22, -w / 2, -0.2), (0.22, w / 2, 0.2), p.dark_stone, bevel=0.06)
        transform([b], Matrix.Translation(((R + 0.74) * math.cos(a), (R + 0.74) * math.sin(a), 0.05)) @ Matrix.Rotation(a, 4, "Z") @ Matrix.Rotation(0.3, 4, "Y"))
    engine_uv(skip=("tower_body", "tower_course", "hoard"))
    return p


# --- Porthuset ----------------------------------------------------------------

G_PW = 5.2  # bredden på portrommet
G_SPRING = 3.0  # der buen begynner
G_D = 4.6  # dybden på midtblokken
T_HW, T_HD = 2.0, 2.7  # flanketårnene
T_X = G_PW / 2 + T_HW
T_TOP = 10.6
MID_TOP = 8.6


def flank_tower(p, sx, rng):
    """Et firkantet flanketårn med skrå fot (ikke mot portrommet),
    hjørnestein, skytespalter, gesims på kragsteiner, et belte av bord og
    valmtak."""
    x = sx * T_X
    xi = sx * G_PW / 2  # mot portrommet
    rows = [(-0.8, 0.55), (0.3, 0.5), (2.0, 0.0), (T_TOP, 0.0)]
    rings = []
    for z, e in rows:
        xo = sx * (G_PW / 2 + 2 * T_HW + e)
        hd = T_HD + e
        xa, xb = min(xi, xo), max(xi, xo)
        rings.append([(xa, -hd, z), (xb, -hd, z), (xb, hd, z), (xa, hd, z)])
    t = mesh("gate_tower", rings, p.stone)
    kit._bevel(t, 0.05)
    town.uv_box(t)
    lip = 0.03
    xa, xb = (xi, x + T_HW + 0.14) if sx > 0 else (x - T_HW - 0.14, xi)
    box("tower_course", (xa, -T_HD - 0.14, 1.95), (xb, T_HD + 0.14, 2.2), p.cap, bevel=0.04)
    # Hjørnestein på de ytre hjørnene, annenhver lang og kort.
    for cx in (x + sx * T_HW,):
        for cy in (-T_HD, T_HD):
            z = 2.2
            k = 0
            while z < T_TOP - 1.0:
                h = rng.uniform(0.42, 0.55)
                lx, ly = (0.62, 0.36) if k % 2 == 0 else (0.36, 0.62)
                ox = -1 if cx > x else 1
                oy = -1 if cy > 0 else 1
                lo = (min(cx - ox * lip, cx + ox * lx), min(cy - oy * lip, cy + oy * ly), z)
                hi = (max(cx - ox * lip, cx + ox * lx), max(cy - oy * lip, cy + oy * ly), z + h - 0.04)
                box("quoin", lo, hi, p.cap, bevel=0.04)
                z += h
                k += 1
    # Skytespalter: tre høyder ut, én inn, og én på yttersida.
    for z in (3.6, 6.2, 8.6):
        slit(p, (x, -T_HD, z), (0, -1), h=0.9)
    slit(p, (x, T_HD, 6.2), (0, 1), h=0.9)
    slit(p, (x + sx * T_HW, 0.0, 7.2), (sx, 0), h=0.9)
    # Kragsteiner under gesimsen rundt toppen.
    for sy in (-1, 1):
        for k in range(5):
            xx = x - T_HW + 0.4 + (2 * T_HW - 0.8) * k / 4
            y0, y1 = (sy * T_HD, sy * (T_HD + 0.3))
            box("corbel", (xx - 0.16, min(y0, y1), T_TOP - 0.62), (xx + 0.16, max(y0, y1), T_TOP - 0.28), p.stone, bevel=0.04)
    for k in range(6):
        yy = -T_HD + 0.4 + (2 * T_HD - 0.8) * k / 5
        x0, x1 = x + sx * T_HW, x + sx * (T_HW + 0.3)
        box("corbel", (min(x0, x1), yy - 0.16, T_TOP - 0.62), (max(x0, x1), yy + 0.16, T_TOP - 0.28), p.stone, bevel=0.04)
    box("tower_cornice", (x - T_HW - 0.32, -T_HD - 0.32, T_TOP - 0.3), (x + T_HW + 0.32, T_HD + 0.32, T_TOP), p.cap, bevel=0.05)
    # Et lavt belte av bord under taket, med luker fram.
    box("tower_hoard", (x - T_HW - 0.12, -T_HD - 0.12, T_TOP), (x + T_HW + 0.12, T_HD + 0.12, T_TOP + 1.0), p.boards, bevel=0.02)
    for cx in (x - T_HW - 0.12, x + T_HW + 0.12):
        for cy in (-T_HD - 0.12, T_HD + 0.12):
            stick("hoard_post", (cx, cy, T_TOP), (cx, cy, T_TOP + 1.05), 0.22, 0.22, p.timber, ("timber", rng.randrange(4)), seed=rng.random() * 9)
    for xx in (x - 0.9, x + 0.9):
        box("hatch", (xx - 0.3, -T_HD - 0.15, T_TOP + 0.3), (xx + 0.3, -T_HD - 0.13, T_TOP + 0.78), p.soot, bevel=0)
    top = hip_roof(p, "gate_roof", T_HW + 0.12, T_HD + 0.12, T_TOP + 1.05, 5.2, 0.75, p.roof, kick=0.4, thick=0.3, at=(x, 0))
    lathe_at("finial", [(0.12, top - 0.25), (0.05, top + 0.1), (0.12, top + 0.25), (0.12, top + 0.36), (0.035, top + 0.5), (0.025, top + 1.2), (0.0, top + 1.4)], 6, p.iron, (x, 0))
    flag = box("vane", (0.03, -0.012, top + 0.75), (0.55, 0.012, top + 1.05), p.iron, bevel=0)
    transform([flag], Matrix.Translation((x, 0, 0)))


def lathe_at(name, prof, n, mat, at):
    ob = lathe(name, prof, n, mat)
    transform([ob], Matrix.Translation((at[0], at[1], 0)))
    return ob


def arch_pts(r, zs, n, a0=math.pi, a1=0.0):
    return [(r * math.cos(a0 + (a1 - a0) * k / (n - 1)), zs + r * math.sin(a0 + (a1 - a0) * k / (n - 1))) for k in range(n)]


def slab(name, y0, y1, pts, mat):
    """Et polygon i xz (ikke nødvendigvis konvekst) trukket ut langs y."""
    bm = bmesh.new()
    a = [bm.verts.new((x, y0, z)) for x, z in pts]
    b = [bm.verts.new((x, y1, z)) for x, z in pts]
    bm.faces.new(a[::-1])
    bm.faces.new(b)
    for i in range(len(pts)):
        j = (i + 1) % len(pts)
        bm.faces.new((a[i], a[j], b[j], b[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = kit._object(name, bm, mat)
    town.uv_box(ob)
    return ob


def whole_image(ob, w, h):
    """Hele bildet over flaten (dørfløyer): u over bredden, v over høyden."""
    me = ob.data
    bm = bmesh.new()
    bm.from_mesh(me)
    uvl = bm.loops.layers.uv.verify()
    for f in bm.faces:
        for lp in f.loops:
            c = lp.vert.co
            lp[uvl].uv = (c.x / w, c.z / h)
    bm.to_mesh(me)
    bm.free()


def gatehouse():
    p = Mats()
    rng = random.Random(23)
    r = G_PW / 2
    zs = G_SPRING
    D = G_D
    for sx in (-1, 1):
        flank_tower(p, sx, rng)
    # Midtblokken over portrommet: buen er skåret ut av tverrsnittet, så
    # flatene langs buen blir tønnehvelvet. Den står litt inn fra tårnene.
    arc = arch_pts(r, zs, 13)
    front = [(-r, -0.8), (-r, zs)] + arc[1:-1] + [(r, zs), (r, -0.8), (r + 0.3, -0.8), (r + 0.3, MID_TOP), (-r - 0.3, MID_TOP), (-r - 0.3, -0.8)]
    slab("gate_mid", -D / 2, D / 2, front, p.stone)
    # Gulvet i portrommet: store heller.
    box("gate_floor", (-r - 0.05, -T_HD - 0.5, -0.3), (r + 0.05, T_HD + 0.5, 0.03), p.cap, bevel=0.03)
    # Buestein rundt åpningen foran og bak, med sluttstein.
    for y, frame in ((-D / 2, ((-r - 1.0, -D / 2), (1.0, 0.0))), (D / 2, ((r + 1.0, D / 2), (-1.0, 0.0)))):
        voussoirs(frame, r + 1.0, zs, r, -1, 0.0, p.cap, n=11, depth=0.5, proud=0.12, bevel=0.03)
    # Bindbjelker under hvelvet i portrommet.
    for k in range(4):
        y = -D / 2 + 0.9 + (D - 1.8) * k / 3
        stick("vault_beam", (-r, y, zs - 0.12), (r, y, zs - 0.12), 0.24, 0.3, p.timber, ("timber", k % 4), seed=k)
    # Bjelkeender gjennom muren foran, med skråstøtter, og en smal bordgang
    # (hurdis) på dem med brystning av bord.
    zb_ = 7.45
    for k, xx in enumerate((-2.25, -1.2, 1.2, 2.25)):
        stick("beam_end", (xx, -D / 2 + 0.3, zb_), (xx, -D / 2 - 0.85, zb_), 0.26, 0.3, p.timber, ("timber", k % 4), seed=k + 3)
        stick("beam_brace", (xx, -D / 2 - 0.02, 6.75), (xx, -D / 2 - 0.7, zb_ - 0.12), 0.16, 0.18, p.timber, ("timber", (k + 1) % 4), seed=k + 7)
    box("hoard_floor", (-r + 0.02, -D / 2 - 0.95, zb_ + 0.15), (r - 0.02, -D / 2, zb_ + 0.27), p.boards, bevel=0.02)
    box("hoard_front", (-r + 0.02, -D / 2 - 0.98, zb_ + 0.27), (r - 0.02, -D / 2 - 0.88, 8.5), p.boards, bevel=0.02)
    for xx in (-r + 0.1, -0.9, 0.9, r - 0.1):
        stick("hoard_post", (xx, -D / 2 - 0.93, zb_ + 0.12), (xx, -D / 2 - 0.93, 8.62), 0.14, 0.14, p.timber, ("timber", 2), seed=xx)
    # Våpenskjoldet over buen: en innfelt steinplate.
    box("panel_frame", (-0.62, -D / 2 - 0.08, 6.28), (0.62, -D / 2 + 0.02, 7.08), p.cap, bevel=0.04)
    box("panel", (-0.46, -D / 2 - 0.12, 6.4), (0.46, -D / 2 - 0.06, 6.96), layer("wall_stone", (0.72, 0.4, 0.32)), bevel=0.03)
    # Portgitteret, heist opp: eik med jernspisser, synlig i buen.
    gy = -D / 2 + 0.45
    for k in range(11):
        xx = -r + 0.3 + (2 * r - 0.6) * k / 10
        topz = zs + math.sqrt(max(0.0, r * r - xx * xx)) + 0.1
        stick("gate_bar", (xx, gy, zs + 1.15), (xx, gy, topz), 0.12, 0.12, p.timber, ("timber", k % 4), bevel=0.0, seed=k)
        lathe_at("spike", [(0.06, zs + 1.15), (0.0, zs + 0.88)], 4, p.iron, (xx, gy))
    for z in (zs + 1.35, zs + 1.95):
        hw = math.sqrt(max(0.0, r * r - (z - zs) ** 2)) - 0.05
        stick("gate_rail", (-hw, gy - 0.1, z), (hw, gy - 0.1, z), 0.1, 0.12, p.timber, ("timber", 1), bevel=0.0)
    # Portfløyene slått inn mot veggene i portrommet, med jernbånd.
    lw, lh = r - 0.08, zs - 0.45
    for sx in (-1, 1):
        leaf = box("leaf", (0, -0.06, 0), (lw, 0.06, lh), p.door_dark, bevel=0.0)
        whole_image(leaf, lw, lh)
        parts = [leaf]
        for z in (0.35, lh * 0.5, lh - 0.35):
            parts.append(box("band", (0.05, -0.095, z - 0.05), (lw - 0.05, 0.095, z + 0.05), p.iron, bevel=0))
        # Hengslet ved veggen bak portgitteret; fløyen ligger langs veggen.
        transform(parts, Matrix.Translation((sx * (r - 0.12), -D / 2 + 1.0, 0.03)) @ Matrix.Rotation(math.pi / 2, 4, "Z"))
    # Brystvernet foran og bak på midtblokken, med murtinder.
    for out in (-1, 1):
        y_out = out * (D / 2 + 0.12)
        y_in = out * (D / 2 - 0.5)
        lo, hi = min(y_in, y_out), max(y_in, y_out)
        box("course", (-r - 0.3, lo - (0.1 if out < 0 else 0), MID_TOP - 0.25), (r + 0.3, hi + (0.1 if out > 0 else 0), MID_TOP + 0.02), p.cap, bevel=0.04)
        box("parapet", (-r - 0.3, lo, MID_TOP), (r + 0.3, hi, MID_TOP + 1.0), p.stone, bevel=0.03)
        for k in range(4):
            xx = -r - 0.3 + 0.55 + (2 * r + 0.6 - 1.1) * k / 3
            h = 0.9 + rng.uniform(-0.06, 0.06)
            box("merlon", (xx - 0.5, lo + 0.02, MID_TOP + 1.0), (xx + 0.5, hi - 0.02, MID_TOP + 1.0 + h), p.stone, bevel=0.03)
            box("merlon_cap", (xx - 0.55, lo - 0.05, MID_TOP + 1.0 + h), (xx + 0.55, hi + 0.05, MID_TOP + 1.16 + h), p.cap, bevel=0.04)
            ys = lo + 0.005 if out < 0 else hi - 0.005
            box("slit", (xx - 0.05, ys - 0.03, MID_TOP + 1.15), (xx + 0.05, ys + 0.03, MID_TOP + 1.75), p.soot, bevel=0)
    # Murgangen og et lite pulttak av spon over den, på stolper; det går inn
    # under takskjegget på tårnene.
    box("walk", (-r - 0.3, -D / 2 + 0.4, MID_TOP - 0.05), (r + 0.3, D / 2 - 0.4, MID_TOP + 0.08), p.cap, bevel=0.02)
    yb, yf = D / 2 - 0.7, -D / 2 + 0.7
    zb, zf = MID_TOP + 1.95, MID_TOP + 2.5
    for xx in (-r + 0.2, 0.0, r - 0.2):
        for yy, zz in ((yf, zf), (yb, zb)):
            stick("walk_post", (xx, yy, MID_TOP + 0.05), (xx, yy, zz), 0.2, 0.2, p.timber, ("timber", rng.randrange(4)), seed=rng.random() * 9)
        stick("walk_tie", (xx, yf - 0.1, zf - 0.15), (xx, yb + 0.1, zb - 0.15), 0.16, 0.22, p.timber, ("timber", 3), seed=xx)
    for yy, zz in ((yf, zf), (yb, zb)):
        stick("walk_plate", (-r - 0.1, yy, zz), (r + 0.1, yy, zz), 0.2, 0.22, p.timber, ("timber", 1), seed=yy)
    over = 0.75
    slope = (zb - zf) / (yb - yf)
    y0, y1 = yf - over, yb + over
    z0, z1 = zf + 0.12 + slope * (y0 - yf), zb + 0.12 + slope * (y1 - yb)
    rings = []
    for xx, dz in ((-r, 0.0), (0.0, -0.12), (r, 0.0)):
        rings.append([(xx, y0, z0 + dz), (xx, y1, z1 + dz), (xx, y1, z1 + dz - 0.2), (xx, y0, z0 + dz - 0.2)])
    roof = mesh("walk_roof", rings, p.roof, closed=True)
    me = roof.data
    bm = bmesh.new()
    bm.from_mesh(me)
    uvl = bm.loops.layers.uv.verify()
    for f in bm.faces:
        for lp in f.loops:
            c = lp.vert.co
            lp[uvl].uv = (c.x, -(c.y - y0) * 1.05)
    bm.to_mesh(me)
    bm.free()
    # Steiner i foten foran.
    for k in range(6):
        sx = 1 if k % 2 else -1
        xx = sx * rng.uniform(T_X - T_HW + 0.5, T_X + T_HW - 0.2)
        w = rng.uniform(0.55, 0.8)
        b = box("foot_stone", (-w / 2, -0.22, -0.2), (w / 2, 0.22, 0.2), p.dark_stone, bevel=0.06)
        transform([b], Matrix.Translation((xx, -T_HD - 0.62, 0.05)) @ Matrix.Rotation(-0.3, 4, "X"))
    engine_uv()
    return p


MODELS = {"town_wall": town_wall, "wall_tower": wall_tower, "gatehouse": gatehouse}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        town.LIT = False
        MODELS[name]()
        town.finish(f"{OUT}/{name}.glb", max_edge=100.0)
