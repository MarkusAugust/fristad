"""Trærne og buskene rundt Fristad, laget i Blender (bpy) og eksportert til
assets/models/trees/. Kjør:

    python3 tools/models/trees.py [oak1 oak2 birch1 spruce1 spruce2 bush1 bush2 ...]

Hvert tre er to modeller: stammen med greinene (`*_trunk.glb`, med bark
bakt som albedo, normalkart og ruhet fra prosedyriske noder) og
bladverket (`*_leaves.glb` / `*_needles.glb`): kort med bladteksturen
fra leaves.py, materialet heter «leaf» eller «needle» så motoren
alfatester og lar dem svaie. Origo er i bakken under stammen, meter.
Greinene vokser rekursivt: hver grein er en NURBS-kurve med radius per
punkt, bevlet til et rør."""

import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from mathutils import Quaternion, Vector  # noqa: E402

OUT = "assets/models/trees"
BAKE = 1024


def clear():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def perpendicular(d, rng):
    """En tilfeldig retning vinkelrett på d."""
    a = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))
    p = a - d * a.dot(d)
    if p.length < 1e-3:
        p = d.cross(Vector((1, 0, 0)))
    return p.normalized()


class Species:
    def __init__(self, **kw):
        self.levels = 3
        self.children = [5, 4, 3]
        self.angle = [55, 45, 40]
        self.ratio = [0.55, 0.6, 0.55]
        self.up = [0.0, 0.25, 0.15, 0.05]
        self.wobble = 0.12
        self.trunk_len = 5.5
        self.trunk_r = 0.28
        self.card = 0.75
        self.cards = 3
        self.segments = 7
        self.__dict__.update(kw)


OAK = Species(children=[4, 3, 3], card=0.85, cards=4)
BIRCH = Species(levels=3, children=[4, 3, 3], angle=[32, 40, 55], ratio=[0.62, 0.55, 0.5], up=[0.0, 0.05, -0.1, -0.35], wobble=0.08, trunk_len=8.0, trunk_r=0.19, card=0.7, cards=4)
BUSH = Species(levels=2, children=[4, 3], angle=[45, 50], ratio=[0.8, 0.7], up=[0.0, 0.15, 0.05], wobble=0.2, trunk_len=0.9, trunk_r=0.04, card=0.45, cards=4, segments=4)


def grow(p, d, length, radius, level, sp, rng, splines, tips, taper_end):
    n = sp.segments
    pts = []
    pos = p.copy()
    dirv = d.normalized()
    seg = length / n
    for i in range(n + 1):
        r = radius * (1.0 - taper_end * i / n)
        pts.append((pos.copy(), max(r, 0.008)))
        wob = Vector((rng.gauss(0, sp.wobble), rng.gauss(0, sp.wobble), rng.gauss(0, sp.wobble)))
        dirv = (dirv + wob * seg + Vector((0, 0, sp.up[min(level, len(sp.up) - 1)])) * seg).normalized()
        pos = pos + dirv * seg
    splines.append((pts, level))
    if level >= sp.levels - 1:
        # Ytterste greinene: blader langs den ytre halvdelen.
        for k in range(n // 2, n + 1):
            tips.append((pts[k][0], dirv))
    if level >= sp.levels:
        return
    count = sp.children[min(level, len(sp.children) - 1)]
    golden = 2.399963
    for c in range(count):
        t = (0.3 if level == 0 else 0.25) + (1.0 - (0.3 if level == 0 else 0.25)) * (c + rng.random() * 0.8) / count
        idx = min(int(t * n), n)
        bp = pts[idx][0]
        pr = pts[idx][1]
        # Barnas retning: ut fra forelderen med arten sin vinkel, spredt rundt.
        parent_dir = (pts[min(idx + 1, n)][0] - pts[max(idx - 1, 0)][0]).normalized()
        axis = perpendicular(parent_dir, rng)
        ang = math.radians(sp.angle[min(level, len(sp.angle) - 1)] + rng.uniform(-12, 12))
        cd = Quaternion(axis, ang) @ parent_dir
        roll = c * golden + rng.uniform(-0.4, 0.4)
        cd = Quaternion(parent_dir, roll) @ cd
        child_len = length * sp.ratio[min(level, len(sp.ratio) - 1)] * rng.uniform(0.75, 1.15)
        if level > 0:
            child_len *= 1.0 - 0.35 * t
        grow(bp, cd, child_len, pr * 0.62, level + 1, sp, rng, splines, tips, 0.8)


def curve_object(name, splines, bevel_res=1, res_u=2):
    cu = bpy.data.curves.new(name, "CURVE")
    cu.dimensions = "3D"
    cu.bevel_depth = 1.0
    cu.bevel_resolution = bevel_res
    cu.use_fill_caps = True
    cu.resolution_u = res_u
    for pts in splines:
        if isinstance(pts, tuple):
            pts = pts[0]
        spl = cu.splines.new("NURBS")
        spl.points.add(len(pts) - 1)
        for i, (p, r) in enumerate(pts):
            spl.points[i].co = (p.x, p.y, p.z, 1.0)
            spl.points[i].radius = r
        spl.use_endpoint_u = True
        spl.order_u = min(4, len(pts))
    ob = bpy.data.objects.new(name, cu)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def to_mesh(ob):
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.object.convert(target="MESH")
    return bpy.context.active_object


def bark_material(kind):
    """Prosedyrisk bark i objektkoordinater (sømløst uansett UV):
    eik grov og furete, bjørk hvit med mørke striper, gran rødbrun og skjellete."""
    mat = bpy.data.materials.new("bark")
    mat.use_nodes = True
    nt = mat.node_tree
    nodes, links = nt.nodes, nt.links
    bsdf = nodes["Principled BSDF"]
    coord = nodes.new("ShaderNodeTexCoord")
    mapping = nodes.new("ShaderNodeMapping")
    links.new(coord.outputs["Object"], mapping.inputs["Vector"])
    ramp = nodes.new("ShaderNodeValToRGB")
    noise = nodes.new("ShaderNodeTexNoise")
    noise.inputs["Detail"].default_value = 9.0
    noise.inputs["Roughness"].default_value = 0.7
    links.new(mapping.outputs["Vector"], noise.inputs["Vector"])
    bump = nodes.new("ShaderNodeBump")
    links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])
    if kind == "birch":
        mapping.inputs["Scale"].default_value = (18.0, 18.0, 2.0)
        noise.inputs["Scale"].default_value = 1.0
        ramp.color_ramp.elements[0].position = 0.42
        ramp.color_ramp.elements[0].color = (0.05, 0.045, 0.04, 1)
        ramp.color_ramp.elements[1].position = 0.5
        ramp.color_ramp.elements[1].color = (0.85, 0.83, 0.78, 1)
        links.new(noise.outputs["Fac"], ramp.inputs["Fac"])
        bump.inputs["Strength"].default_value = 0.25
        bump.inputs["Distance"].default_value = 0.02
        links.new(noise.outputs["Fac"], bump.inputs["Height"])
        bsdf.inputs["Roughness"].default_value = 0.6
    elif kind == "spruce":
        mapping.inputs["Scale"].default_value = (5.0, 5.0, 3.0)
        vor = nodes.new("ShaderNodeTexVoronoi")
        vor.inputs["Scale"].default_value = 9.0
        links.new(mapping.outputs["Vector"], vor.inputs["Vector"])
        noise.inputs["Scale"].default_value = 6.0
        mix = nodes.new("ShaderNodeMix")
        mix.data_type = "FLOAT"
        mix.inputs["Factor"].default_value = 0.5
        links.new(vor.outputs["Distance"], mix.inputs["A"])
        links.new(noise.outputs["Fac"], mix.inputs["B"])
        ramp.color_ramp.elements[0].position = 0.2
        ramp.color_ramp.elements[0].color = (0.12, 0.06, 0.035, 1)
        ramp.color_ramp.elements[1].position = 0.8
        ramp.color_ramp.elements[1].color = (0.36, 0.2, 0.12, 1)
        links.new(mix.outputs["Result"], ramp.inputs["Fac"])
        bump.inputs["Strength"].default_value = 0.7
        bump.inputs["Distance"].default_value = 0.06
        links.new(mix.outputs["Result"], bump.inputs["Height"])
        bsdf.inputs["Roughness"].default_value = 0.9
    else:
        mapping.inputs["Scale"].default_value = (7.0, 7.0, 1.2)
        noise.inputs["Scale"].default_value = 1.4
        ramp.color_ramp.elements[0].position = 0.3
        ramp.color_ramp.elements[0].color = (0.08, 0.055, 0.04, 1)
        ramp.color_ramp.elements[1].position = 0.75
        ramp.color_ramp.elements[1].color = (0.42, 0.34, 0.26, 1)
        links.new(noise.outputs["Fac"], ramp.inputs["Fac"])
        bump.inputs["Strength"].default_value = 0.8
        bump.inputs["Distance"].default_value = 0.08
        links.new(noise.outputs["Fac"], bump.inputs["Height"])
        bsdf.inputs["Roughness"].default_value = 0.88
    # Mose og grønske nederst: grønnere der det er lavt.
    geo = nodes.new("ShaderNodeNewGeometry")
    sep = nodes.new("ShaderNodeSeparateXYZ")
    links.new(geo.outputs["Position"], sep.inputs[0])
    rng_map = nodes.new("ShaderNodeMapRange")
    rng_map.inputs["From Min"].default_value = 0.0
    rng_map.inputs["From Max"].default_value = 1.6
    rng_map.inputs["To Min"].default_value = 0.45
    rng_map.inputs["To Max"].default_value = 0.0
    links.new(sep.outputs["Z"], rng_map.inputs["Value"])
    moss_noise = nodes.new("ShaderNodeTexNoise")
    moss_noise.inputs["Scale"].default_value = 12.0
    mul = nodes.new("ShaderNodeMath")
    mul.operation = "MULTIPLY"
    links.new(rng_map.outputs["Result"], mul.inputs[0])
    links.new(moss_noise.outputs["Fac"], mul.inputs[1])
    moss = nodes.new("ShaderNodeMix")
    moss.data_type = "RGBA"
    links.new(mul.outputs[0], moss.inputs["Factor"])
    links.new(ramp.outputs["Color"], moss.inputs["A"])
    moss.inputs["B"].default_value = (0.18, 0.3, 0.1, 1)
    links.new(moss.outputs["Result"], bsdf.inputs["Base Color"])
    return mat


def bake_bark(ob, kind, name):
    """UV-pakk stammen og bak albedo, normalkart og ruhet til filer; bytt
    materialet til teksturene."""
    ob.data.materials.clear()
    ob.data.materials.append(bark_material(kind))
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=1.2, island_margin=0.004)
    bpy.ops.object.mode_set(mode="OBJECT")
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 12
    scene.render.bake.margin = 6
    mat = ob.data.materials[0]
    nodes = mat.node_tree.nodes

    def bake_to(label, bake_type, colorspace, fmt, **kw):
        img = bpy.data.images.new(label, BAKE, BAKE, alpha=False)
        img.colorspace_settings.name = colorspace
        tex = nodes.new("ShaderNodeTexImage")
        tex.image = img
        nodes.active = tex
        bpy.ops.object.bake(type=bake_type, **kw)
        nodes.remove(tex)
        path = os.path.abspath(f"{OUT}/{name}_{label}.{fmt}")
        img.filepath_raw = path
        img.file_format = "JPEG" if fmt == "jpg" else "PNG"
        img.save()
        return path

    albedo = bake_to("albedo", "DIFFUSE", "sRGB", "jpg", pass_filter={"COLOR"})
    normal = bake_to("normal", "NORMAL", "Non-Color", "png", normal_space="TANGENT")
    rough = bake_to("rough", "ROUGHNESS", "Non-Color", "jpg")
    # Materialet med teksturene.
    tm = bpy.data.materials.new("bark_tex")
    tm.use_nodes = True
    nt = tm.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (1, 1, 1, 1)
    ta = nt.nodes.new("ShaderNodeTexImage")
    ta.image = bpy.data.images.load(albedo)
    nt.links.new(ta.outputs["Color"], bsdf.inputs["Base Color"])
    tn = nt.nodes.new("ShaderNodeTexImage")
    tn.image = bpy.data.images.load(normal)
    tn.image.colorspace_settings.name = "Non-Color"
    nm = nt.nodes.new("ShaderNodeNormalMap")
    nt.links.new(tn.outputs["Color"], nm.inputs["Color"])
    nt.links.new(nm.outputs["Normal"], bsdf.inputs["Normal"])
    tr = nt.nodes.new("ShaderNodeTexImage")
    tr.image = bpy.data.images.load(rough)
    tr.image.colorspace_settings.name = "Non-Color"
    nt.links.new(tr.outputs["Color"], bsdf.inputs["Roughness"])
    ob.data.materials.clear()
    ob.data.materials.append(tm)


def card_mesh(name, cards, texture, mat_name, center, aspect=1.0):
    """Bladkortene: hver (pos, retning, opp, bredde) blir en firkant med
    teksturen; normalene peker ut fra krona, så lyset ligger rundt den."""
    import bmesh
    bm = bmesh.new()
    uv_layer = bm.loops.layers.uv.new("uv")
    normals = []
    for pos, d, up, w in cards:
        right = d.cross(up).normalized() * (w * 0.5)
        along = d.normalized() * (w * aspect * 0.5)
        c = pos
        verts = [bm.verts.new(c - right - along), bm.verts.new(c + right - along), bm.verts.new(c + right + along), bm.verts.new(c - right + along)]
        f = bm.faces.new(verts)
        for loop, uv in zip(f.loops, [(0, 0), (1, 0), (1, 1), (0, 1)]):
            loop[uv_layer].uv = uv
        n = (c - center)
        n = n.normalized() if n.length > 1e-3 else Vector((0, 0, 1))
        normals.extend([n] * 4)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    me.normals_split_custom_set_from_vertices(normals)
    mat = bpy.data.materials.new(mat_name)
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = bpy.data.images.load(os.path.abspath(texture))
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    nt.links.new(tex.outputs["Alpha"], bsdf.inputs["Alpha"])
    bsdf.inputs["Roughness"].default_value = 0.6
    if hasattr(mat, "blend_method"):
        try:
            mat.blend_method = "CLIP"
        except Exception:
            pass
    me.materials.append(mat)
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def export(path):
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=True, export_image_format="AUTO", export_animations=False, export_skins=False, export_cameras=False, export_lights=False)
    tris = sum(len(p.vertices) - 2 for ob in bpy.data.objects if ob.type == "MESH" for p in ob.data.polygons)
    print(f"{path}: {tris} trekanter")


def delete_all():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def broadleaf(name, sp, kind, texture, seed, trunks=1):
    rng = random.Random(seed)
    splines, tips = [], []
    for k in range(trunks):
        base = Vector((0, 0, 0)) if trunks == 1 else Vector((rng.uniform(-0.25, 0.25), rng.uniform(-0.25, 0.25), 0))
        d = Vector((rng.uniform(-0.12, 0.12), rng.uniform(-0.12, 0.12), 1.0)) if trunks == 1 else Vector((rng.uniform(-0.5, 0.5), rng.uniform(-0.5, 0.5), 1.0))
        grow(base, d, sp.trunk_len * rng.uniform(0.9, 1.1), sp.trunk_r, 0, sp, rng, splines, tips, 0.55)
    # Rotfoten: den første stokken får bredere fot.
    splines[0][0][0] = (splines[0][0][0][0], splines[0][0][0][1] * 1.35)
    # Stammen og de grove greinene med flere sider; kvistene ytterst som
    # firkantede rør med færre segmenter, de er gjemt i bladverket.
    thick = [pts for pts, lv in splines if lv < 2]
    thin = [pts for pts, lv in splines if lv >= 2]
    big = sp.trunk_r > 0.1
    parts = [to_mesh(curve_object("trunk", thick, bevel_res=1 if big else 0, res_u=2 if big else 1))]
    if thin:
        parts.append(to_mesh(curve_object("twigs", thin, bevel_res=0, res_u=1)))
    bpy.ops.object.select_all(action="DESELECT")
    for ob in parts:
        ob.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.join()
    trunk = bpy.context.active_object
    if sp.trunk_r > 0.1:
        bake_bark(trunk, kind, name)
    else:
        m = bpy.data.materials.new("stem")
        m.use_nodes = True
        m.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.1, 0.07, 0.05, 1)
        trunk.data.materials.append(m)
    export(f"{OUT}/{name}_trunk.glb")
    delete_all()
    # Bladene: kort rundt hver spiss, litt forskjøvet og tilfeldig vridd.
    cards = []
    center = sum((p for p, _ in tips), Vector()) / max(1, len(tips))
    for pos, d in tips:
        for _ in range(sp.cards):
            q = Quaternion((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))).normalized()
            dirv = q @ Vector((1, 0, 0))
            up = q @ Vector((0, 0, 1))
            off = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * sp.card * 0.35
            cards.append((pos + off, dirv, up, sp.card * rng.uniform(0.8, 1.2)))
    card_mesh("leaves", cards, texture, "leaf", center)
    export(f"{OUT}/{name}_leaves.glb")
    delete_all()


def spruce(name, seed, height=12.0):
    rng = random.Random(seed)
    h = height * rng.uniform(0.9, 1.1)
    # Stammen: rett med litt sving, smalner mot toppen.
    pts = []
    pos = Vector((0, 0, 0))
    d = Vector((rng.uniform(-0.03, 0.03), rng.uniform(-0.03, 0.03), 1.0)).normalized()
    n = 10
    for i in range(n + 1):
        r = 0.2 * (h / 12.0) * (1.0 - 0.92 * i / n) ** 1.2
        pts.append((pos.copy(), max(r, 0.01)))
        d = (d + Vector((rng.gauss(0, 0.02), rng.gauss(0, 0.02), 0))).normalized()
        pos = pos + d * (h / n)
    pts[0] = (pts[0][0], pts[0][1] * 1.4)
    trunk = to_mesh(curve_object("trunk", [pts], bevel_res=2, res_u=2))
    bake_bark(trunk, "spruce", name)
    export(f"{OUT}/{name}_trunk.glb")
    delete_all()
    # Greinene i kranser: tre kort om hver grein, dreid 60 grader om
    # greinaksen og litt tilfeldig tippet, lengre nederst.
    cards = []
    y = h * 0.12
    k = 0
    while y < h - 0.4:
        t = y / h
        count = 8 if t < 0.5 else 6
        for c in range(count):
            a = c / count * 2 * math.pi + k * 0.9 + rng.uniform(-0.25, 0.25)
            ln = (0.4 + (1.0 - t) ** 1.1 * 2.6) * (h / 12.0) * rng.uniform(0.85, 1.1)
            droop = math.radians(rng.uniform(6, 20) + 14 * (1 - t))
            d = Vector((math.cos(a) * math.cos(droop), math.sin(a) * math.cos(droop), -math.sin(droop))).normalized()
            side = Vector((-math.sin(a), math.cos(a), 0))
            start = Vector((0, 0, y)) + d * (ln * 0.5 - 0.05)
            for j in range(3):
                roll = j * math.pi / 3 + rng.uniform(-0.3, 0.3)
                up = Quaternion(d, roll) @ side
                cards.append((start, d, up, ln * 0.7))
        y += 0.4 * (h / 12.0)
        k += 1
    # Toppen: små kort rundt spiret.
    for c in range(5):
        a = c * 1.3
        d = Vector((math.cos(a) * 0.3, math.sin(a) * 0.3, 1.0)).normalized()
        cards.append((Vector((0, 0, h - 0.5)) + d * 0.3, d, Vector((-math.sin(a), math.cos(a), 0)), 0.35))
    card_mesh("needles", cards, f"{OUT}/needles_spruce.png", "needle", Vector((0, 0, h * 0.45)), aspect=1.6)
    export(f"{OUT}/{name}_needles.glb")
    delete_all()


BUILDERS = {
    "oak1": lambda: broadleaf("oak1", OAK, "oak", f"{OUT}/leaves_oak.png", 101),
    "oak2": lambda: broadleaf("oak2", OAK, "oak", f"{OUT}/leaves_oak.png", 202),
    "birch1": lambda: broadleaf("birch1", BIRCH, "birch", f"{OUT}/leaves_birch.png", 303),
    "spruce1": lambda: spruce("spruce1", 404, 12.0),
    "spruce2": lambda: spruce("spruce2", 505, 9.0),
    "bush1": lambda: broadleaf("bush1", BUSH, "bush", f"{OUT}/leaves_bush.png", 606, trunks=4),
    "bush2": lambda: broadleaf("bush2", BUSH, "bush", f"{OUT}/leaves_bush.png", 707, trunks=3),
}

if __name__ == "__main__":
    kinds = [a for a in sys.argv[1:] if a in BUILDERS] or list(BUILDERS)
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        BUILDERS[k]()
