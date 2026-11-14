// Integer maps, endpoints, levels and all reached f32 arithmetic are qualified
// against the real allocations before this exact 256-byte wire is uploaded.
struct Parameters {
    sizes: vec4<u32>,
    source_base: vec4<i32>,
    source_clip: vec4<f32>,
    output_clip: vec4<f32>,
    periodic_map: vec4<f32>,
    control: vec4<u32>,
    maps: vec4<i32>,
    flags: vec4<u32>,
    color: vec4<f32>,
    query: vec4<u32>,
    reserved: array<vec4<u32>, 6>,
}
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var previous_query: texture_2d<f32>;
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
fn direct_cell(index: vec2<i32>) -> vec4<f32> {
    if any(index < vec2<i32>(0)) || any(index >= vec2<i32>(parameters.sizes.xy)) {
        return vec4<f32>(0.0);
    }
    return textureLoad(source, index, 0);
}
fn extrema(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    if parameters.flags.z == 0u { return min(a, b); }
    return max(a, b);
}
fn final_color(value: vec4<f32>) -> vec4<f32> {
    var converted = value;
    if parameters.query.z != 0u { converted = from_working(converted); }
    let alpha = clamp(converted.a, 0.0, 1.0);
    return vec4<f32>(clamp(converted.rgb, vec3<f32>(0.0), vec3<f32>(alpha)), alpha);
}
fn axis_coordinate(pixel: vec2<i32>) -> i32 {
    return pixel[parameters.flags.w];
}
fn with_axis(pixel: vec2<i32>, coordinate: i32) -> vec2<i32> {
    var index = pixel + parameters.source_base.xy;
    index[parameters.flags.w] = coordinate;
    return index;
}
fn endpoint(j: i32, kind: i32, delta: i32) -> i32 {
    if kind == 0 { return 0; }
    if kind == 1 { return i32(parameters.control.w); }
    // The host proves j+delta before this addition, not merely after clamping.
    return clamp(j + delta, 0, i32(parameters.control.w));
}
// Returns a start and exclusive end. For a periodic interval the end may exceed
// N by less than N; only the two selected table addresses are wrapped.
fn interval(pixel: vec2<i32>) -> vec2<i32> {
    let j = axis_coordinate(pixel);
    if j < parameters.source_base.z || j >= parameters.source_base.w {
        return vec2<i32>(0);
    }
    if parameters.control.z != 0u {
        let start = modulo(j + parameters.maps.y, i32(parameters.control.w));
        return vec2<i32>(start, start + i32(parameters.query.x));
    }
    return vec2<i32>(endpoint(j, parameters.maps.x, parameters.maps.y), endpoint(j, parameters.maps.z, parameters.maps.w));
}
fn selected_interval(pixel: vec2<i32>, bounds: vec2<i32>) -> vec4<f32> {
    var last = bounds.y - i32(parameters.control.x);
    if parameters.control.z != 0u { last = modulo(last, i32(parameters.control.w)); }
    return extrema(textureLoad(source, with_axis(pixel, bounds.x), 0), textureLoad(source, with_axis(pixel, last), 0));
}
@fragment
fn rebase_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return direct_cell(vec2<i32>(floor(p.xy)) + parameters.source_base.xy);
}
@fragment
fn copy_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    let coverage = cell_coverage(pixel, parameters.output_clip);
    if coverage == 0.0 { return vec4<f32>(0.0); }
    if parameters.control.z != 0u { return periodic_sample(p.xy) * coverage; }
    let index = pixel + parameters.source_base.xy;
    return direct_cell(index) * cell_coverage(index, parameters.source_clip) * coverage;
}
@fragment
fn prepare_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let index = vec2<i32>(floor(p.xy)) + parameters.source_base.xy;
    var coverage = 1.0;
    if parameters.control.z == 0u { coverage = cell_coverage(index, parameters.source_clip); }
    return to_working(direct_cell(index) * coverage);
}
@fragment
fn table_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let first = vec2<i32>(floor(p.xy));
    var second = first;
    let axis = parameters.flags.w;
    second[axis] += i32(parameters.control.x / 2u);
    if parameters.control.z != 0u { second[axis] = modulo(second[axis], i32(parameters.control.w)); }
    return extrema(textureLoad(source, first, 0), textureLoad(source, second, 0));
}
@fragment
fn first_query_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    let bounds = interval(pixel);
    if bounds.y - bounds.x != 1 { return vec4<f32>(0.0); }
    return textureLoad(source, with_axis(pixel, bounds.x), 0);
}
@fragment
fn refine_query_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    let prior = textureLoad(previous_query, pixel, 0);
    let bounds = interval(pixel);
    let length = bounds.y - bounds.x;
    if length <= 0 { return prior; }
    // firstLeadingBit is reached only for a strictly positive interval length.
    if firstLeadingBit(u32(length)) != parameters.control.y { return prior; }
    return selected_interval(pixel, bounds);
}
@fragment
fn query_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    let bounds = interval(pixel);
    if bounds.y <= bounds.x { return vec4<f32>(0.0); }
    return selected_interval(pixel, bounds);
}
@fragment
fn resolve_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(p.xy));
    let coverage = cell_coverage(pixel, parameters.output_clip);
    if coverage == 0.0 { return vec4<f32>(0.0); }
    if parameters.control.z != 0u { return final_color(periodic_sample(p.xy)) * coverage; }
    return final_color(direct_cell(pixel + parameters.source_base.xy)) * coverage;
}
@fragment
fn constant_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return parameters.color * cell_coverage(vec2<i32>(floor(p.xy)), parameters.output_clip);
}
