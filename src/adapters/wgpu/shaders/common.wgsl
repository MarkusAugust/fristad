// Felles for alle shaderne. Må stemme med `Globals` i render.rs.

struct Light {
    pos_radius: vec4<f32>,
    color_intensity: vec4<f32>,
}

struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    prev_view_proj: mat4x4<f32>,
    // Skyggekartene, ett per kaskade.
    sun_vp: array<mat4x4<f32>, 3>,
    // xyz kamera, w tid i sekunder.
    camera: vec4<f32>,
    // xyz retning mot sola (eller månen), w styrke.
    sun_dir: vec4<f32>,
    // rgb, w natt fra 0 til 1.
    sun_color: vec4<f32>,
    sky_top: vec4<f32>,
    // rgb, w tåketetthet.
    sky_horizon: vec4<f32>,
    // rgb, w hvor fort tåka tynnes ut med høyden.
    ground_bounce: vec4<f32>,
    // rgb, w andel tente vinduer.
    fog_color: vec4<f32>,
    // x eksponering, y antall lys, z en skyggetexel, w gatelys 0..1.
    params: vec4<f32>,
    // x våt, y snødekke, z regn i lufta, w snø i lufta.
    weather: vec4<f32>,
    // bredde, høyde, 1/bredde, 1/høyde.
    screen: vec4<f32>,
    // xy TAA-forskyvning i piksler, z bildenummer, w vind.
    jitter: vec4<f32>,
    // Hvor kaskadene slutter, i meter fra kameraet.
    cascade: vec4<f32>,
    // x skydekke, y nordlys, z lyn, w feilsøkingsvisning.
    sky: vec4<f32>,
    // Meter per flis for teksturlagene, 4 per vec4.
    tex_scale: array<vec4<f32>, 7>,
    palette: array<vec4<f32>, 64>,
    lights: array<Light, 48>,
    // Pipene som ryker: toppen (xyz) og tykkelsen (w), opptil 32.
    smoke: array<vec4<f32>, 32>,
    // Kjellerens rutenett (40 × 40): hjørnet i x og z, gulvhøyden og
    // cellestørrelsen (0 utenfor kjelleren); én byte per celle, fire per
    // u32: bit 0 åpen, bit 1 fri vei mot +x, bit 2 mot +z.
    grid_info: vec4<f32>,
    grid: array<vec4<u32>, 100>,
}

fn grid_cell(ix: i32, iz: i32) -> u32 {
    if (ix < 0 || iz < 0 || ix >= 40 || iz >= 40) {
        return 0u;
    }
    let i = u32(iz * 40 + ix);
    let word = g.grid[i / 16u][(i / 4u) % 4u];
    return (word >> ((i % 4u) * 8u)) & 255u;
}

// Om lyset i `light` når punktet `p` uten å gå gjennom en mur i kjelleren:
// går cellene langs linja mellom dem (flatt) og sjekker hver kant.
fn grid_lit(p: vec3<f32>, light: vec3<f32>) -> f32 {
    let cell = g.grid_info.w;
    if (cell <= 0.0) {
        return 1.0;
    }
    let a = (p.xz - g.grid_info.xy) / cell;
    let b = (light.xz - g.grid_info.xy) / cell;
    var c = vec2<i32>(floor(a));
    let end = vec2<i32>(floor(b));
    let d = b - a;
    let stepv = vec2<i32>(select(-1, 1, d.x >= 0.0), select(-1, 1, d.y >= 0.0));
    let inv = vec2<f32>(1.0 / max(abs(d.x), 1e-5), 1.0 / max(abs(d.y), 1e-5));
    let fr = a - floor(a);
    var t = vec2<f32>(select(fr.x, 1.0 - fr.x, d.x >= 0.0) * inv.x, select(fr.y, 1.0 - fr.y, d.y >= 0.0) * inv.y);
    for (var k = 0; k < 8; k++) {
        if (c.x == end.x && c.y == end.y) {
            return 1.0;
        }
        if (t.x < t.y) {
            // Over kanten i x: fri vei står i cella med lavest x.
            let lo = select(c.x - 1, c.x, stepv.x > 0);
            if ((grid_cell(lo, c.y) & 2u) == 0u) {
                return 0.0;
            }
            c.x += stepv.x;
            t.x += inv.x;
        } else {
            let lo = select(c.y - 1, c.y, stepv.y > 0);
            if ((grid_cell(c.x, lo) & 4u) == 0u) {
                return 0.0;
            }
            c.y += stepv.y;
            t.y += inv.y;
        }
    }
    return 1.0;
}

@group(0) @binding(0) var<uniform> g: Globals;

// Blått-ish støymønster per piksel som bytter hvert bilde, til TAA og dithering.
fn ign(px: vec2<f32>) -> f32 {
    let f = g.jitter.z;
    let p = px + 5.588238 * (f % 64.0);
    return fract(52.9829189 * fract(0.06711056 * p.x + 0.00583715 * p.y));
}

// Himmelfargen i en retning, uten sol og stjerner. Brukes både av himmelen og
// til refleksjoner og tåke.
fn sky_gradient(dir: vec3<f32>) -> vec3<f32> {
    let h = clamp(dir.y, -1.0, 1.0);
    let up = pow(clamp(h, 0.0, 1.0), 0.45);
    var c = mix(g.sky_horizon.rgb, g.sky_top.rgb, up);
    c = mix(c, g.ground_bounce.rgb * 0.6 + g.sky_horizon.rgb * 0.4, smoothstep(0.0, -0.25, h));
    let s = max(dot(dir, g.sun_dir.xyz), 0.0);
    c += g.sun_color.rgb * (pow(s, 6.0) * 0.25 + pow(s, 64.0) * 0.6) * (1.0 - g.sun_color.w * 0.8) * (1.0 - g.sky.x * 0.7);
    return c;
}

// Lineær dybde fra dybdebufferet (perspektiv, revers-ikke).
fn linear_depth(d: f32) -> f32 {
    // proj er perspective_rh med near 0.1 og far 2500.
    let n = 0.1;
    let f = 2500.0;
    return n * f / (f - d * (f - n));
}

fn world_from_depth(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
    let w = g.inv_view_proj * ndc;
    return w.xyz / w.w;
}
