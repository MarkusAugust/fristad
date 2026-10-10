"""Materialet, render av arket ved siden av forbildet, mellomvinkler, turnaround og eksport til glb."""
import json
import os

import numpy as np

from . import job as J
from .texture import b2w, w2b

# ---------------------------------------------------------------- render (Blender)
def build_material(ob):
    import bpy
    mat = ob.data.materials[0]
    nt = mat.node_tree
    for n in list(nt.nodes):
        nt.nodes.remove(n)
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Roughness"].default_value = 0.8
    bsdf.inputs["Specular IOR Level"].default_value = 0.15  # malt stil: lyset er i teksturen, nesten ingen glans
    al = nt.nodes.new("ShaderNodeTexImage")
    al.image = bpy.data.images.load(J.p("head_albedo.png"))
    nm = nt.nodes.new("ShaderNodeTexImage")
    nm.image = bpy.data.images.load(J.p("head_normal.png"))
    nm.image.colorspace_settings.name = "Non-Color"
    nmap = nt.nodes.new("ShaderNodeNormalMap")
    nmap.inputs["Strength"].default_value = 0.8
    nt.links.new(al.outputs[0], bsdf.inputs["Base Color"])
    nt.links.new(nm.outputs[0], nmap.inputs["Color"])
    nt.links.new(nmap.outputs[0], bsdf.inputs["Normal"])
    nt.links.new(bsdf.outputs[0], out.inputs[0])


def render(res=512, samples=32, tag="ark", glb=None):
    import bpy
    import cv2
    from mathutils import Vector
    bpy.ops.wm.open_mainfile(filepath=J.p("head.blend"))
    sc = bpy.context.scene
    ob = bpy.data.objects["head"]
    build_material(ob)
    fr = json.load(open(J.p("frame.json")))["frame"]
    sc.render.engine = "CYCLES"
    sc.cycles.device = "CPU"
    sc.cycles.samples = samples
    sc.cycles.use_denoising = False  # skarpt: støyfjerneren gjør teksturen myk
    sc.render.resolution_x = sc.render.resolution_y = res
    sc.render.film_transparent = True
    sc.view_settings.view_transform = "Standard"
    w = bpy.data.worlds.new("w"); sc.world = w
    w.use_nodes = True
    w.node_tree.nodes["Background"].inputs[0].default_value = (1, 1, 1, 1)
    w.node_tree.nodes["Background"].inputs[1].default_value = 0.85
    cam_d = bpy.data.cameras.new("cam"); cam_d.type = "ORTHO"
    cam = bpy.data.objects.new("cam", cam_d); sc.collection.objects.link(cam); sc.camera = cam
    key_d = bpy.data.lights.new("key", "AREA"); key_d.size = 0.6; key_d.energy = 1.5
    key = bpy.data.objects.new("key", key_d); sc.collection.objects.link(key)
    # midten av hodet i Blender
    vs = np.array([ob.matrix_world @ v.co for v in ob.data.vertices])
    ctr = (vs.min(0) + vs.max(0)) / 2

    def shoot(direction, up, ortho, centre, path):
        d = Vector(direction).normalized()
        cam.location = Vector(centre) + d * 1.0
        cam.rotation_euler = (-d).to_track_quat("-Z", "Y").to_euler()
        cam_d.ortho_scale = ortho
        # nøkkellys oppe til venstre foran, fast i forhold til kameraet
        right = (-d).cross(Vector((0, 0, 1))).normalized()
        key.location = Vector(centre) + d * 0.6 - right * 0.4 + Vector((0, 0, 0.5))
        key.rotation_euler = (Vector(centre) - key.location).to_track_quat("-Z", "Y").to_euler()
        sc.render.filepath = path
        bpy.ops.render.render(write_still=True)

    os.makedirs(J.SHOTS, exist_ok=True)
    for k, v in J.VIEWS.items():
        o = fr[k]
        # midten av flisa i verden -> Blender
        a = (J.TW / 2 - o["c"]) / o["sy"]
        y = -(J.TH / 2 - o["top"]) / o["sy"]
        r = np.array(v["right"], float)
        wpt = r * a + np.array([0, y, 0])
        cb = w2b(wpt)
        # dybden langs kameraet: bruk midten av hodet
        cdir = w2b(np.array(v["cam"], float)) / J.SCALE
        cb = cb - cdir * (cb @ cdir) + cdir * (np.asarray(ctr) @ cdir)
        path = J.p(f"r_{k}.png")
        shoot(tuple(cdir), (0, 0, 1), max(J.TW, J.TH) / o["sy"] * J.SCALE, tuple(cb), path)
    # mellomvinkel: 45 grader mellom forfra og venstre profil, 20 grader ovenfra
    yaw, pitch = np.radians(45), np.radians(20)
    dirw = np.array([np.sin(yaw) * np.cos(pitch), np.sin(pitch), np.cos(yaw) * np.cos(pitch)])
    shoot(tuple(w2b(dirw) / J.SCALE), (0, 0, 1), max(J.TW, J.TH) / fr["front"]["sy"] * J.SCALE, tuple(ctr), J.p("r_mid.png"))
    yaw = np.radians(-40); pitch = np.radians(-10)
    dirw = np.array([np.sin(yaw) * np.cos(pitch), np.sin(pitch), np.cos(yaw) * np.cos(pitch)])
    shoot(tuple(w2b(dirw) / J.SCALE), (0, 0, 1), max(J.TW, J.TH) / fr["front"]["sy"] * J.SCALE, tuple(ctr), J.p("r_mid2.png"))
    def flat(path):  # gjennomsiktig bakgrunn -> samme grå som forbildet
        im = cv2.imread(path, cv2.IMREAD_UNCHANGED).astype(np.float32)
        a = im[:, :, 3:4] / 255
        return (im[:, :, :3] * a + 130 * (1 - a)).astype(np.uint8)
    tiles_out = [flat(J.p(f"r_{k}.png")) for k in J.VIEWS]
    # ovenfra og skrått bakfra: toppen av håret og nakken
    for name, yaw_d, pitch_d in (("r_top", 25, 60), ("r_backlow", 150, -5)):
        yaw, pitch = np.radians(yaw_d), np.radians(pitch_d)
        dirw = np.array([np.sin(yaw) * np.cos(pitch), np.sin(pitch), np.cos(yaw) * np.cos(pitch)])
        shoot(tuple(w2b(dirw) / J.SCALE), (0, 0, 1), max(J.TW, J.TH) / fr["front"]["sy"] * J.SCALE, tuple(ctr),
              J.p(name + ".png"))
    ref = cv2.imread(J.REF)
    ref = cv2.resize(ref, (2 * res, 2 * res), interpolation=cv2.INTER_AREA)
    sheet = np.vstack([np.hstack(tiles_out[:2]), np.hstack(tiles_out[2:])])
    full = np.hstack([ref, sheet])
    cv2.imwrite(os.path.join(J.SHOTS, f"{tag}.png"), full)
    mid = np.hstack([flat(J.p("r_mid.png")), flat(J.p("r_mid2.png")), flat(J.p("r_top.png")), flat(J.p("r_backlow.png"))])
    cv2.imwrite(os.path.join(J.SHOTS, f"{tag}_mellom.png"), mid)
    turnaround(cam, cam_d, key, ctr, shoot)
    export(ob, glb or os.path.join(J.SHOTS, J.NAME + ".glb"))


def turnaround(cam, cam_d, key, ctr, shoot, n=24, res=256):
    """Turnaround (360°, 10° ovenfra) som GIF."""
    import bpy
    from PIL import Image
    sc = bpy.context.scene
    sc.render.resolution_x = sc.render.resolution_y = res
    sc.cycles.samples = 12
    frames = []
    for i in range(n):
        yaw, pitch = 2 * np.pi * i / n, np.radians(10)
        dirw = np.array([np.sin(yaw) * np.cos(pitch), np.sin(pitch), np.cos(yaw) * np.cos(pitch)])
        path = J.p(f"turn_{i:02d}.png")
        shoot(tuple(w2b(dirw) / J.SCALE), (0, 0, 1), max(J.TW, J.TH) * 0.95 * J.SCALE, tuple(ctr), path)
        im = Image.open(path).convert("RGBA")
        bg = Image.new("RGBA", im.size, (130, 130, 130, 255))
        frames.append(Image.alpha_composite(bg, im).convert("RGB"))
    out = os.path.join(J.SHOTS, J.NAME + "_turnaround.gif")
    frames[0].save(out, save_all=True, append_images=frames[1:], duration=120, loop=0)
    print("turnaround", out)


def export(ob, path):
    """glb med én tekstur (JPEG) og normalkart, slanket med tools/models/slim.py."""
    import bpy
    import subprocess
    import sys
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    for o in bpy.context.scene.objects:
        o.select_set(o == ob)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", use_selection=True, export_image_format="JPEG",
                              export_jpeg_quality=90, export_cameras=False, export_lights=False)
    subprocess.run([sys.executable, os.path.join(J.ROOT, "tools/models/slim.py"), path], check=False)
    print("glb", path, len(ob.data.polygons), "trekanter")
