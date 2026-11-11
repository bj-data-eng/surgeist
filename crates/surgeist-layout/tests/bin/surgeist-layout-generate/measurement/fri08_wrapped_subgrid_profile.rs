//! Exact sources and browser measurement inputs for the wrapped Subgrid siblings.

pub(crate) const SOURCES: [(&str, &str); 2] = [
    (
        "html/subgrid/subgrid_sibling_overflow_footer_second_matches_first.html",
        "74c92a5ac60e37031a7f992ea1748a8ee4dc8ceaeb30b4a8e224e145f169d94a",
    ),
    (
        "html/subgrid/subgrid_sibling_overflow_footer_third_matches_first.html",
        "14445537ec1510bf93684d63b798627811d5d42dd0935e8937dbb188251d243e",
    ),
];
pub(crate) const PROFILE: &str = "ahem-10px-fri08-wrapped-subgrid";
pub(crate) const REQUEST: &[u8] = include_bytes!("fri08_wrapped_subgrid_request.json");
pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/fri08_wrapped_subgrid.js";
pub(crate) const SCRIPT: &str = include_str!("fri08_wrapped_subgrid.js");
pub(crate) const LABELS: [&str; 2] = [
    "Top",
    "Long text long enough to wrap onto multiple lines so that overflow:hidden can clip the overflowing portion",
];

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .find_map(|&(path, digest)| (source == path).then_some(digest))
}

pub(crate) fn valid_ranges(
    label: &str,
    ranges: &[crate::fri07_intrinsic_flex_profile::WordRange],
) -> bool {
    let mut prior_line = None;
    !ranges.is_empty()
        && ranges.iter().all(|range| {
            let prior = prior_line.replace(range.line);
            range.start < range.end
                && range.end <= label.len()
                && prior.map_or(range.line == 0, |line| {
                    range.line == line || range.line.checked_sub(1) == Some(line)
                })
                && range.x.is_finite()
                && range.y.is_finite()
                && range.width > 0.0
                && range.height > 0.0
                && (range.x + range.width).is_finite()
                && (range.y + range.height).is_finite()
        })
        && ranges
            .iter()
            .flat_map(|range| range.start..range.end)
            .eq(label
                .encode_utf16()
                .enumerate()
                .filter_map(|(index, unit)| (unit != u16::from(b' ')).then_some(index)))
}

pub(crate) fn valid_clip([width, height, scroll_width, scroll_height]: [u32; 4]) -> bool {
    width > 0
        && height > 0
        && width <= scroll_width
        && height <= scroll_height
        && scroll_width <= i32::MAX as u32
        && scroll_height <= i32::MAX as u32
}
