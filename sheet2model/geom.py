"""Kameraene: hver visning er et ortografisk kamera med en basis (cam, right, up) som kan dreies litt
(yaw om loddrett akse, pitch om bildets høyre, roll om kameraaksen), en skala `sy` (bildepiksler per
verdenspiksel) og et nullpunkt (`c`, `top`) i bildet. Verdenspunkt P -> (u, v):

    u = c   + (P · right) · sy
    v = top − (P · up)    · sy

Innrettingen (`align.py`) finner disse parametrene per visning; alt annet (kalibrering, smelting, projeksjon,
rapport) går gjennom `to_img`, så en dreid profil blir modellert som dreid i stedet for å ødelegge formen.
"""
import numpy as np


def rot(axis, deg):
    a = np.asarray(axis, float); a = a / (np.linalg.norm(a) + 1e-12)
    t = np.radians(deg)
    K = np.array([[0, -a[2], a[1]], [a[2], 0, -a[0]], [-a[1], a[0], 0]])
    return np.eye(3) + np.sin(t) * K + (1 - np.cos(t)) * (K @ K)


def basis(view, o):
    """(cam, right, up) for visningen med dreiningene i rammen `o` (yaw, pitch, roll i grader)."""
    cam, right, up = view.cam.copy(), view.right.copy(), view.up.copy()
    yaw, pitch, roll = o.get("yaw", 0.0), o.get("pitch", 0.0), o.get("roll", 0.0)
    if yaw:
        R = rot((0, 1, 0), yaw)
        cam, right, up = R @ cam, R @ right, R @ up
    if pitch:
        R = rot(right, pitch)
        cam, up = R @ cam, R @ up
    if roll:
        R = rot(cam, roll)
        right, up = R @ right, R @ up
    return cam, right, up


def _dot(X, Y, Z, d):
    """X·d0 + Y·d1 + Z·d2 i float32, uten de leddene som er null (store rutenett)."""
    dt = X.dtype if isinstance(X, np.ndarray) else np.float32
    out = None
    for A, w in ((X, d[0]), (Y, d[1]), (Z, d[2])):
        if abs(w) < 1e-12:
            continue
        term = A * dt.type(w) if isinstance(A, np.ndarray) else A * w
        out = term if out is None else out + term
    return np.zeros(np.shape(X), dt) if out is None else out


def to_img(view, o, X, Y, Z):
    """Verdenspunkt (piksler) -> (u, v) i visningen."""
    cam, right, up = basis(view, o)
    sy = np.float32(o["sy"])
    b = _dot(X, Y, Z, up)
    persp = float(o.get("persp", 0.0))
    if persp:  # svakt perspektiv i tegningen: skalaen endrer seg lineært fra toppen til bunnen (per 500 px)
        sy = sy * (1 + np.float32(persp) * (-b) / np.float32(500.0))
    u = _dot(X, Y, Z, right) * sy + np.float32(o["c"])
    v = np.float32(o["top"]) - b * sy
    return u, v


def toward(view, o, X, Y, Z):
    """Hvor langt fram mot kameraet punktet ligger (verdenspiksler)."""
    return _dot(X, Y, Z, basis(view, o)[0])


def sample(img, u, v, fill=0.0):
    """Bilineær oppslag i et bilde (float32) på flyttallskoordinater, i store blokker."""
    import cv2
    n = u.size
    if n == 0:
        return np.zeros(u.shape, np.float32)
    w = 4096
    h = (n + w - 1) // w
    uu = np.zeros(h * w, np.float32); uu[:n] = u.ravel()
    vv = np.zeros(h * w, np.float32); vv[:n] = v.ravel()
    o = cv2.remap(np.ascontiguousarray(img, dtype=np.float32), uu.reshape(h, w), vv.reshape(h, w),
                  cv2.INTER_LINEAR, borderMode=cv2.BORDER_CONSTANT, borderValue=fill)
    return o.ravel()[:n].reshape(u.shape)


def nearest(img, u, v):
    ui = np.clip(np.round(u).astype(int), 0, img.shape[1] - 1)
    vi = np.clip(np.round(v).astype(int), 0, img.shape[0] - 1)
    return img[vi, ui]


def run_bounds(mb):
    """For hver piksel i masken: venstre og høyre kant (kolonne) av biten i raden den ligger i."""
    W = mb.shape[1]
    idx = np.broadcast_to(np.arange(W), mb.shape)
    left = np.maximum.accumulate(np.where(~mb, idx, -1), axis=1) + 1
    ridx = np.where(~mb[:, ::-1], idx, -1)
    right = (W - 1 - np.maximum.accumulate(ridx, axis=1))[:, ::-1] - 1
    return left.astype(np.float32), right.astype(np.float32)


def w2b(v, scale):
    """Verden (x mot venstre side, y opp, z fram, piksler) -> Blender (meter, -Y fram, Z opp)."""
    v = np.asarray(v, float)
    return np.stack([v[..., 0], -v[..., 2], v[..., 1]], -1) * scale


def b2w(v, scale):
    v = np.asarray(v, float) / scale
    return np.stack([v[..., 0], v[..., 2], -v[..., 1]], -1)


def mirror_frame(src, TW):
    """Rammen til den speilede visningen fra kilden: u speiles, yaw og roll bytter fortegn."""
    o = dict(src)
    o["c"] = (TW - 1) - src["c"]
    o["yaw"] = -src.get("yaw", 0.0)
    o["roll"] = -src.get("roll", 0.0)
    return o


def scatter_max(idx, vals, size, fill=np.nan):
    """out[i] = største vals der idx == i."""
    out = np.full(size, fill, np.float32)
    if len(idx):
        np.fmax.at(out, idx, vals.astype(np.float32))
    return out
