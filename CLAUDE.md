# Fristad – instruksjoner for Claude

Fristad er dokumentasjonen til designsystemet [Fristil](https://github.com/MarkusAugust/fristil) som en 3D-by du går rundt i. Se README.md for oversikt. Svar eieren på norsk.

## Arbeidsflyt

- **Bygg minst mulig på Netlify.** Netlify bygger aldri (`ignore = "exit 0"` i netlify.toml). GitHub Actions (`.github/workflows/web.yml`) bygger Wasm og laster opp `dist/` med Netlify CLI til prosjektet `fristad` (site id står i workflowen, nøkkelen er secret `NETLIFY_AUTH_TOKEN`). Repoet er privat, så også Actions-minutter er begrenset: test lokalt først, push samlet.
- **Test lokalt før push.** `cargo build --bins` og skjermbilder med `snapshot` (fungerer med programvare-GPU, mesa lavapipe):
  `./target/debug/snapshot --out shots/x.png --size 960x540 --eye 7,1.7,9 --yaw 14.5 [--theme dark] [--gate] [--variant secondary]`
  Se på bildene før du pusher.
- **Wasm kan ikke bygges i Claude sitt skymiljø** (static.rust-lang.org er sperret). Wasm-feil vises i Actions: `gh run view <id> --repo MarkusAugust/fristad --log-failed`.
- **Test nettversjonen** i eierens innebygde nettleser på https://fristad.netlify.app. `window.fristad.engine` har `place(x,y,z,yaw,pitch)`, `key(code, down)`, `set_paused`, `set_variant`, og `window.fristad.openPanel/closePanel`.
- Tokens: `bun tools/export-tokens.ts`. Dokumentasjonstekst: `bun tools/export-docs.ts [sti-til-fristil-klone]`.
- Pass på at `[hidden]` faktisk skjuler: elementer med egen `display` trenger regelen øverst i `web/style.css`.

## Beslutninger

- Egen motor i Rust + wgpu (WebGPU, WebGL2 som reserve) + Rapier, for maks ytelse.
- Stil: stilisert realisme, nordisk arkitektur. Realismen kommer fra lys, skygger og vær, ikke fotoskannede teksturer. Prosedyregenerert geometri og shader-materialer, ingen modellfiler så langt.
- 1 rem = 1 meter. Lyst tema = dag, mørkt tema = natt.
- Komponentene skal ikke stå på sokler: hver komponent får en metafor i byen (knapp ved porten, brødsmulespor, brødrister for toast, osv.).
- Teksturer: CC0 (Poly Haven, ambientCG) for grunnmaterialer; eieren kan generere unike med Gemini-prompter.
- Hosting: Netlify for den statiske byen. Railway er aktuelt senere for en flerspillerserver.

## Status (6. oktober 2026)

Ferdig: torget, porten med `.fs-button` (E inspiser, R variant, F åpner porten, T dag/natt), panel med ekte Fristil-komponenter og tekst fra docs, nordlys, skyer, publisering.

Neste:
1. Musefølsomheten var for lav ved første test (økt fra 0.0022 til 0.0035 rad/px, ikke bekreftet av eieren ennå). Vurder en innstilling.
2. Flere komponenter i Gamlebyen (switch som spaker for gatelys, alert som skilt, breadcrumbs som spor, progress som bro, skeleton som stillas, spinner som vindmølle, toast som brødrister, dialog som telefonkiosk, tabs som hus med dører).
3. Bydelene Verkstedet (ramme), Tårnene (frittstående), Tokenparken.
4. Vær (regn, snø, tåke) og `connection-status` som styrer været.
5. Lyd, kart, touch-kontroller, tilgjengelig 2D-modus.
