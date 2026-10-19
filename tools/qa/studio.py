"""Studiobilder av én modell alene, med teksturlagene lagt på slik motoren
gjør (`extras.fristad.layer`, uv i meter delt på flisens mål), i faste
vinkler. Til før- og etterbilder: samme vinkler gir samme bilde så lenge
modellen ikke er endret, uansett hva som står rundt den i byen.

    python3.13 tools/qa/studio.py modell.glb UT_PREFIKS [vinkel ...]

En vinkel er `navn:yaw:pitch:avstand:høyde` (grader rundt modellen fra
forsida, grader ned, avstand i ganger modellens største mål, sikte i
brøkdel av høyden over bakken), f.eks. `front:0:8:1.1:0.45`. Uten vinkler:
forfra, hjørnet, siden og ovenfra. Skriver UT_PREFIKS_<navn>.png.
`FRISTAD_STUDIO_SIZE=320x224` gir mindre bilder (katalogen i redigeringen).
"""
import json
import math
import os
import sys

import bpy
from mathutils import Vector

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TEX = os.path.join(ROOT, "assets", "textures")
VIEWS = ["front:0:6:1.15:0.4", "corner:-35:12:1.2:0.4", "side:-90:6:1.15:0.4", "top:-30:40:1.4:0.3"]


def size_of(layer):
    try:
        with open(os.path.join(TEX, layer, "meta.json")) as f:
            return float(json.load(f).get("size_m", 2.0))
    except (OSError, ValueError):
        return 2.0


def dress(mat):
    """Teksturlaget på materialet: bildet etter uv delt på flisens mål, tonet
    med materialfargen, som i `uvmapped` i world.wgsl."""
    role = mat.get("fristad")
    layer = role.get("layer") if hasattr(role, "get") else None
    if not layer:
        return
    path = os.path.join(TEX, layer, "albedo.jpg")
    if not os.path.exists(path):
        return
    nt = mat.node_tree
    bsdf = next((n for n in nt.nodes if n.type == "BSDF_PRINCIPLED"), None)
    if bsdf is None:
        return
    # Med hjørnefarge (okklusjonen) legger glTF-importen grunnfargen i en
    # MIX-node foran BSDF-en; tonen er fargen som ikke er koblet der.
    base = bsdf.inputs["Base Color"]
    tint = tuple(base.default_value)
    if base.is_linked:
        src = base.links[0].from_node
        tint = next((tuple(i.default_value) for i in src.inputs if i.type == "RGBA" and not i.is_linked), (1.0, 1.0, 1.0, 1.0))
    uv = nt.nodes.new("ShaderNodeTexCoord")
    mp = nt.nodes.new("ShaderNodeMapping")
    s = 1.0 / size_of(layer)
    mp.inputs["Scale"].default_value = (s, s, 1.0)
    img = nt.nodes.new("ShaderNodeTexImage")
    img.image = bpy.data.images.load(path, check_existing=True)
    mul = nt.nodes.new("ShaderNodeMix")
    mul.data_type = "RGBA"
    mul.blend_type = "MULTIPLY"
    mul.inputs["Factor"].default_value = 1.0
    mul.inputs[7].default_value = tint
    nt.links.new(uv.outputs["UV"], mp.inputs["Vector"])
    nt.links.new(mp.outputs["Vector"], img.inputs["Vector"])
    nt.links.new(img.outputs["Color"], mul.inputs[6])
    nt.links.new(mul.outputs[2], bsdf.inputs["Base Color"])


def glow(mat):
    """Flammer og glør (`fire…`, `glow…`) lyser som i spillet."""
    if not mat.name.startswith(("fire", "glow", "magic")):
        return
    bsdf = next((n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED"), None)
    if bsdf is None:
        return
    base = bsdf.inputs["Base Color"]
    col = base.default_value
    if base.is_linked:
        src = base.links[0].from_node
        col = next((i.default_value for i in src.inputs if i.type == "RGBA" and not i.is_linked), col)
    bsdf.inputs["Emission Color"].default_value = col
    bsdf.inputs["Emission Strength"].default_value = 4.0


def main(path, prefix, views):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=path)
    # Fjernversjonen (noden `far` i byggesettet for krypten) står oppå den
    # nære: bare den nære vises.
    for o in list(bpy.context.scene.objects):
        if o.name == "far" or o.name.startswith("far."):
            bpy.data.objects.remove(o, do_unlink=True)
    for m in bpy.data.materials:
        if m.use_nodes:
            dress(m)
            glow(m)
    obs = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    pts = [o.matrix_world @ Vector(c) for o in obs for c in o.bound_box]
    # Det som henger eller står høyt (hyller, lykter), sees der det er:
    # kameraet sikter på tingen, ikke på gulvet under (runde 5).
    bottom = max(0.0, min(p.z for p in pts))
    lo = Vector((min(p.x for p in pts), min(p.y for p in pts), bottom))
    hi = Vector((max(p.x for p in pts), max(p.y for p in pts), max(p.z for p in pts)))
    c = (lo + hi) / 2
    big = max(hi.x - lo.x, hi.y - lo.y, hi.z - bottom, 0.35)
    s = bpy.context.scene
    s.render.engine = "CYCLES"
    s.cycles.samples = 12
    s.cycles.device = "CPU"
    s.render.resolution_x, s.render.resolution_y = [int(v) for v in os.environ.get("FRISTAD_STUDIO_SIZE", "800x560").split("x")]
    world = bpy.data.worlds.new("w")
    s.world = world
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[0].default_value = (0.62, 0.72, 0.86, 1)
    world.node_tree.nodes["Background"].inputs[1].default_value = 0.9
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 10))
    sun = bpy.context.active_object
    sun.data.energy = 3.5
    sun.rotation_euler = (math.radians(50), math.radians(10), math.radians(35))
    # Bakken i bakkelinja (origo); det under er grunnmur i jorda.
    bpy.ops.mesh.primitive_plane_add(size=big * 8, location=(c.x, c.y, 0.0))
    g = bpy.context.active_object
    gm = bpy.data.materials.new("ground")
    gm.use_nodes = True
    gm.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.32, 0.3, 0.24, 1)
    g.data.materials.append(gm)
    cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
    s.collection.objects.link(cam)
    cam.data.lens = 32
    s.camera = cam
    for v in views:
        name, yaw, pitch, dist, aim = v.split(":")
        a, p = math.radians(float(yaw)), math.radians(float(pitch))
        r = big * float(dist)
        target = Vector((c.x, c.y, bottom + (hi.z - bottom) * float(aim)))
        # glTF importeres med −y som forsida.
        cam.location = target + Vector((r * math.sin(a) * math.cos(p), -r * math.cos(a) * math.cos(p), r * math.sin(p)))
        cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
        s.render.filepath = f"{prefix}_{name}.png"
        bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], sys.argv[3:] or VIEWS)
