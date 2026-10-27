//! Bounded lexical input to the original component structural owner.
use crate::component::values::parse::{
    ComponentStack, admit_component, copied, interrupted, odd_trailing_backslashes, range_origin,
};
use crate::component::values::{
    ComponentData, CssComponentValueErrorKind, Lexeme, NumericToken, TokenData, ValueToken,
};
use crate::component::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueLimits,
    CssComponentValues, CssHashFlag, CssNumericTokenKind, CssValueOrigin,
};
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
use crate::tokenization::bounded::Scalar;
use crate::tokenization::tokenizer::{
    NameKind, NumberKind, Record, Request, Span, TokenCursor, TokenKind,
};
use crate::values::color_adjust::utf16_value::{Control, Failure};
use std::mem::size_of;

pub(crate) fn parse_controlled_snapshot(
    source: &CssSourceSnapshot,
    limits: CssComponentValueLimits,
    control: &Control<'_>,
) -> Result<CssComponentValues, CssComponentValueError> {
    let baseline = control.live();
    let result = consume_snapshot(source, limits, control);
    if result.is_err() {
        // Every lexer/component bank has actually dropped on return from the
        // structural owner. The error retains only a handle into the already
        // admitted input source; cleanup cannot observe/refuse interruption.
        control.reconciled(baseline.bytes(), baseline.utf16_units());
    }
    result
}

fn consume_snapshot(
    source: &CssSourceSnapshot,
    limits: CssComponentValueLimits,
    control: &Control<'_>,
) -> Result<CssComponentValues, CssComponentValueError> {
    let at = range_origin(source, 0..0);
    let refused = || interrupted(&at);
    if source.as_str().len() > limits.max_css_bytes {
        return Err(CssComponentValueError::new(
            CssComponentValueErrorKind::ByteLimit,
            CssValueOrigin::Parsed(at),
        ));
    }
    // Source preparation already knows the exact scalar count. Secondary
    // decoded CSS input uses the same fixed lexical cursor; its source owner
    // supplies that count without manufacturing original-input mappings.
    let length = if let Some(length) = source.preprocessed_scalar_count() {
        length
    } else {
        let mut length = 0usize;
        for _ in source.as_str().chars() {
            control.step().map_err(|()| refused())?;
            length = length.checked_add(1).ok_or_else(refused)?;
        }
        length
    };
    let fixed = size_of::<TokenCursor>()
        + size_of::<Vec<Record>>()
        + size_of::<ComponentStack<'_, '_>>()
        + size_of::<std::str::CharIndices<'_>>();
    control.add(fixed).map_err(|()| refused())?;
    let mut records = Vec::new();
    let mut cursor = TokenCursor::new(length, 0);
    let mut scalars = source.as_str().char_indices();
    let mut scalar_index = 0;
    loop {
        control.step().map_err(|()| refused())?;
        match cursor.request() {
            Request::ReadScalar { index } => {
                assert_eq!(
                    index, scalar_index,
                    "original lexical cursor reads in order"
                );
                let (start, value) = scalars.next().expect("prepared scalar count");
                cursor
                    .read_scalar(
                        index,
                        Scalar {
                            value,
                            start,
                            end: start + value.len_utf8(),
                        },
                    )
                    .expect("actual prepared scalar observation");
                scalar_index += 1;
            }
            Request::Compute => cursor.compute().map_err(|_| {
                let _ = control.fail::<()>(Failure::CapacityOverflow);
                refused()
            })?,
            Request::Append { index, record } => {
                assert_eq!(index, records.len());
                control.reserve(&mut records, 1).map_err(|()| refused())?;
                records.push(record);
                cursor.appended(index).expect("actual lexical append");
            }
            Request::Done => break,
        }
    }
    let mut stack = ComponentStack::new(source, limits, 0, Some(control));
    let mut count = 0;
    for record in &records {
        control.step().map_err(|()| refused())?;
        let Record::Token(token) = record else {
            continue;
        };
        let range = token.source.start..token.source.end;
        let origin = range_origin(source, range.clone());
        let closing = match token.kind {
            TokenKind::Delimiter(')') => Some(CssBlockKind::Parenthesis),
            TokenKind::Delimiter(']') => Some(CssBlockKind::SquareBracket),
            TokenKind::Delimiter('}') => Some(CssBlockKind::CurlyBracket),
            _ => None,
        };
        if stack.close(closing, range.clone())? {
            continue;
        }
        let kind = match token.kind {
            TokenKind::Name {
                kind: NameKind::Function,
                ..
            }
            | TokenKind::Delimiter('(') => Some(CssBlockKind::Parenthesis),
            TokenKind::Delimiter('[') => Some(CssBlockKind::SquareBracket),
            TokenKind::Delimiter('{') => Some(CssBlockKind::CurlyBracket),
            _ => None,
        };
        if let Some(kind) = kind {
            let name = match token.kind {
                TokenKind::Name { units, .. } => Some(decoded(&records, units, control, &origin)?),
                _ => None,
            };
            stack.open(kind, name, range, &mut count)?;
            continue;
        }
        admit_component(&mut count, limits, &origin)?;
        let invalid = match token.kind {
            TokenKind::BadString => Some(CssComponentValueErrorKind::BadString),
            TokenKind::BadUrl { .. } => Some(CssComponentValueErrorKind::BadUrl),
            TokenKind::Delimiter(')' | ']' | '}') => {
                Some(CssComponentValueErrorKind::UnmatchedClosingDelimiter)
            }
            _ => None,
        };
        if let Some(kind) = invalid {
            return Err(CssComponentValueError::new(
                kind,
                CssValueOrigin::Parsed(origin),
            ));
        }
        let raw = &source.as_str()[range.clone()];
        let spelling = Lexeme {
            text: copied(raw, Some(control), &origin)?,
            origin: CssValueOrigin::Parsed(origin.clone()),
        };
        // The same EOF terminations as direct collection. An escaped EOF adds
        // no code point inside a string (Syntax 3 §4.3.5), so its closure keeps
        // that effect with an escaped newline; elsewhere it decodes to U+FFFD
        // (§4.3.7), which the closure spells as an explicit escape.
        let escaped_eof = odd_trailing_backslashes(raw);
        let implicit_text = match token.kind {
            TokenKind::String { closed: false, .. } => {
                Some(match (raw.starts_with('\''), escaped_eof) {
                    (true, false) => "'",
                    (true, true) => "\n'",
                    (false, false) => "\"",
                    (false, true) => "\n\"",
                })
            }
            TokenKind::Url { closed: false, .. } => Some(if escaped_eof { "fffd )" } else { ")" }),
            TokenKind::Comment { closed: false } => Some("*/"),
            TokenKind::Name {
                kind: NameKind::Ident | NameKind::At | NameKind::Hash { .. },
                ..
            }
            | TokenKind::Number {
                kind: NumberKind::Dimension,
                ..
            } if escaped_eof => Some("fffd "),
            _ => None,
        };
        let implicit_end = implicit_text
            .map(|text| -> Result<Lexeme, CssComponentValueError> {
                Ok(Lexeme {
                    text: copied(text, Some(control), &origin)?,
                    origin: CssValueOrigin::ImplicitClosure {
                        opening: origin.clone(),
                        at: range_origin(source, range.end..range.end),
                    },
                })
            })
            .transpose()?;
        let data = if let TokenKind::Comment { closed } = token.kind {
            let end = raw.len() - if closed { 2 } else { 0 };
            ComponentData::Comment {
                content: copied(&raw[2..end], Some(control), &origin)?,
                spelling,
                implicit_end,
            }
        } else {
            ComponentData::Token(ValueToken {
                data: token_data_bounded(token.kind, raw, &records, control, &origin)?,
                spelling,
                implicit_end,
            })
        };
        stack.push(CssComponentValue {
            data,
            parsed: Some(origin),
        })?;
    }
    let items = stack.finish(source.as_str().len())?;
    let record_bytes = records.capacity() * size_of::<Record>();
    drop(records);
    // The cursor and iterator are fixed, non-owning registers. Their admitted
    // extent ends together with the lexical bank before result publication.
    control.released(record_bytes + fixed);
    CssComponentValues::from_items_controlled(items, limits, Some(control))
}

fn decoded(
    records: &[Record],
    span: Span,
    control: &Control<'_>,
    origin: &CssParsedOrigin,
) -> Result<Box<str>, CssComponentValueError> {
    let mut length = 0usize;
    for record in &records[span.start..span.end] {
        control.step().map_err(|()| interrupted(origin))?;
        let Record::NameUnit(point) = record else {
            unreachable!("decoded name bank")
        };
        length = length
            .checked_add(point.len_utf8())
            .ok_or_else(|| interrupted(origin))?;
    }
    control.add(length).map_err(|()| interrupted(origin))?;
    let mut text = String::new();
    if text.try_reserve_exact(length).is_err() {
        let _ = control.fail::<()>(Failure::Allocation);
        return Err(interrupted(origin));
    }
    if text.capacity() > length {
        control
            .add(text.capacity() - length)
            .map_err(|()| interrupted(origin))?;
    }
    for record in &records[span.start..span.end] {
        control.step().map_err(|()| interrupted(origin))?;
        let Record::NameUnit(point) = record else {
            unreachable!("decoded name bank")
        };
        text.push(*point);
    }
    let capacity = text.capacity();
    if capacity != length {
        control.add(length).map_err(|()| interrupted(origin))?;
    }
    let result = text.into_boxed_str();
    if capacity != length {
        control.released(capacity);
    }
    Ok(result)
}

fn token_data_bounded(
    kind: TokenKind,
    raw: &str,
    records: &[Record],
    control: &Control<'_>,
    origin: &CssParsedOrigin,
) -> Result<TokenData, CssComponentValueError> {
    let name = |span| decoded(records, span, control, origin);
    Ok(match kind {
        TokenKind::Name {
            kind: NameKind::Ident,
            units,
        } => TokenData::Ident(name(units)?),
        TokenKind::Name {
            kind: NameKind::At,
            units,
        } => TokenData::AtKeyword(name(units)?),
        TokenKind::Name {
            kind: NameKind::Hash { identifier },
            units,
        } => TokenData::Hash {
            value: name(units)?,
            flag: if identifier {
                CssHashFlag::Id
            } else {
                CssHashFlag::Unrestricted
            },
        },
        TokenKind::String { units, .. } => TokenData::String(name(units)?),
        TokenKind::Url { units, .. } => TokenData::Url(name(units)?),
        TokenKind::Whitespace => TokenData::Whitespace,
        TokenKind::Cdo => TokenData::Cdo,
        TokenKind::Cdc => TokenData::Cdc,
        TokenKind::Delimiter(':') => TokenData::Colon,
        TokenKind::Delimiter(';') => TokenData::Semicolon,
        TokenKind::Delimiter(',') => TokenData::Comma,
        TokenKind::Delimiter(point) => TokenData::Delim(point),
        TokenKind::Number {
            kind, layout, unit, ..
        } => {
            let sign = usize::from(raw.starts_with(['+', '-']));
            let mut end = sign + layout.mantissa_digits + usize::from(layout.fraction_digits != 0);
            if layout.exponent_digits != 0 {
                end += 1;
                end += usize::from(matches!(raw.as_bytes().get(end), Some(b'+' | b'-')));
                end += layout.exponent_digits;
            }
            let number = NumericToken {
                representation: copied(&raw[..end], Some(control), origin)?,
                kind: if layout.fraction_digits == 0 && layout.exponent_digits == 0 {
                    CssNumericTokenKind::Integer
                } else {
                    CssNumericTokenKind::Number
                },
                has_sign: sign != 0,
            };
            match kind {
                NumberKind::Number => TokenData::Number(number),
                NumberKind::Percentage => TokenData::Percentage(number),
                NumberKind::Dimension => TokenData::Dimension {
                    number,
                    unit: name(unit)?,
                },
            }
        }
        TokenKind::Name {
            kind: NameKind::Function,
            ..
        }
        | TokenKind::Comment { .. }
        | TokenKind::BadString
        | TokenKind::BadUrl { .. } => unreachable!("structural/error token handled"),
    })
}
