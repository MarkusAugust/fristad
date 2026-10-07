"""Innboet i Gorveks hus: det en barbarkriger fra en film fra 80- og
90-tallet har hjemme. Modellert i Blender (bpy) og eksportert til
assets/models/gorvek/. Kjør:

    python3 tools/models/gorvek.py [table|bench_long|throne|bed|bearskin|shield|axe|sword|chest|keg|cauldron|torch|antlers|stool|skull ...]

Origo er på gulvet under midten (veggting har origo i festet på veggen,
med veggen i y = 0 og tingen ut mot −y, som blir +Z i motoren). Fronten
er +X. Materialet «glow» lyser om natta."""

import math
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import box, clear, cylinder, ellipsoid, export, material, rounded_box, smooth, spin, tube  # noqa: E402

OUT = "assets/models/gorvek"


def mats():
    return {
        "oak": material("oak", (0.36, 0.22, 0.1), 0.65),
        "dark_oak": material("dark_oak", (0.2, 0.12, 0.06), 0.7),
        "fur": material("fur", (0.16, 0.1, 0.06), 0.95),
        "fur_light": material("fur_light", (0.5, 0.4, 0.28), 0.95),
        "wool": material("wool", (0.45, 0.12, 0.1), 0.9),
        "iron": material("iron", (0.2, 0.2, 0.22), 0.45, 1.0),
        "steel": material("steel", (0.75, 0.76, 0.8), 0.25, 1.0),
        "bronze": material("bronze", (0.7, 0.5, 0.2), 0.4, 1.0),
        "leather": material("leather", (0.4, 0.25, 0.14), 0.8),
        "bone": material("bone", (0.85, 0.8, 0.68), 0.6),
        "horn": material("horn", (0.5, 0.42, 0.3), 0.55),
        "stone": material("stone", (0.4, 0.4, 0.38), 0.9),
        "glow": material("glow", (1.0, 0.55, 0.15), 0.3),
        "fire": material("fire", (1.0, 0.5, 0.12), 0.4),
        "rope": material("rope", (0.6, 0.5, 0.3), 0.9),
    }


def table(m):
    """Langbord av grove planker på to bukker."""
    oak, dark = m["oak"], m["dark_oak"]
    for k in range(3):
        rounded_box(f"plank{k}", (0.0, -0.3 + k * 0.3, 0.72), (2.4, 0.28, 0.06), oak, 0.008)
    for sx in (-1, 1):
        box(f"leg{sx}", (sx * 0.95, 0.0, 0.34), (0.1, 0.8, 0.68), dark)
        box(f"foot{sx}", (sx * 0.95, 0.0, 0.04), (0.14, 0.9, 0.08), dark)
    box("stretcher", (0.0, 0.0, 0.2), (1.9, 0.08, 0.1), dark)
    # Et par krus og et fat på bordet.
    for k, (x, y) in enumerate([(-0.7, 0.25), (0.3, -0.2), (0.9, 0.3)]):
        spin(f"mug{k}", [(0.0, 0.0), (0.06, 0.0), (0.06, 0.12), (0.05, 0.12), (0.05, 0.01), (0.0, 0.01)], m["dark_oak"], 12)
        bpy.context.active_object.location = (x, y, 0.75)
    spin("plate", [(0.0, 0.0), (0.18, 0.0), (0.2, 0.02), (0.0, 0.02)], m["dark_oak"], 16)
    bpy.context.active_object.location = (-0.1, 0.1, 0.75)
    ellipsoid("roast", (-0.1, 0.1, 0.82), (0.26, 0.18, 0.12), m["leather"], 10, 6)


def bench_long(m):
    oak, dark = m["oak"], m["dark_oak"]
    rounded_box("seat", (0.0, 0.0, 0.43), (2.2, 0.32, 0.06), oak, 0.008)
    for sx in (-1, 1):
        box(f"leg{sx}", (sx * 0.85, 0.0, 0.2), (0.08, 0.3, 0.4), dark)
    box("stretcher", (0.0, 0.0, 0.12), (1.7, 0.06, 0.08), dark)


def throne(m):
    """Høvdingstolen: høy rygg, gevir på toppen, skinn på setet."""
    oak, dark = m["oak"], m["dark_oak"]
    rounded_box("seat", (0.0, 0.0, 0.48), (0.7, 0.7, 0.08), oak, 0.01)
    for sx, sy in ((-1, -1), (1, -1), (-1, 1), (1, 1)):
        box(f"leg{sx}{sy}", (sx * 0.3, sy * 0.3, 0.22), (0.09, 0.09, 0.44), dark)
    for sy in (-1, 1):
        box(f"post{sy}", (-0.32, sy * 0.3, 0.9), (0.09, 0.09, 1.2), dark)
    rounded_box("back", (-0.32, 0.0, 1.0), (0.06, 0.6, 0.8), oak, 0.01)
    for sy in (-1, 1):
        rounded_box(f"arm{sy}", (0.0, sy * 0.33, 0.72), (0.6, 0.07, 0.06), oak, 0.01)
        box(f"armpost{sy}", (0.25, sy * 0.33, 0.6), (0.06, 0.06, 0.18), dark)
    ellipsoid("pelt", (0.02, 0.0, 0.54), (0.62, 0.62, 0.08), m["fur"], 12, 6)
    # Geviret på toppen av ryggen.
    for sy in (-1, 1):
        base = (-0.32, sy * 0.12, 1.5)
        tube(f"antler{sy}", base, (-0.32, sy * 0.4, 1.85), 0.02, m["horn"], 8)
        tube(f"tine{sy}a", (-0.32, sy * 0.26, 1.67), (-0.3, sy * 0.3, 1.95), 0.012, m["horn"], 6)
        tube(f"tine{sy}b", (-0.32, sy * 0.36, 1.8), (-0.34, sy * 0.52, 1.98), 0.012, m["horn"], 6)


def bed(m):
    """Brisk med madrass, ullteppe og skinn."""
    oak, dark = m["oak"], m["dark_oak"]
    box("frame", (0.0, 0.0, 0.3), (2.1, 1.2, 0.2), dark)
    for sx, sy in ((-1, -1), (1, -1), (-1, 1), (1, 1)):
        box(f"leg{sx}{sy}", (sx * 1.0, sy * 0.55, 0.12), (0.1, 0.1, 0.24), dark)
    box("headboard", (-1.05, 0.0, 0.65), (0.06, 1.2, 0.9), oak)
    rounded_box("mattress", (0.0, 0.0, 0.46), (2.0, 1.1, 0.14), m["rope"], 0.03)
    rounded_box("blanket", (0.25, 0.0, 0.55), (1.4, 1.14, 0.08), m["wool"], 0.03)
    ellipsoid("pelt", (0.3, 0.1, 0.6), (1.1, 0.9, 0.08), m["fur"], 12, 6)
    ellipsoid("pillow", (-0.75, 0.0, 0.58), (0.5, 0.7, 0.16), m["fur_light"], 10, 6)


def bearskin(m):
    """Bjørnefellen på gulvet: kropp, hode med åpen kjeft, fire labber."""
    fur = m["fur"]
    ellipsoid("body", (0.0, 0.0, 0.03), (1.5, 1.2, 0.07), fur, 16, 6)
    ellipsoid("head", (0.95, 0.0, 0.12), (0.5, 0.42, 0.24), fur, 12, 8)
    ellipsoid("snout", (1.18, 0.0, 0.1), (0.26, 0.22, 0.14), fur, 10, 6)
    ellipsoid("jaw", (1.12, 0.0, 0.02), (0.3, 0.2, 0.05), m["bone"], 8, 4)
    ellipsoid("nose", (1.3, 0.0, 0.13), (0.07, 0.07, 0.05), m["iron"], 8, 5)
    for sy in (-1, 1):
        ellipsoid(f"ear{sy}", (0.82, sy * 0.18, 0.26), (0.1, 0.1, 0.1), fur, 8, 5)
        ellipsoid(f"eye{sy}", (1.1, sy * 0.1, 0.17), (0.04, 0.04, 0.04), m["iron"], 6, 4)
        for k in range(3):
            box(f"tooth{sy}{k}", (1.05 + k * 0.08, sy * 0.08, 0.06), (0.02, 0.02, 0.05), m["bone"])
    for sx, sy in ((0.55, -0.7), (0.55, 0.7), (-0.6, -0.65), (-0.6, 0.65)):
        ellipsoid(f"leg{sx}{sy}", (sx, sy, 0.03), (0.5, 0.3, 0.06), fur, 10, 5)
        for k in range(3):
            ellipsoid(f"claw{sx}{sy}{k}", (sx + (0.22 if sx > 0 else -0.22), sy + (k - 1) * 0.08, 0.03), (0.08, 0.03, 0.03), m["bone"], 6, 4)


def shield(m):
    """Rundskjold på veggen: planker, jernkant og skjoldbule. Veggen i y = 0."""
    r = 0.45
    cylinder("board", (0.0, -0.03, 0.0), r, 0.03, m["oak"], rot=(math.pi / 2, 0, 0), vertices=24)
    spin("rim", [(r - 0.02, 0.0), (r + 0.02, 0.0), (r + 0.02, 0.05), (r - 0.02, 0.05)], m["iron"], 24)
    bpy.context.active_object.rotation_euler = (math.pi / 2, 0, 0)
    bpy.context.active_object.location = (0.0, -0.01, 0.0)
    ellipsoid("boss", (0.0, -0.06, 0.0), (0.2, 0.08, 0.2), m["iron"], 12, 8)
    for k in range(4):
        a = k * math.pi / 2 + math.pi / 4
        box(f"band{k}", (math.cos(a) * r * 0.55, -0.055, math.sin(a) * r * 0.55), (0.03, 0.01, r * 0.9), m["iron"], rot=(0, -a + math.pi / 2, 0))


def axe(m):
    """Tohåndsøks, 1,1 m, liggende langs X med skaftet, bladet i +Z."""
    tube("shaft", (-0.55, 0.0, 0.0), (0.55, 0.0, 0.0), 0.022, m["dark_oak"], 10)
    for s in (-1, 1):
        bpy.ops.mesh.primitive_cone_add(vertices=3, radius1=0.2, radius2=0.0, depth=0.015, location=(0.45, 0.0, s * 0.14), rotation=(math.pi / 2, 0, 0))
        blade = bpy.context.active_object
        blade.name = f"blade{s}"
        blade.scale = (1.0, 0.9, 1.0)
        blade.data.materials.append(m["steel"])
    box("eye", (0.45, 0.0, 0.0), (0.08, 0.05, 0.1), m["iron"])
    for k in range(5):
        tube(f"wrap{k}", (-0.5 + k * 0.05, 0.03, 0.0), (-0.48 + k * 0.05, -0.03, 0.0), 0.024, m["leather"], 6)


def sword(m):
    """Bredsverd, 1,2 m langs X med spissen i +X."""
    box("blade", (0.3, 0.0, 0.0), (0.9, 0.012, 0.09), m["steel"])
    bpy.ops.mesh.primitive_cone_add(vertices=4, radius1=0.045, radius2=0.0, depth=0.12, location=(0.81, 0.0, 0.0), rotation=(0, math.pi / 2, 0))
    tip = bpy.context.active_object
    tip.name = "tip"
    tip.scale = (0.3, 1.0, 1.0)
    tip.data.materials.append(m["steel"])
    box("guard", (-0.16, 0.0, 0.0), (0.03, 0.04, 0.3), m["bronze"])
    tube("grip", (-0.18, 0.0, 0.0), (-0.38, 0.0, 0.0), 0.02, m["leather"], 8)
    ellipsoid("pommel", (-0.41, 0.0, 0.0), (0.06, 0.06, 0.06), m["bronze"], 8, 6)


def chest(m):
    """Kiste med buet lokk og jernbeslag."""
    oak, iron = m["oak"], m["iron"]
    box("body", (0.0, 0.0, 0.3), (1.0, 0.56, 0.5), oak)
    cylinder("lid", (0.0, 0.0, 0.55), 0.28, 1.0, oak, rot=(0, math.pi / 2, 0), vertices=16)
    box("lid_cut", (0.0, 0.0, 0.42), (1.02, 0.6, 0.26), oak)
    for sx in (-0.35, 0.35):
        box(f"band{sx}", (sx, 0.0, 0.3), (0.05, 0.58, 0.51), iron)
        box(f"band_lid{sx}", (sx, 0.0, 0.7), (0.05, 0.4, 0.3), iron)
    box("lock", (0.0, -0.29, 0.45), (0.1, 0.02, 0.14), iron)
    for sx, sy in ((-1, -1), (1, -1), (-1, 1), (1, 1)):
        box(f"foot{sx}{sy}", (sx * 0.45, sy * 0.24, 0.03), (0.08, 0.08, 0.06), iron)


def keg(m):
    """Øltønne med jernbånd."""
    spin("staves", [(0.0, 0.0), (0.26, 0.0), (0.3, 0.3), (0.3, 0.5), (0.26, 0.8), (0.0, 0.8)], m["oak"], 18)
    for z in (0.08, 0.36, 0.44, 0.72):
        r = 0.27 if z in (0.08, 0.72) else 0.305
        spin(f"band{z}", [(r, z - 0.025), (r + 0.012, z - 0.025), (r + 0.012, z + 0.025), (r, z + 0.025)], m["iron"], 18)
    cylinder("tap", (0.3, 0.0, 0.2), 0.02, 0.1, m["bronze"], rot=(0, math.pi / 2, 0), vertices=8)


def cauldron(m):
    """Gryte på tre bein over ilden."""
    spin("pot", [(0.0, 0.2), (0.3, 0.2), (0.38, 0.4), (0.36, 0.6), (0.4, 0.62), (0.33, 0.62), (0.3, 0.58), (0.25, 0.4), (0.0, 0.3)], m["iron"], 18)
    for k in range(3):
        a = k * 2 * math.pi / 3
        tube(f"leg{k}", (math.cos(a) * 0.25, math.sin(a) * 0.25, 0.25), (math.cos(a) * 0.32, math.sin(a) * 0.32, 0.0), 0.018, m["iron"], 6)
    for s in (-1, 1):
        tube(f"ear{s}", (s * 0.38, 0.0, 0.6), (s * 0.44, 0.0, 0.72), 0.012, m["iron"], 6)
    tube("handle", (-0.44, 0.0, 0.72), (0.44, 0.0, 0.72), 0.012, m["iron"], 8)
    ellipsoid("stew", (0.0, 0.0, 0.57), (0.6, 0.6, 0.04), m["leather"], 12, 4)


def torch(m):
    """Veggfakkel: jernholder på veggen (y = 0), skaft ut og opp, flamme."""
    box("plate", (0.0, -0.01, 0.0), (0.08, 0.02, 0.16), m["iron"])
    tube("ring", (0.0, -0.02, 0.05), (0.0, -0.12, 0.1), 0.02, m["iron"], 8)
    tube("shaft", (0.0, -0.1, -0.2), (0.0, -0.16, 0.26), 0.022, m["dark_oak"], 8)
    ellipsoid("wrap", (0.0, -0.165, 0.26), (0.07, 0.07, 0.1), m["rope"], 8, 6)
    ellipsoid("flame", (0.0, -0.17, 0.38), (0.09, 0.09, 0.18), m["fire"], 8, 6)
    ellipsoid("flame_tip", (0.0, -0.17, 0.5), (0.04, 0.04, 0.1), m["fire"], 6, 5)


def antlers(m):
    """Elggevir på en plakett, veggen i y = 0."""
    rounded_box("plaque", (0.0, -0.02, 0.0), (0.36, 0.04, 0.28), m["dark_oak"], 0.01)
    ellipsoid("skull", (0.0, -0.08, -0.02), (0.14, 0.12, 0.2), m["bone"], 10, 6)
    for s in (-1, 1):
        base = (s * 0.06, -0.1, 0.06)
        tube(f"beam{s}", base, (s * 0.3, -0.2, 0.3), 0.025, m["horn"], 8)
        # Skovla: en flat ellipsoide ytterst, med takker.
        ellipsoid(f"palm{s}", (s * 0.48, -0.22, 0.42), (0.42, 0.05, 0.32), m["horn"], 10, 6)
        for k in range(4):
            x = s * (0.32 + k * 0.1)
            tube(f"tine{s}{k}", (x, -0.22, 0.5), (x + s * 0.02, -0.22, 0.64 + (k % 2) * 0.04), 0.012, m["horn"], 6)


def stool(m):
    cylinder("seat", (0.0, 0.0, 0.42), 0.18, 0.05, m["oak"], vertices=12)
    for k in range(3):
        a = k * 2 * math.pi / 3
        tube(f"leg{k}", (math.cos(a) * 0.12, math.sin(a) * 0.12, 0.4), (math.cos(a) * 0.17, math.sin(a) * 0.17, 0.0), 0.02, m["dark_oak"], 6)


def skull(m):
    """Et kranium på hylla."""
    ellipsoid("cranium", (0.0, 0.0, 0.11), (0.16, 0.14, 0.15), m["bone"], 12, 8)
    ellipsoid("jaw", (0.04, 0.0, 0.03), (0.12, 0.1, 0.06), m["bone"], 8, 5)
    for s in (-1, 1):
        ellipsoid(f"socket{s}", (0.07, s * 0.04, 0.11), (0.05, 0.04, 0.05), m["iron"], 6, 4)
    for s in (-1, 1):
        tube(f"horn{s}", (-0.02, s * 0.1, 0.18), (-0.05, s * 0.22, 0.32), 0.02, m["horn"], 6)


BUILDERS = {
    "table": table,
    "bench_long": bench_long,
    "throne": throne,
    "bed": bed,
    "bearskin": bearskin,
    "shield": shield,
    "axe": axe,
    "sword": sword,
    "chest": chest,
    "keg": keg,
    "cauldron": cauldron,
    "torch": torch,
    "antlers": antlers,
    "stool": stool,
    "skull": skull,
}

if __name__ == "__main__":
    kinds = [a for a in sys.argv[1:] if a in BUILDERS] or list(BUILDERS)
    import os
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        BUILDERS[k](mats())
        export(f"{OUT}/{k}.glb")
