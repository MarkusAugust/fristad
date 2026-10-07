"""Forhåndsvisning av en glTF-modell i Blender (Cycles på CPU):
python3 tools/models/preview.py assets/models/cars/estate.glb shots/estate.png [samples]"""
import math
import sys

import bpy

path, out = sys.argv[1], sys.argv[2]
samples = int(sys.argv[3]) if len(sys.argv) > 3 else 48
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=path)
# Lakken får en farge her, som i motoren.
paint = bpy.data.materials.get("paint")
if paint:
    paint.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.1, 0.25, 0.55, 1.0)
scene = bpy.context.scene
scene.render.engine = "CYCLES"
scene.cycles.samples = samples
scene.cycles.device = "CPU"
scene.render.resolution_x, scene.render.resolution_y = 960, 600
scene.render.filepath = out
# Bakke, sol og himmel.
bpy.ops.mesh.primitive_plane_add(size=40)
ground = bpy.context.active_object
gm = bpy.data.materials.new("ground")
gm.use_nodes = True
gm.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.45, 0.42, 0.38, 1)
gm.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 0.9
ground.data.materials.append(gm)
bpy.ops.object.light_add(type="SUN", location=(4, -3, 8))
sun = bpy.context.active_object
sun.data.energy = 4.0
sun.rotation_euler = (math.radians(50), math.radians(10), math.radians(35))
world = bpy.data.worlds.new("w")
scene.world = world
world.use_nodes = True
bg = world.node_tree.nodes["Background"]
bg.inputs[0].default_value = (0.6, 0.7, 0.85, 1)
bg.inputs[1].default_value = 0.8
bpy.ops.object.camera_add(location=(6.5, -5.5, 2.6))
cam = bpy.context.active_object
cam.rotation_euler = (math.radians(72), 0, math.radians(49))
cam.data.lens = 40
scene.camera = cam
bpy.ops.render.render(write_still=True)
print("skrev", out)
