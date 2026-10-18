"""Innretting per visning med frihetsgrader, målt og rettet automatisk.

Hver visning får sideveis forskyvning (`c`), høyde (`top`, ikke forfra: den er referansen), skala (`sy`),
dreining om loddrett akse (`yaw`) og rulling (`roll`). De finnes med koordinatsøk mot **leave-one-out-IoU**:
det visuelle skallet fra de andre visningene projiseres med kandidatparametrene og sammenlignes med visningens
egen silhuett. En profil som er tegnet litt dreid, blir da modellert som dreid i stedet for å smelte to kropper
sammen.

En akse de andre visningene ikke sier noe om (dybdemidten i en enkelt profil) gir en flat IoU-kurve; en liten
straff for å gå bort fra startverdien holder parameteren i ro der.
"""
import time

import numpy as np

from . import geom
from .sheet import load_mask

REG = 0.002  # straff per normalisert enhet bort fra startverdien


def small_mask(job, k, vox):
    import cv2
    m = load_mask(job, k).astype(np.float32)
    h, w = m.shape
    s = cv2.resize(m, (int(np.ceil(w / vox)), int(np.ceil(h / vox))), interpolation=cv2.INTER_AREA)
    return s > 0.5


def occupancy(job, k, o, pts, shape, vox):
    u, v = geom.to_img(job.views[k], o, pts[0], pts[1], pts[2])
    ui = np.round(u / vox).astype(int); vi = np.round(v / vox).astype(int)
    ok = (ui >= 0) & (ui < shape[1]) & (vi >= 0) & (vi < shape[0])
    occ = np.zeros(shape, bool)
    occ[vi[ok], ui[ok]] = True
    return occ


def iou(a, b):
    i = np.count_nonzero(a & b); u = np.count_nonzero(a | b)
    return i / u if u else 0.0


def hull_points(job, k, grid, sil):
    """Vokslene inni skallet fra de andre visningene (ikke k, ikke speilbildet av k og ikke k sin kilde)."""
    v = job.views[k]
    skip = {k, v.mirror_of} | {j for j, w in job.views.items() if w.mirror_of == k}
    inside = np.ones(grid.shape, bool)
    for j in job.views:
        if j not in skip:
            inside &= sil[j] > 0
    return shell(grid, inside)


def shell(grid, inside):
    """Bare overflatevokslene: projeksjonen av et fylt volum er den samme som av skallet, og det er ti ganger
    færre punkter."""
    from scipy import ndimage as ndi
    sh = inside & ~ndi.binary_erosion(inside, iterations=1)
    idx = np.nonzero(sh.ravel())[0]
    return (grid.X.ravel()[idx], grid.Y.ravel()[idx], grid.Z.ravel()[idx])


def search(job, k, o0, pts, mask, vox, dofs, coarse=True):
    """Koordinatsøk: hver parameter etter tur, to ganger, med grov og fin steglengde."""
    o = dict(o0)
    shape = mask.shape
    span = dict(c=(60.0, 6.0), top=(40.0, 4.0), sy=(0.06, 0.006), yaw=(dofs.get("yaw", 0.0), 1.0),
                roll=(dofs.get("roll", 0.0), 0.5), pitch=(dofs.get("pitch", 0.0), 0.5))

    def score(cand):
        s = iou(occupancy(job, k, cand, pts, shape, vox), mask)
        pen = 0.0
        for p, (rng, _) in span.items():
            if rng:
                pen += abs(cand[p] - o0[p]) / rng
        return s - REG * pen
    best = score(o)
    start = best
    for _ in range(1):
        for p, (rng, fine) in span.items():
            if not rng or not dofs.get(p, True):
                continue
            step = rng / 4 if coarse else fine
            while step >= fine * 0.99:
                improved = True
                while improved:
                    improved = False
                    for d in (-step, step):
                        cand = dict(o); cand[p] = o[p] + d
                        if p == "sy" and cand[p] <= 0.5:
                            continue
                        s = score(cand)
                        if s > best + 1e-5:
                            best, o = s, cand
                            improved = True
                step /= 2
    return o, start, best


def informative(job, k):
    """Kan visningen k rettes inn sideveis? Bare når en uavhengig visning (ikke dens speilbilde) har k sin
    vannrette bildeakse i sitt eget bildeplan: for en profil er det den andre profilen eller ruta ovenfra, for
    forfra er det profilene. Med bare én profil (body3) gir skallet fra forfra og bakfra bare et bånd i profilen,
    og IoU mot det kan ikke måle noe sideveis; da står startinnrettingen (topp og bunn) urørt."""
    v = job.views[k]
    for j, w in job.views.items():
        if j == k or w.mirror_of == k or v.mirror_of == j:
            continue
        if abs(float(v.right @ w.cam)) < 0.5:
            return True
    return False


def align(job, fr, grid, sil):
    t0 = time.time()
    a = job.opt.get("align", {})
    vox = grid.vox
    report = {}
    order = [k for k, v in job.views.items() if not v.mirror_of]
    for rnd in range(int(a.get("rounds", 2))):
        for k in order:
            v = job.views[k]
            if not informative(job, k):
                report[k] = dict(skipped="ingen uavhengig visning på tvers")
                continue
            pts = hull_points(job, k, grid, sil)
            if len(pts[0]) == 0:
                continue
            mask = small_mask(job, k, vox)
            # høyden (top) og skalaen står som standard: de er gitt av issen og bunnen i alle rutene, og en tynn
            # hattebrem forsvinner om profilen flyttes noen piksler (skallet er et overmål, så IoU kan «vinne» på
            # det). `align_skin` retter høyden når rutene faktisk er tegnet ulikt høyt.
            dofs = dict(c=a.get("shift", True), top=(a.get("shift_v", False) and k != "front"),
                        sy=(a.get("scale", False) and k != "front"), yaw=float(a.get("yaw", 0.0)),
                        roll=float(a.get("roll", 0.0)), pitch=float(a.get("pitch", 0.0)))
            if v.facing == "top":
                dofs["top"] = a.get("shift", True)
                dofs["yaw"] = 0.0  # dreining ovenfra: ikke skilt fra c/top med silhuett alene
            o, before, after = search(job, k, fr[k], pts, mask, vox, dofs, coarse=(rnd == 0))
            fr[k] = o
            for j, w in job.views.items():
                if w.mirror_of == k:
                    fr[j] = geom.mirror_frame(o, job.TW)
            from .fuse import silhouette_fields
            keys = [k] + [j for j, w in job.views.items() if w.mirror_of == k]
            sil.update(silhouette_fields(job, fr, grid, keys))
            report[k] = dict(iou_before=round(before, 4), iou_after=round(after, 4),
                             c=round(o["c"], 1), top=round(o["top"], 1), sy=round(o["sy"], 4),
                             yaw=round(o.get("yaw", 0.0), 2), roll=round(o.get("roll", 0.0), 2))
            print("innretting %s runde %d: IoU %.3f -> %.3f  c %.1f top %.1f sy %.3f yaw %.1f roll %.1f" % (
                k, rnd + 1, before, after, o["c"], o["top"], o["sy"], o.get("yaw", 0.0), o.get("roll", 0.0)))
    # målestokken for dreining: samme yaw i alle visningene er bare hele figuren dreid; trekk fra snittet, så
    # figuren fortsatt vender rett fram
    yaws = [fr[k].get("yaw", 0.0) for k in order if job.views[k].facing != "top"]
    if yaws and abs(np.mean(yaws)) > 1e-6:
        g = float(np.mean(yaws))
        for k in order:
            fr[k]["yaw"] = fr[k].get("yaw", 0.0) - g
            for j, w in job.views.items():
                if w.mirror_of == k:
                    fr[j] = geom.mirror_frame(fr[k], job.TW)
        from .fuse import silhouette_fields
        sil.update(silhouette_fields(job, fr, grid))
        report["yaw_gauge"] = round(g, 2)
    job.time("align", time.time() - t0)
    return fr, sil, report


def fused_iou(job, fr, grid, inside):
    """Den ferdige formens silhuett mot forbildets, per visning (full oppløsning)."""
    out = {}
    pts = shell(grid, inside)
    for k, v in job.views.items():
        if v.mirror_of:
            continue
        m = small_mask(job, k, grid.vox)
        occ = occupancy(job, k, fr[k], pts, m.shape, grid.vox)
        out[k] = round(iou(occ, m), 4)
    return out


def worst_view_image(job, fr, grid, inside, path):
    """Bilde av det som er galt: forbildets silhuett (grønn) over formens (rød) i visningen med lavest IoU."""
    import cv2
    ious = fused_iou(job, fr, grid, inside)
    k = min(ious, key=ious.get)
    m = small_mask(job, k, grid.vox)
    pts = shell(grid, inside)
    occ = occupancy(job, k, fr[k], pts, m.shape, grid.vox)
    im = np.zeros(m.shape + (3,), np.uint8)
    im[..., 2] = occ * 255
    im[..., 1] = m * 255
    cv2.putText(im, "%s IoU %.2f" % (k, ious[k]), (10, 40), cv2.FONT_HERSHEY_SIMPLEX, 1.2, (255, 255, 255), 2)
    cv2.imwrite(path, im)
    return k, ious[k]
