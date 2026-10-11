"""Dybdekart per visning med Depth Anything V2 (ONNX på CPU, vektene lastes ned fra GitHub-releases)."""
import json
import os

import numpy as np

from . import job as J


def ensure_weights():
    """Vektene (ca. 390 MB) lastes ned én gang fra GitHub-releases til target/reconstruct/ (git-ignorert)."""
    if not os.path.exists(J.ONNX):
        import urllib.request
        os.makedirs(os.path.dirname(J.ONNX), exist_ok=True)
        print("laster ned", J.ONNX_URL)
        urllib.request.urlretrieve(J.ONNX_URL, J.ONNX + ".part")
        os.replace(J.ONNX + ".part", J.ONNX)
    return J.ONNX


def depth():
    import cv2
    import onnxruntime as ort
    sess = ort.InferenceSession(ensure_weights(), providers=["CPUExecutionProvider"])
    name = sess.get_inputs()[0].name
    mean = np.array([0.485, 0.456, 0.406], np.float32)
    std = np.array([0.229, 0.224, 0.225], np.float32)
    def run(rgb):
        outs = []
        for flip in (False, True):  # snitt med speilet bilde mot skjevhet
            im = rgb[:, ::-1] if flip else rgb
            x = cv2.resize(im, (518, 518), interpolation=cv2.INTER_CUBIC).astype(np.float32) / 255
            x = ((x - mean) / std).transpose(2, 0, 1)[None]
            o = sess.run(None, {name: x})[0][0]
            o = cv2.resize(o, (im.shape[1], im.shape[0]), interpolation=cv2.INTER_CUBIC)
            outs.append(o[:, ::-1] if flip else o)
        return (outs[0] + outs[1]) / 2

    for k in J.VIEWS:
        if J.VIEWS[k].get("mirror_of"):
            continue
        rgb = cv2.imread(J.p(f"{k}_rgb.png"))[:, :, ::-1]
        m = cv2.imread(J.p(f"{k}_mask.png"), 0) > 127
        H, W = rgb.shape[:2]
        if H <= W * 1.2:
            d = run(rgb)
        else:
            # høye ruter (hele figuren): kvadratiske vinduer med overlapp; hvert vindu får en affin
            # tilpasning (a, b) til det som alt er satt sammen, i overlappet innenfor masken
            ys = np.linspace(0, H - W, int(np.ceil((H - W) / (W * 0.55))) + 1).astype(int)
            acc = np.zeros((H, W), np.float64); wacc = np.zeros((H, W), np.float64)
            for y0 in ys:
                o = run(rgb[y0:y0 + W])
                ramp = np.minimum(np.arange(W) + 1, W - np.arange(W))[:, None].astype(np.float64)
                ramp = np.minimum(ramp, W * 0.25) * np.ones((1, W))
                if wacc.any():
                    cur = acc[y0:y0 + W] / np.maximum(wacc[y0:y0 + W], 1e-9)
                    ov = (wacc[y0:y0 + W] > 0) & m[y0:y0 + W]
                    A = np.stack([o[ov], np.ones(ov.sum())], 1)
                    a, b = np.linalg.lstsq(A, cur[ov], rcond=None)[0]
                    o = a * o + b
                acc[y0:y0 + W] += o * ramp; wacc[y0:y0 + W] += ramp
            d = (acc / np.maximum(wacc, 1e-9)).astype(np.float32)
        np.save(J.p(f"{k}_disp.npy"), d.astype(np.float32))
        v = d[m]
        vis = np.clip((d - v.min()) / (v.max() - v.min()), 0, 1) * m
        cv2.imwrite(J.p(f"{k}_disp.png"), (vis * 255).astype(np.uint8))
        print(k, "disp", float(v.min()), float(v.max()))
    for k, v in J.VIEWS.items():
        if v.get("mirror_of"):
            np.save(J.p(f"{k}_disp.npy"), np.load(J.p(f"{v['mirror_of']}_disp.npy"))[:, ::-1].copy())
