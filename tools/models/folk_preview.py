"""Godkjenningsarket for en innbygger: forbildet øverst, og under det
figuren rendret i hvileposen forfra, skrått forfra, fra siden og bakfra
(samme rekkefølge som arket), og en rad med gangen.

    python3.13 tools/models/folk_preview.py assets/models/npcs/npc_smith.glb smith UT.png

Kameraene er ortografiske og står i høyde med figuren, så bildet kan
sammenlignes rett med arket. Cycles på CPU med få prøver."""

import math
import os
import sys

import bpy
from mathutils import Vector

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
path, name, out = sys.argv[1], sys.argv[2], sys.argv[3]
tmp = out.rsplit(".", 1)[0] + "_parts"
os.makedirs(tmp, exist_ok=True)

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=path)
scene = bpy.context.scene
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
ad = arm.animation_data or arm.animation_data_create()
for t in list(ad.nla_tracks):
    t.mute = True


def pose(clip, frame):
    act = bpy.data.actions.get(clip) or next(a for a in bpy.data.actions if a.name.startswith(clip))
    ad.action = act
    if hasattr(act, "slots") and len(act.slots) > 0:
        ad.action_slot = act.slots[0]
    scene.frame_set(frame)
    return act


def bounds():
    lo, hi = Vector((1e9,) * 3), Vector((-1e9,) * 3)
    deps = bpy.context.evaluated_depsgraph_get()
    for o in bpy.data.objects:
        if o.type == "MESH":
            ev = o.evaluated_get(deps)
            for v in ev.to_mesh().vertices:
                w = o.matrix_world @ v.co
                lo, hi = Vector(map(min, lo, w)), Vector(map(max, hi, w))
            ev.to_mesh_clear()
    return lo, hi


cam_data = bpy.data.cameras.new("cam")
cam_data.type = "ORTHO"
cam = bpy.data.objects.new("cam", cam_data)
scene.collection.objects.link(cam)
scene.camera = cam
sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
sun.data.energy = 2.2
scene.collection.objects.link(sun)
world = bpy.data.worlds.new("w")
scene.world = world
world.use_nodes = True
world.node_tree.nodes["Background"].inputs[0].default_value = (0.5, 0.5, 0.5, 1)
world.node_tree.nodes["Background"].inputs[1].default_value = 1.1
scene.render.engine = "CYCLES"
scene.cycles.device = "CPU"
scene.cycles.samples = 12
scene.cycles.use_denoising = False
scene.render.film_transparent = False
scene.render.resolution_x = 420
scene.render.resolution_y = 760


def shot(yaw, file, height, center=None, scale=None):
    """Kameraet `yaw` grader rundt figuren fra forsida (+X), mot venstre;
    `center` og `scale` gir et nærbilde (høyden på midten og bredden)."""
    a = math.radians(yaw)
    d = Vector((math.cos(a), math.sin(a), 0.0))
    c = Vector((0.0, 0.0, center if center is not None else height * 0.5))
    cam.location = c + d * 6.0
    cam.rotation_euler = (-d).to_track_quat("-Z", "Y").to_euler()
    cam_data.ortho_scale = scale or height * 1.12
    sun.rotation_euler = (d + Vector((0.3, 0.3, 1.2))).to_track_quat("Z", "Y").to_euler()
    scene.render.filepath = file
    bpy.ops.render.render(write_still=True)


pose("idle", 0)
lo, hi = bounds()
height = hi.z
files = []
for k, yaw in enumerate((0, -40, -90, 180)):
    f = f"{tmp}/rest_{k}.png"
    shot(yaw, f, height)
    files.append(f)
act = pose("walk", 0)
n = int(act.frame_range[1])
walk = []
for k in range(6):
    f = f"{tmp}/walk_{k}.png"
    pose("walk", int(k * n / 6))
    shot(-90, f, height)
    walk.append(f)

# Nærbilder av hodet forfra, skrått og fra siden (ansiktet skal være skarpt).
pose("idle", 0)
scene.render.resolution_x = 560
scene.render.resolution_y = 560
heads = []
for k, yaw in enumerate((0, -40, -90)):
    f = f"{tmp}/head_{k}.png"
    shot(yaw, f, height, center=height * 0.9, scale=height * 0.22)
    heads.append(f)

from PIL import Image, ImageDraw  # noqa: E402

ref = Image.open(os.path.join(ROOT, "assets", "textures", "people", "ref", f"{name}_body.jpeg")).convert("RGB")
W = 1680
ref = ref.resize((W, int(ref.height * W / ref.width)))
rest = [Image.open(f).convert("RGB") for f in files]
rw = W // 4
rest = [im.resize((rw, int(im.height * rw / im.width))) for im in rest]
wk = [Image.open(f).convert("RGB") for f in walk]
ww = W // 6
wk = [im.resize((ww, int(im.height * ww / im.width))) for im in wk]
hd = [Image.open(f).convert("RGB") for f in heads]
hw = W // 3
hd = [im.resize((hw, hw)) for im in hd]
H = ref.height + rest[0].height + wk[0].height + hw
sheet = Image.new("RGB", (W, H), (40, 40, 40))
sheet.paste(ref, (0, 0))
for i, im in enumerate(rest):
    sheet.paste(im, (i * rw, ref.height))
for i, im in enumerate(wk):
    sheet.paste(im, (i * ww, ref.height + rest[0].height))
for i, im in enumerate(hd):
    sheet.paste(im, (i * hw, ref.height + rest[0].height + wk[0].height))
ImageDraw.Draw(sheet).text((8, ref.height + 4), "forfra / skrått / siden / bakfra", fill=(255, 255, 255))
sheet.save(out, quality=85)
print(f"skrev {out}")
