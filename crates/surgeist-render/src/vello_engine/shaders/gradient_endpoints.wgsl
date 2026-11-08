// Surgeist-owned extension to the checked Vello fine shaders; see NOTICE-VELLO.md.
// Mode 3 names seven exact RGBA8 words:
// kind, payload, count, first bits, last bits, first RGBA8, last RGBA8.
// Kinds: 0 PadRamp, 1 ProgramPad, 2 ProgramRepeat. Program records have
// position, premultiplied R/G/B/A, outgoing kind and exponent (seven words).
fn gradient_endpoint_texel(index: u32) -> vec4<f32> {
    return textureLoad(gradients, vec2<i32>(i32(index % 512u), i32(index / 512u)), 0);
}

fn gradient_endpoint_word(index: u32) -> u32 {
    return pack4x8unorm(gradient_endpoint_texel(index));
}

// Keep subnormal control thresholds as words, even on FTZ implementations.
fn gradient_endpoint_ordered_bits(bits: u32) -> u32 {
    var canonical = bits;
    if ((bits & 0x7fffffffu) == 0u) {
        canonical = 0u;
    }
    return select(canonical ^ 0x80000000u, ~canonical, (canonical & 0x80000000u) != 0u);
}

fn gradient_program_color(record: u32) -> vec4<f32> {
    return vec4<f32>(
        bitcast<f32>(gradient_endpoint_word(record + 1u)),
        bitcast<f32>(gradient_endpoint_word(record + 2u)),
        bitcast<f32>(gradient_endpoint_word(record + 3u)),
        bitcast<f32>(gradient_endpoint_word(record + 4u)));
}

fn gradient_program_weight(q: f32, a: f32, b: f32, kind: u32, exponent: f32) -> f32 {
    // Exact boundary/run handling happened before entry. These singular limits
    // must branch before division or logarithms, including zero-width intervals.
    if (kind == 2u) { return 1.0; }
    if (kind == 3u) { return 0.0; }
    let s = q - a;
    let r = b - a;
    var p = s / r;
    if (p < 0.0) { p = 0.0; }
    if (p > 1.0) { p = 1.0; }
    if (kind == 0u) { return p; }
    // This exact expression sequence is owned by the CPU weight certificate.
    if ((bitcast<u32>(p) & 0x7f800000u) == 0u) { return 0.0; }
    let l = log2(p);
    var e = exponent * l;
    if (e > 0.0) { e = 0.0; }
    if (e < -126.0) { return 0.0; }
    var weight = exp2(e);
    if (weight < 0.0) { weight = 0.0; }
    if (weight > 1.0) { weight = 1.0; }
    return weight;
}

fn ordinary_gradient_color(raw: f32, descriptor: u32) -> vec4<f32> {
    let kind = gradient_endpoint_word(descriptor);
    let payload = gradient_endpoint_word(descriptor + 1u);
    var q = raw;
    if (kind == 2u) {
        q = raw - floor(raw);
    }
    let q_bits = bitcast<u32>(q);
    // A NaN has no endpoint side. PadRamp retains the native NaN lookup below.
    if ((q_bits & 0x7fffffffu) <= 0x7f800000u) {
        let first = gradient_endpoint_ordered_bits(gradient_endpoint_word(descriptor + 3u));
        let last = gradient_endpoint_ordered_bits(gradient_endpoint_word(descriptor + 4u));
        let coordinate = gradient_endpoint_ordered_bits(q_bits);
        if (first == last) {
            if (coordinate < first) {
                return gradient_endpoint_texel(descriptor + 5u);
            }
            return gradient_endpoint_texel(descriptor + 6u);
        }
        if (coordinate <= first) {
            return gradient_endpoint_texel(descriptor + 5u);
        }
        if (coordinate >= last) {
            return gradient_endpoint_texel(descriptor + 6u);
        }
        if (kind != 0u) {
            let count = gradient_endpoint_word(descriptor + 2u);
            // CPU custody proves count >= 2 and the complete record range.
            // Upper bound chooses the source-last record in an interior run.
            var lo = 0u;
            var hi = count;
            for (var iteration = 0u; iteration < 32u && lo < hi; iteration += 1u) {
                let mid = lo + (hi - lo) / 2u;
                let position = gradient_endpoint_ordered_bits(gradient_endpoint_word(payload + mid * 7u));
                if (position <= coordinate) {
                    lo = mid + 1u;
                } else {
                    hi = mid;
                }
            }
            let lower = payload + (lo - 1u) * 7u;
            let upper = payload + lo * 7u;
            let lower_bits = gradient_endpoint_word(lower);
            let left = gradient_program_color(lower);
            if (coordinate == gradient_endpoint_ordered_bits(lower_bits)) {
                return left;
            }
            let right = gradient_program_color(upper);
            let segment = gradient_endpoint_word(lower + 5u);
            let exponent = bitcast<f32>(gradient_endpoint_word(lower + 6u));
            let weight = gradient_program_weight(q, bitcast<f32>(lower_bits),
                bitcast<f32>(gradient_endpoint_word(upper)), segment, exponent);
            return mix(left, right, vec4<f32>(weight));
        }
    }
    if (kind != 0u) {
        // Qualified programs execute only finite fields. Preserve a bounded
        // result if unsupported arithmetic nevertheless yields NaN.
        return gradient_endpoint_texel(descriptor + 5u);
    }
    let x = i32(round(clamp(raw, 0.0, 1.0) * f32(GRADIENT_WIDTH - 1)));
    return textureLoad(gradients, vec2<i32>(x, i32(payload)), 0);
}

const EXTEND_PAD: u32 = 0u;
