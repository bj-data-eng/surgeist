//! One at-rule prelude grammar and one rule-list recovery loop for every
//! rule-list context.
//!
//! Syntax 3 (2026 CRD) §§5.5.2 and 5.5.3 consume a rule's prelude without
//! regard to where the rule appears; placement is a separate rule of each
//! owning specification (Nesting 1 §3.2, Cascade 5 §6.1, Conditional 5 §8,
//! Cascade 6 §2.5). A [`RuleListContext`] therefore selects admission and the
//! body grammar, while [`parse_at_rule_prelude`] owns every prelude.
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-at-rule>
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents>
use crate::declaration::{CssDeclaration, CssDeclarationList};
use crate::diagnostic::CssError;
use crate::diagnostic::error::{
    invalid_at_rule_block, invalid_at_rule_placement, invalid_syntax, with_at_rule_prelude_context,
    with_media_query_context,
};
use crate::parser::counter_style::{CounterStylePrelude, parse_counter_style_rule};
use crate::parser::font_face::parse_font_face_rule;
use crate::parser::import::{CssImportPrelude, parse_import_prelude};
use crate::parser::keyframes::parse_keyframes_rule;
use crate::parser::nesting::{parse_style_contents, parse_style_rule_block};
use crate::parser::page::parse_page_rule;
use crate::parser::queries::parse_media_query_list as parse_media_query_list_inner;
use crate::parser::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use crate::parser::selectors::{
    SelectorRecovery, parse_rule_selector_list, parse_scoped_style_selector_list,
};
use crate::parser::supports::{parse_supports_condition, with_supports_prelude_context};
use crate::parser::{
    CssNamespacePrelude, CssScopePrelude, CustomMediaPrelude, ScopedBodyKind,
    StrictDeclarationParser, TopLevelPreludePhase, block_item_diagnostic, color_profile,
    consume_failed_rule_block, finish_custom_media, font_feature_values, font_palette_values,
    is_declaration_recovery_unit, named_supports, nesting, parse_container_prelude,
    parse_counter_style_prelude, parse_custom_media_prelude, parse_font_face_prelude,
    parse_keyframes_prelude, parse_layer_prelude, parse_namespace_prelude,
    parse_nested_group_rules, parse_page_prelude, parse_scope_prelude, parse_scoped_rule_list,
    queries, selectors, structural_recovery_action, structural_recovery_production,
    structural_rule_diagnostic, syntax_bridge, when, with_container_prelude_context,
    with_scope_prelude_context,
};
use crate::query::container::CssContainerPrelude;
use crate::query::media::CssMediaQueryList;
use crate::query::supports::CssSupportsLeaf;
use crate::query::{CssCondition, CssWhenLeaf};
use crate::rule::color_profile::CssColorProfileRuleName;
use crate::rule::counter_style::CssCounterStyleRule;
use crate::rule::font_face::{CssFontFaceFamily, CssFontFaceRule};
use crate::rule::font_palette_values::CssFontPaletteName;
use crate::rule::keyframes::{CssKeyframesName, CssKeyframesRule};
use crate::rule::page::{CssPageRule, CssPageSelectorList};
use crate::rule::syntax::StyleContents;
use crate::rule::{
    CssContainerRule, CssImportRule, CssLayerBlockRule, CssLayerName, CssLayerNameList,
    CssLayerStatementRule, CssMediaRule, CssNamespaceName, CssNamespacePrefix, CssNamespaceRule,
    CssNestedDeclarationsRule, CssRule, CssScopeRule, CssSheet, CssSupportsRule,
};
use crate::selector::CssStyleSelectorList;
use crate::syntax_consumption::RuleListSyntax;
use crate::syntax_consumption::{GenericRuleResult, SyntaxDocument};
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserInput, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, Token, match_ignore_ascii_case,
};

/// Where a rule list appears.
#[derive(Clone, Copy)]
pub(super) enum RuleListContext {
    /// The stylesheet's own contents: import and namespace ordering apply.
    Stylesheet,
    /// A group rule's body with neither style-rule nor `@scope` ancestry.
    Group,
    /// A body inside `@scope`: the scope body itself or a group within it.
    Scoped {
        body: ScopedBodyKind,
        style_ancestor: bool,
    },
    /// A style rule's body, or a group rule nested inside one.
    Style,
}

impl RuleListContext {
    const fn style_ancestor(self) -> bool {
        match self {
            Self::Stylesheet | Self::Group => false,
            Self::Scoped { style_ancestor, .. } => style_ancestor,
            Self::Style => true,
        }
    }

    const fn scope_body(self) -> bool {
        matches!(
            self,
            Self::Scoped {
                body: ScopedBodyKind::Scope,
                ..
            }
        )
    }

    // Cascade 6 §2.5: a `@scope` prelude is relative to an enclosing scope or
    // style rule, whose `&` it may reference.
    const fn scope_prelude_grammar(self) -> (selectors::SelectorAnchorMode, bool) {
        match self {
            Self::Stylesheet | Self::Group => (selectors::SelectorAnchorMode::Nesting, false),
            Self::Scoped {
                style_ancestor: false,
                ..
            } => (selectors::SelectorAnchorMode::Scope, true),
            Self::Scoped {
                style_ancestor: true,
                ..
            }
            | Self::Style => (selectors::SelectorAnchorMode::Nesting, true),
        }
    }
}

/// Every at-rule this parser admits, with its production identity.
const AT_RULE_PRODUCTIONS: &[(&str, &str)] = &[
    ("custom-media", "ext.rule.custom-media"),
    ("font-feature-values", "later.rule.font-feature-values"),
    ("font-palette-values", "later.rule.font-palette-values"),
    ("color-profile", "interop.rule.color-profile"),
    ("import", "baseline.rule.import"),
    ("namespace", "later.rule.namespace"),
    ("counter-style", "later.rule.counter-style"),
    ("page", "later.rule.page"),
    ("layer", "baseline.rule.layer-block"),
    ("font-face", "baseline.rule.font-face"),
    ("keyframes", "baseline.rule.keyframes"),
    ("media", "baseline.rule.media"),
    ("supports", "baseline.rule.supports"),
    ("supports-condition", "ext.rule.supports-condition"),
    ("container", "baseline.rule.container"),
    ("when", "ext.rule.when"),
    ("else", "ext.rule.else"),
    ("scope", "baseline.rule.scope"),
];

/// The production of an admitted at-rule name, matched ASCII case-insensitively.
pub(super) fn at_rule_production(name: &str) -> Option<&'static str> {
    AT_RULE_PRODUCTIONS
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(name))
        .map(|(_, production)| *production)
}

/// A parsed at-rule prelude, independent of its placement.
pub(crate) enum AtRulePrelude {
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
    Supports(CssCondition<CssSupportsLeaf>),
    SupportsCondition(named_supports::NamedSupportsPrelude),
    Container(CssContainerPrelude),
    When(CssCondition<CssWhenLeaf>, Vec<usize>),
    Else(Option<CssCondition<CssWhenLeaf>>, Vec<usize>),
    Scope(CssScopePrelude),
}

impl AtRulePrelude {
    pub(super) const fn name(&self) -> &'static str {
        match self {
            Self::CustomMedia(_) => "custom-media",
            Self::FontFeatureValues(_) => "font-feature-values",
            Self::FontPaletteValues(_) => "font-palette-values",
            Self::ColorProfile(_) => "color-profile",
            Self::Import(_) => "import",
            Self::Namespace(_) => "namespace",
            Self::CounterStyle(_) => "counter-style",
            Self::Page(_) => "page",
            Self::Layer(_) => "layer",
            Self::FontFace => "font-face",
            Self::Keyframes(_) => "keyframes",
            Self::Media(_) => "media",
            Self::Supports(_) => "supports",
            Self::SupportsCondition(_) => "supports-condition",
            Self::Container(_) => "container",
            Self::When(_, _) => "when",
            Self::Else(_, _) => "else",
            Self::Scope(_) => "scope",
        }
    }

    pub(super) fn production(&self) -> &'static str {
        at_rule_production(self.name()).expect("every prelude names an admitted at-rule")
    }
}

// Placements whose admission depends on context. Nesting 1 §3.2 admits only
// conditional group rules, `@layer` and `@scope` with a style-rule ancestor;
// `@page` is also excluded from a `@scope` body (Cascade 6 §2.5).
const WITHOUT_STYLE_ANCESTOR: &str = "a rule list without a style-rule ancestor";
const PAGE_PLACEMENT: &str =
    "a rule list without a style-rule ancestor that is not itself a @scope body";
const STYLESHEET_TOP_LEVEL: &str = "the stylesheet top level";

fn reject_with_style_ancestor<'i>(
    context: RuleListContext,
    input: &Parser<'i, '_>,
    name: &str,
) -> Result<(), ParseError<'i, CssError>> {
    if context.style_ancestor() {
        Err(invalid_at_rule_placement(
            input.current_source_location(),
            name,
            WITHOUT_STYLE_ANCESTOR,
        ))
    } else {
        Ok(())
    }
}

/// Parses one at-rule prelude and decides its admission in `context`.
/// `phase` is the stylesheet's import/namespace ordering state, present only
/// in [`RuleListContext::Stylesheet`].
#[expect(
    clippy::too_many_lines,
    reason = "one arm per admitted at-rule keeps the single prelude table visible"
)]
pub(super) fn parse_at_rule_prelude<'i>(
    context: RuleListContext,
    phase: Option<TopLevelPreludePhase>,
    source: &'i str,
    name: CowRcStr<'i>,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<AtRulePrelude, ParseError<'i, CssError>> {
    match_ignore_ascii_case! { &name,
        "import" => {
            let Some(phase) = phase else {
                return Err(invalid_at_rule_placement(
                    input.current_source_location(),
                    "import",
                    STYLESHEET_TOP_LEVEL,
                ));
            };
            if !phase.accepts_import() {
                return Err(invalid_at_rule_placement(
                    input.current_source_location(),
                    "import",
                    "before every non-import top-level rule",
                ));
            }
            let prelude = parse_import_prelude(source, input, diagnostics, recovery).map_err(
                |error| {
                    if queries::media_terminal_error(&error) {
                        return error;
                    }
                    with_at_rule_prelude_context(
                        error,
                        "import",
                        "baseline.rule.import",
                        "a supported @import prelude",
                    )
                },
            )?;
            Ok(AtRulePrelude::Import(Box::new(prelude)))
        },
        "namespace" => {
            let Some(phase) = phase else {
                return Err(invalid_at_rule_placement(
                    input.current_source_location(),
                    "namespace",
                    STYLESHEET_TOP_LEVEL,
                ));
            };
            if phase.after_namespace().is_none() {
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
            Ok(AtRulePrelude::Namespace(prelude))
        },
        "counter-style" => {
            reject_with_style_ancestor(context, input, "counter-style")?;
            Ok(AtRulePrelude::CounterStyle(parse_counter_style_prelude(
                source,
                input,
                recovery.source_snapshot(),
            )?))
        },
        "page" => {
            if context.style_ancestor() || context.scope_body() {
                return Err(invalid_at_rule_placement(
                    input.current_source_location(),
                    "page",
                    PAGE_PLACEMENT,
                ));
            }
            Ok(AtRulePrelude::Page(parse_page_prelude(
                source,
                input,
                recovery.source_snapshot(),
            )?))
        },
        "custom-media" => {
            reject_with_style_ancestor(context, input, "custom-media")?;
            Ok(AtRulePrelude::CustomMedia(Box::new(parse_custom_media_prelude(
                source, input, recovery,
            )?)))
        },
        "font-feature-values" => {
            reject_with_style_ancestor(context, input, "font-feature-values")?;
            Ok(AtRulePrelude::FontFeatureValues(font_feature_values::parse_families(
                source, input, recovery,
            )?))
        },
        "font-palette-values" => {
            reject_with_style_ancestor(context, input, "font-palette-values")?;
            Ok(AtRulePrelude::FontPaletteValues(font_palette_values::parse_name(
                source, input, recovery,
            )?))
        },
        "color-profile" => {
            reject_with_style_ancestor(context, input, "color-profile")?;
            Ok(AtRulePrelude::ColorProfile(color_profile::parse_name(
                source, input, recovery,
            )?))
        },
        "font-face" => {
            reject_with_style_ancestor(context, input, "font-face")?;
            parse_font_face_prelude(input)?;
            Ok(AtRulePrelude::FontFace)
        },
        "keyframes" => {
            reject_with_style_ancestor(context, input, "keyframes")?;
            Ok(AtRulePrelude::Keyframes(parse_keyframes_prelude(input)?))
        },
        "layer" => Ok(AtRulePrelude::Layer(parse_layer_prelude(input).map_err(|error| {
            with_at_rule_prelude_context(
                error,
                "layer",
                "baseline.rule.layer-block",
                "a supported @layer prelude",
            )
        })?)),
        "media" => {
            let query = parse_media_query_list_inner(source, input, diagnostics, recovery)?;
            if !input.is_exhausted() {
                return Err(with_media_query_context(
                    invalid_syntax(input.current_source_location()),
                    None,
                ));
            }
            Ok(AtRulePrelude::Media(query))
        },
        "supports" => Ok(AtRulePrelude::Supports(
            parse_supports_condition(source, input, diagnostics, recovery)
                .map_err(with_supports_prelude_context)?,
        )),
        "supports-condition" => Ok(AtRulePrelude::SupportsCondition(
            named_supports::parse_prelude(input, recovery)?,
        )),
        "when" => {
            let (condition, implicit) =
                when::parse_prelude(source, input, recovery, name.as_ref(), false)?;
            Ok(AtRulePrelude::When(condition.expect("required condition"), implicit))
        },
        "else" => {
            let (condition, implicit) =
                when::parse_prelude(source, input, recovery, name.as_ref(), true)?;
            Ok(AtRulePrelude::Else(condition, implicit))
        },
        "container" => {
            let prelude = parse_container_prelude(source, input, recovery)
                .map_err(with_container_prelude_context)?;
            if !input.is_exhausted() {
                return Err(with_at_rule_prelude_context(
                    invalid_syntax(input.current_source_location()),
                    "container",
                    "baseline.rule.container",
                    "the end of the @container prelude",
                ));
            }
            Ok(AtRulePrelude::Container(prelude))
        },
        "scope" => {
            let (anchors, relative_root) = context.scope_prelude_grammar();
            Ok(AtRulePrelude::Scope(
                parse_scope_prelude(source, input, diagnostics, recovery, anchors, relative_root)
                    .map_err(with_scope_prelude_context)?,
            ))
        },
        _ => Err(input.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name))),
    }
}

/// Per-unit observations a grammar records for its recovery step.
pub(crate) struct UnitState<'i> {
    pub(super) boundary: nesting::StyleRuleBoundary,
    /// A typed resource failure from a qualified-rule attempt, which a native
    /// declaration fallback would otherwise discard.
    pub(super) qualified_resource_error: Option<ParseError<'i, CssError>>,
    /// The prelude failure of a nested qualified-rule attempt (Syntax 3 (2026
    /// CRD) §5.5.5 reparses a failed declaration as a qualified rule).
    pub(super) qualified_prelude_error: Option<ParseError<'i, CssError>>,
    pub(super) rejected_at_rule_start: Option<usize>,
}

impl<'i> UnitState<'i> {
    pub(super) const fn new() -> Self {
        Self {
            boundary: nesting::StyleRuleBoundary::None,
            qualified_resource_error: None,
            qualified_prelude_error: None,
            rejected_at_rule_start: None,
        }
    }

    /// Records a qualified-rule attempt's failure: a resource failure from its
    /// prelude or block, or any other failure of its prelude.
    pub(super) fn record_qualified_failure<T>(
        &mut self,
        result: &Result<T, ParseError<'i, CssError>>,
        prelude: bool,
    ) {
        if let Err(error) = result {
            if crate::diagnostic::error::is_nesting_limit_error(error) {
                self.qualified_resource_error = Some(error.clone());
            } else if prelude {
                self.qualified_prelude_error = Some(error.clone());
            }
        }
    }
}

/// A rule-list grammar: native callbacks plus the unit state its recovery reads.
pub(crate) trait RuleListGrammar<'i, I>:
    RuleBodyItemParser<'i, I, CssError>
    + AtRuleParser<'i, AtRule = I, Error = CssError>
    + QualifiedRuleParser<'i, QualifiedRule = I, Error = CssError>
    + DeclarationParser<'i, Declaration = I, Error = CssError>
{
    fn unit(&mut self) -> &mut UnitState<'i>;
    fn recovery(&self) -> &RecoveryState;
    fn diagnostics(&mut self) -> &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>;
}

/// Receives a rule list's retained items in source order.
pub(crate) trait RuleListSink<I> {
    /// A rule boundary ends any pending declaration run.
    fn partition(&mut self) {}
    fn retain(
        &mut self,
        item: I,
        source: &str,
        unit_end: usize,
        diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    );
}

/// How a list's units are delimited.
pub(crate) enum RuleListUnits {
    /// Units selected by the generic Syntax 3 consumer; declarations are not
    /// admitted in this list.
    Selected {
        document: std::rc::Rc<SyntaxDocument<'static>>,
        selected: Vec<GenericRuleResult>,
    },
    /// Native block contents that try each ident-started unit as a
    /// declaration before a nested rule (Syntax 3 (2026 CRD) §5.5.5).
    Declarations,
}

impl RuleListUnits {
    /// Selects the generic units of the list that starts at `input`.
    pub(super) fn select<'i>(
        source: &str,
        input: &Parser<'i, '_>,
        recovery: &RecoveryState,
        list: RuleListSyntax,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        let (document, selected) = syntax_bridge::rules(source, input, recovery, list)?;
        Ok(Self::Selected { document, selected })
    }

    pub(super) fn select_block_contents<'i>(
        source: &str,
        input: &Parser<'i, '_>,
        recovery: &RecoveryState,
    ) -> Result<Self, ParseError<'i, CssError>> {
        Self::select(source, input, recovery, RuleListSyntax::BlockContents).map_err(|error| {
            crate::diagnostic::error::invalid_component_value(
                input.current_source_location(),
                error,
            )
        })
    }
}

/// The single rule-list recovery loop: each unit is admitted by `grammar`,
/// retained into `sink` or dropped with one diagnostic spanning exactly that
/// unit, and the loop always makes progress.
pub(crate) fn recover_rule_list<'i, I, G, S>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    grammar: &mut G,
    sink: &mut S,
    units: &RuleListUnits,
) where
    G: RuleListGrammar<'i, I>,
    S: RuleListSink<I>,
{
    let recovery = grammar.recovery().clone();
    let mut next_selected = 0;
    loop {
        let progress = RecoveryProgress::record(input);
        *grammar.unit() = UnitState::new();
        let (item, unit_start) = match units {
            RuleListUnits::Selected { document, selected } => {
                let Some(unit) = selected.get(next_selected) else {
                    break;
                };
                next_selected += 1;
                let (unit_start, item) = syntax_bridge::parse_selected(
                    source, input, grammar, &recovery, document, unit,
                );
                (item, unit_start)
            }
            RuleListUnits::Declarations => {
                let unit_start = skip_block_contents_trivia(input);
                let Some(item) = RuleBodyParser::new(input, grammar).next() else {
                    break;
                };
                (
                    item.map_err(|(error, failed_unit)| (Box::new(error), failed_unit)),
                    unit_start,
                )
            }
        };
        let unit = std::mem::replace(grammar.unit(), UnitState::new());
        let declarations = matches!(units, RuleListUnits::Declarations);
        if recover_unit(
            source,
            input,
            grammar,
            sink,
            item,
            unit,
            unit_start,
            progress,
            declarations,
        ) == RecoveryLoopOutcome::Terminated
        {
            break;
        }
    }
    if matches!(units, RuleListUnits::Selected { .. }) {
        syntax_bridge::discard_block_contents_tail(input);
    }
}

/// Skips what native block-contents iteration skips before a unit, returning
/// the unit's start: whitespace, comments, `;` and a stray `}`.
pub(super) fn skip_block_contents_trivia(input: &mut Parser<'_, '_>) -> usize {
    loop {
        let state = input.state();
        match input.next_including_whitespace_and_comments() {
            Ok(
                Token::WhiteSpace(_)
                | Token::Comment(_)
                | Token::Semicolon
                | Token::CloseCurlyBracket,
            ) => {}
            Ok(_) | Err(_) => {
                input.reset(&state);
                return input.position().byte_index();
            }
        }
    }
}

// Keep failure temporaries out of the loop's recursive frame.
#[inline(never)]
#[expect(
    clippy::too_many_arguments,
    reason = "one recovery step reads the unit, its parser and its sink"
)]
fn recover_unit<'i, I, G, S>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    grammar: &mut G,
    sink: &mut S,
    item: Result<I, (Box<ParseError<'i, CssError>>, &'i str)>,
    unit: UnitState<'i>,
    unit_start: usize,
    progress: RecoveryProgress,
    declarations: bool,
) -> RecoveryLoopOutcome
where
    G: RuleListGrammar<'i, I>,
    S: RuleListSink<I>,
{
    let (failed_at_block, failed_block_error) = item
        .as_ref()
        .err()
        .map(|(_, failed_unit)| {
            consume_failed_rule_block(
                source,
                input,
                unit_start,
                grammar.recovery(),
                structural_recovery_production(failed_unit),
            )
        })
        .unwrap_or((false, None));
    let outcome = progress.finish(input, item.is_ok());
    let unit_end = input.position().byte_index();
    if unit.boundary.partitions(failed_at_block) {
        sink.partition();
    }
    match item {
        Ok(item) => sink.retain(item, source, unit_end, grammar.diagnostics()),
        Err((error, failed_unit))
            if declarations
                && is_declaration_recovery_unit(failed_unit)
                && !failed_at_block
                && unit.qualified_resource_error.is_none() =>
        {
            if let Some(diagnostic) = block_item_diagnostic(
                source,
                *error,
                failed_unit,
                unit_end,
                crate::diagnostic::CssRecoveryAction::DropDeclaration,
            ) {
                grammar.diagnostics().push(diagnostic);
            }
        }
        Err((error, failed_unit)) => {
            let error = if let Some(resource) = unit.qualified_resource_error.or(failed_block_error)
            {
                resource
            } else if let Some(prelude) = unit.qualified_prelude_error.filter(|_| failed_at_block) {
                // The unit was consumed as a nested qualified rule whose
                // prelude failed; that is the failure of the dropped rule.
                prelude
            } else {
                syntax_bridge::rejected_at_rule_error(
                    source,
                    grammar.recovery(),
                    unit.rejected_at_rule_start,
                    *error,
                )
            };
            if let Some(diagnostic) = structural_rule_diagnostic(
                source,
                error,
                failed_unit,
                unit_start,
                unit_end,
                structural_recovery_action(failed_unit),
            ) {
                grammar.diagnostics().push(diagnostic);
            }
        }
    }
    outcome
}

/// A rule whose body is not a rule list. Every list that admits one holds
/// the same value.
pub(super) enum LeafRule {
    CounterStyle(CssCounterStyleRule),
    Page(CssPageRule),
    FontFace(CssFontFaceRule),
    Keyframes(CssKeyframesRule),
    FontFeatureValues(crate::rule::font_feature_values::CssFontFeatureValuesRule),
    FontPaletteValues(crate::rule::font_palette_values::CssFontPaletteValuesRule),
    ColorProfile(crate::rule::color_profile::CssColorProfileRule),
    SupportsCondition(crate::rule::CssSupportsConditionRule),
}

impl From<LeafRule> for CssRule {
    fn from(rule: LeafRule) -> Self {
        match rule {
            LeafRule::CounterStyle(rule) => Self::CounterStyle(rule),
            LeafRule::Page(rule) => Self::Page(rule),
            LeafRule::FontFace(rule) => Self::FontFace(rule),
            LeafRule::Keyframes(rule) => Self::Keyframes(rule),
            LeafRule::FontFeatureValues(rule) => Self::FontFeatureValues(rule),
            LeafRule::FontPaletteValues(rule) => Self::FontPaletteValues(rule),
            LeafRule::ColorProfile(rule) => Self::ColorProfile(rule),
            LeafRule::SupportsCondition(rule) => Self::SupportsCondition(rule),
        }
    }
}

/// Parses the block of a leaf rule, or rejects the block of a statement-only
/// rule. A group prelude is returned for its list's own body grammar.
// Keep descriptor parsing off the recursive group-rule frame. Deep rule
// lists run on the test thread's ordinary stack.
#[inline(never)]
pub(super) fn parse_leaf_block<'i>(
    prelude: AtRulePrelude,
    source: &'i str,
    start: &ParserState,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<Result<LeafRule, AtRulePrelude>, ParseError<'i, CssError>> {
    let rule = match prelude {
        AtRulePrelude::CustomMedia(_) => {
            return Err(invalid_at_rule_block(
                input,
                "custom-media",
                "ext.rule.custom-media",
                "a statement-form custom-media rule",
            ));
        }
        AtRulePrelude::Import(_) => {
            return Err(invalid_at_rule_block(
                input,
                "import",
                "baseline.rule.import",
                "a semicolon-terminated @import rule",
            ));
        }
        AtRulePrelude::Namespace(_) => {
            return Err(invalid_at_rule_block(
                input,
                "namespace",
                "later.rule.namespace",
                "a semicolon-terminated @namespace rule",
            ));
        }
        AtRulePrelude::CounterStyle(name) => LeafRule::CounterStyle(parse_counter_style_rule(
            source,
            name,
            input,
            start,
            diagnostics,
            recovery.clone(),
        )?),
        AtRulePrelude::Page(selector) => LeafRule::Page(parse_page_rule(
            source,
            selector,
            input,
            start,
            diagnostics,
            recovery.clone(),
        )?),
        AtRulePrelude::FontFeatureValues(families) => {
            LeafRule::FontFeatureValues(font_feature_values::parse_rule(
                source,
                families,
                input,
                start,
                diagnostics,
                recovery.clone(),
            )?)
        }
        AtRulePrelude::FontPaletteValues(name) => {
            LeafRule::FontPaletteValues(font_palette_values::parse_rule(
                source,
                name,
                input,
                start,
                diagnostics,
                recovery.clone(),
            )?)
        }
        AtRulePrelude::ColorProfile(name) => LeafRule::ColorProfile(color_profile::parse_rule(
            source,
            name,
            input,
            start,
            diagnostics,
            recovery.clone(),
        )?),
        AtRulePrelude::FontFace => LeafRule::FontFace(parse_font_face_rule(
            source,
            input,
            start,
            diagnostics,
            recovery.clone(),
        )?),
        AtRulePrelude::Keyframes(name) => LeafRule::Keyframes(parse_keyframes_rule(
            source,
            name,
            input,
            start,
            diagnostics,
            recovery.clone(),
        )?),
        AtRulePrelude::SupportsCondition(prelude) => LeafRule::SupportsCondition(
            named_supports::parse_rule(source, prelude, start, input, diagnostics, recovery)?,
        ),
        group => return Ok(Err(group)),
    };
    Ok(Ok(rule))
}

/// A rule list's ordinary items: declarations only in a style-rule body.
pub(crate) enum RuleListItem {
    Declaration(Box<CssDeclaration>),
    Rules(Vec<CssRule>),
}

impl RuleListItem {
    pub(super) fn into_rules(self) -> Vec<CssRule> {
        match self {
            Self::Rules(rules) => rules,
            Self::Declaration(_) => unreachable!("an ordinary rule never yields a declaration"),
        }
    }
}

/// The ordinary rule grammar of a stylesheet, group or style-rule body.
pub(crate) struct RuleParser<'s> {
    source: &'s str,
    top_level_phase: Option<TopLevelPreludePhase>,
    /// Whether an enclosing style rule supplies declarations and parent selectors.
    style: bool,
    /// The kind of `@scope` body this list is, or lives within.
    scope: Option<ScopedBodyKind>,
    pub(super) diagnostics: Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    pub(super) recovery: RecoveryState,
    unit: UnitState<'s>,
}

impl<'s> RuleParser<'s> {
    pub(super) fn top_level(source: &'s str, recovery: RecoveryState) -> Self {
        Self::new(
            source,
            recovery,
            Some(TopLevelPreludePhase::Initial),
            false,
            None,
        )
    }

    pub(super) fn group(source: &'s str, recovery: RecoveryState) -> Self {
        Self::new(source, recovery, None, false, None)
    }

    pub(super) fn style(source: &'s str, recovery: RecoveryState) -> Self {
        Self::new(source, recovery, None, true, None)
    }

    /// A `@scope` body, or a group rule's body within one, whose actual style
    /// ancestry is `style_ancestor`.
    pub(super) fn scoped(
        source: &'s str,
        recovery: RecoveryState,
        style_ancestor: bool,
        body: ScopedBodyKind,
    ) -> Self {
        Self::new(source, recovery, None, style_ancestor, Some(body))
    }

    const fn new(
        source: &'s str,
        recovery: RecoveryState,
        top_level_phase: Option<TopLevelPreludePhase>,
        style: bool,
        scope: Option<ScopedBodyKind>,
    ) -> Self {
        Self {
            source,
            top_level_phase,
            style,
            scope,
            diagnostics: Vec::new(),
            recovery,
            unit: UnitState::new(),
        }
    }

    const fn context(&self) -> RuleListContext {
        if let Some(body) = self.scope {
            RuleListContext::Scoped {
                body,
                style_ancestor: self.style,
            }
        } else if self.style {
            RuleListContext::Style
        } else if self.top_level_phase.is_some() {
            RuleListContext::Stylesheet
        } else {
            RuleListContext::Group
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

    fn position(&self, start: &ParserState) -> crate::source::CssSourcePosition {
        self.recovery.source_position(start.position().byte_index())
    }

    // A group rule's body: block contents with this list's style ancestry.
    // Kept out of `parse_block`'s recursive frame.
    #[inline(never)]
    fn group_body<'i>(
        &mut self,
        input: &mut Parser<'s, 'i>,
    ) -> Result<Vec<CssRule>, ParseError<'s, CssError>> {
        if self.scope.is_some() {
            let recovered = parse_scoped_rule_list(
                self.source,
                input,
                self.recovery.clone(),
                self.style,
                ScopedBodyKind::OrdinaryGroup,
            )?;
            self.diagnostics.extend(recovered.diagnostics);
            Ok(recovered.syntax)
        } else if self.style {
            let recovered = parse_style_contents(self.source, input, self.recovery.clone())?;
            self.diagnostics.extend(recovered.diagnostics);
            Ok(recovered.syntax.into_nested_rules())
        } else {
            let recovered = parse_nested_group_rules(self.source, input, self.recovery.clone())?;
            self.diagnostics.extend(recovered.diagnostics);
            Ok(recovered.syntax)
        }
    }

    #[inline(never)]
    fn scope_body<'i>(
        &mut self,
        input: &mut Parser<'s, 'i>,
    ) -> Result<Vec<CssRule>, ParseError<'s, CssError>> {
        let recovered = parse_scoped_rule_list(
            self.source,
            input,
            self.recovery.clone(),
            self.style,
            ScopedBodyKind::Scope,
        )?;
        self.diagnostics.extend(recovered.diagnostics);
        Ok(recovered.syntax)
    }
}

impl<'i> RuleListGrammar<'i, RuleListItem> for RuleParser<'i> {
    fn unit(&mut self) -> &mut UnitState<'i> {
        &mut self.unit
    }
    fn recovery(&self) -> &RecoveryState {
        &self.recovery
    }
    fn diagnostics(&mut self) -> &mut Vec<crate::diagnostic::CssRecoveryDiagnostic> {
        &mut self.diagnostics
    }
}

impl<'i> AtRuleParser<'i> for RuleParser<'i> {
    type Prelude = AtRulePrelude;
    type AtRule = RuleListItem;
    type Error = CssError;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        self.unit.boundary = nesting::StyleRuleBoundary::AtRule;
        parse_at_rule_prelude(
            self.context(),
            self.top_level_phase,
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
    ) -> Result<Self::AtRule, ()> {
        let result = match prelude {
            AtRulePrelude::CustomMedia(prelude) => {
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
            AtRulePrelude::Import(prelude) => {
                self.diagnostics.extend(prelude.diagnostics);
                let at_keyword = collect_at_keyword(self.source, start, &self.recovery);
                self.recovery
                    .retain_component_closures(prelude.implicit_media_closures);
                let rule = CssRule::Import(CssImportRule::new(
                    prelude.target,
                    prelude.layer,
                    prelude.supports,
                    prelude.media,
                    crate::rule::imports::ImportSyntax {
                        at_keyword,
                        prelude: prelude.syntax,
                    },
                ));
                self.mark_successful_import();
                Ok(vec![rule])
            }
            AtRulePrelude::Namespace(prelude) => {
                let rule = CssNamespaceRule::new(prelude.prefix, prelude.name)
                    .with_position(self.position(start));
                if self.mark_successful_namespace(rule.prefix().cloned(), rule.name().clone()) {
                    Ok(vec![CssRule::Namespace(rule)])
                } else {
                    Err(())
                }
            }
            AtRulePrelude::Layer(names) => CssLayerNameList::try_new(names)
                .map(|names| {
                    self.mark_successful_layer_statement();
                    vec![CssRule::LayerStatement(CssLayerStatementRule::new(
                        names,
                        self.position(start),
                    ))]
                })
                .ok_or(()),
            AtRulePrelude::CounterStyle(_)
            | AtRulePrelude::Page(_)
            | AtRulePrelude::FontFeatureValues(_)
            | AtRulePrelude::FontPaletteValues(_)
            | AtRulePrelude::ColorProfile(_)
            | AtRulePrelude::FontFace
            | AtRulePrelude::Keyframes(_)
            | AtRulePrelude::Media(_)
            | AtRulePrelude::Supports(_)
            | AtRulePrelude::SupportsCondition(_)
            | AtRulePrelude::Container(_)
            | AtRulePrelude::When(_, _)
            | AtRulePrelude::Else(_, _)
            | AtRulePrelude::Scope(_) => Err(()),
        };
        match &result {
            Ok(_) => syntax_bridge::retain_statement_eof(
                self.source,
                &self.recovery,
                start,
                &mut self.diagnostics,
            ),
            Err(()) if self.style || self.scope.is_some() => {
                self.unit.rejected_at_rule_start = Some(start.position().byte_index());
            }
            Err(()) => {}
        }
        result.map(RuleListItem::Rules)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Self::Error>> {
        let mut depth = self
            .recovery
            .enter_rule_block(self.source, input, prelude.production())?;
        // An `@else` continues its chain and a named supports condition is a
        // definition: neither ends the import/namespace prelude.
        let body_rule = !matches!(
            prelude,
            AtRulePrelude::Else(_, _) | AtRulePrelude::SupportsCondition(_)
        );
        let prelude = match parse_leaf_block(
            prelude,
            self.source,
            start,
            input,
            &mut self.diagnostics,
            &self.recovery,
        )? {
            Ok(leaf) => {
                if body_rule {
                    self.mark_successful_body_rule();
                }
                depth.retain();
                return Ok(RuleListItem::Rules(vec![leaf.into()]));
            }
            Err(group) => group,
        };
        let rules = match prelude {
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
                let rules = self.group_body(input)?;
                vec![CssRule::LayerBlock(CssLayerBlockRule::new(
                    name,
                    rules,
                    self.position(start),
                ))]
            }
            AtRulePrelude::Media(query) => {
                let rules = self.group_body(input)?;
                vec![CssRule::Media(CssMediaRule::new(
                    query,
                    rules,
                    self.position(start),
                ))]
            }
            AtRulePrelude::Supports(condition) => {
                let rules = self.group_body(input)?;
                vec![CssRule::Supports(CssSupportsRule::new(
                    condition,
                    rules,
                    self.position(start),
                ))]
            }
            AtRulePrelude::When(condition, implicit) => {
                let rules = self.group_body(input)?;
                self.recovery.retain_component_closures(implicit);
                when::assemble_when_rule(condition, rules, self.position(start))
            }
            AtRulePrelude::Else(condition, implicit) => {
                let rules = self.group_body(input)?;
                self.recovery.retain_component_closures(implicit);
                when::assemble_else_rule(condition, rules, self.position(start))
            }
            AtRulePrelude::Container(prelude) => {
                let rules = self.group_body(input)?;
                vec![CssRule::Container(CssContainerRule::new(
                    prelude,
                    rules,
                    self.position(start),
                ))]
            }
            AtRulePrelude::Scope(prelude) => {
                let rules = self.scope_body(input)?;
                vec![CssRule::Scope(CssScopeRule::new(
                    prelude.root,
                    prelude.limit,
                    rules,
                    self.position(start),
                ))]
            }
            leaf => unreachable!("{} has a leaf body", leaf.name()),
        };
        if body_rule {
            self.mark_successful_body_rule();
        }
        depth.retain();
        Ok(RuleListItem::Rules(rules))
    }
}

impl<'i> QualifiedRuleParser<'i> for RuleParser<'i> {
    type Prelude = CssStyleSelectorList;
    type QualifiedRule = RuleListItem;
    type Error = CssError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        self.unit.boundary = nesting::StyleRuleBoundary::QualifiedPrelude;
        let mut recovery =
            SelectorRecovery::new(self.source, &mut self.diagnostics, self.recovery.clone());
        if self.scope.is_some() {
            // A scope body's selectors keep scope anchors, and are relative to the scope.
            let result = parse_scoped_style_selector_list(input, &mut recovery);
            self.unit.record_qualified_failure(&result, true);
            return result;
        }
        if !self.style {
            return parse_rule_selector_list(input, &mut recovery)
                .map(CssStyleSelectorList::absolute);
        }
        let result = selectors::parse_nested_style_selector_list(input, &mut recovery)
            .map(CssStyleSelectorList::new);
        self.unit.record_qualified_failure(&result, true);
        result
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        self.unit.boundary = nesting::StyleRuleBoundary::QualifiedBlock;
        let result = (|| {
            let mut depth =
                self.recovery
                    .enter_rule_block(self.source, input, "baseline.rule.style")?;
            let recovered = parse_style_rule_block(
                self.source,
                selectors,
                self.position(start),
                input,
                self.recovery.clone(),
            )?;
            self.diagnostics.extend(recovered.diagnostics);
            self.mark_successful_body_rule();
            depth.retain();
            Ok(RuleListItem::Rules(recovered.syntax))
        })();
        if self.style || self.scope.is_some() {
            self.unit.record_qualified_failure(&result, false);
        }
        result
    }
}

impl<'i> DeclarationParser<'i> for RuleParser<'i> {
    type Declaration = RuleListItem;
    type Error = CssError;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
        StrictDeclarationParser::new(self.source, self.recovery.clone())
            .parse_value(name, input, declaration_start)
            .map(Box::new)
            .map(RuleListItem::Declaration)
    }
}

impl<'i> RuleBodyItemParser<'i, RuleListItem, CssError> for RuleParser<'i> {
    fn parse_declarations(&self) -> bool {
        self.style
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

/// Collects a statement's at-keyword from its original source.
pub(super) fn collect_at_keyword(
    source: &str,
    start: &ParserState,
    recovery: &RecoveryState,
) -> crate::component::CssComponentValue {
    // An at-keyword never holds a URL repair, so the unprepared source is
    // its exact token stream; reading it is bounded by the token itself.
    let mut token_input = ParserInput::new(source);
    let mut token_parser = Parser::new(&mut token_input);
    token_parser.reset(start);
    crate::component::CssComponentValue::collect_from_parser(
        &mut token_parser,
        recovery.source_snapshot(),
    )
    .expect("parsed statement at-keyword")
}

/// Rules retained by a group body.
pub(crate) struct GroupSink(pub(super) Vec<CssRule>);

impl RuleListSink<RuleListItem> for GroupSink {
    fn retain(
        &mut self,
        item: RuleListItem,
        _: &str,
        _: usize,
        _: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    ) {
        self.0.extend(item.into_rules());
    }
}

/// Rules retained by a stylesheet, which also reports a namespace prefix
/// declared again (CSS Namespaces 3 §3: the last declaration wins).
pub(crate) struct SheetSink(pub(super) CssSheet);

impl RuleListSink<RuleListItem> for SheetSink {
    fn retain(
        &mut self,
        item: RuleListItem,
        source: &str,
        unit_end: usize,
        diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    ) {
        for rule in item.into_rules() {
            if let CssRule::Namespace(namespace) = &rule
                && let Some(previous) =
                    self.0
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
                let previous_position = previous.position().expect("parsed namespace rule");
                let span = crate::source::CssSourceSpan::new(
                    position,
                    crate::source::CssSourcePosition::from_byte_offset_in(source, unit_end),
                )
                .expect("retained namespace rule lies within its source unit");
                diagnostics.push(
                    crate::diagnostic::CssRecoveryDiagnostic::new(
                        crate::diagnostic::error::namespace_redeclaration(
                            namespace.prefix().cloned(),
                            position,
                            previous_position,
                        ),
                        span,
                        crate::diagnostic::CssRecoveryAction::RetainNonconformingRule,
                    )
                    .expect("namespace error is at the retained rule's source start"),
                );
            }
            self.0.push_rule(rule);
        }
    }
}

/// A style rule's contents: the leading declarations, then child rules with
/// later declaration runs as nested declarations rules (Syntax 3 (2026 CRD)
/// §5.5.3).
pub(super) struct StyleSink {
    declarations: Vec<CssDeclaration>,
    rules: Vec<CssRule>,
    buffer: Vec<CssDeclaration>,
    leading_slot_open: bool,
}

impl StyleSink {
    pub(super) const fn new() -> Self {
        Self {
            declarations: Vec::new(),
            rules: Vec::new(),
            buffer: Vec::new(),
            leading_slot_open: true,
        }
    }

    pub(super) fn finish(mut self) -> StyleContents {
        self.partition();
        StyleContents {
            declarations: CssDeclarationList::new(self.declarations),
            rules: self.rules,
        }
    }
}

impl RuleListSink<RuleListItem> for StyleSink {
    fn partition(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        if self.leading_slot_open {
            // A transferred first list occupies the leading slot even if the
            // rule that caused this transfer is rejected and leaves no child.
            self.leading_slot_open = false;
            self.declarations.append(&mut self.buffer);
        } else {
            self.rules
                .push(CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(
                    CssDeclarationList::new(std::mem::take(&mut self.buffer)),
                )));
        }
    }

    fn retain(
        &mut self,
        item: RuleListItem,
        _: &str,
        _: usize,
        _: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    ) {
        match item {
            RuleListItem::Declaration(declaration) => self.buffer.push(*declaration),
            RuleListItem::Rules(rules) => {
                if !rules.is_empty() {
                    self.leading_slot_open = false;
                }
                self.rules.extend(rules);
            }
        }
    }
}

/// A scope body, whose declaration runs are nested declarations rules.
pub(crate) struct ScopedSink {
    rules: Vec<CssRule>,
    buffer: Vec<CssDeclaration>,
}

impl ScopedSink {
    pub(super) const fn new() -> Self {
        Self {
            rules: Vec::new(),
            buffer: Vec::new(),
        }
    }

    pub(super) fn finish(mut self) -> Vec<CssRule> {
        self.partition();
        self.rules
    }
}

impl RuleListSink<RuleListItem> for ScopedSink {
    fn partition(&mut self) {
        if !self.buffer.is_empty() {
            self.rules
                .push(CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(
                    CssDeclarationList::new(std::mem::take(&mut self.buffer)),
                )));
        }
    }

    fn retain(
        &mut self,
        item: RuleListItem,
        _: &str,
        _: usize,
        _: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    ) {
        match item {
            RuleListItem::Declaration(declaration) => self.buffer.push(*declaration),
            RuleListItem::Rules(rules) => self.rules.extend(rules),
        }
    }
}
