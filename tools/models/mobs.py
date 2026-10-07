"""Fiendene og lootet i Legacy-kjelleren, modellert i Blender (bpy), rigget
med armatur og klipp, og bakt til teksturer (common.bake_textured). Kjør:

    python3 tools/models/mobs.py [mutant_button|token ...] [størrelse]

`mutant_button` er en `.fs-button` som har mutert av for mye JavaScript og
forretningslogikk: et krokrygget, tobeint monster av sammenfiltrede kabler
med knappeplata som ansikt og panser, kjeven full av tenner der teksten
sto, tre glødende øyne, klør av krøllparenteser, pigger av `}` på ryggen
og en kabelhale med plugg. Origo i bakken under hoftene, fronten mot +X.
Klippene: idle (løkke), walk (løkke), attack (bittet lander ved
ATTACK_HIT), hit, die. `token` er en designtoken som mynt."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import Clip, armature, bake_textured, box, clear, cylinder, ellipsoid, material, rounded_box, tube, wave, weather  # noqa: E402
from mathutils import Vector  # noqa: E402

OUT = "assets/models/mobs"
FPS = 24
# Sekundet i attack-klippet der bittet lander (bilde 15 av 30).
ATTACK_HIT = 15 / FPS


def mats():
    m = {
        # Knappeplata: aksentblå, skitten og sprukken.
        "plate": material("plate", (0.07, 0.2, 0.52), 0.5),
        "face": material("face", (0.11, 0.28, 0.66), 0.45),
        # Kroppen: kabelsvart med blåskjær, og grønn råte.
        "flesh": material("flesh", (0.035, 0.04, 0.06), 0.65),
        "cable": material("cable", (0.05, 0.05, 0.055), 0.55),
        "rot": material("rot", (0.2, 0.4, 0.1), 0.85),
        "tooth": material("tooth", (0.88, 0.85, 0.75), 0.5),
        "mouth": material("mouth", (0.1, 0.02, 0.02), 0.9),
        "glow_eye": material("glow_eye", (1.0, 0.7, 0.15), 0.3),
        "plug": material("plug", (0.6, 0.6, 0.62), 0.4, 1.0),
        "coin": material("coin", (0.1, 0.3, 0.7), 0.35),
        "glow_rim": material("glow_rim", (0.4, 0.7, 1.0), 0.3),
    }
    weather(m["plate"], 0.9, 1.5, 0.5, 9.0)
    weather(m["face"], 0.9, 1.5, 0.35, 9.0)
    weather(m["flesh"], 0.0, 0.5, 0.3, 6.0)
    return m


# Skjelettet (navn, hode, hale, forelder), i meter, Z opp, fronten mot +X.
BONES = [
    ("root", (0, 0, 0), (0, 0, 0.2), None),
    ("hips", (-0.15, 0, 0.85), (0.0, 0, 1.05), "root"),
    ("chest", (0.0, 0, 1.05), (0.25, 0, 1.2), "hips"),
    ("neck", (0.25, 0, 1.2), (0.4, 0, 1.28), "chest"),
    ("head", (0.4, 0, 1.28), (0.75, 0, 1.3), "neck"),
    ("jaw", (0.44, 0, 1.14), (0.7, 0, 1.1), "head"),
    ("upper_arm.L", (0.12, 0.28, 1.15), (0.3, 0.42, 0.85), "chest"),
    ("forearm.L", (0.3, 0.42, 0.85), (0.55, 0.4, 0.55), "upper_arm.L"),
    ("hand.L", (0.55, 0.4, 0.55), (0.7, 0.4, 0.4), "forearm.L"),
    ("upper_arm.R", (0.12, -0.28, 1.15), (0.3, -0.42, 0.85), "chest"),
    ("forearm.R", (0.3, -0.42, 0.85), (0.55, -0.4, 0.55), "upper_arm.R"),
    ("hand.R", (0.55, -0.4, 0.55), (0.7, -0.4, 0.4), "forearm.R"),
    ("thigh.L", (-0.1, 0.17, 0.85), (0.05, 0.2, 0.5), "hips"),
    ("shin.L", (0.05, 0.2, 0.5), (-0.12, 0.2, 0.15), "thigh.L"),
    ("foot.L", (-0.12, 0.2, 0.15), (0.18, 0.2, 0.0), "shin.L"),
    ("thigh.R", (-0.1, -0.17, 0.85), (0.05, -0.2, 0.5), "hips"),
    ("shin.R", (0.05, -0.2, 0.5), (-0.12, -0.2, 0.15), "thigh.R"),
    ("foot.R", (-0.12, -0.2, 0.15), (0.18, -0.2, 0.0), "shin.R"),
    ("tail1", (-0.25, 0, 0.9), (-0.55, 0.05, 0.75), "hips"),
    ("tail2", (-0.55, 0.05, 0.75), (-0.85, 0.0, 0.5), "tail1"),
    ("tail3", (-0.85, 0.0, 0.5), (-1.1, -0.08, 0.2), "tail2"),
]
BONE_AT = {b[0]: (Vector(b[1]), Vector(b[2])) for b in BONES}
# Deler som henger stivt på ett bein (objektnavn-prefiks → bein).
RIGID = {"plate": "head", "face": "head", "eye": "head", "tooth_u": "head", "mouth": "head", "jaw": "jaw", "tooth_l": "jaw", "plug": "tail3", "pin": "tail3"}
# Tjukke bein rekker lenger når vektene regnes ut.
RADII = {"hips": 0.16, "chest": 0.2, "neck": 0.06, "head": 0.06, "jaw": 0.0, "root": -1.0}


def chain(name, pts, radii, mat, joints=True):
    """Rør fra punkt til punkt med kuler i leddene, som en kabel eller en klo."""
    for i in range(len(pts) - 1):
        tube(f"{name}_{i}", pts[i], pts[i + 1], radii[i], mat, 8)
        if joints and 0 < i:
            ellipsoid(f"{name}_j{i}", pts[i], (radii[i] * 2.1,) * 3, mat, 8, 5)


def claw(name, base, forward, down, mat, length=0.2, r=0.022):
    """En klo som bøyer seg som en krøllparentes: tre ledd som smalner."""
    f, d = Vector(forward).normalized(), Vector(down).normalized()
    b = Vector(base)
    pts = [b, b + f * length * 0.45 - d * length * 0.1, b + f * length * 0.8 + d * length * 0.25, b + f * length * 0.9 + d * length * 0.6]
    chain(name, pts, [r, r * 0.75, r * 0.45], mat)
    tip = pts[-1]
    direction = (pts[-1] - pts[-2]).normalized()
    bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=r * 0.45, radius2=0.0, depth=length * 0.25, location=tip + direction * length * 0.12, rotation=direction.to_track_quat("Z", "Y").to_euler())
    c = bpy.context.active_object
    c.name = f"{name}_tip"
    c.data.materials.append(mat)


def limb(name, bones, radii, mat):
    """Rør langs en kjede av bein, med kuler i leddene."""
    pts = [BONE_AT[bones[0]][0]] + [BONE_AT[b][1] for b in bones]
    chain(name, pts, radii, mat)


def mutant_button(m):
    # Kroppen: to sammenvokste klumper, hofter og bryst, i kabelsvart.
    ellipsoid("belly", (-0.07, 0, 0.92), (0.5, 0.52, 0.46), m["flesh"], 20, 12)
    ellipsoid("chest", (0.12, 0, 1.1), (0.56, 0.62, 0.5), m["flesh"], 20, 12)
    ellipsoid("neckbulge", (0.3, 0, 1.22), (0.26, 0.3, 0.22), m["flesh"], 14, 8)
    # Kabler som snor seg rundt kroppen.
    for k in range(7):
        pts = []
        for s in range(9):
            t = s / 8
            a = k * 0.9 + t * 4.5
            cx = -0.2 + t * 0.5
            cz = 0.92 + t * 0.2
            ry = 0.26 + 0.05 * math.sin(k + s)
            rz = 0.24 + 0.04 * math.cos(k * 2 + s)
            pts.append(Vector((cx, math.cos(a) * ry, cz + math.sin(a) * rz)))
        chain(f"cable{k}", pts, [0.022 + 0.004 * (k % 3)] * 8, m["cable"], joints=False)
    # Råteflekker.
    for k, (x, y, z, s) in enumerate([(0.2, 0.25, 1.15, 0.12), (-0.2, -0.2, 0.95, 0.1), (0.05, -0.3, 1.2, 0.09), (-0.1, 0.27, 1.05, 0.08)]):
        ellipsoid(f"rot{k}", (x, y, z), (s * 1.4, s * 0.9, s * 0.8), m["rot"], 10, 6)
    # Pigger av } på ryggen.
    for k in range(5):
        x = -0.22 + k * 0.1
        z = 1.18 + 0.1 * math.sin(k * 1.2) + k * 0.03
        b = Vector((x, 0, z))
        pts = [b, b + Vector((-0.03, 0.02, 0.08)), b + Vector((-0.08, 0.0, 0.12)), b + Vector((-0.1, -0.02, 0.17))]
        chain(f"spike{k}", pts, [0.02, 0.014, 0.008], m["cable"])
    # Hodet: knappeplata som ansikt, forsida mot +X, med ei lysere flate.
    rounded_box("plate", (0.57, 0, 1.33), (0.16, 0.5, 0.26), m["plate"], 0.035, segments=3)
    rounded_box("face", (0.655, 0, 1.335), (0.02, 0.44, 0.2), m["face"], 0.02, segments=2)
    # Underkjeven: nederste del av plata, som har løsnet.
    rounded_box("jaw", (0.57, 0, 1.11), (0.17, 0.46, 0.11), m["plate"], 0.025, segments=2)
    rounded_box("jawface", (0.657, 0, 1.11), (0.02, 0.4, 0.07), m["face"], 0.015, segments=2)
    jaw_root = ("jawroot", (0.46, 0, 1.11), (0.08, 0.3, 0.1), m["flesh"])
    box(*jaw_root)
    # Gapet: mørkt inni, tenner i over- og underkant.
    box("mouth", (0.56, 0, 1.19), (0.14, 0.42, 0.04), m["mouth"])
    for k in range(7):
        y = -0.18 + k * 0.06
        for prefix, z, flip in (("tooth_u", 1.2, math.pi), ("tooth_l", 1.17, 0.0)):
            h = 0.05 + 0.02 * (k % 2)
            bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.018, radius2=0.0, depth=h, location=(0.64, y + (0.02 if prefix == "tooth_l" else 0.0), z + (h * 0.5 if prefix == "tooth_l" else -h * 0.5)), rotation=(flip, 0, 0))
            t = bpy.context.active_object
            t.name = f"{prefix}{k}"
            t.data.materials.append(m["tooth"])
    # Øynene: tre glødende kuler, ulikt store, trykket inn i plata.
    for k, (y, z, r) in enumerate(((0.13, 1.41, 0.055), (-0.09, 1.43, 0.04), (-0.18, 1.35, 0.03))):
        ellipsoid(f"eye{k}", (0.64, y, z), (r * 2, r * 2, r * 2), m["glow_eye"], 12, 8)
        ellipsoid(f"socket{k}", (0.62, y, z), (r * 2.6, r * 2.6, r * 2.6), m["flesh"], 10, 6)
    # Armene: tjukke overarmer, tynnere underarmer, hender med tre klør.
    for side, sy in (("L", 1), ("R", -1)):
        limb(f"arm{side}", [f"upper_arm.{side}", f"forearm.{side}"], [0.06, 0.045], m["cable"])
        ellipsoid(f"shoulder{side}", BONE_AT[f"upper_arm.{side}"][0], (0.16, 0.16, 0.14), m["flesh"], 10, 6)
        hand = BONE_AT[f"hand.{side}"][0]
        ellipsoid(f"palm{side}", hand + Vector((0.05, 0, -0.04)), (0.14, 0.1, 0.07), m["cable"], 10, 6)
        for k in range(3):
            claw(f"claw{side}{k}", hand + Vector((0.08, (k - 1) * 0.04 * sy, -0.05)), (1, 0.15 * (k - 1) * sy, -0.2), (0, 0, -1), m["cable"])
        # Beina: lår, legg og fot med to tåklør; kne og ankel som kuler.
        limb(f"leg{side}", [f"thigh.{side}", f"shin.{side}"], [0.075, 0.05], m["cable"])
        ellipsoid(f"hip{side}", BONE_AT[f"thigh.{side}"][0], (0.18, 0.16, 0.16), m["flesh"], 10, 6)
        ankle, toe = BONE_AT[f"foot.{side}"]
        mid = (ankle + toe) * 0.5
        d = toe - ankle
        box(f"foot{side}", mid + Vector((0, 0, 0.03)), (d.length, 0.11, 0.07), m["cable"], rot=(0, math.atan2(d.z, d.x) * -1, 0))
        for k in (-1, 1):
            claw(f"toe{side}{k}", toe + Vector((-0.03, k * 0.03 * sy, 0.05)), (1, 0, -0.3), (0, 0, -1), m["cable"], length=0.12, r=0.018)
    # Halen: en kabel med plugg i enden.
    limb("tail", ["tail1", "tail2", "tail3"], [0.035, 0.028, 0.022], m["cable"])
    end = BONE_AT["tail3"][1]
    box("plug", end + Vector((-0.06, -0.01, 0.0)), (0.12, 0.07, 0.06), m["plug"])
    for dy in (-0.02, 0.02):
        box(f"pin{dy}", end + Vector((-0.14, -0.01 + dy, 0.0)), (0.05, 0.008, 0.012), m["plug"])


def mutant_button_clips(arm):
    """Klippene. Dreiningene er i armaturrommet: y er nikk (positiv = framover
    ned for det som peker mot +X), z er sving, x er sidelengs helling."""
    # Idle: puster, ser seg rundt, kjeven henger og halen vagger.
    c = Clip(arm, "idle", 48, FPS)
    c.rest_all(0)
    for f, v in wave(48, 48, 1.0):
        c.rot("chest", f, y=-3 * v)
        c.scale("chest", f, 1.0 + 0.02 * v)
        c.rot("head", f, z=9 * v, y=-3 * abs(v))
        c.rot("jaw", f, y=10 + 7 * v)
        c.rot("upper_arm.L", f, x=4 * v)
        c.rot("upper_arm.R", f, x=-4 * v)
    for f, v in wave(48, 48, 1.0, phase=0.9):
        c.rot("tail1", f, z=18 * v)
    for f, v in wave(48, 48, 1.0, phase=0.3):
        c.rot("tail2", f, z=22 * v)
        c.rot("tail3", f, z=20 * v)
    c.finish()
    # Walk: beina i motfase, armene mot, kroppen dupper.
    c = Clip(arm, "walk", 24, FPS)
    c.rest_all(0)
    for f, v in wave(24, 24, 1.0, step=3):
        c.rot("thigh.L", f, y=38 * v)
        c.rot("thigh.R", f, y=-38 * v)
        c.rot("upper_arm.L", f, y=-28 * v)
        c.rot("upper_arm.R", f, y=28 * v)
        c.rot("head", f, z=4 * v)
        c.rot("tail1", f, z=14 * v)
        c.rot("tail2", f, z=18 * v)
    # Kneet bøyes når beinet svinger fram (legg bakover = +y).
    for f, v in wave(24, 24, 1.0, step=3, phase=math.pi / 2):
        c.rot("shin.L", f, y=18 + 20 * max(0.0, -v) * 2)
        c.rot("shin.R", f, y=18 + 20 * max(0.0, v) * 2)
    for f, v in wave(24, 12, 1.0, step=3):
        c.loc("root", f, (0, 0, 0.025 * v))
        c.rot("chest", f, y=3 * v)
    c.finish()
    # Attack: reiser seg og gaper (varselet), stuper fram og smeller
    # kjeven igjen ved bilde 15, og samler seg.
    c = Clip(arm, "attack", 30, FPS)
    c.rest_all(0)
    for bone, keys in {
        "root": [(10, "loc", (-0.12, 0, 0.05)), (15, "loc", (0.55, 0, -0.05)), (22, "loc", (0.4, 0, 0.0)), (30, "loc", (0, 0, 0))],
        "hips": [(10, "rot", (0, -22, 0)), (15, "rot", (0, 32, 0)), (22, "rot", (0, 12, 0)), (30, "rot", (0, 0, 0))],
        "chest": [(10, "rot", (0, -18, 0)), (15, "rot", (0, 15, 0)), (22, "rot", (0, 5, 0)), (30, "rot", (0, 0, 0))],
        "head": [(10, "rot", (0, -28, 0)), (15, "rot", (0, 14, 0)), (22, "rot", (0, 4, 0)), (30, "rot", (0, 0, 0))],
        "jaw": [(10, "rot", (0, 45, 0)), (14, "rot", (0, 40, 0)), (15, "rot", (0, 0, 0)), (22, "rot", (0, 8, 0)), (30, "rot", (0, 10, 0))],
        "upper_arm.L": [(10, "rot", (35, -70, 0)), (15, "rot", (10, 45, 0)), (22, "rot", (5, 20, 0)), (30, "rot", (0, 0, 0))],
        "upper_arm.R": [(10, "rot", (-35, -70, 0)), (15, "rot", (-10, 45, 0)), (22, "rot", (-5, 20, 0)), (30, "rot", (0, 0, 0))],
        "forearm.L": [(10, "rot", (0, -40, 0)), (15, "rot", (0, 25, 0)), (30, "rot", (0, 0, 0))],
        "forearm.R": [(10, "rot", (0, -40, 0)), (15, "rot", (0, 25, 0)), (30, "rot", (0, 0, 0))],
        "thigh.L": [(10, "rot", (0, 15, 0)), (15, "rot", (0, -25, 0)), (30, "rot", (0, 0, 0))],
        "thigh.R": [(10, "rot", (0, 15, 0)), (15, "rot", (0, -25, 0)), (30, "rot", (0, 0, 0))],
        "tail1": [(10, "rot", (0, -20, 0)), (15, "rot", (0, 25, 0)), (30, "rot", (0, 0, 0))],
        "tail2": [(10, "rot", (0, -25, 0)), (15, "rot", (0, 30, 0)), (30, "rot", (0, 0, 0))],
    }.items():
        for f, kind, v in keys:
            if kind == "loc":
                c.loc(bone, f, v)
            else:
                c.rot(bone, f, *v)
    c.finish()
    # Hit: rykker bakover, hodet slenges til sida.
    c = Clip(arm, "hit", 12, FPS)
    c.rest_all(0)
    c.loc("root", 3, (-0.14, 0, 0.0))
    c.rot("chest", 3, y=-22)
    c.rot("head", 3, z=28, y=-10)
    c.rot("jaw", 3, y=30)
    c.rot("upper_arm.L", 3, y=-25)
    c.rot("upper_arm.R", 3, y=-25)
    c.loc("root", 12, (0, 0, 0))
    for b in ("chest", "head", "jaw", "upper_arm.L", "upper_arm.R"):
        c.rot(b, 12)
    c.finish()
    # Die: skriker, beina svikter, segner om på sida og blir liggende.
    c = Clip(arm, "die", 36, FPS)
    c.rest_all(0)
    c.rot("head", 8, y=-25)
    c.rot("jaw", 8, y=45)
    c.rot("chest", 8, y=-12)
    c.rot("upper_arm.L", 8, x=30, y=-40)
    c.rot("upper_arm.R", 8, x=-30, y=-40)
    for b in ("thigh.L", "thigh.R"):
        c.rot(b, 20, y=55)
    for b in ("shin.L", "shin.R"):
        c.rot(b, 20, y=-75)
    c.loc("root", 20, (0.05, 0, -0.42))
    c.rot("hips", 20, y=20, x=20)
    c.rot("head", 20, y=25)
    c.rot("jaw", 20, y=20)
    c.loc("root", 36, (0.12, 0.1, -0.62))
    c.rot("hips", 36, y=8, x=78)
    c.rot("chest", 36, y=4)
    c.rot("head", 36, y=12, z=10)
    c.rot("jaw", 36, y=28)
    c.rot("upper_arm.L", 36, x=10, y=35)
    c.rot("upper_arm.R", 36, x=-60, y=20)
    c.rot("forearm.L", 36, y=20)
    c.rot("forearm.R", 36, y=-30)
    for b in ("thigh.L", "thigh.R"):
        c.rot(b, 36, y=70)
    for b in ("shin.L", "shin.R"):
        c.rot(b, 36, y=-80)
    c.rot("tail1", 36, y=18, z=0)
    c.rot("tail2", 36, y=10)
    c.rot("tail3", 36, y=6)
    c.finish()
    bpy.context.scene.frame_end = 48


def token(m):
    """Tokenmynten: skive 36 cm i diameter, 4 cm tykk, med glødende kant og
    en opphøyd ring på hver side."""
    cylinder("coin", (0.0, 0.0, 0.18), 0.18, 0.04, m["coin"], rot=(0, math.pi / 2, 0), vertices=24)
    cylinder("rim", (0.0, 0.0, 0.18), 0.185, 0.02, m["glow_rim"], rot=(0, math.pi / 2, 0), vertices=24)
    for sx in (1, -1):
        cylinder(f"ring{sx}", (sx * 0.022, 0.0, 0.18), 0.12, 0.01, m["glow_rim"], rot=(0, math.pi / 2, 0), vertices=20)
        cylinder(f"inner{sx}", (sx * 0.024, 0.0, 0.18), 0.1, 0.01, m["coin"], rot=(0, math.pi / 2, 0), vertices=20)


def build_mutant_button(size):
    m = mats()
    mutant_button(m)
    arm = armature("mutant_button_rig", BONES)
    mutant_button_clips(arm)
    bake_textured(f"{OUT}/mutant_button.glb", size, keep=("glass", "glow", "fire", "mesh"), armature=arm, rigid=RIGID, skin={"blend": 0.05, "radii": RADII})


def build_token(size):
    token(mats())
    bake_textured(f"{OUT}/token.glb", size, keep=("glass", "glow", "fire", "mesh"))


BUILDERS = {"mutant_button": (build_mutant_button, 1024), "token": (build_token, 256)}

if __name__ == "__main__":
    args = sys.argv[1:]
    kinds = [a for a in args if a in BUILDERS] or list(BUILDERS)
    size = next((int(a) for a in args if a.isdigit()), None)
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        build, default_size = BUILDERS[k]
        build(size or default_size)
