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
            let origin = match site.origin() {
                CssSerializedOrigin::End(Some(origin)) | CssSerializedOrigin::Token(origin) => {
                    origin
                }
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
        output
            .finish()
            .expect("checked output boundaries")
            .into_css()
    }

    fn authored_text_since(&self, from: Site<'i>) -> String {
        self.authored_text_between(from, self.site())
    }
}

/// Existing native descriptor/rule consumers enter the same value grammar.
/// The checked property front door never enters this adapter.
pub(crate) enum NativeValueRange {
    /// A single-node helper must not admit an unrelated following value.
    Component,
    /// Family names and settings lists consume a sequence of sibling values.
    Remaining,
    /// The whole scope the native callback was handed, to its end: the
    /// contents of a rule block or the value of a declaration. Only the native
    /// `AtRuleParser`, `QualifiedRuleParser` and `DeclarationParser` methods
    /// whose grammar has not been ported enter a ported grammar this way.
    Scope,
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
    use crate::diagnostic::error::property::SyntaxErrorExt;
    use crate::numeric::expression::NumericInputContext;
    use crate::syntax_consumption::{
        SyntaxInputLimits, checked_document, source_scope_document, source_window_document,
    };
    let checkpoint = input.state();
    let start = input.position().byte_index();
    let whole_scope = matches!(range, NativeValueRange::Scope);
    match range {
        NativeValueRange::Component => {
            input.skip_whitespace();
            let _ = advance_native_component(input);
        }
        NativeValueRange::Remaining | NativeValueRange::Scope => {
            while advance_native_component(input).is_ok() {}
        }
    }
    let end = input.position().byte_index();
    input.reset(&checkpoint);
    match numeric.ordinary() {
        NumericInputContext::Parsed(source) => {
            let window = if whole_scope {
                source_scope_document(source, start..end)
            } else {
                source_window_document(source, start..end)
            };
            let document = window.map_err(|detail| {
                crate::diagnostic::error::invalid_component_value(
                    input.current_source_location(),
                    detail,
                )
            })?;
            let mut parser = document.root();
            let result = parse(&mut parser, numeric);
            let consumed = document
                .boundary(document.root_list(), parser.site().index())
                .offset
                .expect("source cursor boundary");
            advance_native_to(input, consumed);
            result.map_err(SyntaxErrorExt::into_native_source)
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
            let document =
                checked_document(siblings, SyntaxInputLimits::default(), 0).map_err(|detail| {
                    crate::diagnostic::error::invalid_component_value(
                        input.current_source_location(),
                        detail,
                    )
                })?;
            let mut parser = document.root();
            let result = parse(&mut parser, numeric);
            let ordinal = parser.site().index();
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
                    if site.is_closing() {
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
fn advance_native_to(input: &mut cssparser::Parser<'_, '_>, offset: usize) {
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
