//! Exact inputs for the original atomic percentage capture.

pub(crate) const SOURCE: &str = "html/block/fri06_atomic_inline_percentage_block_size.html";
pub(crate) const SOURCE_SHA256: &str =
    "73709999d0514673ee1e7eaead635c37afdce8b6a4467bd5f1ca328c9b7112f1";
pub(crate) const HELPER_SHA256: &str =
    "afad9adf5a3915d88e94b0e3ad2d4768c242783422a0fb3bbc9a277492c10f9c";
pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/prepared_atomic_percentage.js";
pub(crate) const SCRIPT: &str = include_str!("prepared_atomic_percentage.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("prepared_atomic_percentage_request.json");

/// The retained producer and the source-script-custody producer qualified for
/// this observation profile. Other globally known helpers grant no capability.
pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == HELPER_SHA256
        || digest == crate::capture_helper_identity::SCRIPT_CUSTODY_HELPER_SHA256
}
