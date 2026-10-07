"""Fiendene og lootet i Legacy-kjelleren, modellert i Blender (bpy) og bakt
til teksturer (common.bake_textured). Kjør:

    python3 tools/models/mobs.py [mutant_button|token ...] [størrelse]

`mutant_button` er en `.fs-button` som har mutert av for mye JavaScript:
en tjukk knappeplate som kropp, fire edderkoppbein av krøllparenteser,
glødende øyne, et skjevt gap og en kabel med plugg som slenger etter.
Origo i bakken under midten, fronten mot +X. `token` er en designtoken
som mynt: en skive i aksentfargen med glødende kant, origo i bunnen."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import bake_textured, box, clear, cylinder, ellipsoid, material, new_object, rounded_box, smooth, tube, weather  # noqa: E402
from mathutils import Vector  # noqa: E402

OUT = "assets/models/mobs"


def mats():
    m = {
        # Knappens aksentblå, men skitten og med grønn råte.
        "body": material("body", (0.08, 0.22, 0.55), 0.45),
        "face": material("face", (0.12, 0.3, 0.7), 0.4),
        "rot": material("rot", (0.25, 0.45, 0.12), 0.8),
        "leg": material("leg", (0.04, 0.04, 0.045), 0.6),
        "tooth": material("tooth", (0.9, 0.88, 0.8), 0.5),
        "mouth": material("mouth", (0.08, 0.02, 0.02), 0.9),
        "glow_eye": material("glow_eye", (1.0, 0.75, 0.2), 0.3),
        "cable": material("cable", (0.1, 0.1, 0.11), 0.7),
        "plug": material("plug", (0.6, 0.6, 0.62), 0.4, 1.0),
        "coin": material("coin", (0.1, 0.3, 0.7), 0.35),
        "glow_rim": material("glow_rim", (0.4, 0.7, 1.0), 0.3),
    }
    weather(m["body"], 0.3, 0.9, 0.45, 10.0)
    weather(m["face"], 0.3, 0.9, 0.3, 10.0)
    return m


def mutant_button(m):
    """Kroppen: ei knappeplate 90 × 46 × 18 cm som svever 55 cm over bakken
    på fire bein. Råteflekker på sidene, et gap foran, øyne over gapet."""
    z0 = 0.55
    rounded_box("body", (0.0, 0.0, z0 + 0.23), (0.9, 0.46, 0.18), m["body"], 0.05, segments=4)
    # Forsida: ei lysere plate (knappens flate), litt skjev.
    f = rounded_box("face", (0.0, 0.0, z0 + 0.33), (0.86, 0.42, 0.02), m["face"], 0.03, segments=3)
    f.rotation_euler = (0.0, 0.0, math.radians(4))
    # Råte: grønne klumper som vokser ut av kantene.
    for k, (x, y, z, s) in enumerate([(0.35, 0.2, z0 + 0.3, 0.12), (-0.4, -0.18, z0 + 0.2, 0.1), (0.1, -0.26, z0 + 0.28, 0.09), (-0.2, 0.25, z0 + 0.35, 0.08)]):
        ellipsoid(f"rot{k}", (x, y, z), (s * 1.4, s, s * 0.8), m["rot"], 10, 6)
    # Gapet foran: et mørkt hull med tenner i over- og underkant.
    box("mouth", (0.44, 0.0, z0 + 0.2), (0.06, 0.5, 0.1), m["mouth"])
    for k in range(6):
        y = -0.2 + k * 0.08
        bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.025, radius2=0.0, depth=0.06, location=(0.46, y, z0 + 0.22), rotation=(math.pi, 0, 0))
        t = bpy.context.active_object
        t.name = f"tooth_u{k}"
        t.data.materials.append(m["tooth"])
        bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.022, radius2=0.0, depth=0.05, location=(0.46, y + 0.04, z0 + 0.17))
        t = bpy.context.active_object
        t.name = f"tooth_l{k}"
        t.data.materials.append(m["tooth"])
    # Øynene: to glødende kuler ulikt store, på stilker.
    for sy, r in ((0.14, 0.055), (-0.12, 0.04)):
        tube(f"stalk{sy}", (0.3, sy, z0 + 0.4), (0.4, sy * 1.3, z0 + 0.52), 0.015, m["leg"], 6)
        ellipsoid(f"eye{sy}", (0.4, sy * 1.3, z0 + 0.53), (r * 2, r * 2, r * 2), m["glow_eye"], 10, 6)
    # Beina: fire krøllparenteser { } som edderkoppbein, leddet ut og ned.
    for sx in (1, -1):
        for sy in (1, -1):
            hip = Vector((sx * 0.3, sy * 0.2, z0 + 0.2))
            knee = Vector((sx * 0.5, sy * 0.45, z0 + 0.45))
            foot = Vector((sx * 0.62, sy * 0.6, 0.0))
            tube(f"thigh{sx}{sy}", hip, knee, 0.028, m["leg"], 7)
            tube(f"shin{sx}{sy}", knee, foot, 0.022, m["leg"], 7)
            ellipsoid(f"joint{sx}{sy}", knee, (0.07, 0.07, 0.07), m["leg"], 8, 5)
            # Krøllen: en liten bue ved kneet, som i { }.
            mid = (knee + hip) * 0.5 + Vector((0, sy * 0.06, 0))
            tube(f"curl{sx}{sy}", mid, mid + Vector((0, sy * 0.06, 0.05)), 0.014, m["leg"], 6)
    # Kabelen med plugg som slenger etter bak.
    pts = [Vector((-0.45, 0.05, z0 + 0.2)), Vector((-0.7, 0.1, z0 + 0.05)), Vector((-0.95, 0.0, 0.12)), Vector((-1.15, -0.1, 0.05))]
    for i in range(len(pts) - 1):
        tube(f"cable{i}", pts[i], pts[i + 1], 0.016, m["cable"], 6)
    box("plug", (-1.22, -0.12, 0.05), (0.12, 0.07, 0.06), m["plug"])
    for dy in (-0.02, 0.02):
        box(f"pin{dy}", (-1.3, -0.12 + dy, 0.05), (0.05, 0.008, 0.012), m["plug"])


def token(m):
    """Tokenmynten: skive 36 cm i diameter, 4 cm tykk, med glødende kant og
    en opphøyd ring på hver side."""
    cylinder("coin", (0.0, 0.0, 0.18), 0.18, 0.04, m["coin"], rot=(0, math.pi / 2, 0), vertices=24)
    cylinder("rim", (0.0, 0.0, 0.18), 0.185, 0.02, m["glow_rim"], rot=(0, math.pi / 2, 0), vertices=24)
    for sx in (1, -1):
        cylinder(f"ring{sx}", (sx * 0.022, 0.0, 0.18), 0.12, 0.01, m["glow_rim"], rot=(0, math.pi / 2, 0), vertices=20)
        cylinder(f"inner{sx}", (sx * 0.024, 0.0, 0.18), 0.1, 0.01, m["coin"], rot=(0, math.pi / 2, 0), vertices=20)


BUILDERS = {"mutant_button": (mutant_button, 1024), "token": (token, 256)}

if __name__ == "__main__":
    args = sys.argv[1:]
    kinds = [a for a in args if a in BUILDERS] or list(BUILDERS)
    size = next((int(a) for a in args if a.isdigit()), None)
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        build, default_size = BUILDERS[k]
        build(mats())
        bake_textured(f"{OUT}/{k}.glb", size or default_size, keep=("glass", "glow", "fire", "mesh"))
