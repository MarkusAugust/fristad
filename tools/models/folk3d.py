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
from common import armature, bake_textured, clear, pack_weighted  # noqa: E402
import folk_motion  # noqa: E402
from townsfolk import FOLK, bones  # noqa: E402

SRC = "assets/textures/people/3d"
REF = "assets/textures/people/ref"
OUT = "assets/models/npcs"
TRIS = 16000  # trekanter etter forenklingen
SRC_SIZE = 3072  # fargen fra formen bakes først i denne størrelsen
HEAD_DENSITY = float(os.environ.get("FOLK3D_HEAD", 3.0))
HEAD_FROM = 0.82  # hodet og halsen, brøk av høyden
THICK = float(os.environ.get("FOLK3D_THICK", 0.006))  # tykkelsen de åpne flatene får før skallet (m)
FACE_FROM = 0.80  # ansiktet fra bildet forfra legges over denne brøken av høyden
FACE_INSET = 0.006  # masken trekkes så mye inn fra omrisset (brøk av bredden)
FACE_FACING = (0.35, 0.7)
FACE_DOWN = (-0.6, -0.35)
FACE_BLUR = 0.004  # uskarpheten fargene sammenlignes med (brøk av bildet)
FACE_AGREE = (0.15, 0.25)  # metning bildet har mer enn formen: full vekt under, ingen over  # flater som vender mer nedover enn dette, får ikke bildet  # normalen mot kameraet: fra ingen til full vekt
OLD_KEEP = ("speed", "cycle", "tempo", "width", "toe_out", "lift", "hs", "to", "gestures", "talk", "breath")  # det som står av stilen «old»
POSTURE = {"hunch": 0.25, "lean": 0.4, "jut": 0.3, "head": 0.5}  # stilens holdning skaleres, formen har sin egen
NEAREST = (0.03, 0.05)  # strålen begynner så langt utenfor skallet og går så langt innenfor det (m)
REMESH = "decimate"  # nettet over skallet: forenkling, eller «quadriflow» (trenger et rent, lukket skall)
VOXEL = float(os.environ.get("FOLK3D_VOXEL", 0.005))  # skallet (m); grovere smeltet armen sammen med skjørtet
WEIGHT_RADIUS = 0.05  # lag nærmere enn dette (m) får samme vekter (`layered_weights`)
LIMB_BONES = ("arm", "hand", "shoulder")  # bein som ikke jevnes ut med kroppen
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
    if os.environ.get("FOLK3D_REMESH", REMESH) == "quadriflow":
        # Et jevnt firkantnett over skallet: forenklingen presset flatene i
        # trange mellomrom (armen inntil skjørtet) gjennom hverandre, og
        # skjørtet stakk ut gjennom armen som flekker.
        with bpy.context.temp_override(object=ob, active_object=ob, selected_objects=[ob], selected_editable_objects=[ob]):
            bpy.ops.object.quadriflow_remesh(target_faces=tris // 2, use_preserve_sharp=False, use_mesh_symmetry=False, smooth_normals=True)
        print("quadriflow", sum(len(p.vertices) - 2 for p in ob.data.polygons))
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


def nearest_colors(hi, lo, img, size):
    """Fargen i hvert punkt på skallet hentes fra det nærmeste punktet på den
    fulle formen. Med stråler fra skallet (bake selected-to-active) traff
    strålene skjørtet bak forkleet og innsida av tynne armer (svarte flekker
    og mønstre på armene); nærmeste punkt treffer alltid rett flate."""
    from mathutils.bvhtree import BVHTree
    from scipy import ndimage

    # Bildet fra formen (base color), i tall.
    tex = next(n.image for m in hi.data.materials for n in m.node_tree.nodes
               if n.type == "TEX_IMAGE" and any(l.to_socket.name == "Base Color" for l in n.outputs[0].links))
    tw, th = tex.size
    src = np.empty(tw * th * 4, np.float32)
    tex.pixels.foreach_get(src)
    src = src.reshape(th, tw, 4)
    # Den fulle formen: hjørnene og uv per trekant.
    hm = hi.data
    hm.calc_loop_triangles()
    hv = np.array([v.co[:] for v in hm.vertices], np.float64)
    huv_layer = hm.uv_layers[0].data
    tris = hm.loop_triangles
    tri_v = np.array([t.vertices[:] for t in tris])
    tri_l = np.array([t.loops[:] for t in tris])
    luv = np.empty(len(huv_layer) * 2, np.float32)
    huv_layer.foreach_get("uv", luv)
    luv = luv.reshape(-1, 2)
    poly_of = np.array([t.polygon_index for t in tris])
    bvh = BVHTree.FromPolygons(hv.tolist(), tri_v.tolist(), all_triangles=True)
    # Skallet: hvert punkt i teksturen innenfor en trekant (litt utenfor
    # kanten også, så sømmene får farge), med stedet i rommet.
    lm = lo.data
    lm.calc_loop_triangles()
    lv = np.array([v.co[:] for v in lm.vertices], np.float64)
    ln = np.array([v.normal[:] for v in lm.vertices], np.float64)
    luv_lo = np.empty(len(lm.uv_layers["src"].data) * 2, np.float32)
    lm.uv_layers["src"].data.foreach_get("uv", luv_lo)
    luv_lo = luv_lo.reshape(-1, 2) * size
    pix_idx, pix_pos, pix_nrm = [], [], []
    for t in lm.loop_triangles:
        uv = luv_lo[list(t.loops)]
        p = lv[list(t.vertices)]
        nv = ln[list(t.vertices)]
        x0, y0 = np.floor(uv.min(0) - 1).astype(int)
        x1, y1 = np.ceil(uv.max(0) + 1).astype(int)
        xs, ys = np.meshgrid(np.arange(max(x0, 0), min(x1, size)), np.arange(max(y0, 0), min(y1, size)))
        c = np.stack([xs.ravel() + 0.5, ys.ravel() + 0.5], 1)
        a, b, cc = uv
        m = np.array([[b[0] - a[0], cc[0] - a[0]], [b[1] - a[1], cc[1] - a[1]]])
        det = np.linalg.det(m)
        if abs(det) < 1e-12:
            continue
        w = np.linalg.solve(m, (c - a).T).T
        w = np.column_stack([1 - w.sum(1), w])
        inside = (w > -0.02).all(1)
        if not inside.any():
            continue
        w = np.clip(w[inside], 0, None)
        w /= w.sum(1, keepdims=True)
        pix_idx.append(ys.ravel()[inside] * size + xs.ravel()[inside])
        pix_pos.append(w @ p)
        pix_nrm.append(w @ nv)
    pix_idx = np.concatenate(pix_idx)
    pix_pos = np.concatenate(pix_pos)
    pix_nrm = np.concatenate(pix_nrm)
    pix_nrm /= np.maximum(np.linalg.norm(pix_nrm, axis=1, keepdims=True), 1e-9)
    # Nærmeste punkt på den fulle formen, og uv-en der.
    hit_tri = np.zeros(len(pix_pos), np.int64)
    hit_loc = np.zeros((len(pix_pos), 3))
    # Den ytterste flaten langs normalen (en stråle utenfra og inn), ellers
    # den nærmeste: skjørtet i formen kan gå gjennom armen, og da var det
    # nærmeste punktet ofte skjørtet inni armen.
    from mathutils import Vector as V
    out_d, reach = NEAREST[0], NEAREST[0] + NEAREST[1]
    for i, (q, n) in enumerate(zip(pix_pos, pix_nrm)):
        nn = V(n)
        loc, _n, k, _d = bvh.ray_cast(V(q) + nn * out_d, -nn, reach)
        if k is None:
            loc, _n, k, _d = bvh.find_nearest(V(q))
        if k is not None:
            hit_tri[i] = k
            hit_loc[i] = loc
    A, B, C = hv[tri_v[hit_tri, 0]], hv[tri_v[hit_tri, 1]], hv[tri_v[hit_tri, 2]]
    v0, v1, v2 = B - A, C - A, hit_loc - A
    d00, d01, d11 = (v0 * v0).sum(1), (v0 * v1).sum(1), (v1 * v1).sum(1)
    d20, d21 = (v2 * v0).sum(1), (v2 * v1).sum(1)
    den = np.maximum(d00 * d11 - d01 * d01, 1e-18)
    bv = (d11 * d20 - d01 * d21) / den
    bw = (d00 * d21 - d01 * d20) / den
    bu = 1 - bv - bw
    uv = luv[tri_l[hit_tri, 0]] * bu[:, None] + luv[tri_l[hit_tri, 1]] * bv[:, None] + luv[tri_l[hit_tri, 2]] * bw[:, None]
    # Bilineært i bildet fra formen.
    fx = np.clip(uv[:, 0] * tw - 0.5, 0, tw - 1.001)
    fy = np.clip(uv[:, 1] * th - 0.5, 0, th - 1.001)
    ix, iy = fx.astype(int), fy.astype(int)
    ax, ay = (fx - ix)[:, None], (fy - iy)[:, None]
    col = (src[iy, ix] * (1 - ax) * (1 - ay) + src[iy, ix + 1] * ax * (1 - ay) + src[iy + 1, ix] * (1 - ax) * ay + src[iy + 1, ix + 1] * ax * ay)
    out = np.zeros((size * size, 4), np.float32)
    filled = np.zeros(size * size, bool)
    out[pix_idx] = col
    out[pix_idx, 3] = 1.0
    filled[pix_idx] = True
    # Det tomme rundt øyene får fargen fra nærmeste fylte punkt.
    _, (iy2, ix2) = ndimage.distance_transform_edt(~filled.reshape(size, size), return_indices=True)
    out = out.reshape(size, size, 4)[iy2, ix2]
    img.pixels.foreach_set(out.ravel())
    del poly_of
    print("nærmeste punkt for", len(pix_pos), "punkter")


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
    # Hodet får plass og øyene pakkes her, før fargen hentes: teksturen
    # i spillet har de samme øyene, så bakingen til slutt er en ren kopi
    # (pakket den på nytt, la den forkleet og kjolen oppå hverandre).
    head_density(lo, FOLK[name])
    pack_weighted(lo)
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
    nearest_colors(hi, lo, img, size)
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
    # Undersiden av en hjelmbrem eller hattebrem vender nedover og ses bare
    # på skrå forfra; der lå øyenbrynene fra bildet.
    w = mul(w, ramp(nrm.outputs["Z"], FACE_DOWN[0], FACE_DOWN[1]))
    # Bare der formen og bildet er enige i grove trekk: TRELLIS kan sette en
    # hjelmbrem lavere enn bildet, og da lå øyenbrynene på bremmen. Fargene
    # sammenlignes uskarpt (`FACE_BLUR`), så øynene TRELLIS mistet står.
    agree = face_agreement(ob, name, im, sc, cu, top, H)
    at = N.new("ShaderNodeAttribute")
    at.attribute_name = "face_agree"
    w = mul(w, at.outputs["Fac"])
    mix = N.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    L.new(w, mix.inputs["Factor"])
    L.new(base, mix.inputs["A"])
    L.new(t.outputs["Color"], mix.inputs["B"])
    L.new(mix.outputs["Result"], bsdf.inputs["Base Color"])


def face_agreement(ob, name, im, sc, cu, top, H):
    """Per hjørne: 0 der bildet forfra har farge og formen er grå etter
    uskarping (`FACE_AGREE`: hvor mye mer metning bildet har), ellers 1,
    lagret som «face_agree». Fargeavstand skilte ikke en hjelmbrem fra
    huden: hudtonen fra TRELLIS er like langt fra arket som metallet er."""
    from scipy import ndimage
    me = ob.data
    src = bpy.data.images[f"src_{name}"]
    S = src.size[0]
    px = np.empty(S * S * 4, np.float32)
    src.pixels.foreach_get(px)
    px = px.reshape(S, S, 4)[..., :3]
    r = FACE_BLUR * S
    sb = np.stack([ndimage.gaussian_filter(px[..., c], r) for c in range(3)], -1)
    ib = np.stack([ndimage.gaussian_filter(im[..., c] / 255.0, FACE_BLUR * im.shape[1] * 2) for c in range(3)], -1)
    # Begge er sRGB (bildet fra bakingen er et 8-biters bilde).
    uvl = me.uv_layers["src"].data
    vuv = np.zeros((len(me.vertices), 2))
    for poly in me.polygons:
        for li in poly.loop_indices:
            vuv[me.loops[li].vertex_index] = uvl[li].uv
    co = np.array([v.co[:] for v in me.vertices])
    a = sb[np.clip((vuv[:, 1] * S).astype(int), 0, S - 1), np.clip((vuv[:, 0] * S).astype(int), 0, S - 1)]
    W, Hh = im.shape[1], im.shape[0]
    u = np.clip((cu + co[:, 1] * sc).astype(int), 0, W - 1)
    v = np.clip((top + (H - co[:, 2]) * sc).astype(int), 0, Hh - 1)
    b = ib[v, u]
    def sat(c):
        return c.max(1) - c.min(1)

    # Bildet har farge (hud) der formen er grå (metall): da ligger noe annet
    # fra formen foran det bildet viser.
    d = sat(b) - sat(a)
    face = (co[:, 2] > FACE_FROM * H) & (co[:, 0] > np.median(co[co[:, 2] > FACE_FROM * H, 0]))
    print("metning bildet minus formen i ansiktet", np.percentile(d[face], [10, 25, 50, 75, 90, 95]).round(3))
    f = np.clip((FACE_AGREE[1] - d) / (FACE_AGREE[1] - FACE_AGREE[0]), 0.0, 1.0)
    at = me.attributes.new("face_agree", "FLOAT", "POINT")
    at.data.foreach_set("value", f.astype(np.float32))
    return f


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
    layered_weights(ob, arm)
    return arm


def layered_weights(ob, arm, radius=WEIGHT_RADIUS):
    """Lag som ligger oppå hverandre beveger seg sammen: vektene på kroppen
    jevnes ut i rommet (ikke langs flaten) innen `radius`. Skallet har
    kjolen som et eget lag bak forkleet, og varmevektingen ga lagene ulike
    bein, så kjolen gikk gjennom forkleet når beina beveget seg (fiskerkona,
    16. oktober). Armene og hendene står utenfor, så de ikke henger fast i
    skjørtet de ligger inntil."""
    from mathutils.kdtree import KDTree

    names = [b.name for b in arm.data.bones]
    idx = {g.index: names.index(g.name) for g in ob.vertex_groups if g.name in names}
    nv, nb = len(ob.data.vertices), len(names)
    W = np.zeros((nv, nb))
    for v in ob.data.vertices:
        for g in v.groups:
            if g.group in idx:
                W[v.index, idx[g.group]] = g.weight
    W /= np.maximum(W.sum(1, keepdims=True), 1e-9)
    limb = np.array([any(k in n for k in LIMB_BONES) for n in names])
    body = W[:, limb].sum(1) < 0.5
    co = np.array([v.co[:] for v in ob.data.vertices])
    kd = KDTree(int(body.sum()))
    ids = np.nonzero(body)[0]
    for i in ids:
        kd.insert(co[i], int(i))
    kd.balance()
    out = W.copy()
    for i in ids:
        near = kd.find_range(co[i], radius)
        j = np.array([n[1] for n in near])
        d = np.array([n[2] for n in near])
        k = np.exp(-((d / (radius * 0.5)) ** 2))
        out[i] = (W[j] * k[:, None]).sum(0) / k.sum()
    for b, n in enumerate(names):
        g = ob.vertex_groups.get(n) or ob.vertex_groups.new(name=n)
        for i in ids:
            if out[i, b] > 1e-3:
                g.add([int(i)], float(out[i, b]), "REPLACE")
            else:
                g.remove([int(i)])
    print("vekter jevnet ut på", len(ids), "av", nv, "hjørner")


def build(name, size=2048):
    clear()
    ob = load(name)
    hi = ob.copy()
    hi.data = ob.data.copy()
    bpy.context.scene.collection.objects.link(hi)
    simplify(ob)
    transfer(hi, ob, name, SRC_SIZE)
    face_patch(ob, name, FOLK[name])
    H = FOLK[name]
    J = landmarks(ob, H)
    arm = rig(ob, name, J)
    style, grips = folk_motion.FIGURES.get(name, ("default", {}))
    st = dict(folk_motion.STYLES.get(style, {}))
    if style == "old":
        # «old» bøyer knærne, ryggen og albuene; med kappe og formen fra
        # TRELLIS (som står rett) så det ut som de segnet. Bare takten står.
        st = {k: v for k, v in st.items() if k in OLD_KEEP}
    # Formen fra TRELLIS har holdningen fra arket; stilene ble laget for
    # figurer som sto rett, og krokryggen ble dobbel (apotekeren).
    for k, f in POSTURE.items():
        if k in st:
            st[k] *= f
    out = max(math.degrees(math.atan2(abs(J[f"wrist.{s}"].y - J[f"shoulder.{s}"].y), J[f"shoulder.{s}"].z - J[f"wrist.{s}"].z)) for s in ("L", "R"))
    st["arm_out"] = folk_motion.style_of(style)["arm_out"] * 0.5 - out * 0.65
    folk_motion.clips(arm, st, {}, only=("idle", "walk", "talk", "wave"))
    bake_textured(f"{OUT}/npc_{name}.glb", size, keep=("glass", "magic"), armature=arm, uv_angle=None, ao=False, pack=False)
    return J


if __name__ == "__main__":
    args = sys.argv[1:]
    names = [a for a in args if a in FOLK]
    size = next((int(a) for a in args if a.isdigit()), 2048)
    for n in names:
        build(n, size)
