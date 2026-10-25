// Magneten i nivåredigeringen (`snapPlacement` i web/nivaa-plan.js) uten
// nettleser: en gang snapper til en ledig åpning og dreier seg selv, holder
// dreiningen eieren valgte, og står fritt langt unna eller mot en mur.
//   node tools/nivaa-plan-test.mjs
import { snapPlacement, freeOpenings } from "../web/nivaa-plan.js";

let failed = 0;
const check = (ok, what) => {
  console.log(`${ok ? "OK  " : "FEIL"} ${what}`);
  if (!ok) failed++;
};
// En gang på to ruter nord–sør, med åpning i hver ende.
const gang = { cells: [[0, 0, 0], [0, 0, 1]], openings: [{ id: "n", at: [0, 0], y: 0, side: 0 }, { id: "s", at: [0, 1], y: 0, side: 2 }] };
// Et rom på én rute i (0, 0) med en ledig åpning mot øst.
const room = (chosen = null) => ({ edges: [], modules: [{ id: "a", cells: [[0, 0, 0]], openings: [{ id: "o", at: [0, 0], y: 0, side: 1, chosen }] }] });
const taken = new Set(["0,0,0"]);

const s = snapPlacement(room(), gang, [2, 0, 0], [0, 1, 2, 3], false, taken);
check(s && s.rot === 1 && s.at.join() === "2,0,0", `gangen dreies øst–vest og står inntil åpningen (${JSON.stringify(s)})`);
check(s?.links.length === 1 && s.links[0].join() === "2,0,0", "koblingen er midt på kanten mellom dem");
check(snapPlacement(room(), gang, [2, 0, 0], [0], false, taken) === null, "dreid som eieren valgte passer den ikke, og står fritt");
check(snapPlacement(room(), gang, [8, 0, 0], [0, 1, 2, 3], false, taken) === null, "langt unna snapper den ikke");
check(snapPlacement(room("vegg"), gang, [2, 0, 0], [0, 1, 2, 3], false, taken) === null, "en åpning gjort til mur er ikke ledig");
check(freeOpenings(room()).length === 1 && freeOpenings(room(), ["a"]).length === 0, "ledige åpninger, utenom modulen som flyttes");
const joined = { ...room(), edges: [{ at: [0, 0, 0], side: 1, kind: "door" }] };
check(freeOpenings(joined).length === 0, "en åpning som møter noe er ikke ledig");
if (failed) {
  console.log(`${failed} feil`);
  process.exit(1);
}
