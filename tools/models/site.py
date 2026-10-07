"""Byggeplassen rundt skeleton-huset: brakke, minigraver, sementblander,
pall med lecablokker, byggegjerde, trillebår, toalett og tårnkran.
Modellert i Blender (bpy), bakt til teksturer (albedo × AO, normalkart,
ruhet/metall) med skitt og slitasje, og eksportert til
assets/models/site/. Kjør:

    python3 tools/models/site.py [container|excavator|mixer|pallet|fence|wheelbarrow|toilet|crane ...] [størrelse]

Origo er i bakken under midten, fronten mot +X (Blender: Y venstre, Z
opp). Gjerdet står langs X med nettet i XZ-planet; nettet er en plate med
en tegnet maske («mesh», alfatestes i motoren)."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import bake_textured, box, clear, cylinder, ellipsoid, material, new_object, rounded_box, smooth, spin, tube, weather  # noqa: E402
from mathutils import Vector  # noqa: E402

OUT = "assets/models/site"


def mats():
    m = {
        "blue": material("blue", (0.05, 0.16, 0.4), 0.55),
        "yellow": material("yellow", (0.85, 0.55, 0.04), 0.45),
        "orange": material("orange", (0.8, 0.28, 0.04), 0.5),
        "green": material("green", (0.1, 0.25, 0.12), 0.55),
        "plastic_blue": material("plastic_blue", (0.08, 0.3, 0.6), 0.5),
        "plastic_grey": material("plastic_grey", (0.6, 0.62, 0.6), 0.55),
        "dark": material("dark", (0.06, 0.06, 0.065), 0.6),
        "steel": material("steel", (0.5, 0.52, 0.54), 0.4, 1.0),
        "galv": material("galv", (0.6, 0.62, 0.63), 0.5, 1.0),
        "chrome": material("chrome", (0.85, 0.86, 0.88), 0.2, 1.0),
        "rubber": material("rubber", (0.04, 0.04, 0.04), 0.9),
        "concrete": material("concrete", (0.5, 0.49, 0.46), 0.95),
        "block": material("block", (0.58, 0.56, 0.52), 0.95),
        "wood": material("wood", (0.55, 0.42, 0.26), 0.7),
        "glass": material("glass", (0.05, 0.08, 0.1), 0.05),
        "strap": material("strap", (0.9, 0.9, 0.85), 0.5),
        "white": material("white", (0.85, 0.85, 0.82), 0.5),
        "red": material("red", (0.7, 0.08, 0.05), 0.5),
    }
    # Skitt nederst på alt som står ute.
    for name, (lo, hi, s) in {"blue": (0.0, 1.0, 0.5), "yellow": (0.0, 1.2, 0.45), "orange": (0.0, 0.8, 0.5), "green": (0.0, 0.5, 0.5), "plastic_blue": (0.0, 0.8, 0.35), "plastic_grey": (0.0, 0.5, 0.3), "concrete": (0.0, 0.4, 0.35), "block": (0.0, 0.3, 0.25), "wood": (0.0, 0.3, 0.3), "white": (0.0, 0.5, 0.3), "galv": (0.0, 0.5, 0.3)}.items():
        weather(m[name], lo, hi, s)
    return m


def textured_quad(name, corners, texture, mat_name):
    """En firkant med tekstur og alfa (nettet i gjerdet), to-sidig."""
    bm = bmesh.new()
    uv_layer = bm.loops.layers.uv.new("uv")
    uvs = [(0, 0), (1, 0), (1, 1), (0, 1)]
    for order in (corners, list(reversed(corners))):
        verts = [bm.verts.new(c) for c in order]
        f = bm.faces.new(verts)
        for loop, uv in zip(f.loops, uvs if order is corners else list(reversed(uvs))):
            loop[uv_layer].uv = uv
    mat = bpy.data.materials.new(mat_name)
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = bpy.data.images.load(os.path.abspath(texture))
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    nt.links.new(tex.outputs["Alpha"], bsdf.inputs["Alpha"])
    bsdf.inputs["Roughness"].default_value = 0.5
    bsdf.inputs["Metallic"].default_value = 1.0
    try:
        mat.blend_method = "CLIP"
    except Exception:
        pass
    return new_object(name, bm, [mat])


def mesh_texture(path):
    """Tegner nettet i byggegjerdet: ståltråd i et 50 × 200 mm-mønster
    over 3,5 × 2,0 m, med gjennomsiktig luft imellom."""
    from PIL import Image, ImageDraw
    w, h = 1792, 1024
    img = Image.new("RGBA", (w, h), (120, 124, 128, 0))
    d = ImageDraw.Draw(img)
    # 70 celler på 50 mm i x, 10 på 200 mm i z.
    for i in range(71):
        x = round(i * (w - 1) / 70)
        d.line([(x, 0), (x, h)], fill=(150, 154, 158, 255), width=3)
    for j in range(11):
        y = round(j * (h - 1) / 10)
        d.line([(0, y), (w, y)], fill=(150, 154, 158, 255), width=3)
    img.save(path)


# ------------------------------------------------------------------ modellene


def container(m):
    """Brakke: 6,06 × 2,44 × 2,59 m, korrugerte vegger, hjørnekasser,
    dør i enden med låsestenger, og vindu og dør i langsida."""
    L, W, H = 6.06, 2.44, 2.59
    blue = m["blue"]
    rounded_box("shell", (0.0, 0.0, H / 2 + 0.05), (L - 0.1, W - 0.1, H - 0.1), blue, 0.02)
    # Korrugering: vertikale ribber på langsidene og bakenden.
    n = int((L - 0.4) / 0.28)
    for i in range(n + 1):
        x = -L / 2 + 0.2 + i * (L - 0.4) / n
        for sy in (1, -1):
            box(f"rib{i}{sy}", (x, sy * (W / 2 - 0.03), H / 2 + 0.05), (0.07, 0.035, H - 0.4), blue)
    for j in range(7):
        y = -W / 2 + 0.3 + j * (W - 0.6) / 6
        box(f"ribb{j}", (-L / 2 + 0.03, y, H / 2 + 0.05), (0.035, 0.07, H - 0.4), blue)
    # Rammen: hjørnestolper og kantbjelker.
    for sx in (1, -1):
        for sy in (1, -1):
            box(f"corner{sx}{sy}", (sx * (L / 2 - 0.05), sy * (W / 2 - 0.05), H / 2 + 0.05), (0.1, 0.1, H), m["dark"])
    for sz, z in ((0, 0.1), (1, H)):
        for sy in (1, -1):
            box(f"rail{sz}{sy}", (0.0, sy * (W / 2 - 0.04), z), (L, 0.08, 0.1), m["dark"])
        for sx in (1, -1):
            box(f"railx{sz}{sx}", (sx * (L / 2 - 0.04), 0.0, z), (0.08, W, 0.1), m["dark"])
    # Dørene i enden (+X): to blad med fire låsestenger.
    for sy in (1, -1):
        rounded_box(f"door{sy}", (L / 2 - 0.02, sy * (W / 4 - 0.02), H / 2 + 0.05), (0.04, W / 2 - 0.16, H - 0.3), blue, 0.01)
        for k in (0.35, 0.8):
            y = sy * k
            tube(f"bar{sy}{k}", (L / 2 + 0.03, y, 0.3), (L / 2 + 0.03, y, H - 0.2), 0.018, m["galv"])
            box(f"handle{sy}{k}", (L / 2 + 0.06, y + 0.1 * sy, 1.1), (0.03, 0.2, 0.03), m["galv"])
    # Vindu og dør i langsida (−Y), som en brakke.
    box("win_frame", (1.0, -W / 2 - 0.01, 1.6), (1.1, 0.04, 0.8), m["white"])
    box("win_glass", (1.0, -W / 2 - 0.02, 1.6), (1.0, 0.02, 0.7), m["glass"])
    box("sdoor_frame", (-1.2, -W / 2 - 0.01, 1.05), (0.9, 0.04, 2.0), m["white"])
    box("sdoor", (-1.2, -W / 2 - 0.02, 1.05), (0.8, 0.02, 1.9), m["blue"])
    ellipsoid("knob", (-0.9, -W / 2 - 0.05, 1.0), (0.05, 0.05, 0.05), m["galv"], 8, 5)
    # Trapp foran døra.
    box("step", (-1.2, -W / 2 - 0.25, 0.1), (0.9, 0.4, 0.2), m["galv"])
    # Føttene: klosser i hjørnene.
    for sx in (1, -1):
        for sy in (1, -1):
            box(f"foot{sx}{sy}", (sx * (L / 2 - 0.3), sy * (W / 2 - 0.3), 0.025), (0.3, 0.3, 0.05), m["wood"])


def excavator(m):
    """Minigraver, 3,6 m lang: belter, overvogn med motvekt, hytte med
    glass, bom, stikke og skuffe med hydraulikksylindre."""
    y, dark, rub = m["yellow"], m["dark"], m["rubber"]
    # Beltene: løp i gummi med sko, drivhjul og bærehjul.
    for sy in (1, -1):
        cy = sy * 0.62
        rounded_box(f"track{sy}", (0.0, cy, 0.32), (2.3, 0.32, 0.56), rub, 0.22, segments=6)
        rounded_box(f"frame{sy}", (0.0, cy, 0.32), (1.9, 0.2, 0.3), y, 0.03)
        for k in range(13):
            x = -1.05 + k * 0.175
            box(f"shoe{sy}{k}", (x, cy, 0.62), (0.07, 0.34, 0.03), dark)
            box(f"shoe_b{sy}{k}", (x, cy, 0.025), (0.07, 0.34, 0.03), dark)
        for x in (-0.95, 0.95):
            cylinder(f"wheel{sy}{x}", (x, cy + sy * 0.17, 0.32), 0.22, 0.03, y, rot=(math.pi / 2, 0, 0), vertices=16)
    box("carbody", (0.0, 0.0, 0.5), (1.6, 0.9, 0.35), y)
    cylinder("slew", (0.0, 0.0, 0.7), 0.5, 0.1, dark, vertices=24)
    # Overvogna: motor bak, hytte til venstre, bomfeste foran til høyre.
    rounded_box("house", (-0.45, 0.0, 1.15), (1.5, 1.45, 0.8), y, 0.06, segments=3)
    rounded_box("counterweight", (-1.25, 0.0, 1.0), (0.4, 1.45, 0.7), y, 0.08, segments=3)
    box("grille", (-1.46, 0.0, 1.0), (0.02, 0.9, 0.4), dark)
    rounded_box("cab", (0.2, 0.4, 1.7), (1.05, 0.85, 1.6), y, 0.05, segments=3)
    for name, c, s in (("glass_f", (0.74, 0.4, 1.75), (0.02, 0.75, 1.1)), ("glass_l", (0.2, 0.84, 1.8), (0.9, 0.02, 1.0)), ("glass_r", (0.2, -0.04, 1.8), (0.9, 0.02, 1.0)), ("glass_b", (-0.33, 0.4, 1.8), (0.02, 0.75, 1.0))):
        box(name, c, s, m["glass"])
    box("roof", (0.2, 0.4, 2.52), (1.1, 0.9, 0.06), y)
    box("exhaust_cap", (-0.9, -0.3, 1.58), (0.2, 0.2, 0.06), dark)
    tube("exhaust", (-0.9, -0.3, 1.5), (-0.9, -0.3, 1.75), 0.03, dark)
    # Bommen fra festet foran til høyre, opp og fram; stikka ned; skuffa.
    base = Vector((0.55, -0.3, 1.05))
    knee = Vector((1.6, -0.3, 2.3))
    elbow = Vector((2.7, -0.3, 1.5))
    tip = Vector((3.2, -0.3, 0.55))

    def beam(name, a, b, w, h):
        d = b - a
        rot = d.to_track_quat("X", "Z").to_euler()
        rounded_box(name, (a + b) / 2, (d.length, w, h), y, 0.02, rot=rot)

    beam("boom", base, knee, 0.22, 0.3)
    beam("boom_b", knee, elbow, 0.2, 0.26)
    beam("arm", elbow, tip, 0.14, 0.18)
    cylinder("boom_pin", base, 0.09, 0.4, dark, rot=(math.pi / 2, 0, 0), vertices=12)
    # Hydraulikk: sylinder (lakkert) og stempelstang (krom).
    for name, a, b in (("cyl_boom", Vector((0.9, -0.3, 0.9)), Vector((1.5, -0.3, 1.9))), ("cyl_arm", Vector((1.3, -0.3, 2.42)), Vector((2.3, -0.3, 2.0))), ("cyl_bucket", Vector((2.4, -0.3, 1.9)), Vector((2.95, -0.3, 0.95)))):
        mid = a + (b - a) * 0.55
        tube(name, a, mid, 0.05, y)
        tube(name + "_rod", mid, b, 0.025, m["chrome"])
    # Skuffa: en kasse med tenner, åpen mot fram.
    rounded_box("bucket", (3.35, -0.3, 0.3), (0.5, 0.55, 0.45), dark, 0.03)
    box("bucket_lip", (3.6, -0.3, 0.12), (0.08, 0.57, 0.04), m["steel"])
    for k in range(4):
        box(f"tooth{k}", (3.68, -0.3 - 0.21 + k * 0.14, 0.1), (0.12, 0.04, 0.04), m["steel"])
    # Skjær foran.
    rounded_box("blade", (1.4, 0.0, 0.25), (0.08, 1.6, 0.4), y, 0.02)
    for sy in (1, -1):
        box(f"blade_arm{sy}", (0.9, sy * 0.45, 0.3), (1.0, 0.08, 0.08), y)


def mixer(m):
    """Sementblander: tromle på stativ med to hjul, motorkasse og
    vippehjul."""
    o = m["orange"]
    # Tromla, dreid og tiltet 35° bakover.
    drum = spin("drum", [(0.0, 0.0), (0.22, 0.0), (0.36, 0.25), (0.36, 0.55), (0.3, 0.72), (0.24, 0.78), (0.22, 0.78), (0.28, 0.72), (0.33, 0.55), (0.33, 0.25), (0.2, 0.04), (0.0, 0.04)], o, 28)
    drum.location = (0.05, 0.0, 0.62)
    drum.rotation_euler = (0, math.radians(-35), 0)
    for k in range(3):
        a = k * 2 * math.pi / 3
        b = box(f"fin{k}", (0.05 + math.sin(math.radians(35)) * 0.3 * 0, 0.0, 0.62), (0.03, 0.08, 0.5), o)
        b.rotation_euler = (0, math.radians(-35), a)
    # Stativet: to A-ben, aksel, to hjul og håndtak.
    for sy in (1, -1):
        tube(f"leg_f{sy}", (0.3, sy * 0.3, 0.0), (0.0, sy * 0.1, 0.55), 0.02, o)
        tube(f"leg_b{sy}", (-0.3, sy * 0.35, 0.25), (0.0, sy * 0.1, 0.55), 0.02, o)
        tube(f"handle{sy}", (-0.3, sy * 0.35, 0.25), (-0.6, sy * 0.35, 0.65), 0.018, o)
    tube("axle", (-0.3, 0.4, 0.25), (-0.3, -0.4, 0.25), 0.02, m["steel"])
    tube("cross", (0.3, 0.3, 0.0), (0.3, -0.3, 0.0), 0.02, o)
    tube("grip", (-0.6, 0.35, 0.65), (-0.6, -0.35, 0.65), 0.018, m["dark"])
    for sy in (1, -1):
        cylinder(f"wheel{sy}", (-0.3, sy * 0.42, 0.25), 0.25, 0.08, m["rubber"], rot=(math.pi / 2, 0, 0), vertices=20)
        cylinder(f"rim{sy}", (-0.3, sy * 0.42, 0.25), 0.14, 0.09, m["galv"], rot=(math.pi / 2, 0, 0), vertices=16)
    tube("pivot", (0.0, 0.3, 0.55), (0.0, -0.3, 0.55), 0.025, m["steel"])
    rounded_box("motor", (-0.2, 0.0, 0.45), (0.3, 0.26, 0.24), m["dark"], 0.02)
    cylinder("tipwheel", (0.0, -0.36, 0.55), 0.18, 0.03, m["dark"], rot=(math.pi / 2, 0, 0), vertices=16)
    for k in range(4):
        a = k * math.pi / 4
        box(f"spoke{k}", (0.0, -0.36, 0.55), (0.34, 0.015, 0.015), m["dark"], rot=(0, a, 0))


def pallet(m):
    """Europall med to lag lecablokker, strappet."""
    w = m["wood"]
    for y in (-0.33, 0.0, 0.33):
        box(f"bottom{y}", (0.0, y, 0.011), (1.2, 0.1, 0.022), w)
    for x in (-0.5, 0.0, 0.5):
        for y in (-0.33, 0.0, 0.33):
            box(f"block{x}{y}", (x, y, 0.07), (0.145, 0.1, 0.078), w)
    for y in (-0.33, 0.0, 0.33):
        box(f"stringer{y}", (0.0, y, 0.12), (1.2, 0.1, 0.022), w)
    for k, x in enumerate((-0.5, -0.25, 0.0, 0.25, 0.5)):
        box(f"top{k}", (x, 0.0, 0.142), (0.145 if k % 2 == 0 else 0.1, 0.8, 0.022), w)
    # Lecablokkene: 50 × 25 × 20 cm med to hull, tre lag.
    for layer in range(3):
        for i in range(2):
            for j in range(3):
                x = -0.3 + i * 0.6 + (0.0 if layer % 2 == 0 else 0.0)
                y = -0.26 + j * 0.26
                z = 0.153 + 0.1 + layer * 0.2
                if layer == 2 and (i, j) == (1, 2):
                    continue
                b = rounded_box(f"leca{layer}{i}{j}", (x, y, z), (0.5, 0.25, 0.2), m["block"], 0.008)
                for hx in (-0.12, 0.12):
                    bpy.ops.mesh.primitive_cube_add(location=(x + hx, y, z + 0.02))
                    cut = bpy.context.active_object
                    cut.scale = (0.08, 0.075, 0.1)
                    mod = b.modifiers.new("hole", "BOOLEAN")
                    mod.operation = "DIFFERENCE"
                    mod.object = cut
                    mod.solver = "EXACT"
                    bpy.context.view_layer.objects.active = b
                    with bpy.context.temp_override(object=b):
                        bpy.ops.object.modifier_apply(modifier=mod.name)
                    bpy.data.objects.remove(cut)
    for x in (-0.35, 0.35):
        box(f"strap{x}", (x, 0.0, 0.46), (0.016, 0.82, 0.62), m["strap"])
        box(f"strap_t{x}", (x, 0.0, 0.77), (0.016, 0.82, 0.004), m["strap"])


def fence(m):
    """Byggegjerde: 3,5 × 2,0 m panel med nett i galvanisert ramme på
    to betongføtter, pluss klemma mellom panelene."""
    g = m["galv"]
    L, H = 3.5, 2.0
    z0 = 0.15
    for sx in (1, -1):
        tube(f"post{sx}", (sx * (L / 2 - 0.02), 0.0, z0), (sx * (L / 2 - 0.02), 0.0, z0 + H), 0.021, g)
    tube("top", (-L / 2, 0.0, z0 + H - 0.02), (L / 2, 0.0, z0 + H - 0.02), 0.021, g)
    tube("bottom", (-L / 2, 0.0, z0 + 0.02), (L / 2, 0.0, z0 + 0.02), 0.021, g)
    tube("mid", (-L / 2, 0.0, z0 + H * 0.5), (L / 2, 0.0, z0 + H * 0.5), 0.016, g)
    for sx in (1, -1):
        rounded_box(f"foot{sx}", (sx * (L / 2 - 0.02), 0.0, 0.075), (0.75, 0.22, 0.15), m["concrete"], 0.015)
        box(f"clamp{sx}", (sx * (L / 2 - 0.02), 0.0, z0 + H * 0.6), (0.08, 0.1, 0.12), g)
    tex = f"{OUT}/fence_mesh.png"
    mesh_texture(tex)
    textured_quad("mesh", [(-L / 2, 0.0, z0 + 0.03), (L / 2, 0.0, z0 + 0.03), (L / 2, 0.0, z0 + H - 0.03), (-L / 2, 0.0, z0 + H - 0.03)], tex, "mesh")


def wheelbarrow(m):
    """Trillebår med lastekasse i stål, ett hjul og to bein."""
    g = m["green"]
    bm = bmesh.new()
    # Kassa: bunn mindre enn toppen, åpen oppe, med tykkelse via doble flater.
    top = [(-0.45, -0.34), (0.5, -0.3), (0.5, 0.3), (-0.45, 0.34)]
    bot = [(-0.3, -0.22), (0.35, -0.2), (0.35, 0.2), (-0.3, 0.22)]
    zt, zb = 0.62, 0.36
    outer_t = [bm.verts.new((x, y, zt)) for x, y in top]
    outer_b = [bm.verts.new((x, y, zb)) for x, y in bot]
    for k in range(4):
        bm.faces.new((outer_b[k], outer_b[(k + 1) % 4], outer_t[(k + 1) % 4], outer_t[k]))
    bm.faces.new(list(reversed(outer_b)))
    inner_t = [bm.verts.new((x * 0.96, y * 0.94, zt)) for x, y in top]
    inner_b = [bm.verts.new((x * 0.96, y * 0.94, zb + 0.02)) for x, y in bot]
    for k in range(4):
        bm.faces.new((inner_t[k], inner_t[(k + 1) % 4], inner_b[(k + 1) % 4], inner_b[k]))
        bm.faces.new((outer_t[k], outer_t[(k + 1) % 4], inner_t[(k + 1) % 4], inner_t[k]))
    bm.faces.new(inner_b)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    tray = new_object("tray", bm, [g])
    smooth(tray, 30)
    for sy in (1, -1):
        tube(f"frame{sy}", (0.55, sy * 0.2, 0.3), (-0.75, sy * 0.25, 0.62), 0.016, g)
        tube(f"leg{sy}", (-0.35, sy * 0.24, 0.52), (-0.4, sy * 0.26, 0.0), 0.016, g)
        tube(f"grip{sy}", (-0.75, sy * 0.25, 0.62), (-0.95, sy * 0.25, 0.62), 0.018, m["dark"])
    tube("cross", (-0.4, 0.26, 0.0), (-0.4, -0.26, 0.0), 0.016, g)
    cylinder("tyre", (0.55, 0.0, 0.19), 0.19, 0.09, m["rubber"], rot=(math.pi / 2, 0, 0), vertices=20)
    cylinder("rim", (0.55, 0.0, 0.19), 0.1, 0.1, m["galv"], rot=(math.pi / 2, 0, 0), vertices=14)
    tube("axle", (0.55, 0.22, 0.19), (0.55, -0.22, 0.19), 0.012, m["steel"])
    # Litt grus i kassa.
    ellipsoid("load", (0.05, 0.0, 0.58), (0.7, 0.45, 0.14), m["concrete"], 12, 6)


def toilet(m):
    """Byggetoalett: 1,1 × 1,1 × 2,3 m i blå plast med hvitt tak, dør med
    låsindikator, ventilasjonsspalter."""
    pb = m["plastic_blue"]
    rounded_box("body", (0.0, 0.0, 1.1), (1.1, 1.1, 2.2), pb, 0.04, segments=3)
    rounded_box("roof", (0.0, 0.0, 2.24), (1.16, 1.16, 0.12), m["white"], 0.04, segments=3)
    # Døra på +X med panelfelt og håndtak.
    rounded_box("door", (0.555, 0.0, 1.05), (0.03, 0.8, 1.95), pb, 0.01)
    box("door_panel", (0.575, 0.0, 1.25), (0.01, 0.6, 1.1), m["plastic_grey"])
    box("handle", (0.59, -0.28, 1.0), (0.03, 0.03, 0.14), m["plastic_grey"])
    box("lock", (0.59, -0.28, 1.25), (0.02, 0.05, 0.05), m["red"])
    for k in range(6):
        box(f"vent{k}", (0.57, 0.0, 1.95 + k * 0.035), (0.01, 0.5, 0.012), m["plastic_grey"])
    for sy in (1, -1):
        for k in range(6):
            box(f"svent{sy}{k}", (0.0, sy * 0.56, 1.9 + k * 0.04), (0.5, 0.01, 0.012), m["plastic_grey"])
    box("base", (0.0, 0.0, 0.05), (1.2, 1.2, 0.1), m["plastic_grey"])


def crane(m):
    """Tårnkran: ballastfot i betong, 22 m gittermast, svingkrans med
    førerhus, 24 m utligger med løpekatt og krok, 8 m motutligger med
    motvekter, og tårntopp med strekkstag."""
    y, con = m["yellow"], m["concrete"]
    # Foten: fire betongblokker over kryssbjelker.
    box("base_frame", (0.0, 0.0, 0.25), (4.2, 0.5, 0.5), y)
    box("base_frame2", (0.0, 0.0, 0.25), (0.5, 4.2, 0.5), y)
    for sx in (1, -1):
        for sy in (1, -1):
            rounded_box(f"ballast{sx}{sy}", (sx * 1.5, sy * 1.5, 0.55), (2.0, 2.0, 0.5), con, 0.03)
    # Masta: 1,6 m kvadrat, hjørnerør og fagverk i bøyler på 1,6 m.
    s = 0.8
    H = 22.0
    for sx in (1, -1):
        for sy in (1, -1):
            tube(f"chord{sx}{sy}", (sx * s, sy * s, 0.5), (sx * s, sy * s, H), 0.06, y, 8)
    bays = int(H / 1.6)
    for k in range(bays + 1):
        z = 0.5 + k * (H - 0.5) / bays
        for (a, b) in (((s, s), (s, -s)), ((s, -s), (-s, -s)), ((-s, -s), (-s, s)), ((-s, s), (s, s))):
            tube(f"rung{k}{a}", (a[0], a[1], z), (b[0], b[1], z), 0.03, y, 6)
            if k < bays:
                z2 = 0.5 + (k + 1) * (H - 0.5) / bays
                if k % 2 == 0:
                    tube(f"diag{k}{a}", (a[0], a[1], z), (b[0], b[1], z2), 0.028, y, 6)
                else:
                    tube(f"diag{k}{a}", (b[0], b[1], z), (a[0], a[1], z2), 0.028, y, 6)
    # Svingkrans, plattform og førerhus.
    cylinder("slew", (0.0, 0.0, H + 0.15), 1.0, 0.3, m["dark"], vertices=24)
    box("platform", (0.0, 0.0, H + 0.4), (2.6, 2.4, 0.2), y)
    rounded_box("cab", (1.4, -1.0, H + 1.4), (1.3, 1.1, 1.8), y, 0.04)
    box("cab_glass_f", (2.06, -1.0, H + 1.5), (0.02, 0.9, 1.3), m["glass"])
    box("cab_glass_s", (1.4, -1.56, H + 1.5), (1.1, 0.02, 1.3), m["glass"])
    # Tårntoppen: A-ramme 6 m over plattformen.
    top = Vector((0.0, 0.0, H + 6.5))
    for sx in (1, -1):
        for sy in (1, -1):
            tube(f"aframe{sx}{sy}", (sx * 0.7, sy * 0.7, H + 0.5), top, 0.05, y, 8)
    # Utliggeren mot +X: to underliggende bjelker, én over, fagverk.
    J = 24.0
    zj = H + 0.6
    for sy in (1, -1):
        tube(f"jib_lo{sy}", (0.8, sy * 0.6, zj), (J, sy * 0.6, zj), 0.05, y, 8)
    tube("jib_hi", (0.8, 0.0, zj + 1.1), (J - 1.0, 0.0, zj + 1.1), 0.05, y, 8)
    n = int(J / 1.5)
    for k in range(n):
        x = 0.8 + k * (J - 1.8) / n
        x2 = 0.8 + (k + 1) * (J - 1.8) / n
        tube(f"jib_r{k}", (x, 0.6, zj), (x, -0.6, zj), 0.025, y, 6)
        for sy in (1, -1):
            tube(f"jib_d{k}{sy}", (x, sy * 0.6, zj), (x2, 0.0, zj + 1.1), 0.025, y, 6)
    # Motutliggeren mot −X med motvekter.
    C = 8.0
    for sy in (1, -1):
        tube(f"cj_lo{sy}", (-0.8, sy * 0.8, zj), (-C, sy * 0.8, zj), 0.05, y, 8)
    for k in range(5):
        x = -0.8 - k * (C - 0.8) / 5
        tube(f"cj_r{k}", (x, 0.8, zj), (x, -0.8, zj), 0.025, y, 6)
    box("cj_deck", (-C / 2 - 0.4, 0.0, zj + 0.05), (C - 0.8, 1.8, 0.08), m["dark"])
    for k in range(3):
        rounded_box(f"weight{k}", (-C + 0.9 + k * 0.45, 0.0, zj + 0.95), (0.4, 1.9, 1.7), con, 0.03)
    # Strekkstagene fra toppen.
    tube("stay_j1", top, (J * 0.5, 0.0, zj + 1.1), 0.02, m["steel"], 6)
    tube("stay_j2", top, (J - 1.2, 0.0, zj + 1.1), 0.02, m["steel"], 6)
    tube("stay_c", top, (-C + 1.0, 0.0, zj + 0.1), 0.02, m["steel"], 6)
    # Løpekatta og kroken med vaier ned.
    tx = 11.0
    box("trolley", (tx, 0.0, zj - 0.2), (0.8, 1.5, 0.35), m["dark"])
    tube("rope", (tx, 0.0, zj - 0.35), (tx, 0.0, zj - 10.5), 0.012, m["dark"], 6)
    rounded_box("hookblock", (tx, 0.0, zj - 10.9), (0.3, 0.5, 0.7), y, 0.03)
    tube("hook", (tx, 0.0, zj - 11.25), (tx + 0.15, 0.0, zj - 11.6), 0.04, m["steel"], 8)
    # Rekkverk på utliggeren.
    for sy in (1, -1):
        tube(f"rail{sy}", (0.8, sy * 0.6, zj + 0.6), (J - 1.0, sy * 0.6, zj + 0.6), 0.015, y, 6)


BUILDERS = {
    "container": (container, 1024),
    "excavator": (excavator, 1024),
    "mixer": (mixer, 512),
    "pallet": (pallet, 512),
    "fence": (fence, 512),
    "wheelbarrow": (wheelbarrow, 512),
    "toilet": (toilet, 512),
    "crane": (crane, 1024),
}

if __name__ == "__main__":
    args = sys.argv[1:]
    kinds = [a for a in args if a in BUILDERS] or list(BUILDERS)
    size = next((int(a) for a in args if a.isdigit()), None)
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        build, default_size = BUILDERS[k]
        build(mats())
        bake_textured(f"{OUT}/{k}.glb", size or default_size)
