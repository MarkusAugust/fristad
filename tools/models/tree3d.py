"""Trærne fra TRELLIS.2 (`assets/textures/nature/3d/<kilde>.glb`), gjort
spillklare som to modeller med samme origo (foten av stammen):

- `<navn>_trunk`: stammen og greinene. Løvet skilles ut etter fargen i
  TRELLIS-teksturen (grønt), resten forenkles som gjenstandene i
  `item3d.py` (voxel-skall, `folk3d.simplify`, fargen hentet fra den fulle
  formen med `folk3d.transfer`), og en grov utgave (`far`, noden motoren
  legger i det fjerne detaljnivået) forenkles fra den. Kollideren er et
  åttekantet skrog rundt stammen opp til `SOLID_H` (`extras.solids`).
- `<navn>_leaves`: løvet som alfakort (alpha-to-coverage, `card_alpha`) med
  klasene fra eierens løvark (`assets/textures/atlas/`), tonet mot fargen
  TRELLIS ga løvet. Kortene henger der TRELLIS hadde blader (punkter
  trukket etter bladenes flate, minst `spacing` fra hverandre), dreid ut
  fra stammen, med lysstyrken fra TRELLIS i okklusjonen (mørkere inne i
  kronen). Rekkefølgen er stokket, så motorens fjerne nivå (hvert tredje
  kort, forstørret, `Mesh::thinned_cards`) dekker hele kronen.

Vurdert og forkastet (20. oktober): hele treet forenklet som én form
(kronen ble grønne klumper og rankene pølser), og to kryssede kort per rute
på 0,8 m (blekt og tett).

    python3.13 tools/models/tree3d.py [tree_birch ...]
    python3.13 tools/models/tree3d.py --solids [tree_birch ...]  # bare kollideren
"""

import math
import os
import sys

import numpy as np

sys.path.insert(0, __file__.rsplit("/", 1)[0])
sys.path.insert(0, os.path.join(__file__.rsplit("/", 1)[0], "..", "textures"))
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import bake_textured, clear  # noqa: E402
import folk3d  # noqa: E402
import item3d  # noqa: E402

SRC = "assets/textures/nature/3d"
OUT = "assets/models/nature"
SOLID_H = 2.0  # kollideren rundt stammen går så høyt (m)
ROOT_FLARE = 0.6  # over rotfoten (m), der stammen måles til kollideren

# navn: kilde, høyde (m), løvarket og rutene i det (kolonner, rader),
# trekanter i stammen nær og fjern, voxel (m), teksturstørrelsene,
# avstanden mellom kortene (m), kortets høyde (m), og under hvilken høyde
# (m) det grønne ikke blir kort (mosen ved rota).
TREES = {
    # Bjørka (eieren 20. oktober): TRELLIS ga den med løv i geometri
    # (456 000 av 489 000 trekanter), hengende ranker og en mosetue ved rota.
    "tree_birch": dict(src="tree_birch", height=9.0, sheet="assets/textures/atlas/leaves_birch.jpeg", grid=(4, 3), tris=2600, far_tris=380, voxel=0.022, size=2048, leaf_size=2048, spacing=0.36, card=1.05, low=2.4, tone=0.85),
}


def face_colors(ob):
    """Fargen (sRGB, 0–1) midt på hver flate, fra TRELLIS-teksturen."""
    me = ob.data
    tex = next(n.image for m in me.materials for n in m.node_tree.nodes if n.type == "TEX_IMAGE" and any(l.to_socket.name == "Base Color" for l in n.outputs[0].links))
    tw, th = tex.size
    px = np.empty(tw * th * 4, np.float32)
    tex.pixels.foreach_get(px)
    px = px.reshape(th, tw, 4)[..., :3]
    uv = np.empty(len(me.loops) * 2, np.float32)
    me.uv_layers[0].data.foreach_get("uv", uv)
    uv = uv.reshape(-1, 2)
    start = np.empty(len(me.polygons), np.int64)
    me.polygons.foreach_get("loop_start", start)
    count = np.empty(len(me.polygons), np.int64)
    me.polygons.foreach_get("loop_total", count)
    c = np.zeros((len(me.polygons), 2), np.float32)
    for k in range(count.max()):
        c += uv[start + np.minimum(k, count - 1)]
    c /= count.max()
    x = np.clip((c[:, 0] % 1.0) * tw, 0, tw - 1).astype(int)
    y = np.clip((c[:, 1] % 1.0) * th, 0, th - 1).astype(int)
    col = px[y, x]
    # Bildet i Blender er lineært for fargerommet sRGB; tilbake til sRGB.
    if tex.colorspace_settings.name == "sRGB":
        col = np.where(col <= 0.0031308, col * 12.92, 1.055 * np.power(np.clip(col, 0, 1), 1 / 2.4) - 0.055)
    return col


def green(col):
    r, g, b = col[:, 0], col[:, 1], col[:, 2]
    return (g > r * 1.08) & (g > b * 1.15) & (g - np.minimum(r, b) > 0.06)


def faces(ob):
    me = ob.data
    n = len(me.polygons)
    ctr = np.empty(n * 3, np.float32)
    me.polygons.foreach_get("center", ctr)
    area = np.empty(n, np.float32)
    me.polygons.foreach_get("area", area)
    return ctr.reshape(-1, 3), area


def drop_faces(ob, mask):
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    bm.faces.ensure_lookup_table()
    bmesh.ops.delete(bm, geom=[bm.faces[i] for i in np.nonzero(mask)[0]], context="FACES")
    bm.to_mesh(ob.data)
    bm.free()


def stem_solid(co, h, step=0.5):
    """Loddrette, åttekantede skrog rundt stammen fra foten til `h`
    (glTF: y opp), ett per `step` m. Kolliderne er loddrette prismer
    (`Shape::covers_shaped`), så den skrå stammen dekkes av et prisme per
    meter rundt tverrsnittene i den (fra rotfoten, `ROOT_FLARE`, nederst),
    og ingen usynlig vegg står mer enn noen centimeter utenfor barken."""
    out = []
    for y0 in np.arange(0.0, h - 1e-3, step):
        y1 = min(y0 + step, h)
        rings = []
        for z0 in np.linspace(max(y0, ROOT_FLARE), max(y1, ROOT_FLARE), 3):
            ring = co[np.abs(co[:, 2] - z0) < 0.1]
            mid = ring[:, :2].mean(0)
            rings.append((mid, float(np.percentile(np.linalg.norm(ring[:, :2] - mid, axis=1), 70))))
        mid = np.mean([m for m, _ in rings], axis=0)
        r = max(float(np.linalg.norm(m - mid)) + rr for m, rr in rings)
        pts = []
        for y in (y0, y1):
            for k in range(8):
                a = k * math.pi / 4
                x, z = mid[0] + r * math.cos(a), mid[1] + r * math.sin(a)
                # Blender (x, y, z) → glTF (x, z, −y).
                pts.append([round(float(x), 3), round(float(y), 3), round(float(-z), 3)])
        print(f"kollideren {y0:.1f}–{y1:.1f} m: midt", mid.round(3), "radius", round(r, 3))
        out.append({"hull": pts})
    return out


def trunk(name, spec, ob):
    """Stammen: det grønne bort, forenklet med fargen fra den fulle formen,
    eksportert med den grove utgaven og kollideren."""
    hi = ob.copy()
    hi.data = ob.data.copy()
    bpy.context.scene.collection.objects.link(hi)
    v = spec["voxel"]
    keep = {k: getattr(folk3d, k) for k in ("VOXEL", "THICK", "LOOSE")}
    try:
        folk3d.VOXEL, folk3d.THICK, folk3d.LOOSE = v, 1.2 * v, 0.01
        folk3d.simplify(ob, spec["tris"])
        folk3d.transfer(hi, ob, name, spec["size"], unwrap=item3d.atlas)
    finally:
        for k, val in keep.items():
            setattr(folk3d, k, val)
    ob.name = f"{name}_trunk"
    print(ob.name, "trekanter", sum(len(p.vertices) - 2 for p in ob.data.polygons))
    path = f"{OUT}/{name}_trunk.glb"
    bake_textured(path, spec["size"], keep=("glass", "magic"), uv_angle=None, ao=False, pack=False, data_size=1024)
    # Den grove utgaven: samme tekstur og uv, forenklet fra den ferdige.
    clear()
    bpy.ops.import_scene.gltf(filepath=path)
    near = next(o for o in bpy.data.objects if o.type == "MESH")
    far = near.copy()
    far.data = near.data.copy()
    bpy.context.scene.collection.objects.link(far)
    far.name = "far"
    item3d.decimate(far, spec["far_tris"])
    print("fjern", sum(len(p.vertices) - 2 for p in far.data.polygons), "trekanter")
    export(path)
    solids(path)


def solids(path):
    """Kollideren etter stammen i den eksporterte fila (uten Blender:
    `python3 tools/models/tree3d.py --solids`)."""
    import slim

    j, bin_ = slim.read(path)
    node = next(n for n in j["nodes"] if n.get("name") != "far" and "mesh" in n)
    acc = j["meshes"][node["mesh"]]["primitives"][0]["attributes"]["POSITION"]
    p = np.array(list(slim.elements(j, bin_, acc)), np.float64)
    co = np.stack([p[:, 0], -p[:, 2], p[:, 1]], axis=1)  # glTF → Blender
    write_solids(path, stem_solid(co, SOLID_H))


def export(path):
    import slim

    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=True, export_image_format="AUTO", export_vertex_color="ACTIVE", export_animations=False, export_cameras=False, export_lights=False)
    slim.slim(path)


def write_solids(path, solids):
    import slim

    j, bin_ = slim.read(path)
    j.setdefault("extras", {})["solids"] = solids
    slim.write(path, j, bin_)


def sprites(spec, target):
    """Klasene i løvarket skåret ut mot det hvite, tonet mot `target`
    (sRGB) og lagt i et kvadrat med én rute per klase. Gir bildet (RGBA)
    og hver klases uv-rute (u0, v0, u1, v1) med stilken øverst."""
    from PIL import Image
    from leaf_atlas import cutout

    rgb = np.asarray(Image.open(spec["sheet"]).convert("RGB")).astype(np.float32)
    bg = np.median(np.concatenate([rgb[:8].reshape(-1, 3), rgb[-8:].reshape(-1, 3)]), axis=0)
    rgba = cutout(rgb, bg)
    cols, rows = spec["grid"]
    H, W = rgba.shape[:2]
    cells = []
    for r in range(rows):
        for c in range(cols):
            cell = rgba[r * H // rows:(r + 1) * H // rows, c * W // cols:(c + 1) * W // cols]
            ys, xs = np.nonzero(cell[..., 3] > 128)
            if len(ys) < 200:
                continue
            cells.append(cell[ys.min():ys.max() + 1, xs.min():xs.max() + 1])
    # Fargen: snittet av bladene (ikke stilkene) dras mot TRELLIS.
    allpx = np.concatenate([c[..., :3][c[..., 3] > 200] for c in cells]) / 255.0
    leafy = allpx[green(allpx)]
    mean = leafy.mean(0)
    gain = 1.0 + (np.asarray(target) / mean - 1.0) * spec["tone"]
    print("løvarket", mean.round(3), "TRELLIS", np.round(target, 3), "forsterkning", gain.round(3))
    S = spec["leaf_size"]
    out = np.zeros((S, S, 4), np.float32)
    out[..., :3] = np.asarray(target) * 255.0
    cw, ch = S // cols, S // rows
    rects = []
    for k, cell in enumerate(cells):
        r, c = divmod(k, cols)
        s = min((cw - 8) / cell.shape[1], (ch - 8) / cell.shape[0])
        img = Image.fromarray(cell.clip(0, 255).astype(np.uint8), "RGBA").resize((int(cell.shape[1] * s), int(cell.shape[0] * s)), Image.LANCZOS)
        a = np.asarray(img).astype(np.float32)
        a[..., :3] = np.clip(a[..., :3] * gain, 0, 255)
        x0 = c * cw + (cw - a.shape[1]) // 2
        y0 = r * ch + 4
        out[y0:y0 + a.shape[0], x0:x0 + a.shape[1]] = a
        rects.append((x0 / S, 1 - (y0 + a.shape[0]) / S, (x0 + a.shape[1]) / S, 1 - y0 / S))
    return out, rects


def leaves(name, spec, pts, area, col):
    """Kortene der TRELLIS hadde løv."""
    from PIL import Image
    from scipy.spatial import cKDTree

    rng = np.random.default_rng(1350)
    up = pts[:, 2] > spec["low"]
    pts, area, col = pts[up], area[up], col[up]
    target = (col * area[:, None]).sum(0) / area.sum()
    img, rects = sprites(spec, target)
    os.makedirs("target/tree3d", exist_ok=True)
    tex = f"target/tree3d/{name}_leaves.png"
    Image.fromarray(img.clip(0, 255).astype(np.uint8), "RGBA").save(tex)
    # Punkter trukket etter flaten, og så de som står minst `spacing` fra
    # hverandre (grådig, i trukket rekkefølge).
    pick = rng.choice(len(pts), size=min(60000, len(pts)), p=area / area.sum())
    cand = pts[pick]
    tree = cKDTree(pts)
    kept = []
    grid = {}
    sp = spec["spacing"]
    for p in cand:
        key = tuple((p // sp).astype(int))
        near = [q for dx in (-1, 0, 1) for dy in (-1, 0, 1) for dz in (-1, 0, 1) for q in grid.get((key[0] + dx, key[1] + dy, key[2] + dz), [])]
        if any(np.linalg.norm(p - q) < sp for q in near):
            continue
        grid.setdefault(key, []).append(p)
        kept.append(p)
    kept = np.array(kept)
    rng.shuffle(kept)
    lum = lambda c: c @ np.array([0.3, 0.59, 0.11])  # noqa: E731
    mean_lum = lum(target)
    lo, hi = kept.min(0), kept.max(0)
    # Midten av kronen: aksen gjennom snittet, litt under midten i høyden.
    axis = kept[:, :2].mean(0)
    crown_mid = np.array([axis[0], axis[1], lo[2] + 0.45 * (hi[2] - lo[2])])
    reach = np.percentile(np.linalg.norm(kept[:, :2] - axis, axis=1), 90)
    verts, faces_, uvs, occl, norms = [], [], [], [], []
    for p in kept:
        # Lysstyrken rundt punktet i TRELLIS (bladene innerst er mørkere).
        idx = tree.query_ball_point(p, 0.45)
        local = lum((col[idx] * area[idx, None]).sum(0) / area[idx].sum())
        shade = float(np.clip(local / mean_lum, 0.7, 1.1))
        out = p[:2] - axis
        rad = float(np.linalg.norm(out))
        out = out / rad if rad > 1e-3 else np.array([1.0, 0.0])
        # Kortet vender ut fra stammen, dreid tilfeldig.
        a = math.atan2(out[1], out[0]) + rng.uniform(-1.0, 1.0)
        face = np.array([math.cos(a), math.sin(a), 0.0])
        width_dir = np.array([-math.sin(a), math.cos(a), 0.0])
        # Klasen henger ned, vippet utover; øverst i kronen mer flatt, så
        # kronen også dekker ovenfra.
        top = (p[2] - lo[2]) / max(hi[2] - lo[2], 1e-3)
        tilt = rng.uniform(0.1, 0.45) + max(0.0, top - 0.7) * 2.2
        down = np.array([0, 0, -1.0]) * math.cos(tilt) + face * math.sin(tilt)
        k = rng.integers(len(rects))
        u0, v0, u1, v1 = rects[k]
        aspect = (v1 - v0) / (u1 - u0) * (img.shape[1] / img.shape[0])
        h = spec["card"] * rng.uniform(0.8, 1.2)
        w = h / aspect
        t = p - down * 0.18 * h
        b = t + down * h
        corners = [t - width_dir * w / 2, t + width_dir * w / 2, b + width_dir * w / 2, b - width_dir * w / 2]
        if rng.random() < 0.5:
            u0, u1 = u1, u0  # speilet
        cu = [(u0, v1), (u1, v1), (u1, v0), (u0, v0)]
        n_card = np.cross(width_dir, down)
        if n_card @ np.append(out, 0.0) < 0:
            n_card = -n_card
        base = len(verts)
        for corner, uv in zip(corners, cu):
            radial = corner - crown_mid
            radial /= max(np.linalg.norm(radial), 1e-3)
            n = 0.45 * n_card + 0.55 * radial
            norms.append(n / np.linalg.norm(n))
            verts.append(corner)
            uvs.append(uv)
            # Okklusjonen: inne i kronen og nederst mørkere.
            inner = np.clip(np.linalg.norm(corner[:2] - axis) / max(reach, 1e-3), 0, 1)
            occl.append(float(np.clip((0.62 + 0.3 * inner + 0.12 * top) * shade, 0.35, 1.0)))
        faces_.append((base, base + 1, base + 2, base + 3))
    print(name, "kort", len(kept), "trekanter", 2 * len(kept))
    clear()
    me = bpy.data.meshes.new(f"{name}_leaves")
    me.from_pydata([tuple(v) for v in verts], [], faces_)
    me.update()
    uvl = me.uv_layers.new(name="UVMap")
    for poly in me.polygons:
        for li in poly.loop_indices:
            uvl.data[li].uv = uvs[me.loops[li].vertex_index]
    ca = me.color_attributes.new("Color", "BYTE_COLOR", "CORNER")
    for li, loop in enumerate(me.loops):
        o = occl[loop.vertex_index]
        ca.data[li].color = (o, o, o, 1.0)
    me.normals_split_custom_set([tuple(norms[l.vertex_index]) for l in me.loops])
    ob = bpy.data.objects.new(f"{name}_leaves", me)
    bpy.context.scene.collection.objects.link(ob)
    mat = bpy.data.materials.new(f"leaf_{name}")
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    t = nt.nodes.new("ShaderNodeTexImage")
    t.image = bpy.data.images.load(tex)
    nt.links.new(t.outputs["Color"], bsdf.inputs["Base Color"])
    nt.links.new(t.outputs["Alpha"], bsdf.inputs["Alpha"])
    bsdf.inputs["Roughness"].default_value = 0.9
    me.materials.append(mat)
    export(f"{OUT}/{name}_leaves.glb")
    os.remove(tex)


def build(name):
    spec = TREES[name]
    clear()
    ob = item3d.load(spec["src"], 0.0, folder=SRC)
    item3d.place(ob, 1.0 / float(np.ptp(item3d.coords(ob)[:, 2])))
    item3d.place(ob, spec["height"])
    col = face_colors(ob)
    ctr, area = faces(ob)
    leaf = green(col)
    print(name, "flater", len(col), "grønne", int(leaf.sum()))
    # Origo midt i stammen ved foten.
    base = ctr[(~leaf) & (ctr[:, 2] < 0.4)][:, :2].mean(0)
    ob.data.transform(__import__("mathutils").Matrix.Translation((-base[0], -base[1], 0.0)))
    ctr[:, :2] -= base
    pts, la, lc = ctr[leaf], area[leaf], col[leaf]
    drop_faces(ob, leaf)
    trunk(name, spec, ob)
    leaves(name, spec, pts, la, lc)


if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in TREES] or list(TREES)
    if "--solids" in sys.argv:
        for n in names:
            solids(f"{OUT}/{n}_trunk.glb")
        sys.exit(0)
    for n in names:
        build(n)
