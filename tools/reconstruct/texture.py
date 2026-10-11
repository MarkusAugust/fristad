"""Desimering, uv, baking av posisjon/normal/normalkart, og projeksjon av forbildet tilbake på formen."""
import json
import os

import numpy as np

from . import job as J
from .fuse import sample, to_img



def w2b(v):
    """Verden (x mot venstre side, y opp, z fram, piksler) -> Blender (meter, -Y fram, Z opp)."""
    v = np.asarray(v, float)
    return np.stack([v[..., 0], -v[..., 2], v[..., 1]], -1) * J.SCALE


def b2w(v):
    v = np.asarray(v, float) / J.SCALE
    return np.stack([v[..., 0], v[..., 2], -v[..., 1]], -1)


def reset():
    import bpy
    bpy.ops.wm.read_factory_settings(use_empty=True)


def mesh_obj(name, v, f):
    import bpy
    me = bpy.data.meshes.new(name)
    me.from_pydata(w2b(v).tolist(), [], f[:, ::-1].tolist() if False else f.tolist())
    me.update()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def select(*obs, active=None):
    import bpy
    for o in bpy.context.scene.objects:
        o.select_set(False)
    for o in obs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = active or obs[0]


def cut_neck(v, f):
    """Kutt halsen rent med et skrått plan (følger kuttet i profilene) og behold største bit."""
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components
    n = len(v)
    A = coo_matrix((np.ones(len(f) * 3), (f.ravel(), np.roll(f, 1, 1).ravel())), shape=(n, n))
    _, lab = connected_components(A, directed=False)
    big = np.bincount(lab).argmax()
    keep = lab[f[:, 0]] == big
    return f[keep]


def bake(target_tris=None):
    target_tris = target_tris or J.TRIS
    import bpy
    import cv2
    from scipy import ndimage as ndi
    d = np.load(J.p("mesh.npz"))
    v, f = d["v"].astype(float), d["f"]
    f = cut_neck(v, f)
    # marching cubes gir flater med normal inn i feltet (feltet er positivt inni): snu
    f = f[:, [0, 2, 1]]
    reset()
    sc = bpy.context.scene
    hi = mesh_obj("hi", v, f)
    bpy.ops.object.select_all(action="DESELECT")
    select(hi)
    bpy.ops.object.shade_smooth()
    lo = hi.copy(); lo.data = hi.data.copy(); lo.name = "head"
    sc.collection.objects.link(lo)
    select(lo)
    m = lo.modifiers.new("dec", "DECIMATE")
    m.ratio = target_tris / len(f)
    bpy.ops.object.modifier_apply(modifier="dec")
    m = lo.modifiers.new("sm", "SMOOTH"); m.factor = 0.5; m.iterations = 2
    bpy.ops.object.modifier_apply(modifier="sm")
    print("lav", len(lo.data.polygons), "flater")
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=0.70, island_margin=0.004, area_weight=0.0)
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.ops.object.shade_smooth()

    # bak posisjon og glatt normal (lav modell) og normalkart (høy -> lav) med Cycles
    sc.render.engine = "CYCLES"
    sc.cycles.device = "CPU"
    sc.cycles.samples = 1
    sc.render.bake.margin = 8

    def img(name, float_=True):
        im = bpy.data.images.new(name, J.TEX, J.TEX, alpha=False, float_buffer=float_)
        im.colorspace_settings.name = "Non-Color"
        return im

    mat = bpy.data.materials.new("head")
    mat.use_nodes = True
    nt = mat.node_tree
    lo.data.materials.append(mat)
    tex = nt.nodes.new("ShaderNodeTexImage")
    nt.nodes.active = tex
    geo = nt.nodes.new("ShaderNodeNewGeometry")
    emit = nt.nodes.new("ShaderNodeEmission")
    out = nt.nodes["Material Output"]
    nt.links.new(emit.outputs[0], out.inputs[0])
    maps = {}
    for key, sock in (("pos", "Position"), ("nrm", "Normal")):
        im = img(key)
        tex.image = im
        nt.links.new(geo.outputs[sock], emit.inputs[0])
        select(lo)
        bpy.ops.object.bake(type="EMIT", margin=8)
        a = np.empty(J.TEX * J.TEX * 4, np.float32)
        im.pixels.foreach_get(a)
        maps[key] = a.reshape(J.TEX, J.TEX, 4)[:, :, :3].copy()
    # dekning: hvilke tekseler ligger i en uv-øy (bak en hvit emisjon uten marg)
    im = img("cov")
    tex.image = im
    rgb = nt.nodes.new("ShaderNodeRGB"); rgb.outputs[0].default_value = (1, 1, 1, 1)
    nt.links.new(rgb.outputs[0], emit.inputs[0])
    bpy.ops.object.bake(type="EMIT", margin=0)
    a = np.empty(J.TEX * J.TEX * 4, np.float32); im.pixels.foreach_get(a)
    cov = a.reshape(J.TEX, J.TEX, 4)[:, :, 0] > 0.5
    # normalkart fra den høye modellen
    nrm_im = img("normal", float_=False)
    tex.image = nrm_im
    select(hi, lo, active=lo)
    sc.render.bake.use_selected_to_active = True
    sc.render.bake.cage_extrusion = 0.004
    sc.render.bake.max_ray_distance = 0.01
    bpy.ops.object.bake(type="NORMAL", normal_space="TANGENT", margin=8)
    sc.render.bake.use_selected_to_active = False
    nrm_im.filepath_raw = J.p("head_normal.png"); nrm_im.file_format = "PNG"; nrm_im.save()
    np.savez(J.p("texels.npz"), pos=maps["pos"], nrm=maps["nrm"], cov=cov)
    # høy modell til synlighet
    hv = np.empty(len(hi.data.vertices) * 3, np.float32); hi.data.vertices.foreach_get("co", hv)
    np.save(J.p("hi_verts.npy"), hv.reshape(-1, 3))
    bpy.data.objects.remove(hi)
    bpy.ops.wm.save_as_mainfile(filepath=J.p("head.blend"))


def project():
    """Farg hver teksel fra visningene: vekt etter normal mot kameraet, dybdetest, mykt mot silhuetten,
    to frekvensbånd (lave frekvenser blandes bredt, detaljene nesten bare fra den beste visningen)."""
    import cv2
    from scipy import ndimage as ndi
    fr = json.load(open(J.p("frame.json")))["frame"]
    t = np.load(J.p("texels.npz"))
    cov = t["cov"]
    P = b2w(t["pos"][cov])
    N = t["nrm"][cov]
    N = np.stack([N[:, 0], N[:, 2], -N[:, 1]], -1)  # Blender -> verden
    N /= np.linalg.norm(N, axis=1, keepdims=True) + 1e-9
    HV = b2w(np.load(J.p("hi_verts.npy")))
    lo_acc = np.zeros((len(P), 3)); hi_acc = np.zeros((len(P), 3))
    wl_acc = np.zeros(len(P)); wh_acc = np.zeros(len(P))
    best = np.zeros(len(P))
    for k, v in J.VIEWS.items():
        if v.get("geo_only") and not J.OPT.get("mirror_texture"):
            continue
        rgb = cv2.imread(J.p(f"{k}_rgb.png"))[:, :, ::-1].astype(np.float32) / 255
        lin = rgb ** 2.2
        low = cv2.GaussianBlur(lin, (0, 0), 5)
        high = lin - low
        m = cv2.imread(J.p(f"{k}_mask.png"), 0) > 127
        edge = np.clip(ndi.distance_transform_edt(m) / float(J.OPT.get("edge_fade", 10.0)), 0, 1).astype(np.float32)
        cam = np.array(v["cam"], float)
        # dybdebuffer fra den høye modellen
        u, vv = to_img(k, fr, HV[:, 0], HV[:, 1], HV[:, 2])
        tt = HV @ cam
        zb = np.full((J.TH, J.TW), -1e9, np.float32)
        ui = np.clip(np.round(u).astype(int), 0, J.TW - 1); vi = np.clip(np.round(vv).astype(int), 0, J.TH - 1)
        np.maximum.at(zb, (vi, ui), tt.astype(np.float32))
        zb = ndi.maximum_filter(zb, 5)
        u, vv = to_img(k, fr, P[:, 0], P[:, 1], P[:, 2])
        tp = P @ cam
        zs = sample(zb, u, vv, fill=-1e9)
        vis = np.clip(1 - (zs - tp - 4) / 6, 0, 1)  # myk dybdetest (piksler)
        # flater som vender opp (toppen) eller ned ses av ingen: vekt etter retningen i det vannrette planet
        Nh = N * np.array([1, 0.35, 1]); Nh /= np.linalg.norm(Nh, axis=1, keepdims=True) + 1e-9
        ndc = np.clip(Nh @ cam, 0, 1)
        e = sample(edge, u, vv, fill=0)
        base = vis * e
        wl = base * ndc ** (8 if J.OPT.get("detail") == "winner" else 2)
        wh = base * ndc ** 8
        cl = np.stack([sample(low[:, :, c], u, vv) for c in range(3)], -1)
        ch = np.stack([sample(high[:, :, c], u, vv) for c in range(3)], -1)
        lo_acc += wl[:, None] * cl; wl_acc += wl
        if J.OPT.get("detail") == "winner":
            # detaljene bare fra visningen som ser flaten best (myk overgang over et smalt bånd), så linjene
            # (øyne, bryn) aldri tegnes to ganger der visningene ikke stemmer helt
            wh = base * ndc ** 40
        hi_acc += wh[:, None] * ch; wh_acc += wh
        print(k, "synlig andel %.2f" % (vis > 0.5).mean())
    col = lo_acc / np.maximum(wl_acc, 1e-6)[:, None] + hi_acc / np.maximum(wh_acc, 1e-6)[:, None]
    seen = wl_acc > 1e-3
    out = np.zeros((J.TEX, J.TEX, 3), np.float32)
    have = np.zeros((J.TEX, J.TEX), bool)
    idx = np.nonzero(cov)
    out[idx[0][seen], idx[1][seen]] = col[seen]
    have[idx[0][seen], idx[1][seen]] = True
    print("dekket %.3f" % seen.mean())
    # fyll hull og marg med push-pull (uskarp utfylling fra naboene)
    out = pushpull(out, have)
    srgb = np.clip(out, 0, 1) ** (1 / 2.2)
    # Blender-bilder er nedenfra og opp
    cv2.imwrite(J.p("head_albedo.png"), (srgb[::-1, :, ::-1] * 255 + 0.5).astype(np.uint8))


def pushpull(img, have):
    import cv2
    if have.all() or min(img.shape[:2]) < 2:
        return img
    w = have.astype(np.float32)
    small_w = cv2.resize(w, (img.shape[1] // 2, img.shape[0] // 2), interpolation=cv2.INTER_AREA)
    small = cv2.resize(img * w[..., None], (img.shape[1] // 2, img.shape[0] // 2), interpolation=cv2.INTER_AREA)
    sh = small_w > 1e-4
    small = np.where(sh[..., None], small / np.maximum(small_w, 1e-4)[..., None], 0)
    small = pushpull(small, sh)
    up = cv2.resize(small, (img.shape[1], img.shape[0]), interpolation=cv2.INTER_LINEAR)
    return np.where(have[..., None], img, up)


