"""Bilene i Fristad, modellert i Blender (bpy som Python-modul) og
eksportert som glTF til assets/models/cars/. Kjør:

    python3 tools/models/cars.py            # alle tre
    python3 tools/models/cars.py estate     # én

Karosseriet loftes mellom tverrsnitt langs bilen og glattes med
subdivision, hjulbuene skjæres ut med boolean, og hjul, lykter, speil,
støtfangere og grill legges til som egne objekter. Materialet «paint» får
fargen per instans i motoren (flagget PAINT), resten har faste farger.
Bilen peker mot +X, opp er +Z i Blender; glTF-eksporten gjør det til Y opp.
"""

import math
import sys

import bpy
import bmesh
from mathutils import Vector

OUT = "assets/models/cars"


def clear():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def material(name, rgb, rough, metal=0.0):
    m = bpy.data.materials.get(name)
    if m:
        return m
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*rgb, 1.0)
    bsdf.inputs["Roughness"].default_value = rough
    bsdf.inputs["Metallic"].default_value = metal
    return m


def mats():
    return {
        "paint": material("paint", (1.0, 1.0, 1.0), 0.3),
        "glass": material("glass", (0.04, 0.05, 0.07), 0.05),
        "chrome": material("chrome", (0.9, 0.9, 0.92), 0.25, 1.0),
        "tyre": material("tyre", (0.02, 0.02, 0.02), 0.9),
        "plastic": material("plastic", (0.06, 0.06, 0.065), 0.7),
        "lamp": material("lamp", (0.95, 0.95, 0.9), 0.1),
        "tail": material("tail", (0.8, 0.05, 0.03), 0.15),
        "amber": material("amber", (0.95, 0.5, 0.05), 0.15),
        "plate": material("plate", (0.92, 0.92, 0.9), 0.5),
        "interior": material("interior", (0.07, 0.065, 0.06), 0.9),
    }


def new_object(name, bm, mat_list):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    for m in mat_list:
        me.materials.append(m)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def smooth(ob, angle=40.0):
    for p in ob.data.polygons:
        p.use_smooth = True
    with bpy.context.temp_override(object=ob, selected_editable_objects=[ob]):
        bpy.ops.object.shade_smooth_by_angle(angle=math.radians(angle))


def apply_modifiers(ob):
    bpy.context.view_layer.objects.active = ob
    for mod in list(ob.modifiers):
        with bpy.context.temp_override(object=ob):
            bpy.ops.object.modifier_apply(modifier=mod.name)


# ---------------------------------------------------------------- karosseri

class Section:
    """Et halvt tverrsnitt (y >= 0) nedenfra og opp: gulv, kant, terskel,
    beltelinje (bredest), skulder (vindusbunn), vindustopp, takkant, takmidt.
    Verdiene er (bredde, høyde)."""

    def __init__(self, floor_w, floor_z, sill_w, sill_z, belt_w, belt_z, shoulder_w, shoulder_z, glass_w, glass_z, roof_w, roof_z):
        # Glasset ligger 2 cm innenfor skulderen, så ruta leses som en flate
        # i karosseriet og ikke som en fortsettelse av det.
        self.pts = [
            (0.0, floor_z),
            (floor_w, floor_z),
            (sill_w, sill_z),
            (belt_w, belt_z),
            (shoulder_w, shoulder_z),
            (min(glass_w, shoulder_w - 0.02), glass_z),
            (roof_w, roof_z),
            (0.0, roof_z + 0.01),
        ]

    def ring(self, x):
        right = [Vector((x, y, z)) for (y, z) in self.pts]
        left = [Vector((x, -y, z)) for (y, z) in reversed(self.pts[1:-1])]
        return right + left


# Ringen har 14 punkter: høyre side 0–7 nedenfra (0 gulvmidt, 4 skulder,
# 5 vindustopp, 7 takmidt), venstre side 8–13 speilet ovenfra. Sidevinduene
# er segmentene skulder→vindustopp, frontrute og bakrute er toppsegmentene
# mellom to stasjoner der taket skrår.
RING = 14
SIDE = {(4, 5), (9, 10)}
TOP = {(5, 6), (6, 7), (7, 8), (8, 9)}


def glass_segments(kind):
    return {"side": SIDE, "top": TOP, "all": SIDE | TOP}[kind]


def loft(name, stations, glass_between, mats_, crease_rings=()):
    """stations: liste av (x, Section). glass_between: {i: "side"|"top"|"all"}
    for stasjonspar (i, i+1) som har glass. crease_rings: stasjoner der
    ringen er en skarp kant (panserkant, vindusbunn, takkant)."""
    bm = bmesh.new()
    rings = []
    for x, sec in stations:
        ring = [bm.verts.new(p) for p in sec.ring(x)]
        rings.append(ring)
    bm.verts.ensure_lookup_table()
    glass_faces = []
    crease = bm.edges.layers.float.get("crease_edge") or bm.edges.layers.float.new("crease_edge")
    for i in range(len(rings) - 1):
        a, b = rings[i], rings[i + 1]
        n = len(a)
        for k in range(n):
            k2 = (k + 1) % n
            try:
                f = bm.faces.new((a[k], a[k2], b[k2], b[k]))
            except ValueError:
                continue
            if i in glass_between and (k, k2) in glass_segments(glass_between[i]):
                glass_faces.append(f)
    # Endene lukkes.
    for ring in (rings[0], list(reversed(rings[-1]))):
        try:
            bm.faces.new(ring)
        except ValueError:
            pass
    # Skarpe kanter langs bilen: terskel, beltelinje og vindusbunn (skulder).
    n = RING
    along = {2: 0.6, 3: 1.0, 4: 0.9, n - 4: 0.9, n - 3: 1.0, n - 2: 0.6}
    for i in range(len(rings) - 1):
        a, b = rings[i], rings[i + 1]
        for k, c in along.items():
            e = bm.edges.get((a[k], b[k]))
            if e:
                e[crease] = c
    # Skarpe kanter rundt bilen: der panseret møter frontruta, taket osv.
    for i in crease_rings:
        ring = rings[i]
        for k in range(n):
            e = bm.edges.get((ring[k], ring[(k + 1) % n]))
            if e:
                e[crease] = 1.0
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        f.material_index = 1 if f in glass_faces else 0
    ob = new_object(name, bm, [mats_["paint"], mats_["glass"]])
    return ob


def subdivide(ob, levels=2):
    mod = ob.modifiers.new("subd", "SUBSURF")
    mod.levels = levels
    mod.render_levels = levels
    mod.use_creases = True
    apply_modifiers(ob)


def cut_arches(ob, wheel_xs, wheel_z, radius, width):
    for x in wheel_xs:
        for side in (1, -1):
            bpy.ops.mesh.primitive_cylinder_add(vertices=40, radius=radius, depth=width, location=(x, side * (width * 0.5 + 0.35), wheel_z), rotation=(math.pi / 2, 0, 0))
            cyl = bpy.context.active_object
            mod = ob.modifiers.new("arch", "BOOLEAN")
            mod.operation = "DIFFERENCE"
            mod.object = cyl
            mod.solver = "EXACT"
            apply_modifiers(ob)
            bpy.data.objects.remove(cyl)


def bevel(ob, width=0.015, segments=2):
    mod = ob.modifiers.new("bevel", "BEVEL")
    mod.width = width
    mod.segments = segments
    mod.limit_method = "ANGLE"
    mod.angle_limit = math.radians(35)
    apply_modifiers(ob)


# ------------------------------------------------------------------- deler

def box(name, center, size, mat, rot=(0, 0, 0)):
    bpy.ops.mesh.primitive_cube_add(location=center, rotation=rot)
    ob = bpy.context.active_object
    ob.name = name
    ob.scale = (size[0] / 2, size[1] / 2, size[2] / 2)
    ob.data.materials.append(mat)
    return ob


def rounded_box(name, center, size, mat, radius=0.01, rot=(0, 0, 0)):
    ob = box(name, center, size, mat, rot)
    with bpy.context.temp_override(object=ob, selected_editable_objects=[ob]):
        bpy.ops.object.transform_apply(scale=True)
    mod = ob.modifiers.new("bevel", "BEVEL")
    mod.width = radius
    mod.segments = 2
    apply_modifiers(ob)
    smooth(ob, 50)
    return ob


def ellipsoid(name, center, size, mat):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=20, ring_count=10, location=center)
    ob = bpy.context.active_object
    ob.name = name
    ob.scale = (size[0] / 2, size[1] / 2, size[2] / 2)
    ob.data.materials.append(mat)
    smooth(ob, 80)
    return ob


def wheel(name, x, y, z, radius, width, m):
    """Dekk som torus, felg som sylinder med fem eiker, nav."""
    tyre_minor = radius * 0.22
    bpy.ops.mesh.primitive_torus_add(major_segments=28, minor_segments=12, major_radius=radius - tyre_minor, minor_radius=tyre_minor, location=(x, y, z), rotation=(math.pi / 2, 0, 0))
    tyre = bpy.context.active_object
    tyre.name = name + "_tyre"
    tyre.scale = (1.0, 1.0, width / (2 * tyre_minor))
    tyre.data.materials.append(m["tyre"])
    smooth(tyre, 60)
    rim_r = radius - tyre_minor * 1.1
    bpy.ops.mesh.primitive_cylinder_add(vertices=32, radius=rim_r, depth=width * 0.6, location=(x, y, z), rotation=(math.pi / 2, 0, 0))
    rim = bpy.context.active_object
    rim.name = name + "_rim"
    rim.data.materials.append(m["plastic"])
    smooth(rim, 40)
    # Eikene og navet, i krom, litt utenfor felgen.
    side = 1 if y > 0 else -1
    for k in range(5):
        a = k * 2 * math.pi / 5
        sx = (rim_r * 0.45) * math.cos(a)
        sz = (rim_r * 0.45) * math.sin(a)
        box(name + f"_spoke{k}", (x + sx, y + side * width * 0.32, z + sz), (rim_r * 0.85, 0.02, rim_r * 0.22), m["chrome"], rot=(0, a, 0))
    bpy.ops.mesh.primitive_cylinder_add(vertices=20, radius=rim_r * 0.3, depth=0.04, location=(x, y + side * width * 0.33, z), rotation=(math.pi / 2, 0, 0))
    hub = bpy.context.active_object
    hub.name = name + "_hub"
    hub.data.materials.append(m["chrome"])
    smooth(hub, 40)
    bpy.ops.mesh.primitive_cylinder_add(vertices=32, radius=rim_r * 0.98, depth=0.01, location=(x, y + side * width * 0.3, z), rotation=(math.pi / 2, 0, 0))
    disc = bpy.context.active_object
    disc.name = name + "_disc"
    disc.data.materials.append(m["chrome"])


# -------------------------------------------------------------------- biler

def estate_stations(L, W, H):
    """Kantete stasjonsvogn à la Volvo 240: lang flat panser, loddrett
    bakende, stor bakluke."""
    hw = W / 2
    fz = 0.32          # gulvet (over hjulsenteret ligger karosseriet)
    sill = 0.42
    belt = 0.95        # beltelinja
    hood = 1.0
    roof = H
    def body(x_t, top, glass_top, shoulder, gw, rw, flat=False):
        return Section(hw * 0.78, fz, hw * 0.96, sill, hw, belt, hw * 0.97, shoulder, gw, glass_top, rw, top)
    S = []
    S.append((L / 2, Section(hw * 0.7, fz + 0.06, hw * 0.82, sill + 0.08, hw * 0.9, belt - 0.1, hw * 0.86, hood - 0.08, hw * 0.84, hood - 0.06, hw * 0.8, hood - 0.05)))
    S.append((L / 2 - 0.12, Section(hw * 0.76, fz, hw * 0.94, sill, hw * 0.99, belt - 0.02, hw * 0.95, hood - 0.02, hw * 0.93, hood, hw * 0.9, hood + 0.01)))
    S.append((L / 2 - 1.2, body(0, hood + 0.03, hood + 0.02, hood, hw * 0.95, hw * 0.92)))
    S.append((L / 2 - 1.55, body(0, hood + 0.05, hood + 0.04, hood + 0.02, hw * 0.95, hw * 0.92)))   # vindusbunn
    S.append((L / 2 - 2.15, body(0, roof, roof - 0.02, hood + 0.03, hw * 0.86, hw * 0.78)))        # taket starter (A-stolpe topp)
    S.append((L / 2 - 2.9, body(0, roof + 0.01, roof - 0.01, hood + 0.03, hw * 0.88, hw * 0.8)))   # B-stolpe
    S.append((L / 2 - 3.02, body(0, roof + 0.01, roof - 0.01, hood + 0.03, hw * 0.88, hw * 0.8)))
    S.append((-L / 2 + 0.55, body(0, roof - 0.01, roof - 0.03, hood + 0.03, hw * 0.88, hw * 0.8)))  # C-stolpe
    S.append((-L / 2 + 0.4, body(0, roof - 0.04, roof - 0.06, hood + 0.03, hw * 0.88, hw * 0.8)))
    S.append((-L / 2 + 0.05, Section(hw * 0.76, fz, hw * 0.94, sill, hw * 0.99, belt, hw * 0.94, hood + 0.02, hw * 0.9, hood + 0.05, hw * 0.86, hood + 0.07)))
    S.append((-L / 2, Section(hw * 0.7, fz + 0.06, hw * 0.84, sill + 0.06, hw * 0.92, belt - 0.05, hw * 0.9, hood, hw * 0.86, hood + 0.02, hw * 0.8, hood + 0.03)))
    # Frontrute i toppen mellom vindusbunn og takstart, sidevinduer i
    # båndet, bakrute i den bratte bakluka.
    glass = {3: "top", 4: "side", 6: "side", 7: "side", 8: "top"}
    return S, glass


def hatch_stations(L, W, H):
    """Liten kombi med rund front og skrå bakrute."""
    hw = W / 2
    fz = 0.3
    sill = 0.4
    belt = 0.9
    hood = 0.92
    roof = H
    def body(top, glass_top, shoulder, gw, rw):
        return Section(hw * 0.78, fz, hw * 0.96, sill, hw, belt, hw * 0.97, shoulder, gw, glass_top, rw, top)
    S = []
    S.append((L / 2, Section(hw * 0.68, fz + 0.08, hw * 0.8, sill + 0.1, hw * 0.86, belt - 0.15, hw * 0.82, hood - 0.18, hw * 0.78, hood - 0.16, hw * 0.72, hood - 0.15)))
    S.append((L / 2 - 0.15, Section(hw * 0.76, fz, hw * 0.94, sill, hw * 0.98, belt - 0.05, hw * 0.94, hood - 0.06, hw * 0.9, hood - 0.04, hw * 0.86, hood - 0.03)))
    S.append((L / 2 - 0.9, body(hood + 0.04, hood + 0.03, hood + 0.01, hw * 0.94, hw * 0.9)))
    S.append((L / 2 - 1.25, body(hood + 0.08, hood + 0.06, hood + 0.04, hw * 0.94, hw * 0.9)))
    S.append((L / 2 - 1.95, body(roof, roof - 0.02, hood + 0.05, hw * 0.86, hw * 0.76)))
    S.append((L / 2 - 2.6, body(roof + 0.02, roof, hood + 0.05, hw * 0.88, hw * 0.78)))
    S.append((L / 2 - 2.72, body(roof + 0.02, roof, hood + 0.05, hw * 0.88, hw * 0.78)))
    S.append((-L / 2 + 0.6, body(roof - 0.02, roof - 0.04, hood + 0.05, hw * 0.86, hw * 0.76)))
    S.append((-L / 2 + 0.15, Section(hw * 0.76, fz, hw * 0.94, sill, hw * 0.98, belt, hw * 0.92, hood + 0.08, hw * 0.8, roof - 0.3, hw * 0.72, roof - 0.28)))
    S.append((-L / 2, Section(hw * 0.7, fz + 0.06, hw * 0.84, sill + 0.06, hw * 0.92, belt - 0.03, hw * 0.88, hood + 0.02, hw * 0.78, roof - 0.42, hw * 0.7, roof - 0.4)))
    glass = {3: "top", 4: "side", 6: "side", 7: "top"}
    return S, glass


def van_stations(L, W, H):
    """Varebil med flat front og kasse bak."""
    hw = W / 2
    fz = 0.35
    sill = 0.45
    belt = 1.05
    hood = 1.1
    roof = H
    def body(top, glass_top, shoulder, gw, rw):
        return Section(hw * 0.8, fz, hw * 0.97, sill, hw, belt, hw * 0.98, shoulder, gw, glass_top, rw, top)
    S = []
    S.append((L / 2, Section(hw * 0.72, fz + 0.06, hw * 0.86, sill + 0.08, hw * 0.92, belt - 0.2, hw * 0.9, hood - 0.25, hw * 0.86, hood - 0.22, hw * 0.8, hood - 0.2)))
    S.append((L / 2 - 0.12, Section(hw * 0.78, fz, hw * 0.95, sill, hw * 0.99, belt - 0.1, hw * 0.96, hood - 0.08, hw * 0.92, hood - 0.05, hw * 0.88, hood - 0.04)))
    S.append((L / 2 - 0.5, body(hood + 0.02, hood + 0.01, hood, hw * 0.95, hw * 0.9)))
    S.append((L / 2 - 0.75, body(hood + 0.05, hood + 0.04, hood + 0.02, hw * 0.95, hw * 0.9)))
    S.append((L / 2 - 1.5, body(roof, roof - 0.02, hood + 0.03, hw * 0.9, hw * 0.82)))
    S.append((L / 2 - 2.2, body(roof + 0.01, roof - 0.01, hood + 0.03, hw * 0.9, hw * 0.82)))
    S.append((L / 2 - 2.32, body(roof + 0.01, roof - 0.01, hood + 0.03, hw * 0.9, hw * 0.82)))
    S.append((-L / 2 + 0.1, body(roof + 0.01, roof - 0.01, hood + 0.03, hw * 0.9, hw * 0.82)))
    S.append((-L / 2, Section(hw * 0.76, fz + 0.04, hw * 0.92, sill + 0.04, hw * 0.97, belt, hw * 0.94, hood + 0.02, hw * 0.86, roof - 0.06, hw * 0.78, roof - 0.05)))
    glass = {3: "top", 4: "side"}   # frontrute og fordørvindu; kassa bak er tett
    return S, glass


SPECS = {
    "estate": dict(L=4.8, W=1.72, H=1.46, wheel_r=0.32, wheel_w=0.2, wheels=(1.45, -1.35), build=estate_stations, rails=True, crease_rings=(1, 3, 4, 7, 9)),
    "hatch": dict(L=3.9, W=1.66, H=1.5, wheel_r=0.3, wheel_w=0.19, wheels=(1.2, -1.2), build=hatch_stations, rails=False, crease_rings=(1, 3, 4, 7, 8)),
    "van": dict(L=4.9, W=1.9, H=1.95, wheel_r=0.33, wheel_w=0.21, wheels=(1.5, -1.5), build=van_stations, rails=False, crease_rings=(1, 3, 4, 7, 8)),
}


def build(kind):
    clear()
    m = mats()
    s = SPECS[kind]
    L, W, H = s["L"], s["W"], s["H"]
    stations, glass = s["build"](L, W, H)
    body = loft("body", stations, glass, m, s.get("crease_rings", ()))
    subdivide(body, 2)
    wz = s["wheel_r"]
    cut_arches(body, s["wheels"], wz, s["wheel_r"] + 0.07, 0.9)
    bevel(body, 0.012, 2)
    smooth(body, 32)

    for x in s["wheels"]:
        for side in (1, -1):
            wheel(f"wheel_{x:+.1f}_{side}", x, side * (W / 2 - s["wheel_w"] * 0.6), wz, s["wheel_r"], s["wheel_w"], m)

    # Støtfangere, grill, lykter, skilt, speil, håndtak, eksos.
    sill_z = 0.38 if kind != "van" else 0.42
    bz = sill_z + 0.08
    rounded_box("bumper_f", (L / 2 - 0.02, 0, bz), (0.12, W * 0.9, 0.16), m["plastic"], 0.03)
    rounded_box("bumper_r", (-L / 2 + 0.02, 0, bz), (0.12, W * 0.9, 0.16), m["plastic"], 0.03)
    hood_z = {"estate": 1.0, "hatch": 0.92, "van": 1.1}[kind]
    gz = hood_z - 0.2
    rounded_box("grille", (L / 2 - 0.0, 0, gz - 0.06), (0.06, W * 0.36, 0.12), m["plastic"], 0.01)
    for side in (1, -1):
        y = side * W * 0.33
        rounded_box(f"lamp_{side}", (L / 2 - 0.005, y, gz), (0.05, W * 0.2, 0.13), m["lamp"], 0.02)
        rounded_box(f"blink_{side}", (L / 2 - 0.01, side * W * 0.46, gz - 0.02), (0.04, 0.06, 0.09), m["amber"], 0.01)
        rounded_box(f"tail_{side}", (-L / 2 + 0.005, side * W * 0.36, gz + 0.08), (0.04, W * 0.2, 0.16), m["tail"], 0.02)
        # Speil på A-stolpen.
        ax = stations[3][0] - 0.1
        rounded_box(f"mirror_{side}", (ax, side * (W / 2 + 0.09), hood_z + 0.08), (0.1, 0.16, 0.1), m["plastic"], 0.02)
        rounded_box(f"mirror_arm_{side}", (ax, side * (W / 2 + 0.03), hood_z + 0.05), (0.06, 0.08, 0.03), m["plastic"], 0.005)
        # Dørhåndtak.
        for hx in (stations[4][0] - 0.75, stations[6][0] - 0.65):
            rounded_box(f"handle_{side}_{hx:.1f}", (hx, side * (W / 2 + 0.005), hood_z - 0.2), (0.14, 0.02, 0.03), m["chrome"], 0.005)
    rounded_box("plate_f", (L / 2 + 0.02, 0, bz + 0.17), (0.02, 0.5, 0.11), m["plate"], 0.003)
    rounded_box("plate_r", (-L / 2 - 0.0, 0, bz + 0.17), (0.02, 0.5, 0.11), m["plate"], 0.003)
    bpy.ops.mesh.primitive_cylinder_add(vertices=12, radius=0.03, depth=0.15, location=(-L / 2 + 0.05, -W * 0.3, sill_z - 0.04), rotation=(0, math.pi / 2, 0))
    ex = bpy.context.active_object
    ex.data.materials.append(m["chrome"])
    if s["rails"]:
        roof_z = max(v.co.z for v in body.data.vertices)
        for side in (1, -1):
            bpy.ops.mesh.primitive_cylinder_add(vertices=10, radius=0.02, depth=L * 0.42, location=(-0.3, side * W * 0.36, roof_z + 0.03), rotation=(0, math.pi / 2, 0))
            r = bpy.context.active_object
            r.data.materials.append(m["plastic"])
    # Mørkt interiør bak glasset.
    cab = (stations[4][0], stations[7][0] if kind != "van" else stations[6][0])
    box("interior", ((cab[0] + cab[1]) / 2, 0, hood_z + 0.12), (abs(cab[0] - cab[1]) - 0.1, W * 0.86, 0.5), m["interior"])

    # Alt til én glTF, uten hjelpeobjekter.
    path = f"{OUT}/{kind}.glb"
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=False, export_animations=False, export_skins=False, export_cameras=False, export_lights=False)
    tris = sum(len(p.vertices) - 2 for ob in bpy.data.objects if ob.type == "MESH" for p in ob.data.polygons)
    print(f"{path}: {tris} trekanter")


if __name__ == "__main__":
    kinds = [a for a in sys.argv[1:] if a in SPECS] or list(SPECS)
    for k in kinds:
        build(k)
