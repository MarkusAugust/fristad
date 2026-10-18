// Baker materialene til teksturer ved oppstart: albedo og normal/ruhet/høyde,
// ett lag per materiale, sømløst flisbare. Normalen regnes fra høyden her,
// så den er konsistent og får ordentlige mipmaps.
//
// Lagene må stemme med `Layer` i mesh.rs:
// 0 brostein, 1 puss, 2 panel, 3 skifer, 4 takstein, 5 stein, 6 mur,
// 7 eik, 8 metall, 9 gress, 10 bark, 11 grus, 12 betong.

struct Bake {
    // x lag, y flisstørrelse i meter, z 1/oppløsning, w mip-nivå.
    params: vec4<f32>,
}
@group(0) @binding(0) var<uniform> bake: Bake;
@group(0) @binding(1) var src_a: texture_2d<f32>;
@group(0) @binding(2) var src_b: texture_2d<f32>;
@group(0) @binding(3) var src_sampler: sampler;
// Array-teksturene leses som arrays med laget i bake.params.x: WebGL2 kan
// ikke binde ett lag av en array som en 2D-tekstur.
@group(0) @binding(4) var arr_a: texture_2d_array<f32>;
@group(0) @binding(5) var arr_b: texture_2d_array<f32>;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> VsOut {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
    var o: VsOut;
    o.clip = vec4<f32>(p, 0.0, 1.0);
    o.uv = vec2<f32>(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);
    return o;
}

// Periodisk støy, så flisene blir sømløse: gitteret brettes med perioden.
fn phash(p: vec2<f32>, period: f32) -> f32 {
    let q = p - period * floor(p / period);
    return hash21(q + 0.5);
}

fn pnoise(p: vec2<f32>, period: f32) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = phash(i, period);
    let b = phash(i + vec2<f32>(1.0, 0.0), period);
    let c = phash(i + vec2<f32>(0.0, 1.0), period);
    let d = phash(i + vec2<f32>(1.0, 1.0), period);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn pfbm(p: vec2<f32>, period: f32, octaves: i32) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var q = p;
    var per = period;
    for (var i = 0; i < octaves; i++) {
        v += a * pnoise(q, per);
        q = q * 2.0;
        per = per * 2.0;
        a *= 0.5;
    }
    return v;
}

// Periodisk Voronoi: avstand til nærmeste og nest nærmeste senter, og
// nummeret på cella. `p` i celle-enheter, `period` i celler.
struct Vor {
    d1: f32,
    d2: f32,
    id: f32,
    // Retning fra senteret, til å bøye steinene.
    off: vec2<f32>,
}

fn voronoi(p: vec2<f32>, period: f32, jitter: f32) -> Vor {
    let i = floor(p);
    let f = fract(p);
    var r: Vor;
    r.d1 = 9.0;
    r.d2 = 9.0;
    r.id = 0.0;
    r.off = vec2<f32>(0.0);
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let c = vec2<f32>(f32(x), f32(y));
            let cell = i + c;
            let h = phash(cell, period);
            let h2 = phash(cell + 17.3, period);
            let center = c + vec2<f32>(0.5) + (vec2<f32>(h, h2) - 0.5) * jitter;
            let d = f - center;
            let dist = dot(d, d);
            if (dist < r.d1) {
                r.d2 = r.d1;
                r.d1 = dist;
                r.id = h;
                r.off = d;
            } else if (dist < r.d2) {
                r.d2 = dist;
            }
        }
    }
    r.d1 = sqrt(r.d1);
    r.d2 = sqrt(r.d2);
    return r;
}

struct Sample {
    albedo: vec3<f32>,
    height: f32,
    rough: f32,
}

// Murforband: rader forskjøvet, med fuge. Returnerer avstand til fugen
// (i enheter av steinhøyden), steinnummer og posisjon i steinen.
struct Bond {
    edge: f32,
    id: f32,
    local: vec2<f32>,
}

fn bond(uv: vec2<f32>, cols: f32, rows: f32, joint: f32, stagger: f32, wobble: f32) -> Bond {
    var q = vec2<f32>(uv.x * cols, uv.y * rows);
    let row = floor(q.y);
    q.x += row * stagger + (phash(vec2<f32>(row, 3.0), rows) - 0.5) * wobble;
    let cell = floor(q);
    let f = fract(q) - 0.5;
    let ex = (0.5 - abs(f.x)) / cols;
    let ey = (0.5 - abs(f.y)) / rows;
    var b: Bond;
    b.edge = min(ex, ey) - joint * 0.5;
    b.id = phash(cell, cols * rows);
    b.local = f;
    return b;
}

fn mat_cobble(uv: vec2<f32>) -> Sample {
    let cells = 11.0;
    let v = voronoi(uv * cells, cells, 0.85);
    let edge = (v.d2 - v.d1);
    let stone = smoothstep(0.0, 0.22, edge);
    // Avrundet topp.
    let dome = sqrt(max(0.0, 1.0 - min(v.d1 * 1.6, 1.0) * min(v.d1 * 1.6, 1.0)));
    let grain = pfbm(uv * 60.0, 60.0, 3);
    let h = stone * (0.55 + dome * 0.45 + (grain - 0.5) * 0.12) + (1.0 - stone) * (0.1 + grain * 0.15);
    let tone = 0.55 + v.id * 0.45;
    let warm = phash(vec2<f32>(v.id * 97.0, 1.0), 1000.0);
    var col = mix(vec3<f32>(0.33, 0.32, 0.31), vec3<f32>(0.45, 0.40, 0.35), warm) * tone;
    col = mix(col, vec3<f32>(0.36, 0.40, 0.46), smoothstep(0.7, 0.9, phash(vec2<f32>(v.id * 31.0, 2.0), 1000.0)) * 0.5);
    col *= 0.85 + grain * 0.3;
    let sand = vec3<f32>(0.30, 0.27, 0.22) * (0.8 + grain * 0.4);
    let moss = vec3<f32>(0.25, 0.32, 0.14);
    let mossy = smoothstep(0.55, 0.8, pfbm(uv * 3.0, 3.0, 3)) * (1.0 - stone);
    var s: Sample;
    s.albedo = mix(mix(sand, moss, mossy), col, stone);
    s.height = h;
    s.rough = mix(0.95, 0.55 + grain * 0.2, stone);
    return s;
}

fn mat_plaster(uv: vec2<f32>) -> Sample {
    let fine = pfbm(uv * 48.0, 48.0, 4);
    let coarse = pfbm(uv * 6.0, 6.0, 3);
    // Sprekker: tynne, mørke linjer der en ridged-støy er nær toppen.
    let r = abs(pnoise(uv * 9.0 + 3.0, 9.0) - 0.5) * 2.0;
    let crack = smoothstep(0.97, 1.0, 1.0 - r) * smoothstep(0.55, 0.7, pfbm(uv * 2.0, 2.0, 2));
    var s: Sample;
    s.albedo = vec3<f32>(0.92, 0.90, 0.86) * (0.92 + fine * 0.1 + (coarse - 0.5) * 0.08) * (1.0 - crack * 0.5);
    s.height = 0.5 + (fine - 0.5) * 0.25 + (coarse - 0.5) * 0.3 - crack * 0.6;
    s.rough = 0.82 + fine * 0.12;
    return s;
}

fn mat_cladding(uv: vec2<f32>) -> Sample {
    // Liggende panel: 13 bord per flis, med fals og litt vridning.
    let boards = 13.0;
    let y = uv.y * boards;
    let row = floor(y);
    let f = fract(y);
    let id = phash(vec2<f32>(row, 5.0), boards);
    // Hvert bord ligger litt skrått (vestlandspanel): høyden stiger over bordet.
    let bevel = smoothstep(0.0, 0.06, f) * smoothstep(1.0, 0.92, f);
    let h = 0.35 + f * 0.35 * bevel + (1.0 - bevel) * 0.1;
    let grain = pnoise(vec2<f32>(uv.x * 40.0, uv.y * 400.0 + id * 50.0), 40.0);
    let knots = smoothstep(0.75, 0.95, pnoise(vec2<f32>(uv.x * 6.0 + id * 9.0, row * 0.1), 6.0));
    let wear = pfbm(uv * 5.0, 5.0, 3);
    var s: Sample;
    s.albedo = vec3<f32>(0.95, 0.94, 0.91) * (0.9 + grain * 0.1 + (id - 0.5) * 0.06) * (1.0 - knots * 0.15) * (0.94 + wear * 0.08);
    s.height = h + (grain - 0.5) * 0.04 - knots * 0.05;
    s.rough = 0.58 + grain * 0.15 + knots * 0.2;
    return s;
}

fn mat_slate(uv: vec2<f32>) -> Sample {
    // Skifer: rader av heller som overlapper, med skjev underkant.
    let rows = 7.0;
    let cols = 8.0;
    var q = vec2<f32>(uv.x * cols, uv.y * rows);
    let row = floor(q.y);
    q.x += row * 0.5;
    let cell = floor(q);
    let f = fract(q);
    let id = phash(cell, cols * rows);
    let wob = (phash(cell + 7.0, cols * rows) - 0.5) * 0.12;
    // Underkanten av hver helle går litt skrått.
    let bottom = 0.06 + wob + f.x * (phash(cell + 11.0, cols * rows) - 0.5) * 0.1;
    let inside = step(bottom, f.y);
    let side = smoothstep(0.0, 0.03, f.x) * smoothstep(1.0, 0.97, f.x);
    let grain = pfbm(uv * 30.0 + id, 30.0, 3);
    let h = inside * side * (0.6 + f.y * 0.25 + (grain - 0.5) * 0.15) + (1.0 - inside * side) * (0.3 + (grain - 0.5) * 0.1);
    var s: Sample;
    let base = mix(vec3<f32>(0.16, 0.18, 0.22), vec3<f32>(0.24, 0.24, 0.25), id);
    s.albedo = base * (0.8 + grain * 0.4) * mix(0.6, 1.0, inside * side);
    s.height = h;
    s.rough = 0.65 + grain * 0.2;
    return s;
}

fn mat_tiles(uv: vec2<f32>) -> Sample {
    // Takstein i tegl: buede rader, overlappende.
    let rows = 5.0;
    let cols = 8.0;
    var q = vec2<f32>(uv.x * cols, uv.y * rows);
    let row = floor(q.y);
    q.x += (row % 2.0) * 0.5;
    let cell = floor(q);
    let f = fract(q);
    let id = phash(cell, cols * rows);
    let curve = cos((f.x - 0.5) * PI);
    let lip = smoothstep(0.0, 0.12, f.y);
    let grain = pfbm(uv * 40.0, 40.0, 3);
    let h = (0.3 + curve * 0.45) * lip + (1.0 - lip) * 0.2 + f.y * 0.1 + (grain - 0.5) * 0.05;
    let moss = smoothstep(0.6, 0.85, pfbm(uv * 4.0 + 2.0, 4.0, 3)) * (1.0 - curve * 0.6);
    var s: Sample;
    let clay = mix(vec3<f32>(0.62, 0.30, 0.20), vec3<f32>(0.50, 0.24, 0.18), id);
    s.albedo = mix(clay * (0.85 + grain * 0.3), vec3<f32>(0.30, 0.36, 0.18), moss * 0.7);
    s.height = h;
    s.rough = 0.78 + grain * 0.15 - moss * 0.1;
    return s;
}

fn mat_stone(uv: vec2<f32>) -> Sample {
    // Kvaderstein med fuger på 2 cm.
    let b = bond(uv, 3.0, 6.0, 0.012, 0.5, 0.4);
    let face = smoothstep(0.0, 0.012, b.edge);
    let grain = pfbm(uv * 36.0 + b.id * 3.0, 36.0, 4);
    let chip = smoothstep(0.6, 0.9, pfbm(uv * 12.0 + b.id, 12.0, 2));
    let h = face * (0.55 + (grain - 0.5) * 0.3 - chip * 0.15 + b.id * 0.1) + (1.0 - face) * 0.15;
    var s: Sample;
    let tone = mix(vec3<f32>(0.55, 0.53, 0.49), vec3<f32>(0.66, 0.63, 0.57), b.id);
    s.albedo = mix(vec3<f32>(0.35, 0.33, 0.30), tone * (0.8 + grain * 0.35), face);
    s.height = h;
    s.rough = mix(0.95, 0.75 + grain * 0.2, face);
    return s;
}

fn mat_brick(uv: vec2<f32>) -> Sample {
    let b = bond(uv, 6.5, 20.0, 0.012, 0.5, 0.0);
    let face = smoothstep(0.0, 0.01, b.edge);
    let grain = pfbm(uv * 50.0 + b.id * 7.0, 50.0, 3);
    let h = face * (0.6 + (grain - 0.5) * 0.25 + (b.id - 0.5) * 0.1) + (1.0 - face) * 0.2;
    var s: Sample;
    let red = mix(vec3<f32>(0.55, 0.26, 0.20), vec3<f32>(0.42, 0.22, 0.18), b.id);
    let dark = smoothstep(0.8, 1.0, phash(vec2<f32>(b.id * 53.0, 1.0), 2000.0));
    s.albedo = mix(vec3<f32>(0.62, 0.58, 0.52) * (0.8 + grain * 0.3), mix(red, vec3<f32>(0.25, 0.18, 0.16), dark) * (0.85 + grain * 0.3), face);
    s.height = h;
    s.rough = mix(0.95, 0.8, face);
    return s;
}

fn mat_oak(uv: vec2<f32>) -> Sample {
    // Bord på 12 cm, stående langs y.
    let boards = 12.0;
    let x = uv.x * boards;
    let col = floor(x);
    let f = fract(x);
    let id = phash(vec2<f32>(col, 9.0), boards);
    let gap = smoothstep(0.0, 0.04, f) * smoothstep(1.0, 0.96, f);
    let grain = pnoise(vec2<f32>(uv.x * 300.0 + id * 40.0, uv.y * 8.0), 300.0);
    let rings = abs(fract(uv.y * 3.0 + pnoise(vec2<f32>(uv.x * 20.0, uv.y * 2.0 + id), 20.0) * 1.5 + id * 5.0) - 0.5) * 2.0;
    let knot = smoothstep(0.85, 0.97, pnoise(vec2<f32>(uv.x * 10.0 + id * 3.0, uv.y * 4.0), 10.0));
    var s: Sample;
    let base = mix(vec3<f32>(0.52, 0.36, 0.22), vec3<f32>(0.40, 0.27, 0.16), id);
    s.albedo = base * (0.8 + grain * 0.25 + rings * 0.12) * (1.0 - knot * 0.35) * mix(0.5, 1.0, gap);
    s.height = gap * (0.55 + grain * 0.1 - rings * 0.05 - knot * 0.1) + (1.0 - gap) * 0.2;
    s.rough = 0.55 + grain * 0.2 + knot * 0.2;
    return s;
}

fn mat_timber(uv: vec2<f32>) -> Sample {
    // Høvlet gran: lys, gulhvit ved med tydelige årringer og små kvister,
    // bord på 10 cm langs y.
    let boards = 12.0;
    let x = uv.x * boards;
    let col = floor(x);
    let f = fract(x);
    let id = phash(vec2<f32>(col, 3.0), boards);
    let gap = smoothstep(0.0, 0.03, f) * smoothstep(1.0, 0.97, f);
    let grain = pnoise(vec2<f32>(uv.x * 360.0 + id * 40.0, uv.y * 6.0), 360.0);
    let rings = pow(abs(fract(uv.y * 4.0 + pnoise(vec2<f32>(uv.x * 24.0, uv.y * 2.0 + id), 24.0) * 1.2 + id * 5.0) - 0.5) * 2.0, 3.0);
    let knot = smoothstep(0.88, 0.97, pnoise(vec2<f32>(uv.x * 9.0 + id * 3.0, uv.y * 3.0), 9.0));
    var s: Sample;
    let base = mix(vec3<f32>(0.86, 0.74, 0.52), vec3<f32>(0.78, 0.64, 0.42), id);
    s.albedo = base * (0.88 + grain * 0.12 - rings * 0.18) * (1.0 - knot * 0.45) * mix(0.6, 1.0, gap);
    s.height = gap * (0.55 + grain * 0.05 - rings * 0.03 - knot * 0.08) + (1.0 - gap) * 0.3;
    s.rough = 0.6 + grain * 0.15 + knot * 0.15;
    return s;
}

fn mat_plywood(uv: vec2<f32>) -> Sample {
    // OSB: flak av treflis i alle retninger, gulbrun med mørke kanter,
    // og en svak skjøt mellom platene (1,2 m).
    var s: Sample;
    let v = voronoi(uv * 40.0, 40.0, 1.0);
    let id = v.id;
    let flake = pnoise(vec2<f32>(uv.x * 80.0 + id * 30.0, uv.y * 80.0), 80.0);
    let edge = smoothstep(0.0, 0.08, v.d1);
    let seam = smoothstep(0.0, 0.015, fract(uv.x)) * smoothstep(1.0, 0.985, fract(uv.x));
    let base = mix(vec3<f32>(0.78, 0.6, 0.34), vec3<f32>(0.6, 0.44, 0.22), id);
    s.albedo = base * (0.85 + flake * 0.3) * mix(0.75, 1.0, edge) * mix(0.6, 1.0, seam);
    s.height = 0.5 + (id - 0.5) * 0.3 + flake * 0.1;
    s.rough = 0.75 + flake * 0.1;
    return s;
}

fn mat_rock(uv: vec2<f32>) -> Sample {
    // Berg i stor skala: lag som ligger på skrå (strata), sprekker fra en
    // ridged støy, og lysere, forvitrede flater.
    let tilt = uv.x * 0.35 + uv.y;
    let strata = abs(fract(tilt * 9.0 + pfbm(uv * 5.0, 5.0, 3) * 0.8) - 0.5) * 2.0;
    let ridges = 1.0 - abs(pfbm(uv * 7.0 + 3.0, 7.0, 5) * 2.0 - 1.0);
    let cracks = smoothstep(0.75, 0.95, ridges);
    let weathered = pfbm(uv * 2.0 + 9.0, 2.0, 3);
    var s: Sample;
    let dark = vec3<f32>(0.04, 0.045, 0.05);
    let light = vec3<f32>(0.2, 0.19, 0.18);
    s.albedo = mix(dark, light, weathered * 0.7 + strata * 0.3) * (1.0 - cracks * 0.5) * (0.85 + ridges * 0.2);
    s.height = 0.5 + strata * 0.2 - cracks * 0.35 + ridges * 0.15;
    s.rough = 0.9 - weathered * 0.1;
    return s;
}

fn mat_metal(uv: vec2<f32>) -> Sample {
    let dent = pfbm(uv * 8.0, 8.0, 3);
    let rust = smoothstep(0.62, 0.8, pfbm(uv * 5.0 + 4.0, 5.0, 4));
    let fine = pfbm(uv * 90.0, 90.0, 2);
    var s: Sample;
    s.albedo = mix(vec3<f32>(0.16, 0.17, 0.18) * (0.9 + fine * 0.2), vec3<f32>(0.40, 0.20, 0.10) * (0.8 + fine * 0.4), rust);
    s.height = 0.5 + (dent - 0.5) * 0.15 - rust * 0.1 + (fine - 0.5) * 0.05;
    s.rough = mix(0.4 + fine * 0.1, 0.85, rust);
    return s;
}

fn mat_grass(uv: vec2<f32>) -> Sample {
    let clumps = pfbm(uv * 14.0, 14.0, 4);
    let blades = pnoise(vec2<f32>(uv.x * 140.0, uv.y * 140.0), 140.0);
    let dirt = smoothstep(0.3, 0.1, pfbm(uv * 3.0 + 1.0, 3.0, 3));
    var s: Sample;
    let green = mix(vec3<f32>(0.22, 0.36, 0.12), vec3<f32>(0.40, 0.46, 0.16), clumps);
    s.albedo = mix(green * (0.8 + blades * 0.4), vec3<f32>(0.32, 0.26, 0.18), dirt);
    s.height = 0.4 + clumps * 0.3 + blades * 0.2 - dirt * 0.2;
    s.rough = 0.9;
    return s;
}

fn mat_gravel(uv: vec2<f32>) -> Sample {
    let v = voronoi(uv * 40.0, 40.0, 0.9);
    let pebble = smoothstep(0.0, 0.25, v.d2 - v.d1);
    let dome = sqrt(max(0.0, 1.0 - min(v.d1 * 2.2, 1.0) * min(v.d1 * 2.2, 1.0)));
    let fine = pfbm(uv * 80.0, 80.0, 2);
    var s: Sample;
    s.albedo = mix(vec3<f32>(0.40, 0.37, 0.33), mix(vec3<f32>(0.5, 0.48, 0.45), vec3<f32>(0.62, 0.58, 0.52), v.id), pebble) * (0.85 + fine * 0.3);
    s.height = pebble * dome * 0.7 + fine * 0.15;
    s.rough = 0.85;
    return s;
}

fn mat_concrete(uv: vec2<f32>) -> Sample {
    let fine = pfbm(uv * 70.0, 70.0, 3);
    let pits = smoothstep(0.78, 0.9, pnoise(uv * 50.0, 50.0));
    let stain = pfbm(uv * 3.0, 3.0, 3);
    var s: Sample;
    s.albedo = vec3<f32>(0.62, 0.61, 0.58) * (0.9 + fine * 0.15) * (1.0 - pits * 0.3) * (0.9 + stain * 0.15);
    s.height = 0.5 + (fine - 0.5) * 0.1 - pits * 0.3;
    s.rough = 0.85 + fine * 0.1;
    return s;
}

fn material(layer: i32, uv: vec2<f32>) -> Sample {
    switch layer {
        case 0: { return mat_cobble(uv); }
        case 1: { return mat_plaster(uv); }
        case 2: { return mat_cladding(uv); }
        case 3: { return mat_slate(uv); }
        case 4: { return mat_tiles(uv); }
        case 5: { return mat_stone(uv); }
        case 6: { return mat_brick(uv); }
        case 7: { return mat_oak(uv); }
        case 8: { return mat_metal(uv); }
        case 9: { return mat_grass(uv); }
        case 10: { return mat_plaster(uv); }
        case 11: { return mat_gravel(uv); }
        case 12: { return mat_metal(uv); }
        case 13: { return mat_timber(uv); }
        case 14: { return mat_plywood(uv); }
        case 15: { return mat_rock(uv); }
        case 16: { return mat_plaster(uv); }
        case 17: { return mat_gravel(uv); }
        case 18: { return mat_tiles(uv); }
        case 19: { return mat_timber(uv); }
        case 20: { return mat_grass(uv); }
        case 21: { return mat_plaster(uv); }
        case 22: { return mat_plaster(uv); }
        case 23: { return mat_oak(uv); }
        case 24: { return mat_plaster(uv); }
        case 25: { return mat_tiles(uv); }
        default: { return mat_concrete(uv); }
    }
}

// Hvor sterkt høyden preger normalen, per lag (i meter relieff per flis).
fn relief(layer: i32) -> f32 {
    switch layer {
        case 0: { return 0.06; }
        case 1: { return 0.004; }
        case 2: { return 0.02; }
        case 3: { return 0.03; }
        case 4: { return 0.05; }
        case 5: { return 0.03; }
        case 6: { return 0.015; }
        case 7: { return 0.012; }
        case 8: { return 0.006; }
        case 9: { return 0.04; }
        case 10: { return 0.004; }
        case 11: { return 0.02; }
        case 12: { return 0.006; }
        case 13: { return 0.008; }
        case 14: { return 0.004; }
        case 15: { return 1.5; }
        case 16: { return 0.002; }
        case 17: { return 0.02; }
        case 18: { return 0.05; }
        case 19: { return 0.03; }
        case 20: { return 0.04; }
        case 21: { return 0.002; }
        case 22: { return 0.002; }
        case 23: { return 0.002; }
        case 24: { return 0.002; }
        case 25: { return 0.02; }
        default: { return 0.004; }
    }
}

struct BakeOut {
    @location(0) albedo: vec4<f32>,
    @location(1) nrh: vec4<f32>,
}

@fragment
fn fs_bake(i: VsOut) -> BakeOut {
    let layer = i32(bake.params.x);
    let tile = bake.params.y;
    let e = bake.params.z;
    let s = material(layer, i.uv);
    let hx = material(layer, i.uv + vec2<f32>(e, 0.0)).height - material(layer, i.uv - vec2<f32>(e, 0.0)).height;
    let hy = material(layer, i.uv + vec2<f32>(0.0, e)).height - material(layer, i.uv - vec2<f32>(0.0, e)).height;
    // Høyde i meter per meter: relieff / (2e × flis).
    let k = relief(layer) / (2.0 * e * tile);
    // uv.y går nedover i bildet; normalens y skal peke opp i flisa.
    let n = normalize(vec3<f32>(-hx * k, hy * k, 1.0));
    var o: BakeOut;
    o.albedo = vec4<f32>(s.albedo, 1.0);
    o.nrh = vec4<f32>(n.xy * 0.5 + 0.5, clamp(s.rough, 0.0, 1.0), clamp(s.height, 0.0, 1.0));
    return o;
}

// Ett mip-nivå ned: snittet av fire texels. Normalene normaliseres ved bruk.
@fragment
fn fs_mip(i: VsOut) -> BakeOut {
    var o: BakeOut;
    o.albedo = textureSampleLevel(src_a, src_sampler, i.uv, 0.0);
    o.nrh = textureSampleLevel(src_b, src_sampler, i.uv, 0.0);
    return o;
}

// Bladkort: en klase blader på gjennomsiktig bakgrunn. RGB farge, A dekning.
@fragment
fn fs_leaf(i: VsOut) -> @location(0) vec4<f32> {
    let p = i.uv * 2.0 - 1.0;
    var best = 0.0;
    // Bladgrønt også utenfor bladene, så mip-nivåene ikke mørkner mot svart.
    var col = vec3<f32>(0.28, 0.42, 0.12);
    for (var k = 0; k < 26; k++) {
        let fk = f32(k);
        let h1 = hash21(vec2<f32>(fk, 1.0));
        let h2 = hash21(vec2<f32>(fk, 2.0));
        let h3 = hash21(vec2<f32>(fk, 3.0));
        // Sentre spredt i en sirkel, bladene peker utover.
        let r = 0.15 + 0.6 * sqrt(h1);
        let a = h2 * 6.2831;
        let c = vec2<f32>(cos(a), sin(a)) * r;
        let dir = normalize(c + vec2<f32>(cos(h3 * 6.2831), sin(h3 * 6.2831)) * 0.3);
        let d = p - c;
        let along = dot(d, dir);
        let across = dot(d, vec2<f32>(-dir.y, dir.x));
        let len = 0.26 + h3 * 0.12;
        let t = along / len;
        // Bladform: bred på midten, spiss i begge ender.
        let w = 0.08 * sin(clamp(t + 0.5, 0.0, 1.0) * PI) * (1.0 + h1 * 0.4);
        let inside = step(abs(across), w) * step(-0.5, t) * step(t, 0.5);
        let rib = smoothstep(0.012, 0.0, abs(across)) * 0.3;
        let vein = smoothstep(0.015, 0.0, abs(fract(t * 6.0 + across * 3.0) - 0.5) * 0.1) * 0.08;
        if (inside > best) {
            best = inside;
            let green = mix(vec3<f32>(0.16, 0.32, 0.08), vec3<f32>(0.36, 0.50, 0.14), h2);
            let yellow = vec3<f32>(0.5, 0.48, 0.12);
            col = mix(green, yellow, smoothstep(0.8, 1.0, h1) * 0.6) * (1.0 - rib + vein) * (0.85 + (t + 0.5) * 0.2);
        }
    }
    // Litt dekning i kantene tas av alfatesten; gi 0 rundt.
    return vec4<f32>(col, best);
}

// Ett mip-nivå ned for lagene i array-teksturene.
@fragment
fn fs_mip_array(i: VsOut) -> BakeOut {
    let layer = i32(bake.params.x);
    var o: BakeOut;
    o.albedo = textureSampleLevel(arr_a, src_sampler, i.uv, layer, 0.0);
    o.nrh = textureSampleLevel(arr_b, src_sampler, i.uv, layer, 0.0);
    return o;
}

// Ett mip-nivå ned for én tekstur alene.
@fragment
fn fs_mip_one(i: VsOut) -> @location(0) vec4<f32> {
    return textureSampleLevel(src_a, src_sampler, i.uv, 0.0);
}
