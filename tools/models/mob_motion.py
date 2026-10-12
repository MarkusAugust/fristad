"""Bevegelsene til fiendene: gange, hvile, angrep, treff, svimmelhet og død,
med egen stil per slag (tooltipen småløper, modalen tramper, tabellen
kryper på alle fire). Brukes av mobs.py.

Hvert klipp regnes ut bilde for bilde som en `Spec`: hvor roten står,
dreininger for ryggen, hodet, armene og halen (i armaturrommet, relativt
til forelderen, som `Clip.rot`), og mål for anklene og håndleddene.
`solve` går gjennom skjelettet (fremoverkinematikk), legger beina og
armene med to-ledds IK (knærne fram, albuene bak og ut), og løfter roten
om noe ellers ville gått under gulvet. Føttene står dermed stille på
bakken mens kroppen beveger seg, og glir ikke.

Gangen lages på stedet med samme fart som i spillet (`v0` er farten i
modellens målestokk, `Kind::stats().speed / scale` i domain/mobs.rs):
foten som står, glir bakover med akkurat den farten, så den står stille
på gulvet når fienden går. Spillet skalerer avspillingen med farten den
faktisk har (`moved / speed`)."""

import math

from mathutils import Matrix, Quaternion, Vector

FPS = 24


def eul(x=0.0, y=0.0, z=0.0):
    """Dreiningen `Clip.rot` bruker: Z · Y · X, i grader."""
    r = Matrix.Rotation(math.radians(z), 3, "Z") @ Matrix.Rotation(math.radians(y), 3, "Y") @ Matrix.Rotation(math.radians(x), 3, "X")
    return r.to_quaternion()


def smooth(t):
    t = min(1.0, max(0.0, t))
    return t * t * (3 - 2 * t)


def curve(points, ease=smooth):
    """En kurve gjennom (bilde, verdi)-punkter med myk overgang mellom
    hvert par. Verdiene kan være tall eller tupler."""
    points = sorted(points, key=lambda p: p[0])

    def at(f):
        if f <= points[0][0]:
            return points[0][1]
        for (f0, v0), (f1, v1) in zip(points, points[1:]):
            if f <= f1:
                t = ease((f - f0) / max(1e-6, f1 - f0))
                if isinstance(v0, tuple):
                    return tuple(a + (b - a) * t for a, b in zip(v0, v1))
                return v0 + (v1 - v0) * t
        return points[-1][1]

    return at


def snap(t):
    """Rask inn, myk ut: til slag og rykk."""
    t = min(1.0, max(0.0, t))
    return 1 - (1 - t) ** 3


class Spec:
    def __init__(self):
        self.root = Vector((0.0, 0.0, 0.0))
        self.rot = {}
        self.scale = {}
        # Ankelmål (x, y, z, tåvinkel); None gir beinet fra `rot`.
        self.feet = {}
        # Håndleddsmål (x, y, z); uten mål styres armen av `rot`.
        self.hands = {}


class Body:
    """Skjelettet slik det står i hvileposen, med lengdene IK trenger."""

    RADIUS = {"hips": 0.17, "chest": 0.2, "head": 0.12, "neck": 0.08, "jaw": 0.06, "tail1": 0.05, "tail2": 0.04, "tail3": 0.03}

    def __init__(self, bones):
        self.order = [b[0] for b in bones]
        self.head = {b[0]: Vector(b[1]) for b in bones}
        self.tail = {b[0]: Vector(b[2]) for b in bones}
        self.parent = {b[0]: b[3] for b in bones}

    def vec(self, b):
        return self.tail[b] - self.head[b]

    def length(self, b):
        return self.vec(b).length

    def leg_reach(self):
        return self.length("thigh.L") + self.length("shin.L")

    def foot_rest(self, side):
        """Ankelen i hvileposen."""
        return self.head[f"foot.{side}"].copy()

    def wrist_rest(self, side):
        return self.head[f"hand.{side}"].copy()

    def fk(self, R, root):
        G, H = {}, {}
        for b in self.order:
            p = self.parent[b]
            if p is None:
                G[b] = R[b]
                H[b] = self.head[b] + root
            else:
                G[b] = G[p] @ R[b]
                H[b] = H[p] + G[p] @ (self.head[b] - self.head[p])
        return G, H

    def two_bone(self, R, G, H, upper, lower, end, target, pole):
        """Legger `upper` og `lower` så enden av `lower` havner i `target`,
        med leddet mellom dem mot `pole`."""
        parent = self.parent[upper]
        Pg = G[parent]
        A = H[upper]
        L1, L2 = self.length(upper), self.length(lower)
        v = target - A
        d = min(max(v.length, 0.05), (L1 + L2) * 0.999)
        dirv = v.normalized()
        cos_a = max(-1.0, min(1.0, (L1 * L1 + d * d - L2 * L2) / (2 * L1 * d)))
        a = math.acos(cos_a)
        pp = pole - dirv * pole.dot(dirv)
        if pp.length < 1e-5:
            pp = Vector((0, 0, 1)) - dirv * dirv.z
        pp.normalize()
        knee = A + (dirv * math.cos(a) + pp * math.sin(a)) * L1
        tip = A + dirv * d
        q1 = (Pg @ self.vec(upper)).rotation_difference(knee - A)
        Gu = q1 @ Pg
        R[upper] = Pg.inverted() @ Gu
        q2 = (Gu @ self.vec(lower)).rotation_difference(tip - knee)
        Gl = q2 @ Gu
        R[lower] = Gu.inverted() @ Gl
        return Gl

    def solve(self, spec, ik_legs=True, slack=0.0):
        """Lokale dreininger (armaturrommet) for alle bein, og roten."""
        R = {b: eul(*spec.rot.get(b, (0, 0, 0))) for b in self.order}
        root = Vector(spec.root)
        legs = [s for s in ("L", "R") if spec.feet.get(s) is not None and ik_legs]
        arms = [s for s in ("L", "R") if spec.hands.get(s) is not None]
        for _ in range(3):
            G, H = self.fk(R, root)
            for s in legs:
                x, y, z, pitch = spec.feet[s]
                pole = G["hips"] @ Vector((1.0, 0.0, -0.1))
                Gs = self.two_bone(R, G, H, f"thigh.{s}", f"shin.{s}", f"foot.{s}", Vector((x, y, z)), pole)
                R[f"foot.{s}"] = Gs.inverted() @ eul(0, pitch, 0)
            for s in arms:
                sy = 1 if s == "L" else -1
                pole = G["chest"] @ Vector((-1.0, 0.8 * sy, 0.0))
                G, H = self.fk(R, root)
                Gf = self.two_bone(R, G, H, f"upper_arm.{s}", f"forearm.{s}", f"hand.{s}", Vector(spec.hands[s]), pole)
                R[f"hand.{s}"] = Gf.inverted() @ eul(*spec.rot.get(f"hand_world.{s}", (0, 40, 0)))
            # Ingenting under gulvet: det laveste punktet (med tykkelsen)
            # løfter roten. Beina med mål regnes ikke med.
            G, H = self.fk(R, root)
            low = 1e9
            for b in self.order:
                if b == "root" or (b.split(".")[0] in ("thigh", "shin", "foot") and b.split(".")[-1] in legs):
                    continue
                r = self.RADIUS.get(b, 0.04)
                # Armer og bein i et lik får gå litt i gulvet (`slack`), så
                # kroppen ikke blir liggende oppå dem.
                if slack and b not in self.RADIUS:
                    r -= slack
                tip = H[b] + G[b] @ self.vec(b)
                low = min(low, H[b].z - r, tip.z - r)
            if low < 0.0:
                root.z -= low
            else:
                break
        return R, root

    def key(self, clip, frame, spec, ik_legs=True, slack=0.0):
        R, root = self.solve(spec, ik_legs, slack)
        clip.loc("root", frame, tuple(root))
        for b in self.order:
            clip.rotq(b, frame, R[b])
        for b, s in spec.scale.items():
            clip.scale(b, frame, s)


# --- Stilen per slag ---------------------------------------------------------

# frames: bilder i en hel gangsyklus (to skritt); v0: farten (m/s i modellens
# målestokk); duty: andelen av syklusen en fot står (under 0,5 er løp med
# svev); lift: hvor høyt foten løftes; width: halv sporvidde; hunch: hvor
# krokrygget (grader framover i brystet); arm: armsving; sway: vugging
# sidelengs; bob: dupping; tail: halesving; heavy: tramper (lenger
# nedslag, rister ved hvert skritt); quad: går på alle fire; twitch:
# rykker digitalt i hvile; stride_x: hvor foten står i forhold til hofta.
STYLES = {
    # Button Wight (monsters.py): en tung steinplate på korte beinbein som
    # tasser fort og vagger, rett i ryggen (ingen hale å svinge).
    "wight": dict(frames=8, v0=1.4, duty=0.6, lift=0.07, width=0.24, hunch=3, arm=14, sway=0.05, bob=0.025, tail=0, twitch=True, hips_yaw=5, idle=48),
    "button": dict(frames=18, v0=2.1, duty=0.56, lift=0.13, width=0.19, hunch=20, arm=34, sway=0.035, bob=0.03, tail=18, twitch=True, hips_yaw=9, idle=48),
    "tooltip": dict(frames=10, v0=6.5, duty=0.38, lift=0.2, width=0.15, hunch=30, arm=22, sway=0.02, bob=0.07, tail=26, twitch=True, hips_yaw=6, idle=36, hop=True),
    "input": dict(frames=20, v0=1.65, duty=0.62, lift=0.15, width=0.14, hunch=6, arm=10, sway=0.03, bob=0.025, tail=10, twitch=False, hips_yaw=12, idle=56, spears=True),
    "modal": dict(frames=28, v0=0.87, duty=0.7, lift=0.11, width=0.27, hunch=12, arm=20, sway=0.07, bob=0.035, tail=8, twitch=False, hips_yaw=6, idle=60, heavy=True),
    "table": dict(frames=16, v0=1.71, duty=0.52, lift=0.12, width=0.23, hunch=50, arm=0, sway=0.03, bob=0.03, tail=20, twitch=True, hips_yaw=7, idle=40, quad=True),
    "framework": dict(frames=32, v0=0.7, duty=0.7, lift=0.1, width=0.26, hunch=14, arm=16, sway=0.06, bob=0.04, tail=10, twitch=True, hips_yaw=5, idle=64, heavy=True),
}


def crouch_for(body, st, D):
    """Hvor mye roten må senkes så beinet når fram og tilbake i skrittet."""
    reach = body.leg_reach() * 0.95
    hip = body.head["thigh.L"].z
    ankle = body.foot_rest("L").z
    need = math.sqrt(max(0.01, reach * reach - (D / 2 + 0.03) ** 2)) + ankle
    return min(0.0, need - hip) - (0.06 if st.get("heavy") else 0.0) - (0.08 if st.get("quad") else 0.0)


def base_pose(body, st, spec, crouch=0.0):
    """Stillingen alle klippene står i: krokryggen, hodet som ser fram, armene
    litt ute, føttene i sporvidden."""
    h = st["hunch"]
    spec.root = Vector((0.0, 0.0, crouch))
    hips = h * 0.35 if not st.get("quad") else 22
    chest = h * 0.65 if not st.get("quad") else h - 10
    spec.rot.update({
        "hips": (0, hips, 0),
        "chest": (0, chest, 0),
        "neck": (0, -h * 0.35, 0),
        "head": (0, -h * 0.55 - (18 if st.get("quad") else 0), 0),
        "jaw": (0, 12, 0),
        "upper_arm.L": (14, -8, 0),
        "upper_arm.R": (-14, -8, 0),
        "forearm.L": (0, -18, 0),
        "forearm.R": (0, -18, 0),
        "tail1": (0, 8, 0),
        "tail2": (0, 6, 0),
        "tail3": (0, 4, 0),
    })
    if st.get("spears"):
        # Markørspydene holdes fram, klare.
        spec.rot.update({"upper_arm.L": (10, -35, 0), "upper_arm.R": (-10, -35, 0), "forearm.L": (0, -30, 0), "forearm.R": (0, -30, 0)})
    for s, sy in (("L", 1), ("R", -1)):
        a = body.foot_rest(s)
        spec.feet[s] = (a.x, sy * st["width"], a.z, 0.0)
    if st.get("quad"):
        for s, sy in (("L", 1), ("R", -1)):
            spec.hands[s] = (0.62, sy * (st["width"] + 0.08), 0.12)
    return spec


def add(spec, bone, x=0.0, y=0.0, z=0.0):
    ox, oy, oz = spec.rot.get(bone, (0, 0, 0))
    spec.rot[bone] = (ox + x, oy + y, oz + z)


# --- Klippene ----------------------------------------------------------------


def walk(c, body, st):
    T = st["frames"]
    secs = T / FPS
    duty = st["duty"]
    D = st["v0"] * duty * secs
    crouch = crouch_for(body, st, D)
    xc = body.foot_rest("L").x + 0.04
    lowest = 0.0 if duty >= 0.5 else duty / 2
    for f in range(T + 1):
        ph = f / T
        spec = base_pose(body, st, Spec(), crouch)
        # Føttene: står og glir bakover med farten, svinger fram i en bue.
        for s, sy, off in (("L", 1, 0.0), ("R", -1, 0.5)):
            u = (ph + off) % 1.0
            if u < duty:
                k = u / duty
                x = xc + D / 2 - D * k
                z = body.foot_rest(s).z
                pitch = -12 * max(0.0, 1 - k / 0.15) + 22 * max(0.0, (k - 0.75) / 0.25)
            else:
                k = (u - duty) / (1 - duty)
                e = 0.5 - 0.5 * math.cos(math.pi * k)
                x = xc - D / 2 + D * e
                z = body.foot_rest(s).z + st["lift"] * math.sin(math.pi * k) ** 0.8
                pitch = 22 * (1 - k) - 12 * k
            spec.feet[s] = (x, sy * st["width"], z, pitch)
        # Kroppen: dupper (lavest når foten treffer, eller midt i steget i
        # løp), vugger over foten som står, og hofta dreier med skrittet.
        bob = -st["bob"] * math.cos(4 * math.pi * (ph - lowest))
        if st.get("heavy"):
            bob = -st["bob"] * (1 - abs(math.sin(2 * math.pi * (ph - 0.02)))) * 1.6 + st["bob"] * 0.6
        sway = st["sway"] * math.cos(2 * math.pi * (ph - duty / 2))
        surge = 0.015 * math.sin(4 * math.pi * ph)
        spec.root = Vector((surge, sway, crouch + bob))
        yaw = st["hips_yaw"] * math.cos(2 * math.pi * ph)
        roll = 4 * math.cos(2 * math.pi * (ph - duty / 2)) * (1.8 if st.get("heavy") else 1)
        add(spec, "hips", x=roll, z=-yaw)
        add(spec, "chest", x=-roll * 0.6, y=2 * math.sin(4 * math.pi * ph), z=yaw * 0.85)
        add(spec, "neck", z=yaw * 0.1)
        # Hodet holdes stødig: det motvirker duppen.
        add(spec, "head", y=-bob * 60, z=yaw * 0.1)
        add(spec, "jaw", y=6 * math.sin(4 * math.pi * ph + 1.2))
        # Armene svinger mot beina, underarmen og hånda henger etter.
        if not st.get("quad"):
            for s, sgn in (("L", 1), ("R", -1)):
                w = 2 * math.pi * ph + (0 if s == "L" else math.pi)
                add(spec, f"upper_arm.{s}", y=st["arm"] * math.cos(w))
                add(spec, f"forearm.{s}", y=st["arm"] * 0.55 * math.cos(w - 0.9))
                add(spec, f"hand.{s}", y=st["arm"] * 0.4 * math.cos(w - 1.8))
        else:
            # På alle fire: hendene går i diagonal med motsatt fot.
            for s, sy, off in (("L", 1, 0.5), ("R", -1, 0.0)):
                u = (ph + off + 0.08) % 1.0
                if u < duty:
                    k = u / duty
                    x = 0.62 + D / 2 - D * k
                    z = 0.12
                else:
                    k = (u - duty) / (1 - duty)
                    x = 0.62 - D / 2 + D * (0.5 - 0.5 * math.cos(math.pi * k))
                    z = 0.12 + st["lift"] * 1.2 * math.sin(math.pi * k)
                spec.hands[s] = (x, sy * (st["width"] + 0.08), z)
        # Halen svinger med, forsinket ledd for ledd.
        for i, b in enumerate(("tail1", "tail2", "tail3")):
            add(spec, b, y=3 * math.sin(4 * math.pi * ph - i), z=st["tail"] * math.sin(2 * math.pi * ph - 0.8 * (i + 1)))
        if st.get("hop"):
            # Tooltipen hopper litt i svevet.
            add(spec, "head", y=-4 * math.sin(4 * math.pi * ph))
        body.key(c, f, spec)


def idle(c, body, st):
    T = st["idle"]
    crouch = -0.03 - (0.06 if st.get("heavy") else 0.0) - (0.1 if st.get("quad") else 0.0)
    for f in range(0, T + 1, 2):
        ph = f / T
        w = 2 * math.pi * ph
        spec = base_pose(body, st, Spec(), crouch)
        breathe = math.sin(w * 2)
        spec.root = Vector((0.0, 0.025 * math.sin(w), crouch + 0.012 * breathe))
        add(spec, "hips", x=-3 * math.sin(w))
        add(spec, "chest", x=2 * math.sin(w), y=-3 * breathe)
        spec.scale["chest"] = 1.0 + 0.025 * breathe
        look = 14 * math.sin(w) + 6 * math.sin(3 * w + 0.5)
        add(spec, "neck", z=look * 0.35)
        add(spec, "head", y=-3 * abs(math.sin(w)), z=look * 0.65)
        add(spec, "jaw", y=6 + 6 * math.sin(2 * w + 0.4))
        for s, sgn in (("L", 1), ("R", -1)):
            add(spec, f"upper_arm.{s}", x=3 * sgn * math.sin(w + 0.6), y=4 * math.sin(w + (0 if s == "L" else 1.5)))
            add(spec, f"hand.{s}", y=8 * math.sin(2 * w + (0.3 if s == "L" else 2.0)))
        for i, b in enumerate(("tail1", "tail2", "tail3")):
            add(spec, b, z=(16 + 6 * i) * math.sin(w - 0.9 * (i + 1)), y=4 * math.sin(2 * w - i))
        # Digitale rykk: hodet og en arm hakker til sides et par bilder.
        if st.get("twitch"):
            for at, amp in ((0.3, 1.0), (0.72, -0.8)):
                d = abs(ph - at) * T
                if d < 3:
                    j = (1 - d / 3) * amp
                    add(spec, "head", z=28 * j, x=12 * j)
                    add(spec, "jaw", y=25 * abs(j))
                    add(spec, "upper_arm.R", y=-30 * abs(j))
        body.key(c, f, spec)


def alert(c, body, st):
    """Får øye på deg: samler seg, rykker opp med gapet og armene ute, og
    senker seg til en lurende kampstilling."""
    T = 22
    crouch = -0.03 - (0.06 if st.get("heavy") else 0.0) - (0.1 if st.get("quad") else 0.0)
    rise = curve([(0, 0.0), (5, -0.08), (9, 0.06), (15, 0.04), (22, -0.02)])
    pitch = curve([(0, 0.0), (5, 12.0), (9, -22.0), (15, -18.0), (22, 6.0)])
    head = curve([(0, 0.0), (5, 10.0), (9, -30.0), (15, -24.0), (22, 0.0)])
    jaw = curve([(0, 10.0), (6, 15.0), (9, 55.0), (16, 48.0), (22, 22.0)])
    spread = curve([(0, 0.0), (5, -10.0), (9, 42.0), (15, 38.0), (22, 12.0)])
    hop = curve([(0, 0.0), (7, 0.0), (10, 0.18), (13, 0.0)]) if st.get("hop") else (lambda f: 0.0)
    for f in range(T + 1):
        spec = base_pose(body, st, Spec(), crouch)
        spec.root = Vector((0.0, 0.0, crouch + rise(f) + hop(f)))
        if hop(f) > 0:
            for s in ("L", "R"):
                x, y, z, p = spec.feet[s]
                spec.feet[s] = (x, y, z + hop(f) * 0.8, 10)
        tremble = 3 * math.sin(f * 2.6) if 9 <= f <= 15 else 0
        add(spec, "hips", y=pitch(f) * 0.4)
        add(spec, "chest", y=pitch(f) * 0.6, x=tremble)
        add(spec, "head", y=head(f), z=tremble * 2)
        add(spec, "jaw", y=jaw(f))
        if not st.get("quad"):
            for s, sgn in (("L", 1), ("R", -1)):
                add(spec, f"upper_arm.{s}", x=sgn * spread(f), y=-spread(f) * 0.6)
                add(spec, f"forearm.{s}", y=-spread(f) * 0.4)
        add(spec, "tail1", y=-spread(f) * 0.3)
        add(spec, "tail2", y=-spread(f) * 0.2, z=tremble * 3)
        body.key(c, f, spec)


def hit(c, body, st, side=1):
    """Rykker bakover og til sida, overskyter litt fram, og samler seg."""
    T = 14
    crouch = -0.03 - (0.06 if st.get("heavy") else 0.0) - (0.1 if st.get("quad") else 0.0)
    heavy = 0.6 if st.get("heavy") else 1.0
    back = curve([(0, 0.0), (2, -0.16 * heavy), (7, 0.04 * heavy), (14, 0.0)], snap)
    rear = curve([(0, 0.0), (2, -26.0 * heavy), (7, 8.0 * heavy), (14, 0.0)], snap)
    turn = curve([(0, 0.0), (3, 32.0 * side), (8, -8.0 * side), (14, 0.0)], snap)
    jaw = curve([(0, 12.0), (2, 45.0), (8, 20.0), (14, 12.0)])
    flail = curve([(0, 0.0), (3, -45.0), (9, 10.0), (14, 0.0)], snap)
    for f in range(T + 1):
        spec = base_pose(body, st, Spec(), crouch)
        spec.root = Vector((back(f), 0.04 * side * heavy * math.sin(min(f, 7) / 7 * math.pi), crouch - abs(back(f)) * 0.3))
        add(spec, "hips", y=rear(f) * 0.3, z=turn(f) * 0.2)
        add(spec, "chest", y=rear(f) * 0.7, x=-turn(f) * 0.3, z=turn(f) * 0.3)
        add(spec, "head", y=rear(f) * 0.6, z=turn(f), x=turn(f) * 0.3)
        add(spec, "jaw", y=jaw(f))
        if not st.get("quad"):
            add(spec, "upper_arm.L", y=flail(f), x=12 * side)
            add(spec, "upper_arm.R", y=flail(f) * 0.7, x=12 * side)
            add(spec, "forearm.L", y=flail(f) * 0.5)
            add(spec, "forearm.R", y=flail(f) * 0.5)
        add(spec, "tail1", z=-turn(f) * 0.6)
        add(spec, "tail2", z=-turn(f) * 0.8)
        body.key(c, f, spec)


def stagger(c, body, st):
    """Svimmel etter en parade eller et tungt slag: vakler, hodet henger og
    slenger, armene dingler, knærne svikter."""
    T = 30
    crouch = -0.1 - (0.06 if st.get("heavy") else 0.0) - (0.1 if st.get("quad") else 0.0)
    for f in range(0, T + 1, 2):
        w = 2 * math.pi * f / T
        spec = base_pose(body, st, Spec(), crouch)
        spec.root = Vector((0.02 * math.sin(2 * w), 0.06 * math.sin(w), crouch + 0.02 * math.sin(2 * w + 1)))
        add(spec, "hips", x=-6 * math.sin(w), y=6)
        add(spec, "chest", x=8 * math.sin(w - 0.5), y=14 + 4 * math.sin(2 * w))
        add(spec, "neck", y=10, z=10 * math.sin(w - 1.0))
        add(spec, "head", y=22 + 6 * math.sin(2 * w + 0.5), z=24 * math.sin(w - 1.4), x=12 * math.sin(w - 1.2))
        add(spec, "jaw", y=32 + 8 * math.sin(2 * w))
        if not st.get("quad"):
            for s, sgn in (("L", 1), ("R", -1)):
                add(spec, f"upper_arm.{s}", x=-sgn * 6, y=18 + 10 * math.sin(w + (0 if s == "L" else 2)))
                add(spec, f"forearm.{s}", y=12 + 8 * math.sin(w - 0.8))
                add(spec, f"hand.{s}", y=20 * math.sin(w - 1.6))
        for i, b in enumerate(("tail1", "tail2", "tail3")):
            add(spec, b, y=10, z=10 * math.sin(w - 0.8 * (i + 1)))
        body.key(c, f, spec)


def die(c, body, st, forward=False):
    """Skriker, knærne svikter, og den segner om, på sida eller framover på
    tryne, og blir liggende. Beina går fra å stå til å folde seg."""
    T = 36
    crouch = -0.03 - (0.06 if st.get("heavy") else 0.0) - (0.1 if st.get("quad") else 0.0)
    sink = curve([(0, 0.0), (8, 0.04), (18, -0.35), (26, -0.62), (30, -0.58), (36, -0.62)])
    rear = curve([(0, 0.0), (8, -26.0), (18, 10.0), (36, 4.0)])
    jaw = curve([(0, 12.0), (8, 55.0), (20, 30.0), (36, 28.0)])
    if forward:
        tip = curve([(0, 0.0), (14, 0.0), (26, 72.0), (30, 66.0), (36, 70.0)])
        side = curve([(0, 0.0), (36, 8.0)])
        slide = curve([(0, 0.0), (16, 0.0), (28, 0.28), (36, 0.3)])
    else:
        tip = curve([(0, 0.0), (36, 10.0)])
        side = curve([(0, 0.0), (14, 6.0), (26, 80.0), (30, 74.0), (36, 78.0)])
        slide = curve([(0, 0.0), (36, 0.08)])
    legs = curve([(0, 0.0), (10, 0.0), (22, 1.0), (36, 1.0)])
    for f in range(T + 1):
        spec = base_pose(body, st, Spec(), crouch)
        spec.feet = {}
        spec.hands = {}
        spec.root = Vector((slide(f), 0.12 * side(f) / 80, crouch + sink(f)))
        add(spec, "hips", x=side(f), y=tip(f) * 0.6)
        add(spec, "chest", y=rear(f) + tip(f) * 0.4, x=side(f) * 0.1)
        add(spec, "head", y=rear(f) * 0.8 - tip(f) * 0.3, z=20 * legs(f))
        add(spec, "jaw", y=jaw(f))
        k = legs(f)
        for s, sgn in (("L", 1), ("R", -1)):
            if forward:
                # På kne og så på tryne: lårene fram, leggene bak.
                spec.rot[f"thigh.{s}"] = (0, -60 * k - tip(f) * 0.5, 0)
                spec.rot[f"shin.{s}"] = (0, 110 * k, 0)
                spec.rot[f"foot.{s}"] = (0, -40 * k, 0)
                arm_out = curve([(0, 0.0), (8, -40.0), (20, -60.0), (36, -80.0)])(f)
                add(spec, f"upper_arm.{s}", x=sgn * 25 * k, y=arm_out)
                add(spec, f"forearm.{s}", y=-10 * k)
            else:
                # På sida: beina slappe, armen under kroppen legges opp langs
                # hodet, den andre faller fram over brystet.
                spec.rot[f"thigh.{s}"] = (0, -25 * k + sgn * 8 * k, 0)
                spec.rot[f"shin.{s}"] = (0, 35 * k, 0)
                spec.rot[f"foot.{s}"] = (0, -15 * k, 0)
                if s == "R":
                    add(spec, "upper_arm.R", x=-20 * k, y=curve([(0, 0.0), (8, -40.0), (20, -90.0), (36, -120.0)])(f))
                    add(spec, "forearm.R", y=-10 * k)
                else:
                    add(spec, "upper_arm.L", x=-35 * k, y=curve([(0, 0.0), (8, -40.0), (20, 10.0), (36, 25.0)])(f))
                    add(spec, "forearm.L", y=-25 * k)
        for i, b in enumerate(("tail1", "tail2", "tail3")):
            add(spec, b, y=-10 * k - 4 * i, z=8 * k)
        body.key(c, f, spec, ik_legs=False, slack=0.08 * legs(f))


def attack_bite(c, body, st):
    """Varsler: reiser seg, gaper og trekker armene bak (bilde 0–13), stuper
    fram med et skritt og smeller kjeven igjen ved bilde 15, og trekker seg
    tilbake."""
    T = 30
    crouch = -0.03 - (0.06 if st.get("heavy") else 0.0) - (0.1 if st.get("quad") else 0.0)
    lunge = curve([(0, 0.0), (10, -0.14), (13, -0.16), (16, 0.5), (22, 0.4), (30, 0.0)])
    lunge_e = curve([(13, -0.16), (16, 0.5)], snap)
    rise = curve([(0, 0.0), (10, 0.03), (13, 0.04), (16, -0.06), (22, -0.03), (30, 0.0)])
    spine = curve([(0, 0.0), (10, -12.0), (13, -14.0), (16, 24.0), (22, 10.0), (30, 0.0)])
    head = curve([(0, 0.0), (10, -20.0), (13, -24.0), (15, 16.0), (22, 4.0), (30, 0.0)])
    jaw = curve([(0, 12.0), (10, 48.0), (14, 50.0), (15, 0.0), (20, 10.0), (30, 12.0)])
    claw = curve([(0, 0.0), (10, -42.0), (13, -46.0), (16, 50.0), (22, 18.0), (30, 0.0)])
    step = curve([(0, 0.0), (12, 0.0), (16, 0.42), (24, 0.42), (30, 0.0)])
    lift = curve([(0, 0.0), (12, 0.0), (14, 0.12), (16, 0.0), (24, 0.0), (27, 0.08), (30, 0.0)])
    hop = curve([(0, 0.0), (12, 0.0), (14, 0.22), (17, 0.0)]) if st.get("hop") else (lambda f: 0.0)
    for f in range(T + 1):
        spec = base_pose(body, st, Spec(), crouch)
        x = lunge_e(f) if 13 <= f <= 16 else lunge(f)
        tremble = 2.5 * math.sin(f * 3.1) if 9 <= f <= 13 else 0
        spec.root = Vector((x, 0.0, crouch + rise(f) + hop(f)))
        # Fremste fot tar et skritt fram i stupet; den andre står.
        xl, yl, zl, _ = spec.feet["L"]
        spec.feet["L"] = (xl + step(f), yl, zl + lift(f) + hop(f), 0)
        if hop(f) > 0:
            xr, yr, zr, _ = spec.feet["R"]
            spec.feet["R"] = (xr + step(f) * 0.6, yr, zr + hop(f), 10)
        add(spec, "hips", y=spine(f) * 0.35, x=tremble)
        add(spec, "chest", y=spine(f) * 0.65, x=-tremble)
        add(spec, "neck", y=head(f) * 0.3)
        add(spec, "head", y=head(f) * 0.7, z=tremble * 2)
        add(spec, "jaw", y=jaw(f))
        if not st.get("quad"):
            for s, sgn in (("L", 1), ("R", -1)):
                add(spec, f"upper_arm.{s}", y=claw(f), x=sgn * max(0.0, -claw(f)) * 0.4)
                add(spec, f"forearm.{s}", y=claw(f) * 0.5)
                add(spec, f"hand.{s}", y=claw(f) * 0.3)
        else:
            # Tabellen kaster forkroppen opp og slår ned med begge klørne.
            for s, sy in (("L", 1), ("R", -1)):
                hx, hy, hz = spec.hands[s]
                up = curve([(0, 0.0), (10, 0.45), (13, 0.5), (16, 0.0), (30, 0.0)])(f)
                spec.hands[s] = (hx + step(f) + x * 0.5 - up * 0.3, hy, hz + up)
        for i, b in enumerate(("tail1", "tail2", "tail3")):
            add(spec, b, y=-spine(f) * (0.4 + 0.2 * i), z=tremble * 4)
        body.key(c, f, spec)


def attack_spit(c, body, st):
    """Kaster hodet bakover med armene bak (varselet), og slenger det fram
    med gapet åpent ved bilde 15: markøren går."""
    T = 30
    crouch = -0.03
    spine = curve([(0, 0.0), (10, -20.0), (14, -24.0), (15, 18.0), (22, 6.0), (30, 0.0)], snap)
    head = curve([(0, 0.0), (10, -32.0), (14, -34.0), (15, 26.0), (22, 8.0), (30, 0.0)], snap)
    jaw = curve([(0, 12.0), (10, 35.0), (14, 30.0), (15, 60.0), (22, 18.0), (30, 12.0)])
    back = curve([(0, 0.0), (10, -0.1), (14, -0.12), (16, 0.2), (30, 0.0)])
    # Armene (med spydene) slenges bakover i varselet og fram i spyttet.
    arms = curve([(0, 0.0), (10, 45.0), (14, 50.0), (15, -25.0), (22, -10.0), (30, 0.0)], snap)
    for f in range(T + 1):
        spec = base_pose(body, st, Spec(), crouch)
        tremble = 2 * math.sin(f * 3.3) if 8 <= f <= 14 else 0
        spec.root = Vector((back(f), 0.0, crouch + 0.03 * math.sin(min(1.0, f / 14) * math.pi)))
        add(spec, "hips", y=spine(f) * 0.3)
        add(spec, "chest", y=spine(f) * 0.7, x=tremble)
        add(spec, "neck", y=head(f) * 0.35)
        add(spec, "head", y=head(f) * 0.65, z=tremble * 2)
        add(spec, "jaw", y=jaw(f))
        for s, sgn in (("L", 1), ("R", -1)):
            add(spec, f"upper_arm.{s}", x=sgn * 20, y=arms(f))
            add(spec, f"forearm.{s}", y=arms(f) * 0.3)
        for i, b in enumerate(("tail1", "tail2", "tail3")):
            add(spec, b, y=-spine(f) * 0.5, z=tremble * 4)
        body.key(c, f, spec)


def attack_slam(c, body, st):
    """Løfter begge nevene høyt over hodet (varselet, lenge), og slår dem i
    bakken foran seg ved bilde 28, med knærne inn i slaget."""
    T = 44
    crouch = -0.03 - (0.06 if st.get("heavy") else 0.0)
    spine = curve([(0, 0.0), (18, -26.0), (26, -30.0), (28, 34.0), (36, 28.0), (44, 0.0)])
    root_x = curve([(0, 0.0), (18, -0.12), (26, -0.1), (28, 0.3), (36, 0.26), (44, 0.0)])
    sink = curve([(0, 0.0), (18, 0.06), (26, 0.08), (28, -0.2), (36, -0.16), (44, 0.0)])
    raise_ = curve([(0, 0.0), (18, -155.0), (26, -162.0), (27, -80.0), (28, 0.0)])
    head = curve([(0, 0.0), (18, -22.0), (28, 22.0), (44, 0.0)])
    jaw = curve([(0, 12.0), (18, 34.0), (28, 50.0), (44, 12.0)])
    step = curve([(0, 0.0), (22, 0.0), (28, 0.35), (38, 0.35), (44, 0.0)])
    lift = curve([(0, 0.0), (22, 0.0), (25, 0.14), (28, 0.0), (38, 0.0), (41, 0.08), (44, 0.0)])
    for f in range(T + 1):
        spec = base_pose(body, st, Spec(), crouch)
        tremble = 2.5 * math.sin(f * 2.9) if 16 <= f <= 26 else 0
        quake = 3 * math.sin((f - 28) * 4.0) * max(0.0, 1 - (f - 28) / 8) if f >= 28 else 0
        spec.root = Vector((root_x(f), 0.0, crouch + sink(f)))
        xl, yl, zl, _ = spec.feet["L"]
        spec.feet["L"] = (xl + step(f), yl, zl + lift(f), 0)
        add(spec, "hips", y=spine(f) * 0.35)
        add(spec, "chest", y=spine(f) * 0.65, x=tremble + quake)
        add(spec, "head", y=head(f), z=quake * 2)
        add(spec, "jaw", y=jaw(f))
        if f < 28:
            for s, sgn in (("L", 1), ("R", -1)):
                add(spec, f"upper_arm.{s}", x=sgn * 18, y=raise_(f))
                add(spec, f"forearm.{s}", y=curve([(0, 0.0), (18, -35.0), (28, 10.0)])(f))
        else:
            # Nevene i bakken foran føttene, til de løftes igjen.
            k = curve([(28, 1.0), (36, 1.0), (44, 0.0)])(f)
            for s, sy in (("L", 1), ("R", -1)):
                spec.hands[s] = (0.8 + root_x(f) * 0.5, sy * 0.32, 0.14 + (1 - k) * 0.6)
            spec.rot["hand_world.L"] = (0, 70, 0)
            spec.rot["hand_world.R"] = (0, 70, 0)
        for i, b in enumerate(("tail1", "tail2", "tail3")):
            add(spec, b, y=-spine(f) * 0.4, z=quake * 4)
        body.key(c, f, spec)


def roar(c, body, st):
    """Reiser seg, slår ut med armene og brøler: kjeven helt åpen, brystet
    svulmer, hele kroppen rister."""
    T = 36
    crouch = -0.06
    up = curve([(0, 0.0), (12, 1.0), (26, 1.0), (36, 0.0)])
    for f in range(T + 1):
        k = up(f)
        shake = 2.5 * math.sin(f * 3.7) * (1 if 12 <= f <= 26 else 0)
        spec = base_pose(body, st, Spec(), crouch)
        spec.root = Vector((-0.06 * k, 0.0, crouch + 0.08 * k))
        add(spec, "hips", y=-18 * k)
        add(spec, "chest", y=-24 * k, x=shake)
        add(spec, "neck", y=-10 * k)
        add(spec, "head", y=-30 * k, z=shake * 3)
        add(spec, "jaw", y=58 * k)
        spec.scale["chest"] = 1.0 + 0.12 * k
        for s, sgn in (("L", 1), ("R", -1)):
            add(spec, f"upper_arm.{s}", x=sgn * 65 * k, y=-35 * k)
            add(spec, f"forearm.{s}", y=-40 * k)
            add(spec, f"hand.{s}", y=-20 * k + shake * 3)
        for i, b in enumerate(("tail1", "tail2", "tail3")):
            add(spec, b, y=-20 * k, z=shake * 6)
        body.key(c, f, spec)


ATTACKS = {"bite": (attack_bite, 30), "spit": (attack_spit, 30), "slam": (attack_slam, 44)}


def make_clips(Clip, arm, bones, style, attack, extra=()):
    """Alle klippene for én fiende."""
    body = Body(bones)
    st = STYLES[style]
    todo = [
        ("idle", st["idle"], lambda c: idle(c, body, st)),
        ("walk", st["frames"], lambda c: walk(c, body, st)),
        ("attack", ATTACKS[attack][1], lambda c: ATTACKS[attack][0](c, body, st)),
        ("alert", 22, lambda c: alert(c, body, st)),
        ("hit", 14, lambda c: hit(c, body, st, 1)),
        ("hit2", 14, lambda c: hit(c, body, st, -1)),
        ("stagger", 30, lambda c: stagger(c, body, st)),
        ("die", 36, lambda c: die(c, body, st, False)),
        ("die2", 36, lambda c: die(c, body, st, True)),
    ]
    for name in extra:
        if name == "slam":
            todo.append(("slam", 44, lambda c: attack_slam(c, body, st)))
        elif name == "roar":
            todo.append(("roar", 36, lambda c: roar(c, body, st)))
    for name, length, fn in todo:
        c = Clip(arm, name, length, FPS)
        fn(c)
        c.finish()
