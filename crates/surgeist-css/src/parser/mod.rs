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
pub(crate) mod bounded_execution;
mod box_model;
mod box_spacing;
pub(crate) mod color_profile;
mod combinators;
mod contain_intrinsic_size;
pub(crate) mod counter_style;
mod declaration_block;
pub(crate) use declaration_block::parse_snapshot as parse_declaration_block_contents_snapshot;
mod declaration_list;
mod domain_declaration_contents;
pub(crate) mod property_input;
pub(crate) use declaration_block::parse_with_context as parse_declaration_block_contents_with_context;
pub use declaration_block::{
    parse_declaration_block_contents, parse_declaration_block_contents_with_limits,
};
pub(crate) use domain_declaration_contents::parse_page_with_context as parse_page_declaration_block_contents_with_context;
pub use domain_declaration_contents::{
    parse_font_face_declaration_block_contents,
    parse_font_face_declaration_block_contents_with_limits, parse_page_declaration_block_contents,
    parse_page_declaration_block_contents_with_limits,
};
mod descriptor_body;
mod descriptor_values;
mod effects;
mod flex;
mod font_controls;
pub(crate) mod font_face;
pub(crate) mod font_feature_values;
pub(crate) mod font_feature_values_family_list;
pub use font_feature_values_family_list::{
    parse_font_feature_values_family_list, parse_font_feature_values_family_list_with_limits,
};
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
    BoundedParseContext, BoundedParseSyntax, parse_bounded, parse_sheet_bounded, rule_start,
    scoped_rule_into_chunk_rule, scoped_rule_start,
};
#[cfg(test)]
mod native_batch_boundary_tests;
mod page_margin_rule;
#[cfg(test)]
mod preparation_growth_tests;
#[cfg(test)]
mod replay_growth_tests;
mod rule_candidate;
mod rule_list;
pub(crate) use page_margin_rule::parse_with_context as parse_page_margin_rule_with_context;
pub use page_margin_rule::{parse_page_margin_rule, parse_page_margin_rule_with_limits};
use rule_list::{
    AtRulePrelude, GroupSink, RuleListContext, RuleListGrammar, RuleListItem, RuleListUnits,
    RuleParser, ScopedSink, SheetSink, UnitState, parse_at_rule_prelude, recover_rule_list,
};
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
    parse_margin_block_with_context, parse_media_query_list_snapshot,
    parse_page_block_with_context, parse_property_value_text_for_grammar_with_context,
    parse_property_value_text_with_context, parse_rule_with_context,
    parse_scope_block_with_context, parse_scoped_group_block_with_context,
    parse_style_block_with_context,
};
mod boolean_condition;
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
pub use named_supports::{
    parse_supports_test_child, parse_supports_test_child_with_limits, parse_supports_test_contents,
    parse_supports_test_contents_with_limits,
};
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
    CssError, CssFeatureId, basic, from_parse_error, from_rule_parse_error, invalid_at_rule_block,
    invalid_at_rule_body, invalid_at_rule_placement, invalid_custom_declaration_annotation,
    invalid_descriptor_annotation, invalid_known_declaration_annotation, invalid_syntax,
    property_name_error, with_at_rule_prelude_context, with_media_query_context,
    with_property_context,
};
use crate::properties::*;
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
use crate::syntax::*;
use crate::syntax_consumption::RuleListSyntax;
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
            $input: &mut property_input::Parser<'i, 't>,
            $numeric: &crate::numeric::NumericInputContext<'_>,
        ) -> std::result::Result<CssKnownDeclaration, crate::error::property::ParseError<'i, crate::error::property::Error>> {
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
                        $input: &mut property_input::Parser<'i, 't>,
                        $numeric: &crate::numeric::NumericInputContext<'_>,
                    ) -> std::result::Result<CssKnownDeclaration, crate::error::property::ParseError<'i, crate::error::property::Error>> {
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
    input: &mut property_input::Parser<'i, 't>,
) -> std::result::Result<
    CssGlobalKeyword,
    crate::error::property::ParseError<'i, crate::error::property::Error>,
> {
    Err(crate::error::property::unexpected_at(
        input.current_source_location(),
    ))
}

/// Parses a UTF-8 stylesheet into valid authored syntax and recovery diagnostics.
///
/// The ordinary parser retains valid top-level rules in source order and reports
/// each discarded top-level rule with its complete balanced source span. Parsed
/// `@charset` rules are unrecognized and dropped; top-level CDO/CDC tokens are
/// ignored without diagnostics. The UTF-8 input is not decoded. The stylesheet
/// root is structural depth zero; up to 256 shared rule-block/component/function
/// levels are retained, and the first level
/// beyond that drops its smallest enclosing recovery unit. Input deeper than 128
/// levels is parsed on a bounded larger-stack thread; if the platform cannot
/// start it, the report retains no rules and carries a
/// [`CssErrorCode::ParserUnavailable`](crate::CssErrorCode::ParserUnavailable)
/// refusal (limit 128, action `StopAtNestingLimit`), followed only by any tokenizer EOF
/// or escape diagnostics, rather than recursing on the caller's stack. Recovery does not
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
    let (mut sheet, mut diagnostics) =
        parse_sheet_snapshot_inner(source_snapshot, parser_context).into_parts();
    sheet.bind_parse_source(crate::CssParseSource::new(
        source_snapshot.clone(),
        parser_context,
    ));
    for diagnostic in &mut diagnostics {
        diagnostic.bind_source(source_snapshot);
    }
    crate::CssParseReport::new(sheet, diagnostics)
}

fn parse_sheet_snapshot_inner(
    source_snapshot: &CssSourceSnapshot,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<CssSheet> {
    let source = source_snapshot.as_str();
    // The 256-level public limit exceeds small default thread stacks; deep
    // input runs on the bounded parser thread or is refused, never deep here.
    let profile = bounded_execution::StackProfile::Structural;
    let report = bounded_execution::report(profile, source, CssSheet::new, || {
        parse_sheet_bounded(
            source,
            source_snapshot,
            0,
            BoundedParseContext::Rules { top_level: true },
            StyleContextCaptures::default(),
            parser_context,
        )
    });
    recovery::finish_report(source, conditional_chains::sheet(source, report))
}

/// Parses a UTF-8 style attribute into valid ordinary declarations and recovery diagnostics.
///
/// The parser accepts an empty declaration list and an optional final semicolon. Each invalid
/// declaration candidate is discarded independently, so later valid declarations remain
/// eligible. A unit that starts with neither an identifier nor an at-keyword, including a stray
/// `}`, is one malformed declaration discarded through the next root semicolon; so is an
/// identifier-started unit that is not a valid declaration, such as `a {} color: red`.
/// Retained declarations use the same property, custom-property, substitution, and
/// importance grammar as declarations in ordinary style-rule blocks. At-rules, qualified rules,
/// and other non-declaration input never produce a rule node from this front door.
/// Input deeper than 128 levels needs a bounded larger-stack thread; if the
/// platform cannot start it, the report retains no declarations and carries a
/// [`CssErrorCode::ParserUnavailable`](crate::CssErrorCode::ParserUnavailable)
/// refusal (limit 128), followed only by any tokenizer EOF or escape
/// diagnostics, instead of recursing on the caller's stack.
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
    let profile = bounded_execution::StackProfile::Structural;
    let refused = || CssDeclarationList::new(Vec::new());
    let report = bounded_execution::report(profile, source, refused, || {
        parse_style_attribute_inner(source, parser_context)
    });
    recovery::finish_report(source, report)
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

// Detached style and scoped contexts replay one rule list over the caller's
// source; failing to enter that list rejects the whole source.
fn parse_context_inner(
    source: &str,
    recovery: RecoveryState,
    parse: impl for<'i, 't> FnOnce(
        &'i str,
        &mut Parser<'i, 't>,
        RecoveryState,
    ) -> std::result::Result<
        Recovered<Vec<CssRule>>,
        ParseError<'i, CssError>,
    >,
) -> crate::CssParseReport<CssSheet> {
    let working_source = crate::tokenization::prepare(source);
    let mut input = ParserInput::new(&working_source);
    let mut parser = Parser::new(&mut input);
    match parse(source, &mut parser, recovery.clone()) {
        Ok(recovered) => {
            let mut sheet = CssSheet::new();
            for rule in recovered.syntax {
                sheet.push_rule(rule);
            }
            let mut diagnostics = recovered.diagnostics;
            diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(sheet, diagnostics)
        }
        Err(error) => crate::CssParseReport::new(
            CssSheet::new(),
            vec![fragments::reject(
                source,
                error,
                crate::CssRecoveryAction::DropQualifiedRule,
            )],
        ),
    }
}

fn parse_style_context_inner(
    source: &str,
    recovery: RecoveryState,
) -> crate::CssParseReport<CssSheet> {
    parse_context_inner(source, recovery, |source, input, recovery| {
        let recovered = parse_style_contents(source, input, recovery)?;
        Ok(Recovered {
            syntax: recovered.syntax.into_nested_rules(),
            diagnostics: recovered.diagnostics,
        })
    })
}

fn parse_scoped_context_inner(
    source: &str,
    recovery: RecoveryState,
    has_style_ancestor: bool,
    body: ScopedBodyKind,
) -> crate::CssParseReport<CssSheet> {
    parse_context_inner(source, recovery, |source, input, recovery| {
        let recovered = parse_scoped_rule_list(source, input, recovery, has_style_ancestor, body)?;
        Ok(Recovered {
            syntax: recovered
                .syntax
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            diagnostics: recovered.diagnostics,
        })
    })
}

fn parse_sheet_inner(
    source: &str,
    recovery: RecoveryState,
    top_level: bool,
) -> crate::CssParseReport<CssSheet> {
    let working_source = crate::tokenization::prepare(source);
    let mut input = ParserInput::new(&working_source);
    let mut parser = Parser::new(&mut input);
    // A non-top-level sheet is the isolated replay of a unit from a group
    // block, whose list is consumed as that block's contents.
    let (mut grammar, list) = if top_level {
        (
            RuleParser::top_level(source, recovery.clone()),
            RuleListSyntax::Stylesheet,
        )
    } else {
        (
            RuleParser::group(source, recovery.clone()),
            RuleListSyntax::BlockContents,
        )
    };
    let units = match RuleListUnits::select(source, &parser, &recovery, list) {
        Ok(units) => units,
        Err(error) => {
            return crate::CssParseReport::new(
                CssSheet::new(),
                vec![syntax_bridge::arena_error(source, error)],
            );
        }
    };
    let mut sink = SheetSink(CssSheet::new());
    recover_rule_list(source, &mut parser, &mut grammar, &mut sink, &units);
    let mut diagnostics = grammar.diagnostics;
    diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
    crate::CssParseReport::new(sink.0, diagnostics)
}

pub(super) struct Recovered<T> {
    pub(super) syntax: T,
    pub(super) diagnostics: Vec<crate::CssRecoveryDiagnostic>,
}

pub(super) fn block_item_diagnostic(
    source: &str,
    error: ParseError<'_, CssError>,
    failed_unit: &str,
    unit_end: usize,
    action: crate::CssRecoveryAction,
) -> Option<crate::CssRecoveryDiagnostic> {
    let unit_start = unit_end.saturating_sub(failed_unit.len());
    block_item_diagnostic_from_start(source, error, unit_start, unit_end, action)
}

pub(super) fn block_item_diagnostic_from_start(
    source: &str,
    error: ParseError<'_, CssError>,
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

/// Syntax 3 (2026 CRD) §5.4.10 appends a group only while its input is not
/// empty: a discarded comma followed by no token ends the list. Whitespace is
/// a token; a comment is not.
/// <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#parse-comma-separated-list-of-component-values>
pub(super) fn at_final_comma(input: &mut Parser<'_, '_>) -> bool {
    let state = input.state();
    let ends = input.next_including_whitespace().is_err();
    input.reset(&state);
    ends
}

pub(super) fn consume_failed_rule_block<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    failed: bool,
    recovery: &RecoveryState,
    enclosing_production: &'static str,
) -> (bool, Option<ParseError<'i, CssError>>) {
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

/// A dropped unit's diagnostic, spanning exactly that unit.
pub(super) fn structural_rule_diagnostic(
    source: &str,
    error: ParseError<'_, CssError>,
    failed_unit: &str,
    unit_start: usize,
    unit_end: usize,
    action: crate::CssRecoveryAction,
) -> Option<crate::CssRecoveryDiagnostic> {
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
) -> ParseError<'i, CssError> {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TopLevelPreludePhase {
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

struct CssNamespacePrelude {
    prefix: Option<CssNamespacePrefix>,
    name: CssNamespaceName,
}

struct CssScopePrelude {
    root: Option<CssScopeSelectorList>,
    limit: Option<CssScopeSelectorList>,
}

// Component construction uses the same finite selector as parsing. Transport
// models only classify clauses; only original components back the returned rule.
fn parse_container_prelude<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    recovery: &RecoveryState,
) -> std::result::Result<CssContainerPrelude, ParseError<'i, CssError>> {
    let (values, implicit) = collect_container_components(source, input, recovery)?;
    let prelude = container_prelude_from_components(values, input.current_source_location())?;
    recovery.retain_component_closures(implicit);
    Ok(prelude)
}

fn with_container_prelude_context<'i>(error: ParseError<'i, CssError>) -> ParseError<'i, CssError> {
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
) -> std::result::Result<Recovered<Vec<CssRule>>, ParseError<'i, CssError>> {
    let units = RuleListUnits::select_block_contents(source, input, &recovery)?;
    let mut grammar = RuleParser::group(source, recovery);
    let mut sink = GroupSink(Vec::new());
    recover_rule_list(source, input, &mut grammar, &mut sink, &units);
    Ok(Recovered {
        syntax: sink.0,
        diagnostics: grammar.diagnostics,
    })
}

fn parse_page_prelude<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    snapshot: &CssSourceSnapshot,
) -> std::result::Result<CssPageSelectorList, ParseError<'i, CssError>> {
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
) -> std::result::Result<Recovered<CssScopedRuleList>, ParseError<'i, CssError>> {
    if has_style_ancestor {
        recovery.record_style_context(input.position().byte_index());
    }
    let mut grammar = ScopedRuleParser::new(source, recovery.clone(), has_style_ancestor, body);
    // Without a style ancestor neither body admits declarations, so its rule
    // grammar selects consume a block's contents units; with one, the native
    // body parser tries each declaration before a nested rule.
    let units = if has_style_ancestor {
        RuleListUnits::Declarations
    } else {
        RuleListUnits::select_block_contents(source, input, &recovery)?
    };
    let mut sink = ScopedSink::new();
    recover_rule_list(source, input, &mut grammar, &mut sink, &units);
    Ok(Recovered {
        syntax: sink.finish(),
        diagnostics: grammar.diagnostics,
    })
}

fn parse_namespace_prelude<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssNamespacePrelude, ParseError<'i, CssError>> {
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
) -> std::result::Result<Vec<CssLayerName>, ParseError<'i, CssError>> {
    if input.is_exhausted() {
        return Ok(Vec::new());
    }

    // `<layer-name>#`. List post-condition: a remainder after a name is an
    // at-rule syntax error, and a trailing comma reports the missing name.
    combinators::parse_native_comma_list(
        input,
        |input| {
            let name = parse_layer_name(input)?;
            if !input.is_exhausted() {
                return Err(invalid_syntax(input.current_source_location()));
            }
            Ok(name)
        },
        |input, _| {
            parse_layer_name(input)
                .err()
                .unwrap_or_else(|| invalid_syntax(input.current_source_location()))
        },
    )
}

fn parse_layer_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLayerName, ParseError<'i, CssError>> {
    let mut locations = vec![input.current_source_location()];
    let mut components = vec![input.expect_ident_cloned().map_err(basic)?.to_string()];

    loop {
        let boundary = input.state();
        if !matches!(input.next_including_whitespace(), Ok(Token::Delim('.'))) {
            input.reset(&boundary);
            break;
        }

        let component_location = input.current_source_location();
        match input.next_including_whitespace().map_err(basic)? {
            Token::Ident(component) => {
                components.push(component.to_string());
                locations.push(component_location);
            }
            token => {
                return Err(component_location.new_unexpected_token_error(token.clone()));
            }
        }
    }

    // Cascade 5 §6.4.2: the first component that is no layer-name <ident>
    // (a CSS-wide keyword) is responsible.
    let responsible = components
        .iter()
        .position(|component| CssLayerName::try_new([component.as_str()]).is_none())
        .map_or(locations[0], |index| locations[index]);
    CssLayerName::try_new(components).ok_or_else(|| invalid_syntax(responsible))
}

fn parse_scope_prelude<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    state: &RecoveryState,
    root_anchors: selectors::SelectorAnchorMode,
    relative_root: bool,
) -> std::result::Result<CssScopePrelude, ParseError<'i, CssError>> {
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

fn with_scope_prelude_context(error: ParseError<'_, CssError>) -> ParseError<'_, CssError> {
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
) -> Result<CounterStylePrelude, ParseError<'i, CssError>> {
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
) -> Result<(), ParseError<'i, CssError>> {
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
) -> Result<CssKeyframesName, ParseError<'i, CssError>> {
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
    has_style_ancestor: bool,
    body: ScopedBodyKind,
    source: &'s str,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
    unit: UnitState<'s>,
}

impl<'s> ScopedRuleParser<'s> {
    fn new(
        source: &'s str,
        recovery: RecoveryState,
        has_style_ancestor: bool,
        body: ScopedBodyKind,
    ) -> Self {
        Self {
            has_style_ancestor,
            body,
            source,
            diagnostics: Vec::new(),
            recovery,
            unit: UnitState::new(),
        }
    }

    // A group rule's body inside this scoped list.
    #[inline(never)]
    fn group_body<'t>(
        &mut self,
        input: &mut Parser<'s, 't>,
        body: ScopedBodyKind,
    ) -> Result<CssScopedRuleList, ParseError<'s, CssError>> {
        let recovered = parse_scoped_rule_list(
            self.source,
            input,
            self.recovery.clone(),
            self.has_style_ancestor,
            body,
        )?;
        self.diagnostics.extend(recovered.diagnostics);
        Ok(recovered.syntax)
    }
}

impl<'i> RuleListGrammar<'i, ScopedBlockItem> for ScopedRuleParser<'i> {
    fn unit(&mut self) -> &mut UnitState<'i> {
        &mut self.unit
    }
    fn recovery(&self) -> &RecoveryState {
        &self.recovery
    }
    fn diagnostics(&mut self) -> &mut Vec<crate::CssRecoveryDiagnostic> {
        &mut self.diagnostics
    }
}

impl<'i> AtRuleParser<'i> for ScopedRuleParser<'i> {
    type Prelude = AtRulePrelude;
    type AtRule = ScopedBlockItem;
    type Error = CssError;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        self.unit.boundary = nesting::StyleRuleBoundary::AtRule;
        parse_at_rule_prelude(
            RuleListContext::Scoped {
                body: self.body,
                style_ancestor: self.has_style_ancestor,
            },
            None,
            self.source,
            name,
            input,
            &mut self.diagnostics,
            &self.recovery,
        )
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> std::result::Result<Self::AtRule, ()> {
        let result = match prelude {
            AtRulePrelude::CustomMedia(prelude) => {
                let rule = finish_custom_media(
                    self.source,
                    *prelude,
                    start,
                    &self.recovery,
                    &mut self.diagnostics,
                );
                Ok(vec![CssScopedRule::CustomMedia(rule)])
            }
            AtRulePrelude::Layer(names) => CssLayerNameList::try_new(names)
                .map(|names| {
                    vec![CssScopedRule::LayerStatement(
                        CssScopedLayerStatementRule::new(
                            names,
                            self.recovery.source_position(start.position().byte_index()),
                        ),
                    )]
                })
                .ok_or(()),
            AtRulePrelude::Import(_)
            | AtRulePrelude::Namespace(_)
            | AtRulePrelude::Page(_)
            | AtRulePrelude::CounterStyle(_)
            | AtRulePrelude::FontFace
            | AtRulePrelude::Keyframes(_)
            | AtRulePrelude::FontFeatureValues(_)
            | AtRulePrelude::FontPaletteValues(_)
            | AtRulePrelude::ColorProfile(_)
            | AtRulePrelude::Media(_)
            | AtRulePrelude::Supports(_)
            | AtRulePrelude::SupportsCondition(_)
            | AtRulePrelude::Container(_)
            | AtRulePrelude::When(_, _)
            | AtRulePrelude::Else(_, _)
            | AtRulePrelude::Scope(_) => Err(()),
        };
        if result.is_err() {
            self.unit.rejected_at_rule_start = Some(start.position().byte_index());
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
        let prelude = match rule_list::parse_leaf_block(
            prelude,
            self.source,
            start,
            input,
            &mut self.diagnostics,
            &self.recovery,
        )? {
            Ok(leaf) => {
                depth.retain();
                return Ok(ScopedBlockItem::Rules(vec![leaf.into()]));
            }
            Err(group) => group,
        };
        let result = match prelude {
            AtRulePrelude::Media(query) => {
                let rules = self.group_body(input, ScopedBodyKind::OrdinaryGroup)?;
                Ok(vec![CssScopedRule::Media(CssScopedMediaRule::new(
                    query, rules, position,
                ))])
            }
            AtRulePrelude::Supports(condition) => {
                let rules = self.group_body(input, ScopedBodyKind::OrdinaryGroup)?;
                Ok(vec![CssScopedRule::Supports(CssScopedSupportsRule::new(
                    condition, rules, position,
                ))])
            }
            AtRulePrelude::When(condition, implicit) => {
                let rules = self.group_body(input, ScopedBodyKind::OrdinaryGroup)?;
                self.recovery.retain_component_closures(implicit);
                Ok(vec![CssScopedRule::When(CssScopedWhenRule::new(
                    condition, rules, position,
                ))])
            }
            AtRulePrelude::Else(condition, implicit) => {
                let rules = self.group_body(input, ScopedBodyKind::OrdinaryGroup)?;
                self.recovery.retain_component_closures(implicit);
                Ok(vec![CssScopedRule::Else(CssScopedElseRule::new(
                    condition, rules, position,
                ))])
            }
            AtRulePrelude::Container(prelude) => {
                let rules = self.group_body(input, ScopedBodyKind::OrdinaryGroup)?;
                Ok(vec![CssScopedRule::Container(CssScopedContainerRule::new(
                    prelude, rules, position,
                ))])
            }
            AtRulePrelude::Layer(names) => {
                if names.len() > 1 {
                    return Err(invalid_at_rule_block(
                        input,
                        "layer",
                        "baseline.rule.layer-block",
                        "at most one layer name before a block",
                    ));
                }
                let name = names.into_iter().next();
                let rules = self.group_body(input, ScopedBodyKind::OrdinaryGroup)?;
                Ok(vec![CssScopedRule::LayerBlock(
                    CssScopedLayerBlockRule::new(name, rules, position),
                )])
            }
            AtRulePrelude::Scope(prelude) => {
                let rules = self.group_body(input, ScopedBodyKind::Scope)?;
                Ok(vec![CssScopedRule::Scope(CssScopeRule::new(
                    prelude.root,
                    prelude.limit,
                    rules,
                    position,
                ))])
            }
            leaf => unreachable!("{} has a leaf body", leaf.name()),
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
    type Error = CssError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        self.unit.boundary = nesting::StyleRuleBoundary::QualifiedPrelude;
        let mut recovery =
            SelectorRecovery::new(self.source, &mut self.diagnostics, self.recovery.clone());
        let result = parse_scoped_style_selector_list(input, &mut recovery);
        self.unit.record_qualified_failure(&result, true);
        result
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        self.unit.boundary = nesting::StyleRuleBoundary::QualifiedBlock;
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
        self.unit.record_qualified_failure(&result, false);
        result
    }
}

impl<'i> DeclarationParser<'i> for ScopedRuleParser<'i> {
    type Declaration = ScopedBlockItem;
    type Error = CssError;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> std::result::Result<Self::Declaration, ParseError<'i, Self::Error>> {
        StrictDeclarationParser::new(self.source, self.recovery.clone())
            .parse_value(name, input, declaration_start)
            .map(Box::new)
            .map(ScopedBlockItem::Declaration)
    }
}

impl<'i> RuleBodyItemParser<'i, ScopedBlockItem, CssError> for ScopedRuleParser<'i> {
    fn parse_declarations(&self) -> bool {
        self.has_style_ancestor
    }
    fn parse_qualified(&self) -> bool {
        true
    }
}

// CSS Style Attributes §3 parses the value as declaration block contents in
// which a `}` "is merely an invalid token". A unit that starts with anything
// other than an identifier or at-keyword is a malformed declaration, whose
// remnants Syntax 3 (2026 CRD) §5.5.6 consumes through the next root `;`. The
// value admits declarations only, so an identifier-started unit that is not a
// valid declaration is likewise one malformed declaration read through its `;`
// (CSS 2.1 §4.2), never a qualified rule that ends at its `}`.
fn parse_style_attribute_declarations<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
) -> Recovered<CssDeclarationList> {
    let mut declarations = Vec::new();
    let mut diagnostics = Vec::new();
    let mut declaration_parser = StrictDeclarationParser::new(source, recovery.clone());
    loop {
        let progress = RecoveryProgress::record(input);
        let unit_start = declaration_list::skip_empty_declaration_slots(input);
        if declaration_list::starts_bad_declaration(input) {
            let end = match recovery.syntax_document(source) {
                Ok(document) => declaration_list::bad_declaration_end(&document, unit_start),
                Err(error) => {
                    diagnostics.push(syntax_bridge::arena_error(source, error));
                    break;
                }
            };
            diagnostics.extend(declaration_list::drop_bad_declaration(
                source, input, &recovery, unit_start, end,
            ));
            if progress.finish(input, false) == RecoveryLoopOutcome::Terminated {
                break;
            }
            continue;
        }
        let Some(item) = RuleBodyParser::new(input, &mut declaration_parser).next() else {
            break;
        };
        let (failed_at_block, failed_block_error) = item
            .as_ref()
            .err()
            .map(|(_, _)| {
                consume_failed_rule_block(source, input, true, &recovery, "css.declaration")
            })
            .unwrap_or((false, None));
        let retained = item.is_ok();
        let progress_outcome = progress.finish(input, retained);
        let unit_end = input.position().byte_index();
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
            Err((error, _)) => {
                let error = failed_block_error.unwrap_or(error);
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
}

impl<'s> StrictDeclarationParser<'s> {
    pub(super) fn new(source: &'s str, recovery: RecoveryState) -> Self {
        Self { source, recovery }
    }
}

impl<'i> AtRuleParser<'i> for StrictDeclarationParser<'i> {
    type Prelude = ();
    type AtRule = CssDeclaration;
    type Error = CssError;
}

impl<'i> QualifiedRuleParser<'i> for StrictDeclarationParser<'i> {
    type Prelude = ();
    type QualifiedRule = CssDeclaration;
    type Error = CssError;
}

impl<'i> RuleBodyItemParser<'i, CssDeclaration, CssError> for StrictDeclarationParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        false
    }
}

impl<'i> DeclarationParser<'i> for StrictDeclarationParser<'i> {
    type Declaration = CssDeclaration;
    type Error = CssError;

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
) -> std::result::Result<ParsedDeclaration, ParseError<'i, CssError>> {
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
                    parse_property_declaration_source(
                        PropertyValueGrammar::Custom(&custom_name),
                        input,
                        &crate::numeric::NumericInputContext::parsed(source_snapshot),
                        parser_context,
                    )
                    .map_err(|error| with_property_context(error, name.as_ref()))
                })
            })?;
        (value, importance, components, origin)
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
    parse_value: impl FnOnce(&mut Parser<'i, 't>) -> std::result::Result<T, ParseError<'i, CssError>>,
) -> std::result::Result<(T, CssComponentValues, CssParsedOrigin), ParseError<'i, CssError>> {
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

/// The original checked property pipeline under the producer's one live control.
/// The source belongs to the privately issued component graph, not caller input.
pub(crate) fn parse_color_scheme_value_controlled(
    components: &CssComponentValues,
    parser_context: crate::CssParserContext,
    source: &CssSourceSnapshot,
    control: &crate::utf16_value::Control<'_>,
) -> Result<Result<CssDeclarationBody, crate::CssPropertyValueError>, ()> {
    control.step()?;
    let mut document = match crate::syntax_consumption::normalize_checked_controlled(
        components.items(),
        crate::syntax_consumption::SyntaxInputLimits::default(),
        0,
        control,
    ) {
        Ok(document) => document,
        Err(error) => {
            if control.failure().is_some() {
                return Err(());
            }
            control.component_resource(error.kind)?;
            return crate::CssPropertyValueError::from_component_controlled(error, control)
                .map(Err);
        }
    };
    document.bind_controlled_source(source)?;
    // The original checked front rejects implied closure before every grammar,
    // including CSS-wide and substitution-dependent values.
    for node in &document.nodes {
        control.step()?;
        let crate::syntax_consumption::TokenPayload::Checked { component, .. } =
            node.token().payload
        else {
            unreachable!("controlled checked component")
        };
        if let Some(origin) = component.implicit_termination_origin() {
            return crate::CssPropertyValueError::from_direct_grammar_controlled(
                crate::error::property::implicit_eof_kind(),
                crate::CssSerializedOrigin::End(Some(origin.clone())),
                control,
            )
            .map(Err);
        }
    }
    let mut parser = property_input::Parser::new(&document);
    let result = parse_property_value_from_parser(
        PropertyValueGrammar::Known(CssKnownProperty::ColorScheme.grammar()),
        &mut parser,
        &crate::numeric::NumericInputContext::Checked,
        parser_context,
    );
    if control.failure().is_some() {
        return Err(());
    }
    match result {
        Ok(body) => Ok(Ok(body)),
        Err(error) => {
            let (kind, origin) = error.into_parts_controlled(control)?;
            crate::CssPropertyValueError::from_direct_grammar_controlled(kind, origin, control)
                .map(Err)
        }
    }
}

pub(crate) fn parse_property_value_body(
    property: CssPropertyNameRef<'_>,
    components: &CssComponentValues,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclarationBody, crate::CssPropertyValueError> {
    let grammar = match property {
        CssPropertyNameRef::Known(property) => PropertyValueGrammar::Known(property.grammar()),
        CssPropertyNameRef::Custom(name) => PropertyValueGrammar::Custom(name),
        CssPropertyNameRef::SvgGlyphOrientationVertical => {
            PropertyValueGrammar::SvgGlyph(crate::svg_glyph::SvgGlyphAdmission::css(parser_context))
        }
    };
    parse_property_value_body_selected(grammar, components, parser_context)
}

pub(crate) fn parse_property_value_body_for_grammar(
    grammar: CssPropertyGrammar,
    components: &CssComponentValues,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclarationBody, crate::CssPropertyValueError> {
    parse_property_value_body_selected(
        PropertyValueGrammar::Known(grammar),
        components,
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
    components: &CssComponentValues,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclarationBody, crate::CssPropertyValueError> {
    parse_property_value_slice_selected(
        grammar,
        components.items(),
        components.nesting_depth(),
        parser_context,
    )
}

fn parse_property_value_slice_selected(
    grammar: PropertyValueGrammar<'_>,
    components: &[crate::CssComponentValue],
    depth: u32,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclarationBody, crate::CssPropertyValueError> {
    let refused = crate::CssPropertyValueError::from_component;
    bounded_execution::components(components, depth, refused, || {
        let document = crate::syntax_consumption::normalize(
            crate::syntax_consumption::SyntaxInput::CheckedComponents(components),
            crate::syntax_consumption::SyntaxInputLimits::default(),
            0,
        )
        .map_err(crate::CssPropertyValueError::from_component)?;
        let mut parser = property_input::Parser::new(&document);
        parse_property_value_from_parser(
            grammar,
            &mut parser,
            &crate::numeric::NumericInputContext::Checked,
            parser_context,
        )
        .map_err(|error| {
            let (kind, origin) = error.into_parts();
            crate::CssPropertyValueError::from_direct_grammar(kind, origin)
        })
    })
}

fn parse_property_value_from_parser<'i>(
    grammar: PropertyValueGrammar<'_>,
    parser: &mut property_input::Parser<'i, '_>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclarationBody, crate::error::property::ParseError<'i, crate::error::property::Error>>
{
    // Root delimiters precede lexical and symbolic-value admission. Each next
    // skips a complete nested component without promoting or validating it.
    let start = parser.state();
    loop {
        let location = parser.current_source_location();
        let kind = if parser.control().is_some() {
            let Ok(kind) = parser.next_kind(true) else {
                break;
            };
            kind
        } else {
            let Ok(token) = parser.next_including_whitespace_and_comments().cloned() else {
                break;
            };
            match token {
                Token::Delim(value) => crate::syntax_consumption::TokenKind::Delim(value),
                Token::Semicolon => crate::syntax_consumption::TokenKind::Semicolon,
                Token::CloseCurlyBracket => {
                    crate::syntax_consumption::TokenKind::Closing(crate::CssBlockKind::CurlyBracket)
                }
                Token::CloseParenthesis => {
                    crate::syntax_consumption::TokenKind::Closing(crate::CssBlockKind::Parenthesis)
                }
                Token::CloseSquareBracket => crate::syntax_consumption::TokenKind::Closing(
                    crate::CssBlockKind::SquareBracket,
                ),
                _ => crate::syntax_consumption::TokenKind::Other,
            }
        };
        if matches!(
            kind,
            crate::syntax_consumption::TokenKind::Delim('!')
                | crate::syntax_consumption::TokenKind::Semicolon
                | crate::syntax_consumption::TokenKind::Closing(_)
        ) {
            let Some(token) = location.token() else {
                unreachable!("actual fixed delimiter")
            };
            return Err(location.new_unexpected_token_error(token));
        }
    }
    parser.reset(&start);
    let body = parse_property_declaration_body_from_property_input(
        grammar,
        parser,
        numeric,
        parser_context,
    )?;
    parser.expect_exhausted()?;
    Ok(body)
}

fn parse_property_declaration_body_from_property_input<'i>(
    grammar: PropertyValueGrammar<'_>,
    parser: &mut property_input::Parser<'i, '_>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclarationBody, crate::error::property::ParseError<'i, crate::error::property::Error>>
{
    match grammar {
        PropertyValueGrammar::Known(grammar) => parse_known_declaration_body_from_property_input(
            grammar.resolved(),
            parser,
            numeric,
            parser_context,
        ),
        PropertyValueGrammar::SvgGlyph(admission) => {
            parse_svg_glyph_declaration_body_from_property_input(parser, numeric, admission)
        }
        PropertyValueGrammar::Custom(name) => {
            parse_custom_property_value(parser, numeric).map(|value| {
                CssDeclarationBody::Custom(CssCustomDeclaration::new(name.clone(), value))
            })
        }
    }
}

// Native rule/declaration cursors select a source window; only the portable
// property cursor admits the selected value. The source arena retains lexical
// failures so the property grammar supplies their diagnostic context.
fn parse_property_declaration_source<'i>(
    grammar: PropertyValueGrammar<'_>,
    input: &mut Parser<'i, '_>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclarationBody, ParseError<'i, CssError>> {
    let crate::numeric::NumericInputContext::Parsed(snapshot) = numeric.ordinary() else {
        unreachable!("native property adapters receive source input")
    };
    let start = input.position().byte_index();
    while let Ok(token) = input.next_including_whitespace_and_comments().cloned() {
        fragments::finish_nested_component(input, &token)?;
    }
    let end = input.position().byte_index();
    let document = crate::syntax_consumption::normalize(
        crate::syntax_consumption::SyntaxInput::Source(crate::syntax_consumption::SourceWindow {
            text: std::borrow::Cow::Borrowed(snapshot.as_str()),
            range: start..end,
            original: (*snapshot).clone(),
        }),
        crate::syntax_consumption::SyntaxInputLimits::default(),
        0,
    )
    .map_err(|error| {
        crate::error::invalid_component_value(input.current_source_location(), error)
    })?;
    let mut property_input = property_input::Parser::new(&document);
    parse_property_declaration_body_from_property_input(
        grammar,
        &mut property_input,
        numeric,
        parser_context,
    )
    .map_err(|error| {
        let error = error.into_source_error();
        let position = error.position();
        cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value() + 1,
        }
        .new_custom_error(error)
    })
}

fn parse_known_declaration_body<'i, 't>(
    resolved_property: CssResolvedPropertyName,
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclarationBody, ParseError<'i, CssError>> {
    parse_property_declaration_source(
        PropertyValueGrammar::Known(CssPropertyGrammar::from_resolved(resolved_property)),
        input,
        numeric,
        parser_context,
    )
}

fn parse_svg_glyph_declaration_body<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    admission: crate::svg_glyph::SvgGlyphAdmission,
) -> Result<CssDeclarationBody, ParseError<'i, CssError>> {
    parse_property_declaration_source(
        PropertyValueGrammar::SvgGlyph(admission),
        input,
        numeric,
        crate::CssParserContext::default(),
    )
}

fn parse_known_declaration_body_from_property_input<'i, 't>(
    resolved_property: CssResolvedPropertyName,
    input: &mut property_input::Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    parser_context: crate::CssParserContext,
) -> std::result::Result<
    CssDeclarationBody,
    crate::error::property::ParseError<'i, crate::error::property::Error>,
> {
    let known_property = resolved_property.property();
    let context_name = known_property.canonical_name();
    let state = input.state();
    let (authored, has_substitution) = collect_authored_declaration_value(
        input,
        numeric,
        variables::SubstitutionContext::KnownProperty,
    )
    .map_err(|error| crate::error::property::with_property_context(error, context_name))?;
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
                return Err(crate::error::property::with_property_context(
                    crate::error::property::invalid_syntax(input.current_source_location()),
                    context_name,
                ));
            }
            if let Some(control) = input.control() {
                let bytes = authored.retained_backing_bytes();
                drop(authored);
                control.released(bytes);
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
    let authored_bytes = authored.retained_backing_bytes();
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
    .map_err(|error| {
        if let Some(control) = input.control() {
            control.released(authored_bytes);
        }
        crate::error::property::with_property_context(error, context_name)
    })?;
    input.expect_exhausted().map_err(|error| {
        crate::error::property::with_property_context(error.into(), context_name)
    })?;
    Ok(CssDeclarationBody::Known(declaration.with_grammar(
        CssPropertyGrammar::from_resolved(resolved_property),
    )))
}

fn parse_svg_glyph_declaration_body_from_property_input<'i, 't>(
    input: &mut property_input::Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    admission: crate::svg_glyph::SvgGlyphAdmission,
) -> Result<CssDeclarationBody, crate::error::property::ParseError<'i, crate::error::property::Error>>
{
    let name = "glyph-orientation-vertical";
    let parse = |input: &mut property_input::Parser<'i, 't>| {
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
                    return Err(first_location
                        .new_unexpected_token_error::<crate::error::property::Error>(
                            Token::Ident(ident),
                        ));
                }
            } else {
                input.reset(&first_state);
                input.skip_whitespace();
                let location = input.current_source_location();
                let offset = input.position();
                // SVG has no canonical-property error target. Retain the real
                // root cursor/token for a present-token grammar rejection.
                let literal_start = input.state();
                let root_token = input.next().ok().cloned();
                input.reset(&literal_start);
                let component = input
                    .collect_component()
                    .map_err(crate::CssNumericConstructionError::component)
                    .map_err(|error| values::angle_error(numeric, &error, location, offset))?;
                let root_origin = component.origin().clone();
                let value = if matches!(
                    component.view(),
                    crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Number(_))
                ) {
                    crate::CssSvgGlyphOrientationVerticalValue::try_from_unitless(
                        component.clone().into_owned(),
                    )
                } else {
                    crate::CssAngleValue::from_parser_component(
                        component.clone().into_owned(),
                        numeric,
                    )
                    .and_then(crate::CssSvgGlyphOrientationVerticalValue::from_parser_angle)
                }
                .map_err(|error| {
                    let mapped = values::angle_error(numeric, &error, location, offset);
                    if let Some(token) = &root_token
                        && !crate::error::property::is_resource_parse_error(&mapped)
                        && (matches!(
                            token,
                            Token::Number { .. }
                                | Token::Dimension { .. }
                                | Token::Percentage { .. }
                        ) || (matches!(
                            error.kind(),
                            crate::CssNumericConstructionErrorKind::Component(
                                crate::CssComponentValueErrorKind::InvalidToken
                            )
                        ) && error.origin() == Some(&root_origin))
                            || (matches!(
                                token,
                                Token::Function(_)
                                    | Token::ParenthesisBlock
                                    | Token::SquareBracketBlock
                                    | Token::CurlyBracketBlock
                            ) && matches!(
                                error.kind(),
                                crate::CssNumericConstructionErrorKind::RootDomainMismatch
                            ) && error.path() == Some(&[0][..])
                                && error.origin() == Some(&root_origin)))
                    {
                        location.new_unexpected_token_error::<crate::error::property::Error>(
                            token.clone(),
                        )
                    } else {
                        if matches!(
                            error.kind(),
                            crate::CssNumericConstructionErrorKind::UnknownFunction
                                | crate::CssNumericConstructionErrorKind::IncompatibleTypes
                                | crate::CssNumericConstructionErrorKind::InvalidArgumentType
                                | crate::CssNumericConstructionErrorKind::MalformedExpression
                        ) && error.path().is_some_and(|path| !path.is_empty())
                            && error.origin().is_some()
                        {
                            // The numeric owner returns the actual responsible input site.
                            if let Some(token) = mapped.location.token()
                                && !matches!(token, Token::WhiteSpace(_) | Token::Comment(_))
                                && (!matches!(
                                    error.kind(),
                                    crate::CssNumericConstructionErrorKind::MalformedExpression
                                ) || svg_glyph_numeric_present_rejection(&component, &error))
                            {
                                return mapped
                                    .location
                                    .new_unexpected_token_error::<crate::error::property::Error>(
                                        token,
                                    );
                            }
                        }
                        mapped
                    }
                })?;
                if !value.admitted(admission) {
                    return Err(location
                        .new_unexpected_token_error::<crate::error::property::Error>(
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
    parse(input).map_err(|error| crate::error::property::with_property_context(error, name))
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
    components: &CssComponentValues,
    context: crate::CssParserContext,
) -> Result<CssDeclarationBody, crate::CssPropertyValueError> {
    parse_property_value_body_selected(
        PropertyValueGrammar::SvgGlyph(admission),
        components,
        context,
    )
}

fn parse_legacy_property_alias_value<'i, 't>(
    alias: CssLegacyPropertyAlias,
    authored: CssAuthoredDeclarationValue,
    input: &mut property_input::Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<
    CssKnownDeclaration,
    crate::error::property::ParseError<'i, crate::error::property::Error>,
> {
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

/// The single importance boundary of a declaration value. Syntax 3 (2026 CRD)
/// §5.5.6 step 6: a value whose last two non-whitespace tokens are `!` and
/// `important` carries the importance annotation. Each owner decides whether
/// it admits the annotation and which error a rejected annotation reports:
/// `invalid` receives the `!` location and, for a malformed annotation, the
/// first token that is not part of it.
/// <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-declaration>
pub(super) fn parse_importance_boundary<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    importance_admitted: bool,
    invalid: impl FnOnce(
        cssparser::SourceLocation,
        Option<cssparser::BasicParseError<'i>>,
    ) -> ParseError<'i, CssError>,
    parse_value: impl for<'tt> FnOnce(
        &mut Parser<'i, 'tt>,
    ) -> std::result::Result<T, ParseError<'i, CssError>>,
) -> std::result::Result<(T, CssImportance), ParseError<'i, CssError>> {
    let value = input.parse_until_before(Delimiter::Bang, parse_value)?;
    if input.is_exhausted() {
        return Ok((value, CssImportance::Normal));
    }
    let bang_location = input.current_source_location();
    let annotation = input
        .expect_delim('!')
        .and_then(|()| input.expect_ident_matching("important"))
        .and_then(|()| input.expect_exhausted());
    match annotation {
        Ok(()) if importance_admitted => Ok((value, CssImportance::Important)),
        Ok(()) => Err(invalid(bang_location, None)),
        Err(malformed) => Err(invalid(bang_location, Some(malformed))),
    }
}

fn parse_declaration_boundary<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    context: &DeclarationBoundaryContext<'_>,
    parse_value: impl for<'tt> FnOnce(
        &mut Parser<'i, 'tt>,
    ) -> std::result::Result<T, ParseError<'i, CssError>>,
) -> std::result::Result<(T, CssImportance), ParseError<'i, CssError>> {
    let ordinary = matches!(
        context,
        DeclarationBoundaryContext::OrdinaryKnown(_)
            | DeclarationBoundaryContext::OrdinaryCustom(_)
            | DeclarationBoundaryContext::OrdinarySvgGlyph
    );
    parse_importance_boundary(
        input,
        ordinary,
        |location, _| invalid_annotation_for_context(location, context),
        parse_value,
    )
}

fn invalid_annotation_for_context<'i>(
    location: cssparser::SourceLocation,
    context: &DeclarationBoundaryContext<'_>,
) -> ParseError<'i, CssError> {
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
    ) -> std::result::Result<T, ParseError<'i, CssError>>,
) -> std::result::Result<T, ParseError<'i, CssError>> {
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
) -> Result<CustomMediaPrelude, ParseError<'i, CssError>> {
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
            Ok::<_, ParseError<'i, CssError>>(boolean)
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
    let at = rule_list::collect_at_keyword(source, start, recovery);
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
