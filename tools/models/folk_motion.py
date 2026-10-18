"""Bevegelsene til menneskene i byen: gange, stopp, snu, hvile med pust og
vektskift, små gester, prat og vinking, regnet ut bilde for bilde på
spillets 18 bein (samme skjelett som npc.py, people.py og folk.py).

Bruk fra en figur:

    from folk_motion import clips
    rig = armature(f"{name}_rig", game_bones(j))
    clips(rig, "smith", grips={"R": "hang"}, body=kroppen, tools={"R": [hammeren]})

`style` er en av `STYLES` (eller en dict), `grips` sier hva hver hånd
holder: «hang» (hammer, øks, krus etter hanken: henger og pendler litt),
«heavy» (bøtte: henger rett ned, armen ut fra kroppen, kroppen lener seg
mot), «long» (spyd, høygaffel, stav: loddrett ved siden av kroppen,
hånda står nesten stille), «front» (noe som bæres foran, som en bok).
`body` (kroppsobjektet med vektgrupper per bein) gir målene på føttene og
lårene, og `tools` (objektene som henger på hånda) gir punktene som
holdes klar av beina og hoftene.

Skjelettet har ingen fingre: grepet er formet i modellen (MakeHumans
fingerbein bøyes rundt skaftet før vektene samles på `hand.*`), og
redskapet henger stivt på hånda (`rigid` i bake_textured). Klippene
holder derfor hånda i den samme retningen som i hvileposen når den bærer
noe, så skaftet ligger der fingrene er formet rundt det.

Hvordan det regnes ut:

* **Føttene** settes ned på bakken i verden (`Plant`: når foten lander,
  hvor, og når den løftes), og kroppen beveger seg over dem (`Track`:
  hvor kroppen er og hvor den vender, over tid). Klippet lages på stedet,
  så en fot som står, glir bakover i modellen med nøyaktig den farten
  spillet flytter figuren med. Hælen treffer først (foten dreies om
  hælen ned til flat), foten står flat, og hælen løftes (foten dreies om
  tåspissen) til den slipper. I svevet går ankelen i en minimal-rykk-bane
  med løft, og tåa holdes klar av bakken.
* **Bekkenet** står så høyt beina tillater (omvendt pendel): for hvert
  bilde regnes den høyeste roten der hvert bein når foten med kneet bøyd
  som i ekte gange (rundt 15° etter hælnedslaget, 5° midt i støtten).
  Dermed får gangen kontakt, nedtrykk, passering og løft av seg selv.
  Hoftene vipper (frontalplanet) og dreier (horisontalplanet), brystet
  motroterer, hodet holdes stødig i verden, og armene svinger i motfase
  med beina, med albuen og hånda forsinket.
* **IK** med to ledd legger beina (knærne fram, etter fotens retning) og
  armene når hånda har et mål (albuen ned, bak og ut).
* **Redskapene** holdes klar av kroppen: punktene på redskapet måles mot
  kapsler rundt lårene, leggene og hoftene i hvert bilde, og armen føres
  ut fra kroppen så mye som trengs (glattet over tid).

Spillet (crates/fristad-domain/src/domain/npcs.rs) leser farten i gangen fra klippet selv (den
foten som står flat glir med farten), og stopper med `stop`/`stop_r`
som begynner ved hælnedslaget på venstre/høyre fot og bremser jevnt over
`DUTY` av en gangsyklus, og starter med `start`, som akselererer jevnt
over `START` av en gangsyklus og ender i gangens fase 0. Snuingen (`turn_l`/`turn_r`) dreier `TURN_DEG`
grader per klipp. Kjør som skript for å legge klippene på en ferdig
figur (til å prøve uten å bygge den på nytt):

    python3.13 tools/models/folk_motion.py inn.glb ut.glb stil [R:hang] [L:hang]
"""

import math
import os
import sys

import bpy  # noqa: F401 (mathutils kommer med bpy)
from mathutils import Matrix, Quaternion, Vector

FPS = 24
# Andelen av gangsyklusen en fot står. Spillet bremser stoppet over
# DUTY · gangsyklusen, så tallet er det samme i npcs.rs.
DUTY = 0.6
# Så mange grader dreier figuren per klipp i turn_l og turn_r (npcs.rs).
TURN_DEG = 60.0
TURN_FRAMES = 20
# De sittende klippene lages bare for figurene som sitter (`only`).
SEATED_CLIPS = ("sit", "play")


# --- Små hjelpere ----------------------------------------------------------------


def eul(x=0.0, y=0.0, z=0.0):
    """Dreiningen `Clip.rot` bruker: Z · Y · X, i grader, i armaturrommet."""
    r = Matrix.Rotation(math.radians(z), 3, "Z") @ Matrix.Rotation(math.radians(y), 3, "Y") @ Matrix.Rotation(math.radians(x), 3, "X")
    return r.to_quaternion()


def clamp(t, a=0.0, b=1.0):
    return min(b, max(a, t))


def smooth(t):
    t = clamp(t)
    return t * t * (3 - 2 * t)


def smoother(t):
    t = clamp(t)
    return t * t * t * (t * (6 * t - 15) + 10)


def lerp(a, b, t):
    return a + (b - a) * t


def curve(points, ease=smooth, period=None):
    """En kurve gjennom (tid, verdi) med myk overgang mellom punktene. Med
    `period` går den i ring (siste punkt mot det første)."""
    pts = sorted(points, key=lambda p: p[0])
    if period:
        pts = [(pts[-1][0] - period, pts[-1][1])] + pts + [(pts[0][0] + period, pts[0][1])]

    def at(t):
        if period:
            t = pts[1][0] + (t - pts[1][0]) % period
        if t <= pts[0][0]:
            return pts[0][1]
        for (t0, v0), (t1, v1) in zip(pts, pts[1:]):
            if t <= t1:
                return lerp(v0, v1, ease((t - t0) / max(1e-6, t1 - t0)))
        return pts[-1][1]

    return at


def pulse(t, at, rise, hold, fall):
    """0 → 1 → 0: opp over `rise`, står i `hold`, ned over `fall`."""
    if t < at or t > at + rise + hold + fall:
        return 0.0
    if t < at + rise:
        return smooth((t - at) / rise)
    if t < at + rise + hold:
        return 1.0
    return 1.0 - smooth((t - at - rise - hold) / fall)


def noise(t, seed, period):
    """Rolig, løkkende støy i [-1, 1]: tre sinuser med perioder som går opp i `period`."""
    w = 2 * math.pi / period
    return (0.55 * math.sin(w * 2 * t + seed) + 0.3 * math.sin(w * 3 * t + seed * 2.3) + 0.15 * math.sin(w * 5 * t + seed * 4.1))


def gauss_smooth(vals, sigma, loop):
    if sigma <= 0:
        return list(vals)
    n = len(vals)
    r = int(math.ceil(sigma * 3))
    ws = [math.exp(-0.5 * (k / sigma) ** 2) for k in range(-r, r + 1)]
    out = []
    for i in range(n):
        acc = wsum = 0.0
        for k, w in zip(range(-r, r + 1), ws):
            j = i + k
            if loop:
                j %= n
            elif j < 0 or j >= n:
                continue
            acc += vals[j] * w
            wsum += w
        out.append(acc / wsum)
    return out


def side_sign(s):
    return 1 if s == "L" else -1


# --- Stilene ---------------------------------------------------------------------
#
# Lengder i meter for en figur på 1,8 m (skaleres med høyden).
# speed: farten (m/s); cycle: sekunder per gangsyklus (to skritt); width:
# halv sporvidde (ankelen); toe_out: tærne ut (grader); lift: ankelløft i
# svevet; hs/to: fotens vinkel mot bakken ved hælnedslag og ved tåavsparket;
# lr_flex/ms_flex: kneet etter hælnedslaget og midt i støtten; sway:
# bekkenet sidelengs over foten som står; list/yaw: bekkenet vipper og
# dreier; thorax: brystet motroterer; lean: lener seg fram; hunch: krum
# rygg (brystet og nakken, hodet ser likevel fram); head: hodet ned
# (grader); jut: haka fram; arm/elbow/arm_out: armsving, albuebøy og hvor
# langt ut armene henger; bounce: sprett i fraskyvet; shift: vektskift i
# hvile; breath: pust (1 = vanlig); tempo: hvor fort hvilen og gestene
# går; gestures: hvilke småbevegelser figuren gjør i hvile.
BASE = dict(speed=1.0, cycle=1.12, width=0.11, toe_out=7.0, lift=0.055, hs=15.0, to=26.0, lr_flex=15.0, ms_flex=5.0,
            sway=0.024, list=3.5, yaw=5.0, thorax=4.0, lean=2.0, hunch=0.0, head=0.0, jut=0.0, arm=17.0, elbow=16.0,
            arm_out=5.0, bounce=0.0, back=0.055, rise=0.26, shift=0.03, breath=1.0, tempo=1.0, gestures=("look", "belt"), talk=1.0)

STYLES = {
    "default": dict(),
    # Smeden: tung og bredbeint, armene ute fra den brede kroppen, rolige
    # og lange skritt med et tydelig tråkk.
    "smith": dict(speed=0.95, cycle=1.22, width=0.15, toe_out=13.0, lift=0.048, hs=12.0, to=22.0, lr_flex=19.0, sway=0.042,
                  list=2.5, yaw=4.0, thorax=6.0, lean=3.0, hunch=4.0, arm=14.0, elbow=20.0, arm_out=10.0, shift=0.04,
                  breath=1.2, tempo=0.85, gestures=("look", "heft"), talk=0.8),
    # Vertinnen: rolig og rett i ryggen, korte skritt, hoftene med.
    "innkeeper": dict(speed=0.85, cycle=1.12, width=0.085, toe_out=6.0, lift=0.04, hs=12.0, to=22.0, lr_flex=12.0,
                      sway=0.026, list=5.0, yaw=6.5, thorax=3.0, lean=-1.0, head=-2.0, arm=11.0, elbow=18.0, arm_out=7.0,
                      shift=0.03, back=0.03, gestures=("look", "apron"), talk=1.1),
    # Gorvek: tung og truende, vekten fram, haka fram og ned, skuldrene
    # ruller, armene bredt ut fra kroppen.
    "gorvek": dict(speed=1.0, cycle=1.26, width=0.14, toe_out=10.0, lift=0.045, hs=13.0, to=24.0, lr_flex=20.0, sway=0.04,
                   list=2.5, yaw=5.0, thorax=8.0, lean=8.0, hunch=6.0, head=6.0, jut=6.0, arm=15.0, elbow=22.0,
                   arm_out=12.0, shift=0.035, breath=1.4, tempo=0.8, gestures=("look", "neck"), talk=0.7),
    # Gammel og krokete: krum rygg, bøyde knær, korte og forsiktige skritt.
    "old": dict(speed=0.62, cycle=1.3, width=0.1, toe_out=12.0, lift=0.028, hs=6.0, to=14.0, lr_flex=24.0, ms_flex=15.0,
                sway=0.03, list=2.0, yaw=3.0, thorax=2.0, lean=6.0, hunch=22.0, jut=10.0, arm=6.0, elbow=28.0,
                arm_out=8.0, shift=0.02, breath=0.9, tempo=0.7, gestures=("look", "back"), talk=0.8),
    # Gutten: lett og sprett, lange armsving, høye løft og sprett i fraskyvet.
    "boy": dict(speed=1.15, cycle=0.98, width=0.085, toe_out=4.0, lift=0.065, hs=20.0, to=34.0, lr_flex=13.0, sway=0.02,
                list=4.0, yaw=7.0, thorax=6.0, lean=3.0, arm=24.0, elbow=22.0, arm_out=5.0, bounce=0.022, shift=0.04,
                tempo=1.3, gestures=("look", "bounce"), talk=1.4),
    # Vakta: rett og stødig, marsjerer litt.
    "guard": dict(speed=1.0, cycle=1.1, width=0.11, toe_out=8.0, hs=17.0, lean=0.0, head=-1.0, arm=12.0, arm_out=7.0,
                  gestures=("look", "belt"), talk=0.8),
}

# Figurene: stilen og hva hendene holder (de nye figurene fra folk.py).
FIGURES = {
    "smith": ("smith", {"R": "hang"}),
    "innkeeper": ("innkeeper", {"L": "hang"}),
    "gorvek": ("gorvek", {"R": "hang"}),
    "woman": ("innkeeper", {"R": "heavy"}),
    "apothecary": ("old", {"R": "front"}),
    "clerk": ("default", {"R": "front"}),
    "farmer": ("default", {"R": "long"}),
    "boy": ("boy", {}),
    "guard": ("guard", {"R": "long"}),
    "merchant": ("old", {"R": "hang"}),
    "pilgrim": ("old", {"R": "long"}),
    "troubadour": ("default", {}),
}


def style_of(style):
    st = dict(BASE)
    st.update(STYLES.get(style, {}) if isinstance(style, str) else style)
    return st


# --- Kroppen ---------------------------------------------------------------------


class Body:
    """Skjelettet i hvileposen (armaturrommet), med lengdene og målene
    klippene trenger."""

    def __init__(self, arm):
        bones = list(arm.data.bones)
        depth = {}
        for b in bones:
            d, p = 0, b.parent
            while p:
                d, p = d + 1, p.parent
            depth[b.name] = d
        self.order = [b.name for b in sorted(bones, key=lambda b: depth[b.name])]
        self.head = {b.name: Vector(b.head_local) for b in bones}
        self.tail = {b.name: Vector(b.tail_local) for b in bones}
        self.parent = {b.name: (b.parent.name if b.parent else None) for b in bones}
        for s in ("L", "R"):
            for a, c in (("thigh", "shin"), ("shin", "foot"), ("upper_arm", "forearm"), ("forearm", "hand")):
                self.tail[f"{a}.{s}"] = self.head[f"{c}.{s}"].copy()
        top = self.head["head"].z + 0.6 * (self.head["head"].z - self.head["neck"].z) + 0.1
        self.height = max(top, 1.2)
        self.k = self.height / 1.8
        k = self.k
        # Foten: ankelens høyde, hvor langt hælen er bak og tåspissen foran.
        self.ankle_h = self.head["foot.L"].z
        self.heel = 0.075 * k
        self.toe = 0.2 * k
        self.sole = 0.0
        # Kapslene rundt beina, hoftene og magen (redskapene holdes unna).
        self.radius = {"thigh": 0.085 * k, "shin": 0.062 * k, "hips": 0.17 * k}
        self.belly = 0.15 * k
        # Punktene på det hendene holder, i hvileposen.
        self.tools = {"L": [], "R": []}

    def vec(self, b):
        return self.tail[b] - self.head[b]

    def length(self, b):
        return self.vec(b).length

    def leg(self, s="L"):
        return self.length(f"thigh.{s}") + self.length(f"shin.{s}")

    def measure(self, body=None, tools=None):
        """Målene fra meshen: fotens lengde og høyde, tykkelsen på lårene og
        leggene, magen, og punktene på redskapene. `body` er kroppen med
        vektgrupper; `tools` er {side: [objekter]} (eller {side: [punkter]})."""
        if body is not None:
            dom = _dominant(body)
            mw = body.matrix_world
            feet, legs = [], {"thigh": [], "shin": []}
            belly = []
            hz = self.head["hips"].z
            for v, b in zip(body.data.vertices, dom):
                p = mw @ v.co
                if b == "foot.L":
                    feet.append(p)
                elif b in ("thigh.L", "shin.L"):
                    kind = b.split(".")[0]
                    legs[kind].append(_seg_dist(p, self.head[b], self.tail[b]))
                if b in ("hips", "spine") and abs(p.z - (hz + 0.12 * self.k)) < 0.05 and abs(p.y) < 0.08:
                    belly.append(p.x)
            if len(feet) > 20:
                a = self.head["foot.L"]
                self.heel = max(0.03, a.x - min(p.x for p in feet))
                self.toe = max(0.08, max(p.x for p in feet) - a.x)
                self.sole = min(p.z for p in feet)
                self.ankle_h = a.z - self.sole
            for kind, ds in legs.items():
                if len(ds) > 20:
                    ds.sort()
                    self.radius[kind] = ds[int(len(ds) * 0.85)]
            if belly:
                self.belly = max(belly)
        for s, objs in (tools or {}).items():
            pts = []
            for o in objs:
                if isinstance(o, Vector):
                    pts.append(o)
                elif hasattr(o, "data") and hasattr(o.data, "vertices"):
                    pts += [o.matrix_world @ v.co for v in o.data.vertices]
            self.tools[s] = _thin(pts, 160)

    def fk(self, R, root, loc=None):
        loc = loc or {}
        G, H = {}, {}
        for b in self.order:
            p = self.parent[b]
            off = loc.get(b)
            if p is None:
                G[b] = R[b]
                H[b] = self.head[b] + root
            else:
                d = self.head[b] - self.head[p]
                if off is not None:
                    d = d + Vector(off)
                G[b] = G[p] @ R[b]
                H[b] = H[p] + G[p] @ d
        return G, H

    def two_bone(self, R, G, H, upper, lower, target, pole):
        Pg = G[self.parent[upper]]
        A = H[upper]
        L1, L2 = self.length(upper), self.length(lower)
        v = target - A
        d = min(max(v.length, abs(L1 - L2) + 0.02), (L1 + L2) * 0.9995)
        dirv = v.normalized()
        cos_a = clamp((L1 * L1 + d * d - L2 * L2) / (2 * L1 * d), -1.0, 1.0)
        a = math.acos(cos_a)
        pp = pole - dirv * pole.dot(dirv)
        if pp.length < 1e-5:
            pp = Vector((1, 0, 0)) - dirv * dirv.x
        pp.normalize()
        knee = A + (dirv * math.cos(a) + pp * math.sin(a)) * L1
        tip = A + dirv * d
        Gu = (Pg @ self.vec(upper)).rotation_difference(knee - A) @ Pg
        R[upper] = Pg.inverted() @ Gu
        Gl = (Gu @ self.vec(lower)).rotation_difference(tip - knee) @ Gu
        R[lower] = Gu.inverted() @ Gl
        return Gl

    def wrist_fk(self, pose, s):
        """Håndleddet og albuen slik dreiningene alene legger armen."""
        R = {b: eul(*pose.rot.get(b, (0, 0, 0))) for b in self.order}
        G, H = self.fk(R, pose.root, pose.loc)
        return H[f"hand.{s}"], H[f"forearm.{s}"]

    def solve(self, pose):
        """Lokale dreininger (armaturrommet) for alle bein."""
        R = {b: eul(*pose.rot.get(b, (0, 0, 0))) for b in self.order}
        root = Vector(pose.root)
        G, H = self.fk(R, root, pose.loc)
        # Beina: ankelen i målet, kneet fram etter fotens retning.
        for s, (ankle, pitch, yaw) in pose.feet.items():
            q_yaw = eul(0, 0, yaw)
            pole = q_yaw @ Vector((1.0, 0.0, 0.05))
            Gs = self.two_bone(R, G, H, f"thigh.{s}", f"shin.{s}", Vector(ankle), pole)
            R[f"foot.{s}"] = Gs.inverted() @ (q_yaw @ eul(0, -pitch, 0))
        G, H = self.fk(R, root, pose.loc)
        # Hodet og nakken: hodet holdes i retningen `head_world` i verden,
        # nakken tar sin del av veien fra brystet.
        if pose.head_world is not None:
            Gc = G["chest"]
            share = pose.neck_share
            Gn = Gc.slerp(pose.head_world, share) if share > 0 else Gc
            R["neck"] = Gc.inverted() @ Gn @ eul(*pose.rot.get("neck", (0, 0, 0)))
            G, H = self.fk(R, root, pose.loc)
            R["head"] = G["neck"].inverted() @ pose.head_world
            G, H = self.fk(R, root, pose.loc)
        # Armene med mål: IK, blandet fra der dreiningene legger armen.
        for s, hand in pose.hands.items():
            if hand is None:
                continue
            target, w, pole = hand[:3]
            if w <= 1e-4:
                continue
            # Målet (og polen) kan gis i rommet til et bein i hvileposen
            # (brystet for gester, hoftene for beltet), så det følger med
            # når figuren lener seg eller er krokrygget.
            frame = hand[3] if len(hand) > 3 else None
            if frame:
                target = H[frame] + G[frame] @ (Vector(target) - self.head[frame])
                if pole is not None:
                    pole = G[frame] @ pole
            sy = side_sign(s)
            wfk, efk = H[f"hand.{s}"].copy(), H[f"forearm.{s}"].copy()
            A = H[f"upper_arm.{s}"]
            mid = (A + wfk) * 0.5
            pole_fk = (efk - mid)
            if pole_fk.length < 1e-4:
                pole_fk = G["chest"] @ Vector((-0.5, 0.6 * sy, -0.3))
            pole_t = pole if pole is not None else G["chest"] @ Vector((-0.45, 0.75 * sy, -0.5))
            tgt = wfk.lerp(Vector(target), w)
            # Polen dreies rundt aksen skulder–håndledd fra der albuen er
            # (dreiningene) til målet, så albuen aldri slår over.
            ax = (tgt - A).normalized()
            pa = pole_fk - ax * pole_fk.dot(ax)
            pb = pole_t - ax * pole_t.dot(ax)
            if pa.length > 1e-5 and pb.length > 1e-5:
                pole_v = Quaternion().slerp(pa.normalized().rotation_difference(pb.normalized()), w) @ pa.normalized()
            else:
                pole_v = pole_t
            hand_local = R[f"hand.{s}"]
            self.two_bone(R, G, H, f"upper_arm.{s}", f"forearm.{s}", tgt, pole_v)
            R[f"hand.{s}"] = hand_local
            G, H = self.fk(R, root, pose.loc)
        # Hendene som holder noe, holdes i en retning i verden.
        for s, q in pose.hand_world.items():
            if q is None:
                continue
            G, H = self.fk(R, root, pose.loc)
            R[f"hand.{s}"] = G[f"forearm.{s}"].inverted() @ q
        return R, root

    def tool_points(self, R, root, loc, s):
        G, H = self.fk(R, root, loc)
        h = f"hand.{s}"
        return [H[h] + G[h] @ (p - self.head[h]) for p in self.tools[s]], G, H

    def clearance(self, R, root, loc, s):
        """Hvor langt redskapet i hånda `s` er fra lårene, leggene og
        hoftene (negativt: inni). Bare punktene under hofta teller."""
        if not self.tools.get(s):
            return 1.0
        pts, G, H = self.tool_points(R, root, loc, s)
        caps = []
        for side in ("L", "R"):
            for kind in ("thigh", "shin"):
                b = f"{kind}.{side}"
                caps.append((H[b], H[b] + G[b] @ self.vec(b), self.radius[kind]))
        hl, hr = H["thigh.L"], H["thigh.R"]
        up = G["hips"] @ Vector((0, 0, 0.12 * self.k))
        caps.append((hl + up, hr + up, self.radius["hips"] * 0.75))
        best = 1.0
        for p in pts:
            for a, b, r in caps:
                best = min(best, _seg_dist(p, a, b) - r)
        return best


def _seg_dist(p, a, b):
    ab = b - a
    l2 = ab.length_squared
    u = 0.0 if l2 < 1e-12 else clamp((p - a).dot(ab) / l2)
    return (p - (a + ab * u)).length


def _thin(pts, n):
    if len(pts) <= n:
        return list(pts)
    step = len(pts) / n
    return [pts[int(i * step)] for i in range(n)]


def _dominant(ob):
    names = {vg.index: vg.name for vg in ob.vertex_groups}
    out = []
    for v in ob.data.vertices:
        best = max(v.groups, key=lambda g: g.weight, default=None)
        out.append(names.get(best.group, "") if best else "")
    return out


# --- Posen i ett bilde -----------------------------------------------------------


class Pose:
    def __init__(self):
        self.root = Vector((0.0, 0.0, 0.0))
        self.rot = {}
        self.loc = {}
        self.scale = {}
        # side → (ankel, fotens vinkel mot bakken (tå opp +), retning (grader))
        self.feet = {}
        # side → (mål for håndleddet, vekt 0..1, albuens retning eller None[,
        # beinet målet er gitt i, i hvileposen])
        self.hands = {}
        # side → kvaternion: hånda holdes slik i verden (redskapet)
        self.hand_world = {}
        self.head_world = None
        self.neck_share = 0.45

    def add(self, bone, x=0.0, y=0.0, z=0.0):
        ox, oy, oz = self.rot.get(bone, (0.0, 0.0, 0.0))
        self.rot[bone] = (ox + x, oy + y, oz + z)


# --- Føttene: der de står i verden, og kroppen over dem --------------------------


class Plant:
    """En fot som står: lander ved `land` (s) i `at` (verden, x og y) med
    retning `yaw`, og løftes ved `lift`. `hs`: vinkelen ved hælnedslaget,
    `to`: vinkelen når den slipper, `flat`: tida fra nedslag til flat fot,
    `rise`: tida hælen bruker opp før foten slipper."""

    def __init__(self, land, lift, at, yaw=0.0, hs=15.0, to=26.0, flat=0.1, rise=0.2):
        self.land, self.lift, self.at, self.yaw = land, lift, Vector(at), yaw
        self.hs, self.to, self.flat, self.rise = hs, to, flat, rise


class Track:
    """Hvor kroppen er (x, y i verden) og hvor den vender (grader) over tid."""

    def __init__(self, pos, yaw=lambda t: 0.0):
        self.pos, self.yaw = pos, yaw

    def to_model(self, p, t):
        q = eul(0, 0, -self.yaw(t))
        d = Vector((p.x, p.y, p.z)) - Vector((*self.pos(t), 0.0))
        return q @ d


def pivot(body, at, yaw, angle):
    """Ankelen når foten står med vinkelen `angle` (tå opp +) mot bakken i
    `at`: dreid om hælen når tåa er oppe, om tåspissen når hælen er oppe."""
    a = math.radians(angle)
    h = body.ankle_h
    if angle >= 0:
        dx, dz = body.heel, h
        ox = -body.heel
    else:
        dx, dz = -body.toe, h
        ox = body.toe
    x = ox + dx * math.cos(a) - dz * math.sin(a)
    z = dx * math.sin(a) + dz * math.cos(a)
    c, s = math.cos(math.radians(yaw)), math.sin(math.radians(yaw))
    return Vector((at.x + c * x, at.y + s * x, z + body.sole))


def foot_at(body, plants, t, lift, clear=0.012):
    """Ankelen (verden), vinkelen og retningen til en fot ved tida `t`, fra
    fotens nedsettinger (sortert), og om den står."""
    for i, p in enumerate(plants):
        if p.land <= t <= p.lift:
            if t < p.land + p.flat:
                ang = p.hs * (1 - smooth((t - p.land) / p.flat))
            elif t > p.lift - p.rise:
                k = (t - (p.lift - p.rise)) / p.rise
                ang = -p.to * (k ** 1.5)
            else:
                ang = 0.0
            return pivot(body, p.at, p.yaw, ang), ang, p.yaw, True
        if i + 1 < len(plants) and p.lift < t < plants[i + 1].land:
            n = plants[i + 1]
            s = (t - p.lift) / (n.land - p.lift)
            a0 = pivot(body, p.at, p.yaw, -p.to)
            a1 = pivot(body, n.at, n.yaw, n.hs)
            e = smoother(s)
            pos = a0.lerp(a1, e)
            # Løftet: tidlig i svevet (kneet bøyes mest rett etter avsparket).
            pos.z += lift * math.sin(math.pi * clamp(s) ** 0.62) ** 1.2
            ang = curve([(0.0, -p.to), (0.4, -4.0), (0.75, n.hs * 0.6), (1.0, n.hs)])(s)
            yaw = lerp(p.yaw, n.yaw, smooth(s))
            # Tåa og hælen holdes klar av bakken midt i svevet.
            a = math.radians(ang)
            low = min(-body.ankle_h * math.cos(a) + body.toe * math.sin(a), -body.ankle_h * math.cos(a) - body.heel * math.sin(a))
            need = clear * math.sin(math.pi * s) + body.sole
            if pos.z + low < need:
                pos.z = need - low
            return pos, ang, yaw, False
    p = plants[0] if t < plants[0].land else plants[-1]
    return pivot(body, p.at, p.yaw, 0.0), 0.0, p.yaw, True


def place_feet(body, st, pose, plants, track, t):
    """Setter føttene i posen (i modellens rom) og gir hvilke som står."""
    standing = {}
    for s in ("L", "R"):
        pos, ang, yaw, down = foot_at(body, plants[s], t, st["lift"] * body.k)
        m = track.to_model(pos, t)
        m.z = pos.z
        yaw_m = yaw - track.yaw(t) + side_sign(s) * st["toe_out"]
        pose.feet[s] = (m, ang, yaw_m)
        standing[s] = down
    return standing


def max_root(body, pose, s, flex):
    """Den høyeste roten der beinet `s` når ankelen med kneet bøyd `flex` grader."""
    R = {b: eul(*pose.rot.get(b, (0, 0, 0))) for b in body.order}
    root = Vector((pose.root.x, pose.root.y, 0.0))
    G, H = body.fk(R, root, pose.loc)
    hip = H[f"thigh.{s}"]
    L1, L2 = body.length(f"thigh.{s}"), body.length(f"shin.{s}")
    k = math.radians(flex)
    reach = math.sqrt(L1 * L1 + L2 * L2 + 2 * L1 * L2 * math.cos(k))
    a = pose.feet[s][0]
    dxy2 = (a.x - hip.x) ** 2 + (a.y - hip.y) ** 2
    return a.z - hip.z + math.sqrt(max(1e-4, reach * reach - dxy2))


# --- Holdningen og armene --------------------------------------------------------


def posture(body, st, pose):
    """Ryggen, nakken og hodet slik figuren står: lener seg, krum rygg, haka
    fram. Hodet holdes etter `head_world` i verden."""
    lean, hunch = st["lean"], st["hunch"]
    pose.add("hips", y=lean * 0.35)
    pose.add("spine", y=lean * 0.3 + hunch * 0.35)
    pose.add("chest", y=lean * 0.35 + hunch * 0.65)
    pose.add("neck", y=st["jut"])
    pose.head_world = eul(0, st["head"], 0)
    pose.neck_share = 0.35
    for s in ("L", "R"):
        sy = side_sign(s)
        pose.add(f"upper_arm.{s}", x=sy * st["arm_out"], y=-(lean + hunch) * 0.55)
        pose.add(f"forearm.{s}", y=-st["elbow"])


def hold(body, st, pose, grips, swing=None, t=0.0):
    """Hendene som holder noe: «hang» og «heavy» henger i hvileretningen og
    pendler litt etter armen, «long» står loddrett, «front» bæres foran.
    `swing` er armens sving fram (grader) for hver side."""
    swing = swing or {}
    for s, kind in grips.items():
        sy = side_sign(s)
        a = swing.get(s, 0.0)
        if kind == "hang":
            pose.hand_world[s] = eul(0, -0.35 * a, 0)
        elif kind == "heavy":
            pose.add(f"upper_arm.{s}", x=sy * 9.0)
            pose.add(f"forearm.{s}", y=st["elbow"] * 0.7)
            pose.add("spine", x=sy * 2.0)
            pose.add("chest", x=sy * 2.5)
            pose.hand_world[s] = eul(0, -0.15 * a, 0)
        elif kind == "long":
            pose.add(f"forearm.{s}", y=st["elbow"] * 0.3)
            pose.hand_world[s] = eul(0, -0.12 * a, 0)
        elif kind == "front":
            pose.hand_world[s] = eul(0, -0.2 * a, 0)


CARRY = {"hang": 0.4, "heavy": 0.2, "long": 0.2, "front": 0.25}


# --- Klippene --------------------------------------------------------------------


def walk_plants(body, st, cycles=(-2, 3)):
    """Nedsettingene i gange: venstre lander ved t = k·T, høyre ved (k+½)·T,
    og kroppen går med farten `speed`."""
    T = st["Tc"]
    v = st["speed"]
    D = v * DUTY * T
    # Foten treffer litt nærmere under hofta enn den slipper bak (som i ekte
    # gange: hofta strekkes bakover før avsparket).
    xc = body.head["foot.L"].x - st["back"] * body.k
    w = st["width"] * body.k
    plants = {"L": [], "R": []}
    for k in range(*cycles):
        for s, off in (("L", 0.0), ("R", 0.5)):
            land = (k + off) * T
            at = Vector((v * land + xc + D / 2, side_sign(s) * w, 0.0))
            plants[s].append(Plant(land, land + DUTY * T, at, 0.0, st["hs"], st["to"], flat=0.11 * T, rise=st["rise"] * T))
    return plants, D


def arm_swing(st, u, s, amp):
    """Armens sving fram (grader) ved gangfasen `u`: venstre arm fram når
    høyre fot treffer (u = ½), litt forsinket."""
    off = 0.5 if s == "L" else 0.0
    return amp * math.cos(2 * math.pi * (u - off - 0.04))


def walk_pose(body, st, grips, plants, track, t, u, speed_k=1.0, arms_k=1.0):
    """Gangen ved tida `t` (fasen `u` i syklusen): føttene etter
    nedsettingene, bekkenet, ryggen, armene og hodet."""
    pose = Pose()
    posture(body, st, pose)
    standing = place_feet(body, st, pose, plants, track, t)
    w2 = 2 * math.pi * u
    sk = speed_k
    # Bekkenet: sidelengs over foten som står, vipper ned på svevsida,
    # dreier med beinet som går fram.
    sway = st["sway"] * body.k * math.cos(w2 - 2 * math.pi * 0.27) * sk + 0.0
    pose.root = Vector((0.0, sway, 0.0))
    lst = st["list"] * math.cos(w2 - 2 * math.pi * 0.16) * sk
    yaw = -st["yaw"] * math.cos(w2 - 2 * math.pi * 0.02) * sk
    pose.add("hips", x=lst, z=yaw, y=1.2 * math.cos(2 * w2 - 2 * math.pi * 0.35) * sk)
    # Ryggen motroterer: brystet dreier mot bekkenet og vipper motsatt.
    thor = st["thorax"] * math.cos(w2 - 2 * math.pi * 0.04) * sk
    pose.add("spine", x=-lst * 0.6, z=(-yaw + thor) * 0.45)
    pose.add("chest", x=-lst * 0.7, z=(-yaw + thor) * 0.55, y=0.8 * math.cos(2 * w2 - 2 * math.pi * 0.12) * sk)
    # Armene: motfase med beina, albuen og hånda henger etter.
    swing = {}
    for s in ("L", "R"):
        amp = st["arm"] * (CARRY.get(grips.get(s), 1.0)) * sk * arms_k
        a = arm_swing(st, u, s, amp)
        a_lag = arm_swing(st, u - 0.07, s, amp)
        swing[s] = a_lag
        pose.add(f"upper_arm.{s}", y=-a + 2.0 * sk)
        if grips.get(s) not in ("long",):
            pose.add(f"forearm.{s}", y=-0.45 * max(0.0, a_lag) - 0.12 * amp)
            pose.add(f"hand.{s}", y=0.25 * arm_swing(st, u - 0.13, s, amp))
    hold(body, st, pose, grips, swing)
    # Hodet står stødig og ser fram; en anelse med i dreiningen.
    pose.head_world = eul(0, st["head"], 0.15 * thor)
    return pose, standing


FLEX = None


def knee_flex(st, us):
    """Kneet i støtten (grader) ved støttefasen `us` (0 = hælnedslag, DUTY = avspark)."""
    lr, ms = st["lr_flex"], st["ms_flex"]
    return curve([(0.0, ms + 2.0), (0.12, lr), (0.3, ms + 1.5), (0.42, ms), (0.52, ms + 6.0), (DUTY, 30.0)])(us)


def settle_root(body, st, poses, infos, loop, extra=None, base=0.0, sigma=1.2):
    """Roten så høyt beina tillater, med kneet bøyd som i ekte gange (se
    knee_flex), glattet. `infos` per bilde: {side: støttefase eller None}."""
    zs = []
    for pose, info in zip(poses, infos):
        lim = []
        for s in ("L", "R"):
            us = info.get(s)
            flex = knee_flex(st, us) if us is not None else 4.0
            lim.append(max_root(body, pose, s, flex))
        z = min(lim)
        # Myk minimum, så knekken mellom beina blir rund.
        z = -0.012 * math.log(sum(math.exp(-(l - z) / 0.012) for l in lim)) + z
        zs.append(min(z, base))
    zs = gauss_smooth(zs[:-1], sigma, True) + [None] if loop else gauss_smooth(zs, sigma, False)
    if loop:
        zs[-1] = zs[0]
    for i, (pose, z) in enumerate(zip(poses, zs)):
        pose.root.z = z + (extra[i] if extra else 0.0)
    return zs


def walk(body, st, grips):
    T = st["Tc"]
    n = st["frames"]
    plants, D = walk_plants(body, st)
    track = Track(lambda t: (st["speed"] * t, 0.0))
    poses, infos, extra = [], [], []
    for f in range(n + 1):
        t = f / FPS
        u = (t / T) % 1.0
        pose, standing = walk_pose(body, st, grips, plants, track, t, u)
        info = {}
        for s, off in (("L", 0.0), ("R", 0.5)):
            us = (u - off) % 1.0
            info[s] = us if us <= DUTY else None
        poses.append(pose)
        infos.append(info)
        # Sprett: litt ekstra opp i fraskyvet (gutten).
        b = st["bounce"] * body.k
        extra.append(b * max(0.0, math.sin(2 * math.pi * (2 * u - 0.38))) ** 2)
    settle_root(body, st, poses, infos, True, extra)
    return poses


def stop(body, st, grips, side="L"):
    """Fra gange til stillstand: begynner ved hælnedslaget på `side`,
    bremser jevnt over DUTY · T (kroppen kommer over foten), og den bakre
    foten settes ned ved siden av. Så faller kroppen til ro."""
    T = st["Tc"]
    v = st["speed"]
    Td = DUTY * T
    plants, D = walk_plants(body, st, (-2, 1))
    sh = 0.0 if side == "L" else 0.5
    # Speilet: høyre fot først er det samme med en halv syklus forskjøvet.
    lead, back = ("L", "R") if side == "L" else ("R", "L")

    def s_of(t):
        if t <= 0:
            return v * t
        if t < Td:
            return v * (t - t * t / (2 * Td))
        return v * Td / 2

    t0 = sh * T  # klippets t = 0 er gangens t0
    xc = body.head["foot.L"].x - st["back"] * body.k
    w = st["width"] * body.k
    # Fremste fot: lander ved t0 og står. Bakre: står fra t0 − T/2, løftes
    # ved t0 + (DUTY − ½)·T og settes ned ved siden av.
    keep = {s: [p for p in plants[s] if p.land <= t0 + 1e-6] for s in ("L", "R")}
    final = Vector((v * t0 + v * Td / 2 + xc, 0, 0))
    lead_p = keep[lead][-1]
    lead_p.lift = 1e9
    back_p = keep[back][-1]
    lift_t = back_p.lift
    land_t = lift_t + (1 - DUTY) * T * 1.15
    keep[back].append(Plant(land_t, 1e9, Vector((final.x, side_sign(back) * w, 0)), 0.0, st["hs"] * 0.4, 0.0, flat=0.12 * T, rise=0.1))

    def pos(tt):
        return (v * t0 + s_of(tt - t0), 0.0)

    track = Track(pos)
    settle = 0.75
    n = int(round((Td + settle) * FPS))
    poses, infos = [], []
    for f in range(n + 1):
        tc = f / FPS
        t = t0 + tc
        k = max(0.0, 1 - tc / Td)  # hvor mye gange som er igjen
        u = (t / T) % 1.0
        pose, standing = walk_pose(body, st, grips, keep, track, t, u, speed_k=k)
        # Armene faller til ro med en liten etterdønning.
        if tc > Td * 0.6:
            r = (tc - Td * 0.6) / (settle + Td * 0.4)
            for s in ("L", "R"):
                pose.add(f"upper_arm.{s}", y=1.5 * math.sin(r * math.pi * 2) * (1 - r))
        info = {}
        for s in ("L", "R"):
            p = next((p for p in keep[s] if p.land <= t <= p.lift), None)
            if p is None:
                info[s] = None
            else:
                us = (t - p.land) / T
                info[s] = min(us, 0.3) if p.lift > 1e8 else us
        poses.append(pose)
        infos.append(info)
    # Til slutt står roten som i hvilen.
    zs = settle_root(body, st, poses, infos, False)
    rest_z = idle_height(body, st)
    for f, pose in enumerate(poses):
        tc = f / FPS
        e = smooth((tc - Td * 0.7) / (settle + Td * 0.3))
        pose.root.z = lerp(zs[f], rest_z, e)
        pose.root.y *= 1 - e
        pose.root.x = -st["back"] * body.k * smooth((tc - Td * 0.3) / (settle + Td * 0.5))
    return poses


START = 0.4  # andelen av gangsyklusen starten akselererer over (npcs.rs)


def start(body, st, grips):
    """Fra stillstand til gange: vekten over på høyre fot, venstre tar det
    første skrittet, og farten øker jevnt over START · T til full fart.
    Klippet ender nøyaktig der gangen begynner (venstre hæl treffer, fase 0),
    så spillet går rett over i `walk`."""
    T = st["Tc"]
    v = st["speed"]
    Ta = START * T
    D = v * DUTY * T
    xc = body.head["foot.L"].x - st["back"] * body.k
    w = st["width"] * body.k

    def s_of(t):
        return v * t * t / (2 * Ta) if t < Ta else v * Ta / 2 + v * (t - Ta)

    plants = {
        "R": [Plant(-10.0, Ta + (DUTY - 0.5) * T, Vector((xc, -w, 0)), 0.0, 0.0, st["to"], flat=0.01, rise=st["rise"] * T)],
        "L": [Plant(-10.0, 0.06 * T, Vector((xc, w, 0)), 0.0, 0.0, st["to"] * 0.6, flat=0.01, rise=0.06 * T),
              Plant(Ta, Ta + DUTY * T, Vector((xc + D / 2 + s_of(Ta), w, 0)), 0.0, st["hs"], st["to"], flat=0.11 * T, rise=st["rise"] * T)],
    }
    track = Track(lambda t: (s_of(t), 0.0))
    n = int(round(Ta * FPS))
    poses, infos = [], []
    for f in range(n + 1):
        t = f / FPS
        k = smooth(t / Ta) * 0.6 + 0.4 * (t / Ta)
        u = ((t - Ta) / T) % 1.0
        pose, standing = walk_pose(body, st, grips, plants, track, t, u, speed_k=k)
        # Vekten flyttes over høyre fot før venstre løftes.
        pose.root.y += -st["sway"] * body.k * 0.8 * math.sin(math.pi * clamp(t / (0.7 * Ta)))
        pose.root.x = -st["back"] * body.k * (1 - smooth(t / Ta))
        info = {"R": min(0.3, 0.15 + t / T), "L": None if t < Ta - 1e-6 else 0.0}
        if t < 0.06 * T:
            info["L"] = 0.3
        poses.append(pose)
        infos.append(info)
    zs = settle_root(body, st, poses, infos, False)
    rest_z = idle_height(body, st)
    for f, pose in enumerate(poses):
        e = smooth((f / FPS) / (0.6 * Ta))
        pose.root.z = lerp(min(rest_z, zs[f]), zs[f], e)
    return poses


def idle_height(body, st):
    """Roten i hvile: kneet litt bøyd (aldri låst)."""
    return -0.008 * body.k - (0.03 * body.k if st["ms_flex"] > 10 else 0.0)


def stand_feet(body, st, pose, shift=0.0):
    """Føttene i hvile: der gangen slutter, side om side i sporvidden."""
    xc = body.head["foot.L"].x - st["back"] * body.k
    w = st["width"] * body.k
    for s in ("L", "R"):
        pose.feet[s] = (Vector((xc, side_sign(s) * w, body.ankle_h + body.sole)), 0.0, side_sign(s) * st["toe_out"])


def breathe(body, st, pose, t, period):
    """Pusten: brystet løfter seg og vider seg ut, skuldrene stiger litt."""
    b = st["breath"]
    # Innpust raskere enn utpust.
    ph = (t / period) % 1.0
    v = math.sin(math.pi * ph / 0.42) if ph < 0.42 else math.cos(math.pi * (ph - 0.42) / 0.58 / 2) ** 2
    v = clamp(v) if ph < 0.42 else v
    pose.add("spine", y=-0.7 * b * v)
    pose.add("chest", y=-1.1 * b * v)
    s = 1.0 + 0.007 * b * v
    pose.scale["chest"] = (s, s, s)
    for c in ("neck", "upper_arm.L", "upper_arm.R"):
        pose.scale[c] = (1 / s, 1 / s, 1 / s)
    for side in ("L", "R"):
        pose.loc[f"upper_arm.{side}"] = Vector((0.0, 0.0, 0.006 * b * v * body.k))
        pose.add(f"upper_arm.{side}", x=side_sign(side) * 0.6 * b * v)
    return v


def weight(body, st, pose, w):
    """Vekten over den ene foten (w = 1: venstre): bekkenet over foten, hofta
    på den andre siden synker og kneet der bøyes, brystet retter seg opp."""
    pose.root.y += w * st["shift"] * body.k
    pose.add("hips", x=4.5 * w, z=2.0 * w)
    pose.add("spine", x=-2.5 * w)
    pose.add("chest", x=-2.5 * w, z=-1.5 * w)


def idle_pose(body, st, grips, t, period, look=True):
    pose = Pose()
    posture(body, st, pose)
    stand_feet(body, st, pose)
    pose.root = Vector((-st["back"] * body.k, 0.0, idle_height(body, st)))
    br = breathe(body, st, pose, t, period / 3)
    # Vektskiftet: står på den ene foten en stund, så den andre.
    P = period
    w = curve([(0.0, 0.0), (0.12 * P, 0.85), (0.42 * P, 0.9), (0.55 * P, -0.75), (0.86 * P, -0.8)], period=P)(t)
    weight(body, st, pose, w)
    pose.root.z -= 0.004 * body.k * abs(w)
    # Små bevegelser i armene (aldri helt stiv).
    for s in ("L", "R"):
        sy = side_sign(s)
        pose.add(f"upper_arm.{s}", y=1.2 * noise(t, 1.3 + sy, P) + 0.6 * br, x=sy * 0.8 * noise(t, 2.1 + sy, P))
        pose.add(f"forearm.{s}", y=-1.5 * noise(t, 0.4 + sy, P))
    hold(body, st, pose, grips)
    # Hodet ser seg rolig omkring.
    yaw = (6.0 * noise(t, 0.7, P) + 2.0 * w) if look else 0.0
    pitch = st["head"] + 2.0 * noise(t, 3.3, P) - 0.8 * br
    pose.head_world = eul(1.5 * noise(t, 5.1, P), pitch, yaw)
    return pose, w


def idle(body, st, grips):
    P = round(12.0 / st["tempo"] * 4) / 4
    n = int(round(P * FPS))
    return [idle_pose(body, st, grips, f / FPS, P)[0] for f in range(n + 1)]


def free_hands(grips):
    return [s for s in ("R", "L") if s not in grips]


def shoulder(body, s):
    return body.head[f"upper_arm.{s}"]


def gesture(body, st, grips, kind):
    """Småbevegelser i hvile, 4–6 s, som begynner og slutter i hvilen."""
    P = 12.0
    if kind == "look":
        L = 4.6 / st["tempo"] ** 0.5
        side = 1
        yaw = curve([(0, 0.0), (0.22 * L, 42.0 * side), (0.45 * L, 40.0 * side), (0.68 * L, -28.0 * side), (0.82 * L, -26.0 * side), (L, 0.0)])
        n = int(round(L * FPS))
        out = []
        for f in range(n + 1):
            t = f / FPS
            pose, w = idle_pose(body, st, grips, t, P, look=False)
            y = yaw(t)
            pose.add("spine", z=y * 0.08)
            pose.add("chest", z=y * 0.17)
            weight(body, st, pose, 0.25 * y / 42.0)
            pose.head_world = eul(0, st["head"] - 3.0 * abs(y) / 42.0, y * 0.75)
            pose.neck_share = 0.5
            out.append(pose)
        return out
    L = 5.0 / st["tempo"] ** 0.5
    n = int(round(L * FPS))
    out = []
    free = free_hands(grips)
    carry = next(iter(grips), None)
    for f in range(n + 1):
        t = f / FPS
        pose, w = idle_pose(body, st, grips, t, P)
        k = L / 5.0
        if kind == "belt" and free:
            # Retter på beltet: hendene til beltet foran, et lite drag opp.
            e = pulse(t, 0.4 * k, 0.9 * k, 2.4 * k, 0.9 * k)
            tug = 0.018 * body.k * max(0.0, math.sin(2 * math.pi * (t - 1.4 * k) / (1.0 * k))) if 1.4 * k < t < 3.4 * k else 0.0
            for s in free:
                sy = side_sign(s)
                bz = body.head["hips"].z + 0.07 * body.k + tug
                pose.hands[s] = (Vector((body.belly + 0.04 * body.k, sy * 0.1 * body.k, bz)), e, None, "hips")
                pose.add(f"hand.{s}", y=-25 * e, x=-sy * 20 * e)
            pose.head_world = eul(0, st["head"] + 18 * e, 0)
            pose.add("chest", y=4 * e)
        elif kind == "heft" and carry:
            # Løfter redskapet litt, kjenner på tyngden, ser på det.
            e = pulse(t, 0.5 * k, 0.9 * k, 2.2 * k, 1.0 * k)
            bob = 6.0 * math.sin(2 * math.pi * (t - 1.4 * k) / (0.7 * k)) * e if 1.4 * k < t < 2.8 * k else 0.0
            sy = side_sign(carry)
            pose.add(f"upper_arm.{carry}", y=-12 * e, x=sy * 6 * e)
            pose.add(f"forearm.{carry}", y=-62 * e - bob)
            pose.hand_world[carry] = eul(-sy * 10 * e, -30 * e - bob * 0.5, 0) @ pose.hand_world.get(carry, Quaternion())
            pose.head_world = eul(0, st["head"] + 16 * e, -sy * 14 * e)
            pose.neck_share = 0.5
        elif kind == "apron" and free:
            # Glatter ut forkleet: hånda stryker ned foran magen, to ganger.
            s = free[0]
            sy = side_sign(s)
            e = pulse(t, 0.4 * k, 0.8 * k, 2.6 * k, 0.9 * k)
            ph = clamp((t - 1.2 * k) / (2.6 * k))
            sweep = 0.5 - 0.5 * math.cos(2 * math.pi * 2 * ph)
            z = body.head["hips"].z + (0.16 - 0.2 * sweep) * body.k
            pose.hands[s] = (Vector((body.belly + 0.05 * body.k, sy * 0.04 * body.k, z)), e, None, "hips")
            pose.add(f"hand.{s}", y=-35 * e)
            pose.head_world = eul(0, st["head"] + 20 * e, 0)
            pose.add("chest", y=3 * e)
        elif kind == "neck":
            # Knekker nakken til hver side og ruller skuldrene.
            tl = curve([(0, 0.0), (0.7 * k, 0.0), (1.3 * k, 22.0), (1.7 * k, 22.0), (2.4 * k, -20.0), (2.8 * k, -20.0), (3.4 * k, 0.0), (L, 0.0)])(t)
            roll = pulse(t, 3.3 * k, 0.5 * k, 0.1, 0.8 * k)
            pose.head_world = eul(tl, st["head"] - 4.0 * abs(tl) / 22.0, 0)
            pose.neck_share = 0.6
            for s in ("L", "R"):
                pose.loc[f"upper_arm.{s}"] = Vector((-0.02 * roll, 0, 0.035 * roll)) * body.k
            pose.add("chest", y=-5 * roll)
        elif kind == "back":
            # Tar seg til korsryggen og retter seg opp.
            e = pulse(t, 0.4 * k, 1.0 * k, 2.0 * k, 1.1 * k)
            for s in free[:1]:
                sy = side_sign(s)
                pose.hands[s] = (Vector((-0.17 * body.k, sy * 0.1 * body.k, body.head["hips"].z + 0.06 * body.k)), e, Vector((-0.2, sy, 0.2)), "hips")
                pose.add(f"hand.{s}", y=30 * e)
            pose.add("spine", y=-8 * e)
            pose.add("chest", y=-10 * e)
            pose.head_world = eul(0, st["head"] - 10 * e, 0)
        elif kind == "bounce":
            # Vipper på tærne to ganger og ser seg om.
            e = pulse(t, 0.6 * k, 0.3 * k, 1.8 * k, 0.4 * k)
            b = max(0.0, math.sin(2 * math.pi * (t - 0.6 * k) / (1.0 * k))) * e
            pose.root.z += 0.035 * b * body.k
            for s in ("L", "R"):
                a, _, yw = pose.feet[s]
                pose.feet[s] = (pivot(body, Vector((a.x, a.y, 0)), yw, -28 * b), -28 * b, yw)
            pose.head_world = eul(0, st["head"], 25 * pulse(t, 2.6 * k, 0.6 * k, 0.8 * k, 0.8 * k))
        out.append(pose)
    return out


def talk(body, st, grips):
    """Prater: hånda som er fri understreker med åpne gester i takt med
    ordene, hodet nikker, brystet lener seg litt inn."""
    L = 8.0
    n = int(round(L * FPS))
    free = free_hands(grips)
    lead = free[0] if free else None
    other = free[1] if len(free) > 1 else None
    g = st["talk"]
    beats = [0.7, 1.55, 2.35, 3.6, 4.4, 5.2, 6.5, 7.1]
    out = []
    for f in range(n + 1):
        t = f / FPS
        pose, w = idle_pose(body, st, grips, t, L, look=False)
        pose.add("chest", y=2.5 * g)
        # Slagene: rask opp, myk ned.
        beat = sum(math.exp(-((t - b) / 0.13) ** 2) for b in beats)
        lift = curve([(0, 0.0), (0.45, 1.0), (2.8, 0.9), (3.2, 0.55), (3.55, 1.0), (5.5, 0.85), (5.9, 0.35), (6.3, 0.9), (7.5, 0.6), (L, 0.0)])(t)
        if lead:
            sy = side_sign(lead)
            sh = shoulder(body, lead)
            reach = 0.27 * body.k + 0.05 * g * body.k * math.sin(2 * math.pi * 3 * t / L)
            tgt = Vector((sh.x + reach + 0.03 * body.k, sh.y - sy * 0.03 * body.k, sh.z - 0.38 * body.k + 0.05 * beat * g * body.k))
            tgt.y += sy * 0.05 * body.k * math.sin(2 * math.pi * 4 * t / L)
            # Albuen henger ned og litt ut, ikke opp.
            pose.hands[lead] = (tgt, lift * min(1.0, 0.65 + 0.35 * g), Vector((-0.25, 0.35 * sy, -1.0)), "chest")
            pose.add(f"hand.{lead}", y=-10 - 8 * beat * g, x=sy * 8.0)
            pose.add(f"forearm.{lead}", y=0.0)
        if other:
            sy = side_sign(other)
            sh = shoulder(body, other)
            e = (pulse(t, 2.1, 0.4, 1.6, 0.6) + pulse(t, 6.2, 0.4, 0.8, 0.6)) * 0.8
            tgt = Vector((sh.x + 0.22 * body.k, sh.y - sy * 0.02 * body.k, sh.z - 0.46 * body.k + 0.04 * beat * body.k))
            pose.hands[other] = (tgt, e, Vector((-0.25, 0.35 * sy, -1.0)), "chest")
            pose.add(f"hand.{other}", y=-15 * e)
        nod = sum(math.exp(-((t - b - 0.05) / 0.16) ** 2) for b in beats)
        tilt = 4.0 * math.sin(2 * math.pi * t / L) * g
        pose.head_world = eul(tilt, st["head"] + 5.0 * nod * g, 5.0 * math.sin(2 * math.pi * 2 * t / L + 0.6) * g)
        pose.neck_share = 0.5
        out.append(pose)
    return out


def wave(body, st, grips):
    """Vinker: den frie hånda opp over skulderen, tre vink, og ned igjen."""
    free = free_hands(grips)
    s = free[0] if free else "R"
    sy = side_sign(s)
    L = 3.5
    n = int(round(L * FPS))
    out = []
    for f in range(n + 1):
        t = f / FPS
        pose, w = idle_pose(body, st, grips, t, 12.0, look=False)
        e = pulse(t, 0.05, 1.05, 1.35, 1.0)
        sh = shoulder(body, s)
        wag = math.sin(2 * math.pi * (t - 1.1) / 0.45) if 1.1 < t < 2.45 else 0.0
        # Hånda går i en bue ut til siden og opp (ikke tett forbi skulderen).
        p0 = body.head[f"hand.{s}"]
        p1 = Vector((sh.x + 0.34 * body.k, sh.y + sy * 0.34 * body.k, sh.z - 0.24 * body.k))
        p2 = Vector((sh.x + 0.22 * body.k, sh.y + sy * 0.14 * body.k, sh.z + 0.17 * body.k))
        tgt = p0 * (1 - e) ** 2 + p1 * 2 * e * (1 - e) + p2 * e * e
        tgt += Vector((0.0, sy * 0.06 * wag * body.k, 0.0))
        pose.hands[s] = (tgt, smooth(min(1.0, 2.0 * e)), Vector((-0.2, sy * 1.0, -0.8)), "chest")
        pose.add(f"hand.{s}", x=-sy * 18 * wag * e)
        pose.add("chest", x=-sy * 3 * e)
        weight(body, st, pose, -sy * 0.3 * e)
        pose.head_world = eul(-sy * 4 * e, st["head"] - 3 * e, 0)
        out.append(pose)
    return out


# --- Sittende: trubaduren på krakken ved ildstedet i Vertshuset ------------------
#
# Figuren står med føttene i origo; sittende er origo punktet på gulvet rett
# under midten av setet, så spillet setter figuren der krakken står
# (`NpcSpec.seated`). Bekkenet ligger `SEAT` over gulvet (krakken i
# Vertshuset er 0,43 m) og litt over setet, føttene står flatt foran, og
# beina legges med IK, så knærne kommer der de skal av seg selv.

SEAT = 0.43  # setets høyde over gulvet (m), krakken `stool`
SIT_TEMPO = 1.5  # slag per sekund i spilleklippet (sangen i Vertshuset går rundt 90 i minuttet)


def seated_pose(body, st, t, period):
    """Sitter rett på krakken med føttene flatt foran, puster, og ser rolig
    omkring. Gir posen og pusten."""
    k = body.k
    pose = Pose()
    hips = body.head["hips"]
    # Bekkenet over midten av setet, litt over det (setebeina).
    pose.root = Vector((0.01 * k - hips.x, 0.0, SEAT + 0.085 * k - hips.z))
    # Bekkenet vipper litt bakover, ryggen retter seg og brystet kommer fram.
    pose.add("hips", y=-6.0)
    pose.add("spine", y=4.0 + st["hunch"] * 0.3)
    pose.add("chest", y=4.0 + st["hunch"] * 0.5)
    pose.add("neck", y=st["jut"] * 0.5)
    pose.neck_share = 0.4
    for s in ("L", "R"):
        sy = side_sign(s)
        # Føttene flatt på gulvet foran setet, litt fra hverandre og ut.
        pose.feet[s] = (Vector((0.4 * k, sy * 0.15 * k, body.ankle_h + body.sole)), 0.0, sy * 9.0)
        pose.add(f"upper_arm.{s}", x=sy * st["arm_out"] * 0.5)
    br = breathe(body, st, pose, t, period / 3)
    pose.head_world = eul(1.5 * noise(t, 5.1, period), st["head"] * 0.5 + 3.0 + 2.0 * noise(t, 3.3, period) - 0.8 * br, 7.0 * noise(t, 0.7, period))
    return pose, br


def sit(body, st, grips):
    """Sitter på krakken med hendene på lårene (når trubaduren snakker med
    deg i stedet for å spille)."""
    L = 10.0
    n = int(round(L * FPS))
    k = body.k
    hips = body.head["hips"]
    out = []
    for f in range(n + 1):
        t = f / FPS
        pose, br = seated_pose(body, st, t, L)
        for s in ("L", "R"):
            sy = side_sign(s)
            # Hendene hviler midt på lårene (målet i hoftenes rom).
            tgt = hips + Vector((0.24 * k + 0.01 * k * noise(t, 2.0 + sy, L), sy * 0.13 * k, 0.07 * k + 0.004 * k * br))
            pose.hands[s] = (tgt, 1.0, Vector((-0.4, sy * 0.6, -0.7)), "hips")
            pose.add(f"hand.{s}", y=-15.0)
        out.append(pose)
    return out


def play(body, st, grips):
    """Spiller lutt sittende: lutten ligger på høyre lår med halsen opp mot
    venstre, høyre hånd slår over strengene i takt (ned på slaget, opp på
    halvslaget), venstre hånd griper på halsen og flytter seg innimellom, og
    kroppen vugger og hodet nikker med. Hendene står i hoftenes rom, så de
    blir på lutten mens overkroppen vugger. Lutten selv er egen modell
    (kommer fra TRELLIS) og henges på hoftene, der hendene står."""
    L = 8.0
    n = int(round(L * FPS))
    k = body.k
    hips = body.head["hips"]
    beat = SIT_TEMPO
    out = []
    for f in range(n + 1):
        t = f / FPS
        pose, br = seated_pose(body, st, t, L)
        ph = (t * beat) % 1.0
        # Slaget: raskt ned over strengene på slaget, roligere opp igjen, og
        # et mindre slag opp på halvslaget.
        down = curve([(0.0, 1.0), (0.12, -1.0), (0.5, -0.35), (0.6, 0.8), (1.0, 1.0)])(ph)
        accent = 1.0 if int(t * beat) % 4 == 0 else 0.75
        rh = hips + Vector((0.23 * k, -0.07 * k + 0.012 * k * down, 0.17 * k + 0.045 * k * down * accent))
        pose.hands["R"] = (rh, 1.0, Vector((-0.25, -1.0, -0.55)), "hips")
        pose.add("hand.R", y=18.0 - 12.0 * down, x=6.0 * down)
        # Grepet: langs halsen (opp mot venstre og fram), flytter seg hver
        # andre takt og trykker litt på hvert slag.
        slide = curve([(0.0, 0.0), (2.6, 0.0), (2.9, 1.0), (5.3, 1.0), (5.6, 0.4), (7.4, 0.4), (7.7, 0.0)], period=L)(t)
        press = math.exp(-((ph - 0.04) / 0.08) ** 2)
        along = Vector((0.45, 0.85, 0.3)).normalized()
        lh = hips + Vector((0.2 * k, -0.02 * k, 0.16 * k)) + along * (0.40 - 0.07 * slide) * k
        pose.hands["L"] = (lh, 1.0, Vector((-0.3, 0.45, -1.0)), "hips")
        pose.add("hand.L", y=-10.0 - 6.0 * press, x=20.0)
        # Kroppen vugger fra side til side over to takter og lener seg inn
        # mot lutten; hodet nikker på slaget og ser ut i rommet.
        sway = math.sin(2 * math.pi * t * beat / 4.0)
        pose.add("spine", x=2.5 * sway, y=3.0)
        pose.add("chest", x=2.0 * sway, z=-3.0 * sway, y=3.0)
        nod = math.exp(-((ph - 0.08) / 0.12) ** 2)
        pose.head_world = eul(-3.0 * sway, st["head"] * 0.5 - 2.0 + 3.0 * nod - 0.8 * br, 6.0 * noise(t, 0.7, L) + 4.0 * sway)
        out.append(pose)
    return out


def turn(body, st, grips, direction=1):
    """Snur på stedet: små skritt, overkroppen og hodet leder dreiningen.
    Kroppen dreier `TURN_DEG` grader per klipp (spillet dreier figuren like
    fort), så foten som står, dreier motsatt vei i modellen."""
    n = TURN_FRAMES
    Tt = n / FPS
    om = direction * TURN_DEG / Tt
    xc = body.head["foot.L"].x - st["back"] * body.k
    w = st["width"] * body.k
    plants = {"L": [], "R": []}
    swing = 0.36 * Tt
    # Foten som går først, er den på innsida av svingen.
    first = "L" if direction > 0 else "R"
    for k in range(-2, 3):
        for s in ("L", "R"):
            off = 0.0 if s == first else 0.5
            lift = (k + off) * Tt
            land = lift + swing
            nxt = lift + Tt
            yaw = om * (land + nxt) / 2
            home = eul(0, 0, yaw) @ Vector((xc, side_sign(s) * w, 0))
            plants[s].append(Plant(land, nxt, home, yaw, 4.0, 10.0, flat=0.06, rise=0.07))
    track = Track(lambda t: (0.0, 0.0), lambda t: om * t)
    poses, infos = [], []
    for f in range(n + 1):
        t = f / FPS
        pose = Pose()
        posture(body, st, pose)
        standing = {}
        for s in ("L", "R"):
            pos, ang, yaw, down = foot_at(body, plants[s], t, 0.035 * body.k)
            m = track.to_model(pos, t)
            m.z = pos.z
            pose.feet[s] = (m, ang, yaw - track.yaw(t) + side_sign(s) * st["toe_out"])
            standing[s] = down
        u = t / Tt
        # Vekten over foten som står.
        wl = math.cos(2 * math.pi * (u - (0.68 if first == "L" else 0.18)))
        pose.root = Vector((-st["back"] * body.k, 0.6 * st["shift"] * body.k * wl, 0.0))
        pose.add("hips", z=direction * 6.0, x=2.0 * wl)
        pose.add("spine", z=direction * 4.0)
        pose.add("chest", z=direction * 5.0)
        for s in ("L", "R"):
            pose.add(f"upper_arm.{s}", y=-direction * side_sign(s) * 3.0)
        hold(body, st, pose, grips)
        pose.head_world = eul(0, st["head"], direction * 22.0)
        poses.append(pose)
        infos.append({s: (0.3 if standing[s] else None) for s in ("L", "R")})
    settle_root(body, st, poses, infos, True, base=idle_height(body, st))
    return poses


# --- Etterarbeid og nøkler -------------------------------------------------------


def keep_clear(body, poses, grips, loop, margin=0.012):
    """Armen som holder noe, føres ut fra kroppen så mye som trengs for at
    redskapet aldri går gjennom beina eller hoftene."""
    for s in grips:
        if not body.tools.get(s):
            continue
        sy = side_sign(s)
        need = []
        for pose in poses:
            extra = 0.0
            for _ in range(14):
                p2 = _with_out(pose, s, sy, extra)
                R, root = body.solve(p2)
                c = body.clearance(R, root, p2.loc, s)
                if c >= margin:
                    break
                extra += 2.5
            need.append(extra)
        # Redskapet holdes over bakken: albuen bøyes litt mer om det trengs.
        lift = []
        for pose, e in zip(poses, need):
            p2 = _with_out(pose, s, sy, e)
            bend = 0.0
            for _ in range(12):
                R, root = body.solve(p2)
                pts, G, H = body.tool_points(R, root, p2.loc, s)
                if min(p.z for p in pts) >= 0.015:
                    break
                bend += 2.0
                p2 = _with_out(pose, s, sy, e)
                p2.rot[f"forearm.{s}"] = tuple(a + b for a, b in zip(p2.rot.get(f"forearm.{s}", (0, 0, 0)), (0, -bend, 0)))
            lift.append(bend)
        if max(lift) > 0:
            if loop:
                lift = lift[:-1]
            lift = gauss_smooth([max(lift[(i + k) % len(lift)] if loop else lift[min(len(lift) - 1, max(0, i + k))] for k in range(-3, 4)) for i in range(len(lift))], 2.0, loop)
            if loop:
                lift.append(lift[0])
            for pose, e in zip(poses, lift):
                pose.add(f"forearm.{s}", y=-e)
        if max(need) <= 0:
            continue
        # Glattes: holdes ute litt før og etter, og myk inn og ut.
        r = 3
        if loop:
            need = need[:-1]
        n = len(need)
        held = [max(need[(i + k) % n] if loop else need[min(n - 1, max(0, i + k))] for k in range(-r, r + 1)) for i in range(n)]
        held = gauss_smooth(held, 2.0, loop)
        if loop:
            held.append(held[0])
        for pose, e in zip(poses, held):
            pose.add(f"upper_arm.{s}", x=sy * e)


def _with_out(pose, s, sy, extra):
    if extra <= 0:
        return pose
    p = Pose()
    p.root, p.loc, p.scale, p.feet, p.hands = pose.root, pose.loc, pose.scale, pose.feet, pose.hands
    p.hand_world, p.head_world, p.neck_share = pose.hand_world, pose.head_world, pose.neck_share
    p.rot = dict(pose.rot)
    ox, oy, oz = p.rot.get(f"upper_arm.{s}", (0, 0, 0))
    p.rot[f"upper_arm.{s}"] = (ox + sy * extra, oy, oz)
    return p


def bake(body, arm, name, poses, step=1):
    """Nøkkelbildene for et klipp fra posene."""
    from common import Clip

    n = len(poses) - 1
    c = Clip(arm, name, n, FPS)
    frames = list(range(0, n + 1, step))
    if frames[-1] != n:
        frames.append(n)
    for f in frames:
        pose = poses[f]
        R, root = body.solve(pose)
        c.loc("root", f, tuple(root))
        for b in body.order:
            c.rotq(b, f, R[b])
        for b in ("upper_arm.L", "upper_arm.R"):
            c.loc(b, f, tuple(pose.loc.get(b, (0, 0, 0))))
        for b in ("chest", "neck", "upper_arm.L", "upper_arm.R"):
            c.scale(b, f, pose.scale.get(b, (1, 1, 1)))
    c.finish()
    return c


def prepare(arm, style="default", body=None, tools=None):
    B = Body(arm)
    B.measure(body, tools)
    st = style_of(style)
    # Hele bilder per syklus; farten holdes, skrittet følger.
    st["frames"] = max(12, int(round(st["cycle"] * FPS / 2)) * 2)
    st["Tc"] = st["frames"] / FPS
    st["speed"] = st["speed"] * (1.0 if B.k > 0.9 else 0.5 + 0.5 * B.k / 0.9)
    return B, st


def clips(arm, style="default", grips=None, body=None, tools=None, only=None):
    """Alle klippene på armaturen `arm`. Gir stilen (med farten)."""
    import bpy

    grips = dict(grips or {})
    B, st = prepare(arm, style, body, tools)
    made = {
        "walk": (lambda: walk(B, st, grips), True, 1),
        "idle": (lambda: idle(B, st, grips), True, 2),
        "talk": (lambda: talk(B, st, grips), True, 1),
        "wave": (lambda: wave(B, st, grips), False, 1),
        "start": (lambda: start(B, st, grips), False, 1),
        "stop": (lambda: stop(B, st, grips, "L"), False, 1),
        "stop_r": (lambda: stop(B, st, grips, "R"), False, 1),
        "turn_l": (lambda: turn(B, st, grips, 1), True, 1),
        "turn_r": (lambda: turn(B, st, grips, -1), True, 1),
        "look": (lambda: gesture(B, st, grips, "look"), False, 1),
        "fidget": (lambda: gesture(B, st, grips, st["gestures"][1]), False, 1),
        # Sittende (trubaduren): bare når de bes om (`only`).
        "sit": (lambda: sit(B, st, grips), True, 1),
        "play": (lambda: play(B, st, grips), True, 1),
    }
    for name, (fn, loop, step) in made.items():
        if (only and name not in only) or (not only and name in SEATED_CLIPS):
            continue
        poses = fn()
        keep_clear(B, poses, grips, loop)
        bake(B, arm, name, poses, step)
    bpy.context.scene.frame_end = 288
    return st


# --- Prøve på en ferdig figur ----------------------------------------------------


def tool_points_from_mesh(ob, arm, side, far=0.15):
    """Punktene på redskapet i en sammenslått figur: hjørner som henger
    helt på hånda og ligger lenger enn `far` fra håndleddet (fingrene er
    nærmere)."""
    g = ob.vertex_groups.get(f"hand.{side}")
    if g is None:
        return []
    wrist = Vector(arm.data.bones[f"hand.{side}"].head_local)
    mw = ob.matrix_world
    out = []
    for v in ob.data.vertices:
        for vg in v.groups:
            if vg.group == g.index and vg.weight > 0.98:
                p = mw @ v.co
                if (p - wrist).length > far:
                    out.append(p)
    return out


def main(argv):
    import bpy

    src, dst, style = argv[0], argv[1], argv[2]
    grips = dict(a.split(":") for a in argv[3:] if ":" in a)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=os.path.abspath(src))
    arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
    meshes = [o for o in bpy.data.objects if o.type == "MESH" and o.parent == arm]
    body = max(meshes, key=lambda o: len(o.data.vertices))
    ad = arm.animation_data
    if ad:
        for t in list(ad.nla_tracks):
            ad.nla_tracks.remove(t)
        ad.action = None
    for a in list(bpy.data.actions):
        bpy.data.actions.remove(a)
    tools = {s: tool_points_from_mesh(body, arm, s) for s in grips}
    st = clips(arm, style, grips, body=body, tools=tools)
    print(f"stil {style}: {st['speed']:.2f} m/s, syklus {st['Tc']:.3f} s, skritt {st['speed'] * st['Tc'] / 2:.2f} m, grep {grips}, "
          f"redskapspunkter {[len(v) for v in tools.values()]}")
    bpy.ops.export_scene.gltf(filepath=os.path.abspath(dst), export_format="GLB", export_apply=True, export_materials="EXPORT",
                              export_yup=True, export_normals=True, export_texcoords=True, export_image_format="JPEG",
                              export_jpeg_quality=92, export_animations=True, export_skins=True, export_animation_mode="NLA_TRACKS",
                              export_force_sampling=True, export_optimize_animation_size=False, export_cameras=False, export_lights=False)
    import slim

    slim.slim(os.path.abspath(dst))


if __name__ == "__main__":
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    main(sys.argv[1:])
