"""Fiendene og lootet i Legacy-kjelleren, modellert i Blender (bpy), rigget
med armatur og klipp, og bakt til teksturer (common.bake_textured). Kjør:

    python3 tools/models/mobs.py [mutant_button|mutant_tooltip|mutant_input|mutant_modal|mutant_table|framework|token|key|bolt ...] [størrelse]

Alle fiendene er komponenter som har mutert av for mye JavaScript og
forretningslogikk, og deler samme skjelett (BONES) og klippnavn (mob_motion.py),
men har hver sin kropp og sitt angrep. Origo i bakken under hoftene,
fronten mot +X; størrelsen settes per slag i `domain/mobs.rs`.

- `mutant_button`: `.fs-button`, et krokrygget tobeint monster av
  sammenfiltrede kabler med knappeplata som ansikt, kjeve full av tenner,
  tre glødende øyne, klør av krøllparenteser og kabelhale med plugg. Biter.
- `mutant_tooltip`: liten, rask, med tooltipboksen og pila som hode og ett
  stort øye. Går i flokk og biter.
- `mutant_input`: høy og tynn, med inputfeltet som hode, markøren som
  glødende klinge i gapet og markørspyd for hender. Spytter markører.
- `mutant_modal`: svær, med dialogboksen som hode, knappene i foten som
  never og bakteppet som skjold på ryggen. Slår med begge nevene.
- `mutant_table`: bred miniboss med tabellen som ryggskjold, kolonne-
  overskriften som hode og sorteringspilene som horn. Bærer nøkkelen.
- `framework`: Rammeverket, sluttbossen: et stillas av bjelker med en
  glødende kjerne, komponentplater sittende fast i buret, kjever av
  vinkelparenteser. Sveiper, slår i bakken (`slam`) og brøler (`roar`,
  kaller på tooltips).

Klippene lages i mob_motion.py, i stilen til hvert slag: idle og walk
(løkker), attack (treffet lander ved ATTACK_HIT, SLAM_HIT for slaget),
alert (får øye på deg), hit og hit2, stagger (svimmel, løkke), die og
die2, og for bossen slam og roar. `tools/models/clip_sheet.py` viser et
klipp som kontaktark.
`token` er en designtoken som mynt, `key` nøkkelen til portene og `bolt`
markøren inputen spytter."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import Clip, armature, bake_textured, box, clear, cylinder, ellipsoid, material, rounded_box, tube, weather  # noqa: E402
from mathutils import Vector  # noqa: E402
import mob_motion  # noqa: E402

OUT = "assets/models/mobs"
FPS = 24
# Sekundet i attack-klippet der bittet lander (bilde 15 av 30).
ATTACK_HIT = 15 / FPS
# Slaget med begge nevene (modal, bossens slam): bilde 28 av 44.
SLAM_HIT = 28 / FPS


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
        # Tooltip: mørk boks med lys tekst.
        "tip_box": material("tip_box", (0.1, 0.1, 0.12), 0.6),
        "tip_text": material("tip_text", (0.85, 0.85, 0.88), 0.5),
        # Input: hvitt felt med grå kant, grå plassholder og blå markør.
        "field": material("field", (0.9, 0.9, 0.92), 0.4),
        "border": material("border", (0.45, 0.47, 0.52), 0.55),
        "placeholder": material("placeholder", (0.55, 0.56, 0.6), 0.6),
        "glow_caret": material("glow_caret", (0.35, 0.6, 1.0), 0.3),
        # Modal: hvit dialog, mørk tittel, blå og grå knapp, svart bakteppe.
        "modal": material("modal", (0.93, 0.93, 0.95), 0.4),
        "modal_text": material("modal_text", (0.15, 0.15, 0.18), 0.6),
        "fist_primary": material("fist_primary", (0.07, 0.2, 0.52), 0.5),
        "fist_secondary": material("fist_secondary", (0.55, 0.57, 0.6), 0.5),
        "backdrop": material("backdrop", (0.02, 0.02, 0.025), 0.8),
        # Tabell: hvit plate, grå linjer, lys overskriftsrad.
        "table_top": material("table_top", (0.95, 0.95, 0.96), 0.45),
        "table_line": material("table_line", (0.6, 0.62, 0.68), 0.55),
        "table_head": material("table_head", (0.82, 0.85, 0.9), 0.45),
        # Rammeverket: mørkt stål, glødende kjerne, plater i komponentfarger.
        "beam": material("beam", (0.22, 0.22, 0.25), 0.45, 1.0),
        "glow_core": material("glow_core", (1.0, 0.35, 0.08), 0.3),
        "plate_red": material("plate_red", (0.6, 0.12, 0.1), 0.5),
        "plate_green": material("plate_green", (0.12, 0.45, 0.2), 0.5),
        "plate_grey": material("plate_grey", (0.7, 0.7, 0.72), 0.5),
        "rust": material("rust", (0.38, 0.22, 0.12), 0.75, 0.6),
    }
    weather(m["plate"], 0.9, 1.5, 0.5, 9.0)
    weather(m["face"], 0.9, 1.5, 0.35, 9.0)
    weather(m["flesh"], 0.0, 0.5, 0.3, 6.0)
    for k in ("field", "modal", "table_top", "table_head", "tip_box"):
        weather(m[k], 0.9, 1.6, 0.4, 9.0)
    weather(m["beam"], 0.6, 1.5, 0.5, 7.0)
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
RIGID = {"plate": "head", "face": "head", "eye": "head", "tooth_u": "head", "mouth": "head", "jaw": "jaw", "tooth_l": "jaw", "plug": "tail3", "pin": "tail3", "head_": "head", "jaw_": "jaw", "fistL": "hand.L", "fistR": "hand.R", "back_": "chest", "spearL": "hand.L", "spearR": "hand.R"}
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
    cone(f"{name}_tip", tip + direction * length * 0.12, r * 0.45, length * 0.25, mat, direction=direction)


def cone(name, location, radius, depth, mat, direction=(0, 0, 1), vertices=6):
    """En kjegle med spissen i `direction`."""
    d = Vector(direction).normalized()
    bpy.ops.mesh.primitive_cone_add(vertices=vertices, radius1=radius, radius2=0.0, depth=depth, location=Vector(location), rotation=d.to_track_quat("Z", "Y").to_euler())
    c = bpy.context.active_object
    c.name = name
    c.data.materials.append(mat)
    return c


def limb(name, bones, radii, mat):
    """Rør langs en kjede av bein, med kuler i leddene."""
    pts = [BONE_AT[bones[0]][0]] + [BONE_AT[b][1] for b in bones]
    chain(name, pts, radii, mat)


def teeth(m, prefix_u="tooth_u", prefix_l="tooth_l", n=7, y0=-0.18, dy=0.06, x=0.64, zu=1.2, zl=1.17, r=0.018):
    """Tenner i over- og underkant av gapet; de øvre henger på hodet, de
    nedre på kjeven."""
    for k in range(n):
        y = y0 + k * dy
        for prefix, z, up in ((prefix_u, zu, False), (prefix_l, zl, True)):
            h = 0.05 + 0.02 * (k % 2)
            cone(f"{prefix}{k}", (x, y + (0.02 if up else 0.0), z + (h * 0.5 if up else -h * 0.5)), r, h, m["tooth"], direction=(0, 0, 1 if up else -1))


def eyes(m, spec, mat=None, socket=True):
    """Glødende øyne trykket inn i hodet: (y, z, r) per øye."""
    for k, (y, z, r) in enumerate(spec):
        ellipsoid(f"head_eye{k}", (0.64, y, z), (r * 2, r * 2, r * 2), mat or m["glow_eye"], 12, 8)
        if socket:
            ellipsoid(f"head_socket{k}", (0.62, y, z), (r * 2.6, r * 2.6, r * 2.6), m["flesh"], 10, 6)


def body_cables(m, n=7, scale=1.0):
    """Kabler som snor seg rundt kroppen."""
    for k in range(n):
        pts = []
        for s in range(9):
            t = s / 8
            a = k * 0.9 + t * 4.5
            cx = -0.2 + t * 0.5
            cz = 0.92 + t * 0.2
            ry = (0.26 + 0.05 * math.sin(k + s)) * scale
            rz = (0.24 + 0.04 * math.cos(k * 2 + s)) * scale
            pts.append(Vector((cx, math.cos(a) * ry, cz + math.sin(a) * rz)))
        chain(f"cable{k}", pts, [0.022 + 0.004 * (k % 3)] * 8, m["cable"], joints=False)


def spikes(m, n=5, scale=1.0, x0=-0.22):
    """Pigger av } på ryggen."""
    for k in range(n):
        x = x0 + k * 0.1
        z = 1.18 + 0.1 * math.sin(k * 1.2) + k * 0.03
        b = Vector((x, 0, z))
        pts = [b, b + Vector((-0.03, 0.02, 0.08)) * scale, b + Vector((-0.08, 0.0, 0.12)) * scale, b + Vector((-0.1, -0.02, 0.17)) * scale]
        chain(f"spike{k}", pts, [0.02 * scale, 0.014 * scale, 0.008 * scale], m["cable"])


def arms(m, radii=(0.06, 0.045), shoulder=(0.16, 0.16, 0.14), claws=3, claw_len=0.2, mat=None, palm=True):
    mat = mat or m["cable"]
    for side, sy in (("L", 1), ("R", -1)):
        limb(f"arm{side}", [f"upper_arm.{side}", f"forearm.{side}"], list(radii), mat)
        ellipsoid(f"shoulder{side}", BONE_AT[f"upper_arm.{side}"][0], shoulder, m["flesh"], 10, 6)
        hand = BONE_AT[f"hand.{side}"][0]
        if palm:
            ellipsoid(f"palm{side}", hand + Vector((0.05, 0, -0.04)), (0.14, 0.1, 0.07), mat, 10, 6)
        for k in range(claws):
            claw(f"claw{side}{k}", hand + Vector((0.08, (k - 1) * 0.04 * sy, -0.05)), (1, 0.15 * (k - 1) * sy, -0.2), (0, 0, -1), mat, length=claw_len)


def legs(m, radii=(0.075, 0.05), hip=(0.18, 0.16, 0.16), foot=(0.11, 0.07), mat=None):
    mat = mat or m["cable"]
    for side, sy in (("L", 1), ("R", -1)):
        limb(f"leg{side}", [f"thigh.{side}", f"shin.{side}"], list(radii), mat)
        ellipsoid(f"hip{side}", BONE_AT[f"thigh.{side}"][0], hip, m["flesh"], 10, 6)
        ankle, toe = BONE_AT[f"foot.{side}"]
        mid = (ankle + toe) * 0.5
        d = toe - ankle
        box(f"foot{side}", mid + Vector((0, 0, 0.03)), (d.length, foot[0], foot[1]), mat, rot=(0, math.atan2(d.z, d.x) * -1, 0))
        for k in (-1, 1):
            claw(f"toe{side}{k}", toe + Vector((-0.03, k * 0.03 * sy, 0.05)), (1, 0, -0.3), (0, 0, -1), mat, length=0.12, r=0.018)


def tail(m, radii=(0.035, 0.028, 0.022), plug=True):
    limb("tail", ["tail1", "tail2", "tail3"], list(radii), m["cable"])
    if plug:
        end = BONE_AT["tail3"][1]
        box("plug", end + Vector((-0.06, -0.01, 0.0)), (0.12, 0.07, 0.06), m["plug"])
        for dy in (-0.02, 0.02):
            box(f"pin{dy}", end + Vector((-0.14, -0.01 + dy, 0.0)), (0.05, 0.008, 0.012), m["plug"])


def mutant_button(m):
    # Kroppen: to sammenvokste klumper, hofter og bryst, i kabelsvart.
    ellipsoid("belly", (-0.07, 0, 0.92), (0.5, 0.52, 0.46), m["flesh"], 20, 12)
    ellipsoid("chest", (0.12, 0, 1.1), (0.56, 0.62, 0.5), m["flesh"], 20, 12)
    ellipsoid("neckbulge", (0.3, 0, 1.22), (0.26, 0.3, 0.22), m["flesh"], 14, 8)
    body_cables(m)
    # Råteflekker.
    for k, (x, y, z, s) in enumerate([(0.2, 0.25, 1.15, 0.12), (-0.2, -0.2, 0.95, 0.1), (0.05, -0.3, 1.2, 0.09), (-0.1, 0.27, 1.05, 0.08)]):
        ellipsoid(f"rot{k}", (x, y, z), (s * 1.4, s * 0.9, s * 0.8), m["rot"], 10, 6)
    spikes(m)
    # Hodet: knappeplata som ansikt, forsida mot +X, med ei lysere flate.
    rounded_box("plate", (0.57, 0, 1.33), (0.16, 0.5, 0.26), m["plate"], 0.035, segments=3)
    rounded_box("face", (0.655, 0, 1.335), (0.02, 0.44, 0.2), m["face"], 0.02, segments=2)
    # Underkjeven: nederste del av plata, som har løsnet.
    rounded_box("jaw", (0.57, 0, 1.11), (0.17, 0.46, 0.11), m["plate"], 0.025, segments=2)
    rounded_box("jawface", (0.657, 0, 1.11), (0.02, 0.4, 0.07), m["face"], 0.015, segments=2)
    box("jawroot", (0.46, 0, 1.11), (0.08, 0.3, 0.1), m["flesh"])
    # Gapet: mørkt inni, tenner i over- og underkant.
    box("mouth", (0.56, 0, 1.19), (0.14, 0.42, 0.04), m["mouth"])
    teeth(m)
    # Øynene: tre glødende kuler, ulikt store, trykket inn i plata.
    eyes(m, ((0.13, 1.41, 0.055), (-0.09, 1.43, 0.04), (-0.18, 1.35, 0.03)))
    arms(m)
    legs(m)
    tail(m)


def mutant_tooltip(m):
    """Liten og rask: tooltipboksen med pila som hode, ett stort øye."""
    ellipsoid("belly", (-0.05, 0, 0.95), (0.42, 0.44, 0.4), m["flesh"], 18, 10)
    ellipsoid("chest", (0.12, 0, 1.1), (0.42, 0.46, 0.38), m["flesh"], 18, 10)
    body_cables(m, n=4, scale=0.8)
    spikes(m, n=3, scale=0.8, x0=-0.18)
    rounded_box("head_box", (0.57, 0, 1.33), (0.2, 0.44, 0.22), m["tip_box"], 0.04, segments=3)
    # Pila på toppen av boksen, og to tekstlinjer på forsida.
    cone("head_arrow", (0.5, 0, 1.5), 0.09, 0.14, m["tip_box"], vertices=4)
    box("head_text0", (0.675, 0.03, 1.4), (0.01, 0.26, 0.025), m["tip_text"])
    box("head_text1", (0.675, -0.04, 1.34), (0.01, 0.16, 0.025), m["tip_text"])
    rounded_box("jaw_box", (0.56, 0, 1.13), (0.18, 0.4, 0.1), m["tip_box"], 0.03, segments=2)
    box("jawroot", (0.46, 0, 1.12), (0.08, 0.26, 0.1), m["flesh"])
    box("head_mouth", (0.56, 0, 1.2), (0.14, 0.36, 0.04), m["mouth"])
    teeth(m, "head_tooth", "jaw_tooth", n=6, y0=-0.15, r=0.015)
    eyes(m, ((0.04, 1.4, 0.08),))
    arms(m, radii=(0.04, 0.03), shoulder=(0.12, 0.12, 0.1), claw_len=0.16)
    legs(m, radii=(0.05, 0.035), hip=(0.13, 0.12, 0.12), foot=(0.09, 0.06))
    tail(m, radii=(0.022, 0.018, 0.014), plug=False)


def mutant_input(m):
    """Høy og tynn: inputfeltet som hode, markøren som glødende klinge i
    gapet, markørspyd for hender."""
    ellipsoid("belly", (-0.05, 0, 0.92), (0.36, 0.4, 0.5), m["flesh"], 18, 10)
    ellipsoid("chest", (0.1, 0, 1.12), (0.4, 0.46, 0.56), m["flesh"], 18, 10)
    body_cables(m, n=4, scale=0.75)
    spikes(m, n=4)
    rounded_box("head_border", (0.56, 0, 1.33), (0.13, 0.64, 0.3), m["border"], 0.03, segments=3)
    rounded_box("head_field", (0.57, 0, 1.33), (0.14, 0.6, 0.26), m["field"], 0.03, segments=3)
    # Plassholderteksten og markøren i feltet.
    box("head_ph0", (0.645, 0.12, 1.4), (0.01, 0.24, 0.025), m["placeholder"])
    box("head_ph1", (0.645, 0.2, 1.33), (0.01, 0.1, 0.025), m["placeholder"])
    box("head_caret", (0.65, -0.2, 1.34), (0.015, 0.02, 0.2), m["glow_caret"])
    rounded_box("jaw_border", (0.56, 0, 1.1), (0.14, 0.6, 0.13), m["border"], 0.03, segments=2)
    rounded_box("jaw_field", (0.57, 0, 1.1), (0.15, 0.56, 0.1), m["field"], 0.025, segments=2)
    box("jawroot", (0.46, 0, 1.11), (0.08, 0.3, 0.1), m["flesh"])
    box("head_mouth", (0.56, 0, 1.19), (0.14, 0.52, 0.04), m["mouth"])
    # Tennene er loddrette streker, som markører.
    for k in range(9):
        y = -0.22 + k * 0.055
        box(f"head_tooth{k}", (0.64, y, 1.215), (0.012, 0.012, 0.05), m["tooth"])
        box(f"jaw_tooth{k}", (0.64, y + 0.027, 1.16), (0.012, 0.012, 0.04), m["tooth"])
    # Øynene: to glødende slisser.
    for k, y in enumerate((0.18, -0.08)):
        box(f"head_eye{k}", (0.648, y, 1.45), (0.01, 0.09, 0.02), m["glow_caret"])
    arms(m, radii=(0.045, 0.035), shoulder=(0.13, 0.13, 0.11), claws=0, palm=False)
    for side, sy in (("L", 1), ("R", -1)):
        hand = BONE_AT[f"hand.{side}"][0]
        ellipsoid(f"palm{side}", hand + Vector((0.04, 0, -0.03)), (0.12, 0.09, 0.07), m["cable"], 10, 6)
        box(f"spear{side}", hand + Vector((0.3, 0, -0.1)), (0.5, 0.025, 0.05), m["glow_caret"], rot=(0, 0.25, 0))
        box(f"spear{side}_sock", hand + Vector((0.1, 0, -0.06)), (0.12, 0.05, 0.08), m["cable"], rot=(0, 0.25, 0))
    legs(m, radii=(0.06, 0.04), hip=(0.15, 0.13, 0.13))
    tail(m)


def mutant_modal(m):
    """Svær: dialogboksen som hode, knappene i foten som never, bakteppet
    som skjold på ryggen."""
    ellipsoid("belly", (-0.07, 0, 0.9), (0.62, 0.7, 0.55), m["flesh"], 22, 12)
    ellipsoid("chest", (0.12, 0, 1.12), (0.72, 0.8, 0.62), m["flesh"], 22, 12)
    ellipsoid("neckbulge", (0.32, 0, 1.24), (0.3, 0.4, 0.26), m["flesh"], 14, 8)
    body_cables(m, n=8, scale=1.3)
    for k, (x, y, z, s) in enumerate([(0.2, 0.3, 1.2, 0.14), (-0.2, -0.28, 0.95, 0.12), (0.05, -0.36, 1.25, 0.1)]):
        ellipsoid(f"rot{k}", (x, y, z), (s * 1.4, s * 0.9, s * 0.8), m["rot"], 10, 6)
    # Bakteppet: en mørk, avrundet plate på ryggen.
    rounded_box("back_shell", (-0.32, 0, 1.12), (0.1, 0.95, 0.75), m["backdrop"], 0.06, rot=(0, 0.15, 0), segments=3)
    rounded_box("head_modal", (0.57, 0, 1.36), (0.16, 0.64, 0.4), m["modal"], 0.04, segments=3)
    # Tittellinja og krysset øverst, gapet nederst i dialogen.
    box("head_title", (0.655, 0.1, 1.5), (0.01, 0.36, 0.045), m["modal_text"])
    for k, r in enumerate((0.785, -0.785)):
        box(f"head_x{k}", (0.655, -0.25, 1.5), (0.01, 0.1, 0.016), m["modal_text"], rot=(r, 0, 0))
    box("head_mouth", (0.56, 0, 1.24), (0.14, 0.56, 0.05), m["mouth"])
    teeth(m, "head_tooth", "jaw_tooth", n=9, y0=-0.24, zu=1.26, zl=1.2, r=0.02)
    rounded_box("jaw_box", (0.57, 0, 1.1), (0.17, 0.6, 0.14), m["modal"], 0.03, segments=2)
    box("jawroot", (0.46, 0, 1.11), (0.08, 0.36, 0.12), m["flesh"])
    # Knappene i foten av dialogen.
    box("jaw_btn0", (0.66, -0.16, 1.06), (0.01, 0.18, 0.06), m["fist_primary"])
    box("jaw_btn1", (0.66, 0.1, 1.06), (0.01, 0.18, 0.06), m["fist_secondary"])
    eyes(m, ((0.2, 1.42, 0.045), (-0.2, 1.42, 0.045)))
    arms(m, radii=(0.09, 0.075), shoulder=(0.24, 0.24, 0.2), claws=0, palm=False)
    for side, mat in (("L", m["fist_primary"]), ("R", m["fist_secondary"])):
        hand = BONE_AT[f"hand.{side}"][0]
        rounded_box(f"fist{side}", hand + Vector((0.08, 0, -0.08)), (0.26, 0.22, 0.2), mat, 0.05, segments=3)
    legs(m, radii=(0.1, 0.08), hip=(0.24, 0.22, 0.22), foot=(0.15, 0.09))
    tail(m, radii=(0.045, 0.036, 0.028))


def mutant_table(m):
    """Bred miniboss: tabellen som ryggskjold, kolonneoverskriften som hode,
    sorteringspilene som horn."""
    ellipsoid("belly", (-0.05, 0, 0.9), (0.6, 0.8, 0.5), m["flesh"], 22, 12)
    ellipsoid("chest", (0.12, 0, 1.1), (0.66, 0.9, 0.52), m["flesh"], 22, 12)
    body_cables(m, n=6, scale=1.4)
    # Ryggskjoldet: tabellplata med rutenett og overskriftsrad fremst.
    rounded_box("back_top", (-0.05, 0, 1.42), (0.9, 1.0, 0.05), m["table_top"], 0.02, rot=(0, -0.17, 0), segments=2)
    for k in range(4):
        y = -0.375 + k * 0.25
        box(f"back_liney{k}", (-0.05, y, 1.455), (0.9, 0.012, 0.012), m["table_line"], rot=(0, -0.17, 0))
    for k in range(3):
        x = -0.35 + k * 0.3
        box(f"back_linex{k}", (x, 0, 1.455 - (x + 0.05) * 0.17), (0.012, 1.0, 0.012), m["table_line"], rot=(0, -0.17, 0))
    box("back_head", (0.36, 0, 1.49), (0.18, 1.0, 0.055), m["table_head"], rot=(0, -0.17, 0))
    rounded_box("head_box", (0.57, 0, 1.33), (0.16, 0.6, 0.22), m["table_head"], 0.03, segments=3)
    box("head_text", (0.655, 0.04, 1.4), (0.01, 0.3, 0.03), m["modal_text"])
    for k, y in enumerate((0.22, -0.22)):
        cone(f"head_horn{k}", (0.5, y, 1.52), 0.05, 0.16, m["table_line"], vertices=4)
    box("head_mouth", (0.56, 0, 1.2), (0.14, 0.5, 0.04), m["mouth"])
    teeth(m, "head_tooth", "jaw_tooth", n=8, y0=-0.21, r=0.02)
    rounded_box("jaw_box", (0.57, 0, 1.1), (0.17, 0.56, 0.11), m["table_top"], 0.025, segments=2)
    box("jawroot", (0.46, 0, 1.11), (0.08, 0.34, 0.1), m["flesh"])
    eyes(m, ((0.18, 1.42, 0.045), (-0.18, 1.42, 0.045), (0.07, 1.45, 0.03), (-0.07, 1.45, 0.03)))
    arms(m, radii=(0.07, 0.055), shoulder=(0.2, 0.2, 0.17), claw_len=0.24)
    legs(m, radii=(0.09, 0.065), hip=(0.22, 0.2, 0.2), foot=(0.13, 0.08))
    tail(m, radii=(0.04, 0.032, 0.025))


def framework(m):
    """Rammeverket: et stillas av bjelker rundt en glødende kjerne, med
    komponentplater sittende fast i buret og kjever av vinkelparenteser."""
    ellipsoid("belly", (-0.05, 0, 0.92), (0.46, 0.5, 0.42), m["flesh"], 18, 10)
    ellipsoid("core", (0.08, 0, 1.08), (0.4, 0.4, 0.4), m["glow_core"], 14, 8)
    # Buret: fire stolper og tre ringer av bjelker.
    for k, (x, y) in enumerate(((0.3, 0.35), (0.3, -0.35), (-0.25, 0.4), (-0.25, -0.4))):
        box(f"beam_v{k}", (x, y, 1.05), (0.06, 0.06, 0.64), m["beam"])
    for k, z in enumerate((0.76, 1.05, 1.34)):
        for name, c, s in (("f", (0.3, 0, z), (0.05, 0.76, 0.05)), ("b", (-0.25, 0, z), (0.05, 0.86, 0.05)), ("l", (0.02, 0.38, z), (0.6, 0.05, 0.05)), ("r", (0.02, -0.38, z), (0.6, 0.05, 0.05))):
            box(f"beam_{name}{k}", c, s, m["beam"], rot=(0, 0, 0.065 if name in "lr" else 0))
    # Plater som sitter fast i buret.
    rounded_box("plate_badge", (0.33, 0.2, 1.2), (0.03, 0.2, 0.1), m["plate"], 0.015)
    rounded_box("plate_tag", (0.3, -0.22, 0.95), (0.03, 0.18, 0.08), m["plate_grey"], 0.02, rot=(0.2, 0, 0.1))
    rounded_box("plate_alert", (-0.3, 0.25, 1.15), (0.03, 0.24, 0.12), m["plate_red"], 0.015, rot=(0, 0, 0.2))
    rounded_box("plate_ok", (-0.05, -0.42, 1.2), (0.16, 0.03, 0.1), m["plate_green"], 0.015, rot=(0.1, 0, 0))
    rounded_box("plate_card", (0.05, 0.42, 0.9), (0.2, 0.03, 0.14), m["plate_grey"], 0.015)
    spikes(m, n=6, scale=1.5, x0=-0.3)
    # Hodet: en bjelkeboks med bryn og vinkelparenteser på sidene.
    rounded_box("head_box", (0.55, 0, 1.35), (0.22, 0.5, 0.22), m["beam"], 0.02, segments=2)
    box("head_brow", (0.6, 0, 1.47), (0.14, 0.58, 0.05), m["beam"])
    for side, sy in (("L", 1), ("R", -1)):
        for k, (dz, rx) in enumerate(((0.06, 0.6), (-0.06, -0.6))):
            box(f"head_lt{side}{k}", (0.62, sy * 0.3, 1.36 + dz), (0.02, 0.03, 0.12), m["beam"], rot=(rx * sy, 0, 0))
    box("head_mouth", (0.56, 0, 1.22), (0.14, 0.44, 0.05), m["mouth"])
    teeth(m, "head_tooth", "jaw_tooth", n=8, y0=-0.2, zu=1.24, zl=1.19, r=0.022)
    rounded_box("jaw_box", (0.55, 0, 1.1), (0.22, 0.48, 0.12), m["beam"], 0.02, segments=2)
    box("jawroot", (0.44, 0, 1.11), (0.08, 0.3, 0.1), m["flesh"])
    eyes(m, ((0.16, 1.4, 0.05), (-0.16, 1.4, 0.05), (0.0, 1.43, 0.035), (0.22, 1.3, 0.03), (-0.22, 1.3, 0.03)), mat=m["glow_core"])
    arms(m, radii=(0.08, 0.065), shoulder=(0.22, 0.22, 0.18), claw_len=0.3, mat=m["beam"])
    legs(m, radii=(0.1, 0.075), hip=(0.24, 0.22, 0.22), foot=(0.14, 0.09), mat=m["beam"])
    tail(m, radii=(0.045, 0.036, 0.028))


def token(m):
    """Tokenmynten: skive 36 cm i diameter, 4 cm tykk, med glødende kant og
    en opphøyd ring på hver side."""
    cylinder("coin", (0.0, 0.0, 0.18), 0.18, 0.04, m["coin"], rot=(0, math.pi / 2, 0), vertices=24)
    cylinder("rim", (0.0, 0.0, 0.18), 0.185, 0.02, m["glow_rim"], rot=(0, math.pi / 2, 0), vertices=24)
    for sx in (1, -1):
        cylinder(f"ring{sx}", (sx * 0.022, 0.0, 0.18), 0.12, 0.01, m["glow_rim"], rot=(0, math.pi / 2, 0), vertices=20)
        cylinder(f"inner{sx}", (sx * 0.024, 0.0, 0.18), 0.1, 0.01, m["coin"], rot=(0, math.pi / 2, 0), vertices=20)


def key(m):
    """Nøkkelen til de rustne portene: 30 cm lang, rustent jern, med et
    rundt grep og to tenner. Origo i bakken under midten."""
    cylinder("shaft", (0.0, 0.0, 0.06), 0.014, 0.22, m["rust"], rot=(0, math.pi / 2, 0), vertices=10)
    bpy.ops.mesh.primitive_torus_add(location=(-0.15, 0.0, 0.06), rotation=(0, 0, 0), major_radius=0.05, minor_radius=0.012, major_segments=16, minor_segments=8)
    t = bpy.context.active_object
    t.name = "bow"
    t.data.materials.append(m["rust"])
    for i, (x, h) in enumerate(((0.07, 0.05), (0.1, 0.035))):
        box(f"bit{i}", (x, 0.0, 0.06 - h * 0.5 - 0.01), (0.02, 0.012, h), m["rust"])


def bolt(m):
    """Markøren inputen spytter: en glødende loddrett strek med mørk kjerne,
    30 cm høy, origo i midten, flyr mot +X."""
    box("core", (0.0, 0.0, 0.0), (0.05, 0.03, 0.3), m["cable"])
    box("glow", (0.0, 0.0, 0.0), (0.03, 0.045, 0.32), m["glow_caret"])
    cone("tip", (0.05, 0.0, 0.0), 0.03, 0.08, m["glow_caret"], direction=(1, 0, 0), vertices=6)


def build_rigged(name, body, attack, extra=(), style="button"):
    def build(size):
        m = mats()
        body(m)
        arm = armature(f"{name}_rig", BONES)
        # Klippene i slagets stil (mob_motion.py), med IK for føttene.
        mob_motion.make_clips(Clip, arm, BONES, style, attack, extra)
        bpy.context.scene.frame_end = 64
        bake_textured(f"{OUT}/{name}.glb", size, keep=("glass", "glow", "fire", "mesh"), armature=arm, rigid=RIGID, skin={"blend": 0.05, "radii": RADII})

    return build


def build_plain(name, body):
    def build(size):
        body(mats())
        bake_textured(f"{OUT}/{name}.glb", size, keep=("glass", "glow", "fire", "mesh"))

    return build


BUILDERS = {
    "mutant_button": (build_rigged("mutant_button", mutant_button, "bite", style="button"), 1024),
    "mutant_tooltip": (build_rigged("mutant_tooltip", mutant_tooltip, "bite", style="tooltip"), 512),
    "mutant_input": (build_rigged("mutant_input", mutant_input, "spit", style="input"), 512),
    "mutant_modal": (build_rigged("mutant_modal", mutant_modal, "slam", style="modal"), 1024),
    "mutant_table": (build_rigged("mutant_table", mutant_table, "bite", style="table"), 512),
    "framework": (build_rigged("framework", framework, "bite", ("slam", "roar"), style="framework"), 1024),
    "token": (build_plain("token", token), 256),
    "key": (build_plain("key", key), 256),
    "bolt": (build_plain("bolt", bolt), 128),
}

if __name__ == "__main__":
    args = sys.argv[1:]
    kinds = [a for a in args if a in BUILDERS] or list(BUILDERS)
    size = next((int(a) for a in args if a.isdigit()), None)
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        build, default_size = BUILDERS[k]
        build(size or default_size)
