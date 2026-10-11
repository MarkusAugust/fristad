# sheet2model: frittstående pakke for rekonstruksjon fra forbildeark

Start 11. okt (etter planen i `docs/rekonstruksjon-plan.md`). Pakken: `sheet2model/` i rota.
Mellomfiler i `target/sheet2model/<navn>/`, dybdebuffer i `target/sheet2model/cache/depth/`.

## Logg
- Lest: rekonstruksjon.md, planen, hode_z.md, hele tools/reconstruct (2 300 linjer), sheets.json, prompter-folk.md, ref/README.md.
- Plan for pakken: config (Job-objekt i stedet for globale J.*), sheet, depth (buffer på sha av ruta), fuse (generell
  kalibrering mot det visuelle skallet, så en rute ovenfra også kan brukes), align (frihetsgrader per visning mot
  leave-one-out-skallets IoU), mesh (desimering + uv i bpy, sekunder), texture (numpy: id-kart i uv, z-buffer,
  robust vekting, screened Poisson med DCT, lysdemping, fliser på topp/underside), report, finish (numpy-render av
  ark og mellomvinkel, glb via bpy), assemble, rig, targets/fristad.
- 07:45 Steg 1–3 virker på smedens hode: `config.py`, `sheet.py`, `depth.py` (buffer, seedet fra tools/reconstruct),
  `geom.py` (kamerabasis med yaw/pitch/roll), `fuse.py` (kalibrering mot det visuelle skallet), `align.py`
  (leave-one-out-IoU, skallvoksler, yaw-målestokk trukket fra), `mesh.py` (bpy desimering+uv, 18 s), `texture.py`
  (numpy: id-kart, z-buffer, robust vekting, screened Poisson, lysdemping, normalkart fra nærmeste høye hjørne, 30 s),
  `finish.py` (punktsplatting-render, glb via bpy). Tid smith_head: seg 2, depth 0 (buffer), fuse 75 (align 29), mesh 18,
  tekstur 30, render+glb 9 = ca. 2,2 min. Ark: `shots/sheet2model/smith_head/ark.png`, `ark_mellom.png`: ingen sømmer,
  mellomvinklene holder. Fotometrisk feil 0,05–0,07, IoU 0,88–0,93, 36 teksler/cm i ansiktet.
- 08:20 Koordinatoren: hodet var mykere og flatere enn tools/reconstruct. Årsak: (a) punktsplatting-renderen (nå
  rendres arkene med Cycles som før, `finish.render_bpy`, splattingen er bare `render: "fast"`), (b) Poisson på hele
  bildet trakk mellomfrekvensene mot den brede blandingen, (c) lysdempingen gjorde ansiktet mørkere. Nå: to
  frekvensbånd som før (lavt bredt og robust, detaljene n·c¹² fra beste visning), screened Poisson (λ 0,5) bare på
  det lave båndet over sømmene, lysdemping av som standard (`delight: 0`). Skarphet (Laplace-varians, ansiktsutsnitt
  i renderen forfra): gammel 182, ny 208. `sharpness` i rapporten måles på albedoen i ansiktet (teksler som vender
  fram), terskel `sharpness_min`.
- 09:00 Kroppen virker: `assemble.py` (hodet på etter silhuettene, kroppen kuttet, redskapet ut, hammer modellert
  loddrett fra neven, spøkelse fjernet, hull lukket med vifter, ett uv-kart med hodet ×2,2, én projeksjon rett fra
  begge arkene, redskapets piksler utelatt fra kroppen, normalkart fra begge høye modeller), `rig.py` (leddene,
  skjelettet fra målet, vekter ryddet, klipp fra folk_motion via `targets/fristad.py`, glb slanket), `report.py`,
  `cli.py` med rettelsene. smith_body: rapport OK (IoU 0,91/0,91/0,91, foto 0,074, 1 komponent, 1 hull, 12 276
  trekanter, 10,5 teksler/cm, skarphet 271). Tid kropp: fuse 168 (align 45), mesh 33, assemble 34, render 12, rig 4.
  Gjenstår synlig: en kort trestump i venstre neve (som i den gamle kropp_4), lysere flekk på forkleet der hammeren
  var, hammerskaftet stikker litt opp. Fuse er for tregt (pilgrim_head 217 s): optimaliseres.
- README.md, PROMPTS.md, pyproject.toml, template.py (grensesnitt + MakeHuman-forenkling + enkel fit), tests/,
  docs/rekonstruksjon.md og CLAUDE.md oppdatert.
- 09:50 Fart: `fuse` fra 75/168/217 s til 25–32 s (grid på høyst 14 M voksler med større voksel, projeksjonen bufret
  per visning og i float32, bare overflatevoksler i IoU, ett søk per parameter). Hele smeden (hode + kropp,
  rigget) nå ca. 1,6 + 2,1 min med Cycles-arkene inkludert; uten arkene under 2 min per figur.
- Feil funnet og rettet: innrettingen flyttet profilenes høyde noen piksler på Pilegrimen, og den tynne
  hattebremmen forsvant (skallet er et overmål, så IoU «vant» på det). Høyde og skala står nå som standard
  (`align.shift_v`, `align.scale` = false); `align_skin` retter høyden når rutene er tegnet ulikt. `dome` måles i
  piksler (9) uansett voksel, og er av for hatter. Kroppen: tool-/ghost-punkter ut av z-bufferen, hi-normaler
  lagres i `hi_n.npy`, redskapets piksler males over (inpaint) og får liten vekt, `prune_hand` fjerner
  skaftstumpen i den tomme neven, hull lukkes med vifter, hodet 4 500 + kroppen 9 500 trekanter.
- Gjenstander (eierens utvidelse): `props.py` med prior box/cylinder/barrel passet til silhuetten (IoU på grovt
  rutenett), oppsettene `prop1`/`prop2` med 3/4-visninger og speiling, `tiles: "auto"` velger malt flis under.
  Oppslag `chest.json` og `barrel.json` (furniture_b.jpeg rute 1 og 7). PROMPTS.md har prop2-malen og 3×3-oppskriften.
- 10:40 Regresjonstesten: `python3.13 -m pytest sheet2model/tests` → 5 passed (3 min; smith_head, innkeeper_head,
  pilgrim_head fra bufferen, med rettelsene på, pluss to enhetstester). Tersklene per figur i `EXPECT`.
- Kista: prior `chest` (boks + halvsylinder-lokk, `lid` fra silhuetten), yaw ±30° og pitch ±15° per rute pluss
  svakt perspektiv (`persp` i rammen, skala lineært topp→bunn), projeksjon bare på flater som vender mot kameraet
  (n·c > 0,3), resten speiling og flis (flisen legges nå på alle usette flater på gjenstander). Kista: IoU 0,80,
  skarphet 121, pitch 18°, yaw 30°. Tønna: IoU 0,89 (skarphet 48 < 60: forbildet er mykt malt; terskelen bør være
  per oppslag). Begge i `shots/sheet2model/chest|barrel/ark.png`, `ark_mellom.png`.
- Sluttall (ny pakke, `rapport.json`; gammel = tools/reconstruct målt med samme IoU-funksjon på det gamle feltet):
  smith_head IoU 0,92/0,92/0,87/0,87 (gammel 0,92/0,91/0,86/0,85), foto 0,066, skarphet render 72 (gammel 64), 93 s.
  smith_body IoU 0,90/0,92/0,93 (gammel 0,90/0,90/0,92), foto 0,065, 12 338 tri, 132 s med Cycles-ark.
  pilgrim_head IoU 0,88/0,82/0,91/0,91 (gammel 0,91/0,83/0,88/0,88), foto 0,114 (over 0,11: rettes av hudlinjer
  med rettelsene på), render-skarphet 113 (gammel 107), 104 s. pilgrim_body IoU 0,85/0,92/0,84 (gammel
  0,91/0,90/0,89), foto 0,122 (hodets front–høyre), 2 komponenter i hodet (flak), 118 s.
- Gjenstår: trestumpen/mørk flekk ved smedens hender (inpaintet redskapsområde på forkleet), pilgrim_body hodeflak,
  `revolve`/glb-prior, prop2 på ekte ark når de kommer, malen (template.py) steg 4, alle 13 figurer på nytt.
