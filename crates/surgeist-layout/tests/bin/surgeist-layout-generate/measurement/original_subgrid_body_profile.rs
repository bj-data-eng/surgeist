//! Immutable inputs for the original subgrid body capture.

pub(crate) const SOURCE: &str = "html/subgrid/subgrid_gap_percentage_border_bottom.html";
pub(crate) const SOURCE_SHA256: &str =
    "db5cc28750958aa0df6c0692caafd131d284858e435e9fb0a1f6330527deb568";
pub(crate) const HELPER_SHA256: &str =
    "afad9adf5a3915d88e94b0e3ad2d4768c242783422a0fb3bbc9a277492c10f9c";
pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/original_subgrid_body.js";
pub(crate) const SCRIPT: &str = include_str!("original_subgrid_body.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("original_subgrid_body_request.json");

/// Producing helpers qualified for this observation profile. Global helper
/// recognition grants no capability.
pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == HELPER_SHA256
        || digest == crate::capture_helper_identity::SCRIPT_CUSTODY_HELPER_SHA256
        || digest == crate::capture_helper_identity::FIXED_ORTHOGONAL_HELPER_SHA256
        || digest == crate::capture_helper_identity::COMPUTED_MARGIN_HELPER_SHA256
}
