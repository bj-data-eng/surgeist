//! Exact inputs for the original Grid axis fixtures.

pub(crate) const SOURCES: &[&str; 9] = &[
    "html/grid/grid_axes_horizontal_parent_orthogonal_child.html",
    "html/grid/grid_axes_horizontal_tb_parallel.html",
    "html/grid/grid_axes_sideways_lr_parallel.html",
    "html/grid/grid_axes_sideways_opposing.html",
    "html/grid/grid_axes_sideways_rl_parallel.html",
    "html/grid/grid_axes_vertical_lr_parallel.html",
    "html/grid/grid_axes_vertical_opposing.html",
    "html/grid/grid_axes_vertical_parent_orthogonal_child.html",
    "html/grid/grid_axes_vertical_rl_parallel.html",
];
const DIGESTS: [&str; SOURCES.len()] = [
    "9a6ed599fdff05d8d6ed67906ddfb8c83788709bad8eeb0c553b665db31f89fd",
    "ee07393be63a6f765d058673d343a272cabec992207babbc76093825d04979e8",
    "5ec60c7c7bcbc7a6c50043eb1664594225937956b1e7c2bca7d2e10a710f88a4",
    "1e80a3db57d3ef15950fb933cecd7cfb4bc46962f77228c3e363c6ffbc614e67",
    "bc0b13dd1c51f2ed5dd47f9e52c54ea8c0f272efd01b3de88f16b2ba56aa5c62",
    "65bed02d82b9f354e6f204fc45ccd0189a3f1f9c2f960c7daa39ffff5768c547",
    "1ae93fda4f44afefe59b34beb070543941a4bfc2856062660b398dd7592a1632",
    "b97113a60c81944899d36045c92ecd94c10e838595884fbbfda9416b62a2cea6",
    "317b8415b1b2a12142b6997fffb3b124f3bfaeab0fb3cebb351606ff9952f872",
];

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .zip(DIGESTS)
        .find_map(|(path, digest)| (*path == source).then_some(digest))
}

pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/original_grid_axes.js";
pub(crate) const SCRIPT: &str = include_str!("original_grid_axes.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("original_grid_axes_request.json");

pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == crate::capture_helper_identity::FIXED_ORTHOGONAL_HELPER_SHA256
        || digest == crate::capture_helper_identity::COMPUTED_MARGIN_HELPER_SHA256
}
