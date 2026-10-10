//! Strict CSS ingestion for Surgeist style sheets.
//!
//! This module parses CSS syntax into CSS-owned authored syntax values. It is
//! strict by design: unsupported selectors, at-rules, properties, and values are
//! errors instead of browser-style recoverable invalid declarations.
//!
//! Parse failures expose typed [`ErrorKind`] values plus source line and column
//! information so callers do not need to parse display strings.

mod alignment;
mod background;
mod border_color;
mod border_radius;
mod border_style;
mod border_width;
mod box_model;
mod box_spacing;
pub(crate) mod color_profile;
mod contain_intrinsic_size;
pub(crate) mod counter_style;
mod declaration_block;
mod declaration_list;
pub(crate) use declaration_block::parse_with_context as parse_declaration_block_contents_with_context;
pub use declaration_block::{
    parse_declaration_block_contents, parse_declaration_block_contents_with_limits,
};
mod descriptor_body;
mod descriptor_values;
mod effects;
mod flex;
mod font_controls;
pub(crate) mod font_face;
pub(crate) mod font_feature_values;
mod font_palette;
pub(crate) mod font_palette_values;
mod font_settings;
mod font_variant;
mod fragments;
mod style_selector_fragments;
pub use style_selector_fragments::{
    CssAdmittedStyleSelectors, CssParsedStyleSelectors, CssStyleSelectorContext,
    parse_style_selector_list, parse_style_selector_list_with_limits,
};
mod import;
use import::{CssImportPrelude, parse_import_prelude};
pub(crate) use import::{construct_import, import_boundaries_match};
mod reconstruction;
use reconstruction::{
    BoundedParseContext, BoundedParseSyntax, isolate_source_span, parse_bounded,
    parse_sheet_bounded, rule_start, scoped_rule_into_chunk_rule, scoped_rule_start,
};
#[cfg(test)]
mod native_batch_boundary_tests;
mod rule_candidate;
#[cfg(test)]
mod style_scope_media_separator_tests;
mod syntax_bridge;
pub use declaration_list::parse_declaration_list_text;
pub(crate) use declaration_list::parse_declaration_list_text_with_context;
pub(crate) use page::{first_page_component, is_page_margin_property, page_declaration_violation};
pub use rule_candidate::{
    CssAdmittedRule, CssRuleAdmissionContext, CssRuleSyntax, CssRuleSyntaxKind,
    classify_rule_syntax, classify_rule_syntax_with_limits,
};
mod gap;
pub(crate) mod image_1d;
mod masking;
mod ui;
pub use fragments::{
    parse_color_profile_block, parse_color_profile_descriptor_value, parse_counter_style_block,
    parse_counter_style_descriptor_value, parse_cssom_media_query, parse_declaration,
    parse_font_face_block, parse_font_face_descriptor_value, parse_font_feature_display_value,
    parse_font_feature_value, parse_font_feature_value_block, parse_font_feature_values_block,
    parse_font_palette_descriptor_value, parse_font_palette_values_block, parse_group_block,
    parse_keyframe_declaration_block, parse_keyframes_block, parse_margin_block, parse_media_query,
    parse_media_query_list, parse_page_block, parse_page_descriptor_value,
    parse_page_selector_list, parse_property_value_text, parse_property_value_text_for_grammar,
    parse_relative_selector_list, parse_rule, parse_scope_block, parse_scoped_group_block,
    parse_selector, parse_selector_list, parse_style_block, parse_supports_test_block,
};
pub(crate) use fragments::{
    parse_declaration_with_context, parse_group_block_with_context,
    parse_keyframe_declaration_block_with_context, parse_keyframes_block_with_context,
    parse_margin_block_with_context, parse_page_block_with_context,
    parse_property_value_text_for_grammar_with_context, parse_property_value_text_with_context,
    parse_rule_with_context, parse_scope_block_with_context, parse_scoped_group_block_with_context,
    parse_style_block_with_context,
};
mod color;
mod color_adjustment;
mod container_properties;
mod container_query;
mod container_scroll;
mod container_style;
mod content_values;
mod generated_content;
mod grid;
mod grid_placement;
mod inset;
mod item_flow;
mod keyframe_fragments;
mod keyframes;
pub use keyframe_fragments::{
    CssParsedKeyframeRule, CssParsedKeyframeSelectors, parse_keyframe_rule,
    parse_keyframe_selector_list,
};
mod layout;
mod list_styles;
mod motion;
mod multicolumn;
mod nesting;
mod overflow_controls;
mod page;
mod position;
mod queries;
mod query_components;
mod quirky_color;
mod quirky_length;
mod shape_outside;
use motion::*;
mod shapes;
use shape_outside::parse_shape_outside;
mod view_transitions;
use view_transitions::parse_view_transition_name;
mod will_change;
// Shared checked media construction uses the same private admission engine.
pub(crate) use queries::{construct_media_condition, construct_media_query};
pub(crate) use supports::{
    construct_supports_condition, construct_supports_condition_with_context,
    construct_supports_declaration, construct_supports_declaration_with_context,
};
mod conditional_chains;
mod recovery;
pub(crate) use recovery::finish_report;
mod scroll_snap;
mod scrollbar;
mod selectors;
mod sizing;
mod sizing_controls;
mod speech;
mod supports;
mod when;
use crate::{CssScopedElseRule, CssScopedWhenRule, CssWhenCondition};
pub(crate) use queries::construct_when_media_feature;
pub(crate) use when::construct_when_condition;
mod text_alignment;
mod text_flow;
mod timing;
mod typography;
mod url;
mod values;
mod variables;
pub(crate) use fragments::parse_svg_glyph_attribute_value;
pub(crate) use variables::first_substitution_origin;

use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, Delimiter, ParseError, Parser, ParserInput,
    ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, Token,
    match_ignore_ascii_case,
};

use crate::{
    CssColorProfileDescriptorKind, CssColorProfileDescriptorValue, CssColorProfileRuleName,
    CssFontPaletteDescriptorKind, CssFontPaletteDescriptorValue, CssFontPaletteName,
};
use alignment::*;
use background::*;
use border_color::*;
use border_radius::*;
use border_style::*;
use border_width::*;
use box_model::*;
use box_spacing::*;
use color::parse_color;
use color_adjustment::*;
use contain_intrinsic_size::*;
use container_properties::*;
#[cfg(test)]
pub(crate) use container_query::parse_container_condition_for_test;
use container_query::{collect_container_components, container_prelude_from_components};
pub(crate) use container_query::{
    construct_container_condition, construct_container_prelude, container_condition_from_enclosed,
};
use counter_style::{CounterStylePrelude, parse_counter_style_name, parse_counter_style_rule};
use effects::*;
use flex::*;
use font_controls::*;
use font_face::parse_font_face_rule;
use font_palette::parse_font_palette;
use font_settings::*;
use font_variant::*;
use gap::*;
use generated_content::*;
use grid::*;
use grid_placement::{parse_grid_area, parse_grid_line, parse_grid_line_range};
use inset::*;
use item_flow::*;
use keyframes::{parse_keyframes_name, parse_keyframes_rule};
use layout::*;
use list_styles::*;
use masking::*;
use multicolumn::*;
use nesting::{parse_style_contents, parse_style_rule_block};
use overflow_controls::*;
use page::{parse_page_rule, parse_page_selector};
use position::*;
use queries::parse_media_query_list as parse_media_query_list_inner;
#[cfg(test)]
pub(crate) use queries::parse_media_query_list_for_test;
use recovery::{
    RecoveryLoopOutcome, RecoveryProgress, RecoveryState, StyleContextCaptures,
    recovery_action_for_error,
};
use scroll_snap::*;
use scrollbar::*;
use selectors::{
    SelectorRecovery, parse_rule_selector_list, parse_scope_boundary_selector_list,
    parse_scoped_style_selector_list,
};
use sizing::{parse_max_size_value, parse_size_value};
use sizing_controls::*;
use supports::{parse_supports_condition, with_supports_prelude_context};
use ui::*;
use will_change::*;
pub(crate) mod named_supports;
use speech::*;
use text_alignment::*;
use text_flow::*;
use timing::*;
use typography::*;
mod text_decoration;
use text_decoration::*;
pub(crate) use variables::contains_substitution;
use variables::{
    collect_authored_declaration_value, parse_custom_property_name, parse_custom_property_value,
};

use crate::component_values::CssComponentValues;
use crate::error::{
    CssFeatureId, Error, basic, from_parse_error, from_rule_parse_error, invalid_at_rule_block,
    invalid_at_rule_body, invalid_at_rule_placement, invalid_custom_declaration_annotation,
    invalid_descriptor_annotation, invalid_known_declaration_annotation, invalid_syntax,
    property_name_error, unexpected_at, with_at_rule_prelude_context, with_media_query_context,
    with_property_context,
};
use crate::properties::*;
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
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
    CssFeatureId::new("ext.rule.supports-condition"),
    CssFeatureId::new("baseline.rule.import"),
    CssFeatureId::new("ext.import.layer"),
    CssFeatureId::new("ext.stylesheet.prelude-order"),
    CssFeatureId::new("baseline.rule.layer-statement"),
    CssFeatureId::new("baseline.rule.layer-block"),
    CssFeatureId::new("baseline.rule.media"),
    CssFeatureId::new("official.rule.conditional-group-context"),
    CssFeatureId::new("baseline.rule.scope"),
    CssFeatureId::new("later.rule.namespace"),
    CssFeatureId::new("later.rule.supports"),
    CssFeatureId::new("later.rule.counter-style"),
    CssFeatureId::new("later.rule.page"),
    CssFeatureId::new("later.rule.font-feature-values"),
    CssFeatureId::new("later.rule.font-palette-values"),
    CssFeatureId::new("ext.rule.custom-media"),
];

static IMPLEMENTED_QUALIFIED_RULES: &[CssFeatureId] = &[CssFeatureId::new("baseline.rule.style")];

static IMPLEMENTED_DECLARATIONS: &[CssFeatureId] = &[
    CssFeatureId::new("foundation.declaration-list.style-attribute"),
    CssFeatureId::new("foundation.declaration.importance"),
    CssFeatureId::new("official.declaration.generic"),
];
static IMPLEMENTED_SVG_GLYPH: &[CssFeatureId] = &[CssFeatureId::new(
    "interop.property.svg-glyph-orientation-vertical",
)];

static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("official.value.stylesheet"),
    CssFeatureId::new("official.value.declaration-list"),
];

static IMPLEMENTED_CONTAINER_EXTENSIONS: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.rule.container")];

static ATOMIC_IMPLEMENTATION_INVENTORIES: &[CssAtomicImplementationInventory] = &[
    CssAtomicImplementationInventory {
        module: "crate::parser::when",
        kind: CssAtomicImplementationKind::Rule,
        stable_ids: when::IMPLEMENTED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::quirky_length",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: quirky_length::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::parse_svg_glyph_declaration_body",
        kind: CssAtomicImplementationKind::PropertyExtension,
        stable_ids: IMPLEMENTED_SVG_GLYPH,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::quirky_color",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: quirky_color::IMPLEMENTED_SHARED_VALUES,
    },
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
        module: "crate::parser::font_palette_values",
        kind: CssAtomicImplementationKind::Rule,
        stable_ids: font_palette_values::IMPLEMENTED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::color_profile",
        kind: CssAtomicImplementationKind::Rule,
        stable_ids: color_profile::IMPLEMENTED_RULES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::font_face",
        kind: CssAtomicImplementationKind::Descriptor,
        stable_ids: font_face::IMPLEMENTED_DESCRIPTORS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::font_palette_values",
        kind: CssAtomicImplementationKind::Descriptor,
        stable_ids: font_palette_values::IMPLEMENTED_DESCRIPTORS,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::color_profile",
        kind: CssAtomicImplementationKind::Descriptor,
        stable_ids: color_profile::IMPLEMENTED_DESCRIPTORS,
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
        module: "crate::parser::image_1d",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: image_1d::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::background",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: background::IMPLEMENTED_SHARED_VALUES,
    },
    CssAtomicImplementationInventory {
        module: "crate::parser::layout",
        kind: CssAtomicImplementationKind::SharedValue,
        stable_ids: layout::IMPLEMENTED_SHARED_VALUES,
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
        $all_value:ty,
        $all_parser:ident, $all_dispatch:block $(, expansion = $all_expansion:ident { $($all_metadata:tt)* })?;
        $(
        $variant:ident, $canonical:literal, [$($alias:literal),*], $stable_id:literal,
        $value:ty, $wrapper:ident, $accessor:ident, $parser:ident, $dispatch:block
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


                    let keyword = $all_dispatch;
                    Ok(CssKnownDeclaration::from_global(CssKnownProperty::All, keyword))
                }
                $(crate::CssKnownProperty::$variant => {
                    // Keep each property's parsed value and wrapper construction in its own
                    // frame: one generated dispatch frame exhausts deep rule-parsing stacks.
                    #[inline(never)]
                    fn parse_variant<'i, 't>(
                        authored: CssAuthoredDeclarationValue,
                        $input: &mut Parser<'i, 't>,
                        $numeric: &crate::numeric::NumericInputContext<'_>,
                    ) -> std::result::Result<CssKnownDeclaration, ParseError<'i, Error>> {
                        let _ = $numeric;
                        let _authored_value_type = std::marker::PhantomData::<$value>;

                        let value = $dispatch;
                        Ok(CssKnownDeclaration::from_value(
                            CssKnownDeclarationValue::$variant(CssDeclaredValue::Value(
                                $wrapper::new(authored, value),
                            )),
                        ))
                    }
                    parse_variant(authored, $input, $numeric)
                },)*
            }
        }
    };
}

property_schema!(define_property_dispatch, input, numeric);

fn parse_all_property<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGlobalKeyword, ParseError<'i, Error>> {
    Err(unexpected_at(input.current_source_location()))
}

/// Parses a UTF-8 stylesheet into valid authored syntax and recovery diagnostics.
///
/// The ordinary parser retains valid top-level rules in source order and reports
/// each discarded top-level rule with its complete balanced source span. Parsed
/// `@charset` rules are unrecognized and dropped; top-level CDO/CDC tokens are
/// ignored without diagnostics. The UTF-8 input is not decoded. The stylesheet
/// root is structural depth zero; up to 256 shared rule-block/component/function
/// levels are retained, and the first level
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
    parse_sheet_with_context(source, crate::CssParserContext::default())
}

pub(crate) fn parse_sheet_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<CssSheet> {
    let source_snapshot = CssSourceSnapshot::new(source);
    parse_sheet_snapshot(&source_snapshot, parser_context)
}

pub(crate) fn parse_sheet_snapshot(
    source_snapshot: &CssSourceSnapshot,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<CssSheet> {
    let source = source_snapshot.as_str();
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
                        source_snapshot,
                        0,
                        BoundedParseContext::Rules { top_level: true },
                        StyleContextCaptures::default(),
                        parser_context,
                    )
                });
            match parser {
                Ok(parser) => match parser.join() {
                    Ok(report) => report,
                    Err(panic) => std::panic::resume_unwind(panic),
                },
                Err(_) => parse_sheet_bounded(
                    source,
                    source_snapshot,
                    0,
                    BoundedParseContext::Rules { top_level: true },
                    StyleContextCaptures::default(),
                    parser_context,
                ),
            }
        });
        return recovery::finish_report(source, conditional_chains::sheet(source, report));
    }
    let report = parse_sheet_bounded(
        source,
        source_snapshot,
        0,
        BoundedParseContext::Rules { top_level: true },
        StyleContextCaptures::default(),
        parser_context,
    );
    recovery::finish_report(source, conditional_chains::sheet(source, report))
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
    parse_style_attribute_with_context(source, crate::CssParserContext::default())
}

pub(crate) fn parse_style_attribute_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<CssDeclarationList> {
    if recovery::maximum_nested_depth(source) > recovery::DIRECT_PARSE_DEPTH {
        let report = std::thread::scope(|scope| {
            let parser = std::thread::Builder::new()
                .name("surgeist-css-bounded-style-attribute-parser".to_owned())
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    parse_style_attribute_inner(source, parser_context)
                });
            match parser {
                Ok(parser) => match parser.join() {
                    Ok(report) => report,
                    Err(panic) => std::panic::resume_unwind(panic),
                },
                Err(_) => parse_style_attribute_inner(source, parser_context),
            }
        });
        return recovery::finish_report(source, report);
    }
    recovery::finish_report(source, parse_style_attribute_inner(source, parser_context))
}

fn parse_style_attribute_inner(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<CssDeclarationList> {
    let recovery = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
        .with_parser_context(parser_context);
    let working_source = crate::tokenization::prepare(source);
    let mut input = ParserInput::new(&working_source);
    let mut parser = Parser::new(&mut input);
    let recovered = parse_style_attribute_declarations(source, &mut parser, recovery.clone());
    let syntax = recovered.syntax;
    let mut diagnostics = recovered.diagnostics;
    diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
    crate::CssParseReport::new(syntax, diagnostics)
}

#[derive(Clone, Copy)]
enum ScopedBodyKind {
    Scope,
    OrdinaryGroup,
}

fn parse_style_context_inner(
    source: &str,
    recovery: RecoveryState,
) -> crate::CssParseReport<CssSheet> {
    let working_source = crate::tokenization::prepare(source);
    let mut input = ParserInput::new(&working_source);
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
    body: ScopedBodyKind,
) -> crate::CssParseReport<CssSheet> {
    let working_source = crate::tokenization::prepare(source);
    let mut input = ParserInput::new(&working_source);
    let mut parser = Parser::new(&mut input);
    match parse_scoped_rule_list(
        source,
        &mut parser,
        recovery.clone(),
        has_style_ancestor,
        body,
    ) {
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

fn parse_sheet_inner(
    source: &str,
    recovery: RecoveryState,
    top_level: bool,
) -> crate::CssParseReport<CssSheet> {
    let working_source = crate::tokenization::prepare(source);
    let mut input = ParserInput::new(&working_source);
    let mut parser = Parser::new(&mut input);
    let mut rule_parser = if top_level {
        StrictRuleParser::top_level(source, recovery.clone())
    } else {
        StrictRuleParser::nested(source, recovery.clone())
    };
    let mut sheet = CssSheet::new();
    let mut diagnostics = Vec::new();
    let mut previous_end = parser.position().byte_index();

    {
        let (document, selected) = match syntax_bridge::rules(source, &parser, &recovery, top_level)
        {
            Ok(selected) => selected,
            Err(error) => {
                return crate::CssParseReport::new(
                    sheet,
                    vec![syntax_bridge::arena_error(source, error)],
                );
            }
        };
        for selected in &selected {
            let progress = RecoveryProgress::record(&parser);
            let result = syntax_bridge::parse_selected(
                source,
                &mut parser,
                &mut rule_parser,
                &recovery,
                &document,
                selected,
            );
            let failed_block_error = result.as_ref().err().and_then(|(_, failed_unit)| {
                consume_failed_rule_block(
                    source,
                    &mut parser,
                    true,
                    &recovery,
                    structural_recovery_production(failed_unit),
                )
                .1
            });
            let retained = result.is_ok();
            let progress_outcome = progress.finish(&mut parser, retained);
            let unit_end = parser.position().byte_index();
            diagnostics.append(&mut rule_parser.diagnostics);
            match result {
                Ok(parsed_rules) => {
                    for rule in parsed_rules {
                        if let CssRule::Namespace(namespace) = &rule
                            && let Some(previous) =
                                sheet
                                    .rules()
                                    .iter()
                                    .rev()
                                    .find_map(|retained| match retained {
                                        CssRule::Namespace(previous)
                                            if previous.prefix() == namespace.prefix() =>
                                        {
                                            Some(previous)
                                        }
                                        _ => None,
                                    })
                        {
                            let position = namespace.position().expect("parsed namespace rule");
                            let previous_position =
                                previous.position().expect("parsed namespace rule");
                            let span = crate::CssSourceSpan::new(
                                position,
                                crate::CssSourcePosition::from_byte_offset_in(source, unit_end),
                            )
                            .expect("retained namespace rule lies within its source unit");
                            diagnostics.push(
                                crate::CssRecoveryDiagnostic::new(
                                    crate::error::namespace_redeclaration(
                                        namespace.prefix().cloned(),
                                        position,
                                        previous_position,
                                    ),
                                    span,
                                    crate::CssRecoveryAction::RetainNonconformingRule,
                                )
                                .expect("namespace error is at the retained rule's source start"),
                            );
                        }
                        sheet.push_rule(rule);
                    }
                }
                Err((error, failed_unit)) => {
                    let error = failed_block_error.unwrap_or(*error);
                    let unit_start =
                        recovery_unit_start(source, previous_end, unit_end, failed_unit);
                    let ordinary_action = if failed_unit.trim_start().starts_with('@') {
                        crate::CssRecoveryAction::DropAtRule
                    } else {
                        crate::CssRecoveryAction::DropQualifiedRule
                    };
                    let action = recovery_action_for_error(&error, ordinary_action);
                    let error = from_rule_parse_error(source, failed_unit, error);
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

    diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));

    crate::CssParseReport::new(sheet, diagnostics)
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
            diagnostics: Vec::new(),
            recovery,
        }
    }

    fn nested(source: &'s str, recovery: RecoveryState) -> Self {
        Self {
            source,
            top_level_phase: None,
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
    FontPaletteValues(CssFontPaletteName),
    ColorProfile(CssColorProfileRuleName),
    Import(Box<CssImportPrelude>),
    Namespace(CssNamespacePrelude),
    CounterStyle(CounterStylePrelude),
    Page(CssPageSelectorList),
    Layer(Vec<CssLayerName>),
    FontFace,
    Keyframes(CssKeyframesName),
    Media(CssMediaQueryList),
    Supports(CssSupportsCondition),
    SupportsCondition(named_supports::NamedSupportsPrelude),
    Container(CssContainerPrelude),
    When(CssWhenCondition, Vec<usize>),
    Else(Option<CssWhenCondition>, Vec<usize>),
    Scope(CssScopePrelude),
}

impl StrictAtRulePrelude {
    fn production(&self) -> &'static str {
        match self {
            Self::CustomMedia(_) => "ext.rule.custom-media",
            Self::FontFeatureValues(_) => "later.rule.font-feature-values",
            Self::FontPaletteValues(_) => "later.rule.font-palette-values",
            Self::ColorProfile(_) => "interop.rule.color-profile",
            Self::Import(_) => "baseline.rule.import",
            Self::Namespace(_) => "later.rule.namespace",
            Self::CounterStyle(_) => "later.rule.counter-style",
            Self::Page(_) => "later.rule.page",
            Self::Layer(_) => "baseline.rule.layer-block",
            Self::FontFace => "baseline.rule.font-face",
            Self::Keyframes(_) => "baseline.rule.keyframes",
            Self::Media(_) => "baseline.rule.media",
            Self::Supports(_) => "baseline.rule.supports",
            Self::SupportsCondition(_) => "ext.rule.supports-condition",
            Self::Container(_) => "baseline.rule.container",
            Self::When(_, _) => "ext.rule.when",
            Self::Else(_, _) => "ext.rule.else",
            Self::Scope(_) => "baseline.rule.scope",
        }
    }
}

struct CssNamespacePrelude {
    prefix: Option<CssNamespacePrefix>,
    name: CssNamespaceName,
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
                let prelude = parse_namespace_prelude(input).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "namespace",
                        "later.rule.namespace",
                        "an optional prefix followed by one string or URL namespace name",
                    )
                })?;
                Ok(StrictAtRulePrelude::Namespace(prelude))
            },
            "counter-style" => Ok(StrictAtRulePrelude::CounterStyle(
                parse_counter_style_prelude(self.source, input, self.recovery.source_snapshot())?,
            )),
            "page" => Ok(StrictAtRulePrelude::Page(parse_page_prelude(self.source, input, self.recovery.source_snapshot())?)),
            "custom-media" => Ok(StrictAtRulePrelude::CustomMedia(Box::new(parse_custom_media_prelude(self.source, input, &self.recovery)?))),
            "font-feature-values" => Ok(StrictAtRulePrelude::FontFeatureValues(font_feature_values::parse_families(self.source, input, &self.recovery)?)),
            "font-palette-values" => Ok(StrictAtRulePrelude::FontPaletteValues(font_palette_values::parse_name(self.source, input, &self.recovery)?)),
            "color-profile" => Ok(StrictAtRulePrelude::ColorProfile(color_profile::parse_name(self.source, input, &self.recovery)?)),
            "font-face" => {
                parse_font_face_prelude(input)?;
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
                let name = parse_keyframes_prelude(input)?;
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
                        invalid_syntax(input.current_source_location()),
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
            "supports-condition" => Ok(StrictAtRulePrelude::SupportsCondition(
                named_supports::parse_prelude(input, &self.recovery)?,
            )),
            "when" => {
                let (condition, implicit) = when::parse_prelude(self.source, input, &self.recovery, name.as_ref(), false)?;
                Ok(StrictAtRulePrelude::When(condition.expect("required condition"), implicit))
            },
            "else" => {
                let (condition, implicit) = when::parse_prelude(self.source, input, &self.recovery, name.as_ref(), true)?;
                Ok(StrictAtRulePrelude::Else(condition, implicit))
            },
            "container" => {
                let prelude = parse_container_prelude(self.source, input, &self.recovery)
                    .map_err(with_container_prelude_context)?;
                if !input.is_exhausted() {
                    return Err(with_at_rule_prelude_context(
                        invalid_syntax(input.current_source_location()),
                        "container",
                        "baseline.rule.container",
                        "the end of the @container prelude",
                    ));
                }
                Ok(StrictAtRulePrelude::Container(prelude))
            },
            "scope" => Ok(StrictAtRulePrelude::Scope(
                parse_scope_prelude(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                    selectors::SelectorAnchorMode::Nesting,
                    false,
                ).map_err(with_scope_prelude_context)?,
            )),
            _ => Err(input.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name))),
        }
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> std::result::Result<Self::AtRule, ()> {
        let result = match prelude {
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
                ));
                self.mark_successful_import();
                Ok(vec![rule])
            }
            StrictAtRulePrelude::Namespace(prelude) => {
                let rule = CssNamespaceRule::new(prelude.prefix, prelude.name)
                    .with_position(self.recovery.source_position(start.position().byte_index()));
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
                    self.recovery.source_position(start.position().byte_index()),
                ))])
            }
            StrictAtRulePrelude::FontFeatureValues(_) => Err(()),
            StrictAtRulePrelude::FontPaletteValues(_) => Err(()),
            StrictAtRulePrelude::ColorProfile(_) => Err(()),
            StrictAtRulePrelude::FontFace => Err(()),
            StrictAtRulePrelude::Keyframes(_) => Err(()),
            StrictAtRulePrelude::Media(_) => Err(()),
            StrictAtRulePrelude::Supports(_) => Err(()),
            StrictAtRulePrelude::SupportsCondition(_) => Err(()),
            StrictAtRulePrelude::Container(_) => Err(()),
            StrictAtRulePrelude::When(_, _)
            | StrictAtRulePrelude::Else(_, _)
            | StrictAtRulePrelude::Scope(_) => Err(()),
        };
        if result.is_ok() {
            syntax_bridge::retain_statement_eof(
                self.source,
                &self.recovery,
                start,
                &mut self.diagnostics,
            );
        }
        result
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
                    self.recovery.source_position(start.position().byte_index()),
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
            StrictAtRulePrelude::FontPaletteValues(name) => {
                let rule = font_palette_values::parse_rule(
                    self.source,
                    name,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::FontPaletteValues(rule)])
            }
            StrictAtRulePrelude::ColorProfile(name) => {
                let rule = color_profile::parse_rule(
                    self.source,
                    name,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::ColorProfile(rule)])
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
                    self.recovery.source_position(start.position().byte_index()),
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
                    self.recovery.source_position(start.position().byte_index()),
                ))])
            }
            StrictAtRulePrelude::SupportsCondition(prelude) => {
                let rule = named_supports::parse_rule(
                    self.source,
                    prelude,
                    start,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                )?;
                Ok(vec![CssRule::SupportsCondition(rule)])
            }
            StrictAtRulePrelude::When(condition, implicit) => {
                let recovered =
                    parse_nested_group_rules(self.source, input, self.recovery.clone())?;
                self.recovery.retain_component_closures(implicit);
                self.diagnostics.extend(recovered.diagnostics);
                self.mark_successful_body_rule();
                Ok(when::assemble_when_rule(
                    condition,
                    recovered.syntax,
                    self.recovery.source_position(start.position().byte_index()),
                ))
            }
            StrictAtRulePrelude::Else(condition, implicit) => {
                let recovered =
                    parse_nested_group_rules(self.source, input, self.recovery.clone())?;
                self.recovery.retain_component_closures(implicit);
                self.diagnostics.extend(recovered.diagnostics);
                Ok(when::assemble_else_rule(
                    condition,
                    recovered.syntax,
                    self.recovery.source_position(start.position().byte_index()),
                ))
            }
            StrictAtRulePrelude::Container(prelude) => {
                let recovered =
                    parse_nested_group_rules(self.source, input, self.recovery.clone())?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::Container(CssContainerRule::new(
                    prelude,
                    rules,
                    self.recovery.source_position(start.position().byte_index()),
                ))])
            }
            StrictAtRulePrelude::Scope(prelude) => {
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    false,
                    ScopedBodyKind::Scope,
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                self.mark_successful_body_rule();
                Ok(vec![CssRule::Scope(CssScopeRule::new(
                    prelude.root,
                    prelude.limit,
                    rules,
                    self.recovery.source_position(start.position().byte_index()),
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
            self.recovery.source_position(start.position().byte_index()),
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

// Component construction uses the same finite selector as parsing. Transport
// models only classify clauses; only original components back the returned rule.
fn parse_container_prelude<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    recovery: &RecoveryState,
) -> std::result::Result<CssContainerPrelude, ParseError<'i, Error>> {
    let (values, implicit) = collect_container_components(source, input, recovery)?;
    let prelude = container_prelude_from_components(values, input.current_source_location())?;
    recovery.retain_component_closures(implicit);
    Ok(prelude)
}

fn with_container_prelude_context<'i>(error: ParseError<'i, Error>) -> ParseError<'i, Error> {
    if queries::media_terminal_error(&error) {
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
    let mut rule_parser = StrictRuleParser::nested(source, recovery.clone());
    let mut rules = Vec::new();
    let mut diagnostics = Vec::new();
    let mut previous_end = input.position().byte_index();
    {
        let (document, selected) =
            syntax_bridge::rules(source, input, &recovery, false).map_err(|error| {
                crate::error::invalid_component_value(input.current_source_location(), error)
            })?;
        for selected in &selected {
            let progress = RecoveryProgress::record(input);
            let item = syntax_bridge::parse_selected(
                source,
                input,
                &mut rule_parser,
                &recovery,
                &document,
                selected,
            );
            let failed_block_error = item.as_ref().err().and_then(|(_, failed_unit)| {
                consume_failed_rule_block(
                    source,
                    input,
                    true,
                    &rule_parser.recovery,
                    structural_recovery_production(failed_unit),
                )
                .1
            });
            let retained = item.is_ok();
            let progress_outcome = progress.finish(input, retained);
            let unit_end = input.position().byte_index();
            diagnostics.append(&mut rule_parser.diagnostics);
            match item {
                Ok(parsed_rules) => rules.extend(parsed_rules),
                Err((error, failed_unit)) => {
                    let error = failed_block_error.unwrap_or(*error);
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

fn parse_page_prelude<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    snapshot: &CssSourceSnapshot,
) -> std::result::Result<CssPageSelectorList, ParseError<'i, Error>> {
    let selector = parse_page_selector(input, snapshot).map_err(|error| {
        with_at_rule_prelude_context(
            error,
            "page",
            "later.rule.page",
            "an empty prelude or a complete named/pseudo Page selector list",
        )
    })?;
    let following = source
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
    Ok(selector)
}

fn parse_scoped_rule_list<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
    has_style_ancestor: bool,
    body: ScopedBodyKind,
) -> std::result::Result<Recovered<CssScopedRuleList>, ParseError<'i, Error>> {
    if has_style_ancestor {
        recovery.record_style_context(input.position().byte_index());
    }
    let mut rule_parser = ScopedRuleParser {
        source,
        diagnostics: Vec::new(),
        recovery: recovery.clone(),
        has_style_ancestor,
        body,
        boundary: nesting::StyleRuleBoundary::None,
        qualified_resource_error: None,
        rejected_at_rule_start: None,
    };
    let mut rules = Vec::new();
    let mut declarations = Vec::new();
    let mut previous_end = input.position().byte_index();
    let mut generic = if !has_style_ancestor && matches!(body, ScopedBodyKind::OrdinaryGroup) {
        let (document, selected) =
            syntax_bridge::rules(source, input, &recovery, false).map_err(|error| {
                crate::error::invalid_component_value(input.current_source_location(), error)
            })?;
        Some((document, selected, 0_usize))
    } else {
        None
    };
    let mut items = RuleBodyParser::new(input, &mut rule_parser);
    loop {
        let progress = RecoveryProgress::record(items.input);
        items.parser.boundary = nesting::StyleRuleBoundary::None;
        items.parser.qualified_resource_error = None;
        items.parser.rejected_at_rule_start = None;
        let item = if let Some((document, selected, next)) = &mut generic {
            let index = *next;
            *next += 1;
            selected.get(index).map(|selected| {
                syntax_bridge::parse_selected(
                    source,
                    items.input,
                    items.parser,
                    &recovery,
                    document,
                    selected,
                )
            })
        } else {
            items
                .next()
                .map(|item| item.map_err(|(error, failed_unit)| (Box::new(error), failed_unit)))
        };
        let qualified_resource_error = items.parser.qualified_resource_error.take();
        let rejected_at_rule_start = items.parser.rejected_at_rule_start.take();
        let Some(item) = item else { break };
        let (failed_at_block, failed_block_error) = item
            .as_ref()
            .err()
            .map(|(_, failed_unit)| {
                consume_failed_rule_block(
                    source,
                    items.input,
                    true,
                    &items.parser.recovery,
                    structural_recovery_production(failed_unit),
                )
            })
            .unwrap_or((false, None));
        let progress_outcome = progress.finish(items.input, item.is_ok());
        let unit_end = items.input.position().byte_index();
        if items.parser.boundary.partitions(failed_at_block) {
            flush_scoped_declarations(&mut declarations, &mut rules);
        }
        match item {
            Ok(ScopedBlockItem::Declaration(declaration)) => declarations.push(*declaration),
            Ok(ScopedBlockItem::Rules(parsed)) => rules.extend(parsed),
            Err((error, failed_unit))
                if has_style_ancestor
                    && is_declaration_recovery_unit(failed_unit)
                    && !failed_at_block
                    && qualified_resource_error.is_none() =>
            {
                if let Some(diagnostic) = block_item_diagnostic(
                    source,
                    *error,
                    failed_unit,
                    unit_end,
                    crate::CssRecoveryAction::DropDeclaration,
                ) {
                    items.parser.diagnostics.push(diagnostic);
                }
            }
            Err((error, failed_unit)) => {
                let error = if let Some(resource) = qualified_resource_error.or(failed_block_error)
                {
                    resource
                } else {
                    syntax_bridge::rejected_at_rule_error(
                        source,
                        &items.parser.recovery,
                        rejected_at_rule_start,
                        *error,
                    )
                };
                if let Some(diagnostic) = structural_rule_diagnostic(
                    source,
                    error,
                    failed_unit,
                    previous_end,
                    unit_end,
                    structural_recovery_action(failed_unit),
                ) {
                    items.parser.diagnostics.push(diagnostic);
                }
            }
        }
        previous_end = unit_end;
        if progress_outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }
    flush_scoped_declarations(&mut declarations, &mut rules);
    Ok(Recovered {
        syntax: CssScopedRuleList::from_rules(rules),
        diagnostics: rule_parser.diagnostics,
    })
}

fn flush_scoped_declarations(buffer: &mut Vec<CssDeclaration>, rules: &mut Vec<CssScopedRule>) {
    if !buffer.is_empty() {
        rules.push(CssScopedRule::NestedDeclarations(
            CssNestedDeclarationsRule::new(CssDeclarationList::new(std::mem::take(buffer))),
        ));
    }
}

fn parse_namespace_prelude<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssNamespacePrelude, ParseError<'i, Error>> {
    let prefix = input
        .try_parse(Parser::expect_ident_cloned)
        .ok()
        .map(|prefix| CssNamespacePrefix::new(prefix.to_string()));
    let name = input.expect_url_or_string().map_err(basic)?;
    input.expect_exhausted().map_err(basic)?;

    Ok(CssNamespacePrelude {
        prefix,
        name: CssNamespaceName::new(name.to_string()),
    })
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
        return Err(invalid_syntax(input.current_source_location()));
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

    CssLayerName::try_new(components).ok_or_else(|| invalid_syntax(location))
}

fn parse_scope_prelude<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    state: &RecoveryState,
    root_anchors: selectors::SelectorAnchorMode,
    relative_root: bool,
) -> std::result::Result<CssScopePrelude, ParseError<'i, Error>> {
    let root = if input.try_parse(Parser::expect_parenthesis_block).is_ok() {
        let _boundary_depth = state.enter_component_block(source, input, "baseline.rule.scope")?;
        Some(input.parse_nested_block(|input| {
            let mut recovery = SelectorRecovery::new(source, diagnostics, state.clone());
            parse_scope_boundary_selector_list(input, &mut recovery, root_anchors, relative_root)
        })?)
    } else {
        None
    };

    let limit = if input
        .try_parse(|input| input.expect_ident_matching("to"))
        .is_ok()
    {
        input.expect_parenthesis_block().map_err(basic)?;
        let _boundary_depth = state.enter_component_block(source, input, "baseline.rule.scope")?;
        Some(input.parse_nested_block(|input| {
            let mut recovery = SelectorRecovery::new(source, diagnostics, state.clone());
            parse_scope_boundary_selector_list(
                input,
                &mut recovery,
                selectors::SelectorAnchorMode::Scope,
                true,
            )
        })?)
    } else {
        None
    };

    if !input.is_exhausted() {
        return Err(invalid_syntax(input.current_source_location()));
    }

    Ok(CssScopePrelude { root, limit })
}

fn with_scope_prelude_context(error: ParseError<'_, Error>) -> ParseError<'_, Error> {
    if crate::error::is_nesting_limit_error(&error) {
        error
    } else {
        with_at_rule_prelude_context(
            error,
            "scope",
            "baseline.rule.scope",
            "a supported @scope prelude",
        )
    }
}

fn parse_counter_style_prelude<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    source_snapshot: &CssSourceSnapshot,
) -> Result<CounterStylePrelude, ParseError<'i, Error>> {
    let name = parse_counter_style_name(input, source_snapshot).map_err(|error| {
        with_at_rule_prelude_context(
            error,
            "counter-style",
            "later.rule.counter-style",
            "one non-reserved counter-style name",
        )
    })?;
    let following = source
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
    Ok(name)
}

fn parse_font_face_prelude<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<(), ParseError<'i, Error>> {
    if !input.is_exhausted() {
        return Err(with_at_rule_prelude_context(
            invalid_syntax(input.current_source_location()),
            "font-face",
            "baseline.rule.font-face",
            "an empty @font-face prelude",
        ));
    }
    Ok(())
}

fn parse_keyframes_prelude<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssKeyframesName, ParseError<'i, Error>> {
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
            invalid_syntax(input.current_source_location()),
            "keyframes",
            "baseline.rule.keyframes",
            "the end of the @keyframes prelude",
        ));
    }
    Ok(name)
}

enum ScopedBlockItem {
    Declaration(Box<CssDeclaration>),
    Rules(Vec<CssScopedRule>),
}

struct ScopedRuleParser<'s> {
    boundary: nesting::StyleRuleBoundary,
    qualified_resource_error: Option<ParseError<'s, Error>>,
    rejected_at_rule_start: Option<usize>,
    has_style_ancestor: bool,
    body: ScopedBodyKind,
    source: &'s str,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
}

impl<'s> ScopedRuleParser<'s> {
    // Keep descriptor parsing off the recursive group-rule frame. Deep scoped
    // rule lists run on the test thread's ordinary stack.
    #[inline(never)]
    fn parse_font_palette_block<'t>(
        &mut self,
        name: CssFontPaletteName,
        input: &mut Parser<'s, 't>,
        start: &ParserState,
    ) -> Result<Vec<CssScopedRule>, ParseError<'s, Error>> {
        let rule = font_palette_values::parse_rule(
            self.source,
            name,
            input,
            start,
            &mut self.diagnostics,
            self.recovery.clone(),
        )?;
        Ok(vec![CssScopedRule::FontPaletteValues(rule)])
    }
    #[inline(never)]
    fn parse_color_profile_block<'t>(
        &mut self,
        name: CssColorProfileRuleName,
        input: &mut Parser<'s, 't>,
        start: &ParserState,
    ) -> Result<Vec<CssScopedRule>, ParseError<'s, Error>> {
        let rule = color_profile::parse_rule(
            self.source,
            name,
            input,
            start,
            &mut self.diagnostics,
            self.recovery.clone(),
        )?;
        Ok(vec![CssScopedRule::ColorProfile(rule)])
    }
}

enum ScopedAtRulePrelude {
    Page(CssPageSelectorList),
    CounterStyle(CounterStylePrelude),
    FontFace,
    Keyframes(CssKeyframesName),
    CustomMedia(Box<CustomMediaPrelude>),
    FontFeatureValues(Vec<CssFontFaceFamily>),
    FontPaletteValues(CssFontPaletteName),
    ColorProfile(CssColorProfileRuleName),
    Media(CssMediaQueryList),
    Supports(CssSupportsCondition),
    SupportsCondition(named_supports::NamedSupportsPrelude),
    Container(CssContainerPrelude),
    When(CssWhenCondition, Vec<usize>),
    Else(Option<CssWhenCondition>, Vec<usize>),
    Layer(Vec<CssLayerName>),
    Scope(CssScopePrelude),
}

impl ScopedAtRulePrelude {
    fn production(&self) -> &'static str {
        match self {
            Self::Page(_) => "later.rule.page",
            Self::CounterStyle(_) => "later.rule.counter-style",
            Self::FontFace => "baseline.rule.font-face",
            Self::Keyframes(_) => "baseline.rule.keyframes",
            Self::CustomMedia(_) => "ext.rule.custom-media",
            Self::FontFeatureValues(_) => "later.rule.font-feature-values",
            Self::FontPaletteValues(_) => "later.rule.font-palette-values",
            Self::ColorProfile(_) => "interop.rule.color-profile",
            Self::Media(_) => "baseline.rule.media",
            Self::Supports(_) => "baseline.rule.supports",
            Self::SupportsCondition(_) => "ext.rule.supports-condition",
            Self::Container(_) => "baseline.rule.container",
            Self::When(_, _) => "ext.rule.when",
            Self::Else(_, _) => "ext.rule.else",
            Self::Layer(_) => "baseline.rule.layer-block",
            Self::Scope(_) => "baseline.rule.scope",
        }
    }
}

impl<'i> AtRuleParser<'i> for ScopedRuleParser<'i> {
    type Prelude = ScopedAtRulePrelude;
    type AtRule = ScopedBlockItem;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        self.boundary = nesting::StyleRuleBoundary::AtRule;
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
                        invalid_syntax(input.current_source_location()),
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
            "supports-condition" => Ok(ScopedAtRulePrelude::SupportsCondition(
                named_supports::parse_prelude(input, &self.recovery)?,
            )),
            "when" => {
                let (condition, implicit) = when::parse_prelude(self.source, input, &self.recovery, name.as_ref(), false)?;
                Ok(ScopedAtRulePrelude::When(condition.expect("required condition"), implicit))
            },
            "else" => {
                let (condition, implicit) = when::parse_prelude(self.source, input, &self.recovery, name.as_ref(), true)?;
                Ok(ScopedAtRulePrelude::Else(condition, implicit))
            },
            "container" => {
                let prelude = parse_container_prelude(self.source, input, &self.recovery)
                    .map_err(with_container_prelude_context)?;
                if !input.is_exhausted() {
                    return Err(with_at_rule_prelude_context(
                        invalid_syntax(input.current_source_location()),
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
                parse_scope_prelude(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                    if self.has_style_ancestor {
                        selectors::SelectorAnchorMode::Nesting
                    } else {
                        selectors::SelectorAnchorMode::Scope
                    },
                    true,
                ).map_err(with_scope_prelude_context)?,
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
            "font-palette-values" => {
                if self.has_style_ancestor {
                    return Err(invalid_at_rule_placement(input.current_source_location(), "font-palette-values", "a rule list without a style-rule ancestor"));
                }
                Ok(ScopedAtRulePrelude::FontPaletteValues(font_palette_values::parse_name(self.source, input, &self.recovery)?))
            },
            "color-profile" => {
                if self.has_style_ancestor {
                    return Err(invalid_at_rule_placement(input.current_source_location(), "color-profile", "a rule list without a style-rule ancestor"));
                }
                Ok(ScopedAtRulePrelude::ColorProfile(color_profile::parse_name(self.source, input, &self.recovery)?))
            },
            "font-face" => {
                if self.has_style_ancestor {
                    return Err(invalid_at_rule_placement(input.current_source_location(), "font-face", "a rule list without a style-rule ancestor"));
                }
                parse_font_face_prelude(input)?;
                Ok(ScopedAtRulePrelude::FontFace)
            },
            "keyframes" => {
                if self.has_style_ancestor {
                    return Err(invalid_at_rule_placement(input.current_source_location(), "keyframes", "a rule list without a style-rule ancestor"));
                }
                let name = parse_keyframes_prelude(input)?;
                Ok(ScopedAtRulePrelude::Keyframes(name))
            },
            "namespace" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "namespace",
                "the stylesheet top level",
            )),
            "counter-style" => {
                if self.has_style_ancestor {
                    return Err(invalid_at_rule_placement(input.current_source_location(), "counter-style", "a rule list without a style-rule ancestor"));
                }
                let name = parse_counter_style_prelude(self.source, input, self.recovery.source_snapshot())?;
                Ok(ScopedAtRulePrelude::CounterStyle(name))
            },
            "page" => {
                if self.has_style_ancestor || matches!(self.body, ScopedBodyKind::Scope) {
                    return Err(invalid_at_rule_placement(input.current_source_location(), "page", "an ordinary group body without a style-rule ancestor"));
                }
                Ok(ScopedAtRulePrelude::Page(parse_page_prelude(self.source, input, self.recovery.source_snapshot())?))
            },
            _ => Err(input.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name))),
        }
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> std::result::Result<Self::AtRule, ()> {
        let result = match prelude {
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
            ScopedAtRulePrelude::Layer(names) => CssLayerNameList::try_new(names)
                .map(|names| {
                    vec![CssScopedRule::LayerStatement(
                        CssScopedLayerStatementRule::new(
                            names,
                            self.recovery.source_position(start.position().byte_index()),
                        ),
                    )]
                })
                .ok_or(()),
            ScopedAtRulePrelude::Page(_)
            | ScopedAtRulePrelude::CounterStyle(_)
            | ScopedAtRulePrelude::FontFace
            | ScopedAtRulePrelude::Keyframes(_)
            | ScopedAtRulePrelude::FontFeatureValues(_)
            | ScopedAtRulePrelude::FontPaletteValues(_)
            | ScopedAtRulePrelude::ColorProfile(_)
            | ScopedAtRulePrelude::Media(_)
            | ScopedAtRulePrelude::Supports(_)
            | ScopedAtRulePrelude::SupportsCondition(_)
            | ScopedAtRulePrelude::Container(_)
            | ScopedAtRulePrelude::When(_, _)
            | ScopedAtRulePrelude::Else(_, _)
            | ScopedAtRulePrelude::Scope(_) => Err(()),
        };
        if result.is_err() {
            self.rejected_at_rule_start = Some(start.position().byte_index());
        } else {
            syntax_bridge::retain_statement_eof(
                self.source,
                &self.recovery,
                start,
                &mut self.diagnostics,
            );
        }
        result.map(ScopedBlockItem::Rules)
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
        let position = self.recovery.source_position(start.position().byte_index());
        let result = match prelude {
            ScopedAtRulePrelude::Page(selector) => {
                let rule = parse_page_rule(
                    self.source,
                    selector,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                Ok(vec![CssScopedRule::Page(rule)])
            }
            ScopedAtRulePrelude::CounterStyle(name) => {
                let rule = parse_counter_style_rule(
                    self.source,
                    name,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                Ok(vec![CssScopedRule::CounterStyle(rule)])
            }
            ScopedAtRulePrelude::FontFace => {
                let rule = parse_font_face_rule(
                    self.source,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                Ok(vec![CssScopedRule::FontFace(rule)])
            }
            ScopedAtRulePrelude::Keyframes(name) => {
                let rule = parse_keyframes_rule(
                    self.source,
                    name,
                    input,
                    start,
                    &mut self.diagnostics,
                    self.recovery.clone(),
                )?;
                Ok(vec![CssScopedRule::Keyframes(rule)])
            }

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
            ScopedAtRulePrelude::FontPaletteValues(name) => {
                self.parse_font_palette_block(name, input, start)
            }
            ScopedAtRulePrelude::ColorProfile(name) => {
                self.parse_color_profile_block(name, input, start)
            }
            ScopedAtRulePrelude::Media(query) => {
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    self.has_style_ancestor,
                    ScopedBodyKind::OrdinaryGroup,
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
                    ScopedBodyKind::OrdinaryGroup,
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                Ok(vec![CssScopedRule::Supports(CssScopedSupportsRule::new(
                    condition, rules, position,
                ))])
            }
            ScopedAtRulePrelude::SupportsCondition(prelude) => {
                let rule = named_supports::parse_rule(
                    self.source,
                    prelude,
                    start,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                )?;
                Ok(vec![CssScopedRule::SupportsCondition(rule)])
            }
            ScopedAtRulePrelude::When(condition, implicit) => {
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    self.has_style_ancestor,
                    ScopedBodyKind::OrdinaryGroup,
                )?;
                self.recovery.retain_component_closures(implicit);
                self.diagnostics.extend(recovered.diagnostics);
                Ok(vec![CssScopedRule::When(CssScopedWhenRule::new(
                    condition,
                    recovered.syntax,
                    position,
                ))])
            }
            ScopedAtRulePrelude::Else(condition, implicit) => {
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    self.has_style_ancestor,
                    ScopedBodyKind::OrdinaryGroup,
                )?;
                self.recovery.retain_component_closures(implicit);
                self.diagnostics.extend(recovered.diagnostics);
                Ok(vec![CssScopedRule::Else(CssScopedElseRule::new(
                    condition,
                    recovered.syntax,
                    position,
                ))])
            }
            ScopedAtRulePrelude::Container(prelude) => {
                let recovered = parse_scoped_rule_list(
                    self.source,
                    input,
                    self.recovery.clone(),
                    self.has_style_ancestor,
                    ScopedBodyKind::OrdinaryGroup,
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                Ok(vec![CssScopedRule::Container(CssScopedContainerRule::new(
                    prelude, rules, position,
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
                    ScopedBodyKind::OrdinaryGroup,
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
                    ScopedBodyKind::Scope,
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
        result.map(ScopedBlockItem::Rules)
    }
}

impl<'i> QualifiedRuleParser<'i> for ScopedRuleParser<'i> {
    type Prelude = CssScopedStyleSelectorList;
    type QualifiedRule = ScopedBlockItem;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        self.boundary = nesting::StyleRuleBoundary::QualifiedPrelude;
        let mut recovery =
            SelectorRecovery::new(self.source, &mut self.diagnostics, self.recovery.clone());
        let result = parse_scoped_style_selector_list(input, &mut recovery);
        if let Err(error) = &result
            && crate::error::is_nesting_limit_error(error)
        {
            self.qualified_resource_error = Some(error.clone());
        }
        result
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        self.boundary = nesting::StyleRuleBoundary::QualifiedBlock;
        let result = (|| {
            let mut depth =
                self.recovery
                    .enter_rule_block(self.source, input, "baseline.rule.style")?;
            let recovered = parse_style_contents(self.source, input, self.recovery.clone())?;
            self.diagnostics.extend(recovered.diagnostics);
            depth.retain();
            Ok(ScopedBlockItem::Rules(vec![CssScopedRule::Style(
                CssScopedStyleRule::new(
                    selectors,
                    recovered.syntax.declarations,
                    recovered.syntax.rules,
                    self.recovery.source_position(start.position().byte_index()),
                ),
            )]))
        })();
        if let Err(error) = &result
            && crate::error::is_nesting_limit_error(error)
        {
            self.qualified_resource_error = Some(error.clone());
        }
        result
    }
}

impl<'i> DeclarationParser<'i> for ScopedRuleParser<'i> {
    type Declaration = ScopedBlockItem;
    type Error = Error;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> std::result::Result<Self::Declaration, ParseError<'i, Self::Error>> {
        StrictDeclarationParser::new(self.source, self.recovery.clone(), false)
            .parse_value(name, input, declaration_start)
            .map(Box::new)
            .map(ScopedBlockItem::Declaration)
    }
}

impl<'i> RuleBodyItemParser<'i, ScopedBlockItem, Error> for ScopedRuleParser<'i> {
    fn parse_declarations(&self) -> bool {
        self.has_style_ancestor
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
            self.recovery.parser_context(),
        )?;
        self.recovery.retain_component_closures(implicit_closures);
        self.recovery.retain_navigation_diagnostic(&parsed.body);
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
    components: CssComponentValues,
    name: CssParsedOrigin,
    value_origin: CssParsedOrigin,
    parser_context: crate::CssParserContext,
}

impl ParsedDeclaration {
    fn into_declaration(self) -> CssDeclaration {
        self.into_declaration_in_context(crate::syntax::DeclarationContext::Ordinary)
    }

    fn into_declaration_in_context(
        self,
        declaration_context: crate::syntax::DeclarationContext,
    ) -> CssDeclaration {
        CssDeclaration::new_parsed(
            self.parser_context,
            declaration_context,
            self.body,
            self.importance,
            self.components,
            self.name,
            self.value_origin,
        )
    }

    fn into_keyframe_declaration(self) -> CssKeyframeDeclaration {
        CssKeyframeDeclaration::from_parsed(self.into_declaration())
    }
}

enum DeclarationBoundaryContext<'a> {
    OrdinarySvgGlyph,
    KeyframeSvgGlyph,
    OrdinaryKnown(crate::CssKnownProperty),
    OrdinaryCustom(CssCustomPropertyName),
    KeyframeKnown(crate::CssKnownProperty),
    KeyframeCustom(CssCustomPropertyName),
    Descriptor {
        at_rule: &'a str,
        descriptor: &'a str,
    },
}

/// The single admission predicate for known properties in authored keyframes.
pub(crate) fn keyframe_property_admitted(property: CssKnownProperty) -> bool {
    !matches!(
        property,
        CssKnownProperty::AnimationName
            | CssKnownProperty::AnimationDuration
            | CssKnownProperty::AnimationDelay
            | CssKnownProperty::AnimationIterationCount
            | CssKnownProperty::AnimationDirection
            | CssKnownProperty::AnimationFillMode
            | CssKnownProperty::AnimationPlayState
            | CssKnownProperty::Animation
    )
}

pub(super) fn parse_declaration_core<'i, 't>(
    mode: DeclarationMode,
    name: CowRcStr<'i>,
    input: &mut Parser<'i, 't>,
    declaration_start: &ParserState,
    source_snapshot: &CssSourceSnapshot,
    parser_context: crate::CssParserContext,
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
                    parse_custom_property_value(
                        input,
                        &crate::numeric::NumericInputContext::parsed(source_snapshot),
                    )
                    .map_err(|error| with_property_context(error, name.as_ref()))
                })
            })?;
        (
            CssDeclarationBody::Custom(CssCustomDeclaration::new(custom_name, value)),
            importance,
            components,
            origin,
        )
    } else if parser_context.selects_svg_glyph(name.as_ref()) {
        let context = match mode {
            DeclarationMode::Ordinary => DeclarationBoundaryContext::OrdinarySvgGlyph,
            DeclarationMode::Keyframe => DeclarationBoundaryContext::KeyframeSvgGlyph,
        };
        let ((body, components, origin), importance) =
            parse_declaration_boundary(input, &context, |input| {
                collect_declaration_value(input, source_snapshot, |input| {
                    parse_svg_glyph_declaration_body(
                        input,
                        &crate::numeric::NumericInputContext::parsed(source_snapshot),
                        crate::svg_glyph::SvgGlyphAdmission::css(parser_context),
                    )
                })
            })?;
        (body, importance, components, origin)
    } else {
        let resolved_property = resolve_property_name(name.as_ref()).ok_or_else(|| {
            property_name_error(declaration_start.source_location(), name.as_ref())
        })?;
        let known_property = resolved_property.property();
        if matches!(mode, DeclarationMode::Keyframe) && !keyframe_property_admitted(known_property)
        {
            return Err(with_property_context(
                crate::error::unexpected_at(declaration_start.source_location()),
                known_property.canonical_name(),
            ));
        }
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
                        parser_context,
                    )
                })
            })?;
        (body, importance, components, origin)
    };
    Ok(ParsedDeclaration {
        body,
        importance,
        components,
        name: name_origin,
        value_origin,
        parser_context,
    })
}

pub(super) fn collect_declaration_value<'i, 't, T>(
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
    parser_context: crate::CssParserContext,
) -> std::result::Result<CssDeclarationBody, Error> {
    let grammar = match property {
        CssPropertyNameRef::Known(property) => PropertyValueGrammar::Known(property.grammar()),
        CssPropertyNameRef::Custom(name) => PropertyValueGrammar::Custom(name),
        CssPropertyNameRef::SvgGlyphOrientationVertical => {
            PropertyValueGrammar::SvgGlyph(crate::svg_glyph::SvgGlyphAdmission::css(parser_context))
        }
    };
    parse_property_value_body_selected(grammar, source, numeric, parser_context)
}

pub(crate) fn parse_property_value_body_for_grammar(
    grammar: CssPropertyGrammar,
    source: &str,
    numeric: &crate::numeric::NumericInputContext<'_>,
    parser_context: crate::CssParserContext,
) -> std::result::Result<CssDeclarationBody, Error> {
    parse_property_value_body_selected(
        PropertyValueGrammar::Known(grammar),
        source,
        numeric,
        parser_context,
    )
}

#[derive(Clone, Copy)]
enum PropertyValueGrammar<'a> {
    Known(CssPropertyGrammar),
    Custom(&'a CssCustomPropertyName),
    SvgGlyph(crate::svg_glyph::SvgGlyphAdmission),
}

fn parse_property_value_body_selected(
    grammar: PropertyValueGrammar<'_>,
    source: &str,
    numeric: &crate::numeric::NumericInputContext<'_>,
    parser_context: crate::CssParserContext,
) -> std::result::Result<CssDeclarationBody, Error> {
    fragments::bounded_execution(source, || {
        let working_source = crate::tokenization::prepare(source);
        let mut input = ParserInput::new(&working_source);
        let mut parser = Parser::new(&mut input);
        parse_property_value_from_parser(grammar, source, &mut parser, numeric, parser_context)
            .map_err(|error| from_parse_error(source, error))
    })
}

fn parse_property_value_from_parser<'i>(
    grammar: PropertyValueGrammar<'_>,
    source: &'i str,
    parser: &mut Parser<'i, '_>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    parser_context: crate::CssParserContext,
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
            parse_known_declaration_body(grammar.resolved(), parser, numeric, parser_context)
        }
        PropertyValueGrammar::SvgGlyph(admission) => {
            parse_svg_glyph_declaration_body(parser, numeric, admission)
        }
        PropertyValueGrammar::Custom(name) => {
            parse_custom_property_value(parser, numeric).map(|value| {
                CssDeclarationBody::Custom(CssCustomDeclaration::new(name.clone(), value))
            })
        }
    }?;
    parser.expect_exhausted()?;
    Ok(body)
}

fn parse_known_declaration_body<'i, 't>(
    resolved_property: CssResolvedPropertyName,
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    parser_context: crate::CssParserContext,
) -> std::result::Result<CssDeclarationBody, ParseError<'i, Error>> {
    let known_property = resolved_property.property();
    let context_name = known_property.canonical_name();
    let state = input.state();
    let (authored, has_substitution) = collect_authored_declaration_value(
        input,
        numeric,
        variables::SubstitutionContext::KnownProperty,
    )
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
                    invalid_syntax(input.current_source_location()),
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

    let length_scope = quirky_length::property_context(known_property, parser_context, numeric);
    let numeric = &length_scope;
    let declaration = match resolved_property {
        CssResolvedPropertyName::Canonical(_) => quirky_color::parse_property_value(
            known_property,
            authored,
            input,
            numeric,
            parser_context,
        ),
        CssResolvedPropertyName::LegacyShorthand(alias) => {
            parse_legacy_property_alias_value(alias, authored, input, numeric)
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

fn parse_svg_glyph_declaration_body<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    admission: crate::svg_glyph::SvgGlyphAdmission,
) -> Result<CssDeclarationBody, ParseError<'i, Error>> {
    let name = "glyph-orientation-vertical";
    let parse = |input: &mut Parser<'i, 't>| {
        let state = input.state();
        let (authored, substitution) = collect_authored_declaration_value(
            input,
            numeric,
            variables::SubstitutionContext::KnownProperty,
        )?;
        let declared = if substitution {
            CssDeclaredValue::SubstitutionDependent(CssSubstitutionDependentValue::new(authored))
        } else {
            input.reset(&state);
            let first_state = loop {
                let candidate = input.state();
                match input.next_including_whitespace_and_comments() {
                    Ok(Token::WhiteSpace(_) | Token::Comment(_)) => {}
                    _ => break candidate,
                }
            };
            input.reset(&first_state);
            let first_location = input.current_source_location();
            if let Ok(ident) = input.try_parse(|input| input.expect_ident_cloned()) {
                if let Some(global) = parse_global_keyword(&ident) {
                    input.expect_exhausted()?;
                    CssDeclaredValue::Global(global)
                } else if ident.eq_ignore_ascii_case("auto") {
                    input.expect_exhausted()?;
                    CssDeclaredValue::Value(crate::CssSvgGlyphOrientationVerticalValue::auto())
                } else {
                    return Err(
                        first_location.new_unexpected_token_error::<Error>(Token::Ident(ident))
                    );
                }
            } else {
                input.reset(&first_state);
                input.skip_whitespace();
                let location = input.current_source_location();
                let offset = input.position().byte_index();
                // SVG has no canonical-property error target. Retain the real
                // root cursor/token for a present-token grammar rejection.
                let literal_start = input.state();
                let root_token = input.next().ok().cloned();
                input.reset(&literal_start);
                let component = numeric
                    .collect(input)
                    .map_err(|error| values::angle_error(numeric, &error, location, offset))?;
                let root_origin = component.origin().clone();
                let value = if matches!(
                    component.view(),
                    crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Number(_))
                ) {
                    crate::CssSvgGlyphOrientationVerticalValue::try_from_unitless(component.clone())
                } else {
                    crate::CssAngleValue::from_parser_component(component.clone(), numeric)
                        .and_then(crate::CssSvgGlyphOrientationVerticalValue::from_parser_angle)
                }
                .map_err(|error| {
                    let mapped = values::angle_error(numeric, &error, location, offset);
                    if let Some(token) = &root_token
                        && !crate::error::is_resource_parse_error(&mapped)
                        && (matches!(token, Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. })
                            || (matches!(error.kind(), crate::CssNumericConstructionErrorKind::Component(crate::CssComponentValueErrorKind::InvalidToken))
                                && error.origin() == Some(&root_origin))
                            || (matches!(token, Token::Function(_) | Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock)
                                && matches!(error.kind(), crate::CssNumericConstructionErrorKind::RootDomainMismatch)
                                && error.path() == Some(&[0][..])
                                && error.origin() == Some(&root_origin)))
                    {
                        location.new_unexpected_token_error::<Error>(token.clone())
                    } else {
                        if matches!(error.kind(), crate::CssNumericConstructionErrorKind::UnknownFunction | crate::CssNumericConstructionErrorKind::IncompatibleTypes | crate::CssNumericConstructionErrorKind::InvalidArgumentType | crate::CssNumericConstructionErrorKind::MalformedExpression)
                            && error.path().is_some_and(|path| !path.is_empty())
                            && error.origin().is_some()
                        {
                            // The numeric owner identifies the responsible child;
                            // use its mapped cursor rather than the outer function.
                            let source = match numeric.ordinary() {
                                crate::numeric::NumericInputContext::Parsed(source) => source.as_str(),
                                crate::numeric::NumericInputContext::Components(_, serialized) => serialized.as_css(),
                                crate::numeric::NumericInputContext::QuirkyLengths(_) => unreachable!("ordinary numeric provenance"),
                            };
                            let responsible = crate::CssSourcePosition::from_source_location_in(source, mapped.location).byte_offset().value();
                            if let Some((start, _, token)) = crate::tokenization::next_source_token(source, responsible)
                                && !matches!(token, Token::WhiteSpace(_) | Token::Comment(_))
                                // Distinguish an actual residual operand from the
                                // native cursor's EOF/operator-spacing fallback.
                                && (!matches!(error.kind(), crate::CssNumericConstructionErrorKind::MalformedExpression)
                                    || svg_glyph_numeric_present_rejection(&component, &error))
                            {
                                return cssparser::ParseError {
                                    kind: cssparser::ParseErrorKind::Custom(crate::error::unexpected_token_at(source, start, &token)),
                                    location: mapped.location,
                                };
                            }
                        }
                        mapped
                    }
                })?;
                if !value.admitted(admission) {
                    return Err(location.new_unexpected_token_error::<Error>(
                        root_token.expect("only a direct Number has mode-dependent admission"),
                    ));
                }
                input.expect_exhausted()?;
                CssDeclaredValue::Value(value)
            }
        };
        Ok(CssDeclarationBody::SvgGlyphOrientationVertical(
            crate::CssSvgGlyphOrientationVerticalDeclaration::new(declared, admission),
        ))
    };
    parse(input).map_err(|error| with_property_context(error, name))
}

fn svg_glyph_numeric_present_rejection(
    root: &crate::CssComponentValue,
    error: &crate::CssNumericConstructionError,
) -> bool {
    use crate::{CssComponentValueRef as Component, CssValueTokenRef as Value};
    let operand = |component: &crate::CssComponentValue| match component.view() {
        Component::Token(
            Value::Number(_) | Value::Dimension { .. } | Value::Percentage(_) | Value::Ident(_),
        )
        | Component::Function(_) => true,
        Component::Block(block) => block.kind() == crate::CssBlockKind::Parenthesis,
        _ => false,
    };
    let Some(path) = error.path() else {
        return false;
    };
    if path.first() != Some(&0) {
        return false;
    }
    let Some((&index, parent_path)) = path[1..].split_last() else {
        return false;
    };
    let mut parent = root;
    for &index in parent_path {
        let children = match parent.view() {
            Component::Function(function) => function.values(),
            Component::Block(block) => block.values(),
            _ => return false,
        };
        let Some(child) = children.items().get(index) else {
            return false;
        };
        parent = child;
    }
    let children = match parent.view() {
        Component::Function(function) => function.values(),
        Component::Block(block) => block.values(),
        _ => return false,
    };
    let Some(current) = children.items().get(index) else {
        return false;
    };
    if error.origin() != Some(current.origin()) {
        return false;
    }
    let previous = children.items()[..index].iter().rev().find(|component| {
        !matches!(
            component.view(),
            Component::Comment(_) | Component::Token(Value::Whitespace(_))
        )
    });
    if operand(current) {
        // A completed preceding operand proves sequence() left this real value.
        // An operator predecessor instead identifies a spacing error.
        return previous.is_some_and(operand);
    }
    match current.view() {
        Component::Comment(_) | Component::Token(Value::Whitespace(_)) => false,
        Component::Token(Value::Delim('+' | '-' | '*' | '/')) => {
            // At a segment's first operand parse_node rejects this actual token.
            // Elsewhere it can be an operator-spacing or EOF fallback cursor.
            previous.is_none()
                || previous.is_some_and(|component| {
                    matches!(component.view(), Component::Token(Value::Comma))
                })
        }
        // These component classes are rejected by parse_node itself. They cannot
        // be the last-operator fallback for an absent operand.
        _ => true,
    }
}

pub(crate) fn parse_svg_glyph_value_body(
    admission: crate::svg_glyph::SvgGlyphAdmission,
    source: &str,
    numeric: &crate::numeric::NumericInputContext<'_>,
    context: crate::CssParserContext,
) -> Result<CssDeclarationBody, Error> {
    parse_property_value_body_selected(
        PropertyValueGrammar::SvgGlyph(admission),
        source,
        numeric,
        context,
    )
}

fn parse_legacy_property_alias_value<'i, 't>(
    alias: CssLegacyPropertyAlias,
    authored: CssAuthoredDeclarationValue,
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssKnownDeclaration, ParseError<'i, Error>> {
    match alias {
        CssLegacyPropertyAlias::GlyphOrientationVertical => {
            let value = parse_glyph_orientation_vertical(input, numeric)?;
            Ok(CssKnownDeclaration::from_value(
                CssKnownDeclarationValue::TextOrientation(CssDeclaredValue::Value(
                    CssTextOrientationPropertyValue::new(authored, value),
                )),
            ))
        }
        CssLegacyPropertyAlias::PageBreakBefore => {
            let value = parse_page_break_between(input)?;
            Ok(CssKnownDeclaration::from_value(
                CssKnownDeclarationValue::BreakBefore(CssDeclaredValue::Value(
                    CssBreakBeforePropertyValue::new(authored, value),
                )),
            ))
        }
        CssLegacyPropertyAlias::PageBreakAfter => {
            let value = parse_page_break_between(input)?;
            Ok(CssKnownDeclaration::from_value(
                CssKnownDeclarationValue::BreakAfter(CssDeclaredValue::Value(
                    CssBreakAfterPropertyValue::new(authored, value),
                )),
            ))
        }
        CssLegacyPropertyAlias::PageBreakInside => {
            let value = parse_page_break_inside(input)?;
            Ok(CssKnownDeclaration::from_value(
                CssKnownDeclarationValue::BreakInside(CssDeclaredValue::Value(
                    CssBreakInsidePropertyValue::new(authored, value),
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
            | DeclarationBoundaryContext::OrdinarySvgGlyph
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
        DeclarationBoundaryContext::OrdinarySvgGlyph
        | DeclarationBoundaryContext::KeyframeSvgGlyph => {
            crate::error::invalid_svg_glyph_declaration_annotation(
                location,
                matches!(context, DeclarationBoundaryContext::KeyframeSvgGlyph),
            )
        }
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
            invalid_syntax(location),
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
                .ok_or_else(|| invalid_syntax(p.current_source_location()))?;
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
    let working_source = crate::tokenization::prepare(source);
    let mut token_input = ParserInput::new(&working_source);
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
