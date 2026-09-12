//! Token grammar and representation interpretation from CSS Syntax 3 §7.1.
//! Original token identity is checked before interpreting borrowed spellings.

use cssparser::{ParseError, Parser, SourceLocation, Token};

use crate::error::{Error, incomplete_descriptor_at, invalid_descriptor_token_at};
use crate::syntax::CssUnicodeRange;

struct RangeToken<'i> {
    token: Token<'i>,
    spelling: &'i str,
    location: SourceLocation,
}

impl RangeToken<'_> {
    fn error<'i>(&self) -> ParseError<'i, Error> {
        invalid_descriptor_token_at(
            self.location,
            "font-face",
            "unicode-range",
            &self.token,
            self.spelling,
        )
    }
}

fn missing<'i>(end: SourceLocation) -> ParseError<'i, Error> {
    incomplete_descriptor_at(end, "font-face", "unicode-range")
}

pub(super) fn parse<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssUnicodeRange, ParseError<'i, Error>> {
    // A valid range has at most eight tokens (u, +, six question marks).
    // A ninth token suffices to prove failure, even for unbounded malformed input.
    let mut tokens = Vec::with_capacity(9);
    let mut whitespace = None;
    loop {
        let state = input.state();
        let start = input.position();
        let location = input.current_source_location();
        let Ok(token) = input.next_including_whitespace_and_comments().cloned() else {
            break;
        };
        if matches!(token, Token::Comma) {
            if tokens.is_empty() {
                return Err(invalid_descriptor_token_at(
                    location,
                    "font-face",
                    "unicode-range",
                    &token,
                    input.slice_from(start),
                ));
            }
            input.reset(&state);
            break;
        }
        if matches!(token, Token::Comment(_)) {
            continue;
        }
        let token = RangeToken {
            token,
            spelling: input.slice_from(start),
            location,
        };
        if matches!(token.token, Token::WhiteSpace(_)) {
            if !tokens.is_empty() && whitespace.is_none() {
                whitespace = Some(token);
            }
            continue;
        }
        if let Some(space) = whitespace.take() {
            tokens.push(space);
            if tokens.len() == 9 {
                break;
            }
        }
        let terminal = !matches!(
            token.token,
            Token::Ident(_)
                | Token::Number { .. }
                | Token::Dimension { .. }
                | Token::Delim('+' | '?' | '-')
        );
        tokens.push(token);
        if tokens.len() == 9 || terminal {
            break;
        }
    }
    let end = input.current_source_location();
    check_token_grammar(&tokens, end)?;
    interpret(&tokens[1..], end)
}

fn check_token_grammar<'i>(
    tokens: &[RangeToken<'_>],
    end: SourceLocation,
) -> Result<(), ParseError<'i, Error>> {
    let first = tokens.first().ok_or_else(|| missing(end))?;
    if !matches!(&first.token, Token::Ident(value) if value.eq_ignore_ascii_case("u")) {
        return Err(first.error());
    }
    let second = tokens.get(1).ok_or_else(|| missing(end))?;
    let question_start = match second.token {
        Token::Delim('+') => {
            let third = tokens.get(2).ok_or_else(|| missing(end))?;
            match third.token {
                Token::Ident(_) => 3,
                Token::Delim('?') => 2,
                _ => return Err(third.error()),
            }
        }
        Token::Dimension { .. } => 2,
        Token::Number { .. } => {
            if let Some(third) = tokens.get(2)
                && matches!(third.token, Token::Number { .. } | Token::Dimension { .. })
            {
                return match tokens.get(3) {
                    Some(extra) => Err(extra.error()),
                    None => Ok(()),
                };
            }
            2
        }
        _ => return Err(second.error()),
    };
    for token in &tokens[question_start..] {
        if !matches!(token.token, Token::Delim('?')) {
            return Err(token.error());
        }
    }
    Ok(())
}

// The logical representation stream retains its original token owner at each
// byte. No comment-stripped input, numeric serialization, or decoded ident text
// is fed back through tokenization, and malformed long tokens are not copied.
fn interpret<'i>(
    tokens: &[RangeToken<'_>],
    end: SourceLocation,
) -> Result<CssUnicodeRange, ParseError<'i, Error>> {
    let mut text = tokens
        .iter()
        .flat_map(|token| token.spelling.bytes().map(move |byte| (byte, token)))
        .peekable();
    match text.next() {
        Some((b'+', _)) => {}
        Some((_, token)) => return Err(token.error()),
        None => return Err(missing(end)),
    }
    let endpoint = text
        .peek()
        .map(|(_, token)| *token)
        .ok_or_else(|| missing(end))?;
    let mut start = 0u32;
    let mut finish = 0u32;
    let mut digits = 0;
    let mut wildcard = false;
    while let Some(&(byte, token)) = text.peek() {
        let digit = (byte as char).to_digit(16);
        if byte != b'?' && (wildcard || digit.is_none()) {
            break;
        }
        if digits == 6 {
            return Err(token.error());
        }
        text.next();
        digits += 1;
        if byte == b'?' {
            wildcard = true;
            start *= 16;
            finish = finish * 16 + 15;
        } else if let Some(digit) = digit {
            start = start * 16 + digit;
            finish = start;
        }
    }
    if digits == 0 {
        return Err(endpoint.error());
    }
    if wildcard {
        if let Some((_, token)) = text.next() {
            return Err(token.error());
        }
        return CssUnicodeRange::try_new(start, finish).ok_or_else(|| endpoint.error());
    }
    match text.next() {
        None => return CssUnicodeRange::try_new(start, start).ok_or_else(|| endpoint.error()),
        Some((b'-', _)) => {}
        Some((_, token)) => return Err(token.error()),
    }
    let endpoint = text
        .peek()
        .map(|(_, token)| *token)
        .ok_or_else(|| missing(end))?;
    let mut finish = 0u32;
    for (digits, (byte, token)) in text.enumerate() {
        let Some(digit) = (byte as char).to_digit(16) else {
            return Err(token.error());
        };
        if digits == 6 {
            return Err(token.error());
        }
        finish = finish * 16 + digit;
    }
    CssUnicodeRange::try_new(start, finish).ok_or_else(|| endpoint.error())
}
