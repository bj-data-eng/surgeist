use crate::diagnostic::error::{
    DiagnosticOrigin, ErrorKind, encountered_mut, optional_encountered_mut, production_for_at_rule,
    token_kind,
};
use crate::diagnostic::{CssError, CssTokenKind, CssTokenSummary};
use crate::rule::CssAtRuleName;
use crate::source::CssSourcePosition;
use cssparser::ParserInput;
use cssparser::{Parser, Token};

impl CssError {
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

        // The crate's location rule (reference "Source coordinates and
        // diagnostics"): a token that cannot continue the production is
        // responsible, including a closing `)` or `]`; when input ends first,
        // the last authored token is. A grammar owner that already names its
        // token keeps it; one that names the end of input still yields the
        // last authored token of the value.
        let end_of_input = matches!(&self.kind, ErrorKind::InvalidDescriptorValue(detail)
            if detail.origin == DiagnosticOrigin::EndOfInput);
        if matches!(&self.kind, ErrorKind::InvalidDescriptorValue(detail)
            if detail.origin == DiagnosticOrigin::Token)
        {
            return self;
        }

        // An at-rule prelude or a style rule's selector list ends at its block
        // (Syntax 3 §5.4.7–§5.4.8), and an at-rule's own at-keyword opens
        // rather than belongs to its prelude.
        let prelude = matches!(
            self.kind,
            ErrorKind::InvalidAtRulePrelude(_) | ErrorKind::InvalidSelector(_)
        );
        if !end_of_input
            && let Some((start, summary)) = next_authored_token_at(source, self.position)
        {
            if let Some(token) = encountered_mut(&mut self.kind) {
                if token.kind == summary.kind {
                    token.authored = summary.authored;
                    self.position = CssSourcePosition::from_byte_offset_in(source, start);
                }
            } else if let Some(slot) = optional_encountered_mut(&mut self.kind) {
                let ending = ends_input(summary.kind)
                    || (prelude && summary.kind == CssTokenKind::CurlyBracketBlock);
                if !ending {
                    *slot = Some(summary);
                }
                // An ending token is no violating token, but it locates an
                // input that holds no authored token of its own.
                if !ending || slot.is_none() {
                    self.position = CssSourcePosition::from_byte_offset_in(source, start);
                }
            }
        }

        if let Some(slot) = optional_encountered_mut(&mut self.kind)
            && slot.is_none()
            && let Some((start, summary)) = previous_authored_token_before(source, self.position)
            // A colon in a prelude belongs to it (a pseudo-class or feature),
            // rather than separating a declaration name from its value.
            && (!precedes_input(summary.kind) || (prelude && summary.kind == CssTokenKind::Colon))
            && !(prelude && summary.kind == CssTokenKind::AtKeyword)
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

/// A token that ends a declaration value or rule rather than belonging to it.
pub(super) const fn ends_input(kind: CssTokenKind) -> bool {
    matches!(
        kind,
        CssTokenKind::Semicolon | CssTokenKind::CloseCurlyBracket
    )
}

/// A token before a value's own input (its name separator or enclosing
/// block), which is never the value's last authored token.
pub(super) const fn precedes_input(kind: CssTokenKind) -> bool {
    ends_input(kind) || matches!(kind, CssTokenKind::Colon | CssTokenKind::CurlyBracketBlock)
}

fn previous_authored_token_before(
    source: &str,
    position: CssSourcePosition,
) -> Option<(usize, CssTokenSummary)> {
    let end = position.byte_offset().value().min(source.len());
    let boundary = |end: usize| {
        source
            .get(..end)?
            .char_indices()
            .rev()
            .find_map(|(index, character)| {
                (matches!(character, '{' | ';' | ':' | ',' | '(' | '[')
                    && !is_escaped(source, index))
                .then_some(index)
            })
    };
    match boundary(end) {
        // Nothing authored follows the last boundary, so the token that the
        // boundary ends (a comma, or a function or block opening) is last.
        // Whitespace after a comma is a list member of its own (Syntax 3
        // §5.4.10), which holds no authored token.
        Some(index)
            if source.get(index + 1..end).is_some_and(|rest| {
                last_token_in(source, index + 1, rest).is_none()
                    && !(source[index..].starts_with(',')
                        && begins_with_whitespace_token(&source[index + 1..]))
            }) =>
        {
            let start = boundary(index).map_or(0, |before| before + 1);
            last_token_in(source, start, source.get(start..=index)?)
        }
        Some(index) => last_token_in(source, index + 1, source.get(index + 1..end)?),
        None => last_token_in(source, 0, source.get(..end)?),
    }
}

/// Whether the byte at `index` is consumed by an escape: an odd run of
/// backslashes before it ends in one that escapes it (Syntax 3 (2026 CRD)
/// §4.3.7), so `\{` is part of an ident and does not open a block.
fn is_escaped(source: &str, index: usize) -> bool {
    source.as_bytes()[..index]
        .iter()
        .rev()
        .take_while(|&&byte| byte == b'\\')
        .count()
        % 2
        == 1
}

/// Whether the first token of `rest`, past any comments, is whitespace.
fn begins_with_whitespace_token(mut rest: &str) -> bool {
    while let Some(comment) = rest.strip_prefix("/*") {
        let Some(end) = comment.find("*/") else {
            return false;
        };
        rest = &comment[end + 2..];
    }
    rest.starts_with([' ', '\t', '\n', '\r', '\u{c}'])
}

/// The last authored token of `window`, which starts at `window_start` in
/// `source`.
fn last_token_in(
    source: &str,
    window_start: usize,
    window: &str,
) -> Option<(usize, CssTokenSummary)> {
    debug_assert_eq!(
        source.get(window_start..window_start + window.len()),
        Some(window)
    );
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
        for (part, range) in
            crate::tokenization::source_token_parts(window, token, token_start..token_end)
                .into_iter()
                .flatten()
        {
            previous = Some((
                window_start + range.start,
                CssTokenSummary {
                    kind: token_kind(&part),
                    authored: window.get(range)?.to_owned(),
                },
            ));
        }
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
pub(crate) mod operator_tests;
