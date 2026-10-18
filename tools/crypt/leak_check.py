"""Tetthetssjekk for kryptnivåene: slipper noen stråle ut i «himmelen»?

Fra punkter på navigasjonsnettet (hvert 1 m over hver polygon, og midten av
hver polygon, i øyehøyde) skytes stråler i 128 retninger (også opp og ned).
En stråle som kommer 60 m uten å treffe geometri, eller ut av nivåets
omhylling, er en lekkasje: et hull i muren, i hvelvet eller ved en trapp,
som eieren ser som lyse kiler og himmel inne i krypten.

Geometrien er nivåets flater (`MB` fra `kit_grav.py`, før eksporten) pluss
mekanismene fra byggesettet der merkene står (døra opp, trappa ned,
gitteret, snarveidøra, den falske veggen, fallemmen), som dekker åpningene
sine. Flatene rasteres i et voksel-rutenett på `VOXEL` m (tett prøving av
hver trekant, så tynne vegger blir hele), utvidet ett voksel så en stråle
ikke kan smette diagonalt mellom to voksler, og strålene marsjerer gjennom
rutenettet i steg på et halvt voksel. Hull mindre enn rundt 10 cm ses ikke;
det er de store eieren klager på.

    python3 tools/crypt/leak_check.py 1       # sjekker nivå 1 (bygger det først)
    python3 tools/crypt/leak_check.py 1 --glb # de eksporterte bitene (crypt_01_*.glb) i stedet for flatene i minnet

`build_level.py` kjører sjekken selv og nekter å eksportere et nivå som lekker
(`--no-check` hopper over). Resultatet skrives som `shots/krypten/leaks_NN.txt`
med hullene gruppert per meter, så de kan finnes igjen med
`snapshot --level N --eye X,Y,Z`.
"""
import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

VOXEL = 0.08
STEP = VOXEL * 0.5
REACH = 60.0
DIRS = 128
EYE = 1.6
MARGIN = 1.0  # luft rundt omhyllingen: en stråle som når hit, er ute

# Merke → modellene fra byggesettet som står der (kit_grav.mechanisms()).
KIT_AT_MARK = {
    "up": ["kit_up"], "down": ["kit_down"], "rest": ["kit_rest"], "gate": ["kit_gate"],
    "shortcut": ["kit_shortcut", "kit_shortcut_leaf"], "secret_wall": ["kit_secret_wall"],
    "trapdoor": ["kit_trapdoor"], "ladder": ["kit_ladder"], "lever": ["kit_lever"],
}


def fibonacci_sphere(n):
    i = np.arange(n) + 0.5
    phi = np.arccos(1 - 2 * i / n)
    theta = math.pi * (1 + 5 ** 0.5) * i
    return np.stack([np.cos(theta) * np.sin(phi), np.cos(phi), np.sin(theta) * np.sin(phi)], 1)


def triangles(mb, yaw=0.0, at=(0.0, 0.0, 0.0)):
    """Flatene i en MB som trekanter (N, 3, 3), dreid om y og flyttet."""
    c, s = math.cos(yaw), math.sin(yaw)
    out = []
    for pts, _, _, _ in mb.faces:
        p = np.asarray(pts, np.float64)
        if yaw:
            p = np.stack([p[:, 0] * c + p[:, 2] * s, p[:, 1], -p[:, 0] * s + p[:, 2] * c], 1)
        p = p + np.asarray(at, np.float64)
        for k in range(1, len(p) - 1):
            out.append((p[0], p[k], p[k + 1]))
    return np.asarray(out, np.float64).reshape(-1, 3, 3)


def kit_triangles(marks):
    """Mekanismene fra byggesettet der merkene står."""
    import kit_grav as K
    mech = None
    tris = []
    for m in marks:
        names = KIT_AT_MARK.get(m["kind"])
        if not names or "pos" not in m:
            continue
        if mech is None:
            mech = K.mechanisms()
        for name in names:
            mb = mech.get(name)
            if mb is not None and mb.faces:
                tris.append(triangles(mb, float(m.get("yaw", 0.0)), tuple(m["pos"])))
    return np.concatenate(tris) if tris else np.zeros((0, 3, 3))


def voxelize(tris, lo, shape):
    """Trekantene prøvd tett (hvert halve voksel) inn i et bool-rutenett."""
    solid = np.zeros(shape, bool)
    a, b, c = tris[:, 0], tris[:, 1], tris[:, 2]
    e1, e2 = b - a, c - a
    la, lb = np.linalg.norm(e1, axis=1), np.linalg.norm(e2, axis=1)
    h = STEP
    na = np.maximum(1, np.ceil(la / h)).astype(int)
    nb = np.maximum(1, np.ceil(lb / h)).astype(int)
    # Grupper etter (na, nb) så hver gruppe prøves med én utsending.
    key = na * 100000 + nb
    for kv in np.unique(key):
        sel = np.nonzero(key == kv)[0]
        i, j = int(kv // 100000), int(kv % 100000)
        u = (np.arange(i + 1) / i)[:, None]
        v = (np.arange(j + 1) / j)[None, :]
        keep = (u + v) <= 1.0 + 1e-9
        uu, vv = np.broadcast_to(u, keep.shape)[keep], np.broadcast_to(v, keep.shape)[keep]
        pts = a[sel][:, None, :] + uu[None, :, None] * e1[sel][:, None, :] + vv[None, :, None] * e2[sel][:, None, :]
        idx = np.floor((pts.reshape(-1, 3) - lo) / VOXEL).astype(int)
        ok = np.all((idx >= 0) & (idx < np.asarray(shape)), axis=1)
        idx = idx[ok]
        solid[idx[:, 0], idx[:, 1], idx[:, 2]] = True
    # Utvid ett voksel (2 × 2 × 2), så en stråle ikke smetter diagonalt.
    thick = solid.copy()
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                if dx or dy or dz:
                    thick[dx:, dy:, dz:] |= solid[:shape[0] - dx, :shape[1] - dy, :shape[2] - dz]
    return thick


def room_envelope(rooms, lo, shape):
    """Rommene som prismer (polygonen med 0,5 m luft, gulv − 0,5 til tak + 0,5) i rutenettet: der inne
    er «inne»; der en lekkende stråle sist var inne, er hullet."""
    from PIL import Image, ImageDraw
    inside = np.zeros(shape, bool)
    for r in rooms:
        img = Image.new("1", (shape[2], shape[0]), 0)
        d = ImageDraw.Draw(img)
        from shapely.geometry import Polygon
        poly = Polygon(r["poly"]).buffer(0.5)
        for part in getattr(poly, "geoms", [poly]):
            pts = [((z - lo[2]) / VOXEL, (x - lo[0]) / VOXEL) for x, z in part.exterior.coords]
            d.polygon(pts, fill=1)
        mask = np.asarray(img, bool)  # (x, z)
        y0 = max(0, int((r["y"] - 0.5 - lo[1]) / VOXEL))
        # Opp til toppen av hvelvet eller kuppelen over (rommets h er vederlaget):
        # en stråle som ender oppe i hvelvet i en stor sal etter 60 m, er ikke et hull.
        y1 = min(shape[1], int((r["y"] + r["h"] + 4.6 - lo[1]) / VOXEL) + 1)
        inside[:, y0:y1, :] |= mask[:, None, :]
    return inside


def nav_points(nav):
    """Punkter i øyehøyde over navigasjonsnettet: midten av hver polygon og et rutenett på 1 m."""
    from shapely.geometry import Point, Polygon
    verts = np.asarray(nav["verts"], np.float64)
    out = []
    for poly in nav["polys"]:
        p = verts[poly]
        # Planet gjennom hjørnene (minste kvadrater) gir høyden i hvert punkt.
        A = np.c_[p[:, 0], p[:, 2], np.ones(len(p))]
        coef, *_ = np.linalg.lstsq(A, p[:, 1], rcond=None)
        y_at = lambda x, z: coef[0] * x + coef[1] * z + coef[2]
        shp = Polygon(p[:, [0, 2]])
        cx, cz = shp.centroid.x, shp.centroid.y
        if shp.contains(Point(cx, cz)):
            out.append((cx, y_at(cx, cz) + EYE, cz))
        x0, z0, x1, z1 = shp.bounds
        for x in np.arange(math.ceil(x0), x1 + 1e-6, 1.0):
            for z in np.arange(math.ceil(z0), z1 + 1e-6, 1.0):
                if shp.contains(Point(x, z)):
                    out.append((x, y_at(x, z) + EYE, z))
    return np.asarray(out, np.float64).reshape(-1, 3)


def march(origins, dirs, solid, inside, lo):
    """Marsjer alle strålene; gir (lekkende stråle-indekser, hullsted per stråle)."""
    shape = np.asarray(solid.shape)
    n_o, n_d = len(origins), len(dirs)
    O = np.repeat(origins, n_d, 0)
    D = np.tile(dirs, (n_o, 1))
    N = len(O)
    alive = np.ones(N, bool)
    leaked = np.zeros(N, bool)
    last_in = O.copy()
    steps = int(REACH / STEP)
    idx_all = np.arange(N)
    for k in range(1, steps + 1):
        act = idx_all[alive]
        if not len(act):
            break
        P = O[act] + D[act] * (k * STEP)
        idx = np.floor((P - lo) / VOXEL).astype(int)
        out = np.any((idx < 0) | (idx >= shape), axis=1)
        leaked[act[out]] = True
        alive[act[out]] = False
        rest = ~out
        ai, ii = act[rest], idx[rest]
        hit = solid[ii[:, 0], ii[:, 1], ii[:, 2]]
        alive[ai[hit]] = False
        keep = ai[~hit]
        ik = ii[~hit]
        ins = inside[ik[:, 0], ik[:, 1], ik[:, 2]]
        last_in[keep[ins]] = P[rest][~hit][ins]
    # De som fortsatt lever etter `REACH` lekker bare om de er utenfor omhyllingen der (en lang gang er ikke et hull).
    act = idx_all[alive]
    if len(act):
        P = O[act] + D[act] * (steps * STEP)
        idx = np.clip(np.floor((P - lo) / VOXEL).astype(int), 0, shape - 1)
        leaked[act[~inside[idx[:, 0], idx[:, 1], idx[:, 2]]]] = True
    return np.nonzero(leaked)[0], last_in, O, D


def glb_triangles(path):
    """Trekantene i en glb-fil (posisjonene er float32 i nivåets rom; nodene står i origo)."""
    import json
    import struct
    with open(path, "rb") as f:
        data = f.read()
    magic, _, _ = struct.unpack_from("<III", data, 0)
    assert magic == 0x46546C67, path
    off = 12
    js, bin_ = None, None
    while off < len(data):
        n, typ = struct.unpack_from("<II", data, off)
        chunk = data[off + 8:off + 8 + n]
        if typ == 0x4E4F534A:
            js = json.loads(chunk)
        elif typ == 0x004E4942:
            bin_ = chunk
        off += 8 + n
    comp = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}
    size = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}

    def read(ai):
        acc = js["accessors"][ai]
        bv = js["bufferViews"][acc["bufferView"]]
        dt = np.dtype(comp[acc["componentType"]])
        k = size[acc["type"]]
        start = bv.get("byteOffset", 0) + acc.get("byteOffset", 0)
        stride = bv.get("byteStride", dt.itemsize * k)
        if stride == dt.itemsize * k:
            return np.frombuffer(bin_, dt, acc["count"] * k, start).reshape(acc["count"], k)
        raw = np.frombuffer(bin_, np.uint8, stride * (acc["count"] - 1) + dt.itemsize * k, start)
        return np.lib.stride_tricks.as_strided(raw[:dt.itemsize * k].view(dt), (acc["count"], k), (stride, dt.itemsize)).copy()

    out = []
    for node in js.get("nodes", []):
        if "mesh" not in node:
            continue
        T = np.asarray(node.get("translation", [0, 0, 0]), np.float64)
        for prim in js["meshes"][node["mesh"]]["primitives"]:
            pos = read(prim["attributes"]["POSITION"]).astype(np.float64) + T
            if "indices" in prim:
                idx = read(prim["indices"]).reshape(-1).astype(np.int64)
            else:
                idx = np.arange(len(pos))
            out.append(pos[idx.reshape(-1, 3)])
    return np.concatenate(out) if out else np.zeros((0, 3, 3))


def check(mb, nav, rooms, marks, level_n=0, report_path=None, verbose=True, tris=None):
    """Gir lista over hull: [(x, y, z, antall stråler, rom)] gruppert per meter. `tris` i stedet for
    `mb` sjekker ferdige trekanter (de eksporterte bitene)."""
    tris = triangles(mb) if tris is None else np.asarray(tris, np.float64)
    kt = kit_triangles(marks)
    if len(kt):
        tris = np.concatenate([tris, kt])
    lo = tris.reshape(-1, 3).min(0) - MARGIN
    hi = tris.reshape(-1, 3).max(0) + MARGIN
    shape = tuple(int(math.ceil((hi[i] - lo[i]) / VOXEL)) + 1 for i in range(3))
    if verbose:
        print(f"tetthetssjekk nivå {level_n}: {len(tris)} trekanter, rutenett {shape} à {VOXEL} m")
    solid = voxelize(tris, lo, shape)
    inside = room_envelope(rooms, lo, shape)
    pts = nav_points(nav)
    dirs = fibonacci_sphere(DIRS)
    leaks, last_in, O, D = march(pts, dirs, solid, inside, lo)
    if verbose:
        print(f"  {len(pts)} punkter × {DIRS} retninger: {len(leaks)} lekkende stråler")
    # Grupper hullene per meter.
    groups = {}
    for i in leaks:
        h = last_in[i]
        key = (round(float(h[0])), round(float(h[1])), round(float(h[2])))
        g = groups.setdefault(key, [0, None])
        g[0] += 1
        if g[1] is None:
            g[1] = (O[i], D[i])
    from shapely.geometry import Point, Polygon
    polys = [(r["id"], Polygon(r["poly"]), r["y"], r["h"]) for r in rooms]

    def room_of(x, y, z):
        best = None
        for rid, poly, y0, h in polys:
            d = poly.distance(Point(x, z)) + (max(0.0, y0 - y) + max(0.0, y - y0 - h))
            if best is None or d < best[0]:
                best = (d, rid)
        return best[1] if best else "?"

    out = []
    for (x, y, z), (count, (o, d)) in sorted(groups.items(), key=lambda kv: -kv[1][0]):
        out.append((x, y, z, count, room_of(x, y, z), tuple(float(v) for v in np.round(o, 2)), tuple(float(v) for v in np.round(d, 2))))
    lines = [f"nivå {level_n}: {len(leaks)} lekkende stråler fra {len(pts)} punkter, {len(out)} hull (gruppert per meter)"]
    for x, y, z, count, rid, o, d in out:
        lines.append(f"  hull ved ({x}, {y}, {z}) i {rid}: {count} stråler, f.eks. fra {o} retning {d}")
    if report_path:
        os.makedirs(os.path.dirname(report_path), exist_ok=True)
        with open(report_path, "w") as f:
            f.write("\n".join(lines) + "\n")
    if verbose:
        print("\n".join(lines[:41]))
        if len(lines) > 41:
            print(f"  … og {len(lines) - 41} til (se {report_path})")
    return out


def main():
    import json
    import build_level as B
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    n = int(args[0]) if args else 1
    report = os.path.join(B.ROOT, "shots", "krypten", f"leaks_{n:02d}.txt")
    if "--glb" in sys.argv:  # de eksporterte bitene, slik motoren får dem
        with open(os.path.join(B.OUT, f"crypt_{n:02d}.json")) as f:
            data = json.load(f)
        tris = np.concatenate([glb_triangles(os.path.join(B.OUT, f"{name}.glb")) for name in data["pieces"]])
        leaks = check(None, data["nav"], data["rooms"], data["marks"], n, report.replace(".txt", "_glb.txt"), tris=tris)
    else:
        L = B.Level(n)
        L.build()
        leaks = check(L.mb, L.nav(), L.rooms_json(), L.marks, n, report)
    sys.exit(1 if leaks else 0)


if __name__ == "__main__":
    main()
