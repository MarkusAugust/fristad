"""Innbyggernes ark lest som mål (`assets/textures/people/ref/<navn>_body`):
figuren forfra, fra siden og bakfra på grå bakgrunn. Ren numpy, uten
Blender, så målene kan prøves alene:

    python3.13 tools/models/folk_sheet.py smith

`measure` gir landemerkene (skuldre, albuer, håndledd, skrittet, knær,
ankler, tærne) og delene rad for rad: kroppen med hodet (og skjørtet), to
armer og to bein, hver som en stabel tverrsnitt (midte og halve bredder
forfra og fra siden, i piksler). `townsfolk.py` lager formen, skjelettet og
projiseringen av arket fra dem."""

import math
import os
import sys

import numpy as np
from PIL import Image
from scipy import ndimage

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REF = os.path.join(ROOT, "assets", "textures", "people", "ref")

# Hånda fra håndleddet til fingertuppene, og ankelen over sålen, som del av
# høyden (vanlige kroppsmål; arkene viser ikke leddene).
HAND = 0.105
ANKLE = 0.05
HIP = 0.5
REACH = 0.46  # fra skulderen til fingertuppene, høyst (del av høyden)
ARM_OUT = 0.11  # en bit som ligger lenger ut fra midten enn dette (del av høyden), kan være en arm
KNEE = 0.28
STEP = 6  # rader mellom tverrsnittene (px)


def mask(path):
    """Figuren på arket. Bakgrunnen er en svak gradient, så den måles per
    rad fra kantene; bakkestreken (tynn og vannrett) tas bort; bare små hull
    fylles, så mellomrommet mellom armen og kroppen blir stående."""
    im = np.asarray(Image.open(path).convert("RGB")).astype(float)
    edge = np.concatenate([im[:, :30], im[:, -30:]], 1)
    bg = np.median(edge, 1)[:, None, :]
    fg = (np.abs(im - bg).sum(2) > 30) | ((im.max(2) - im.min(2)) > 22)
    fg = ndimage.binary_opening(fg, structure=np.ones((9, 1)))
    fg = ndimage.binary_opening(fg, iterations=2)
    holes = ndimage.binary_fill_holes(fg) & ~fg
    lab, k = ndimage.label(holes)
    if k:
        sizes = ndimage.sum(holes, lab, range(1, k + 1))
        fg |= np.isin(lab, [i + 1 for i in range(k) if sizes[i] < 1500])
    return fg


def views(fg):
    """De tre figurene (forfra, siden, bakfra) fra venstre mot høyre, som
    bokser (x0, x1, y0, y1)."""
    lab, k = ndimage.label(fg)
    sizes = ndimage.sum(fg, lab, range(1, k + 1))
    boxes = []
    for i in np.argsort(-sizes)[:3]:
        ys, xs = np.where(lab == i + 1)
        boxes.append((int(xs.min()), int(xs.max()) + 1, int(ys.min()), int(ys.max()) + 1))
    if len(boxes) < 3:
        raise SystemExit(f"fant {len(boxes)} figurer på arket, ikke tre")
    boxes.sort()
    return boxes


def runs(row, x0, x1, least=5):
    """Sammenhengende biter av figuren i raden mellom x0 og x1."""
    r = row[x0:x1].astype(np.int8)
    d = np.diff(np.concatenate([[0], r, [0]]))
    s, e = np.where(d == 1)[0], np.where(d == -1)[0]
    return [(int(a + x0), int(b + x0)) for a, b in zip(s, e) if b - a >= least]


def measure(name, height):
    path = os.path.join(REF, f"{name}_body.jpeg")
    fg = mask(path)
    F, S, B = views(fg)
    ground = F[3]
    # Toppen av hodet over kroppens midtlinje (en høygaffel eller et spyd
    # kan stikke høyere enn hodet).
    lower = fg[ground - (ground - F[2]) // 2: ground - (ground - F[2]) // 10, F[0]:F[1]]
    cols = lower.sum(0)
    cx0 = F[0] + int((cols * np.arange(len(cols))).sum() / max(1, cols.sum()))
    top = next(y for y in range(F[2], ground) if fg[y, cx0 - 12: cx0 + 12].any())
    H = ground - top
    px = height / H

    def front(y):
        return runs(fg[y], F[0], F[1])

    def side(y):
        return runs(fg[y], S[0], S[1])

    def back(y):
        return runs(fg[y], B[0], B[1])

    def central(rs, c):
        # Biten som rommer midtlinja (eller er nærmest den).
        return min(rs, key=lambda r: 0 if r[0] <= c < r[1] else min(abs(r[0] - c), abs(r[1] - c))) if rs else None

    # Midtlinja: midt i hodet forfra og bakfra (hodet står fritt).
    head_rows = range(top + int(0.03 * H), top + int(0.09 * H))
    cxF = float(np.median([sum(central(front(y), (F[0] + F[1]) // 2)) / 2 for y in head_rows]))
    cxB = float(np.median([sum(central(back(y), (B[0] + B[1]) // 2)) / 2 for y in head_rows]))
    # Hodebredden der ansiktet er (en hattebrem eller hjelm er bredere).
    face = [np.diff(central(front(y), cxF))[0] for y in range(top + int(0.04 * H), top + int(0.13 * H)) if front(y)]
    head_w = float(np.percentile(face, 20))

    # Skuldrene: der figuren forfra blir mye bredere enn hodet.
    widths = [(y, np.diff(central(front(y), cxF))[0]) for y in range(top + int(0.13 * H), top + int(0.35 * H)) if front(y)]
    wide = {y for y, w in widths if w > 1.7 * head_w}
    span = int(0.04 * H)
    # Skuldrene er brede over et stykke; en brem er bare noen få rader.
    shoulder = next((y for y, w in widths if all(z in wide for z in range(y, y + span) if z in dict(widths))), None)
    if shoulder is None:
        # Store hoder (gutten) eller hette: der bredden vokser mest.
        shoulder = max(zip(widths, widths[6:]), key=lambda p: p[1][1] - p[0][1])[1][0]

    # Armene står fri av kroppen nedenfor albuen: bitene utenfor den midtre.
    free = {"R": {}, "L": {}}  # forfra: R til venstre i bildet, L til høyre
    for y in range(shoulder, ground - int(0.2 * H)):
        rs = front(y)
        # Kroppen (og beina under den) står innenfor; en arm henger utenfor.
        inner = [r for r in rs if abs((r[0] + r[1]) / 2 - cxF) < ARM_OUT * H]
        if not inner:
            continue
        c = (min(r[0] for r in inner), max(r[1] for r in inner))
        for side_name, pick in (("R", [r for r in rs if r[1] <= c[0]]), ("L", [r for r in rs if r[0] >= c[1]])):
            if pick:
                a, b = min(r[0] for r in pick), max(r[1] for r in pick)
                free[side_name][y] = (a, b)
    arms_raw = {k: dict(v) for k, v in free.items()}

    def is_arm(r, y):
        return any(y in arms_raw[k] and r[0] < arms_raw[k][y][1] and r[1] > arms_raw[k][y][0] for k in ("R", "L"))

    sep, hand_end = {}, {}
    for s in ("R", "L"):
        ys = sorted(free[s])
        if not ys:
            raise SystemExit(f"{name}: fant ikke armen {s} fri av kroppen")
        # Den lengste sammenhengende strekningen er armen.
        best, cur = [], [ys[0]]
        for a, b in zip(ys, ys[1:]):
            if b - a <= 3:
                cur.append(b)
            else:
                best, cur = max(best, cur, key=len), [b]
        best = max(best, cur, key=len)
        # Fingertuppene når ikke lenger enn `REACH` under skulderen (et
        # sverd ved hofta eller et skaft i hånda fortsetter ellers armen).
        best = [y for y in best if y <= shoulder + int(REACH * H)]
        sep[s], hand_end[s] = best[0], best[-1]
        free[s] = {y: free[s][y] for y in best}
    wrist = {s: hand_end[s] - int(HAND * H) for s in free}
    elbow = {s: shoulder + int(0.52 * (wrist[s] - shoulder)) for s in free}
    for s in free:
        # En arm som holder noe foran seg (krus, morter) ser kort ut forfra:
        # leddene fordeles langs det som synes, så ingen bein blir null.
        reach = hand_end[s] - shoulder
        if reach < 0.33 * H:
            elbow[s] = shoulder + int(0.45 * reach)
            wrist[s] = shoulder + int(0.75 * reach)

    # Skrittet: der kroppen deler seg i to bein som står delt helt ned.
    crotch = None
    for y in range(shoulder + int(0.25 * H), ground - int(0.08 * H)):
        rs = [r for r in front(y) if not is_arm(r, y)]
        legs = [r for r in rs if (r[1] - r[0]) > 0.03 * H and abs((r[0] + r[1]) / 2 - cxF) < 0.15 * H]
        if len(legs) == 2 and all(len([r for r in front(z) if (r[1] - r[0]) > 0.02 * H and abs((r[0] + r[1]) / 2 - cxF) < 0.15 * H and not is_arm(r, z)]) >= 2 for z in range(y, ground - int(0.08 * H), 8)):
            crotch = y
            break
    skirt = crotch is None or crotch > ground - int(0.2 * H)
    if skirt:
        crotch = ground - int(0.47 * H)
    ankle = ground - int(ANKLE * H)
    # Hofteleddet og kneet etter kroppens mål: klærne skjuler skrittet, og
    # der buksebeina skilles under et forkle er ikke der beinet begynner.
    hip = min(crotch, ground - int(HIP * H))
    knee = ground - int(KNEE * H)

    # Siden: midtlinja ved hoftene, og hvilken vei figuren ser (tærne).
    hip_side = central(side(crotch), (S[0] + S[1]) // 2)
    sx0 = (hip_side[0] + hip_side[1]) / 2
    toe_row, ankle_row = side(ground - 3), side(ankle - int(0.04 * H))
    ta, tb = central(toe_row, sx0), central(ankle_row, sx0)
    facing = 1 if (ta[1] - tb[1]) > (tb[0] - ta[0]) else -1

    def body_at(y):
        """Kroppen (eller hodet) i raden: midte og halve bredder forfra og fra
        siden. Over armene er den midtre biten kroppen med armene inntil, så
        armenes bredde trekkes fra."""
        c = central(front(y), cxF)
        if c is None:
            return None
        a, b = c
        if y >= crotch - int(0.03 * H):
            # Ved skrittet er kroppen begge beina samlet.
            near = [r for r in front(y) if abs((r[0] + r[1]) / 2 - cxF) < 0.22 * H and (r[1] - r[0]) > 0.02 * H]
            if near:
                a, b = min(r[0] for r in near), max(r[1] for r in near)
        if shoulder < y:
            for s, edge in (("R", 0), ("L", 1)):
                if y < sep[s] and y > shoulder:
                    w = arm_width(s, y)
                    if edge == 0:
                        a += int(w)
                    else:
                        b -= int(w)
        sc = central(side(y), sx0)
        if sc is None:
            return None
        return ((a + b) / 2 - cxF, (b - a) / 2, (sc[0] + sc[1]) / 2 - sx0, (sc[1] - sc[0]) / 2)

    def arm_width(s, y):
        # Over der armen står fri: bredden der den først står fri, litt
        # tykkere opp mot skulderen (ermet).
        a, b = free[s][sep[s]]
        w = b - a
        t = max(0.0, min(1.0, (sep[s] - y) / max(1, sep[s] - shoulder)))
        return w * (1.0 + 0.15 * t)

    def body_side_center(y):
        sc = central(side(y), sx0)
        return (sc[0] + sc[1]) / 2 - sx0 if sc else 0.0

    parts = {}
    # Kroppen med hodet, ned til skrittet (eller bakken under skjørtet).
    end = ground if skirt else crotch + int(0.02 * H)
    parts["body"] = [(y, *m) for y in range(top, end, STEP) if (m := body_at(y))]
    if skirt:
        # Nederste rad med kropp (sålen under skjørtet kan mangle forfra).
        for y in range(ground - 1, ground - int(0.05 * H), -1):
            if (m := body_at(y)):
                parts["body"].append((y, *m))
                break
    # Skulderlinja skrår fra nakken ut til skuldrene (ellers en flat
    # hylle med hjørner over armene).
    rows = parts["body"]
    neck = [r for r in rows if r[0] < shoulder]
    if neck:
        nw = neck[-1][2]
        span = int(0.06 * H)
        for i, r in enumerate(rows):
            if shoulder <= r[0] < shoulder + span:
                t = (r[0] - shoulder) / span
                k = 1 - (1 - t) ** 2
                rows[i] = (r[0], r[1], nw + (r[2] - nw) * (0.35 + 0.65 * k), r[3], r[4])
    # Under brystet blir kroppen ikke brått bredere enn radene over: der
    # hånda eller det den holder rører kroppen forfra, er raden ellers en
    # flat skive (smedens hammer mot forkleet).
    rows = parts["body"]
    for i, r in enumerate(rows):
        if r[0] < shoulder + int(0.2 * H):
            continue
        prev = rows[max(0, i - 8): i]
        if not prev:
            continue
        mw = sorted(p[2] for p in prev)[len(prev) // 2]
        md = sorted(p[4] for p in prev)[len(prev) // 2]
        mc = sorted(p[1] for p in prev)[len(prev) // 2]
        if r[2] > mw * 1.08 or r[4] > md * 1.12:
            rows[i] = (r[0], mc if r[2] > mw * 1.08 else r[1], min(r[2], mw * 1.08), r[3], min(r[4], md * 1.12))
    # Armene: fra skulderen (inne i kroppen) til fingertuppene.
    for s in ("R", "L"):
        rows = []
        a, b = free[s][sep[s]]
        c0 = (a + b) / 2
        edge = central(front(shoulder + int(0.02 * H)), cxF)
        sh = (edge[0] + arm_width(s, shoulder) * 0.5) if s == "R" else (edge[1] - arm_width(s, shoulder) * 0.5)
        for y in range(shoulder, sep[s], STEP):
            t = (y - shoulder) / max(1, sep[s] - shoulder)
            rows.append((y, sh + (c0 - sh) * t - cxF, arm_width(s, y) / 2))
        for y in range(sep[s], hand_end[s] + 1, STEP):
            if y in free[s]:
                a, b = free[s][y]
                rows.append((y, (a + b) / 2 - cxF, (b - a) / 2))
        # Hånda er høyst litt bredere enn håndleddet (det hånda holder, som
        # smedens hammer, gjør ellers tverrsnittene til flate skiver).
        wr = [hw for y, cy, hw in rows if abs(y - wrist[s]) < 0.02 * H]
        cap = (sum(wr) / len(wr) if wr else max(r[2] for r in rows)) * 1.25
        rows = [(y, cy, min(hw, cap) if y > wrist[s] else hw) for y, cy, hw in rows]
        # Toppen av armen rundes inn i skulderen.
        y0, c0, w0 = rows[0]
        drop = int(0.035 * H)
        rows = [r for r in rows if r[0] >= y0 + drop]
        y0, c0, w0 = rows[0]
        rows = [(y0 - int(w0 * 0.75 * f), c0, w0 * math.sqrt(max(0.0, 1 - f * f)) + 1) for f in (0.98, 0.9, 0.75, 0.5, 0.25)] + rows
        # Armen ligger midt i kroppen sett fra siden, litt fram ved hånda.
        parts[f"arm.{s}"] = [(y, cy, hw, body_side_center(y), hw) for y, cy, hw in rows]
    # Beina: fra like over skrittet til sålen; tærne fra sida.
    for s in ("R", "L"):
        rows = []
        for y in range(crotch - int(0.03 * H), ground, STEP):
            rs = [r for r in front(y) if abs((r[0] + r[1]) / 2 - cxF) < 0.15 * H and not is_arm(r, y)]
            rs = [r for r in rs if (r[1] - r[0]) > 0.02 * H]
            if skirt:
                if y < ground - int(0.12 * H):
                    continue
            if len(rs) >= 2:
                r = rs[0] if s == "R" else rs[-1]
            elif len(rs) == 1:
                # Skjørtet eller kroppen dekker begge: halvparten hver.
                a, b = rs[0]
                m = (a + b) / 2
                r = (a, m) if s == "R" else (m, b)
            else:
                continue
            sc = central(side(y), sx0)
            if sc is None:
                continue
            sd = (sc[1] - sc[0]) / 2
            if y < ankle:
                sd = min(sd, (r[1] - r[0]) / 2 * 1.25)
            rows.append((y, (r[0] + r[1]) / 2 - cxF, (r[1] - r[0]) / 2, (sc[0] + sc[1]) / 2 - sx0 if y >= ankle else body_side_center(y), sd))
        # Over der beina skilles (under forkleet eller kjortelen) har låret
        # samme bredde som der det først synes, ikke halve kroppen.
        if not skirt and rows:
            seen = next((r for r in rows if r[0] >= crotch), rows[-1])
            rows = [r if r[0] >= crotch else (r[0], seen[1], seen[2], r[3], min(r[4], seen[4])) for r in rows]
            rows = [(y, seen[1], seen[2], body_side_center(y), seen[4]) for y in range(hip - int(0.03 * H), crotch, STEP)] + [r for r in rows if r[0] >= crotch]
        parts[f"leg.{s}"] = rows
    return dict(
        name=name, height=height, px=px, top=top, ground=ground, H=H, skirt=skirt, facing=facing,
        views={"front": cxF, "back": cxB, "side": sx0, "ground": ground, "side_dir": facing},
        size=fg.shape[::-1],
        y=dict(shoulder=shoulder, crotch=crotch, hip=hip, knee=knee, ankle=ankle, **{f"elbow.{s}": elbow[s] for s in elbow}, **{f"wrist.{s}": wrist[s] for s in wrist}, **{f"hand_end.{s}": hand_end[s] for s in hand_end}),
        parts=parts,
    )


if __name__ == "__main__":
    m = measure(sys.argv[1], float(sys.argv[2]) if len(sys.argv) > 2 else 1.8)
    print({k: v for k, v in m.items() if k != "parts"})
    for k, rows in m["parts"].items():
        print(k, len(rows), rows[:2], rows[-2:])
