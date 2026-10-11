# Lærdom: feil som er blitt regler

Hver feil som oppdages (av en kontroll, vurdereren eller eieren) får en årsak og en regel i felles kode med en test, og føres her (`docs/plan.md`, prinsipp 3). Tersklene strammes, aldri slakkes.

## 11. oktober

- **Teksturlag som ikke finnes, ga en stille flat farge** (smia brukte «bark»). Regel: `tools/models/roles.py` stopper eksporten når laget ikke finnes i `assets/textures/`, og motoren laster ikke en modell med ukjent lag (`modellene_har_roller`).
- **Ryddingen fjernet hjelpere byggesettet brukte** (`town.SMOKE`, `LIT`, `RELIEF`), så husene ikke kunne eksporteres. Regel: `tools/models/check_scripts.py` (testen `blenderskriptene_henger_sammen`).
- **Materialrollen ble gjettet ut fra navnet** i motoren. Regel: rollene er data i glb-fila (`extras.fristad`), og motoren sammenligner ingen navn (`lasteren_leser_rollene_og_gjetter_ikke`).
- **Teksturkontrollen (P-T), kalibrert mot bildene:**
  - Skjøten målt som snitt over hele kanten druknet skjøter som bare synes et stykke, og et forholdstall mot nabosteget overdrev på glatte flater (puss). Regel: den verste åttendedelen langs skjøten mot 99-persentilen av de naturlige overgangene i bildet; skjøter synes med øyet fra rundt 1,0 (`MAX_SEAM`).
  - Laplace-variansen straffet glatte materialer. Regel: hvor mye de sterkeste kantene mister ved 1 piksel uskarphet (`MIN_CRISP` 1,40; oppskalerte bilder ligger under).
  - Ramme målt som forskjell mellom kantbånd slo ut på mønstre som kuttes ved kanten (takstein, brostein). Regel: en ramme er en stripe som er jevn langs hele kanten (`MAX_FRAME`).
  - Perspektiv kan ikke skilles fra mønstre som er grovere på én side med vilje (`core_stone`) med enkle mål. Regel: perspektiv vurderes med øyet i den estetiske kontrollen; P-T har ingen perspektivsjekk.
  - Arkene (dører, list, skilt, vinduer) er ikke fliser. Regel: slaget står i `assets/catalog/textures.json`, og P-T sjekker arkene bare for størrelse og skarphet.
- **Kildebildet for kryptpussen var fortsatt stjernetapetet** som eieren underkjente, under samme navn som laget. Regel: katalogen (`docs/katalog.md`) viser kilden, laget og eierens dom side om side.
