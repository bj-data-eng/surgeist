//! What recovery reads of the structure of a source: how deeply it nests, which
//! blocks the input ended, where the nesting limit is met, and how far a
//! grammar unit reaches.
//!
//! Every answer is a query on the token tree of the source (Syntax 3 sections
//! 5.4.9 and 5.5.7 to 5.5.10), never a second tokenization, except the one
//! depth that picks the stack a parse runs on, which is asked before any tree is
//! built and so reads the tokens once and builds nothing.
use std::borrow::Cow;
use std::ops::Range;

use crate::diagnostic::error::NativeError;
use surgeist_css_parser::document::{BlockKind, GroupEnd, GroupKind, ListId, NodeId};
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::{Offset, Token};

use crate::STRUCTURAL_NESTING_LIMIT;
use crate::diagnostic::error::nesting_limit;
use crate::parser::syntax_types::Document;

/// Counts `nodes` read by a query, for the tests that bound the work.
fn scanned(nodes: usize) {
    #[cfg(test)]
    super::bounded_execution::work::add_scanned(nodes);
    let _ = nodes;
}

/// The blocks and functions an opening token opens, as nesting counts them.
fn opens(token: &Token<'_>) -> Option<BlockKind> {
    match token {
        Token::Function(_) | Token::OpenParen => Some(BlockKind::Paren),
        Token::OpenSquare => Some(BlockKind::Square),
        Token::OpenCurly => Some(BlockKind::Curly),
        _ => None,
    }
}

/// The block a closing token closes.
fn closes(token: &Token<'_>) -> Option<BlockKind> {
    match token {
        Token::CloseParen => Some(BlockKind::Paren),
        Token::CloseSquare => Some(BlockKind::Square),
        Token::CloseCurly => Some(BlockKind::Curly),
        _ => None,
    }
}

/// How deeply `source` nests blocks and functions, the way the token tree
/// counts: a closing token closes only the block it matches.
///
/// This is asked before the tree is built, to pick the stack the build and the
/// parse run on, so it walks the tokens and keeps only the open blocks.
pub(super) fn nesting_depth(source: &str) -> u32 {
    let options = Options::default();
    let mut lexer = Lexer::new(source, &options);
    let mut blocks: Vec<BlockKind> = Vec::new();
    let mut maximum = 0;
    while let Some(lexeme) = lexer.next_lexeme() {
        let token = lexeme.token();
        if let Some(opening) = opens(&token) {
            blocks.push(opening);
            maximum = maximum.max(blocks.len());
        } else if let Some(closing) = closes(&token)
            && blocks.last() == Some(&closing)
        {
            blocks.pop();
        }
    }
    u32::try_from(maximum).unwrap_or(u32::MAX)
}

/// The first opening token of `source` enclosed by exactly `depth` open
/// blocks, matching blocks as [`nesting_depth`] does.
pub(super) fn first_opening_at_depth(source: &str, depth: u32) -> Option<usize> {
    let options = Options::default();
    let mut lexer = Lexer::new(source, &options);
    let mut blocks: Vec<BlockKind> = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        let token = lexeme.token();
        if let Some(opening) = opens(&token) {
            if u32::try_from(blocks.len()).is_ok_and(|open| open == depth) {
                return Some(lexeme.range().start.value());
            }
            blocks.push(opening);
        } else if let Some(closing) = closes(&token)
            && blocks.last() == Some(&closing)
        {
            blocks.pop();
        }
    }
    None
}

/// The opening token of every block the input ended instead of closing, in
/// source order, and the start of the string or URL it ended: the places an
/// implicit closure is owed. An opening is its `(` for a function and its own
/// byte for a block.
pub(super) fn unclosed_openings(document: &Document<'_>) -> Vec<usize> {
    let mut unclosed = Vec::new();
    scanned(document.node_count());
    for node in document.nodes() {
        if !matches!(document.group_end(node), Some(GroupEnd::Eof { .. })) {
            continue;
        }
        let Some(range) = document.range(node) else {
            continue;
        };
        // The opening token of a function ends at its `(`.
        unclosed.push(match document.group_kind(node) {
            Some(GroupKind::Function) => (range.start.value()
                + document.spelling(node).map_or(1, str::len))
            .saturating_sub(1),
            _ => range.start.value(),
        });
    }
    // An unterminated string or URL runs to the end of the input, so there is
    // at most one, and it is the last token.
    if let Some(error) = document
        .lexical_errors()
        .iter()
        .rev()
        .find(|error| error.is_unterminated_string() || error.is_unterminated_url())
    {
        unclosed.push(error.range().start.value());
    }
    unclosed
}

/// A component that starts at the nesting limit or beyond it.
pub(super) struct DeniedComponent {
    pub(super) range: Range<usize>,
    pub(super) block: BlockKind,
}

/// The outermost components that start where `base_depth` and the blocks that
/// enclose them reach the nesting limit, in source order.
pub(super) fn over_limit_components(
    document: &Document<'_>,
    base_depth: u32,
) -> Vec<DeniedComponent> {
    let text_end = document.source().map_or(usize::MAX, str::len);
    let mut found = Vec::new();
    // The lists to visit, with how many blocks enclose each.
    let mut pending: Vec<(ListId, u32)> = vec![(document.root_list(), 0)];
    while let Some((list, enclosing)) = pending.pop() {
        let mut deeper = Vec::new();
        scanned(document.list(list).len());
        for &node in document.list(list) {
            let Some(children) = document.children(node) else {
                continue;
            };
            let Some(range) = document.range(node) else {
                continue;
            };
            if base_depth.saturating_add(enclosing) >= STRUCTURAL_NESTING_LIMIT {
                let block = match document.group_kind(node) {
                    Some(GroupKind::Block(block)) => block,
                    _ => BlockKind::Paren,
                };
                let end = match document.group_end(node) {
                    Some(GroupEnd::Closed {
                        range: Some(closer),
                        ..
                    }) => closer.end.value(),
                    _ => text_end,
                };
                found.push(DeniedComponent {
                    range: range.start.value()..end,
                    block,
                });
            } else {
                deeper.push((children, enclosing + 1));
            }
        }
        // The lists of a level go on the stack in reverse, so that the
        // components come out in source order.
        pending.extend(deeper.into_iter().rev());
    }
    found.sort_by_key(|component| component.range.start);
    found
}

/// The view of `source` that recovery reads its units from: every component
/// that starts beyond the nesting limit is masked, but a `{}` block, which the
/// tree denies on its own, so that it stays an opaque unit of its list. The
/// mask is as long as the original and keeps its line breaks.
pub(super) fn normalization_view<'s>(
    source: &'s str,
    components: &[DeniedComponent],
) -> Cow<'s, str> {
    if components.is_empty() {
        return Cow::Borrowed(source);
    }
    let mut view = source.as_bytes().to_vec();
    for component in components {
        if component.block == BlockKind::Curly {
            continue;
        }
        let Some(bytes) = view.get_mut(component.range.clone()) else {
            continue;
        };
        for byte in bytes {
            if !matches!(*byte, b'\n' | b'\r' | b'\x0c') {
                *byte = b' ';
            }
        }
    }
    // Only ASCII bytes are overwritten by ASCII bytes, or whole characters by
    // spaces, one per byte, so the view is valid text; if a component split a
    // character, the source stands.
    String::from_utf8(view).map_or(Cow::Borrowed(source), Cow::Owned)
}

/// Where the units of a list begin and end, as the consumer of a list chooses
/// them (Syntax 3 sections 5.4.4 to 5.4.6, 5.5.1 and 5.5.5).
#[derive(Clone, Copy)]
pub(super) enum StructuralListContext {
    /// Consume a stylesheet's contents (§5.5.1): CDO/CDC are discarded.
    Stylesheet,
    /// Parse a rule (§5.4.6): one unit, without CDO/CDC discard or a
    /// qualified-rule stop token.
    OneRule,
    /// Consume a block's contents (§5.5.5), used for every curly block:
    /// a `;` ends or precedes every unit.
    BlockContents,
}

impl StructuralListContext {
    fn ignores_prefix(self, token: &Token<'_>, at_unit_start: bool) -> bool {
        match token {
            Token::Whitespace | Token::Comment => true,
            Token::Cdo | Token::Cdc => at_unit_start && matches!(self, Self::Stylesheet),
            _ => false,
        }
    }

    /// Whether a `;` ends the unit in progress: it always ends a unit of a
    /// block's contents, and in the other lists only an at-rule (§5.5.2).
    fn semicolon_ends_unit(self, unit: Option<&UnitStart>) -> bool {
        matches!(self, Self::BlockContents) || unit.is_some_and(|unit| unit.at_rule)
    }
}

struct UnitStart {
    offset: usize,
    at_rule: bool,
}

/// An opening past the nesting limit in the prelude of the last unit, which the
/// input ended before it closed.
pub(super) struct EofLimit {
    pub(super) unit_start: usize,
    pub(super) opening_offset: usize,
    pub(super) enclosing_production: &'static str,
}

/// The unit that the end of the input leaves open with an opening past the
/// nesting limit in its prelude, if there is one: the last component of the
/// list is a block the input never closed, and the first opening inside it that
/// starts at or beyond the limit comes before any `{}` block of the unit.
pub(super) fn eof_limit(
    document: &Document<'_>,
    base_depth: u32,
    list: StructuralListContext,
) -> Option<EofLimit> {
    let root = document.root_list();
    let nodes = document.list(root);
    let last = *nodes.last()?;
    if !matches!(document.group_end(last), Some(GroupEnd::Eof { .. })) {
        return None;
    }
    let opening = first_over_limit(document, last, base_depth)?;
    // The unit the last component belongs to, found as the list's consumer
    // finds it.
    let mut unit: Option<UnitStart> = None;
    let mut first_root_curly = None;
    scanned(nodes.len());
    for &node in nodes {
        let token = document.token(node);
        let start = document.range(node)?.start.value();
        if list.ignores_prefix(&token, unit.is_none()) {
            continue;
        }
        if matches!(token, Token::Semicolon) && list.semicolon_ends_unit(unit.as_ref()) {
            unit = None;
            first_root_curly = None;
            continue;
        }
        unit.get_or_insert(UnitStart {
            offset: start,
            at_rule: matches!(token, Token::AtKeyword(_)),
        });
        if node == last {
            break;
        }
        if document.children(node).is_some() {
            // A completed component of a prelude does not end the rule; a
            // `{}` block does.
            if matches!(
                document.group_kind(node),
                Some(GroupKind::Block(BlockKind::Curly))
            ) {
                unit = None;
                first_root_curly = None;
            }
        }
    }
    // The last component is a block of the unit, and a `{}` block opens the
    // body.
    if matches!(
        document.group_kind(last),
        Some(GroupKind::Block(BlockKind::Curly))
    ) {
        first_root_curly = Some(document.range(last)?.start.value());
    }
    let unit = unit.unwrap_or(UnitStart {
        offset: document.range(last)?.start.value(),
        at_rule: false,
    });
    if first_root_curly.is_some_and(|curly| curly < opening) {
        // The body parser owns limits after the rule's prelude has ended.
        return None;
    }
    let production = if let Some(node) = nodes.iter().find(|node| {
        document
            .range(**node)
            .is_some_and(|range| range.start.value() == unit.offset)
    }) && let Token::AtKeyword(name) = document.token(*node)
    {
        if name.eq_ignore_ascii_case("media") {
            "official.media.query-list-core"
        } else if name.eq_ignore_ascii_case("supports") {
            "baseline.rule.supports"
        } else if name.eq_ignore_ascii_case("container") {
            "baseline.rule.container"
        } else {
            "baseline.selector.complex"
        }
    } else {
        "baseline.selector.complex"
    };
    Some(EofLimit {
        unit_start: unit.offset,
        opening_offset: opening,
        enclosing_production: production,
    })
}

/// The start of the first opening in the subtree of `group` (itself included)
/// that has `base_depth` and the blocks around it at the nesting limit or
/// beyond, in source order.
fn first_over_limit(document: &Document<'_>, group: NodeId, base_depth: u32) -> Option<usize> {
    // Opening depth grows along a path, so the first in source order is found
    // by walking the tree depth first.
    let mut pending: Vec<(NodeId, u32)> = vec![(group, 0)];
    while let Some((node, enclosing)) = pending.pop() {
        scanned(1);
        let range = document.range(node)?;
        if base_depth.saturating_add(enclosing) >= STRUCTURAL_NESTING_LIMIT {
            return Some(range.start.value());
        }
        if let Some(children) = document.children(node) {
            pending.extend(
                document
                    .list(children)
                    .iter()
                    .rev()
                    .filter(|child| document.children(**child).is_some())
                    .map(|child| (*child, enclosing + 1)),
            );
        }
    }
    None
}

/// Where a scan of nested components ends.
#[derive(Clone, Copy)]
pub(super) enum ScanBoundary {
    /// A declaration's value: a `;` or an unmatched `}`.
    DeclarationValue,
    /// The contents of a block that failed or is entered: an unmatched `}`.
    FailedCurlyBlock,
    /// A prelude of a specialized grammar: a `;` or a `{` block.
    SpecializedPrelude,
    /// A member of a comma-separated list: a `;`, a `{` block or a `,`.
    CommaMember,
    /// The contents of a component: its closing token.
    ComponentContents,
}

/// Validates the nesting of the components that begin at the byte `start` of
/// the source, which `document` is the token tree of, and returns the exclusive
/// end of the grammar unit they belong to. Token starts are not structural
/// opening identities: URL tokens consume their own contents, and function
/// tokens end at their actual opening parenthesis.
///
/// # Errors
///
/// Returns the nesting limit, located at the first opening whose depth is at the
/// limit: `base_depth` and the blocks opened since `start` count.
pub(super) fn scan_nested(
    document: &Document<'_>,
    source: &str,
    start: usize,
    base_depth: u32,
    enclosing_production: &'static str,
    boundary: ScanBoundary,
) -> Result<usize, NativeError> {
    let end_of_source = source.len();
    let Some(site) = document.node_at_offset(Offset::new(start.min(end_of_source))) else {
        return Ok(end_of_source);
    };
    // The component walk: the list being read, and the lists to return to when
    // an opened block ends.
    let mut list = site.list();
    let mut index = site.index();
    let mut returns: Vec<(ListId, usize)> = Vec::new();
    if site.is_closing() {
        // The scan begins at the closing token of the block that holds it.
        let group = document.list(list)[index];
        if let Some(end) = unmatched_closer(document, group, boundary) {
            return Ok(end);
        }
        index += 1;
    }
    loop {
        scanned(1);
        let nodes = document.list(list);
        let Some(&node) = nodes.get(index) else {
            // The end of the list is the closing token of its block.
            if let Some((outer, next)) = returns.pop() {
                list = outer;
                index = next;
                continue;
            }
            let Some(group) = document.parent_group(list) else {
                return Ok(end_of_source);
            };
            match closer_start(document, group) {
                // The end of the input ended the block.
                None => return Ok(end_of_source),
                Some(closer) => {
                    if let Some(end) = unmatched_closer_at(document, group, closer, boundary) {
                        return Ok(end);
                    }
                    (list, index) = document.location(group);
                    index += 1;
                    continue;
                }
            }
        };
        let token = document.token(node);
        let token_start = document
            .range(node)
            .map_or(end_of_source, |range| range.start.value());
        if closes(&token).is_some() {
            // A closing token that closes nothing here.
            if returns.is_empty()
                && matches!(
                    (boundary, closes(&token)),
                    (ScanBoundary::ComponentContents, _)
                        | (
                            ScanBoundary::FailedCurlyBlock | ScanBoundary::DeclarationValue,
                            Some(BlockKind::Curly)
                        )
                )
            {
                return Ok(token_start);
            }
            index += 1;
            continue;
        }
        if returns.is_empty() {
            let stops = match boundary {
                ScanBoundary::DeclarationValue => matches!(token, Token::Semicolon),
                ScanBoundary::SpecializedPrelude => {
                    matches!(token, Token::Semicolon | Token::OpenCurly)
                }
                ScanBoundary::CommaMember => {
                    matches!(token, Token::Semicolon | Token::OpenCurly | Token::Comma)
                }
                ScanBoundary::FailedCurlyBlock | ScanBoundary::ComponentContents => false,
            };
            if stops {
                return Ok(token_start);
            }
        }
        if opens(&token).is_some() {
            let nested_depth =
                base_depth.saturating_add(u32::try_from(returns.len()).unwrap_or(u32::MAX));
            if nested_depth >= STRUCTURAL_NESTING_LIMIT {
                return Err(nesting_limit(
                    crate::syntax_consumption::DocumentExt::snapshot(document).map_or_else(
                        || {
                            crate::source::CssSourcePosition::from_byte_offset_in(
                                source,
                                token_start,
                            )
                        },
                        |snapshot| snapshot.position_or_walk(token_start),
                    ),
                    STRUCTURAL_NESTING_LIMIT,
                    enclosing_production,
                ));
            }
            if let Some(children) = document.children(node) {
                returns.push((list, index + 1));
                list = children;
                index = 0;
                continue;
            }
        }
        index += 1;
    }
}

/// The start of the closing token of `group`, when the input did not end it.
fn closer_start(document: &Document<'_>, group: NodeId) -> Option<usize> {
    match document.group_end(group) {
        Some(GroupEnd::Closed {
            range: Some(closer),
            ..
        }) => Some(closer.start.value()),
        _ => None,
    }
}

/// The end of a scan that the closing token of the block that holds its start
/// ends, if `boundary` ends at one.
fn unmatched_closer(
    document: &Document<'_>,
    group: NodeId,
    boundary: ScanBoundary,
) -> Option<usize> {
    let closer = closer_start(document, group)?;
    unmatched_closer_at(document, group, closer, boundary)
}

fn unmatched_closer_at(
    document: &Document<'_>,
    group: NodeId,
    closer: usize,
    boundary: ScanBoundary,
) -> Option<usize> {
    let curly = matches!(
        document.group_kind(group),
        Some(GroupKind::Block(BlockKind::Curly))
    );
    match boundary {
        ScanBoundary::ComponentContents => Some(closer),
        ScanBoundary::FailedCurlyBlock | ScanBoundary::DeclarationValue if curly => Some(closer),
        _ => None,
    }
}

/// Where the first token that is no whitespace or comment begins in the part
/// `member_start..member_end` of the source, or where the part ends.
pub(super) fn first_non_trivia(source: &str, member_start: usize, member_end: usize) -> usize {
    let bounded_start = member_start.min(source.len());
    let bounded_end = member_end.min(source.len()).max(bounded_start);
    let Some(member) = source.get(bounded_start..bounded_end) else {
        return bounded_end;
    };
    let options = Options::default();
    let mut lexer = Lexer::new(member, &options);
    while let Some(lexeme) = lexer.next_lexeme() {
        if !matches!(lexeme.token(), Token::Whitespace | Token::Comment) {
            return bounded_start.saturating_add(lexeme.range().start.value());
        }
    }
    bounded_end
}
