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
- `bat`: Alert-flaggermusa, en rund kropp med utropstegnet over hodet som
  henger stivt på hodet, vingene på hvert sitt bein og en kort hale.
  Spillet bestemmer flyhøyden (`mob_bat.rs`), så klippene henger på stedet.
- `beetle`: Badge-billen, kuppelskallet med hornet og øynene stivt på
  `chest` og fire korte bein, ett per hjørne (som spinneren). Egne klipp for
  stangeløpet (`charge_wind`, `charge`) og for å ligge velt på ryggen og
  sprelle (`flipped`, `mob_badge.rs`).
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


WING_CLEAR = 0.01  # vingene begynner så langt utenfor kubens side (brøk av høyden)


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
    # Vingene er det som står utenfor kubens side: sida måles over vingene
    # (eieren 20. oktober: «deler av kroppen følger med armen ut når han
    # beveger på den»; med en fast grense på 0,3 av høyden fulgte kanten av
    # kuben vingen).
    top = (lab == main) & (co[:, 2] > 0.72 * H) & (co[:, 2] < 0.95 * H)
    edge = {}
    for nm, sg in (("wing.L", 1), ("wing.R", -1)):
        q = sg * co[top, 1]
        q = q[q > 0]
        edge[nm] = max(WING * H, float(np.percentile(q, 99)) + WING_CLEAR * H) if len(q) else WING * H
        print(f"{nm} fra {edge[nm] / H:.3f} av høyden ut til sida")
        labels[(sg * co[:, 1] > edge[nm]) & (co[:, 2] < 0.7 * H)] = nm
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
        ("wing.L", (0, edge["wing.L"], 0.55 * H), (0, 0.48 * H, 0.5 * H), "head"),
        ("wing.R", (0, -edge["wing.R"], 0.55 * H), (0, -0.48 * H, 0.5 * H), "head"),
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


# --- Alert-flaggermusa -------------------------------------------------------


BAT_TAIL = 0.6  # halen kortes inn til så mye av lengden (en liten hale)
BAT_WING_CLEAR = 0.012  # vingen begynner så langt utenfor kroppens side (brøk av høyden)


def _short_tail(ob, co, H, x0):
    """Halen bak `x0` trekkes sammen langs lengden (`BAT_TAIL`): x mot
    roten og midtlinja (høyden per skive) like mye opp mot rotens høyde,
    så tykkelsen står og halen blir kort og tykk."""
    tail = (co[:, 0] < x0) & (np.abs(co[:, 1]) < 0.2 * H) & (co[:, 2] < 0.45 * H)
    if not tail.any():
        return co
    q = co[tail]
    step = 0.02 * H
    k = ((x0 - q[:, 0]) / step).astype(int)
    mid = np.array([q[k == i, 2].mean() if (k == i).any() else np.nan for i in range(k.max() + 1)])
    for i in range(1, len(mid)):
        if np.isnan(mid[i]):
            mid[i] = mid[i - 1]
    base = mid[0]
    new = q.copy()
    new[:, 0] = x0 + (q[:, 0] - x0) * BAT_TAIL
    # Midtlinja i den nye lengden: høyden der skiva kom fra, trukket mot roten.
    kn = np.minimum(((x0 - new[:, 0]) / step / BAT_TAIL).astype(int), len(mid) - 1)
    new[:, 2] = q[:, 2] - mid[k] + base + (mid[kn] - base) * BAT_TAIL
    co = co.copy()
    co[tail] = new
    ob.data.vertices.foreach_set("co", co.ravel())
    ob.data.update()
    print(f"halen: {int(tail.sum())} hjørner, kortet til {BAT_TAIL:.0%}")
    return co


def bat(ob, co, H, armature, Clip):
    """Kroppen, hodet og utropstegnet (løse biter over hodet) stivt på
    `head`, så hele flaggermusa ser mot deg; vingene (det som står utenfor
    kroppens side) på `wing.L`/`wing.R` med leddet ved skulderen, og halen
    (bak ryggen, lavt) på `tail`."""
    lab = _components(ob)
    main = np.bincount(lab).argmax()
    # Kroppens side måles under vingene (bunnen av bryst og buk), der
    # bare kroppen er; vingene står utenfor den.
    low = (lab == main) & (co[:, 2] > 0.12 * H) & (co[:, 2] < 0.42 * H) & (np.abs(co[:, 1]) < 0.4 * H)
    side = float(np.percentile(np.abs(co[low, 1]), 99.5))
    edge = side + BAT_WING_CLEAR * H
    print(f"kroppens side {side / H:.3f}, vingene fra {edge / H:.3f} av høyden")
    # Halen: bak ryggen, lavt og smalt.
    back = co[(lab == main) & (co[:, 2] > 0.3 * H) & (co[:, 2] < 0.6 * H) & (np.abs(co[:, 1]) < 0.1 * H), 0].min()
    x0 = back - 0.02 * H
    co = _short_tail(ob, co, H, x0)
    labels = np.array(["head"] * len(co), dtype=object)
    for nm, sg in (("wing.L", 1), ("wing.R", -1)):
        labels[(sg * co[:, 1] > edge) & (lab == main)] = nm
    tail = (co[:, 0] < x0) & (np.abs(co[:, 1]) < edge) & (co[:, 2] < 0.45 * H) & (lab == main)
    labels[tail] = "tail"
    # Vingeleddet: midt i vingeroten.
    shoulder = {}
    for nm, sg in (("wing.L", 1), ("wing.R", -1)):
        root = (labels == nm) & (sg * co[:, 1] < edge + 0.06 * H)
        p = co[root].mean(0) if root.any() else np.array([0.1 * H, sg * edge, 0.6 * H])
        tip = co[labels == nm]
        far = tip[np.argmax(sg * tip[:, 1])]
        shoulder[nm] = (Vector((p[0], sg * edge, p[2])), Vector(far))
    q = co[tail]
    t0 = Vector((x0, 0.0, float(q[q[:, 0] > x0 - 0.05 * H, 2].mean()) if len(q) else 0.25 * H))
    t1 = Vector(q[np.argmin(q[:, 0])]) if len(q) else t0 + Vector((-0.2 * H, 0, 0))
    body = Vector((float(co[lab == main, 0].mean()), 0.0, 0.45 * H))
    print("skuldrene:", {k: tuple(round(x / H, 3) for x in v[0]) for k, v in shoulder.items()}, "halen fra", tuple(round(x / H, 3) for x in t0), "til", tuple(round(x / H, 3) for x in t1))
    B = [
        ("root", (0, 0, 0), (0, 0, 0.1 * H), None),
        ("hips", tuple(body), tuple(body + Vector((0, 0, 0.1 * H))), "root"),
        ("chest", tuple(body), tuple(body + Vector((0, 0, 0.15 * H))), "hips"),
        ("neck", tuple(body), tuple(body + Vector((0.05 * H, 0, 0))), "chest"),
        ("head", tuple(body + Vector((0.05 * H, 0, 0))), tuple(body + Vector((0.4 * H, 0, 0))), "neck"),
        ("wing.L", tuple(shoulder["wing.L"][0]), tuple(shoulder["wing.L"][1]), "head"),
        ("wing.R", tuple(shoulder["wing.R"][0]), tuple(shoulder["wing.R"][1]), "head"),
        ("tail", tuple(t0), tuple(t1), "head"),
    ]
    arm = armature("alert_bat_rig", B)
    _assign(ob, arm, labels)
    bat_clips(arm, H, Clip)
    return arm, B


def bat_clips(arm, H, Clip):
    """Klippene henger på stedet: flyhøyden, stupet og fallet til gulvet
    styrer spillet (`mob_bat.rs`). `flap` er vingeslaget (opp er positivt),
    `sweep` legger vingene bakover langs kroppen (stupet)."""

    def pose(c, f, root=(0, 0, 0), pitch=0.0, roll=0.0, yaw=0.0, flap=0.0, sweep=0.0, tail=0.0, tail_side=0.0, fall=(0.0, 0.0)):
        c.loc("root", f, root)
        c.rot("root", f, x=fall[0], y=fall[1])
        c.rot("hips", f, x=roll, y=pitch, z=yaw)
        c.rot("chest", f)
        c.rot("neck", f)
        c.rot("head", f)
        c.rot("wing.L", f, x=flap, z=-sweep)
        c.rot("wing.R", f, x=-flap, z=sweep)
        c.rot("tail", f, y=tail, z=tail_side)

    def beat(t, n, amp=38.0, mid=8.0):
        # Slaget ned er raskere enn løftet (som en ekte flaggermus).
        ph = (n * t) % 1.0
        w = math.cos(math.pi * ph / 0.4) if ph < 0.4 else -math.cos(math.pi * (ph - 0.4) / 0.6)
        return mid + amp * w

    def bob(t, n, amp=0.04):
        # Kroppen løftes når vingene slår ned.
        return amp * H * math.sin(2 * math.pi * (n * t - 0.15))

    _clip(Clip, arm, "idle", 24, lambda c, f, t: pose(c, f, root=(0, 0, bob(t, 3)), pitch=-4, flap=beat(t, 3), tail=10 * math.sin(2 * math.pi * 3 * t - 1.0), tail_side=8 * math.sin(2 * math.pi * t)))
    _clip(Clip, arm, "walk", 16, lambda c, f, t: pose(c, f, root=(0, 0, bob(t, 3, 0.05)), pitch=16, flap=beat(t, 3, 44, 4), sweep=6, tail=-6 + 8 * math.sin(2 * math.pi * 3 * t - 1.0)))

    def alert(c, f, t):
        # Varselskriket: rygger opp med hodet bakover, vingene spent ut og
        # løftet, og hele kroppen dirrer mens den skriker (bilde 5–17).
        u = smooth01(min(1.0, f / 5)) * (1 - smooth01(max(0.0, (f - 17) / 5)))
        shake = math.sin(f * 2.6) * u
        pose(c, f, root=(-0.05 * H * u, 0, 0.05 * H * u), pitch=-26 * u + 3 * shake, roll=4 * shake, flap=(1 - u) * beat(t, 2) + u * (48 + 6 * shake), sweep=-10 * u, tail=-30 * u)
    _clip(Clip, arm, "alert", 22, alert)

    def attack(c, f, t):
        # Stupet (treffet i bilde 18 av 36): vingene opp og nesa opp (0–7),
        # vingene legges bakover langs kroppen og nesa ned (7–17), bittet
        # med hodet fram (18), vingene slår kraftig og den løfter seg igjen.
        if f <= 7:
            u = smooth01(f / 7)
            pose(c, f, pitch=-4 - 18 * u, flap=8 + 50 * u, tail=-20 * u)
        elif f <= 18:
            u = smooth01((f - 7) / 11)
            pose(c, f, pitch=-22 + 70 * u, flap=58 - 70 * u, sweep=55 * u, tail=-20 + 35 * u)
        else:
            u = smooth01((f - 18) / 18)
            snap = math.exp(-(f - 18) / 2.5)
            pose(c, f, pitch=48 * (1 - u) - 4 * u - 12 * snap, flap=(-12 * (1 - u)) * (1 - u) + u * beat(t, 3, 44, 8), sweep=55 * (1 - smooth01((f - 18) / 5)), tail=15 * (1 - u))
    _clip(Clip, arm, "attack", 36, attack)
    for name, side in (("hit", 1.0), ("hit2", -1.0)):
        _clip(Clip, arm, name, 14, lambda c, f, t, side=side: pose(c, f, root=(-0.1 * H * math.exp(-4 * t), 0, 0), roll=side * 28 * math.exp(-4 * t) * math.cos(10 * t), pitch=-14 * math.exp(-5 * t), flap=-30 * math.exp(-4 * t) + (1 - math.exp(-4 * t)) * beat(t, 2), tail=20 * math.exp(-4 * t)))
    # Svimmel (etter en parade ligger den på gulvet): spreller med vingene.
    _clip(Clip, arm, "stagger", 30, lambda c, f, t: pose(c, f, root=(0, 0, 0.02 * H * abs(math.sin(4 * math.pi * t))), roll=18 * math.sin(4 * math.pi * t), yaw=25 * math.sin(2 * math.pi * t), flap=-10 + 30 * math.sin(10 * math.pi * t), tail=20 * math.sin(6 * math.pi * t)))
    # Døden: vingene klapper sammen, og den velter på ryggen (die) eller
    # framover på snuten (die2) og blir liggende; spillet lar den falle.
    for name, sgn in (("die", -1.0), ("die2", 1.0)):
        def die(c, f, t, sgn=sgn):
            u = smooth01(min(1.0, t * 1.5))
            lift = (0.12 if sgn < 0 else 0.55) * H * u
            pose(c, f, root=(0, 0, lift), fall=(0.0, sgn * 88 * u), flap=(1 - u) * beat(t, 4) - 12 * u, sweep=20 * u, tail=(50 if sgn < 0 else -45) * u)
        _clip(Clip, arm, name, 36, die)


# --- Badge-billen -------------------------------------------------------------


BEETLE_FOOT = 0.07  # beina er det nederste (brøk av høyden) i hvert hjørne
BEETLE_BODY = 0.5  # kroppen er alt over så høyt
BEETLE_LEG_R = 0.2  # et bein er det innenfor så langt fra foten (vannrett)
BEETLE_TOP = 1.0  # velt på ryggen ligger toppen av skallet på gulvet


def beetle(ob, co, H, armature, Clip):
    """Skallet, hodet og hornet stivt på `chest`; hvert hjørne går til frøet
    nærmest langs skallet (kroppen øverst eller foten i hjørnet), som hos
    spinneren, så skallkanten over beina ikke følger beinet."""
    mid = co[:, :2].mean(0)
    seeds = {"chest": co[:, 2] > BEETLE_BODY * H}
    corners = (("leg.FL", 1, 1), ("leg.FR", 1, -1), ("leg.BL", -1, 1), ("leg.BR", -1, -1))
    for nm, sx, sy in corners:
        seeds[nm] = (co[:, 2] < BEETLE_FOOT * H) & (np.sign(co[:, 0] - mid[0]) == sx) & (np.sign(co[:, 1] - mid[1]) == sy)
        print(f"{nm}: frø {int(seeds[nm].sum())} hjørner")
    names = list(seeds)
    D = np.stack([_geodesic(ob, co, seeds[nm]) for nm in names], 1)
    labels = np.array(names, dtype=object)[D.argmin(1)]
    # Et bein er bare det som er lavt og nær foten: skallkanten over og
    # buken mellom beina går til kroppen (ellers dro beina tynne fliser av
    # buken med seg i gangen).
    labels[(labels != "chest") & (co[:, 2] > 0.3 * H)] = "chest"
    for nm, _, _ in corners:
        c = co[seeds[nm], :2].mean(0)
        labels[(labels == nm) & (np.hypot(co[:, 0] - c[0], co[:, 1] - c[1]) > BEETLE_LEG_R * H)] = "chest"
    legs = {}
    for nm, _, _ in corners:
        q = co[labels == nm]
        top = q[q[:, 2] > np.percentile(q[:, 2], 85)].mean(0)
        foot = q[q[:, 2] < np.percentile(q[:, 2], 15)].mean(0)
        legs[nm] = (Vector(top), Vector((foot[0], foot[1], 0.0)))
    body = Vector((float(mid[0]), 0.0, 0.45 * H))
    front = float(co[:, 0].max())
    B = [
        ("root", (0, 0, 0), (0, 0, 0.1 * H), None),
        ("hips", tuple(body), tuple(body + Vector((0, 0, 0.1 * H))), "root"),
        ("chest", tuple(body), tuple(body + Vector((0.3 * H, 0, 0))), "hips"),
        ("neck", (front * 0.6, 0, 0.35 * H), (front * 0.8, 0, 0.35 * H), "chest"),
        ("head", (front * 0.8, 0, 0.35 * H), (front, 0, 0.35 * H), "neck"),
    ]
    for nm, (top, foot) in legs.items():
        B.append((nm, tuple(top), tuple(foot), "hips"))
    arm = armature("badge_beetle_rig", B)
    _assign(ob, arm, labels)
    beetle_clips(arm, H, Clip)
    return arm, B


def beetle_clips(arm, H, Clip):
    diag = {"leg.FL": 0.0, "leg.BR": 0.0, "leg.FR": 0.5, "leg.BL": 0.5}

    def pose(c, f, root=(0, 0, 0), pitch=0.0, roll=0.0, yaw=0.0, legs=None, splay=None, flip=0.0):
        """`pitch` positiv er nesa ned; `legs` sving fram og bak per bein,
        `splay` ut til sida; `flip` velter den om lengdeaksen (grader)."""
        c.loc("root", f, root)
        c.rot("root", f, x=flip)
        c.rot("hips", f, x=roll, y=pitch, z=yaw)
        c.rot("chest", f)
        c.rot("neck", f)
        c.rot("head", f)
        for nm in diag:
            sg = 1.0 if nm.endswith("L") else -1.0
            c.rot(nm, f, x=sg * (splay or {}).get(nm, 0.0), y=(legs or {}).get(nm, 0.0))

    def tap(t, amp, speed=1.0):
        return {nm: amp * math.sin(2 * math.pi * (t * speed + ph)) for nm, ph in diag.items()}

    def bob(t, amp, speed=2.0):
        return (0, 0, amp * H * abs(math.sin(math.pi * t * speed)))

    _clip(Clip, arm, "idle", 48, lambda c, f, t: pose(c, f, root=(0, 0, 0.008 * H * math.sin(2 * math.pi * t)), pitch=2 * math.sin(2 * math.pi * t), legs=tap(t, 3)))
    # Gangen: fire korte bein i diagonal, skallet vugger.
    _clip(Clip, arm, "walk", 12, lambda c, f, t: pose(c, f, root=bob(t, 0.03), roll=4 * math.sin(2 * math.pi * t), legs=tap(t, 30)))

    # Støtet med hornet: rygger og senker nesa (0–10), støter fram og opp
    # (treffet i bilde 15) og trekker seg tilbake.
    def attack(c, f, t):
        if f <= 10:
            u = smooth01(f / 10)
            pose(c, f, root=(-0.08 * H * u, 0, -0.02 * H * u), pitch=14 * u, legs=tap(t, 8, 2))
        elif f <= 15:
            u = (f - 10) / 5
            pose(c, f, root=(-0.08 * H + 0.4 * H * u, 0, 0.04 * H * math.sin(math.pi * u)), pitch=14 - 30 * u, legs=tap(t, 30, 3))
        else:
            u = smooth01((f - 15) / 15)
            pose(c, f, root=(0.32 * H * (1 - u), 0, 0), pitch=-16 * (1 - u))
    _clip(Clip, arm, "attack", 30, attack)

    # Før stangeløpet: senker hornet, rygger og skraper med forbeina (14
    # bilder, `mob_badge.rs` WIND).
    def wind(c, f, t):
        u = smooth01(t * 1.6)
        scrape = {"leg.FL": -24 * u * math.sin(2 * math.pi * 3 * t), "leg.FR": 24 * u * math.sin(2 * math.pi * 3 * t)}
        pose(c, f, root=(-0.06 * H * u, 0, -0.03 * H * u), pitch=16 * u, legs=scrape)
    _clip(Clip, arm, "charge_wind", 14, wind)
    # Stangeløpet: nesa ned, beina går fort (løkke).
    _clip(Clip, arm, "charge", 8, lambda c, f, t: pose(c, f, root=bob(t, 0.05), pitch=16, roll=3 * math.sin(2 * math.pi * t), legs=tap(t, 40)))

    # Velt på ryggen etter å ha løpt i muren: ligger på skallet og spreller
    # med beina (løkke; spillet lar den ligge `FLIP_SECS`).
    def flipped(c, f, t):
        kick = {nm: 35 * math.sin(2 * math.pi * (2 * t + ph)) for nm, ph in diag.items()}
        pose(c, f, root=(0, 0, BEETLE_TOP * H), flip=180.0 + 8 * math.sin(2 * math.pi * t), legs=kick, splay={nm: 20.0 for nm in diag})
    _clip(Clip, arm, "flipped", 24, flipped)

    _clip(Clip, arm, "alert", 22, lambda c, f, t: pose(c, f, root=(0, 0, 0.1 * H * math.sin(math.pi * min(1.0, 1.6 * t)) * (1 if t < 0.62 else 0)), pitch=-10 * math.sin(math.pi * t), legs=tap(t, 16, 2)))
    for name, side in (("hit", 1.0), ("hit2", -1.0)):
        _clip(Clip, arm, name, 14, lambda c, f, t, side=side: pose(c, f, root=(-0.1 * H * math.exp(-4 * t), 0, 0), roll=side * 14 * math.exp(-4 * t) * math.cos(10 * t), pitch=-8 * math.exp(-5 * t)))
    _clip(Clip, arm, "stagger", 30, lambda c, f, t: pose(c, f, roll=10 * math.sin(4 * math.pi * t), yaw=8 * math.sin(2 * math.pi * t), legs=tap(t, 14, 2)))
    # Døden: velter på ryggen (die) eller synker sammen med beina ut (die2).
    def die(c, f, t):
        u = smooth01(min(1.0, t * 1.6)) ** 2
        pose(c, f, root=(0, 0, BEETLE_TOP * H * u), flip=180.0 * u, legs={nm: -20 * u for nm in diag}, splay={nm: 30 * u for nm in diag})
    _clip(Clip, arm, "die", 36, die)

    def die2(c, f, t):
        u = smooth01(min(1.0, t * 1.6)) ** 2
        pose(c, f, root=(0, 0, -0.12 * H * u), pitch=6 * u, splay={nm: 60 * u for nm in diag})
    _clip(Clip, arm, "die2", 36, die2)


RIGS = {"spinner": spinner, "wisp": wisp, "bat": bat, "beetle": beetle}
