//! Source and independent text measurements for the intrinsic Grid Lanes pair.

use serde::{Deserialize, Serialize};

pub(crate) const SOURCES: [(&str, &str); 2] = [
    (
        "html/grid-lanes/grid_lanes_min_content_container_sizing.html",
        "4bf761c29cb69a3b68046ec6c28938c653ac846805a63df82249d1a31e47131f",
    ),
    (
        "html/grid-lanes/grid_lanes_max_content_container_sizing.html",
        "997009e354f48de29fec1bf88e11c89b9cd4b4d930cca4d33ce4cc0f4a0ab1d3",
    ),
];
pub(crate) const PROFILE: &str = "monospace-15px-fri08-grid-lanes";
pub(crate) const REQUEST: &[u8] = include_bytes!("fri08_grid_lanes_text_request.json");
pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/fri08_grid_lanes_text.js";
pub(crate) const SCRIPT: &str = include_str!("fri08_grid_lanes_text.js");
pub(crate) const LABELS: [&str; 4] = ["Number 1", "Number 2", "Number 3", "Number 4"];
pub(crate) const WIDTHS: [&str; 2] = ["min-content", "max-content"];

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .find_map(|&(path, digest)| (source == path).then_some(digest))
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Probe {
    pub(crate) inline_size: f32,
    pub(crate) block_size: f32,
    pub(crate) word_lines: [u8; 2],
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Sizes {
    pub(crate) min_content: Probe,
    pub(crate) max_content: Probe,
    pub(crate) last_wrap: Probe,
    pub(crate) first_fit: Probe,
    pub(crate) zero: Probe,
}

impl Sizes {
    pub(crate) fn valid(self) -> bool {
        let Self {
            min_content,
            max_content,
            last_wrap,
            first_fit,
            zero,
        } = self;
        [min_content, max_content, last_wrap, first_fit, zero]
            .into_iter()
            .all(|probe| probe.inline_size.is_finite() && probe.inline_size >= 0.0)
            && min_content.inline_size > 0.0
            && last_wrap.inline_size >= min_content.inline_size
            && first_fit.inline_size <= max_content.inline_size
            && first_fit.inline_size - last_wrap.inline_size == 1.0 / 64.0
            && zero.inline_size == 0.0
            && [min_content, last_wrap, zero]
                .into_iter()
                .all(|probe| probe.block_size == 30.0 && probe.word_lines == [0, 1])
            && [max_content, first_fit]
                .into_iter()
                .all(|probe| probe.block_size == 15.0 && probe.word_lines == [0, 0])
    }
}
