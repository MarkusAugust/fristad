"""Baker teksturer på modellene fra street.py, plaza.py, house.py og
gorvek.py: bygger hver modell på nytt, legger treårer og slitasje på
materialene, og baker albedo × AO, normalkart og ruhet/metall til ett
atlas per modell (common.bake_textured). Filene skrives over der de
ligger, så motoren bruker dem som før. Kjør:

    python3 tools/models/bake_models.py [street|plaza|house|gorvek ...] [modellnavn ...] [størrelse]
"""

import importlib
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import bake_textured, clear  # noqa: E402

# Teksturstørrelse per modell; resten får 512.
SIZES = {"lamp": 1024, "bicycle": 1024, "market_stall": 1024, "bench": 1024, "throne": 1024, "bed": 1024, "bearskin": 1024, "table": 1024}
# Hvor høyt skitten går (m) per modul; vegg-ting i gorvek har origo i veggen.
DIRT = {"street": (0.0, 0.5, 0.4), "plaza": (0.0, 0.4, 0.35), "house": (-0.2, 0.2, 0.2), "gorvek": (0.0, 0.3, 0.3)}


def grain(mat, scale=40.0, strength=0.3):
    """Treårer og slitasje i ett: fin, strukket støy i både farge og bump."""
    nodes, links = mat.node_tree.nodes, mat.node_tree.links
    bsdf = nodes["Principled BSDF"]
    coord = nodes.new("ShaderNodeTexCoord")
    mapping = nodes.new("ShaderNodeMapping")
    mapping.inputs["Scale"].default_value = (scale * 0.15, scale, scale)
    links.new(coord.outputs["Object"], mapping.inputs["Vector"])
    noise = nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = 1.0
    noise.inputs["Detail"].default_value = 5.0
    noise.inputs["Roughness"].default_value = 0.65
    links.new(mapping.outputs["Vector"], noise.inputs["Vector"])
    bump = nodes.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = strength
    bump.inputs["Distance"].default_value = 0.003
    links.new(noise.outputs["Fac"], bump.inputs["Height"])
    links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])
    # Fargen varierer med årene.
    src = bsdf.inputs["Base Color"].links[0].from_socket if bsdf.inputs["Base Color"].is_linked else None
    if src is None:
        rgb = nodes.new("ShaderNodeRGB")
        rgb.outputs[0].default_value = bsdf.inputs["Base Color"].default_value[:]
        src = rgb.outputs[0]
    fac = nodes.new("ShaderNodeMapRange")
    links.new(noise.outputs["Fac"], fac.inputs["Value"])
    fac.inputs["To Min"].default_value = 0.8
    fac.inputs["To Max"].default_value = 1.12
    gray = nodes.new("ShaderNodeCombineColor")
    for ch in ("Red", "Green", "Blue"):
        links.new(fac.outputs["Result"], gray.inputs[ch])
    mul = nodes.new("ShaderNodeMix")
    mul.data_type = "RGBA"
    mul.blend_type = "MULTIPLY"
    mul.inputs["Factor"].default_value = 1.0
    links.new(src, mul.inputs["A"])
    links.new(gray.outputs[0], mul.inputs["B"])
    links.new(mul.outputs["Result"], bsdf.inputs["Base Color"])


def dress(module):
    """Legger årer på tre, grove bump på pels/ull/stein, og skitt nederst."""
    from common import weather
    lo, hi, strength = DIRT[module]
    for mat in bpy.data.materials:
        n = mat.name
        if not mat.use_nodes or n.startswith(("glass", "glow", "fire", "mesh", "paint")):
            continue
        if any(k in n for k in ("oak", "wood", "timber", "plank")):
            grain(mat, 40.0, 0.35)
        elif any(k in n for k in ("fur", "wool", "leather", "hide", "canvas", "stripe")):
            grain(mat, 25.0, 0.6)
        elif any(k in n for k in ("stone", "iron", "rust", "bronze", "clay", "brick")):
            grain(mat, 60.0, 0.25)
        weather(mat, lo, hi, strength, 12.0)


if __name__ == "__main__":
    args = sys.argv[1:]
    modules = [a for a in args if a in DIRT] or list(DIRT)
    size = next((int(a) for a in args if a.isdigit()), None)
    names = [a for a in args if a not in DIRT and not a.isdigit()]
    for module in modules:
        mod = importlib.import_module(module)
        for name, build in mod.BUILDERS.items():
            if names and name not in names:
                continue
            clear()
            build(mod.mats())
            dress(module)
            path = f"{mod.OUT}/{name}.glb"
            os.makedirs(mod.OUT, exist_ok=True)
            bake_textured(path, size or SIZES.get(name, 512), keep=("glass", "glow", "fire", "mesh"))
