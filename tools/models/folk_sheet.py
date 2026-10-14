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
KNEE_AT = 0.52  # kneet: så langt fra ankelen mot hofta
ELBOW = 0.56  # albuen på en rett arm: så langt fra skulderen mot håndleddet
STEP = 6  # rader mellom tverrsnittene (px)
CHIN = 0.1  # haka ligger minst så langt under hodetoppen (del av høyden)
SLOPE = 1.5  # hodet: bredden endrer seg høyst så mange piksler per rad
BELLY = 0.035  # kroppen fra siden: så mye (del av høyden) ut over linja fra brystet til hofta
UNDER = 0.45  # så mye av armens bredde kroppen går inn under armen


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

    # Over skuldrene er det ingen armer å skille fra kroppen, så hullene
    # fylles der (en grå jernhatt ligner bakgrunnen og får ellers hull).
    head_end = top + int(0.17 * H)
    fgh = fg.copy()
    fgh[top - 4: head_end] = ndimage.binary_fill_holes(ndimage.binary_closing(fg[top - 4: head_end], iterations=6))

    def row(y):
        return fgh[y] if y < head_end else fg[y]

    def front(y):
        return runs(row(y), F[0], F[1])

    def side(y):
        return runs(row(y), S[0], S[1])

    def back(y):
        return runs(row(y), B[0], B[1])

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
    # Kanten av skjørtet (eller kjortelen): første rad nedenfra der beina
    # står delt eller kroppen er smalere enn halve skjørtet; skjørtet
    # slutter der, og beina går opp inn under det.
    hem = crotch
    if skirt:
        def lower(y):
            return [r for r in front(y) if abs((r[0] + r[1]) / 2 - cxF) < 0.2 * H and (r[1] - r[0]) > 0.02 * H and not is_arm(r, y)]
        wide = float(np.median([max((r[1] - r[0] for r in lower(y)), default=0) for y in range(crotch, crotch + int(0.15 * H), 4)]))
        hem = next((y for y in range(crotch + int(0.1 * H), ground) if len(lower(y)) >= 2 or max((r[1] - r[0] for r in lower(y)), default=0) < 0.5 * wide), ground)
    ankle = ground - int(ANKLE * H)
    # Hofteleddet og kneet etter kroppens mål: klærne skjuler skrittet, og
    # der buksebeina skilles under et forkle er ikke der beinet begynner.
    hip = min(crotch, ground - int(HIP * H))
    knee = ground - int(KNEE * H)

    # Siden: midtlinja ved hoftene, og hvilken vei figuren ser (tærne).
    hip_side = central(side(crotch), (S[0] + S[1]) // 2)
    sx0 = (hip_side[0] + hip_side[1]) / 2
    # Foten over de nederste radene samlet (en tresko står på hæl og tå
    # med en bue mellom, og bare hælen er nær midten).
    ankle_row = side(ankle - int(0.04 * H))
    tb = central(ankle_row, sx0)
    foot = [r for y in range(ground - int(0.03 * H), ground - 2) for r in side(y) if r[1] > tb[0] - 0.1 * H and r[0] < tb[1] + 0.1 * H]
    ta = (min(r[0] for r in foot), max(r[1] for r in foot)) if foot else tb
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
            # Kroppen går et stykke inn under armen (`UNDER`), så det ikke
            # blir hull mellom dem; ved skulderen dekker den hele bredden
            # og trekkes inn over `span` (skulderen er rund på arket).
            ramp = min(1.0, (y - shoulder) / max(1, int(0.05 * H)))
            for s, edge in (("R", 0), ("L", 1)):
                if y < sep[s]:
                    w = arm_width(s, y) * (1 - UNDER) * ramp
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
    end = hem - int(0.005 * H) if skirt else crotch + int(0.02 * H)
    parts["body"] = [(y, *m) for y in range(top, end, STEP) if (m := body_at(y))]
    if skirt and (m := body_at(end)):
        parts["body"].append((end, *m))
    # Hodet glattes med median over noen rader: hårtupper, en hjelmknapp
    # eller en flekk i masken skal ikke vri tverrsnittene.
    rows = parts["body"]
    head = [i for i, r in enumerate(rows) if r[0] < shoulder]
    cols = list(zip(*rows)) if rows else []
    for i in head:
        win = [j for j in head if abs(j - i) <= 3]
        rows[i] = (rows[i][0], *[float(np.median([rows[j][k] for j in win])) for k in range(1, 5)])
    # Under brystet blir kroppen ikke brått bredere enn radene over: der
    # hånda eller det den holder rører kroppen forfra, er raden ellers en
    # flat skive (smedens hammer mot forkleet).
    rows = parts["body"]
    for i, r in enumerate(rows):
        if r[0] < shoulder + int(0.05 * H):
            continue
        prev = rows[max(0, i - 8): i]
        if not prev:
            continue
        mw = sorted(p[2] for p in prev)[len(prev) // 2]
        md = sorted(p[4] for p in prev)[len(prev) // 2]
        mc = sorted(p[1] for p in prev)[len(prev) // 2]
        if r[2] > mw * 1.08 or r[4] > md * 1.12:
            rows[i] = (r[0], mc if r[2] > mw * 1.08 else r[1], min(r[2], mw * 1.08), r[3], min(r[4], md * 1.12))
    # Fra siden henger armen foran eller bak kroppen, og hånda holder ofte
    # noe (spydet, krukka): mellom brystet og hofta stikker kroppen ikke
    # mer enn `BELLY` ut over linja mellom dem (vakten fikk ellers en
    # hylle framover der hånda med spydet var).
    rows = parts["body"]
    ya, yb = shoulder + int(0.03 * H), max(hand_end.values()) + int(0.03 * H)
    ra = min(rows, key=lambda r: abs(r[0] - ya))
    rb = min(rows, key=lambda r: abs(r[0] - yb))
    slack = BELLY * H
    for i, r in enumerate(rows):
        if not ra[0] < r[0] < rb[0]:
            continue
        t = (r[0] - ra[0]) / max(1, rb[0] - ra[0])
        lo = (ra[3] - ra[4]) + ((rb[3] - rb[4]) - (ra[3] - ra[4])) * t - slack
        hi = (ra[3] + ra[4]) + ((rb[3] + rb[4]) - (ra[3] + ra[4])) * t + slack
        e0, e1 = max(r[3] - r[4], lo), min(r[3] + r[4], hi)
        if e1 > e0:
            rows[i] = (r[0], r[1], r[2], (e0 + e1) / 2, (e1 - e0) / 2)
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
        # Armen holder seg innenfor omrisset forfra: toppen stakk ellers
        # opp over skulderlinja på arket og fikk bakgrunnen som farge.
        clipped = []
        for y, cy, hw in rows:
            X = cxF + cy
            run = next((r for r in front(int(y)) if r[0] <= X < r[1]), None)
            if run is None:
                continue
            lo, hi = max(X - hw, run[0] + 1), min(X + hw, run[1] - 1)
            if hi - lo < 2:
                continue
            clipped.append((y, (lo + hi) / 2 - cxF, (hi - lo) / 2))
        rows = clipped
        # Armen ligger midt i kroppen sett fra siden, litt fram ved hånda.
        parts[f"arm.{s}"] = [(y, cy, hw, body_side_center(y), hw) for y, cy, hw in rows]
    # Beina: fra like over skrittet til sålen; tærne fra sida.
    for s in ("R", "L"):
        rows = []
        for y in range(crotch - int(0.03 * H), ground, STEP):
            rs = [r for r in front(y) if abs((r[0] + r[1]) / 2 - cxF) < 0.15 * H and not is_arm(r, y)]
            rs = [r for r in rs if (r[1] - r[0]) > 0.02 * H]
            if skirt and y < hem - int(0.06 * H):
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
        split = hem + 2 if skirt else crotch
        if rows:
            seen = next((r for r in rows if r[0] >= split), rows[-1])
            rows = [r if r[0] >= split else (r[0], seen[1], seen[2], r[3], min(r[4], seen[4])) for r in rows]
            if not skirt:
                rows = [(y, seen[1], seen[2], body_side_center(y), seen[4]) for y in range(hip - int(0.03 * H), crotch, STEP)] + [r for r in rows if r[0] >= crotch]
        parts[f"leg.{s}"] = rows
    m = dict(
        name=name, height=height, px=px, top=top, ground=ground, H=H, skirt=skirt, facing=facing,
        views={"front": cxF, "back": cxB, "side": sx0, "ground": ground, "side_dir": facing},
        size=fg.shape[::-1],
        y=dict(shoulder=shoulder, crotch=crotch, hem=hem, hip=hip, knee=knee, ankle=ankle, **{f"elbow.{s}": elbow[s] for s in elbow}, **{f"wrist.{s}": wrist[s] for s in wrist}, **{f"hand_end.{s}": hand_end[s] for s in hand_end}),
        parts=parts,
    )
    m["y"].update(model_joints(m))
    return m


def model_joints(m):
    """Leddene lest av formen (eieren 14. oktober: arket definerer modellen,
    og modellen former skjelettet), i arkets rader:

    - ankelen der beinet er smalest over foten,
    - kneet midt mellom hofta og ankelen (litt over),
    - hofta der kroppen er bredest under livet (livet er det smaleste
      under brystet), og ikke under skrittet eller skjørtekanten,
    - håndleddet der armen er smalest over hånda,
    - albuen der armen bøyer forfra; en rett arm har albuen der over- og
      underarmen har kroppens forhold (`ELBOW` av veien til håndleddet).

    Skjules et ledd av klærne (kneet under skjørtet), gjelder forholdet."""
    H, ground, Y, parts = m["H"], m["ground"], m["y"], m["parts"]
    out = {}

    def narrowest(rows, y0, y1, k=2):
        cand = [r for r in rows if y0 <= r[0] <= y1]
        return min(cand, key=lambda r: r[k])[0] if cand else None

    # Ankelen: nedenfra den første raden der beinet fra siden er under
    # halvparten av fotens lengde (foten stikker fram under ankelen).
    legs = [parts[f"leg.{s}"] for s in ("L", "R") if parts.get(f"leg.{s}")]
    ank = []
    for rows in legs:
        sole = [r for r in rows if r[0] >= ground - int(0.025 * H)]
        if not sole:
            continue
        foot = max(r[4] for r in sole)
        up = sorted((r for r in rows if ground - int(0.12 * H) <= r[0] < ground - int(0.02 * H)), key=lambda r: -r[0])
        a = next((r[0] for r in up if r[4] < 0.55 * foot), None)
        if a is not None:
            ank.append(a)
    if ank:
        out["ankle"] = int(np.median(ank))
    body = parts["body"]
    waist = narrowest(body, Y["shoulder"] + int(0.15 * H), Y["shoulder"] + int(0.33 * H))
    if waist is not None:
        lim = min(Y["crotch"], Y["hem"]) if not m["skirt"] else waist + int(0.14 * H)
        cand = [r for r in body if waist < r[0] <= min(lim, waist + int(0.14 * H))]
        if cand:
            # Klærne kan vide seg ut (kjortelen): hofta holdes innenfor
            # kroppens vanlige mål.
            out["hip"] = min(ground - int(0.45 * H), max(ground - int(0.53 * H), max(cand, key=lambda r: r[2])[0]))
    hip, ankle = out.get("hip", Y["hip"]), out.get("ankle", Y["ankle"])
    out["knee"] = int(ankle - KNEE_AT * (ankle - hip))
    for s in ("L", "R"):
        arm = parts.get(f"arm.{s}")
        if not arm:
            continue
        end = Y[f"hand_end.{s}"]
        reach = end - Y["shoulder"]
        w = narrowest(arm, end - int(0.15 * H), end - int(0.06 * H))
        if w is not None and reach >= 0.33 * H:
            out[f"wrist.{s}"] = w
        wrist = out.get(f"wrist.{s}", Y[f"wrist.{s}"])
        sh = Y["shoulder"]

        def at(y):
            r = min(arm, key=lambda r: abs(r[0] - y))
            return np.array([r[1], r[0]], dtype=float)

        def bend(e):
            a, b = at(e) - at(sh + 0.02 * H), at(wrist) - at(e)
            c = a.dot(b) / max(1e-6, np.linalg.norm(a) * np.linalg.norm(b))
            return math.degrees(math.acos(max(-1.0, min(1.0, c))))

        cands = list(range(int(sh + 0.35 * (wrist - sh)), int(sh + 0.65 * (wrist - sh)), 3))
        best = max(cands, key=bend) if cands else None
        if best is not None and bend(best) > 10:
            out[f"elbow.{s}"] = best
        else:
            out[f"elbow.{s}"] = int(sh + ELBOW * (wrist - sh))
    return out


if __name__ == "__main__":
    m = measure(sys.argv[1], float(sys.argv[2]) if len(sys.argv) > 2 else 1.8)
    print({k: v for k, v in m.items() if k != "parts"})
    for k, rows in m["parts"].items():
        print(k, len(rows), rows[:2], rows[-2:])


def head_fit(m):
    """Hodearket (`<navn>_head`: forfra oppe til venstre, profilene oppe
    til høyre og nede til venstre, bakfra nede til høyre) passet inn på
    kroppsarket per visning: målestokken og forskyvningen der omrisset av
    hodet dekker omrisset på kroppsarket best (IoU over hodet og halsen),
    og for siden den profilen (eventuelt speilet) som passer. Gir
    {visning: (fil, s, tx, ty, speil, (x0, x1)), "neck": rad}: en piksel
    (X, Y) på kroppsarket er (tx + s·hx, ty + s·hy) på hodearket."""
    from PIL import Image as _I

    name = m["name"]
    path = os.path.join(REF, f"{name}_head.jpeg")
    if not os.path.exists(path):
        return None
    hm = mask(path)
    lab, k = ndimage.label(hm)
    sizes = ndimage.sum(hm, lab, range(1, k + 1))
    quads = []
    for i in np.argsort(-sizes)[:4]:
        ys, xs = np.where(lab == i + 1)
        quads.append((int(xs.min()), int(xs.max()) + 1, int(ys.min()), int(ys.max()) + 1, i + 1))
    quads.sort(key=lambda b: (b[2] > hm.shape[0] // 2, b[0]))
    front_q, side_qs, back_q = quads[0], [quads[1], quads[2]], quads[3]
    body = mask(os.path.join(REF, f"{name}_body.jpeg"))
    top, H = m["top"], m["H"]
    neck = m["y"]["shoulder"] - int(0.02 * H)

    def fit(view_x, quad, flips):
        # Kroppsarket rundt hodet i denne visningen.
        cx = int(view_x)
        half = int(0.16 * H)
        B = body[top - 4: neck, cx - half: cx + half].astype(np.float32)
        q = (lab[quad[2]: quad[3], quad[0]: quad[1]] == quad[4]).astype(np.uint8) * 255
        best = None
        for flip in flips:
            qq = q[:, ::-1] if flip else q
            for s in np.arange(0.25, 1.4, 0.01):
                # s: kroppspiksler per hodepiksel.
                w, h = max(4, int(qq.shape[1] * s)), max(4, int(qq.shape[0] * s))
                R = np.asarray(_I.fromarray(qq).resize((w, h))) > 127
                R = R[: B.shape[0]]
                if R.shape[0] < B.shape[0] * 0.5:
                    continue
                for dx in range(-24, 25, 3):
                    x0 = (B.shape[1] - w) // 2 + dx
                    if x0 < 0 or x0 + w > B.shape[1]:
                        continue
                    C = np.zeros_like(B, dtype=bool)
                    C[4: 4 + R.shape[0], x0: x0 + w][: B.shape[0] - 4] = R[: B.shape[0] - 4]
                    inter = (C & (B > 0)).sum()
                    iou = inter / max(1, (C | (B > 0)).sum())
                    if best is None or iou > best[0]:
                        # Kroppspiksel X = (cx - half + x0) + s·(hx - qx0)  →  tx = cx - half + x0 - s·qx0.
                        tx = cx - half + x0 - s * (quad[1] - 1 if flip else quad[0])
                        ty = top - 4 + 4 - s * quad[2]
                        best = (iou, float(s), float(tx), float(ty), flip, (quad[0], quad[1]))
        return best

    v = m["views"]
    out = {"file": path, "neck": neck}
    out["front"] = fit(v["front"], front_q, (False,))
    out["back"] = fit(v["back"], back_q, (False,))
    # Profilen må se samme vei som kroppen på sidebildet: omrisset passer
    # nesten like godt speilvendt, og da sto ansiktet bak på hodet.
    def faces(q):
        rows = [(y, runs(lab[y] == q[4], q[0], q[1], least=3)) for y in range(q[2], q[3])]
        rows = [(y, (r[0][0] + r[-1][1]) / 2) for y, r in rows if r]
        n = len(rows)
        head = np.mean([c for _, c in rows[int(n * 0.2): int(n * 0.6)]])
        neck = np.mean([c for _, c in rows[int(n * 0.85):]])
        return 1 if head > neck else -1

    out["side"], q = max(((fit(v["side"], q, (faces(q) != m["facing"],)), q) for q in side_qs), key=lambda b: b[0][0])
    out["quads"] = {"front": front_q, "side": q, "back": back_q}
    out["mask"] = ndimage.binary_fill_holes(ndimage.binary_closing(hm, iterations=6))
    out["labels"] = lab
    # Midt i hodet forfra og bakfra står hodet midt på kroppens hode (en
    # fjær eller en lokk på den ene sida drar passingen til siden).
    body_c = float(np.median([r[1] for r in m["parts"]["body"] if r[0] < m["y"]["shoulder"]]))
    for view, x0 in (("front", v["front"]), ("back", v["back"])):
        iou, s, tx, ty, flip, qx = out[view]
        c = quad_center(out, view)
        want = x0 + (body_c if view == "front" else -body_c)
        out[view] = (iou, s, tx + want - (tx + s * c), ty, flip, qx)
    return out


def quad_rows(fit, view):
    """Bitene av hodet per rad i ruta for `view` på hodearket."""
    hm, lab, q = fit["mask"], fit["labels"], fit["quads"][view]
    sub = hm[q[2]: q[3], q[0]: q[1]] & ndimage.binary_dilation(lab[q[2]: q[3], q[0]: q[1]] == q[4], iterations=8)
    out = {}
    for hy in range(sub.shape[0]):
        rs = runs(sub[hy], 0, sub.shape[1], least=3)
        if rs:
            out[hy + q[2]] = [(a + q[0], b + q[0]) for a, b in rs]
    return out


def quad_center(fit, view):
    """Midtlinja i ruta (median over haka og halsen, der ingen fjær eller
    lokk stikker ut)."""
    fr = quad_rows(fit, view)
    ys = sorted(fr)
    low = ys[int(len(ys) * 0.6): int(len(ys) * 0.85)]
    return float(np.median([sum(max(fr[y], key=lambda r: r[1] - r[0])) / 2 for y in low]))


def head_rows(m, fit):
    """Hodet som tverrsnitt (y, midte forfra, halv bredde, midte fra siden,
    halv dybde; kroppsarkets piksler) lest av hodearket selv: bredden fra
    ruta forfra og forkant og bakkant fra profilen, gjennom passingen i
    `head_fit`. Da står ansiktet forfra og profilen på samme sted på formen,
    og bildene blandes uten striper (eieren 14. oktober: «tekstur rundt
    hodet er rart»). Bredden er symmetrisk om midten (en fjær eller en lokk
    på den ene sida er ikke hode)."""
    v = m["views"]

    def view_rows(view):
        _, s, tx, ty, flip, (qx0, qx1) = fit[view]
        return quad_rows(fit, view), s, tx, ty, flip, qx1

    def to_body(hx, s, tx, flip, qx1):
        return tx + s * ((2 * (qx1 - 1) - hx) if flip else hx)

    fr, sF, txF, tyF, _, _ = view_rows("front")
    sr, sS, txS, tyS, flipS, qx1S = view_rows("side")
    ys = sorted(fr)
    cF = quad_center(fit, "front")
    ys_s = sorted(sr)

    top = int(max(tyF + sF * ys[0], tyS + sS * ys_s[0]))
    rows = []
    for Y in range(top, fit["neck"] + STEP + 1, max(2, STEP // 2)):
        hyF, hyS = int(round((Y - tyF) / sF)), int(round((Y - tyS) / sS))
        if hyF not in fr or hyS not in sr:
            continue
        # Hele raden (hår og ansikt kan være egne biter med en glipe
        # mellom), og forfra symmetrisk om midten.
        a, b = fr[hyF][0][0], fr[hyF][-1][1]
        hw = min(cF - a, b - cF)
        if hw <= 1:
            continue
        a2, b2 = sr[hyS][0][0], sr[hyS][-1][1]
        Xa, Xb = to_body(a2, sS, txS, flipS, qx1S), to_body(b2, sS, txS, flipS, qx1S)
        Xc = to_body(cF, sF, txF, False, 0)
        # Forkant og bakkant i modellens retning (+ er fram mot ansiktet).
        ea, eb = (Xa - v["side"]) * m["facing"], (Xb - v["side"]) * m["facing"]
        rows.append((Y, Xc - v["front"], hw * sF, max(ea, eb), min(ea, eb)))
    # Median over noen rader (hårtupper, en fjær bak øret).
    out = []
    # Forkanten glattes over et lengre stykke: nesa stikker fram over noen
    # få rader, og som del av ellipsen ble den en hylle tvers over ansiktet
    # (ansiktet males uansett på fra bildet forfra).
    for i, r in enumerate(rows):
        win = rows[max(0, i - 3): i + 4]
        wide = rows[max(0, i - 10): i + 11]
        out.append((r[0], *[float(np.median([w[k] for w in (wide if k == 3 else win)])) for k in range(1, 5)]))
    # Haka: der profilen trekker seg mest inn (ikke der hodet er smalest
    # forfra: under et skaut faller håret ut). Hodeleddet står der.
    # Snitt over et stykke før og etter, så nesa (lepper og hake kommer
    # fram igjen under den) ikke tas for haka.
    k = 10
    # Haka er minst `CHIN` av høyden under hodetoppen (nesa og skjegget
    # over den trekker seg også inn).
    lower = [i for i in range(k, len(out) - k) if out[i][0] >= m["top"] + CHIN * m["H"]]
    if lower:
        def recede(i):
            return np.mean([r[3] for r in rows[i: i + k]]) - np.mean([r[3] for r in rows[i - k: i]])
        i = min(lower, key=recede)
        chin = out[i][0]
    else:
        chin = out[-1][0]
    # Hodet (med halsen fra hodearket) går helt ned i skuldrene, så
    # skjøten mot kroppen ligger inne i den; haka er der hodeleddet står.
    out = [r for r in out if r[0] <= fit["neck"] + STEP]
    # Ingen hylle: bredden og kantene vokser høyst `SLOPE` piksler per rad
    # opp eller ned (der håret faller ut under skautet, var hodet en
    # vannrett hylle med strukket tekstur og en sprekk).
    def envelope(vals, ys, k=SLOPE):
        return [min(vals[j] + k * abs(ys[i] - ys[j]) for j in range(len(vals))) for i in range(len(vals))]

    ys_o = [r[0] for r in out]
    hw_e = envelope([r[2] for r in out], ys_o)
    fr_e = envelope([r[3] for r in out], ys_o)
    bk_e = [-x for x in envelope([-r[4] for r in out], ys_o)]
    out = [(r[0], r[1], hw_e[i], fr_e[i], bk_e[i]) for i, r in enumerate(out)]
    # Tverrsnittet er en ellipse foran og en bak en fast akse midt i
    # skallen, så et skaut eller en flette bak ikke flytter ansiktet.
    axis = float(np.median([(r[3] + r[4]) / 2 for r in out]))
    out = [(r[0], r[1], r[2], axis * m["facing"], max(2.0, r[3] - axis), max(2.0, axis - r[4]), "split") for r in out]
    return out, chin
