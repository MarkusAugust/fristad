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
    # navn, start, slutt, kilde (klipp eller stilbilde), overgang inn, gradering, lyder, tekst.
    # Regelen (eieren 17. oktober): bildet av en ting kommer idet den nevnes, aldri før; en pause fylles av bildet
    # som står der eller av svart, aldri av neste bilde; rett klipp på ordet når stedet skifter. Tidene følger
    # innspillingen (lines.json, kontrollert mot en Whisper-transkripsjon): hvert klipp begynner 0.15 s før ordet.
    dict(name="00_doren", t=(0.0, 7.94), still="opening/begravelsen/bg.webp", move=((0.62, 0.6, 0.55), (0.62, 0.6, 0.45)), enter=("black", 2.0), grade="night", amb=["loop_owl", "loop_wind"], voice=[(0, 0.4), (1, 6.08)]),
    dict(name="01_riket", t=(7.94, 16.11), clip="01_riket", enter=("mix", 1.0), grade="gold", amb=["loop_birds", "loop_wind"], voice=[(2, 8.09)]),
    dict(name="04_skriptoriet_inne", t=(16.11, 23.41), clip="04_skriptoriet_inne", still="opening/gullalderen/bg.webp", move=((0.5, 0.42, 0.95), (0.45, 0.62, 0.72)), enter=("mix", 0.8), grade="gold", amb=["loop_fire"], voice=[(3, 16.26)]),
    dict(name="05_ansgar", t=(23.41, 32.48), clip="05_ansgar", still="shots/05_ansgar.png", move=((0.7, 0.45, 0.8), (0.7, 0.42, 0.62)), enter=("mix", 0.6), grade="gold", voice=[(4, 23.56)]),
    dict(name="07_raadet", t=(32.48, 46.11), clip="07_raadet", still="shots/07_raadet.png", move=((0.5, 0.5, 1.0), (0.5, 0.52, 0.8)), enter=("black", 0.6), grade="cold", amb=["loop_wind"], voice=[(5, 32.63), (6, 42.24)]),
    dict(name="08_komponentboken", t=(46.11, 59.16), clip="08_komponentboken", still="shots/08_komponentboken.jpeg", move=((0.5, 0.5, 1.0), (0.3, 0.58, 0.62)), enter="cut", grade="cold", voice=[(7, 53.73)], titles=[("latin", "Omnia lingua Reactia.", 53.73, 59.01, 2.42)]),
    dict(name="08b_seglet", t=(59.16, 68.23), still="shots/08_komponentboken.jpeg", move=((0.3, 0.58, 0.62), (0.3, 0.58, 0.5)), enter=("mix", 0.4), grade="cold", voice=[(8, 59.31), (9, 64.51)], titles=[("latin", "Omnia una compage.", 59.31, 64.21, 2.03), ("latin", "Nulla exceptio.", 64.51, 68.83, 1.91)]),
    dict(name="11_inkvisisjonen", t=(68.23, 76.81), clip="11_inkvisisjonen", still="shots/11_inkvisisjonen.png", move=((0.4, 0.55, 0.9), (0.6, 0.55, 0.8)), enter=("black", 0.6), grade="fire", amb=["loop_fire", "loop_wind"], voice=[(10, 68.38)], titles=[("heading", "Inkvisisjonen", 68.38, 72.38, 0.95)]),
    dict(name="13_i_rammen", t=(76.81, 81.63), clip="13_i_rammen", still="shots/13_i_rammen.png", move=((0.5, 0.5, 0.9), (0.5, 0.5, 0.7)), enter=("mix", 0.5), grade="fire", amb=["loop_fire"], voice=[(11, 76.96)]),
    dict(name="14_natta", t=(81.63, 92.16), clip="14_natta", still="shots/14_natta.png", move=((0.5, 0.5, 1.0), (0.5, 0.55, 0.78)), enter=("mix", 0.8), grade="night", amb=["loop_owl", "loop_fire"], voice=[(12, 81.78)]),
    dict(name="15_tyngre", t=(92.16, 95.42), clip="08_komponentboken", clip_in=5.0, still="shots/08_komponentboken.jpeg", move=((0.3, 0.58, 0.62), (0.3, 0.58, 0.56)), enter=("mix", 0.6), grade="cold", voice=[(13, 92.31)]),
    dict(name="16_bruddet", t=(95.42, 106.14), clip="16_bruddet", still="opening/fallet/bg.webp", move=((0.5, 0.5, 1.0), (0.5, 0.5, 0.9)), enter="cut", grade="glitch", amb=["loop_wind"], voice=[(14, 95.57), (15, 98.92)]),
    dict(name="17_rammeverket", t=(106.14, 120.02), clip="17_rammeverket", still="shots/17_rammeverket.png", move=((0.6, 0.6, 0.85), (0.6, 0.45, 0.8)), enter=("white", 0.3), grade="night", amb=["loop_storm", "loop_rain"], voice=[(16, 106.29), (17, 113.41)], sfx=[("thunder", 106.1), ("thunder", 111.1)]),
    dict(name="17a_gorvek_kommer", t=(120.02, 128.71), clip="17a_gorvek_kommer", still="shots/17a_gorvek_kommer.jpeg", move=((0.5, 0.55, 1.0), (0.5, 0.45, 0.8)), enter=("black", 0.5), grade="night", amb=["loop_storm", "loop_rain"], voice=[(18, 120.17)], sfx=[("thunder", 122.0)], titles=[("heading", "Gorvek", 120.17, 124.67, 0.7)]),
    dict(name="17b_gorvek", t=(128.71, 137.2), clip="17b_gorvek", still="shots/17b_gorvek.jpeg", move=((0.5, 0.5, 1.0), (0.45, 0.45, 0.8)), enter="cut", grade="night", amb=["loop_storm", "loop_rain"], sfx=[("thunder", 129.2)]),
    dict(name="18b_delene", t=(137.2, 148.78), clip="18b_delene", still="shots/18b_delene.jpeg", move=((0.4, 0.55, 1.0), (0.6, 0.55, 0.82)), enter=("mix", 1.0), grade="night", amb=["loop_owl", "loop_wind"], voice=[(19, 137.35)]),
    dict(name="18_kirkegaarden", t=(148.78, 155.67), clip="18_kirkegaarden", still="opening/begravelsen/mid.webp", move=((0.5, 0.5, 1.0), (0.56, 0.58, 0.5)), enter=("mix", 1.0), grade="night", amb=["loop_owl", "loop_wind"], voice=[(20, 148.93)]),
    dict(name="20_doren_mures", t=(155.67, 158.65), still="opening/begravelsen/bg.webp", move=((0.62, 0.6, 0.45), (0.62, 0.6, 0.42)), enter=("mix", 0.8), grade="night", amb=["loop_wind"], voice=[(21, 155.82)], titles=[("heading", "Han kom aldri opp.", 155.82, 159.25, 1.19)]),
    dict(name="21_jorda", t=(158.65, 164.17), clip="21_jorda", still="intro/jordlag.webp", move=((0.5, 0.36, 0.62), (0.5, 0.74, 0.62)), enter=("mix", 1.0), grade="crypt", amb=["loop_drips"], voice=[(22, 158.8)]),
    dict(name="22_det_som_sover", t=(164.17, 174.99), clip="22_det_som_sover", still="shots/22_det_som_sover.jpeg", move=((0.5, 0.5, 1.0), (0.45, 0.6, 0.62)), enter=("mix", 0.8), grade="crypt", amb=["loop_cave", "loop_drips"], voice=[(23, 164.32), (24, 169.7)]),
    dict(name="23_veien", t=(174.99, 189.2), clip="23_veien", still="shots/23_veien.jpeg", move=((0.45, 0.5, 1.0), (0.6, 0.5, 0.78)), enter=("black", 1.6), grade="gold", amb=["loop_birds", "loop_wind"], voice=[(25, 175.14), (26, 182.36)], titles=[("title", "Fristad · Anno 1350", 184.2, 189.2, 2.0)]),
]

END = 189.2
FADE_OUT = 2.5  # filmen toner til svart (spillet står bak)

# Partituret (cuene i web/score.js) etter tidslinja.
MUSIC = [
    (0.0, "crypt"),
    (7.8, "dawn"),
    (16.0, "scriptorium"),
    (38.0, "council"),
    (46.2, "decree"),
    (64.2, "inquisition"),
    (76.7, "workarounds"),
    (92.0, "fall"),
    (106.0, "fall"),
    (137.0, "burial"),
    (158.5, "crypt"),
    (174.8, "gate"),
    (187.2, "stop"),
]

# Lydsporet fra Suno (eieren 17. oktober, `assets/music/film/NN_navn.mp3`, 2–3,5 min hver):
# (fil, fra i sporet, fra i filmen, til i filmen, inntoning, uttoning). Bitene overlapper
# med toningene sine, så de glir over i hverandre. Mangler mappa, brukes `target/film/music.wav`.
SCORE = [
    ("04_krypten", 0.0, 0.0, 9.5, 1.5, 2.0),            # døra: det stille anslaget
    ("01_riket", 0.0, 7.94, 34.48, 2.5, 3.0),     # riket, Skriptoriet, Ansgar
    ("02_raadet", 0.0, 31.979999999999997, 94.16, 2.5, 3.0),  # rådet, budet, Inkvisisjonen, omveiene (bygger)
    ("03_bruddet", 110.0, 92.16, 137.7, 2.0, 1.2),  # bruddet, Rammeverket, Gorvek: slutthugget lander når hugget er over
    ("04_krypten", 160.0, 137.2, 176.69, 1.5, 3.0),   # prosesjonen, døra mures, jorda, hjertet
    ("05_veien", 177.0, 174.99, 192.0, 2.0, 2.5),        # veien til porten, sporets egen slutt
]
