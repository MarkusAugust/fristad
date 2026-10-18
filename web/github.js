// Lagringen fra redigeringen (docs/redigering.md, steg 5): filene motoren
// gir (`engine.save_files()`) havner på grenen `redigering` i fristad-repoet,
// med GitHubs Git Data API fra nettleseren, én commit per økt. Grenen bygges
// ikke i Actions; Claude kjører kontrollene og slår den inn. Neste gang du
// redigerer, fortsetter du fra det som ligger der (`branchFiles`).
//
// Nøkkelen (en finkornet personlig nøkkel med «Contents: Read and write»
// bare på fristad-repoet) huskes bare i denne nettleseren.

const REPO = "MarkusAugust/fristad";
const BRANCH = "redigering";
// Grenen begynner fra forhåndsvisningen første gang.
const BASE = "claude/detaljnivaa";
const KEY = "fristad.github-token";

export function token() {
  try {
    return localStorage.getItem(KEY) || "";
  } catch {
    return "";
  }
}

export function setToken(t) {
  try {
    if (t) localStorage.setItem(KEY, t);
    else localStorage.removeItem(KEY);
  } catch {
    /* ingen lagring i denne nettleseren */
  }
}

async function api(path, init = {}) {
  const res = await fetch(`https://api.github.com/repos/${REPO}${path}`, {
    ...init,
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token()}`,
      "X-GitHub-Api-Version": "2022-11-28",
      ...(init.body ? { "Content-Type": "application/json" } : {}),
    },
  });
  if (res.status === 404 && init.allow404) return null;
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    if (res.status === 401 || res.status === 403) throw new Error(`GitHub nektet (${res.status}). Sjekk at nøkkelen gjelder fristad-repoet med «Contents: Read and write».`);
    throw new Error(`GitHub svarte ${res.status}: ${text.slice(0, 200)}`);
  }
  return res.json();
}

// Lagringene i én økt blir én commit: lagrer du igjen, erstattes commiten
// du lagret sist i denne fanen (samme forelder), så grenen får én commit per
// økt og ikke én per trykk på Lagre (eieren 12. oktober).
const LAST = "fristad.edit-commit";

function last() {
  try {
    return JSON.parse(sessionStorage.getItem(LAST) || "null");
  } catch {
    return null;
  }
}

// Filene på grenen `redigering` (de som finnes der), som `{path, text}`.
// Gir [] når grenen ikke finnes eller du ikke har nøkkel.
export async function branchFiles(paths) {
  if (!token()) return [];
  const ref = await api(`/git/ref/heads/${BRANCH}`, { allow404: true });
  if (!ref) return [];
  const out = [];
  for (const path of paths) {
    const f = await api(`/contents/${path}?ref=${BRANCH}`, { allow404: true });
    if (!f?.content) continue;
    const bytes = Uint8Array.from(atob(f.content.replace(/\n/g, "")), (c) => c.charCodeAt(0));
    out.push({ path, text: new TextDecoder().decode(bytes) });
  }
  return out;
}

// Filene på grenen `redigering` (lages fra BASE om den ikke finnes). Gir
// commitens sha.
export async function commit(files, message) {
  let ref = await api(`/git/ref/heads/${BRANCH}`, { allow404: true });
  if (!ref) {
    const base = await api(`/git/ref/heads/${encodeURIComponent(BASE)}`);
    ref = await api("/git/refs", { method: "POST", body: JSON.stringify({ ref: `refs/heads/${BRANCH}`, sha: base.object.sha }) });
  }
  const head = ref.object.sha;
  const mine = last();
  const replace = mine && mine.sha === head;
  const parentSha = replace ? mine.parent : head;
  const tip = await api(`/git/commits/${head}`);
  const tree = await api("/git/trees", {
    method: "POST",
    body: JSON.stringify({ base_tree: tip.tree.sha, tree: files.map((f) => ({ path: f.path, mode: "100644", type: "blob", content: f.text })) }),
  });
  if (tree.sha === tip.tree.sha) return { sha: head, unchanged: true };
  const made = await api("/git/commits", { method: "POST", body: JSON.stringify({ message, tree: tree.sha, parents: [parentSha] }) });
  await api(`/git/refs/heads/${BRANCH}`, { method: "PATCH", body: JSON.stringify({ sha: made.sha, force: !!replace }) });
  try {
    sessionStorage.setItem(LAST, JSON.stringify({ sha: made.sha, parent: parentSha }));
  } catch {
    /* uten lagring blir hver lagring sin egen commit */
  }
  return { sha: made.sha, unchanged: false, replaced: !!replace };
}

export const SAVE_BRANCH = BRANCH;
