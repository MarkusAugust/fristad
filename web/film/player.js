// Åpningsfilmen: den ferdige videoen (`dist/film/intro.mp4`, klippet av
// tools/film/cut.py etter tools/film/timeline.py) spilles over spillet i
// 16:9 med svarte felt. Grensesnittet er det samme som den gamle malte
// filmen hadde: `tick(dt)` gir hvor mye av skjermen filmen dekker (1 mens
// den går, ned mot 0 når den toner bort til slutt, så main.js kan sette
// spilleren ved sørporten bak den), og `done` løses med `{ skipped }`.
// Esc, mellomrom eller knappen «Hopp over» hopper over.

const FADE = 1.6; // sekunder fra videoens svarte slutt til spillet er framme

export function createFilm({ src = "film/intro.mp4" } = {}) {
  const root = document.createElement("div");
  root.className = "film film-video";
  root.innerHTML = `
    <video class="film-player" playsinline preload="auto"></video>
    <p class="film-rotate">Snu telefonen for større bilde</p>
    <div class="film-loading">Laster filmen …</div>
    <button type="button" class="fs-button film-skip">Hopp over</button>
    <div class="film-progress"><div class="film-progress-fill"></div></div>`;
  document.body.appendChild(root);
  document.body.classList.add("filming");
  root.style.pointerEvents = "auto";

  const video = root.querySelector("video");
  const loading = root.querySelector(".film-loading");
  const fill = root.querySelector(".film-progress-fill");
  const skipBtn = root.querySelector(".film-skip");

  let cover = 1;
  let ending = false;
  let skipped = false;
  let finished = false;
  let resolveDone;
  const done = new Promise((r) => (resolveDone = r));

  function finish(wasSkipped) {
    if (finished) return;
    finished = true;
    skipped = wasSkipped;
    if (wasSkipped) {
      cover = 0;
      cleanup();
    } else {
      ending = true; // tick toner bort over FADE
    }
  }
  function cleanup() {
    document.removeEventListener("keydown", onKey, true);
    try {
      video.pause();
      video.removeAttribute("src");
      video.load();
    } catch {}
    root.remove();
    document.body.classList.remove("filming");
    resolveDone({ skipped });
  }
  function onKey(e) {
    if (e.key === "Escape" || e.key === " ") {
      e.preventDefault();
      e.stopPropagation();
      finish(true);
    }
  }
  document.addEventListener("keydown", onKey, true);
  skipBtn.addEventListener("click", () => finish(true));

  video.src = src;
  video.addEventListener("canplay", () => (loading.hidden = true), { once: true });
  video.addEventListener("ended", () => finish(false));
  video.addEventListener("error", () => {
    console.warn("Filmen kunne ikke spilles:", video.error);
    finish(true);
  });
  video.addEventListener("timeupdate", () => {
    if (video.duration) fill.style.width = `${(100 * video.currentTime) / video.duration}%`;
  });
  const play = video.play();
  if (play?.catch) {
    play.catch((e) => {
      console.warn("Filmen fikk ikke starte:", e);
      finish(true);
    });
  }

  return {
    done,
    tick(dt) {
      if (ending && cover > 0) {
        cover = Math.max(0, cover - dt / FADE);
        root.style.opacity = String(cover);
        if (cover === 0) cleanup();
      }
      return cover;
    },
  };
}
