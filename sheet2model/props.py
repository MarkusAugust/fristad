"""Gjenstander (`prop1`, `prop2`): formen kommer fra en prior som passes til silhuetten, ikke fra silhuettene alene.

Priorer (`opt.prior`): `box` (halvmål hx, hy, hz), `chest` (boks med halvsylinder-lokk, `lid` = lokkets radius som andel av hz), `cylinder` (radius r, halvhøyde hy), `barrel` (sylinder med
bul: r(y) = r·(1 + bulge·(1 − (y/hy)²))). `revolve` (radiusprofil fra silhuetten) og en glb-fil som prior er
ikke laget ennå (se README). Priorens mål og visningens vipp (pitch), dreining (yaw ±10°) og nullpunkt finnes med
koordinatsøk mot IoU mellom priorens projiserte silhuett og forbildets, på et grovt rutenett (sekunder).
Feltet blir min(prior + toleranse, silhuett, dybde) som for figurene, med dybden kalibrert mot prioren.
"""
import numpy as np

from . import align, geom
from .sheet import load_mask


def prior_inside(kind, p, X, Y, Z):
    """Signert «innenfor»-verdi (positiv inni, i piksler) for prioren med parametrene p."""
    if kind == "box":
        return np.minimum(np.minimum(p["hx"] - np.abs(X), p["hy"] - np.abs(Y)), p["hz"] - np.abs(Z))
    if kind == "chest":  # boks med halvsylinder-lokk langs x; lokkets radius er `lid` · hz
        body = np.minimum(np.minimum(p["hx"] - np.abs(X), p["hy"] - np.abs(Y)), p["hz"] - np.abs(Z))
        r = max(p.get("lid", 0.8), 0.05) * p["hz"]
        lid = np.minimum(np.minimum(p["hx"] - np.abs(X), r - np.sqrt((Y - p["hy"]) ** 2 + Z * Z)), Y - p["hy"] + r)
        return np.maximum(body, lid)
    r = p["r"]
    if kind == "barrel":
        r = r * (1 + p.get("bulge", 0.12) * (1 - np.clip(Y / max(p["hy"], 1), -1, 1) ** 2))
    return np.minimum(r - np.sqrt(X * X + Z * Z), p["hy"] - np.abs(Y))


def init_params(kind, job, fr):
    k = next(k for k, v in job.views.items() if not v.mirror_of)
    m = load_mask(job, k)
    ys, xs = np.nonzero(m)
    w, h = (xs.max() - xs.min()) / fr[k]["sy"], (ys.max() - ys.min()) / fr[k]["sy"]
    if kind == "box":
        a = w / (2 * np.sqrt(2))
        return dict(hx=a, hz=a, hy=0.45 * h)
    if kind == "chest":
        a = w / (2 * np.sqrt(2))
        return dict(hx=a * 1.2, hz=a * 0.8, hy=0.3 * h, lid=0.8)
    return dict(r=w / 2 * 0.95, hy=0.45 * h, bulge=0.12)


def fit(job, fr, grid, kind=None, coarse=2):
    """Pass prioren og visningens vipp/dreining/nullpunkt til silhuetten. Gir (parametre, prior-felt på grid, fr)."""
    kind = kind or job.opt.get("prior", "box")
    keys = [k for k, v in job.views.items() if not v.mirror_of]
    # grovt rutenett for søket
    sub = (slice(None, None, coarse),) * 3
    Xc, Yc, Zc = grid.X[sub], grid.Y[sub], grid.Z[sub]
    vox = grid.vox * coarse
    masks = {k: align.small_mask(job, k, vox) for k in keys}
    p = init_params(kind, job, fr)
    o = {k: dict(fr[k]) for k in keys}

    def score(p_, o_):
        inside = prior_inside(kind, p_, Xc, Yc, Zc) > 0
        from scipy import ndimage as ndi
        sh = inside & ~ndi.binary_erosion(inside)
        idx = np.nonzero(sh.ravel())[0]
        pts = (Xc.ravel()[idx], Yc.ravel()[idx], Zc.ravel()[idx])
        s = 0.0
        for k in keys:
            occ = align.occupancy(job, k, o_[k], pts, masks[k].shape, vox)
            s += align.iou(occ, masks[k])
        return s / len(keys)
    best = score(p, o)
    ranges = {("p", n): (0.3 * v, 2.0) for n, v in p.items() if n not in ("bulge", "lid")}
    ranges[("p", "bulge")] = (0.1, 0.01)
    ranges[("p", "lid")] = (0.4, 0.02)
    for k in keys:  # tegningen er sjelden nøyaktig 45°/20° og ikke helt ortografisk
        o[k].setdefault("persp", 0.0)
        ranges[(k, "pitch")] = (float(job.opt.get("pitch_range", 15.0)), 1.0)
        ranges[(k, "yaw")] = (float(job.opt.get("yaw_range", 30.0)), 1.0)
        ranges[(k, "c")] = (40.0, 2.0)
        ranges[(k, "top")] = (40.0, 2.0)
        ranges[(k, "persp")] = (0.12, 0.01)
    for rnd in range(2):
        for (who, name), (rng, fine) in ranges.items():
            if (who == "p" and name not in p) or (who != "p" and name not in o[who]):
                continue
            step = rng / 2 if rnd == 0 else rng / 8
            while step >= fine:
                improved = True
                while improved:
                    improved = False
                    for d in (-step, step):
                        p2 = dict(p); o2 = {k: dict(v) for k, v in o.items()}
                        if who == "p":
                            p2[name] = p[name] + d
                            if (p2[name] <= 2 and name not in ("bulge", "lid")) or (name == "lid" and not 0.05 <= p2[name] <= 1.0):
                                continue
                        else:
                            o2[who][name] = o[who][name] + d
                        s = score(p2, o2)
                        if s > best + 1e-5:
                            best, p, o = s, p2, o2
                            improved = True
                step /= 2
    for k in keys:
        fr[k] = o[k]
        for j, w in job.views.items():
            if w.mirror_of == k:
                fr[j] = geom.mirror_frame(o[k], job.TW)
    print("prior %s: %s  IoU %.3f  pitch %.1f yaw %.1f persp %.2f" % (kind, {n: round(float(v), 2) for n, v in p.items()}, best,
          fr[keys[0]].get("pitch", 0), fr[keys[0]].get("yaw", 0), fr[keys[0]].get("persp", 0)))
    field = prior_inside(kind, p, grid.X, grid.Y, grid.Z).astype(np.float32)
    view = {k: dict(pitch=round(fr[k].get("pitch", 0), 1), yaw=round(45 + fr[k].get("yaw", 0), 1), persp=round(fr[k].get("persp", 0), 3)) for k in keys}
    return dict(kind=kind, params={n: round(float(v), 2) for n, v in p.items()}, iou=round(best, 4), view=view), field, fr


def auto_tile(job, mean_rgb, prefer=("wood", "plank", "oak", "timber", "board", "tre", "bord")):
    """Den malte flisen (assets/textures/painted/*/albedo.png) med snittfargen nærmest `mean_rgb` (lineær), blant
    flisene med et av `prefer`-ordene i navnet når slike finnes (en kiste får plank, ikke brynje)."""
    import glob
    import os
    import cv2
    best = None
    paths = glob.glob(os.path.join(job.root, "assets/textures/painted/*/albedo.png"))
    pref = [p for p in paths if any(w in os.path.basename(os.path.dirname(p)).lower() for w in prefer)]
    prefer_list = job.opt.get("tile_prefer")
    if prefer_list:
        pref = [p for p in paths if any(w in os.path.basename(os.path.dirname(p)).lower() for w in prefer_list)]
    for path in (pref or paths):
        im = cv2.imread(path)
        if im is None:
            continue
        small = cv2.resize(im, (64, 64), interpolation=cv2.INTER_AREA)[:, :, ::-1].astype(np.float32) / 255
        d = np.linalg.norm((small ** 2.2).reshape(-1, 3).mean(0) - mean_rgb)
        if best is None or d < best[0]:
            best = (d, path)
    return best[1] if best else None
