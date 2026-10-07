//! Authored component construction entries over the shared normalized arena.

use super::*;
use crate::syntax_consumption::{
    GenericFaultKind, GenericSyntaxFault, SourceWindow, SyntaxDocument, SyntaxInput,
    SyntaxInputLimits, SyntaxRange, consume_comma_separated_components, consume_one_component,
    normalize,
};
use crate::{CssParseReport, CssRecoveryAction, CssRecoveryDiagnostic};
use std::borrow::Cow;

/// Parses exactly one authored component, ignoring outer whitespace and comments.
///
/// Empty input, extra nontrivia components, and checked lexical failures return
/// `None` with original-source diagnostics. Functions and blocks closed by EOF
/// remain available with implicit closure origins and recovery diagnostics;
/// such reports fail clean-report validation. This entry selects no value grammar.
#[must_use]
pub fn parse_component_value(source: &str) -> CssParseReport<Option<CssComponentValue>> {
    parse_component_value_with_limits(source, CssComponentValueLimits::default())
}

/// Parses one authored component under whole-input resource limits.
///
/// Limits include all source bytes and components, including ignored outer
/// trivia, and the generated spelling of the retained component. A resource
/// failure rejects the complete input without retaining a partial component.
#[must_use]
pub fn parse_component_value_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssComponentValue>> {
    let report = match document(source, limits) {
        Err(error) => rejected(source, error, None),
        Ok(document) => match consume_one_component(&mut document.cursor(document.root)) {
            Err(fault) => rejected(source, count_error(&document, fault), None),
            Ok(node) => match promote(&document, &[node], limits.max_css_bytes) {
                Err(error) => rejected(source, error, None),
                Ok(values) => {
                    let diagnostics = implicit_diagnostics(source, &values);
                    let component = values
                        .items
                        .into_vec()
                        .into_iter()
                        .next()
                        .expect("one selected component promotes to one checked component");
                    CssParseReport::new(Some(component), diagnostics)
                }
            },
        },
    };
    crate::parser::finish_report(source, report)
}

/// Parses authored component lists separated by top-level comma tokens.
///
/// Every raw segment has one slot. Empty segments are valid empty lists,
/// including empty input and the final segment after a trailing comma. Segment
/// whitespace and comments are retained; nested commas do not split groups.
/// A checked lexical failure produces `None` only for its segment, with a
/// diagnostic spanning that segment. Implicit EOF closures retain syntax and
/// diagnostics, so clean-report validation fails. No caller grammar is selected.
#[must_use]
pub fn parse_comma_separated_component_values(
    source: &str,
) -> CssParseReport<Vec<Option<CssComponentValues>>> {
    parse_comma_separated_component_values_with_limits(source, CssComponentValueLimits::default())
}

/// Parses comma-separated authored lists under whole-input resource limits.
///
/// Source bytes, aggregate components and nesting are admitted once for the
/// complete input, including separating commas and trivia. Generated spelling
/// bytes are bounded across retained lists and separators. Any resource failure
/// rejects the complete input and returns no slots.
#[must_use]
pub fn parse_comma_separated_component_values_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Vec<Option<CssComponentValues>>> {
    let report = match document(source, limits) {
        Err(error) => rejected(source, error, Vec::new()),
        Ok(document) => comma_lists(source, &document, limits),
    };
    crate::parser::finish_report(source, report)
}

fn document(
    source: &str,
    limits: CssComponentValueLimits,
) -> Result<SyntaxDocument<'_>, CssComponentValueError> {
    // Deny excessive input before retaining a source snapshot.
    if source.len() > limits.max_css_bytes {
        return Err(CssComponentValueError::new(
            CssComponentValueErrorKind::ByteLimit,
            CssValueOrigin::UnretainedInput {
                byte_length: source.len(),
            },
        ));
    }
    normalize(
        SyntaxInput::Source(SourceWindow {
            text: Cow::Borrowed(source),
            range: 0..source.len(),
            original: CssSourceSnapshot::new(source),
        }),
        SyntaxInputLimits {
            max_depth: limits.max_depth,
            max_components: limits.max_components,
            max_known_spelling_bytes: limits.max_css_bytes,
        },
        0,
    )
}

fn promote(
    document: &SyntaxDocument<'_>,
    nodes: &[crate::syntax_consumption::NodeId],
    max_css_bytes: usize,
) -> Result<CssComponentValues, CssComponentValueError> {
    // Whole-input admission already charged count and depth, including nodes
    // omitted by this selection. Promotion owns lexical construction and the
    // canonical serializer owns generated byte admission.
    let values = parse::promote_nodes(document, nodes, CssComponentValueLimits::default())?;
    serialize::validate(&values, max_css_bytes)?;
    Ok(values)
}

fn comma_lists(
    source: &str,
    document: &SyntaxDocument<'_>,
    limits: CssComponentValueLimits,
) -> CssParseReport<Vec<Option<CssComponentValues>>> {
    let ranges = consume_comma_separated_components(&mut document.cursor(document.root));
    let mut lists = Vec::with_capacity(ranges.len());
    let mut diagnostics = Vec::new();
    let mut output = CssCanonicalBuilder::counting(limits.max_css_bytes);
    for range in ranges {
        let nodes = &document.lists[range.list][range.start..range.end];
        match promote(document, nodes, usize::MAX) {
            Ok(values) => {
                if let Err(error) = output.push_components(values.items()) {
                    return rejected(source, error, Vec::new());
                }
                diagnostics.extend(implicit_diagnostics(source, &values));
                lists.push(Some(values));
            }
            Err(error) if crate::error::is_component_resource_error(&error) => {
                return rejected(source, error, Vec::new());
            }
            Err(error) => {
                diagnostics.push(rejection(source, error, source_range(document, &range)));
                lists.push(None);
            }
        }
        // The selected range excludes its actual separator. Reuse that source
        // payload through canonical promotion rather than inventing a token.
        if let Some(&comma) = document.lists[range.list].get(range.end) {
            let separator =
                parse::promote_nodes(document, &[comma], CssComponentValueLimits::default())
                    .expect("range selector ends only at an admitted comma");
            if let Err(error) = output.push_components(separator.items()) {
                return rejected(source, error, Vec::new());
            }
        }
    }
    CssParseReport::new(lists, diagnostics)
}

fn source_range(document: &SyntaxDocument<'_>, range: &SyntaxRange) -> Range<usize> {
    let start = document
        .boundary(range.list, range.start)
        .source
        .unwrap()
        .offset;
    let end = document
        .boundary(range.list, range.end)
        .source
        .unwrap()
        .offset;
    start..end
}

fn count_error(document: &SyntaxDocument<'_>, fault: GenericSyntaxFault) -> CssComponentValueError {
    let kind = match fault.kind {
        GenericFaultKind::EmptyInput => CssComponentValueErrorKind::EmptyInput,
        GenericFaultKind::TrailingInput => CssComponentValueErrorKind::TrailingInput,
        _ => unreachable!("single-component selector only reports count faults"),
    };
    let origin = if let Some(&node) = document.lists[fault.at.list].get(fault.at.index) {
        document.nodes[node].token().origin.clone().into_owned()
    } else {
        let at = fault.at.source.expect("source entry has real EOF");
        CssValueOrigin::Parsed(
            CssParsedOrigin::from_range(&at.snapshot, at.offset..at.offset)
                .expect("actual EOF is an original-source boundary"),
        )
    };
    CssComponentValueError::new(kind, origin)
}

fn rejected<T>(source: &str, error: CssComponentValueError, syntax: T) -> CssParseReport<T> {
    CssParseReport::new(syntax, vec![rejection(source, error, 0..source.len())])
}

fn rejection(
    source: &str,
    detail: CssComponentValueError,
    range: Range<usize>,
) -> CssRecoveryDiagnostic {
    let action = if detail.kind() == CssComponentValueErrorKind::NestingLimit {
        CssRecoveryAction::StopAtNestingLimit
    } else {
        CssRecoveryAction::RejectInput
    };
    let error = crate::error::from_parse_error(
        source,
        crate::error::invalid_component_value(
            cssparser::SourceLocation { line: 0, column: 1 },
            detail,
        ),
    );
    let span = CssSourceSpan::new(
        CssSourcePosition::from_byte_offset_in(source, range.start),
        CssSourcePosition::from_byte_offset_in(source, range.end),
    )
    .expect("selected source range is ordered");
    CssRecoveryDiagnostic::new(error, span, action)
        .expect("component fault belongs to its selected source range")
}

fn implicit_diagnostics(source: &str, values: &CssComponentValues) -> Vec<CssRecoveryDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut pending: Vec<_> = values.items().iter().rev().collect();
    while let Some(component) = pending.pop() {
        if let Some(children) = component.child_values() {
            pending.extend(children.items().iter().rev());
        }
        // Comments and escaped identifier EOF are independently published by
        // the existing source lexical diagnostic owner.
        if matches!(
            component.view(),
            CssComponentValueRef::Function(_)
                | CssComponentValueRef::Block(_)
                | CssComponentValueRef::Token(
                    CssValueTokenRef::String(_) | CssValueTokenRef::Url(_)
                )
        ) && let Some(CssValueOrigin::ImplicitClosure { at, .. }) =
            component.implicit_termination_origin()
        {
            diagnostics.push(
                CssRecoveryDiagnostic::new(
                    crate::error::implicit_eof(source),
                    at.span(),
                    CssRecoveryAction::RetainWithImplicitClosure,
                )
                .expect("implicit termination belongs to actual source EOF"),
            );
        }
    }
    // All implicit endings are at this source EOF. Nested recovery is
    // discovered before its enclosing group, as in existing authored reports.
    diagnostics.reverse();
    diagnostics
}
