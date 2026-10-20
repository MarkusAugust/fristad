"""Det en fiende holder, satt på beinet slik spillet gjør (`mob_rigs::HELD`):
bueskytterens bue (`bow.py`) på beinet «bow» (`mob3d.bow_bone`). Bildene
som rendrer glb-fila alene (`clip_sheet.py`, `tools/qa/studio.py`) så ellers
bare buen på ryggen, og eieren trodde han skjøt med tomme hender.

Beinets rom er buens rom i Blender (z opp), og importen gir begge tilbake i
Blenders rom, så buen står i `armatur.matrix_world @ beinet.matrix`."""

import os

import bpy

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
# beinet → modellen (som `mob_rigs::HELD` og `models!` i gltf.rs)
HELD = {"bow": "assets/models/props/bow.glb"}


def attach():
    """Henter modellene til beinene som finnes i scenen. Gir [(armatur,
    bein, [objekter])]; `place` setter dem etter posen."""
    out = []
    for arm in [o for o in bpy.data.objects if o.type == "ARMATURE"]:
        for bone, path in HELD.items():
            if bone not in arm.pose.bones:
                continue
            before = set(bpy.data.objects)
            bpy.ops.import_scene.gltf(filepath=os.path.join(ROOT, path))
            new = [o for o in bpy.data.objects if o not in before]
            for o in new:
                if o.type == "MESH":
                    # Bare meshen; nodenes egne foreldre slippes.
                    m = o.matrix_world.copy()
                    o.parent = None
                    o.matrix_world = m
            meshes = [o for o in new if o.type == "MESH"]
            for o in meshes:
                o["held_local"] = [v for row in o.matrix_world for v in row]
            out.append((arm, bone, meshes))
    place(out)
    return out


def place(held):
    """Setter det som holdes etter posen nå (kall etter `scene.frame_set`)."""
    from mathutils import Matrix

    if not held:
        return
    bpy.context.view_layer.update()
    for arm, bone, meshes in held:
        m = arm.matrix_world @ arm.pose.bones[bone].matrix
        for o in meshes:
            v = list(o["held_local"])
            local = Matrix([v[0:4], v[4:8], v[8:12], v[12:16]])
            o.matrix_world = m @ local
