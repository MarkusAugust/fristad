"""Innbyggerne: én stilisert menneskemodell, rigget, med klippene idle
(løkke), talk (løkke) og wave. Kjør:

    python3 tools/models/npc.py [størrelse]

Tunikaen og ermene er materialet «paint», så fargen settes per instans
(Builder::place_tinted / ModelInstance). Origo i bakken mellom føttene,
fronten mot +X, 1,78 m høy."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import Clip, armature, bake_textured, box, clear, ellipsoid, material, rounded_box, tube, wave, weather  # noqa: E402
from mathutils import Vector  # noqa: E402

OUT = "assets/models/npc"
FPS = 24


def mats():
    m = {
        "skin": material("skin", (0.75, 0.56, 0.44), 0.6),
        "hair": material("hair", (0.18, 0.1, 0.05), 0.7),
        "paint": material("paint", (1.0, 1.0, 1.0), 0.75),
        "pants": material("pants", (0.2, 0.18, 0.16), 0.8),
        "boot": material("boot", (0.14, 0.09, 0.05), 0.7),
        "belt": material("belt", (0.25, 0.14, 0.07), 0.7),
        "eye": material("eye", (0.05, 0.05, 0.06), 0.4),
    }
    weather(m["pants"], 0.0, 0.5, 0.3, 8.0)
    weather(m["boot"], 0.0, 0.3, 0.4, 8.0)
    return m


BONES = [
    ("root", (0, 0, 0), (0, 0, 0.1), None),
    ("hips", (0, 0, 0.95), (0, 0, 1.05), "root"),
    ("spine", (0, 0, 1.05), (0, 0, 1.3), "hips"),
    ("chest", (0, 0, 1.3), (0, 0, 1.5), "spine"),
    ("neck", (0, 0, 1.5), (0, 0, 1.58), "chest"),
    ("head", (0, 0, 1.58), (0, 0, 1.8), "neck"),
    ("upper_arm.L", (0, 0.22, 1.46), (0.02, 0.27, 1.16), "chest"),
    ("forearm.L", (0.02, 0.27, 1.16), (0.1, 0.28, 0.9), "upper_arm.L"),
    ("hand.L", (0.1, 0.28, 0.9), (0.14, 0.28, 0.8), "forearm.L"),
    ("upper_arm.R", (0, -0.22, 1.46), (0.02, -0.27, 1.16), "chest"),
    ("forearm.R", (0.02, -0.27, 1.16), (0.1, -0.28, 0.9), "upper_arm.R"),
    ("hand.R", (0.1, -0.28, 0.9), (0.14, -0.28, 0.8), "forearm.R"),
    ("thigh.L", (0, 0.1, 0.95), (0.01, 0.11, 0.5), "hips"),
    ("shin.L", (0.01, 0.11, 0.5), (0, 0.12, 0.08), "thigh.L"),
    ("foot.L", (0, 0.12, 0.08), (0.18, 0.12, 0.0), "shin.L"),
    ("thigh.R", (0, -0.1, 0.95), (0.01, -0.11, 0.5), "hips"),
    ("shin.R", (0.01, -0.11, 0.5), (0, -0.12, 0.08), "thigh.R"),
    ("foot.R", (0, -0.12, 0.08), (0.18, -0.12, 0.0), "shin.R"),
]
AT = {b[0]: (Vector(b[1]), Vector(b[2])) for b in BONES}
RIGID = {"head": "head", "hair": "head", "eye": "head", "nose": "head", "ear": "head", "foot": "foot", "boot": "foot"}
RADII = {"hips": 0.1, "spine": 0.12, "chest": 0.14, "root": -1.0}


def limb(name, bones, radii, mat):
    pts = [AT[bones[0]][0]] + [AT[b][1] for b in bones]
    for i in range(len(pts) - 1):
        tube(f"{name}{i}", pts[i], pts[i + 1], radii[i], mat, 10)
        if i > 0:
            ellipsoid(f"{name}_j{i}", pts[i], (radii[i] * 2.1,) * 3, mat, 10, 6)


def human(m):
    # Hodet med hår, ører, nese og øyne.
    ellipsoid("head", (0.0, 0.0, 1.69), (0.2, 0.19, 0.24), m["skin"], 16, 10)
    ellipsoid("hair", (-0.02, 0.0, 1.74), (0.21, 0.21, 0.2), m["hair"], 16, 10)
    for sy in (1, -1):
        ellipsoid(f"ear{sy}", (0.0, sy * 0.1, 1.68), (0.03, 0.02, 0.05), m["skin"], 8, 5)
        ellipsoid(f"eye{sy}", (0.09, sy * 0.035, 1.71), (0.02, 0.025, 0.02), m["eye"], 8, 5)
    ellipsoid("nose", (0.1, 0.0, 1.66), (0.04, 0.03, 0.035), m["skin"], 8, 5)
    tube("neck", (0, 0, 1.5), (0, 0, 1.62), 0.055, m["skin"], 10)
    # Overkroppen: bryst og mage i tunikaen, belte, hofter i buksa.
    rounded_box("chest", (0.0, 0.0, 1.38), (0.24, 0.38, 0.26), m["paint"], 0.06, segments=3)
    rounded_box("belly", (0.0, 0.0, 1.16), (0.23, 0.34, 0.22), m["paint"], 0.06, segments=3)
    box("belt", (0.0, 0.0, 1.04), (0.25, 0.36, 0.05), m["belt"])
    rounded_box("hipbox", (0.0, 0.0, 0.94), (0.24, 0.34, 0.18), m["pants"], 0.05, segments=3)
    for sy in (1, -1):
        ellipsoid(f"shoulder{sy}", (0.0, sy * 0.22, 1.46), (0.13, 0.13, 0.12), m["paint"], 10, 6)
    # Armene i ermer, hender i hud.
    for side, sy in (("L", 1), ("R", -1)):
        limb(f"arm{side}", [f"upper_arm.{side}", f"forearm.{side}"], [0.055, 0.045], m["paint"])
        ellipsoid(f"hand{side}", AT[f"hand.{side}"][0] + Vector((0.02, 0, -0.05)), (0.07, 0.05, 0.12), m["skin"], 10, 6)
        limb(f"leg{side}", [f"thigh.{side}", f"shin.{side}"], [0.08, 0.065], m["pants"])
        a, t = AT[f"foot.{side}"]
        box(f"boot{side}", (0.07, sy * 0.12, 0.05), (0.27, 0.11, 0.1), m["boot"])
        box(f"foot{side}top", (-0.02, sy * 0.12, 0.12), (0.12, 0.1, 0.1), m["boot"])


def clips(arm):
    c = Clip(arm, "idle", 72, FPS)
    c.rest_all(0)
    for f, v in wave(72, 72, 1.0, step=6):
        c.scale("chest", f, (1.0 + 0.02 * v, 1.0 + 0.02 * v, 1.0))
        c.rot("spine", f, y=1.5 * v)
        c.rot("head", f, z=6 * v, y=2 * abs(v))
        c.rot("upper_arm.L", f, x=2 * v)
        c.rot("upper_arm.R", f, x=-2 * v)
    for f, v in wave(72, 36, 1.0, step=6, phase=1.0):
        c.rot("hips", f, x=1.5 * v)
    c.finish()
    # Snakker: nikker, gestikulerer med høyre hånd, vipper på hodet.
    c = Clip(arm, "talk", 48, FPS)
    c.rest_all(0)
    for f, v in wave(48, 24, 1.0, step=4):
        c.rot("head", f, y=5 + 4 * v, z=4 * v)
        c.rot("upper_arm.R", f, y=-35 - 10 * v, x=-15 + 8 * v)
        c.rot("forearm.R", f, y=-50 - 15 * v)
        c.rot("hand.R", f, x=20 * v)
        c.rot("chest", f, z=3 * v)
    c.finish()
    # Vinker.
    c = Clip(arm, "wave", 36, FPS)
    c.rest_all(0)
    c.rot("upper_arm.R", 6, x=-140, y=-20)
    c.rot("forearm.R", 6, x=-20)
    for f, v in wave(30, 10, 1.0, step=3, offset=0):
        c.rot("hand.R", 6 + f, x=-15 + 25 * v)
    c.rot("upper_arm.R", 36)
    c.rot("forearm.R", 36)
    c.rot("hand.R", 36)
    c.finish()
    # Går: ett skritt med hvert bein på et sekund, armene i motfase,
    # kneet bøyd når foten svinger fram, hoftene opp og ned.
    c = Clip(arm, "walk", 24, FPS)
    for f in range(0, 25, 2):
        a = 2 * math.pi * f / 24
        sw = math.sin(a)
        c.rot("thigh.L", f, y=-24 * sw)
        c.rot("thigh.R", f, y=24 * sw)
        c.rot("shin.L", f, y=32 * max(0.0, math.sin(a - 0.9)) + 4)
        c.rot("shin.R", f, y=32 * max(0.0, math.sin(a - 0.9 + math.pi)) + 4)
        c.rot("foot.L", f, y=-10 * math.sin(a - 0.4))
        c.rot("foot.R", f, y=10 * math.sin(a - 0.4))
        c.rot("upper_arm.L", f, y=16 * sw)
        c.rot("upper_arm.R", f, y=-16 * sw)
        c.rot("forearm.L", f, y=-12 - 6 * max(0.0, sw))
        c.rot("forearm.R", f, y=-12 - 6 * max(0.0, -sw))
        c.rot("spine", f, z=4 * sw)
        c.rot("chest", f, z=-5 * sw, y=3)
        c.rot("head", f, z=2 * sw)
        c.loc("hips", f, (0, 0, -0.02 * abs(math.cos(a))))
    c.finish()
    bpy.context.scene.frame_end = 72


if __name__ == "__main__":
    args = sys.argv[1:]
    size = next((int(a) for a in args if a.isdigit()), 1024)
    os.makedirs(OUT, exist_ok=True)
    clear()
    m = mats()
    human(m)
    rig = armature("npc_rig", BONES)
    clips(rig)
    bake_textured(f"{OUT}/npc.glb", size, keep=("glass", "glow", "fire", "mesh"), armature=rig, rigid=RIGID, skin={"blend": 0.05, "radii": RADII})
