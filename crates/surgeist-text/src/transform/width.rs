//! Single raw wide/narrow mapping, without recursive compatibility normalization.

pub(super) fn map(cp: u32) -> Option<u32> {
    let mappings = super::width_data::MAPPINGS;
    mappings
        .binary_search_by_key(&cp, |&(input, _)| input)
        .ok()
        .map(|index| mappings[index].1)
}

/// Reverse the same single Unicode wide/narrow relation. This is not NFKC and
/// does not infer an input from font output or from measured character width.
pub(super) fn inverse(cp: u32) -> Option<u32> {
    super::width_data::MAPPINGS
        .iter()
        .find_map(|&(input, output)| (output == cp).then_some(input))
}
