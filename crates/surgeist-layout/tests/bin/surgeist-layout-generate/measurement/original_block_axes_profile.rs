//! Exact source and capture inputs for the original Block axis fixtures.

pub(crate) const SOURCES: &[&str; 5] = &[
    "html/block/block_axes_horizontal_tb.html",
    "html/block/block_axes_vertical_rl.html",
    "html/block/block_axes_vertical_lr.html",
    "html/block/block_axes_sideways_rl.html",
    "html/block/block_axes_sideways_lr.html",
];
const DIGESTS: [&str; SOURCES.len()] = [
    "158ae9aae424eb3607f7f8607d4f8193e3ec643feeb0d88f686083132cf3e202",
    "34072a33a30f4e31f97d0d795fec9b2b0677f93dddaf03b785c14127641dfd7e",
    "722bfeb44fc456adebd4fd48113dad52910e8a9d9660c75c167a15649d43c3fd",
    "1e801f54fa0ca03894716d821ba96942fa13fc9ae770203d03a8bdbb56f0e7ea",
    "bcbc0c8f660e1664db5936c019092b1f5aafaebaff841cd5a4ba1749c79d1ccc",
];

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .zip(DIGESTS)
        .find_map(|(path, digest)| (*path == source).then_some(digest))
}

pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/original_block_axes.js";
pub(crate) const SCRIPT: &str = include_str!("original_block_axes.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("original_block_axes_request.json");

pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == crate::capture_helper_identity::FIXED_ORTHOGONAL_HELPER_SHA256
}
