//! Shared selected descriptor root admission; nested matched blocks remain data.
use crate::error::{Error, basic};
use cssparser::{ParseError, Parser, Token};

pub(super) fn validate_root<'i>(
    input: &mut Parser<'i, '_>,
    owner: &str,
    descriptor: &str,
) -> Result<(), ParseError<'i, Error>> {
    let start = input.state();
    loop {
        let token_start = input.position();
        let location = input.current_source_location();
        let token = match input.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(error) if matches!(error.kind, cssparser::BasicParseErrorKind::EndOfInput) => break,
            Err(error) => return Err(basic(error)),
        };
        if matches!(
            token,
            Token::Semicolon
                | Token::CloseCurlyBracket
                | Token::CloseParenthesis
                | Token::CloseSquareBracket
        ) {
            return Err(crate::error::invalid_descriptor_token_at(
                location,
                owner,
                descriptor,
                &token,
                input.slice_from(token_start),
            ));
        }
        super::fragments::finish_nested_component(input, &token)?;
    }
    input.reset(&start);
    Ok(())
}

pub(super) fn consume_remaining_components<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<(), ParseError<'i, Error>> {
    loop {
        let token = match input.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(error) if matches!(error.kind, cssparser::BasicParseErrorKind::EndOfInput) => break,
            Err(error) => return Err(basic(error)),
        };
        if matches!(
            token,
            Token::Function(_)
                | Token::ParenthesisBlock
                | Token::SquareBracketBlock
                | Token::CurlyBracketBlock
        ) {
            input.parse_nested_block(consume_remaining_components)?;
        }
    }
    Ok(())
}
