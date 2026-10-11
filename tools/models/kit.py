"""Byggesettet for den malte byen (docs/ny-stil.md): deler som husene settes
sammen av, med de malte teksturlagene og få trekanter.

Detaljene (stein, planker, takstein, årer) ligger i teksturene. Det som
synes i silhuetten er geometri: tykke bjelker med én liten avfasing,
utkragede etasjer, høye tak som henger litt i midten, karmer og piper.
Okklusjonen bakes i hjørnefargen av `town.finish`.

Akser som i town.py: X til høyre, −Y fram (gata), Z opp, origo midt på
grunnflaten i bakkehøyde.

Uv-ene er i meter for flisbare lag. Bjelkene får én rad i `timber` (fire
bjelker per meter i bildet), lister og karmer en stripe i trim-arket
(`trim`, åtte striper), og dørene hele bildet."""

import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
import town  # noqa: E402
from mathutils import Euler, Matrix, Vector  # noqa: E402
from town import collide, daylight, ensure_uv, layer, plain  # noqa: E402

PIECES = town.PIECES

# FRISTAD_STYLE=flat: ingen bildeteksturer, bare flate, malte farger per
# materiale (prøven «farger rett på modellene»). Formen og okklusjonen i
# hjørnefargen gjør resten.
FLAT = {
    "plaster": 0xF0D9A4, "plaster_white": 0xF2EBD8, "plaster_red": 0xC9654A, "plaster_old": 0xE6D2A2,
    "timber": 0x5C3A22, "timber_grey": 0x857F74, "thatch": 0xE2AE48, "shingles": 0xA9673A, "shingles_tar": 0x3E2C22,
    "slate": 0x55647C, "tiles": 0xC2563A, "stone": 0x9C9080, "wall_stone": 0x8E8A84, "ruin": 0x8A8676,
    "ashlar": 0xD4B67C, "planks": 0xA4683A, "oak_planed": 0xB07848, "trim": 0x6E4628, "iron": 0x3C3C44, "door": 0x8C5630,
    "door_b": 0x4C3222, "weatherboard": 0x9C6C40, "weatherboard_grey": 0x9A948A, "cladding": 0x4C3628,
    "logs": 0x9C6434, "wattle": 0x8C7048, "straw_floor": 0xD8B262, "flagstone": 0xA29686, "linen": 0xE0D4B8,
    "wool": 0xE8E0D0, "leather": 0x7C4C30, "cobble": 0x9A9080, "sod": 0x6E8A3A, "chimney": 0x9A5440,
}


def _lin(c):
    return ((c / 255.0 + 0.055) / 1.055) ** 2.4 if c > 10 else c / 255.0 / 12.92


if os.environ.get("FRISTAD_STYLE") == "flat":
    _layer = town.layer

    def _flat(name, tint=(1.0, 1.0, 1.0), rough=0.5):
        h = FLAT.get(name, 0xB0A090)
        rgb = tuple(_lin((h >> sh) & 255) * t for sh, t in zip((16, 8, 0), tint))
        key = f"flat_{name}_{'_'.join(f'{v:.3f}' for v in rgb)}"
        return plain(key, rgb, 0.8)

    town.layer = _flat
    layer = _flat

# Stripene i trim-arket, ovenfra.
CARVED, DARK_BEAM, MOULDING, SILL, BRICKS, IRON_BAND, ROUGH_PLANK, ROPE = range(8)


class Paint:
    """Materialene til ett hus: malte lag med hvit tone."""

    def __init__(self, plaster="plaster", timber="timber", roof="thatch", plinth="stone"):
        self.plaster = layer(plaster)
        self.timber = layer(timber)
        self.roof = layer(roof)
        self.stone = layer(plinth)
        self.floor = layer("planks")
        # Høvlet eik til bordplater og møbler.
        self.oak = layer("oak_planed")
        self.trim = layer("trim")
        self.iron = layer("iron", rough=0.45)
        self.door = layer("door")
        self.door_dark = layer("door_b")
        self.boards = layer("weatherboard")
        self.cladding = layer("cladding")
        self.flag = layer("flagstone")
        self.straw = layer("straw_floor")
        self.wall_stone = layer("wall_stone")
        self.glass = plain("glass", (0.05, 0.06, 0.06), 0.05)
        self.fire = plain("fire_hearth", (1.0, 0.55, 0.2), 0.5)
        self.soot = layer("stone", (0.25, 0.22, 0.2))


# --- Grunnformer ------------------------------------------------------------


def _bevel(ob, width, segments=1):
    if width <= 0:
        return
    mod = ob.modifiers.new("bevel", "BEVEL")
    mod.width = width
    mod.segments = segments
    mod.limit_method = "ANGLE"
    mod.angle_limit = math.radians(40)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.object.modifier_apply(modifier=mod.name)


def _object(name, bm, mat):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    if isinstance(mat, (list, tuple)):
        for m in mat:
            me.materials.append(m)
    else:
        me.materials.append(mat)
    # Uv-laget fra bmesh beholdes (det får bare navnet eksporten venter).
    if len(me.uv_layers) == 1 and me.uv_layers[0].name != "UVMap":
        me.uv_layers[0].name = "UVMap"
    ensure_uv(me)
    PIECES.append(ob)
    return ob


def box(name, lo, hi, mat, bevel=0.025, uv="box"):
    """En boks fra hjørnet `lo` til `hi` i husets akser (ikke dreid), med uv i
    meter fra de faktiske koordinatene, så naboflater møtes i teksturen."""
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    lo, hi = Vector(lo), Vector(hi)
    for v in bm.verts:
        v.co = Vector(((lo.x + hi.x) / 2 + v.co.x * (hi.x - lo.x), (lo.y + hi.y) / 2 + v.co.y * (hi.y - lo.y), (lo.z + hi.z) / 2 + v.co.z * (hi.z - lo.z)))
    ob = _object(name, bm, mat)
    _bevel(ob, min(bevel, min(hi - lo) * 0.3))
    if uv == "box":
        town.uv_box(ob)
    return ob


def stick(name, a, b, w, h, mat, strip=None, bevel=0.03, roll=0.0, seed=0):
    """En bjelke fra a til b (w bred, h høy), med kantene langs bjelken
    avfaset (åttekantet profil, 28 trekanter). Med `strip` = ("timber", k)
    får den bjelke nummer k i timber-laget; ("trim", k) stripe k i
    trim-arket; ellers uv i meter langs bjelken. `bevel` = 0 gir en ren
    boks (småting)."""
    a, b = Vector(a), Vector(b)
    d = b - a
    L = d.length
    bm = bmesh.new()
    c = min(bevel, w * 0.3, h * 0.3)
    if c > 0:
        prof = [(w / 2 - c, h / 2), (-w / 2 + c, h / 2), (-w / 2, h / 2 - c), (-w / 2, -h / 2 + c), (-w / 2 + c, -h / 2), (w / 2 - c, -h / 2), (w / 2, -h / 2 + c), (w / 2, h / 2 - c)]
    else:
        prof = [(w / 2, h / 2), (-w / 2, h / 2), (-w / 2, -h / 2), (w / 2, -h / 2)]
    ends = [[bm.verts.new((x, y, z)) for y, z in prof] for x in (-L / 2, L / 2)]
    n = len(prof)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((ends[0][i], ends[0][j], ends[1][j], ends[1][i]))
    bm.faces.new(ends[0][::-1])
    bm.faces.new(ends[1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = _object(name, bm, mat)
    fit_strip(ob, strip, seed)
    q = Vector((1, 0, 0)).rotation_difference(d.normalized())
    ob.rotation_mode = "QUATERNION"
    ob.rotation_quaternion = q @ Euler((roll, 0, 0)).to_quaternion()
    ob.location = (a + b) / 2
    return ob


def fit_strip(ob, strip, seed=0):
    """Uv langs lokal x. Flatene på tvers får en stripe av bildet: rad k av
    fire i timber-laget, stripe k av åtte i trim-arket."""
    me = ob.data
    bm = bmesh.new()
    bm.from_mesh(me)
    uvl = bm.loops.layers.uv.verify()
    off = random.Random(seed).uniform(0, 7)
    for f in bm.faces:
        n = f.normal
        ax = max(range(3), key=lambda i: abs(n[i]))
        across = 2 if ax == 1 else 1
        if ax == 0:
            # Endeflatene: stripen over høyden, litt bredde.
            ls = [lp.vert.co for lp in f.loops]
            u_of = lambda p: p.y
            across = 2
        else:
            ls = [lp.vert.co for lp in f.loops]
            u_of = lambda p: p.x
        lo = min(p[across] for p in ls)
        hi = max(p[across] for p in ls)
        span = max(hi - lo, 1e-4)
        for lp in f.loops:
            p = lp.vert.co
            t = (p[across] - lo) / span
            if strip is None:
                lp[uvl].uv = (u_of(p) + off, p[across])
            elif strip[0] == "timber":
                k = strip[1]
                lp[uvl].uv = (u_of(p) + off, k * 0.25 + 0.02 + t * 0.21)
            else:
                k = strip[1]
                v0, v1 = 1 - (k + 1) / 8, 1 - k / 8
                lp[uvl].uv = ((u_of(p) + off) * 0.125 / span, v0 + 0.006 + t * (v1 - v0 - 0.012))
    bm.to_mesh(me)
    bm.free()


def slab_grid(name, frame, length, z0, z1, thick, holes, mat, bevel=0.0):
    """En vegg som plater rundt åpningene. `frame` er (start (x, y), retning
    (x, y)) for veggens midtlinje; lokalt går s langs veggen fra 0 til
    `length`, t på tvers (±thick/2) og z opp. `holes` er (s0, s1, z0, z1).
    Uv i meter langs veggen, så pussen går sammenhengende rundt åpningene."""
    (px, py), (dx, dy) = frame
    ss = sorted({0.0, length, *[h[0] for h in holes], *[h[1] for h in holes]})
    zs = sorted({z0, z1, *[h[2] for h in holes], *[h[3] for h in holes]})
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    nx, ny = -dy, dx
    for i in range(len(ss) - 1):
        for j in range(len(zs) - 1):
            sa, sb, za, zb = ss[i], ss[i + 1], zs[j], zs[j + 1]
            cs, cz = (sa + sb) / 2, (za + zb) / 2
            if any(h[0] <= cs <= h[1] and h[2] <= cz <= h[3] for h in holes):
                continue
            if sb - sa < 1e-4 or zb - za < 1e-4:
                continue
            corners = {}
            for si, s in enumerate((sa, sb)):
                for ti, t in enumerate((-thick / 2, thick / 2)):
                    for zi, z in enumerate((za, zb)):
                        corners[(si, ti, zi)] = bm.verts.new((px + dx * s + nx * t, py + dy * s + ny * t, z))
            quads = [
                ((0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)),
                ((1, 1, 0), (0, 1, 0), (0, 1, 1), (1, 1, 1)),
                ((0, 1, 0), (0, 0, 0), (0, 0, 1), (0, 1, 1)),
                ((1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)),
                ((0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)),
                ((0, 1, 0), (1, 1, 0), (1, 0, 0), (0, 0, 0)),
            ]
            for q in quads:
                f = bm.faces.new([corners[k] for k in q])
                for lp, k in zip(f.loops, q):
                    s = (sa, sb)[k[0]]
                    z = (za, zb)[k[2]]
                    t = (-thick / 2, thick / 2)[k[1]]
                    if q in (quads[2], quads[3]):
                        lp[uvl].uv = (t, z)
                    elif q in (quads[4], quads[5]):
                        lp[uvl].uv = (s, t)
                    else:
                        lp[uvl].uv = (s, z)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = _object(name, bm, mat)
    _bevel(ob, bevel)
    return ob


def wall_frame(p0, p1):
    """(start, retning) og lengde for en vegg fra p0 til p1 (x, y)."""
    dx, dy = p1[0] - p0[0], p1[1] - p0[1]
    L = math.hypot(dx, dy)
    return ((p0[0], p0[1]), (dx / L, dy / L)), L


def at_wall(frame, s, t, z):
    (px, py), (dx, dy) = frame
    return Vector((px + dx * s - dy * t, py + dy * s + dx * t, z))


# --- Bindingsverk -------------------------------------------------------------


def half_timbered(p, name, p0, p1, z0, z1, out, openings, rng, thick=0.3, beam=0.34, collide_it=True):
    """En bindingsverksvegg fra p0 til p1 mellom z0 og z1: pussplate med
    åpninger, og bjelker utenpå mot `out` (+1 eller −1 langs veggens
    normal): svill, stavler ved endene og åpningene, losholt og skråbånd.
    `openings` er (s midt, z0, z1, bredde)."""
    frame, L = wall_frame(p0, p1)
    holes = [(s - w / 2, s + w / 2, za, zb) for s, za, zb, w in openings]
    slab_grid(name, frame, L, z0, z1, thick, holes, p.plaster)
    face = out * (thick / 2 + 0.04)
    k = rng.randrange(4)
    # Litt skjevt, som håndhogd tømmer i et gammelt hus.
    j = lambda a: rng.uniform(-a, a)
    # Svill og stavlægje.
    stick("sill", at_wall(frame, -0.08, face, z0 + beam / 2 + j(0.03)), at_wall(frame, L + 0.08, face, z0 + beam / 2 + j(0.03)), 0.2, beam, p.timber, ("timber", k), seed=rng.random() * 99)
    stick("plate", at_wall(frame, -0.08, face, z1 - beam / 2 + j(0.04)), at_wall(frame, L + 0.08, face, z1 - beam / 2 + j(0.04)), 0.2, beam, p.timber, ("timber", (k + 1) % 4), seed=rng.random() * 99)
    # Stavlene: ved endene, rundt åpningene og ellers med 1,4 m imellom.
    posts = {beam / 2, L - beam / 2}
    for s, za, zb, w in openings:
        posts.add(s - w / 2 - beam / 2)
        posts.add(s + w / 2 + beam / 2)
    xs = sorted(posts)
    filled = []
    for a, b in zip(xs, xs[1:]):
        filled.append(a)
        n = int((b - a) / 1.9)
        for i in range(1, n + 1):
            filled.append(a + (b - a) * i / (n + 1))
    filled.append(xs[-1])
    posts = sorted(set(round(x, 3) for x in filled))

    def in_opening(s, z):
        return any(abs(s - so) < w / 2 + 0.02 and za - 0.02 < z < zb + 0.02 for so, za, zb, w in openings)

    for s in posts:
        if in_opening(s, (z0 + z1) / 2):
            continue
        stick("post", at_wall(frame, s + j(0.04), face, z0 + beam), at_wall(frame, s + j(0.07), face, z1 - beam), 0.19, beam, p.timber, ("timber", rng.randrange(4)), seed=rng.random() * 99)
    # Losholt i brysthøyde mellom stavlene, og skråbånd i felt uten åpning.
    mid = z0 + (z1 - z0) * 0.48
    for a, b in zip(posts, posts[1:]):
        if b - a < 0.5:
            continue
        sa, sb = a + beam / 2, b - beam / 2
        if not any(in_opening((sa + sb) / 2, z) for z in (mid, z0 + 0.4, z1 - 0.4)):
            stick("rail", at_wall(frame, sa, face, mid + j(0.06)), at_wall(frame, sb, face, mid + j(0.06)), 0.17, beam * 0.85, p.timber, ("timber", rng.randrange(4)), seed=rng.random() * 99)
            if rng.random() < 0.45:
                lo, hi = (sa, sb) if rng.random() < 0.5 else (sb, sa)
                stick("brace", at_wall(frame, lo, face, z0 + beam), at_wall(frame, hi, face, mid - beam * 0.4), 0.17, beam * 0.8, p.timber, ("timber", rng.randrange(4)), seed=rng.random() * 99)
    if collide_it:
        wall_colliders(frame, L, z0, z1, thick, holes)
    return frame, L


def wall_colliders(frame, L, z0, z1, thick, holes):
    """Kolliderne til en vegg: biter mellom åpningene som går fra gulvet
    (dørene) eller over og under (vinduene)."""
    cuts = sorted([h for h in holes if h[2] < z0 + 0.3], key=lambda h: h[0])
    s = 0.0
    for h in cuts + [(L, L, 0, 0)]:
        if h[0] - s > 0.05:
            _wall_box(frame, s, h[0], z0, z1, thick)
        if h[1] > h[0]:
            _wall_box(frame, h[0], h[1], h[3], z1, thick)
        s = h[1]


def _wall_box(frame, sa, sb, za, zb, thick):
    (px, py), (dx, dy) = frame
    c = at_wall(frame, (sa + sb) / 2, 0, (za + zb) / 2)
    yaw = math.atan2(dy, dx)
    collide(tuple(c), (sb - sa, thick + 0.1, zb - za), yaw)


# --- Vinduer og dører ---------------------------------------------------------


# Vinduskarmene (eieren, 11. oktober), felles for alle vinduer i byggesettet:
# karm av bjelker rundt åpningen (sider og overligger), vannbrett nederst
# med helling utover, og sprosser der ruta er delt, i husets trevirke som
# dørkarmene.
FRAME_W = 0.09  # karmen: bredde langs veggen
FRAME_OUT = 0.05  # så langt karmen står ut fra veggen
FRAME_IN = 0.03  # og så langt den går inn i veggen
SILL_TILT = math.radians(9)  # vannbrettets helling utover


def window_frame(p, frame, s, zc, w, h, out, thick, sill=True):
    """Karmen rundt en åpning på sida `out`: to sidekarmer, overligger og
    vannbrett (med helling utover) eller en flat sålbenk (`sill=False`,
    innsida)."""
    t = FRAME_W
    face = out * (thick / 2 + (FRAME_OUT - FRAME_IN) / 2)
    depth = FRAME_OUT + FRAME_IN
    for sx in (-1, 1):
        x = s + sx * (w / 2 + t / 2)
        stick("win_jamb", at_wall(frame, x, face, zc - h / 2), at_wall(frame, x, face, zc + h / 2 + t), depth, t, p.trim, ("trim", DARK_BEAM), bevel=0.0)
    stick("win_head", at_wall(frame, s - w / 2 - t, face, zc + h / 2 + t / 2), at_wall(frame, s + w / 2 + t, face, zc + h / 2 + t / 2), depth, t, p.trim, ("trim", DARK_BEAM), bevel=0.0)
    (px, py), (dx, dy) = frame
    yaw = math.atan2(dy, dx)
    if sill:
        # Vannbrettet: et bord som stikker lenger ut og heller utover, så
        # regnet renner av.
        sw = w / 2 + t + 0.04
        board = box("win_sill", (-sw, -0.02 - FRAME_IN, -0.04), (sw, FRAME_OUT + 0.07, 0.0), p.trim, bevel=0.0)
        c = at_wall(frame, s, out * thick / 2, zc - h / 2)
        # Lokal +y peker ut av veggen; dreid om x så ytterkanten går ned.
        rot = Matrix.Rotation(yaw + (0 if out > 0 else math.pi), 4, "Z") @ Matrix.Rotation(-SILL_TILT, 4, "X")
        board.matrix_world = Matrix.Translation(c) @ rot @ board.matrix_world
    else:
        stick("win_sill", at_wall(frame, s - w / 2 - t, face, zc - h / 2 - 0.02), at_wall(frame, s + w / 2 + t, face, zc - h / 2 - 0.02), depth, 0.04, p.trim, ("trim", DARK_BEAM), bevel=0.0)


def mullions(p, frame, s, zc, w, h, out, bar=0.05):
    """Sprossene i ruta: kryss i store ruter, to loddrette i brede og lave,
    én i middels brede; små ruter er hele."""
    uprights = []
    if w >= 0.75 and h >= 0.8:
        uprights, cross = [s], True
    elif w >= 1.1:
        uprights, cross = [s - w / 6, s + w / 6], False
    elif w >= 0.6:
        uprights, cross = [s], False
    else:
        cross = False
    for x in uprights:
        stick("mullion", at_wall(frame, x, out * 0.02, zc - h / 2), at_wall(frame, x, out * 0.02, zc + h / 2), bar, bar, p.trim, ("trim", DARK_BEAM), bevel=0.0)
    if cross:
        stick("transom", at_wall(frame, s - w / 2, out * 0.02, zc + h * 0.1), at_wall(frame, s + w / 2, out * 0.02, zc + h * 0.1), bar, bar, p.trim, ("trim", DARK_BEAM), bevel=0.0)


def window(p, frame, s, zc, w, h, out, thick=0.3, shutters=True, rng=None, lit=True, inside=False):
    """Vindu: karm av bjelker med vannbrett (`window_frame`), sprosser der
    ruta er delt (`mullions`), blyglass (gløder om natta) og skodder slått
    opp. `inside` gir karm på innsida også (hus med interiør)."""
    window_frame(p, frame, s, zc, w, h, out, thick)
    mullions(p, frame, s, zc, w, h, out)
    if inside:
        window_frame(p, frame, s, zc, w, h, -out, thick, sill=False)
    t = FRAME_W
    face = out * (thick / 2 + 0.04)
    # Glasset litt innfelt, og blysprosser på skrå (rutemønster).
    g = at_wall(frame, s, out * 0.02, zc)
    (px, py), (dx, dy) = frame
    yaw = math.atan2(dy, dx)
    pane = town.block("glass", (w, 0.02, h), p.glass, at=tuple(g), rot=(0, 0, yaw), bevel=0)
    PIECES.remove(pane)
    PIECES.append(pane)
    if lit:
        daylight(pane, (-dy * -out, dx * -out, 0))
    if shutters:
        # Skoddene står slått opp, nesten flatt mot veggen ved siden av vinduet.
        r = rng or random.Random(1)
        sw = w / 2 + 0.04
        for sx in (-1, 1):
            tilt = math.radians(r.uniform(8, 18))
            hinge = at_wall(frame, s + sx * (w / 2 + t), face + out * 0.03, zc)
            ob = box("shutter", (0, -0.03, -h / 2 - 0.03), (sw, 0.03, h / 2 + 0.03), p.cladding, bevel=0.0)
            hinges = [box("shutter_hinge", (0.02, -0.045, zz - 0.03), (sw * 0.85, 0.045, zz + 0.03), p.iron, bevel=0.0) for zz in (h * 0.25,)]
            # Lokal x peker bort fra vinduet langs veggen, dreid litt ut.
            ang = yaw + (0 if sx > 0 else math.pi) + sx * out * tilt
            for o in [ob] + hinges:
                o.matrix_world = Matrix.Translation(hinge) @ Matrix.Rotation(ang, 4, "Z") @ o.matrix_world


def door(p, frame, s, z0, w, h, out, thick=0.3, open_in=True, dark=False):
    """Dør: karm av mørke bjelker og et overstykke med utskåret list, og
    døra (hele det malte dørbildet) slått inn i huset."""
    face = out * (thick / 2 + 0.06)
    t = 0.18
    for sx in (-1, 1):
        x = s + sx * (w / 2 + t / 2)
        stick("door_post", at_wall(frame, x, face, z0), at_wall(frame, x, face, z0 + h + t), 0.2, t, p.trim, ("trim", DARK_BEAM), bevel=0.03)
    stick("door_head", at_wall(frame, s - w / 2 - t - 0.1, face + out * 0.02, z0 + h + t / 2 + 0.04), at_wall(frame, s + w / 2 + t + 0.1, face + out * 0.02, z0 + h + t / 2 + 0.04), 0.24, t + 0.08, p.trim, ("trim", CARVED), bevel=0.03)
    (px, py), (dx, dy) = frame
    yaw = math.atan2(dy, dx)
    leaf_t = 0.07
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co = Vector((v.co.x * w + w / 2, v.co.y * leaf_t, v.co.z * h + h / 2))
    ob = _object("door_leaf", bm, p.door_dark if dark else p.door)
    uvl_bm = bmesh.new()
    uvl_bm.from_mesh(ob.data)
    uvl = uvl_bm.loops.layers.uv.verify()
    for f in uvl_bm.faces:
        for lp in f.loops:
            c = lp.vert.co
            lp[uvl].uv = (c.x / w if f.normal.y * out >= 0 else 1 - c.x / w, c.z / h)
    uvl_bm.to_mesh(ob.data)
    uvl_bm.free()
    # Hengslet på venstre side sett utenfra, slått inn.
    hinge = at_wall(frame, s - w / 2, -out * (thick / 2 - 0.05), z0)
    # Slått inn i huset (mot −out langs veggens normal).
    swing = -out * math.radians(100)
    ob.rotation_euler = Euler((0, 0, yaw + (swing if open_in else 0)))
    ob.location = hinge
    # Trinn av hogd stein foran.
    return ob


# --- Tak, gavler, piper -------------------------------------------------------


def gable_roof(p, x0, x1, y0, y1, z_eave, pitch, overhang=0.9, gable_over=0.7, thick=0.42, sag=0.32, kick=0.35, name="roof"):
    """Saltak med mønet langs x, fra veggene y0..y1 og x0..x1: to tykke
    takflater med avrundet takskjegg som henger litt i midten, og en
    mønerull. Gir høyden på mønet."""
    half = (y1 - y0) / 2
    yc = (y0 + y1) / 2
    rise = half * math.tan(math.radians(pitch))
    ridge = z_eave + rise
    run = half + overhang
    slope = run / math.cos(math.radians(pitch))
    xa, xb = x0 - gable_over, x1 + gable_over
    n = 8
    cp, sp = math.cos(math.radians(pitch)), math.sin(math.radians(pitch))
    for side in (-1, 1):
        bm = bmesh.new()
        uvl = bm.loops.layers.uv.verify()
        g = {}
        bm_pts = {}
        for i in range(n + 1):
            u = i / n
            x = xa + (xb - xa) * u
            dip = sag * math.sin(math.pi * u)
            # Tre rader nedover: mønet (som svaier litt), et knekkpunkt, og
            # takfoten som bretter seg ut (`kick`) og svaier mest i midten.
            # Knekken ligger rett over veggen, så svaien bare tar takskjegget
            # (ellers stikker stavlægjen opp gjennom taket).
            rows = ((0.0, 0.5, 0.0), (half / cp, 0.0, 0.0), (slope, 1.0, kick))
            for j, (d, sg, up) in enumerate(rows):
                yy = yc + side * d * cp
                zz = ridge - d * sp - dip * sg + up
                g[(i, j, 0)] = bm.verts.new((x, yy, zz))
                bm_pts[(i, j)] = (yy, zz - 0.12)
                g[(i, j, 1)] = bm.verts.new((x, yy - side * thick * sp, zz - thick * cp))
        last = len(rows) - 1
        for i in range(n):
            for j in range(last):
                for k in (0, 1):
                    bm.faces.new([g[(i, j, k)], g[(i + 1, j, k)], g[(i + 1, j + 1, k)], g[(i, j + 1, k)]])
            for j in (0, last):
                bm.faces.new([g[(i, j, 0)], g[(i + 1, j, 0)], g[(i + 1, j, 1)], g[(i, j, 1)]])
        for i in (0, n):
            for j in range(last):
                bm.faces.new([g[(i, j, 0)], g[(i, j + 1, 0)], g[(i, j + 1, 1)], g[(i, j, 1)]])
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        # Uv: u langs mønet, v nedover takflaten (halmen og sponet renner ned).
        for f in bm.faces:
            for lp in f.loops:
                c = lp.vert.co
                lp[uvl].uv = (c.x, -abs(c.y - yc) / cp)
        ob = _object(f"{name}_{side}", bm, p.roof)
        _bevel(ob, 0.08, 2)
        # Vindskiene langs gavlkantene, oppå takflaten og etter knekken.
        for i in (0, n):
            x = xa + (xb - xa) * i / n - (0.06 if i == 0 else -0.06)
            for j in range(last):
                a_ = bm_pts[(i, j)]
                b_ = bm_pts[(i, j + 1)]
                stick("barge", (x, a_[0], a_[1] + 0.02), (x, b_[0], b_[1] + 0.02), 0.1, 0.36, p.trim, ("trim", DARK_BEAM), bevel=0.02)
    # Mønerull.
    bpy.ops.mesh.primitive_cylinder_add(vertices=8, radius=0.3, depth=xb - xa + 0.1, location=((xa + xb) / 2, yc, ridge - sag * 0.3 + 0.02), rotation=(0, math.pi / 2, 0))
    rl = bpy.context.active_object
    rl.name = "ridge_roll"
    rl.data.materials.append(p.roof)
    town.uv_box(rl, along="x")
    PIECES.append(rl)
    return ridge


def gable_wall(p, x, y0, y1, z0, ridge_z, pitch, out, rng, thick=0.3):
    """Gavltrekanten over veggen i x: puss med bjelker (stavler og en
    hanebjelke) mot `out` (±1 langs x)."""
    yc = (y0 + y1) / 2
    half = (y1 - y0) / 2
    pts = [(y0, z0), (y1, z0), (yc, ridge_z - 0.25)]
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    front = [bm.verts.new((x - thick / 2, yy, zz)) for yy, zz in pts]
    back = [bm.verts.new((x + thick / 2, yy, zz)) for yy, zz in pts]
    bm.faces.new(front[::-1])
    bm.faces.new(back)
    for i in range(3):
        j = (i + 1) % 3
        bm.faces.new((front[i], front[j], back[j], back[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        for lp in f.loops:
            c = lp.vert.co
            lp[uvl].uv = (c.y, c.z)
    _object("gable", bm, p.plaster)
    face = x + out * (thick / 2 + 0.03)
    tan = math.tan(math.radians(pitch))
    beam = 0.24
    # Stavler i gavlen og en hanebjelke.
    for frac in (-0.55, 0.0, 0.55):
        yy = yc + frac * half
        top = ridge_z - 0.25 - abs(yy - yc) * tan
        if top - z0 > 0.6:
            stick("gable_post", (face, yy, z0 + 0.05), (face, yy, top - 0.1), 0.15, beam, p.timber, ("timber", rng.randrange(4)), seed=rng.random() * 9)
    zc = z0 + (ridge_z - z0) * 0.42
    w = half - (zc - z0) / tan
    stick("collar", (face, yc - w + 0.1, zc), (face, yc + w - 0.1, zc), 0.15, beam, p.timber, ("timber", rng.randrange(4)), seed=rng.random() * 9)


def chimney(p, x, y, z0, z1, w=1.1, smoke=0.8, lean=4.0):
    """Klumpete steinpipe som smalner av oppover og lener seg litt, med en
    tykk krage. Røyken registreres øverst."""
    h = z1 - z0
    tilt = math.radians(lean)
    top = (x + math.sin(tilt) * h, y, z1)
    town.SMOKE.append(((top[0], top[1], z1 + 0.25), smoke))
    bm = bmesh.new()
    rows = []
    for k, (z, ww) in enumerate(((z0, w), (z0 + h * 0.6, w * 0.88), (z1, w * 0.8))):
        t = (z - z0) / h
        cx = x + math.sin(tilt) * h * t * t
        rows.append([bm.verts.new((cx + sx * ww / 2, y + sy * ww / 2, z)) for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))])
    for a_, b_ in zip(rows, rows[1:]):
        for i in range(4):
            j = (i + 1) % 4
            bm.faces.new((a_[i], a_[j], b_[j], b_[i]))
    bm.faces.new(rows[0][::-1])
    bm.faces.new(rows[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = _object("chimney", bm, layer("chimney"))
    _bevel(ob, 0.06)
    town.uv_box(ob)
    cw = w * 0.8
    box("chimney_cap", (top[0] - cw / 2 - 0.12, y - cw / 2 - 0.12, z1), (top[0] + cw / 2 + 0.12, y + cw / 2 + 0.12, z1 + 0.22), p.stone, bevel=0.06)
    box("chimney_hole", (top[0] - cw / 2 + 0.15, y - cw / 2 + 0.15, z1 + 0.18), (top[0] + cw / 2 - 0.15, y + cw / 2 - 0.15, z1 + 0.25), p.soot, bevel=0)


# --- Inne ---------------------------------------------------------------------


def planks(p, name, lo, hi, mat=None):
    """Plankegulv eller -bord: en flat boks med uv i meter."""
    return box(name, lo, hi, mat or p.floor, bevel=0.01)


# Trappereglene (eieren, 11. oktober), felles for alle trapper i byggesettet.
# Spilleren er en kapsel på 1,8 m som ikke skal måtte gå på kne.
RISE_MAX = 0.20  # trinnhøyde
TREAD_MIN = 0.26  # inntrinn
HEADROOM = 2.1  # fri høyde fra hvert trinn opp til det som henger over
LANDING = 1.0  # fritt gulv foran nederste og øverste trinn
WALL_PAD = 0.05  # veggkolliderne er så mye tykkere enn pussen (`_wall_box`)
STAIR_MARGIN = 0.35  # kapselens radius: så mye må trapphullet gå forbi trinnet


def stair_steps(z0, z1):
    """Færreste trinn som holder trinnhøyden under `RISE_MAX`."""
    return max(1, math.ceil((z1 - z0) / RISE_MAX - 1e-6))


def stair_run(z0, z1, n=None):
    """Lengden trappa trenger (n trinn à `TREAD_MIN`)."""
    return (n or stair_steps(z0, z1)) * TREAD_MIN


def stair_well(a, b, z0, z1, n, under):
    """Hvor trapphullet må begynne (langs retningen, fra foten `a` mot toppen
    `b`) når `under` er undersida av bjelkelaget over trappa: fra og med det
    første trinnet som ikke har `HEADROOM` under bjelkelaget, pluss
    kapselens radius. Hullet går så videre til toppen."""
    rise, tread = (z1 - z0) / n, abs(b - a) / n
    d = 1 if b > a else -1
    for k in range(n):
        if under - (z0 + (k + 1) * rise) < HEADROOM:
            return a + d * max(0.0, k * tread - STAIR_MARGIN)
    return b


def stairs(p, x0, x1, a, b, z0, z1, n=None, along="y", rails=(), mat=None, room=None, under=None, well=None):
    """En rett trapp mellom x0 og x1 (på tvers) fra `a` (nederst) til `b`
    (øverst) langs `along`, fra z0 til z1, med n trinn (ellers så få som
    reglene tillater). Kollideren er en rampe over trinnene.

    Reglene sjekkes og feiler høyt (flytt heller trappa i huset):
    - trinnhøyde ≤ `RISE_MAX`, inntrinn ≥ `TREAD_MIN`;
    - `room` = (gulvet ved foten, gulvet ved toppen): hvor langt det er
      fritt langs retningen før en vegg eller noe annet, forbi foten og
      forbi toppen (minst `LANDING`);
    - `under` = undersida av bjelkelaget over, og `well` = (fra, til)
      trapphullet langs retningen: hvert trinn som ikke ligger helt i
      hullet (med kapselens radius) har minst `HEADROOM` opp til `under`.
      `stair_well` regner ut hvor hullet må begynne."""
    mat = mat or p.floor
    n = n or stair_steps(z0, z1)
    step = (b - a) / n
    rise = (z1 - z0) / n
    where = f"trappa {along}={min(a, b):.2f}..{max(a, b):.2f}, z={z0:.2f}..{z1:.2f}, {n} trinn"
    assert rise <= RISE_MAX + 1e-6, f"{where}: trinnhøyden {rise:.3f} m er over {RISE_MAX} m; bruk minst {stair_steps(z0, z1)} trinn"
    assert abs(step) >= TREAD_MIN - 1e-6, f"{where}: inntrinnet {abs(step):.3f} m er under {TREAD_MIN} m; trappa må være minst {stair_run(z0, z1, n):.2f} m lang"
    d = 1 if b > a else -1
    if room is not None:
        lo_free, hi_free = (room[0] - a) * -d, (room[1] - b) * d
        assert lo_free >= LANDING - 1e-6, f"{where}: bare {lo_free:.2f} m fritt gulv foran nederste trinn (minst {LANDING} m)"
        assert hi_free >= LANDING - 1e-6, f"{where}: bare {hi_free:.2f} m fritt gulv foran øverste trinn (minst {LANDING} m)"
    if under is not None:
        w0, w1 = (min(well), max(well)) if well else (None, None)
        for k in range(n):
            s0, s1 = a + k * step, a + (k + 1) * step
            # Kapselens radius til hver side, men ikke forbi foten eller
            # toppen (der står du på gulvet, lavere enn på trinnet).
            lo, hi = max(min(s0, s1) - STAIR_MARGIN, min(a, b)), min(max(s0, s1) + STAIR_MARGIN, max(a, b))
            in_well = well is not None and w0 <= lo and hi <= w1
            head = under - (z0 + (k + 1) * rise)
            assert in_well or head >= HEADROOM - 1e-6, f"{where}: trinn {k + 1} har bare {head:.2f} m hoderom under bjelkelaget ({under:.2f}); trapphullet må begynne ved {stair_well(a, b, z0, z1, n, under):.2f} (nå {well})"
    for k in range(n):
        s0, s1 = a + k * step, a + (k + 1) * step
        z = z0 + (k + 1) * rise
        lo = (min(x0, x1), min(s0, s1), z - 0.07) if along == "y" else (min(s0, s1), min(x0, x1), z - 0.07)
        hi = (max(x0, x1), max(s0, s1), z) if along == "y" else (max(s0, s1), max(x0, x1), z)
        box("step", lo, hi, mat, bevel=0.01)
    # Vangene ligger under trinnesene og ender under gulvet oppe, så de
    # ikke stikker opp gjennom det.
    for xx in (x0, x1):
        if along == "y":
            stick("stringer", (xx, a - step * 0.3, z0 + rise * 0.7 - 0.17), (xx, b, z1 - 0.17), 0.07, 0.28, p.timber, ("timber", 1))
        else:
            stick("stringer", (a - step * 0.3, xx, z0 + rise * 0.7 - 0.17), (b, xx, z1 - 0.17), 0.07, 0.28, p.timber, ("timber", 1))
    for xx in rails:
        if along == "y":
            stick("handrail", (xx, a, z0 + 0.95), (xx, b, z1 + 0.95), 0.06, 0.07, p.timber, ("timber", 2))
            for k in range(0, n + 1, 3):
                s = a + k * step
                stick("baluster", (xx, s, z0 + k * rise), (xx, s, z0 + k * rise + 0.95), 0.05, 0.05, p.timber, ("timber", 0))
        else:
            stick("handrail", (a, xx, z0 + 0.95), (b, xx, z1 + 0.95), 0.06, 0.07, p.timber, ("timber", 2))
            for k in range(0, n + 1, 3):
                s = a + k * step
                stick("baluster", (s, xx, z0 + k * rise), (s, xx, z0 + k * rise + 0.95), 0.05, 0.05, p.timber, ("timber", 0))
    # Rampen: fra a (litt foran) til b.
    lo_x, hi_x = min(x0, x1), max(x0, x1)
    pad = step * 0.6
    if along == "y":
        town.HULLS.append([(x, a - pad, z0 - 0.3) for x in (lo_x, hi_x)] + [(x, a - pad, z0) for x in (lo_x, hi_x)] + [(x, b, z1 + 0.02) for x in (lo_x, hi_x)] + [(x, b, z0 - 0.3) for x in (lo_x, hi_x)])
    else:
        town.HULLS.append([(a - pad, y, z0 - 0.3) for y in (lo_x, hi_x)] + [(a - pad, y, z0) for y in (lo_x, hi_x)] + [(b, y, z1 + 0.02) for y in (lo_x, hi_x)] + [(b, y, z0 - 0.3) for y in (lo_x, hi_x)])


def floor_frame(p, name, lo, hi, well, z0, z1, collide_it=True):
    """Et etasjeskille fra (x, y) `lo` til `hi` mellom z0 og z1 som rammer
    rundt trapphullet `well` = (x0, y0, x1, y1): fire plater som møtes
    kant i kant, så ingen flater deler plan. Kolliderne følger platene."""
    wx0, wy0, wx1, wy1 = well
    pieces = [((lo[0], lo[1]), (wx0, hi[1])), ((wx1, lo[1]), (hi[0], hi[1])), ((wx0, lo[1]), (wx1, wy0)), ((wx0, wy1), (wx1, hi[1]))]
    for (ax, ay), (bx, by) in pieces:
        if bx - ax < 1e-3 or by - ay < 1e-3:
            continue
        planks(p, name, (ax, ay, z0), (bx, by, z1))
        if collide_it:
            collide(((ax + bx) / 2, (ay + by) / 2, (z0 + z1) / 2), (bx - ax, by - ay, z1 - z0))


def well_rail(p, segments, z, h=0.95, collide_it=True):
    """Gelenderet rundt trapphullet oppe: stolper og håndlist langs hvert
    (a, b)-stykke i gulvhøyde `z`, med kollider."""
    for (ax, ay), (bx, by) in segments:
        stick("srail", (ax, ay, z + h), (bx, by, z + h), 0.06, 0.07, p.timber, ("timber", 2))
        L = math.hypot(bx - ax, by - ay)
        for k in range(max(2, round(L / 1.2) + 1)):
            t = k / max(1, round(L / 1.2))
            px, py = ax + (bx - ax) * t, ay + (by - ay) * t
            stick("srail_post", (px, py, z), (px, py, z + h + 0.05), 0.09, 0.09, p.timber, ("timber", 0))
        if collide_it:
            collide(((ax + bx) / 2, (ay + by) / 2, z + (h + 0.05) / 2), (abs(bx - ax) + 0.12, abs(by - ay) + 0.12, h + 0.05))


def table(p, x, y, z, w, d, h=0.78, yaw=0.0, collide_it=True):
    """Bord med plate av planker og fire ben."""
    c = Vector((x, y, z))
    top = box("table_top", (-w / 2, -d / 2, h - 0.07), (w / 2, d / 2, h), p.oak, bevel=0.02)
    legs = []
    for sx in (-1, 1):
        for sy in (-1, 1):
            legs.append(box("table_leg", (sx * (w / 2 - 0.12) - 0.05, sy * (d / 2 - 0.1) - 0.05, 0), (sx * (w / 2 - 0.12) + 0.05, sy * (d / 2 - 0.1) + 0.05, h - 0.07), p.timber, bevel=0.012))
    for ob in [top] + legs:
        _place(ob, c, yaw)
    if collide_it:
        collide((x, y, z + h / 2), (w, d, h), yaw)


def bench(p, x, y, z, w, yaw=0.0, collide_it=True):
    top = box("bench_top", (-w / 2, -0.17, 0.4), (w / 2, 0.17, 0.46), p.oak, bevel=0.015)
    legs = [box("bench_leg", (sx * (w / 2 - 0.15) - 0.05, -0.14, 0), (sx * (w / 2 - 0.15) + 0.05, 0.14, 0.4), p.timber, bevel=0.01) for sx in (-1, 1)]
    for ob in [top] + legs:
        _place(ob, Vector((x, y, z)), yaw)
    if collide_it:
        collide((x, y, z + 0.23), (w, 0.34, 0.46), yaw)


def stool(p, x, y, z):
    box("stool_top", (x - 0.18, y - 0.18, z + 0.42), (x + 0.18, y + 0.18, z + 0.48), p.oak, bevel=0.015)
    for sx in (-1, 1):
        for sy in (-1, 1):
            stick("stool_leg", (x + sx * 0.12, y + sy * 0.12, z), (x + sx * 0.1, y + sy * 0.1, z + 0.43), 0.045, 0.045, p.timber, ("timber", 0))


def barrel(p, x, y, z, r=0.32, h=0.85, lying=False, yaw=0.0):
    """Tønne: åttekantet stav-kropp som buker ut, og to jernbånd."""
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    n, rings = 10, 5
    verts = []
    for j in range(rings + 1):
        t = j / rings
        rr = r * (0.86 + 0.14 * math.sin(math.pi * t))
        row = [bm.verts.new((rr * math.cos(2 * math.pi * i / n), rr * math.sin(2 * math.pi * i / n), h * t)) for i in range(n)]
        verts.append(row)
    for j in range(rings):
        for i in range(n):
            a, b = verts[j][i], verts[j][(i + 1) % n]
            c, d = verts[j + 1][(i + 1) % n], verts[j + 1][i]
            f = bm.faces.new((a, b, c, d))
            for lp, (uu, vv) in zip(f.loops, ((i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1))):
                lp[uvl].uv = (uu * 2 * math.pi * r / n, vv * h / rings)
    for row, z in ((verts[0], 0), (verts[-1], h)):
        f = bm.faces.new(row if z > 0 else row[::-1])
        for lp in f.loops:
            lp[uvl].uv = (lp.vert.co.x, lp.vert.co.y)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = _object("barrel", bm, p.cladding)
    hoops = []
    for zz in (0.18, h - 0.18):
        bpy.ops.mesh.primitive_torus_add(major_radius=r * (0.86 + 0.14 * math.sin(math.pi * zz / h)) + 0.005, minor_radius=0.018, major_segments=n, minor_segments=3, location=(0, 0, zz))
        hp = bpy.context.active_object
        hp.data.materials.append(p.iron)
        town.uv_box(hp)
        PIECES.append(hp)
        hoops.append(hp)
    for o in [ob] + hoops:
        if lying:
            o.location = o.location + Vector((0, 0, -h / 2))
            o.matrix_world = Matrix.Translation((x, y, z + r)) @ Matrix.Rotation(yaw, 4, "Z") @ Matrix.Rotation(math.pi / 2, 4, "Y") @ o.matrix_world
        else:
            o.matrix_world = Matrix.Translation((x, y, z)) @ Matrix.Rotation(yaw, 4, "Z") @ o.matrix_world
    return ob


def _place(ob, c, yaw):
    ob.matrix_world = Matrix.Translation(c) @ Matrix.Rotation(yaw, 4, "Z") @ ob.matrix_world


def fire(p, x, y, z, size=0.5, light_it=True, intensity=3.0, radius=8.0):
    """Ild: kubber i kryss og flammer (lyser hele døgnet), og et lys."""
    for a in (0.0, 1.2, 2.3):
        d = Vector((math.cos(a), math.sin(a), 0)) * size * 0.5
        stick("log", (x - d.x, y - d.y, z + 0.06), (x + d.x, y + d.y, z + 0.1), 0.1, 0.1, layer("logs"), bevel=0.02)
    for k, (dx, dy, hh) in enumerate(((0, 0, 0.45), (0.1, 0.05, 0.3), (-0.1, -0.04, 0.32))):
        bm = bmesh.new()
        bmesh.ops.create_cone(bm, cap_ends=False, segments=5, radius1=size * 0.22, radius2=0.0, depth=hh * size * 2)
        for v in bm.verts:
            v.co += Vector((x + dx * size, y + dy * size, z + 0.08 + hh * size))
        _object("flame", bm, p.fire)
    if light_it:
        town.light((x, y, z + 0.5), (1.0, 0.55, 0.25), intensity, radius)


def candle(p, x, y, z, light_it=False):
    box("candle", (x - 0.025, y - 0.025, z), (x + 0.025, y + 0.025, z + 0.12), layer("plaster_white"), bevel=0.008)
    box("candle_flame", (x - 0.012, y - 0.012, z + 0.12), (x + 0.012, y + 0.012, z + 0.17), p.fire, bevel=0.005)
    if light_it:
        town.light((x, y, z + 0.25), (1.0, 0.65, 0.35), 0.8, 4.0)
