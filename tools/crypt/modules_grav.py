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


# Høyden på lokket til sarkofagen og skrinet (`kit_sarcophagus`, `scroll_chest`), der lysene står.
SARCOPHAGUS_TOP = 1.14
SCROLL_CHEST_TOP = 0.59


def prop(id_, model, cell, pos, yaw, size=None):
    """Pynt i en celle: en modell fra byggesettet eller møblene
    (`crypt::PROPS`), med fotavtrykket i `size` (bredde og dybde i modellens
    rom) som hull i navigasjonsnettet."""
    t = {"id": id_, "kind": "prop", "anchor": f"celle:{cell}", "pos": pos, "yaw": yaw, "label": model}
    if size:
        t["size"] = size
    return t


def module(name, label, kind, about, cells, torches, **extra):
    room = {"id": "rom", "name": label, "theme": "grav", "mood": {"torches": torches}}
    if extra.pop("secret", False):
        room["secret"] = True
    m = {"id": f"grav/{name}", "name": label, "theme": "grav", "kind": kind, "about": about, "rooms": [room] + extra.pop("rooms_extra", []), "cells": cells}
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
    # Kuppelen var for mørk (eieren 19. oktober): ei hjulkrone henger fra
    # sluttsteinen (kroken i toppen av kuppelen, `ROT_CROWN` 7,6 m; hjulet
    # 3,2 m under), og lyset fra den og et svakt skjær i toppen når hvelvet.
    module("rotunde", "Rotunde", "rom", "Rundt rom på 12 × 12 m med kuppel, ei hjulkrone under sluttsteinen, fire nisjer med fakler og en gang ut midt på hver side.",
           [{"id": "c", "bit": "grav/rotunde", "at": [0, 0, 0], "rot": 0}], 1.0,
           things=[prop("krone", "kit_chandelier", "c", [0, 7.55, 0], 0)],
           lights=[{"id": "krone", "anchor": "celle:c", "pos": [0, 4.6, 0], "color": [1.0, 0.66, 0.34], "range": 10, "flicker": True, "intensity": 3.0},
                   {"id": "kuppel", "anchor": "celle:c", "pos": [0, 6.6, 0], "color": [0.85, 0.72, 0.6], "range": 7, "flicker": False}]),
    # Ferdige rom.
    module("kapell", "Kapellet med døra opp", "ferdig",
           "Inngangen til nivået: et kapell på 3 × 3 fag med fire søyler, døra opp midt på nordveggen, det frittstående alteret "
           "med lys under midthvelvet og benkerader vendt mot alteret. Startstedet er i midtgangen bak benkene.",
           grid("grav/romfelt", 3, 3, -1, -1), 0.25,
           walls=[{"at": [0, -1], "side": "N"}],
           things=[{"id": "start", "kind": "start", "anchor": "celle:c11", "pos": [-1.3, 0, 0], "yaw": -90},
                   {"id": "opp", "kind": "up", "anchor": "celle:c10", "pos": [0, 0, -0.45], "yaw": 0},
                   # Alteret står fritt midt i kapellet, så det aldri stenger en åpning.
                   prop("alter", "altar", "c11", [0.4, 0, 0], 90, [1.5, 0.95])]
                  + [prop(f"benk_{n}", "bench_16", "c01", [x, 0, z], 90, [1.6, 0.34]) for n, (x, z) in enumerate((x, z) for x in (0.3, 1.5) for z in (-1.75, 1.75))],
           lights=[{"id": "alterlys", "anchor": "celle:c11", "pos": [-0.3, 2.0, 0], "color": [1.0, 0.68, 0.35], "range": 7, "flicker": True, "intensity": 3.0},
                   {"id": "lys_n", "anchor": "celle:c11", "pos": [0.4, 1.9, -0.5], "color": [1.0, 0.68, 0.35], "range": 4.5, "flicker": True},
                   {"id": "lys_s", "anchor": "celle:c11", "pos": [0.4, 1.9, 0.5], "color": [1.0, 0.68, 0.35], "range": 4.5, "flicker": True}]),
    module("gravgang", "Gravgangen", "ferdig",
           "En gang på 12 m med gravnisjer og gjordbuer, sarkofager annenhver side med lys på lokket, og en knapp som går vakt fram og tilbake.",
           grid("grav/gang", 1, 3, 0, -1, variant="b"), 0.6,
           things=[{"id": "vakt", "kind": "mob", "anchor": "celle:c01", "pos": [0, 0, -3], "yaw": 0, "label": "button",
                    "patrol": [[0, 0, -3.5], [0, 0, 3.5]]}]
                  + [prop(f"sarkofag_{c}", "kit_sarcophagus", c, [x, 0, 0], 0, [1.04, 2.14]) for c, x in (("c00", -1.03), ("c01", 1.03), ("c02", -1.03))]
                  + [prop(f"lys_{c}", "candle", c, [x, SARCOPHAGUS_TOP, 0.65], 0) for c, x in (("c00", -1.03), ("c02", -1.03))],
           lights=[{"id": f"lys_{c}", "anchor": f"celle:{c}", "pos": [x, SARCOPHAGUS_TOP + 0.35, 0.65], "color": [1.0, 0.66, 0.32], "range": 3.5, "flicker": True}
                   for c, x in (("c00", -1.03), ("c02", -1.03))]),
    # Mekanismene som moduler (eieren 19. oktober): hver virker alene, med
    # koblingene inni modulen (spak → port, nøkkel → port, slåsiden på
    # snarveien), så den kan dras ut og settes sammen uten mer.
    module("porthall", "Porthallen med spak", "ferdig",
           "To rom på 4 × 4 m med et portgitter imellom. Spaken i forrommet heiser gitteret, og ei kiste står i hallen bak.",
           [{"id": "hall", "bit": "grav/romfelt", "at": [0, 0, 0], "rot": 0},
            {"id": "forrom", "bit": "grav/romfelt", "at": [0, 0, 1], "rot": 0, "room": "forrom"}], 0.5,
           rooms_extra=[{"id": "forrom", "name": "Forrommet", "theme": "grav", "mood": {"torches": 0.5}}],
           things=[{"id": "port", "kind": "gate", "anchor": "kant:forrom/N"},
                   {"id": "spak", "kind": "lever", "anchor": "celle:forrom", "pos": [-1.2, 0, 0.7], "yaw": 90},
                   {"id": "kiste", "kind": "chest", "anchor": "celle:hall", "pos": [0, 0, -1.0], "yaw": 0}],
           walls=[{"at": [0, 0], "side": "N"}],
           links=[{"from": "spak", "to": "port", "kind": "opens"}]),
    module("nokkelport", "Nøkkelporten", "ferdig",
           "En vaktstue på 4 × 8 m der en elite går vakt med nøkkelen, og et portgitter inn til et lite kammer med ei kiste. Porten åpnes med nøkkelen eliten bærer.",
           [{"id": "stue_n", "bit": "grav/romfelt", "at": [0, 0, 0], "rot": 0},
            {"id": "stue_s", "bit": "grav/romfelt", "at": [0, 0, 1], "rot": 0},
            {"id": "kammer", "bit": "grav/romfelt", "at": [1, 0, 0], "rot": 0, "room": "kammer"}], 0.5,
           rooms_extra=[{"id": "kammer", "name": "Kammeret", "theme": "grav", "mood": {"torches": 0.3}}],
           things=[{"id": "port", "kind": "gate", "anchor": "kant:stue_n/E"},
                   {"id": "vokter", "kind": "elite", "anchor": "celle:stue_n", "pos": [0, 0, -0.5], "yaw": 180, "label": "button",
                    "patrol": [[0, 0, -0.5], [0, 0, 4.5]]},
                   {"id": "kiste", "kind": "chest", "anchor": "celle:kammer", "pos": [0.9, 0, 0], "yaw": -90}],
           walls=[{"at": [1, 0], "side": "N"}, {"at": [1, 0], "side": "E"}, {"at": [1, 0], "side": "S"}],
           links=[{"from": "vokter", "to": "port", "kind": "key"}]),
    module("snarvei", "Snarveien", "ferdig",
           "To rom på 4 × 4 m med ei snarveidør imellom. Slåen sitter i det nordre rommet, så døra åpnes derfra og står åpen etterpå.",
           [{"id": "innerst", "bit": "grav/romfelt", "at": [0, 0, 0], "rot": 0},
            {"id": "ytterst", "bit": "grav/romfelt", "at": [0, 0, 1], "rot": 0, "room": "ytterst"}], 0.5,
           rooms_extra=[{"id": "ytterst", "name": "Ytterst", "theme": "grav", "mood": {"torches": 0.5}}],
           things=[{"id": "dor", "kind": "shortcut", "anchor": "kant:ytterst/N"}],
           links=[{"from": "dor", "to": "rom", "kind": "from"}]),
    module("fellegang", "Fellegangen", "ferdig",
           "En gang på 8 m med ei trykkplate midt i. Tråkker du på den, skytes piler tvers over gangen fra muren.",
           grid("grav/gang", 1, 2), 0.4,
           things=[{"id": "plate", "kind": "trap", "anchor": "celle:c01", "pos": [0, 0, -1.6], "yaw": 90, "label": "arrows", "size": [1.2, 1.2]}]),
    module("hvilested", "Hvilestedet", "ferdig",
           "Et lite kapell på 4 × 4 m med hvilestedet mot nordmuren og en lapp fra munkene. Her tennes lyset, og her våkner du.",
           grid("grav/romfelt", 1, 1), 0.6,
           walls=[{"at": [0, 0], "side": "N"}],
           things=[{"id": "hvile", "kind": "rest", "anchor": "celle:c00", "pos": [0, 0, -1.0], "yaw": 0},
                   {"id": "lapp", "kind": "lore", "anchor": "celle:c00", "pos": [1.15, 0, -1.1], "yaw": -45, "label": "Et blad fra munkene",
                    "text": "Den som tenner lyset her, våkner her. Fyll lykta før du går dypere; mørket der nede er ikke tomt."}],
           lights=[{"id": "hvilelys", "anchor": "celle:c00", "pos": [0, 1.6, -1.0], "color": [1.0, 0.7, 0.4], "range": 5, "flicker": True}]),
    # Bosshallen (bossene er 4–5 m høye): hallen på 10 × 10 m med hvelvet
    # til 10 m, og et forrom med arenaporten, som slår igjen bak deg til
    # bossen faller.
    module("bosshall", "Bosshallen", "ferdig",
           "En hall på 10 × 10 m med krysshvelv til 10 m, søyler i hjørnene og en gang ut midt på hver side. Bossen venter i hallen, "
           "og portgitteret mellom forrommet og hallen slår igjen bak deg til den faller.",
           [{"id": "hall", "bit": "grav/hall", "at": [0, 0, 0], "rot": 0},
            {"id": "forrom", "bit": "grav/romfelt", "at": [0, 0, 2], "rot": 0, "room": "forrom"}], 0.8,
           rooms_extra=[{"id": "forrom", "name": "Forrommet", "theme": "grav", "mood": {"torches": 0.5}}],
           things=[{"id": "port", "kind": "gate", "anchor": "kant:forrom/N"},
                   {"id": "boss", "kind": "boss", "anchor": "celle:hall", "pos": [0, 0, -2.0], "yaw": 0, "label": "modal"},
                   prop("krone", "kit_chandelier", "hall", [0, 9.95, 0], 0)],
           links=[{"from": "port", "to": "rom", "kind": "arena"}],
           lights=[{"id": "krone", "anchor": "celle:hall", "pos": [0, 6.9, 0], "color": [1.0, 0.68, 0.38], "range": 12, "flicker": True, "intensity": 3.0},
                   {"id": "hvelv", "anchor": "celle:hall", "pos": [0, 8.8, 0], "color": [0.9, 0.75, 0.6], "range": 8, "flicker": False}]
           # Lyset fra hjørnene opp i hvelvkappene (eieren 20. oktober: hvelvene var svarte).
           + [{"id": f"kappe{k}", "anchor": "celle:hall", "pos": [sx * 3.3, 6.2, sz * 3.3], "color": [1.0, 0.7, 0.42], "range": 7, "flicker": False, "intensity": 1.4}
              for k, (sx, sz) in enumerate(((-1, -1), (1, -1), (1, 1), (-1, 1)))]),
    module("skattkammer", "Skattkammeret", "ferdig",
           "Et hemmelig rom på 4 × 4 m med en kiste midt i, skrin, kister og sekker i hjørnene og en rustning på stativ. "
           "Inngangen er en falsk vegg med sprekker der kammeret møter et annet rom.",
           grid("grav/romfelt", 1, 1), 0.0, secret=True,
           openings=[{"at": [0, 0], "side": s, "kind": "falsk"} for s in ("N", "E", "S", "W")],
           things=[{"id": "kiste", "kind": "chest", "anchor": "celle:c00", "pos": [-0.6, 0, 0], "yaw": 90},
                   prop("skrin", "scroll_chest", "c00", [1.1, 0, -1.25], 0, [0.95, 0.6]),
                   prop("kiste_so", "chest_forms", "c00", [1.25, 0, 1.0], 90, [1.15, 0.65]),
                   prop("rustning", "kit_armor", "c00", [-1.2, 0, -1.2], 45, [0.8, 0.6]),
                   prop("sekker", "sacks", "c00", [-1.15, 0, 1.15], 0, [1.0, 1.15]),
                   prop("lys", "candle", "c00", [1.3, SCROLL_CHEST_TOP, -1.35], 0)],
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
