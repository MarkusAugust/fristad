"""Armene, våpenet og skjoldet sett fra kameraet i førsteperson (72°
synsfelt, 16:9), som kontaktark over et klipp:

    python3.13 tools/models/fp_preview.py ARMER VÅPEN SKJOLD KLIPP UT.png [bilder]

ARMER er `assets/models/arms/arms.glb` o.l.; våpenet og skjoldet festes i
hendene (`hand.R`, `hand.L`) som i spillet. Uten bilder: åtte jevnt over
klippet (ett for `idle`)."""

import math
import os
import sys
import tempfile

import bpy
from mathutils import Matrix


def load(path):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)
    return [o for o in bpy.data.objects if o not in before]


def main(arms, weapon, shield, clip, out, count=8, single=False):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    objs = load(arms)
    rig = next(o for o in objs if o.type == "ARMATURE")
    acts = [a for a in bpy.data.actions if a.name.split("_")[0] == clip or a.name == clip or a.name.startswith(clip + "_")]
    act = next((a for a in acts if a.name == clip), acts[0] if acts else None)
    for path, bone in ((weapon, "hand.R"), (shield, "hand.L")):
        if path == "-":
            continue
        for o in load(path):
            if o.type != "MESH":
                continue
            # Festet i leddet slik det står i hvileposen (som skinnmatrisen i spillet).
            mw = o.matrix_world.copy()
            o.parent = rig
            o.parent_type = "BONE"
            o.parent_bone = bone
            pb = rig.pose.bones[bone]
            o.matrix_parent_inverse = (rig.matrix_world @ pb.matrix @ Matrix.Translation((0, pb.length, 0))).inverted()
            o.matrix_basis = mw
    rig.animation_data_create()
    for t in list(rig.animation_data.nla_tracks):
        rig.animation_data.nla_tracks.remove(t)
    rig.animation_data.action = act
    if act is not None and hasattr(act, "slots") and len(act.slots):
        rig.animation_data.action_slot = act.slots[0]
    f0, f1 = (int(v) for v in act.frame_range) if act else (0, 0)
    if isinstance(count, list):
        frames = count
    else:
        frames = [f0] if clip == "idle" else [round(f0 + (f1 - f0) * i / max(1, count - 1)) for i in range(count)]
    cam = bpy.data.cameras.new("cam")
    cam.sensor_fit = "VERTICAL"
    cam.angle_y = math.radians(72)
    cam.clip_start = 0.05
    co = bpy.data.objects.new("cam", cam)
    co.rotation_euler = (math.radians(90), 0, 0)
    bpy.context.scene.collection.objects.link(co)
    sc = bpy.context.scene
    sc.camera = co
    sc.render.engine = "BLENDER_WORKBENCH"
    sc.display.shading.light = "STUDIO"
    sc.display.shading.color_type = "TEXTURE"
    sc.render.resolution_x, sc.render.resolution_y = 960, 540
    world = bpy.data.worlds.new("w")
    sc.world = world
    sc.display.shading.background_type = "VIEWPORT"
    sc.display.shading.background_color = (0.16, 0.17, 0.19)
    tmp = tempfile.mkdtemp()
    shots = []
    for f in frames:
        sc.frame_set(f)
        p = os.path.join(tmp, f"{f:03d}.png")
        sc.render.filepath = p
        bpy.ops.render.render(write_still=True)
        shots.append((f, p))
    from PIL import Image, ImageDraw

    ims = [Image.open(p).convert("RGB") for _, p in shots]
    if single:
        ims[0].save(out)
        print(f"{clip} {frames[0]} -> {out}")
        return
    cols = min(4, len(ims))
    rows = (len(ims) + cols - 1) // cols
    w, h = ims[0].size
    sheet = Image.new("RGB", (w * cols // 2, h * rows // 2), (20, 20, 20))
    for i, (im, (f, _)) in enumerate(zip(ims, shots)):
        im = im.resize((w // 2, h // 2))
        ImageDraw.Draw(im).text((8, 6), f"{clip} {f}", fill=(255, 255, 255))
        sheet.paste(im, ((i % cols) * w // 2, (i // cols) * h // 2))
    sheet.save(out)
    print(f"{clip}: bilder {frames} -> {out}")


if __name__ == "__main__":
    a = sys.argv[1:]
    # Et sjette argument «=N» gir ett helt bilde av bilde N.
    if len(a) > 5 and a[5].startswith("="):
        main(a[0], a[1], a[2], a[3], a[4], [int(a[5][1:])], single=True)
    else:
        main(a[0], a[1], a[2], a[3], a[4], int(a[5]) if len(a) > 5 else 8)
