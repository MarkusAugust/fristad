"""Steg 4 i planen: en felles grunnkropp (mal) med kjent topologi, uv-kart, skjelett og vekter som tilpasses
forbildet, i stedet for en ny form fra bunnen for hver figur.

**Status: grensesnittet og første byggestein.** `Template` beskriver hva en mal må ha, `fit(template, views)`
er inngangen byggeløpet skal kalle i stedet for `fuse` + `mesh` når en mal er gitt, og `from_makehuman` leser
MakeHumans grunnmodell (`base.obj`, CC0) og forenkler den til en mal uten klær. Selve tilpasningen
(`fit`) er en enkel versjon: stiv innretting og skalering per akse mot silhuettene, så en glatt deformasjon
(Laplace-regularisert) som drar malens silhuettkanter mot forbildets. Det som gjenstår står i README under
«Malen».

Formater: malen er en `.npz` med `v` (hvile, meter, front mot +z, y opp), `f`, `uv` (per flatehjørne),
`weights` (hjørne × bein), `bones` (navn, hode, hale, forelder), `parts` (per flate: hud, hår, hånd, fot, klær...).
"""
import os

import numpy as np


class Template:
    def __init__(self, v, f, uv, weights=None, bones=None, parts=None, name="template"):
        self.v, self.f, self.uv = np.asarray(v, np.float32), np.asarray(f, np.int32), uv
        self.weights, self.bones, self.parts, self.name = weights, bones, parts, name

    @classmethod
    def load(cls, path):
        d = np.load(path, allow_pickle=True)
        return cls(d["v"], d["f"], d["uv"] if "uv" in d else None, d.get("weights"), d.get("bones"), d.get("parts"),
                   name=os.path.splitext(os.path.basename(path))[0])

    def save(self, path):
        np.savez(path, v=self.v, f=self.f, uv=self.uv if self.uv is not None else np.zeros((len(self.f), 3, 2), np.float32),
                 weights=self.weights if self.weights is not None else np.zeros((len(self.v), 0), np.float32),
                 bones=np.array(self.bones or [], dtype=object), parts=self.parts if self.parts is not None else np.zeros(len(self.f), np.int32))


def read_obj(path):
    """Enkel OBJ-leser (hjørner og flater, trianguleres)."""
    v, f = [], []
    for line in open(path):
        if line.startswith("v "):
            v.append([float(x) for x in line.split()[1:4]])
        elif line.startswith("f "):
            idx = [int(p.split("/")[0]) - 1 for p in line.split()[1:]]
            for i in range(1, len(idx) - 1):
                f.append([idx[0], idx[i], idx[i + 1]])
    return np.array(v, np.float32), np.array(f, np.int32)


def from_makehuman(obj_path, tris=8000, head_tris=3000):
    """MakeHumans grunnkropp (y opp, front mot +z, i desimeter) forenklet til en mal: kroppen til `tris`
    trekanter, hodet tettere (`head_tris`). Uten klær og uten vekter: de må legges på fra MakeHumans rigg
    (`rigs/standard`) i neste steg."""
    from .mesh import bpy_mesh, bpy_read, bpy_reset, bpy_select
    import bpy
    v, f = read_obj(obj_path)
    v = v * 0.1  # desimeter -> meter
    v[:, 1] -= v[:, 1].min()
    top = v[:, 1].max()
    head = v[f].mean(1)[:, 1] > top - 0.28
    out_v, out_f = [], []
    for sel, n in ((head, head_tris), (~head, tris - head_tris)):
        bpy_reset()
        ob = bpy_mesh("part", v, f[sel])
        bpy_select(ob)
        m = ob.modifiers.new("dec", "DECIMATE"); m.ratio = min(1.0, n / max(int(sel.sum()), 1))
        bpy.ops.object.modifier_apply(modifier="dec")
        pv, pf, _ = bpy_read(ob)
        out_f.append(pf + sum(len(x) for x in out_v)); out_v.append(pv)
    V = np.concatenate(out_v); F = np.concatenate(out_f)
    parts = np.concatenate([np.ones(len(out_f[0]), np.int32), np.zeros(len(out_f[1]), np.int32)])
    return Template(V, F, None, parts=parts, name="makehuman")


def fit(template, views, iterations=3, smooth=0.6):
    """Tilpass malen til silhuettene. `views`: liste av (maske (H×W bool), kamera-retning (cam, right, up), skala
    px/m, nullpunkt (c, top)). Første versjon:

    1. stiv innretting: malens omsluttende boks mot silhuettenes (høyde fra forfra, bredde fra forfra, dybde fra
       profilen), som skalering per akse;
    2. `iterations` runder der hvert hjørne på malens silhuettkant i en visning dras mot nærmeste punkt på
       forbildets kant i den visningen, med forskyvningene glattet over nettet (`smooth`).

    Gir en ny `Template` med samme topologi. Ingen dybdeterm ennå, ingen klær, ingen vekter."""
    from scipy import ndimage as ndi
    from scipy.sparse import coo_matrix
    V = template.v.copy()
    F = template.f
    # 1. skalering per akse mot boksene
    ext = {}
    for mask, (cam, right, up), s, (c, top) in views:
        ys, xs = np.nonzero(mask)
        w = (xs.max() - xs.min()) / s; h = (ys.max() - ys.min()) / s
        ax = int(np.argmax(np.abs(right)))
        ext[ax] = w
        ext[int(np.argmax(np.abs(up)))] = h
    for ax, size in ext.items():
        cur = np.ptp(V[:, ax])
        if cur > 1e-6:
            V[:, ax] = (V[:, ax] - V[:, ax].mean()) * (size / cur) + V[:, ax].mean()
    V[:, 1] -= V[:, 1].min()
    # nabolag for glatting
    n = len(V)
    A = coo_matrix((np.ones(len(F) * 3), (F.ravel(), np.roll(F, 1, 1).ravel())), shape=(n, n)).tocsr()
    A = A + A.T
    deg = np.asarray(A.sum(1)).ravel() + 1e-9
    # 2. kantene dras mot forbildets kanter
    for _ in range(iterations):
        D = np.zeros_like(V); Wt = np.zeros(n)
        for mask, (cam, right, up), s, (c, top) in views:
            cam, right, up = (np.asarray(x, float) for x in (cam, right, up))
            u = c + (V @ right) * s; vv = top - (V @ up) * s
            edge = mask ^ ndi.binary_erosion(mask)
            dist, (ri, ci) = ndi.distance_transform_edt(~edge, return_indices=True)
            ui = np.clip(np.round(u).astype(int), 0, mask.shape[1] - 1); vi = np.clip(np.round(vv).astype(int), 0, mask.shape[0] - 1)
            # malens silhuettkant i denne visningen: hjørner nær ytterkanten av projeksjonen
            proj = np.zeros(mask.shape, bool); proj[vi, ui] = True
            proj = ndi.binary_closing(proj, iterations=3)
            pedge = proj ^ ndi.binary_erosion(proj)
            on_edge = pedge[vi, ui]
            tu, tv = ci[vi, ui], ri[vi, ui]
            du = (tu - u) / s; dv = -(tv - vv) / s
            d3 = du[:, None] * right + dv[:, None] * up
            D[on_edge] += d3[on_edge]; Wt[on_edge] += 1
        D[Wt > 0] /= Wt[Wt > 0, None]
        for _ in range(8):  # spre forskyvningene over nettet
            D = (1 - smooth) * D + smooth * (A @ D) / deg[:, None]
        V = V + D
    return Template(V, F, template.uv, template.weights, template.bones, template.parts, name=template.name + "_fit")
