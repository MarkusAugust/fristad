"""Forbilder lagt på modellene: eierens ark som materiale (uv-laget «src»)
med bakgrunnen fylt fra innsida av figuren, så flater som stikker litt
utenfor omrisset ikke blir grå. Brukes av `monsters.py` og `arms.py`."""

import os

import bpy

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def sheet_material(name, path, rough=0.8):
    """Et materiale som viser forbildet etter uv-laget «src»."""
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = rough
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = bpy.data.images.load(path)
    tex.interpolation = "Cubic"
    uvn = nt.nodes.new("ShaderNodeUVMap")
    uvn.uv_map = "src"
    nt.links.new(uvn.outputs[0], tex.inputs[0])
    nt.links.new(tex.outputs[0], bsdf.inputs["Base Color"])
    return m


def filled_sheet(path):
    """Arket med bakgrunnen fylt med fargen til nærmeste piksel av figuren,
    så en flate som stikker litt utenfor omrisset ikke blir grå."""
    import numpy as np
    from PIL import Image
    from scipy import ndimage
    out = os.path.join(ROOT, "target", "sheets", os.path.basename(path).rsplit(".", 1)[0] + "_fill.png")
    if os.path.exists(out) and os.path.getmtime(out) > os.path.getmtime(path):
        return out
    im = np.asarray(Image.open(path).convert("RGB")).astype(np.int32)
    bg = im[4, 4]
    fg = np.abs(im - bg).sum(2) > 36
    fg = ndimage.binary_opening(fg, iterations=2)
    # Grå partier inni figuren ligner bakgrunnen; de er figur så lenge de
    # er omsluttet av den.
    fg = ndimage.binary_fill_holes(ndimage.binary_closing(fg, iterations=4))
    # Fyllet tar fargen fra innsida av figuren, ikke fra den mørke streken
    # rundt (da blir kanten svarte stråler).
    inner = ndimage.binary_erosion(fg, iterations=4) & (im.sum(2) > 3 * 70)
    _, (iy, ix) = ndimage.distance_transform_edt(~inner, return_indices=True)
    keep = ndimage.binary_erosion(fg, iterations=6)
    iy = np.where(keep, np.arange(im.shape[0])[:, None], iy)
    ix = np.where(keep, np.arange(im.shape[1])[None, :], ix)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    Image.fromarray(im[iy, ix].astype(np.uint8)).save(out)
    return out


def blended_sheet_material(name, path, rough=0.8, soft=(0.55, 0.75), front_attr=None):
    """Som `sheet_material`, men med to uv-lag: «src» forfra eller bakfra
    og «src_side» fra siden, blandet etter hvilken vei flaten vender
    (|n_y| mot |n_x| i objektets rom), mykt mellom vinklene i `soft`. Da
    ser man aldri profilen og ansiktet forfra side om side på samme hode
    (innbyggerne i `townsfolk.py`)."""
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = rough
    img = bpy.data.images.load(path)
    texs = []
    for uv_name in ("src", "src_side"):
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = img
        tex.interpolation = "Cubic"
        uvn = nt.nodes.new("ShaderNodeUVMap")
        uvn.uv_map = uv_name
        nt.links.new(uvn.outputs[0], tex.inputs[0])
        texs.append(tex)
    # Hvor mye flaten vender til siden: |n_y| / (|n_x| + |n_y|).
    geo = nt.nodes.new("ShaderNodeNewGeometry")
    xf = nt.nodes.new("ShaderNodeVectorTransform")
    xf.vector_type = "NORMAL"
    xf.convert_from = "WORLD"
    xf.convert_to = "OBJECT"
    nt.links.new(geo.outputs["Normal"], xf.inputs[0])
    sep = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(xf.outputs[0], sep.inputs[0])

    def op(kind, a, b=None):
        n = nt.nodes.new("ShaderNodeMath")
        n.operation = kind
        for i, v in enumerate((a, b)):
            if v is None:
                continue
            if isinstance(v, (int, float)):
                n.inputs[i].default_value = v
            else:
                nt.links.new(v, n.inputs[i])
        return n.outputs[0]

    ax, ay = op("ABSOLUTE", sep.outputs["X"]), op("ABSOLUTE", sep.outputs["Y"])
    side = op("DIVIDE", ay, op("ADD", op("ADD", ax, ay), 1e-4))
    ramp = nt.nodes.new("ShaderNodeMapRange")
    ramp.interpolation_type = "SMOOTHSTEP"
    ramp.inputs["From Min"].default_value = soft[0]
    ramp.inputs["From Max"].default_value = soft[1]
    nt.links.new(side, ramp.inputs["Value"])
    factor = ramp.outputs[0]
    if front_attr:
        # En vekt per hjørne (`front_attr`, 0–1) holder bildet forfra der
        # den er 1, med myk overgang (innbyggernes ansikt, `townsfolk.py`).
        at = nt.nodes.new("ShaderNodeAttribute")
        at.attribute_name = front_attr
        factor = op("MULTIPLY", factor, op("SUBTRACT", 1.0, at.outputs["Fac"]))
    mix = nt.nodes.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    nt.links.new(factor, mix.inputs["Factor"])
    nt.links.new(texs[0].outputs[0], mix.inputs["A"])
    nt.links.new(texs[1].outputs[0], mix.inputs["B"])
    nt.links.new(mix.outputs["Result"], bsdf.inputs["Base Color"])
    return m
