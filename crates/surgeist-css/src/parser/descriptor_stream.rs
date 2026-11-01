//! What the descriptor grammars share on the parser crate's token streams.
//!
//! A descriptor grammar reads the value of a descriptor from a [`Stream`] and
//! reports its failures as the native errors the recovery diagnostics are
//! written in, located by the boundary of the stream the failure is at. The
//! boundaries of a document of source text are bytes of the source; those of a
//! document of checked components are the bytes of the serialization of the
//! components, which is what the component-native constructors report.
use std::cell::RefCell;
use std::ops::Range;

use cssparser::{ParseError, SourceLocation};
use surgeist_css_parser::Token;
use surgeist_css_parser::grammar::{BlockGrammar, UnitKind};
use surgeist_css_parser::stream::StopToken;

use super::block_units::{
    Failure, NativeError, drive_units, dropped_unit, location_at, offset_of, refused_declaration,
};
use super::declaration_unit::DeclarationText;
use super::property_input::StreamExt as _;
use super::property_input::{SiteExt, advance_native_to, native_scope_end};
use super::recovery::RecoveryState;
use super::syntax_types::{Attached, Document, Site, Stream, StreamError, SyntaxError};
use super::{block_item_diagnostic, is_declaration_recovery_unit, structural_rule_diagnostic};
use crate::component::{CssComponentValues, CssSerializedValue};
use crate::diagnostic::error::property::SyntaxErrorExt;
use crate::diagnostic::error::{
    invalid_component_value, invalid_descriptor_annotation, invalid_descriptor_syntax_token_at,
    unexpected_authored_syntax_token, unexpected_syntax_end, unexpected_syntax_token,
};
use crate::diagnostic::{CssError, CssRecoveryAction, CssRecoveryDiagnostic};
use crate::numeric::expression::NumericInputContext;
use crate::source::{CssSourcePosition, CssSourceSnapshot};

/// The native error of a value grammar. It borrows nothing from the input.
pub(super) type NativeValueError = ParseError<'static, CssError>;
/// The failure of a value grammar.
pub(super) type ValueError<'i> = SyntaxError<'i, NativeValueError>;
/// The result of a value grammar.
pub(super) type Value<'i, T> = Result<T, ValueError<'i>>;

/// Where the boundary `site` of a document is, as the native diagnostics say
/// it: a bytes position of the source for a document of source text, and one
/// of the serialization for a document of checked components.
pub(super) fn location(numeric: &NumericInputContext<'_>, site: Site<'_>) -> SourceLocation {
    match numeric.ordinary() {
        NumericInputContext::Parsed(snapshot) => location_at(snapshot, offset_of(site)),
        NumericInputContext::Components(values, serialized) => {
            at_serialized_offset(serialized, serialized_offset(values, serialized, site))
        }
        NumericInputContext::Checked | NumericInputContext::QuirkyLengths(_) => {
            SourceLocation { line: 0, column: 1 }
        }
    }
}

/// Where the byte `offset` of the serialization is, as the native diagnostics
/// say it.
pub(super) fn at_serialized_offset(
    serialized: &CssSerializedValue,
    offset: usize,
) -> SourceLocation {
    let position = CssSourcePosition::from_byte_offset_in(serialized.as_css(), offset);
    SourceLocation {
        line: position.line().value(),
        column: position.column().value().saturating_add(1),
    }
}

/// The byte of the serialization of `values` that the boundary `site` of the
/// document of `values` is at.
pub(super) fn serialized_offset(
    values: &CssComponentValues,
    serialized: &CssSerializedValue,
    site: Site<'_>,
) -> usize {
    let end = serialized.as_css().len();
    let mut path = site.component_path();
    if path
        .first()
        .is_none_or(|index| *index >= values.items().len())
    {
        return end;
    }
    if site.is_closing() {
        serialized
            .component_end_for_path(&path)
            .map(|end| end.saturating_sub(1))
    } else if let Some(offset) = serialized.component_offset_for_path(&path) {
        Some(offset)
    } else if path.len() > 1 {
        path.pop();
        serialized
            .component_end_for_path(&path)
            .map(|end| end.saturating_sub(1))
    } else {
        None
    }
    .unwrap_or(end)
}

/// The byte of the text the boundary `site` of a document is at, in the terms of
/// the recovery diagnostics: a byte of the source for a document of source
/// text, and of the serialization of the components for a document of
/// components.
pub(super) fn byte_offset(numeric: &NumericInputContext<'_>, site: Site<'_>) -> usize {
    match numeric.ordinary() {
        NumericInputContext::Components(values, serialized) => {
            serialized_offset(values, serialized, site)
        }
        _ => site.offset().map_or(0, surgeist_css_parser::Offset::value),
    }
}

/// The spelling of the token at the boundary `site`. Comments are transparent
/// to the token methods, so the token a failure names is the first one that is
/// no comment, and a comment in front of it is not its spelling.
pub(super) fn token_spelling<'i>(site: Site<'i>) -> Option<&'i str> {
    if site.is_closing() {
        return site.spelling();
    }
    let document = site.document();
    let nodes = document.list(site.list());
    let node = nodes
        .get(site.index()..)?
        .iter()
        .find(|node| document.token(**node) != Token::Comment)?;
    document.spelling(*node)
}

/// The native error of a failure of a value grammar.
pub(super) fn native(
    numeric: &NumericInputContext<'_>,
    error: &ValueError<'_>,
) -> NativeValueError {
    if let Some(native) = error.custom_value() {
        return native.clone();
    }
    let at = location(numeric, error.at());
    match error.unexpected_token() {
        // The whitespace is named as it was written.
        Some(token @ Token::Whitespace) => {
            unexpected_authored_syntax_token(at, token, token_spelling(error.at()).unwrap_or(" "))
        }
        Some(token) => unexpected_syntax_token(at, token),
        None => unexpected_syntax_end(at),
    }
}

/// A failure of a value grammar at `at` that `make` builds the native error
/// of, given where `at` is.
pub(super) fn fail<'i>(
    numeric: &NumericInputContext<'_>,
    at: Site<'i>,
    make: impl FnOnce(SourceLocation) -> NativeValueError,
) -> ValueError<'i> {
    SyntaxError::custom(at, make(location(numeric, at)))
}

/// The failure of a property grammar, as a failure of a value grammar that
/// reports what the property grammar's own front door would have.
pub(super) fn from_property<'i, T>(
    numeric: &NumericInputContext<'_>,
    result: Result<T, SyntaxError<'i, crate::diagnostic::error::property::Error>>,
) -> Value<'i, T> {
    result.map_err(|error| {
        let at = error.at();
        let native = match numeric.ordinary() {
            NumericInputContext::Components(values, serialized) => {
                let error = error.qualify();
                let offset = serialized_offset(values, serialized, error.at());
                error.into_native_at(serialized.as_css(), offset)
            }
            _ => error.into_native_source(),
        };
        SyntaxError::custom(at, native)
    })
}

/// How much of the stream a property grammar is handed.
#[derive(Clone, Copy)]
pub(super) enum Span {
    /// The next component, after the whitespace: a grammar of one value must
    /// not admit an unrelated following value.
    Component,
    /// Every component that remains: names and settings lists.
    Remaining,
}

/// Runs a property grammar over a document of the components `span` selects,
/// and moves the stream past what the grammar consumed. The grammar reads
/// those components and no others, so the diagnostics it reports are about
/// them alone.
pub(super) fn from_window<'i, T>(
    numeric: &NumericInputContext<'_>,
    input: &mut Stream<'i>,
    span: Span,
    parse: impl for<'w> FnOnce(
        &mut Stream<'w>,
    )
        -> Result<T, SyntaxError<'w, crate::diagnostic::error::property::Error>>,
) -> Value<'i, T> {
    let first = input.site();
    let mut probe = input.clone();
    match span {
        Span::Component => {
            probe.discard_whitespace();
            let _ = probe.consume_token_with_comments();
        }
        Span::Remaining => while probe.consume_token_with_comments().is_ok() {},
    }
    let last = probe.site();
    match numeric.ordinary() {
        NumericInputContext::Parsed(snapshot) => {
            let (start, end) = (offset_of(first), offset_of(last));
            let document = crate::syntax_consumption::source_window_document(snapshot, start..end)
                .map_err(|error| fail(numeric, first, |at| invalid_component_value(at, error)))?;
            let mut window = document.root();
            let result = parse(&mut window);
            let consumed = window
                .site()
                .offset()
                .map_or(end, surgeist_css_parser::Offset::value);
            while input
                .site()
                .offset()
                .is_some_and(|at| at.value() < consumed)
                && input.consume_token_with_comments().is_ok()
            {}
            result.map_err(|error| SyntaxError::custom(first, error.into_native_source()))
        }
        NumericInputContext::Components(values, serialized) => {
            let count = last.index().saturating_sub(first.index());
            let path = first.component_path();
            let (head, parent) = path
                .split_last()
                .map_or((0, &[][..]), |(head, parent)| (*head, parent));
            let siblings = if count == 0 {
                &[][..]
            } else if parent.is_empty() {
                values.items()
            } else {
                values
                    .component_at_path(parent)
                    .and_then(crate::component::CssComponentValue::child_values)
                    .map_or(&[][..], crate::component::CssComponentValues::items)
            };
            let siblings = siblings.get(head..head + count).unwrap_or(&[]);
            let end = match siblings.len() {
                0 => serialized_offset(values, serialized, first),
                length => {
                    let mut last_path = parent.to_vec();
                    last_path.push(head + length - 1);
                    serialized
                        .component_end_for_path(&last_path)
                        .unwrap_or_else(|| serialized.as_css().len())
                }
            };
            let document = crate::syntax_consumption::checked_document(
                siblings,
                crate::syntax_consumption::SyntaxInputLimits::default(),
                0,
            )
            .map_err(|error| fail(numeric, first, |at| invalid_component_value(at, error)))?;
            let mut window = document.root();
            let result = parse(&mut window);
            let ordinal = window.site().index();
            for _ in 0..ordinal {
                if input.consume_token_with_comments().is_err() {
                    break;
                }
            }
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
                    path.push(head + relative[0]);
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
                SyntaxError::custom(first, error.into_native_at(serialized.as_css(), offset))
            })
        }
        NumericInputContext::Checked | NumericInputContext::QuirkyLengths(_) => {
            from_property(numeric, parse(input))
        }
    }
}

/// [`from_window`] over the next component.
pub(super) fn from_component<'i, T>(
    numeric: &NumericInputContext<'_>,
    input: &mut Stream<'i>,
    parse: impl for<'w> FnOnce(
        &mut Stream<'w>,
    )
        -> Result<T, SyntaxError<'w, crate::diagnostic::error::property::Error>>,
) -> Value<'i, T> {
    from_window(numeric, input, Span::Component, parse)
}

/// [`from_window`] over every component that remains.
pub(super) fn from_remaining<'i, T>(
    numeric: &NumericInputContext<'_>,
    input: &mut Stream<'i>,
    parse: impl for<'w> FnOnce(
        &mut Stream<'w>,
    )
        -> Result<T, SyntaxError<'w, crate::diagnostic::error::property::Error>>,
) -> Value<'i, T> {
    from_window(numeric, input, Span::Remaining, parse)
}

/// The comma-separated list of the stream, each item read by `item`: the
/// `#` multiplier. The whitespace that begins an item is not part of it.
pub(super) fn comma_list<'i, T>(
    input: &mut Stream<'i>,
    mut item: impl FnMut(&mut Stream<'i>) -> Value<'i, T>,
) -> Value<'i, Vec<T>> {
    input.comma_segments(|segment| {
        segment.discard_whitespace();
        item(segment)
    })
}

/// Runs `consume`, which discards whitespace and then consumes a token, and
/// reports a token it did not expect where the stream stood before the
/// whitespace: the boundary a reader of the text points at.
pub(super) fn from_here<'i, T, E>(
    input: &mut Stream<'i>,
    consume: impl FnOnce(&mut Stream<'i>) -> Result<T, StreamError<'i>>,
) -> Result<T, SyntaxError<'i, E>> {
    let before = input.site();
    consume(input).map_err(|error| match error.unexpected_token() {
        Some(token) => before.unexpected(token),
        None => SyntaxError::from(error),
    })
}

/// The components of every token of the document `input` reads, which is the
/// scope of a native callback, or the failure to collect them and where it is
/// reported: at the end of the top-level component that holds the failing token.
pub(super) fn collect_scope(
    input: &Stream<'_>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssComponentValues, (crate::component::CssComponentValueError, SourceLocation)> {
    let document = input.start_site().document();
    input.remaining_components().map_err(|error| {
        let end = input.end_site();
        let at = match numeric.ordinary() {
            NumericInputContext::Parsed(snapshot) => {
                component_failure_location(snapshot, document, &error, offset_of(end))
            }
            _ => location(numeric, end),
        };
        (error, at)
    })
}

/// The components of the scope of the native parser `input`, to its end, which
/// is where the parser is left; or the fault that kept them from being
/// collected and where it is reported. `start` is where the scope begins.
pub(super) struct CollectedScope {
    pub(super) values:
        Result<CssComponentValues, (crate::component::CssComponentValueError, SourceLocation)>,
    pub(super) start: SourceLocation,
}

/// Collects the components of the scope of the native parser `input`.
pub(super) fn collect_native_scope<'i>(
    input: &mut cssparser::Parser<'i, '_>,
    snapshot: &CssSourceSnapshot,
) -> Result<CollectedScope, ParseError<'i, CssError>> {
    parse_scope(input, snapshot, |stream, numeric| {
        let start = location(numeric, stream.site());
        let values = collect_scope(stream, numeric);
        consume_remaining(stream);
        Ok(CollectedScope { values, start })
    })
}

/// Whether the comma just consumed ends the stream, so that it adds no member
/// to a comma-separated list (Syntax 3 section 5.4.10). A whitespace token
/// after it is a token; a comment is not.
pub(super) fn at_final_comma(input: &Stream<'_>) -> bool {
    input.clone().consume_token().is_err()
}

/// Requires that only whitespace remains, and reports what remains where the
/// whitespace begins. The check does not move the cursor.
pub(super) fn expect_exhausted<'i, E>(input: &mut Stream<'i>) -> Result<(), SyntaxError<'i, E>> {
    let at = input.site();
    let mark = input.mark();
    let result = input
        .require_empty()
        .map_err(|error| match error.unexpected_token() {
            Some(token) => at.unexpected(token),
            None => SyntaxError::from(error),
        });
    input.restore(mark);
    result
}

/// The boundary after the token at `at`: after the opening token of a function
/// or block, inside it.
pub(super) fn after_token<'i>(at: Site<'i>) -> Site<'i> {
    let document = at.document();
    at.node()
        .and_then(|node| match document.children(node) {
            Some(list) => document.site(list, 0),
            None => document.site(at.list(), at.index() + 1),
        })
        .unwrap_or(at)
}

/// Where the contents begin of the function or block that was consumed last,
/// given the stream's site after it; `site` when it was not a group.
pub(super) fn inside_consumed<'i>(site: Site<'i>) -> Site<'i> {
    let document = site.document();
    site.index()
        .checked_sub(1)
        .and_then(|previous| document.list(site.list()).get(previous))
        .and_then(|node| document.children(*node))
        .and_then(|list| document.site(list, 0))
        .unwrap_or(site)
}

/// Consumes the rest of the stream, nested blocks included.
pub(super) fn consume_remaining(input: &mut Stream<'_>) {
    // A group is one node of its list, so consuming its opening token moves
    // past its contents; comments are tokens of the list like any other.
    while input.consume_token_with_comments().is_ok() {}
}

/// Admits the root of a descriptor's value: no `;` and no closing token may
/// end a component of the declaration's value. The stream is left where it was.
pub(super) fn validate_root<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    owner: &str,
    descriptor: &str,
) -> Result<(), ValueError<'i>> {
    let start = input.mark();
    loop {
        let at = input.site();
        let token = match input.consume_token_with_comments() {
            Ok(token) => token,
            Err(error) if error.unexpected_token().is_none() => break,
            Err(error) => return Err(error.into()),
        };
        if matches!(
            token,
            Token::Semicolon | Token::CloseCurly | Token::CloseParen | Token::CloseSquare
        ) {
            return Err(fail(numeric, at, |location| {
                invalid_descriptor_syntax_token_at(location, owner, descriptor, token)
            }));
        }
    }
    input.restore(start);
    Ok(())
}

/// The boundary the first `!` at the root of `document` is at, if there is one.
fn first_bang(document: &Document<'_>) -> Option<usize> {
    let list = document.root_list();
    let nodes = document.list(list);
    let index = nodes
        .iter()
        .position(|node| matches!(document.token(*node), Token::Delim('!')))?;
    Some(offset_of(document.site(list, index)?))
}

/// How the text of a descriptor's value is bounded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Boundary {
    /// The root of the whole text is admitted first, and the value ends at `!`.
    Whole,
    /// The value ends at `!`, and its root is admitted before its components
    /// are collected.
    ValidatedValue,
    /// The value ends at `!`, and its grammar admits its own root.
    Value,
    /// The text is the value, and its root is admitted first.
    ValidatedText,
    /// The text is the value: no annotation is looked for.
    Text,
}

impl Boundary {
    /// Whether the root of the whole text is admitted before the value is split.
    const fn validates_scope(self) -> bool {
        matches!(self, Self::Whole)
    }

    /// Whether the root of the value is admitted before its components are
    /// collected.
    const fn validates_value(self) -> bool {
        matches!(self, Self::ValidatedValue | Self::ValidatedText)
    }

    /// Whether the value ends at the first `!`, which is then an annotation no
    /// descriptor admits.
    const fn splits_annotation(self) -> bool {
        matches!(self, Self::Whole | Self::ValidatedValue | Self::Value)
    }
}

/// Parses the value of a descriptor whose text is `value_start..end` of the
/// source: the value is what precedes the first `!`, and any `!` is an
/// annotation no descriptor admits (Syntax 3 section 5.5.6), judged once the
/// value has been read. `parse` reads the value from a stream of its own and
/// gets the components it holds.
pub(super) fn parse_descriptor_text<T>(
    snapshot: &CssSourceSnapshot,
    value_start: usize,
    end: usize,
    owner: &str,
    descriptor: &str,
    boundary: Boundary,
    parse: impl for<'d> FnOnce(
        &mut Stream<'d>,
        CssComponentValues,
        &NumericInputContext<'_>,
        Range<usize>,
    ) -> Value<'d, T>,
) -> Result<T, NativeValueError> {
    let numeric = NumericInputContext::parsed(snapshot);
    let failed_at = |end: usize| {
        move |error: crate::component::CssComponentValueError| {
            invalid_component_value(location_at(snapshot, end), error)
        }
    };
    // The whole text is read only to admit its root or to find the annotation.
    let scope = if boundary.validates_scope() || boundary.splits_annotation() {
        Some(
            crate::syntax_consumption::source_scope_document(snapshot, value_start..end)
                .map_err(failed_at(end))?,
        )
    } else {
        None
    };
    if boundary.validates_scope()
        && let Some(scope) = &scope
    {
        validate_root(&mut scope.root(), &numeric, owner, descriptor)
            .map_err(|error| native(&numeric, &error))?;
    }
    let bang = scope
        .as_ref()
        .filter(|_| boundary.splits_annotation())
        .and_then(first_bang);
    let value_end = bang.unwrap_or(end);
    let value = crate::syntax_consumption::source_window_document(snapshot, value_start..value_end)
        .map_err(failed_at(value_end))?;
    if boundary.validates_value() {
        validate_root(&mut value.root(), &numeric, owner, descriptor)
            .map_err(|error| native(&numeric, &error))?;
    }
    let components = crate::component::values::parse::promote_window(&value).map_err(|error| {
        invalid_component_value(
            component_failure_location(snapshot, &value, &error, value_end),
            error,
        )
    })?;
    let range = value_start..value_end;
    let parsed = parse(&mut value.root(), components, &numeric, range)
        .map_err(|error| native(&numeric, &error))?;
    match bang {
        Some(bang) => Err(invalid_descriptor_annotation(
            location_at(snapshot, bang),
            owner,
            descriptor,
        )),
        None => Ok(parsed),
    }
}

/// Parses the value of a descriptor from the checked components of a document:
/// the same boundary [`parse_descriptor_text`] draws, drawn with the stream.
pub(super) fn parse_descriptor_checked<'i, T>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    owner: &str,
    descriptor: &str,
    parse: impl FnOnce(&mut Stream<'i>) -> Value<'i, T>,
) -> Result<T, NativeValueError> {
    let parsed = input
        .within_list_until(StopToken::ExclamationMark, parse)
        .map_err(|error| native(numeric, &error))?;
    if input.only_whitespace_remains() {
        return Ok(parsed);
    }
    let at = input.site();
    Err(invalid_descriptor_annotation(
        location(numeric, at),
        owner,
        descriptor,
    ))
}

/// The grammar of the descriptors of a block, driven by the parser crate.
pub(super) trait DescriptorGrammar<'i, Item>:
    BlockGrammar<'i, Attached<'i>, Item = Item, Error = NativeError<'i>>
{
    /// Admits the declaration named `name` at `text` once more: the one a block
    /// reads to its `;` although Syntax 3 section 5.5.6 step 8 refuses it.
    fn readmit(&mut self, name: &str, text: &DeclarationText) -> Result<Item, NativeError<'i>>;
}

/// What the recovery of a descriptor block reads.
pub(super) struct DescriptorRecovery<'a> {
    pub(super) source: &'a str,
    pub(super) recovery: &'a RecoveryState,
    /// The production a failed rule's block is charged to.
    pub(super) production: &'static str,
    /// What is dropped for a unit that begins like a declaration.
    pub(super) declaration: CssRecoveryAction,
    /// Whether a unit whose text begins with `@` (after whitespace, as Rust
    /// reads it) is judged an at-rule before it is judged a declaration.
    pub(super) at_rule_first: bool,
}

/// Admits once more the declaration that Syntax 3 section 5.5.6 step 8 refuses
/// and the driver did not show to the grammar: its block reads it to its `;`
/// all the same, and reports the grammar's verdict on it (until the declaration
/// paths follow step 8, in step 12). Returns the item and the node of the list
/// where the declaration ends, or the verdict; `None` when the failed unit is no
/// such declaration.
fn readmit_refused<'i, Item, G>(
    failed: &Failure<'i>,
    readmit: &mut G,
) -> Option<Result<(Item, usize), NativeError<'i>>>
where
    G: DescriptorGrammar<'i, Item>,
{
    let (start, end) = refused_declaration(failed)?;
    let text = DeclarationText::read(start, end);
    let name = match start.token() {
        Some(Token::Ident(name)) => name,
        _ => "",
    };
    Some(readmit.readmit(name, &text).map(|item| (item, end)))
}

/// Drives the units of the block `contents` through `grammar`: a valid unit is
/// retained, and every other is dropped with the recovery diagnostic of
/// Syntax 3 section 7.1.
pub(super) fn drive_descriptors<'i, Item, G>(
    contents: &mut Stream<'i>,
    grammar: &mut G,
    readmit: &mut G,
    context: &DescriptorRecovery<'_>,
) -> (Vec<Item>, Vec<CssRecoveryDiagnostic>)
where
    G: DescriptorGrammar<'i, Item>,
{
    let diagnostics = RefCell::new(Vec::new());
    let items = drive_descriptors_into(contents, grammar, readmit, context, &diagnostics);
    (items, diagnostics.into_inner())
}

/// [`drive_descriptors`], whose diagnostics go to a sink the grammar shares: a
/// grammar that recovers a nested block reports into the same list, in source
/// order.
pub(super) fn drive_descriptors_into<'i, Item, G>(
    contents: &mut Stream<'i>,
    grammar: &mut G,
    readmit: &mut G,
    context: &DescriptorRecovery<'_>,
    sink: &RefCell<Vec<CssRecoveryDiagnostic>>,
) -> Vec<Item>
where
    G: DescriptorGrammar<'i, Item>,
{
    let snapshot = context.recovery.source_snapshot();
    let mut items = Vec::new();
    drive_units(contents, grammar, |unit: Result<Item, Failure<'i>>| {
        let failed = match unit {
            Ok(item) => {
                items.push(item);
                return None;
            }
            Err(failed) => failed,
        };
        let mut dropped = dropped_unit(snapshot, context.source, &failed);
        match readmit_refused(&failed, readmit) {
            Some(Ok((item, end))) => {
                items.push(item);
                return Some(end);
            }
            Some(Err(error)) => dropped.error = error,
            None => {}
        }
        // An at-rule whose prelude was rejected (a stream failure, or an error
        // reported at the prelude's start) ends before the `;` or `{` that
        // ended it; one whose block was rejected is dropped whole.
        let text = if dropped.kind == UnitKind::AtRule
            && (failed.error().is_stream_error() || failed.error().at() != failed.start())
        {
            dropped.text_to_prelude_end()
        } else {
            dropped.text()
        };
        // A failed rule's block is checked for the resources a grammar would
        // have been charged for entering it.
        let error = dropped
            .block_contents
            .and_then(|contents| {
                context.recovery.check_failed_rule_block_at(
                    context.source,
                    contents,
                    context.production,
                )
            })
            .unwrap_or(dropped.error);
        let action = if context.at_rule_first && text.trim_start().starts_with('@') {
            CssRecoveryAction::DropAtRule
        } else if is_declaration_recovery_unit(text) {
            context.declaration
        } else if text.trim_start().starts_with('@') {
            CssRecoveryAction::DropAtRule
        } else {
            CssRecoveryAction::DropQualifiedRule
        };
        if let Some(diagnostic) = structural_rule_diagnostic(
            context.source,
            error,
            text,
            dropped.start,
            dropped.end,
            action,
        ) {
            sink.borrow_mut().push(diagnostic);
        }
        None
    });
    items
}

/// Runs a value grammar over the whole scope of the native parser `input`,
/// and leaves the parser where the grammar left the stream.
pub(super) fn parse_scope<'i, T>(
    input: &mut cssparser::Parser<'i, '_>,
    snapshot: &CssSourceSnapshot,
    parse: impl for<'d> FnOnce(&mut Stream<'d>, &NumericInputContext<'_>) -> Value<'d, T>,
) -> Result<T, ParseError<'i, CssError>> {
    let start = input.position().byte_index();
    let end = native_scope_end(input);
    let numeric = NumericInputContext::parsed(snapshot);
    let document = crate::syntax_consumption::source_scope_document(snapshot, start..end)
        .map_err(|error| invalid_component_value(input.current_source_location(), error))?;
    let mut stream = document.root();
    let value = parse(&mut stream, &numeric).map_err(|error| native(&numeric, &error))?;
    let consumed = stream
        .site()
        .offset()
        .map_or(end, surgeist_css_parser::Offset::value);
    advance_native_to(input, consumed);
    Ok(value)
}

/// The position of a declaration at the byte `offset` of the source.
pub(super) fn declaration_position(
    snapshot: &CssSourceSnapshot,
    offset: usize,
) -> CssSourcePosition {
    snapshot
        .position_at(offset)
        .unwrap_or_else(|| CssSourcePosition::from_byte_offset_in(snapshot.as_str(), offset))
}

/// Drives the units of a block of descriptors and at-rules whose blocks the
/// block's grammar enters and rejects: a unit that is not valid is dropped with
/// the recovery diagnostic of a block item, and the valid units around it
/// survive.
pub(super) fn drive_block_items<'i, Item, G>(
    contents: &mut Stream<'i>,
    grammar: &mut G,
    readmit: &mut G,
    source: &str,
    recovery: &RecoveryState,
    declaration: CssRecoveryAction,
) -> (Vec<Item>, Vec<CssRecoveryDiagnostic>)
where
    G: DescriptorGrammar<'i, Item>,
{
    let snapshot = recovery.source_snapshot();
    let mut items = Vec::new();
    let mut diagnostics = Vec::new();
    drive_units(contents, grammar, |unit: Result<Item, Failure<'i>>| {
        let failed = match unit {
            Ok(item) => {
                items.push(item);
                return None;
            }
            Err(failed) => failed,
        };
        let mut dropped = dropped_unit(snapshot, source, &failed);
        match readmit_refused(&failed, readmit) {
            Some(Ok((item, end))) => {
                items.push(item);
                return Some(end);
            }
            Some(Err(error)) => dropped.error = error,
            None => {}
        }
        // The dropped unit of a failed rule ends after its block's opening
        // token; the block is skipped with the unit. An at-rule whose prelude
        // was accepted had its block entered and is dropped whole; one whose
        // prelude was rejected (a stream failure) ends before the `;` or `{`
        // that ended it.
        let (text, end) = match dropped.kind {
            UnitKind::AtRule if failed.error().is_stream_error() => {
                (dropped.text_to_prelude_end(), dropped.text_to_block().1)
            }
            UnitKind::AtRule => (dropped.text(), dropped.end),
            _ => dropped.text_to_block(),
        };
        let action = if is_declaration_recovery_unit(text) {
            declaration
        } else {
            CssRecoveryAction::DropAtRule
        };
        if let Some(diagnostic) = block_item_diagnostic(source, dropped.error, text, end, action) {
            diagnostics.push(diagnostic);
        }
        None
    });
    (items, diagnostics)
}

/// Where the failure to collect the components of `document` is reported: at the
/// end of the top-level component that holds the failing token, and at `end`
/// when the failure is about no token of the text.
pub(super) fn component_failure_location(
    snapshot: &CssSourceSnapshot,
    document: &Document<'_>,
    error: &crate::component::CssComponentValueError,
    end: usize,
) -> SourceLocation {
    let at = match error.origin() {
        crate::component::CssValueOrigin::Parsed(origin) => {
            let failing = origin.span().start().byte_offset().value();
            document
                .list(document.root_list())
                .iter()
                .filter_map(|node| document.range(*node))
                .find(|range| {
                    range.start.value() <= failing
                        && failing < range.end.value().max(range.start.value() + 1)
                })
                .map_or(end, |range| range.end.value())
        }
        _ => end,
    };
    location_at(snapshot, at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax_consumption::source_document;

    fn document(text: &str) -> (CssSourceSnapshot, Document<'_>) {
        let snapshot = CssSourceSnapshot::new(text);
        let document = source_document(text, &snapshot, 0).unwrap();
        (snapshot, document)
    }

    #[test]
    fn the_exhaustion_check_leaves_the_cursor_before_the_trailing_whitespace() {
        let (_, document) = document("a  /*c*/ ");
        let mut input = document.root();
        input.consume_ident().unwrap();
        expect_exhausted::<()>(&mut input).unwrap();
        assert_eq!(input.site().offset().map(|offset| offset.value()), Some(1));
    }

    #[test]
    fn the_exhaustion_failure_is_reported_where_the_whitespace_begins() {
        let (_, document) = document("a  b");
        let mut input = document.root();
        input.consume_ident().unwrap();
        let error = expect_exhausted::<()>(&mut input).unwrap_err();
        assert_eq!(error.at().offset().map(|offset| offset.value()), Some(1));
        assert_eq!(error.unexpected_token(), Some(Token::Ident("b")));
        assert_eq!(input.site().offset().map(|offset| offset.value()), Some(1));
    }

    #[test]
    fn consuming_the_remainder_takes_nested_blocks_and_trailing_comments() {
        let (_, document) = document("a f(b c) [d] /*x*/");
        let mut input = document.root();
        consume_remaining(&mut input);
        assert_eq!(
            input.site().offset().map(|offset| offset.value()),
            Some("a f(b c) [d] /*x*/".len())
        );
    }

    #[test]
    fn a_comma_list_item_does_not_begin_with_whitespace() {
        let (_, document) = document(" a , b");
        let mut input = document.root();
        let starts = comma_list(&mut input, |item| {
            let start = item.site().offset().map(|offset| offset.value());
            item.consume_ident()?;
            Ok::<_, ValueError<'_>>(start)
        })
        .unwrap();
        assert_eq!(starts, [Some(1), Some(5)]);
    }

    #[test]
    fn a_property_grammar_reads_the_next_component_and_no_other() {
        let text = " 1 2";
        let (snapshot, document) = document(text);
        let numeric = NumericInputContext::parsed(&snapshot);
        let mut input = document.root();
        from_component(&numeric, &mut input, |window| {
            crate::parser::values::parse_integer_value(window, &numeric)
        })
        .unwrap();
        assert_eq!(input.site().offset().map(|offset| offset.value()), Some(2));
        from_component(&numeric, &mut input, |window| {
            crate::parser::values::parse_integer_value(window, &numeric)
        })
        .unwrap();
        assert_eq!(input.site().offset().map(|offset| offset.value()), Some(4));
    }

    #[test]
    fn a_family_name_is_read_from_every_remaining_component() {
        let (snapshot, document) = document("Fancy Font");
        let numeric = NumericInputContext::parsed(&snapshot);
        let mut input = document.root();
        let family = from_remaining(&numeric, &mut input, |window| {
            crate::parser::typography::parse_non_generic_font_family_name(window)
        })
        .unwrap();
        assert_eq!(family.as_str(), "Fancy Font");
        assert!(input.only_whitespace_remains());
    }

    #[test]
    fn a_property_grammar_cannot_see_the_components_before_its_window() {
        // The failure is about the end of the window, not about the `1` that
        // precedes it, which is not part of the window.
        let text = "1 ";
        let (snapshot, document) = document(text);
        let numeric = NumericInputContext::parsed(&snapshot);
        let mut input = document.root();
        from_component(&numeric, &mut input, |window| {
            crate::parser::values::parse_integer_value(window, &numeric)
        })
        .unwrap();
        let error = from_component(&numeric, &mut input, |window| {
            crate::parser::values::parse_integer_value(window, &numeric)
        })
        .unwrap_err();
        let native = native(&numeric, &error);
        assert!(matches!(
            native.kind,
            cssparser::ParseErrorKind::Custom(CssError {
                kind: crate::diagnostic::error::ErrorKind::UnexpectedEnd(_),
                ..
            })
        ));
    }
}
