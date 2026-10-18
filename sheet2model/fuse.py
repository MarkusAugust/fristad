"""Steg 3: innretting, kalibrering av relativ dybde mot det visuelle skallet, sammensmelting i et volum og
marching cubes.

Volumet er et felt F (positivt inni) på et rutenett av voksler à `vox` piksler. F er minimum over visningene av
to ting: signert avstand til silhuetten og avstand bak dybdeflaten. Nær silhuettkanten er dybden upålitelig, så der
tar silhuetten over. Tverrsnittene kan få en superellipse som prior (`elliptic`, hele figurer).
"""
import json
import time

import numpy as np

from . import geom
from .sheet import load_mask


# ---------------------------------------------------------------- rutenettet
class Grid:
    def __init__(self, xs, ys, zs, vox):
        self.xs, self.ys, self.zs, self.vox = xs, ys, zs, vox
        X, Y, Z = np.meshgrid(xs, ys, zs, indexing="ij")
        self.X, self.Y, self.Z = X.astype(np.float32), Y.astype(np.float32), Z.astype(np.float32)
        self.shape = X.shape
        self._proj = {}

    @property
    def origin(self):
        return np.array([self.xs[0], self.ys[0], self.zs[0]])

    def project(self, view, o, flat=True):
        """(u, v) for alle vokslene, bufret per visning og ramme (silhuett og dybde bruker samme projeksjon)."""
        key = (view.name, round(o["c"], 3), round(o["top"], 3), round(o["sy"], 5), o.get("yaw", 0.0), o.get("pitch", 0.0), o.get("roll", 0.0))
        if key not in self._proj:
            if len(self._proj) > 8:
                self._proj.clear()
            self._proj[key] = geom.to_img(view, o, self.X.ravel(), self.Y.ravel(), self.Z.ravel())
        u, v = self._proj[key]
        return (u, v) if flat else (u.reshape(self.shape), v.reshape(self.shape))


def make_grid(job, fr, vox=None, margin=32):
    """Rutenettet dekker silhuettene (med marg): bredden fra forfra/bakfra, dybden fra profilene, høyden fra forfra.
    Vokselstørrelsen økes om rutenettet ellers ville fått mer enn `max_voxels` voksler (store hatter)."""
    vox = vox or job.opt.get("vox", 3.0)
    if job.prop:  # symmetrisk om origo: største utstrekning i noen rute, pluss marg
        R = 0
        for k, v in job.views.items():
            if v.mirror_of:
                continue
            o = fr[k]
            R = max(R, (o["u1"] - o["u0"]) / 2 / o["sy"] * 1.2, (o["bot"] - o["top"]) / 2 / o["sy"] * 1.2)
        R += margin
        vmax = float(job.opt.get("max_voxels", 14e6))
        if (2 * R / vox) ** 3 > vmax:  # store ark (1024 × 2048 per rute): grovere voksel, ellers tar fuse minutter
            vox = float(np.ceil((2 * R) / vmax ** (1 / 3) * 2) / 2)
            print("rutenettet er stort: voksel %.1f px" % vox)
        xs = np.arange(-R, R, vox)
        return Grid(xs, xs.copy(), xs.copy(), vox)
    H = fr["front"]["bot"] - fr["front"]["top"]

    def span(keys, sign):
        lo, hi = [], []
        for k in keys:
            if k not in job.views:
                continue
            m = load_mask(job, k)
            cols = np.nonzero(m.any(0))[0]
            a0, a1 = (cols.min() - fr[k]["c"]) / fr[k]["sy"], (cols.max() - fr[k]["c"]) / fr[k]["sy"]
            if sign(k) < 0:
                a0, a1 = -a1, -a0
            lo.append(a0); hi.append(a1)
        return min(lo) - margin, max(hi) + margin
    xr = span(("front", "back"), lambda k: 1 if k == "front" else -1)
    zr = span(("right", "left"), lambda k: 1 if k == "right" else -1)
    n = (xr[1] - xr[0]) * (H + 20) * (zr[1] - zr[0])
    vmax = float(job.opt.get("max_voxels", 14e6))
    if n / vox ** 3 > vmax:
        vox = float(np.ceil((n / vmax) ** (1 / 3) * 2) / 2)
        print("rutenettet er stort: voksel %.1f px" % vox)
    xs = np.arange(xr[0], xr[1], vox)
    ys = np.arange(-H - 10, 10, vox)
    zs = np.arange(zr[0], zr[1], vox)
    return Grid(xs, ys, zs, vox)


# ---------------------------------------------------------------- innretting (startverdier)
def frame(job):
    """Startinnretting: samme topp og bunn (ortografisk, vannrett kamera gir samme høyeste punkt i alle
    visninger), sentrum for x fra forfra/bakfra og z fra midten av profilene. Ruta ovenfra får skalaen fra
    forfra og nullpunktet fra midten av silhuetten; `align.py` finjusterer alt."""
    out = {}
    for k in job.views:
        m = load_mask(job, k)
        ys, xs = np.nonzero(m)
        out[k] = dict(top=int(ys.min()), bot=int(ys.max()), yaw=0.0, pitch=0.0, roll=0.0,
                      u0=int(xs.min()), u1=int(xs.max()))
    if job.prop:
        # gjenstander: nullpunktet i midten av silhuetten (verden har origo i gjenstandens midte), samme skala i
        # alle rutene, og kameraet vippet (`view_pitch`) som arket er tegnet
        for k, v in job.views.items():
            o = out[k]
            o["sy"] = 1.0
            o["c"] = 0.5 * (o["u0"] + o["u1"]); o["top"] = 0.5 * (o["top"] + o["bot"])
            o["pitch"] = float(job.opt.get("view_pitch", 20.0))
        for k, v in job.views.items():
            if v.mirror_of:
                out[k] = geom.mirror_frame(out[v.mirror_of], job.TW)
        return out
    f = out["front"]
    for k in job.views:
        o = out[k]
        o["sy"] = (o["bot"] - o["top"]) / (f["bot"] - f["top"])
    if job.opt.get("same_scale"):  # rutene er tegnet i samme målestokk; bunnen (halskuttet) er upålitelig
        for k in job.views:
            out[k]["sy"] = 1.0
    if job.opt.get("align_skin"):
        align_skin(job, out)
    for k in job.views:  # vannrett sentrum: midten av silhuetten over ørene (øverste 45 %)
        m = load_mask(job, k)
        o = out[k]
        if job.views[k].facing == "top":
            o["sy"] = f["sy"]
            o["c"] = 0.5 * (o["u0"] + o["u1"])
            o["top"] = 0.5 * (o["top"] + o["bot"])  # v for z = 0 (midten av hodet i dybden)
            continue
        r0, r1 = o["top"] + 40, o["top"] + int(0.45 * (o["bot"] - o["top"]))
        cols = np.nonzero(m[r0:r1].any(0))[0]
        o["c"] = 0.5 * (cols.min() + cols.max())
    for k, v in job.views.items():
        if v.mirror_of:
            out[k] = geom.mirror_frame(out[v.mirror_of], job.TW)
    return out


def align_skin(job, out):
    """Høyde og skala per profil fra hudlinjene: andelen hud per rad (pannen under skautet eller håret, ansiktet,
    halsen) skal ligge likt i profilene som forfra. Bildegeneratoren tegner ikke issen og halskuttet like høyt i
    alle rutene, så topp og bunn alene gir feil høyde på øynene (dobbelt ansikt i mellomvinkelen)."""
    from .sheet import load_rgb, skin_mask

    def signal(k):
        sk = skin_mask(load_rgb(job, k), vmin=120) & load_mask(job, k)
        m = load_mask(job, k)
        return sk.sum(1) / np.maximum(m.sum(1), 1)

    f = out["front"]
    sf = signal("front")
    rows = np.arange(f["top"], f["bot"])
    for k, v in job.views.items():
        if k in ("front", "back") or v.mirror_of or v.facing == "top":
            continue
        sk_ = signal(k)
        best = (-1e9, out[k]["sy"], out[k]["top"])
        for a in np.arange(0.9, 1.1001, 0.005):
            for b in range(-60, 61, 1):
                vv = np.clip(np.round(out[k]["top"] + b + (rows - f["top"]) * a).astype(int), 0, len(sk_) - 1)
                x, y = sf[rows], sk_[vv]
                if x.std() < 1e-6 or y.std() < 1e-6:
                    continue
                c = np.corrcoef(x, y)[0, 1]
                if c > best[0]:
                    best = (c, a, out[k]["top"] + b)
        print("hudlinjer", k, "korr %.3f skala %.3f topp %d" % best)
        out[k]["sy"], out[k]["top"] = float(best[1]), int(best[2])


# ---------------------------------------------------------------- silhuettfeltene
def signed_silhouette(job, k):
    from scipy import ndimage as ndi
    mb = load_mask(job, k)
    return (ndi.distance_transform_edt(mb) - ndi.distance_transform_edt(~mb)).astype(np.float32)


def silhouette_fields(job, fr, grid, keys=None):
    """Signert avstand til silhuetten (verdenspiksler, positiv inni) per visning, på rutenettet."""
    out = {}
    for k in (keys or job.views):
        sd = signed_silhouette(job, k)
        u, v = grid.project(job.views[k], fr[k])
        out[k] = (geom.sample(sd, u, v, fill=-500).reshape(grid.shape) / fr[k]["sy"]).astype(np.float32)
    return out


def hull_depth(job, k, fr, grid, inside):
    """Det visuelle skallets nærmeste flate sett fra visningen k: per bildepiksel det største «fram mot
    kameraet» blant vokslene inni skallet."""
    view, o = job.views[k], fr[k]
    idx = np.nonzero(inside.ravel())[0]
    X, Y, Z = grid.X.ravel()[idx], grid.Y.ravel()[idx], grid.Z.ravel()[idx]
    u, v = geom.to_img(view, o, X, Y, Z)
    t = geom.toward(view, o, X, Y, Z)
    H, W = load_mask(job, k).shape
    ui = np.clip(np.round(u).astype(int), 0, W - 1); vi = np.clip(np.round(v).astype(int), 0, H - 1)
    return geom.scatter_max(vi * W + ui, t, H * W).reshape(H, W)


# ---------------------------------------------------------------- kalibrering
def calibrate(job, fr, grid, sil, prior=None):
    """Relativ dybde -> metrisk (verdenspiksler fram mot kameraet): per bilderad skal det nærmeste punktet i
    dybdekartet (98-persentilen) ligge på det visuelle skallets nærmeste flate (fra de andre visningene).
    Felles skala for alle visningene med egen forskyvning per visning, eller egen skala per visning
    (`scale_per_view`), Huber-vektet."""
    inside = np.ones(grid.shape, bool) if prior is None else prior > 0
    for k in job.views:
        inside &= sil[k] > 0
    H = (fr["front"]["bot"] - fr["front"]["top"]) if "front" in fr else 1.0
    data = {}
    lo_, hi_ = job.opt.get("cal_rows", (0.04, 0.85))  # hattebremmer o.l. øverst forstyrrer kalibreringen
    for k, view in job.views.items():
        m = load_mask(job, k)
        d = np.load(job.p(f"{k}_disp.npy"))
        zb = hull_depth(job, k, fr, grid, inside)
        rows = np.nonzero(m.any(1))[0]
        dm, tg, keep = [], [], []
        for v in rows[::2]:
            row = d[v][m[v]]
            t = zb[v][np.isfinite(zb[v])]
            if len(row) < 4 or len(t) < 1:
                continue
            dm.append(np.percentile(row, 98)); tg.append(t.max())
            if view.facing == "top" or job.prop:
                keep.append(True)
            else:
                y = (v - fr[k]["top"]) / fr[k]["sy"]
                keep.append(lo_ * H < y < hi_ * H)
        dm, tg, keep = np.array(dm), np.array(tg), np.array(keep, bool)
        data[k] = (dm[keep], tg[keep])
    keys = list(job.views)

    def solve(per_view):
        rows, rhs = [], []
        ncol = 2 * len(keys) if per_view else 1 + len(keys)
        for i, k in enumerate(keys):
            x0, y0 = data[k]
            A = np.zeros((len(x0), ncol))
            if per_view:
                A[:, 2 * i] = x0; A[:, 2 * i + 1] = 1
            else:
                A[:, 0] = x0; A[:, 1 + i] = 1
            rows.append(A); rhs.append(y0)
        A = np.concatenate(rows); y = np.concatenate(rhs)
        w = np.ones(len(y))
        for _ in range(8):
            sol = np.linalg.lstsq(A * w[:, None], y * w, rcond=None)[0]
            r = y - A @ sol
            s = np.median(np.abs(r)) * 1.5 + 1e-3
            w = 1 / np.maximum(1, np.abs(r) / s)
        cal, off = {}, 0
        for i, k in enumerate(keys):
            n = len(data[k][0])
            a, b = (sol[2 * i], sol[2 * i + 1]) if per_view else (sol[0], sol[1 + i])
            cal[k] = dict(a=float(a), b=float(b), residual=float(np.median(np.abs(r[off:off + n]))) if n else 0.0)
            off += n
        return cal
    per_view = bool(job.opt.get("scale_per_view"))
    cal = solve(per_view)
    if not per_view and cal[keys[0]]["a"] <= 0:  # dybden kan ikke være speilvendt: egen skala per visning
        cal = solve(True)
    for k in keys:  # en negativ skala i en enkelt visning er ubrukelig: fall tilbake på medianen av de andre
        if cal[k]["a"] <= 0:
            good = [c["a"] for c in cal.values() if c["a"] > 0]
            cal[k]["a"] = float(np.median(good)) if good else 1.0
            x0, y0 = data[k]
            cal[k]["b"] = float(np.median(y0 - cal[k]["a"] * x0)) if len(x0) else 0.0
            cal[k]["fallback"] = True
    for k in keys:
        print(k, "a=%.2f b=%.1f  avvik median %.1f px" % (cal[k]["a"], cal[k]["b"], cal[k]["residual"]))
    return cal


# ---------------------------------------------------------------- feltet
def depth_field(job, k, fr, cal, grid):
    """Avstand bak dybdeflaten (positiv bak) for visningen k, på rutenettet."""
    from scipy import ndimage as ndi
    view, o = job.views[k], fr[k]
    mb = load_mask(job, k)
    d = np.load(job.p(f"{k}_disp.npy"))
    depth = cal[k]["a"] * d + cal[k]["b"]
    R = float(job.opt.get("edge_slack", 28.0))  # tynne lemmer trenger liten slakk, ellers blir de kantete bokser
    dist = ndi.distance_transform_edt(mb)
    depth = depth + np.maximum(0, R - dist) * float(job.opt.get("edge_gain", 1.5))
    if k == "back" and "back_cut" in job.opt:  # halskuttet sett bakfra er ikke en flate på hodet
        depth[int(o["top"] + job.opt["back_cut"] * (o["bot"] - o["top"])):] += 1e3
    depth = np.where(mb, depth, -1e3).astype(np.float32)
    u, v = grid.project(view, o)
    surf = geom.sample(depth, u, v, fill=-1e3).reshape(grid.shape)
    t = geom.toward(view, o, grid.X, grid.Y, grid.Z)
    margin = float(job.opt.get("depth_margin", 0.0))
    if margin and job.opt.get("tool_box") and not view.mirror_of:
        # bare rundt redskapet: på redskapssiden, utenfor kroppen, i høyden til boksen i profilen
        from .sheet import tool_boxes
        bb = tool_boxes(job)
        o_ = fr["right"]
        ytop = max(-(b[1] - o_["top"]) / o_["sy"] for b in bb); ybot = min(-(b[3] - o_["top"]) / o_["sy"] for b in bb)
        sg = 1 if job.opt.get("tool_side", "R") == "R" else -1
        X = grid.X
        margin = np.where((grid.Y <= ytop) & (grid.Y >= ybot) & (sg * X < -0.5 * np.abs(X).max()), margin, 0.0)
    return (surf + margin - t).astype(np.float32)


def build_field(job, fr, cal, grid, sil, prior=None):
    F = np.full(grid.shape, 1e3, np.float32)
    if prior is not None:
        F = np.minimum(F, prior + float(job.opt.get("prior_tol", 3.0)))
    carve = float(job.opt.get("prior_carve", 12.0)) if prior is not None else None
    for k, view in job.views.items():
        dv = depth_field(job, k, fr, cal, grid)
        if carve is not None:  # gjenstander: dybden gir bare relieff (høyst `prior_carve` px inn i prioren)
            dv = np.maximum(dv, prior - carve)
        sv = np.minimum(sil[k], dv)
        if view.mirror_of:
            # den speilede profilen er en antakelse om den andre siden: den får bare skjære i sin egen halvdel,
            # ellers skjærer den bort det den ene hånda holder (redskapet er tatt ut av den)
            cam = geom.basis(view, fr[k])[0]
            own = (grid.X * cam[0] + grid.Z * cam[2]) > -12
            sv = np.where(own, sv, 1e3)
        F = np.minimum(F, sv)
    if job.opt.get("elliptic"):
        F = np.minimum(F, elliptic(job, fr, grid, job.opt.get("elliptic_p", 2.5)) + job.opt.get("elliptic_tol", 4.0))
    if "neck" in job.opt:
        F = clean_neck(job, F, grid, fr, job.opt["neck"])
    H = (fr["front"]["bot"] - fr["front"]["top"]) if "front" in fr else 0.0
    dome = job.opt.get("dome", 0.14)
    if dome > 0 and "top" not in job.views:
        # toppen: ingen visning ser ovenfra, så kantene der dybdeflatene skjærer hverandre glattes til en kuppel
        from scipy import ndimage as ndi
        wtop = np.clip((grid.Y + dome * H) / 40.0, 0, 1)
        F = F * (1 - wtop) + ndi.gaussian_filter(F, 9.0 / grid.vox) * wtop  # 9 px uansett vokselstørrelse
    from scipy import ndimage as ndi
    return ndi.gaussian_filter(F, 0.8)


def elliptic(job, fr, grid, pw=2.5):
    """Tverrsnitt som superellipse: bredden fra biten i raden forfra, dybden fra biten i raden i profilen.
    Dybdemodellen gir lite relieff på en hel figur, så uten dette blir armer og bein firkantede bokser."""
    out = np.full(grid.shape, 1e3, np.float32)
    fk, pk = "front", "right"
    if fk not in job.views or pk not in job.views:
        return out
    mf, mp = load_mask(job, fk), load_mask(job, pk)
    lf, rf = geom.run_bounds(mf)
    lp, rp = geom.run_bounds(mp)
    uf, vf = grid.project(job.views[fk], fr[fk])
    up, vp = grid.project(job.views[pk], fr[pk])
    inside = geom.nearest(mf, uf, vf) & geom.nearest(mp, up, vp)
    of, op = fr[fk], fr[pk]
    xa = (geom.nearest(lf, uf, vf) - of["c"]) / of["sy"]; xb = (geom.nearest(rf, uf, vf) + 1 - of["c"]) / of["sy"]
    za = (geom.nearest(lp, up, vp) - op["c"]) / op["sy"]; zb = (geom.nearest(rp, up, vp) + 1 - op["c"]) / op["sy"]
    xc, rx = (xa + xb) / 2, np.maximum((xb - xa) / 2, 1)
    zc, rz = (za + zb) / 2, np.maximum((zb - za) / 2, 1)
    e = (np.abs(grid.X.ravel() - xc) / rx) ** pw + (np.abs(grid.Z.ravel() - zc) / rz) ** pw
    val = (1 - e ** (1 / pw)) * np.minimum(rx, rz)
    val = np.where(inside, val, 1e3)  # utenfor tar silhuettene seg av det
    return np.minimum(out, val.reshape(grid.shape).astype(np.float32))


def clean_neck(job, F, grid, fr, o_):
    """Rent, skrått halskutt og en elliptisk hals bak skjegget (der forbildet ikke er enig med seg selv)."""
    X, Y, Z = grid.X, grid.Y, grid.Z
    o = fr["right"]

    def wpt(u, v):
        return (u - o["c"]) / o["sy"], -(v - o["top"]) / o["sy"]
    z1, y1 = wpt(*o_["plane"][0]); z2, y2 = wpt(*o_["plane"][1])
    k_ = (y2 - y1) / (z2 - z1)
    plane = (Y - (y1 + k_ * (Z - z1))) / np.sqrt(1 + k_ * k_) - o_["plane_up"]
    ytop = -(o_["top_v"] - fr["front"]["top"])
    hx = o_["half_x"] * (1 + o_.get("flare", 0.25) * np.clip((ytop - Y) / 150.0, 0, 1))
    e = np.sqrt((X / hx) ** 2 + ((Z - o_["z0"]) / o_["half_z"]) ** 2)
    neck = np.minimum((1 - e) * o_["half_z"], (ytop + 120) - Y)
    y905 = -(o_["front_v"] - fr["front"]["top"])
    neck = np.minimum(neck, (Y - y905) + np.maximum(0, -20 - Z) * 1.5)
    region = np.clip((ytop - Y) / 20.0, 0, 1) * np.clip((o_["beard_z"] - Z) / 20.0, 0, 1)
    F = F * (1 - region) + np.maximum(neck, np.minimum(F, -1)) * region
    F = np.maximum(F, np.where(Y < ytop + 120, neck, -1e3))
    return np.minimum(F, plane)


# ---------------------------------------------------------------- hele steget
def fuse(job, fr=None):
    from skimage import measure
    from . import align
    t0 = time.time()
    fr = fr or frame(job)
    grid = make_grid(job, fr)
    sil = silhouette_fields(job, fr, grid)
    report = {}
    prior = None
    if job.prop:
        from . import props
        report["prior"], prior, fr = props.fit(job, fr, grid)
        sil = silhouette_fields(job, fr, grid)
        if job.opt.get("prior_only", True):
            # geometrien er prioren (shapes.py bygger nettet): feltet brukes bare til IoU og feilbildet
            F = prior
            report["iou"] = align.fused_iou(job, fr, grid, F > 0)
            np.save(job.p("field.npy"), F)
            np.savez(job.p("grid.npz"), x0=grid.xs[0], y0=grid.ys[0], z0=grid.zs[0], vox=grid.vox)
            json.dump(dict(frame=fr, cal={}, **report), open(job.p("frame.json"), "w"), indent=1)
            job.time("fuse", time.time() - t0)
            return fr, {}, report
    else:
        a = job.opt.get("align")
        if a and (a.get("rounds", 0) or 0) > 0:
            fr, sil, report["align"] = align.align(job, fr, grid, sil)
    cal = calibrate(job, fr, grid, sil, prior)
    F = build_field(job, fr, cal, grid, sil, prior)
    report["iou"] = align.fused_iou(job, fr, grid, F > 0)
    np.save(job.p("field.npy"), F)
    verts, faces, _, _ = measure.marching_cubes(F, 0.0, spacing=(grid.vox,) * 3, step_size=1)
    verts += grid.origin
    faces = faces[:, [0, 2, 1]]  # feltet er positivt inni: snu, så normalene peker ut
    print("trekanter (høy)", len(faces))
    np.savez(job.p("mesh.npz"), v=verts.astype(np.float32), f=faces.astype(np.int32))
    np.savez(job.p("grid.npz"), x0=grid.xs[0], y0=grid.ys[0], z0=grid.zs[0], vox=grid.vox)
    json.dump(dict(frame=fr, cal=cal, **report), open(job.p("frame.json"), "w"), indent=1)
    job.time("fuse", time.time() - t0)
    return fr, cal, report


def load_frame(job):
    d = json.load(open(job.p("frame.json")))
    return d["frame"], d["cal"]


def look(job, out=None, yaws=(0, -90, 90, 180, -45), pitch=(0, 0, 0, 0, 20)):
    """Rask kontroll uten Blender: skyggelagt dybde av feltet fra flere vinkler."""
    import cv2
    from scipy import ndimage as ndi
    F = np.load(job.p("field.npy"))
    ims = []
    for yaw, pt in zip(yaws, pitch):
        G = F
        if yaw:
            G = ndi.rotate(G, yaw, axes=(0, 2), reshape=False, order=1, cval=-50)
        if pt:
            G = ndi.rotate(G, pt, axes=(1, 2), reshape=False, order=1, cval=-50)
        inside = G > 0
        idx = inside[:, :, ::-1].argmax(2)
        hit = inside.any(2)
        dep = (G.shape[2] - idx).astype(np.float32)
        gy, gx = np.gradient(ndi.gaussian_filter(dep, 0.7))
        n = np.stack([-gx, -gy, np.ones_like(dep)], -1)
        n /= np.linalg.norm(n, axis=-1, keepdims=True)
        L = np.array([-0.4, 0.5, 0.75]); L /= np.linalg.norm(L)
        sh = np.clip(n @ L, 0, 1) * 0.8 + 0.2
        img = np.where(hit, sh, 0.5).T[::-1]
        ims.append((img * 255).astype(np.uint8))
    out = out or job.shot("look.png")
    cv2.imwrite(out, np.hstack(ims))
    return out
