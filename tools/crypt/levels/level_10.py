"""Nivå 10: Kjernen.

En trakt av terrasser som skrur seg nedover mot rotunden i bunnen, der Rammeverket
venter. Du ser rotunden fra starten og går rundt den tre ganger, hver gang lavere og
trangere: den ytre spiralen, den midtre og den indre. Ingen trapp ned; dette er bunnen.
"""

import math

from plan import *

C = (0.0, 46.0)  # midten av trakten og rotunden


def polar(r, deg):
    """Punktet r meter fra midten, deg grader med klokka fra nord."""
    a = math.radians(deg)
    return (round(C[0] + r * math.sin(a), 2), round(C[1] - r * math.cos(a), 2))


def along(r, a0, a1, step=10):
    """Patrulje fram og tilbake langs en terrasse."""
    n = max(2, int(abs(a1 - a0) / step) + 1)
    fwd = [polar(r, a0 + (a1 - a0) * i / (n - 1)) for i in range(n)]
    return fwd + fwd[-2:0:-1]


def arc(r0, r1, a0, a1, step=8):
    """Et buet stykke av en terrasse mellom radius r0 og r1, fra vinkel a0 til a1."""
    n = max(2, int(abs(a1 - a0) / step) + 1)
    angs = [a0 + (a1 - a0) * i / (n - 1) for i in range(n)]
    return ("poly", [polar(r1, a) for a in angs] + [polar(r0, a) for a in reversed(angs)])


# Terrassene: rotunden r 12, den indre 16–21, den midtre 25–31, den ytre 35–41.
OUT0, OUT1, OUTM = 35, 41, 38
MID0, MID1, MIDM = 25, 31, 28
IN0, IN1, INM = 16, 21, 18.5

LEVEL = Level(
    n=10,
    name="Kjernen",
    theme="Den korrupte kjernen",
    intro="Trappa ender på kanten av en trakt. Langt nede lyser rotunden hvitt, og terrassene skrur seg "
          "ned mot den i tre runder. Alt er plast og sømmer nå; steinen er bare et minne.",
    rooms=[
        Room("port", "Kanten", ("rect", 0, 0, 10, 8), y=0, h=5,
             note="Start. Trappa opp til nivå 9 i nord. Herfra ser du rett ned i trakten og rotunden."),
        Room("ytre_o", "Den ytre spiralen (øst)", arc(OUT0, OUT1, 8, 172), y=-3, h=12, at=polar(OUTM, 60),
             note="Bred terrasse som heller fra 0 til -6 m. Innerkanten er et stup ned til den midtre spiralen, "
                  "sperret av lysende sømmer."),
        Room("hvile", "Det siste alteret", ("rect", 43, 15, 9, 8), y=-2, h=5,
             note="Hvilestedet, i en nisje i traktveggen nær starten. Lysheisen kommer opp hit."),
        Room("buffer", "Det glemte bufferet", ("rect", 37, 83, 8, 7), y=-5, h=3, secret=True,
             note="Bak en vegg som henger igjen ett bilde etter resten når du snur deg. Sjelden komponent."),
        Room("ytre_v", "Den ytre spiralen (vest)", arc(OUT0, OUT1, 188, 352), y=-9, h=12, at=polar(OUTM, 240),
             note="Fra -6 til -12 m. Plasten er sprukket her, og noen steder er gulvet bare et rutenett av sømmer."),
        Room("cache", "Cachen", ("rect", -45, 19, 10, 8), y=-9, h=4,
             note="Stabler av halvferdige komponenter. Bakhold av Skeleton loaders som laster inn rundt deg."),
        Room("kilde", "Kildekoden", ("rect", -39, 81, 9, 7), y=-20, h=3.5, secret=True,
             note="Du faller gjennom det sprukne gulvet og lander i et rom fullt av skrift på veggene: hvordan "
                  "Rammeverket ble til. Lore og varig styrke; en trapp opp til Kompilatoren."),
        Room("komp", "Kompilatoren", arc(MID0, MID1, 188, 352), y=-16, h=8, at=polar(MIDM, 300),
             note="Den vestre halvdelen av den midtre spiralen: et verksted der Grids og Cards settes sammen på "
                  "samlebånd. Sidegren og løkke."),
        Room("midt", "Den midtre spiralen", arc(MID0, MID1, 8, 172), y=-15, h=10, at=polar(MIDM, 120),
             note="Fra -12 til -18 m. Tre sømnoder i innerveggen lyser i takt."),
        Room("node", "Nodekammeret", arc(IN0, IN1, 24, 156), y=-20, h=4, secret=True, at=polar(INM, 90),
             note="Den østre halvdelen av den indre ringen. Åpner seg når de tre sømnodene er slått ut: varig "
                  "styrke og en kiste."),
        Room("indre", "Den indre spiralen", arc(IN0, IN1, 188, 352), y=-24, h=8, at=polar(INM, 270),
             note="Fra -20 til -28 m, trang og bratt. Rotunden lyser opp gjennom sømmene i gulvet."),
        Room("rotunde", "Rotunden", ("circle", C[0], C[1], 12), y=-32, h=30,
             levels=[("ring", C[0], C[1], 10, 12, -29)],
             note="Bossarenaen: rund sal med en hevet kant (-29 m). Rammeverket står midt i lyset."),
    ],
    links=[
        Link("port", "ytre_o", STAIRS, path=[polar(OUTM, 8), polar(OUTM, 45)], width=4),
        Link("ytre_o", "hvile", DOOR, path=[polar(OUTM, 55)], width=2.5),
        Link("ytre_o", "buffer", SECRET, path=[polar(OUTM, 100), polar(OUTM, 140)], note="veggen som henger igjen"),
        Link("ytre_o", "ytre_v", DOOR, path=[polar(OUTM, 100), polar(OUTM, 135), polar(OUTM, 180), polar(OUTM, 225)], width=4,
             note="sørporten mellom de to halvdelene"),
        Link("ytre_v", "cache", DOOR, path=[polar(OUTM, 270), polar(OUTM, 300)], width=2.5),
        Link("ytre_v", "kilde", FALL, path=[polar(OUTM, 232)], note="det sprukne gulvet"),
        Link("kilde", "komp", STAIRS, path=[polar(MIDM, 232), polar(MIDM, 270)], width=1.5),
        Link("ytre_v", "komp", STAIRS, path=[polar(OUTM, 270), polar(MIDM, 270)], width=2.5, note="trapp ned i terrasseveggen; løkka"),
        Link("ytre_v", "midt", STAIRS, path=[polar(OUTM, 270), polar(OUTM, 315), polar(OUTM, 352), polar(MIDM, 8), polar(MIDM, 45), polar(MIDM, 90)],
             width=3, note="rampen ned i nord"),
        Link("midt", "komp", SHORTCUT, path=[polar(MIDM, 150), polar(MIDM, 180), polar(MIDM, 225), polar(MIDM, 270)], width=3,
             note="sømsperren i sør, slås ut fra den midtre spiralen"),
        Link("midt", "node", SECRET, path=[polar(23, 105)], note="åpnes av de tre sømnodene"),
        Link("midt", "indre", STAIRS, path=[polar(MIDM, 150), polar(MIDM, 172), polar(INM, 188), polar(INM, 225)],
             width=3, note="trappa ned i sør"),
        Link("indre", "rotunde", STAIRS, path=[polar(INM, 315), polar(INM, 352)], width=4,
             note="den brede trappa ned i rotunden"),
        Link("indre", "hvile", SHORTCUT, path=[polar(INM, 352), (12, 22), (34, 17)], width=2,
             note="lysheisen: en bro av lys som stiger fra foten av den indre spiralen til alteret"),
    ],
    marks=[
        Mark(START, 0, 1),
        Mark(UP, -2, -3, "opp til nivå 9"),
        Mark(MOB, *polar(OUTM, 35), "Grid", n=2, patrol=along(OUTM, 15, 85)),
        Mark(REST, 45, 15, "alteret"),
        Mark(MOB, *polar(OUTM, 100), "Card", n=2),
        Mark(TRAP, *polar(OUTM, 122), "sømmer som slår"),
        Mark(HINT, *polar(OUTM, 135), "henger igjen"),
        Mark(RARE, 38, 84, "Carousel"),
        Mark(MOB, *polar(OUTM, 205), "Breadcrumb", n=2),
        Mark(MOB, *polar(OUTM, 285), "Skeleton loader", n=3, patrol=along(OUTM, 255, 330)),
        Mark(HINT, *polar(OUTM, 228), "sprukket gulv"),
        Mark(MOB, -45, 19, "Skeleton loader i bakhold", n=3),
        Mark(CHEST, -48, 22),
        Mark(LORE, -41, 80, "kildekoden"),
        Mark(BONUS, -37, 83, "+ liv"),
        Mark(MOB, *polar(MIDM, 250), "Grid", n=2, patrol=along(MIDM, 200, 340)),
        Mark(MOB, *polar(MIDM, 300), "Tabs", n=2),
        Mark(CHEST, *polar(MIDM, 320), "samlebåndet"),
        Mark(MOB, *polar(MIDM, 22), "Popover", n=2),
        Mark(LEVER, *polar(MID0 + 0.5, 60), "node"),
        Mark(LEVER, *polar(MID0 + 0.5, 105), "node"),
        Mark(LEVER, *polar(MID0 + 0.5, 150), "node"),
        Mark(MOB, *polar(MIDM, 140), "Accordion", n=2),
        Mark(MOB, *polar(MIDM, 165), "modal"),
        Mark(TRAP, *polar(MIDM, 75), "glitch-fliser"),
        Mark(BONUS, *polar(INM, 70), "+ utholdenhet"),
        Mark(CHEST, *polar(INM, 110)),
        Mark(MOB, *polar(INM, 240), "Card", n=2),
        Mark(MOB, *polar(INM, 300), "Popover", n=2),
        Mark(BOSS, C[0], C[1] + 2, "Rammeverket"),
        Mark(LORE, C[0] - 6, C[1] + 6, "siste side"),
    ],
    main=["port", "ytre_o", "ytre_v", "midt", "indre", "rotunde"],
    notes=[
        "Lærer ingenting nytt; nivået prøver alt: spiralen går rundt rotunden tre ganger, hver runde lavere, "
        "trangere og med flere fiender.",
        "Landemerket er rotunden selv: du ser den lyse fra kanten ved starten, og den er aldri ute av syne. "
        "Hovedveien går med klokka hele veien; sømmene i gulvet lyser varmt i gåretningen.",
        "Hvilestedet er Det siste alteret ved den ytre spiralen, nær starten. Lysheisen fra foten av den "
        "indre spiralen er snarveien tilbake før sluttbossen.",
        "Løkka: fra den ytre spiralen ned trappa til Kompilatoren, og sømsperren i sør slås ut fra den midtre.",
        "Hemmelighetene: veggen som henger igjen ett bilde (Det glemte bufferet, Carousel), det sprukne gulvet "
        "ned til Kildekoden (lore om Rammeverket, + liv) og de tre sømnodene som åpner Nodekammeret "
        "(+ utholdenhet).",
        "Fiendene: alle temaene kommer tilbake korrupte og sterkere: Grid, Skeleton loader og Popover fra "
        "nivå 9, Card, Tabs og Breadcrumb fra borgen, Accordion fra grotta og modalen fra gravkjelleren. "
        "Ingen elite: sluttbossen er det eneste som står igjen.",
        "Bossen: Rammeverket. Ingen trapp ned; når det faller, åpnes portalen hjem midt i rotunden.",
    ],
)
