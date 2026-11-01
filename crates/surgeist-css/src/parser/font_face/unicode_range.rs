//! Unicode-range descriptor values from CSS Syntax 3 (2026-10-01) §5.5.11.
//!
//! The value is re-tokenized from its original text with unicode ranges
//! allowed, so each range is one contiguous `<unicode-range-token>` (§4.3.14)
//! starting with a literal `U` or `u` (§4.3.1). The grammar of the descriptor
//! (Fonts 4 `<urange>#`) is then a comma-separated list whose every member is
//! one such token; whitespace or a comment ends a token (§4.3.1 consumes
//! comments before every token). The tokenizer does not judge the range: a
//! bound above U+10FFFF, or an end before the start, is the descriptor's
//! business ([`CssUnicodeRange::try_new`]).
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-unicode-range-value>
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-unicode-range-token>

use cssparser::SourceLocation;
use surgeist_css_parser::Token;

use crate::component::{CssSerializedOrigin, CssSerializedValue};
use crate::diagnostic::error::{
    incomplete_descriptor_at, invalid_component_value, invalid_descriptor_authored_syntax_token_at,
    unexpected_at,
};
use crate::numeric::expression::NumericInputContext;
use crate::parser::block_units::{location_at, offset_of};
use crate::parser::combinators::parse_native_comma_list;
use crate::parser::descriptor_stream::{
    NativeValueError, Value, ValueError, at_serialized_offset, consume_remaining, fail,
    serialized_offset,
};
use crate::parser::syntax_types::{Site, Stream, SyntaxError};
use crate::rule::font_face::CssUnicodeRange;
use crate::source::CssSourceSnapshot;

/// Where a boundary of the re-tokenized text is, in the terms of the
/// diagnostics.
enum Locator<'a> {
    /// The text is a window of the source: boundaries are bytes of it.
    Source(&'a CssSourceSnapshot),
    /// The text is the serialization of checked components from byte `base` on,
    /// without the comments the serialization inserted between tokens; each
    /// entry of `inserted` is where such a comment was in the text and how long.
    Serialized {
        serialized: &'a CssSerializedValue,
        base: usize,
        inserted: Vec<(usize, usize)>,
    },
}

impl Locator<'_> {
    fn at(&self, site: Site<'_>) -> SourceLocation {
        let offset = site.offset().map_or(0, surgeist_css_parser::Offset::value);
        match self {
            Self::Source(snapshot) => location_at(snapshot, offset),
            Self::Serialized {
                serialized,
                base,
                inserted,
            } => {
                let removed: usize = inserted
                    .iter()
                    .take_while(|(at, _)| *at <= offset)
                    .map(|(_, length)| length)
                    .sum();
                at_serialized_offset(serialized, base + offset + removed)
            }
        }
    }
}

/// The ranges of the value of a `unicode-range` descriptor that `input` holds,
/// to its end. The text of the value is tokenized again with unicode ranges
/// allowed, and `input` is left at its end.
pub(in crate::parser) fn parse_list<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, Vec<CssUnicodeRange>> {
    let at = input.site();
    let end = input.end_site();
    let ranges = match numeric.ordinary() {
        NumericInputContext::Parsed(snapshot) => {
            let range = offset_of(at)..offset_of(end);
            let document = crate::syntax_consumption::unicode_range_document(snapshot, range)
                .map_err(|error| fail(numeric, at, |at| invalid_component_value(at, error)))?;
            read(&mut document.root(), &Locator::Source(snapshot))
        }
        NumericInputContext::Components(values, serialized) => {
            let base = serialized_offset(values, serialized, at);
            let finish = serialized_offset(values, serialized, end).max(base);
            let (text, inserted) = authored_text(serialized, base..finish);
            let document = crate::syntax_consumption::unicode_range_text_document(&text)
                .map_err(|error| fail(numeric, at, |at| invalid_component_value(at, error)))?;
            read(
                &mut document.root(),
                &Locator::Serialized {
                    serialized,
                    base,
                    inserted,
                },
            )
        }
        NumericInputContext::Checked | NumericInputContext::QuirkyLengths(_) => {
            return Err(fail(numeric, at, unexpected_at));
        }
    }
    .map_err(|error| SyntaxError::custom(at, error))?;
    consume_remaining(input);
    Ok(ranges)
}

// Checked components have no source text. Their serialization inserts empty
// comments only where the table of section 9 requires one; such a separator is
// not authored, so the components' own adjacent spellings remain one range text.
fn authored_text(
    serialized: &CssSerializedValue,
    window: std::ops::Range<usize>,
) -> (String, Vec<(usize, usize)>) {
    let mut text = String::new();
    let mut inserted = Vec::new();
    for segment in serialized.segments() {
        let range = segment.byte_range();
        let (start, end) = (range.start.max(window.start), range.end.min(window.end));
        if start >= end {
            continue;
        }
        if matches!(segment.origin(), CssSerializedOrigin::Separator { .. }) {
            inserted.push((text.len(), end - start));
        } else if let Some(part) = serialized.as_css().get(start..end) {
            text.push_str(part);
        }
    }
    (text, inserted)
}

fn read<'w>(
    stream: &mut Stream<'w>,
    locator: &Locator<'_>,
) -> Result<Vec<CssUnicodeRange>, NativeValueError> {
    parse_native_comma_list(
        stream,
        |item| range(item, locator),
        |_, at| {
            SyntaxError::custom(
                at,
                incomplete_descriptor_at(locator.at(at), "font-face", "unicode-range"),
            )
        },
    )
    .map_err(|error| {
        error
            .custom_value()
            .cloned()
            .unwrap_or_else(|| unexpected_at(locator.at(error.at())))
    })
}

/// One member of the list: exactly one range token.
fn range<'w>(
    item: &mut Stream<'w>,
    locator: &Locator<'_>,
) -> Result<CssUnicodeRange, ValueError<'w>> {
    item.discard_whitespace();
    let at = item.site();
    let invalid = |site: Site<'w>, token: Token<'w>| invalid_at(locator, site, token);
    let Ok(token) = item.consume_token() else {
        return Err(SyntaxError::custom(
            at,
            incomplete_descriptor_at(locator.at(item.end_site()), "font-face", "unicode-range"),
        ));
    };
    let Token::UnicodeRange(code_points) = token else {
        return Err(cannot_continue(item, at, token, locator));
    };
    item.discard_whitespace();
    let extra = item.site();
    if let Some(next) = item.next_token() {
        return Err(invalid(extra, next));
    }
    CssUnicodeRange::try_new(code_points.start(), code_points.end())
        .ok_or_else(|| invalid(at, token))
}

/// The failure of a member at the token `token` at `site`.
fn invalid_at<'w>(locator: &Locator<'_>, site: Site<'w>, token: Token<'w>) -> ValueError<'w> {
    SyntaxError::custom(
        site,
        invalid_descriptor_authored_syntax_token_at(
            locator.at(site),
            "font-face",
            "unicode-range",
            token,
            site.spelling().unwrap_or_default(),
        ),
    )
}

/// The failure of a member that begins with `token` at `at`, which is not a
/// range token. A literal `u` followed by a plus sign begins a range that
/// needs a hexadecimal digit or a question mark next (section 4.3.14): the
/// token that does not continue it is responsible, and a member that ends
/// inside it is incomplete. Any other first token is responsible itself.
fn cannot_continue<'w>(
    item: &mut Stream<'w>,
    at: Site<'w>,
    token: Token<'w>,
    locator: &Locator<'_>,
) -> ValueError<'w> {
    let incomplete = |item: &Stream<'w>| {
        SyntaxError::custom(
            at,
            incomplete_descriptor_at(locator.at(item.end_site()), "font-face", "unicode-range"),
        )
    };
    let begins_range = matches!(token, Token::Ident(_))
        && at
            .spelling()
            .is_some_and(|spelling| spelling.eq_ignore_ascii_case("u"));
    if !begins_range {
        return invalid_at(locator, at, token);
    }
    item.discard_whitespace();
    let next_at = item.site();
    let Ok(next) = item.consume_token() else {
        return incomplete(item);
    };
    if !matches!(next, Token::Delim('+')) {
        return invalid_at(locator, next_at, next);
    }
    item.discard_whitespace();
    let after_at = item.site();
    match item.consume_token() {
        Ok(after) => invalid_at(locator, after_at, after),
        Err(_) => incomplete(item),
    }
}
