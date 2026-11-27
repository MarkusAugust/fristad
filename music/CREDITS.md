# Musikken

Instrumentprøvene i denne mappa (`<instrument>/<tone>.mp3`) er toner fra lydfonten
**FluidR3_GM** av Frank Wen og bidragsyterne, gjengitt som mp3 av Benjamin Gleitzman i
[gleitz/midi-js-soundfonts](https://github.com/gleitz/midi-js-soundfonts) (mappa `FluidR3_GM/`).
De hentes uendret med `sh tools/music/fetch.sh`, bare noen få toner per instrument; spillet
spiller mellomtonene ved å endre avspillingsfarten (`web/score.js`).

Lisenser:

- FluidR3_GM.sf2: MIT-lisensen, Copyright (c) Frank Wen (slik den er gitt ut i Debian-pakken
  `fluid-soundfont-gm` og i FluidSynth-prosjektet).
- gleitz/midi-js-soundfonts: MIT-lisensen, Copyright (C) 2012 Benjamin Gleitzman. Repoets README
  oppgir i tillegg Creative Commons Attribution 3.0 (https://creativecommons.org/licenses/by/3.0/)
  for de gjengitte FluidR3-filene; denne krediteringen oppfyller begge.

Instrumentene: `recorder` (blokkfløyte), `shanai` (skalmeie), `fiddle` (fele),
`acoustic_guitar_nylon` (lutt), `orchestral_harp` (harpe), `dulcimer` (psalter), `choir_aahs`
(munkekor), `church_organ` (positiv), `cello` og `contrabass` (bordun), `bagpipe` (sekkepipe),
`tubular_bells` (kirkeklokke), `timpani` (pauke) og `taiko_drum` (tromme).

Melodiene er Dies irae (sekvens fra 1200-tallet) og egne melodier i middelalderstil; alt er i
fri rettighet eller laget for Fristad.

MIT-lisensen (gjelder begge):

> Permission is hereby granted, free of charge, to any person obtaining a copy of this software
> and associated documentation files (the "Software"), to deal in the Software without
> restriction, including without limitation the rights to use, copy, modify, merge, publish,
> distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
> Software is furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all copies or
> substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
> BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
> NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
> DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
> OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
