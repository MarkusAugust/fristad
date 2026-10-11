# Forbildene til monstrene

Fra eieren etter `docs/prompter-monstre.md`. `<navn>_body` er kroppen forfra, fra siden og bakfra (ikke kvadratisk, rundt 2750 × 1540); `<navn>_head` er hodet i 2 × 2 for elite og boss. Bygges med `sheet2model` (`body3`/`head2x2`) og monstermålet (oppgave #24).

| Fil | Monster | Nivå | Tema |
|---|---|---|---|
| `button_wight_body` | Button Wight | trash | gravkjelleren |
| `tooltip_sprite_body` | Tooltip Sprite | trash | gravkjelleren |
| `datepicker_priest_body/_head` | Datepicker Priest | elite | gravkjelleren |
| `modal_body/_head` | Modalen | boss | gravkjelleren |

Avvik: Button Wight har teksten «SUBMIT» på brystet og Datepicker Priest «DECEMBER» i panna (tekst i teksturen er greit på disse). Button Wight-profilen har armen delvis foran kroppen.
| `input_leech_body` | Input Leech | trash | kloakken |
| `toast_frog_body` | Toast Frog | trash | kloakken |
| `select_broodmother_body/_head` | Select Broodmother | elite | kloakken |
| `table_body/_head` | Tabellen | boss | kloakken |

Avvik: Tabellens kroppsark viser den i to lag (oppe og nede) – bruk øverste rad; profilen er krummet som en S, så formen leses best fra forfra/bakfra pluss segmentene. Input Leech ligger lavt (kroppen er bredere enn høy), så `scale` i oppslaget må settes etter lengden.
