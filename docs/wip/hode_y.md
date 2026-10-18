# Hode Y: Buttonsmeden som «digital leire»

Start 10. okt 16:32 UTC. Kode: `tools/models/hode_y.py`. Ark: `shots/hode_y/`.

## Fremgangsmåte
- Hodet er et signert avstandsfelt (SDF) i numpy på et voxelgitter (1,6 mm): ellipsoider, runde kjegler og kapsler,
  smeltet med smooth-min (som DynaMesh, men med myke overganger), hulninger med smooth-subtraksjon.
- Hår, skjegg, bart og bryn: lokker som runde kjegler langs kurver som «kjemmes» over hodeformen
  (flytfelt projisert på flaten), tykke ved roten og spisse i tuppen, med liten blanding så det blir furer mellom.
- Marching cubes → Blender → desimering → Smart UV (ansiktet mer plass) → teksturen males per teksel
  ved å projisere tilbake til feltet (farge, SDF-AO, krumning, normalkart).

## Logg

### Runde 1 (17:00) – `shots/hode_y/runde_1.png`
- Rørledningen virker hele veien: SDF-skulptur (hud ~16 s, 101 lokker ~25 s) → marching cubes → Blender
  (hud 3 800 + hår 4 600 trekanter) → Smart UV med ansiktet 1,8× → maling per teksel (farge, SDF-AO, krumning,
  tangentrom-normalkart) → glb (`shots/hode_y/smith_head_y.glb`) → Cycles-ark. Hele løpet ~3 min.
- Feil underveis: helt svart render fordi flatene fra marching cubes var snudd (normalkartet pekte innover). Rettet.
- Bedre enn forsøkene før: ingen projeksjon og ingen sømmer; lokkene er ekte former med furer mellom.
- Fortsatt galt: for mange tynne «pølser» i skjegg og hår (forbildet har færre, større klumper),
  for mange grå striper (ser stripete ut), ansiktet er blekt og flatt, øynene er smale sprekker,
  nesa for liten, hårtoppen for høy og smal, skjegget står ut som blokker på sidene.

### Runde 2–3 (17:15–18:12) – `shots/hode_y/runde_3.png` (beste)
- Rettet: øynene har blå iris og pupill (uv-feil: `transform_apply` brukte også posisjonen, og den primitive kula
  hadde alt et uv-lag); ansiktet er malt (øyehuler, rynker, kråkeføtter, nasolabialfolder, rød nese og kinn,
  sot, mørk vippelinje); potetnese, tyngre og lavere brynbue, fyldigere kinn; færre og bredere lokker,
  grått bare på tinningene; 13 store skjeggklumper og en tung bart.
- Lang feiljakt på en kantete flekk på nesa: det var en fold i én uv-øy fra Smart UV med vinkelgrense 60°.
  Med 40° er den borte. (Ikke tekstur, ikke projeksjon, ikke tangenter.)
- Fortsatt galt mot forbildet: håret er en hjelm uten den fyldige bølgen bakover, bryn og skjegglokker ser
  ut som pølser, skjegget er for bredt på sidene, og ansiktet er mer karikatur enn forbildet.
- Avsluttet 18:12 fordi koordinatoren valgte en annen fremgangsmåte (dybde fra forbildet).

## Det som kan gjenbrukes
- **Øynene** (`eyes_blender`, `eye_texture`): uv-kule med polen fram, plan uv forfra i lokale koordinater,
  iris fra `eyes.jpeg` (rad 1 kol 2) skalert så irisen er halve eplet, sclera utenfor. Gropa må bryte gjennom
  hudflaten (eplet ca. 0,3 cm foran huden) og lokkene er skall rundt eplet kuttet med parabler (mandelform).
- **Mal per teksel med bur-projeksjon** (`raster`, løkka i `main_full`): rasteriser lavpoly-uv,
  gå langs lavpoly-normalen til høypolyflaten (gradienten som reserve), og les farge, AO, krumning og
  normal der. Bruk Blenders MikkTSpace-tangenter (`calc_tangents`, interpolert per teksel) til
  tangentrom-normalkartet. Fyll tomme teksler med nærmeste (`distance_transform_edt`).
- **SDF-AO** (`sdf_ao`) og krumning fra Laplace av glattet felt gir malt WoW-lys billig.
- **Marching cubes → Blender** (`march`, `to_blender`): skimage gir riktig rekkefølge på hjørnene (ikke snu).
  Corrective Smooth, Decimate (collapse) og så rydding (remove_doubles, dissolve_degenerate, normals_make_consistent).
- **Smart UV med 40° vinkelgrense**, ikke 60°: høyere grense gir folder på buede former (nesetippen).
- Feller: hudflisen `skin_ruddy` har kantete flekker og arr. Bruk flat farge eller sterkt uskarpe fliser.
  Decimate med vertex group virket motsatt (knuste ansiktet).
