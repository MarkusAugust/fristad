// Sjekker at alt nettlaget kaller på musikken (`music.X(...)` i web/main.js)
// faktisk er funksjoner i `createMusic`. En getter med samme navn som en
// funksjon i objektet skjulte `music.song`, og kjøpet av mjød frøs hele
// spillet (eieren 19. oktober). Kjøres av tools/build-web.sh.
import { readFileSync } from "node:fs";
import { createMusic } from "../web/music.js";

const main = readFileSync(new URL("../web/main.js", import.meta.url), "utf8");
const called = [...new Set([...main.matchAll(/\bmusic\.(\w+)\s*\??\.?\(/g)].map((m) => m[1]))];
const music = createMusic(() => null);
const broken = called.filter((name) => typeof music[name] !== "function");
if (broken.length) {
  console.error(`music.js: ${broken.join(", ")} kalles som funksjon i main.js, men er ikke det`);
  process.exit(1);
}
console.log(`music.js: ${called.length} funksjoner kalt fra main.js finnes (${called.join(", ")})`);
