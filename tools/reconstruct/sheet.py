"""Skjær ut visningene fra et forbildeark og segmenter dem mot den grå bakgrunnen."""
import json
import os

import numpy as np

from . import job as J


def tiles():
    import cv2
    im = cv2.imread(J.REF)[:, :, ::-1]
    return {k: np.ascontiguousarray(im[v["rc"][0] * J.TH:(v["rc"][0] + 1) * J.TH, v["rc"][1] * J.TW:(v["rc"][1] + 1) * J.TW])
            for k, v in J.VIEWS.items()}


# ---------------------------------------------------------------- seg
def seg():
    import cv2
    from scipy import ndimage as ndi
    for k, t in tiles().items():
        f = t.astype(np.float32)
        # bakgrunnen: grå med svak gradient; modeller den med medianen langs kanten per rad/kolonne
        bg = np.median(np.concatenate([f[:8].reshape(-1, 3), f[-8:].reshape(-1, 3),
                                       f[:, :8].reshape(-1, 3), f[:, -8:].reshape(-1, 3)]), 0)
        d = np.linalg.norm(f - bg, axis=2)
        sat = f.max(2) - f.min(2)
        fg = (d > 14) | (sat > 14)
        fg = ndi.binary_opening(fg, iterations=2)
        lab, n = ndi.label(fg)
        sizes = ndi.sum(fg, lab, range(1, n + 1))
        fg = lab == (1 + int(np.argmax(sizes)))
        fg = ndi.binary_closing(fg, iterations=6)
        fg = ndi.binary_fill_holes(fg)
        m = (fg * 255).astype(np.uint8)
        m = cv2.GaussianBlur(m, (5, 5), 0)
        cv2.imwrite(J.p(f"{k}_rgb.png"), t[:, :, ::-1])
        cv2.imwrite(J.p(f"{k}_mask.png"), m)
        ys, xs = np.nonzero(m > 127)
        print(k, "bbox x", xs.min(), xs.max(), "y", ys.min(), ys.max())


