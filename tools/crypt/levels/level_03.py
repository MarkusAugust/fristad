"""Nivå 3: Kloakken.

Nytt tema. Lærer vannstanden: en hovedkanal med gangbaner og bruer, og et sluseratt
som tømmer kanalen og rotunden, så nye ganger åpner seg nede i bunnen. En elite bærer
nøkkelen til porten ned.
"""

import math

from plan import *


def ngon(cx, cz, r, n, rot=0.0):
    return [(round(cx + r * math.cos(rot + 2 * math.pi * i / n), 2),
             round(cz + r * math.sin(rot + 2 * math.pi * i / n), 2)) for i in range(n)]


LEVEL = Level(
    n=3,
    name="Kloakken",
    theme="Kloakken og cisternene",
    intro="Trappa fra gravkjelleren ender ved slusevokterens stue. Under byen renner hovedkanalen; "
          "vannet står høyt til slusa i øst er dreid, og porten ned er låst.",
    rooms=[
        Room("trappefot", "Trappefoten", ("rect", 0, 0, 8, 7), y=0, h=5,
             note="Start. Trappa opp til nivå 2 i nord. Lyden av rennende vann."),
        Room("vokterstue", "Slusevokterens stue", ("rect", -16, 5, 9, 7), y=0, h=3.5,
             note="Hvilestedet. Køye, kart over kanalene på veggen, eikedør i vest (snarveien fra utløpet)."),
        Room("kanal", "Hovedkanalen", ("rect", 38, 24, 84, 10), y=-3, h=8,
             levels=[("rect", 38, 20.25, 84, 2.5, 0.0), ("rect", 38, 27.75, 84, 2.5, 0.0),
                     ("rect", 18, 24, 3, 5, 0.3), ("rect", 58, 24, 3, 5, 0.3)],
             note="Kanalen med gangbaner på hver side (0 m) og to bruer over. Til slusa er dreid står vannet "
                  "en halv meter under gangbanene; etterpå kan du gå i bunnen (−3 m)."),
        Room("mosegrotta", "Mosegrotta", ("poly", [(20, 0), (28, -3), (35, 3), (33, 12), (22, 12), (18, 6)]),
             y=0.5, h=4, note="Naturlig grotte der kanalmuren har rast. Mose, sopp og en kiste."),
        Room("fossen", "Fossen", ("rect", 62, 6, 12, 12), y=2, h=7,
             note="Et rør i nordmuren spyr vann ned i kanalen. Bak fossen skimtes et gitter."),
        Room("basseng", "Det glemte bassenget", ("rect", 62, -11, 10, 7), y=2, h=3.5, secret=True,
             note="Bak gitteret bak fossen: et gammelt romersk basseng med Search på bunnen."),
        Room("sluse", "Slusekammeret", ("rect", 92, 24, 12, 16), y=0, h=7,
             note="Det store sluserattet. Dreier du det, synker vannet i kanalen og rotunden med et brøl."),
        Room("kapell", "Det druknede kapellet", ("rect", 52, 41, 8, 6), y=-4, h=3.2, secret=True,
             note="En dør i kanalbunnen som bare synes når vannet er borte. Slusevokterens dagbok og varig styrke."),
        Room("rottereir", "Rottereiret", ("poly", [(-4, 38), (10, 37), (14, 45), (7, 52), (-3, 49)]),
             y=-3.5, h=3.5, note="Ved enden av kanalbunnen. Søppel og bein; toasts i bakhold under slammet. Kiste."),
        Room("silkammer", "Silkammeret", ("poly", ngon(34, 58, 8, 8, math.pi / 8)), y=-4, h=6,
             note="Åttekantet kammer med store jerngitter som siler vannet. Eliten med nøkkelen holder til her."),
        Room("rotunden", "Rotunden", ("circle", 78, 54, 9), y=-3, h=9,
             levels=[("ring", 78, 54, 7, 9, 0.0)],
             note="Rundt overløpskammer med ringgang (0 m). Vannet virvler mot sluket; når det er tømt, "
                  "går en dør i bunnen vest til silkammeret."),
        Room("underkum", "Underkummen", ("rect", 64, 78, 9, 7), y=-8, h=3, secret=True,
             note="Sluket i rotunden er løst. Du glir ned et rør og lander i en kum. Stige opp til utløpet."),
        Room("utlop", "Utløpet", ("rect", 30, 82, 14, 10), y=-5, h=6,
             note="Der kloakken renner ut i dypet. Den låste porten ned i sør; eikedør i vest (snarveien)."),
        Room("nedgang", "Nedgangen", ("rect", 30, 98, 8, 6), y=-6, h=5,
             note="Trappa ned til cisternen med det kalde blå lyset; du hører fossen der nede."),
    ],
    links=[
        Link("trappefot", "vokterstue", DOOR, width=2.5),
        Link("trappefot", "kanal", STAIRS, path=[(0, 20.25), (30, 20.25)], note="ned til den nordre gangbanen"),
        Link("kanal", "mosegrotta", DOOR, path=[(26, 20)], width=2.5),
        Link("kanal", "fossen", STAIRS, path=[(62, 20)], width=2.5),
        Link("fossen", "basseng", SECRET, note="gitteret bak fossen"),
        Link("kanal", "sluse", DOOR, path=[(80, 24)], width=3),
        Link("sluse", "rotunden", HALL, path=[(92, 42)], width=2.5, note="den søndre gangbanen"),
        Link("kanal", "kapell", SECRET, path=[(52, 29)], note="døra i bunnen, synes når vannet er borte"),
        Link("kanal", "rottereir", HALL, path=[(4, 29)], width=2.5, note="fra kanalbunnen, åpen når vannet er borte"),
        Link("kanal", "silkammer", HALL, path=[(34, 29)], width=2.5,
             note="stikkrenna: fra kanalbunnen til silkammeret, åpen når vannet er borte"),
        Link("rotunden", "silkammer", DOOR, path=[(60, 56)], width=2.5, note="døra i bunnen av rotunden"),
        Link("rotunden", "underkum", FALL, path=[(78, 70)], note="sluket"),
        Link("underkum", "utlop", LADDER, path=[(46, 80)], note="stige og kryperør"),
        Link("silkammer", "utlop", HALL, width=3),
        Link("utlop", "nedgang", GATE, note="porten ned, låst; nøkkelen har eliten"),
        Link("utlop", "vokterstue", SHORTCUT, path=[(-12, 82), (-24, 40), (-22, 8)], width=2,
             note="eikedør med slå, åpnes fra utløpet"),
    ],
    marks=[
        Mark(START, 0, 1),
        Mark(UP, 0, -3, "opp til nivå 2"),
        Mark(REST, -18, 5, "alteret"),
        Mark(MOB, 12, 20, "input", n=2, patrol=[(6, 20.25), (74, 20.25)]),
        Mark(MOB, 22, 25, "toast", n=3),
        Mark(MOB, 48, 28, "knapp", n=2),
        Mark(MOB, 27, 6, "tooltip", n=2),
        Mark(CHEST, 31, 9),
        Mark(MOB, 62, 9.5, "input", n=2),
        Mark(HINT, 57, 4, "lys bak fossen"),
        Mark(RARE, 62, -11, "Search"),
        Mark(LEVER, 96, 22, "sluseratt"),
        Mark(MOB, 89, 28, "select", n=2),
        Mark(CHEST, 95, 30),
        Mark(TRAP, 92, 36, "flodluke"),
        Mark(MOB, 71, 50, "toast", n=3),
        Mark(MOB, 84, 59, "select", patrol=[(78, 46), (85, 54), (78, 62), (71, 54)]),
        Mark(HINT, 80, 62, "virvel i sluket"),
        Mark(BONUS, 64, 78, "+ utholdenhet"),
        Mark(CHEST, 67, 80),
        Mark(HINT, 52, 31, "bobler"),
        Mark(LORE, 50, 41, "dagboka"),
        Mark(BONUS, 54, 43, "+ liv"),
        Mark(MOB, 4, 44, "toast i bakhold", n=4),
        Mark(CHEST, 9, 48),
        Mark(TRAP, 34, 44, "giftgass"),
        Mark(ELITE, 34, 58, "Select med nøkkelen"),
        Mark(MOB, 30, 54, "input", n=2),
        Mark(MOB, 28, 83, "toast", n=2),
        Mark(MOB, 34, 79, "knapp", n=2),
        Mark(DOWN, 30, 98, "til nivå 4"),
    ],
    main=["trappefot", "kanal", "sluse", "rotunden", "silkammer", "utlop", "nedgang"],
    notes=[
        "Lærer vannstanden: slusa forandrer hele nivået. Kanalbunnen, døra i rotunden, rottereiret og et hemmelig kapell åpner seg når vannet er borte.",
        "Landemerket: hovedkanalen, 84 m lang, med to bruer. Fossen i nordmuren og brølet fra slusa i øst kan høres fra hele kanalen.",
        "Lyset: grønnlige lykter langs den nordre gangbanen viser veien mot slusa. Når vannet synker, tennes de søndre lyktene og lyset fra silkammeret synes nede i stikkrenna.",
        "Løkka: slusa → rotunden → silkammeret → stikkrenna tilbake til kanalen. Snarveien fra utløpet går til slusevokterens stue (hvilestedet).",
        "Hemmelighetene: gitteret bak fossen (Search), døra i kanalbunnen (dagboka, + liv) og sluket i rotunden ned i underkummen (+ utholdenhet, stige opp til utløpet).",
        "Fellene: flodluka på den søndre gangbanen skyller deg ned i vannet, og en trykkplate i stikkrenna slipper ut giftgass.",
        "Nye fiender: input (spydbærere på gangbanene), select (trekker deg ned under vannet) og toast (dukker opp av vannet, slår og forsvinner). Eliten er en stor select med nøkkelen.",
    ],
)
