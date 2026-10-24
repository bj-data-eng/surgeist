//! The post-Phase-I, pre-analysis presentation transformation owner.
//!
//! Mapping facts transcribed from CSS Text Module Level 4, 14 August 2026,
//! Appendix G: https://www.w3.org/TR/2026/WD-css-text-4-20260814/#small-kana
//! Copyright © 2026 World Wide Web Consortium. This adapts the table to Rust
//! scalar matching; Text owns execution and source correspondence. Retained
//! terms: licenses/w3c/software-license-2023.txt (relative to the crate root).
//!
//! Atoms retain exact original and Phase-I associations. Only semantic code
//! points and presentation coordinates change; plaintext never reads this output.

use crate::{Result, engine::resolve, projection::SourceProjection};

mod casing;
mod execute;
mod greek;
mod irish;
pub(crate) mod relation;

pub(crate) fn apply(
    input: SourceProjection,
    styles: &[resolve::AnalyzedStyleRun],
    processed: &crate::ProcessedSource,
    paragraph: &crate::protocol::paragraph::ParagraphData,
) -> Result<(SourceProjection, std::sync::Arc<relation::TransformMap>)> {
    execute::apply(input, styles, processed, paragraph)
}

/// Complete CSS Text 4 (14 August 2026), normative Appendix G, Small Kana
/// Mappings. This is neither width conversion nor Unicode normalization.
fn full_size_kana(cp: u32) -> Option<char> {
    Some(match cp {
        0x3041 => '\u{3042}',
        0x3043 => '\u{3044}',
        0x3045 => '\u{3046}',
        0x3047 => '\u{3048}',
        0x3049 => '\u{304a}',
        0x3095 => '\u{304b}',
        0x3096 => '\u{3051}',
        0x1b132 => '\u{3053}',
        0x3063 => '\u{3064}',
        0x3083 => '\u{3084}',
        0x3085 => '\u{3086}',
        0x3087 => '\u{3088}',
        0x308e => '\u{308f}',
        0x1b150 => '\u{3090}',
        0x1b151 => '\u{3091}',
        0x1b152 => '\u{3092}',
        0x30a1 => '\u{30a2}',
        0x30a3 => '\u{30a4}',
        0x30a5 => '\u{30a6}',
        0x30a7 => '\u{30a8}',
        0x30a9 => '\u{30aa}',
        0x30f5 => '\u{30ab}',
        0x31f0 => '\u{30af}',
        0x30f6 => '\u{30b1}',
        0x1b155 => '\u{30b3}',
        0x31f1 => '\u{30b7}',
        0x31f2 => '\u{30b9}',
        0x30c3 => '\u{30c4}',
        0x31f3 => '\u{30c8}',
        0x31f4 => '\u{30cc}',
        0x31f5 => '\u{30cf}',
        0x31f6 => '\u{30d2}',
        0x31f7 => '\u{30d5}',
        0x31f8 => '\u{30d8}',
        0x31f9 => '\u{30db}',
        0x31fa => '\u{30e0}',
        0x30e3 => '\u{30e4}',
        0x30e5 => '\u{30e6}',
        0x30e7 => '\u{30e8}',
        0x31fb => '\u{30e9}',
        0x31fc => '\u{30ea}',
        0x31fd => '\u{30eb}',
        0x31fe => '\u{30ec}',
        0x31ff => '\u{30ed}',
        0x30ee => '\u{30ef}',
        0x1b164 => '\u{30f0}',
        0x1b165 => '\u{30f1}',
        0x1b166 => '\u{30f2}',
        0x1b167 => '\u{30f3}',
        0xff67 => '\u{ff71}',
        0xff68 => '\u{ff72}',
        0xff69 => '\u{ff73}',
        0xff6a => '\u{ff74}',
        0xff6b => '\u{ff75}',
        0xff6f => '\u{ff82}',
        0xff6c => '\u{ff94}',
        0xff6d => '\u{ff95}',
        0xff6e => '\u{ff96}',
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
