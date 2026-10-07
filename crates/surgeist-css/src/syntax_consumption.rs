//! Private CSS Syntax 3 input normalization and generic structural consumption.
//!
//! Checked feature grammar remains a later admission phase. This arena retains
//! errors in input order, and accepts supplied items without serialization.

use std::{borrow::Cow, ops::Range};

use crate::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueRef, CssParsedOrigin, CssSourceSnapshot, CssValueOrigin, CssValueTokenRef,
};
use cssparser::Token;

pub(crate) type NodeId = usize;
pub(crate) type ListId = usize;

pub(crate) enum SyntaxInput<'a> {
    Source(SourceWindow<'a>),
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private supplied-token normalization contract, exercised by direct-input tests"
        )
    )]
    Tokens(&'a [SyntaxToken<'a>]),
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private error-bearing grouped-input normalization contract, exercised by direct-input tests"
        )
    )]
    Components {
        document: &'a SyntaxDocument<'a>,
        list: ListId,
    },
    CheckedComponents(&'a [CssComponentValue]),
}

#[derive(Clone)]
pub(crate) struct SourceWindow<'a> {
    pub(crate) text: Cow<'a, str>,
    pub(crate) range: Range<usize>,
    pub(crate) original: CssSourceSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedLexeme {
    Leaf,
    Opening,
    Closing,
}

#[derive(Clone, Debug)]
pub(crate) enum TokenPayload<'a> {
    Native(Token<'a>),
    Whitespace(Box<str>),
    Comment(Box<str>),
    Checked {
        component: &'a CssComponentValue,
        lexeme: CheckedLexeme,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct SyntaxToken<'a> {
    pub(crate) payload: TokenPayload<'a>,
    pub(crate) spelling: Option<Cow<'a, str>>,
    pub(crate) origin: Cow<'a, CssValueOrigin>,
}

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
    Colon,
    Cdo,
    Cdc,
    Delim(char),
    BadString,
    BadUrl,
    Other,
}

pub(crate) fn native_token_kind<'a>(token: &'a Token<'_>) -> TokenKind<'a> {
    match token {
        Token::Ident(name) => TokenKind::Ident(name),
        Token::AtKeyword(name) => TokenKind::AtKeyword(name),
        Token::Function(name) => TokenKind::Function(name),
        Token::ParenthesisBlock => TokenKind::Opening(CssBlockKind::Parenthesis),
        Token::SquareBracketBlock => TokenKind::Opening(CssBlockKind::SquareBracket),
        Token::CurlyBracketBlock => TokenKind::Opening(CssBlockKind::CurlyBracket),
        Token::CloseParenthesis => TokenKind::Closing(CssBlockKind::Parenthesis),
        Token::CloseSquareBracket => TokenKind::Closing(CssBlockKind::SquareBracket),
        Token::CloseCurlyBracket => TokenKind::Closing(CssBlockKind::CurlyBracket),
        Token::WhiteSpace(_) => TokenKind::Whitespace,
        Token::Comment(_) => TokenKind::Comment,
        Token::Semicolon => TokenKind::Semicolon,
        Token::Colon => TokenKind::Colon,
        Token::CDO => TokenKind::Cdo,
        Token::CDC => TokenKind::Cdc,
        Token::Delim(value) => TokenKind::Delim(*value),
        Token::BadString(_) => TokenKind::BadString,
        Token::BadUrl(_) => TokenKind::BadUrl,
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

pub(crate) fn ignored_rule_list_prefix(kind: TokenKind<'_>, top_level: bool) -> bool {
    matches!(kind, TokenKind::Whitespace | TokenKind::Comment)
        || top_level && matches!(kind, TokenKind::Cdo | TokenKind::Cdc)
}

pub(crate) fn semicolon_terminates_rule(dispatch: GenericRuleDispatch) -> bool {
    matches!(dispatch, GenericRuleDispatch::At)
}

impl SyntaxToken<'_> {
    pub(crate) fn kind(&self) -> TokenKind<'_> {
        match &self.payload {
            TokenPayload::Whitespace(_) => TokenKind::Whitespace,
            TokenPayload::Comment(_) => TokenKind::Comment,
            TokenPayload::Native(token) => native_token_kind(token),
            TokenPayload::Checked { component, lexeme } => match (lexeme, component.view()) {
                (CheckedLexeme::Opening, CssComponentValueRef::Function(value)) => {
                    TokenKind::Function(value.name())
                }
                (CheckedLexeme::Opening, CssComponentValueRef::Block(value)) => {
                    TokenKind::Opening(value.kind())
                }
                (CheckedLexeme::Closing, CssComponentValueRef::Function(_)) => {
                    TokenKind::Closing(CssBlockKind::Parenthesis)
                }
                (CheckedLexeme::Closing, CssComponentValueRef::Block(value)) => {
                    TokenKind::Closing(value.kind())
                }
                (CheckedLexeme::Leaf, CssComponentValueRef::Comment(_)) => TokenKind::Comment,
                (CheckedLexeme::Leaf, CssComponentValueRef::Token(value)) => match value {
                    CssValueTokenRef::Ident(name) => TokenKind::Ident(name),
                    CssValueTokenRef::AtKeyword(name) => TokenKind::AtKeyword(name),
                    CssValueTokenRef::Whitespace(_) => TokenKind::Whitespace,
                    CssValueTokenRef::Semicolon => TokenKind::Semicolon,
                    CssValueTokenRef::Colon => TokenKind::Colon,
                    CssValueTokenRef::Cdo => TokenKind::Cdo,
                    CssValueTokenRef::Cdc => TokenKind::Cdc,
                    CssValueTokenRef::Delim(value) => TokenKind::Delim(value),
                    _ => TokenKind::Other,
                },
                _ => unreachable!("private checked lexeme role matches its component"),
            },
        }
    }

    fn checked(component: &CssComponentValue, lexeme: CheckedLexeme) -> SyntaxToken<'_> {
        let (text, origin) = component
            .structural_lexeme(lexeme == CheckedLexeme::Closing)
            .expect("private checked lexeme exists");
        SyntaxToken {
            payload: TokenPayload::Checked { component, lexeme },
            spelling: Some(Cow::Borrowed(text)),
            origin: Cow::Borrowed(origin),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceSite {
    pub(crate) snapshot: CssSourceSnapshot,
    pub(crate) offset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SyntaxBoundary {
    pub(crate) list: ListId,
    pub(crate) index: usize,
    pub(crate) source: Option<SourceSite>,
}

#[derive(Clone, Debug)]
pub(crate) enum GroupEnd<'a> {
    Explicit(SyntaxToken<'a>),
    Implicit {
        opening: Cow<'a, CssValueOrigin>,
        at: SyntaxBoundary,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SyntaxTokenFault {
    BadString,
    BadUrl,
    UnexpectedCloser,
}

#[derive(Clone, Debug)]
pub(crate) enum SyntaxNode<'a> {
    Token(SyntaxToken<'a>),
    Comment(SyntaxToken<'a>),
    Error {
        token: SyntaxToken<'a>,
        cause: SyntaxTokenFault,
    },
    Function {
        opening: SyntaxToken<'a>,
        children: ListId,
        end: GroupEnd<'a>,
    },
    Block {
        kind: CssBlockKind,
        opening: SyntaxToken<'a>,
        children: ListId,
        end: GroupEnd<'a>,
    },
    // Source recovery only: the scanner denied this complete Curly component.
    // Its actual opening and document range remain available for unit selection,
    // but no child list or checked payload has been admitted.
    DeniedCurlyBlock {
        opening: SyntaxToken<'a>,
    },
}

impl<'a> SyntaxNode<'a> {
    pub(crate) fn token(&self) -> &SyntaxToken<'a> {
        match self {
            Self::Token(token) | Self::Comment(token) | Self::Error { token, .. } => token,
            Self::Function { opening, .. }
            | Self::Block { opening, .. }
            | Self::DeniedCurlyBlock { opening } => opening,
        }
    }
    pub(crate) fn children(&self) -> Option<ListId> {
        match self {
            Self::Function { children, .. } | Self::Block { children, .. } => Some(*children),
            _ => None,
        }
    }
    pub(crate) fn end(&self) -> Option<&GroupEnd<'a>> {
        match self {
            Self::Function { end, .. } | Self::Block { end, .. } => Some(end),
            _ => None,
        }
    }
    pub(crate) fn is_trivia(&self) -> bool {
        matches!(
            self.token().kind(),
            TokenKind::Whitespace | TokenKind::Comment
        )
    }
    fn is_curly_block(&self) -> bool {
        matches!(
            self,
            Self::Block {
                kind: CssBlockKind::CurlyBracket,
                ..
            } | Self::DeniedCurlyBlock { .. }
        )
    }
    pub(crate) fn denied_error(&self) -> Option<CssComponentValueError> {
        match self {
            Self::DeniedCurlyBlock { opening } => Some(CssComponentValueError::new(
                CssComponentValueErrorKind::NestingLimit,
                opening.origin.clone().into_owned(),
            )),
            _ => None,
        }
    }
}

pub(crate) struct SyntaxDocument<'a> {
    pub(crate) nodes: Vec<SyntaxNode<'a>>,
    pub(crate) lists: Vec<Vec<NodeId>>,
    ends: Vec<Option<SourceSite>>,
    pub(crate) root: ListId,
    pub(crate) metrics: SyntaxInputMetrics,
    pub(crate) source: Option<SourceWindow<'a>>,
    ranges: Vec<Option<Range<usize>>>,
}

impl SyntaxDocument<'_> {
    pub(crate) fn cursor(&self, list: ListId) -> SyntaxCursor<'_> {
        SyntaxCursor {
            document: self,
            list,
            next: 0,
            end: self.lists[list].len(),
            current: None,
            reconsume: false,
        }
    }
    pub(crate) fn cursor_range(&self, range: &SyntaxRange) -> SyntaxCursor<'_> {
        assert!(range.start <= range.end && range.end <= self.lists[range.list].len());
        SyntaxCursor {
            document: self,
            list: range.list,
            next: range.start,
            end: range.end,
            current: None,
            reconsume: false,
        }
    }
    pub(crate) fn range(&self, node: NodeId) -> Option<Range<usize>> {
        self.ranges[node].clone()
    }
    pub(crate) fn boundary(&self, list: ListId, index: usize) -> SyntaxBoundary {
        let source = if let Some(node) = self.lists[list].get(index) {
            self.range(*node).and_then(|range| {
                self.source.as_ref().map(|window| SourceSite {
                    snapshot: window.original.clone(),
                    offset: range.start,
                })
            })
        } else {
            self.ends[list].clone()
        };
        SyntaxBoundary {
            list,
            index,
            source,
        }
    }
    pub(crate) fn list_at_source_start(&self, offset: usize) -> Option<ListId> {
        if self
            .source
            .as_ref()
            .is_some_and(|source| source.range.start == offset)
        {
            return Some(self.root);
        }
        self.nodes.iter().enumerate().find_map(|(id, node)| {
            let children = node.children()?;
            let range = self.range(id)?;
            (range.start + node.token().spelling.as_ref()?.len() == offset).then_some(children)
        })
    }
    pub(crate) fn cursor_at_source(&self, offset: usize) -> Option<SyntaxCursor<'_>> {
        self.lists.iter().enumerate().find_map(|(list, nodes)| {
            let next = nodes
                .iter()
                .position(|node| self.range(*node).is_some_and(|range| range.start == offset))?;
            Some(SyntaxCursor {
                document: self,
                list,
                next,
                end: nodes.len(),
                current: None,
                reconsume: false,
            })
        })
    }
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SyntaxInputMetrics {
    pub(crate) components: usize,
    pub(crate) maximum_depth: u32,
    pub(crate) known_spelling_bytes: usize,
    pub(crate) unspelled_tokens: usize,
}

struct Builder<'a> {
    document: SyntaxDocument<'a>,
    limits: SyntaxInputLimits,
    base_depth: u32,
}

impl<'a> Builder<'a> {
    fn new(limits: SyntaxInputLimits, base_depth: u32) -> Self {
        Self {
            document: SyntaxDocument {
                nodes: Vec::new(),
                lists: vec![Vec::new()],
                ends: vec![None],
                root: 0,
                metrics: SyntaxInputMetrics::default(),
                source: None,
                ranges: Vec::new(),
            },
            limits,
            base_depth,
        }
    }
    fn charge(
        &mut self,
        token: &SyntaxToken<'_>,
        component: bool,
        depth: u32,
    ) -> Result<(), CssComponentValueError> {
        let error = |kind| CssComponentValueError::new(kind, token.origin.clone().into_owned());
        let metrics = &mut self.document.metrics;
        if component {
            metrics.components = metrics
                .components
                .checked_add(1)
                .ok_or_else(|| error(CssComponentValueErrorKind::CapacityOverflow))?;
            if metrics.components > self.limits.max_components {
                return Err(error(CssComponentValueErrorKind::ComponentLimit));
            }
        }
        let depth = self
            .base_depth
            .checked_add(depth)
            .ok_or_else(|| error(CssComponentValueErrorKind::CapacityOverflow))?;
        if depth > self.limits.max_depth.min(256) {
            return Err(error(CssComponentValueErrorKind::NestingLimit));
        }
        metrics.maximum_depth = metrics.maximum_depth.max(depth);
        if !matches!(
            token.origin.as_ref(),
            CssValueOrigin::ImplicitClosure { .. }
        ) {
            if let Some(spelling) = &token.spelling {
                return self.charge_known_spelling(token, spelling.len());
            } else {
                metrics.unspelled_tokens = metrics
                    .unspelled_tokens
                    .checked_add(1)
                    .ok_or_else(|| error(CssComponentValueErrorKind::CapacityOverflow))?;
            }
        }
        Ok(())
    }
    fn charge_known_spelling(
        &mut self,
        token: &SyntaxToken<'_>,
        bytes: usize,
    ) -> Result<(), CssComponentValueError> {
        let error = |kind| CssComponentValueError::new(kind, token.origin.clone().into_owned());
        let metrics = &mut self.document.metrics;
        metrics.known_spelling_bytes = metrics
            .known_spelling_bytes
            .checked_add(bytes)
            .ok_or_else(|| error(CssComponentValueErrorKind::CapacityOverflow))?;
        if metrics.known_spelling_bytes > self.limits.max_known_spelling_bytes {
            return Err(error(CssComponentValueErrorKind::ByteLimit));
        }
        Ok(())
    }
    fn list(&mut self, end: Option<SourceSite>) -> Result<ListId, CssComponentValueError> {
        self.document.lists.try_reserve(1).map_err(|_| capacity())?;
        self.document.ends.try_reserve(1).map_err(|_| capacity())?;
        let id = self.document.lists.len();
        self.document.lists.push(Vec::new());
        self.document.ends.push(end);
        Ok(id)
    }
    fn push(
        &mut self,
        list: ListId,
        node: SyntaxNode<'a>,
        range: Option<Range<usize>>,
    ) -> Result<NodeId, CssComponentValueError> {
        self.document.nodes.try_reserve(1).map_err(|_| capacity())?;
        self.document
            .ranges
            .try_reserve(1)
            .map_err(|_| capacity())?;
        self.document.lists[list]
            .try_reserve(1)
            .map_err(|_| capacity())?;
        let id = self.document.nodes.len();
        self.document.nodes.push(node);
        self.document.ranges.push(range);
        self.document.lists[list].push(id);
        Ok(id)
    }
    fn leaf(
        &mut self,
        list: ListId,
        token: SyntaxToken<'a>,
        range: Option<Range<usize>>,
        depth: u32,
    ) -> Result<(), CssComponentValueError> {
        self.charge(&token, true, depth)?;
        let node = match token.kind() {
            TokenKind::BadString => SyntaxNode::Error {
                token,
                cause: SyntaxTokenFault::BadString,
            },
            TokenKind::BadUrl => SyntaxNode::Error {
                token,
                cause: SyntaxTokenFault::BadUrl,
            },
            TokenKind::Closing(_) => SyntaxNode::Error {
                token,
                cause: SyntaxTokenFault::UnexpectedCloser,
            },
            TokenKind::Comment => SyntaxNode::Comment(token),
            _ => SyntaxNode::Token(token),
        };
        self.push(list, node, range)?;
        Ok(())
    }
    fn opening(
        &mut self,
        parent: ListId,
        token: SyntaxToken<'a>,
        range: Option<Range<usize>>,
        depth: u32,
        end: Option<SourceSite>,
    ) -> Result<(NodeId, ListId), CssComponentValueError> {
        self.charge(&token, true, depth)?;
        let children = self.list(end)?;
        let implicit = GroupEnd::Implicit {
            opening: token.origin.clone(),
            at: SyntaxBoundary {
                list: children,
                index: 0,
                source: None,
            },
        };
        let node = match token.kind() {
            TokenKind::Function(_) => SyntaxNode::Function {
                opening: token,
                children,
                end: implicit,
            },
            TokenKind::Opening(kind) => SyntaxNode::Block {
                kind,
                opening: token,
                children,
                end: implicit,
            },
            _ => unreachable!("private opening dispatch"),
        };
        let id = self.push(parent, node, range)?;
        Ok((id, children))
    }
    fn close(&mut self, node: NodeId, end: GroupEnd<'a>, source_end: Option<usize>) {
        match &mut self.document.nodes[node] {
            SyntaxNode::Function { end: actual, .. } | SyntaxNode::Block { end: actual, .. } => {
                *actual = end
            }
            _ => unreachable!("only groups have an end"),
        }
        if let (Some(range), Some(end)) = (&mut self.document.ranges[node], source_end) {
            range.end = end;
        }
    }
}

fn capacity() -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::CapacityOverflow,
        CssValueOrigin::Programmatic,
    )
}

// Own provider string payloads without retokenization. Provider whitespace and
// comments alone use borrowed &str rather than CowRcStr, so retain those two
// payloads in explicitly owned private variants.
fn own_payload(token: Token<'_>) -> TokenPayload<'static> {
    let token = match token {
        Token::Ident(value) => Token::Ident(value.as_ref().to_owned().into()),
        Token::AtKeyword(value) => Token::AtKeyword(value.as_ref().to_owned().into()),
        Token::Hash(value) => Token::Hash(value.as_ref().to_owned().into()),
        Token::IDHash(value) => Token::IDHash(value.as_ref().to_owned().into()),
        Token::QuotedString(value) => Token::QuotedString(value.as_ref().to_owned().into()),
        Token::UnquotedUrl(value) => Token::UnquotedUrl(value.as_ref().to_owned().into()),
        Token::Function(value) => Token::Function(value.as_ref().to_owned().into()),
        Token::BadUrl(value) => Token::BadUrl(value.as_ref().to_owned().into()),
        Token::BadString(value) => Token::BadString(value.as_ref().to_owned().into()),
        Token::Dimension {
            has_sign,
            value,
            int_value,
            unit,
        } => Token::Dimension {
            has_sign,
            value,
            int_value,
            unit: unit.as_ref().to_owned().into(),
        },
        Token::WhiteSpace(value) => return TokenPayload::Whitespace(value.into()),
        Token::Comment(value) => return TokenPayload::Comment(value.into()),
        Token::Delim(value) => Token::Delim(value),
        Token::Number {
            has_sign,
            value,
            int_value,
        } => Token::Number {
            has_sign,
            value,
            int_value,
        },
        Token::Percentage {
            has_sign,
            unit_value,
            int_value,
        } => Token::Percentage {
            has_sign,
            unit_value,
            int_value,
        },
        Token::Colon => Token::Colon,
        Token::Semicolon => Token::Semicolon,
        Token::Comma => Token::Comma,
        Token::IncludeMatch => Token::IncludeMatch,
        Token::DashMatch => Token::DashMatch,
        Token::PrefixMatch => Token::PrefixMatch,
        Token::SuffixMatch => Token::SuffixMatch,
        Token::SubstringMatch => Token::SubstringMatch,
        Token::CDO => Token::CDO,
        Token::CDC => Token::CDC,
        Token::ParenthesisBlock => Token::ParenthesisBlock,
        Token::SquareBracketBlock => Token::SquareBracketBlock,
        Token::CurlyBracketBlock => Token::CurlyBracketBlock,
        Token::CloseParenthesis => Token::CloseParenthesis,
        Token::CloseSquareBracket => Token::CloseSquareBracket,
        Token::CloseCurlyBracket => Token::CloseCurlyBracket,
    };
    TokenPayload::Native(token)
}

pub(crate) fn source_document(
    text: &str,
    original: &CssSourceSnapshot,
    base_depth: u32,
) -> Result<SyntaxDocument<'static>, CssComponentValueError> {
    normalize(
        SyntaxInput::Source(SourceWindow {
            text: Cow::Owned(text.to_owned()),
            range: 0..text.len(),
            original: original.clone(),
        }),
        SyntaxInputLimits::default(),
        base_depth,
    )
}

// Only the native source resource owner supplies these scanner-proved ranges.
// Direct Source/Tokens/Components/CheckedComponents admission remains strict.
pub(crate) fn recovery_source_document(
    text: &str,
    original: &CssSourceSnapshot,
    base_depth: u32,
    denied_curly: &[Range<usize>],
) -> Result<SyntaxDocument<'static>, CssComponentValueError> {
    if denied_curly.is_empty() {
        return source_document(text, original, base_depth);
    }
    let mut builder = Builder::new(SyntaxInputLimits::default(), base_depth);
    normalize_source(
        &mut builder,
        SourceWindow {
            text: Cow::Owned(text.to_owned()),
            range: 0..text.len(),
            original: original.clone(),
        },
        denied_curly,
    )?;
    Ok(builder.document)
}

fn normalize_source<'a>(
    builder: &mut Builder<'a>,
    window: SourceWindow<'a>,
    denied_curly: &[Range<usize>],
) -> Result<(), CssComponentValueError> {
    let text = window
        .text
        .get(..window.range.end)
        .expect("bounded source range");
    if window.range.len() > builder.limits.max_known_spelling_bytes {
        return Err(CssComponentValueError::new(
            CssComponentValueErrorKind::ByteLimit,
            CssValueOrigin::UnretainedInput {
                byte_length: window.range.len(),
            },
        ));
    }
    builder.document.ends[0] = Some(SourceSite {
        snapshot: window.original.clone(),
        offset: window.range.end,
    });
    builder.document.source = Some(window.clone());
    let mut frames = Vec::new();
    let mut denied = denied_curly.iter().peekable();
    let mut offset = window.range.start;
    while offset < window.range.end {
        let (_, end, token) = crate::tokenization::next_source_token(text, offset)
            .expect("nonempty token-boundary source window");
        if let Some(range) = denied.peek()
            && range.start == offset
        {
            assert!(
                matches!(token, Token::CurlyBracketBlock),
                "scanner denied a real Curly opening"
            );
            let opening = SyntaxToken {
                payload: own_payload(token),
                spelling: Some(Cow::Owned(text[offset..end].to_owned())),
                origin: Cow::Owned(CssValueOrigin::Parsed(
                    CssParsedOrigin::from_range(&window.original, offset..end)
                        .expect("actual denied opening"),
                )),
            };
            // The denial is one boundary at its admitted parent depth. Its
            // complete known source spelling is charged once without tokenizing
            // or admitting any child, including the depth-257 opening.
            builder.charge(&opening, true, frames.len() as u32)?;
            builder.charge_known_spelling(&opening, range.len() - (end - offset))?;
            let parent = frames.last().map_or(0, |frame: &RawFrame| frame.list);
            builder.push(
                parent,
                SyntaxNode::DeniedCurlyBlock { opening },
                Some((*range).clone()),
            )?;
            offset = range.end;
            denied.next();
            continue;
        }
        for (token, range) in crate::tokenization::source_token_parts(text, token, offset..end)
            .into_iter()
            .flatten()
        {
            let origin = if window.original.as_str().get(range.clone()) == text.get(range.clone()) {
                CssValueOrigin::Parsed(
                    CssParsedOrigin::from_range(&window.original, range.clone())
                        .expect("original token range"),
                )
            } else {
                CssValueOrigin::Programmatic
            };
            raw_token(
                builder,
                &mut frames,
                SyntaxToken {
                    payload: own_payload(token),
                    spelling: Some(Cow::Owned(text[range.clone()].to_owned())),
                    origin: Cow::Owned(origin),
                },
                Some(range),
            )?;
        }
        offset = end;
    }
    assert!(
        denied.next().is_none(),
        "all scanner-proved denied boundaries were consumed"
    );
    finish_raw(builder, frames);
    Ok(())
}

struct RawFrame {
    node: NodeId,
    list: ListId,
    kind: CssBlockKind,
}

pub(crate) fn normalize<'a>(
    input: SyntaxInput<'a>,
    limits: SyntaxInputLimits,
    base_depth: u32,
) -> Result<SyntaxDocument<'a>, CssComponentValueError> {
    let mut builder = Builder::new(limits, base_depth);
    match input {
        SyntaxInput::Source(window) => {
            normalize_source(&mut builder, window, &[])?;
        }
        SyntaxInput::Tokens(tokens) => {
            let mut frames = Vec::new();
            for token in tokens {
                raw_token(&mut builder, &mut frames, token.clone(), None)?;
            }
            finish_raw(&mut builder, frames);
        }
        SyntaxInput::CheckedComponents(values) => {
            enum Work<'a> {
                Value(&'a CssComponentValue, ListId, u32),
                Close(NodeId, &'a CssComponentValue, u32),
            }
            let mut pending = Vec::new();
            for value in values.iter().rev() {
                pending.push(Work::Value(value, 0, 0));
            }
            while let Some(work) = pending.pop() {
                match work {
                    Work::Value(value, list, depth) => {
                        let children = match value.view() {
                            CssComponentValueRef::Function(value) => Some(value.values().items()),
                            CssComponentValueRef::Block(value) => Some(value.values().items()),
                            _ => None,
                        };
                        if let Some(children) = children {
                            let opening = SyntaxToken::checked(value, CheckedLexeme::Opening);
                            let (node, child) =
                                builder.opening(list, opening, None, depth + 1, None)?;
                            pending.push(Work::Close(node, value, depth + 1));
                            for value in children.iter().rev() {
                                pending.push(Work::Value(value, child, depth + 1));
                            }
                        } else {
                            builder.leaf(
                                list,
                                SyntaxToken::checked(value, CheckedLexeme::Leaf),
                                None,
                                depth,
                            )?;
                        }
                    }
                    Work::Close(node, value, depth) => {
                        let closing = SyntaxToken::checked(value, CheckedLexeme::Closing);
                        let child = builder.document.nodes[node]
                            .children()
                            .expect("group children");
                        let end = if let CssValueOrigin::ImplicitClosure { at, .. } =
                            closing.origin.as_ref()
                        {
                            builder.document.ends[child] = Some(SourceSite {
                                snapshot: at.source().clone(),
                                offset: at.span().start().byte_offset().value(),
                            });
                            GroupEnd::Implicit {
                                opening: Cow::Borrowed(value.origin()),
                                at: builder
                                    .document
                                    .boundary(child, builder.document.lists[child].len()),
                            }
                        } else {
                            builder.charge(&closing, false, depth)?;
                            if let CssValueOrigin::Parsed(origin) = closing.origin.as_ref() {
                                builder.document.ends[child] = Some(SourceSite {
                                    snapshot: origin.source().clone(),
                                    offset: origin.span().start().byte_offset().value(),
                                });
                            }
                            GroupEnd::Explicit(closing)
                        };
                        builder.close(node, end, None);
                    }
                }
            }
        }
        SyntaxInput::Components { document, list } => {
            enum Work {
                Node(NodeId, ListId, u32),
                Close(NodeId, NodeId, u32),
            }
            let mut pending = Vec::new();
            for node in document.lists[list].iter().rev() {
                pending.push(Work::Node(*node, 0, 0));
            }
            // Root EOF belongs to this supplied list only if it was really supplied.
            builder.document.ends[0] = document.ends[list].clone();
            while let Some(work) = pending.pop() {
                match work {
                    Work::Node(old, list, depth) => {
                        let value = &document.nodes[old];
                        if let Some(error) = value.denied_error() {
                            return Err(error);
                        }
                        if let Some(children) = value.children() {
                            let (new, child) = builder.opening(
                                list,
                                value.token().clone(),
                                None,
                                depth + 1,
                                document.ends[children].clone(),
                            )?;
                            pending.push(Work::Close(new, old, depth + 1));
                            for node in document.lists[children].iter().rev() {
                                pending.push(Work::Node(*node, child, depth + 1));
                            }
                        } else {
                            builder.leaf(list, value.token().clone(), None, depth)?;
                        }
                    }
                    Work::Close(new, old, depth) => {
                        let end = document.nodes[old].end().expect("group end").clone();
                        if let GroupEnd::Explicit(token) = &end {
                            builder.charge(token, false, depth)?;
                        }
                        let end = match end {
                            GroupEnd::Implicit { opening, .. } => {
                                let child = builder.document.nodes[new]
                                    .children()
                                    .expect("group children");
                                GroupEnd::Implicit {
                                    opening,
                                    at: builder
                                        .document
                                        .boundary(child, builder.document.lists[child].len()),
                                }
                            }
                            end => end,
                        };
                        builder.close(new, end, None);
                    }
                }
            }
        }
    }
    Ok(builder.document)
}

fn raw_token<'a>(
    builder: &mut Builder<'a>,
    frames: &mut Vec<RawFrame>,
    token: SyntaxToken<'a>,
    range: Option<Range<usize>>,
) -> Result<(), CssComponentValueError> {
    let kind = token.kind();
    if let TokenKind::Closing(kind) = kind
        && let Some(frame) = frames.pop_if(|frame| frame.kind == kind)
    {
        builder.charge(&token, false, frames.len() as u32 + 1)?;
        builder.document.ends[frame.list] = range.as_ref().and_then(|range| {
            builder.document.source.as_ref().map(|window| SourceSite {
                snapshot: window.original.clone(),
                offset: range.start,
            })
        });
        let end = range.map(|range| range.end);
        builder.close(frame.node, GroupEnd::Explicit(token), end);
        return Ok(());
    }
    let parent = frames.last().map_or(0, |frame| frame.list);
    let block = match kind {
        TokenKind::Function(_) => Some(CssBlockKind::Parenthesis),
        TokenKind::Opening(kind) => Some(kind),
        _ => None,
    };
    if let Some(kind) = block {
        let (node, list) = builder.opening(parent, token, range, frames.len() as u32 + 1, None)?;
        frames.try_reserve(1).map_err(|_| capacity())?;
        frames.push(RawFrame { node, list, kind });
    } else {
        builder.leaf(parent, token, range, frames.len() as u32)?;
    }
    Ok(())
}

fn finish_raw(builder: &mut Builder<'_>, mut frames: Vec<RawFrame>) {
    while let Some(frame) = frames.pop() {
        let source = builder.document.ends[0].clone();
        builder.document.ends[frame.list] = source.clone();
        let at = builder
            .document
            .boundary(frame.list, builder.document.lists[frame.list].len());
        let opening = builder.document.nodes[frame.node].token().origin.clone();
        builder.close(
            frame.node,
            GroupEnd::Implicit { opening, at },
            source.map(|source| source.offset),
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CursorItem {
    Node(NodeId),
    EndOfInput(SyntaxBoundary),
}

#[derive(Clone)]
pub(crate) struct SyntaxCursor<'a> {
    pub(crate) document: &'a SyntaxDocument<'a>,
    pub(crate) list: ListId,
    next: usize,
    end: usize,
    current: Option<usize>,
    reconsume: bool,
}
impl SyntaxCursor<'_> {
    pub(crate) fn position(&self) -> usize {
        if self.reconsume {
            self.current.unwrap_or(self.next)
        } else {
            self.next
        }
    }
    pub(crate) fn peek(&self) -> CursorItem {
        self.document.lists[self.list]
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
            if self.reconsume {
                self.reconsume = false;
            } else {
                self.current = Some(self.next);
                self.next += 1;
            }
        } else {
            self.current = Some(self.next);
            self.reconsume = false;
        }
        item
    }
    pub(crate) fn reconsume_current(&mut self) {
        if self.current.is_some() {
            self.reconsume = true;
        }
    }
    pub(crate) fn skip_trivia(&mut self) {
        while let CursorItem::Node(node) = self.peek() {
            if !self.document.nodes[node].is_trivia() {
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
    EndOfInput(SyntaxBoundary),
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
pub(crate) fn consume_at_rule(cursor: &mut SyntaxCursor<'_>) -> GenericAtRule {
    let start = cursor.position();
    let CursorItem::Node(name) = cursor.consume() else {
        unreachable!("at-keyword dispatch")
    };
    debug_assert!(matches!(
        cursor.document.nodes[name].token().kind(),
        TokenKind::AtKeyword(_)
    ));
    let mut prelude = Vec::new();
    let (termination, block, problem) = loop {
        match cursor.consume() {
            CursorItem::EndOfInput(at) => {
                break (
                    RuleTermination::EndOfInput(at),
                    None,
                    Some(fault(cursor, start, GenericFaultKind::AtRuleEndOfInput)),
                );
            }
            CursorItem::Node(node) => match &cursor.document.nodes[node] {
                value if value.is_curly_block() => {
                    break (RuleTermination::Block, Some(node), None);
                }
                value
                    if value.token().kind() == TokenKind::Semicolon
                        && semicolon_terminates_rule(rule_dispatch(
                            cursor.document.nodes[name].token().kind(),
                        )) =>
                {
                    break (RuleTermination::Semicolon(node), None, None);
                }
                _ => prelude.push(node),
            },
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
    let start = cursor.position();
    let mut prelude = Vec::new();
    loop {
        match cursor.consume() {
            CursorItem::EndOfInput(_) => {
                return Err(fault(
                    cursor,
                    start,
                    GenericFaultKind::QualifiedRuleEndOfInput,
                ));
            }
            CursorItem::Node(node) => {
                if cursor.document.nodes[node].is_curly_block() {
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
pub(crate) fn consume_rules(
    cursor: &mut SyntaxCursor<'_>,
    top_level: bool,
) -> Vec<GenericRuleResult> {
    let mut rules = Vec::new();
    loop {
        cursor.skip_trivia();
        let CursorItem::Node(node) = cursor.consume() else {
            break;
        };
        let kind = cursor.document.nodes[node].token().kind();
        if ignored_rule_list_prefix(kind, top_level) {
            continue;
        }
        cursor.reconsume_current();
        match rule_dispatch(kind) {
            GenericRuleDispatch::At => rules.push(Ok(GenericRule::At(consume_at_rule(cursor)))),
            GenericRuleDispatch::Qualified => {
                rules.push(consume_qualified_rule(cursor).map(GenericRule::Qualified))
            }
        }
    }
    rules
}
pub(crate) fn consume_one_rule(cursor: &mut SyntaxCursor<'_>) -> GenericRuleResult {
    cursor.skip_trivia();
    let start = cursor.position();
    let CursorItem::Node(node) = cursor.peek() else {
        return Err(fault(cursor, start, GenericFaultKind::EmptyInput));
    };
    let rule =
        if rule_dispatch(cursor.document.nodes[node].token().kind()) == GenericRuleDispatch::At {
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
    if !matches!(
        cursor.document.nodes[name].token().kind(),
        TokenKind::Ident(_)
    ) {
        return Err(fault(cursor, start, GenericFaultKind::InvalidDeclaration));
    }
    cursor.skip_trivia();
    let colon = cursor.position();
    if !matches!(cursor.consume(), CursorItem::Node(node) if cursor.document.nodes[node].token().kind() == TokenKind::Colon)
    {
        return Err(fault(cursor, start, GenericFaultKind::InvalidDeclaration));
    }
    let value_start = cursor.position();
    cursor.skip_trivia();
    let mut value = Vec::new();
    while let CursorItem::Node(node) = cursor.consume() {
        value.push(node);
    }
    let meaningful: Vec<_> = value
        .iter()
        .enumerate()
        .filter(|(_, node)| !cursor.document.nodes[**node].is_trivia())
        .map(|(index, _)| index)
        .collect();
    let mut important = false;
    let mut importance_range = None;
    let mut value_end = cursor.position();
    if let [.., bang, word] = meaningful.as_slice()
        && cursor.document.nodes[value[*bang]].token().kind() == TokenKind::Delim('!')
        && matches!(cursor.document.nodes[value[*word]].token().kind(), TokenKind::Ident(name) if name.eq_ignore_ascii_case("important"))
    {
        let bang_index = cursor.document.lists[cursor.list][value_start..cursor.position()]
            .iter()
            .position(|node| *node == value[*bang])
            .expect("value node belongs to candidate")
            + value_start;
        value_end = bang_index;
        importance_range = Some(SyntaxRange {
            list: cursor.list,
            start: bang_index,
            end: cursor.position(),
        });
        value.remove(*word);
        value.remove(*bang);
        important = true;
    }
    while value_end > value_start
        && cursor.document.nodes[cursor.document.lists[cursor.list][value_end - 1]].is_trivia()
    {
        value_end -= 1;
    }
    while value
        .last()
        .is_some_and(|node| cursor.document.nodes[*node].is_trivia())
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
        if cursor.document.nodes[node].token().kind() == TokenKind::Semicolon {
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

pub(crate) enum GenericDeclarationListItem {
    At(GenericAtRule),
    Declaration {
        range: SyntaxRange,
        parsed: Result<GenericDeclaration, GenericSyntaxFault>,
    },
}

pub(crate) fn consume_declaration_list(
    cursor: &mut SyntaxCursor<'_>,
) -> Vec<GenericDeclarationListItem> {
    let mut items = Vec::new();
    loop {
        cursor.skip_trivia();
        let CursorItem::Node(node) = cursor.peek() else {
            break;
        };
        if cursor.document.nodes[node].token().kind() == TokenKind::Semicolon {
            cursor.consume();
        } else if rule_dispatch(cursor.document.nodes[node].token().kind())
            == GenericRuleDispatch::At
        {
            items.push(GenericDeclarationListItem::At(consume_at_rule(cursor)));
        } else {
            let range = consume_declaration_candidate(cursor);
            let parsed = consume_declaration(&mut cursor.document.cursor_range(&range));
            items.push(GenericDeclarationListItem::Declaration { range, parsed });
        }
    }
    items
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod independent_tests;

#[cfg(test)]
mod finer_recovery_independent_tests;

#[cfg(test)]
mod source_payload_tests;

#[cfg(test)]
mod declaration_boundary_tests;
