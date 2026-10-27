//! Source-preserving lexical corrections for the pinned cssparser provider.
//!
//! CSS Syntax 3 §4.3.6 leaves the first non-whitespace character pending for
//! §4.3.15. cssparser 0.37 consumes that character first. Only when that character
//! starts a backslash run immediately before ')' does this change the boundary.
//! The grammar receives a same-length mask inside an already-bad URL; authored
//! spellings and positions always come from the unmodified source snapshot.

use std::borrow::Cow;
use std::ops::Range;

use cssparser::{Parser, ParserInput, Token};
use surgeist_css_parser::lexer::protocol::{NumberCursor, NumberStep};
use surgeist_css_parser::lexical;

// Test-only work counter: the input bytes walked by `prepare` on this thread.
#[cfg(test)]
thread_local! {
    pub(crate) static PREPARED_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Prepare one complete working input, preserving any masks already applied by
/// structural recovery. Clean inputs remain borrowed. Each token is visited at
/// its actual boundary, including URLs whose provider cursor overruns that end.
/// Preparation is linear in input bytes: a provider overrun stops at the next
/// unescaped ')' (or EOF), so overruns from distinct corrected URLs do not
/// overlap. Prefix/remnants walks inspect only their own URL, and masks use one
/// lazy byte-buffer allocation plus one final UTF-8 validation.
pub(crate) fn prepare(source: &str) -> Cow<'_, str> {
    #[cfg(test)]
    PREPARED_BYTES.with(|prepared| prepared.set(prepared.get() + source.len()));
    let mut masked: Option<Vec<u8>> = None;
    let mut offset = 0;
    while let Some((end, _, correction)) = source_token(source, offset) {
        if let Some(byte) = correction {
            masked.get_or_insert_with(|| source.as_bytes().to_vec())[byte] = b'_';
        }
        offset = end;
    }
    match masked {
        Some(bytes) => Cow::Owned(String::from_utf8(bytes).expect("only ASCII bytes are masked")),
        None => Cow::Borrowed(source),
    }
}

/// Read one original-source token without automatic block skipping. Raw range
/// walkers need the true boundary but must not prepare every remaining suffix.
/// BadUrl keeps its provider payload semantics: original inner contents, without
/// leading URL whitespace or an unescaped closing ')'.
pub(crate) fn next_source_token(source: &str, offset: usize) -> Option<(usize, usize, Token<'_>)> {
    let (end, token, _) = source_token(source, offset)?;
    let (token, range) = source_token_parts(source, token, offset..end)
        .into_iter()
        .flatten()
        .next()
        .expect("a provider capture has a first canonical token");
    Some((range.start, range.end, token))
}

/// The selected Syntax token stream has no combined attribute-match tokens.
/// Native selector grammar still uses the provider's semantic forms directly.
pub(crate) const fn combined_operator_prefix(token: &Token<'_>) -> Option<char> {
    match token {
        Token::IncludeMatch => Some('~'),
        Token::DashMatch => Some('|'),
        Token::PrefixMatch => Some('^'),
        Token::SuffixMatch => Some('$'),
        Token::SubstringMatch => Some('*'),
        _ => None,
    }
}

/// Partition an actual source capture without changing decoded provider
/// payloads. Each part owns its real consumed range in the original input.
pub(crate) fn source_token_parts<'a>(
    source: &str,
    token: Token<'a>,
    range: Range<usize>,
) -> [Option<(Token<'a>, Range<usize>)>; 2] {
    if let Some(first) = combined_operator_prefix(&token) {
        return [
            Some((Token::Delim(first), range.start..range.start + 1)),
            Some((Token::Delim('='), range.start + 1..range.end)),
        ];
    }
    let text = &source[range.clone()];
    let numeric_end = if matches!(token, Token::Dimension { .. }) {
        numeric_prefix_length(text)
    } else {
        0
    };
    match token {
        Token::Ident(_) if text == "-" && !would_start_identifier(text) => {
            [Some((Token::Delim('-'), range.clone())), None]
        }
        Token::AtKeyword(_) if text == "@-" && !would_start_identifier(&text[1..]) => [
            Some((Token::Delim('@'), range.start..range.start + 1)),
            Some((Token::Delim('-'), range.start + 1..range.end)),
        ],
        Token::Dimension {
            has_sign,
            value,
            int_value,
            ..
        } if &text[numeric_end..] == "-" && !would_start_identifier(&text[numeric_end..]) => {
            let split = range.start + numeric_end;
            [
                Some((
                    Token::Number {
                        has_sign,
                        value,
                        int_value,
                    },
                    range.start..split,
                )),
                Some((Token::Delim('-'), split..range.end)),
            ]
        }
        Token::Hash(value) | Token::IDHash(value) => {
            let token = if would_start_identifier(&text[1..]) {
                Token::IDHash(value)
            } else {
                Token::Hash(value)
            };
            [Some((token, range.clone())), None]
        }
        token => [Some((token, range.clone())), None],
    }
}

/// CSS Syntax 3 §4.3.9 over authored code points: the pre-pass sees text that
/// input preprocessing (§3.3) has not yet normalized, so each code point is
/// preprocessed here before [`lexical::would_start_ident`], which describes
/// preprocessed input, decides. EOF is a valid escape second code point.
fn would_start_identifier(text: &str) -> bool {
    let mut points = text.chars().map(preprocessed);
    let first = points.next();
    let second = points.next();
    let third = points.next();
    lexical::would_start_ident(first, second, third)
}

/// CSS Syntax 3 §3.3 applied to one authored code point: U+0000 becomes
/// U+FFFD, and U+000D and U+000C become U+000A.
const fn preprocessed(point: char) -> char {
    match point {
        '\0' => '\u{fffd}',
        '\r' | '\u{c}' => '\n',
        point => point,
    }
}

// CSS Syntax's consume-a-number spelling boundary. The dependency supplies the
// token category; this scan preserves the exact prefix without float conversion.
pub(crate) fn numeric_prefix_length(representation: &str) -> usize {
    let bytes = representation.as_bytes();
    let mut cursor = NumberCursor::new();
    let mut end = 0;
    loop {
        let window = [bytes.get(end), bytes.get(end + 1), bytes.get(end + 2)]
            .map(|byte| byte.copied().map(char::from));
        match cursor
            .step(window)
            .expect("a borrowed token's counters fit its length")
        {
            NumberStep::Consume(_) => end += 1,
            NumberStep::Transition => {}
            NumberStep::Done(_) => return end,
        }
    }
}

fn source_token(source: &str, offset: usize) -> Option<(usize, Token<'_>, Option<usize>)> {
    let remaining = source.get(offset..)?;
    if remaining.is_empty() {
        return None;
    }
    let mut storage = ParserInput::new(remaining);
    let mut parser = Parser::new(&mut storage);
    let mut token = parser
        .next_including_whitespace_and_comments()
        .ok()?
        .clone();
    let mut end = offset + parser.position().byte_index();
    let correction = if matches!(token, Token::BadUrl(_)) {
        if let Some(correction) = bad_url_correction(remaining) {
            end = offset + correction.end;
            token = Token::BadUrl(remaining[correction.contents].into());
            Some(offset + correction.mask)
        } else {
            None
        }
    } else {
        None
    };
    Some((end, token, correction))
}

/// Inspect only a provider-confirmed bad URL, not general CSS tokenization. Its
/// decoded name is already 'url', so the first literal '(' ends its name even
/// when that name contains hexadecimal escapes and their whitespace terminator.
struct BadUrlCorrection {
    end: usize,
    mask: usize,
    contents: std::ops::Range<usize>,
}

fn bad_url_correction(source: &str) -> Option<BadUrlCorrection> {
    let bytes = source.as_bytes();
    let mut offset = source.find('(')? + 1;
    while bytes.get(offset).is_some_and(|byte| whitespace(*byte)) {
        offset += 1;
    }
    let contents_start = offset;
    loop {
        let byte = *bytes.get(offset)?;
        match byte {
            b')'
            | b'"'
            | b'\''
            | b'('
            | b'\x01'..=b'\x08'
            | b'\x0b'
            | b'\x0e'..=b'\x1f'
            | b'\x7f' => return None,
            b'\\' => {
                if bytes.get(offset + 1).is_some_and(|byte| newline(*byte)) {
                    return None;
                }
                offset = escape_end(source, offset);
            }
            byte if whitespace(byte) => {
                while bytes.get(offset).is_some_and(|byte| whitespace(*byte)) {
                    offset += 1;
                }
                let first = offset;
                while bytes.get(offset) == Some(&b'\\') {
                    offset += 1;
                }
                let count = offset - first;
                if count == 0 || bytes.get(offset) != Some(&b')') {
                    return None;
                }
                // An odd run escapes the closer: neutralize that closer. An
                // even run exposes it: neutralize the second slash so losing
                // the first slash no longer leaves an odd escape run.
                let mask = if count % 2 == 1 { offset } else { first + 1 };
                let closing = remnants_closing(bytes, first);
                return Some(BadUrlCorrection {
                    end: closing.map_or(bytes.len(), |closing| closing + 1),
                    mask,
                    contents: contents_start..closing.unwrap_or(bytes.len()),
                });
            }
            _ => offset += 1,
        }
    }
}

fn escape_end(source: &str, slash: usize) -> usize {
    let bytes = source.as_bytes();
    let mut offset = slash + 1;
    if bytes.get(offset).is_some_and(u8::is_ascii_hexdigit) {
        let start = offset;
        while offset - start < 6 && bytes.get(offset).is_some_and(u8::is_ascii_hexdigit) {
            offset += 1;
        }
        if bytes.get(offset).is_some_and(|byte| whitespace(*byte)) {
            if bytes.get(offset) == Some(&b'\r') && bytes.get(offset + 1) == Some(&b'\n') {
                offset += 1;
            }
            offset += 1;
        }
    } else if let Some(character) = source[offset..].chars().next() {
        offset += character.len_utf8();
    }
    offset
}

fn remnants_closing(bytes: &[u8], mut offset: usize) -> Option<usize> {
    while offset < bytes.len() {
        match bytes[offset] {
            b')' => return Some(offset),
            b'\\' if matches!(bytes.get(offset + 1), Some(b'\\' | b')')) => offset += 2,
            _ => offset += 1,
        }
    }
    None
}

/// Whether an authored byte is whitespace once preprocessed (§3.3, §4.2).
const fn whitespace(byte: u8) -> bool {
    lexical::is_whitespace(preprocessed(byte as char))
}

/// Whether an authored byte is a newline once preprocessed (§3.3, §4.2).
const fn newline(byte: u8) -> bool {
    lexical::is_newline(preprocessed(byte as char))
}

#[cfg(test)]
mod tests {
    use super::{newline, whitespace};

    // Authored text has not been preprocessed. Syntax 3 §3.3 turns U+000D,
    // U+000C and the pair U+000D U+000A into U+000A, and §4.2 defines a newline
    // as U+000A and whitespace as a newline, U+0009 or U+0020. A byte of
    // authored text is therefore a newline when it is LF, CR or FF, and
    // whitespace when it is also TAB or SPACE; no other byte, including every
    // byte of a multi-byte sequence and NUL, is either.
    #[test]
    fn authored_bytes_are_newlines_and_whitespace_as_preprocessing_and_the_definitions_say() {
        for byte in 0..=u8::MAX {
            let expected_newline = matches!(byte, 0x0A | 0x0D | 0x0C);
            let expected_whitespace = expected_newline || matches!(byte, 0x09 | 0x20);
            assert_eq!(newline(byte), expected_newline, "newline {byte:#04x}");
            assert_eq!(
                whitespace(byte),
                expected_whitespace,
                "whitespace {byte:#04x}"
            );
        }
    }
}
