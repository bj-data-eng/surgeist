//! The ranges of the "non-ASCII ident code point" definition in section 4.2 of
//! the pinned edition (CSS Syntax Module Level 3, 1 October 2026), copied from
//! its text, as an expectation that does not ask the crate's predicate.

/// Section 4.2, "non-ASCII ident code point": a code point whose value is any
/// of these bullets. All of the ranges are inclusive.
pub const ADMITTED: [(u32, u32); 15] = [
    // U+00B7
    (0x00b7, 0x00b7),
    // between U+00C0 and U+00D6
    (0x00c0, 0x00d6),
    // between U+00D8 and U+00F6
    (0x00d8, 0x00f6),
    // between U+00F8 and U+037D
    (0x00f8, 0x037d),
    // between U+037F and U+1FFF
    (0x037f, 0x1fff),
    // U+200C
    (0x200c, 0x200c),
    // U+200D
    (0x200d, 0x200d),
    // U+203F
    (0x203f, 0x203f),
    // U+2040
    (0x2040, 0x2040),
    // between U+2070 and U+218F
    (0x2070, 0x218f),
    // between U+2C00 and U+2FEF
    (0x2c00, 0x2fef),
    // between U+3001 and U+D7FF
    (0x3001, 0xd7ff),
    // between U+F900 and U+FDCF
    (0xf900, 0xfdcf),
    // between U+FDF0 and U+FFFD
    (0xfdf0, 0xfffd),
    // greater than or equal to U+10000 (to the last code point, U+10FFFF)
    (0x1_0000, 0x10_ffff),
];

/// Whether section 4.2 admits `point` in an ident (a non-ASCII code point only).
pub fn admitted(point: char) -> bool {
    let value = u32::from(point);
    ADMITTED
        .iter()
        .any(|&(low, high)| (low..=high).contains(&value))
}
