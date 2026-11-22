struct TurbulenceParameters {
    clip: vec4<f32>,
    phase_step: vec4<f32>,
    stitching: vec4<i32>,
    settings: vec4<u32>,
    reserved: vec4<u32>,
    permutation: array<vec4<u32>, 64>,
    gradients: array<vec4<f32>, 512>,
}

@group(0) @binding(0) var<uniform> parameters: TurbulenceParameters;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[index], 0.0, 1.0);
}

fn permutation(index: u32) -> u32 {
    let slot = index & 255u;
    return parameters.permutation[slot / 4u][slot % 4u];
}

fn lattice_slot(lattice: i32, wrap: i32, period: i32) -> u32 {
    let slot = bitcast<u32>(lattice);
    if period == 0 { return slot & 255u; }

    // Reduce to [wrap - period, wrap). Preserve the specification's single
    // subtraction where it suffices and also wrap corners farther away, as can
    // occur when a fractional tile origin has a one-cell period.
    // Signed comparisons establish the subtraction order. Unsigned distances
    // span the full i32 range, and only the low eight output bits are needed.
    let cutoff = bitcast<u32>(wrap);
    let width = u32(period);
    var wrapped = slot;
    if lattice >= wrap {
        let distance = slot - cutoff;
        if distance < width {
            wrapped = slot - width;
        } else {
            wrapped = cutoff - width + distance % width;
        }
    } else {
        let distance = cutoff - slot;
        if distance > width {
            let remainder = distance % width;
            wrapped = cutoff - select(remainder, width, remainder == 0u);
        }
    }
    return wrapped & 255u;
}

fn gradient_dot(x_selector: u32, y_lattice: u32, offset: vec2<f32>) -> vec4<f32> {
    let slot = 2u * permutation(x_selector + y_lattice);
    return offset.x * parameters.gradients[slot] + offset.y * parameters.gradients[slot + 1u];
}

// Filter Effects 1 § 9.21 uses four independent channel gradients on the same lattice.
fn noise(point: vec2<f32>, wrap: vec2<i32>, period: vec2<i32>) -> vec4<f32> {
    let base = floor(point);
    let lattice = vec2<i32>(base);
    let left = permutation(lattice_slot(lattice.x, wrap.x, period.x));
    let right = permutation(lattice_slot(lattice.x + 1, wrap.x, period.x));
    let top_y = lattice_slot(lattice.y, wrap.y, period.y);
    let bottom_y = lattice_slot(lattice.y + 1, wrap.y, period.y);

    let fraction = point - base;
    let fade = fraction * fraction * (3.0 - 2.0 * fraction);
    let top_left = gradient_dot(left, top_y, fraction);
    let top_right = gradient_dot(right, top_y, fraction - vec2<f32>(1.0, 0.0));
    let bottom_left = gradient_dot(left, bottom_y, fraction - vec2<f32>(0.0, 1.0));
    let bottom_right = gradient_dot(right, bottom_y, fraction - vec2<f32>(1.0));
    let top = top_left + fade.x * (top_right - top_left);
    let bottom = bottom_left + fade.x * (bottom_right - bottom_left);
    return top + fade.y * (bottom - top);
}

fn encode_srgb(linear: vec3<f32>) -> vec3<f32> {
    let low = linear * 12.92;
    let high = 1.055 * pow(linear, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(high, low, linear <= vec3<f32>(0.0031308));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let cell_minimum = position.xy - vec2<f32>(0.5);
    let covered_minimum = max(cell_minimum, parameters.clip.xy);
    let covered_maximum = min(cell_minimum + vec2<f32>(1.0), parameters.clip.zw);
    let overlap = clamp(covered_maximum - covered_minimum, vec2<f32>(0.0), vec2<f32>(1.0));
    let coverage = overlap.x * overlap.y;
    if coverage == 0.0 { return vec4<f32>(0.0); }
    // Sample inside the primitive, then apply its coverage once to premultiplied RGBA.
    let sample_center = covered_minimum + overlap * 0.5;

    let octaves = parameters.settings.x;
    let fractal = parameters.settings.y != 0u;
    var sum = vec4<f32>(0.0);
    var point = fma(sample_center, parameters.phase_step.zw, parameters.phase_step.xy);
    var wrap = parameters.stitching.xy;
    var period = parameters.stitching.zw;
    var weight = 1.0;
    for (var octave = 0u; octave < octaves; octave += 1u) {
        let sample = noise(point, wrap, period);
        sum += weight * select(abs(sample), sample, fractal);
        if octave + 1u < octaves {
            point *= 2.0;
            wrap *= 2;
            period *= 2;
            weight *= 0.5;
        }
    }

    let straight = clamp(select(sum, (sum + 1.0) * 0.5, fractal),
        vec4<f32>(0.0), vec4<f32>(1.0));
    var rgb = straight.rgb;
    if parameters.settings.z != 0u { rgb = encode_srgb(rgb); }
    return vec4<f32>(rgb * straight.a, straight.a) * coverage;
}
