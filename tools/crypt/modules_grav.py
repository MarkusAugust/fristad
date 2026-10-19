"""Modulene for gravkjelleren (`docs/nivaaredigering.md`, del 21): skriver
`assets/levels/moduler/grav/<navn>.json` i fast form. Modulene er data
eieren setter sammen som Lego; dette skriptet er bare måten de første ble
skrevet på (de kan like gjerne endres for hånd).

Hver modul har cellene i sitt eget rom (midtcella i (0, 0)), rommet med
stemningen, standardinnholdet og lysene. Hver ytre side som ikke er lukket,
er en åpning (`auto`: dør mot et rom, gangåpning mot en gang); `walls`
lukker en side, og `openings` gir en åpning et annet slag.

    python3 tools/crypt/modules_grav.py
"""

import json
import os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "assets", "levels", "moduler", "grav")


def grid(bit, cols, rows, i0=0, j0=0, rot=0, variant=None):
    """Cellene i et rektangel: id `c<kolonne><rad>`, fra nordvest."""
    out = []
    for r in range(rows):
        for c in range(cols):
            cell = {"id": f"c{c}{r}", "bit": bit, "at": [i0 + c, 0, j0 + r], "rot": rot}
            if variant:
                cell["variant"] = variant
            out.append(cell)
    return out


def module(name, label, kind, about, cells, torches, **extra):
    room = {"id": "rom", "name": label, "theme": "grav", "mood": {"torches": torches}}
    if extra.pop("secret", False):
        room["secret"] = True
    m = {"id": f"grav/{name}", "name": label, "theme": "grav", "kind": kind, "about": about, "rooms": [room], "cells": cells}
    m.update(extra)
    return name, m


MODULES = [
    # Rommene: romfelt med krysshvelv, søyler der fire felt møtes.
    module("rom_1x1", "Rom 1 × 1", "rom", "Et lite rom på 4 × 4 m med krysshvelv. Åpning midt på hver side.", grid("grav/romfelt", 1, 1), 0.5),
    module("rom_1x2", "Rom 1 × 2", "rom", "Et smalt rom på 4 × 8 m, to fag med gjordbue imellom.", grid("grav/romfelt", 1, 2), 0.5),
    module("rom_2x2", "Rom 2 × 2", "rom", "Et rom på 8 × 8 m, fire fag med én søyle i midten.", grid("grav/romfelt", 2, 2), 0.5),
    module("rom_3x2", "Rom 3 × 2", "rom", "En sal på 12 × 8 m, seks fag med to søyler.", grid("grav/romfelt", 3, 2, -1, 0), 0.4),
    module("rom_3x3", "Rom 3 × 3 med søyler", "rom", "En sal på 12 × 12 m, ni fag med fire søyler.", grid("grav/romfelt", 3, 3, -1, -1), 0.4),
    # Gangene.
    module("gang_1", "Gang", "gang", "Rett gang på 4 m med tønnehvelv, åpen i begge ender.", grid("grav/gang", 1, 1), 0.4),
    module("gang_2", "Gang 2", "gang", "Rett gang på 8 m, åpen i begge ender.", grid("grav/gang", 1, 2), 0.4),
    module("gang_hjorne", "Gang, hjørne", "gang", "Gangen svinger 90°.", grid("grav/gang_hjorne", 1, 1), 0.4),
    module("gang_t", "Gang, T", "gang", "Gangen deler seg i tre.", grid("grav/gang_t", 1, 1), 0.4),
    module("gang_kryss", "Gang, kryss", "gang", "Fire ganger møtes.", grid("grav/gang_kryss", 1, 1), 0.4),
    module("blindgang", "Blindgang", "gang", "Gangen ender i en mur.", grid("grav/gang_blind", 1, 1), 0.3),
    # Trappene.
    module("halvtrapp", "Halvtrapp (1 m)", "trapp", "Trapp 1 m opp mot nord over én celle, under et skrått tønnehvelv.", grid("grav/halvtrapp", 1, 1), 0.4),
    module("halv_etasje", "Halv etasje (2 m)", "trapp", "Trapp 2 m opp mot nord over én celle.", grid("grav/trapp2", 1, 1), 0.4),
    module("nedgang", "Nedgang", "trapp", "Romfeltet med trappa ned til neste nivå. Står det inntil et rom, åpnes det med en bue.",
           grid("grav/nedgang", 1, 1), 0.3,
           openings=[{"at": [0, 0], "side": s, "kind": "bue"} for s in ("E", "S", "W")],
           things=[{"id": "ned", "kind": "down", "anchor": "celle:c00"}]),
    # De sammensatte.
    module("rotunde", "Rotunde", "rom", "Rundt rom på 12 × 12 m med kuppel, fire nisjer med fakler og en gang ut midt på hver side.",
           [{"id": "c", "bit": "grav/rotunde", "at": [0, 0, 0], "rot": 0}], 1.0,
           lights=[{"id": "kuppel", "anchor": "celle:c", "pos": [0, 5.5, 0], "color": [0.55, 0.6, 0.8], "range": 9, "flicker": False}]),
    # Ferdige rom.
    module("kapell", "Kapellet med døra opp", "ferdig",
           "Inngangen til nivået: en sal på 3 × 3 fag med fire søyler, døra opp midt på nordveggen og startstedet midt i.",
           grid("grav/romfelt", 3, 3, -1, -1), 0.5,
           walls=[{"at": [0, -1], "side": "N"}],
           things=[{"id": "start", "kind": "start", "anchor": "celle:c11", "pos": [0, 0, 0], "yaw": -90},
                   {"id": "opp", "kind": "up", "anchor": "celle:c10", "pos": [0, 0, -0.45], "yaw": 0}]),
    module("gravgang", "Gravgangen", "ferdig",
           "En gang på 12 m med gravheller og gjordbuer, og en knapp som går vakt fram og tilbake.",
           grid("grav/gang", 1, 3, 0, -1, variant="b"), 0.6,
           things=[{"id": "vakt", "kind": "mob", "anchor": "celle:c01", "pos": [0, 0, -3], "yaw": 0, "label": "button",
                    "patrol": [[0, 0, -3.5], [0, 0, 3.5]]}]),
    module("skattkammer", "Skattkammeret", "ferdig",
           "Et hemmelig rom på 4 × 4 m med en kiste. Inngangen er en falsk vegg med sprekker der kammeret møter et annet rom.",
           grid("grav/romfelt", 1, 1), 0.0, secret=True,
           openings=[{"at": [0, 0], "side": s, "kind": "falsk"} for s in ("N", "E", "S", "W")],
           things=[{"id": "kiste", "kind": "chest", "anchor": "celle:c00", "pos": [-0.6, 0, 0], "yaw": 90}],
           lights=[{"id": "lys", "anchor": "celle:c00", "pos": [0, 2.5, 0], "color": [1.0, 0.62, 0.3], "range": 6, "flicker": True}]),
]


def write(name, m):
    """Fast form: feltene i rekkefølge, én ting per linje i listene."""
    lines = ["{"]
    keys = list(m.keys())
    for k, key in enumerate(keys):
        v = m[key]
        comma = "," if k < len(keys) - 1 else ""
        if isinstance(v, list) and v and isinstance(v[0], dict):
            lines.append(f'  "{key}": [')
            for n, item in enumerate(v):
                lines.append("    " + json.dumps(item, ensure_ascii=False) + ("," if n < len(v) - 1 else ""))
            lines.append("  ]" + comma)
        else:
            lines.append(f'  "{key}": {json.dumps(v, ensure_ascii=False)}{comma}')
    lines.append("}")
    with open(os.path.join(OUT, f"{name}.json"), "w") as f:
        f.write("\n".join(lines) + "\n")


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, m in MODULES:
        write(name, m)
    print(f"{len(MODULES)} moduler i {os.path.relpath(OUT, ROOT)}")


if __name__ == "__main__":
    main()
