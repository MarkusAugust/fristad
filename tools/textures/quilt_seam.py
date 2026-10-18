"""Gjør en malt flis flisbar uten spøkelser: bildegeneratoren lager ofte
fliser der formene nesten møtes, men strøkene i kantradene ikke passer, så
det blir en strek i skjøten. Her overlappes et bånd nederst med et bånd
øverst (og høyre med venstre), og skjøten legges langs den veien gjennom
båndet der de to er mest like (minste feil, dynamisk programmering, som i
image quilting), oftest langs fugene. Bildet blir båndet mindre hver vei,
og møtes da med seg selv uten synlig søm.

    python3 tools/textures/quilt_seam.py <inn> <ut.png> [--band 160] [--even] [--only y|x]

`--even` jevner ut de store lysforskjellene først (generatoren gjør ofte
kantene mørkere enn midten, som blir et rutenett når flisen gjentas): den
glattede lysheten deles bort og erstattes med snittet, så detaljene står.
`--only y` syr bare topp mot bunn (og `--only x` bare sideveis), for
bilder som allerede er sømløse den andre veien.
"""

import sys

import numpy as np
from PIL import Image


def cut_rows(a, band):
    """Overlapper de nederste `band` radene med de øverste og fjerner de
    øverste fra bildet: bildet blir `band` rader lavere og flisbart i
    høyden."""
    h = a.shape[0]
    top = a[:band]
    bot = a[h - band:]
    err = ((top - bot) ** 2).sum(axis=2)  # (band, w)
    # Veien går fra venstre mot høyre; i hver kolonne velges raden der
    # bildet bytter fra bunnen (over) til toppen (under).
    w = err.shape[1]
    cost = err.copy()
    back = np.zeros_like(err, dtype=np.int32)
    for x in range(1, w):
        prev = cost[:, x - 1]
        left = np.concatenate(([np.inf], prev[:-1]))
        right = np.concatenate((prev[1:], [np.inf]))
        choice = np.stack([left, prev, right])
        k = choice.argmin(axis=0)
        cost[:, x] += choice[k, np.arange(band)]
        back[:, x] = np.arange(band) + k - 1
    path = np.zeros(w, dtype=np.int32)
    path[-1] = cost[:, -1].argmin()
    for x in range(w - 1, 0, -1):
        path[x - 1] = back[path[x], x]
    # Myk overgang på noen få piksler rundt veien.
    rows = np.arange(band)[:, None]
    t = np.clip((rows - path[None, :] + 2.0) / 4.0, 0.0, 1.0)[:, :, None]
    blend = bot * (1.0 - t) + top * t
    return np.concatenate([a[band:h - band], blend], axis=0)


def even(a, r=220.0):
    """Deler bort lysforskjellene større enn `r` piksler (glattet med omslag)."""
    h, w = a.shape[:2]
    lum = a.mean(axis=2)
    fy = np.fft.fftfreq(h)[:, None]
    fx = np.fft.fftfreq(w)[None, :]
    g = np.exp(-2.0 * (np.pi * r) ** 2 * (fx * fx + fy * fy))
    low = np.real(np.fft.ifft2(np.fft.fft2(lum) * g))
    return a * (lum.mean() / np.maximum(low, 1.0))[:, :, None]


def main():
    args = sys.argv[1:]
    band = int(args[args.index("--band") + 1]) if "--band" in args else 160
    a = np.asarray(Image.open(args[0]).convert("RGB")).astype(np.float32)
    if "--even" in args:
        a = even(a)
    only = args[args.index("--only") + 1] if "--only" in args else ""
    if only != "x":
        a = cut_rows(a, band)
    if only != "y":
        a = cut_rows(a.transpose(1, 0, 2), band).transpose(1, 0, 2)
    Image.fromarray(np.clip(a, 0, 255).round().astype(np.uint8)).save(args[1])
    print(f"{args[1]}: {a.shape[1]} × {a.shape[0]}")


if __name__ == "__main__":
    main()
