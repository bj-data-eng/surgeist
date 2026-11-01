//! One at-rule prelude grammar and one rule-list recovery loop for every
//! rule-list context.
//!
//! Syntax 3 (2026 CRD) §§5.5.2 and 5.5.3 consume a rule's prelude without
//! regard to where the rule appears; placement is a separate rule of each
//! owning specification (Nesting 1 §3.2, Cascade 5 §6.1, Conditional 5 §8,
//! Cascade 6 §2.5). A [`RuleListContext`] therefore selects admission and the
//! body grammar, while [`parse_at_rule_prelude`] owns every prelude. The units
//! of a list are the parser crate's: the recovery loop reads them from its
//! iterators over the document, and the grammar is handed each rule with its
//! prelude and block already delimited.
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-at-rule>
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents>
use crate::declaration::{CssDeclaration, CssDeclarationList};
use crate::diagnostic::CssError;
use crate::diagnostic::error::{
    invalid_at_rule_body_at, invalid_at_rule_placement, invalid_syntax, unexpected_syntax_end,
    unexpected_syntax_token, with_at_rule_prelude_context, with_media_query_context,
};
use crate::parser::block_units::{
    DeclarationUnit, Failure, GrammarError, NativeError, dropped_unit, grammar_error, location_at,
    native_error, offset_of,
};
use crate::parser::counter_style::{CounterStylePrelude, parse_counter_style_rule};
use crate::parser::declaration_unit::DeclarationText;
use crate::parser::font_face::parse_font_face_rule;
use crate::parser::import::{CssImportPrelude, parse_import_prelude};
use crate::parser::keyframes::parse_keyframes_rule;
use crate::parser::nesting::{parse_style_contents, parse_style_rule_block};
use crate::parser::page::parse_page_rule;
use crate::parser::queries::parse_media_query_list as parse_media_query_list_inner;
use crate::parser::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use crate::parser::selectors::{
    SelectorRecovery, parse_rule_selector_list_stream, parse_scoped_style_selector_list_stream,
};
use crate::parser::supports::{parse_supports_condition, with_supports_prelude_context};
use crate::parser::syntax_types::{Attached, Site, Stream, SyntaxError};
use crate::parser::{
    CssNamespacePrelude, CssScopePrelude, CustomMediaPrelude, ScopedBodyKind,
    StrictDeclarationParser, TopLevelPreludePhase, block_item_diagnostic, color_profile,
    finish_custom_media, font_feature_values, font_palette_values, is_declaration_recovery_unit,
    named_supports, nesting, parse_container_prelude, parse_counter_style_prelude,
    parse_custom_media_prelude, parse_font_face_prelude, parse_keyframes_prelude,
    parse_layer_prelude, parse_namespace_prelude, parse_nested_group_rules, parse_page_prelude,
    parse_scope_prelude, parse_scoped_rule_list, queries, selectors, structural_recovery_action,
    structural_rule_diagnostic, when, with_container_prelude_context, with_scope_prelude_context,
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
use cssparser::ParseError;
use surgeist_css_parser::Offset;
use surgeist_css_parser::grammar::{
    AtRule, BlockGrammar, QualifiedRule, RuleEnd, RuleGrammar, UnitKind,
};
use surgeist_css_parser::match_ignore_ascii_case;
use surgeist_css_parser::stream::{StopToken, StopTokens};

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

/// Where the stream `input` stands, as the native errors say it.
fn here(input: &Stream<'_>, recovery: &RecoveryState) -> cssparser::SourceLocation {
    location_at(recovery.source_snapshot(), offset_of(input.site()))
}

fn reject_with_style_ancestor<'i>(
    context: RuleListContext,
    input: &Stream<'i>,
    recovery: &RecoveryState,
    name: &str,
) -> Result<(), NativeError<'i>> {
    if context.style_ancestor() {
        Err(invalid_at_rule_placement(
            here(input, recovery),
            name,
            WITHOUT_STYLE_ANCESTOR,
        ))
    } else {
        Ok(())
    }
}

/// Parses one at-rule prelude and decides its admission in `context`.
/// `phase` is the stylesheet's import/namespace ordering state, present only
/// in [`RuleListContext::Stylesheet`]. `input` is the prelude the driver
/// delimited.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per admitted at-rule keeps the single prelude table visible"
)]
pub(super) fn parse_at_rule_prelude<'i>(
    context: RuleListContext,
    phase: Option<TopLevelPreludePhase>,
    source: &str,
    name: &'i str,
    input: &mut Stream<'i>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<AtRulePrelude, NativeError<'i>> {
    match_ignore_ascii_case! { name,
        "import" => {
            let Some(phase) = phase else {
                return Err(invalid_at_rule_placement(
                    here(input, recovery),
                    "import",
                    STYLESHEET_TOP_LEVEL,
                ));
            };
            if !phase.accepts_import() {
                return Err(invalid_at_rule_placement(
                    here(input, recovery),
                    "import",
                    "before every non-import top-level rule",
                ));
            }
            let prelude = parse_import_prelude(source, input, recovery).map_err(
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
                    here(input, recovery),
                    "namespace",
                    STYLESHEET_TOP_LEVEL,
                ));
            };
            if phase.after_namespace().is_none() {
                return Err(invalid_at_rule_placement(
                    here(input, recovery),
                    "namespace",
                    "after initial layer statements and imports, before later layers or body rules",
                ));
            }
            let prelude = parse_namespace_prelude(input, recovery.source_snapshot()).map_err(|error| {
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
            reject_with_style_ancestor(context, input, recovery, "counter-style")?;
            Ok(AtRulePrelude::CounterStyle(parse_counter_style_prelude(
                source,
                input,
                recovery.source_snapshot(),
            )?))
        },
        "page" => {
            if context.style_ancestor() || context.scope_body() {
                return Err(invalid_at_rule_placement(
                    here(input, recovery),
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
            reject_with_style_ancestor(context, input, recovery, "custom-media")?;
            Ok(AtRulePrelude::CustomMedia(Box::new(parse_custom_media_prelude(
                source, input, recovery,
            )?)))
        },
        "font-feature-values" => {
            reject_with_style_ancestor(context, input, recovery, "font-feature-values")?;
            Ok(AtRulePrelude::FontFeatureValues(font_feature_values::parse_families(
                source, input, recovery,
            )?))
        },
        "font-palette-values" => {
            reject_with_style_ancestor(context, input, recovery, "font-palette-values")?;
            Ok(AtRulePrelude::FontPaletteValues(font_palette_values::parse_name(
                source, input, recovery,
            )?))
        },
        "color-profile" => {
            reject_with_style_ancestor(context, input, recovery, "color-profile")?;
            Ok(AtRulePrelude::ColorProfile(color_profile::parse_name(
                source, input, recovery,
            )?))
        },
        "font-face" => {
            reject_with_style_ancestor(context, input, recovery, "font-face")?;
            parse_font_face_prelude(input, recovery)?;
            Ok(AtRulePrelude::FontFace)
        },
        "keyframes" => {
            reject_with_style_ancestor(context, input, recovery, "keyframes")?;
            Ok(AtRulePrelude::Keyframes(parse_keyframes_prelude(
                input,
                recovery.source_snapshot(),
            )?))
        },
        "layer" => Ok(AtRulePrelude::Layer(parse_layer_prelude(input, recovery.source_snapshot()).map_err(|error| {
            with_at_rule_prelude_context(
                error,
                "layer",
                "baseline.rule.layer-block",
                "a supported @layer prelude",
            )
        })?)),
        "media" => {
            let query = parse_media_query_list_inner(source, input, diagnostics, recovery)?;
            if !input.only_whitespace_remains() {
                return Err(with_media_query_context(
                    invalid_syntax(here(input, recovery)),
                    None,
                ));
            }
            Ok(AtRulePrelude::Media(query))
        },
        "supports" => Ok(AtRulePrelude::Supports(
            parse_supports_condition(source, input, recovery)
                .map_err(with_supports_prelude_context)?,
        )),
        "supports-condition" => Ok(AtRulePrelude::SupportsCondition(
            named_supports::parse_prelude(input, recovery)?,
        )),
        "when" => {
            let (condition, implicit) =
                when::parse_prelude(source, input, recovery, name, false)?;
            Ok(AtRulePrelude::When(condition.expect("required condition"), implicit))
        },
        "else" => {
            let (condition, implicit) =
                when::parse_prelude(source, input, recovery, name, true)?;
            Ok(AtRulePrelude::Else(condition, implicit))
        },
        "container" => {
            let prelude = parse_container_prelude(source, input, recovery)
                .map_err(with_container_prelude_context)?;
            if !input.only_whitespace_remains() {
                return Err(with_at_rule_prelude_context(
                    invalid_syntax(here(input, recovery)),
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
        _ => Err(ParseError {
            kind: cssparser::ParseErrorKind::Basic(cssparser::BasicParseErrorKind::AtRuleInvalid(
                cssparser::CowRcStr::from(name),
            )),
            location: here(input, recovery),
        }),
    }
}

/// Per-unit observations a grammar records for its recovery step.
pub(crate) struct UnitState {
    pub(super) boundary: nesting::StyleRuleBoundary,
    /// A typed resource failure from a qualified-rule attempt, which a
    /// declaration fallback would otherwise discard.
    pub(super) qualified_resource_error: Option<ParseError<'static, CssError>>,
    /// The resource failure found in the block of a rule whose prelude failed,
    /// which is what entering the block would have been charged.
    pub(super) block_error: Option<ParseError<'static, CssError>>,
    /// The prelude of a rule that has a block failed, so the block was left
    /// pending.
    pub(super) prelude_failed: bool,
    /// The index in its list of the first node of the unit the grammar was
    /// handed.
    pub(super) start_index: Option<usize>,
}

impl UnitState {
    pub(super) const fn new() -> Self {
        Self {
            boundary: nesting::StyleRuleBoundary::None,
            qualified_resource_error: None,
            block_error: None,
            prelude_failed: false,
            start_index: None,
        }
    }

    /// Records a qualified-rule attempt's resource failure from its prelude or
    /// block.
    pub(super) fn record_qualified_failure<T>(&mut self, result: &Result<T, NativeError<'_>>) {
        if let Err(error) = result
            && crate::diagnostic::error::is_nesting_limit_error(error)
            && let cssparser::ParseErrorKind::Custom(custom) = &error.kind
        {
            self.qualified_resource_error = Some(ParseError {
                kind: cssparser::ParseErrorKind::Custom(custom.clone()),
                location: error.location,
            });
        }
    }
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

/// A grammar of a rule list: the parser crate's block grammar, and what the
/// recovery loop reads of it and asks of it.
pub(crate) trait RuleListGrammar<'i>:
    BlockGrammar<'i, Attached<'i>, Error = NativeError<'i>>
{
    /// What the grammar recorded of the unit it was last handed.
    fn unit(&mut self) -> &mut UnitState;
    fn recovery(&self) -> &RecoveryState;
    fn diagnostics(&mut self) -> &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>;
    /// Admits the at-rule `name` whose prelude and block the driver delimited:
    /// the grammar's at-rule callback, and the way the recovery asks for a rule
    /// whose block the document denied.
    fn admit_at_rule(
        &mut self,
        name: &'i str,
        start: Site<'i>,
        rule_end: RuleEnd,
        end: Site<'i>,
        prelude: &mut Stream<'i>,
        block: RuleBlock<'i>,
    ) -> Result<Self::Item, NativeError<'i>>;
    /// Admits a qualified rule, as [`admit_at_rule`](Self::admit_at_rule) does.
    fn admit_qualified(
        &mut self,
        start: Site<'i>,
        prelude: &mut Stream<'i>,
        block: RuleBlock<'i>,
    ) -> Result<Self::Item, NativeError<'i>>;
    /// Reads the prelude of a qualified rule that met no block, for what
    /// reading it records.
    fn read_prelude(&mut self, _prelude: &mut Stream<'i>) {}
    /// What a dropped unit with the text `text` is reported as dropped.
    fn drop_action(&self, text: &str) -> crate::diagnostic::CssRecoveryAction {
        structural_recovery_action(text)
    }
}

/// How a rule list's units are delimited.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuleListKind {
    /// A stylesheet's contents (Syntax 3 (2026 CRD) §5.5.1): a stray `;` or `}`
    /// begins a qualified rule, and a custom-property look-alike returns
    /// nothing.
    Stylesheet,
    /// A block's contents whose grammar admits rules and no declaration
    /// (§5.5.5): a declaration attempt is never valid, so every unit is read as
    /// a rule.
    Rules,
    /// A block's contents that try each unit as a declaration before a nested
    /// rule (§5.5.5).
    Declarations,
}

/// The unit a rule list's driver reads next, and the stream after it.
type NextUnit<'i, I> = (Option<Result<I, Failure<'i>>>, Stream<'i>);

fn next_unit<'i, G: RuleListGrammar<'i>>(
    grammar: &mut G,
    stream: Stream<'i>,
    kind: RuleListKind,
) -> NextUnit<'i, G::Item> {
    match kind {
        RuleListKind::Stylesheet => {
            let mut units = stream.stylesheet_contents(grammar);
            let unit = units.next();
            (unit, units.stream())
        }
        RuleListKind::Rules | RuleListKind::Declarations => {
            let mut units = stream.block_contents(grammar);
            let unit = units.next();
            (unit, units.stream())
        }
    }
}

/// The single rule-list recovery loop: each unit is admitted by `grammar`,
/// retained into `sink` or dropped with one diagnostic spanning exactly that
/// unit, and the loop always makes progress.
pub(crate) fn recover_rule_list<'i, G, S>(
    source: &str,
    contents: Stream<'i>,
    kind: RuleListKind,
    grammar: &mut G,
    sink: &mut S,
) where
    G: RuleListGrammar<'i>,
    S: RuleListSink<G::Item>,
{
    let snapshot = grammar.recovery().source_snapshot().clone();
    let mut stream = contents;
    let mut refusal_reported = false;
    loop {
        let before = stream.clone();
        let progress = RecoveryProgress::record(Offset::new(offset_of(before.site())));
        *grammar.unit() = UnitState::new();
        let (next, after) = next_unit(grammar, stream, kind);
        stream = after;
        let after_offset = Offset::new(offset_of(stream.site()));
        let Some(unit) = next else {
            if kind == RuleListKind::Stylesheet {
                report_returned_nothing(source, grammar, &before, usize::MAX);
            }
            break;
        };
        let boundary = grammar.unit().boundary;
        let failed = match unit {
            Ok(item) => {
                if kind == RuleListKind::Stylesheet
                    && let Some(index) = grammar.unit().start_index
                {
                    report_returned_nothing(source, grammar, &before, index);
                }
                let outcome = progress.finish(after_offset, true);
                if boundary.partitions(false) {
                    sink.partition();
                }
                sink.retain(item, source, after_offset.value(), grammar.diagnostics());
                if outcome == RecoveryLoopOutcome::Terminated {
                    break;
                }
                continue;
            }
            Err(failed) => failed,
        };
        refusal_reported |= failed.start().document().is_refused();
        if kind == RuleListKind::Stylesheet {
            report_returned_nothing(source, grammar, &before, failed.start().index());
        }
        let facts = read_failure(source, &snapshot, kind, grammar, &before, &failed);
        let mut unit_state = std::mem::replace(grammar.unit(), UnitState::new());
        let outcome = progress.finish(after_offset, false);
        if unit_state.boundary.partitions(facts.at_block) {
            sink.partition();
        }
        let declaration_unit = kind == RuleListKind::Declarations
            && is_declaration_recovery_unit(facts.text)
            && !facts.at_block
            && unit_state.qualified_resource_error.is_none();
        if declaration_unit {
            if let Some(diagnostic) = block_item_diagnostic(
                source,
                facts.error,
                facts.text,
                facts.unit_end,
                crate::diagnostic::CssRecoveryAction::DropDeclaration,
            ) {
                grammar.diagnostics().push(diagnostic);
            }
        } else {
            let error = if let Some(resource) = unit_state
                .qualified_resource_error
                .take()
                .or(unit_state.block_error.take())
            {
                resource
            } else {
                facts.error
            };
            if let Some(diagnostic) = structural_rule_diagnostic(
                source,
                error,
                facts.text,
                facts.unit_start,
                facts.unit_end,
                grammar.drop_action(facts.text),
            ) {
                grammar.diagnostics().push(diagnostic);
            }
        }
        if outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }
    // A refusal that cut the list short without a unit to carry it.
    if stream.is_refused()
        && !refusal_reported
        && let Some(diagnostic) = crate::parser::block_units::refusal_diagnostic(
            &snapshot,
            &stream,
            grammar.drop_action(""),
        )
    {
        grammar.diagnostics().push(diagnostic);
    }
}

/// What the recovery of a dropped unit reads: the failure the grammar's
/// callbacks would have returned for it, and where they would have left the
/// unit.
struct Facts<'i, 's> {
    error: NativeError<'i>,
    /// The text of the unit up to where the callbacks left it.
    text: &'s str,
    unit_start: usize,
    unit_end: usize,
    /// Whether the unit's block was left pending: the prelude of a rule with a
    /// block failed.
    at_block: bool,
}

/// The reporting of a rule that returned nothing in a stylesheet because it
/// began like a custom property: it consumed its block and is dropped with an
/// invalid-rule error. The driver skips it without a unit, so it is found
/// between where the previous unit ended (`before`) and the node `until` of
/// the list.
fn report_returned_nothing<'i, G: RuleListGrammar<'i>>(
    source: &str,
    grammar: &mut G,
    before: &Stream<'i>,
    until: usize,
) {
    use crate::syntax_consumption::DocumentExt as _;
    let site = before.site();
    let document = site.document();
    let list = site.list();
    let nodes = document.list(list);
    let snapshot = grammar.recovery().source_snapshot().clone();
    let mut index = site.index();
    let limit = until.min(nodes.len());
    while index < limit {
        let node = nodes[index];
        if matches!(
            document.token(node),
            surgeist_css_parser::Token::Whitespace
                | surgeist_css_parser::Token::Comment
                | surgeist_css_parser::Token::Cdo
                | surgeist_css_parser::Token::Cdc
        ) {
            index += 1;
            continue;
        }
        let start = index;
        while index < nodes.len() && !document.is_curly_block(nodes[index]) {
            index += 1;
        }
        let end = (index + 1).min(nodes.len());
        let (Some(from), Some(to)) = (document.site(list, start), document.site(list, end)) else {
            return;
        };
        let (from, to) = (offset_of(from), offset_of(to));
        let error = ParseError {
            kind: cssparser::ParseErrorKind::Basic(
                cssparser::BasicParseErrorKind::QualifiedRuleInvalid,
            ),
            location: location_at(&snapshot, from),
        };
        if let Some(diagnostic) = structural_rule_diagnostic(
            source,
            error,
            &source[from..to],
            from,
            to,
            crate::diagnostic::CssRecoveryAction::DropQualifiedRule,
        ) {
            grammar.diagnostics().push(diagnostic);
        }
        index = end;
    }
}

/// Reads the facts of a failed unit of `kind`.
fn read_failure<'i, 's, G: RuleListGrammar<'i>>(
    source: &'s str,
    snapshot: &crate::source::CssSourceSnapshot,
    kind: RuleListKind,
    grammar: &mut G,
    before: &Stream<'i>,
    failed: &Failure<'i>,
) -> Facts<'i, 's> {
    use crate::syntax_consumption::DocumentExt as _;
    let dropped = dropped_unit(snapshot, source, failed);
    let unit_start = dropped.start;
    // A block the document denied for nesting too deep was never shown to the
    // grammar: the prelude is read, and the block refused, here.
    if failed.error().is_denied_block() {
        return denied_facts(grammar, before, failed, &dropped);
    }
    match failed.kind() {
        UnitKind::AtRule | UnitKind::QualifiedRule => {
            let (text, at_block) = if kind == RuleListKind::Declarations
                && grammar.unit().prelude_failed
                && dropped.block_contents.is_some()
            {
                if failed.kind() == UnitKind::AtRule {
                    // The prelude was rejected before the block.
                    (dropped.text_to_prelude_end(), true)
                } else {
                    (dropped.text_to_block().0, true)
                }
            } else if kind == RuleListKind::Declarations
                && failed.kind() == UnitKind::AtRule
                && grammar.unit().prelude_failed
            {
                // The prelude was rejected before the `;` that ended it.
                (dropped.text_to_prelude_end(), false)
            } else {
                (dropped.text(), false)
            };
            let error = if kind != RuleListKind::Declarations
                || (at_block && failed.kind() == UnitKind::QualifiedRule)
            {
                // The unit was consumed as a nested qualified rule whose
                // prelude failed (or is no declaration at all): that is the
                // failure of the dropped rule.
                native_error(snapshot, failed.error())
            } else {
                dropped.error.clone()
            };
            // A qualified rule that met the end of its list read the prelude as
            // well: the grammar did not see the unit.
            if failed.kind() == UnitKind::QualifiedRule
                && failed.rule_end() != Some(RuleEnd::Block)
                && kind != RuleListKind::Declarations
            {
                replay_prelude(grammar, before, failed.start(), StopToken::OpenCurly.into());
            }
            Facts {
                error,
                text,
                unit_start,
                unit_end: dropped.end,
                at_block,
            }
        }
        // A unit that met no block: a declaration, or what a later edition of the
        // parser crate adds.
        _ => {
            let start = failed.start();
            let end = failed.end();
            let document = start.document();
            let list = start.list();
            let nodes = document.list(list);
            let returned_nothing = nodes
                .get(start.index()..end.index())
                .unwrap_or_default()
                .iter()
                .any(|node| document.is_curly_block(*node));
            let semicolon_consumed = end.index() > 0
                && nodes.get(end.index() - 1).is_some_and(|node| {
                    matches!(document.token(*node), surgeist_css_parser::Token::Semicolon)
                });
            match kind {
                RuleListKind::Declarations => {
                    // The unit was tried as a declaration and then as a
                    // qualified rule that met no block: the prelude was read.
                    // A custom-property look-alike returns nothing without its
                    // prelude being read (Syntax 3 §5.5.3).
                    let custom = crate::syntax_consumption::custom_property_rule_prelude(
                        document,
                        &nodes[start.index()..end.index()],
                    );
                    if !returned_nothing && !custom {
                        replay_prelude(
                            grammar,
                            before,
                            start,
                            StopToken::Semicolon | StopToken::OpenCurly,
                        );
                    }
                    Facts {
                        error: dropped.error.clone(),
                        text: dropped.text(),
                        unit_start,
                        unit_end: dropped.end,
                        at_block: false,
                    }
                }
                _ => {
                    // A rule list admits no declaration: the unit was read as a
                    // qualified rule that a `;` stopped, that met the end of
                    // its list, or that began like a custom property.
                    let before_semicolon = if semicolon_consumed {
                        document
                            .site(list, end.index() - 1)
                            .map_or_else(|| offset_of(end), offset_of)
                    } else {
                        offset_of(end)
                    };
                    let (error, unit_end) = if returned_nothing {
                        (
                            ParseError {
                                kind: cssparser::ParseErrorKind::Basic(
                                    cssparser::BasicParseErrorKind::QualifiedRuleInvalid,
                                ),
                                location: location_at(snapshot, unit_start),
                            },
                            before_semicolon,
                        )
                    } else if failed.rule_end() == Some(RuleEnd::Semicolon) {
                        (
                            unexpected_syntax_token(
                                location_at(snapshot, offset_of(end)),
                                surgeist_css_parser::Token::Semicolon,
                            ),
                            offset_of(end),
                        )
                    } else {
                        replay_prelude(grammar, before, start, StopToken::OpenCurly.into());
                        (
                            unexpected_syntax_end(location_at(snapshot, offset_of(end))),
                            offset_of(end),
                        )
                    };
                    Facts {
                        error,
                        text: &source[unit_start..unit_end],
                        unit_start,
                        unit_end,
                        at_block: false,
                    }
                }
            }
        }
    }
}

/// The error of the one rule of a detached source whose block the document
/// denied for nesting too deep: the grammar reads its prelude and refuses its
/// block.
pub(super) fn denied_rule_error<'i, G: RuleListGrammar<'i>>(
    source: &str,
    grammar: &mut G,
    before: &Stream<'i>,
    failed: &Failure<'i>,
) -> NativeError<'i> {
    let snapshot = grammar.recovery().source_snapshot().clone();
    let dropped = dropped_unit(&snapshot, source, failed);
    denied_facts(grammar, before, failed, &dropped).error
}

/// The facts of a rule whose block the document denied for nesting too deep.
fn denied_facts<'i, 's, G: RuleListGrammar<'i>>(
    grammar: &mut G,
    before: &Stream<'i>,
    failed: &Failure<'i>,
    dropped: &crate::parser::block_units::DroppedUnit<'s, 'i>,
) -> Facts<'i, 's> {
    let start = failed.start();
    let end = failed.end();
    // The block the document denied ends the rule; its opening is where the
    // rule ends if the document does not know it.
    let opening = block_opening(end).unwrap_or_else(|| offset_of(end));
    let mut stream = before.clone();
    while stream.site().index() < start.index() && stream.consume_token_with_comments().is_ok() {}
    let name = match start.token() {
        Some(surgeist_css_parser::Token::AtKeyword(name)) => Some(name),
        _ => None,
    };
    if name.is_some() {
        // The at-keyword is the rule's name; its prelude follows.
        let _ = stream.consume_token_with_comments();
    }
    let mut result = None;
    let _ = stream.within_list_until(StopToken::Semicolon | StopToken::OpenCurly, |prelude| {
        result = Some(match name {
            Some(name) => grammar
                .admit_at_rule(
                    name,
                    start,
                    failed.rule_end().unwrap_or(RuleEnd::Block),
                    end,
                    prelude,
                    RuleBlock::Denied { opening },
                )
                .map(drop),
            None => grammar
                .admit_qualified(start, prelude, RuleBlock::Denied { opening })
                .map(drop),
        });
        Ok::<(), SyntaxError<'i, ()>>(())
    });
    let error = result
        .and_then(Result::err)
        .unwrap_or_else(|| native_error(grammar.recovery().source_snapshot(), failed.error()));
    let at_block = grammar.unit().prelude_failed;
    Facts {
        error,
        text: if at_block {
            dropped.text_to_block().0
        } else {
            dropped.text()
        },
        unit_start: dropped.start,
        unit_end: dropped.end,
        at_block,
    }
}

/// Reads again, for what reading it records, the prelude of a qualified rule
/// that met no block.
fn replay_prelude<'i, G: RuleListGrammar<'i>>(
    grammar: &mut G,
    before: &Stream<'i>,
    start: Site<'i>,
    stops: StopTokens,
) {
    let mut stream = before.clone();
    while stream.site().index() < start.index() && stream.consume_token_with_comments().is_ok() {}
    let _ = stream.within_list_until(stops, |prelude| {
        grammar.read_prelude(prelude);
        Ok::<(), SyntaxError<'i, ()>>(())
    });
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

/// The error of a block that an at-rule whose body is a statement refuses,
/// located at the block's opening token.
fn block_refused<'i>(
    recovery: &RecoveryState,
    contents: usize,
    name: &str,
    production: &'static str,
    expectation: &'static str,
) -> NativeError<'i> {
    let snapshot = recovery.source_snapshot();
    let position =
        crate::source::CssSourcePosition::from_byte_offset_in(snapshot.as_str(), contents)
            .previous_ascii_byte();
    let mut location = location_at(snapshot, contents);
    location.column = location.column.saturating_sub(1).max(1);
    invalid_at_rule_body_at(position, location, name, production, expectation)
}

/// Parses the block of a leaf rule, or rejects the block of a statement-only
/// rule. A group prelude is returned for its list's own body grammar.
// Keep descriptor parsing off the recursive group-rule frame. Deep rule
// lists run on the test thread's ordinary stack.
#[inline(never)]
pub(super) fn parse_leaf_block<'i>(
    prelude: AtRulePrelude,
    source: &str,
    start: Site<'i>,
    mut contents: Stream<'i>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<Result<LeafRule, AtRulePrelude>, NativeError<'i>> {
    let refused = |name, production, expectation| {
        Err(block_refused(
            recovery,
            offset_of(contents.start_site()),
            name,
            production,
            expectation,
        ))
    };
    let position = recovery.source_position(offset_of(start));
    let rule = match prelude {
        AtRulePrelude::CustomMedia(_) => {
            return refused(
                "custom-media",
                "ext.rule.custom-media",
                "a statement-form custom-media rule",
            );
        }
        AtRulePrelude::Import(_) => {
            return refused(
                "import",
                "baseline.rule.import",
                "a semicolon-terminated @import rule",
            );
        }
        AtRulePrelude::Namespace(_) => {
            return refused(
                "namespace",
                "later.rule.namespace",
                "a semicolon-terminated @namespace rule",
            );
        }
        AtRulePrelude::CounterStyle(name) => LeafRule::CounterStyle(parse_counter_style_rule(
            source,
            name,
            &mut contents,
            position,
            diagnostics,
            recovery.clone(),
        )),
        AtRulePrelude::Page(selector) => LeafRule::Page(parse_page_rule(
            source,
            selector,
            &mut contents,
            position,
            diagnostics,
            recovery.clone(),
        )),
        AtRulePrelude::FontFeatureValues(families) => {
            LeafRule::FontFeatureValues(font_feature_values::parse_rule(
                source,
                families,
                &mut contents,
                position,
                diagnostics,
                recovery.clone(),
            )?)
        }
        AtRulePrelude::FontPaletteValues(name) => {
            LeafRule::FontPaletteValues(font_palette_values::parse_rule(
                source,
                name,
                &mut contents,
                position,
                diagnostics,
                recovery.clone(),
            )?)
        }
        AtRulePrelude::ColorProfile(name) => LeafRule::ColorProfile(color_profile::parse_rule(
            source,
            name,
            &mut contents,
            position,
            diagnostics,
            recovery.clone(),
        )),
        AtRulePrelude::FontFace => LeafRule::FontFace(parse_font_face_rule(
            source,
            &mut contents,
            position,
            diagnostics,
            recovery.clone(),
        )),
        AtRulePrelude::Keyframes(name) => LeafRule::Keyframes(parse_keyframes_rule(
            source,
            name,
            &mut contents,
            position,
            diagnostics,
            recovery.clone(),
        )),
        AtRulePrelude::SupportsCondition(prelude) => {
            LeafRule::SupportsCondition(named_supports::parse_rule(
                source,
                prelude,
                start,
                &mut contents,
                diagnostics,
                recovery,
            )?)
        }
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

/// The block of a rule as its grammar sees it.
pub(crate) enum RuleBlock<'i> {
    /// The rule has no block.
    Absent,
    /// The contents of the rule's block, which opens at the byte `opening`.
    Open {
        opening: usize,
        contents: Stream<'i>,
    },
    /// A block that nested too deep for the document to keep, which opens at
    /// the byte `opening`.
    Denied { opening: usize },
}

impl RuleBlock<'_> {
    const fn opening(&self) -> Option<usize> {
        match self {
            Self::Absent => None,
            Self::Open { opening, .. } | Self::Denied { opening } => Some(*opening),
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
    pub(super) unit: UnitState,
    /// Whether the last rule the grammar was handed was admitted.
    pub(super) completed: bool,
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
            completed: false,
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

    fn position(&self, start: Site<'_>) -> crate::source::CssSourcePosition {
        self.recovery.source_position(offset_of(start))
    }

    // A group rule's body: block contents with this list's style ancestry.
    // Kept out of `parse_block`'s recursive frame.
    #[inline(never)]
    fn group_body(&mut self, contents: Stream<'_>) -> Vec<CssRule> {
        if self.scope.is_some() {
            let recovered = parse_scoped_rule_list(
                self.source,
                contents,
                self.recovery.clone(),
                self.style,
                ScopedBodyKind::OrdinaryGroup,
            );
            self.diagnostics.extend(recovered.diagnostics);
            recovered.syntax
        } else if self.style {
            let recovered = parse_style_contents(self.source, contents, self.recovery.clone());
            self.diagnostics.extend(recovered.diagnostics);
            recovered.syntax.into_nested_rules()
        } else {
            let recovered = parse_nested_group_rules(self.source, contents, self.recovery.clone());
            self.diagnostics.extend(recovered.diagnostics);
            recovered.syntax
        }
    }

    #[inline(never)]
    fn scope_body(&mut self, contents: Stream<'_>) -> Vec<CssRule> {
        let recovered = parse_scoped_rule_list(
            self.source,
            contents,
            self.recovery.clone(),
            self.style,
            ScopedBodyKind::Scope,
        );
        self.diagnostics.extend(recovered.diagnostics);
        recovered.syntax
    }

    /// Admits the at-rule `name`, whose prelude is `prelude` and whose block
    /// is `block`.
    pub(super) fn admit_at_rule<'i>(
        &mut self,
        name: &'i str,
        start: Site<'i>,
        rule_end: RuleEnd,
        end: Site<'i>,
        prelude: &mut Stream<'i>,
        block: RuleBlock<'i>,
    ) -> Result<RuleListItem, NativeError<'i>> {
        self.unit.boundary = nesting::StyleRuleBoundary::AtRule;
        self.unit.start_index = Some(start.index());
        let parsed = parse_at_rule_prelude(
            self.context(),
            self.top_level_phase,
            self.source,
            name,
            prelude,
            &mut self.diagnostics,
            &self.recovery,
        );
        let snapshot = self.recovery.source_snapshot().clone();
        let Some(opening) = block.opening() else {
            let prelude = match parsed {
                Ok(prelude) => prelude,
                Err(error) => {
                    self.unit.prelude_failed = true;
                    return Err(error);
                }
            };
            let rules = self.rule_without_block(prelude, start, rule_end, end);
            return match rules {
                Ok(rules) => {
                    self.completed = true;
                    Ok(RuleListItem::Rules(rules))
                }
                Err(()) if rule_end == RuleEnd::Semicolon => Err(unexpected_syntax_token(
                    location_at(&snapshot, offset_of(end)),
                    surgeist_css_parser::Token::Semicolon,
                )),
                Err(()) => Err(unexpected_syntax_end(location_at(
                    &snapshot,
                    offset_of(end),
                ))),
            };
        };
        let prelude = match parsed {
            Ok(prelude) => prelude,
            Err(error) => {
                self.unit.prelude_failed = true;
                self.unit.block_error = self.recovery.check_failed_rule_block_at(
                    self.source,
                    opening + 1,
                    "css.at-rule",
                );
                return Err(error);
            }
        };
        let result = self.parse_block(prelude, start, opening, block);
        self.completed = result.is_ok();
        result
    }

    fn rule_without_block<'i>(
        &mut self,
        prelude: AtRulePrelude,
        start: Site<'i>,
        rule_end: RuleEnd,
        end: Site<'i>,
    ) -> Result<Vec<CssRule>, ()> {
        let result = match prelude {
            AtRulePrelude::CustomMedia(prelude) => {
                let Some(rule) =
                    finish_custom_media(*prelude, start, &self.recovery, &mut self.diagnostics)
                else {
                    return Err(());
                };
                self.mark_successful_body_rule();
                Ok(vec![CssRule::CustomMedia(rule)])
            }
            AtRulePrelude::Import(prelude) => {
                let Some(at_keyword) = collect_at_keyword(start) else {
                    return Err(());
                };
                self.diagnostics.extend(prelude.diagnostics);
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
        if result.is_ok() {
            retain_statement_eof(self.source, rule_end, start, end, &mut self.diagnostics);
        }
        result
    }

    fn parse_block<'i>(
        &mut self,
        prelude: AtRulePrelude,
        start: Site<'i>,
        opening: usize,
        block: RuleBlock<'i>,
    ) -> Result<RuleListItem, NativeError<'i>> {
        let mut depth = self.recovery.enter_rule_block(
            self.source,
            Offset::new(opening),
            prelude.production(),
        )?;
        let RuleBlock::Open { contents, .. } = block else {
            // The document kept no contents for the block, so the structural
            // ceiling is the failure.
            return Err(crate::diagnostic::error::nesting_limit(
                self.source,
                opening,
                crate::parser::recovery::STRUCTURAL_NESTING_LIMIT,
                prelude.production(),
            ));
        };
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
            contents.clone(),
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
        // Positions are found in source order: the rule's own, then its body's.
        let position = self.position(start);
        let rules = match prelude {
            AtRulePrelude::Layer(names) => {
                if names.len() > 1 {
                    return Err(block_refused(
                        &self.recovery,
                        offset_of(contents.start_site()),
                        "layer",
                        "baseline.rule.layer-block",
                        "at most one layer name before a block",
                    ));
                }
                let name = names.into_iter().next();
                let rules = self.group_body(contents);
                vec![CssRule::LayerBlock(CssLayerBlockRule::new(
                    name, rules, position,
                ))]
            }
            AtRulePrelude::Media(query) => {
                let rules = self.group_body(contents);
                vec![CssRule::Media(CssMediaRule::new(query, rules, position))]
            }
            AtRulePrelude::Supports(condition) => {
                let rules = self.group_body(contents);
                vec![CssRule::Supports(CssSupportsRule::new(
                    condition, rules, position,
                ))]
            }
            AtRulePrelude::When(condition, implicit) => {
                let rules = self.group_body(contents);
                self.recovery.retain_component_closures(implicit);
                when::assemble_when_rule(condition, rules, position)
            }
            AtRulePrelude::Else(condition, implicit) => {
                let rules = self.group_body(contents);
                self.recovery.retain_component_closures(implicit);
                when::assemble_else_rule(condition, rules, position)
            }
            AtRulePrelude::Container(prelude) => {
                let rules = self.group_body(contents);
                vec![CssRule::Container(CssContainerRule::new(
                    prelude, rules, position,
                ))]
            }
            AtRulePrelude::Scope(prelude) => {
                let rules = self.scope_body(contents);
                vec![CssRule::Scope(CssScopeRule::new(
                    prelude.root,
                    prelude.limit,
                    rules,
                    position,
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

    /// Reads the prelude of a qualified rule as selectors.
    pub(super) fn qualified_prelude<'i>(
        &mut self,
        input: &mut Stream<'i>,
    ) -> Result<CssStyleSelectorList, NativeError<'i>> {
        self.unit.boundary = nesting::StyleRuleBoundary::QualifiedPrelude;
        let mut recovery =
            SelectorRecovery::new(self.source, &mut self.diagnostics, self.recovery.clone());
        if self.scope.is_some() {
            // A scope body's selectors keep scope anchors, and are relative to the scope.
            let result = parse_scoped_style_selector_list_stream(input, &mut recovery);
            self.unit.record_qualified_failure(&result);
            return result;
        }
        if !self.style {
            return parse_rule_selector_list_stream(input, &mut recovery)
                .map(CssStyleSelectorList::absolute);
        }
        let result = selectors::parse_nested_style_selector_list_stream(input, &mut recovery)
            .map(CssStyleSelectorList::new);
        self.unit.record_qualified_failure(&result);
        result
    }

    /// Admits a qualified rule whose prelude is `prelude` and whose block is
    /// `block`.
    pub(super) fn admit_qualified<'i>(
        &mut self,
        start: Site<'i>,
        prelude: &mut Stream<'i>,
        block: RuleBlock<'i>,
    ) -> Result<RuleListItem, NativeError<'i>> {
        self.unit.start_index = Some(start.index());
        let parsed = self.qualified_prelude(prelude);
        let Some(opening) = block.opening() else {
            // A qualified rule ends at its block; without one the rule ended
            // where its prelude did.
            return Err(unexpected_syntax_end(location_at(
                self.recovery.source_snapshot(),
                offset_of(start),
            )));
        };
        let selectors = match parsed {
            Ok(selectors) => selectors,
            Err(error) => {
                self.unit.prelude_failed = true;
                self.unit.block_error = self.recovery.check_failed_rule_block_at(
                    self.source,
                    opening + 1,
                    "css.qualified-rule",
                );
                return Err(error);
            }
        };
        self.unit.boundary = nesting::StyleRuleBoundary::QualifiedBlock;
        let result = self.qualified_block(selectors, start, opening, block);
        if self.style || self.scope.is_some() {
            self.unit.record_qualified_failure(&result);
        }
        self.completed = result.is_ok();
        result
    }

    fn qualified_block<'i>(
        &mut self,
        selectors: CssStyleSelectorList,
        start: Site<'i>,
        opening: usize,
        block: RuleBlock<'i>,
    ) -> Result<RuleListItem, NativeError<'i>> {
        let mut depth = self.recovery.enter_rule_block(
            self.source,
            Offset::new(opening),
            "baseline.rule.style",
        )?;
        let RuleBlock::Open { contents, .. } = block else {
            return Err(crate::diagnostic::error::nesting_limit(
                self.source,
                opening,
                crate::parser::recovery::STRUCTURAL_NESTING_LIMIT,
                "baseline.rule.style",
            ));
        };
        let recovered = parse_style_rule_block(
            self.source,
            selectors,
            self.position(start),
            contents,
            self.recovery.clone(),
        );
        self.diagnostics.extend(recovered.diagnostics);
        self.mark_successful_body_rule();
        depth.retain();
        Ok(RuleListItem::Rules(recovered.syntax))
    }

    /// The resource failure found in the block of the rule that failed, which
    /// is what entering the block would have been charged. It takes
    /// precedence over the rule's own failure.
    pub(super) fn take_block_resource_error<'i>(&mut self) -> Option<NativeError<'i>> {
        self.unit
            .qualified_resource_error
            .take()
            .or_else(|| self.unit.block_error.take())
    }

    /// Admits the declaration named `name` that lies at `text`.
    pub(super) fn admit_declaration<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<RuleListItem, NativeError<'i>> {
        StrictDeclarationParser::new(self.source, self.recovery.clone())
            .admit(name, text)
            .map(Box::new)
            .map(RuleListItem::Declaration)
    }
}

/// The block of a rule that ends at `end`, whose contents are `contents` when it
/// has a block.
pub(super) fn rule_block_of<'i>(contents: Option<Stream<'i>>, end: Site<'i>) -> RuleBlock<'i> {
    match (contents, block_opening(end)) {
        (Some(contents), Some(opening)) => RuleBlock::Open { opening, contents },
        _ => RuleBlock::Absent,
    }
}

/// The byte where the `{}`-block that ends the rule `end` opens.
fn block_opening(end: Site<'_>) -> Option<usize> {
    let document = end.document();
    let block = *document.list(end.list()).get(end.index().checked_sub(1)?)?;
    Some(document.range(block)?.start.value())
}

impl<'i> RuleGrammar<'i, Attached<'i>> for RuleParser<'_> {
    type Item = RuleListItem;
    type Error = NativeError<'i>;

    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        let at = rule.start();
        let block = rule_block_of(rule.block(), rule.end());
        let mut prelude = rule.prelude();
        self.admit_at_rule(
            rule.name(),
            at,
            rule.rule_end(),
            rule.end(),
            &mut prelude,
            block,
        )
        .map_err(|error| grammar_error(at, error))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = rule.start();
        let block = rule_block_of(Some(rule.block()), rule.end());
        let mut prelude = rule.prelude();
        self.admit_qualified(at, &mut prelude, block)
            .map_err(|error| grammar_error(at, error))
    }
}

impl<'i> RuleListGrammar<'i> for RuleParser<'_> {
    fn unit(&mut self) -> &mut UnitState {
        &mut self.unit
    }

    fn recovery(&self) -> &RecoveryState {
        &self.recovery
    }

    fn diagnostics(&mut self) -> &mut Vec<crate::diagnostic::CssRecoveryDiagnostic> {
        &mut self.diagnostics
    }

    fn admit_at_rule(
        &mut self,
        name: &'i str,
        start: Site<'i>,
        rule_end: RuleEnd,
        end: Site<'i>,
        prelude: &mut Stream<'i>,
        block: RuleBlock<'i>,
    ) -> Result<RuleListItem, NativeError<'i>> {
        RuleParser::admit_at_rule(self, name, start, rule_end, end, prelude, block)
    }

    fn admit_qualified(
        &mut self,
        start: Site<'i>,
        prelude: &mut Stream<'i>,
        block: RuleBlock<'i>,
    ) -> Result<RuleListItem, NativeError<'i>> {
        RuleParser::admit_qualified(self, start, prelude, block)
    }

    fn read_prelude(&mut self, prelude: &mut Stream<'i>) {
        let _ = self.qualified_prelude(prelude);
    }
}

impl<'i> BlockGrammar<'i, Attached<'i>> for RuleParser<'_> {
    fn declaration(
        &mut self,
        declaration: DeclarationUnit<'i>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = declaration.start();
        if !self.style {
            // A declaration is no rule: the unit is read again as one.
            return Err(SyntaxError::invalid_qualified_rule_error(at));
        }
        self.admit_declaration(declaration.name(), &DeclarationText::of(&declaration))
            .map_err(|error| grammar_error(at, error))
    }
}

/// Collects a statement's at-keyword from its original source; none when the
/// statement does not start at one.
pub(super) fn collect_at_keyword(start: Site<'_>) -> Option<crate::component::CssComponentValue> {
    let node = start.node()?;
    crate::component::values::parse::promote_nodes(
        start.document(),
        &[node],
        crate::component::CssComponentValueLimits::default(),
    )
    .ok()
    .and_then(crate::component::CssComponentValues::into_single)
}

/// Reports the statement at-rule that the end of its list ended (Syntax 3
/// §5.5.2: this is a parse error), retained all the same.
fn retain_statement_eof(
    source: &str,
    rule_end: RuleEnd,
    start: Site<'_>,
    end: Site<'_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
) {
    if !matches!(rule_end, RuleEnd::Eof | RuleEnd::CloseBrace) {
        return;
    }
    let (from, to) = (offset_of(start), offset_of(end));
    let Some(span) = crate::source::CssSourceSpan::new(
        crate::source::CssSourcePosition::from_byte_offset_in(source, from),
        crate::source::CssSourcePosition::from_byte_offset_in(source, to),
    ) else {
        return;
    };
    if let Some(diagnostic) = crate::diagnostic::CssRecoveryDiagnostic::new(
        crate::diagnostic::error::unterminated_at_rule(source, to),
        span,
        crate::diagnostic::CssRecoveryAction::RetainNonconformingRule,
    ) {
        diagnostics.push(diagnostic);
    }
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
