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
    bone = material("bone", (0.82, 0.76, 0.62), 0.65)
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
    for x, y, hgt, lean in ((0.08, -WIGHT_W / 2 + 0.14, 0.13, 0.12), (-0.08, WIGHT_W / 2 - 0.14, 0.16, -0.1)):
        c = cylinder("candle", (x, y, top + hgt / 2 - 0.02), 0.055, hgt, wax, rot=(lean, 0, 0), vertices=12)
        tube("candle_wick", (x, y - lean * hgt, top + hgt - 0.03), (x + 0.005, y - lean * hgt, top + hgt + 0.01), 0.006, rubble, 5)
        ellipsoid("wax_pool", (x, y, top), (0.1, 0.11, 0.035), wax, 12, 6)
        ellipsoid("wax_lump", (x + 0.03, y + 0.03, top + 0.03), (0.06, 0.05, 0.05), wax, 10, 6)
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
        bone_limb(f"arm_{side}", [sh, el, wr], (0.042, 0.034), bone)
        tube(f"arm_{side}_ulna", el + Vector((0.02, 0, 0)), wr + Vector((0.02, 0, 0)), 0.022, bone, 8)
        bone_hand(f"hand_{side}", wr, tip - wr, (0, sy, 0), bone, 1.5)
        hip, knee, ankle, toe = B[f"thigh.{side}"][0], B[f"shin.{side}"][0], B[f"foot.{side}"][0], B[f"foot.{side}"][1]
        # Klumpete beinbein som på arket: tjukke skaft, store knoker.
        bone_limb(f"leg_{side}", [hip, knee, ankle], (0.06, 0.052), bone)
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
    wood = material("staff_wood", (0.3, 0.24, 0.18), 0.7)
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
            tube(f"toe_{side}{k}", ankle + Vector((0.03, dy, -0.05)), Vector((toe.x + 0.07, ankle.y + dy * 1.2, 0.02)), 0.016, bone, 6)
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


BUILDERS = {"mutant_button": (build_wight, 1024), "mutant_tooltip": (build_sprite, 1024), "datepicker_priest": (build_priest, 2048)}

if __name__ == "__main__":
    args = sys.argv[1:]
    kinds = [a for a in args if a in BUILDERS] or list(BUILDERS)
    size = next((int(a) for a in args if a.isdigit()), None)
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        build, default_size = BUILDERS[k]
        build(size or default_size)
