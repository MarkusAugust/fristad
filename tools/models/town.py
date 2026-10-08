"""Husene i kjøpstaden, modellert i Blender (bpy) og eksportert til
assets/models/town/. Kjør:

    python3 tools/models/town.py [house_a house_b ...]

Husene er bygd av ekte deler: steinsokkel, bindingsverk av enkeltbjelker
med skråstag, kalkpuss mellom, framskutt overetasje med bjelkeender,
vinduer med karm, glass og skodder, plankedør med jernbeslag, og bratt
spontak med vindskier, mønebord og steinpipe.

Materialene er teksturlagene i motoren (`layer_<lag>`), lagt på etter uv i
meter som regnes ut for hver del før den dreies, så årene følger bjelkene
og spon-radene følger takflaten. Okklusjonen bakes i hjørnefargen (Cycles
AO til vertex colors), så fugene og krokene blir mørke.

Akser i Blender: X til høyre, −Y fram (gata), Z opp. Origo midt på
grunnflaten i bakkehøyde. I motoren blir −Y til +Z, så døra vender mot
lokal +Z som byen forventer."""

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
        self.timber = layer("oak", timber_tint, 0.5)
        self.boards = layer("cladding", (0.6, 0.57, 0.53), 0.5)
        self.roof = layer(roof, (0.62, 0.62, 0.6) if roof == "slate" else (1.0, 1.0, 1.0), 0.5)
        self.iron = layer("metal", (0.55, 0.5, 0.48), 0.5)
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


def lamp(c, color=(1.0, 0.68, 0.38), intensity=1.6, radius=8.0):
    LAMPS.append((tuple(c), tuple(color), intensity, radius))


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


def keep_uv(target, names):
    """Booleanen tar med uv-lagene til kutteren; behold bare de målet hadde
    (ellers eksporteres kubens 0..1-uv i stedet for uv i meter)."""
    for layer in [l for l in target.data.uv_layers if l.name not in names]:
        target.data.uv_layers.remove(layer)


# Steiner som står ut av en mur (navnet på muren → delene), så hull som
# skjæres senere kan ta dem med.
RELIEF = {}


def clear_relief(target, size, at, margin=0.3):
    """Fjern steinene utenpå `target` som ligger i åpningen."""
    keep = []
    for ob in RELIEF.get(target.name, []):
        p = ob.location
        if all(abs(p[i] - at[i]) < size[i] / 2 + margin for i in range(3)):
            if ob in PIECES:
                PIECES.remove(ob)
            bpy.data.objects.remove(ob, do_unlink=True)
        else:
            keep.append(ob)
    if target.name in RELIEF:
        RELIEF[target.name] = keep


def cut(target, size, at, rot=(0, 0, 0)):
    """Skjær et hull (vindu, dør) gjennom en del, med boolean."""
    clear_relief(target, size, at)
    names = {l.name for l in target.data.uv_layers}
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=at, rotation=rot)
    c = bpy.context.active_object
    c.scale = size
    mod = target.modifiers.new("cut", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.object = c
    mod.solver = "EXACT"
    bpy.context.view_layer.objects.active = target
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(c)
    keep_uv(target, names)


def carve(target, cutters):
    """Skjær bort alt av `target` som ligger inne i `cutters` (én boolean
    mot en samling), og regn uv på nytt."""
    if not cutters:
        return
    coll = bpy.data.collections.new("cutters")
    bpy.context.scene.collection.children.link(coll)
    copies = []
    for c in cutters:
        d = c.copy()
        d.data = c.data
        coll.objects.link(d)
        copies.append(d)
    names = {l.name for l in target.data.uv_layers}
    mod = target.modifiers.new("carve", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.operand_type = "COLLECTION"
    mod.collection = coll
    mod.solver = "EXACT"
    bpy.context.view_layer.objects.active = target
    bpy.ops.object.modifier_apply(modifier=mod.name)
    for d in copies:
        bpy.data.objects.remove(d)
    bpy.data.collections.remove(coll)
    keep_uv(target, names)
    uv_box(target)


# --- Delene i et hus ------------------------------------------------------


# Hus man kan gå inn i: lyset inne skinner ut gjennom rutene om natta.
LIT = False


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


def window(m, x, z, y_face, w, h, normal_y, shutters=True, rng=None):
    """Et vindu i en vegg med yta i y = y_face (normalen ±y): karm, sprosser,
    glass innfelt, og skodder som står ut."""
    s = normal_y
    t = 0.07
    # Karmen: fire bjelker rundt hullet.
    beam("win_top", (x - w / 2 - t, y_face + s * 0.03, z + h / 2 + t / 2), (x + w / 2 + t, y_face + s * 0.03, z + h / 2 + t / 2), 0.1, t, m.timber)
    beam("win_bot", (x - w / 2 - t, y_face + s * 0.05, z - h / 2 - t / 2), (x + w / 2 + t, y_face + s * 0.05, z - h / 2 - t / 2), 0.14, t, m.timber)
    for sx in (-1, 1):
        beam("win_side", (x + sx * (w / 2 + t / 2), y_face + s * 0.03, z - h / 2), (x + sx * (w / 2 + t / 2), y_face + s * 0.03, z + h / 2), 0.1, t, m.timber)
    # Glasset innfelt, med blysprosser i rutemønster.
    daylight(block("glass", (w, 0.02, h), m.glass, at=(x, y_face - s * 0.12, z), bevel=0), (0, -s, 0))
    # Blysprossene på begge sider av glasset.
    for g in (0.11, 0.13):
        for k in range(1, 3):
            block("lead", (0.012, 0.012, h), m.iron, at=(x - w / 2 + w * k / 3, y_face - s * g, z), bevel=0)
        for k in range(1, 4):
            block("lead", (w, 0.012, 0.012), m.iron, at=(x, y_face - s * g, z - h / 2 + h * k / 4), bevel=0)
    if shutters:
        # Skoddene: to plankeflater som står litt ut fra veggen, slått opp.
        for sx in (-1, 1):
            ang = sx * (math.radians(100 + (rng.uniform(-8, 8) if rng else 0)))
            hinge = Vector((x + sx * (w / 2 + t), y_face + s * 0.05, z))
            sw = w / 2
            ob = block("shutter", (sw, 0.035, h + 0.04), m.boards, bevel=0.006)
            ob.rotation_euler = Euler((0, 0, ang * (1 if s < 0 else -1) * -1))
            off = Matrix.Rotation(ob.rotation_euler.z, 4, "Z") @ Vector((sx * sw / 2, 0, 0))
            ob.location = hinge + off.xyz
            for zz in (-h * 0.3, h * 0.3):
                hb = block("hinge", (sw * 0.9, 0.045, 0.035), m.iron, bevel=0.004)
                hb.rotation_euler = ob.rotation_euler
                hb.location = hinge + off.xyz + Vector((0, 0, zz))


def door(m, x, y_face, normal_y, w=1.0, h=2.0, z0=0.0, open=False):
    """Plankedør med jernbeslag og ring, i karm, med steintrinn foran.
    `open`: døra står slått innover, hengslet på venstre side."""
    s = normal_y
    t = 0.12
    for sx in (-1, 1):
        beam("door_post", (x + sx * (w / 2 + t / 2), y_face + s * 0.04, z0), (x + sx * (w / 2 + t / 2), y_face + s * 0.04, z0 + h + t), 0.14, t, m.timber)
    beam("door_lintel", (x - w / 2 - t, y_face + s * 0.05, z0 + h + t / 2), (x + w / 2 + t, y_face + s * 0.05, z0 + h + t / 2), 0.16, t, m.timber)
    # Plankene, litt ulike, innfelt i døråpningen.
    n = 5
    if open:
        # Slått innover om hengslet: plankene langs en retning inn i huset.
        a = math.radians(95)
        d = Vector((math.cos(a), -s * math.sin(a), 0))
        hinge = Vector((x - w / 2, y_face - s * 0.06, 0))
        for k in range(n):
            u = w * (k + 0.5) / n
            block("door_plank", (w / n - 0.006, 0.05, h), m.boards, at=hinge + d * u + Vector((0, 0, z0 + h / 2)), rot=(0, 0, math.atan2(d.y, d.x)), bevel=0.005)
        for zz in (0.35, h - 0.35):
            block("door_strap", (w * 0.85, 0.02, 0.06), m.iron, at=hinge + d * (w * 0.45) + Vector((s * 0.03 * d.y, 0, z0 + zz)), rot=(0, 0, math.atan2(d.y, d.x)), bevel=0.004)
        block("step", (w + 0.5, 0.45, 0.18), m.stone, at=(x, y_face + s * 0.25, z0 - 0.06), bevel=0.03, segments=2)
        return
    for k in range(n):
        px = x - w / 2 + w * (k + 0.5) / n
        block("door_plank", (w / n - 0.006, 0.05, h), m.boards, at=(px, y_face - s * 0.06, z0 + h / 2), bevel=0.005)
    for zz in (0.35, h - 0.35):
        block("door_strap", (w * 0.85, 0.02, 0.06), m.iron, at=(x - w * 0.05, y_face - s * 0.03, z0 + zz), bevel=0.004)
    bpy.ops.mesh.primitive_torus_add(major_radius=0.06, minor_radius=0.01, location=(x + w * 0.3, y_face - s * 0.02, z0 + 1.0), rotation=(math.pi / 2, 0, 0))
    ring = bpy.context.active_object
    ring.data.materials.append(m.iron)
    uv_box(ring)
    PIECES.append(ring)
    # Steintrinnet.
    block("step", (w + 0.5, 0.45, 0.18), m.stone, at=(x, y_face + s * 0.25, z0 - 0.06), bevel=0.03, segments=2)


def timber_wall(m, name, x0, x1, y, z0, z1, normal_y, openings, rng, braces=True):
    """En bindingsverksvegg langs x i y: svill, stolper, løsholt, toppsvill og
    skråstag, med kalkpuss mellom, og hull for vinduer og dører.
    `openings` er (x, z, b, h, slag) med slag "w" (vindu) eller "d" (dør)."""
    s = normal_y
    bw = 0.2  # bjelkebredde i veggen
    thick = 0.22
    # Pussen: én plate litt innenfor bjelkene, med hull.
    wall = block(f"{name}_plaster", (x1 - x0, thick - 0.06, z1 - z0), m.plaster, at=((x0 + x1) / 2, y - s * 0.03, (z0 + z1) / 2), bevel=0)
    for ox, oz, ob, oh, kind in openings:
        cut(wall, (ob + (0.02 if kind == "w" else 0.04), 1.0, oh + 0.02), ((ox), y, oz))
    uv_box(wall)
    # Bjelkene i flukt med yta, litt ut.
    fy = y + s * 0.0
    first_beam = len(PIECES)
    beam(f"{name}_sill", (x0, fy, z0 + bw / 2), (x1, fy, z0 + bw / 2), thick, bw, m.timber)
    beam(f"{name}_plate", (x0, fy, z1 - bw / 2), (x1, fy, z1 - bw / 2), thick, bw, m.timber)
    # Stolper med jevne mellomrom, men ikke midt i en åpning.
    span = x1 - x0
    count = max(2, round(span / 1.25))
    posts = [x0 + bw / 2 + (span - bw) * k / count for k in range(count + 1)]
    blocked = lambda px: any(abs(px - ox) < ob / 2 + bw / 2 + 0.02 for ox, _, ob, _, _ in openings)
    posts = [p for p in posts if not blocked(p) or p in (posts[0], posts[-1])]
    for px in posts:
        beam(f"{name}_post", (px, fy, z0 + bw), (px, fy, z1 - bw), thick * 0.95, bw, m.timber, roll=math.pi / 2)
    # Løsholt i brysthøyde mellom stolpene der det ikke er åpninger, og
    # skråstag i endefeltene.
    mid = z0 + (z1 - z0) * 0.45
    for a, b in zip(posts[:-1], posts[1:]):
        c = (a + b) / 2
        hole = [o for o in openings if a < o[0] < b]
        if hole:
            ox, oz, ob, oh, kind = hole[0]
            # Bjelker rundt åpningen: over og under (vinduer).
            beam(f"{name}_head", (a + bw / 2, fy, oz + oh / 2 + 0.06), (b - bw / 2, fy, oz + oh / 2 + 0.06), thick * 0.9, 0.12, m.timber)
            if kind == "w":
                beam(f"{name}_under", (a + bw / 2, fy, oz - oh / 2 - 0.06), (b - bw / 2, fy, oz - oh / 2 - 0.06), thick * 0.9, 0.12, m.timber)
            continue
        beam(f"{name}_rail", (a + bw / 2, fy, mid), (b - bw / 2, fy, mid), thick * 0.9, bw * 0.8, m.timber)
        if braces and (a == posts[0] or b == posts[-1]) and rng.random() < 0.9:
            # Skråstaget går fra svilla ved hjørnet opp mot løsholtet.
            if a == posts[0]:
                beam(f"{name}_brace", (a + bw / 2, fy, z0 + bw), (b - bw / 2, fy, mid - bw * 0.4), thick * 0.85, bw * 0.8, m.timber)
            else:
                beam(f"{name}_brace", (b - bw / 2, fy, z0 + bw), (a + bw / 2, fy, mid - bw * 0.4), thick * 0.85, bw * 0.8, m.timber)
    # Pussen skjæres bort der bjelkene ligger, så den bare finnes i feltene
    # mellom dem: ingen hjørner gjemt under bjelkene (de ville fått svart
    # okklusjon som smitter ut i feltet).
    carve(wall, PIECES[first_beam:])


def side_wall(m, name, y0, y1, x, z0, z1, normal_x, openings, rng):
    """En bindingsverksvegg langs y i x: bygd som en vegg langs x og dreid 90°."""
    # Bygg i et midlertidig rom: x' = y, normalen ±y'; så drei om z.
    before = len(PIECES)
    # Dreid −90° speiles y; bygg fra −y1 til −y0 så veggen havner riktig.
    if normal_x < 0:
        y0, y1 = -y1, -y0
        openings = [(-o[0], *o[1:]) for o in openings]
    timber_wall(m, name, y0, y1, 0.0, z0, z1, -1.0, openings, rng)
    for ox, oz, ob, oh, kind in openings:
        if kind == "w":
            window(m, ox, oz, -0.11, ob, oh, -1.0, shutters=rng.random() < 0.5, rng=rng)
    rot = Matrix.Rotation(math.radians(90 if normal_x > 0 else -90), 4, "Z")
    for ob in PIECES[before:]:
        mw = Matrix.Translation((x, 0, 0)) @ rot @ ob.matrix_world
        ob.matrix_world = mw
    _ = name


def roof(m, w, d, eave, pitch_deg, overhang=0.45, gable_to_street=True):
    """Saltak med spontak: to takflater, vindskier, mønebord, og takfot.
    Gavlen mot gata betyr mønet langs y."""
    pitch = math.radians(pitch_deg)
    run = (w if gable_to_street else d) / 2 + overhang
    length = (d if gable_to_street else w) + overhang * 2
    slope = run / math.cos(pitch)
    rise = run * math.tan(pitch)
    for side in (-1, 1):
        # Takflaten bygd flat (u langs mønet, v opp skrått), så vippet.
        if gable_to_street:
            ob = block("roof", (slope, length, 0.16), m.roof, bevel=0.01)
            # Uv: u langs mønet (y), v opp fallet: bygg uv på nytt for det.
            _roof_uv(ob, along_y=True)
            ob.rotation_euler = Euler((0, side * pitch, 0))
            ob.location = (side * run / 2, 0, eave + rise / 2 + 0.08)
            # Vindskiene langs gavlkantene.
            for sy in (-1, 1):
                a = Vector((side * run, sy * length / 2, eave + 0.05))
                b = Vector((0, sy * length / 2, eave + rise + 0.12))
                beam("barge", a, b, 0.05, 0.22, m.boards)
        else:
            ob = block("roof", (length, slope, 0.16), m.roof, bevel=0.01)
            _roof_uv(ob, along_y=False)
            ob.rotation_euler = Euler((-side * pitch, 0, 0))
            ob.location = (0, side * run / 2, eave + rise / 2 + 0.08)
            for sx in (-1, 1):
                a = Vector((sx * length / 2, side * run, eave + 0.05))
                b = Vector((sx * length / 2, 0, eave + rise + 0.12))
                beam("barge", a, b, 0.05, 0.22, m.boards)
    # Mønebordene.
    if gable_to_street:
        beam("ridge", (0, -length / 2, eave + rise + 0.2), (0, length / 2, eave + rise + 0.2), 0.3, 0.08, m.boards)
    else:
        beam("ridge", (-length / 2, 0, eave + rise + 0.2), (length / 2, 0, eave + rise + 0.2), 0.3, 0.08, m.boards)
    return rise


def _roof_uv(ob, along_y):
    """Takflatene: u langs mønet, v opp fallet, i meter, på oversida."""
    me = ob.data
    bm = bmesh.new()
    bm.from_mesh(me)
    uvl = bm.loops.layers.uv.verify()
    for f in bm.faces:
        for loop in f.loops:
            p = loop.vert.co
            loop[uvl].uv = (p.y, p.x) if along_y else (p.x, p.y)
    bm.to_mesh(me)
    bm.free()


def gable(m, w, eave, rise, y, normal_y, rng, boards=True):
    """Gavltrekanten: stående bord eller bindingsverk med puss."""
    pts = [(-w / 2, 0.0), (w / 2, 0.0), (0.0, rise)]
    mat = m.boards if boards else m.plaster
    ob = prism("gable", pts, 0.16, mat, at=(0, y, eave))
    if boards:
        # Bordene står: uv med v opp er riktig fra uv_box (x, z).
        # Lekter over skjøtene.
        n = int(w / 0.22)
        for k in range(1, n):
            x = -w / 2 + w * k / n
            top = rise * (1 - abs(x) / (w / 2))
            if top > 0.15:
                beam("batten", (x, y + normal_y * 0.09, eave + 0.02), (x, y + normal_y * 0.09, eave + top - 0.05), 0.02, 0.05, m.boards, roll=math.pi / 2)
        # Et lite lufteluke-vindu øverst.
        window(m, 0.0, eave + rise * 0.4, y + normal_y * 0.08, 0.35, 0.4, normal_y, shutters=False)
    return ob


# Pipene som ryker: toppen og tykkelsen (0..1); `finish` skriver dem som
# «smoke».
SMOKE = []


def chimney(m, x, y, z0, z1, smoke=0.6):
    """Steinpipe: murt av enkeltstein, med en krage øverst. Røyken stiger
    fra toppen (`smoke` er tykkelsen)."""
    SMOKE.append(((x, y, z1 + 0.15), smoke))
    rng = random.Random(int(x * 100 + y * 10))
    w = 0.7
    block("chimney", (w, w, z1 - z0), m.stone, at=(x, y, (z0 + z1) / 2), bevel=0.02)
    block("chimney_cap", (w + 0.16, w + 0.16, 0.12), m.stone, at=(x, y, z1 + 0.05), bevel=0.02)
    block("chimney_hole", (w - 0.25, w - 0.25, 0.1), layer("stone", (0.08, 0.07, 0.06), 0.5), at=(x, y, z1 + 0.08), bevel=0)
    _ = rng


def plinth(m, w, d, h, rng):
    """Steinsokkelen: murt av kampestein, litt bredere enn huset."""
    block("plinth", (w + 0.16, d + 0.16, h + 0.24), m.stone, at=(0, 0, h / 2 - 0.12), bevel=0.04, segments=2)
    # Enkeltstein som stikker litt ut, så sokkelen ikke er en glatt kloss.
    for k in range(int((w + d) * 2.2)):
        side = rng.choice((0, 1, 2, 3))
        if side < 2:
            x = rng.uniform(-w / 2, w / 2)
            y = (-1 if side == 0 else 1) * (d / 2 + 0.09)
            size = (rng.uniform(0.25, 0.5), 0.1, rng.uniform(0.15, h * 0.7))
        else:
            y = rng.uniform(-d / 2, d / 2)
            x = (-1 if side == 2 else 1) * (w / 2 + 0.09)
            size = (0.1, rng.uniform(0.25, 0.5), rng.uniform(0.15, h * 0.7))
        block("plinth_stone", size, m.stone, at=(x, y, rng.uniform(0.1, h - 0.1)), bevel=0.03, segments=2)


# --- Husene ----------------------------------------------------------------


def half_timbered(spec):
    """Et bindingsverkshus med gavlen eller langsida mot gata."""
    rng = random.Random(spec["seed"])
    m = M(plaster_tint=tuple(c * 0.42 for c in spec.get("plaster", (1, 0.965, 0.905))), timber_tint=spec.get("timber", (1, 1, 1)), roof=spec.get("roof", "tiles"))
    w, d = spec["w"], spec["d"]
    floors = spec["floors"]
    fh = 2.7
    base = spec.get("plinth", 0.55)
    jetty = spec.get("jetty", 0.35)
    plinth(m, w, d, base, rng)
    z = base
    for f in range(floors):
        ext = jetty * f  # framskutt overetasje mot gata
        fy = -d / 2 - ext
        z1 = z + fh
        # Fasaden mot gata: dør i første etasje, vinduer.
        openings = []
        if f == 0:
            dx = spec.get("door_x", -w * 0.22)
            openings.append((dx, z + 1.0 + 0.05, 1.0, 2.0, "d"))
            for wx in spec.get("windows0", [w * 0.22]):
                openings.append((wx, z + 1.45, 0.8, 0.95, "w"))
        else:
            n = spec.get("windows_up", 2)
            for k in range(n):
                openings.append((-w / 2 + w * (k + 0.5) / n, z + 1.45, 0.8, 0.95, "w"))
        timber_wall(m, f"front{f}", -w / 2, w / 2, fy, z, z1, -1.0, openings, rng)
        for ox, oz, ob, oh, kind in openings:
            if kind == "w":
                window(m, ox, oz, fy - 0.11, ob, oh, -1.0, shutters=f > 0 or rng.random() < 0.7, rng=rng)
            else:
                door(m, ox, fy - 0.11, -1.0, ob, oh, z, open=spec.get("open_door", False))
                if spec.get("lantern", True):
                    wall_lantern(m, ox + 0.95 * spec.get("lantern_side", 1), fy - 0.1, z + 2.25, -1.0)
        # Baksida og sidene.
        back = [(0.0, z + 1.45, 0.7, 0.8, "w")] if rng.random() < 0.8 else []
        timber_wall(m, f"back{f}", -w / 2, w / 2, d / 2, z, z1, 1.0, back, rng)
        for ox, oz, ob, oh, _ in back:
            window(m, ox, oz, d / 2 + 0.11, ob, oh, 1.0, shutters=False)
        for sx in (-1, 1):
            side_open = [(rng.uniform(-d * 0.2, d * 0.2), z + 1.45, 0.7, 0.8, "w")] if rng.random() < 0.6 else []
            side_wall(m, f"side{f}{sx}", -d / 2 - ext, d / 2, sx * w / 2, z, z1, sx, side_open, rng)
        # Bjelkeendene under den framskutte etasjen.
        if f > 0 and jetty > 0:
            for k in range(int(w / 0.6) + 1):
                x = -w / 2 + 0.15 + (w - 0.3) * k / max(1, int(w / 0.6))
                beam("joist", (x, -d / 2 + 0.3, z - 0.1), (x, -d / 2 - ext - 0.12, z - 0.1), 0.16, 0.18, m.timber)
            # Gulvbjelken som bærer utkragingen.
            beam("bressumer", (-w / 2, -d / 2 - ext, z + 0.05), (w / 2, -d / 2 - ext, z + 0.05), 0.24, 0.22, m.timber)
        z = z1
    # Taket.
    eave = z
    gable_street = spec.get("gable", True)
    rise = roof(m, w + 0.1, d + jetty * (floors - 1), eave, spec.get("pitch", 52), gable_to_street=gable_street)
    if gable_street:
        for sy, ny in ((-d / 2 - jetty * (floors - 1), -1.0), (d / 2, 1.0)):
            gable(m, w, eave, rise, sy, ny, rng, boards=spec.get("gable_boards", True))
    else:
        # Gavlene på sidene: bygd langs x og dreid.
        for sx in (-1, 1):
            before = len(PIECES)
            gable(m, d + jetty * (floors - 1), eave, rise, 0.0, -1.0, rng)
            rot = Matrix.Rotation(math.radians(90 if sx > 0 else -90), 4, "Z")
            for ob in PIECES[before:]:
                ob.matrix_world = Matrix.Translation((sx * w / 2, -jetty * (floors - 1) / 2, 0)) @ rot @ ob.matrix_world
    if spec.get("chimney", True):
        chimney(m, w * 0.22, d * 0.15, eave + rise * 0.4, eave + rise + 0.8)
    return m


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
    for p in ob.data.polygons:
        p.use_smooth = False
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
    if COLLIDERS or LIGHTS or HULLS or LAMPS or SMOKE:
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
        }
        with open(path.replace(".glb", "_inside.json"), "w") as f:
            json.dump(data, f)
        print(f"  {len(COLLIDERS)} kollidere, {len(LIGHTS)} lys, {len(LAMPS)} lykter, {len(SMOKE)} piper")
    COLLIDERS.clear()
    LAMPS.clear()
    SMOKE.clear()
    RELIEF.clear()
    LIGHTS.clear()
    HULLS.clear()


HOUSES = {
    # Smalt hus med gavl mot gata, to etasjer, framskutt.
    "house_a": dict(seed=1, w=6.4, d=8.0, floors=2, jetty=0.4, pitch=54, door_x=-1.4, windows0=[1.3], windows_up=2),
    # Bredere, langsida mot gata, okerpuss.
    "house_b": dict(seed=2, roof="slate", w=9.0, d=6.5, floors=2, jetty=0.3, pitch=48, gable=False, plaster=(0.95, 0.85, 0.66), door_x=-2.4, windows0=[0.4, 2.8], windows_up=3),
    # Lavt hus med én etasje og bratt tak.
    "house_c": dict(seed=3, roof="thatch", w=7.0, d=7.5, floors=1, jetty=0.0, pitch=56, door_x=1.6, windows0=[-1.5], windows_up=0, plinth=0.7),
    # Høyt, smalt kjøpmannshus med tre etasjer.
    "house_d": dict(seed=4, roof="slate", w=6.0, d=9.0, floors=3, jetty=0.32, pitch=56, door_x=-1.3, windows0=[1.2], windows_up=2, plaster=(1.0, 0.97, 0.92)),
    # Langsida mot gata, to etasjer, mørkere tømmer.
    "house_e": dict(seed=5, w=10.0, d=7.0, floors=2, jetty=0.35, pitch=46, gable=False, door_x=0.0, windows0=[-3.0, 3.0], windows_up=4, timber=(0.8, 0.78, 0.76)),
    # Gavlhus med puss i gavlen og blekrosa puss.
    "house_f": dict(seed=6, roof="thatch", w=7.2, d=8.5, floors=2, jetty=0.4, pitch=50, door_x=1.8, windows0=[-1.6], windows_up=3, gable_boards=False, plaster=(0.98, 0.9, 0.86)),
}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in HOUSES] or list(HOUSES)
    for name in names:
        clear()
        PIECES.clear()
        half_timbered(HOUSES[name])
        finish(f"{OUT}/{name}.glb")
