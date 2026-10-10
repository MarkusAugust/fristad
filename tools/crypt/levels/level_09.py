"""Nivå 9: Overgangen.

Her går borgen over i Fristil. Steinen får sømmer som lyser, speilsalen glitcher mellom
to tilstander, og etter brohodet henger gangene i løse luften over tomrommet mellom
plattformer av plast. Den siste steinen er hvilestedet. Tvillingvokteren bærer nøkkelen.
"""

from plan import *

LEVEL = Level(
    n=9,
    name="Overgangen",
    theme="Den korrupte kjernen",
    intro="Trappa ender i en hall av vanlig stein, men fugene lyser. Lenger inne blir steinen glatt "
          "som plast, og bak speilsalen er det ikke berg lenger, bare et mørkt tomrom.",
    rooms=[
        Room("tomrom", "Tomrommet", ("poly", [(20, 20), (60, 12), (102, 14), (120, 38), (118, 96), (92, 116),
                                              (40, 112), (18, 96), (14, 60)]),
             y=-32, h=60, backdrop=True, secret=True, at=(104, 106),
             note="Ingen bunn å se, bare sømmer som lyser langt nede. Faller du andre steder, løfter økta deg "
                  "opp igjen der du sto. Under det sprukne brostykket ligger et nett av sømmer som tar imot deg."),
        Room("port", "Sømporten", ("rect", 0, 0, 10, 8), y=0, h=5,
             note="Start. Trappa opp til nivå 8 i nord. Én lysende søm i muren, som på nivå 1."),
        Room("sprekk", "Sprekkgangen", ("rect", 0, 20, 4, 24), y=0, h=4,
             note="Steinen sprekker opp i rutenett; noen fliser blinker ut og inn. De som blinker, bærer ikke."),
        Room("speil", "Speilsalen", ("rect", 0, 48, 22, 18), y=-2, h=9,
             note="Glitcher mellom to tilstander hvert tiende sekund: søylesal av stein og tom sal av plast. "
                  "Dørene er ikke på samme sted i de to."),
        Room("bakrom", "Det andre rommet", ("rect", -18, 48, 7, 7), y=-2, h=3, secret=True,
             note="Finnes bare i plasttilstanden: en dør der det i steintilstanden er en vegg som flimrer."),
        Room("brohode", "Brohodet", ("circle", 26, 48, 4), y=-2, h=6,
             note="Siste stein før tomrommet. Herfra ser du alle plattformene og den hvite søylen på øya."),
        Room("oya", "Den siste steinen", ("circle", 56, 58, 7), y=-4, h=10,
             note="Hvilestedet: en øy av ekte stein med alteret og en hvit lyssøyle som synes fra hele tomrommet."),
        Room("plast", "Plastkammeret", ("poly", [(76, 26), (94, 22), (100, 34), (92, 46), (78, 42)]),
             y=-6, h=7, note="Gulv og vegger av glatt, matt plast med lysende sømmer. Grid-fiender deler rommet i ruter."),
        Room("arkiv", "Arkivet", ("rect", 109, 33, 8, 10), y=-6, h=4,
             note="Hyller av bøker som er blitt til grå blokker; én lesbar bok igjen."),
        Room("fragment", "Borgfragmentet", ("poly", [(22, 74), (36, 71), (41, 81), (31, 89), (20, 84)]),
             y=-6, h=8, note="Et stykke av ridderhallen som river seg løs og driver: halve søyler, en fane og en kiste. "
                             "Bueskytterne her skyter mot broene."),
        Room("kube", "Kuben", ("rect", 88, 76, 14, 14), y=-8, h=10,
             note="Et svevende rom som vrir seg: gulvet blir vegg når du går inn. Bakhold av Skeleton loaders."),
        Room("tvilling", "Tvillingsalen", ("rect", 52, 96, 20, 12), y=-9, h=8,
             note="Salen er to like rom lagt oppå hverandre; vokteren står i begge. Slår du den ene, blinker den andre."),
        Room("nedgang", "Nedgangen", ("circle", 26, 110, 5), y=-10, h=6,
             note="Bak porten. Trappa ned til nivå 10 lyser blått gjennom plasten."),
    ],
    links=[
        Link("port", "sprekk", DOOR, width=3),
        Link("sprekk", "speil", STAIRS, width=3),
        Link("speil", "bakrom", SECRET, note="flimrende vegg i steintilstanden"),
        Link("speil", "brohode", DOOR, width=3, note="bare åpen i plasttilstanden"),
        Link("brohode", "oya", BRIDGE, path=[(34, 44), (42, 50), (48, 58)], width=2,
             note="gangen vrir seg om seg selv"),
        Link("oya", "plast", BRIDGE, path=[(64, 48), (70, 36), (80, 32)], width=2),
        Link("plast", "arkiv", DOOR, width=2),
        Link("plast", "kube", BRIDGE, path=[(98, 52), (96, 64)], width=1.5, note="smal, uten rekkverk"),
        Link("plast", "tomrom", FALL, path=[(100, 58)], note="sprukket brostykke"),
        Link("tomrom", "kube", LADDER, path=[(106, 88)], width=1.5, note="løftestrålen fra nettet opp til kuben"),
        Link("kube", "oya", SHORTCUT, path=[(72, 70)], width=2,
             note="bro av lys som bygger seg fra kuben når du trår på sømmen"),
        Link("oya", "fragment", BRIDGE, path=[(46, 68), (38, 74)], width=1.5, note="broa henger i kjettinger"),
        Link("kube", "tvilling", BRIDGE, path=[(78, 90)], width=2),
        Link("tvilling", "nedgang", GATE, path=[(38, 104)], width=3, note="porten, åpnes med vokterens nøkkel"),
    ],
    marks=[
        Mark(START, 0, 1),
        Mark(UP, 0, -3, "opp til nivå 8"),
        Mark(HINT, 4, -2, "søm i muren"),
        Mark(MOB, 0, 14, "Card", n=3),
        Mark(TRAP, 0, 24, "blinkende fliser"),
        Mark(MOB, 0, 30, "Tabs"),
        Mark(MOB, -4, 46, "Skeleton loader", n=3),
        Mark(MOB, 6, 52, "Breadcrumb", n=2),
        Mark(HINT, -11, 48, "flimmer"),
        Mark(BONUS, -18, 48, "+ liv"),
        Mark(MOB, 24, 46, "Popover"),
        Mark(MOB, 38, 46, "Popover", n=2),
        Mark(REST, 56, 58, "alteret"),
        Mark(MOB, 86, 34, "Grid", n=3, patrol=[(80, 30), (94, 28), (90, 42), (80, 40)]),
        Mark(MOB, 84, 40, "Accordion"),
        Mark(CHEST, 96, 34),
        Mark(LORE, 110, 30, "den siste lesbare boka"),
        Mark(HINT, 99, 54, "søm under broa"),
        Mark(MOB, 109, 37, "Skeleton loader", n=2),
        Mark(MOB, 30, 78, "Breadcrumb", n=3),
        Mark(CHEST, 26, 84, "fanen"),
        Mark(RARE, 104, 99, "Radio"),
        Mark(LORE, 95, 96, "nettets stemme"),
        Mark(MOB, 88, 76, "Skeleton loader i bakhold", n=4),
        Mark(TRAP, 84, 82, "rommet vrir seg"),
        Mark(LEVER, 80, 72, "sømmen"),
        Mark(MOB, 66, 92, "Popover", n=2),
        Mark(ELITE, 52, 96, "Tvillingvokteren (nøkkel)"),
        Mark(MOB, 46, 100, "Grid", n=3),
        Mark(DOWN, 26, 110, "til nivå 10"),
    ],
    main=["port", "sprekk", "speil", "brohode", "oya", "plast", "kube", "tvilling", "nedgang"],
    notes=[
        "Lærer at verden ikke er fast: flisene blinker, speilsalen bytter tilstand og dørene flytter seg. "
        "Du venter på riktig tilstand i stedet for å slåss mot den.",
        "Landemerket er den hvite lyssøylen på Den siste steinen, synlig fra brohodet og alle plattformene. "
        "Hovedveien har varme sømmer, sidegrenene kalde.",
        "Hvilestedet er Den siste steinen midtveis. Lysbroen fra kuben er snarveien tilbake, så tvillingsalen "
        "ligger 50 m fra alteret.",
        "Løkka: øya, plastkammeret, kuben og lysbroen tilbake til øya. Borgfragmentet er en sidegren med "
        "kiste, der bueskytterne skyter mot broene til du har tatt dem.",
        "Hemmelighetene: det andre rommet i speilsalen (bare i plasttilstanden, + liv), og det sprukne "
        "brostykket mellom plastkammeret og kuben: hopp, og nettet i tomrommet tar deg imot (Radio, lore), "
        "løftestrålen tar deg opp i kuben.",
        "Fellene: blinkende fliser i sprekkgangen og kuben som vrir seg når du står i den.",
        "Nye fiender: Grid (deler gulvet i ruter som lyser før de slår), Skeleton loader (grå skygger som bare "
        "kan treffes når de er lastet inn) og Popover (dukker opp bak deg på broene). Card, Tabs, Breadcrumb "
        "og Accordion kommer igjen, delvis korrupte. Tvillingvokteren er eliten med nøkkelen.",
    ],
)
