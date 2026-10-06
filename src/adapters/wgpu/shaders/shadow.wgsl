// Kaskaderte skyggekart i et 2×2-atlas. Kaskade 0 er nærmest og skarpest.

@group(1) @binding(0) var shadow_map: texture_depth_2d;
@group(1) @binding(1) var shadow_sampler: sampler_comparison;

fn cascade_for(world: vec3<f32>) -> i32 {
    let d = distance(world, g.camera.xyz);
    if (d < g.cascade.x) { return 0; }
    if (d < g.cascade.y) { return 1; }
    return 2;
}

// Atlasrute for kaskaden: 0 oppe til venstre, 1 oppe til høyre, 2 nede til venstre.
fn atlas_uv(c: i32, uv: vec2<f32>) -> vec2<f32> {
    let o = vec2<f32>(f32(c % 2), f32(c / 2));
    return (clamp(uv, vec2<f32>(0.001), vec2<f32>(0.999)) + o) * 0.5;
}

fn shadow_sample(c: i32, world: vec3<f32>, taps: i32, rot_seed: vec2<f32>) -> f32 {
    let lp = g.sun_vp[c] * vec4<f32>(world, 1.0);
    let p = lp.xyz / lp.w;
    let uv = p.xy * vec2<f32>(0.5, -0.5) + 0.5;
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || p.z > 1.0) {
        return 1.0;
    }
    let texel = g.params.z;
    let bias = select(0.0012, 0.0006, c == 0);
    var sum = 0.0;
    let rot = hash21(rot_seed) * 6.2831;
    let cs = vec2<f32>(cos(rot), sin(rot));
    let n = f32(taps);
    for (var i = 0; i < taps; i++) {
        let fi = f32(i);
        let r = sqrt((fi + 0.5) / n) * 1.6;
        let a = fi * 2.39996;
        var d = vec2<f32>(cos(a), sin(a)) * r;
        d = vec2<f32>(d.x * cs.x - d.y * cs.y, d.x * cs.y + d.y * cs.x);
        sum += textureSampleCompareLevel(shadow_map, shadow_sampler, atlas_uv(c, uv + d * texel), p.z - bias);
    }
    return sum / n;
}

// Skygge fra sola i et punkt, med myk overgang mellom kaskadene.
fn sun_shadow(world: vec3<f32>, n: vec3<f32>, rot_seed: vec2<f32>) -> f32 {
    let ndl = dot(n, g.sun_dir.xyz);
    let c = cascade_for(world);
    let scale = select(select(0.12, 0.05, c == 1), 0.025, c == 0);
    let offset = n * scale * (1.0 + 2.0 * (1.0 - clamp(ndl, 0.0, 1.0)));
    let taps = select(select(6, 8, c == 1), 12, c == 0);
    var s = shadow_sample(c, world + offset, taps, rot_seed);
    // Bland mot neste kaskade nær grensa.
    let d = distance(world, g.camera.xyz);
    let edge = select(select(1e9, g.cascade.y, c == 1), g.cascade.x, c == 0);
    let blend = smoothstep(edge * 0.85, edge, d);
    if (blend > 0.0 && c < 2) {
        let s2 = shadow_sample(c + 1, world + offset * 2.5, 6, rot_seed);
        s = mix(s, s2, blend);
    }
    return s;
}

// Billig skygge til tåkemarsjen: én prøve, riktig kaskade.
fn sun_shadow_cheap(world: vec3<f32>) -> f32 {
    let c = cascade_for(world);
    let lp = g.sun_vp[c] * vec4<f32>(world, 1.0);
    let p = lp.xyz / lp.w;
    let uv = p.xy * vec2<f32>(0.5, -0.5) + 0.5;
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || p.z > 1.0) {
        return 1.0;
    }
    return textureSampleCompareLevel(shadow_map, shadow_sampler, atlas_uv(c, uv), p.z - 0.002);
}
