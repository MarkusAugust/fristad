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
    python3.13 tools/models/mob3d.py --glow navbar   # bare glødeteksturen
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
import mob_rigid  # noqa: E402

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
                                  arm=0.32, leg=0.06, shoulder_in=0.08),
                      # Kjettingene fra håndleddene til livet deles der de er
                      # tynnest (`tethers`): halvdelen ved hånda følger
                      # håndleddet, halvdelen ved livet hoftene.
                      tether=[dict(seed=dict(x=(-0.08, -0.02), y=(0.24, 0.3), z=(0.37, 0.45)), within=dict(x=(-0.16, 0.06), y=(0.18, 0.46), z=(0.26, 0.5)),
                                   avoid=(("forearm", 0.055), ("hand", 0.05), ("upper_arm", 0.09)))],
                      glow=((300.0, 40.0), (345.0, 20.0))),
    # Navbaren (boss, nivå 8): 5 m i spillet med `scale` 2,5. En ridderkonge
    # i plate med kappe, krone og et åk over skuldrene (bjelken går ut
    # forbi hendene) med fem skjold i kjettinger; åket og skjoldene følger
    # brystet stivt (`rigid`), og bare de som står foran kappa måles som
    # bein (`leg_back`). Angrepet er neveslaget (`attack_fist`).
    # Bare de rosa sømmene og visiret gløder, ikke de fiolette skjoldene
    # (`glow_thin`: baksida av skjoldene er malt i samme farge, men er brede
    # flater).
    "navbar": dict(glow=((300.0, 20.0),), glow_thin=24, height=5.0 / 2.5, size=4096, tris=25000, voxel=0.006, thick=0.006, style="navbar", attack="fist", extra=("slam", "roar"),
                   joints=dict(hip=0.35, knee=0.19, ankle=0.06, chest=0.57, neck=0.77, head=0.83, shoulder=0.69, elbow=0.52, wrist=0.43, tip=0.31,
                               arm=0.27, leg=0.04, shoulder_in=0.15, leg_back=-0.08, leg_out=0.22),
                   # Bjelken med skjoldene (bak skuldrene, x under 0: ellers tok
                   # sonen med skulderplata), og midten av bjelken bak hjelmen.
                   rigid=[dict(bone="chest", seed=dict(z=(0.85, 0.93), y=(0.2, 1.0)), within=dict(x=(-1.0, 0.0), z=(0.57, 1.0), y=(0.12, 1.0)),
                               avoid=("upper_arm", "forearm", "hand", "head"), clear=0.07),
                          dict(bone="chest", seed=dict(x=(-1.0, -0.03), z=(0.86, 0.92), y=(0.0, 0.12)), within=dict(x=(-1.0, -0.015), z=(0.85, 0.93), y=(0.0, 0.2)),
                               blend=2),
                          # Kappa bak ryggen følger brystet, ikke armene (den
                          # løftet seg med armene i brølet).
                          dict(bone="chest", seed=dict(x=(-1.0, -0.12), z=(0.45, 0.7)), within=dict(x=(-1.0, -0.04), z=(0.4, 0.8)),
                               avoid=("upper_arm", "forearm", "hand", "head"), clear=0.05, blend=4)]),
    # Dropdownen (boss, nivå 6): 5 m. En steinkjempe med lange armer og et
    # V-formet hode; krystallene på skuldrene og ryggen følger brystet.
    # Armene er grodd inntil kroppen: brystet holdes samlet (`torso`).
    "dropdown": dict(torso=True, glow=((300.0, 40.0), (192.0, 20.0)), height=5.0 / 2.5, size=4096, tris=25000, voxel=0.006, thick=0.006, style="dropdown", attack="slam", extra=("roar",),
                     joints=dict(hip=0.43, knee=0.27, ankle=0.075, chest=0.67, neck=0.8, head=0.87, shoulder=0.75, elbow=0.5, wrist=0.33, tip=0.22,
                                 arm=0.26, leg=0.04, shoulder_in=0.14, leg_out=0.32),
                     rigid=[dict(bone="chest", seed=dict(z=(0.8, 1.0), y=(0.22, 1.0)), within=dict(z=(0.62, 1.0), y=(0.15, 1.0)),
                                 avoid=("upper_arm", "forearm", "hand", "head"), clear=0.06)]),
    # Tvillingvokteren (elite, nivå 9): 2,4 m med `ELITE_SCALE` 1,3. Står i
    # et skritt med armene nede langs kroppen (ikke A-stilling), halvt stein
    # (høyre) og halvt plast (venstre).
    "twin_warden": dict(glow_front=True, height=2.4 / 1.3, size=2048, tris=16000, voxel=0.005, thick=0.006, style="warden", attack="fist", extra=(),
                        joints=dict(hip=0.47, knee=0.25, ankle=0.06, chest=0.67, neck=0.85, head=0.9, shoulder=0.77, elbow=0.57, wrist=0.42, tip=0.3,
                                    arm=0.27, leg=0.03, shoulder_in=0.11, leg_out=0.25)),
    # Breadcrumb-bueskytteren (nivå 7–8): 1,6 m. Buen og koggeret på ryggen
    # følger brystet, ikke armene.
    "breadcrumb_archer": dict(height=1.6, size=2048, tris=14000, voxel=0.004, thick=0.005, style="archer", attack="shoot", extra=(),
                              joints=dict(hip=0.3, knee=0.17, ankle=0.04, chest=0.57, neck=0.75, head=0.82, shoulder=0.72, elbow=0.55, wrist=0.4, tip=0.3,
                                          arm=0.19, leg=0.02, shoulder_in=0.11, leg_out=0.2),
                              rigid=[dict(bone="chest", seed=dict(x=(-1.0, -0.09), z=(0.4, 1.0)), within=dict(x=(-1.0, -0.06), z=(0.4, 1.0)),
                                          avoid=("upper_arm", "forearm", "hand", "head"), clear=0.05)]),
    # Skeleton Loader (nivå 9–10): 1,8 m, et tynt skjelett av grå staver
    # (fint skall, så ribbeina og de to beina i underarmen står hver for
    # seg); klemma med seglet på brystet følger brystet.
    "skeleton_loader": dict(height=1.8, size=2048, tris=16000, voxel=0.003, thick=0.004, style="loader", attack="bite", extra=(),
                            joints=dict(hip=0.47, knee=0.25, ankle=0.06, chest=0.67, neck=0.8, head=0.86, shoulder=0.8, elbow=0.62, wrist=0.5, tip=0.4,
                                        arm=0.2, leg=0.04, shoulder_in=0.11, leg_out=0.2),
                            rigid=[dict(bone="chest", seed=dict(x=(0.085, 1.0), y=(0.03, 0.16), z=(0.66, 0.77)), within=dict(x=(0.05, 1.0), y=(0.0, 0.2), z=(0.62, 0.8)),
                                        avoid=(("upper_arm", 0.04), ("head", 0.06)), clear=0.04)]),
    # Grid-vokteren (nivå 9): 2,2 m, bygd av hule hvite celler.
    "grid_warden": dict(height=2.2, size=2048, tris=16000, voxel=0.005, thick=0.006, style="grid", attack="fist", extra=(),
                        joints=dict(hip=0.42, knee=0.22, ankle=0.05, chest=0.62, neck=0.8, head=0.88, shoulder=0.78, elbow=0.58, wrist=0.42, tip=0.32,
                                    arm=0.21, leg=0.03, shoulder_in=0.12, leg_out=0.2)),
    # Spinneren (nivå 5–6): 0,8 m, et steinhjul på høykant med navet mot
    # deg, på fire korte bein (`mob_rigid.spinner`).
    "spinner": dict(height=0.8, size=2048, tris=12000, voxel=0.003, thick=0.004, rig="spinner"),
    # Progress-formannen (elite, nivå 5): 2,2 m med `ELITE_SCALE` 1,3; armene
    # tett inntil kroppen, hakka i høyre neve (følger hånda) og kassen på
    # ryggen (følger brystet).
    # Glasstolpen i brystet og øynene er røde til rosa i teksturen (`glow`),
    # og brystet holdes samlet (`torso`).
    "progress_foreman": dict(torso=True, glow=((345.0, 25.0),), height=2.2 / 1.3, size=2048, tris=18000, voxel=0.005, thick=0.006, style="foreman", attack="slam", extra=("roar",),
                             joints=dict(hip=0.38, knee=0.2, ankle=0.05, chest=0.62, neck=0.8, head=0.86, shoulder=0.78, elbow=0.55, wrist=0.36, tip=0.25,
                                         arm=0.29, leg=0.04, shoulder_in=0.125, leg_out=0.28),
                             rigid=[dict(bone="hand.R", seed=dict(sy=(-1.0, -0.42), z=(0.0, 0.2)), within=dict(sy=(-1.0, -0.36), z=(0.0, 0.3)),
                                         avoid=("thigh", "shin", "foot"), clear=0.03, blend=2),
                                    dict(bone="chest", seed=dict(x=(-1.0, -0.14), y=(0.0, 0.2), z=(0.55, 1.0)), within=dict(x=(-1.0, -0.1), y=(0.0, 0.24), z=(0.5, 1.0)),
                                         avoid=("upper_arm", "forearm", "hand", "head"), clear=0.09)]),
    # Kortknekten (nivå 7–8): 1,5 m, kroppen er en tykk steinplate (kortet)
    # med hodet bak toppen; plata følger brystet stivt.
    "card_squire": dict(height=1.5, size=2048, tris=14000, voxel=0.004, thick=0.005, style="squire", attack="fist", extra=(),
                        joints=dict(hip=0.3, knee=0.17, ankle=0.05, chest=0.6, neck=0.8, head=0.88, shoulder=0.72, elbow=0.55, wrist=0.42, tip=0.33,
                                    arm=0.25, leg=0.04, shoulder_in=0.115, leg_out=0.24),
                        rigid=[dict(bone="chest", seed=dict(x=(0.07, 1.0), y=(0.0, 0.2), z=(0.4, 0.75)), within=dict(y=(0.0, 0.25), z=(0.2, 0.83)),
                                    avoid=(("upper_arm", 0.04), ("forearm", 0.04), ("hand", 0.04), ("thigh", 0.05), ("shin", 0.05), ("head", 0.06)), clear=0.04)]),
    # Popover-irrbloset (nivå 9): 0,9 m, svever (`mob_rigid.wisp`); de tre
    # kubene bak er løse biter som står (`loose`).
    "popover_wisp": dict(height=0.9, size=2048, tris=12000, voxel=0.003, thick=0.004, rig="wisp", loose=0.0002),
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
        # Store flate flater (kortknektens plate) har få hjørner: båndet
        # gjøres bredere til noe står i det.
        for band in (0.03, 0.06, 0.12, 0.25):
            m = (np.abs(co[:, 2] - z) < band * H) & (sg * co[:, 1] > lo) & (sg * co[:, 1] < hi)
            if m.sum() >= 3:
                break
        q = co[m]
        return Vector(((q[:, 0].min() + q[:, 0].max()) / 2, (q[:, 1].min() + q[:, 1].max()) / 2, z))

    # Kappa bak beina (Navbaren) står ikke i beinet: bare punktene foran
    # `leg_back` (brøk av høyden, x) og innenfor `leg_out` måles.
    back = j.get("leg_back", -9.0) * H
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
            keep = co
            co = co[co[:, 0] > back]
            J[f"{k}.{s}"] = mid(j[k] * H, j["leg"] * H, j.get("leg_out", j["arm"]) * H, sg)
            co = keep
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


def _inside(co, box, H):
    """Punktene i boksen (brøk av høyden): `x`, `z`, `y` (avstanden ut
    til sida, |y|) og `sy` (y med fortegn, venstre positiv), hver som
    (fra, til)."""
    m = np.ones(len(co), bool)
    for k, i, f in (("x", 0, lambda v: v), ("y", 1, np.abs), ("sy", 1, lambda v: v), ("z", 2, lambda v: v)):
        if k in box:
            lo, hi = box[k]
            m &= (f(co[:, i]) >= lo * H) & (f(co[:, i]) <= hi * H)
    return m


def _seg_dist(p, a, b):
    ab = b - a
    t = np.clip(((p - a) @ ab) / max(ab @ ab, 1e-12), 0.0, 1.0)
    return np.linalg.norm(p - (a + t[:, None] * ab), axis=1)


def limb_guard(ob, arm, H, seed_r=0.08, slack=1.5):
    """Vektene til en lem (armen eller beinet på hver side) bare der lemmen
    når langs skallet: varmen ga låret og magen vekter fra hånda som henger
    inntil, så låret fikk pigger og brystet skrumpet når armene ble løftet
    (Dropdownen). Frøet er hjørnene som står nærmest et bein i lemmen
    (innen `seed_r` av høyden); en vekt står der veien langs kantene fra
    frøet er høyst `slack` ganger avstanden i rommet (pluss litt), og
    tones ut over 0,1 av høyden etter det."""
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import dijkstra
    from scipy.spatial import cKDTree

    me = ob.data
    co = coords(ob)
    names = [b.name for b in arm.data.bones]
    segs = {b.name: (np.array(b.head_local), np.array(b.tail_local)) for b in arm.data.bones if b.name != "root"}
    order = list(segs)
    D = np.stack([_seg_dist(co, *segs[k]) for k in order], 1)
    near = np.array(order)[D.argmin(1)]
    dmin = D.min(1)
    ed = np.empty(len(me.edges) * 2, np.int64)
    me.edges.foreach_get("vertices", ed)
    ed = ed.reshape(-1, 2)
    L = np.linalg.norm(co[ed[:, 0]] - co[ed[:, 1]], axis=1)
    n = len(co)
    G = coo_matrix((np.r_[L, L], (np.r_[ed[:, 0], ed[:, 1]], np.r_[ed[:, 1], ed[:, 0]])), shape=(n, n)).tocsr()
    groups = {g.name: g for g in ob.vertex_groups}
    W = np.zeros((n, len(names)))
    gi = {g.index: names.index(g.name) for g in ob.vertex_groups if g.name in names}
    for v in me.vertices:
        for g in v.groups:
            if g.group in gi:
                W[v.index, gi[g.group]] = g.weight
    total = 0
    for s in ("L", "R"):
        for chain in (("upper_arm", "forearm", "hand"), ("thigh", "shin", "foot")):
            bs = [f"{c}.{s}" for c in chain if f"{c}.{s}" in names]
            cols = [names.index(b) for b in bs]
            seed = np.isin(near, bs) & (dmin < seed_r * H)
            if not seed.any():
                continue
            geo = dijkstra(G, indices=np.nonzero(seed)[0], min_only=True)
            euc, _ = cKDTree(co[seed]).query(co)
            k = np.clip(1.0 - (geo - slack * euc - 0.03 * H) / (0.1 * H), 0.0, 1.0)
            has = W[:, cols].sum(1) > 1e-4
            cut = has & (k < 1.0)
            total += int((has & (k == 0)).sum())
            W[np.ix_(cut, cols)] *= k[cut, None]
    # Det som ikke har vekt igjen, får beinet nærmest i rommet.
    empty = W.sum(1) < 1e-4
    for i in np.nonzero(empty)[0]:
        W[i, names.index(near[i])] = 1.0
    W /= W.sum(1, keepdims=True)
    for b, nm in enumerate(names):
        g = groups.get(nm) or ob.vertex_groups.new(name=nm)
        g.remove([int(i) for i in np.nonzero(W[:, b] <= 1e-3)[0]])
        for i in np.nonzero(W[:, b] > 1e-3)[0]:
            g.add([int(i)], float(W[i, b]), "REPLACE")
    print(f"lemmene: {total} hjørner mistet vekten fra en lem som ikke når dit langs skallet, {int(empty.sum())} fikk nærmeste bein")


def rigid_parts(ob, arm, spec):
    """Det som henger på kroppen og ikke er en lem (Navbarens åk med de fem
    skjoldene, krystallene på ryggen, buen og koggeret), følger ett bein
    stivt (`rigid`: ryggen eller brystet). Varmevektingen ga skjoldene
    armvekter, så de strakk seg med armene, som kjettingene på Rammeverket.
    Hver sone har frøet (`seed`, en boks i brøk av høyden), og derfra
    fylles det langs kantene i skallet så lenge punktet står i `within` og
    ikke nærmere enn `clear` (brøk av høyden) fra beina i `avoid` (armene,
    hodet): det som henger sammen med frøet uten å gå gjennom en lem. Så
    jevnes overgangen ut over `blend` kanter."""
    zones = spec.get("rigid", ())
    if not zones:
        return
    me = ob.data
    co = coords(ob)
    H = float(co[:, 2].max())
    ed = np.empty(len(me.edges) * 2, np.int64)
    me.edges.foreach_get("vertices", ed)
    ed = ed.reshape(-1, 2)
    from scipy.sparse import coo_matrix
    n = len(co)
    adj = coo_matrix((np.ones(len(ed) * 2), (np.r_[ed[:, 0], ed[:, 1]], np.r_[ed[:, 1], ed[:, 0]])), shape=(n, n)).tocsr()
    bones = {b.name: (np.array(b.head_local), np.array(b.tail_local)) for b in arm.data.bones}
    for zi, z in enumerate(zones):
        ok = _inside(co, z["within"], H)
        for b in z.get("avoid", ()):
            # Et bein alene bruker `clear`, (bein, avstand) sin egen.
            b, c = (b, z["clear"]) if isinstance(b, str) else b
            for nm in [x for x in bones if x == b or x.startswith(b + ".")]:
                ok &= _seg_dist(co, *bones[nm]) > c * H
        seed = _inside(co, z["seed"], H) & ok
        got = seed.copy()
        front = np.nonzero(seed)[0]
        while len(front):
            nb = np.unique(adj[front].indices)
            nb = nb[ok[nb] & ~got[nb]]
            got[nb] = True
            front = nb
        # Overgangen: `blend` kanter ut fra sonen tar vekten gradvis.
        k = np.where(got, 1.0, 0.0)
        ring = got.copy()
        steps = z.get("blend", 3)
        for i in range(steps):
            nb = np.unique(adj[np.nonzero(ring)[0]].indices)
            nb = nb[~ring[nb]]
            k[nb] = np.maximum(k[nb], 1.0 - (i + 1) / (steps + 1))
            ring[nb] = True
        bone = ob.vertex_groups.get(z["bone"]) or ob.vertex_groups.new(name=z["bone"])
        idx = np.nonzero(k > 0)[0]
        for i in idx:
            v = me.vertices[int(i)]
            w = float(k[i])
            for g in v.groups:
                g.weight *= 1.0 - w
            bone.add([int(i)], w + sum(g.weight for g in v.groups if g.group == bone.index), "REPLACE")
        print(f"stivt på {z['bone']}: {int(got.sum())} hjørner (frø {int(seed.sum())}), overgang {len(idx) - int(got.sum())}")
        if os.environ.get("MOB3D_DEBUG"):
            np.save(os.environ["MOB3D_DEBUG"] + f"_{zi}.npy", np.c_[co, k])


def _weights(ob, names):
    """Vektene som matrise (hjørne × bein i `names`)."""
    gi = {g.index: names.index(g.name) for g in ob.vertex_groups if g.name in names}
    W = np.zeros((len(ob.data.vertices), len(names)))
    for v in ob.data.vertices:
        for g in v.groups:
            if g.group in gi:
                W[v.index, gi[g.group]] = g.weight
    return W


def _set_weights(ob, names, W, rows):
    """Vektene i `rows` skrives tilbake (normalisert)."""
    W = W / np.maximum(W.sum(1, keepdims=True), 1e-9)
    for b, nm in enumerate(names):
        g = ob.vertex_groups.get(nm) or ob.vertex_groups.new(name=nm)
        g.remove([int(i) for i in rows if W[i, b] <= 1e-3])
        for i in rows:
            if W[i, b] > 1e-3:
                g.add([int(i)], float(W[i, b]), "REPLACE")


def _adjacency(ob, co):
    from scipy.sparse import coo_matrix

    ed = np.empty(len(ob.data.edges) * 2, np.int64)
    ob.data.edges.foreach_get("vertices", ed)
    ed = ed.reshape(-1, 2)
    L = np.linalg.norm(co[ed[:, 0]] - co[ed[:, 1]], axis=1)
    n = len(co)
    return coo_matrix((np.r_[L, L], (np.r_[ed[:, 0], ed[:, 1]], np.r_[ed[:, 1], ed[:, 0]])), shape=(n, n)).tocsr()


ARM_BONES = ("upper_arm", "forearm", "hand")
LEG_BONES = ("thigh", "shin", "foot")


def torso_guard(ob, arm, spec):
    """Kroppen holdes samlet når armene løftes og ryggen bøyes (Dropdownen,
    Progress-formannen): armene er grodd inntil kroppen i skallet, så
    varmen ga brystplatene armvekter, og platene som ligger lagvis fikk
    ulike andeler av hoftene og brystet, så brystet krøllet seg. I kroppen
    (innenfor skulderleddet minus `TORSO_MARGIN` til sida, mellom hoftene
    og halsen) tas armvektene bort (tonet ut over `TORSO_FADE` utover), og
    delingen mellom hoftene og brystet blir en jevn funksjon av høyden:
    hoftene ved hofteleddet, brystet ved brystleddet, jevnt imellom (en
    smal overgang krøllet platene der).
    Hode, nakke og bein beholder sine vekter. Bare for fiendene med
    `torso=True` i tabellen (Rammeverkets skulderplater går innenfor
    skulderleddet og ble dratt ut)."""
    if not spec.get("torso"):
        return
    names = [b.name for b in arm.data.bones]
    if "hips" not in names or "chest" not in names:
        return
    bones = {b.name: (np.array(b.head_local), np.array(b.tail_local)) for b in arm.data.bones}
    co = coords(ob)
    H = spec["height"]
    W = _weights(ob, names)
    hip_z, chest_z, neck_z = bones["hips"][0][2], bones["chest"][0][2], bones["neck"][0][2]
    sh = min(abs(bones["shoulder.L"][0][1]), abs(bones["shoulder.R"][0][1])) if "shoulder.L" in bones else abs(bones["upper_arm.L"][0][1])
    y_in = sh - TORSO_MARGIN * H
    ay = np.abs(co[:, 1])
    # Hvor mye punktet er kropp: 1 innenfor `y_in`, 0 ute ved `y_in + fade`,
    # og bare mellom hoftene og halsen.
    k = 1.0 - np.clip((ay - y_in) / (TORSO_FADE * H), 0.0, 1.0)
    k *= np.clip((co[:, 2] - (hip_z - 0.02 * H)) / (0.04 * H), 0.0, 1.0)
    k *= np.clip((neck_z + 0.02 * H - co[:, 2]) / (0.04 * H), 0.0, 1.0)
    arms = [i for i, n in enumerate(names) if n.split(".")[0] in ARM_BONES]
    hi, ci = names.index("hips"), names.index("chest")
    rows = np.nonzero(k > 0)[0]
    before = W[rows][:, arms].sum(1)
    # Armvektene ut.
    W[np.ix_(rows, arms)] *= (1.0 - k[rows])[:, None]
    # Hoftene og brystet etter høyden.
    u = np.clip((co[rows, 2] - hip_z) / max(chest_z - hip_z, 1e-6), 0.0, 1.0)
    u = u * u * (3 - 2 * u)
    spine = W[rows, hi] + W[rows, ci] + before * k[rows]
    kk = k[rows]
    W[rows, hi] = (1 - kk) * W[rows, hi] + kk * spine * (1 - u)
    W[rows, ci] = (1 - kk) * W[rows, ci] + kk * spine * u
    _set_weights(ob, names, W, rows)
    print(f"kroppen: {int((k > 0.99).sum())} hjørner (+{int(((k > 0) & (k <= 0.99)).sum())} i overgangen), armvekt tatt bort {float((before * k[rows]).sum()):.0f}")


TORSO_MARGIN = 0.02  # innenfor skulderleddet (brøk av høyden)
TORSO_FADE = 0.05  # armvekten tones inn over så langt utover


def tethers(ob, arm, spec):
    """Det som henger mellom en lem og kroppen (Rammeverkets kjettinger fra
    håndleddene til livet, `tether` i tabellen) deles i to der det er
    tynnest: halvdelen ved lemmen følger stedet den henger i på lemmen
    stivt, halvdelen ved kroppen stedet på kroppen, og skallet skilles
    langs kuttet. Med vekter fra begge ble kjettingen strukket som gummi
    når armene beveget seg, og en stiv sone på kroppen rev med seg armen.

    Sonen fylles langs kantene fra frøet (`seed`) innenfor `within` og
    unna beina i `avoid` (som `rigid_parts`). For hver sammenhengende bit
    (én kjetting per side) er ankrene hjørnene rett utenfor: de som mest
    følger en arm (`ARM_BONES`) eller et bein (`LEG_BONES`) og de som
    følger kroppen. Hvert hjørne i biten får brøken t = veien til lemmen
    delt på summen av veiene til begge, kuttet legges der færrest hjørner
    har samme t (det tynneste leddet, mellom 0,2 og 0,8), og hver halvdel
    tar snittet av vektene til ankrene den nådde."""
    import bmesh
    from scipy.sparse.csgraph import connected_components, dijkstra

    zones = spec.get("tether", ())
    if not zones:
        return
    names = [b.name for b in arm.data.bones]
    bones = {b.name: (np.array(b.head_local), np.array(b.tail_local)) for b in arm.data.bones}
    co = coords(ob)
    H = float(co[:, 2].max())
    G = _adjacency(ob, co)
    W = _weights(ob, names)
    W /= np.maximum(W.sum(1, keepdims=True), 1e-9)
    limb = np.array([n.split(".")[0] in ARM_BONES + LEG_BONES for n in names])
    labels = np.zeros(len(co), np.int64)  # 0 utenfor, ellers nummeret på halvdelen + 1
    halves = []
    for z in zones:
        ok = _inside(co, z["within"], H)
        for b in z.get("avoid", ()):
            b, c = (b, z["clear"]) if isinstance(b, str) else b
            for nm in [x for x in bones if x == b or x.startswith(b + ".")]:
                ok &= _seg_dist(co, *bones[nm]) > c * H
        got = _inside(co, z["seed"], H) & ok
        front = np.nonzero(got)[0]
        while len(front):
            nb = np.unique(G[front].indices)
            nb = nb[ok[nb] & ~got[nb]]
            got[nb] = True
            front = nb
        idx = np.nonzero(got)[0]
        k, lab = connected_components(G[idx][:, idx], directed=False)
        for c in range(k):
            C = idx[lab == c]
            if len(C) < 20:
                continue
            inC = np.zeros(len(co), bool)
            inC[C] = True
            N = np.unique(G[C].indices)
            N = N[~got[N]]
            dom = W[N].argmax(1)
            Na, Nb = N[limb[dom]], N[~limb[dom]]
            if not len(Na) or not len(Nb):
                print(f"tjoret: en bit på {len(C)} hjørner henger ikke mellom lem og kropp, står")
                continue
            sub = np.r_[C, N]
            Gs = G[sub][:, sub]
            pos = {int(v): i for i, v in enumerate(sub)}
            da, _, sa = dijkstra(Gs, indices=[pos[int(v)] for v in Na], min_only=True, return_predecessors=True)
            db, _, sb = dijkstra(Gs, indices=[pos[int(v)] for v in Nb], min_only=True, return_predecessors=True)
            m = len(C)
            da, db, sa, sb = da[:m], db[:m], sa[:m], sb[:m]
            fin = np.isfinite(da) & np.isfinite(db)
            t = np.where(fin, da / np.maximum(da + db, 1e-9), np.where(np.isfinite(da), 0.0, 1.0))
            bins = np.linspace(0.2, 0.8, 13)
            cnt = np.histogram(t[fin], bins)[0]
            cut = float((bins[cnt.argmin()] + bins[cnt.argmin() + 1]) / 2)
            near = t < cut
            for side, pick, src in ((0, near, sa), (1, ~near, sb)):
                s = src[pick]
                s = s[s >= 0]
                w = W[sub[s]].mean(0) if len(s) else W[C[pick]].mean(0)
                halves.append(w / w.sum())
                labels[C[pick]] = len(halves)
            top = names[int(halves[-2].argmax())], names[int(halves[-1].argmax())]
            print(f"tjoret: {len(C)} hjørner, kuttet ved t={cut:.2f} ({int(cnt.min())} hjørner der), {int(near.sum())} følger {top[0]}, {int((~near).sum())} følger {top[1]}")
    if not halves:
        return
    # Skallet skilles langs kuttet: flatene får halvdelen de fleste
    # hjørnene har, og kantene mellom to halvdeler deles.
    gidx = {nm: (ob.vertex_groups.get(nm) or ob.vertex_groups.new(name=nm)).index for nm in names}
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    bm.verts.ensure_lookup_table()
    tag = bm.faces.layers.int.new("tether")
    for f in bm.faces:
        ls = [int(labels[v.index]) for v in f.verts if labels[v.index]]
        f[tag] = max(set(ls), key=ls.count) if len(ls) * 2 > len(f.verts) else 0
    pair = lambda a, b: a and b and a != b and (a - 1) // 2 == (b - 1) // 2
    edges = [e for e in bm.edges if len(e.link_faces) == 2 and pair(e.link_faces[0][tag], e.link_faces[1][tag])]
    bmesh.ops.split_edges(bm, edges=edges)
    dl = bm.verts.layers.deform.verify()
    n = 0
    for v in bm.verts:
        ls = {f[tag] for f in v.link_faces if f[tag]}
        if len(ls) != 1:
            continue
        w = halves[ls.pop() - 1]
        d = v[dl]
        d.clear()
        for b, nm in enumerate(names):
            if w[b] > 1e-3:
                d[gidx[nm]] = float(w[b])
        n += 1
    bm.to_mesh(ob.data)
    bm.free()
    print(f"tjoret: {len(edges)} kanter delt, {n} hjørner følger halvdelen sin")



# Fargene som gløder (`glow`): (fargetone i grader, halv bredde); standard er
# magenta, sømmene og øynene fiendene har fra kongens rammeverk.
# TRELLIS maler dem fiolette til rosa (tone 280–340) og ofte mørke.
GLOW_HUES = ((300.0, 40.0),)
GLOW_SAT = (0.4, 0.6)  # metningen: ingen glød under, full over
GLOW_VAL = (0.25, 0.45)  # lysheten: ingen glød under, full over


def thin_glow(m, width):
    """Bare de tynne stripene og de små flekkene gløder (sømmene og
    øynene): en sammenhengende flekk i masken som er bredere enn `width`
    teksler (dobbelt avstand fra kanten til midten) tas bort. Navbarens
    skjold er malt fiolette på baksida i samme farge som sømmene, og
    lyste som store flater."""
    from scipy import ndimage

    on = m > 0.15
    lab, n = ndimage.label(on, np.ones((3, 3)))
    if not n:
        return m
    d = ndimage.distance_transform_edt(on)
    wide = np.asarray(ndimage.maximum(d, lab, np.arange(1, n + 1))) * 2 > width
    drop = ndimage.binary_dilation(wide[lab - 1] & on, iterations=2)
    print(f"glød: {int(wide.sum())} av {n} flekker er bredere enn {width:.0f} teksler og lyser ikke")
    return np.where(drop, 0.0, m)


REF = "assets/textures/monsters/ref"


def front_glow(j, bin_, name, hues, size):
    """Gløden fra eierens bilde forfra (`<navn>_front.jpg`), lagt på flatene
    som vender fram og står fremst: TRELLIS mistet de magenta sømmene og
    øynene på Tvillingvokteren (teksturen har dem ikke), men bildet TRELLIS
    ble laget av har dem. Bildet og formen passes sammen etter ytterkantene
    (høyden og midten), som `folk3d.face_patch`; hver teksel i glødekartet
    får punktet og normalen sin fra trekanten den ligger i."""
    from PIL import Image
    from scipy import ndimage

    import slim

    path = next((f"{REF}/{name}_front.{e}" for e in ("jpg", "jpeg", "png") if os.path.exists(f"{REF}/{name}_front.{e}")), None)
    if path is None:
        print("glød forfra: fant ikke bildet")
        return np.zeros((size, size, 3))
    ref = Image.open(path).convert("RGB")
    im = np.asarray(ref).astype(np.float32) / 255.0
    hsv = np.asarray(ref.convert("HSV")).astype(np.float32) / 255.0
    bg = np.median(np.concatenate([im[:8].reshape(-1, 3), im[-8:].reshape(-1, 3)]), 0)
    fg = np.linalg.norm(im - bg, axis=2) > 0.12
    rows, cols = np.nonzero(fg)
    h, sat, val = hsv[..., 0] * 360.0, hsv[..., 1], hsv[..., 2]

    def ramp(x, lo, hi):
        t = np.clip((x - lo) / (hi - lo), 0.0, 1.0)
        return t * t * (3 - 2 * t)

    m = np.zeros_like(h)
    for c, w in hues:
        d = np.abs((h - c + 180.0) % 360.0 - 180.0)
        m = np.maximum(m, 1.0 - ramp(d, w * 0.6, w))
    m *= ramp(sat, *GLOW_SAT) * ramp(val, *GLOW_VAL) * fg
    m = np.clip(ndimage.gaussian_filter(m, 1.5) * 1.4, 0, 1)
    col = im / np.maximum(im.max(-1, keepdims=True), 0.3) * 0.9
    # Formen fra glb-fila (y opp) i Blenders rom (z opp, y til venstre).
    prim = j["meshes"][0]["primitives"][0]
    P = np.array(slim.elements(j, bin_, prim["attributes"]["POSITION"]), np.float64)
    P = np.c_[P[:, 0], -P[:, 2], P[:, 1]]
    UV = np.array(slim.elements(j, bin_, prim["attributes"]["TEXCOORD_0"]), np.float64)
    F = np.array(slim.elements(j, bin_, prim["indices"]), np.int64).reshape(-1, 3)
    H = P[:, 2].max()
    sc = (rows.max() - rows.min()) / H
    cmid = (cols.min() + cols.max()) / 2 - (P[:, 1].min() + P[:, 1].max()) / 2 * sc
    out = np.zeros((size, size, 3), np.float32)
    samples = []  # (piksel i glødekartet, punkt, framvendthet)
    for f in F:
        a, b, c = P[f]
        n = np.cross(b - a, c - a)
        ln = np.linalg.norm(n)
        if ln < 1e-12 or n[0] / ln < 0.3:
            continue
        uv = UV[f] * size - 0.5
        lo = np.floor(uv.min(0)).astype(int)
        hi = np.ceil(uv.max(0)).astype(int)
        xs, ys = np.meshgrid(np.arange(lo[0], hi[0] + 1), np.arange(lo[1], hi[1] + 1))
        q = np.c_[xs.ravel(), ys.ravel()].astype(np.float64)
        e1, e2 = uv[1] - uv[0], uv[2] - uv[0]
        det = e1[0] * e2[1] - e1[1] * e2[0]
        if abs(det) < 1e-12:
            continue
        d = q - uv[0]
        l1 = (d[:, 0] * e2[1] - d[:, 1] * e2[0]) / det
        l2 = (e1[0] * d[:, 1] - e1[1] * d[:, 0]) / det
        ok = (l1 >= -0.02) & (l2 >= -0.02) & (l1 + l2 <= 1.02)
        ok &= (q[:, 0] >= 0) & (q[:, 1] >= 0) & (q[:, 0] < size) & (q[:, 1] < size)
        if not ok.any():
            continue
        p = a + l1[ok, None] * (b - a) + l2[ok, None] * (c - a)
        samples.append((q[ok].astype(int), p, np.full(ok.sum(), n[0] / ln)))
    if not samples:
        return out
    q = np.concatenate([s[0] for s in samples])
    p = np.concatenate([s[1] for s in samples])
    fx = np.concatenate([s[2] for s in samples])
    r = np.clip((rows.max() - p[:, 2] * sc).round().astype(int), 0, im.shape[0] - 1)
    cc = np.clip((cmid + p[:, 1] * sc).round().astype(int), 0, im.shape[1] - 1)
    # Bare det som står fremst (dybden per rute på 4 piksler i bildet).
    cell = (r // 4) * (im.shape[1] // 4 + 1) + cc // 4
    front = np.full(cell.max() + 1, -np.inf)
    np.maximum.at(front, cell, p[:, 0])
    seen = p[:, 0] >= front[cell] - 0.015 * H
    k = (m[r, cc] * ramp(fx, 0.3, 0.6) * seen)[:, None]
    np.maximum.at(out, (q[:, 1], q[:, 0]), (col[r, cc] * k).astype(np.float32))
    print(f"glød forfra: {float((out.max(-1) > 0.4).mean()):.2%} av glødekartet fra {os.path.basename(path)}")
    return out


def glow(path, hues=GLOW_HUES, thin=None, front=None):
    """De glødende flatene skilles ut etter fargen i teksturen og blir
    modellens glødetekstur (emissive), og materialet heter `….glow`, så det
    lyser hele døgnet (`roles.py`, `world.wgsl`). Masken regnes per teksel
    med myke overganger (fargetone, metning og lyshet), så kantene følger
    malingen og ikke trekantene."""
    import io
    from PIL import Image

    import slim

    j, bin_ = slim.read(path)
    mat = next(m for m in j["materials"] if m.get("name", "").split(".")[0] == "body")
    if "emissiveTexture" in mat:
        print("glød: har alt glødetekstur")
        return
    tex = mat["pbrMetallicRoughness"]["baseColorTexture"]["index"]
    img = j["images"][j["textures"][tex]["source"]]
    view = j["bufferViews"][img["bufferView"]]
    start = view.get("byteOffset", 0)
    im = Image.open(io.BytesIO(bin_[start : start + view["byteLength"]])).convert("RGB")
    im = im.resize((im.width // 2, im.height // 2), Image.LANCZOS)
    a = np.asarray(im).astype(np.float32) / 255.0
    hsv = np.asarray(im.convert("HSV")).astype(np.float32) / 255.0
    h, s, v = hsv[..., 0] * 360.0, hsv[..., 1], hsv[..., 2]

    def ramp(x, lo, hi):
        t = np.clip((x - lo) / (hi - lo), 0.0, 1.0)
        return t * t * (3 - 2 * t)

    m = np.zeros_like(h)
    for c, w in hues:
        d = np.abs((h - c + 180.0) % 360.0 - 180.0)
        m = np.maximum(m, 1.0 - ramp(d, w * 0.6, w))
    m *= ramp(s, *GLOW_SAT) * ramp(v, *GLOW_VAL)
    if thin:
        m = thin_glow(m, thin * im.width / 2048)
    # Gløden i malingens fargetone, men lys: fargen skaleres opp til den
    # sterkeste kanalen er nær hel (mørk fiolett blir lysende fiolett).
    bright = a / np.maximum(a.max(-1, keepdims=True), 0.3) * 0.9
    lit = bright * m[..., None]
    if front:
        lit = np.maximum(lit, front_glow(j, bin_, front, hues, im.width))
    out = Image.fromarray((np.clip(lit, 0, 1) * 255.0 + 0.5).astype(np.uint8))
    buf = io.BytesIO()
    out.save(buf, "JPEG", quality=88, subsampling=0)
    data = buf.getvalue()
    pad = b"\0" * (-len(bin_) % 4)
    j["bufferViews"].append({"buffer": 0, "byteOffset": len(bin_) + len(pad), "byteLength": len(data)})
    bin_ = bin_ + pad + data
    j["buffers"][0]["byteLength"] = len(bin_)
    j["images"].append({"bufferView": len(j["bufferViews"]) - 1, "mimeType": "image/jpeg", "name": "glow"})
    j["textures"].append({"source": len(j["images"]) - 1, "sampler": j["textures"][tex].get("sampler", 0)})
    mat["emissiveTexture"] = {"index": len(j["textures"]) - 1}
    mat["emissiveFactor"] = [1.0, 1.0, 1.0]
    mat["name"] = mat.get("name", "body").split(".")[0] + ".glow"
    slim.write(path, j, bin_)
    slim.tag(path)
    print(f"glød: {float((m > 0.5).mean()):.1%} av teksturen")


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


def shell(name, spec, size):
    """Skallet med fargen fra TRELLIS-formen (steg 1–3)."""
    clear()
    ob = load(name, spec["height"])
    hi = ob.copy()
    hi.data = ob.data.copy()
    bpy.context.scene.collection.objects.link(hi)
    keep = {k: getattr(folk3d, k) for k in ("VOXEL", "THICK", "LOOSE")}
    try:
        folk3d.VOXEL = spec["voxel"]
        folk3d.THICK = spec["thick"]
        # Små løse biter som hører til (kubene bak irrbloset) står.
        folk3d.LOOSE = spec.get("loose", folk3d.LOOSE)
        folk3d.simplify(ob, spec["tris"])
        # Fargen hentes i samme størrelse som teksturen (TRELLIS-fila har 4096).
        folk3d.transfer(hi, ob, name, size, unwrap=unwrap)
    finally:
        for k, v in keep.items():
            setattr(folk3d, k, v)
    ob.name = name
    return ob


def build(name, size=None):
    spec = MOBS[name]
    size = size or spec.get("size", 2048)
    # `MOB3D_CACHE=mappe`: skallet med fargen lagres etter overføringen (det
    # tar minutter), så vektene og klippene kan prøves på nytt raskt.
    cache = os.environ.get("MOB3D_CACHE")
    cpath = cache and os.path.abspath(f"{cache}/{name}.blend")
    if cpath and os.path.exists(cpath):
        bpy.ops.wm.open_mainfile(filepath=cpath)
        ob = bpy.data.objects[name]
    else:
        ob = shell(name, spec, size)
        if cpath:
            for o in [o for o in bpy.data.objects if o is not ob]:
                bpy.data.objects.remove(o)
            bpy.ops.outliner.orphans_purge(do_recursive=True)
            for img in bpy.data.images:
                if img.has_data and not img.packed_file:
                    img.pack()
            bpy.ops.wm.save_as_mainfile(filepath=cpath)
    if spec.get("rig"):
        # Uten to armer og to bein: delene stivt på egne bein og egne klipp.
        arm, B = mob_rigid.RIGS[spec["rig"]](ob, coords(ob), spec["height"], armature, Clip)
    else:
        J = landmarks(ob, spec)
        B = bones(J)
        arm = rig(ob, name, B)
        limb_guard(ob, arm, spec["height"])
        torso_guard(ob, arm, spec)
        rigid_parts(ob, arm, spec)
        tethers(ob, arm, spec)
        mob_motion.make_clips(Clip, arm, B, spec["style"], spec["attack"], spec.get("extra", ()))
    print(name, "trekanter", sum(len(p.vertices) - 2 for p in ob.data.polygons))
    bake_textured(f"{OUT}/{name}.glb", size, keep=("glass", "magic"), armature=arm, uv_angle=None, ao=False, pack=False, data_size=min(DATA_SIZE, size))
    glow(f"{OUT}/{name}.glb", spec.get("glow", GLOW_HUES), spec.get("glow_thin"), spec.get("glow_front") and name)


if __name__ == "__main__":
    args = sys.argv[1:]
    if args[:1] == ["--glow"]:
        # Bare glødeteksturen på nytt i en ferdig fil (uten å bygge den).
        for n in args[1:]:
            glow(f"{OUT}/{n}.glb", MOBS[n].get("glow", GLOW_HUES), MOBS[n].get("glow_thin"), MOBS[n].get("glow_front") and n)
        sys.exit(0)
    size = next((int(a) for a in args if a.isdigit()), None)
    for n in [a for a in args if a in MOBS] or list(MOBS):
        build(n, size)
