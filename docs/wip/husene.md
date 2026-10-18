# Husene: trapper, trapphull og småting ute (11. oktober)

Oppgave fra eieren (tre feil, løses med felles regler i byggesettet):
1. Trappene (Vertshuset, Apoteket, Skriptoriet): hoderom, trinnmål, fritt gulv ved foten og toppen, gelender ved hullet. Regler i `kit.stairs`, feiler høyt om de ikke kan oppfylles.
2. Glitch rundt trapphullet oppe i Vertshuset (flater i samme plan).
3. Småting ute (tønner, vedstabel) bygd inn i husmodellen svever når bakken skrår: `town.prop` i Blender → `props` i `<navn>_inside.json` → `building()` setter dem på bakken.
Tillegg (etterpå): vinduskarmer felles i `kit.window`.

## Logg
- Lest koden. Funn: Vertshuset 13 trinn à 25 cm (for høyt), bare 0,5 m fritt ved foten (veggen i x = 5,7), trinnene under bjelkelaget har 1,76 m hoderom. Skriptoriet 13 trinn à 21,8 cm, 0,8 m til veggen ved foten. Apoteket er innenfor (19,6 cm / 27,9 cm).
- kit.py: `RISE_MAX`, `TREAD_MIN`, `HEADROOM`, `LANDING`, `STAIR_MARGIN`, `stair_steps`, `stair_run`, `stair_well`; `stairs(... n=None, room=, under=, well=)` med assert; `floor_frame` (etasjeskille som rammer rundt hullet) og `well_rail` (gelender ved hullet). Vangene ender under gulvet oppe.
- town.py: `prop(navn, x, y, yaw)` → `props` i `<navn>_inside.json`. town.rs: `Inside.props`, `house_prop` (på bakken, kollider fra målene, `HOUSE_PROPS`).
- Vertshuset: 17 trinn (0,28..4,7), hull 0,28..4,27 (`stair_well`), kjellertrappa 14 trinn (−1,44..2,2), bjelkeendene ute slutter i veggen og ligger under gulvbordene (`JETTY_T` 6 cm) – det var de som lå i plan med gulvet oppe og stakk inn i hullet. Tønnene foran er `town.prop`. Eksportert.
- Apoteket: foten 1 m fra sørveggen, 14 trinn, hull fra `stair_well`; Smia: vedstabelen i bakgården er `town.prop("woodpile")`. Eksportert.
- Skriptoriet (sacred.py, bare konstantene og trappekallet): 15 trinn, foten 1 m fra sørmuren, galleriets forkant i y = 1,5.
- Testen `trappene_har_hoderom` nederst i src/tests.rs.
- Venter: en annen hjelper har en kompileringsfeil i `enclose` i town.rs (ikke min).
- Veggkolliderne er 5 cm tykkere enn pussen (`_wall_box`): `kit.WALL_PAD`, trappeføttene står `LANDING + WALL_PAD` fra veggen. `trappene_har_hoderom` og `man_kan_ga_opp_trappene` grønne.
- Fant at `layer("brick")` i Apotekets ovn (fra ecdb6c5) ikke er noe teksturlag (`alt_er_ekte_modeller_med_teksturer` falt ved re-eksport): byttet til laget `chimney`.
- Vinduskarmene (tillegget): `kit.window_frame` (karm 9 cm, 5 cm ut/3 cm inn, vannbrett med 9° helling), `kit.mullions` (kryss / to loddrette / én), `window(..., inside=True)` i husene med interiør. Vertshuset og Apoteket er eksportert med dem; Smia, hjemmet og de tjue husene gjenstår.
- Sikkerhetskopi: wip/husene (b96c0ba).
- Smia, hjemmet og de tjue husene eksportert med vinduskarmene (husene 3 248–10 364 trekanter; house_l var ~10 000 fra før, +3 %). `cargo test --release`: 47 bestått. Bilder i `shots/husene/` (for_ark, etter_ark, etter_props_ark, vinduer_ark, etter_vedstabel_inne_s).
- Ikke mitt: `home.py` skriver `gallery_<komponent>.glb` (utracket), og en annen hjelper har slettet `gallery_piece.glb` i indeksen.

## Status
Ferdig: trappereglene, hullene, småting ute, vinduskarmene. Ingenting pushet til claude/detaljnivaa. Sikkerhetskopi på wip/husene.
