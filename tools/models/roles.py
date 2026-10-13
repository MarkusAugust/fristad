"""Rollene til materialene og modellene, skrevet inn i glb-fila som data.

Motoren (`src/adapters/gltf.rs`) leser `extras.fristad` på hvert materiale og
på roten, og gjetter ikke ut fra navnene. Navnereglene står bare her:

- `layer_<lag>[.variant]`: teksturlaget `<lag>` etter modellens uv i meter;
  `.paint` får lakkfargen fra instansen, `.magic` gløder hele døgnet.
- `paint…`: fargen fra instansen.
- `glow…`: lyser om natta (`lamp`). `fire…`: brenner hele døgnet.
- `glass…`: speiler; `glass_day` slipper dagslyset inn, `glass_lit` lyser ut
  om natta.
- `water…`: vann. `magic…`: gløder i sin egen farge hele døgnet;
  `magic_paint…` i fargen fra instansen.
- `leaf…`, `needle…`, `grass…`: alfakort som svaier; gresset mørkere mot rota.
- Modeller som heter `kit_…` eller `crypt_NN…` står under bakken (ingen vær),
  `…_leaves`/`…_needles` får glisne kort på avstand, `…_trunk` et grovt nett.

Alt annet er et vanlig materiale (`plain`).
"""

import os

KINDS = ("plain", "lamp", "glass", "water", "magic")
# Teksturlagene som finnes: én mappe per lag (`Layer` i geometry.rs).
TEXTURES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "textures")
FLAGS = ("paint", "glow_always", "glow_lamp", "no_weather", "alpha", "wind", "grass")


def underground(model):
    rest = model[len("crypt_"):] if model.startswith("crypt_") else ""
    return model.startswith("kit_") or rest[:1].isdigit()


def material(name, model):
    """Rollen til materialet `name` i modellen `model`: {kind, layer, flags}."""
    named = name.startswith
    kind, layer, flags = "plain", None, []
    if named("layer_"):
        layer = name[len("layer_"):].split(".")[0]
        if not layer or not os.path.isdir(os.path.join(TEXTURES, layer)):
            raise ValueError(f"{model}: materialet {name!r} viser til teksturlaget {layer!r}, som ikke finnes i assets/textures")
        if name.endswith(".paint"):
            flags.append("paint")
        if ".magic" in name:
            flags.append("glow_always")
    if named("paint"):
        flags.append("paint")
    if named("glow"):
        kind = "lamp"
    if named("glass"):
        kind = "glass"
    if named("glass_day"):
        flags.append("glow_always")
    if named("glass_lit"):
        flags.append("glow_lamp")
    if named("water"):
        kind = "water"
    if underground(model):
        flags.append("no_weather")
    if named("fire"):
        kind = "lamp"
        flags.append("glow_always")
    if named("magic"):
        kind = "magic"
        flags.append("glow_always")
        # `magic_paint…`: gløder i fargen fra instansen (tokenene).
        if named("magic_paint"):
            flags.append("paint")
    if named("leaf") or named("needle") or named("grass"):
        flags += ["alpha", "wind", "no_weather"]
    if named("grass"):
        flags.append("grass")
    flags = sorted(set(flags), key=FLAGS.index)
    return {"kind": kind, "layer": layer, "flags": flags}


def model(name):
    """Rollene til hele modellen: hvordan den forenkles på avstand."""
    if name.endswith("_leaves") or name.endswith("_needles"):
        return {"far": "cards"}
    if name.endswith("_trunk"):
        return {"far_cell": 0.6}
    return {}
