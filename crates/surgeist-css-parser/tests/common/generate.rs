//! The case generator shared by the property tests and the cssparser oracle:
//! pieces of authored CSS joined without separators, so every construct meets
//! every other, balanced and not, including the escaped block punctuation of
//! `\{`, `\}`, `\(`, `\)`, `\[` and `\]` in every context.
//!
//! The pieces are chosen from CSS Syntax 3 sections 4.3.1 to 4.3.15: each token
//! class, each way to end a string, url or comment, and each way a backslash can
//! appear. The generator is deterministic in its entropy.

#![allow(dead_code)]

/// The pieces. Escapes are weighted separately in [`generate`].
pub const PIECES: &[&str] = &[
    "a",
    "b-c",
    "_x",
    "--y",
    "-z",
    "\u{e9}",
    "\u{2603}",
    "\u{1f600}",
    "\u{a0}",
    "\u{fffd}",
    " ",
    "  ",
    "\t",
    "\n",
    "\r",
    "\r\n",
    "\u{c}",
    "\0",
    "{",
    "}",
    "(",
    ")",
    "[",
    "]",
    ";",
    ":",
    ",",
    "!",
    "%",
    "&",
    "*",
    "/",
    "=",
    ">",
    "?",
    "^",
    "|",
    "~",
    "$",
    "<",
    "@",
    "#",
    ".",
    "+",
    "-",
    "1",
    "23",
    "4.5",
    ".6",
    "-7",
    "+8",
    "9e1",
    "1E-2",
    "3e+",
    "1e",
    "0.0.2",
    "px",
    "%",
    "em",
    "\"",
    "'",
    "\"s\"",
    "'t'",
    "\"u\n",
    "'v\\\n'",
    "/*",
    "*/",
    "/**/",
    "/* c */",
    "<!--",
    "-->",
    "#id",
    "#1",
    "@media",
    "@-x",
    "f(",
    "calc(",
    "url(",
    "url(a)",
    "url( b )",
    "url(\"c\")",
    "url(d e)",
    "URL(",
    "url(f\"",
    "\\41 ",
    "\\0",
    "\\d800 ",
    "\\110000 ",
    "\\\\",
    "\\",
    "\\\n",
    "~=",
    "|=",
    "^=",
    "$=",
    "*=",
    "U+26",
    "u+4??",
    "U+0-7F",
    "#-\\\n",
    "-\\\n",
    "url(a \\)b)",
];

/// The six escapes whose escaped punctuation is an ident code point, not a
/// delimiter (4.3.7).
pub const ESCAPES: [&str; 6] = ["\\{", "\\}", "\\(", "\\)", "\\[", "\\]"];

/// The most pieces in one case.
pub const MAX_PIECES: usize = 24;

/// The entropy bytes a case consumes.
pub const ENTROPY_BYTES: usize = 2 * MAX_PIECES + 1;

/// Builds a case from `entropy`. About two in five pieces are escaped
/// punctuation; the rest come from [`PIECES`].
pub fn generate(entropy: &[u8]) -> String {
    let byte = |index: usize| usize::from(entropy.get(index).copied().unwrap_or(0));
    let count = 1 + byte(0) % MAX_PIECES;
    let mut text = String::new();
    for piece in 0..count {
        let pick = byte(1 + 2 * piece);
        let index = byte(2 + 2 * piece);
        if pick % 5 < 2 {
            text.push_str(ESCAPES[index % ESCAPES.len()]);
        } else {
            text.push_str(PIECES[index % PIECES.len()]);
        }
    }
    text
}

/// A deterministic byte source (SplitMix64), so the cases do not depend on any
/// framework's random number generator.
pub struct Entropy(u64);

impl Entropy {
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn bytes(&mut self, count: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(count + 8);
        while out.len() < count {
            out.extend_from_slice(&self.next_u64().to_le_bytes());
        }
        out.truncate(count);
        out
    }
}
