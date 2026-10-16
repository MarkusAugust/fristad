"""Åpningsfilmen som data: skriver `web/film/opening.json` etter
`docs/film/manus.md` (kanon: `docs/kanon.md`).

Verden er et bånd langs x (tiden går mot høyre), med lagene i dybden langs
−z. Kameraet ser mot −z med synsfeltet `FOV`, så et 16:9-bilde `d` foran
kameraet fyller bildet når det er `fill(d)` høyt. Hver del har sitt sted
langs x (og hovedstaden høyt oppe, krypten under bakken), og kameraet flyr
fra del til del uten kutt; der det hopper, er det bak et slør.

  python3 tools/film/intro.py
"""

import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FOV = 40
TAN = math.tan(math.radians(FOV / 2))
ASPECT = 2048 / 1143  # eierens bilder (2752 × 1536)


def fill(d, k=1.0):
    """Høyden som fyller bildet `d` foran kameraet (gange `k`)."""
    return round(2 * TAN * d * k, 3)


def ground(cam_y, d, k=0.95):
    """Høyden på bunnen av bildet `d` foran kameraet (litt over, `k`)."""
    return round(cam_y - TAN * d * k, 3)


LAYERS = []
TEXTS = []


def layer(src, x, y, z, h, **o):
    LAYERS.append({"src": src, "at": [round(x, 3), round(y, 3), round(z, 3)], "h": h, **o})


def strip(src, x0, x1, y, z, h, **o):
    """Et bilde som gjentas langs x, annethvert speilvendt, så skjøtene møtes."""
    w = h * ASPECT
    n = math.ceil((x1 - x0) / w) + 1
    for i in range(n):
        # Litt omlegg, så skjøten aldri blir en strek.
        layer(src, x0 + i * w * 0.985, y, z, h, flip=bool(i % 2), **o)


def text(t, at, lines, h, width, align="left", pace=0.12):
    TEXTS.append({"t": t, "at": [round(v, 3) for v in at], "lines": lines, "h": round(h, 3), "width": round(width, 3), "align": align, "pace": pace})


SAYS = []


def say(t, _cam, d, lines, dx=-0.42, dy=0.3, size=0.085, width=0.62, align="left"):
    """En tekst `d` foran kameraet midt i tiden `t`, med venstre kant `dx` og
    første linje `dy` ut i bildet (brøk av halve bredden og høyden), og
    overskriftshøyden `size` av bildehøyden. Stedet regnes ut fra kamerabanen
    når den er ferdig (`place_texts`), og teksten vender mot kameraet der."""
    SAYS.append((t, d, lines, dx, dy, size, width, align))


def hermite(keys, t, get):
    """Samme bane som filmen (Catmull–Rom med tiden som parameter)."""
    i = 0
    while i < len(keys) - 2 and t > keys[i + 1]["t"]:
        i += 1
    a, b = keys[i], keys[min(i + 1, len(keys) - 1)]
    if a is b or t <= a["t"]:
        return list(get(a))
    if t >= b["t"]:
        return list(get(b))
    h = b["t"] - a["t"]
    s = (t - a["t"]) / h

    def tangent(k):
        if k == 0 or k == len(keys) - 1:
            return [0, 0, 0]
        p, n = keys[k - 1], keys[k + 1]
        return [(get(n)[j] - get(p)[j]) / (n["t"] - p["t"]) for j in range(3)]

    m0, m1 = tangent(i), tangent(i + 1)
    s2, s3 = s * s, s * s * s
    return [get(a)[j] * (2 * s3 - 3 * s2 + 1) + m0[j] * h * (s3 - 2 * s2 + s) + get(b)[j] * (-2 * s3 + 3 * s2) + m1[j] * h * (s3 - s2) for j in range(3)]


SCALE = {"heading": 1.0, "quote": 0.78, "latin": 0.62, "text": 0.5}


def basis(keys, t):
    p = hermite(keys, t, lambda k: k["p"])
    look = hermite(keys, t, lambda k: k["look"])
    f = [look[j] - p[j] for j in range(3)]
    n = math.sqrt(sum(v * v for v in f))
    f = [v / n for v in f]
    r = [-f[2], 0, f[0]]
    rn = math.sqrt(sum(v * v for v in r))
    r = [v / rn for v in r]
    u = [r[1] * f[2] - r[2] * f[1], r[2] * f[0] - r[0] * f[2], r[0] * f[1] - r[1] * f[0]]
    return p, f, r, u


def block(lines, h, width):
    """Bredden og høyden på en tekst (verdensenheter), grovt: tegnene er
    rundt halvparten så brede som linja er høy."""
    w = 0
    hh = 0
    for style, line in lines:
        lh = h * SCALE[style]
        chars = len(line) * lh * 0.5
        rows = max(1, math.ceil(chars / width))
        w = max(w, min(chars, width))
        hh += rows * lh * 1.25 + lh * 0.2
    return w, hh


def outside(keys, t, at, w, hh, align):
    """Hvor langt teksten stikker ut av bildet (brøk, 0 er helt inne) i løpet
    av tiden den står, med en marg."""
    worst = 0
    x0 = {"left": 0, "center": -w / 2, "right": -w}[align]
    steps = 12
    for i in range(steps + 1):
        tt = t[0] + (t[1] - t[0]) * i / steps
        p, f, r, u = basis(keys, tt)
        for cx in (x0, x0 + w):
            for cy in (0.6, -hh):
                q = [at[0] + cx, at[1] + cy * (1 if cy > 0 else 1), at[2]]
                v = [q[j] - p[j] for j in range(3)]
                dz = sum(v[j] * f[j] for j in range(3))
                if dz <= 0.5:
                    return 9
                sx = sum(v[j] * r[j] for j in range(3)) / (dz * TAN * 16 / 9)
                sy = sum(v[j] * u[j] for j in range(3)) / (dz * TAN)
                worst = max(worst, abs(sx) - 0.86, sy - 0.86, -sy - 0.78)
    return max(worst, 0)


def place_texts():
    """Setter tekstene der manuset ber om (`say`), sett fra kameraet midt i
    tiden de står, og flytter dem så de holder seg inne i bildet hele tiden
    (nærmeste sted som går, ellers lenger unna)."""
    keys = sorted(CAMERA, key=lambda k: k["t"])
    for t, d, lines, dx, dy, size, width, align in SAYS:
        mid = (t[0] + t[1]) / 2
        best = None
        fallback = None
        for scale in (1.0, 1.4, 2.0, 2.8, 4.0, 6.0, 9.0):
            dd = d * scale
            p, f, r, u = basis(keys, mid)
            vh = 2 * TAN * dd
            vw = vh * 16 / 9
            h = size * vh
            wv = width * vw
            bw, bh = block(lines, h, wv)
            cands = []
            for ox in [i * 0.08 for i in range(-12, 13)]:
                for oy in [i * 0.08 for i in range(-12, 13)]:
                    at = [p[j] + f[j] * dd + r[j] * (dx + ox) * vw / 2 + u[j] * (dy + oy) * vh / 2 for j in range(3)]
                    cands.append((ox * ox + oy * oy, at))
            cands.sort(key=lambda c: c[0])
            for _, at in cands:
                o = outside(keys, t, at, bw, bh, align)
                if o == 0:
                    best = (at, h, wv)
                    break
                if fallback is None or o < fallback[0]:
                    fallback = (o, (at, h, wv))
            if best:
                break
        if not best:
            print(f"teksten stikker {fallback[0]:.2f} ut:", lines[0][1])
            best = fallback[1]
        at, h, wv = best
        text(t, at, lines, h, wv, align)
        TEXTS[-1]["face"] = [round(v, 3) for v in hermite(keys, mid, lambda k: k["p"])]


I = "intro/"
O = "opening/"


def pick(new, old):
    """Det nye bildet når eieren har laget det (`docs/film/prompter-nye-bilder.md`), ellers det gamle."""
    return I + new + ".webp" if (ROOT / "assets/film/intro" / (new + ".webp")).exists() else O + old
CAMERA = []


def cam(t, p, look=None, fov=FOV, roll=0):
    CAMERA.append({"t": t, "p": [round(v, 3) for v in p], "look": [round(v, 3) for v in (look or [p[0], p[1], p[2] - 10])], "fov": fov, "roll": roll})


# --- 1. Riket (0–14): fra himmelen ned over åsene med byene --------------------
SKY_D = 600
layer(I + "himmel_morgen.webp", 40, 60, -SKY_D, fill(SKY_D, 2.2), follow=0.92, camY=6, show=[0, 0.5, 34, 36])
strip(I + "aaser_fjerne.webp", 0, 160, -18, -200, fill(200, 1.05))
strip(I + "aaser_naere.webp", -20, 120, -10, -70, fill(70, 1.1))
strip(I + "forgrunn_gress.webp", -10, 85, ground(4, 6, 1.0), -6, fill(6, 0.7), anchor="bottom", sway=0.04, show=[0, 0, 12.5, 14.5])
cam(0, [-25, 34, 30], [5, 4, -150])
cam(6, [8, 12, 4], [25, 2, -150])
cam(14, [72, 4.5, 0], [92, 3, -80])
say([1.5, 7.2], [10, 11, 4], 30, [["heading", "Fristad"]], dx=-0.2, dy=0.42, size=0.16)
say([7.6, 14], [55, 5, 1], 26, [["text", "Riket var fullt av byer, og hver by hadde sitt skriptorium og sitt eget tungemål."], ["quote", "Men alle delte én mønsterbok."]], dx=-0.62, dy=0.55)

# --- 2. Mønsterboka (14–35): inn gjennom vinduet i Skriptoriet -----------------
X = 100
F_D = 40
fh = fill(F_D, 1.0)
# Midten av vinduet står 0,473 fra venstre og 0,302 fra toppen av det
# beskårne bildet (`cutout.py`, 2048 × 1466); kameraet går gjennom det i høyde 4.
fw = fh * 2048 / 1466
layer(I + "skriptoriet_fasade.webp", X + (0.5 - 0.473) * fw, 4 - (0.5 - 0.302) * fh, -F_D, fh)
# Gjennom vinduet, før kameraet er inne: rommet som et lite bilde rett bak
# vinduet (det rager ikke ut forbi fasaden), som toner over i det store.
layer(O + "gullalderen/bg.webp", X, 4, -40.6, 10, show=[0, 0, 19.6, 20.6])
layer(O + "gullalderen/bg.webp", X, 4, -95, 46, show=[19.6, 20.6, 36, 36.5])
layer(O + "gullalderen/mid.webp", X - 3.5, ground(4, 25, 0.9), -70, fill(25, 0.42), anchor="bottom", breathe=0.006, show=[19.6, 20.6, 36, 36.5])
layer(pick("lesepulten", "gullalderen/fg.webp"), X - 5.5, ground(4, 10, 1.05), -55, fill(10, 0.45), anchor="bottom", lights=[[0.62, 0.12, "#ffcf70", 0.9]], show=[19.6, 20.6, 36, 36.5])
layer(I + "ansgar_bok.webp", X + 6.5, ground(4, 17, 0.98), -62, fill(17, 0.8), anchor="bottom", breathe=0.008, lights=[[0.6, 0.48, "#ffe7a8", 3.0, 0.3]], show=[19.6, 20.6, 36, 36.5])
cam(18, [X, 4, 2], [X, 4, -60])
cam(21, [X, 4.1, -36], [X, 4, -90])
cam(23.5, [X - 1, 4, -45], [X - 2, 3, -90])
cam(29, [X + 3, 3.8, -48], [X + 5, 3, -80])
cam(34.5, [X + 6, 5, -50], [X + 7, 10, -80])
say([23, 27], [X - 1, 4, -45], 20, [["quote", "Den kunne leses på alle tungemål."]], dx=-0.78, dy=0.7, size=0.07, width=0.8, align="left")
say([27.3, 31.3], [X - 1, 4, -45], 14, [["text", "Hver by bygde av den akkurat det den trengte, og boka sto aldri i veien."]], dx=-0.78, dy=0.75, width=0.55)
say([31.5, 35], [X + 3, 3.8, -48], 12, [["text", "I Fristad var det"], ["heading", "mester Ansgar"], ["text", "som holdt den."]], dx=-0.65, dy=0.55, size=0.1, width=0.5)

# --- 3. Rådet (35–48): opp til den kalde salen i hovedstaden -------------------
X, Y = 200, 60
layer(I + "himmel_morgen.webp", 300, 60, -SKY_D, fill(SKY_D, 2.2), follow=0.95, camY=10, show=[36, 37, 73, 73.5])
layer(I + "raadssal.webp", X, Y, -60, fill(66, 1.5), tint="#dfe8ff", show=[34.5, 36, 48.5, 50])
layer(I + "raadet.webp", X + 1, ground(Y, 30, 0.85), -30, fill(30, 0.55), anchor="bottom", breathe=0.004, show=[34.5, 36, 48.5, 50])
layer(I + "kjopmenn.webp", X - 13, ground(Y, 20, 0.95), -20, fill(20, 0.48), anchor="bottom", walk=True, path=[[39, X - 14, ground(Y, 20, 0.95), -20], [48, X - 2, ground(Y, 20, 0.95), -20]], show=[39, 40.5, 49, 50])
layer(I + "komponentboken.webp", X - 4.5, ground(Y, 10, 1.15), -10, fill(10, 0.7), anchor="bottom", mask="ink", show=[42, 44, 49, 50], lights=[[0.5, 0.3, "#9fc4ff", 2.2, 0.2]])
cam(35.6, [X - 4, Y + 1, 3], [X - 2, Y, -60])
cam(47.5, [X + 4, Y, -3], [X + 2, Y - 1, -60])
say([37, 42], [X - 4, Y + 1, 3], 30, [["text", "I hovedstaden satt et råd som aldri hadde holdt en fjærpenn,"], ["text", "og ved siden av dem kjøpmennene fra Kontoret med sine regler."]], dx=-0.78, dy=0.78, width=0.85)
say([42.5, 47.5], [X + 4, Y, -3], 30, [["quote", "De mente at alle byer burde ha det likt,"], ["quote", "og at de visste best hvordan."]], dx=0.0, dy=0.7, width=0.9, align="center", size=0.085)

# --- 4. Budet (48–60): ned til torget, kongens segl over byen ------------------
X, Y = 300, 4
layer(I + "torget_bud.webp", X, Y, -60, fill(62, 1.45), show=[47, 48.5, 60.5, 62])
layer(I + "kongens_segl.webp", X, Y + 15, -50, fill(50, 0.36), mask="ink", show=[49.5, 52, 59.5, 61], lights=[[0.5, 0.5, "#c8e0ff", 9, 0.15]])
layer(I + "herold.webp", X + 7, ground(Y, 28, 0.95), -28, fill(28, 0.6), anchor="bottom", breathe=0.006, show=[47, 48.5, 60.5, 62])
layer(I + "folkemengde_bud.webp", X - 3, ground(Y, 14, 1.12), -14, fill(14, 0.55), anchor="bottom", breathe=0.005, show=[47, 48.5, 60.5, 62])
cam(48.5, [X - 10, Y + 22, 2], [X - 2, Y + 12, -60])
cam(52.5, [X - 2, Y + 3, 0], [X, Y + 3, -60])
cam(59.8, [X + 3, Y + 2.5, -5], [X + 3, Y + 2, -60])
say([52.5, 59.5], [X - 2, Y + 3, 0], 36, [["text", "Så kom kongens bud, på kanselliets latin:"], ["quote", "Omnia lingua Reactia."], ["text", "Alt på hoffets tunge."], ["quote", "Omnia una compage."], ["text", "Alt med ett rammeverk."], ["quote", "Nulla exceptio."], ["text", "Ingen avvik."]], dx=-0.78, dy=0.8, width=0.42, size=0.075)

# --- 5. Inkvisisjonen (60–72) --------------------------------------------------
X = 400
layer(I + "skriptoriet_kveld.webp", X, Y, -60, fill(62, 1.45), show=[59.5, 61, 72.5, 73.5])
layer(I + "baal.webp", X + 11, ground(Y, 34, 0.95), -34, fill(34, 0.32), anchor="bottom", mask="ink", show=[62, 64, 73, 74], lights=[[0.5, 0.45, "#ff9a3c", 7, 1.0], [0.4, 0.3, "#ffb050", 4, 1.0]])
layer(I + "inkvisitorer.webp", X - 9, ground(Y, 24, 0.95), -24, fill(24, 0.5), anchor="bottom", walk=True, path=[[60, X - 14, ground(Y, 24, 0.95), -24], [72, X - 1, ground(Y, 24, 0.95), -24]], show=[59.5, 61, 72.5, 73.5])
layer(I + "komponent_i_ramme.webp", X + 4.5, Y - 0.2, -9, fill(9, 0.42), mask="ink", show=[66.5, 68, 73, 74], lights=[[0.5, 0.45, "#e8ff9a", 1.6, 0.6]])
cam(60.5, [X - 8, Y + 1, 0], [X - 5, Y, -60])
cam(66.5, [X, Y + 0.5, -1], [X + 2, Y, -40])
cam(71.6, [X + 4, Y - 0.1, -3.5], [X + 4.5, Y - 0.2, -9])
say([61, 65.5], [X - 8, Y + 1, 0], 30, [["heading", "Inkvisisjonen"], ["text", "dro fra by til by."]], dx=-0.78, dy=0.78, size=0.11)
say([65.8, 69.2], [X, Y + 0.5, -1], 25, [["text", "Mønsterbøkene ble brent, og hver komponent ble spent fast i rammeverket, med Kontorets regler stemplet på."]], dx=-0.75, dy=0.72, width=0.5)
say([68.6, 71.9], [X + 4, Y - 0.1, -3.5], 4.5, [["quote", "En knapp kunne ikke lenger"], ["quote", "bare være en knapp."]], dx=0.0, dy=-0.4, width=0.6, align="center", size=0.07)

# --- 6. Omveiene (72–85): skriverne om natta -------------------------------------
X = 500
layer(I + "skriptoriet_natt.webp", X, Y, -50, fill(53, 1.45), mask="glitch", show=[72, 73, 85.2, 87])
layer(I + "omveiene.webp", X, ground(Y, 20, 0.95), -20, fill(20, 0.55), anchor="bottom", breathe=0.006, lights=[[0.385, 0.2, "#ffcf70", 0.9], [0.705, 0.17, "#ffcf70", 0.9]], mask="glitch", show=[72, 73, 85, 86.5])
cam(73.2, [X - 5, Y + 0.6, 0], [X - 3, Y, -50])
cam(84.8, [X + 5, Y, -3], [X + 3, Y - 0.5, -50])
say([74, 79], [X - 5, Y + 0.6, 0], 28, [["text", "Skriverne lærte å skrive rundt boka i stedet for med den."]], dx=-0.78, dy=0.78, width=0.6)
say([79.5, 84.7], [X + 5, Y, -3], 28, [["quote", "Omvei på omvei."], ["quote", "Lapp på lapp."], ["quote", "Natt etter natt."]], dx=0.1, dy=0.78, width=0.35, size=0.07)

# --- 7. Bruddet (85–100): alt brekker i samme rom, rammeverket reiser seg ------
X = 500
layer(O + "fallet/bg.webp", X, Y, -50.5, fill(53, 1.45), mask="glitch", show=[84.6, 86.5, 92, 93])
layer(O + "fallet/mid.webp", X + 1, Y + 1, -22, fill(22, 0.45), mask="glitch", show=[86.5, 88, 92, 93], sway=0.15)
layer(O + "fallet/fg.webp", X - 4, ground(Y, 9, 1.05), -9, fill(9, 0.4), anchor="bottom", mask="glitch", show=[87, 88.5, 92, 93])
X2 = 700
layer(I + "byen_natt.webp", X2, Y + 6, -80, fill(80, 1.3), show=[91.5, 93.5, 101, 102])
RISE_Y = ground(Y, 55, 0.9)
layer(I + "rammeverket_reiser_seg.webp", X2 + 2, RISE_Y - 30, -55, fill(55, 0.85), anchor="bottom", path=[[93, X2 + 2, RISE_Y - 30, -55], [99.5, X2 + 2, RISE_Y - 4, -55]], show=[93, 94, 101, 102], lights=[[0.52, 0.32, "#ff3cc8", 10, 0.5]])
cam(91, [X + 8, Y, -8], [X + 9, Y, -50])
cam(93.6, [X2, Y + 6, 10], [X2, Y + 8, -80])
cam(100, [X2, Y + 8, 4], [X2, Y + 16, -80])
say([86, 91], [X - 4, Y, 0], 26, [["text", "Boka ble tyngre for hver regel."], ["text", "Så kom den store oppgraderingen."]], dx=-0.78, dy=0.78, width=0.6)
say([94, 99.8], [X2, Y + 6, 10], 60, [["heading", "Alt som var bygd, brakk på én natt."], ["text", "Komponentene vred seg til noe annet, og over dem reiste rammeverket seg selv."]], dx=-0.78, dy=-0.45, width=0.8, size=0.08)

# --- 8. Forseglingen (100–125): munkene lokker dem ned, Ansgar går med ----------
X = 880
NIGHT_D = 600
layer(I + "natthimmel.webp", 850, 60, -NIGHT_D, fill(NIGHT_D, 2.2), follow=0.95, camY=5, show=[99.5, 101.5, 131, 132])
strip(I + "veien_natt.webp", 760, 960, -14, -120, fill(120, 1.1), show=[99, 101, 126, 128])
BX, BH = X + 10, fill(66, 1.7)
BW = BH * 2560 / 1429
layer(O + "begravelsen/bg.webp", BX, Y + 1, -60, BH, feather=[0.12, 0.03, 0, 0.1], show=[104, 108, 126, 127])
# Kryptdøra står 0,82 fra venstre og 0,6 fra toppen av kirkegårdsbildet (kirkedøra er den til venstre).
DOOR = [BX + (0.82 - 0.5) * BW, Y + 1 + (0.5 - 0.6) * BH, -58]
layer(I + "munkene_lokker.webp", 800, ground(Y, 26, 0.95), -26, fill(26, 0.42), anchor="bottom", walk=True, path=[[100, 806, ground(Y, 26, 0.95), -26], [118, DOOR[0] - 8, ground(Y, 26, 0.95), -26]], lights=[[0.9, 0.36, "#ffb050", 2.0, 1.0], [0.7, 0.38, "#ffb050", 2.0, 1.0]], show=[100.5, 102, 119, 121])
strip(I + "forgrunn_steiner.webp", 790, 900, ground(Y, 7, 1.02), -7, fill(7, 0.75), anchor="bottom", show=[100, 101, 115, 116.5])
layer(I + "ansgar_ned.webp", DOOR[0] - 4, ground(Y, 30, 0.9), -30, fill(30, 0.3), anchor="bottom", walk=True, path=[[112, DOOR[0] - 4, ground(Y, 30, 0.9), -30], [121.5, DOOR[0], DOOR[1] - 1.5, -55]], show=[112, 113, 120.5, 122], lights=[[0.43, 0.27, "#ffb050", 1.6, 1.0]])
cam(100.5, [790, Y + 2, 0], [800, Y, -100])
cam(108, [870, Y + 1, 0], [880, Y, -60])
cam(116, [DOOR[0] - 8, Y, 0], [DOOR[0] - 2, Y - 1, -60])
cam(123.5, [DOOR[0], DOOR[1] + 1, -20], [DOOR[0], DOOR[1], -60])
say([102, 108], [800, Y + 2, 0], 30, [["text", "Munkene i Fristad lokket dem ned i krypten under kirka, og murte igjen."]], dx=-0.78, dy=0.78, width=0.6)
say([110, 116], [0, 0, 0], 30, [["text", "Mester Ansgar gikk ned med dem, med de siste sidene av mønsterboka."]], dx=-0.78, dy=0.78, width=0.6)
say([117.5, 123], [0, 0, 0], 30, [["heading", "Han kom aldri opp igjen."]], dx=-0.6, dy=0.78, size=0.09, width=0.85)

# --- 9. Krypten i dag (125–138): ned gjennom jorda til det som sover ------------
X = 1000
JD = 40
jh = fill(JD, 1.55)
layer(I + "jordlag.webp", X, 0 - (0.5 - 0.1) * jh, -JD, jh, feather=[0.05, 0.05, 0.2, 0], show=[124.5, 125.5, 139, 140])
layer(O + "krypten/bg.webp", X, -45, -40, fill(40, 1.25), feather=[0.06, 0.06, 0.22, 0.12], show=[124.5, 125.5, 139, 140])
layer(O + "krypten/mid.webp", X + 1, ground(-45, 22, 0.95), -22, fill(22, 0.45), anchor="bottom", breathe=0.008, show=[124.5, 125.5, 139, 140])
layer(O + "krypten/bg.webp", X, -80, -75, fill(80, 1.45), tint="#6a5a7a", show=[124.5, 125.5, 139, 140])
layer(O + "krypten/fg.webp", X - 3.5, ground(-45, 8, 1.05), -8, fill(8, 0.25), anchor="bottom", show=[124.5, 125.5, 139, 140])
layer(I + "rammeverket_sover.webp", X, -80, -35, fill(35, 0.75), breathe=0.01, show=[124.5, 125.5, 139, 140], lights=[[0.52, 0.36, "#ff3cc8", 8, 0.35], [0.8, 0.72, "#ffb050", 1.4, 1.0]])
cam(125, [X, 7, 0], [X, 1, -40])
cam(129, [X, -15, 0], [X, -17, -40])
cam(132.5, [X, -44, 0], [X, -45, -40])
cam(138, [X, -77, -6], [X, -79, -35])
say([126, 131], [X, -15, 0], 25, [["quote", "Generasjoner senere er bare ryktene igjen."]], dx=0.0, dy=0.6, width=0.9, align="center", size=0.08)
say([133, 138], [X, -77, -6], 20, [["quote", "Men nede i mørket har noe begynt å våkne."]], dx=0.0, dy=-0.7, width=0.95, align="center", size=0.075)

# --- 10. Fristad, anno 1350 (138–151): veien til sørporten ---------------------
X = 1100
layer(I + "himmel_morgen.webp", X, 60, -SKY_D, fill(SKY_D, 2.2), follow=0.95, camY=5, show=[138, 139.5, 152, 153])
layer(O + "sorporten/bg.webp", X, Y, -70, fill(70, 1.25), show=[138, 139.5, 152, 153])
layer(pick("den_fremmede", "sorporten/mid.webp"), X - 4, ground(Y, 16, 0.92), -16, fill(16, 0.5), anchor="bottom", walk=True, path=[[138, X - 5, ground(Y, 16, 0.92), -16], [151, X - 1, ground(Y, 16, 0.92), -16]], show=[138, 139.5, 152, 153])
layer(pick("veiskiltet", "sorporten/fg.webp"), X - 7, ground(Y, 8, 1.02), -8, fill(8, 0.6), anchor="bottom", sway=0.03, show=[138, 139.5, 152, 153])
strip(I + "forgrunn_gress.webp", X - 12, X + 14, ground(Y, 5, 1.02), -5, fill(5, 0.7), anchor="bottom", sway=0.04, show=[138, 139.5, 152, 153])
cam(139.2, [X - 8, Y + 1.2, 0], [X - 2, Y, -70])
cam(151, [X + 1, Y, -10], [X + 1, Y - 0.5, -70])
say([140.5, 145.5], [X - 6, Y + 1, 0], 40, [["heading", "Fristad, anno 1350"]], dx=0.0, dy=0.55, size=0.14, width=1.6, align="center")
say([145.5, 150], [X - 1, Y, -6], 30, [["quote", "En fremmed kommer til porten."]], dx=0.0, dy=0.7, width=1.0, align="center", size=0.08)

SCRIPT = {
    "title": "Fristad – åpningsfilmen",
    "note": "Lagd av tools/film/intro.py etter docs/film/manus.md. Rediger skriptet, ikke denne fila.",
    "end": 151,
    "fade": [148.5, 151],
    "layers": LAYERS,
    "texts": TEXTS,
    "camera": sorted(CAMERA, key=lambda k: k["t"]),
    "events": [
        {"t": 0, "cue": "dawn", "sound": "road"},
        {"t": 14, "cue": "scriptorium", "sound": "hall"},
        {"t": 35, "cue": "council", "sound": None},
        {"t": 48, "cue": "decree", "sound": "market"},
        {"t": 60, "cue": "inquisition", "sound": "night"},
        {"t": 72, "cue": "workarounds", "sound": "hall"},
        {"t": 85, "cue": "fall", "sound": "glitch"},
        {"t": 100, "cue": "burial", "sound": "night"},
        {"t": 125, "cue": "crypt", "sound": "crypt"},
        {"t": 138.5, "cue": "gate", "sound": "road"},
    ],
    "veils": [
        {"t": [-1, -1, 0.3, 2.5]},
        {"t": [34.3, 35.1, 36.0, 37.4], "color": "white"},
        {"t": [71.7, 72.3, 72.8, 73.6]},
        {"t": [91.3, 91.8, 92.6, 93.4]},
        {"t": [93.9, 93.95, 94.0, 94.25], "color": "white"},
        {"t": [96.4, 96.45, 96.5, 96.8], "color": "white"},
        {"t": [122.2, 123.6, 124.6, 125.8]},
        {"t": [137.6, 138.3, 138.6, 139.6]},
    ],
    "dust": [
        {"t": 0, "amount": 0.25, "color": "#ffe9c0", "drift": [0.3, 0.05, 0]},
        {"t": 21, "amount": 0.7, "color": "#ffd48a", "drift": [0.05, 0.03, 0]},
        {"t": 35.5, "amount": 0.25, "color": "#cfe0ff", "drift": [0.02, -0.02, 0]},
        {"t": 48, "amount": 0.3, "color": "#ffe9c0", "drift": [0.1, 0.03, 0]},
        {"t": 60, "amount": 0.45, "color": "#ffb070", "drift": [0.05, 0.2, 0]},
        {"t": 72.5, "amount": 0.4, "color": "#ffd48a", "drift": [0.02, 0.02, 0]},
        {"t": 85, "amount": 0.6, "color": "#ff7ae0", "drift": [0.0, 0.3, 0]},
        {"t": 100, "amount": 0.3, "color": "#cfe0ff", "drift": [0.15, -0.05, 0]},
        {"t": 125, "amount": 0.5, "color": "#b9c8ff", "drift": [0.0, -0.1, 0]},
        {"t": 138.5, "amount": 0.3, "color": "#ffe9c0", "drift": [0.3, 0.05, 0]},
    ],
}

if __name__ == "__main__":
    place_texts()
    out = ROOT / "web/film/opening.json"
    out.write_text(json.dumps(SCRIPT, ensure_ascii=False, indent=1) + "\n")
    print(out, len(LAYERS), "lag,", len(TEXTS), "tekster,", len(CAMERA), "kameranøkler")
