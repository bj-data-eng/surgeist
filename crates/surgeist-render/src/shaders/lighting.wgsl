// Layout is shared by all stages. Coordinates and arithmetic are qualified by the host.
struct Parameters {
    sizes: vec4<u32>,
    flags: vec4<u32>,
    sample_map: vec4<f32>,
    clip: vec4<f32>,
    node_base_step: vec4<f32>,
    surface_material: vec4<f32>,
    incident: vec4<f32>,
    spot_axis: vec4<f32>,
    spot_cone: vec4<f32>,
    light_color: vec4<f32>,
    shade_to_bump: vec4<i32>,
    light_anchor: vec2<i32>,
    height_anchor: f32,
    material_exponent: f32,
    reserved: array<vec4<u32>, 4>,
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var<uniform> parameters: Parameters;

const X_MIN: u32 = 1u;
const X_MAX: u32 = 2u;
const Y_MIN: u32 = 4u;
const Y_MAX: u32 = 8u;

const LIGHT_DISTANT: u32 = 0u;
const LIGHT_SPOT: u32 = 2u;
const LIGHT_NONE: u32 = 3u;
const CONE_ZERO: u32 = 1u;
const CONE_POSITIVE: u32 = 2u;

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(positions[index], 0.0, 1.0);
}

fn sample_coordinate(pixel: vec2<i32>) -> vec2<f32> {
    let scaled = vec2<f32>(pixel) * parameters.sample_map.zw;
    return scaled + parameters.sample_map.xy;
}

fn effective_strength(value: f32) -> f32 {
    return select(0.0, value, value >= MIN_ALPHA);
}

fn cell_coverage(pixel: vec2<i32>) -> f32 {
    let minimum = max(vec2<f32>(pixel), parameters.clip.xy);
    let maximum = min(vec2<f32>(pixel) + vec2<f32>(1.0), parameters.clip.zw);
    let overlap = clamp(maximum - minimum, vec2<f32>(0.0), vec2<f32>(1.0));
    return overlap.x * overlap.y;
}

fn capture_alpha(index: vec2<i32>) -> f32 {
    if any(index < vec2<i32>(0)) || any(index >= vec2<i32>(parameters.sizes.xy)) {
        return 0.0;
    }
    let alpha = effective_strength(textureLoad(source, index, 0).a);
    return effective_strength(alpha * cell_coverage(index));
}

@fragment
fn resample_bump_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let coordinate = sample_coordinate(vec2<i32>(floor(position.xy)));
    let base = vec2<i32>(floor(coordinate));
    let fraction = fract(coordinate);
    let top = mix(capture_alpha(base), capture_alpha(base + vec2<i32>(1, 0)), fraction.x);
    let bottom = mix(capture_alpha(base + vec2<i32>(0, 1)), capture_alpha(base + vec2<i32>(1, 1)), fraction.x);
    let alpha = effective_strength(mix(top, bottom, fraction.y));
    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}

struct AxisEdges {
    first: bool,
    last: bool,
}

fn at_edge(edge: u32) -> bool {
    return (parameters.flags.x & edge) != 0u;
}

fn bump_alpha(index: vec2<i32>, maximum: vec2<i32>) -> f32 {
    // The nine opportunities include unavailable exterior taps. Edge kernels
    // discard those taps, so the clamped loads do not extend the bump field.
    return effective_strength(textureLoad(source, clamp(index, vec2<i32>(0), maximum), 0).a);
}

fn difference(alpha: vec3<f32>, edges: AxisEdges) -> f32 {
    if edges.first {
        if edges.last { return 0.0; }
        return alpha.z - alpha.y;
    }
    if edges.last { return alpha.y - alpha.x; }
    return alpha.z - alpha.x;
}

fn axis_average(values: vec3<f32>, edges: AxisEdges) -> f32 {
    if edges.first {
        if edges.last { return values.y; }
        return (2.0 * values.y + values.z) / 3.0;
    }
    if edges.last { return (values.x + 2.0 * values.y) / 3.0; }
    return (values.x + 2.0 * values.y + values.z) * 0.25;
}

fn scaled_gradient(values: vec3<f32>, derivative: AxisEdges, smoothing: AxisEdges, ratio: f32) -> f32 {
    if derivative.first && derivative.last { return 0.0; }
    let average = clamp(axis_average(values, smoothing), -1.0, 1.0);
    if derivative.first || derivative.last { return (2.0 * ratio) * average; }
    return ratio * average;
}

fn scaled_direction(value: vec3<f32>) -> vec3<f32> {
    let vector = select(vec3<f32>(0.0), value, abs(value) >= vec3<f32>(MIN_ALPHA));
    let magnitude = abs(vector);
    let scale = max(max(magnitude.x, magnitude.y), magnitude.z);
    if scale == 0.0 { return vec3<f32>(0.0); }
    var scaled: vec3<f32>;
    // Bound the divisor even for the largest finite f32. Its reciprocal is
    // then normal, including on adapters that flush subnormal arithmetic.
    if scale > 8.507059173023462e37 {
        scaled = (vector * 0.25) / (scale * 0.25);
    } else {
        scaled = vector / scale;
    }
    return select(vec3<f32>(0.0), scaled, abs(scaled) >= vec3<f32>(MIN_ALPHA));
}

fn normalize_scaled(scaled: vec3<f32>) -> vec3<f32> {
    if all(scaled == vec3<f32>(0.0)) { return scaled; }
    return scaled * inverseSqrt(dot(scaled, scaled));
}

fn unit_vector(value: vec3<f32>) -> vec3<f32> {
    return normalize_scaled(scaled_direction(value));
}

fn incident_vector(pixel: vec2<i32>, alpha: f32) -> vec3<f32> {
    if parameters.flags.z == LIGHT_NONE { return vec3<f32>(0.0); }
    if parameters.flags.z == LIGHT_DISTANT { return parameters.incident.xyz; }
    let offset = vec2<f32>(pixel - parameters.light_anchor) * parameters.node_base_step.zw;
    let node_xy = offset + parameters.node_base_step.xy;
    let height_delta = parameters.surface_material.z * (parameters.height_anchor - alpha);
    return vec3<f32>(-node_xy, parameters.incident.z + height_delta);
}

fn spot_strength(light: vec3<f32>, direction: vec3<f32>) -> f32 {
    if parameters.flags.z != LIGHT_SPOT { return 1.0; }
    if parameters.flags.w == CONE_ZERO {
        return select(0.0, 1.0, all(-direction == scaled_direction(parameters.spot_axis.xyz)));
    }
    let cosine = clamp(-dot(light, parameters.spot_axis.xyz), 0.0, 1.0);
    let focus = effective_strength(clamp(pow(cosine, parameters.spot_axis.w), 0.0, 1.0));
    if parameters.flags.w == CONE_POSITIVE {
        let taper = clamp((cosine - parameters.spot_cone.x) * parameters.spot_cone.y, 0.0, 1.0);
        return effective_strength(focus * taper);
    }
    return focus;
}

fn material_response(normal: vec3<f32>, light: vec3<f32>) -> f32 {
    if parameters.material_exponent == 0.0 {
        return clamp(dot(normal, light), 0.0, 1.0);
    }
    if all(light == vec3<f32>(0.0)) { return 0.0; }
    var halfway_vector = vec3<f32>(light.xy, 1.0 + light.z);
    if light.z < 0.0 {
        // Use 1 + Lz = |Lxy|² / (1 - Lz), and scale the entire halfway
        // vector by the dominant horizontal component before squaring.
        // This preserves a near-antipodal Z through cancellation and underflow.
        let scale = max(abs(light.x), abs(light.y));
        if scale < MIN_ALPHA { return 0.0; }
        let horizontal = light.xy / scale;
        let z = scale * (dot(horizontal, horizontal) / (1.0 - light.z));
        halfway_vector = vec3<f32>(horizontal, z);
    }
    let halfway = unit_vector(halfway_vector);
    let alignment = clamp(dot(normal, halfway), 0.0, 1.0);
    return clamp(pow(alignment, parameters.material_exponent), 0.0, 1.0);
}

fn shade(pixel: vec2<i32>, alpha: f32, normal: vec3<f32>) -> vec4<f32> {
    let direction = scaled_direction(incident_vector(pixel, alpha));
    let light = normalize_scaled(direction);
    let angle = effective_strength(material_response(normal, light));
    let angular_strength = effective_strength(angle * spot_strength(light, direction));
    let strength = effective_strength(parameters.surface_material.w * angular_strength);
    var working_rgb = parameters.light_color.rgb;
    if parameters.flags.y != 0u {
        working_rgb = clamp(decode(working_rgb), vec3<f32>(0.0), vec3<f32>(1.0));
    }
    working_rgb = select(vec3<f32>(0.0), working_rgb, working_rgb >= vec3<f32>(MIN_ALPHA));
    let rgb = clamp(strength * working_rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    let output_alpha = select(1.0, max(max(rgb.r, rgb.g), rgb.b), parameters.material_exponent > 0.0);
    return vec4<f32>(rgb, output_alpha);
}

@fragment
fn shade_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(position.xy));
    let index = pixel + parameters.shade_to_bump.xy;
    let maximum = vec2<i32>(parameters.sizes.xy) - vec2<i32>(1);
    let x = AxisEdges(at_edge(X_MIN) && index.x == 0, at_edge(X_MAX) && index.x == maximum.x);
    let y = AxisEdges(at_edge(Y_MIN) && index.y == 0, at_edge(Y_MAX) && index.y == maximum.y);

    let top = vec3<f32>(
        bump_alpha(index + vec2<i32>(-1, -1), maximum),
        bump_alpha(index + vec2<i32>(0, -1), maximum),
        bump_alpha(index + vec2<i32>(1, -1), maximum),
    );
    let middle = vec3<f32>(
        bump_alpha(index + vec2<i32>(-1, 0), maximum),
        bump_alpha(index, maximum),
        bump_alpha(index + vec2<i32>(1, 0), maximum),
    );
    let bottom = vec3<f32>(
        bump_alpha(index + vec2<i32>(-1, 1), maximum),
        bump_alpha(index + vec2<i32>(0, 1), maximum),
        bump_alpha(index + vec2<i32>(1, 1), maximum),
    );

    // Smooth in the perpendicular axis after taking each difference. Keeping
    // intermediate differences within [-1, 1] bounds the scaled gradients.
    let rows = vec3<f32>(difference(top, x), difference(middle, x), difference(bottom, x));
    let columns = vec3<f32>(
        difference(vec3<f32>(top.x, middle.x, bottom.x), y),
        difference(vec3<f32>(top.y, middle.y, bottom.y), y),
        difference(vec3<f32>(top.z, middle.z, bottom.z), y),
    );
    let dx = scaled_gradient(rows, x, y, parameters.surface_material.x);
    let dy = scaled_gradient(columns, y, x, parameters.surface_material.y);
    return shade(pixel, middle.y, unit_vector(vec3<f32>(-dx, -dy, 1.0)));
}

@fragment
fn zero_bump_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return shade(vec2<i32>(floor(position.xy)), 0.0, vec3<f32>(0.0, 0.0, 1.0));
}

fn result_sample(index: vec2<i32>) -> vec4<f32> {
    let maximum = vec2<i32>(parameters.sizes.xy) - vec2<i32>(1);
    var mapped = index;
    if at_edge(X_MIN) { mapped.x = max(mapped.x, 0); }
    if at_edge(X_MAX) { mapped.x = min(mapped.x, maximum.x); }
    if at_edge(Y_MIN) { mapped.y = max(mapped.y, 0); }
    if at_edge(Y_MAX) { mapped.y = min(mapped.y, maximum.y); }
    return textureLoad(source, mapped, 0);
}

@fragment
fn resolve_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(floor(position.xy));
    let coordinate = sample_coordinate(pixel);
    let base = vec2<i32>(floor(coordinate));
    let fraction = fract(coordinate);
    let top = mix(result_sample(base), result_sample(base + vec2<i32>(1, 0)), fraction.x);
    let bottom = mix(result_sample(base + vec2<i32>(0, 1)), result_sample(base + vec2<i32>(1, 1)), fraction.x);
    let value = from_working(mix(top, bottom, fraction.y));
    let alpha = clamp(value.a, 0.0, 1.0);
    let valid = vec4<f32>(clamp(value.rgb, vec3<f32>(0.0), vec3<f32>(alpha)), alpha);
    return valid * cell_coverage(pixel);
}
