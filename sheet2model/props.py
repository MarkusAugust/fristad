"""Gjenstander (`prop1`, `prop2`): formen kommer fra en prior som passes til silhuetten, ikke fra silhuettene alene.

`opt.size` låser priorens mål til spillets (meter per parameter, f.eks. `{"hx": 0.3, "hy": 0.23, "hz": 0.54}`); da
søkes bare kameraet per rute (vipp, dreining, perspektiv, nullpunkt og målestokk `sy`). `opt.init` gir startmål i
piksler som holdes innenfor ±`param_range`, `opt.fix` låser enkeltmål i piksler.

Priorer (`opt.prior`): `box` (halvmål hx, hy, hz), `chest` (boks med halvellipse-lokk over hele dybden, `lid` = lokkets høyde som andel av hx), `cylinder` (radius r, halvhøyde hy), `barrel` (sylinder med
bul: r(y) = r·(1 + bulge·(1 − (y/hy)²)), søkeområdet `bulge_range`), `cylinder_x` (liggende sylinder langs x: r, hx),
`ring` (kum med hull: r, hy, indre radius `ring_inner` · r), `cone_x` (kjegle langs x: r0, r1, hx), `gable` (saltak: prisme med mønet langs x: hx, hy, hz), `revolve`
(kontrollradier r0…r5 jevnt fra −hy til hy). `revolve` (radiusprofil fra silhuetten) og en glb-fil som prior er
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
    if kind == "chest":
        # boks med halvsylinder-lokk: hengslet langs z, så låsflaten (den lange sida) vender mot ±x som i
        # prop-rutene (forsiden til høyre i bildet er +x); lokkets radius er `lid` · hx
        # lokket er en halv ellipse over hele dybden (halvakse hx) med høyden `lid` · hx
        body = np.minimum(np.minimum(p["hx"] - np.abs(X), p["hy"] - np.abs(Y)), p["hz"] - np.abs(Z))
        h = max(p.get("lid", 0.8), 0.05) * p["hx"]
        e = 1 - np.sqrt((X / p["hx"]) ** 2 + ((Y - p["hy"]) / h) ** 2)
        lid = np.minimum(np.minimum(p["hz"] - np.abs(Z), e * min(p["hx"], h)), Y - p["hy"] + h)
        return np.maximum(body, lid)
    if kind == "revolve":  # dreiekropp med kontrollradier jevnt fra −hy til hy (lineært imellom)
        keys = sorted(k for k in p if k.startswith("r") and k[1:].isdigit())
        rs = np.array([p[k] for k in keys], np.float32)
        rr = np.interp(np.clip(Y / max(p["hy"], 1), -1, 1), np.linspace(-1, 1, len(rs)), rs)
        return np.minimum(rr - np.sqrt(X * X + Z * Z), p["hy"] - np.abs(Y))
    if kind == "gable":  # saltak: trekantet prisme med mønet langs x i y = hy, takskjegget i y = −hy, halv bredde hz der
        half = p["hz"] * np.clip((p["hy"] - Y) / max(2 * p["hy"], 1), 0, 1)
        return np.minimum(np.minimum(p["hx"] - np.abs(X), p["hy"] - np.abs(Y)), half - np.abs(Z))
    if kind == "cone_x":  # kjegle langs x: radius r0 ved −hx, r1 ved +hx (hornet på ambolten)
        t = np.clip((X / max(p["hx"], 1) + 1) / 2, 0, 1)
        rr = p["r0"] + (p["r1"] - p["r0"]) * t
        return np.minimum(rr - np.sqrt(Y * Y + Z * Z), p["hx"] - np.abs(X))
    r = p["r"]
    if kind == "cylinder_x":  # liggende sylinder langs x (brønnvinda)
        return np.minimum(r - np.sqrt(Y * Y + Z * Z), p["hx"] - np.abs(X))
    if kind == "ring":  # kum: sylinder med hull (indre radius `ri` · r, fra opt.ring_inner, ikke søkt)
        rho = np.sqrt(X * X + Z * Z)
        return np.minimum(np.minimum(r - rho, rho - p["ri"] * r), p["hy"] - np.abs(Y))
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
        return dict(hx=a * 0.8, hz=a * 1.2, hy=0.3 * h, lid=float(job.opt.get("lid", 0.8)))
    if kind == "cylinder_x":
        return dict(r=0.4 * h, hx=w / (2 * 0.70710678) * 0.9)
    if kind == "cone_x":
        return dict(r0=0.4 * h, r1=0.15 * h, hx=w / (2 * 0.70710678) * 0.9)
    if kind == "gable":
        a = w / (2 * np.sqrt(2))
        return dict(hx=a * 1.2, hy=0.35 * h, hz=a * 0.9)
    if kind == "revolve":
        d = dict(hy=0.45 * h)
        for i in range(int(job.opt.get("revolve_n", 6))):
            d[f"r{i}"] = w / 2 * 0.8
        return d
    if kind == "ring":
        return dict(r=w / 2 * 0.95, hy=0.45 * h, ri=float(job.opt.get("ring_inner", 0.7)))
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
    init = {n: float(v) for n, v in (job.opt.get("init") or {}).items()}  # startmål fra oppslaget (piksler)
    fixp = {n: float(v) for n, v in (job.opt.get("fix") or {}).items()}  # mål som ikke søkes
    size = job.opt.get("size") or {}  # spillets mål i meter: prioren låses, og rutene får hver sin skala (sy)
    for n, v in size.items():
        fixp[n] = float(v) / job.scale if n not in ("lid", "bulge", "ri") else float(v)
    p.update(init); p.update(fixp)
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
    ranges = {("p", n): (0.3 * v, 2.0) for n, v in p.items() if n not in ("bulge", "lid", "ri") and n not in fixp}
    bounds = {}
    pr_ = float(job.opt.get("param_range", 0.3))
    for n, v in init.items():  # oppgitte startmål holdes innenfor ±param_range
        if n in ranges or ("p", n) in ranges:
            bounds[("p", n)] = (v * (1 - pr_), v * (1 + pr_))
    if "bulge" in p and "bulge" not in fixp:
        ranges[("p", "bulge")] = (float(job.opt.get("bulge_range", 0.1)), 0.01)
    if "lid" in job.opt or "lid" in fixp:  # lokkets radius gitt i oppslaget (som andel av hx): søkes ikke
        ranges.pop(("p", "lid"), None)
    elif kind == "chest":
        ranges[("p", "lid")] = (0.4, 0.02)
    # vippen og perspektivet holdes i et vindu rundt tegningens (ellers bytter søket vipp mot perspektiv)
    fixed = job.opt.get("fix_view") or {}  # deler av en sammensatt gjenstand: kameraet fra hoveddelen
    for k in keys:  # tegningen er sjelden nøyaktig 45°/20° og ikke helt ortografisk
        o[k].setdefault("persp", 0.0)
        if k in fixed:
            o[k].update({n: float(v) for n, v in fixed[k].items()})
        else:
            pr, sr = float(job.opt.get("pitch_range", 8.0)), float(job.opt.get("persp_range", 0.06))
            ranges[(k, "pitch")] = (pr, 1.0)
            ranges[(k, "yaw")] = (float(job.opt.get("yaw_range", 30.0)), 1.0)
            ranges[(k, "persp")] = (sr, 0.01)
            bounds[(k, "pitch")] = (o[k]["pitch"] - pr, o[k]["pitch"] + pr)
            bounds[(k, "persp")] = (-sr, sr)
            yr = float(job.opt.get("yaw_range", 30.0))
            bounds[(k, "yaw")] = (o[k].get("yaw", 0.0) - yr, o[k].get("yaw", 0.0) + yr)
        ranges[(k, "c")] = (40.0, 2.0)
        ranges[(k, "top")] = (40.0, 2.0)
        if size and k not in fixed:  # målene er låst: tegningens målestokk finnes per rute
            ranges[(k, "sy")] = (0.3, 0.01)
            bounds[(k, "sy")] = (0.3, 3.0)
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
                            if (p2[name] <= 2 and name not in ("bulge", "lid")) or (name == "lid" and not 0.05 <= p2[name] <= 1.0) or (name == "bulge" and p2[name] < -0.3):
                                continue
                            lo_hi = bounds.get(("p", name))
                            if lo_hi and not lo_hi[0] <= p2[name] <= lo_hi[1]:
                                continue
                        else:
                            o2[who][name] = o[who][name] + d
                            lo_hi = bounds.get((who, name))
                            if lo_hi and not lo_hi[0] <= o2[who][name] <= lo_hi[1]:
                                continue
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
    print("prior %s: %s  IoU %.3f  pitch %.1f yaw %.1f persp %.2f sy %s" % (kind, {n: round(float(v), 2) for n, v in p.items()}, best,
          fr[keys[0]].get("pitch", 0), fr[keys[0]].get("yaw", 0), fr[keys[0]].get("persp", 0), [round(fr[k]["sy"], 3) for k in keys]))
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
