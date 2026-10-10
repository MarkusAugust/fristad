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

MakeHuman-dataene ligger i .claude/makehuman/ (hentes på nytt fra
github.com/makehumancommunity/mpfb2, CC0). Origo i bakken mellom føttene,
fronten mot +X, venstre mot +Y."""

import gzip
import json
import math
import os
import random
import subprocess
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import armature, bake_textured, clear  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402
import folk_motion  # noqa: E402

OUT = "assets/models/npc"
TEX = os.path.abspath("assets/textures")
FACES = os.path.abspath("assets/textures/faces")
ATLAS = os.path.abspath("assets/models/npc")
MH = os.path.abspath(".claude/makehuman")


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
    _bsdf(m).inputs["Roughness"].default_value = 0.64
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
        # `arm_dirs`: retningene for overarmen og underarmen per side i
        # MakeHumans rom (x venstre, −y fram, z opp), ellers ned langs siden.
        ua, fa = spec.get("arm_dirs", {}).get(s, ((sx * 0.12, -0.04, -1.0), (sx * 0.06, -0.3, -1.0)))
        aim(f"upperarm01.{s}", ua)
        aim(f"upperarm02.{s}", ua)
        aim(f"lowerarm01.{s}", fa)
        aim(f"lowerarm02.{s}", fa)
        if "hand_dirs" in spec:
            aim(f"wrist.{s}", spec["hand_dirs"][s])
        # `roll`: vri underarmen om sin egen akse (grader), så knyttneven
        # står riktig rundt grepet.
        roll = spec.get("roll", {}).get(s, 0.0)
        if roll:
            for bn in (f"lowerarm01.{s}", f"lowerarm02.{s}"):
                pb = arm.pose.bones[bn]
                bpy.context.view_layer.update()
                ax = (pb.tail - pb.head).normalized()
                head = pb.matrix.translation.copy()
                rot = Matrix.Rotation(math.radians(roll / 2), 4, ax)
                pb.matrix = Matrix.Translation(head) @ rot @ Matrix.Translation(-head) @ pb.matrix
                bpy.context.view_layer.update()
        grip = s in spec["grip"]
        # `curl`: grader per ledd, felles eller per finger ({2: …, 5: …}).
        curl = spec.get("curl", 68) if grip else 18
        for f in range(2, 6):
            c = curl[f] if isinstance(curl, dict) else curl
            for k in range(1, 4):
                pb = arm.pose.bones[f"finger{f}-{k}.{s}"]
                pb.rotation_mode = "XYZ"
                pb.rotation_euler = (math.radians(c * (1.1 if k == 1 else 1.0)), 0, 0)
        for k in range(2, 4):
            pb = arm.pose.bones[f"finger1-{k}.{s}"]
            pb.rotation_mode = "XYZ"
            pb.rotation_euler = (math.radians(spec.get("thumb", 35) if grip else 10), 0, 0)
        # `thumb_in`: tommelen føres inn over fingrene (rundt grepet).
        if grip and "thumb_in" in spec:
            pb = arm.pose.bones[f"finger1-1.{s}"]
            pb.rotation_mode = "XYZ"
            pb.rotation_euler = tuple(math.radians(v) for v in spec["thumb_in"])
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


# Flatene på kroppen som et klesplagg ligger over (kroppsnavn → flater).
COVERED = {}


def hide_covered(body):
    """Fjerner huden under klærne: flatene et plagg ligger over, når alle
    naboene også er dekket (én rad hud blir igjen under kanten, så det ikke
    blir hull når plagget bøyer seg). Ingen ser dem, og de tok plass i
    teksturen fra ansiktet."""
    covered = COVERED.get(body.name)
    if not covered:
        return
    bm = bmesh.new()
    bm.from_mesh(body.data)
    bm.faces.ensure_lookup_table()
    kill = [f for f in bm.faces if f.index in covered and all(g.index in covered for e in f.edges for g in e.link_faces)]
    bmesh.ops.delete(bm, geom=kill, context="FACES")
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
    bm.to_mesh(body.data)
    bm.free()
    print(f"hud under klærne: {len(kill)} flater fjernet")


def shell(body, name, pick, offset, thick, mat, shape=None):
    """Et klesplagg av kroppens egne flater: flatene der alle hjørnene
    oppfyller `pick(i, co)`, løftet `offset` ut langs normalen (og flyttet
    med `shape(co) -> co` om gitt) og gjort `thick` tykt."""
    bm = bmesh.new()
    bm.from_mesh(body.data)
    bm.verts.ensure_lookup_table()
    keep = [f for f in bm.faces if all(pick(v.index, v.co) for v in f.verts)]
    COVERED.setdefault(body.name, set()).update(f.index for f in keep)
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
    # Venstre profil: egne ankre om fotoet ikke er et rent speilbilde.
    ul2, vl2 = side.get("eye_l", (1 - ue, ve2))
    sl = (ex - ear_x) / (side.get("ear_l", 1 - side["ear"]) - ul2)
    project_uv(body, "left", lambda p: (ul2 - (p.x - ex) / sl, vl2 + (ez - p.z) / sl))
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
    top = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["Z"], 0.15, 0.45), _ramp(nt, pos.outputs["Z"], ez, ez + 0.03))
    # Bak øret og over det: hår.
    behind = _math(nt, "MULTIPLY", _ramp(nt, pos.outputs["X"], ear_x + 0.03, ear_x), _ramp(nt, pos.outputs["Z"], ez - 0.06, ez - 0.03))
    if ph.get("crown") == "skin":
        # Skallet isse (munken): håret bare i kransen rundt, ikke på toppen.
        hair = _math(nt, "MULTIPLY", behind, _ramp(nt, nrm.outputs["Z"], 0.6, 0.35))
    else:
        hair = _math(nt, "MAXIMUM", top, behind)
    col = _mix(nt, col, ht.outputs["Color"], hair)
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
    # ... og ikke på nesa og ansiktet (der tar fotoet forfra over).
    fwd = _math(nt, "MULTIPLY", fwd, _ramp(nt, pos.outputs["X"], ear_x + 0.085, ear_x + 0.06))
    w_right = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["Y"], -0.2, -0.55), fwd)
    w_left = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["Y"], 0.2, 0.55), fwd)
    # Bakfotoet ikke over issen (der tar hår- eller skautflisen over).
    w_back = _math(nt, "MULTIPLY", w_back, _ramp(nt, nrm.outputs["Z"], 0.55, 0.3))
    # Fotoet forfra bare over ansiktet: innenfor kinnene og under issen.
    ay = _math(nt, "ABSOLUTE", pos.outputs["Y"])
    w_front = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["X"], 0.3, 0.65), _ramp(nt, ay, 0.072, 0.058))
    w_front = _math(nt, "MULTIPLY", w_front, _ramp(nt, pos.outputs["Z"], ez + 0.085, ez + 0.06))
    for uvname, path, w, patch in (("back", back["file"], w_back, back["skin"]), ("right", side["file_r"], w_right, side["skin_r"]), ("left", side["file_l"], w_left, side["skin_l"]), ("front", ph["file"], w_front, ph["cheek"])):
        c, f = photo(uvname, path, w, patch)
        col = _mix(nt, col, c, f)
    nt.links.new(col, b.inputs["Base Color"])
    # Huden er matt med litt glans, håret mattere: med fast 0,55 glinset
    # issen og håret som plast i spillet.
    nt.links.new(_mix_f(nt, 0.64, 0.8, hair), b.inputs["Roughness"])
    return m


SMITH = {"name": "smith", "macro": (1.0, 0.45, 0.7, 0.25), "height": 1.8, "grip": "R",
         "photo": {"file": f"{FACES}/smed.jpg", "eyes": (0.373, 0.627, 0.49), "mouth": 0.79, "cheek": (0.27, 0.55, 0.35, 0.66),
                   "side": {"file_r": f"{FACES}/smed_hoyre.jpg", "file_l": f"{FACES}/smed_venstre.jpg", "eye": (0.635, 0.352), "ear": 0.35,
                            "skin_r": (0.5, 0.37, 0.58, 0.45), "skin_l": (0.42, 0.37, 0.5, 0.45)},
                   "back": {"file": f"{FACES}/smed_bak.jpg", "ears": (0.275, 0.7, 0.415), "skin": (0.42, 0.6, 0.58, 0.66)}},
         "tiles": {"skin": f"{TEX}/people/smed_hud.jpg", "hair": f"{TEX}/people/smed_haar.jpg", "beard": f"{TEX}/people/smed_skjegg.jpg"}}
INNKEEPER = {"name": "innkeeper", "macro": (0.0, 0.45, 0.2, 0.45), "height": 1.66, "grip": "R",
             "photo": {"file": f"{FACES}/vertinne.jpg", "eyes": (0.402, 0.598, 0.41), "mouth": 0.615, "cheek": (0.33, 0.47, 0.4, 0.56),
                       "side": {"file_r": f"{FACES}/vertinne_hoyre.jpg", "file_l": f"{FACES}/vertinne_venstre.jpg", "eye": (0.685, 0.35), "ear": 0.44,
                                "skin_r": (0.6, 0.5, 0.68, 0.6), "skin_l": (0.32, 0.5, 0.4, 0.6)},
                       "back": {"file": f"{FACES}/vertinne_bak.jpg", "ears": (0.29, 0.665, 0.305), "skin": (0.4, 0.66, 0.6, 0.7)}},
             "tiles": {"skin": f"{TEX}/people/vertinne_hud.jpg", "hair": f"{TEX}/people/vertinne_skaut.jpg", "beard": None}}


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


def skirt(body, name, z_top, z_bot, mat, flare=0.12, rows=16, n=40, legs=False):
    """Et skjørt: ringer fra livet ned mot anklene, utenfor kroppen og
    beina, som vider seg ut nedover. Henger stivt på hoftene (rigid
    «skirt»)."""
    if legs:
        # Bare overkroppen og beina, så armene og hendene ikke tar med seg skjørtet.
        dom = dominant(body)
        part = {"hips", "spine", "thigh.L", "thigh.R", "shin.L", "shin.R"}
        polys = [list(p.vertices) for p in body.data.polygons if all(dom[i] in part for i in p.vertices)]
        tree = BVHTree.FromPolygons([v.co for v in body.data.vertices], polys)
    else:
        tree = BVHTree.FromObject(body, bpy.context.evaluated_depsgraph_get())
    bm = bmesh.new()
    rings = []
    rad = [0.0] * n
    for r in range(rows + 1):
        t = r / rows
        z = z_top + (z_bot - z_top) * t
        ring = []
        for k in range(n):
            a = 2 * math.pi * k / n
            d = Vector((math.cos(a), math.sin(a), 0))
            hit = tree.ray_cast(Vector((0, 0, z)) + d * 0.6, -d, 0.6)
            reach = (hit[0] - Vector((0, 0, z))).length if hit[0] is not None else 0.1
            rad[k] = max(rad[k], reach + 0.015) if r else reach + 0.015
            rr = rad[k] + flare * t * t
            ring.append(bm.verts.new(Vector((0, 0, z)) + d * rr))
        rings.append(ring)
    for r in range(rows):
        for k in range(n):
            j = (k + 1) % n
            bm.faces.new((rings[r][k], rings[r][j], rings[r + 1][j], rings[r + 1][k]))
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.006)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    return ob


def innkeeper():
    spec = INNKEEPER
    body, j, eye, mouth = build_human(spec)
    ez = (eye["l"].z + eye["r"].z) / 2
    t = spec["tiles"]
    dom = dominant(body)
    body.data.materials.append(skin_material("skin", t["skin"], (0.95, 0.9, 0.86), scale=0.22))
    ear_x = head_photos(body, spec, eye, mouth)
    body.data.materials.append(head_material2("face", spec, t["skin"], t["hair"], None, ez, ez - 0.06, ear_x))
    for p in body.data.polygons:
        p.material_index = 1 if all(dom[i] in ("head", "neck") for i in p.vertices) else 0
    belt_z = j["spine04"][0].z + 0.02
    neck_z = j["neck01"][0].z - 0.01
    # Linserken: overkroppen og armene ned til håndleddet.
    linen = cloth("chemise", "linen", lin((0.85, 0.81, 0.72)), 0.3, dirt=(1.1, 0.25))
    wrist_z = j["wrist.R"][0].z + 0.03
    shell(body, "chemise", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R", "forearm.L", "forearm.R"} and co.z < neck_z and co.z > wrist_z, 0.006, 0.003, linen)
    # Ullkjolen: livet over serken, og skjørtet ned mot anklene.
    wool = cloth("dress", "wool", lin((0.3, 0.22, 0.16)), 0.3, dirt=(0.5, 0.5))
    top = j["spine01"][0].z + 0.07
    shell(body, "bodice", lambda i, co: dom[i] in {"hips", "spine", "chest"} and belt_z - 0.05 < co.z < top, 0.014, 0.004, wool)
    skirt(body, "skirt", belt_z - 0.02, 0.07, wool, flare=0.1)
    # Lærselene over skuldrene.
    leather = cloth("straps", "leather", lin((0.42, 0.26, 0.16)), 0.15, rough=0.5)
    for sy in (-1, 1):
        tube(f"bodice_strap{sy}", Vector((0.1, sy * 0.08, top - 0.02)), Vector((0.0, sy * 0.1, j["neck01"][0].z - 0.02)), 0.012, 0.012, leather)
        tube(f"bodice_strapb{sy}", Vector((0.0, sy * 0.1, j["neck01"][0].z - 0.02)), Vector((-0.1, sy * 0.08, top - 0.02)), 0.012, 0.012, leather)
    # Linforkleet foran skjørtet, og skoene.
    apron = cloth("apron", "linen", lin((0.82, 0.78, 0.7)), 0.3, dirt=(0.8, 0.35))
    hanging_panel(body, "apron", belt_z, 0.3, 0.13, 0.2, apron, gap=0.13)
    shoes = cloth("shoes", "leather", lin((0.3, 0.21, 0.14)), 0.2, rough=0.6)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, shoes)
    # Ølkruset av tre i høyre hånd.
    wood = cloth("mug", "oak", lin((0.7, 0.55, 0.4)), 0.12, rough=0.6)
    w, m3 = j["wrist.R"][0], j["finger3-1.R"][0]
    grip = w + (m3 - w) * 1.2
    c = grip + Vector((0.035, 0.0, -0.02))
    tube("mug_body", c + Vector((0, 0, -0.07)), c + Vector((0, 0, 0.06)), 0.042, 0.046, wood, n=16)
    tube("mug_band", c + Vector((0, 0, 0.02)), c + Vector((0, 0, 0.032)), 0.049, 0.049, cloth("mug_iron", "metal", lin((0.4, 0.38, 0.36)), 0.1), n=16)
    return j


# Kona på torget: fotoet forfra er «kvinne», sidene og bakhodet lånes fra
# Vertinnen (samme skaut), og flisene likeså.
WOMAN = {"name": "woman", "macro": (0.0, 0.55, 0.15, 0.35), "height": 1.63, "grip": "R",
         "photo": {"file": f"{FACES}/kvinne.jpg", "eyes": (0.396, 0.613, 0.449), "mouth": 0.684, "cheek": (0.33, 0.52, 0.4, 0.6),
                   "side": {"file_r": f"{FACES}/vertinne_hoyre.jpg", "file_l": f"{FACES}/vertinne_venstre.jpg", "eye": (0.685, 0.35), "ear": 0.44,
                            "skin_r": (0.6, 0.5, 0.68, 0.6), "skin_l": (0.32, 0.5, 0.4, 0.6)},
                   "back": {"file": f"{FACES}/vertinne_bak.jpg", "ears": (0.29, 0.665, 0.305), "skin": (0.4, 0.66, 0.6, 0.7)}},
         "tiles": {"skin": f"{TEX}/people/vertinne_hud.jpg", "hair": f"{TEX}/people/vertinne_skaut.jpg", "beard": None}}


def woman():
    """Kona på torget: linserk, kjole av blåfarget ull med opprullede
    ermer, et grått overforkle og en trebøtte i hånda."""
    spec = WOMAN
    body, j, ez, dom, _ = dress_photos(spec, (0.93, 0.86, 0.8))
    belt_z = j["spine04"][0].z + 0.02
    neck_z = j["neck01"][0].z - 0.01
    elbow_z = j["lowerarm01.R"][0].z if "lowerarm01.R" in j else j["wrist.R"][0].z + 0.2
    linen = cloth("chemise", "linen", lin((0.8, 0.76, 0.66)), 0.3, dirt=(1.1, 0.3))
    shell(body, "chemise", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R"} and elbow_z < co.z < neck_z, 0.006, 0.003, linen)
    wool = cloth("dress", "wool", lin((0.2, 0.26, 0.36)), 0.3, dirt=(0.5, 0.55))
    top = j["spine01"][0].z + 0.06
    shell(body, "bodice", lambda i, co: dom[i] in {"hips", "spine", "chest"} and belt_z - 0.05 < co.z < top, 0.014, 0.004, wool)
    skirt(body, "skirt", belt_z - 0.02, 0.06, wool, flare=0.12)
    apron = cloth("apron", "linen", lin((0.55, 0.53, 0.5)), 0.3, dirt=(0.9, 0.4))
    hanging_panel(body, "apron", belt_z, 0.32, 0.14, 0.22, apron, gap=0.07)
    shoes = cloth("shoes", "leather", lin((0.26, 0.18, 0.12)), 0.2, rough=0.6)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, shoes)
    # Trebøtta med jernbånd, hengende i hanken fra høyre hånd.
    wood = cloth("tool_bucket", "oak", lin((0.62, 0.48, 0.34)), 0.12, rough=0.6)
    c = hand_grip(j) + Vector((0.0, 0.0, -0.2))
    tube("tool_bucket_body", c + Vector((0, 0, -0.14)), c + Vector((0, 0, 0.04)), 0.1, 0.12, wood, n=18)
    iron = cloth("tool_band", "metal", lin((0.36, 0.34, 0.32)), 0.1)
    for dz in (-0.1, 0.0):
        tube(f"tool_bucket_band{dz}", c + Vector((0, 0, dz)), c + Vector((0, 0, dz + 0.015)), 0.112 + dz * 0.1, 0.112 + dz * 0.1, iron, n=18)
    tube("tool_bucket_bail", c + Vector((0, -0.11, 0.04)), hand_grip(j), 0.006, 0.006, iron)
    tube("tool_bucket_bail2", c + Vector((0, 0.11, 0.04)), hand_grip(j), 0.006, 0.006, iron)
    return j


APOTHECARY = {"name": "apothecary", "macro": (1.0, 0.85, 0.15, 0.1), "height": 1.74, "grip": "R",
              "photo": {"file": f"{FACES}/gammel.jpg", "eyes": (0.372, 0.628, 0.515), "mouth": 0.825, "cheek": (0.32, 0.58, 0.4, 0.66),
                        "side": {"file_r": f"{FACES}/apotek_right.jpeg", "file_l": f"{FACES}/apotek_left.jpeg", "eye": (0.67, 0.385), "ear": 0.355,
                                 "eye_l": (0.325, 0.385), "ear_l": 0.66,
                                 "skin_r": (0.55, 0.45, 0.62, 0.52), "skin_l": (0.38, 0.45, 0.45, 0.52)},
                        "back": {"file": f"{FACES}/apotek_back.jpeg", "ears": (0.248, 0.752, 0.48), "skin": (0.44, 0.65, 0.56, 0.71)}},
              "tiles": {"skin": f"{FACES}/apotek_skin.jpeg", "hair": f"{FACES}/apotek_hair.jpeg", "beard": None}}
CLERK = {"name": "clerk", "macro": (1.0, 0.7, 0.2, 0.05), "height": 1.76, "grip": "R",
         "photo": {"file": f"{FACES}/munk.jpg", "eyes": (0.4, 0.584, 0.412), "mouth": 0.644, "cheek": (0.33, 0.46, 0.4, 0.54), "crown": "skin",
                   "side": {"file_r": f"{FACES}/scribe_right.jpeg", "file_l": f"{FACES}/scribe_left.jpeg", "eye": (0.685, 0.38), "ear": 0.395,
                            "eye_l": (0.315, 0.38), "ear_l": 0.62,
                            "skin_r": (0.55, 0.42, 0.62, 0.5), "skin_l": (0.38, 0.42, 0.45, 0.5)},
                   "back": {"file": f"{FACES}/scribe_back.jpeg", "ears": (0.278, 0.722, 0.41), "skin": (0.42, 0.5, 0.58, 0.57)}},
         "tiles": {"skin": f"{FACES}/scribe_skin.jpeg", "hair": f"{FACES}/scribe_hair.jpeg", "beard": None}}
# Gorvek: en hardbarket barbar i førtiårene, høyere, tyngre og mer
# muskuløs enn de andre i byen.
GORVEK = {"name": "gorvek", "macro": (1.0, 0.62, 1.0, 0.6), "height": 1.98, "grip": "R",
          "photo": {"file": f"{FACES}/gorvek_front.jpeg", "eyes": (0.418, 0.59, 0.385), "mouth": 0.575, "cheek": (0.34, 0.48, 0.4, 0.54),
                    "side": {"file_r": f"{FACES}/gorvek_right.jpeg", "file_l": f"{FACES}/gorvek_left.jpeg", "eye": (0.651, 0.38), "ear": 0.38,
                             "eye_l": (0.3, 0.38), "ear_l": 0.59,
                             "skin_r": (0.52, 0.42, 0.58, 0.48), "skin_l": (0.42, 0.42, 0.48, 0.48)},
                    "back": {"file": f"{FACES}/gorvek_back.jpeg", "ears": (0.314, 0.692, 0.348), "skin": (0.15, 0.7, 0.3, 0.8)}},
          "tiles": {"skin": f"{FACES}/gorvek_skin.jpeg", "hair": f"{FACES}/gorvek_hair.jpeg", "beard": f"{FACES}/gorvek_beard.jpeg"}}


def dress_photos(spec, skin_tone, body_mat=None):
    """Kroppen og hodet med eierens fotoer og fliser (som smeden): gir
    kroppen, leddene, øyehøyden og hvilket bein hvert hjørne følger."""
    body, j, eye, mouth = build_human(spec)
    ez = (eye["l"].z + eye["r"].z) / 2
    t = spec["tiles"]
    dom = dominant(body)
    body.data.materials.append(body_mat(body, j) if body_mat else skin_material("skin", t["skin"], skin_tone, scale=0.22))
    ear_x = head_photos(body, spec, eye, mouth)
    body.data.materials.append(head_material2("face", spec, t["skin"], t["hair"], t["beard"], ez, ez - 0.065, ear_x))
    for p in body.data.polygons:
        p.material_index = 1 if all(dom[i] in ("head", "neck") for i in p.vertices) else 0
    return body, j, ez, dom, ear_x


def hand_grip(j, k=1.22):
    w, m3 = j["wrist.R"][0], j["finger3-1.R"][0]
    return w + (m3 - w) * k


def apothecary():
    spec = APOTHECARY
    body, j, ez, dom, _ = dress_photos(spec, (0.9, 0.84, 0.8))
    belt_z = j["spine04"][0].z + 0.02
    neck_z = j["neck01"][0].z - 0.01
    wrist_z = j["wrist.R"][0].z + 0.03
    # Linskjorta under kjortelen, ermene ned til håndleddet.
    linen = cloth("shirt", "linen", lin((0.82, 0.78, 0.68)), 0.3, dirt=(1.1, 0.25))
    shell(body, "shirt", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R", "forearm.L", "forearm.R"} and wrist_z < co.z < neck_z, 0.005, 0.003, linen)
    # Den lange ullkjortelen i mosegrønt: livet og et skjørt til anklene.
    wool = cloth("robe", "wool", lin((0.36, 0.4, 0.27)), 0.3, dirt=(0.5, 0.55))
    shell(body, "robe_top", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R"} and belt_z - 0.06 < co.z < neck_z - 0.02, 0.012, 0.004, wool)
    skirt(body, "skirt", belt_z - 0.02, 0.06, wool, flare=0.09)
    # Lærbeltet med punger og en kniv.
    leather = cloth("belt", "leather", lin((0.3, 0.2, 0.12)), 0.15, rough=0.5)
    shell(body, "belt", lambda i, co: abs(co.z - belt_z) < 0.02 and dom[i] in {"hips", "spine"}, 0.03, 0.006, leather)
    for k, (dy, h) in enumerate(((0.1, 0.09), (-0.06, 0.07))):
        block(f"pouch{k}", Vector((0.1, dy, belt_z - h / 2 - 0.01)), (0.04, 0.08, h), leather, rot=(0, 0.1, 0), bevel=0.012)
    shoes = cloth("shoes", "leather", lin((0.28, 0.2, 0.13)), 0.2, rough=0.6)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, shoes)
    # En leirkrukke med tøy over munnen i høyre hånd.
    clay = cloth("tool_clay", "earth", lin((0.75, 0.5, 0.36)), 0.1, rough=0.7)
    c = hand_grip(j) + Vector((0.03, 0.0, -0.02))
    tube("tool_jar", c + Vector((0, 0, -0.08)), c + Vector((0, 0, 0.05)), 0.048, 0.04, clay, n=18)
    tube("tool_jar_lid", c + Vector((0, 0, 0.05)), c + Vector((0, 0, 0.065)), 0.042, 0.03, cloth("tool_cloth", "linen", lin((0.75, 0.7, 0.6)), 0.1), n=18)
    return j


def clerk():
    spec = CLERK
    body, j, ez, dom, _ = dress_photos(spec, (0.92, 0.86, 0.82))
    belt_z = j["spine04"][0].z + 0.02
    neck_z = j["neck01"][0].z - 0.01
    wrist_z = j["wrist.R"][0].z + 0.04
    # Kutta i brun ull: overkroppen og vide ermer, og skjørtet til føttene.
    wool = cloth("habit", "wool", lin((0.24, 0.18, 0.14)), 0.3, dirt=(0.45, 0.6))
    shell(body, "habit_top", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R", "forearm.L", "forearm.R"} and wrist_z < co.z < neck_z, 0.014, 0.004, wool)
    skirt(body, "skirt", belt_z - 0.02, 0.04, wool, flare=0.13)
    # Hetta ligger nede på skuldrene som en tykk krage.
    shell(body, "hood", lambda i, co: dom[i] in {"neck", "chest", "spine"} and neck_z - 0.11 < co.z < neck_z + 0.035, 0.035, 0.012, wool)
    # Taubeltet med ender som henger ned.
    rope = cloth("cord", "linen", lin((0.7, 0.62, 0.48)), 0.06, rough=0.8, bump=0.8)
    shell(body, "cord", lambda i, co: abs(co.z - belt_z) < 0.009 and dom[i] in {"hips", "spine"}, 0.03, 0.008, rope)
    for k, dy in enumerate((0.03, 0.06)):
        tube(f"cord_end{k}", Vector((0.11, dy, belt_z)), Vector((0.12, dy + 0.01, belt_z - 0.32 - k * 0.06)), 0.007, 0.006, rope, n=8)
    shoes = cloth("shoes", "leather", lin((0.26, 0.18, 0.12)), 0.2, rough=0.6)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, shoes)
    # Ei bok i lærbind med messinghjørner i høyre hånd.
    g = hand_grip(j, 1.15)
    cover = cloth("tool_cover", "leather", lin((0.32, 0.12, 0.08)), 0.1, rough=0.5)
    pages = cloth("tool_pages", "linen", lin((0.85, 0.8, 0.66)), 0.05)
    block("tool_book", g + Vector((0.03, 0.0, -0.06)), (0.17, 0.05, 0.23), cover, rot=(0, 0.15, 0), bevel=0.005)
    block("tool_pages", g + Vector((0.035, 0.0, -0.06)), (0.165, 0.04, 0.22), pages, rot=(0, 0.15, 0), bevel=0.002)
    return j


# --- Byfolkene -----------------------------------------------------------
# Bonden, gutten, vaktmannen, kremmeren og pilegrimen. Rundt 1350 gikk de
# fleste med hette (kaprun med kappe over skuldrene og en tipp bak), hatt
# eller hjelm, så bare fotoet forfra synes; sidene og bakhodet lånes fra
# en annen figur og havner under stoffet. Kremmeren, vaktmannen og
# pilegrimen låner ansikt fra Apotekeren, Smeden og Skriveren til eieren
# lager egne (docs/teksturer-neste.md).
def borrowed(name, front, eyes, mouth, cheek, macro, height, sides=SMITH):
    return {"name": name, "macro": macro, "height": height, "grip": "R",
            "photo": {"file": f"{FACES}/{front}", "eyes": eyes, "mouth": mouth, "cheek": cheek,
                      "side": sides["photo"]["side"], "back": sides["photo"]["back"]},
            "tiles": {"skin": sides["tiles"]["skin"], "hair": sides["tiles"]["hair"], "beard": None}}


FARMER = borrowed("farmer", "ung_mann.jpg", (0.396, 0.6, 0.473), 0.69, (0.33, 0.52, 0.4, 0.6), (1.0, 0.1, 0.45, 0.2), 1.78, APOTHECARY)
BOY = borrowed("boy", "gutt.jpg", (0.406, 0.59, 0.42), 0.62, (0.33, 0.48, 0.4, 0.56), (1.0, 0.0, 0.0, 0.0), 1.58, APOTHECARY)
GUARD = borrowed("guard", "smed.jpg", (0.373, 0.627, 0.49), 0.79, (0.27, 0.55, 0.35, 0.66), (1.0, 0.35, 0.65, 0.3), 1.82, SMITH)
MERCHANT = borrowed("merchant", "gammel.jpg", (0.372, 0.628, 0.515), 0.825, (0.32, 0.58, 0.4, 0.66), (1.0, 0.75, 0.1, 0.55), 1.72, APOTHECARY)
PILGRIM = borrowed("pilgrim", "munk.jpg", (0.4, 0.584, 0.412), 0.644, (0.33, 0.46, 0.4, 0.54), (1.0, 0.55, 0.2, 0.0), 1.75, CLERK)


def face_ellipse(j, ez):
    """Åpningen for ansiktet: en ellipse i yz-planet fra brynene til under
    haka, smal nok til at tinningene og ørene (det lånte håret) dekkes."""
    c = Vector((0.0, 0.0, ez - 0.035))
    ry, rz = 0.06, 0.095
    front = j["head"][0].x + 0.02
    return c, ry, rz, front


def hooded(body, name, j, ez, dom, below, offset, thick, mat):
    """Et plagg over hodet (hette eller brynjehette): stoffet spennes over
    hodet og halsen som et konvekst skall (over ørene og fra haka ned mot
    halsen, slik en ekte hette faller), med en rein, elliptisk åpning for
    ansiktet og en rullet kant rundt den."""
    c, ry, rz, front = face_ellipse(j, ez)
    inside = lambda co: co.x > front - 0.03 and ((co.y - c.y) / ry) ** 2 + ((co.z - c.z) / rz) ** 2 < 1.0  # noqa: E731
    pts = [v.co.copy() for i, v in enumerate(body.data.vertices) if dom[i] in {"head", "neck"} and v.co.z > below and not inside(v.co)]
    bm = bmesh.new()
    vs = [bm.verts.new(p) for p in pts]
    bmesh.ops.convex_hull(bm, input=vs)
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    for _ in range(3):
        bmesh.ops.subdivide_edges(bm, edges=[e for e in bm.edges if e.calc_length() > 0.02], cuts=1, use_grid_fill=True)
        bmesh.ops.triangulate(bm, faces=bm.faces[:])
    # Bunnen (snittet gjennom halsen) og ansiktet skjæres bort.
    cut = lambda f: inside(f.calc_center_median()) or all(v.co.z < below + 0.012 for v in f.verts)  # noqa: E731
    bmesh.ops.delete(bm, geom=[f for f in bm.faces if cut(f)], context="FACES")
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
    # Glatt ut fasettene og løft stoffet ut fra kroppen.
    tree = BVHTree.FromObject(body, bpy.context.evaluated_depsgraph_get())
    inner = [v for v in bm.verts if not v.is_boundary]
    for _ in range(4):
        bmesh.ops.smooth_vert(bm, verts=inner, factor=0.5, use_axis_x=True, use_axis_y=True, use_axis_z=True)
        for v in inner:
            loc, nrm, _, _ = tree.find_nearest(v.co)
            if loc is not None and (v.co - loc).dot(nrm) < offset:
                v.co = loc + nrm * offset
    bmesh.ops.smooth_vert(bm, verts=inner, factor=0.5, use_axis_x=True, use_axis_y=True, use_axis_z=True)
    bm.normal_update()
    # Kanten mot ansiktet legges på ellipsen, så den ikke blir trappete.
    rim = []
    for v in bm.verts:
        if v.is_boundary and v.co.z > c.z - rz * 1.3 and v.co.x > front - 0.04:
            k = math.hypot((v.co.y - c.y) / ry, (v.co.z - c.z) / rz) or 1.0
            v.co.y = c.y + (v.co.y - c.y) / k
            v.co.z = c.z + (v.co.z - c.z) / k
            rim.append(v.co.copy())
    for f in bm.faces:
        if f.normal.dot(f.calc_center_median() - Vector((j["head"][0].x, 0.0, ez))) < 0:
            f.normal_flip()
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    # Tykkelsen med modifikatoren (jevn langs normalene, uten spisser der
    # utglattingen har gjort flatene små).
    mod = ob.modifiers.new("thick", "SOLIDIFY")
    mod.thickness = thick
    mod.offset = -1.0
    ob.data = bpy.data.meshes.new_from_object(ob.evaluated_get(bpy.context.evaluated_depsgraph_get()))
    ob.modifiers.clear()
    copy_weights(body, ob)
    # Den rullede kanten: en lukket kurve gjennom kantpunktene.
    if len(rim) > 6:
        # Jevnt rundt ellipsen, med dybden (x) som snittet av kantpunktene
        # i hver sektor.
        nb = 40
        bins = [[] for _ in range(nb)]
        for q in rim:
            a = math.atan2((q.z - c.z) / rz, (q.y - c.y) / ry)
            bins[int((a + math.pi) / (2 * math.pi) * nb) % nb].append(q.x)
        xs = [sum(b) / len(b) if b else None for b in bins]
        known = [x for x in xs if x is not None]
        xs = [x if x is not None else min(known) for x in xs]
        rim = []
        for k in range(nb):
            a = -math.pi + (k + 0.5) / nb * 2 * math.pi
            x = sum(xs[(k + d) % nb] for d in range(-3, 4)) / 7
            rim.append(Vector((x, c.y + math.cos(a) * ry, c.z + math.sin(a) * rz)))
        cu = bpy.data.curves.new(name + "_rim", "CURVE")
        cu.dimensions = "3D"
        cu.bevel_depth = 0.011
        cu.bevel_resolution = 3
        sp = cu.splines.new("NURBS")
        sp.points.add(len(rim) - 1)
        for p, q in zip(sp.points, rim):
            p.co = (q.x - 0.004, q.y, q.z, 1.0)
        sp.use_cyclic_u = True
        sp.order_u = 4
        tmp = bpy.data.objects.new(name + "_rim", cu)
        bpy.context.scene.collection.objects.link(tmp)
        rme = bpy.data.meshes.new_from_object(tmp.evaluated_get(bpy.context.evaluated_depsgraph_get()))
        bpy.data.objects.remove(tmp)
        ro = bpy.data.objects.new(name + "_rim", rme)
        bpy.context.scene.collection.objects.link(ro)
        rme.materials.clear()
        rme.materials.append(mat)
        for p in rme.polygons:
            p.use_smooth = True
        copy_weights(body, ro)
    return ob


def dome(name, base, r, h, mat, n=32, rows=8):
    """En kuppel (hattepull eller hjelm): en halv ellipsoide over `base`."""
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=n, v_segments=rows * 2, radius=1.0)
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < -1e-4], context="VERTS")
    for v in bm.verts:
        v.co = base + Vector((v.co.x * r, v.co.y * r, v.co.z * h))
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    mod = ob.modifiers.new("thick", "SOLIDIFY")
    mod.thickness = 0.006
    ob.data = bpy.data.meshes.new_from_object(ob.evaluated_get(bpy.context.evaluated_depsgraph_get()))
    ob.modifiers.clear()
    return ob


def ball(name, c, r, mat, n=20):
    """En ellipsoide med halvaksene `r`."""
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=n, v_segments=n // 2 + 2, radius=1.0)
    for v in bm.verts:
        v.co = c + Vector((v.co.x * r[0], v.co.y * r[1], v.co.z * r[2]))
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    return ob


def brim(name, base, r_in, r_out, drop, mat, thick=0.008, n=40):
    """En hattebrem: en ring fra `r_in` til `r_out`, som heller `drop` ned
    mot kanten, med avrundet kant."""
    bm = bmesh.new()
    rings = []
    for t in (0.0, 0.33, 0.66, 1.0):
        r = r_in + (r_out - r_in) * t
        z = -drop * t * t
        rings.append([bm.verts.new(base + Vector((math.cos(2 * math.pi * k / n) * r, math.sin(2 * math.pi * k / n) * r, z))) for k in range(n)])
    for a, b in zip(rings, rings[1:]):
        for k in range(n):
            bm.faces.new((a[k], a[(k + 1) % n], b[(k + 1) % n], b[k]))
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = True
    mod = ob.modifiers.new("thick", "SOLIDIFY")
    mod.thickness = thick
    mod.offset = 0.0
    ob.data = bpy.data.meshes.new_from_object(ob.evaluated_get(bpy.context.evaluated_depsgraph_get()))
    ob.modifiers.clear()
    return ob


def hood(body, j, ez, dom, mat, tail=0.32):
    """Kaprunhette: over hodet og halsen, åpen for ansiktet, med en kappe
    over skuldrene og en tipp (liripipe) som henger ned på ryggen."""
    neck_z = j["neck01"][0].z - 0.01
    hooded(body, "hood", j, ez, dom, neck_z - 0.05, 0.012, 0.006, mat)
    # Kappa over skuldrene, ned til midt på overarmen.
    cape_z = neck_z - 0.17
    shell(body, "hood_cape", lambda i, co: dom[i] in {"neck", "chest", "spine", "upper_arm.L", "upper_arm.R"} and cape_z < co.z < neck_z + 0.02, 0.034, 0.006, mat)
    # Tippen bak, fra bakhodet ned over kappa.
    if tail:
        top = Vector((j["head"][0].x - 0.125, 0.0, ez - 0.01))
        tube("strap_chest_tail", top, top + Vector((-0.05, 0.0, -tail)), 0.024, 0.009, mat, n=14)


def dress_common(spec, tone):
    """Kroppen med fotoet, og ledd, øyehøyde og bein per hjørne."""
    body, j, ez, dom, _ = dress_photos(spec, tone)
    return body, j, ez, dom, j["spine04"][0].z + 0.02, j["neck01"][0].z - 0.01


def farmer():
    """Bonden: linskjorte med ermene brettet opp, kne-lang kjortel i
    jordbrun ull, hoser, lærbelte, kaprun i grått og en høygaffel."""
    spec = FARMER
    body, j, ez, dom, belt_z, neck_z = dress_common(spec, (0.92, 0.85, 0.8))
    elbow_z = j["wrist.R"][0].z + 0.2
    linen = cloth("shirt", "linen", lin((0.78, 0.72, 0.6)), 0.3, dirt=(1.1, 0.35))
    shell(body, "shirt", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R", "forearm.L", "forearm.R"} and elbow_z < co.z < neck_z, 0.005, 0.003, linen)
    wool = cloth("kirtle", "wool", lin((0.4, 0.3, 0.2)), 0.3, dirt=(0.7, 0.55))
    shell(body, "kirtle", lambda i, co: dom[i] in {"hips", "spine", "chest"} and belt_z - 0.05 < co.z < neck_z - 0.03, 0.013, 0.004, wool)
    skirt(body, "skirt", belt_z - 0.02, j["lowerleg01.L"][0].z + 0.06 if "lowerleg01.L" in j else 0.5, wool, flare=0.06, legs=True)
    hose = cloth("hose", "wool", lin((0.28, 0.27, 0.24)), 0.3, dirt=(0.5, 0.6))
    shell(body, "hose", lambda i, co: dom[i] in {"thigh.L", "thigh.R", "shin.L", "shin.R"} and 0.09 < co.z < belt_z - 0.05, 0.004, 0.002, hose)
    belt = cloth("belt", "leather", lin((0.3, 0.2, 0.12)), 0.15, rough=0.5)
    shell(body, "belt", lambda i, co: abs(co.z - belt_z) < 0.02 and dom[i] in {"hips", "spine"}, 0.03, 0.006, belt)
    shoes = cloth("shoes", "leather", lin((0.27, 0.19, 0.13)), 0.2, rough=0.6)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, shoes)
    hood(body, j, ez, dom, cloth("hood", "wool", lin((0.42, 0.4, 0.36)), 0.3, dirt=(0.3, 0.4)))
    # Høygaffelen: skaftet gjennom hånda, tre tinder av tre øverst.
    oak = cloth("tool_shaft", "oak", lin((0.7, 0.58, 0.44)), 0.15, rough=0.6)
    g = hand_grip(j)
    base = g + Vector((0.0, 0.0, -0.72))
    tip = g + Vector((0.04, 0.0, 0.85))
    tube("tool_shaft", base, tip, 0.016, 0.015, oak)
    for dy in (-0.05, 0.0, 0.05):
        tube(f"tool_tine{dy}", tip + Vector((0, 0, -0.02)), tip + Vector((0.02, dy, 0.24)), 0.009, 0.005, oak, n=8)
    tube("tool_bar", tip + Vector((0, -0.055, 0.0)), tip + Vector((0, 0.055, 0.0)), 0.01, 0.01, oak, n=8)
    return j


def boy():
    """Gutten: kort, grønn kjortel over linskjorta, hoser, kaprun i
    blått og en sekk over skulderen."""
    spec = BOY
    body, j, ez, dom, belt_z, neck_z = dress_common(spec, (0.94, 0.86, 0.8))
    wrist_z = j["wrist.R"][0].z + 0.03
    linen = cloth("shirt", "linen", lin((0.82, 0.78, 0.68)), 0.3, dirt=(1.0, 0.3))
    shell(body, "shirt", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R", "forearm.L", "forearm.R"} and wrist_z < co.z < neck_z, 0.005, 0.003, linen)
    wool = cloth("kirtle", "wool", lin((0.3, 0.38, 0.24)), 0.3, dirt=(0.7, 0.5))
    shell(body, "kirtle", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R"} and belt_z - 0.05 < co.z < neck_z - 0.03, 0.012, 0.004, wool)
    skirt(body, "skirt", belt_z - 0.02, belt_z - 0.3, wool, flare=0.05, legs=True)
    hose = cloth("hose", "wool", lin((0.36, 0.3, 0.24)), 0.3, dirt=(0.5, 0.6))
    shell(body, "hose", lambda i, co: dom[i] in {"thigh.L", "thigh.R", "shin.L", "shin.R"} and 0.09 < co.z < belt_z - 0.05, 0.004, 0.002, hose)
    belt = cloth("belt", "leather", lin((0.3, 0.2, 0.12)), 0.15, rough=0.5)
    shell(body, "belt", lambda i, co: abs(co.z - belt_z) < 0.018 and dom[i] in {"hips", "spine"}, 0.026, 0.006, belt)
    shoes = cloth("shoes", "leather", lin((0.3, 0.21, 0.14)), 0.2, rough=0.6)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, shoes)
    hood(body, j, ez, dom, cloth("hood", "wool", lin((0.26, 0.32, 0.46)), 0.3, dirt=(0.3, 0.4)), tail=0.22)
    # Sekken over venstre skulder: en pølse av strie bak på ryggen.
    sack = cloth("strap_chest_sack", "linen", lin((0.62, 0.55, 0.42)), 0.12, rough=0.85, bump=0.7, dirt=(0.6, 0.5))
    back = Vector((j["spine01"][0].x - 0.17, 0.05, j["spine01"][0].z - 0.05))
    block("strap_chest_sack", back, (0.12, 0.26, 0.3), sack, rot=(0.3, 0.1, 0), bevel=0.05)
    tube("strap_chest_sackstrap", back + Vector((0.05, 0.08, 0.13)), j["neck01"][0] + Vector((0.03, 0.08, -0.04)), 0.012, 0.012, sack, n=8)
    return j


def guard():
    """Vaktmannen: vattert våpenkjortel, brynjehette under en jernhatt
    med brem, lærbelte med sverd i skjeden, støvler og et spyd."""
    spec = GUARD
    body, j, ez, dom, belt_z, neck_z = dress_common(spec, (0.86, 0.8, 0.76))
    wrist_z = j["wrist.R"][0].z + 0.03
    gambeson = cloth("gambeson", "linen", lin((0.62, 0.5, 0.34)), 0.12, rough=0.85, bump=1.0, dirt=(0.8, 0.5))
    shell(body, "gambeson", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R", "forearm.L", "forearm.R"} and wrist_z < co.z < neck_z, 0.022, 0.006, gambeson)
    skirt(body, "skirt", belt_z - 0.02, j["lowerleg01.L"][0].z + 0.12 if "lowerleg01.L" in j else 0.55, gambeson, flare=0.07, legs=True)
    wool = cloth("hose", "wool", lin((0.25, 0.22, 0.2)), 0.3, dirt=(0.5, 0.6))
    shell(body, "hose", lambda i, co: dom[i] in {"thigh.L", "thigh.R", "shin.L", "shin.R"} and 0.25 < co.z < belt_z - 0.05, 0.004, 0.002, wool)
    boots = cloth("boots", "leather", lin((0.3, 0.21, 0.14)), 0.2, rough=0.55, dirt=(0.25, 0.5))
    shell(body, "boots", lambda i, co: 0.07 < co.z < 0.36, 0.012, 0.004, boots)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, boots)
    # Brynjehetta: jernringer over hodet, halsen og skuldrene, åpen for ansiktet.
    mail = cloth("mail", "metal", lin((0.66, 0.66, 0.68)), 0.05, rough=0.45, bump=1.2, dirt=(0.6, 0.35))
    hooded(body, "mail_coif", j, ez, dom, neck_z - 0.05, 0.014, 0.005, mail)
    shell(body, "mail_cape", lambda i, co: dom[i] in {"neck", "chest", "spine", "upper_arm.L", "upper_arm.R"} and neck_z - 0.16 < co.z < neck_z + 0.02, 0.032, 0.005, mail)
    # Jernhatten (kettle hat): en kuppel og en brem
    # som heller nedover.
    iron = cloth("hat_iron", "metal", lin((0.55, 0.54, 0.52)), 0.2, rough=0.4, bump=0.2, dirt=(0.4, 0.35))
    crown = Vector((j["head"][0].x + 0.005, 0.0, ez + 0.03))
    dome("hat_dome", crown, 0.135, 0.135, iron)
    brim("hat_brim", crown + Vector((0, 0, 0.004)), 0.12, 0.2, 0.045, iron, thick=0.005)
    leather = cloth("belt", "leather", lin((0.28, 0.19, 0.12)), 0.15, rough=0.5)
    shell(body, "belt", lambda i, co: abs(co.z - belt_z) < 0.022 and dom[i] in {"hips", "spine"}, 0.045, 0.006, leather)
    # Sverdet i skjeden på venstre hofte.
    block("pouch_scabbard", Vector((0.0, 0.16, belt_z - 0.4)), (0.05, 0.03, 0.8), leather, rot=(0.0, -0.25, 0.0), bevel=0.01)
    block("pouch_hilt", Vector((0.08, 0.16, belt_z + 0.02)), (0.03, 0.12, 0.03), iron, bevel=0.005)
    # Spydet: askeskaft og jernspiss.
    ash = cloth("tool_spear", "oak", lin((0.72, 0.62, 0.48)), 0.15, rough=0.6)
    g = hand_grip(j)
    base = g + Vector((-0.02, 0.0, -0.8))
    tip = g + Vector((0.03, 0.0, 1.15))
    tube("tool_spear", base, tip, 0.016, 0.015, ash)
    tube("tool_spearhead", tip, tip + Vector((0.005, 0, 0.24)), 0.03, 0.002, iron, n=4)
    return j


def merchant():
    """Kremmeren: lang kjortel i dyr rødbrun ull med pelskant, lærbelte
    med en tung pung, kaprun i mørkt grønt, og en vekt i hånda."""
    spec = MERCHANT
    body, j, ez, dom, belt_z, neck_z = dress_common(spec, (0.9, 0.84, 0.8))
    wrist_z = j["wrist.R"][0].z + 0.03
    wool = cloth("gown", "wool", lin((0.42, 0.18, 0.12)), 0.3, dirt=(0.5, 0.45))
    shell(body, "gown", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R", "forearm.L", "forearm.R"} and wrist_z < co.z < neck_z, 0.014, 0.004, wool)
    skirt(body, "skirt", belt_z - 0.02, 0.09, wool, flare=0.1, legs=True)
    fur = cloth("fur", "wool", lin((0.55, 0.47, 0.38)), 0.08, rough=0.95, bump=1.4)
    shell(body, "fur_cuff", lambda i, co: dom[i] in {"forearm.L", "forearm.R"} and wrist_z < co.z < wrist_z + 0.05, 0.022, 0.012, fur)
    shoes = cloth("shoes", "leather", lin((0.22, 0.15, 0.1)), 0.2, rough=0.5)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, shoes)
    leather = cloth("belt", "leather", lin((0.25, 0.16, 0.1)), 0.15, rough=0.45)
    shell(body, "belt", lambda i, co: abs(co.z - belt_z) < 0.02 and dom[i] in {"hips", "spine"}, 0.03, 0.006, leather)
    block("pouch_purse", Vector((0.11, -0.1, belt_z - 0.07)), (0.05, 0.1, 0.12), leather, rot=(0, 0.1, 0), bevel=0.02)
    brass = cloth("pouch_brass", "metal", lin((0.75, 0.6, 0.3)), 0.1, rough=0.35)
    block("pouch_clasp", Vector((0.14, -0.1, belt_z - 0.02)), (0.012, 0.08, 0.012), brass, bevel=0.003)
    hood(body, j, ez, dom, cloth("hood", "wool", lin((0.18, 0.28, 0.2)), 0.3, dirt=(0.25, 0.4)), tail=0.45)
    shell(body, "fur_collar", lambda i, co: dom[i] in {"neck", "chest", "spine"} and neck_z - 0.18 < co.z < neck_z - 0.13, 0.05, 0.015, fur)
    # Skålvekta: en stang med to skåler i kjeder, holdt i midten.
    g = hand_grip(j)
    tube("tool_beam", g + Vector((0.02, -0.12, -0.02)), g + Vector((0.02, 0.12, -0.02)), 0.006, 0.006, brass, n=8)
    for dy in (-0.11, 0.11):
        tube(f"tool_chain{dy}", g + Vector((0.02, dy, -0.02)), g + Vector((0.02, dy, -0.16)), 0.002, 0.002, brass, n=6)
        tube(f"tool_pan{dy}", g + Vector((0.02, dy, -0.17)), g + Vector((0.02, dy, -0.16)), 0.05, 0.05, brass, n=16)
    return j


def pilgrim():
    """Pilegrimen: lang grå reisekappe, hette under en vid filthatt med
    kamskjell, en veske av lær på hofta, og staven med kalebass."""
    spec = PILGRIM
    body, j, ez, dom, belt_z, neck_z = dress_common(spec, (0.9, 0.84, 0.8))
    wrist_z = j["wrist.R"][0].z + 0.03
    wool = cloth("robe", "wool", lin((0.4, 0.38, 0.35)), 0.3, dirt=(0.9, 0.75))
    shell(body, "robe", lambda i, co: dom[i] in {"hips", "spine", "chest", "upper_arm.L", "upper_arm.R", "forearm.L", "forearm.R"} and wrist_z < co.z < neck_z, 0.016, 0.004, wool)
    skirt(body, "skirt", belt_z - 0.02, 0.05, wool, flare=0.14, legs=True)
    shoes = cloth("shoes", "leather", lin((0.3, 0.22, 0.15)), 0.2, rough=0.65, dirt=(0.3, 0.7))
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, shoes)
    rope = cloth("cord", "linen", lin((0.65, 0.58, 0.45)), 0.06, rough=0.8, bump=0.8)
    shell(body, "cord", lambda i, co: abs(co.z - belt_z) < 0.009 and dom[i] in {"hips", "spine"}, 0.03, 0.008, rope)
    hood(body, j, ez, dom, cloth("hood", "wool", lin((0.3, 0.27, 0.24)), 0.3, dirt=(0.5, 0.5)), tail=0)
    # Filthatten med bred brem, brettet opp foran, og et kamskjell.
    felt = cloth("hat_felt", "wool", lin((0.3, 0.25, 0.2)), 0.1, rough=0.9, bump=0.5)
    crown = Vector((j["head"][0].x + 0.0, 0.0, ez + 0.045))
    dome("hat_crown", crown, 0.125, 0.11, felt)
    brim("hat_brim", crown + Vector((0, 0, 0.004)), 0.11, 0.24, 0.035, felt, thick=0.007)
    shell_mat = cloth("hat_shell", "linen", lin((0.92, 0.88, 0.8)), 0.05, rough=0.5)
    sh = dome("hat_shell", Vector((0, 0, 0)), 0.03, 0.012, shell_mat, n=12, rows=4)
    sh.rotation_euler = (0, math.radians(80), 0)
    sh.location = crown + Vector((0.112, 0.0, 0.045))
    leather = cloth("scrip", "leather", lin((0.34, 0.24, 0.15)), 0.15, rough=0.55)
    block("pouch_scrip", Vector((0.03, 0.17, belt_z - 0.12)), (0.06, 0.16, 0.18), leather, rot=(0, 0.05, 0), bevel=0.02)
    # Staven, høyere enn ham, med en kalebass.
    ash = cloth("tool_staff", "oak", lin((0.6, 0.48, 0.34)), 0.15, rough=0.65)
    g = hand_grip(j)
    base = g + Vector((0.0, 0.0, -0.78))
    top = g + Vector((0.05, 0.0, 1.05))
    tube("tool_staff", base, top, 0.018, 0.016, ash)
    gourd = cloth("tool_gourd", "earth", lin((0.75, 0.6, 0.35)), 0.1, rough=0.6)
    # Kalebassen: to kuler oppå hverandre, hengt i en snor fra staven.
    g0 = top + Vector((0.05, 0.0, -0.32))
    ball("tool_gourd0", g0, (0.055, 0.055, 0.05), gourd)
    ball("tool_gourd1", g0 + Vector((0, 0, 0.075)), (0.036, 0.036, 0.04), gourd)
    tube("tool_gourd_cord", top + Vector((0.0, 0.0, -0.2)), g0 + Vector((0, 0, 0.11)), 0.003, 0.003, rope, n=6)
    return j


def strap_over(body, name, a, b, r, mat, n=14):
    """En reim forfra over kroppen fra `a` til `b`: punktene langs linja
    legges på huden (en stråle bakover mot kroppen) og løftes litt ut, så
    reima ligger over brystet i stedet for å gå gjennom det."""
    tree = BVHTree.FromObject(body, bpy.context.evaluated_depsgraph_get())
    pts = []
    for k in range(n + 1):
        p = a.lerp(b, k / n)
        hit = tree.ray_cast(p + Vector((0.4, 0.0, 0.0)), Vector((-1.0, 0.0, 0.0)), 0.8)
        pts.append(Vector((max(p.x, hit[0].x + r * 0.5 + 0.003), p.y, p.z)) if hit[0] is not None else p)
    # Ei flat reim, dobbelt så bred som tykk, med kantene avfaset av
    # solidify.
    bm = bmesh.new()
    prev = None
    for k, p in enumerate(pts):
        d = (pts[min(k + 1, n)] - pts[max(k - 1, 0)]).normalized()
        side = d.cross(Vector((1.0, 0.0, 0.0))).normalized() * r * 1.4
        pair = (bm.verts.new(p - side), bm.verts.new(p + side))
        if prev:
            bm.faces.new((prev[0], prev[1], pair[1], pair[0]))
        prev = pair
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=r * 0.7)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    me.materials.append(mat)
    copy_weights(body, ob)
    return ob


def hair_locks(body, j, name, mat, z_top, z_bot, ez, n=230, arc=100.0, rows=12, seed=3):
    """Langt hår som enkeltlokker: hver lokk starter på hodet (fra issen
    bak, over øret på sidene), følger hodet ned og henger så rett ned over
    nakken og skuldrene, smalner mot tuppen og ligger i lag utenpå
    hverandre."""
    rng = random.Random(seed)
    tree = BVHTree.FromObject(body, bpy.context.evaluated_depsgraph_get())
    cx = j["head"][0].x - 0.015
    bm = bmesh.new()
    for _ in range(n):
        a = rng.uniform(-arc, arc)
        side = abs(a) > 70
        z0 = (ez + rng.uniform(0.03, 0.07)) if side else (z_top - rng.uniform(0.03, 0.08))
        z1 = z_bot + rng.uniform(-0.07, 0.05)
        if side:
            # Lokkene foran øret slutter ved kjeven: lenger ned falt de over
            # pelsen og hang som svarte flak foran brystet.
            z1 = ez - rng.uniform(0.08, 0.14)
        lift = rng.uniform(0.002, 0.01)
        width = rng.uniform(0.015, 0.026)
        reach = 0.0
        prev = None
        for r in range(rows + 1):
            t = r / rows
            a += rng.uniform(-1.2, 1.2)
            ar = math.radians(a)
            d = Vector((-math.cos(ar), math.sin(ar), 0.0))
            z = z0 + (z1 - z0) * t
            c = Vector((cx, 0.0, z))
            hit = tree.ray_cast(c + d * 0.6, -d, 0.6)
            rad = (hit[0] - c).length if hit[0] is not None else reach
            reach = max(rad + lift + 0.012 * t, reach - 0.003) if r else rad + lift
            mid = c + d * reach
            # Spiss i begge ender: glir ut av håret på hodet øverst.
            tan = Vector((math.sin(ar), math.cos(ar), 0.0)) * (width * min(1.0, 0.15 + t / 0.2) * (1.0 - 0.75 * t) / 2)
            pair = (bm.verts.new(mid - tan), bm.verts.new(mid + tan))
            if prev:
                bm.faces.new((prev[0], prev[1], pair[1], pair[0]))
            prev = pair
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.0015)
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


def beard_tufts(body, dom, name, mat, ez, ear_x, seed=5):
    """Skjegg med volum: korte, spisse tuster fra hjørnene i skjeggfeltet,
    lagt ut fra huden og ned (lengre på haka), og en bart som henger over
    overleppa. Leppene holdes fri."""
    rng = random.Random(seed)
    mz = ez - 0.068
    bm = bmesh.new()
    me = body.data
    for v in me.vertices:
        co, n = v.co, v.normal
        if dom[v.index] not in ("head", "neck") or co.x < ear_x + 0.03 or n.x < -0.2:
            continue
        lip = abs(co.y) < 0.032 and abs(co.z - mz) < 0.013
        moustache = abs(co.y) < 0.04 and mz + 0.013 < co.z < mz + 0.03 and n.x > 0.3
        jaw = ez - 0.17 < co.z < mz - 0.013 or (mz - 0.013 <= co.z < ez - 0.035 and abs(co.y) > 0.045)
        if lip or not (moustache or jaw):
            continue
        for _ in range(2):
            p = co + n * 0.001 + Vector((rng.uniform(-0.003, 0.003), rng.uniform(-0.003, 0.003), rng.uniform(-0.003, 0.003)))
            down = Vector((0, 0, -1)) - n * (-n.z)
            chin = max(0.0, (mz - co.z) / 0.08)
            d = (n * rng.uniform(0.12, 0.4) + down.normalized() * 0.9 + Vector((0, rng.uniform(-0.35, 0.35), 0))).normalized()
            length = (0.011 if moustache else 0.014 + 0.024 * chin) * rng.uniform(0.6, 1.4)
            side = n.cross(d).normalized() * (rng.uniform(0.004, 0.007) / 2)
            prev = None
            for k in range(3):
                t = k / 2
                q = p + d * length * t + n * 0.003 * math.sin(t * math.pi)
                w = 1.0 - 0.85 * t
                pair = (bm.verts.new(q - side * w), bm.verts.new(q + side * w))
                if prev:
                    bm.faces.new((prev[0], prev[1], pair[1], pair[0]))
                prev = pair
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.0012)
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    ob = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(ob)
    mesh.materials.append(mat)
    for p_ in mesh.polygons:
        p_.use_smooth = True
    copy_weights(body, ob)
    print(f"skjegg: {len(mesh.polygons)} flater")
    return ob


def scarred_back(spec):
    """Huden til Gorvek: hudflisen, og bakfotoet (med arrene på skuldrene og
    ryggen) lagt på øvre del av ryggen, festet i skuldrene."""
    def make(body, j):
        sl, sr = j["upperarm01.L"][0], j["upperarm01.R"][0]
        sz = (sl.z + sr.z) / 2
        # Skuldrene i fotoet står ved u 0,1 og 0,9 i høyde v 0,72.
        sb = (sl.y - sr.y) / 0.8
        project_uv(body, "backbody", lambda p: (0.5 - p.y / sb, 0.72 + (sz - p.z) / sb))
        m = bpy.data.materials.new("skin")
        m.use_nodes = True
        nt = m.node_tree
        b = _bsdf(m)
        tc = nt.nodes.new("ShaderNodeTexCoord")
        st = _boxtex(nt, spec["tiles"]["skin"], 0.22, tc.outputs["Object"])
        col = _mix(nt, st.outputs["Color"], (0.88, 0.8, 0.74), 1.0, "MULTIPLY")
        uv = nt.nodes.new("ShaderNodeUVMap")
        uv.uv_map = "backbody"
        img = nt.nodes.new("ShaderNodeTexImage")
        img.image = bpy.data.images.load(spec["photo"]["back"]["file"], check_existing=True)
        img.extension = "EXTEND"
        nt.links.new(uv.outputs["UV"], img.inputs["Vector"])
        sep = nt.nodes.new("ShaderNodeSeparateXYZ")
        nt.links.new(uv.outputs["UV"], sep.inputs[0])
        geo = nt.nodes.new("ShaderNodeNewGeometry")
        nrm = nt.nodes.new("ShaderNodeSeparateXYZ")
        nt.links.new(geo.outputs["Normal"], nrm.inputs[0])
        # Bare ryggen (flater som vender bakover), og ikke utenfor fotoet
        # (v under 0,98 er nederst i bildet).
        w = _math(nt, "MULTIPLY", _ramp(nt, nrm.outputs["X"], -0.2, -0.55), _ramp(nt, sep.outputs["Y"], 0.02, 0.08))
        w = _math(nt, "MULTIPLY", w, _ramp(nt, sep.outputs["X"], 0.02, 0.08))
        w = _math(nt, "MULTIPLY", w, _ramp(nt, sep.outputs["X"], 0.98, 0.92))
        gain = tuple(float(min(1.4, max(0.7, a / max(1e-4, c)))) for a, c in zip(patch_mean(spec["tiles"]["skin"], (0.2, 0.2, 0.8, 0.8)), patch_mean(img.image.filepath, spec["photo"]["back"]["skin"])))
        photo = _mix(nt, img.outputs["Color"], gain, 1.0, "MULTIPLY")
        col = _mix(nt, col, photo, w)
        nt.links.new(col, b.inputs["Base Color"])
        b.inputs["Roughness"].default_value = 0.5
        return m
    return make


def gorvek():
    spec = GORVEK
    body, j, ez, dom, ear_x = dress_photos(spec, None, body_mat=scarred_back(spec))
    belt_z = j["spine04"][0].z + 0.02
    neck_z = j["neck01"][0].z - 0.01
    t = spec["tiles"]
    # Det lange håret: fra over ørene ned over nakken til skulderbladene.
    hair = skin_material("hair", t["hair"], (0.82, 0.74, 0.66), scale=0.18)
    crown = max(v.co.z for v in body.data.vertices)
    hair_locks(body, j, "hair", hair, crown - 0.01, neck_z - 0.27, ez)
    # Skjegget med volum: korte tuster som vokser ut fra kjeven, haka og
    # under haka og peker nedover, og en bart over leppa. Leppene er fri.
    beard = skin_material("beard", t["beard"], (0.78, 0.7, 0.62), scale=0.04)
    beard_tufts(body, dom, "beard", beard, ez, ear_x)
    # Pelskappa over skuldrene (grå ulvepels), med en lærreim over brystet.
    fur = skin_material("fur", t["beard"], (0.62, 0.52, 0.42), scale=0.07)
    sh_z = (j["upperarm01.L"][0].z + j["upperarm01.R"][0].z) / 2
    el_z = j["lowerarm01.L"][0].z
    shell(body, "fur", lambda i, co: (dom[i] in {"chest", "spine", "neck"} and sh_z - 0.12 < co.z < neck_z + 0.03 and co.x < 0.06) or (dom[i] in {"upper_arm.L", "upper_arm.R"} and co.z > el_z + 0.17), 0.04, 0.03, fur)
    leather = cloth("leather", "leather", lin((0.26, 0.18, 0.12)), 0.18, rough=0.55, dirt=(0.7, 0.5))
    strap_over(body, "strap_chest", Vector((0.16, 0.13, sh_z - 0.02)), Vector((0.17, -0.12, belt_z + 0.1)), 0.016, leather)
    # Lærbuksa og pelsviklede støvler.
    shell(body, "trousers", lambda i, co: dom[i] in {"hips", "thigh.L", "thigh.R", "shin.L", "shin.R"} and 0.28 < co.z < belt_z + 0.04, 0.006, 0.003, leather)
    shell(body, "wraps", lambda i, co: 0.07 < co.z < 0.4 and dom[i] in {"shin.L", "shin.R", "foot.L", "foot.R"}, 0.018, 0.006, fur)
    for sy in (-1, 1):
        shoe(body, f"shoe{sy}", sy, leather)
    # Det brede beltet med jernspenne, og armskinner av lær.
    iron = cloth("iron", "metal", lin((0.5, 0.48, 0.46)), 0.15, rough=0.45, bump=0.3)
    shell(body, "belt", lambda i, co: abs(co.z - belt_z) < 0.035 and dom[i] in {"hips", "spine"}, 0.035, 0.008, leather)
    block("buckle", Vector((0.19, 0.0, belt_z)), (0.03, 0.09, 0.075), iron, bevel=0.006)
    for s_ in "LR":
        w_z = j[f"wrist.{s_}"][0].z
        shell(body, f"bracer{s_}", lambda i, co, s_=s_, w_z=w_z: dom[i] == f"forearm.{s_}" and co.z > w_z + 0.02 and co.z < w_z + 0.17, 0.008, 0.005, leather)
    # Skjeggøksa i høyre hånd: askeskaft gjennom grepet, hodet nede ved kneet.
    ash = cloth("tool_ash", "oak", lin((0.7, 0.58, 0.44)), 0.15, rough=0.6)
    g = hand_grip(j)
    top, bot = g + Vector((-0.03, 0, 0.14)), g + Vector((0.06, 0, -0.62))
    tube("tool_haft", top, bot, 0.017, 0.016, ash)
    steel = cloth("tool_steel", "metal", lin((0.62, 0.6, 0.58)), 0.12, rough=0.35, bump=0.2)
    d = (bot - top).normalized()
    head = bot - d * 0.07
    block("tool_eye", head, (0.05, 0.035, 0.08), steel, rot=(0, -math.atan2(d.x, -d.z), 0), bevel=0.006)
    block("tool_blade", head + Vector((0.11, 0, -0.03)), (0.17, 0.012, 0.11), steel, rot=(0, -math.atan2(d.x, -d.z) + 0.12, 0), bevel=0.004)
    block("tool_beard", head + Vector((0.16, 0, -0.1)), (0.07, 0.01, 0.12), steel, rot=(0, -math.atan2(d.x, -d.z) - 0.3, 0), bevel=0.004)
    return j


MODELS = {"smith": smith, "innkeeper": innkeeper, "apothecary": apothecary, "clerk": clerk, "gorvek": gorvek, "woman": woman, "farmer": farmer, "boy": boy, "guard": guard, "merchant": merchant, "pilgrim": pilgrim}


# Hvor tett teksturen er per del (1 er som flatene tilsier).
# Stilen i folk_motion.py og hva høyre hånd holder («hang», «heavy», «long», «front»).
MOTION = {"smith": ("smith", "hang"), "innkeeper": ("innkeeper", "hang"), "gorvek": ("gorvek", "hang"), "woman": ("innkeeper", "heavy"),
          "apothecary": ("old", "front"), "clerk": ("default", "front"), "farmer": ("default", "long"), "boy": ("boy", None),
          "guard": ("guard", "long"), "merchant": ("old", "hang"), "pilgrim": ("old", "long")}
DENSITY = {"hair": 0.0, "beard": 0.0, "fur": 0.5, "wraps": 0.5, "hood": 0.7, "mail": 0.6, "tool": 0.5, "strap": 0.5, "pouch": 0.5, "shoe": 0.5, "boots": 0.6, "skirt": 0.6, "hose": 0.6, "trousers": 0.6}


def face_density(body, j, k=3.0):
    """Flateattributtet «uv_density» på kroppen: hodet og halsen k ganger
    tettere enn resten."""
    if body is None:
        return
    dom = dominant(body)
    a = body.data.attributes.new("uv_density", "FLOAT", "FACE")
    a.data.foreach_set("value", [k if all(dom[i] in ("head", "neck") for i in p.vertices) else 1.0 for p in body.data.polygons])


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
    hide_covered(bpy.data.objects["body"])
    for ob in bpy.data.objects:
        if ob.type == "MESH" and "UVMap" not in ob.data.uv_layers:
            ob.data.uv_layers.new(name="UVMap")
    if "--preview" in args:
        preview(args[args.index("--preview") + 1], j["head"][0].z + 0.08)
        sys.exit(0)
    # Ansiktet får mest plass i teksturen; hårlokker, skjegg og pels minst.
    face_density(bpy.data.objects.get("body"), j)
    rig = armature(f"{name}_rig", game_bones(j))
    # Bevegelsene (folk_motion.py): stilen og hva høyre hånd holder.
    style, kind = MOTION[name]
    tools = [o for o in bpy.data.objects if o.type == "MESH" and (o.name.startswith("tool") or o.name.startswith("mug"))]
    folk_motion.clips(rig, style, {"R": kind} if kind else {}, body=bpy.data.objects.get("body"), tools={"R": tools} if kind else None)
    bake_textured(f"{OUT}/npc_{name}.glb", size, keep=("glass", "glow", "fire", "mesh"), armature=rig, rigid={"tool": "hand.R", "mug": "hand.R", "apron": "spine", "skirt": "hips", "bodice_strap": "chest", "pouch": "hips", "cord_end": "hips", "buckle": "hips", "strap_chest": "chest", "hat": "head"}, skin={"blend": 0.05}, density=DENSITY, jpeg_quality=92)
