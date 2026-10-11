"""Nettet: største bit, desimering til `tris` trekanter, glatting og uv-kart.

Desimeringen og Smart UV gjøres i Blender som Python-modul (bpy) fordi det tar sekunder og er robust; alt som er
tungt (teksturen) gjøres i numpy. Resultatet er `lo.npz` (hjørner, flater, uv per flatehjørne) og `hi.npy`
(hjørnene i den høye modellen, til z-bufferen). Et `template`-mål kan hoppe over dette steget helt: malen har
allerede uv-kart.
"""
import time

import numpy as np


def largest_component(v, f):
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components
    n = len(v)
    A = coo_matrix((np.ones(len(f) * 3), (f.ravel(), np.roll(f, 1, 1).ravel())), shape=(n, n))
    k, lab = connected_components(A, directed=False)
    big = np.bincount(lab).argmax()
    return f[lab[f[:, 0]] == big], k


def components(v, f):
    """Antall sammenhengende biter blant flatene (ubrukte hjørner teller ikke)."""
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components
    n = len(v)
    A = coo_matrix((np.ones(len(f) * 3), (f.ravel(), np.roll(f, 1, 1).ravel())), shape=(n, n))
    _, lab = connected_components(A, directed=False)
    return len(np.unique(lab[np.unique(f)]))


def boundary_loops(f):
    """Antall hull (lukkede kantløkker uten nabo)."""
    e = np.sort(np.concatenate([f[:, [0, 1]], f[:, [1, 2]], f[:, [2, 0]]]), 1)
    key = e[:, 0].astype(np.int64) * (f.max() + 1) + e[:, 1]
    uniq, cnt = np.unique(key, return_counts=True)
    bnd = uniq[cnt == 1]
    if len(bnd) == 0:
        return 0
    a = bnd // (f.max() + 1); b = bnd % (f.max() + 1)
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components
    n = f.max() + 1
    A = coo_matrix((np.ones(len(a)), (a, b)), shape=(n, n))
    k, lab = connected_components(A, directed=False)
    used = np.unique(np.concatenate([a, b]))
    return len(np.unique(lab[used]))


def bpy_reset():
    import bpy
    bpy.ops.wm.read_factory_settings(use_empty=True)


def bpy_mesh(name, v, f, uv=None):
    """Et Blender-objekt fra hjørner (Blender-koordinater) og flater, valgfritt med uv per flatehjørne."""
    import bpy
    me = bpy.data.meshes.new(name)
    me.from_pydata(np.asarray(v, float).tolist(), [], np.asarray(f).tolist())
    me.update()
    if uv is not None:
        layer = me.uv_layers.new(name="UVMap")
        layer.data.foreach_set("uv", np.asarray(uv, np.float32).ravel())
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def bpy_select(*obs, active=None):
    import bpy
    for o in bpy.context.scene.objects:
        o.select_set(False)
    for o in obs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = active or obs[0]


def bpy_read(ob):
    """Hjørner, trekantflater og uv per flatehjørne fra et Blender-objekt (trianguleres)."""
    import bpy
    bpy_select(ob)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.quads_convert_to_tris()
    bpy.ops.object.mode_set(mode="OBJECT")
    me = ob.data
    v = np.empty(len(me.vertices) * 3, np.float32); me.vertices.foreach_get("co", v)
    f = np.empty(len(me.polygons) * 3, np.int32); me.polygons.foreach_get("vertices", f)
    uv = None
    if me.uv_layers:
        uv = np.empty(len(me.loops) * 2, np.float32); me.uv_layers.active.data.foreach_get("uv", uv)
        uv = uv.reshape(-1, 3, 2)
    return v.reshape(-1, 3), f.reshape(-1, 3), uv


def decimate_uv(v, f, tris, angle=0.70, margin=0.004, smooth=2):
    """Desimer til `tris` trekanter, glatt litt, Smart UV. Inn og ut i samme koordinater."""
    import bpy
    bpy_reset()
    ob = bpy_mesh("lo", v, f)
    bpy_select(ob)
    bpy.ops.object.shade_smooth()
    if len(f) > tris:
        m = ob.modifiers.new("dec", "DECIMATE"); m.ratio = tris / len(f)
        bpy.ops.object.modifier_apply(modifier="dec")
    if smooth:
        m = ob.modifiers.new("sm", "SMOOTH"); m.factor = 0.5; m.iterations = smooth
        bpy.ops.object.modifier_apply(modifier="sm")
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=angle, island_margin=margin, area_weight=0.0)
    bpy.ops.object.mode_set(mode="OBJECT")
    return bpy_read(ob)


def decimate(v, f, tris):
    """Desimer til `tris` trekanter (bpy), uten uv. Inn og ut i samme koordinater; bare brukte hjørner."""
    import bpy
    if len(f) <= tris:
        return v, f
    bpy_reset()
    ob = bpy_mesh("dec", v, f)
    bpy_select(ob)
    m = ob.modifiers.new("dec", "DECIMATE"); m.ratio = tris / len(f)
    bpy.ops.object.modifier_apply(modifier="dec")
    v2, f2, _ = bpy_read(ob)
    return v2.astype(float), f2


def build(job, tris=None):
    """Fra `mesh.npz` (marching cubes) til `lo.npz` med uv og `hi.npy`."""
    t0 = time.time()
    d = np.load(job.p("mesh.npz"))
    v, f = d["v"].astype(float), d["f"]
    f, ncomp = largest_component(v, f)
    lv, lf, uv = decimate_uv(v, f, tris or job.tris)
    np.savez(job.p("lo.npz"), v=lv.astype(np.float32), f=lf.astype(np.int32), uv=uv.astype(np.float32))
    used = np.unique(f)
    np.save(job.p("hi.npy"), v[used].astype(np.float32))
    from .texture import vertex_normals
    np.save(job.p("hi_n.npy"), vertex_normals(v.astype(np.float32), f)[used].astype(np.float32))
    print("lav modell: %d trekanter (høy: %d, %d løse biter kastet)" % (len(lf), len(f), ncomp - 1))
    job.time("mesh", time.time() - t0)
    return lv, lf, uv
