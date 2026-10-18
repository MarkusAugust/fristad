// Skyggen fra sola: ett skyggekart rundt kameraet ut til `g.misc.w`, som
// toner bort fra `g.misc.z`, og om spilleren vil en kaskade til ut til
// `g.shadow.w` (grovere, side om side i samme kart). Prøvene står fast (ingen
// tilfeldig dreiing), så kantene er rolige uten støy: fire (2 × 2 texels)
// eller tolv i en poissonskive, hver bilineært sammenlignet, til myke kanter.

@group(1) @binding(0) var shadow_map: texture_depth_2d;
@group(1) @binding(1) var shadow_sampler: sampler_comparison;
// Skyggepasset: x er kaskaden som tegnes (0 eller 1).
@group(1) @binding(2) var<uniform> shadow_pass: vec4<f32>;

// Punktet i skyggekartets rom: uv i kaskaden og dybden.
fn shadow_coord(vp: mat4x4<f32>, p: vec3<f32>) -> vec3<f32> {
    let c = vp * vec4<f32>(p, 1.0);
    let q = c.xyz / c.w;
    return vec3<f32>(q.xy * vec2<f32>(0.5, -0.5) + 0.5, q.z);
}

fn in_cascade(c: vec3<f32>) -> bool {
    return all(c.xy > vec2<f32>(0.0)) && all(c.xy < vec2<f32>(1.0)) && c.z <= 1.0;
}

// Lys (1) eller skygge (0) i kaskade `k`, filtrert etter innstillingen.
fn shadow_taps(c: vec3<f32>, k: f32, bias: f32) -> f32 {
    let n = max(g.shadow.z, 1.0);
    let t = g.shadow.xy;
    let lo = k / n + t.x;
    let hi = (k + 1.0) / n - t.x;
    let base = vec2<f32>((k + c.x) / n, c.y);
    let z = c.z - bias;
    if (g.quality.z > 1.5) {
        let poisson = array<vec2<f32>, 12>(
            vec2<f32>(-0.326, -0.406), vec2<f32>(-0.840, -0.074), vec2<f32>(-0.696, 0.457),
            vec2<f32>(-0.203, 0.621), vec2<f32>(0.962, -0.195), vec2<f32>(0.473, -0.480),
            vec2<f32>(0.519, 0.767), vec2<f32>(0.185, -0.893), vec2<f32>(0.507, 0.064),
            vec2<f32>(0.896, 0.412), vec2<f32>(-0.322, -0.933), vec2<f32>(-0.792, -0.598),
        );
        var s = 0.0;
        for (var i = 0; i < 12; i++) {
            let o = base + poisson[i] * t * 1.8;
            s += textureSampleCompareLevel(shadow_map, shadow_sampler, vec2<f32>(clamp(o.x, lo, hi), o.y), z);
        }
        return s / 12.0;
    }
    let h = t * 0.5;
    var s = textureSampleCompareLevel(shadow_map, shadow_sampler, base + vec2<f32>(-h.x, -h.y), z);
    s += textureSampleCompareLevel(shadow_map, shadow_sampler, base + vec2<f32>(h.x, -h.y), z);
    s += textureSampleCompareLevel(shadow_map, shadow_sampler, base + vec2<f32>(-h.x, h.y), z);
    s += textureSampleCompareLevel(shadow_map, shadow_sampler, base + vec2<f32>(h.x, h.y), z);
    return s * 0.25;
}

fn sun_shadow(world: vec3<f32>, n: vec3<f32>) -> f32 {
    if (g.quality.z < 0.5) {
        return 1.0;
    }
    let d = distance(world, g.camera.xyz);
    let two = g.shadow.z > 1.5;
    // Punktet flyttes litt ut langs normalen (mer på skrå mot sola), så
    // flatene ikke skygger for seg selv; mer i den grove kaskaden.
    let ndl = clamp(dot(n, g.sun_dir.xyz), 0.0, 1.0);
    let lean = 1.0 + 2.0 * (1.0 - ndl);
    var far_s = 1.0;
    if (two && d > g.misc.z) {
        let c2 = shadow_coord(g.sun_vp2, world + n * 0.08 * lean);
        if (in_cascade(c2)) {
            far_s = mix(shadow_taps(c2, 1.0, 0.0007), 1.0, smoothstep(g.shadow.w * 0.82, g.shadow.w, d));
        }
    }
    let fade = smoothstep(g.misc.z, g.misc.w, d);
    if (fade >= 1.0) {
        return select(1.0, far_s, two);
    }
    let c = shadow_coord(g.sun_vp, world + n * 0.03 * lean);
    if (!in_cascade(c)) {
        return select(1.0, far_s, two);
    }
    return mix(shadow_taps(c, 0.0, 0.0005), select(1.0, far_s, two), fade);
}
