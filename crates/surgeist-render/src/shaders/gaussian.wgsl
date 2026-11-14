// Rebased texel maps are qualified by the frame owner before upload.
struct Parameters {
    sizes: vec4<u32>,
    source_base: vec4<i32>,
    source_clip: vec4<f32>,
    output_clip: vec4<f32>,
    periodic_map: vec4<f32>,
    stage_taps: vec4<u32>,
    modes: vec4<u32>,
    flags: vec4<u32>,
    color: vec4<f32>,
    reserved: array<vec4<u32>, 7>,
}
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var auxiliary: texture_2d<f32>;
@group(0) @binding(2) var<uniform> parameters: Parameters;
// Existing Gaussian owner serializes complete (signed offset, weight) records.
@group(0) @binding(3) var<storage, read> kernel: array<vec2<f32>>;

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(positions[index], 0.0, 1.0);
}
fn cell_coverage(pixel: vec2<i32>, clip: vec4<f32>) -> f32 {
    if any(clip.zw <= clip.xy) { return 0.0; }
    let minimum = max(vec2<f32>(pixel), clip.xy);
    let maximum = min(vec2<f32>(pixel) + vec2<f32>(1.0), clip.zw);
    let overlap = clamp(maximum - minimum, vec2<f32>(0.0), vec2<f32>(1.0));
    return overlap.x * overlap.y;
}
fn outside(index: vec2<i32>) -> bool {
    return any(index < vec2<i32>(0)) || any(index >= vec2<i32>(parameters.sizes.xy));
}
fn modulo(value: i32, period: i32) -> i32 {
    let remainder = value % period;
    // Conditional addition avoids overflowing a positive remainder + period.
    if remainder < 0 { return remainder + period; }
    return remainder;
}
fn edge_index(value: i32, extent: i32) -> i32 {
    switch parameters.modes.x {
        case 0u: { return clamp(value, 0, extent - 1); }
        case 1u: { return modulo(value, extent); }
        case 2u: {
            var block = value / extent;
            var cell = value % extent;
            if cell < 0 { cell += extent; block -= 1; }
            if modulo(block, 2) != 0 { return extent - 1 - cell; }
            return cell;
        }
        default: { return value; }
    }
}
fn direct_cell(index: vec2<i32>) -> vec4<f32> {
    if outside(index) { return vec4<f32>(0.0); }
    return textureLoad(source, index, 0);
}
fn original_cell(index: vec2<i32>) -> vec4<f32> {
    if any(parameters.sizes.xy == vec2<u32>(0u)) { return vec4<f32>(0.0); }
    if parameters.modes.z != 0u { return direct_cell(index); }
    if parameters.modes.x == 3u && outside(index) { return vec4<f32>(0.0); }
    let mapped = vec2<i32>(edge_index(index.x, i32(parameters.sizes.x)), edge_index(index.y, i32(parameters.sizes.y)));
    return textureLoad(source, mapped, 0);
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
fn final_color(value: vec4<f32>) -> vec4<f32> {
    var converted = value;
    if parameters.flags.x != 0u { converted = from_working(converted); }
    let alpha = clamp(converted.a, 0.0, 1.0);
    return vec4<f32>(clamp(converted.rgb, vec3<f32>(0.0), vec3<f32>(alpha)), alpha);
}
@fragment
fn constant_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return parameters.color * cell_coverage(vec2<i32>(floor(position.xy)), parameters.output_clip);
}
@fragment
fn source_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(position.xy));
    let stage = parameters.stage_taps.x;
    if stage == 5u { return parameters.color * cell_coverage(pixel, parameters.output_clip); }
    var output_coverage = 1.0;
    if stage == 1u || stage == 4u {
        output_coverage = cell_coverage(pixel, parameters.output_clip);
        if output_coverage == 0.0 { return vec4<f32>(0.0); }
    }
    if any(parameters.sizes.xy == vec2<u32>(0u)) { return vec4<f32>(0.0); }
    if (stage == 1u || stage == 4u) && parameters.modes.y != 0u {
        let sampled = periodic_sample(position.xy);
        if stage == 1u { return sampled * output_coverage; }
        return final_color(sampled) * output_coverage;
    }
    let index = parameters.source_base.xy + pixel;
    switch stage {
        case 0u: { return direct_cell(index); }
        case 1u: {
            if outside(index) { return vec4<f32>(0.0); }
            return direct_cell(index) * cell_coverage(index, parameters.source_clip) * output_coverage;
        }
        case 2u: {
            if outside(index) { return vec4<f32>(0.0); }
            var input_coverage = 1.0;
            if parameters.modes.y == 0u { input_coverage = cell_coverage(index, parameters.source_clip); }
            if input_coverage == 0.0 { return vec4<f32>(0.0); }
            return to_working(direct_cell(index)) * input_coverage;
        }
        default: { return final_color(direct_cell(index)) * output_coverage; }
    }
}

@fragment
fn fragment_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(position.xy));
    if any(parameters.sizes.xy == vec2<u32>(0u)) { return vec4<f32>(0.0); }
    let index = parameters.source_base.xy + pixel;
            var sum = vec4<f32>(0.0);
            if parameters.modes.w == 0u { sum = textureLoad(auxiliary, pixel, 0); }
            for (var tap = 0u; tap < 32u; tap += 1u) {
                if tap >= parameters.stage_taps.z { break; }
                // Binding 3 exposes only this checked, contiguous record range.
                let record = kernel[tap];
                var sample_index = index;
                if parameters.stage_taps.w == 0u { sample_index.x += i32(record.x); }
                else { sample_index.y += i32(record.x); }
                sum += original_cell(sample_index) * record.y;
            }
            return sum;
}
