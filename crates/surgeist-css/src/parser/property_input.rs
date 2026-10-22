//! One bounded property grammar cursor over source and checked syntax arenas.
use crate::error::property::{BasicParseError, ParseError, ParseErrorKind};
use crate::syntax_consumption::{
    CheckedLexeme, GroupEnd, ListId, SyntaxDocument, SyntaxNode, SyntaxToken, TokenKind,
    TokenPayload,
};
use crate::utf16_value::Control;
use crate::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueLimits, CssComponentValueRef, CssHashFlag, CssNumericTokenKind,
    CssNumericTokenRef, CssSerializedOrigin, CssSourcePosition, CssValueOrigin, CssValueTokenRef,
};
use cssparser::BasicParseErrorKind;
use cssparser::{CowRcStr, Delimiter, Delimiters, Token};
use std::{borrow::Cow, marker::PhantomData, ops::Range};

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
    pub(crate) fn new_custom_error<E, E2: From<E>>(self, error: E) -> ParseError<'i, E2> {
        if self
            .control()
            .is_some_and(|control| control.add(std::mem::size_of::<E2>()).is_err())
        {
            return self.new_error(BasicParseErrorKind::EndOfInput);
        }
        ParseError {
            kind: ParseErrorKind::Custom(Box::new(error.into())),
            location: self,
        }
    }
    pub(crate) fn new_unexpected_token_error<E>(self, token: Token<'i>) -> ParseError<'i, E> {
        self.new_basic_unexpected_token_error(token).into()
    }
    pub(crate) fn new_basic_unexpected_token_error(self, token: Token<'i>) -> BasicParseError<'i> {
        BasicParseError {
            kind: BasicParseErrorKind::UnexpectedToken(token),
            location: self,
        }
    }
    pub(crate) fn new_error<E>(self, kind: BasicParseErrorKind<'i>) -> ParseError<'i, E> {
        BasicParseError {
            kind,
            location: self,
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ParserState<'i> {
    position: Site<'i>,
    pending: Option<usize>,
    start: usize,
    end: usize,
}
impl<'i> ParserState<'i> {
    pub(crate) fn position(&self) -> Site<'i> {
        self.position
    }
}

pub(crate) type Parser<'i, 't> = PropertyParser<'i, 't>;
pub(crate) struct PropertyParser<'i, 't> {
    document: &'i SyntaxDocument<'i>,
    list: ListId,
    start: usize,
    next: usize,
    end: usize,
    pending: Option<usize>,
    token: Option<Token<'i>>,
    lifetime: PhantomData<&'t mut ()>,
}
impl<'i, 't> PropertyParser<'i, 't> {
    pub(crate) fn control(&self) -> Option<&'i Control<'i>> {
        self.document.control
    }
    pub(crate) fn controlled_step(&self) -> Result<(), BasicParseError<'i>> {
        if self
            .control()
            .is_some_and(|control| control.step().is_err())
        {
            return Err(BasicParseError {
                kind: BasicParseErrorKind::EndOfInput,
                location: self.position(),
            });
        }
        Ok(())
    }
    pub(crate) fn reserve_controlled<T>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), BasicParseError<'i>> {
        self.controlled_step()?;
        if let Some(control) = self.control() {
            control
                .reserve(values, additional)
                .map_err(|_| BasicParseError {
                    kind: BasicParseErrorKind::EndOfInput,
                    location: self.position(),
                })
        } else {
            values.try_reserve(additional).map_err(|_| BasicParseError {
                kind: BasicParseErrorKind::EndOfInput,
                location: self.position(),
            })
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
            lifetime: PhantomData,
        }
    }
    pub(crate) fn position(&self) -> Site<'i> {
        Site {
            document: self.document,
            list: self.list,
            index: self.next,
            closing: false,
        }
    }
    pub(crate) fn current_source_location(&self) -> Site<'i> {
        self.position()
    }
    pub(crate) fn state(&self) -> ParserState<'i> {
        ParserState {
            position: self.position(),
            pending: self.pending,
            start: self.start,
            end: self.end,
        }
    }
    pub(crate) fn reset(&mut self, state: &ParserState<'i>) {
        assert!(
            std::ptr::eq(self.document, state.position.document)
                && self.list == state.position.list
                && self.start == state.start
                && self.end == state.end
                && (self.start..=self.end).contains(&state.position.index)
        );
        self.next = state.position.index;
        self.pending = state.pending;
        self.token = None;
    }
    pub(crate) fn try_parse<F, T, E>(&mut self, parse: F) -> Result<T, E>
    where
        F: FnOnce(&mut Self) -> Result<T, E>,
    {
        let state = self.state();
        let result = parse(self);
        if result.is_err() {
            self.reset(&state);
        }
        result
    }
    pub(crate) fn skip_whitespace(&mut self) {
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
    pub(crate) fn is_exhausted(&mut self) -> bool {
        if self.controlled_step().is_err() {
            return true;
        }
        let state = self.state();
        self.skip_whitespace();
        let result = self.next == self.end;
        self.reset(&state);
        result
    }
    pub(crate) fn next_kind(
        &mut self,
        include_trivia: bool,
    ) -> Result<TokenKind<'i>, BasicParseError<'i>> {
        if !include_trivia {
            self.skip_whitespace();
        }
        self.controlled_step()?;
        if self.next == self.end {
            return Err(BasicParseError {
                kind: BasicParseErrorKind::EndOfInput,
                location: self.position(),
            });
        }
        let id = self.document.lists[self.list][self.next];
        self.next += 1;
        let node = &self.document.nodes[id];
        self.pending = node.children().map(|_| id);
        Ok(node.token().kind())
    }
    pub(crate) fn next(&mut self) -> Result<&Token<'i>, BasicParseError<'i>> {
        self.skip_whitespace();
        self.next_including_whitespace_and_comments()
    }
    pub(crate) fn next_including_whitespace_and_comments(
        &mut self,
    ) -> Result<&Token<'i>, BasicParseError<'i>> {
        self.controlled_step()?;
        let location = self.position();
        if self.next == self.end {
            return Err(BasicParseError {
                kind: BasicParseErrorKind::EndOfInput,
                location,
            });
        }
        let id = self.document.lists[self.list][self.next];
        self.next += 1;
        let node = &self.document.nodes[id];
        self.pending = node.children().map(|_| id);
        self.token = Some(native_token(node.token()));
        Ok(self.token.as_ref().expect("selected token"))
    }
    pub(crate) fn expect_exhausted(&mut self) -> Result<(), BasicParseError<'i>> {
        if self.control().is_some() {
            let state = self.state();
            self.skip_whitespace();
            let location = self.position();
            let result = match self.next_kind(false) {
                Ok(_) => Err(BasicParseError {
                    kind: BasicParseErrorKind::EndOfInput,
                    location,
                }),
                Err(error)
                    if self
                        .control()
                        .is_some_and(|control| control.failure().is_some()) =>
                {
                    Err(error)
                }
                Err(_) => Ok(()),
            };
            self.reset(&state);
            return result;
        }
        let state = self.state();
        self.skip_whitespace();
        let site = self.position();
        let result = match self.next() {
            Ok(token) => Err(site.new_basic_unexpected_token_error(token.clone())),
            Err(_) => Ok(()),
        };
        self.reset(&state);
        result
    }
    pub(crate) fn new_unexpected_token_error<E>(&self, token: Token<'i>) -> ParseError<'i, E> {
        self.position().new_unexpected_token_error(token)
    }
    pub(crate) fn new_error<E>(&self, kind: BasicParseErrorKind<'i>) -> ParseError<'i, E> {
        self.position().new_error(kind)
    }
    fn expect_token(
        &mut self,
        predicate: impl FnOnce(&Token<'i>) -> bool,
    ) -> Result<(), BasicParseError<'i>> {
        self.skip_whitespace();
        let site = self.position();
        let token = self.next()?;
        if predicate(token) {
            Ok(())
        } else {
            Err(site.new_basic_unexpected_token_error(token.clone()))
        }
    }
    pub(crate) fn expect_ident(&mut self) -> Result<&CowRcStr<'i>, BasicParseError<'i>> {
        self.skip_whitespace();
        let site = self.position();
        if self.control().is_some() {
            match self.next_kind(false)? {
                TokenKind::Ident(name) => {
                    self.token = Some(Token::Ident(name.into()));
                }
                _ => {
                    return Err(BasicParseError {
                        kind: BasicParseErrorKind::EndOfInput,
                        location: site,
                    });
                }
            }
            let Some(Token::Ident(name)) = self.token.as_ref() else {
                unreachable!("actual checked identifier")
            };
            return Ok(name);
        }
        match self.next()? {
            Token::Ident(name) => Ok(name),
            token => Err(site.new_basic_unexpected_token_error(token.clone())),
        }
    }
    pub(crate) fn expect_ident_cloned(&mut self) -> Result<CowRcStr<'i>, BasicParseError<'i>> {
        self.expect_ident().cloned()
    }
    pub(crate) fn expect_ident_matching(
        &mut self,
        expected: &str,
    ) -> Result<(), BasicParseError<'i>> {
        self.skip_whitespace();
        let site = self.position();
        let ident = self.expect_ident()?;
        if ident.eq_ignore_ascii_case(expected) {
            Ok(())
        } else {
            Err(site.new_basic_unexpected_token_error(Token::Ident(ident.clone())))
        }
    }
    pub(crate) fn expect_function(&mut self) -> Result<&CowRcStr<'i>, BasicParseError<'i>> {
        self.skip_whitespace();
        let site = self.position();
        match self.next()? {
            Token::Function(name) => Ok(name),
            token => Err(site.new_basic_unexpected_token_error(token.clone())),
        }
    }
    pub(crate) fn expect_function_matching(
        &mut self,
        expected: &str,
    ) -> Result<(), BasicParseError<'i>> {
        self.expect_token(
            |token| matches!(token, Token::Function(name) if name.eq_ignore_ascii_case(expected)),
        )
    }
    pub(crate) fn expect_string_cloned(&mut self) -> Result<CowRcStr<'i>, BasicParseError<'i>> {
        self.skip_whitespace();
        let site = self.position();
        match self.next()? {
            Token::QuotedString(value) => Ok(value.clone()),
            token => Err(site.new_basic_unexpected_token_error(token.clone())),
        }
    }
    pub(crate) fn expect_delim(&mut self, expected: char) -> Result<(), BasicParseError<'i>> {
        self.expect_token(|token| matches!(token, Token::Delim(value) if *value == expected))
    }
    pub(crate) fn expect_comma(&mut self) -> Result<(), BasicParseError<'i>> {
        self.expect_token(|token| matches!(token, Token::Comma))
    }
    #[cfg(test)]
    pub(crate) fn expect_semicolon(&mut self) -> Result<(), BasicParseError<'i>> {
        self.expect_token(|token| matches!(token, Token::Semicolon))
    }
    pub(crate) fn parse_entirely<F, T, E>(&mut self, parse: F) -> Result<T, ParseError<'i, E>>
    where
        F: FnOnce(&mut Self) -> Result<T, ParseError<'i, E>>,
    {
        let value = parse(self)?;
        self.expect_exhausted()?;
        Ok(value)
    }
    pub(crate) fn parse_nested_block<F, T, E>(&mut self, parse: F) -> Result<T, ParseError<'i, E>>
    where
        F: FnOnce(&mut Parser<'i, '_>) -> Result<T, ParseError<'i, E>>,
    {
        let Some(id) = self.pending.take() else {
            return Err(self.new_error(BasicParseErrorKind::EndOfInput));
        };
        let children = self.document.nodes[id].children().expect("pending group");
        let mut nested = Parser::bounded(
            self.document,
            children,
            0..self.document.lists[children].len(),
        );
        let result = parse(&mut nested);
        result.and_then(|value| {
            nested.expect_exhausted()?;
            Ok(value)
        })
    }
    pub(crate) fn parse_until_before<F, T, E>(
        &mut self,
        delimiters: Delimiters,
        parse: F,
    ) -> Result<T, ParseError<'i, E>>
    where
        F: FnOnce(&mut Parser<'i, '_>) -> Result<T, ParseError<'i, E>>,
    {
        let end = (self.next..self.end)
            .find(|index| {
                matches_delimiter(
                    self.document.nodes[self.document.lists[self.list][*index]]
                        .token()
                        .kind(),
                    delimiters,
                )
            })
            .unwrap_or(self.end);
        let mut nested = Parser::bounded(self.document, self.list, self.next..end);
        let result = parse(&mut nested).and_then(|value| {
            nested.expect_exhausted()?;
            Ok(value)
        });
        self.next = end;
        self.pending = None;
        result
    }
    /// The shared Values 4 §2.3 `#` multiplier: one or more items separated by
    /// commas. Each item parser is bounded at its comma and must consume its
    /// whole item; whole-list rules stay with the caller as post-conditions.
    pub(crate) fn parse_comma_separated<F, T, E>(
        &mut self,
        mut parse: F,
    ) -> Result<Vec<T>, ParseError<'i, E>>
    where
        F: FnMut(&mut Parser<'i, '_>) -> Result<T, ParseError<'i, E>>,
    {
        let mut values = Vec::new();
        loop {
            let value = self.parse_until_before(Delimiter::Comma, &mut parse)?;
            self.reserve_controlled(&mut values, 1)?;
            values.push(value);
            if self.is_exhausted() {
                return Ok(values);
            }
            self.expect_comma()?;
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
        self.skip_whitespace();
        if self.next == self.end {
            let origin = match self.position().origin() {
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
        let values = crate::component_values::promote_nodes(
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
        _numeric: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<CssComponentValue, crate::CssNumericConstructionError> {
        self.collect_component()
            .map(Cow::into_owned)
            .map_err(crate::CssNumericConstructionError::component)
    }
    /// Owned text for retained output only. Grammar never consumes this string.
    pub(crate) fn slice(&self, range: Range<Site<'i>>) -> String {
        assert!(
            std::ptr::eq(range.start.document, self.document)
                && std::ptr::eq(range.end.document, self.document)
                && range.start.list == self.list
                && range.end.list == self.list
        );
        if let Some(source) = &self.document.source {
            let start = self
                .document
                .boundary(self.list, range.start.index)
                .source
                .expect("source boundary")
                .offset;
            let end = self
                .document
                .boundary(self.list, range.end.index)
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
        let mut output = crate::component_values::CssCanonicalBuilder::new(usize::MAX);
        for id in &self.document.lists[self.list][range.start.index..range.end.index] {
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
    pub(crate) fn slice_from(&self, start: Site<'i>) -> String {
        self.slice(start..self.position())
    }
}
fn implicit_end_origin(
    opening: &CssValueOrigin,
    at: &crate::syntax_consumption::SyntaxBoundary,
) -> CssSerializedOrigin {
    let origin = match (opening, &at.source) {
        (CssValueOrigin::Parsed(opening), Some(source)) => {
            crate::CssParsedOrigin::from_range(&source.snapshot, source.offset..source.offset)
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
fn matches_delimiter(kind: TokenKind<'_>, delimiters: Delimiters) -> bool {
    let wanted = match kind {
        TokenKind::Semicolon => Delimiter::Semicolon,
        TokenKind::Comma => Delimiter::Comma,
        TokenKind::Delim('!') => Delimiter::Bang,
        TokenKind::Opening(CssBlockKind::CurlyBracket) => Delimiter::CurlyBracketBlock,
        _ => return false,
    };
    delimiters | wanted == delimiters
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
    use crate::syntax_consumption::{SyntaxInput, SyntaxInputLimits, normalize, source_document};
    use crate::{CssComponentValues, CssSourceSnapshot, parse_component_values};

    #[test]
    fn independent_checked_leaves_do_not_merge_into_a_new_token() {
        let values = CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("a").unwrap(),
            CssComponentValue::try_ident("b").unwrap(),
        ])
        .unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let mut input = Parser::new(&document);
        assert_eq!(input.expect_ident_cloned().unwrap().as_ref(), "a");
        assert_eq!(input.expect_ident_cloned().unwrap().as_ref(), "b");
        assert!(input.is_exhausted());
    }

    #[test]
    fn checked_collection_borrows_the_exact_original_component() {
        let values = parse_component_values("calc(1px + 2px)").unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let mut input = Parser::new(&document);
        let value = input.collect_component().unwrap();
        assert!(matches!(value, Cow::Borrowed(_)));
        assert!(std::ptr::eq(value.as_ref(), &values.items()[0]));
    }

    #[test]
    fn failed_nested_parse_advances_outer_and_reset_restores_the_pending_group() {
        let snapshot = CssSourceSnapshot::new("f(a b) tail");
        let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
        let mut input = Parser::new(&document);
        input.expect_function_matching("f").unwrap();
        let checkpoint = input.state();
        let error = input
            .parse_nested_block(|nested| {
                nested.expect_ident_matching("a")?;
                nested.expect_ident_matching("wrong")?;
                Ok::<_, ParseError<'_, ()>>(())
            })
            .unwrap_err();
        assert_eq!(error.location.lexeme(), Some("b"));
        assert_eq!(input.expect_ident_cloned().unwrap().as_ref(), "tail");
        input.reset(&checkpoint);
        input
            .parse_nested_block(|nested| {
                nested.expect_ident_matching("a")?;
                nested.expect_ident_matching("b")?;
                Ok::<_, ParseError<'_, ()>>(())
            })
            .unwrap();
        assert_eq!(input.expect_ident_cloned().unwrap().as_ref(), "tail");
    }

    #[test]
    fn lexical_error_leaves_remain_visible_until_grammar_visits_them() {
        let snapshot = CssSourceSnapshot::new("url(a b)!");
        let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
        let mut input = Parser::new(&document);
        assert!(matches!(input.next().unwrap(), Token::BadUrl(_)));
        assert_eq!(input.next().unwrap(), &Token::Delim('!'));
    }

    #[test]
    fn descendant_sites_select_real_mixed_source_children_and_closing_boundaries() {
        let children = parse_component_values("\n 2px").unwrap();
        let origin = children.items()[1].origin().clone();
        let group = CssComponentValue::try_function("f", children).unwrap();
        let values = CssComponentValues::try_new(vec![group]).unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let input = Parser::new(&document);
        let child = input.position().descendant(&[0, 1], false).unwrap();
        assert_eq!(child.value_origin(), Some(origin));
        assert!(matches!(child.token(), Some(Token::Dimension { .. })));
        let closing = input.position().descendant(&[0], true).unwrap();
        assert_eq!(closing.value_origin(), Some(CssValueOrigin::Programmatic));
        assert_eq!(closing.token(), Some(Token::CloseParenthesis));
    }

    #[test]
    fn nested_end_reports_the_actual_closing_token_and_root_end_reports_eof() {
        let values = parse_component_values("f()").unwrap();
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
        let mut input = Parser::new(&document);
        input.expect_function_matching("f").unwrap();
        let nested = input
            .parse_nested_block(|nested| {
                nested
                    .expect_ident_cloned()
                    .map_err(crate::error::property::basic)
            })
            .unwrap_err();
        let (kind, origin) = nested.into_parts();
        assert!(matches!(kind, crate::ErrorKind::UnexpectedEnd(_)));
        assert_eq!(origin, CssSerializedOrigin::Token(closing.clone()));
        let (kind, origin) =
            crate::error::property::basic(input.expect_ident_cloned().unwrap_err()).into_parts();
        assert!(matches!(kind, crate::ErrorKind::UnexpectedEnd(_)));
        assert_eq!(origin, CssSerializedOrigin::End(Some(closing)));
    }

    #[test]
    fn bounded_delimiter_parse_keeps_the_real_outer_delimiter() {
        let snapshot = CssSourceSnapshot::new("a f(b;c);tail");
        let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
        let mut input = Parser::new(&document);
        input
            .parse_until_before(Delimiter::Semicolon, |nested| {
                nested.expect_ident_matching("a")?;
                nested.expect_function_matching("f")?;
                Ok::<_, ParseError<'_, ()>>(())
            })
            .unwrap();
        assert_eq!(input.position().lexeme(), Some(";"));
        input.expect_semicolon().unwrap();
        assert_eq!(input.expect_ident_cloned().unwrap().as_ref(), "tail");
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
    numeric: &crate::numeric::NumericInputContext<'n>,
    range: NativeValueRange,
    parse: F,
) -> Result<T, cssparser::ParseError<'s, crate::Error>>
where
    F: for<'i, 't> FnOnce(
        &mut Parser<'i, 't>,
        &crate::numeric::NumericInputContext<'n>,
    ) -> Result<T, ParseError<'i, crate::error::property::Error>>,
{
    use crate::numeric::NumericInputContext;
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
                crate::error::invalid_component_value(input.current_source_location(), detail)
            })?;
            let mut parser = Parser::new(&document);
            let result = parse(&mut parser, numeric);
            let consumed = document
                .boundary(document.root, parser.position().ordinal())
                .source
                .expect("source cursor boundary")
                .offset;
            advance_native_to(input, consumed);
            result.map_err(ParseError::into_native_source)
        }
        NumericInputContext::Components(values, serialized) => {
            let paths = serialized
                .component_paths_in_range(start..end)
                .ok_or_else(|| crate::error::invalid_syntax(input.current_source_location()))?;
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
                crate::error::invalid_component_value(input.current_source_location(), detail)
            })?;
            let mut parser = Parser::new(&document);
            let result = parse(&mut parser, numeric);
            let ordinal = parser.position().ordinal();
            let consumed = paths
                .get(ordinal)
                .and_then(|path| serialized.component_offset_for_path(path))
                .unwrap_or(end);
            advance_native_to(input, consumed);
            result.map_err(|error| {
                let error = error.qualify();
                let site = error.location;
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
    use crate::numeric::NumericInputContext;
    use crate::{CssComponentValues, CssSourceSnapshot, parse_component_values};

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
                input.expect_function_matching("f")?;
                input.parse_nested_block(|nested| {
                    nested.expect_ident_matching("a")?;
                    Ok::<_, ParseError<'_, crate::error::property::Error>>(42)
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
        let values = CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("a").unwrap(),
            CssComponentValue::try_ident("b").unwrap(),
        ])
        .unwrap();
        let serialized = values.serialize().unwrap();
        let mut storage = cssparser::ParserInput::new(serialized.as_css());
        let mut native = cssparser::Parser::new(&mut storage);
        let address = adapt_native(
            &mut native,
            &NumericInputContext::Components(&values, &serialized),
            NativeValueRange::Component,
            |input, _| {
                let component = input.collect_component().map_err(|error| {
                    crate::error::property::invalid_component_value(input.position(), error)
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
        let values = parse_component_values("f(a wrong)").unwrap();
        let serialized = values.serialize().unwrap();
        let mut storage = cssparser::ParserInput::new(serialized.as_css());
        let mut native = cssparser::Parser::new(&mut storage);
        let error = adapt_native(
            &mut native,
            &NumericInputContext::Components(&values, &serialized),
            NativeValueRange::Component,
            |input, _| {
                input.expect_function_matching("f")?;
                input.parse_nested_block(|nested| {
                    nested.expect_ident_matching("a")?;
                    nested.expect_ident_matching("other")?;
                    Ok::<_, ParseError<'_, crate::error::property::Error>>(())
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
