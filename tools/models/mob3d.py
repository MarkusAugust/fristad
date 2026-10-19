"""Fiendene fra en 3D-form (`assets/textures/monsters/3d/<navn>.glb`, laget
av TRELLIS.2 fra eierens bilde forfra, `docs/prompter/trellis-monstre.md`),
gjort spillklare med de samme stegene som innbyggerne i `folk3d.py`:

1. reise formen opp med fronten mot +X og skalere den til høyden i
   modellens målestokk (`height`: høyden i spillet delt på `Stats.scale`
   i `mob_kind.rs`), foten i z = 0 (`folk3d.load`);
2. et tett, lukket skall over formen (`folk3d.simplify`: tykkelse på de åpne
   flatene, voxel-skall, løse biter bort, forenklet til `tris`);
3. fargen fra den fulle formen hentet uendret over på skallet
   (`folk3d.transfer`/`nearest_colors`, uv-laget «src», pakket før fargen
   hentes); ingen ansiktslapp, teksturen brukes som den er;
4. leddene lest av formen (`landmarks`: høydene står i tabellen som brøk av
   høyden, sidene og dybden måles i formen), fiendeskjelettet
   (`BONES`, samme navn som i `monsters.py`), vektene fra Blender jevnet ut
   i rommet (`folk3d.layered_weights`) og klippene fra `mob_motion` med
   slagets stil (`STYLES`);
5. eksport med `bake_textured(..., uv_angle=None, ao=False, pack=False)`:
   en ren kopi av fargen (teksturen fra TRELLIS har skyggene malt inn).

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
# Skallet av en rustet kjempe har tusenvis av uv-øyer (rammene, kjettingene,
# sidene): med kanten skalert dekket de 8 % av teksturen, med en fast kant på
# 4 piksler i 2048 rundt 35 %.
PACK = dict(margin=0.002, margin_method="FRACTION")
SRC_SIZE = 2048  # fargen fra formen hentes i denne størrelsen (som teksturen)

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
    "framework": dict(height=4.0 / 2.3, tris=25000, voxel=0.006, thick=0.006, style="framework", attack="bite", extra=("slam", "roar"),
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


def build(name, size=2048):
    spec = MOBS[name]
    clear()
    ob = load(name, spec["height"])
    hi = ob.copy()
    hi.data = ob.data.copy()
    bpy.context.scene.collection.objects.link(hi)
    keep = {k: getattr(folk3d, k) for k in ("VOXEL", "THICK", "PACK")}
    try:
        folk3d.VOXEL = spec["voxel"]
        folk3d.THICK = spec["thick"]
        folk3d.PACK = PACK
        folk3d.simplify(ob, spec["tris"])
        folk3d.transfer(hi, ob, name, SRC_SIZE)
    finally:
        for k, v in keep.items():
            setattr(folk3d, k, v)
    ob.name = name
    J = landmarks(ob, spec)
    B = bones(J)
    arm = rig(ob, name, B)
    mob_motion.make_clips(Clip, arm, B, spec["style"], spec["attack"], spec.get("extra", ()))
    print(name, "trekanter", sum(len(p.vertices) - 2 for p in ob.data.polygons))
    bake_textured(f"{OUT}/{name}.glb", size, keep=("glass", "magic"), armature=arm, uv_angle=None, ao=False, pack=False)


if __name__ == "__main__":
    args = sys.argv[1:]
    size = next((int(a) for a in args if a.isdigit()), 2048)
    for n in [a for a in args if a in MOBS] or list(MOBS):
        build(n, size)
