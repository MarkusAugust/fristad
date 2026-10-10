"""Flettverksgjerdet (`wattle_fence`) til gårdene og kirkegården, modellert
i Blender (bpy) og eksportert til assets/models/town/. Resten av gatelivet
lages av streetlife.py. Kjør:

    python3 tools/models/street.py

Et stykke på 2 m langs lokal x, med staker som går 25 cm ned i bakken og
to vidjer som binder toppen. Akser som i town.py: −Y fram, Z opp, origo
midt på grunnflaten."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from common import clear  # noqa: E402
from mathutils import Vector  # noqa: E402

from town import PIECES, beam, block, finish, layer  # noqa: E402

OUT = "assets/models/town"


def wattle_fence():
    """Flettverket er en tett plate med eierens flettverkstekstur (som
    gjerdene rundt kjøkkenhagene), med staker som stikker opp og ned i
    bakken, og to vidjer som binder toppen. Tidligere var vidjene enkeltris,
    og på avstand ble gjerdet bare tynne mørke staver."""
    rng = random.Random(72)
    L, H = 2.0, 1.0
    wattle = layer("wattle")
    stake = layer("timber_grey", (0.85, 0.8, 0.75), 0.5)
    withy = layer("wattle", (0.75, 0.68, 0.6), 0.5)
    # Platen: litt kortere enn stykket, så stakene i endene tar skjøten.
    block("wattle", (L - 0.02, 0.07, H - 0.1), wattle, at=(0, 0, 0.06 + (H - 0.1) / 2), bevel=0.02)
    # Stakene: én i hver ende og én i midten, ujevne på toppen.
    for x in (-L / 2, 0.0, L / 2):
        top = H + rng.uniform(0.04, 0.16)
        beam("stake", (x, 0, -0.25), (x + rng.uniform(-0.015, 0.015), rng.uniform(-0.015, 0.015), top), 0.075, 0.075, stake, roll=rng.uniform(0, 3))
    # Vidjene som binder toppen, flettet foran og bak stakene.
    for k, z in enumerate((H - 0.06, H - 0.02)):
        pts = []
        for i in range(9):
            x = -L / 2 + L * i / 8
            y = 0.045 * math.cos(math.pi * (i / 2 + k))
            pts.append(Vector((x, y, z + rng.uniform(-0.012, 0.012))))
        for a, b in zip(pts, pts[1:]):
            beam("withy", a, b, 0.035, 0.035, withy)


if __name__ == "__main__":
    clear()
    PIECES.clear()
    wattle_fence()
    finish(f"{OUT}/wattle_fence.glb", max_edge=3.0)
