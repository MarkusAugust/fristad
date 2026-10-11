"""Regresjonstest: smeden, Vertinnen og Pilegrimen (hodene) bygges fra bufrede dybdekart, uten nettverk, og
tallene i rapporten sjekkes mot tersklene. Dybden leses fra `target/sheet2model/cache/depth/`; mangler den,
hoppes testen over (ellers ville den lastet ned 390 MB og regnet dybde i et minutt per ark).

    python3.13 -m pytest sheet2model/tests            # alle tre (ca. 2–3 minutter per figur)
    SHEET2MODEL_TEST_ONLY=smith_head python3.13 -m pytest sheet2model/tests

Kjøres med flock på render-låsen i skyen (bpy): `flock /tmp/fristad-render.lock python3.13 -m pytest ...`.
"""
import json
import os
import sys

import pytest

HERE = os.path.dirname(os.path.abspath(__file__))
PKG = os.path.dirname(HERE)
ROOT = os.path.dirname(PKG)
sys.path.insert(0, ROOT)

FIGURES = ["smith_head", "innkeeper_head", "pilgrim_head"]
# forventet minst dette (målt 11. okt 2026 med denne pakken; tersklene i rapporten er strengere på IoU)
EXPECT = {
    "smith_head": dict(iou_min=0.85, photo_max=0.09, sharpness_min=60, face_texels_cm_min=25),
    # Vertinnens og Pilegrimens hodeark stemmer dårligere med seg selv (profilene tegnet i annen høyde/dreining):
    # tersklene her er nivået de holder i dag, så en endring som gjør dem verre, oppdages
    "innkeeper_head": dict(iou_min=0.75, photo_max=0.13, sharpness_min=50, face_texels_cm_min=10),
    "pilgrim_head": dict(iou_min=0.80, photo_max=0.12, sharpness_min=60, face_texels_cm_min=20),
}


def _job(name):
    from sheet2model import config
    return config.load_sheet(name, sheets_dir=os.path.join(PKG, "examples/fristad"), root=ROOT)


def _cached(job):
    from sheet2model import depth, sheet
    sheet.seg(job)
    return all(os.path.exists(depth.cache_path(job, k)) for k in job.real_views())


@pytest.mark.parametrize("name", [n for n in FIGURES if n in os.environ.get("SHEET2MODEL_TEST_ONLY", ",".join(FIGURES)).split(",")])
def test_figure(name):
    pytest.importorskip("bpy")
    job = _job(name)
    job.work = os.path.join(ROOT, "target/sheet2model/test", name)
    job.shots = os.path.join(ROOT, "target/sheet2model/test", name, "shots")
    if not _cached(job):
        pytest.skip("ingen bufret dybde for " + name)
    from sheet2model import cli, report, targets
    job.limits.update(EXPECT[name])
    try:
        r = cli.build(job, steps=["depth", "fuse", "mesh", "texture", "report"], target=targets.load("fristad", root=ROOT), fix=True)
    except SystemExit as e:  # rapporten under terskel også etter rettelsene: vis tallene
        r = report.compute(job)
        pytest.fail("%s: %s" % (name, e))
    assert r is not None
    e = EXPECT[name]
    assert r["worst_iou"] >= e["iou_min"], r["iou"]
    assert r["worst_photo"] <= e["photo_max"], r["photometric"]
    assert r["components"] == 1
    assert r["holes"] <= 1  # halskuttet
    assert r["sharpness"] >= e["sharpness_min"]
    assert r["face_texels_cm"] >= e["face_texels_cm_min"]
    assert r["seconds_total"] < 300
    assert os.path.exists(job.p("rapport.json"))
    json.load(open(job.p("rapport.json")))


def test_report_judge():
    from sheet2model import report
    r = dict(limits=dict(iou_min=0.8, photo_max=0.1, components_max=1, holes_max=1, face_texels_cm_min=3, sharpness_min=0),
             worst_iou=0.7, worst_iou_view="right", worst_photo=0.2, worst_photo_pair="front-right", components=3, holes=4,
             face_texels_cm=1.0, layout="head2x2", tool=dict(found=False, side="R"))
    p = report.judge(r)
    assert len(p) == 6
    assert any(f for f in report.FIXES if f[1](r))


def test_geometry_roundtrip():
    import numpy as np
    from sheet2model import geom
    from sheet2model.config import View
    v = View("right", "face_right")
    o = dict(c=100.0, top=50.0, sy=1.0, yaw=10.0, roll=2.0)
    cam, right, up = geom.basis(v, o)
    assert abs(cam @ right) < 1e-9 and abs(cam @ up) < 1e-9 and abs(right @ up) < 1e-9
    u, vv = geom.to_img(v, o, np.array([0.0]), np.array([0.0]), np.array([0.0]))
    assert u[0] == 100.0 and vv[0] == 50.0
    m = geom.mirror_frame(o, 1024)
    assert m["yaw"] == -10.0 and m["c"] == 1023 - 100.0
