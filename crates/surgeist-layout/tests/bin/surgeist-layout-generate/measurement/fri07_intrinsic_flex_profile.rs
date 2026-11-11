use serde::{Deserialize, Serialize};

pub(crate) const SOURCE: &str = "html/flex/fri07_intrinsic_flex_basis.html";
pub(crate) const SOURCE_SHA256: &str =
    "a79a520ec55e6c45e8358d73e77343d8338de6878dc79b2f62dde7e630cde599";
pub(crate) const PROFILE: &str = "ahem-10px-fri07-intrinsic-flex-text";
pub(crate) const REQUEST: &[u8] = include_bytes!("fri07_intrinsic_flex_text_request.json");
pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/fri07_intrinsic_flex_text.js";
pub(crate) const SCRIPT: &str = include_str!("fri07_intrinsic_flex_text.js");
pub(crate) const LABELS: [&str; 2] = ["alpha beta gamma", "longword short tail"];

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WordRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) line: usize,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

pub(crate) fn valid_words(label: &str, words: &[WordRange]) -> bool {
    let mut offset = 0;
    let mut previous_line = None;
    words.len() == label.split(' ').count()
        && label.split(' ').zip(words).all(|(word, range)| {
            let start = offset;
            let end = start + word.encode_utf16().count();
            offset = end + 1;
            let previous = previous_line.replace(range.line);
            range.start == start
                && range.end == end
                && previous.map_or(range.line == 0, |line| {
                    range.line == line || range.line.checked_sub(1) == Some(line)
                })
                && range.x.is_finite()
                && range.y.is_finite()
                && range.width.is_finite()
                && range.height.is_finite()
                && range.width > 0.0
                && range.height > 0.0
                && (range.x + range.width).is_finite()
                && (range.y + range.height).is_finite()
        })
}
