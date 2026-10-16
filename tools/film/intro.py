"""Åpningsfilmen som data: skriver `web/film/opening.json` etter
`docs/film/manus.md` (kanon: `docs/kanon.md`).

Filmen er SCENER (eieren 16. oktober: «Du skal lage SCENER med lag som ser bra
ut, og som har dybde mellom lagene, ikke dette sirkuset»). Hver scene har et
heltekamera og et maleri som fyller bildet `BG_D` foran det. Utklippene
plasseres i BILDET: midten `cx` og foten `foot` (brøk av bildet, 0 oppe til
venstre), høyden `h` (brøk av bildehøyden) og dybden `d` foran kameraet. Foten
er den nederste pikselen i utklippet (alfa), og den settes på gulvet i
maleriet eller rett under bildekanten, så ingen svever. Kameraet holder høyden
i scenen, så føttene står der de står; det glir bare litt sidelengs (lagene
forskyves mot hverandre) og zoomer litt. Utklippene males fram når kameraet er
framme og bort før det går videre, og mellom scenene glir kameraet sidelengs
til neste maleri, som står ved siden av med myk skjøt.

  python3 tools/film/intro.py
"""

import json
import math
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
FILM = ROOT / "assets/film"
FOV = 40
TAN = math.tan(math.radians(FOV / 2))
WIDE = 16 / 9
BG_D = 60  # maleriet bak hver scene
BG_K = 1.2  # så mye større enn bildet maleriet er (rom for sidegliding og zoom)
ZI = -44  # kameraet inne i Skriptoriet og i alle scenene etter det
I = "intro/"
O = "opening/"


def fill(d, k=1.0):
    """Høyden som fyller bildet `d` foran kameraet (gange `k`)."""
    return 2 * TAN * d * k


def r3(v):
    return [round(x, 3) for x in v]


_BOX = {}


def box(src):
    """Bildets sideforhold, omrisset av det som synes (alfa over 0,5) som
    brøk (venstre, topp, høyre, bunn) og tyngdepunktet sidelengs (en skygge
    på bakken drar ikke figuren til siden)."""
    if src not in _BOX:
        im = Image.open(FILM / src)
        a = np.asarray(im.convert("RGBA"))[..., 3] > 127
        ys, xs = np.nonzero(a)
        h, w = a.shape
        _BOX[src] = (im.width / im.height, xs.min() / w, ys.min() / h, (xs.max() + 1) / w, (ys.max() + 1) / h, xs.mean() / w)
    return _BOX[src]


LAYERS = []
CAMERA = []
SAYS = []


def layer(src, at, h, **o):
    LAYERS.append({"src": src, "at": r3(at), "h": round(h, 3), **o})


def cam(t, p, fov=FOV):
    CAMERA.append({"t": t, "p": r3(p), "look": r3([p[0], p[1], p[2] - 10]), "fov": fov})


def strip(src, x0, x1, y, z, h, **o):
    """Et bånd som gjentas langs x, annethvert speilvendt, så skjøtene møtes."""
    w = h * box(src)[0]
    for i in range(math.ceil((x1 - x0) / w) + 1):
        layer(src, [x0 + i * w * 0.985, y, z], h, anchor="bottom", flip=bool(i % 2), **o)


def backdrop(src, x, y, z, show=(22.4, 22.5, 999, 999), feather=(0.08, 0, 0, 0), **o):
    """Maleriet bak en scene, `BG_D` foran heltekameraet i (x, y, z+BG_D).
    Skjøten mot maleriet til venstre er myk (`feather`). Maleriene står ikke
    før kameraet er inne i Skriptoriet (de ville ellers synes bak åsene)."""
    o = {**o, "feather": list(feather)}
    if show:
        o["show"] = show
    layer(src, [x, y, z], fill(BG_D, BG_K), **o)


def figure(src, hero, cx, foot, h, d, show, **o):
    """Et utklipp sett fra heltekameraet `hero`: midten av det som synes i
    `cx` (tyngdepunktet), foten i `foot` (brøk av bildet), `h` av
    bildehøyden, `d` foran."""
    aspect, x0, y0, x1, y1, mx = box(src)
    fh = fill(d)
    H = h * fh / (y1 - y0)
    W = H * aspect
    foot_y = hero[1] + (0.5 - foot) * fh
    x = hero[0] + (cx - 0.5) * fh * WIDE - (mx - 0.5) * W
    layer(src, [x, foot_y - (1 - y1) * H, hero[2] - d], H, anchor="bottom", show=show, mask=o.pop("mask", "ink"), **o)


def say(t, hero, lines, dx, dy, size=0.075, width=0.6, align="left", d=20):
    """En tekst `d` foran heltekameraet, med venstre kant (eller midten) `dx`
    og toppen `dy` ut i bildet (brøk av halve bredden og høyden, 0 er midten),
    og overskriftshøyden `size` av bildehøyden. `place_texts` sjekker at den
    holder seg inne i bildet mens den står, og flytter den om ikke."""
    SAYS.append((t, hero, d, lines, dx, dy, size, width, align))


class Scene:
    """Et heltekamera i (x, y, z) og tiden kameraet er framme [inn, ut].
    Kameraet glir `truck` sidelengs og zoomer fra `fov[0]` til `fov[1]`."""

    def __init__(self, x, y, z, t_in, t_out, truck=1.5, fov=(40, 37), keys=True):
        self.hero = [x, y, z]
        self.t = (t_in, t_out)
        if keys:
            cam(t_in, [x - truck / 2, y, z], fov[0])
            cam(t_out, [x + truck / 2, y, z], fov[1])

    def show(self, at=0.3, until=None, fade=1.6):
        """Når et utklipp males fram (`at` etter at kameraet er framme) og bort."""
        a = self.t[0] + at
        b = self.t[1] if until is None else self.t[0] + until
        return [round(a, 2), round(a + fade, 2), round(b - 1.4, 2), round(b - 0.2, 2)]

    def figure(self, src, cx, foot, h, d, at=0.3, until=None, **o):
        figure(src, self.hero, cx, foot, h, d, self.show(at, until), **o)

    def say(self, t, lines, dx, dy, **o):
        say([self.t[0] + t[0], self.t[0] + t[1]], self.hero, lines, dx, dy, **o)


BG_W = fill(BG_D, BG_K) * 2048 / 1143
SP = BG_W * 0.92  # avstanden mellom maleriene: de overlapper, med myk skjøt

# --- 1. Riket (0–13): morgen over åsene med byene ----------------------------------------
SKY_D = 600
HILLS = [0, 0, 20.8, 21.5]
layer(I + "himmel_morgen.webp", [40, 30, -SKY_D], fill(SKY_D, 1.35), follow=0.9, camY=0, show=HILLS)
strip(I + "aaser_fjerne.webp", -120, 260, (0.5 - 0.93) * fill(200), -200, fill(200, 1.1), show=HILLS)
strip(I + "aaser_naere.webp", -70, 140, (0.5 - 1.06) * fill(70), -70, fill(70, 0.98), show=HILLS)
RIKET = Scene(2, 0, 0, 0.0, 12.5, truck=10, fov=(40, 40))
RIKET.say((1.5, 6.6), [["heading", "Fristad"]], 0.0, 0.62, size=0.17, width=1.2, align="center", d=40)
RIKET.say((7.0, 12.8), [["text", "Riket var fullt av byer, og hver by hadde sitt skriptorium og sitt eget tungemål."], ["quote", "Men alle delte én mønsterbok."]], 0.0, 0.7, width=0.62, align="center", d=40)

# --- 2. Skriptoriet (13–35): fasaden i landskapet, inn gjennom vinduet, Ansgar ----------
SIZES = json.loads((FILM / "intro/sizes.json").read_text())
win = SIZES["skriptoriet_fasade"]["hole"]
XF = 64
fh = fill(40)
fw = fh * box(I + "skriptoriet_fasade.webp")[0]
# Vinduet står midt i bildet fra (XF, 0, 0); kameraet går rett inn gjennom det.
layer(I + "skriptoriet_fasade.webp", [XF + (0.5 - win["x"]) * fw, (win["y"] - 0.5) * fh, -40], fh)
cam(16.6, [XF, 0, 0])
cam(18.6, [XF, 0, 0])
cam(23.0, [XF, 0, ZI])
GULL = Scene(XF, 0, ZI, 23.0, 35.0, keys=False)
cam(35.0, [XF + 0.7, 0, ZI], 37)
backdrop(O + "gullalderen/bg.webp", XF, 0, ZI - BG_D, show=[20.8, 21.5, 999, 999], feather=(0, 0, 0, 0))
GULL.figure(I + "ansgar_bok.webp", 0.79, 1.02, 0.84, 13, at=0.6, lights=[[0.6, 0.48, "#ffe7a8", 1.2, 0.3]])
GULL.say((1.0, 4.6), [["quote", "Den kunne leses på alle tungemål."]], -0.88, 0.88, size=0.08, width=0.75)
GULL.say((4.9, 8.4), [["text", "Hver by bygde av den akkurat det den trengte, og boka sto aldri i veien."]], -0.88, 0.88, width=0.55)
GULL.say((8.6, 11.9), [["text", "I Fristad var det"], ["heading", "mester Ansgar"], ["text", "som holdt den."]], -0.88, 0.88, size=0.1, width=0.5)

# --- 3. Rådet (35–49) ---------------------------------------------------------------------
X = XF + SP
RAAD = Scene(X, 0, ZI, 38.5, 49.0)
backdrop(I + "raadssal.webp", X, 0, ZI - BG_D, tint="#e4ebff")
RAAD.figure(I + "raadet.webp", 0.5, 0.95, 0.55, 28, at=0.2)
RAAD.figure(I + "kjopmenn.webp", 0.2, 0.97, 0.4, 21, at=1.8)
RAAD.figure(I + "komponentboken.webp", 0.85, 1.03, 0.55, 11, at=5.0, lights=[[0.5, 0.3, "#9fc4ff", 1.6, 0.2]])
RAAD.say((0.6, 5.0), [["text", "I hovedstaden satt et råd som aldri hadde holdt en fjærpenn,"], ["text", "og ved siden av dem kjøpmennene fra Kontoret med sine regler."]], 0.0, 0.88, width=0.9, align="center")
RAAD.say((5.3, 10.2), [["quote", "De mente at alle byer burde ha det likt,"], ["quote", "og at de visste best hvordan."]], 0.0, 0.88, width=0.9, align="center", size=0.08)

# --- 4. Budet (49–62): herolden på torget -------------------------------------------------
X += SP
BUD = Scene(X, 0, ZI, 52.0, 62.0)
backdrop(I + "torget_bud.webp", X, 0, ZI - BG_D)
BUD.figure(I + "herold.webp", 0.74, 0.87, 0.5, 30, at=0.2)
BUD.figure(I + "folkemengde_bud.webp", 0.32, 1.03, 0.5, 12, at=0.6)
BUD.say((0.5, 9.6), [["text", "Så kom kongens bud, på kanselliets latin:"], ["quote", "Omnia lingua Reactia."], ["text", "Alt på hoffets tunge."], ["quote", "Omnia una compage."], ["text", "Alt med ett rammeverk."], ["quote", "Nulla exceptio."], ["text", "Ingen avvik."]], -0.9, 0.9, width=0.5, size=0.07)

# --- 5. Inkvisisjonen (62–76): foran Skriptoriet om kvelden ----------------------------------
X += SP
INK = Scene(X, 0, ZI, 65.0, 76.0, keys=False)
backdrop(I + "skriptoriet_kveld.webp", X, 0, ZI - BG_D)
INK.figure(I + "inkvisitorer.webp", 0.55, 0.97, 0.55, 20, at=0.2)
INK.figure(I + "baal.webp", 0.2, 1.03, 0.3, 9, at=1.5, lights=[[0.5, 0.45, "#ff9a3c", 2.4, 1.0], [0.4, 0.3, "#ffb050", 1.4, 1.0]])
INK.figure(I + "komponent_i_ramme.webp", 0.86, 1.03, 0.38, 8, at=3.8, lights=[[0.5, 0.45, "#e8ff9a", 0.8, 0.6]])
# Kameraet glir litt mot komponenten i rammen.
cam(65.0, [X - 1.0, 0, ZI], 40)
cam(76.0, [X + 1.0, 0, ZI], 37)
INK.say((0.5, 3.8), [["heading", "Inkvisisjonen"], ["text", "dro fra by til by."]], -0.9, 0.9, size=0.11, width=0.6)
INK.say((4.0, 6.4), [["text", "Mønsterbøkene ble brent, og hver komponent ble spent fast i rammeverket, med Kontorets regler stemplet på."]], -0.9, 0.9, width=0.55)
INK.say((7.6, 11.2), [["quote", "En knapp kunne ikke lenger"], ["quote", "bare være en knapp."]], 0.15, 0.9, width=0.8, size=0.08)

# --- 6. Omveiene (76–89): skriverne om natta ----------------------------------------------------
X += SP
NATT = Scene(X, 0, ZI, 79.0, 89.0, truck=1.2, fov=(40, 38))
backdrop(I + "skriptoriet_natt.webp", X, 0, ZI - BG_D, show=[22.4, 22.5, 91.0, 92.0])
NATT.figure(I + "omveiene.webp", 0.5, 1.02, 0.5, 14, at=0.2, until=10.6, lights=[[0.385, 0.2, "#ffcf70", 0.5], [0.705, 0.17, "#ffcf70", 0.5]])
NATT.say((0.6, 5.0), [["text", "Skriverne lærte å skrive rundt boka i stedet for med den."]], -0.9, 0.9, width=0.6)
NATT.say((5.3, 9.8), [["quote", "Omvei på omvei."], ["quote", "Lapp på lapp."], ["quote", "Natt etter natt."]], 0.42, 0.9, width=0.4, size=0.075)

# --- 7. Bruddet (89–104): samme rom brekker, så reiser rammeverket seg over byen -------------
FALL = Scene(X, 0, ZI, 89.0, 96.0, keys=False)
cam(96.0, [X + 0.9, 0, ZI], 37)
backdrop(O + "fallet/bg.webp", X, 0, ZI - BG_D + 0.1, show=[89.0, 91.0, 96.0, 96.2], mask="glitch")
FALL.figure(O + "fallet/mid.webp", 0.62, 0.95, 0.45, 20, at=1.4, until=7.2, mask="glitch")
FALL.figure(O + "fallet/fg.webp", 0.19, 1.03, 0.4, 8, at=2.2, until=7.2, mask="glitch")
FALL.say((0.6, 6.6), [["text", "Boka ble tyngre for hver regel."], ["text", "Så kom den store oppgraderingen."]], -0.9, 0.9, width=0.6)
X += SP
BYEN = Scene(X, 0, ZI, 96.1, 104.0, truck=1.0, fov=(40, 37))
backdrop(I + "byen_natt.webp", X, 0, ZI - BG_D)
BYEN.figure(I + "rammeverket_reiser_seg.webp", 0.77, 1.03, 0.72, 32, at=0.6, until=8.4, mask="glitch", lights=[[0.52, 0.32, "#ff3cc8", 4, 0.5]])
BYEN.say((1.2, 7.6), [["heading", "Alt som var bygd, brakk på én natt."], ["text", "Komponentene vred seg til noe annet, og over dem reiste rammeverket seg selv."]], -0.9, 0.9, width=0.42, size=0.08)

# --- 8. Forseglingen (104–124): kirkegården, munkene og Ansgar ----------------------------
X += SP
GRAV = Scene(X, 0, ZI, 107.5, 124.0, truck=1.2, fov=(40, 38))
backdrop(O + "begravelsen/bg.webp", X, 0, ZI - BG_D)
GRAV.figure(I + "munkene_lokker.webp", 0.47, 1.03, 0.38, 14, at=0.3, lights=[[0.9, 0.36, "#ffb050", 0.8, 1.0], [0.7, 0.38, "#ffb050", 0.8, 1.0]])
GRAV.figure(I + "ansgar_ned.webp", 0.17, 1.03, 0.62, 11, at=6.0, lights=[[0.43, 0.27, "#ffb050", 0.9, 1.0]])
GRAV.say((0.8, 5.8), [["text", "Munkene i Fristad lokket dem ned i krypten under kirka, og murte igjen."]], -0.9, 0.9, width=0.55)
GRAV.say((6.4, 11.4), [["text", "Mester Ansgar gikk ned med dem, med de siste sidene av mønsterboka."]], -0.9, 0.9, width=0.55)
GRAV.say((11.9, 16.0), [["heading", "Han kom aldri opp igjen."]], -0.9, 0.9, size=0.09, width=0.8)

# --- 9. Krypten i dag (124–140): ned gjennom jorda, og det som sover --------------------------
# Jordlaget henger under kirkegårdsmaleriet, og kameraet senker seg rett ned.
JH = fill(BG_D, BG_K)
JTOP = -JH / 2 + 0.27 * JH + 2.0  # gresset i jordlaget (0,27 ned) litt over bunnen av kirkegården
layer(I + "jordlag.webp", [X, JTOP - JH, ZI - BG_D + 0.1], JH, anchor="bottom", feather=[0, 0, 0, 0.18])
JORD_Y = JTOP - JH + fill(BG_D) / 2 + 0.6
cam(124.2, [X + 0.6, 0, ZI], 38)
cam(128.4, [X + 0.6, JORD_Y, ZI], 40)
cam(131.9, [X + 0.6, JORD_Y, ZI], 38)
say([128.0, 131.4], [X + 0.6, JORD_Y, ZI], [["quote", "Generasjoner senere er bare ryktene igjen."]], 0.0, 0.88, width=0.9, align="center", size=0.08)
X += SP
KRYPT = Scene(X, 0, ZI, 132.5, 140.0, truck=0.8, fov=(40, 36))
backdrop(O + "krypten/bg.webp", X, 0, ZI - BG_D, tint="#c9c2d8")
KRYPT.figure(I + "rammeverket_sover.webp", 0.5, 0.9, 0.38, 30, at=0.2, lights=[[0.52, 0.36, "#ff3cc8", 3, 0.35], [0.8, 0.72, "#ffb050", 0.7, 1.0]])
KRYPT.figure(O + "krypten/fg.webp", 0.17, 1.04, 0.3, 8, at=0.2)
KRYPT.say((1.2, 7.0), [["quote", "Men nede i mørket har noe begynt å våkne."]], 0.0, 0.88, width=0.9, align="center", size=0.08)

# --- 10. Fristad, anno 1350 (140–155): den fremmede på veien til sørporten ------------------
X += SP
PORT = Scene(X, 0, ZI, 141.0, 155.0, truck=1.2, fov=(40, 33))
backdrop(O + "sorporten/bg.webp", X, 0, ZI - BG_D)
PORT.figure(I + "den_fremmede.webp", 0.4, 1.03, 0.62, 10, at=0.0, until=99)
PORT.figure(I + "veiskiltet.webp", 0.14, 1.03, 0.5, 8, at=0.0, until=99)
PORT.say((1.0, 6.0), [["heading", "Fristad, anno 1350"]], 0.0, 0.8, size=0.14, width=1.2, align="center")
PORT.say((6.4, 11.0), [["quote", "En fremmed kommer til porten."]], 0.0, 0.8, width=0.9, align="center", size=0.08)


def slope(keys, k, j):
    """Samme bane som filmen: monoton kubisk (Fritsch–Butland)."""
    if k == 0 or k == len(keys) - 1:
        return 0
    p, c, n = keys[k - 1], keys[k], keys[k + 1]
    h0, h1 = c["t"] - p["t"], n["t"] - c["t"]
    d0 = (c["p"][j] - p["p"][j]) / h0
    d1 = (n["p"][j] - c["p"][j]) / h1
    if d0 * d1 <= 0:
        return 0
    return 3 * (h0 + h1) / ((2 * h1 + h0) / d0 + (h1 + 2 * h0) / d1)


def pose(keys, t):
    i = 0
    while i < len(keys) - 2 and t > keys[i + 1]["t"]:
        i += 1
    a, b = keys[i], keys[i + 1]
    if t <= a["t"]:
        return a["p"], a["fov"]
    if t >= b["t"]:
        return b["p"], b["fov"]
    h = b["t"] - a["t"]
    s = (t - a["t"]) / h
    s2, s3 = s * s, s * s * s
    p = [a["p"][j] * (2 * s3 - 3 * s2 + 1) + slope(keys, i, j) * h * (s3 - 2 * s2 + s) + b["p"][j] * (-2 * s3 + 3 * s2) + slope(keys, i + 1, j) * h * (s3 - s2) for j in range(3)]
    return p, a["fov"] + (b["fov"] - a["fov"]) * (3 * s2 - 2 * s3)


SCALE = {"heading": 1.0, "quote": 0.78, "latin": 0.62, "text": 0.5}


def block(lines, h, width):
    """Bredden og høyden på en tekst (verdensenheter), grovt: tegnene er
    rundt halvparten så brede som linja er høy."""
    w = hh = 0
    for style, line in lines:
        lh = h * SCALE[style]
        chars = len(line) * lh * 0.5
        rows = max(1, math.ceil(chars / width))
        w = max(w, min(chars, width))
        hh += rows * lh * 1.25 + lh * 0.2
    return w, hh


def outside(keys, t, at, w, hh, h, align):
    """Hvor langt teksten stikker ut av bildet (brøk, 0 er inne med marg)."""
    worst = 0
    x0 = {"left": 0, "center": -w / 2, "right": -w}[align]
    for i in range(13):
        p, fov = pose(keys, t[0] + (t[1] - t[0]) * i / 12)
        tn = math.tan(math.radians(fov / 2))
        dz = p[2] - at[2]
        if dz <= 0.5:
            return 9
        for cx in (x0, x0 + w):
            for cy in (0.6 * h, -hh):
                sx = (at[0] + cx - p[0]) / (dz * tn * WIDE)
                sy = (at[1] + cy - p[1]) / (dz * tn)
                worst = max(worst, abs(sx) - 0.94, sy - 0.94, -sy - 0.9)
    return max(worst, 0)


TEXTS = []


def place_texts():
    keys = CAMERA
    for t, hero, d, lines, dx, dy, size, width, align in SAYS:
        vh = fill(d)
        vw = vh * WIDE
        h = size * vh
        wv = width * vw
        bw, bh = block(lines, h, wv)
        best = None
        cands = sorted((ox * ox + oy * oy, ox, oy) for ox in [i * 0.04 for i in range(-10, 11)] for oy in [i * 0.04 for i in range(-10, 11)])
        for _, ox, oy in cands:
            at = [hero[0] + (dx + ox) * vw / 2, hero[1] + (dy + oy) * vh / 2 - h * 0.6, hero[2] - d]
            if outside(keys, t, at, bw, bh, h, align) == 0:
                best = at
                if ox or oy:
                    print(f"  flyttet {ox:+.2f},{oy:+.2f}:", lines[0][1])
                break
        if best is None:
            print("teksten stikker ut:", lines[0][1])
            best = [hero[0] + dx * vw / 2, hero[1] + dy * vh / 2 - h * 0.6, hero[2] - d]
        TEXTS.append({"t": t, "at": r3(best), "lines": lines, "h": round(h, 3), "width": round(wv, 3), "align": align, "pace": 0.12})


SCRIPT = {
    "title": "Fristad – åpningsfilmen",
    "note": "Lagd av tools/film/intro.py etter docs/film/manus.md. Rediger skriptet, ikke denne fila.",
    "end": 155,
    "fade": [152.5, 155],
    "layers": LAYERS,
    "texts": TEXTS,
    "camera": CAMERA,
    "events": [
        {"t": 0, "cue": "dawn", "sound": "road"},
        {"t": 14, "cue": "scriptorium", "sound": "hall"},
        {"t": 35.5, "cue": "council", "sound": None},
        {"t": 49.5, "cue": "decree", "sound": "market"},
        {"t": 62.5, "cue": "inquisition", "sound": "night"},
        {"t": 76.5, "cue": "workarounds", "sound": "hall"},
        {"t": 89, "cue": "fall", "sound": "glitch"},
        {"t": 104.5, "cue": "burial", "sound": "night"},
        {"t": 124.5, "cue": "crypt", "sound": "crypt"},
        {"t": 140.5, "cue": "gate", "sound": "road"},
    ],
    "veils": [
        {"t": [-1, -1, 0.3, 2.5]},
        {"t": [95.7, 96.0, 96.15, 96.9], "color": "white"},
        {"t": [98.3, 98.35, 98.4, 98.7], "color": "white"},
        {"t": [131.3, 131.9, 132.5, 133.2]},
        {"t": [139.4, 140.0, 141.0, 141.8]},
    ],
    "dust": [
        {"t": 0, "amount": 0.2, "color": "#ffe9c0", "drift": [0.2, 0.03, 0]},
        {"t": 22, "amount": 0.6, "color": "#ffd48a", "drift": [0.05, 0.03, 0]},
        {"t": 36, "amount": 0.2, "color": "#cfe0ff", "drift": [0.02, -0.02, 0]},
        {"t": 50, "amount": 0.25, "color": "#ffe9c0", "drift": [0.1, 0.03, 0]},
        {"t": 63, "amount": 0.4, "color": "#ffb070", "drift": [0.05, 0.2, 0]},
        {"t": 77, "amount": 0.35, "color": "#ffd48a", "drift": [0.02, 0.02, 0]},
        {"t": 89, "amount": 0.5, "color": "#ff7ae0", "drift": [0.0, 0.3, 0]},
        {"t": 105, "amount": 0.25, "color": "#cfe0ff", "drift": [0.1, -0.05, 0]},
        {"t": 125, "amount": 0.4, "color": "#b9c8ff", "drift": [0.0, -0.1, 0]},
        {"t": 141, "amount": 0.25, "color": "#ffe9c0", "drift": [0.2, 0.03, 0]},
    ],
}

if __name__ == "__main__":
    CAMERA.sort(key=lambda k: k["t"])
    times = [k["t"] for k in CAMERA]
    assert len(times) == len(set(times)), "to kameranøkler på samme tid"
    place_texts()
    out = ROOT / "web/film/opening.json"
    out.write_text(json.dumps(SCRIPT, ensure_ascii=False, indent=1) + "\n")
    print(out, len(LAYERS), "lag,", len(TEXTS), "tekster,", len(CAMERA), "kameranøkler")
