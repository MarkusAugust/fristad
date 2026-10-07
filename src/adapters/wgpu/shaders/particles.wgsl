// Regn og snø: instansierte små firkanter i en boks rundt kameraet.
// Posisjonen regnes ut i vertex-shaderen fra instansnummeret og tida, så
// ingenting må oppdateres fra CPU-en.

struct Pass {
    // x 0 regn / 1 snø, y mengde 0..1.
    params: vec4<f32>,
}
@group(1) @binding(4) var<uniform> pass_u: Pass;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha: f32,
}

const BOX: vec3<f32> = vec3<f32>(28.0, 16.0, 28.0);

/// Heltallshash (lowbias32) til 0..1.
fn ihash(x: u32) -> f32 {
    var h = x;
    h ^= h >> 16u;
    h *= 0x7feb352du;
    h ^= h >> 15u;
    h *= 0x846ca68bu;
    h ^= h >> 16u;
    return f32(h & 0xffffffu) / 16777216.0;
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let snow = pass_u.params.x > 0.5;
    let fi = f32(ii);
    // Heltallshash per flak: den flyttallsbaserte hash21 på en tellende
    // indeks ga et gitter, og snøen falt i firkantede klumper.
    let h1 = ihash(ii * 4u + 1u);
    let h2 = ihash(ii * 4u + 2u);
    let h3 = ihash(ii * 4u + 3u);
    let t = g.camera.w;
    let wind = g.jitter.w;
    var speed = 9.0 + h3 * 3.0;
    if (snow) {
        speed = 0.9 + h3 * 0.6;
    }
    // Fall: y går nedover med tida, og brettes inn i boksen.
    var p = vec3<f32>(h1 * BOX.x, h2 * BOX.y, ihash(ii * 4u) * BOX.z);
    p.y = BOX.y - fract((p.y / BOX.y) + t * speed / BOX.y) * BOX.y;
    var drift = vec3<f32>(wind * 2.0, 0.0, wind * 0.6) * (p.y / speed);
    if (snow) {
        drift += vec3<f32>(sin(t * 1.3 + fi) * 0.6, 0.0, cos(t * 1.1 + fi * 0.7) * 0.6);
    }
    p += drift;
    // Boksen følger kameraet, brettet så dråpene ikke hopper.
    let cam = g.camera.xyz;
    p.x = cam.x - BOX.x * 0.5 + fract((p.x - cam.x) / BOX.x + 0.5) * BOX.x;
    p.z = cam.z - BOX.z * 0.5 + fract((p.z - cam.z) / BOX.z + 0.5) * BOX.z;
    p.y = cam.y - 4.0 + p.y;

    // Firkanten: for regn en strek langs fallretningen, for snø en liten lapp.
    let corner = vec2<f32>(f32(vi & 1u), f32((vi >> 1u) & 1u)) * 2.0 - 1.0;
    let to_cam = normalize(cam - p);
    let right = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), to_cam));
    var fall = normalize(vec3<f32>(-wind * 0.25, 1.0, -wind * 0.07));
    var size = vec2<f32>(0.006, 0.22);
    if (snow) {
        size = vec2<f32>(0.02 + h3 * 0.02, 0.02 + h3 * 0.02);
        fall = cross(to_cam, right);
    }
    let wp = p + right * corner.x * size.x + fall * corner.y * size.y;
    var o: VsOut;
    o.clip = g.view_proj * vec4<f32>(wp, 1.0);
    o.uv = corner;
    // Nære dråper er tydeligere; bruk mengden som andel som er synlig.
    let dist = distance(p, cam);
    o.alpha = step(h1 * 0.999, pass_u.params.y) * smoothstep(30.0, 6.0, dist);
    return o;
}

@fragment
fn fs(i: VsOut) -> @location(0) vec4<f32> {
    let snow = pass_u.params.x > 0.5;
    var a = 0.0;
    if (snow) {
        a = smoothstep(1.0, 0.3, length(i.uv)) * 0.9;
    } else {
        a = smoothstep(1.0, 0.0, abs(i.uv.x)) * smoothstep(1.0, 0.6, abs(i.uv.y)) * 0.35;
    }
    let night = g.sun_color.w;
    let light = mix(g.sky_horizon.rgb * 1.6 + g.sun_color.rgb * 0.3, vec3<f32>(0.06, 0.07, 0.1), night);
    var col = light;
    if (snow) {
        col = mix(vec3<f32>(0.9, 0.92, 0.97), vec3<f32>(0.25, 0.27, 0.33), night);
    }
    // Lyktene lyser opp dråpene om natta.
    let count = i32(g.params.y);
    var lamp = vec3<f32>(0.0);
    for (var k = 0; k < count; k++) {
        let li = g.lights[k];
        let d = distance(li.pos_radius.xyz, g.camera.xyz);
        lamp += li.color_intensity.rgb * li.color_intensity.w * 0.02 / (1.0 + d * d * 0.3);
    }
    col += lamp;
    return vec4<f32>(col, a * i.alpha);
}
