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
            return Err(input.new_unexpected_token_error(token));
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
