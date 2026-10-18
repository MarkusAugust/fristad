"""Stedfortredere for ikonene på utstyrskortene (inventaret og galleriet i
skriptoriet): `assets/textures/icons/<id>.png`, 512 × 512 på nøytral
bakgrunn, til eieren har malt de ekte etter `docs/prompter/ikoner-utstyr.md`.

    python3 tools/models/gear_icons.py [id ...] [--force]

Våpnene og skjoldene rendres fra modellen med `item_preview.py` (våpnene
på skrå som i arkene), rustningene er hanska og armen fra grepsarket i
fargene til settet (det modellen er malt med), og lykta er lykta fra
Skriptoriet (`preview.py`). Et ikon som finnes og ikke er en stedfortreder
(eieren har lagt det der), skrives aldri over: stedfortrederne står i
`assets/textures/icons/placeholders.json`."""

import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "assets", "textures", "icons")
HERE = os.path.dirname(os.path.abspath(__file__))
SIZE = 512
BG = (118, 114, 106)  # nøytral, varm grå (samme som promptene ber om)

# id i `craft.rs` → (slag, modell eller sett)
ICONS = {
    "rusty_sword": ("weapon", "sword"),
    "component_sword": ("weapon", "sword_rare"),
    "tag_dagger": ("weapon", "dagger"),
    "card_axe": ("weapon", "axe"),
    "alert_mace": ("weapon", "mace"),
    "tooltip_stiletto": ("weapon", "dagger_rare"),
    "divider_blade": ("weapon", "sword_legend"),
    "tabs_axe": ("weapon", "axe_rare"),
    "old_shield": ("shield", "shield_round"),
    "card_shield": ("shield", "shield_round_blue"),
    "badge_heater": ("shield", "shield_heater"),
    "fieldset_tower": ("shield", "shield_tower"),
    "dialog_heater": ("shield", "shield_heater_violet"),
    "badge_jerkin": ("armor", "arms_leather"),
    "tag_cuirass": ("armor", "arms_leather_dark"),
    "fieldset_mail": ("armor", "arms_mail"),
    "search_plate": ("armor", "arms_plate"),
    "dialog_plate": ("armor", "arms_plate_dark"),
    "search_lantern": ("lantern", "furniture/hanging_lantern"),
}


def background():
    import numpy as np
    from PIL import Image

    y, x = np.mgrid[0:SIZE, 0:SIZE] / (SIZE - 1) - 0.5
    k = 1.0 - 0.35 * np.clip(np.hypot(x, y) * 1.6, 0, 1) ** 2
    return Image.fromarray((np.array(BG)[None, None, :] * k[..., None]).astype(np.uint8))


def keyed(path):
    """Bildet med den ensfargede bakgrunnen fra rendringen gjort gjennomsiktig."""
    import numpy as np
    from PIL import Image
    from scipy import ndimage

    a = np.asarray(Image.open(path).convert("RGB")).astype(int)
    bg = a[2, 2]
    fg = np.abs(a - bg).sum(2) > 10
    fg = ndimage.binary_opening(fg, iterations=1)
    alpha = (ndimage.gaussian_filter(fg.astype(float), 0.7) * 255).astype(np.uint8)
    rgba = np.dstack([a.astype(np.uint8), alpha])
    return Image.fromarray(rgba, "RGBA")


def fit(img, margin=0.1):
    """Beskåret til innholdet og lagt midt på bakgrunnen."""
    box = img.getbbox()
    img = img.crop(box)
    side = int(SIZE * (1 - 2 * margin))
    k = side / max(img.size)
    img = img.resize((max(1, int(img.width * k)), max(1, int(img.height * k))))
    out = background().convert("RGBA")
    out.alpha_composite(img, ((SIZE - img.width) // 2, (SIZE - img.height) // 2))
    return out.convert("RGB")


def render_item(model, tmp):
    glb = os.path.join(ROOT, "assets", "models", "arms", f"{model}.glb")
    pre = os.path.join(tmp, model)
    subprocess.run([sys.executable, os.path.join(HERE, "item_preview.py"), glb, pre], check=True, capture_output=True)
    return f"{pre}_front.png"


PLAIN = """
import bpy, sys
from mathutils import Vector
path, out = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=path)
obs = [o for o in bpy.data.objects if o.type == "MESH"]
pts = [o.matrix_world @ v.co for o in obs for v in o.data.vertices]
lo = Vector([min(p[i] for p in pts) for i in range(3)])
hi = Vector([max(p[i] for p in pts) for i in range(3)])
c, size = (lo + hi) / 2, max(hi - lo)
sc = bpy.context.scene
sc.render.engine = "BLENDER_WORKBENCH"
sc.display.shading.light = "STUDIO"
sc.display.shading.color_type = "TEXTURE"
sc.display.shading.show_cavity = True
sc.display.shading.background_type = "VIEWPORT"
sc.display.shading.background_color = (0.55, 0.57, 0.6)
sc.render.resolution_x = sc.render.resolution_y = 900
cam = bpy.data.cameras.new("cam")
cam.type = "ORTHO"
cam.ortho_scale = size * 1.12
co = bpy.data.objects.new("cam", cam)
sc.collection.objects.link(co)
sc.camera = co
d = Vector((0.5, 1.0, -0.25)).normalized()
co.location = c - d * size * 3
co.rotation_euler = d.to_track_quat("-Z", "Y").to_euler()
sc.render.filepath = out
bpy.ops.render.render(write_still=True)
"""


def render_lantern(model, tmp):
    out = os.path.join(tmp, "lantern.png")
    glb = os.path.join(ROOT, "assets", "models", f"{model}.glb")
    subprocess.run([sys.executable, "-c", PLAIN, glb, out], check=True, capture_output=True)
    return out


def armor_sheet(kind):
    """Hånda med armen ovenfra fra grepsarket i settets farger."""
    sys.path.insert(0, HERE)
    import arms
    from PIL import Image
    from scipy import ndimage
    import numpy as np

    path = arms.tinted_sheet(kind)
    gy, dx = arms.GRIP_ROWS[kind]
    raw = np.asarray(Image.open(arms.GRIP_SHEET).convert("RGB")).astype(int)
    col = arms.COL_TOP + dx
    box = (col - 200, gy - 110, col + 200, gy + arms.SLEEVE_IMG)
    m = np.abs(raw - raw[4, 4]).max(2) > 25
    m = ndimage.binary_fill_holes(ndimage.binary_opening(m, iterations=2))
    img = Image.open(path).convert("RGB").crop(box)
    alpha = Image.fromarray((m[box[1]:box[3], box[0]:box[2]] * 255).astype(np.uint8))
    img.putalpha(alpha)
    return img


def main(args):
    from PIL import Image

    force = "--force" in args
    which = [a for a in args if a in ICONS] or list(ICONS)
    os.makedirs(OUT, exist_ok=True)
    plist = os.path.join(OUT, "placeholders.json")
    placeholders = set(json.load(open(plist))) if os.path.exists(plist) else set()
    with tempfile.TemporaryDirectory() as tmp:
        for id_ in which:
            dst = os.path.join(OUT, f"{id_}.png")
            if os.path.exists(dst) and id_ not in placeholders and not force:
                print(f"{id_}: eierens ikon står, hoppes over")
                continue
            kind, model = ICONS[id_]
            if kind == "armor":
                img = armor_sheet(model)
            elif kind == "lantern":
                img = keyed(render_lantern(model, tmp))
            else:
                img = keyed(render_item(model, tmp))
                if kind == "weapon":
                    # På skrå som i arkene: spissen opp mot høyre.
                    img = img.rotate(-25 if model.startswith("axe") else -40, resample=Image.BICUBIC, expand=True)
            fit(img).save(dst)
            placeholders.add(id_)
            print(dst)
    json.dump(sorted(placeholders), open(plist, "w"), indent=1)


if __name__ == "__main__":
    main(sys.argv[1:])
