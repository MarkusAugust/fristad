"""Slanker glb-filene fra Blender uten å endre utseendet:

- normalene lagres som tre bytes med fortegn (normalisert) i stedet for tre
  flyttall (KHR_mesh_quantization), og
- hjørnefargen (okklusjonen) som fire bytes i stedet for fire 16-bits tall.

Motoren pakker ut normalene selv (`packed_normals` i src/adapters/gltf.rs).
Filer som allerede er slanket, hoppes over.

    python3 tools/models/slim.py assets/models/**/*.glb
"""
import json
import struct
import sys

F32, U16, U8, I8 = 5126, 5123, 5121, 5120
SIZE = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
COUNT = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}


def read(path):
    b = open(path, "rb").read()
    jl = struct.unpack("<I", b[12:16])[0]
    j = json.loads(b[20 : 20 + jl])
    off = 20 + jl
    bin_ = b""
    if off < len(b):
        bl = struct.unpack("<I", b[off : off + 4])[0]
        bin_ = b[off + 8 : off + 8 + bl]
    return j, bin_


def write(path, j, bin_):
    js = json.dumps(j, separators=(",", ":")).encode()
    js += b" " * (-len(js) % 4)
    bin_ += b"\0" * (-len(bin_) % 4)
    out = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(bin_))
    out += struct.pack("<II", len(js), 0x4E4F534A) + js
    out += struct.pack("<II", len(bin_), 0x004E4942) + bin_
    open(path, "wb").write(out)


def elements(j, bin_, a):
    """Verdiene i en accessor som lister av tall."""
    acc = j["accessors"][a]
    view = j["bufferViews"][acc["bufferView"]]
    n, ct = COUNT[acc["type"]], acc["componentType"]
    size = SIZE[ct]
    stride = view.get("byteStride", n * size)
    start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    fmt = "<" + {F32: "f", U16: "H", U8: "B", I8: "b", 5122: "h", 5125: "I"}[ct] * n
    out = []
    for i in range(acc["count"]):
        v = struct.unpack_from(fmt, bin_, start + i * stride)
        if acc.get("normalized"):
            v = tuple(x / {U16: 65535.0, U8: 255.0, I8: 127.0, 5122: 32767.0}[ct] for x in v)
        out.append(v)
    return out


def slim(path):
    j, bin_ = read(path)
    if "KHR_mesh_quantization" in j.get("extensionsUsed", []):
        return None
    before = len(bin_)
    views = j["bufferViews"]
    new = {}  # accessor → (bytes, stride)
    for mesh in j.get("meshes", []):
        for prim in mesh["primitives"]:
            attrs = prim["attributes"]
            a = attrs.get("NORMAL")
            if a is not None and a not in new and j["accessors"][a]["componentType"] == F32:
                data = bytearray()
                for x, y, z in elements(j, bin_, a):
                    q = [max(-127, min(127, round(c * 127.0))) for c in (x, y, z)]
                    data += struct.pack("<bbbb", *q, 0)
                new[a] = (bytes(data), 4, I8, "VEC3")
            a = attrs.get("COLOR_0")
            if a is not None and a not in new and j["accessors"][a]["componentType"] != U8:
                data = bytearray()
                for c in elements(j, bin_, a):
                    c = list(c) + [1.0] * (4 - len(c))
                    data += struct.pack("<BBBB", *[max(0, min(255, round(v * 255.0))) for v in c])
                new[a] = (bytes(data), 4, U8, "VEC4")
    if not new:
        return None
    # De gamle visningene som bare de omgjorte accessorene bruker, faller ut.
    used = set()
    for i, acc in enumerate(j["accessors"]):
        if i not in new and "bufferView" in acc:
            used.add(acc["bufferView"])
    for img in j.get("images", []):
        if "bufferView" in img:
            used.add(img["bufferView"])
    out = bytearray()
    remap = {}
    new_views = []
    for i, v in enumerate(views):
        if i not in used:
            continue
        out += b"\0" * (-len(out) % 4)
        start = v.get("byteOffset", 0)
        chunk = bin_[start : start + v["byteLength"]]
        nv = dict(v)
        nv["byteOffset"] = len(out)
        out += chunk
        remap[i] = len(new_views)
        new_views.append(nv)
    for i, acc in enumerate(j["accessors"]):
        if i in new:
            data, stride, ct, typ = new[i]
            out += b"\0" * (-len(out) % 4)
            new_views.append({"buffer": 0, "byteOffset": len(out), "byteLength": len(data), "byteStride": stride, "target": 34962})
            out += data
            acc["bufferView"] = len(new_views) - 1
            acc["byteOffset"] = 0
            acc["componentType"] = ct
            acc["normalized"] = True
            acc["type"] = typ
            acc.pop("min", None)
            acc.pop("max", None)
        elif "bufferView" in acc:
            acc["bufferView"] = remap[acc["bufferView"]]
    for img in j.get("images", []):
        if "bufferView" in img:
            img["bufferView"] = remap[img["bufferView"]]
    j["bufferViews"] = new_views
    j["buffers"][0]["byteLength"] = len(out)
    j.setdefault("extensionsUsed", []).append("KHR_mesh_quantization")
    write(path, j, bytes(out))
    return before, len(out)


if __name__ == "__main__":
    total = [0, 0]
    for p in sys.argv[1:]:
        r = slim(p)
        if r:
            total[0] += r[0]
            total[1] += r[1]
            print(f"{p}: {r[0] / 1e6:.1f} → {r[1] / 1e6:.1f} MB")
    print(f"I alt {total[0] / 1e6:.0f} → {total[1] / 1e6:.0f} MB")
