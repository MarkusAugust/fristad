"""Innbyggerne fra en 3D-form (`assets/textures/people/3d/<navn>.glb`, laget
av TRELLIS.2 fra bildet forfra av arket i A-stilling, `<navn>_apose.jpeg`).

Formen og teksturen er riktige fra start (ansiktet, hendene, forkleet), så
her er det bare å gjøre den spillklar, uten å male kroppen på nytt (eieren
15. oktober: teksturen som følger med passer hundre ganger bedre enn arket
projisert på formen):

1. reise den opp med fronten mot +X og skalere til høyden i `FOLK`;
2. et tett, lukket skall over formen (`VOXEL`), forenklet til `TRIS`
   trekanter (`simplify`);
3. fargen fra den fulle formen bakt uendret over på skallet (`transfer`),
   og ansiktet fra bildet forfra som TRELLIS ble laget av (`face_patch`):
   TRELLIS mister små ting som øynene, men bildet forfra passer nøyaktig
   på flatene som vender rett fram;
4. finne leddene i formen (`landmarks`), rigge spillets 18 bein og lage
   klippene fra `folk_motion`.

    python3.13 tools/models/folk3d.py smith [størrelse]
"""

import math
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from mathutils import Vector  # noqa: E402
from common import armature, bake_textured, clear  # noqa: E402
import folk_motion  # noqa: E402
from townsfolk import FOLK, bones  # noqa: E402

SRC = "assets/textures/people/3d"
REF = "assets/textures/people/ref"
OUT = "assets/models/npcs"
TRIS = 16000  # trekanter etter forenklingen
SRC_SIZE = 4096  # fargen fra formen bakes først i denne størrelsen
HEAD_DENSITY = 3.0
HEAD_FROM = 0.82  # hodet og halsen, brøk av høyden
THICK = 0.012  # tykkelsen de åpne flatene får før skallet (m)
FACE_FROM = 0.80  # ansiktet fra bildet forfra legges over denne brøken av høyden
FACE_INSET = 0.006  # masken trekkes så mye inn fra omrisset (brøk av bredden)
FACE_FACING = (0.35, 0.7)  # normalen mot kameraet: fra ingen til full vekt
VOXEL = 0.01  # skallet (m)
LOOSE = 0.002  # løse biter med færre flater enn denne brøken av skallet fjernes


def load(name):
    """Formen stående med føttene på bakken, fronten mot +X og høyden fra
    `FOLK`."""
    bpy.ops.import_scene.gltf(filepath=os.path.join(SRC, f"{name}.glb"))
    obs = [o for o in bpy.data.objects if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in obs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = obs[0]
    if len(obs) > 1:
        bpy.ops.object.join()
    ob = bpy.context.view_layer.objects.active
    from mathutils import Matrix
    mw = ob.matrix_world.copy()
    ob.parent = None
    ob.matrix_world = Matrix.Identity(4)
    # TRELLIS ser mot −Y i Blender; spillet vil ha fronten mot +X.
    ob.data.transform(Matrix.Rotation(math.radians(90), 4, "Z") @ mw)
    co = np.array([v.co[:] for v in ob.data.vertices])
    lo, hi = co.min(0), co.max(0)
    s = FOLK[name] / (hi[2] - lo[2])
    ob.data.transform(Matrix.Translation((0, 0, -lo[2] * s)) @ Matrix.Scale(s, 4) @ Matrix.Translation((-(lo[0] + hi[0]) / 2, -(lo[1] + hi[1]) / 2, 0)))
    for o in [o for o in bpy.data.objects if o is not ob]:
        bpy.data.objects.remove(o)
    ob.name = "body"
    return ob


def simplify(ob, tris=TRIS):
    """Et tett, lukket skall over formen (voxel), forenklet til `tris`.
    Formen fra TRELLIS har åpne kanter og hjørner delt langs sømmene, og
    da river forenklingen den i småbiter; fargen hentes fra den fulle
    formen etterpå (`transfer`)."""
    bpy.context.view_layer.objects.active = ob
    # Åpne, tynne flater (forkleet, håret) får tykkelse først, ellers
    # faller de mellom vokslene og skallet får hull.
    so = ob.modifiers.new("thick", "SOLIDIFY")
    so.thickness = THICK
    so.offset = 0.0
    bpy.ops.object.modifier_apply(modifier=so.name)
    rm = ob.modifiers.new("remesh", "REMESH")
    rm.mode = "VOXEL"
    rm.voxel_size = VOXEL
    rm.use_smooth_shade = True
    bpy.ops.object.modifier_apply(modifier=rm.name)
    # Løse småbiter (hårtuster, flak) kan ikke forenkles og gir tusen
    # småøyer i teksturen; bare biter med minst `LOOSE` av flatene står.
    import bmesh
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    bm.faces.ensure_lookup_table()
    seen = set()
    drop = []
    for f in bm.faces:
        if f.index in seen:
            continue
        comp, st = [f], [f]
        seen.add(f.index)
        while st:
            g = st.pop()
            for e in g.edges:
                for h in e.link_faces:
                    if h.index not in seen:
                        seen.add(h.index)
                        comp.append(h)
                        st.append(h)
        if len(comp) < LOOSE * len(bm.faces):
            drop += comp
    print("skall", len(bm.faces), "løse flater", len(drop))
    bmesh.ops.delete(bm, geom=drop, context="FACES")
    bm.to_mesh(ob.data)
    bm.free()
    for _ in range(4):
        n = sum(len(p.vertices) - 2 for p in ob.data.polygons)
        if n <= tris * 1.05:
            break
        mod = ob.modifiers.new("dec", "DECIMATE")
        mod.ratio = tris / n
        bpy.ops.object.modifier_apply(modifier=mod.name)
        print("forenklet", n, "->", sum(len(p.vertices) - 2 for p in ob.data.polygons))
    for p in ob.data.polygons:
        p.use_smooth = True
    return ob


def transfer(hi, lo, name, size=2048):
    """Fargen fra den fulle formen bakes over på den forenklede i egne uv
    (laget «src»): forenklingen slår sammen flater over sømmene i formens
    uv, og der ble bildet revet i svarte flekker."""
    me = lo.data
    while me.uv_layers:
        me.uv_layers.remove(me.uv_layers[0])
    me.uv_layers.new(name="src")
    bpy.ops.object.select_all(action="DESELECT")
    lo.select_set(True)
    bpy.context.view_layer.objects.active = lo
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=1.15, island_margin=0.004)
    bpy.ops.object.mode_set(mode="OBJECT")
    img = bpy.data.images.new(f"src_{name}", size, size)
    mat = bpy.data.materials.new(f"npc_{name}")
    mat.use_nodes = True
    nt = mat.node_tree
    t = nt.nodes.new("ShaderNodeTexImage")
    t.image = img
    nt.nodes.active = t
    me.materials.clear()
    me.materials.append(mat)
    for m in hi.data.materials:
        b = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
        b.inputs["Metallic"].default_value = 0.0
        for l in list(b.inputs["Metallic"].links):
            m.node_tree.links.remove(l)
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    sc.cycles.samples = 1
    sc.render.bake.use_selected_to_active = True
    sc.render.bake.cage_extrusion = 0.03
    sc.render.bake.max_ray_distance = 0.0
    sc.render.bake.margin = 8
    hi.hide_render = False
    hi.select_set(True)
    lo.select_set(True)
    bpy.context.view_layer.objects.active = lo
    bpy.ops.object.bake(type="DIFFUSE", pass_filter={"COLOR"})
    sc.render.bake.use_selected_to_active = False
    if os.environ.get("FOLK3D_DUMP"):
        img.filepath_raw = os.environ["FOLK3D_DUMP"]
        img.file_format = "PNG"
        img.save()
    bpy.data.objects.remove(hi)
    bsdf = nt.nodes["Principled BSDF"]
    uv = nt.nodes.new("ShaderNodeUVMap")
    uv.uv_map = "src"
    nt.links.new(uv.outputs[0], t.inputs[0])
    nt.links.new(t.outputs["Color"], bsdf.inputs["Base Color"])
    bsdf.inputs["Roughness"].default_value = 0.9
    bsdf.inputs["Metallic"].default_value = 0.0
    me.uv_layers.new(name="UVMap", do_init=True)


def face_patch(ob, name, H):
    """Ansiktet fra bildet TRELLIS ble laget av (`<navn>_apose_front.png`).
    TRELLIS maler teksturen ut fra formen, og små ting som øynene blir
    borte (smeden fikk lukkede streker). Bildet legges bare på ansiktet: på
    hodet, på flatene som vender rett fram og innenfor bredden av ansiktet,
    med myk overgang til fargen fra formen. Resten av figuren står urørt."""
    path = os.path.join(REF, f"{name}_apose_front.png")
    if not os.path.exists(path):
        return
    im = np.asarray(Image.open(path).convert("RGB")).astype(float)
    bg = np.median(np.concatenate([im[:8].reshape(-1, 3), im[-8:].reshape(-1, 3)]), 0)
    fg = np.linalg.norm(im - bg, axis=2) > 30
    rows, cols = np.nonzero(fg)
    W, Hh = im.shape[1], im.shape[0]
    top, bot = rows.min(), rows.max()
    # Bare innenfor omrisset, trukket `FACE_INSET` inn og myknet: formen fra
    # TRELLIS stikker litt utenfor (håret), og der lå bakgrunnen og ble
    # malt på som lyse flekker.
    from scipy import ndimage
    k = max(1, int(FACE_INSET * W))
    inner = ndimage.binary_erosion(ndimage.binary_fill_holes(fg), iterations=k)
    soft = ndimage.gaussian_filter(inner.astype(float), k / 2)
    os.makedirs("target/folk3d", exist_ok=True)
    mpath = os.path.abspath(f"target/folk3d/{name}_face_mask.png")
    Image.fromarray((soft * 255).astype(np.uint8)).save(mpath)
    mask = bpy.data.images.load(mpath)
    mask.colorspace_settings.name = "Non-Color"
    sc = (bot - top) / H  # piksler per meter
    co = np.array([v.co[:] for v in ob.data.vertices])
    lo, hi = co.min(0), co.max(0)
    # Midten av figuren i bildet og i formen (begge midt mellom ytterkantene).
    cu = (cols.min() + cols.max()) / 2 - (lo[1] + hi[1]) / 2 * sc
    head = co[co[:, 2] > FACE_FROM * H]
    hx = float(np.median(head[:, 0]))
    hw = float(np.percentile(np.abs(head[:, 1]), 90))
    mat = ob.data.materials[0]
    nt = mat.node_tree
    N, L = nt.nodes, nt.links
    bsdf = N["Principled BSDF"]
    base = bsdf.inputs["Base Color"].links[0].from_socket
    geo = N.new("ShaderNodeNewGeometry")
    pos = N.new("ShaderNodeSeparateXYZ")
    L.new(geo.outputs["Position"], pos.inputs[0])
    nrm = N.new("ShaderNodeSeparateXYZ")
    L.new(geo.outputs["Normal"], nrm.inputs[0])

    def lin(sock, mul, add):
        m = N.new("ShaderNodeMath")
        m.operation = "MULTIPLY_ADD"
        L.new(sock, m.inputs[0])
        m.inputs[1].default_value = mul
        m.inputs[2].default_value = add
        return m.outputs[0]

    def ramp(sock, a, b):
        r = N.new("ShaderNodeMapRange")
        r.interpolation_type = "SMOOTHSTEP"
        L.new(sock, r.inputs["Value"])
        r.inputs["From Min"].default_value = a
        r.inputs["From Max"].default_value = b
        return r.outputs["Result"]

    def mul(a, b):
        m = N.new("ShaderNodeMath")
        m.operation = "MULTIPLY"
        L.new(a, m.inputs[0])
        L.new(b, m.inputs[1])
        return m.outputs[0]

    uv = N.new("ShaderNodeCombineXYZ")
    L.new(lin(pos.outputs["Y"], sc / W, cu / W), uv.inputs[0])
    L.new(lin(pos.outputs["Z"], sc / Hh, 1 - (top + H * sc) / Hh), uv.inputs[1])
    t = N.new("ShaderNodeTexImage")
    t.image = bpy.data.images.load(os.path.abspath(path))
    t.extension = "EXTEND"
    L.new(uv.outputs[0], t.inputs[0])
    mt = N.new("ShaderNodeTexImage")
    mt.image = mask
    mt.extension = "CLIP"
    L.new(uv.outputs[0], mt.inputs[0])
    ay = N.new("ShaderNodeMath")
    ay.operation = "ABSOLUTE"
    L.new(pos.outputs["Y"], ay.inputs[0])
    w = mul(ramp(pos.outputs["Z"], FACE_FROM * H, (FACE_FROM + 0.02) * H), ramp(nrm.outputs["X"], FACE_FACING[0], FACE_FACING[1]))
    w = mul(w, ramp(lin(ay.outputs[0], -1.0, hw), 0.0, 0.25 * hw))
    w = mul(w, ramp(pos.outputs["X"], hx - 0.01, hx + 0.02))
    w = mul(w, mt.outputs["Color"])
    mix = N.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    L.new(w, mix.inputs["Factor"])
    L.new(base, mix.inputs["A"])
    L.new(t.outputs["Color"], mix.inputs["B"])
    L.new(mix.outputs["Result"], bsdf.inputs["Base Color"])


def head_density(ob, H):
    """Ansiktet er det man ser på: hodet får `HEAD_DENSITY` ganger så mye
    plass i teksturen som resten (`bake_textured` leser «uv_density»)."""
    a = ob.data.attributes.new("uv_density", "FLOAT", "FACE")
    a.data.foreach_set("value", [HEAD_DENSITY if p.center.z > HEAD_FROM * H else 1.0 for p in ob.data.polygons])


def landmarks(ob, H):
    """Leddene i formen (meter; x fram, y mot venstre, z opp), lest av
    hjørnene: armene står ut i A-stilling, så fingertuppene er det som står
    lengst ut til sidene, og beina skilles under forkleet."""
    co = np.array([v.co[:] for v in ob.data.vertices])
    x, y, z = co[:, 0], co[:, 1], co[:, 2]

    def band(z0, z1):
        return (z >= z0 * H) & (z <= z1 * H)

    J = {}
    mid = lambda m: float((x[m].min() + x[m].max()) / 2) if m.any() else 0.0  # noqa: E731
    # Ryggraden midt i kroppen, i høyde etter vanlige proporsjoner.
    core = np.abs(y) < 0.08 * H
    for k, f in (("pelvis", 0.53), ("spine03", 0.60), ("spine01", 0.71), ("neck", 0.835), ("head", 0.875)):
        J[k] = Vector((mid(core & band(f - 0.02, f + 0.02)), 0.0, f * H))
    J["top"] = Vector((J["head"].x, 0.0, float(z.max())))
    for s, sg in (("L", 1), ("R", -1)):
        side = sg * y > 0
        # Fingertuppen: det som står lengst ut, mellom hofta og brystet.
        arm = side & band(0.3, 0.8)
        reach = sg * y[arm]
        far = reach >= np.percentile(reach, 99.7)
        tip = co[arm][far].mean(0)
        J[f"finger.{s}"] = Vector(tip)
        # Skulderen: litt inn fra kanten av kroppen i skulderhøyde.
        sh = side & band(0.80, 0.83)
        both = band(0.80, 0.83)
        w = float(np.percentile(np.abs(y[both]), 97)) if both.any() else 0.18 * H
        J[f"shoulder.{s}"] = Vector((mid(sh), sg * 0.78 * w, 0.805 * H))
        sv = J[f"shoulder.{s}"]
        d = (Vector(tip) - sv)
        L = d.length
        J[f"wrist.{s}"] = sv + d * ((L - 0.105 * H) / L)
        el = sv + (J[f"wrist.{s}"] - sv) * 0.5
        J[f"elbow.{s}"] = Vector((el.x - 0.015 * H, el.y, el.z))
        # Beina: midten av hvert bein i kne- og ankelhøyde.
        knee = side & band(0.26, 0.30) & (sg * y < 0.2 * H)
        ank = side & band(0.045, 0.07) & (sg * y < 0.2 * H)
        ky = float(np.median(y[knee])) if knee.any() else sg * 0.1 * H
        ay = float(np.median(y[ank])) if ank.any() else ky
        J[f"knee.{s}"] = Vector((mid(knee), ky, 0.28 * H))
        J[f"ankle.{s}"] = Vector((mid(ank) - 0.01 * H, ay, 0.055 * H))
        J[f"hip.{s}"] = Vector((J["pelvis"].x, ky * 0.9, 0.51 * H))
        foot = side & band(0.0, 0.04) & (np.abs(y - ay) < 0.06 * H)
        J[f"toe.{s}"] = Vector((float(x[foot].max()) if foot.any() else J[f"ankle.{s}"].x + 0.12 * H, ay, 0.02 * H))
    return J


def rig(ob, name, J):
    arm = armature(f"npc_{name}_rig", bones(J))
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    if not any(len(v.groups) for v in ob.data.vertices[:200]):
        bpy.ops.object.parent_set(type="ARMATURE_ENVELOPE")
    return arm


def build(name, size=2048):
    clear()
    ob = load(name)
    hi = ob.copy()
    hi.data = ob.data.copy()
    bpy.context.scene.collection.objects.link(hi)
    simplify(ob)
    transfer(hi, ob, name, SRC_SIZE)
    face_patch(ob, name, FOLK[name])
    head_density(ob, FOLK[name])
    H = FOLK[name]
    J = landmarks(ob, H)
    arm = rig(ob, name, J)
    style, grips = folk_motion.FIGURES.get(name, ("default", {}))
    st = dict(folk_motion.STYLES.get(style, {}))
    out = max(math.degrees(math.atan2(abs(J[f"wrist.{s}"].y - J[f"shoulder.{s}"].y), J[f"shoulder.{s}"].z - J[f"wrist.{s}"].z)) for s in ("L", "R"))
    st["arm_out"] = folk_motion.style_of(style)["arm_out"] * 0.5 - out * 0.65
    folk_motion.clips(arm, st, {}, only=("idle", "walk", "talk", "wave"))
    bake_textured(f"{OUT}/npc_{name}.glb", size, keep=("glass", "magic"), armature=arm, uv_angle=None, ao=False)
    return J


if __name__ == "__main__":
    args = sys.argv[1:]
    names = [a for a in args if a in FOLK]
    size = next((int(a) for a in args if a.isdigit()), 2048)
    for n in names:
        build(n, size)
