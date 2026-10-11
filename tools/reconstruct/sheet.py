"""Skjær ut visningene fra et forbildeark og segmenter dem mot den grå bakgrunnen."""
import json
import os

import numpy as np

from . import job as J


def tiles():
    import cv2
    im = cv2.imread(J.REF)[:, :, ::-1]
    if J.CROPS:  # figuren i sin egen stripe, sentrert i en rute på J.TW med bakgrunnsfarge rundt
        bg = np.median(im[:8].reshape(-1, 3), 0).astype(np.uint8)
        out = {}
        for k, v in J.VIEWS.items():
            x0, x1 = J.CROPS[v["rc"][1]]
            t = np.empty((J.TH, J.TW, 3), np.uint8); t[:] = bg
            o = (J.TW - (x1 - x0)) // 2
            t[:, o:o + x1 - x0] = im[:J.TH, x0:x1]
            out[k] = t
        return out
    return {k: np.ascontiguousarray(im[v["rc"][0] * J.TH:(v["rc"][0] + 1) * J.TH, v["rc"][1] * J.TW:(v["rc"][1] + 1) * J.TW])
            for k, v in J.VIEWS.items()}


# ---------------------------------------------------------------- seg
def seg():
    import cv2
    from scipy import ndimage as ndi
    for k, t in tiles().items():
        if J.VIEWS[k].get("mirror_of"):
            continue
        f = t.astype(np.float32)
        # bakgrunnen: grå med svak gradient; modeller den med medianen langs kanten per rad/kolonne
        bg = np.median(np.concatenate([f[:8].reshape(-1, 3), f[-8:].reshape(-1, 3),
                                       f[:, :8].reshape(-1, 3), f[:, -8:].reshape(-1, 3)]), 0)
        d = np.linalg.norm(f - bg, axis=2)
        sat = f.max(2) - f.min(2)
        fg = (d > 14) | (sat > 14)
        if J.OPT.get("floor_line"):  # tynne vannrette linjer (gulvet under føttene): fjern med vertikal åpning
            thin = fg & ~ndi.binary_opening(fg, structure=np.ones((9, 1)))
            fg &= ~thin
        fg = ndi.binary_opening(fg, iterations=2)
        lab, n = ndi.label(fg)
        sizes = ndi.sum(fg, lab, range(1, n + 1))
        fg = lab == (1 + int(np.argmax(sizes)))
        fg = ndi.binary_closing(fg, iterations=6)
        if J.OPT.get("open_holes"):
            # hull med bakgrunnsfarge (mellom arm og kropp, mellom beina) står åpne; små hull fylles
            holes = ndi.binary_fill_holes(fg) & ~fg
            lab, n = ndi.label(holes)
            for i in range(1, n + 1):
                h = lab == i
                if h.sum() < J.OPT.get("min_hole", 400) or np.median(d[h]) > 14:
                    fg |= h
        else:
            fg = ndi.binary_fill_holes(fg)
        m = (fg * 255).astype(np.uint8)
        m = cv2.GaussianBlur(m, (5, 5), 0)
        cv2.imwrite(J.p(f"{k}_rgb.png"), t[:, :, ::-1])
        cv2.imwrite(J.p(f"{k}_mask.png"), m)
        ys, xs = np.nonzero(m > 127)
        print(k, "bbox x", xs.min(), xs.max(), "y", ys.min(), ys.max())
    for k, v in J.VIEWS.items():  # speilede visninger
        if v.get("mirror_of"):
            for kind in ("rgb", "mask"):
                im = cv2.imread(J.p(f"{v['mirror_of']}_{kind}.png"), cv2.IMREAD_UNCHANGED)
                cv2.imwrite(J.p(f"{k}_{kind}.png"), im[:, ::-1])
