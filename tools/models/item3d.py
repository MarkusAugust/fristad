"""Gjenstander fra en 3D-form (`assets/textures/items/3d/<kilde>.glb`, laget
av TRELLIS.2 fra eierens bilder), gjort spillklare som innbyggerne i
`folk3d.py`, med de samme stegene:

1. reise formen opp (TRELLIS ser mot −Y i Blender og er rundt 1 enhet høy),
   dreie den, ta bort det som ikke skal være med (`cut`: delene som beveger
   seg og er egne modeller), og skalere den til målet til modellen den
   erstatter (`height`, foten i z = 0, midt over origo);
2. et tett, lukket skall over formen (`folk3d.simplify`: tykkelse på de åpne
   flatene, voxel-skall, løse biter bort, forenklet til `tris`);
3. fargen fra den fulle formen bakt over på skallet (`folk3d.transfer`, uv-
   laget «src»), og eksport med `bake_textured(..., uv_angle=None, ao=False)`
   (teksturen fra formen har skyggene malt inn).

Målene (voxel og tykkelse) står i meter i `folk3d` og
gjelder en figur på 1,8 m; her settes de etter gjenstandens størrelse for
hver gjenstand og settes tilbake etterpå, så `folk3d` er uendret.

    python3.13 tools/models/item3d.py barrel [kit_rest ...]

Vurdert og ikke tatt i bruk (15. oktober): kista (to kister i én form, nesten
kubisk med høyt hvelvet lokk, ikke hul og uten eget lokk om hengslet
`crypt::CHEST_HINGE`), sverdet og øksa (grepet 4,6 og 5,5 cm, hånda holder
2,8–3,6, `arms.GRIP_D`; klinga 8 cm bred og 2,6 cm tykk), rundskjoldet
(håndtaket er en stropp i bulen, ikke `arms.HANDLE` bak brettet der neven
holder, og spilleren ser mest baksida), armene (åtte løse underarmer uten
skjelett) og treet (store flate blad på en rett stamme, og naturen er ikke
bygd ennå).
"""

import math
import os
import sys

import numpy as np

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from mathutils import Matrix  # noqa: E402
from common import bake_textured, clear  # noqa: E402
import folk3d  # noqa: E402

SRC = "assets/textures/items/3d"
SRC_SIZE = 2048  # fargen fra formen bakes først i denne størrelsen

# navn: kilde, utfil, høyde (m), dreining om z (grader), trekanter, voxel (m),
# teksturstørrelse, og bokser som tas bort og midten (i formens enheter: 1
# høy, foten i z = 0, midt i omrisset).
ITEMS = {
    # Tønna i byen (som `streetlife.barrel`: 0,86 m høy).
    "barrel": dict(src="barrel", out="assets/models/town/barrel.glb", height=0.86, yaw=0.0, tris=3000, voxel=0.006, size=1024),
    # Hvilestedet i krypten: sokkelen med lysestaken, lyset står på
    # toppen av stearinlyset der flammen (`kit_rest_flame`, 1,72 m) brenner.
    "kit_rest": dict(src="rest_shrine", out="assets/models/crypt/kit_rest.glb", wick=1.72, yaw=0.0, tris=4000, voxel=0.007, size=1024),
    # Brønnen: kummen, stolpene og taket. Vinda (med sveiven), tauet og
    # bøtta er egne modeller som økta dreier og heiser (`town::WELL_PARTS`),
    # så de tas bort her; vinda i formen står 0,565 oppe og skal sitte i
    # `town::WELL_AXLE` (1,55 m), og midten er midt mellom stolpene.
    "well": dict(src="well", out="assets/models/town/well.glb", height=1.55 / 0.565, center=(0.058, 0.0), yaw=0.0, tris=8000, voxel=0.012, size=2048,
                 cut=[((-0.12, -0.08, 0.49), (0.236, 0.08, 0.645)),  # valsen med tauet kveilet
                      ((0.035, -0.05, 0.15), (0.09, 0.05, 0.5)),  # tauet ned i sjakta
                      ((-0.4, -0.09, 0.42), (-0.198, 0.09, 0.64)),  # sveiven utenfor stolpen
                      ((0.314, -0.09, 0.5), (0.4, 0.09, 0.64))]),  # akselenden på den andre sida
}


def load(src, yaw):
    """Formen samlet i ett objekt uten foreldre, dreid `yaw` om z."""
    bpy.ops.import_scene.gltf(filepath=os.path.join(SRC, f"{src}.glb"))
    obs = [o for o in bpy.data.objects if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in obs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = obs[0]
    if len(obs) > 1:
        bpy.ops.object.join()
    ob = bpy.context.view_layer.objects.active
    mw = ob.matrix_world.copy()
    ob.parent = None
    ob.matrix_world = Matrix.Identity(4)
    ob.data.transform(Matrix.Rotation(math.radians(yaw), 4, "Z") @ mw)
    for o in [o for o in bpy.data.objects if o is not ob]:
        bpy.data.objects.remove(o)
    ob.name = "item"
    return ob


def coords(ob):
    co = np.empty(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    return co.reshape(-1, 3)


def place(ob, s, center=None):
    """Skalerer med `s` og setter foten i z = 0 midt over origo (midten av
    omrisset, eller `center` (x, y) i formens enheter)."""
    co = coords(ob)
    lo, hi = co.min(0), co.max(0)
    cx, cy = center if center is not None else ((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2)
    ob.data.transform(Matrix.Translation((0, 0, -lo[2] * s)) @ Matrix.Scale(s, 4) @ Matrix.Translation((-cx, -cy, 0)))


def cut(ob, boxes):
    """Tar bort flatene som har alle hjørnene inne i en av boksene
    ((x0, y0, z0), (x1, y1, z1))."""
    if not boxes:
        return
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    drop = []
    for f in bm.faces:
        for lo, hi in boxes:
            if all(all(lo[i] <= v.co[i] <= hi[i] for i in range(3)) for v in f.verts):
                drop.append(f)
                break
    print("tar bort", len(drop), "flater")
    bmesh.ops.delete(bm, geom=drop, context="FACES")
    bm.to_mesh(ob.data)
    bm.free()


def scale_of(ob, spec):
    """Skalaen fra formens enheter til meter."""
    co = coords(ob)
    h = co[:, 2].max() - co[:, 2].min()
    if "height" in spec:
        return spec["height"] / h
    if "wick" in spec:
        # Toppen av stearinlyset: det høyeste nær midten, under veken (den
        # tynne tråden øverst, de siste prosentene av høyden).
        z0 = co[:, 2].min()
        top = co[:, 2].max()
        mid = co[:, :2] - (co[:, :2].min(0) + co[:, :2].max(0)) / 2
        r = np.linalg.norm(mid, axis=1)
        # Bredden per høyde nær toppen: stearinlyset er bredt, veken smal.
        zs = np.linspace(top - 0.2 * h, top, 80)
        wide = [float(np.percentile(r[np.abs(co[:, 2] - z) < 0.002], 95)) if np.any(np.abs(co[:, 2] - z) < 0.002) else 0.0 for z in zs]
        body = max(wide)
        cand = [z for z, w in zip(zs, wide) if w > 0.5 * body]
        wz = max(cand) - z0
        print(f"toppen av lyset {wz / h:.3f} av høyden")
        return spec["wick"] / wz
    raise SystemExit("mangler mål")


def build(name):
    spec = ITEMS[name]
    clear()
    ob = load(spec["src"], spec.get("yaw", 0.0))
    # Først 1 enhet høy med foten i z = 0 midt over origo; bokser og
    # `center` står i disse enhetene.
    place(ob, 1.0 / float(np.ptp(coords(ob)[:, 2])))
    cut(ob, spec.get("cut"))
    s = scale_of(ob, spec)
    place(ob, s, spec.get("center"))
    co = coords(ob)
    print(name, "mål", (co.max(0) - co.min(0)).round(3))
    hi = ob.copy()
    hi.data = ob.data.copy()
    bpy.context.scene.collection.objects.link(hi)
    v = spec["voxel"]
    keep = {k: getattr(folk3d, k) for k in ("VOXEL", "THICK", "LOOSE")}
    try:
        folk3d.VOXEL = v
        folk3d.THICK = spec.get("thick", 1.2 * v)
        folk3d.LOOSE = spec.get("loose", folk3d.LOOSE)
        folk3d.simplify(ob, spec["tris"])
        folk3d.transfer(hi, ob, name, SRC_SIZE)
    finally:
        for k, val in keep.items():
            setattr(folk3d, k, val)
    ob.name = name
    tris = sum(len(p.vertices) - 2 for p in ob.data.polygons)
    print(name, "trekanter", tris)
    bake_textured(spec["out"], spec["size"], keep=("glass", "magic"), uv_angle=None, ao=False)


if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in ITEMS] or list(ITEMS)
    for n in names:
        build(n)
