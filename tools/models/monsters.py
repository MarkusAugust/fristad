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
from common import Clip, armature, bake_textured, clear, cylinder, ellipsoid, material, rounded_box, tube, weather  # noqa: E402
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


def slab_from_sheet(name, mat, w, h, t, zc, r, front, back, side, img_size):
    """En plate (bredde langs y, høyde langs z, tykkelse langs x) med
    forsida (+X) og baksida (−X) fra rutene `front` og `back` i arket og
    kantene fra `side` (pikselrektangler x0, y0, x1, y1). Arket forfra har
    +Y til høyre, bakfra til venstre, og fra siden står forsida til
    venstre."""
    W, H = img_size
    bm = bmesh.new()
    base = bm.loops.layers.uv.new("UVMap")
    src = bm.loops.layers.uv.new("src")
    ring = rounded_rect(w, h, r)
    zb = zc - h / 2

    def at(rect, fu, fv):
        x0, y0, x1, y1 = rect
        return ((x0 + fu * (x1 - x0)) / W, 1.0 - (y1 - fv * (y1 - y0)) / H)

    fr = [bm.verts.new((t / 2, y, zc + z)) for y, z in ring]
    bk = [bm.verts.new((-t / 2, y, zc + z)) for y, z in ring]
    f = bm.faces.new(fr)
    for lp in f.loops:
        c = lp.vert.co
        lp[src].uv = at(front, (c.y + w / 2) / w, (c.z - zb) / h)
        lp[base].uv = (c.y, c.z)
    f = bm.faces.new(bk[::-1])
    for lp in f.loops:
        c = lp.vert.co
        lp[src].uv = at(back, (w / 2 - c.y) / w, (c.z - zb) / h)
        lp[base].uv = (c.y, c.z)
    n = len(ring)
    for i in range(n):
        j = (i + 1) % n
        f = bm.faces.new((fr[i], bk[i], bk[j], fr[j]))
        ny = ring[i][0] + ring[j][0]
        nz = ring[i][1] + ring[j][1]
        for lp in f.loops:
            c = lp.vert.co
            fu = (t / 2 - c.x) / t
            if abs(ny) / w >= abs(nz) / h:
                lp[src].uv = at(side, fu, (c.z - zb) / h)
            else:
                # Oppe og nede: et smalt bånd øverst i sideruta.
                lp[src].uv = at(side, fu, 0.97 - 0.05 * (c.y + w / 2) / w)
            lp[base].uv = (c.x, c.y + c.z)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    me.materials.append(mat)
    bpy.context.scene.collection.objects.link(ob)
    mod = ob.modifiers.new("bevel", "BEVEL")
    mod.width = 0.025
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

    slab_from_sheet("slab", sheet, WIGHT_W, WIGHT_H, WIGHT_T, WIGHT_ZC, 0.14, WIGHT_FRONT, WIGHT_BACK, WIGHT_SIDE, (2912, 1440))
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


BUILDERS = {"mutant_button": (build_wight, 1024)}

if __name__ == "__main__":
    args = sys.argv[1:]
    kinds = [a for a in args if a in BUILDERS] or list(BUILDERS)
    size = next((int(a) for a in args if a.isdigit()), None)
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        build, default_size = BUILDERS[k]
        build(size or default_size)
