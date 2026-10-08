"""Forhåndsvisning av en kjellermodell innenfra (Cycles på CPU), med lysene
fra json-fila og et svakt fyllys ved kameraet:

    python3 tools/models/dng_preview.py assets/models/dungeon/dng_hall.glb ut.png X,Y,Z YAW [PITCH] [samples]

Posisjonen er i motorens akser (x øst, y opp, z sør) i modellens rom; yaw 0
ser mot nord (−z), 90 mot vest. Med posisjonen «top» rendres et kart
ovenfra med hvelvet skåret bort."""
import json
import math
import sys

import bpy
from mathutils import Euler, Vector

path, out, where = sys.argv[1], sys.argv[2], sys.argv[3]
yaw = float(sys.argv[4]) if len(sys.argv) > 4 else 0.0
pitch = float(sys.argv[5]) if len(sys.argv) > 5 else 0.0
samples = int(sys.argv[6]) if len(sys.argv) > 6 else 32
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=path)
info = json.load(open(path.replace(".glb", ".json")))
scene = bpy.context.scene
scene.render.engine = "CYCLES"
scene.cycles.samples = samples
scene.cycles.device = "CPU"
scene.render.resolution_x, scene.render.resolution_y = 960, 600
scene.render.filepath = out
world = bpy.data.worlds.new("w")
scene.world = world
world.use_nodes = True
world.node_tree.nodes["Background"].inputs[1].default_value = 0.0
B = lambda p: Vector((p[0], -p[2], p[1]))
for p, color, intensity, radius in info["lights"]:
    bpy.ops.object.light_add(type="POINT", location=B(p))
    li = bpy.context.active_object
    li.data.color = color
    li.data.energy = intensity * 60
    li.data.shadow_soft_size = 0.1
bpy.ops.object.camera_add()
cam = bpy.context.active_object
scene.camera = cam
if where == "top":
    w, d = info["w"] * 4.0, info["d"] * 4.0
    cam.data.type = "ORTHO"
    cam.data.ortho_scale = max(w, d) * 1.05
    cam.location = (w / 2, -d / 2, 30)
    cam.rotation_euler = (0, 0, 0)
    cam.data.clip_start = 30 - 2.6
    cam.data.clip_end = 60
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 20))
    bpy.context.active_object.data.energy = 3.0
    bpy.context.active_object.data.use_shadow = False
else:
    x, y, z = (float(v) for v in where.split(","))
    cam.location = B((x, y, z))
    cam.rotation_euler = Euler((math.radians(90 + pitch), 0, math.radians(yaw)))
    cam.data.lens = 18
    bpy.ops.object.light_add(type="POINT", location=B((x, y + 0.3, z)))
    bpy.context.active_object.data.energy = 40
bpy.ops.render.render(write_still=True)
print("skrev", out)
