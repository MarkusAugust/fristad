"""Innbyggerne som ekte figurer, bygd på MakeHumans grunnmodell (CC0):
kroppen med riktig anatomi, hender og øyne, formet med MakeHumans
formmål (kjønn, alder, muskler, vekt), posert med MakeHumans standardrigg
(armene ned, fingrene rundt redskapet) og overført til spillets enkle
skjelett (samme bein og klipp som npc.py). Eierens ansiktsfoto
(assets/textures/faces/) projiseres forfra på det ekte ansiktet, festet i
øynene og munnen; håret og skjegget ellers er teksturer fra eierens ark.
Klærne er skall av kroppens egne flater (skjorte, bukse, kjole, støvler),
og forkleet henger foran. Kjør:

    python3.13 tools/models/people.py smith|innkeeper [--preview ut.png] [størrelse]

MakeHuman-dataene hentes til tools/models/.makehuman/ første gang (fra
github.com/makehumancommunity/mpfb2, CC0). Origo i bakken mellom føttene,
fronten mot +X, venstre mot +Y."""

import gzip
import json
import math
import os
import subprocess
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import armature, bake_textured, clear  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402
from npc import clips  # noqa: E402

OUT = "assets/models/npc"
TEX = os.path.abspath("assets/textures")
FACES = os.path.abspath("assets/textures/faces")
ATLAS = os.path.abspath("assets/models/npc")
MH = os.path.abspath("tools/models/.makehuman")


def fetch():
    """Henter grunnmodellen, formmålene og riggen fra MPFB (CC0)."""
    if os.path.exists(f"{MH}/3dobjs/base.obj"):
        return
    tmp = "/tmp/mpfb2"
    if not os.path.exists(tmp):
        subprocess.run(["git", "clone", "--depth", "1", "--filter=blob:none", "--sparse", "https://github.com/makehumancommunity/mpfb2.git", tmp], check=True)
        subprocess.run(["git", "-C", tmp, "sparse-checkout", "set", "src/mpfb/data/3dobjs", "src/mpfb/data/rigs/standard", "src/mpfb/data/targets/macrodetails"], check=True)
    os.makedirs(MH, exist_ok=True)
    for d in ("3dobjs", "rigs", "targets"):
        subprocess.run(["cp", "-r", f"{tmp}/src/mpfb/data/{d}", MH], check=True)
    subprocess.run(["cp", f"{tmp}/LICENSE.ASSETS.md", MH], check=True)


def lin(c):
    """sRGB (0..1) til lineær."""
    return tuple(((x + 0.055) / 1.055) ** 2.4 if x > 0.04045 else x / 12.92 for x in c)


# --- Materialer -------------------------------------------------------------


def _bsdf(mat):
    return mat.node_tree.nodes["Principled BSDF"]


def _mix(nt, a, b, fac, blend="MIX"):
    """Blander to farger (utganger eller rgb-tupler) med faktoren `fac`."""
    m = nt.nodes.new("ShaderNodeMix")
    m.data_type = "RGBA"
    m.blend_type = blend
    for slot, v in ((6, a), (7, b)):
        if isinstance(v, tuple):
            m.inputs[slot].default_value = (*v, 1.0)
        else:
            nt.links.new(v, m.inputs[slot])
    if isinstance(fac, (int, float)):
        m.inputs["Factor"].default_value = fac
    else:
        nt.links.new(fac, m.inputs["Factor"])
    return m.outputs[2]


def _math(nt, op, a, b=None, clamp=False):
    n = nt.nodes.new("ShaderNodeMath")
    n.operation = op
    n.use_clamp = clamp
    for i, v in enumerate((a, b)):
        if v is None:
            continue
        if isinstance(v, (int, float)):
            n.inputs[i].default_value = v
        else:
            nt.links.new(v, n.inputs[i])
    return n.outputs[0]


def _mix_f(nt, a, b, t):
    """a·(1−t) + b·t for tall."""
    return _math(nt, "ADD", _math(nt, "MULTIPLY", a, _math(nt, "SUBTRACT", 1.0, t)), _math(nt, "MULTIPLY", b, t))


def _ramp(nt, v, lo, hi):
    """0 under `lo`, 1 over `hi`, mykt imellom."""
    n = nt.nodes.new("ShaderNodeMapRange")
    n.interpolation_type = "SMOOTHSTEP"
    n.inputs["From Min"].default_value = lo
    n.inputs["From Max"].default_value = hi
    nt.links.new(v, n.inputs["Value"])
    return n.outputs[0]


def _boxtex(nt, path, scale, mapping_from):
    mp = nt.nodes.new("ShaderNodeMapping")
    mp.inputs["Scale"].default_value = (1 / scale,) * 3
    nt.links.new(mapping_from, mp.inputs["Vector"])
    img = nt.nodes.new("ShaderNodeTexImage")
    img.image = bpy.data.images.load(path, check_existing=True)
    img.projection = "BOX"
    img.projection_blend = 0.35
    nt.links.new(mp.outputs["Vector"], img.inputs["Vector"])
    return img


def cloth(name, layer, tint=(1, 1, 1), scale=0.35, rough=0.85, bump=0.4, dirt=None):
    """Et materiale med teksturlaget fra eieren (albedo og høyde), lagt på med
    boksprojeksjon i objektets rom (`scale` meter per flis), tonet. `dirt`
    (høyde, styrke) gjør det mørkere og skittent nederst."""
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    b = _bsdf(m)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    img = _boxtex(nt, f"{TEX}/{layer}/albedo.jpg", scale, tc.outputs["Object"])
    col = _mix(nt, img.outputs["Color"], tint, 1.0, "MULTIPLY")
    if dirt:
        h, s = dirt
        sep = nt.nodes.new("ShaderNodeSeparateXYZ")
        nt.links.new(tc.outputs["Object"], sep.inputs[0])
        f = _math(nt, "MULTIPLY", _ramp(nt, sep.outputs["Z"], h, 0.0), s)
        col = _mix(nt, col, (0.25, 0.2, 0.16), f, "MULTIPLY")
    nt.links.new(col, b.inputs["Base Color"])
    b.inputs["Roughness"].default_value = rough
    h = _boxtex(nt, f"{TEX}/{layer}/height.jpg", scale, tc.outputs["Object"])
    h.image.colorspace_settings.name = "Non-Color"
    bp = nt.nodes.new("ShaderNodeBump")
    bp.inputs["Strength"].default_value = bump
    bp.inputs["Distance"].default_value = 0.004
    nt.links.new(h.outputs["Color"], bp.inputs["Height"])
    nt.links.new(bp.outputs["Normal"], b.inputs["Normal"])
    return m


def flat(name, rgb, rough=0.6, metal=0.0):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    b = _bsdf(m)
    b.inputs["Base Color"].default_value = (*rgb, 1.0)
    b.inputs["Roughness"].default_value = rough
    b.inputs["Metallic"].default_value = metal
    return m


def crop(path, box, name):
    """Et utsnitt (x0, y0, x1, y1 i 0..1 fra øverst til venstre) av et ark,
    med bakgrunnen fylt med snittfargen, lagret som PNG."""
    import numpy as np
    out = f"/tmp/{name}.png"
    img = bpy.data.images.load(path, check_existing=True)
    w, h = img.size
    px = np.array(img.pixels[:]).reshape(h, w, 4)[::-1]
    x0, y0, x1, y1 = (int(box[0] * w), int(box[1] * h), int(box[2] * w), int(box[3] * h))
    part = px[y0:y1, x0:x1].copy()
    a = part[:, :, 3:4]
    mean = (part[:, :, :3] * a).sum(axis=(0, 1)) / max(1e-6, a.sum())
    part[:, :, :3] = part[:, :, :3] * a + mean * (1 - a)
    part[:, :, 3] = 1.0
    ni = bpy.data.images.new(name, part.shape[1], part.shape[0], alpha=True)
    ni.pixels = part[::-1].ravel().tolist()
    ni.filepath_raw = out
    ni.file_format = "PNG"
    ni.save()
    return out


def photo_skin(photo, at):
    """Hudfargen i fotoet (snitt rundt punktet `at`, 0..1 fra øverst til
    venstre), lineær."""
    import numpy as np
    img = bpy.data.images.load(photo, check_existing=True)
    w, h = img.size
    px = np.array(img.pixels[:]).reshape(h, w, 4)[::-1]
    x, y = int(at[0] * w), int(at[1] * h)
    c = px[y - 12:y + 12, x - 12:x + 12, :3].reshape(-1, 3).mean(axis=0)
    return tuple(float(v) for v in c)


def head_material(name, face, skin_tex, skin_tone, hair=None, beard=None):
    """Hodet: fotoet forfra (uv-laget «face») der flaten vender fram og
    ligger innenfor bildet, og ellers hud (et utsnitt av kinnet i fotoet),
    hår over hårfestet (`hair`: (tekstur, tone, hårfestet i panna, over
    ørene, i nakken)) og skjegg på kjeven og under haka (`beard`:
    (tekstur, tone, under z)). Maskene regnes i objektets rom (meter, z opp,
    x fram)."""
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    b = _bsdf(m)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    pos = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(tc.outputs["Object"], pos.inputs[0])
    geo = nt.nodes.new("ShaderNodeNewGeometry")
    nrm = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(geo.outputs["Normal"], nrm.inputs[0])
    st = _boxtex(nt, skin_tex, 0.06, tc.outputs["Object"])
    col = _mix(nt, st.outputs["Color"], skin_tone, 1.0, "MULTIPLY")
    if beard:
        tex, tone, below = beard
        bt = _boxtex(nt, tex, 0.07, tc.outputs["Object"])
        bc = _mix(nt, bt.outputs["Color"], tone, 1.0, "MULTIPLY")
        mask = _math(nt, "MULTIPLY", _ramp(nt, pos.outputs["Z"], below + 0.015, below - 0.015), _ramp(nt, pos.outputs["X"], -0.06, -0.02))
        col = _mix(nt, col, bc, mask)
    if hair:
        tex, tone, front_z, side_z, nape_z = hair
        ht = _boxtex(nt, tex, 0.09, tc.outputs["Object"])
        hc = _mix(nt, ht.outputs["Color"], tone, 1.0, "MULTIPLY")
        # Hårfestet: høyt i panna, over ørene på sidene, ned i nakken bak.
        f = _ramp(nt, pos.outputs["X"], 0.03, 0.07)
        bk = _ramp(nt, pos.outputs["X"], -0.02, -0.06)
        line = _math(nt, "ADD", _math(nt, "MULTIPLY", f, front_z - side_z), side_z)
        line = _math(nt, "ADD", line, _math(nt, "MULTIPLY", bk, nape_z - side_z))
        mask = _ramp(nt, _math(nt, "SUBTRACT", pos.outputs["Z"], line), -0.01, 0.01)
        col = _mix(nt, col, hc, mask)
    uv = nt.nodes.new("ShaderNodeUVMap")
    uv.uv_map = "face"
    img = nt.nodes.new("ShaderNodeTexImage")
    img.image = bpy.data.images.load(face, check_existing=True)
    img.extension = "EXTEND"
    nt.links.new(uv.outputs["UV"], img.inputs["Vector"])
    # Fotoet bare der flaten vender fram og ligger innenfor bildet.
    sep = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(uv.outputs["UV"], sep.inputs[0])
    inside = _math(nt, "MULTIPLY", _ramp(nt, sep.outputs["Y"], 0.0, 0.04), _ramp(nt, sep.outputs["Y"], 1.0, 0.96))
    inside = _math(nt, "MULTIPLY", inside, _math(nt, "MULTIPLY", _ramp(nt, sep.outputs["X"], 0.1, 0.18), _ramp(nt, sep.outputs["X"], 0.9, 0.82)))
    front = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["X"], 0.15, 0.5), inside)
    col = _mix(nt, col, img.outputs["Color"], front)
    nt.links.new(col, b.inputs["Base Color"])
    b.inputs["Roughness"].default_value = 0.55
    return m


def skin_material(name, skin_tex, tone, scale=0.06):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    tc = nt.nodes.new("ShaderNodeTexCoord")
    st = _boxtex(nt, skin_tex, scale, tc.outputs["Object"])
    nt.links.new(_mix(nt, st.outputs["Color"], tone, 1.0, "MULTIPLY"), _bsdf(m).inputs["Base Color"])
    _bsdf(m).inputs["Roughness"].default_value = 0.55
    return m


# --- MakeHuman ---------------------------------------------------------------


def load_obj():
    """Grunnmodellen: hjørnene (MakeHumans akser, desimeter, y opp, front
    +z) og flatene med gruppenavn."""
    verts, faces, group = [], [], None
    for line in open(f"{MH}/3dobjs/base.obj"):
        if line.startswith("v "):
            verts.append(Vector(tuple(map(float, line.split()[1:4]))))
        elif line.startswith("g "):
            group = line.split()[1]
        elif line.startswith("f "):
            faces.append((group, [int(t.split("/")[0]) - 1 for t in line.split()[1:]]))
    return verts, faces


def apply_targets(verts, targets):
    for name, w in targets:
        if w <= 0:
            continue
        with gzip.open(f"{MH}/targets/macrodetails/{name}.target.gz", "rt") as f:
            for line in f:
                p = line.split()
                if len(p) == 4 and not line.startswith("#"):
                    i = int(p[0])
                    verts[i] += Vector((float(p[1]), float(p[2]), float(p[3]))) * w


def macro(gender, age, muscle, weight):
    """Formmålene og vektene som MakeHuman blander dem med: `gender` 0 kvinne
    til 1 mann, `age` 0 ung (25) til 1 gammel (90), `muscle` og `weight` 0
    gjennomsnitt til 1 mest."""
    out = []
    for g, gw in (("female", 1 - gender), ("male", gender)):
        for a, aw in (("young", 1 - age), ("old", age)):
            out.append((f"caucasian-{g}-{a}", gw * aw))
            for mu, mw in (("averagemuscle", 1 - muscle), ("maxmuscle", muscle)):
                for we, ww in (("averageweight", 1 - weight), ("maxweight", weight)):
                    out.append((f"universal-{g}-{a}-{mu}-{we}", gw * aw * mw * ww))
    return out


# Hvilke av MakeHumans bein som blir hvilke av spillets.
def our_bone(mh):
    side = mh[-2:] if mh[-2:] in (".L", ".R") else ""
    base = mh[: -2] if side else mh
    if base in ("root", "spine05", "spine04", "pelvis"):
        return "hips"
    if base in ("spine03", "spine02"):
        return "spine"
    if base in ("spine01", "breast", "clavicle"):
        return "chest"
    if base.startswith("neck"):
        return "neck"
    if base in ("shoulder01", "upperarm01", "upperarm02"):
        return "upper_arm" + side
    if base.startswith("lowerarm"):
        return "forearm" + side
    if base in ("wrist",) or base.startswith("metacarpal") or base.startswith("finger"):
        return "hand" + side
    if base.startswith("upperleg"):
        return "thigh" + side
    if base.startswith("lowerleg"):
        return "shin" + side
    if base == "foot" or base.startswith("toe"):
        return "foot" + side
    return "head"


def build_human(spec):
    """Kroppen etter `spec`: formmålene, høyden, armene ned og hendene
    (`grip`: høyre hånd rundt et skaft). Gir kroppsobjektet (med
    vektgrupper for spillets bein) og leddene i spillets akser."""
    fetch()
    verts, faces = load_obj()
    apply_targets(verts, macro(*spec["macro"]))
    keep = ("body", "helper-l-eye", "helper-r-eye")
    used = sorted({i for g, f in faces if g in keep for i in f})
    remap = {old: new for new, old in enumerate(used)}

    def mp(v):
        # MakeHuman (x, y, z) i desimeter → MPFB i Blender: x, −z, y i meter.
        return Vector((v.x, -v.z, v.y)) * 0.1

    bm = bmesh.new()
    bv = [bm.verts.new(mp(verts[i])) for i in used]
    eyes = []
    for g, f in faces:
        if g in keep:
            try:
                face = bm.faces.new([bv[remap[i]] for i in f])
                if g != "body":
                    eyes.append(face.index if face.index >= 0 else None)
            except ValueError:
                pass
    me = bpy.data.meshes.new("body")
    bm.to_mesh(me)
    bm.free()
    body = bpy.data.objects.new("body", me)
    bpy.context.scene.collection.objects.link(body)
    groups = {}
    for g, f in faces:
        if g.startswith("joint-"):
            groups.setdefault(g, set()).update(f)

    def joint(name):
        s = groups[name]
        return sum((mp(verts[i]) for i in s), Vector()) / len(s)

    # MakeHumans standardrigg med vektene, til å posere kroppen.
    rig = json.load(open(f"{MH}/rigs/standard/rig.default.json"))
    weights = json.load(open(f"{MH}/rigs/standard/weights.default.json"))["weights"]

    def place(d):
        st = d["strategy"]
        if st == "CUBE":
            return joint(d["cube_name"])
        if st == "VERTEX":
            return mp(verts[d["vertex_index"]])
        return sum((mp(verts[i]) for i in d["vertex_indices"]), Vector()) / len(d["vertex_indices"])

    data = bpy.data.armatures.new("mh")
    arm = bpy.data.objects.new("mh", data)
    bpy.context.scene.collection.objects.link(arm)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="EDIT")
    made = {}
    order = []
    left = list(rig)
    while left:
        for name in list(left):
            par = rig[name].get("parent")
            if not par or par in made:
                d = rig[name]
                eb = data.edit_bones.new(name)
                eb.head = place(d["head"])
                eb.tail = place(d["tail"])
                if (eb.tail - eb.head).length < 1e-4:
                    eb.tail = eb.head + Vector((0, 0, 0.01))
                eb.roll = d.get("roll", 0.0)
                if par:
                    eb.parent = made[par]
                made[name] = eb
                order.append(name)
                left.remove(name)
    bpy.ops.object.mode_set(mode="OBJECT")
    for bone, vw in weights.items():
        vg = body.vertex_groups.new(name=bone)
        for i, w in vw:
            if i in remap:
                vg.add([remap[i]], w, "REPLACE")
    mod = body.modifiers.new("arm", "ARMATURE")
    mod.object = arm
    # Posering: armene ned langs siden med albuen litt bøyd, fingrene.
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="POSE")

    def aim(bone, direction):
        pb = arm.pose.bones[bone]
        bpy.context.view_layer.update()
        cur = (pb.tail - pb.head).normalized()
        q = cur.rotation_difference(Vector(direction).normalized())
        m = pb.matrix.copy()
        head = m.translation.copy()
        rot = q.to_matrix().to_4x4()
        pb.matrix = Matrix.Translation(head) @ rot @ Matrix.Translation(-head) @ m
        bpy.context.view_layer.update()

    for s, sx in (("L", 1), ("R", -1)):
        aim(f"upperarm01.{s}", (sx * 0.12, -0.04, -1.0))
        aim(f"upperarm02.{s}", (sx * 0.1, -0.05, -1.0))
        aim(f"lowerarm01.{s}", (sx * 0.06, -0.3, -1.0))
        aim(f"lowerarm02.{s}", (sx * 0.05, -0.32, -1.0))
        grip = spec["grip"] == s
        curl = 68 if grip else 18
        for f in range(2, 6):
            for k in range(1, 4):
                pb = arm.pose.bones[f"finger{f}-{k}.{s}"]
                pb.rotation_mode = "XYZ"
                pb.rotation_euler = (math.radians(curl * (1.1 if k == 1 else 1.0)), 0, 0)
        for k in range(2, 4):
            pb = arm.pose.bones[f"finger1-{k}.{s}"]
            pb.rotation_mode = "XYZ"
            pb.rotation_euler = (math.radians(35 if grip else 10), 0, 0)
    bpy.context.view_layer.update()
    pose = {pb.name: (pb.head.copy(), pb.tail.copy()) for pb in arm.pose.bones}
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.modifier_apply(modifier="arm")
    # Vektene samles på spillets bein.
    ours = {}
    for vg in body.vertex_groups:
        ours.setdefault(our_bone(vg.name), []).append(vg.index)
    head_idx = {vg.index for vg in body.vertex_groups if our_bone(vg.name) == "head"}
    sums = [dict() for _ in body.data.vertices]
    for v in body.data.vertices:
        for g in v.groups:
            ob = our_bone(body.vertex_groups[g.group].name)
            sums[v.index][ob] = sums[v.index].get(ob, 0.0) + g.weight
    for vg in list(body.vertex_groups):
        body.vertex_groups.remove(vg)
    made_g = {}
    for i, d in enumerate(sums):
        tot = sum(d.values())
        if tot <= 0:
            continue
        for bname, w in d.items():
            if bname not in made_g:
                made_g[bname] = body.vertex_groups.new(name=bname)
            made_g[bname].add([i], w / tot, "REPLACE")
    _ = head_idx
    bpy.data.objects.remove(arm)
    # Til spillets akser: drei 90° om z (front −y → +x), skaler til høyden og
    # sett føttene på bakken.
    rot = Matrix.Rotation(math.radians(90), 4, "Z")
    zs = [(rot @ v.co).z for v in body.data.vertices]
    h = max(zs) - min(zs)
    k = spec["height"] / h
    lift = -min(zs) * k
    T = Matrix.Translation((0, 0, lift)) @ Matrix.Scale(k, 4) @ rot
    body.data.transform(T)
    for p in body.data.polygons:
        p.use_smooth = True
    joints = {n: (T @ a, T @ b) for n, (a, b) in pose.items()}
    eye = {s: T @ joint(f"joint-{s}-eye") for s in ("l", "r")}
    mouth = T @ joint("joint-mouth")
    return body, joints, eye, mouth


def game_bones(j):
    """Spillets 18 bein (samme navn og rekkefølge som npc.BONES) fra
    MakeHuman-leddene."""
    H = lambda n: tuple(j[n][0])  # noqa: E731
    T = lambda n: tuple(j[n][1])  # noqa: E731
    bones = [
        ("root", (0, 0, 0), (0, 0, 0.1), None),
        ("hips", H("spine05"), H("spine03"), "root"),
        ("spine", H("spine03"), H("spine01"), "hips"),
        ("chest", H("spine01"), H("neck01"), "spine"),
        ("neck", H("neck01"), H("head"), "chest"),
        ("head", H("head"), T("head"), "neck"),
    ]
    for s in ("L", "R"):
        bones += [
            (f"upper_arm.{s}", H(f"upperarm01.{s}"), H(f"lowerarm01.{s}"), "chest"),
            (f"forearm.{s}", H(f"lowerarm01.{s}"), H(f"wrist.{s}"), f"upper_arm.{s}"),
            (f"hand.{s}", H(f"wrist.{s}"), H(f"finger3-1.{s}"), f"forearm.{s}"),
        ]
    for s in ("L", "R"):
        bones += [
            (f"thigh.{s}", H(f"upperleg01.{s}"), H(f"lowerleg01.{s}"), "hips"),
            (f"shin.{s}", H(f"lowerleg01.{s}"), H(f"foot.{s}"), f"thigh.{s}"),
            (f"foot.{s}", H(f"foot.{s}"), H(f"toe3-1.{s}"), f"shin.{s}"),
        ]
    return bones


def dominant(body):
    """Det tyngste beinet for hvert hjørne."""
    names = {vg.index: vg.name for vg in body.vertex_groups}
    out = []
    for v in body.data.vertices:
        best = max(v.groups, key=lambda g: g.weight, default=None)
        out.append(names[best.group] if best else "")
    return out


def face_uv(ob, eye, mouth, photo):
    """Uv-laget «face»: fotoet projisert forfra (langs x), festet så
    øynene og munnen i fotoet (`photo`: eyes (u venstre, u høyre, v), mouth
    v) havner på øynene og munnen i modellen."""
    me = ob.data
    if "UVMap" not in me.uv_layers:
        me.uv_layers.new(name="UVMap")
    if "face" not in me.uv_layers:
        me.uv_layers.new(name="face")
    me.uv_layers.active = me.uv_layers["UVMap"]
    el, er = eye["l"], eye["r"]
    ul, ur, ve = photo["eyes"]
    # Personens venstre øye (+y) er til høyre i bildet.
    su = (el.y - er.y) / (ur - ul)
    mid_y = (el.y + er.y) / 2
    ez = (el.z + er.z) / 2
    sv = (ez - mouth.z) / (photo["mouth"] - ve)
    layer = me.uv_layers["face"]
    mw = ob.matrix_world
    for poly in me.polygons:
        for li in poly.loop_indices:
            p = mw @ me.vertices[me.loops[li].vertex_index].co
            u = (ul + ur) / 2 + (p.y - mid_y) / su
            v = ve + (ez - p.z) / sv
            layer.data[li].uv = (u, 1.0 - v)


def shell(body, name, pick, offset, thick, mat, shape=None):
    """Et klesplagg av kroppens egne flater: flatene der alle hjørnene
    oppfyller `pick(i, co)`, løftet `offset` ut langs normalen (og flyttet
    med `shape(co) -> co` om gitt) og gjort `thick` tykt."""
    bm = bmesh.new()
    bm.from_mesh(body.data)
    bm.verts.ensure_lookup_table()
    keep = [f for f in bm.faces if all(pick(v.index, v.co) for v in f.verts)]
    kill = [f for f in bm.faces if f not in set(keep)]
    bmesh.ops.delete(bm, geom=kill, context="FACES")
    bm.normal_update()
    for v in bm.verts:
        v.co = v.co + v.normal * offset
        if shape:
            v.co = shape(v.co)
    if thick:
        bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=thick)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    # Vektene følger med fra kroppen (nærmeste hjørne).
    copy_weights(body, ob)
    return ob


def shoe(body, name, side, mat, top=0.13):
    """En sko rundt foten: den konvekse innhyllingen av fotens hjørner,
    delt opp, glattet og litt større, så tærne ikke synes."""
    pts = [v.co.copy() for v in body.data.vertices if v.co.z < top and v.co.y * side > 0.02]
    bm = bmesh.new()
    vs = [bm.verts.new(p) for p in pts]
    bmesh.ops.convex_hull(bm, input=vs)
    loose = [v for v in bm.verts if not v.link_faces]
    bmesh.ops.delete(bm, geom=loose, context="VERTS")
    bmesh.ops.subdivide_edges(bm, edges=bm.edges[:], cuts=2, use_grid_fill=True)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    for _ in range(6):
        bmesh.ops.smooth_vert(bm, verts=bm.verts[:], factor=0.5, use_axis_x=True, use_axis_y=True, use_axis_z=True)
    c = sum((v.co for v in bm.verts), Vector()) / len(bm.verts)
    for v in bm.verts:
        d = v.co - c
        v.co = c + Vector((d.x * 1.06, d.y * 1.12, d.z * 1.04))
        v.co.z = max(v.co.z, 0.0)
    bm.normal_update()
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    copy_weights(body, ob)
    return ob


def smooth_low(ob, below, n):
    """Glatter ut hjørnene under `below` (tærne i en støvel)."""
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    vs = [v for v in bm.verts if v.co.z < below]
    for _ in range(n):
        bmesh.ops.smooth_vert(bm, verts=vs, factor=0.6, use_axis_x=True, use_axis_y=True, use_axis_z=False)
    bm.to_mesh(ob.data)
    bm.free()


def copy_weights(src, dst):
    tree = BVHTree.FromObject(src, bpy.context.evaluated_depsgraph_get())
    names = [vg.name for vg in src.vertex_groups]
    groups = {n: dst.vertex_groups.get(n) or dst.vertex_groups.new(name=n) for n in names}
    for v in dst.data.vertices:
        loc, _, fi, _ = tree.find_nearest(v.co)
        if fi is None:
            continue
        poly = src.data.polygons[fi]
        vi = min(poly.vertices, key=lambda i: (src.data.vertices[i].co - loc).length)
        for g in src.data.vertices[vi].groups:
            groups[names[g.group]].add([v.index], g.weight, "REPLACE")


def tube(name, a, b, r0, r1, mat, n=12):
    a, b = Vector(a), Vector(b)
    bm = bmesh.new()
    d = b - a
    q = d.to_track_quat("Z", "Y")
    rings = []
    for t, r in ((0.0, r0), (1.0, r1)):
        ring = [bm.verts.new(a + d * t + q @ Vector((math.cos(2 * math.pi * k / n) * r, math.sin(2 * math.pi * k / n) * r, 0))) for k in range(n)]
        rings.append(ring)
    for k in range(n):
        j = (k + 1) % n
        bm.faces.new((rings[0][k], rings[0][j], rings[1][j], rings[1][k]))
    bm.faces.new(rings[0][::-1])
    bm.faces.new(rings[1])
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    return ob


def block(name, center, size, mat, rot=(0, 0, 0), bevel=0.004):
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bmesh.ops.scale(bm, vec=Vector(size), verts=bm.verts)
    if bevel:
        bmesh.ops.bevel(bm, geom=bm.edges[:] + bm.verts[:], offset=bevel, segments=2, affect="EDGES")
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    ob.rotation_euler = rot
    ob.location = center
    return ob


def hanging_panel(body, name, z_top, z_bot, w_top, w_bot, mat, gap=0.012, rows=18, cols=12, thick=0.006):
    """Et forkle: et panel foran kroppen fra `z_top` til `z_bot`, som følger
    brystet og magen og så henger rett ned (ikke inn mellom beina)."""
    tree = BVHTree.FromObject(body, bpy.context.evaluated_depsgraph_get())
    bm = bmesh.new()
    grid = []
    reach = [-1.0] * (cols + 1)
    for r in range(rows + 1):
        t = r / rows
        z = z_top + (z_bot - z_top) * t
        half = w_top + (w_bot - w_top) * min(1.0, t * 1.5)
        row = []
        for c in range(cols + 1):
            y = (c / cols * 2 - 1) * half
            hit = tree.ray_cast(Vector((0.5, y, z)), Vector((-1, 0, 0)), 1.0)
            x = hit[0].x if hit[0] is not None else 0.05
            # Henger: aldri lenger inn enn det som er over.
            reach[c] = max(reach[c] - 0.004, x) if r else x
            row.append(bm.verts.new((reach[c] + gap, y, z)))
        grid.append(row)
    for r in range(rows):
        for c in range(cols):
            bm.faces.new((grid[r][c], grid[r][c + 1], grid[r + 1][c + 1], grid[r + 1][c]))
    # Sidekantene buer litt bakover rundt kroppen.
    for row in grid:
        for c in (0, cols):
            row[c].co.x -= 0.02
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=thick)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    # Forkleet henger stivt på ryggen (bake_textured: rigid «apron»).
    return ob


# --- Figurene ----------------------------------------------------------------


def patch_mean(path, box):
    """Snittfargen (lineær) i et utsnitt (x0, y0, x1, y1 i 0..1 fra øverst
    til venstre) av et bilde."""
    import numpy as np
    img = bpy.data.images.load(path, check_existing=True)
    w, h = img.size
    px = np.array(img.pixels[:]).reshape(h, w, 4)[::-1]
    x0, y0, x1, y1 = (int(box[0] * w), int(box[1] * h), int(box[2] * w), int(box[3] * h))
    return np.array(px[y0:y1, x0:x1, :3].reshape(-1, 3).mean(axis=0))


def project_uv(ob, layer, fn):
    """Et uv-lag der hvert hjørne får `fn(p) -> (u, v)` (v fra toppen)."""
    me = ob.data
    if "UVMap" not in me.uv_layers:
        me.uv_layers.new(name="UVMap")
    if layer not in me.uv_layers:
        me.uv_layers.new(name=layer)
    me.uv_layers.active = me.uv_layers["UVMap"]
    data = me.uv_layers[layer].data
    for poly in me.polygons:
        for li in poly.loop_indices:
            u, v = fn(me.vertices[me.loops[li].vertex_index].co)
            data[li].uv = (u, 1.0 - v)


def head_photos(body, spec, eye, mouth):
    """Uv-lagene for de fire fotoene: forfra festet i øynene og munnen, fra
    sidene i øyet og munnhøyden, bakfra i ørene."""
    ph = spec["photo"]
    el, er = eye["l"], eye["r"]
    ez = (el.z + er.z) / 2
    ex = (el.x + er.x) / 2
    ul, ur, ve = ph["eyes"]
    su = (el.y - er.y) / (ur - ul)
    sv = su
    mid_y = (el.y + er.y) / 2
    project_uv(body, "front", lambda p: ((ul + ur) / 2 + (p.y - mid_y) / su, ve + (ez - p.z) / sv))
    # Sidene: festet i øyet og midt i øret. Personens høyre side (−y) har
    # fronten mot høyre i bildet.
    side = ph["side"]
    ear_z = ez - 0.025
    near = [v.co for v in body.data.vertices if abs(v.co.z - ear_z) < 0.015 and v.co.x < ex - 0.03]
    wide = max(abs(c.y) for c in near)
    ear = [c for c in near if abs(c.y) > wide - 0.012]
    ear_x = sum(c.x for c in ear) / len(ear)
    ue, ve2 = side["eye"]
    ss = (ex - ear_x) / (ue - side["ear"])
    print(f"ankre: øyne x {ex:.3f} z {ez:.3f}, øret x {ear_x:.3f}, forfra {su:.3f} m per bilde, siden {ss:.3f}")
    project_uv(body, "right", lambda p: (ue + (p.x - ex) / ss, ve2 + (ez - p.z) / ss))
    project_uv(body, "left", lambda p: ((1 - ue) - (p.x - ex) / ss, ve2 + (ez - p.z) / ss))
    # Bakfra: personens venstre (+y) er til venstre i bildet; festet i ørene.
    back = ph["back"]
    ear_z = ez - 0.025
    head = [v.co for v in body.data.vertices if abs(v.co.z - ear_z) < 0.012 and v.co.x < ex - 0.05]
    half = max(abs(c.y) for c in head) if head else 0.075
    bl, br, bv = back["ears"]
    sb = 2 * half / (br - bl)
    project_uv(body, "back", lambda p: ((bl + br) / 2 - p.y / sb, bv + (ear_z - p.z) / sb))
    return ear_x


def head_material2(name, spec, skin_tex, hair_tex, beard_tex, ez, mouth_z, ear_x):
    """Hodet: de fire fotoene blandet etter hvilken vei flaten vender
    (forfra, sidene, bakfra), og der ingen dekker (issen, under haka):
    hår- og skjeggflisene. Fotoene er fargejustert mot fotoet forfra."""
    ph = spec["photo"]
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    b = _bsdf(m)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    pos = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(tc.outputs["Object"], pos.inputs[0])
    geo = nt.nodes.new("ShaderNodeNewGeometry")
    nrm = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(geo.outputs["Normal"], nrm.inputs[0])
    # Bunnen: hud, hår på issen og bakhodet, skjegg under haka.
    st = _boxtex(nt, skin_tex, 0.08, tc.outputs["Object"])
    col = st.outputs["Color"]
    ht = _boxtex(nt, hair_tex, 0.12, tc.outputs["Object"])
    top = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["Z"], 0.25, 0.6), _ramp(nt, pos.outputs["Z"], ez, ez + 0.03))
    # Bak øret og over det: hår.
    behind = _math(nt, "MULTIPLY", _ramp(nt, pos.outputs["X"], ear_x + 0.03, ear_x), _ramp(nt, pos.outputs["Z"], ez - 0.06, ez - 0.03))
    col = _mix(nt, col, ht.outputs["Color"], _math(nt, "MAXIMUM", top, behind))
    if beard_tex:
        bt = _boxtex(nt, beard_tex, 0.1, tc.outputs["Object"])
        under = _math(nt, "MULTIPLY", _ramp(nt, pos.outputs["Z"], mouth_z, mouth_z - 0.02), _ramp(nt, pos.outputs["X"], -0.06, -0.02))
        col = _mix(nt, col, bt.outputs["Color"], under)
    ref = patch_mean(ph["file"], ph["cheek"])

    def photo(uvname, path, weight, patch):
        uv = nt.nodes.new("ShaderNodeUVMap")
        uv.uv_map = uvname
        img = nt.nodes.new("ShaderNodeTexImage")
        img.image = bpy.data.images.load(path, check_existing=True)
        img.extension = "EXTEND"
        nt.links.new(uv.outputs["UV"], img.inputs["Vector"])
        sep = nt.nodes.new("ShaderNodeSeparateXYZ")
        nt.links.new(uv.outputs["UV"], sep.inputs[0])
        inside = _math(nt, "MULTIPLY", _ramp(nt, sep.outputs["Y"], 0.0, 0.04), _ramp(nt, sep.outputs["Y"], 1.0, 0.96))
        inside = _math(nt, "MULTIPLY", inside, _math(nt, "MULTIPLY", _ramp(nt, sep.outputs["X"], 0.02, 0.06), _ramp(nt, sep.outputs["X"], 0.98, 0.94)))
        gain = tuple(float(min(1.4, max(0.7, r / max(1e-4, c)))) for r, c in zip(ref, patch_mean(path, patch)))
        c = _mix(nt, img.outputs["Color"], gain, 1.0, "MULTIPLY")
        return c, _math(nt, "MULTIPLY", weight, inside)

    side = ph["side"]
    back = ph["back"]
    w_back = _ramp(nt, nrm.outputs["X"], -0.25, -0.6)
    # Profilfotoene bare fra litt bak øret og fram.
    hi = _ramp(nt, pos.outputs["X"], ear_x - 0.045, ear_x - 0.02)
    lo = _ramp(nt, pos.outputs["X"], ear_x + 0.0, ear_x + 0.025)
    fwd = _mix_f(nt, lo, hi, _ramp(nt, pos.outputs["Z"], ez - 0.07, ez - 0.04))
    fwd = _math(nt, "MULTIPLY", fwd, _ramp(nt, pos.outputs["Z"], ez + 0.085, ez + 0.06))
    w_right = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["Y"], -0.2, -0.55), fwd)
    w_left = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["Y"], 0.2, 0.55), fwd)
    # Fotoet forfra bare over ansiktet: innenfor kinnene og under issen.
    ay = _math(nt, "ABSOLUTE", pos.outputs["Y"])
    w_front = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["X"], 0.3, 0.65), _ramp(nt, ay, 0.072, 0.058))
    w_front = _math(nt, "MULTIPLY", w_front, _ramp(nt, pos.outputs["Z"], ez + 0.085, ez + 0.06))
    for uvname, path, w, patch in (("back", back["file"], w_back, back["skin"]), ("right", side["file_r"], w_right, side["skin_r"]), ("left", side["file_l"], w_left, side["skin_l"]), ("front", ph["file"], w_front, ph["cheek"])):
        c, f = photo(uvname, path, w, patch)
        col = _mix(nt, col, c, f)
    nt.links.new(col, b.inputs["Base Color"])
    b.inputs["Roughness"].default_value = 0.55
    return m


SMITH = {"name": "smith", "macro": (1.0, 0.45, 0.7, 0.25), "height": 1.8, "grip": "R",
         "photo": {"file": f"{FACES}/smed.jpg", "eyes": (0.373, 0.627, 0.49), "mouth": 0.79, "cheek": (0.27, 0.55, 0.35, 0.66),
                   "side": {"file_r": f"{FACES}/smed_hoyre.jpg", "file_l": f"{FACES}/smed_venstre.jpg", "eye": (0.635, 0.352), "ear": 0.35,
                            "skin_r": (0.5, 0.37, 0.58, 0.45), "skin_l": (0.42, 0.37, 0.5, 0.45)},
                   "back": {"file": f"{FACES}/smed_bak.jpg", "ears": (0.275, 0.7, 0.415), "skin": (0.42, 0.6, 0.58, 0.66)}},
         "tiles": {"skin": f"{TEX}/people/smed_hud.jpg", "hair": f"{TEX}/people/smed_haar.jpg", "beard": f"{TEX}/people/smed_skjegg.jpg"}}
INNKEEPER = {"name": "innkeeper", "macro": (0.0, 0.45, 0.2, 0.45), "height": 1.66, "grip": "R",
             "photo": {"file": f"{FACES}/vertinne.jpg", "eyes": (0.402, 0.598, 0.41), "mouth": 0.615, "cheek": (0.33, 0.47, 0.4, 0.56)}}


def dress_head(body, spec, eye, mouth, hair, beard):
    """Hode og hals får ansiktsmaterialet; resten av kroppen huden (et
    utsnitt av kinnet i fotoet, så hudfargen stemmer)."""
    dom = dominant(body)
    photo = spec["photo"]
    cheek = photo["cheek"]
    skin_tex = crop(photo["file"], cheek, f"skin_{spec['name']}")
    tone = (1.0, 1.0, 1.0)
    skin = skin_material("skin", skin_tex, tone)
    face = head_material("face", photo["file"], skin_tex, tone, hair=hair, beard=beard)
    body.data.materials.append(skin)
    body.data.materials.append(face)
    for p in body.data.polygons:
        p.material_index = 1 if all(dom[i] in ("head", "neck") for i in p.vertices) else 0
    face_uv(body, eye, mouth, photo)
    return dom


def smith():
    spec = SMITH
    body, j, eye, mouth = build_human(spec)
    ez = (eye["l"].z + eye["r"].z) / 2
    t = spec["tiles"]
    dom = dominant(body)
    body.data.materials.append(skin_material("skin", t["skin"], (0.86, 0.8, 0.76), scale=0.22))
    ear_x = head_photos(body, spec, eye, mouth)
    body.data.materials.append(head_material2("face", spec, t["skin"], t["hair"], t["beard"], ez, ez - 0.065, ear_x))
    for p in body.data.polygons:
        p.material_index = 1 if all(dom[i] in ("head", "neck") for i in p.vertices) else 0
    belt_z = j["spine04"][0].z + 0.02
    # Linskjorta: overkroppen og overarmene, ermene brettet opp over albuen.
    linen = cloth("shirt", "linen", lin((0.8, 0.74, 0.62)), 0.3, dirt=(1.2, 0.3))
    torso = {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R"}
    neck_z = j["neck01"][0].z - 0.01
    shell(body, "shirt", lambda i, co: dom[i] in torso and co.z < neck_z and co.z > belt_z - 0.12, 0.007, 0.004, linen)
    # Ullbuksa: hofter og lår ned til støvlene.
    wool = cloth("trousers", "wool", lin((0.3, 0.26, 0.21)), 0.3, dirt=(0.6, 0.6))
    shell(body, "trousers", lambda i, co: (dom[i] in {"hips", "thigh.L", "thigh.R", "shin.L", "shin.R"}) and co.z < belt_z + 0.04 and co.z > 0.25, 0.005, 0.003, wool)
    # Støvlene: føttene og leggen opp til midt på.
    leather = cloth("boots", "leather", lin((0.36, 0.25, 0.17)), 0.2, rough=0.6, dirt=(0.25, 0.5))
    shell(body, "boots", lambda i, co: 0.07 < co.z < 0.33, 0.012, 0.004, leather)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, leather)
    # Lærforkleet fra brystet til knærne, med nakkestropp.
    apron = cloth("apron", "leather", lin((0.4, 0.28, 0.18)), 0.25, rough=0.55, bump=0.6, dirt=(1.3, 0.45))
    top = j["spine01"][0].z + 0.08
    hanging_panel(body, "apron", top, j["lowerleg01.L"][0].z + 0.02, 0.11, 0.19, apron, gap=0.02)
    for sy in (-1, 1):
        a = Vector((0.0, sy * 0.1, top)) + Vector((0.11, 0, 0))
        tube(f"apron_strap{sy}", a, j["neck01"][0] + Vector((0.0, sy * 0.055, 0.02)), 0.008, 0.008, apron)
    # Beltet.
    belt = cloth("belt", "leather", lin((0.28, 0.19, 0.12)), 0.15, rough=0.5)
    shell(body, "belt", lambda i, co: abs(co.z - belt_z) < 0.022 and dom[i] in {"hips", "spine"}, 0.03, 0.006, belt)
    # Hammeren i høyre hånd: skaftet gjennom grepet, hodet nede.
    iron = cloth("iron", "metal", lin((0.62, 0.6, 0.58)), 0.2, rough=0.45, bump=0.2)
    oak = cloth("handle", "oak", lin((0.75, 0.6, 0.45)), 0.15, rough=0.6)
    w, m3 = j["wrist.R"][0], j["finger3-1.R"][0]
    grip = w + (m3 - w) * 1.25 + Vector((0.0, 0.0, 0.0))
    tube("tool_handle", grip + Vector((-0.02, 0, 0.09)), grip + Vector((0.05, 0, -0.26)), 0.014, 0.012, oak)
    block("tool_head", grip + Vector((0.055, 0, -0.28)), (0.07, 0.04, 0.13), iron, rot=(0, 0.19, 0), bevel=0.006)
    return j


MODELS = {"smith": smith}


def preview(out, focus):
    """Rendrer figuren rett fra Blender (Cycles), helfigur og nær ansiktet."""
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 24
    scene.render.resolution_x, scene.render.resolution_y = 900, 900
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[0].default_value = (0.55, 0.6, 0.68, 1)
    world.node_tree.nodes["Background"].inputs[1].default_value = 0.8
    scene.world = world
    bpy.ops.object.light_add(type="SUN", location=(3, -2, 5))
    sun = bpy.context.active_object
    sun.data.energy = 3.0
    sun.rotation_euler = (math.radians(50), math.radians(10), math.radians(60))
    for name, loc, look, lens in (("full", (2.8, -1.2, 1.1), (0, 0, 0.92), 50), ("face", (0.75, -0.28, focus), (0, 0, focus - 0.02), 85), ("right", (0.0, -0.8, focus), (0, 0, focus - 0.02), 85), ("back", (-0.8, 0.0, focus), (0, 0, focus - 0.02), 85)):
        bpy.ops.object.camera_add(location=loc)
        cam = bpy.context.active_object
        cam.data.lens = lens
        d = Vector(look) - Vector(loc)
        cam.rotation_euler = d.to_track_quat("-Z", "Y").to_euler()
        scene.camera = cam
        scene.render.filepath = out.replace(".png", f"_{name}.png")
        bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    args = sys.argv[1:]
    name = next(a for a in args if a in MODELS)
    size = next((int(a) for a in args if a.isdigit()), 2048)
    clear()
    j = MODELS[name]()
    for ob in bpy.data.objects:
        if ob.type == "MESH" and "UVMap" not in ob.data.uv_layers:
            ob.data.uv_layers.new(name="UVMap")
    if "--preview" in args:
        preview(args[args.index("--preview") + 1], j["head"][0].z + 0.08)
        sys.exit(0)
    rig = armature(f"{name}_rig", game_bones(j))
    clips(rig)
    bake_textured(f"{OUT}/npc_{name}.glb", size, keep=("glass", "glow", "fire", "mesh"), armature=rig, rigid={"tool": "hand.R", "mug": "hand.R", "apron": "spine"}, skin={"blend": 0.05})
