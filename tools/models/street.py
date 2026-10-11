"""Gjerdene, modellert i Blender (bpy) og eksportert til assets/models/town/:
flettverket (`wattle_fence`), skigarden (`rail_fence`, skråstilte staur
mellom par av staur), stakittet (`picket_fence`, kløyvde bord på to
ledere) og den lave steinmuren (`stone_fence`). Byen velger variant per
sperrelinje (`town::enclose`). Resten av gatelivet lages av streetlife.py.
Kjør:

    python3 tools/models/street.py [wattle_fence rail_fence picket_fence stone_fence]

Hvert stykke er 2 m langs lokal x og stokkes i x så det når fram til
veggen; stolpene går 25 cm ned i bakken. Akser som i town.py: −Y fram, Z
opp, origo midt på grunnflaten."""

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


def rail_fence():
    """Skigarden: par av staur bundet sammen, med lange skråstilte staur
    (skiene) lagt mellom parene, som på gårdene."""
    rng = random.Random(31)
    L, H = 2.0, 1.15
    wood = layer("timber_grey", (0.9, 0.86, 0.8), 0.55)
    dark = layer("timber", (0.8, 0.74, 0.66), 0.55)
    withy = layer("wattle", (0.7, 0.62, 0.52), 0.5)
    # Staurparene i endene og midten.
    for x in (-L / 2, 0.0, L / 2):
        for sy in (-1, 1):
            top = H + rng.uniform(-0.03, 0.08)
            beam("staur", (x + rng.uniform(-0.01, 0.01), sy * 0.08, -0.25), (x + rng.uniform(-0.02, 0.02), sy * 0.06, top), 0.07, 0.07, wood, roll=rng.uniform(0, 3))
        # Vidjebåndene som binder paret.
        for z in (0.35, 0.78):
            block("band", (0.1, 0.22, 0.03), withy, at=(x, 0, z + rng.uniform(-0.02, 0.02)), bevel=0.008)
    # Skiene: skrå staur i to lag mellom parene, lagt oppå hverandre.
    for k in range(5):
        x0 = -L / 2 - 0.12 + rng.uniform(-0.03, 0.03)
        x1 = L / 2 + 0.12 + rng.uniform(-0.03, 0.03)
        z0 = 0.1 + k * 0.17 + rng.uniform(-0.02, 0.02)
        z1 = z0 + 0.42
        beam("ski", (x0, 0.0, z0), (x1, 0.0, z1), 0.065, 0.065, dark if k % 2 else wood, roll=rng.uniform(0, 3))


def picket_fence():
    """Stakittet: kløyvde bord med spisse topper på to ledere mellom
    stolpene, ujevne i høyde og litt skjeve."""
    rng = random.Random(44)
    L, H = 2.0, 1.1
    post = layer("timber_grey", (0.85, 0.8, 0.74), 0.55)
    board = layer("planks", (0.9, 0.86, 0.8), 0.6)
    rail = layer("timber", (0.82, 0.76, 0.68), 0.55)
    for x in (-L / 2, L / 2):
        beam("post", (x, 0, -0.25), (x, 0, H + 0.04), 0.1, 0.1, post)
        block("cap", (0.13, 0.13, 0.04), post, at=(x, 0, H + 0.06), bevel=0.01)
    for z in (0.3, 0.85):
        block("rail", (L - 0.1, 0.045, 0.07), rail, at=(0, 0.06, z), along="x", bevel=0.008)
    n = 11
    for i in range(n):
        x = -L / 2 + 0.11 + (L - 0.22) * i / (n - 1)
        top = H - 0.08 + rng.uniform(-0.05, 0.03)
        w = 0.1
        tilt = rng.uniform(-0.03, 0.03)
        # Bordet uten fas (detaljene ligger i teksturen), og en spiss topp
        # av to smalere klosser.
        block("picket", (w, 0.025, top - 0.05), board, at=(x, 0.1, (top + 0.05) / 2), rot=(0, tilt, 0), along="z", bevel=0)
        block("tipa", (w * 0.72, 0.025, 0.06), board, at=(x, 0.1, top + 0.025), rot=(0, tilt, 0), along="z", bevel=0)
        block("tipb", (w * 0.4, 0.025, 0.05), board, at=(x, 0.1, top + 0.07), rot=(0, tilt, 0), along="z", bevel=0)


def stone_fence():
    """Den lave steinmuren: en tørrmur av gråstein med dekkheller på toppen,
    litt ujevn i flukt, og steiner som stikker ut i silhuetten. Steinene er
    malt i laget `wall_stone`."""
    rng = random.Random(58)
    L, H, T = 2.0, 1.0, 0.4
    stone = layer("wall_stone", (0.92, 0.9, 0.86), 0.7)
    dark = layer("wall_stone", (0.72, 0.7, 0.66), 0.7)
    cap = layer("wall_stone", (0.7, 0.69, 0.67), 0.7)
    # Kjernen: litt smalere oppe enn nede, og 20 cm ned i bakken.
    block("core", (L, T, H + 0.2), stone, at=(0, 0, (H - 0.2) / 2), bevel=0.03, segments=2)
    block("core_top", (L - 0.06, T - 0.1, 0.2), dark, at=(0, 0, H - 0.05), bevel=0.03)
    # Steiner som stikker ut på begge sider.
    for k in range(9):
        x = -L / 2 + 0.15 + (L - 0.3) * rng.random()
        z = 0.12 + 0.75 * rng.random()
        sy = rng.choice((-1, 1))
        block("stone", (rng.uniform(0.18, 0.34), rng.uniform(0.1, 0.16), rng.uniform(0.1, 0.17)), dark if k % 3 == 0 else stone, at=(x, sy * (T / 2 - 0.03), z), rot=(rng.uniform(-0.08, 0.08), rng.uniform(-0.06, 0.06), rng.uniform(-0.2, 0.2)), bevel=0.02)
    # Dekkhellene på toppen, lagt litt på skrå, kant i kant.
    n = 5
    for i in range(n):
        w = L / n - 0.015
        x = -L / 2 + (i + 0.5) * L / n
        block("capstone", (w, T + 0.08, rng.uniform(0.07, 0.1)), cap, at=(x, rng.uniform(-0.01, 0.01), H + 0.03 + rng.uniform(-0.01, 0.01)), rot=(rng.uniform(-0.05, 0.05), rng.uniform(-0.03, 0.03), 0), bevel=0.015)


MODELS = {"wattle_fence": wattle_fence, "rail_fence": rail_fence, "picket_fence": picket_fence, "stone_fence": stone_fence}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        MODELS[name]()
        finish(f"{OUT}/{name}.glb", max_edge=3.0)
