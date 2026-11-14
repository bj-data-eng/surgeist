// One generic program: all stage and kernel payload choices are runtime data.
struct Parameters {
    capture: vec4<f32>,
    kernel: vec4<f32>,
    source_grid: vec4<i32>,
    result_grid: vec4<i32>,
    source_domain: vec4<f32>,
    result_domain: vec4<f32>,
    result_capture: vec4<f32>,
    order_target: vec4<u32>,
    stage_taps_edge: vec4<u32>,
    flags: vec4<u32>,
    bias: vec4<f32>,
    reserved: array<vec4<u32>, 5>,
}
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var auxiliary: texture_2d<f32>;
@group(0) @binding(2) var<uniform> parameters: Parameters;
@group(0) @binding(3) var<storage, read> weights: array<f32>;

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(positions[index], 0.0, 1.0);
}

// Native policy treats subnormal alpha as effective zero before any division.
const MIN_ALPHA: f32 = 1.1754943508222875e-38;
fn effective(value: vec4<f32>) -> vec4<f32> {
    if value.a < MIN_ALPHA { return vec4<f32>(0.0); }
    return value;
}
fn decode(value: vec3<f32>) -> vec3<f32> {
    let low = value / 12.92;
    let high = pow((value + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, value <= vec3<f32>(0.04045));
}
fn encode(value: vec3<f32>) -> vec3<f32> {
    let low = value * 12.92;
    let high = 1.055 * pow(max(value, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(high, low, value <= vec3<f32>(0.0031308));
}
fn to_working(value: vec4<f32>) -> vec4<f32> {
    let v = effective(value);
    if v.a == 0.0 { return v; }
    if parameters.flags.y != 0u { return vec4<f32>(decode(v.rgb / v.a) * v.a, v.a); }
    return v;
}
fn from_working(value: vec4<f32>) -> vec4<f32> {
    let v = effective(value);
    if v.a == 0.0 { return v; }
    if parameters.flags.y != 0u { return vec4<f32>(encode(v.rgb / v.a) * v.a, v.a); }
    return v;
}

// Euclidean remainder handles negative indices and arbitrarily many periods.
fn modulo(value: i32, period: i32) -> i32 {
    let remainder = value % period;
    return select(remainder, remainder + period, remainder < 0);
}
fn edge_index(value: i32, origin: i32, extent: i32) -> i32 {
    let relative = value - origin;
    switch parameters.stage_taps_edge.w {
        case 0u: { return origin + clamp(relative, 0, extent - 1); }
        case 1u: { return origin + modulo(relative, extent); }
        case 2u: {
            // Quotient parity implements period 2N without overflowing 2N.
            var block = relative / extent;
            var cell = relative % extent;
            if cell < 0 { cell += extent; block -= 1; }
            return origin + select(cell, extent - 1 - cell, modulo(block, 2) != 0);
        }
        default: { return value; }
    }
}
fn outside(index: vec2<i32>, origin: vec2<i32>, extent: vec2<i32>) -> bool {
    return any(index < origin) || any(index >= origin + extent);
}
fn coverage(point: vec2<f32>, step: vec2<f32>, domain: vec4<f32>) -> f32 {
    let minimum = max(point - step * 0.5, domain.xy);
    let maximum = min(point + step * 0.5, domain.zw);
    let overlap = max(maximum - minimum, vec2<f32>(0.0));
    return clamp(overlap.x / step.x, 0.0, 1.0) * clamp(overlap.y / step.y, 0.0, 1.0);
}

// Apply the semantic image-domain clip once to each raw capture cell,
// before bilinear reconstruction; capture allocation is only outward coverage.
fn capture_cell(index: vec2<i32>, convert: bool) -> vec4<f32> {
    let scale = parameters.capture.z;
    let domain = parameters.source_domain;
    // An empty semantic image has no coordinates to map or narrow to i32.
    if any(domain.zw <= domain.xy) { return vec4<f32>(0.0); }
    let low = vec2<i32>(floor((domain.xy - parameters.capture.xy) * scale));
    let high = vec2<i32>(ceil((domain.zw - parameters.capture.xy) * scale));
    let extent = high - low;
    if any(extent <= vec2<i32>(0)) { return vec4<f32>(0.0); }
    if !convert && outside(index, low, extent) { return vec4<f32>(0.0); }
    // Preparation clamps reconstruction to the finite capture raster. The
    // convolution edge mode is applied later, solely to the prepared grid.
    let mapped = clamp(index, low, high - vec2<i32>(1));
    if outside(mapped, vec2<i32>(0), vec2<i32>(textureDimensions(source))) { return vec4<f32>(0.0); }
    let center = parameters.capture.xy + (vec2<f32>(mapped) + vec2<f32>(0.5)) / scale;
    var value = textureLoad(source, mapped, 0);
    if convert { value = to_working(value); }
    return value * coverage(center, vec2<f32>(1.0 / scale), domain);
}
fn capture_bilinear(point: vec2<f32>, convert: bool) -> vec4<f32> {
    let coordinate = (point - parameters.capture.xy) * parameters.capture.z - vec2<f32>(0.5);
    let base = vec2<i32>(floor(coordinate));
    let fraction = fract(coordinate);
    let top = mix(capture_cell(base, convert), capture_cell(base + vec2<i32>(1, 0), convert), fraction.x);
    let bottom = mix(capture_cell(base + vec2<i32>(0, 1), convert), capture_cell(base + vec2<i32>(1, 1), convert), fraction.x);
    return mix(top, bottom, fraction.y);
}

// The finite prepared kernel source is the sole edge-extension domain.
fn kernel_cell(index: vec2<i32>) -> vec4<f32> {
    let grid = parameters.source_grid;
    if any(grid.zw <= vec2<i32>(0)) { return vec4<f32>(0.0); }
    if parameters.stage_taps_edge.w == 3u && outside(index, grid.xy, grid.zw) { return vec4<f32>(0.0); }
    let mapped = vec2<i32>(edge_index(index.x, grid.x, grid.z), edge_index(index.y, grid.y, grid.w));
    return textureLoad(source, mapped - grid.xy, 0);
}
fn result_cell(index: vec2<i32>) -> vec4<f32> {
    let grid = parameters.result_grid;
    if outside(index, grid.xy, grid.zw) { return vec4<f32>(0.0); }
    return textureLoad(source, index - grid.xy, 0);
}
fn result_bilinear(point: vec2<f32>) -> vec4<f32> {
    let coordinate = (point - parameters.kernel.xy) / parameters.kernel.zw - vec2<f32>(0.5);
    let base = vec2<i32>(floor(coordinate));
    let fraction = fract(coordinate);
    let top = mix(result_cell(base), result_cell(base + vec2<i32>(1, 0)), fraction.x);
    let bottom = mix(result_cell(base + vec2<i32>(0, 1)), result_cell(base + vec2<i32>(1, 1)), fraction.x);
    return mix(top, bottom, fraction.y);
}

@fragment
fn fragment_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(position.xy));
    switch parameters.stage_taps_edge.x {
        case 0u: {
            if any(parameters.source_grid.zw <= vec2<i32>(0)) { return vec4<f32>(0.0); }
            let index = parameters.source_grid.xy + pixel;
            let point = parameters.kernel.xy + (vec2<f32>(index) + vec2<f32>(0.5)) * parameters.kernel.zw;
            return capture_bilinear(point, true);
        }
        case 1u: {
            let index = parameters.result_grid.xy + pixel;
            var sum = vec4<f32>(0.0);
            if parameters.flags.z == 0u { sum = textureLoad(auxiliary, pixel, 0); }
            // One bounded batch; original row-major coefficient order is retained.
            for (var tap = 0u; tap < 32u; tap += 1u) {
                if tap >= parameters.stage_taps_edge.z { break; }
                let raw = parameters.stage_taps_edge.y + tap;
                let column = raw % parameters.order_target.x;
                let row = raw / parameters.order_target.x;
                let offset = vec2<i32>(parameters.order_target.xy - vec2<u32>(1u) - vec2<u32>(column, row)) - vec2<i32>(parameters.order_target.zw);
                var sample = kernel_cell(index + offset);
                if parameters.flags.x != 0u {
                    sample = effective(sample);
                    if sample.a != 0.0 { sample = vec4<f32>(sample.rgb / sample.a, 0.0); }
                }
                sum += sample * weights[raw];
            }
            return sum;
        }
        case 2u: {
            let sum = textureLoad(auxiliary, pixel, 0);
            let bias = parameters.bias.x;
            if parameters.flags.x != 0u {
                let alpha = effective(kernel_cell(parameters.result_grid.xy + pixel)).a;
                return vec4<f32>(clamp(sum.rgb + vec3<f32>(bias), vec3<f32>(0.0), vec3<f32>(1.0)) * alpha, alpha);
            }
            // RGB bias observes raw alpha before either component is clamped.
            let raw_alpha = sum.a + bias;
            let raw_rgb = sum.rgb + vec3<f32>(bias * raw_alpha);
            let alpha = clamp(raw_alpha, 0.0, 1.0);
            return vec4<f32>(clamp(raw_rgb, vec3<f32>(0.0), vec3<f32>(alpha)), alpha);
        }
        case 3u: {
            let point = parameters.result_capture.xy + position.xy / parameters.result_capture.z;
            let value = from_working(result_bilinear(point));
            return value * coverage(point, vec2<f32>(1.0 / parameters.result_capture.z), parameters.result_domain);
        }
        case 5u: {
            // Rebase a wider shared capture onto the semantic source capture
            // lattice. Preparation alone applies the source-domain clip.
            let point = parameters.result_capture.xy + position.xy / parameters.result_capture.z;
            let index = vec2<i32>(floor((point - parameters.capture.xy) * parameters.capture.z));
            if outside(index, vec2<i32>(0), vec2<i32>(textureDimensions(source))) { return vec4<f32>(0.0); }
            return textureLoad(source, index, 0);
        }
        default: {
            // Kernel-count recovery bypasses units and all color round trips.
            let point = parameters.result_capture.xy + position.xy / parameters.result_capture.z;
            return capture_bilinear(point, false) * coverage(point, vec2<f32>(1.0 / parameters.result_capture.z), parameters.result_domain);
        }
    }
}
