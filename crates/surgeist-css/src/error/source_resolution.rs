use super::*;
use cssparser::ParserInput;

impl Error {
    pub(super) fn resolve_source(mut self, source: &str) -> Self {
        // Component origins already refer to the shared original snapshot, which
        // may differ from this parser's same-length masked working source.
        if matches!(self.kind, ErrorKind::InvalidComponentValue(_)) {
            return self;
        }
        // Location-only errors carry byte zero until they can be resolved against
        // the authored source. Body-parser errors retain their exact nonzero cursor.
        self.position = if self.position.byte_offset().value() != 0 {
            CssSourcePosition::from_byte_offset_in(source, self.position.byte_offset().value())
        } else {
            let source_location = cssparser::SourceLocation {
                line: self.position.line().value(),
                column: self.position.column().value().saturating_add(1),
            };
            CssSourcePosition::from_source_location_in(source, source_location)
        };

        // Explicit token spellings and missing-input positions are already paired
        // by their grammar owner. Inference would replace whitespace tokens or
        // rewind bounded EOF to a token from the preceding production.
        if matches!(&self.kind, ErrorKind::InvalidDescriptorValue(detail)
            if detail.origin != DiagnosticOrigin::Inferred)
        {
            return self;
        }

        if let Some((start, summary)) = next_authored_token_at(source, self.position) {
            if let Some(token) = encountered_mut(&mut self.kind) {
                if token.kind == summary.kind {
                    token.authored = summary.authored;
                    self.position = CssSourcePosition::from_byte_offset_in(source, start);
                }
            } else if let Some(slot) = optional_encountered_mut(&mut self.kind)
                && !is_bounded_end_token(summary.kind)
            {
                *slot = Some(summary);
                self.position = CssSourcePosition::from_byte_offset_in(source, start);
            }
        }

        if let Some(slot) = optional_encountered_mut(&mut self.kind)
            && slot.is_none()
            && let Some((start, summary)) = previous_authored_token_before(source, self.position)
            && !is_boundary_token(summary.kind)
        {
            *slot = Some(summary);
            self.position = CssSourcePosition::from_byte_offset_in(source, start);
        }

        if let Some(name) = at_rule_name(&self.kind)
            && let Some(start) = authored_at_rule_start(source, self.position, name.as_str())
        {
            self.position = CssSourcePosition::from_byte_offset_in(source, start);
        }

        if let ErrorKind::InvalidAtRuleBody(detail) = &mut self.kind
            && detail.name.as_str() == "at-rule"
            && let Some((_, name)) = authored_at_rule_before(source, self.position)
        {
            detail.name = CssAtRuleName::new(name);
            detail.production = production_for_at_rule(detail.name.as_str());
        }
        self
    }
}

fn next_authored_token_at(
    source: &str,
    position: CssSourcePosition,
) -> Option<(usize, CssTokenSummary)> {
    let start = position.byte_offset().value();
    let mut offset = start;
    loop {
        let (token_start, token_end, token) =
            crate::tokenization::next_source_token(source, offset)?;
        offset = token_end;
        if matches!(token, Token::WhiteSpace(_) | Token::Comment(_)) {
            continue;
        }
        let authored = source.get(token_start..token_end)?.to_owned();
        return Some((
            token_start,
            CssTokenSummary {
                kind: token_kind(&token),
                authored,
            },
        ));
    }
}

const fn is_bounded_end_token(kind: CssTokenKind) -> bool {
    matches!(
        kind,
        CssTokenKind::Semicolon
            | CssTokenKind::CloseParenthesis
            | CssTokenKind::CloseSquareBracket
            | CssTokenKind::CloseCurlyBracket
    )
}

const fn is_boundary_token(kind: CssTokenKind) -> bool {
    is_bounded_end_token(kind)
        || matches!(
            kind,
            CssTokenKind::Colon
                | CssTokenKind::Comma
                | CssTokenKind::ParenthesisBlock
                | CssTokenKind::SquareBracketBlock
                | CssTokenKind::CurlyBracketBlock
        )
}

fn previous_authored_token_before(
    source: &str,
    position: CssSourcePosition,
) -> Option<(usize, CssTokenSummary)> {
    let end = position.byte_offset().value().min(source.len());
    let window_start = source[..end]
        .char_indices()
        .rev()
        .find_map(|(index, character)| {
            matches!(character, '{' | ';' | ':' | ',' | '(' | '[').then_some(index + 1)
        })
        .unwrap_or(0);
    let window = source.get(window_start..end)?;
    let working_source = crate::tokenization::prepare(window);
    let mut input = ParserInput::new(&working_source);
    let mut parser = Parser::new(&mut input);
    let mut previous = None;
    while !parser.is_exhausted() {
        let token_start = parser.position().byte_index();
        let token = parser
            .next_including_whitespace_and_comments()
            .ok()?
            .clone();
        let token_end = parser.position().byte_index();
        if matches!(token, Token::WhiteSpace(_) | Token::Comment(_)) {
            continue;
        }
        previous = Some((
            window_start + token_start,
            CssTokenSummary {
                kind: token_kind(&token),
                authored: window.get(token_start..token_end)?.to_owned(),
            },
        ));
    }
    previous
}

fn at_rule_name(kind: &ErrorKind) -> Option<&CssAtRuleName> {
    match kind {
        ErrorKind::InvalidAtRulePlacement(detail) => Some(&detail.name),
        ErrorKind::UnknownAtRule(detail) => Some(&detail.name),
        ErrorKind::UnsupportedAtRule(detail) => Some(&detail.name),
        _ => None,
    }
}

fn authored_at_rule_start(
    source: &str,
    position: CssSourcePosition,
    expected_name: &str,
) -> Option<usize> {
    authored_at_rule_before(source, position)
        .and_then(|(start, name)| name.eq_ignore_ascii_case(expected_name).then_some(start))
}

fn authored_at_rule_before(source: &str, position: CssSourcePosition) -> Option<(usize, String)> {
    let end = position.byte_offset().value().min(source.len());
    for (start, _) in source[..end].rmatch_indices('@') {
        let tail = source.get(start..)?;
        let mut input = ParserInput::new(tail);
        let mut parser = Parser::new(&mut input);
        if let Ok(Token::AtKeyword(name)) = parser.next_including_whitespace_and_comments() {
            return Some((start, name.to_string()));
        }
    }
    None
}

#[cfg(test)]
mod operator_tests;
