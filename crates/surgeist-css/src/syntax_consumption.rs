//! Private CSS Syntax 3 input normalization and generic structural consumption.
//!
//! Checked feature grammar remains a later admission phase. This module builds
//! the token tree (the parser crate's [`Document`], closed over CSS's payload)
//! of source text or of checked components, and keeps the generic rule-unit
//! consumers that recovery reads it with.

use std::ops::Range;

use surgeist_css_parser::document::{BlockKind, Component, GroupEnd, ListId, NodeId};
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::limits::{DepthExcess, Limits, Meter};
use surgeist_css_parser::token::{HashKind, Number};
use surgeist_css_parser::{Offset, Token};

use crate::component::values::CssComponentValueErrorKind;
use crate::component::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueRef, CssHashFlag,
    CssValueOrigin, CssValueTokenRef,
};
use crate::parser::syntax_types::{Attached, Document, Host, Site};
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
use crate::values::color_adjust::utf16_value::Control;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind<'a> {
    Ident(&'a str),
    AtKeyword(&'a str),
    Function(&'a str),
    Opening(CssBlockKind),
    Closing(CssBlockKind),
    Whitespace,
    Comment,
    Semicolon,
    Comma,
    Colon,
    Cdo,
    Cdc,
    Delim(char),
    BadString,
    BadUrl,
    Other,
}

/// The rule-driver's reading of a token of the tree.
pub(crate) fn token_kind<'a>(token: Token<'a>) -> TokenKind<'a> {
    match token {
        Token::Ident(name) => TokenKind::Ident(name),
        Token::AtKeyword(name) => TokenKind::AtKeyword(name),
        Token::Function(name) => TokenKind::Function(name),
        Token::OpenParen => TokenKind::Opening(CssBlockKind::Parenthesis),
        Token::OpenSquare => TokenKind::Opening(CssBlockKind::SquareBracket),
        Token::OpenCurly => TokenKind::Opening(CssBlockKind::CurlyBracket),
        Token::CloseParen => TokenKind::Closing(CssBlockKind::Parenthesis),
        Token::CloseSquare => TokenKind::Closing(CssBlockKind::SquareBracket),
        Token::CloseCurly => TokenKind::Closing(CssBlockKind::CurlyBracket),
        Token::Whitespace => TokenKind::Whitespace,
        Token::Comment => TokenKind::Comment,
        Token::Semicolon => TokenKind::Semicolon,
        Token::Comma => TokenKind::Comma,
        Token::Colon => TokenKind::Colon,
        Token::Cdo => TokenKind::Cdo,
        Token::Cdc => TokenKind::Cdc,
        Token::Delim(value) => TokenKind::Delim(value),
        Token::BadString => TokenKind::BadString,
        Token::BadUrl => TokenKind::BadUrl,
        Token::Hash { .. }
        | Token::String(_)
        | Token::Url(_)
        | Token::Number(_)
        | Token::Percentage(_)
        | Token::Dimension { .. }
        | Token::UnicodeRange(_) => TokenKind::Other,
    }
}

/// The rule-driver's reading of a token produced by the native provider, for
/// the recovery code that still drives it.
pub(crate) fn native_token_kind<'a>(token: &'a cssparser::Token<'_>) -> TokenKind<'a> {
    use cssparser::Token as Native;
    match token {
        Native::Ident(name) => TokenKind::Ident(name),
        Native::AtKeyword(name) => TokenKind::AtKeyword(name),
        Native::Function(name) => TokenKind::Function(name),
        Native::ParenthesisBlock => TokenKind::Opening(CssBlockKind::Parenthesis),
        Native::SquareBracketBlock => TokenKind::Opening(CssBlockKind::SquareBracket),
        Native::CurlyBracketBlock => TokenKind::Opening(CssBlockKind::CurlyBracket),
        Native::CloseParenthesis => TokenKind::Closing(CssBlockKind::Parenthesis),
        Native::CloseSquareBracket => TokenKind::Closing(CssBlockKind::SquareBracket),
        Native::CloseCurlyBracket => TokenKind::Closing(CssBlockKind::CurlyBracket),
        Native::WhiteSpace(_) => TokenKind::Whitespace,
        Native::Comment(_) => TokenKind::Comment,
        Native::Semicolon => TokenKind::Semicolon,
        Native::Comma => TokenKind::Comma,
        Native::Colon => TokenKind::Colon,
        Native::CDO => TokenKind::Cdo,
        Native::CDC => TokenKind::Cdc,
        Native::Delim(value) => TokenKind::Delim(*value),
        Native::BadString(_) => TokenKind::BadString,
        Native::BadUrl(_) => TokenKind::BadUrl,
        _ => TokenKind::Other,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GenericRuleDispatch {
    At,
    Qualified,
}

pub(crate) fn rule_dispatch(kind: TokenKind<'_>) -> GenericRuleDispatch {
    if matches!(kind, TokenKind::AtKeyword(_)) {
        GenericRuleDispatch::At
    } else {
        GenericRuleDispatch::Qualified
    }
}

pub(crate) fn semicolon_terminates_rule(dispatch: GenericRuleDispatch) -> bool {
    matches!(dispatch, GenericRuleDispatch::At)
}

/// A real boundary of a document, and where it is in the document's text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SyntaxBoundary {
    pub(crate) list: ListId,
    pub(crate) index: usize,
    /// The byte offset of the boundary in the text the document was built
    /// over; none for a boundary of checked components that carry no text.
    pub(crate) offset: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SyntaxInputLimits {
    pub(crate) max_depth: u32,
    pub(crate) max_components: usize,
    pub(crate) max_known_spelling_bytes: usize,
}
impl Default for SyntaxInputLimits {
    fn default() -> Self {
        Self {
            max_depth: 256,
            max_components: usize::MAX,
            max_known_spelling_bytes: usize::MAX,
        }
    }
}
impl SyntaxInputLimits {
    /// The parser crate's limits for a build. A build that starts deeper than
    /// zero is told so by its caller (`base_depth`); the ceiling stays the
    /// whole one.
    fn limits(self) -> Limits {
        Limits::new(
            self.max_depth.min(256),
            self.max_components,
            self.max_known_spelling_bytes,
        )
    }
}

// ----- building a document

fn capacity() -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::CapacityOverflow,
        CssValueOrigin::Programmatic,
    )
}

fn failure_kind(error: &surgeist_css_parser::document::BuildError) -> CssComponentValueErrorKind {
    if error.is_depth_limit() {
        CssComponentValueErrorKind::NestingLimit
    } else if error.is_node_limit() {
        CssComponentValueErrorKind::ComponentLimit
    } else if error.is_byte_limit() {
        CssComponentValueErrorKind::ByteLimit
    } else {
        CssComponentValueErrorKind::CapacityOverflow
    }
}

/// Maps the failure to build over `text` (which starts at `window` in
/// `original`) to the error CSS reports: the kind, and the origin of the token
/// the build stopped at, which is read again from the text.
fn source_failure(
    error: &surgeist_css_parser::document::BuildError,
    text: &str,
    window: usize,
    original: &CssSourceSnapshot,
) -> CssComponentValueError {
    let kind = failure_kind(error);
    let origin = error
        .offset()
        .and_then(|at| {
            let start = at.value();
            let local = text.get(start.checked_sub(window)?..)?;
            let options = Options::default().origin(at);
            let end = Lexer::new(local, &options)
                .next_lexeme()?
                .range()
                .end
                .value();
            let origin = CssParsedOrigin::from_range(original, start..end)?;
            Some(CssValueOrigin::Parsed(origin))
        })
        .unwrap_or(CssValueOrigin::Programmatic);
    CssComponentValueError::new(kind, origin)
}

fn too_long(length: usize) -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::ByteLimit,
        CssValueOrigin::UnretainedInput {
            byte_length: length,
        },
    )
}

/// Parses `text`, which is `original`'s text from `window` on, and attaches
/// the host that knows `original`.
fn build_source<'a>(
    text: &'a str,
    window: usize,
    original: &CssSourceSnapshot,
    limits: SyntaxInputLimits,
    base_depth: u32,
) -> Result<Document<'a>, CssComponentValueError> {
    if text.len() > limits.max_known_spelling_bytes {
        return Err(too_long(text.len()));
    }
    let options = Options::default().origin(Offset::new(window));
    let mut document = Document::parse(
        text,
        &options,
        limits.limits(),
        base_depth,
        DepthExcess::Fail,
    )
    .map_err(|error| source_failure(&error, text, window, original))?;
    document.attach(Host::source(original).into_attached());
    Ok(document)
}

pub(crate) fn source_document<'a>(
    text: &'a str,
    original: &CssSourceSnapshot,
    base_depth: u32,
) -> Result<Document<'a>, CssComponentValueError> {
    build_source(text, 0, original, SyntaxInputLimits::default(), base_depth)
}

/// The tree of the window `range` of the text `original` holds, whose
/// positions are those of `original`.
pub(crate) fn source_window_document(
    original: &CssSourceSnapshot,
    range: Range<usize>,
) -> Result<Document<'_>, CssComponentValueError> {
    let text = original
        .as_str()
        .get(range.clone())
        .expect("bounded source range");
    build_source(text, range.start, original, SyntaxInputLimits::default(), 0)
}

/// The tree of the window `range` of the text `original` holds, tokenized with
/// unicode ranges allowed (Syntax 3 section 5.5.11), whose positions are those
/// of `original`.
pub(crate) fn unicode_range_document(
    original: &CssSourceSnapshot,
    range: Range<usize>,
) -> Result<Document<'_>, CssComponentValueError> {
    let text = original
        .as_str()
        .get(range.clone())
        .expect("bounded source range");
    let limits = SyntaxInputLimits::default();
    if text.len() > limits.max_known_spelling_bytes {
        return Err(too_long(text.len()));
    }
    let mut document = surgeist_css_parser::unicode_range::consume_value(
        text,
        Offset::new(range.start),
        limits.limits(),
    )
    .map_err(|error| source_failure(&error, text, range.start, original))?;
    document.attach(Host::source(original).into_attached());
    Ok(document)
}

/// The tree of `text`, which no snapshot holds, tokenized with unicode ranges
/// allowed, whose positions are those of `text`.
pub(crate) fn unicode_range_text_document(
    text: &str,
) -> Result<Document<'_>, CssComponentValueError> {
    let limits = SyntaxInputLimits::default();
    if text.len() > limits.max_known_spelling_bytes {
        return Err(too_long(text.len()));
    }
    surgeist_css_parser::unicode_range::consume_value(text, Offset::new(0), limits.limits())
        .map_err(|error| {
            CssComponentValueError::new(failure_kind(&error), CssValueOrigin::Programmatic)
        })
}

/// The tree of the window `range` of the text `original` holds, for a native
/// callback that hands a ported grammar its whole scope, whose positions are
/// those of `original`.
///
/// The tree has no ceiling on nesting of its own. The scope lies inside blocks
/// the structural scans of the recovery state have already counted, and they,
/// not the depth of a tree built for the scope's contents, judge each unit
/// against the ceiling of 256 levels (counted from the whole source): a unit
/// nested past it is dropped alone, and the units around it survive. A tree
/// that failed the build instead would drop the whole scope. Building is
/// iterative, so depth costs no stack.
pub(crate) fn source_scope_document(
    original: &CssSourceSnapshot,
    range: Range<usize>,
) -> Result<Document<'_>, CssComponentValueError> {
    let text = original
        .as_str()
        .get(range.clone())
        .expect("bounded source range");
    let options = Options::default().origin(Offset::new(range.start));
    let mut document = Document::parse(
        text,
        &options,
        Limits::new(u32::MAX, usize::MAX, usize::MAX),
        0,
        DepthExcess::Fail,
    )
    .map_err(|error| source_failure(&error, text, range.start, original))?;
    document.attach(Host::source(original).into_attached());
    Ok(document)
}

/// How far past a token the lexer looks to decide where the token ends, in
/// scalars (the machine's lookahead window).
const LEXER_LOOKAHEAD: usize = 4;

/// The tree of the first `end` bytes of `original`'s text and the few scalars
/// after them that the lexer reads to end the tokens before `end`, under no
/// limit of its own: for a reader that wants to know what a build cut short at
/// `end` had admitted. The text it reads is bounded by `end`, so the tree is no
/// larger than what the limited build held; an `end` that is not a character
/// boundary reads nothing.
pub(crate) fn source_prefix_unlimited(
    original: &CssSourceSnapshot,
    end: usize,
) -> Result<Document<'_>, CssComponentValueError> {
    let text = original.as_str();
    let head = text.get(..end).unwrap_or_default();
    let slack: usize = text[head.len()..]
        .chars()
        .take(LEXER_LOOKAHEAD)
        .map(char::len_utf8)
        .sum();
    let window = &text[..head.len() + slack];
    let mut document = Document::parse(
        window,
        &Options::default(),
        Limits::new(u32::MAX, usize::MAX, usize::MAX),
        0,
        DepthExcess::Fail,
    )
    .map_err(|error| source_failure(&error, window, 0, original))?;
    document.attach(Host::source(original).into_attached());
    Ok(document)
}

pub(crate) fn source_document_with_limits(
    text: &str,
    limits: crate::component::CssComponentValueLimits,
) -> Result<Document<'_>, CssComponentValueError> {
    source_document_with_optional_snapshot(text, None, limits)
}

pub(crate) fn source_document_snapshot_with_limits(
    original: &CssSourceSnapshot,
    limits: crate::component::CssComponentValueLimits,
) -> Result<Document<'_>, CssComponentValueError> {
    source_document_with_optional_snapshot(original.as_str(), Some(original), limits)
}

fn source_document_with_optional_snapshot<'a>(
    text: &'a str,
    original: Option<&CssSourceSnapshot>,
    limits: crate::component::CssComponentValueLimits,
) -> Result<Document<'a>, CssComponentValueError> {
    if text.len() > limits.max_css_bytes() {
        return Err(too_long(text.len()));
    }
    let original = original
        .cloned()
        .unwrap_or_else(|| CssSourceSnapshot::new(text));
    build_source(
        text,
        0,
        &original,
        SyntaxInputLimits {
            max_depth: limits.max_nesting_depth(),
            max_components: limits.max_components(),
            max_known_spelling_bytes: limits.max_css_bytes(),
        },
        0,
    )
}

/// The tree recovery reads a whole source with, which outlives the call that
/// built it and owns `text`, the allocation it is given. Nesting past the
/// structural limit is denied block by block, as the structural scan found it:
/// `text` is the scan's view of the source.
pub(crate) fn recovery_source_document(
    text: impl Into<String>,
    original: &CssSourceSnapshot,
    base_depth: u32,
) -> Result<Document<'static>, CssComponentValueError> {
    let limits = SyntaxInputLimits::default();
    let text: String = text.into();
    if text.len() > limits.max_known_spelling_bytes {
        return Err(too_long(text.len()));
    }
    // The view differs from the original only in bytes the scan masked, never
    // in where a token starts, so the original answers for the failing token.
    let mut document = Document::parse_owned(
        text,
        &Options::default(),
        limits.limits(),
        base_depth,
        DepthExcess::DenyBlock,
    )
    .map_err(|error| source_failure(&error, original.as_str(), 0, original))?;
    document.attach(Host::source(original).into_attached());
    Ok(document)
}

/// The token of a checked component's leaf or opening lexeme.
fn checked_token(component: &CssComponentValue) -> Token<'_> {
    match component.view() {
        CssComponentValueRef::Function(value) => Token::Function(value.name()),
        CssComponentValueRef::Block(value) => match value.kind() {
            CssBlockKind::Parenthesis => Token::OpenParen,
            CssBlockKind::SquareBracket => Token::OpenSquare,
            CssBlockKind::CurlyBracket => Token::OpenCurly,
        },
        CssComponentValueRef::Comment(_) => Token::Comment,
        CssComponentValueRef::Token(value) => match value {
            CssValueTokenRef::Ident(value) => Token::Ident(value),
            CssValueTokenRef::AtKeyword(value) => Token::AtKeyword(value),
            CssValueTokenRef::Hash { value, flag } => Token::Hash {
                value,
                kind: match flag {
                    CssHashFlag::Id => HashKind::Id,
                    CssHashFlag::Unrestricted => HashKind::Unrestricted,
                },
            },
            CssValueTokenRef::String(value) => Token::String(value),
            CssValueTokenRef::Url(value) => Token::Url(value),
            CssValueTokenRef::Delim(value) => Token::Delim(value),
            CssValueTokenRef::Number(number) => {
                Token::Number(checked_number(number.representation()))
            }
            CssValueTokenRef::Percentage(number) => {
                Token::Percentage(checked_number(number.representation()))
            }
            CssValueTokenRef::Dimension { number, unit } => Token::Dimension {
                number: checked_number(number.representation()),
                unit,
            },
            CssValueTokenRef::Whitespace(_) => Token::Whitespace,
            CssValueTokenRef::Colon => Token::Colon,
            CssValueTokenRef::Semicolon => Token::Semicolon,
            CssValueTokenRef::Comma => Token::Comma,
            CssValueTokenRef::Cdo => Token::Cdo,
            CssValueTokenRef::Cdc => Token::Cdc,
        },
    }
}

fn checked_number(representation: &str) -> Number<'_> {
    Number::parse(representation).expect("a checked numeric token spells a number")
}

/// The interval of a checked component's lexeme in the source it was parsed
/// from, when it has one.
fn lexeme_range(origin: &CssValueOrigin) -> Option<Range<Offset>> {
    match origin {
        CssValueOrigin::Parsed(origin) => Some(
            Offset::new(origin.span().start().byte_offset().value())
                ..Offset::new(origin.span().end().byte_offset().value()),
        ),
        _ => None,
    }
}

/// Builds the tree of checked `values`, which start `base_depth` deep.
///
/// Components that were parsed keep the intervals they were parsed at, in the
/// text of their snapshot; a metered build (`control`) also keeps that text.
fn build_checked<'a>(
    values: &'a [CssComponentValue],
    limits: SyntaxInputLimits,
    base_depth: u32,
    control: Option<(&'a Control<'a>, &'a CssSourceSnapshot)>,
) -> Result<Document<'a>, CssComponentValueError> {
    use crate::parser::syntax_types::TreeBuilder;
    enum Work<'a> {
        Value(&'a CssComponentValue, ListId),
        Close(NodeId, &'a CssComponentValue),
    }
    let meter = control.map(|(control, _)| control);
    let control_step = || meter.map_or(Ok(()), |control| control.step().map_err(|()| capacity()));
    let reserve = |pending: &mut Vec<Work<'a>>| match meter {
        Some(control) => control.reserve(pending, 1).map_err(|()| capacity()),
        None => pending.try_reserve(1).map_err(|_| capacity()),
    };
    // The builder counts the base depth itself, so the ceiling is the whole one.
    let mut builder: TreeBuilder<'a> = Document::builder(
        limits.limits(),
        base_depth,
        meter.map(|control| control as &dyn Meter),
    )
    .map_err(|_| capacity())?;
    let mut pending = Vec::new();
    let root = builder.root();
    for value in values.iter().rev() {
        control_step()?;
        reserve(&mut pending)?;
        pending.push(Work::Value(value, root));
    }
    while !pending.is_empty() {
        control_step()?;
        let Some(work) = pending.pop() else { break };
        match work {
            Work::Value(value, list) => {
                let (_, origin) = value
                    .structural_lexeme(false)
                    .expect("a component owns its leaf or opening lexeme");
                let fail = |error: surgeist_css_parser::document::BuildError| {
                    CssComponentValueError::new(failure_kind(&error), origin.clone())
                };
                let range = lexeme_range(origin);
                let attached = Some(Attached::Component(value));
                let children = match value.view() {
                    CssComponentValueRef::Function(value) => Some(value.values().items()),
                    CssComponentValueRef::Block(value) => Some(value.values().items()),
                    _ => None,
                };
                let spelled = range.is_some();
                if let Some(children) = children {
                    let (node, child) = builder
                        .open(list, checked_token(value), range, attached)
                        .map_err(fail)?;
                    if !spelled {
                        let (text, _) = value.structural_lexeme(false).expect("opening lexeme");
                        builder.charge_spelling(text.len()).map_err(fail)?;
                    }
                    reserve(&mut pending)?;
                    pending.push(Work::Close(node, value));
                    for value in children.iter().rev() {
                        control_step()?;
                        reserve(&mut pending)?;
                        pending.push(Work::Value(value, child));
                    }
                } else {
                    builder
                        .leaf(list, checked_token(value), range, attached)
                        .map_err(fail)?;
                    if !spelled {
                        let (text, _) = value.structural_lexeme(false).expect("leaf lexeme");
                        builder.charge_spelling(text.len()).map_err(fail)?;
                    }
                }
            }
            Work::Close(node, value) => {
                let (text, origin) = value
                    .structural_lexeme(true)
                    .expect("a group owns its closing lexeme");
                let fail = |error: surgeist_css_parser::document::BuildError| {
                    CssComponentValueError::new(failure_kind(&error), origin.clone())
                };
                if let CssValueOrigin::ImplicitClosure { at, .. } = origin {
                    let at = Some(Offset::new(at.span().start().byte_offset().value()));
                    builder.close(node, GroupEnd::Eof { at }).map_err(fail)?;
                } else {
                    let range = lexeme_range(origin);
                    let spelled = range.is_some();
                    builder
                        .close(
                            node,
                            GroupEnd::Closed {
                                range,
                                payload: Some(Attached::Component(value)),
                            },
                        )
                        .map_err(fail)?;
                    if !spelled {
                        builder.charge_spelling(text.len()).map_err(fail)?;
                    }
                }
            }
        }
    }
    if let Some(control) = meter {
        let bytes = pending.capacity() * std::mem::size_of::<Work<'a>>();
        drop(pending);
        control.released(bytes);
    }
    let source = control.map(|(_, source)| source);
    let mut document = builder.finish(source.map(CssSourceSnapshot::as_str));
    document.attach(Host::checked(source, meter).into_attached());
    Ok(document)
}

/// One token of a test document of supplied tokens: a checked component's
/// leaf, or a lexical fault no checked component can hold.
#[cfg(test)]
pub(crate) enum Supplied<'a> {
    Checked(&'a CssComponentValue),
    /// A fault with no interval: programmatic.
    Fault(Token<'a>),
    /// A fault at an interval of the document's text.
    FaultAt(Token<'a>, Range<usize>),
}

/// A root list of supplied tokens, as a host that holds a mixture of checked
/// components and error tokens would build it. A fault has no payload, and no
/// interval unless it is [`Supplied::FaultAt`] one of `text`, which the faults
/// positions then belong to; a fault with none is programmatic.
#[cfg(test)]
pub(crate) fn supplied_document<'a>(
    items: &[Supplied<'a>],
    text: Option<&'a CssSourceSnapshot>,
) -> Document<'a> {
    let mut builder: crate::parser::syntax_types::TreeBuilder<'a> =
        Document::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    for item in items {
        match item {
            Supplied::Checked(component) => {
                let (_, origin) = component.structural_lexeme(false).unwrap();
                builder
                    .leaf(
                        root,
                        checked_token(component),
                        lexeme_range(origin),
                        Some(Attached::Component(component)),
                    )
                    .unwrap();
            }
            Supplied::Fault(token) => {
                builder.leaf(root, *token, None, None).unwrap();
            }
            Supplied::FaultAt(token, range) => {
                builder
                    .leaf(
                        root,
                        *token,
                        Some(Offset::new(range.start)..Offset::new(range.end)),
                        None,
                    )
                    .unwrap();
            }
        }
    }
    let mut document = builder.finish(text.map(CssSourceSnapshot::as_str));
    document.attach(Host::checked(text, None).into_attached());
    document
}

pub(crate) fn checked_document<'a>(
    values: &'a [CssComponentValue],
    limits: SyntaxInputLimits,
    base_depth: u32,
) -> Result<Document<'a>, CssComponentValueError> {
    build_checked(values, limits, base_depth, None)
}

/// The tree of the checked components of one controlled front, whose text is
/// `source`: metered by `control`, with the intervals the components were parsed
/// at.
pub(crate) fn checked_document_controlled<'a>(
    values: &'a [CssComponentValue],
    limits: SyntaxInputLimits,
    base_depth: u32,
    control: &'a Control<'a>,
    source: &'a CssSourceSnapshot,
) -> Result<Document<'a>, CssComponentValueError> {
    build_checked(values, limits, base_depth, Some((control, source)))
}

// ----- reading a document

/// What the rule-unit consumers and recovery read of a document, on top of
/// the parser crate's own navigation.
pub(crate) trait DocumentExt {
    /// The rule driver's reading of the token the node holds; for a group, its
    /// opening token.
    fn kind(&self, node: NodeId) -> TokenKind<'_>;
    fn is_trivia(&self, node: NodeId) -> bool;
    /// Whether the node is a `{}`-block, denied or not.
    fn is_curly_block(&self, node: NodeId) -> bool;
    /// The error of a `{}`-block denied for nesting too deep.
    fn denied_error(&self, node: NodeId) -> Option<CssComponentValueError>;
    /// The snapshot the document's text belongs to.
    fn snapshot(&self) -> Option<&CssSourceSnapshot>;
    /// Where the node's own token came from: for a group, its opening token.
    fn origin(&self, node: NodeId) -> CssValueOrigin;
    /// Where the closing token of the group `node` came from, when it has one.
    fn closing_origin(&self, node: NodeId) -> CssValueOrigin;
    fn boundary(&self, list: ListId, index: usize) -> SyntaxBoundary;
    fn cursor(&self, list: ListId) -> SyntaxCursor<'_>;
    fn cursor_range(&self, range: &SyntaxRange) -> SyntaxCursor<'_>;
}

impl DocumentExt for Document<'_> {
    fn kind(&self, node: NodeId) -> TokenKind<'_> {
        token_kind(self.token(node))
    }

    fn is_trivia(&self, node: NodeId) -> bool {
        self.token(node).is_trivia()
    }

    fn is_curly_block(&self, node: NodeId) -> bool {
        matches!(
            self.component(node),
            Component::Block {
                kind: BlockKind::Curly,
                ..
            } | Component::Denied { .. }
        )
    }

    fn denied_error(&self, node: NodeId) -> Option<CssComponentValueError> {
        matches!(self.component(node), Component::Denied { .. }).then(|| {
            CssComponentValueError::new(CssComponentValueErrorKind::NestingLimit, self.origin(node))
        })
    }

    fn snapshot(&self) -> Option<&CssSourceSnapshot> {
        crate::parser::syntax_types::host(self)?.snapshot()
    }

    fn origin(&self, node: NodeId) -> CssValueOrigin {
        if let Some(Attached::Component(component)) = self.payload(node) {
            let (_, origin) = component
                .structural_lexeme(false)
                .expect("a component owns its leaf or opening lexeme");
            return origin.clone();
        }
        let (Some(snapshot), Some(range), Some(spelling)) =
            (self.snapshot(), self.range(node), self.spelling(node))
        else {
            return CssValueOrigin::Programmatic;
        };
        // A denied block spells its whole extent; its token is the opening.
        let spelling = if matches!(self.component(node), Component::Denied { .. }) {
            spelling.get(..1).unwrap_or(spelling)
        } else {
            spelling
        };
        let start = range.start.value();
        let end = start + spelling.len();
        if snapshot.as_str().get(start..end) == Some(spelling)
            && let Some(origin) = CssParsedOrigin::from_range(snapshot, start..end)
        {
            CssValueOrigin::Parsed(origin)
        } else {
            CssValueOrigin::Programmatic
        }
    }

    fn closing_origin(&self, node: NodeId) -> CssValueOrigin {
        if let Some(Attached::Component(component)) = self.payload(node) {
            let closing = match component.view() {
                CssComponentValueRef::Function(function) => function.closing_origin(),
                CssComponentValueRef::Block(block) => block.closing_origin(),
                _ => unreachable!("a group owns its closing origin"),
            };
            return closing.clone();
        }
        let range = match self.group_end(node) {
            Some(GroupEnd::Closed {
                range: Some(range), ..
            }) => range,
            // A group the input ended instead of closing is closed implicitly,
            // at the end of its input.
            Some(GroupEnd::Eof { at: Some(at) }) => {
                let at = at.value();
                return match (self.origin(node), self.snapshot()) {
                    (CssValueOrigin::Parsed(opening), Some(snapshot)) => {
                        CssParsedOrigin::from_range(snapshot, at..at)
                            .map(|at| CssValueOrigin::ImplicitClosure { opening, at })
                            .expect("actual child EOF")
                    }
                    _ => CssValueOrigin::Programmatic,
                };
            }
            _ => return CssValueOrigin::Programmatic,
        };
        let (Some(snapshot), Some(spelling)) = (
            self.snapshot(),
            self.closing_site(node).and_then(Site::spelling),
        ) else {
            return CssValueOrigin::Programmatic;
        };
        let (start, end) = (range.start.value(), range.end.value());
        if snapshot.as_str().get(start..end) == Some(spelling)
            && let Some(origin) = CssParsedOrigin::from_range(snapshot, start..end)
        {
            CssValueOrigin::Parsed(origin)
        } else {
            CssValueOrigin::Programmatic
        }
    }

    fn boundary(&self, list: ListId, index: usize) -> SyntaxBoundary {
        let offset = match self.list(list).get(index) {
            Some(node) => self.range(*node).map(|range| range.start.value()),
            None => self.list_end(list).map(Offset::value),
        };
        SyntaxBoundary {
            list,
            index,
            offset,
        }
    }

    fn cursor(&self, list: ListId) -> SyntaxCursor<'_> {
        SyntaxCursor {
            document: self,
            list,
            next: 0,
            end: self.list(list).len(),
        }
    }

    fn cursor_range(&self, range: &SyntaxRange) -> SyntaxCursor<'_> {
        assert!(range.start <= range.end && range.end <= self.list(range.list).len());
        SyntaxCursor {
            document: self,
            list: range.list,
            next: range.start,
            end: range.end,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CursorItem {
    Node(NodeId),
    EndOfInput(SyntaxBoundary),
}

#[derive(Clone)]
pub(crate) struct SyntaxCursor<'a> {
    pub(crate) document: &'a Document<'a>,
    pub(crate) list: ListId,
    next: usize,
    end: usize,
}
impl SyntaxCursor<'_> {
    pub(crate) fn position(&self) -> usize {
        self.next
    }
    pub(crate) fn peek(&self) -> CursorItem {
        self.document
            .list(self.list)
            .get(self.position())
            .filter(|_| self.position() < self.end)
            .copied()
            .map_or_else(
                || CursorItem::EndOfInput(self.document.boundary(self.list, self.end)),
                CursorItem::Node,
            )
    }
    pub(crate) fn consume(&mut self) -> CursorItem {
        let item = self.peek();
        if matches!(item, CursorItem::Node(_)) {
            self.next += 1;
        }
        item
    }
    pub(crate) fn skip_trivia(&mut self) {
        while let CursorItem::Node(node) = self.peek() {
            if !self.document.is_trivia(node) {
                break;
            }
            self.consume();
        }
    }
    pub(crate) fn boundary(&self) -> SyntaxBoundary {
        self.document.boundary(self.list, self.position())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SyntaxRange {
    pub(crate) list: ListId,
    pub(crate) start: usize,
    pub(crate) end: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GenericFaultKind {
    AtRuleEndOfInput,
    QualifiedRuleEndOfInput,
    /// A nested qualified rule reached its `<semicolon-token>` stop token,
    /// which remains unconsumed for the enclosing block's contents.
    QualifiedRuleStopToken,
    EmptyInput,
    TrailingInput,
    InvalidDeclaration,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GenericSyntaxFault {
    pub(crate) kind: GenericFaultKind,
    pub(crate) at: SyntaxBoundary,
    pub(crate) range: SyntaxRange,
}
#[derive(Clone, Debug)]
pub(crate) enum RuleTermination {
    Semicolon(NodeId),
    Block,
    EndOfInput,
}
#[derive(Clone, Debug)]
pub(crate) struct GenericAtRule {
    pub(crate) name: NodeId,
    pub(crate) prelude: Vec<NodeId>,
    pub(crate) block: Option<NodeId>,
    pub(crate) termination: RuleTermination,
    pub(crate) range: SyntaxRange,
    pub(crate) fault: Option<GenericSyntaxFault>,
}
#[derive(Clone, Debug)]
pub(crate) struct GenericQualifiedRule {
    pub(crate) prelude: Vec<NodeId>,
    pub(crate) block: NodeId,
    pub(crate) range: SyntaxRange,
}
#[derive(Clone, Debug)]
pub(crate) enum GenericRule {
    At(GenericAtRule),
    Qualified(GenericQualifiedRule),
}

pub(crate) fn custom_property_rule_prelude(document: &Document<'_>, prelude: &[NodeId]) -> bool {
    let mut meaningful = prelude.iter().filter(|node| !document.is_trivia(**node));
    matches!(meaningful.next().map(|node| document.kind(*node)), Some(TokenKind::Ident(name)) if name.starts_with("--"))
        && matches!(
            meaningful.next().map(|node| document.kind(*node)),
            Some(TokenKind::Colon)
        )
}
pub(crate) type GenericRuleResult = Result<GenericRule, GenericSyntaxFault>;

fn fault(cursor: &SyntaxCursor<'_>, start: usize, kind: GenericFaultKind) -> GenericSyntaxFault {
    GenericSyntaxFault {
        kind,
        at: cursor.boundary(),
        range: SyntaxRange {
            list: cursor.list,
            start,
            end: cursor.position(),
        },
    }
}

/// Selects exactly one raw component within this cursor's bounded list.
/// Lexical faults and implicit group endings remain owned by the document.
pub(crate) fn consume_one_component(
    cursor: &mut SyntaxCursor<'_>,
) -> Result<NodeId, GenericSyntaxFault> {
    cursor.skip_trivia();
    let start = cursor.position();
    let CursorItem::Node(node) = cursor.consume() else {
        return Err(fault(cursor, start, GenericFaultKind::EmptyInput));
    };
    cursor.skip_trivia();
    if matches!(cursor.peek(), CursorItem::Node(_)) {
        return Err(fault(cursor, start, GenericFaultKind::TrailingInput));
    }
    Ok(node)
}

/// Selects same-document root ranges separated by actual comma tokens.
/// Nested groups are single components; every empty range is retained.
pub(crate) fn consume_comma_separated_components(
    cursor: &mut SyntaxCursor<'_>,
) -> Vec<SyntaxRange> {
    let mut ranges = Vec::new();
    let mut start = cursor.position();
    loop {
        let before = cursor.position();
        match cursor.consume() {
            CursorItem::Node(node) if cursor.document.kind(node) == TokenKind::Comma => {
                ranges.push(SyntaxRange {
                    list: cursor.list,
                    start,
                    end: before,
                });
                start = cursor.position();
            }
            CursorItem::Node(_) => {}
            CursorItem::EndOfInput(_) => {
                ranges.push(SyntaxRange {
                    list: cursor.list,
                    start,
                    end: before,
                });
                return ranges;
            }
        }
    }
}

pub(crate) fn consume_at_rule(cursor: &mut SyntaxCursor<'_>) -> GenericAtRule {
    consume_at_rule_context(cursor, false)
}

pub(crate) fn consume_at_rule_context(
    cursor: &mut SyntaxCursor<'_>,
    nested: bool,
) -> GenericAtRule {
    let start = cursor.position();
    let CursorItem::Node(name) = cursor.consume() else {
        unreachable!("at-keyword dispatch")
    };
    debug_assert!(matches!(
        cursor.document.kind(name),
        TokenKind::AtKeyword(_)
    ));
    let mut prelude = Vec::new();
    let (termination, block, problem) = loop {
        if nested && block_contents_end(cursor) {
            break (
                RuleTermination::EndOfInput,
                None,
                Some(fault(cursor, start, GenericFaultKind::AtRuleEndOfInput)),
            );
        }
        match cursor.consume() {
            CursorItem::EndOfInput(_) => {
                break (
                    RuleTermination::EndOfInput,
                    None,
                    Some(fault(cursor, start, GenericFaultKind::AtRuleEndOfInput)),
                );
            }
            CursorItem::Node(node) => {
                if cursor.document.is_curly_block(node) {
                    break (RuleTermination::Block, Some(node), None);
                }
                if cursor.document.kind(node) == TokenKind::Semicolon
                    && semicolon_terminates_rule(rule_dispatch(cursor.document.kind(name)))
                {
                    break (RuleTermination::Semicolon(node), None, None);
                }
                prelude.push(node);
            }
        }
    };
    GenericAtRule {
        name,
        prelude,
        block,
        termination,
        range: SyntaxRange {
            list: cursor.list,
            start,
            end: cursor.position(),
        },
        fault: problem,
    }
}
pub(crate) fn consume_qualified_rule(
    cursor: &mut SyntaxCursor<'_>,
) -> Result<GenericQualifiedRule, GenericSyntaxFault> {
    consume_qualified_rule_context(cursor, false)
}

pub(crate) fn consume_qualified_rule_context(
    cursor: &mut SyntaxCursor<'_>,
    nested: bool,
) -> Result<GenericQualifiedRule, GenericSyntaxFault> {
    let start = cursor.position();
    let mut prelude = Vec::new();
    loop {
        if nested && block_contents_end(cursor) {
            return Err(fault(
                cursor,
                start,
                GenericFaultKind::QualifiedRuleEndOfInput,
            ));
        }
        if nested && next_is_semicolon(cursor) {
            return Err(fault(
                cursor,
                start,
                GenericFaultKind::QualifiedRuleStopToken,
            ));
        }
        match cursor.consume() {
            CursorItem::EndOfInput(_) => {
                return Err(fault(
                    cursor,
                    start,
                    GenericFaultKind::QualifiedRuleEndOfInput,
                ));
            }
            CursorItem::Node(node) => {
                if cursor.document.is_curly_block(node) {
                    return Ok(GenericQualifiedRule {
                        prelude,
                        block: node,
                        range: SyntaxRange {
                            list: cursor.list,
                            start,
                            end: cursor.position(),
                        },
                    });
                }
                prelude.push(node);
            }
        }
    }
}
fn next_is_semicolon(cursor: &SyntaxCursor<'_>) -> bool {
    matches!(cursor.peek(), CursorItem::Node(node) if cursor.document.kind(node) == TokenKind::Semicolon)
}
pub(crate) fn consume_one_rule(cursor: &mut SyntaxCursor<'_>) -> GenericRuleResult {
    cursor.skip_trivia();
    let start = cursor.position();
    let CursorItem::Node(node) = cursor.peek() else {
        return Err(fault(cursor, start, GenericFaultKind::EmptyInput));
    };
    let rule = if rule_dispatch(cursor.document.kind(node)) == GenericRuleDispatch::At {
        GenericRule::At(consume_at_rule(cursor))
    } else {
        GenericRule::Qualified(consume_qualified_rule(cursor)?)
    };
    cursor.skip_trivia();
    if matches!(cursor.peek(), CursorItem::Node(_)) {
        return Err(fault(cursor, start, GenericFaultKind::TrailingInput));
    }
    Ok(rule)
}

#[derive(Clone, Debug)]
pub(crate) struct GenericDeclaration {
    pub(crate) name: NodeId,
    pub(crate) value: Vec<NodeId>,
    pub(crate) important: bool,
    pub(crate) colon: usize,
    pub(crate) value_range: SyntaxRange,
    pub(crate) importance_range: Option<SyntaxRange>,
    pub(crate) range: SyntaxRange,
}
pub(crate) fn consume_declaration(
    cursor: &mut SyntaxCursor<'_>,
) -> Result<GenericDeclaration, GenericSyntaxFault> {
    let start = cursor.position();
    let CursorItem::Node(name) = cursor.consume() else {
        return Err(fault(cursor, start, GenericFaultKind::InvalidDeclaration));
    };
    if !matches!(cursor.document.kind(name), TokenKind::Ident(_)) {
        return Err(fault(cursor, start, GenericFaultKind::InvalidDeclaration));
    }
    cursor.skip_trivia();
    let colon = cursor.position();
    if !matches!(cursor.consume(), CursorItem::Node(node) if cursor.document.kind(node) == TokenKind::Colon)
    {
        return Err(fault(cursor, start, GenericFaultKind::InvalidDeclaration));
    }
    let value_start = cursor.position();
    cursor.skip_trivia();
    let mut value = Vec::new();
    while let CursorItem::Node(node) = cursor.consume() {
        value.push(node);
    }
    let mut important = false;
    let mut importance_range = None;
    let mut value_end = cursor.position();
    if let Some((bang, word)) = crate::parser::trailing_importance(
        &value,
        |node| cursor.document.is_trivia(*node),
        |node| cursor.document.kind(*node) == TokenKind::Delim('!'),
        |node| matches!(cursor.document.kind(*node), TokenKind::Ident(name) if name.eq_ignore_ascii_case("important")),
    ) {
        let bang_index = cursor.document.list(cursor.list)[value_start..cursor.position()]
            .iter()
            .position(|node| *node == value[bang])
            .expect("value node belongs to candidate")
            + value_start;
        value_end = bang_index;
        importance_range = Some(SyntaxRange {
            list: cursor.list,
            start: bang_index,
            end: cursor.position(),
        });
        value.remove(word);
        value.remove(bang);
        important = true;
    }
    while value_end > value_start
        && cursor
            .document
            .is_trivia(cursor.document.list(cursor.list)[value_end - 1])
    {
        value_end -= 1;
    }
    while value
        .last()
        .is_some_and(|node| cursor.document.is_trivia(*node))
    {
        value.pop();
    }
    Ok(GenericDeclaration {
        name,
        value,
        important,
        colon,
        value_range: SyntaxRange {
            list: cursor.list,
            start: value_start,
            end: value_end,
        },
        importance_range,
        range: SyntaxRange {
            list: cursor.list,
            start,
            end: cursor.position(),
        },
    })
}

// Syntax's declaration-list attempt is bounded by a root semicolon, including
// error nodes and complete nested groups. The delimiter remains outside it.
pub(crate) fn consume_declaration_candidate(cursor: &mut SyntaxCursor<'_>) -> SyntaxRange {
    let start = cursor.position();
    while let CursorItem::Node(node) = cursor.peek() {
        if cursor.document.kind(node) == TokenKind::Semicolon {
            break;
        }
        cursor.consume();
    }
    SyntaxRange {
        list: cursor.list,
        start,
        end: cursor.position(),
    }
}

// Block contents consumes nested units only up to a root closer (Syntax §5.5.5).
pub(crate) fn block_contents_end(cursor: &SyntaxCursor<'_>) -> bool {
    match cursor.peek() {
        CursorItem::EndOfInput(_) => true,
        CursorItem::Node(node) => {
            cursor.document.kind(node) == TokenKind::Closing(CssBlockKind::CurlyBracket)
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
pub(crate) mod independent_tests;

#[cfg(test)]
pub(crate) mod finer_recovery_independent_tests;

#[cfg(test)]
pub(crate) mod source_payload_tests;

#[cfg(test)]
pub(crate) mod declaration_boundary_tests;

#[cfg(test)]
pub(crate) mod raw_component_boundaries;
