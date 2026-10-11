# SMAA

Oppslagstabellene til kantfilteret SMAA (Jorge Jimenez m.fl., <https://github.com/iryoku/smaa>, MIT-lisens i `LICENSE.txt`), hentet fra `Textures/AreaTex.h` og `Textures/SearchTex.h` og lagret som rå bytes:

- `area.bin`: 160 × 560, RG8 (arealene).
- `search.bin`: 64 × 16, R8 (søkelengdene).

Skyggeren er portert til WGSL i `src/adapters/wgpu/shaders/smaa.wgsl` (forhåndsinnstillingen HIGH uten diagonaler).
