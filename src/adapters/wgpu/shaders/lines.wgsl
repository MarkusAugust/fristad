// Linjene redigeringen tegner over bildet (det valgte, regelbruddene,
// håndtakene og linjeutkastet): ensfargede, uten dybdetest, så de synes
// gjennom veggene.

struct LineOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs(@location(0) pos: vec3<f32>, @location(1) color: vec3<f32>) -> LineOut {
    var o: LineOut;
    o.clip = g.view_proj * vec4<f32>(pos, 1.0);
    o.color = color;
    return o;
}

@fragment
fn fs(i: LineOut) -> @location(0) vec4<f32> {
    return vec4<f32>(i.color, 1.0);
}
