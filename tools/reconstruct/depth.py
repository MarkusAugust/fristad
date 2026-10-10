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
    for k in J.VIEWS:
        rgb = cv2.imread(J.p(f"{k}_rgb.png"))[:, :, ::-1]
        outs = []
        for flip in (False, True):  # snitt med speilet bilde mot skjevhet
            im = rgb[:, ::-1] if flip else rgb
            x = cv2.resize(im, (518, 518), interpolation=cv2.INTER_CUBIC).astype(np.float32) / 255
            x = ((x - mean) / std).transpose(2, 0, 1)[None]
            o = sess.run(None, {name: x})[0][0]
            o = cv2.resize(o, (J.TW, J.TH), interpolation=cv2.INTER_CUBIC)
            outs.append(o[:, ::-1] if flip else o)
        d = (outs[0] + outs[1]) / 2
        np.save(J.p(f"{k}_disp.npy"), d.astype(np.float32))
        m = cv2.imread(J.p(f"{k}_mask.png"), 0) > 127
        v = d[m]
        vis = np.clip((d - v.min()) / (v.max() - v.min()), 0, 1) * m
        cv2.imwrite(J.p(f"{k}_disp.png"), (vis * 255).astype(np.uint8))
        print(k, "disp", float(v.min()), float(v.max()))


