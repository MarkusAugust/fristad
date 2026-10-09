// Skyggen fra sola: ett skyggekart rundt kameraet ut til `g.misc.w`, som
// toner bort fra `g.misc.z`. Fire faste prøver (2 × 2 texels, hver
// bilineært sammenlignet) gir skarpe, rolige kanter uten støy.

@group(1) @binding(0) var shadow_map: texture_depth_2d;
@group(1) @binding(1) var shadow_sampler: sampler_comparison;

fn sun_shadow(world: vec3<f32>, n: vec3<f32>) -> f32 {
    let d = distance(world, g.camera.xyz);
    let fade = smoothstep(g.misc.z, g.misc.w, d);
    if (fade >= 1.0) {
        return 1.0;
    }
    // Punktet flyttes litt ut langs normalen (mer på skrå mot sola), så
    // flatene ikke skygger for seg selv.
    let ndl = clamp(dot(n, g.sun_dir.xyz), 0.0, 1.0);
    let p4 = g.sun_vp * vec4<f32>(world + n * 0.03 * (1.0 + 2.0 * (1.0 - ndl)), 1.0);
    let p = p4.xyz / p4.w;
    let uv = p.xy * vec2<f32>(0.5, -0.5) + 0.5;
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || p.z > 1.0) {
        return 1.0;
    }
    let t = g.params.z * 0.5;
    let z = p.z - 0.0005;
    var s = textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2<f32>(-t, -t), z);
    s += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2<f32>(t, -t), z);
    s += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2<f32>(-t, t), z);
    s += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2<f32>(t, t), z);
    return mix(s * 0.25, 1.0, fade);
}
