# TRELLIS: fiendene

Slik lages en ny fiende fra TRELLIS. Alle fiendene på lista er laget (25. oktober), og Icon Button er droppet (eieren), så lista er tom. Navnene og utseendet følger kanon (`docs/kanon.md`): komponenter fra mønsterboka som ble spent fast i kongens rammeverk, fikk Kontorets regler stemplet på seg og vred seg til noe som kryper. Formen og fargene følger fiendene som finnes (`assets/models/mobs/`, bildene i `assets/textures/monsters/ref/`), så rollelisten henger sammen.

**Slik gjør du det:**

1. Lag ETT bilde med prompten (ikke ark med flere retninger: TRELLIS dikter selv sidene og ryggen, og vi maler aldri bilder på formen). Sjekk at armene står fri av kroppen, at beina er fra hverandre og at ingenting tynt (spyd, horn, kjettinger) er blitt borte; lag det på nytt ellers.
2. Lagre bildet som `assets/textures/monsters/ref/<id>_front.png`.
3. Kjør bildet gjennom https://huggingface.co/spaces/microsoft/TRELLIS.2 med **Resolution 1536, Decimation Target 500000, Texture Size 4096** (nytt frø om formen eller ansiktet blir dårlig), og last opp glb-fila som `<id>.glb`. Den legges i `assets/textures/monsters/3d/<id>.glb`.
4. Jeg gjør den spillklar med TRELLIS-teksturen som den er (samme steg som `folk3d.py`/`item3d.py`: lukket skall, forenkling til trekantbudsjettet), rigger den med skjelettet og klippene fra `tools/models/mob_motion.py` og viser deg den før den tas i bruk.

Tobeinte fiender står i A-stilling forfra; fiender uten armer eller med mange bein tas fra siden, litt skrått. Våpen er grodd fast i kroppen eller henger på ryggen, aldri i hendene.

| Nivå | Størrelse | Kjennetegn |
|---|---|---|
| Vanlig | kne- til mannshøy | ett hovedmateriale, enkel silhuett, ett øye |
| Elite | mannshøy eller mer | rustning, en glødende jernnøkkel grodd inn i kroppen (bærer nøkkelen) |
| Boss | 3–5 ganger et menneske | flere deler, glødende kjerne, kongens segl |
