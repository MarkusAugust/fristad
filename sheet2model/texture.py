"""Steg 5: teksturen, helt i numpy (ingen Cycles).

1. uv-kartet rasteres til et id-kart (hvilken trekant hver teksel ligger i) med cv2, og barysentriske vekter gir
   posisjon og normal per teksel. Margen rundt øyene fylles med nærmeste trekant.
2. Hver visning gir hver teksel en farge: myk dybdetest mot z-bufferen til den høye modellen, vekt etter normalen
   mot kameraet, mykt mot silhuettkanten, og mindre vekt der fargen avviker fra de andre visningene (robust).
3. Blandingen er en screened Poisson-løsning i uv (DCT): gradientene fra den beste visningen per teksel beholdes,
   mens fargen holdes mykt nær den brede blandingen, så det aldri blir sømmer, heller ikke der visningene har
   litt ulikt lys.
4. Et grovt lysestimat (lav frekvens av luminansen mot normalen) dempes, så spillets lys ikke dobles.
5. Flater ingen visning ser (toppen, undersiden) fylles med push-pull, eller med fliser når de er gitt.
6. Normalkartet lages fra den høye modellen (nærmeste hjørnenormal) i tangentrommet til den lave.
"""
import json
import time

import numpy as np

from . import geom
from .sheet import load_mask, load_rgb


# ---------------------------------------------------------------- uv -> teksler
class Texels:
    def __init__(self, tex, ids, bary, pos, nrm, cov, tri_part=None):
        self.tex, self.ids, self.bary, self.pos, self.nrm, self.cov = tex, ids, bary, pos, nrm, cov
        self.part = None if tri_part is None else np.where(ids >= 0, tri_part[np.maximum(ids, 0)], -1)


def vertex_normals(v, f):
    n = np.cross(v[f[:, 1]] - v[f[:, 0]], v[f[:, 2]] - v[f[:, 0]])
    vn = np.zeros_like(v)
    for i in range(3):
        np.add.at(vn, f[:, i], n)
    return vn / (np.linalg.norm(vn, axis=1, keepdims=True) + 1e-12)


def rasterize(v, f, uv, tex, pad=6, tri_part=None):
    """Id-kart, barysentriske vekter, posisjon og normal per teksel. Rad 0 øverst (uv v = 1)."""
    import cv2
    from scipy import ndimage as ndi
    px = np.empty_like(uv)
    px[..., 0] = uv[..., 0] * tex
    px[..., 1] = (1 - uv[..., 1]) * tex
    ids = np.full((tex, tex), -1, np.int32)
    # id-kartet: trekantene fylles med id+1 kodet i tre byte
    img = np.zeros((tex, tex, 3), np.uint8)
    for i in range(len(f)):
        c = i + 1
        pts = np.round(px[i]).astype(np.int32).reshape(1, 3, 2)
        cv2.fillPoly(img, pts, (c & 255, (c >> 8) & 255, (c >> 16) & 255))
        cv2.polylines(img, pts, True, (c & 255, (c >> 8) & 255, (c >> 16) & 255), 1)
    ids = (img[..., 0].astype(np.int32) | (img[..., 1].astype(np.int32) << 8) | (img[..., 2].astype(np.int32) << 16)) - 1
    cov = ids >= 0
    if pad:  # margen: nærmeste trekant
        _, (ri, ci) = ndi.distance_transform_edt(~cov, return_indices=True)
        near = ids[ri, ci]
        dist = ndi.distance_transform_edt(~cov)
        ids = np.where(cov | (dist <= pad), near, -1)
    use = ids >= 0
    yy, xx = np.nonzero(use)
    t = ids[yy, xx]
    P = px[t]  # (n, 3, 2)
    x, y = xx + 0.5, yy + 0.5
    d = (P[:, 1, 0] - P[:, 0, 0]) * (P[:, 2, 1] - P[:, 0, 1]) - (P[:, 2, 0] - P[:, 0, 0]) * (P[:, 1, 1] - P[:, 0, 1])
    d = np.where(np.abs(d) < 1e-9, 1e-9, d)
    l1 = ((x - P[:, 0, 0]) * (P[:, 2, 1] - P[:, 0, 1]) - (P[:, 2, 0] - P[:, 0, 0]) * (y - P[:, 0, 1])) / d
    l2 = ((P[:, 1, 0] - P[:, 0, 0]) * (y - P[:, 0, 1]) - (x - P[:, 0, 0]) * (P[:, 1, 1] - P[:, 0, 1])) / d
    l0 = 1 - l1 - l2
    L = np.clip(np.stack([l0, l1, l2], 1), 0, 1)
    L /= L.sum(1, keepdims=True) + 1e-12
    vn = vertex_normals(v, f)
    pos = np.einsum("nk,nkd->nd", L, v[f[t]])
    nrm = np.einsum("nk,nkd->nd", L, vn[f[t]])
    nrm /= np.linalg.norm(nrm, axis=1, keepdims=True) + 1e-12
    bary = np.zeros((tex, tex, 3), np.float32); bary[yy, xx] = L
    P3 = np.zeros((tex, tex, 3), np.float32); P3[yy, xx] = pos
    N3 = np.zeros((tex, tex, 3), np.float32); N3[yy, xx] = nrm
    return Texels(tex, ids, bary, P3, N3, use, tri_part)


# ---------------------------------------------------------------- kildene
class Source:
    """Et ark som farger en del av tekslene: jobben (visningene, rammen, arbeidsmappa) og en maske over tekslene,
    pluss en transform fra tekselrommet (verden til hele figuren) til arkets eget rom."""

    def __init__(self, job, fr=None, part_mask=None, to_local=None, to_local_dir=None, hi=None, exclude=None):
        self.job = job
        self.fr = fr or json.load(open(job.p("frame.json")))["frame"]
        self.part_mask = part_mask
        self.exclude = exclude or {}  # visning -> maske over piksler som ikke skal farge noe (redskapet)
        self.to_local = to_local or (lambda P: P)
        self.to_local_dir = to_local_dir or (lambda N: N)
        self.hi = hi if hi is not None else np.load(job.p("hi.npy"))


def zbuffer(job, k, o, hiP, shape):
    from scipy import ndimage as ndi
    view = job.views[k]
    u, v = geom.to_img(view, o, hiP[:, 0], hiP[:, 1], hiP[:, 2])
    t = geom.toward(view, o, hiP[:, 0], hiP[:, 1], hiP[:, 2])
    ui = np.clip(np.round(u).astype(int), 0, shape[1] - 1); vi = np.clip(np.round(v).astype(int), 0, shape[0] - 1)
    zb = geom.scatter_max(vi * shape[1] + ui, t, shape[0] * shape[1], fill=-1e9).reshape(shape)
    return ndi.maximum_filter(zb, 5)


def view_samples(job, k, o, P, N, hiP, lowpass=5.0, exclude=None):
    """Én visning sett fra tekslene: synlighet, kantvekt, normal mot kamera, farge (lineær) og lavpassfarge.
    `exclude` er piksler som ikke får farge noe (redskapet i hånda: ellers havner det på forkleet bak når
    redskapet er skilt ut)."""
    import cv2
    from scipy import ndimage as ndi
    view = job.views[k]
    rgb = load_rgb(job, k).astype(np.float32) / 255
    lin = rgb ** 2.2
    low = cv2.GaussianBlur(lin, (0, 0), lowpass)
    m = load_mask(job, k)
    edge = np.clip(ndi.distance_transform_edt(m) / float(job.opt.get("edge_fade", 10.0)), 0, 1).astype(np.float32)
    if exclude is not None and exclude.any():
        # redskapet males over fra kroppen rundt (inpaint) og får liten vekt, så flaten bak det får kroppens
        # tekstur i stedet for en flat flekk fra push-pull
        ex = ndi.binary_dilation(exclude, iterations=4)
        rgb = cv2.inpaint((rgb * 255).astype(np.uint8), (ex * 255).astype(np.uint8), 11, cv2.INPAINT_TELEA).astype(np.float32) / 255
        lin = rgb ** 2.2
        low = cv2.GaussianBlur(lin, (0, 0), lowpass)
        edge *= np.where(ndi.binary_dilation(ex, iterations=2), 0.25, 1.0).astype(np.float32)
    zb = zbuffer(job, k, o, hiP, m.shape)
    u, v = geom.to_img(view, o, P[:, 0], P[:, 1], P[:, 2])
    tp = geom.toward(view, o, P[:, 0], P[:, 1], P[:, 2])
    zs = geom.sample(zb, u, v, fill=-1e9)
    vis = np.clip(1 - (zs - tp - 4) / 6, 0, 1)  # myk dybdetest (piksler)
    cam = geom.basis(view, o)[0]
    Nh = N * np.array([1, 0.35, 1], np.float32); Nh /= np.linalg.norm(Nh, axis=1, keepdims=True) + 1e-9
    ndc = np.clip(Nh @ cam, 0, 1)
    e = geom.sample(edge, u, v, fill=0)
    if job.prop:  # gjenstander: bare flater som vender mot kameraet får farge; resten fra speiling eller flis
        e = e * (np.clip((N @ cam - 0.3) / 0.15, 0, 1)).astype(np.float32)
    col = np.stack([geom.sample(lin[:, :, c], u, v) for c in range(3)], -1)
    cl = np.stack([geom.sample(low[:, :, c], u, v) for c in range(3)], -1)
    return dict(vis=vis, edge=e, ndc=ndc, col=col, low=cl, base=vis * e)


def luma(c):
    return c[..., 0] * 0.2126 + c[..., 1] * 0.7152 + c[..., 2] * 0.0722


# ---------------------------------------------------------------- projeksjonen
def project(job, texels, sources, out_stem="albedo"):
    """Farg tekslene fra kildene og skriv `<stem>.png` (sRGB). Gir rapporttallene for teksturen."""
    import cv2
    t0 = time.time()
    tex = texels.tex
    use = texels.cov
    yy, xx = np.nonzero(use)
    n = len(yy)
    P_all = texels.pos[yy, xx]; N_all = texels.nrm[yy, xx]
    blend = np.zeros((n, 3), np.float32); wsum = np.zeros(n, np.float32)
    winner_w = np.zeros(n, np.float32); hi_best = np.zeros(n, np.float32)
    winner_id = np.full(n, -1, np.int32)
    low_blend = np.zeros((n, 3), np.float32)
    photo = {}
    vid = 0
    for src in sources:
        sel = np.ones(n, bool) if src.part_mask is None else src.part_mask[yy, xx]
        if not sel.any():
            continue
        P = src.to_local(P_all[sel]); N = src.to_local_dir(N_all[sel])
        N = N / (np.linalg.norm(N, axis=1, keepdims=True) + 1e-9)
        hiP = src.hi
        samples = {}
        for k in src.job.texture_views():
            samples[k] = view_samples(src.job, k, src.fr[k], P, N, hiP, exclude=src.exclude.get(k))
        keys = list(samples)
        # robust vekting: fargen i hver visning mot den vektede medianen av de andre
        W0 = {k: samples[k]["base"] * samples[k]["ndc"] ** 2 for k in keys}
        tot = sum(W0.values()) + 1e-6
        mean = sum(W0[k][:, None] * samples[k]["low"] for k in keys) / tot[:, None]
        for _ in range(2):
            Wr = {}
            for k in keys:
                dev = np.linalg.norm(samples[k]["low"] - mean, axis=1)
                Wr[k] = W0[k] * np.exp(-(dev / 0.12) ** 2)
            tot = sum(Wr.values()) + 1e-6
            mean = sum(Wr[k][:, None] * samples[k]["low"] for k in keys) / tot[:, None]
        # fotometrisk feil per par av nabovisninger: teksler begge ser godt
        for i, a in enumerate(keys):
            for b in keys[i + 1:]:
                both = (samples[a]["base"] * samples[a]["ndc"] > 0.3) & (samples[b]["base"] * samples[b]["ndc"] > 0.3)
                if both.sum() > 200:
                    d = np.abs(luma(samples[a]["low"][both]) - luma(samples[b]["low"][both]))
                    photo[f"{src.job.name}:{a}-{b}"] = round(float(np.mean(d)), 4)
        # to frekvensbånd: de lave frekvensene blandes bredt (n·c², robust), detaljene (over ~5 px i ruta)
        # nesten bare fra den beste visningen (n·c¹²), som i tools/reconstruct. Det holder detaljene like skarpe
        # som forbildet; Poisson brukes bare på det lave båndet, mot sømmer der lyset er ulikt.
        idx = np.nonzero(sel)[0]
        for k in keys:
            w = Wr[k]
            low_blend[idx] += w[:, None] * samples[k]["low"]; wsum[idx] += w
            rob = np.exp(-(np.linalg.norm(samples[k]["low"] - mean, axis=1) / 0.12) ** 2)
            wd = samples[k]["base"] * samples[k]["ndc"] ** 12 * rob
            blend[idx] += wd[:, None] * (samples[k]["col"] - samples[k]["low"]); winner_w[idx] += wd
            better = wd > hi_best[idx]
            hi_best[idx[better]] = wd[better]
            winner_id[idx[better]] = vid
            vid += 1
    seen = wsum > 1e-3
    low_blend[seen] /= wsum[seen, None]
    hiw = winner_w > 1e-6
    blend[hiw] /= winner_w[hiw, None]  # detaljbåndet
    coverage = float(seen.mean()) if n else 0.0
    print("dekket %.3f" % coverage)
    B = np.zeros((tex, tex, 3), np.float32); B[yy[seen], xx[seen]] = low_blend[seen]
    Hb = np.zeros((tex, tex, 3), np.float32); Hb[yy[seen], xx[seen]] = blend[seen]
    ID = np.full((tex, tex), -2, np.int32); ID[yy[seen], xx[seen]] = winner_id[seen]
    have = np.zeros((tex, tex), bool); have[yy[seen], xx[seen]] = True
    # lysdemping (bare på det lave båndet, og bare som en glatt funksjon av normalen)
    strength = float(job.opt.get("delight", 0.0))
    if strength > 0 and seen.sum() > 1000:
        gain = delight(low_blend[seen], N_all[seen], strength)
        B[yy[seen], xx[seen]] *= gain[:, None]
    Bf = pushpull(B, have)
    Bf = fill_tiles(job, Bf, have, texels, yy, xx, seen, N_all)
    lam = float(job.opt.get("poisson", 0.5))
    if lam > 0:
        Bf = poisson_blend(Bf, Bf, ID, lam)  # glatter det lave båndet over sømmene mellom visningene
    out = np.where(use[..., None], Bf + Hb, Bf)
    srgb = np.clip(out, 0, 1) ** (1 / 2.2)
    cv2.imwrite(job.p(f"{out_stem}.png"), (srgb[:, :, ::-1] * 255 + 0.5).astype(np.uint8))
    job.time("texture", time.time() - t0)
    face = np.zeros((tex, tex), bool); face[yy[seen], xx[seen]] = N_all[seen, 2] > 0.5
    if texels.part is not None and (texels.part == 1).any():
        face &= texels.part == 1
    return dict(coverage=round(coverage, 4), photometric=photo, sharpness=sharpness(srgb, face))


def sharpness(srgb, mask):
    """Skarphet: variansen til Laplace-operatoren på luminansen (0–255) i masken (ansiktet: teksler som vender
    fram). Dagens tools/reconstruct gir rundt 60 på smedens hode."""
    import cv2
    g = (luma(srgb) * 255).astype(np.float32)
    l = cv2.Laplacian(g, cv2.CV_32F)
    return round(float(l[mask].var()), 1) if mask.sum() > 100 else 0.0


def delight(low, N, strength):
    """Et grovt lysestimat: luminansen (lavpass) som a + b·N over tekslene. Faktoren mean/(a + b·N) demper den
    retningsbestemte delen av lyset, blandet med `strength`."""
    L = luma(low)
    A = np.concatenate([np.ones((len(N), 1)), N], 1)
    w = (L > 0.02).astype(float)
    sol = np.linalg.lstsq(A * w[:, None], L * w, rcond=None)[0]
    est = A @ sol
    gain = np.clip(np.mean(est) / np.maximum(est, 1e-3), 0.7, 1.4)
    return (1 - strength) + strength * gain


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


def fill_tiles(job, Bf, have, texels, yy, xx, seen, N_all):
    """Toppen (normal opp) og undersiden (normal ned) som ingen visning ser: en flis i verdens xz, tonet til
    push-pull-fargen der, i stedet for strukket nabofarge. `opt.tiles = {"up": sti, "down": sti, "cm": 8}`."""
    import cv2
    tiles = job.opt.get("tiles")
    if not tiles:
        return Bf
    if tiles == "auto":  # den malte flisen som ligner mest på gjenstanden, under den
        from .props import auto_tile
        path = auto_tile(job, Bf[yy[seen], xx[seen]].mean(0))
        print("flis under:", path)
        tiles = dict(down=path, cm=float(job.opt.get("tile_cm", 25)))
    out = Bf.copy()
    for key, sign in (("up", 1), ("down", -1)):
        path = tiles.get(key)
        if not path:
            continue
        t = cv2.imread(path)
        if t is None:
            continue
        t = (t[:, :, ::-1].astype(np.float32) / 255) ** 2.2
        unseen = ~seen & (sign * N_all[:, 1] > 0.6)
        if job.prop and key == "down":  # gjenstander: alt ingen rute ser (baksiden, undersiden) får flisen
            unseen = ~seen
        if not unseen.any():
            continue
        P = texels.pos[yy[unseen], xx[unseen]]
        per = float(tiles.get("cm", 8)) / 100 / job.scale  # verdenspiksler per flis
        # flisen legges etter de to aksene flaten ikke vender langs (sidene får den loddrett)
        Nn = np.abs(N_all[unseen])
        A = np.where(Nn[:, [1]] > 0.6, P[:, [0]], np.where(Nn[:, [0]] > Nn[:, [2]], P[:, [2]], P[:, [0]]))[:, 0]
        Bc = np.where(Nn[:, 1] > 0.6, P[:, 2], P[:, 1])
        u = ((A / per) % 1) * (t.shape[1] - 1); v = ((Bc / per) % 1) * (t.shape[0] - 1)
        c = np.stack([geom.sample(t[:, :, i], u, v) for i in range(3)], -1)
        base = Bf[yy[unseen], xx[unseen]]
        c = c * (base.mean(0) / (t.reshape(-1, 3).mean(0) + 1e-6))
        out[yy[unseen], xx[unseen]] = np.clip(c, 0, 1)
    return out


def poisson_blend(Wf, Bf, ID, lam):
    """Screened Poisson med DCT (Neumann-rand): minimer |∇f − g|² + λ|f − B|², der g er gradienten til
    vinnerbildet W innenfor samme vinner, og gradienten til B over sømmene."""
    from scipy.fft import dctn, idctn
    H, W_ = Wf.shape[:2]
    same_x = ID[:, 1:] == ID[:, :-1]
    same_y = ID[1:, :] == ID[:-1, :]
    gx = np.where(same_x[..., None], Wf[:, 1:] - Wf[:, :-1], 0.0)  # over sømmen: ingen sprang
    gy = np.where(same_y[..., None], Wf[1:, :] - Wf[:-1, :], 0.0)
    div = np.zeros_like(Wf)
    div[:, :-1] += gx; div[:, 1:] -= gx
    div[:-1, :] += gy; div[1:, :] -= gy
    ky = 2 - 2 * np.cos(np.pi * np.arange(H) / H)
    kx = 2 - 2 * np.cos(np.pi * np.arange(W_) / W_)
    denom = lam + ky[:, None] + kx[None, :]
    out = np.empty_like(Wf)
    for c in range(3):
        rhs = lam * Bf[..., c] + div[..., c]
        out[..., c] = idctn(dctn(rhs, type=2, norm="ortho") / denom, type=2, norm="ortho")
    return out


# ---------------------------------------------------------------- normalkartet
def normal_map(job, texels, lo, hi_v, hi_n, out_stem="normal", flip=None):
    """Tangentrom-normalkart fra den høye modellen: nærmeste høye hjørnes normal, uttrykt i den lave trekantens
    tangentramme (T langs u, B langs v)."""
    import cv2
    from scipy.spatial import cKDTree
    t0 = time.time()
    v, f, uv = lo
    tex = texels.tex
    yy, xx = np.nonzero(texels.cov)
    P = texels.pos[yy, xx]; N = texels.nrm[yy, xx]
    if flip is not None:
        P = flip(P)
    tree = cKDTree(hi_v)
    _, j = tree.query(P, workers=-1)
    n = hi_n[j]
    n = np.where((n * N).sum(1, keepdims=True) < 0, -n, n)  # samme side som den lave
    t = texels.ids[yy, xx]
    e1 = v[f[t, 1]] - v[f[t, 0]]; e2 = v[f[t, 2]] - v[f[t, 0]]
    d1 = uv[t, 1] - uv[t, 0]; d2 = uv[t, 2] - uv[t, 0]
    det = d1[:, 0] * d2[:, 1] - d2[:, 0] * d1[:, 1]
    det = np.where(np.abs(det) < 1e-12, 1e-12, det)
    T = (e1 * d2[:, 1:2] - e2 * d1[:, 1:2]) / det[:, None]
    T -= N * (T * N).sum(1, keepdims=True)
    T /= np.linalg.norm(T, axis=1, keepdims=True) + 1e-12
    Bt = np.cross(N, T) * np.sign(det)[:, None]
    ts = np.stack([(n * T).sum(1), (n * Bt).sum(1), np.maximum((n * N).sum(1), 0.05)], 1)
    ts /= np.linalg.norm(ts, axis=1, keepdims=True)
    img = np.full((tex, tex, 3), (128, 128, 255), np.uint8)
    img[yy, xx] = np.clip((ts * 0.5 + 0.5) * 255 + 0.5, 0, 255).astype(np.uint8)
    img = cv2.GaussianBlur(img, (0, 0), 0.8)
    cv2.imwrite(job.p(f"{out_stem}.png"), img[:, :, ::-1])
    job.time("normal", time.time() - t0)


# ---------------------------------------------------------------- hele steget (én modell, ett ark)
def build(job):
    lo = np.load(job.p("lo.npz"))
    v, f, uv = lo["v"], lo["f"], lo["uv"]
    texels = rasterize(v, f, uv, job.tex)
    np.savez(job.p("texels.npz"), pos=texels.pos, nrm=texels.nrm, cov=texels.cov, ids=texels.ids)
    rep = project(job, texels, [Source(job)])
    hi = np.load(job.p("hi.npy"))
    hn = np.load(job.p("hi_n.npy"))
    normal_map(job, texels, (v, f, uv), hi, hn)
    rep["face_texels_cm"] = face_density(job, texels, v, f, uv)
    json.dump(rep, open(job.p("texture.json"), "w"), indent=1)
    return rep


def face_density(job, texels, v, f, uv, region=None):
    """Teksler per cm i ansiktet: kvadratroten av uv-areal / verdensareal over trekantene som vender fram og
    ligger i den øverste delen (hodet), omregnet med tekststørrelsen og skalaen."""
    n = np.cross(v[f[:, 1]] - v[f[:, 0]], v[f[:, 2]] - v[f[:, 0]])
    area_w = 0.5 * np.linalg.norm(n, axis=1) * (job.scale * 100) ** 2  # cm²
    d1 = uv[:, 1] - uv[:, 0]; d2 = uv[:, 2] - uv[:, 0]
    area_uv = 0.5 * np.abs(d1[:, 0] * d2[:, 1] - d2[:, 0] * d1[:, 1]) * texels.tex ** 2
    fn = n / (np.linalg.norm(n, axis=1, keepdims=True) + 1e-12)
    c = v[f].mean(1)
    if region is None:
        top = c[:, 1].max(); H = top - c[:, 1].min()
        region = (fn[:, 2] > 0.3) & (c[:, 1] > top - 0.25 * H) if job.layout.startswith("body") else (fn[:, 2] > 0.3)
    if region.sum() < 3:
        return 0.0
    return round(float(np.sqrt(area_uv[region].sum() / max(area_w[region].sum(), 1e-9))), 2)
