"""Modulene for de fire andre temaene (`docs/nivaaredigering.md`, del 24):
kloakken, gruva, borgen og kjernen. Formene er de samme som gravkjellerens
(rom, ganger, trapper, nedgangen), bygd av de samme bitene, og temaet gir
steinen (`assets/levels/steiner.json`: kloakken tegl, gruva berg med kvader
i søylene, borgen kvader, kjernen kjernestein). Ferdigrommene per tema er
satt sammen av byggesettets pynt og mekanismer (`crypt::PROPS`, merkene).

Det som trenger egne biter (kloakkens kanal med gangbane, gruvas
heissjakt, borgens gallerier, kjernens plastsømmer i muren), står i
`docs/nivaaredigering.md` del 24 som det som gjenstår.

    python3 tools/crypt/modules_themes.py
"""

import copy
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import modules_grav as G  # noqa: E402

ROOT = G.ROOT
BASE = os.path.join(ROOT, "assets", "levels", "moduler")

# Formene fra gravkjelleren som hvert tema får (navnet i modules_grav).
FORMS = ["rom_1x1", "rom_1x2", "rom_2x2", "rom_3x2", "rom_3x3", "gang_1", "gang_2", "gang_hjorne", "gang_t", "gang_kryss", "blindgang",
         "halvtrapp", "halv_etasje", "nedgang", "rotunde"]

# Temaet: navnet i paletten og fakkeltettheten (kloakken og gruva er
# mørkere enn borgen).
THEMES = {
    "kloakk": ("Kloakken", 0.35),
    "gruve": ("Gruva", 0.3),
    "borg": ("Borgen", 0.55),
    "kjerne": ("Kjernen", 0.4),
}

prop = G.prop
grid = G.grid


def themed(theme, m, torches=None):
    """En modul fra gravkjelleren i et annet tema: samme celler, innhold og
    lys, temaet på modulen og rommene (som gir steinen)."""
    m = copy.deepcopy(m)
    name = m["id"].split("/", 1)[1]
    m["id"] = f"{theme}/{name}"
    m["theme"] = theme
    for r in m["rooms"]:
        r["theme"] = theme
        if torches is not None and r.get("mood", {}).get("torches", 0) > 0:
            r["mood"] = {"torches": round(min(r["mood"]["torches"], torches + 0.1), 2)}
    return name, m


def module(theme, name, label, kind, about, cells, torches, **extra):
    n, m = G.module(name, label, kind, about, cells, torches, **extra)
    return themed(theme, m)


def light(id_, cell, pos, color, rng, flicker=True, intensity=None):
    out = {"id": id_, "anchor": f"celle:{cell}", "pos": pos, "color": color, "range": rng, "flicker": flicker}
    if intensity:
        out["intensity"] = intensity
    return out


WARM = [1.0, 0.66, 0.34]
VIOLET = [0.7, 0.4, 1.0]
BLUE = [0.45, 0.65, 1.0]

# Pynt på muren: fanene henger i murflaten (FACE = 1,6 m fra cellemidten).
F = 1.58


def banner(id_, cell, side, along=0.0, y=2.9):
    """En fane på murflaten på siden `side` (N, E, S, W) i cella, vendt inn."""
    pos, yaw = {"N": ([along, y, -F], 0), "S": ([along, y, F], 180), "W": ([-F, y, along], 90), "E": ([F, y, along], -90)}[side]
    return prop(id_, "kit_banner", cell, pos, yaw)


EXTRA = {
    "kloakk": [
        module("kloakk", "slusekammer", "Slusekammeret", "ferdig",
               "En sal på 12 × 8 m der vannet står til knærne. Spaken ved muren åpner slusa, og vannet renner ut.",
               grid("grav/romfelt", 3, 2, -1, 0), 0.35,
               things=[{"id": "vann", "kind": "water", "anchor": "celle:c10", "pos": [0, 0.35, 2.0], "size": [11.2, 7.2]},
                       {"id": "spak", "kind": "lever", "anchor": "celle:c00", "pos": [-1.2, 0.35, -0.8], "yaw": 90}],
               links=[{"from": "spak", "to": "vann", "kind": "opens"}],
               lights=[light("skjaer", "c11", [0, 2.6, 0], [0.55, 0.75, 0.7], 9, False)]),
        module("kloakk", "smuglerlager", "Smuglerlageret", "ferdig",
               "Et rom på 8 × 8 m med tønner, kasser, sekker og hyller langs murene, og ei kiste bak tønnene.",
               grid("grav/romfelt", 2, 2), 0.35,
               things=[prop("tonne1", "barrel_tall", "c00", [-1.15, 0, -1.15], 0, [0.7, 0.7]),
                       prop("tonne2", "barrel_tall", "c00", [-0.35, 0, -1.2], 30, [0.7, 0.7]),
                       prop("kasse1", "cellar_crate", "c10", [1.1, 0, -1.2], 10, [0.7, 0.6]),
                       prop("kasse2", "cellar_crate", "c10", [0.3, 0, -1.25], -8, [0.7, 0.6]),
                       prop("sekker", "sacks", "c01", [-1.1, 0, 1.1], 0, [1.0, 1.15]),
                       prop("hylle", "cellar_shelf", "c11", [1.33, 0, 0.2], 0, [0.45, 2.5]),
                       {"id": "kiste", "kind": "chest", "anchor": "celle:c00", "pos": [-1.1, 0, -0.25], "yaw": 90}],
               lights=[light("lykt", "c11", [0, 2.4, 0], WARM, 7)]),
    ],
    "gruve": [
        module("gruve", "gruvegang", "Gruvegangen", "ferdig",
               "En gang på 8 m med skinner langs gulvet, en malmvogn og lamper i taket.",
               grid("grav/gang", 1, 2), 0.25,
               things=[prop("skinne1", "kit_rail", "c00", [0, 0, -1.0], 0), prop("skinne2", "kit_rail", "c00", [0, 0, 1.0], 0),
                       prop("skinne3", "kit_rail", "c01", [0, 0, -1.0], 0), prop("skinne4", "kit_rail", "c01", [0, 0, 1.0], 0),
                       prop("vogn", "kit_cart", "c01", [0, 0, 0.6], 0, [1.25, 1.5]),
                       prop("lampe1", "kit_lamp", "c00", [0, 3.55, 0], 0), prop("lampe2", "kit_lamp", "c01", [0, 3.55, -1.5], 0)],
               lights=[light("lampe1", "c00", [0, 3.0, 0], WARM, 6), light("lampe2", "c01", [0, 3.0, -1.5], WARM, 6)]),
        module("gruve", "gruvesal", "Gruvesalen", "ferdig",
               "En sal på 12 × 8 m med tømmerstøtter langs murene, skinner over gulvet og en malmvogn.",
               grid("grav/romfelt", 3, 2, -1, 0), 0.25,
               things=[prop("stotte1", "kit_strut", "c00", [0, 0, -1.35], 0, [3.5, 0.34]),
                       prop("stotte2", "kit_strut", "c20", [0, 0, -1.35], 0, [3.5, 0.34]),
                       prop("stotte3", "kit_strut", "c01", [0, 0, 1.35], 0, [3.5, 0.34]),
                       prop("stotte4", "kit_strut", "c21", [0, 0, 1.35], 0, [3.5, 0.34])]
                      + [prop(f"skinne{k}", "kit_rail", c, [x, 0, 0.0], 90) for k, (c, x) in enumerate((("c01", -1.0), ("c01", 1.0), ("c11", -1.0), ("c11", 1.0), ("c21", -1.0)))]
                      + [prop("vogn", "kit_cart", "c21", [0.6, 0, 0.0], 90, [1.25, 1.5]),
                         prop("lampe1", "kit_lamp", "c00", [0, 2.95, -1.35], 0), prop("lampe2", "kit_lamp", "c21", [0, 2.95, 1.35], 0)],
               lights=[light("lampe1", "c00", [0, 2.5, -1.2], WARM, 7), light("lampe2", "c21", [0, 2.5, 1.2], WARM, 7)]),
        module("gruve", "krystallgrotte", "Krystallgrotta", "ferdig",
               "Et rom på 8 × 8 m der blå krystaller vokser ut av gulvet og lyser kaldt.",
               grid("grav/romfelt", 2, 2), 0.0,
               things=[prop("krystall1", "kit_crystal", "c00", [-0.9, 0, -0.9], 20, [1.2, 1.5]),
                       prop("krystall2", "kit_crystal", "c10", [1.0, 0, -0.8], 70, [1.2, 1.5]),
                       prop("krystall3", "kit_crystal", "c01", [-1.0, 0, 0.9], 140, [1.2, 1.5]),
                       prop("krystall4", "kit_crystal", "c11", [0.9, 0, 1.0], 250, [1.2, 1.5])],
               lights=[light("krystall1", "c00", [-0.9, 1.0, -0.9], BLUE, 6, False), light("krystall4", "c11", [0.9, 1.0, 1.0], BLUE, 6, False)]),
    ],
    "borg": [
        module("borg", "tronsal", "Tronsalen", "ferdig",
               "En sal på 12 × 12 m med fire søyler, tronen mot nordmuren, faner på murene, statuer og rustninger langs veggene.",
               grid("grav/romfelt", 3, 3, -1, -1), 0.45,
               walls=[{"at": [0, -1], "side": "N"}],
               things=[prop("trone", "kit_throne", "c10", [0, 0, -0.85], 0, [1.6, 1.4]),
                       prop("statue_v", "kit_statue", "c00", [-0.9, 0, -1.0], 30, [1.4, 1.0]),
                       prop("statue_o", "kit_statue", "c20", [0.9, 0, -1.0], -30, [1.4, 1.0]),
                       prop("rustning1", "kit_armor", "c01", [-1.25, 0, 0], 90, [0.8, 0.6]),
                       prop("rustning2", "kit_armor", "c21", [1.25, 0, 0], -90, [0.8, 0.6]),
                       banner("fane1", "c10", "N", -1.2), banner("fane2", "c10", "N", 1.2),
                       banner("fane3", "c00", "W"), banner("fane4", "c20", "E"), banner("fane5", "c02", "W"), banner("fane6", "c22", "E")],
               lights=[light("trone", "c10", [0, 2.6, -0.4], WARM, 8, True, 2.0)]),
        module("borg", "vaktstue", "Vaktstua", "ferdig",
               "Et rom på 8 × 8 m med våpenstativ, rustninger, en benk og tønner.",
               grid("grav/romfelt", 2, 2), 0.5,
               things=[prop("stativ", "kit_rack", "c00", [0, 0, -1.35], 0, [2.2, 0.45]),
                       prop("rustning1", "kit_armor", "c10", [1.25, 0, -1.2], -45, [0.8, 0.6]),
                       prop("rustning2", "kit_armor", "c11", [1.25, 0, 1.2], -135, [0.8, 0.6]),
                       prop("benk", "bench_20", "c01", [0, 0, 1.3], 0, [2.0, 0.34]),
                       prop("tonne", "barrel_tall", "c01", [-1.2, 0, 0.2], 0, [0.7, 0.7]),
                       banner("fane", "c10", "N")]),
        module("borg", "riddergang", "Riddergangen", "ferdig",
               "En gang på 12 m med gjordbuer og faner på murene mellom buene.",
               grid("grav/gang", 1, 3, 0, -1, variant="b"), 0.6,
               things=[banner("fane1", "c00", "W", 0.0, 1.85), banner("fane2", "c01", "E", 0.0, 1.85), banner("fane3", "c02", "W", 0.0, 1.85)]),
    ],
    "kjerne": [
        module("kjerne", "plastsal", "Plastsalen", "ferdig",
               "En sal på 12 × 8 m der muren og gulvet har blitt plast, og fiolette krystaller vokser ut av sømmene.",
               grid("grav/romfelt", 3, 2, -1, 0), 0.3,
               things=[prop("krystall1", "kit_crystal", "c00", [-1.0, 0, -1.0], 30, [1.2, 1.5]),
                       prop("krystall2", "kit_crystal", "c21", [1.0, 0, 1.0], 200, [1.2, 1.5])],
               lights=[light("som1", "c00", [-1.0, 1.0, -1.0], VIOLET, 7, False), light("som2", "c21", [1.0, 1.0, 1.0], VIOLET, 7, False)],
               stone="plast"),
        module("kjerne", "kjernerotunde", "Kjernerotunden", "ferdig",
               "Rotunden i kjernen: kuppelen er plast, muren stein som går over i plast, og et fiolett lys står i midten.",
               [{"id": "c", "bit": "grav/rotunde", "at": [0, 0, 0], "rot": 0}], 0.4,
               things=[prop("krone", "kit_chandelier", "c", [0, 7.55, 0], 0)],
               lights=[light("midt", "c", [0, 4.6, 0], VIOLET, 11, False, 2.5), light("krone", "c", [0, 4.4, 0], WARM, 8, True)]),
    ],
}


def main():
    forms = {n: m for n, m in G.MODULES if n in FORMS}
    total = 0
    for theme, (_, torches) in THEMES.items():
        out = os.path.join(BASE, theme)
        os.makedirs(out, exist_ok=True)
        G.OUT = out
        mods = [themed(theme, forms[n], torches) for n in FORMS] + EXTRA[theme]
        for name, m in mods:
            stone = m.pop("stone", None)
            if stone:
                for c in m["cells"]:
                    c["stone"] = stone
            G.write(name, m)
        total += len(mods)
        print(f"{len(mods)} moduler i {os.path.relpath(out, ROOT)}")
    print(f"{total} i alt")


if __name__ == "__main__":
    main()
