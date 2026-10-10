"""Oppsettet for én rekonstruksjon: forbildet, visningene (rute og retning), arbeidsmappa og parametrene.

Settes av `setup()` (fra cli.py); modulene leser verdiene herfra.
"""
import json
import os

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
WEIGHTS = os.path.join(ROOT, "target/reconstruct")
ONNX_URL = "https://github.com/fabio-sim/Depth-Anything-ONNX/releases/download/v2.0.0/depth_anything_v2_vitb.onnx"
ONNX = os.path.join(WEIGHTS, "depth_anything_v2_vitb.onnx")

# Retningene en rute kan vise: hvor kameraet står (hodets rom: x mot figurens venstre, y opp, z fram)
# og hvilken akse bildets høyre peker langs.
FACING = {
    "front": dict(cam=(0, 0, 1), right=(1, 0, 0)),
    "face_right": dict(cam=(-1, 0, 0), right=(0, 0, 1)),  # profil, ansiktet mot høyre i bildet
    "face_left": dict(cam=(1, 0, 0), right=(0, 0, -1)),  # profil, ansiktet mot venstre i bildet
    "back": dict(cam=(0, 0, -1), right=(-1, 0, 0)),
}
LAYOUTS = {
    # rader, kolonner, rute per visning (rad, kolonne)
    "head2x2": dict(grid=(2, 2), views={"front": (0, 0), "face_right": (0, 1), "face_left": (1, 0), "back": (1, 1)}),
    "body3": dict(grid=(1, 3), views={"front": (0, 0), "face_right": (0, 1), "back": (0, 2)}),
}

REF = WORK = SHOTS = NAME = None
VIEWS = {}
TW = TH = T = 1024
SCALE = 0.00028  # meter per verdenspiksel
TEX = 2048
TRIS = 7000
OPT = {}  # avvik per ark (sheets.json): hals, kutt osv.


def p(*a):
    os.makedirs(WORK, exist_ok=True)
    return os.path.join(WORK, *a)


def setup(ref, layout="head2x2", name=None, views=None, scale=None, tris=None, opt=None, shots=None):
    global REF, WORK, SHOTS, NAME, VIEWS, TW, TH, T, SCALE, TRIS, OPT
    import cv2
    REF = os.path.abspath(ref)
    NAME = name or os.path.splitext(os.path.basename(ref))[0]
    WORK = os.path.join(ROOT, "target/reconstruct", NAME)
    SHOTS = shots or os.path.join(ROOT, "shots/reconstruct", NAME)
    lay = LAYOUTS[layout]
    vmap = dict(views or lay["views"])  # facing -> (rad, kolonne)
    h, w = cv2.imread(REF).shape[:2]
    TH, TW = h // lay["grid"][0], w // lay["grid"][1]
    T = TH
    VIEWS = {}
    names = {"front": "front", "face_right": "right", "face_left": "left", "back": "back"}
    for facing, rc in vmap.items():
        VIEWS[names[facing]] = dict(rc=tuple(rc), **FACING[facing])
    SCALE = scale or SCALE
    TRIS = tris or TRIS
    OPT = opt or {}


def from_config(name, **kw):
    cfg = json.load(open(os.path.join(os.path.dirname(__file__), "sheets.json")))[name]
    setup(os.path.join(ROOT, cfg["ref"]), cfg.get("layout", "head2x2"), name=name, views=cfg.get("views"),
          scale=cfg.get("scale"), opt=cfg.get("opt"), **kw)
