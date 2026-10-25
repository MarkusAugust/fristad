"""Fiendene fra TRELLIS som er en kjede av ledd (`mob3d.py`, `rig` i
tabellen): kroppen deles langs lengden i segmenter som hvert har sitt bein,
og vektene glir jevnt over fra det ene beinet til det neste ved hver fuge,
så kjeden bøyer seg uten at skallet rives (et stivt kutt ga sprekker i
fugene når kroppen bølget). Klippene har de samme navnene som fiendene
ellers (idle, walk, attack, alert, hit, hit2, stagger, die, die2), og
angrepene som ingen andre har, egne klipp:

- `combobox`: Den grusomme Comboboxen, 3 m lang og mannshøy. Hodet (feltet
  av beinhvitt tre med pila), halsen, seks valgplater av skifer hengslet
  sammen, og halen med kjettingen og jernklossen; tre beinpar under
  platene. Angrepet er pisken (`attack`: lista slås rundt lavt langs
  gulvet, hopp over den; `lash_high`: den reiser seg og slår rundt i
  brysthøyde, dukk under), så velger den deg (`choose`: platene foran
  skrangler og gløder) og hamrer platene ned (`strike`, dobbel skade).
- `worm`: Divider-ormen, 1,2 m lang og ankelhøy, med en ryggrad av åtte
  ledd som bølger fra side til side når den kryper. Angrepet er å grave seg
  ned (`burrow`), gå under gulvet og dukke opp ved deg (`emerge`, bittet).
"""

import math

import numpy as np

import bpy  # noqa: F401
from mathutils import Vector

FPS = 24


def smooth01(u):
    u = min(1.0, max(0.0, u))
    return u * u * (3 - 2 * u)


def _clip(Clip, arm, name, length, fn):
    c = Clip(arm, name, length, FPS)
    for f in range(length + 1):
        fn(c, f, f / length)
    c.finish()


def _weights(ob, arm, W, names):
    """Vektene (hjørne × bein) skrives som grupper, og skallet henges på
    skjelettet (uten Blenders varme)."""
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.parent_set(type="ARMATURE_NAME")
    W = W / np.maximum(W.sum(1, keepdims=True), 1e-9)
    for b, nm in enumerate(names):
        g = ob.vertex_groups.get(nm) or ob.vertex_groups.new(name=nm)
        idx = np.nonzero(W[:, b] > 1e-3)[0]
        for i in idx:
            g.add([int(i)], float(W[i, b]), "REPLACE")
        print(f"{nm}: {len(idx)} hjørner ({int((W[:, b] > 0.99).sum())} helt)")


def _along(s, joints, blend):
    """Vektene langs kjeden: `s` er stedet langs kroppen per hjørne, og
    `joints` fugene mellom segmentene (stigende). Segment k går fra fuge k-1
    til fuge k; over hver fuge glir vekten fra det ene til det neste over
    `blend` (halv bredde, samme mål som `s`)."""
    n = len(joints) + 1
    W = np.zeros((len(s), n))
    # Hvor mye punktet har kommet forbi hver fuge (0 før, 1 etter).
    past = np.stack([np.clip((s - (j - blend)) / (2 * blend), 0.0, 1.0) for j in joints], 1)
    past = past * past * (3 - 2 * past)
    for k in range(n):
        before = past[:, k - 1] if k > 0 else np.ones(len(s))
        after = past[:, k] if k < n - 1 else np.zeros(len(s))
        W[:, k] = np.clip(before - after, 0.0, 1.0)
    return W


# --- Comboboxen --------------------------------------------------------------

# Fugene langs kroppen (x i brøk av høyden, fronten mot +x): halen,
# platene bak, platene midt, platene foran, halsen og hodet.
CB_JOINTS = (-0.6, -0.36, -0.12, 0.14, 0.41, 0.53)
CB_SEGS = ("tail", "seg.2", "seg.1", "hips", "chest", "neck", "head")
CB_BLEND = 0.035  # fugene glir over så mye (brøk av høyden)
CB_LEG_TOP = 0.2  # beina under så høyt (brøk av høyden), overgangen over
CB_LEG_FADE = 0.04
# Beinparene (x i brøk av høyden): bak, midt og foran, og beinet de henger i.
CB_LEGS = (("B", -0.32, "seg.1"), ("M", 0.0, "hips"), ("F", 0.32, "chest"))
CB_LEG_X = (0.1, 0.13)  # beinet så langt fra midten av paret, tonet ut til


def combobox(ob, co, H, armature, Clip):
    x, y, z = co.T
    names = list(CB_SEGS)
    W = _along(x / H, CB_JOINTS, CB_BLEND)
    # Beina: det under `CB_LEG_TOP` og ikke i halen, til beinparet nærmest
    # langs x og sida.
    legs = []
    leg = (z < (CB_LEG_TOP + CB_LEG_FADE) * H) & (x > CB_JOINTS[0] * H)
    k = np.clip(((CB_LEG_TOP + CB_LEG_FADE) * H - z) / (2 * CB_LEG_FADE * H), 0.0, 1.0)
    k = k * k * (3 - 2 * k) * leg
    dx = np.stack([np.abs(x - lx * H) for _, lx, _ in CB_LEGS], 1)
    near = np.argmin(dx, 1)
    # Bare under beinparet (ikke platekantene mellom dem).
    w = np.clip((CB_LEG_X[1] * H - dx.min(1)) / ((CB_LEG_X[1] - CB_LEG_X[0]) * H), 0.0, 1.0)
    k = k * w * w * (3 - 2 * w)
    LW = np.zeros((len(co), 6))
    for i, (tag, lx, parent) in enumerate(CB_LEGS):
        for j, (side, sg) in enumerate((("L", 1.0), ("R", -1.0))):
            m = (near == i) & (np.sign(y) == sg)
            col = 2 * i + j
            LW[m, col] = k[m]
            q = co[leg & m & (z < CB_LEG_TOP * H)]
            top = Vector((float(q[:, 0].mean()), float(q[:, 1].mean()), CB_LEG_TOP * H)) if len(q) else Vector((lx * H, sg * 0.25 * H, CB_LEG_TOP * H))
            foot = q[q[:, 2] < np.percentile(q[:, 2], 10)].mean(0) if len(q) else np.array([lx * H, sg * 0.25 * H, 0.0])
            legs.append((f"leg.{tag}.{side}", top, Vector((float(foot[0]), float(foot[1]), 0.0)), parent))
    W = np.c_[W * (1.0 - LW.sum(1, keepdims=True)), LW]
    names += [n for n, _, _, _ in legs]

    def mid(a, b):
        m = (x > a * H) & (x < b * H) & (z > CB_LEG_TOP * H)
        return float(np.median(z[m])) if m.any() else 0.6 * H

    J = [CB_JOINTS[0] - 0.3] + list(CB_JOINTS) + [0.8]
    P = [Vector((J[i] * H, 0.0, mid(J[max(i - 1, 0)], J[min(i + 1, len(J) - 1)]))) for i in range(len(J))]
    # P[i] er fugen mellom segment i-1 og i; hoftene (segment 3) er roten i
    # kjeden, og beina peker ut fra den: fram mot hodet og bak mot halen.
    hips_a, hips_b = P[3], P[4]
    B = [
        ("root", (0, 0, 0), (0, 0, 0.1 * H), None),
        ("hips", tuple((hips_a + hips_b) / 2), tuple(hips_b), "root"),
        ("chest", tuple(P[4]), tuple(P[5]), "hips"),
        ("neck", tuple(P[5]), tuple(P[6]), "chest"),
        ("head", tuple(P[6]), tuple(P[7]), "neck"),
        ("seg.1", tuple(P[3]), tuple(P[2]), "hips"),
        ("seg.2", tuple(P[2]), tuple(P[1]), "seg.1"),
        ("tail", tuple(P[1]), tuple(P[0]), "seg.2"),
    ]
    for n, top, foot, parent in legs:
        B.append((n, tuple(top), tuple(foot), parent))
    arm = armature("cruel_combobox_rig", B)
    _weights(ob, arm, W, names)
    combobox_clips(arm, H, Clip)
    return arm, B


LEG_NAMES = [f"leg.{t}.{s}" for t in ("B", "M", "F") for s in ("L", "R")]
# Trefotsgangen: venstre foran og bak med høyre i midten, og omvendt.
TRIPOD = {"leg.F.L": 0.0, "leg.B.L": 0.0, "leg.M.R": 0.0, "leg.F.R": 0.5, "leg.B.R": 0.5, "leg.M.L": 0.5}


def combobox_clips(arm, H, Clip):
    def pose(c, f, root=(0, 0, 0), yaw=0.0, pitch=0.0, roll=0.0, body=None, legs=None, lift=None):
        """`body`: dreininger per segment (navn → (x, y, z)); `legs`:
        sving fram og bak per bein (grader), `lift`: løft ut til sida."""
        c.loc("root", f, root)
        c.rot("root", f, x=roll, y=pitch, z=yaw)
        body = body or {}
        for nm in CB_SEGS:
            rx, ry, rz = body.get(nm, (0.0, 0.0, 0.0))
            c.rot(nm, f, x=rx, y=ry, z=rz)
        for nm in LEG_NAMES:
            sg = 1.0 if nm.endswith("L") else -1.0
            c.rot(nm, f, x=sg * (lift or {}).get(nm, 0.0), y=(legs or {}).get(nm, 0.0))

    def gait(t, amp, k=1):
        sw = {nm: amp * math.sin(2 * math.pi * (k * t + ph)) for nm, ph in TRIPOD.items()}
        # Foten løftes mens beinet svinger fram (cos > 0).
        up = {nm: 14.0 * max(0.0, math.cos(2 * math.pi * (k * t + ph))) for nm, ph in TRIPOD.items()}
        return sw, up

    def wave(t, amp, k=1):
        """Platene bølger svakt fra side til side, bakover langs kroppen."""
        return {nm: (0.0, 0.0, amp * math.sin(2 * math.pi * (k * t - 0.15 * i))) for i, nm in enumerate(("chest", "seg.1", "seg.2", "tail"))}

    # Hvilen: rykker digitalt (holdt et par bilder om gangen), hodet ser seg om.
    def idle(c, f, t):
        q = math.floor(t * 8) / 8
        body = wave(q, 2.0)
        body["head"] = (0.0, 0.0, 10.0 * math.sin(2 * math.pi * q))
        body["neck"] = (0.0, -3.0 * math.sin(4 * math.pi * q), 0.0)
        pose(c, f, root=(0, 0, 0.004 * H * math.sin(2 * math.pi * t)), body=body)
    _clip(Clip, arm, "idle", 48, idle)

    # Gangen: trefotsgang, to skritt per syklus, kroppen bølger.
    def walk(c, f, t):
        sw, up = gait(t, 24.0)
        body = wave(t, 5.0)
        body["head"] = (0.0, 0.0, -4.0 * math.sin(2 * math.pi * t))
        pose(c, f, root=(0, 0, 0.012 * H * abs(math.sin(2 * math.pi * t))), body=body, legs=sw, lift=up)
    _clip(Clip, arm, "walk", 20, walk)

    # Pisken: lista spennes bakover (0–12), kroppen slår rundt en hel
    # omdreining med halen og platene etter som en piske (12–26, treffet når
    # halen feier forbi foran i bilde 20), og den finner fotfestet (–36).
    # Lav (`attack`): huker seg ned med halen langs gulvet (hopp over);
    # høy (`lash_high`): reiser seg med nesa ned og halen i brysthøyde
    # (dukk under).
    def lash(high):
        def fn(c, f, t):
            sw, up = gait(t, 10.0, 3)
            if f <= 12:
                u = smooth01(f / 12)
                yaw = -50.0 * u
                lag = 0.0
                cur = 18.0 * u
            elif f <= 26:
                u = (f - 12) / 14
                e = u * u * (3 - 2 * u)
                yaw = -50.0 + 410.0 * e
                # Platene henger etter når kroppen slår rundt.
                lag = -28.0 * math.sin(math.pi * u)
                cur = 18.0 * (1 - u)
            else:
                u = smooth01((f - 26) / 10)
                yaw = 360.0
                lag = 10.0 * math.sin(math.pi * u) * (1 - u)
                cur = 0.0
            pose_h = smooth01(min(f / 10, 1.0)) * (1 - smooth01((f - 26) / 10))
            body = {"seg.1": (0.0, 0.0, cur * 0.5 + lag * 0.6), "seg.2": (0.0, 0.0, cur * 0.8 + lag), "tail": (0.0, 0.0, cur + lag * 1.3)}
            if high:
                # Nesa ned, halen og de bakre platene opp.
                body = {k: (v[0], v[1] - 12.0 * pose_h, v[2]) for k, v in body.items()}
                body["head"] = (0.0, -20.0 * pose_h, 0.0)
                pose(c, f, root=(0, 0, 0.12 * H * pose_h), yaw=yaw, pitch=16.0 * pose_h, body=body, legs=sw, lift={n: 6.0 * pose_h for n in LEG_NAMES})
            else:
                body = {k: (v[0], v[1] + 4.0 * pose_h, v[2]) for k, v in body.items()}
                body["head"] = (0.0, 8.0 * pose_h, 0.0)
                pose(c, f, root=(0, 0, -0.05 * H * pose_h), yaw=yaw, pitch=-4.0 * pose_h, body=body, legs=sw, lift={n: 22.0 * pose_h for n in LEG_NAMES})
        return fn
    _clip(Clip, arm, "attack", 36, lash(False))
    _clip(Clip, arm, "lash_high", 36, lash(True))

    # Valget: står, platene foran skrangler og løfter seg mot deg, hodet
    # sikter (spillet tenner lyset i platene).
    def choose(c, f, t):
        q = math.floor(t * 12) / 12
        rattle = 4.0 * math.sin(2 * math.pi * 6 * q)
        u = smooth01(t * 2)
        body = {"chest": (rattle, -8.0 * u, 0.0), "neck": (0.0, -6.0 * u, 0.0), "head": (0.0, 4.0 * u, 0.0), "hips": (rattle * 0.5, 0.0, 0.0)}
        pose(c, f, body=body)
    _clip(Clip, arm, "choose", 24, choose)

    # Slaget med plata: forkroppen reiser seg (0–10), hamrer ned foran seg
    # (treffet i bilde 15) og trekker seg tilbake.
    def strike(c, f, t):
        if f <= 10:
            u = smooth01(f / 10)
            up, ahead = -32.0 * u, 0.0
        elif f <= 15:
            u = (f - 10) / 5
            up, ahead = -32.0 + 52.0 * u * u, 0.08 * H * u
        else:
            u = smooth01((f - 15) / 15)
            up, ahead = 20.0 * (1 - u), 0.08 * H * (1 - u)
        body = {"hips": (0.0, up * 0.35, 0.0), "chest": (0.0, up * 0.5, 0.0), "neck": (0.0, up * 0.3, 0.0), "head": (0.0, -up * 0.2, 0.0)}
        pose(c, f, root=(ahead, 0, 0), body=body, legs={n: 8.0 * math.sin(math.pi * t) for n in LEG_NAMES})
    _clip(Clip, arm, "strike", 30, strike)

    def alert(c, f, t):
        u = math.sin(math.pi * t)
        q = math.floor(t * 12) / 12
        body = {"neck": (0.0, -18.0 * u, 0.0), "head": (0.0, -8.0 * u, 12.0 * math.sin(4 * math.pi * q)), "chest": (3.0 * math.sin(2 * math.pi * 6 * q) * u, -4.0 * u, 0.0)}
        pose(c, f, root=(0, 0, 0.02 * H * u), body=body)
    _clip(Clip, arm, "alert", 22, alert)
    for name, side in (("hit", 1.0), ("hit2", -1.0)):
        def hit(c, f, t, side=side):
            e = math.exp(-4 * t)
            body = {"neck": (0.0, -10.0 * e, side * 14.0 * e * math.cos(9 * t)), "seg.1": (0.0, 0.0, -side * 8.0 * e), "seg.2": (0.0, 0.0, -side * 10.0 * e), "tail": (0.0, 0.0, -side * 14.0 * e)}
            pose(c, f, root=(-0.05 * H * e, 0, 0), roll=side * 6.0 * e, body=body)
        _clip(Clip, arm, name, 14, hit)

    def stagger(c, f, t):
        s = math.sin(4 * math.pi * t)
        body = {"neck": (0.0, 10.0, 12.0 * s), "head": (0.0, 10.0, 0.0), "seg.2": (0.0, 0.0, -8.0 * s), "tail": (0.0, 0.0, -12.0 * s)}
        pose(c, f, roll=6.0 * s, body=body, legs={n: 6.0 * math.sin(2 * math.pi * (t * 2 + ph)) for n, ph in TRIPOD.items()})
    _clip(Clip, arm, "stagger", 30, stagger)

    # Døden: beina sklir ut og den synker ned på platene (die), eller
    # velter over på sida (die2).
    def die(c, f, t):
        u = smooth01(min(1.0, t * 1.5)) ** 2
        body = {"neck": (0.0, 25.0 * u, 0.0), "head": (0.0, 20.0 * u, 0.0), "tail": (0.0, 15.0 * u, 20.0 * u), "seg.2": (0.0, 0.0, 10.0 * u)}
        pose(c, f, root=(0, 0, -0.17 * H * u), body=body, lift={n: 70.0 * u for n in LEG_NAMES})
    _clip(Clip, arm, "die", 36, die)

    def die2(c, f, t):
        u = smooth01(min(1.0, t * 1.5)) ** 2
        body = {"neck": (0.0, 15.0 * u, 20.0 * u), "tail": (0.0, 0.0, -20.0 * u)}
        pose(c, f, root=(0, 0.1 * H * u, 0.08 * H * math.sin(math.pi * u)), roll=-80.0 * u, body=body, legs={n: 20.0 * u for n in LEG_NAMES})
    _clip(Clip, arm, "die2", 36, die2)


# --- Divider-ormen -----------------------------------------------------------

# Fugene langs ryggraden (veien langs skallet fra halespissen, brøk av den
# lengste): åtte ledd fra halen til hodet.
WORM_JOINTS = (0.13, 0.26, 0.39, 0.52, 0.63, 0.72, 0.8)
WORM_SEGS = ("tail.3", "tail.2", "tail.1", "hips", "spine", "chest", "neck", "head")
WORM_BLEND = 0.035


def worm(ob, co, H, armature, Clip):
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import dijkstra

    me = ob.data
    ed = np.empty(len(me.edges) * 2, np.int64)
    me.edges.foreach_get("vertices", ed)
    ed = ed.reshape(-1, 2)
    L = np.linalg.norm(co[ed[:, 0]] - co[ed[:, 1]], axis=1)
    n = len(co)
    G = coo_matrix((np.r_[L, L], (np.r_[ed[:, 0], ed[:, 1]], np.r_[ed[:, 1], ed[:, 0]])), shape=(n, n)).tocsr()
    # Halespissen er det bakerste (fronten mot +x).
    tip = co[:, 0] < co[:, 0].min() + 0.02 * H
    d = dijkstra(G, indices=np.nonzero(tip)[0], min_only=True)
    far = np.isfinite(d)
    s = np.where(far, d, 0.0) / d[far].max()
    print(f"ryggraden: {int((~far).sum())} hjørner henger ikke sammen med halen")
    W = _along(s, WORM_JOINTS, WORM_BLEND)
    # Ryggraden: midten av hjørnene ved hver fuge, og endene.
    J = [0.0] + list(WORM_JOINTS) + [1.0]

    def at(v, w=0.02):
        m = np.abs(s - v) < w
        return Vector(co[m].mean(0)) if m.any() else Vector(co[np.argmin(np.abs(s - v))])

    P = [at(v) for v in J]
    # Hodet peker fram (fra halsen mot snuten), ikke mot toppen av skallen.
    snout = Vector(co[np.argmax(co[:, 0])])
    P[-1] = Vector((snout.x, 0.0, P[-2].z))
    for p in P:
        p.y = 0.0
    B = [
        ("root", (0, 0, 0), (0, 0, 0.1 * H), None),
        ("hips", tuple(P[3]), tuple(P[4]), "root"),
        ("spine", tuple(P[4]), tuple(P[5]), "hips"),
        ("chest", tuple(P[5]), tuple(P[6]), "spine"),
        ("neck", tuple(P[6]), tuple(P[7]), "chest"),
        ("head", tuple(P[7]), tuple(P[8]), "neck"),
        ("tail.1", tuple(P[3]), tuple(P[2]), "hips"),
        ("tail.2", tuple(P[2]), tuple(P[1]), "tail.1"),
        ("tail.3", tuple(P[1]), tuple(P[0]), "tail.2"),
    ]
    for b in B:
        print(b[0], tuple(round(v, 3) for v in b[1]), "→", tuple(round(v, 3) for v in b[2]))
    arm = armature("divider_worm_rig", B)
    _weights(ob, arm, W, list(WORM_SEGS))
    worm_clips(arm, H, Clip)
    return arm, B


def worm_clips(arm, H, Clip):
    # Leddene fra hoftene og fram, og fra hoftene og bak.
    FWD = ("spine", "chest")
    BACK = ("tail.1", "tail.2", "tail.3")

    def pose(c, f, root=(0, 0, 0), yaw=0.0, pitch=0.0, roll=0.0, side=None, lift=None, neck=(0.0, 0.0), head=(0.0, 0.0)):
        """`side`: dreining om opp-aksen per ledd (bølgen), `lift`: opp og
        ned per ledd; `neck` og `head`: (opp/ned, til sida)."""
        c.loc("root", f, root)
        c.rot("root", f, x=roll, y=pitch, z=yaw)
        side = side or {}
        lift = lift or {}
        for nm in ("hips",) + FWD + BACK:
            c.rot(nm, f, y=lift.get(nm, 0.0), z=side.get(nm, 0.0))
        c.rot("neck", f, y=neck[0], z=neck[1])
        c.rot("head", f, y=head[0], z=head[1])

    def wave(t, amp, k=1):
        """Bølgen går bakover langs kroppen: hvert ledd litt etter det foran,
        og bakover bøyer de seg motsatt vei (beina peker bakover)."""
        out = {}
        for i, nm in enumerate(("chest", "spine", "hips", "tail.1", "tail.2", "tail.3")):
            a = amp * math.sin(2 * math.pi * (k * t - 0.17 * i))
            out[nm] = a if i < 3 else -a
        return out

    def idle(c, f, t):
        p = 2 * math.pi * t
        pose(c, f, side=wave(t, 4.0), neck=(-4.0 * math.sin(p), 6.0 * math.sin(p)), head=(3.0 * math.sin(2 * p), 10.0 * math.sin(p + 1.0)))
    _clip(Clip, arm, "idle", 48, idle)

    def walk(c, f, t):
        p = 2 * math.pi * t
        pose(c, f, side=wave(t, 16.0), lift={nm: 4.0 * math.sin(p * 2 - i) for i, nm in enumerate(("spine", "hips", "tail.1"))}, neck=(0.0, -6.0 * math.sin(p)), head=(0.0, -8.0 * math.sin(p - 0.6)))
    _clip(Clip, arm, "walk", 24, walk)

    # Bittet: halsen trekkes bakover og opp (0–11), hugger fram og ned
    # (treffet i bilde 15), og trekkes tilbake.
    def bite(c, f, t):
        if f <= 11:
            u = smooth01(f / 11)
            nk, hd, ahead = -24.0 * u, -10.0 * u, -0.03 * H * u
        elif f <= 15:
            u = (f - 11) / 4
            nk, hd, ahead = -24.0 + 64.0 * u, -10.0 + 30.0 * u, -0.03 * H + 0.28 * H * u
        else:
            u = smooth01((f - 15) / 15)
            nk, hd, ahead = 40.0 * (1 - u), 20.0 * (1 - u), 0.25 * H * (1 - u)
        pose(c, f, root=(ahead, 0, 0), side=wave(t, 4.0, 2), neck=(nk, 0.0), head=(hd, 0.0), lift={"chest": nk * 0.3})
    _clip(Clip, arm, "attack", 30, bite)

    # Graver seg ned: hodet stuper i gulvet, og resten følger etter ned
    # (roten under gulvet; spillet skjuler den når klippet er ferdig).
    def burrow(c, f, t):
        u = smooth01(t)
        dive = smooth01(t * 1.6)
        pose(c, f, root=(0.25 * H * u, 0, -1.1 * H * u * u), pitch=40.0 * dive, side=wave(t, 18.0, 3), neck=(35.0 * dive, 0.0), head=(25.0 * dive, 0.0))
    _clip(Clip, arm, "burrow", 24, burrow)

    # Dukker opp: bryter opp av gulvet med hodet høyt (0–9), hugger ned
    # (treffet i bilde 12) og legger seg til (–30).
    def emerge(c, f, t):
        if f <= 9:
            u = smooth01(f / 9)
            root, pitch, nk, hd = (-0.1 * H, 0, -1.1 * H * (1 - u)), -35.0 * (1 - u) - 15.0 * u, -20.0 * u, -10.0 * u
        elif f <= 12:
            u = (f - 9) / 3
            root, pitch, nk, hd = (-0.1 * H + 0.3 * H * u, 0, 0.0), -15.0 + 15.0 * u, -20.0 + 60.0 * u, -10.0 + 30.0 * u
        else:
            u = smooth01((f - 12) / 18)
            root, pitch, nk, hd = (0.2 * H * (1 - u), 0, 0.0), 0.0, 40.0 * (1 - u), 20.0 * (1 - u)
        pose(c, f, root=root, pitch=pitch, side=wave(t, 12.0 * (1 - t), 2), neck=(nk, 0.0), head=(hd, 0.0))
    _clip(Clip, arm, "emerge", 30, emerge)

    def alert(c, f, t):
        u = math.sin(math.pi * t)
        pose(c, f, side=wave(t, 6.0, 2), neck=(-25.0 * u, 0.0), head=(10.0 * u, 14.0 * math.sin(4 * math.pi * t)))
    _clip(Clip, arm, "alert", 22, alert)
    for name, sd in (("hit", 1.0), ("hit2", -1.0)):
        def hit(c, f, t, sd=sd):
            e = math.exp(-4 * t)
            pose(c, f, root=(-0.08 * H * e, 0, 0), side={nm: sd * 14.0 * e * (1 if i < 3 else -1) for i, nm in enumerate(("chest", "spine", "hips", "tail.1", "tail.2", "tail.3"))}, neck=(-12.0 * e, sd * 20.0 * e * math.cos(9 * t)))
        _clip(Clip, arm, name, 14, hit)

    def stagger(c, f, t):
        s = math.sin(4 * math.pi * t)
        pose(c, f, roll=10.0 * s, side=wave(t, 10.0, 2), neck=(15.0, 15.0 * s), head=(15.0, 0.0))
    _clip(Clip, arm, "stagger", 30, stagger)

    # Døden: halsen synker til gulvet og den ruller over på sida (die),
    # eller blir liggende rett ut (die2).
    def die(c, f, t, roll):
        u = smooth01(min(1.0, t * 1.5)) ** 2
        pose(c, f, root=(0, 0, 0.04 * H * u), roll=roll * u, side={nm: 10.0 * u for nm in ("chest", "hips", "tail.2")}, neck=(70.0 * u, 0.0), head=(30.0 * u, 0.0))
    _clip(Clip, arm, "die", 36, lambda c, f, t: die(c, f, t, 85.0))
    _clip(Clip, arm, "die2", 36, lambda c, f, t: die(c, f, t, 0.0))


RIGS = {"combobox": combobox, "worm": worm}
