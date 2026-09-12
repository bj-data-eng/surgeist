use cssparser::{BasicParseErrorKind, ParseError, Parser, Token};

use crate::error::{CssFeatureId, Error, basic, invalid_syntax};
use crate::syntax::{
    CssAuthoredDeclarationValue, CssCustomPropertyDeclaredValue, CssCustomPropertyName,
    CssCustomPropertyValue,
};
use crate::validation::parse_global_keyword;

pub(super) static IMPLEMENTED_DECLARATIONS: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.declaration.custom-property")];

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.value.substitution-dependent")];

pub(crate) fn parse_custom_property_name(name: &str) -> Option<CssCustomPropertyName> {
    CssCustomPropertyName::from_ident_token(name)
}

pub(crate) fn parse_custom_property_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssCustomPropertyDeclaredValue, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.expect_ident_cloned()
        && let Some(keyword) = parse_global_keyword(&ident)
    {
        if input.is_exhausted() {
            return Ok(CssCustomPropertyDeclaredValue::Global(keyword));
        }
        return Err(invalid_syntax(
            input.current_source_location(),
            "CSS global keyword must be the entire custom property value",
        ));
    }
    input.reset(&state);

    let (authored, _) = collect_authored_declaration_value(input)?;
    Ok(CssCustomPropertyDeclaredValue::Value(
        CssCustomPropertyValue::new(authored),
    ))
}

pub(crate) fn collect_authored_declaration_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<(CssAuthoredDeclarationValue, bool), ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.position();
    let mut end = start;
    let mut has_substitution = false;
    consume_authored_value_tokens(input, &mut has_substitution, &mut end)?;
    Ok((
        CssAuthoredDeclarationValue::new(input.slice(start..end)),
        has_substitution,
    ))
}

fn consume_authored_value_tokens<'i, 't>(
    input: &mut Parser<'i, 't>,
    has_substitution: &mut bool,
    end: &mut cssparser::SourcePosition,
) -> Result<(), ParseError<'i, Error>> {
    consume_authored_value_tokens_with_restrictions(input, has_substitution, end, false)
}

fn consume_authored_value_tokens_with_restrictions<'i, 't>(
    input: &mut Parser<'i, 't>,
    has_substitution: &mut bool,
    end: &mut cssparser::SourcePosition,
    reject_fallback_top_level_tokens: bool,
) -> Result<(), ParseError<'i, Error>> {
    loop {
        input.skip_whitespace();
        let token_location = input.current_source_location();
        let token = match input.next() {
            Ok(token) => token.clone(),
            Err(error) => {
                return match error.kind {
                    BasicParseErrorKind::EndOfInput => Ok(()),
                    _ => Err(basic(error)),
                };
            }
        };
        if reject_fallback_top_level_tokens
            && matches!(&token, Token::Semicolon | Token::Delim('!'))
        {
            return Err(token_location.new_unexpected_token_error(token));
        }
        if token.is_parse_error() {
            return Err(token_location.new_unexpected_token_error(token));
        }
        if let Token::Function(name) = &token
            && name.eq_ignore_ascii_case("var")
        {
            *has_substitution = true;
            input.parse_nested_block(|input| {
                parse_variable_reference(input, has_substitution, end)
            })?;
        } else if is_nested_block_start(&token) {
            input.parse_nested_block(|input| {
                consume_authored_value_tokens(input, has_substitution, end)
            })?;
        }
        *end = input.position();
    }
}

fn parse_variable_reference<'i, 't>(
    input: &mut Parser<'i, 't>,
    has_substitution: &mut bool,
    end: &mut cssparser::SourcePosition,
) -> Result<(), ParseError<'i, Error>> {
    let name_location = input.current_source_location();
    let name = input.expect_ident_cloned().map_err(basic)?;
    if parse_custom_property_name(&name).is_none() {
        return Err(invalid_syntax(
            name_location,
            "`var()` must reference a custom property name",
        ));
    }

    if input.is_exhausted() {
        return Ok(());
    }

    input.expect_comma().map_err(basic)?;
    consume_authored_value_tokens_with_restrictions(input, has_substitution, end, true)
}

fn is_nested_block_start(token: &Token<'_>) -> bool {
    matches!(
        token,
        Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock
    )
}

/// Admit the same authored-value/var grammar over immutable components. This
/// traversal is independent of text emission: parsed slices are retained only
/// after the complete value has passed admission.
pub(super) fn authored_value_from_components(
    items: &[crate::CssComponentValue],
) -> Result<Option<CssAuthoredDeclarationValue>, crate::CssComponentValueError> {
    use crate::{CssComponentValueRef as Component, CssValueTokenRef as Token};
    let trivia = |item: &crate::CssComponentValue| {
        matches!(
            item.view(),
            Component::Comment(_) | Component::Token(Token::Whitespace(_))
        )
    };
    let Some(start) = items.iter().position(|item| !trivia(item)) else {
        return Ok(None);
    };
    let end = items
        .iter()
        .rposition(|item| !trivia(item))
        .expect("nonempty value")
        + 1;
    let items = &items[start..end];
    let mut pending = vec![(items, false)];
    while let Some((values, restricted)) = pending.pop() {
        for component in values {
            match component.view() {
                Component::Token(Token::Semicolon | Token::Delim('!')) if restricted => {
                    return Ok(None);
                }
                Component::Function(function) if function.name().eq_ignore_ascii_case("var") => {
                    let mut arguments = function
                        .values()
                        .items()
                        .iter()
                        .enumerate()
                        .filter(|(_, item)| !trivia(item));
                    let Some((_, name)) = arguments.next() else {
                        return Ok(None);
                    };
                    let Component::Token(Token::Ident(name)) = name.view() else {
                        return Ok(None);
                    };
                    if parse_custom_property_name(name).is_none() {
                        return Ok(None);
                    }
                    if let Some((index, separator)) = arguments.next() {
                        if !matches!(separator.view(), Component::Token(Token::Comma)) {
                            return Ok(None);
                        }
                        pending.push((&function.values().items()[index + 1..], true));
                    }
                }
                Component::Function(function) => pending.push((function.values().items(), false)),
                Component::Block(block) => pending.push((block.values().items(), false)),
                _ => {}
            }
        }
    }
    // Every selected sibling must belong to one contiguous original snapshot.
    // Mixed or programmatic components use canonical text only as the existing
    // authored-string payload; that text is never used for grammar selection.
    if let Some(first) = items[0].parsed_origin() {
        let mut offset = first.span().start().byte_offset().value();
        let contiguous = items.iter().all(|item| {
            let Some(origin) = item.parsed_origin() else {
                return false;
            };
            if !origin.source().same_snapshot(first.source())
                || origin.span().start().byte_offset().value() != offset
            {
                return false;
            }
            offset = origin.span().end().byte_offset().value();
            true
        });
        if contiguous {
            return Ok(Some(CssAuthoredDeclarationValue::new(
                &first.source().as_str()[first.span().start().byte_offset().value()..offset],
            )));
        }
    }
    let mut builder = crate::component_values::CssCanonicalBuilder::new(usize::MAX);
    builder.push_components(items)?;
    Ok(Some(CssAuthoredDeclarationValue::new(
        builder.finish()?.as_css(),
    )))
}
