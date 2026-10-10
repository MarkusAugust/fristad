"""Nivå 7: Den sunkne borgen.

En hel borg sank i berget og står under et steintak. Vindebroen er heist, så veien inn
går ned i vollgrava, gjennom utfallsporten og opp langs brystvernet til porthuset, der
vinsjen senker broen (snarveien tilbake til hvilestedet). Kortkapteinen i kastellet
bærer nøkkelen til fangehullet og trappa ned.
"""

from plan import *

# Borgmuren: x 16–68, z 34–88. Brystvernet ligger på +5 m langs alle fire murene.
WALK = 5

LEVEL = Level(
    n=7,
    name="Den sunkne borgen",
    theme="Borgen under jorda",
    intro="Trappa kommer ut på en hylle over ei vollgrav. Over den står en hel borg under et steintak, "
          "med vindebroen heist og bueskyttere på brystvernet.",
    rooms=[
        Room("trapp", "Trappefoten", ("rect", 0, 0, 10, 8), y=0, h=4,
             note="Start. Trappa opp til nivå 6 i nord."),
        Room("ytre", "Ytterbredden", ("poly", [(-4, 12), (20, 9), (58, 11), (66, 16), (54, 22), (10, 22), (-2, 19)]),
             y=-1, h=16, at=(28, 16),
             note="Hylla foran vollgrava, med leirbålet til en hær som aldri kom inn. Herfra ser du hele borgen: "
                  "den heiste broen, brystvernet og lyset i kastellet."),
        Room("grav", "Vollgrava", ("poly", [(0, 23), (72, 23), (72, 30), (11, 30), (11, 98), (0, 98)]),
             y=-7, h=12, backdrop=True, at=(5, 60),
             note="Tørr grav med gjørme og råtne påler. Steiner faller fra skoddehullene over deg."),
        Room("skatt", "Raset hjørne", ("rect", 5, 105, 7, 6), y=-7, h=3, secret=True,
             note="Bak en raset mur i enden av grava: kiste med sjelden komponent."),
        Room("vesttarn", "Vesttårnet", ("circle", 15, 88, 5.5), y=-7, h=14,
             levels=[("ring", 15, 88, 3, 5.5, WALK)],
             note="Utfallsporten fra grava går inn i foten av tårnet. Vindeltrapp opp til brystvernet, dør inn til gården."),
        Room("gard", "Borggården", ("rect", 42, 61, 52, 54), y=-1, h=16, at=(38, 54),
             levels=[("rect", 42, 35.5, 52, 3, WALK), ("rect", 17.5, 61, 3, 54, WALK),
                     ("rect", 66.5, 61, 3, 54, WALK), ("rect", 42, 86.5, 52, 3, WALK)],
             note="Brostein, en tørr brønn og falmede faner. Steintaket bæres av murene. Brystvernet rundt (+5 m)."),
        Room("porthus", "Porthuset", ("rect", 42, 33, 12, 8), y=-1, h=10,
             levels=[("rect", 42, 33, 10, 6, WALK)],
             note="Vinsjrommet ligger oppe (+5 m), på høyde med brystvernet. Vinsjen senker broen og heiser gitteret."),
        Room("stall", "Stallen", ("rect", 58, 46, 14, 8), y=-1, h=5,
             note="Bakhold: Cardene står i båsene som hester og snur seg når du går forbi."),
        Room("kapell", "Borgkapellet", ("rect", 30, 74, 10, 10), y=-1, h=8,
             note="Lite kapell med knust glassmaleri. Lysestaken på alteret kan dreies."),
        Room("gravkammer", "Borgherrens grav", ("rect", 31, 104, 9, 6), y=-6, h=3, secret=True,
             note="Dreier du lysestaken, glir en steinhelle til side og en trapp går ned. Borgherrens grav: lore og varig styrke."),
        Room("kastell", "Kastellet", ("rect", 55, 72, 18, 16), y=-1, h=10,
             levels=[("rect", 55, 66, 18, 4, 4)],
             note="Hovedtårnet. Kortkapteinen holder hallen med to skjoldbærere; bueskyttere på galleriet (+4 m)."),
        Room("fangehull", "Fangehullet", ("rect", 53, 106, 24, 8), y=-8, h=3.5,
             note="Celler på begge sider; fangene er bare rustning nå. Fakler langs midtgangen viser veien."),
        Room("nedgang", "Nedgangen", ("circle", 73, 106, 5), y=-9, h=6,
             note="Trappa ned til nivå 8, med det kalde blå lyset."),
    ],
    links=[
        Link("trapp", "ytre", STAIRS, path=[(0, 10)]),
        Link("ytre", "grav", LADDER, path=[(4, 21)], width=1.5, note="stige ned i vollgrava"),
        Link("grav", "skatt", SECRET, path=[(5, 96)], note="raset mur, trekk mellom steinene"),
        Link("grav", "vesttarn", DOOR, path=[(5, 88)], width=2, note="utfallsporten"),
        Link("vesttarn", "porthus", STAIRS, path=[(17.5, 78), (17.5, 36), (36, 35.5)], width=2.5,
             note="vindeltrappa opp i tårnet og langs brystvernet (+5 m)"),
        Link("porthus", "ytre", SHORTCUT, path=[(42, 24)], width=4,
             note="vindebroen, senkes med vinsjen i porthuset"),
        Link("porthus", "gard", GATE, width=4, note="gitteret under porthuset, heises med vinsjen"),
        Link("vesttarn", "gard", DOOR, path=[(22, 82)], width=2, note="løkka: tårnet, brystvernet, porthuset og gården"),
        Link("gard", "stall", DOOR, width=2.5),
        Link("gard", "kapell", DOOR, width=2),
        Link("kapell", "gravkammer", SECRET, path=[(31, 92)], note="steinhella under alteret"),
        Link("gard", "kastell", DOOR, width=3),
        Link("kastell", "fangehull", GATE, path=[(55, 92)], width=2.5, note="kjellerporten, låst med kapteinens nøkkel"),
        Link("fangehull", "nedgang", DOOR, width=2.5),
    ],
    marks=[
        Mark(START, 0, 1),
        Mark(UP, -2, -3, "opp til nivå 6"),
        Mark(REST, 12, 15, "leirbålet"),
        Mark(MOB, 46, 15, "Card", n=2, patrol=[(32, 13), (58, 14)]),
        Mark(MOB, 5, 46, "tabell i gjørma", n=2),
        Mark(TRAP, 5, 64, "skoddehull"),
        Mark(MOB, 5, 78, "Card"),
        Mark(HINT, 5, 95, "trekk"),
        Mark(RARE, 5, 105, "Switch"),
        Mark(MOB, 15, 90, "Tabs", n=2),
        Mark(MOB, 17.5, 60, "Breadcrumb-bueskytter", n=3, patrol=[(17.5, 44), (17.5, 76)]),
        Mark(MOB, 28, 35.5, "Breadcrumb-bueskytter", n=2),
        Mark(MOB, 66.5, 54, "Breadcrumb-bueskytter", n=2, patrol=[(66.5, 40), (66.5, 82)]),
        Mark(LEVER, 45, 33, "vinsjen"),
        Mark(MOB, 39, 33, "Tabs"),
        Mark(MOB, 40, 50, "Card", n=3, patrol=[(26, 44), (46, 44), (46, 58), (26, 58)]),
        Mark(MOB, 58, 46, "Card i bakhold", n=3),
        Mark(CHEST, 63, 44, "høyet"),
        Mark(TRAP, 42, 61, "brønnen"),
        Mark(MOB, 28, 72, "Tabs", n=2),
        Mark(HINT, 33, 71, "trekk i lysene"),
        Mark(LEVER, 30, 76.5, "lysestaken"),
        Mark(LORE, 29, 104, "gravskriften"),
        Mark(BONUS, 33, 106, "+ liv"),
        Mark(ELITE, 55, 75, "Kortkapteinen (nøkkel)"),
        Mark(MOB, 50, 72, "Card", n=2),
        Mark(MOB, 58, 66, "Breadcrumb-bueskytter", n=2),
        Mark(CHEST, 62, 78),
        Mark(MOB, 50, 106, "modal", n=2),
        Mark(LORE, 60, 109, "rissene i veggen"),
        Mark(DOWN, 73, 106, "til nivå 8"),
    ],
    main=["trapp", "ytre", "grav", "vesttarn", "porthus", "gard", "kastell", "fangehull", "nedgang"],
    notes=[
        "Lærer avstandsfiender og høyde: bueskytterne på brystvernet skyter ned i grava og gården, "
        "så du må opp på muren og ta dem der.",
        "Landemerket er hele borgen sett fra ytterbredden: den heiste broen, faklene på brystvernet og det "
        "varme lyset i kastellvinduene, som er målet.",
        "Hvilestedet er leirbålet på ytterbredden, nær starten. Vinsjen i porthuset senker vindebroen: "
        "snarveien rett tilbake til leirbålet.",
        "Løkka: tårnet, brystvernet, porthuset og gården, med en dør fra gården tilbake inn i tårnet.",
        "Hemmelighetene: den rasede muren i enden av vollgrava (trekk, Switch) og lysestaken i kapellet "
        "som åpner trappa til borgherrens grav (lore, + liv).",
        "Fellene: steiner fra skoddehullene over grava, og fra taket over brønnen når du går over gården.",
        "Nye fiender: Card (skjoldbærer, må slås fra siden eller parères), Breadcrumb som kommer igjen fra nivå 2 (nå bueskytter i rekke, skyter "
        "en etter en langs brystvernet) og Tabs (vakt med tre ansikter som bytter angrep). Kortkapteinen er "
        "eliten med nøkkelen til kjellerporten.",
    ],
)
