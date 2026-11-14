// Shared qualified wrapped-bilinear reconstruction of an actual period canvas.
fn modulo(value: i32, period: i32) -> i32 {
    let remainder = value % period;
    // Conditional addition avoids overflowing a positive remainder + period.
    if remainder < 0 { return remainder + period; }
    return remainder;
}
fn periodic_cell(index: vec2<i32>) -> vec4<f32> {
    let mapped = vec2<i32>(modulo(index.x, i32(parameters.sizes.x)), modulo(index.y, i32(parameters.sizes.y)));
    return textureLoad(source, mapped, 0);
}
fn periodic_sample(position: vec2<f32>) -> vec4<f32> {
    if any(parameters.sizes.xy == vec2<u32>(0u)) { return vec4<f32>(0.0); }
    // Keep this ordering in agreement with the host f32 arithmetic proof.
    let scaled = position * parameters.periodic_map.zw;
    let translated = scaled + parameters.periodic_map.xy;
    let coordinate = translated - vec2<f32>(0.5);
    let base = vec2<i32>(floor(coordinate));
    let fraction = fract(coordinate);
    let top = mix(periodic_cell(base), periodic_cell(base + vec2<i32>(1, 0)), fraction.x);
    let bottom = mix(periodic_cell(base + vec2<i32>(0, 1)), periodic_cell(base + vec2<i32>(1, 1)), fraction.x);
    return mix(top, bottom, fraction.y);
}
