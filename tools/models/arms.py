"""Spillerens armer i førsteperson og våpnene og skjoldene de holder, laget
etter eierens forbilder (`assets/textures/weapons_ref/`). Kjør:

    python3.13 tools/models/arms.py [arms arms_leather arms_mail sword axe dagger shield_round shield_tower]

Kamerarommet: kameraet i origo ser langs +Y i Blender (−Z i glTF), Z er
opp og X er høyre. Spillet poserer instansen i øyet med kameraets dreining
(`domain/combat.rs`). Armene er tre modeller med samme rigg og samme klipp,
valgt etter rustningen: `arms` (linerme), `arms_leather` (Badgevams) og
`arms_mail` (Fieldset-brynje). Klippene: idle (løkke), swing1–3 (komboen,
treffet ved SWING_HIT), heavy (ladning til HEAVY_HOLD, hugget ved
HEAVY_HIT), block (holdes på siste bilde), parry og hurt.

Hånda er bygd i sitt eget rom: grepet langs x gjennom origo, fingrene mot
+y (fra håndleddet), håndbaken mot +z, tommelen mot −x (høyre hånd); klinga
går ut på tommelsida (−x). Grepsarket (`arms_grip.jpeg`) viser nettopp det:
hånda ovenfra, fra tommelsida, nedenfra, og venstre hånd ovenfra, i fire
rader (linerme, lærarmbeskytter, brynje med lærhanske, stålhanske), og
grepet er 3 cm tykt (55 px). Våpnene er egne modeller i armenes rom, med
grepet der knyttneven er i hvile; spillet fester dem i hendenes
skinnmatriser."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import Clip, armature, bake_textured, clear, cylinder, ellipsoid, material, wave  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from sheetproj import ROOT, filled_sheet, sheet_material  # noqa: E402

OUT = "assets/models/arms"
REF = os.path.join(ROOT, "assets", "textures", "weapons_ref")
FPS = 24
# Sekundene i klippene der treffet lander (speiles i domain/combat.rs).
SWING_HIT = (6 / FPS, 6 / FPS, 7 / FPS)
HEAVY_HOLD = 10 / FPS
HEAVY_HIT = 13 / FPS

# --- Grepsarket ---------------------------------------------------------------

GRIP_SHEET = os.path.join(REF, "arms_grip.jpeg")
GRIP_PX = 0.03 / 55  # grepet er 3 cm og 55 px
# Radene (y til midten av grepet) og kolonnene (x til grepets akse) per sett.
GRIP_ROWS = {"arms": (140, 0), "arms_leather": (633, 1), "arms_mail": (1127, 10)}
COL_TOP, COL_SIDE, COL_BELOW, COL_LEFT = 460, 828, 1191, 1555
WRIST_IMG = 159  # px fra grepet ned til håndleddet ovenfra
SLEEVE_IMG = 345  # px fra grepet ned til nederst i arket

# Mål i håndas rom (meter).
WRIST_Y = -WRIST_IMG * GRIP_PX  # håndleddet
FOREARM = 0.25  # håndleddet til albuen
ELBOW_Y = WRIST_Y - FOREARM
ARM_PAST = 0.15  # overarmen går så langt forbi skulderen
# Overarmen får arket fra et bånd av rader under grepet (px): linermet,
# den bare armen under lærarmbeskytteren, brynjeermet.
UPPER_BAND = {"arms": (285, 340), "arms_leather": (300, 340), "arms_mail": (215, 275)}
# Grepet i hånda er 3,2 cm tykt (arket: 3 cm); våpnene skal passe (`grip_fits`).
GRIP_D = (0.028, 0.036)


# Underarmen over radene i arket, per sett: (brøkdel fra håndleddet til
# albuen, rad fra grepet). Uten egen liste strekkes den jevnt. Brynja: hanskens
# mansjett tar bare det nærmeste stykket ved håndleddet, og brynjeradene resten
# (eieren i runde 18: «Er dette chainmail?»; `mail_coverage`).
FOREARM_ROWS = {"arms_mail": ((0.0, WRIST_IMG), (0.12, 222), (0.97, 322), (1.0, SLEEVE_IMG))}
# Bånd som legges i sann målestokk og speiles fram og tilbake i stedet for å
# strekkes (brynjeringene ble lange bølger strukket fire ganger): (fra t, til t,
# første rad, siste rad).
FOREARM_TILE = {"arms_mail": (0.12, 0.97, 225, 320)}
MAIL_COVER = 0.7


def forearm_row(kind, t):
    pts = FOREARM_ROWS.get(kind, ((0.0, WRIST_IMG), (1.0, SLEEVE_IMG)))
    t = min(1.0, max(0.0, t))
    tile = FOREARM_TILE.get(kind)
    if tile and tile[0] <= t <= tile[1]:
        span = tile[3] - tile[2]
        d = (t - tile[0]) * FOREARM / GRIP_PX % (2 * span)
        return tile[2] + (d if d <= span else 2 * span - d)
    for (t0, r0), (t1, r1) in zip(pts, pts[1:]):
        if t <= t1:
            return r0 + (r1 - r0) * (t - t0) / max(1e-9, t1 - t0)
    return pts[-1][1]


def img_y(y, gy, kind=None):
    """Raden i arket for y i håndas rom: hånda i sann målestokk, og
    underarmen strukket fra håndleddet til nederst i arket (arket viser bare
    en del av underarmen), etter `FOREARM_ROWS`."""
    if y >= WRIST_Y:
        return gy - y / GRIP_PX
    return gy + forearm_row(kind, (WRIST_Y - y) / FOREARM)


def mail_coverage(kind):
    """Hvor stor del av underarmens lengde som ligger på brynjeradene i arket
    (lav metning midt i armen ovenfra)."""
    import numpy as np
    from PIL import Image

    gy, dx = GRIP_ROWS[kind]
    hsv = np.asarray(Image.open(GRIP_SHEET).convert("HSV")).astype(float)
    col = COL_TOP + dx

    def chain(r):
        return hsv[int(round(gy + r)), col - 30:col + 30, 1].mean() < 50

    ts = [i / 100 for i in range(101)]
    return sum(chain(forearm_row(kind, t)) for t in ts) / len(ts)


def project_hand(ob, gy, dx, mirror, row=None, kind=None):
    """Uv-laget «src» for en del av armen bygd i håndas rom: ovenfra (håndbaken,
    +z), nedenfra (håndflaten, −z) eller fra tommelsida, etter hvilken vei
    flaten vender. Lillefingersida har ingen egen visning i arket og får
    håndbaken eller håndflaten (tommelsida der ville vist tommelen to
    ganger). `mirror`: venstre hånd (bygd speilet, tommelen mot +x), som har
    sin egen visning ovenfra. `row`: raden i arket for y (overarmen)."""
    me = ob.data
    src = me.uv_layers.get("src") or me.uv_layers.new(name="src")
    s = -1.0 if mirror else 1.0
    row = row or (lambda y: img_y(y, gy, kind))
    for poly in me.polygons:
        n = poly.normal
        thumb = n.x * s < 0 and abs(n.x) > abs(n.z) * 1.25
        for li in poly.loop_indices:
            p = me.vertices[me.loops[li].vertex_index].co
            Y = row(p.y)
            if thumb:
                X = COL_SIDE + dx - p.z / GRIP_PX
            elif n.z >= 0:
                X = (COL_LEFT if mirror else COL_TOP) + dx + p.x / GRIP_PX
            else:
                X = COL_BELOW + dx - s * p.x / GRIP_PX
            src.data[li].uv = (X / 2000.0, 1.0 - Y / 2000.0)


def rings_object(name, mat, rings, caps=("fan", "fan")):
    """En lukket kropp av ringer (like mange hjørner i hver), med lokk i
    begge ender: «fan» samler lokket i et punkt litt utenfor, «flat» er en
    flate. Lukket og uten åpne kanter (`solid_report`)."""
    bm = bmesh.new()
    vr = [[bm.verts.new(p) for p in r] for r in rings]
    n = len(rings[0])
    for a, b in zip(vr, vr[1:]):
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((a[i], a[j], b[j], b[i]))
    for end, ring, other in ((0, vr[0], rings[1]), (1, vr[-1], rings[-2])):
        c = sum((v.co for v in ring), Vector()) / n
        if caps[end] == "fan":
            o = sum((Vector(p) for p in other), Vector()) / n
            tip = bm.verts.new(c + (c - o).normalized() * min(0.006, (c - o).length * 0.5) if (c - o).length > 1e-9 else c)
            for i in range(n):
                j = (i + 1) % n
                bm.faces.new((ring[j], ring[i], tip) if end == 0 else (ring[i], ring[j], tip))
        else:
            bm.faces.new(ring[::-1] if end == 0 else ring)
    bmesh.ops.triangulate(bm, faces=[f for f in bm.faces if len(f.verts) > 4])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    me.materials.append(mat)
    bpy.context.scene.collection.objects.link(ob)
    me.uv_layers.new(name="UVMap")
    for p in me.polygons:
        p.use_smooth = True
    return ob


def superellipse(cx, hx, cz, hz, y, n=24, p=2.0):
    """En ring i (x, z) på høyden y: avrundet rektangel for p > 2."""
    out = []
    for i in range(n):
        a = 2 * math.pi * i / n
        c, s = math.cos(a), math.sin(a)
        out.append((cx + hx * math.copysign(abs(c) ** (2 / p), c), y, cz + hz * math.copysign(abs(s) ** (2 / p), s)))
    return out


def silhouettes(kind):
    """Omrisset av hånda og underarmen i arket, ovenfra (x) og fra
    tommelsida (z), per rad fra grepet (px): {rad: (x0, x1, z0, z1)} i meter
    i håndas rom. Grepet stikker ut av neven i arket; i radene der det ligger
    (±26 px) tas bredden fra radene over og under."""
    import numpy as np
    from PIL import Image
    from scipy import ndimage

    gy, dx = GRIP_ROWS[kind]
    a = np.asarray(Image.open(GRIP_SHEET).convert("RGB")).astype(float)
    bg = np.median(a[:20].reshape(-1, 3), 0)
    m = np.abs(a - bg).max(2) > 25
    m = ndimage.binary_opening(m, iterations=2)
    m = ndimage.binary_fill_holes(m)
    rows = range(-100, SLEEVE_IMG + 1)
    spans = {}
    for view, col in (("top", COL_TOP + dx), ("side", COL_SIDE + dx)):
        win = m[gy - 110:gy + SLEEVE_IMG + 2, col - 200:col + 200].copy()
        # Grepet binder visningene sammen i arket: det skjæres bort først.
        win[110 - 27:110 + 28] = False
        lab, _ = ndimage.label(win)
        keep = lab == lab[110 + 80, 200]
        # Nevens rader over grepet hører til samme kropp som den under.
        top_lab = lab[110 - 40, 200]
        if top_lab:
            keep |= lab == top_lab
        lo, hi = {}, {}
        for r in rows:
            xs = np.nonzero(keep[110 + r])[0] if 0 <= 110 + r < keep.shape[0] else []
            if len(xs):
                lo[r], hi[r] = xs.min() - 200, xs.max() - 200
        # Grepets rader: rett linje mellom radene over og under.
        r0, r1 = -28, 28
        for r in range(r0 + 1, r1):
            t = (r - r0) / (r1 - r0)
            lo[r] = lo[r0] + (lo[r1] - lo[r0]) * t
            hi[r] = hi[r0] + (hi[r1] - hi[r0]) * t
        ks = sorted(lo)
        sm = {}
        for d in (lo, hi):
            v = ndimage.gaussian_filter1d(ndimage.median_filter(np.array([d[k] for k in ks], float), 7), 2.0)
            sm[id(d)] = dict(zip(ks, v))
        spans[view] = (sm[id(lo)], sm[id(hi)])
    out = {}
    (tl, th), (sl, sh) = spans["top"], spans["side"]
    for r in rows:
        if r in tl and r in sl:
            # Ovenfra: x = bildets x. Fra tommelsida: venstre i bildet er +z.
            out[r] = (tl[r] * GRIP_PX, th[r] * GRIP_PX, -sh[r] * GRIP_PX, -sl[r] * GRIP_PX)
    return out


def arm_mesh(kind, mat, mirror):
    """Hånda og underarmen som én kropp: ringer loftet etter omrisset i
    arket fra knokene til albuen (underarmen strukket som i `img_y`)."""
    sil = silhouettes(kind)
    top = min(sil)
    rings = []

    def ring(r, y, p):
        x0, x1, z0, z1 = sil[r]
        if mirror:
            x0, x1 = -x1, -x0
        return superellipse((x0 + x1) / 2, max(0.004, (x1 - x0) / 2), (z0 + z1) / 2, max(0.004, (z1 - z0) / 2), y, 24, p)

    r = top + 3
    while r <= WRIST_IMG:
        rings.append(ring(r, -r * GRIP_PX, 2.6 if r < 60 else 2.2))
        r += 4 if r < 60 else 8
    for i in range(1, 21):
        t = i / 20
        r = int(round(forearm_row(kind, t)))
        r = max(k for k in sil if k <= r)
        rings.append(ring(r, WRIST_Y - t * FOREARM, 2.0))
    return rings_object("arm", mat, rings, caps=("fan", "flat")), rings[-1]


def upper_arm_mesh(mat, lower_ring, L):
    """Overarmen i sitt eget rom (som håndas: −y mot skulderen), fra litt
    inne i underarmen til 15 cm forbi skulderen, med albuen rund."""
    r0 = 0.5 * (max(p[0] for p in lower_ring) - min(p[0] for p in lower_ring))
    q0 = 0.5 * (max(p[2] for p in lower_ring) - min(p[2] for p in lower_ring))
    R = max(r0, q0)
    rings = []
    # Albuen: en halvkule som runder av enden.
    for a in (70, 50, 30, 0):
        k = math.cos(math.radians(a))
        rings.append(superellipse(0, R * k, 0, R * k, 0.035 + R * math.sin(math.radians(a)) * 0.8, 20))
    for i in range(1, 13):
        t = i / 12
        rr = R * (1.0 + 0.18 * math.sin(min(1.0, t * 1.4) * math.pi / 2))
        rings.append(superellipse(0, rr, 0, rr * 0.95, 0.035 - t * (L + 0.035), 20))
    return rings_object("upper", mat, rings, caps=("fan", "flat"))


# --- Hvileposen i kamerarommet -------------------------------------------------


def frame(origin, x, y_hint):
    """Et høyrehendt rom med x-aksen `x` og y-aksen så nær `y_hint` som går,
    plassert i `origin`."""
    x = Vector(x).normalized()
    y = Vector(y_hint) - x * x.dot(Vector(y_hint))
    y.normalize()
    z = x.cross(y)
    M = Matrix((x, y, z)).transposed().to_4x4()
    M.translation = Vector(origin)
    return M


def grip_frame(origin, forearm, blade):
    """Hånda med underarmen (og fingrene) langs `forearm` og grepet så klinga
    står så nær `blade` som går vinkelrett på underarmen (før det skrå
    grepet, `GRIP_TILT`)."""
    y = Vector(forearm).normalized()
    b = Vector(blade)
    return frame(origin, -(b - y * b.dot(y)), y)


# Høyre hånd nede til høyre: underarmen stiger fram mot hånda, tommelen over
# grepet og klinga opp (eieren i runde 16: fingertuppene mot kameraet; i
# runde 17: underarmen vannrett og klinga rett opp så ut som et knekt
# håndledd).
RIGHT = grip_frame((0.27, 0.46, -0.25), (-0.28, 0.9, 0.34), (-0.2, 0.3, 0.93))
# Grepet ligger på skrå over håndflaten, som i en ekte knyttneve: klinga
# heller fram mot knokene. Våpnene står i dette rommet (`WEAPON`).
GRIP_TILT = 20.0
WEAPON = RIGHT @ Matrix.Rotation(math.radians(-GRIP_TILT), 4, "Z")
# Venstre hånd: skjoldets håndtak står loddrett i neven (tommelen opp),
# underarmen peker rett fram, og knokene mot skjoldet.
LEFT = frame((-0.3, 0.48, -0.36), (0.0, -0.05, 1.0), (0.08, 1.0, 0.1))
SHOULDER = {"R": Vector((0.2, -0.1, -0.3)), "L": Vector((-0.2, -0.1, -0.3))}
# Skjoldet vender rett fram, dreid 10° ut mot venstre (eieren i runde 16).
SHIELD_OUT = 10.0
# Grepet på skjoldet i blokken: foran brystet, litt til venstre.
BLOCK_GRIP = Vector((-0.12, 0.46, -0.15))


def at(M, p):
    return M @ Vector(p)


def bones():
    out = [("root", (0, 0, 0), (0, 0.1, 0), None)]
    for side, M in (("R", RIGHT), ("L", LEFT)):
        e, w, g = at(M, (0, ELBOW_Y, 0)), at(M, (0, WRIST_Y, 0)), at(M, (0, 0.03, 0))
        out += [(f"upper_arm.{side}", tuple(SHOULDER[side]), tuple(e), "root"), (f"forearm.{side}", tuple(e), tuple(w), f"upper_arm.{side}"), (f"hand.{side}", tuple(w), tuple(g), f"forearm.{side}")]
    return out


BONES = bones()


def upper_frame(side):
    """Overarmens rom: origo i albuen, −y mot skulderen, z (armens
    «håndbak») så nær opp som går."""
    M = RIGHT if side == "R" else LEFT
    e = at(M, (0, ELBOW_Y, 0))
    y = (e - SHOULDER[side]).normalized()
    up = Vector((0, 0, 1))
    x = y.cross(up).normalized()
    z = x.cross(y)
    F = Matrix((x, y, z)).transposed().to_4x4()
    F.translation = e
    return F


def build_arms(kind):
    gy, dx = GRIP_ROWS[kind][0], GRIP_ROWS[kind][1]
    mat = sheet_material(f"{kind}_sheet", filled_sheet(GRIP_SHEET))
    rigid = {}
    b0, b1 = UPPER_BAND[kind]
    for side, M in (("R", RIGHT), ("L", LEFT)):
        mirror = side == "L"
        ob, last = arm_mesh(kind, mat, mirror)
        ob.name = f"arm_{side}"
        project_hand(ob, gy, dx, mirror, kind=kind)
        ob.data.transform(M)
        ob.data.update()
        # Fra albuen til 15 cm forbi skulderen: enden er alltid utenfor bildet.
        L = (at(M, (0, ELBOW_Y, 0)) - SHOULDER[side]).length + ARM_PAST
        up = upper_arm_mesh(mat, last, L)
        up.name = f"upper_{side}"
        project_hand(up, gy, dx, mirror, row=lambda y: gy + b0 + min(1.0, max(0.0, -y / L)) * (b1 - b0))
        up.data.transform(upper_frame(side))
        up.data.update()
        rigid[up.name] = f"upper_arm.{side}"
    return rigid


# --- Klippene ------------------------------------------------------------------


def ik(c, f, side, grip, rb, pole):
    """Hånda i `side` dreid `rb` (fra hvileposen, i kamerarommet) med grepet i
    `grip` (`c` None: bare bøyen i håndleddet, uten nøkkelbilder). Albuen ligger på sirkelen skulderen og håndleddet gir; den
    velges der håndleddet bøyes minst (nær `pole`, albuen ned og ut).
    Underarmen tar hele vridningen fra hånda, så hånda bare bøyes litt i
    håndleddet og aldri vris (eieren i runde 17: håndleddet så knekt ut, og
    skjoldhånda var bøyd som gummi; `wrist_bend` måler det)."""
    M = RIGHT if side == "R" else LEFT
    S = SHOULDER[side]
    G0, W0, E0 = M.translation.copy(), at(M, (0, WRIST_Y, 0)), at(M, (0, ELBOW_Y, 0))
    y0 = (W0 - E0).normalized()
    w = Vector(grip) - rb @ (G0 - W0)
    a, b = (E0 - S).length, (W0 - E0).length
    dv = w - S
    # Et mål armen ikke når, trekker hånda mot kameraet i stedet (runde 17:
    # opptrekket i det tunge hugget havnet foran øyet).
    if dv.length > a + b + 0.01:
        if c is None:
            return 1e9
        raise SystemExit(f"{c.name} bilde {f}: grepet {tuple(round(v, 2) for v in grip)} er {100 * (dv.length - a - b):.0f} cm utenfor rekkevidden til {side}")
    dist = min(max(dv.length, abs(a - b) + 0.01), a + b - 0.005)
    u = dv.normalized()
    w = S + u * dist
    pole = Vector(pole)
    p = (pole - u * pole.dot(u)).normalized()
    q = u.cross(p)
    cos_a = (a * a + dist * dist - b * b) / (2 * a * dist)
    al = math.acos(max(-1.0, min(1.0, cos_a)))
    hand = rb @ y0
    best = None
    for i in range(72):
        t = 2 * math.pi * i / 72
        e = S + (u * math.cos(al) + (p * math.cos(t) + q * math.sin(t)) * math.sin(al)) * a
        bend = math.degrees((w - e).angle(hand))
        # Albuen helst mot `pole`: straff som vokser når den dreies bort.
        score = bend + 25.0 * (1.0 - math.cos(t))
        if best is None or score < best[0]:
            best = (score, e, bend)
    if c is None:
        return best[2]
    e = best[1]
    ru = (E0 - S).rotation_difference(e - S)
    rf = hand.rotation_difference(w - e) @ rb
    c.rotq(f"upper_arm.{side}", f, ru)
    c.rotq(f"forearm.{side}", f, ru.inverted() @ rf)
    c.rotq(f"hand.{side}", f, rf.inverted() @ rb)


def clips(rig):
    from mathutils import Quaternion

    # Idle: pusten løfter armene litt, og klinga vugger.
    c = Clip(rig, "idle", 48, FPS)
    c.rest_all(0)
    for f, v in wave(48, 48, 1.0):
        c.loc("root", f, (0, 0.004 * v, 0.008 * v))
        c.rot("upper_arm.R", f, x=1.5 * v)
        c.rot("upper_arm.L", f, x=-1.2 * v)
    c.finish()
    # Kombo: hugg fra høyre, hugg fra venstre, hugg ovenfra. Hvert
    # nøkkelbilde sier hvor grepet skal være og hvilken vei klinga peker i
    # kamerarommet (x høyre, y fram, z opp); overarmen dreies så grepet når
    # dit, og hånda så klinga peker dit med flatsida mot kameraet. Treffet er
    # i bilde 6, 6 og 7 (`SWING_HIT`), og klinga krysser midten av bildet
    # der fienden står.
    B0 = -Vector(WEAPON.col[0][:3]).normalized()  # klinga i hvile
    E0 = Vector(WEAPON.col[1][:3]).normalized()  # eggen (knokene) i hvile

    # Huggene er glatte baner gjennom nøkkelbildene, og hvert bilde regnes
    # ut for seg (spillet glir bare mellom nabobilder).
    def catmull(points, t):
        # Glatt kurve gjennom punktene (t fra 0 til len−1).
        n = len(points)
        k = min(int(t), n - 2)
        u = t - k
        p0, p1, p2, p3 = (Vector(points[max(k - 1, 0)]), Vector(points[k]), Vector(points[k + 1]), Vector(points[min(k + 2, n - 1)]))
        return 0.5 * ((2 * p1) + (-p0 + p2) * u + (2 * p0 - 5 * p1 + 4 * p2 - p3) * u * u + (-p0 + 3 * p1 - 3 * p2 + p3) * u ** 3)

    def track(keys, f):
        # Verdien i bilde f langs kurven gjennom (bilde, verdi)-parene.
        frames = [k for k, _ in keys]
        if f <= frames[0]:
            return Vector(keys[0][1])
        if f >= frames[-1]:
            return Vector(keys[-1][1])
        k = max(i for i, fr in enumerate(frames) if fr <= f)
        t = k + (f - frames[k]) / (frames[k + 1] - frames[k])
        return catmull([v for _, v in keys], t)

    # Huggene drives av skulderen (eieren 14. oktober: «det ser ut som at
    # albuen roterer i stedet for at svinget er drevet av skulderleddet»).
    # Hele armen dreies om skulderen langs hugglinja (`motion`: veien hånda
    # går i treffet); underarmen rulles én gang i opptrekket så klinga
    # ligger i hugglinja (`roll`), og holdes slik til hugget er over;
    # albuen strekkes litt i treffet og håndleddet slår klinga fram
    # innenfor `WRIST_BEND`. Grepet og klinga følger av leddene, ikke
    # omvendt. Nøklene: (bilde, skulder, albue, håndledd, kropp) i grader;
    # skulderen positiv er opptrekket og negativ gjennomføringen.
    S = SHOULDER["R"]
    G0, W0, EL0 = RIGHT.translation.copy(), at(RIGHT, (0, WRIST_Y, 0)), at(RIGHT, (0, ELBOW_Y, 0))
    y0 = (W0 - EL0).normalized()

    def arm_pose(grip, roll, cock, pole=(0.35, -0.2, -1.0)):
        """Armen med grepet i `grip`: albuen fra IK (skulderen og håndleddet,
        albuen ned og ut mot `pole`), underarmen dreid med overarmen og rullet
        `roll` grader om sin egen akse, og hånda slått `cock` grader i
        håndleddet (i klingas plan). Gir (ru, rf, rb) i kamerarommet."""
        a_, b_ = (EL0 - S).length, (W0 - EL0).length
        # Håndleddet står like langt fra grepet som i hvile, i retning fra
        # skulderen (første gjetning); to runder retter det inn.
        w = Vector(grip) - (G0 - W0)
        for _ in range(3):
            dv = w - S
            dist = min(max(dv.length, abs(a_ - b_) + 0.01), a_ + b_ - 0.005)
            u = dv.normalized()
            w = S + u * dist
            pv = Vector(pole)
            p_ = (pv - u * pv.dot(u)).normalized()
            cos_a = (a_ * a_ + dist * dist - b_ * b_) / (2 * a_ * dist)
            e = S + (u * cos_a + p_ * math.sqrt(max(0.0, 1 - cos_a * cos_a))) * a_
            # Overarmen tar med seg albuens plan (å dreie albuen rundt er å
            # rotere overarmen i skulderen), så underarmen bare bøyes som et
            # hengsel og rulles.
            def basis(a, b):
                x = a.normalized()
                y = a.cross(b).normalized()
                return Matrix((x, y, x.cross(y))).transposed()

            ru = (basis(e - S, w - e) @ basis(EL0 - S, W0 - EL0).transposed()).to_quaternion()
            rf = (ru @ y0).rotation_difference(w - e) @ ru @ Quaternion(y0, math.radians(roll))
            bl = Quaternion(y0, math.radians(roll)) @ B0
            axis = y0.cross(bl).normalized()
            rb = rf @ Quaternion(axis, math.radians(cock))
            w = Vector(grip) - rb @ (G0 - W0)
        return ru, rf, rb

    def swing(name, length, hit, keys, shield_drop=1.0, pole=None):
        """`keys`: (bilde, grep, håndledd, kropp). Rullen av underarmen
        velges så eggen går først gjennom treffet (bildene rundt `hit`), den
        minste som gjør det; den kommer inn i opptrekket og går ut etter
        treffet, så underarmen rulles bare to ganger per hugg."""
        # Hånda holdes like langt fra skulderen som i hvile (litt lenger ut
        # i treffet): da står albuen stille, og skulderen gjør hugget.
        R0 = (G0 - S).length

        def on_sphere(f, g):
            return tuple(S + (Vector(g) - S).normalized() * (R0 + (REACH_OUT if f == hit else 0.0)))

        grips = [(0, tuple(G0))] + [(f, on_sphere(f, g)) for f, g, _, _ in keys] + [(length, tuple(G0))]
        cocks = [(0, (0.0, 0.0, 0.0))] + [(f, (w_, 0.0, 0.0)) for f, _, w_, _ in keys] + [(length, (0.0, 0.0, 0.0))]
        roots = [(0, (0, 0, 0))] + [(f, r_) for f, _, _, r_ in keys] + [(length, (0, 0, 0))]
        first, last = keys[0][0], hit + 1

        def share(f):
            k = min(1.0, f / max(1, first)) if f < first else (1.0 if f <= last else max(0.0, 1 - (f - last) / max(1, length - last)))
            return k * k * (3 - 2 * k)

        # Albuen glir fra hvileposens retning til hoggets og tilbake, så
        # hugget begynner og slutter i hvileposen.
        rest_pole = (EL0 - S).normalized()

        def pole_at(f):
            if pole is None:
                return rest_pole
            k = share(f)
            return rest_pole * (1 - k) + Vector(pole).normalized() * k

        def tip(f, roll):
            _, _, rb = arm_pose(track(grips, f), roll * share(f), track(cocks, f).x, pole_at(f))
            return track(roots, f) + track(grips, f) + (rb @ B0) * 0.6, rb

        def lead(roll):
            worst = 2.0
            for f in (hit - 1, hit, hit + 1):
                v = (tip(f + 1, roll)[0] - tip(f - 1, roll)[0]).normalized()
                worst = min(worst, (tip(f, roll)[1] @ E0).dot(v))
            return worst

        roll = max(range(-150, 151, 6), key=lambda r: min(lead(r), EDGE_LEADS + 0.1) - abs(r) / 900.0)
        if os.environ.get("FRISTAD_DEBUG"):
            print(name, "rull", roll, "eggen", round(lead(roll), 2))
        c = Clip(rig, name, length, FPS)
        c.rest_all(0)
        for f in range(length + 1):
            ru, rf, rb = arm_pose(track(grips, f), roll * share(f), track(cocks, f).x, pole_at(f))
            c.rotq("upper_arm.R", f, ru)
            c.rotq("forearm.R", f, ru.inverted() @ rf)
            c.rotq("hand.R", f, rf.inverted() @ rb)
            c.loc("root", f, tuple(track(roots, f)))
            w = max(0.0, math.sin(math.pi * f / length)) ** 0.5 * shield_drop
            ik(c, f, "L", LEFT.translation + Vector((-0.14, -0.12, -0.14)) * w, Quaternion(), (-0.35, -0.2, -1.0))
        c.finish()

    # Hånda går en stor bue over bildet (skulderen gjør jobben), og står
    # foran høyre skulder i treffet med underarmen mot fienden, så klinga
    # feier over midten med eggen først. Håndleddet slår klinga bakover i
    # opptrekket (+) og fram gjennom treffet (−).
    # Hugg fra høyre: fra ved høyre øre, skrått ned mot venstre hofte.
    swing("swing1", 13, 6, [
        (3, (0.36, 0.2, 0.06), 25, (0.02, -0.02, 0.01)),
        (5, (0.3, 0.42, -0.04), 10, (0.0, 0.03, 0.0)),
        (6, (0.2, 0.48, -0.14), -5, (-0.02, 0.06, 0.0)),
        (7, (0.1, 0.46, -0.22), -15, (-0.03, 0.05, 0.0)),
        (9, (0.03, 0.42, -0.32), -25, (-0.03, 0.04, -0.01)),
        (11, (0.12, 0.42, -0.3), -10, (-0.01, 0.02, 0.0)),
    ], shield_drop=1.25)
    # Tilbakehugg: fra over venstre skulder, skrått ned mot høyre.
    swing("swing2", 16, 6, [
        (3, (-0.02, 0.36, 0.04), 12, (-0.02, -0.02, 0.01)),
        (5, (0.0, 0.44, 0.0), 10, (0.0, 0.03, 0.0)),
        (6, (0.12, 0.5, -0.1), -5, (0.02, 0.06, 0.0)),
        (7, (0.24, 0.46, -0.2), -15, (0.03, 0.05, 0.0)),
        (9, (0.4, 0.34, -0.36), -25, (0.03, 0.04, -0.01)),
        (12, (0.32, 0.42, -0.3), -10, (0.01, 0.02, 0.0)),
    ])
    # Hugg ovenfra: hånda over hodet og rett ned foran høyre skulder.
    swing("swing3", 16, 7, [
        (4, (0.24, 0.14, 0.18), 25, (0, -0.03, 0.03)),
        (6, (0.22, 0.38, 0.04), 10, (0, 0.03, 0.0)),
        (7, (0.19, 0.48, -0.1), -25, (0, 0.08, -0.03)),
        (8, (0.18, 0.46, -0.22), -28, (0, 0.07, -0.04)),
        (10, (0.17, 0.38, -0.36), -25, (0, 0.05, -0.04)),
        (13, (0.22, 0.42, -0.3), -10, (0, 0.02, -0.01)),
    ])
    # Tungt hugg: hånda høyt og langt bak over høyre skulder, holdes mens det
    # lades (bilde 10, HEAVY_HOLD), og så et hugg rett ned der hele kroppen
    # kaster seg fram, med en lang gjennomføring nede.
    swing("heavy", 24, 13, [
        (4, (0.32, 0.22, 0.08), 20, (0.02, -0.03, 0.02)),
        (8, (0.3, 0.01, 0.23), 28, (0.03, -0.06, 0.04)),
        (10, (0.3, 0.0, 0.24), 28, (0.03, -0.07, 0.045)),
        (12, (0.26, 0.36, 0.06), -4, (0.0, 0.05, -0.02)),
        (13, (0.18, 0.48, -0.12), -20, (-0.04, 0.17, -0.09)),
        (15, (0.17, 0.4, -0.36), -30, (-0.05, 0.16, -0.1)),
        (18, (0.2, 0.3, -0.52), -30, (-0.03, 0.07, -0.06)),
        (21, (0.16, 0.42, -0.32), -10, (-0.01, 0.02, -0.02)),
    ], shield_drop=1.3)
    # Blokk: skjoldet løftes foran brystet og dreies rett fram (ikke inn
    # forbi rett fram), holdes på siste bilde.
    pole_l = (-0.35, -0.2, -1.0)
    face = Quaternion((0, 0, 1), math.radians(-SHIELD_OUT))
    c = Clip(rig, "block", 8, FPS)
    c.rest_all(0)
    ik(c, 8, "L", BLOCK_GRIP, face, pole_l)
    c.rot("upper_arm.R", 8, x=-8, z=-12)
    c.finish()
    # Parade: fra blokken støter skjoldet fram og kommer tilbake.
    c = Clip(rig, "parry", 12, FPS)
    c.rest_all(0)
    ik(c, 0, "L", BLOCK_GRIP, face, pole_l)
    ik(c, 4, "L", BLOCK_GRIP + Vector((0.02, 0.03, 0.02)), face, pole_l)
    c.loc("root", 4, (-0.02, 0.04, 0.0))
    ik(c, 12, "L", BLOCK_GRIP, face, pole_l)
    c.loc("root", 12, (0, 0, 0))
    c.finish()
    # Treffes: armene slenges, skjoldet kastes litt inn.
    c = Clip(rig, "hurt", 10, FPS)
    c.rest_all(0)
    c.loc("root", 2, (0.04, -0.06, -0.04))
    c.rot("upper_arm.R", 2, x=-8, z=8)
    c.rot("upper_arm.L", 2, x=-6, z=-4)
    c.loc("root", 10, (0, 0, 0))
    c.rot("upper_arm.R", 10)
    c.rot("upper_arm.L", 10)
    c.finish()
    bpy.context.scene.frame_end = 48


# --- Våpnene ------------------------------------------------------------------
#
# Hvert våpen er tegnet på skrå i arket. Bildet dreies så våpenet står
# loddrett med spissen opp, og våpenet bygges av delene langs aksen (`PARTS`),
# hver en lukket kropp med sin egen form: klinga linseformet, parerstanga og
# knappen med ekte tykkelse, grepet og skaftet runde, øksebladet en kile.
# Eieren i runde 16: det blåste og slankede sverdet hadde hull og var en nål
# fra sida (`solid_report` stopper nå hull, `GRIP_D` et grep hånda ikke
# passer). Arket legges på fra sida.

WEAPONS = {
    # navn: (ark, lengde m, grepets midte i brøkdel fra nederst)
    "sword": ("weapon_sword.jpeg", 0.95, 0.16),
    "axe": ("weapon_axe.jpeg", 0.72, 0.13),
    "dagger": ("weapon_dagger.jpeg", 0.42, 0.24),
}
# Terskelen mot bakgrunnen per ark (lyst metall på grått trenger lav terskel).
MASK_AT = {"sword": 14, "axe": 14, "dagger": 30
}


def upright(name):
    """Våpenet dreid loddrett med spissen opp og beskåret: (bilde, maske,
    metall) som numpy-tabeller, lagret som `target/sheets/<navn>_upright.png`."""
    import numpy as np
    from PIL import Image
    from scipy import ndimage

    src = os.path.join(REF, WEAPONS[name][0])
    # Dolken: råbildet, ikke det utfylte. Der breddeprofilen fyller inn den
    # lyse sida av klinga, er bakgrunnen grå som stålet (det utfylte tar
    # konturens mørke).
    im = Image.open(src if name == "dagger" else filled_sheet(src)).convert("RGB")
    raw = np.asarray(Image.open(src).convert("RGB")).astype(float)
    bg = np.median(np.concatenate([raw[:20].reshape(-1, 3), raw[-20:].reshape(-1, 3)]), 0)
    m = np.abs(raw - bg).max(2) > MASK_AT[name]
    m = ndimage.binary_opening(m, iterations=2)
    lab, k = ndimage.label(m)
    sz = ndimage.sum(m, lab, range(1, k + 1))
    # Det lyse metallet ligger nær bakgrunnsgrått, men den malte konturen
    # er tydelig: den lukkes og fylles, ellers blir klinga kløyvd.
    m = lab == (np.argmax(sz) + 1)
    m = ndimage.binary_closing(np.pad(m, 12), structure=np.ones((3, 3)), iterations=2 if name == "dagger" else 10)[12:-12, 12:-12]
    m = ndimage.binary_fill_holes(m)
    ys, xs = np.nonzero(m)
    cov = np.cov(np.stack([xs - xs.mean(), ys - ys.mean()]))
    v = np.linalg.eigh(cov)[1][:, 1]
    ang = math.degrees(math.atan2(v[1], v[0]))
    # Dreid så aksen står loddrett; spissen er enden med det smaleste metallet.
    rot = ang + 90.0
    mi = Image.fromarray((m * 255).astype(np.uint8)).rotate(rot, resample=Image.BILINEAR, expand=True)
    ii = im.rotate(rot, resample=Image.BICUBIC, expand=True)
    mm = np.asarray(mi) > 127
    ys, xs = np.nonzero(mm)
    box = (xs.min() - 4, ys.min() - 4, xs.max() + 5, ys.max() + 5)
    mm = mm[box[1]:box[3], box[0]:box[2]]
    ii = ii.crop(box)
    arr = np.asarray(ii).astype(float)
    # Metallet (klinga, øksebladet) står over grepet i alle våpnene.
    hsv0 = np.asarray(Image.fromarray(arr.astype(np.uint8)).convert("HSV")).astype(float)
    met0 = (hsv0[..., 1] < 45) & mm
    if np.nonzero(met0)[0].mean() > np.nonzero(mm & ~met0)[0].mean():
        mm, arr = mm[::-1, ::-1], arr[::-1, ::-1]
    if name == "dagger":
        # Dolken: konturen er åpen langs den lyse klinga og knappen er en
        # skive i bakgrunnens farge, så masken bygges av delene langs aksen
        # (klinga, parerstanga, grepet, knappen), hver sitt konvekse skall.
        from skimage.morphology import convex_hull_image

        # Bredden er spennet fra kant til kant (knappen er bare en bue).
        width = np.array([(np.ptp(np.nonzero(r)[0]) + 1) if r.any() else 0 for r in mm], dtype=float)
        rows = np.nonzero(width)[0]
        top, bot = rows.min(), rows.max()
        guard = np.nonzero(width >= 0.75 * width.max())[0]
        g0, g1 = guard.min(), guard.max()
        below = np.arange(g1 + 1, bot + 1)
        grip_w = np.percentile(width[below], 20)
        # Knappen: nedenfra og opp så lenge den er bredere enn grepet.
        p0 = bot
        while p0 > g1 + 1 and width[p0 - 1] > grip_w * 1.3:
            p0 -= 1
        # Grepet begynner der bredden har falt til grepets (under kanten av
        # parerstanga), ellers blir skallet et trapes.
        gm = np.median(width[below])
        q0 = next((r for r in below if width[r] <= gm * 1.15), g1 + 1)
        out = np.zeros_like(mm)
        for r0, r1, hull in ((top, g0 + 3, True), (g0, q0 + 3, False), (q0 - 2, p0 + 3, False), (p0 - 2, bot + 1, True)):
            part = np.zeros_like(mm)
            part[r0:r1] = mm[r0:r1]
            if not part.any():
                continue
            if hull:
                out |= convex_hull_image(part)
            else:
                # Parerstanga og grepet (med ringer): rad for rad fra kant til kant.
                for r in range(r0, r1):
                    xs_r = np.nonzero(part[r])[0]
                    if len(xs_r):
                        out[r, xs_r.min():xs_r.max() + 1] = True
        # Symmetrisk om aksen (midt i klinga): hver rad like langt ut på
        # begge sider, til den korteste kanten (skygger og striper ved siden
        # av grepet kommer bare på den ene sida).
        mid = float(np.median([np.nonzero(out[r])[0].mean() for r in range(top, g0) if out[r].any()]))
        mm = np.zeros_like(out)
        for r in range(out.shape[0]):
            xs_r = np.nonzero(out[r])[0]
            if len(xs_r):
                half = min(mid - xs_r.min(), xs_r.max() - mid)
                if half > 0:
                    mm[r, int(mid - half):int(mid + half) + 1] = True
        # Delene henger sammen (en glipe mellom dem ville delt omrisset).
        mm = ndimage.binary_closing(mm, structure=np.ones((15, 3)))
    hsv = np.asarray(Image.fromarray(arr.astype(np.uint8)).convert("HSV")).astype(float)
    # Metall er grått (lav metning); flekker av rust på bladet og bleke
    # flekker i treet jevnes ut med et medianfilter.
    metal = ndimage.median_filter((hsv[..., 1] < 55).astype(np.uint8), size=21).astype(bool) & mm
    out = os.path.join(ROOT, "target", "sheets", f"{name}_upright.png")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    Image.fromarray(arr.astype(np.uint8)).save(out)
    return out, mm, metal


# Delene langs aksen i det dreide bildet (brøkdel av høyden fra spissen):
# (slag, fra, til). Slagene:
#   blade  klinge: linseformet snitt (eggen skarp, ryggen `ridge` m tykk på hver side)
#   bar    parerstang: avrundet stav, 2 cm tykk
#   grip   grep og skaft: rundt snitt, skalert så grepet er 3,2 cm der hånda holder
#   disc   knapp som skive: tykkelsen 40 % av bredden
#   ball   knapp som kule
#   head   økseblad: omrisset fra masken, kile fra skaftet ut mot eggen
PARTS = {
    "sword": [("blade", 0.008, 0.668), ("bar", 0.664, 0.722), ("grip", 0.718, 0.895), ("disc", 0.885, 0.995)],
    "axe": [("head", 0.0, 0.40), ("grip", 0.03, 0.985)],
    "dagger": [("blade", 0.01, 0.462), ("disc", 0.455, 0.565), ("grip", 0.555, 0.875), ("ball", 0.86, 0.995)],
}
# Klinger der omrisset ikke kan brukes: (halv bredde px, spissens del av klinga).
# Spissen smalner jevnt og svakt buet til et punkt, som på arket (eieren 13.
# oktober: «spissen er for rund»).
BLADE_SHAPE = {"dagger": (127, 0.45)}
# Spissen er spiss: 2 cm fra spissen er klinga høyst så stor del av sin
# største bredde (en rund spiss er nesten full bredde der, `tip_check`).
TIP_AT, TIP_MAX = 0.02, 0.5
RIDGE = {"sword": 0.005, "dagger": 0.004, "axe": 0.011}


def weapon_profile(mask):
    """Spennet per rad (venstre, høyre kolonne) og midtlinja, glattet."""
    import numpy as np
    from scipy import ndimage

    h = mask.shape[0]
    lo = np.full(h, np.nan)
    hi = np.full(h, np.nan)
    for r in range(h):
        xs = np.nonzero(mask[r])[0]
        if len(xs):
            lo[r], hi[r] = xs.min(), xs.max() + 1
    return lo, hi


def smooth_series(v, size):
    import numpy as np
    from scipy import ndimage

    ok = ~np.isnan(v)
    idx = np.arange(len(v))
    v = np.interp(idx, idx[ok], v[ok])
    return ndimage.gaussian_filter1d(ndimage.median_filter(v, size), size / 4)


def weapon_parts(name, mask, metal, px, gy, length):
    """Delene som lukkede kropper i håndas rom (aksen langs −x mot spissen,
    bredden langs y, tykkelsen langs z), og for hver del hvor mye bredden er
    skalert (for arket). Returnerer [(navn, ringer, skala)] og grepets
    tykkelse der hånda holder."""
    import numpy as np

    h, w = mask.shape
    lo, hi = weapon_profile(mask)
    mid = smooth_series((lo + hi) / 2, 41)
    half = (hi - lo) / 2
    parts = []
    # Grepets bredde der hånda holder, og skalaen som gir 3,2 cm.
    gr = int(gy)
    grip_half = float(np.nanmedian(half[gr - 15:gr + 15])) * px
    k_grip = 0.016 / grip_half

    def X(r):
        return -(gy - r) * px

    for kind, f0, f1 in PARTS[name]:
        r0, r1 = int(f0 * h), min(h - 1, int(f1 * h))
        if kind == "head":
            parts.append((kind, None, 1.0, (r0, r1, mid)))
            continue
        hv = half.copy()
        cm = mid
        if kind == "blade":
            # Klinga er rett og symmetrisk om én akse: den bredeste sida
            # (malte hakk i eggen gjør den ene sida smal), glattet, og aldri
            # smalere mot parerstanga enn lenger opp.
            c = blade_axis(lo, hi, r0, r1)
            mid[r0:r1 + 1] = c
            hv = np.maximum(c - lo, hi - c)
            hv = smooth_series(hv, 61)
            hv[r0:r1 + 1] = np.maximum.accumulate(hv[r0:r1 + 1])
            if name in BLADE_SHAPE:
                # Omrisset er et skall der (dolken): klinga er parallell med
                # målt bredde og smalner til en spiss.
                half_px, tip = BLADE_SHAPE[name]
                for r in range(r0, r1 + 1):
                    t = (r - r0) / max(1, (r1 - r0) * tip)
                    hv[r] = half_px * ((1 - (1 - t) ** 1.6) if t < 1 else 1.0)
        elif kind == "grip":
            # Skaftet og grepet etter treet alene: der øksebladet sitter,
            # er bredden bladets, så den fylles inn fra treet over og under.
            wlo, whi = weapon_profile(mask & ~metal)
            wh = (whi - wlo) / 2
            wh[: r0] = np.nan
            wh[r1 + 1:] = np.nan
            wide = wh > 1.6 * np.nanmedian(wh[r0:r1])
            wh[wide] = np.nan
            hv = smooth_series(wh, 21)
            wm = (wlo + whi) / 2
            wm[np.isnan(wh)] = np.nan
            cm = smooth_series(wm, 61)
        elif kind in ("bar", "disc"):
            hv = smooth_series(half, 9)
        step = max(1, (r1 - r0) // {"blade": 40, "bar": 10, "grip": 24, "disc": 14, "ball": 14}[kind])
        sample = list(range(r0, r1, step)) + [r1]
        rings = []
        k = k_grip if kind == "grip" else 1.0
        for r in sample:
            # Midtlinja følger bildet (bare tykkelsen skaleres), så skaftet
            # går gjennom øksehodet der arket viser det.
            cy = (cm[r] - mid[gr]) * px
            if kind == "ball":
                R = (r1 - r0) / 2 * px
                t = (r - (r0 + r1) / 2) * px
                hw = math.sqrt(max(0.0, R * R - t * t))
                hw = max(hw, 0.0005)
                rings.append(superellipse_x(X(r), cy, hw, 0.0, hw, 16))
                continue
            hw = max(float(hv[r]) * px * k, 0.0008)
            if kind == "blade":
                # Ryggen tynner ut mot spissen, eggen er skarp.
                t = RIDGE[name] * min(1.0, 0.3 + hw / 0.012)
                rings.append(lens(X(r), cy, hw, t))
            elif kind == "bar":
                rings.append(superellipse_x(X(r), cy, hw, 0.0, 0.01, 16, 3.0))
            elif kind == "disc":
                rings.append(superellipse_x(X(r), cy, hw, 0.0, max(0.006, 0.4 * hw), 16, 2.5))
            else:
                rings.append(superellipse_x(X(r), cy, hw, 0.0, hw, 16))
        if kind == "blade":
            tip_check(name, [(X(r), max(float(hv[r]) * px, 0.0008)) for r in range(r0, r1 + 1)])
        parts.append((kind, rings, k, cm))
    return parts, 2 * grip_half * k_grip, mid


def tip_check(name, profile):
    """Stopper en klinge med rund spiss: halv bredde `TIP_AT` bak spissen
    mot den største. `profile` er [(x, halv bredde)] fra spissen."""
    x0 = profile[0][0]
    widest = max(h for _, h in profile)
    near = min(profile, key=lambda p: abs(abs(p[0] - x0) - TIP_AT))[1]
    print(f"{name}: spissen {near / widest * 100:.0f} % av bredden {TIP_AT * 100:.0f} cm inn")
    if near / widest > TIP_MAX:
        raise SystemExit(f"{name}: spissen er rund ({near / widest * 100:.0f} % av bredden {TIP_AT * 100:.0f} cm inn, høyst {TIP_MAX * 100:.0f})")


def blade_axis(lo, hi, r0, r1):
    """Klingas akse: midten i radene der begge eggene er hele (bredden nær
    medianen), samme for hele klinga."""
    import numpy as np

    w = hi[r0:r1] - lo[r0:r1]
    ok = ~np.isnan(w)
    med = np.median(w[ok])
    good = ok & (np.abs(w - med) < 0.06 * med)
    return float(np.median(((lo + hi) / 2)[r0:r1][good]))


def superellipse_x(x, cy, hy, cz, hz, n=16, p=2.0):
    """En ring i (y, z) på x (våpnenes akse)."""
    out = []
    for i in range(n):
        a = 2 * math.pi * i / n
        c, s = math.cos(a), math.sin(a)
        out.append((x, cy + hy * math.copysign(abs(c) ** (2 / p), c), cz + hz * math.copysign(abs(s) ** (2 / p), s)))
    return out


def lens(x, cy, hy, t, n=6):
    """Klingesnitt: skarp egg i begge sider, ryggen `t` tykk på hver side."""
    out = []
    for i in range(n + 1):
        u = 1 - 2 * i / n
        out.append((x, cy + hy * u, t * (1 - u * u)))
    for i in range(1, n):
        u = -1 + 2 * i / n
        out.append((x, cy + hy * u, -t * (1 - u * u)))
    return out


def head_mesh(name, mat, mask, metal, px, gy, cx, rows, cap):
    """Øksebladet: omrisset av metallet i radene, glattet så malte hakk
    forsvinner, fylt med trekanter (uten slanking, så ingen hull), og løftet
    som en kile fra skaftet ut mot eggen; undersida speilet."""
    import numpy as np
    from scipy import ndimage
    from skimage import measure

    r0, r1 = rows
    m = np.zeros_like(mask)
    m[r0:r1] = mask[r0:r1] & metal[r0:r1]
    m = ndimage.binary_opening(m, iterations=6)
    m = ndimage.binary_closing(np.pad(m, 20), iterations=14)[20:-20, 20:-20]
    m = ndimage.binary_fill_holes(m)
    lab, k = ndimage.label(m)
    m = lab == (np.argmax(ndimage.sum(m, lab, range(1, k + 1))) + 1)
    h, w = m.shape
    dist = ndimage.distance_transform_edt(m)
    outline = max(measure.find_contours(np.pad(m, 1).astype(float), 0.5), key=len) - 1.0
    outline = measure.approximate_polygon(outline, tolerance=3.0)[:-1]

    def pos(py, pxx):
        return Vector((-(gy - py) * px, (pxx - cx) * px, 0.0))

    bm = bmesh.new()
    vs = [bm.verts.new(pos(py, pxx)) for py, pxx in outline]
    face = bm.faces.new(vs)
    bmesh.ops.triangulate(bm, faces=[face], quad_method="BEAUTY", ngon_method="BEAUTY")
    for _ in range(2):
        bmesh.ops.subdivide_edges(bm, edges=bm.edges[:], cuts=1, use_grid_fill=True)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    for f in bm.faces:
        if f.normal.z < 0:
            f.normal_flip()
    edge = {v for e in bm.edges if e.is_boundary for v in e.verts}
    # Eggen er den delen av kanten lengst fra skaftet; bladet er tykkest ved
    # skaftet og tynnest i eggen, og kanten ellers har litt tykkelse.
    for v in bm.verts:
        py = min(h - 1, max(0, int(round(gy + v.co.x / px))))
        pxx = min(w - 1, max(0, int(round(cx + v.co.y / px))))
        d = dist[py, pxx] * px
        off = abs(v.co.y)
        z = cap * max(0.12, 1.0 - off / 0.16) * min(1.0, d / 0.01 + 0.25)
        v.co.z = 0.0008 if v in edge else z
    dup = bmesh.ops.duplicate(bm, geom=bm.verts[:] + bm.edges[:] + bm.faces[:])
    dv = [g for g in dup["geom"] if isinstance(g, bmesh.types.BMVert)]
    for v in dv:
        v.co.z = -v.co.z
    bmesh.ops.reverse_faces(bm, faces=[g for g in dup["geom"] if isinstance(g, bmesh.types.BMFace)])
    # Kanten sys sammen med et smalt bånd (eggen har tykkelse 1,6 mm).
    bnd = [e for e in bm.edges if e.is_boundary]
    bmesh.ops.bridge_loops(bm, edges=bnd)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    me.materials.append(mat)
    bpy.context.scene.collection.objects.link(ob)
    me.uv_layers.new(name="UVMap")
    for p in me.polygons:
        p.use_smooth = True
    return ob


def solid_report(ob):
    """Åpne kanter (bare én flate) og kanter med mer enn to flater."""
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    bad = sum(1 for e in bm.edges if len(e.link_faces) != 2)
    bm.free()
    return bad


def build_weapon(name):
    import numpy as np

    sheet, length, grip_at = WEAPONS[name][:3]
    path, mask, metal = upright(name)
    h, w = mask.shape
    px = length / h
    gy = h * (1.0 - grip_at)
    mat = sheet_material(f"{name}_sheet", path, rough=0.5)
    parts, grip_d, mid = weapon_parts(name, mask, metal, px, gy, length)
    lo, hi = weapon_profile(mask)
    cx = float(mid[int(gy)])
    obs = []
    for i, (kind, rings, k, cm) in enumerate(parts):
        if kind == "head":
            ob = head_mesh(f"{name}_head", mat, mask, metal, px, gy, cx, cm[:2], RIDGE[name])
        else:
            caps = ("fan", "fan")
            ob = rings_object(f"{name}_{kind}{i}", mat, rings, caps=caps)
        bad = solid_report(ob)
        if bad:
            raise SystemExit(f"{name}: {ob.name} er ikke lukket ({bad} åpne kanter)")
        # Arket fra sida på alle flatene (bredden tilbake til bildets skala).
        me = ob.data
        src = me.uv_layers.get("src") or me.uv_layers.new(name="src")
        for poly in me.polygons:
            for li in poly.loop_indices:
                co = me.vertices[me.loops[li].vertex_index].co
                py = gy + co.x / px
                r = min(h - 1, max(0, int(py)))
                if kind == "head":
                    pxx = cx + co.y / px
                else:
                    off = (co.y - (cm[r] - mid[int(gy)]) * px) / (px * k)
                    if kind == "blade" and abs((hi[r] - mid[r]) - (mid[r] - lo[r])) > 8:
                        # Klinga får arket fra den hele sida speilet over
                        # aksen der eggen er malt med hakk (den andre sida
                        # er bakgrunn der).
                        off = abs(off) * (1.0 if hi[r] - mid[r] > mid[r] - lo[r] else -1.0)
                    pxx = cm[r] + off
                src.data[li].uv = ((pxx + 0.5) / w, 1.0 - (py + 0.5) / h)
        obs.append(ob)
    print(f"{name}: grepet {grip_d * 100:.1f} cm, {sum(len(o.data.polygons) for o in obs)} flater")
    if not GRIP_D[0] <= grip_d <= GRIP_D[1]:
        raise SystemExit(f"{name}: grepet er {grip_d * 100:.1f} cm (hånda holder {GRIP_D[0] * 100:.1f}–{GRIP_D[1] * 100:.1f})")
    # Øksa: eggen skal vende fram mot fienden (som knokene, +y i håndas
    # rom). Eggen er den lengste sida av hodet målt fra skaftets egen
    # midtlinje der hodet sitter, ikke fra grepet (skaftet er bøyd; runde
    # 18 målte fra grepet og snudde eggen bakover).
    if name == "axe":
        r0, r1 = next(p[3][:2] for p in parts if p[0] == "head")
        haft = next(p[3] for p in parts if p[0] == "grip")
        ys, xs = np.nonzero(mask[r0:r1] & metal[r0:r1])
        hc = haft[ys + r0]
        right, left = float(np.max(xs - hc)), float(np.max(hc - xs))
        sign = 1.0 if right >= left else -1.0
        if sign < 0:
            for ob in obs:
                ob.data.transform(Matrix.Scale(-1, 4, (0, 1, 0)))
                ob.data.flip_normals()
        # Eggen og skaftet der eggen er lengst ut, i håndas rom.
        k = int(np.argmax(sign * (xs - hc)))
        row = ys[k] + r0
        attach = Vector((-(gy - row) * px, (hc[k] - cx) * px * sign, 0.0))
        edge = Vector((-(gy - row) * px, (xs[k] - cx) * px * sign, 0.0))
        ang = edge_forward(attach, edge)
        print(f"{name}: eggen {ang:.0f}° fra retningen mot fienden i hvile")
        if ang > EDGE_FORWARD:
            raise SystemExit(f"{name}: eggen peker {ang:.0f}° bort fra fienden (høyst {EDGE_FORWARD:.0f})")
    for ob in obs:
        ob.data.transform(WEAPON)
        ob.data.update()
    return obs


EDGE_FORWARD = 40.0
TARGET = Vector((0.0, 2.0, 0.0))  # fienden: midt i bildet 2 m fram


def edge_forward(attach, edge, M=None):
    """Vinkelen mellom retningen fra skaftet ut til eggen og retningen mot
    fienden midt i bildet, i kamerarommet (hånda i hvile, eller `M`).
    Eieren i runde 18: eggen må peke mot monsteret som skal slås."""
    M = M or WEAPON
    a, e = M @ attach, M @ edge
    return math.degrees((e - a).angle(TARGET - a))


# --- Skjoldene ----------------------------------------------------------------

SHIELDS = {
    # navn: (forside, bakside, sentrum og radius/halv bredde og høyde i px, mål i m)
    "shield_round": ("shield_round_front.jpeg", "shield_back.jpeg"),
    "shield_tower": ("shield_tower_front.jpeg", "shield_tower_back.jpeg"),
}


HANDLE = 0.055  # håndtaket står så langt bak skjoldet


def shield_axes():
    """Skjoldets rom i kamerarommet: sentrum foran venstre knyttneve, fram
    (ut fra skjoldet), opp og høyre slik spilleren ser det."""
    g = at(LEFT, (0, 0, 0))
    # Opp langs håndtaket (grepet i neven), fram rett fram dreid ut mot
    # venstre, uavhengig av hvor underarmen peker.
    up = Vector(LEFT.col[0][:3]).normalized()
    want = Vector((-math.sin(math.radians(SHIELD_OUT)), math.cos(math.radians(SHIELD_OUT)), 0.0))
    fw = (want - up * want.dot(up)).normalized()
    rt = fw.cross(up).normalized()
    return g, fw, up, rt


def shield_angle(fw):
    """Hvor mange grader skjoldet vender ut mot venstre fra rett fram."""
    return math.degrees(math.atan2(-fw.x, fw.y))


def build_shield(name):
    front, back = SHIELDS[name]
    fmat = sheet_material(f"{name}_front", filled_sheet(os.path.join(REF, front)), rough=0.7)
    bmat = sheet_material(f"{name}_back", filled_sheet(os.path.join(REF, back)), rough=0.7)
    g, fw, up, rt = shield_axes()
    parts = []
    if name == "shield_round":
        R, T = 0.33, 0.022
        fc, fr = (1023, 1022), 944
        bc, br = (1024, 1025), 917
        # Skiva: sylinder om fram-aksen, med bulen foran.
        disc = cylinder("shield_disc", (0, 0, 0), R, T, fmat, rot=(math.pi / 2, 0, 0), vertices=40)
        disc.data.materials.append(bmat)
        parts.append(disc)
        boss = ellipsoid("shield_boss", (0, T / 2 + 0.005, 0), (0.15, 0.08, 0.15), fmat, 16, 8)
        parts.append(boss)
        rim = None
        bpy.ops.mesh.primitive_torus_add(major_radius=R, minor_radius=0.013, major_segments=40, minor_segments=8, rotation=(math.pi / 2, 0, 0))
        rim = bpy.context.active_object
        rim.name = "shield_rim"
        rim.data.materials.append(fmat)
        parts.append(rim)
        # Håndtaket bak (loddrett, så grepet står som i hånda).
        hb = cylinder("shield_handle", (0, -T / 2 - HANDLE, 0), 0.016, 0.16, bmat, vertices=10)
        parts.append(hb)
        for z in (-0.07, 0.07):
            parts.append(cylinder(f"shield_handle_post{z:+.2f}", (0, -T / 2 - HANDLE / 2, z), 0.012, HANDLE, bmat, rot=(math.pi / 2, 0, 0), vertices=8))

        def uv(p, n):
            # Rundskjoldet dreies 90°: håndtaket i arket ligger vannrett.
            u, v = p.z, -p.x
            if n.y >= 0:
                return (fc[0] - u / R * fr, fc[1] - v / R * fr)
            return (bc[0] + u / R * br, bc[1] - v / R * br)
    else:
        W, H, T = 0.62, 0.82, 0.03
        fbox = (340, 140, 1707, 1912)
        parts.append(_box("shield_board", (0, 0, 0), (W, T, H), fmat))
        parts[-1].data.materials.append(bmat)
        # Håndtaket og festene får baksida av arket (læret og jernet der).
        parts.append(cylinder("shield_handle", (0, -T / 2 - HANDLE, 0), 0.016, 0.2, bmat, vertices=10))
        for z in (-0.09, 0.09):
            parts.append(_box(f"shield_handle_post{z:+.2f}", (0, -T / 2 - HANDLE / 2, z), (0.025, HANDLE, 0.025), bmat))

        def uv(p, n):
            fx = (fbox[0] + fbox[2]) / 2
            fy = (fbox[1] + fbox[3]) / 2
            sx = (fbox[2] - fbox[0]) / W
            sy = (fbox[3] - fbox[1]) / H
            if n.y >= 0:
                return (fx - p.x * sx, fy - p.z * sy)
            return (fx + p.x * sx, fy - p.z * sy)
    for ob in parts:
        apply_mods(ob)
        me = ob.data
        src = me.uv_layers.get("src") or me.uv_layers.new(name="src")
        for poly in me.polygons:
            if len(me.materials) > 1:
                poly.material_index = 0 if poly.normal.y >= 0 else 1
            for li in poly.loop_indices:
                X, Y = uv(me.vertices[me.loops[li].vertex_index].co, poly.normal)
                src.data[li].uv = (X / 2048.0, 1.0 - Y / 2048.0)
    # Skjoldet står foran knyttneven: håndtaket i grepet, og knokene klar
    # av brettet (`HANDLE`).
    M = Matrix((rt, fw, up)).transposed().to_4x4()
    M.translation = g + fw * (T / 2 + HANDLE)
    for ob in parts:
        ob.data.transform(M)
        ob.data.update()


def _box(name, c, size, mat):
    bpy.ops.mesh.primitive_cube_add(location=c)
    ob = bpy.context.active_object
    ob.name = name
    ob.scale = (size[0] / 2, size[1] / 2, size[2] / 2)
    with bpy.context.temp_override(object=ob, selected_editable_objects=[ob]):
        bpy.ops.object.transform_apply(scale=True)
    ob.data.materials.append(mat)
    if not ob.data.uv_layers:
        ob.data.uv_layers.new(name="UVMap")
    return ob


def apply_mods(ob):
    bpy.context.view_layer.objects.active = ob
    for m in list(ob.modifiers):
        with bpy.context.temp_override(object=ob):
            bpy.ops.object.modifier_apply(modifier=m.name)
    ob.data.transform(ob.matrix_world)
    ob.matrix_world = Matrix.Identity(4)


# --- Klinga holder seg unna armene og kameraet ---------------------------------

BLADE = {"sword": (0.1, 0.8), "axe": (0.45, 0.62), "dagger": (0.06, 0.32)}  # fra grepet: klingas start og spiss (m langs −x)


def poses(rig, clips):
    """Hvert bilde i klippene: (klipp, bilde, {bein: skinnmatrise})."""
    ad = rig.animation_data
    for clip in clips:
        act = bpy.data.actions[clip]
        for t in ad.nla_tracks:
            t.mute = True
        ad.action = act
        if hasattr(act, "slots") and len(act.slots):
            ad.action_slot = act.slots[0]
        f0, f1 = (int(v) for v in act.frame_range)
        for f in range(f0, f1 + 1):
            bpy.context.scene.frame_set(f)
            bpy.context.view_layer.update()
            yield clip, f, {b.name: rig.matrix_world @ b.matrix @ rig.data.bones[b.name].matrix_local.inverted() for b in rig.pose.bones}
        ad.action = None
        for t in ad.nla_tracks:
            t.mute = False


ALL_CLIPS = ("idle", "swing1", "swing2", "swing3", "heavy", "block", "parry", "hurt")


def blade_clearance(rig, weapon="sword"):
    """Minste avstand fra klinga til underarmene, venstre hånd, skjoldet og
    kameraet i alle bildene av huggene (eieren om presten: våpenet går
    aldri gjennom kroppen). Kapsler rundt underarmene og knyttneven, en
    skive for skjoldet og en kule rundt øyet; klinga er linja fra grepet ut
    til spissen i høyre hånds rom."""
    a0, a1 = BLADE[weapon]
    worst = (1e9, "", 0)
    g, fw, up, rt = shield_axes()
    centre = g + fw * (0.03 / 2 + HANDLE)
    for clip, f, P in poses(rig, ("swing1", "swing2", "swing3", "heavy")):
        H = P["hand.R"]
        pts = [H @ (WEAPON @ Vector((-(a0 + (a1 - a0) * i / 30), 0, 0))) for i in range(31)]
        caps = []
        for side in ("R", "L"):
            fb = rig.pose.bones[f"forearm.{side}"]
            caps.append((f"underarm {side}", rig.matrix_world @ fb.head, rig.matrix_world @ fb.tail, 0.055))
        lh = rig.pose.bones["hand.L"]
        caps.append(("venstre hånd", rig.matrix_world @ lh.head, rig.matrix_world @ lh.tail, 0.06))
        caps.append(("kameraet", Vector((0, 0, 0)), Vector((0, 0, 0)), 0.12))
        HL = P["hand.L"]
        sc, sn = HL @ centre, (HL.to_3x3() @ fw).normalized()
        for p in pts:
            for nm, a, b, r in caps:
                ab = b - a
                t = 0.0 if ab.length < 1e-6 else max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
                d = (p - (a + ab * t)).length - r
                if d < worst[0]:
                    worst = (d, f"{clip} bilde {f}: {nm}", f)
            # Skjoldet: rundskjoldet og tårnskjoldet samlet (skiva med
            # radius 0,33 m og brettet 0,62 × 0,82 m), 4 cm tykt.
            q = p - sc
            ax = q.dot(sn)
            u, v = q.dot(HL.to_3x3() @ rt), q.dot(HL.to_3x3() @ up)
            out = min(max(0.0, math.hypot(u, v) - 0.33), math.hypot(max(0.0, abs(u) - 0.31), max(0.0, abs(v) - 0.41)))
            d = math.hypot(out, max(0.0, abs(ax) - 0.02)) if out > 0 else abs(ax) - 0.02
            if d < worst[0]:
                worst = (d, f"{clip} bilde {f}: skjoldet", f)
    return worst


def arms_leave_frame(rig):
    """Enden av overarmen (15 cm forbi skulderen) er utenfor bildet i hvert
    bilde av alle klippene (72° synsfelt, 16:9, og alt bak nærplanet):
    armene går ut av bildet og ender aldri midt i det (eieren i runde 16:
    «flyr det avkappede armer inn i bildet?»). Gir det verste: (hvor langt
    inne i bildet, i brøkdel av halve bildet; hvor)."""
    th = math.tan(math.radians(36))
    tw = th * 16 / 9
    worst = (-1e9, "")
    for clip, f, P in poses(rig, ALL_CLIPS):
        for side in ("R", "L"):
            F = upper_frame(side)
            L = ((RIGHT if side == "R" else LEFT) @ Vector((0, ELBOW_Y, 0)) - SHOULDER[side]).length + ARM_PAST
            R = 0.07
            for i in range(12):
                a = 2 * math.pi * i / 12
                p = P[f"upper_arm.{side}"] @ (F @ Vector((R * math.cos(a), -L, R * math.sin(a))))
                if p.y <= 0.05:
                    inside = -1.0
                else:
                    inside = 1.0 - max(abs(p.x) / (p.y * tw), abs(p.z) / (p.y * th))
                if inside > worst[0]:
                    worst = (inside, f"{clip} bilde {f}: {side}")
    return worst


WRIST_BEND = 35.0  # største bøy i håndleddet (grader)
WRIST_TWIST = 15.0  # største vridning mellom underarmen og hånda


def wrist_bend(rig):
    """Største bøy og vridning i håndleddet i hvert klipp, for begge hender:
    vinkelen mellom underarmen og hånda, og vridningen om underarmen. Et
    ekte håndledd bøyes lite og vris ikke (det gjør underarmen). Stopper
    eksporten over `WRIST_BEND` og `WRIST_TWIST` (eieren i runde 17)."""
    out = {}
    for clip, f, P in poses(rig, ALL_CLIPS):
        for side, M in (("R", RIGHT), ("L", LEFT)):
            y0 = (at(M, (0, WRIST_Y, 0)) - at(M, (0, ELBOW_Y, 0))).normalized()
            qf = P[f"forearm.{side}"].to_quaternion()
            qh = P[f"hand.{side}"].to_quaternion()
            rel = qf.inverted() @ qh
            bend = math.degrees((rel @ y0).angle(y0))
            tw = abs(math.degrees(2 * math.atan2(Vector(rel[1:]).dot(y0), rel[0])))
            tw = min(tw, 360 - tw)
            k = (clip, side)
            o = out.get(k, (0.0, 0.0, 0))
            out[k] = (max(o[0], bend), max(o[1], tw), f if bend > o[0] else o[2])
    for (clip, side), (bend, tw, f) in sorted(out.items()):
        if bend > WRIST_BEND or tw > WRIST_TWIST:
            raise SystemExit(f"håndleddet {side} i {clip} bilde {f}: bøyd {bend:.0f}° (høyst {WRIST_BEND:.0f}), vridd {tw:.0f}° (høyst {WRIST_TWIST:.0f})")
    return max(v[0] for v in out.values()), max(v[1] for v in out.values())


HITS = {"swing1": 6, "swing2": 6, "swing3": 7, "heavy": 13}
EDGE_LEADS = 0.5


def edge_leads(rig):
    """I treffet går eggen først: retningen eggen vender (knokene, +y i
    våpenrommet) mot retningen klinga beveger seg, som cos. Gir det minste
    (cos, klipp). Eieren i runde 18: eggen må peke mot monsteret som skal slås."""
    worst = (2.0, "")
    for clip, hit in HITS.items():
        tips, edge = {}, None
        for c, f, P in poses(rig, (clip,)):
            if f in (hit - 1, hit + 1):
                tips[f] = P["hand.R"] @ (WEAPON @ Vector((-0.6, 0, 0)))
            if f == hit:
                edge = (P["hand.R"].to_3x3() @ Vector(WEAPON.col[1][:3])).normalized()
        v = (tips[hit + 1] - tips[hit - 1]).normalized()
        cos = edge.dot(v)
        if cos < worst[0]:
            worst = (cos, clip)
    return worst


SPIN_MAX = 20.0  # største vridning av klinga om sin egen akse per bilde (grader)
HEAVY_ARC = 150.0  # det tunge hugget feier minst så mange grader fra ladningen
HEAVY_LUNGE = 2.0  # og kroppen kaster seg så mange ganger lenger fram


def blade_spin(rig):
    """Hvor mye klinga vris om sin egen akse fra ett bilde til det neste i
    huggene (eieren 13. oktober: «sverdet snurrer merkelig»). Stopper
    eksporten over `SPIN_MAX`. Gir (grader, klipp, bilde)."""
    worst = (0.0, "", 0)
    last = {}
    for clip, f, P in poses(rig, ("swing1", "swing2", "swing3", "heavy")):
        M = P["hand.R"].to_3x3() @ WEAPON.to_3x3()
        axis, edge = (M @ Vector((-1, 0, 0))).normalized(), (M @ Vector((0, 1, 0))).normalized()
        if clip in last and last[clip][0] == f - 1:
            pa, pe = last[clip][1], last[clip][2]
            # Vridningen: eggen før, lagt over i den nye aksen, mot eggen nå.
            pe = (pe - axis * pe.dot(axis)).normalized()
            spin = math.degrees(pe.angle(edge - axis * edge.dot(axis))) if pe.length > 0 else 0.0
            if os.environ.get("FRISTAD_DEBUG"):
                print(f"  {clip} {f}: vris {spin:.0f}°, klinga {math.degrees(pa.angle(axis)):.0f}°")
            if spin > worst[0]:
                worst = (spin, clip, f)
        last[clip] = (f, axis, edge)
    if worst[0] > SPIN_MAX:
        raise SystemExit(f"klinga vris {worst[0]:.0f}° på ett bilde i {worst[1]} bilde {worst[2]} (høyst {SPIN_MAX:.0f})")
    return worst


def heavy_distinct(rig):
    """Det tunge hugget skal kjennes (eieren 13. oktober: «ingen merkbar
    forskjell på vanlig slag og hardt slag»): klinga feier minst
    `HEAVY_ARC` grader langs veien fra ladningen (bilde 10) til
    gjennomføringen (18), hele hugget feier lenger enn hvert av de vanlige,
    og kroppen kaster seg minst `HEAVY_LUNGE` ganger så langt fram. Gir
    (bue, feid, mest feid i et vanlig hugg, utfall)."""
    blades, lunge = {}, {}
    for clip, f, P in poses(rig, ("swing1", "swing2", "swing3", "heavy")):
        blades.setdefault(clip, {})[f] = ((P["hand.R"] @ WEAPON).to_3x3() @ Vector((-1, 0, 0))).normalized()
        lunge[clip] = max(lunge.get(clip, 0.0), P["root"].translation.length)

    def along(d, a=None, b=None):
        fs = [f for f in sorted(d) if (a is None or f >= a) and (b is None or f <= b)]
        return math.degrees(sum(d[x].angle(d[y]) for x, y in zip(fs, fs[1:])))

    light = ("swing1", "swing2", "swing3")
    arc = along(blades["heavy"], 10, 18)
    swept = along(blades["heavy"])
    most = max(along(blades[c]) for c in light)
    ratio = lunge["heavy"] / max(1e-6, max(lunge[c] for c in light))
    if arc < HEAVY_ARC or swept <= most or ratio < HEAVY_LUNGE:
        raise SystemExit(f"det tunge hugget skiller seg for lite ut: bue {arc:.0f}° (minst {HEAVY_ARC:.0f}), feid {swept:.0f}° mot {most:.0f}° i de vanlige, utfallet {ratio:.1f}× (minst {HEAVY_LUNGE})")
    return arc, swept, most, ratio


REACH_OUT = 0.02  # i treffet strekkes armen så mye lenger ut (m)
SHOULDER_DRIVES = 1.5  # skulderen dreier overarmen minst så mange ganger mer enn albuen dreier underarmen


def shoulder_drives(rig):
    """Huggene drives av skulderen (eieren 14. oktober: «det ser ut som at
    albuen roterer i stedet for at svinget er drevet av skulderleddet»):
    gjennom hvert hugg dreies overarmen (fra skulderen) minst
    `SHOULDER_DRIVES` ganger så mye som underarmen dreies mot overarmen
    (albuen som bøyes og vris), summert bilde for bilde. Gir det verste
    (forhold, klipp, overarm, albue)."""
    from mathutils import Quaternion  # noqa: F401
    up, rel = {}, {}
    for clip, f, P in poses(rig, ("swing1", "swing2", "swing3", "heavy")):
        qu = P["upper_arm.R"].to_quaternion()
        qf = P["forearm.R"].to_quaternion()
        up.setdefault(clip, []).append(qu)
        rel.setdefault(clip, []).append(qu.inverted() @ qf)

    def swept(qs):
        return math.degrees(sum(a.rotation_difference(b).angle for a, b in zip(qs, qs[1:])))

    worst = (1e9, "", 0.0, 0.0)
    for clip in up:
        u, e = swept(up[clip]), swept(rel[clip])
        r = u / max(1e-6, e)
        if os.environ.get("FRISTAD_DEBUG"):
            print(f"  {clip}: overarmen {u:.0f}°, albuen {e:.0f}° ({r:.1f}×)")
        if r < worst[0]:
            worst = (r, clip, u, e)
    return worst


def shield_check(rig):
    """Skjoldet i hvile vender 0–20° ut mot venstre fra rett fram, og blokken
    og paraden dreier det aldri inn forbi rett fram (eieren i runde 16)."""
    g, fw, up, rt = shield_axes()
    rest = shield_angle(fw)
    if not 0.0 <= rest <= 20.0:
        raise SystemExit(f"skjoldet vender {rest:.0f}° ut i hvile (skal være 0–20°)")
    worst = 1e9
    for clip, f, P in poses(rig, ("block", "parry", "hurt", "idle")):
        n = (P["hand.L"].to_3x3() @ fw).normalized()
        worst = min(worst, shield_angle(n))
    if worst < -2.0:
        raise SystemExit(f"skjoldet dreies {-worst:.0f}° inn mot midten")
    return rest, worst


def build_rig_and_clips():
    rig = armature("arms_rig", BONES)
    clips(rig)
    return rig


if __name__ == "__main__":
    args = sys.argv[1:]
    kinds = ["arms", "arms_leather", "arms_mail"]
    which = [a for a in args if a in kinds or a in WEAPONS or a in SHIELDS] or kinds + list(WEAPONS) + list(SHIELDS)
    os.makedirs(OUT, exist_ok=True)
    for name in which:
        clear()
        if name in kinds:
            if name == "arms_mail":
                cover = mail_coverage(name)
                print(f"brynja: {cover * 100:.0f} % av underarmen")
                if cover < MAIL_COVER:
                    raise SystemExit(f"brynja dekker {cover * 100:.0f} % av underarmen (minst {MAIL_COVER * 100:.0f})")
            rigid = build_arms(name)
            rig = build_rig_and_clips()
            for w in WEAPONS:
                d, where, _ = blade_clearance(rig, w)
                print(f"{w}: klinga minst {d * 100:.0f} cm unna ({where})")
                if d < 0.0:
                    raise SystemExit(f"{w}: klinga går gjennom {where}")
            inside, where = arms_leave_frame(rig)
            print(f"armene: enden minst {-inside:.2f} utenfor bildet ({where})")
            if inside > 0.0:
                raise SystemExit(f"armen ender inne i bildet ({where})")
            rest, worst = shield_check(rig)
            print(f"skjoldet: {rest:.0f}° ut i hvile, minst {worst:.0f}° i blokken")
            bend, tw = wrist_bend(rig)
            print(f"håndleddet: høyst {bend:.0f}° bøyd og {tw:.0f}° vridd")
            spin, sc, sf = blade_spin(rig)
            print(f"klinga vris høyst {spin:.0f}° per bilde ({sc} bilde {sf})")
            arc, swept, most, ratio = heavy_distinct(rig)
            print(f"det tunge hugget: bue {arc:.0f}° fra ladningen, feier {swept:.0f}° (de vanlige høyst {most:.0f}°), utfallet {ratio:.1f}×")
            r, sc, u, e = shoulder_drives(rig)
            print(f"skulderen driver: overarmen {u:.0f}° mot albuen {e:.0f}° i {sc} ({r:.1f}×, minst {SHOULDER_DRIVES})")
            if r < SHOULDER_DRIVES:
                raise SystemExit(f"albuen driver hugget i {sc}: overarmen dreies {u:.0f}°, underarmen mot overarmen {e:.0f}° ({r:.1f}×, minst {SHOULDER_DRIVES})")
            cos, clip = edge_leads(rig)
            print(f"eggen går først i treffet: cos minst {cos:.2f} ({clip})")
            if cos < EDGE_LEADS:
                raise SystemExit(f"eggen går ikke først i treffet i {clip} (cos {cos:.2f}, minst {EDGE_LEADS})")
            if os.environ.get("FRISTAD_CHECK_ONLY"):
                continue
            bake_textured(f"{OUT}/{name}.glb", 1024, keep=("glass", "magic"), armature=rig, rigid=rigid, skin={"blend": 0.03, "radii": {"root": -1.0}})
        elif name in WEAPONS:
            build_weapon(name)
            # Atlaset fyller lite av flaten (lange, smale øyer), så våpnene bakes i 2048.
            bake_textured(f"{OUT}/{name}.glb", 2048, keep=("glass", "magic"))
        else:
            build_shield(name)
            bake_textured(f"{OUT}/{name}.glb", 1024, keep=("glass", "magic"))
