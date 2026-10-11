"""Innretting, kalibrering av relativ dybde mot silhuettene, sammensmelting i et volum, marching cubes."""
import json
import os

import numpy as np

from . import job as J


def load_view(k):
    import cv2
    m = cv2.imread(J.p(f"{k}_mask.png"), 0).astype(np.float32) / 255
    d = np.load(J.p(f"{k}_disp.npy"))
    return m, d


def frame():
    """Innretting: samme topp og bunn (ortografisk, vannrett kamera gir samme høyeste punkt i alle
    visninger), sentrum for x fra forfra/bakfra og z fra midten av profilene."""
    out = {}
    for k in J.VIEWS:
        m, _ = load_view(k)
        ys, xs = np.nonzero(m > 0.5)
        out[k] = dict(top=int(ys.min()), bot=int(ys.max()))
    f = out["front"]
    for k in J.VIEWS:
        o = out[k]
        o["sy"] = (o["bot"] - o["top"]) / (f["bot"] - f["top"])  # bildepiksler per verdenspiksel
    if J.OPT.get("same_scale"):  # rutene er tegnet i samme målestokk; bunnen (halskuttet) er upålitelig
        for k in J.VIEWS:
            out[k]["sy"] = 1.0
    if J.OPT.get("align") == "skin":
        align_skin(out)
    # vannrett sentrum: midten av silhuetten over ørene (øverste 45 %)
    for k in J.VIEWS:
        m, _ = load_view(k)
        o = out[k]
        r0, r1 = o["top"] + 40, o["top"] + int(0.45 * (o["bot"] - o["top"]))
        cols = np.nonzero((m[r0:r1] > 0.5).any(0))[0]
        if k in ("front", "back"):
            o["c"] = 0.5 * (cols.min() + cols.max())
        else:
            o["c"] = 0.5 * (cols.min() + cols.max())
    return out


def align_skin(out):
    """Høyde og skala per visning fra hudlinjene: andelen hud per rad (pannen under skautet eller håret, ansiktet,
    halsen) skal ligge likt i profilene som forfra. Bildegeneratoren tegner ikke issen og halskuttet like høyt i
    alle rutene, så topp og bunn alene gir feil høyde på øynene (dobbelt ansikt i mellomvinkelen)."""
    import cv2

    def signal(k):
        rgb = cv2.imread(J.p(f"{k}_rgb.png"))[:, :, ::-1]
        m = cv2.imread(J.p(f"{k}_mask.png"), 0) > 127
        hsv = cv2.cvtColor(rgb, cv2.COLOR_RGB2HSV).astype(np.float32)
        h, s_, v_ = hsv[..., 0] * 2, hsv[..., 1], hsv[..., 2]
        sk = (h > 2) & (h < 28) & (s_ > 45) & (s_ < 170) & (v_ > 120) & m
        return sk.sum(1) / np.maximum(m.sum(1), 1)

    f = out["front"]
    sf = signal("front")
    rows = np.arange(f["top"], f["bot"])
    for k in J.VIEWS:
        if k in ("front", "back") or J.VIEWS[k].get("mirror_of"):
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
        print("innretting", k, "korr %.3f skala %.3f topp %d (var %.3f, %d)" % (best[0], best[1], best[2], out[k]["sy"], out[k]["top"]))
        out[k]["sy"], out[k]["top"] = float(best[1]), int(best[2])
    for k in J.VIEWS:  # speilede visninger følger kilden
        if J.VIEWS[k].get("mirror_of"):
            src = out[J.VIEWS[k]["mirror_of"]]
            out[k]["sy"], out[k]["top"] = src["sy"], src["top"]


def to_img(k, fr, X, Y, Z):
    """Verdenspunkt (front-piksler, y opp fra toppen) -> (u, v) i visning k."""
    v = J.VIEWS[k]
    r = np.array(v["right"], float)
    a = X * r[0] + Y * 0 + Z * r[2]
    o = fr[k]
    u = o["c"] + a * o["sy"]
    vv = o["top"] + (-Y) * o["sy"]
    return u, vv


def sample(img, u, v, fill=0.0):
    import cv2
    n = u.size
    w = 4096
    h = (n + w - 1) // w
    uu = np.zeros(h * w, np.float32); uu[:n] = u.ravel()
    vv = np.zeros(h * w, np.float32); vv[:n] = v.ravel()
    o = cv2.remap(img.astype(np.float32), uu.reshape(h, w), vv.reshape(h, w),
                  cv2.INTER_LINEAR, borderMode=cv2.BORDER_CONSTANT, borderValue=fill)
    return o.ravel()[:n].reshape(u.shape)


def calibrate(fr):
    """Relativ dybde -> metrisk: per rad skal det nærmeste punktet i dybdekartet ligge på ytterkanten
    av silhuetten fra visningene på tvers (profilene for forfra/bakfra og omvendt)."""
    H = fr["front"]["bot"] - fr["front"]["top"]
    ys = np.arange(0, H, 2.0)
    ext = {}
    for k in J.VIEWS:
        m, _ = load_view(k)
        lo, hi = [], []
        for y in ys:
            vv = int(round(fr[k]["top"] + y * fr[k]["sy"]))
            cols = np.nonzero(m[min(vv, J.TH - 1)] > 0.5)[0]
            if len(cols) < 4:
                lo.append(np.nan); hi.append(np.nan); continue
            a0 = (cols.min() - fr[k]["c"]) / fr[k]["sy"]
            a1 = (cols.max() - fr[k]["c"]) / fr[k]["sy"]
            lo.append(a0); hi.append(a1)
        ext[k] = (np.array(lo), np.array(hi))
    # ytterkanter i verden per rad (snittet av visningene som finnes)
    inf = np.full(len(ys), np.nan)
    def fmin(*a):
        a = [x for x in a if x is not None]
        return np.fmin.reduce(a) if len(a) > 1 else a[0]
    def fmax(*a):
        a = [x for x in a if x is not None]
        return np.fmax.reduce(a) if len(a) > 1 else a[0]
    g = lambda k, i, sgn=1: (sgn * ext[k][i]) if k in ext else None
    zmax = fmin(g("right", 1), g("left", 0, -1))
    zmin = fmax(g("right", 0), g("left", 1, -1))
    xmax = fmin(g("front", 1), g("back", 0, -1))
    xmin = fmax(g("front", 0), g("back", 1, -1))
    target = {"front": zmax, "back": -zmin, "right": -xmin, "left": xmax}  # avstand fram mot kameraet
    data = {}
    for k in J.VIEWS:
        m, d = load_view(k)
        dm = []
        for y in ys:
            vv = int(round(fr[k]["top"] + y * fr[k]["sy"]))
            row = d[min(vv, J.TH - 1)][m[min(vv, J.TH - 1)] > 0.5]
            dm.append(np.percentile(row, 98) if len(row) > 4 else np.nan)
        dm = np.array(dm)
        t = target[k]
        ok = np.isfinite(dm) & np.isfinite(t) & (ys > 0.04 * H) & (ys < 0.85 * H)
        data[k] = (dm[ok], t[ok])
    # felles skala a for alle visningene (samme modell, samme scene), egen forskyvning b per visning
    keys = list(J.VIEWS)
    rows, rhs = [], []
    for i, k in enumerate(keys):
        x0, y0 = data[k]
        A = np.zeros((len(x0), 1 + len(keys)))
        A[:, 0] = x0
        A[:, 1 + i] = 1
        rows.append(A); rhs.append(y0)
    A = np.concatenate(rows); y = np.concatenate(rhs)
    if J.OPT.get("scale_per_view"):  # egen skala per visning (hele figurer: relieffet varierer mellom visningene)
        A2 = np.zeros((len(y), 2 * len(keys)))
        off = 0
        for i, k in enumerate(keys):
            n = len(data[k][0])
            A2[off:off + n, 2 * i] = data[k][0]; A2[off:off + n, 2 * i + 1] = 1
            off += n
        w = np.ones(len(y))
        for _ in range(8):
            sol2 = np.linalg.lstsq(A2 * w[:, None], y * w, rcond=None)[0]
            r = y - A2 @ sol2
            s_ = np.median(np.abs(r)) * 1.5 + 1e-3
            w = 1 / np.maximum(1, np.abs(r) / s_)
        cal, off = {}, 0
        for i, k in enumerate(keys):
            n = len(data[k][0])
            cal[k] = (float(sol2[2 * i]), float(sol2[2 * i + 1]))
            print(k, "a=%.2f b=%.1f  avvik median %.1f px" % (cal[k][0], cal[k][1], np.median(np.abs(r[off:off + n]))))
            off += n
        return cal
    w = np.ones(len(y))
    for _ in range(8):
        sol = np.linalg.lstsq(A * w[:, None], y * w, rcond=None)[0]
        r = y - A @ sol
        s = np.median(np.abs(r)) * 1.5 + 1e-3
        w = 1 / np.maximum(1, np.abs(r) / s)
    cal = {}
    off = 0
    for i, k in enumerate(keys):
        n = len(data[k][0])
        cal[k] = (float(sol[0]), float(sol[1 + i]))
        print(k, "a=%.2f b=%.1f  avvik median %.1f px" % (sol[0], sol[1 + i], np.median(np.abs(r[off:off + n]))))
        off += n
    return cal


def fuse(vox=3.0):
    import cv2
    from scipy import ndimage as ndi
    from skimage import measure
    fr = frame()
    cal = calibrate(fr)
    json.dump(dict(frame=fr, cal=cal), open(J.p("frame.json"), "w"), indent=1)
    H = fr["front"]["bot"] - fr["front"]["top"]
    # rutenettet dekker silhuettene (med marg): bredden fra forfra/bakfra, dybden fra profilene
    def span(keys, sign):
        lo, hi = [], []
        for k in keys:
            if k not in J.VIEWS:
                continue
            m, _ = load_view(k)
            cols = np.nonzero((m > 0.5).any(0))[0]
            a0, a1 = (cols.min() - fr[k]["c"]) / fr[k]["sy"], (cols.max() - fr[k]["c"]) / fr[k]["sy"]
            if sign(k) < 0:
                a0, a1 = -a1, -a0
            lo.append(a0); hi.append(a1)
        return min(lo) - 24, max(hi) + 24
    xr = span(("front", "back"), lambda k: 1 if k == "front" else -1)
    zr = span(("right", "left"), lambda k: 1 if k == "right" else -1)
    xs = np.arange(xr[0], xr[1], vox)
    ys = np.arange(-H - 10, 10, vox)
    zs = np.arange(zr[0], zr[1], vox)
    X, Y, Z = np.meshgrid(xs, ys, zs, indexing="ij")
    F = np.full(X.shape, 1e3, np.float32)
    for k, v in J.VIEWS.items():
        m, d = load_view(k)
        # silhuetten som signert avstand i piksler (positiv inni)
        mb = m > 0.5
        sd = ndi.distance_transform_edt(mb) - ndi.distance_transform_edt(~mb)
        # dybden: avstand fram mot kameraet = a*disp + b; fyll utenfor med kanten
        a, b = cal[k]
        depth = a * d + b
        # nær silhuettkanten er dybden upålitelig (der tar silhuetten over): slakk som toner inn over R px
        R = J.OPT.get("edge_slack", 28.0)  # tynne lemmer trenger liten slakk, ellers blir de kantete bokser
        dist = ndi.distance_transform_edt(mb)
        depth = depth + np.maximum(0, R - dist) * J.OPT.get("edge_gain", 1.5)
        if k == "back" and "back_cut" in J.OPT:  # halskuttet sett bakfra er ikke en flate på hodet
            depth[int(fr[k]["top"] + J.OPT["back_cut"] * (fr[k]["bot"] - fr[k]["top"])):] += 1e3
        depth = np.where(mb, depth, -1e3)
        u, vv = to_img(k, fr, X.ravel(), Y.ravel(), Z.ravel())
        s_sil = sample(sd, u, vv, fill=-500).reshape(X.shape) / fr[k]["sy"]
        cam = np.array(v["cam"], float)
        t = X * cam[0] + Y * cam[1] + Z * cam[2]  # hvor langt fram mot kameraet punktet er
        surf = sample(depth, u, vv, fill=-1e3).reshape(X.shape)
        s_dep = surf + J.OPT.get("depth_margin", 0.0) - t  # positiv bak flaten (margin: tynne ting foran
        # kroppen, som et økseblad, får dybdemodellen ofte lagt på kroppen bak; silhuettene og ellipsen tar resten)
        sv = np.minimum(s_sil, s_dep)
        if v.get("mirror_of"):
            # den speilede profilen er en antakelse om den andre siden: den får bare skjære i sin egen halvdel,
            # ellers skjærer den bort det den ene hånda holder (redskapet er tatt ut av den)
            own = (X * cam[0] + Z * cam[2]) > -12
            sv = np.where(own, sv, 1e3)
        F = np.minimum(F, sv)
        print(k, "ferdig")
    if J.OPT.get("elliptic"):
        F = np.minimum(F, elliptic(X, Y, Z, fr, J.OPT.get("elliptic_p", 2.5)) + J.OPT.get("elliptic_tol", 4.0))
    if "neck" in J.OPT:
        F = clean_neck(F, X, Y, Z, fr, J.OPT["neck"])
    # toppen: ingen visning ser ovenfra, så kantene der dybdeflatene skjærer hverandre glattes til en kuppel
    if J.OPT.get("dome", 0.14) > 0:
        wtop = np.clip((Y + J.OPT.get("dome", 0.14) * H) / 40.0, 0, 1)
        F = F * (1 - wtop) + ndi.gaussian_filter(F, 3.0) * wtop
    F = ndi.gaussian_filter(F, 0.8)
    np.save(J.p("field.npy"), F)
    verts, faces, normals, _ = measure.marching_cubes(F, 0.0, spacing=(vox, vox, vox), step_size=1)
    verts += np.array([xs[0], ys[0], zs[0]])
    # behold den største biten
    print("trekanter", len(faces))
    np.savez(J.p("mesh.npz"), v=verts.astype(np.float32), f=faces.astype(np.int32))
    np.savez(J.p("grid.npz"), x0=xs[0], y0=ys[0], z0=zs[0], vox=vox)


def run_bounds(mb):
    """For hver piksel i masken: venstre og høyre kant (kolonne) av biten i raden den ligger i."""
    W = mb.shape[1]
    idx = np.broadcast_to(np.arange(W), mb.shape)
    left = np.maximum.accumulate(np.where(~mb, idx, -1), axis=1) + 1
    ridx = np.where(~mb[:, ::-1], idx, -1)
    right = (W - 1 - np.maximum.accumulate(ridx, axis=1))[:, ::-1] - 1
    return left.astype(np.float32), right.astype(np.float32)


def elliptic(X, Y, Z, fr, pw=2.5):
    """Tverrsnitt som superellipse: bredden fra biten i raden forfra, dybden fra biten i raden i profilen.
    Dybdemodellen gir lite relieff på en hel figur, så uten dette blir armer og bein firkantede bokser."""
    from scipy import ndimage as ndi
    out = np.full(X.shape, 1e3, np.float32)
    pairs = [("front", "right")]
    for fk, pk in pairs:
        if fk not in J.VIEWS or pk not in J.VIEWS:
            continue
        import cv2
        mf = cv2.imread(J.p(f"{fk}_mask.png"), 0) > 127
        mp = cv2.imread(J.p(f"{pk}_mask.png"), 0) > 127
        lf, rf = run_bounds(mf)
        lp, rp = run_bounds(mp)
        uf, vf = to_img(fk, fr, X.ravel(), Y.ravel(), Z.ravel())
        up, vp = to_img(pk, fr, X.ravel(), Y.ravel(), Z.ravel())
        def near(img, u, v):
            ui = np.clip(np.round(u).astype(int), 0, img.shape[1] - 1)
            vi = np.clip(np.round(v).astype(int), 0, img.shape[0] - 1)
            return img[vi, ui]
        inside = near(mf, uf, vf) & near(mp, up, vp)
        of, op = fr[fk], fr[pk]
        xa = (near(lf, uf, vf) - of["c"]) / of["sy"]; xb = (near(rf, uf, vf) + 1 - of["c"]) / of["sy"]
        za = (near(lp, up, vp) - op["c"]) / op["sy"]; zb = (near(rp, up, vp) + 1 - op["c"]) / op["sy"]
        xc, rx = (xa + xb) / 2, np.maximum((xb - xa) / 2, 1)
        zc, rz = (za + zb) / 2, np.maximum((zb - za) / 2, 1)
        e = (np.abs(X.ravel() - xc) / rx) ** pw + (np.abs(Z.ravel() - zc) / rz) ** pw
        val = (1 - e ** (1 / pw)) * np.minimum(rx, rz)
        val = np.where(inside, val, 1e3)  # utenfor tar silhuettene seg av det
        out = np.minimum(out, val.reshape(X.shape).astype(np.float32))
    return out


def clean_neck(F, X, Y, Z, fr, o_):
    """Rent, skrått halskutt og en elliptisk hals bak skjegget (der forbildet ikke er enig med seg selv)."""
    # rent halskutt: skrått plan langs undersiden av halsen i profilen (skjegget henger foran og under)
    o = fr["right"]
    def wpt(u, v):
        return (u - o["c"]) / o["sy"], -(v - o["top"]) / o["sy"]
    z1, y1 = wpt(*o_["plane"][0]); z2, y2 = wpt(*o_["plane"][1])
    k_ = (y2 - y1) / (z2 - z1)
    plane = (Y - (y1 + k_ * (Z - z1))) / np.sqrt(1 + k_ * k_) - o_["plane_up"]  # positiv over planet (litt over den ujevne kanten)
    # halsen under nakkehåret: forbildet er ikke enig med seg selv (skjegget forfra dekker halsen, så
    # silhuetten forfra skjærer den i flisete biter). Erstatt den med en ren, elliptisk hals målt bakfra
    # og fra profilen, bak skjegget.
    ytop = -(o_["top_v"] - fr["front"]["top"])
    hx = o_["half_x"] * (1 + o_.get("flare", 0.25) * np.clip((ytop - Y) / 150.0, 0, 1))  # trapes mot skuldrene
    e = np.sqrt((X / hx) ** 2 + ((Z - o_["z0"]) / o_["half_z"]) ** 2)
    neck = np.minimum((1 - e) * o_["half_z"], (ytop + 120) - Y)
    # halsen skal ikke stikke ut under skjegget foran: bunn ved v = 905 foran, slipper bakover
    y905 = -(o_["front_v"] - fr["front"]["top"])
    neck = np.minimum(neck, (Y - y905) + np.maximum(0, -20 - Z) * 1.5)
    region = np.clip((ytop - Y) / 20.0, 0, 1) * np.clip((o_["beard_z"] - Z) / 20.0, 0, 1)
    F = F * (1 - region) + np.maximum(neck, np.minimum(F, -1)) * region
    F = np.maximum(F, np.where(Y < ytop + 120, neck, -1e3))
    F = np.minimum(F, plane)
    return F


def look(out=None, yaws=(0, -90, 90, 180, -45), pitch=(0, 0, 0, 0, 20)):
    """Rask kontroll uten Blender: skyggelagt dybde av feltet fra flere vinkler."""
    import cv2
    from scipy import ndimage as ndi
    F = np.load(J.p("field.npy"))
    ims = []
    for yaw, pt in zip(yaws, pitch):
        G = F
        if yaw:
            G = ndi.rotate(G, yaw, axes=(0, 2), reshape=False, order=1, cval=-50)  # om y-aksen
        if pt:
            G = ndi.rotate(G, pt, axes=(1, 2), reshape=False, order=1, cval=-50)
        inside = G > 0
        # kamera ved +z: første innside fra høy z
        idx = inside[:, :, ::-1].argmax(2)
        hit = inside.any(2)
        dep = (G.shape[2] - idx).astype(np.float32)
        gy, gx = np.gradient(ndi.gaussian_filter(dep, 0.7))
        n = np.stack([-gx, -gy, np.ones_like(dep)], -1)
        n /= np.linalg.norm(n, axis=-1, keepdims=True)
        L = np.array([-0.4, 0.5, 0.75]); L /= np.linalg.norm(L)
        sh = np.clip(n @ L, 0, 1) * 0.8 + 0.2
        img = np.where(hit, sh, 0.5)
        img = img.T[::-1]  # x -> kolonner, y opp
        ims.append((img * 255).astype(np.uint8))
    a = np.hstack(ims)
    out = out or os.path.join(J.SHOTS, "look.png")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    cv2.imwrite(out, a)


