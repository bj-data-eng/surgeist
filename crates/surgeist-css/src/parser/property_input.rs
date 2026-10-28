//! One bounded property grammar cursor over source and checked syntax arenas.
use crate::component::values::CssComponentValueErrorKind;
use crate::component::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueLimits,
    CssComponentValueRef, CssHashFlag, CssNumericTokenKind, CssNumericTokenRef,
    CssSerializedOrigin, CssValueOrigin, CssValueTokenRef,
};
use crate::diagnostic::error::property::{StreamError, SyntaxError};
use crate::source::CssSourcePosition;
use crate::syntax_consumption::{
    CheckedLexeme, GroupEnd, ListId, SyntaxDocument, SyntaxNode, SyntaxToken, TokenKind,
    TokenPayload,
};
use crate::values::color_adjust::utf16_value::Control;
use cssparser::{CowRcStr, Token};
use std::{borrow::Cow, ops::Range};

/// A real component boundary; never a byte offset in serialized checked input.
#[derive(Clone, Copy)]
pub(crate) struct Site<'i> {
    document: &'i SyntaxDocument<'i>,
    list: ListId,
    index: usize,
    closing: bool,
}
impl std::fmt::Debug for Site<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Site")
            .field("list", &self.list)
            .field("index", &self.index)
            .field("closing", &self.closing)
            .finish()
    }
}
impl PartialEq for Site<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.document, other.document)
            && self.list == other.list
            && self.index == other.index
            && self.closing == other.closing
    }
}
impl Eq for Site<'_> {}
impl<'i> Site<'i> {
    pub(crate) fn control(self) -> Option<&'i Control<'i>> {
        self.document.control
    }
    pub(crate) fn checked_component(self) -> Option<(&'i CssComponentValue, bool)> {
        match &self.node()?.token().payload {
            TokenPayload::Checked { component, .. } => Some((*component, self.closing)),
            _ => None,
        }
    }

    fn node(self) -> Option<&'i SyntaxNode<'i>> {
        self.document.lists[self.list]
            .get(self.index)
            .map(|id| &self.document.nodes[*id])
    }
    fn syntax_token(self) -> Option<&'i SyntaxToken<'i>> {
        let node = self.node()?;
        if self.closing {
            match node.end()? {
                GroupEnd::Explicit(token) => Some(token),
                _ => None,
            }
        } else {
            Some(node.token())
        }
    }
    pub(crate) fn token(self) -> Option<Token<'i>> {
        self.syntax_token().map(native_token)
    }
    pub(crate) fn ordinal(self) -> usize {
        self.index
    }
    pub(crate) fn value_origin(self) -> Option<CssValueOrigin> {
        match self.origin() {
            CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => {
                Some(origin)
            }
            _ => None,
        }
    }
    pub(crate) fn descendant(self, path: &[usize], closing: bool) -> Option<Self> {
        let mut site = self;
        for (depth, index) in path.iter().copied().enumerate() {
            if self
                .document
                .control
                .is_some_and(|control| control.step().is_err())
            {
                return None;
            }
            if depth == 0 {
                site.index = site.index.checked_add(index)?;
            } else {
                site.list = site.node()?.children()?;
                site.index = index;
            }
            site.node()?;
        }
        site.closing = closing;
        if closing {
            site.node()?.end()?;
        }
        Some(site)
    }
    /// Whether an operator follows this component in its list, so that it
    /// starts a multi-term calculation.
    pub(crate) fn begins_expression(self) -> bool {
        let nodes = &self.document.lists[self.list];
        let Some(index) = (self.index + 1..nodes.len())
            .find(|index| !self.document.nodes[nodes[*index]].is_trivia())
        else {
            return false;
        };
        matches!(
            Self {
                index,
                closing: false,
                ..self
            }
            .token(),
            Some(Token::Delim('+' | '-' | '*' | '/'))
        )
    }
    /// The end of this component's list when it is an operator that follows
    /// an operand and that no further component follows: the operand it
    /// requires is missing there. An operator that starts its argument is
    /// itself responsible.
    pub(crate) fn trailing_operator_end(self) -> Option<Self> {
        if !matches!(self.token(), Some(Token::Delim('+' | '-' | '*' | '/'))) {
            return None;
        }
        let nodes = &self.document.lists[self.list];
        let operand = (0..self.index)
            .rev()
            .find(|index| !self.document.nodes[nodes[*index]].is_trivia())
            .is_some_and(|index| {
                !matches!(
                    Self {
                        index,
                        closing: false,
                        ..self
                    }
                    .token(),
                    Some(Token::Comma)
                )
            });
        if !operand {
            return None;
        }
        (self.index + 1..nodes.len())
            .all(|index| self.document.nodes[nodes[index]].is_trivia())
            .then_some(Self {
                index: nodes.len(),
                closing: false,
                ..self
            })
    }
    /// The end of the group at `path`'s children, before its closing
    /// boundary: where a function cut short of its grammar ends.
    pub(crate) fn descendant_children_end(self, path: &[usize]) -> Option<Self> {
        let group = self.descendant(path, false)?;
        let list = group.node()?.children()?;
        Some(Self {
            document: self.document,
            list,
            index: self.document.lists[list].len(),
            closing: false,
        })
    }
    /// Locate an explicit source fault only within this cursor's original snapshot.
    /// Checked input deliberately has no reverse lookup by origin identity.
    pub(crate) fn source_origin(self, origin: &CssValueOrigin) -> Option<Self> {
        let source = self.document.source.as_ref()?;
        let CssValueOrigin::Parsed(origin) = origin else {
            return None;
        };
        if !source.original.same_snapshot(origin.source()) {
            return None;
        }
        let offset = origin.span().start().byte_offset().value();
        for (list, nodes) in self.document.lists.iter().enumerate() {
            for (index, id) in nodes.iter().copied().enumerate() {
                if self
                    .document
                    .control
                    .is_some_and(|control| control.step().is_err())
                {
                    return None;
                }
                let node = &self.document.nodes[id];
                if self
                    .document
                    .range(id)
                    .is_some_and(|range| range.start == offset)
                {
                    return Some(Self {
                        document: self.document,
                        list,
                        index,
                        closing: false,
                    });
                }
                if let Some(GroupEnd::Explicit(token)) = node.end()
                    && let CssValueOrigin::Parsed(closing) = token.origin.as_ref()
                    && closing.span().start().byte_offset().value() == offset
                {
                    return Some(Self {
                        document: self.document,
                        list,
                        index,
                        closing: true,
                    });
                }
            }
        }
        None
    }
    pub(crate) fn component_path(self) -> Vec<usize> {
        let mut path = vec![self.index];
        let mut list = self.list;
        while list != self.document.root {
            let parent = self
                .document
                .lists
                .iter()
                .enumerate()
                .find_map(|(parent_list, nodes)| {
                    nodes
                        .iter()
                        .position(|id| self.document.nodes[*id].children() == Some(list))
                        .map(|index| (parent_list, index))
                })
                .expect("normalized child list belongs to one group");
            path.push(parent.1);
            list = parent.0;
        }
        path.reverse();
        path
    }
    /// Adjacent lexical candidates in this actual component list. An enclosing
    /// close is a boundary candidate; it never exposes a following sibling.
    pub(crate) fn diagnostic_candidates(self) -> (Option<Self>, Option<Self>) {
        let context = if self.closing {
            self.node()
                .and_then(SyntaxNode::children)
                .map_or(self, |list| Self {
                    document: self.document,
                    list,
                    index: self.document.lists[list].len(),
                    closing: false,
                })
        } else {
            self
        };
        let nodes = &self.document.lists[context.list];
        let mut next = None;
        for (index, node) in nodes.iter().enumerate().skip(context.index) {
            if self
                .control()
                .is_some_and(|control| control.step().is_err())
            {
                return (None, None);
            }
            if !self.document.nodes[*node].is_trivia() {
                next = Some(Self { index, ..context });
                break;
            }
        }
        if next.is_none() {
            'parents: for (list, nodes) in self.document.lists.iter().enumerate() {
                for (index, id) in nodes.iter().enumerate() {
                    if self
                        .control()
                        .is_some_and(|control| control.step().is_err())
                    {
                        return (None, None);
                    }
                    if self.document.nodes[*id].children() == Some(context.list) {
                        next = Some(Self {
                            document: self.document,
                            list,
                            index,
                            closing: true,
                        });
                        break 'parents;
                    }
                }
            }
        }
        let mut previous = None;
        for index in (0..context.index).rev() {
            if self
                .control()
                .is_some_and(|control| control.step().is_err())
            {
                return (None, None);
            }
            if !self.document.nodes[nodes[index]].is_trivia() {
                previous = Some(Self {
                    index,
                    closing: self.document.nodes[nodes[index]].end().is_some(),
                    ..context
                });
                break;
            }
        }
        (next, previous)
    }
    pub(crate) fn lexeme(self) -> Option<&'i str> {
        self.syntax_token()?.spelling.as_deref()
    }
    pub(crate) fn source_position(self) -> Option<CssSourcePosition> {
        if self.closing
            && let Some(GroupEnd::Implicit { at, .. }) = self.node().and_then(SyntaxNode::end)
            && let Some(source) = &at.source
        {
            return Some(CssSourcePosition::from_byte_offset_in(
                source.snapshot.as_str(),
                source.offset,
            ));
        }
        if let Some(token) = self.syntax_token() {
            match token.origin.as_ref() {
                CssValueOrigin::Parsed(origin) => return Some(origin.span().start()),
                CssValueOrigin::ImplicitClosure { at, .. } => return Some(at.span().start()),
                _ => {}
            }
        }
        let boundary = self.document.boundary(self.list, self.index);
        boundary
            .source
            .map(|site| CssSourcePosition::from_byte_offset_in(site.snapshot.as_str(), site.offset))
    }
    pub(crate) fn origin(self) -> CssSerializedOrigin {
        if self.closing
            && let Some(node) = self.node()
            && let TokenPayload::Checked { component, .. } = &node.token().payload
        {
            let closing = match component.view() {
                CssComponentValueRef::Function(function) => function.closing_origin(),
                CssComponentValueRef::Block(block) => block.closing_origin(),
                _ => unreachable!("closing bookmark belongs to an actual group"),
            };
            return CssSerializedOrigin::Token(closing.clone());
        }
        if self.closing
            && let Some(GroupEnd::Implicit { opening, at }) = self.node().and_then(SyntaxNode::end)
        {
            return implicit_end_origin(opening, at);
        }
        if let Some(token) = self.syntax_token() {
            return CssSerializedOrigin::Token(token.origin.clone().into_owned());
        }
        // A child EOF is the original closing boundary, even for empty groups.
        if self.index == self.document.lists[self.list].len() {
            for node in &self.document.nodes {
                if self
                    .control()
                    .is_some_and(|control| control.step().is_err())
                {
                    return CssSerializedOrigin::End(None);
                }
                if node.children() == Some(self.list) {
                    return match node.end().expect("group end") {
                        GroupEnd::Explicit(token) => {
                            CssSerializedOrigin::Token(token.origin.clone().into_owned())
                        }
                        GroupEnd::Implicit { opening, at } => implicit_end_origin(opening, at),
                    };
                }
            }
        }
        let previous = self
            .index
            .checked_sub(1)
            .and_then(|index| self.document.lists[self.list].get(index))
            .map(|id| &self.document.nodes[*id]);
        CssSerializedOrigin::End(previous.map(|node| match node.end() {
            Some(GroupEnd::Explicit(token)) => token.origin.clone().into_owned(),
            Some(GroupEnd::Implicit { opening, at }) => match implicit_end_origin(opening, at) {
                CssSerializedOrigin::End(Some(origin)) => origin,
                _ => unreachable!("implicit boundary retains its opening origin"),
            },
            None => node.token().origin.clone().into_owned(),
        }))
    }
    /// The grammar's own error, charged to the meter like any retained value.
    pub(crate) fn grammar_error<E, E2: From<E>>(self, error: E) -> SyntaxError<'i, E2> {
        if self
            .control()
            .is_some_and(|control| control.add(std::mem::size_of::<E2>()).is_err())
        {
            return self.end_of_stream();
        }
        SyntaxError::custom(self, error.into())
    }
    pub(crate) fn unexpected<E>(self, token: Token<'i>) -> SyntaxError<'i, E> {
        StreamError::unexpected(self, token).into()
    }
    pub(crate) fn end_of_stream<E>(self) -> SyntaxError<'i, E> {
        StreamError::end_of_stream(self).into()
    }
}

/// Spec 5.5.7: a stop token a list of component values can end at.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StopToken {
    Comma,
    Semicolon,
    ExclamationMark,
    OpenCurly,
}
/// A set of stop tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StopTokens(u8);
impl StopToken {
    const fn bit(self) -> u8 {
        match self {
            Self::Comma => 1,
            Self::Semicolon => 2,
            Self::ExclamationMark => 4,
            Self::OpenCurly => 8,
        }
    }
}
impl StopTokens {
    const fn contains(self, stop: StopToken) -> bool {
        self.0 & stop.bit() != 0
    }
}
impl From<StopToken> for StopTokens {
    fn from(stop: StopToken) -> Self {
        Self(stop.bit())
    }
}

/// A restore point of a stream (spec 5.3, "mark").
#[derive(Clone, Copy, Debug)]
pub(crate) struct Mark<'i> {
    site: Site<'i>,
    pending: Option<usize>,
    start: usize,
    end: usize,
}
impl<'i> Mark<'i> {
    pub(crate) fn site(&self) -> Site<'i> {
        self.site
    }
}

/// One bounded cursor over a list of the syntax arena (spec 5.3, "token stream").
pub(crate) struct Stream<'i> {
    document: &'i SyntaxDocument<'i>,
    list: ListId,
    start: usize,
    next: usize,
    end: usize,
    pending: Option<usize>,
    token: Option<Token<'i>>,
}
impl<'i> Stream<'i> {
    pub(crate) fn control(&self) -> Option<&'i Control<'i>> {
        self.document.control
    }
    pub(crate) fn controlled_step(&self) -> Result<(), StreamError<'i>> {
        if self
            .control()
            .is_some_and(|control| control.step().is_err())
        {
            return Err(StreamError::end_of_stream(self.site()));
        }
        Ok(())
    }
    pub(crate) fn reserve_controlled<T>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), StreamError<'i>> {
        self.controlled_step()?;
        if let Some(control) = self.control() {
            control
                .reserve(values, additional)
                .map_err(|_| StreamError::end_of_stream(self.site()))
        } else {
            values
                .try_reserve(additional)
                .map_err(|_| StreamError::end_of_stream(self.site()))
        }
    }

    pub(crate) fn new(document: &'i SyntaxDocument<'i>) -> Self {
        Self::bounded(
            document,
            document.root,
            0..document.lists[document.root].len(),
        )
    }
    pub(crate) fn bounded(
        document: &'i SyntaxDocument<'i>,
        list: ListId,
        range: Range<usize>,
    ) -> Self {
        assert!(range.start <= range.end && range.end <= document.lists[list].len());
        Self {
            document,
            list,
            start: range.start,
            next: range.start,
            end: range.end,
            pending: None,
            token: None,
        }
    }
    pub(crate) fn site(&self) -> Site<'i> {
        Site {
            document: self.document,
            list: self.list,
            index: self.next,
            closing: false,
        }
    }
    pub(crate) fn mark(&self) -> Mark<'i> {
        Mark {
            site: self.site(),
            pending: self.pending,
            start: self.start,
            end: self.end,
        }
    }
    pub(crate) fn restore(&mut self, mark: Mark<'i>) {
        assert!(
            std::ptr::eq(self.document, mark.site.document)
                && self.list == mark.site.list
                && self.start == mark.start
                && self.end == mark.end
                && (self.start..=self.end).contains(&mark.site.index)
        );
        self.next = mark.site.index;
        self.pending = mark.pending;
        self.token = None;
    }
    pub(crate) fn attempt<F, T, E>(&mut self, parse: F) -> Result<T, E>
    where
        F: FnOnce(&mut Self) -> Result<T, E>,
    {
        let mark = self.mark();
        let result = parse(self);
        if result.is_err() {
            self.restore(mark);
        }
        result
    }
    pub(crate) fn discard_whitespace(&mut self) {
        while self.next < self.end {
            if self.controlled_step().is_err() {
                break;
            }
            if !self.document.nodes[self.document.lists[self.list][self.next]].is_trivia() {
                break;
            }
            self.next += 1;
        }
        self.pending = None;
    }
    pub(crate) fn only_whitespace_remains(&mut self) -> bool {
        if self.controlled_step().is_err() {
            return true;
        }
        let state = self.mark();
        self.discard_whitespace();
        let result = self.next == self.end;
        self.restore(state);
        result
    }
    pub(crate) fn next_kind(
        &mut self,
        include_trivia: bool,
    ) -> Result<TokenKind<'i>, StreamError<'i>> {
        if !include_trivia {
            self.discard_whitespace();
        }
        self.controlled_step()?;
        if self.next == self.end {
            return Err(StreamError::end_of_stream(self.site()));
        }
        let id = self.document.lists[self.list][self.next];
        self.next += 1;
        let node = &self.document.nodes[id];
        self.pending = node.children().map(|_| id);
        Ok(node.token().kind())
    }
    pub(crate) fn consume_after_whitespace(&mut self) -> Result<&Token<'i>, StreamError<'i>> {
        self.discard_whitespace();
        self.consume_token_with_comments()
    }
    pub(crate) fn consume_token_with_comments(&mut self) -> Result<&Token<'i>, StreamError<'i>> {
        self.controlled_step()?;
        let location = self.site();
        if self.next == self.end {
            return Err(StreamError::end_of_stream(location));
        }
        let id = self.document.lists[self.list][self.next];
        self.next += 1;
        let node = &self.document.nodes[id];
        self.pending = node.children().map(|_| id);
        self.token = Some(native_token(node.token()));
        Ok(self.token.as_ref().expect("selected token"))
    }
    pub(crate) fn require_empty(&mut self) -> Result<(), StreamError<'i>> {
        if self.control().is_some() {
            let state = self.mark();
            self.discard_whitespace();
            let location = self.site();
            let result = match self.next_kind(false) {
                Ok(_) => Err(StreamError::end_of_stream(location)),
                Err(error)
                    if self
                        .control()
                        .is_some_and(|control| control.failure().is_some()) =>
                {
                    Err(error)
                }
                Err(_) => Ok(()),
            };
            self.restore(state);
            return result;
        }
        let state = self.mark();
        self.discard_whitespace();
        let site = self.site();
        let result = match self.consume_after_whitespace() {
            Ok(token) => Err(StreamError::unexpected(site, token.clone())),
            Err(_) => Ok(()),
        };
        self.restore(state);
        result
    }
    pub(crate) fn unexpected<E>(&self, token: Token<'i>) -> SyntaxError<'i, E> {
        self.site().unexpected(token)
    }
    pub(crate) fn end_of_stream<E>(&self) -> SyntaxError<'i, E> {
        self.site().end_of_stream()
    }
    fn consume_matching(
        &mut self,
        predicate: impl FnOnce(&Token<'i>) -> bool,
    ) -> Result<(), StreamError<'i>> {
        self.discard_whitespace();
        let site = self.site();
        let token = self.consume_after_whitespace()?;
        if predicate(token) {
            Ok(())
        } else {
            Err(StreamError::unexpected(site, token.clone()))
        }
    }
    fn ident(&mut self) -> Result<&CowRcStr<'i>, StreamError<'i>> {
        self.discard_whitespace();
        let site = self.site();
        if self.control().is_some() {
            match self.next_kind(false)? {
                TokenKind::Ident(name) => {
                    self.token = Some(Token::Ident(name.into()));
                }
                _ => {
                    return Err(StreamError::end_of_stream(site));
                }
            }
            let Some(Token::Ident(name)) = self.token.as_ref() else {
                unreachable!("actual checked identifier")
            };
            return Ok(name);
        }
        match self.consume_after_whitespace()? {
            Token::Ident(name) => Ok(name),
            token => Err(StreamError::unexpected(site, token.clone())),
        }
    }
    pub(crate) fn consume_ident(&mut self) -> Result<CowRcStr<'i>, StreamError<'i>> {
        self.ident().cloned()
    }
    pub(crate) fn consume_keyword(&mut self, expected: &str) -> Result<(), StreamError<'i>> {
        self.discard_whitespace();
        let site = self.site();
        let ident = self.ident()?;
        if ident.eq_ignore_ascii_case(expected) {
            Ok(())
        } else {
            Err(StreamError::unexpected(site, Token::Ident(ident.clone())))
        }
    }
    pub(crate) fn consume_function_name(&mut self) -> Result<&CowRcStr<'i>, StreamError<'i>> {
        self.discard_whitespace();
        let site = self.site();
        match self.consume_after_whitespace()? {
            Token::Function(name) => Ok(name),
            token => Err(StreamError::unexpected(site, token.clone())),
        }
    }
    pub(crate) fn consume_function_named(&mut self, expected: &str) -> Result<(), StreamError<'i>> {
        self.consume_matching(
            |token| matches!(token, Token::Function(name) if name.eq_ignore_ascii_case(expected)),
        )
    }
    pub(crate) fn consume_string(&mut self) -> Result<CowRcStr<'i>, StreamError<'i>> {
        self.discard_whitespace();
        let site = self.site();
        match self.consume_after_whitespace()? {
            Token::QuotedString(value) => Ok(value.clone()),
            token => Err(StreamError::unexpected(site, token.clone())),
        }
    }
    pub(crate) fn consume_delim(&mut self, expected: char) -> Result<(), StreamError<'i>> {
        self.consume_matching(|token| matches!(token, Token::Delim(value) if *value == expected))
    }
    pub(crate) fn consume_comma(&mut self) -> Result<(), StreamError<'i>> {
        self.consume_matching(|token| matches!(token, Token::Comma))
    }
    #[cfg(test)]
    pub(crate) fn consume_semicolon(&mut self) -> Result<(), StreamError<'i>> {
        self.consume_matching(|token| matches!(token, Token::Semicolon))
    }
    pub(crate) fn parse_complete<F, T, E>(&mut self, parse: F) -> Result<T, SyntaxError<'i, E>>
    where
        F: FnOnce(&mut Self) -> Result<T, SyntaxError<'i, E>>,
    {
        let value = parse(self)?;
        self.require_empty()?;
        Ok(value)
    }
    pub(crate) fn within_block<F, T, E>(&mut self, parse: F) -> Result<T, SyntaxError<'i, E>>
    where
        F: FnOnce(&mut Stream<'i>) -> Result<T, SyntaxError<'i, E>>,
    {
        let Some(id) = self.pending.take() else {
            return Err(self.end_of_stream());
        };
        let children = self.document.nodes[id].children().expect("pending group");
        let mut nested = Stream::bounded(
            self.document,
            children,
            0..self.document.lists[children].len(),
        );
        let result = parse(&mut nested);
        result.and_then(|value| {
            nested.require_empty()?;
            Ok(value)
        })
    }
    pub(crate) fn within_list_until<F, T, E>(
        &mut self,
        stops: impl Into<StopTokens>,
        parse: F,
    ) -> Result<T, SyntaxError<'i, E>>
    where
        F: FnOnce(&mut Stream<'i>) -> Result<T, SyntaxError<'i, E>>,
    {
        let stops = stops.into();
        let end = (self.next..self.end)
            .find(|index| {
                is_stop_token(
                    self.document.nodes[self.document.lists[self.list][*index]]
                        .token()
                        .kind(),
                    stops,
                )
            })
            .unwrap_or(self.end);
        let mut nested = Stream::bounded(self.document, self.list, self.next..end);
        let result = parse(&mut nested).and_then(|value| {
            nested.require_empty()?;
            Ok(value)
        });
        self.next = end;
        self.pending = None;
        result
    }
    /// The shared Values 4 §2.3 `#` multiplier: one or more items separated by
    /// commas. Each item parser is bounded at its comma and must consume its
    /// whole item; whole-list rules stay with the caller as post-conditions.
    pub(crate) fn comma_segments<F, T, E>(
        &mut self,
        mut parse: F,
    ) -> Result<Vec<T>, SyntaxError<'i, E>>
    where
        F: FnMut(&mut Stream<'i>) -> Result<T, SyntaxError<'i, E>>,
    {
        let mut values = Vec::new();
        loop {
            let value = self.within_list_until(StopToken::Comma, &mut parse)?;
            self.reserve_controlled(&mut values, 1)?;
            values.push(value);
            if self.only_whitespace_remains() {
                return Ok(values);
            }
            self.consume_comma()?;
        }
    }
    pub(crate) fn collect_component(
        &mut self,
    ) -> Result<Cow<'i, CssComponentValue>, CssComponentValueError> {
        self.controlled_step().map_err(|_| {
            CssComponentValueError::new(
                CssComponentValueErrorKind::CapacityOverflow,
                CssValueOrigin::Programmatic,
            )
        })?;
        self.discard_whitespace();
        if self.next == self.end {
            let origin = match self.site().origin() {
                CssSerializedOrigin::End(Some(origin)) | CssSerializedOrigin::Token(origin) => {
                    origin
                }
                _ => CssValueOrigin::Programmatic,
            };
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                origin,
            ));
        }
        let id = self.document.lists[self.list][self.next];
        self.next += 1;
        self.pending = None;
        if let TokenPayload::Checked { component, .. } = self.document.nodes[id].token().payload {
            return Ok(Cow::Borrowed(component));
        }
        let values = crate::component::values::parse::promote_nodes(
            self.document,
            &[id],
            CssComponentValueLimits::default(),
        )?;
        values.into_single().map(Cow::Owned).ok_or_else(|| {
            CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                self.document.nodes[id].token().origin.clone().into_owned(),
            )
        })
    }
    pub(crate) fn collect_numeric_component(
        &mut self,
        _numeric: &crate::numeric::expression::NumericInputContext<'_>,
    ) -> Result<CssComponentValue, crate::numeric::CssNumericConstructionError> {
        self.collect_component()
            .map(Cow::into_owned)
            .map_err(crate::numeric::CssNumericConstructionError::component)
    }
    /// Owned text for retained output only. Grammar never consumes this string.
    pub(crate) fn text_between(&self, from: Site<'i>, to: Site<'i>) -> String {
        assert!(
            std::ptr::eq(from.document, self.document)
                && std::ptr::eq(to.document, self.document)
                && from.list == self.list
                && to.list == self.list
        );
        if let Some(source) = &self.document.source {
            let start = self
                .document
                .boundary(self.list, from.index)
                .source
                .expect("source boundary")
                .offset;
            let end = self
                .document
                .boundary(self.list, to.index)
                .source
                .expect("source boundary")
                .offset;
            let text = &source.original.as_str()[start..end];
            return match self.control() {
                Some(control) => control.string(text).unwrap_or_default(),
                None => text.to_owned(),
            };
        }
        assert!(
            self.control().is_none(),
            "controlled property slices retain their actual source"
        );
        let mut output = crate::component::values::serialize::CssCanonicalBuilder::new(usize::MAX);
        for id in &self.document.lists[self.list][from.index..to.index] {
            let TokenPayload::Checked { component, .. } = &self.document.nodes[*id].token().payload
            else {
                unreachable!("checked property output borrows checked components");
            };
            output
                .push_components(std::slice::from_ref(*component))
                .expect("checked output boundaries");
        }
        output
            .finish()
            .expect("checked output boundaries")
            .into_css()
    }
    pub(crate) fn text_since(&self, start: Site<'i>) -> String {
        self.text_between(start, self.site())
    }
}
fn implicit_end_origin(
    opening: &CssValueOrigin,
    at: &crate::syntax_consumption::SyntaxBoundary,
) -> CssSerializedOrigin {
    let origin = match (opening, &at.source) {
        (CssValueOrigin::Parsed(opening), Some(source)) => {
            crate::source::CssParsedOrigin::from_range(
                &source.snapshot,
                source.offset..source.offset,
            )
            .map(|at| CssValueOrigin::ImplicitClosure {
                opening: opening.clone(),
                at,
            })
            .unwrap_or_else(|| CssValueOrigin::Parsed(opening.clone()))
        }
        _ => opening.clone(),
    };
    CssSerializedOrigin::End(Some(origin))
}
fn is_stop_token(kind: TokenKind<'_>, stops: StopTokens) -> bool {
    let wanted = match kind {
        TokenKind::Semicolon => StopToken::Semicolon,
        TokenKind::Comma => StopToken::Comma,
        TokenKind::Delim('!') => StopToken::ExclamationMark,
        TokenKind::Opening(CssBlockKind::CurlyBracket) => StopToken::OpenCurly,
        _ => return false,
    };
    stops.contains(wanted)
}
/// Numeric fields mirror lexical dispatch only. Exact numerical grammar uses
/// `collect_component` and the CSS numerical owner, never these rounded fields.
fn numeric_fields(number: CssNumericTokenRef<'_>) -> (bool, f64, Option<i32>) {
    let value: f64 = number
        .representation()
        .parse()
        .expect("checked numeric spelling");
    // Like the lexical provider, `Some` records the integer token category;
    // saturation is only a dispatch observation, never the numerical value.
    let int_value = (number.kind() == CssNumericTokenKind::Integer).then_some(value as i32);
    (number.has_sign(), value, int_value)
}
fn native_token<'i>(token: &'i SyntaxToken<'i>) -> Token<'i> {
    match &token.payload {
        TokenPayload::Native(token) => token.clone(),
        TokenPayload::Whitespace(value) => Token::WhiteSpace(value),
        TokenPayload::Comment(value) => Token::Comment(value),
        TokenPayload::Checked { component, lexeme } => match (lexeme, component.view()) {
            (CheckedLexeme::Opening, CssComponentValueRef::Function(value)) => {
                Token::Function(value.name().into())
            }
            (CheckedLexeme::Opening, CssComponentValueRef::Block(value)) => match value.kind() {
                CssBlockKind::Parenthesis => Token::ParenthesisBlock,
                CssBlockKind::SquareBracket => Token::SquareBracketBlock,
                CssBlockKind::CurlyBracket => Token::CurlyBracketBlock,
            },
            (CheckedLexeme::Closing, _) => match token.kind() {
                TokenKind::Closing(CssBlockKind::Parenthesis) => Token::CloseParenthesis,
                TokenKind::Closing(CssBlockKind::SquareBracket) => Token::CloseSquareBracket,
                _ => Token::CloseCurlyBracket,
            },
            (CheckedLexeme::Leaf, CssComponentValueRef::Comment(value)) => Token::Comment(value),
            (CheckedLexeme::Leaf, CssComponentValueRef::Token(value)) => match value {
                CssValueTokenRef::Ident(value) => Token::Ident(value.into()),
                CssValueTokenRef::AtKeyword(value) => Token::AtKeyword(value.into()),
                CssValueTokenRef::Hash { value, flag } => match flag {
                    CssHashFlag::Id => Token::IDHash(value.into()),
                    CssHashFlag::Unrestricted => Token::Hash(value.into()),
                },
                CssValueTokenRef::String(value) => Token::QuotedString(value.into()),
                CssValueTokenRef::Url(value) => Token::UnquotedUrl(value.into()),
                CssValueTokenRef::Delim(value) => Token::Delim(value),
                CssValueTokenRef::Number(number) => {
                    let (has_sign, value, int_value) = numeric_fields(number);
                    Token::Number {
                        has_sign,
                        value: value as f32,
                        int_value,
                    }
                }
                CssValueTokenRef::Percentage(number) => {
                    let (has_sign, value, int_value) = numeric_fields(number);
                    Token::Percentage {
                        has_sign,
                        unit_value: (value / 100.0) as f32,
                        int_value,
                    }
                }
                CssValueTokenRef::Dimension { number, unit } => {
                    let (has_sign, value, int_value) = numeric_fields(number);
                    Token::Dimension {
                        has_sign,
                        value: value as f32,
                        int_value,
                        unit: unit.into(),
                    }
                }
                CssValueTokenRef::Whitespace(value) => Token::WhiteSpace(value),
                CssValueTokenRef::Colon => Token::Colon,
                CssValueTokenRef::Semicolon => Token::Semicolon,
                CssValueTokenRef::Comma => Token::Comma,
                CssValueTokenRef::Cdo => Token::CDO,
                CssValueTokenRef::Cdc => Token::CDC,
            },
            _ => unreachable!("checked lexeme role"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::CssComponentValues;
    use crate::source::CssSourceSnapshot;
    use crate::syntax_consumption::{SyntaxInput, SyntaxInputLimits, normalize, source_document};

    #[test]
    fn independent_checked_leaves_do_not_merge_into_a_new_token() {
        let values = CssComponentValues::try_new(
            vec![
                CssComponentValue::try_ident("a").unwrap(),
                CssComponentValue::try_ident("b").unwrap(),
            ],
            Default::default(),
        )
        .unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let mut input = Stream::new(&document);
        assert_eq!(input.consume_ident().unwrap().as_ref(), "a");
        assert_eq!(input.consume_ident().unwrap().as_ref(), "b");
        assert!(input.only_whitespace_remains());
    }

    #[test]
    fn checked_collection_borrows_the_exact_original_component() {
        let values = crate::component::values::parse_component_values("calc(1px + 2px)").unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let mut input = Stream::new(&document);
        let value = input.collect_component().unwrap();
        assert!(matches!(value, Cow::Borrowed(_)));
        assert!(std::ptr::eq(value.as_ref(), &values.items()[0]));
    }

    #[test]
    fn failed_nested_parse_advances_outer_and_restore_returns_the_pending_group() {
        let snapshot = CssSourceSnapshot::new("f(a b) tail");
        let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
        let mut input = Stream::new(&document);
        input.consume_function_named("f").unwrap();
        let checkpoint = input.mark();
        let error = input
            .within_block(|nested| {
                nested.consume_keyword("a")?;
                nested.consume_keyword("wrong")?;
                Ok::<_, SyntaxError<'_, ()>>(())
            })
            .unwrap_err();
        assert_eq!(error.at().lexeme(), Some("b"));
        assert_eq!(input.consume_ident().unwrap().as_ref(), "tail");
        input.restore(checkpoint);
        input
            .within_block(|nested| {
                nested.consume_keyword("a")?;
                nested.consume_keyword("b")?;
                Ok::<_, SyntaxError<'_, ()>>(())
            })
            .unwrap();
        assert_eq!(input.consume_ident().unwrap().as_ref(), "tail");
    }

    #[test]
    fn lexical_error_leaves_remain_visible_until_grammar_visits_them() {
        let snapshot = CssSourceSnapshot::new("url(a b)!");
        let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
        let mut input = Stream::new(&document);
        assert!(matches!(
            input.consume_after_whitespace().unwrap(),
            Token::BadUrl(_)
        ));
        assert_eq!(
            input.consume_after_whitespace().unwrap(),
            &Token::Delim('!')
        );
    }

    #[test]
    fn descendant_sites_select_real_mixed_source_children_and_closing_boundaries() {
        let children = crate::component::values::parse_component_values("\n 2px").unwrap();
        let origin = children.items()[1].origin().clone();
        let group = CssComponentValue::try_function("f", children).unwrap();
        let values = CssComponentValues::try_new(vec![group], Default::default()).unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let input = Stream::new(&document);
        let child = input.site().descendant(&[0, 1], false).unwrap();
        assert_eq!(child.value_origin(), Some(origin));
        assert!(matches!(child.token(), Some(Token::Dimension { .. })));
        let closing = input.site().descendant(&[0], true).unwrap();
        assert_eq!(closing.value_origin(), Some(CssValueOrigin::Programmatic));
        assert_eq!(closing.token(), Some(Token::CloseParenthesis));
    }

    #[test]
    fn nested_end_reports_the_actual_closing_token_and_root_end_reports_eof() {
        let values = crate::component::values::parse_component_values("f()").unwrap();
        let CssComponentValueRef::Function(function) = values.items()[0].view() else {
            unreachable!()
        };
        let closing = function.closing_origin().clone();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let mut input = Stream::new(&document);
        input.consume_function_named("f").unwrap();
        let nested = input
            .within_block(|nested| {
                nested
                    .consume_ident()
                    .map_err(crate::diagnostic::error::property::basic)
            })
            .unwrap_err();
        let (kind, origin) = nested.into_parts();
        assert!(matches!(
            kind,
            crate::diagnostic::error::ErrorKind::UnexpectedEnd(_)
        ));
        assert_eq!(origin, CssSerializedOrigin::Token(closing.clone()));
        let (kind, origin) =
            crate::diagnostic::error::property::basic(input.consume_ident().unwrap_err())
                .into_parts();
        assert!(matches!(
            kind,
            crate::diagnostic::error::ErrorKind::UnexpectedEnd(_)
        ));
        assert_eq!(origin, CssSerializedOrigin::End(Some(closing)));
    }

    #[test]
    fn bounded_delimiter_parse_keeps_the_real_outer_delimiter() {
        let snapshot = CssSourceSnapshot::new("a f(b;c);tail");
        let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
        let mut input = Stream::new(&document);
        input
            .within_list_until(StopToken::Semicolon, |nested| {
                nested.consume_keyword("a")?;
                nested.consume_function_named("f")?;
                Ok::<_, SyntaxError<'_, ()>>(())
            })
            .unwrap();
        assert_eq!(input.site().lexeme(), Some(";"));
        input.consume_semicolon().unwrap();
        assert_eq!(input.consume_ident().unwrap().as_ref(), "tail");
    }
}

/// Existing native descriptor/rule consumers enter the same value grammar.
/// The checked property front door never enters this adapter.
pub(crate) enum NativeValueRange {
    /// A single-node helper must not admit an unrelated following value.
    Component,
    /// Family names and settings lists consume a sequence of sibling values.
    Remaining,
}

pub(crate) fn adapt_native<'s, 'n, T, F>(
    input: &mut cssparser::Parser<'s, '_>,
    numeric: &crate::numeric::expression::NumericInputContext<'n>,
    range: NativeValueRange,
    parse: F,
) -> Result<T, cssparser::ParseError<'s, crate::diagnostic::CssError>>
where
    F: for<'i> FnOnce(
        &mut Stream<'i>,
        &crate::numeric::expression::NumericInputContext<'n>,
    ) -> Result<T, SyntaxError<'i, crate::diagnostic::error::property::Error>>,
{
    use crate::numeric::expression::NumericInputContext;
    use crate::syntax_consumption::{SourceWindow, SyntaxInput, SyntaxInputLimits, normalize};
    let checkpoint = input.state();
    let start = input.position().byte_index();
    match range {
        NativeValueRange::Component => {
            input.skip_whitespace();
            let _ = advance_native_component(input);
        }
        NativeValueRange::Remaining => while advance_native_component(input).is_ok() {},
    }
    let end = input.position().byte_index();
    input.reset(&checkpoint);
    match numeric.ordinary() {
        NumericInputContext::Parsed(source) => {
            let document = normalize(
                SyntaxInput::Source(SourceWindow {
                    text: Cow::Borrowed(source.as_str()),
                    range: start..end,
                    original: (*source).clone(),
                }),
                SyntaxInputLimits::default(),
                0,
            )
            .map_err(|detail| {
                crate::diagnostic::error::invalid_component_value(
                    input.current_source_location(),
                    detail,
                )
            })?;
            let mut parser = Stream::new(&document);
            let result = parse(&mut parser, numeric);
            let consumed = document
                .boundary(document.root, parser.site().ordinal())
                .source
                .expect("source cursor boundary")
                .offset;
            advance_native_to(input, consumed);
            result.map_err(SyntaxError::into_native_source)
        }
        NumericInputContext::Components(values, serialized) => {
            let paths = serialized
                .component_paths_in_range(start..end)
                .ok_or_else(|| {
                    crate::diagnostic::error::invalid_syntax(input.current_source_location())
                })?;
            let (siblings, first, parent) = if let Some(path) = paths.first() {
                let (first, parent) = path.split_last().expect("component path");
                let siblings = if parent.is_empty() {
                    values.items()
                } else {
                    values
                        .component_at_path(parent)
                        .and_then(CssComponentValue::child_values)
                        .expect("mapped component parent")
                        .items()
                };
                (&siblings[*first..*first + paths.len()], *first, parent)
            } else {
                (&[][..], 0, &[][..])
            };
            let document = normalize(
                SyntaxInput::CheckedComponents(siblings),
                SyntaxInputLimits::default(),
                0,
            )
            .map_err(|detail| {
                crate::diagnostic::error::invalid_component_value(
                    input.current_source_location(),
                    detail,
                )
            })?;
            let mut parser = Stream::new(&document);
            let result = parse(&mut parser, numeric);
            let ordinal = parser.site().ordinal();
            let consumed = paths
                .get(ordinal)
                .and_then(|path| serialized.component_offset_for_path(path))
                .unwrap_or(end);
            advance_native_to(input, consumed);
            result.map_err(|error| {
                let error = error.qualify();
                let site = error.at();
                let relative = site.component_path();
                let offset = if relative
                    .first()
                    .is_some_and(|index| *index >= siblings.len())
                {
                    Some(end)
                } else {
                    let mut path = parent.to_vec();
                    path.push(first + relative[0]);
                    path.extend_from_slice(&relative[1..]);
                    if site.closing {
                        serialized
                            .component_end_for_path(&path)
                            .map(|end| end.saturating_sub(1))
                    } else if let Some(offset) = serialized.component_offset_for_path(&path) {
                        Some(offset)
                    } else if relative.len() > 1 {
                        path.pop();
                        serialized
                            .component_end_for_path(&path)
                            .map(|end| end.saturating_sub(1))
                    } else {
                        None
                    }
                }
                .unwrap_or(end);
                error.into_native_at(serialized.as_css(), offset)
            })
        }
        NumericInputContext::Checked => {
            unreachable!("checked property input enters the component cursor directly")
        }
        NumericInputContext::QuirkyLengths(_) => unreachable!("ordinary numeric context"),
    }
}
fn advance_native_component(input: &mut cssparser::Parser<'_, '_>) -> Result<(), ()> {
    let nested = matches!(
        input
            .next_including_whitespace_and_comments()
            .map_err(|_| ())?,
        Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock
    );
    if nested {
        input
            .parse_nested_block(|nested| {
                while nested.next_including_whitespace_and_comments().is_ok() {}
                Ok::<_, cssparser::ParseError<'_, ()>>(())
            })
            .map_err(|_| ())?;
    }
    Ok(())
}
fn advance_native_to(input: &mut cssparser::Parser<'_, '_>, offset: usize) {
    while input.position().byte_index() < offset && advance_native_component(input).is_ok() {}
    assert_eq!(
        input.position().byte_index(),
        offset,
        "portable cursor selects an actual native boundary"
    );
}

#[cfg(test)]
mod native_adapter_tests {
    use super::*;
    use crate::component::CssComponentValues;
    use crate::numeric::expression::NumericInputContext;
    use crate::source::CssSourceSnapshot;

    #[test]
    fn source_url_prefix_does_not_admit_the_unconsumed_nested_suffix() {
        let prefix = " /*lead*/ url(test)";
        let source = CssSourceSnapshot::new(&format!(
            "{prefix} {}x{}",
            "future(".repeat(257),
            ")".repeat(257)
        ));
        let mut storage = cssparser::ParserInput::new(source.as_str());
        let mut native = cssparser::Parser::new(&mut storage);
        let value = adapt_native(
            &mut native,
            &NumericInputContext::Parsed(&source),
            NativeValueRange::Component,
            super::super::url::parse_url,
        )
        .unwrap();
        assert_eq!(value.as_str(), "test");
        assert_eq!(native.position().byte_index(), prefix.len());
        native.expect_function_matching("future").unwrap();
    }

    #[test]
    fn source_family_sequence_retains_all_words() {
        let source = CssSourceSnapshot::new("Fancy Font");
        let mut storage = cssparser::ParserInput::new(source.as_str());
        let mut native = cssparser::Parser::new(&mut storage);
        let value = adapt_native(
            &mut native,
            &NumericInputContext::Parsed(&source),
            NativeValueRange::Remaining,
            |input, _| super::super::typography::parse_non_generic_font_family_name(input),
        )
        .unwrap();
        assert_eq!(value.as_str(), "Fancy Font");
        assert!(native.is_exhausted());
    }

    #[test]
    fn source_prefix_adapter_leaves_the_following_component_available() {
        let source = CssSourceSnapshot::new("f(a) tail");
        let mut storage = cssparser::ParserInput::new(source.as_str());
        let mut native = cssparser::Parser::new(&mut storage);
        let value = adapt_native(
            &mut native,
            &NumericInputContext::Parsed(&source),
            NativeValueRange::Component,
            |input, _| {
                input.consume_function_named("f")?;
                input.within_block(|nested| {
                    nested.consume_keyword("a")?;
                    Ok::<_, SyntaxError<'_, crate::diagnostic::error::property::Error>>(42)
                })
            },
        )
        .unwrap();
        assert_eq!(value, 42);
        assert_eq!(native.position().byte_index(), 4);
        assert_eq!(native.expect_ident().unwrap().as_ref(), "tail");
    }

    #[test]
    fn checked_prefix_adapter_borrows_the_original_component_across_inserted_separators() {
        let values = CssComponentValues::try_new(
            vec![
                CssComponentValue::try_ident("a").unwrap(),
                CssComponentValue::try_ident("b").unwrap(),
            ],
            Default::default(),
        )
        .unwrap();
        let serialized = values.serialize_tokens(Default::default()).unwrap();
        let mut storage = cssparser::ParserInput::new(serialized.as_css());
        let mut native = cssparser::Parser::new(&mut storage);
        let address = adapt_native(
            &mut native,
            &NumericInputContext::Components(&values, &serialized),
            NativeValueRange::Component,
            |input, _| {
                let component = input.collect_component().map_err(|error| {
                    crate::diagnostic::error::property::invalid_component_value(input.site(), error)
                })?;
                assert!(matches!(component, Cow::Borrowed(_)));
                Ok(component.as_ref() as *const CssComponentValue as usize)
            },
        )
        .unwrap();
        assert_eq!(
            address,
            &values.items()[0] as *const CssComponentValue as usize
        );
        assert_eq!(native.expect_ident().unwrap().as_ref(), "b");
    }

    #[test]
    fn checked_nested_adapter_maps_the_actual_child_path_to_existing_native_input() {
        let values = crate::component::values::parse_component_values("f(a wrong)").unwrap();
        let serialized = values.serialize_tokens(Default::default()).unwrap();
        let mut storage = cssparser::ParserInput::new(serialized.as_css());
        let mut native = cssparser::Parser::new(&mut storage);
        let error = adapt_native(
            &mut native,
            &NumericInputContext::Components(&values, &serialized),
            NativeValueRange::Component,
            |input, _| {
                input.consume_function_named("f")?;
                input.within_block(|nested| {
                    nested.consume_keyword("a")?;
                    nested.consume_keyword("other")?;
                    Ok::<_, SyntaxError<'_, crate::diagnostic::error::property::Error>>(())
                })
            },
        )
        .unwrap_err();
        let cssparser::ParseErrorKind::Custom(error) = error.kind else {
            panic!("typed checked error");
        };
        assert_eq!(
            error.position().byte_offset().value(),
            serialized.component_offset_for_path(&[0, 2]).unwrap()
        );
        assert_eq!(native.position().byte_index(), serialized.as_css().len());
    }
}
