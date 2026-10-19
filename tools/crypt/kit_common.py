"""Standardmålene for nivåredigeringen (docs/nivaaredigering.md, del 3) og
manifestet byggesettet skriver for kjernen (`KitDef`,
`crates/fristad-domain/src/domain/world/crypt/kit.rs`).

De samme tallene står i kjernen (`compose/grid.rs`); testene der og
`kit_cells.py --check` sjekker at de stemmer.

Akser som i spillet: x mot øst, y opp, z mot sør, i meter.

- En **celle** er 4 × 4 m med midten i origo (gulvet i y = 0).
- En **halvkant** står med origo midt på kantlinja og går 0,4 m inn i cella
  mot +z (murflaten i z = 0,4 vender mot +z, inn i rommet).
- Et **kvarthjørne** står med origo i hjørnet og fyller ruta 0,4 × 0,4 m mot
  +x og +z.
- En **søyle** står med origo i hjørnet der fire celler møtes.
"""

import json
import math
import os

CELL = 4.0
HALF = CELL / 2
WALL_T = 0.4  # halvkanten: halvparten av muren på 0,8 m
FACE = HALF - WALL_T  # murflaten fra cellemidten (1,6)
WALL_TOP = 5.5  # murene, hjørnene og overmurene går så høyt over gulvet
WALL_FOOT = -0.05  # og så langt ned
FLOOR_T = 0.5  # gulvskiva
NAV_INSET = 0.4

# Hvelvene: krysshvelv i rommene (vederlaget ROOM_S, pilhøyden ROOM_R over
# halve cella) og tønnehvelv i gangene og trappene (ellipse over halve
# cella, så profilen er 2,6 m høy ved murflaten og 3,6 m i toppen).
ROOM_S, ROOM_R = 3.0, 2.0
GANG_S, GANG_R = 1.1, 2.5

# Åpningene, felles for alle temaer.
DOOR_W, DOOR_SPRING = 1.6, 1.5  # toppen i 2,3 (`SHORTCUT_H`)
GATE_W, GATE_H = 2.8, 3.0
SECRET_W, SECRET_H = 2.0, 2.6

# Fri høyde over det gangbare (romhøyden i nivåfila).
ROOM_CLEAR = 4.0
GANG_CLEAR = 3.05


def gang_h(u):
    """Høyden på gangprofilen u meter fra midten."""
    return GANG_S + GANG_R * math.sqrt(max(0.0, 1.0 - (u / HALF) ** 2))


def room_h(u):
    """Høyden på buen i krysshvelvet langs en kant, u meter fra midten."""
    return ROOM_S + ROOM_R * math.sqrt(max(0.0, 1.0 - (u / HALF) ** 2))


def vault_ts():
    """Prøvene langs hvelvene (t i [−1, 1] over halve cella): tettere mot
    foten, og med punktene ved murflaten (±0,8), så gangprofilen i
    overmuren og tønnehvelvet har de samme hjørnene og ingen sprekk."""
    ts = {round(-math.cos(math.pi * k / 12), 9) for k in range(13)}
    ts |= {-FACE / HALF, FACE / HALF}
    return sorted(ts)


SIDES = ["N", "E", "S", "W"]


def rounded(v):
    """Tallene avrundet til 0,1 mm, som i manifestet (og `extras.kit`)."""
    if isinstance(v, float):
        return round(v, 4) + 0.0
    if isinstance(v, (list, tuple)):
        return [rounded(x) for x in v]
    if isinstance(v, dict):
        return {k: rounded(x) for k, x in v.items()}
    return v


def write_manifest(path, theme, version, pieces):
    """Manifestet for et tema: bitene med flatene, kolliderne og det
    gangbare, slik kjernen leser det (`KitDef`)."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    lines = [json.dumps(rounded(p), ensure_ascii=False, separators=(",", ":")) for p in pieces]
    text = "{\n" + f'  "theme": "{theme}",\n  "version": {version},\n  "pieces": [\n    ' + ",\n    ".join(lines) + "\n  ]\n}\n"
    with open(path, "w") as f:
        f.write(text)
