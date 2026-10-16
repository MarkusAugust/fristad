// Etterbehandling: glød og tonemapping. Begge tegner en trekant over hele
// skjermen.

struct Post {
    params: vec4<f32>,
    // Størrelsen på kilden: bredde, høyde, 1/bredde, 1/høyde.
    texel: vec4<f32>,
    // x: hvor malt bildet er i filmen (0..1).
    extra: vec4<f32>,
}
@group(1) @binding(0) var tex0: texture_2d<f32>;
@group(1) @binding(1) var tex1: texture_2d<f32>;
@group(1) @binding(2) var lin: sampler;
@group(1) @binding(3) var<uniform> post: Post;

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

// Bildet tonemappet og fargegradert i ett punkt (uten vignett og flash).
fn graded(uv: vec2<f32>, exposure: f32, bloom: f32) -> vec3<f32> {
    var hdr = textureSampleLevel(tex0, lin, uv, 0.0).rgb;
    hdr += textureSampleLevel(tex1, lin, uv, 0.0).rgb * bloom;
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

// Filmens malte bilde: et Kuwahara-filter med åtte myke sektorer
// (Kyprianidis' polynomvekter) legger flatene i strøk med samme farge og
// holder kantene skarpe, som pensel og ikke uskarphet. Radien går med
// oppløsningen, så strøkene er like store på alle skjermer.
fn painted(uv: vec2<f32>, exposure: f32, bloom: f32) -> vec3<f32> {
    let radius = post.texel.y / 120.0;
    let n = 6;
    let step = radius / f32(n);
    let zeta = 2.0 / f32(n);
    let sc = sin(0.58);
    let eta = (zeta + cos(0.58)) / (sc * sc);
    var m: array<vec4<f32>, 8>;
    var s: array<vec3<f32>, 8>;
    for (var k = 0; k < 8; k++) {
        m[k] = vec4<f32>(0.0);
        s[k] = vec3<f32>(0.0);
    }
    for (var y = -n; y <= n; y++) {
        for (var x = -n; x <= n; x++) {
            var v = vec2<f32>(f32(x), f32(y)) / f32(n);
            if (dot(v, v) > 1.0) {
                continue;
            }
            let c = graded(uv + vec2<f32>(f32(x), f32(y)) * step * post.texel.zw, exposure, bloom);
            var w: array<f32, 8>;
            var vxx = zeta - eta * v.x * v.x;
            var vyy = zeta - eta * v.y * v.y;
            var z = max(0.0, v.y + vxx);
            w[0] = z * z;
            z = max(0.0, -v.x + vyy);
            w[2] = z * z;
            z = max(0.0, -v.y + vxx);
            w[4] = z * z;
            z = max(0.0, v.x + vyy);
            w[6] = z * z;
            let d = 0.70710678 * vec2<f32>(v.x - v.y, v.x + v.y);
            vxx = zeta - eta * d.x * d.x;
            vyy = zeta - eta * d.y * d.y;
            z = max(0.0, d.y + vxx);
            w[1] = z * z;
            z = max(0.0, -d.x + vyy);
            w[3] = z * z;
            z = max(0.0, -d.y + vxx);
            w[5] = z * z;
            z = max(0.0, d.x + vyy);
            w[7] = z * z;
            var sum = 0.0;
            for (var k = 0; k < 8; k++) {
                sum += w[k];
            }
            let gauss = exp(-3.125 * dot(v, v)) / max(sum, 1e-6);
            for (var k = 0; k < 8; k++) {
                let wk = w[k] * gauss;
                m[k] += vec4<f32>(c * wk, wk);
                s[k] += c * c * wk;
            }
        }
    }
    var acc = vec4<f32>(0.0);
    for (var k = 0; k < 8; k++) {
        let mean = m[k].rgb / max(m[k].w, 1e-6);
        let var3 = abs(s[k] / max(m[k].w, 1e-6) - mean * mean);
        let sigma = var3.r + var3.g + var3.b;
        let a = 1.0 / (1.0 + pow(sigma * 1000.0, 4.0));
        acc += vec4<f32>(mean * a, a);
    }
    return acc.rgb / max(acc.w, 1e-6);
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
    let paint = post.extra.x;
    if (paint > 0.0) {
        // Malt: strøkene, litt mer mettet og varmere, og mørkere kanter.
        var p = painted(i.uv, exposure, bloom);
        let lum = dot(p, vec3<f32>(0.2126, 0.7152, 0.0722));
        p = max(mix(vec3<f32>(lum), p, 1.12), vec3<f32>(0.0));
        p *= mix(vec3<f32>(0.97, 0.99, 1.06), vec3<f32>(1.06, 1.0, 0.9), smoothstep(0.1, 0.7, lum));
        p *= 1.0 - dot(v, v) * 0.16;
        c = mix(c, p, paint);
    }
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
