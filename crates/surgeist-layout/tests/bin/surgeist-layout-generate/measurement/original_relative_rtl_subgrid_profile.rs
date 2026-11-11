//! Exact original positioned RTL Subgrid capture inputs.

pub(crate) const SOURCE: &str = "html/subgrid/subgrid_abspos_relative_rtl_column_3_to_5.html";
pub(crate) const SOURCE_SHA256: &str =
    "2b6237299e4175920e98f6325ae3da048fdafc897e5193f696ac13e4f575a47e";

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    (source == SOURCE).then_some(SOURCE_SHA256)
}

pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/original_relative_rtl_subgrid.js";
pub(crate) const SCRIPT: &str = include_str!("original_relative_rtl_subgrid.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("original_relative_rtl_subgrid_request.json");

pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == crate::capture_helper_identity::COMPUTED_MARGIN_HELPER_SHA256
}
