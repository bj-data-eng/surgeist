//! Exact MathML Core C.1 operation; complete original-node eligibility is
//! independent of the actual surviving Phase-I codepoint being mapped.
use crate::Text;

pub(super) fn map(cp: u32) -> Option<u32> {
    let mappings = super::math_data::MAPPINGS;
    mappings
        .binary_search_by_key(&cp, |&(original, _)| original)
        .ok()
        .map(|i| mappings[i].1)
}

pub(super) fn eligible(text: &Text) -> bool {
    // All pinned keys are BMP scalars. Isolated surrogates and supplementary
    // input cannot match; exact original content is never normalized.
    text.len() == 1 && map(u32::from(text.code_units()[0])).is_some()
}
