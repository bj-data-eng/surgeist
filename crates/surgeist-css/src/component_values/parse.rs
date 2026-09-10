use cssparser::{
    BasicParseErrorKind, ParseError, ParseErrorKind, Parser, ParserInput, ParserState,
};

use super::*;

pub(super) fn parse(
    source: &str,
    limits: CssComponentValueLimits,
) -> Result<CssComponentValues, CssComponentValueError> {
    if source.len() > limits.max_css_bytes {
        return Err(CssComponentValueError::new(
            CssComponentValueErrorKind::ByteLimit,
            CssValueOrigin::UnretainedInput {
                byte_length: source.len(),
            },
        ));
    }
    let snapshot = CssSourceSnapshot(Arc::from(source));
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    let mut count = 0;
    let result = consume_values(&mut parser, &snapshot, limits, 0, &mut count);
    let (items, _) = result.map_err(|error| match error.kind {
        ParseErrorKind::Custom(error) => error,
        ParseErrorKind::Basic(_) => CssComponentValueError::new(
            CssComponentValueErrorKind::InvalidToken,
            CssValueOrigin::Parsed(parsed_origin(&snapshot, &parser.state(), &parser.state())),
        ),
    })?;
    let values = CssComponentValues::from_items(items, limits)?;
    serialize::validate(&values, limits.max_css_bytes)?;
    Ok(values)
}

fn consume_values<'i, 't>(
    input: &mut Parser<'i, 't>,
    source: &CssSourceSnapshot,
    limits: CssComponentValueLimits,
    depth: u32,
    count: &mut usize,
) -> Result<(Vec<CssComponentValue>, ParserState), ParseError<'i, CssComponentValueError>> {
    let mut items = Vec::new();
    loop {
        let start = input.state();
        let token = match input.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                return Ok((items, input.state()));
            }
            Err(error) => return Err(error.into()),
        };
        let end = input.state();
        let origin = parsed_origin(source, &start, &end);
        let error_at_token =
            |kind| CssComponentValueError::new(kind, CssValueOrigin::Parsed(origin.clone()));
        *count = count.checked_add(1).ok_or_else(|| {
            input.new_custom_error(error_at_token(CssComponentValueErrorKind::CapacityOverflow))
        })?;
        if *count > limits.max_components {
            return Err(
                input.new_custom_error(error_at_token(CssComponentValueErrorKind::ComponentLimit))
            );
        }
        let spelling = Lexeme {
            text: input.slice(start.position()..end.position()).into(),
            origin: CssValueOrigin::Parsed(origin.clone()),
        };
        let block_kind = match &token {
            Token::Function(_) | Token::ParenthesisBlock => Some(CssBlockKind::Parenthesis),
            Token::SquareBracketBlock => Some(CssBlockKind::SquareBracket),
            Token::CurlyBracketBlock => Some(CssBlockKind::CurlyBracket),
            _ => None,
        };
        if let Some(kind) = block_kind {
            if depth >= limits.max_depth {
                return Err(input
                    .new_custom_error(error_at_token(CssComponentValueErrorKind::NestingLimit)));
            }
            let (children, closing_start) = input.parse_nested_block(|nested| {
                consume_values(nested, source, limits, depth + 1, count)
            })?;
            let closing_end = input.state();
            let (_, closing_text) = kind.delimiters();
            let closing =
                if closing_end.position().byte_index() > closing_start.position().byte_index() {
                    Lexeme {
                        text: input
                            .slice(closing_start.position()..closing_end.position())
                            .into(),
                        origin: CssValueOrigin::Parsed(parsed_origin(
                            source,
                            &closing_start,
                            &closing_end,
                        )),
                    }
                } else {
                    Lexeme {
                        text: closing_text.into(),
                        origin: CssValueOrigin::ImplicitClosure {
                            opening: origin,
                            at: parsed_origin(source, &closing_end, &closing_end),
                        },
                    }
                };
            let values = CssComponentValues::from_items(children, limits)
                .map_err(|error| input.new_custom_error(error))?;
            let data = if let Token::Function(name) = token {
                ComponentData::Function(CssFunctionValue {
                    name: name.as_ref().into(),
                    opening: spelling,
                    values,
                    closing,
                })
            } else {
                ComponentData::Block(CssSimpleBlock {
                    kind,
                    opening: spelling,
                    values,
                    closing,
                })
            };
            items.push(CssComponentValue { data });
            continue;
        }
        let invalid = match &token {
            Token::BadString(_) => Some(CssComponentValueErrorKind::BadString),
            Token::BadUrl(_) => Some(CssComponentValueErrorKind::BadUrl),
            Token::CloseParenthesis | Token::CloseSquareBracket | Token::CloseCurlyBracket => {
                Some(CssComponentValueErrorKind::UnmatchedClosingDelimiter)
            }
            _ => None,
        };
        if let Some(kind) = invalid {
            return Err(input.new_custom_error(error_at_token(kind)));
        }
        let implicit_end = token_termination(&token, &spelling.text).map(|text| Lexeme {
            text: text.into_boxed_str(),
            origin: CssValueOrigin::ImplicitClosure {
                opening: origin,
                at: parsed_origin(source, &end, &end),
            },
        });
        let data = if let Token::Comment(content) = token {
            ComponentData::Comment {
                content: content.into(),
                spelling,
                implicit_end,
            }
        } else {
            let data = token_data(&token, &spelling.text).ok_or_else(|| {
                input.new_custom_error(CssComponentValueError::new(
                    CssComponentValueErrorKind::InvalidToken,
                    spelling.origin.clone(),
                ))
            })?;
            ComponentData::Token(ValueToken {
                data,
                spelling,
                implicit_end,
            })
        };
        items.push(CssComponentValue { data });
    }
}

fn parsed_origin(
    source: &CssSourceSnapshot,
    start: &ParserState,
    end: &ParserState,
) -> CssParsedOrigin {
    let start = CssSourcePosition::from_cssparser(start.position(), start.source_location());
    let end = CssSourcePosition::from_cssparser(end.position(), end.source_location());
    CssParsedOrigin {
        source: source.clone(),
        span: CssSourceSpan::new(start, end)
            .expect("tokenizer source positions advance monotonically"),
    }
}

fn token_data(token: &Token<'_>, representation: &str) -> Option<TokenData> {
    Some(match token {
        Token::Ident(value) => TokenData::Ident(value.as_ref().into()),
        Token::AtKeyword(value) => TokenData::AtKeyword(value.as_ref().into()),
        Token::Hash(value) => TokenData::Hash {
            value: value.as_ref().into(),
            flag: CssHashFlag::Unrestricted,
        },
        Token::IDHash(value) => TokenData::Hash {
            value: value.as_ref().into(),
            flag: CssHashFlag::Id,
        },
        Token::QuotedString(value) => TokenData::String(value.as_ref().into()),
        Token::UnquotedUrl(value) => TokenData::Url(value.as_ref().into()),
        Token::Delim(value) => TokenData::Delim(*value),
        Token::Number { .. } => TokenData::Number(numeric(representation)),
        Token::Percentage { .. } => {
            TokenData::Percentage(numeric(&representation[..representation.len() - 1]))
        }
        Token::Dimension { unit, .. } => TokenData::Dimension {
            number: numeric(&representation[..numeric_prefix_length(representation)]),
            unit: unit.as_ref().into(),
        },
        Token::WhiteSpace(_) => TokenData::Whitespace,
        Token::Colon => TokenData::Colon,
        Token::Semicolon => TokenData::Semicolon,
        Token::Comma => TokenData::Comma,
        Token::IncludeMatch => TokenData::IncludeMatch,
        Token::DashMatch => TokenData::DashMatch,
        Token::PrefixMatch => TokenData::PrefixMatch,
        Token::SuffixMatch => TokenData::SuffixMatch,
        Token::SubstringMatch => TokenData::SubstringMatch,
        Token::CDO => TokenData::Cdo,
        Token::CDC => TokenData::Cdc,
        Token::Comment(_)
        | Token::Function(_)
        | Token::ParenthesisBlock
        | Token::SquareBracketBlock
        | Token::CurlyBracketBlock
        | Token::BadUrl(_)
        | Token::BadString(_)
        | Token::CloseParenthesis
        | Token::CloseSquareBracket
        | Token::CloseCurlyBracket => return None,
    })
}

fn numeric(representation: &str) -> NumericToken {
    NumericToken {
        representation: representation.into(),
        kind: if representation.contains(['.', 'e', 'E']) {
            CssNumericTokenKind::Number
        } else {
            CssNumericTokenKind::Integer
        },
        has_sign: representation.starts_with(['+', '-']),
    }
}

// CSS Syntax's consume-a-number spelling boundary. The dependency supplies the
// token category; this scan preserves the exact prefix without float conversion.
fn numeric_prefix_length(representation: &str) -> usize {
    let bytes = representation.as_bytes();
    let mut end = usize::from(
        bytes
            .first()
            .is_some_and(|byte| matches!(byte, b'+' | b'-')),
    );
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    if bytes.get(end) == Some(&b'.') && bytes.get(end + 1).is_some_and(u8::is_ascii_digit) {
        end += 1;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
    }
    if bytes
        .get(end)
        .is_some_and(|byte| matches!(byte, b'e' | b'E'))
    {
        let mut exponent = end + 1;
        if bytes
            .get(exponent)
            .is_some_and(|byte| matches!(byte, b'+' | b'-'))
        {
            exponent += 1;
        }
        if bytes.get(exponent).is_some_and(u8::is_ascii_digit) {
            end = exponent + 1;
            while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                end += 1;
            }
        }
    }
    end
}

fn odd_trailing_backslashes(text: &str) -> bool {
    text.bytes().rev().take_while(|byte| *byte == b'\\').count() % 2 == 1
}

fn has_unescaped_final(text: &str, closing: u8) -> bool {
    text.len() > 1
        && text.as_bytes().last() == Some(&closing)
        && !odd_trailing_backslashes(&text[..text.len() - 1])
}

fn token_termination(token: &Token<'_>, text: &str) -> Option<String> {
    match token {
        Token::QuotedString(_) => {
            let quote = text.as_bytes()[0];
            if has_unescaped_final(text, quote) {
                None
            } else {
                let mut suffix = String::new();
                // An escaped EOF contributes no code point inside a string.
                // An escaped newline preserves that effect before the new quote.
                if odd_trailing_backslashes(text) {
                    suffix.push('\n');
                }
                suffix.push(char::from(quote));
                Some(suffix)
            }
        }
        Token::UnquotedUrl(_) => {
            if has_unescaped_final(text, b')') {
                None
            } else {
                // Outside strings, an escaped EOF contributes U+FFFD.
                Some(
                    if odd_trailing_backslashes(text) {
                        "fffd )"
                    } else {
                        ")"
                    }
                    .to_owned(),
                )
            }
        }
        Token::Comment(_) if text.len() < 4 || !text.ends_with("*/") => Some("*/".to_owned()),
        Token::Ident(_)
        | Token::AtKeyword(_)
        | Token::Hash(_)
        | Token::IDHash(_)
        | Token::Dimension { .. }
            if odd_trailing_backslashes(text) =>
        {
            Some("fffd ".to_owned())
        }
        _ => None,
    }
}

pub(super) fn programmatic_token(
    spelling: &str,
    invalid_kind: CssComponentValueErrorKind,
) -> Result<CssComponentValue, CssComponentValueError> {
    let mut values = parse(spelling, CssComponentValueLimits::default())
        .map_err(|_| CssComponentValueError::programmatic(invalid_kind))?;
    if values.items.len() != 1 {
        return Err(CssComponentValueError::programmatic(invalid_kind));
    }
    let ComponentData::Token(token) = &mut values.items[0].data else {
        return Err(CssComponentValueError::programmatic(invalid_kind));
    };
    if token.implicit_end.is_some() {
        return Err(CssComponentValueError::programmatic(invalid_kind));
    }
    token.spelling.origin = CssValueOrigin::Programmatic;
    let [value] =
        <[_; 1]>::try_from(values.items.into_vec()).expect("exactly one token was checked");
    Ok(value)
}
