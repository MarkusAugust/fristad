"""Nivå 8: Ridderhallen.

Hjertet i den sunkne borgen: en forhall med våpenkammer, pendelgangen mellom
riddersstatuene, den store søylesalen med galleri, kapellet med hvilestedet, den runde
riddersalen, gildehallen og tronsalen der Navbaren venter.
"""

from plan import *

GALLERY = 3  # galleriene over salene

LEVEL = Level(
    n=8,
    name="Ridderhallen",
    theme="Borgen under jorda",
    intro="Trappa ender i forhallen til ridderne. Rad på rad av rustninger står langs veggene, og "
          "foran deg svinger pendlene i gangen mellom statuene.",
    rooms=[
        Room("forhall", "Forhallen", ("rect", 0, 0, 12, 10), y=0, h=7,
             note="Start. Trappa opp til nivå 7 i nord. Faner med falmede våpen."),
        Room("vapen", "Våpenkammeret", ("rect", -21, 0, 14, 10), y=0, h=5,
             note="Stativer med spyd og skjold. To rustninger våkner når du tar fra kista."),
        Room("smed", "Smedkammeret", ("rect", -36, 1, 8, 8), y=0, h=3.5, secret=True,
             note="Bak en vegg med rustflekker som renner ut av fugene: smedens ambolt, lore og varig styrke."),
        Room("pendel", "Pendelgangen", ("rect", 0, 19, 5, 18), y=0, h=9,
             note="Statuer i nisjer på begge sider; tre pendler svinger tvers over gangen. Rytmen høres før du ser dem."),
        Room("vakt", "Vaktgangen", ("poly", [(-24, 6), (-18, 6), (-18, 38), (-24, 40)]), y=-1, h=3.5,
             at=(-21, 22),
             note="Smal gang fra våpenkammeret til kapellet, rundt pendlene. Patrulje av Card."),
        Room("soylesal", "Søylesalen", ("rect", 0, 46, 30, 28), y=-2, h=12,
             levels=[("rect", -13.5, 46, 3, 28, GALLERY), ("rect", 13.5, 46, 3, 28, GALLERY)],
             note="To rader søyler med ryttere i stein. Galleri langs begge langveggene (+3 m) med bueskyttere."),
        Room("kapell", "Ridderkapellet", ("poly", [(-20, 40), (-30, 40), (-34, 43), (-35, 46), (-34, 49), (-30, 52), (-20, 52)]),
             y=-2, h=9, at=(-26, 46),
             note="Hvilestedet midtveis, alteret under et tak av skjold med våpenmerker."),
        Room("rundsal", "Det runde bord", ("circle", 34, 46, 10), y=-2, h=12,
             levels=[("ring", 34, 46, 7.5, 10, GALLERY)],
             note="Rund sal med et steinbord og tolv ridderstatuer. Tre av statuene kan dreies; gallerier rundt (+3 m)."),
        Room("skatt", "Skattkammeret", ("rect", 34, 27, 8, 6), y=GALLERY, h=3, secret=True,
             note="Når de tre statuene ser mot bordet, åpner en dør seg fra galleriet."),
        Room("gilde", "Gildehallen", ("rect", 32, 78, 22, 14), y=-2, h=8,
             levels=[("rect", 32, 72.5, 22, 3, GALLERY)],
             note="Lange bord og veltede kar. Spillemannsgalleri over døra (+3 m). Tabs i bakhold under bordene."),
        Room("vin", "Vinkjelleren", ("rect", 30, 99, 11, 7), y=-7, h=3, secret=True,
             note="En lem under det største karet gir etter. Tønner, lore, kiste og en stige opp i forværelset."),
        Room("forvaerelse", "Forværelset", ("poly", [(-3, 76), (3, 76), (7, 79), (7, 85), (3, 88), (-3, 88), (-7, 85), (-7, 79)]),
             y=-2, h=6, at=(0, 82),
             note="Foran tronsalen. Gitteret mot søylesalen heises med en spak her: snarveien til kapellet."),
        Room("tronsal", "Tronsalen", ("poly", [(-13, 96), (13, 96), (13, 115), (8, 123), (0, 126), (-8, 123), (-13, 115)]),
             y=-4, h=16, at=(0, 109), levels=[("rect", 0, 119, 10, 5, -3)],
             note="Høy sal med søyler i to rader og lenker mellom dem. Tronen på en forhøyning i sør. Navbaren."),
        Room("nedgang", "Nedgangen", ("rect", 0, 131, 8, 6), y=-5, h=5,
             note="Bak tronen; døra åpner seg når Navbaren faller. Det blå lyset synes gjennom sprekkene i døra."),
    ],
    links=[
        Link("forhall", "vapen", DOOR, width=2.5),
        Link("vapen", "smed", SECRET, note="rustflekkene"),
        Link("forhall", "pendel", DOOR, width=3),
        Link("pendel", "soylesal", STAIRS, width=3),
        Link("vapen", "vakt", DOOR, width=2),
        Link("vakt", "kapell", STAIRS, width=2, note="løkka rundt pendlene"),
        Link("soylesal", "kapell", DOOR, width=3),
        Link("soylesal", "rundsal", DOOR, width=4),
        Link("rundsal", "skatt", SECRET, note="døra fra galleriet, åpnes av statuene"),
        Link("rundsal", "gilde", HALL, path=[(34, 66)], width=3),
        Link("gilde", "vin", FALL, path=[(30, 88)], note="lemmen under karet"),
        Link("vin", "forvaerelse", LADDER, path=[(16, 100), (6, 88)], width=1.5),
        Link("gilde", "forvaerelse", DOOR, path=[(14, 80)], width=3),
        Link("forvaerelse", "soylesal", SHORTCUT, width=3, note="gitteret, heises fra forværelset"),
        Link("forvaerelse", "tronsal", DOOR, width=4),
        Link("tronsal", "nedgang", DOOR, width=2.5),
    ],
    marks=[
        Mark(START, 0, 1),
        Mark(UP, 0, -3.5, "opp til nivå 7"),
        Mark(MOB, -17, 3, "Card", n=2),
        Mark(CHEST, -22, -3),
        Mark(HINT, -27, -2, "rust"),
        Mark(LORE, -38, -1, "bok"),
        Mark(BONUS, -38, 3, ""),
        Mark(TRAP, 0, 15, "pendel"),
        Mark(TRAP, 0, 23, "pendel"),
        Mark(MOB, -21, 24, "Card", n=2, patrol=[(-21, 10), (-21, 36)]),
        Mark(MOB, 0, 40, "Card", n=3, patrol=[(-6, 36), (6, 36), (6, 56), (-6, 56)]),
        Mark(MOB, -13.5, 50, "Breadcrumb", n=2),
        Mark(MOB, 13.5, 42, "Breadcrumb", n=2),
        Mark(MOB, 0, 54, "modal"),
        Mark(REST, -30, 46, "alteret"),
        Mark(MOB, 34, 46, "Tabs", n=3),
        Mark(LEVER, 27, 41, "statue"),
        Mark(LEVER, 41, 41, "statue"),
        Mark(LEVER, 34, 54, "statue"),
        Mark(HINT, 31, 38, "slitte spor i gulvet"),
        Mark(RARE, 34, 27, "Avatar"),
        Mark(MOB, 28, 80, "Tabs i bakhold", n=3),
        Mark(MOB, 32, 72.5, "Breadcrumb", n=2),
        Mark(MOB, 36, 80, "Card"),
        Mark(CHEST, 41, 83),
        Mark(HINT, 30, 84, "hult under karet"),
        Mark(LORE, 27, 99, "kjellermesterens regnskap"),
        Mark(CHEST, 34, 101),
        Mark(LEVER, 3, 78, "gitteret"),
        Mark(TRAP, 0, 91, "pendel over døra"),
        Mark(MOB, -3, 84, "Card", n=2),
        Mark(BOSS, 0, 112, "Navbaren"),
        Mark(DOWN, 0, 131, "til nivå 9"),
    ],
    main=["forhall", "pendel", "soylesal", "rundsal", "gilde", "forvaerelse", "tronsal", "nedgang"],
    notes=[
        "Lærer rytme og timing: pendlene først i en tom gang, så i forværelset med fiender rundt deg. "
        "Den som vil, kan gå rundt pendlene gjennom vaktgangen.",
        "Landemerket er søylesalen med rytterstatuene; fra galleriet ser du gitteret mot forværelset og det "
        "røde lyset fra tronsalen bak.",
        "Hvilestedet er ridderkapellet midtveis. Gitteret fra forværelset til søylesalen er snarveien før bossen "
        "(30 m til kapellet).",
        "Hemmelighetene: rustflekkene i våpenkammeret (smedkammeret, + utholdenhet), de tre statuene i Det runde "
        "bord som må se mot bordet (skattkammeret, Avatar), og lemmen under karet i gildehallen (vinkjelleren "
        "med lore og stige opp i forværelset).",
        "Fiendene: Card, Breadcrumb og Tabs fra nivå 7 kommer sterkere og i større flokker, med bueskyttere "
        "på galleriene over nærkampen. Ingen elite.",
        "Bossen: Navbaren er en ridder av lenker. Den kaster lenker rundt søylene og drar deg mot seg; "
        "bryt søylene, eller hold deg bak dem.",
    ],
)
