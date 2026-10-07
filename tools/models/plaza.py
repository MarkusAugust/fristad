"""Torget i Fristad: kafébord, kafestol, parasoll, torgbod og postkasse,
modellert i Blender (bpy) og eksportert til assets/models/plaza/. Kjør:

    python3 tools/models/plaza.py [cafe_table|cafe_chair|parasol|market_stall|mailbox ...]

Stolen vender mot +X (ryggen i −X). Boden har disken mot +Z, som den
ekstruderte hadde. Postkassa har åpningen mot +Z; teksten «POST» legges på
i motoren."""

import math
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import box, clear, cylinder, ellipsoid, export, material, rounded_box, smooth, spin, tube  # noqa: E402

OUT = "assets/models/plaza"


def mats():
    return {
        "green": material("green", (0.06, 0.16, 0.12), 0.45),
        "oak": material("oak", (0.42, 0.27, 0.14), 0.6),
        "oak_light": material("oak_light", (0.55, 0.38, 0.2), 0.65),
        "canvas": material("canvas", (0.85, 0.78, 0.62), 0.8),
        "canvas_dark": material("canvas_dark", (0.68, 0.6, 0.45), 0.8),
        "stripe_red": material("stripe_red", (0.55, 0.06, 0.06), 0.75),
        "stripe_white": material("stripe_white", (0.9, 0.87, 0.8), 0.75),
        "iron": material("iron", (0.045, 0.045, 0.05), 0.55),
        "red": material("red", (0.6, 0.02, 0.05), 0.3),
        "dark": material("dark", (0.1, 0.1, 0.11), 0.6),
        "apple": material("apple", (0.6, 0.05, 0.04), 0.4),
        "pear": material("pear", (0.4, 0.55, 0.1), 0.5),
        "orange": material("orange", (0.85, 0.3, 0.03), 0.5),
        "lemon": material("lemon", (0.8, 0.6, 0.05), 0.5),
    }


def cafe_table(m):
    """Bistrobord: støpt fot, søyle, rund plate i eik."""
    spin("base", [(0.0, 0.0), (0.24, 0.0), (0.24, 0.015), (0.2, 0.03), (0.1, 0.05), (0.04, 0.07), (0.025, 0.1), (0.022, 0.68), (0.06, 0.7), (0.06, 0.72), (0.0, 0.72)], m["green"], 20)
    top = cylinder("top", (0.0, 0.0, 0.735), 0.34, 0.03, m["oak"], vertices=28)
    mod = top.modifiers.new("bevel", "BEVEL")
    mod.width = 0.008
    mod.segments = 2
    from common import apply_modifiers
    apply_modifiers(top)
    smooth(top, 50)
    cylinder("rim", (0.0, 0.0, 0.735), 0.345, 0.012, m["green"], vertices=28)


def cafe_chair(m):
    """Bistrostol i stål med eikesete og buet rygg, vendt mot +X."""
    g = m["green"]
    for sx, sy in ((0.16, 0.16), (0.16, -0.16), (-0.14, 0.16), (-0.14, -0.16)):
        tube(f"leg{sx:+.2f}{sy:+.2f}", (sx * 1.06, sy * 1.06, 0.0), (sx, sy, 0.44), 0.011, g)
    tube("stretch_f", (0.16, 0.16, 0.2), (0.16, -0.16, 0.2), 0.008, g)
    tube("stretch_b", (-0.14, 0.16, 0.2), (-0.14, -0.16, 0.2), 0.008, g)
    rounded_box("seat", (0.01, 0.0, 0.455), (0.38, 0.38, 0.025), m["oak"], 0.008)
    # Ryggen: to bøyde stolper og tre tverrspiler.
    for sy in (0.16, -0.16):
        tube(f"back_post{sy:+.2f}", (-0.17, sy, 0.44), (-0.22, sy, 0.9), 0.011, g)
    for z in (0.62, 0.74, 0.86):
        x = -0.17 - (z - 0.44) * 0.11
        tube(f"back_bar{z:.2f}", (x, 0.16, z), (x, -0.16, z), 0.009, g)
    rounded_box("back_top", (-0.225, 0.0, 0.92), (0.03, 0.36, 0.04), m["oak"], 0.008)


def parasol(m):
    """Parasoll: stang fra bordet opp, åtte paneler i vekslende lerret."""
    g = m["green"]
    tube("pole", (0.0, 0.0, 0.74), (0.0, 0.0, 2.45), 0.02, g)
    ellipsoid("tip", (0.0, 0.0, 2.47), (0.06, 0.06, 0.08), g, 10, 6)
    import bmesh
    from common import new_object
    bm = bmesh.new()
    apex = bm.verts.new((0.0, 0.0, 2.42))
    n = 8
    rim = []
    for k in range(n):
        a = k * 2 * math.pi / n
        rim.append(bm.verts.new((math.cos(a) * 1.15, math.sin(a) * 1.15, 2.0)))
    faces = []
    for k in range(n):
        # Hvert panel får en liten bue mellom ribbene via et midtpunkt.
        a = (k + 0.5) * 2 * math.pi / n
        mid = bm.verts.new((math.cos(a) * 1.12, math.sin(a) * 1.12, 1.95))
        f1 = bm.faces.new((apex, rim[k], mid))
        f2 = bm.faces.new((apex, mid, rim[(k + 1) % n]))
        f1.material_index = f2.material_index = k % 2
        faces += [f1, f2]
    # Undersiden som egne, speilvendte flater.
    for f in list(faces):
        verts = [bm.verts.new(v.co) for v in reversed(f.verts)]
        nf = bm.faces.new(verts)
        nf.material_index = f.material_index
    ob = new_object("canopy", bm, [m["canvas"], m["canvas_dark"]])
    for k in range(n):
        a = k * 2 * math.pi / n
        tube(f"rib{k}", (0.0, 0.0, 2.4), (math.cos(a) * 1.15, math.sin(a) * 1.15, 2.0), 0.008, g)
    return ob


def crate(name, x, m, fruit):
    rounded_box(name, (x, 0.0, 1.0), (0.6, 0.4, 0.18), m["oak_light"], 0.006)
    for i in range(12):
        fx = x - 0.22 + (i % 4) * 0.14
        fy = -0.12 + (i // 4) * 0.12
        fz = 1.12 + (0.03 if i % 3 == 0 else 0.0)
        ellipsoid(f"{name}_fruit{i}", (fx, fy, fz), (0.09, 0.09, 0.085), fruit[(i // 5) % len(fruit)], 10, 6)


def market_stall(m):
    """Torgbod: disk, bakvegg, fire stolper, skrå stripet seilduk, tre
    kasser med frukt. Disken vender mot +Y i Blender (+Z... nei: −Z i glTF).
    Den ekstruderte hadde fronten mot +Z i motoren, altså −Y her."""
    oak = m["oak"]
    rounded_box("counter", (0.0, 0.0, 0.9), (2.6, 1.2, 0.06), oak, 0.01)
    rounded_box("front", (0.0, -0.55, 0.5), (2.6, 0.04, 0.8), oak, 0.01)
    for sx in (-1.22, 1.22):
        for sy, h in ((0.55, 2.5), (-0.55, 2.2)):
            box(f"post{sx:+.1f}{sy:+.1f}", (sx, sy, h / 2), (0.08, 0.08, h), oak)
    box("beam_b", (0.0, 0.55, 2.46), (2.6, 0.08, 0.08), oak)
    box("beam_f", (0.0, -0.55, 2.16), (2.6, 0.08, 0.08), oak)
    # Seilduken: åtte striper fra bakre bjelke ned til litt utenfor fronten.
    for k in range(8):
        mat = m["stripe_red"] if k % 2 == 0 else m["stripe_white"]
        x = -1.4 + 0.4 * k
        box(f"stripe{k}", (x, -0.1, 2.4), (0.4, 1.8, 0.02), mat, rot=(0.2, 0, 0))
        # Takket kant foran.
        box(f"valance{k}", (x, -0.93, 2.14), (0.4, 0.02, 0.18), mat)
    for name, x, fruit in (("crate_a", -0.85, (m["apple"], m["pear"], m["orange"])), ("crate_b", -0.1, (m["orange"], m["lemon"], m["apple"])), ("crate_c", 0.65, (m["pear"], m["apple"], m["lemon"]))):
        crate(name, x, m, fruit)


def mailbox(m):
    """Rød postkasse på stolpe, 1,5 m, åpningen mot −Y (mot +Z i motoren)."""
    cylinder("post", (0.0, 0.0, 0.48), 0.04, 0.95, m["iron"], vertices=12)
    rounded_box("body", (0.0, 0.0, 1.2), (0.46, 0.34, 0.52), m["red"], 0.05, segments=4)
    rounded_box("slot_lip", (0.0, -0.175, 1.36), (0.3, 0.02, 0.03), m["dark"], 0.004)
    rounded_box("slot", (0.0, -0.176, 1.33), (0.28, 0.01, 0.015), m["dark"], 0.002)
    rounded_box("door", (0.0, 0.0, 0.995), (0.4, 0.3, 0.06), m["red"], 0.02)
    rounded_box("hinge", (0.17, -0.171, 1.1), (0.03, 0.01, 0.2), m["dark"], 0.003)


BUILDERS = {"cafe_table": cafe_table, "cafe_chair": cafe_chair, "parasol": parasol, "market_stall": market_stall, "mailbox": mailbox}

if __name__ == "__main__":
    kinds = [a for a in sys.argv[1:] if a in BUILDERS] or list(BUILDERS)
    import os
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        BUILDERS[k](mats())
        export(f"{OUT}/{k}.glb")
