"""Et våpen eller skjold fra armenes rom (`tools/models/arms.py`) lagt
rett mot kameraet til godkjenningsbilder: klinga opp med flatsida mot
kameraet, skjoldet forfra og bakfra.

    python3.13 tools/models/item_preview.py MODELL.glb UT_PREFIKS

Skriver UT_PREFIKS_front.png, _corner.png og _side.png (skjoldene:
_front.png, _back.png og _corner.png)."""

import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bpy  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import arms  # noqa: E402


def main(path, out):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=path)
    obs = [o for o in bpy.data.objects if o.type == "MESH"]
    name = os.path.basename(path)[:-4]
    if name.startswith("shield"):
        g, fw, up, rt = arms.shield_axes()
        M = Matrix((rt, fw, up)).transposed().to_4x4()
        M.translation = g
        # Skjoldets rom: x høyre, y fram (forsida), z opp.
        to_local = M.inverted()
        views = {"front": (0, -1, 0), "back": (0, 1, 0), "corner": (0.6, -0.8, 0.25)}
        disp = Matrix.Identity(4)
    else:
        # Håndas rom: klinga langs −x, bredden langs y, flatsida langs z.
        to_local = arms.RIGHT.inverted()
        # Klinga opp (−x → +z), bredden vannrett (y → x), flatsida mot kameraet (z → −y).
        disp = Matrix(((0, 1, 0, 0), (0, 0, -1, 0), (-1, 0, 0, 0), (0, 0, 0, 1)))
        views = {"front": (0, -1, 0), "corner": (0.7, -0.7, 0.15), "side": (1, 0, 0)}
    # glTF-importen setter modellen i Blenders rom slik den ble eksportert.
    for o in obs:
        o.data.transform(disp @ to_local @ o.matrix_world)
        o.matrix_world = Matrix.Identity(4)
    pts = [v.co for o in obs for v in o.data.vertices]
    lo = Vector((min(p.x for p in pts), min(p.y for p in pts), min(p.z for p in pts)))
    hi = Vector((max(p.x for p in pts), max(p.y for p in pts), max(p.z for p in pts)))
    c, size = (lo + hi) / 2, max(hi - lo)
    sc = bpy.context.scene
    sc.render.engine = "BLENDER_WORKBENCH"
    sc.display.shading.light = "STUDIO"
    sc.display.shading.color_type = "TEXTURE"
    sc.display.shading.show_cavity = True
    sc.display.shading.background_type = "VIEWPORT"
    sc.display.shading.background_color = (0.55, 0.57, 0.6)
    sc.render.resolution_x, sc.render.resolution_y = 900, 900
    sc.world = bpy.data.worlds.new("w")
    cam = bpy.data.cameras.new("cam")
    cam.type = "ORTHO"
    cam.ortho_scale = size * 1.12
    co = bpy.data.objects.new("cam", cam)
    sc.collection.objects.link(co)
    sc.camera = co
    for v, d in views.items():
        d = Vector(d).normalized()
        co.location = c - d * size * 3
        co.rotation_euler = d.to_track_quat("-Z", "Y").to_euler()
        sc.render.filepath = f"{out}_{v}.png"
        bpy.ops.render.render(write_still=True)
        print(sc.render.filepath)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
