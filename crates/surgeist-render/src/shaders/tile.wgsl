struct TileParameters {
    output_clip: vec4<f32>,
    phase_step: vec4<f32>,
    period_window: vec4<u32>,
    source_extent: vec4<u32>,
    source_clip: vec4<f32>,
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var<uniform> parameters: TileParameters;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[index], 0.0, 1.0);
}

fn cell_coverage(index: vec2<i32>, clip: vec4<f32>) -> f32 {
    let cell_minimum = vec2<f32>(index);
    let minimum = max(cell_minimum, clip.xy);
    let maximum = min(cell_minimum + vec2<f32>(1.0), clip.zw);
    let overlap = clamp(maximum - minimum, vec2<f32>(0.0), vec2<f32>(1.0));
    return overlap.x * overlap.y;
}

fn modulo(index: i32, period: i32) -> i32 {
    let remainder = index % period;
    if remainder < 0 { return remainder + period; }
    return remainder;
}

fn capture_sample(coordinate: vec2<f32>, extent: vec2<i32>) -> vec4<f32> {
    let base = vec2<i32>(floor(coordinate));
    let upper = min(base + vec2<i32>(1), extent - vec2<i32>(1));
    let fraction = fract(coordinate);
    let top = mix(textureLoad(source, base, 0),
        textureLoad(source, vec2<i32>(upper.x, base.y), 0), fraction.x);
    let bottom = mix(textureLoad(source, vec2<i32>(base.x, upper.y), 0),
        textureLoad(source, upper, 0), fraction.x);
    return mix(top, bottom, fraction.y);
}

fn periodic_cell(index: vec2<i32>) -> vec4<f32> {
    let period = vec2<i32>(parameters.period_window.xy);
    let wrapped = vec2<i32>(modulo(index.x, period.x), modulo(index.y, period.y));
    // The full period determines phase. Cells outside the stored hard-clipped
    // window contribute transparent black to the wrapped interpolation.
    let local = wrapped - vec2<i32>(parameters.period_window.zw);
    let extent = vec2<i32>(parameters.source_extent.xy);
    if any(local < vec2<i32>(0)) || any(local >= extent) {
        return vec4<f32>(0.0);
    }

    let clip = parameters.source_clip;
    let size = vec2<f32>(extent);
    let aligned = vec2<bool>(clip.x == 0.0 && clip.z == size.x, clip.y == 0.0 && clip.w == size.y);
    if all(aligned) { return textureLoad(source, local, 0); }

    let pixel = vec2<f32>(local);
    let minimum = max(pixel, clip.xy);
    let maximum = min(pixel + vec2<f32>(1.0), clip.zw);
    let overlap = clamp(maximum - minimum, vec2<f32>(0.0), vec2<f32>(1.0));
    let coverage = overlap.x * overlap.y;
    let clip_size = clip.zw - clip.xy;
    if coverage == 0.0 || any(clip_size < vec2<f32>(1.1754943508222875e-38)) {
        return vec4<f32>(0.0);
    }

    // The capture viewport covers only the exact hard domain. Reconstruct the
    // contributing part of this conceptual period cell on that viewport, then
    // apply its coverage once.
    let relative_centroid = (minimum - clip.xy) + overlap * 0.5;
    let mapped = relative_centroid / clip_size * size - vec2<f32>(0.5);
    let coordinate = select(clamp(mapped, vec2<f32>(0.0), size - vec2<f32>(1.0)), pixel, aligned);
    return capture_sample(coordinate, extent) * coverage;
}

@fragment
fn fs_repeat(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(position.xy));
    let coverage = cell_coverage(pixel, parameters.output_clip);
    if coverage == 0.0 || any(parameters.period_window.xy == vec2<u32>(0u)) {
        return vec4<f32>(0.0);
    }

    // Match the frame owner's qualified multiply/add order. The uploaded phase
    // already includes the half-texel shift at the first output cell center.
    let scaled = vec2<f32>(pixel) * parameters.phase_step.zw;
    let coordinate = scaled + parameters.phase_step.xy;
    let base = vec2<i32>(floor(coordinate));
    let fraction = fract(coordinate);
    let top = mix(periodic_cell(base), periodic_cell(base + vec2<i32>(1, 0)), fraction.x);
    let bottom = mix(periodic_cell(base + vec2<i32>(0, 1)),
        periodic_cell(base + vec2<i32>(1, 1)), fraction.x);
    return mix(top, bottom, fraction.y) * coverage;
}

@fragment
fn fs_transparent() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0);
}
