// All coordinates are rebased texel indices or texel-boundary rectangles.
struct OffsetParameters {
    translation_ratio: vec4<f32>,
    source_clip: vec4<f32>,
    result_clip: vec4<f32>,
    extents: vec4<u32>,
    modes: vec4<u32>,
    reserved: array<vec4<u32>, 11>,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var<uniform> parameters: OffsetParameters;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn cell_coverage(cell_minimum: vec2<f32>, clip: vec4<f32>) -> f32 {
    let extent = max(min(cell_minimum + vec2<f32>(1.0), clip.zw)
        - max(cell_minimum, clip.xy), vec2<f32>(0.0));
    return extent.x * extent.y;
}

fn source_cell(index: vec2<i32>) -> vec4<f32> {
    // Check the real allocation before any load, then mask the finite D cell.
    if (any(index < vec2<i32>(0)) || any(index >= vec2<i32>(textureDimensions(source_texture)))) {
        return vec4<f32>(0.0);
    }
    let value = textureLoad(source_texture, index, 0);
    // The native shadow producer has already applied F coverage to this cell.
    if parameters.modes.z != 0u { return value; }
    return value * cell_coverage(vec2<f32>(index), parameters.source_clip);
}

fn reconstruct(coordinate: vec2<f32>) -> vec4<f32> {
    if (parameters.modes.x == 0u) {
        // Frame qualification proves every reached coordinate is integral.
        return source_cell(vec2<i32>(coordinate));
    }
    if (parameters.modes.x == 1u) {
        return source_cell(vec2<i32>(floor(coordinate + vec2<f32>(0.5))));
    }
    let base = vec2<i32>(floor(coordinate));
    if (parameters.modes.x == 2u) {
        let fraction = coordinate - vec2<f32>(base);
        let top = mix(source_cell(base), source_cell(base + vec2<i32>(1, 0)), fraction.x);
        let bottom = mix(source_cell(base + vec2<i32>(0, 1)),
            source_cell(base + vec2<i32>(1, 1)), fraction.x);
        return mix(top, bottom, fraction.y);
    }
    var result = vec4<f32>(0.0);
    for (var y = -1; y <= 2; y += 1) {
        let tap_y = base.y + y;
        let weight_y = mitchell_netravali(coordinate.y - f32(tap_y));
        for (var x = -1; x <= 2; x += 1) {
            let tap_x = base.x + x;
            let weight_x = mitchell_netravali(coordinate.x - f32(tap_x));
            result += source_cell(vec2<i32>(tap_x, tap_y)) * (weight_x * weight_y);
        }
    }
    let alpha = clamp(result.a, 0.0, 1.0);
    return vec4<f32>(clamp(result.rgb, vec3<f32>(0.0), vec3<f32>(alpha)), alpha);
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let coordinate = parameters.translation_ratio.xy
        + (input.position.xy - vec2<f32>(0.5)) * parameters.translation_ratio.zw;
    // Frame qualification proves finite reachable arithmetic. This guard must
    // precede floor/i32 conversion even for a partly intersecting large map.
    if (any(coordinate < vec2<f32>(-2.0))
        || any(coordinate > vec2<f32>(parameters.extents.xy) + vec2<f32>(1.0))) {
        return vec4<f32>(0.0);
    }
    if (parameters.modes.y != 0u) {
        // Aligned lattice rebase: raw integer copy, without either coverage.
        let index = vec2<i32>(coordinate);
        if (any(index < vec2<i32>(0)) || any(index >= vec2<i32>(textureDimensions(source_texture)))) {
            return vec4<f32>(0.0);
        }
        return textureLoad(source_texture, index, 0);
    }
    let reconstructed = reconstruct(coordinate);
    if parameters.modes.z != 0u {
        // Clamp signed cubic alpha at the sampling owner, then defer O coverage
        // until the original-over-shadow merge has completed.
        return vec4<f32>(0.0, 0.0, 0.0, clamp(reconstructed.a, 0.0, 1.0));
    }
    return reconstructed
        * cell_coverage(input.position.xy - vec2<f32>(0.5), parameters.result_clip);
}
