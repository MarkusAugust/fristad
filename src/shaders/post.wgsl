// Etterbehandling: glød (bloom) og tonemapping til skjermen.

struct Post {
    // x eksponering, y bloom-styrke, z 1 hvis vi må gjøre sRGB-kodingen selv,
    // w terskel for glød.
    params: vec4<f32>,
    // xy størrelsen på kilden i texels, zw en texel.
    texel: vec4<f32>,
}

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> post: Post;
@group(0) @binding(3) var bloom_tex: texture_2d<f32>;

struct FsIn {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> FsIn {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    var o: FsIn;
    o.clip = vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
    o.uv = vec2<f32>(p.x, 1.0 - p.y);
    return o;
}

fn tap(uv: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(src, samp, uv, 0.0).rgb;
}

// 13 prøver, som i Call of Duty: Advanced Warfare. Første nivå tar bare med
// det som er lysere enn terskelen, og demper enkeltpiksler som blinker.
@fragment
fn fs_down(i: FsIn) -> @location(0) vec4<f32> {
    let t = post.texel.zw;
    let uv = i.uv;
    let a = tap(uv + t * vec2<f32>(-2.0, -2.0));
    let b = tap(uv + t * vec2<f32>(0.0, -2.0));
    let c = tap(uv + t * vec2<f32>(2.0, -2.0));
    let d = tap(uv + t * vec2<f32>(-2.0, 0.0));
    let e = tap(uv);
    let f = tap(uv + t * vec2<f32>(2.0, 0.0));
    let g2 = tap(uv + t * vec2<f32>(-2.0, 2.0));
    let h = tap(uv + t * vec2<f32>(0.0, 2.0));
    let k = tap(uv + t * vec2<f32>(2.0, 2.0));
    let j = tap(uv + t * vec2<f32>(-1.0, -1.0));
    let l = tap(uv + t * vec2<f32>(1.0, -1.0));
    let m = tap(uv + t * vec2<f32>(-1.0, 1.0));
    let n = tap(uv + t * vec2<f32>(1.0, 1.0));
    var c0 = e * 0.125 + (a + c + g2 + k) * 0.03125 + (b + d + f + h) * 0.0625 + (j + l + m + n) * 0.125;
    if (post.params.w > 0.0) {
        // Ett ugyldig tall ville spredt seg til en svart firkant i gløden.
        if (any(c0 != c0)) {
            c0 = vec3<f32>(0.0);
        }
        c0 = min(c0, vec3<f32>(64.0));
        let lum = dot(c0, vec3<f32>(0.2126, 0.7152, 0.0722));
        let knee = post.params.w;
        let soft = clamp(lum - knee * 0.5, 0.0, knee) ;
        let contrib = max(soft * soft / (2.0 * knee + 1e-4), lum - knee) / max(lum, 1e-4);
        c0 = c0 * contrib / (1.0 + lum * 0.1);
    }
    return vec4<f32>(c0, 1.0);
}

@fragment
fn fs_up(i: FsIn) -> @location(0) vec4<f32> {
    let t = post.texel.zw;
    let uv = i.uv;
    var c = tap(uv) * 4.0;
    c += (tap(uv + vec2<f32>(t.x, 0.0)) + tap(uv - vec2<f32>(t.x, 0.0)) + tap(uv + vec2<f32>(0.0, t.y)) + tap(uv - vec2<f32>(0.0, t.y))) * 2.0;
    c += tap(uv + t) + tap(uv - t) + tap(uv + vec2<f32>(t.x, -t.y)) + tap(uv + vec2<f32>(-t.x, t.y));
    return vec4<f32>(c / 16.0, 1.0);
}

// ACES, tilnærmet av Stephen Hill.
fn aces(x: vec3<f32>) -> vec3<f32> {
    let m1 = mat3x3<f32>(
        vec3<f32>(0.59719, 0.07600, 0.02840),
        vec3<f32>(0.35458, 0.90834, 0.13383),
        vec3<f32>(0.04823, 0.01566, 0.83777),
    );
    let m2 = mat3x3<f32>(
        vec3<f32>(1.60475, -0.10208, -0.00327),
        vec3<f32>(-0.53108, 1.10813, -0.07276),
        vec3<f32>(-0.07367, -0.00605, 1.07602),
    );
    let v = m1 * x;
    let a = v * (v + 0.0245786) - 0.000090537;
    let b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return clamp(m2 * (a / b), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

@fragment
fn fs_tonemap(i: FsIn) -> @location(0) vec4<f32> {
    let hdr = textureSampleLevel(src, samp, i.uv, 0.0).rgb;
    let bloom = textureSampleLevel(bloom_tex, samp, i.uv, 0.0).rgb;
    var c = (hdr + bloom * post.params.y) * post.params.x;
    c = aces(c);
    // Svak vignett.
    let d = i.uv - 0.5;
    c *= 1.0 - dot(d, d) * 0.35;
    if (post.params.z > 0.5) {
        c = linear_to_srgb(c);
    }
    // Støy mot stripete gradienter i himmelen.
    let n = fract(sin(dot(i.clip.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453) - 0.5;
    c += n / 255.0;
    return vec4<f32>(c, 1.0);
}
