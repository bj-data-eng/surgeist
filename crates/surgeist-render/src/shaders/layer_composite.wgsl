struct PassSpatialUniform {
    source_origin_scale: vec4<f32>,
    destination_origin_scale: vec4<f32>,
    source_destination_extents: vec4<u32>,
}

struct CompositeParameters {
    affine_linear: vec4<f32>,
    affine_translation: vec4<f32>,
    mask_bounds: vec4<f32>,
    mask_dimensions: vec4<u32>,
    mask_texel_facts: vec4<f32>,
    opacity: f32,
    blend: u32,
    mask_quality: u32,
    mask_extend: u32,
    // Clip presence, mask value and application bits, source coordinate space, operator code.
    presence: vec4<u32>,
}

@group(0) @binding(0)
var source_texture: texture_2d<f32>;

@group(0) @binding(1)
var source_sampler: sampler;

@group(0) @binding(2)
var parent_texture: texture_2d<f32>;

@group(0) @binding(3)
var clip_coverage_texture: texture_2d<f32>;

@group(0) @binding(4)
var alpha_mask_texture: texture_2d<f32>;

@group(0) @binding(5)
var<uniform> spatial: PassSpatialUniform;

@group(0) @binding(6)
var<uniform> parameters: CompositeParameters;

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

fn destination_point(destination_position: vec2<f32>) -> vec2<f32> {
    return spatial.destination_origin_scale.xy
        + destination_position / spatial.destination_origin_scale.z;
}

fn layer_local_point(destination_position: vec2<f32>) -> vec2<f32> {
    let point = destination_point(destination_position);
    return vec2<f32>(
        parameters.affine_linear.x * point.x
            + parameters.affine_linear.z * point.y
            + parameters.affine_translation.x,
        parameters.affine_linear.y * point.x
            + parameters.affine_linear.w * point.y
            + parameters.affine_translation.y,
    );
}

fn source_texel(destination_position: vec2<f32>) -> vec2<f32> {
    var source_point = layer_local_point(destination_position);
    if (parameters.presence.z == 1u) {
        source_point = destination_point(destination_position);
    }
    return (source_point - spatial.source_origin_scale.xy) * spatial.source_origin_scale.z;
}

fn sample_source(destination_position: vec2<f32>) -> vec4<f32> {
    let texel = source_texel(destination_position);
    let extent = vec2<f32>(spatial.source_destination_extents.xy);
    let inside = all(texel >= vec2<f32>(0.0)) && all(texel < extent);
    var sampled: vec4<f32>;
    if (parameters.presence.z == 1u) {
        // Surface-grid sources share the destination's device-pixel lattice.
        // Load that pixel directly so composition cannot smooth it again.
        sampled = textureLoad(source_texture, vec2<i32>(floor(texel)), 0);
    } else {
        sampled = textureSample(
            source_texture,
            source_sampler,
            clamp(texel / extent, vec2<f32>(0.0), vec2<f32>(1.0)),
        );
    }
    let bounded = select(vec4<f32>(0.0), clamp(sampled, vec4<f32>(0.0), vec4<f32>(1.0)), inside);
    let alpha = bounded.a;
    return vec4<f32>(min(bounded.rgb, vec3<f32>(alpha)), alpha);
}

fn load_parent(destination_position: vec2<f32>) -> vec4<f32> {
    let coordinate = vec2<i32>(floor(destination_position));
    let dimensions = vec2<i32>(textureDimensions(parent_texture));
    if (any(coordinate < vec2<i32>(0)) || any(coordinate >= dimensions)) {
        return vec4<f32>(0.0);
    }
    let sampled = clamp(
        textureLoad(parent_texture, coordinate, 0),
        vec4<f32>(0.0),
        vec4<f32>(1.0),
    );
    return vec4<f32>(min(sampled.rgb, vec3<f32>(sampled.a)), sampled.a);
}

fn sample_clip_alpha(destination_position: vec2<f32>) -> f32 {
    let coordinate = vec2<i32>(floor(destination_position));
    let dimensions = vec2<i32>(textureDimensions(clip_coverage_texture));
    if (any(coordinate < vec2<i32>(0)) || any(coordinate >= dimensions)) {
        return 0.0;
    }
    return clamp(textureLoad(clip_coverage_texture, coordinate, 0).a, 0.0, 1.0);
}

fn euclidean_mod(index: i32, length: i32) -> i32 {
    let remainder = index % length;
    return select(remainder + length, remainder, remainder >= 0);
}

fn extend_mask_index(index: i32, length: i32) -> i32 {
    if (parameters.mask_extend == 0u) {
        return clamp(index, 0, length - 1);
    }
    if (parameters.mask_extend == 1u) {
        return euclidean_mod(index, length);
    }
    let period = length * 2;
    let reflected = euclidean_mod(index, period);
    return select(period - reflected - 1, reflected, reflected < length);
}

fn mask_alpha_tap(coordinate: vec2<i32>) -> f32 {
    let dimensions = vec2<i32>(parameters.mask_dimensions.xy);
    let extended = vec2<i32>(
        extend_mask_index(coordinate.x, dimensions.x),
        extend_mask_index(coordinate.y, dimensions.y),
    );
    let texel = textureLoad(alpha_mask_texture, extended, 0);
    let mode = parameters.presence.y & 3u;
    if (mode == 1u) {
        return texel.a;
    }
    var rgb = texel.rgb;
    if (mode == 3u) {
        let linear_low = rgb / 12.92;
        let linear_high = pow((rgb + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
        rgb = select(linear_high, linear_low, rgb <= vec3<f32>(0.04045));
    }
    return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722)) * texel.a;
}

fn mitchell_netravali(distance_value: f32) -> f32 {
    let distance = abs(distance_value);
    let squared = distance * distance;
    let cubed = squared * distance;
    if (distance < 1.0) {
        return (7.0 * cubed - 12.0 * squared + 5.3333335) / 6.0;
    }
    if (distance < 2.0) {
        return (-2.3333335 * cubed + 12.0 * squared - 20.0 * distance + 10.666667) / 6.0;
    }
    return 0.0;
}

fn sample_mask_alpha(destination_position: vec2<f32>) -> f32 {
    let local = layer_local_point(destination_position);
    let minimum = parameters.mask_bounds.xy;
    let maximum = minimum + parameters.mask_bounds.zw;
    if (any(local < minimum) || any(local > maximum)) {
        return 0.0;
    }

    let normalized = (local - minimum) / parameters.mask_bounds.zw;
    let sample_texel = (normalized - parameters.mask_texel_facts.xy)
        / parameters.mask_texel_facts.zw;
    if (parameters.mask_quality == 0u) {
        return clamp(mask_alpha_tap(vec2<i32>(floor(sample_texel + vec2<f32>(0.5)))), 0.0, 1.0);
    }

    if (parameters.mask_quality == 3u) {
        // Sample a virtual nearest-neighbor enlargement with a bilinear kernel.
        // Map each virtual tap back before applying the authored extend policy.
        let factors = vec2<f32>(parameters.mask_dimensions.zw);
        let enlarged = (sample_texel + vec2<f32>(0.5)) * factors - vec2<f32>(0.5);
        let base = floor(enlarged);
        let fraction = enlarged - base;
        let first = vec2<i32>(floor(base / factors));
        let second = vec2<i32>(floor((base + vec2<f32>(1.0)) / factors));
        let top = mix(mask_alpha_tap(first), mask_alpha_tap(vec2<i32>(second.x, first.y)), fraction.x);
        let bottom = mix(mask_alpha_tap(vec2<i32>(first.x, second.y)), mask_alpha_tap(second), fraction.x);
        return clamp(mix(top, bottom, fraction.y), 0.0, 1.0);
    }

    let base = vec2<i32>(floor(sample_texel));
    let fraction = sample_texel - vec2<f32>(base);
    if (parameters.mask_quality == 1u) {
        let top = mix(
            mask_alpha_tap(base),
            mask_alpha_tap(base + vec2<i32>(1, 0)),
            fraction.x,
        );
        let bottom = mix(
            mask_alpha_tap(base + vec2<i32>(0, 1)),
            mask_alpha_tap(base + vec2<i32>(1, 1)),
            fraction.x,
        );
        return clamp(mix(top, bottom, fraction.y), 0.0, 1.0);
    }

    var alpha = 0.0;
    for (var offset_y: i32 = -1; offset_y <= 2; offset_y = offset_y + 1) {
        let tap_y = base.y + offset_y;
        let weight_y = mitchell_netravali(sample_texel.y - f32(tap_y));
        for (var offset_x: i32 = -1; offset_x <= 2; offset_x = offset_x + 1) {
            let tap_x = base.x + offset_x;
            let weight_x = mitchell_netravali(sample_texel.x - f32(tap_x));
            alpha = alpha
                + mask_alpha_tap(vec2<i32>(tap_x, tap_y)) * weight_x * weight_y;
        }
    }
    return clamp(alpha, 0.0, 1.0);
}

fn attenuate_source(source: vec4<f32>, coverage: f32) -> vec4<f32> {
    let amount = clamp(coverage, 0.0, 1.0) * clamp(parameters.opacity, 0.0, 1.0);
    return source * amount;
}

fn source_without_outer(destination_position: vec2<f32>) -> vec4<f32> {
    return attenuate_source(sample_source(destination_position), 1.0);
}

fn source_with_clip(destination_position: vec2<f32>) -> vec4<f32> {
    return attenuate_source(
        sample_source(destination_position),
        sample_clip_alpha(destination_position),
    );
}

fn masked_source_color(destination_position: vec2<f32>) -> vec4<f32> {
    if ((parameters.presence.y & 4u) != 0u) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return sample_source(destination_position);
}

fn source_with_mask(destination_position: vec2<f32>) -> vec4<f32> {
    return attenuate_source(
        masked_source_color(destination_position),
        sample_mask_alpha(destination_position),
    );
}

fn source_with_clip_mask(destination_position: vec2<f32>) -> vec4<f32> {
    let clip_alpha = sample_clip_alpha(destination_position);
    let mask_alpha = sample_mask_alpha(destination_position);
    return attenuate_source(masked_source_color(destination_position), clip_alpha * mask_alpha);
}

fn safe_straight_rgb(premultiplied: vec4<f32>) -> vec3<f32> {
    let transparent = premultiplied.a == 0.0;
    let divisor = select(premultiplied.a, 1.0, transparent);
    let rgb = clamp(premultiplied.rgb / divisor, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(rgb, vec3<f32>(0.0), transparent);
}

fn color_dodge(source: f32, backdrop: f32) -> f32 {
    // The backdrop guard takes precedence at (Cb, Cs) = (0, 1).
    if (backdrop == 0.0) {
        return 0.0;
    }
    if (source == 1.0) {
        return 1.0;
    }
    return min(1.0, backdrop / (1.0 - source));
}

fn color_burn(source: f32, backdrop: f32) -> f32 {
    // The backdrop guard takes precedence at (Cb, Cs) = (1, 0).
    if (backdrop == 1.0) {
        return 1.0;
    }
    if (source == 0.0) {
        return 0.0;
    }
    return 1.0 - min(1.0, (1.0 - backdrop) / source);
}

fn soft_light(source: f32, backdrop: f32) -> f32 {
    if (source <= 0.5) {
        return backdrop - (1.0 - 2.0 * source) * backdrop * (1.0 - backdrop);
    }
    var d: f32;
    if (backdrop <= 0.25) {
        d = ((16.0 * backdrop - 12.0) * backdrop + 4.0) * backdrop;
    } else {
        d = sqrt(backdrop);
    }
    return backdrop + (2.0 * source - 1.0) * (d - backdrop);
}

fn nonseparable_lum(color: vec3<f32>) -> f32 {
    return 0.3 * color.r + 0.59 * color.g + 0.11 * color.b;
}

fn nonseparable_sat(color: vec3<f32>) -> f32 {
    return max(max(color.r, color.g), color.b) - min(min(color.r, color.g), color.b);
}

fn nonseparable_clip_color(color: vec3<f32>) -> vec3<f32> {
    // Keep the original luminosity and extrema across both independent conditions.
    // Reachable SetLum inputs have L in [0, 1], so active denominators are nonzero.
    let luminosity = nonseparable_lum(color);
    let minimum = min(min(color.r, color.g), color.b);
    let maximum = max(max(color.r, color.g), color.b);
    var result = color;
    if (minimum < 0.0) {
        result = luminosity + (result - luminosity) * luminosity / (luminosity - minimum);
    }
    if (maximum > 1.0) {
        result = luminosity
            + (result - luminosity) * (1.0 - luminosity) / (maximum - luminosity);
    }
    return result;
}

fn nonseparable_set_lum(color: vec3<f32>, luminosity: f32) -> vec3<f32> {
    // Exact identities avoid a rounded constant shifted color reaching a clip pole.
    if (luminosity == 0.0 || luminosity == 1.0
        || (color.r == color.g && color.g == color.b)) {
        return vec3<f32>(luminosity);
    }
    let difference = luminosity - nonseparable_lum(color);
    return nonseparable_clip_color(color + vec3<f32>(difference));
}

fn nonseparable_set_sat(color: vec3<f32>, saturation: f32) -> vec3<f32> {
    // Sort channel identities, then write back to their original RGB positions.
    var minimum = 0u;
    var middle = 1u;
    var maximum = 2u;
    if (color[minimum] > color[middle]) {
        let previous = minimum;
        minimum = middle;
        middle = previous;
    }
    if (color[middle] > color[maximum]) {
        let previous = middle;
        middle = maximum;
        maximum = previous;
    }
    if (color[minimum] > color[middle]) {
        let previous = minimum;
        minimum = middle;
        middle = previous;
    }
    var result = vec3<f32>(0.0);
    if (color[maximum] > color[minimum]) {
        result[middle] = (color[middle] - color[minimum]) * saturation
            / (color[maximum] - color[minimum]);
        result[maximum] = saturation;
    }
    return result;
}

fn blend_rgb(source: vec3<f32>, backdrop: vec3<f32>, mode: u32) -> vec3<f32> {
    switch mode {
        case 1u: {
            return source * backdrop;
        }
        case 2u: {
            return source + backdrop - source * backdrop;
        }
        case 3u: {
            let low = 2.0 * source * backdrop;
            let high = 1.0 - 2.0 * (1.0 - source) * (1.0 - backdrop);
            return select(high, low, backdrop <= vec3<f32>(0.5));
        }
        case 4u: {
            return min(source, backdrop);
        }
        case 5u: {
            return max(source, backdrop);
        }
        case 7u: {
            return vec3<f32>(
                color_dodge(source.r, backdrop.r),
                color_dodge(source.g, backdrop.g),
                color_dodge(source.b, backdrop.b),
            );
        }
        case 8u: {
            return vec3<f32>(
                color_burn(source.r, backdrop.r),
                color_burn(source.g, backdrop.g),
                color_burn(source.b, backdrop.b),
            );
        }
        case 9u: {
            let low = 2.0 * source * backdrop;
            let high = 1.0 - 2.0 * (1.0 - source) * (1.0 - backdrop);
            return select(high, low, source <= vec3<f32>(0.5));
        }
        case 10u: {
            return vec3<f32>(
                soft_light(source.r, backdrop.r),
                soft_light(source.g, backdrop.g),
                soft_light(source.b, backdrop.b),
            );
        }
        case 11u: {
            return abs(backdrop - source);
        }
        case 12u: {
            return backdrop + source - 2.0 * backdrop * source;
        }
        case 13u: {
            return nonseparable_set_lum(
                nonseparable_set_sat(source, nonseparable_sat(backdrop)),
                nonseparable_lum(backdrop),
            );
        }
        case 14u: {
            return nonseparable_set_lum(
                nonseparable_set_sat(backdrop, nonseparable_sat(source)),
                nonseparable_lum(backdrop),
            );
        }
        case 15u: {
            return nonseparable_set_lum(source, nonseparable_lum(backdrop));
        }
        case 16u: {
            return nonseparable_set_lum(backdrop, nonseparable_lum(source));
        }
        default: {
            return source;
        }
    }
}

fn porter_duff_factors(source_alpha: f32, backdrop_alpha: f32, operation: u32) -> vec2<f32> {
    switch operation {
        case 1u: { // Clear
            return vec2<f32>(0.0, 0.0);
        }
        case 2u: { // Copy
            return vec2<f32>(1.0, 0.0);
        }
        case 3u: { // Destination
            return vec2<f32>(0.0, 1.0);
        }
        case 4u: { // DestinationOver
            return vec2<f32>(1.0 - backdrop_alpha, 1.0);
        }
        case 5u: { // SourceIn
            return vec2<f32>(backdrop_alpha, 0.0);
        }
        case 6u: { // DestinationIn
            return vec2<f32>(0.0, source_alpha);
        }
        case 7u: { // SourceOut
            return vec2<f32>(1.0 - backdrop_alpha, 0.0);
        }
        case 8u: { // DestinationOut
            return vec2<f32>(0.0, 1.0 - source_alpha);
        }
        case 9u: { // SourceAtop
            return vec2<f32>(backdrop_alpha, 1.0 - source_alpha);
        }
        case 10u: { // DestinationAtop
            return vec2<f32>(1.0 - backdrop_alpha, source_alpha);
        }
        case 11u: { // Xor
            return vec2<f32>(1.0 - backdrop_alpha, 1.0 - source_alpha);
        }
        default: { // SourceOver (code 0)
            return vec2<f32>(1.0, 1.0 - source_alpha);
        }
    }
}

const PLUS_LIGHTER_OPERATOR: u32 = 12u;

fn composite_against_parent(source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    if (parameters.blend == 6u || parameters.presence.w == PLUS_LIGHTER_OPERATOR) {
        return clamp(source + backdrop, vec4<f32>(0.0), vec4<f32>(1.0));
    }

    let source_straight = safe_straight_rgb(source);
    let backdrop_straight = safe_straight_rgb(backdrop);
    let blended = clamp(
        blend_rgb(source_straight, backdrop_straight, parameters.blend),
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    );
    if (parameters.presence.w == 0u) {
        // Retain the existing SourceOver arithmetic and its clip attenuation.
        let alpha = clamp(source.a + backdrop.a - source.a * backdrop.a, 0.0, 1.0);
        let premultiplied = (1.0 - backdrop.a) * source.rgb
            + (1.0 - source.a) * backdrop.rgb
            + source.a * backdrop.a * blended;
        return vec4<f32>(min(clamp(premultiplied, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(alpha)), alpha);
    }

    // Blend on straight components, then modulate by backdrop alpha once.
    // This is alpha_s * Cs', kept premultiplied for the selected factors.
    let blended_source = (1.0 - backdrop.a) * source.rgb
        + source.a * backdrop.a * blended;
    let factors = porter_duff_factors(source.a, backdrop.a, parameters.presence.w);
    let alpha = clamp(factors.x * source.a + factors.y * backdrop.a, 0.0, 1.0);
    let premultiplied = factors.x * blended_source + factors.y * backdrop.rgb;
    return vec4<f32>(min(clamp(premultiplied, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(alpha)), alpha);
}

fn destination_composite(source: vec4<f32>, destination_position: vec2<f32>) -> vec4<f32> {
    return composite_against_parent(source, load_parent(destination_position));
}

fn covered_destination_composite(
    source: vec4<f32>,
    destination_position: vec2<f32>,
    coverage: f32,
) -> vec4<f32> {
    let backdrop = load_parent(destination_position);
    let complete = composite_against_parent(source, backdrop);
    let amount = clamp(coverage, 0.0, 1.0);
    return (1.0 - amount) * backdrop + amount * complete;
}

@fragment
fn fragment_normal(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return source_without_outer(position.xy);
}

@fragment
fn fragment_normal_clip(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return source_with_clip(position.xy);
}

@fragment
fn fragment_normal_mask(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return source_with_mask(position.xy);
}

@fragment
fn fragment_normal_clip_mask(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return source_with_clip_mask(position.xy);
}

@fragment
fn fragment_destination(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return destination_composite(source_without_outer(position.xy), position.xy);
}

@fragment
fn fragment_destination_clip(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    if (parameters.presence.w == 0u || parameters.presence.w == PLUS_LIGHTER_OPERATOR) {
        return destination_composite(source_with_clip(position.xy), position.xy);
    }
    return covered_destination_composite(
        source_without_outer(position.xy),
        position.xy,
        sample_clip_alpha(position.xy),
    );
}

@fragment
fn fragment_destination_mask(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return destination_composite(source_with_mask(position.xy), position.xy);
}

@fragment
fn fragment_destination_clip_mask(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    if (parameters.presence.w == 0u || parameters.presence.w == PLUS_LIGHTER_OPERATOR) {
        return destination_composite(source_with_clip_mask(position.xy), position.xy);
    }
    return covered_destination_composite(
        source_with_mask(position.xy),
        position.xy,
        sample_clip_alpha(position.xy),
    );
}
