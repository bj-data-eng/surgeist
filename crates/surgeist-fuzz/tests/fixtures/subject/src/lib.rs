//! Deliberately faulty subject used only to demonstrate fuzz authoring.
#![forbid(unsafe_code)]

/// Independent expectation: a byte slice reports its length. The two markers
/// intentionally violate that expectation with distinguishable failure classes.
pub fn reported_bytes(bytes: &[u8]) -> usize {
    bytes.len()
        + if bytes.windows(4).any(|part| part == b"BUG!") {
            1
        } else if bytes.windows(4).any(|part| part == b"ALT!") {
            2
        } else {
            0
        }
}

/// Independent expectation: every UTF-16 code unit counts once, even when it
/// is a non-scalar surrogate. This controlled subject violates that rule.
pub fn reported_units(units: &[u16]) -> usize {
    units.len() + usize::from(units.contains(&0xd800))
}

/// A toy fixture relation. Invalid references reject without changing state.
pub fn use_node(created: &mut [bool; 8], id: u8) -> bool {
    created.get(usize::from(id)).copied().unwrap_or(false)
}
