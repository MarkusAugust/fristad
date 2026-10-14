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
ARM_WIDE = 1.6  # en rad gjennom armen er høyst så mange ganger så bred som armen vanligvis
SHAFT = 0.026  # under hånda er en stripe smalere enn dette (del av høyden) et skaft
SHAFT_LONG = 0.04  # når den er lengre enn dette
THING = SHAFT  # det arket viser utenfor delene, bredere enn et skaft, er en ting figuren bærer (skaftene blir egne gjenstander)
THING_BACK = 0.015  # bak på kroppen (fletta) er en ting minst så bred
THING_TALL = 0.03  # og høyere enn dette
CARRIED = 0.7  # det figuren bærer på ryggen er høyst så bredt som kroppen forfra
CHIN = 0.1  # haka ligger minst så langt under hodetoppen (del av høyden)
HEAD_UPPER = 0.7  # hodet passes mot så stor del ovenfra av veien fra issen til halsen
BRIM = 0.02  # en brem er minst så høy (del av høyden)
SLOPE_BRIM = 4.0  # og vokser høyst så mange piksler per rad
SLOPE = 1.5  # hodet: bredden endrer seg høyst så mange piksler per rad
HOLE_BG = 15  # et hull i figuren som skiller seg mer enn dette fra bakgrunnen (sum over rgb), er tøy
SHELF = 0.015  # kanten fra siden springer mer enn dette (del av høyden) fra én rad til neste: en hylle
BELLY = 0.035  # kroppen fra siden: så mye (del av høyden) ut over linja fra brystet til hofta


def mask(path):
    """Figuren på arket. Bakgrunnen er en svak gradient, så den måles per
    rad fra kantene; bakkestreken (tynn og vannrett) tas bort; små hull og
    hull som ikke har bakgrunnens farge fylles, så mellomrommet mellom armen
    og kroppen blir stående."""
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
        # Et stort hull er bakgrunn (mellom armen og kroppen) bare om det har
        # bakgrunnens farge: lyst tøy nær den (vaskekonas skjørt) ga hull
        # i folder midt i figuren, og kroppen fikk en glipe tvers over.
        d = np.abs(im - bg).sum(2)
        bgish = ndimage.median(d, lab, range(1, k + 1))
        fg |= np.isin(lab, [i + 1 for i in range(k) if sizes[i] < 1500 or bgish[i] > HOLE_BG])
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
        # Fingertuppene når ikke lenger enn `REACH` under skulderen (et
        # sverd ved hofta eller et skaft i hånda fortsetter ellers armen).
        ys = [y for y in ys if y <= shoulder + int(REACH * H)]
        if not ys:
            raise SystemExit(f"{name}: fant ikke armen {s} fri av kroppen innen rekkevidden")
        strips, cur = [], [ys[0]]
        for a, b in zip(ys, ys[1:]):
            if b - a <= 3:
                cur.append(b)
            else:
                strips.append(cur)
                cur = [b]
        strips.append(cur)
        # Den lengste strekningen er armen, og strekningene over den hører
        # med: en arm som holder noe inntil kroppen (kurven) står fri ved
        # overarmen og igjen ved hånda, og med bare den nederste ble armen
        # en rett linje fra skulderen og hull innenfor albuen (eieren
        # 14. oktober). Mellom strekningene går armen rett.
        main = max(strips, key=len)
        best = [y for st in strips if st[0] <= main[0] and len(st) >= 0.02 * H for y in st]
        best = sorted(set(best) | set(main))
        # Et skaft i hånda (høygaffelen, spydet) fortsetter under hånda som
        # en smal stripe; den er ikke armen (armen gikk ellers ned langs
        # skaftet til hofta). Fingertuppene smalner bare et lite stykke.
        tail = 0
        for y in reversed(best):
            a, b = free[s][y]
            if b - a >= SHAFT * H:
                break
            tail += 1
        if tail and best[-1] - best[-tail] >= SHAFT_LONG * H:
            best = best[:-tail]
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
        def grounded(r, y):
            # Et bein står på bakken: midt i biten er arket fylt helt ned.
            # Bøtta vaskekona bærer ved skjørtet er også en bit for seg, men
            # henger i lufta (skjørtet sluttet ellers ved bøtta, og kroppen
            # fikk en glipe tvers over).
            x = int((r[0] + r[1]) / 2)
            return bool(fg[y: ground - int(0.01 * H), x].all())

        wide = float(np.median([max((r[1] - r[0] for r in lower(y)), default=0) for y in range(crotch, crotch + int(0.15 * H), 4)]))
        hem = next((y for y in range(crotch + int(0.1 * H), ground) if len([r for r in lower(y) if grounded(r, y)]) >= 2 or max((r[1] - r[0] for r in lower(y)), default=0) < 0.5 * wide), ground)
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
        # Der armen ligger inntil kroppen forfra, fyller kroppen hele
        # omrisset, og armen legges utenpå: et smalere bryst ga hull
        # innenfor albuen der armen på arket går et annet sted enn armen i
        # modellen (eieren 14. oktober, `folk_check`).
        sc = central(side(y), sx0)
        if sc is None:
            return None
        if skirt and y >= crotch:
            # Kanten av skjørtet bretter seg, og fra siden er den flere
            # biter: alle hører med (tjenestejentas skjørt ble kappet).
            near = [r for r in side(y) if abs((r[0] + r[1]) / 2 - sx0) < 0.2 * H]
            if near:
                sc = (min(r[0] for r in near), max(r[1] for r in near))
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
    # Skjørtet går helt ned til kanten (raden over `hem`).
    end = hem - 2 if skirt else crotch + int(0.02 * H)
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
    # Under brystet, der en hånd er, blir kroppen ikke brått bredere enn
    # radene over: der hånda eller det den holder rører kroppen forfra, er
    # raden ellers en flat skive (smedens hammer mot forkleet).
    rows = parts["body"]
    # Bare der en hånd er: skjørtet som vider seg ut mot kanten ble ellers
    # kappet (og kjøpmannens mage, lenger ned).
    at_hand = [(wrist[s] - int(0.03 * H), hand_end[s] + int(0.03 * H)) for s in hand_end]
    for i, r in enumerate(rows):
        if r[0] < shoulder + int(0.05 * H) or not any(a <= r[0] <= b for a, b in at_hand):
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
    # noe (spydet, krukka): mellom brystet og hofta stikker kroppen ikke mer
    # enn `BELLY` ut over linja mellom dem der kanten springer brått ut
    # (vakten fikk ellers en hylle framover der underarmen med spydet var).
    # En mage buler jevnt og står (kjøpmannen).
    rows = parts["body"]
    ya, yb = shoulder + int(0.03 * H), max(hand_end.values()) + int(0.03 * H)
    ra = min(rows, key=lambda r: abs(r[0] - ya))
    rb = min(rows, key=lambda r: abs(r[0] - yb))
    slack = BELLY * H
    # Hyllene sjekkes ned til skrittet eller skjørtekanten (spydskaftet
    # inntil kjortelen under hånda); linja fortsetter rett under hånda.
    end_y = hem if skirt else crotch
    span = [i for i, r in enumerate(rows) if ra[0] < r[0] < end_y]

    def limits(r):
        t = min(1.0, (r[0] - ra[0]) / max(1, rb[0] - ra[0]))
        lo = (ra[3] - ra[4]) + ((rb[3] - rb[4]) - (ra[3] - ra[4])) * t - slack
        hi = (ra[3] + ra[4]) + ((rb[3] + rb[4]) - (ra[3] + ra[4])) * t + slack
        return lo, hi

    for side_ in (1, -1):
        edge = [rows[i][3] + side_ * rows[i][4] for i in span]
        line = [limits(rows[i])[1 if side_ > 0 else 0] for i in span]
        out_ = [(e - l) * side_ > 0 for e, l in zip(edge, line)]
        j = 0
        while j < len(span):
            if not out_[j]:
                j += 1
                continue
            k = j
            while k + 1 < len(span) and out_[k + 1]:
                k += 1
            lo_, hi_ = max(0, j - 1), min(len(span) - 1, k + 1)
            jump = max(abs(edge[q + 1] - edge[q]) for q in range(lo_, hi_)) if hi_ > lo_ else 0
            if jump > SHELF * H:
                for q in range(j, k + 1):
                    i = span[q]
                    r = rows[i]
                    lo, hi = limits(r)
                    e0, e1 = max(r[3] - r[4], lo), min(r[3] + r[4], hi)
                    if e1 > e0:
                        rows[i] = (r[0], r[1], r[2], (e0 + e1) / 2, (e1 - e0) / 2)
            j = k + 1
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
        # En underarm som ligger vannrett (tjenestejenta holder krukka foran
        # seg) gir rader like brede som hele underarmen: en flat skive ved
        # albuen. Armen er høyst `ARM_WIDE` ganger så bred som den er
        # vanligvis, og beholder den ytre delen (albuen).
        usual = float(np.median([free[s][y][1] - free[s][y][0] for y in free[s]]))
        for y in range(sep[s], hand_end[s] + 1, STEP):
            if y in free[s]:
                a, b = free[s][y]
                if b - a > ARM_WIDE * usual and y < wrist[s]:
                    a, b = (a, a + ARM_WIDE * usual) if s == "R" else (b - ARM_WIDE * usual, b)
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
        # Midt i kroppen slik den står etter hyllene (den rå biten fra siden
        # tok med spydet, og armen sto foran vakta).
        def body_mid(y):
            r = interp(parts["body"], y)
            return r[2] if r else body_side_center(y)

        parts[f"arm.{s}"] = [(y, cy, hw, body_mid(y), hw) for y, cy, hw in rows]
    # Beina: fra like over skrittet til sålen; tærne fra sida.
    for s in ("R", "L"):
        rows = []
        # Fra siden følger beinet sin egen bit nedover (øksa som henger ved
        # leggen eller hånda dro ellers midten fram og tilbake, og beina
        # sto i sikksakk: eieren 14. oktober «beina er vrengt»).
        track = sx0
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
            sc = central(side(y), track)
            if sc is None:
                continue
            sd = (sc[1] - sc[0]) / 2
            mid = (sc[0] + sc[1]) / 2
            if y < ankle:
                # Over ankelen er beinet høyst litt dypere enn det er bredt,
                # midt i biten det følger.
                sd = min(sd, (r[1] - r[0]) / 2 * 1.25)
                track = mid
            rows.append((y, (r[0] + r[1]) / 2 - cxF, (r[1] - r[0]) / 2, mid - sx0, sd))
        # Over der beina skilles (under forkleet eller kjortelen) har låret
        # samme bredde som der det først synes, ikke halve kroppen.
        split = hem + 2 if skirt else crotch
        if rows:
            seen = next((r for r in rows if r[0] >= split), rows[-1])
            rows = [r if r[0] >= split else (r[0], seen[1], seen[2], r[3], min(r[4], seen[4])) for r in rows]
            if not skirt:
                rows = [(y, seen[1], seen[2], body_side_center(y), seen[4]) for y in range(hip - int(0.03 * H), crotch, STEP)] + [r for r in rows if r[0] >= crotch]
        # Fra siden er beinet to rette stykker som møtes i kneet (låret og
        # leggen): midten legges på dem, og rader som skiller seg ut (hånda
        # eller øksa ved låret) tas bort før linjene trekkes.
        # Linjene trekkes bare gjennom radene der beinet synes (under skrittet
        # eller skjørtet), og kneet står der linjene passer best.
        idx = [i for i, r in enumerate(rows) if r[0] < ankle]
        seen_idx = [i for i in idx if rows[i][0] >= (hem if skirt else crotch)]
        if len(seen_idx) >= 4:
            ys0 = np.array([rows[i][0] for i in seen_idx], float)
            xs = np.array([rows[i][3] for i in seen_idx], float)

            def legfit(k):
                ys = ys0 - k
                A = np.stack([np.ones_like(ys), np.minimum(ys, 0), np.maximum(ys, 0)], 1)
                keep = np.ones(len(ys), bool)
                for _ in range(4):
                    coef = np.linalg.lstsq(A[keep], xs[keep], rcond=None)[0]
                    res = np.abs(xs - A @ coef)
                    keep = res <= 2.5 * max(2.0, float(np.median(res[keep])))
                    if keep.sum() < 4:
                        break
                return float(np.median(res)), coef

            cands = [k for k in range(knee - int(0.06 * H), knee + int(0.06 * H) + 1, STEP) if ys0.min() < k < ys0.max()] or [knee]
            k, coef = min(((k, legfit(k)[1]) for k in cands), key=lambda kc: legfit(kc[0])[0])
            for i in idx:
                y = rows[i][0] - k
                x = coef[0] + coef[1] * min(y, 0) + coef[2] * max(y, 0)
                sd = rows[i][4]
                if rows[i][0] < (hem if skirt else crotch):
                    # Inne i kroppen holdes låret innenfor den (linja kom
                    # ellers ut foran kjortelen på vakta).
                    b = interp(parts["body"], rows[i][0])
                    if b:
                        sd = min(sd, b[3])
                        x = min(max(x, b[2] - b[3] + sd), b[2] + b[3] - sd)
                rows[i] = (*rows[i][:3], float(x), sd)
        parts[f"leg.{s}"] = rows
    parts.update(held(parts, front, cxF, H, shoulder, min(hem, ground - int(0.2 * H))))
    parts.update(carried(parts, side, sx0, facing, H, shoulder, min(hem, ground - int(0.2 * H))))
    m = dict(
        name=name, height=height, px=px, top=top, ground=ground, H=H, skirt=skirt, facing=facing,
        views={"front": cxF, "back": cxB, "side": sx0, "ground": ground, "side_dir": facing},
        size=fg.shape[::-1],
        y=dict(shoulder=shoulder, crotch=crotch, hem=hem, hip=hip, knee=knee, ankle=ankle, **{f"elbow.{s}": elbow[s] for s in elbow}, **{f"wrist.{s}": wrist[s] for s in wrist}, **{f"hand_end.{s}": hand_end[s] for s in hand_end}),
        parts=parts,
    )
    m["y"].update(model_joints(m))
    return m


def interp(rows, y):
    """Tverrsnittet i høyden `y` mellom radene (som meshen binder dem
    sammen), eller None utenfor delen."""
    if not rows or y < rows[0][0] or y > rows[-1][0]:
        return None
    for a, b in zip(rows, rows[1:]):
        if a[0] <= y <= b[0]:
            t = (y - a[0]) / max(1, b[0] - a[0])
            return tuple(p + (q - p) * t for p, q in zip(a[1:5], b[1:5]))
    return rows[-1][1:5]


def held(parts, front, cxF, H, y0, y1):
    """Det arket viser forfra som verken kroppen, armene eller beina dekker
    (kurven fiskerkona bærer på armen, krukka, hammeren mot forkleet), som
    egne deler `held.R`/`held.L` ved armen den er nærmest. Uten dem var det
    hull gjennom figuren innenfor albuen, der bildet av tingen bare lå på
    skjørtet bak (eieren 14. oktober). Tingen er like dyp som den er bred,
    høyst like dyp som kroppen, og står midt i kroppen sett fra siden."""
    keys = [k for k in parts if k == "body" or k.startswith(("arm.", "leg."))]
    found = {"R": [], "L": []}
    for y in range(y0, y1, STEP):
        cover = []
        for k in keys:
            r = interp(parts[k], y)
            if r:
                cover.append((cxF + r[0] - r[1] - 2, cxF + r[0] + r[1] + 2))
        body = interp(parts["body"], y)
        if body is None:
            continue
        for a, b in front(y):
            # Det som står igjen av løpet når det dekkede er trukket fra.
            segs = [(a, b)]
            for c0, c1 in cover:
                segs = [q for p0, p1 in segs for q in ((p0, min(p1, c0)), (max(p0, c1), p1)) if q[1] - q[0] > 0]
            for p0, p1 in segs:
                if p1 - p0 < THING * H:
                    continue
                c = (p0 + p1) / 2
                arms = {s: interp(parts.get(f"arm.{s}", []), y) for s in ("R", "L")}
                s = min((s for s in arms if arms[s]), key=lambda s: abs(cxF + arms[s][0] - c), default="R" if c < cxF else "L")
                hw = (p1 - p0) / 2
                found[s].append((y, c - cxF, hw, body[2], min(hw, body[3])))
    out = {}
    for s, rows in found.items():
        # Bare sammenhengende biter av en viss høyde (ikke en kant av omrisset).
        runs, cur = [], []
        for r in rows:
            if cur and (r[0] - cur[-1][0] > STEP or abs(r[1] - cur[-1][1]) > r[2] + cur[-1][2]):
                runs.append(cur)
                cur = []
            cur.append(r)
        if cur:
            runs.append(cur)
        runs = [r for r in runs if r[-1][0] - r[0][0] >= THING_TALL * H]
        if runs:
            best = max(runs, key=lambda r: sum(q[2] for q in r))
            out[f"held.{s}"] = best
    return out


def carried(parts, side, sx0, facing, H, y0, y1):
    """Det arket viser fra siden som ingen del dekker: sekken på ryggen til
    gutten, kappa, kurven på hofta sett fra siden. Blir delene
    `held.back` midt på kroppen sett forfra (høyst `CARRIED` av kroppens
    bredde), og følger brystet; det som står foran en hånd
    (krukka) blir `held.R`/`held.L` ved hånda, om den ikke alt bærer noe
    som synes forfra (`held`)."""
    keys = [k for k in parts if k == "body" or k.startswith(("arm.", "leg.", "held."))]
    # Hånda på hver arm (de nederste radene); det som henger foran hånda
    # (krukka tjenestejenta holder) følger hånda som ligger nærmest midten
    # forfra, den som holder noe foran seg.
    hands = {k[-1]: (r[-1][0] - int(0.16 * H), r[-1][0] + int(0.03 * H), r[-1][1]) for k, r in parts.items() if k.startswith("arm.") and r}
    found = {"back": [], "front": [], **{s: [] for s in hands}}
    for y in range(y0, y1, STEP):
        body = interp(parts["body"], y)
        if body is None:
            continue
        cover = []
        for k in keys:
            r = interp(parts[k], y)
            if r:
                cover.append((sx0 + r[2] - r[3] - 2, sx0 + r[2] + r[3] + 2))
        for a, b in side(y):
            segs = [(a, b)]
            for c0, c1 in cover:
                segs = [q for p0, p1 in segs for q in ((p0, min(p1, c0)), (max(p0, c1), p1)) if q[1] - q[0] > 0]
            for p0, p1 in segs:
                c = (p0 + p1) / 2
                where = "front" if (c - sx0 - body[2]) * facing > 0 else "back"
                # Bak på kroppen (fletta, sekken) holder ingen et skaft.
                if p1 - p0 < (THING_BACK if where == "back" else THING) * H:
                    continue
                d = (p1 - p0) / 2
                near = [s for s, (h0, h1, hx) in hands.items() if h0 <= y <= h1]
                if where == "front" and near:
                    s = min(near, key=lambda s: abs(hands[s][2]))
                    found[s].append((y, hands[s][2], min(body[1] * CARRIED, d * 1.2), c - sx0, d))
                    continue
                if where == "front":
                    # Foran kroppen og over hendene er det underarmen som
                    # holder noe (vakta): en skive på brystet ble en hylle.
                    continue
                found[where].append((y, body[0], min(body[1] * CARRIED, d * 1.2), c - sx0, d))
    out = {}
    for w, rows in found.items():
        runs, cur = [], []
        for r in rows:
            if cur and (r[0] - cur[-1][0] > STEP or abs(r[3] - cur[-1][3]) > r[4] + cur[-1][4]):
                runs.append(cur)
                cur = []
            cur.append(r)
        if cur:
            runs.append(cur)
        runs = [r for r in runs if r[-1][0] - r[0][0] >= THING_TALL * H]
        if f"held.{w}" in parts:
            continue
        # De to største bitene (fletta og albuen som stikker bakover).
        for n, run in enumerate(sorted(runs, key=lambda r: -sum(q[4] for q in r))[:2]):
            out[f"held.{w}" + ("" if n == 0 else str(n + 1))] = run
    return out


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
        upper = 4 + int(HEAD_UPPER * (neck - top))
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
                    # Bare hodet over haka: skuldrene som hodearket tar med
                    # (tjenestejenta), dro ellers hodet 18 % for smalt.
                    Cu, Bu = C[:upper], B[:upper] > 0
                    inter = (Cu & Bu).sum()
                    iou = inter / max(1, (Cu | Bu).sum())
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
    # Profilen som passer omrisset best, speilet eller ikke: kroppen på
    # sidebildet ser samme vei som hodet der (en regel med halsen bak midten
    # av hodet tok feil når skjegget henger, og ga ansiktet i bakhodet).
    out["side"], q = max(((fit(v["side"], q, (False, True)), q) for q in side_qs), key=lambda b: b[0][0])
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
        wide = rows[max(0, i - 6): i + 7]
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

    def brimmed(raw):
        """`envelope`, men en hattebrem er hode: et bånd over `BRIM` av
        høyden som stikker ut over det over og under (bondens stråhatt ble
        en smal kjegle) får bratte kanter (`SLOPE_BRIM`), ikke vertikale."""
        env = envelope(raw, ys_o)
        wide = [raw[i] > env[i] + 0.05 * abs(env[i]) for i in range(len(raw))]
        span = int(0.05 * m["H"])
        i = 0
        while i < len(raw):
            if not wide[i]:
                i += 1
                continue
            j = i
            while j + 1 < len(raw) and wide[j + 1]:
                j += 1
            peak = max(raw[i: j + 1])
            above = [raw[k] for k in range(len(raw)) if ys_o[i] - span <= ys_o[k] < ys_o[i]]
            below = [raw[k] for k in range(len(raw)) if ys_o[j] < ys_o[k] <= ys_o[j] + span]
            out_ = peak - 0.3 * abs(peak)
            if ys_o[j] - ys_o[i] >= BRIM * m["H"] and above and below and min(above) < out_ and min(below) < out_:
                soft = envelope(raw, ys_o, SLOPE_BRIM)
                for k in range(i, j + 1):
                    env[k] = max(env[k], soft[k])
            i = j + 1
        return env

    hw_e = brimmed([r[2] for r in out])
    # Forkanten (pannen, nesa, haka) tas ikke med: der kappet regelen haka.
    fr_e = [r[3] for r in out]
    bk_e = [-x for x in brimmed([-r[4] for r in out])]
    out = [(r[0], r[1], hw_e[i], fr_e[i], bk_e[i]) for i, r in enumerate(out)]
    # Tverrsnittet er en ellipse foran og en bak en fast akse midt i
    # skallen, så et skaut eller en flette bak ikke flytter ansiktet.
    axis = float(np.median([(r[3] + r[4]) / 2 for r in out]))
    out = [(r[0], r[1], r[2], axis * m["facing"], max(2.0, r[3] - axis), max(2.0, axis - r[4]), "split") for r in out]
    return out, chin
