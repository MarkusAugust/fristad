"""Fiendene fra en 3D-form (`assets/textures/monsters/3d/<navn>.glb`, laget
av TRELLIS.2 fra eierens bilde forfra, `docs/prompter/trellis-monstre.md`),
gjort spillklare med de samme stegene som innbyggerne i `folk3d.py`:

1. reise formen opp med fronten mot +X og skalere den til høyden i
   modellens målestokk (`height`: høyden i spillet delt på `Stats.scale`
   i `mob_kind.rs`), foten i z = 0 (`folk3d.load`);
2. et tett, lukket skall over formen (`folk3d.simplify`: tykkelse på de åpne
   flatene, voxel-skall, løse biter bort, forenklet til `tris`);
3. fargen fra den fulle formen hentet uendret over på skallet
   (`folk3d.transfer`/`nearest_colors`, uv-laget «src», lagt før fargen
   hentes); ingen ansiktslapp, teksturen brukes som den er. Øyene legges med
   xatlas (`unwrap`, `pip install xatlas`): Blenders smart_project ga
   3 500 småøyer som dekket 34 % av teksturen, xatlas gir rundt 1 900 og
   dekker nær 70 %, og hodet og brystet forfra får `DENSE` ganger så mange
   piksler per meter som resten (det man ser nært);
4. leddene lest av formen (`landmarks`: høydene står i tabellen som brøk av
   høyden, sidene og dybden måles i formen), fiendeskjelettet
   (`BONES`, samme navn som i `monsters.py`), vektene fra Blender jevnet ut
   i rommet (`folk3d.layered_weights`) og klippene fra `mob_motion` med
   slagets stil (`STYLES`);
5. eksport med `bake_textured(..., uv_angle=None, ao=False, pack=False)`:
   en ren kopi av fargen (teksturen fra TRELLIS har skyggene malt inn), i
   størrelsen `size` i tabellen (bossene 4096, de vanlige 2048; normal- og
   ruhetskartet 1024, formen er glatt).

    python3.13 tools/models/mob3d.py framework [størrelse]
"""

import os
import sys

import numpy as np

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from mathutils import Vector  # noqa: E402
from common import Clip, armature, bake_textured, clear  # noqa: E402
import folk3d  # noqa: E402
import mob_motion  # noqa: E402

SRC = "assets/textures/monsters/3d"
OUT = "assets/models/mobs"
# Kanten rundt hver øy i piksler i den ferdige teksturen (resten fylles med
# fargen fra kanten, `nearest_colors`, så mipmapene ikke trekker inn sort).
PADDING = 2
# Hodet (over `DENSE_FROM` av høyden) og brystet forfra (normalen fram,
# mellom hoftene og halsen): så mange ganger flere piksler per meter.
DENSE = (1.6, 1.4)
DENSE_FROM = 0.76
DATA_SIZE = 1024  # normal- og ruhetskartet (et glatt skall, samme ruhet)

# navn: høyden i modellens målestokk, trekanter, voxel og tykkelse (m i
# modellen), stilen og angrepet i `mob_motion`, ekstra klipp, og leddene
# (høyden som brøk av `height`; `arm` er grensen ut til sida der armen
# begynner, `leg` der beinet slutter innover).
MOBS = {
    # Rammeverket (sluttbossen, nivå 10): 4 m i spillet med `scale` 2,3. En
    # bred, tung jernkjempe med korte bein (skrittet i 36 % av høyden),
    # armene hengende nesten rett ned med nevene i hoftehøyde, et lite hode
    # med krone over skulderplatene. Angrepet er bittet (`Stats.attack_hit`
    # er bittets), slaget i bakken og brølet er egne klipp.
    "framework": dict(height=4.0 / 2.3, size=4096, tris=25000, voxel=0.006, thick=0.006, style="framework", attack="bite", extra=("slam", "roar"),
                      joints=dict(hip=0.36, knee=0.2, ankle=0.07, chest=0.52, neck=0.74, head=0.82, shoulder=0.62, elbow=0.45, wrist=0.31, tip=0.16,
                                  arm=0.32, leg=0.06, shoulder_in=0.08)),
}


def coords(ob):
    co = np.empty(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    return co.reshape(-1, 3)


def load(name, height):
    """`folk3d.load` med fiendens mappe og høyde."""
    keep = (folk3d.SRC, folk3d.FOLK)
    try:
        folk3d.SRC = SRC
        folk3d.FOLK = {**folk3d.FOLK, name: height}
        return folk3d.load(name)
    finally:
        folk3d.SRC, folk3d.FOLK = keep


def landmarks(ob, spec):
    """Leddene (x fram, y mot venstre, z opp): høydene fra tabellen, og midt
    i armen og beinet målt i formen i den høyden (midten mellom ytterkantene
    i hver retning)."""
    co = coords(ob)
    H = spec["height"]
    j = spec["joints"]

    def mid(z, lo, hi, sg):
        """Midten av det som står i høyden `z` (±3 % av høyden) mellom `lo`
        og `hi` ut til sida `sg`."""
        m = (np.abs(co[:, 2] - z) < 0.03 * H) & (sg * co[:, 1] > lo) & (sg * co[:, 1] < hi)
        q = co[m]
        return Vector(((q[:, 0].min() + q[:, 0].max()) / 2, (q[:, 1].min() + q[:, 1].max()) / 2, z))

    J = {}
    for k in ("hip", "chest", "neck", "head"):
        z = j[k] * H
        J[k] = mid(z, -0.15 * H, 0.15 * H, 1)
        J[k].y = 0.0
    J["top"] = Vector((J["head"].x, 0.0, float(co[:, 2].max())))
    for s, sg in (("L", 1), ("R", -1)):
        arm = j["arm"] * H
        for k in ("elbow", "wrist", "tip"):
            J[f"{k}.{s}"] = mid(j[k] * H, arm, 2 * H, sg)
        # Skulderen står litt innenfor armen, i skulderplatene.
        e = J[f"elbow.{s}"]
        J[f"shoulder.{s}"] = Vector((J["chest"].x, e.y - sg * j["shoulder_in"] * H, j["shoulder"] * H))
        for k in ("knee", "ankle"):
            J[f"{k}.{s}"] = mid(j[k] * H, j["leg"] * H, arm, sg)
        J[f"hip.{s}"] = Vector((J["hip"].x, J[f"knee.{s}"].y * 0.9, j["hip"] * H))
        a = J[f"ankle.{s}"]
        foot = (co[:, 2] < 0.03 * H) & (np.abs(co[:, 1] - a.y) < 0.08 * H)
        J[f"toe.{s}"] = Vector((float(co[foot, 0].max()), a.y, 0.0))
    for k, v in sorted(J.items()):
        print(f"{k:12s}", tuple(round(c, 3) for c in v))
    return J


def bones(J):
    """Fiendeskjelettet (som `MODAL_BONES` i `monsters.py`, uten kjeve og
    hale): hodebeinet peker fram, så spillet dreier det mot deg
    (`Mob.look`)."""
    hip, chest, neck, head = J["hip"], J["chest"], J["neck"], J["head"]
    fwd = Vector((0.2 * (J["top"].z - head.z), 0.0, 0.0))
    out = [
        ("root", (0, 0, 0), (0, 0, 0.2), None),
        ("hips", tuple(hip), tuple(chest), "root"),
        ("chest", tuple(chest), tuple(neck), "hips"),
        ("neck", tuple(neck), tuple(head), "chest"),
        ("head", tuple(head), tuple(head + fwd), "neck"),
    ]
    for s in ("L", "R"):
        out += [
            (f"upper_arm.{s}", tuple(J[f"shoulder.{s}"]), tuple(J[f"elbow.{s}"]), "chest"),
            (f"forearm.{s}", tuple(J[f"elbow.{s}"]), tuple(J[f"wrist.{s}"]), f"upper_arm.{s}"),
            (f"hand.{s}", tuple(J[f"wrist.{s}"]), tuple(J[f"tip.{s}"]), f"forearm.{s}"),
            (f"thigh.{s}", tuple(J[f"hip.{s}"]), tuple(J[f"knee.{s}"]), "hips"),
            (f"shin.{s}", tuple(J[f"knee.{s}"]), tuple(J[f"ankle.{s}"]), f"thigh.{s}"),
            (f"foot.{s}", tuple(J[f"ankle.{s}"]), tuple(J[f"toe.{s}"]), f"shin.{s}"),
        ]
    return out


def rig(ob, name, B):
    """Skjelettet med vektene fra Blender (varme), jevnet ut i rommet så
    platene som ligger oppå hverandre følger samme bein."""
    arm = armature(f"{name}_rig", B)
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    bare = sum(1 for v in ob.data.vertices if not len(v.groups))
    print("uten vekt etter varmen:", bare, "av", len(ob.data.vertices))
    if bare > 0.2 * len(ob.data.vertices):
        for g in list(ob.vertex_groups):
            ob.vertex_groups.remove(g)
        bpy.ops.object.parent_set(type="ARMATURE_ENVELOPE")
    folk3d.layered_weights(ob, arm)
    return arm


def unwrap(ob, size):
    """Øyene med xatlas: først delt og foldet ut etter formen, så skaleres
    hodet og brystet forfra (`DENSE`), og øyene pakkes tett på nytt med
    rotasjon (brute force, et par minutter). Skallet trianguleres først, så
    hver flate er én trekant i samme rekkefølge som i xatlas."""
    import bmesh
    import xatlas
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components

    bm = bmesh.new()
    bm.from_mesh(ob.data)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    bm.to_mesh(ob.data)
    bm.free()
    me = ob.data
    v = coords(ob).astype(np.float32)
    f = np.array([p.vertices[:] for p in me.polygons], np.uint32)
    pack = xatlas.PackOptions()
    pack.resolution = size
    pack.padding = PADDING
    pack.bilinear = True
    pack.rotate_charts = True
    at = xatlas.Atlas()
    at.add_mesh(v, f)
    at.generate(xatlas.ChartOptions(), pack)
    vm, ix, uv = at[0]
    assert (vm[ix] == f).all(), "xatlas byttet om på hjørnene"
    # Øyene er de sammenhengende bitene i uv (hjørnene er delt i sømmene).
    n = len(uv)
    g = coo_matrix((np.ones(ix.size), (ix.ravel(), ix[:, [1, 2, 0]].ravel())), shape=(n, n))
    k, lab = connected_components(g, directed=False)
    p = v[f]
    c = p.mean(1)
    nrm = np.cross(p[:, 1] - p[:, 0], p[:, 2] - p[:, 0])
    nrm /= np.linalg.norm(nrm, axis=1, keepdims=True) + 1e-12
    H = float(v[:, 2].max())
    w = np.ones(len(f))
    w[(c[:, 2] > 0.40 * H) & (c[:, 2] <= DENSE_FROM * H) & (nrm[:, 0] > 0.3)] = DENSE[1]
    w[c[:, 2] > DENSE_FROM * H] = DENSE[0]
    fl = lab[ix[:, 0]]
    cw = np.bincount(fl, w, k) / np.bincount(fl, minlength=k)
    uv = uv.copy()
    for i in np.nonzero(np.abs(cw - 1) > 1e-3)[0]:
        m = lab == i
        mid = uv[m].mean(0)
        uv[m] = mid + (uv[m] - mid) * cw[i]
    pack.bruteForce = True
    at = xatlas.Atlas()
    at.add_uv_mesh(uv.astype(np.float32), ix.astype(np.uint32))
    at.generate(xatlas.ChartOptions(), pack)
    vm2, ix2, uv2 = at[0]
    assert (vm2[ix2] == ix).all(), "pakkingen byttet om på hjørnene"
    t = uv2[ix2]
    e1, e2 = t[:, 1] - t[:, 0], t[:, 2] - t[:, 0]
    print(f"xatlas: {k} øyer, dekning {np.abs(e1[:, 0] * e2[:, 1] - e1[:, 1] * e2[:, 0]).sum() / 2:.1%}")
    flat = uv2[ix2].reshape(-1, 2).astype(np.float32)
    # Løkkene i hver trekant står i samme rekkefølge som hjørnene.
    loops = np.array([l for p in me.polygons for l in p.loop_indices])
    out = np.zeros((len(me.loops), 2), np.float32)
    out[loops] = flat
    me.uv_layers["src"].data.foreach_set("uv", out.ravel())


def build(name, size=None):
    spec = MOBS[name]
    clear()
    ob = load(name, spec["height"])
    hi = ob.copy()
    hi.data = ob.data.copy()
    bpy.context.scene.collection.objects.link(hi)
    size = size or spec.get("size", 2048)
    keep = {k: getattr(folk3d, k) for k in ("VOXEL", "THICK")}
    try:
        folk3d.VOXEL = spec["voxel"]
        folk3d.THICK = spec["thick"]
        folk3d.simplify(ob, spec["tris"])
        # Fargen hentes i samme størrelse som teksturen (TRELLIS-fila har 4096).
        folk3d.transfer(hi, ob, name, size, unwrap=unwrap)
    finally:
        for k, v in keep.items():
            setattr(folk3d, k, v)
    ob.name = name
    J = landmarks(ob, spec)
    B = bones(J)
    arm = rig(ob, name, B)
    mob_motion.make_clips(Clip, arm, B, spec["style"], spec["attack"], spec.get("extra", ()))
    print(name, "trekanter", sum(len(p.vertices) - 2 for p in ob.data.polygons))
    bake_textured(f"{OUT}/{name}.glb", size, keep=("glass", "magic"), armature=arm, uv_angle=None, ao=False, pack=False, data_size=min(DATA_SIZE, size))


if __name__ == "__main__":
    args = sys.argv[1:]
    size = next((int(a) for a in args if a.isdigit()), None)
    for n in [a for a in args if a in MOBS] or list(MOBS):
        build(n, size)
