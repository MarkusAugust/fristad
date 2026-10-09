"""Kontaktark for et klipp: rendrer en rigget modell i flere bilder av et
klipp, side om side, med Workbench (raskt), til å se på bevegelsene:

    python3 tools/models/clip_sheet.py modell.glb klipp ut.png [bilder] [side|front|top]

Modellen ses fra siden (fronten mot høyre), og en strek i bakkehøyde viser
om føttene glir eller går gjennom gulvet."""

import math
import os
import sys
import tempfile

import bpy
from mathutils import Vector

path, clip, out = sys.argv[1], sys.argv[2], sys.argv[3]
n = int(sys.argv[4]) if len(sys.argv) > 4 else 8
view = sys.argv[5] if len(sys.argv) > 5 else "side"

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=path)
scene = bpy.context.scene
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
ad = arm.animation_data or arm.animation_data_create()
for t in list(ad.nla_tracks):
    t.mute = True
act = bpy.data.actions.get(clip) or next((a for a in bpy.data.actions if a.name.endswith(clip) or a.name.startswith(clip)), None)
if act is None:
    raise SystemExit(f"fant ikke klippet {clip} blant {[a.name for a in bpy.data.actions]}")
ad.action = act
if hasattr(act, "slots") and len(act.slots) > 0:
    ad.action_slot = act.slots[0]
start, end = act.frame_range
scene.render.fps = 24

# Størrelsen fra hvileposen.
scene.frame_set(int(start))
# Bare meshene som henger på skjelettet (ikke beinformene).
for o in bpy.data.objects:
    if o.type == "MESH" and o.parent is None:
        o.hide_render = True
meshes = [o for o in bpy.data.objects if o.type == "MESH" and o.parent is not None]
lo = Vector((1e9, 1e9, 1e9))
hi = Vector((-1e9, -1e9, -1e9))
for o in meshes:
    for c in o.bound_box:
        w = o.matrix_world @ Vector(c)
        lo = Vector(map(min, lo, w))
        hi = Vector(map(max, hi, w))
size = max((hi - lo).length, 0.5)
mid = (lo + hi) * 0.5

# Bakken: en tynn plate.
bpy.ops.mesh.primitive_plane_add(size=size * 4, location=(mid.x, mid.y, 0.0))
floor = bpy.context.active_object
mat = bpy.data.materials.new("floor")
mat.diffuse_color = (0.35, 0.35, 0.38, 1)
floor.data.materials.append(mat)

cam_data = bpy.data.cameras.new("cam")
cam_data.type = "ORTHO"
cam_data.ortho_scale = size * 1.05
cam = bpy.data.objects.new("cam", cam_data)
scene.collection.objects.link(cam)
scene.camera = cam
d = size * 3
if view == "front":
    cam.location = (mid.x + d, mid.y, mid.z + size * 0.05)
elif view == "top":
    cam.location = (mid.x, mid.y, mid.z + d)
else:
    cam.location = (mid.x, mid.y - d, mid.z + size * 0.05)
direction = Vector(mid) - cam.location
cam.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()

scene.render.engine = "BLENDER_WORKBENCH"
scene.display.shading.light = "STUDIO"
scene.display.shading.color_type = "SINGLE"
scene.display.shading.single_color = (0.75, 0.72, 0.68)
scene.display.shading.show_cavity = True
scene.display.shading.show_shadows = True
scene.render.resolution_x = 360
scene.render.resolution_y = 360
scene.render.film_transparent = False

tmp = tempfile.mkdtemp()
frames = [round(start + (end - start) * k / max(1, n - 1)) for k in range(n)] if n > 1 else [int(start)]
files = []
for k, f in enumerate(frames):
    scene.frame_set(int(f))
    p = os.path.join(tmp, f"{k:02d}.png")
    scene.render.filepath = p
    bpy.ops.render.render(write_still=True)
    files.append((p, f))

# Setter bildene sammen i en rad med bildenummer.
imgs = [bpy.data.images.load(p) for p, _ in files]
w, h = imgs[0].size
cols = min(n, 6)
rows = math.ceil(n / cols)
sheet = bpy.data.images.new("sheet", w * cols, h * rows)
px = [0.0] * (w * cols * h * rows * 4)
for k, img in enumerate(imgs):
    cx, cy = k % cols, rows - 1 - k // cols
    src = list(img.pixels)
    for y in range(h):
        row = ((cy * h + y) * w * cols + cx * w) * 4
        px[row:row + w * 4] = src[y * w * 4:(y + 1) * w * 4]
sheet.pixels = px
sheet.filepath_raw = out
sheet.file_format = "PNG"
sheet.save()
print(f"{clip}: bilder {[f for _, f in files]} -> {out}")
