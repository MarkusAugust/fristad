"""Nivå 2: Gravkongens hall.

Første bossnivå. Lærer to mål i hver sin fløy, vertikalitet (balkonger, et tårn med
galleri) og en fallem som ikke dreper deg, men setter deg i en kjeller full av fiender.
Nederst venter Modalen i det runde gravkammeret.
"""

import math

from plan import *


def ngon(cx, cz, r, n, rot=0.0):
    return [(round(cx + r * math.cos(rot + 2 * math.pi * i / n), 2),
             round(cz + r * math.sin(rot + 2 * math.pi * i / n), 2)) for i in range(n)]


def ring_path(cx, cz, r, n=8):
    return ngon(cx, cz, r, n)


LEVEL = Level(
    n=2,
    name="Gravkongens hall",
    theme="Gravkjelleren",
    intro="Trappa fra nivå 1 ender ved en søylegang med sarkofager. Gravkongens sal har to fløyer; "
          "en spak i hver senker broa over avgrunnen til gravkammeret der Modalen venter.",
    rooms=[
        Room("trappefot", "Trappefoten", ("rect", 0, 0, 8, 7), y=0, h=5,
             note="Start. Trappa opp til nivå 1 i nord."),
        Room("soylegang", "Sarkofaggangen", ("rect", 0, 24, 9, 30), y=-1, h=6,
             note="Søyler med sarkofager imellom. Én sarkofag i vestre rad er sprukket, og det trekker kaldt."),
        Room("likkapell", "Likkapellet", ("rect", 20, 30, 9, 8), y=-1, h=5,
             note="Hvilestedet: alteret med lysestaken. Eikedøra i sør er snarveien fra forhallen."),
        Room("kongegrav", "Gravkongens grav", ("rect", -18, 24, 8, 9), y=-4, h=3.5, secret=True,
             note="Gjennom den sprukne sarkofagen og ned. Den egentlige gravkongen ligger her, med Divider i hendene. "
                  "En lav gang går videre til kjelleren under vestfløyen."),
        Room("kongesal", "Kongesalen", ("rect", 0, 54, 28, 20), y=-2, h=11,
             levels=[("rect", -11.5, 54, 5, 20, 2.5), ("rect", 11.5, 54, 5, 20, 2.5),
                     ("ring", 0, 54, 2.5, 4.5, 0.5)],
             note="Sal i to etasjer. Balkongene (+2,5 m) langs vest og øst nås med trappa som går rundt den store gravkista midt i salen."),
        Room("nisjegang", "Nisjegangen", ("rect", -28, 54, 19, 3.5), y=2.5, h=3.2,
             note="Vestfløyen fra balkongen. Fallem midt i gangen; den ene steinhella ligger litt lavere enn de andre."),
        Room("balsam", "Balsamkammeret", ("poly", ngon(-46, 54, 7, 6)), y=2.5, h=5,
             note="Sekskantet kammer med balsameringsbord. Venstre spak bak bordet."),
        Room("kjeller", "Kjelleren", ("rect", -30, 72, 16, 10), y=-6, h=3.5,
             note="Fallemmen ender her, i et lavt hvelv fullt av fiender. Stigen opp går til kongesalen."),
        Room("kronike", "Krønikekammeret", ("rect", -50, 79, 7, 6), y=-6, h=3, secret=True,
             note="Bak en falsk vegg i kjelleren (lyd bak veggen): gravkongens krønike og varig styrke."),
        Room("spydgang", "Spydgangen", ("rect", 27, 50, 16, 3), y=2.5, h=3.2,
             note="Østfløyen fra balkongen. Trykkplater; spyd stikker ut av gravnisjene."),
        Room("gravtarn", "Gravtårnet", ("circle", 46, 50, 9), y=-1, h=14,
             levels=[("ring", 46, 50, 6, 9, 6.0)],
             note="Rundt tårn med vindeltrapp langs muren opp til galleriet (+6 m). Høyre spak øverst."),
        Room("tarnnisje", "Tårnnisjen", ("rect", 58, 38, 5, 4), y=6, h=2.5, secret=True,
             note="En løs stein i galleriet; bak ligger en nisje med varig styrke."),
        Room("bruhall", "Bruhallen", ("rect", 0, 76, 20, 8), y=-2, h=9,
             note="Kanten av avgrunnen. Broa henger i to kjettinger oppe i taket til begge spakene er trukket."),
        Room("forhall", "Forhallen", ("rect", 0, 96, 14, 8), y=-3, h=6,
             note="På andre sida av broa. Eikedør med slå i vest (snarveien til likkapellet)."),
        Room("arena", "Modalens gravkammer", ("circle", 0, 118, 12), y=-4, h=12,
             note="Rundt gravkammer med en enorm steinkiste i midten. Porten stenger bak deg."),
        Room("nedgang", "Nedgangen", ("rect", 0, 138, 9, 6), y=-5, h=5,
             note="Åpnes når Modalen faller. Trappa ned til kloakken med det kalde blå lyset."),
    ],
    links=[
        Link("trappefot", "soylegang", STAIRS),
        Link("soylegang", "likkapell", DOOR, path=[(10, 30)], width=2.5),
        Link("soylegang", "kongegrav", SECRET, path=[(-4, 30)], note="den sprukne sarkofagen"),
        Link("kongegrav", "kjeller", HALL, path=[(-18, 40), (-26, 60)], width=2,
             note="lav gang under vestfløyen; slå på kjellersida"),
        Link("soylegang", "kongesal", DOOR, width=4),
        Link("kongesal", "nisjegang", DOOR, path=[(-13, 54)], width=2.5, note="fra vestre balkong"),
        Link("nisjegang", "balsam", DOOR, width=2.5),
        Link("nisjegang", "kjeller", FALL, path=[(-28, 62)], note="fallemmen"),
        Link("kjeller", "kongesal", LADDER, path=[(-18, 64)], note="stige opp i kongesalen"),
        Link("kjeller", "kronike", SECRET, note="falsk vegg"),
        Link("kongesal", "spydgang", DOOR, path=[(13, 50)], width=2.5, note="fra østre balkong"),
        Link("spydgang", "gravtarn", STAIRS, width=2.5),
        Link("gravtarn", "tarnnisje", SECRET, path=[(53, 42)], note="løs stein i galleriet"),
        Link("kongesal", "bruhall", HALL, width=4),
        Link("bruhall", "forhall", BRIDGE, width=3, note="senkes når begge spakene er trukket"),
        Link("forhall", "arena", GATE, width=4, note="steinporten stenger bak deg"),
        Link("arena", "nedgang", DOOR, width=3),
        Link("forhall", "likkapell", SHORTCUT, path=[(18, 96), (24, 68), (24, 40)], width=2,
             note="eikedør med slå, åpnes fra forhallen"),
    ],
    marks=[
        Mark(START, 0, 1),
        Mark(UP, 0, -3, "opp til nivå 1"),
        Mark(REST, 22, 28, "alteret"),
        Mark(MOB, 0, 14, "avatar", n=2),
        Mark(MOB, 2, 30, "breadcrumb", patrol=[(-2, 12), (-2, 36), (2, 36), (2, 12)]),
        Mark(HINT, -3.5, 32, "sprukken sarkofag"),
        Mark(RARE, -19, 22, "Divider"),
        Mark(CHEST, -16, 27),
        Mark(MOB, 4, 52, "accordion i bakhold", n=3),
        Mark(MOB, -11.5, 50, "tooltip", n=2, patrol=[(-11.5, 46), (-11.5, 62)]),
        Mark(MOB, 7, 60, "knapp", n=2),
        Mark(TRAP, -28, 56.5, "fallem"),
        Mark(MOB, -24, 55, "avatar", n=2),
        Mark(LEVER, -51, 54, "venstre spak"),
        Mark(MOB, -47, 50, "knapp", n=3),
        Mark(CHEST, -46, 58),
        Mark(MOB, -31, 68.5, "breadcrumb + avatar", n=4),
        Mark(CHEST, -25, 75),
        Mark(HINT, -36, 72, "lyd bak veggen"),
        Mark(LORE, -51, 78, "krøniken"),
        Mark(BONUS, -49, 81, "+ liv"),
        Mark(TRAP, 27, 50, "spyd"),
        Mark(MOB, 50, 50, "accordion", n=2),
        Mark(MOB, 39, 53, "tooltip", n=2, patrol=ring_path(46, 50, 7.5)),
        Mark(LEVER, 46, 58, "høyre spak"),
        Mark(HINT, 52, 43, "en annen stein"),
        Mark(BONUS, 58, 38, "+ utholdenhet"),
        Mark(MOB, 0, 76, "avatar", n=2),
        Mark(CHEST, 5, 97),
        Mark(BOSS, 0, 118, "Modalen"),
        Mark(DOWN, 0, 138, "til nivå 3"),
    ],
    main=["trappefot", "soylegang", "kongesal", "nisjegang", "balsam", "nisjegang", "kongesal",
          "spydgang", "gravtarn", "spydgang", "kongesal", "bruhall", "forhall", "arena", "nedgang"],
    notes=[
        "Lærer to mål i hver sin fløy, balkonger og tårn, og første boss. Kongesalen er navet: du kommer tilbake dit tre ganger og lærer den å kjenne.",
        "Landemerket: den store gravkista midt i kongesalen med trappa rundt. Fra døra ser du rett gjennom salen til bruhallen og kjettingene som holder broa oppe.",
        "Lyset: hver spak tenner en rekke kullpanner langs sin balkong mot bruhallen, så du ser hvilken fløy som er gjort. Når begge er tent, senkes broa med et brak.",
        "Fallemmen i nisjegangen er en felle, ikke en død: kjelleren under har fire fiender og en stige opp til kongesalen.",
        "Hemmelighetene: den sprukne sarkofagen (Divider, og en snarvei til kjelleren), den falske veggen i kjelleren (krøniken, + liv) og den løse steinen i tårngalleriet (+ utholdenhet).",
        "Nye fiender: avatar (hodeskaller som ruller og hopper), breadcrumb (beinkjede som kryper etter deg på patrulje) og accordion (kister som folder seg ut i bakhold).",
        "Modalen: lokket på kista smeller i bakken, og et teppe av mørke legger seg over arenaen så bare lysestakene i kanten lyser. Snarveien gjør veien fra hvilestedet til porten rundt 85 m.",
    ],
)
