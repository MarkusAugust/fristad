"""Spillerens armer i førsteperson: høyre hånd med sverd, venstre med
rundskjold, rigget og animert. Kjør:

    python3 tools/models/arms.py [størrelse]

Modellen står i kamerarommet: kameraet i origo ser langs +Y i Blender
(−Z i glTF), Z er opp og X er høyre. Spillet poserer instansen i øyet
med kameraets dreining. Klippene: idle (løkke), swing1, swing2, swing3
(komboen, treffet ved SWING_HIT), heavy (ladning til HEAVY_HOLD, hugget
lander ved HEAVY_HIT), block (holdes på siste bilde), parry, hurt."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import Clip, armature, bake_textured, box, clear, cylinder, ellipsoid, material, rounded_box, tube, wave, weather  # noqa: E402
from mathutils import Vector  # noqa: E402

OUT = "assets/models/arms"
FPS = 24
# Sekundene i klippene der treffet lander (speiles i domain/combat.rs).
SWING_HIT = (5 / FPS, 5 / FPS, 7 / FPS)
HEAVY_HOLD = 10 / FPS
HEAVY_HIT = 13 / FPS


def mats():
    m = {
        "skin": material("skin", (0.62, 0.44, 0.34), 0.6),
        "leather": material("leather", (0.22, 0.13, 0.08), 0.75),
        "strap": material("strap", (0.12, 0.08, 0.05), 0.8),
        "steel": material("steel", (0.75, 0.76, 0.78), 0.3, 1.0),
        "iron": material("iron", (0.3, 0.3, 0.32), 0.55, 1.0),
        "wood": material("wood", (0.45, 0.3, 0.17), 0.7),
        "grip": material("grip", (0.15, 0.1, 0.07), 0.85),
    }
    weather(m["steel"], -0.2, 0.3, 0.25, 20.0)
    weather(m["wood"], -0.3, 0.1, 0.4, 12.0)
    return m


# Skjelettet (navn, hode, hale, forelder) i kamerarommet.
BONES = [
    ("root", (0, 0, 0), (0, 0.1, 0), None),
    ("upper_arm.R", (0.26, -0.08, -0.3), (0.36, 0.2, -0.52), "root"),
    ("forearm.R", (0.36, 0.2, -0.52), (0.29, 0.5, -0.41), "upper_arm.R"),
    ("hand.R", (0.29, 0.5, -0.41), (0.27, 0.6, -0.37), "forearm.R"),
    ("sword", (0.27, 0.6, -0.37), (0.12, 1.45, 0.12), "hand.R"),
    ("upper_arm.L", (-0.26, -0.08, -0.3), (-0.38, 0.2, -0.5), "root"),
    ("forearm.L", (-0.38, 0.2, -0.5), (-0.33, 0.5, -0.4), "upper_arm.L"),
    ("hand.L", (-0.33, 0.5, -0.4), (-0.33, 0.59, -0.38), "forearm.L"),
    ("shield", (-0.33, 0.59, -0.38), (-0.33, 0.69, -0.38), "hand.L"),
]
AT = {b[0]: (Vector(b[1]), Vector(b[2])) for b in BONES}
RIGID = {"blade": "sword", "fuller": "sword", "guard": "sword", "grip": "sword", "pommel": "sword", "shield": "shield", "rim": "shield", "boss": "shield", "handle": "shield", "finger.R": "hand.R", "finger.L": "hand.L", "palm": "hand"}
RADII = {"root": -1.0}


def arm(side, m):
    s = ".R" if side == "R" else ".L"
    ua, fa, ha = AT["upper_arm" + s], AT["forearm" + s], AT["hand" + s]
    # Ermet i lær over overarmen og underarmen, stroppet ved albuen.
    tube(f"sleeve_u{side}", ua[0], ua[1], 0.062, m["leather"], 12)
    ellipsoid(f"elbow{side}", ua[1], (0.13, 0.13, 0.13), m["leather"], 12, 8)
    tube(f"sleeve_f{side}", fa[0], fa[1], 0.052, m["leather"], 12)
    mid = (fa[0] + fa[1]) * 0.5
    tube(f"strap{side}", mid - (fa[1] - fa[0]) * 0.05, mid + (fa[1] - fa[0]) * 0.05, 0.056, m["strap"], 12)
    tube(f"cuff{side}", fa[1] - (fa[1] - fa[0]) * 0.12, fa[1], 0.058, m["strap"], 12)
    # Hånda: en håndflate og fire fingre som griper.
    d = (ha[1] - ha[0]).normalized()
    ellipsoid(f"palm{side}", ha[0] + d * 0.05, (0.08, 0.11, 0.06), m["skin"], 12, 8)
    sx = 1 if side == "R" else -1
    for k in range(4):
        base = ha[0] + d * (0.07 + k * 0.012) + Vector((sx * -0.02, 0, 0.025 - k * 0.018))
        p1 = base + Vector((sx * 0.03, 0.02, -0.015))
        p2 = p1 + Vector((sx * 0.015, 0.01, -0.03))
        tube(f"finger{s}{k}a", base, p1, 0.011, m["skin"], 8)
        tube(f"finger{s}{k}b", p1, p2, 0.01, m["skin"], 8)
    tb = ha[0] + d * 0.06 + Vector((sx * 0.02, 0.0, 0.03))
    tube(f"finger{s}thumb", tb, tb + Vector((sx * -0.01, 0.03, 0.01)), 0.011, m["skin"], 8)


def sword(m):
    g0, g1 = AT["sword"]
    d = (g1 - g0).normalized()
    up = Vector((0, 0, 1))
    # Grepet rundt hånda, parerstang, klinge med hulkil, knapp.
    tube("grip", g0 - d * 0.08, g0 + d * 0.06, 0.016, m["grip"], 10)
    ellipsoid("pommel", g0 - d * 0.1, (0.045, 0.045, 0.045), m["iron"], 10, 6)
    guard_c = g0 + d * 0.07
    rot = d.to_track_quat("Y", "Z").to_euler()
    box("guard", guard_c, (0.2, 0.025, 0.02), m["iron"], rot=rot)
    blade_len = (g1 - guard_c).length
    bc = guard_c + d * blade_len * 0.5
    rounded_box("blade", bc, (0.05, blade_len, 0.006), m["steel"], 0.002, rot=rot, segments=1)
    rounded_box("fuller", bc - d * blade_len * 0.1, (0.012, blade_len * 0.7, 0.008), m["iron"], 0.002, rot=rot, segments=1)
    # Spissen: en kjegle.
    bpy.ops.mesh.primitive_cone_add(vertices=4, radius1=0.025, radius2=0.0, depth=0.09, location=g1 + d * 0.045, rotation=d.to_track_quat("Z", "Y").to_euler())
    tip = bpy.context.active_object
    tip.name = "blade_tip"
    tip.scale = (1.0, 0.25, 1.0)
    tip.data.materials.append(m["steel"])
    del up


def shield(m):
    h0, h1 = AT["shield"]
    c = h0 + Vector((-0.02, 0.06, 0.04))
    rot = (math.pi / 2, 0, 0)
    cylinder("shield", c, 0.2, 0.025, m["wood"], rot=rot, vertices=28)
    cylinder("rim", c + Vector((0, 0.002, 0)), 0.21, 0.02, m["iron"], rot=rot, vertices=28)
    cylinder("shield_face", c + Vector((0, 0.01, 0)), 0.19, 0.012, m["wood"], rot=rot, vertices=28)
    ellipsoid("boss", c + Vector((0, 0.03, 0)), (0.1, 0.07, 0.1), m["iron"], 14, 8)
    for a in range(4):
        ang = a * math.pi / 2 + math.pi / 4
        box(f"rivet{a}", c + Vector((math.cos(ang) * 0.165, 0.014, math.sin(ang) * 0.165)), (0.02, 0.01, 0.02), m["iron"])
    for k in (-1, 1):
        box(f"plank{k}", c + Vector((k * 0.07, 0.016, 0)), (0.006, 0.004, 0.36), m["grip"])
    # Håndtaket bak skjoldet, der hånda griper.
    box("handle", c + Vector((0, -0.03, 0)), (0.03, 0.025, 0.14), m["grip"])


def build_arms(m):
    arm("R", m)
    arm("L", m)
    sword(m)
    shield(m)


def clips(arm):
    # Idle: pusten løfter armene litt, skjoldet vagger.
    c = Clip(arm, "idle", 48, FPS)
    c.rest_all(0)
    for f, v in wave(48, 48, 1.0):
        c.loc("root", f, (0, 0.004 * v, 0.008 * v))
        c.rot("upper_arm.R", f, x=1.5 * v)
        c.rot("upper_arm.L", f, x=-1.2 * v, z=1.0 * v)
    c.finish()
    # Kombo: sving fra høyre, sving fra venstre, hugg ovenfra.
    c = Clip(arm, "swing1", 12, FPS)
    c.rest_all(0)
    c.rot("upper_arm.R", 3, x=12, z=-32)
    c.rot("forearm.R", 3, z=-10)
    c.rot("hand.R", 3, y=-20)
    c.rot("upper_arm.R", 6, x=-6, z=48)
    c.rot("forearm.R", 6, z=20)
    c.rot("hand.R", 6, y=25)
    c.loc("root", 6, (-0.04, 0.03, -0.01))
    c.rot("upper_arm.R", 12)
    c.rot("forearm.R", 12)
    c.rot("hand.R", 12)
    c.loc("root", 12, (0, 0, 0))
    c.finish()
    c = Clip(arm, "swing2", 12, FPS)
    c.rest_all(0)
    c.rot("upper_arm.R", 3, x=8, z=40)
    c.rot("forearm.R", 3, z=18)
    c.rot("hand.R", 3, y=30)
    c.rot("upper_arm.R", 6, x=-8, z=-40)
    c.rot("forearm.R", 6, z=-14)
    c.rot("hand.R", 6, y=-25)
    c.loc("root", 6, (0.04, 0.03, -0.01))
    c.rot("upper_arm.R", 12)
    c.rot("forearm.R", 12)
    c.rot("hand.R", 12)
    c.loc("root", 12, (0, 0, 0))
    c.finish()
    c = Clip(arm, "swing3", 16, FPS)
    c.rest_all(0)
    c.rot("upper_arm.R", 4, x=70, z=8)
    c.rot("forearm.R", 4, x=20)
    c.rot("upper_arm.R", 8, x=-38, z=-4)
    c.rot("forearm.R", 8, x=-10)
    c.loc("root", 8, (0, 0.05, -0.04))
    c.rot("upper_arm.R", 16)
    c.rot("forearm.R", 16)
    c.loc("root", 16, (0, 0, 0))
    c.finish()
    # Tungt hugg: lang ladning over hodet, og et hugg som drar kroppen med.
    c = Clip(arm, "heavy", 24, FPS)
    c.rest_all(0)
    c.rot("upper_arm.R", 10, x=32, z=-30)
    c.rot("forearm.R", 10, x=22, z=-8)
    c.rot("upper_arm.L", 10, x=10, z=-10)
    c.loc("root", 10, (0.02, -0.03, 0.03))
    c.rot("upper_arm.R", 13, x=-45, z=-10)
    c.rot("forearm.R", 13, x=-15)
    c.loc("root", 13, (-0.02, 0.08, -0.07))
    c.rot("upper_arm.R", 17, x=-40, z=-8)
    c.loc("root", 17, (-0.01, 0.05, -0.06))
    c.rot("upper_arm.R", 24)
    c.rot("forearm.R", 24)
    c.rot("upper_arm.L", 24)
    c.loc("root", 24, (0, 0, 0))
    c.finish()
    # Blokk: skjoldet opp foran ansiktet, holdes på siste bilde.
    c = Clip(arm, "block", 8, FPS)
    c.rest_all(0)
    c.rot("upper_arm.L", 8, x=14, z=-20)
    c.rot("forearm.L", 8, x=6, z=-10)
    c.rot("hand.L", 8, y=10)
    c.rot("upper_arm.R", 8, x=-8, z=-12)
    c.finish()
    # Parade: fra blokken, skjoldet støter fram og kommer tilbake.
    c = Clip(arm, "parry", 12, FPS)
    c.rest_all(0)
    c.rot("upper_arm.L", 0, x=14, z=-20)
    c.rot("forearm.L", 0, x=6, z=-10)
    c.rot("upper_arm.L", 4, x=22, z=-26)
    c.rot("forearm.L", 4, x=30, z=-16)
    c.loc("root", 4, (-0.03, 0.08, 0.0))
    c.rot("upper_arm.L", 12, x=14, z=-20)
    c.rot("forearm.L", 12, x=6, z=-10)
    c.loc("root", 12, (0, 0, 0))
    c.finish()
    # Treffes: armene slenges, skjoldet kastes litt inn.
    c = Clip(arm, "hurt", 10, FPS)
    c.rest_all(0)
    c.loc("root", 2, (0.04, -0.06, -0.04))
    c.rot("upper_arm.R", 2, x=-15, z=12)
    c.rot("upper_arm.L", 2, x=-10, z=10)
    c.loc("root", 10, (0, 0, 0))
    c.rot("upper_arm.R", 10)
    c.rot("upper_arm.L", 10)
    c.finish()
    bpy.context.scene.frame_end = 48


if __name__ == "__main__":
    args = sys.argv[1:]
    size = next((int(a) for a in args if a.isdigit()), 1024)
    os.makedirs(OUT, exist_ok=True)
    clear()
    m = mats()
    build_arms(m)
    rig = armature("arms_rig", BONES)
    clips(rig)
    bake_textured(f"{OUT}/arms.glb", size, keep=("glass", "glow", "fire", "mesh"), armature=rig, rigid=RIGID, skin={"blend": 0.04, "radii": RADII})
