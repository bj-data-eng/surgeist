// Surgeist-owned extension to the checked Vello fine shaders; see NOTICE-VELLO.md.
// Mode 3 stores a flat texel offset, followed by five RGBA8 descriptor texels:
// first offset bits, last offset bits, native ramp row, first color, last color.
fn gradient_endpoint_texel(index: u32) -> vec4<f32> {
    return textureLoad(gradients, vec2<i32>(i32(index % 512u), i32(index / 512u)), 0);
}

fn gradient_endpoint_word(index: u32) -> u32 {
    return pack4x8unorm(gradient_endpoint_texel(index));
}

// Compare binary32 representations without converting endpoint words to floats.
// In particular, an admitted subnormal threshold must not flush to zero. The
// coordinate is the actual native computed value; earlier arithmetic is unchanged.
fn gradient_endpoint_ordered_bits(bits: u32) -> u32 {
    var canonical = bits;
    if ((bits & 0x7fffffffu) == 0u) {
        canonical = 0u;
    }
    return select(canonical ^ 0x80000000u, ~canonical, (canonical & 0x80000000u) != 0u);
}

fn ordinary_gradient_color(raw: f32, descriptor: u32) -> vec4<f32> {
    let raw_bits = bitcast<u32>(raw);
    let row = gradient_endpoint_word(descriptor + 2u);
    // A NaN has no endpoint side. Preserve the previous native Pad lookup rather
    // than inventing endpoint semantics for unsupported field arithmetic.
    if ((raw_bits & 0x7fffffffu) <= 0x7f800000u) {
        let first = gradient_endpoint_ordered_bits(gradient_endpoint_word(descriptor));
        let last = gradient_endpoint_ordered_bits(gradient_endpoint_word(descriptor + 1u));
        let coordinate = gradient_endpoint_ordered_bits(raw_bits);
        if (first == last) {
            // The ordinary coincident transition is right-continuous.
            if (coordinate < first) {
                return gradient_endpoint_texel(descriptor + 3u);
            }
            return gradient_endpoint_texel(descriptor + 4u);
        }
        if (coordinate <= first) {
            return gradient_endpoint_texel(descriptor + 3u);
        }
        if (coordinate >= last) {
            return gradient_endpoint_texel(descriptor + 4u);
        }
    }
    let x = i32(round(clamp(raw, 0.0, 1.0) * f32(GRADIENT_WIDTH - 1)));
    return textureLoad(gradients, vec2<i32>(x, i32(row)), 0);
}

const EXTEND_PAD: u32 = 0u;
