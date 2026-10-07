"""Rendrer en rigget modell i en gitt pose, til å sjekke klippene:

    python3 tools/models/pose_preview.py modell.glb klipp bilde ut.png [first|side]

`first` setter kameraet i origo og ser langs +Y (kamerarommet til armene),
`side` ser på modellen fra skrått forfra. Cycles på CPU med få prøver."""

import math
import sys

import bpy
from mathutils import Vector

path, clip, frame, out = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
mode = sys.argv[5] if len(sys.argv) > 5 else "side"

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=path)
scene = bpy.context.scene
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
ad = arm.animation_data or arm.animation_data_create()
# Importen lager en action per klipp (navnet kan ha prefiks); den valgte
# settes som aktiv action, NLA-sporene dempes.
for t in list(ad.nla_tracks):
    t.mute = True
names = [a.name for a in bpy.data.actions]
act = bpy.data.actions.get(clip) or next((a for a in bpy.data.actions if a.name.endswith(clip) or a.name.startswith(clip)), None)
if act is None:
    raise SystemExit(f"fant ikke klippet {clip} blant {names}")
ad.action = act
if hasattr(act, "slots") and len(act.slots) > 0:
    ad.action_slot = act.slots[0]
scene.frame_set(frame)

cam_data = bpy.data.cameras.new("cam")
cam = bpy.data.objects.new("cam", cam_data)
scene.collection.objects.link(cam)
scene.camera = cam
if mode == "first":
    cam.location = (0, 0, 0)
    cam.rotation_euler = (math.pi / 2, 0, 0)
    cam_data.angle = math.radians(80)
else:
    lo = Vector((1e9,) * 3)
    hi = Vector((-1e9,) * 3)
    for o in bpy.data.objects:
        if o.type == "MESH":
            for c in o.bound_box:
                w = o.matrix_world @ Vector(c)
                lo = Vector(map(min, lo, w))
                hi = Vector(map(max, hi, w))
    c = (lo + hi) / 2
    r = (hi - lo).length
    cam.location = c + Vector((r * 1.2, -r * 1.1, r * 0.6))
    direction = c - cam.location
    cam.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
    cam_data.angle = math.radians(50)

sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
sun.rotation_euler = (math.radians(50), math.radians(20), math.radians(30))
sun.data.energy = 3.0
scene.collection.objects.link(sun)
world = bpy.data.worlds.new("w")
scene.world = world
world.use_nodes = True
world.node_tree.nodes["Background"].inputs[0].default_value = (0.6, 0.65, 0.7, 1)
world.node_tree.nodes["Background"].inputs[1].default_value = 0.8
# Cycles på CPU (Eevee trenger en GPU); få prøver holder til en sjekk.
scene.render.engine = "CYCLES"
scene.cycles.device = "CPU"
scene.cycles.samples = 8
scene.cycles.use_denoising = False
scene.render.resolution_x = 960
scene.render.resolution_y = 600
scene.render.filepath = out
bpy.ops.render.render(write_still=True)
print(f"skrev {out} ({clip} bilde {frame})")
