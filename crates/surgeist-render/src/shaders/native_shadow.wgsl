// Each entry point reaches only the textures in its concrete stage layout.
// All maps and clips are relative to real allocations, qualified by the host.
struct Parameters {
    sizes: vec4<u32>,
    source_base: vec4<i32>,
    source_clip: vec4<f32>,
    original_filter_clip: vec4<f32>,
    output_clip: vec4<f32>,
    periodic_map: vec4<f32>,
    flags: vec4<u32>,
    flood: vec4<f32>,
    color: vec4<f32>,
    opacity: vec4<f32>,
    reserved: array<vec4<u32>, 6>,
}
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var shifted_alpha: texture_2d<f32>;
@group(0) @binding(2) var<uniform> parameters: Parameters;

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(positions[index], 0.0, 1.0);
}
fn cell_coverage(pixel: vec2<i32>, clip: vec4<f32>) -> f32 {
    let minimum = max(vec2<f32>(pixel), clip.xy);
    let maximum = min(vec2<f32>(pixel) + vec2<f32>(1.0), clip.zw);
    let overlap = clamp(maximum - minimum, vec2<f32>(0.0), vec2<f32>(1.0));
    return overlap.x * overlap.y;
}
fn finite_original(pixel: vec2<i32>) -> vec4<f32> {
    let index = parameters.source_base.xy + pixel;
    if any(index < vec2<i32>(0)) || any(index >= vec2<i32>(parameters.sizes.xy)) {
        return vec4<f32>(0.0);
    }
    return textureLoad(source, index, 0) * cell_coverage(index, parameters.source_clip);
}
fn periodic_original(position: vec2<f32>, pixel: vec2<i32>) -> vec4<f32> {
    return periodic_sample(position) * cell_coverage(pixel, parameters.original_filter_clip);
}
fn constant_original(pixel: vec2<i32>) -> vec4<f32> {
    return parameters.color * cell_coverage(pixel, parameters.original_filter_clip);
}
fn merged(original: vec4<f32>, blurred_alpha: f32, pixel: vec2<i32>) -> vec4<f32> {
    // Public flood RGB is straight numeric-sRGB. Its alpha and raw opacity enter once.
    let h = blurred_alpha * (parameters.flood.a * parameters.opacity.x);
    var flood_rgb = parameters.flood.rgb;
    if parameters.flags.y != 0u { flood_rgb = decode(flood_rgb); }
    let shadow = vec4<f32>(flood_rgb * h, h);
    let value = from_working(to_working(original) + shadow * (1.0 - original.a));
    let alpha = clamp(value.a, 0.0, 1.0);
    let valid = vec4<f32>(clamp(value.rgb, vec3<f32>(0.0), vec3<f32>(alpha)), alpha);
    // Destination F/flood/merge membership is fused after source-over.
    return valid * cell_coverage(pixel, parameters.output_clip);
}
fn shadow_alpha(pixel: vec2<i32>) -> f32 {
    return textureLoad(shifted_alpha, pixel, 0).a;
}
@fragment
fn finite_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    return merged(finite_original(pixel), shadow_alpha(pixel), pixel);
}
@fragment
fn finite_original_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    return merged(finite_original(pixel), 0.0, pixel);
}
@fragment
fn periodic_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    return merged(periodic_original(p.xy, pixel), shadow_alpha(pixel), pixel);
}
@fragment
fn periodic_original_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    return merged(periodic_original(p.xy, pixel), 0.0, pixel);
}
@fragment
fn constant_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    return merged(constant_original(pixel), shadow_alpha(pixel), pixel);
}
@fragment
fn constant_original_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    return merged(constant_original(pixel), 0.0, pixel);
}
