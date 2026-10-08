"""Gresstustene på enga: kryssede kort med strå og tuster fra gressatlaset
(tools/textures/grass_atlas.py, fra eierens ark). Kjør:

    python3 tools/models/grass.py

Fire varianter i assets/models/trees/ (og fire blomstertuster flowers_a–d
fra eierens blomsterark): grass_a (grønne tuster), grass_b
(tørre tuster), grass_c (spredte strå) og grass_d (høye strå med aks).
Hvert kort er to ledd høyt og bøyer litt utover, normalen peker opp (så
baksidene ikke blir svarte), og materialet heter «grass»: motoren
alfatester det og lar det svaie, mer jo høyere opp. Rota står i origo."""

import json
import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import slim  # noqa: E402
import bmesh  # noqa: E402
from common import clear  # noqa: E402
from mathutils import Vector  # noqa: E402

OUT = "assets/models/trees"
ATLAS = f"{OUT}/grass_atlas.png"
ITEMS = json.load(open(f"{OUT}/grass_items.json"))["items"]
FLOWER_ATLAS = f"{OUT}/flowers_atlas.png"
# Blomstene fra eierens ark (tools/textures/sheets.py flowers), etter slag.
FLOWERS = {}
for it in json.load(open(f"{OUT}/flowers_items.json"))["items"]:
    FLOWERS.setdefault(it["kind"], []).append(it["box"])

# Bitene i atlaset etter slag (indeksene fra grass_items.json).
CLUMPS = [28, 29, 32, 33]
DRY = [34, 27]
BLADES = [21, 22, 23, 25, 26, 30, 31, 18, 2]
STALKS = [5, 6, 8, 9, 10, 11, 13, 14, 16, 17]


def material(atlas=ATLAS):
    m = bpy.data.materials.new("grass")
    m.use_nodes = True
    nt = m.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = bpy.data.images.load(os.path.abspath(atlas))
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    nt.links.new(tex.outputs["Alpha"], bsdf.inputs["Alpha"])
    bsdf.inputs["Roughness"].default_value = 0.8
    try:
        m.blend_method = "CLIP"
    except Exception:
        pass
    return m


def card(bm, uvl, item, height, at, yaw, lean, rng):
    """Et kort med biten `item`, `height` høyt, rota i `at`, dreid `yaw`, i
    to ledd som bøyer `lean` utover."""
    x0, y0, x1, y1 = ITEMS[item] if isinstance(item, int) else item
    width = height * (x1 - x0) / (y1 - y0)
    right = Vector((math.cos(yaw), math.sin(yaw), 0)) * (width / 2)
    out = Vector((-math.sin(yaw), math.cos(yaw), 0)) * lean * rng.choice((-1, 1))
    rows = []
    for t in (0.0, 0.5, 1.0):
        c = Vector(at) + Vector((0, 0, height * t)) + out * (t * t)
        v = 1.0 - (y1 - (y1 - y0) * t)
        rows.append(([bm.verts.new(c - right), bm.verts.new(c + right)], v))
    for (lo, vlo), (hi, vhi) in zip(rows, rows[1:]):
        f = bm.faces.new((lo[0], lo[1], hi[1], hi[0]))
        for loop, uv in zip(f.loops, [(x0, vlo), (x1, vlo), (x1, vhi), (x0, vhi)]):
            loop[uvl].uv = uv


def tuft(name, seed, parts, atlas=ATLAS):
    """`parts`: [(bitene, antall, høyde (min, maks), spredning)]."""
    clear()
    rng = random.Random(seed)
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.new("UVMap")
    for pool, n, (hmin, hmax), spread in parts:
        base_yaw = rng.uniform(0, math.pi)
        for k in range(n):
            item = rng.choice(pool)
            r = rng.uniform(0, spread)
            a = rng.uniform(0, 2 * math.pi)
            yaw = base_yaw + k * math.pi / max(n, 1) + rng.uniform(-0.2, 0.2)
            card(bm, uvl, item, rng.uniform(hmin, hmax), (r * math.cos(a), r * math.sin(a), 0.0), yaw, rng.uniform(0.02, 0.08), rng)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.normals_split_custom_set_from_vertices([(0.0, 0.0, 1.0)] * len(me.vertices))
    me.materials.append(material(atlas))
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    path = f"{OUT}/{name}.glb"
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=True, export_image_format="AUTO", export_animations=False, export_skins=False, export_cameras=False, export_lights=False)
    slim.slim(path)  # normalene og hjørnefargen pakkes (slim.py)
    print(f"{path}: {len(me.polygons) * 2} trekanter, {os.path.getsize(path) // 1024} kB")


TUFTS = {
    "grass_a": (1, [(CLUMPS, 3, (0.38, 0.55), 0.05), (BLADES, 2, (0.3, 0.45), 0.2)]),
    "grass_b": (2, [(DRY, 3, (0.4, 0.6), 0.05), (STALKS, 2, (0.55, 0.8), 0.15)]),
    "grass_c": (3, [(BLADES, 4, (0.28, 0.48), 0.2)]),
    "grass_d": (4, [(STALKS, 5, (0.6, 0.95), 0.18), (BLADES, 2, (0.3, 0.42), 0.1)]),
}

# Blomstertuster: noen planter av ett eller to slag, med gresstrå rundt
# fra gressatlaset er ikke mulig i samme materiale, så tustene er rene
# blomster (gresset står tett rundt dem på enga).
FLOWER_TUFTS = {
    "flowers_a": (11, [(FLOWERS["buttercup"], 3, (0.3, 0.45), 0.12), (FLOWERS["clover"], 2, (0.18, 0.26), 0.1)]),
    "flowers_b": (12, [(FLOWERS["harebell"], 3, (0.28, 0.4), 0.12)]),
    "flowers_c": (13, [(FLOWERS["yarrow"], 2, (0.45, 0.6), 0.1), (FLOWERS["buttercup"], 1, (0.3, 0.4), 0.1)]),
    "flowers_d": (14, [(FLOWERS["cowparsley"], 2, (0.7, 0.95), 0.12), (FLOWERS["clover"], 2, (0.18, 0.26), 0.12)]),
}

if __name__ == "__main__":
    for name, (seed, parts) in TUFTS.items():
        tuft(name, seed, parts)
    for name, (seed, parts) in FLOWER_TUFTS.items():
        tuft(name, seed, parts, FLOWER_ATLAS)
