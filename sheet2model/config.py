"""Oppsettet for én bygging: arket, visningene (rute og retning), arbeidsmappa og parametrene.

Alt står i et `Job`-objekt som sendes til modulene (ingen globale verdier), så flere jobber kan leve i samme prosess
og testene kan kjøre uten å rydde opp etter hverandre.

Rommet («verden»): x mot figurens venstre, y opp, z fram, i piksler fra forfra-ruta. `scale` er meter per piksel.
"""
import hashlib
import json
import os

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))

# Retningene en rute kan vise: hvor kameraet står, hvilken akse bildets høyre peker langs og hvilken bildets opp.
FACING = {
    "front": dict(cam=(0, 0, 1), right=(1, 0, 0), up=(0, 1, 0)),
    "face_right": dict(cam=(-1, 0, 0), right=(0, 0, 1), up=(0, 1, 0)),  # profil, ansiktet mot høyre i bildet
    "face_left": dict(cam=(1, 0, 0), right=(0, 0, -1), up=(0, 1, 0)),  # profil, ansiktet mot venstre i bildet
    "back": dict(cam=(0, 0, -1), right=(-1, 0, 0), up=(0, 1, 0)),
    "top": dict(cam=(0, 1, 0), right=(1, 0, 0), up=(0, 0, -1)),  # ovenfra, med bakhodet opp i bildet
    # 3/4-visninger (gjenstander): kameraet 45° fra forsiden; vippen (pitch) settes i rammen
    "q_front_left": dict(cam=(0.70710678, 0, 0.70710678), right=(0.70710678, 0, -0.70710678), up=(0, 1, 0)),
    "q_front_right": dict(cam=(-0.70710678, 0, 0.70710678), right=(0.70710678, 0, 0.70710678), up=(0, 1, 0)),
    "q_back_right": dict(cam=(-0.70710678, 0, -0.70710678), right=(-0.70710678, 0, 0.70710678), up=(0, 1, 0)),
    "q_back_left": dict(cam=(0.70710678, 0, -0.70710678), right=(-0.70710678, 0, -0.70710678), up=(0, 1, 0)),
}
NAMES = {"front": "front", "face_right": "right", "face_left": "left", "back": "back", "top": "top",
         "q_front_left": "qfl", "q_front_right": "qfr", "q_back_right": "qbr", "q_back_left": "qbl"}

LAYOUTS = {
    # rader, kolonner, rute per visning (rad, kolonne)
    "head2x2": dict(grid=(2, 2), views={"front": (0, 0), "face_right": (0, 1), "face_left": (1, 0), "back": (1, 1)}),
    # kroppsarkene står ikke i like store tredjedeler: rutene finnes fra mellomrommene mellom figurene
    "body3": dict(grid=(1, 3), split="gaps", views={"front": (0, 0), "face_right": (0, 1), "back": (0, 2)},
                  mirror={"face_left": "face_right"}),  # bare én profil: den andre siden er speilet
    "single": dict(grid=(1, 1), views={"front": (0, 0)}),
    # gjenstander: én 3/4-visning (ruta oppgis med `views` og `grid` i oppslaget), den andre siden speilet
    "prop1": dict(grid=(1, 1), views={"q_front_left": (0, 0)}, mirror={"q_front_right": "q_front_left"}, prop=True),
    # to visninger: 3/4 forfra venstre og 3/4 bakfra høyre, begge speilet for de to andre kvadrantene
    "prop2": dict(grid=(1, 2), views={"q_front_left": (0, 0), "q_back_right": (0, 1)},
                  mirror={"q_front_right": "q_front_left", "q_back_left": "q_back_right"}, prop=True),
}

DEFAULT_OPT = dict(
    edge_slack=28.0, edge_gain=1.5, edge_fade=10.0, dome=0.14, vox=3.0, flake_frac=0.02,
    align=dict(shift=True, shift_v=False, scale=False, yaw=8.0, roll=3.0, rounds=2),
    poisson=0.5, delight=0.0, min_hole=400, render="cycles",
)
DEFAULT_LIMITS = dict(iou_min=0.80, photo_max=0.11, components_max=1, holes_max=1, face_texels_cm_min=3.0,
                      sharpness_min=60.0)


class View:
    def __init__(self, name, facing, rc=None, file=None, mirror_of=None, geo_only=False):
        self.name, self.facing, self.rc, self.file = name, facing, rc, file
        self.mirror_of, self.geo_only = mirror_of, geo_only
        f = FACING[facing]
        self.cam = np.array(f["cam"], float)
        self.right = np.array(f["right"], float)
        self.up = np.array(f["up"], float)

    def __repr__(self):
        return f"View({self.name}, {self.facing}, rc={self.rc}, mirror_of={self.mirror_of})"


class Job:
    def __init__(self, ref, layout="head2x2", name=None, views=None, scale=None, tris=None, opt=None, root=None,
                 work=None, shots=None, target=None, head=None, limits=None, tex=2048, extra=None, grid=None):
        self.ref = os.path.abspath(ref)
        self.name = name or os.path.splitext(os.path.basename(ref))[0]
        self.root = os.path.abspath(root or os.getcwd())
        self.work = work or os.path.join(self.root, "target/sheet2model", self.name)
        self.shots = shots or os.path.join(self.root, "shots/sheet2model", self.name)
        self.cache = os.path.join(self.root, "target/sheet2model/cache")
        self.layout = layout
        self.scale = scale or (0.00028 if layout.startswith("head") else 0.0015 if layout.startswith("prop") else 0.0009)
        self.tris = tris or (7000 if layout.startswith("head") else 3000 if layout.startswith("prop") else 14000)
        self.tex = tex
        self.target = target
        self.head = head  # navnet på hodearket som settes på kroppen
        self.opt = json.loads(json.dumps(DEFAULT_OPT))
        for k, v in (opt or {}).items():
            if isinstance(v, dict) and isinstance(self.opt.get(k), dict):
                self.opt[k].update(v)
            else:
                self.opt[k] = v
        self.limits = dict(DEFAULT_LIMITS, **(limits or {}))
        if layout.startswith("body"):  # hodets halsåpning inne i kragen og kroppens krage: to åpninger som ikke ses
            self.limits.setdefault("holes_max_body", 2)
            if not (limits or {}).get("holes_max"):
                self.limits["holes_max"] = 2
        self.views = {}
        self.crops = None
        self.TW = self.TH = None
        self.grid = grid
        self.prop = bool(LAYOUTS[layout].get("prop"))
        if self.prop:
            self.opt.setdefault("mirror_texture", True)
            self.opt.setdefault("dome", 0)
            self.opt.setdefault("edge_slack", 10.0)
            self.opt.setdefault("prior", "box")
            self.opt.setdefault("view_pitch", 20.0)
            self.limits.setdefault("face_texels_cm_min", 0.0)
            self.limits["face_texels_cm_min"] = 0.0
            self.limits["holes_max"] = max(self.limits.get("holes_max", 1), 1)
        self._layout_views(views, extra or {})
        self.timings = {}

    # ------------------------------------------------------------------ oppsett
    def _layout_views(self, views, extra):
        import cv2
        lay = LAYOUTS[self.layout]
        vmap = dict(views or lay["views"])  # facing -> (rad, kolonne)
        im = cv2.imread(self.ref)
        if im is None:
            raise FileNotFoundError(self.ref)
        h, w = im.shape[:2]
        g = tuple(self.grid) if self.grid else lay["grid"]
        self.TH, self.TW = h // g[0], w // g[1]
        if lay.get("split") == "gaps":
            self.crops, self.TW = gap_split(im, lay["grid"][1])
        for facing, rc in vmap.items():
            self.views[NAMES[facing]] = View(NAMES[facing], facing, rc=tuple(rc))
        for facing, path in extra.items():  # egne bilder utenom arket (ruta ovenfra)
            p = path if os.path.isabs(path) else os.path.join(self.root, path)
            self.views[NAMES[facing]] = View(NAMES[facing], facing, file=p)
        mirror = dict(lay.get("mirror", {}))
        if self.prop:
            for facing, src in mirror.items():  # speilbildet brukes også til teksturen (gjenstander er symmetriske)
                if facing not in vmap and src in vmap:
                    self.views[NAMES[facing]] = View(NAMES[facing], facing, rc=tuple(vmap[src]), mirror_of=NAMES[src],
                                                     geo_only=True)
            return
        if ("face_left" in vmap) != ("face_right" in vmap):
            # bare én profil: den andre siden er den speilet, både for formen og teksturen (ellers blir siden
            # malt med visningen forfra på skrå, og det ser ut som et ekstra ansikt)
            self.opt.setdefault("mirror_texture", True)
            if "face_left" not in vmap:
                mirror.setdefault("face_left", "face_right")
            else:
                mirror.setdefault("face_right", "face_left")
        if self.opt.get("one_profile") and "left" in self.views and "right" in self.views:
            # rettelse: den ene profilen stemmer ikke med resten (dobbelt ansikt); bruk bare den andre, speilet
            del self.views["left"]
            mirror["face_left"] = "face_right"
            self.opt["mirror_texture"] = True
        for facing, src in mirror.items():
            if NAMES[facing] not in self.views:
                self.views[NAMES[facing]] = View(NAMES[facing], facing, rc=tuple(vmap[src]), mirror_of=NAMES[src],
                                                 geo_only=True)

    def p(self, *a):
        os.makedirs(self.work, exist_ok=True)
        return os.path.join(self.work, *a)

    def shot(self, *a):
        os.makedirs(self.shots, exist_ok=True)
        return os.path.join(self.shots, *a)

    @property
    def tile_shape(self):
        return (self.TH, self.TW)

    def real_views(self):
        """Visningene med eget bilde (ikke speilede)."""
        return {k: v for k, v in self.views.items() if not v.mirror_of}

    def texture_views(self):
        return {k: v for k, v in self.views.items() if not v.geo_only or self.opt.get("mirror_texture")}

    def time(self, step, seconds):
        self.timings[step] = round(float(seconds), 2)
        try:
            json.dump(self.timings, open(self.p("tid.json"), "w"), indent=1)
        except OSError:
            pass


def gap_split(im, n):
    """Del arket i n figurer ved mellomrommene (kolonner uten forgrunn); bredden blir den største + marg."""
    f = im.astype(np.float32)
    bg = np.median(np.concatenate([f[:8].reshape(-1, 3), f[:, :8].reshape(-1, 3), f[:, -8:].reshape(-1, 3)]), 0)
    fg = (np.linalg.norm(f - bg, axis=2) > 20) | ((f.max(2) - f.min(2)) > 20)
    col = fg[: int(im.shape[0] * 0.86)].sum(0) > 3  # uten gulvlinja nederst (den står ulikt høyt i arkene)
    runs, x = [], 0
    while x < len(col):
        if col[x]:
            x0 = x
            while x < len(col) and col[x]:
                x += 1
            runs.append((x0, x))
        x += 1
    while len(runs) > n:  # slå sammen biter med små mellomrom til n figurer
        gaps = [runs[i + 1][0] - runs[i][1] for i in range(len(runs) - 1)]
        i = int(np.argmin(gaps))
        runs[i:i + 2] = [(runs[i][0], runs[i + 1][1])]
    if len(runs) == n:
        cuts = [0] + [(runs[i][1] + runs[i + 1][0]) // 2 for i in range(n - 1)] + [im.shape[1]]
    else:  # figurene (eller et redskap) overlapper i kolonnene: kutt der det er minst forgrunn nær tredjedelene
        cnt = fg[: int(im.shape[0] * 0.86)].sum(0).astype(float)
        W = im.shape[1]
        cuts = [0]
        for i in range(1, n):
            a, b = int(W * (i / n - 0.12)), int(W * (i / n + 0.12))
            cuts.append(a + int(np.argmin(np.convolve(cnt[a:b], np.ones(9) / 9, "same"))))
        cuts.append(W)
    crops = [(cuts[i], cuts[i + 1]) for i in range(n)]
    tw = max(b - a for a, b in crops) + 40
    tw += tw % 2
    return crops, tw


# ---------------------------------------------------------------- sheets/*.json
def load_sheet(path_or_name, sheets_dir=None, root=None, **kw):
    """Les ett oppslag: `sheets/<navn>.json` (eller en sti til json-fila). Stier i fila er relative til `root`
    (standard: mappa over `sheets/`)."""
    if os.path.isfile(path_or_name) and path_or_name.endswith(".json"):
        path = path_or_name
    else:
        sheets_dir = sheets_dir or os.environ.get("SHEET2MODEL_SHEETS") or os.path.join(HERE, "examples/fristad")
        path = os.path.join(sheets_dir, path_or_name + ".json")
    cfg = json.load(open(path))
    root = root or cfg.get("root") or os.environ.get("SHEET2MODEL_ROOT") or os.path.dirname(os.path.dirname(os.path.abspath(path)))
    if "root" in cfg and not os.path.isabs(cfg["root"]):
        root = os.path.abspath(os.path.join(os.path.dirname(path), cfg["root"]))
    name = cfg.get("name") or os.path.splitext(os.path.basename(path))[0]
    ref = cfg["ref"] if os.path.isabs(cfg["ref"]) else os.path.join(root, cfg["ref"])
    args = dict(layout=cfg.get("layout", "head2x2"), name=name, views=cfg.get("views"), scale=cfg.get("scale"),
                tris=cfg.get("tris"), opt=cfg.get("opt"), root=root, target=cfg.get("target"), head=cfg.get("head"),
                limits=cfg.get("limits"), extra=cfg.get("extra"), grid=cfg.get("grid"))
    args.update({k: v for k, v in kw.items() if v is not None})
    return Job(ref, **args)


def file_hash(path, extra=b""):
    h = hashlib.sha1()
    h.update(open(path, "rb").read())
    h.update(extra)
    return h.hexdigest()[:16]
