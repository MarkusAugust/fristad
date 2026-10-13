"""Monstrene i krypten, bygd etter eierens ark (`assets/textures/monsters/ref/`,
`docs/prompter-monstre.md`): formen modelleres her, og der arket viser en
flate rett forfra eller bakfra (Button Wights steinplate), legges arket selv
på som bilde (uv-laget «src», bakt inn av `common.bake_textured`). Skjelettet
har beinnavnene og klippene fra `mob_motion.py`, men står der dette monsteret
har armer og bein. Kjør:

    python3.13 tools/models/monsters.py [mutant_button ...] [størrelse]

Origo i bakken under hoftene, fronten mot +X (Blender), som de andre
fiendene; motoren gir navnet til slaget i `domain/mobs.rs` (`Kind::model`)."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import Clip, armature, bake_textured, box, clear, cylinder, ellipsoid, material, rounded_box, tube, weather  # noqa: E402
from mathutils import Vector  # noqa: E402
import mob_motion  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REF = os.path.join(ROOT, "assets", "textures", "monsters", "ref")
OUT = "assets/models/mobs"


def sheet_material(name, path, rough=0.8):
    """Et materiale som viser forbildet etter uv-laget «src»."""
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = rough
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = bpy.data.images.load(path)
    tex.interpolation = "Cubic"
    uvn = nt.nodes.new("ShaderNodeUVMap")
    uvn.uv_map = "src"
    nt.links.new(uvn.outputs[0], tex.inputs[0])
    nt.links.new(tex.outputs[0], bsdf.inputs["Base Color"])
    return m


def rounded_rect(w, h, r, n=6):
    """Omrisset av et avrundet rektangel (y, z) rundt origo, mot klokka."""
    pts = []
    for cy, cz, a0 in ((w / 2 - r, h / 2 - r, 0.0), (-w / 2 + r, h / 2 - r, 90.0), (-w / 2 + r, -h / 2 + r, 180.0), (w / 2 - r, -h / 2 + r, 270.0)):
        for k in range(n + 1):
            a = math.radians(a0 + 90.0 * k / n)
            pts.append((cy + r * math.cos(a), cz + r * math.sin(a)))
    return pts


def bubble(w, h, r, pw, pd, n=6):
    """Omrisset av en snakkeboble: et avrundet rektangel med en spiss ned
    midt på (bredde `pw`, dybde `pd`)."""
    pts = rounded_rect(w, h, r, n)
    k = 3 * (n + 1)  # nederst til høyre begynner her (270°)
    tip = [(-pw / 2, -h / 2), (0.0, -h / 2 - pd), (pw / 2, -h / 2)]
    return pts[:k] + tip + pts[k:]


def slab_from_sheet(name, mat, profile, t, zc, front, back, side, img_size, bevel=0.025):
    """En plate (omrisset `profile` i (y, z) rundt z = `zc`, tykkelse `t`
    langs x) med forsida (+X) og baksida (−X) fra rutene `front` og `back`
    i arket og kantene fra `side` (pikselrektangler x0, y0, x1, y1 rundt
    omrissets ytre mål). Arket forfra har +Y til høyre, bakfra til venstre,
    og fra siden står forsida til venstre."""
    W, H = img_size
    bm = bmesh.new()
    base = bm.loops.layers.uv.new("UVMap")
    src = bm.loops.layers.uv.new("src")
    ys = [p[0] for p in profile]
    zs = [p[1] + zc for p in profile]
    y0, y1, zb, zt = min(ys), max(ys), min(zs), max(zs)
    w, h = y1 - y0, zt - zb

    def at(rect, fu, fv):
        x0, py0, x1, py1 = rect
        return ((x0 + fu * (x1 - x0)) / W, 1.0 - (py1 - fv * (py1 - py0)) / H)

    fr = [bm.verts.new((t / 2, y, zc + z)) for y, z in profile]
    bk = [bm.verts.new((-t / 2, y, zc + z)) for y, z in profile]
    f = bm.faces.new(fr)
    for lp in f.loops:
        c = lp.vert.co
        lp[src].uv = at(front, (c.y - y0) / w, (c.z - zb) / h)
        lp[base].uv = (c.y, c.z)
    f = bm.faces.new(bk[::-1])
    for lp in f.loops:
        c = lp.vert.co
        lp[src].uv = at(back, (y1 - c.y) / w, (c.z - zb) / h)
        lp[base].uv = (c.y, c.z)
    n = len(profile)
    for i in range(n):
        j = (i + 1) % n
        f = bm.faces.new((fr[i], bk[i], bk[j], fr[j]))
        dy = abs(profile[j][0] - profile[i][0])
        dz = abs(profile[j][1] - profile[i][1])
        for lp in f.loops:
            c = lp.vert.co
            fu = (t / 2 - c.x) / t
            if dz / h >= dy / w:
                lp[src].uv = at(side, fu, (c.z - zb) / h)
            else:
                # Oppe og nede: et smalt bånd øverst i sideruta.
                lp[src].uv = at(side, fu, 0.97 - 0.05 * (c.y - y0) / w)
            lp[base].uv = (c.x, c.y + c.z)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    me.materials.append(mat)
    bpy.context.scene.collection.objects.link(ob)
    mod = ob.modifiers.new("bevel", "BEVEL")
    mod.width = bevel
    mod.segments = 2
    mod.limit_method = "ANGLE"
    return ob


# --- Button Wight -----------------------------------------------------------
#
# Arket (`button_wight_body.jpeg`, 2912 × 1440): forfra, fra siden og bakfra.
# Figuren er 894 px høy (282–1176) og 1,5 m i spillet; steinplata er nesten
# hele kroppen, med korte beinbein under og tynne beinarmer ut fra sidene.

WIGHT_SHEET = os.path.join(REF, "button_wight_body.jpeg")
WIGHT_PX = 1.5 / (1176 - 282)
WIGHT_FRONT = (245, 370, 957, 994)
WIGHT_BACK = (2013, 391, 2687, 977)
WIGHT_SIDE = (1290, 378, 1705, 967)
WIGHT_W = (WIGHT_FRONT[2] - WIGHT_FRONT[0]) * WIGHT_PX
WIGHT_H = (WIGHT_FRONT[3] - WIGHT_FRONT[1]) * WIGHT_PX
WIGHT_ZB = (1176 - WIGHT_FRONT[3]) * WIGHT_PX
WIGHT_T = 0.42
WIGHT_ZC = WIGHT_ZB + WIGHT_H / 2

# Skjelettet: navnene og klippene som de andre fiendene (mob_motion.py), men
# hoftene inne i plata over de korte beina, skuldrene på sidene av plata og
# hodet der linsa sitter øverst foran (linsa dreier seg mot deg). Halen har
# ingen deler (den er der for klippene).
WIGHT_BONES = [
    ("root", (0, 0, 0), (0, 0, 0.2), None),
    ("hips", (0.0, 0, 0.5), (0.0, 0, 0.62), "root"),
    ("chest", (0.0, 0, 0.62), (0.0, 0, 1.1), "hips"),
    ("neck", (0.05, 0, 1.12), (0.16, 0, 1.18), "chest"),
    ("head", (WIGHT_T / 2 + 0.02, 0, 1.23), (WIGHT_T / 2 + 0.22, 0, 1.23), "neck"),
    ("jaw", (WIGHT_T / 2, 0, 1.12), (WIGHT_T / 2 + 0.12, 0, 1.1), "head"),
    ("upper_arm.L", (0.0, 0.6, 0.88), (0.02, 0.69, 0.64), "chest"),
    ("forearm.L", (0.02, 0.69, 0.64), (0.07, 0.72, 0.42), "upper_arm.L"),
    ("hand.L", (0.07, 0.72, 0.42), (0.1, 0.73, 0.28), "forearm.L"),
    ("upper_arm.R", (0.0, -0.6, 0.88), (0.02, -0.69, 0.64), "chest"),
    ("forearm.R", (0.02, -0.69, 0.64), (0.07, -0.72, 0.42), "upper_arm.R"),
    ("hand.R", (0.07, -0.72, 0.42), (0.1, -0.73, 0.28), "forearm.R"),
    ("thigh.L", (0.0, 0.24, 0.5), (0.03, 0.26, 0.27), "hips"),
    ("shin.L", (0.03, 0.26, 0.27), (-0.02, 0.26, 0.07), "thigh.L"),
    ("foot.L", (-0.02, 0.26, 0.07), (0.12, 0.26, 0.0), "shin.L"),
    ("thigh.R", (0.0, -0.24, 0.5), (0.03, -0.26, 0.27), "hips"),
    ("shin.R", (0.03, -0.26, 0.27), (-0.02, -0.26, 0.07), "thigh.R"),
    ("foot.R", (-0.02, -0.26, 0.07), (0.12, -0.26, 0.0), "shin.R"),
    ("tail1", (-0.12, 0, 0.9), (-0.16, 0, 1.0), "hips"),
    ("tail2", (-0.16, 0, 1.0), (-0.17, 0, 1.1), "tail1"),
    ("tail3", (-0.17, 0, 1.1), (-0.17, 0, 1.2), "tail2"),
]
WIGHT_RIGID = {"slab": "chest", "candle": "chest", "wax": "chest", "rubble": "chest", "eye": "head"}
WIGHT_RADII = {"root": -1.0, "hips": 0.0, "chest": -0.3, "neck": -1.0, "head": -1.0, "jaw": -1.0, "tail1": -1.0, "tail2": -1.0, "tail3": -1.0}


def bone_limb(name, pts, radii, mat):
    """Et lem av bein: skaft mellom leddene og knoker i leddene."""
    for i in range(len(pts) - 1):
        tube(f"{name}_{i}", pts[i], pts[i + 1], radii[i], mat, 10)
    for i in range(1, len(pts) - 1):
        r = radii[i] * 1.7
        ellipsoid(f"{name}_k{i}", pts[i], (r, r, r * 1.1), mat, 12, 8)


def bone_hand(name, wrist, down, out, mat, k=1.0):
    """Hånd av bein: et håndbein og tre krumme fingre og en tommel (`k`
    skalerer)."""
    wrist, down, out = Vector(wrist), Vector(down).normalized(), Vector(out).normalized()
    palm = wrist + down * 0.07 * k
    ellipsoid(f"{name}_palm", wrist + down * 0.035 * k, (0.035 * k, 0.03 * k, 0.045 * k), mat, 10, 6)
    fwd = Vector((1, 0, 0))
    for i, s in enumerate((-1.0, 0.0, 1.0)):
        a = palm + fwd * (s * 0.025 * k)
        b = a + (down * 0.06 + out * 0.01) * k
        c = b + (down * 0.045 - out * 0.015) * k
        tube(f"{name}_f{i}a", a, b, 0.011 * k, mat, 6)
        tube(f"{name}_f{i}b", b, c, 0.009 * k, mat, 6)
        ellipsoid(f"{name}_f{i}k", b, (0.013 * k,) * 3, mat, 6, 4)
    t0 = wrist + (down * 0.04 + fwd * 0.03) * k
    tube(f"{name}_thumb", t0, t0 + (down * 0.04 + fwd * 0.03 - out * 0.02) * k, 0.01 * k, mat, 6)


def button_wight():
    sheet = sheet_material("wight_sheet", WIGHT_SHEET)
    bone = material("bone", (0.86, 0.77, 0.58), 0.65)
    weather(bone, 0.6, 1.5, 0.25, 12.0)
    wax = material("wax", (0.86, 0.8, 0.66), 0.55)
    rubble = material("rubble", (0.16, 0.16, 0.17), 0.85)
    weather(rubble, 0.4, 1.4, 0.3, 8.0)
    steel = material("steel", (0.62, 0.64, 0.68), 0.35, 1.0)
    glass = material("glass_eye", (0.75, 0.82, 0.9), 0.05)

    slab_from_sheet("slab", sheet, rounded_rect(WIGHT_W, WIGHT_H, 0.14), WIGHT_T, WIGHT_ZC, WIGHT_FRONT, WIGHT_BACK, WIGHT_SIDE, (2912, 1440))
    # Linsa øverst foran, i en stålring (den dreier seg mot deg).
    lx, lz = WIGHT_T / 2 + 0.02, 1.23
    bpy.ops.mesh.primitive_torus_add(major_radius=0.085, minor_radius=0.022, major_segments=20, minor_segments=8, location=(lx, 0, lz), rotation=(0, math.pi / 2, 0))
    ring = bpy.context.active_object
    ring.name = "eye_ring"
    ring.data.materials.append(steel)
    ellipsoid("eye_lens", (lx + 0.01, 0, lz), (0.05, 0.075, 0.075), glass, 16, 10)
    # Lysene med rennende voks i hjørnene oppe, og grus og stein bak.
    top = WIGHT_ZC + WIGHT_H / 2
    # Lysene er brent ned og har rent over kanten (voksen foran er malt i
    # arket, så her er bare stumpene og pølene oppå).
    for x, y, hgt, lean in ((0.08, -WIGHT_W / 2 + 0.14, 0.09, 0.08), (-0.08, WIGHT_W / 2 - 0.14, 0.11, -0.06)):
        c = cylinder("candle", (x, y, top + hgt / 2 - 0.02), 0.06, hgt, wax, rot=(lean, 0, 0), vertices=12)
        tube("candle_wick", (x, y - lean * hgt, top + hgt - 0.03), (x + 0.005, y - lean * hgt, top + hgt + 0.01), 0.006, rubble, 5)
        ellipsoid("wax_pool", (x, y, top), (0.16, 0.18, 0.06), wax, 12, 6)
        ellipsoid("wax_lump", (x + 0.03, y + 0.03, top + 0.03), (0.1, 0.09, 0.08), wax, 10, 6)
        # Voksen renner over kanten foran og bak.
        for k, (dx, dz, ln) in enumerate(((WIGHT_T / 2, -0.06, 0.16), (-WIGHT_T / 2, -0.05, 0.12))):
            sx = 1.0 if dx > 0 else -1.0
            ellipsoid(f"wax_drip{k}", (dx + sx * 0.008, y + 0.01, top - ln / 2), (0.03, 0.06, ln), wax, 10, 6)
            ellipsoid(f"wax_drop{k}", (dx + sx * 0.012, y + 0.01, top - ln + 0.01), (0.035, 0.05, 0.05), wax, 8, 5)
    # Grus og steinbiter oppå bak, uregelmessige.
    import random
    rng = random.Random(3)
    for k in range(9):
        x, y, sz = rng.uniform(-0.17, 0.0), rng.uniform(-0.38, 0.38), rng.uniform(0.04, 0.085)
        ob = ellipsoid(f"rubble{k}", (x, y, top + sz * 0.45), (sz * rng.uniform(1.0, 1.6), sz * rng.uniform(1.0, 1.5), sz * rng.uniform(0.7, 1.0)), rubble, 7, 5)
        ob.rotation_euler = (rng.uniform(-0.4, 0.4), rng.uniform(-0.4, 0.4), rng.uniform(0, 3))
    # Armene ut fra sidene av plata, hengende, med hender av bein.
    B = {b[0]: (Vector(b[1]), Vector(b[2])) for b in WIGHT_BONES}
    for side in ("L", "R"):
        sy = 1.0 if side == "L" else -1.0
        sh, el, wr, tip = B[f"upper_arm.{side}"][0], B[f"forearm.{side}"][0], B[f"hand.{side}"][0], B[f"hand.{side}"][1]
        ellipsoid(f"shoulder_{side}", sh + Vector((0, -sy * 0.01, 0)), (0.075, 0.06, 0.075), bone, 12, 8)
        # Kraftige, knudrete armbein som på arket.
        bone_limb(f"arm_{side}", [sh, el, wr], (0.058, 0.046), bone)
        tube(f"arm_{side}_ulna", el + Vector((0.03, 0, 0)), wr + Vector((0.03, 0, 0)), 0.03, bone, 8)
        bone_hand(f"hand_{side}", wr, tip - wr, (0, sy, 0), bone, 1.5)
        hip, knee, ankle, toe = B[f"thigh.{side}"][0], B[f"shin.{side}"][0], B[f"foot.{side}"][0], B[f"foot.{side}"][1]
        # Klumpete beinbein som på arket: tjukke skaft, store knoker.
        bone_limb(f"leg_{side}", [hip, knee, ankle], (0.078, 0.068), bone)
        ellipsoid(f"knee_cap_{side}", knee + Vector((0.04, 0, 0.0)), (0.05, 0.07, 0.06), bone, 12, 8)
        # Foten: en kraftig hæl og tre tær fram.
        ellipsoid(f"heel_{side}", ankle + Vector((-0.01, 0, -0.03)), (0.075, 0.07, 0.05), bone, 12, 8)
        for k, dy in enumerate((-0.045, 0.0, 0.045)):
            a = ankle + Vector((0.03, dy, -0.045))
            tube(f"toe_{side}{k}", a, Vector((toe.x + 0.06, ankle.y + dy * 1.3, 0.02)), 0.022, bone, 8)


def build_wight(size):
    button_wight()
    arm = armature("mutant_button_rig", WIGHT_BONES)
    mob_motion.make_clips(Clip, arm, WIGHT_BONES, "wight", "bite")
    bpy.context.scene.frame_end = 64
    bake_textured(f"{OUT}/mutant_button.glb", size, keep=("glass",), armature=arm, rigid=WIGHT_RIGID, skin={"blend": 0.04, "radii": WIGHT_RADII})


# --- Tooltip Sprite ---------------------------------------------------------
#
# Arket (`tooltip_sprite_body.jpeg`, 2752 × 1536): en snakkeboble med lysende
# skjerm og pikseløyne, tykk grå ramme, lys og stein oppå og bak, og fire
# lange beinbein. Figuren er 1028 px høy (268–1296) og 1,4 m i modellen
# (spillet skalerer slaget til 0,55, `Kind::Tooltip`).

SPRITE_SHEET = os.path.join(REF, "tooltip_sprite_body.jpeg")
SPRITE_PX = 1.4 / (1296 - 268)
SPRITE_FRONT = (205, 395, 875, 1091)
SPRITE_BACK = (1885, 395, 2575, 1091)
SPRITE_SIDE = (1229, 395, 1514, 1091)
SPRITE_W = (875 - 205) * SPRITE_PX
SPRITE_H = (915 - 395) * SPRITE_PX
SPRITE_PD = (1091 - 915) * SPRITE_PX
SPRITE_T = (1514 - 1229) * SPRITE_PX
SPRITE_ZC = (1296 - (395 + 915) / 2) * SPRITE_PX

# Beina: de fremre er «armene» (gangarten på alle fire setter hendene i
# bakken), de bakre beina; knærne står opp over boblens underkant.
SPRITE_BONES = [
    ("root", (0, 0, 0), (0, 0, 0.2), None),
    ("hips", (0.0, 0, 0.56), (0.0, 0, 0.7), "root"),
    ("chest", (0.0, 0, 0.7), (0.0, 0, 1.1), "hips"),
    ("neck", (0.05, 0, 1.0), (0.12, 0, 1.04), "chest"),
    ("head", (SPRITE_T / 2, 0, 0.95), (SPRITE_T / 2 + 0.2, 0, 0.95), "neck"),
    ("jaw", (SPRITE_T / 2, 0, 0.85), (SPRITE_T / 2 + 0.1, 0, 0.83), "head"),
    ("upper_arm.L", (0.08, 0.16, 0.56), (0.24, 0.4, 0.66), "chest"),
    ("forearm.L", (0.24, 0.4, 0.66), (0.34, 0.46, 0.04), "upper_arm.L"),
    ("hand.L", (0.34, 0.46, 0.04), (0.4, 0.47, 0.0), "forearm.L"),
    ("upper_arm.R", (0.08, -0.16, 0.56), (0.24, -0.4, 0.66), "chest"),
    ("forearm.R", (0.24, -0.4, 0.66), (0.34, -0.46, 0.04), "upper_arm.R"),
    ("hand.R", (0.34, -0.46, 0.04), (0.4, -0.47, 0.0), "forearm.R"),
    ("thigh.L", (-0.08, 0.16, 0.56), (-0.24, 0.38, 0.66), "hips"),
    ("shin.L", (-0.24, 0.38, 0.66), (-0.32, 0.44, 0.04), "thigh.L"),
    ("foot.L", (-0.32, 0.44, 0.04), (-0.38, 0.45, 0.0), "shin.L"),
    ("thigh.R", (-0.08, -0.16, 0.56), (-0.24, -0.38, 0.66), "hips"),
    ("shin.R", (-0.24, -0.38, 0.66), (-0.32, -0.44, 0.04), "thigh.R"),
    ("foot.R", (-0.32, -0.44, 0.04), (-0.38, -0.45, 0.0), "shin.R"),
    ("tail1", (-0.1, 0, 0.8), (-0.12, 0, 0.9), "hips"),
    ("tail2", (-0.12, 0, 0.9), (-0.13, 0, 1.0), "tail1"),
    ("tail3", (-0.13, 0, 1.0), (-0.13, 0, 1.1), "tail2"),
]
SPRITE_RIGID = {"slab": "chest", "candle": "chest", "wax": "chest", "rubble": "chest", "under": "chest"}


def tooltip_sprite():
    sheet = sheet_material("sprite_sheet", SPRITE_SHEET)
    bone = material("bone", (0.78, 0.72, 0.6), 0.65)
    weather(bone, 0.0, 0.8, 0.3, 12.0)
    wax = material("wax", (0.86, 0.8, 0.66), 0.55)
    rubble = material("rubble", (0.17, 0.18, 0.17), 0.85)
    weather(rubble, 0.6, 1.3, 0.3, 8.0)
    slab_from_sheet("slab", sheet, bubble(SPRITE_W, SPRITE_H, 0.12, 0.26, SPRITE_PD), SPRITE_T, SPRITE_ZC, SPRITE_FRONT, SPRITE_BACK, SPRITE_SIDE, (2752, 1536), bevel=0.02)
    top = SPRITE_ZC + SPRITE_H / 2
    # Lysene oppå (venstre forfra, høyre bakfra), brent ned.
    for x, y, hgt in ((0.06, -SPRITE_W / 2 + 0.13, 0.12), (-0.07, SPRITE_W / 2 - 0.12, 0.14)):
        cylinder("candle", (x, y, top + hgt / 2 - 0.02), 0.045, hgt, wax, vertices=12)
        tube("candle_wick", (x, y, top + hgt - 0.03), (x, y, top + hgt + 0.01), 0.005, rubble, 5)
        ellipsoid("wax_pool", (x, y, top), (0.08, 0.09, 0.03), wax, 12, 6)
    # Steinbiter som har grodd fast oppå og bak.
    import random
    rng = random.Random(5)
    for k in range(10):
        x = rng.uniform(-SPRITE_T / 2 - 0.05, -0.02)
        y, z = rng.uniform(-0.3, 0.38), rng.uniform(top - 0.25, top + 0.02)
        sz = rng.uniform(0.05, 0.09)
        ob = ellipsoid(f"rubble{k}", (x, y, z), (sz * rng.uniform(1.0, 1.5), sz * rng.uniform(1.0, 1.6), sz * rng.uniform(0.7, 1.1)), rubble, 7, 5)
        ob.rotation_euler = (rng.uniform(-0.4, 0.4), rng.uniform(-0.4, 0.4), rng.uniform(0, 3))
    # Undersida: et beinfeste der beina går inn i boblen.
    ellipsoid("under", (0.0, 0, SPRITE_ZC - SPRITE_H / 2 + 0.02), (0.18, 0.26, 0.07), bone, 12, 6)
    B = {b[0]: (Vector(b[1]), Vector(b[2])) for b in SPRITE_BONES}
    for side in ("L", "R"):
        for upper, lower, end in ((f"upper_arm.{side}", f"forearm.{side}", f"hand.{side}"), (f"thigh.{side}", f"shin.{side}", f"foot.{side}")):
            a, k, e, tip = B[upper][0], B[lower][0], B[end][0], B[end][1]
            bone_limb(f"leg_{upper}", [a, k, e], (0.03, 0.024), bone)
            ellipsoid(f"leg_{upper}_knee", k, (0.04, 0.04, 0.045), bone, 10, 6)
            tube(f"leg_{upper}_claw", e, tip + Vector((0, 0, -0.005)), 0.016, bone, 6)


def build_sprite(size):
    tooltip_sprite()
    arm = armature("mutant_tooltip_rig", SPRITE_BONES)
    mob_motion.make_clips(Clip, arm, SPRITE_BONES, "sprite", "bite")
    bpy.context.scene.frame_end = 64
    bake_textured(f"{OUT}/mutant_tooltip.glb", size, keep=("glass",), armature=arm, rigid=SPRITE_RIGID, skin={"blend": 0.03, "radii": WIGHT_RADII})


# --- Datepicker Priest -----------------------------------------------------
#
# Arket (`datepicker_priest_body.jpeg`, 2752 × 1536): eliten i gravkjelleren,
# en mager prest i kappe vevd av kalenderruter, med vindushode og pilene som
# øyne, kalenderen på brystet, en lysende nøkkel, krumstav med urskive og
# gravstein og kiste på ryggen. Figuren er 1299 px høy (180–1479) og 2,1 m.
# Formen modelleres grovt, og `project_sheet` legger arket på hver flate fra
# den visningen flaten vender mot (forfra, bakfra eller fra siden).

PRIEST_SHEET = os.path.join(REF, "datepicker_priest_body.jpeg")
PRIEST_PX = 2.1 / (1479 - 180)
# Midtlinja i hver visning (px) og bakken (rad); sidebildet viser −Y-sida
# med fronten mot høyre.
PRIEST_VIEWS = {"front": 463, "back": 2317, "side": 1360, "ground": 1479}


def filled_sheet(path):
    """Arket med bakgrunnen fylt med fargen til nærmeste piksel av figuren,
    så en flate som stikker litt utenfor omrisset ikke blir grå."""
    import numpy as np
    from PIL import Image
    from scipy import ndimage
    out = os.path.join(ROOT, "target", "monsters", os.path.basename(path).rsplit(".", 1)[0] + "_fill.png")
    if os.path.exists(out) and os.path.getmtime(out) > os.path.getmtime(path):
        return out
    im = np.asarray(Image.open(path).convert("RGB")).astype(np.int32)
    bg = im[4, 4]
    fg = np.abs(im - bg).sum(2) > 36
    fg = ndimage.binary_opening(fg, iterations=2)
    # Grå partier inni figuren ligner bakgrunnen; de er figur så lenge de
    # er omsluttet av den.
    fg = ndimage.binary_fill_holes(ndimage.binary_closing(fg, iterations=4))
    # Fyllet tar fargen fra innsida av figuren, ikke fra den mørke streken
    # rundt (da blir kanten svarte stråler).
    inner = ndimage.binary_erosion(fg, iterations=4) & (im.sum(2) > 3 * 70)
    _, (iy, ix) = ndimage.distance_transform_edt(~inner, return_indices=True)
    keep = ndimage.binary_erosion(fg, iterations=6)
    iy = np.where(keep, np.arange(im.shape[0])[:, None], iy)
    ix = np.where(keep, np.arange(im.shape[1])[None, :], ix)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    Image.fromarray(im[iy, ix].astype(np.uint8)).save(out)
    return out


def project_sheet(ob, img_size, views, px, only=None):
    """Uv-laget «src» for hver flate i `ob`: forfra (+X), bakfra (−X) eller
    fra siden (±Y), etter hvilken vei flaten vender i verden. `only` låser
    alle flatene til én visning eller et par (f.eks. ("front", "back"))."""
    W, H = img_size
    me = ob.data
    if not me.uv_layers:
        me.uv_layers.new(name="UVMap")
    src = me.uv_layers.get("src") or me.uv_layers.new(name="src")
    mw = ob.matrix_world
    nm = mw.to_3x3().inverted().transposed()
    for poly in me.polygons:
        n = (nm @ poly.normal).normalized()
        ax, ay = abs(n.x), abs(n.y)
        if only and len(only) == 1:
            view = only[0]
        elif (only and "side" not in only) or ax >= ay * 0.5:
            view = "front" if (n.x > 0 if ax > 0.05 else (mw @ poly.center).x > 0) else "back"
        else:
            view = "side"
        for li in poly.loop_indices:
            p = mw @ me.vertices[me.loops[li].vertex_index].co
            if view == "front":
                X = views["front"] + p.y / px
            elif view == "back":
                X = views["back"] - p.y / px
            else:
                X = views["side"] + p.x / px
            Y = views["ground"] - p.z / px
            src.data[li].uv = (X / W, 1.0 - Y / H)


def loft(name, mat, rings, n=24, jag=None, seed=1):
    """En kappe: ringer (z, foran, bak, halv bredde) loftet nedover, åpen
    nederst. Den nederste ringen får en flosset kant (`jag`: hvor mye opp)."""
    import random
    rng = random.Random(seed)
    bm = bmesh.new()
    rows = []
    for k, (z, xf, xb, hy) in enumerate(rings):
        row = []
        for i in range(n):
            a = 2 * math.pi * i / n
            c, s = math.cos(a), math.sin(a)
            x = xf * c if c > 0 else xb * c
            y = hy * s
            zz = z
            if jag and k == len(rings) - 1:
                zz = z + rng.uniform(0, jag) * (1.0 if i % 2 else 0.4)
            row.append(bm.verts.new((x, y, zz)))
        rows.append(row)
    for a, b in zip(rows, rows[1:]):
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((a[i], a[j], b[j], b[i]))
    bm.faces.new(rows[0][::-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    me.materials.append(mat)
    bpy.context.scene.collection.objects.link(ob)
    me.uv_layers.new(name="UVMap")
    return ob


def prism(name, mat, outline, depth, x, rot=(0, 0, 0)):
    """Et flatt stykke (omrisset i (y, z)) med tykkelse `depth` langs x."""
    bm = bmesh.new()
    fr = [bm.verts.new((x + depth / 2, y, z)) for y, z in outline]
    bk = [bm.verts.new((x - depth / 2, y, z)) for y, z in outline]
    bm.faces.new(fr)
    bm.faces.new(bk[::-1])
    for i in range(len(outline)):
        j = (i + 1) % len(outline)
        bm.faces.new((fr[i], bk[i], bk[j], fr[j]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    me.materials.append(mat)
    bpy.context.scene.collection.objects.link(ob)
    me.uv_layers.new(name="UVMap")
    ob.rotation_euler = rot
    return ob


PRIEST_BONES = [
    ("root", (0, 0, 0), (0, 0, 0.2), None),
    ("hips", (0.0, 0, 1.0), (0.0, 0, 1.15), "root"),
    ("chest", (0.0, 0, 1.15), (0.0, 0, 1.62), "hips"),
    ("neck", (0.03, 0, 1.64), (0.1, 0, 1.78), "chest"),
    ("head", (0.12, 0, 1.92), (0.32, 0, 1.92), "neck"),
    ("jaw", (0.2, 0, 1.8), (0.28, 0, 1.78), "head"),
    ("upper_arm.L", (0.0, 0.33, 1.58), (0.04, 0.38, 1.33), "chest"),
    ("forearm.L", (0.04, 0.38, 1.33), (0.27, 0.52, 1.24), "upper_arm.L"),
    ("hand.L", (0.27, 0.52, 1.24), (0.34, 0.58, 1.22), "forearm.L"),
    ("upper_arm.R", (0.0, -0.33, 1.58), (-0.07, -0.42, 1.32), "chest"),
    ("forearm.R", (-0.07, -0.42, 1.32), (-0.02, -0.46, 1.05), "upper_arm.R"),
    ("hand.R", (-0.02, -0.46, 1.05), (0.0, -0.47, 0.86), "forearm.R"),
    ("thigh.L", (0.0, 0.13, 1.0), (0.03, 0.2, 0.52), "hips"),
    ("shin.L", (0.03, 0.2, 0.52), (-0.02, 0.22, 0.09), "thigh.L"),
    ("foot.L", (-0.02, 0.22, 0.09), (0.16, 0.22, 0.0), "shin.L"),
    ("thigh.R", (0.0, -0.13, 1.0), (0.03, -0.2, 0.52), "hips"),
    ("shin.R", (0.03, -0.2, 0.52), (-0.02, -0.22, 0.09), "thigh.R"),
    ("foot.R", (-0.02, -0.22, 0.09), (0.16, -0.22, 0.0), "shin.R"),
    ("tail1", (-0.2, 0, 1.3), (-0.25, 0, 1.5), "hips"),
    ("tail2", (-0.25, 0, 1.5), (-0.27, 0, 1.7), "tail1"),
    ("tail3", (-0.27, 0, 1.7), (-0.28, 0, 1.9), "tail2"),
]
PRIEST_RIGID = {
    "robe": "hips", "torso": "chest", "pauldron": "chest", "lame": "chest", "hood": "chest", "key": "chest", "chain": "chest",
    "coffin": "chest", "grave": "chest", "backbone": "chest", "candle": "chest", "wax": "chest", "pixel": "chest",
    "header": "head", "stem": "head", "eye": "head", "staff": "hand.L", "clock": "hand.L", "spike": "hand.L",
}
PRIEST_RADII = {"root": -1.0, "hips": -0.2, "chest": -0.3, "neck": -1.0, "head": -1.0, "jaw": -1.0, "tail1": -1.0, "tail2": -1.0, "tail3": -1.0}


def arrow(name, mat, x, yc, zc, w, h, d, flip):
    """Et pilhode med skaft (øynene), forfra i (y, z)."""
    s = -1.0 if flip else 1.0
    pts = [(-0.5, -0.17), (0.05, -0.17), (0.05, -0.5), (0.5, 0.0), (0.05, 0.5), (0.05, 0.17), (-0.5, 0.17)]
    out = [(yc + s * u * w, zc + v * h) for u, v in pts]
    if flip:
        out = out[::-1]
    return prism(name, mat, out, d, x)


def datepicker_priest():
    sheet = sheet_material("priest_sheet", filled_sheet(PRIEST_SHEET))
    bone = material("bone", (0.8, 0.74, 0.6), 0.65)
    weather(bone, 0.0, 1.2, 0.25, 12.0)
    steel = material("steel", (0.55, 0.58, 0.62), 0.4, 1.0)
    weather(steel, 0.0, 2.0, 0.3, 10.0)
    wood = material("staff_wood", (0.16, 0.11, 0.08), 0.7)
    weather(wood, 0.0, 1.0, 0.3, 16.0)
    bronze = material("clock_bronze", (0.52, 0.42, 0.3), 0.45, 1.0)
    wax = material("wax", (0.86, 0.8, 0.66), 0.55)
    stone = material("grave_stone", (0.24, 0.25, 0.27), 0.85)
    weather(stone, 0.8, 2.1, 0.35, 8.0)
    magenta = material("magic_key", (0.95, 0.35, 1.0), 0.3)
    cyan = material("magic_eye", (0.45, 0.95, 1.0), 0.3)
    px = PRIEST_PX
    sheeted = []

    # Kappa fra beltet og ned, flosset nederst.
    sheeted.append(loft("robe", sheet, [(1.2, 0.2, 0.16, 0.21), (1.1, 0.22, 0.2, 0.23), (0.94, 0.225, 0.21, 0.27), (0.77, 0.226, 0.22, 0.29), (0.61, 0.226, 0.235, 0.33), (0.45, 0.226, 0.25, 0.35), (0.29, 0.23, 0.26, 0.36)], n=28, jag=0.12))
    # Overkroppen med kalenderen foran, og hetta rundt nakken.
    sheeted.append(rounded_box("torso", (0.02, 0, 1.4), (0.38, 0.44, 0.5), sheet, radius=0.06, segments=3))
    sheeted.append(ellipsoid("hood", (0.02, 0, 1.66), (0.3, 0.36, 0.24), sheet, 18, 10))
    # Skulderplatene: en kuppel og to skinner under på hver side.
    for sy in (1.0, -1.0):
        d = ellipsoid(f"pauldron_{sy:+.0f}", (0.0, sy * 0.32, 1.6), (0.32, 0.26, 0.2), sheet, 18, 10)
        sheeted.append(d)
        for k, z in enumerate((1.52, 1.44)):
            sheeted.append(rounded_box(f"lame_{sy:+.0f}{k}", (0.0, sy * (0.37 + 0.02 * k), z), (0.32, 0.12, 0.08), sheet, radius=0.02, rot=(sy * math.radians(22), 0, 0)))
    # Vindushodet: tittellinja og ruta med pilene, og stammen under.
    sheeted.append(rounded_box("header", (0.18, 0, 2.0), (0.17, 0.41, 0.19), sheet, radius=0.025, segments=3))
    sheeted.append(rounded_box("stem", (0.2, 0, 1.83), (0.11, 0.17, 0.16), sheet, radius=0.02))
    for k, sy in enumerate((-1.0, 1.0)):
        arrow(f"eye_{k}", cyan, 0.27, sy * 0.1, 1.975, 0.11, 0.075, 0.012, sy < 0)
    # Nakken.
    tube("hood_neck", (0.02, 0, 1.62), (0.14, 0, 1.82), 0.055, steel, 10)
    # Nøkkelen i kjedet foran kalenderen.
    kx = 0.235
    bpy.ops.mesh.primitive_torus_add(major_radius=0.045, minor_radius=0.012, major_segments=18, minor_segments=6, location=(kx, 0, 1.53), rotation=(0, math.pi / 2, 0))
    ring = bpy.context.active_object
    ring.name = "key_ring"
    ring.data.materials.append(magenta)
    tube("key_shaft", (kx, 0, 1.49), (kx, 0, 1.12), 0.013, magenta, 8)
    prism("key_bit", magenta, [(0.0, 1.12), (0.07, 1.12), (0.07, 1.15), (0.035, 1.15), (0.035, 1.17), (0.07, 1.17), (0.07, 1.2), (0.0, 1.2)], 0.018, kx)
    for sy in (1.0, -1.0):
        tube(f"chain_{sy:+.0f}", (kx - 0.01, 0, 1.57), (0.12, sy * 0.11, 1.68), 0.006, steel, 5)
    # Armene: beinarmer i ermer, med armskinner.
    B = {b[0]: (Vector(b[1]), Vector(b[2])) for b in PRIEST_BONES}
    for side in ("L", "R"):
        sy = 1.0 if side == "L" else -1.0
        sh, el, wr, tip = B[f"upper_arm.{side}"][0], B[f"forearm.{side}"][0], B[f"hand.{side}"][0], B[f"hand.{side}"][1]
        sleeve = tube(f"sleeve_{side}", sh, el, 0.07, sheet, 12)
        sheeted.append(sleeve)
        bone_limb(f"arm_{side}", [el, wr], (0.03,), bone)
        bracer = tube(f"bracer_{side}", el + (wr - el) * 0.12, el + (wr - el) * 0.85, 0.06, steel, 12)
        bone_hand(f"hand_{side}", wr, tip - wr, (0, sy, 0), bone, 1.6)
        # Beina under kappa: knokete leggbein og sandaler.
        hip, knee, ankle, toe = B[f"thigh.{side}"][0], B[f"shin.{side}"][0], B[f"foot.{side}"][0], B[f"foot.{side}"][1]
        bone_limb(f"leg_{side}", [hip, knee, ankle], (0.05, 0.035), bone)
        ellipsoid(f"heel_{side}", ankle + Vector((-0.01, 0, -0.04)), (0.12, 0.1, 0.08), bone, 10, 6)
        for k, dy in enumerate((-0.035, 0.0, 0.035)):
            tube(f"toe_{side}{k}", ankle + Vector((0.03, dy, -0.05)), Vector((toe.x - 0.02, ankle.y + dy * 1.2, 0.03)), 0.02, bone, 6)
            ellipsoid(f"toe_{side}{k}_tip", Vector((toe.x - 0.02, ankle.y + dy * 1.2, 0.03)), (0.04, 0.035, 0.03), bone, 8, 5)
        rounded_box(f"sandal_{side}", (ankle.x + 0.07, ankle.y, 0.012), (0.3, 0.12, 0.024), steel, radius=0.008)
        rounded_box(f"sandal_strap_{side}", (ankle.x + 0.05, ankle.y, 0.05), (0.06, 0.13, 0.03), steel, radius=0.008)
    # Krumstaven i venstre hånd med urskiva øverst.
    hx, hy = 0.36, 0.6
    tube("staff_shaft", (hx, hy, 0.03), (hx, hy, 1.8), 0.022, wood, 10)
    for k, z in enumerate((0.03, 1.36, 1.64, 1.78)):
        ellipsoid(f"staff_knob{k}", (hx, hy, z), (0.06, 0.06, 0.05), bronze, 10, 6)
    cz, cr = 1.97, 0.19
    bpy.ops.mesh.primitive_torus_add(major_radius=cr, minor_radius=0.022, major_segments=32, minor_segments=8, location=(hx, hy, cz), rotation=(0, math.pi / 2, 0))
    rim = bpy.context.active_object
    rim.name = "clock_rim"
    rim.data.materials.append(bronze)
    face = cylinder("clock_face", (hx, hy, cz), cr - 0.01, 0.03, sheet, rot=(0, math.pi / 2, 0), vertices=32)
    project_sheet(face, (2752, 1536), PRIEST_VIEWS, px, only=("front", "back"))
    for k in range(8):
        a = 2 * math.pi * k / 8
        d = Vector((0, math.sin(a), math.cos(a)))
        c = Vector((hx, hy, cz)) + d * (cr + 0.06)
        bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.025, radius2=0.0, depth=0.1, location=c)
        sp = bpy.context.active_object
        sp.name = f"spike{k}"
        sp.rotation_mode = "QUATERNION"
        sp.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(d)
        sp.data.materials.append(bronze)
    # Ryggen: ei kiste og en gravstein som har grodd fast, med bein og voks.
    coffin = [(-0.15, -0.33), (0.15, -0.33), (0.23, 0.12), (0.12, 0.33), (-0.12, 0.33), (-0.23, 0.12)]
    c = prism("coffin", sheet, [(0.08 + y, 1.48 + z) for y, z in coffin], 0.09, -0.225)
    c.rotation_euler = (0, math.radians(-8), 0)
    sheeted.append(c)
    g = prism("grave_slab", sheet, [(-0.11, -0.39), (0.11, -0.39), (0.11, 0.29), (0.0, 0.39), (-0.11, 0.29)], 0.08, 0.0)
    g.location = (-0.24, -0.2, 1.62)
    g.rotation_euler = (math.radians(14), 0, 0)
    sheeted.append(g)
    tube("backbone_0", (-0.29, -0.2, 1.25), (-0.29, 0.3, 1.72), 0.022, bone, 8)
    tube("backbone_1", (-0.29, 0.25, 1.22), (-0.28, -0.15, 1.7), 0.02, bone, 8)
    for k, (yy, zz) in enumerate(((0.06, 1.62), (-0.12, 1.42))):
        ellipsoid(f"wax_back{k}", (-0.28, yy, zz), (0.08, 0.16, 0.12), wax, 10, 6)
    # Lyset på venstre skulder (høyre i arket forfra).
    cylinder("candle", (0.0, 0.36, 1.78), 0.035, 0.14, wax, vertices=10)
    ellipsoid("wax_pool", (0.0, 0.36, 1.71), (0.14, 0.12, 0.05), wax, 10, 6)
    tube("candle_wick", (0.0, 0.36, 1.84), (0.0, 0.36, 1.87), 0.004, stone, 4)
    # Pikselbiter som brekker av ved armen.
    import random
    rng = random.Random(7)
    for k in range(7):
        y = -0.5 - rng.uniform(0.02, 0.1)
        box(f"pixel{k}", (rng.uniform(-0.05, 0.08), y, rng.uniform(1.0, 1.35)), (0.02, 0.02, 0.02), cyan if k % 2 else magenta)
    bpy.context.view_layer.update()  # matrix_world etter location/rotation
    for ob in sheeted:
        project_sheet(ob, (2752, 1536), PRIEST_VIEWS, px)


def build_priest(size):
    datepicker_priest()
    arm = armature("datepicker_priest_rig", PRIEST_BONES)
    mob_motion.make_clips(Clip, arm, PRIEST_BONES, "priest", "slam")
    bpy.context.scene.frame_end = 64
    bake_textured(f"{OUT}/datepicker_priest.glb", size, keep=("glass", "magic"), armature=arm, rigid=PRIEST_RIGID, skin={"blend": 0.04, "radii": PRIEST_RADII})


# --- Modalen ----------------------------------------------------------------
#
# Arket (`modal_body.jpeg`, 2752 × 1536): bossen i gravkjelleren, en
# dialogboks på armerte bein med lukkeknappen (den glødende X-en) i panna,
# et glødende rom av piksler og gravstein i buken, skulderplater med
# gravstein og voks, en fillete kappe, beinarmer og store hansker med
# Confirm og Cancel, og kabler bak. Figuren er 1197 px høy (220–1417) og
# 1,6 m i modellen (spillet skalerer slaget til 1,5, `Kind::Modal`).

MODAL_SHEET = os.path.join(REF, "modal_body.jpeg")
MODAL_PX = 1.6 / (1417 - 220)
MODAL_VIEWS = {"front": 594, "back": 2215, "side": 1465, "ground": 1417}
MODAL_BONES = [
    ("root", (0, 0, 0), (0, 0, 0.2), None),
    ("hips", (0.0, 0, 0.55), (0.0, 0, 0.7), "root"),
    ("chest", (0.0, 0, 0.7), (0.0, 0, 1.35), "hips"),
    ("neck", (0.0, 0, 1.35), (0.03, 0, 1.4), "chest"),
    ("head", (0.03, 0, 1.47), (0.23, 0, 1.47), "neck"),
    ("jaw", (0.15, 0, 1.41), (0.21, 0, 1.39), "head"),
    ("upper_arm.L", (0.0, 0.44, 1.12), (0.03, 0.5, 0.84), "chest"),
    ("forearm.L", (0.03, 0.5, 0.84), (0.07, 0.5, 0.62), "upper_arm.L"),
    ("hand.L", (0.07, 0.5, 0.62), (0.09, 0.5, 0.3), "forearm.L"),
    ("upper_arm.R", (0.0, -0.44, 1.12), (0.03, -0.5, 0.84), "chest"),
    ("forearm.R", (0.03, -0.5, 0.84), (0.07, -0.5, 0.62), "upper_arm.R"),
    ("hand.R", (0.07, -0.5, 0.62), (0.09, -0.5, 0.3), "forearm.R"),
    ("thigh.L", (0.0, 0.22, 0.55), (0.02, 0.29, 0.33), "hips"),
    ("shin.L", (0.02, 0.29, 0.33), (-0.01, 0.29, 0.1), "thigh.L"),
    ("foot.L", (-0.01, 0.29, 0.1), (0.15, 0.29, 0.0), "shin.L"),
    ("thigh.R", (0.0, -0.22, 0.55), (0.02, -0.29, 0.33), "hips"),
    ("shin.R", (0.02, -0.29, 0.33), (-0.01, -0.29, 0.1), "thigh.R"),
    ("foot.R", (-0.01, -0.29, 0.1), (0.15, -0.29, 0.0), "shin.R"),
    ("tail1", (-0.24, 0, 1.0), (-0.26, 0, 0.8), "chest"),
    ("tail2", (-0.26, 0, 0.8), (-0.27, 0, 0.6), "tail1"),
    ("tail3", (-0.27, 0, 0.6), (-0.28, 0, 0.4), "tail2"),
]
MODAL_RIGID = {
    "box": "chest", "screen": "chest", "pauldron": "chest", "grave": "chest", "candle": "chest", "wax": "chest", "cloak": "chest", "pixel": "chest",
    "lid": "head", "eye": "head", "skirt": "hips", "cable": "chest",
    "gloveL": "hand.L", "gloveR": "hand.R", "bootL": "foot.L", "bootR": "foot.R",
}
MODAL_RADII = {"root": -1.0, "hips": -0.1, "chest": -0.25, "neck": -1.0, "head": -1.0, "jaw": -1.0, "tail1": -0.4, "tail2": -0.4, "tail3": -0.4}


def modal():
    sheet = sheet_material("modal_sheet", filled_sheet(MODAL_SHEET))
    bone = material("bone", (0.8, 0.74, 0.6), 0.65)
    weather(bone, 0.0, 1.0, 0.25, 12.0)
    wax = material("wax", (0.86, 0.8, 0.66), 0.55)
    cable = material("cable", (0.12, 0.13, 0.15), 0.5)
    orange = material("magic_close", (1.0, 0.45, 0.12), 0.3)
    cyan = material("magic_pixel", (0.4, 0.95, 1.0), 0.3)
    magenta = material("magic_pixel_m", (0.95, 0.3, 1.0), 0.3)
    px = MODAL_PX
    sheeted = []
    # Dialogboksen: kroppen, og tittellinja med lukkeknappen som hode.
    sheeted.append(rounded_box("box", (-0.015, 0, 0.96), (0.42, 0.6, 0.82), sheet, radius=0.04, segments=3))
    sheeted.append(rounded_box("lid", (-0.015, 0, 1.49), (0.44, 0.62, 0.24), sheet, radius=0.035, segments=3))
    cylinder("eye_ring", (0.21, 0.0, 1.43), 0.07, 0.03, cable, rot=(0, math.pi / 2, 0), vertices=20)
    cylinder("eye_disc", (0.225, 0.0, 1.43), 0.055, 0.012, orange, rot=(0, math.pi / 2, 0), vertices=20)
    for k, a in enumerate((45, -45)):
        box(f"eye_x{k}", (0.235, 0.0, 1.43), (0.012, 0.012, 0.075), orange, rot=(math.radians(a), 0, 0))
    # Skulderplatene, med gravstein og voks oppå.
    for sy in (1.0, -1.0):
        sheeted.append(rounded_box(f"pauldron_{sy:+.0f}", (0.0, sy * 0.43, 1.24), (0.38, 0.26, 0.3), sheet, radius=0.04, rot=(-sy * math.radians(14), 0, 0)))
        g = prism(f"grave_{sy:+.0f}", sheet, [(-0.06, -0.09), (0.06, -0.09), (0.06, 0.05), (0.0, 0.1), (-0.06, 0.05)], 0.05, 0.0)
        g.location = (-0.04, sy * 0.47, 1.47)
        g.rotation_euler = (-sy * 0.25, 0, 0)
        sheeted.append(g)
        for k, (dx, dy, h) in enumerate(((0.08, 0.02, 0.09), (0.02, -0.06, 0.06))):
            cylinder(f"candle_{sy:+.0f}{k}", (dx, sy * (0.4 + dy), 1.41 + h / 2), 0.022, h, wax, vertices=8)
        ellipsoid(f"wax_{sy:+.0f}", (0.05, sy * 0.4, 1.4), (0.16, 0.14, 0.05), wax, 10, 6)
        # Kappa henger bak armene, fillete nederst.
        cl = prism(f"cloak_{sy:+.0f}", sheet, [(-0.13, 0.32), (0.13, 0.32), (0.15, -0.1), (0.1, -0.06), (0.05, -0.16), (0.0, -0.08), (-0.06, -0.14), (-0.11, -0.04), (-0.15, -0.1)], 0.03, 0.0)
        cl.location = (-0.1, sy * 0.45, 0.82)
        sheeted.append(cl)
    # Armene: beinarmer og store hansker med knappene.
    B = {b[0]: (Vector(b[1]), Vector(b[2])) for b in MODAL_BONES}
    for side in ("L", "R"):
        sy = 1.0 if side == "L" else -1.0
        sh, el, wr, tip = B[f"upper_arm.{side}"][0], B[f"forearm.{side}"][0], B[f"hand.{side}"][0], B[f"hand.{side}"][1]
        bone_limb(f"arm_{side}", [sh, el, wr], (0.045, 0.038), bone)
        tube(f"arm_{side}_ulna", el + Vector((-0.03, 0, 0)), wr + Vector((-0.03, 0, 0)), 0.025, bone, 8)
        sheeted.append(rounded_box(f"glove{side}_cuff", (wr.x + 0.02, wr.y, 0.56), (0.24, 0.26, 0.15), sheet, radius=0.03))
        sheeted.append(rounded_box(f"glove{side}_hand", (wr.x + 0.02, wr.y, 0.38), (0.2, 0.22, 0.2), sheet, radius=0.03))
        for k, dx in enumerate((-0.06, 0.0, 0.06)):
            sheeted.append(rounded_box(f"glove{side}_finger{k}", (wr.x + 0.02 + dx, wr.y, 0.25), (0.05, 0.2, 0.09), sheet, radius=0.015))
        sheeted.append(rounded_box(f"glove{side}_thumb", (wr.x + 0.12, wr.y - sy * 0.08, 0.4), (0.05, 0.06, 0.13), sheet, radius=0.015))
        # Beina: armerte lår og legger, kneskåler og støvler.
        hip, knee, ankle, toe = B[f"thigh.{side}"][0], B[f"shin.{side}"][0], B[f"foot.{side}"][0], B[f"foot.{side}"][1]
        sheeted.append(tube(f"leg_thigh_{side}", hip, knee, 0.1, sheet, 14))
        sheeted.append(tube(f"leg_shin_{side}", knee, ankle + Vector((0, 0, 0.03)), 0.09, sheet, 14))
        sheeted.append(rounded_box(f"leg_knee_{side}", knee + Vector((0.07, 0, 0)), (0.1, 0.19, 0.14), sheet, radius=0.03))
        sheeted.append(rounded_box(f"boot{side}", (ankle.x + 0.04, ankle.y, 0.06), (0.32, 0.22, 0.12), sheet, radius=0.03))
    # Skjørtet av fliller under boksen, foran beina.
    sheeted.append(loft("skirt", sheet, [(0.6, 0.2, 0.2, 0.26), (0.42, 0.2, 0.2, 0.27)], n=20, jag=0.08, seed=3))
    # Kablene bak, hengende fra boksen.
    import random
    rng = random.Random(11)
    for k in range(6):
        y0 = rng.uniform(-0.18, 0.18)
        top = Vector((-0.22, y0, rng.uniform(1.2, 1.45)))
        mid = Vector((-0.25 - rng.uniform(0, 0.03), y0 + rng.uniform(-0.05, 0.05), rng.uniform(0.75, 0.95)))
        end = Vector((-0.25, y0 + rng.uniform(-0.05, 0.05), rng.uniform(0.25, 0.5)))
        tube(f"cable_{k}a", top, mid, 0.014, cable, 6)
        tube(f"cable_{k}b", mid, end, 0.014, cable, 6)
    for k in range(8):
        y = rng.choice((-1, 1)) * rng.uniform(0.31, 0.36)
        box(f"pixel{k}", (rng.uniform(-0.1, 0.2), y, rng.uniform(0.6, 1.6)), (0.025, 0.025, 0.025), cyan if k % 2 else magenta)
    bpy.context.view_layer.update()
    for ob in sheeted:
        project_sheet(ob, (2752, 1536), MODAL_VIEWS, px)


def build_modal(size):
    modal()
    arm = armature("mutant_modal_rig", MODAL_BONES)
    mob_motion.make_clips(Clip, arm, MODAL_BONES, "dialog", "slam")
    bpy.context.scene.frame_end = 64
    bake_textured(f"{OUT}/mutant_modal.glb", size, keep=("glass", "magic"), armature=arm, rigid=MODAL_RIGID, skin={"blend": 0.04, "radii": MODAL_RADII})


# --- Input Leech --------------------------------------------------------------
#
# Arket (`input_leech_body.jpeg`, 2048 × 2048): en lav snegl i kloakken, et
# avrundet tekstfelt på magen med markøren som ett loddrett øye foran,
# mose og murstein grodd fast, sugekopper under. 400 px høy (828–1228) og
# 0,5 m i modellen (spillet skalerer slaget til 1,15, `Kind::Input`). Den
# har ingen bein, så klippene er bølging langs ryggen (`leech_clips`).

LEECH_SHEET = os.path.join(REF, "input_leech_body.jpeg")
LEECH_PX = 0.5 / (1228 - 828)
LEECH_VIEWS = {"front": 335, "back": 1704, "side": 1035, "ground": 1228}
# Snitt langs kroppen: (x, halv bredde, bunn, topp), fra halen til hodet.
LEECH_SECTIONS = [(-0.43, 0.12, 0.05, 0.1), (-0.36, 0.22, 0.04, 0.2), (-0.24, 0.27, 0.035, 0.255), (-0.04, 0.28, 0.035, 0.27),
                  (0.1, 0.29, 0.035, 0.33), (0.2, 0.295, 0.035, 0.44), (0.3, 0.295, 0.035, 0.49), (0.38, 0.29, 0.04, 0.49), (0.42, 0.26, 0.06, 0.46)]
LEECH_BONES = [
    ("root", (0, 0, 0), (0, 0, 0.1), None),
    ("hips", (0.0, 0, 0.13), (0.14, 0, 0.15), "root"),
    ("chest", (0.14, 0, 0.15), (0.26, 0, 0.2), "hips"),
    ("neck", (0.26, 0, 0.2), (0.3, 0, 0.27), "chest"),
    ("head", (0.3, 0, 0.3), (0.44, 0, 0.3), "neck"),
    ("jaw", (0.36, 0, 0.12), (0.44, 0, 0.1), "head"),
    ("tail1", (0.0, 0, 0.13), (-0.14, 0, 0.13), "hips"),
    ("tail2", (-0.14, 0, 0.13), (-0.28, 0, 0.12), "tail1"),
    ("tail3", (-0.28, 0, 0.12), (-0.43, 0, 0.08), "tail2"),
]
LEECH_RIGID = {}  # øyet følger flata det sitter i (stivt på hodet løsnet det i utfallet)
LEECH_RADII = {"root": -1.0, "jaw": -1.0}


def loft_x(name, mat, sections, n=28):
    """En kropp loftet langs x: snittene (x, halv bredde, bunn, topp) er
    avrundede rektangler i (y, z), med lokk i begge ender."""
    bm = bmesh.new()
    rings = []
    for x, hw, zb, zt in sections:
        h = zt - zb
        r = min(0.45 * h, 0.9 * hw, 0.14)
        pts = rounded_rect(2 * hw, h, r, n=max(2, n // 8))
        rings.append([bm.verts.new((x, y, (zb + zt) / 2 + z)) for y, z in pts])
    for a, b in zip(rings, rings[1:]):
        m = len(a)
        for i in range(m):
            j = (i + 1) % m
            bm.faces.new((a[i], a[j], b[j], b[i]))
    bm.faces.new(rings[0][::-1])
    bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    me.materials.append(mat)
    bpy.context.scene.collection.objects.link(ob)
    me.uv_layers.new(name="UVMap")
    mod = ob.modifiers.new("sub", "SUBSURF")
    mod.levels = 1
    mod.render_levels = 1
    return ob


def input_leech():
    sheet = sheet_material("leech_sheet", filled_sheet(LEECH_SHEET))
    cup = material("leech_cup", (0.42, 0.47, 0.5), 0.5)
    cyan = material("magic_caret", (0.45, 0.95, 1.0), 0.3)
    body = loft_x("body", sheet, LEECH_SECTIONS)
    # Markøren: ett loddrett, lysende øye i skjermen foran.
    box("eye_caret", (0.433, 0.0, 0.28), (0.02, 0.035, 0.15), cyan)
    # Sugekoppene under, seks på hver side.
    for k in range(6):
        x = -0.32 + 0.13 * k
        for sy in (1.0, -1.0):
            cylinder(f"cup_{k}{sy:+.0f}", (x, sy * 0.19, 0.02), 0.035, 0.04, cup, vertices=10)
    bpy.context.view_layer.update()
    project_sheet(body, (2048, 2048), LEECH_VIEWS, LEECH_PX)


def leech_clips(arm):
    """Klippene til en snegl uten bein: bølging langs ryggen i gangen,
    utfall med hodet, rykk, svimmelhet og død."""
    chain = ["tail3", "tail2", "tail1", "hips", "chest", "neck", "head"]

    def pose(c, f, wave=0.0, amp=0.0, pitch=None, root=(0, 0, 0), roll=0.0, jaw=0.0):
        for i, b in enumerate(chain):
            z = amp * math.sin(wave - 0.9 * i) * (1.0 if b != "head" else -0.4)
            y = (pitch or {}).get(b, 0.0)
            c.rot(b, f, x=roll if b == "hips" else 0.0, y=y, z=z)
        c.rot("jaw", f, y=-jaw)
        c.loc("root", f, root)
        c.rot("root", f)

    def clip(name, length, fn, step=1):
        c = Clip(arm, name, length, mob_motion.FPS)
        for f in range(0, length + 1, step):
            fn(c, f, f / length)
        c.finish()

    # Gangen: en bølge bakover langs kroppen, og et lite dytt fram.
    clip("walk", 16, lambda c, f, t: pose(c, f, 2 * math.pi * t, 9.0, root=(0.01 * math.sin(4 * math.pi * t), 0, 0.006 * abs(math.sin(2 * math.pi * t)))))
    clip("idle", 48, lambda c, f, t: pose(c, f, 2 * math.pi * t, 3.0, {"neck": -4 * math.sin(2 * math.pi * t), "head": 3 * math.sin(4 * math.pi * t)}), 2)

    def attack(c, f, t):
        # Trekker seg sammen og reiser hodet, så utfall fram (treff i bilde 15), og tilbake.
        k = f
        if k <= 10:
            u = k / 10
            pose(c, f, 0, 0, {"chest": -10 * u, "neck": -22 * u, "head": -8 * u}, root=(-0.06 * u, 0, 0), jaw=4 * u)
        elif k <= 15:
            u = (k - 10) / 5
            pose(c, f, 0, 0, {"chest": -10 + 22 * u, "neck": -22 + 40 * u, "head": -8 + 18 * u}, root=(-0.06 + 0.26 * u, 0, 0), jaw=4 + 20 * u)
        else:
            u = (k - 15) / 15
            e = 1 - (1 - u) ** 2
            pose(c, f, 0, 0, {"chest": 12 * (1 - e), "neck": 18 * (1 - e), "head": 10 * (1 - e)}, root=(0.2 * (1 - e), 0, 0), jaw=24 * (1 - e))
    clip("attack", 30, attack)
    clip("alert", 22, lambda c, f, t: pose(c, f, 0, 0, {"neck": -28 * math.sin(math.pi * min(1, t * 1.6)), "head": 10 * math.sin(math.pi * min(1, t * 1.6))}, jaw=8 * math.sin(math.pi * t)))
    for name, side in (("hit", 1.0), ("hit2", -1.0)):
        clip(name, 14, lambda c, f, t, side=side: pose(c, f, 1.0, side * 16 * math.exp(-4 * t) * math.cos(10 * t), {"neck": -10 * math.exp(-5 * t)}, root=(-0.06 * math.exp(-4 * t), 0, 0)))
    clip("stagger", 30, lambda c, f, t: pose(c, f, 2 * math.pi * 2 * t, 12.0 * math.sin(math.pi * t), {"neck": 14 * math.sin(math.pi * t), "head": 10}, roll=8 * math.sin(4 * math.pi * t)))
    # Døden: velter over på sida, eller synker sammen og flater ut.
    clip("die", 36, lambda c, f, t: pose(c, f, 0, 0, {"neck": 20 * min(1, t * 2)}, root=(0, 0, -0.02 * min(1, t * 2)), roll=85 * min(1, t * 1.8) ** 2, jaw=18 * min(1, t * 2)))
    clip("die2", 36, lambda c, f, t: pose(c, f, 0, 6 * (1 - t), {"chest": 8 * min(1, t * 2), "neck": 35 * min(1, t * 2), "head": 20 * min(1, t * 2), "tail2": -10 * min(1, t * 2)}, root=(0.05 * min(1, t * 2), 0, -0.03 * min(1, t * 2)), jaw=10))


def build_leech(size):
    input_leech()
    arm = armature("mutant_input_rig", LEECH_BONES)
    leech_clips(arm)
    bpy.context.scene.frame_end = 48
    bake_textured(f"{OUT}/mutant_input.glb", size, keep=("glass", "magic"), armature=arm, rigid=LEECH_RIGID, skin={"blend": 0.08, "radii": LEECH_RADII})


# --- Tabellen ----------------------------------------------------------------
#
# Arket (`table_body.jpeg`, 2048 × 2048): bossen i kloakken, en tusenbein av
# tabellrader. Arket viser den stående: til venstre buken med radene og
# ansiktet, i midten fra siden (krummet som en S), til høyre ryggen med
# murstein og slim. I spillet kryper den langs bakken, så radene (venstre
# stripe) ligger på ryggen der du ser dem, og hodet er reist med ansiktet
# fram. Stripa er 260 px bred (0,45 m) og 1700 px lang (2,9 m); spillet
# skalerer slaget til 1,4 (`Kind::Table`).

TABLE_SHEET = os.path.join(REF, "table_body.jpeg")
TABLE_PX = 0.45 / 260
TABLE_TOP_X, TABLE_BELLY_X, TABLE_MID_Y = 360, 1712, 1000
TABLE_Z = 0.36  # midt i kroppen over bakken
TABLE_ROWS = [400 + 97.2 * k for k in range(15)]  # radene i stripa (px), fra hodet bakover
TABLE_SPINE = [("hips", -0.1, 0.4), ("chest", 0.4, 0.9), ("neck", 0.9, 1.15), ("head", 1.15, 1.55)]
TABLE_TAIL = [("tail1", -0.1, -0.5), ("tail2", -0.5, -0.9), ("tail3", -0.9, -1.35)]


def strip_x(Y):
    """Lengden i stripa (rad i px) til x i modellen (hodet mot +x)."""
    return (TABLE_MID_Y - Y) * TABLE_PX


def table_uv(ob, img_size):
    """Uv-laget «src» i stripas rom: oppover (+z) radene, nedover ryggen fra
    høyre stripe, sidene og endene fra kanten av radene."""
    W, H = img_size
    me = ob.data
    if not me.uv_layers:
        me.uv_layers.new(name="UVMap")
    src = me.uv_layers.get("src") or me.uv_layers.new(name="src")
    px = TABLE_PX
    for poly in me.polygons:
        n = poly.normal
        for li in poly.loop_indices:
            p = me.vertices[me.loops[li].vertex_index].co
            Y = TABLE_MID_Y - p.x / px
            if n.z >= max(abs(n.x), abs(n.y)) * 0.8:
                X = TABLE_TOP_X + p.y / px
            elif -n.z >= max(abs(n.x), abs(n.y)) * 0.8:
                X = TABLE_BELLY_X - p.y / px
            elif abs(n.y) >= abs(n.x):
                # Sida: kanten av rada, med høyden som en smal stripe innover.
                X = TABLE_TOP_X + math.copysign(118 - (p.z - TABLE_Z) / px * 0.4, n.y)
            else:
                X = TABLE_TOP_X + p.y / px
                Y = Y + math.copysign(40, -n.x) + (p.z - TABLE_Z) / px * 0.3
            src.data[li].uv = (X / W, 1.0 - Y / H)


def table_bones():
    bones = [("root", (0, 0, 0), (0, 0, 0.2), None)]
    for name, a, b in TABLE_SPINE:
        parent = {"hips": "root", "chest": "hips", "neck": "chest", "head": "neck"}[name]
        bones.append((name, (a, 0, TABLE_Z), (b, 0, TABLE_Z + (0.15 if name == "head" else 0.0)), parent))
    bones.append(("jaw", (1.3, 0, TABLE_Z + 0.05), (1.45, 0, TABLE_Z), "head"))
    for i, (name, a, b) in enumerate(TABLE_TAIL):
        bones.append((name, (a, 0, TABLE_Z), (b, 0, TABLE_Z - 0.02 * i), "hips" if i == 0 else TABLE_TAIL[i - 1][0]))
    for k, x in enumerate(table_leg_xs()):
        for sy, side in ((1.0, "L"), (-1.0, "R")):
            bones.append((f"leg{k:02d}.{side}", (x, sy * 0.22, TABLE_Z), (x, sy * 0.5, TABLE_Z + 0.04), spine_at(x)))
    return bones


def table_leg_xs():
    return [strip_x((TABLE_ROWS[k] + TABLE_ROWS[k + 1]) / 2) for k in range(len(TABLE_ROWS) - 1)]


def spine_at(x):
    for name, a, b in TABLE_SPINE + TABLE_TAIL:
        if min(a, b) <= x <= max(a, b):
            return name
    return "tail3" if x < 0 else "chest"


def table_monster():
    sheet = sheet_material("table_sheet", filled_sheet(TABLE_SHEET))
    leg = material("leg_plastic", (0.5, 0.56, 0.62), 0.45)
    weather(leg, 0.0, 0.5, 0.2, 10.0)
    claw = material("leg_tip", (0.2, 0.22, 0.26), 0.5)
    cable = material("cable", (0.1, 0.11, 0.13), 0.5)
    cyan = material("magic_core", (0.45, 0.95, 1.0), 0.3)
    horn = material("horn", (0.15, 0.16, 0.19), 0.5)
    parts = []
    # Radene: én kloss per rad, med en smal fuge imellom.
    for k in range(len(TABLE_ROWS) - 1):
        x0, x1 = strip_x(TABLE_ROWS[k + 1]) + 0.006, strip_x(TABLE_ROWS[k]) - 0.006
        b = rounded_box(f"row{k:02d}", ((x0 + x1) / 2, 0, TABLE_Z), (x1 - x0, 0.45, 0.17), sheet, radius=0.02)
        parts.append((b, spine_at((x0 + x1) / 2)))
    # Brystet med kjernen som lyser, og hodet (tegnet flatt, så reist).
    xc0, xc1 = strip_x(400), strip_x(300)
    chest = rounded_box("row_chest", ((xc0 + xc1) / 2, 0, TABLE_Z), (xc1 - xc0, 0.3, 0.15), sheet, radius=0.03)
    parts.append((chest, "neck"))
    xh0, xh1 = strip_x(300), strip_x(140)
    head = rounded_box("head_block", ((xh0 + xh1) / 2, 0, TABLE_Z), (xh1 - xh0, 0.52, 0.2), sheet, radius=0.03)
    parts.append((head, "head"))
    for ob, _ in parts:
        apply_mods(ob)
        table_uv(ob, (2048, 2048))
    # Hodet vippes 60° fram om bakkanten, så ansiktet (oversida i arket)
    # ser fram og litt opp.
    pivot = Vector((xh0, 0, TABLE_Z + 0.1))
    from mathutils import Matrix
    R = Matrix.Translation(pivot) @ Matrix.Rotation(math.radians(60), 4, "Y") @ Matrix.Translation(-pivot)
    head.data.transform(R)
    for sy in (1.0, -1.0):
        hb = (R @ Vector((xh1 - 0.02, sy * 0.2, TABLE_Z + 0.1))).to_3d()
        bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.035, radius2=0.0, depth=0.16, location=hb)
        h = bpy.context.active_object
        h.name = f"head_horn{sy:+.0f}"
        h.rotation_euler = (sy * -0.35, math.radians(60), 0)
        h.data.materials.append(horn)
    core = ellipsoid("core_glow", ((xc0 + xc1) / 2 + 0.02, 0, TABLE_Z + 0.08), (0.09, 0.09, 0.05), cyan, 10, 6)
    # Beina: ett par per rad, ut og opp til kneet, ned til en spiss klo.
    for k, x in enumerate(table_leg_xs()):
        for sy, side in ((1.0, "L"), (-1.0, "R")):
            a = Vector((x, sy * 0.2, TABLE_Z))
            kn = Vector((x + 0.02, sy * 0.5, TABLE_Z + 0.04))
            ft = Vector((x + 0.06, sy * 0.74, 0.02))
            tube(f"leg{k:02d}.{side}_a", a, kn, 0.03, leg, 8)
            ellipsoid(f"leg{k:02d}.{side}_k", kn, (0.07, 0.07, 0.07), leg, 8, 6)
            tube(f"leg{k:02d}.{side}_b", kn, kn + (ft - kn) * 0.75, 0.026, leg, 8)
            tube(f"leg{k:02d}.{side}_c", kn + (ft - kn) * 0.75, ft, 0.018, claw, 6)
    # Kablene som henger etter halen.
    for k, dy in enumerate((-0.1, -0.03, 0.05, 0.12)):
        x = strip_x(TABLE_ROWS[-1])
        tube(f"cable_{k}a", (x + 0.05, dy, TABLE_Z - 0.04), (x - 0.25, dy * 1.3, 0.12), 0.012, cable, 6)
        tube(f"cable_{k}b", (x - 0.25, dy * 1.3, 0.12), (x - 0.45, dy * 1.6, 0.02), 0.012, cable, 6)
    return [(ob.name, bone) for ob, bone in parts]


def apply_mods(ob):
    bpy.context.view_layer.objects.active = ob
    for m in list(ob.modifiers):
        bpy.ops.object.modifier_apply(modifier=m.name)
    # Lokasjonen inn i hjørnene, så uv-en regnes i modellens rom.
    ob.data.transform(ob.matrix_world)
    ob.matrix_world = ob.matrix_world.__class__.Identity(4)


def table_clips(arm, legs):
    """Gangen: en bølge sideveis langs ryggen og beina i takt bakover,
    to og to i motfase; utfall med hodet ned, rykk, død på ryggen."""
    spine = ["head", "neck", "chest", "hips", "tail1", "tail2", "tail3"]

    def pose(c, f, wave=0.0, amp=0.0, legamp=0.0, pitch=None, root=(0, 0, 0), roll=0.0, curl=0.0, jaw=0.0):
        for i, b in enumerate(spine):
            c.rot(b, f, y=(pitch or {}).get(b, 0.0), z=amp * math.sin(wave + 0.8 * i))
        for k, side in legs:
            sy = 1.0 if side == "L" else -1.0
            ph = wave + 0.7 * k + (0.0 if side == "L" else math.pi)
            swing = legamp * math.sin(ph)
            lift = max(0.0, legamp * 0.8 * math.cos(ph))
            c.rot(f"leg{k:02d}.{side}", f, x=sy * (lift + curl), z=swing)
        c.rot("jaw", f, y=-jaw)
        c.rot("root", f, x=roll)
        c.loc("root", f, root)

    def clip(name, length, fn, step=1):
        c = Clip(arm, name, length, mob_motion.FPS)
        for f in range(0, length + 1, step):
            fn(c, f, f / length)
        c.finish()

    clip("walk", 16, lambda c, f, t: pose(c, f, 2 * math.pi * t, 5.0, 22.0, root=(0, 0, 0.01 * abs(math.sin(4 * math.pi * t)))))

    def idle(c, f, t):
        jerk = 6.0 if 20 <= f <= 23 else 0.0  # rykker digitalt
        pose(c, f, 2 * math.pi * t, 1.5, 4.0, {"head": -4 * math.sin(2 * math.pi * t) - jerk, "neck": jerk})
    clip("idle", 40, idle)

    def attack(c, f, t):
        if f <= 10:
            u = f / 10
            pose(c, f, 0, 0, 0, {"chest": -14 * u, "neck": -18 * u, "head": -10 * u}, root=(-0.08 * u, 0, 0), jaw=6 * u)
        elif f <= 15:
            u = (f - 10) / 5
            pose(c, f, 0, 0, 0, {"chest": -14 + 26 * u, "neck": -18 + 40 * u, "head": -10 + 30 * u}, root=(-0.08 + 0.3 * u, 0, 0), jaw=6 + 24 * u)
        else:
            e = 1 - (1 - (f - 15) / 15) ** 2
            pose(c, f, 0, 0, 0, {"chest": 12 * (1 - e), "neck": 22 * (1 - e), "head": 20 * (1 - e)}, root=(0.22 * (1 - e), 0, 0), jaw=30 * (1 - e))
    clip("attack", 30, attack)
    clip("alert", 22, lambda c, f, t: pose(c, f, 0, 0, 6.0 * math.sin(math.pi * t), {"chest": -16 * math.sin(math.pi * min(1, 1.6 * t)), "neck": -14 * math.sin(math.pi * min(1, 1.6 * t))}, jaw=10 * math.sin(math.pi * t)))
    for name, side in (("hit", 1.0), ("hit2", -1.0)):
        clip(name, 14, lambda c, f, t, side=side: pose(c, f, 1.0, side * 10 * math.exp(-4 * t) * math.cos(10 * t), 8 * math.exp(-4 * t), {"neck": -8 * math.exp(-5 * t)}, root=(-0.08 * math.exp(-4 * t), 0, 0)))
    clip("stagger", 30, lambda c, f, t: pose(c, f, 4 * math.pi * t, 8 * math.sin(math.pi * t), 10.0, {"head": 12 * math.sin(math.pi * t)}, roll=6 * math.sin(4 * math.pi * t)))
    clip("die", 36, lambda c, f, t: pose(c, f, 0, 0, 0, {"head": 15 * min(1, 2 * t)}, root=(0, 0, 0.25 * math.sin(math.pi * min(1, 1.5 * t)) - 0.0), roll=170 * min(1, 1.5 * t) ** 2, curl=40 * min(1, 2 * t), jaw=20))
    clip("die2", 36, lambda c, f, t: pose(c, f, 0, 4 * (1 - t), 0, {"chest": 6, "neck": 20 * min(1, 2 * t), "head": 25 * min(1, 2 * t)}, root=(0, 0, -0.18 * min(1, 2 * t)), curl=-25 * min(1, 2 * t), jaw=12))


def build_table(size):
    rigid_parts = table_monster()
    bones = table_bones()
    arm = armature("mutant_table_rig", bones)
    legs = [(k, side) for k in range(len(table_leg_xs())) for side in ("L", "R")]
    table_clips(arm, legs)
    bpy.context.scene.frame_end = 40
    rigid = {name: bone for name, bone in rigid_parts}
    rigid.update({"head_horn": "head", "core": "neck", "cable": "tail3"})
    for k, side in legs:
        rigid[f"leg{k:02d}.{side}_"] = f"leg{k:02d}.{side}"
    bake_textured(f"{OUT}/mutant_table.glb", size, keep=("glass", "magic"), armature=arm, rigid=rigid, skin={"blend": 0.04, "radii": {"root": -1.0}})


# --- Toast Frog --------------------------------------------------------------
#
# Arket (`toast_frog_body.jpeg`, 2048 × 2048): en tykk frosk som er et
# toast-varsel, med en grønn «success»-stripe nedover ryggen, en rund
# lukkeknapp som øye på hver side av hodet, lange svømmeføtter og bred munn.
# 455 px høy (792–1247) og 0,55 m.

FROG_SHEET = os.path.join(REF, "toast_frog_body.jpeg")
FROG_PX = 0.55 / (1247 - 792)
FROG_VIEWS = {"front": 372, "back": 1683, "side": 1040, "ground": 1247}
FROG_BONES = [
    ("root", (0, 0, 0), (0, 0, 0.1), None),
    ("hips", (-0.12, 0, 0.22), (0.0, 0, 0.26), "root"),
    ("chest", (0.0, 0, 0.26), (0.15, 0, 0.3), "hips"),
    ("neck", (0.15, 0, 0.3), (0.2, 0, 0.36), "chest"),
    ("head", (0.2, 0, 0.4), (0.38, 0, 0.4), "neck"),
    ("jaw", (0.2, 0, 0.29), (0.37, 0, 0.27), "head"),
    ("upper_arm.L", (0.14, 0.17, 0.24), (0.19, 0.25, 0.12), "chest"),
    ("forearm.L", (0.19, 0.25, 0.12), (0.22, 0.28, 0.03), "upper_arm.L"),
    ("hand.L", (0.22, 0.28, 0.03), (0.3, 0.31, 0.0), "forearm.L"),
    ("upper_arm.R", (0.14, -0.17, 0.24), (0.19, -0.25, 0.12), "chest"),
    ("forearm.R", (0.19, -0.25, 0.12), (0.22, -0.28, 0.03), "upper_arm.R"),
    ("hand.R", (0.22, -0.28, 0.03), (0.3, -0.31, 0.0), "forearm.R"),
    ("thigh.L", (-0.12, 0.19, 0.2), (0.0, 0.33, 0.2), "hips"),
    ("shin.L", (0.0, 0.33, 0.2), (-0.15, 0.36, 0.05), "thigh.L"),
    ("foot.L", (-0.15, 0.36, 0.05), (0.03, 0.42, 0.0), "shin.L"),
    ("thigh.R", (-0.12, -0.19, 0.2), (0.0, -0.33, 0.2), "hips"),
    ("shin.R", (0.0, -0.33, 0.2), (-0.15, -0.36, 0.05), "thigh.R"),
    ("foot.R", (-0.15, -0.36, 0.05), (0.03, -0.42, 0.0), "shin.R"),
    ("tail1", (-0.24, 0, 0.26), (-0.29, 0, 0.24), "hips"),
    ("tail2", (-0.29, 0, 0.24), (-0.32, 0, 0.22), "tail1"),
    ("tail3", (-0.32, 0, 0.22), (-0.34, 0, 0.2), "tail2"),
]
FROG_RIGID = {"eye": "head"}


def toast_frog():
    sheet = sheet_material("frog_sheet", filled_sheet(FROG_SHEET))
    parts = []
    parts.append(ellipsoid("body", (-0.03, 0, 0.27), (0.62, 0.54, 0.42), sheet, 24, 14))
    parts.append(ellipsoid("head_dome", (0.17, 0, 0.36), (0.38, 0.46, 0.27), sheet, 24, 12))
    parts.append(ellipsoid("head_lip", (0.25, 0, 0.3), (0.28, 0.42, 0.1), sheet, 20, 8))
    for sy in (1.0, -1.0):
        parts.append(ellipsoid(f"eye_{sy:+.0f}", (0.17, sy * 0.17, 0.5), (0.15, 0.15, 0.15), sheet, 16, 10))
    B = {b[0]: (Vector(b[1]), Vector(b[2])) for b in FROG_BONES}
    for side in ("L", "R"):
        sy = 1.0 if side == "L" else -1.0
        sh, el, wr, tip = B[f"upper_arm.{side}"][0], B[f"forearm.{side}"][0], B[f"hand.{side}"][0], B[f"hand.{side}"][1]
        parts.append(tube(f"arm_{side}_a", sh, el, 0.045, sheet, 10))
        parts.append(tube(f"arm_{side}_b", el, wr, 0.035, sheet, 10))
        parts.append(ellipsoid(f"hand_{side}", wr + Vector((0.04, sy * 0.01, -0.015)), (0.12, 0.11, 0.025), sheet, 10, 5))
        hip, knee, ankle, toe = B[f"thigh.{side}"][0], B[f"shin.{side}"][0], B[f"foot.{side}"][0], B[f"foot.{side}"][1]
        parts.append(tube(f"leg_{side}_a", hip, knee, 0.07, sheet, 12))
        parts.append(ellipsoid(f"leg_{side}_k", knee, (0.12, 0.12, 0.12), sheet, 10, 6))
        parts.append(tube(f"leg_{side}_b", knee, ankle, 0.045, sheet, 10))
        parts.append(tube(f"leg_{side}_c", ankle, toe + Vector((0, 0, 0.01)), 0.03, sheet, 8))
        parts.append(ellipsoid(f"foot_{side}", toe + Vector((0.03, sy * 0.02, 0.0)), (0.16, 0.13, 0.025), sheet, 10, 5))
    bpy.context.view_layer.update()
    for ob in parts:
        project_sheet(ob, (2048, 2048), FROG_VIEWS, FROG_PX)


def build_frog(size):
    toast_frog()
    arm = armature("mutant_toast_rig", FROG_BONES)
    mob_motion.make_clips(Clip, arm, FROG_BONES, "frog", "spit")
    bpy.context.scene.frame_end = 64
    bake_textured(f"{OUT}/mutant_toast.glb", size, keep=("glass", "magic"), armature=arm, rigid=FROG_RIGID, skin={"blend": 0.04, "radii": {"root": -1.0, "jaw": -1.0, "neck": -1.0}})


# --- Select Broodmother -------------------------------------------------------
#
# Arket (`select_broodmother_body.jpeg`, 2912 × 1440): eliten i kloakken, en
# tønne av en nedtrekksliste med vinkelhodet og linseøyet øverst foran, en
# lysende nøkkel i det valgte feltet, skulderplater med pigger, armer med et
# nøkkelspyd, et trekkspill av valg med små hodeskaller under og åtte
# leddede bein. 1103 px høy (167–1270) og 1,6 m (eliten skaleres 1,3).

BROOD_SHEET = os.path.join(REF, "select_broodmother_body.jpeg")
BROOD_PX = 1.6 / (1270 - 167)
BROOD_VIEWS = {"front": 512, "back": 2423, "side": 1499, "ground": 1270}
BROOD_LEGS = [(-1, 35), (-1, 70), (-1, 110), (-1, 145), (1, 35), (1, 70), (1, 110), (1, 145)]  # (side, vinkel fra fronten)


def brood_leg_names():
    out = []
    for k in range(4):
        out += [(k, "L"), (k, "R")]
    return out


def brood_leg_geom(k, side):
    sy = 1.0 if side == "L" else -1.0
    ang = math.radians([35, 70, 110, 145][k])
    d = Vector((math.cos(ang), sy * math.sin(ang), 0.0))
    hip = Vector((0.0, 0.0, 0.72)) + d * 0.3
    knee = Vector((0.0, 0.0, 1.0)) + d * 0.72
    foot = Vector((0.0, 0.0, 0.0)) + d * 1.0
    return hip, knee, foot


BROOD_BONES = [
    ("root", (0, 0, 0), (0, 0, 0.2), None),
    ("hips", (0.0, 0, 0.72), (0.0, 0, 0.9), "root"),
    ("chest", (0.0, 0, 0.9), (0.0, 0, 1.35), "hips"),
    ("neck", (0.2, 0, 1.25), (0.32, 0, 1.3), "chest"),
    ("head", (0.32, 0, 1.33), (0.5, 0, 1.33), "neck"),
    ("jaw", (0.36, 0, 1.22), (0.48, 0, 1.2), "head"),
    ("upper_arm.L", (0.0, 0.42, 1.25), (0.08, 0.52, 0.98), "chest"),
    ("forearm.L", (0.08, 0.52, 0.98), (0.22, 0.5, 0.82), "upper_arm.L"),
    ("hand.L", (0.22, 0.5, 0.82), (0.28, 0.5, 0.74), "forearm.L"),
    ("upper_arm.R", (0.0, -0.42, 1.25), (0.08, -0.52, 0.98), "chest"),
    ("forearm.R", (0.08, -0.52, 0.98), (0.22, -0.5, 0.82), "upper_arm.R"),
    ("hand.R", (0.22, -0.5, 0.82), (0.28, -0.5, 0.74), "forearm.R"),
    ("tail1", (0.0, 0, 0.72), (0.0, 0, 0.6), "hips"),
    ("tail2", (0.0, 0, 0.6), (0.0, 0, 0.48), "tail1"),
    ("tail3", (0.0, 0, 0.48), (0.0, 0, 0.36), "tail2"),
]
for _k, _s in brood_leg_names():
    _h, _kn, _f = brood_leg_geom(_k, _s)
    BROOD_BONES.append((f"leg{_k:02d}.{_s}", tuple(_h), tuple(_kn), "hips"))


def broodmother():
    sheet = sheet_material("brood_sheet", filled_sheet(BROOD_SHEET))
    iron = material("rust_iron", (0.36, 0.22, 0.14), 0.6, 1.0)
    weather(iron, 0.0, 1.8, 0.35, 12.0)
    cyan = material("magic_lens", (0.45, 0.95, 1.0), 0.3)
    keyglow = material("magic_key", (0.45, 0.95, 1.0), 0.3)
    parts = []
    # Tønna: en sylinder på tvers (aksen langs y), med nedtrekkslista foran.
    body = cylinder("body_drum", (0.0, 0, 1.08), 0.45, 0.66, sheet, rot=(math.pi / 2, 0, 0), vertices=28)
    parts.append(body)
    parts.append(rounded_box("body_panel", (0.36, 0, 1.0), (0.14, 0.6, 0.52), sheet, radius=0.03))
    parts.append(rounded_box("head_chevron", (0.36, 0, 1.36), (0.2, 0.34, 0.22), sheet, radius=0.04))
    ellipsoid("eye_lens", (0.46, 0, 1.36), (0.05, 0.11, 0.08), cyan, 14, 8)
    # Nøkkelen i det valgte feltet.
    bpy.ops.mesh.primitive_torus_add(major_radius=0.05, minor_radius=0.012, major_segments=16, minor_segments=6, location=(0.44, -0.12, 0.98), rotation=(0, math.pi / 2, 0))
    ring = bpy.context.active_object
    ring.name = "key_ring"
    ring.data.materials.append(keyglow)
    tube("key_shaft", (0.44, -0.07, 0.98), (0.44, 0.16, 0.98), 0.012, keyglow, 6)
    for k, dy in enumerate((0.1, 0.15)):
        tube(f"key_bit{k}", (0.44, dy, 0.98), (0.44, dy, 0.93), 0.012, keyglow, 6)
    # Skulderplatene med pigger.
    for sy in (1.0, -1.0):
        parts.append(ellipsoid(f"pauldron_{sy:+.0f}", (0.0, sy * 0.4, 1.3), (0.36, 0.26, 0.24), sheet, 16, 10))
        for k, (dx, dz) in enumerate(((0.08, 0.1), (-0.08, 0.1), (0.0, 0.14))):
            bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.025, radius2=0.0, depth=0.1, location=(dx, sy * 0.48, 1.3 + dz))
            sp = bpy.context.active_object
            sp.name = f"pauldron_spike{sy:+.0f}{k}"
            sp.rotation_euler = (-sy * 0.6, 0, 0)
            sp.data.materials.append(iron)
    # Armene: pansrede rør og en stor hanske, nøkkelspydet i venstre.
    B = {b[0]: (Vector(b[1]), Vector(b[2])) for b in BROOD_BONES}
    for side in ("L", "R"):
        sh, el, wr = B[f"upper_arm.{side}"][0], B[f"forearm.{side}"][0], B[f"hand.{side}"][0]
        parts.append(tube(f"arm_{side}_a", sh, el, 0.07, sheet, 10))
        parts.append(tube(f"arm_{side}_b", el, wr, 0.08, sheet, 10))
        parts.append(ellipsoid(f"glove_{side}", wr + Vector((0.03, 0, -0.03)), (0.14, 0.12, 0.14), sheet, 10, 6))
    tube("spear_shaft", (0.28, 0.5, 0.25), (0.28, 0.5, 1.75), 0.018, iron, 8)
    for k, z in enumerate((1.55, 1.62, 1.69)):
        box(f"spear_tooth{k}", (0.28, 0.54, z), (0.02, 0.06, 0.025), iron)
    bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.035, radius2=0.0, depth=0.14, location=(0.28, 0.5, 1.82))
    tip = bpy.context.active_object
    tip.name = "spear_tip"
    tip.data.materials.append(iron)
    # Trekkspillet av valg under tønna.
    for k in range(7):
        z = 0.7 - k * 0.055
        r = 0.2 if k % 2 == 0 else 0.17
        parts.append(cylinder(f"accordion{k}", (0.0, 0, z), r, 0.05, sheet, vertices=20))
    # Beina: åtte leddede bein ut fra undersida av tønna.
    for k, side in brood_leg_names():
        hip, knee, foot = brood_leg_geom(k, side)
        parts.append(tube(f"leg{k:02d}.{side}_a", hip, knee, 0.05, sheet, 8))
        parts.append(ellipsoid(f"leg{k:02d}.{side}_k", knee, (0.11, 0.11, 0.11), sheet, 8, 6))
        parts.append(tube(f"leg{k:02d}.{side}_b", knee, foot + Vector((0, 0, 0.05)), 0.04, sheet, 8))
        bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.04, radius2=0.0, depth=0.1, location=foot + Vector((0, 0, 0.05)), rotation=(math.pi, 0, 0))
        c = bpy.context.active_object
        c.name = f"leg{k:02d}.{side}_tip"
        c.data.materials.append(iron)
    bpy.context.view_layer.update()
    for ob in parts:
        project_sheet(ob, (2912, 1440), BROOD_VIEWS, BROOD_PX)


def build_brood(size):
    broodmother()
    arm = armature("mutant_select_rig", BROOD_BONES)
    legs = brood_leg_names()
    table_clips(arm, legs)
    bpy.context.scene.frame_end = 40
    rigid = {"body": "chest", "head": "head", "eye": "head", "key": "chest", "pauldron": "chest", "spear": "hand.L", "accordion": "tail2"}
    for k, side in legs:
        rigid[f"leg{k:02d}.{side}_"] = f"leg{k:02d}.{side}"
    bake_textured(f"{OUT}/mutant_select.glb", size, keep=("glass", "magic"), armature=arm, rigid=rigid, skin={"blend": 0.04, "radii": {"root": -1.0, "jaw": -1.0}})


BUILDERS = {"mutant_button": (build_wight, 1024), "mutant_tooltip": (build_sprite, 1024), "datepicker_priest": (build_priest, 2048), "mutant_modal": (build_modal, 2048), "mutant_input": (build_leech, 1024), "mutant_table": (build_table, 2048), "mutant_toast": (build_frog, 1024), "mutant_select": (build_brood, 2048)}

if __name__ == "__main__":
    args = sys.argv[1:]
    kinds = [a for a in args if a in BUILDERS] or list(BUILDERS)
    size = next((int(a) for a in args if a.isdigit()), None)
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        build, default_size = BUILDERS[k]
        build(size or default_size)
