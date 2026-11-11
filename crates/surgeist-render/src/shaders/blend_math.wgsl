fn surgeist_blend_color_dodge(backdrop: vec3<f32>, source: vec3<f32>) -> vec3<f32> {
    let zero = vec3<f32>(0.0);
    let one = vec3<f32>(1.0);
    let divide = source < one;
    let divisor = select(one, one - source, divide);
    let ratio = min(backdrop, divisor) / divisor;
    // Extended source values from local Plus give a negative ratio that
    // ordinary blending clamps to zero. Preserve the exact one endpoint.
    let endpoint = select(zero, one, source == one);
    return select(select(endpoint, ratio, divide), zero, backdrop == zero);
}

fn surgeist_blend_color_burn(backdrop: vec3<f32>, source: vec3<f32>) -> vec3<f32> {
    let zero = vec3<f32>(0.0);
    let one = vec3<f32>(1.0);
    let complement = max(one - backdrop, zero);
    let divide = (complement > zero) & (complement < source);
    let divisor = select(one, source, divide);
    let clamped = select(one, zero, complement == zero);
    let ratio = select(clamped, complement / divisor, divide);
    let burned = select(zero, one - ratio, source > zero);
    return select(burned, one, backdrop == one);
}

fn surgeist_blend_soft_light(backdrop: vec3<f32>, source: vec3<f32>) -> vec3<f32> {
    let quarter = vec3<f32>(0.25);
    // The fine raster path can supply extended color after local Plus.
    // Bound the eagerly evaluated cubic to the range that selects it.
    let low = min(backdrop, quarter);
    let polynomial = ((16.0 * low - 12.0) * low + 4.0) * low;
    let d = select(sqrt(max(backdrop, quarter)), polynomial, backdrop <= quarter);
    let half = vec3<f32>(0.5);
    let dark_source = min(source, half);
    let dark = backdrop - (1.0 - 2.0 * dark_source) * backdrop * (1.0 - backdrop);
    let light = backdrop + (2.0 * source - 1.0) * (d - backdrop);
    return select(light, dark, source <= half);
}

fn surgeist_blend_clip_color(color: vec3<f32>, luminosity: f32) -> vec3<f32> {
    // Both corrections use the original luminosity and extrema.
    let minimum = min(min(color.r, color.g), color.b);
    let maximum = max(max(color.r, color.g), color.b);
    let low_span = luminosity - minimum;
    let high_span = maximum - luminosity;
    let clip_low = (minimum < 0.0) & (low_span > 0.0);
    let clip_high = (maximum > 1.0) & (high_span > 0.0);
    let low_divisor = select(1.0, low_span, clip_low);
    let high_divisor = select(1.0, high_span, clip_high);
    let low = luminosity + (color - luminosity) * luminosity / low_divisor;
    let result = select(color, low, clip_low);
    let high = luminosity + (result - luminosity) * (1.0 - luminosity) / high_divisor;
    return select(result, high, clip_high);
}

fn surgeist_blend_luminosity_identity(color: vec3<f32>, luminosity: f32) -> bool {
    let gray = all(color == vec3<f32>(color.r));
    return any(vec3<bool>(luminosity == 0.0, luminosity == 1.0, gray));
}

fn surgeist_blend_set_sat(color: vec3<f32>, saturation: f32) -> vec3<f32> {
    let minimum = min(min(color.r, color.g), color.b);
    let maximum = max(max(color.r, color.g), color.b);
    let span = maximum - minimum;
    let chromatic = span > 0.0;
    let divisor = select(1.0, span, chromatic);
    let scaled = (color - minimum) * saturation / divisor;
    let result = select(scaled, vec3<f32>(saturation), color == vec3<f32>(maximum));
    return select(vec3<f32>(0.0), result, vec3<bool>(chromatic));
}
