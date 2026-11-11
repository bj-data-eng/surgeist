//! Exact inputs for the original vertical prepared break bands.

const LR: &str = "html/block/block_br_vertical_lr_inline_block_metrics.html";
const RL: &str = "html/block/block_br_vertical_rl_inline_block_metrics.html";
const RL_RTL: &str = "html/block/block_br_vertical_rl_rtl_inline_block_metrics.html";
pub(crate) const SOURCES: &[&str; 3] = &[LR, RL, RL_RTL];
const DIGESTS: [&str; SOURCES.len()] = [
    "a9fbabd1b36f1e5f77e650bf00370c1ce9aced4798a91cf19de414fddbafda63",
    "e38e24ebc52f83041252b155f4ebb403a572ff64034c75d82ced94deb8207595",
    "2681d0c5fbf33644e5d10a74f26986de3a1fd53833c0f19ffb73f805c465bfaf",
];
pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .zip(DIGESTS)
        .find_map(|(path, digest)| (*path == source).then_some(digest))
}

pub(crate) const HELPER_SHA256: &str =
    "afad9adf5a3915d88e94b0e3ad2d4768c242783422a0fb3bbc9a277492c10f9c";
pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/vertical_br_font_band.js";
pub(crate) const SCRIPT: &str = include_str!("vertical_br_font_band.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("vertical_br_font_band_request.json");

/// Producing helpers qualified for this observation profile. Global helper
/// recognition grants no capability.
pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == HELPER_SHA256
        || digest == crate::capture_helper_identity::SCRIPT_CUSTODY_HELPER_SHA256
        || digest == crate::capture_helper_identity::FIXED_ORTHOGONAL_HELPER_SHA256
        || digest == crate::capture_helper_identity::COMPUTED_MARGIN_HELPER_SHA256
}
