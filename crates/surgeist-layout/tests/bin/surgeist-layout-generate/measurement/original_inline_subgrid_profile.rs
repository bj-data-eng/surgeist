//! Exact original inline Subgrid alignment capture inputs.

pub(crate) const EDITION: &str = "original-inline-subgrid-alignment";
pub(crate) const SOURCES: [(&str, &str, &str); 2] = [
    (
        "html/subgrid/subgrid_alignment_002_center_start_item.html",
        "5d8a3620a4e98eb6779e4c5dffa3ca8a9fe732751564e7dd48c16d4c63875b1f",
        "horizontal-tb",
    ),
    (
        "html/subgrid/subgrid_alignment_002_center_start_orthogonal_item.html",
        "23c1d0a7df422aa43f565cecd3f04af49ab03411d4c2d1521d624c96ba0871d6",
        "vertical-lr",
    ),
];

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .find_map(|&(path, digest, _)| (path == source).then_some(digest))
}

pub(crate) fn subgrid_writing_mode(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .find_map(|&(path, _, mode)| (path == source).then_some(mode))
}

pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/original_inline_subgrid.js";
pub(crate) const SCRIPT: &str = include_str!("original_inline_subgrid.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("original_inline_subgrid_request.json");

pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == crate::capture_helper_identity::COMPUTED_MARGIN_HELPER_SHA256
}
