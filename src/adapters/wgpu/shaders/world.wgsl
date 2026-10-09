// Verden: alt som har geometri. Materialene er bakte teksturlag lagt på
// triplanart fra verdensposisjonen, så ingenting trenger uv, og alt tåler
// at du går helt inntil. Vinduer har rom bak glasset (interior mapping),
// blad og tekst bruker uv, og modeller bruker egne teksturer.
//
// Samme vertex-shader brukes av dybdeprepasset, skyggene og hovedpasset, så
// TAA-forskyvningen og vinden er identisk og dybden kan testes med Equal.

struct Instance {
    @location(7) m0: vec4<f32>,
    @location(8) m1: vec4<f32>,
    @location(9) m2: vec4<f32>,
    @location(10) m3: vec4<f32>,
    // x markering, y animasjon (0..1) eller lakk; z utseendet (lagene for
    // kjellerens fire roller, 5 bit hver, bit 20 slår det på), w tonen.
    @location(11) extra: vec4<f32>,
    // x: plassen i leddpaletten + 1 for skinnede instanser, 0 ellers;
    // y: flammefargen i utseendet; z: plassen til forrige bildes transform
    // i forrige palett + 1; w: forrige bildes skinn i forrige palett + 1.
    @location(12) extra2: vec4<f32>,
}

/// Leddmatrisene for skinnede figurer: fire texels (kolonnene) per
/// matrise, 256 matriser per rad.
@group(0) @binding(1) var skin_tex: texture_2d<f32>;
/// Paletten fra forrige bilde, til bevegelsesvektorene. Bak skinnene ligger
/// transformene til de bevegelige instansene (extra2.z er plassen + 1).
@group(0) @binding(2) var prev_skin_tex: texture_2d<f32>;

struct Pass {
    // x kaskade (skygge) eller 0.
    params: vec4<f32>,
}
@group(1) @binding(2) var ao_tex: texture_2d<f32>;
@group(1) @binding(3) var frame_sampler: sampler;
@group(1) @binding(4) var<uniform> pass_u: Pass;

@group(2) @binding(0) var tex_albedo: texture_2d_array<f32>;
@group(2) @binding(1) var tex_nrh: texture_2d_array<f32>;
@group(2) @binding(2) var tex_sampler: sampler;
@group(2) @binding(4) var leaf_tex: texture_2d<f32>;
@group(2) @binding(5) var clamp_sampler: sampler;

@group(3) @binding(0) var model_albedo: texture_2d<f32>;
@group(3) @binding(1) var model_normal: texture_2d<f32>;
@group(3) @binding(2) var model_mr: texture_2d<f32>;
@group(3) @binding(3) var model_emissive: texture_2d<f32>;

const K_PLAIN: u32 = 200u;
const K_GLASS: u32 = 201u;
const K_LEAF: u32 = 202u;
const K_MAGIC: u32 = 203u;
const K_LAMP: u32 = 204u;
const K_SKELETON: u32 = 205u;
const K_WATER: u32 = 206u;
const K_MODEL: u32 = 207u;
const K_TERRAIN: u32 = 209u;

const F_WIND: u32 = 4u;
const F_ROT_UV: u32 = 8u;
const F_NO_WEATHER: u32 = 16u;
const F_NO_DIRT: u32 = 32u;
const F_ALPHA: u32 = 64u;
const F_METAL: u32 = 128u;
/// Teksturlag etter flatens uv i meter (modeller med «layer_»-materialer).
const F_UVMAP: u32 = 128u;
/// Lakk på modeller: samme bit som ROT_UV, som modeller ikke bruker.
const F_PAINT: u32 = 8u;

struct VsIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec4<f32>, // snorm8, w ubrukt
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) mat: vec4<u32>,
    @location(5) joints: vec4<u32>,
    @location(6) weights: vec4<f32>,
}

struct VsOut {
    @builtin(position) @invariant clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) @interpolate(flat) mat: vec4<u32>,
    @location(4) uv: vec2<f32>,
    @location(5) @interpolate(flat) extra: vec4<f32>,
    // Okklusjonen fra modellen, blandet over trekanten (mat er flat).
    @location(6) ao: f32,
    // Bakken: tørt gress og søle (vektene i hjørnene).
    @location(7) dw: vec2<f32>,
}

fn joint_mat(i: u32) -> mat4x4<f32> {
    let x = i32((i % 256u) * 4u);
    let y = i32(i / 256u);
    return mat4x4<f32>(
        textureLoad(skin_tex, vec2<i32>(x, y), 0),
        textureLoad(skin_tex, vec2<i32>(x + 1, y), 0),
        textureLoad(skin_tex, vec2<i32>(x + 2, y), 0),
        textureLoad(skin_tex, vec2<i32>(x + 3, y), 0),
    );
}

// Hjørnets skinnmatrise: de fire leddene vektet.
fn skin_of(v: VsIn, base: u32) -> mat4x4<f32> {
    return joint_mat(base + v.joints.x) * v.weights.x
        + joint_mat(base + v.joints.y) * v.weights.y
        + joint_mat(base + v.joints.z) * v.weights.z
        + joint_mat(base + v.joints.w) * v.weights.w;
}

fn prev_joint_mat(i: u32) -> mat4x4<f32> {
    let x = i32((i % 256u) * 4u);
    let y = i32(i / 256u);
    return mat4x4<f32>(
        textureLoad(prev_skin_tex, vec2<i32>(x, y), 0),
        textureLoad(prev_skin_tex, vec2<i32>(x + 1, y), 0),
        textureLoad(prev_skin_tex, vec2<i32>(x + 2, y), 0),
        textureLoad(prev_skin_tex, vec2<i32>(x + 3, y), 0),
    );
}

fn prev_skin_of(v: VsIn, base: u32) -> mat4x4<f32> {
    return prev_joint_mat(base + v.joints.x) * v.weights.x
        + prev_joint_mat(base + v.joints.y) * v.weights.y
        + prev_joint_mat(base + v.joints.z) * v.weights.z
        + prev_joint_mat(base + v.joints.w) * v.weights.w;
}

// Matrisen forrige bilde: samme som nå for det som står stille.
fn prev_model_of(i: Instance, v: VsIn) -> mat4x4<f32> {
    var m = mat4x4<f32>(i.m0, i.m1, i.m2, i.m3);
    if (i.extra2.z > 0.5) {
        m = prev_joint_mat(u32(i.extra2.z) - 1u);
    }
    if (i.extra2.x > 0.5) {
        if (i.extra2.w > 0.5) {
            return m * prev_skin_of(v, u32(i.extra2.w) - 1u);
        }
        return m * skin_of(v, u32(i.extra2.x) - 1u);
    }
    return m;
}

// Instansens matrise, med skinningen inni for skinnede figurer.
fn model_of(i: Instance, v: VsIn) -> mat4x4<f32> {
    let m = mat4x4<f32>(i.m0, i.m1, i.m2, i.m3);
    if (i.extra2.x > 0.5) {
        return m * skin_of(v, u32(i.extra2.x) - 1u);
    }
    return m;
}

fn jittered(clip: vec4<f32>) -> vec4<f32> {
    var c = clip;
    c.x += g.jitter.x * g.screen.z * 2.0 * clip.w;
    c.y -= g.jitter.y * g.screen.w * 2.0 * clip.w;
    return c;
}

fn wind_offset(world: vec3<f32>, v: VsIn) -> vec3<f32> {
    if ((v.mat.z & F_WIND) == 0u) {
        return vec3<f32>(0.0);
    }
    let t = g.camera.w;
    let strength = g.jitter.w;
    // Hele treet svaier sakte, bladene flagrer raskere. uv.y er hvor høyt opp i kronen.
    let sway = sin(t * 0.9 + world.x * 0.3 + world.z * 0.2) * 0.5 + sin(t * 1.7 + world.z * 0.5) * 0.25;
    let flutter = sin(t * 5.0 + world.x * 7.0 + world.y * 5.0) * 0.5 + sin(t * 7.3 + world.z * 9.0) * 0.5;
    // Bladene bærer vindvekten i okklusjonsbyten; alfakort fra modellene
    // (blad, gress) svaier etter høyden over rota; annet etter uv.y.
    var h = clamp(v.uv.y, 0.0, 1.0);
    if (v.mat.x == K_LEAF) {
        h = f32(v.mat.w) / 255.0;
    } else if ((v.mat.z & F_ALPHA) != 0u) {
        h = clamp(v.pos.y / 0.7, 0.0, 1.0);
    }
    return vec3<f32>(sway * 0.12 + flutter * 0.025, flutter * 0.015, sway * 0.08) * (0.3 + strength) * h;
}

@vertex
fn vs(v: VsIn, inst: Instance) -> VsOut {
    return vertex(v, inst);
}

// Hovedpasset og prepasset regner likt, så dybden kan testes med Equal.
fn vertex(v: VsIn, inst: Instance) -> VsOut {
    var o: VsOut;
    let m = model_of(inst, v);
    var w = (m * vec4<f32>(v.pos, 1.0)).xyz;
    w += wind_offset(w, v);
    o.world = w;
    o.clip = jittered(g.view_proj * vec4<f32>(w, 1.0));
    o.normal = (m * vec4<f32>(v.normal.xyz, 0.0)).xyz;
    o.color = v.color;
    o.mat = v.mat;
    o.ao = f32(v.mat.w) / 255.0;
    o.uv = v.uv;
    o.dw = v.weights.xy;
    o.extra = inst.extra;
    look(&o, v, inst);
    return o;
}

fn unpack_rgb(p: f32) -> vec3<f32> {
    let u = u32(p);
    return vec3<f32>(f32((u >> 16u) & 255u), f32((u >> 8u) & 255u), f32(u & 255u)) / 255.0;
}

// Utseendet per instans (kjelleretasjene): rollene mur (stein, 5), hvelv
// (hogd stein, 6), kvader (heller, 12) og gulv (fliser, 4) får lagene og
// tonen fra instansen, og flammene en annen farge. Hjørnefargen er sRGB.
fn look(o: ptr<function, VsOut>, v: VsIn, inst: Instance) {
    if (inst.extra.z < 1048576.0) {
        return;
    }
    let roles = u32(inst.extra.z);
    var role = 4u;
    switch v.mat.x {
        case 5u: { role = 0u; }
        case 6u: { role = 1u; }
        case 12u: { role = 2u; }
        case 4u: { role = 3u; }
        default: {}
    }
    if (role < 4u) {
        (*o).mat.x = (roles >> (5u * role)) & 31u;
        (*o).color = vec4<f32>((*o).color.rgb * pow(unpack_rgb(inst.extra.w), vec3<f32>(1.0 / 2.2)), (*o).color.a);
    } else if (v.mat.x == K_LAMP && inst.extra2.y > 0.0) {
        (*o).color = vec4<f32>(pow(unpack_rgb(inst.extra2.y), vec3<f32>(1.0 / 2.2)), (*o).color.a);
    }
}

struct ShadowOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) mat: vec4<u32>,
}

@vertex
fn vs_shadow(v: VsIn, inst: Instance) -> ShadowOut {
    let m = model_of(inst, v);
    var w = (m * vec4<f32>(v.pos, 1.0)).xyz;
    w += wind_offset(w, v);
    var o: ShadowOut;
    o.clip = g.sun_vp[i32(pass_u.params.x)] * vec4<f32>(w, 1.0);
    o.uv = v.uv;
    o.mat = v.mat;
    return o;
}

// Dekning for blad og modellblader, som er gjennomhullet med alfatest.
// Dawn krever at textureSample står i uniform kontrollflyt, og grenene her
// avhenger av per-piksel-verdier. Derfor sendes uv-gradientene inn fra
// toppen av passet (der flyten er uniform) og samplingen bruker dem.
fn coverage(kind: u32, flags: u32, uv: vec2<f32>, duvx: vec2<f32>, duvy: vec2<f32>) -> f32 {
    if (kind == K_LEAF) {
        return textureSampleGrad(leaf_tex, clamp_sampler, uv, duvx, duvy).a;
    }
    if (kind == K_MODEL && (flags & F_ALPHA) != 0u) {
        // Teksturene er JPEG uten alfa; bakgrunnen bak bladene er svart.
        let a = textureSampleGrad(model_albedo, tex_sampler, uv, duvx, duvy);
        return a.a * smoothstep(0.03, 0.12, max(a.r, max(a.g, a.b)));
    }
    return 1.0;
}

@fragment
fn fs_shadow(i: ShadowOut) {
    let duvx = dpdx(i.uv);
    let duvy = dpdy(i.uv);
    if (coverage(i.mat.x, i.mat.z, i.uv, duvx, duvy) < 0.5) {
        discard;
    }
}

// Dybdeprepasset skriver normalen til SSAO og bevegelsen på skjermen til
// TAA, og tester alfa likt med hovedpasset.
struct PreOut {
    @location(0) normal: vec4<f32>,
    @location(1) depth: f32,
    @location(2) velocity: vec2<f32>,
}

struct VsPre {
    @builtin(position) @invariant clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) mat: vec4<u32>,
    // Uten TAA-forskyvning: nå og forrige bilde.
    @location(3) now: vec4<f32>,
    @location(4) before: vec4<f32>,
}

@vertex
fn vs_prepass(v: VsIn, inst: Instance) -> VsPre {
    let o = vertex(v, inst);
    var p: VsPre;
    p.clip = o.clip;
    p.normal = o.normal;
    p.uv = o.uv;
    p.mat = o.mat;
    p.now = g.view_proj * vec4<f32>(o.world, 1.0);
    var w = (prev_model_of(inst, v) * vec4<f32>(v.pos, 1.0)).xyz;
    w += wind_offset(w, v);
    p.before = g.prev_view_proj * vec4<f32>(w, 1.0);
    return p;
}

@fragment
fn fs_prepass(i: VsPre, @builtin(front_facing) front: bool) -> PreOut {
    let duvx = dpdx(i.uv);
    let duvy = dpdy(i.uv);
    if (coverage(i.mat.x, i.mat.z, i.uv, duvx, duvy) < 0.5) {
        discard;
    }
    var n = normalize(i.normal);
    if (!front) {
        n = -n;
    }
    var o: PreOut;
    o.normal = vec4<f32>(n * 0.5 + 0.5, 1.0);
    o.depth = i.clip.z;
    // Fra ndc til uv (y ned).
    let d = i.now.xy / i.now.w - i.before.xy / max(i.before.w, 1e-6);
    o.velocity = vec2<f32>(d.x * 0.5, -d.y * 0.5);
    return o;
}

fn ggx(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, rough: f32, f0: vec3<f32>) -> vec3<f32> {
    let h = normalize(v + l);
    let a = max(rough * rough, 0.002);
    let a2 = a * a;
    let ndh = max(dot(n, h), 0.0);
    let ndl = max(dot(n, l), 0.0);
    let ndv = max(dot(n, v), 1e-3);
    let d = a2 / (PI * pow(ndh * ndh * (a2 - 1.0) + 1.0, 2.0));
    let k = a * 0.5;
    let vis = 0.25 / ((ndl * (1.0 - k) + k) * (ndv * (1.0 - k) + k));
    let f = f0 + (1.0 - f0) * pow(1.0 - max(dot(v, h), 0.0), 5.0);
    return d * vis * f;
}

struct Surf {
    albedo: vec3<f32>,
    rough: f32,
    metal: f32,
    n: vec3<f32>,
    cavity: f32,
    emissive: vec3<f32>,
    // 0 vanlig, 1 blad (lys gjennom), 2 glass.
    kind: u32,
    // Okklusjon fra geometrien.
    ao: f32,
}

// Triplanar oppslag i de bakte lagene. Returnerer albedo, normal i verden,
// ruhet og høyde.
struct Tex {
    albedo: vec3<f32>,
    n: vec3<f32>,
    rough: f32,
    height: f32,
}

fn sample_plane(layer: i32, uv: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>, t: vec3<f32>, b: vec3<f32>, n: vec3<f32>) -> Tex {
    let a = textureSampleGrad(tex_albedo, tex_sampler, uv, layer, gx, gy).rgb;
    let nrh = textureSampleGrad(tex_nrh, tex_sampler, uv, layer, gx, gy);
    let tn = vec3<f32>(nrh.xy * 2.0 - 1.0, 0.0);
    let tz = sqrt(max(0.0, 1.0 - dot(tn.xy, tn.xy)));
    var o: Tex;
    o.albedo = a;
    o.n = normalize(t * tn.x + b * tn.y + n * tz);
    o.rough = nrh.z;
    o.height = nrh.w;
    return o;
}

fn triplanar(layer: i32, world: vec3<f32>, dwx: vec3<f32>, dwy: vec3<f32>, n_geo: vec3<f32>, rot: bool) -> Tex {
    return triplanar_s(layer, world, dwx, dwy, n_geo, rot, 1.0);
}

// Som `triplanar`, med flisa `mul` ganger større.
fn triplanar_s(layer: i32, world: vec3<f32>, dwx: vec3<f32>, dwy: vec3<f32>, n_geo: vec3<f32>, rot: bool, mul: f32) -> Tex {
    let scale = g.tex_scale[layer / 4][layer % 4] * mul;
    let p = world / scale;
    // Gradientene til uv-ene følger av gradientene til verdensposisjonen.
    let dx = dwx / scale;
    let dy = dwy / scale;
    var w = pow(abs(n_geo), vec3<f32>(6.0));
    w = w / (w.x + w.y + w.z);
    // Flaten i xz (gulv og tak), xy (vegger mot ±z) og zy (vegger mot ±x).
    // Teksturens y skal peke opp på vegger, og normalens z ut av flaten.
    let sx = sign(n_geo.x + 1e-5);
    let sz = sign(n_geo.z + 1e-5);
    let sy = sign(n_geo.y + 1e-5);
    var uv_x = vec2<f32>(-p.z * sx, p.y);
    var uv_z = vec2<f32>(p.x * sz, p.y);
    var gx_x = vec2<f32>(-dx.z * sx, dx.y);
    var gy_x = vec2<f32>(-dy.z * sx, dy.y);
    var gx_z = vec2<f32>(dx.x * sz, dx.y);
    var gy_z = vec2<f32>(dy.x * sz, dy.y);
    if (rot) {
        uv_x = vec2<f32>(p.y, p.z * sx);
        uv_z = vec2<f32>(p.y, -p.x * sz);
        gx_x = vec2<f32>(dx.y, dx.z * sx);
        gy_x = vec2<f32>(dy.y, dy.z * sx);
        gx_z = vec2<f32>(dx.y, -dx.x * sz);
        gy_z = vec2<f32>(dy.y, -dy.x * sz);
    }
    let uv_y = vec2<f32>(p.x, -p.z * sy);
    let gx_y = vec2<f32>(dx.x, -dx.z * sy);
    let gy_y = vec2<f32>(dy.x, -dy.z * sy);
    var o: Tex;
    o.albedo = vec3<f32>(0.0);
    o.n = vec3<f32>(0.0);
    o.rough = 0.0;
    o.height = 0.0;
    if (w.x > 0.01) {
        var t = vec3<f32>(0.0, 0.0, -sx);
        var b = vec3<f32>(0.0, 1.0, 0.0);
        if (rot) { t = vec3<f32>(0.0, 1.0, 0.0); b = vec3<f32>(0.0, 0.0, sx); }
        let s = sample_plane(layer, uv_x, gx_x, gy_x, t, b, vec3<f32>(sx, 0.0, 0.0));
        o.albedo += s.albedo * w.x; o.n += s.n * w.x; o.rough += s.rough * w.x; o.height += s.height * w.x;
    }
    if (w.z > 0.01) {
        var t = vec3<f32>(sz, 0.0, 0.0);
        var b = vec3<f32>(0.0, 1.0, 0.0);
        if (rot) { t = vec3<f32>(0.0, 1.0, 0.0); b = vec3<f32>(-sz, 0.0, 0.0); }
        let s = sample_plane(layer, uv_z, gx_z, gy_z, t, b, vec3<f32>(0.0, 0.0, sz));
        o.albedo += s.albedo * w.z; o.n += s.n * w.z; o.rough += s.rough * w.z; o.height += s.height * w.z;
    }
    if (w.y > 0.01) {
        let s = sample_plane(layer, uv_y, gx_y, gy_y, vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, -sy), vec3<f32>(0.0, sy, 0.0));
        o.albedo += s.albedo * w.y; o.n += s.n * w.y; o.rough += s.rough * w.y; o.height += s.height * w.y;
    }
    o.n = normalize(o.n);
    return o;
}

// Et teksturlag lagt på etter flatens egen uv (i meter), med tangentene
// fra skjermderivatene, så årene følger bjelken og radene takflaten.
fn uvmapped(layer: i32, uv_m: vec2<f32>, duvx_m: vec2<f32>, duvy_m: vec2<f32>, dp1: vec3<f32>, dp2: vec3<f32>, n: vec3<f32>) -> Tex {
    let scale = g.tex_scale[layer / 4][layer % 4];
    let uv = uv_m / scale;
    let gx = duvx_m / scale;
    let gy = duvy_m / scale;
    let dp2perp = cross(dp2, n);
    let dp1perp = cross(n, dp1);
    var t = dp2perp * gx.x + dp1perp * gy.x;
    var b = dp2perp * gx.y + dp1perp * gy.y;
    let invmax = inverseSqrt(max(dot(t, t), dot(b, b)) + 1e-12);
    t = t * invmax;
    b = b * invmax;
    if (dot(t, t) < 1e-8) {
        t = normalize(cross(n, vec3<f32>(0.0, 1.0, 0.3)));
        b = cross(n, t);
    }
    return sample_plane(layer, uv, gx, gy, t, b, n);
}

// Bakken: brostein, jord, berg, skogbunn og gress blandet etter vektene i
// hjørnene og høydekartene, så steinene stikker opp gjennom jorda og gresset
// vokser mellom steinene, i stedet for harde kanter. Planart fra oven, berget
// triplanart (det står i skråninger).
// `road`: koordinatene langs nærmeste jordvei i meter (på tvers, langs),
// med gradientene `dr1`/`dr2`. Søla med hjulsporene legges etter dem, så
// sporene følger veien og bøyer seg i svingene. Der to veier møtes, hopper
// koordinatene over en trekant; der strekkes de, og søla tar verdens akser.
// `earth`: hvor mye av jorda som er tråkket jord (lag 17) i stedet for søle.
fn terrain(w_raw: vec4<f32>, dry: f32, wet: f32, world: vec3<f32>, dp1: vec3<f32>, dp2: vec3<f32>, n: vec3<f32>, road: vec2<f32>, dr1: vec2<f32>, dr2: vec2<f32>, earth: f32) -> Tex {
    // Vektene kommer fra hjørnene i et rutenett på 2 m og blandes lineært
    // over trekantene; i overgangene forskyves de med støy i to skalaer,
    // så kantene mellom brostein, jord og gress blir ujevne og flekkete i
    // stedet for å følge rutene.
    let warp = vec4<f32>(
        fbm2(world.xz * 0.55 + vec2<f32>(3.1, 7.7)),
        fbm2(world.xz * 0.45 + vec2<f32>(9.2, 1.3)),
        fbm2(world.xz * 0.35 + vec2<f32>(5.5, 4.4)),
        fbm2(world.xz * 0.3 + vec2<f32>(2.2, 8.8)),
    ) + vec4<f32>(
        fbm2(world.xz * 2.1 + vec2<f32>(1.7, 3.3)),
        fbm2(world.xz * 1.9 + vec2<f32>(6.1, 2.2)),
        fbm2(world.xz * 1.7 + vec2<f32>(4.4, 9.9)),
        fbm2(world.xz * 1.5 + vec2<f32>(8.8, 5.5)),
    ) * 0.4 - vec4<f32>(0.7);
    let band = clamp(w_raw * (vec4<f32>(1.0) - w_raw) * 4.0, vec4<f32>(0.0), vec4<f32>(1.0));
    var w_in = clamp(w_raw + warp * band * 0.9, vec4<f32>(0.0), vec4<f32>(1.0));
    let total_w = w_in.x + w_in.y + w_in.z + w_in.w;
    if (total_w > 1.0) {
        w_in /= total_w;
    }
    let grass_w = max(0.0, 1.0 - w_in.x - w_in.y - w_in.z - w_in.w);
    // Lagene: brostein 0, søle 11, berg 15, skogbunn 14, gress 9, tråkket jord 17.
    var layers = array<i32, 6>(0, 11, 15, 14, 9, 17);
    var weights = array<f32, 6>(w_in.x, w_in.y * (1.0 - earth), w_in.z, w_in.w, grass_w, w_in.y * earth);
    var samples: array<Tex, 6>;
    var best = -1.0;
    for (var k = 0; k < 6; k++) {
        if (weights[k] > 0.003) {
            let layer = layers[k];
            if (layer == 15) {
                samples[k] = triplanar(layer, world, dp1, dp2, n, false);
            } else if (layer == 11) {
                let scale = g.tex_scale[layer / 4][layer % 4];
                // Retningene på bakken der veikoordinatene vokser (løst fra
                // gradientene), og hvor mange meter en meter veikoordinat er:
                // nær 1 langs veien, mye mindre der koordinatene hopper.
                let det = dr1.x * dr2.y - dr1.y * dr2.x;
                var pu = (dp1 * dr2.y - dp2 * dr1.y) / det;
                var pv = (dp2 * dr1.x - dp1 * dr2.x) / det;
                let trust = select(0.0, smoothstep(0.35, 0.7, min(length(pu.xz), length(pv.xz))), abs(det) > 1e-12);
                pu = vec3<f32>(pu.x, 0.0, pu.z);
                pv = vec3<f32>(pv.x, 0.0, pv.z);
                let tu = select(vec3<f32>(1.0, 0.0, 0.0), normalize(pu), dot(pu, pu) > 1e-10);
                let tv = select(vec3<f32>(0.0, 0.0, -1.0), normalize(pv), dot(pv, pv) > 1e-10);
                let along = sample_plane(layer, road / scale, dr1 / scale, dr2 / scale, tu, tv, vec3<f32>(0.0, 1.0, 0.0));
                let uv = vec2<f32>(world.x, -world.z) / scale;
                let flat = sample_plane(layer, uv, vec2<f32>(dp1.x, -dp1.z) / scale, vec2<f32>(dp2.x, -dp2.z) / scale, vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, -1.0), vec3<f32>(0.0, 1.0, 0.0));
                var s = flat;
                s.albedo = mix(flat.albedo, along.albedo, trust);
                s.n = normalize(mix(flat.n, along.n, trust));
                s.rough = mix(flat.rough, along.rough, trust);
                s.height = mix(flat.height, along.height, trust);
                samples[k] = s;
                samples[k].n = normalize(n + vec3<f32>(s.n.x, 0.0, s.n.z));
            } else {
                let scale = g.tex_scale[layer / 4][layer % 4];
                let uv = vec2<f32>(world.x, -world.z) / scale;
                let s = sample_plane(layer, uv, vec2<f32>(dp1.x, -dp1.z) / scale, vec2<f32>(dp2.x, -dp2.z) / scale, vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, -1.0), vec3<f32>(0.0, 1.0, 0.0));
                samples[k] = s;
                // Normalen fra oven vippes etter bakken.
                samples[k].n = normalize(n + vec3<f32>(s.n.x, 0.0, s.n.z));
            }
            best = max(best, weights[k] + samples[k].height * 0.6);
        }
    }
    // Høydeblanding: det som stikker høyest der vektene er like, vinner.
    var o: Tex;
    o.albedo = vec3<f32>(0.0);
    o.n = vec3<f32>(0.0);
    o.rough = 0.0;
    o.height = 0.0;
    var total = 0.0;
    for (var k = 0; k < 6; k++) {
        if (weights[k] > 0.003) {
            let b = max(weights[k] + samples[k].height * 0.6 - best + 0.18, 0.0);
            o.albedo += samples[k].albedo * b;
            o.n += samples[k].n * b;
            o.rough += samples[k].rough * b;
            o.height += samples[k].height * b;
            total += b;
        }
    }
    o.albedo /= max(total, 1e-4);
    o.n = normalize(o.n);
    o.rough /= max(total, 1e-4);
    o.height /= max(total, 1e-4);
    // Gresset gulner på ryggene og i åkrene, og varierer i store flekker.
    let tone = mix(vec3<f32>(0.86, 0.9, 0.78), vec3<f32>(1.12, 0.98, 0.7), dry);
    let gw = grass_w * smoothstep(0.0, 0.3, grass_w);
    o.albedo *= mix(vec3<f32>(1.0), tone * (0.85 + 0.3 * fbm2(world.xz * 0.04)), gw);
    // Søle: mørkere, glattere og blank.
    o.albedo *= 1.0 - wet * 0.45;
    o.rough = mix(o.rough, 0.25, wet * 0.7);
    return o;
}

// Rommet bak et vindu. `p` er verdensposisjonen på glasset, `v` retningen
// fra øyet. Rommet er 3 × 3 × 3 m, forankret i gulvet og vinduscella.
fn interior(p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, night: f32) -> vec3<f32> {
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let t = normalize(cross(up, n));
    let b = cross(n, t);
    let room = 3.0;
    // Celle langs veggen og i høyden.
    let along = dot(p, t);
    let cell = vec3<f32>(floor(along / room), floor((p.y - 0.45) / 3.0), floor(dot(p, n) * 0.3));
    let h = hash31(cell);
    let lit = step(h, g.fog_color.w) * night;
    let warm = mix(vec3<f32>(1.0, 0.72, 0.42), vec3<f32>(0.95, 0.9, 0.8), hash31(cell + 3.0));
    // Stråle inn i rommet, i rommets koordinater: x langs, y opp, z innover.
    let d = vec3<f32>(dot(-v, t), -v.y, -dot(-v, n));
    let o = vec3<f32>(along - cell.x * room, p.y - 0.45 - cell.y * 3.0, 0.0);
    // Avstand til bakvegg, sidevegger, gulv og tak.
    var tmin = (room - o.z) / max(d.z, 1e-4);
    var which = 0;
    if (d.x > 0.0) { let tx = (room - o.x) / d.x; if (tx < tmin) { tmin = tx; which = 1; } }
    else if (d.x < 0.0) { let tx = -o.x / d.x; if (tx < tmin) { tmin = tx; which = 1; } }
    if (d.y > 0.0) { let ty = (3.0 - o.y) / d.y; if (ty < tmin) { tmin = ty; which = 2; } }
    else if (d.y < 0.0) { let ty = -o.y / d.y; if (ty < tmin) { tmin = ty; which = 3; } }
    let hit = o + d * tmin;
    var col = vec3<f32>(0.0);
    let wallcol = mix(vec3<f32>(0.75, 0.70, 0.62), vec3<f32>(0.55, 0.62, 0.66), hash31(cell + 5.0));
    if (which == 0) {
        // Bakveggen: en lampe og et bilde eller en bokhylle, som støy.
        let pic = fbm2(hit.xy * 1.5 + cell.xy * 7.0);
        col = wallcol * (0.5 + pic * 0.4) * (0.4 + 0.6 * smoothstep(3.0, 1.0, hit.y));
    } else if (which == 1) {
        col = wallcol * 0.5 * (0.4 + 0.6 * smoothstep(3.0, 0.5, hit.y));
    } else if (which == 2) {
        col = vec3<f32>(0.9, 0.88, 0.85) * 0.55;
    } else {
        col = vec3<f32>(0.35, 0.26, 0.18) * (0.6 + fbm2(hit.xz * 3.0) * 0.4);
    }
    // Gardin og møbler i forgrunnen mørkner det.
    let curtain = smoothstep(0.3, 0.7, fbm2(vec2<f32>(along * 2.0, p.y * 2.0) + cell.xy * 3.0));
    col *= mix(1.0, 0.3, curtain * 0.7);
    return col * warm * (0.015 + lit * (0.9 + h * 0.6));
}

fn shade(i: VsOut, s_in: Surf, n_geo: vec3<f32>) -> vec3<f32> {
    var s = s_in;
    let n = s.n;
    let v = normalize(g.camera.xyz - i.world);
    let night = g.sun_color.w;
    let f0 = mix(vec3<f32>(0.04), s.albedo, s.metal);
    // Smijern er ikke speil: hamret, oksidert og sotet jern sprer også lys,
    // ellers blir det svart innendørs der det ikke har himmel å speile.
    let diffuse_albedo = s.albedo * (1.0 - s.metal * 0.55);
    let screen_uv = i.clip.xy * g.screen.zw;
    let ssao = textureSampleLevel(ao_tex, frame_sampler, screen_uv, 0.0).r;

    // Sola eller månen.
    let l = g.sun_dir.xyz;
    let ndl = max(dot(n, l), 0.0);
    var wrap = ndl;
    if (s.kind == 1u) {
        wrap = max((dot(n, l) + 0.4) / 1.4, 0.0);
    }
    let sh = sun_shadow(i.world, n_geo, i.world.xz * 37.0 + i.world.y);
    let sun = g.sun_color.rgb * g.sun_dir.w * sh;
    var color = sun * (diffuse_albedo * wrap + ggx(n, v, l, s.rough, f0) * PI * ndl);
    if (s.kind == 1u) {
        // Lys gjennom bladet.
        // Dempet der løvet er tett og skygget (okklusjonen), så tynne
        // grantopper ikke lyser opp i motlys.
        let back = max(dot(-n, l), 0.0) * pow(max(dot(-v, l), 0.0), 3.0);
        color += sun * s.albedo * vec3<f32>(0.8, 0.95, 0.5) * back * 0.3 * s.ao * sh;
    }

    // Himmellys fra alle kanter, og lys som spretter opp fra bakken.
    let ao = s.ao * s.cavity * ssao;
    // Himmellyset er sterkere og mindre blått enn himmelfargen rett opp:
    // det kommer fra hele hvelvingen, mest fra horisonten.
    let sky_amb = mix(g.sky_top.rgb, g.sky_horizon.rgb, 0.6) * 1.25;
    var hemi = mix(g.ground_bounce.rgb, sky_amb, n.y * 0.5 + 0.5);
    if (s.kind == 1u) {
        // Løv og barnåler: normalene peker ut fra kronen, og i toppen rett
        // opp, men en tett krone skygger seg selv. Himmellyset tas nesten
        // likt fra alle kanter, så grantoppene ikke lyser hvite.
        hemi = mix(g.ground_bounce.rgb, sky_amb, 0.45 + n.y * 0.12) * 0.85;
    }
    color += diffuse_albedo * hemi * ao;

    // Refleksjon av himmelen, uskarpere jo ruere flaten er.
    let r = reflect(-v, n);
    let gloss = (1.0 - s.rough) * (1.0 - s.rough);
    let fres = f0 + (max(vec3<f32>(gloss), f0) - f0) * pow(1.0 - max(dot(n, v), 0.0), 5.0);
    let r_blur = normalize(mix(r, n, s.rough * s.rough));
    let env = mix(sky_gradient(r_blur), hemi, s.rough * 0.7) * 0.8;
    color += env * fres * ao * mix(0.6, 1.0, sh);

    // Lyktene og lysene i vinduene.
    let count = i32(g.params.y);
    for (var k = 0; k < count; k++) {
        let li = g.lights[k];
        let d = li.pos_radius.xyz - i.world;
        let dist = length(d);
        let rad = li.pos_radius.w;
        // I kjelleren stopper murene lyset (punktet flyttes litt ut fra
        // flaten, så muren ikke skygger for seg selv).
        if (dist < rad && grid_lit(i.world + n_geo * 0.35, li.pos_radius.xyz) > 0.5) {
            let ld = d / dist;
            let win = pow(clamp(1.0 - pow(dist / rad, 4.0), 0.0, 1.0), 2.0);
            let att = win / (dist * dist + 0.5);
            let pl = li.color_intensity.rgb * li.color_intensity.w * att;
            let pndl = max(dot(n, ld), 0.0);
            color += pl * (diffuse_albedo * pndl + ggx(n, v, ld, s.rough, f0) * PI * pndl) * mix(0.5, 1.0, ao);
        }
    }

    color += s.emissive;

    // Markering når du sikter på noe.
    // Markering når du sikter på noe (0..1), og hvitt blink når en fiende
    // er truffet (over 1).
    let hl = i.extra.x;
    if (hl > 0.0) {
        let rim = pow(1.0 - max(dot(n, v), 0.0), 2.0);
        let mark = select(min(hl, 1.0), 0.0, hl > 1.0);
        let flash = max(hl - 1.0, 0.0);
        color += (s.albedo * 0.35 + vec3<f32>(rim * 0.6)) * mark * (0.5 + 0.5 * sin(g.camera.w * 5.0)) + vec3<f32>(2.5) * flash;
    }
    return color;
}

// Vær: regnvåte flater blir mørkere og blankere, og det samler seg vann i
// pyttene i brosteinen. Snø legger seg på det som vender opp.
fn weather(s_in: Surf, world: vec3<f32>, n_geo: vec3<f32>, height: f32, flags: u32) -> Surf {
    var s = s_in;
    if ((flags & F_NO_WEATHER) != 0u || s.kind == 2u) {
        return s;
    }
    let up = clamp(n_geo.y, 0.0, 1.0);
    let wet = g.weather.x;
    if (wet > 0.0) {
        let exposure = mix(0.4, 1.0, up);
        let w = wet * exposure;
        s.albedo *= 1.0 - 0.4 * w;
        s.rough = mix(s.rough, 0.15, w * 0.8);
        // Pytter der høyden er lav og det er flatt.
        if (up > 0.95 && world.y < 0.3) {
            let basin = smoothstep(0.45, 0.25, fbm2(world.xz * 0.35 + 7.0)) * smoothstep(0.6, 0.3, height);
            let puddle = basin * smoothstep(0.3, 0.8, wet);
            s.n = normalize(mix(s.n, vec3<f32>(0.0, 1.0, 0.0), puddle));
            s.rough = mix(s.rough, 0.03, puddle);
            s.albedo *= 1.0 - 0.5 * puddle;
            // Ringer fra regndråper.
            if (g.weather.z > 0.0 && puddle > 0.5) {
                let t = g.camera.w;
                let cell = floor(world.xz * 2.0);
                let h = hash21(cell);
                let age = fract(t * 1.5 + h * 7.0);
                let c = cell / 2.0 + 0.25 + (vec2<f32>(hash21(cell + 1.0), hash21(cell + 2.0)) - 0.5) * 0.4;
                let rr = length(world.xz - c);
                let ring = smoothstep(0.02, 0.0, abs(rr - age * 0.35)) * (1.0 - age);
                s.n = normalize(s.n + vec3<f32>(world.x - c.x, 0.0, world.z - c.y) * ring * 1.5);
            }
        }
    }
    let snow = g.weather.y;
    if (snow > 0.0) {
        let n_up = clamp(s.n.y, 0.0, 1.0);
        let edge = fbm2(world.xz * 2.0 + world.y) * 0.5;
        let cover = smoothstep(0.35 + edge * 0.5, 0.75 + edge * 0.3, n_up * (0.6 + height * 0.6)) * snow;
        let sparkle = pow(fbm2(world.xz * 40.0 + world.y * 20.0), 6.0) * 2.0;
        s.albedo = mix(s.albedo, vec3<f32>(0.9, 0.93, 0.97) * (0.9 + sparkle), cover);
        s.rough = mix(s.rough, 0.75, cover);
        s.n = normalize(mix(s.n, vec3<f32>(0.0, 1.0, 0.0), cover * 0.6));
        s.metal *= 1.0 - cover;
    }
    return s;
}

@fragment
fn fs(i: VsOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let kind = i.mat.x;
    let slot = i.mat.y;
    let flags = i.mat.z;
    // Gradientene tas her, i uniform kontrollflyt, og brukes i alle oppslag.
    let dp1 = dpdx(i.world);
    let dp2 = dpdy(i.world);
    let duv1 = dpdx(i.uv);
    let duv2 = dpdy(i.uv);
    let glow = flags & 3u;
    var n_geo = normalize(i.normal);
    // Alfakort (blad, gress) har normalen ut eller opp på begge sider.
    if (!front && (flags & F_ALPHA) == 0u) {
        n_geo = -n_geo;
    }
    let night = g.sun_color.w;
    let v = normalize(g.camera.xyz - i.world);

    var tint = srgb_to_linear(i.color.rgb);
    if (slot > 0u) {
        tint = g.palette[slot].rgb;
    }

    var s: Surf;
    s.albedo = tint;
    s.rough = i.color.a;
    s.metal = 0.0;
    s.n = n_geo;
    s.cavity = 1.0;
    s.emissive = vec3<f32>(0.0);
    s.kind = 0u;
    s.ao = i.ao;
    var height = 0.5;

    if (kind < 32u) {
        var t: Tex;
        if ((flags & F_UVMAP) != 0u) {
            t = uvmapped(i32(kind), i.uv, duv1, duv2, dp1, dp2, n_geo);
        } else {
            t = triplanar(i32(kind), i.world, dp1, dp2, n_geo, (flags & F_ROT_UV) != 0u);
            // Berget i fjellene: på avstand tar en flis som er ti ganger
            // større over, så de store flatene ikke gjentar seg i ruter.
            if (kind == 15u) {
                let far = smoothstep(40.0, 160.0, distance(i.world, g.camera.xyz));
                if (far > 0.01) {
                    let big = triplanar_s(15, i.world, dp1, dp2, n_geo, false, 10.0);
                    t.albedo = mix(t.albedo, big.albedo, far);
                    t.n = normalize(mix(t.n, big.n, far));
                    t.rough = mix(t.rough, big.rough, far);
                    t.height = mix(t.height, big.height, far);
                }
            }
        }
        // Stor, svak variasjon så flisene ikke repeterer synlig.
        let variation = 0.86 + 0.28 * fbm2(i.world.xz * 0.11 + i.world.y * 0.07);
        s.albedo = t.albedo * tint * variation;
        s.n = t.n;
        s.rough = clamp(t.rough * i.color.a * 2.0, 0.02, 1.0);
        height = t.height;
        // Fugene og sprekkene er mørkere, lyset når ikke ned.
        s.cavity = mix(0.6, 1.0, smoothstep(0.0, 0.5, height));
        // Skitt og fukt nederst på veggene.
        if ((flags & F_NO_DIRT) == 0u && abs(n_geo.y) < 0.5) {
            let dirt = smoothstep(1.2, 0.0, i.world.y) * (0.6 + fbm2(i.world.xz * 2.0 + i.world.y) * 0.4);
            s.albedo *= 1.0 - dirt * 0.3;
            s.rough = mix(s.rough, 0.9, dirt * 0.3);
        }
        if (kind == 8u) {
            s.metal = 0.6;
        }
    } else if (kind == K_TERRAIN) {
        // Okklusjonsbyten bærer andelen tråkket jord på bakken.
        let t = terrain(i.color, i.dw.x, i.dw.y, i.world, dp1, dp2, n_geo, i.uv, duv1, duv2, i.ao);
        s.ao = 1.0;
        s.albedo = t.albedo * (0.9 + 0.2 * fbm2(i.world.xz * 0.11));
        s.n = t.n;
        s.rough = clamp(t.rough, 0.04, 1.0);
        height = t.height;
        s.cavity = mix(0.65, 1.0, smoothstep(0.0, 0.5, height));
    } else if (kind == K_MODEL) {
        let a = textureSampleGrad(model_albedo, tex_sampler, i.uv, duv1, duv2);
        if ((flags & F_ALPHA) != 0u && coverage(kind, flags, i.uv, duv1, duv2) < 0.5) {
            discard;
        }
        var paint = vec3<f32>(1.0);
        if ((flags & F_PAINT) != 0u) {
            // Lakkfargen ligger pakket i instansen, 8 bit per kanal.
            let p = u32(i.extra.y);
            paint = vec3<f32>(f32((p >> 16u) & 255u), f32((p >> 8u) & 255u), f32(p & 255u)) / 255.0;
        }
        s.albedo = a.rgb * tint * paint;
        let mr = textureSampleGrad(model_mr, tex_sampler, i.uv, duv1, duv2);
        // Ruheten per materiale ligger i hjørnefargens alfa; metall er et flagg.
        s.rough = clamp(mr.g * i.color.a, 0.03, 1.0);
        s.metal = mr.b * select(0.0, 1.0, (flags & F_METAL) != 0u);
        // Normal map med tangent fra skjermderivatene.
        let nm = textureSampleGrad(model_normal, tex_sampler, i.uv, duv1, duv2).xyz * 2.0 - 1.0;
        let dp2perp = cross(dp2, n_geo);
        let dp1perp = cross(n_geo, dp1);
        let t = dp2perp * duv1.x + dp1perp * duv2.x;
        let b = dp2perp * duv1.y + dp1perp * duv2.y;
        let invmax = inverseSqrt(max(dot(t, t), dot(b, b)) + 1e-8);
        let tbn = mat3x3<f32>(t * invmax, b * invmax, n_geo);
        if (dot(t, t) > 1e-8) {
            s.n = normalize(tbn * nm);
        }
        s.emissive = textureSampleGrad(model_emissive, tex_sampler, i.uv, duv1, duv2).rgb * (0.2 + night * 8.0 * g.params.w);
        if ((flags & F_ALPHA) != 0u) {
            // Alfakort (barnåler, gress, blader i modellene) er løv: matte,
            // uten himmelspeiling i flate vinkler, og med lys gjennom.
            s.kind = 1u;
            s.rough = max(s.rough, 0.85);
        }
    } else if (kind == K_GLASS) {
        s.kind = 2u;
        s.albedo = vec3<f32>(0.0);
        s.rough = 0.04;
        if (glow == 3u) {
            // Innsiden av en vindusrute: dagslyset slipper gjennom grønnlig,
            // ujevnt blåst glass, med bobler og lysere midt i hver rute.
            let sky_amb = mix(g.sky_horizon.rgb, g.sky_top.rgb, 0.35);
            let q = vec2<f32>(dot(i.world.xz, vec2<f32>(0.7071, -0.7071)), i.world.y);
            let wobble = 0.55 + 0.6 * fbm2(q * 7.0);
            // Hver rute er blåst for seg: litt ulik tykkelse og farge.
            let pane = hash31(vec3<f32>(floor(q * 3.8), 7.0));
            let tone = mix(vec3<f32>(0.62, 0.78, 0.64), vec3<f32>(0.8, 0.84, 0.7), pane);
            s.rough = 0.35;
            s.emissive = tone * sky_amb * wobble * (0.9 + pane * 0.5);
        } else if (glow == 2u) {
            // Utsida av en rute i et hus man kan gå inn i: lys, ild og
            // lysekroner inne skinner varmt gjennom det ujevne glasset om
            // natta, og blafrer litt.
            let q = vec2<f32>(dot(i.world.xz, vec2<f32>(0.7071, -0.7071)), i.world.y);
            let pane = hash31(vec3<f32>(floor(q * 3.8), 3.0));
            let wobble = 0.6 + 0.5 * fbm2(q * 7.0);
            let flick = 0.9 + 0.1 * sin(g.camera.w * 6.0 + pane * 17.0) * sin(g.camera.w * 2.3 + pane * 5.0);
            let warm = mix(vec3<f32>(1.0, 0.55, 0.22), vec3<f32>(1.0, 0.72, 0.4), pane);
            s.rough = 0.2;
            s.emissive = warm * (0.04 + night * 2.2) * wobble * flick;
        } else {
            s.emissive = interior(i.world, n_geo, v, night);
        }
        if (glow != 3u) {
            // Om dagen utenfra: det grønnlige, ujevne glasset speiler himmelen
            // (mer på skrå) og slipper litt av det lyse rommet bak gjennom,
            // så rutene ikke blir svarte hull.
            let day = 1.0 - night;
            let q = vec2<f32>(dot(i.world.xz, vec2<f32>(0.7071, -0.7071)), i.world.y);
            let pane = hash31(vec3<f32>(floor(q * 3.8), 5.0));
            let wobble = 0.6 + 0.5 * fbm2(q * 7.0);
            let sky = mix(g.sky_horizon.rgb, g.sky_top.rgb, 0.5);
            let fres = 0.25 + 0.75 * pow(1.0 - max(dot(n_geo, v), 0.0), 3.0);
            let glass = mix(vec3<f32>(0.55, 0.7, 0.58), vec3<f32>(0.75, 0.8, 0.66), pane);
            s.emissive += day * (sky * fres * 0.55 + glass * sky * 0.18) * wobble;
        }
    } else if (kind == K_LEAF) {
        let l = textureSampleGrad(leaf_tex, clamp_sampler, i.uv, duv1, duv2);
        if (l.a < 0.5) {
            discard;
        }
        s.kind = 1u;
        s.albedo = l.rgb * tint;
        s.rough = 0.7;
        s.ao = 0.85;
    } else if (kind == K_LAMP) {
        s.rough = 0.3;
        // Lamper tennes om natta; ild (glød 3) brenner hele døgnet.
        let lit = select(0.15 + night * 10.0 * g.params.w, 6.0, glow == 3u);
        s.emissive = tint * vec3<f32>(1.0, 0.8, 0.55) * lit;
    } else if (kind == K_MAGIC) {
        // Portalen: lyser i sin egen farge, uten ildens varme filter.
        s.rough = 0.3;
        s.albedo = tint * 0.3;
        s.emissive = tint * 2.4;
    } else if (kind == K_SKELETON) {
        // Skimmeret i .fs-skeleton: et lyst bånd som glir over.
        let band = fract((i.world.x + i.world.y * 0.5 - g.camera.w * 1.2) / 6.0);
        let shimmer = smoothstep(0.3, 0.5, band) * smoothstep(0.7, 0.5, band);
        s.albedo = tint * (1.0 + shimmer * 0.35);
        s.rough = 0.8;
        s.emissive = tint * shimmer * (0.05 + night * 0.4);
    } else if (kind == K_WATER) {
        s.albedo = vec3<f32>(0.02, 0.04, 0.05);
        s.rough = 0.02;
        let t = g.camera.w;
        let ripple = vec3<f32>(fbm2(i.world.xz * 6.0 + t * 0.5) - 0.5, 0.0, fbm2(i.world.zx * 6.0 - t * 0.4) - 0.5);
        s.n = normalize(n_geo + ripple * 0.15);
    }

    if (glow == 3u) {
        s.emissive += s.albedo * (0.05 + night * 0.35);
    } else if (glow == 2u && kind != K_GLASS) {
        s.emissive += tint * vec3<f32>(1.0, 0.78, 0.5) * (0.2 + night * 9.0 * g.params.w);
    }

    s = weather(s, i.world, n_geo, height, flags);
    let dbg = i32(g.sky.w);
    if (dbg > 0) {
        let screen_uv = i.clip.xy * g.screen.zw;
        switch dbg {
            case 1: { return vec4<f32>(s.albedo, 1.0); }
            case 2: { return vec4<f32>(s.n * 0.5 + 0.5, 1.0); }
            case 3: { return vec4<f32>(vec3<f32>(textureSampleLevel(ao_tex, frame_sampler, screen_uv, 0.0).r * s.ao), 1.0); }
            case 4: { return vec4<f32>(vec3<f32>(s.rough), 1.0); }
            case 5: { return vec4<f32>(vec3<f32>(sun_shadow(i.world, n_geo, i.world.xz)), 1.0); }
            case 6: { return vec4<f32>(shade(i, s, n_geo) * 0.4, 1.0); }
            case 7: {
                // Bare sol-diffus.
                let l = g.sun_dir.xyz;
                return vec4<f32>(g.sun_color.rgb * g.sun_dir.w * max(dot(s.n, l), 0.0) * s.albedo * 0.4, 1.0);
            }
            case 8: {
                // Bare refleksjon av himmelen.
                let v = normalize(g.camera.xyz - i.world);
                let r = reflect(-v, s.n);
                let f0 = vec3<f32>(0.04);
                let fres = f0 + (max(vec3<f32>(1.0 - s.rough), f0) - f0) * pow(1.0 - max(dot(s.n, v), 0.0), 5.0);
                return vec4<f32>(sky_gradient(r) * fres * 3.0, 1.0);
            }
            case 10: {
                let screen_uv2 = i.clip.xy * g.screen.zw;
                let ssao = textureSampleLevel(ao_tex, frame_sampler, screen_uv2, 0.0).r;
                let ao = s.ao * s.cavity * ssao;
                let sky_amb = mix(g.sky_top.rgb, g.sky_horizon.rgb, 0.6) * 1.25;
                let hemi = mix(g.ground_bounce.rgb, sky_amb, s.n.y * 0.5 + 0.5);
                return vec4<f32>(s.albedo * hemi * ao, 1.0);
            }
            case 11: {
                let v = normalize(g.camera.xyz - i.world);
                let f0 = mix(vec3<f32>(0.04), s.albedo, s.metal);
                let r = reflect(-v, s.n);
                let fres = f0 + (max(vec3<f32>(1.0 - s.rough), f0) - f0) * pow(1.0 - max(dot(s.n, v), 0.0), 5.0);
                let r_blur = normalize(mix(r, s.n, s.rough * s.rough));
                let hemi = mix(g.ground_bounce.rgb, g.sky_top.rgb, s.n.y * 0.5 + 0.5);
                let env = mix(sky_gradient(r_blur), hemi, s.rough * 0.7) * 0.8;
                return vec4<f32>(env * fres, 1.0);
            }
            case 12: {
                let v = normalize(g.camera.xyz - i.world);
                let f0 = mix(vec3<f32>(0.04), s.albedo, s.metal);
                let l = g.sun_dir.xyz;
                let ndl = max(dot(s.n, l), 0.0);
                return vec4<f32>(g.sun_color.rgb * g.sun_dir.w * (s.albedo * ndl + ggx(s.n, v, l, s.rough, f0) * PI * ndl) * 0.4, 1.0);
            }
            case 13: { return vec4<f32>(vec3<f32>(0.18), 1.0); }
            default: { return vec4<f32>(s.emissive, 1.0); }
        }
    }
    var color = shade(i, s, n_geo);
    if (kind == K_SKELETON && i.extra.y > 0.0) {
        color += tint * i.extra.y;
    }
    return vec4<f32>(color, 1.0);
}
