// All map coordinates and displacement offsets are host-qualified integers.
struct Parameters {
    sizes: vec4<u32>,
    source_base: vec4<i32>,
    source_clip: vec4<f32>,
    output_clip: vec4<f32>,
    periodic_map: vec4<f32>,
    color: vec4<f32>,
    control: vec4<u32>,
    map_size: vec4<u32>,
    reserved: array<vec4<u32>, 8>,
}
struct Address { q0: i32, outside: u32 }
struct MapRecord { pixel: u32, coverage: f32, outside: u32, padding: u32 }
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var raw_map: texture_2d<f32>;
@group(0) @binding(2) var<uniform> parameters: Parameters;
@group(0) @binding(3) var<storage, read> addresses: array<Address>;
@group(0) @binding(4) var<storage, read> records: array<MapRecord>;

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(positions[index], 0.0, 1.0);
}
fn coverage(pixel: vec2<i32>, clip: vec4<f32>) -> f32 {
    let minimum = max(vec2<f32>(pixel), clip.xy);
    let maximum = min(vec2<f32>(pixel) + vec2<f32>(1.0), clip.zw);
    let overlap = clamp(maximum - minimum, vec2<f32>(0.0), vec2<f32>(1.0));
    return overlap.x * overlap.y;
}
fn source_cell(index: vec2<i32>) -> vec4<f32> {
    if any(index < vec2<i32>(0)) || any(index >= vec2<i32>(parameters.sizes.xy)) { return vec4<f32>(0.0); }
    return textureLoad(source, index, 0);
}
@fragment
fn realize_image_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    let index = pixel + parameters.source_base.xy;
    return source_cell(index) * coverage(index, parameters.source_clip);
}
@fragment
fn realize_periodic_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return periodic_sample(p.xy) * coverage(vec2<i32>(floor(p.xy)), parameters.source_clip);
}
@fragment
fn realize_constant_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return parameters.color * coverage(vec2<i32>(floor(p.xy)), parameters.source_clip);
}
@fragment
fn passthrough_image_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    let index = pixel + parameters.source_base.xy;
    return source_cell(index) * coverage(index, parameters.source_clip) * coverage(pixel, parameters.output_clip);
}
@fragment
fn passthrough_periodic_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    return periodic_sample(p.xy) * coverage(pixel, parameters.source_clip) * coverage(pixel, parameters.output_clip);
}
@fragment
fn passthrough_constant_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    return parameters.color * coverage(pixel, parameters.source_clip) * coverage(pixel, parameters.output_clip);
}
fn displaced(pixel: vec2<i32>, channels: vec2<u32>) -> vec4<f32> {
    let x = addresses[channels.x];
    let y = addresses[256u + channels.y];
    if x.outside != 0u || y.outside != 0u { return vec4<f32>(0.0); }
    // The host proves both sums before this integer addition.
    let index = vec2<i32>(x.q0, y.q0) + pixel;
    return source_cell(index) * coverage(pixel, parameters.output_clip);
}
fn map_bytes(pixel: vec2<i32>) -> vec4<u32> {
    let x = records[u32(pixel.x)];
    let y = records[parameters.control.w + u32(pixel.y)];
    if x.outside != 0u || y.outside != 0u || x.pixel >= parameters.map_size.x || y.pixel >= parameters.map_size.y { return vec4<u32>(0u); }
    let covered = x.coverage * y.coverage;
    if covered <= 0.0 { return vec4<u32>(0u); }
    let raw = vec4<u32>(floor(textureLoad(raw_map, vec2<i32>(i32(x.pixel), i32(y.pixel)), 0) * 255.0 + vec4<f32>(0.5)));
    if raw.a == 0u { return vec4<u32>(0u); }
    let covered_alpha = f32(raw.a) * covered;
    let alpha = u32(floor(covered_alpha + 0.5));
    return vec4<u32>(raw.rgb, alpha);
}
@fragment
fn displace_raw_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    let map = map_bytes(pixel);
    return displaced(pixel, vec2<u32>(map[u32(parameters.source_base.z)], map[u32(parameters.source_base.w)]));
}
@fragment
fn displace_zero_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return displaced(vec2<i32>(floor(p.xy)), vec2<u32>(0u));
}
@fragment
fn transparent_main() -> @location(0) vec4<f32> { return vec4<f32>(0.0); }
