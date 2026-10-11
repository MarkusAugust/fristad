"""Steg 2: relativ dybde per visning med Depth Anything V2 (ONNX på CPU), bufret per bilde.

Bufferen ligger i `<root>/target/sheet2model/cache/depth/<sha>.npy`, der sha er av selve ruta (rgb-png) og
modellnavnet. Bygges figuren om med samme ark, koster dybden ingenting, og testene kjører uten nettverk og uten
onnxruntime når bufferen er fylt.
"""
import os
import shutil
import time

import numpy as np

from .config import file_hash

ONNX_URL = "https://github.com/fabio-sim/Depth-Anything-ONNX/releases/download/v2.0.0/depth_anything_v2_vitb.onnx"
MODEL = "depth_anything_v2_vitb"


def weights_path(job):
    return os.path.join(job.cache, MODEL + ".onnx")


def ensure_weights(job):
    """Vektene (ca. 390 MB) lastes ned én gang fra GitHub-releases."""
    p = weights_path(job)
    if not os.path.exists(p):
        alt = os.environ.get("SHEET2MODEL_ONNX")  # alt nedlastet et annet sted
        if alt and os.path.exists(alt):
            os.makedirs(os.path.dirname(p), exist_ok=True)
            os.symlink(os.path.abspath(alt), p)
            return p
        import urllib.request
        os.makedirs(os.path.dirname(p), exist_ok=True)
        print("laster ned", ONNX_URL)
        urllib.request.urlretrieve(ONNX_URL, p + ".part")
        os.replace(p + ".part", p)
    return p


def cache_key(job, k):
    return file_hash(job.p(f"{k}_rgb.png"), MODEL.encode())


def cache_path(job, k):
    return os.path.join(job.cache, "depth", cache_key(job, k) + ".npy")


class Net:
    def __init__(self, job):
        import onnxruntime as ort
        self.sess = ort.InferenceSession(ensure_weights(job), providers=["CPUExecutionProvider"])
        self.name = self.sess.get_inputs()[0].name
        self.mean = np.array([0.485, 0.456, 0.406], np.float32)
        self.std = np.array([0.229, 0.224, 0.225], np.float32)

    def run(self, rgb):
        import cv2
        outs = []
        for flip in (False, True):  # snitt med speilet bilde mot skjevhet
            im = rgb[:, ::-1] if flip else rgb
            x = cv2.resize(im, (518, 518), interpolation=cv2.INTER_CUBIC).astype(np.float32) / 255
            x = ((x - self.mean) / self.std).transpose(2, 0, 1)[None]
            o = self.sess.run(None, {self.name: x})[0][0]
            o = cv2.resize(o, (im.shape[1], im.shape[0]), interpolation=cv2.INTER_CUBIC)
            outs.append(o[:, ::-1] if flip else o)
        return (outs[0] + outs[1]) / 2

    def tile(self, rgb, m):
        """Hele ruta: kvadratisk direkte, høye ruter (hele figuren) i kvadratiske vinduer med overlapp, der hvert
        vindu får en affin tilpasning til det som alt er satt sammen (i overlappet innenfor masken)."""
        H, W = rgb.shape[:2]
        if H <= W * 1.2:
            return self.run(rgb)
        ys = np.linspace(0, H - W, int(np.ceil((H - W) / (W * 0.55))) + 1).astype(int)
        acc = np.zeros((H, W), np.float64); wacc = np.zeros((H, W), np.float64)
        for y0 in ys:
            o = self.run(rgb[y0:y0 + W])
            ramp = np.minimum(np.arange(W) + 1, W - np.arange(W))[:, None].astype(np.float64)
            ramp = np.minimum(ramp, W * 0.25) * np.ones((1, W))
            if wacc.any():
                cur = acc[y0:y0 + W] / np.maximum(wacc[y0:y0 + W], 1e-9)
                ov = (wacc[y0:y0 + W] > 0) & m[y0:y0 + W]
                A = np.stack([o[ov], np.ones(ov.sum())], 1)
                a, b = np.linalg.lstsq(A, cur[ov], rcond=None)[0]
                o = a * o + b
            acc[y0:y0 + W] += o * ramp; wacc[y0:y0 + W] += ramp
        return (acc / np.maximum(wacc, 1e-9)).astype(np.float32)


def depth(job):
    import cv2
    from .sheet import load_mask, load_rgb
    t0 = time.time()
    net = None
    hits = 0
    for k in job.real_views():
        cp = cache_path(job, k)
        if os.path.exists(cp):
            d = np.load(cp)
            hits += 1
        else:
            net = net or Net(job)
            d = net.tile(load_rgb(job, k), load_mask(job, k)).astype(np.float32)
            os.makedirs(os.path.dirname(cp), exist_ok=True)
            np.save(cp, d)
        np.save(job.p(f"{k}_disp.npy"), d)
        m = load_mask(job, k)
        v = d[m]
        vis = np.clip((d - v.min()) / (v.max() - v.min() + 1e-9), 0, 1) * m
        cv2.imwrite(job.p(f"{k}_disp.png"), (vis * 255).astype(np.uint8))
    for k, v in job.views.items():
        if v.mirror_of:
            d = np.load(job.p(f"{v.mirror_of}_disp.npy")).copy()
            tp = job.p("tool_src.npy")
            if job.opt.get("tool_box") and os.path.exists(tp):
                # redskapet tas ut av dybden også: hver rad interpoleres over redskapet fra kroppen rundt
                t = np.load(tp)
                for y in np.nonzero(t.any(1))[0]:
                    xs = np.arange(d.shape[1]); ok = ~t[y]
                    d[y, t[y]] = np.interp(xs[t[y]], xs[ok], d[y, ok])
            np.save(job.p(f"{k}_disp.npy"), d[:, ::-1].copy())
    print("dybde: %d av %d fra bufferen" % (hits, len(job.real_views())))
    job.time("depth", time.time() - t0)
