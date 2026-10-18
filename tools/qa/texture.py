"""P-T: teksturlagene (`docs/plan.md` del 3).

Kilden er eierens malte bilde (`assets/textures/painted/<lag>/albedo.png`);
laget spillet bruker er det avledede (`assets/textures/<lag>/albedo.jpg`,
`tools/textures/derive.py`). Kilden må være stor nok, uten ramme, med jevnt
lys og uten perspektiv; laget må gå i ett med seg selv i alle retninger og
være skarpt. Arkene (`kind` i katalogen) sjekkes bare for størrelse og
skarphet. Alle målene er rene numpy-regninger på sekunder.
"""
import glob
import os

import numpy as np
from PIL import Image
from scipy import ndimage

from .gate import ROOT, Report

PAINTED = os.path.join(ROOT, "assets", "textures", "painted")
LAYERS = os.path.join(ROOT, "assets", "textures")

# Tersklene. Hver har en begrunnelse; de strammes, aldri slakkes.
# Bildegeneratoren gir rundt 1888 × 1888 (3,6 Mpx); 1,8 Mpx er minst det som
# gir 1 teksel per mm på en flis på 1,3 m.
MIN_MPX = 1.8
# En ramme eller bård: en rad eller kolonne ytterst som er mer enn 25 ganger
# jevnere enn en vanlig rad (ekte rammer gir over 1000; en fuge eller bordkant
# langs kanten, som i ashlar, under 20).
MAX_FRAME = 25.0
# Jevnt lys: den store variasjonen (uskarp til en åttendedel av bildet) skal
# ikke spenne mer enn 18 % av lysstyrken, ellers blir flisene flekkete på avstand.
MAX_LIGHT = 0.18
# Perspektiv (et bilde tatt på skrå) måles ikke her: detaljenergien skilte ikke
# perspektiv fra mønstre som er grovere på én side med vilje (core_stone), så
# det vurderes med øyet i den estetiske kontrollen (`docs/kvalitet/laerdom.md`).
# Flisbart: skjøten når flisa legges mot seg selv, skal ikke være mer markert
# enn den mest markerte naturlige overgangen i bildet (99-persentilen av
# overgangene mellom nabokolonner, som fugene i murverk). Kalibrert 11. okt.
# mot bilder: skjøtene synes med øyet fra rundt 1,0 (thatch 1,5, snow 1,2).
MAX_SEAM = 1.0
# Skarpt: de sterkeste kantene skal miste minst 40 % når bildet gjøres 1
# piksel uskarpere. Myke, oppskalerte bilder (oak_planed 1,34, snow 1,22)
# mister lite; skarpe malte (wall_stone 1,74) mye. Kalibrert 11. okt.
MIN_CRISP = 1.40


def luma(a):
    return a[..., :3] @ np.array([0.2126, 0.7152, 0.0722])


def load(path):
    return np.asarray(Image.open(path).convert("RGB"), dtype=np.float64) / 255.0


def frame(l):
    """En ramme eller bård er en stripe ytterst (3 %) som er jevn langs hele
    kanten. Gir hvor mange ganger jevnere den jevneste raden eller kolonnen
    ytterst er enn en vanlig rad inne i bildet (mønster som kuttes ved
    kanten, som takstein, er like urolig som resten)."""
    h, w = l.shape
    worst = 0.0
    for a in (l, l.T, l[::-1], l.T[::-1]):
        n = max(2, a.shape[0] * 3 // 100)
        stds = a.std(axis=1)
        typical = max(np.median(stds[n:-n]), 1e-4)
        worst = max(worst, typical / max(stds[:n].min(), 1e-4))
    return worst


def light(l):
    """Spennet i den store lysvariasjonen (p95 − p5 av et kraftig uskarpt
    bilde) delt på middelverdien."""
    small = l[:: max(1, l.shape[0] // 256), :: max(1, l.shape[1] // 256)]
    blur = ndimage.gaussian_filter(small, sigma=small.shape[0] / 8, mode="wrap")
    return (np.percentile(blur, 95) - np.percentile(blur, 5)) / max(blur.mean(), 1e-3)


def seam(l):
    """Skjøten når flisa legges mot seg selv, mot 99-persentilen av de
    naturlige overgangene mellom nabokolonner. Begge målt som den verste
    åttendedelen langs overgangen, så en skjøt som bare synes et stykke,
    ikke drukner. Den verste av de to retningene."""
    out = []
    for a in (l, l.T):
        span = max(1, a.shape[0] // 8)
        cols = ndimage.uniform_filter1d(np.abs(np.diff(a, axis=1)), size=span, axis=0).max(axis=0)
        edge = ndimage.uniform_filter1d(np.abs(a[:, 0] - a[:, -1]), size=span).max()
        out.append(edge / max(np.percentile(cols, 99), 1e-6))
    return max(out)


def crisp(l):
    """Hvor mye de sterkeste kantene (99-persentilen av gradienten) mister
    når bildet gjøres 1 piksel uskarpere."""
    def p99(a):
        return np.percentile(np.hypot(ndimage.sobel(a, 0), ndimage.sobel(a, 1)), 99)
    return p99(l) / max(p99(ndimage.gaussian_filter(l, 1.0)), 1e-9)


def kind(name):
    """Flis eller ark (`assets/catalog/textures.json`). Et ark (dør, list,
    skilt, vinduer) legges ikke i flis, så ramme, lys og skjøt gjelder ikke."""
    import json
    with open(os.path.join(ROOT, "assets", "catalog", "textures.json")) as f:
        return json.load(f)["textures"].get(name, {}).get("kind", "tile")


def check(name):
    """P-T for laget `name`: kilden og det avledede laget."""
    r = Report("P-T", name)
    tile = kind(name) == "tile"
    src = os.path.join(PAINTED, name, "albedo.png")
    if os.path.exists(src):
        a = load(src)
        l = luma(a)
        r.add("kilde: megapiksler", a.shape[0] * a.shape[1] / 1e6, MIN_MPX, "min", "stor nok til 1 teksel per mm")
        if tile:
            r.add("kilde: ramme", frame(l), MAX_FRAME, "max", "ingen ramme, bård eller kant rundt motivet")
            r.add("kilde: lys", light(l), MAX_LIGHT, "max", "jevnt lys over hele flisa")
    out = os.path.join(LAYERS, name, "albedo.jpg")
    if os.path.exists(out):
        l = luma(load(out))
        if tile:
            r.add("lag: skjøt", seam(l), MAX_SEAM, "max", "flisa går i ett med seg selv")
        r.add("lag: skarphet", crisp(l), MIN_CRISP, "min", "skarpe penselstrøk, ikke mykt eller oppskalert")
    return r


def names():
    """Alle lagene som har en malt kilde eller et avledet lag."""
    found = {os.path.basename(os.path.dirname(p)) for p in glob.glob(os.path.join(PAINTED, "*", "albedo.png"))}
    found |= {os.path.basename(os.path.dirname(p)) for p in glob.glob(os.path.join(LAYERS, "*", "albedo.jpg"))}
    return sorted(found)
