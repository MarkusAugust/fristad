// Etterbehandling: glød og tonemapping. Begge tegner en trekant over hele
// skjermen.

struct Post {
    params: vec4<f32>,
    // Størrelsen på kilden: bredde, høyde, 1/bredde, 1/høyde.
    texel: vec4<f32>,
    // Tonemappingen: xy sola på skjermen (uv), z styrken på solstrålene,
    // w den malte graderingen (1) eller den gamle ACES-kurven (0).
    sun: vec4<f32>,
}
@group(1) @binding(0) var tex0: texture_2d<f32>;
@group(1) @binding(1) var tex1: texture_2d<f32>;
@group(1) @binding(2) var lin: sampler;
@group(1) @binding(3) var<uniform> post: Post;
// Skygge i krokene i halv oppløsning (hvit når den er av).
@group(1) @binding(4) var tex_ao: texture_2d<f32>;

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

// ---- Glød. ----

fn karis(c: vec3<f32>) -> f32 {
    return 1.0 / (1.0 + max(max(c.r, c.g), c.b));
}

// Byttes et ugyldig tall (NaN eller uendelig) ut med 0. Ett eneste slikt i
// bildet ville ellers spredd seg gjennom gløden til en svart firkant.
fn finite(c: vec3<f32>) -> vec3<f32> {
    let bad = (bitcast<vec3<u32>>(c) & vec3<u32>(0x7f800000u)) == vec3<u32>(0x7f800000u);
    return select(c, vec3<f32>(0.0), bad);
}

@fragment
fn fs_down(i: VsOut) -> @location(0) vec4<f32> {
    let t = post.texel.zw;
    var c = vec3<f32>(0.0);
    let offs = array<vec2<f32>, 13>(
        vec2<f32>(-2.0, -2.0), vec2<f32>(0.0, -2.0), vec2<f32>(2.0, -2.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0),
        vec2<f32>(-2.0, 0.0), vec2<f32>(0.0, 0.0), vec2<f32>(2.0, 0.0),
        vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-2.0, 2.0), vec2<f32>(0.0, 2.0), vec2<f32>(2.0, 2.0),
    );
    let w = array<f32, 13>(0.03125, 0.0625, 0.03125, 0.125, 0.125, 0.0625, 0.125, 0.0625, 0.125, 0.125, 0.03125, 0.0625, 0.03125);
    let first = post.params.w > 0.5;
    for (var k = 0; k < 13; k++) {
        var s = textureSampleLevel(tex0, lin, i.uv + offs[k] * t, 0.0).rgb;
        if (first) {
            s = finite(s);
            // Bare det som er lysere enn hvitt gløder, dempet mot flimmer.
            s = max(s - vec3<f32>(1.0), vec3<f32>(0.0)) * karis(s);
        }
        c += s * w[k];
    }
    return vec4<f32>(c, 1.0);
}

@fragment
fn fs_up(i: VsOut) -> @location(0) vec4<f32> {
    let t = post.texel.zw;
    var c = vec3<f32>(0.0);
    c += textureSampleLevel(tex0, lin, i.uv + vec2<f32>(-1.0, -1.0) * t, 0.0).rgb * 1.0;
    c += textureSampleLevel(tex0, lin, i.uv + vec2<f32>(0.0, -1.0) * t, 0.0).rgb * 2.0;
    c += textureSampleLevel(tex0, lin, i.uv + vec2<f32>(1.0, -1.0) * t, 0.0).rgb * 1.0;
    c += textureSampleLevel(tex0, lin, i.uv + vec2<f32>(-1.0, 0.0) * t, 0.0).rgb * 2.0;
    c += textureSampleLevel(tex0, lin, i.uv, 0.0).rgb * 4.0;
    c += textureSampleLevel(tex0, lin, i.uv + vec2<f32>(1.0, 0.0) * t, 0.0).rgb * 2.0;
    c += textureSampleLevel(tex0, lin, i.uv + vec2<f32>(-1.0, 1.0) * t, 0.0).rgb * 1.0;
    c += textureSampleLevel(tex0, lin, i.uv + vec2<f32>(0.0, 1.0) * t, 0.0).rgb * 2.0;
    c += textureSampleLevel(tex0, lin, i.uv + vec2<f32>(1.0, 1.0) * t, 0.0).rgb * 1.0;
    return vec4<f32>(c / 16.0, 1.0);
}

// ---- Tonemapping. tex0 er bildet, tex1 gløden. ----

fn aces(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// Solstråler gjennom lufta: himmelen (alfa 0 i HDR-bildet) nær sola
// samles langs linja fra punktet mot sola, så husene, trærne og murene
// skjærer lyset i stråler. Faste steg uten støy; kort rekkevidde, så
// trinnene ikke synes.
fn sun_rays(uv: vec2<f32>) -> vec3<f32> {
    let sun = post.sun.xy;
    let aspect = post.texel.x * post.texel.w;
    let step = (sun - uv) * (0.7 / 24.0);
    var p = uv;
    var acc = vec3<f32>(0.0);
    var w = 1.0;
    for (var k = 0; k < 24; k++) {
        p += step;
        let t = textureSampleLevel(tex0, lin, p, 0.0);
        let r = (p - sun) * vec2<f32>(aspect, 1.0);
        let near = exp(-dot(r, r) * 14.0);
        acc += min(t.rgb, vec3<f32>(6.0)) * (1.0 - t.a) * near * w;
        w *= 0.95;
    }
    return acc * (post.sun.z * 0.04);
}

// Den malte tonemappingen: ACES-kurven på lysstyrken med fargene beholdt
// (lys ild blir ikke gul, himmelen ikke cyan), og det som da går over hvitt
// glir mykt mot hvitt som på lerret; blandet halvt med ACES per kanal.
fn painted(x: vec3<f32>) -> vec3<f32> {
    let l = luma(x);
    let mapped = aces(vec3<f32>(l)).r;
    var kept = x * (mapped / max(l, 1e-5));
    let over = max(max(kept.r, kept.g), kept.b);
    if (over > 1.0) {
        kept = mix(kept / over, vec3<f32>(1.0), clamp((over - 1.0) * 0.5, 0.0, 1.0));
    }
    return mix(kept, aces(x), 0.5);
}

// Bildet tonemappet og fargegradert i ett punkt (uten vignett og flash).
fn graded(uv: vec2<f32>, exposure: f32, bloom: f32) -> vec3<f32> {
    var hdr = textureSampleLevel(tex0, lin, uv, 0.0).rgb * textureSampleLevel(tex_ao, lin, uv, 0.0).r;
    hdr += textureSampleLevel(tex1, lin, uv, 0.0).rgb * bloom;
    if (post.sun.z > 0.0) {
        hdr += sun_rays(uv);
    }
    if (post.sun.w > 0.5) {
        return paint_grade(painted(hdr * exposure));
    }
    var c = aces(hdr * exposure);
    // Fargegradering for den malte stilen: mettede farger, en mild
    // S-kurve, varme lys og litt kjøligere skygger.
    var lum = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
    c = max(mix(vec3<f32>(lum), c, 1.22), vec3<f32>(0.0));
    c = mix(c, c * c * (3.0 - 2.0 * c), 0.2);
    lum = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
    c = mix(c, c * vec3<f32>(0.94, 0.98, 1.08), smoothstep(0.4, 0.0, lum) * 0.5);
    c = mix(c, c * vec3<f32>(1.05, 1.0, 0.9), smoothstep(0.45, 1.0, lum) * 0.5);
    return c;
}

// Fargegraderingen i den malte stilen etter stedet og tida: varm by om
// dagen med gyldne lys og blålige skygger, blå måneskinnsnatt der lyktene
// og vinduene står varme, og kald kjeller (grønnblå skygger) der faklene
// og lykta er det eneste varme.
fn paint_grade(c_in: vec3<f32>) -> vec3<f32> {
    var c = c_in;
    let night = clamp(g.sun_color.w, 0.0, 1.0);
    let cellar = g.room_info.x > 0.0;
    var lum = luma(c);
    // Metning og en mild S-kurve, som før.
    c = max(mix(vec3<f32>(lum), c, select(1.22, 1.15, cellar)), vec3<f32>(0.0));
    c = mix(c, c * c * (3.0 - 2.0 * c), 0.2);
    lum = luma(c);
    let shadows = smoothstep(0.42, 0.0, lum);
    let lights = smoothstep(0.4, 1.0, lum);
    var cold = vec3<f32>(0.94, 0.98, 1.08);
    var warm = vec3<f32>(1.06, 1.0, 0.88);
    var cold_k = 0.5;
    if (cellar) {
        cold = vec3<f32>(0.84, 0.95, 1.14);
        warm = vec3<f32>(1.06, 0.99, 0.88);
        cold_k = 0.7;
    } else {
        cold = mix(cold, vec3<f32>(0.9, 0.96, 1.1), night);
        cold_k = mix(0.5, 0.6, night);
    }
    c = mix(c, c * cold, shadows * cold_k);
    // Det varme går bare der fargen allerede er varm (ild, lys, sol), så
    // en blå himmel ikke blir grå.
    let warmth = smoothstep(-0.05, 0.15, c.r - c.b);
    c = mix(c, c * warm, lights * mix(0.35, 0.6, warmth));
    // Litt løftede, aldri helt svarte skygger, som maling på lerret.
    c = c + vec3<f32>(0.0012, 0.0012, 0.0018) * (1.0 - smoothstep(0.0, 0.02, lum));
    return c;
}

@fragment
fn fs_tonemap(i: VsOut) -> @location(0) vec4<f32> {
    let exposure = post.params.x;
    let bloom = post.params.y;
    if (g.sky.w > 0.0) {
        var d = clamp(textureSampleLevel(tex0, lin, i.uv, 0.0).rgb, vec3<f32>(0.0), vec3<f32>(1.0));
        if (post.params.z > 0.5) {
            d = linear_to_srgb(d);
        }
        return vec4<f32>(d, 1.0);
    }
    var c = graded(i.uv, exposure, bloom);
    let v = i.uv * 2.0 - 1.0;
    // Vignett.
    c *= 1.0 - dot(v, v) * 0.06;
    // Rødt flash langs kantene når spilleren er truffet.
    let hurt = post.params.w;
    if (hurt > 0.0) {
        c = mix(c, vec3<f32>(0.55, 0.02, 0.0), hurt * smoothstep(0.78, 1.45, length(v)) * 0.85);
    }
    c = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    if (post.params.z > 0.5) {
        c = linear_to_srgb(c);
    }
    return vec4<f32>(c, 1.0);
}
