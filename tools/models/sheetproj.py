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
