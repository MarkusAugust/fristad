# Sammenslåingen – arbeidslogg

Oppgave: slå `origin/rydd/gamle` og `origin/wip/krypten` inn i `merge/alt-nytt` (fra `claude/detaljnivaa` 877766b) og få alt grønt. Arbeidskopi: `/home/user/fristad-merge`, target `/home/user/fristad/target-merge`.

## Steg 1: `rydd/gamle` (f62e53a)
- Konflikter: ti `assets/models/npc/npc_*.glb` (slettet i ryddingen, nye i HEAD) → HEAD beholdt; `gltf.rs` (npc-oppføringene mot ryddingens tomme liste) → alle 14 npc-oppføringer beholdt, `npc_fishwife` lagt til, portalmodellene ut (filene er slettet).
- `npc.glb`, `npc_woman.glb` og atlasene var gamle og er borte (ryddingen); `town.rs` faller tilbake på `NPC_MODEL`, som bare varsles som manglende.
- `geometry.rs`, `town.rs`, `tests.rs`, `world.wgsl`, CLAUDE.md gikk automatisk. Lagene er 0–45 (46 lag), alle med mapper i `assets/textures/`.
- Disk: 1,7 GB ledig ved start; slettet mitt gamle `scratchpad/tgt` og tre snapshot-kopier (→ 3,5 GB).

## Steg 2: `wip/krypten` (plan)
- Konflikter ventet: tre npc-glb (HEAD), `docs/wip/figurene.md` (HEAD, lengst), `gltf.rs` (kryptens, uten aliaset `new_layer_name`), `geometry.rs` (ryddingens 0–45 + kryptlagene 46–54, LAYERS 55), `dungeon.rs` (slett), `world/mod.rs`, `tests.rs` (kryptens, med ryddingens `skip`), `fetch-assets.ts` (begge).
- Avlede de ni kryptlagene med `derive.py`; tex_scale 16 × 4 er alt i kryptens `wgpu/mod.rs` og `common.wgsl`.
