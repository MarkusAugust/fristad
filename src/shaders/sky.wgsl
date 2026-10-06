// Himmelen: gradient, sol, og om natta stjerner, måne og nordlys.

struct SkyOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> SkyOut {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
    var o: SkyOut;
    o.clip = vec4<f32>(p, 1.0, 1.0);
    o.ndc = p;
    return o;
}

fn stars(dir: vec3<f32>) -> vec3<f32> {
    // Retningen deles i små celler. Noen få celler har en stjerne.
    let d = dir * 220.0;
    let cell = floor(d);
    let h = hash31(cell);
    if (h > 0.985) {
        let center = cell + 0.5 + (vec3<f32>(hash31(cell + 1.0), hash31(cell + 2.0), hash31(cell + 3.0)) - 0.5) * 0.6;
        let dist = length(d - center);
        let tw = 0.75 + 0.25 * sin(g.camera.w * (1.0 + h * 3.0) + h * 50.0);
        let size = smoothstep(0.38, 0.0, dist);
        let tint = mix(vec3<f32>(0.75, 0.85, 1.0), vec3<f32>(1.0, 0.9, 0.75), hash31(cell + 9.0));
        return tint * size * tw * (h - 0.985) * 260.0;
    }
    return vec3<f32>(0.0);
}

fn aurora(dir: vec3<f32>) -> vec3<f32> {
    if (dir.y < 0.02) {
        return vec3<f32>(0.0);
    }
    // Lysgardiner: bånd projisert opp på et høyt plan, med bølger i tid.
    var acc = vec3<f32>(0.0);
    let t = g.camera.w * 0.05;
    for (var i = 0; i < 6; i++) {
        let fi = f32(i);
        let height = 1.0 + fi * 0.06;
        let p = dir.xz / (dir.y + 0.35) * height;
        let wave = p.x * 0.9 + sin(p.y * 1.4 + t * 3.0 + fi * 0.3) * 0.6 + fbm2(p * 0.8 + t) * 1.2;
        let x = (wave - 0.6) * 3.2;
        let band = exp(-x * x);
        let streak = 0.55 + 0.45 * noise2(vec2<f32>(p.y * 9.0 + fi, t * 8.0));
        let col = mix(vec3<f32>(0.1, 1.0, 0.55), vec3<f32>(0.45, 0.25, 1.0), fi / 6.0);
        acc += col * band * streak * (1.0 - fi / 7.0);
    }
    let fade = smoothstep(0.02, 0.25, dir.y) * smoothstep(1.0, 0.45, dir.y);
    return acc * fade * 0.045;
}

@fragment
fn fs(i: SkyOut) -> @location(0) vec4<f32> {
    let far = g.inv_view_proj * vec4<f32>(i.ndc, 1.0, 1.0);
    let near = g.inv_view_proj * vec4<f32>(i.ndc, 0.0, 1.0);
    let dir = normalize(far.xyz / far.w - near.xyz / near.w);
    let night = g.sun_color.w;

    var c = sky_gradient(dir);

    // Skyer: et lag med støy projisert på et høyt, flatt plan.
    if (dir.y > 0.0) {
        let p = dir.xz / (dir.y + 0.08) * 1.6 + vec2<f32>(g.camera.w * 0.004, 0.0);
        let shape = fbm2(p * 0.9) * 0.65 + fbm2(p * 2.7 + 4.0) * 0.35;
        let cover = smoothstep(0.5, 0.78, shape) * smoothstep(0.0, 0.22, dir.y);
        let sunward = pow(max(dot(dir, g.sun_dir.xyz), 0.0), 3.0);
        let lit = mix(g.sky_horizon.rgb * 1.15 + g.sun_color.rgb * 0.25, g.sun_color.rgb * 1.2, sunward * 0.5);
        let shade = mix(lit, g.sky_top.rgb * 0.9, smoothstep(0.62, 0.95, shape) * 0.35);
        c = mix(c, shade * (1.0 - night * 0.85), cover * 0.9);
    }

    // Sola om dagen, månen om natta: begge er en skive i sun_dir.
    let s = dot(dir, g.sun_dir.xyz);
    let disk = smoothstep(0.99985, 0.99995, s);
    let sun_disk = g.sun_color.rgb * disk * 40.0 * (1.0 - night);
    let moon = vec3<f32>(0.85, 0.9, 1.0) * smoothstep(0.99955, 0.99968, s) * 3.0 * night;
    c += sun_disk + moon;

    if (night > 0.0) {
        c += (stars(dir) * smoothstep(0.0, 0.15, dir.y) + aurora(dir)) * night;
    }
    return vec4<f32>(c, 1.0);
}
