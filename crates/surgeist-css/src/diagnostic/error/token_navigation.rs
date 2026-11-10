//! The tokens around a byte of a text, read from the token tree of the text
//! (Syntax 3 sections 5.2 and 5.4.7): the last token before the byte that is no
//! trivia, and the last at-keyword before it.
//!
//! The tree holds a token once, as the lexer read it, so a delimiter inside a
//! string, a comment or a URL is part of that token and no boundary.

use std::cell::OnceCell;

use surgeist_css_parser::document::{GroupEnd, GroupKind, ListId};
use surgeist_css_parser::lexer::Options;
use surgeist_css_parser::limits::{DepthExcess, Limits};
use surgeist_css_parser::{Document, Token};

use crate::diagnostic::error::syntax_token_kind;
use crate::diagnostic::{CssTokenKind, CssTokenSummary};

/// What the qualification of a diagnostic asks of the tokens of its text.
pub(crate) trait TokenContext {
    /// The text the tokens belong to.
    fn text(&self) -> &str;
    /// The last token that starts before the byte `offset` and is no trivia,
    /// with where it starts; none for the empty member of a list that
    /// whitespace follows a comma of (Syntax 3 section 5.4.10).
    fn previous_token(&self, offset: usize) -> Option<(usize, CssTokenSummary)>;
    /// The last at-keyword that starts before the byte `offset`, with where it
    /// starts and its name.
    fn previous_at_keyword(&self, offset: usize) -> Option<(usize, String)>;
}

impl<P> TokenContext for Document<'_, P> {
    fn text(&self) -> &str {
        self.source().unwrap_or_default()
    }

    fn previous_token(&self, offset: usize) -> Option<(usize, CssTokenSummary)> {
        previous_token_in(self, self.source()?, offset)
    }

    fn previous_at_keyword(&self, offset: usize) -> Option<(usize, String)> {
        previous_at_keyword_in(self, offset)
    }
}

/// A document of a part of a text together with the text its offsets belong
/// to: a token tree of a scope holds the offsets of the whole text, and the
/// text of the scope alone.
pub(crate) struct Scoped<'a, 'd, P> {
    pub(crate) document: &'a Document<'d, P>,
    pub(crate) text: &'a str,
}

impl<P> TokenContext for Scoped<'_, '_, P> {
    fn text(&self) -> &str {
        self.text
    }

    fn previous_token(&self, offset: usize) -> Option<(usize, CssTokenSummary)> {
        previous_token_in(self.document, self.text, offset)
    }

    fn previous_at_keyword(&self, offset: usize) -> Option<(usize, String)> {
        previous_at_keyword_in(self.document, offset)
    }
}

/// [`TokenContext::previous_token`] of `document`, whose offsets belong to
/// `text`.
fn previous_token_in<P>(
    document: &Document<'_, P>,
    text: &str,
    offset: usize,
) -> Option<(usize, CssTokenSummary)> {
    let (list, index) = boundary_at(document, offset);
    let nodes = document.list(list);
    for (back, &node) in nodes.iter().take(index).enumerate().rev() {
        let token = document.token(node);
        if matches!(token, Token::Whitespace | Token::Comment) {
            continue;
        }
        if matches!(token, Token::Comma) {
            // A whitespace token after the comma, past any comments, is a
            // member of the list of its own, which holds no token.
            let following = nodes
                .iter()
                .skip(back + 1)
                .map(|&node| document.token(node))
                .find(|token| !matches!(token, Token::Comment));
            if matches!(following, Some(Token::Whitespace)) {
                return None;
            }
        }
        return Some(match document.children(node) {
            None => {
                let range = document.range(node)?;
                (
                    range.start.value(),
                    CssTokenSummary {
                        kind: syntax_token_kind(token),
                        authored: document.spelling(node).unwrap_or_default().to_owned(),
                    },
                )
            }
            // A group that ended before the byte: its closing token is the
            // last one.
            Some(_) => match document.group_end(node) {
                Some(GroupEnd::Closed {
                    range: Some(closer),
                    ..
                }) => (
                    closer.start.value(),
                    CssTokenSummary {
                        kind: closing_kind(document.group_kind(node)),
                        authored: text
                            .get(closer.start.value()..closer.end.value())?
                            .to_owned(),
                    },
                ),
                _ => opening_summary(document, node)?,
            },
        });
    }
    // Nothing before the byte in its list: the opening of the group that
    // holds the list.
    let group = document.parent_group(list)?;
    opening_summary(document, group)
}

/// [`TokenContext::previous_at_keyword`] of `document`.
fn previous_at_keyword_in<P>(document: &Document<'_, P>, offset: usize) -> Option<(usize, String)> {
    let (mut list, mut index) = boundary_at(document, offset);
    loop {
        for &node in document.list(list).iter().take(index).rev() {
            if let Token::AtKeyword(name) = document.token(node) {
                return Some((document.range(node)?.start.value(), name.to_owned()));
            }
        }
        let group = document.parent_group(list)?;
        (list, index) = document.location(group);
    }
}

/// The kind of the token that closes a group of `kind`.
fn closing_kind(kind: Option<GroupKind>) -> CssTokenKind {
    use surgeist_css_parser::document::BlockKind;
    match kind {
        Some(GroupKind::Block(BlockKind::Square)) => CssTokenKind::CloseSquareBracket,
        Some(GroupKind::Block(BlockKind::Curly)) => CssTokenKind::CloseCurlyBracket,
        _ => CssTokenKind::CloseParenthesis,
    }
}

/// The summary of the token that opens `group`, with where it starts.
fn opening_summary<P>(
    document: &Document<'_, P>,
    group: surgeist_css_parser::document::NodeId,
) -> Option<(usize, CssTokenSummary)> {
    Some((
        document.range(group)?.start.value(),
        CssTokenSummary {
            kind: syntax_token_kind(document.token(group)),
            authored: document.spelling(group)?.to_owned(),
        },
    ))
}

/// The list that holds the boundary at the byte `offset` and the index of that
/// boundary in it: the number of nodes of the list that start before `offset`.
/// A group whose contents the byte lies in holds the boundary, down to the
/// innermost.
fn boundary_at<P>(document: &Document<'_, P>, offset: usize) -> (ListId, usize) {
    let mut list = document.root_list();
    loop {
        let nodes = document.list(list);
        let before = nodes.partition_point(|&node| {
            document
                .range(node)
                .is_some_and(|range| range.start.value() < offset)
        });
        let inside = before
            .checked_sub(1)
            .and_then(|index| nodes.get(index).copied())
            .and_then(|group| {
                let children = document.children(group)?;
                let opening_end =
                    document.range(group)?.start.value() + document.spelling(group)?.len();
                let end = document.list_end(children).map(|end| end.value());
                (opening_end <= offset && end.is_none_or(|end| offset <= end)).then_some(children)
            });
        match inside {
            Some(children) => list = children,
            None => return (list, before),
        }
    }
}

/// The tokens of a text that is lexed when the qualification first asks for
/// them: the context of a diagnostic whose caller holds no token tree.
pub(crate) struct LexedText<'s> {
    text: &'s str,
    document: OnceCell<Option<Document<'s, ()>>>,
}

impl<'s> LexedText<'s> {
    pub(crate) const fn new(text: &'s str) -> Self {
        Self {
            text,
            document: OnceCell::new(),
        }
    }

    fn document(&self) -> Option<&Document<'s, ()>> {
        self.document
            .get_or_init(|| {
                Document::parse(
                    self.text,
                    &Options::default(),
                    Limits::new(u32::MAX, usize::MAX, usize::MAX),
                    0,
                    DepthExcess::Fail,
                )
                .ok()
            })
            .as_ref()
    }
}

impl TokenContext for LexedText<'_> {
    fn text(&self) -> &str {
        self.text
    }

    fn previous_token(&self, offset: usize) -> Option<(usize, CssTokenSummary)> {
        self.document()?.previous_token(offset)
    }

    fn previous_at_keyword(&self, offset: usize) -> Option<(usize, String)> {
        self.document()?.previous_at_keyword(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::{LexedText, Scoped, TokenContext};
    use crate::source::CssSourceSnapshot;
    use crate::syntax_consumption::source_scope_document;

    /// The token tree of a scope of a text holds the offsets of the whole text
    /// and reads the tokens of the scope: with the whole text it answers as the
    /// token tree of the whole text does, at every byte of the scope.
    #[test]
    fn a_token_tree_of_a_scope_answers_with_the_offsets_of_the_whole_text() {
        let text = "a{b:c}\u{a0}@media 'x;y' /* ; */ f(g[h], i) , \u{1f600}j k(l) ; m";
        let snapshot = CssSourceSnapshot::new(text);
        let whole = LexedText::new(text);
        let scope_start = text.find("@media").unwrap();
        let scope = source_scope_document(&snapshot, scope_start..text.len()).unwrap();
        let scoped = Scoped {
            document: &scope,
            text: snapshot.as_str(),
        };
        // Nothing of the scope precedes its first byte.
        for offset in (scope_start + 1..=text.len()).filter(|&at| text.is_char_boundary(at)) {
            assert_eq!(
                scoped.previous_token(offset),
                whole.previous_token(offset),
                "token before {offset}"
            );
            assert_eq!(
                scoped.previous_at_keyword(offset),
                whole.previous_at_keyword(offset),
                "at-keyword before {offset}"
            );
        }
    }
}
