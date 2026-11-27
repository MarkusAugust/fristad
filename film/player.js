// Åpningsfilmen: den ferdige videoen (`dist/film/intro.mp4`, klippet av
// tools/film/cut.py etter tools/film/timeline.py) spilles over spillet i
// 16:9 med svarte felt. Grensesnittet er det samme som den gamle malte
// filmen hadde: `tick(dt)` gir hvor mye av skjermen filmen dekker (1 mens
// den går, ned mot 0 når den toner bort til slutt, så main.js kan sette
// spilleren ved sørporten bak den), og `done` løses med `{ skipped }`.
// Esc, mellomrom eller knappen «Hopp over» hopper over. `gesture` sier om det
// var et klikk eller mellomrom (en brukerhandling), så musa kan fanges med en gang.
// Et klikk på linja nederst hopper til det stedet i filmen, og den kan dras.

const FADE = 1.6; // sekunder fra videoens svarte slutt til spillet er framme
const SEEK_END = 0.5; // sekunder før slutten et hopp høyst går til, så et klikk helt til høyre ikke avslutter filmen

export function createFilm({ src = "film/intro.mp4" } = {}) {
  const root = document.createElement("div");
  root.className = "film film-video";
  root.innerHTML = `
    <video class="film-player" playsinline preload="auto"></video>
    <p class="film-rotate">Snu telefonen for større bilde</p>
    <div class="film-loading">Laster filmen …</div>
    <button type="button" class="lt-button film-skip">Hopp over</button>
    <div class="film-progress"><div class="film-progress-fill"></div></div>`;
  document.body.appendChild(root);
  document.body.classList.add("filming");
  root.style.pointerEvents = "auto";

  const video = root.querySelector("video");
  const loading = root.querySelector(".film-loading");
  const bar = root.querySelector(".film-progress");
  const fill = root.querySelector(".film-progress-fill");
  const skipBtn = root.querySelector(".film-skip");

  let cover = 1;
  let ending = false;
  let skipped = false;
  let gesture = false;
  let finished = false;
  let resolveDone;
  const done = new Promise((r) => (resolveDone = r));

  function finish(wasSkipped, byGesture = false) {
    if (finished) return;
    finished = true;
    skipped = wasSkipped;
    gesture = byGesture;
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
    resolveDone({ skipped, gesture });
  }
  function onKey(e) {
    if (e.key === "Escape" || e.key === " ") {
      e.preventDefault();
      e.stopPropagation();
      finish(true, e.key === " ");
    }
  }
  document.addEventListener("keydown", onKey, true);
  skipBtn.addEventListener("click", () => finish(true, true));

  video.src = src;
  video.addEventListener("canplay", () => (loading.hidden = true), { once: true });
  video.addEventListener("ended", () => finish(false));
  video.addEventListener("error", () => {
    console.warn("Filmen kunne ikke spilles:", video.error);
    finish(true);
  });
  video.addEventListener("timeupdate", () => {
    if (video.duration && !seeking) fill.style.width = `${(100 * video.currentTime) / video.duration}%`;
  });
  // Linja: klikk eller dra for å hoppe i filmen.
  let seeking = false;
  function seek(e) {
    if (!Number.isFinite(video.duration) || finished) return;
    const box = bar.getBoundingClientRect();
    const part = Math.min(1, Math.max(0, (e.clientX - box.left) / box.width));
    const time = Math.min(part * video.duration, Math.max(0, video.duration - SEEK_END));
    video.currentTime = time;
    fill.style.width = `${(100 * time) / video.duration}%`;
  }
  bar.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    seeking = true;
    bar.classList.add("seeking");
    bar.setPointerCapture(e.pointerId);
    seek(e);
  });
  bar.addEventListener("pointermove", (e) => {
    if (seeking) seek(e);
  });
  const release = () => {
    seeking = false;
    bar.classList.remove("seeking");
  };
  bar.addEventListener("pointerup", release);
  bar.addEventListener("pointercancel", release);
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
