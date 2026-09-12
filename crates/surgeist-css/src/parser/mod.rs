//! Strict CSS ingestion for Surgeist style sheets.
//!
//! This module parses CSS syntax into CSS-owned authored syntax values. It is
//! strict by design: unsupported selectors, at-rules, properties, and values are
//! errors instead of browser-style recoverable invalid declarations.
//!
//! Parse failures expose typed [`ErrorKind`] values plus source line and column
//! information so callers do not need to parse display strings.

mod background;
mod box_model;
mod counter_style;
mod effects;
mod font_face;
mod font_feature_values;
mod fragments;
pub use fragments::{
    parse_declaration, parse_font_face_descriptor_value, parse_media_query, parse_media_query_list,
    parse_property_value_text, parse_property_value_text_for_grammar, parse_rule, parse_selector,
    parse_selector_list, parse_style_block,
};
mod generated_content;
mod grid;
mod keyframes;
mod layout;
mod multicolumn;
mod nesting;
mod page;
mod queries;
// Shared checked media construction uses the same private admission engine.
pub(crate) use queries::{construct_media_condition, construct_media_query};
pub(crate) use supports::{construct_supports_condition, construct_supports_declaration};
mod recovery;
mod selectors;
mod supports;
mod timing;
mod typography;
mod values;
mod variables;

use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, Delimiter, ParseError, Parser, ParserInput,
    ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, Token,
    match_ignore_ascii_case,
};

use background::*;
use box_model::*;
use counter_style::{parse_counter_style_name, parse_counter_style_rule};
use effects::*;
use font_face::parse_font_face_rule;
use generated_content::*;
use grid::*;
use keyframes::{parse_keyframes_name, parse_keyframes_rule};
use layout::*;
use multicolumn::*;
use nesting::{parse_style_contents, parse_style_rule_block};
use page::{parse_page_rule, parse_page_selector};
#[cfg(test)]
pub(crate) use queries::parse_container_condition_for_test;
#[cfg(test)]
pub(crate) use queries::parse_media_query_list_for_test;
use queries::{parse_container_condition, parse_media_query_list as parse_media_query_list_inner};
use recovery::{
    GroupKind, RecoveryLoopOutcome, RecoveryProgress, RecoveryState, StructuralParent,
    StructuralPreflightOutcome, StyleContextCaptures, preflight_specialized_eof_limit,
    preflight_structural_nesting, recovery_action_for_error,
};
use selectors::{
    SelectorRecovery, parse_rule_selector_list, parse_scope_boundary_selector_list,
    parse_scoped_style_selector_list,
};
use supports::{
    parse_supports_condition, parse_supports_declaration, with_supports_prelude_context,
};
use timing::*;
use typography::*;
pub(crate) use values::numeric_relative_channel;
use values::*;
use variables::{
    collect_authored_declaration_value, parse_custom_property_name, parse_custom_property_value,
};

use crate::component_values::{CssComponentValues, CssParsedOrigin, CssSourceSnapshot};
use crate::error::{
    CssFeatureId, Error, basic, from_parse_error, from_rule_parse_error, invalid_at_rule_block,
    invalid_at_rule_body, invalid_at_rule_placement, invalid_custom_declaration_annotation,
    invalid_descriptor_annotation, invalid_encoding_declaration,
    invalid_known_declaration_annotation, invalid_root_syntax, invalid_syntax,
    is_nesting_limit_error, normalize_encoding_error, property_name_error, unsupported_value,
    with_at_rule_prelude_context, with_encoding_declaration_context, with_media_query_context,
    with_property_context,
};
use crate::properties::*;
use crate::syntax::*;
use crate::validation::parse_global_keyword;

#[expect(
    dead_code,
    reason = "private atomic implementation reconciliation metadata"
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CssAtomicImplementationKind {
    Rule,
    QualifiedRule,
    Declaration,
    Descriptor,
    Selector,
    Media,
    SharedValue,
    PropertyExtension,
    ContainerExtension,
}

#[expect(
    dead_code,
    reason = "private atomic implementation reconciliation metadata"
)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct CssAtomicImplementationInventory {
    pub(crate) module: &'static str,
    pub(crate) kind: CssAtomicImplementationKind,
    pub(crate) stable_ids: &'static [CssFeatureId],
}

static IMPLEMENTED_RULES: &[CssFeatureId] = &[
    CssFeatureId::new("baseline.rule.import"),
    CssFeatureId::new("ext.import.layer"),
    CssFeatureId::new("ext.stylesheet.prelude-order"),
    CssFeatureId::new("baseline.rule.layer-statement"),
    CssFeatureId::new("baseline.rule.layer-block"),
    CssFeatureId::new("baseline.rule.media"),
    CssFeatureId::new("official.rule.conditional-group-context"),
    CssFeatureId::new("baseline.rule.scope"),
    CssFeatureId::new("foundation.encoding.charset"),
    CssFeatureId::new("later.rule.namespace"),
    CssFeatureId::new("later.rule.supports"),
    CssFeatureId::new("later.rule.counter-style"),
    CssFeatureId::new("later.rule.page"),
    CssFeatureId::new("later.rule.font-feature-values"),
    CssFeatureId::new("ext.rule.custom-media"),
];

static IMPLEMENTED_QUALIFIED_RULES: &[CssFeatureId] = &[CssFeatureId::new("baseline.rule.style")];

static IMPLEMENTED_DECLARATIONS: &[CssFeatureId] = &[
    CssFeatureId::new("foundation.declaration-list.style-attribute"),
    CssFeatureId::new("foundation.declaration.importance"),
    CssFeatureId::new("official.declaration.generic"),
];

static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("official.value.stylesheet"),
    CssFeatureId::new("official.value.declaration-list"),
];

static IMPLEMENTED_CONTAINER_EXTENSIONS: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.rule.container")];

static ATOMIC_IMPLEMENTATION_INVENTORIES: &[CssAtomicImplementationInventory] = &[
    CssAtomicImplementationInventory {
        module: "crate::parser",
        kind: CssAtomicImplementationKind::Rule,
        stable_ids: IMPLEMENTED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser",
        kind: CssAtomicImplementationKind::QualifiedRule,
        stable_ids: IMPLEMENTED_QUALIFIED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser",
        kind: CssAtomicImplementationKind::Declaration,
        stable_ids: IMPLEMENTED_DECLARATIONS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::recovery",
        kind: CssAtomicImplementationKind::Rule,
        stable_ids: recovery::IMPLEMENTED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::recovery",
        kind: CssAtomicImplementationKind::QualifiedRule,
        stable_ids: recovery::IMPLEMENTED_QUALIFIED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::recovery",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: recovery::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser",
        kind: CssAtomicImplementationKind::ContainerExtension,
        stable_ids: IMPLEMENTED_CONTAINER_EXTENSIONS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::font_face",
        kind: CssAtomicImplementationKind::Rule,
        stable_ids: font_face::IMPLEMENTED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::font_feature_values",
        kind: CssAtomicImplementationKind::Rule,
        stable_ids: font_feature_values::IMPLEMENTED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::font_face",
        kind: CssAtomicImplementationKind::Descriptor,
        stable_ids: font_face::IMPLEMENTED_DESCRIPTORS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::font_face",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: font_face::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::keyframes",
        kind: CssAtomicImplementationKind::Rule,
        stable_ids: keyframes::IMPLEMENTED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::nesting",
        kind: CssAtomicImplementationKind::Selector,
        stable_ids: nesting::IMPLEMENTED_SELECTORS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::page",
        kind: CssAtomicImplementationKind::Selector,
        stable_ids: page::IMPLEMENTED_SELECTORS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::queries",
        kind: CssAtomicImplementationKind::Media,
        stable_ids: queries::IMPLEMENTED_MEDIA,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::queries",
        kind: CssAtomicImplementationKind::ContainerExtension,
        stable_ids: queries::IMPLEMENTED_CONTAINER_EXTENSIONS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::selectors",
        kind: CssAtomicImplementationKind::Selector,
        stable_ids: selectors::IMPLEMENTED_SELECTORS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::supports",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: supports::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::supports",
        kind: CssAtomicImplementationKind::Selector,
        stable_ids: supports::IMPLEMENTED_SELECTORS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::variables",
        kind: CssAtomicImplementationKind::Declaration,
        stable_ids: variables::IMPLEMENTED_DECLARATIONS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::variables",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: variables::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::values",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: values::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::box_model",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: box_model::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::background",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: background::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::grid",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: grid::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::effects",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: effects::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::timing",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: timing::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::typography",
        kind: CssAtomicImplementationKind::PropertyExtension,
        stable_ids: typography::IMPLEMENTED_PROPERTY_EXTENSIONS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::typography",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: typography::IMPLEMENTED_SHARED_VALUES,
    },
];

pub(crate) const fn atomic_implementation_inventories()
-> &'static [CssAtomicImplementationInventory] {
    ATOMIC_IMPLEMENTATION_INVENTORIES
}

macro_rules! define_property_dispatch {
    ($input:ident, $numeric:ident;
        All, $all_canonical:literal, [$($all_alias:literal),*], $all_stable_id:literal,
        $all_value:ty, $all_wrapper:ident, $all_representation:ident,
        $all_parser:ident, $all_dispatch:block $(, expansion = $all_expansion:ident { $($all_metadata:tt)* })?;
        $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $wrapper:ident, $representation:ident, $parser:ident, $dispatch:block
        $(, expansion = $expansion:ident { $($metadata:tt)* })?;
    )*) => {
        fn parse_known_property_value<'i, 't>(
            property: crate::CssKnownProperty,
            authored: CssAuthoredDeclarationValue,
            $input: &mut Parser<'i, 't>,
            $numeric: &crate::numeric::NumericInputContext<'_>,
        ) -> std::result::Result<CssKnownDeclaration, ParseError<'i, Error>> {
            match property {
                crate::CssKnownProperty::All => {
                    let _authored_value_type = std::marker::PhantomData::<$all_value>;
                    let _wrapper_type = std::marker::PhantomData::<$all_wrapper>;
                    let _representation_type = stringify!($all_representation);
                    let keyword = $all_dispatch;
                    Ok(CssKnownDeclaration::from_global(CssKnownProperty::All, keyword))
                }
                $(crate::CssKnownProperty::$variant => {
                    let _authored_value_type = std::marker::PhantomData::<$value>;
                    let _representation_type = stringify!($representation);
                    let value = $dispatch;
                    Ok(CssKnownDeclaration::from_value(
                        CssKnownDeclarationValue::$variant(CssDeclaredValue::Value(
                            $wrapper::new(authored, value),
                        )),
                    ))
                },)*
            }
        }
    };
}

property_schema!(define_property_dispatch, input, numeric);

fn parse_all_property<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGlobalKeyword, ParseError<'i, Error>> {
    Err(unsupported_value(
        input,
        None,
        "`all` only accepts CSS-wide global keywords",
    ))
}

fn parse_overflow_property<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOverflowI01PropertyValue, ParseError<'i, Error>> {
    match parse_overflow_value(input)? {
        CssValue::Overflow(value) => Ok(CssOverflowI01PropertyValue::Single(value)),
        CssValue::OverflowAxes(value) => Ok(CssOverflowI01PropertyValue::Pair(value)),
    }
}

/// Parses a UTF-8 stylesheet into valid authored syntax and recovery diagnostics.
///
/// The ordinary parser retains valid top-level rules in source order and reports
/// each discarded top-level rule with its complete balanced source span. A valid
/// leading legacy `@charset` declaration is metadata only and never decodes the
/// already-UTF-8 input. The stylesheet root is structural depth zero; up to 256
/// shared rule-block/component/function levels are retained, and the first level
/// beyond that drops its smallest enclosing recovery unit. Recovery does not
/// apply cascade, substitution, selector matching, contextual resolution, or
/// resource loading.
///
/// ```
/// use surgeist_css::{CssRecoveryAction, CssRule, parse_sheet};
///
/// let report = parse_sheet(
///     ".before { color: red; } @unknown fn({x;y}); .after { color: blue; }",
/// );
/// assert_eq!(report.syntax().rules().len(), 2);
/// assert!(!report.is_clean());
/// assert!(matches!(report.syntax().rules()[0], CssRule::Style(_)));
/// assert!(matches!(
///     report.diagnostics()[0].action(),
///     CssRecoveryAction::DropAtRule
/// ));
/// ```
pub fn parse_sheet(source: &str) -> crate::CssParseReport<CssSheet> {
    let source_snapshot = CssSourceSnapshot::new(source);
    if recovery::maximum_nested_depth(source) > recovery::DIRECT_PARSE_DEPTH {
        // The public limit is intentionally higher than the platform's small
        // default test-thread stack. A bounded parser thread preserves the exact
        // 256-level contract without making ordinary shallow parses pay for it.
        let report = std::thread::scope(|scope| {
            let parser = std::thread::Builder::new()
                .name("surgeist-css-bounded-parser".to_owned())
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    parse_sheet_bounded(
                        source,
                        &source_snapshot,
                        0,
                        BoundedParseContext::Sheet,
                        StyleContextCaptures::default(),
                    )
                });
            match parser {
                Ok(parser) => match parser.join() {
                    Ok(report) => report,
                    Err(panic) => std::panic::resume_unwind(panic),
                },
                Err(_) => parse_sheet_bounded(
                    source,
                    &source_snapshot,
                    0,
                    BoundedParseContext::Sheet,
                    StyleContextCaptures::default(),
                ),
            }
        });
        return recovery::finish_report(source, report);
    }
    let report = parse_sheet_bounded(
        source,
        &source_snapshot,
        0,
        BoundedParseContext::Sheet,
        StyleContextCaptures::default(),
    );
    recovery::finish_report(source, report)
}

/// Parses a UTF-8 style attribute into valid ordinary declarations and recovery diagnostics.
///
/// The parser accepts an empty declaration list and an optional final semicolon. Each invalid
/// declaration candidate is discarded independently, so later valid declarations remain
/// eligible. Retained declarations use the same property, custom-property, substitution, and
/// importance grammar as declarations in ordinary style-rule blocks. At-rules, qualified rules,
/// and other non-declaration input never produce a rule node from this front door.
///
/// ```
/// use surgeist_css::{CssPropertyNameRef, CssRecoveryAction, parse_style_attribute};
///
/// let report = parse_style_attribute("color: red; @unknown x; width: 2px !important;");
/// assert_eq!(report.syntax().len(), 2);
/// assert!(matches!(
///     report.syntax()[1].property_name(),
///     CssPropertyNameRef::Known(property) if property.canonical_name() == "width"
/// ));
/// assert!(matches!(
///     report.diagnostics()[0].action(),
///     CssRecoveryAction::DropDeclaration
/// ));
/// ```
#[must_use]
pub fn parse_style_attribute(source: &str) -> crate::CssParseReport<CssDeclarationList> {
    if recovery::maximum_nested_depth(source) > recovery::DIRECT_PARSE_DEPTH {
        let report = std::thread::scope(|scope| {
            let parser = std::thread::Builder::new()
                .name("surgeist-css-bounded-style-attribute-parser".to_owned())
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || parse_style_attribute_inner(source));
            match parser {
                Ok(parser) => match parser.join() {
                    Ok(report) => report,
                    Err(panic) => std::panic::resume_unwind(panic),
                },
                Err(_) => parse_style_attribute_inner(source),
            }
        });
        return recovery::finish_report(source, report);
    }
    recovery::finish_report(source, parse_style_attribute_inner(source))
}

fn parse_style_attribute_inner(source: &str) -> crate::CssParseReport<CssDeclarationList> {
    let recovery = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    let recovered = parse_style_attribute_declarations(source, &mut parser, recovery.clone());
    let syntax = recovered.syntax;
    let mut diagnostics = recovered.diagnostics;
    diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
    crate::CssParseReport::new(syntax, diagnostics)
}

#[derive(Clone)]
enum BoundedParseContext {
    Sheet,
    Style,
    Scoped { has_style_ancestor: bool },
}

impl BoundedParseContext {
    fn is_style(&self) -> bool {
        matches!(self, Self::Style)
    }
    fn has_style_ancestor(&self) -> bool {
        matches!(
            self,
            Self::Style
                | Self::Scoped {
                    has_style_ancestor: true
                }
        )
    }
}

fn parse_sheet_bounded(
    source: &str,
    source_snapshot: &CssSourceSnapshot,
    base_depth: u32,
    context: BoundedParseContext,
    style_context_captures: StyleContextCaptures,
) -> crate::CssParseReport<CssSheet> {
    let report = parse_sheet_bounded_with_captures(
        source,
        source_snapshot,
        base_depth,
        context,
        style_context_captures,
    );
    let (sheet, mut diagnostics) = report.into_parts();
    let eof_limit = diagnostics.iter().any(|diagnostic| {
        diagnostic.action() == crate::CssRecoveryAction::StopAtNestingLimit
            && diagnostic.span().end().byte_offset().value() == source.len()
    });
    if eof_limit {
        diagnostics.retain(|diagnostic| {
            diagnostic.action() != crate::CssRecoveryAction::RetainWithImplicitClosure
        });
    }
    crate::CssParseReport::new(sheet, diagnostics)
}

fn parse_sheet_bounded_with_captures(
    source: &str,
    source_snapshot: &CssSourceSnapshot,
    base_depth: u32,
    context: BoundedParseContext,
    style_context_captures: StyleContextCaptures,
) -> crate::CssParseReport<CssSheet> {
    if let Some(limit) = preflight_specialized_eof_limit(source, base_depth) {
        let masked = mask_source_span(source, limit.unit_start, source.len());
        let outer = parse_sheet_bounded_with_captures(
            &masked,
            source_snapshot,
            base_depth,
            context,
            style_context_captures,
        );
        let (sheet, mut diagnostics) = outer.into_parts();
        diagnostics.retain(|diagnostic| {
            diagnostic.action() != crate::CssRecoveryAction::RetainWithImplicitClosure
        });
        let error = from_parse_error(
            source,
            crate::error::nesting_limit(
                source,
                limit.opening_offset,
                recovery::STRUCTURAL_NESTING_LIMIT,
                limit.enclosing_production,
            ),
        );
        if let Some(span) = crate::CssSourceSpan::new(
            crate::CssSourcePosition::from_byte_offset_in(source, limit.unit_start),
            crate::CssSourcePosition::from_byte_offset_in(source, source.len()),
        ) && let Some(diagnostic) = crate::CssRecoveryDiagnostic::new(
            error,
            span,
            crate::CssRecoveryAction::StopAtNestingLimit,
        ) {
            diagnostics.push(diagnostic);
        }
        return crate::CssParseReport::new(sheet, diagnostics);
    }
    let Some(preflight) = preflight_structural_nesting(
        source,
        base_depth,
        context.is_style(),
        context.has_style_ancestor(),
    ) else {
        let recovery = RecoveryState::at_depth_with_snapshot(
            source,
            base_depth,
            style_context_captures,
            source_snapshot.clone(),
        );
        return match context {
            BoundedParseContext::Sheet => parse_sheet_inner(source, recovery),
            BoundedParseContext::Style => parse_style_context_inner(source, recovery),
            BoundedParseContext::Scoped { has_style_ancestor } => {
                parse_scoped_context_inner(source, recovery, has_style_ancestor)
            }
        };
    };
    if matches!(&preflight.outcome, StructuralPreflightOutcome::Split) {
        for &content_start in preflight
            .style_context_starts
            .iter()
            .chain(&preflight.style_ancestry_starts)
        {
            style_context_captures.register(content_start);
        }
    }
    let masked = mask_source_span(source, preflight.unit_start, preflight.unit_end);
    // Parse at most one bounded structural chunk at a time. Same-length masks
    // retain original byte/line coordinates, and the completed child syntax is
    // spliced back into its parser-produced enclosing groups.
    let scoped_context = matches!(context, BoundedParseContext::Scoped { .. });
    let outer = parse_sheet_bounded_with_captures(
        &masked,
        source_snapshot,
        base_depth,
        context,
        style_context_captures.clone(),
    );
    let (outer_sheet, mut diagnostics) = outer.into_parts();

    match preflight.outcome {
        StructuralPreflightOutcome::Split => {
            let isolated = isolate_source_span(source, preflight.unit_start, preflight.unit_end);
            let has_style_ancestor =
                preflight
                    .style_ancestry_starts
                    .last()
                    .is_some_and(|&content_start| {
                        style_context_captures.contains_parsed(content_start)
                    });
            let child_context = if preflight
                .style_context_starts
                .last()
                .is_some_and(|&content_start| style_context_captures.contains_parsed(content_start))
            {
                BoundedParseContext::Style
            } else if scoped_context
                || preflight
                    .parents
                    .iter()
                    .any(|parent| matches!(parent.kind, GroupKind::Scope))
            {
                BoundedParseContext::Scoped { has_style_ancestor }
            } else {
                BoundedParseContext::Sheet
            };
            let child = parse_sheet_bounded(
                &isolated,
                source_snapshot,
                preflight.parent_depth,
                child_context,
                style_context_captures,
            );
            let (child_sheet, mut child_diagnostics) = child.into_parts();
            diagnostics.append(&mut child_diagnostics);
            let sheet = splice_preflight_rules(
                &outer_sheet,
                &preflight.parents,
                preflight.unit_start,
                child_sheet.rules().to_vec(),
            );
            crate::CssParseReport::new(sheet, diagnostics)
        }
        StructuralPreflightOutcome::NestingLimit {
            opening_offset,
            enclosing_production,
        } => {
            diagnostics.retain(|diagnostic| {
                diagnostic.action() != crate::CssRecoveryAction::RetainWithImplicitClosure
            });
            let error = from_parse_error(
                source,
                crate::error::nesting_limit(
                    source,
                    opening_offset,
                    recovery::STRUCTURAL_NESTING_LIMIT,
                    enclosing_production,
                ),
            );
            if let Some(span) = crate::CssSourceSpan::new(
                crate::CssSourcePosition::from_byte_offset_in(source, preflight.unit_start),
                crate::CssSourcePosition::from_byte_offset_in(source, preflight.unit_end),
            ) && let Some(diagnostic) = crate::CssRecoveryDiagnostic::new(
                error,
                span,
                crate::CssRecoveryAction::StopAtNestingLimit,
            ) {
                diagnostics.push(diagnostic);
            }
            let sheet = splice_preflight_rules(
                &outer_sheet,
                &preflight.parents,
                preflight.unit_start,
                Vec::new(),
            );
            crate::CssParseReport::new(sheet, diagnostics)
        }
    }
}

fn parse_style_context_inner(
    source: &str,
    recovery: RecoveryState,
) -> crate::CssParseReport<CssSheet> {
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    let recovered = parse_style_contents(source, &mut parser, recovery.clone());
    match recovered {
        Ok(recovered) => {
            let mut sheet = CssSheet::new();
            for rule in recovered.syntax.into_nested_rules() {
                sheet.push_rule(rule);
            }
            let mut diagnostics = recovered.diagnostics;
            diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(sheet, diagnostics)
        }
        Err(error) => {
            let action =
                recovery_action_for_error(&error, crate::CssRecoveryAction::DropQualifiedRule);
            let error = from_parse_error(source, error);
            let diagnostics = crate::CssSourceSpan::new(
                crate::CssSourcePosition::from_byte_offset_in(source, 0),
                crate::CssSourcePosition::from_byte_offset_in(source, source.len()),
            )
            .and_then(|span| crate::CssRecoveryDiagnostic::new(error, span, action))
            .into_iter()
            .collect();
            crate::CssParseReport::new(CssSheet::new(), diagnostics)
        }
    }
}

fn parse_scoped_context_inner(
    source: &str,
    recovery: RecoveryState,
    has_style_ancestor: bool,
) -> crate::CssParseReport<CssSheet> {
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    match parse_scoped_rule_list(source, &mut parser, recovery.clone(), has_style_ancestor) {
        Ok(recovered) => {
            let mut sheet = CssSheet::new();
            for rule in recovered.syntax.rules().iter().cloned() {
                sheet.push_rule(scoped_rule_into_chunk_rule(rule));
            }
            let mut diagnostics = recovered.diagnostics;
            diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(sheet, diagnostics)
        }
        Err(error) => {
            let action =
                recovery_action_for_error(&error, crate::CssRecoveryAction::DropQualifiedRule);
            let error = from_parse_error(source, error);
            let diagnostics = crate::CssSourceSpan::new(
                crate::CssSourcePosition::from_byte_offset_in(source, 0),
                crate::CssSourcePosition::from_byte_offset_in(source, source.len()),
            )
            .and_then(|span| crate::CssRecoveryDiagnostic::new(error, span, action))
            .into_iter()
            .collect();
            crate::CssParseReport::new(CssSheet::new(), diagnostics)
        }
    }
}

// The bounded parser's internal carrier holds already-validated chunks. Scoped selector
// context remains in the selector model; splicing restores scoped rule-list wrappers.
// This conversion never reparses a scoped prelude as an ordinary stylesheet selector.
fn scoped_rule_into_chunk_rule(rule: CssScopedRule) -> CssRule {
    match rule {
        CssScopedRule::Style(rule) => CssRule::Style(CssStyleRule::new(
            CssStyleSelectorList::new(
                rule.selectors()
                    .selectors()
                    .iter()
                    .map(|selector| match selector {
                        CssScopedStyleSelector::Selector(selector) => {
                            CssStyleSelector::Selector(selector.clone())
                        }
                        CssScopedStyleSelector::Relative(relative) => {
                            CssStyleSelector::Relative(relative.clone())
                        }
                    })
                    .collect(),
            ),
            rule.declarations().clone(),
            rule.rules().to_vec(),
            rule.position(),
        )),
        CssScopedRule::Media(rule) => CssRule::Media(CssMediaRule::new(
            rule.query().clone(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position(),
        )),
        CssScopedRule::Supports(rule) => CssRule::Supports(CssSupportsRule::new(
            rule.condition().clone(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position(),
        )),
        CssScopedRule::Container(rule) => CssRule::Container(CssContainerRule::new(
            rule.name().cloned(),
            rule.condition().clone(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position(),
        )),
        CssScopedRule::LayerBlock(rule) => CssRule::LayerBlock(CssLayerBlockRule::new(
            rule.name().cloned(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position(),
        )),
        CssScopedRule::LayerStatement(rule) => CssRule::LayerStatement(CssLayerStatementRule::new(
            rule.names().clone(),
            rule.position(),
        )),
        CssScopedRule::Scope(rule) => CssRule::Scope(rule),
        CssScopedRule::CustomMedia(rule) => CssRule::CustomMedia(rule),
        CssScopedRule::FontFeatureValues(rule) => CssRule::FontFeatureValues(rule),
    }
}

fn mask_source_span(source: &str, start: usize, end: usize) -> String {
    let mut masked = source.as_bytes().to_vec();
    for byte in masked
        .get_mut(start.min(source.len())..end.min(source.len()))
        .into_iter()
        .flatten()
    {
        if !matches!(*byte, b'\n' | b'\r' | b'\x0c') {
            *byte = b' ';
        }
    }
    String::from_utf8(masked).expect("ASCII masking preserves UTF-8")
}

fn isolate_source_span(source: &str, start: usize, end: usize) -> String {
    let mut isolated = source.as_bytes().to_vec();
    for (offset, byte) in isolated.iter_mut().enumerate() {
        if (offset < start || offset >= end) && !matches!(*byte, b'\n' | b'\r' | b'\x0c') {
            *byte = b' ';
        }
    }
    String::from_utf8(isolated).expect("ASCII masking preserves UTF-8")
}

fn splice_preflight_rules(
    sheet: &CssSheet,
    parents: &[StructuralParent],
    child_start: usize,
    child_rules: Vec<CssRule>,
) -> CssSheet {
    let rules = splice_rule_list(sheet.rules(), parents, child_start, child_rules);
    let mut rebuilt = CssSheet::new();
    if let Some(encoding) = sheet.encoding().cloned() {
        rebuilt.set_encoding(encoding);
    }
    for rule in rules {
        rebuilt.push_rule(rule);
    }
    rebuilt
}

fn splice_rule_list(
    rules: &[CssRule],
    parents: &[StructuralParent],
    child_start: usize,
    child_rules: Vec<CssRule>,
) -> Vec<CssRule> {
    if parents.is_empty() {
        let mut combined = rules
            .iter()
            .cloned()
            .flat_map(|rule| split_style_declaration_run(rule, child_start))
            .collect::<Vec<_>>();
        let insertion = combined
            .iter()
            .position(|rule| rule_start(rule) > child_start)
            .unwrap_or(combined.len());
        combined.splice(insertion..insertion, child_rules);
        return combined;
    }
    let parent = &parents[0];
    rules
        .iter()
        .cloned()
        .map(|rule| {
            if rule_start(&rule) != parent.start {
                return rule;
            }
            if matches!(parent.kind, GroupKind::Scope)
                && let CssRule::Scope(scope) = rule
            {
                let rebuilt = splice_scoped_rule_list(
                    scope.rules().rules(),
                    &parents[1..],
                    child_start,
                    child_rules.clone(),
                );
                return CssRule::Scope(CssScopeRule::new(
                    scope.root().cloned(),
                    scope.limit().cloned(),
                    CssScopedRuleList::from_rules(rebuilt),
                    scope.position(),
                ));
            }
            if let CssRule::Style(style) = rule {
                let (declarations, rules) = splice_style_body(
                    style.declarations(),
                    style.rules(),
                    &parents[1..],
                    child_start,
                    child_rules.clone(),
                );
                return CssRule::Style(CssStyleRule::new(
                    style.selectors().clone(),
                    declarations,
                    rules,
                    style.position(),
                ));
            }
            let nested = group_rules(&rule).unwrap_or_default();
            let rebuilt = splice_rule_list(nested, &parents[1..], child_start, child_rules.clone());
            rebuild_group_rule(rule, rebuilt)
        })
        .collect()
}

fn splice_style_body(
    leading: &CssDeclarationList,
    rules: &[CssRule],
    parents: &[StructuralParent],
    child_start: usize,
    child_rules: Vec<CssRule>,
) -> (CssDeclarationList, Vec<CssRule>) {
    let mut declarations = leading.as_slice().to_vec();
    let mut nested = rules.to_vec();
    if parents.is_empty() && !child_rules.is_empty() {
        let split = declarations.partition_point(|declaration| {
            declaration
                .position()
                .expect("parsed declarations have source positions")
                .byte_offset()
                .value()
                < child_start
        });
        let trailing = declarations.split_off(split);
        if !trailing.is_empty() {
            nested.insert(
                0,
                CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(
                    CssDeclarationList::new(trailing),
                )),
            );
        }
    }
    let rebuilt = splice_rule_list(&nested, parents, child_start, child_rules);
    (CssDeclarationList::new(declarations), rebuilt)
}

fn splice_scoped_rule_list(
    rules: &[CssScopedRule],
    parents: &[StructuralParent],
    child_start: usize,
    child_rules: Vec<CssRule>,
) -> Vec<CssScopedRule> {
    if parents.is_empty() {
        let mut combined = rules.to_vec();
        let insertion = combined
            .iter()
            .position(|rule| scoped_rule_start(rule) > child_start)
            .unwrap_or(combined.len());
        combined.splice(
            insertion..insertion,
            child_rules.into_iter().filter_map(into_scoped_rule),
        );
        return combined;
    }
    let parent = &parents[0];
    rules
        .iter()
        .cloned()
        .map(|rule| {
            if scoped_rule_start(&rule) != parent.start {
                return rule;
            }
            if let CssScopedRule::Style(style) = rule {
                let (declarations, nested) = splice_style_body(
                    style.declarations(),
                    style.rules(),
                    &parents[1..],
                    child_start,
                    child_rules.clone(),
                );
                return CssScopedRule::Style(CssScopedStyleRule::new(
                    style.selectors().clone(),
                    declarations,
                    nested,
                    style.position(),
                ));
            }
            let nested = scoped_group_rules(&rule).unwrap_or_default();
            let rebuilt =
                splice_scoped_rule_list(nested, &parents[1..], child_start, child_rules.clone());
            rebuild_scoped_group_rule(rule, rebuilt)
        })
        .collect()
}

fn split_style_declaration_run(rule: CssRule, child_start: usize) -> Vec<CssRule> {
    let CssRule::NestedDeclarations(run) = rule else {
        return vec![rule];
    };
    let declarations = run.declarations().as_slice();
    let split = declarations.partition_point(|declaration| {
        declaration
            .position()
            .expect("parsed declarations have source positions")
            .byte_offset()
            .value()
            < child_start
    });
    if split == 0 || split == declarations.len() {
        return vec![CssRule::NestedDeclarations(run)];
    }
    vec![
        CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(CssDeclarationList::new(
            declarations[..split].to_vec(),
        ))),
        CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(CssDeclarationList::new(
            declarations[split..].to_vec(),
        ))),
    ]
}

fn scoped_group_rules(rule: &CssScopedRule) -> Option<&[CssScopedRule]> {
    match rule {
        CssScopedRule::Media(rule) => Some(rule.rules().rules()),
        CssScopedRule::Supports(rule) => Some(rule.rules().rules()),
        CssScopedRule::Container(rule) => Some(rule.rules().rules()),
        CssScopedRule::LayerBlock(rule) => Some(rule.rules().rules()),
        CssScopedRule::Scope(rule) => Some(rule.rules().rules()),
        _ => None,
    }
}

fn rebuild_scoped_group_rule(rule: CssScopedRule, rules: Vec<CssScopedRule>) -> CssScopedRule {
    let rules = CssScopedRuleList::from_rules(rules);
    match rule {
        CssScopedRule::Media(rule) => CssScopedRule::Media(CssScopedMediaRule::new(
            rule.query().clone(),
            rules,
            rule.position(),
        )),
        CssScopedRule::Supports(rule) => CssScopedRule::Supports(CssScopedSupportsRule::new(
            rule.condition().clone(),
            rules,
            rule.position(),
        )),
        CssScopedRule::Container(rule) => CssScopedRule::Container(CssScopedContainerRule::new(
            rule.name().cloned(),
            rule.condition().clone(),
            rules,
            rule.position(),
        )),
        CssScopedRule::LayerBlock(rule) => CssScopedRule::LayerBlock(CssScopedLayerBlockRule::new(
            rule.name().cloned(),
            rules,
            rule.position(),
        )),
        CssScopedRule::Scope(rule) => CssScopedRule::Scope(CssScopeRule::new(
            rule.root().cloned(),
            rule.limit().cloned(),
            rules,
            rule.position(),
        )),
        _ => rule,
    }
}

fn into_scoped_rule(rule: CssRule) -> Option<CssScopedRule> {
    match rule {
        CssRule::Style(rule) => {
            let selectors = CssScopedStyleSelectorList::try_new(
                rule.selectors()
                    .selectors()
                    .iter()
                    .map(|selector| match selector {
                        CssStyleSelector::Selector(selector) => {
                            CssScopedStyleSelector::Selector(selector.clone())
                        }
                        CssStyleSelector::Relative(relative) => {
                            CssScopedStyleSelector::Relative(relative.clone())
                        }
                    })
                    .collect(),
            )?;
            Some(CssScopedRule::Style(CssScopedStyleRule::new(
                selectors,
                rule.declarations().clone(),
                rule.rules().to_vec(),
                rule.position(),
            )))
        }
        CssRule::LayerStatement(rule) => Some(CssScopedRule::LayerStatement(
            CssScopedLayerStatementRule::new(rule.names().clone(), rule.position()),
        )),
        CssRule::LayerBlock(rule) => Some(CssScopedRule::LayerBlock(CssScopedLayerBlockRule::new(
            rule.name().cloned(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position(),
        ))),
        CssRule::Media(rule) => Some(CssScopedRule::Media(CssScopedMediaRule::new(
            rule.query().clone(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position(),
        ))),
        CssRule::Supports(rule) => Some(CssScopedRule::Supports(CssScopedSupportsRule::new(
            rule.condition().clone(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position(),
        ))),
        CssRule::Container(rule) => Some(CssScopedRule::Container(CssScopedContainerRule::new(
            rule.name().cloned(),
            rule.condition().clone(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position(),
        ))),
        CssRule::Scope(rule) => Some(CssScopedRule::Scope(rule)),
        CssRule::CustomMedia(rule) => Some(CssScopedRule::CustomMedia(rule)),
        CssRule::FontFeatureValues(rule) => Some(CssScopedRule::FontFeatureValues(rule)),
        CssRule::NestedDeclarations(_)
        | CssRule::Import(_)
        | CssRule::Namespace(_)
        | CssRule::CounterStyle(_)
        | CssRule::Page(_)
        | CssRule::FontFace(_)
        | CssRule::Keyframes(_) => None,
    }
}

fn scoped_rule_start(rule: &CssScopedRule) -> usize {
    match rule {
        CssScopedRule::CustomMedia(rule) => {
            return rule
                .position()
                .expect("parsed custom-media rule")
                .byte_offset()
                .value();
        }
        CssScopedRule::FontFeatureValues(rule) => {
            return rule
                .position()
                .expect("parser-owned font rule has a source position")
                .byte_offset()
                .value();
        }
        CssScopedRule::Style(rule) => rule.position(),
        CssScopedRule::Media(rule) => rule.position(),
        CssScopedRule::Supports(rule) => rule.position(),
        CssScopedRule::Container(rule) => rule.position(),
        CssScopedRule::LayerStatement(rule) => rule.position(),
        CssScopedRule::LayerBlock(rule) => rule.position(),
        CssScopedRule::Scope(rule) => rule.position(),
    }
    .byte_offset()
    .value()
}

fn group_rules(rule: &CssRule) -> Option<&[CssRule]> {
    match rule {
        CssRule::LayerBlock(rule) => Some(rule.rules()),
        CssRule::Media(rule) => Some(rule.rules()),
        CssRule::Supports(rule) => Some(rule.rules()),
        CssRule::Container(rule) => Some(rule.rules()),
        _ => None,
    }
}

fn rebuild_group_rule(rule: CssRule, rules: Vec<CssRule>) -> CssRule {
    match rule {
        CssRule::LayerBlock(rule) => CssRule::LayerBlock(CssLayerBlockRule::new(
            rule.name().cloned(),
            rules,
            rule.position(),
        )),
        CssRule::Media(rule) => CssRule::Media(CssMediaRule::new(
            rule.query().clone(),
            rules,
            rule.position(),
        )),
        CssRule::Supports(rule) => CssRule::Supports(CssSupportsRule::new(
            rule.condition().clone(),
            rules,
            rule.position(),
        )),
        CssRule::Container(rule) => CssRule::Container(CssContainerRule::new(
            rule.name().cloned(),
            rule.condition().clone(),
            rules,
            rule.position(),
        )),
        _ => rule,
    }
}

fn rule_start(rule: &CssRule) -> usize {
    match rule {
        CssRule::CustomMedia(rule) => {
            return rule
                .position()
                .expect("parsed custom-media rule")
                .byte_offset()
                .value();
        }
        CssRule::FontFeatureValues(rule) => {
            return rule
                .position()
                .expect("parser-owned font rule has a source position")
                .byte_offset()
                .value();
        }
        CssRule::Import(rule) => rule.position(),
        CssRule::Namespace(rule) => rule.position(),
        CssRule::CounterStyle(rule) => rule.position(),
        CssRule::Page(rule) => rule.position(),
        CssRule::LayerStatement(rule) => rule.position(),
        CssRule::LayerBlock(rule) => rule.position(),
        CssRule::FontFace(rule) => rule.position(),
        CssRule::Keyframes(rule) => rule.position(),
        CssRule::Style(rule) => rule.position(),
        CssRule::NestedDeclarations(rule) => rule.position(),
        CssRule::Media(rule) => rule.position(),
        CssRule::Supports(rule) => rule.position(),
        CssRule::Container(rule) => rule.position(),
        CssRule::Scope(rule) => rule.position(),
    }
    .byte_offset()
    .value()
}

fn parse_sheet_inner(source: &str, recovery: RecoveryState) -> crate::CssParseReport<CssSheet> {
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    if source.starts_with('\u{feff}') {
        let _ = parser.next_including_whitespace_and_comments();
    }
    let mut rule_parser = StrictRuleParser::top_level(source, recovery.clone());
    let mut sheet = CssSheet::new();
    let mut diagnostics = Vec::new();
    let mut previous_end = parser.position().byte_index();

    {
        let mut rules = RuleBodyParser::new(&mut parser, &mut rule_parser);
        loop {
            let progress = RecoveryProgress::record(rules.input);
            if let Some(diagnostic) = discard_malformed_top_level_token(source, rules.input) {
                rules.parser.encoding_allowed = false;
                previous_end = diagnostic.span().end().byte_offset().value();
                diagnostics.push(diagnostic);
                if progress.finish(rules.input, false) == RecoveryLoopOutcome::Terminated {
                    break;
                }
                continue;
            }
            let Some(result) = rules.next() else {
                break;
            };
            let failed_block_error = result.as_ref().err().and_then(|(_, failed_unit)| {
                consume_failed_rule_block(
                    source,
                    rules.input,
                    true,
                    &recovery,
                    structural_recovery_production(failed_unit),
                )
                .1
            });
            let retained = result.is_ok();
            let progress_outcome = progress.finish(rules.input, retained);
            let unit_end = rules.input.position().byte_index();
            diagnostics.append(&mut rules.parser.diagnostics);
            match result {
                Ok(parsed_rules) => {
                    for rule in parsed_rules {
                        sheet.push_rule(rule);
                    }
                }
                Err((error, failed_unit)) => {
                    let error = failed_block_error.unwrap_or(error);
                    let unit_start =
                        recovery_unit_start(source, previous_end, unit_end, failed_unit);
                    let ordinary_action = if failed_unit.trim_start().starts_with('@') {
                        crate::CssRecoveryAction::DropAtRule
                    } else {
                        crate::CssRecoveryAction::DropQualifiedRule
                    };
                    let action = recovery_action_for_error(&error, ordinary_action);
                    let error = from_rule_parse_error(source, failed_unit, error);
                    let error = if recovery_at_rule_name(failed_unit)
                        .is_some_and(|name| name.eq_ignore_ascii_case("charset"))
                    {
                        normalize_encoding_error(source, unit_start, unit_end, failed_unit, error)
                    } else {
                        error
                    };
                    if let Some(span) = crate::CssSourceSpan::new(
                        crate::CssSourcePosition::from_byte_offset_in(source, unit_start),
                        crate::CssSourcePosition::from_byte_offset_in(source, unit_end),
                    ) && let Some(diagnostic) =
                        crate::CssRecoveryDiagnostic::new(error, span, action)
                    {
                        diagnostics.push(diagnostic);
                    }
                }
            }
            previous_end = unit_end;
            if progress_outcome == RecoveryLoopOutcome::Terminated {
                break;
            }
        }
    }

    if let Some(encoding) = rule_parser.encoding.take() {
        sheet.set_encoding(encoding);
    }

    diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));

    crate::CssParseReport::new(sheet, diagnostics)
}

fn discard_malformed_top_level_token(
    source: &str,
    input: &mut Parser<'_, '_>,
) -> Option<crate::CssRecoveryDiagnostic> {
    loop {
        let state = input.state();
        let token_start = input.position().byte_index();
        match input.next_including_whitespace_and_comments() {
            Ok(Token::WhiteSpace(_) | Token::Comment(_)) => {}
            Ok(token @ (Token::CDO | Token::CDC)) => {
                let token = token.clone();
                let token_end = input.position().byte_index();
                let error = crate::error::unexpected_token_at(source, token_start, &token);
                let span = crate::CssSourceSpan::new(
                    crate::CssSourcePosition::from_byte_offset_in(source, token_start),
                    crate::CssSourcePosition::from_byte_offset_in(source, token_end),
                )?;
                return crate::CssRecoveryDiagnostic::new(
                    error,
                    span,
                    crate::CssRecoveryAction::IgnoreLegacyToken,
                );
            }
            Ok(token @ (Token::Semicolon | Token::CloseCurlyBracket)) => {
                let token = token.clone();
                let token_end = input.position().byte_index();
                let error = invalid_root_syntax(source, token_start, &token);
                let span = crate::CssSourceSpan::new(
                    crate::CssSourcePosition::from_byte_offset_in(source, token_start),
                    crate::CssSourcePosition::from_byte_offset_in(source, token_end),
                )?;
                return crate::CssRecoveryDiagnostic::new(
                    error,
                    span,
                    crate::CssRecoveryAction::DropQualifiedRule,
                );
            }
            Ok(_) | Err(_) => {
                input.reset(&state);
                return None;
            }
        }
    }
}

fn discard_malformed_style_attribute_token(
    source: &str,
    input: &mut Parser<'_, '_>,
) -> Option<crate::CssRecoveryDiagnostic> {
    loop {
        let state = input.state();
        let token_start = input.position().byte_index();
        match input.next_including_whitespace_and_comments() {
            Ok(Token::WhiteSpace(_) | Token::Comment(_)) => {}
            Ok(
                token @ (Token::CloseParenthesis
                | Token::CloseSquareBracket
                | Token::CloseCurlyBracket),
            ) => {
                let token = token.clone();
                let token_end = input.position().byte_index();
                let error = crate::error::unexpected_token_at(source, token_start, &token);
                let span = crate::CssSourceSpan::new(
                    crate::CssSourcePosition::from_byte_offset_in(source, token_start),
                    crate::CssSourcePosition::from_byte_offset_in(source, token_end),
                )?;
                return crate::CssRecoveryDiagnostic::new(
                    error,
                    span,
                    crate::CssRecoveryAction::DropDeclaration,
                );
            }
            Ok(_) | Err(_) => {
                input.reset(&state);
                return None;
            }
        }
    }
}

fn recovery_unit_start(
    source: &str,
    previous_end: usize,
    unit_end: usize,
    failed_unit: &str,
) -> usize {
    let bounded_end = unit_end.min(source.len());
    let bounded_start = previous_end.min(bounded_end);
    source
        .get(bounded_start..bounded_end)
        .and_then(|bounded| bounded.find(failed_unit))
        .map_or(bounded_start, |relative| bounded_start + relative)
}

fn recovery_at_rule_name(failed_unit: &str) -> Option<&str> {
    let after_at = failed_unit.trim_start().strip_prefix('@')?;
    let name_end = after_at
        .find(|character: char| !character.is_alphanumeric() && character != '-')
        .unwrap_or(after_at.len());
    after_at.get(..name_end)
}

pub(super) struct Recovered<T> {
    pub(super) syntax: T,
    pub(super) diagnostics: Vec<crate::CssRecoveryDiagnostic>,
}

pub(super) fn block_item_diagnostic(
    source: &str,
    error: ParseError<'_, Error>,
    failed_unit: &str,
    unit_end: usize,
    action: crate::CssRecoveryAction,
) -> Option<crate::CssRecoveryDiagnostic> {
    let unit_start = unit_end.saturating_sub(failed_unit.len());
    block_item_diagnostic_from_start(source, error, unit_start, unit_end, action)
}

pub(super) fn block_item_diagnostic_from_start(
    source: &str,
    error: ParseError<'_, Error>,
    unit_start: usize,
    unit_end: usize,
    action: crate::CssRecoveryAction,
) -> Option<crate::CssRecoveryDiagnostic> {
    let action = recovery_action_for_error(&error, action);
    let error = from_parse_error(source, error);
    let span = crate::CssSourceSpan::new(
        crate::CssSourcePosition::from_byte_offset_in(source, unit_start),
        crate::CssSourcePosition::from_byte_offset_in(source, unit_end),
    )?;
    if span.start() == span.end() {
        return None;
    }
    crate::CssRecoveryDiagnostic::new(error, span, action)
}

pub(super) fn consume_failed_rule_block<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    failed: bool,
    recovery: &RecoveryState,
    enclosing_production: &'static str,
) -> (bool, Option<ParseError<'i, Error>>) {
    let position = input.position().byte_index();
    let failed_at_block = failed
        && position > 0
        && position < source.len()
        && source.as_bytes().get(position - 1) == Some(&b'{');
    if failed_at_block {
        let nesting_error = recovery.check_failed_rule_block(source, input, enclosing_production);
        let _: std::result::Result<(), ParseError<'_, ()>> = input.parse_nested_block(|nested| {
            while nested.next_including_whitespace_and_comments().is_ok() {}
            Ok(())
        });
        return (true, nesting_error);
    }
    (false, None)
}

pub(super) fn structural_rule_diagnostic(
    source: &str,
    error: ParseError<'_, Error>,
    failed_unit: &str,
    previous_end: usize,
    unit_end: usize,
    action: crate::CssRecoveryAction,
) -> Option<crate::CssRecoveryDiagnostic> {
    let unit_start = recovery_unit_start(source, previous_end, unit_end, failed_unit);
    let action = recovery_action_for_error(&error, action);
    let error = from_rule_parse_error(source, failed_unit, error);
    let span = crate::CssSourceSpan::new(
        crate::CssSourcePosition::from_byte_offset_in(source, unit_start),
        crate::CssSourcePosition::from_byte_offset_in(source, unit_end),
    )?;
    if span.start() == span.end() {
        return None;
    }
    crate::CssRecoveryDiagnostic::new(error, span, action)
}

pub(super) fn structural_recovery_action(failed_unit: &str) -> crate::CssRecoveryAction {
    if failed_unit.trim_start().starts_with('@') {
        crate::CssRecoveryAction::DropAtRule
    } else {
        crate::CssRecoveryAction::DropQualifiedRule
    }
}

pub(super) fn structural_recovery_production(failed_unit: &str) -> &'static str {
    if failed_unit.trim_start().starts_with('@') {
        "css.at-rule"
    } else {
        "css.qualified-rule"
    }
}

pub(super) fn top_level_only_at_rule_placement<'i>(
    location: cssparser::SourceLocation,
    name: &str,
) -> ParseError<'i, Error> {
    invalid_at_rule_placement(location, name, "the stylesheet top level")
}

pub(super) fn is_declaration_recovery_unit(failed_unit: &str) -> bool {
    let mut input = ParserInput::new(failed_unit);
    let mut parser = Parser::new(&mut input);
    matches!(
        parser.next_including_whitespace_and_comments(),
        Ok(Token::Ident(_))
    )
}

struct StrictRuleParser<'s> {
    source: &'s str,
    top_level_phase: Option<TopLevelPreludePhase>,
    encoding_allowed: bool,
    source_len: usize,
    encoding: Option<CssEncodingDeclaration>,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TopLevelPreludePhase {
    Initial,
    Imports,
    Namespaces,
    Body,
}

impl TopLevelPreludePhase {
    const fn accepts_import(self) -> bool {
        matches!(self, Self::Initial | Self::Imports)
    }

    const fn after_import(self) -> Self {
        match self {
            Self::Initial | Self::Imports => Self::Imports,
            Self::Namespaces | Self::Body => self,
        }
    }

    const fn after_layer_statement(self) -> Self {
        match self {
            // Cascade 5 permits initial layer statements before imports and namespaces.
            Self::Initial => Self::Initial,
            Self::Imports | Self::Namespaces | Self::Body => Self::Body,
        }
    }

    const fn after_body_rule(self) -> Self {
        Self::Body
    }

    const fn after_namespace(self) -> Option<Self> {
        match self {
            Self::Initial | Self::Imports | Self::Namespaces => Some(Self::Namespaces),
            Self::Body => None,
        }
    }
}

/// Immutable namespace bindings used to parse authored selector and rule fragments.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CssNamespaceContext(CssNamespaceBindings);
impl CssNamespaceContext {
    /// Takes bindings in authored order; the last binding for each prefix wins.
    pub fn from_bindings(
        bindings: impl IntoIterator<Item = (Option<CssNamespacePrefix>, CssNamespaceName)>,
    ) -> Self {
        let mut result = Self::default();
        for (prefix, name) in bindings {
            result.0.activate(prefix, name);
        }
        result
    }
    /// Copies the retained top-level namespace declarations from a stylesheet.
    pub fn from_sheet(sheet: &CssSheet) -> Self {
        Self::from_bindings(sheet.rules().iter().filter_map(|rule| match rule {
            CssRule::Namespace(rule) => Some((rule.prefix().cloned(), rule.name().clone())),
            _ => None,
        }))
    }
    /// Returns the default binding, distinguishing absence from the empty namespace.
    pub fn default_namespace(&self) -> Option<&CssNamespaceName> {
        self.0.default.as_ref()
    }
    /// Returns the case-sensitive named binding.
    pub fn named_namespace(&self, prefix: &CssNamespacePrefix) -> Option<&CssNamespaceName> {
        self.0
            .named
            .iter()
            .find(|(p, _)| p == prefix)
            .map(|(_, name)| name)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CssNamespaceBindings {
    default: Option<CssNamespaceName>,
    named: Vec<(CssNamespacePrefix, CssNamespaceName)>,
}

impl CssNamespaceBindings {
    fn activate(&mut self, prefix: Option<CssNamespacePrefix>, name: CssNamespaceName) {
        if let Some(prefix) = prefix {
            if let Some((_, active_name)) = self
                .named
                .iter_mut()
                .find(|(active_prefix, _)| active_prefix == &prefix)
            {
                *active_name = name;
            } else {
                self.named.push((prefix, name));
            }
        } else {
            self.default = Some(name);
        }
    }

    fn has_active_binding(
        &self,
        prefix: Option<&CssNamespacePrefix>,
        name: &CssNamespaceName,
    ) -> bool {
        if let Some(prefix) = prefix {
            self.named
                .iter()
                .find(|(active_prefix, _)| active_prefix == prefix)
                .is_some_and(|(_, active_name)| active_name == name)
        } else {
            self.default.as_ref() == Some(name)
        }
    }

    fn has_default(&self) -> bool {
        self.default.is_some()
    }

    fn active_prefix(&self, prefix: &str) -> Option<&CssNamespacePrefix> {
        self.named
            .iter()
            .find(|(active_prefix, _)| active_prefix.as_str() == prefix)
            .map(|(active_prefix, _)| active_prefix)
    }
}

impl<'s> StrictRuleParser<'s> {
    fn top_level(source: &'s str, recovery: RecoveryState) -> Self {
        Self {
            source,
            top_level_phase: Some(TopLevelPreludePhase::Initial),
            encoding_allowed: true,
            source_len: source.len(),
            encoding: None,
            diagnostics: Vec::new(),
            recovery,
        }
    }

    fn isolated_rule(source: &'s str, recovery: RecoveryState) -> Self {
        Self {
            encoding_allowed: false,
            ..Self::top_level(source, recovery)
        }
    }

    fn nested(source: &'s str, recovery: RecoveryState) -> Self {
        Self {
            source,
            top_level_phase: None,
            encoding_allowed: false,
            source_len: usize::MAX,
            encoding: None,
            diagnostics: Vec::new(),
            recovery,
        }
    }

    fn mark_successful_import(&mut self) {
        if let Some(phase) = self.top_level_phase.as_mut() {
            *phase = phase.after_import();
        }
    }

    fn mark_successful_layer_statement(&mut self) {
        if let Some(phase) = self.top_level_phase.as_mut() {
            *phase = phase.after_layer_statement();
        }
    }

    fn mark_successful_body_rule(&mut self) {
        if let Some(phase) = self.top_level_phase.as_mut() {
            *phase = phase.after_body_rule();
        }
    }

    fn mark_successful_namespace(
        &mut self,
        prefix: Option<CssNamespacePrefix>,
        name: CssNamespaceName,
    ) -> bool {
        let Some(phase) = self.top_level_phase else {
            return false;
        };
        let Some(next) = phase.after_namespace() else {
            return false;
        };
        self.top_level_phase = Some(next);
        self.recovery
            .activate_namespace(prefix.clone(), name.clone());
        debug_assert!(
            self.recovery
                .has_active_namespace_binding(prefix.as_ref(), &name)
        );
        true
    }

    fn import_is_allowed(&self) -> Option<bool> {
        self.top_level_phase
            .map(TopLevelPreludePhase::accepts_import)
    }

    fn namespace_is_allowed(&self) -> Option<bool> {
        self.top_level_phase
            .map(|phase| phase.after_namespace().is_some())
    }
}

enum StrictAtRulePrelude {
    CustomMedia(Box<CustomMediaPrelude>),
    FontFeatureValues(Vec<CssFontFaceFamily>),
    Encoding(String),
    Import(Box<CssImportPrelude>),
    Namespace(CssNamespacePrelude),
    CounterStyle(CssCounterStyleName),
    Page(Option<CssPageSelector>),
    Layer(Vec<CssLayerName>),
    FontFace,
    Keyframes(CssKeyframesName),
    Media(CssMediaQueryList),
    Supports(CssSupportsCondition),
    Container(CssContainerPrelude),
    Scope(CssScopePrelude),
}

impl StrictAtRulePrelude {
    fn production(&self) -> &'static str {
        match self {
            Self::CustomMedia(_) => "ext.rule.custom-media",
            Self::FontFeatureValues(_) => "later.rule.font-feature-values",
            Self::Encoding(_) => "css.encoding-declaration",
            Self::Import(_) => "baseline.rule.import",
            Self::Namespace(_) => "later.rule.namespace",
            Self::CounterStyle(_) => "later.rule.counter-style",
            Self::Page(_) => "later.rule.page",
            Self::Layer(_) => "baseline.rule.layer-block",
            Self::FontFace => "baseline.rule.font-face",
            Self::Keyframes(_) => "baseline.rule.keyframes",
            Self::Media(_) => "baseline.rule.media",
            Self::Supports(_) => "baseline.rule.supports",
            Self::Container(_) => "baseline.rule.container",
            Self::Scope(_) => "baseline.rule.scope",
        }
    }
}

struct CssImportPrelude {
    target: CssImportTarget,
    layer: Option<CssImportLayer>,
    supports: Option<CssImportSupports>,
    media: Option<CssMediaQueryList>,
    implicit_media_closures: Vec<usize>,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    syntax: crate::imports::ImportPreludeSyntax,
}

struct CssNamespacePrelude {
    prefix: Option<CssNamespacePrefix>,
    name: CssNamespaceName,
}

struct CssContainerPrelude {
    name: Option<CssContainerName>,
    condition: CssContainerCondition,
}

struct CssScopePrelude {
    root: Option<CssScopeSelectorList>,
    limit: Option<CssScopeSelectorList>,
}

impl<'i> AtRuleParser<'i> for StrictRuleParser<'i> {
    type Prelude = StrictAtRulePrelude;
    type AtRule = Vec<CssRule>;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        if name.eq_ignore_ascii_case("charset") {
            let encoding_allowed = self.encoding_allowed;
            self.encoding_allowed = false;
            if !encoding_allowed {
                return Err(invalid_encoding_declaration(
                    input.current_source_location(),
                ));
            }

            let prelude_start = input.position();
            let label = input
                .expect_string_cloned()
                .map_err(|error| with_encoding_declaration_context(error.into()))?;
            let authored = input.slice(prelude_start..input.position());
            let quote = authored.trim_start().as_bytes().first().copied();
            if !matches!(quote, Some(b'"')) || label.is_empty() {
                return Err(with_encoding_declaration_context(
                    input.new_unexpected_token_error(cssparser::Token::QuotedString(label)),
                ));
            }
            if !input.is_exhausted() {
                let token = input.next_including_whitespace_and_comments()?.clone();
                return Err(with_encoding_declaration_context(
                    input.new_error(cssparser::BasicParseErrorKind::UnexpectedToken(token)),
                ));
            }
            if input.position().byte_index() == self.source_len {
                return Err(with_encoding_declaration_context(
                    input.new_error(cssparser::BasicParseErrorKind::EndOfInput),
                ));
            }
            return Ok(StrictAtRulePrelude::Encoding(label.to_string()));
        }

        self.encoding_allowed = false;
        match_ignore_ascii_case! { &name,
            "import" => {
                let Some(import_is_allowed) = self.import_is_allowed() else {
                    return Err(invalid_at_rule_placement(
                        input.current_source_location(),
                        "import",
                        "the stylesheet top level",
                    ));
                };
                if !import_is_allowed {
                    return Err(invalid_at_rule_placement(
                        input.current_source_location(),
                        "import",
                        "before every non-import top-level rule",
                    ));
                }
                let prelude = parse_import_prelude(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                ).map_err(|error| {
                    if queries::media_terminal_error(&error) { return error; }
                    with_at_rule_prelude_context(
                        error,
                        "import",
                        "baseline.rule.import",
                        "a supported @import prelude",
                    )
                })?;
                Ok(StrictAtRulePrelude::Import(Box::new(prelude)))
            },
            "namespace" => {
                let Some(namespace_is_allowed) = self.namespace_is_allowed() else {
                    return Err(invalid_at_rule_placement(
                        input.current_source_location(),
                        "namespace",
                        "the stylesheet top level",
                    ));
                };
                if !namespace_is_allowed {
                    return Err(invalid_at_rule_placement(
                        input.current_source_location(),
                        "namespace",
                        "after initial layer statements and imports, before later layers or body rules",
                    ));
                }
                let prelude = parse_namespace_prelude(self.source, input).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "namespace",
                        "later.rule.namespace",
                        "an optional prefix followed by one string or URL namespace name",
                    )
                })?;
                Ok(StrictAtRulePrelude::Namespace(prelude))
            },
            "counter-style" => {
                if self.top_level_phase.is_none() {
                    return Err(top_level_only_at_rule_placement(
                        input.current_source_location(),
                        "counter-style",
                    ));
                }
                let name = parse_counter_style_name(input).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "counter-style",
                        "later.rule.counter-style",
                        "one non-reserved counter-style name",
                    )
                })?;
                let following = self
                    .source
                    .get(input.position().byte_index()..)
                    .unwrap_or_default()
                    .trim_start();
                if following.is_empty() || following.starts_with(';') {
                    return Err(invalid_at_rule_body(
                        input,
                        "counter-style",
                        "later.rule.counter-style",
                        "a block-form counter-style rule",
                    ));
                }
                Ok(StrictAtRulePrelude::CounterStyle(name))
            },
            "page" => {
                if self.top_level_phase.is_none() {
                    return Err(top_level_only_at_rule_placement(
                        input.current_source_location(),
                        "page",
                    ));
                }
                let selector = parse_page_selector(input).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "page",
                        "later.rule.page",
                        "an empty prelude or one of :left, :right, or :first",
                    )
                })?;
                let following = self
                    .source
                    .get(input.position().byte_index()..)
                    .unwrap_or_default()
                    .trim_start();
                if following.is_empty() || following.starts_with(';') {
                    return Err(invalid_at_rule_body(
                        input,
                        "page",
                        "later.rule.page",
                        "a block-form page rule",
                    ));
                }
                Ok(StrictAtRulePrelude::Page(selector))
            },
            "custom-media" => Ok(StrictAtRulePrelude::CustomMedia(Box::new(parse_custom_media_prelude(self.source, input, &self.recovery)?))),
            "font-feature-values" => Ok(StrictAtRulePrelude::FontFeatureValues(font_feature_values::parse_families(self.source, input, &self.recovery)?)),
            "font-face" => {
                if !input.is_exhausted() {
                    return Err(with_at_rule_prelude_context(
                        invalid_syntax(
                            input.current_source_location(),
                            "unexpected token after font-face at-rule name",
                        ),
                        "font-face",
                        "baseline.rule.font-face",
                        "an empty @font-face prelude",
                    ));
                }
                Ok(StrictAtRulePrelude::FontFace)
            },
            "layer" => Ok(StrictAtRulePrelude::Layer(
                parse_layer_prelude(input).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "layer",
                        "baseline.rule.layer-block",
                        "a supported @layer prelude",
                    )
                })?,
            )),
            "keyframes" => {
                let name = parse_keyframes_name(input).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "keyframes",
                        "baseline.rule.keyframes",
                        "a supported keyframes name",
                    )
                })?;
                if !input.is_exhausted() {
                    return Err(with_at_rule_prelude_context(
                        invalid_syntax(
                            input.current_source_location(),
                            "unexpected token after keyframes name",
                        ),
                        "keyframes",
                        "baseline.rule.keyframes",
                        "the end of the @keyframes prelude",
                    ));
                }
                Ok(StrictAtRulePrelude::Keyframes(name))
            },
            "media" => {
                let query = parse_media_query_list_inner(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                )?;
                if !input.is_exhausted() {
                    return Err(crate::error::with_media_query_context(
                        invalid_syntax(
                            input.current_source_location(),
                            "unexpected token after media query list",
                        ),
                        None,
                    ));
                }
                Ok(StrictAtRulePrelude::Media(query))
            },
            "supports" => {
                let condition = parse_supports_condition(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                ).map_err(with_supports_prelude_context)?;
                Ok(StrictAtRulePrelude::Supports(condition))
            },
            "container" => {
                let prelude = parse_container_prelude(self.source, input, &self.recovery)
                    .map_err(with_container_prelude_context)?;
                if !input.is_exhausted() {
                    return Err(with_at_rule_prelude_context(
                        invalid_syntax(
                            input.current_source_location(),
                            "unexpected token after container condition",
                        ),
                        "container",
                        "baseline.rule.container",
                        "the end of the @container prelude",
                    ));
                }
                Ok(StrictAtRulePrelude::Container(prelude))
            },
            "scope" => Ok(StrictAtRulePrelude::Scope(
                parse_scope_prelude(self.source, input, &mut self.diagnostics, &self.recovery).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "scope",
                        "baseline.rule.scope",
                        "a supported @scope prelude",
                    )
                })?,
            )),
            _ => Err(input.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name))),
        }
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> std::result::Result<Self::AtRule, ()> {
        match prelude {
            StrictAtRulePrelude::CustomMedia(prelude) => {
                let rule = finish_custom_media(
                    self.source,
                    *prelude,
                    start,
                    &self.recovery,
                    &mut self.diagnostics,
                );
                self.mark_successful_body_rule();
                Ok(vec![CssRule::CustomMedia(rule)])
            }
            StrictAtRulePrelude::Encoding(label) => {
                self.encoding = Some(CssEncodingDeclaration::new(
                    label,
                    crate::source::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                ));
                Ok(Vec::new())
            }
            StrictAtRulePrelude::Import(prelude) => {
                self.diagnostics.extend(prelude.diagnostics);
                let mut token_input = ParserInput::new(self.source);
                let mut token_parser = Parser::new(&mut token_input);
                token_parser.reset(start);
                let at_keyword = crate::CssComponentValue::collect_from_parser(
                    &mut token_parser,
                    self.recovery.source_snapshot(),
                )
                .expect("parsed import at-keyword");
                self.recovery
                    .retain_component_closures(prelude.implicit_media_closures);
                let rule = CssRule::Import(CssImportRule::new(
                    prelude.target,
                    prelude.layer,
                    prelude.supports,
                    prelude.media,
                    crate::imports::ImportSyntax {
                        at_keyword,
                        prelude: prelude.syntax,
                    },
                    crate::source::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                ));
                self.mark_successful_import();
                Ok(vec![rule])
            }
            StrictAtRulePrelude::Namespace(prelude) => {
                let rule = CssNamespaceRule::new(
                    prelude.prefix,
                    prelude.name,
                    crate::source::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                );
                if !self.mark_successful_namespace(rule.prefix().cloned(), rule.name().clone()) {
                    return Err(());
                }
                Ok(vec![CssRule::Namespace(rule)])
            }
            StrictAtRulePrelude::CounterStyle(_) => Err(()),
            StrictAtRulePrelude::Page(_) => Err(()),
            StrictAtRulePrelude::Layer(names) => {
                let names = CssLayerNameList::try_new(names).ok_or(())?;
                self.mark_successful_layer_statement();
                Ok(vec![CssRule::LayerStatement(CssLayerStatementRule::new(
                    names,
                    crate::source::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                ))])
            }
            StrictAtRulePrelude::FontFeatureValues(_) => Err(()),
            StrictAtRulePrelude::FontFace => Err(()),
            StrictAtRulePrelude::Keyframes(_) => Err(()),
            StrictAtRulePrelude::Media(_) => Err(()),
            StrictAtRulePrelude::Supports(_) => Err(()),
            StrictAtRulePrelude::Container(_) => Err(()),
            StrictAtRulePrelude::Scope(_) => Err(()),
        }
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::AtRule, ParseError<'i, Self::Error>> {
        let mut depth = self
            .recovery
            .enter_rule_block(self.source, input, prelude.production())?;
        let result = match prelude {
            StrictAtRulePrelude::CustomMedia(_) => Err(invalid_at_rule_block(
                input,
                "custom-media",
                "ext.rule.custom-media",
                "a statement-form custom-media rule",
            )),
            StrictAtRulePrelude::Encoding(_) => Err(invalid_encoding_declaration(
                input.current_source_location(),
            )),
            StrictAtRulePrelude::Import(_) => Err(invalid_at_rule_block(
                input,
                "import",
                "baseline.rule.import",
                "a semicolon-terminated @import rule",
            )),
            StrictAtRulePrelude::Namespace(_) => Err(invalid_at_rule_block(
                input,
                "namespace",
                "later.rule.namespace",
                "a semicolon-terminated @namespace rule",
            )),
            StrictAtRulePrelude::CounterStyle(name) => {
                let rule = parse_counter_style_rule(
                    self.source,
                    name,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::CounterStyle(rule)])
            }
            StrictAtRulePrelude::Page(selector) => {
                let rule = parse_page_rule(
                    self.source,
                    selector,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::Page(rule)])
            }
            StrictAtRulePrelude::Layer(names) => {
                if names.len() > 1 {
                    return Err(invalid_at_rule_block(
                        input,
                        "layer",
                        "baseline.rule.layer-block",
                        "at most one layer name before a block",
                    ));
                }
                let name = names.into_iter().next();
                let recovered =
                    parse_nested_group_rules(self.source, input, self.recovery.clone())?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::LayerBlock(CssLayerBlockRule::new(
                    name,
                    rules,
                    crate::source::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                ))])
            }
            StrictAtRulePrelude::FontFeatureValues(families) => {
                let rule = font_feature_values::parse_rule(
                    self.source,
                    families,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::FontFeatureValues(rule)])
            }
            StrictAtRulePrelude::FontFace => {
                let rule = parse_font_face_rule(
                    self.source,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::FontFace(rule)])
            }
            StrictAtRulePrelude::Keyframes(name) => {
                let rule = parse_keyframes_rule(
                    self.source,
                    name,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::Keyframes(rule)])
            }
            StrictAtRulePrelude::Media(query) => {
                let recovered =
                    parse_nested_group_rules(self.source, input, self.recovery.clone())?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::Media(CssMediaRule::new(
                    query,
                    rules,
                    crate::source::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                ))])
            }
            StrictAtRulePrelude::Supports(condition) => {
                let recovered =
                    parse_nested_group_rules(self.source, input, self.recovery.clone())?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::Supports(CssSupportsRule::new(
                    condition,
                    rules,
                    crate::source::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                ))])
            }
            StrictAtRulePrelude::Container(prelude) => {
                let recovered =
                    parse_nested_group_rules(self.source, input, self.recovery.clone())?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::Container(CssContainerRule::new(
                    prelude.name,
                    prelude.condition,
                    rules,
                    crate::source::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                ))])
            }
            StrictAtRulePrelude::Scope(prelude) => {
                let recovered =
                    parse_scoped_rule_list(self.source, input, self.recovery.clone(), false)?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::Scope(CssScopeRule::new(
                    prelude.root,
                    prelude.limit,
                    rules,
                    crate::source::CssSourcePosition::from_cssparser(
                        start.position(),
                        start.source_location(),
                    ),
                ))])
            }
        };
        if result.is_ok() {
            depth.retain();
        }
        result
    }
}

impl<'i> QualifiedRuleParser<'i> for StrictRuleParser<'i> {
    type Prelude = Vec<CssSelector>;
    type QualifiedRule = Vec<CssRule>;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        self.encoding_allowed = false;
        let mut recovery =
            SelectorRecovery::new(self.source, &mut self.diagnostics, self.recovery.clone());
        parse_rule_selector_list(input, &mut recovery)
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut depth =
            self.recovery
                .enter_rule_block(self.source, input, "baseline.rule.style")?;
        let recovered = parse_style_rule_block(
            self.source,
            CssStyleSelectorList::absolute(selectors),
            crate::source::CssSourcePosition::from_cssparser(
                start.position(),
                start.source_location(),
            ),
            input,
            self.recovery.clone(),
        )?;
        self.diagnostics.extend(recovered.diagnostics);
        let rules = recovered.syntax;
        self.mark_successful_body_rule();
        depth.retain();
        Ok(rules)
    }
}

impl<'i> DeclarationParser<'i> for StrictRuleParser<'i> {
    type Declaration = Vec<CssRule>;
    type Error = Error;
}

impl<'i> RuleBodyItemParser<'i, Vec<CssRule>, Error> for StrictRuleParser<'i> {
    fn parse_declarations(&self) -> bool {
        false
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

fn parse_import_prelude<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    _diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<CssImportPrelude, ParseError<'i, Error>> {
    input.skip_whitespace();
    let target_start = input.state();
    let target = parse_import_target(input)?;
    let target_end = input.state();
    input.reset(&target_start);
    let target_component =
        crate::CssComponentValue::collect_from_parser(input, recovery.source_snapshot()).map_err(
            |error| crate::error::invalid_component_value(input.current_source_location(), error),
        )?;
    input.reset(&target_end);
    queries::check_media_import_components(source, input, recovery)?;
    // Include the accepted target in closure ownership: URL tokenization can
    // succeed at EOF without consuming a closing parenthesis.
    input.reset(&target_start);
    let prelude_closures =
        recovery.check_specialized_components(source, input, "baseline.media.query-list")?;
    input.reset(&target_end);
    let selected = select_import_clauses(source, input, recovery)?;
    let probe = recovery.detached_probe();
    let mut diagnostics = selected.diagnostics;
    let mut implicit = selected.implicit_closures;
    let clauses_end = input.position().byte_index();
    implicit.extend(
        prelude_closures
            .into_iter()
            .filter(|opening| *opening < clauses_end),
    );
    let media = if input.is_exhausted() {
        None
    } else {
        let parsed =
            queries::parse_media_query_list_with_closures(source, input, &mut diagnostics, &probe)?;
        implicit.extend(parsed.implicit_closures);
        Some(parsed.queries)
    };
    while input.next_including_whitespace_and_comments().is_ok() {}
    let end = input.position().byte_index();
    let semicolon = if source.as_bytes().get(end) == Some(&b';') {
        crate::CssValueOrigin::Parsed(
            CssParsedOrigin::from_range(recovery.source_snapshot(), end..end + 1)
                .expect("parsed import terminator"),
        )
    } else {
        crate::CssValueOrigin::Programmatic
    };
    Ok(CssImportPrelude {
        target,
        layer: selected.layer,
        supports: selected.supports,
        media,
        implicit_media_closures: implicit,
        diagnostics,
        syntax: crate::imports::ImportPreludeSyntax {
            target: target_component,
            layer: selected.layer_component,
            supports: selected.supports_component,
            semicolon,
        },
    })
}
struct ImportSelection {
    layer: Option<CssImportLayer>,
    supports: Option<CssImportSupports>,
    layer_component: Option<crate::CssComponentValue>,
    supports_component: Option<crate::CssComponentValue>,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    implicit_closures: Vec<usize>,
}
fn import_clause_component<'i>(
    input: &mut Parser<'i, '_>,
    start: &ParserState,
    recovery: &RecoveryState,
) -> Result<crate::CssComponentValue, ParseError<'i, Error>> {
    let end = input.state();
    input.reset(start);
    let result = crate::CssComponentValue::collect_from_parser(input, recovery.source_snapshot())
        .map_err(|error| {
            crate::error::invalid_component_value(input.current_source_location(), error)
        });
    input.reset(&end);
    result
}
fn select_import_clauses<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<ImportSelection, ParseError<'i, Error>> {
    let start = input.state();
    let mut fallback = None;
    for (layer_present, supports_present) in
        [(true, true), (true, false), (false, true), (false, false)]
    {
        input.reset(&start);
        let probe = recovery.detached_probe();
        let mut diagnostics = Vec::new();
        let candidate = (|| {
            let (layer, layer_component) = if layer_present {
                input.skip_whitespace();
                let at = input.state();
                let layer = parse_import_layer(input)?.ok_or_else(|| {
                    invalid_syntax(
                        input.current_source_location(),
                        "expected import layer clause",
                    )
                })?;
                (
                    Some(layer),
                    Some(import_clause_component(input, &at, &probe)?),
                )
            } else {
                (None, None)
            };
            let (supports, supports_component) = if supports_present {
                input.skip_whitespace();
                let at = input.state();
                let supports = parse_import_supports(source, input, &mut diagnostics, &probe)?
                    .ok_or_else(|| {
                        invalid_syntax(
                            input.current_source_location(),
                            "expected import supports clause",
                        )
                    })?;
                (
                    Some(supports),
                    Some(import_clause_component(input, &at, &probe)?),
                )
            } else {
                (None, None)
            };
            Ok(ImportSelection {
                layer,
                supports,
                layer_component,
                supports_component,
                diagnostics,
                implicit_closures: probe.pending_component_closures(),
            })
        })();
        let candidate = match candidate {
            Ok(value) => value,
            Err(error) if queries::media_terminal_error(&error) => return Err(error),
            Err(_) => continue,
        };
        let media_start = input.state();
        let clean = if candidate.diagnostics.is_empty() {
            probe_import_media(source, input, &probe)
        } else {
            Ok(false)
        }?;
        input.reset(&media_start);
        if clean {
            return Ok(candidate);
        }
        if fallback.is_none() {
            fallback = Some((candidate, media_start));
        }
    }
    let (selected, media_start) =
        fallback.expect("absent optional clauses always form a candidate");
    input.reset(&media_start);
    Ok(selected)
}
fn probe_import_media<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<bool, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(true);
    }
    loop {
        let result = input.parse_until_before(Delimiter::Comma, |member| {
            queries::parse_media_query(
                source,
                member,
                &crate::numeric::NumericInputContext::parsed(recovery.source_snapshot()),
            )?;
            member.expect_exhausted()?;
            Ok(())
        });
        match result {
            Ok(()) => {}
            Err(error) if queries::media_terminal_error(&error) => return Err(error),
            Err(_) => return Ok(false),
        }
        if input.is_exhausted() {
            return Ok(true);
        }
        input.expect_comma().map_err(basic)?;
        if input.is_exhausted() {
            return Ok(false);
        }
    }
}
pub(crate) fn import_boundaries_match(
    serialized: &crate::CssSerializedValue,
    expected: [Option<usize>; 2],
    original_origin: &crate::CssValueOrigin,
) -> Result<bool, crate::CssImportSerializationError> {
    let source = serialized.as_css();
    let recovery = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
    let mut buffer = ParserInput::new(source);
    let mut input = Parser::new(&mut buffer);
    let result = (|| {
        let selected = select_import_clauses(source, &mut input, &recovery)?;
        let actual = [
            selected.layer_component.as_ref(),
            selected.supports_component.as_ref(),
        ]
        .map(|component| {
            component
                .and_then(crate::CssComponentValue::parsed_origin)
                .map(|origin| origin.span().end().byte_offset().value())
        });
        Ok(actual == expected && probe_import_media(source, &mut input, &recovery)?)
    })();
    result.map_err(|error: ParseError<'_, Error>| {
        let error = from_parse_error(source, error);
        let offset = match error.kind() {
            crate::ErrorKind::InvalidComponentValue(component) => {
                crate::media::parsed_position(component.origin())
                    .map_or(error.position().byte_offset().value(), |position| {
                        position.byte_offset().value()
                    })
            }
            _ => error.position().byte_offset().value(),
        };
        let origin = serialized
            .value_origin_at(offset)
            .unwrap_or(original_origin)
            .clone();
        let kind = match error.kind() {
            crate::ErrorKind::InvalidComponentValue(component) => Some(component.kind()),
            crate::ErrorKind::NestingLimit(_) => {
                Some(crate::CssComponentValueErrorKind::NestingLimit)
            }
            _ => None,
        };
        match kind {
            Some(kind) => crate::CssImportSerializationError::Component(
                crate::CssComponentValueError::new(kind, origin),
            ),
            None => crate::CssImportSerializationError::InterpretationChanged { origin },
        }
    })
}
fn parse_import_supports<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> std::result::Result<Option<CssImportSupports>, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_function_matching("supports"))
        .is_err()
    {
        return Ok(None);
    }

    let condition = input.parse_nested_block(|nested| {
        if let Some(declaration) = supports::grammar_probe(nested.try_parse(|nested| {
            let declaration = parse_supports_declaration(nested, recovery.source_snapshot())?;
            nested.expect_exhausted().map_err(basic)?;
            Ok::<_, ParseError<'i, Error>>(declaration)
        }))? {
            let lexical = declaration.lexical().clone();
            return Ok(CssSupportsCondition::new(
                CssSupportsConditionKind::Declaration(Box::new(declaration)),
                lexical,
                true,
            ));
        }

        parse_supports_condition(source, nested, diagnostics, recovery)
    })?;

    Ok(Some(CssImportSupports::new(condition)))
}

fn parse_container_prelude<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    recovery: &RecoveryState,
) -> std::result::Result<CssContainerPrelude, ParseError<'i, Error>> {
    let implicit =
        recovery.check_specialized_components(source, input, "baseline.rule.container")?;
    let state = input.state();
    let name = if let Ok(name) = input.try_parse(Parser::expect_ident_cloned) {
        if let Some(name) = CssContainerName::try_new(name.to_string()) {
            Some(name)
        } else {
            input.reset(&state);
            None
        }
    } else {
        None
    };
    let condition = parse_container_condition(input)?;

    if input.is_exhausted() {
        recovery.retain_component_closures(implicit);
    }

    Ok(CssContainerPrelude { name, condition })
}

fn with_container_prelude_context<'i>(error: ParseError<'i, Error>) -> ParseError<'i, Error> {
    if is_nesting_limit_error(&error) {
        error
    } else {
        with_at_rule_prelude_context(
            error,
            "container",
            "baseline.rule.container",
            "a supported @container prelude",
        )
    }
}

fn parse_nested_group_rules<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
) -> std::result::Result<Recovered<Vec<CssRule>>, ParseError<'i, Error>> {
    let mut rule_parser = StrictRuleParser::nested(source, recovery);
    let mut rules = Vec::new();
    let mut diagnostics = Vec::new();
    let mut previous_end = input.position().byte_index();
    {
        let mut items = RuleBodyParser::new(input, &mut rule_parser);
        loop {
            let progress = RecoveryProgress::record(items.input);
            let Some(item) = items.next() else {
                break;
            };
            let failed_block_error = item.as_ref().err().and_then(|(_, failed_unit)| {
                consume_failed_rule_block(
                    source,
                    items.input,
                    true,
                    &items.parser.recovery,
                    structural_recovery_production(failed_unit),
                )
                .1
            });
            let retained = item.is_ok();
            let progress_outcome = progress.finish(items.input, retained);
            let unit_end = items.input.position().byte_index();
            diagnostics.append(&mut items.parser.diagnostics);
            match item {
                Ok(parsed_rules) => rules.extend(parsed_rules),
                Err((error, failed_unit)) => {
                    let error = failed_block_error.unwrap_or(error);
                    let action = structural_recovery_action(failed_unit);
                    if let Some(diagnostic) = structural_rule_diagnostic(
                        source,
                        error,
                        failed_unit,
                        previous_end,
                        unit_end,
                        action,
                    ) {
                        diagnostics.push(diagnostic);
                    }
                }
            }
            previous_end = unit_end;
            if progress_outcome == RecoveryLoopOutcome::Terminated {
                break;
            }
        }
    }
    Ok(Recovered {
        syntax: rules,
        diagnostics,
    })
}

pub(super) fn parse_scoped_rule_list<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
    has_style_ancestor: bool,
) -> std::result::Result<Recovered<CssScopedRuleList>, ParseError<'i, Error>> {
    if has_style_ancestor {
        recovery.record_style_context(input.position().byte_index());
    }
    let mut rule_parser = ScopedRuleParser {
        source,
        diagnostics: Vec::new(),
        recovery,
        has_style_ancestor,
    };
    let mut rules = Vec::new();
    let mut diagnostics = Vec::new();
    let mut previous_end = input.position().byte_index();
    {
        let mut items = RuleBodyParser::new(input, &mut rule_parser);
        loop {
            let progress = RecoveryProgress::record(items.input);
            let Some(item) = items.next() else {
                break;
            };
            let failed_block_error = item.as_ref().err().and_then(|(_, failed_unit)| {
                consume_failed_rule_block(
                    source,
                    items.input,
                    true,
                    &items.parser.recovery,
                    structural_recovery_production(failed_unit),
                )
                .1
            });
            let retained = item.is_ok();
            let progress_outcome = progress.finish(items.input, retained);
            let unit_end = items.input.position().byte_index();
            diagnostics.append(&mut items.parser.diagnostics);
            match item {
                Ok(parsed_rules) => rules.extend(parsed_rules),
                Err((error, failed_unit)) => {
                    let error = failed_block_error.unwrap_or(error);
                    let action = structural_recovery_action(failed_unit);
                    if let Some(diagnostic) = structural_rule_diagnostic(
                        source,
                        error,
                        failed_unit,
                        previous_end,
                        unit_end,
                        action,
                    ) {
                        diagnostics.push(diagnostic);
                    }
                }
            }
            previous_end = unit_end;
            if progress_outcome == RecoveryLoopOutcome::Terminated {
                break;
            }
        }
    }
    Ok(Recovered {
        syntax: CssScopedRuleList::from_rules(rules),
        diagnostics,
    })
}

fn parse_import_target<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssImportTarget, ParseError<'i, Error>> {
    let location = input.current_source_location();

    if let Ok(value) = input.try_parse(Parser::expect_string_cloned) {
        return Ok(CssImportTarget::String(CssImportString::new(
            value.as_ref(),
        )));
    }

    if let Ok(value) = input.try_parse(Parser::expect_url) {
        return Ok(CssImportTarget::Url(CssImportUrl::new(value.as_ref())));
    }

    Err(invalid_syntax(
        location,
        "expected string or URL import target",
    ))
}

fn parse_namespace_prelude<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssNamespacePrelude, ParseError<'i, Error>> {
    let prefix = input
        .try_parse(Parser::expect_ident_cloned)
        .ok()
        .map(|prefix| CssNamespacePrefix::new(prefix.to_string()));
    let name = input.expect_url_or_string().map_err(basic)?;
    input.expect_exhausted().map_err(basic)?;

    if source
        .as_bytes()
        .get(input.position().byte_index())
        .is_none()
    {
        return Err(invalid_syntax(
            input.current_source_location(),
            "namespace rules require a terminating semicolon",
        ));
    }

    Ok(CssNamespacePrelude {
        prefix,
        name: CssNamespaceName::new(name.to_string()),
    })
}

fn parse_import_layer<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Option<CssImportLayer>, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("layer"))
        .is_ok()
    {
        return Ok(Some(CssImportLayer::Anonymous));
    }

    if input
        .try_parse(|input| input.expect_function_matching("layer"))
        .is_ok()
    {
        let layer_name = input.parse_nested_block(parse_import_layer_name)?;
        return Ok(Some(CssImportLayer::Named(layer_name)));
    }

    Ok(None)
}

fn parse_import_layer_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLayerName, ParseError<'i, Error>> {
    let name = parse_layer_name(input)?;
    if !input.is_exhausted() {
        return Err(invalid_syntax(
            input.current_source_location(),
            "unexpected token in import layer name",
        ));
    }
    Ok(name)
}

fn parse_layer_prelude<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Vec<CssLayerName>, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    loop {
        names.push(parse_layer_name(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    if !input.is_exhausted() {
        return Err(invalid_syntax(
            input.current_source_location(),
            "unexpected token after layer name list",
        ));
    }
    Ok(names)
}

fn parse_layer_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLayerName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let mut components = vec![input.expect_ident_cloned().map_err(basic)?.to_string()];

    loop {
        let boundary = input.state();
        if !matches!(input.next_including_whitespace(), Ok(Token::Delim('.'))) {
            input.reset(&boundary);
            break;
        }

        let component_location = input.current_source_location();
        match input.next_including_whitespace().map_err(basic)? {
            Token::Ident(component) => components.push(component.to_string()),
            token => {
                return Err(component_location.new_unexpected_token_error(token.clone()));
            }
        }
    }

    CssLayerName::try_new(components).ok_or_else(|| invalid_syntax(location, "invalid layer name"))
}

fn parse_scope_prelude<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    state: &RecoveryState,
) -> std::result::Result<CssScopePrelude, ParseError<'i, Error>> {
    let root = if input.try_parse(Parser::expect_parenthesis_block).is_ok() {
        Some(input.parse_nested_block(|input| {
            let mut recovery = SelectorRecovery::new(source, diagnostics, state.clone());
            parse_scope_boundary_selector_list(input, &mut recovery)
        })?)
    } else {
        None
    };

    let limit = if input
        .try_parse(|input| input.expect_ident_matching("to"))
        .is_ok()
    {
        input.expect_parenthesis_block().map_err(basic)?;
        Some(input.parse_nested_block(|input| {
            let mut recovery = SelectorRecovery::new(source, diagnostics, state.clone());
            parse_scope_boundary_selector_list(input, &mut recovery)
        })?)
    } else {
        None
    };

    if !input.is_exhausted() {
        return Err(invalid_syntax(
            input.current_source_location(),
            "unexpected token after scope prelude",
        ));
    }

    Ok(CssScopePrelude { root, limit })
}

struct ScopedRuleParser<'s> {
    has_style_ancestor: bool,
    source: &'s str,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
}

enum ScopedAtRulePrelude {
    CustomMedia(Box<CustomMediaPrelude>),
    FontFeatureValues(Vec<CssFontFaceFamily>),
    Media(CssMediaQueryList),
    Supports(CssSupportsCondition),
    Container(CssContainerPrelude),
    Layer(Vec<CssLayerName>),
    Scope(CssScopePrelude),
}

impl ScopedAtRulePrelude {
    fn production(&self) -> &'static str {
        match self {
            Self::CustomMedia(_) => "ext.rule.custom-media",
            Self::FontFeatureValues(_) => "later.rule.font-feature-values",
            Self::Media(_) => "baseline.rule.media",
            Self::Supports(_) => "baseline.rule.supports",
            Self::Container(_) => "baseline.rule.container",
            Self::Layer(_) => "baseline.rule.layer-block",
            Self::Scope(_) => "baseline.rule.scope",
        }
    }
}

impl<'i> AtRuleParser<'i> for ScopedRuleParser<'i> {
    type Prelude = ScopedAtRulePrelude;
    type AtRule = Vec<CssScopedRule>;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        match_ignore_ascii_case! { &name,
            "media" => {
                let query = parse_media_query_list_inner(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                )?;
                if !input.is_exhausted() {
                    return Err(with_media_query_context(
                        invalid_syntax(
                            input.current_source_location(),
                            "unexpected token after media query list",
                        ),
                        None,
                    ));
                }
                Ok(ScopedAtRulePrelude::Media(query))
            },
            "supports" => {
                let condition = parse_supports_condition(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                ).map_err(with_supports_prelude_context)?;
                Ok(ScopedAtRulePrelude::Supports(condition))
            },
            "container" => {
                let prelude = parse_container_prelude(self.source, input, &self.recovery)
                    .map_err(with_container_prelude_context)?;
                if !input.is_exhausted() {
                    return Err(with_at_rule_prelude_context(
                        invalid_syntax(
                            input.current_source_location(),
                            "unexpected token after container condition",
                        ),
                        "container",
                        "baseline.rule.container",
                        "the end of the @container prelude",
                    ));
                }
                Ok(ScopedAtRulePrelude::Container(prelude))
            },
            "layer" => Ok(ScopedAtRulePrelude::Layer(
                parse_layer_prelude(input).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "layer",
                        "baseline.rule.layer-block",
                        "a supported @layer prelude",
                    )
                })?,
            )),
            "scope" => Ok(ScopedAtRulePrelude::Scope(
                parse_scope_prelude(self.source, input, &mut self.diagnostics, &self.recovery).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "scope",
                        "baseline.rule.scope",
                        "a supported @scope prelude",
                    )
                })?,
            )),
            "import" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "import",
                "the stylesheet top level",
            )),
            "custom-media" => {
                if self.has_style_ancestor { return Err(invalid_at_rule_placement(input.current_source_location(), "custom-media", "a rule list without a style-rule ancestor")); }
                Ok(ScopedAtRulePrelude::CustomMedia(Box::new(parse_custom_media_prelude(self.source, input, &self.recovery)?)))
            },
            "font-feature-values" => {
                if self.has_style_ancestor {
                    return Err(invalid_at_rule_placement(input.current_source_location(), "font-feature-values", "a rule list without a style-rule ancestor"));
                }
                Ok(ScopedAtRulePrelude::FontFeatureValues(font_feature_values::parse_families(self.source, input, &self.recovery)?))
            },
            "font-face" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "font-face",
                "a stylesheet or conditional group rule list",
            )),
            "keyframes" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "keyframes",
                "a stylesheet or conditional group rule list",
            )),
            "namespace" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "namespace",
                "the stylesheet top level",
            )),
            "counter-style" => Err(top_level_only_at_rule_placement(
                input.current_source_location(),
                "counter-style",
            )),
            "page" => Err(top_level_only_at_rule_placement(
                input.current_source_location(),
                "page",
            )),
            _ => Err(input.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name))),
        }
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> std::result::Result<Self::AtRule, ()> {
        match prelude {
            ScopedAtRulePrelude::CustomMedia(prelude) => {
                let rule = finish_custom_media(
                    self.source,
                    *prelude,
                    start,
                    &self.recovery,
                    &mut self.diagnostics,
                );
                Ok(vec![CssScopedRule::CustomMedia(rule)])
            }
            ScopedAtRulePrelude::Layer(names) => {
                let names = CssLayerNameList::try_new(names).ok_or(())?;
                Ok(vec![CssScopedRule::LayerStatement(
                    CssScopedLayerStatementRule::new(
                        names,
                        crate::source::CssSourcePosition::from_cssparser(
                            start.position(),
                            start.source_location(),
                        ),
                    ),
                )])
            }
            ScopedAtRulePrelude::FontFeatureValues(_)
            | ScopedAtRulePrelude::Media(_)
            | ScopedAtRulePrelude::Supports(_)
            | ScopedAtRulePrelude::Container(_)
            | ScopedAtRulePrelude::Scope(_) => Err(()),
        }
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::AtRule, ParseError<'i, Self::Error>> {
        let mut depth = self
            .recovery
            .enter_rule_block(self.source, input, prelude.production())?;
        let position = crate::source::CssSourcePosition::from_cssparser(
            start.position(),
            start.source_location(),
        );
        let result = match prelude {
            ScopedAtRulePrelude::CustomMedia(_) => Err(invalid_at_rule_block(
                input,
                "custom-media",
                "ext.rule.custom-media",
                "a statement-form custom-media rule",
            )),
            ScopedAtRulePrelude::FontFeatureValues(families) => {
                let rule = font_feature_values::parse_rule(
                    self.source,
                    families,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                Ok(vec![CssScopedRule::FontFeatureValues(rule)])
            }
            ScopedAtRulePrelude::Media(query) => {
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    self.has_style_ancestor,
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                Ok(vec![CssScopedRule::Media(CssScopedMediaRule::new(
                    query, rules, position,
                ))])
            }
            ScopedAtRulePrelude::Supports(condition) => {
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    self.has_style_ancestor,
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                Ok(vec![CssScopedRule::Supports(CssScopedSupportsRule::new(
                    condition, rules, position,
                ))])
            }
            ScopedAtRulePrelude::Container(prelude) => {
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    self.has_style_ancestor,
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                Ok(vec![CssScopedRule::Container(CssScopedContainerRule::new(
                    prelude.name,
                    prelude.condition,
                    rules,
                    position,
                ))])
            }
            ScopedAtRulePrelude::Layer(names) => {
                if names.len() > 1 {
                    return Err(invalid_at_rule_block(
                        input,
                        "layer",
                        "baseline.rule.layer-block",
                        "at most one layer name before a block",
                    ));
                }
                let name = names.into_iter().next();
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    self.has_style_ancestor,
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                Ok(vec![CssScopedRule::LayerBlock(
                    CssScopedLayerBlockRule::new(name, rules, position),
                )])
            }
            ScopedAtRulePrelude::Scope(prelude) => {
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    self.has_style_ancestor,
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                Ok(vec![CssScopedRule::Scope(CssScopeRule::new(
                    prelude.root,
                    prelude.limit,
                    rules,
                    position,
                ))])
            }
        };
        if result.is_ok() {
            depth.retain();
        }
        result
    }
}

impl<'i> QualifiedRuleParser<'i> for ScopedRuleParser<'i> {
    type Prelude = CssScopedStyleSelectorList;
    type QualifiedRule = Vec<CssScopedRule>;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let mut recovery =
            SelectorRecovery::new(self.source, &mut self.diagnostics, self.recovery.clone());
        parse_scoped_style_selector_list(input, &mut recovery)
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut depth =
            self.recovery
                .enter_rule_block(self.source, input, "baseline.rule.style")?;
        let recovered = parse_style_contents(self.source, input, self.recovery.clone())?;
        self.diagnostics.extend(recovered.diagnostics);
        depth.retain();
        Ok(vec![CssScopedRule::Style(CssScopedStyleRule::new(
            selectors,
            recovered.syntax.declarations,
            recovered.syntax.rules,
            crate::source::CssSourcePosition::from_cssparser(
                start.position(),
                start.source_location(),
            ),
        ))])
    }
}

impl<'i> DeclarationParser<'i> for ScopedRuleParser<'i> {
    type Declaration = Vec<CssScopedRule>;
    type Error = Error;
}

impl<'i> RuleBodyItemParser<'i, Vec<CssScopedRule>, Error> for ScopedRuleParser<'i> {
    fn parse_declarations(&self) -> bool {
        false
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

fn parse_style_attribute_declarations<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
) -> Recovered<CssDeclarationList> {
    let mut declarations = Vec::new();
    let mut diagnostics = Vec::new();
    let mut declaration_parser = StrictDeclarationParser::new(source, recovery.clone(), true);
    let mut items = RuleBodyParser::new(input, &mut declaration_parser);
    let mut previous_end = items.input.position().byte_index();
    loop {
        let progress = RecoveryProgress::record(items.input);
        if let Some(diagnostic) = discard_malformed_style_attribute_token(source, items.input) {
            previous_end = diagnostic.span().end().byte_offset().value();
            diagnostics.push(diagnostic);
            if progress.finish(items.input, false) == RecoveryLoopOutcome::Terminated {
                break;
            }
            continue;
        }
        let Some(item) = items.next() else {
            break;
        };
        let (failed_at_block, failed_block_error) = item
            .as_ref()
            .err()
            .map(|(_, _)| {
                consume_failed_rule_block(source, items.input, true, &recovery, "css.declaration")
            })
            .unwrap_or((false, None));
        let retained = item.is_ok();
        let progress_outcome = progress.finish(items.input, retained);
        let unit_end = items.input.position().byte_index();
        match item {
            Ok(declaration) => declarations.push(declaration),
            Err((error, failed_unit))
                if is_declaration_recovery_unit(failed_unit) && !failed_at_block =>
            {
                if let Some(diagnostic) = block_item_diagnostic(
                    source,
                    error,
                    failed_unit,
                    unit_end,
                    crate::CssRecoveryAction::DropDeclaration,
                ) {
                    diagnostics.push(diagnostic);
                }
            }
            Err((error, failed_unit)) => {
                let error = failed_block_error.unwrap_or(error);
                let unit_start = recovery_unit_start(source, previous_end, unit_end, failed_unit);
                if let Some(diagnostic) = block_item_diagnostic_from_start(
                    source,
                    error,
                    unit_start,
                    unit_end,
                    crate::CssRecoveryAction::DropDeclaration,
                ) {
                    diagnostics.push(diagnostic);
                }
            }
        }
        previous_end = unit_end;
        if progress_outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }
    Recovered {
        syntax: CssDeclarationList::new(declarations),
        diagnostics,
    }
}

pub(super) struct StrictDeclarationParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
    parse_non_declarations: bool,
}

impl<'s> StrictDeclarationParser<'s> {
    pub(super) fn new(
        source: &'s str,
        recovery: RecoveryState,
        parse_non_declarations: bool,
    ) -> Self {
        Self {
            source,
            recovery,
            parse_non_declarations,
        }
    }
}

impl<'i> AtRuleParser<'i> for StrictDeclarationParser<'i> {
    type Prelude = ();
    type AtRule = CssDeclaration;
    type Error = Error;
}

impl<'i> QualifiedRuleParser<'i> for StrictDeclarationParser<'i> {
    type Prelude = ();
    type QualifiedRule = CssDeclaration;
    type Error = Error;
}

impl<'i> RuleBodyItemParser<'i, CssDeclaration, Error> for StrictDeclarationParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        self.parse_non_declarations
    }
}

impl<'i> DeclarationParser<'i> for StrictDeclarationParser<'i> {
    type Declaration = CssDeclaration;
    type Error = Error;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> std::result::Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let implicit_closures =
            self.recovery
                .check_component_values(self.source, input, "css.declaration")?;
        let parsed = parse_declaration_core(
            DeclarationMode::Ordinary,
            name,
            input,
            declaration_start,
            self.recovery.source_snapshot(),
        )?;
        self.recovery.retain_component_closures(implicit_closures);
        Ok(parsed.into_declaration())
    }
}

#[derive(Clone, Copy)]
pub(super) enum DeclarationMode {
    Ordinary,
    Keyframe,
}

pub(super) struct ParsedDeclaration {
    pub(super) body: CssDeclarationBody,
    pub(super) importance: CssImportance,
    pub(super) position: crate::source::CssSourcePosition,
    components: CssComponentValues,
    name: CssParsedOrigin,
    value_origin: CssParsedOrigin,
}

impl ParsedDeclaration {
    fn into_declaration(self) -> CssDeclaration {
        CssDeclaration::new_parsed(
            self.body,
            self.importance,
            self.components,
            self.name,
            self.value_origin,
        )
    }
}

enum DeclarationBoundaryContext<'a> {
    OrdinaryKnown(crate::CssKnownProperty),
    OrdinaryCustom(CssCustomPropertyName),
    KeyframeKnown(crate::CssKnownProperty),
    KeyframeCustom(CssCustomPropertyName),
    Descriptor {
        at_rule: &'a str,
        descriptor: &'a str,
    },
}

pub(super) fn parse_declaration_core<'i, 't>(
    mode: DeclarationMode,
    name: CowRcStr<'i>,
    input: &mut Parser<'i, 't>,
    declaration_start: &ParserState,
    source_snapshot: &CssSourceSnapshot,
) -> std::result::Result<ParsedDeclaration, ParseError<'i, Error>> {
    // Revisit only the name token. The original snapshot is shared even when
    // this parser reads a same-length masked structural chunk.
    let value_start = input.state();
    input.reset(declaration_start);
    input.expect_ident()?;
    let name_origin = CssParsedOrigin::from_range(
        source_snapshot,
        declaration_start.position().byte_index()..input.position().byte_index(),
    )
    .expect("parser name boundaries belong to the original source");
    let position = name_origin.span().start();
    input.reset(&value_start);

    let (body, importance, components, value_origin) = if name.starts_with("--") {
        let Some(custom_name) = parse_custom_property_name(name.as_ref()) else {
            return Err(property_name_error(
                declaration_start.source_location(),
                name.as_ref(),
            ));
        };
        let context = match mode {
            DeclarationMode::Ordinary => {
                DeclarationBoundaryContext::OrdinaryCustom(custom_name.clone())
            }
            DeclarationMode::Keyframe => {
                DeclarationBoundaryContext::KeyframeCustom(custom_name.clone())
            }
        };
        let ((value, components, origin), importance) =
            parse_declaration_boundary(input, &context, |input| {
                collect_declaration_value(input, source_snapshot, |input| {
                    parse_custom_property_value(input)
                        .map_err(|error| with_property_context(error, name.as_ref()))
                })
            })?;
        (
            CssDeclarationBody::Custom(CssCustomDeclaration::new(custom_name, value)),
            importance,
            components,
            origin,
        )
    } else {
        let resolved_property = resolve_property_name(name.as_ref()).ok_or_else(|| {
            property_name_error(declaration_start.source_location(), name.as_ref())
        })?;
        let known_property = resolved_property.property();
        let context = match mode {
            DeclarationMode::Ordinary => DeclarationBoundaryContext::OrdinaryKnown(known_property),
            DeclarationMode::Keyframe => DeclarationBoundaryContext::KeyframeKnown(known_property),
        };
        let ((body, components, origin), importance) =
            parse_declaration_boundary(input, &context, |input| {
                collect_declaration_value(input, source_snapshot, |input| {
                    parse_known_declaration_body(
                        resolved_property,
                        input,
                        &crate::numeric::NumericInputContext::parsed(source_snapshot),
                    )
                })
            })?;
        (body, importance, components, origin)
    };
    Ok(ParsedDeclaration {
        body,
        importance,
        position,
        components,
        name: name_origin,
        value_origin,
    })
}

fn collect_declaration_value<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    source_snapshot: &CssSourceSnapshot,
    parse_value: impl FnOnce(&mut Parser<'i, 't>) -> std::result::Result<T, ParseError<'i, Error>>,
) -> std::result::Result<(T, CssComponentValues, CssParsedOrigin), ParseError<'i, Error>> {
    let start = input.state();
    let value = parse_value(input)?;
    input.reset(&start);
    let components =
        CssComponentValues::collect_from_parser(input, source_snapshot).map_err(|error| {
            crate::error::invalid_component_value(input.current_source_location(), error)
        })?;
    let origin = CssParsedOrigin::from_range(
        source_snapshot,
        start.position().byte_index()..input.position().byte_index(),
    )
    .expect("parser value boundaries belong to the original source");
    Ok((value, components, origin))
}

pub(crate) fn parse_property_value_body(
    property: CssPropertyNameRef<'_>,
    source: &str,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssDeclarationBody, Error> {
    let grammar = match property {
        CssPropertyNameRef::Known(property) => PropertyValueGrammar::Known(property.grammar()),
        CssPropertyNameRef::Custom(name) => PropertyValueGrammar::Custom(name),
    };
    parse_property_value_body_selected(grammar, source, numeric)
}

pub(crate) fn parse_property_value_body_for_grammar(
    grammar: CssPropertyGrammar,
    source: &str,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssDeclarationBody, Error> {
    parse_property_value_body_selected(PropertyValueGrammar::Known(grammar), source, numeric)
}

enum PropertyValueGrammar<'a> {
    Known(CssPropertyGrammar),
    Custom(&'a CssCustomPropertyName),
}

fn parse_property_value_body_selected(
    grammar: PropertyValueGrammar<'_>,
    source: &str,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssDeclarationBody, Error> {
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    parse_property_value_from_parser(grammar, source, &mut parser, numeric)
        .map_err(|error| from_parse_error(source, error))
}

fn parse_property_value_from_parser<'i>(
    grammar: PropertyValueGrammar<'_>,
    source: &'i str,
    parser: &mut Parser<'i, '_>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssDeclarationBody, ParseError<'i, Error>> {
    // Reject root boundaries before the symbolic-substitution shortcut. Nested
    // punctuation remains data, including custom-property curly blocks.
    let start = parser.state();
    loop {
        let offset = parser.position().byte_index();
        let location = parser.current_source_location();
        let Ok(token) = parser.next_including_whitespace_and_comments().cloned() else {
            break;
        };
        if matches!(
            token,
            Token::Delim('!')
                | Token::Semicolon
                | Token::CloseCurlyBracket
                | Token::CloseParenthesis
                | Token::CloseSquareBracket
        ) {
            return Err(location
                .new_custom_error(crate::error::unexpected_token_at(source, offset, &token)));
        }
        fragments::finish_nested_component(parser, &token)?;
    }
    parser.reset(&start);
    let body = match grammar {
        PropertyValueGrammar::Known(grammar) => {
            parse_known_declaration_body(grammar.resolved(), parser, numeric)
        }
        PropertyValueGrammar::Custom(name) => parse_custom_property_value(parser).map(|value| {
            CssDeclarationBody::Custom(CssCustomDeclaration::new(name.clone(), value))
        }),
    }?;
    parser.expect_exhausted()?;
    Ok(body)
}

fn parse_known_declaration_body<'i, 't>(
    resolved_property: CssResolvedPropertyName,
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssDeclarationBody, ParseError<'i, Error>> {
    let known_property = resolved_property.property();
    let context_name = known_property.canonical_name();
    let state = input.state();
    let (authored, has_substitution) = collect_authored_declaration_value(input)
        .map_err(|error| with_property_context(error, context_name))?;
    if has_substitution {
        return Ok(CssDeclarationBody::Known(
            CssKnownDeclaration::from_substitution_dependent(
                known_property,
                CssSubstitutionDependentValue::new(authored),
            )
            .with_grammar(CssPropertyGrammar::from_resolved(resolved_property)),
        ));
    }
    input.reset(&state);

    let state = input.state();
    if let Ok(ident) = input.expect_ident_cloned() {
        if let Some(keyword) = parse_global_keyword(&ident) {
            if !input.is_exhausted() {
                return Err(with_property_context(
                    invalid_syntax(
                        input.current_source_location(),
                        "CSS global keyword must be the entire declaration value",
                    ),
                    context_name,
                ));
            }
            return Ok(CssDeclarationBody::Known(
                CssKnownDeclaration::from_global(known_property, keyword)
                    .with_grammar(CssPropertyGrammar::from_resolved(resolved_property)),
            ));
        }
        input.reset(&state);
    } else {
        input.reset(&state);
    }

    let declaration = match resolved_property {
        CssResolvedPropertyName::Canonical(_) => {
            parse_known_property_value(known_property, authored, input, numeric)
        }
        CssResolvedPropertyName::LegacyShorthand(alias) => {
            parse_legacy_property_alias_value(alias, authored, input)
        }
    }
    .map_err(|error| with_property_context(error, context_name))?;
    input
        .expect_exhausted()
        .map_err(|error| with_property_context(error.into(), context_name))?;
    Ok(CssDeclarationBody::Known(declaration.with_grammar(
        CssPropertyGrammar::from_resolved(resolved_property),
    )))
}

fn parse_legacy_property_alias_value<'i, 't>(
    alias: CssLegacyPropertyAlias,
    authored: CssAuthoredDeclarationValue,
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssKnownDeclaration, ParseError<'i, Error>> {
    match alias {
        CssLegacyPropertyAlias::GlyphOrientationVertical => {
            let value = parse_glyph_orientation_vertical(input)?;
            Ok(CssKnownDeclaration::from_value(
                CssKnownDeclarationValue::TextOrientation(CssDeclaredValue::Value(
                    CssTextOrientationPropertyValue::new(authored, value),
                )),
            ))
        }
    }
}

fn parse_declaration_boundary<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    context: &DeclarationBoundaryContext<'_>,
    parse_value: impl for<'tt> FnOnce(
        &mut Parser<'i, 'tt>,
    ) -> std::result::Result<T, ParseError<'i, Error>>,
) -> std::result::Result<(T, CssImportance), ParseError<'i, Error>> {
    let value = input.parse_until_before(Delimiter::Bang, parse_value)?;
    if input.is_exhausted() {
        return Ok((value, CssImportance::Normal));
    }

    let bang_location = input.current_source_location();
    let annotation_valid = input.expect_delim('!').is_ok()
        && input.expect_ident_matching("important").is_ok()
        && input.is_exhausted();
    let ordinary = matches!(
        context,
        DeclarationBoundaryContext::OrdinaryKnown(_)
            | DeclarationBoundaryContext::OrdinaryCustom(_)
    );
    if annotation_valid && ordinary {
        Ok((value, CssImportance::Important))
    } else {
        Err(invalid_annotation_for_context(bang_location, context))
    }
}

fn invalid_annotation_for_context<'i>(
    location: cssparser::SourceLocation,
    context: &DeclarationBoundaryContext<'_>,
) -> ParseError<'i, Error> {
    match context {
        DeclarationBoundaryContext::OrdinaryKnown(property) => {
            invalid_known_declaration_annotation(location, *property, false)
        }
        DeclarationBoundaryContext::OrdinaryCustom(property) => {
            invalid_custom_declaration_annotation(location, property, false)
        }
        DeclarationBoundaryContext::KeyframeKnown(property) => {
            invalid_known_declaration_annotation(location, *property, true)
        }
        DeclarationBoundaryContext::KeyframeCustom(property) => {
            invalid_custom_declaration_annotation(location, property, true)
        }
        DeclarationBoundaryContext::Descriptor {
            at_rule,
            descriptor,
        } => invalid_descriptor_annotation(location, at_rule, descriptor),
    }
}

pub(super) fn parse_descriptor_boundary<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    at_rule: &str,
    descriptor: &str,
    parse_value: impl for<'tt> FnOnce(
        &mut Parser<'i, 'tt>,
    ) -> std::result::Result<T, ParseError<'i, Error>>,
) -> std::result::Result<T, ParseError<'i, Error>> {
    let context = DeclarationBoundaryContext::Descriptor {
        at_rule,
        descriptor,
    };
    parse_declaration_boundary(input, &context, parse_value).map(|(value, _)| value)
}

struct CustomMediaPrelude {
    name: crate::CssCustomMediaName,
    body: crate::CssCustomMediaBody,
    body_origin: crate::CssValueOrigin,
    semicolon: crate::CssValueOrigin,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    implicit: Vec<usize>,
}
fn parse_custom_media_prelude<'i>(
    source: &str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<CustomMediaPrelude, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let component =
        crate::CssComponentValue::collect_from_parser(input, recovery.source_snapshot())
            .map_err(|e| crate::error::invalid_component_value(location, e))?;
    let name = crate::CssCustomMediaName::try_from_component(component).map_err(|_| {
        with_at_rule_prelude_context(
            invalid_syntax(location, "expected extension name"),
            "custom-media",
            "ext.rule.custom-media",
            "an extension name followed by a custom-media body",
        )
    })?;
    let body_start = input.state();
    let boolean = input
        .try_parse(|p| {
            let name = p.expect_ident_cloned().map_err(basic)?;
            let boolean = crate::custom_media::boolean_keyword(&name)
                .ok_or_else(|| invalid_syntax(p.current_source_location(), "not a boolean body"))?;
            p.expect_exhausted().map_err(basic)?;
            Ok::<_, ParseError<'i, Error>>(boolean)
        })
        .ok();
    input.reset(&body_start);
    let mut diagnostics = Vec::new();
    let mut implicit = Vec::new();
    let (body, body_origin) = if let Some(boolean) = boolean {
        input.skip_whitespace();
        let token =
            crate::CssComponentValue::collect_from_parser(input, recovery.source_snapshot())
                .map_err(|e| {
                    crate::error::invalid_component_value(input.current_source_location(), e)
                })?;
        while input.next_including_whitespace_and_comments().is_ok() {}
        (
            if boolean {
                crate::CssCustomMediaBody::True
            } else {
                crate::CssCustomMediaBody::False
            },
            token.origin().clone(),
        )
    } else {
        let probe = recovery.detached_probe();
        let parsed =
            queries::parse_media_query_list_with_closures(source, input, &mut diagnostics, &probe)?;
        implicit.extend(parsed.implicit_closures);
        implicit.extend(probe.pending_component_closures());
        (
            crate::CssCustomMediaBody::Media(parsed.queries),
            crate::CssValueOrigin::Programmatic,
        )
    };
    let end = input.position().byte_index();
    let semicolon = if source.as_bytes().get(end) == Some(&b';') {
        crate::CssValueOrigin::Parsed(
            CssParsedOrigin::from_range(recovery.source_snapshot(), end..end + 1)
                .expect("custom-media semicolon"),
        )
    } else {
        crate::CssValueOrigin::Programmatic
    };
    Ok(CustomMediaPrelude {
        name,
        body,
        body_origin,
        semicolon,
        diagnostics,
        implicit,
    })
}
fn finish_custom_media(
    source: &str,
    prelude: CustomMediaPrelude,
    start: &ParserState,
    recovery: &RecoveryState,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
) -> crate::CssCustomMediaRule {
    let mut token_input = ParserInput::new(source);
    let mut parser = Parser::new(&mut token_input);
    parser.reset(start);
    let at = crate::CssComponentValue::collect_from_parser(&mut parser, recovery.source_snapshot())
        .expect("parsed custom-media at-keyword");
    diagnostics.extend(prelude.diagnostics);
    recovery.retain_component_closures(prelude.implicit);
    crate::CssCustomMediaRule::parsed(
        prelude.name,
        prelude.body,
        at.origin().clone(),
        prelude.body_origin,
        prelude.semicolon,
    )
}

#[cfg(test)]
mod splice_tests {
    use super::*;

    #[test]
    fn scoped_splice_orders_empty_styles_by_parser_produced_rule_positions() {
        let source = "@scope{.before-empty{}@layer{}.after-empty{}}";
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let [CssRule::Scope(scope)] = report.syntax().rules() else {
            panic!("expected one scope rule");
        };
        let [
            CssScopedRule::Style(before),
            CssScopedRule::LayerBlock(recovered),
            CssScopedRule::Style(after),
        ] = scope.rules().rules()
        else {
            panic!("expected scoped splice fixture in authored order");
        };
        assert!(before.declarations().is_empty());
        assert!(after.declarations().is_empty());
        assert!(
            before.position().byte_offset() < recovered.position().byte_offset()
                && recovered.position().byte_offset() < after.position().byte_offset()
        );

        let spliced = splice_scoped_rule_list(
            &[
                CssScopedRule::Style(before.clone()),
                CssScopedRule::Style(after.clone()),
            ],
            &[],
            recovered.position().byte_offset().value(),
            vec![CssRule::LayerBlock(CssLayerBlockRule::new(
                recovered.name().cloned(),
                Vec::new(),
                recovered.position(),
            ))],
        );
        assert_eq!(spliced, scope.rules().rules());

        let stable_tie = splice_scoped_rule_list(
            &[
                CssScopedRule::Style(before.clone()),
                CssScopedRule::Style(after.clone()),
            ],
            &[],
            before.position().byte_offset().value(),
            vec![CssRule::LayerBlock(CssLayerBlockRule::new(
                recovered.name().cloned(),
                Vec::new(),
                recovered.position(),
            ))],
        );
        assert_eq!(
            stable_tie,
            [
                CssScopedRule::Style(before.clone()),
                CssScopedRule::LayerBlock(recovered.clone()),
                CssScopedRule::Style(after.clone()),
            ]
        );
    }
}
