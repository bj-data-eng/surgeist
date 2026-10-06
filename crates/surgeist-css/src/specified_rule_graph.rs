//! Borrowed authored rule graph traversal under one monotonic context.
//!
//! Each logical rule charges one input and projection node: published leaf
//! writers retain their existing charge, and new graph arms charge before
//! scheduling children. Rust enum carriers and child slices are transparent.
//! Slice continuations retain only the remaining children, so graph width does
//! not allocate a pending event for every sibling before checking work limits.

use crate::specified_rule_serialization::{SpecifiedRuleSerializationSource, SpecifiedRuleWriter};
use crate::{
    CssDeclarationList, CssLayerName, CssLayerNameList, CssMediaQueryList, CssRule, CssScopeRule,
    CssScopedRule, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
};

type Result<T> = std::result::Result<T, SpecifiedRuleSerializationSource>;
type ValueResult<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

enum Event<'a> {
    Ordinary(&'a CssRule),
    Scoped(&'a CssScopedRule),
    OrdinaryChildren(&'a [CssRule], bool),
    ScopedChildren(&'a [CssScopedRule], bool),
    Declarations(&'a CssDeclarationList),
    Text(&'static str),
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
    pub(crate) fn append_rule_graph(&mut self, rule: &CssRule) -> Result<()> {
        let mut work = Vec::new();
        push(&mut work, Event::Ordinary(rule))?;
        while let Some(event) = work.pop() {
            match event {
                Event::Text(text) => self.append(text)?,
                Event::Declarations(declarations) => {
                    self.append_authored_declaration_list(declarations)?;
                }
                Event::OrdinaryChildren(rules, separator) => {
                    if let Some((first, rest)) = rules.split_first() {
                        if separator {
                            self.append(" ")?;
                        }
                        push(&mut work, Event::OrdinaryChildren(rest, true))?;
                        push(&mut work, Event::Ordinary(first))?;
                    }
                }
                Event::ScopedChildren(rules, separator) => {
                    if let Some((first, rest)) = rules.split_first() {
                        if separator {
                            self.append(" ")?;
                        }
                        push(&mut work, Event::ScopedChildren(rest, true))?;
                        push(&mut work, Event::Scoped(first))?;
                    }
                }
                Event::Ordinary(rule) => match rule {
                    CssRule::Namespace(rule) => self.namespace(rule)?,
                    CssRule::CounterStyle(rule) => rule.append_to_rule_writer(self)?,
                    CssRule::FontFace(rule) => self.font_face(rule)?,
                    CssRule::FontFeatureValues(rule) => self.font_features(rule)?,
                    CssRule::FontPaletteValues(rule) => self.palette(rule)?,
                    CssRule::ColorProfile(rule) => self.color_profile(rule)?,
                    CssRule::SupportsCondition(rule) => self.named_supports_rule(rule)?,
                    CssRule::NestedDeclarations(rule) => {
                        self.node()?;
                        push(&mut work, Event::Declarations(rule.declarations()))?;
                    }
                    CssRule::Page(rule) => {
                        self.node()?;
                        self.page(rule)?;
                    }
                    CssRule::Keyframes(rule) => {
                        self.node()?;
                        self.keyframes(rule)?;
                    }
                    CssRule::CustomMedia(rule) => {
                        self.node()?;
                        rule.append_specified(&mut self.context, &mut self.css)?;
                    }
                    CssRule::Import(rule) => {
                        self.node()?;
                        rule.append_specified(&mut self.context, &mut self.css)?;
                    }
                    CssRule::LayerStatement(rule) => {
                        self.node()?;
                        self.append("@layer ")?;
                        self.layer_names(rule.names())?;
                        self.append(";")?;
                    }
                    CssRule::LayerBlock(rule) => {
                        self.node()?;
                        self.layer_prelude(rule.name())?;
                        self.open_block(
                            &mut work,
                            (!rule.rules().is_empty())
                                .then_some(Event::OrdinaryChildren(rule.rules(), false)),
                        )?;
                    }
                    CssRule::Style(rule) => {
                        self.node()?;
                        self.style_selectors(rule.selectors())?;
                        self.style_block(&mut work, rule.declarations(), rule.rules())?;
                    }
                    CssRule::Media(rule) => {
                        self.node()?;
                        self.media_prelude(rule.query())?;
                        self.open_block(
                            &mut work,
                            (!rule.rules().is_empty())
                                .then_some(Event::OrdinaryChildren(rule.rules(), false)),
                        )?;
                    }
                    CssRule::Supports(rule) => {
                        self.node()?;
                        self.append("@supports ")?;
                        rule.condition()
                            .append_specified(&mut self.context, &mut self.css)?;
                        self.open_block(
                            &mut work,
                            (!rule.rules().is_empty())
                                .then_some(Event::OrdinaryChildren(rule.rules(), false)),
                        )?;
                    }
                    CssRule::Container(rule) => {
                        self.node()?;
                        self.append("@container ")?;
                        rule.prelude()
                            .append_specified(&mut self.context, &mut self.css)?;
                        self.open_block(
                            &mut work,
                            (!rule.rules().is_empty())
                                .then_some(Event::OrdinaryChildren(rule.rules(), false)),
                        )?;
                    }
                    CssRule::Scope(rule) => self.scope_block(&mut work, rule)?,
                },
                Event::Scoped(rule) => match rule {
                    CssScopedRule::CounterStyle(rule) => rule.append_to_rule_writer(self)?,
                    CssScopedRule::FontFace(rule) => self.font_face(rule)?,
                    CssScopedRule::FontFeatureValues(rule) => self.font_features(rule)?,
                    CssScopedRule::FontPaletteValues(rule) => self.palette(rule)?,
                    CssScopedRule::ColorProfile(rule) => self.color_profile(rule)?,
                    CssScopedRule::SupportsCondition(rule) => self.named_supports_rule(rule)?,
                    CssScopedRule::NestedDeclarations(rule) => {
                        self.node()?;
                        push(&mut work, Event::Declarations(rule.declarations()))?;
                    }
                    CssScopedRule::Page(rule) => {
                        self.node()?;
                        self.page(rule)?;
                    }
                    CssScopedRule::Keyframes(rule) => {
                        self.node()?;
                        self.keyframes(rule)?;
                    }
                    CssScopedRule::CustomMedia(rule) => {
                        self.node()?;
                        rule.append_specified(&mut self.context, &mut self.css)?;
                    }
                    CssScopedRule::LayerStatement(rule) => {
                        self.node()?;
                        self.append("@layer ")?;
                        self.layer_names(rule.names())?;
                        self.append(";")?;
                    }
                    CssScopedRule::LayerBlock(rule) => {
                        self.node()?;
                        self.layer_prelude(rule.name())?;
                        self.open_block(
                            &mut work,
                            (!rule.rules().rules().is_empty())
                                .then_some(Event::ScopedChildren(rule.rules().rules(), false)),
                        )?;
                    }
                    CssScopedRule::Style(rule) => {
                        self.node()?;
                        self.scoped_style_selectors(rule.selectors())?;
                        // Scoped style children have ordinary nesting semantics.
                        self.style_block(&mut work, rule.declarations(), rule.rules())?;
                    }
                    CssScopedRule::Media(rule) => {
                        self.node()?;
                        self.media_prelude(rule.query())?;
                        self.open_block(
                            &mut work,
                            (!rule.rules().rules().is_empty())
                                .then_some(Event::ScopedChildren(rule.rules().rules(), false)),
                        )?;
                    }
                    CssScopedRule::Supports(rule) => {
                        self.node()?;
                        self.append("@supports ")?;
                        rule.condition()
                            .append_specified(&mut self.context, &mut self.css)?;
                        self.open_block(
                            &mut work,
                            (!rule.rules().rules().is_empty())
                                .then_some(Event::ScopedChildren(rule.rules().rules(), false)),
                        )?;
                    }
                    CssScopedRule::Container(rule) => {
                        self.node()?;
                        self.append("@container ")?;
                        rule.prelude()
                            .append_specified(&mut self.context, &mut self.css)?;
                        self.open_block(
                            &mut work,
                            (!rule.rules().rules().is_empty())
                                .then_some(Event::ScopedChildren(rule.rules().rules(), false)),
                        )?;
                    }
                    CssScopedRule::Scope(rule) => self.scope_block(&mut work, rule)?,
                },
            }
        }
        Ok(())
    }

    fn open_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        child: Option<Event<'a>>,
    ) -> ValueResult<()> {
        self.append(" {")?;
        push(work, Event::Text(" }"))?;
        if let Some(child) = child {
            self.append(" ")?;
            push(work, child)?;
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
    ) -> ValueResult<()> {
        self.append(" {")?;
        push(work, Event::Text(" }"))?;
        if !declarations.is_empty() || !children.is_empty() {
            self.append(" ")?;
        }
        if !children.is_empty() {
            push(
                work,
                Event::OrdinaryChildren(children, !declarations.is_empty()),
            )?;
        }
        push(work, Event::Declarations(declarations))
    }

    fn scope_block<'a>(
        &mut self,
        work: &mut Vec<Event<'a>>,
        rule: &'a CssScopeRule,
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
            (!rule.rules().rules().is_empty())
                .then_some(Event::ScopedChildren(rule.rules().rules(), false)),
        )
    }

    fn media_prelude(&mut self, list: &CssMediaQueryList) -> Result<()> {
        self.append("@media")?;
        if !list.queries().is_empty() {
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
