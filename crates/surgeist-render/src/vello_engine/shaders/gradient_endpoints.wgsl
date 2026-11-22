// Surgeist-owned extension to the checked Vello fine shaders; see NOTICE-VELLO.md.
// Mode 3 names eight exact RGBA8 words:
// kind, payload, count, first bits, last bits, first RGBA8, last RGBA8, phase bits.
// Kinds: 0 PadRamp, 1 ProgramPad, 2 ProgramRepeat, 3 DistanceStep,
// 4 DistanceProgramPad, 5 DistanceProgramRepeat, 6 CirclePad, 7 CircleRepeat.
// Circle descriptors replace the last five words with span length, span
// fraction, origin length, origin fraction and a zero word. Circle records
// add fraction after length, with repeating records relative to the origin.
// Program records have
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
    return gradient_program_weight_ratio(q - a, b - a, kind, exponent);
}

fn gradient_program_weight_ratio(numerator: f32, denominator: f32, kind: u32, exponent: f32) -> f32 {
    var p = numerator / denominator;
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

fn gradient_circle_position(record: u32) -> vec2<u32> {
    return vec2<u32>(gradient_endpoint_word(record), gradient_endpoint_word(record + 1u));
}

fn gradient_circle_key(bits: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(gradient_endpoint_ordered_bits(bits.x), gradient_endpoint_ordered_bits(bits.y));
}

fn gradient_circle_norm(x: f32, y: f32) -> f32 {
    let magnitude = max(abs(x), abs(y));
    if (magnitude < 0x1p-126f) { return 0.0; }
    if (x == 0.0 || y == 0.0) { return magnitude; }
    let scaled_x = x / magnitude;
    let scaled_y = y / magnitude;
    let squared_radius = scaled_x * scaled_x + scaled_y * scaled_y;
    return max(magnitude, magnitude * sqrt(squared_radius));
}

// Units are 2^-149. A finite binary32 has magnitude < 2^128 and a
// certified cycle has |k| <= 2^24. Four signed terms therefore fit in
// 320 bits, including sign, even before cancellation.
const CIRCLE_DYADIC_LIMBS: u32 = 10u;
alias CircleDyadic = array<u32, CIRCLE_DYADIC_LIMBS>;

fn gradient_circle_product(a: u32, b: u32) -> vec2<u32> {
    let al = a & 0xffffu;
    let ah = a >> 16u;
    let bl = b & 0xffffu;
    let bh = b >> 16u;
    let low = al * bl;
    let cross_a = al * bh;
    let cross_b = ah * bl;
    let middle = (low >> 16u) + (cross_a & 0xffffu) + (cross_b & 0xffffu);
    let high = ah * bh + (cross_a >> 16u) + (cross_b >> 16u) + (middle >> 16u);
    return vec2<u32>((low & 0xffffu) | (middle << 16u), high);
}

fn gradient_circle_accumulate(sum: ptr<function, CircleDyadic>, bits: u32, coefficient: i32) {
    let magnitude = bits & 0x7fffffffu;
    if (magnitude == 0u || coefficient == 0) { return; }
    let biased_exponent = magnitude >> 23u;
    let fraction = magnitude & 0x7fffffu;
    let significand = select(fraction | 0x800000u, fraction, biased_exponent == 0u);
    let shift = select(biased_exponent - 1u, 0u, biased_exponent == 0u);
    let product = gradient_circle_product(significand, u32(abs(coefficient)));
    let base = shift / 32u;
    let offset = shift % 32u;
    var words = vec3<u32>(product, 0u);
    if (offset != 0u) {
        let remaining = 32u - offset;
        words = vec3<u32>(product.x << offset,
            (product.y << offset) | (product.x >> remaining),
            product.y >> remaining);
    }
    let subtract = ((bits & 0x80000000u) != 0u) != (coefficient < 0);
    var carry = 0u;
    for (var i = base; i < CIRCLE_DYADIC_LIMBS; i += 1u) {
        let limb = i - base;
        var term = 0u;
        if (limb < 3u) { term = words[limb]; }
        let prior = (*sum)[i];
        if (subtract) {
            let difference = prior - term;
            (*sum)[i] = difference - carry;
            carry = u32(prior < term || difference < carry);
        } else {
            let combined = prior + term;
            let next = combined + carry;
            (*sum)[i] = next;
            carry = u32(combined < prior || next < combined);
        }
        if (limb >= 2u && carry == 0u) { break; }
    }
}

fn gradient_circle_dyadic_compare(value: CircleDyadic, control: u32) -> i32 {
    var difference = value;
    gradient_circle_accumulate(&difference, control, -1);
    if ((difference[CIRCLE_DYADIC_LIMBS - 1u] & 0x80000000u) != 0u) { return -1; }
    for (var i = 0u; i < CIRCLE_DYADIC_LIMBS; i += 1u) {
        if (difference[i] != 0u) { return 1; }
    }
    return 0;
}

// Convert a positive cancellation-free numerator to binary32 with a
// round-to-nearest-even guard and sticky bit. Direction was decided with
// the exact dyadic value, independently of this interpolation conversion.
fn gradient_circle_round_numerator(difference: CircleDyadic) -> f32 {
    var top = CIRCLE_DYADIC_LIMBS;
    loop {
        if (top == 0u) { return 0.0; }
        top -= 1u;
        if (difference[top] != 0u) { break; }
    }
    let highest = 32u * top + 31u - countLeadingZeros(difference[top]);
    if (highest < 23u) { return bitcast<f32>(difference[0]); }
    if (highest > 276u) { return 0x1.fffffep127f; }
    let shift = highest - 23u;
    let limb = shift / 32u;
    let offset = shift % 32u;
    var leading = difference[limb] >> offset;
    if (limb < top && offset != 0u) {
        leading |= difference[limb + 1u] << (32u - offset);
    }
    if (shift != 0u) {
        let guard = shift - 1u;
        let guard_limb = guard / 32u;
        let guard_mask = 1u << (guard % 32u);
        if ((difference[guard_limb] & guard_mask) != 0u) {
            var sticky = (difference[guard_limb] & (guard_mask - 1u)) != 0u;
            for (var i = 0u; i < guard_limb && !sticky; i += 1u) {
                sticky = difference[i] != 0u;
            }
            if (sticky || (leading & 1u) != 0u) { leading += 1u; }
        }
    }
    let exponent = (highest - 22u) << 23u;
    return bitcast<f32>(min(exponent + (leading - 0x800000u), 0x7f7fffffu));
}

fn gradient_circle_numerator(value: CircleDyadic, control: u32, reverse: bool) -> f32 {
    var difference = value;
    gradient_circle_accumulate(&difference, control, -1);
    if (reverse) {
        var carry = 1u;
        for (var i = 0u; i < CIRCLE_DYADIC_LIMBS; i += 1u) {
            difference[i] = ~difference[i] + carry;
            carry = u32(difference[i] == 0u && carry == 1u);
        }
    }
    return gradient_circle_round_numerator(difference);
}

struct CircleQuery {
    raw_bits: u32,
    repeat: bool,
    exact_length: CircleDyadic,
    exact_fraction: CircleDyadic,
}

fn gradient_circle_query_length(query: ptr<function, CircleQuery>, length: u32) -> i32 {
    if ((*query).repeat) {
        return gradient_circle_dyadic_compare((*query).exact_length, length);
    }
    let value = gradient_endpoint_ordered_bits((*query).raw_bits);
    let limit = gradient_endpoint_ordered_bits(length);
    if (value == limit) { return 0; }
    return select(1, -1, value < limit);
}

fn gradient_circle_query_compare(query: ptr<function, CircleQuery>, position: vec2<u32>) -> i32 {
    let leading = gradient_circle_query_length(query, position.x);
    if (leading != 0) { return leading; }
    if ((*query).repeat) {
        return gradient_circle_dyadic_compare((*query).exact_fraction, position.y);
    }
    let value = gradient_endpoint_ordered_bits(0u);
    let limit = gradient_endpoint_ordered_bits(position.y);
    if (value == limit) { return 0; }
    return select(1, -1, value < limit);
}

fn gradient_circle_repeat_query(query: ptr<function, CircleQuery>, raw: f32, descriptor: u32) {
    let span = vec2<u32>(gradient_endpoint_word(descriptor + 3u), gradient_endpoint_word(descriptor + 4u));
    let origin = vec2<u32>(gradient_endpoint_word(descriptor + 5u), gradient_endpoint_word(descriptor + 6u));
    let delta = raw - bitcast<f32>(origin.x);
    var cycle = i32(floor(delta / bitcast<f32>(span.x)));
    var length: CircleDyadic;
    gradient_circle_accumulate(&length, bitcast<u32>(raw), 1);
    gradient_circle_accumulate(&length, origin.x, -1);
    gradient_circle_accumulate(&length, span.x, -cycle);
    // The host proves the native quotient error is less than one. Correct
    // its integral candidate before checking the secondary seam direction.
    if (gradient_circle_dyadic_compare(length, 0u) < 0) {
        cycle -= 1;
        gradient_circle_accumulate(&length, span.x, 1);
    } else if (gradient_circle_dyadic_compare(length, span.x) >= 0) {
        cycle += 1;
        gradient_circle_accumulate(&length, span.x, -1);
    }
    var fraction: CircleDyadic;
    gradient_circle_accumulate(&fraction, origin.y, -1);
    gradient_circle_accumulate(&fraction, span.y, -cycle);
    if (gradient_circle_dyadic_compare(length, 0u) == 0
        && gradient_circle_dyadic_compare(fraction, 0u) < 0) {
        gradient_circle_accumulate(&length, span.x, 1);
        gradient_circle_accumulate(&fraction, span.y, 1);
    }
    (*query).exact_length = length;
    (*query).exact_fraction = fraction;
}

fn gradient_circle_weight(query: ptr<function, CircleQuery>, a: vec2<u32>, b: vec2<u32>, segment: u32, operand: f32) -> f32 {
    if (segment == 4u) {
        // An interior hint on the left ring jumps to a half mix on that ring
        // and to the right color as soon as the leading radius changes.
        return select(1.0, 0.5, gradient_circle_query_length(query, a.x) == 0);
    }
    if (segment == 5u) {
        if (gradient_circle_query_length(query, b.x) != 0) { return 0.0; }
        let right_fraction = bitcast<f32>(b.y);
        var numerator = right_fraction;
        if ((*query).repeat) {
            numerator = gradient_circle_numerator((*query).exact_fraction, b.y, true);
        }
        let distance = max(0.0, numerator / (right_fraction - operand));
        if (distance > 126.0) { return 0.0; }
        return exp2(-distance);
    }
    if (segment == 2u) { return 1.0; }
    if (segment == 3u) { return 0.0; }
    let same_ring = gradient_endpoint_ordered_bits(a.x) == gradient_endpoint_ordered_bits(b.x);
    let left = select(a.x, a.y, same_ring);
    let right = select(b.x, b.y, same_ring);
    let span = bitcast<f32>(right) - bitcast<f32>(left);
    var numerator = select(bitcast<f32>((*query).raw_bits), 0.0, same_ring) - bitcast<f32>(left);
    if ((*query).repeat) {
        if (same_ring) {
            numerator = gradient_circle_numerator((*query).exact_fraction, left, false);
        } else {
            numerator = gradient_circle_numerator((*query).exact_length, left, false);
        }
    }
    return gradient_program_weight_ratio(numerator, span, segment, operand);
}

fn gradient_circle_color(raw: f32, descriptor: u32, payload: u32, repeat: bool) -> vec4<f32> {
    let count = gradient_endpoint_word(descriptor + 2u);
    let first = gradient_circle_position(payload);
    let last_record = payload + (count - 1u) * 8u;
    let last = gradient_circle_position(last_record);
    let raw_bits = bitcast<u32>(raw);
    let magnitude = raw_bits & 0x7fffffffu;
    if (magnitude > 0x7f800000u || (repeat && magnitude == 0x7f800000u)) {
        return gradient_program_color(payload + 1u);
    }
    var query: CircleQuery;
    query.raw_bits = raw_bits;
    query.repeat = repeat;
    if (repeat) { gradient_circle_repeat_query(&query, raw, descriptor); }

    let first_side = gradient_circle_query_compare(&query, first);
    if (all(gradient_circle_key(first) == gradient_circle_key(last))) {
        if (first_side < 0) { return gradient_program_color(payload + 1u); }
        return gradient_program_color(last_record + 1u);
    }
    if (first_side <= 0) { return gradient_program_color(payload + 1u); }
    if (gradient_circle_query_compare(&query, last) >= 0) { return gradient_program_color(last_record + 1u); }

    // First/last boundary colors are handled above. The upper bound search
    // chooses the last record in every coincident interior run.
    var lo = 0u;
    var hi = count;
    for (var iteration = 0u; iteration < 32u && lo < hi; iteration += 1u) {
        let mid = lo + (hi - lo) / 2u;
        let position = gradient_circle_position(payload + mid * 8u);
        if (gradient_circle_query_compare(&query, position) < 0) {
            hi = mid;
        } else {
            lo = mid + 1u;
        }
    }
    let lower = payload + (lo - 1u) * 8u;
    let upper = payload + lo * 8u;
    let left_position = gradient_circle_position(lower);
    let left = gradient_program_color(lower + 1u);
    if (gradient_circle_query_compare(&query, left_position) == 0) { return left; }
    let right = gradient_program_color(upper + 1u);
    let segment = gradient_endpoint_word(lower + 6u);
    let operand = bitcast<f32>(gradient_endpoint_word(lower + 7u));
    let weight = gradient_circle_weight(&query, left_position, gradient_circle_position(upper), segment, operand);
    return mix(left, right, vec4<f32>(weight));
}

fn ordinary_gradient_color(raw: f32, descriptor: u32) -> vec4<f32> {
    let kind = gradient_endpoint_word(descriptor);
    let payload = gradient_endpoint_word(descriptor + 1u);
    if (kind == 6u || kind == 7u) {
        return gradient_circle_color(raw, descriptor, payload, kind == 7u);
    }
    var q = raw;
    if (kind >= 3u) {
        // Exact representation fold precedes offsets, phase and endpoints.
        // In particular this preserves subnormal cutoff discrimination.
        q = bitcast<f32>(bitcast<u32>(q) & 0x7fffffffu);
    }
    if (kind == 5u) {
        let phase = bitcast<f32>(gradient_endpoint_word(descriptor + 7u));
        q = q - phase;
        q = q - floor(q);
    } else if (kind == 2u) {
        q = q - floor(q);
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
        if (kind != 0u && kind != 3u) {
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
