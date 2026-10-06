// Verden: alle husene, bakken, trærne og propsene.
//
// Ingen teksturer. Brostein, puss, treverk, skifer og mur tegnes her ut fra
// materialnummeret og posisjonen i verden, så nedlastingen er null bytes og
// detaljene er like skarpe uansett hvor nær du går.

struct Object {
    model: mat4x4<f32>,
    // x: markering når du sikter på den.
    extra: vec4<f32>,
}

@group(1) @binding(0) var<uniform> obj: Object;
@group(2) @binding(0) var shadow_map: texture_depth_2d;
@group(2) @binding(1) var shadow_sampler: sampler_comparison;

struct VsIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) mat: vec4<u32>,
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) @interpolate(flat) mat: vec4<u32>,
    @location(4) ao: f32,
}

@vertex
fn vs(v: VsIn) -> VsOut {
    var o: VsOut;
    let w = obj.model * vec4<f32>(v.pos, 1.0);
    o.world = w.xyz;
    o.clip = g.view_proj * w;
    o.normal = (obj.model * vec4<f32>(v.normal, 0.0)).xyz;
    o.color = v.color;
    o.mat = v.mat;
    o.ao = f32(v.mat.w) / 255.0;
    return o;
}

@vertex
fn vs_shadow(v: VsIn) -> @builtin(position) vec4<f32> {
    let w = obj.model * vec4<f32>(v.pos, 1.0);
    return g.sun_view_proj * w;
}

fn shadow(world: vec3<f32>, n: vec3<f32>) -> f32 {
    let ndl = dot(n, g.sun_dir.xyz);
    let offset = n * (0.04 + 0.08 * (1.0 - clamp(ndl, 0.0, 1.0)));
    let lp = g.sun_view_proj * vec4<f32>(world + offset, 1.0);
    let p = lp.xyz / lp.w;
    let uv = p.xy * vec2<f32>(0.5, -0.5) + 0.5;
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || p.z > 1.0) {
        return 1.0;
    }
    let texel = g.params.z;
    var sum = 0.0;
    // 16 prøver i et rotert mønster gir myke kanter uten stripemønster.
    let rot = hash21(world.xz * 37.0) * 6.2831;
    let cs = vec2<f32>(cos(rot), sin(rot));
    for (var i = 0; i < 16; i++) {
        let fi = f32(i);
        let r = sqrt((fi + 0.5) / 16.0) * 1.8;
        let a = fi * 2.39996;
        var d = vec2<f32>(cos(a), sin(a)) * r;
        d = vec2<f32>(d.x * cs.x - d.y * cs.y, d.x * cs.y + d.y * cs.x);
        sum += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + d * texel, p.z - 0.0008);
    }
    return sum / 16.0;
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
}

// Mønster for murstein og brostein: avstand til fugen og et nummer per stein.
fn bond(p: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
    var q = p / size;
    let row = floor(q.y);
    q.x += row * 0.5 + hash21(vec2<f32>(row, 3.1)) * 0.3;
    let cell = floor(q);
    let f = fract(q) - 0.5;
    let edge = min((0.5 - abs(f.x)) * size.x, (0.5 - abs(f.y)) * size.y);
    return vec3<f32>(edge, hash21(cell), 0.0);
}

fn detail(s_in: Surf, kind: u32, world: vec3<f32>, n_geo: vec3<f32>, fw: f32) -> Surf {
    var s = s_in;
    let an = abs(n_geo);
    // Planet flaten ligger i, så mønstrene følger veggen.
    var uv = world.xz;
    if (an.x > an.y && an.x > an.z) {
        uv = world.zy;
    } else if (an.z > an.y) {
        uv = world.xy;
    }
    switch kind {
        case 1u: { // Brostein
            let b = bond(world.xz, vec2<f32>(0.24, 0.17));
            // Fugene tones ut på avstand, ellers blir de flimrende striper.
            let far = smoothstep(0.012, 0.05, fw);
            let joint = mix(smoothstep(0.0, 0.025 + fw, b.x), 0.85, far);
            let tone = 0.78 + b.y * 0.38 + (fbm2(world.xz * 0.35) - 0.5) * 0.3;
            s.albedo *= mix(0.45, tone, joint);
            s.rough = mix(1.0, s.rough * (0.9 + b.y * 0.2), joint);
            s.cavity = mix(0.55, 1.0, joint);
            // Avrundede steiner: normalen bøyes mot kanten.
            let e = 0.004;
            let bx = bond(world.xz + vec2<f32>(e, 0.0), vec2<f32>(0.24, 0.17)).x;
            let bz = bond(world.xz + vec2<f32>(0.0, e), vec2<f32>(0.24, 0.17)).x;
            let grad = vec2<f32>(bx - b.x, bz - b.x) / e;
            let bump = (1.0 - smoothstep(0.0, 0.05, b.x)) * 0.6 * (1.0 - far);
            s.n = normalize(s.n - vec3<f32>(grad.x, 0.0, grad.y) * bump);
        }
        case 2u: { // Puss
            let v = fbm2(uv * 1.3) - 0.5;
            s.albedo *= 1.0 + v * 0.1;
            s.albedo *= 1.0 - smoothstep(0.3, 0.0, world.y - 0.4) * 0.15;
        }
        case 3u: { // Treverk
            let grain = noise2(vec2<f32>(uv.x * 2.0, uv.y * 40.0));
            s.albedo *= 0.9 + grain * 0.18;
        }
        case 4u: { // Glass
            s.rough = 0.05;
        }
        case 5u: { // Skifer
            let b = bond(vec2<f32>(world.x + world.z, world.y * 1.4), vec2<f32>(0.4, 0.25));
            let joint = smoothstep(0.0, 0.02, b.x);
            s.albedo *= mix(0.55, 0.85 + b.y * 0.3, joint);
            s.cavity = mix(0.6, 1.0, joint);
        }
        case 6u: { // Mur
            if (an.y < 0.5) {
                let b = bond(uv, vec2<f32>(0.9, 0.45));
                let joint = smoothstep(0.0, 0.02, b.x);
                s.albedo *= mix(1.15, 0.85 + b.y * 0.25, joint);
                s.cavity = mix(0.7, 1.0, joint);
            } else {
                s.albedo *= 0.9 + fbm2(uv * 2.0) * 0.2;
            }
        }
        case 7u: { // Gress
            s.albedo *= 0.75 + fbm2(world.xz * 3.0) * 0.5;
        }
        case 8u: { // Metall
            s.metal = 0.8;
        }
        case 9u: { // Løv
            let v = fbm2(world.xz * 1.7 + world.y * 1.3);
            s.albedo *= 0.6 + v * 0.75;
            s.cavity = 0.75 + v * 0.25;
        }
        default: {}
    }
    return s;
}

@fragment
fn fs(i: VsOut) -> @location(0) vec4<f32> {
    let kind = i.mat.x;
    let slot = i.mat.y;
    let glow = i.mat.z;
    let n_geo = normalize(i.normal);
    // Hvor mye verden én piksel dekker, til å tone ut fine mønstre.
    let fw = length(fwidth(i.world.xz));

    var s: Surf;
    s.albedo = srgb_to_linear(i.color.rgb);
    if (slot > 0u) {
        s.albedo = g.palette[slot].rgb;
    }
    s.rough = i.color.a;
    s.metal = 0.0;
    s.n = n_geo;
    s.cavity = 1.0;
    s = detail(s, kind, i.world, n_geo, fw);

    let n = s.n;
    let v = normalize(g.camera.xyz - i.world);
    let night = g.sun_color.w;
    let f0 = mix(vec3<f32>(0.04), s.albedo, s.metal);
    let diffuse_albedo = s.albedo * (1.0 - s.metal);

    // Sola eller månen.
    let l = g.sun_dir.xyz;
    let ndl = max(dot(n, l), 0.0);
    var wrap = ndl;
    if (kind == 9u) {
        wrap = max((dot(n, l) + 0.5) / 1.5, 0.0);
    }
    let sh = shadow(i.world, n_geo);
    let sun = g.sun_color.rgb * g.sun_dir.w * sh;
    var color = sun * (diffuse_albedo * wrap + ggx(n, v, l, s.rough, f0) * PI * ndl);

    // Himmellys fra alle kanter, og lys som spretter opp fra bakken.
    let ao = i.ao * s.cavity;
    let hemi = mix(g.ground_bounce.rgb, g.sky_top.rgb, n.y * 0.5 + 0.5);
    color += diffuse_albedo * hemi * ao;

    // Refleksjon av himmelen.
    let r = reflect(-v, n);
    let fres = f0 + (1.0 - f0) * pow(1.0 - max(dot(n, v), 0.0), 5.0);
    let env = sky_gradient(r) * (1.0 - s.rough * 0.85);
    color += env * fres * ao * mix(0.6, 1.0, sh);

    // Lyktene.
    let count = i32(g.params.y);
    for (var k = 0; k < count; k++) {
        let li = g.lights[k];
        let d = li.pos_radius.xyz - i.world;
        let dist = length(d);
        let rad = li.pos_radius.w;
        if (dist < rad) {
            let ld = d / dist;
            let win = pow(clamp(1.0 - pow(dist / rad, 4.0), 0.0, 1.0), 2.0);
            let att = win / (dist * dist + 0.5);
            let pl = li.color_intensity.rgb * li.color_intensity.w * att;
            let pndl = max(dot(n, ld), 0.0);
            color += pl * (diffuse_albedo * pndl + ggx(n, v, ld, s.rough, f0) * PI * pndl) * mix(0.6, 1.0, ao);
        }
    }

    // Det som lyser selv.
    if (glow == 1u) {
        // Hvert vindu er en egen celle, og noen er tent om kvelden.
        let cell = floor(vec3<f32>(i.world.x * 0.7, i.world.y / 3.0, i.world.z * 0.7));
        let h = hash31(cell);
        if (h < g.params.w) {
            let warm = mix(vec3<f32>(1.0, 0.66, 0.36), vec3<f32>(1.0, 0.85, 0.65), hash31(cell + 7.0));
            color += warm * night * (1.6 + h * 2.0);
        }
    } else if (glow == 2u) {
        color += vec3<f32>(1.0, 0.75, 0.48) * (0.3 + night * 9.0);
    } else if (glow == 3u) {
        color += s.albedo * (0.06 + night * 0.35);
    }

    // Markering når du sikter på noe.
    let hl = obj.extra.x;
    if (hl > 0.0) {
        let rim = pow(1.0 - max(dot(n, v), 0.0), 2.0);
        color += (s.albedo * 0.35 + vec3<f32>(rim * 0.6)) * hl * (0.5 + 0.5 * sin(g.camera.w * 5.0));
    }

    color = fog(color, i.world);
    return vec4<f32>(color, 1.0);
}
