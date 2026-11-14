struct PassSpatialUniform {
    source_origin_scale: vec4<f32>,
    destination_origin_scale: vec4<f32>,
    source_destination_extents: vec4<u32>,
}

struct CompositeParameters {
    source_texel_linear: vec4<f32>,
    source_mask_translation: vec4<f32>,
    mask_normalized_linear: vec4<f32>,
    mask_dimensions: vec4<u32>,
    mask_texel_facts: vec4<f32>,
    opacity: f32,
    blend: u32,
    mask_quality: u32,
    mask_extend: u32,
    // Clip presence, mask value and application bits, source coordinate space, operator code.
    presence: vec4<u32>,
    // Gaussian-only O cell clip in actual source-allocation texel coordinates.
    source_operation_clip: vec4<f32>,
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

fn source_texel(destination_position: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        parameters.source_texel_linear.x * destination_position.x
            + parameters.source_texel_linear.z * destination_position.y
            + parameters.source_mask_translation.x,
        parameters.source_texel_linear.y * destination_position.x
            + parameters.source_texel_linear.w * destination_position.y
            + parameters.source_mask_translation.y,
    );
}

fn mask_normalized(destination_position: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        parameters.mask_normalized_linear.x * destination_position.x
            + parameters.mask_normalized_linear.z * destination_position.y
            + parameters.source_mask_translation.z,
        parameters.mask_normalized_linear.y * destination_position.x
            + parameters.mask_normalized_linear.w * destination_position.y
            + parameters.source_mask_translation.w,
    );
}

fn load_surface_source(texel: vec2<f32>, extent: vec2<f32>) -> vec4<f32> {
    // Guard the conversion as well as the load, including distant finite maps.
    if (any(texel < vec2<f32>(0.0)) || any(texel >= extent)) {
        return vec4<f32>(0.0);
    }
    return textureLoad(source_texture, vec2<i32>(floor(texel)), 0);
}

fn sample_source(destination_position: vec2<f32>) -> vec4<f32> {
    let texel = source_texel(destination_position);
    let extent = vec2<f32>(spatial.source_destination_extents.xy);
    let inside = all(texel >= vec2<f32>(0.0)) && all(texel < extent);
    var sampled: vec4<f32>;
    if ((parameters.presence.z & 1u) != 0u) {
        // Surface-grid sources share the destination's device-pixel lattice.
        // Load that pixel directly so composition cannot smooth it again.
        sampled = load_surface_source(texel, extent);
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

// Reconstruct the bounded native-filter operation coverage with the exact source
// lattice, clamp-to-edge bilinear kernel, and center-inside allocation guard.
// Source alpha cannot identify this geometric footprint (it may be transparent).
fn operation_cell_coverage(index: vec2<i32>, extent: vec2<i32>) -> f32 {
    let cell = vec2<f32>(clamp(index, vec2<i32>(0), extent - vec2<i32>(1)));
    let clip = parameters.source_operation_clip;
    let overlap = clamp(min(cell + vec2<f32>(1.0), clip.zw) - max(cell, clip.xy), vec2<f32>(0.0), vec2<f32>(1.0));
    return overlap.x * overlap.y;
}

fn filter_operation_coverage(destination_position: vec2<f32>) -> f32 {
    let texel = source_texel(destination_position);
    let extent = vec2<f32>(spatial.source_destination_extents.xy);
    if any(texel < vec2<f32>(0.0)) || any(texel >= extent) { return 0.0; }
    let integer_extent = vec2<i32>(spatial.source_destination_extents.xy);
    if (parameters.presence.z & 1u) != 0u {
        return operation_cell_coverage(vec2<i32>(floor(texel)), integer_extent);
    }
    let normalized = clamp(texel / extent, vec2<f32>(0.0), vec2<f32>(1.0));
    let coordinate = normalized * extent - vec2<f32>(0.5);
    let base = vec2<i32>(floor(coordinate));
    let fraction = fract(coordinate);
    let top = mix(operation_cell_coverage(base, integer_extent), operation_cell_coverage(base + vec2<i32>(1, 0), integer_extent), fraction.x);
    let bottom = mix(operation_cell_coverage(base + vec2<i32>(0, 1), integer_extent), operation_cell_coverage(base + vec2<i32>(1, 1), integer_extent), fraction.x);
    return mix(top, bottom, fraction.y);
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


fn sample_mask_alpha(destination_position: vec2<f32>) -> f32 {
    let normalized = mask_normalized(destination_position);
    if (any(normalized < vec2<f32>(0.0)) || any(normalized > vec2<f32>(1.0))) {
        return 0.0;
    }

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

fn nonseparable_lum(color: vec3<f32>) -> f32 {
    return 0.3 * color.r + 0.59 * color.g + 0.11 * color.b;
}

fn nonseparable_sat(color: vec3<f32>) -> f32 {
    return max(max(color.r, color.g), color.b) - min(min(color.r, color.g), color.b);
}

fn nonseparable_clip_color(color: vec3<f32>) -> vec3<f32> {
    return surgeist_blend_clip_color(color, nonseparable_lum(color));
}

fn nonseparable_set_lum(color: vec3<f32>, luminosity: f32) -> vec3<f32> {
    let exact = vec3<bool>(surgeist_blend_luminosity_identity(color, luminosity));
    let difference = luminosity - nonseparable_lum(color);
    let shifted = select(color + vec3<f32>(difference), vec3<f32>(0.0), exact);
    let clipped = nonseparable_clip_color(shifted);
    return select(clipped, vec3<f32>(luminosity), exact);
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
            return surgeist_blend_color_dodge(backdrop, source);
        }
        case 8u: {
            return surgeist_blend_color_burn(backdrop, source);
        }
        case 9u: {
            let low = 2.0 * source * backdrop;
            let high = 1.0 - 2.0 * (1.0 - source) * (1.0 - backdrop);
            return select(high, low, source <= vec3<f32>(0.5));
        }
        case 10u: {
            return surgeist_blend_soft_light(backdrop, source);
        }
        case 11u: {
            return abs(backdrop - source);
        }
        case 12u: {
            return backdrop + source - 2.0 * backdrop * source;
        }
        case 13u: {
            return nonseparable_set_lum(
                surgeist_blend_set_sat(source, nonseparable_sat(backdrop)),
                nonseparable_lum(backdrop),
            );
        }
        case 14u: {
            return nonseparable_set_lum(
                surgeist_blend_set_sat(backdrop, nonseparable_sat(source)),
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

fn operation_composite(source: vec4<f32>, backdrop: vec4<f32>, destination_position: vec2<f32>) -> vec4<f32> {
    let complete = composite_against_parent(source, backdrop);
    // These six effective operators discard transparent source areas. The
    // source already contains O coverage, so restore only uncovered destination.
    let operation = parameters.presence.w;
    let destructive = operation == 1u || operation == 2u || operation == 5u
        || operation == 6u || operation == 7u || operation == 10u;
    if (parameters.presence.z & 2u) != 0u && destructive
        && parameters.blend != 6u && operation != PLUS_LIGHTER_OPERATOR {
        let covered = complete + (1.0 - filter_operation_coverage(destination_position)) * backdrop;
        // Storage/interpolation rounding can make covered source alpha slightly
        // exceed its analytic coverage. Restore valid premultiplication here,
        // before the authored whole-operation clip interpolates this result.
        let alpha = clamp(covered.a, 0.0, 1.0);
        return vec4<f32>(clamp(covered.rgb, vec3<f32>(0.0), vec3<f32>(alpha)), alpha);
    }
    return complete;
}

fn destination_composite(source: vec4<f32>, destination_position: vec2<f32>) -> vec4<f32> {
    return operation_composite(source, load_parent(destination_position), destination_position);
}

fn covered_destination_composite(
    source: vec4<f32>,
    destination_position: vec2<f32>,
    coverage: f32,
) -> vec4<f32> {
    let backdrop = load_parent(destination_position);
    let complete = operation_composite(source, backdrop, destination_position);
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
