// Skygge i krokene: okklusjon i skjermrommet fra dybden, i halv
// oppløsning. Tolv faste prøver i en halvkule over flaten (ingen tilfeldig
// dreiing, så bildet aldri får støy eller flimrer), så en utjevning som
// stopper ved kanter i dybden. Ute i 35–55 m toner den bort.

@group(1) @binding(0) var depth_tex: texture_depth_2d;
@group(1) @binding(1) var ao_in: texture_2d<f32>;

struct AoOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> AoOut {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
    var o: AoOut;
    o.clip = vec4<f32>(p, 0.0, 1.0);
    o.uv = vec2<f32>(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);
    return o;
}

// Punktet i verden bak en piksel i dybden (full oppløsning).
fn world_at(px: vec2<i32>, size: vec2<i32>) -> vec4<f32> {
    let q = clamp(px, vec2<i32>(0), size - vec2<i32>(1));
    let d = textureLoad(depth_tex, q, 0);
    let uv = (vec2<f32>(q) + 0.5) / vec2<f32>(size);
    let w = g.inv_view_proj * vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    return vec4<f32>(w.xyz / w.w, d);
}

@fragment
fn fs_ao(i: AoOut) -> @location(0) vec4<f32> {
    let size = vec2<i32>(textureDimensions(depth_tex));
    let px = vec2<i32>(i.uv * vec2<f32>(size));
    let c = world_at(px, size);
    if (c.w >= 1.0) {
        return vec4<f32>(1.0);
    }
    let p = c.xyz;
    let cam = g.camera.xyz;
    let dist = distance(p, cam);
    if (dist > 55.0) {
        return vec4<f32>(1.0);
    }
    // Normalen fra naboene, fra den siden som ligger nærmest i dybden (så
    // kantene ikke får en skrå normal).
    let r = world_at(px + vec2<i32>(1, 0), size).xyz - p;
    let l = p - world_at(px - vec2<i32>(1, 0), size).xyz;
    let u = world_at(px + vec2<i32>(0, 1), size).xyz - p;
    let d = p - world_at(px - vec2<i32>(0, 1), size).xyz;
    let dx = select(r, l, dot(l, l) < dot(r, r));
    let dy = select(u, d, dot(d, d) < dot(u, u));
    var n = normalize(cross(dy, dx));
    if (dot(n, cam - p) < 0.0) {
        n = -n;
    }
    let t = normalize(cross(n, select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(n.y) > 0.9)));
    let b = cross(n, t);
    let kernel = array<vec3<f32>, 12>(
        vec3<f32>(0.42, 0.10, 0.20), vec3<f32>(-0.31, 0.38, 0.25), vec3<f32>(-0.12, -0.47, 0.32),
        vec3<f32>(0.55, -0.42, 0.40), vec3<f32>(-0.68, -0.11, 0.30), vec3<f32>(0.18, 0.71, 0.45),
        vec3<f32>(0.80, 0.25, 0.50), vec3<f32>(-0.45, 0.62, 0.60), vec3<f32>(-0.20, -0.82, 0.55),
        vec3<f32>(0.10, -0.15, 0.95), vec3<f32>(0.62, -0.70, 0.35), vec3<f32>(-0.85, -0.45, 0.28),
    );
    let radius = 0.4;
    var occ = 0.0;
    for (var k = 0; k < 12; k++) {
        let o = kernel[k];
        let s = p + (t * o.x + b * o.y + n * o.z) * radius;
        let cs = g.view_proj * vec4<f32>(s, 1.0);
        let suv = cs.xy / cs.w * vec2<f32>(0.5, -0.5) + 0.5;
        if (any(suv < vec2<f32>(0.0)) || any(suv > vec2<f32>(1.0))) {
            continue;
        }
        let hit = world_at(vec2<i32>(suv * vec2<f32>(size)), size);
        let scene = distance(hit.xyz, cam);
        let sample_d = distance(s, cam);
        if (scene < sample_d - 0.04) {
            // Bare det som er nær nok teller (ikke en mur langt foran).
            occ += smoothstep(0.0, 1.0, radius / max(abs(dist - scene), 1e-3));
        }
    }
    let ao = 1.0 - occ / 12.0 * 0.85;
    return vec4<f32>(mix(ao, 1.0, smoothstep(35.0, 55.0, dist)));
}

// Utjevningen: 5 × 5 i halv oppløsning, vektet etter hvor nær naboen er i
// dybden, så kroken ikke smitter over på en kant foran.
@fragment
fn fs_blur(i: AoOut) -> @location(0) vec4<f32> {
    let half = vec2<i32>(textureDimensions(ao_in));
    let size = vec2<i32>(textureDimensions(depth_tex));
    let hp = vec2<i32>(i.uv * vec2<f32>(half));
    let cam = g.camera.xyz;
    let d0 = distance(world_at(hp * 2, size).xyz, cam);
    var sum = 0.0;
    var wsum = 0.0;
    for (var y = -2; y <= 2; y++) {
        for (var x = -2; x <= 2; x++) {
            let q = clamp(hp + vec2<i32>(x, y), vec2<i32>(0), half - vec2<i32>(1));
            let dq = distance(world_at(q * 2, size).xyz, cam);
            let w = exp(-abs(dq - d0) / (0.05 * d0 + 0.05)) * (1.0 - 0.12 * f32(abs(x) + abs(y)));
            sum += textureLoad(ao_in, q, 0).r * w;
            wsum += w;
        }
    }
    return vec4<f32>(sum / max(wsum, 1e-4));
}
