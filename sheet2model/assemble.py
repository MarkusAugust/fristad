"""Steg 6: sett sammen en hel figur: kroppen fra kroppsarket og hodet fra hodearket (mye høyere oppløsning),
plassert etter silhuettene, og redskapet i hånda skilt ut som egen del. Alt får ett uv-kart (hodet mer plass)
og teksturen projiseres én gang, rett fra kildearkene, inn i det (ingen ombaking).

Resultat: `assembled.npz` (v, f, uv, part, part_names, hipsrigid), `albedo_full.png`, `normal_full.png`,
`texels_full.npz`.
"""
import json
import os
import time

import numpy as np

from . import geom, texture
from .sheet import hsv, load_mask, load_rgb, skin_mask, tool_boxes


# ---------------------------------------------------------------- hodet på kroppen
def _widths(m, o, rows):
    out = []
    for y in rows:
        v = int(round(o["top"] + y * o["sy"]))
        c = np.nonzero(m[min(max(v, 0), m.shape[0] - 1)])[0]
        out.append((c.max() - c.min()) / o["sy"] if len(c) else np.nan)
    return np.array(out)


def head_fit(body, head, fb, fh):
    """Skala (kroppspiksler per hodepiksel) og forskyvning i z, funnet ved å passe hodets silhuett forfra og fra
    siden til kroppsarket over øyehøyde (skallen: ingen skjegg, ingen ører)."""
    ys = np.arange(10, 200, 4.0)  # hodepiksler fra toppen: issen til brynene
    best = []
    for k in ("front", "right"):
        wh = _widths(load_mask(head, k), fh[k], ys)
        mb = load_mask(body, k)
        errs = []
        for s in np.arange(0.2, 1.6, 0.002):
            wb = _widths(mb, fb[k], ys * s)
            errs.append((np.nanmean((wb - s * wh) ** 2), s))
        best.append(min(errs)[1])
    s = float(np.mean(best))

    def zmid(job, o, rows):
        m = load_mask(job, "right")
        zs = []
        for y in rows:
            v = int(round(o["top"] + y * o["sy"]))
            c = np.nonzero(m[min(max(v, 0), m.shape[0] - 1)])[0]
            if len(c):
                zs.append(((c.min() + c.max()) / 2 - o["c"]) / o["sy"])
        return float(np.mean(zs))
    dz = zmid(body, fb["right"], ys[:20] * s) - s * zmid(head, fh["right"], ys[:20])
    print("hodet: skala %.3f (forfra %.3f, profil %.3f), dz %.1f" % (s, best[0], best[1], dz))
    return s, dz, best


# ---------------------------------------------------------------- hjelpere på nett
def face_centres(v, f):
    return v[f].mean(1)


def face_normals(v, f):
    n = np.cross(v[f[:, 1]] - v[f[:, 0]], v[f[:, 2]] - v[f[:, 0]])
    return n / (np.linalg.norm(n, axis=1, keepdims=True) + 1e-12)


def boundary_loops_of(f, nv):
    """Kantløkkene (kanter som bare én flate bruker), som lister av rettede kanter (a, b) per løkke."""
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components
    e = np.concatenate([f[:, [0, 1]], f[:, [1, 2]], f[:, [2, 0]]])
    key = np.minimum(e[:, 0], e[:, 1]).astype(np.int64) * nv + np.maximum(e[:, 0], e[:, 1])
    uniq, inv, cnt = np.unique(key, return_inverse=True, return_counts=True)
    bnd = e[cnt[inv] == 1]
    if len(bnd) == 0:
        return []
    A = coo_matrix((np.ones(len(bnd)), (bnd[:, 0], bnd[:, 1])), shape=(nv, nv))
    _, lab = connected_components(A, directed=False)
    loops = {}
    for a_, b_ in bnd:
        loops.setdefault(lab[a_], []).append((a_, b_))
    return list(loops.values())


def fill_holes(v, f, uv=None, max_edges=None):
    """Lukk kantløkkene med en vifte fra løkkas midtpunkt (robust også når kanten ikke er en ren løkke).
    Løkker med flere kanter enn `max_edges` står åpne. Nye flater får uv (0, 0)."""
    loops = boundary_loops_of(f, len(v))
    newv, newf = [], []
    for loop in loops:
        if len(loop) < 3 or (max_edges and len(loop) > max_edges):
            continue
        idx = np.unique(np.array(loop).ravel())
        c = len(v) + len(newv)
        newv.append(v[idx].mean(0))
        for a_, b_ in loop:
            newf.append((b_, a_, c))  # motsatt retning av flaten som eier kanten: normalen peker ut
    if not newf:
        return v, f, uv, 0
    v2 = np.concatenate([v, np.array(newv, v.dtype)])
    f2 = np.concatenate([f, np.array(newf, f.dtype)])
    if uv is not None:
        uv = np.concatenate([uv, np.zeros((len(newf), 3, 2), np.float32)])
    return v2, f2, uv, len(newf)


def drop_flakes(v, f, keep_frac=0.02):
    """Fjern løse biter (flak etter kuttene) som er mindre enn en liten andel av modellen."""
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components
    n = len(v)
    A = coo_matrix((np.ones(len(f) * 3), (f.ravel(), np.roll(f, 1, 1).ravel())), shape=(n, n))
    _, lab = connected_components(A, directed=False)
    fl = lab[f[:, 0]]
    sizes = np.bincount(fl)
    keep = sizes[fl] >= keep_frac * len(f)
    print("flak fjernet:", int((~keep).sum()), "flater i", int((sizes < keep_frac * len(f)).sum() - (sizes == 0).sum()), "biter")
    return f[keep]


def in_mask(job, k, o, W, m):
    u, v = geom.to_img(job.views[k], o, W[:, 0], W[:, 1], W[:, 2])
    return geom.nearest(m, u, v)


# ---------------------------------------------------------------- redskapet
def hammer_mask(job, rgb, m, past=190, width=12, column=False, hand=None, kind="iron", region=None, handle=True):
    """Redskapet i én visning: det største grå jernstykket (hammerhodet) og et bånd langs skaftet fra hodet
    gjennom hånda (nærmeste hudflekk) og litt forbi."""
    from scipy import ndimage as ndi
    h, s, v = hsv(rgb)
    if kind == "wood":  # trekrus o.l.: gulbrunt tre (kjolen og forkleet er rødere eller lysere)
        iron = (h > 22) & (h < 45) & (s > 70) & (s < 200) & (v > 60) & (v < 215) & m
    else:
        iron = (s < 40) & (v > 35) & (v < 140) & m
    if region is not None:
        iron &= region
    lab, n = ndi.label(ndi.binary_opening(iron, iterations=2))
    if n == 0:
        return np.zeros_like(m)
    sizes = ndi.sum(np.ones_like(lab), lab, range(1, n + 1))
    blob = lab == (1 + int(np.argmax(sizes)))
    cy, cx = ndi.center_of_mass(blob)
    skin = skin_mask(rgb) & m
    out = ndi.binary_dilation(blob, iterations=4 if handle else 9)
    if not handle:
        return out & m
    if column:  # forfra står skaftet loddrett bak neven, over hammerhodet
        ys, xs = np.nonzero(blob)
        col = np.zeros_like(m)
        col[max(0, ys.min() - 260): ys.max() + 1, max(0, xs.min() - 20): xs.max() + 21] = True
        return (out | col) & m
    sy_, sx_ = np.nonzero(ndi.binary_opening(skin, iterations=2))
    if len(sx_):
        i = int(np.argmin(np.hypot(sx_ - cx, sy_ - cy)))
        a = np.array([cx, cy]); hp = np.array([sx_[i], sy_[i]], float)
        u = (hp - a) / (np.linalg.norm(hp - a) + 1e-9)
        b = hp + u * past
        if hand is not None:
            hand.append(hp)
        yy, xx = np.mgrid[: m.shape[0], : m.shape[1]]
        ab = b - a
        t = np.clip(((xx - a[0]) * ab[0] + (yy - a[1]) * ab[1]) / (ab @ ab), 0, 1)
        d = np.hypot(xx - (a[0] + t * ab[0]), yy - (a[1] + t * ab[1]))
        out |= (d < width) & m
    return out


def tool_masks(job, fb, sg):
    tkw = dict(kind=job.opt.get("tool_kind", "iron"), handle=job.opt.get("tool_handle", True))
    masks = {}
    for kk in ("front", "right"):
        rgb = load_rgb(job, kk)
        mm = load_mask(job, kk)
        reg = np.ones_like(mm)
        if kk == "front":  # figurens høyre er til venstre i bildet forfra
            reg[:] = False
            c = int(fb["front"]["c"])
            if sg > 0:
                reg[:, :c] = True
            else:
                reg[:, c:] = True
        o_ = fb[kk]
        reg[: int(o_["top"] + 0.42 * (o_["bot"] - o_["top"]))] = False  # under brystet (ikke pels, spenner)
        masks[kk] = hammer_mask(job, rgb, mm, column=(kk == "front" and tkw["handle"]), region=reg, **tkw)
    boxes = tool_boxes(job)
    if boxes:  # redskapet i profilen: boksen minus huden
        from scipy import ndimage as ndi
        rgb = load_rgb(job, "right")
        skin = skin_mask(rgb)
        bx = np.zeros_like(masks["right"])
        for x0, y0, x1, y1 in boxes:
            bx[y0:y1, x0:x1] = True
        masks["right"] = ndi.binary_dilation(bx & ~ndi.binary_dilation(skin, iterations=3), iterations=4) & load_mask(job, "right")
        if job.opt.get("tool_strip") == "color" and os.path.exists(job.p("tool_src.npy")):
            masks["right"] = np.load(job.p("tool_src.npy")) & masks["right"]
    return masks, tkw


def tool_vertices(job, fb, W, masks, sg, H):
    inside = np.ones(len(W), bool)
    for kk, mm in masks.items():
        inside &= in_mask(job, kk, fb[kk], W, mm)
    inside &= (sg * W[:, 0] < -0.25 * np.ptp(W[:, 0]) / 2) & (W[:, 1] < -0.45 * H)
    return inside


def ghost_vertices(job, fb, W, masks, sg, H, tkw):
    """Den speilede profilen gir et «spøkelse» av redskapet i den andre hånda: hjørner i redskapsmasken fra
    siden på figurens andre side, utenfor neven, og alt i den hånda utenfor nevens dybde."""
    from scipy import ndimage as ndi
    hand = []
    rgb = load_rgb(job, "right")
    mr = load_mask(job, "right")
    preg = np.ones_like(mr)
    preg[: int(fb["right"]["top"] + 0.42 * (fb["right"]["bot"] - fb["right"]["top"]))] = False
    hammer_mask(job, rgb, mr, hand=hand, kind=tkw["kind"], region=preg)
    boxes = tool_boxes(job)
    if boxes:  # neven: hudpikselen nærmest midten av boksen
        b0 = boxes[0]
        sk = skin_mask(rgb) & mr
        sy_, sx_ = np.nonzero(ndi.binary_opening(sk, iterations=2))
        if len(sx_):
            cx, cy = (b0[0] + b0[2]) / 2, (b0[1] + b0[3]) / 2
            i = int(np.argmin(np.hypot(sx_ - cx, sy_ - cy)))
            hand = [np.array([sx_[i], sy_[i]], float)]
    o = fb["right"]
    fist_z = (hand[0][0] - o["c"]) / o["sy"] if hand else 0.0
    fist_y = -(hand[0][1] - o["top"]) / o["sy"] if hand else 0.0
    inr = in_mask(job, "right", o, W, masks["right"])
    mf = load_mask(job, "front")
    lf, rf = geom.run_bounds(mf)
    uf, vf = geom.to_img(job.views["front"], fb["front"], W[:, 0], W[:, 1], W[:, 2])
    c0 = int(round(fb["front"]["c"]))
    torso = (geom.nearest(lf, uf, vf) <= c0) & (geom.nearest(rf, uf, vf) >= c0) & geom.nearest(mf, uf, vf)
    xm = (sg * W[:, 0]).max()
    ghost = inr & (~torso | ((sg * W[:, 0] > 0.66 * xm) & (W[:, 1] > -0.7 * H))) \
        & (sg * W[:, 0] > 0.55 * xm) & (W[:, 1] < fist_y + 10) & (W[:, 1] < -0.45 * H) \
        & (np.hypot(W[:, 2] - fist_z, W[:, 1] - (fist_y - 30)) > 42)
    lefthand = (sg * W[:, 0] > 0.66 * xm) & (W[:, 1] < fist_y + 70) & (W[:, 1] > fist_y - 140)
    ghost |= lefthand & (np.abs(W[:, 2] - fist_z) > 48)
    return ghost


def prune_hand(job, fb, bv, bf, Jw, side):
    """Hånda uten redskap: alt som ligger lenger fra nevens akse enn nevens radius (målt forfra) i nevens høyde,
    er rester av det speilede redskapet (skaftstumpen). Radiusen er halve bredden av hånda i forfra-masken."""
    mf = load_mask(job, "front")
    o = fb["front"]
    w_ = np.asarray(Jw[f"wrist.{side}"], float); f_ = np.asarray(Jw[f"finger.{side}"], float)
    c = (w_ + f_) / 2
    v = int(round(o["top"] - c[1] * o["sy"])); u = int(round(o["c"] + c[0] * o["sy"]))
    row = mf[min(max(v, 0), mf.shape[0] - 1)]
    cols = np.nonzero(row)[0]
    if not len(cols):
        return bf, 0
    cut = np.nonzero(np.diff(cols) > 1)[0]
    starts = np.r_[cols[0], cols[cut + 1]]; ends = np.r_[cols[cut], cols[-1]]
    runs = [(a, b) for a, b in zip(starts, ends) if a - 8 <= u <= b + 8]
    if not runs:
        return bf, 0
    a, b = min(runs, key=lambda t: t[1] - t[0])
    r = 0.5 * (b - a) / o["sy"] * 1.3 + 6
    cx = ((a + b) / 2 - o["c"]) / o["sy"]
    cen = face_centres(bv, bf)
    win = (cen[:, 1] > c[1] - 70) & (cen[:, 1] < c[1] + 40) & (np.sign(cen[:, 0]) == np.sign(c[0])) & (np.abs(cen[:, 0]) > 0.5 * abs(c[0]))
    far = win & (np.hypot(cen[:, 0] - cx, cen[:, 2] - c[2]) > r)
    print("hånd %s: %d flater utenfor nevens radius %.0f px fjernet" % (side, int(far.sum()), r))
    return bf[~far], int(far.sum())


def model_tool(tool_v, tool_f, Jw, scale):
    """Bytt den rekonstruerte hammeren (flat, fra to tynne silhuetter) med en enkel modell med volum: et rundt
    skaft og et jernhode med fas, lagt langs hovedaksen til det rekonstruerte redskapet. Gir (v, f, materiale
    per flate: 0 jern, 1 tre) i verdenspiksler."""
    import bpy  # noqa: F401 (bmesh finnes først etter bpy)
    import bmesh
    from mathutils import Matrix, Vector
    from .mesh import bpy_reset
    P = geom.w2b(tool_v[np.unique(tool_f)], scale)
    c = P.mean(0)
    _, _, vt = np.linalg.svd(P - c, full_matrices=False)
    a = vt[0]
    t = (P - c) @ a
    wrist = geom.w2b(np.asarray(Jw["finger.R"], float) * 0.5 + np.asarray(Jw["wrist.R"], float) * 0.5, scale)
    e0, e1 = c + a * t.min(), c + a * t.max()
    head_end, tail_end = (e0, e1) if np.linalg.norm(e0 - wrist) > np.linalg.norm(e1 - wrist) else (e1, e0)
    ax = tail_end - head_end; L = np.linalg.norm(ax); ax /= L
    if abs(ax[2]) < 0.7:
        # den rekonstruerte biten er mest hammerhodet (bred, ikke lang): et hengende redskap står loddrett fra
        # neven, med hodet nederst
        low = P[np.argmin(P[:, 2])]
        head_end = np.array([low[0], low[1], low[2]])
        ax = np.array([0.0, 0.0, 1.0]); L = max(float(wrist[2] - low[2]) + 0.03, 0.28)
    near = P[np.abs((P - head_end) @ ax) < 0.35 * L]
    _, _, v2 = np.linalg.svd(near - near.mean(0), full_matrices=False)
    hx = v2[0] - ax * (v2[0] @ ax); hx /= np.linalg.norm(hx) + 1e-9
    hy = np.cross(ax, hx)
    hl = float(np.clip(np.ptp((near - near.mean(0)) @ hx), 0.08, 0.13))
    hc = head_end + ax * 0.028
    L = float(np.clip(L, 0.28, 0.42))
    bpy_reset()
    bm = bmesh.new()
    m = Matrix(((hx[0], hy[0], ax[0], hc[0]), (hx[1], hy[1], ax[1], hc[1]), (hx[2], hy[2], ax[2], hc[2]), (0, 0, 0, 1)))
    r = bmesh.ops.create_cube(bm, size=1.0)
    for v in r["verts"]:
        v.co = Vector((v.co.x * hl, v.co.y * 0.045, v.co.z * 0.05))
    bmesh.ops.bevel(bm, geom=list(set(e for v in r["verts"] for e in v.link_edges)), offset=0.006, segments=2, affect="EDGES")
    head_faces = set(bm.faces)
    bmesh.ops.transform(bm, matrix=m, verts=list(bm.verts))
    L2 = L - 0.03
    r2 = bmesh.ops.create_cone(bm, cap_ends=True, segments=10, radius1=0.017, radius2=0.014, depth=L2)
    m2 = Matrix(((hx[0], hy[0], ax[0], 0), (hx[1], hy[1], ax[1], 0), (hx[2], hy[2], ax[2], 0), (0, 0, 0, 1)))
    mid = head_end + ax * (0.03 + L2 / 2)
    for v in r2["verts"]:
        v.co = m2 @ v.co + Vector(mid)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    bm.verts.ensure_lookup_table(); bm.faces.ensure_lookup_table()
    V = np.array([v.co[:] for v in bm.verts])
    F = np.array([[v.index for v in f.verts] for f in bm.faces], np.int32)
    mat = np.array([0 if f in head_faces else 1 for f in bm.faces], np.int32)
    # flatene fra bevel og triangulering: jernhodet er alt nær hodets midte
    cen = V[F].mean(1)
    mat = np.where(np.linalg.norm(cen - hc, axis=1) < hl, 0, 1).astype(np.int32)
    bm.free()
    print("hammeren modellert: skaft %.2f m, hode %.2f m" % (L2, hl))
    return geom.b2w(V, scale), F, mat


def hips_iron(job, fb, W, sg):
    """Jern som henger på hofta (nøkler) på siden uten redskap: stivt på hoftene i riggen."""
    from scipy import ndimage as ndi
    rgb = load_rgb(job, "front")
    h, s, v = hsv(rgb)
    iron = (s < 45) & (v > 30) & (v < 150) & load_mask(job, "front")
    c = int(fb["front"]["c"])
    reg = np.zeros_like(iron)
    if sg > 0:
        reg[:, c:] = True
    else:
        reg[:, :c] = True
    o = fb["front"]; H = o["bot"] - o["top"]
    reg[: int(o["top"] + 0.38 * H)] = False
    reg[int(o["top"] + 0.62 * H):] = False
    iron = ndi.binary_dilation(ndi.binary_opening(iron & reg, iterations=1), iterations=6)
    zfront = W[:, 2] > np.percentile(W[:, 2], 55)
    m = in_mask(job, "front", o, W, iron) & zfront
    print("jern på hofta:", int(m.sum()), "hjørner stive på hoftene")
    return m


# ---------------------------------------------------------------- uv for hele figuren
def joint_uv(parts, head_part=1, head_gain=2.2, margin=0.003):
    """Ett uv-kart for alle delene (Smart UV), der hodets øyer får `head_gain` ganger så lange kanter."""
    import bpy  # noqa: F401 (bmesh finnes først etter bpy)
    import bmesh
    import bpy
    from .mesh import bpy_mesh, bpy_read, bpy_reset, bpy_select
    V = np.concatenate([p[0] for p in parts]); F = []; part = []
    off = 0
    for i, (v, f) in enumerate(parts):
        F.append(f + off); part.append(np.full(len(f), i)); off += len(v)
    F = np.concatenate(F); part = np.concatenate(part)
    bpy_reset()
    ob = bpy_mesh("full", V, F)
    bpy_select(ob)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=0.70, island_margin=margin, area_weight=0.0)
    bpy.ops.object.mode_set(mode="OBJECT")
    if head_gain != 1.0 and (part == head_part).any():
        bm = bmesh.new(); bm.from_mesh(ob.data)
        uv = bm.loops.layers.uv.active
        bm.faces.ensure_lookup_table()
        faces = [bm.faces[i] for i in np.nonzero(part == head_part)[0]]
        key = lambda l: (round(l[uv].uv.x, 5), round(l[uv].uv.y, 5), l.vert.index)  # noqa: E731
        by_vert, seen = {}, set()
        for f in faces:
            for l in f.loops:
                by_vert.setdefault(key(l), []).append(f)
        for f in faces:
            if f.index in seen:
                continue
            stack, isl = [f], []
            seen.add(f.index)
            while stack:
                g = stack.pop(); isl.append(g)
                for l in g.loops:
                    for h in by_vert.get(key(l), []):
                        if h.index not in seen:
                            seen.add(h.index); stack.append(h)
            pts = [l[uv].uv.copy() for g in isl for l in g.loops]
            c = sum(pts, type(pts[0])((0, 0))) / len(pts)
            for g in isl:
                for l in g.loops:
                    l[uv].uv = c + (l[uv].uv - c) * head_gain
        bm.to_mesh(ob.data); bm.free()
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.select_all(action="SELECT")
        bpy.ops.uv.pack_islands(margin=margin + 0.001, rotate=True)
        bpy.ops.object.mode_set(mode="OBJECT")
    v2, f2, uv2 = bpy_read(ob)
    return v2, f2, uv2, part


# ---------------------------------------------------------------- hele steget
def assemble(job, head, target=None):
    """`job` er kroppsarket (ferdig til og med `mesh`), `head` hodearket (ferdig til og med `mesh`)."""
    from .fuse import load_frame
    from .rig import joints
    t0 = time.time()
    fb, _ = load_frame(job); fh, _ = load_frame(head)
    s, dz, fits = head_fit(job, head, fb, fh)
    rep = dict(head_scale=round(s, 3), head_scale_front=round(fits[0], 3), head_scale_side=round(fits[1], 3), head_dz=round(dz, 1))
    H = fb["front"]["bot"] - fb["front"]["top"]
    from .mesh import decimate
    bl = np.load(job.p("lo.npz")); hl = np.load(head.p("lo.npz"))
    head_tris = int(job.opt.get("head_tris", 4500))
    bv, bf = decimate(bl["v"].astype(float), bl["f"], job.tris - head_tris)
    hv, hf = decimate(hl["v"].astype(float), hl["f"], head_tris)
    hv = hv * s + np.array([0, 0, dz])
    to_head = lambda P: (P - np.array([0, 0, dz])) / s  # noqa: E731
    m = 1 / job.scale  # piksler per meter
    # kutt kroppen litt over halskuttet på hodet (bakerst), så hodets hals går ned i kragen
    back = hv[hv[:, 2] < np.median(hv[:, 2])]
    ycut = back[:, 1].min() + 0.02 * m
    neck = hv[hv[:, 1] < ycut + 0.04 * m]
    nc = neck[:, [0, 2]].mean(0)
    nr = np.percentile(np.linalg.norm(neck[:, [0, 2]] - nc, axis=1), 50) * 0.9
    c = face_centres(bv, bf)
    r = np.hypot(c[:, 0] - nc[0], c[:, 2] - nc[1])
    drop = (c[:, 1] > ycut + 0.07 * m) | ((c[:, 1] > ycut) & (r < nr))
    bf = bf[~drop]
    print("kroppen kuttet over y = %.0f px (%d flater borte)" % (ycut, int(drop.sum())))
    # kuttflaten under halsen på hodet (malt mørk i forbildet bakfra) skal ikke stikke ut over kragen
    hn = face_normals(hv, hf); hc = face_centres(hv, hf)
    cap = (hn[:, 1] < -0.5) & (hc[:, 1] < ycut + 0.06 * m)
    hf = hf[~cap]
    print("kuttflaten på hodet fjernet:", int(cap.sum()), "flater")
    hf = drop_flakes(hv, hf, 0.02)  # rester av kuttflaten
    hv, hf, _, added = fill_holes(hv, hf, max_edges=60)  # småhull etter kuttet; halsåpningen står (inne i kragen)
    print("småhull i hodet lukket:", added, "flater")
    # redskapet
    tool_side = job.opt.get("tool_side")
    sg = 1 if (tool_side or "R") == "R" else -1
    masks, tkw = tool_masks(job, fb, sg) if tool_side else ({}, {})
    if tool_side and job.opt.get("drop_mirrored_tool") and not job.opt.get("drop_pole"):
        ghost = ghost_vertices(job, fb, bv, masks, sg, H, tkw)
        dead = ghost[bf].any(1)
        bf = bf[~dead]
        print("spøkelset i den andre hånda:", int(dead.sum()), "flater fjernet")
        rep["ghost_faces"] = int(dead.sum())
    if tool_side and job.opt.get("drop_mirrored_tool") and not job.opt.get("drop_pole"):
        Jw0 = joints(job, fb, (target.frac if target else __import__("sheet2model.targets.base", fromlist=["FRAC"]).FRAC))
        bf, rep["pruned_hand_faces"] = prune_hand(job, fb, bv, bf, Jw0, "L" if sg > 0 else "R")
    bf = drop_flakes(bv, bf, job.opt.get("flake_frac", 0.02))
    bv, bf, _, added = fill_holes(bv, bf)
    print("hull lukket:", added, "flater")
    tool = None
    if tool_side and not job.opt.get("drop_pole"):
        inside = tool_vertices(job, fb, bv, masks, sg, H)
        tf_sel = inside[bf].sum(1) >= 2
        print("redskapet:", int(tf_sel.sum()), "flater")
        if tf_sel.sum() > 20:
            tf = drop_flakes(bv, bf[tf_sel], 0.2)
            bf = bf[~tf_sel]
            bv, bf, _, added = fill_holes(bv, bf)
            print("hull etter redskapet lukket:", added, "flater")
            tool_mat = None
            if job.opt.get("model_tool", "hammer") == "hammer":
                Jw = joints(job, fb, (target.frac if target else __import__("sheet2model.targets.base", fromlist=["FRAC"]).FRAC))
                tv, tf2, tool_mat = model_tool(bv, tf, Jw, job.scale)
                tool = (tv, tf2, tool_mat)
            else:
                tv, tf2, _ = fill_holes(bv, tf)
                tool = (tv, tf2, None)
            rep["tool"] = dict(found=True, side=tool_side, faces=int(len(tool[1])), modelled=tool_mat is not None)
        else:
            rep["tool"] = dict(found=False, side=tool_side)
    hips = hips_iron(job, fb, bv, sg) if job.opt.get("hips_iron") else None
    # ett uv-kart for alle delene
    parts = [(bv, bf), (hv, hf)] + ([(tool[0], tool[1])] if tool else [])
    V, F, UV, part = joint_uv(parts, head_part=1, head_gain=job.opt.get("head_gain", 2.2))
    names = ["body", "head"] + (["tool"] if tool else [])
    hipsrigid = np.zeros(len(V), bool)
    if hips is not None:
        hipsrigid[: len(bv)] = hips
    np.savez(job.p("assembled.npz"), v=V.astype(np.float32), f=F.astype(np.int32), uv=UV.astype(np.float32),
             part=part.astype(np.int32), part_names=np.array(names), hipsrigid=hipsrigid)
    # teksturen: hver del fra sitt ark, i det felles uv-kartet
    texels = texture.rasterize(V, F, UV, job.tex, tri_part=part)
    np.savez(job.p("texels_full.npz"), pos=texels.pos, nrm=texels.nrm, cov=texels.cov, ids=texels.ids)
    body_hi = np.load(job.p("hi.npy")); head_hi = np.load(head.p("hi.npy"))
    if tool_side and masks:
        # den høye modellen har fortsatt redskapet (og spøkelset) i seg: uten dette skygger de for forkleet bak i
        # z-bufferen, og forkleet blir mørkt der
        keep = ~tool_vertices(job, fb, body_hi, masks, sg, H)
        if job.opt.get("drop_mirrored_tool") and not job.opt.get("drop_pole"):
            keep &= ~ghost_vertices(job, fb, body_hi, masks, sg, H, tkw)
        body_hi = body_hi[keep]
    else:
        keep = np.ones(len(body_hi), bool)
    excl = {}
    if tool_side and masks:  # redskapets piksler farger ikke kroppen (det er skilt ut, eller tatt bort)
        excl = dict(masks)
        if os.path.exists(job.p("tool_src.npy")) and "left" in job.views:
            from scipy import ndimage as ndi
            excl["left"] = ndi.binary_dilation(np.load(job.p("tool_src.npy")), iterations=14)[:, ::-1]
    sources = [texture.Source(job, fb, part_mask=texels.part == 0, hi=body_hi, exclude=excl),
               texture.Source(head, fh, part_mask=texels.part == 1, to_local=to_head, hi=head_hi)]
    if tool and tool[2] is None:
        sources.append(texture.Source(job, fb, part_mask=texels.part == 2, hi=body_hi))
    trep = texture.project(job, texels, sources, out_stem="albedo_full")
    if tool and tool[2] is not None:
        paint_tool(job, texels, part, tool[2], "albedo_full")
    rep.update(trep)
    # normalkartet fra begge de høye modellene (hodet flyttet inn i kroppens rom)
    bn = np.load(job.p("hi_n.npy"))[keep]
    hnn = np.load(head.p("hi_n.npy"))
    hi_v = np.concatenate([body_hi, head_hi * s + np.array([0, 0, dz], np.float32)])
    hi_n = np.concatenate([bn, hnn])
    if tool:
        tv = tool[0][np.unique(tool[1])]
        hi_v = np.concatenate([hi_v, tv]); hi_n = np.concatenate([hi_n, texture.vertex_normals(tool[0], tool[1])[np.unique(tool[1])]])
    texture.normal_map(job, texels, (V, F, UV), hi_v.astype(np.float32), hi_n.astype(np.float32), out_stem="normal_full")
    rep["face_texels_cm"] = texture.face_density(job, texels, V, F, UV, region=(part == 1) & (face_normals(V, F)[:, 2] > 0.3))
    rep["tris"] = int(len(F))
    json.dump(rep, open(job.p("assemble.json"), "w"), indent=1)
    job.time("assemble", time.time() - t0)
    return rep


def paint_tool(job, texels, part, mat, stem):
    """Den modellerte hammeren får malte farger (jern og tre med litt støy) rett i teksturen."""
    import cv2
    im = cv2.imread(job.p(f"{stem}.png"))
    yy, xx = np.nonzero((texels.part == 2))
    fm = mat[texels.ids[yy, xx] - np.searchsorted(np.nonzero(part == 2)[0], np.nonzero(part == 2)[0][0])] if False else None
    tri = texels.ids[yy, xx]
    first = int(np.nonzero(part == 2)[0][0])
    kind = mat[np.clip(tri - first, 0, len(mat) - 1)]
    rng = np.random.default_rng(1)
    noise = 1 + 0.12 * (rng.random(len(yy)) - 0.5)
    iron = np.array([0.10, 0.105, 0.115]) ** (1 / 2.2) * 255
    wood = np.array([0.28, 0.15, 0.07]) ** (1 / 2.2) * 255
    col = np.where(kind[:, None] == 0, iron, wood) * noise[:, None]
    im[yy, xx] = np.clip(col[:, ::-1], 0, 255).astype(np.uint8)
    cv2.imwrite(job.p(f"{stem}.png"), im)
