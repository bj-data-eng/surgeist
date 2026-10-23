//! Actual component backing and same-input equivalence for the retained owner.
//!
//! Component construction bounds nesting to 256. These traversals use that
//! existing bound and allocate no traversal bank. Shared source payloads and the
//! root inline `CssComponentValues` belong to their separate retaining owners.

use super::*;
use crate::CssRetainedPayloadError;
use std::mem::size_of;

fn observe<E>(admit: &mut impl FnMut() -> Result<(), E>) -> Result<(), CssRetainedPayloadError<E>> {
    admit().map_err(CssRetainedPayloadError::Admission)
}

fn add<E>(total: &mut usize, bytes: usize) -> Result<(), CssRetainedPayloadError<E>> {
    *total = total
        .checked_add(bytes)
        .ok_or(CssRetainedPayloadError::CapacityOverflow)?;
    Ok(())
}

fn text_bytes<E>(
    text: &str,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<usize, CssRetainedPayloadError<E>> {
    observe(admit)?;
    Ok(text.len())
}

fn lexeme_bytes<E>(
    lexeme: &Lexeme,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<usize, CssRetainedPayloadError<E>> {
    observe(admit)?;
    text_bytes(&lexeme.text, admit)
}

fn optional_lexeme_bytes<E>(
    lexeme: &Option<Lexeme>,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<usize, CssRetainedPayloadError<E>> {
    observe(admit)?;
    match lexeme {
        Some(value) => lexeme_bytes(value, admit),
        None => Ok(0),
    }
}

fn token_bytes<E>(
    token: &ValueToken,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<usize, CssRetainedPayloadError<E>> {
    observe(admit)?;
    let mut total = lexeme_bytes(&token.spelling, admit)?;
    add(
        &mut total,
        optional_lexeme_bytes(&token.implicit_end, admit)?,
    )?;
    observe(admit)?;
    match &token.data {
        TokenData::Ident(value)
        | TokenData::AtKeyword(value)
        | TokenData::Hash { value, .. }
        | TokenData::String(value)
        | TokenData::Url(value) => add(&mut total, text_bytes(value, admit)?)?,
        TokenData::Number(number) | TokenData::Percentage(number) => {
            add(&mut total, text_bytes(&number.representation, admit)?)?;
        }
        TokenData::Dimension { number, unit } => {
            add(&mut total, text_bytes(&number.representation, admit)?)?;
            add(&mut total, text_bytes(unit, admit)?)?;
        }
        TokenData::Delim(_)
        | TokenData::Whitespace
        | TokenData::Colon
        | TokenData::Semicolon
        | TokenData::Comma
        | TokenData::Cdo
        | TokenData::Cdc => {}
    }
    Ok(total)
}

/// Counts actual boxed backing, excluding inline root and shared source owners.
/// String lengths are actual Box allocations; no String capacity is inferred.
/// Admission precedes each graph, discriminator and backing-length observation.
pub(crate) fn retained_backing_bytes_with<E>(
    values: &CssComponentValues,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<usize, CssRetainedPayloadError<E>> {
    observe(admit)?;
    let length = values.items.len();
    let mut total = length
        .checked_mul(size_of::<CssComponentValue>())
        .ok_or(CssRetainedPayloadError::CapacityOverflow)?;
    for index in 0..length {
        observe(admit)?;
        let value = &values.items[index];
        match &value.data {
            ComponentData::Token(token) => add(&mut total, token_bytes(token, admit)?)?,
            ComponentData::Function(function) => {
                add(&mut total, text_bytes(&function.name, admit)?)?;
                add(&mut total, lexeme_bytes(&function.opening, admit)?)?;
                add(
                    &mut total,
                    retained_backing_bytes_with(&function.values, admit)?,
                )?;
                add(&mut total, lexeme_bytes(&function.closing, admit)?)?;
            }
            ComponentData::Block(block) => {
                add(&mut total, lexeme_bytes(&block.opening, admit)?)?;
                add(
                    &mut total,
                    retained_backing_bytes_with(&block.values, admit)?,
                )?;
                add(&mut total, lexeme_bytes(&block.closing, admit)?)?;
            }
            ComponentData::Comment {
                content,
                spelling,
                implicit_end,
            } => {
                add(&mut total, text_bytes(content, admit)?)?;
                add(&mut total, lexeme_bytes(spelling, admit)?)?;
                add(&mut total, optional_lexeme_bytes(implicit_end, admit)?)?;
            }
        }
    }
    Ok(total)
}

fn same_scalar<E, T: PartialEq>(
    left: &T,
    right: &T,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    observe(admit)?;
    observe(admit)?;
    Ok(left == right)
}

fn same_text<E>(
    left: &str,
    right: &str,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    observe(admit)?;
    let length = left.len();
    observe(admit)?;
    if length != right.len() {
        return Ok(false);
    }
    for index in 0..length {
        observe(admit)?;
        let a = left.as_bytes()[index];
        observe(admit)?;
        let b = right.as_bytes()[index];
        if a != b {
            return Ok(false);
        }
    }
    Ok(true)
}

fn same_origin<E>(
    left: &CssValueOrigin,
    right: &CssValueOrigin,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    observe(admit)?;
    observe(admit)?;
    match (left, right) {
        (
            CssValueOrigin::UnretainedInput { byte_length: a },
            CssValueOrigin::UnretainedInput { byte_length: b },
        ) => same_scalar(a, b, admit),
        (CssValueOrigin::Programmatic, CssValueOrigin::Programmatic) => Ok(true),
        (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) => same_span(a, b, admit),
        (
            CssValueOrigin::ImplicitClosure {
                opening: a,
                at: a_at,
            },
            CssValueOrigin::ImplicitClosure {
                opening: b,
                at: b_at,
            },
        ) => {
            if !same_span(a, b, admit)? {
                return Ok(false);
            }
            same_span(a_at, b_at, admit)
        }
        _ => Ok(false),
    }
}

fn same_span<E>(
    left: &CssParsedOrigin,
    right: &CssParsedOrigin,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    observe(admit)?;
    let a = left.span();
    observe(admit)?;
    Ok(a == right.span())
}

fn same_parsed<E>(
    left: &Option<CssParsedOrigin>,
    right: &Option<CssParsedOrigin>,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    observe(admit)?;
    observe(admit)?;
    match (left, right) {
        (None, None) => Ok(true),
        (Some(a), Some(b)) => same_span(a, b, admit),
        _ => Ok(false),
    }
}

fn same_lexeme<E>(
    left: &Lexeme,
    right: &Lexeme,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    if !same_text(&left.text, &right.text, admit)? {
        return Ok(false);
    }
    same_origin(&left.origin, &right.origin, admit)
}

fn same_optional_lexeme<E>(
    left: &Option<Lexeme>,
    right: &Option<Lexeme>,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    observe(admit)?;
    observe(admit)?;
    match (left, right) {
        (None, None) => Ok(true),
        (Some(a), Some(b)) => same_lexeme(a, b, admit),
        _ => Ok(false),
    }
}

fn same_number<E>(
    left: &NumericToken,
    right: &NumericToken,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    Ok(same_scalar(&left.kind, &right.kind, admit)?
        && same_scalar(&left.has_sign, &right.has_sign, admit)?
        && same_text(&left.representation, &right.representation, admit)?)
}

fn same_token_data<E>(
    left: &TokenData,
    right: &TokenData,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    observe(admit)?;
    observe(admit)?;
    match (left, right) {
        (TokenData::Ident(a), TokenData::Ident(b))
        | (TokenData::AtKeyword(a), TokenData::AtKeyword(b))
        | (TokenData::String(a), TokenData::String(b))
        | (TokenData::Url(a), TokenData::Url(b)) => same_text(a, b, admit),
        (
            TokenData::Hash {
                value: a,
                flag: a_flag,
            },
            TokenData::Hash {
                value: b,
                flag: b_flag,
            },
        ) => Ok(same_scalar(a_flag, b_flag, admit)? && same_text(a, b, admit)?),
        (TokenData::Delim(a), TokenData::Delim(b)) => same_scalar(a, b, admit),
        (TokenData::Number(a), TokenData::Number(b))
        | (TokenData::Percentage(a), TokenData::Percentage(b)) => same_number(a, b, admit),
        (
            TokenData::Dimension {
                number: a,
                unit: a_unit,
            },
            TokenData::Dimension {
                number: b,
                unit: b_unit,
            },
        ) => Ok(same_number(a, b, admit)? && same_text(a_unit, b_unit, admit)?),
        (TokenData::Whitespace, TokenData::Whitespace)
        | (TokenData::Colon, TokenData::Colon)
        | (TokenData::Semicolon, TokenData::Semicolon)
        | (TokenData::Comma, TokenData::Comma)
        | (TokenData::Cdo, TokenData::Cdo)
        | (TokenData::Cdc, TokenData::Cdc) => Ok(true),
        _ => Ok(false),
    }
}

fn same_component<E>(
    left: &CssComponentValue,
    right: &CssComponentValue,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    if !same_parsed(&left.parsed, &right.parsed, admit)? {
        return Ok(false);
    }
    observe(admit)?;
    observe(admit)?;
    match (&left.data, &right.data) {
        (ComponentData::Token(a), ComponentData::Token(b)) => {
            Ok(same_token_data(&a.data, &b.data, admit)?
                && same_lexeme(&a.spelling, &b.spelling, admit)?
                && same_optional_lexeme(&a.implicit_end, &b.implicit_end, admit)?)
        }
        (ComponentData::Function(a), ComponentData::Function(b)) => {
            Ok(same_text(&a.name, &b.name, admit)?
                && same_lexeme(&a.opening, &b.opening, admit)?
                && equivalent_in_same_input_with(&a.values, &b.values, admit)?
                && same_lexeme(&a.closing, &b.closing, admit)?)
        }
        (ComponentData::Block(a), ComponentData::Block(b)) => {
            Ok(same_scalar(&a.kind, &b.kind, admit)?
                && same_lexeme(&a.opening, &b.opening, admit)?
                && equivalent_in_same_input_with(&a.values, &b.values, admit)?
                && same_lexeme(&a.closing, &b.closing, admit)?)
        }
        (
            ComponentData::Comment {
                content: a,
                spelling: a_spelling,
                implicit_end: a_end,
            },
            ComponentData::Comment {
                content: b,
                spelling: b_spelling,
                implicit_end: b_end,
            },
        ) => Ok(same_text(a, b, admit)?
            && same_lexeme(a_spelling, b_spelling, admit)?
            && same_optional_lexeme(a_end, b_end, admit)?),
        _ => Ok(false),
    }
}

/// Compares checked graphs after the outer owner has admitted equality of their
/// complete original input and parser context. Both graphs must be privately
/// issued from that single input. Origins compare variants and coordinates;
/// fresh source/occurrence identities are deliberately not consulted here.
/// Every byte read and fixed-size graph observation is admitted before reading.
pub(crate) fn equivalent_in_same_input_with<E>(
    left: &CssComponentValues,
    right: &CssComponentValues,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, CssRetainedPayloadError<E>> {
    if !same_scalar(&left.count, &right.count, admit)?
        || !same_scalar(&left.depth, &right.depth, admit)?
    {
        return Ok(false);
    }
    observe(admit)?;
    let length = left.items.len();
    observe(admit)?;
    if length != right.items.len() {
        return Ok(false);
    }
    for index in 0..length {
        observe(admit)?;
        let a = &left.items[index];
        observe(admit)?;
        let b = &right.items[index];
        if !same_component(a, b, admit)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accepted() -> Result<(), ()> {
        Ok(())
    }

    #[test]
    fn backing_counts_nested_boxes_and_excludes_source_and_root_header() {
        let values = parse_component_values("f(a)[1px]/*x*/").unwrap();
        // Five actual component records; f/f(/), a/a, [/], 1/px/1px,
        // and x//*x*/ contribute exactly twenty separately owned text bytes.
        assert_eq!(
            retained_backing_bytes_with(&values, &mut accepted).unwrap(),
            5 * size_of::<CssComponentValue>() + 20
        );
        let empty = parse_component_values("").unwrap();
        assert_eq!(
            retained_backing_bytes_with(&empty, &mut accepted).unwrap(),
            0
        );
    }

    #[test]
    fn backing_includes_implicit_termination_and_decoded_token_payloads() {
        let value = CssComponentValue {
            parsed: None,
            data: ComponentData::Token(ValueToken {
                data: TokenData::String("x".into()),
                spelling: programmatic_lexeme("\"x"),
                implicit_end: Some(programmatic_lexeme("\"")),
            }),
        };
        let values = CssComponentValues::try_new(vec![value], Default::default()).unwrap();
        assert_eq!(
            retained_backing_bytes_with(&values, &mut accepted).unwrap(),
            size_of::<CssComponentValue>() + 4
        );
    }

    #[test]
    fn independently_parsed_origins_compare_without_snapshot_identity() {
        let input = "var(--x, light dark)/*tail*/";
        let a = parse_component_values(input).unwrap();
        let b = parse_component_values(input).unwrap();
        assert!(equivalent_in_same_input_with(&a, &b, &mut accepted).unwrap());
        assert!(
            !a.items[0]
                .parsed
                .as_ref()
                .unwrap()
                .source()
                .same_snapshot(b.items[0].parsed.as_ref().unwrap().source())
        );
    }

    #[test]
    fn all_token_payloads_and_implicit_graphs_are_compared() {
        for input in [
            "x @a #b \"s\" url(u) + 1 2% 3px :;, <!-- -->",
            "f([a])/*x",
            "\"x",
        ] {
            let a = parse_component_values(input).unwrap();
            let b = parse_component_values(input).unwrap();
            assert!(equivalent_in_same_input_with(&a, &b, &mut accepted).unwrap());
        }
        let a = parse_component_values("f([a])").unwrap();
        let mut b = a.clone();
        let ComponentData::Function(function) = &mut b.items[0].data else {
            panic!("function")
        };
        function.closing = programmatic_lexeme(")");
        assert!(!equivalent_in_same_input_with(&a, &b, &mut accepted).unwrap());
        let a = CssComponentValues::try_new(
            vec![CssComponentValue::try_number("1").unwrap()],
            Default::default(),
        )
        .unwrap();
        let mut b = a.clone();
        let ComponentData::Token(token) = &mut b.items[0].data else {
            panic!("token")
        };
        let TokenData::Number(number) = &mut token.data else {
            panic!("number")
        };
        number.has_sign = true;
        assert!(!equivalent_in_same_input_with(&a, &b, &mut accepted).unwrap());
    }

    #[test]
    fn cancellation_at_every_observation_preserves_both_graphs() {
        let a = parse_component_values("f(light dark)/*x*/").unwrap();
        let b = parse_component_values("f(light dark)/*x*/").unwrap();
        let a_before = a.clone();
        let b_before = b.clone();
        let mut observations = 0;
        equivalent_in_same_input_with(&a, &b, &mut || {
            observations += 1;
            Ok::<_, usize>(())
        })
        .unwrap();
        for refusal in 0..observations {
            let mut seen = 0;
            let result = equivalent_in_same_input_with(&a, &b, &mut || {
                let current = seen;
                seen += 1;
                if current == refusal {
                    Err(current)
                } else {
                    Ok(())
                }
            });
            assert_eq!(result, Err(CssRetainedPayloadError::Admission(refusal)));
            assert_eq!(a, a_before);
            assert_eq!(b, b_before);
        }
        let mut observations = 0;
        retained_backing_bytes_with(&a, &mut || {
            observations += 1;
            Ok::<_, usize>(())
        })
        .unwrap();
        for refusal in 0..observations {
            let mut seen = 0;
            assert_eq!(
                retained_backing_bytes_with(&a, &mut || {
                    let current = seen;
                    seen += 1;
                    if current == refusal {
                        Err(current)
                    } else {
                        Ok(())
                    }
                }),
                Err(CssRetainedPayloadError::Admission(refusal))
            );
            assert_eq!(a, a_before);
        }
    }

    #[test]
    fn overflow_is_distinct_from_admission() {
        let mut total = usize::MAX;
        assert_eq!(
            add::<()>(&mut total, 1),
            Err(CssRetainedPayloadError::CapacityOverflow)
        );
        assert_eq!(total, usize::MAX);
    }
}
