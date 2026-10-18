"""Tegner trekanter til et bilde uten grafikkort: ortografisk eller med
perspektiv, fra en retning, flatt skyggelagt (lys ovenfra fra venstre),
med malerens algoritme (bakerst først). Til miniatyrer i katalogen og til
silhuettene portene sammenligner med forbildene."""
import math

import cv2
import numpy as np


def camera(yaw, pitch):
    """Høyrevektor, oppvektor og blikkretning for et kamera som ser mot
    origo fra `yaw` grader rundt y (0 = forfra, fra +z) og `pitch` grader ned."""
    y, p = math.radians(yaw), math.radians(pitch)
    fwd = np.array([-math.sin(y) * math.cos(p), -math.sin(p), -math.cos(y) * math.cos(p)])
    right = np.cross(fwd, [0.0, 1.0, 0.0])
    right /= np.linalg.norm(right)
    up = np.cross(right, fwd)
    return right, up, fwd


def render(mesh, size=256, yaw=-35.0, pitch=25.0, background=(40, 40, 40), mask=False):
    """Bildet (RGB uint8), eller silhuetten (bool) om `mask`."""
    tris = mesh.tris
    img = np.zeros((size, size, 3), np.uint8)
    img[:] = background
    if len(tris) == 0:
        return img[..., 0] > 255 if mask else img
    right, up, fwd = camera(yaw, pitch)
    lo, hi = mesh.bounds
    centre = (lo + hi) / 2
    p = tris - centre
    x = p @ right
    yv = p @ up
    z = p @ fwd
    extent = max(x.max() - x.min(), yv.max() - yv.min(), 1e-6)
    scale = (size * 0.9) / extent
    cx = (x.max() + x.min()) / 2
    cy = (yv.max() + yv.min()) / 2
    px = (x - cx) * scale + size / 2
    py = size / 2 - (yv - cy) * scale
    order = np.argsort(-z.mean(axis=1))
    if mask:
        m = np.zeros((size, size), np.uint8)
        pts = np.stack([px, py], -1)
        for t in order:
            cv2.fillConvexPoly(m, np.round(pts[t] * 16).astype(np.int32), 1, lineType=cv2.LINE_8, shift=4)
        return m > 0
    n = np.cross(tris[:, 1] - tris[:, 0], tris[:, 2] - tris[:, 0])
    n /= np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-12)
    # Tosidig: normalen mot kameraet.
    n *= np.where((n @ fwd) > 0, -1.0, 1.0)[:, None]
    light = np.array([-0.5, 0.8, 0.35])
    light /= np.linalg.norm(light)
    shade = 0.35 + 0.65 * np.clip(n @ light, 0, 1)
    base = mesh.tri_colors()
    rgb = np.clip(base * shade[:, None] * 255, 0, 255).astype(np.uint8)
    pts = np.round(np.stack([px, py], -1) * 16).astype(np.int32)
    for t in order:
        cv2.fillConvexPoly(img, pts[t], tuple(int(c) for c in rgb[t]), lineType=cv2.LINE_AA, shift=4)
    return img
