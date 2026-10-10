"""Nivå 4: Den store cisternen.

Bossnivået for kloakken. En enorm søylesal i vann som du går ned gjennom i tre høyder:
nordgalleriet øverst, gangbroene og øya med hvilestedet i midten, og den nedre gangbroa
som først kommer opp av vannet når ventilen i akvedukten er stengt. Nederst kjemper
Tabellen på en flåte av plattformer.
"""

import math

from plan import *


def ngon(cx, cz, r, n, rot=0.0):
    return [(round(cx + r * math.cos(rot + 2 * math.pi * i / n), 2),
             round(cz + r * math.sin(rot + 2 * math.pi * i / n), 2)) for i in range(n)]


LEVEL = Level(
    n=4,
    name="Den store cisternen",
    theme="Kloakken og cisternene",
    intro="Trappa fra kloakken munner ut høyt oppe i nordveggen av en søylesal så stor at lykta ikke når "
          "den andre sida. Øya midt i vannet har hvilestedet; Tabellen venter under salen.",
    rooms=[
        Room("trappefot", "Trappefoten", ("rect", 34, 12, 8, 7), y=0, h=5,
             note="Start. Trappa opp til nivå 3. En åpning i sør ut mot salen; du hører vann som drypper langt nede."),
        Room("cisternen", "Cisternen", ("rect", 40, 55, 64, 42), y=-10, h=20,
             note="Søylesalen, 64 × 42 m, med 40 søyler i vann. Vannet står på −8 m til ventilen er stengt, "
                  "så på −9,5 m. Faller du, svømmer du til en stige."),
        Room("nordgalleri", "Nordgalleriet", ("rect", 40, 30.5, 64, 3), y=0, h=4,
             note="Øverste høyde: gallerigang hogd inn i nordveggen, med buer ut mot salen og utsikt over alt."),
        Room("pumpehus", "Pumpehuset", ("rect", 80, 24, 12, 10), y=0, h=5,
             note="Gamle pumper og kjettinger. Trapp ned langs østveggen til gangbroa, og en vindeltrapp ned til søylelunden."),
        Room("oya", "Øya", ("circle", 40, 52, 6), y=-4, h=16,
             note="Hvilestedet: alteret står på en øy av søyler midt i salen, synlig fra overalt. Kjettingheisen ned til filterkammeret."),
        Room("akvedukt", "Akvedukten", ("rect", -2, 54, 6, 36), y=-4, h=5,
             note="Vannrenna som fyller cisternen, med gangbane langs kanten. Vannet stuper over vestveggen."),
        Room("bygg", "Byggmesterens kammer", ("rect", -15, 57, 7, 7), y=-4, h=3, secret=True,
             note="Bak en falsk vegg i akvedukten: byggmesterens tegninger av cisternen og varig styrke."),
        Room("munning", "Kanalmunningen", ("rect", -2, 27, 8, 6), y=-4, h=3, secret=True,
             note="Der vannet kommer inn. Tørrlagt når ventilen er stengt: en kiste og varig styrke i slammet."),
        Room("ventil", "Ventilkammeret", ("rect", -4, 84, 12, 10), y=-4, h=5,
             note="Det store ventilhjulet. Stenger du det, tørker akvedukten ut og vannet i salen synker."),
        Room("nedregang", "Den nedre gangbroa", ("rect", 40, 73.5, 64, 3), y=-7, h=4,
             note="Ligger under vann til ventilen er stengt. Går langs sørveggen fra ventilkammeret til søylelunden."),
        Room("soylelund", "Søylelunden", ("poly", [(76, 58), (90, 54), (96, 68), (88, 82), (74, 80)]),
             y=-6, h=8, note="Tett skog av søyler i grunt vann. Toasts i bakhold mellom søylene; en kiste innerst."),
        Room("soylefot", "Søylefoten", ("rect", 16, 18, 8, 7), y=-12, h=3, secret=True,
             note="Inne i en hul søyle: du slipper deg ned fra nordgalleriet til et rom under vannet. Stige opp til trappefoten."),
        Room("filter", "Filterkammeret", ("rect", 40, 88, 16, 8), y=-7, h=5,
             note="Forkammeret. Kjettingheisen opp til øya (snarveien) kalles ned herfra."),
        Room("arena", "Plattformsalen", ("circle", 40, 112, 14), y=-11, h=14,
             levels=[("circle", 33, 106, 3.5, -9), ("circle", 48, 107, 3, -8.5),
                     ("circle", 34, 119, 3, -9.5), ("circle", 47, 119, 3.5, -9)],
             note="Rund sal under cisternen med svart vann og fire flytende plattformer. Tabellen deler seg mellom dem."),
        Room("nedgang", "Nedgangen", ("rect", 40, 134, 8, 6), y=-12, h=5,
             note="Åpnes når Tabellen faller. Trappa ned til gruva."),
    ],
    links=[
        Link("trappefot", "nordgalleri", STAIRS, path=[(34, 30.5)], note="ut på nordgalleriet"),
        Link("nordgalleri", "cisternen", FALL, path=[(54, 36)], note="ned i vannet"),
        Link("cisternen", "oya", LADDER, note="stiger opp av vannet ved søylene"),
        Link("nordgalleri", "pumpehus", DOOR, width=2.5),
        Link("nordgalleri", "soylefot", FALL, path=[(16, 30.5)], note="den hule søylen"),
        Link("soylefot", "trappefot", LADDER, note="stige opp til en luke"),
        Link("pumpehus", "oya", BRIDGE, path=[(72, 40), (72, 52)], width=2.5,
             note="trapp ned langs østveggen og gangbroa på −4 m"),
        Link("pumpehus", "soylelund", STAIRS, path=[(90, 40)], width=2, note="vindeltrapp"),
        Link("oya", "akvedukt", BRIDGE, path=[(8, 52)], width=2.5, note="vestre gangbro"),
        Link("akvedukt", "bygg", SECRET, note="falsk vegg"),
        Link("akvedukt", "munning", GATE, note="tørrlagt når ventilen er stengt"),
        Link("akvedukt", "ventil", HALL, width=2.5),
        Link("ventil", "nedregang", STAIRS, path=[(8, 80)], width=2.5),
        Link("nedregang", "soylelund", HALL, path=[(74, 73.5)], width=2.5),
        Link("nedregang", "filter", DOOR, width=3),
        Link("filter", "oya", SHORTCUT, path=[(44, 70)], width=2, note="kjettingheisen"),
        Link("filter", "arena", GATE, width=4, note="porten stenger bak deg"),
        Link("arena", "nedgang", DOOR, width=3),
    ],
    marks=[
        Mark(START, 34, 13),
        Mark(UP, 34, 9, "opp til nivå 3"),
        Mark(MOB, 50, 30.5, "input", n=2, patrol=[(10, 30.5), (70, 30.5)]),
        Mark(HINT, 16, 30.5, "søylen klinger hult"),
        Mark(RARE, 16, 18, "Fieldset"),
        Mark(MOB, 80, 24, "select", n=2),
        Mark(CHEST, 84, 21),
        Mark(MOB, 64, 52, "toast", n=3),
        Mark(REST, 40, 50, "alteret"),
        Mark(MOB, 18, 52, "toast", n=2),
        Mark(MOB, -2, 41, "input", n=3, patrol=[(-2, 38), (-2, 70)]),
        Mark(TRAP, -2, 62, "vannstøt"),
        Mark(HINT, -5, 57, "kaldt trekk"),
        Mark(LORE, -17, 59.5, "tegningene"),
        Mark(BONUS, -13, 55.5, "+ liv"),
        Mark(CHEST, -4, 27),
        Mark(BONUS, 0, 28, "+ utholdenhet"),
        Mark(LEVER, -8, 87, "ventilen"),
        Mark(MOB, -2, 82, "knapp + select", n=3),
        Mark(MOB, 22, 73.5, "pagination", n=3, patrol=[(10, 73.5), (70, 73.5)]),
        Mark(MOB, 84, 68, "toast i bakhold", n=4),
        Mark(MOB, 80, 61, "input", n=2),
        Mark(CHEST, 90, 72),
        Mark(MOB, 40, 86, "select + input", n=3),
        Mark(BOSS, 40, 112, "Tabellen"),
        Mark(DOWN, 40, 134, "til nivå 5"),
    ],
    main=["trappefot", "nordgalleri", "pumpehus", "oya", "akvedukt", "ventil", "nedregang",
          "filter", "arena", "nedgang"],
    notes=[
        "Lærer å lese en stor sal i høyder: du ser målet (øya, lyset fra plattformsalen) lenge før du når det, og går ned i tre trinn: galleriet, gangbroene, den nedre gangbroa.",
        "Landemerket: øya med hvilestedet midt i salen. Lysestaken på alteret synes fra galleriet, fra gangbroene og fra vannet; tenner du den, lyser den hele nivået.",
        "Lyset: fakler langs gangbroene viser veien, og et kaldt blått skjær stiger opp fra plattformsalen under sørveggen. Når vannet synker, tennes lyktene på den nedre gangbroa en etter en.",
        "Løkka: øya → akvedukten → ventilen → den nedre gangbroa → søylelunden → pumpehuset → øya. Snarveien er kjettingheisen fra filterkammeret opp til øya.",
        "Hemmelighetene: den hule søylen fra nordgalleriet (Fieldset, stige opp til trappefoten), den falske veggen i akvedukten (tegningene, + liv) og kanalmunningen som tørker ut når ventilen er stengt (+ utholdenhet).",
        "Fellen: en trykkplate i akvedukten slipper et vannstøt fra rørene som skyller deg over kanten og ned i cisternen. Ikke dødelig, men du må svømme til øya.",
        "Fiendene: input, select og toast sterkere enn i kloakken, og pagination (små rader som følger hverandre i flokk). Tabellen deler seg i rader på hver sin plattform og setter seg sammen igjen.",
    ],
)
