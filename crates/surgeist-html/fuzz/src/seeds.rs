//! Independent source literals; no expected output is sampled from parsing.
use crate::{UnicodeSource, error};
use surgeist_fuzz::Result;
const TEXT: [&str; 8] = [
    "<p ID=x id=y>A&amp;B\r\nC</p>",
    "",
    "<p>A\0B</p>",
    "<svg><a xlink:href='x' viewBox='0 0 1 1'></a></svg>",
    "<template><p>T</p></template><p>Z</p>",
    "<?target data?><p>P</p>",
    "<p>A<p>B",
    "<table>X<tr><td>T</td></tr></table>",
];
const SURROGATE: [u16; 13] = [0xfeff, 60, 112, 62, 65, 0xd800, 13, 10, 66, 60, 47, 112, 62];
pub(crate) fn source(id: u8) -> Result<UnicodeSource> {
    match id {
        1 => Ok(UnicodeSource::Utf16(SURROGATE.to_vec())),
        0 | 2..=7 => Ok(UnicodeSource::Utf8(TEXT[usize::from(id)].into())),
        _ => Err(error("unknown literal")),
    }
}
pub(crate) fn matches(id: u8, s: &UnicodeSource) -> bool {
    match s {
        UnicodeSource::Utf8(s) => {
            id != 1
                && TEXT
                    .get(usize::from(id))
                    .is_some_and(|expected| s == expected)
        }
        UnicodeSource::Utf16(s) => id == 1 && s.as_slice() == SURROGATE,
    }
}
/// Validate a closed literal oracle WITHOUT materializing the untrusted source.
pub(crate) fn payload_matches(id: u8, encoding: u8, payload: &[u8]) -> bool {
    if id == 1 {
        encoding == 1
            && payload.len() == 26
            && payload
                .chunks_exact(2)
                .zip(SURROGATE)
                .all(|(p, u)| u16::from_le_bytes([p[0], p[1]]) == u)
    } else {
        encoding == 0
            && TEXT
                .get(usize::from(id))
                .is_some_and(|s| s.as_bytes() == payload)
    }
}
