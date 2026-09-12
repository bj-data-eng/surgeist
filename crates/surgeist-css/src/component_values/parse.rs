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
    let snapshot = CssSourceSnapshot::new(source);
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    collect(&mut parser, &snapshot, limits)
}

pub(super) fn collect(
    parser: &mut Parser<'_, '_>,
    snapshot: &CssSourceSnapshot,
    limits: CssComponentValueLimits,
) -> Result<CssComponentValues, CssComponentValueError> {
    let mut count = 0;
    let result = consume_values(parser, snapshot, limits, 0, &mut count, false);
    let (items, _) = result.map_err(|error| match error.kind {
        ParseErrorKind::Custom(error) => error,
        ParseErrorKind::Basic(_) => CssComponentValueError::new(
            CssComponentValueErrorKind::InvalidToken,
            CssValueOrigin::Parsed(parsed_origin(snapshot, &parser.state(), &parser.state())),
        ),
    })?;
    let values = CssComponentValues::from_items(items, limits)?;
    serialize::validate(&values, limits.max_css_bytes)?;
    Ok(values)
}

pub(super) fn collect_one(
    parser: &mut Parser<'_, '_>,
    snapshot: &CssSourceSnapshot,
) -> Result<CssComponentValue, CssComponentValueError> {
    let limits = CssComponentValueLimits::default();
    let (items, _) =
        consume_values(parser, snapshot, limits, 0, &mut 0, true).map_err(|error| {
            match error.kind {
                ParseErrorKind::Custom(error) => error,
                ParseErrorKind::Basic(_) => CssComponentValueError::new(
                    CssComponentValueErrorKind::InvalidToken,
                    CssValueOrigin::Parsed(parsed_origin(
                        snapshot,
                        &parser.state(),
                        &parser.state(),
                    )),
                ),
            }
        })?;
    let values = CssComponentValues::try_new(items)?;
    values.items.into_vec().into_iter().next().ok_or_else(|| {
        CssComponentValueError::new(
            CssComponentValueErrorKind::InvalidToken,
            CssValueOrigin::Parsed(parsed_origin(snapshot, &parser.state(), &parser.state())),
        )
    })
}

fn consume_values<'i, 't>(
    input: &mut Parser<'i, 't>,
    source: &CssSourceSnapshot,
    limits: CssComponentValueLimits,
    depth: u32,
    count: &mut usize,
    single: bool,
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
                let content_start = nested.position().byte_index();
                // cssparser skips unvisited blocks with its own heap-backed
                // stack. Keep this callback shallow, then construct descendants
                // from their exact bounded source range without parser recursion.
                while nested.next_including_whitespace_and_comments().is_ok() {}
                let closing_start = nested.state();
                let children = consume_range(
                    source,
                    content_start..closing_start.position().byte_index(),
                    limits,
                    depth + 1,
                    count,
                )
                .map_err(|error| nested.new_custom_error(error))?;
                Ok((children, closing_start))
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
            items.push(CssComponentValue {
                data,
                parsed: Some(parsed_origin(source, &start, &closing_end)),
            });
            if single {
                return Ok((items, input.state()));
            }
            continue;
        }
        items.push(
            leaf(source, token, spelling, origin, end.position().byte_index())
                .map_err(|error| input.new_custom_error(error))?,
        );
        if single {
            return Ok((items, input.state()));
        }
    }
}

struct OpenFrame {
    kind: CssBlockKind,
    name: Option<Box<str>>,
    opening: Lexeme,
    origin: CssParsedOrigin,
    children: Vec<CssComponentValue>,
}

impl OpenFrame {
    fn finish(
        self,
        source: &CssSourceSnapshot,
        closing_range: std::ops::Range<usize>,
        limits: CssComponentValueLimits,
    ) -> Result<CssComponentValue, CssComponentValueError> {
        let end = closing_range.end;
        let start = self.origin.span().start().byte_offset().value();
        let closing = if closing_range.is_empty() {
            Lexeme {
                text: self.kind.delimiters().1.into(),
                origin: CssValueOrigin::ImplicitClosure {
                    opening: self.origin,
                    at: range_origin(source, closing_range),
                },
            }
        } else {
            Lexeme {
                text: source.as_str()[closing_range.clone()].into(),
                origin: CssValueOrigin::Parsed(range_origin(source, closing_range)),
            }
        };
        let values = CssComponentValues::from_items(self.children, limits)?;
        let data = if let Some(name) = self.name {
            ComponentData::Function(CssFunctionValue {
                name,
                opening: self.opening,
                values,
                closing,
            })
        } else {
            ComponentData::Block(CssSimpleBlock {
                kind: self.kind,
                opening: self.opening,
                values,
                closing,
            })
        };
        Ok(CssComponentValue {
            data,
            parsed: Some(range_origin(source, start..end)),
        })
    }
}

fn consume_range(
    source: &CssSourceSnapshot,
    range: std::ops::Range<usize>,
    limits: CssComponentValueLimits,
    base_depth: u32,
    count: &mut usize,
) -> Result<Vec<CssComponentValue>, CssComponentValueError> {
    let mut items = Vec::new();
    let mut frames: Vec<OpenFrame> = Vec::new();
    let mut offset = range.start;
    while offset < range.end {
        // Restart only at a verified token boundary. Reading a single token
        // exposes delimiters without cssparser automatically skipping a block;
        // strings, comments, URLs and escapes retain dependency token semantics.
        let mut parser_input = ParserInput::new(&source.as_str()[offset..range.end]);
        let mut parser = Parser::new(&mut parser_input);
        let token = parser
            .next_including_whitespace_and_comments()
            .expect("a nonempty token-boundary suffix contains a token")
            .clone();
        let end = offset + parser.position().byte_index();
        let closing = match token {
            Token::CloseParenthesis => Some(CssBlockKind::Parenthesis),
            Token::CloseSquareBracket => Some(CssBlockKind::SquareBracket),
            Token::CloseCurlyBracket => Some(CssBlockKind::CurlyBracket),
            _ => None,
        };
        if let Some(frame) = frames.pop_if(|frame| Some(frame.kind) == closing) {
            let value = frame.finish(source, offset..end, limits)?;
            push_value(&mut items, &mut frames, value);
            offset = end;
            continue;
        }
        let origin = range_origin(source, offset..end);
        let error_at_token =
            |kind| CssComponentValueError::new(kind, CssValueOrigin::Parsed(origin.clone()));
        *count = count
            .checked_add(1)
            .ok_or_else(|| error_at_token(CssComponentValueErrorKind::CapacityOverflow))?;
        if *count > limits.max_components {
            return Err(error_at_token(CssComponentValueErrorKind::ComponentLimit));
        }
        let spelling = Lexeme {
            text: source.as_str()[offset..end].into(),
            origin: CssValueOrigin::Parsed(origin.clone()),
        };
        let kind = match token {
            Token::Function(_) | Token::ParenthesisBlock => Some(CssBlockKind::Parenthesis),
            Token::SquareBracketBlock => Some(CssBlockKind::SquareBracket),
            Token::CurlyBracketBlock => Some(CssBlockKind::CurlyBracket),
            _ => None,
        };
        if let Some(kind) = kind {
            if frames.len() >= (limits.max_depth - base_depth) as usize {
                return Err(error_at_token(CssComponentValueErrorKind::NestingLimit));
            }
            let name = if let Token::Function(name) = token {
                Some(name.as_ref().into())
            } else {
                None
            };
            frames.push(OpenFrame {
                kind,
                name,
                opening: spelling,
                origin,
                children: Vec::new(),
            });
        } else {
            let value = leaf(source, token, spelling, origin, end)?;
            push_value(&mut items, &mut frames, value);
        }
        offset = end;
    }
    while let Some(frame) = frames.pop() {
        let value = frame.finish(source, range.end..range.end, limits)?;
        push_value(&mut items, &mut frames, value);
    }
    Ok(items)
}

fn push_value(
    items: &mut Vec<CssComponentValue>,
    frames: &mut [OpenFrame],
    value: CssComponentValue,
) {
    if let Some(frame) = frames.last_mut() {
        frame.children.push(value);
    } else {
        items.push(value);
    }
}

fn leaf(
    source: &CssSourceSnapshot,
    token: Token<'_>,
    spelling: Lexeme,
    origin: CssParsedOrigin,
    end: usize,
) -> Result<CssComponentValue, CssComponentValueError> {
    let invalid = match &token {
        Token::BadString(_) => Some(CssComponentValueErrorKind::BadString),
        Token::BadUrl(_) => Some(CssComponentValueErrorKind::BadUrl),
        Token::CloseParenthesis | Token::CloseSquareBracket | Token::CloseCurlyBracket => {
            Some(CssComponentValueErrorKind::UnmatchedClosingDelimiter)
        }
        _ => None,
    };
    if let Some(kind) = invalid {
        return Err(CssComponentValueError::new(kind, spelling.origin.clone()));
    }
    let implicit_end = token_termination(&token, &spelling.text).map(|text| Lexeme {
        text: text.into_boxed_str(),
        origin: CssValueOrigin::ImplicitClosure {
            opening: origin.clone(),
            at: range_origin(source, end..end),
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
            CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                spelling.origin.clone(),
            )
        })?;
        ComponentData::Token(ValueToken {
            data,
            spelling,
            implicit_end,
        })
    };
    Ok(CssComponentValue {
        data,
        parsed: Some(origin),
    })
}

fn range_origin(source: &CssSourceSnapshot, range: std::ops::Range<usize>) -> CssParsedOrigin {
    CssParsedOrigin::from_range(source, range)
        .expect("tokenizer positions are ordered UTF-8 boundaries in the original source")
}

fn parsed_origin(
    source: &CssSourceSnapshot,
    start: &ParserState,
    end: &ParserState,
) -> CssParsedOrigin {
    CssParsedOrigin::from_range(
        source,
        start.position().byte_index()..end.position().byte_index(),
    )
    .expect("tokenizer positions are ordered UTF-8 boundaries in the original source")
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
    values.items[0].parsed = None;
    let [value] =
        <[_; 1]>::try_from(values.items.into_vec()).expect("exactly one token was checked");
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_admission_precedes_depth_checks_and_preserves_error_advancement() {
        let source = "f(g(x)) tail";
        for (depth, count, kind, start, end) in [
            (0, 0, CssComponentValueErrorKind::ComponentLimit, 0, 2),
            (0, 1, CssComponentValueErrorKind::NestingLimit, 0, 2),
            (1, 1, CssComponentValueErrorKind::ComponentLimit, 2, 7),
            (1, 2, CssComponentValueErrorKind::NestingLimit, 2, 7),
        ] {
            let snapshot = CssSourceSnapshot::new(source);
            let mut parser_input = ParserInput::new(source);
            let mut parser = Parser::new(&mut parser_input);
            let limits = CssComponentValueLimits::try_new(depth, count, 100).unwrap();
            let error = collect(&mut parser, &snapshot, limits).unwrap_err();
            assert_eq!(error.kind(), kind);
            let CssValueOrigin::Parsed(origin) = error.origin() else {
                panic!("the rejected opening retains its source origin");
            };
            assert_eq!(origin.span().start().byte_offset().value(), start);
            assert_eq!(origin.span().end().byte_offset().value(), start + 2);
            assert_eq!(parser.position().byte_index(), end);
        }
    }

    #[test]
    fn single_collection_preserves_the_following_token_after_success_or_error() {
        for (source, end, expected_error) in [
            ("f([x]) tail", 6, None),
            (
                "f([x)]) tail",
                7,
                Some(CssComponentValueErrorKind::UnmatchedClosingDelimiter),
            ),
        ] {
            let snapshot = CssSourceSnapshot::new(source);
            let mut parser_input = ParserInput::new(source);
            let mut parser = Parser::new(&mut parser_input);
            let result = collect_one(&mut parser, &snapshot);
            assert_eq!(result.err().map(|error| error.kind()), expected_error);
            assert_eq!(parser.position().byte_index(), end);
            assert_eq!(parser.expect_ident().unwrap().as_ref(), "tail");
        }
    }

    #[test]
    fn bounded_collection_preserves_the_outer_delimiter_and_source_coordinates() {
        let source = "prefix;f([x]);tail";
        let snapshot = CssSourceSnapshot::new(source);
        let mut parser_input = ParserInput::new(source);
        let mut parser = Parser::new(&mut parser_input);
        parser.expect_ident_matching("prefix").unwrap();
        parser.expect_semicolon().unwrap();
        let values = parser
            .parse_until_before(cssparser::Delimiter::Semicolon, |nested| {
                collect(nested, &snapshot, CssComponentValueLimits::default())
                    .map_err(|error| nested.new_custom_error::<_, CssComponentValueError>(error))
            })
            .unwrap();
        assert_eq!(values.serialize().unwrap().as_css(), "f([x])");
        let CssValueOrigin::Parsed(opening) = values.items()[0].origin() else {
            panic!("the function has its original opening origin");
        };
        assert_eq!(opening.span().start().byte_offset().value(), 7);
        assert_eq!(opening.span().end().byte_offset().value(), 9);
        assert_eq!(parser.position().byte_index(), 13);
        parser.expect_semicolon().unwrap();
        assert_eq!(parser.expect_ident().unwrap().as_ref(), "tail");
    }
}
