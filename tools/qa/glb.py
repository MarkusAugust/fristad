"""Leser glb-filene uten Blender: trekantene i verdensrommet (nodenes
transform tatt med), materialet per trekant med rollen fra `extras.fristad`,
og fargen hvert materiale har (teksturlagets snittfarge, den bakte
teksturens snittfarge eller fargefaktoren)."""
import io
import json
import os
import struct

import numpy as np
from PIL import Image

from .gate import ROOT

COUNT = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}
DTYPE = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}
NORM = {5120: 127.0, 5121: 255.0, 5122: 32767.0, 5123: 65535.0}


def read(path):
    b = open(path, "rb").read()
    jl = struct.unpack("<I", b[12:16])[0]
    j = json.loads(b[20 : 20 + jl])
    off = 20 + jl
    bin_ = b[off + 8 : off + 8 + struct.unpack("<I", b[off : off + 4])[0]] if off < len(b) else b""
    return j, bin_


def accessor(j, bin_, i):
    a = j["accessors"][i]
    v = j["bufferViews"][a["bufferView"]]
    n = COUNT[a["type"]]
    dt = np.dtype(DTYPE[a["componentType"]])
    start = v.get("byteOffset", 0) + a.get("byteOffset", 0)
    stride = v.get("byteStride", n * dt.itemsize)
    raw = np.frombuffer(bin_, dtype=np.uint8, count=stride * (a["count"] - 1) + n * dt.itemsize, offset=start)
    out = np.lib.stride_tricks.as_strided(raw, shape=(a["count"], n * dt.itemsize), strides=(stride, 1)).copy().view(dt).reshape(a["count"], n)
    out = out.astype(np.float64)
    if a.get("normalized"):
        out /= NORM[a["componentType"]]
    return out


def node_matrix(n):
    if "matrix" in n:
        return np.array(n["matrix"], dtype=np.float64).reshape(4, 4).T
    t = np.array(n.get("translation", [0, 0, 0]), dtype=np.float64)
    x, y, z, w = n.get("rotation", [0, 0, 0, 1])
    r = np.array([
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ])
    s = np.array(n.get("scale", [1, 1, 1]), dtype=np.float64)
    m = np.eye(4)
    m[:3, :3] = r * s
    m[:3, 3] = t
    return m


_layer_colors = {}


def layer_color(name):
    """Snittfargen (lineær 0..1 som sRGB) til teksturlaget."""
    if name not in _layer_colors:
        p = os.path.join(ROOT, "assets", "textures", name, "albedo.jpg")
        c = np.asarray(Image.open(p).convert("RGB").resize((16, 16)), dtype=np.float64).reshape(-1, 3).mean(0) / 255.0 if os.path.exists(p) else np.array([0.6, 0.6, 0.6])
        _layer_colors[name] = c
    return _layer_colors[name]


def material_colors(j, bin_):
    """Snittfargen per materiale, og den bakte teksturen (eller None)."""
    out, images = [], []
    for m in j.get("materials", []):
        role = m.get("extras", {}).get("fristad", {})
        pbr = m.get("pbrMetallicRoughness", {})
        f = np.array(pbr.get("baseColorFactor", [1, 1, 1, 1])[:3])
        if role.get("layer"):
            c = layer_color(role["layer"]) * np.clip(f ** (1 / 2.2), 0, 1)
        elif "baseColorTexture" in pbr:
            img = j["images"][j["textures"][pbr["baseColorTexture"]["index"]]["source"]]
            v = j["bufferViews"][img["bufferView"]]
            data = bin_[v.get("byteOffset", 0) : v.get("byteOffset", 0) + v["byteLength"]]
            tex = np.asarray(Image.open(io.BytesIO(data)).convert("RGB").resize((256, 256)), dtype=np.float64) / 255.0
            images.append(tex)
            out.append(tex.reshape(-1, 3).mean(0))
            continue
        else:
            c = np.clip(f ** (1 / 2.2), 0, 1)
        out.append(c)
        images.append(None)
    return out, images


class Mesh:
    """Trekantene (n × 3 × 3, meter), materialet per trekant, fargene per
    materiale og rollene."""

    def __init__(self, path):
        j, bin_ = read(path)
        self.path = path
        self.name = os.path.splitext(os.path.basename(path))[0]
        self.roles = [m.get("extras", {}).get("fristad") for m in j.get("materials", [])]
        self.material_names = [m.get("name", "") for m in j.get("materials", [])]
        self.colors, self.images = material_colors(j, bin_)
        tris, mats, uvs = [], [], []
        scene = j.get("scenes", [{}])[j.get("scene", 0)]

        def walk(i, parent):
            n = j["nodes"][i]
            m = parent @ node_matrix(n)
            if "mesh" in n and "skin" not in n:
                for prim in j["meshes"][n["mesh"]]["primitives"]:
                    if prim.get("mode", 4) != 4:
                        continue
                    p = accessor(j, bin_, prim["attributes"]["POSITION"])
                    p = (np.c_[p, np.ones(len(p))] @ m.T)[:, :3]
                    idx = accessor(j, bin_, prim["indices"]).astype(np.int64).ravel() if "indices" in prim else np.arange(len(p))
                    tris.append(p[idx.reshape(-1, 3)])
                    mats.append(np.full(len(idx) // 3, prim.get("material", -1)))
                    uv = accessor(j, bin_, prim["attributes"]["TEXCOORD_0"]) if "TEXCOORD_0" in prim["attributes"] else np.zeros((len(p), 2))
                    uvs.append(uv[idx.reshape(-1, 3)].mean(1))
            elif "mesh" in n:
                # Skinnede nett ligger i skjelettets rom: hjørnene som de er.
                for prim in j["meshes"][n["mesh"]]["primitives"]:
                    p = accessor(j, bin_, prim["attributes"]["POSITION"])
                    idx = accessor(j, bin_, prim["indices"]).astype(np.int64).ravel() if "indices" in prim else np.arange(len(p))
                    tris.append(p[idx.reshape(-1, 3)])
                    mats.append(np.full(len(idx) // 3, prim.get("material", -1)))
                    uv = accessor(j, bin_, prim["attributes"]["TEXCOORD_0"]) if "TEXCOORD_0" in prim["attributes"] else np.zeros((len(p), 2))
                    uvs.append(uv[idx.reshape(-1, 3)].mean(1))
            for c in n.get("children", []):
                walk(c, m)

        for r in scene.get("nodes", []):
            walk(r, np.eye(4))
        self.tris = np.concatenate(tris) if tris else np.zeros((0, 3, 3))
        self.mats = np.concatenate(mats) if mats else np.zeros(0, dtype=int)
        self.uv = np.concatenate(uvs) if uvs else np.zeros((0, 2))

    def tri_colors(self):
        """Fargen per trekant: fra den bakte teksturen ved uv-midtpunktet,
        ellers materialets snittfarge."""
        colors = np.array(self.colors) if self.colors else np.zeros((1, 3)) + 0.6
        out = np.where(self.mats[:, None] >= 0, colors[np.clip(self.mats, 0, len(colors) - 1)], 0.6)
        for k, tex in enumerate(self.images):
            if tex is None:
                continue
            sel = self.mats == k
            if sel.any():
                u = (np.mod(self.uv[sel, 0], 1.0) * 255).astype(int)
                v = (np.mod(self.uv[sel, 1], 1.0) * 255).astype(int)
                out[sel] = tex[v, u]
        return out

    @property
    def bounds(self):
        p = self.tris.reshape(-1, 3)
        return p.min(0), p.max(0)

    def layers(self):
        return sorted({r["layer"] for r in self.roles if r and r.get("layer")})
