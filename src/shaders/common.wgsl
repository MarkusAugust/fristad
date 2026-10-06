// Felles for alle shaderne. Må stemme med `Globals` i render.rs.

struct Light {
    pos_radius: vec4<f32>,
    color_intensity: vec4<f32>,
}

struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    sun_view_proj: mat4x4<f32>,
    // xyz kamera, w tid i sekunder.
    camera: vec4<f32>,
    // xyz retning mot sola (eller månen), w styrke.
    sun_dir: vec4<f32>,
    // rgb, w natt fra 0 til 1.
    sun_color: vec4<f32>,
    sky_top: vec4<f32>,
    // rgb, w tåketetthet.
    sky_horizon: vec4<f32>,
    ground_bounce: vec4<f32>,
    // rgb, w hvor fort tåka tynnes ut med høyden.
    fog_color: vec4<f32>,
    // x eksponering, y antall lys, z en skyggetexel, w andel tente vinduer.
    params: vec4<f32>,
    palette: array<vec4<f32>, 16>,
    lights: array<Light, 32>,
}

@group(0) @binding(0) var<uniform> g: Globals;

const PI: f32 = 3.14159265;

fn hash21(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 456.21));
    q += dot(q, q + 45.32);
    return fract(q.x * q.y);
}

fn hash31(p: vec3<f32>) -> f32 {
    var q = fract(p * 0.1031);
    q += dot(q, q.zyx + 31.32);
    return fract((q.x + q.y) * q.z);
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm2(p: vec2<f32>) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var q = p;
    for (var i = 0; i < 4; i++) {
        v += a * noise2(q);
        q = q * 2.03 + vec2<f32>(1.7, 9.2);
        a *= 0.5;
    }
    return v;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

// Himmelfargen i en retning, uten sol og stjerner. Brukes både av himmelen og
// til refleksjoner og tåke.
fn sky_gradient(dir: vec3<f32>) -> vec3<f32> {
    let h = clamp(dir.y, -1.0, 1.0);
    let up = pow(clamp(h, 0.0, 1.0), 0.45);
    var c = mix(g.sky_horizon.rgb, g.sky_top.rgb, up);
    // Under horisonten: mørkere, mot bakkefargen.
    c = mix(c, g.ground_bounce.rgb * 0.6 + g.sky_horizon.rgb * 0.4, smoothstep(0.0, -0.25, h));
    // Lys rundt sola.
    let s = max(dot(dir, g.sun_dir.xyz), 0.0);
    c += g.sun_color.rgb * (pow(s, 6.0) * 0.25 + pow(s, 64.0) * 0.6) * (1.0 - g.sun_color.w * 0.8);
    return c;
}

fn fog(color: vec3<f32>, world: vec3<f32>) -> vec3<f32> {
    let v = world - g.camera.xyz;
    let dist = length(v);
    let dir = v / max(dist, 1e-4);
    let density = g.sky_horizon.w;
    let falloff = g.fog_color.w;
    // Høydetåke: tettere nede, integrert langs strålen.
    let y0 = g.camera.y;
    let dy = v.y;
    var height_term = exp(-falloff * y0);
    if (abs(dy) > 0.01) {
        height_term = (exp(-falloff * y0) - exp(-falloff * (y0 + dy))) / (falloff * dy);
    }
    let amount = 1.0 - exp(-density * dist * height_term);
    let sun = pow(max(dot(dir, g.sun_dir.xyz), 0.0), 8.0);
    let fc = mix(g.fog_color.rgb, sky_gradient(normalize(vec3<f32>(dir.x, 0.05, dir.z))), 0.5)
        + g.sun_color.rgb * sun * 0.35 * (1.0 - g.sun_color.w);
    return mix(color, fc, clamp(amount, 0.0, 1.0));
}
