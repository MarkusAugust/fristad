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
SLEEVE = 0.7  # smalner armen brått til mindre enn dette av raden over, slutter ermet og hånda begynner
LEG_WIDE = 1.8  # er biten fra siden så mange ganger bredere enn beinet forfra, er det kappa rundt
LEG_WIN, LEG_JUMP = 11, 0.02  # kanten av beinet forfra: medianen over så mange rader, og så langt fra den (andel av høyden) er noe som henger inntil
SHIN_LO, SHIN_HI = 0.6, 1.25  # under skjørtet: beinet mellom så mange ganger leggen lenger ned
TOUCH = 0.012  # en ting ligger inntil armen når den er så nær (andel av høyden; bøtta henger litt fra hånda)
HANGING = 0.3  # det som henger under hånda er høyst så høyt (andel av høyden)
SPLIT = 0.04  # en kjole åpen foran: halvdelene er så nær midten (andel av høyden)
HELD_LOW = 0.12  # det hånda bærer forfra går ned hit over bakken (andel av høyden; bøtta henger lavt)
HAND_DEPTH = 0.6  # hånda er så tykk som den er bred
HAND_TIP = 0.025  # og rundes av over så mye av høyden mot fingertuppene
HAND_OUT = 0.008  # stikker sidebildet så mye ut foran kroppen ved hånda (del av høyden), står hånda der
ARM_GAP = 0.08  # en bit av armen under en glipe kortere enn dette (del av høyden) hører til armen
SHAFT = 0.026  # under hånda er en stripe smalere enn dette (del av høyden) et skaft
SHAFT_LONG = 0.04  # når den er lengre enn dette
THING = SHAFT  # det arket viser utenfor delene, bredere enn et skaft, er en ting figuren bærer (skaftene blir egne gjenstander)
THING_BACK = 0.015  # bak på kroppen (fletta) er en ting minst så bred
THING_TALL = 0.03  # og høyere enn dette
CARRIED = 0.7  # det figuren bærer på ryggen er høyst så bredt som kroppen forfra
CHIN = 0.1  # haka ligger minst så langt under hodetoppen (del av høyden)
HEAD_UPPER = 0.7  # hodet passes mot så stor del ovenfra av veien fra issen til halsen
RECEDE = 0.025  # under haka trekker profilen seg så mye inn (del av høyden) mot halsen
LIPS = 0.05  # kommer profilen fram igjen innen så langt under (del av høyden), var innskjæringen munnen
BEARD_BELOW = 0.06  # så langt under halsen (del av høyden) leses hodearket etter skjegg
BEARD_WIDEN = 1.1  # blir hodet bredere enn dette under halsen, er det skuldrene
SKIN_TOL = 70  # en piksel er hud når den er så nær fargen på kinnet (sum over rgb)
FINGER = 0.3  # rader nederst på hånda smalere enn så mye av hånda er fingertupper som peker ut
TOOL_REACH, TOOL_DOWN, TOOL_W = 0.1, 0.3, 0.02  # verktøyet forfra: så langt til siden for hånda og så langt ned, og der det er minst så bredt teller det nedover fra siden (andel av høyden)
TOOL_NEAR, TOOL_ABOVE, TOOL_BELOW, TOOL_GAP = 0.1, 0.03, 0.15, 0.015  # verktøyet: fra siden så langt til hver side av hånda og så langt ned, fra så langt under den; forfra fra så langt over den (andel av høyden)
SKIN_TONE, SKIN_SAT, SKIN_DARK, SKIN_LIGHT = 0.06, 0.035, 0.6, 1.5  # hud fra siden: fargetonen (r og g av summen), metningen og lysstyrken mot hånda forfra
SKIN_W, SKIN_OUT, SKIN_ROWS = 0.01, 0.025, 6  # armen fra siden etter hudfargen: minste bredde og avvik fra naboene (andel av høyden), rader
SKIN_GAP = 6
FRAMED_MIN = 0.2  # og bare når håret tar minst så stor del av bredden
FRAMED = 0.8  # bredden følger huden fra så langt ned fra issen mot haka (munnen)
SKIN_STRIPES = 14  # et skjegg er stripete: spredningen i fargen under munnen er større enn dette  # så mange piksler uten hud etter hverandre, og huden er slutt
HEAD_BELOW = 0.012  # hodet går så langt under haka (del av høyden)
HEAD_SMOOTH = 0.006  # hodets bredde og bakkant glattes over så mye av høyden (sigma)
EAR = 0.045  # det som stikker ut på sidene av hodet over kortere stykke enn dette (del av høyden) er ører
BRIM_HIGH = 0.45  # ligger det bredeste av hodet over så stor del av høyden ned mot haka, er det en brem
UNDER_BRIM = 0.09  # og under den er utstikk kortere enn dette (del av høyden) hår, ikke hode
EAR_FROM = 0.35  # ørene sitter under så stor del av hodets høyde fra issen
EAR_LONG = 0.075  # og der er utstikk kortere enn dette (del av høyden) ører
DOME_OVER = 1.08  # over issen kan profilen stikke så mye ut over kuppelen
CROWN = 0.3  # issen er minst en kuppel så langt ned (del av hodets høyde)
FACE_FROM = 0.1  # fremspringene tones inn over så mye av hodets høyde under issen
FACE_RELIEF = 0.03  # fremspring i ansiktet smalere enn dette (del av høyden) er nese, lepper, hake
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


def measure_posed(name, height, tools=(), rest=()):
    """`measure`, med armene i `rest` bygd hengende etter bildet bakfra.
    Det armen og det den holder dekket forfra (`occluded`: rader med fra og
    til mot midten forfra), males over på kroppen (`townsfolk.unoccluded`)."""
    m = measure(name, height, tools)
    if not rest:
        return m
    occ = []
    for s in rest:
        for k, rows in m["parts"].items():
            if k == f"arm.{s}":
                occ += [(r[0], r[1] - r[2], r[1] + r[2]) for r in rows if r[0] >= m["y"][f"elbow.{s}"]]
            elif k.startswith(f"held.{s}"):
                occ += [(r[0], r[1] - r[2], r[1] + r[2]) for r in rows]
    m2 = measure(name, height, tools, rest, (min(r[0] for r in occ), max(r[0] for r in occ)) if occ else None)
    m2["occluded"] = sorted(occ)
    return m2


def measure(name, height, tools=(), rest=(), occluded_rows=None):
    path = os.path.join(REF, f"{name}_body.jpeg")
    fg = mask(path)
    rgb = np.asarray(Image.open(path).convert("RGB")).astype(float)
    bare, bare_side, skins = [], {}, []  # bar hud på armen fra siden: (y, fra, til) mot midten fra siden, og hudfargen på hendene
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

    back_view = back  # `back` brukes til noe annet lenger ned

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

    def back_runs(y):
        # Bildet bakfra speilet inn i bildet forfra (figurens høyre arm
        # står til venstre i begge).
        return sorted((int(cxF - (b - cxB)), int(cxF - (a - cxB))) for a, b in back_view(y))

    # En arm som arket viser bøyd forfra (den holder krukka eller morteren)
    # men hengende bakfra, bygges hengende etter bildet bakfra (`rest`):
    # to positurer i én arm ble bånd og skiver (eieren 15. oktober).
    for s in rest:
        free[s] = {}
        for y in range(shoulder, ground - int(0.2 * H)):
            rs = back_runs(y)
            inner = [r for r in rs if abs((r[0] + r[1]) / 2 - cxF) < ARM_OUT * H]
            if not inner:
                continue
            c = (min(r[0] for r in inner), max(r[1] for r in inner))
            pick = [r for r in rs if r[1] <= c[0]] if s == "R" else [r for r in rs if r[0] >= c[1]]
            if pick:
                free[s][y] = (min(r[0] for r in pick), max(r[1] for r in pick))

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
        # Og under den: hånda etter en glipe (der hånda rører kappa) hører
        # til armen, ellers ble den en løs ting som svevde (apotekeren).
        last = main[-1]
        for st in strips:
            if st[0] > main[-1] and len(st) >= 0.02 * H and st[0] - last <= ARM_GAP * H:
                a0, b0 = free[s][last]
                a1, b1 = free[s][st[0]]
                if a1 < b0 + 0.04 * H and b1 > a0 - 0.04 * H:
                    best += st
                    last = st[-1]
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
        if skirt and not a <= cxF < b:
            # Kjolen eller kappa er åpen foran nede ved kanten: kroppen er
            # begge halvdelene (kjøpmannen mistet den ene).
            left = [r for r in front(y) if r[1] <= cxF and cxF - r[1] < SPLIT * H]
            right = [r for r in front(y) if r[0] > cxF and r[0] - cxF < SPLIT * H]
            if left and right:
                a, b = max(left, key=lambda r: r[1])[0], min(right, key=lambda r: r[0])[1]
        if not skirt and y >= crotch - int(0.03 * H):
            # Ved skrittet er kroppen begge beina samlet (under et skjørt er
            # det skjørtet; der tok regelen med hendene, og skriveren fikk
            # tallerkener ved hoftene).
            # Bare det som står på bakken (bondens knivslire henger i lufta),
            # og bredere enn et skaft (høygaffelen når også bakken).
            near = [r for r in front(y) if abs((r[0] + r[1]) / 2 - cxF) < 0.22 * H and (r[1] - r[0]) > 0.02 * H
                    and (r[0] <= cxF < r[1] or (r[1] - r[0] > 2 * SHAFT * H and fg[y: ground - int(0.01 * H), int((r[0] + r[1]) / 2)].all()))]
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
            # biter: de som ligger innenfor skjørtet hører med (tjenestejentas
            # skjørt ble kappet), ikke tåa som stikker fram under det
            # (apotekeren fikk en kloss).
            near = [r for r in side(y) if abs((r[0] + r[1]) / 2 - sx0) < 0.2 * H and r[0] >= sc[0] - 0.03 * H and r[1] <= sc[1] + 0.03 * H]
            if near:
                sc = (min(sc[0], min(r[0] for r in near)), max(sc[1], max(r[1] for r in near)))
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
    at_hand = {s: (wrist[s] - int(0.03 * H), hand_end[s]) for s in hand_end}
    # Målt mot kroppen over hendene (radene like over krøp utover, og
    # bonden fikk en planke av knivslira).
    hand_top = min(a for a, b in at_hand.values())
    ref = [r for r in rows if hand_top - int(0.08 * H) <= r[0] < hand_top - int(0.01 * H)]
    for i, r in enumerate(rows):
        # Bare kanten på den sida der hånda rører kroppen (står hånda fri,
        # er det skjørtet som vider seg ut: vaskekona og fiskerkona mistet
        # skjørtet ved hånda).
        sides = [s for s, (a, b) in at_hand.items() if a <= r[0] <= b and r[0] not in free[s]]
        if r[0] < shoulder + int(0.05 * H) or not sides:
            continue
        prev = ref or rows[max(0, i - 8): i]
        if not prev:
            continue
        mw = sorted(p[2] for p in prev)[len(prev) // 2]
        md = sorted(p[4] for p in prev)[len(prev) // 2]
        mc = sorted(p[1] for p in prev)[len(prev) // 2]
        e0, e1 = r[1] - r[2], r[1] + r[2]
        if "R" in sides:
            e0 = max(e0, mc - mw * 1.08)
        if "L" in sides:
            e1 = min(e1, mc + mw * 1.08)
        rows[i] = (r[0], (e0 + e1) / 2, (e1 - e0) / 2, r[3], min(r[4], md * 1.12))
    # Der en arm bygd etter bildet bakfra (`rest`) lå bøyd over kroppen
    # forfra og fra siden (underarmen og krukka), følger kroppen linja
    # mellom radene over og under: den ble ellers en kile fram og bredere
    # på den sida (tjenestejenta).
    if rest and occluded_rows:
        rows = parts["body"]
        oy0, oy1 = occluded_rows
        out_ = [i for i, r in enumerate(rows) if oy0 <= r[0] <= oy1]
        if out_:
            ia, ib = max(0, out_[0] - 1), min(len(rows) - 1, out_[-1] + 1)
            ra_, rb_ = rows[ia], rows[ib]
            for i in out_:
                r = rows[i]
                t = (r[0] - ra_[0]) / max(1, rb_[0] - ra_[0])
                lerp = lambda u, v: u + (v - u) * t  # noqa: E731
                # Forfra: kanten på armens side.
                e_out = []
                for s_ in rest:
                    sgn = -1 if s_ == "R" else 1
                    e_line = lerp(ra_[1] + sgn * ra_[2], rb_[1] + sgn * rb_[2])
                    e_out.append((sgn, e_line))
                lo_, hi_ = r[1] - r[2], r[1] + r[2]
                for sgn, e_line in e_out:
                    if sgn < 0:
                        lo_ = max(lo_, e_line)
                    else:
                        hi_ = min(hi_, e_line)
                # Fra siden: forkanten.
                f_line = lerp(ra_[3] + facing * ra_[4], rb_[3] + facing * rb_[4])
                b_ = r[3] - facing * r[4]
                f_ = r[3] + facing * r[4]
                f_ = min(f_, f_line) if facing > 0 else max(f_, f_line)
                rows[i] = (r[0], (lo_ + hi_) / 2, (hi_ - lo_) / 2, (b_ + f_) / 2, abs(f_ - b_) / 2)
    # Det som henger under hånda (vaskekonas bøtte) ligger inntil skjørtet
    # forfra, og kroppen ble så bred som bøtta. Kanten forfra springer brått
    # ut og brått inn igjen der: radene imellom legges på linja mellom
    # radene utenfor, og tingen blir sin egen del (`held`).
    rows = parts["body"]
    idx = [i for i, r in enumerate(rows) if shoulder + int(0.05 * H) < r[0] < (hem if skirt else crotch)]
    for side_ in (1, -1):
        edge = {i: rows[i][1] + side_ * rows[i][2] for i in idx}
        i = 1
        while i < len(idx):
            a, b = idx[i - 1], idx[i]
            if (edge[b] - edge[a]) * side_ <= SHELF * H:
                i += 1
                continue
            back = next((j for j in range(i + 1, len(idx)) if rows[idx[j]][0] - rows[b][0] <= HANGING * H
                         and (edge[idx[j - 1]] - edge[idx[j]]) * side_ > SHELF * H and (edge[idx[j]] - edge[a]) * side_ < 2 * SHELF * H), None)
            if back is None:
                # Henger tingen helt ned til kanten av kjortelen (bondens
                # knivslire), springer kanten aldri inn igjen: radene ut til
                # kanten legges på kanten over.
                tail = idx[i:]
                if tail and rows[tail[-1]][0] - rows[b][0] <= HANGING * H and rows[tail[-1]][0] >= rows[idx[-1]][0] \
                        and all((edge[q] - edge[a]) * side_ > SHELF * H for q in tail):
                    for q in tail:
                        r = rows[q]
                        other = r[1] - side_ * r[2]
                        rows[q] = (r[0], (edge[a] + other) / 2, abs(edge[a] - other) / 2, r[3], r[4])
                        edge[q] = edge[a]
                    break
                i += 1
                continue
            e0, e1, y0_, y1_ = edge[a], edge[idx[back]], rows[a][0], rows[idx[back]][0]
            for j in range(i, back):
                q = idx[j]
                r = rows[q]
                e = e0 + (e1 - e0) * (r[0] - y0_) / max(1, y1_ - y0_)
                other = r[1] - side_ * r[2]
                rows[q] = (r[0], (e + other) / 2, abs(e - other) / 2, r[3], r[4])
                edge[q] = e
            i = back + 1
    # Fra siden henger armen foran eller bak kroppen, og hånda holder ofte
    # noe (spydet, krukka): mellom brystet og hofta stikker kroppen ikke mer
    # enn `BELLY` ut over linja mellom dem der kanten springer brått ut
    # (vakten fikk ellers en hylle framover der underarmen med spydet var).
    # En mage buler jevnt og står (kjøpmannen).
    rows = parts["body"]
    # Linja ender over skrittet (gutten har hendene under det, og linja ned
    # til beina kappet forsiden av kjortelen).
    ya, yb = shoulder + int(0.03 * H), min(max(hand_end.values()) + int(0.03 * H), (hem if skirt else crotch) - int(0.02 * H))
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

    # Over linja (ved skuldrene) tar sidearket med fletta eller håret som
    # henger bak: ryggen står der ikke mer enn `SHELF` bak ryggen under (det
    # ble en hylle over ryggen på tjenestejenta, og fletta blir sin egen del).
    rb_back = ra[3] - facing * ra[4]
    for i, r in enumerate(rows):
        if shoulder - int(0.06 * H) <= r[0] < ra[0]:
            b_ = r[3] - facing * r[4]
            f_ = r[3] + facing * r[4]
            if (rb_back - b_) * facing > SHELF * H:
                b_ = rb_back - facing * SHELF * H
                rows[i] = (r[0], r[1], r[2], (b_ + f_) / 2, abs(f_ - b_) / 2)
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
    # Armen ligger midt i kroppen sett fra siden, litt fram ved hånda.
    # Midt i kroppen slik den står etter hyllene (den rå biten fra siden
    # tok med spydet, og armen sto foran vakta).
    def body_mid(y):
        r = interp(parts["body"], y)
        return r[2] if r else body_side_center(y)

    def skin_side(s, rows_):
        """Midten av armen fra siden (mot `sx0`) i radene der arket viser
        bar hud nær kroppen, i fargen hånda har forfra."""
        ys_h = [y for y in free[s] if hand_end[s] - 0.05 * H <= y <= hand_end[s] - 0.01 * H]
        if not ys_h:
            return {}
        skin = np.median([rgb[y, (free[s][y][0] + free[s][y][1]) // 2] for y in ys_h], 0)
        skins.append(skin)
        out = {}
        for y, hw in rows_:
            if y < elbow[s]:
                continue
            yy = int(y)
            # Bare innenfor omrisset fra siden (ikke bildet ved siden av).
            run_ = central(side(yy), sx0 + body_mid(y))
            if run_ is None:
                continue
            x0, x1 = int(run_[0]), int(run_[1])
            # Fargetonen, ikke lysstyrken: hånda forfra ligger i skygge
            # og armen fra siden i lys.
            ok = skin_like(rgb[yy, x0:x1], skin) & fg[yy, x0:x1]
            # Strekene mellom fingrene deler ikke hånda.
            ok = ndimage.binary_closing(ok, np.ones(SKIN_GAP * 2 + 1, bool)) & fg[yy, x0:x1]
            runs_ = runs(ok, 0, len(ok))
            runs_ = [(a + x0, b + x0) for a, b in runs_ if b - a >= SKIN_W * H]
            if not runs_:
                continue
            # Den bredeste biten er armen (en flekk i skyggen er smal).
            a, b = max(runs_, key=lambda r: r[1] - r[0])
            c = (a + b) / 2
            out[y] = c - sx0
            bare.append((y, a - sx0, b - sx0))
        # Enkeltrader som skiller seg ut fra naboene (en flekk på
        # skjørtet) tas bort.
        ks = sorted(out)
        vs = np.array([out[k] for k in ks], float)
        keep = {}
        for i, k in enumerate(ks):
            near_ = vs[max(0, i - 4): i + 5]
            if abs(vs[i] - np.median(near_)) <= SKIN_OUT * H:
                keep[k] = out[k]
        return keep

    # Sidearket viser bare armen nærmest (den andre armen fant kurven og
    # fisken og ble stripete bakfra).
    # Sidearket viser den sida figuren snur mot oss: ser den mot høyre i
    # bildet, er det høyre arm (smedens venstre arm fant hånda med hammeren
    # og gikk fram som en hale; flest hudrader valgte feil arm).
    near_arm = "R" if facing > 0 else "L"
    tool_hand = {}  # nederste rad med hud på armen fra siden: (y, midte)

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
        # (det hånda holder blir en egen ting, `held`; et krus i hånda ble
        # ellers en kile på vertshusverten)
        cap = (sum(wr) / len(wr) if wr else max(r[2] for r in rows)) * 1.1
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
        view_runs = back_runs if s in rest else front
        for y, cy, hw in rows:
            X = cxF + cy
            run = next((r for r in view_runs(int(y)) if r[0] <= X < r[1]), None)
            if run is None:
                continue
            lo, hi = max(X - hw, run[0] + 1), min(X + hw, run[1] - 1)
            if hi - lo < 2:
                continue
            clipped.append((y, (lo + hi) / 2 - cxF, (hi - lo) / 2))
        rows = clipped

        # Fra siden står hånda der arket viser den: stikker sidebildet ut
        # foran kroppen ved hånda (gutten), flyttes armen fram dit, jevnt fra
        # albuen.
        def arm_mid(y, hw):
            mid = body_mid(y)
            if y < elbow[s] or s in rest:
                # (en hengende arm står ikke fram der sidearket viser krukka)
                return mid
            b = interp(parts["body"], y)
            run = central(side(int(y)), sx0 + mid)
            if not b or not run:
                return mid
            front_edge = (run[1] - sx0) if facing > 0 else -(run[0] - sx0)
            body_front = (b[2] + b[3]) * facing if facing > 0 else -(b[2] - b[3])
            excess = front_edge - body_front
            if excess < HAND_OUT * H:
                return mid
            t = min(1.0, (y - elbow[s]) / max(1, wrist[s] - elbow[s]))
            want = (front_edge - hw) * facing
            return mid + (want - mid) * t if (want - mid) * facing > 0 else mid

        # Glattet langs armen (rad for rad ble hånda taggete).
        from scipy.ndimage import gaussian_filter1d, median_filter
        mids = np.array([arm_mid(y, hw) for y, cy, hw in rows], float)
        # Der armen er bar (underarmen og hånda), finnes den på sidearket
        # etter hudfargen på hånda forfra: armen sto ellers et annet sted enn
        # på arket, og bildet av armen ble malt på kjolen bak den (en arm
        # til, sett skrått og fra siden).
        skin_at = skin_side(s, [(y, hw) for y, cy, hw in rows]) if s == near_arm and s not in rest else {}
        if len(skin_at) >= SKIN_ROWS:
            bare_side[s] = [q for q in bare if q[0] in skin_at]
        bare.clear()
        if skin_at:
            tool_hand[s] = sorted(skin_at.items())[-1]
        if len(skin_at) >= SKIN_ROWS:
            ys_ = np.array([r[0] for r in rows], float)
            known = sorted(skin_at.items())
            ky = np.array([k for k, _ in known], float)
            kx = np.array([v for _, v in known], float)
            first, last = ky[0], ky[-1]
            m0 = body_mid(first) if first > elbow[s] else kx[0]
            for i, y in enumerate(ys_):
                if y >= first:
                    mids[i] = float(np.interp(y, ky, kx))
                elif y > elbow[s]:
                    t = (y - elbow[s]) / max(1, first - elbow[s])
                    mids[i] = body_mid(y) + (kx[0] - m0) * t
        if len(mids) > 4:
            mids = gaussian_filter1d(median_filter(mids, size=9, mode="nearest"), 2.0, mode="nearest")
        # Hånda er tynnere enn den er bred og rund i tuppen (en rund stabel
        # med flat bunn ble en tallerken i hånda på skriveren).
        tip = HAND_TIP * H
        end_y = rows[-1][0] if rows else 0
        # Hånda begynner der armen smalner brått (et vidt erme slutter), ellers
        # ved håndleddet (håndleddet inne i ermet gjorde enden av ermet til en
        # flat, hudfarget tallerken på skriveren).
        start = wrist[s]
        for i in range(1, len(rows)):
            y, _, hw = rows[i]
            if y < end_y - 0.12 * H:
                continue
            above = [r[2] for r in rows[max(0, i - 8): i]]
            if above and hw < SLEEVE * max(above):
                start = max(start, y)
                break

        wrist_w = min((r[2] for r in rows if abs(r[0] - start) <= 0.02 * H), default=1e9)

        def hand(y, hw):
            if y <= start:
                return hw, hw
            t = max(0.0, min(1.0, (y - (end_y - tip)) / tip))
            r = float(np.sqrt(max(0.15, 1.0 - t * t)))
            # En hånd som holder noe (krusset til vertshusverten) er bredere
            # enn håndleddet; den gjøres ikke flat.
            # (men en hånd med et verktøy er en neve, ikke en plate)
            depth = HAND_DEPTH if hw <= 1.3 * wrist_w or s in tools else 1.0
            return hw * r, hw * depth * r

        arm_rows = [(y, cy, *(lambda a, b: (a, float(mx), b))(*hand(y, hw))) for (y, cy, hw), mx in zip(rows, mids)]
        # Fingertupper som peker ut til siden (smedens åpne hånd) er ikke en
        # hale ut av hånda: under håndleddet står hånda høyst så langt fra
        # midten av håndleddet som den er bred, og de tynne radene nederst
        # tas bort.
        at_wrist = [r for r in arm_rows if abs(r[0] - wrist[s]) < 0.02 * H]
        if at_wrist:
            wc = float(np.median([r[1] for r in at_wrist]))
            ww = max(r[2] for r in arm_rows if r[0] >= wrist[s] - 0.02 * H)
            arm_rows = [(y, min(max(cy, wc - ww), wc + ww), *r) if y > wrist[s] else (y, cy, *r) for y, cy, *r in arm_rows]
            while len(arm_rows) > 3 and arm_rows[-1][0] > wrist[s] and arm_rows[-1][2] < FINGER * ww:
                arm_rows.pop()
        parts[f"arm.{s}"] = arm_rows
    # Beina: fra like over skrittet til sålen; tærne fra sida.
    for s in ("R", "L"):
        rows = []
        robe = set()
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
            # Er biten fra siden mye bredere enn beinet forfra, er det kappa
            # eller skjørtet rundt beinet; den styrer ikke midten (apotekeren
            # fikk en kloss ut av kappa).
            wide = (sc[1] - sc[0]) > LEG_WIDE * (r[1] - r[0]) and y < ankle
            if wide:
                robe.add(y)
            rows.append((y, (r[0] + r[1]) / 2 - cxF, (r[1] - r[0]) / 2, None if wide else mid - sx0, sd))
        # Over der beina skilles (under forkleet eller kjortelen) har låret
        # samme bredde som der det først synes, ikke halve kroppen.
        split = hem + 2 if skirt else crotch
        if skirt and rows:
            # Rett under kanten av kappa ser arket den bakre kanten av kappa
            # mellom beina (kanten er en ellipse sett litt ovenfra): der
            # kappa synes fra siden, er beinet like smalt som leggen lenger
            # ned (apotekeren fikk en kloss under kappa; kjøpmannens vide
            # bukser er beinet).
            mid_y = (hem + ankle) / 2
            shin = [r for r in rows if mid_y <= r[0] < ankle]
            if shin:
                cx_ = float(np.median([r[1] for r in shin]))
                hw_ = float(np.median([r[2] for r in shin]))
                sd_ = float(np.median([r[4] for r in shin]))
                rows = [
                    (y, cx_, hw_, sm, min(max(sd, SHIN_LO * sd_), SHIN_HI * sd_)) if y in robe and hw > SHIN_HI * hw_ else
                    (y, cx, hw, sm, min(max(sd, SHIN_LO * sd_), SHIN_HI * sd_)) if hem <= y < mid_y else (y, cx, hw, sm, sd)
                    for y, cx, hw, sm, sd in rows
                ]
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
        seen_idx = [i for i in idx if rows[i][0] >= (hem if skirt else crotch) and rows[i][3] is not None]
        if len(seen_idx) < 4:
            # For få rader der bare beinet synes: rett ned fra ankelen.
            base = next((r[3] for r in rows if r[0] >= ankle and r[3] is not None), 0.0)
            for i in idx:
                rows[i] = (*rows[i][:3], base, rows[i][4])
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
            top_seen = ys0.min()
            for i in idx:
                # Over der beinet synes (under et skjørt) står det rett: linja
                # gjennom noen få rader vippet ellers framover (apotekeren fikk
                # en kloss ut av kappa).
                y = max(rows[i][0], top_seen) - k
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
        # Kanten av beinet forfra hopper ikke: det som henger inntil låret
        # (øksehodet til Gorvek) ble ellers en del av beinet. En rad der
        # kanten står mer enn `LEG_JUMP` fra medianen av naboene, legges på
        # medianen.
        if len(rows) > 6:
            lo_e = np.array([r[1] - r[2] for r in rows], float)
            hi_e = np.array([r[1] + r[2] for r in rows], float)
            from scipy.ndimage import median_filter
            lo_m = median_filter(lo_e, size=LEG_WIN, mode="nearest")
            hi_m = median_filter(hi_e, size=LEG_WIN, mode="nearest")
            lo_e = np.where(np.abs(lo_e - lo_m) > LEG_JUMP * H, lo_m, lo_e)
            hi_e = np.where(np.abs(hi_e - hi_m) > LEG_JUMP * H, hi_m, hi_e)
            rows = [(r[0], (a + b) / 2, (b - a) / 2, *r[3:]) for r, a, b in zip(rows, lo_e, hi_e)]
            # Og fra siden (øksehodet foran låret).
            if all(r[3] is not None for r in rows):
                lo_e = np.array([r[3] - r[4] for r in rows], float)
                hi_e = np.array([r[3] + r[4] for r in rows], float)
                lo_m = median_filter(lo_e, size=LEG_WIN, mode="nearest")
                hi_m = median_filter(hi_e, size=LEG_WIN, mode="nearest")
                lo_e = np.where(np.abs(lo_e - lo_m) > LEG_JUMP * H, lo_m, lo_e)
                hi_e = np.where(np.abs(hi_e - hi_m) > LEG_JUMP * H, hi_m, hi_e)
                rows = [(*r[:3], (a + b) / 2, (b - a) / 2) for r, a, b in zip(rows, lo_e, hi_e)]
        parts[f"leg.{s}"] = rows
    parts.update(held(parts, front, cxF, H, shoulder, min(hem, ground - int(HELD_LOW * H))))
    parts.update(carried(parts, side, sx0, facing, H, shoulder, min(hem, ground - int(0.2 * H))))
    # Det en hånd med et verktøy holder (smedens hammer) blir en egen
    # gjenstand, ikke en del av figuren (den ble en skål i hånda).
    tool_side, tool_front = [], []  # verktøyet fra siden og forfra: (y, fra, til) mot midten i den visningen
    for s in tuple(tools) + tuple(rest):
        for k in [k for k in parts if k.startswith(f"held.{s}")]:
            if s in tools:
                tool_side += [(r[0], r[3] - r[4], r[3] + r[4]) for r in parts[k]]
                tool_front += [(r[0], r[1] - r[2], r[1] + r[2]) for r in parts[k]]
            parts.pop(k)
    # Verktøyet forfra: det i omrisset under og rundt hånda med verktøyet
    # som verken kroppen, armene eller beina dekker (øksa henger ned langs låret,
    # og motoren fant den ikke som noe hånda bærer).
    for s in tools:
        arm = parts.get(f"arm.{s}")
        if not arm:
            continue
        hx = cxF + arm[-1][1]
        for y in range(int(hand_end[s] - TOOL_ABOVE * H), int(min(ground - 0.05 * H, hand_end[s] + TOOL_DOWN * H)), STEP):
            cover = []
            for k in [k for k in parts if k == "body" or k.startswith(("leg.", "arm."))]:
                r = interp(parts[k], y)
                if r:
                    cover.append((cxF + r[0] - r[1], cxF + r[0] + r[1]))
            for a, b in front(y):
                segs = [(a, b)]
                for c0, c1 in cover:
                    segs = [q for p0, p1 in segs for q in ((p0, min(p1, c0)), (max(p0, c1), p1)) if q[1] - q[0] > 0]
                for p0, p1 in segs:
                    if abs((p0 + p1) / 2 - hx) < TOOL_REACH * H and p1 - p0 >= 3:
                        tool_front.append((y, p0 - cxF, p1 - cxF))
    # Verktøyet i hånda nærmest fra siden (smedens hammer) ligger over
    # forkleet, så omrisset viser det ikke: en sirkel rundt hånda, fra litt
    # over den og ned, males bort fra sidearket (`townsfolk.armless_side`);
    # gjenstanden kommer der.
    for s in tools:
        arm = parts.get(f"arm.{s}")
        if s != near_arm or not arm:
            continue
        # Der arket viser hånda fra siden (huden nederst på armen).
        hy, hx = tool_hand.get(s, (arm[-1][0], arm[-1][3]))
        # Verktøyet henger fra hånda: fra like under neven (den tar arket,
        # og ble ellers uskarp) og `TOOL_BELOW` ned, `TOOL_NEAR` til hver side.
        # Så langt ned som verktøyet går forfra (øksa henger til kneet).
        low = max([r[0] for r in tool_front if r[2] - r[1] >= TOOL_W * H] + [hy + TOOL_BELOW * H])
        for y in range(int(hy + TOOL_GAP * H), int(low), STEP):
            tool_side.append((y, hx - TOOL_NEAR * H, hx + TOOL_NEAR * H))
    m = dict(
        name=name, height=height, px=px, top=top, ground=ground, H=H, skirt=skirt, facing=facing,
        views={"front": cxF, "back": cxB, "side": sx0, "ground": ground, "side_dir": facing},
        size=fg.shape[::-1],
        y=dict(shoulder=shoulder, crotch=crotch, hem=hem, hip=hip, knee=knee, ankle=ankle, **{f"elbow.{s}": elbow[s] for s in elbow}, **{f"wrist.{s}": wrist[s] for s in wrist}, **{f"hand_end.{s}": hand_end[s] for s in hand_end}),
        parts=parts,
        bare_side=bare_side.get(near_arm, []),
        rest=list(rest),
        tool_side=tool_side,
        tool_front=tool_front,
        skin=[float(v) for v in np.mean(skins, 0)] if skins else None,
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
        runs = chains(rows, 1, 2)
        runs = [r for r in runs if r[-1][0] - r[0][0] >= THING_TALL * H]
        # Tingen henger i hånda: en rad i den ligger inntil armen (ellers
        # svevde krukka ved siden av vertshusverten).
        arm = parts.get(f"arm.{s}", [])
        runs = [r for r in runs if any(touches(q, arm, TOUCH * H) for q in r) or hangs(r, arm, TOUCH * H)]
        if runs:
            best = max(runs, key=lambda r: sum(q[2] for q in r))
            out[f"held.{s}"] = best
    return out


def skin_like(px, skin):
    """Pikslene (rgb, siste akse) som har hudfargen `skin`: samme fargetone
    og metning (rødt minus blått mot lysstyrken; forkleet har nesten samme
    tone, men er blekere), lysere eller mørkere."""
    px = np.asarray(px, float)
    lum = px.sum(-1) + 1e-6
    s_lum = float(np.sum(skin))
    tone = np.abs(px[..., :2] / lum[..., None] - np.asarray(skin[:2]) / s_lum).sum(-1)
    sat = (px[..., 0] - px[..., 2]) / lum
    s_sat = (skin[0] - skin[2]) / s_lum
    return (tone < SKIN_TONE) & (np.abs(sat - s_sat) < SKIN_SAT) & (lum > SKIN_DARK * s_lum) & (lum < SKIN_LIGHT * s_lum)


def chains(rows, c, w):
    """Radene (sortert etter y) samlet i sammenhengende biter: en rad hører
    til biten der raden over ligger inntil den (midten `c`, halv bredde
    `w`). Flere biter i samme rad følges hver for seg (bøtta og en flik av
    skjørtet ved siden av ble ellers kappet opp i småbiter)."""
    runs = []
    for r in rows:
        near = [run for run in runs if 0 < r[0] - run[-1][0] <= STEP and abs(r[c] - run[-1][c]) <= r[w] + run[-1][w]]
        if near:
            min(near, key=lambda run: abs(r[c] - run[-1][c])).append(r)
        else:
            runs.append([r])
    return runs


def hangs(run, arm, slack):
    """Tingen `run` henger under hånda: den begynner like under der armen
    slutter, rett under hånda (vaskekonas bøtte)."""
    if not arm:
        return False
    y, x, w = arm[-1][:3]
    q = run[0]
    return 0 <= q[0] - y <= 2 * slack and abs(q[1] - x) <= q[2] + w + slack


def side_touches(q, arm, slack):
    """Raden `q` (y, …, midte fra siden, halv dybde) ligger inntil armen
    fra siden."""
    a = interp(arm, q[0])
    return a is not None and abs(q[3] - a[2]) <= q[4] + a[3] + slack


def touches(q, arm, slack=6):
    """Raden `q` i en ting (y, midte, halv bredde, …) ligger inntil armen."""
    a = interp(arm, q[0])
    return a is not None and abs(q[1] - a[0]) <= q[2] + a[1] + slack


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
    hands = {k[-1]: (r[-1][0] - int(0.16 * H), r[-1][0] + int(0.03 * H), r[-1][1], max(q[2] for q in r[-12:])) for k, r in parts.items() if k.startswith("arm.") and r}
    found = {"back": [], "front": [], **{s: [] for s in hands}}
    for y in range(y0, y1, STEP):
        body = interp(parts["body"], y)
        if body is None:
            continue
        cover, torso = [], []
        for k in keys:
            r = interp(parts[k], y)
            if r:
                cover.append((sx0 + r[2] - r[3] - 2, sx0 + r[2] + r[3] + 2))
                if k == "body" or k.startswith("leg."):
                    torso.append(cover[-1])
        for a, b in side(y):
            segs = [(a, b)]
            for c0, c1 in cover:
                segs = [q for p0, p1 in segs for q in ((p0, min(p1, c0)), (max(p0, c1), p1)) if q[1] - q[0] > 0]
            for p0, p1 in segs:
                # Det figuren bærer, ligger inntil den (en løs bit av øksa
                # svevde ved siden av Gorvek).
                c = (p0 + p1) / 2
                at_hand_ = any(h0 <= y <= h1 for h0, h1, _, _ in hands.values())
                if not at_hand_ and not any(abs(p0 - c1) <= 4 or abs(p1 - c0) <= 4 for c0, c1 in cover):
                    continue
                where = "front" if (c - sx0 - body[2]) * facing > 0 else "back"
                # Bak på kroppen (fletta, sekken) holder ingen et skaft, og
                # ved hånda er det hånda selv (gutten) eller det den holder.
                at_hand_ = any(h0 <= y <= h1 for h0, h1, _, _ in hands.values())
                if p1 - p0 < (THING_BACK if where == "back" or at_hand_ else THING) * H:
                    continue
                d = (p1 - p0) / 2
                near = [s for s, (h0, h1, hx, _) in hands.items() if h0 <= y <= h1]
                if where == "front" and near:
                    s = min(near, key=lambda s: abs(hands[s][2]))
                    # Høyst så bred som hånda forfra (70 % av kroppen ble
                    # tallerkener i hendene på skriveren).
                    arm_r = interp(parts.get(f"arm.{s}", []), y)
                    wmax = (arm_r[1] if arm_r else hands[s][3]) * 1.2
                    found[s].append((y, arm_r[0] if arm_r else hands[s][2], min(wmax, d * 1.2), c - sx0, d))
                    continue
                if where == "front":
                    # Foran kroppen og over hendene er det underarmen som
                    # holder noe (vakta): en skive på brystet ble en hylle.
                    continue
                if not any(abs(p0 - c1) <= TOUCH * H or abs(p1 - c0) <= TOUCH * H for c0, c1 in torso):
                    # Bak på kroppen ligger tingen inntil kroppen, ikke bare
                    # inntil hånda (øksehodet bak Gorveks legg svevde).
                    continue
                found[where].append((y, body[0], min(body[1] * CARRIED, d * 1.2), c - sx0, d))
    out = {}
    for w, rows in found.items():
        runs = chains(rows, 3, 4)
        runs = [r for r in runs if r[-1][0] - r[0][0] >= THING_TALL * H]
        if f"held.{w}" in parts:
            continue
        if w in ("R", "L"):
            # Ved hånda: tingen ligger inntil hånda forfra og fra siden
            # (spydet foran vakta ble ellers en planke som svevde foran ham).
            arm = parts.get(f"arm.{w}", [])
            runs = [r for r in runs if any(touches(q, arm, TOUCH * H) and side_touches(q, arm, TOUCH * H) for q in r)]
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
    # Raden leses helt ned til halsen på kroppsarket og litt til: et skjegg
    # som henger under (smeden) hører til hodet, og haka avgjør under.
    for Y in range(top, fit["neck"] + STEP + int(BEARD_BELOW * m["H"]) + 1, max(2, STEP // 2)):
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
    # Haka: nedover fra nesa er haka (eller enden av skjegget) siste rad
    # før profilen trekker seg kraftig inn mot halsen (det gamle snittet,
    # der profilen trakk seg mest inn, fant munnen under nesa og skjegget).
    F_raw = np.array([r[3] for r in rows], float)
    Y_raw = np.array([r[0] for r in rows], float)
    face = [i for i in range(len(rows)) if Y_raw[i] >= m["top"] + CHIN * m["H"]]
    # Uten noe fall under haka slutter hodet ved halsen som før.
    chin = min(out[-1][0], fit["neck"] + STEP)
    if face:
        nose = max(face[: max(1, len(face) * 2 // 3)], key=lambda i: F_raw[i])
        step = max(1.0, float(np.median(np.diff(Y_raw))))
        # Fallet under haka er brått (fiskerkona: 55 px over 9 px), mens
        # leppene under nesa bare trekker seg litt inn.
        w = max(1, int(0.012 * m["H"] / step))
        start = nose + max(1, int(0.02 * m["H"] / step))
        ahead = max(1, int(LIPS * m["H"] / step))
        k = start + w
        while k < len(rows):
            if F_raw[k - w] - F_raw[k] > RECEDE * m["H"]:
                # Kommer profilen fram igjen like under, var det munnen under
                # barten og ikke haka (smedens skjegg ble kuttet der).
                back = next((q for q in range(k + 1, min(len(rows), k + ahead)) if F_raw[q] >= F_raw[k - w] - 0.3 * RECEDE * m["H"]), None)
                if back is not None:
                    k = back + w
                    continue
                # Den siste raden før fallet begynner.
                j = max(q for q in range(k - w, k + 1) if F_raw[k - w] - F_raw[q] < 0.3 * RECEDE * m["H"])
                chin = float(Y_raw[j])
                break
            k += 1
    # Et skjegg som henger under halsen (smeden) smalner av nedover, mens
    # skuldrene på hodearket blir bredere: hodet følger skjegget så langt
    # bredden ikke vokser.
    neck_end = fit["neck"] + STEP
    at = [r for r in rows if r[0] <= neck_end]
    if chin >= neck_end - STEP and at:
        below = [r for r in rows if r[0] > neck_end]
        if below and max(r[2] for r in below) <= at[-1][2] * BEARD_WIDEN:
            chin = below[-1][0]
            # Under halsen holder bakkanten seg der den var ved halsen: hodet
            # dekker nakken ned til skuldrene, og skjegget henger foran (med
            # bakkanten trukket fram ble halsen bak en kloss med flatt lokk).
            i0 = max(i for i, r in enumerate(out) if r[0] <= neck_end)
            back0 = out[i0][4]
            for i in range(i0 + 1, len(out)):
                y, cx, hw, fe, be = out[i]
                out[i] = (y, cx, hw, fe, min(be, back0))
    # Hodet slutter like under haka: det som henger ned ved halsen (det
    # lange håret til fiskerkona, fletta) ble et bredt rør med huden fra
    # profilen på sidene. Halsen og håret under kommer fra kroppsarket, og
    # skjøten ligger under kjeven.
    # Et skjegg kan henge under halsen fra passingen (smeden): da gjelder
    # haka.
    out = [r for r in out if r[0] <= max(min(fit["neck"] + STEP, chin + HEAD_BELOW * m["H"]), chin)]
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
    out = framed_face(out, chin, m, fit, fr, sF, tyF, cF)
    out = rounded_head(out, chin, m)
    # Tverrsnittet er en ellipse foran og en bak en fast akse midt i
    # skallen, så et skaut eller en flette bak ikke flytter ansiktet.
    axis = float(np.median([(r[3] + r[4]) / 2 for r in out]))
    out = [(r[0], r[1], r[2], axis * m["facing"], max(2.0, r[3] - axis), max(2.0, axis - r[4]), r[5], r[6], "split") for r in out]
    return out, chin


def framed_face(rows, chin, m, fit, fr, sF, tyF, cF):
    """Hår som henger ved siden av kinnene (fiskerkona) henger bak kinnet:
    fra munnen og ned til haka følger bredden foran på hodet huden forfra
    (den sjette verdien), når haka er bar; bak beholder hodet hele bredden. Huden er fargen midt på kinnet; der den slutter til
    sidene, slutter hodet (litt utenfor). Med skjegg eller et skaut over
    haka står bredden som den er."""
    from PIL import Image

    im = np.asarray(Image.open(fit["file"]).convert("RGB")).astype(float)
    ys = sorted(fr)
    if not ys:
        return rows
    y0, y1 = ys[0], ys[-1]
    c = int(round(cF))

    def hy_of(Y):
        return int(round((Y - tyF) / sF))

    # Kinnet: et stykke ut fra midten, midt i ansiktet (under øynene).
    hc = hy_of(chin)
    hn = int(y0 + 0.62 * (hc - y0))
    half = max(4, int(0.18 * (max(fr[hn], key=lambda r: r[1] - r[0])[1] - max(fr[hn], key=lambda r: r[1] - r[0])[0]) / 2)) if hn in fr else 8
    patch = np.concatenate([im[hn - 6: hn + 6, c - 3 * half: c - 2 * half].reshape(-1, 3), im[hn - 6: hn + 6, c + 2 * half: c + 3 * half].reshape(-1, 3)])
    skin = np.median(patch, 0)

    def is_skin(px):
        return np.abs(px - skin).sum() < SKIN_TOL

    # Bar hake: midt under munnen er det hud, jevn og nær kinnet (et
    # skjegg i samme farge er stripete).
    hm_ = int(hc - 0.06 * (hc - y0))
    if not (0 <= hm_ < im.shape[0]):
        return rows
    chin_px = im[hm_ - 4: hm_ + 4, c - 6: c + 6].reshape(-1, 3)
    if np.abs(np.median(chin_px, 0) - skin).sum() > SKIN_TOL * 0.6 or chin_px.std(0).mean() > SKIN_STRIPES:
        return [(*r, r[2]) for r in rows]
    out = []
    # Fra munnen og ned (ved øynene bryter skygger og rynker huden opp).
    hmouth = int(y0 + FRAMED * (hc - y0))
    for r in rows:
        Y = r[0]
        hy = hy_of(Y)
        if not (hmouth <= hy <= hc) or hy not in fr:
            out.append((*r, r[2]))
            continue
        row = im[hy]
        a = b = c
        gap = 0
        while a > 0 and gap < SKIN_GAP:
            a -= 1
            gap = 0 if is_skin(row[a]) else gap + 1
        while b < im.shape[1] - 1 and gap < SKIN_GAP:
            b += 1
            gap = 0 if is_skin(row[b]) else gap + 1
        span = (min(c - a, b - c) - SKIN_GAP) * sF * 1.06
        # Bare forkanten: håret henger bak kinnet (profilen viser det). Bare
        # når håret tar en god del av bredden: skygger i ansiktet bryter
        # også huden opp (skriveren fikk klemt hake).
        front_w = r[2] if span > (1 - FRAMED_MIN) * r[2] else max(span, 0.5 * r[2])
        out.append((r[0], r[1], r[2], r[3], r[4], front_w))
    return out


def rounded_head(rows, chin, m):
    """Et hode er glatt og rundt (eieren 14. oktober: «hodet er
    deformert»; smeden hadde en spiss kjegle på toppen og en søyle for
    hode, gutten en flat isse og klumper av hårtuster). Radene fra
    omrisset gjøres til et hode:

    - bredden og bakkanten glattes (`HEAD_SMOOTH`), så hver hårtust ikke
      blir form (håret er malt på), og ørene er ikke bredde (`EAR`);
    - issen er minst en kuppel (`CROWN` av høyden ned, avrundet over den
      øverste raden), ikke flat og ikke spiss;
    - fra kinnbeina og ned smalner hodet bare (ingen hengekinn);
    - forkanten deles i ansiktsplanet og det som stikker fram av det
      (nesa, leppene, haka): planet er forkanten uten de smale fremspringene
      (`FACE_RELIEF` av høyden), og fremspringet legges som en smal bule
      midt i ansiktet (`ring_mesh`), ikke over hele bredden.

    Gir (y, midte, halv bredde, forkant, bakkant, fremspring)."""
    from scipy.ndimage import gaussian_filter1d, grey_opening, median_filter

    ys = np.array([r[0] for r in rows], float)
    cy = np.array([r[1] for r in rows], float)
    W = np.array([r[2] for r in rows], float)
    Wf = np.array([r[5] for r in rows], float)
    F = np.array([r[3] for r in rows], float)
    B = np.array([r[4] for r in rows], float)
    dy = float(np.median(np.diff(ys))) if len(ys) > 1 else 3.0
    sig = HEAD_SMOOTH * m["H"] / dy
    # Ørene stikker ut på sidene over et kort stykke; de er ikke hodets
    # bredde (guttens kinn ble runde og brede av dem). Bremmer er høyere.
    W0 = W.copy()
    W = grey_opening(W, size=max(3, int(EAR * m["H"] / dy)), mode="nearest")
    # Ørene sitter midt på hodet og kan være lange (skriveren); der tas også
    # lengre utstikk bort. Bremmene sitter høyere opp.
    low = ys > ys[0] + EAR_FROM * max(1.0, chin - ys[0])
    W = np.where(low, np.minimum(W, grey_opening(W0, size=max(3, int(EAR_LONG * m["H"] / dy)), mode="nearest")), W)
    # Under en brem (det bredeste ligger i den øvre delen av hodet) stikker
    # håret ut under hatten; det er heller ikke hodets bredde (bonden fikk
    # vinger ved tinningene).
    hh0 = max(1.0, chin - ys[0])
    i_max = int(np.argmax(W0[: max(1, int(np.searchsorted(ys, chin)))]))
    if ys[i_max] < ys[0] + BRIM_HIGH * hh0:
        big = grey_opening(W0, size=max(3, int(UNDER_BRIM * m["H"] / dy)), mode="nearest")
        below = np.arange(len(W)) > i_max + int(BRIM * m["H"] / dy)
        W = np.where(below, np.minimum(W, big), W)
    # Median først: den holder kanten på en brem skarp (glattingen alene
    # smurte bremmen nedover, og bonden fikk vinger under hatten).
    k = 2 * int(sig) + 1
    W = gaussian_filter1d(median_filter(W, size=k, mode="nearest"), sig / 3, mode="nearest")
    B = gaussian_filter1d(median_filter(B, size=k, mode="nearest"), sig / 3, mode="nearest")
    Fs = gaussian_filter1d(F, 1.0, mode="nearest")
    win = max(3, int(FACE_RELIEF * m["H"] / dy))
    plane = gaussian_filter1d(grey_opening(Fs, size=win, mode="nearest"), sig, mode="nearest")
    relief = np.maximum(0.0, Fs - plane)
    F = plane
    # Issen.
    hh = max(1.0, chin - ys[0])
    i_chin = int(np.argmin(np.abs(ys - chin)))
    y_ref = ys[0] + CROWN * hh
    # Nesa, leppene og haka sitter i ansiktet: over pannen er et fremspring
    # en hårtust, og som bule ble den en rygg opp over issen og en spiss
    # topp (smeden, eieren 14. oktober «hodet deformert»).
    relief *= np.clip((ys - y_ref) / max(dy, FACE_FROM * hh), 0.0, 1.0)
    i_ref = int(np.argmin(np.abs(ys - y_ref)))
    w_ref = float(min(W[i_ref], np.median(W[i_ref: max(i_ref + 1, i_chin)])))
    c0 = (F[i_ref] + B[i_ref]) / 2
    f_ref, b_ref = F[i_ref] - c0, c0 - B[i_ref]
    # Kuppelen når akkurat opp til toppen av omrisset og er rund helt opp
    # (over omrisset ble toppen et flatt lokk).
    rise = max(dy, y_ref - ys[0])
    top = y_ref - rise

    def dome(y):
        t = (y_ref - y) / rise
        return float(np.sqrt(max(0.0, 1.0 - t * t)))

    for i in range(i_ref):
        d = dome(ys[i])
        W[i] = max(W[i], w_ref * d)
        # Fra siden er issen også en kuppel: en hårtust foran som går rett
        # opp, ga et loddrett ansikt til toppen og en skarp kant (smeden,
        # eieren 14. oktober «hodet deformert fra siden»).
        F[i] = c0 + min(F[i] - c0, f_ref * d * DOME_OVER) if F[i] - c0 > f_ref * d else c0 + f_ref * d
        B[i] = c0 - min(c0 - B[i], b_ref * d * DOME_OVER) if c0 - B[i] > b_ref * d else c0 - b_ref * d
    # Kuppelen lukker seg nesten i et punkt (den første raden var en
    # fjerdedel av bredden, og toppen ble en flat skive).
    cap_ys = [top + rise * f for f in (0.002, 0.02, 0.06)] + list(np.arange(top + 0.12 * rise, ys[0] - 0.5, dy))
    cap = [(y, cy[0], max(1.0, w_ref * dome(y)), c0 + max(1.0, f_ref * dome(y)), c0 - max(1.0, b_ref * dome(y)), 0.0)
           for y in cap_ys if y < ys[0] - 0.5]
    # Kjeven: fra kinnbeina og ned blir hodet bare smalere.
    i_cheek = i_ref + int(np.argmax(W[i_ref: max(i_ref + 1, i_chin)]))
    for i in range(i_cheek + 1, len(W)):
        W[i] = min(W[i], W[i - 1])
    # Under haka rundes hodet av (en flat bunn ga gutten og vakta kantete
    # kjever forfra).
    under = HEAD_BELOW * m["H"]
    for i in range(len(W)):
        if ys[i] > chin:
            t = min(1.0, (ys[i] - chin) / under)
            W[i] *= float(np.sqrt(max(0.35, 1.0 - t * t)))
    Wf = np.minimum(gaussian_filter1d(Wf, sig, mode="nearest"), W)
    body = [(float(ys[i]), float(cy[i]), float(W[i]), float(F[i]), float(B[i]), float(relief[i]), float(Wf[i])) for i in range(len(ys))]
    return [(float(c[0]), float(c[1]), float(c[2]), float(c[3]), float(c[4]), 0.0, float(c[2])) for c in cap] + body
