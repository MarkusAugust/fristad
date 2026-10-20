"""Fiendene fra TRELLIS som ikke har to armer og to bein (`mob3d.py`,
`rig` i tabellen): delene henger stivt på hvert sitt bein, valgt etter
formen, og klippene er egne (som `skull_clips` og `leech_clips` i
`monsters.py`). Klippene har de samme navnene som fiendene ellers (idle,
walk, attack med treffet i bilde 15 av 30, alert, hit, hit2, stagger, die,
die2), og nakken og hodet står der spillet dreier dem mot deg.

- `spinner`: et steinhjul med jerntenner på høykant (navet mot deg) på fire
  korte bein. Hjulet dreier om navet mens beina tasser, og hullet i navet er
  munnen.
- `wisp`: Popover-irrbloset, en hvit kube med ett øye som svever, to små
  vinger, stikkeren under og tre glødende kuber bak (løse biter, eget bein
  som dreier sakte).
"""

import math

import numpy as np

import bpy  # noqa: F401
from mathutils import Vector  # noqa: F401

FPS = 24


def smooth01(u):
    u = min(1.0, max(0.0, u))
    return u * u * (3 - 2 * u)


def _clip(Clip, arm, name, length, fn):
    c = Clip(arm, name, length, FPS)
    for f in range(length + 1):
        fn(c, f, f / length)
    c.finish()


def _components(ob):
    """De sammenhengende bitene i skallet (merke per hjørne)."""
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components

    me = ob.data
    ed = np.empty(len(me.edges) * 2, np.int64)
    me.edges.foreach_get("vertices", ed)
    ed = ed.reshape(-1, 2)
    n = len(me.vertices)
    g = coo_matrix((np.ones(len(ed)), (ed[:, 0], ed[:, 1])), shape=(n, n))
    return connected_components(g, directed=False)[1]


def _geodesic(ob, co, seed):
    """Veien langs kantene i skallet fra frøet til hvert hjørne."""
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import dijkstra

    me = ob.data
    ed = np.empty(len(me.edges) * 2, np.int64)
    me.edges.foreach_get("vertices", ed)
    ed = ed.reshape(-1, 2)
    L = np.linalg.norm(co[ed[:, 0]] - co[ed[:, 1]], axis=1)
    n = len(co)
    G = coo_matrix((np.r_[L, L], (np.r_[ed[:, 0], ed[:, 1]], np.r_[ed[:, 1], ed[:, 0]])), shape=(n, n)).tocsr()
    if not seed.any():
        return np.full(n, np.inf)
    return dijkstra(G, indices=np.nonzero(seed)[0], min_only=True)


def _split(ob, labels):
    """Skallet deles langs grensen mellom delene (hver flate får delen de
    fleste hjørnene har), så en del som dreier (hjulet) ikke drar lange
    flater etter seg fra en som står (beina). Gir delen per hjørne etter
    delingen."""
    import bmesh
    from collections import Counter

    me = ob.data
    face = [Counter(labels[v] for v in p.vertices).most_common(1)[0][0] for p in me.polygons]
    bm = bmesh.new()
    bm.from_mesh(me)
    bm.faces.ensure_lookup_table()
    face = _islands(bm, face)
    tag = bm.faces.layers.int.new("part")
    names = sorted(set(face))
    for f in bm.faces:
        f[tag] = names.index(face[f.index])
    # Hver del løsnes fra resten (også der flater bare møtes i et hjørne).
    for k in range(1, len(names)):
        fs = [f for f in bm.faces if f[tag] == k]
        bmesh.ops.split(bm, geom=fs, use_only_faces=False)
    bm.verts.index_update()
    loose = [v for v in bm.verts if not v.link_faces]
    bmesh.ops.delete(bm, geom=loose, context="VERTS")
    bm.verts.index_update()
    out = np.array([""] * len(bm.verts), dtype=object)
    for f in bm.faces:
        for v in f.verts:
            out[v.index] = names[f[tag]]
    bm.to_mesh(me)
    bm.free()
    print("delt i", len(names), "deler")
    return out


# En bit av en del som henger sammen med færre flater enn dette (brøk av
# delen) går til delen rundt den: ellers sto små fliser av hjulet igjen ved
# beina (og av beina på hjulet) og stakk ut når hjulet spant.
ISLAND = 0.04


def _islands(bm, face):
    """Små øyer av en del (flater som henger sammen langs kanter) tas av
    delen som omgir dem flest kanter."""
    from collections import Counter

    face = list(face)
    for _ in range(3):
        seen = [False] * len(face)
        total = Counter(face)
        moved = 0
        for f0 in bm.faces:
            if seen[f0.index]:
                continue
            lab = face[f0.index]
            comp, stack = [], [f0]
            seen[f0.index] = True
            while stack:
                f = stack.pop()
                comp.append(f)
                for e in f.edges:
                    for g in e.link_faces:
                        if not seen[g.index] and face[g.index] == lab:
                            seen[g.index] = True
                            stack.append(g)
            if len(comp) >= ISLAND * total[lab]:
                continue
            ids = {f.index for f in comp}
            around = Counter(face[g.index] for f in comp for e in f.edges for g in e.link_faces if g.index not in ids)
            if around:
                new = around.most_common(1)[0][0]
                for f in comp:
                    face[f.index] = new
                moved += len(comp)
        print("småøyer flyttet:", moved, "flater")
        if not moved:
            break
    return face


def _assign(ob, arm, labels):
    """Hvert hjørne helt på beinet i `labels` (navn per hjørne); skallet
    deles først langs grensene (`_split`)."""
    labels = _split(ob, labels)
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.parent_set(type="ARMATURE_NAME")
    for name in sorted(set(labels)):
        g = ob.vertex_groups.get(name) or ob.vertex_groups.new(name=name)
        g.add([int(i) for i in np.nonzero(labels == name)[0]], 1.0, "REPLACE")
        print(f"{name}: {int((labels == name).sum())} hjørner")


# --- Spinneren ---------------------------------------------------------------


WHEEL_SLACK = 0.006  # luft rundt hjulets omdreide profil (brøk av høyden)


def spinner(ob, co, H, armature, Clip):
    """Hjulet på `chest`, som dreier om x (navet mot deg), og ett bein per
    hjørne."""
    # Hvert hjørne går til frøet som er nærmest langs skallet: hjulet (det
    # innerste av skiva rundt navet) eller foten i hvert hjørne (det nederste
    # utenfor tennene). Et skille i rommet ga lange flater over hjulskiva som
    # hang i beina og ble revet når hjulet dreide.
    hub = Vector((0.0, 0.0, 0.545 * H))
    r = np.hypot(co[:, 1], co[:, 2] - hub.z)
    # Hjulet: skiva rundt navet, tennene nederst mellom beina og ringen
    # over beina.
    seeds = {"chest": (r < 0.3 * H) | ((np.abs(co[:, 1]) < 0.12 * H) & (r < 0.47 * H)) | ((r < 0.47 * H) & (co[:, 2] > 0.25 * H))}
    corners = (("leg.FL", 1, 1), ("leg.FR", 1, -1), ("leg.BL", -1, 1), ("leg.BR", -1, -1))
    for nm, sx, sy in corners:
        seeds[nm] = (co[:, 2] < 0.08 * H) & (r > 0.47 * H) & (np.sign(co[:, 0]) == sx) & (np.sign(co[:, 1]) == sy)
    names = list(seeds)
    D = np.stack([_geodesic(ob, co, seeds[nm]) for nm in names], 1)
    labels = np.array(names, dtype=object)[D.argmin(1)]
    # Grensen mellom hjulet og beina er rommet hjulet fyller når det
    # dreier: profilen (største |x| per avstand fra navet) måles i øvre
    # halvdel, der det ikke er bein, og alt innenfor den omdreide profilen
    # er hjul, alt utenfor under navet bein (hjørnet i samme kvadrant).
    # Langs kantene ble grensen taggete, og fliser av hjulet sto igjen på
    # beina (og av beina på hjulet) og stakk ut når hjulet spant.
    step = 0.02 * H
    k = np.minimum((r / step).astype(int), 200)
    up = co[:, 2] > hub.z
    prof = np.full(201, -1.0)
    np.maximum.at(prof, k[up], np.abs(co[up, 0]))
    for i in range(1, 201):
        if prof[i] < 0 and prof[i - 1] >= 0 and (prof[i + 1:] >= 0).any():
            prof[i] = prof[i - 1]
    inside = np.abs(co[:, 0]) <= prof[k] + WHEEL_SLACK * H
    labels[inside] = "chest"
    for nm, sx, sy in corners:
        out = ~inside & ~up & (np.sign(co[:, 0]) == sx) & (np.sign(co[:, 1]) == sy)
        labels[out] = nm
    legs = {}
    for nm, sx, sy in corners:
        q = co[labels == nm]
        top = q[q[:, 2] > np.percentile(q[:, 2], 85)].mean(0)
        foot = q[q[:, 2] < np.percentile(q[:, 2], 15)].mean(0)
        legs[nm] = (Vector(top), Vector((foot[0], foot[1], 0.0)))
    B = [
        ("root", (0, 0, 0), (0, 0, 0.1 * H), None),
        ("hips", tuple(hub), tuple(hub + Vector((0, 0, 0.1 * H))), "root"),
        ("chest", tuple(hub), tuple(hub + Vector((0.15 * H, 0, 0))), "hips"),
        ("neck", tuple(hub), tuple(hub + Vector((0.05 * H, 0, 0))), "hips"),
        ("head", tuple(hub + Vector((0.05 * H, 0, 0))), tuple(hub + Vector((0.25 * H, 0, 0))), "neck"),
    ]
    for nm, (top, foot) in legs.items():
        B.append((nm, tuple(top), tuple(foot), "hips"))
    arm = armature("spinner_rig", B)
    _assign(ob, arm, labels)
    spinner_clips(arm, H, Clip)
    return arm, B


def spinner_clips(arm, H, Clip):
    diag = {"leg.FL": 0.0, "leg.BR": 0.0, "leg.FR": 0.5, "leg.BL": 0.5}

    def pose(c, f, wheel=0.0, root=(0, 0, 0), pitch=0.0, roll=0.0, legs=None, fall=(0.0, 0.0)):
        c.loc("root", f, root)
        c.rot("root", f, x=fall[0], y=fall[1])
        c.rot("hips", f, x=roll, y=pitch)
        # Hjulet dreier om navet (x i armaturrommet: navet peker fram).
        c.rot("chest", f, x=wheel)
        c.rot("neck", f)
        c.rot("head", f)
        for nm in diag:
            c.rot(nm, f, y=(legs or {}).get(nm, 0.0))

    def tap(t, amp, speed=1.0):
        return {nm: amp * math.sin(2 * math.pi * (t * speed + ph)) for nm, ph in diag.items()}

    _clip(Clip, arm, "idle", 48, lambda c, f, t: pose(c, f, wheel=12 * math.sin(2 * math.pi * t), root=(0, 0, 0.006 * H * abs(math.sin(2 * math.pi * t)))))
    # Gangen: én omdreining per syklus, beina tasser i diagonal.
    _clip(Clip, arm, "walk", 16, lambda c, f, t: pose(c, f, wheel=-360.0 * t, root=(0, 0, 0.02 * H * abs(math.sin(4 * math.pi * t))), pitch=4, legs=tap(t, 28)))

    def attack(c, f, t):
        # Bakover og ned mens hjulet spinner opp (0–11), fram med munnen
        # (navet) i treffet (15), og tilbake.
        spin = -720.0 * t * t
        if f <= 11:
            u = smooth01(f / 11)
            pose(c, f, wheel=spin, root=(-0.08 * H * u, 0, -0.03 * H * u), pitch=-10 * u, legs=tap(t, 10, 3))
        elif f <= 15:
            u = (f - 11) / 4
            pose(c, f, wheel=spin, root=(-0.08 * H + 0.48 * H * u, 0, 0.12 * H * math.sin(math.pi * u * 0.9)), pitch=-10 + 28 * u, legs=tap(t, 30, 3))
        else:
            u = smooth01((f - 15) / 15)
            pose(c, f, wheel=spin, root=(0.4 * H * (1 - u), 0, 0.01 * H * (1 - u)), pitch=18 * (1 - u))
    _clip(Clip, arm, "attack", 30, attack)
    _clip(Clip, arm, "alert", 22, lambda c, f, t: pose(c, f, wheel=-90 * smooth01(t), root=(0, 0, 0.12 * H * math.sin(math.pi * min(1.0, 1.5 * t)) * (1 if t < 0.67 else 0)), legs=tap(t, 20, 2)))
    for name, side in (("hit", 1.0), ("hit2", -1.0)):
        _clip(Clip, arm, name, 14, lambda c, f, t, side=side: pose(c, f, wheel=30 * math.exp(-4 * t) * side, root=(-0.1 * H * math.exp(-4 * t), 0, 0), roll=side * 16 * math.exp(-4 * t) * math.cos(10 * t), pitch=-8 * math.exp(-5 * t)))
    _clip(Clip, arm, "stagger", 30, lambda c, f, t: pose(c, f, wheel=20 * math.sin(4 * math.pi * t), roll=12 * math.sin(4 * math.pi * t) * math.sin(math.pi * t), legs=tap(t, 12, 2)))
    # Døden: velter bakover (die) eller framover på navet (die2) og blir
    # liggende flatt, beina i været.
    for name, sgn in (("die", -1.0), ("die2", 1.0)):
        def die(c, f, t, sgn=sgn):
            u = smooth01(min(1.0, t * 1.6)) ** 2
            pose(c, f, wheel=-60 * u, root=(0, 0, 0.12 * H * u), fall=(0.0, sgn * 90 * u), legs={nm: -30 * u for nm in diag})
        _clip(Clip, arm, name, 36, die)


# --- Popover-irrbloset -------------------------------------------------------


def wisp(ob, co, H, armature, Clip):
    """Kroppen på `head` (den ser mot deg), vingene på hvert sitt bein,
    stikkeren under (det under `STING`) og kubene (de løse bitene utenom den
    største) på `cubes`, som henger på hoftene og dreier sakte."""
    STING, WING = 0.38, 0.3
    lab = _components(ob)
    main = np.bincount(lab).argmax()
    labels = np.array(["head"] * len(co), dtype=object)
    labels[co[:, 2] < STING * H] = "stinger"
    side = (np.abs(co[:, 1]) > WING * H) & (co[:, 2] < 0.7 * H)
    labels[side & (co[:, 1] > 0)] = "wing.L"
    labels[side & (co[:, 1] < 0)] = "wing.R"
    loose = lab != main
    labels[loose] = "cubes"
    mid = Vector(co[loose].mean(0)) if loose.any() else Vector((0, -0.45 * H, 0.65 * H))
    print("kubene:", int(loose.sum()), "hjørner i", len(set(lab[loose])), "biter, midt i", tuple(round(v, 3) for v in mid))
    body = Vector((0.0, 0.0, 0.66 * H))
    B = [
        ("root", (0, 0, 0), (0, 0, 0.1 * H), None),
        ("hips", tuple(body), tuple(body + Vector((0, 0, 0.1 * H))), "root"),
        ("chest", tuple(body), tuple(body + Vector((0, 0, 0.15 * H))), "hips"),
        ("neck", tuple(body), tuple(body + Vector((0.05 * H, 0, 0))), "chest"),
        ("head", tuple(body + Vector((0.05 * H, 0, 0))), tuple(body + Vector((0.3 * H, 0, 0))), "neck"),
        ("wing.L", (0, WING * H, 0.55 * H), (0, 0.48 * H, 0.5 * H), "head"),
        ("wing.R", (0, -WING * H, 0.55 * H), (0, -0.48 * H, 0.5 * H), "head"),
        ("stinger", (0, 0, STING * H), (0, 0, 0.05 * H), "head"),
        ("cubes", tuple(mid), tuple(mid + Vector((0, 0, 0.1 * H))), "hips"),
    ]
    arm = armature("popover_wisp_rig", B)
    _assign(ob, arm, labels)
    wisp_clips(arm, H, Clip)
    return arm, B


def wisp_clips(arm, H, Clip):
    lift = 0.12 * H  # svever så høyt over gulvet

    def pose(c, f, t, root=(0, 0, 0), pitch=0.0, roll=0.0, flap=0.0, sting=0.0, cubes=0.0, cube_at=(0, 0, 0), fall=(0.0, 0.0), yaw=0.0):
        c.loc("root", f, (root[0], root[1], lift + root[2]))
        c.rot("root", f, x=fall[0], y=fall[1])
        c.rot("hips", f, x=roll, y=pitch, z=yaw)
        c.rot("chest", f)
        c.rot("neck", f)
        c.rot("head", f)
        c.rot("wing.L", f, x=-flap)
        c.rot("wing.R", f, x=flap)
        c.rot("stinger", f, y=sting)
        c.rot("cubes", f, z=cubes)
        c.loc("cubes", f, cube_at)

    def bob(t, k=1):
        return 0.03 * H * math.sin(2 * math.pi * k * t)

    _clip(Clip, arm, "idle", 48, lambda c, f, t: pose(c, f, t, root=(0, 0, bob(t)), flap=12 * math.sin(4 * math.pi * t), cubes=360 * t, cube_at=(0, 0, -bob(t) * 1.5)))
    _clip(Clip, arm, "walk", 24, lambda c, f, t: pose(c, f, t, root=(0, 0, bob(t, 2)), pitch=12, flap=22 * math.sin(6 * math.pi * t), cubes=360 * t, cube_at=(-0.06 * H, 0, -bob(t, 2))))

    def attack(c, f, t):
        # Reiser seg og trekker stikkeren bak (0–11), stuper fram og ned
        # med stikkeren først (treffet i 15), og svever tilbake.
        if f <= 11:
            u = smooth01(f / 11)
            pose(c, f, t, root=(-0.1 * H * u, 0, 0.15 * H * u), pitch=-22 * u, flap=30 * math.sin(8 * math.pi * t), sting=-25 * u, cubes=360 * t)
        elif f <= 15:
            u = (f - 11) / 4
            pose(c, f, t, root=(-0.1 * H + 0.5 * H * u, 0, 0.15 * H - 0.2 * H * u), pitch=-22 + 62 * u, flap=-25 * u, sting=-25 + 45 * u, cubes=360 * t)
        else:
            u = smooth01((f - 15) / 15)
            pose(c, f, t, root=(0.4 * H * (1 - u), 0, -0.05 * H * (1 - u)), pitch=40 * (1 - u), sting=20 * (1 - u), cubes=360 * t)
    _clip(Clip, arm, "attack", 30, attack)
    _clip(Clip, arm, "alert", 22, lambda c, f, t: pose(c, f, t, root=(0, 0, 0.1 * H * math.sin(math.pi * t)), pitch=-10 * math.sin(math.pi * t), flap=35 * math.sin(math.pi * t), cubes=180 * t))
    for name, side in (("hit", 1.0), ("hit2", -1.0)):
        _clip(Clip, arm, name, 14, lambda c, f, t, side=side: pose(c, f, t, root=(-0.12 * H * math.exp(-4 * t), 0, 0), roll=side * 22 * math.exp(-4 * t) * math.cos(10 * t), pitch=-12 * math.exp(-5 * t), flap=-20 * math.exp(-4 * t)))
    _clip(Clip, arm, "stagger", 30, lambda c, f, t: pose(c, f, t, root=(0, 0, -0.04 * H * math.sin(math.pi * t)), roll=14 * math.sin(4 * math.pi * t) * math.sin(math.pi * t), yaw=40 * math.sin(2 * math.pi * t), flap=-15, cubes=720 * t))
    # Døden: faller til gulvet og legger seg på sida (die) eller framover
    # (die2); kubene slokner ned på gulvet bak.
    for name, axis in (("die", 0), ("die2", 1)):
        def die(c, f, t, axis=axis):
            u = smooth01(min(1.0, t * 1.5)) ** 2
            fall = (85 * u, 0.0) if axis == 0 else (0.0, 85 * u)
            pose(c, f, t, root=(0, 0, -lift * u + 0.28 * H * u), fall=fall, flap=-30 * u, cube_at=(0, 0, -0.45 * H * u))
        _clip(Clip, arm, name, 36, die)


RIGS = {"spinner": spinner, "wisp": wisp}
