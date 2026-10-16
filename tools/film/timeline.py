"""Tidslinja til åpningsfilmen (planen i `docs/film/plan-aaa.md`).

Hver innstilling har et Veo-klipp i `assets/film/veo/<navn>.mp4` (laget av
eieren fra første og siste bilde), eller et maleri som vises stille med en
langsom glidning til klippet finnes. Stemmen ligger i
`assets/film/voice/fortelleren.mp3`, klippet i replikker etter
`lines.json` (`voice`: hvilken replikk som begynner når innstillingen
begynner, eller en tid). Tidene er sekunder i den ferdige filmen.

`cut.py` leser dette og skriver filmen.
"""

# Replikkene i rekkefølge (indeksen brukes i `voice`), samme som i
# docs/film/prompter-innstillinger.md.
LINES = 25
# titles: (stil, tekst, fra, til, skrivetid): teksten skrives bokstav for bokstav over skrivetiden, i takt med replikken.

# Fargegraderingen per akt: (temperatur, metning, kontrast, vignett).
GRADES = {
    "gold": dict(temp=0.06, sat=1.08, con=1.04, vig=0.35),
    "cold": dict(temp=-0.08, sat=0.92, con=1.06, vig=0.4),
    "fire": dict(temp=0.04, sat=1.05, con=1.08, vig=0.45),
    "glitch": dict(temp=-0.02, sat=1.15, con=1.1, vig=0.5),
    "night": dict(temp=-0.06, sat=0.95, con=1.05, vig=0.45),
    "crypt": dict(temp=-0.04, sat=0.95, con=1.1, vig=0.6),
}

# Overgangen INN i innstillingen: "cut", ("mix", s), ("black", s) (ned i
# svart og opp igjen), ("white", s) (lynet).
SHOTS = [
    # navn, start, slutt, kilde (klipp eller stilbilde), overgang inn, gradering, lyder, tekst
    dict(name="00_doren", t=(0.0, 8.0), still="opening/begravelsen/bg.webp", move=((0.62, 0.6, 0.55), (0.62, 0.6, 0.45)), enter=("black", 2.0), grade="night", amb=["loop_owl", "loop_wind"], voice=[(0, 1.0), (1, 6.3)]),
    dict(name="01_riket", t=(8.0, 17.6), clip="01_riket", enter=("mix", 1.2), grade="gold", amb=["loop_birds", "loop_wind"], voice=[(2, 8.6)]),
    dict(name="04_skriptoriet_inne", t=(17.6, 26.2), clip="04_skriptoriet_inne", still="opening/gullalderen/bg.webp", move=((0.5, 0.42, 0.95), (0.45, 0.62, 0.72)), enter=("mix", 1.0), grade="gold", amb=["quill"], voice=[(3, 18.0)]),
    dict(name="05_ansgar", t=(26.2, 32.4), clip="05_ansgar", still="shots/05_ansgar.png", move=((0.7, 0.45, 0.8), (0.7, 0.42, 0.62)), enter=("mix", 0.8), grade="gold", voice=[(4, 26.4)]),
    dict(name="06_raadssalen", t=(32.4, 42.6), clip="06_raadssalen", still="intro/raadssal.webp", move=((0.5, 0.5, 1.0), (0.5, 0.5, 0.7)), enter=("black", 1.2), grade="cold", amb=["loop_wind"], voice=[(5, 32.9)]),
    dict(name="07_raadet", t=(42.6, 50.2), clip="07_raadet", still="shots/07_raadet.png", move=((0.5, 0.5, 1.0), (0.5, 0.52, 0.8)), enter=("mix", 0.8), grade="cold", voice=[(6, 43.0)]),
    dict(name="08_komponentboken", t=(50.2, 59.6), clip="08_komponentboken", still="shots/08_komponentboken.jpeg", move=((0.5, 0.5, 1.0), (0.3, 0.58, 0.62)), enter=("mix", 0.8), grade="cold", voice=[(7, 50.4), (8, 53.4), (9, 57.9)], titles=[("latin", "Omnia lingua Reactia.", 50.4, 53.2, 1.1), ("latin", "Omnia una compage.", 53.4, 57.6, 1.65), ("latin", "Nulla exceptio.", 57.9, 60.4, 0.85)]),
    dict(name="11_inkvisisjonen", t=(59.6, 68.0), clip="11_inkvisisjonen", still="shots/11_inkvisisjonen.png", move=((0.4, 0.55, 0.9), (0.6, 0.55, 0.8)), enter=("black", 1.0), grade="fire", amb=["loop_fire", "loop_wind"], voice=[(10, 59.9)], titles=[("heading", "Inkvisisjonen", 59.9, 64.0, 0.95)]),
    dict(name="13_i_rammen", t=(68.0, 74.0), clip="13_i_rammen", still="shots/13_i_rammen.png", move=((0.5, 0.5, 0.9), (0.5, 0.5, 0.7)), enter=("mix", 0.8), grade="fire", amb=["loop_fire"], voice=[(11, 68.1)]),
    dict(name="14_natta", t=(74.0, 85.6), clip="14_natta", still="shots/14_natta.png", move=((0.5, 0.5, 1.0), (0.5, 0.55, 0.78)), enter=("mix", 1.0), grade="night", amb=["quill", "loop_wind"], voice=[(12, 74.0)]),
    dict(name="15_tyngre", t=(85.6, 88.4), clip="08_komponentboken", clip_in=5.0, still="shots/08_komponentboken.jpeg", move=((0.3, 0.58, 0.62), (0.3, 0.58, 0.56)), enter=("mix", 0.6), grade="cold", voice=[(13, 85.7)]),
    dict(name="16_bruddet", t=(88.4, 95.0), clip="16_bruddet", still="opening/fallet/bg.webp", move=((0.5, 0.5, 1.0), (0.5, 0.5, 0.9)), enter="cut", grade="glitch", amb=["loop_cave"], voice=[(14, 88.6), (15, 91.5)], sfx=[("thunder", 88.4)]),
    dict(name="17_rammeverket", t=(95.0, 105.6), clip="17_rammeverket", still="shots/17_rammeverket.png", move=((0.6, 0.6, 0.85), (0.6, 0.45, 0.8)), enter=("white", 0.3), grade="night", amb=["loop_storm", "loop_rain"], voice=[(16, 95.3)], sfx=[("thunder", 95.0), ("thunder", 101.5)]),
    dict(name="18_kirkegaarden", t=(105.6, 122.0), clip="18_kirkegaarden", still="opening/begravelsen/mid.webp", move=((0.5, 0.5, 1.0), (0.56, 0.58, 0.5)), enter=("black", 1.4), grade="night", amb=["loop_owl", "loop_wind"], voice=[(17, 105.7), (18, 108.2), (19, 116.0)]),
    dict(name="20_doren_mures", t=(122.0, 125.4), still="opening/begravelsen/bg.webp", move=((0.62, 0.6, 0.45), (0.62, 0.6, 0.42)), enter=("mix", 1.0), grade="night", amb=["loop_wind"], voice=[(20, 122.9)], titles=[("heading", "Han kom aldri opp.", 122.9, 126.0, 1.3)]),
    dict(name="21_jorda", t=(125.4, 130.4), clip="21_jorda", still="intro/jordlag.webp", move=((0.5, 0.36, 0.62), (0.5, 0.74, 0.62)), enter=("mix", 1.2), grade="crypt", amb=["loop_drips"], voice=[(21, 125.5)]),
    dict(name="22_det_som_sover", t=(130.4, 135.8), clip="22_det_som_sover", still="shots/22_det_som_sover.jpeg", move=((0.5, 0.5, 1.0), (0.45, 0.6, 0.62)), enter=("mix", 0.8), grade="crypt", amb=["loop_cave", "loop_drips"], voice=[(22, 130.6)]),
    dict(name="23_veien", t=(135.8, 149.0), clip="23_veien", still="shots/23_veien.jpeg", move=((0.45, 0.5, 1.0), (0.6, 0.5, 0.78)), enter=("black", 1.6), grade="gold", amb=["loop_birds", "loop_wind"], voice=[(23, 136.2), (24, 142.6)], titles=[("title", "Fristad · Anno 1350", 144.0, 149.0, 2.0)]),
]

END = 149.0
FADE_OUT = 2.5  # filmen toner til svart (spillet står bak)

# Partituret (cuene i web/score.js) etter tidslinja.
MUSIC = [
    (0.0, "crypt"),
    (8.0, "dawn"),
    (17.6, "scriptorium"),
    (32.4, "council"),
    (50.2, "decree"),
    (59.6, "inquisition"),
    (74.0, "workarounds"),
    (88.4, "fall"),
    (105.6, "burial"),
    (125.4, "crypt"),
    (135.8, "gate"),
    (147.0, "stop"),
]
