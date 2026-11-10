//! Property P6 of the parser design (#1508, #1487): over the alphabet of block
//! punctuation, escapes, strings, comments and URLs, every sheet and every list
//! of block contents is parsed to the end without a panic, and a `{}` block
//! comes only from a `{` the input spells as a token: Syntax 3 (2026 CRD)
//! §4.3.1 consumes a `{` as the start of a block only outside a string, a
//! comment, a URL and an escape (§4.3.5 to §4.3.7, §4.3.2).
//!
//! The alphabet has no `@`, so a rule can only be a qualified rule, and a
//! qualified rule needs a `{}` block (§5.5.3): a source whose own reading finds
//! no `{` token retains no rule. The reading here is written from the
//! specification text, not taken from the parser under test.

use surgeist_css::CssParser;

/// The alphabet: block punctuation, the escape, the delimiters of declarations,
/// the openers of strings, comments and URLs with the text that closes them,
/// and the line break an escape cannot cross.
const ALPHABET: &[&str] = &[
    "{", "}", "(", ")", "[", "]", "\\", ";", ":", "\"", "'", "url(", "/*", "*/", "\n", "a",
];

/// The `{` tokens of `source` (§4.3.1), found without recursion into blocks:
/// a `{` that no escape, string, comment or URL swallows.
fn curly_tokens(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0;
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'/' if bytes.get(at + 1) == Some(&b'*') => {
                // A comment runs to its `*/`, or to the end of the input.
                at = source[at + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |end| at + 2 + end + 2);
            }
            quote @ (b'"' | b'\'') => {
                // A string ends at its quote, at a newline (a bad string, which
                // does not consume the newline), or at the end of the input; a
                // backslash escapes the next code point, and a backslash
                // before a newline continues the string.
                at += 1;
                while at < bytes.len() {
                    match bytes[at] {
                        b'\\' => at += 2,
                        byte if byte == quote => {
                            at += 1;
                            break;
                        }
                        b'\n' => break,
                        _ => at += 1,
                    }
                }
            }
            b'\\' => {
                // A valid escape takes the next code point; a backslash
                // before a newline is a delimiter.
                if bytes.get(at + 1).is_some_and(|next| *next != b'\n') {
                    at += 2;
                } else {
                    at += 1;
                }
            }
            b'u' | b'U'
                if bytes[at..].len() >= 4
                    && source[at..at + 4].eq_ignore_ascii_case("url(")
                    && (at == 0 || !bytes[at - 1].is_ascii_alphanumeric()) =>
            {
                at += 4;
                // Whitespace, then either a quote (a function token: the
                // string is read as above) or an unquoted URL, which runs to
                // its `)` and takes an escape for the next code point.
                let rest = &source[at..];
                let trimmed = rest.trim_start_matches([' ', '\n']);
                if trimmed.starts_with(['"', '\'']) {
                    continue;
                }
                at += rest.len() - trimmed.len();
                while at < bytes.len() {
                    match bytes[at] {
                        b'\\' => at += 2,
                        b')' => {
                            at += 1;
                            break;
                        }
                        _ => at += 1,
                    }
                }
            }
            b'{' => {
                count += 1;
                at += 1;
            }
            _ => at += 1,
        }
    }
    count
}

/// Every source of up to `length` pieces of the alphabet, in order.
fn exhaustive(length: usize) -> impl Iterator<Item = String> {
    (1..=length).flat_map(|pieces| {
        let total = ALPHABET.len().pow(u32::try_from(pieces).unwrap_or(0));
        (0..total).map(move |mut index| {
            let mut source = String::new();
            for _ in 0..pieces {
                source.push_str(ALPHABET[index % ALPHABET.len()]);
                index /= ALPHABET.len();
            }
            source
        })
    })
}

/// A deterministic walk over longer sources.
fn random(count: usize, pieces: usize) -> impl Iterator<Item = String> {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    (0..count).map(move |_| {
        let mut source = String::new();
        for _ in 0..pieces {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let index = usize::try_from(state % ALPHABET.len() as u64).unwrap_or(0);
            source.push_str(ALPHABET[index]);
        }
        source
    })
}

fn check(source: &str) {
    let parser = CssParser::default();
    let sheet = parser.parse_sheet(source);
    let contents = parser.parse_block_contents(source);
    // Each report ends: it holds a diagnostic for no more units than the
    // source has bytes.
    assert!(sheet.diagnostics().len() <= source.len() + 1, "{source:?}");
    assert!(
        contents.diagnostics().len() <= source.len() + 1,
        "{source:?}"
    );
    if curly_tokens(source) == 0 {
        assert!(
            sheet.syntax().rules().is_empty(),
            "a rule without a `{{` token in {source:?}: {:?}",
            sheet.syntax()
        );
    }
}

#[test]
fn every_short_source_of_the_block_punctuation_alphabet_is_parsed_and_opens_blocks_only_at_a_curly_token()
 {
    for source in exhaustive(4) {
        check(&source);
    }
}

#[test]
fn long_sources_of_the_block_punctuation_alphabet_are_parsed_and_open_blocks_only_at_a_curly_token()
{
    for source in random(4000, 24) {
        check(&source);
    }
}
