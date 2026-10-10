"""Innbyggerne i den malte stilen, formet etter eierens forbilder fra Gemini
(`assets/textures/people/ref/<navn>_body.jpeg` forfra, fra siden og bakfra,
og `<navn>_head.jpeg` med hodet i fire vinkler). Kjør:

    python3.13 tools/models/folk.py smith|innkeeper|gorvek [--preview ut.png] [--fit ut.png] [størrelse]

Fremgangsmåten (se docs/prompter-folk.md):

1. **Forbildene måles.** Figuren skilles fra den grå bakgrunnen (`Sheet`),
   og visningene deles opp. Skalaen er høyden i piksler; leddene (skuldre,
   albuer, håndledd, hofter, knær, ankler) er målt for hånd i `FIGURES`.
2. **Kroppen** er MakeHumans grunnmodell (CC0) med armene posert ned og
   vektene samlet på spillets 18 bein. Den rettes inn etter leddene i
   forbildet (`retarget`: hvert bein dreies og strekkes, så albuen,
   håndleddet og kneet havner der de er i tegningen), og formes så etter
   silhuettene (`fit_rows`): rad for rad strekkes bredden forfra og
   dybden fra siden til silhuettene i forbildet, del for del (armene,
   overkroppen og beina hver for seg).
3. **Hodet** formes etter hodearket: profilen (panne, nese, lepper, hake)
   og bredden forfra, og øynene, nesen og munnen flyttes dit de er i
   tegningen (`fit_head`). Øynene er egne kuler med malt iris.
4. **Hår, skjegg, skaut og pels** er store, glatte lokker og flak
   (`lock`), lagt etter formen i forbildet.
5. **Klærne** er kroppens egen flate (skjorta, buksa, støvlene), og det
   som henger fritt (forkle, skjørt, pelskrage) er egne skall.
6. **Teksturen** males av forbildene: hvert tekstel får fargen fra den
   visningen som ser det best (forfra, fra sidene, bakfra), bare der
   flaten faktisk synes i den visningen (`paint`), hodet fra hodearket og
   resten fra kroppsarket. Det ingen visning ser, får fargen fra nærmeste
   synlige punkt. Normalkartet lages av de malte detaljene.

Trenger numpy, scipy og opencv i Blenders Python
(`python3.13 -m pip install scipy opencv-python-headless`).
MakeHuman-dataene ligger i .claude/makehuman/ (se people.py). Origo i
bakken mellom føttene, fronten mot +X, venstre mot +Y."""

import gzip
import json
import math
import os
import subprocess
import sys

import cv2
import numpy as np
from scipy import ndimage as ndi

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402
from mathutils.bvhtree import BVHTree  # noqa: E402

OUT = "assets/models/npc"
REF = os.path.abspath("assets/textures/people/ref")
MH = os.path.abspath(".claude/makehuman")
TMP = os.environ.get("FOLK_TMP", "/tmp/folk")


# --- Figurene ------------------------------------------------------------------
#
# Leddene er målt i kroppsarket (piksler i hele bildet): `front` (kolonne,
# rad) for høyre (R, til venstre i bildet) og venstre (L) side, `side`
# (kolonne, rad) i sidevisningen (figuren ser mot høyre, så vi ser høyre
# side). `cf` er midtlinja forfra (nesen). `skip` er radene der silhuetten
# ikke skal styre kroppen (forkle, skjørt, pels), per visning.

FIGURES = {
    "smith": {
        "height": 1.80, "macro": (1.0, 0.5, 0.75, 0.45), "cf": 400,
        "front": {"shoulder": ((190, 545), (620, 545)), "elbow": ((112, 800), (698, 800)), "wrist": ((97, 1085), (703, 1095)),
                  "hip": ((305, 1090), (495, 1090)), "knee": ((250, 1540), (540, 1540)), "ankle": ((215, 1862), (570, 1862))},
        "side": {"shoulder": (945, 548), "elbow": (878, 800), "wrist": (990, 1082), "hip": (985, 1090), "knee": (975, 1540), "ankle": (962, 1862)},
        "hand": 1.25, "grip": "R", "neck_row": 470,
        "face": {"eyes": ((427, 415), (602, 415)), "nose": (505, 530), "mouth": (505, 612), "side_eye": (1717, 423), "ear": (1455, 470)},
        "skip": {"front": ((1150, 1380),), "side": ((1000, 1390),)},
    },
    "innkeeper": {
        "height": 1.66, "macro": (0.0, 0.5, 0.3, 0.75), "cf": 375,
        "front": {"shoulder": ((188, 612), (565, 612)), "elbow": ((105, 840), (650, 840)), "wrist": ((85, 1030), (675, 1030)),
                  "hip": ((300, 1040), (450, 1040)), "knee": ((285, 1440), (468, 1440)), "ankle": ((265, 1800), (495, 1800))},
        "side": {"shoulder": (960, 605), "elbow": (950, 850), "wrist": (1015, 1040), "hip": (1000, 1040), "knee": (995, 1440), "ankle": (980, 1800)},
        "hand": 1.2, "grip": "L", "neck_row": 520,
        "face": {"eyes": ((210, 220), (304.3, 220)), "nose": (255.7, 262.9), "mouth": (257, 304), "side_eye": (378.3, 713.3), "ear": (256.7, 756.7)},
    },
    "gorvek": {
        "height": 1.98, "macro": (1.0, 0.35, 1.0, 0.35), "cf": 398,
        "front": {"shoulder": ((180, 565), (622, 565)), "elbow": ((110, 825), (690, 825)), "wrist": ((100, 1030), (690, 1030)),
                  "hip": ((312, 935), (488, 935)), "knee": ((265, 1450), (525, 1450)), "ankle": ((240, 1822), (560, 1822))},
        "side": {"shoulder": (995, 570), "elbow": (975, 830), "wrist": (1045, 1030), "hip": (1030, 935), "knee": (998, 1450), "ankle": (978, 1822)},
        "hand": 1.25, "grip": "R", "neck_row": 425,
        "face": {"eyes": ((201.4, 204.3), (282.9, 204.3)), "nose": (240, 240), "mouth": (240, 275.7), "side_eye": (328.4, 693.8), "ear": (215.9, 715.6)},
    },
}


# --- Forbildene ----------------------------------------------------------------


def load_rgb(path):
    return cv2.imread(path, cv2.IMREAD_COLOR)[:, :, ::-1].astype(np.float32) / 255.0


def segment(img):
    """Figuren mot den flate grå bakgrunnen: alt som skiller seg fra
    bakgrunnsfargen eller har kanter, med små hull fylt og bakkelinja
    fjernet."""
    a = img * 255.0
    bg = np.median(np.concatenate([a[:40].reshape(-1, 3), a[:, :40].reshape(-1, 3)]), 0)
    d = np.abs(a - bg).max(2)
    g = cv2.GaussianBlur(a, (0, 0), 1.5).mean(2)
    gm = np.hypot(cv2.Sobel(g, cv2.CV_32F, 1, 0), cv2.Sobel(g, cv2.CV_32F, 0, 1))
    m = ((d > 14) | (gm > 25)).astype(np.uint8)
    m = cv2.morphologyEx(m, cv2.MORPH_CLOSE, np.ones((7, 7), np.uint8)).astype(bool)
    holes = ndi.binary_fill_holes(m) & ~m
    hl, hn = ndi.label(holes)
    if hn:
        hs = ndi.sum(holes, hl, range(1, hn + 1))
        m |= np.isin(hl, 1 + np.where(hs < 1500 * (a.shape[0] / 2048) ** 2)[0])
    m = cv2.morphologyEx(m.astype(np.uint8), cv2.MORPH_OPEN, np.ones((17, 1), np.uint8)).astype(bool)
    lab, n = ndi.label(m)
    if n:
        sz = ndi.sum(m, lab, range(1, n + 1))
        m = np.isin(lab, 1 + np.where(sz > 2000 * (a.shape[0] / 2048) ** 2)[0])
    m = cv2.morphologyEx(m.astype(np.uint8), cv2.MORPH_OPEN, np.ones((5, 5), np.uint8)).astype(bool)
    return m


class View:
    """Én visning i et ark: bildet, masken og hvordan et punkt i figurens
    rom (x fram, y venstre, z opp, meter) havner i bildet. `axis` er
    retningen kameraet ser fra (enhetsvektor ut fra figuren), `right`
    retningen som går mot høyre i bildet."""

    def __init__(self, img, mask, axis, right, scale, origin):
        self.img, self.mask = img, mask
        self.axis, self.right = Vector(axis), Vector(right)
        self.s = scale  # piksler per meter
        self.o = origin  # (kolonne, rad) for punktet (0, 0, 0)

    def px(self, p):
        """(kolonne, rad) for et punkt."""
        return (self.o[0] + p.dot(self.right) * self.s, self.o[1] - p.z * self.s)

    def pxs(self, pts):
        """Som px for en (n, 3)-tabell."""
        r = np.array(self.right[:])
        return np.stack([self.o[0] + pts @ r * self.s, self.o[1] - pts[:, 2] * self.s], 1)

    def depth(self, pts):
        """Hvor nær kameraet punktene er (meter langs aksen)."""
        return pts @ np.array(self.axis[:])


class Sheet:
    """Kroppsarket: tre visninger side om side, samme skala og bakkelinje."""

    def __init__(self, name, fig):
        img = load_rgb(f"{REF}/{name}_body.jpeg")
        mask = segment(img)
        h, w = mask.shape
        occ = mask.sum(0)
        cuts = []
        for c in (w / 3, 2 * w / 3):
            lo, hi = int(c - w * 0.08), int(c + w * 0.08)
            cuts.append(lo + int(np.argmin(occ[lo:hi])))
        parts = [(0, cuts[0]), (cuts[0], cuts[1]), (cuts[1], w)]

        def sub(k):
            m = np.zeros_like(mask)
            a, b = parts[k]
            m[:, a:b] = mask[:, a:b]
            return m

        rows = np.where(sub(0).any(1))[0]
        self.ground = rows[-1]
        self.s = (rows[-1] - rows[0]) / fig["height"]
        self.cf = fig["cf"]
        # Sidevisningen: x = 0 ved ankelen.
        self.cs = fig["side"]["ankle"][0]
        # Bakfra: midtlinja der baksilhuetten passer best med den speilede
        # silhuetten forfra.
        front, back = sub(0), sub(2)
        fx = front.sum(0).astype(float)
        bx = back.sum(0).astype(float)
        fc = (fx * np.arange(w)).sum() / fx.sum()
        bc = (bx * np.arange(w)).sum() / bx.sum()
        best = None
        for d in range(-40, 41):
            c = bc + d
            # Speil forfra om cf og flytt til c.
            cols = np.arange(w)
            src = 2 * self.cf - (cols - c + self.cf)
            ok = (src >= 0) & (src < w)
            mirrored = np.zeros(w)
            mirrored[ok] = fx[src[ok].astype(int)]
            err = np.abs(mirrored - bx).sum()
            if best is None or err < best[0]:
                best = (err, c)
        self.cb = best[1]
        _ = fc
        self.img, self.mask = img, mask
        o = lambda c: (c, self.ground)  # noqa: E731
        self.front = View(img, sub(0), (1, 0, 0), (0, 1, 0), self.s, o(self.cf))
        self.side = View(img, sub(1), (0, -1, 0), (1, 0, 0), self.s, o(self.cs))
        self.back = View(img, sub(2), (-1, 0, 0), (0, -1, 0), self.s, o(self.cb))

    def joints(self, fig):
        """Leddene i figurens rom: {navn.side: Vector}."""
        out = {}
        for k, (r, l) in fig["front"].items():
            sc, sr = fig["side"][k]
            x = (sc - self.cs) / self.s
            for s, (c, row) in (("R", r), ("L", l)):
                z = (self.ground - (row + sr) / 2) / self.s
                out[f"{k}.{s}"] = Vector((x, (c - self.cf) / self.s, z))
        return out


# Hvilken rute i hodearket som er hvilken visning (kolonne, rad). «right»
# er høyre side av hodet (ansiktet mot høyre kant i bildet).
QUADS = {"smith": {"front": (0, 0), "right": (1, 0), "left": (0, 1), "back": (1, 1)},
         "innkeeper": {"front": (0, 0), "left": (1, 0), "right": (0, 1), "back": (1, 1)},
         "gorvek": {"front": (0, 0), "left": (1, 0), "right": (0, 1), "back": (1, 1)}}


def edges(img):
    g = cv2.GaussianBlur(img.mean(2), (0, 0), 1.2)
    return np.hypot(cv2.Sobel(g, cv2.CV_32F, 1, 0), cv2.Sobel(g, cv2.CV_32F, 0, 1))


class HeadSheet:
    """Hodearket: fire visninger, hver registrert mot hodet i kroppsarket
    (felles skala for arket, flytting per visning, funnet med
    krysskorrelasjon av kantene), så et punkt i figurens rom havner riktig
    i hver visning."""

    def __init__(self, name, fig, sheet):
        img = load_rgb(f"{REF}/{name}_head.jpeg")
        mask = segment(img)
        H, Wi = mask.shape
        hq, wq = H // 2, Wi // 2
        self.img, self.mask = img, mask
        crops = {}
        for vn, (qx, qy) in QUADS[name].items():
            bv = {"front": sheet.front, "back": sheet.back, "right": sheet.side, "left": sheet.side}[vn]
            m = bv.mask[:fig["neck_row"]]
            top = np.where(m.any(1))[0][0]
            r1 = top + int(0.30 * sheet.s)
            cols = np.where(m[top:top + int(0.2 * sheet.s)].any(0))[0]
            X0, Y0 = cols[0] - 10, top - 10
            crop = bv.img[Y0:r1, X0:cols[-1] + 10]
            if vn == "left":
                crop = crop[:, ::-1]
            crops[vn] = (bv, X0, Y0, crop.shape[1], edges(crop), (qx * wq, qy * hq))
        def search(vn, ks):
            bv, X0, Y0, Wc, ec, (ox, oy) = crops[vn]
            sub = img[oy:oy + hq, ox:ox + wq]
            best = (-2, None, None)
            for k in ks:
                es = edges(cv2.resize(sub, None, fx=k, fy=k, interpolation=cv2.INTER_AREA))
                if es.shape[0] < ec.shape[0] or es.shape[1] < ec.shape[1]:
                    continue
                r = cv2.matchTemplate(es, ec, cv2.TM_CCOEFF_NORMED)
                _, mx, _, loc = cv2.minMaxLoc(r)
                if mx > best[0]:
                    best = (mx, k, loc)
            return best

        # Forfra først over hele skalaen; de andre visningene nær den.
        f = search("front", np.linspace(0.15, 1.8, 166))
        locs = {"front": f}
        for vn in crops:
            if vn != "front":
                locs[vn] = search(vn, np.linspace(f[1] * 0.85, f[1] * 1.15, 31))
        print("hodearket: " + ", ".join(f"{vn} {m:.2f} (skala {k:.3f})" for vn, (m, k, _) in locs.items()))
        self.views = {}
        for vn, (bv, X0, Y0, Wc, ec, (ox, oy)) in crops.items():
            _, k, (lx, ly) = locs[vn]
            s = sheet.s / k
            row0 = (ly + bv.o[1] - Y0) / k + oy
            vm = np.zeros_like(mask)
            vm[oy:oy + hq, ox:ox + wq] = mask[oy:oy + hq, ox:ox + wq]
            if vn == "left":
                col0 = (lx + Wc - 1 - (bv.o[0] - X0)) / k + ox
                v = View(img, vm, (0, 1, 0), (-1, 0, 0), s, (col0, row0))
            else:
                col0 = (lx + bv.o[0] - X0) / k + ox
                v = View(img, vm, bv.axis, bv.right, s, (col0, row0))
            self.views[vn] = v


# --- MakeHuman -----------------------------------------------------------------


def fetch():
    """Grunnmodellen, formmålene og riggen fra MPFB (CC0)."""
    if os.path.exists(f"{MH}/3dobjs/base.obj"):
        return
    tmp = "/tmp/mpfb2"
    if not os.path.exists(tmp):
        subprocess.run(["git", "clone", "--depth", "1", "--filter=blob:none", "--sparse", "https://github.com/makehumancommunity/mpfb2.git", tmp], check=True)
        subprocess.run(["git", "-C", tmp, "sparse-checkout", "set", "src/mpfb/data/3dobjs", "src/mpfb/data/rigs/standard", "src/mpfb/data/targets/macrodetails"], check=True)
    os.makedirs(MH, exist_ok=True)
    for d in ("3dobjs", "rigs", "targets"):
        subprocess.run(["cp", "-r", f"{tmp}/src/mpfb/data/{d}", MH], check=True)
    subprocess.run(["cp", f"{tmp}/LICENSE.ASSETS.md", MH], check=True)


def load_obj():
    verts, faces, group = [], [], None
    for line in open(f"{MH}/3dobjs/base.obj"):
        if line.startswith("v "):
            verts.append(Vector(tuple(map(float, line.split()[1:4]))))
        elif line.startswith("g "):
            group = line.split()[1]
        elif line.startswith("f "):
            faces.append((group, [int(t.split("/")[0]) - 1 for t in line.split()[1:]]))
    return verts, faces


def apply_targets(verts, targets):
    for name, w in targets:
        if w <= 0:
            continue
        with gzip.open(f"{MH}/targets/macrodetails/{name}.target.gz", "rt") as f:
            for line in f:
                p = line.split()
                if len(p) == 4 and not line.startswith("#"):
                    verts[int(p[0])] += Vector((float(p[1]), float(p[2]), float(p[3]))) * w


def macro(gender, age, muscle, weight):
    out = []
    for g, gw in (("female", 1 - gender), ("male", gender)):
        for a, aw in (("young", 1 - age), ("old", age)):
            out.append((f"caucasian-{g}-{a}", gw * aw))
            for mu, mw in (("averagemuscle", 1 - muscle), ("maxmuscle", muscle)):
                for we, ww in (("averageweight", 1 - weight), ("maxweight", weight)):
                    out.append((f"universal-{g}-{a}-{mu}-{we}", gw * aw * mw * ww))
    return out


def our_bone(mh):
    """Hvilket av spillets 18 bein et MakeHuman-bein blir."""
    side = mh[-2:] if mh[-2:] in (".L", ".R") else ""
    base = mh[:-2] if side else mh
    if base in ("root", "spine05", "spine04", "pelvis"):
        return "hips"
    if base in ("spine03", "spine02"):
        return "spine"
    if base in ("spine01", "breast", "clavicle"):
        return "chest"
    if base.startswith("neck"):
        return "neck"
    if base in ("shoulder01", "upperarm01", "upperarm02"):
        return "upper_arm" + side
    if base.startswith("lowerarm"):
        return "forearm" + side
    if base == "wrist" or base.startswith("metacarpal") or base.startswith("finger"):
        return "hand" + side
    if base.startswith("upperleg"):
        return "thigh" + side
    if base.startswith("lowerleg"):
        return "shin" + side
    if base == "foot" or base.startswith("toe"):
        return "foot" + side
    return "head"


def link(name, bm_or_me):
    me = bm_or_me
    if isinstance(bm_or_me, bmesh.types.BMesh):
        me = bpy.data.meshes.new(name)
        bm_or_me.to_mesh(me)
        bm_or_me.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def template(fig, joints):
    """MakeHuman-kroppen med formmålene, armene posert ned mot albuene og
    håndleddene i forbildet og hendene rundt grepet, i spillets akser.
    Gir kroppen (vektene på spillets bein) og leddene."""
    fetch()
    verts, faces = load_obj()
    apply_targets(verts, macro(*fig["macro"]))
    keep = ("body",)
    used = sorted({i for g, f in faces if g in keep for i in f})
    remap = {old: new for new, old in enumerate(used)}

    def mp(v):
        return Vector((v.x, -v.z, v.y)) * 0.1

    bm = bmesh.new()
    bv = [bm.verts.new(mp(verts[i])) for i in used]
    for g, f in faces:
        if g in keep:
            try:
                bm.faces.new([bv[remap[i]] for i in f])
            except ValueError:
                pass
    body = link("body", bm)
    groups = {}
    for g, f in faces:
        if g.startswith("joint-"):
            groups.setdefault(g, set()).update(f)

    def joint(name):
        s = groups[name]
        return sum((mp(verts[i]) for i in s), Vector()) / len(s)

    rig = json.load(open(f"{MH}/rigs/standard/rig.default.json"))
    weights = json.load(open(f"{MH}/rigs/standard/weights.default.json"))["weights"]

    def place(d):
        st = d["strategy"]
        if st == "CUBE":
            return joint(d["cube_name"])
        if st == "VERTEX":
            return mp(verts[d["vertex_index"]])
        return sum((mp(verts[i]) for i in d["vertex_indices"]), Vector()) / len(d["vertex_indices"])

    data = bpy.data.armatures.new("mh")
    arm = bpy.data.objects.new("mh", data)
    bpy.context.scene.collection.objects.link(arm)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="EDIT")
    made = {}
    left = list(rig)
    while left:
        for name in list(left):
            par = rig[name].get("parent")
            if not par or par in made:
                d = rig[name]
                eb = data.edit_bones.new(name)
                eb.head = place(d["head"])
                eb.tail = place(d["tail"])
                if (eb.tail - eb.head).length < 1e-4:
                    eb.tail = eb.head + Vector((0, 0, 0.01))
                eb.roll = d.get("roll", 0.0)
                if par:
                    eb.parent = made[par]
                made[name] = eb
                left.remove(name)
    bpy.ops.object.mode_set(mode="OBJECT")
    for bone, vw in weights.items():
        vg = body.vertex_groups.new(name=bone)
        for i, w in vw:
            if i in remap:
                vg.add([remap[i]], w, "REPLACE")
    mod = body.modifiers.new("arm", "ARMATURE")
    mod.object = arm
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="POSE")

    def aim(bone, direction):
        pb = arm.pose.bones[bone]
        bpy.context.view_layer.update()
        cur = (pb.tail - pb.head).normalized()
        q = cur.rotation_difference(Vector(direction).normalized())
        head = pb.matrix.translation.copy()
        pb.matrix = Matrix.Translation(head) @ q.to_matrix().to_4x4() @ Matrix.Translation(-head) @ pb.matrix
        bpy.context.view_layer.update()

    # Retningene fra forbildet (spillets akser: x fram, y venstre) til
    # MakeHumans (x venstre, −y fram): (x, y, z) → (y, −x, z).
    def mh_dir(v):
        return Vector((v.y, -v.x, v.z))

    for s in ("L", "R"):
        ua = mh_dir(joints[f"elbow.{s}"] - joints[f"shoulder.{s}"])
        fa = mh_dir(joints[f"wrist.{s}"] - joints[f"elbow.{s}"])
        for b in ("upperarm01", "upperarm02"):
            aim(f"{b}.{s}", ua)
        for b in ("lowerarm01", "lowerarm02"):
            aim(f"{b}.{s}", fa)
        th = mh_dir(joints[f"knee.{s}"] - joints[f"hip.{s}"])
        sh = mh_dir(joints[f"ankle.{s}"] - joints[f"knee.{s}"])
        for b in ("upperleg01", "upperleg02"):
            aim(f"{b}.{s}", th)
        for b in ("lowerleg01", "lowerleg02"):
            aim(f"{b}.{s}", sh)
        grip = s in fig["grip"]
        curl = 70 if grip else 22
        for f in range(2, 6):
            for k in range(1, 4):
                pb = arm.pose.bones[f"finger{f}-{k}.{s}"]
                pb.rotation_mode = "XYZ"
                pb.rotation_euler = (math.radians(curl * (1.1 if k == 1 else 1.0)), 0, 0)
        for k in range(2, 4):
            pb = arm.pose.bones[f"finger1-{k}.{s}"]
            pb.rotation_mode = "XYZ"
            pb.rotation_euler = (math.radians(38 if grip else 12), 0, 0)
    bpy.context.view_layer.update()
    pose = {pb.name: (pb.head.copy(), pb.tail.copy()) for pb in arm.pose.bones}
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.modifier_apply(modifier="arm")
    sums = [dict() for _ in body.data.vertices]
    for v in body.data.vertices:
        for g in v.groups:
            ob = our_bone(body.vertex_groups[g.group].name)
            sums[v.index][ob] = sums[v.index].get(ob, 0.0) + g.weight
    for vg in list(body.vertex_groups):
        body.vertex_groups.remove(vg)
    made_g = {}
    for i, d in enumerate(sums):
        tot = sum(d.values())
        for bname, w in d.items():
            if bname not in made_g:
                made_g[bname] = body.vertex_groups.new(name=bname)
            made_g[bname].add([i], w / max(tot, 1e-9), "REPLACE")
    bpy.data.objects.remove(arm)
    # Spillets akser: front −y → +x.
    rot = Matrix.Rotation(math.radians(90), 4, "Z")
    body.data.transform(rot)
    for p in body.data.polygons:
        p.use_smooth = True
    j = {n: (rot @ a, rot @ b) for n, (a, b) in pose.items()}
    marks = {"eye.L": rot @ joint("joint-l-eye"), "eye.R": rot @ joint("joint-r-eye"), "mouth": rot @ joint("joint-mouth")}
    return body, j, marks


# --- Vektene som tabeller ------------------------------------------------------

BONES18 = ["root", "hips", "spine", "chest", "neck", "head", "upper_arm.L", "forearm.L", "hand.L", "upper_arm.R", "forearm.R", "hand.R",
           "thigh.L", "shin.L", "foot.L", "thigh.R", "shin.R", "foot.R"]


def weight_table(ob):
    """(n, 18)-tabell med vektene til hjørnene."""
    W = np.zeros((len(ob.data.vertices), len(BONES18)))
    names = {vg.index: vg.name for vg in ob.vertex_groups}
    col = {n: i for i, n in enumerate(BONES18)}
    for v in ob.data.vertices:
        for g in v.groups:
            n = names[g.group]
            if n in col:
                W[v.index, col[n]] += g.weight
    s = W.sum(1, keepdims=True)
    return W / np.maximum(s, 1e-9)


def set_weights(ob, W):
    for vg in list(ob.vertex_groups):
        ob.vertex_groups.remove(vg)
    for k, n in enumerate(BONES18):
        vg = ob.vertex_groups.new(name=n)
        idx = np.where(W[:, k] > 1e-4)[0]
        for i in idx:
            vg.add([int(i)], float(W[i, k]), "REPLACE")


def coords(ob):
    a = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", a)
    return a.reshape(-1, 3)


def set_coords(ob, P):
    ob.data.vertices.foreach_set("co", P.ravel())
    ob.data.update()


def W_of(W, *names):
    return sum(W[:, BONES18.index(n)] for n in names)


# --- Inn i forbildet ---------------------------------------------------------------


def segment_map(h, t, H, T, k_perp=1.0):
    """Affin avbildning som fører segmentet h→t til H→T: dreining,
    strekk langs beinet, og `k_perp` på tvers."""
    d0, d1 = t - h, T - H
    q = d0.normalized().rotation_difference(d1.normalized()).to_matrix()
    k = d1.length / max(d0.length, 1e-6)
    a = d0.normalized()
    S = Matrix.Identity(3) * k_perp + (k - k_perp) * Matrix(((a.x * a.x, a.x * a.y, a.x * a.z), (a.y * a.x, a.y * a.y, a.y * a.z), (a.z * a.x, a.z * a.y, a.z * a.z)))
    M = q @ S
    return M, H - M @ h


def retarget(body, j, J, fig, marks):
    """Fører kroppen inn i leddene i forbildet med lineær skinning:
    hvert av spillets bein får en affin avbildning som fører leddene dets
    til leddene i forbildet (`J`), og hjørnene blandes etter vektene. Gir de
    nye leddene."""
    P = coords(body)
    W = weight_table(body)
    T = {}
    mid = lambda a, b: (a + b) / 2  # noqa: E731
    # Overkroppen: fra midt mellom hoftene til midt mellom skuldrene.
    h0 = mid(j["upperleg01.L"][0], j["upperleg01.R"][0])
    s0 = mid(j["upperarm01.L"][0], j["upperarm01.R"][0])
    h1 = mid(J["hip.L"], J["hip.R"])
    s1 = mid(J["shoulder.L"], J["shoulder.R"])
    # Bredden: hoftene og skuldrene.
    kw = ((J["shoulder.L"] - J["shoulder.R"]).length / (j["upperarm01.L"][0] - j["upperarm01.R"][0]).length
          + (J["hip.L"] - J["hip.R"]).length / (j["upperleg01.L"][0] - j["upperleg01.R"][0]).length) / 2
    Mt, ct = segment_map(h0, s0, h1, s1)
    Mt = Mt @ Matrix(((1, 0, 0), (0, kw, 0), (0, 0, 1)))
    ct = h1 - Mt @ h0
    for b in ("root", "hips", "spine", "chest"):
        T[b] = (Mt, ct)
    neck0 = j["neck01"][0]
    T["neck"] = (Matrix.Identity(3), Mt @ neck0 + ct - neck0)
    T["head"] = T["neck"]
    pairs = {"upper_arm": ("upperarm01", "lowerarm01", "shoulder", "elbow"), "forearm": ("lowerarm01", "wrist", "elbow", "wrist"),
             "thigh": ("upperleg01", "lowerleg01", "hip", "knee"), "shin": ("lowerleg01", "foot", "knee", "ankle")}
    for s in ("L", "R"):
        for b, (a0, a1, b0, b1) in pairs.items():
            T[f"{b}.{s}"] = segment_map(j[f"{a0}.{s}"][0], j[f"{a1}.{s}"][0], J[f"{b0}.{s}"], J[f"{b1}.{s}"])
        # Hånden: dreid med underarmen og større.
        fa0, fa1 = j[f"lowerarm01.{s}"][0], j[f"wrist.{s}"][0]
        q = (fa1 - fa0).normalized().rotation_difference((J[f"wrist.{s}"] - J[f"elbow.{s}"]).normalized())
        Mh = q.to_matrix() * fig["hand"]
        w0 = j[f"wrist.{s}"][0]
        T[f"hand.{s}"] = (Mh, J[f"wrist.{s}"] - Mh @ w0)
        # Foten: flyttet med ankelen.
        a0 = j[f"foot.{s}"][0]
        T[f"foot.{s}"] = (Matrix.Identity(3), J[f"ankle.{s}"] - a0)
    out = np.zeros_like(P)
    for k, n in enumerate(BONES18):
        w = W[:, k]
        if not w.any():
            continue
        M, c = T[n]
        Mn = np.array([list(r) for r in M])
        out += w[:, None] * (P @ Mn.T + np.array(c[:]))
    set_coords(body, out)

    def apply(n, p):
        M, c = T[n]
        return M @ p + c


    for k in list(marks):
        marks[k] = apply("head", marks[k])
    jj = {}
    for name, (a, b) in j.items():
        bn = our_bone(name)
        jj[name] = (apply(bn, a), apply(bn, b))
    # Leddene mellom to bein: det som forbildet sier.
    for s in ("L", "R"):
        for mhn, jn in (("upperarm01", "shoulder"), ("lowerarm01", "elbow"), ("wrist", "wrist"), ("upperleg01", "hip"), ("lowerleg01", "knee"), ("foot", "ankle")):
            a, b = jj[f"{mhn}.{s}"]
            jj[f"{mhn}.{s}"] = (J[f"{jn}.{s}"], b)
    return jj


# --- Silhuetten -----------------------------------------------------------------


def silhouette(P, tris, view, shape, sel=None):
    """Masken av trekantene (indekser i P) sett fra visningen."""
    m = np.zeros(shape, np.uint8)
    q = view.pxs(P)
    t = tris if sel is None else tris[sel]
    polys = (q[t] * 4).round().astype(np.int32)
    for poly in polys:
        cv2.fillConvexPoly(m, poly, 1, lineType=cv2.LINE_8, shift=2)
    return m.astype(bool)


def runs(row):
    """(start, slutt) for sammenhengende biter i en rad."""
    d = np.diff(np.concatenate([[0], row.astype(np.int8), [0]]))
    return list(zip(np.where(d == 1)[0], np.where(d == -1)[0]))


def row_map(tr, rr, tol=25):
    """Kontrollpunkter (fra, til) for én rad: bitene i malen (`tr`) føres
    til bitene i forbildet (`rr`) de overlapper."""
    nt, nr = len(tr), len(rr)
    if not nt or not nr:
        return []
    # Grupper: sammenhengende komponenter av overlapp.
    adj = [[j for j, (c, d) in enumerate(rr) if a - tol < d and c < b + tol] for (a, b) in tr]
    seen_t, out = set(), []
    for i in range(nt):
        if i in seen_t or not adj[i]:
            continue
        ts, rs = {i}, set(adj[i])
        grow = True
        while grow:
            grow = False
            for k in range(nt):
                if k not in ts and set(adj[k]) & rs:
                    ts.add(k)
                    rs |= set(adj[k])
                    grow = True
        seen_t |= ts
        ts, rs = sorted(ts), sorted(rs)
        if len(ts) == len(rs):
            for a, b in zip(ts, rs):
                out += [(tr[a][0], rr[b][0]), (tr[a][1], rr[b][1])]
        else:
            out += [(tr[ts[0]][0], rr[rs[0]][0]), (tr[ts[-1]][1], rr[rs[-1]][1])]
    out.sort()
    clean = []
    for a, b in out:
        if not clean or (a > clean[-1][0] and b > clean[-1][1]):
            clean.append((a, b))
    return clean


def fit_rows(ob, view, axis, mask_ref, sel_tris, move, skip=(), sigma=5.0, strength=1.0):
    """Strekker hjørnene (langs `axis`, 0 x eller 1 y) rad for rad så
    silhuetten av trekantene `sel_tris` fra `view` følger `mask_ref`.
    `move` (n,) er hvor mye hvert hjørne får flytte seg (0–1). `skip` er
    radområder (rad0, rad1) der forbildet ikke gjelder; der brukes
    forskyvningen fra radene rundt."""
    P = coords(ob)
    tris = np.array([t.vertices[:] for t in ob.data.loop_triangles])
    H, Wd = mask_ref.shape
    mine = silhouette(P, tris, view, mask_ref.shape, sel_tris)
    rows = np.where(mine.any(1))[0]
    r0, r1 = rows[0], rows[-1] + 1
    cols = np.arange(Wd, dtype=np.float64)
    D = np.zeros((H, Wd))
    have = np.zeros(H, bool)
    for r in range(r0, r1):
        if any(a <= r < b for a, b in skip):
            continue
        cp = row_map(runs(mine[r]), runs(mask_ref[r]))
        if not cp:
            continue
        src = np.array([c[0] for c in cp], float)
        dst = np.array([c[1] for c in cp], float)
        d = np.interp(cols, src, dst - src)
        D[r] = d
        have[r] = True
    # Radene uten forbilde: lineært mellom naboene.
    idx = np.where(have)[0]
    if len(idx) == 0:
        return
    for r in range(r0, r1):
        if not have[r]:
            lo = idx[idx < r]
            hi = idx[idx > r]
            if len(lo) and len(hi):
                t = (r - lo[-1]) / (hi[0] - lo[-1])
                D[r] = D[lo[-1]] * (1 - t) + D[hi[0]] * t
            elif len(lo):
                D[r] = D[lo[-1]]
            elif len(hi):
                D[r] = D[hi[0]]
    D = ndi.gaussian_filter1d(D, sigma, axis=0)
    q = view.pxs(P)
    c = np.clip(q[:, 0], 0, Wd - 1.001)
    r = np.clip(q[:, 1], 0, H - 1.001)
    d = ndi.map_coordinates(D, [r, c], order=1)
    sign = view.right[axis]
    P[:, axis] += d / view.s * sign * move * strength
    set_coords(ob, P)


def overlay(views, objs, out):
    """Forbildet med silhuetten av objektene over (rød kant), side om side."""
    pics = []
    for v in views:
        img = (v.img * 255).astype(np.uint8)[:, :, ::-1].copy()
        m = np.zeros(v.mask.shape, bool)
        for ob in objs:
            P = coords(ob) @ np.array(ob.matrix_world.to_3x3()).T + np.array(ob.matrix_world.translation[:])
            tris = np.array([t.vertices[:] for t in ob.data.loop_triangles])
            m |= silhouette(P, tris, v, v.mask.shape)
        m = cv2.morphologyEx(m.astype(np.uint8), cv2.MORPH_CLOSE, np.ones((5, 5), np.uint8)).astype(bool)
        edge = m ^ cv2.erode(m.astype(np.uint8), np.ones((3, 3), np.uint8)).astype(bool)
        redge = v.mask ^ cv2.erode(v.mask.astype(np.uint8), np.ones((3, 3), np.uint8)).astype(bool)
        img[m] = (img[m] * 0.75 + np.array([255, 120, 0]) * 0.25).astype(np.uint8)
        img[redge] = (0, 255, 0)
        img[edge] = (0, 0, 255)
        cols = np.where(v.mask.any(0))[0]
        rows = np.where(v.mask.any(1))[0]
        pic = img[max(rows[0] - 20, 0):rows[-1] + 20, max(cols[0] - 20, 0):cols[-1] + 20]
        pics.append(pic)
    h = max(p.shape[0] for p in pics)
    cv2.imwrite(out, np.hstack([cv2.resize(p, (int(p.shape[1] * h / p.shape[0]), h)) for p in pics]))


def align_torso(body, sheet, J):
    """Flytter overkroppen (alt over beina) fram eller tilbake så midten av
    den følger midten i sidevisningen mellom hoftene og skuldrene."""
    P = coords(body)
    W = weight_table(body)
    tris = np.array([t.vertices[:] for t in body.data.loop_triangles])
    v = sheet.side
    torso = W_of(W, "hips", "spine", "chest")
    sel = torso[tris].min(1) > 0.5
    mine = silhouette(P, tris, v, v.mask.shape, sel)
    z0 = (J["hip.L"].z + J["shoulder.L"].z) / 2
    z1 = J["shoulder.L"].z - 0.08
    shifts = []
    for z in np.linspace(z0, z1, 12):
        r = int(v.o[1] - z * v.s)
        a, b = np.where(mine[r])[0][[0, -1]]
        rr = runs(v.mask[r])
        best = max(rr, key=lambda q: min(q[1], b) - max(q[0], a))
        shifts.append(((best[0] + best[1]) - (a + b)) / 2)
    dx = float(np.median(shifts)) / v.s
    legs = W_of(W, "thigh.L", "thigh.R", "shin.L", "shin.R", "foot.L", "foot.R")
    P[:, 0] += dx * (1 - legs)
    set_coords(body, P)
    print(f"overkroppen flyttet {dx * 100:.1f} cm fram")
    return dx


def fit_body(body, sheet, fig):
    """Kroppen etter silhuettene forfra og fra siden (ikke hodet og
    halsen, som formes etter hodearket)."""
    W = weight_table(body)
    move = 1.0 - np.clip(W_of(W, "head") + W_of(W, "neck") * 0.7, 0, 1)
    tris = np.array([t.vertices[:] for t in body.data.loop_triangles])
    sel = W_of(W, "head")[tris].max(1) < 0.5
    sk = fig.get("skip", {})
    fit_rows(body, sheet.front, 1, sheet.front.mask, sel, move, skip=sk.get("front", ()) + ((0, fig["neck_row"]),))
    fit_rows(body, sheet.side, 0, sheet.side.mask, sel, move, skip=sk.get("side", ()) + ((0, fig["neck_row"]),))


# --- Hodet -------------------------------------------------------------------------


def unview(v, col, row):
    """(sideveis, z) i meter for et punkt i en visning."""
    return (col - v.o[0]) / v.s, (v.o[1] - row) / v.s


def face_targets(hs, fig):
    """Landemerkene i ansiktet i figurens rom, fra hodearket: øynene
    (sentrum av øyeeplet), nesetippen, munnen og ørene."""
    f = fig["face"]
    fv, rv = hs.views["front"], hs.views["right"]
    out = {}
    ex, ez_side = unview(rv, *f["side_eye"])
    for s, (c, r) in zip(("R", "L"), f["eyes"]):
        y, z = unview(fv, c, r)
        out[f"eye.{s}"] = Vector((ex - 0.012, y, z))
    ny, nz = unview(fv, *f["nose"])
    # Nesetippen i profil: det fremste i silhuetten i nesehøyden.
    row = int(round(rv.o[1] - nz * rv.s))
    band = rv.mask[row - 3:row + 4]
    cols = np.where(band.any(0))[0]
    out["nose"] = Vector(((cols[-1] - rv.o[0]) / rv.s, ny, nz))
    my, mz = unview(fv, *f["mouth"])
    out["mouth"] = Vector((0.0, my, mz))
    ax, az = unview(rv, *f["ear"])
    out["ear"] = Vector((ax, 0.0, az))
    return out


def face_marks(body, marks):
    """De samme landemerkene på MakeHuman-hodet."""
    P = coords(body)
    W = weight_table(body)
    head = W_of(W, "head") > 0.5
    eye_z = (marks["eye.L"].z + marks["eye.R"].z) / 2
    mid = (marks["eye.L"].y + marks["eye.R"].y) / 2
    sel = head & (np.abs(P[:, 1] - mid) < 0.008) & (P[:, 2] < eye_z) & (P[:, 2] > marks["mouth"].z)
    nose = P[sel][np.argmax(P[sel, 0])]
    ymax = np.abs(P[head, 1] - mid).max()
    ear = head & (np.abs(P[:, 1] - mid) > ymax - 0.012) & (np.abs(P[:, 2] - eye_z + 0.015) < 0.03)
    out = dict(marks)
    out["nose"] = Vector(nose)
    out["ear"] = Vector(P[ear].mean(0))
    return out


def fit_axis(src, dst):
    """Skala og forskyvning (minste kvadrater) som fører src til dst."""
    A = np.stack([src, np.ones_like(src)], 1)
    (k, c), *_ = np.linalg.lstsq(A, dst, rcond=None)
    return k, c


def fit_head(body, marks, tgt):
    """Hodet (og halvt halsen) strekkes per akse så øynene, nesen, munnen
    og øret havner der de er i hodearket."""
    m = face_marks(body, marks)
    ky, cy = fit_axis(np.array([m["eye.R"].y, m["eye.L"].y]), np.array([tgt["eye.R"].y, tgt["eye.L"].y]))
    kz, cz = fit_axis(np.array([m["eye.R"].z, m["eye.L"].z, m["nose"].z, m["mouth"].z]), np.array([tgt["eye.R"].z, tgt["eye.L"].z, tgt["nose"].z, tgt["mouth"].z]))
    kx, cx = fit_axis(np.array([m["ear"].x, (m["eye.R"].x + m["eye.L"].x) / 2, m["nose"].x]), np.array([tgt["ear"].x, (tgt["eye.R"].x + tgt["eye.L"].x) / 2, tgt["nose"].x]))
    print(f"hodet: skala x {kx:.2f} y {ky:.2f} z {kz:.2f}")
    P = coords(body)
    W = weight_table(body)
    w = np.clip(W_of(W, "head") + 0.5 * W_of(W, "neck"), 0, 1)[:, None]
    Q = np.stack([P[:, 0] * kx + cx, P[:, 1] * ky + cy, P[:, 2] * kz + cz], 1)
    set_coords(body, P + w * (Q - P))
    f = lambda v: Vector((v.x * kx + cx, v.y * ky + cy, v.z * kz + cz))  # noqa: E731
    return {k: f(v) for k, v in marks.items()}


def clear():
    bpy.ops.wm.read_factory_settings(use_empty=True)


if __name__ == "__main__":
    args = sys.argv[1:]
    name = next(a for a in args if a in FIGURES)
    fig = FIGURES[name]
    os.makedirs(TMP, exist_ok=True)
    clear()
    sheet = Sheet(name, fig)
    J = sheet.joints(fig)
    body, j, marks = template(fig, J)
    j = retarget(body, j, J, fig, marks)
    align_torso(body, sheet, J)
    hs = HeadSheet(name, fig, sheet)
    marks = fit_head(body, marks, face_targets(hs, fig))
    if "--fit0" in args:
        overlay([sheet.front, sheet.side, sheet.back], [body], args[args.index("--fit0") + 1])
    fit_body(body, sheet, fig)
    if "--fit" in args:
        out = args[args.index("--fit") + 1]
        overlay([sheet.front, sheet.side, sheet.back], [body], out)
        overlay([hs.views[k] for k in ("front", "right", "left", "back")], [body], out.replace(".jpg", "_head.jpg"))
        sys.exit(0)
