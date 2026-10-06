// Etterbehandling: SSAO, tåke med lysstråler, TAA, glød og tonemapping.
// Alle tegner en trekant over hele skjermen. Hvilke teksturer som er
// bundet avhenger av passet, se render.rs.

struct Post {
    params: vec4<f32>,
    // Størrelsen på kilden: bredde, høyde, 1/bredde, 1/høyde.
    texel: vec4<f32>,
}
@group(2) @binding(0) var tex0: texture_2d<f32>;
@group(2) @binding(1) var tex1: texture_2d<f32>;
@group(2) @binding(2) var tex2: texture_2d<f32>;
@group(2) @binding(3) var depth_tex: texture_depth_2d;
@group(2) @binding(4) var lin: sampler;
@group(2) @binding(5) var near: sampler;
@group(2) @binding(6) var<uniform> post: Post;

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

fn depth_at(uv: vec2<f32>) -> f32 {
    let size = vec2<f32>(textureDimensions(depth_tex));
    let c = vec2<i32>(clamp(uv * size, vec2<f32>(0.0), size - 1.0));
    return textureLoad(depth_tex, c, 0);
}

// ---- SSAO i halv oppløsning. tex0 er normalene fra prepasset. ----

@fragment
fn fs_ssao(i: VsOut) -> @location(0) vec4<f32> {
    let d = depth_at(i.uv);
    if (d >= 1.0) {
        return vec4<f32>(1.0);
    }
    let p = world_from_depth(i.uv, d);
    let n = normalize(textureSample(tex0, near, i.uv).xyz * 2.0 - 1.0);
    let dist = distance(p, g.camera.xyz);
    let radius = post.params.x;
    let px = i.clip.xy;
    let rnd = ign(px);
    let rnd2 = ign(px + 17.0);
    // Ortonormal ramme rundt normalen, rotert tilfeldig per piksel.
    var t = normalize(cross(n, vec3<f32>(0.0, 1.0, 0.01)));
    if (abs(n.y) > 0.99) {
        t = normalize(cross(n, vec3<f32>(1.0, 0.0, 0.0)));
    }
    let b = cross(n, t);
    let rot = rnd * 6.2831;
    let tr = t * cos(rot) + b * sin(rot);
    let br = cross(n, tr);
    var occ = 0.0;
    let taps = 10;
    for (var k = 0; k < taps; k++) {
        let fk = f32(k) + rnd2;
        let a = fk * 2.39996;
        let r = sqrt((fk + 0.5) / f32(taps));
        // Hemisfære: mer vekt nær normalen.
        let dir = tr * cos(a) * r + br * sin(a) * r + n * (0.3 + 0.7 * (1.0 - r * r));
        let sp = p + dir * radius * (0.3 + 0.7 * fract(fk * 0.618));
        let cp = g.view_proj * vec4<f32>(sp, 1.0);
        let suv = vec2<f32>(cp.x / cp.w * 0.5 + 0.5, 0.5 - cp.y / cp.w * 0.5);
        if (any(suv < vec2<f32>(0.0)) || any(suv > vec2<f32>(1.0))) {
            continue;
        }
        let sd = depth_at(suv);
        let scene_dist = linear_depth(sd);
        let sample_dist = linear_depth(cp.z / cp.w);
        let diff = sample_dist - scene_dist;
        if (diff > 0.02) {
            occ += smoothstep(radius * 2.0, 0.0, diff);
        }
    }
    let ao = 1.0 - occ / f32(taps) * post.params.y;
    // Mindre effekt langt unna, der presisjonen svikter.
    return vec4<f32>(mix(ao, 1.0, smoothstep(60.0, 150.0, dist)));
}

// Dybdebevisst uskarphet av SSAO, én retning per pass (params.zw).
@fragment
fn fs_blur(i: VsOut) -> @location(0) vec4<f32> {
    let d0 = linear_depth(depth_at(i.uv));
    var sum = 0.0;
    var wsum = 0.0;
    for (var k = -3; k <= 3; k++) {
        let uv = i.uv + post.params.zw * post.texel.zw * f32(k);
        let d = linear_depth(depth_at(uv));
        let w = exp(-f32(k * k) * 0.3) * exp(-abs(d - d0) * 2.0);
        sum += textureSample(tex0, lin, uv).r * w;
        wsum += w;
    }
    return vec4<f32>(sum / max(wsum, 1e-4));
}

// ---- Tåke med lysstråler. tex0 er HDR-bildet. ----

fn phase_hg(cos_t: f32, gg: f32) -> f32 {
    let denom = 1.0 + gg * gg - 2.0 * gg * cos_t;
    return (1.0 - gg * gg) / (4.0 * PI * pow(max(denom, 1e-3), 1.5));
}

// Hvor mye en lykt lyser opp tåka langs strålen, lukket form for 1/d².
fn lamp_scatter(origin: vec3<f32>, dir: vec3<f32>, len: f32, lp: vec3<f32>, radius: f32) -> f32 {
    let q = origin - lp;
    let b = dot(dir, q);
    let c = dot(q, q);
    let h2 = max(c - b * b, 0.04);
    let h = sqrt(h2);
    let inv = (atan((len + b) / h) - atan(b / h)) / h;
    // Ikke la lykter langt fra strålen telle.
    return inv * smoothstep(radius, radius * 0.3, h);
}

@fragment
fn fs_fog(i: VsOut) -> @location(0) vec4<f32> {
    let color = textureSample(tex0, near, i.uv).rgb;
    let d = depth_at(i.uv);
    let cam = g.camera.xyz;
    var p_end = world_from_depth(i.uv, min(d, 0.99999));
    var dist = distance(p_end, cam);
    let is_sky = d >= 1.0;
    if (is_sky) {
        dist = 600.0;
    }
    let dir = normalize(p_end - cam);
    let density = g.sky_horizon.w;
    let falloff = g.ground_bounce.w;
    let night = g.sun_color.w;

    // Høydetåke analytisk for utslokkingen.
    let y0 = cam.y;
    let dy = dir.y * dist;
    var height_term = exp(-falloff * y0);
    if (abs(dy) > 0.01) {
        height_term = (exp(-falloff * y0) - exp(-falloff * (y0 + dy))) / (falloff * dy);
    }
    let optical = density * dist * height_term;
    let transmittance = exp(-optical);

    // Innspredning fra sola: marsj gjennom skyggekartet med dithering.
    let steps = 14;
    let march_len = min(dist, 90.0);
    let jitter = ign(i.clip.xy);
    var sun_vis = 0.0;
    for (var k = 0; k < steps; k++) {
        let t = (f32(k) + jitter) / f32(steps);
        let p = cam + dir * (t * march_len);
        let w = exp(-falloff * max(p.y, 0.0));
        sun_vis += sun_shadow_cheap(p) * w;
    }
    sun_vis /= f32(steps);
    // Resten av strålen antas delvis i sol.
    let tail = clamp((dist - march_len) / max(dist, 1.0), 0.0, 1.0);
    sun_vis = mix(sun_vis, 0.6, tail);
    let cos_t = dot(dir, g.sun_dir.xyz);
    let phase = phase_hg(cos_t, 0.55) * 4.0 * PI;
    let sun_in = g.sun_color.rgb * g.sun_dir.w * sun_vis * mix(0.35, phase, 0.6);
    let ambient_in = mix(g.fog_color.rgb, sky_gradient(normalize(vec3<f32>(dir.x, 0.05, dir.z))), 0.5);
    var inscatter = (ambient_in * 0.7 + sun_in * 0.3) * (1.0 - transmittance);

    // Lyktene lyser opp tåka rundt seg.
    let count = i32(g.params.y);
    var lamp = vec3<f32>(0.0);
    for (var k = 0; k < count; k++) {
        let li = g.lights[k];
        let r = li.pos_radius.w;
        lamp += li.color_intensity.rgb * li.color_intensity.w * lamp_scatter(cam, dir, dist, li.pos_radius.xyz, r);
    }
    inscatter += lamp * density * 1.4 * (0.5 + g.weather.z * 1.5 + g.weather.w);

    var out = color * transmittance + inscatter;
    // Lyn lyser opp alt i tåka.
    out += g.sky.z * (1.0 - transmittance) * vec3<f32>(0.8, 0.85, 1.0);
    return vec4<f32>(out, 1.0);
}

// ---- TAA. tex0 er bildet nå, tex1 historien. ----

fn rgb_to_ycocg(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(0.25 * c.r + 0.5 * c.g + 0.25 * c.b, 0.5 * c.r - 0.5 * c.b, -0.25 * c.r + 0.5 * c.g - 0.25 * c.b);
}

fn ycocg_to_rgb(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z);
}

@fragment
fn fs_taa(i: VsOut) -> @location(0) vec4<f32> {
    let cur = textureSample(tex0, near, i.uv).rgb;
    if (post.params.x < 0.5) {
        return vec4<f32>(cur, 1.0);
    }
    // Finn den nærmeste dybden i 3×3 for en stabil reprojeksjon på kanter.
    var best_d = 1.0;
    var best_uv = i.uv;
    var mn = vec3<f32>(1e9);
    var mx = vec3<f32>(-1e9);
    var m1 = vec3<f32>(0.0);
    var m2 = vec3<f32>(0.0);
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let uv = i.uv + vec2<f32>(f32(x), f32(y)) * post.texel.zw;
            let d = depth_at(uv);
            if (d < best_d) {
                best_d = d;
                best_uv = uv;
            }
            let c = rgb_to_ycocg(textureSample(tex0, near, uv).rgb);
            mn = min(mn, c);
            mx = max(mx, c);
            m1 += c;
            m2 += c * c;
        }
    }
    // Varians-klemme gir mindre flimmer enn ren min/maks.
    let mu = m1 / 9.0;
    let sigma = sqrt(max(m2 / 9.0 - mu * mu, vec3<f32>(0.0)));
    let gamma = 1.25;
    mn = max(mn, mu - sigma * gamma);
    mx = min(mx, mu + sigma * gamma);

    let wp = world_from_depth(best_uv, best_d);
    let prev = g.prev_view_proj * vec4<f32>(wp, 1.0);
    let puv = vec2<f32>(prev.x / prev.w * 0.5 + 0.5, 0.5 - prev.y / prev.w * 0.5);
    if (any(puv < vec2<f32>(0.0)) || any(puv > vec2<f32>(1.0)) || best_d >= 1.0) {
        // Himmelen: reprojiser retningen i stedet, den er langt unna.
        if (best_d >= 1.0) {
            let far = world_from_depth(i.uv, 0.9999);
            let pr = g.prev_view_proj * vec4<f32>(far, 1.0);
            let suv = vec2<f32>(pr.x / pr.w * 0.5 + 0.5, 0.5 - pr.y / pr.w * 0.5);
            if (all(suv >= vec2<f32>(0.0)) && all(suv <= vec2<f32>(1.0))) {
                let h = rgb_to_ycocg(textureSample(tex1, lin, suv).rgb);
                let hc = clamp(h, mn, mx);
                return vec4<f32>(ycocg_to_rgb(mix(rgb_to_ycocg(cur), hc, 0.9)), 1.0);
            }
        }
        return vec4<f32>(cur, 1.0);
    }
    let hist = rgb_to_ycocg(textureSample(tex1, lin, puv).rgb);
    let clamped = clamp(hist, mn, mx);
    // Mindre vekt på historien der den måtte klemmes mye (bevegelse).
    let dev = length(hist - clamped) / max(length(sigma), 1e-3);
    let blend = post.params.y * (1.0 - smoothstep(0.5, 2.5, dev) * 0.5);
    let out = mix(rgb_to_ycocg(cur), clamped, blend);
    return vec4<f32>(max(ycocg_to_rgb(out), vec3<f32>(0.0)), 1.0);
}

// ---- Glød. ----

fn karis(c: vec3<f32>) -> f32 {
    return 1.0 / (1.0 + max(max(c.r, c.g), c.b));
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
        var s = textureSample(tex0, lin, i.uv + offs[k] * t).rgb;
        if (first) {
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
    c += textureSample(tex0, lin, i.uv + vec2<f32>(-1.0, -1.0) * t).rgb * 1.0;
    c += textureSample(tex0, lin, i.uv + vec2<f32>(0.0, -1.0) * t).rgb * 2.0;
    c += textureSample(tex0, lin, i.uv + vec2<f32>(1.0, -1.0) * t).rgb * 1.0;
    c += textureSample(tex0, lin, i.uv + vec2<f32>(-1.0, 0.0) * t).rgb * 2.0;
    c += textureSample(tex0, lin, i.uv).rgb * 4.0;
    c += textureSample(tex0, lin, i.uv + vec2<f32>(1.0, 0.0) * t).rgb * 2.0;
    c += textureSample(tex0, lin, i.uv + vec2<f32>(-1.0, 1.0) * t).rgb * 1.0;
    c += textureSample(tex0, lin, i.uv + vec2<f32>(0.0, 1.0) * t).rgb * 2.0;
    c += textureSample(tex0, lin, i.uv + vec2<f32>(1.0, 1.0) * t).rgb * 1.0;
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

@fragment
fn fs_tonemap(i: VsOut) -> @location(0) vec4<f32> {
    let exposure = post.params.x;
    let bloom = post.params.y;
    var hdr = textureSample(tex0, lin, i.uv).rgb;
    if (g.sky.w > 0.0 && g.sky.w < 13.0) {
        var d = clamp(hdr, vec3<f32>(0.0), vec3<f32>(1.0));
        if (post.params.z > 0.5) {
            d = linear_to_srgb(d);
        }
        return vec4<f32>(d, 1.0);
    }
    hdr += textureSample(tex1, lin, i.uv).rgb * bloom;
    hdr *= exposure;
    var c = aces(hdr);
    // Litt varmere i lysene og kjøligere i skyggene, som film.
    let lum = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
    c = mix(c, c * vec3<f32>(0.96, 0.98, 1.05), smoothstep(0.5, 0.0, lum) * 0.5);
    c = mix(c, c * vec3<f32>(1.03, 1.0, 0.96), smoothstep(0.5, 1.0, lum) * 0.4);
    // Vignett og fint korn.
    let v = i.uv * 2.0 - 1.0;
    c *= 1.0 - dot(v, v) * 0.12;
    let grain = (ign(i.clip.xy) - 0.5) * 0.02 * (1.0 - lum * 0.5);
    c = clamp(c + grain, vec3<f32>(0.0), vec3<f32>(1.0));
    if (post.params.z > 0.5) {
        c = linear_to_srgb(c);
    }
    return vec4<f32>(c, 1.0);
}
