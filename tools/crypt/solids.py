"""Kolliderne som følger formen (eieren 18. oktober: «man henger seg opp i
noen usynlige vegger»).

En boks rundt en lysestake, en søyle, et fat med glør eller et gitter er
større enn det du ser, og kapselen stopper i lufta ved siden av tingen.
`fit` legger kollideren etter det som synes: geometrien inni boksen
samples som punkter, deles i skiver på `BAND` i høyden, og hver skive
rastres i ruter på `CELL` (det som er omsluttet fylles). En skive som er
nesten konveks blir et skrog av punktene (søylen, fatet), ellers bokser
over rutene (gitteret, stigen, en mur med døråpning). Like skiver over
hverandre slås sammen. En boks som allerede sitter tett, et gulv (toppen
er dekket) og en boks uten noe synlig inni står som før.

Brukes av `build_level.py` på nivåets egne kollidere (med bitene og
pynten fra byggesettet som geometri), og med `--models` på byggesettets
modeller: kolliderne skrives i modellens rom som `extras.solids` i
glb-fila, og motoren setter dem ut med modellen (`Builder::place_solids`)
og flytter dem med portgitteret, snarveidøra og den falske veggen.

    python3 tools/crypt/solids.py --models [navn …]
"""

import json
import math
import os
import sys

import numpy as np
from scipy import ndimage
from scipy.spatial import ConvexHull, QhullError

CELL = 0.05  # rutene i skivene (m): feilen langs en stråle er høyst en halv diagonal
BAND = 0.05  # skivene i høyden (m); like skiver over hverandre slås sammen
STEP = 0.025  # avstanden mellom punktene på flatene (m)
EPS = 0.02  # geometri så nær boksen regnes med (flaten muren står i)
CONVEX = 0.9  # en skive som fyller så mye av skroget sitt, blir et skrog
TIGHT = 0.93  # en boks som er så full i hver skive, står som før
MIN_T = 0.04  # så tynn blir ingen kollider i høyden (gulvet, en hylle)

# Materialene som ikke er noe å gå inn i (flammene, glød, vann).
SOFT = ("fire", "water", "glow")


def triangles(faces):
    """Trekantene (n, 3, 3) fra MB-flater (`pts, uvs, mat, smooth`), uten
    de myke materialene."""
    out = []
    for f in faces:
        pts, mat = f[0], f[2]
        if any(str(mat).startswith(s) for s in SOFT):
            continue
        for i in range(1, len(pts) - 1):
            out.append((pts[0], pts[i], pts[i + 1]))
    return np.array(out, dtype=np.float64).reshape(-1, 3, 3)


def sample(tris, step=STEP):
    """Punkter på trekantene med høyst `step` mellom (pluss hjørnene)."""
    if len(tris) == 0:
        return np.zeros((0, 3))
    a, b, c = tris[:, 0], tris[:, 1], tris[:, 2]
    longest = np.maximum(np.maximum(np.linalg.norm(b - a, axis=1), np.linalg.norm(c - b, axis=1)), np.linalg.norm(a - c, axis=1))
    out = [tris.reshape(-1, 3)]
    for n in np.unique(np.clip(np.ceil(longest / step), 1, 400).astype(int)):
        sel = np.ceil(longest / step).clip(1, 400).astype(int) == n
        i, j = np.meshgrid(np.arange(n + 1), np.arange(n + 1), indexing="ij")
        keep = i + j <= n
        u, v = (i[keep] / n), (j[keep] / n)
        A, B, C = a[sel], b[sel], c[sel]
        p = A[:, None, :] + (B - A)[:, None, :] * u[None, :, None] + (C - A)[:, None, :] * v[None, :, None]
        out.append(p.reshape(-1, 3))
    return np.concatenate(out)


class Scene:
    """Trekantene som synes, med mål, så bare de nær en boks samples."""

    def __init__(self, tris):
        self.tris = tris
        self.lo = tris.min(axis=1) if len(tris) else np.zeros((0, 3))
        self.hi = tris.max(axis=1) if len(tris) else np.zeros((0, 3))

    def near(self, lo, hi):
        """Punktene på trekantene som når inn i grensene lo–hi (x, y, z)."""
        if len(self.tris) == 0:
            return np.zeros((0, 3))
        sel = np.all(self.hi >= np.array(lo), axis=1) & np.all(self.lo <= np.array(hi), axis=1)
        if not sel.any():
            return np.zeros((0, 3))
        p = sample(self.tris[sel])
        keep = np.all(p >= np.array(lo), axis=1) & np.all(p <= np.array(hi), axis=1)
        return p[keep]


def rects(mask):
    """Grådige rektangler over de fylte rutene: (i0, j0, i1, j1) eksklusivt."""
    m = mask.copy()
    out = []
    ni, nj = m.shape
    for i in range(ni):
        j = 0
        while j < nj:
            if not m[i, j]:
                j += 1
                continue
            j1 = j
            while j1 < nj and m[i, j1]:
                j1 += 1
            i1 = i + 1
            while i1 < ni and m[i1, j:j1].all():
                i1 += 1
            m[i:i1, j:j1] = False
            out.append((i, j, i1, j1))
            j = j1
    return out


def hull2d(p):
    try:
        h = ConvexHull(p)
    except (QhullError, ValueError):
        return None
    return p[h.vertices]


def fit_box(scene, center, size, yaw, clip=None, convex=None):
    """Kolliderne for én boks (verdens rom). Gir None når boksen står som før
    (tett, et gulv, eller ingenting synlig inni)."""
    c = np.array(center, dtype=np.float64)
    half = np.array(size, dtype=np.float64) / 2
    cy, sy = math.cos(yaw), math.sin(yaw)
    # Boksens akser i verden (yaw om Y, som Rapier): lokal x -> (cos, 0, -sin), lokal z -> (sin, 0, cos).
    ex = np.array([cy, 0.0, -sy])
    ez = np.array([sy, 0.0, cy])
    r = math.hypot(half[0], half[2]) + EPS
    pts = scene.near((c[0] - r, c[1] - half[1] - EPS, c[2] - r), (c[0] + r, c[1] + half[1] + EPS, c[2] + r))
    if clip is not None and len(pts):
        N, D = clip
        pts = pts[np.all(pts @ N.T <= D + EPS, axis=1)]
    if len(pts) == 0:
        return None
    d = pts - c
    loc = np.stack([d @ ex, d[:, 1], d @ ez], axis=1)
    inside = (np.abs(loc[:, 0]) <= half[0] + EPS) & (np.abs(loc[:, 2]) <= half[2] + EPS) & (np.abs(loc[:, 1]) <= half[1] + EPS)
    loc = loc[inside]
    if len(loc) == 0:
        return None
    ni = max(1, int(math.ceil(2 * half[0] / CELL)))
    nj = max(1, int(math.ceil(2 * half[2] / CELL)))

    def grid(p):
        i = np.clip(((p[:, 0] + half[0]) / CELL).astype(int), 0, ni - 1)
        j = np.clip(((p[:, 2] + half[2]) / CELL).astype(int), 0, nj - 1)
        g = np.zeros((ni, nj), dtype=bool)
        g[i, j] = True
        return g

    # Skivene (uten flatene akkurat i bunnen og toppen av boksen: gulvet
    # under og taket over hører ikke til tingen).
    body = loc[(loc[:, 1] > -half[1] + 0.01) & (loc[:, 1] < half[1] - 0.01)]
    if len(body) == 0:
        return None
    nb = max(1, int(math.ceil(2 * half[1] / BAND)))
    bands = []
    full = True
    fill = []
    for b in range(nb):
        y0 = -half[1] + 2 * half[1] * b / nb
        y1 = -half[1] + 2 * half[1] * (b + 1) / nb
        p = body[(body[:, 1] >= y0) & (body[:, 1] <= y1)]
        if len(p) == 0:
            full = False
            fill.append(0.0)
            continue
        g = grid(p)
        solid = ndimage.binary_fill_holes(ndimage.binary_closing(g, iterations=1) | g)
        fill.append(float(solid.mean()))
        if solid.mean() < TIGHT:
            full = False
        bands.append((p, solid, y0, y1))
    # Tett: hver skive fylt, og det som synes når helt ut til sidene.
    gap = max(half[0] - body[:, 0].max(), body[:, 0].min() + half[0], half[2] - body[:, 2].max(), body[:, 2].min() + half[2])
    if full and gap <= CELL:
        return None
    # Hver bit av hver skive: et skrog når den er nesten konveks, ellers
    # bokser over rutene. Biter over hverandre som er like, slås sammen.
    stacks = []  # [kind, data, ylo, yhi, open]
    for p, solid, y0, y1 in bands:
        lab, n = ndimage.label(solid)
        pi = np.clip(((p[:, 0] + half[0]) / CELL).astype(int), 0, ni - 1)
        pj = np.clip(((p[:, 2] + half[2]) / CELL).astype(int), 0, nj - 1)
        plab = lab[pi, pj]
        ylo, yhi = float(p[:, 1].min()), float(p[:, 1].max())
        pieces = []
        for k in range(1, n + 1):
            comp = lab == k
            q = p[plab == k]
            h = hull2d(q[:, [0, 2]]) if len(q) >= 3 else None
            area = comp.sum() * CELL * CELL
            if h is not None:
                ha = 0.5 * abs(np.dot(h[:, 0], np.roll(h[:, 1], 1)) - np.dot(h[:, 1], np.roll(h[:, 0], 1)))
                if ha > 1e-4 and area / ha >= (CONVEX if convex is None else convex):
                    pieces.append(("hull", h, None))
                    continue
            for i0, j0, i1, j1 in rects(comp):
                x0, x1 = -half[0] + i0 * CELL, min(-half[0] + i1 * CELL, half[0])
                z0, z1 = -half[2] + j0 * CELL, min(-half[2] + j1 * CELL, half[2])
                inr = (pi >= i0) & (pi < i1) & (pj >= j0) & (pj < j1)
                ext = (p[inr][:, [0, 2]].min(0), p[inr][:, [0, 2]].max(0)) if inr.any() else None
                pieces.append(("rect", (round(x0, 3), round(z0, 3), round(x1, 3), round(z1, 3)), ext))
        used = set()
        for kind, data, ext in pieces:
            hit = None
            for si, st in enumerate(stacks):
                if si in used or st[0] != kind or abs(st[3] - ylo) > BAND * 1.5:
                    continue
                if (kind == "rect" and st[1] == data) or (kind == "hull" and same_hull(st[1], data)):
                    hit = si
                    break
            if hit is None:
                stacks.append([kind, data, ylo, yhi, ext])
                used.add(len(stacks) - 1)
            else:
                st = stacks[hit]
                if kind == "hull":
                    st[1] = np.concatenate([st[1], data])
                if ext is not None:
                    st[4] = ext if st[4] is None else (np.minimum(st[4][0], ext[0]), np.maximum(st[4][1], ext[1]))
                st[3] = yhi
                used.add(hit)
    out = []
    for kind, data, ylo, yhi, ext in stacks:
        # Et gulv eller en hylle er bare flaten: minst `MIN_T` tykk, nedover i boksen.
        if yhi - ylo < MIN_T:
            ylo = max(-half[1], yhi - MIN_T)
        if kind == "hull":
            h = hull2d(data)
            if h is None:
                continue
            pts3 = []
            for y in (ylo, yhi):
                for x, z in h:
                    w = c + ex * x + ez * z + np.array([0.0, y, 0.0])
                    pts3.append([round(float(v), 3) for v in w])
            out.append({"hull": pts3})
        else:
            x0, z0, x1, z1 = data
            # Ned til det som synes i rutene (en mur som står midt i en
            # rute skal ikke bli 5 cm tykkere enn den er), minst `MIN_W`.
            if ext is not None:
                x0, x1 = shrink(x0, x1, ext[0][0], ext[1][0])
                z0, z1 = shrink(z0, z1, ext[0][1], ext[1][1])
            mid = c + ex * ((x0 + x1) / 2) + ez * ((z0 + z1) / 2) + np.array([0.0, (ylo + yhi) / 2, 0.0])
            out.append({"box": {"center": [round(float(v), 3) for v in mid], "size": [round(float(x1 - x0), 3), round(max(yhi - ylo, 0.02), 3), round(float(z1 - z0), 3)], "yaw": round(yaw, 4)}})
    return out


MIN_W = 0.03


def shrink(a, b, lo, hi):
    """Grensene a–b til en rute krympet til punktene lo–hi i den, minst `MIN_W`."""
    a2, b2 = max(a, lo), min(b, hi)
    if b2 - a2 < MIN_W:
        m = (a2 + b2) / 2
        a2, b2 = max(a, m - MIN_W / 2), min(b, m + MIN_W / 2)
    return a2, b2


def same_hull(a, b, tol=0.03):
    """To skrog i xz er like (hvert hjørne nær det andre skroget)."""
    ha, hb = hull2d(a), hull2d(b)
    if ha is None or hb is None:
        return False
    for p, h in ((ha, hb), (hb, ha)):
        for q in p:
            if np.min(np.linalg.norm(h - q, axis=1)) > tol and dist_to_poly(q, h) > tol:
                return False
    return True


def dist_to_poly(q, h):
    best = 1e9
    for i in range(len(h)):
        a, b = h[i], h[(i + 1) % len(h)]
        ab = b - a
        t = np.clip(np.dot(q - a, ab) / max(np.dot(ab, ab), 1e-12), 0.0, 1.0)
        best = min(best, float(np.linalg.norm(a + ab * t - q)))
    return best


# -- Hvilke kollidere står i lufta (som testen `ingen_usynlige_vegger_i_krypten`) --

RAY_TOL = 0.12  # det som synes, skal stå så nær bak kollideren langs strålen
RAY_REACH = 1.2  # strålene fra gulvet er så lange
HEIGHTS = (0.45, 0.95, 1.5)


def floor_points(nav, step=0.7):
    """Stedene på gulvet: midten og et rutenett i hver polygon (som testen)."""
    out = []
    V = nav["verts"]
    for ids in nav["polys"]:
        pts = np.array([V[i] for i in ids], dtype=np.float64)
        if len(pts) < 3:
            continue
        # Mot klokka sett ovenfra, som nettet i motoren (`nav.rs`).
        xs, zs = pts[:, 0], pts[:, 2]
        if np.dot(xs, np.roll(zs, -1)) - np.dot(zs, np.roll(xs, -1)) < 0:
            pts = pts[::-1]
        xz = pts[:, [0, 2]]
        n = len(xz)
        cands = [xz.mean(axis=0)]
        lo, hi = xz.min(0), xz.max(0)
        for x in np.arange(math.ceil(lo[0] / step), math.floor(hi[0] / step) + 1) * step:
            for z in np.arange(math.ceil(lo[1] / step), math.floor(hi[1] / step) + 1) * step:
                cands.append(np.array([x, z]))
        for q in cands:
            ok = True
            for k in range(n):
                a, b = xz[k], xz[(k + 1) % n]
                if (b[0] - a[0]) * (q[1] - a[1]) - (b[1] - a[1]) * (q[0] - a[0]) < -1e-4:
                    ok = False
                    break
            if ok:
                out.append((q[0], height_in(pts, q), q[1]))
    return np.array(out)


def height_in(pts, q):
    """Høyden i polygonen der q (x, z) står, som `Nav::height_in`: viften
    fra det første hjørnet, trekanten punktet er i (eller nærmest)."""
    best = (1e9, pts[0][1])
    a = pts[0][[0, 2]]
    for k in range(1, len(pts) - 1):
        b, c = pts[k][[0, 2]], pts[k + 1][[0, 2]]
        cr = lambda u, v: u[0] * v[1] - u[1] * v[0]
        d = cr(b - a, c - a)
        if abs(d) < 1e-8:
            continue
        w1 = cr(q - a, c - a) / d
        w2 = cr(b - a, q - a) / d
        w0 = 1.0 - w1 - w2
        y = pts[0][1] * w0 + pts[k][1] * w1 + pts[k + 1][1] * w2
        out = max(-w0, -w1, -w2, 0.0)
        if out < best[0]:
            best = (out, y)
    return best[1]


class Solids:
    """Kolliderne som bokser og skrog, til stråler."""

    def __init__(self, colliders):
        self.planes = []  # per kollider: (normaler, avstander) for halvrommene
        lo, hi = [], []
        for col in colliders:
            if "box" in col:
                b = col["box"]
                c = np.array(b["center"], dtype=np.float64)
                h = np.array(b["size"], dtype=np.float64) / 2
                y = b.get("yaw", 0.0)
                ex = np.array([math.cos(y), 0.0, -math.sin(y)])
                ez = np.array([math.sin(y), 0.0, math.cos(y)])
                ey = np.array([0.0, 1.0, 0.0])
                N = np.array([ex, -ex, ey, -ey, ez, -ez])
                D = np.array([ex @ c + h[0], -ex @ c + h[0], c[1] + h[1], -c[1] + h[1], ez @ c + h[2], -ez @ c + h[2]])
                r = np.abs(ex) * h[0] + np.abs(ey) * h[1] + np.abs(ez) * h[2]
                lo.append(c - r)
                hi.append(c + r)
            else:
                pts = np.array(col["hull"], dtype=np.float64)
                try:
                    hh = ConvexHull(pts)
                except (QhullError, ValueError):
                    N, D = np.zeros((0, 3)), np.zeros(0)
                    lo.append(pts.min(0))
                    hi.append(pts.max(0))
                    self.planes.append((N, D))
                    continue
                N = hh.equations[:, :3]
                D = -hh.equations[:, 3]
                lo.append(pts.min(0))
                hi.append(pts.max(0))
            self.planes.append((N, D))
        self.lo, self.hi = np.array(lo), np.array(hi)

    def cast(self, o, dirs, maxd):
        """Første treff for hver stråle: (avstand, kollider) eller (inf, -1)."""
        near = np.nonzero(np.all(self.hi >= o - maxd, axis=1) & np.all(self.lo <= o + maxd, axis=1))[0]
        best = np.full(len(dirs), np.inf)
        who = np.full(len(dirs), -1)
        for k in near:
            N, D = self.planes[k]
            if len(N) == 0:
                continue
            nd = dirs @ N.T  # (r, f)
            no = N @ o - D  # (f,) > 0 utenfor
            with np.errstate(divide="ignore", invalid="ignore"):
                t = -no[None, :] / nd
            enter = np.where(nd < 0, t, -np.inf).max(axis=1)
            exit_ = np.where(nd > 0, t, np.inf).min(axis=1)
            # Parallelle flater: strålen er utenfor om origo er utenfor.
            outside = np.any((np.abs(nd) < 1e-12) & (no[None, :] > 0), axis=1)
            enter = np.maximum(enter, 0.0)
            hit = (~outside) & (enter <= exit_) & (enter <= maxd)
            closer = hit & (enter < best)
            best[closer] = enter[closer]
            who[closer] = k
        return best, who


def ray_tris(o, dirs, tris):
    """Avstanden til nærmeste trekant for hver stråle (begge sider)."""
    if len(tris) == 0:
        return np.full(len(dirs), np.inf)
    a = tris[:, 0]
    e1 = tris[:, 1] - a
    e2 = tris[:, 2] - a
    out = np.full(len(dirs), np.inf)
    for r, d in enumerate(dirs):
        p = np.cross(d, e2)
        det = np.einsum("ij,ij->i", e1, p)
        ok = np.abs(det) > 1e-12
        inv = np.where(ok, 1.0 / np.where(ok, det, 1.0), 0.0)
        s = o - a
        u = np.einsum("ij,ij->i", s, p) * inv
        q = np.cross(s, e1)
        v = (q @ d) * inv
        t = np.einsum("ij,ij->i", e2, q) * inv
        good = ok & (u >= 0) & (u <= 1) & (v >= 0) & (u + v <= 1) & (t >= 0)
        if good.any():
            out[r] = t[good].min()
    return out


RAY_EDGE = 0.03  # treffet ligger så nær noe som synes: strålen streifer en kant


def touches(tris, q, r):
    """Om en av trekantene ligger innen `r` fra punktet."""
    if len(tris) == 0:
        return False
    sel = np.all(tris.min(axis=1) <= q + r, axis=1) & np.all(tris.max(axis=1) >= q - r, axis=1)
    for t in tris[sel]:
        if np.linalg.norm(closest(q, t) - q) <= r:
            return True
    return False


def closest(p, t):
    """Det nærmeste punktet på trekanten t (3, 3) til p."""
    a, b, c = t
    ab, ac, ap = b - a, c - a, p - a
    d1, d2 = ab @ ap, ac @ ap
    if d1 <= 0 and d2 <= 0:
        return a
    bp = p - b
    d3, d4 = ab @ bp, ac @ bp
    if d3 >= 0 and d4 <= d3:
        return b
    vc = d1 * d4 - d3 * d2
    if vc <= 0 and d1 >= 0 and d3 <= 0:
        return a + ab * (d1 / (d1 - d3))
    cp = p - c
    d5, d6 = ab @ cp, ac @ cp
    if d6 >= 0 and d5 <= d6:
        return c
    vb = d5 * d2 - d1 * d6
    if vb <= 0 and d2 >= 0 and d6 <= 0:
        return a + ac * (d2 / (d2 - d6))
    va = d3 * d6 - d5 * d4
    if va <= 0 and (d4 - d3) >= 0 and (d5 - d6) >= 0:
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)))
    den = 1.0 / (va + vb + vc)
    return a + ab * (vb * den) + ac * (vc * den)


def floating(colliders, tris, nav):
    """Kolliderne som står i lufta: for hver en stråle fra gulvet treffer
    uten at noe synes innen `RAY_TOL` bak treffet, treffpunktene."""
    solids = Solids(colliders)
    lo, hi = tris.min(axis=1), tris.max(axis=1)
    # Tettere enn testen (32 retninger, punktene hver 0,35 m), så det testen
    # finner, er funnet her først.
    ang = np.arange(32) * 2 * math.pi / 32
    dirs = np.stack([np.cos(ang), np.zeros(32), np.sin(ang)], axis=1)
    bad = {}
    for p in floor_points(nav, 0.35):
        r = RAY_REACH + RAY_TOL + 0.6
        sel = np.all(hi >= p - np.array([r, 0.5, r]), axis=1) & np.all(lo <= p + np.array([r, 2.5, r]), axis=1)
        near = tris[sel]
        for h in HEIGHTS:
            o = p + np.array([0.0, h, 0.0])
            t1, who = solids.cast(o, dirs, RAY_REACH)
            t2, _ = solids.cast(o + np.array([0.0, 0.06, 0.0]), dirs, RAY_REACH + 0.5)
            # Står punktet inni en kollider (bak en falsk vegg som er stengt), er det ingen vegg å se.
            with np.errstate(invalid="ignore"):
                wall = np.isfinite(t1) & np.isfinite(t2) & (np.abs(t2 - t1) <= 0.03) & (t1 > 0.02)
            if not wall.any():
                continue
            seen = ray_tris(o, dirs[wall], near)
            for t, w, sv, d in zip(t1[wall], who[wall], seen, dirs[wall]):
                if not sv <= t + RAY_TOL and not touches(near, o + d * t, RAY_EDGE):
                    bad.setdefault(int(w), []).append(o + d * t)
    return bad


def fit_hull(scene, pts, convex=None):
    """Som `fit_box`, for et skrog (en rampe over trappa): det som synes
    inni skroget, i skrogets aksejusterte mål."""
    pts = np.array(pts, dtype=np.float64)
    try:
        h = ConvexHull(pts)
    except (QhullError, ValueError):
        return None
    lo, hi = pts.min(0), pts.max(0)
    return fit_box(scene, ((lo + hi) / 2).tolist(), (hi - lo).tolist(), 0.0, clip=(h.equations[:, :3], -h.equations[:, 3]), convex=convex)


def volume(col):
    if "box" in col:
        return float(np.prod(col["box"]["size"]))
    try:
        return float(ConvexHull(np.array(col["hull"])).volume)
    except (QhullError, ValueError):
        return 0.0


SPECK = 0.02  # en bit så liten (m³) som fortsatt står i lufta, tas bort


SEG = 1.0  # en lang boks deles i biter på så mye, og bare bitene som står i lufta legges om


def fit_part(scene, col, hits):
    """En boks som står i lufta: langs den lengste siden i biter på `SEG`;
    bitene nær treffene legges etter formen, resten står som bokser."""
    b = col["box"]
    c = np.array(b["center"], dtype=np.float64)
    size = np.array(b["size"], dtype=np.float64)
    yaw = b.get("yaw", 0.0)
    ex = np.array([math.cos(yaw), 0.0, -math.sin(yaw)])
    ez = np.array([math.sin(yaw), 0.0, math.cos(yaw)])
    axis, e = (0, ex) if size[0] >= size[2] else (2, ez)
    L = size[axis]
    if L <= 2 * SEG:
        return fit_box(scene, b["center"], b["size"], yaw)
    n = int(math.ceil(L / SEG))
    step = L / n
    u = [float((np.array(h) - c) @ e) for h in hits]
    near = [any(abs((-L / 2 + (k + 0.5) * step) - x) <= step / 2 + 0.5 for x in u) for k in range(n)]
    out, changed = [], False
    k = 0
    while k < n:
        k1 = k
        while k1 < n and near[k1] == near[k]:
            k1 += 1
        a, bb = -L / 2 + k * step, -L / 2 + k1 * step
        sz = size.copy()
        sz[axis] = bb - a
        mid = c + e * ((a + bb) / 2)
        part = {"box": {"center": [round(float(v), 3) for v in mid], "size": [round(float(v), 3) for v in sz], "yaw": yaw}}
        fitted = fit_box(scene, part["box"]["center"], part["box"]["size"], yaw) if near[k] else None
        if fitted is None:
            out.append(part)
        else:
            out.extend(fitted)
            changed = True
        k = k1
    return out if changed else None


def empty(scene, b, small=3.0, inset=0.02):
    """En liten boks (høyst `small` m) uten noe som synes inni."""
    size = np.array(b["size"], dtype=np.float64)
    if max(size[0], size[2]) > small:
        return False
    c = np.array(b["center"], dtype=np.float64)
    half = size / 2
    yaw = b.get("yaw", 0.0)
    ex = np.array([math.cos(yaw), 0.0, -math.sin(yaw)])
    ez = np.array([math.sin(yaw), 0.0, math.cos(yaw)])
    r = math.hypot(half[0], half[2])
    pts = scene.near((c[0] - r, c[1] - half[1], c[2] - r), (c[0] + r, c[1] + half[1], c[2] + r))
    if len(pts) == 0:
        return True
    d = pts - c
    inside = (np.abs(d @ ex) < half[0] - inset) & (np.abs(d @ ez) < half[2] - inset) & (np.abs(d[:, 1]) < half[1] - inset)
    return not inside.any()


POST = 0.35  # en boks så smal (m) og minst en halv meter høy er en stolpe (baluster, lysestake)


def is_post(col):
    size = col["box"]["size"]
    return max(size[0], size[2]) <= POST and size[1] >= 0.5


def corners(col):
    """Hjørnene (eller punktene) til en kollider, i verdens rom."""
    if "hull" in col:
        return np.array(col["hull"], dtype=np.float64)
    b = col["box"]
    c = np.array(b["center"], dtype=np.float64)
    half = np.array(b["size"], dtype=np.float64) / 2
    yaw = b.get("yaw", 0.0)
    ex = np.array([math.cos(yaw), 0.0, -math.sin(yaw)])
    ez = np.array([math.sin(yaw), 0.0, math.cos(yaw)])
    return np.array([c + ex * sx * half[0] + np.array([0.0, sy * half[1], 0.0]) + ez * sz * half[2] for sx in (-1, 1) for sy in (-1, 1) for sz in (-1, 1)])


def post_hull(pieces):
    """Bitene av en stolpe (skiver på 5 cm, med sokkel og kapitél) som ett
    skrog: de mange tynne skivene over hverandre fanget kapselen i
    vindeltrappa i den sunkne borgen."""
    pts = np.concatenate([corners(p) for p in pieces])
    try:
        h = ConvexHull(pts)
    except (QhullError, ValueError):
        return pieces
    return [{"hull": [[round(float(v), 3) for v in pts[k]] for k in h.vertices]}]


def trim_hull(col, depth=0.6):
    """Et skrog under en trapp eller en rampe som står i lufta (enden under
    reposet, sida mot rommet under): bare flaten du går på og `depth`
    under den."""
    pts = np.array(col["hull"], dtype=np.float64)
    try:
        h = ConvexHull(pts)
    except (QhullError, ValueError):
        return None
    top = set()
    for eq, simplex in zip(h.equations, h.simplices):
        if eq[1] > 0.3:
            top.update(int(i) for i in simplex)
    if len(top) < 3:
        return None
    floor = pts[:, 1].min()
    upper = pts[sorted(top)]
    lower = upper.copy()
    lower[:, 1] = np.maximum(lower[:, 1] - depth, floor)
    if np.all(lower[:, 1] <= floor + 0.05):
        return None
    return [{"hull": [[round(float(v), 3) for v in p] for p in np.concatenate([upper, lower])]}]


def fit(colliders, scene, nav=None):
    """Nivåets kollidere med dem som står i lufta lagt etter formen (bare
    dem strålene fra gulvet finner, `floating`). Små biter som fortsatt
    står i lufta etterpå (en kant av en rute), tas bort."""
    if nav is None:
        out = []
        for col in colliders:
            new = fit_box(scene, col["box"]["center"], col["box"]["size"], col["box"].get("yaw", 0.0)) if "box" in col else None
            out.extend(new or [col])
        return out, 0
    # I runder: når det som sto foran er lagt om, kan strålene nå noe bak.
    out = list(colliders)
    fresh = set()  # nye biter (indekser i `out`)
    tried = set()  # id-ene til dem som er forsøkt
    changed = 0
    for _ in range(4):
        targets = floating(out, scene.tris, nav)
        # Nye biter (en skive som var nesten konveks, en trimmet trapp) som
        # fortsatt står i lufta, legges etter formen inni seg én gang til.
        todo = {k: v for k, v in targets.items() if id(out[k]) not in tried}
        if not todo:
            break
        nxt, nfresh = [], set()
        for k, col in enumerate(out):
            if k not in todo:
                if k in fresh:
                    nfresh.add(len(nxt))
                nxt.append(col)
                continue
            tried.add(id(col))
            if "box" in col and empty(scene, col["box"], small=1e9 if k in fresh else 3.0, inset=-0.005 if k in fresh else 0.02):
                # Ingenting synes inni: en usynlig vegg (pynten står et annet
                # sted), og tingen som synes har sine egne kollidere.
                changed += 1
                continue
            post = False
            if "box" in col:
                new = fit_part(scene, col, todo[k])
                if new and is_post(col):
                    # Stolpen er ferdig: skroget legges ikke om igjen.
                    new, post = post_hull(new), True
            elif k in fresh:
                new = fit_hull(scene, col["hull"], convex=2.0)
            else:
                new = trim_hull(col)
            if new is None:
                nxt.append(col)
            else:
                if not post:
                    nfresh.update(range(len(nxt), len(nxt) + len(new)))
                nxt.extend(new)
                changed += 1
        out, fresh = nxt, nfresh
    if fresh:
        again = floating(out, scene.tris, nav)
        drop = {k for k in again if k in fresh and volume(out[k]) < SPECK}
        out = [c for k, c in enumerate(out) if k not in drop]
    return out, changed


# -- Byggesettets modeller ----------------------------------------------------

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
MODELS = os.path.join(ROOT, "assets", "models", "crypt")


def glb_triangles(path):
    """Trekantene i en glb-fil (modellens rom, nodenes transformasjoner
    tatt med), uten de myke materialene."""
    sys.path.insert(0, os.path.join(ROOT, "tools", "models"))
    import slim

    j, bin_ = slim.read(path)
    mats = [m.get("name", "") for m in j.get("materials", [])]

    def node_mats(i, parent):
        n = j["nodes"][i]
        m = np.eye(4)
        if "matrix" in n:
            m = np.array(n["matrix"], dtype=np.float64).reshape(4, 4).T
        else:
            t = n.get("translation", [0, 0, 0])
            q = n.get("rotation", [0, 0, 0, 1])
            s = n.get("scale", [1, 1, 1])
            x, y, z, w = q
            r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)], [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)], [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
            m[:3, :3] = r * np.array(s)[None, :]
            m[:3, 3] = t
        m = parent @ m
        yield i, m
        for k in n.get("children", []):
            yield from node_mats(k, m)

    scene = j.get("scenes", [{}])[j.get("scene", 0)]
    tris = []
    for root in scene.get("nodes", []):
        for i, m in node_mats(root, np.eye(4)):
            mesh = j["nodes"][i].get("mesh")
            if mesh is None:
                continue
            for prim in j["meshes"][mesh]["primitives"]:
                name = mats[prim["material"]] if "material" in prim and prim["material"] < len(mats) else ""
                if any(name.startswith(s) for s in SOFT):
                    continue
                pos = np.array(slim.elements(j, bin_, prim["attributes"]["POSITION"]), dtype=np.float64)
                idx = np.array(slim.elements(j, bin_, prim["indices"]), dtype=np.int64).reshape(-1, 3) if "indices" in prim else np.arange(len(pos)).reshape(-1, 3)
                w = pos @ m[:3, :3].T + m[:3, 3]
                tris.append(w[idx])
    return np.concatenate(tris) if tris else np.zeros((0, 3, 3))


def model_solids(path):
    """Kolliderne til en modell i modellens eget rom: boksen rundt den,
    lagt etter formen (eller boksen selv når den sitter tett)."""
    tris = glb_triangles(path)
    if len(tris) == 0:
        return []
    lo, hi = tris.reshape(-1, 3).min(0), tris.reshape(-1, 3).max(0)
    # Litt luft over og under, så bunnen og toppen er med i skivene.
    center = ((lo + hi) / 2).tolist()
    size = (hi - lo + np.array([0.0, 0.04, 0.0])).tolist()
    scene = Scene(tris)
    fitted = fit_box(scene, center, size, 0.0)
    if fitted is None:
        return [{"box": {"center": [round(v, 3) for v in center], "size": [round(v, 3) for v in size], "yaw": 0.0}}]
    return fitted


# Byggesettets modeller som står i veien (motoren setter dem ut etter merkene).
KIT = ["chest", "kit_lore", "kit_bonus", "kit_lever", "kit_rest", "kit_up", "kit_down", "kit_ladder", "kit_ladder_top", "kit_gate", "kit_shortcut", "kit_shortcut_leaf", "kit_secret_wall"]
# Pynten som står på gulvet (`PROPS` i `crypt/mod.rs`, uten det som henger).
KIT += ["kit_sarcophagus", "kit_strut", "kit_rail", "kit_cart", "kit_crystal", "kit_armor", "kit_statue", "kit_rack", "kit_throne"]


def write_solids(path, solids):
    sys.path.insert(0, os.path.join(ROOT, "tools", "models"))
    import slim

    j, bin_ = slim.read(path)
    extras = j.setdefault("extras", {})
    if extras.get("solids") == solids:
        return False
    extras["solids"] = solids
    slim.write(path, j, bin_)
    return True


def main():
    args = sys.argv[1:]
    if "--models" in args:
        names = [a for a in args if not a.startswith("--")]
        if not names:
            names = KIT + sorted(f[:-4] for f in os.listdir(MODELS) if f.startswith("crypt_") and "_secret_" in f and f.endswith(".glb"))
        for name in names:
            path = os.path.join(MODELS, f"{name}.glb")
            if not os.path.exists(path):
                import glob

                found = glob.glob(os.path.join(ROOT, "assets", "models", "*", f"{name}.glb"))
                path = found[0] if found else path
            if not os.path.exists(path):
                print(f"  {name}: finnes ikke")
                continue
            s = model_solids(path)
            changed = write_solids(path, s)
            print(f"  {name}: {len(s)} kollidere{' (skrevet)' if changed else ''}")
        return
    print(__doc__)


if __name__ == "__main__":
    main()
