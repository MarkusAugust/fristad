"""Gjenstander i spillet: plassering (dreining mot spillets front, høyde i meter, origo på bakken) og deling i to
modeller om et plan (kista: kassa og lokket med origo i hengslet).

`opt` i oppslaget:
- `game_yaw`: grader om loddrett akse før eksport (90 dreier +x-flaten, den tegneren kaller forsiden, til +z).
- `height`: modellens høyde i meter (skalerer alt likt); `ground: false` lar origo stå i midten (standard: på bunnen).
- `split`: `{"at": "lid" | <y i meter over bunnen>, "names": ["chest", "chest_lid"], "hinge": [0, 0.46, -0.28]}`:
  deles om planet y = at (`lid` er priorens kropphøyde); med `hinge` skaleres høyden og dybden så planet og
  bakkanten treffer hengslet, og den øvre delen får origo i hengslet. Snittflatene fylles og får den mørkeste
  tekselen i teksturen.
"""
import json

import numpy as np


def place(job, v, fr=None, ground=None):
    """Verdenspiksler -> verdenspiksler dreid, skalert og satt på bakken. Gir (v, info)."""
    v = np.asarray(v, np.float64).copy()
    info = {}
    yaw = float(job.opt.get("game_yaw", 0.0))
    if yaw:
        t = np.radians(yaw)
        x, z = v[:, 0].copy(), v[:, 2].copy()
        v[:, 0] = x * np.cos(t) - z * np.sin(t)
        v[:, 2] = x * np.sin(t) + z * np.cos(t)
    h = job.opt.get("height")
    if h:
        f = float(job.opt.get("_scale_factor") or float(h) / (np.ptp(v[:, 1]) * job.scale))
        job.opt["_scale_factor"] = f  # samme faktor for de egne modellene etterpå
        v *= f
        info["scale_factor"] = round(f, 4)
    if job.opt.get("ground", True) if ground is None else ground:
        info["ground_shift"] = float(-v[:, 1].min())
        info["ground_shift_m"] = round(float(-v[:, 1].min()) * job.scale, 4)
        v[:, 1] -= v[:, 1].min()
    info["size_m"] = [round(float(x) * job.scale, 3) for x in np.ptp(v, axis=0)]
    return v, info


def darkest_uv(job, albedo="albedo.png"):
    import cv2
    im = cv2.imread(job.p(albedo), 0).astype(np.float32)
    im = cv2.GaussianBlur(im, (0, 0), 6)
    T = im.shape[0]
    y, x = np.unravel_index(int(np.argmin(im[8:-8, 8:-8])), im[8:-8, 8:-8].shape)
    return ((x + 8 + 0.5) / T, 1 - (y + 8 + 0.5) / T)


def cut(v, f, uv, y0, keep_below, cap_uv):
    """Del nettet om planet y = y0 (bmesh bisect) og fyll snittet; snittflatene får uv `cap_uv`."""
    import bmesh
    from .mesh import bpy_mesh, bpy_read, bpy_reset
    bpy_reset()
    ob = bpy_mesh("cut", v, f, uv)
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    uvl = bm.loops.layers.uv.active
    r = bmesh.ops.bisect_plane(bm, geom=bm.verts[:] + bm.edges[:] + bm.faces[:], plane_co=(0, float(y0), 0),
                               plane_no=(0, 1, 0), clear_outer=keep_below, clear_inner=not keep_below)
    edges = [g for g in r["geom_cut"] if isinstance(g, bmesh.types.BMEdge)]
    res = bmesh.ops.triangle_fill(bm, use_beauty=True, use_dissolve=False, edges=edges)
    caps = [g for g in res["geom"] if isinstance(g, bmesh.types.BMFace)]
    for face in caps:
        for lp in face.loops:
            lp[uvl].uv = cap_uv
    # snittflaten vender ut av delen: opp på kassa, ned på lokket
    want = 1.0 if keep_below else -1.0
    wrong = [f_ for f_ in caps if f_.normal.y * want < 0]
    if wrong:
        bmesh.ops.reverse_faces(bm, faces=wrong)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    bm.to_mesh(ob.data)
    bm.free()
    v2, f2, uv2 = bpy_read(ob)
    # bare brukte hjørner
    used = np.unique(f2)
    remap = np.full(len(v2), -1, np.int64); remap[used] = np.arange(len(used))
    return v2[used], remap[f2], uv2


def split(job, v, f, uv, spec, prior=None, part=None):
    """Gir [(navn, v, f, uv), …] i verdenspiksler (plassert), den øvre delen med origo i hengslet."""
    at = spec.get("at", "lid")
    ymin = v[:, 1].min()
    if part is not None and (part == 1).any():  # delene er alt hver sin del i nettet (shapes.py): ingen kutting
        names = spec.get("names") or [job.name, job.name + "_lid"]
        y0 = v[f[part == 1]][:, :, 1].min()
        hinge = spec.get("hinge")
        if hinge:  # likt skalert så kassa når opp til hengslet, og skjøvet så bakkanten står i hengslets z
            sy = hinge[1] / ((y0 - ymin) * job.scale)
            v = v * sy
            y0 = ymin * sy + hinge[1] / job.scale
            dz = hinge[2] / job.scale - v[:, 2].min()
            v[:, 2] += dz
            print("split: skalert %.3f og skjøvet %.1f px i z for hengslet %s" % (sy, dz, hinge))
        out = []
        for i, name in enumerate(names[:2]):
            sel = part == i
            fi = f[sel]
            used = np.unique(fi)
            remap = np.full(len(v), -1, np.int64); remap[used] = np.arange(len(used))
            vi = v[used].copy()
            if i == 1:
                zback = vi[:, 2].min()
                vi[:, 1] -= y0; vi[:, 2] -= zback
            out.append((name, vi, remap[fi], uv[sel]))
        info = dict(plane_m=round(float(y0 - v[:, 1].min()) * job.scale, 3), body_tris=int(len(out[0][2])), lid_tris=int(len(out[1][2])))
        print("split:", json.dumps(info))
        return out, info
    if at == "lid":
        if not prior or "hy" not in prior.get("params", {}):
            raise SystemExit("split 'lid' trenger prior chest")
        # kroppen går fra −hy til hy om priorens nullpunkt (origo før place(): skalert med _factor, løftet _shift)
        y0 = prior["params"]["hy"] * float(spec.get("_factor", 1.0)) + float(spec.get("_shift", 0.0))
    else:
        y0 = ymin + float(at) / job.scale
    names = spec.get("names") or [job.name, job.name + "_lid"]
    hinge = spec.get("hinge")
    if hinge:
        sy = hinge[1] / ((y0 - ymin) * job.scale)
        zmin = v[:, 2].min()
        sxz = abs(hinge[2]) / (abs(zmin) * job.scale)
        v = v * np.array([sxz, sy, sxz])
        y0 = ymin * sy + hinge[1] / job.scale
        print("split: skalert y %.3f, xz %.3f for hengslet %s" % (sy, sxz, hinge))
    cap = darkest_uv(job)
    body = cut(v, f, uv, y0, True, cap)
    lid = cut(v, f, uv, y0, False, cap)
    lv = lid[0].copy()
    zback = lv[:, 2].min()
    lv[:, 1] -= y0
    lv[:, 2] -= zback
    info = dict(plane_m=round(float(y0 - v[:, 1].min()) * job.scale, 3), hinge_m=[0.0, round(float(y0 - v[:, 1].min()) * job.scale, 3), round(float(zback) * job.scale, 3)],
                body_tris=int(len(body[1])), lid_tris=int(len(lid[1])))
    print("split:", json.dumps(info))
    return [(names[0], body[0], body[1], body[2]), (names[1], lv, lid[1], lid[2])], info
