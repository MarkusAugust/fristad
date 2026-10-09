// Felles for alle shaderne. Må stemme med `Globals` i render.rs.

struct Light {
    pos_radius: vec4<f32>,
    color_intensity: vec4<f32>,
}

struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    // Skyggekartet fra sola (én kaskade).
    sun_vp: mat4x4<f32>,
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
    // x vind, y MSAA-prøver, z der skyggen begynner å tone bort, w der den
    // slutter (meter fra kameraet).
    misc: vec4<f32>,
    // x skydekke, y nordlys, z lyn, w feilsøkingsvisning.
    sky: vec4<f32>,
    // Meter per flis for teksturlagene, 4 per vec4.
    tex_scale: array<vec4<f32>, 12>,
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

// Tåka i et punkt: høydetåke regnet analytisk langs strålen fra kameraet,
// lyst opp av himmelen og (mot sola) av sollyset, og lyn. Regnes rett i
// hvert punkt i hovedpasset, uten eget pass og uten støy.
fn fog(color: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    let density = g.sky_horizon.w;
    if (density <= 0.0) {
        return color;
    }
    let cam = g.camera.xyz;
    let ray = p - cam;
    let dist = length(ray);
    let dir = ray / max(dist, 1e-4);
    let falloff = g.ground_bounce.w;
    let dy = dir.y * dist;
    var height_term = exp(-falloff * cam.y);
    if (abs(dy) > 0.01) {
        height_term = (exp(-falloff * cam.y) - exp(-falloff * (cam.y + dy))) / (falloff * dy);
    }
    let transmittance = exp(-density * dist * height_term);
    let cos_t = dot(dir, g.sun_dir.xyz);
    let gg = 0.55;
    let phase = (1.0 - gg * gg) / pow(1.0 + gg * gg - 2.0 * gg * cos_t, 1.5);
    let sun_in = g.sun_color.rgb * g.sun_dir.w * 0.6 * mix(0.35, phase, 0.6);
    let ambient_in = mix(g.fog_color.rgb, sky_gradient(normalize(vec3<f32>(dir.x, 0.05, dir.z))), 0.5);
    var inscatter = (ambient_in * 0.7 + sun_in * 0.3) * (1.0 - transmittance);
    inscatter += g.sky.z * (1.0 - transmittance) * vec3<f32>(0.8, 0.85, 1.0);
    return color * transmittance + inscatter;
}
