"""Avleder normalkart, høyde og ruhet fra et albedo-bilde (fra en
bildegenerator eller et foto), så et enkelt bilde blir et fullt teksturlag.

    python3 tools/textures/derive.py <lag> <bilde> [--size-m 2.0] [--rough 0.8] [--depth 1.0] [--invert]
        [--keystone 0.3] [--crop] [--seamless]

Skriver assets/textures/<lag>/albedo.jpg, normal.jpg, height.jpg,
rough.jpg og meta.json (meter per flis). Alt regnes med omslag i kantene,
så flisene møtes uten søm. Høyden er lysheten med de store variasjonene
trukket fra: fugene og sprekkene er mørke og blir lave. `--invert` snur det
for materialer der det mørke stikker opp. For bilder som ikke er rene
fliser: `--keystone K` retter opp et bilde tatt på skrå (øverste rad K
mindre enn nederste), `--crop` tar et kvadrat midt i bildet i stedet for å
strekke det, og `--seamless` blander inn en kopi forskjøvet en halv flis
langs kantene så bildet møtes uten søm. Lagnavnene er de i
`Layer::name` (cobble, plaster, oak, tiles, stone, cladding …)."""

import json
import os
import sys

import numpy as np
from PIL import Image

SIZE = 1024


def blur(a, r):
    """Gaussisk uskarphet med omslag i kantene (sirkulær, via FFT), så
    flisene møtes uten søm."""
    h, w = a.shape
    fy = np.fft.fftfreq(h)[:, None]
    fx = np.fft.fftfreq(w)[None, :]
    g = np.exp(-2.0 * (np.pi * r) ** 2 * (fx * fx + fy * fy))
    return np.real(np.fft.ifft2(np.fft.fft2(a) * g)).astype(np.float32)


def sample(a, xs, ys):
    """Bilineært oppslag i `a` (h, w, c) i punktene (xs, ys)."""
    h, w = a.shape[:2]
    xs = np.clip(xs, 0, w - 1.001)
    ys = np.clip(ys, 0, h - 1.001)
    x0, y0 = np.floor(xs).astype(int), np.floor(ys).astype(int)
    fx, fy = (xs - x0)[..., None], (ys - y0)[..., None]
    return (a[y0, x0] * (1 - fx) * (1 - fy) + a[y0, x0 + 1] * fx * (1 - fy) + a[y0 + 1, x0] * (1 - fx) * fy + a[y0 + 1, x0 + 1] * fx * fy)


def keystone(a, k):
    """Retter opp et bilde tatt på skrå: målestokken vokser lineært fra
    1 − k øverst til 1 nederst, så radene blir like høye (logaritmisk
    avbildning i høyden) og like brede (midten holdes)."""
    h, w = a.shape[:2]
    y = np.arange(h, dtype=np.float32)
    scale = (1 - k) + k * y / (h - 1)
    u = np.cumsum(1.0 / scale)
    u = (u - u[0]) / (u[-1] - u[0]) * (h - 1)
    # Utbildets rad → inngangsrad (u er stigende), og målestokken der.
    yi = np.interp(np.arange(h), u, y)
    si = (1 - k) + k * yi / (h - 1)
    ref = si.mean()
    xo, yo = np.meshgrid(np.arange(w, dtype=np.float32), np.arange(h, dtype=np.float32))
    cx = (w - 1) / 2
    xs = cx + (xo - cx) * (si[yo.astype(int)] / ref)[..., None][..., 0]
    ys = yi[yo.astype(int)]
    # Kutt bort kantene som mangler øverst.
    out = sample(a, xs, ys)
    margin = int(np.ceil((1 - (1 - k) / ref) * w / 2)) + 2
    return out[:, margin:w - margin]


def seamless(a, band=0.22):
    """Blander bildet med seg selv forskjøvet en halv flis, først på
    langs og så på tvers: kopien dekker kantene (der den møtes uten søm)
    og originalen midten (der kopiens søm går)."""
    for axis in (1, 0):
        n = a.shape[axis]
        t = np.abs(np.arange(n) - (n - 1) / 2) / ((n - 1) / 2)
        wgt = np.clip((1 - t) / band, 0, 1)
        wgt = wgt * wgt * (3 - 2 * wgt)
        wgt = wgt[None, :, None] if axis == 1 else wgt[:, None, None]
        a = a * wgt + np.roll(a, n // 2, axis=axis) * (1 - wgt)
    return a


def main():
    args = sys.argv[1:]
    layer, src = args[0], args[1]
    opt = lambda k, d: float(args[args.index(k) + 1]) if k in args else d
    size_m = opt("--size-m", 2.0)
    rough_base = opt("--rough", 0.8)
    depth = opt("--depth", 1.0)
    invert = "--invert" in args
    out = f"assets/textures/{layer}"
    os.makedirs(out, exist_ok=True)

    img = Image.open(src).convert("RGB")
    if "--keystone" in args or "--crop" in args or "--seamless" in args:
        a = np.asarray(img, dtype=np.float32)
        if "--keystone" in args:
            a = keystone(a, opt("--keystone", 0.3))
        if "--crop" in args:
            h, w = a.shape[:2]
            n = min(h, w)
            a = a[(h - n) // 2:(h - n) // 2 + n, (w - n) // 2:(w - n) // 2 + n]
        if "--seamless" in args:
            a = seamless(a)
        img = Image.fromarray(a.clip(0, 255).astype(np.uint8))
    img = img.resize((SIZE, SIZE), Image.LANCZOS)
    img.save(f"{out}/albedo.jpg", quality=90)
    rgb = np.asarray(img, dtype=np.float32) / 255.0
    lum = rgb @ np.array([0.299, 0.587, 0.114], dtype=np.float32)
    if invert:
        # Lyse fuger blir lave, men de dypeste sprekkene (mørkest i bildet)
        # ligger fortsatt lavest.
        shadow = np.clip((lum - 0.12) / 0.18, 0.0, 1.0)
        lum = (1.0 - lum) * shadow
    # Høyden: lysheten minus det store (belysning og fargeflekker), litt
    # myket opp, og strukket til 0..1.
    h = blur(lum, 2.0) - blur(lum, 40.0) * 0.85
    h = blur(h, 1.2)
    lo, hi = np.percentile(h, 1), np.percentile(h, 99)
    h = np.clip((h - lo) / max(hi - lo, 1e-4), 0, 1)
    Image.fromarray((h * 255).astype(np.uint8)).save(f"{out}/height.jpg", quality=92)

    # Normalen fra høyden (OpenGL-konvensjon, grønt opp), med omslag.
    strength = 6.0 * depth
    dx = (np.roll(h, -1, axis=1) - np.roll(h, 1, axis=1)) * 0.5
    dy = (np.roll(h, -1, axis=0) - np.roll(h, 1, axis=0)) * 0.5
    nx, ny, nz = -dx * strength, dy * strength, np.ones_like(h)
    length = np.sqrt(nx * nx + ny * ny + nz * nz)
    n = np.stack([nx / length, ny / length, nz / length], axis=-1)
    Image.fromarray(((n * 0.5 + 0.5) * 255).astype(np.uint8)).save(f"{out}/normal.jpg", quality=92)

    # Ruheten: grunnverdien, glattere på toppene som slites, ruere i fugene.
    r = np.clip(rough_base + (0.5 - h) * 0.2 + (lum - lum.mean()) * -0.1, 0.05, 1.0)
    Image.fromarray((r * 255).astype(np.uint8)).save(f"{out}/rough.jpg", quality=92)

    with open(f"{out}/meta.json", "w") as f:
        json.dump({"size_m": size_m, "source": os.path.basename(src)}, f)
    print(f"{out}: albedo, normal, høyde, ruhet ({size_m} m per flis)")


if __name__ == "__main__":
    main()
