"""Husdetaljene i Fristad: vinduer (2×2 og 2×3 ruter), dørlampe og
blomsterkasse, modellert i Blender (bpy) og eksportert til
assets/models/house/. Kjør:

    python3 tools/models/house.py [window_22|window_23|door_lamp|flower_box ...]

Vinduene står i XZ-planet med utsiden mot −Y (som blir +Z i motoren, ut av
veggen), origo nederst i midten av hullet. Materialet «glass» blir
kind::GLASS i lasteren, «glow» lyser om natta."""

import math
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import box, clear, cylinder, ellipsoid, export, material, rounded_box, smooth, tube  # noqa: E402

OUT = "assets/models/house"


def mats():
    return {
        "trim": material("trim", (0.85, 0.83, 0.78), 0.55),
        "glass": material("glass", (0.05, 0.07, 0.1), 0.05),
        "chrome": material("chrome", (0.8, 0.8, 0.82), 0.3, 1.0),
        "iron": material("iron", (0.045, 0.045, 0.05), 0.55),
        "glow": material("glow", (1.0, 0.88, 0.65), 0.2),
        "oak": material("oak", (0.4, 0.26, 0.13), 0.6),
        "soil": material("soil", (0.1, 0.07, 0.05), 0.95),
        "leaf": material("leaf", (0.12, 0.3, 0.08), 0.7),
        "red": material("red", (0.65, 0.05, 0.1), 0.6),
        "yellow": material("yellow", (0.9, 0.65, 0.05), 0.6),
        "white": material("white", (0.9, 0.88, 0.8), 0.6),
        "purple": material("purple", (0.35, 0.1, 0.5), 0.6),
        "pink": material("pink", (0.85, 0.3, 0.45), 0.6),
    }


def window(m, w, h, panes):
    """Karm 7 cm, ramme innenfor, sprosser, glass og et lite håndtak."""
    trim = m["trim"]
    fw, fd = 0.07, 0.1
    # Karmen: fire lister, bakkant i y = 0, utsiden mot −y.
    rounded_box("frame_l", (-w / 2 + fw / 2, -fd / 2, h / 2), (fw, fd, h), trim, 0.006)
    rounded_box("frame_r", (w / 2 - fw / 2, -fd / 2, h / 2), (fw, fd, h), trim, 0.006)
    rounded_box("frame_t", (0.0, -fd / 2, h - fw / 2), (w - 2 * fw, fd, fw), trim, 0.006)
    rounded_box("frame_b", (0.0, -fd / 2, fw / 2), (w - 2 * fw, fd, fw), trim, 0.006)
    # Ramma (vindusbladet) 3 cm innenfor karmen, litt tilbaketrukket.
    iw, ih = w - 2 * fw, h - 2 * fw
    sw, sd, sy = 0.045, 0.05, -0.035
    rounded_box("sash_l", (-iw / 2 + sw / 2, sy, h / 2), (sw, sd, ih), trim, 0.004)
    rounded_box("sash_r", (iw / 2 - sw / 2, sy, h / 2), (sw, sd, ih), trim, 0.004)
    rounded_box("sash_t", (0.0, sy, h - fw - sw / 2), (iw - 2 * sw, sd, sw), trim, 0.004)
    rounded_box("sash_b", (0.0, sy, fw + sw / 2), (iw - 2 * sw, sd, sw), trim, 0.004)
    gw, gh = iw - 2 * sw, ih - 2 * sw
    bar = 0.03
    for i in range(1, panes[0]):
        x = -gw / 2 + gw * i / panes[0]
        rounded_box(f"mullion{i}", (x, sy + 0.005, h / 2), (bar, sd - 0.01, gh), trim, 0.003)
    for j in range(1, panes[1]):
        z = fw + sw + gh * j / panes[1]
        rounded_box(f"transom{j}", (0.0, sy + 0.005, z), (gw, sd - 0.01, bar), trim, 0.003)
    glass = box("glass", (0.0, sy + 0.012, h / 2), (gw, 0.004, gh), m["glass"])
    # Håndtaket på høyre ramme.
    rounded_box("handle", (iw / 2 - sw / 2, sy - 0.03, h * 0.45), (0.015, 0.03, 0.1), m["chrome"], 0.004)


def door_lamp(m):
    """Lykt ved døra: brakett fra veggen (y = 0), kasse med glass og tak."""
    iron, glow = m["iron"], m["glow"]
    box("bracket", (0.0, -0.08, 0.14), (0.05, 0.16, 0.05), iron)
    box("plate", (0.0, -0.01, 0.14), (0.1, 0.02, 0.14), iron)
    tube("rod", (0.0, -0.16, 0.14), (0.0, -0.2, 0.08), 0.012, iron)
    cx, cy, cz = 0.0, -0.2, 0.0
    box("glass", (cx, cy, cz), (0.16, 0.16, 0.22), glow)
    for sx in (-1, 1):
        for sy in (-1, 1):
            box(f"corner{sx}{sy}", (cx + sx * 0.08, cy + sy * 0.08, cz), (0.015, 0.015, 0.24), iron)
    box("base", (cx, cy, cz - 0.12), (0.2, 0.2, 0.025), iron)
    bpy.ops.mesh.primitive_cone_add(vertices=4, radius1=0.16, radius2=0.03, depth=0.09, location=(cx, cy, cz + 0.16), rotation=(0, 0, math.pi / 4))
    roof = bpy.context.active_object
    roof.name = "roof"
    roof.data.materials.append(iron)
    ellipsoid("knob", (cx, cy, cz + 0.22), (0.03, 0.03, 0.04), iron, 8, 6)


def flower_box(m):
    """Blomsterkasse 1,1 m, hengt på veggen (y = 0), bunn i z = 0."""
    oak = m["oak"]
    w, d, h = 1.1, 0.26, 0.22
    rounded_box("box_f", (0.0, -d + 0.015, h / 2), (w, 0.03, h), oak, 0.005)
    rounded_box("box_b", (0.0, -0.015, h / 2), (w, 0.03, h), oak, 0.005)
    for sx in (-1, 1):
        rounded_box(f"box_s{sx}", (sx * (w / 2 - 0.015), -d / 2, h / 2), (0.03, d, h), oak, 0.005)
    box("box_bottom", (0.0, -d / 2, 0.015), (w, d, 0.03), oak)
    box("soil", (0.0, -d / 2, h - 0.03), (w - 0.06, d - 0.06, 0.02), m["soil"])
    colors = ["red", "yellow", "white", "purple", "pink", "red", "yellow", "white"]
    for k in range(8):
        x = -w / 2 + 0.1 + k * (w - 0.2) / 7
        y = -d / 2 + (0.06 if k % 2 else -0.06)
        top = h + 0.12 + (k % 3) * 0.03
        tube(f"stem{k}", (x, y, h - 0.02), (x, y, top), 0.004, m["leaf"], vertices=5)
        ellipsoid(f"leaf{k}a", (x + 0.04, y, h + 0.04), (0.09, 0.05, 0.03), m["leaf"], 8, 5)
        ellipsoid(f"leaf{k}b", (x - 0.04, y + 0.02, h + 0.07), (0.08, 0.05, 0.03), m["leaf"], 8, 5)
        ellipsoid(f"bloom{k}", (x, y, top + 0.02), (0.07, 0.07, 0.06), m[colors[k]], 10, 6)
        ellipsoid(f"bloom{k}s", (x + 0.03, y - 0.03, top - 0.03), (0.045, 0.045, 0.04), m[colors[(k + 3) % 8]], 8, 5)


BUILDERS = {
    "window_22": lambda m: window(m, 1.0, 1.5, (2, 2)),
    "window_23": lambda m: window(m, 1.0, 1.5, (2, 3)),
    "door_lamp": door_lamp,
    "flower_box": flower_box,
}

if __name__ == "__main__":
    kinds = [a for a in sys.argv[1:] if a in BUILDERS] or list(BUILDERS)
    import os
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        BUILDERS[k](mats())
        export(f"{OUT}/{k}.glb")
