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
        if J.OPT.get("drop_pole"):
            # lange staver (høygaffel, spyd, pilegrimsstav) er tynne og loddrette: tas ut av silhuetten, ellers
            # blir toppen av staven «issen» i innrettingen
            thin = fg & ~ndi.binary_opening(fg, structure=np.ones((1, 41)))
            # bare lange, loddrette biter er staver (resten, som halsen og fingrene, settes tilbake)
            lab, n = ndi.label(thin)
            pole = np.zeros_like(thin)
            for i, sl in enumerate(ndi.find_objects(lab)):
                if sl is not None and (sl[0].stop - sl[0].start) > 140:
                    pole |= lab == i + 1
            fg &= ~ndi.binary_dilation(pole, iterations=2)
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
        for x0, y0, x1, y1 in J.OPT.get("erase", {}).get(k, []):  # rester av redskap, satt for hånd per ark
            fg[y0:y1, x0:x1] = False
        lab, n = ndi.label(fg)  # bare figuren (rester av staver o.l. faller bort)
        if n > 1:
            sizes = ndi.sum(fg, lab, range(1, n + 1))
            fg = lab == (1 + int(np.argmax(sizes)))
        m = (fg * 255).astype(np.uint8)
        m = cv2.GaussianBlur(m, (5, 5), 0)
        cv2.imwrite(J.p(f"{k}_rgb.png"), t[:, :, ::-1])
        cv2.imwrite(J.p(f"{k}_mask.png"), m)
        ys, xs = np.nonzero(m > 127)
        print(k, "bbox x", xs.min(), xs.max(), "y", ys.min(), ys.max())
    for k, v in J.VIEWS.items():  # speilede visninger
        if v.get("mirror_of"):
            rgb = cv2.imread(J.p(f"{v['mirror_of']}_rgb.png"))
            m = cv2.imread(J.p(f"{v['mirror_of']}_mask.png"), 0)
            if J.OPT.get("tool_box") and J.OPT.get("mirror_strip_tool", True):
                rgb, m = strip_tool(rgb, m)
            cv2.imwrite(J.p(f"{k}_rgb.png"), rgb[:, ::-1])
            cv2.imwrite(J.p(f"{k}_mask.png"), m[:, ::-1])


def strip_tool(bgr, m8):
    """Redskapet bare den ene hånda holder, skal ikke speiles over i den andre: fjern det fra profilen før
    speilingen. Det som stikker ut forbi kroppen (og hånda) tas bort fra silhuetten; der redskapet ligger foran
    kroppen, blir silhuetten stående, og fargen males over fra kroppen rundt (inpaint)."""
    import cv2
    from scipy import ndimage as ndi
    from .assemble import hammer_mask
    m = m8 > 127
    tool = tool_mask(bgr[:, :, ::-1], m)
    if J.OPT.get("tool_box"):  # redskapet innenfor en boks i profilen (sheets.json), på farge, uten hud
        import cv2 as _cv
        boxes = J.OPT["tool_box"]
        boxes = [boxes] if isinstance(boxes[0], int) else boxes
        hsv = _cv.cvtColor(bgr, _cv.COLOR_BGR2HSV).astype(np.float32)
        h, s_, v_ = hsv[..., 0] * 2, hsv[..., 1], hsv[..., 2]
        iron = (s_ < 45) & (v_ > 90) & (v_ < 225)
        wood = (h > 15) & (h < 50) & (s_ > 25) & (s_ < 200) & (v_ > 110)
        skin = (h > 2) & (h < 28) & (s_ > 45) & (s_ < 170) & (v_ > 140)
        box = np.zeros_like(m)
        for x0, y0, x1, y1 in boxes:
            box[y0:y1, x0:x1] = True
        noskin = ~ndi.binary_dilation(skin, iterations=3)
        if J.OPT.get("tool_strip", "box") == "box":
            tool = box & m & noskin  # hele boksen utenom hånda (redskapet ligger inne i silhuetten)
        else:  # på farge: jern og lyst tre, vokst litt inn i omrisset, men ikke inn i mørke klær
            col = ndi.binary_closing((iron | wood) & box & m & noskin, iterations=3)
            tool = col | (ndi.binary_dilation(col, iterations=6) & (v_ > 55) & box & m & noskin)
            tool = ndi.binary_fill_holes(ndi.binary_closing(tool, iterations=4)) & box & m & noskin
    tool = ndi.binary_dilation(tool, iterations=2 if J.OPT.get("tool_box") else 4) & m
    np.save(J.p("tool_src.npy"), tool)
    rest = m & ~tool
    import cv2 as _cv
    _h = _cv.cvtColor(bgr, _cv.COLOR_BGR2HSV).astype(np.float32)
    skin = (_h[..., 0] * 2 > 2) & (_h[..., 0] * 2 < 28) & (_h[..., 1] > 45) & (_h[..., 1] < 170) & (_h[..., 2] > 140)
    body = rest & ~ndi.binary_dilation(skin, iterations=4)  # kroppen uten hånda: spennet per rad
    keep = np.zeros_like(m)
    for y in np.nonzero(tool.any(1))[0]:
        c = np.nonzero(body[y])[0]
        if len(c):
            keep[y, c.min():c.max() + 1] = True
    m2 = rest | (tool & keep) | (tool & ndi.binary_closing(rest, iterations=4))
    if J.OPT.get("tool_box"):  # tynne rester av skaftet langs kanten
        bxm = np.zeros_like(m)
        bb = J.OPT["tool_box"]; bb = [bb] if isinstance(bb[0], int) else bb
        for x0, y0, x1, y1 in bb:
            bxm[y0:y1, x0:x1] = True
        m2 = np.where(bxm, ndi.binary_opening(m2, iterations=4), m2)
    lab, n = ndi.label(m2)
    if n > 1:
        sizes = ndi.sum(m2, lab, range(1, n + 1))
        m2 = lab == (1 + int(np.argmax(sizes)))
    paint = cv2.inpaint(bgr, (tool * 255).astype(np.uint8), 9, cv2.INPAINT_TELEA)
    print("redskapet fjernet fra profilen før speiling:", int(tool.sum()), "px,", int((m & ~m2).sum()), "px ut av silhuetten")
    return paint, (m2 * 255).astype(np.uint8)



def tool_mask(rgb, m, reach=70):
    """Redskapet i en profil på farge: et frø (største jernflekk, eller største treflekk for et krus) i høyde
    med hånda, vokst gjennom jern- og trefarger opptil `reach` px fra frøet. Huden (hånda) er ikke med."""
    import cv2
    from scipy import ndimage as ndi
    hsv = cv2.cvtColor(rgb, cv2.COLOR_RGB2HSV).astype(np.float32)
    h, s, v = hsv[..., 0] * 2, hsv[..., 1], hsv[..., 2]
    ys = np.nonzero(m.any(1))[0]
    top, bot = ys.min(), ys.max()
    reg = np.zeros_like(m)
    reg[int(top + 0.38 * (bot - top)): int(top + 0.70 * (bot - top))] = True
    iron = (s < 45) & (v > 35) & (v < 200) & m
    wood = (h > 18) & (h < 45) & (s > 50) & (s < 190) & (v > 70) & m
    seedc = wood if J.OPT.get("tool_kind") == "wood" else iron
    lab, n = ndi.label(ndi.binary_opening(seedc & reg, iterations=2))
    if n == 0:
        return np.zeros_like(m)
    sizes = ndi.sum(np.ones_like(lab), lab, range(1, n + 1))
    seed = lab == (1 + int(np.argmax(sizes)))
    near = ndi.distance_transform_edt(~seed) < reach
    cand = (iron | wood) & near & reg
    lab, n = ndi.label(ndi.binary_closing(cand | seed, iterations=2))
    keep = np.unique(lab[seed]); keep = keep[keep > 0]
    return np.isin(lab, keep) & m
