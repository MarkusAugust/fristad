"""Spillerens armer i førsteperson, sverdet og skjoldet, i den detaljgraden
eieren vil ha: armene er MakeHumans ekte armer og hender (CC0, som
innbyggerne i people.py), posert i kamerarommet med knyttnevene rundt
sverdgrepet og skjoldhåndtaket. Over dem en vattert gambeson med
quiltede kanaler, armskinner av lær med spenner, og lærhansker med
mansjett. Sverdet er et smidd riddersverd (Oakeshott XII): linseformet
klinge med hulkil og slipte egger, smidd parerstang, lærtvunnet grep og
hjulknapp med naglet angel. Skjoldet er et rundskjold av planker med
malt lærtrekk, råhudkant med sting, kuppelbule med nagler og håndtak bak.
Kjør:

    python3 tools/models/fp_arms.py [arms sword shield_round axe dagger shield_tower] [--preview ut.png] [størrelse]

Modellen står i kamerarommet som før (arms.py): kameraet i origo ser langs
+Y, Z opp, X høyre. Beina heter det samme (root, upper_arm, forearm,
hand, sword, shield), og klippene er de samme (`arms.clips`), så spillet
trenger ingen endringer. Våpnene bygges i hvileposen der hendene griper
(`assets/models/arms/rig.json`, skrevet når armene bygges)."""

import json
import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from arms import clips  # noqa: E402
from common import armature, bake_textured, clear  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from people import TEX, _boxtex, _bsdf, _math, _mix, _ramp, build_human, lin, shell  # noqa: E402

OUT = "assets/models/arms"
RIG = f"{OUT}/rig.json"

# Retningene for armene i kamerarommet (som armene i arms.py).
CAM_DIRS = {
    "R": ((0.08, 0.30, -0.13), (-0.08, 0.30, 0.17)),
    "L": ((-0.18, 0.26, -0.2), (0.03, 0.30, 0.06)),
}


def cam_to_mh(v):
    """Kamerarommet (x høyre, y fram, z opp) til MakeHumans (x venstre, −y fram)."""
    return (-v[0], -v[1], v[2])


PLAYER = {
    "name": "player",
    "macro": (1.0, 0.3, 0.9, 0.2),
    "height": 1.84,
    "grip": "LR",
    "curl": 74,
    "arm_dirs": {s: (cam_to_mh(a), cam_to_mh(b)) for s, (a, b) in CAM_DIRS.items()},
    # Høyre knyttneve med tommelen opp (klinga peker opp og fram).
    "roll": {"R": 45.0},
}


# --- Materialer ---------------------------------------------------------------


def new_mat(name):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    return m, m.node_tree, _bsdf(m)


def local_coords(nt):
    """Modellens eget rom (attributtet «lc», lagret før delene flyttes på
    plass), så slipestriper og maling følger delen."""
    a = nt.nodes.new("ShaderNodeAttribute")
    a.attribute_name = "lc"
    return a.outputs["Vector"]


def local_coords_turned(nt):
    """Som `local_coords`, dreid 90° om z (årene langs plankene)."""
    mp = nt.nodes.new("ShaderNodeMapping")
    mp.inputs["Rotation"].default_value = (0.0, 0.0, math.pi / 2)
    nt.links.new(local_coords(nt), mp.inputs["Vector"])
    return mp.outputs["Vector"]


def noise(nt, vec, scale, detail=4.0, stretch=(1, 1, 1)):
    mp = nt.nodes.new("ShaderNodeMapping")
    mp.inputs["Scale"].default_value = stretch
    nt.links.new(vec, mp.inputs["Vector"])
    n = nt.nodes.new("ShaderNodeTexNoise")
    n.inputs["Scale"].default_value = scale
    n.inputs["Detail"].default_value = detail
    nt.links.new(mp.outputs["Vector"], n.inputs["Vector"])
    return n.outputs["Fac"]


def bump(nt, b, height, strength, distance=0.002):
    bp = nt.nodes.new("ShaderNodeBump")
    bp.inputs["Strength"].default_value = strength
    bp.inputs["Distance"].default_value = distance
    nt.links.new(height, bp.inputs["Height"])
    nt.links.new(bp.outputs["Normal"], b.inputs["Normal"])


def steel(name, polish=0.2, tone=(0.62, 0.63, 0.66), streak=40.0):
    """Slipt stål: slipestriper langs klingen (lokal z), fine riper, litt
    anløpt og flekket mot parerstanga."""
    m, nt, b = new_mat(name)
    lc = local_coords(nt)
    streaks = noise(nt, lc, 260.0, 2.0, (1.0, 1.0, 1.0 / streak))
    scratches = noise(nt, lc, 90.0, 8.0)
    stains = noise(nt, lc, 9.0, 3.0)
    col = _mix(nt, tone, _math(nt, "MULTIPLY", 1.0, 1.0) if False else (0.42, 0.4, 0.38), _ramp(nt, stains, 0.62, 0.8))
    col = _mix(nt, col, (0.82, 0.83, 0.86), _math(nt, "MULTIPLY", _ramp(nt, streaks, 0.55, 0.75), 0.35))
    nt.links.new(col, b.inputs["Base Color"])
    rough = _math(nt, "ADD", polish, _math(nt, "MULTIPLY", streaks, 0.18))
    rough = _math(nt, "ADD", rough, _math(nt, "MULTIPLY", _ramp(nt, stains, 0.62, 0.8), 0.25))
    nt.links.new(rough, b.inputs["Roughness"])
    b.inputs["Metallic"].default_value = 1.0
    bump(nt, b, _math(nt, "ADD", _math(nt, "MULTIPLY", streaks, 0.3), _ramp(nt, scratches, 0.7, 0.72)), 0.12, 0.0004)
    return m


def forged(name, tone=(0.28, 0.27, 0.27)):
    """Smijern: mørkt, hamret, med slitte, blankere kanter i noisen."""
    m, nt, b = new_mat(name)
    lc = local_coords(nt)
    ham = noise(nt, lc, 70.0, 6.0)
    pit = noise(nt, lc, 220.0, 3.0)
    col = _mix(nt, tone, (0.48, 0.47, 0.46), _ramp(nt, ham, 0.55, 0.7))
    col = _mix(nt, col, (0.2, 0.12, 0.08), _math(nt, "MULTIPLY", _ramp(nt, pit, 0.66, 0.72), 0.6))
    nt.links.new(col, b.inputs["Base Color"])
    nt.links.new(_math(nt, "ADD", 0.38, _math(nt, "MULTIPLY", ham, 0.25)), b.inputs["Roughness"])
    b.inputs["Metallic"].default_value = 1.0
    bump(nt, b, _math(nt, "ADD", ham, _math(nt, "MULTIPLY", pit, 0.5)), 0.35, 0.0015)
    return m


def textured(name, layer, tint, scale, rough=0.7, bump_s=0.5, coords=None):
    """Et teksturlag fra eieren (albedo og høyde), boksprojisert, tonet."""
    m, nt, b = new_mat(name)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    vec = coords(nt) if coords else tc.outputs["Object"]
    img = _boxtex(nt, f"{TEX}/{layer}/albedo.jpg", scale, vec)
    nt.links.new(_mix(nt, img.outputs["Color"], tint, 1.0, "MULTIPLY"), b.inputs["Base Color"])
    b.inputs["Roughness"].default_value = rough
    h = _boxtex(nt, f"{TEX}/{layer}/height.jpg", scale, vec)
    h.image.colorspace_settings.name = "Non-Color"
    bump(nt, b, h.outputs["Color"], bump_s, 0.003)
    return m, nt, b


def gambeson():
    """Vattert lin: ullent, med quiltede kanaler på tvers av armen (bølger
    langs kameraets y) og sting i fugene."""
    m, nt, b = textured("gambeson", "linen", lin((0.55, 0.43, 0.3)), 0.12, rough=0.85, bump_s=0.3)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    w = nt.nodes.new("ShaderNodeTexWave")
    w.wave_type = "BANDS"
    w.bands_direction = "Y"
    w.inputs["Scale"].default_value = 26.0
    w.inputs["Distortion"].default_value = 1.2
    w.inputs["Detail"].default_value = 1.0
    nt.links.new(tc.outputs["Object"], w.inputs["Vector"])
    ridge = _math(nt, "POWER", w.outputs["Fac"], 0.35)
    bump(nt, b, ridge, 0.9, 0.006)
    # Mørkere i fugene.
    base = b.inputs["Base Color"].links[0].from_socket
    nt.links.new(_mix(nt, base, (0.5, 0.45, 0.4), _ramp(nt, w.outputs["Fac"], 0.2, 0.0), "MULTIPLY"), b.inputs["Base Color"])
    return m


def leather(name, tint, rough=0.55, scale=0.08, bump_s=0.25):
    m, nt, b = textured(name, "leather", lin(tint), scale, rough=rough, bump_s=bump_s)
    return m


# --- Mesh-hjelpere --------------------------------------------------------------


def link_mesh(name, bm, mat, smooth=True):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    if isinstance(mat, (list, tuple)):
        for mm in mat:
            me.materials.append(mm)
    else:
        me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = smooth
    return ob


def place(ob, frame):
    """Lagre de lokale koordinatene («lc») og flytt delen på plass med
    rammen (4 × 4)."""
    me = ob.data
    attr = me.attributes.new("lc", "FLOAT_VECTOR", "POINT")
    for i, v in enumerate(me.vertices):
        attr.data[i].vector = v.co
    me.transform(frame)
    return ob


def loft(name, sections, mat, cap=(True, True), mats_by_column=None):
    """Et legeme fra tverrsnitt (lister av punkter, like mange i hver),
    lukket rundt og med lokk i endene. `mats_by_column(k)` gir
    materialindeksen for flatene mellom punkt k og k+1."""
    bm = bmesh.new()
    rings = [[bm.verts.new(p) for p in sec] for sec in sections]
    n = len(sections[0])
    for a, b2 in zip(rings, rings[1:]):
        for k in range(n):
            j = (k + 1) % n
            f = bm.faces.new((a[k], a[j], b2[j], b2[k]))
            if mats_by_column:
                f.material_index = mats_by_column(k)
    if cap[0]:
        bm.faces.new(rings[0][::-1])
    if cap[1]:
        bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return link_mesh(name, bm, mat)


def lathe(name, profile, mat, n=32, axis_frame=None):
    """Et omdreiningslegeme om lokal z fra profilen [(r, z)]."""
    secs = []
    for r, z in profile:
        secs.append([Vector((math.cos(2 * math.pi * k / n) * max(r, 1e-4), math.sin(2 * math.pi * k / n) * max(r, 1e-4), z)) for k in range(n)])
    bm = bmesh.new()
    rings = [[bm.verts.new(p) for p in s] for s in secs]
    for a, b2 in zip(rings, rings[1:]):
        for k in range(n):
            j = (k + 1) % n
            bm.faces.new((a[k], a[j], b2[j], b2[k]))
    bm.faces.new(rings[0][::-1])
    bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return link_mesh(name, bm, mat)


def frame_of(origin, z, x):
    """En ramme med lokal z langs `z` og lokal x så nær `x` som mulig."""
    z = Vector(z).normalized()
    y = z.cross(Vector(x)).normalized()
    x = y.cross(z).normalized()
    m = Matrix((x, y, z)).transposed().to_4x4()
    m.translation = Vector(origin)
    return m


# --- Armene ---------------------------------------------------------------------


def build_arms():
    body, j, eye, _ = build_human(PLAYER)
    em = (eye["l"] + eye["r"]) / 2
    # Spillets akser (front +x, venstre +y) til kamerarommet (fram +y, høyre +x).
    R = Matrix(((0, -1, 0), (1, 0, 0), (0, 0, 1))).to_4x4()
    T = R @ Matrix.Translation(-em)
    body.data.transform(T)
    J = {n: (T @ a, T @ b) for n, (a, b) in j.items()}
    names = {vg.index: vg.name for vg in body.vertex_groups}
    dom = []
    for v in body.data.vertices:
        best = max(v.groups, key=lambda g: g.weight, default=None)
        dom.append(names[best.group] if best else "")
    arm_bones = {f"{b}.{s}" for b in ("upper_arm", "forearm", "hand") for s in "LR"}
    # Hvor langt ut på underarmen et punkt ligger (0 albuen, 1 håndleddet).
    def along(co, s):
        e, w = J[f"lowerarm01.{s}"][0], J[f"wrist.{s}"][0]
        d = w - e
        return (co - e).dot(d) / d.length_squared

    def side(co):
        return "R" if co.x > 0 else "L"

    shoulder_cut = {s: J[f"upperarm01.{s}"][0] for s in "LR"}

    def on_arm(i, co):
        if dom[i] not in arm_bones:
            return False
        s = side(co)
        # Overarmen kuttes litt nedenfor skulderleddet (bak kameraet).
        u = J[f"lowerarm01.{s}"][0] - shoulder_cut[s]
        return (co - shoulder_cut[s]).dot(u) / u.length_squared > 0.12

    # Klærne, utenfra og inn: gambeson til håndleddet, armskinner av lær,
    # hansker med mansjett.
    gam = gambeson()
    brown = leather("bracer", (0.33, 0.2, 0.11), 0.5)
    glove_m = leather("glove", (0.24, 0.16, 0.1), 0.5, 0.15, 0.12)
    shell(body, "sleeve", lambda i, co: on_arm(i, co) and dom[i] != f"hand.{side(co)}" and along(co, side(co)) < 0.86, 0.013, 0.006, gam)
    shell(body, "bracer", lambda i, co: dom[i] == f"forearm.{side(co)}" and 0.3 < along(co, side(co)) < 0.8, 0.026, 0.004, brown)
    shell(body, "glove", lambda i, co: on_arm(i, co) and along(co, side(co)) > 0.8, 0.0028, 0.0015, glove_m)
    shell(body, "glove_cuff", lambda i, co: dom[i] in arm_bones and 0.74 < along(co, side(co)) < 0.95, 0.03, 0.003, glove_m,
          shape=None)
    # Stroppene rundt armskinnene (tett på lærets flate), med spenner av
    # jern på oversida.
    iron = forged("buckle_iron")
    strap_m = leather("strap", (0.16, 0.1, 0.06), 0.5, 0.04)
    body.data.update()
    for s in "LR":
        e, w = J[f"lowerarm01.{s}"][0], J[f"wrist.{s}"][0]
        d = (w - e).normalized()
        for t in (0.42, 0.68):
            band = lambda i, co, s=s, t=t: dom[i] == f"forearm.{s}" and abs(along(co, s) - t) < 0.035  # noqa: E731
            shell(body, f"strap{s}{t}", band, 0.031, 0.003, strap_m)
            top = max((v for v in body.data.vertices if band(v.index, v.co)), key=lambda v: v.co.z)
            n = top.normal.normalized()
            buckle(f"buckle{s}{t}", top.co + n * 0.034, d, n, iron)
    # Bare armene blir igjen.
    bm = bmesh.new()
    bm.from_mesh(body.data)
    bm.verts.ensure_lookup_table()
    kill = [f for f in bm.faces if not all(on_arm(v.index, v.co) for v in f.verts)]
    bmesh.ops.delete(bm, geom=kill, context="FACES")
    loose = [v for v in bm.verts if not v.link_faces]
    bmesh.ops.delete(bm, geom=loose, context="VERTS")
    bm.to_mesh(body.data)
    bm.free()
    skin = textured("skin", "plaster", lin((0.85, 0.66, 0.55)), 0.05, rough=0.55, bump_s=0.1)[0]
    body.data.materials.clear()
    body.data.materials.append(skin)
    for ob in bpy.data.objects:
        if ob.type == "MESH":
            ob.vertex_groups.clear()
    # Skjelettet og grepene.
    bones = [("root", (0, 0, 0), (0, 0.1, 0), None)]
    grips = {}
    for s in "RL":
        sh, el, wr, k3 = J[f"upperarm01.{s}"][0], J[f"lowerarm01.{s}"][0], J[f"wrist.{s}"][0], J[f"finger3-1.{s}"][0]
        k2, k5 = J[f"finger2-1.{s}"][0], J[f"finger5-1.{s}"][0]
        m2, m5 = J[f"finger2-2.{s}"][0], J[f"finger5-2.{s}"][0]
        # Grepet: midt mellom knokene og de bøyde fingrene; aksen fra
        # lillefingeren mot pekefingeren (ut på tommelsida).
        centre = (k2 + k5 + m2 + m5) / 4
        axis = (k2 - k5).normalized()
        knuckle = (((k2 + k5) / 2) - wr).normalized()
        grips[s] = {"centre": list(centre), "axis": list(axis), "knuckle": list(knuckle), "wrist": list(wr)}
        bones += [
            (f"upper_arm.{s}", tuple(sh), tuple(el), "root"),
            (f"forearm.{s}", tuple(el), tuple(wr), f"upper_arm.{s}"),
            (f"hand.{s}", tuple(wr), tuple(k3), f"forearm.{s}"),
        ]
        if s == "R":
            bones.append(("sword", tuple(centre), tuple(centre + axis * 0.9), "hand.R"))
        else:
            bones.append(("shield", tuple(centre), tuple(centre + knuckle * 0.1), "hand.L"))
    os.makedirs(OUT, exist_ok=True)
    with open(RIG, "w") as f:
        json.dump({"bones": bones, "grips": grips}, f, indent=1)
    return bones


def strap_ring(name, c, d, r, w, t, mat, n=28):
    """En lærstropp rundt armen: ring med bredde w og tykkelse t."""
    q = d.to_track_quat("Z", "Y")
    secs = []
    for k in range(n):
        a = 2 * math.pi * k / n
        rad = q @ Vector((math.cos(a), math.sin(a), 0))
        secs.append([c + rad * (r + dr) + d * dz for dr, dz in ((0, -w / 2), (t, -w / 2), (t, w / 2), (0, w / 2))])
    bm = bmesh.new()
    rings = [[bm.verts.new(p) for p in s] for s in secs]
    for k in range(n):
        a, b2 = rings[k], rings[(k + 1) % n]
        for i in range(4):
            j = (i + 1) % 4
            bm.faces.new((a[i], a[j], b2[j], b2[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return link_mesh(name, bm, mat)


def buckle(name, c, d, n, mat):
    """En rektangulær spenne med torn, liggende på stroppen."""
    side = d.cross(n).normalized()
    bm = bmesh.new()
    for off, size in (((0, 0.011), (0.028, 0.004)), ((0, -0.011), (0.028, 0.004)), ((0.012, 0), (0.004, 0.026)), ((-0.012, 0), (0.004, 0.026)), ((0, 0), (0.022, 0.0025))):
        cen = c + side * off[0] + d * off[1] + n * 0.003
        ret = bmesh.ops.create_cube(bm, size=1.0)
        vs = ret["verts"]
        for v in vs:
            p = v.co
            v.co = cen + side * p.x * size[0] + d * p.y * size[1] + n * p.z * 0.004
    return link_mesh(name, bm, mat, smooth=False)


# --- Våpnene ----------------------------------------------------------------------


def grip_frame(s):
    g = json.load(open(RIG))["grips"][s]
    return Vector(g["centre"]), Vector(g["axis"]), Vector(g["knuckle"]), Vector(g["wrist"])


def sword():
    """Riddersverd: klinge 78 cm, linseformet med hulkil, slipte egger,
    smidd parerstang, lærtvunnet grep med ryggstrenger, hjulknapp."""
    c, axis, knuckle, _ = grip_frame("R")
    # Klingas egger ligger i planet med knokene (hugget går fram).
    F = frame_of(c, axis, knuckle)
    blade_m = steel("steel_flat", 0.22)
    edge_m = steel("steel_edge", 0.1, (0.72, 0.73, 0.76), 80.0)
    guard_m = forged("guard_iron")
    grip_m = leather("grip_leather", (0.18, 0.11, 0.07), 0.6, 0.03)
    s0 = 0.075
    L = 0.78
    secs = []
    n_len = 48
    for i in range(n_len + 1):
        u = i / n_len
        z = s0 + u * L
        # Bredden smalner av, spissen de siste 14 %.
        w = 0.052 + (0.026 - 0.052) * u
        tip = max(0.0, (u - 0.86) / 0.14)
        w *= math.sqrt(max(0.0, 1 - tip * tip)) if tip > 0 else 1.0
        w = max(w, 0.0006)
        th = 0.0068 + (0.0028 - 0.0068) * u
        th = max(th * (1 - tip * 0.9), 0.0004)
        fl = max(0.0, 1 - u / 0.62)
        fw, fd = 0.0075 * fl, 0.0018 * fl
        bev = min(0.009, w * 0.35)
        top = [(w / 2, 0.0), (w / 2 - bev, th * 0.42), (fw * 1.6 + 0.0001, th / 2), (fw, th / 2 - fd * 0.6), (0.0, th / 2 - fd), (-fw, th / 2 - fd * 0.6), (-fw * 1.6 - 0.0001, th / 2), (-(w / 2 - bev), th * 0.42)]
        bot = [(-w / 2, 0.0), (-(w / 2 - bev), -th * 0.42), (-fw * 1.6 - 0.0001, -th / 2), (-fw, -th / 2 + fd * 0.6), (0.0, -th / 2 + fd), (fw, -th / 2 + fd * 0.6), (fw * 1.6 + 0.0001, -th / 2), (w / 2 - bev, -th * 0.42)]
        secs.append([Vector((x, y, z)) for x, y in top + bot])
    # Flatene mellom eggpunktene (0–1, 7–8, 8–9, 15–0) er de slipte eggene.
    blade = loft("blade", secs, [blade_m, edge_m], mats_by_column=lambda k: 1 if k in (0, 7, 8, 15) else 0)
    # Flatene på klinga er slipte plan, ikke runde.
    for p_ in blade.data.polygons:
        p_.use_smooth = False
    place(blade, F)
    # Parerstanga: et smidd stag, kraftig på midten og smalere ut mot de
    # svakt bøyde endene, med en liten skjoldplate over klinga.
    gsecs = []
    for i in range(25):
        t = -1 + 2 * i / 24
        x = t * 0.105
        hw = 0.011 - 0.004 * abs(t) ** 1.5
        hh = 0.009 - 0.0025 * abs(t)
        zc = 0.064 + 0.012 * abs(t) ** 2.2
        gsecs.append([Vector((x, dy * hw, zc + dz * hh)) for dy, dz in ((-1, -1), (1, -1), (1, 1), (-1, 1))])
    guard = loft("guard", [[Vector((p.y, p.z, p.x)) for p in s] for s in gsecs], guard_m)
    guard.data.transform(Matrix(((0, 0, 1), (1, 0, 0), (0, 1, 0))).to_4x4())
    bev = guard.modifiers.new("b", "BEVEL")
    bev.width = 0.0025
    bev.segments = 2
    for k, sx in enumerate((-1, 1)):
        lathe(f"guard_end{k}", [(0.0, -0.009), (0.008, -0.006), (0.0095, 0.0), (0.008, 0.006), (0.0, 0.009)], guard_m, 12)
        e = bpy.data.objects[f"guard_end{k}"]
        e.data.transform(Matrix.Translation((sx * 0.11, 0, 0.077)) @ Matrix.Rotation(math.pi / 2, 4, "Y"))
        place(e, F)
    eblock = lathe("ecusson", [(0.0, 0.054), (0.016, 0.056), (0.016, 0.078), (0.0, 0.08)], guard_m, 4)
    eblock.data.transform(Matrix.Scale(1.0, 4))
    place(eblock, F @ Matrix.Diagonal((1.0, 0.55, 1.0, 1.0)) @ Matrix.Rotation(math.pi / 4, 4, "Z"))
    place(guard, F)
    # Grepet: lærtvunnet over tre, litt ovalt, med spiralrygger.
    gsec = []
    for i in range(41):
        z = -0.075 + i / 40 * 0.13
        ring = []
        for k in range(24):
            a = 2 * math.pi * k / 24
            r = 0.0155 + 0.0013 * max(0.0, math.sin(2 * math.pi * (z / 0.011 + a / (2 * math.pi))))
            r *= 1.0 - 0.12 * math.cos(2 * a)
            ring.append(Vector((math.cos(a) * r * 0.85, math.sin(a) * r * 1.1, z)))
        gsec.append(ring)
    place(loft("grip", gsec, grip_m), F)
    for z in (-0.078, 0.057):
        place(lathe(f"grip_ferrule{z}", [(0.0, z - 0.004), (0.0175, z - 0.004), (0.0185, z), (0.0175, z + 0.004), (0.0, z + 0.004)], guard_m, 20), F)
    # Hjulknappen: flat skive med opphøyd midte, og angelen naglet over.
    wheel = lathe("pommel", [(0.0, -0.016), (0.026, -0.015), (0.032, -0.01), (0.033, 0.0), (0.032, 0.01), (0.026, 0.015), (0.0, 0.016)], guard_m, 32)
    for r, h in ((0.014, 0.019), ):
        boss = lathe("pommel_boss", [(0.0, -h), (r, -h + 0.002), (r, h - 0.002), (0.0, h)], guard_m, 24)
        place(boss, F @ Matrix.Translation((0, 0, -0.112)) @ Matrix.Rotation(math.pi / 2, 4, "Y"))
    place(wheel, F @ Matrix.Translation((0, 0, -0.112)) @ Matrix.Rotation(math.pi / 2, 4, "Y"))
    place(lathe("peen", [(0.0, -0.146), (0.007, -0.145), (0.006, -0.141), (0.004, -0.138)], guard_m, 12), F)
    return F


def shield_round():
    """Rundskjold, 76 cm: sju planker med fuger, malt lærtrekk (rødt og
    okergult, kvadrert, slitt ned til treet), råhudkant med sting,
    kuppelbule med nagler, håndtak og polstring bak."""
    c, axis, knuckle, wrist = grip_frame("L")
    # Skjoldet vender fram (som knokene) og litt inn mot midten.
    # Skjoldet holdes ut til venstre med kanten fram: flata vender fram og
    # ut mot venstre, så du ser kanten og litt av baksida.
    normal = Vector((-0.25, 1.0, 0.05)).normalized()
    F = frame_of(c + normal * 0.045, normal, Vector((0, 0, 1)))
    rng = random.Random(7)
    R = 0.34
    wood, wnt, wb = textured("shield_wood", "oak", lin((0.75, 0.6, 0.45)), 0.25, rough=0.7, bump_s=0.6, coords=local_coords_turned)
    # Malingen på trekket: kvadrert rødt og oker, flasset av i kantene og
    # langs riper, så treet kommer fram.
    paint, pnt, pb = new_mat("shield_paint")
    lc = local_coords(pnt)
    sep = pnt.nodes.new("ShaderNodeSeparateXYZ")
    pnt.links.new(lc, sep.inputs[0])
    quad = _math(pnt, "GREATER_THAN", _math(pnt, "MULTIPLY", sep.outputs["X"], sep.outputs["Y"]), 0.0)
    col = _mix(pnt, lin((0.55, 0.08, 0.05)), lin((0.78, 0.55, 0.18)), quad)
    # Et svart kors gjennom midten.
    cross = _math(pnt, "MAXIMUM", _ramp(pnt, _math(pnt, "ABSOLUTE", sep.outputs["X"]), 0.035, 0.03), _ramp(pnt, _math(pnt, "ABSOLUTE", sep.outputs["Y"]), 0.035, 0.03))
    col = _mix(pnt, col, lin((0.06, 0.05, 0.05)), cross)
    wear = noise(pnt, lc, 14.0, 8.0)
    rad = _math(pnt, "LENGTH", lc) if False else None
    rim = pnt.nodes.new("ShaderNodeVectorMath")
    rim.operation = "LENGTH"
    pnt.links.new(lc, rim.inputs[0])
    worn = _math(pnt, "ADD", _ramp(pnt, wear, 0.62, 0.66), _ramp(pnt, rim.outputs["Value"], R - 0.06, R - 0.01))
    img = _boxtex(pnt, f"{TEX}/oak/albedo.jpg", 0.25, lc)
    bare = _mix(pnt, img.outputs["Color"], lin((0.7, 0.55, 0.4)), 1.0, "MULTIPLY")
    grime = noise(pnt, lc, 4.0, 4.0)
    col = _mix(pnt, col, (0.25, 0.22, 0.2), _math(pnt, "MULTIPLY", _ramp(pnt, grime, 0.45, 0.7), 0.5), "MULTIPLY")
    col = _mix(pnt, col, bare, _math(pnt, "MINIMUM", worn, 1.0))
    pnt.links.new(col, pb.inputs["Base Color"])
    pnt.links.new(_math(pnt, "ADD", 0.55, _math(pnt, "MULTIPLY", worn, 0.2)), pb.inputs["Roughness"])
    bump(pnt, pb, _math(pnt, "SUBTRACT", 1.0, _math(pnt, "MINIMUM", worn, 1.0)), 0.4, 0.001)
    hide = leather("rawhide", (0.62, 0.5, 0.36), 0.6, 0.06)
    iron = forged("boss_iron", (0.3, 0.29, 0.28))
    grip_wood = leather("grip_wrap", (0.2, 0.13, 0.08), 0.6, 0.03)
    # Plankene: sju bord på tvers, klippet til sirkelen, med fuger.
    n = 7
    pw = 2 * R / n
    for k in range(n):
        y0 = -R + k * pw + 0.0008
        y1 = y0 + pw - 0.0016
        th = 0.011 + rng.uniform(-0.0012, 0.0012)
        pts = []
        steps = 10
        for side_y, sgn in ((y0, 1), (y1, -1)):
            pass
        top, bot = [], []
        for i in range(steps + 1):
            y = y1 - (y1 - y0) * i / steps if False else y0 + (y1 - y0) * i / steps
            x = math.sqrt(max(0.0, R * R - y * y))
            top.append((x, y))
            bot.append((-x, y))
        poly = top + bot[::-1]
        bm = bmesh.new()
        front = [bm.verts.new((x, y, 0.0)) for x, y in poly]
        back = [bm.verts.new((x, y, -th)) for x, y in poly]
        bm.faces.new(front)
        bm.faces.new(back[::-1])
        m_ = len(poly)
        for i in range(m_):
            j = (i + 1) % m_
            bm.faces.new((front[j], front[i], back[i], back[j]))
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        place(link_mesh(f"plank{k}", bm, wood, smooth=False), F)
    # Trekket: en tynn skive over plankene, malt.
    face = lathe("shield_face", [(0.0, 0.0), (R - 0.004, 0.0), (R - 0.004, 0.0018), (0.0, 0.0018)], paint, 64)
    place(face, F)
    # Kanten: råhud brettet over kanten, med sting gjennom.
    rim_secs = []
    for k in range(64):
        a = 2 * math.pi * k / 64
        d = Vector((math.cos(a), math.sin(a), 0))
        rim_secs.append([d * (R - 0.012) + Vector((0, 0, 0.004)), d * (R + 0.004) + Vector((0, 0, 0.002)), d * (R + 0.005) + Vector((0, 0, -0.012)), d * (R - 0.012) + Vector((0, 0, -0.014)), d * (R - 0.012) + Vector((0, 0, -0.011)), d * (R + 0.001) + Vector((0, 0, -0.01)), d * (R + 0.001) + Vector((0, 0, 0.0)), d * (R - 0.012) + Vector((0, 0, 0.0021))])
    rim_secs.append(rim_secs[0])
    place(loft("rim", rim_secs, hide, cap=(False, False)), F)
    for k in range(36):
        a = 2 * math.pi * (k + 0.5) / 36
        d = Vector((math.cos(a), math.sin(a), 0))
        bm = bmesh.new()
        bmesh.ops.create_cube(bm, size=1.0)
        for v in bm.verts:
            p = v.co
            v.co = d * (R - 0.008 + p.x * 0.008) + Vector((-d.y, d.x, 0)) * p.y * 0.0025 + Vector((0, 0, 0.0045 + p.z * 0.0015))
        place(link_mesh(f"stitch{k}", bm, hide, smooth=False), F)
    # Bula: flens med seks nagler og en kuppel.
    place(lathe("boss", [(0.0, 0.0018), (0.088, 0.0018), (0.088, 0.0045), (0.072, 0.007), (0.066, 0.016), (0.06, 0.03), (0.045, 0.045), (0.022, 0.053), (0.0, 0.055)], iron, 40), F)
    for k in range(6):
        a = 2 * math.pi * k / 6
        place(lathe(f"rivet{k}", [(0.0, 0.004), (0.007, 0.0045), (0.006, 0.0075), (0.003, 0.009), (0.0, 0.0095)], iron, 10), F @ Matrix.Translation((math.cos(a) * 0.08, math.sin(a) * 0.08, 0)))
    # Bak: håndtaket på tvers over hullet bak bula, og en polstret plate.
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co = Vector((v.co.x * 0.26, v.co.y * 0.03, -0.011 - 0.012 + v.co.z * 0.024))
    place(link_mesh("handle", bm, wood), F)
    hgrip = lathe("handle_wrap", [(0.0, -0.06), (0.017, -0.058), (0.018, 0.0), (0.017, 0.058), (0.0, 0.06)], grip_wood, 16)
    place(hgrip, F @ Matrix.Translation((0, 0, -0.035)) @ Matrix.Rotation(math.pi / 2, 4, "Y"))
    return F


def axe():
    """Skjeggøks: askeskaft med jernhylse og et smidd blad med skjegg."""
    c, axis, knuckle, _ = grip_frame("R")
    F = frame_of(c, axis, knuckle)
    shaft = leather("axe_shaft", (0.7, 0.55, 0.38), 0.6, 0.1)
    head_m = forged("axe_iron", (0.34, 0.33, 0.33))
    edge_m = steel("axe_edge", 0.15)
    place(lathe("shaft", [(0.0, -0.12), (0.017, -0.118), (0.018, 0.2), (0.016, 0.52), (0.0, 0.53)], shaft, 16), F)
    # Bladet: profilen i kneplanet (x fram), tykkere bak, tynt mot eggen.
    prof = [(0.0, 0.05), (0.06, 0.06), (0.15, 0.12), (0.17, 0.0), (0.15, -0.1), (0.07, -0.05), (0.03, -0.035), (0.0, -0.03)]
    secs = []
    for x, z in prof:
        t = min(1.0, x / 0.17)
        th = 0.012 * (1 - t) + 0.0015
        secs.append((x, z, th))
    bm = bmesh.new()
    top = [bm.verts.new((x, th / 2, 0.45 + z)) for x, z, th in secs]
    bot = [bm.verts.new((x, -th / 2, 0.45 + z)) for x, z, th in secs]
    bm.faces.new(top)
    bm.faces.new(bot[::-1])
    for i in range(len(secs)):
        j = (i + 1) % len(secs)
        f = bm.faces.new((top[j], top[i], bot[i], bot[j]))
        f.material_index = 1 if i in (2, 3) else 0
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    place(link_mesh("axe_head", bm, [head_m, edge_m], smooth=False), F)
    place(lathe("socket", [(0.0, 0.39), (0.024, 0.39), (0.026, 0.45), (0.024, 0.51), (0.0, 0.51)], head_m, 16), F)
    return F


def dagger():
    """Rondelldolk: smal trekantet klinge, skiver av jern som parer og knapp."""
    c, axis, knuckle, _ = grip_frame("R")
    F = frame_of(c, axis, knuckle)
    blade_m = steel("dagger_steel", 0.18)
    iron = forged("dagger_iron")
    grip_m = leather("dagger_grip", (0.25, 0.16, 0.1), 0.6, 0.03)
    secs = []
    for i in range(25):
        u = i / 24
        z = 0.07 + u * 0.3
        w = 0.022 * (1 - u) + 0.0005
        th = 0.01 * (1 - u) + 0.0005
        secs.append([Vector((w / 2, 0, z)), Vector((0, th / 2, z)), Vector((-w / 2, 0, z)), Vector((0, -th / 2, z))])
    place(loft("blade", secs, blade_m), F)
    for z in (0.065, -0.075):
        place(lathe(f"rondel{z}", [(0.0, z - 0.005), (0.03, z - 0.004), (0.032, z), (0.03, z + 0.004), (0.0, z + 0.005)], iron, 24), F)
    place(lathe("grip", [(0.0, -0.07), (0.014, -0.07), (0.016, 0.0), (0.014, 0.06), (0.0, 0.06)], grip_m, 16), F)
    return F


def shield_tower():
    """Trekantskjold (heater): buet treskjold med lærtrekk i blått og hvitt,
    jernkant og håndtak bak."""
    c, axis, knuckle, _ = grip_frame("L")
    normal = Vector((-0.95, 0.3, 0.05)).normalized()
    F = frame_of(c + normal * 0.05, normal, Vector((0, 0, 1)))
    paint, pnt, pb = new_mat("heater_paint")
    lc = local_coords(pnt)
    sep = pnt.nodes.new("ShaderNodeSeparateXYZ")
    pnt.links.new(lc, sep.inputs[0])
    stripes = _math(pnt, "GREATER_THAN", _math(pnt, "SINE", _math(pnt, "MULTIPLY", sep.outputs["X"], 40.0)), 0.0)
    col = _mix(pnt, lin((0.12, 0.2, 0.45)), lin((0.85, 0.82, 0.74)), stripes)
    wear = noise(pnt, lc, 12.0, 8.0)
    col = _mix(pnt, col, lin((0.5, 0.38, 0.26)), _ramp(pnt, wear, 0.64, 0.68))
    pnt.links.new(col, pb.inputs["Base Color"])
    pb.inputs["Roughness"].default_value = 0.6
    iron = forged("heater_iron")
    grip_m = leather("heater_grip", (0.2, 0.13, 0.08), 0.6, 0.03)
    # Omrisset: rett topp, buede sider ned mot spissen; buet om lokal y.
    outline = []
    for i in range(21):
        t = i / 20
        outline.append((-0.3 + 0.6 * t, 0.36))
    for i in range(1, 21):
        t = i / 20
        x = 0.3 * math.cos(t * math.pi / 2) ** 0.8
        outline.append((x, 0.36 - t * 0.9))
    for i in range(1, 20):
        t = 1 - i / 20
        x = -0.3 * math.cos(t * math.pi / 2) ** 0.8
        outline.append((x, 0.36 - t * 0.9))
    bend = lambda x: -0.9 * x * x  # noqa: E731
    bm = bmesh.new()
    front = [bm.verts.new((x, y, bend(x))) for x, y in outline]
    back = [bm.verts.new((x, y, bend(x) - 0.014)) for x, y in outline]
    bm.faces.new(front)
    bm.faces.new(back[::-1])
    for i in range(len(outline)):
        j = (i + 1) % len(outline)
        bm.faces.new((front[j], front[i], back[i], back[j]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    place(link_mesh("heater", bm, paint, smooth=True), F)
    for x0, y0, x1, y1 in ((-0.3, 0.36, 0.3, 0.36),):
        bm = bmesh.new()
        bmesh.ops.create_cube(bm, size=1.0)
        for v in bm.verts:
            v.co = Vector((v.co.x * 0.62, 0.36 + v.co.y * 0.02, v.co.z * 0.02))
        place(link_mesh("heater_band", bm, iron), F)
    place(lathe("heater_handle", [(0.0, -0.06), (0.016, -0.058), (0.017, 0.0), (0.016, 0.058), (0.0, 0.06)], grip_m, 16), F @ Matrix.Translation((0, 0, -0.035)) @ Matrix.Rotation(math.pi / 2, 4, "Y"))
    return F


WEAPONS = {"sword": sword, "axe": axe, "dagger": dagger, "shield_round": shield_round, "shield_tower": shield_tower}


def preview(out):
    """Rendrer det spilleren ser: kameraet i origo langs +Y, 72° høyt."""
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 32
    scene.render.resolution_x, scene.render.resolution_y = 1280, 720
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[0].default_value = (0.2, 0.18, 0.16, 1)
    world.node_tree.nodes["Background"].inputs[1].default_value = 0.6
    scene.world = world
    bpy.ops.object.light_add(type="POINT", location=(0.8, 0.9, 0.5))
    bpy.context.active_object.data.energy = 80
    bpy.context.active_object.data.color = (1.0, 0.7, 0.45)
    bpy.ops.object.light_add(type="POINT", location=(-0.6, 0.2, 0.4))
    bpy.context.active_object.data.energy = 25
    bpy.ops.object.camera_add(location=(0, 0, 0))
    cam = bpy.context.active_object
    cam.rotation_euler = (math.pi / 2, 0, 0)
    cam.data.sensor_fit = "VERTICAL"
    cam.data.angle = math.radians(72)
    cam.data.clip_start = 0.05
    scene.camera = cam
    scene.render.filepath = out
    bpy.ops.render.render(write_still=True)
    # Og fra siden og ovenfra, til å se hele oppstillingen.
    for tag, loc in (("side", (1.1, 0.3, 0.1)), ("top", (0.0, 0.35, 1.2))):
        bpy.ops.object.camera_add(location=loc)
        c2 = bpy.context.active_object
        c2.rotation_euler = (Vector((0.0, 0.35, -0.3)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
        c2.data.lens = 24
        scene.camera = c2
        scene.render.filepath = out.replace(".png", f"_{tag}.png")
        bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    args = sys.argv[1:]
    size = next((int(a) for a in args if a.isdigit()), 2048)
    which = [a for a in args if a in WEAPONS or a == "arms"] or ["arms"] + list(WEAPONS)
    prev = args[args.index("--preview") + 1] if "--preview" in args else None
    os.makedirs(OUT, exist_ok=True)
    if prev:
        # Alt sammen i ett bilde: armene og sverd og skjold.
        clear()
        build_arms()
        sword()
        shield_round()
        for ob in bpy.data.objects:
            for m in ob.modifiers:
                if m.type == "BEVEL":
                    pass
        preview(prev)
        sys.exit(0)
    for name in which:
        clear()
        if name == "arms":
            bones = build_arms()
            rig = armature("arms_rig", bones)
            clips(rig)
            bake_textured(f"{OUT}/arms.glb", size, keep=("glass", "glow", "fire", "mesh"), armature=rig, rigid={f"{k}{s}": f"forearm.{s}" for k in ("buckle", "strap") for s in "LR"}, skin={"blend": 0.035})
        else:
            WEAPONS[name]()
            bake_textured(f"{OUT}/{name}.glb", 1024, keep=("glass", "glow", "fire", "mesh"))
