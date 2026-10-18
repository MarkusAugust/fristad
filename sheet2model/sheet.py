"""Steg 1: skjær ut visningene fra arket og segmenter dem mot den grå bakgrunnen.

Skriver `<visning>_rgb.png` og `<visning>_mask.png` i arbeidsmappa, og `maskert.json` med det som ble tatt ut
(staver, redskap i den speilede profilen, slettebokser), så det kan sjekkes.
"""
import json
import time

import numpy as np

SKIN = dict(h=(2, 28), s=(45, 170), v=(140, 256))


def hsv(rgb):
    import cv2
    x = cv2.cvtColor(np.ascontiguousarray(rgb), cv2.COLOR_RGB2HSV).astype(np.float32)
    return x[..., 0] * 2, x[..., 1], x[..., 2]


def skin_mask(rgb, vmin=140):
    h, s, v = hsv(rgb)
    return (h > SKIN["h"][0]) & (h < SKIN["h"][1]) & (s > SKIN["s"][0]) & (s < SKIN["s"][1]) & (v > vmin)


def tiles(job):
    """Rutene som rgb (H, W, 3) per visning med eget bilde."""
    import cv2
    im = cv2.imread(job.ref)[:, :, ::-1]
    out = {}
    for k, v in job.real_views().items():
        if v.file:
            t = cv2.imread(v.file)[:, :, ::-1]
            out[k] = np.ascontiguousarray(t)
            continue
        if job.crops:  # figuren i sin egen stripe, sentrert i en rute på TW med bakgrunnsfarge rundt
            bg = np.median(im[:8].reshape(-1, 3), 0).astype(np.uint8)
            x0, x1 = job.crops[v.rc[1]]
            t = np.empty((job.TH, job.TW, 3), np.uint8); t[:] = bg
            o = (job.TW - (x1 - x0)) // 2
            t[:, o:o + x1 - x0] = im[:job.TH, x0:x1]
            out[k] = t
        else:
            r, c = v.rc
            out[k] = np.ascontiguousarray(im[r * job.TH:(r + 1) * job.TH, c * job.TW:(c + 1) * job.TW])
    return out


def background(f):
    return np.median(np.concatenate([f[:8].reshape(-1, 3), f[-8:].reshape(-1, 3),
                                     f[:, :8].reshape(-1, 3), f[:, -8:].reshape(-1, 3)]), 0)


def largest(fg):
    from scipy import ndimage as ndi
    lab, n = ndi.label(fg)
    if n <= 1:
        return fg
    sizes = ndi.sum(fg, lab, range(1, n + 1))
    return lab == (1 + int(np.argmax(sizes)))


def segment(job, k, t, log):
    """Silhuetten i én rute: avstand fra bakgrunnsfargen og metning, største bit, hull lukket (eller åpne der de
    har bakgrunnsfarge), gulvlinje og staver tatt ut."""
    import cv2
    from scipy import ndimage as ndi
    opt = job.opt
    f = t.astype(np.float32)
    bg = background(f)
    d = np.linalg.norm(f - bg, axis=2)
    sat = f.max(2) - f.min(2)
    fg = (d > 14) | (sat > 14)
    if opt.get("floor_line"):  # tynne vannrette linjer (gulvet under føttene): fjern med vertikal åpning
        thin = fg & ~ndi.binary_opening(fg, structure=np.ones((9, 1)))
        fg &= ~thin
    fg = ndi.binary_opening(fg, iterations=2)
    box = (opt.get("part_box") or {}).get(k)
    if box:  # én del av en sammensatt gjenstand: bare det som ligger i boksen (x0, y0, x1, y1 i ruta)
        x0, y0, x1, y1 = box
        keep = np.zeros_like(fg); keep[y0:y1, x0:x1] = True
        fg &= keep
    fg = largest(fg)
    if opt.get("drop_pole"):
        # lange staver (høygaffel, spyd, pilegrimsstav) er tynne og loddrette: tas ut av silhuetten, ellers
        # blir toppen av staven «issen» i innrettingen
        thin = fg & ~ndi.binary_opening(fg, structure=np.ones((1, 41)))
        lab, n = ndi.label(thin)
        pole = np.zeros_like(thin)
        for i, sl in enumerate(ndi.find_objects(lab)):
            if sl is not None and (sl[0].stop - sl[0].start) > 140:
                pole |= lab == i + 1
        if pole.any():
            log.append(dict(view=k, what="stav", px=int(pole.sum())))
        fg &= ~ndi.binary_dilation(pole, iterations=2)
        fg = largest(fg)
    fg = ndi.binary_closing(fg, iterations=6)
    if opt.get("open_holes"):
        # hull med bakgrunnsfarge (mellom arm og kropp, mellom beina) står åpne; små hull fylles
        holes = ndi.binary_fill_holes(fg) & ~fg
        lab, n = ndi.label(holes)
        for i in range(1, n + 1):
            h = lab == i
            if h.sum() < opt.get("min_hole", 400) or np.median(d[h]) > 14:
                fg |= h
    else:
        fg = ndi.binary_fill_holes(fg)
    for x0, y0, x1, y1 in opt.get("erase", {}).get(k, []):  # rester av redskap, satt for hånd per ark
        fg[y0:y1, x0:x1] = False
        log.append(dict(view=k, what="slettet boks", box=[x0, y0, x1, y1]))
    fg = largest(fg)
    m = (fg * 255).astype(np.uint8)
    return cv2.GaussianBlur(m, (5, 5), 0)


def seg(job):
    import cv2
    t0 = time.time()
    log = []
    for k, t in tiles(job).items():
        m = segment(job, k, t, log)
        cv2.imwrite(job.p(f"{k}_rgb.png"), t[:, :, ::-1])
        cv2.imwrite(job.p(f"{k}_mask.png"), m)
        ys, xs = np.nonzero(m > 127)
        print(k, "bbox x", xs.min(), xs.max(), "y", ys.min(), ys.max())
    for k, v in job.views.items():  # speilede visninger
        if v.mirror_of:
            rgb = cv2.imread(job.p(f"{v.mirror_of}_rgb.png"))
            m = cv2.imread(job.p(f"{v.mirror_of}_mask.png"), 0)
            if job.opt.get("tool_box") and job.opt.get("mirror_strip_tool", True):
                rgb, m, n = strip_tool(job, rgb, m)
                log.append(dict(view=k, what="redskap tatt ut før speiling", px=int(n)))
            cv2.imwrite(job.p(f"{k}_rgb.png"), rgb[:, ::-1])
            cv2.imwrite(job.p(f"{k}_mask.png"), m[:, ::-1])
    json.dump(log, open(job.p("maskert.json"), "w"), indent=1)
    job.time("seg", time.time() - t0)
    return log


def tool_boxes(job):
    b = job.opt.get("tool_box")
    if not b:
        return []
    return [b] if isinstance(b[0], int) else b


def strip_tool(job, bgr, m8):
    """Redskapet bare den ene hånda holder, skal ikke speiles over i den andre: fjern det fra profilen før
    speilingen. Det som stikker ut forbi kroppen (og hånda) tas bort fra silhuetten; der redskapet ligger foran
    kroppen, blir silhuetten stående, og fargen males over fra kroppen rundt (inpaint)."""
    import cv2
    from scipy import ndimage as ndi
    m = m8 > 127
    rgb = bgr[:, :, ::-1]
    tool = tool_mask(job, rgb, m)
    boxes = tool_boxes(job)
    if boxes:  # redskapet innenfor en boks i profilen, på farge, uten hud
        h, s_, v_ = hsv(rgb)
        iron = (s_ < 45) & (v_ > 90) & (v_ < 225)
        wood = (h > 15) & (h < 50) & (s_ > 25) & (s_ < 200) & (v_ > 110)
        skin = skin_mask(rgb)
        box = np.zeros_like(m)
        for x0, y0, x1, y1 in boxes:
            box[y0:y1, x0:x1] = True
        noskin = ~ndi.binary_dilation(skin, iterations=3)
        if job.opt.get("tool_strip", "box") == "box":
            tool = box & m & noskin  # hele boksen utenom hånda (redskapet ligger inne i silhuetten)
        else:  # på farge: jern og lyst tre, vokst litt inn i omrisset, men ikke inn i mørke klær
            col = ndi.binary_closing((iron | wood) & box & m & noskin, iterations=3)
            tool = col | (ndi.binary_dilation(col, iterations=6) & (v_ > 55) & box & m & noskin)
            tool = ndi.binary_fill_holes(ndi.binary_closing(tool, iterations=4)) & box & m & noskin
    tool = ndi.binary_dilation(tool, iterations=2 if boxes else 4) & m
    np.save(job.p("tool_src.npy"), tool)
    rest = m & ~tool
    skin = skin_mask(rgb)
    body = rest & ~ndi.binary_dilation(skin, iterations=4)  # kroppen uten hånda: spennet per rad
    keep = np.zeros_like(m)
    for y in np.nonzero(tool.any(1))[0]:
        c = np.nonzero(body[y])[0]
        if len(c):
            keep[y, c.min():c.max() + 1] = True
    m2 = rest | (tool & keep) | (tool & ndi.binary_closing(rest, iterations=4))
    if boxes:  # tynne rester av skaftet langs kanten
        bxm = np.zeros_like(m)
        for x0, y0, x1, y1 in boxes:
            bxm[y0:y1, x0:x1] = True
        m2 = np.where(bxm, ndi.binary_opening(m2, iterations=4), m2)
    m2 = largest(m2)
    paint = cv2.inpaint(bgr, (tool * 255).astype(np.uint8), 9, cv2.INPAINT_TELEA)
    print("redskapet fjernet fra profilen før speiling:", int(tool.sum()), "px,", int((m & ~m2).sum()), "px ut av silhuetten")
    return paint, (m2 * 255).astype(np.uint8), tool.sum()


def tool_mask(job, rgb, m, reach=70):
    """Redskapet i en profil på farge: et frø (største jernflekk, eller største treflekk for et krus) i høyde
    med hånda, vokst gjennom jern- og trefarger opptil `reach` px fra frøet. Huden (hånda) er ikke med."""
    from scipy import ndimage as ndi
    h, s, v = hsv(rgb)
    ys = np.nonzero(m.any(1))[0]
    top, bot = ys.min(), ys.max()
    reg = np.zeros_like(m)
    reg[int(top + 0.38 * (bot - top)): int(top + 0.70 * (bot - top))] = True
    iron = (s < 45) & (v > 35) & (v < 200) & m
    wood = (h > 18) & (h < 45) & (s > 50) & (s < 190) & (v > 70) & m
    seedc = wood if job.opt.get("tool_kind") == "wood" else iron
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


def load_mask(job, k):
    import cv2
    return cv2.imread(job.p(f"{k}_mask.png"), 0) > 127


def load_rgb(job, k):
    import cv2
    return cv2.imread(job.p(f"{k}_rgb.png"))[:, :, ::-1]
