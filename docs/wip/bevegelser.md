# Bevegelsene til menneskene (logg)

## Gjort
- `tools/models/folk_motion.py`: biblioteket. Klippene `walk`, `idle`, `talk`, `wave` (som før) og nye `start`, `stop`, `stop_r`, `turn_l`, `turn_r`, `look`, `fidget`. Stiler i `STYLES` (default, smith, innkeeper, gorvek, old, boy, guard) og figurene i `FIGURES`. Grep: `hang`, `heavy`, `long`, `front`.
  - Føttene settes ned i verden (`Plant`), og kroppen beveger seg over dem (`Track`), så foten som står, glir med akkurat spillets fart. Hælnedslag → flat → hælløft om tåspissen. Bekkenhøyden kommer fra beinlengden (omvendt pendel) med kneet bøyd etter gangkurvene.
  - Målt mot gangkurvene (smeden): kneet 4° ved hælnedslaget, 18° i nedtrykket, 9° midt i støtten, 34° ved avsparket og 53° i svevet; hofta +23° til −9°; bekkenet går 3,4 cm opp og ned.
  - Redskapet holdes klar av lårene, leggene og hoftene (`keep_clear`, kapsler) og over bakken.
- `tools/models/motion_sheet.py`: kontaktark og GIF (Workbench). Figuren flyttes som i spillet over et rutemønster.
- `src/domain/anim.rs`: `Rig::stride_speed` (farten leses av klippet) og en test.
- `src/domain/npcs.rs`: tilstandene stå / start / gå / stopp / snu (mot deg eller mot målet), gester i hvile, praten etter stoppet. Faller tilbake til den gamle oppførselen når klippene mangler. Tre nye tester.
- `tools/models/people.py`: bruker `folk_motion.clips` (tabellen `MOTION`).

## Virker
- Testene i anim og npcs og `innbyggerne_gar_og_stopper` går (kjørt i en egen kopi av HEAD, fordi treet ikke kompilerte testene mens krypten var under arbeid).
- Kvalitetssjekken (scratch qa.py) på alle elleve gamle figurene: ingen sko gjennom bakken, ingen glidning, ingen skjøter i løkkene, og den målte farten stemmer.

## Gjenstår
- Vinkingen er rettet: en hilsen forover, målene i brystets rom, høyst 14° per bilde.
- Kontaktark og GIF-er ligger i shots/bevegelser/ (git-ignorert).
- Skjørtene i people.py er stive på `hips`, så det bakre beinet stikker ut. Det må fikses i modellen (vekter mellom hoftene og lårene).
- Den gamle smeden har deler av forkleet vektet på armen (flak når armen løftes). Det er modellen, ikke klippet.
- Prøve på de nye figurene fra folk.py når de finnes.
