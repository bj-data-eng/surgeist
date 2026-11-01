//! CSS's reading of the parser crate's token tree: provenance, metering and
//! retained text for the grammars, and the adapter by which the grammars the
//! provider still drives enter them.
use crate::component::values::CssComponentValueErrorKind;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValueRef,
    CssSerializedOrigin, CssValueOrigin,
};
use crate::parser::syntax_types::{Attached, Document, Site, Stream, SyntaxError};
use crate::source::{CssParsedOrigin, CssSourcePosition, CssSourceSnapshot};
use crate::syntax_consumption::DocumentExt;
use crate::values::color_adjust::utf16_value::Control;
use std::borrow::Cow;
use surgeist_css_parser::document::{GroupEnd, ListId, NodeId};
#[cfg(test)]
use surgeist_css_parser::stream::StopToken;
use surgeist_css_parser::{Offset, Token};

/// The site of the boundary before `index` in `list`, or the group's closing
/// boundary when `closing`.
fn site_at<'i>(
    document: &'i Document<'i>,
    list: ListId,
    index: usize,
    closing: bool,
) -> Option<Site<'i>> {
    if closing {
        document.closing_site(*document.list(list).get(index)?)
    } else {
        document.site(list, index)
    }
}

/// What CSS reads of a boundary of a token tree that the tree itself does not
/// say: where its tokens came from, and the metering of the control that owns
/// it.
pub(crate) trait SiteExt<'i>: Sized {
    fn control(self) -> Option<&'i Control<'i>>;
    /// The checked component whose token the boundary sits at, and whether the
    /// boundary is its closing token.
    fn checked_component(self) -> Option<(&'i CssComponentValue, bool)>;
    fn value_origin(self) -> Option<CssValueOrigin>;
    fn descendant(self, path: &[usize], closing: bool) -> Option<Self>;
    /// Whether an operator follows this component in its list, so that it
    /// starts a multi-term calculation.
    fn begins_expression(self) -> bool;
    /// The end of this component's list when it is an operator that follows
    /// an operand and that no further component follows: the operand it
    /// requires is missing there. An operator that starts its argument is
    /// itself responsible.
    fn trailing_operator_end(self) -> Option<Self>;
    /// The end of the group at `path`'s children, before its closing
    /// boundary: where a function cut short of its grammar ends.
    fn descendant_children_end(self, path: &[usize]) -> Option<Self>;
    /// Locate an explicit source fault only within this boundary's original snapshot.
    /// Checked input deliberately has no reverse lookup by origin identity.
    fn source_origin(self, origin: &CssValueOrigin) -> Option<Self>;
    fn component_path(self) -> Vec<usize>;
    /// Adjacent lexical candidates in this actual component list. An enclosing
    /// close is a boundary candidate; it never exposes a following sibling.
    fn diagnostic_candidates(self) -> (Option<Self>, Option<Self>);
    fn lexeme(self) -> Option<&'i str>;
    fn source_position(self) -> Option<CssSourcePosition>;
    fn origin(self) -> CssSerializedOrigin;
}

fn step(control: Option<&Control<'_>>) -> bool {
    control.is_none_or(|control| control.step().is_ok())
}

fn operator(token: Option<Token<'_>>) -> bool {
    matches!(token, Some(Token::Delim('+' | '-' | '*' | '/')))
}

impl<'i> SiteExt<'i> for Site<'i> {
    fn control(self) -> Option<&'i Control<'i>> {
        crate::parser::syntax_types::host(self.document())?.control()
    }

    fn checked_component(self) -> Option<(&'i CssComponentValue, bool)> {
        match self.document().payload(self.node()?)? {
            Attached::Component(component) => Some((*component, self.is_closing())),
            Attached::Host(_) => None,
        }
    }

    fn value_origin(self) -> Option<CssValueOrigin> {
        match self.origin() {
            CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => {
                Some(origin)
            }
            _ => None,
        }
    }

    fn descendant(self, path: &[usize], closing: bool) -> Option<Self> {
        let document = self.document();
        let (mut list, mut index) = (self.list(), self.index());
        for (depth, step_index) in path.iter().copied().enumerate() {
            if !step(self.control()) {
                return None;
            }
            if depth == 0 {
                index = index.checked_add(step_index)?;
            } else {
                list = document.children(*document.list(list).get(index)?)?;
                index = step_index;
            }
            document.list(list).get(index)?;
        }
        site_at(document, list, index, closing)
    }

    fn begins_expression(self) -> bool {
        let document = self.document();
        let nodes = document.list(self.list());
        let Some(index) = (self.index() + 1..nodes.len())
            .find(|index| !document.token(nodes[*index]).is_trivia())
        else {
            return false;
        };
        operator(Some(document.token(nodes[index])))
    }

    fn trailing_operator_end(self) -> Option<Self> {
        if !operator(self.token()) {
            return None;
        }
        let document = self.document();
        let nodes = document.list(self.list());
        let operand = (0..self.index())
            .rev()
            .find(|index| !document.token(nodes[*index]).is_trivia())
            .is_some_and(|index| document.token(nodes[index]) != Token::Comma);
        if !operand {
            return None;
        }
        (self.index() + 1..nodes.len())
            .all(|index| document.token(nodes[index]).is_trivia())
            .then(|| document.site(self.list(), nodes.len()))
            .flatten()
    }

    fn descendant_children_end(self, path: &[usize]) -> Option<Self> {
        let document = self.document();
        let group = self.descendant(path, false)?;
        let list = document.children(group.node()?)?;
        document.site(list, document.list(list).len())
    }

    fn source_origin(self, origin: &CssValueOrigin) -> Option<Self> {
        let document = self.document();
        document.source()?;
        let CssValueOrigin::Parsed(origin) = origin else {
            return None;
        };
        if !document.snapshot()?.same_snapshot(origin.source()) {
            return None;
        }
        if !step(self.control()) {
            return None;
        }
        document.node_at_offset(Offset::new(origin.span().start().byte_offset().value()))
    }

    fn component_path(self) -> Vec<usize> {
        let document = self.document();
        let mut path = vec![self.index()];
        let mut list = self.list();
        while let Some(parent) = document.parent_group(list) {
            let (parent_list, index) = document.location(parent);
            path.push(index);
            list = parent_list;
        }
        path.reverse();
        path
    }

    fn diagnostic_candidates(self) -> (Option<Self>, Option<Self>) {
        let document = self.document();
        let context = if self.is_closing() {
            self.node()
                .and_then(|node| document.children(node))
                .and_then(|list| document.site(list, document.list(list).len()))
                .unwrap_or(self)
        } else {
            self
        };
        let nodes = document.list(context.list());
        let mut next = None;
        for (index, node) in nodes.iter().enumerate().skip(context.index()) {
            if !step(self.control()) {
                return (None, None);
            }
            if !document.token(*node).is_trivia() {
                next = document.site(context.list(), index);
                break;
            }
        }
        if next.is_none()
            && let Some(parent) = document.parent_group(context.list())
        {
            if !step(self.control()) {
                return (None, None);
            }
            next = document.closing_site(parent);
        }
        let mut previous = None;
        for index in (0..context.index()).rev() {
            if !step(self.control()) {
                return (None, None);
            }
            if !document.token(nodes[index]).is_trivia() {
                previous = site_at(
                    document,
                    context.list(),
                    index,
                    document.children(nodes[index]).is_some(),
                );
                break;
            }
        }
        (next, previous)
    }

    fn lexeme(self) -> Option<&'i str> {
        if let Some((component, closing)) = self.checked_component() {
            if closing
                && !matches!(
                    self.document().group_end(self.node()?)?,
                    GroupEnd::Closed { .. }
                )
            {
                return None;
            }
            return component.structural_lexeme(closing).map(|(text, _)| text);
        }
        self.spelling()
    }

    fn source_position(self) -> Option<CssSourcePosition> {
        let document = self.document();
        let snapshot = document.snapshot();
        if self.is_closing()
            && let Some(node) = self.node()
            && let Some(GroupEnd::Eof { at: Some(at) }) = document.group_end(node)
            && let Some(snapshot) = snapshot
        {
            return Some(CssSourcePosition::from_byte_offset_in(
                snapshot.as_str(),
                at.value(),
            ));
        }
        if let Some(origin) = self.token_origin() {
            match origin {
                CssValueOrigin::Parsed(origin) => return Some(origin.span().start()),
                CssValueOrigin::ImplicitClosure { at, .. } => return Some(at.span().start()),
                _ => {}
            }
        }
        let offset = document.boundary(self.list(), self.index()).offset?;
        Some(CssSourcePosition::from_byte_offset_in(
            snapshot?.as_str(),
            offset,
        ))
    }

    fn origin(self) -> CssSerializedOrigin {
        let document = self.document();
        if self.is_closing()
            && let Some(node) = self.node()
        {
            if let Some((component, _)) = self.checked_component() {
                let closing = match component.view() {
                    CssComponentValueRef::Function(function) => function.closing_origin(),
                    CssComponentValueRef::Block(block) => block.closing_origin(),
                    _ => unreachable!("closing bookmark belongs to an actual group"),
                };
                return CssSerializedOrigin::Token(closing.clone());
            }
            if matches!(document.group_end(node), Some(GroupEnd::Eof { .. })) {
                return group_end_origin(document, node);
            }
        }
        if let Some(origin) = self.token_origin() {
            return CssSerializedOrigin::Token(origin);
        }
        // A child EOF is the original closing boundary, even for empty groups.
        if self.index() == document.list(self.list()).len()
            && let Some(group) = document.parent_group(self.list())
        {
            return group_end_origin(document, group);
        }
        let previous = self
            .index()
            .checked_sub(1)
            .and_then(|index| document.list(self.list()).get(index))
            .copied();
        CssSerializedOrigin::End(previous.map(|node| {
            if document.children(node).is_some() {
                match group_end_origin(document, node) {
                    CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => {
                        origin
                    }
                    _ => unreachable!("an ended group retains its closing or opening origin"),
                }
            } else {
                document.origin(node)
            }
        }))
    }
}

/// Site reads that are not part of the provenance contract.
trait SitePrivate<'i>: Sized {
    /// The origin of the token the boundary sits at: the node's own token, or
    /// the closing token of a closing boundary, when there is one.
    fn token_origin(self) -> Option<CssValueOrigin>;
}

impl<'i> SitePrivate<'i> for Site<'i> {
    fn token_origin(self) -> Option<CssValueOrigin> {
        let document = self.document();
        let node = self.node()?;
        if !self.is_closing() {
            return Some(document.origin(node));
        }
        match document.group_end(node)? {
            GroupEnd::Closed { .. } => Some(document.closing_origin(node)),
            _ => None,
        }
    }
}

/// Where the group `node` ended: at its closing token, or at the end of its
/// input.
fn group_end_origin(document: &Document<'_>, node: NodeId) -> CssSerializedOrigin {
    match document.group_end(node) {
        Some(GroupEnd::Closed { .. }) => CssSerializedOrigin::Token(document.closing_origin(node)),
        _ => {
            let (opening, at) = match document.payload(node) {
                Some(Attached::Component(component)) => (
                    component.origin().clone(),
                    match component.structural_lexeme(true).map(|(_, origin)| origin) {
                        Some(CssValueOrigin::ImplicitClosure { at, .. }) => {
                            Some((at.source().clone(), at.span().start().byte_offset().value()))
                        }
                        _ => None,
                    },
                ),
                _ => (
                    document.origin(node),
                    match (document.snapshot(), document.group_end(node)) {
                        (Some(snapshot), Some(GroupEnd::Eof { at: Some(at) })) => {
                            Some((snapshot.clone(), at.value()))
                        }
                        _ => None,
                    },
                ),
            };
            implicit_end_origin(&opening, at)
        }
    }
}

fn implicit_end_origin(
    opening: &CssValueOrigin,
    at: Option<(CssSourceSnapshot, usize)>,
) -> CssSerializedOrigin {
    let origin = match (opening, at) {
        (CssValueOrigin::Parsed(opening), Some((snapshot, offset))) => {
            CssParsedOrigin::from_range(&snapshot, offset..offset)
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

/// What CSS asks of a stream beyond the parser crate's own operations: the
/// control that meters it, checked components, and retained text.
pub(crate) trait StreamExt<'i> {
    fn control(&self) -> Option<&'i Control<'i>>;
    /// Takes one step of the control, as an observation that has no stream
    /// operation of its own does.
    fn controlled_step<E>(&self) -> Result<(), SyntaxError<'i, E>>;
    /// Makes room in `values` for `additional` more, charging the control.
    fn reserve_controlled<T, E>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), SyntaxError<'i, E>>;
    fn collect_component(&mut self) -> Result<Cow<'i, CssComponentValue>, CssComponentValueError>;
    /// The components of every token the stream still holds, as a document of
    /// source text reads them; the stream does not move.
    fn remaining_components(
        &self,
    ) -> Result<crate::component::CssComponentValues, CssComponentValueError>;
    fn collect_numeric_component(
        &mut self,
        numeric: &crate::numeric::expression::NumericInputContext<'_>,
    ) -> Result<CssComponentValue, crate::numeric::CssNumericConstructionError>;
    /// Owned text for retained output only. Grammar never consumes this string.
    fn authored_text_between(&self, from: Site<'i>, to: Site<'i>) -> String;
    fn authored_text_since(&self, from: Site<'i>) -> String;
}

impl<'i> StreamExt<'i> for Stream<'i> {
    fn control(&self) -> Option<&'i Control<'i>> {
        self.site().control()
    }

    fn controlled_step<E>(&self) -> Result<(), SyntaxError<'i, E>> {
        if self
            .control()
            .is_some_and(|control| control.step().is_err())
        {
            return Err(self.end_of_stream());
        }
        Ok(())
    }

    fn reserve_controlled<T, E>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), SyntaxError<'i, E>> {
        self.controlled_step()?;
        let reserved = match self.control() {
            Some(control) => control.reserve(values, additional).map_err(|_| ()),
            None => values.try_reserve(additional).map_err(|_| ()),
        };
        reserved.map_err(|()| self.end_of_stream())
    }

    fn collect_component(&mut self) -> Result<Cow<'i, CssComponentValue>, CssComponentValueError> {
        self.controlled_step::<()>().map_err(|_| {
            CssComponentValueError::new(
                CssComponentValueErrorKind::CapacityOverflow,
                CssValueOrigin::Programmatic,
            )
        })?;
        self.discard_whitespace();
        let site = self.site();
        let Ok(component) = self.consume_component_value() else {
            // No component at the end of source text is an absent one, at the end.
            let absent = site
                .offset()
                .filter(|_| site.node().is_none())
                .and_then(|offset| {
                    use crate::syntax_consumption::DocumentExt as _;
                    let snapshot = site.document().snapshot()?;
                    CssParsedOrigin::from_range(snapshot, offset.value()..offset.value())
                })
                .map(CssValueOrigin::Parsed);
            let origin = match (absent, site.origin()) {
                (Some(origin), _) => origin,
                (
                    _,
                    CssSerializedOrigin::End(Some(origin)) | CssSerializedOrigin::Token(origin),
                ) => origin,
                _ => CssValueOrigin::Programmatic,
            };
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                origin,
            ));
        };
        let node = component.node();
        if let Some(Attached::Component(component)) = component.payload() {
            return Ok(Cow::Borrowed(component));
        }
        let document: &'i Document<'i> = site.document();
        let values = crate::component::values::parse::promote_nodes(
            document,
            &[node],
            CssComponentValueLimits::default(),
        )?;
        values.into_single().map(Cow::Owned).ok_or_else(|| {
            CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                document.origin(node),
            )
        })
    }

    fn remaining_components(
        &self,
    ) -> Result<crate::component::CssComponentValues, CssComponentValueError> {
        let (from, to) = (self.site(), self.end_site());
        let document = from.document();
        let nodes = &document.list(from.list())[from.index()..to.index()];
        crate::component::values::parse::promote_run(document, nodes)
    }

    fn collect_numeric_component(
        &mut self,
        _numeric: &crate::numeric::expression::NumericInputContext<'_>,
    ) -> Result<CssComponentValue, crate::numeric::CssNumericConstructionError> {
        self.collect_component()
            .map(Cow::into_owned)
            .map_err(crate::numeric::CssNumericConstructionError::component)
    }

    fn authored_text_between(&self, from: Site<'i>, to: Site<'i>) -> String {
        if let Some(text) = self.text_between(from, to) {
            return match self.control() {
                Some(control) => control.string(text).unwrap_or_default(),
                None => text.to_owned(),
            };
        }
        assert!(
            self.control().is_none(),
            "controlled property slices retain their actual source"
        );
        let document: &Document<'_> = from.document();
        let mut output = crate::component::values::serialize::CssCanonicalBuilder::new(usize::MAX);
        for node in &document.list(from.list())[from.index()..to.index()] {
            let Some(Attached::Component(component)) = document.payload(*node) else {
                unreachable!("checked property output borrows checked components");
            };
            output
                .push_components(std::slice::from_ref(component))
                .expect("checked output boundaries");
        }
        output.into_text()
    }

    fn authored_text_since(&self, from: Site<'i>) -> String {
        self.authored_text_between(from, self.site())
    }
}

/// Runs a stream grammar over the whole scope the native parser `input` was
/// handed, to its end: the contents of a rule block or the value of a
/// declaration, as the source text `source` holds, and leaves `input` after what
/// the grammar consumed. Only the native `AtRuleParser`, `QualifiedRuleParser`
/// and `DeclarationParser` methods whose grammar has not been ported enter a
/// ported grammar this way.
pub(crate) fn adapt_native<'s, T, F>(
    input: &mut cssparser::Parser<'s, '_>,
    source: &crate::source::CssSourceSnapshot,
    parse: F,
) -> Result<T, cssparser::ParseError<'s, crate::diagnostic::CssError>>
where
    F: for<'i> FnOnce(
        &mut Stream<'i>,
    ) -> Result<T, SyntaxError<'i, crate::diagnostic::error::property::Error>>,
{
    use crate::diagnostic::error::property::SyntaxErrorExt;
    use crate::syntax_consumption::source_scope_document;
    let start = input.position().byte_index();
    let end = native_scope_end(input);
    let document = source_scope_document(source, start..end).map_err(|detail| {
        crate::diagnostic::error::invalid_component_value(input.current_source_location(), detail)
    })?;
    let mut parser = document.root();
    let result = parse(&mut parser);
    let consumed = document
        .boundary(document.root_list(), parser.site().index())
        .offset
        .expect("source cursor boundary");
    advance_native_to(input, consumed);
    result.map_err(SyntaxErrorExt::into_native_source)
}
fn advance_native_component(input: &mut cssparser::Parser<'_, '_>) -> Result<(), ()> {
    use cssparser::Token as Native;
    let nested = matches!(
        input
            .next_including_whitespace_and_comments()
            .map_err(|_| ())?,
        Native::Function(_)
            | Native::ParenthesisBlock
            | Native::SquareBracketBlock
            | Native::CurlyBracketBlock
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
/// The byte where the scope of the native parser `input` ends, which is the
/// end of the last component it holds. The parser is left where it was.
pub(super) fn native_scope_end(input: &mut cssparser::Parser<'_, '_>) -> usize {
    let checkpoint = input.state();
    while advance_native_component(input).is_ok() {}
    let end = input.position().byte_index();
    input.reset(&checkpoint);
    end
}

pub(super) fn advance_native_to(input: &mut cssparser::Parser<'_, '_>, offset: usize) {
    while input.position().byte_index() < offset && advance_native_component(input).is_ok() {}
    assert_eq!(
        input.position().byte_index(),
        offset,
        "portable cursor selects an actual native boundary"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::CssComponentValues;
    use crate::diagnostic::error::property::SyntaxErrorExt;
    use crate::source::CssSourceSnapshot;
    use crate::syntax_consumption::{SyntaxInputLimits, checked_document, source_document};

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
        let document = checked_document(values.items(), SyntaxInputLimits::default(), 0).unwrap();
        let mut input = document.root();
        assert_eq!(input.consume_ident().unwrap(), "a");
        assert_eq!(input.consume_ident().unwrap(), "b");
        assert!(input.only_whitespace_remains());
    }

    #[test]
    fn checked_collection_borrows_the_exact_original_component() {
        let values = crate::component::values::parse_component_values("calc(1px + 2px)").unwrap();
        let document = checked_document(values.items(), SyntaxInputLimits::default(), 0).unwrap();
        let mut input = document.root();
        let value = input.collect_component().unwrap();
        assert!(matches!(value, Cow::Borrowed(_)));
        assert!(std::ptr::eq(value.as_ref(), &values.items()[0]));
    }

    #[test]
    fn failed_nested_parse_advances_outer_and_restore_returns_the_pending_group() {
        let snapshot = CssSourceSnapshot::new("f(a b) tail");
        let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
        let mut input = document.root();
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
        assert_eq!(input.consume_ident().unwrap(), "tail");
        input.restore(checkpoint);
        input
            .within_block(|nested| {
                nested.consume_keyword("a")?;
                nested.consume_keyword("b")?;
                Ok::<_, SyntaxError<'_, ()>>(())
            })
            .unwrap();
        assert_eq!(input.consume_ident().unwrap(), "tail");
    }

    #[test]
    fn lexical_error_leaves_remain_visible_until_grammar_visits_them() {
        let snapshot = CssSourceSnapshot::new("url(a b)!");
        let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
        let mut input = document.root();
        assert_eq!(input.consume_after_whitespace().unwrap(), Token::BadUrl);
        assert_eq!(input.consume_after_whitespace().unwrap(), Token::Delim('!'));
    }

    #[test]
    fn descendant_sites_select_real_mixed_source_children_and_closing_boundaries() {
        let children = crate::component::values::parse_component_values("\n 2px").unwrap();
        let origin = children.items()[1].origin().clone();
        let group = CssComponentValue::try_function("f", children).unwrap();
        let values = CssComponentValues::try_new(vec![group], Default::default()).unwrap();
        let document = checked_document(values.items(), SyntaxInputLimits::default(), 0).unwrap();
        let input = document.root();
        let child = input.site().descendant(&[0, 1], false).unwrap();
        assert_eq!(child.value_origin(), Some(origin));
        assert!(matches!(child.token(), Some(Token::Dimension { .. })));
        let closing = input.site().descendant(&[0], true).unwrap();
        assert_eq!(closing.value_origin(), Some(CssValueOrigin::Programmatic));
        assert_eq!(closing.token(), Some(Token::CloseParen));
    }

    #[test]
    fn nested_end_reports_the_actual_closing_token_and_root_end_reports_eof() {
        let values = crate::component::values::parse_component_values("f()").unwrap();
        let CssComponentValueRef::Function(function) = values.items()[0].view() else {
            unreachable!()
        };
        let closing = function.closing_origin().clone();
        let document = checked_document(values.items(), SyntaxInputLimits::default(), 0).unwrap();
        let mut input = document.root();
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
        let mut input = document.root();
        input
            .within_list_until(StopToken::Semicolon, |nested| {
                nested.consume_keyword("a")?;
                nested.consume_function_named("f")?;
                Ok::<_, SyntaxError<'_, ()>>(())
            })
            .unwrap();
        assert_eq!(input.site().lexeme(), Some(";"));
        input.consume_semicolon().unwrap();
        assert_eq!(input.consume_ident().unwrap(), "tail");
    }
}

#[cfg(test)]
mod native_adapter_tests {
    use super::*;
    use crate::source::CssSourceSnapshot;

    #[test]
    fn a_scope_grammar_leaves_the_native_parser_after_what_it_consumed() {
        let source = CssSourceSnapshot::new("f(a) tail");
        let mut storage = cssparser::ParserInput::new(source.as_str());
        let mut native = cssparser::Parser::new(&mut storage);
        let value = adapt_native(&mut native, &source, |input| {
            input.consume_function_named("f")?;
            input.within_block(|nested| {
                nested.consume_keyword("a")?;
                Ok::<_, SyntaxError<'_, crate::diagnostic::error::property::Error>>(42)
            })
        })
        .unwrap();
        assert_eq!(value, 42);
        assert_eq!(native.position().byte_index(), 4);
        assert_eq!(native.expect_ident().unwrap().as_ref(), "tail");
    }
}
