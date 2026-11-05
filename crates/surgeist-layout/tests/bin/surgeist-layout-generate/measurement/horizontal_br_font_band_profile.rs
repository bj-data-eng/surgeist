//! Exact inputs for the horizontal prepared break font-band capture.

pub(crate) const SOURCE: &str = "html/block/block_br_inline_block_metrics.html";
pub(crate) const SOURCE_SHA256: &str =
    "e80f4d521108ff7481d3d4779cf1498f198ee9808106cde762e69f203173ea90";
pub(crate) const HELPER_SHA256: &str =
    "afad9adf5a3915d88e94b0e3ad2d4768c242783422a0fb3bbc9a277492c10f9c";
pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/horizontal_br_font_band.js";
pub(crate) const SCRIPT: &str = include_str!("horizontal_br_font_band.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("horizontal_br_font_band_request.json");
