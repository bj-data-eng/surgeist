use super::token_navigation::TokenContext;
use crate::diagnostic::error::{
    DiagnosticOrigin, ErrorKind, encountered_mut, lexer_at, optional_encountered_mut,
    production_for_at_rule, syntax_token_kind,
};
use crate::diagnostic::{CssError, CssTokenKind, CssTokenSummary};
use crate::rule::CssAtRuleName;
use crate::source::CssSourcePosition;
use surgeist_css_parser::Token;

impl CssError {
    pub(super) fn resolve_source(mut self, tokens: &impl TokenContext) -> Self {
        let source = tokens.text();
        // Component origins already refer to the shared original snapshot, which
        // may differ from this parser's same-length masked working source.
        if matches!(self.kind, ErrorKind::InvalidComponentValue(_)) {
            return self;
        }
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
                    self.position = self.position.resolved_near(source, start);
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
                    self.position = self.position.resolved_near(source, start);
                }
            }
        }

        if let Some(slot) = optional_encountered_mut(&mut self.kind)
            && slot.is_none()
            && let Some((start, summary)) = tokens.previous_token(self.position.byte_offset().value())
            // A colon in a prelude belongs to it (a pseudo-class or feature),
            // rather than separating a declaration name from its value.
            && (!precedes_input(summary.kind) || (prelude && summary.kind == CssTokenKind::Colon))
            && !(prelude && summary.kind == CssTokenKind::AtKeyword)
        {
            *slot = Some(summary);
            self.position = self.position.resolved_near(source, start);
        }

        if let Some(name) = at_rule_name(&self.kind)
            && let Some(start) = tokens
                .previous_at_keyword(self.position.byte_offset().value())
                .and_then(|(start, found)| {
                    found.eq_ignore_ascii_case(name.as_str()).then_some(start)
                })
        {
            self.position = self.position.resolved_near(source, start);
        }

        if let ErrorKind::InvalidAtRuleBody(detail) = &mut self.kind
            && detail.name.as_str() == "at-rule"
            && let Some((_, name)) = tokens.previous_at_keyword(self.position.byte_offset().value())
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
    let mut lexer = lexer_at(source, position.byte_offset().value())?;
    while let Some(lexeme) = lexer.next_lexeme() {
        if matches!(lexeme.token(), Token::Whitespace | Token::Comment) {
            continue;
        }
        let range = lexeme.range();
        let (start, end) = (range.start.value(), range.end.value());
        return Some((
            start,
            CssTokenSummary {
                kind: syntax_token_kind(lexeme.token()),
                authored: source.get(start..end)?.to_owned(),
            },
        ));
    }
    None
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

fn at_rule_name(kind: &ErrorKind) -> Option<&CssAtRuleName> {
    match kind {
        ErrorKind::InvalidAtRulePlacement(detail) => Some(&detail.name),
        ErrorKind::UnknownAtRule(detail) => Some(&detail.name),
        ErrorKind::UnsupportedAtRule(detail) => Some(&detail.name),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) mod operator_tests;
