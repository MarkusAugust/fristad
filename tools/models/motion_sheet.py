"""Kontaktark og animerte GIF-er av klippene til en rigget figur, rendret
med Workbench (trenger libegl1). Figuren flyttes gjennom verden slik
spillet flytter den (gangen med farten målt i klippet, stoppet bremset
over DUTY av gangsyklusen, snuingen dreid TURN_DEG per klipp), over et
rutemønster på bakken, så en fot som glir, synes med en gang.

    python3.13 tools/models/motion_sheet.py modell.glb klipp ut_prefiks [--views side,front] [--frames N] [--size 360] [--hand R]

Skriver `<ut_prefiks>_<klipp>.png` (kontaktark: én rad per visning) og
`<ut_prefiks>_<klipp>.gif` (visningene side om side, i sanntid).
`--hand R` gir et nærbilde av hånda som følger den (grepet og redskapet).
Skriver også farten målt i gangen og glidningen til føttene."""

import math
import os
import sys
import tempfile

import bpy
from mathutils import Euler, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from folk_motion import DUTY, TURN_DEG  # noqa: E402

FPS = 24


def args():
    a = sys.argv[1:]
    opts = {"step": "1", "views": "side,front", "frames": "0", "size": "360", "hand": "", "gif": "1", "tex": "1"}
    pos = []
    i = 0
    while i < len(a):
        if a[i].startswith("--"):
            opts[a[i][2:]] = a[i + 1]
            i += 2
        else:
            pos.append(a[i])
            i += 1
    return pos, opts


def load(path):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=os.path.abspath(path))
    arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
    for o in bpy.data.objects:
        if o.type == "MESH" and o.parent is None:
            o.hide_render = True
    ad = arm.animation_data or arm.animation_data_create()
    for t in list(ad.nla_tracks):
        t.mute = True
    return arm


def use(arm, name):
    act = bpy.data.actions.get(name) or next((a for a in bpy.data.actions if a.name.split("_")[0] == name or a.name.startswith(name + ".")), None)
    if act is None:
        raise SystemExit(f"fant ikke {name} blant {[a.name for a in bpy.data.actions]}")
    ad = arm.animation_data
    ad.action = act
    if hasattr(act, "slots") and len(act.slots) > 0:
        ad.action_slot = act.slots[0]
    return act


def ankles(arm, f):
    bpy.context.scene.frame_set(f)
    out = {}
    for s in ("L", "R"):
        pb = arm.pose.bones[f"foot.{s}"]
        out[s] = arm.matrix_world @ pb.head
    return out


def walk_speed(arm):
    """Farten i gangen slik spillet måler den: foten som står flat (ankelen
    nær sitt laveste) glir bakover; medianen av farten der."""
    act = use(arm, "walk")
    a0, a1 = int(act.frame_range[0]), int(act.frame_range[1])
    arm.location = (0, 0, 0)
    arm.rotation_euler = (0, 0, 0)
    samples = [ankles(arm, f) for f in range(a0, a1 + 1)]
    speeds = []
    for s in ("L", "R"):
        zmin = min(p[s].z for p in samples)
        for p, q in zip(samples, samples[1:]):
            if p[s].z < zmin + 0.004 and q[s].z < zmin + 0.004:
                d = q[s] - p[s]
                speeds.append(math.hypot(d.x, d.y) * FPS)
    speeds.sort()
    return speeds[len(speeds) // 2] if speeds else 0.0, (a1 - a0) / FPS


def mover(clip, v0, Tc, dur):
    """(x, yaw) for figuren ved tida t, slik spillet flytter den."""
    if clip == "walk":
        return lambda t: (v0 * t, 0.0)
    if clip.startswith("stop"):
        Td = DUTY * Tc
        return lambda t: (v0 * (t - t * t / (2 * Td)) if t < Td else v0 * Td / 2, 0.0)
    if clip == "turn_l":
        return lambda t: (0.0, TURN_DEG * t / dur)
    if clip == "turn_r":
        return lambda t: (0.0, -TURN_DEG * t / dur)
    return lambda t: (0.0, 0.0)


def checker(name, n=64):
    img = bpy.data.images.new(name, n, n)
    px = []
    for y in range(n):
        for x in range(n):
            c = 0.5 if ((x // 8) + (y // 8)) % 2 else 0.3
            px += [c, c, c * 1.05, 1.0]
    img.pixels = px
    return img


def ground(size):
    bpy.ops.mesh.primitive_plane_add(size=size, location=(0, 0, 0))
    pl = bpy.context.active_object
    mat = bpy.data.materials.new("ground")
    mat.use_nodes = True
    nt = mat.node_tree
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = checker("chk")
    tex.interpolation = "Closest"
    coord = nt.nodes.new("ShaderNodeTexCoord")
    mp = nt.nodes.new("ShaderNodeMapping")
    mp.inputs["Scale"].default_value = (size / 0.8, size / 0.8, 1)  # ruter på 10 cm
    nt.links.new(coord.outputs["UV"], mp.inputs["Vector"])
    nt.links.new(mp.outputs["Vector"], tex.inputs["Vector"])
    nt.links.new(tex.outputs["Color"], nt.nodes["Principled BSDF"].inputs["Base Color"])
    nt.nodes.active = tex
    pl.data.materials.append(mat)
    return pl


def main():
    pos, o = args()
    path, clip, prefix = pos
    arm = load(path)
    scene = bpy.context.scene
    scene.render.fps = FPS
    v0, Tc = walk_speed(arm) if "walk" in [a.name.split("_")[0] for a in bpy.data.actions] or bpy.data.actions.get("walk") else (0.0, 1.0)
    act = use(arm, clip)
    a0, a1 = int(act.frame_range[0]), int(act.frame_range[1])
    dur = (a1 - a0) / FPS
    move = mover(clip, v0, Tc, dur)
    span = move(dur)[0]
    size = int(o["size"])
    height = 1.9
    ground(max(8.0, span * 2 + 4))
    scene.render.engine = "BLENDER_WORKBENCH"
    sh = scene.display.shading
    sh.light = "STUDIO"
    sh.color_type = "TEXTURE" if o["tex"] == "1" else "SINGLE"
    sh.single_color = (0.75, 0.72, 0.68)
    sh.show_cavity = True
    sh.show_shadows = True
    sh.shadow_intensity = 0.6
    scene.display.shadow_shift = 0.1
    sh.background_type = "VIEWPORT"
    sh.background_color = (0.3, 0.33, 0.38)
    scene.view_settings.view_transform = "Standard"
    scene.view_settings.exposure = 0.35
    scene.render.resolution_x = size
    scene.render.resolution_y = size
    cam_data = bpy.data.cameras.new("cam")
    cam = bpy.data.objects.new("cam", cam_data)
    scene.collection.objects.link(cam)
    scene.camera = cam
    hand = o["hand"]
    views = o["views"].split(",")
    n = int(o["frames"]) or (a1 - a0 + 1)
    step = int(o["step"])
    frames = [round(a0 + (a1 - a0) * k / max(1, n - 1)) for k in range(n)] if n < (a1 - a0 + 1) else list(range(a0, a1 + 1, step))
    tmp = tempfile.mkdtemp()
    files = {v: [] for v in views}
    slide = []
    planted = {}
    for f in frames:
        t = (f - a0) / FPS
        x, yaw = move(t)
        arm.location = (x, 0, 0)
        arm.rotation_euler = (0, 0, math.radians(yaw))
        scene.frame_set(f)
        bpy.context.view_layer.update()
        # Glidningen: en ankel som står lavt, skal stå stille i verden.
        for s in ("L", "R"):
            p = arm.matrix_world @ arm.pose.bones[f"foot.{s}"].head
            if s in planted and p.z < planted[s][1] + 0.004 and abs(p.z - planted[s][0].z) < 0.004:
                slide.append((p - planted[s][0]).xy.length)
            planted[s] = (p, min(planted.get(s, (p, p.z))[1], p.z))
        for v in views:
            if hand:
                pb = arm.pose.bones[f"hand.{hand}"]
                c = arm.matrix_world @ pb.head + (arm.matrix_world.to_3x3() @ Vector((0.04, 0, -0.1)))
                cam_data.type = "PERSP"
                cam_data.lens = 60
                d = {"side": Vector((0.0, -1.0, 0.1)), "front": Vector((1.0, 0.0, 0.1)), "back": Vector((-1.0, 0.0, 0.1)), "out": Vector((0.0, 1 if hand == "L" else -1, 0.15))}[v]
                d = arm.matrix_world.to_3x3() @ d
                cam.location = c + d.normalized() * 0.75
            else:
                cam_data.type = "ORTHO"
                cam_data.ortho_scale = 2.3
                c = Vector((x + 0.05, 0, height / 2))
                if clip == "walk" or clip.startswith("stop"):
                    c.x = x if clip == "walk" else 0.25
                d = {"side": Vector((0.0, -1.0, 0.16)), "front": Vector((1.0, 0.0, 0.16)), "back": Vector((-1.0, 0.0, 0.16)), "top": Vector((0.0, 0.0, 1.0)),
                     "three": Vector((0.7, -0.7, 0.25))}[v]
                cam.location = c + d.normalized() * 6.0
            direction = c - cam.location
            cam.rotation_euler = direction.to_track_quat("-Z", "Y" if v != "top" else "X").to_euler()
            p = os.path.join(tmp, f"{v}_{f:04d}.png")
            scene.render.filepath = p
            bpy.ops.render.render(write_still=True)
            files[v].append(p)
    from PIL import Image, ImageDraw

    # Kontaktarket: høyst 12 bilder per visning.
    pick = [round(k * (len(frames) - 1) / 11) for k in range(12)] if len(frames) > 12 else list(range(len(frames)))
    W = size * len(pick)
    sheet = Image.new("RGB", (W, size * len(views)), (40, 40, 40))
    draw = ImageDraw.Draw(sheet)
    for r, v in enumerate(views):
        for c, k in enumerate(pick):
            im = Image.open(files[v][k]).convert("RGB")
            sheet.paste(im, (c * size, r * size))
            draw.text((c * size + 6, r * size + 4), f"{clip} {frames[k]}", fill=(255, 255, 255))
    sheet.save(f"{prefix}_{clip}.png")
    if o["gif"] == "1":
        ims = []
        for k in range(len(frames)):
            row = Image.new("RGB", (size * len(views), size))
            for c, v in enumerate(views):
                row.paste(Image.open(files[v][k]).convert("RGB"), (c * size, 0))
            ims.append(row.convert("P", palette=Image.ADAPTIVE, colors=128))
        ims[0].save(f"{prefix}_{clip}.gif", save_all=True, append_images=ims[1:], duration=int(1000 * step / FPS), loop=0)
    worst = max(slide) if slide else 0.0
    print(f"{clip}: {len(frames)} bilder, gangfart {v0:.3f} m/s (syklus {Tc:.3f} s), glidning per bilde høyst {worst * 100:.1f} cm -> {prefix}_{clip}.png")


main()
