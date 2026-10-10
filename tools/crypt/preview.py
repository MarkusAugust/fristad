"""Forhåndsvisning av kryptmodellene med de malte teksturlagene (Cycles på
CPU). `tools/models/preview.py` viser `layer_`-materialene som flate
farger; her får de bildene fra `assets/textures/<lag>/` (albedo og normal,
uv i meter delt på flisstørrelsen), okklusjonen fra hjørnefargen, og ild,
magi og korrupsjon gløder.

    python3 tools/crypt/preview.py ut.png modell.glb [flere.glb ...] [valg]

Valg:
    --json nivå.json        lysene fra nivåfila (og bitene i `pieces` om ingen glb er gitt)
    --eye X,Y,Z --yaw D [--pitch D]   kamera inne i nivået (motorens akser:
                            yaw 0 ser mot nord (−z), 90 mot vest)
    --lens MM               brennvidde (standard 16)
    --top                   kart ovenfra (ortografisk), taket skåret bort
    --size 1280x720 --samples 48 --ambient 0.04
Uten --eye: rundt modellen som i tools/models/preview.py, med sol og gulv."""

import json
import math
import os
import sys

import bpy
from mathutils import Vector

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TEX = os.path.join(ROOT, "assets", "textures")

args = sys.argv[1:]
out = args[0]
glbs = [a for a in args[1:] if a.endswith(".glb")]


def opt(name, default=None):
    return args[args.index(name) + 1] if name in args else default


level = json.load(open(opt("--json"))) if opt("--json") else None
if level and not glbs:
    d = os.path.dirname(opt("--json"))
    glbs = [os.path.join(d, f"{p}.glb") for p in level["pieces"]]

bpy.ops.wm.read_factory_settings(use_empty=True)
for g in glbs:
    bpy.ops.import_scene.gltf(filepath=g)
scene = bpy.context.scene
scene.render.engine = "CYCLES"
scene.cycles.device = "CPU"
scene.cycles.samples = int(opt("--samples", 48))
scene.cycles.use_denoising = True
w, h = (int(v) for v in opt("--size", "1280x720").split("x"))
scene.render.resolution_x, scene.render.resolution_y = w, h
scene.render.filepath = out
scene.view_settings.view_transform = "AgX"
scene.view_settings.look = "AgX - Medium High Contrast"
world = bpy.data.worlds.new("w")
scene.world = world
world.use_nodes = True
amb = float(opt("--ambient", 0.035))
world.node_tree.nodes["Background"].inputs[0].default_value = (0.42, 0.45, 0.75, 1.0)
world.node_tree.nodes["Background"].inputs[1].default_value = amb

images = {}


def image(layer, kind):
    key = (layer, kind)
    if key not in images:
        for ext in ("jpg", "png"):
            p = os.path.join(TEX, layer, f"{kind}.{ext}")
            if os.path.exists(p):
                im = bpy.data.images.load(p)
                if kind != "albedo":
                    im.colorspace_settings.name = "Non-Color"
                images[key] = im
                break
        else:
            images[key] = None
    return images[key]


def size_m(layer):
    try:
        return json.load(open(os.path.join(TEX, layer, "meta.json")))["size_m"]
    except OSError:
        return 2.0


def setup(mat):
    name = mat.name
    nt = mat.node_tree
    bsdf = nt.nodes.get("Principled BSDF")
    if bsdf is None:
        return
    tint = tuple(bsdf.inputs["Base Color"].default_value)
    links = nt.links
    ao = nt.nodes.new("ShaderNodeVertexColor")
    ao.layer_name = ""
    if name.startswith("fire"):
        bsdf.inputs["Emission Color"].default_value = tint
        bsdf.inputs["Emission Strength"].default_value = 12.0
        return
    if name.startswith("magic"):
        bsdf.inputs["Emission Color"].default_value = tint
        bsdf.inputs["Emission Strength"].default_value = 8.0
        return
    if name.startswith("water"):
        bsdf.inputs["Base Color"].default_value = (0.02, 0.04, 0.05, 1)
        bsdf.inputs["Roughness"].default_value = 0.03
        return
    if not name.startswith("layer_"):
        return
    layer = name[len("layer_"):].split(".")[0]
    alb = image(layer, "albedo")
    if alb is None:
        return
    s = size_m(layer)
    tc = nt.nodes.new("ShaderNodeTexCoord")
    mp = nt.nodes.new("ShaderNodeMapping")
    mp.inputs["Scale"].default_value = (1 / s, 1 / s, 1)
    links.new(tc.outputs["UV"], mp.inputs["Vector"])
    ti = nt.nodes.new("ShaderNodeTexImage")
    ti.image = alb
    links.new(mp.outputs["Vector"], ti.inputs["Vector"])
    mix = nt.nodes.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    mix.blend_type = "MULTIPLY"
    mix.inputs["Factor"].default_value = 1.0
    links.new(ti.outputs["Color"], mix.inputs[6])
    mix.inputs[7].default_value = tint
    mix2 = nt.nodes.new("ShaderNodeMix")
    mix2.data_type = "RGBA"
    mix2.blend_type = "MULTIPLY"
    mix2.inputs["Factor"].default_value = 1.0
    links.new(mix.outputs[2], mix2.inputs[6])
    links.new(ao.outputs["Color"], mix2.inputs[7])
    links.new(mix2.outputs[2], bsdf.inputs["Base Color"])
    nm = image(layer, "normal")
    if nm is not None:
        tn = nt.nodes.new("ShaderNodeTexImage")
        tn.image = nm
        links.new(mp.outputs["Vector"], tn.inputs["Vector"])
        nmap = nt.nodes.new("ShaderNodeNormalMap")
        nmap.inputs["Strength"].default_value = 0.5
        links.new(tn.outputs["Color"], nmap.inputs["Color"])
        links.new(nmap.outputs["Normal"], bsdf.inputs["Normal"])
    bsdf.inputs["Metallic"].default_value = 0.0
    if ".magic" in name:
        # De lysende sømmene: det mettede i bildet gløder.
        hsv = nt.nodes.new("ShaderNodeSeparateColor")
        hsv.mode = "HSV"
        links.new(ti.outputs["Color"], hsv.inputs["Color"])
        ramp = nt.nodes.new("ShaderNodeMapRange")
        ramp.inputs["From Min"].default_value = 0.25
        ramp.inputs["From Max"].default_value = 0.6
        links.new(hsv.outputs[1], ramp.inputs["Value"])
        links.new(ti.outputs["Color"], bsdf.inputs["Emission Color"])
        mulnode = nt.nodes.new("ShaderNodeMath")
        mulnode.operation = "MULTIPLY"
        mulnode.inputs[1].default_value = 6.0
        links.new(ramp.outputs["Result"], mulnode.inputs[0])
        links.new(mulnode.outputs[0], bsdf.inputs["Emission Strength"])


for m in bpy.data.materials:
    if m.use_nodes:
        setup(m)

B = lambda p: Vector((p[0], -p[2], p[1]))
cam_data = bpy.data.cameras.new("cam")
cam = bpy.data.objects.new("cam", cam_data)
scene.collection.objects.link(cam)
scene.camera = cam

if level:
    for li in level["lights"]:
        ld = bpy.data.lights.new("l", "POINT")
        c = li["color"]
        k = max(c)
        ld.color = (c[0] / k, c[1] / k, c[2] / k)
        ld.energy = 55.0 * k * (li["range"] / 7.0) ** 2
        ld.shadow_soft_size = 0.08
        ld.use_custom_distance = True
        ld.cutoff_distance = li["range"] * 1.5
        o = bpy.data.objects.new("l", ld)
        o.location = B(li["pos"])
        scene.collection.objects.link(o)

if "--top" in args:
    lo = Vector((1e9, 1e9, 1e9))
    hi = -lo
    for o in scene.objects:
        if o.type == "MESH":
            for c in o.bound_box:
                v = o.matrix_world @ Vector(c)
                lo = Vector(map(min, lo, v))
                hi = Vector(map(max, hi, v))
    cut = float(opt("--cut", 2.6))
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = max(hi.x - lo.x, hi.y - lo.y) * 1.04
    scene.render.resolution_x = int(w)
    scene.render.resolution_y = int(w * (hi.y - lo.y) / max(hi.x - lo.x, 1e-3))
    cam.location = ((lo.x + hi.x) / 2, (lo.y + hi.y) / 2, 200)
    cam_data.clip_start = 200 - float(opt("--top-y", 99))
    cam_data.clip_end = 400
    sun = bpy.data.lights.new("sun", "SUN")
    sun.energy = 2.5
    so = bpy.data.objects.new("sun", sun)
    so.rotation_euler = (0.3, 0.2, 0.5)
    scene.collection.objects.link(so)
elif opt("--eye"):
    x, y, z = (float(v) for v in opt("--eye").split(","))
    yaw = float(opt("--yaw", 0))
    pitch = float(opt("--pitch", 0))
    cam.location = B((x, y, z))
    cam.rotation_euler = (math.radians(90 + pitch), 0, math.radians(yaw))
    cam_data.lens = float(opt("--lens", 16))
    cam_data.clip_start = 0.05
    fill = float(opt("--fill", 0))
    if fill > 0:
        ld = bpy.data.lights.new("fill", "POINT")
        ld.energy = fill
        ld.color = (0.7, 0.75, 1.0)
        o = bpy.data.objects.new("fill", ld)
        o.location = B((x, y + 0.4, z))
        scene.collection.objects.link(o)
else:
    lo = Vector((1e9, 1e9, 1e9))
    hi = -lo
    for o in scene.objects:
        if o.type == "MESH":
            for c in o.bound_box:
                v = o.matrix_world @ Vector(c)
                lo = Vector(map(min, lo, v))
                hi = Vector(map(max, hi, v))
    ctr = (lo + hi) / 2
    size = (hi - lo).length
    bpy.ops.mesh.primitive_plane_add(size=size * 4, location=(ctr.x, ctr.y, lo.z - 0.001))
    gm = bpy.data.materials.new("ground")
    gm.use_nodes = True
    gm.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.08, 0.08, 0.1, 1)
    bpy.context.active_object.data.materials.append(gm)
    sun = bpy.data.lights.new("sun", "SUN")
    sun.energy = 3.0
    sun.color = (1.0, 0.9, 0.8)
    so = bpy.data.objects.new("sun", sun)
    so.rotation_euler = (math.radians(50), 0, math.radians(float(opt("--sun", 30))))
    scene.collection.objects.link(so)
    world.node_tree.nodes["Background"].inputs[1].default_value = 0.6
    yaw = math.radians(float(opt("--yaw", 35)))
    el = math.radians(float(opt("--pitch", 25)))
    dist = size * float(opt("--dist", 1.1))
    cam.location = ctr + Vector((math.sin(yaw) * math.cos(el), -math.cos(yaw) * math.cos(el), math.sin(el))) * dist
    d = ctr - cam.location
    cam.rotation_euler = d.to_track_quat("-Z", "Y").to_euler()
    cam_data.lens = float(opt("--lens", 35))

bpy.ops.render.render(write_still=True)
print("skrev", out)
