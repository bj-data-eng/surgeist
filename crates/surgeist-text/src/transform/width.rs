//! Single raw wide/narrow mapping, without recursive compatibility normalization.

pub(super) fn map(cp: u32) -> Option<u32> {
    let mappings = super::width_data::MAPPINGS;
    mappings
        .binary_search_by_key(&cp, |&(input, _)| input)
        .ok()
        .map(|index| mappings[index].1)
}
