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
from common import Clip, armature, bake_textured, clear, cylinder, ellipsoid, material, tube, wave  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from sheetproj import ROOT, filled_sheet, sheet_material  # noqa: E402

OUT = "assets/models/arms"
REF = os.path.join(ROOT, "assets", "textures", "weapons_ref")
FPS = 24
# Sekundene i klippene der treffet lander (speiles i domain/combat.rs).
SWING_HIT = (5 / FPS, 5 / FPS, 7 / FPS)
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
# Hvor mye tykkere ermet er over armen per sett (lærarmbeskytter, brynje).
BULK = {"arms": 0.0, "arms_leather": 0.004, "arms_mail": 0.006}


def img_y(y, gy):
    """Raden i arket for y i håndas rom: hånda i sann målestokk, og
    underarmen strukket fra håndleddet til nederst i arket (arket viser bare
    en del av underarmen)."""
    if y >= WRIST_Y:
        return gy - y / GRIP_PX
    t = min(1.0, (WRIST_Y - y) / FOREARM)
    return gy + WRIST_IMG + t * (SLEEVE_IMG - WRIST_IMG)


def project_hand(ob, gy, dx, mirror):
    """Uv-laget «src» for en del av armen bygd i håndas rom: ovenfra (håndbaken,
    +z), nedenfra (håndflaten, −z) eller fra tommelsida (±x), etter hvilken
    vei flaten vender. `mirror`: venstre hånd (bygd speilet, tommelen mot
    +x), som har sin egen visning ovenfra i arket."""
    me = ob.data
    src = me.uv_layers.get("src") or me.uv_layers.new(name="src")
    s = -1.0 if mirror else 1.0
    for poly in me.polygons:
        n = poly.normal
        ax, az = abs(n.x), abs(n.z)
        for li in poly.loop_indices:
            p = me.vertices[me.loops[li].vertex_index].co
            Y = img_y(p.y, gy)
            if az >= ax * 0.8 and n.z > 0:
                X = (COL_LEFT if mirror else COL_TOP) + dx + p.x / GRIP_PX
            elif az >= ax * 0.8:
                X = COL_BELOW + dx - s * p.x / GRIP_PX
            else:
                X = COL_SIDE + dx - p.z / GRIP_PX
            src.data[li].uv = (X / 2000.0, 1.0 - Y / 2000.0)


def loft_y(name, mat, sections, n=16):
    """En kropp loftet langs y: snittene (y, halv bredde i x, nederst z,
    øverst z) er ellipser."""
    bm = bmesh.new()
    rings = []
    for y, hw, zb, zt in sections:
        zc, hz = (zb + zt) / 2, (zt - zb) / 2
        rings.append([bm.verts.new((hw * math.cos(2 * math.pi * i / n), y, zc + hz * math.sin(2 * math.pi * i / n))) for i in range(n)])
    for a, b in zip(rings, rings[1:]):
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((a[i], a[j], b[j], b[i]))
    bm.faces.new(rings[0])
    bm.faces.new(rings[-1][::-1])
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


def finger_arc(name, mat, x, R, r, a0, a1, n=8, m=8):
    """En krøllet finger: en del av en ring om grepets akse (x), fra vinkelen
    a0 til a1 i (y, z)-planet (0 er rett fram, +90 er opp)."""
    bm = bmesh.new()
    rings = []
    for i in range(n + 1):
        a = math.radians(a0 + (a1 - a0) * i / n)
        c = Vector((x, R * math.cos(a), R * math.sin(a)))
        out = Vector((0, math.cos(a), math.sin(a)))
        # Fingertuppen smalner av.
        rr = r * (1.0 - 0.25 * max(0.0, (i / n - 0.7) / 0.3))
        rings.append([bm.verts.new(c + (Vector((1, 0, 0)) * math.cos(2 * math.pi * k / m) + out * math.sin(2 * math.pi * k / m)) * rr) for k in range(m)])
    for a, b in zip(rings, rings[1:]):
        for k in range(m):
            j = (k + 1) % m
            bm.faces.new((a[k], a[j], b[j], b[k]))
    bm.faces.new(rings[0][::-1])
    bm.faces.new(rings[-1])
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


def hand_parts(mat, kind, mirror):
    """Knyttneven og underarmen i håndas rom. Returnerer (navn, del) der
    delen er `hand`, `forearm` eller `upper`."""
    s = -1.0 if mirror else 1.0
    b = BULK[kind]
    parts = []
    # Håndbaken og håndflaten, fra håndleddet til knokene.
    parts.append(("hand", loft_y("hand_back", mat, [(WRIST_Y, 0.031, -0.024, 0.024), (-0.06, 0.04, -0.03, 0.032), (-0.025, 0.048, -0.034, 0.042), (0.0, 0.05, -0.03, 0.046), (0.012, 0.048, -0.015, 0.044)])))
    # Fire fingre krøllet rundt grepet (pekefingeren på tommelsida, −x).
    for k, (x, w) in enumerate(((-0.034, 0.0125), (-0.011, 0.013), (0.012, 0.0125), (0.033, 0.011))):
        parts.append(("hand", finger_arc(f"hand_finger{k}", mat, s * x, 0.031, w, 70, -150)))
    # Tommelen: fra håndroten over pekefingeren og langfingeren på undersida.
    t0, t1, t2 = Vector((s * -0.04, -0.055, -0.012)), Vector((s * -0.058, -0.015, -0.03)), Vector((s * -0.03, 0.02, -0.04))
    parts.append(("hand", tube("hand_thumb_a", t0, t1, 0.016, mat, 10)))
    parts.append(("hand", tube("hand_thumb_b", t1, t2, 0.013, mat, 10)))
    parts.append(("hand", ellipsoid("hand_thumb_k", t1, (0.032, 0.032, 0.03), mat, 10, 6)))
    # Underarmen og ermet, fra håndleddet til albuen og videre opp.
    fa = [(WRIST_Y + 0.004, 0.03 + b, -0.024 - b, 0.025 + b), (WRIST_Y - 0.03, 0.033 + b, -0.028 - b, 0.028 + b), (WRIST_Y - 0.11, 0.04 + b, -0.034 - b, 0.034 + b), (WRIST_Y - 0.19, 0.046 + b, -0.039 - b, 0.039 + b), (ELBOW_Y, 0.05 + b, -0.043 - b, 0.043 + b)]
    parts.append(("forearm", loft_y("forearm_skin", mat, fa)))
    # Ermet: den opprullede kanten og ermet videre over albuen.
    parts.append(("forearm", loft_y("forearm_cuff", mat, [(WRIST_Y - 0.13, 0.045, -0.038, 0.04), (WRIST_Y - 0.15, 0.056, -0.05, 0.05), (WRIST_Y - 0.2, 0.058, -0.052, 0.052), (ELBOW_Y, 0.056, -0.05, 0.05)])))
    parts.append(("upper", loft_y("upper_sleeve", mat, [(ELBOW_Y + 0.02, 0.055, -0.05, 0.05), (ELBOW_Y - 0.12, 0.06, -0.055, 0.055), (ELBOW_Y - 0.3, 0.062, -0.058, 0.058)])))
    return parts


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


# Høyre hånd: klinga peker opp og litt fram og innover (−x i håndas rom),
# underarmen går fram fra nede til høyre i bildet.
RIGHT = frame((0.22, 0.5, -0.27), (0.3, -0.1, -0.95), (-0.55, 1.0, 0.25))
# Venstre hånd: grepet på skjoldet står loddrett (tommelen opp), underarmen
# fram fra nede til venstre, håndbaken ut mot venstre.
LEFT = frame((-0.34, 0.48, -0.42), (0.05, -0.12, 1.0), (0.32, 1.0, 0.06))
SHOULDER = {"R": Vector((0.2, -0.12, -0.3)), "L": Vector((-0.2, -0.12, -0.3))}


def at(M, p):
    return M @ Vector(p)


def bones():
    out = [("root", (0, 0, 0), (0, 0.1, 0), None)]
    for side, M in (("R", RIGHT), ("L", LEFT)):
        e, w, g = at(M, (0, ELBOW_Y, 0)), at(M, (0, WRIST_Y, 0)), at(M, (0, 0.03, 0))
        out += [(f"upper_arm.{side}", tuple(SHOULDER[side]), tuple(e), "root"), (f"forearm.{side}", tuple(e), tuple(w), f"upper_arm.{side}"), (f"hand.{side}", tuple(w), tuple(g), f"forearm.{side}")]
    return out


BONES = bones()


def build_arms(kind):
    gy, dx = GRIP_ROWS[kind][0], GRIP_ROWS[kind][1]
    mat = sheet_material(f"{kind}_sheet", filled_sheet(GRIP_SHEET))
    rigid = {}
    for side, M in (("R", RIGHT), ("L", LEFT)):
        mirror = side == "L"
        for part, ob in hand_parts(mat, kind, mirror):
            ob.name = f"{ob.name}_{side}"
            # Rørene og kulene har plassen sin i objektet; den legges i
            # hjørnene før arket og hvileposen (ellers ble tommelen igjen).
            ob.data.transform(ob.matrix_world)
            ob.matrix_world = Matrix.Identity(4)
            project_hand(ob, gy, dx, mirror)
            ob.data.transform(M)
            ob.data.update()
            rigid[ob.name] = {"hand": f"hand.{side}", "forearm": f"forearm.{side}", "upper": f"upper_arm.{side}"}[part]
    return rigid


# --- Klippene ------------------------------------------------------------------


def clips(rig):
    # Idle: pusten løfter armene litt, og klinga vugger.
    c = Clip(rig, "idle", 48, FPS)
    c.rest_all(0)
    for f, v in wave(48, 48, 1.0):
        c.loc("root", f, (0, 0.004 * v, 0.008 * v))
        c.rot("upper_arm.R", f, x=1.5 * v)
        c.rot("upper_arm.L", f, x=-1.2 * v, z=1.0 * v)
    c.finish()
    # Kombo: hugg fra høyre, hugg fra venstre, hugg ovenfra. Hvert
    # nøkkelbilde sier hvor grepet skal være og hvilken vei klinga peker i
    # kamerarommet (x høyre, y fram, z opp); overarmen dreies så grepet når
    # dit, og hånda så klinga peker dit med flatsida mot kameraet. Treffet er
    # i bilde 5, 5 og 7 (`SWING_HIT`), og klinga krysser midten av bildet
    # der fienden står.
    from mathutils import Quaternion

    S = SHOULDER["R"]
    G0 = RIGHT.translation.copy()
    B0 = -Vector(RIGHT.col[0][:3]).normalized()  # klinga i hvile
    N0 = Vector(RIGHT.col[2][:3]).normalized()  # flatsida i hvile

    W0 = at(RIGHT, (0, WRIST_Y, 0))
    E0 = at(RIGHT, (0, ELBOW_Y, 0))

    def aim(c, f, grip, blade, root=(0, 0, 0)):
        d = Vector(blade).normalized()
        rb = B0.rotation_difference(d)
        # Rull klinga om sin egen akse så flatsida vender mot øyet.
        n = rb @ N0
        v = -(Vector(grip) + d * 0.4).normalized()
        v = v - d * v.dot(d)
        if v.length > 1e-6:
            v.normalize()
            n = n - d * n.dot(d)
            n.normalize()
            ang = math.atan2(d.dot(n.cross(v)), n.dot(v))
            if abs(ang) > math.pi / 2:
                ang -= math.copysign(math.pi, ang)  # begge flatene er like
            rb = Quaternion(d, ang) @ rb
        # Håndleddet der grepet havner når hånda er dreid slik, og albuen
        # med to-ledds IK (albuen ned og ut).
        w = Vector(grip) - rb @ (G0 - W0)
        a, b = (E0 - S).length, (W0 - E0).length
        dv = w - S
        dist = min(max(dv.length, abs(a - b) + 0.01), a + b - 0.005)
        u = dv.normalized()
        pole = Vector((0.35, -0.2, -1.0))
        pole = (pole - u * pole.dot(u)).normalized()
        cos_a = (a * a + dist * dist - b * b) / (2 * a * dist)
        al = math.acos(max(-1.0, min(1.0, cos_a)))
        e = S + (u * math.cos(al) + pole * math.sin(al)) * a
        w = S + u * dist
        ru = (E0 - S).rotation_difference(e - S)
        rf = (ru @ (W0 - E0)).rotation_difference(w - e) @ ru
        c.rotq("upper_arm.R", f, ru)
        c.rotq("forearm.R", f, ru.inverted() @ rf)
        c.rotq("hand.R", f, rf.inverted() @ rb)
        c.loc("root", f, root)

    def swing(name, length, keys):
        c = Clip(rig, name, length, FPS)
        c.rest_all(0)
        for f, grip, blade, root in keys:
            aim(c, f, grip, blade, root)
        for b in ("upper_arm.R", "forearm.R", "hand.R", "upper_arm.L", "forearm.L"):
            c.rot(b, length)
        c.loc("root", length, (0, 0, 0))
        c.finish()

    # Alle nøkkelbildene står innenfor synsfeltet (72°): grepet rundt en
    # halv meter fram og høyst 0,25 m over eller under øyet.
    swing("swing1", 12, [
        (3, (0.34, 0.55, 0.0), (0.6, 0.3, 0.75), (0.03, -0.02, 0.02)),
        (5, (0.12, 0.5, -0.1), (-0.95, 0.25, 0.15), (-0.02, 0.06, 0.0)),
        (8, (0.0, 0.5, -0.12), (-0.55, 0.7, -0.3), (-0.03, 0.04, -0.01)),
    ])
    swing("swing2", 12, [
        (3, (0.0, 0.55, 0.08), (-0.4, 0.25, 0.88), (-0.03, -0.02, 0.02)),
        (5, (0.1, 0.5, -0.1), (0.95, 0.25, 0.1), (0.03, 0.06, 0.0)),
        (8, (0.32, 0.5, -0.12), (0.75, 0.6, -0.2), (0.04, 0.04, -0.01)),
    ])
    swing("swing3", 16, [
        (4, (0.22, 0.5, 0.18), (0.05, 0.15, 0.98), (0, -0.03, 0.03)),
        (7, (0.08, 0.52, 0.05), (0.0, 0.75, 0.65), (0, 0.08, -0.03)),
        (10, (0.1, 0.52, -0.15), (-0.05, 0.95, -0.3), (0, 0.05, -0.04)),
    ])
    # Tungt hugg: lang opptrekk over skulderen (holdes til bilde 10), og et
    # hugg ned som drar kroppen med.
    swing("heavy", 24, [
        (10, (0.3, 0.55, 0.05), (0.7, 0.35, 0.6), (0.02, -0.04, 0.02)),
        (13, (0.08, 0.52, 0.05), (0.0, 0.75, 0.65), (-0.02, 0.1, -0.05)),
        (17, (0.1, 0.52, -0.15), (-0.05, 0.95, -0.3), (-0.01, 0.07, -0.06)),
    ])
    # Blokk: skjoldet opp foran ansiktet, holdes på siste bilde.
    c = Clip(rig, "block", 8, FPS)
    c.rest_all(0)
    c.rot("upper_arm.L", 8, x=24, z=-30)
    c.rot("forearm.L", 8, x=6, z=-10)
    c.rot("upper_arm.R", 8, x=-8, z=-12)
    c.finish()
    # Parade: fra blokken støter skjoldet fram og kommer tilbake.
    c = Clip(rig, "parry", 12, FPS)
    c.rest_all(0)
    c.rot("upper_arm.L", 0, x=24, z=-30)
    c.rot("forearm.L", 0, x=6, z=-10)
    c.rot("upper_arm.L", 4, x=30, z=-34)
    c.rot("forearm.L", 4, x=14, z=-14)
    c.loc("root", 4, (-0.03, 0.08, 0.0))
    c.rot("upper_arm.L", 12, x=24, z=-30)
    c.rot("forearm.L", 12, x=6, z=-10)
    c.loc("root", 12, (0, 0, 0))
    c.finish()
    # Treffes: armene slenges, skjoldet kastes litt inn.
    c = Clip(rig, "hurt", 10, FPS)
    c.rest_all(0)
    c.loc("root", 2, (0.04, -0.06, -0.04))
    c.rot("upper_arm.R", 2, x=-8, z=8)
    c.rot("upper_arm.L", 2, x=-6, z=6)
    c.loc("root", 10, (0, 0, 0))
    c.rot("upper_arm.R", 10)
    c.rot("upper_arm.L", 10)
    c.finish()
    bpy.context.scene.frame_end = 48


# --- Våpnene ------------------------------------------------------------------
#
# Hvert våpen er tegnet på skrå i arket. Bildet dreies så våpenet står
# loddrett med spissen opp, og formen blåses opp fra omrisset: metallet blir
# en tynn kile med rygg midt på (klinga, parerstanga, øksebladet), tre og lær
# runde (grepet, skaftet). Begge flatene får bildet.

WEAPONS = {
    # navn: (ark, lengde m, grepets midte i brøkdel fra nederst, metall: helling og største halvtykkelse)
    "sword": ("weapon_sword.jpeg", 0.95, 0.16, (0.2, 0.0045)),
    "axe": ("weapon_axe.jpeg", 0.72, 0.13, (0.3, 0.009)),
    "dagger": ("weapon_dagger.jpeg", 0.42, 0.24, (0.2, 0.004)),
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


def blown(name, mat, mask, metal, px, gy_px, cx_px, slope, cap):
    """Formen blåst opp fra omrisset: omrisset (glatt, fra masken) fylles
    med trekanter som deles opp, og hvert hjørne løftes etter avstanden til
    kanten (kile for metall, rund for tre og lær); undersida er speilet.
    Bygd i håndas rom: aksen langs −x (spissen), bredden langs y og
    tykkelsen langs z."""
    import numpy as np
    from scipy import ndimage
    from skimage import measure

    h, w = mask.shape
    dist = ndimage.distance_transform_edt(mask)
    # Rundingen på tre og lær etter bredden av skaftet selv, ikke bladet.
    round_d = np.where(metal, 0.0, dist)
    ridge = ndimage.maximum_filter(round_d, size=int(max(3, round_d.max() * 2 + 1)))
    outline = max(measure.find_contours(np.pad(mask, 1).astype(float), 0.5), key=len) - 1.0
    outline = measure.approximate_polygon(outline, tolerance=1.2)[:-1]

    def pos(py, pxx):
        return Vector((-(gy_px - py) * px, (pxx - cx_px) * px, 0.0))

    bm = bmesh.new()
    vs = [bm.verts.new(pos(py, pxx)) for py, pxx in outline]
    face = bm.faces.new(vs)
    bmesh.ops.triangulate(bm, faces=[face], quad_method="BEAUTY", ngon_method="BEAUTY")
    for _ in range(3):
        bmesh.ops.subdivide_edges(bm, edges=bm.edges[:], cuts=1, use_grid_fill=True)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    for f in bm.faces:
        if f.normal.z < 0:
            f.normal_flip()
    edge = {v for e in bm.edges if e.is_boundary for v in e.verts}

    def thick(v):
        py = min(h - 1, max(0, int(round(gy_px + v.co.x / px))))
        pxx = min(w - 1, max(0, int(round(cx_px + v.co.y / px))))
        if v in edge or not mask[py, pxx]:
            return 0.0
        d = dist[py, pxx] * px
        if metal[py, pxx]:
            return min(slope * d, cap)
        R = max(ridge[py, pxx] * px, 1e-4)
        return 0.92 * math.sqrt(max(0.0, d * (2 * R - d)))

    for v in bm.verts:
        v.co.z = thick(v)
    # Undersida: en speilet kopi, sydd sammen langs kanten.
    dup = bmesh.ops.duplicate(bm, geom=bm.verts[:] + bm.edges[:] + bm.faces[:])
    for v in [g for g in dup["geom"] if isinstance(g, bmesh.types.BMVert)]:
        v.co.z = -v.co.z
    bmesh.ops.reverse_faces(bm, faces=[g for g in dup["geom"] if isinstance(g, bmesh.types.BMFace)])
    bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=1e-6)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.uv_layers.new(name="UVMap")
    src = me.uv_layers.new(name="src")
    for poly in me.polygons:
        for li in poly.loop_indices:
            co = me.vertices[me.loops[li].vertex_index].co
            pxx = co.y / px + cx_px
            py = gy_px + co.x / px
            src.data[li].uv = ((pxx + 0.5) / w, 1.0 - (py + 0.5) / h)
    ob = bpy.data.objects.new(name, me)
    me.materials.append(mat)
    bpy.context.scene.collection.objects.link(ob)
    for p in me.polygons:
        p.use_smooth = True
    # Slankes til rundt 1 500 trekanter.
    mod = ob.modifiers.new("dec", "DECIMATE")
    mod.ratio = min(1.0, 1500 / max(1, len(me.polygons)))
    bpy.context.view_layer.objects.active = ob
    with bpy.context.temp_override(object=ob):
        bpy.ops.object.modifier_apply(modifier=mod.name)
    return ob


def build_weapon(name):
    import numpy as np

    sheet, length, grip_at, (slope, cap) = WEAPONS[name]
    path, mask, metal = upright(name)
    h, w = mask.shape
    px = length / h
    gy = h * (1.0 - grip_at)
    cols = np.nonzero(mask[int(gy)])[0]
    cx = cols.mean() if len(cols) else w / 2
    mat = sheet_material(f"{name}_sheet", path, rough=0.5)
    ob = blown(name, mat, mask, metal, px, gy, cx, slope, cap)
    # Øksa: bladet skal vende fram (som knokene, +y i håndas rom).
    if name == "axe":
        upper = mask[: int(h * 0.3)]
        ys, xs = np.nonzero(upper)
        if xs.mean() < cx:
            ob.data.transform(Matrix.Scale(-1, 4, (0, 1, 0)))
            ob.data.flip_normals()
    ob.data.transform(RIGHT)
    ob.data.update()
    return ob


# --- Skjoldene ----------------------------------------------------------------

SHIELDS = {
    # navn: (forside, bakside, sentrum og radius/halv bredde og høyde i px, mål i m)
    "shield_round": ("shield_round_front.jpeg", "shield_back.jpeg"),
    "shield_tower": ("shield_tower_front.jpeg", "shield_tower_back.jpeg"),
}


def shield_axes():
    """Skjoldets rom i kamerarommet: sentrum foran venstre knyttneve, fram
    (ut fra skjoldet), opp og høyre slik spilleren ser det."""
    g = at(LEFT, (0, 0, 0))
    fw = Vector(LEFT.col[1][:3]).normalized()
    up = Vector(LEFT.col[0][:3]).normalized()
    rt = fw.cross(up).normalized()
    return g, fw, up, rt


def build_shield(name):
    front, back = SHIELDS[name]
    fmat = sheet_material(f"{name}_front", filled_sheet(os.path.join(REF, front)), rough=0.7)
    bmat = sheet_material(f"{name}_back", filled_sheet(os.path.join(REF, back)), rough=0.7)
    iron = material("iron", (0.32, 0.32, 0.34), 0.5, 1.0)
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
        hb = cylinder("shield_handle", (0, -T / 2 - 0.03, 0), 0.016, 0.16, bmat, vertices=10)
        parts.append(hb)
        for z in (-0.07, 0.07):
            parts.append(cylinder(f"shield_handle_post{z:+.2f}", (0, -T / 2 - 0.015, z), 0.012, 0.03, bmat, rot=(math.pi / 2, 0, 0), vertices=8))

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
        parts.append(_box("shield_handle", (0, -T / 2 - 0.03, 0), (0.03, 0.02, 0.2), iron))
        for z in (-0.09, 0.09):
            parts.append(_box(f"shield_handle_post{z:+.2f}", (0, -T / 2 - 0.015, z), (0.025, 0.03, 0.025), iron))

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
    # Skjoldet står foran knyttneven: håndtaket i grepet.
    M = Matrix((rt, fw, up)).transposed().to_4x4()
    M.translation = g + fw * (0.05 + 0.011)
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


def blade_clearance(rig, weapon="sword"):
    """Minste avstand fra klinga til underarmene, venstre hånd og kameraet i
    alle bildene av huggene (eieren om presten: våpenet går aldri gjennom
    kroppen). Kapsler rundt underarmene og knyttneven, og en kule rundt
    øyet; klinga er linja fra grepet ut til spissen i høyre hånds rom."""
    a0, a1 = BLADE[weapon]
    worst = (1e9, "", 0)
    for clip in ("swing1", "swing2", "swing3", "heavy"):
        act = bpy.data.actions[clip]
        ad = rig.animation_data
        for t in ad.nla_tracks:
            t.mute = True
        ad.action = act
        if hasattr(act, "slots") and len(act.slots):
            ad.action_slot = act.slots[0]
        f0, f1 = (int(v) for v in act.frame_range)
        for f in range(f0, f1 + 1):
            bpy.context.scene.frame_set(f)
            bpy.context.view_layer.update()
            P = {b.name: rig.matrix_world @ b.matrix @ rig.data.bones[b.name].matrix_local.inverted() for b in rig.pose.bones}
            H = P["hand.R"]
            pts = [H @ (RIGHT @ Vector((-(a0 + (a1 - a0) * i / 30), 0, 0))) for i in range(31)]
            caps = []
            for side in ("R", "L"):
                fb = rig.pose.bones[f"forearm.{side}"]
                caps.append((f"underarm {side}", rig.matrix_world @ fb.head, rig.matrix_world @ fb.tail, 0.055))
            lh = rig.pose.bones["hand.L"]
            caps.append(("venstre hånd", rig.matrix_world @ lh.head, rig.matrix_world @ lh.tail, 0.06))
            caps.append(("kameraet", Vector((0, 0, 0)), Vector((0, 0, 0)), 0.12))
            for p in pts:
                for nm, a, b, r in caps:
                    ab = b - a
                    t = 0.0 if ab.length < 1e-6 else max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
                    d = (p - (a + ab * t)).length - r
                    if d < worst[0]:
                        worst = (d, f"{clip} bilde {f}: {nm}", f)
        ad.action = None
        for t in ad.nla_tracks:
            t.mute = False
    return worst


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
            rigid = build_arms(name)
            rig = build_rig_and_clips()
            d, where, _ = blade_clearance(rig)
            print(f"klinga: minst {d * 100:.0f} cm unna ({where})")
            if d < 0.0:
                raise SystemExit(f"klinga går gjennom {where}")
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
