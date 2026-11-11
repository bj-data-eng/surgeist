//! Exact source identities for the empty-line break bands.

const HORIZONTAL: &str = "html/block/block_br_empty_lines_metrics.html";
const VERTICAL: &str = "html/block/block_br_vertical_rl_empty_lines_metrics.html";
pub(crate) const SOURCES: &[&str; 2] = &[HORIZONTAL, VERTICAL];
const DIGESTS: [&str; SOURCES.len()] = [
    "7d4717c7ae542e751bab7601b4c94e07a4edf1d1bc399de5351da9c09979b4e1",
    "170fc32902b9543fcdeca4d142dfbafa878e7a1fa69f2223f982b40071334d07",
];

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .zip(DIGESTS)
        .find_map(|(path, digest)| (*path == source).then_some(digest))
}

pub(crate) const HELPER_SHA256: &str =
    "afad9adf5a3915d88e94b0e3ad2d4768c242783422a0fb3bbc9a277492c10f9c";
pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/empty_line_br_font_band.js";
pub(crate) const SCRIPT: &str = include_str!("empty_line_br_font_band.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("empty_line_br_font_band_request.json");

/// Producing helpers qualified for this observation profile. Global helper
/// recognition grants no capability.
pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == HELPER_SHA256
        || digest == crate::capture_helper_identity::SCRIPT_CUSTODY_HELPER_SHA256
        || digest == crate::capture_helper_identity::FIXED_ORTHOGONAL_HELPER_SHA256
        || digest == crate::capture_helper_identity::COMPUTED_MARGIN_HELPER_SHA256
}
