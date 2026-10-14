"""Kontrollen av en innbygger mot arket (eieren 14. oktober: «Jeg er IKKE
fornøyd med motoren som skal lage 3d figurer av sheets … Den må forbedres
slik at du treffer mer riktig med en gang»). Figuren rendres i hvileposen
forfra, fra siden og bakfra i arkets egen målestokk (ortografisk, flat
tekstur uten lys), og sammenlignes med arket piksel for piksel:

- **mangler**: arket viser figuren, modellen er tom (hull gjennom kroppen,
  innenfor albuen, en kappet hake),
- **stikker ut**: modellen er der arket er bakgrunn,
- **svart**: modellen er svart der arket ikke er det (bakingen),
- **fargen**: middelavviket der begge viser figuren (to ansikter, et
  speilvendt hode, feil bilde på en flate).

Tynne ting figuren holder (spyd, høygaffel, skaft) er ikke laget ennå og
regnes ikke som mangler. Skriver tallene som json og et diagnosebilde
(rødt mangler, blått stikker ut, gult der arket er uenig med seg selv
forfra og bakfra, og fargeavviket) og stopper med feil
over tersklene:

    python3.13 tools/models/folk_check.py smith [UT_DIR]
"""

import json
import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import numpy as np  # noqa: E402
from mathutils import Vector  # noqa: E402
from PIL import Image  # noqa: E402
from scipy import ndimage  # noqa: E402

import folk_sheet  # noqa: E402

ROOT = folk_sheet.ROOT
MISSING = 0.02  # høyst så stor del av figuren på arket kan mangle i modellen (per visning)
EXTRA = 0.04  # høyst så stor del kan stikke ut
COLOR = 0.16  # høyst så stort middelavvik i fargen (0–1 per kanal) der begge viser figuren
HOLE = 0.004  # høyst så stor del av figuren i ett sammenhengende hull (av høyden², som areal)
OVERLAP = 0.01  # høyst så stor del av teksturen kan to flater dele (da får den ene den andres farge: den svarte hanska)
BLACK = 0.002  # høyst så stor del av figuren kan være svart i modellen der arket ikke er det
TOOL_THICK = 0.02  # et skaft er høyst så tykt (halve tykkelsen, del av høyden)
SHAFT_LEN = 0.08  # og minst så langt
SHAFT_V = 0.04  # et loddrett stykke av et skaft er minst så langt
TOOL_NEAR = 0.1  # det som mangler innen så langt fra hånda med et verktøy (del av høyden), er verktøyet
THIN = 0.025  # det som er tynnere enn så mye av høyden på arket, er ting figuren holder


def render(path, m, view, out):
    """Modellen i hvileposen sett som arket, i arkets piksler (hele arkets
    høyde, et vindu rundt visningen i bredden)."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=path)
    from mathutils import Matrix

    for o in bpy.data.objects:
        if o.type == "ARMATURE":
            if o.animation_data:
                o.animation_data.action = None
                for t in o.animation_data.nla_tracks:
                    t.mute = True
            # Hvileposen er formen slik den ble bygd etter arket.
            for pb in o.pose.bones:
                pb.matrix_basis = Matrix.Identity(4)
    # Workbench viser bildet i den aktive noden; det skal være fargen, ikke
    # okklusjonen (mørk der armen møter kroppen) eller normalkartet.
    for mat in bpy.data.materials:
        if not mat.use_nodes:
            continue
        nt = mat.node_tree
        bsdf = next((n for n in nt.nodes if n.type == "BSDF_PRINCIPLED"), None)
        if bsdf and bsdf.inputs["Base Color"].links:
            src = bsdf.inputs["Base Color"].links[0].from_node
            while src.type != "TEX_IMAGE" and src.inputs and any(i.links for i in src.inputs):
                src = next(i.links[0].from_node for i in src.inputs if i.links)
            if src.type == "TEX_IMAGE":
                nt.nodes.active = src
    sc = bpy.context.scene
    sc.frame_set(0)
    W, H = m["size"]
    px, ground = m["px"], m["ground"]
    win = 900
    cam_d = bpy.data.cameras.new("c")
    cam_d.type = "ORTHO"
    cam_d.ortho_scale = max(win, H) * px
    cam = bpy.data.objects.new("c", cam_d)
    sc.collection.objects.link(cam)
    sc.camera = cam
    d = {"front": Vector((1, 0, 0)), "back": Vector((-1, 0, 0)), "side": Vector((0, -m["facing"], 0))}[view]
    zc = (ground - H / 2) * px
    cam.location = Vector((0, 0, zc)) + d * 10
    cam.rotation_euler = (-d).to_track_quat("-Z", "Y").to_euler()
    sc.render.engine = "BLENDER_WORKBENCH"
    sh = sc.display.shading
    sh.light = "FLAT"
    sh.color_type = "TEXTURE"
    sc.view_settings.view_transform = "Standard"
    sc.render.film_transparent = True
    sc.render.resolution_x = win
    sc.render.resolution_y = H
    sc.render.filepath = out
    bpy.ops.render.render(write_still=True)
    return win


def uv_overlap(n=1024, shrink=0.7):
    """Hvor stor del av teksturen som brukes av mer enn én flate (i
    modellen som er lastet). Hver flate krympes litt mot midten, så naboer
    som deler en kant ikke telles."""
    from PIL import ImageDraw

    cnt = np.zeros((n, n), np.int32)
    for ob in bpy.data.objects:
        # Importen lager en kule til å vise beina i skjelettet; den er ikke i fila.
        if ob.type != "MESH" or not ob.data.uv_layers or not ob.data.materials:
            continue
        uv = ob.data.uv_layers.active.data
        for p in ob.data.polygons:
            pts = [(uv[li].uv[0] * n, (1 - uv[li].uv[1]) * n) for li in p.loop_indices]
            cx, cy = sum(a for a, _ in pts) / len(pts), sum(b for _, b in pts) / len(pts)
            pts = [(cx + (a - cx) * shrink, cy + (b - cy) * shrink) for a, b in pts]
            x0, y0 = max(0, int(min(a for a, _ in pts))), max(0, int(min(b for _, b in pts)))
            x1, y1 = min(n, int(max(a for a, _ in pts)) + 2), min(n, int(max(b for _, b in pts)) + 2)
            if x1 <= x0 or y1 <= y0:
                continue
            im = Image.new("L", (x1 - x0, y1 - y0), 0)
            ImageDraw.Draw(im).polygon([(a - x0, b - y0) for a, b in pts], fill=1)
            cnt[y0:y1, x0:x1] += np.asarray(im)
    used = (cnt > 0).sum()
    return float((cnt > 1).sum() / max(1, used))


def check(name, out_dir):
    import townsfolk  # noqa: E402

    # Målestokken følger modellens egen høyde (fra byggingen).
    m = folk_sheet.measure(name, townsfolk.FOLK[name])
    path = os.path.join(ROOT, "assets", "models", "npcs", f"npc_{name}.glb")
    sheet_path = os.path.join(folk_sheet.REF, f"{name}_body.jpeg")
    sheet = np.asarray(Image.open(sheet_path).convert("RGB")).astype(float) / 255
    fg = folk_sheet.mask(sheet_path)
    H = m["H"]
    thin = max(3, int(THIN * H))
    os.makedirs(out_dir, exist_ok=True)
    report, panels = {"name": name, "views": {}}, []
    # Der bildet forfra (speilet) og bakfra er uenige om omrisset (armen
    # holder krukka forfra, men henger bakfra), kan ingen modell treffe
    # begge: det regnes ikke i noen av dem, men tallet står i rapporten.
    win0 = 900
    def strip(view):
        cx = int(round(m["views"][view]))
        out = np.zeros((fg.shape[0], win0), bool)
        x0 = cx - win0 // 2
        a, b = max(0, x0), min(fg.shape[1], x0 + win0)
        out[:, a - x0: b - x0] = fg[:, a:b]
        lab, k = ndimage.label(out)
        if k:
            mid = lab[:, win0 // 2 - 40: win0 // 2 + 40]
            out = lab == (np.bincount(mid[mid > 0]).argmax() if (mid > 0).any() else 0)
        return out
    f_mask, b_mask = strip("front"), strip("back")[:, ::-1]
    disagree = ndimage.binary_opening(f_mask ^ b_mask, iterations=2)
    report["sheet_disagrees"] = float(disagree.sum() / max(1, f_mask.sum()))
    ignore = {"front": disagree, "back": disagree[:, ::-1], "side": np.zeros_like(disagree)}
    for view in ("front", "side", "back"):
        tmp = os.path.join(out_dir, f"{name}_{view}.png")
        win = render(path, m, view, tmp)
        if view == "front":
            report["uv_overlap"] = uv_overlap()
        img = np.asarray(Image.open(tmp).convert("RGBA")).astype(float) / 255
        model = img[..., 3] > 0.5
        cx = int(round(m["views"][view]))
        x0 = cx - win // 2
        ref = np.zeros_like(model)
        col = np.zeros(model.shape + (3,))
        a, b = max(0, x0), min(fg.shape[1], x0 + win)
        ref[:, a - x0: b - x0] = fg[:, a:b]
        col[:, a - x0: b - x0] = sheet[:, a:b]
        # Bare den sammenhengende figuren i midten (nabovisningene utenfor).
        lab, k = ndimage.label(ref)
        if k:
            mid = lab[:, win // 2 - 40: win // 2 + 40]
            keep = np.bincount(mid[mid > 0]).argmax() if (mid > 0).any() else 0
            ref = lab == keep
        # Ting figuren holder (tynne skaft) regnes ikke med.
        body = ndimage.binary_opening(ref, structure=np.ones((thin, thin)))
        body = ndimage.binary_dilation(body, iterations=thin // 2) & ref
        area = max(1, body.sum())
        missing = body & ~model
        # Kanten av omrisset (noen piksler) er ikke hull.
        edge = body & ~ndimage.binary_erosion(body, iterations=3)
        missing &= ~edge & ~ignore[view]
        extra = model & ~ndimage.binary_dilation(ref, iterations=4) & ~ignore[view]
        both = model & body & ~edge
        err = np.abs(img[..., :3] - col).mean(2)
        # Det figuren holder (øksa, høygaffelen) blir en egen gjenstand: det
        # som mangler og henger sammen med området rundt hånda, telles ikke.
        toolmask = np.zeros_like(missing)
        for s, tool in townsfolk.TOOLS.get(name, {}).items():
            arm = m["parts"].get(f"arm.{s}")
            if not arm:
                continue
            y, cy, hw, sx, sd = arm[-1]
            if view == "side":
                x = win // 2 + sx
            else:
                x = win // 2 + (cy if view == "front" else -cy)
            yy, xx = np.ogrid[: missing.shape[0], : missing.shape[1]]
            near = (yy - y) ** 2 + (xx - x) ** 2 < (TOOL_NEAR * H) ** 2
            lab, k = ndimage.label(missing)
            hit = np.unique(lab[near & missing])
            held_by = np.isin(lab, hit[hit > 0])
            # Resten av skaftet (langs ansiktet, ned mot bakken) er lange,
            # rette og smale biter; en glipe langs omrisset følger kroppen
            # og er ikke rett.
            lab2, k2 = ndimage.label(missing & ~held_by)
            for i in range(1, k2 + 1):
                ys_, xs_ = np.nonzero(lab2 == i)
                if len(ys_) < 20:
                    continue
                pts = np.stack([ys_, xs_], 1).astype(float)
                pts -= pts.mean(0)
                w_, vec = np.linalg.eigh(np.cov(pts.T))
                major, minor = pts @ vec[:, 1], pts @ vec[:, 0]
                if np.ptp(major) >= SHAFT_LEN * H and np.abs(minor).max() <= TOOL_THICK * H:
                    held_by |= lab2 == i
            # Et loddrett skaft som flyter sammen med noe annet (gaffelen
            # rett foran ansiktet til bonden): lange loddrette strekninger
            # som er smale på tvers.
            tall = ndimage.binary_opening(missing, structure=np.ones((int(SHAFT_V * H), 1)))
            wide = ndimage.binary_opening(missing, structure=np.ones((1, int(2 * TOOL_THICK * H))))
            shaft = tall & ~wide
            held_by |= ndimage.binary_dilation(shaft, iterations=2) & missing
            # Og det som henger i skaftet utenfor figuren (gaffelen øverst).
            lab3, k3 = ndimage.label(missing & ~held_by)
            if k3:
                touch = ndimage.binary_dilation(held_by, iterations=3)
                outside = ~ndimage.binary_dilation(model, iterations=4)
                for i in range(1, k3 + 1):
                    blob = lab3 == i
                    if (blob & touch).any() and (blob & outside).sum() > 0.7 * blob.sum():
                        held_by |= blob
            v_tool = report.setdefault("tools", {})
            v_tool[f"{view}.{tool}"] = float(held_by.sum() / area)
            missing &= ~held_by
            toolmask |= held_by
        lab, k = ndimage.label(missing)
        holes = sorted(ndimage.sum(missing, lab, range(1, k + 1)), reverse=True)[:3] if k else []
        # Det som stikker ut, de største bitene.
        le, ke = ndimage.label(extra)
        if ke:
            sz = ndimage.sum(extra, le, range(1, ke + 1))
            cm = ndimage.center_of_mass(extra, le, range(1, ke + 1))
            report.setdefault("extra_blobs", {})[view] = [(round(float(sz[i] / max(1, body.sum())), 4), int(cm[i][0]), int(cm[i][1] - win // 2)) for i in np.argsort(-sz)[:6]]
        # De største bitene som mangler (andel av figuren, midtpunkt i arket).
        if k:
            sz = ndimage.sum(missing, lab, range(1, k + 1))
            cm = ndimage.center_of_mass(missing, lab, range(1, k + 1))
            top = np.argsort(-sz)[:6]
            report.setdefault("blobs", {})[view] = [(round(float(sz[i] / area), 4), int(cm[i][0]), int(cm[i][1] - win // 2)) for i in top]
        # Svart i modellen der arket ikke er svart (bakingen så en annen
        # flate først, eller okklusjonen: de svarte flekkene på armen).
        dark = both & (img[..., :3].max(2) < 0.06) & (col.max(2) > 0.15)
        v = {
            "black": float(dark.sum() / area),
            "missing": float(missing.sum() / area),
            "extra": float(extra.sum() / area),
            "color": float(err[both].mean()) if both.any() else 1.0,
            "hole": float((holes[0] if holes else 0) / (H * H)),
        }
        report["views"][view] = v
        # Diagnosebildet: arket, modellen, rødt mangler / blått stikker ut, fargeavviket.
        y0, y1 = max(0, m["top"] - 30), min(model.shape[0], m["ground"] + 30)
        diag = np.ones(model.shape + (3,)) * 0.85
        diag[body] = (0.6, 0.6, 0.6)
        diag[model & body] = (0.75, 0.75, 0.75)
        diag[missing] = (0.9, 0.1, 0.1)
        diag[extra] = (0.1, 0.3, 0.9)
        diag[ignore[view] & ~(missing | extra)] = (0.95, 0.8, 0.3)
        diag[toolmask] = (0.6, 0.3, 0.8)
        heat = np.clip(err * 3, 0, 1)
        hm = np.stack([heat, 1 - heat, 0 * heat], 2) * both[..., None] + (1 - both[..., None]) * 0.85
        rendered = img[..., :3] * img[..., 3:4] + (1 - img[..., 3:4]) * 0.85
        for p in (col, rendered, diag, hm):
            panels.append((p[y0:y1] * 255).astype(np.uint8))
    row = []
    for i in range(0, len(panels), 4):
        row.append(np.concatenate(panels[i: i + 4], 1))
    im = Image.fromarray(np.concatenate(row, 0))
    im.thumbnail((2400, 2400))
    im.save(os.path.join(out_dir, f"{name}_check.jpg"), quality=85)
    fails = []
    for view, v in report["views"].items():
        for key, limit in (("missing", MISSING), ("extra", EXTRA), ("color", COLOR), ("hole", HOLE), ("black", BLACK)):
            if v[key] > limit:
                fails.append(f"{view}: {key} {v[key]:.3f} (høyst {limit})")
    if report["uv_overlap"] > OVERLAP:
        fails.append(f"teksturen: {report['uv_overlap'] * 100:.1f} % delt mellom flater (høyst {OVERLAP * 100:.0f} %)")
    report["fails"] = fails
    with open(os.path.join(out_dir, f"{name}_check.json"), "w") as f:
        json.dump(report, f, indent=1, ensure_ascii=False)
    return report


if __name__ == "__main__":
    name = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(ROOT, "target", "qa", "folk")
    r = check(name, out)
    print(f"{name}: teksturen delt mellom flater {r['uv_overlap'] * 100:.1f} %, arket uenig forfra og bakfra {r['sheet_disagrees'] * 100:.1f} %")
    for view, v in r["views"].items():
        print(f"{name} {view}: mangler {v['missing'] * 100:.1f} %, stikker ut {v['extra'] * 100:.1f} %, største hull {v['hole'] * 1e4:.1f}‱, fargeavvik {v['color']:.3f}, svart {v['black'] * 100:.2f} %")
    if r["fails"]:
        print("FEIL: " + "; ".join(r["fails"]))
        sys.exit(1)
