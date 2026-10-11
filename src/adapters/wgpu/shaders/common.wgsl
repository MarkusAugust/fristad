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
    tex_scale: array<vec4<f32>, 16>,
    palette: array<vec4<f32>, 64>,
    lights: array<Light, 48>,
    // Pipene som ryker: toppen (xyz) og tykkelsen (w), opptil 32.
    smoke: array<vec4<f32>, 32>,
    // Rommene i kryptnivået (lyset stoppes av murene): x antallet (0 oppe
    // i byen); boksene (x0, z0, x1, z1); gulvet og taket, to rom per vec4;
    // og rommene hvert lys lyser i, to u32 per lys og to lys per vec4.
    room_info: vec4<f32>,
    room_box: array<vec4<f32>, 64>,
    room_y: array<vec4<f32>, 32>,
    light_rooms: array<vec4<u32>, 24>,
    // x grafikknivået (0 enkel, 1 standard, 2 avansert), y flat farge
    // (målingen `?bench`).
    quality: vec4<f32>,
}

// Rommet et punkt er i: det minste rommet med en boks som rommer det, eller
// −1 (inne i muren, eller oppe i byen).
fn room_of(p: vec3<f32>) -> i32 {
    let n = i32(g.room_info.x);
    var best = -1;
    var area = 1.0e9;
    for (var i = 0; i < n; i++) {
        let b = g.room_box[i];
        let yy = g.room_y[i / 2];
        let y = select(yy.xy, yy.zw, (i % 2) == 1);
        if (p.x >= b.x && p.x <= b.z && p.z >= b.y && p.z <= b.w && p.y >= y.x - 0.3 && p.y <= y.y + 0.3) {
            let a = (b.z - b.x) * (b.w - b.y);
            if (a < area) {
                area = a;
                best = i;
            }
        }
    }
    return best;
}

// Om lys nummer `k` lyser i rommet: i sitt eget og i rom med åpen
// forbindelse. Utenfor alle rom lyser bare lyktene med alle bitene satt
// (lykta du bærer). Oppe i byen lyser alt.
fn light_reaches(k: i32, room: i32) -> bool {
    if (g.room_info.x <= 0.0) {
        return true;
    }
    let v = g.light_rooms[k / 2];
    let odd = (k % 2) == 1;
    let lo = select(v.x, v.z, odd);
    let hi = select(v.y, v.w, odd);
    if (room < 0) {
        return lo == 0xffffffffu && hi == 0xffffffffu;
    }
    if (room < 32) {
        return ((lo >> u32(room)) & 1u) == 1u;
    }
    return ((hi >> u32(room - 32)) & 1u) == 1u;
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
