//! Exact source inputs for the fixed-width original Flex parent capture.

pub(crate) const SOURCES: &[&str] = &[
    "html/grid/grid_available_space_greater_than_max_content.html",
    "html/grid/grid_available_space_smaller_than_max_content.html",
    "html/grid/grid_available_space_smaller_than_min_content.html",
];
const DIGESTS: [&str; SOURCES.len()] = [
    "83d57a71bc2ebf2abe3e4342cfc1532b61d4902b0605bc646abe033c4ebb7d57",
    "95abfc426f1337c0a4c936852eaaab07b4a1d651366ecf99727d0844a67b84a0",
    "eb8d3865e36202ddd5b1751bbcf3b428bbb0f5482f7e96af519a73afc4cd469d",
];

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .zip(DIGESTS)
        .find_map(|(path, digest)| (*path == source).then_some(digest))
}

pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/original_flex_parent.js";
pub(crate) const SCRIPT: &str = include_str!("original_flex_parent.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("original_flex_parent_request.json");

pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == crate::capture_helper_identity::FIXED_ORTHOGONAL_HELPER_SHA256
}
