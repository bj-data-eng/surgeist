//! Borrowed authored rule graph traversal under one monotonic context.
//!
//! Each logical rule charges one input and projection node: published leaf
//! writers retain their existing charge, and new graph arms charge before
//! scheduling children. Rust enum carriers and child slices are transparent.
//! Slice continuations retain only the remaining children, so graph width does
//! not allocate a pending event for every sibling before checking work limits.

use crate::cssom_rule_serialization::{
    CssRuleCssomFormat, CssRuleCssomKind, CssRuleCssomSerializationError, RuleCssomSource,
};
use crate::specified_rule_serialization::{SpecifiedRuleSerializationSource, SpecifiedRuleWriter};
use crate::{
    CssDeclarationList, CssLayerName, CssLayerNameList, CssMediaQueryList, CssRule, CssScopeRule,
    CssScopedRule, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
};

type Result<T> = std::result::Result<T, RuleCssomSource>;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Format {
    Compact,
    Cssom,
}

struct GraphFailure {
    source: RuleCssomSource,
    path: Vec<usize>,
    keyframe_block_index: Option<usize>,
}
type ValueResult<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

enum Event<'a> {
    Ordinary(&'a CssRule, Option<usize>),
    Scoped(&'a CssScopedRule, Option<usize>),
    OrdinaryChildren(&'a [CssRule], usize, bool),
    ScopedChildren(&'a [CssScopedRule], usize, bool),
    Declarations(&'a CssDeclarationList),
    Text(&'static str),
    EndRule(bool),
}

fn push<'a>(work: &mut Vec<Event<'a>>, event: Event<'a>) -> ValueResult<()> {
    work.try_reserve(1).map_err(|_| {
        CssSpecifiedValueSerializationError::new(
            CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        )
    })?;
    work.push(event);
    Ok(())
}

impl SpecifiedRuleWriter {
    /// One borrowed loop visits both rule enums without constructing a second
    /// graph, recursively walking groups, or resetting any provider budget.
    pub(crate) fn append_rule_graph(
        &mut self,
        rule: &CssRule,
    ) -> std::result::Result<(), SpecifiedRuleSerializationSource> {
        self.append_graph(rule, Format::Compact, Vec::new())
            .map_err(|error| match error.source {
                RuleCssomSource::Provider(source) => source,
                _ => unreachable!("compact traversal uses only compact providers"),
            })
    }

    pub(crate) fn append_cssom_rule_graph(
        &mut self,
        rule: &CssRule,
        path: Vec<usize>,
    ) -> std::result::Result<(), CssRuleCssomSerializationError> {
        self.append_graph(rule, Format::Cssom, path)
            .map_err(|failure| {
                CssRuleCssomSerializationError::new(
                    failure.source,
                    failure.path,
                    failure.keyframe_block_index,
                )
            })
    }

    fn append_graph(
        &mut self,
        rule: &CssRule,
        format: Format,
        mut path: Vec<usize>,
    ) -> std::result::Result<(), GraphFailure> {
        let mut work = Vec::new();
        let mut keyframe_block_index = None;
        let result =
            (|| -> Result<()> {
                push(&mut work, Event::Ordinary(rule, None))?;
                while let Some(event) = work.pop() {
                    match event {
                        Event::Text(text) => self.append(text)?,
                        Event::EndRule(pop) => {
                            if pop {
                                path.pop();
                            }
                        }
                        Event::Declarations(declarations) => {
                            if format == Format::Cssom {
                                self.append_cssom_declaration_list(declarations)?;
                            } else {
                                self.append_authored_declaration_list(declarations)?;
                            }
                        }
                        Event::OrdinaryChildren(rules, index, separator) => {
                            if let Some((first, rest)) = rules.split_first() {
                                if format == Format::Cssom {
                                    self.append("\n  ")?;
                                } else if separator {
                                    self.append(" ")?;
                                }
                                push(&mut work, Event::OrdinaryChildren(rest, index + 1, true))?;
                                push(&mut work, Event::Ordinary(first, Some(index)))?;
                            }
                        }
                        Event::ScopedChildren(rules, index, separator) => {
                            if let Some((first, rest)) = rules.split_first() {
                                if format == Format::Cssom {
                                    self.append("\n  ")?;
                                } else if separator {
                                    self.append(" ")?;
                                }
                                push(&mut work, Event::ScopedChildren(rest, index + 1, true))?;
                                push(&mut work, Event::Scoped(first, Some(index)))?;
                            }
                        }
                        Event::Ordinary(rule, index) => {
                            if format == Format::Cssom
                                && let Some(index) = index
                            {
                                path.try_reserve(1).map_err(|_| {
                                    CssSpecifiedValueSerializationError::new(
                                        CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                                    )
                                })?;
                                path.push(index);
                            }
                            if format == Format::Cssom {
                                push(&mut work, Event::EndRule(index.is_some()))?;
                            }
                            match rule {
                                CssRule::Namespace(rule) => self.namespace(rule)?,
                                CssRule::CounterStyle(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::CounterStyle,
                                        ));
                                    }
                                    rule.append_to_rule_writer(self)?
                                }
                                CssRule::FontFace(rule) => self.font_face(rule)?,
                                CssRule::FontFeatureValues(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::FontFeatureValues,
                                        ));
                                    }
                                    self.font_features(rule)?
                                }
                                CssRule::FontPaletteValues(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::FontPaletteValues,
                                        ));
                                    }
                                    self.palette(rule)?
                                }
                                CssRule::ColorProfile(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::ColorProfile,
                                        ));
                                    }
                                    self.color_profile(rule)?
                                }
                                CssRule::SupportsCondition(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::SupportsCondition,
                                        ));
                                    }
                                    self.named_supports_rule(rule)?
                                }
                                CssRule::NestedDeclarations(rule) => {
                                    self.node()?;
                                    push(&mut work, Event::Declarations(rule.declarations()))?;
                                }
                                CssRule::Page(rule) => {
                                    self.node()?;
                                    if format == Format::Cssom {
                                        return Err(RuleCssomSource::SourceUndefined(
                                            CssRuleCssomKind::Page,
                                        ));
                                    }
                                    self.page(rule)?;
                                }
                                CssRule::Keyframes(rule) => {
                                    self.node()?;
                                    if format == Format::Cssom {
                                        self.keyframes_cssom(rule, &mut keyframe_block_index)?;
                                    } else {
                                        self.keyframes(rule)?;
                                    }
                                }
                                CssRule::CustomMedia(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::CustomMedia,
                                        ));
                                    }
                                    self.node()?;
                                    rule.append_specified(&mut self.context, &mut self.css)?;
                                }
                                CssRule::Import(rule) => {
                                    self.node()?;
                                    rule.append_specified(&mut self.context, &mut self.css)?;
                                }
                                CssRule::LayerStatement(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::LayerStatement,
                                        ));
                                    }
                                    self.node()?;
                                    self.append("@layer ")?;
                                    self.layer_names(rule.names())?;
                                    self.append(";")?;
                                }
                                CssRule::LayerBlock(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::LayerBlock,
                                        ));
                                    }
                                    self.node()?;
                                    self.layer_prelude(rule.name())?;
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().is_empty()).then_some(
                                            Event::OrdinaryChildren(rule.rules(), 0, false),
                                        ),
                                    )?;
                                }
                                CssRule::Style(rule) => {
                                    self.node()?;
                                    if format == Format::Cssom {
                                        self.cssom_style_selectors(rule.selectors())?;
                                    } else {
                                        self.style_selectors(rule.selectors())?;
                                    }
                                    self.style_block(
                                        &mut work,
                                        rule.declarations(),
                                        rule.rules(),
                                        format,
                                    )?;
                                }
                                CssRule::Media(rule) => {
                                    self.node()?;
                                    self.media_prelude(rule.query(), format)?;
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().is_empty()).then_some(
                                            Event::OrdinaryChildren(rule.rules(), 0, false),
                                        ),
                                    )?;
                                }
                                CssRule::Supports(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::Supports,
                                        ));
                                    }
                                    self.node()?;
                                    self.append("@supports ")?;
                                    rule.condition()
                                        .append_specified(&mut self.context, &mut self.css)?;
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().is_empty()).then_some(
                                            Event::OrdinaryChildren(rule.rules(), 0, false),
                                        ),
                                    )?;
                                }
                                CssRule::Container(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::Container,
                                        ));
                                    }
                                    self.node()?;
                                    self.append("@container ")?;
                                    rule.prelude()
                                        .append_specified(&mut self.context, &mut self.css)?;
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().is_empty()).then_some(
                                            Event::OrdinaryChildren(rule.rules(), 0, false),
                                        ),
                                    )?;
                                }
                                CssRule::Scope(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::Scope,
                                        ));
                                    }
                                    self.scope_block(&mut work, rule, format)?
                                }
                            }
                        }
                        Event::Scoped(rule, index) => {
                            if format == Format::Cssom
                                && let Some(index) = index
                            {
                                path.try_reserve(1).map_err(|_| {
                                    CssSpecifiedValueSerializationError::new(
                                        CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                                    )
                                })?;
                                path.push(index);
                            }
                            if format == Format::Cssom {
                                push(&mut work, Event::EndRule(index.is_some()))?;
                            }
                            match rule {
                                CssScopedRule::CounterStyle(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::CounterStyle,
                                        ));
                                    }
                                    rule.append_to_rule_writer(self)?
                                }
                                CssScopedRule::FontFace(rule) => self.font_face(rule)?,
                                CssScopedRule::FontFeatureValues(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::FontFeatureValues,
                                        ));
                                    }
                                    self.font_features(rule)?
                                }
                                CssScopedRule::FontPaletteValues(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::FontPaletteValues,
                                        ));
                                    }
                                    self.palette(rule)?
                                }
                                CssScopedRule::ColorProfile(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::ColorProfile,
                                        ));
                                    }
                                    self.color_profile(rule)?
                                }
                                CssScopedRule::SupportsCondition(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::SupportsCondition,
                                        ));
                                    }
                                    self.named_supports_rule(rule)?
                                }
                                CssScopedRule::NestedDeclarations(rule) => {
                                    self.node()?;
                                    push(&mut work, Event::Declarations(rule.declarations()))?;
                                }
                                CssScopedRule::Page(rule) => {
                                    self.node()?;
                                    if format == Format::Cssom {
                                        return Err(RuleCssomSource::SourceUndefined(
                                            CssRuleCssomKind::Page,
                                        ));
                                    }
                                    self.page(rule)?;
                                }
                                CssScopedRule::Keyframes(rule) => {
                                    self.node()?;
                                    if format == Format::Cssom {
                                        self.keyframes_cssom(rule, &mut keyframe_block_index)?;
                                    } else {
                                        self.keyframes(rule)?;
                                    }
                                }
                                CssScopedRule::CustomMedia(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::CustomMedia,
                                        ));
                                    }
                                    self.node()?;
                                    rule.append_specified(&mut self.context, &mut self.css)?;
                                }
                                CssScopedRule::LayerStatement(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::LayerStatement,
                                        ));
                                    }
                                    self.node()?;
                                    self.append("@layer ")?;
                                    self.layer_names(rule.names())?;
                                    self.append(";")?;
                                }
                                CssScopedRule::LayerBlock(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::LayerBlock,
                                        ));
                                    }
                                    self.node()?;
                                    self.layer_prelude(rule.name())?;
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().rules().is_empty()).then_some(
                                            Event::ScopedChildren(rule.rules().rules(), 0, false),
                                        ),
                                    )?;
                                }
                                CssScopedRule::Style(rule) => {
                                    self.node()?;
                                    if format == Format::Cssom {
                                        self.cssom_scoped_style_selectors(rule.selectors())?;
                                    } else {
                                        self.scoped_style_selectors(rule.selectors())?;
                                    }
                                    // Scoped style children have ordinary nesting semantics.
                                    self.style_block(
                                        &mut work,
                                        rule.declarations(),
                                        rule.rules(),
                                        format,
                                    )?;
                                }
                                CssScopedRule::Media(rule) => {
                                    self.node()?;
                                    self.media_prelude(rule.query(), format)?;
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().rules().is_empty()).then_some(
                                            Event::ScopedChildren(rule.rules().rules(), 0, false),
                                        ),
                                    )?;
                                }
                                CssScopedRule::Supports(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::Supports,
                                        ));
                                    }
                                    self.node()?;
                                    self.append("@supports ")?;
                                    rule.condition()
                                        .append_specified(&mut self.context, &mut self.css)?;
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().rules().is_empty()).then_some(
                                            Event::ScopedChildren(rule.rules().rules(), 0, false),
                                        ),
                                    )?;
                                }
                                CssScopedRule::Container(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::Container,
                                        ));
                                    }
                                    self.node()?;
                                    self.append("@container ")?;
                                    rule.prelude()
                                        .append_specified(&mut self.context, &mut self.css)?;
                                    self.open_block(
                                        &mut work,
                                        format,
                                        (!rule.rules().rules().is_empty()).then_some(
                                            Event::ScopedChildren(rule.rules().rules(), 0, false),
                                        ),
                                    )?;
                                }
                                CssScopedRule::Scope(rule) => {
                                    if format == Format::Cssom {
                                        self.node()?;
                                        return Err(RuleCssomSource::FormatUnavailable(
                                            CssRuleCssomFormat::Scope,
                                        ));
                                    }
                                    self.scope_block(&mut work, rule, format)?
                                }
                            }
                        }
                    }
                }
                Ok(())
            })();
        result.map_err(|source| GraphFailure {
            source,
            path,
            keyframe_block_index,
        })
    }

    fn open_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        format: Format,
        child: Option<Event<'a>>,
    ) -> ValueResult<()> {
        self.append(" {")?;
        push(
            work,
            Event::Text(if format == Format::Cssom { "\n}" } else { " }" }),
        )?;
        if let Some(child) = child {
            if format == Format::Compact {
                self.append(" ")?;
            }
            push(work, child)?;
        } else if format == Format::Cssom {
            self.append("\n")?;
        }
        Ok(())
    }

    /// The leading declaration list charges its shared aggregate even when
    /// empty. Later runs remain logical rules at their original child position.
    fn style_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        declarations: &'a CssDeclarationList,
        children: &'a [CssRule],
        format: Format,
    ) -> Result<()> {
        if format == Format::Cssom {
            self.append(" {")?;
            if children.is_empty() {
                if !declarations.is_empty() {
                    self.append(" ")?;
                }
                self.append_cssom_declaration_list(declarations)?;
                self.append(" }")?;
            } else {
                if !declarations.is_empty() {
                    self.append("\n  ")?;
                }
                self.append_cssom_declaration_list(declarations)?;
                push(work, Event::Text("\n}"))?;
                push(work, Event::OrdinaryChildren(children, 0, false))?;
            }
            return Ok(());
        }
        self.append(" {")?;
        push(work, Event::Text(" }"))?;
        if !declarations.is_empty() || !children.is_empty() {
            self.append(" ")?;
        }
        if !children.is_empty() {
            push(
                work,
                Event::OrdinaryChildren(children, 0, !declarations.is_empty()),
            )?;
        }
        push(work, Event::Declarations(declarations))?;
        Ok(())
    }

    fn scope_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        rule: &'a CssScopeRule,
        format: Format,
    ) -> ValueResult<()> {
        self.node()?;
        self.append("@scope")?;
        if let Some(root) = rule.root() {
            self.append(" (")?;
            self.scope_selectors(root)?;
            self.append(")")?;
        }
        if let Some(limit) = rule.limit() {
            self.append(" to (")?;
            self.scope_selectors(limit)?;
            self.append(")")?;
        }
        self.open_block(
            work,
            format,
            (!rule.rules().rules().is_empty()).then_some(Event::ScopedChildren(
                rule.rules().rules(),
                0,
                false,
            )),
        )
    }

    fn media_prelude(&mut self, list: &CssMediaQueryList, format: Format) -> Result<()> {
        self.append("@media")?;
        if format == Format::Cssom || !list.queries().is_empty() {
            self.append(" ")?;
        }
        let captured = list.capture_cssom(&mut self.context)?;
        self.append(captured.as_css())?;
        Ok(())
    }

    fn layer_prelude(&mut self, name: Option<&CssLayerName>) -> ValueResult<()> {
        self.append("@layer")?;
        if let Some(name) = name {
            self.append(" ")?;
            self.layer_name(name)?;
        }
        Ok(())
    }

    /// A name charges one aggregate and each retained identifier component
    /// once. The component slice adds no extra charge.
    fn layer_name(&mut self, name: &CssLayerName) -> ValueResult<()> {
        self.node()?;
        for (index, component) in name.components().iter().enumerate() {
            if index != 0 {
                self.append(".")?;
            }
            self.node()?;
            self.append_identifier(component)?;
        }
        Ok(())
    }

    /// A layer-name list adds one aggregate before its ordered names.
    fn layer_names(&mut self, names: &CssLayerNameList) -> ValueResult<()> {
        self.node()?;
        for (index, name) in names.names().iter().enumerate() {
            if index != 0 {
                self.append(", ")?;
            }
            self.layer_name(name)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CssRule, CssSpecifiedRuleSerializationErrorKind as RuleKind,
        CssSpecifiedValueSerializationErrorKind as Kind,
        CssSpecifiedValueSerializationLimits as Limits, parse_sheet,
    };

    fn exact_nodes(source: &str, expected: &str, nodes: usize) {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let before = report.clone();
        assert_eq!(
            report
                .syntax()
                .to_specified_css_with_limits(Limits::new(nodes, nodes, expected.len()))
                .unwrap(),
            expected,
        );
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes, nodes, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            let error = report
                .syntax()
                .to_specified_css_with_limits(limits)
                .unwrap_err();
            assert_eq!(error.kind(), RuleKind::Resource(kind));
            assert_eq!(error.rule_index(), Some(0));
            assert_eq!(report, before);
        }
        assert_eq!(report, before);
    }

    #[test]
    fn empty_media_lists_keep_one_prelude_space_and_their_aggregate_cost() {
        // Sheet, media rule, and empty query-list aggregate each cost one.
        exact_nodes("@media{}", "@media { }", 3);
        // A scope adds one logical rule, without an enum/list carrier charge.
        exact_nodes("@scope{@media{}}", "@scope { @media { } }", 4);
    }

    #[test]
    fn layer_names_charge_lists_names_and_identifier_components_once() {
        // Sheet/rule 2 + name aggregate 1 + identifier component 1.
        exact_nodes("@layer base{}", "@layer base { }", 4);
        // Sheet/rule/list 3 + two names 2 + three components 3.
        exact_nodes("@layer a.b,c;", "@layer a.b, c;", 8);
        // Anonymous blocks have no name or name-list aggregate.
        exact_nodes("@layer{}", "@layer { }", 2);
    }

    #[test]
    fn recovered_charset_only_sheet_charges_its_empty_retained_aggregate() {
        let report = parse_sheet("@charset \"windows-1252\";");
        assert!(!report.is_clean());
        let [diagnostic] = report.diagnostics() else {
            panic!("one unknown charset")
        };
        assert_eq!(
            diagnostic.error().code(),
            crate::CssErrorCode::UnknownAtRule
        );
        assert_eq!(diagnostic.action(), crate::CssRecoveryAction::DropAtRule);
        let before = report.clone();
        assert_eq!(
            report
                .syntax()
                .to_specified_css_with_limits(Limits::new(1, 1, 0))
                .unwrap(),
            "",
        );
        for (limits, kind) in [
            (Limits::new(0, 1, 0), Kind::InputNodeLimit),
            (Limits::new(1, 0, 0), Kind::ProjectionNodeLimit),
        ] {
            let error = report
                .syntax()
                .to_specified_css_with_limits(limits)
                .unwrap_err();
            assert_eq!(error.kind(), RuleKind::Resource(kind));
            assert_eq!(error.rule_index(), None);
        }
        assert_eq!(report, before);
        assert!(report.syntax().rules().is_empty());
    }

    #[test]
    fn final_group_delimiters_report_the_enclosing_rule_without_returning_partial_text() {
        let report = parse_sheet("@namespace n 'u';@scope{@layer{@font-face{}}}");
        assert!(report.is_clean());
        let before = report.clone();
        let expected = "@namespace n url(\"u\");\n@scope { @layer { @font-face { } } }";
        let error = report
            .syntax()
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len() - 1))
            .unwrap_err();
        assert_eq!(error.kind(), RuleKind::Resource(Kind::ByteLimit));
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(report, before);
        assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
        assert!(matches!(report.syntax().rules()[1], CssRule::Scope(_)));
    }
}
