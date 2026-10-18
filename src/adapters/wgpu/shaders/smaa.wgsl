// SMAA 1× (Jimenez m.fl., MIT-lisens, assets/smaa/): kantutjevning etter
// tonemappingen, i tre pass over hele skjermen. Portert fra SMAA.hlsl med
// forhåndsinnstillingen HIGH uten diagonalsøk. Teksturkoordinatene har origo
// øverst til venstre, som i D3D, så tabellene brukes som de er.
//
// 1. `fs_edges`: kantene der lysstyrken hopper (rg: kant mot venstre/over).
// 2. `fs_weights`: hvor langt hver kant går og hvor mye som skal blandes,
//    slått opp i arealtabellen.
// 3. `fs_blend`: hver piksel blandes med naboen langs kanten.
//
// Bildet inn er gamma-kodet (8 bit); `smaa.flags.x` sier om utbildet er et
// sRGB-format, så fargen må gjøres lineær igjen før den skrives.

struct Smaa {
    // 1/bredde, 1/høyde, bredde, høyde.
    metrics: vec4<f32>,
    flags: vec4<f32>,
}
@group(0) @binding(0) var tex_a: texture_2d<f32>;
@group(0) @binding(1) var tex_b: texture_2d<f32>;
@group(0) @binding(2) var area_tex: texture_2d<f32>;
@group(0) @binding(3) var search_tex: texture_2d<f32>;
@group(0) @binding(4) var lin: sampler;
@group(0) @binding(5) var pnt: sampler;
@group(0) @binding(6) var<uniform> smaa: Smaa;

const THRESHOLD: f32 = 0.1;
const MAX_SEARCH_STEPS: f32 = 16.0;
const CORNER_ROUNDING_NORM: f32 = 0.25;
const LOCAL_CONTRAST: f32 = 2.0;
const AREATEX_MAX_DISTANCE: f32 = 16.0;
const AREATEX_PIXEL_SIZE: vec2<f32> = vec2<f32>(1.0 / 160.0, 1.0 / 560.0);
const AREATEX_SUBTEX_SIZE: f32 = 1.0 / 7.0;
const SEARCHTEX_SIZE: vec2<f32> = vec2<f32>(66.0, 33.0);
const SEARCHTEX_PACKED_SIZE: vec2<f32> = vec2<f32>(64.0, 16.0);

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

fn luma(uv: vec2<f32>) -> f32 {
    return dot(textureSampleLevel(tex_a, pnt, uv, 0.0).rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// ---- 1. Kantene (tex_a: bildet) ----

@fragment
fn fs_edges(i: VsOut) -> @location(0) vec4<f32> {
    let m = smaa.metrics;
    let uv = i.uv;
    let l = luma(uv);
    let l_left = luma(uv + vec2<f32>(-m.x, 0.0));
    let l_top = luma(uv + vec2<f32>(0.0, -m.y));
    var delta = vec4<f32>(abs(l - vec2<f32>(l_left, l_top)), 0.0, 0.0);
    var edges = step(vec2<f32>(THRESHOLD), delta.xy);
    if (dot(edges, vec2<f32>(1.0)) == 0.0) {
        return vec4<f32>(0.0);
    }
    let l_right = luma(uv + vec2<f32>(m.x, 0.0));
    let l_bottom = luma(uv + vec2<f32>(0.0, m.y));
    delta = vec4<f32>(delta.xy, abs(l - vec2<f32>(l_right, l_bottom)));
    var max_delta = max(delta.xy, delta.zw);
    let l_leftleft = luma(uv + vec2<f32>(-2.0 * m.x, 0.0));
    let l_toptop = luma(uv + vec2<f32>(0.0, -2.0 * m.y));
    let d2 = abs(vec2<f32>(l_left, l_top) - vec2<f32>(l_leftleft, l_toptop));
    max_delta = max(max_delta, d2);
    let final_delta = max(max_delta.x, max_delta.y);
    // Bare kanter som er sterke nok mot naboene (lokal kontrast).
    edges *= step(vec2<f32>(final_delta), LOCAL_CONTRAST * delta.xy);
    return vec4<f32>(edges, 0.0, 0.0);
}

// ---- 2. Blandevektene (tex_a: kantene) ----

fn edge(uv: vec2<f32>) -> vec2<f32> {
    return textureSampleLevel(tex_a, lin, uv, 0.0).rg;
}

fn search_length(e: vec2<f32>, offset: f32) -> f32 {
    var scale = SEARCHTEX_SIZE * vec2<f32>(0.5, -1.0);
    var bias = SEARCHTEX_SIZE * vec2<f32>(offset, 1.0);
    scale += vec2<f32>(-1.0, 1.0);
    bias += vec2<f32>(0.5, -0.5);
    scale *= 1.0 / SEARCHTEX_PACKED_SIZE;
    bias *= 1.0 / SEARCHTEX_PACKED_SIZE;
    return textureSampleLevel(search_tex, lin, scale * e + bias, 0.0).r;
}

fn search_x_left(start: vec2<f32>, end: f32) -> f32 {
    let m = smaa.metrics;
    var uv = start;
    var e = vec2<f32>(0.0, 1.0);
    while (uv.x > end && e.g > 0.8281 && e.r == 0.0) {
        e = edge(uv);
        uv -= vec2<f32>(2.0 * m.x, 0.0);
    }
    let offset = -(255.0 / 127.0) * search_length(e, 0.0) + 3.25;
    return m.x * offset + uv.x;
}

fn search_x_right(start: vec2<f32>, end: f32) -> f32 {
    let m = smaa.metrics;
    var uv = start;
    var e = vec2<f32>(0.0, 1.0);
    while (uv.x < end && e.g > 0.8281 && e.r == 0.0) {
        e = edge(uv);
        uv += vec2<f32>(2.0 * m.x, 0.0);
    }
    let offset = -(255.0 / 127.0) * search_length(e, 0.5) + 3.25;
    return -m.x * offset + uv.x;
}

fn search_y_up(start: vec2<f32>, end: f32) -> f32 {
    let m = smaa.metrics;
    var uv = start;
    var e = vec2<f32>(1.0, 0.0);
    while (uv.y > end && e.r > 0.8281 && e.g == 0.0) {
        e = edge(uv);
        uv -= vec2<f32>(0.0, 2.0 * m.y);
    }
    let offset = -(255.0 / 127.0) * search_length(e.gr, 0.0) + 3.25;
    return m.y * offset + uv.y;
}

fn search_y_down(start: vec2<f32>, end: f32) -> f32 {
    let m = smaa.metrics;
    var uv = start;
    var e = vec2<f32>(1.0, 0.0);
    while (uv.y < end && e.r > 0.8281 && e.g == 0.0) {
        e = edge(uv);
        uv += vec2<f32>(0.0, 2.0 * m.y);
    }
    let offset = -(255.0 / 127.0) * search_length(e.gr, 0.5) + 3.25;
    return -m.y * offset + uv.y;
}

fn area(dist: vec2<f32>, e1: f32, e2: f32) -> vec2<f32> {
    var uv = AREATEX_MAX_DISTANCE * round(4.0 * vec2<f32>(e1, e2)) + dist;
    uv = AREATEX_PIXEL_SIZE * uv + 0.5 * AREATEX_PIXEL_SIZE;
    // SMAA 1×: underprøven har forskyvning 0.
    uv.y = AREATEX_SUBTEX_SIZE * 0.0 + uv.y;
    return textureSampleLevel(area_tex, lin, uv, 0.0).rg;
}

// Kanten et antall hele piksler unna.
fn edge_at(uv: vec2<f32>, dx: f32, dy: f32) -> vec2<f32> {
    return edge(uv + vec2<f32>(dx, dy) * smaa.metrics.xy);
}

fn corner_h(w: vec2<f32>, uv: vec4<f32>, d: vec2<f32>) -> vec2<f32> {
    let left_right = step(d.xy, d.yx);
    var rounding = (1.0 - CORNER_ROUNDING_NORM) * left_right;
    rounding /= left_right.x + left_right.y;
    var factor = vec2<f32>(1.0);
    factor.x -= rounding.x * edge_at(uv.xy, 0.0, 1.0).r;
    factor.x -= rounding.y * edge_at(uv.zw, 1.0, 1.0).r;
    factor.y -= rounding.x * edge_at(uv.xy, 0.0, -2.0).r;
    factor.y -= rounding.y * edge_at(uv.zw, 1.0, -2.0).r;
    return w * clamp(factor, vec2<f32>(0.0), vec2<f32>(1.0));
}

fn corner_v(w: vec2<f32>, uv: vec4<f32>, d: vec2<f32>) -> vec2<f32> {
    let left_right = step(d.xy, d.yx);
    var rounding = (1.0 - CORNER_ROUNDING_NORM) * left_right;
    rounding /= left_right.x + left_right.y;
    var factor = vec2<f32>(1.0);
    factor.x -= rounding.x * edge_at(uv.xy, 1.0, 0.0).g;
    factor.x -= rounding.y * edge_at(uv.zw, 1.0, 1.0).g;
    factor.y -= rounding.x * edge_at(uv.xy, -2.0, 0.0).g;
    factor.y -= rounding.y * edge_at(uv.zw, -2.0, 1.0).g;
    return w * clamp(factor, vec2<f32>(0.0), vec2<f32>(1.0));
}

@fragment
fn fs_weights(i: VsOut) -> @location(0) vec4<f32> {
    let m = smaa.metrics;
    let uv = i.uv;
    let pix = uv * m.zw;
    let off0 = m.xyxy * vec4<f32>(-0.25, -0.125, 1.25, -0.125) + uv.xyxy;
    let off1 = m.xyxy * vec4<f32>(-0.125, -0.25, -0.125, 1.25) + uv.xyxy;
    let off2 = m.xxyy * vec4<f32>(-2.0, 2.0, -2.0, 2.0) * MAX_SEARCH_STEPS + vec4<f32>(off0.xz, off1.yw);
    var weights = vec4<f32>(0.0);
    let e = textureSampleLevel(tex_a, lin, uv, 0.0).rg;

    if (e.g > 0.0) {
        // Kant over: hvor langt går den til venstre og høyre?
        var d: vec2<f32>;
        var coords: vec3<f32>;
        coords.x = search_x_left(off0.xy, off2.x);
        coords.y = off1.y;
        d.x = coords.x;
        let e1 = edge(coords.xy).r;
        coords.z = search_x_right(off0.zw, off2.y);
        d.y = coords.z;
        d = abs(round(m.zz * d - pix.xx));
        let e2 = edge_at(coords.zy, 1.0, 0.0).r;
        var w = area(sqrt(d), e1, e2);
        w = corner_h(w, vec4<f32>(coords.x, uv.y, coords.z, uv.y), d);
        weights = vec4<f32>(w, weights.zw);
    }

    if (e.r > 0.0) {
        // Kant til venstre: hvor langt går den opp og ned?
        var d: vec2<f32>;
        var coords: vec3<f32>;
        coords.y = search_y_up(off1.xy, off2.z);
        coords.x = off0.x;
        d.x = coords.y;
        let e1 = edge(coords.xy).g;
        coords.z = search_y_down(off1.zw, off2.w);
        d.y = coords.z;
        d = abs(round(m.ww * d - pix.yy));
        let e2 = edge_at(coords.xz, 0.0, 1.0).g;
        var w = area(sqrt(d), e1, e2);
        w = corner_v(w, vec4<f32>(uv.x, coords.y, uv.x, coords.z), d);
        weights = vec4<f32>(weights.xy, w);
    }
    return weights;
}

// ---- 3. Blandingen (tex_a: bildet, tex_b: vektene) ----

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn out_color(c: vec4<f32>) -> vec4<f32> {
    if (smaa.flags.x > 0.5) {
        return vec4<f32>(srgb_to_linear(c.rgb), 1.0);
    }
    return vec4<f32>(c.rgb, 1.0);
}

@fragment
fn fs_blend(i: VsOut) -> @location(0) vec4<f32> {
    let m = smaa.metrics;
    let uv = i.uv;
    let off = m.xyxy * vec4<f32>(1.0, 0.0, 0.0, 1.0) + uv.xyxy;
    var a: vec4<f32>;
    a.x = textureSampleLevel(tex_b, lin, off.xy, 0.0).a; // høyre
    a.y = textureSampleLevel(tex_b, lin, off.zw, 0.0).g; // under
    let here = textureSampleLevel(tex_b, lin, uv, 0.0);
    a.w = here.x; // over
    a.z = here.z; // venstre
    if (dot(a, vec4<f32>(1.0)) < 1e-5) {
        return out_color(textureSampleLevel(tex_a, lin, uv, 0.0));
    }
    let h = max(a.x, a.z) > max(a.y, a.w);
    var offset = vec4<f32>(0.0, a.y, 0.0, a.w);
    var weight = a.yw;
    if (h) {
        offset = vec4<f32>(a.x, 0.0, a.z, 0.0);
        weight = a.xz;
    }
    weight /= dot(weight, vec2<f32>(1.0));
    let coord = offset * vec4<f32>(m.xy, -m.xy) + uv.xyxy;
    var c = weight.x * textureSampleLevel(tex_a, lin, coord.xy, 0.0);
    c += weight.y * textureSampleLevel(tex_a, lin, coord.zw, 0.0);
    return out_color(c);
}
