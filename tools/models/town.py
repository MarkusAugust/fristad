"""Felles hjelpere for modellene i byen (brukes av `kit.py`, `houses.py`,
`tavern.py`, `workshops.py`, `sacred.py`, `fortress.py`, `streetlife.py`,
`street.py`, `home.py`, `yard.py` og `portal.py`): materialene (`layer`,
`plain`), deler med uv i meter (`block`, `beam`, `prism`, `uv_box`),
registrene som `finish` skriver til `<navn>_inside.json` (`collide`, `ramp`,
`light`, `lamp`, `prop`, `wall_lantern`, røyken), vindusruter med dagslys
(`daylight`), og `finish` (slår sammen, deler lange kanter, baker
okklusjonen i hjørnefargen og eksporterer).

Akser i Blender: X til høyre, −Y fram (gata), Z opp. Origo midt på
grunnflaten i bakkehøyde. I motoren blir −Y til +Z."""

import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import slim  # noqa: E402
import bmesh  # noqa: E402
from common import clear  # noqa: E402
from mathutils import Euler, Matrix, Vector  # noqa: E402

OUT = "assets/models/town"


# --- Materialer ------------------------------------------------------------


def layer(name, tint=(1.0, 1.0, 1.0), rough=0.5):
    """Et teksturlag fra motoren: `layer_<lag>` med fargen som tone."""
    key = f"layer_{name}"
    suffix = 0
    # Samme lag med ulik tone får hvert sitt materiale (Blender legger på .001).
    while True:
        n = key if suffix == 0 else f"{key}.{suffix:03d}"
        m = bpy.data.materials.get(n)
        if m is None:
            m = bpy.data.materials.new(n)
            m.use_nodes = True
            b = m.node_tree.nodes["Principled BSDF"]
            b.inputs["Base Color"].default_value = (*tint, 1.0)
            b.inputs["Roughness"].default_value = rough
            return m
        b = m.node_tree.nodes["Principled BSDF"]
        if tuple(round(v, 3) for v in b.inputs["Base Color"].default_value[:3]) == tuple(round(v, 3) for v in tint):
            return m
        suffix += 1


def plain(name, rgb, rough=0.5):
    m = bpy.data.materials.get(name)
    if m:
        return m
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Base Color"].default_value = (*rgb, 1.0)
    b.inputs["Roughness"].default_value = rough
    return m


class M:
    """Materialene til et hus."""

    def __init__(self, plaster_tint=(0.42, 0.4, 0.37), timber_tint=(1.0, 1.0, 1.0), roof="tiles"):
        self.stone = layer("stone", (0.5, 0.48, 0.45), 0.5)
        self.plaster = layer("plaster", plaster_tint, 0.5)
        self.timber = layer("timber", timber_tint, 0.5)
        self.boards = layer("cladding", (0.6, 0.57, 0.53), 0.5)
        self.roof = layer(roof, (0.62, 0.62, 0.6) if roof == "slate" else (1.0, 1.0, 1.0), 0.5)
        self.iron = layer("iron", (0.55, 0.5, 0.48), 0.5)
        self.glass = plain("glass", (0.05, 0.06, 0.06), 0.05)


# --- Geometri --------------------------------------------------------------

PIECES = []
# Hus man kan gå inn i: kollidere (midte, mål, yaw om z) og lys (posisjon,
# farge, styrke, radius) i Blenders akser; `finish` skriver dem til
# `<navn>_inside.json` i motorens akser, og byen bruker dem i stedet for
# en boks rundt huset.
COLLIDERS = []
LIGHTS = []
HULLS = []


def collide(c, size, yaw=0.0):
    COLLIDERS.append((tuple(c), tuple(size), yaw))


def ramp(x0, x1, y_low, y_high, z_low, z_high):
    """En skrå rampe som kollider over trinn (spilleren går opp den), fra
    bakken i y_low til gulvet i y_high, mellom x0 og x1."""
    HULLS.append([(x, y_low, z_low - 0.3) for x in (x0, x1)] + [(x, y_low, z_low) for x in (x0, x1)] + [(x, y_high, z_high) for x in (x0, x1)] + [(x, y_high, z_low - 0.3) for x in (x0, x1)])


def light(c, color=(1.0, 0.6, 0.3), intensity=1.0, radius=5.0):
    LIGHTS.append((tuple(c), tuple(color), intensity, radius))


# Lykter som bare lyser om natta (vegglykta ved døra); `finish` skriver dem
# som «lamps».
LAMPS = []

# Hus man kan gå inn i: lyset inne skinner ut gjennom rutene om natta
# (`daylight`, satt i `buildings.py`).
LIT = False

# Pipene som ryker: toppen og tykkelsen (0..1); `finish` skriver dem som
# «smoke».
SMOKE = []


def lamp(c, color=(1.0, 0.68, 0.38), intensity=1.6, radius=8.0):
    LAMPS.append((tuple(c), tuple(color), intensity, radius))


# Småting ute (tønner, kasser, vedstabler) som hører til huset, men står på
# bakken, ikke på modellens z = 0: `finish` skriver dem som «props», og byen
# setter hver av dem på terrenget med `Builder::place` (så `audit` gjelder).
# (modellnavn i `models!`, (x, y) i husets akser, yaw om z)
PROPS = []


def prop(name, x, y, yaw=0.0):
    PROPS.append((name, (x, y), yaw))


# --- Møblene som egne modeller ----------------------------------------------
#
# `with piece("bench", (x, y, z), yaw):` rundt koden som lager et møbel: det
# som lages i blokka (deler, kollidere, lys), tas ut av huset. Første gang et
# navn brukes, blir det modellen `assets/models/furniture/<navn>.glb` (med
# kolliderne og lysene i `<navn>_inside.json`, i møbelets eget rom, origo på
# gulvet midt under), og hver gang skrives stedet til
# `assets/layout/inside/<hus>.json` (`write_contents`). Samme navn må gi samme
# møbel; et møbel med en annen farge får et eget navn.

FURNITURE = "assets/models/furniture"
PLACED = []      # (navn, (x, y, z), yaw, targets) i husets akser
# Veggene (fra `kit.slab_grid`): (ramme, lengde, z0, z1, tykkelse, hull).
# Hvert møbel sjekkes mot dem (`check_piece`).
WALLS = []
_WALL_MARK = 0


def check_points(name, pts, gap=0.4):
    """Punktene (verden) går ikke inn i en vegg og står ikke innen `gap`
    foran et vindu (se `check_piece`)."""
    _check(name, pts, gap)


def check_piece(name, made):
    """Et møbel går ikke inn i en vegg (mer enn 3 cm) og dekker ikke et
    vindu (står ikke innen 40 cm foran en åpning som ikke går fra gulvet);
    ellers stopper eksporten (eierens dom i runde 7: tønner i veggene,
    hyller over vinduet)."""
    _check(name, [o.matrix_world @ v.co for o in made if o.type == "MESH" for v in o.data.vertices], 0.4)


def _check(name, pts, gap):
    for (px, py), (dx, dy), L, z0, z1, thick, holes in WALLS:
        nx, ny = -dy, dx
        for p in pts:
            s = (p.x - px) * dx + (p.y - py) * dy
            t = (p.x - px) * nx + (p.y - py) * ny
            if not (0.02 < s < L - 0.02 and z0 + 0.02 < p.z < z1 - 0.02):
                continue
            hole = next((h for h in holes if h[0] - 0.02 < s < h[1] + 0.02 and h[2] - 0.02 < p.z < h[3] + 0.02), None)
            if hole is None:
                if abs(t) < thick / 2 - 0.03:
                    raise SystemExit(f"møbelet «{name}» går {thick / 2 - abs(t):.2f} m inn i veggen ved ({p.x:.2f}, {p.y:.2f}, {p.z:.2f})")
            elif hole[2] > z0 + 0.3 and abs(t) < thick / 2 + gap and hole[0] + 0.03 < s < hole[1] - 0.03 and hole[2] + 0.03 < p.z < hole[3] - 0.03:
                raise SystemExit(f"møbelet «{name}» dekker vinduet ved ({p.x:.2f}, {p.y:.2f}, {p.z:.2f})")
_PIECE_DEFS = {}  # navn -> (blend-fil, trekanter, kollidere, lys, hull)


class piece:
    def __init__(self, name, at, yaw=0.0, targets=()):
        self.name, self.at, self.yaw, self.targets = name, tuple(at), yaw, list(targets)

    def __enter__(self):
        self.before = set(bpy.data.objects)
        self.n = (len(COLLIDERS), len(LIGHTS), len(HULLS))
        return self

    def __exit__(self, *exc):
        if exc[0]:
            return False
        bpy.context.view_layer.update()  # deler med location/rotation satt
        made = [o for o in bpy.data.objects if o not in self.before]
        check_piece(self.name, made)
        cols, lights, hulls = COLLIDERS[self.n[0]:], LIGHTS[self.n[1]:], HULLS[self.n[2]:]
        del COLLIDERS[self.n[0]:], LIGHTS[self.n[1]:], HULLS[self.n[2]:]
        for o in made:
            if o in PIECES:
                PIECES.remove(o)
        # Til møbelets eget rom: flytt og drei tilbake.
        c, s = math.cos(-self.yaw), math.sin(-self.yaw)
        ox, oy, oz = self.at
        local = lambda v: ((v[0] - ox) * c - (v[1] - oy) * s, (v[0] - ox) * s + (v[1] - oy) * c, v[2] - oz)
        tris = sum(len(p.vertices) - 2 for o in made if o.type == "MESH" for p in o.data.polygons)
        if self.name not in _PIECE_DEFS:
            os.makedirs("target/pieces", exist_ok=True)
            blend = f"target/pieces/{self.name}.blend"
            to = Matrix.Rotation(-self.yaw, 4, "Z") @ Matrix.Translation((-ox, -oy, -oz))
            for o in made:
                if o.parent is None:
                    o.matrix_world = to @ o.matrix_world
            bpy.data.libraries.write(blend, set(made), fake_user=True)
            _PIECE_DEFS[self.name] = (blend, tris, [(local(cc), sz, y - self.yaw) for cc, sz, y in cols], [(local(cc), col, i, r) for cc, col, i, r in lights], [[local(q) for q in pts] for pts in hulls])
        elif _PIECE_DEFS[self.name][1] != tris:
            raise RuntimeError(f"møbelet «{self.name}» har {tris} trekanter her, men {_PIECE_DEFS[self.name][1]} første gang: gi det et eget navn")
        for o in made:
            bpy.data.objects.remove(o, do_unlink=True)
        PLACED.append((self.name, self.at, self.yaw, self.targets))
        return False


def write_contents(house):
    """Skriv stedene til møblene i `house` til `assets/layout/inside/<hus>.json`
    (motorens akser, husets rom), om fila ikke finnes: etter første gang er
    fila sannheten, og redigeringsmodusen endrer den. Så eksporteres møblene
    som er nye (`FRISTAD_PIECES=alle` eksporterer alle på nytt)."""
    import json
    path = f"assets/layout/inside/{house}.json"
    if not os.path.exists(path) or os.environ.get("FRISTAD_CONTENTS") == "skriv":
        items = []
        for name, (x, y, z), yaw, targets in PLACED:
            h = yaw / 2
            items.append({"at": {"pos": [round(x, 5), round(z, 5), round(-y, 5)], "rot": [0, round(math.sin(h), 5), 0, round(math.cos(h), 5)]}, "model": name, "mounted": True, **({"targets": targets} if targets else {})})
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as f:
            f.write("[\n" + ",\n".join("  " + json.dumps(i, separators=(",", ":"), ensure_ascii=False) for i in items) + "\n]\n")
        print(f"{path}: {len(items)} møbler (kjør `cargo run --bin layout` for fast form)")
    else:
        print(f"{path} finnes og skrives ikke over")
    PLACED.clear()
    export_pieces()


def export_pieces():
    """Hvert møbel i sin egen glb, med kolliderne og lysene sine."""
    defs = dict(_PIECE_DEFS)
    _PIECE_DEFS.clear()
    for name, (blend, _, cols, lights, hulls) in defs.items():
        out = f"{FURNITURE}/{name}.glb"
        if os.path.exists(out) and os.environ.get("FRISTAD_PIECES") != "alle":
            continue
        clear()
        PIECES.clear()
        with bpy.data.libraries.load(blend) as (src, dst):
            dst.objects = list(src.objects)
        for o in dst.objects:
            bpy.context.scene.collection.objects.link(o)
        COLLIDERS.extend(cols)
        LIGHTS.extend(lights)
        HULLS.extend(hulls)
        finish(out, max_edge=100.0)


def wall_lantern(m, x, y_face, z, normal_y):
    """Ei vegglykt ved døra: smijernsarm med stag, og en lykt med hornruter
    og et talglys som henger i en krok. `z` er høyden på armen."""
    s = normal_y
    iron = m.iron
    horn = plain("glow_horn", (0.3, 0.19, 0.08), 0.5)
    flame = plain("glow_flame", (1.0, 0.7, 0.35), 0.4)
    wax = layer("plaster", (0.75, 0.7, 0.55), 0.5)
    block("lantern_plate", (0.12, 0.03, 0.34), iron, at=(x, y_face + s * 0.015, z - 0.08), bevel=0.006)
    beam("lantern_arm", (x, y_face + s * 0.02, z), (x, y_face + s * 0.46, z), 0.03, 0.03, iron)
    beam("lantern_strut", (x, y_face + s * 0.02, z - 0.22), (x, y_face + s * 0.3, z - 0.01), 0.022, 0.022, iron)
    # Snirkelen ytterst på armen og kroken.
    beam("lantern_curl", (x, y_face + s * 0.46, z), (x, y_face + s * 0.5, z + 0.05), 0.022, 0.022, iron)
    ly = y_face + s * 0.4
    beam("lantern_hook", (x, ly, z), (x, ly, z - 0.1), 0.012, 0.012, iron)
    top = z - 0.1
    revolve_ring = bpy.ops.mesh.primitive_torus_add
    revolve_ring(major_radius=0.025, minor_radius=0.006, major_segments=10, minor_segments=4, location=(x, ly, top - 0.02), rotation=(math.pi / 2, 0, 0))
    ring = bpy.context.active_object
    ring.data.materials.append(iron)
    ensure_uv(ring.data)
    uv_box(ring)
    PIECES.append(ring)
    # Lykta: tak som en lav pyramide, ramme, hornruter og bunn.
    h = 0.26
    c = Vector((x, ly, top - 0.06 - h / 2))
    prism("lantern_roof", [(-0.12, 0.0), (0.12, 0.0), (0.0, 0.07)], 0.24, iron, at=(c.x, c.y, top - 0.08))
    block("lantern_top", (0.2, 0.2, 0.025), iron, at=c + Vector((0, 0, h / 2)), bevel=0.004)
    block("lantern_base", (0.2, 0.2, 0.03), iron, at=c - Vector((0, 0, h / 2)), bevel=0.004)
    for sx in (-1, 1):
        for sy in (-1, 1):
            block("lantern_post", (0.016, 0.016, h), iron, at=c + Vector((sx * 0.092, sy * 0.092, 0)), bevel=0.002)
    for k in range(4):
        a = k * math.pi / 2
        d = Vector((math.cos(a), math.sin(a), 0)) * 0.088
        block("lantern_horn", (0.004, 0.17, h - 0.03) if k % 2 == 0 else (0.17, 0.004, h - 0.03), horn, at=c + d, bevel=0)
        block("lantern_bar", (0.012, 0.18, 0.012) if k % 2 == 0 else (0.18, 0.012, 0.012), iron, at=c + d * 1.05, bevel=0.002)
    block("lantern_candle", (0.04, 0.04, 0.09), wax, at=c - Vector((0, 0, h / 2 - 0.06)), bevel=0.01)
    block("lantern_flame", (0.018, 0.018, 0.045), flame, at=c - Vector((0, 0, h / 2 - 0.13)), bevel=0.006)
    lamp((c.x, c.y, c.z))


def ensure_uv(me):
    """Alle delene skal ha uv-laget «UVMap»: når delene slås sammen, holdes
    lag med ulike navn adskilt, og eksporten tar bare det første."""
    if "UVMap" not in me.uv_layers:
        for layer in list(me.uv_layers):
            me.uv_layers.remove(layer)
        me.uv_layers.new(name="UVMap")
    me.uv_layers.active = me.uv_layers["UVMap"]


def uv_box(ob, along=None):
    """Uv i meter fra de lokale koordinatene, etter flatens hovedakse: vegger
    mot ±Y får (x, z), vegger mot ±X (y, z), flater opp og ned (x, y).
    `along` = "x" legger u langs lokal x på alle flater (bjelker), så årene
    følger lengden."""
    me = ob.data
    ensure_uv(me)
    bm = bmesh.new()
    bm.from_mesh(me)
    uvl = bm.loops.layers.uv.verify()
    for f in bm.faces:
        n = f.normal
        ax = max(range(3), key=lambda i: abs(n[i]))
        for loop in f.loops:
            p = loop.vert.co
            if along == "x":
                if ax == 0:
                    uv = (p.y, p.z)
                elif ax == 1:
                    uv = (p.x, p.z)
                else:
                    uv = (p.x, p.y)
            else:
                uv = [(p.y, p.z), (p.x, p.z), (p.x, p.y)][ax]
            loop[uvl].uv = uv
    bm.to_mesh(me)
    bm.free()


def block(name, size, mat, at=(0, 0, 0), rot=(0, 0, 0), bevel=0.012, along=None, segments=1):
    """En avfaset kloss: bygd sentrert i origo, uv i meter, så dreid og flyttet."""
    bpy.ops.mesh.primitive_cube_add(size=1.0)
    ob = bpy.context.active_object
    ob.name = name
    ob.scale = size
    bpy.ops.object.transform_apply(scale=True)
    if bevel > 0:
        mod = ob.modifiers.new("bevel", "BEVEL")
        mod.width = min(bevel, min(size) * 0.45)
        mod.segments = segments
        mod.limit_method = "NONE"
        bpy.ops.object.modifier_apply(modifier=mod.name)
    ob.data.materials.append(mat)
    uv_box(ob, along)
    ob.rotation_euler = Euler(rot)
    ob.location = at
    PIECES.append(ob)
    return ob


def beam(name, a, b, w, h, mat, roll=0.0):
    """En bjelke fra punkt a til b, w bred og h høy, med årene langs."""
    a, b = Vector(a), Vector(b)
    d = b - a
    ob = block(name, (d.length, w, h), mat, along="x", bevel=0.015)
    # Lokal x mot d.
    q = Vector((1, 0, 0)).rotation_difference(d.normalized())
    ob.rotation_mode = "QUATERNION"
    ob.rotation_quaternion = q @ Euler((roll, 0, 0)).to_quaternion()
    ob.location = (a + b) / 2
    return ob


def prism(name, pts2d, depth, mat, at, rot=(0, 0, 0)):
    """Et prisme fra et polygon i xz-planet, `depth` tykt langs y (gavler)."""
    bm = bmesh.new()
    front = [bm.verts.new((x, -depth / 2, z)) for x, z in pts2d]
    back = [bm.verts.new((x, depth / 2, z)) for x, z in pts2d]
    bm.faces.new(front[::-1])
    bm.faces.new(back)
    n = len(pts2d)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((front[i], front[j], back[j], back[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    uv_box(ob)
    ob.rotation_euler = Euler(rot)
    ob.location = at
    PIECES.append(ob)
    return ob


def daylight(ob, inward):
    """Flatene på ruta som vender inn i huset får «glass_day»: innenfra
    skinner dagslyset gjennom, utenfra speiler ruta som før. I hus man kan
    gå inn i (`LIT`) får utsida «glass_lit», som lyser varmt om natta."""
    mat = plain("glass_day", (0.05, 0.06, 0.06), 0.3)
    ob.data.materials.append(mat)
    idx = len(ob.data.materials) - 1
    lit = None
    if LIT:
        ob.data.materials.append(plain("glass_lit", (0.05, 0.06, 0.06), 0.1))
        lit = len(ob.data.materials) - 1
    bpy.context.view_layer.update()
    rot = ob.matrix_world.to_3x3()
    inward = Vector(inward).normalized()
    for p in ob.data.polygons:
        d = (rot @ p.normal).normalized().dot(inward)
        if d > 0.5:
            p.material_index = idx
        elif d < -0.5 and lit is not None:
            p.material_index = lit
    return ob


def smooth(ob):
    """Merk en del som rund: `finish` gir den myke normaler (alt annet er
    flatt)."""
    a = ob.data.attributes.new("keep_smooth", "BOOLEAN", "FACE")
    for v in a.data:
        v.value = True
    ob.data.shade_smooth()
    return ob


def finish(path, ao_samples=160, max_edge=0.6):
    """Slå sammen delene, del store flater så okklusjonen får hjørner å
    bo i, bak AO til hjørnefargen, og eksporter."""
    if os.environ.get("FRISTAD_JSON_ONLY"):
        # Bare kolliderne, lysene og røyken (modellen er uendret).
        write_inside(path)
        return
    bpy.ops.object.select_all(action="DESELECT")
    meshes = [o for o in bpy.data.objects if o.type == "MESH"]
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    ob = bpy.context.active_object
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    # Store flater deles i biter på ca. 0,5 m.
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    # Først trekanter (hullene fra boolean gir mangekanter), så deles de
    # lange kantene; trekanter deles rent.
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    for _ in range(4):
        long = [e for e in bm.edges if e.calc_length() > max_edge]
        if not long:
            break
        bmesh.ops.subdivide_edges(bm, edges=long, cuts=1, use_grid_fill=False)
        bmesh.ops.triangulate(bm, faces=[f for f in bm.faces if len(f.verts) > 3])
    bm.to_mesh(ob.data)
    bm.free()
    # Flatt overalt, unntatt deler merket med `smooth()` (runde former som
    # høystakken), som beholder myke normaler.
    keep = ob.data.attributes.get("keep_smooth")
    for p in ob.data.polygons:
        p.use_smooth = bool(keep and keep.data[p.index].value)
    # Okklusjonen til hjørnefargen.
    attr = ob.data.color_attributes.new("ao", "BYTE_COLOR", "POINT")
    ob.data.color_attributes.active_color = attr
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = ao_samples
    scene.render.bake.target = "VERTEX_COLORS"
    world = bpy.data.worlds.new("w") if not scene.world else scene.world
    scene.world = world
    # Bare nærskygge: fugene og hjørnene, ikke hele huset over flatene
    # (lang rekkevidde ga store lyse og mørke trekanter i pussen).
    world.light_settings.distance = 0.6
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.object.bake(type="AO")
    # Mykere: 0,35..1 i stedet for 0..1, så fugene er mørke uten å bli svarte.
    import numpy as np
    col = np.empty(len(attr.data) * 4, dtype=np.float32)
    attr.data.foreach_get("color", col)
    col = col.reshape(-1, 4)
    # Støyen fra Cycles jevnes ut over naboene (tre runder), så flatene ikke
    # blir flekkete trekanter.
    edges = np.empty(len(ob.data.edges) * 2, dtype=np.int64)
    ob.data.edges.foreach_get("vertices", edges)
    edges = edges.reshape(-1, 2)
    ao = col[:, 0].copy()
    for _ in range(3):
        acc = ao.copy()
        cnt = np.ones_like(ao)
        np.add.at(acc, edges[:, 0], ao[edges[:, 1]])
        np.add.at(acc, edges[:, 1], ao[edges[:, 0]])
        np.add.at(cnt, edges[:, 0], 1)
        np.add.at(cnt, edges[:, 1], 1)
        ao = acc / cnt
    # Bare de tydelige fugene og hjørnene mørkner; middels verdier på åpne
    # flater (som ga synlige trekanter) løftes til fullt lys.
    t = np.clip((ao - 0.3) / 0.55, 0.0, 1.0)
    col[:, 0] = col[:, 1] = col[:, 2] = 0.45 + 0.55 * t * t * (3.0 - 2.0 * t)
    if os.environ.get("FRISTAD_GRADIENT", "1") == "1" and ("/town/" in path or "/furniture/" in path):
        # Malt stil: mørkere nede ved bakken (skitt og skygge), lysere oppe.
        zs = np.empty(len(ob.data.vertices) * 3, dtype=np.float32)
        ob.data.vertices.foreach_get("co", zs)
        z = zs.reshape(-1, 3)[:, 2]
        g = 0.68 + 0.32 * np.clip((z + 0.2) / 3.2, 0.0, 1.0)
        col[:, 0] *= g
        col[:, 1] *= g
        col[:, 2] *= g
    attr.data.foreach_set("color", col.ravel())
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=True, export_vertex_color="ACTIVE", export_all_vertex_colors=False, export_animations=False, export_skins=False, export_cameras=False, export_lights=False)
    slim.slim(path)  # normalene og hjørnefargen pakkes (slim.py)
    tris = sum(len(p.vertices) - 2 for p in ob.data.polygons)
    print(f"{path}: {tris} trekanter, {os.path.getsize(path) // 1024} kB")
    write_inside(path)


def write_inside(path):
    """Skriv kolliderne, lysene, lyktene og pipene til `<navn>_inside.json`
    i motorens akser, og tøm registrene."""
    if COLLIDERS or LIGHTS or HULLS or LAMPS or SMOKE or PROPS:
        import json
        # Blender (x, y, z) er (x, z, −y) i motoren; yaw om z blir yaw om y.
        eng = lambda v: [round(v[0], 3), round(v[2], 3), round(-v[1], 3)]
        size = lambda v: [round(v[0], 3), round(v[2], 3), round(v[1], 3)]
        data = {
            "boxes": [[eng(c), size(sz), round(yaw, 4)] for c, sz, yaw in COLLIDERS],
            "lights": [[eng(c), list(col), i, r] for c, col, i, r in LIGHTS],
            "hulls": [[eng(p) for p in pts] for pts in HULLS],
            "lamps": [[eng(c), list(col), i, r] for c, col, i, r in LAMPS],
            "smoke": [[eng(c), s] for c, s in SMOKE],
            "props": [[name, eng((x, y, 0.0)), round(yaw, 4)] for name, (x, y), yaw in PROPS],
        }
        with open(path.replace(".glb", "_inside.json"), "w") as f:
            json.dump(data, f)
        print(f"  {len(COLLIDERS)} kollidere, {len(LIGHTS)} lys, {len(LAMPS)} lykter, {len(SMOKE)} piper, {len(PROPS)} småting ute")
    COLLIDERS.clear()
    # Veggene fra utsida gjelder for møblene i innsida som lages etterpå;
    # etter en innside, eller en ny utside, gjelder de ikke lenger.
    global _WALL_MARK
    if path.endswith("_interior.glb"):
        WALLS.clear()
    else:
        del WALLS[:_WALL_MARK]
    _WALL_MARK = len(WALLS)
    PROPS.clear()
    LAMPS.clear()
    SMOKE.clear()
    LIGHTS.clear()
    HULLS.clear()
