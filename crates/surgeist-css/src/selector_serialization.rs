//! Authored selector emission. Symbolic nesting and scope anchors stay symbolic.
//! This provider owns selector grammar; enclosing rule formatting has a separate owner.
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::syntax::{CssPseudoSuffixContext, CssSelectorGrammarContext};
use crate::*;
type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;
/// Output proof scope, independent of the existing intrinsic selector grammar.
#[derive(Clone, Copy, Eq, PartialEq)]
enum OutputRole {
    Authored,
    LiteralStyle,
    SelectorArgument,
}

enum Event<'a> {
    Selector(&'a CssSelector, CssSelectorGrammarContext),
    Complex(&'a CssComplexSelector, CssSelectorGrammarContext),
    Compound(&'a CssCompoundSelector, CssSelectorGrammarContext),
    CompoundTail(
        &'a CssCompoundSelector,
        u8,
        usize,
        CssSelectorGrammarContext,
        CssPseudoSuffixContext,
    ),
    ComplexParts(
        &'a [CssComplexSelectorPart],
        usize,
        CssSelectorGrammarContext,
    ),
    Pseudo(&'a CssPseudoClass, CssSelectorGrammarContext),
    Element(&'a CssPseudoElement, CssSelectorGrammarContext),
    Attribute(&'a CssAttributeSelector),
    List(&'a [CssSelector], usize, CssSelectorGrammarContext),
    PseudoList(
        &'a [CssPseudoSelectorListItem],
        usize,
        CssSelectorGrammarContext,
        bool,
    ),
    RelativeList(&'a [CssRelativeSelector], usize, CssSelectorGrammarContext),
    StyleList(&'a [CssStyleSelector], usize, bool),
    ScopedStyleList(&'a [CssScopedStyleSelector], usize, bool),
    ScopeList(&'a [CssScopeSelector], usize, bool),
    Relative(&'a CssRelativeSelector, CssSelectorGrammarContext),
    LiteralMember(&'a CssSelector, Option<CssSelectorCombinator>, bool),
    Text(&'a str),
    OutputRole(OutputRole),
    Nth(CssNthPattern),
}
fn push<'a>(work: &mut Vec<Event<'a>>, event: Event<'a>) -> Result<()> {
    work.try_reserve(1).map_err(|_| {
        CssSpecifiedValueSerializationError::new(
            CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        )
    })?;
    work.push(event);
    Ok(())
}
/// Selector-function arguments conservatively retain every explicit universal,
/// including non-subject compounds. Restore the caller's output proof on return.
fn push_argument<'a>(work: &mut Vec<Event<'a>>, event: Event<'a>, role: OutputRole) -> Result<()> {
    push(work, Event::OutputRole(role))?;
    push(work, event)?;
    push(
        work,
        Event::OutputRole(if role == OutputRole::Authored {
            OutputRole::Authored
        } else {
            OutputRole::SelectorArgument
        }),
    )
}

impl CssSelector {
    /// Emits an authored selector, without resolving its symbolic ancestry.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits atomically with cumulative semantic-node and UTF-8 byte limits.
    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.selector(self)?;
        Ok(writer.css)
    }
}
impl CssSelectorList {
    /// Emits the complete ordinary list without resolving symbolic ancestry.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Emits atomically with one cumulative aggregate/member/UTF-8 budget.
    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.ordinary_selectors(self)?;
        Ok(writer.css)
    }
}

impl CssRelativeSelectorList {
    /// Emits general relatives with their explicit or implicit leading relation.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Emits atomically, including every relative carrier in one shared budget.
    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.relative_selectors(self)?;
        Ok(writer.css)
    }
}

/// The actual role of a checked scope boundary. Start binds to its enclosing
/// context; End binds to the scope introduced by its rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssScopeSelectorCssomContext {
    Start(CssScopeNestingContext),
    End,
}

impl CssSelectorList {
    /// Formats a literal ordinary list using actual namespace URI bindings.
    pub fn serialize_cssom(&self, namespaces: &CssNamespaceContext) -> Result<String> {
        self.serialize_cssom_with_limits(
            namespaces,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn serialize_cssom_with_limits(
        &self,
        namespaces: &CssNamespaceContext,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.node()?;
        for selector in self.selectors() {
            writer.check_anchor_domain(selector, false)?;
        }
        writer.selector_events_with_role(
            Event::List(self.selectors(), 0, CssSelectorGrammarContext::ORDINARY),
            OutputRole::LiteralStyle,
            Some(namespaces),
        )?;
        Ok(writer.css)
    }
}
impl CssRelativeSelectorList {
    /// Formats general relative selectors. These are not nested style rules;
    /// their leading relationships remain relative, including inside `:has()`.
    pub fn serialize_cssom(&self, namespaces: &CssNamespaceContext) -> Result<String> {
        self.serialize_cssom_with_limits(
            namespaces,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn serialize_cssom_with_limits(
        &self,
        namespaces: &CssNamespaceContext,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.node()?;
        for relative in self.selectors() {
            writer.check_anchor_domain(relative.selector(), false)?;
        }
        writer.selector_events_with_role(
            Event::RelativeList(self.selectors(), 0, CssSelectorGrammarContext::ORDINARY),
            OutputRole::LiteralStyle,
            Some(namespaces),
        )?;
        Ok(writer.css)
    }
}
impl CssStyleSelectorList {
    /// Formats checked ordinary or nested style selectors in their actual role.
    /// A scoped role is rejected rather than changing their anchor domain.
    pub fn serialize_cssom(
        &self,
        namespaces: &CssNamespaceContext,
        context: CssStyleSelectorContext,
    ) -> Result<String> {
        self.serialize_cssom_with_limits(
            namespaces,
            context,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn serialize_cssom_with_limits(
        &self,
        namespaces: &CssNamespaceContext,
        context: CssStyleSelectorContext,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let nested = match context {
            CssStyleSelectorContext::Ordinary => false,
            CssStyleSelectorContext::Nested => true,
            CssStyleSelectorContext::Scoped(_) => return Err(invalid_context()),
        };
        let mut writer = SpecifiedRuleWriter::new(limits);
        for member in self.selectors() {
            let selector = match member {
                CssStyleSelector::Selector(value) => value,
                CssStyleSelector::Relative(value) => {
                    if !nested {
                        return Err(invalid_context());
                    }
                    value.selector()
                }
            };
            writer.check_anchor_domain(selector, false)?;
        }
        writer.cssom_style_selectors(self, nested, Some(namespaces))?;
        Ok(writer.css)
    }
}
impl CssScopedStyleSelectorList {
    /// Formats scoped selectors while preserving their genuine scope anchors.
    pub fn serialize_cssom(
        &self,
        namespaces: &CssNamespaceContext,
        context: CssStyleSelectorContext,
    ) -> Result<String> {
        self.serialize_cssom_with_limits(
            namespaces,
            context,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn serialize_cssom_with_limits(
        &self,
        namespaces: &CssNamespaceContext,
        context: CssStyleSelectorContext,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let CssStyleSelectorContext::Scoped(ancestor) = context else {
            return Err(invalid_context());
        };
        let nested = ancestor == CssStyleAncestor::Present;
        let mut writer = SpecifiedRuleWriter::new(limits);
        for member in self.selectors() {
            let selector = match member {
                CssScopedStyleSelector::Selector(value) => value,
                CssScopedStyleSelector::Relative(value) => value.selector(),
            };
            writer.check_anchor_domain(selector, true)?;
        }
        writer.cssom_scoped_style_selectors(self, nested, Some(namespaces))?;
        Ok(writer.css)
    }
}
impl CssParsedStyleSelectors {
    /// Delegates using the actual retained admission role and original graph.
    /// Namespace URI bindings remain an explicit, separate context input.
    pub fn serialize_cssom(&self, namespaces: &CssNamespaceContext) -> Result<String> {
        self.serialize_cssom_with_limits(
            namespaces,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn serialize_cssom_with_limits(
        &self,
        namespaces: &CssNamespaceContext,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        match self.selectors() {
            CssAdmittedStyleSelectors::Ordinary(list) => {
                list.serialize_cssom_with_limits(namespaces, self.context(), limits)
            }
            CssAdmittedStyleSelectors::Scoped(list) => {
                list.serialize_cssom_with_limits(namespaces, self.context(), limits)
            }
        }
    }
}
impl CssScopeSelectorList {
    /// Formats one scope bound without parentheses or an enclosing rule.
    pub fn serialize_cssom(
        &self,
        namespaces: &CssNamespaceContext,
        context: CssScopeSelectorCssomContext,
    ) -> Result<String> {
        self.serialize_cssom_with_limits(
            namespaces,
            context,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn serialize_cssom_with_limits(
        &self,
        namespaces: &CssNamespaceContext,
        context: CssScopeSelectorCssomContext,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let (nested, scoped) = match context {
            CssScopeSelectorCssomContext::Start(context) => (
                context != CssScopeNestingContext::None,
                context == CssScopeNestingContext::Scope,
            ),
            CssScopeSelectorCssomContext::End => (false, true),
        };
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.node()?;
        for member in self.selectors() {
            if matches!(member, CssScopeSelector::Relative(_))
                && matches!(
                    context,
                    CssScopeSelectorCssomContext::Start(CssScopeNestingContext::None)
                )
            {
                return Err(invalid_context());
            }
            writer.check_anchor_domain(member.selector(), scoped)?;
        }
        writer.selector_events_with_role(
            Event::ScopeList(self.selectors(), 0, nested),
            OutputRole::LiteralStyle,
            Some(namespaces),
        )?;
        Ok(writer.css)
    }
}
impl CssScopeRule {
    /// Returns the independently formatted start bound, or None for an omitted
    /// start. No empty string substitutes for an absent bound.
    pub fn serialize_cssom_start(
        &self,
        namespaces: &CssNamespaceContext,
        nesting: CssScopeNestingContext,
    ) -> Result<Option<String>> {
        self.serialize_cssom_start_with_limits(
            namespaces,
            nesting,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn serialize_cssom_start_with_limits(
        &self,
        namespaces: &CssNamespaceContext,
        nesting: CssScopeNestingContext,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Option<String>> {
        self.root()
            .map(|list| {
                list.serialize_cssom_with_limits(
                    namespaces,
                    CssScopeSelectorCssomContext::Start(nesting),
                    limits,
                )
            })
            .transpose()
    }
    /// Returns the independently formatted end bound, or None for an omitted end.
    pub fn serialize_cssom_end(&self, namespaces: &CssNamespaceContext) -> Result<Option<String>> {
        self.serialize_cssom_end_with_limits(
            namespaces,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn serialize_cssom_end_with_limits(
        &self,
        namespaces: &CssNamespaceContext,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Option<String>> {
        self.limit()
            .map(|list| {
                list.serialize_cssom_with_limits(
                    namespaces,
                    CssScopeSelectorCssomContext::End,
                    limits,
                )
            })
            .transpose()
    }
}
fn invalid_context() -> CssSpecifiedValueSerializationError {
    CssSpecifiedValueSerializationError::new(
        CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
    )
}

impl SpecifiedRuleWriter {
    pub(crate) fn node(&mut self) -> Result<()> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)
    }
    pub(crate) fn keyword(&mut self, text: &str) -> Result<()> {
        self.node()?;
        self.append(text)
    }
    pub(crate) fn combinator(&mut self, value: CssSelectorCombinator, leading: bool) -> Result<()> {
        self.append(match (value, leading) {
            (CssSelectorCombinator::Descendant, true) => "",
            (CssSelectorCombinator::Descendant, false) => " ",
            (CssSelectorCombinator::Child, true) => "> ",
            (CssSelectorCombinator::Child, false) => " > ",
            (CssSelectorCombinator::NextSibling, true) => "+ ",
            (CssSelectorCombinator::NextSibling, false) => " + ",
            (CssSelectorCombinator::SubsequentSibling, true) => "~ ",
            (CssSelectorCombinator::SubsequentSibling, false) => " ~ ",
            (CssSelectorCombinator::Column, true) => "|| ",
            (CssSelectorCombinator::Column, false) => " || ",
        })
    }

    fn selector_identifier(&mut self, value: &str) -> Result<()> {
        if value.is_empty() || value.contains('\0') {
            return Err(CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
            ));
        }
        self.append_identifier(value)
    }

    fn name_prefix(&mut self, prefix: &CssQualifiedNamePrefix) -> Result<()> {
        match prefix {
            CssQualifiedNamePrefix::Unqualified => Ok(()),
            CssQualifiedNamePrefix::ExplicitNone => self.append("|"),
            CssQualifiedNamePrefix::Any => self.append("*|"),
            CssQualifiedNamePrefix::Named(name) => {
                self.selector_identifier(name.as_str())?;
                self.append("|")
            }
        }
    }
    fn has_anchor(
        &mut self,
        selector: &CssSelector,
        kind: crate::selector_anchors::AnchorKind,
    ) -> Result<bool> {
        crate::selector_anchors::selector_has_anchor(selector, kind, &mut |depth| {
            if depth > crate::STRUCTURAL_NESTING_LIMIT {
                return Err(invalid_context());
            }
            self.context.charge_input(1)
        })
    }
    fn check_anchor_domain(&mut self, selector: &CssSelector, scoped: bool) -> Result<()> {
        let forbidden = if scoped {
            crate::selector_anchors::AnchorKind::TypedNesting
        } else {
            crate::selector_anchors::AnchorKind::TypedScope
        };
        if self.has_anchor(selector, forbidden)? {
            Err(invalid_context())
        } else {
            Ok(())
        }
    }
    fn check_name_context(
        &self,
        prefix: &CssQualifiedNamePrefix,
        constraint: &CssNamespaceConstraint,
        element: bool,
        namespaces: &CssNamespaceContext,
    ) -> Result<()> {
        let available = match constraint {
            CssNamespaceConstraint::Default => namespaces.default_namespace().is_some(),
            CssNamespaceConstraint::Named(prefix) => namespaces.named_namespace(prefix).is_some(),
            _ => true,
        };
        if !available {
            return Err(CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::NamespaceBindingUnavailable,
            ));
        }
        crate::rule_construction::check_namespace(prefix, constraint, namespaces, element)
            .map_err(|_| invalid_context())
    }
    fn output_name_prefix(
        &mut self,
        prefix: &CssQualifiedNamePrefix,
        constraint: &CssNamespaceConstraint,
        element: bool,
        role: OutputRole,
        namespaces: Option<&CssNamespaceContext>,
    ) -> Result<()> {
        if role != OutputRole::Authored
            && let Some(namespaces) = namespaces
        {
            self.check_name_context(prefix, constraint, element, namespaces)?;
        }
        if role == OutputRole::Authored || matches!(prefix, CssQualifiedNamePrefix::Any) {
            return self.name_prefix(prefix);
        }
        if !element && matches!(constraint, CssNamespaceConstraint::ExplicitNone) {
            return Ok(());
        }
        let Some(namespaces) = namespaces else {
            return self.name_prefix(prefix);
        };
        let missing = || {
            CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::NamespaceBindingUnavailable,
            )
        };
        let uri = match constraint {
            CssNamespaceConstraint::Any => return self.name_prefix(prefix),
            CssNamespaceConstraint::ExplicitNone => "",
            CssNamespaceConstraint::Default => {
                namespaces.default_namespace().ok_or_else(missing)?.as_str()
            }
            CssNamespaceConstraint::Named(name) => namespaces
                .named_namespace(name)
                .ok_or_else(missing)?
                .as_str(),
        };
        if uri.is_empty() {
            return if element { self.append("|") } else { Ok(()) };
        }
        if element
            && namespaces
                .default_namespace()
                .is_some_and(|default| default.as_str() == uri)
        {
            return Ok(());
        }
        self.name_prefix(prefix)
    }
    pub(crate) fn selector(&mut self, selector: &CssSelector) -> Result<()> {
        self.selector_events(Event::Selector(
            selector,
            CssSelectorGrammarContext::ORDINARY,
        ))
    }

    pub(crate) fn complex_selector(&mut self, selector: &CssComplexSelector) -> Result<()> {
        self.selector_events(Event::Complex(
            selector,
            CssSelectorGrammarContext::ORDINARY,
        ))
    }

    pub(crate) fn compound_selector(&mut self, selector: &CssCompoundSelector) -> Result<()> {
        self.selector_events(Event::Compound(
            selector,
            CssSelectorGrammarContext::ORDINARY,
        ))
    }

    pub(crate) fn complex_selector_part(&mut self, part: &CssComplexSelectorPart) -> Result<()> {
        self.combinator(part.combinator(), false)?;
        self.compound_selector(part.selector())
    }

    pub(crate) fn attribute_selector(&mut self, selector: &CssAttributeSelector) -> Result<()> {
        self.selector_events(Event::Attribute(selector))
    }

    pub(crate) fn ordinary_selectors(&mut self, list: &CssSelectorList) -> Result<()> {
        self.node()?;
        self.selector_events(Event::List(
            list.selectors(),
            0,
            CssSelectorGrammarContext::ORDINARY,
        ))
    }

    pub(crate) fn pseudo_selector_members(&mut self, selectors: &[CssSelector]) -> Result<()> {
        self.node()?;
        self.selector_events(Event::List(
            selectors,
            0,
            CssSelectorGrammarContext::ORDINARY.logical_arguments(),
        ))
    }

    pub(crate) fn relative_selectors(&mut self, list: &CssRelativeSelectorList) -> Result<()> {
        self.node()?;
        self.selector_events(Event::RelativeList(
            list.selectors(),
            0,
            CssSelectorGrammarContext::ORDINARY,
        ))
    }

    /// Each authored list charges one aggregate. Enum member carriers are
    /// transparent; a relative selector charges its leading-combinator node
    /// before consuming the same selector provider as an absolute member.
    pub(crate) fn style_selectors(&mut self, list: &CssStyleSelectorList) -> Result<()> {
        self.node()?;
        self.selector_events(Event::StyleList(list.selectors(), 0, false))
    }

    pub(crate) fn scoped_style_selectors(
        &mut self,
        list: &CssScopedStyleSelectorList,
    ) -> Result<()> {
        self.node()?;
        self.selector_events(Event::ScopedStyleList(list.selectors(), 0, false))
    }

    pub(crate) fn cssom_style_selectors(
        &mut self,
        list: &CssStyleSelectorList,
        nested: bool,
        namespaces: Option<&CssNamespaceContext>,
    ) -> Result<()> {
        self.node()?;
        self.selector_events_with_role(
            Event::StyleList(list.selectors(), 0, nested),
            OutputRole::LiteralStyle,
            namespaces,
        )
    }

    pub(crate) fn cssom_scoped_style_selectors(
        &mut self,
        list: &CssScopedStyleSelectorList,
        nested: bool,
        namespaces: Option<&CssNamespaceContext>,
    ) -> Result<()> {
        self.node()?;
        self.selector_events_with_role(
            Event::ScopedStyleList(list.selectors(), 0, nested),
            OutputRole::LiteralStyle,
            namespaces,
        )
    }

    pub(crate) fn scope_selectors(&mut self, list: &CssScopeSelectorList) -> Result<()> {
        self.node()?;
        self.selector_events(Event::ScopeList(list.selectors(), 0, false))
    }

    fn selector_events(&mut self, initial: Event<'_>) -> Result<()> {
        self.selector_events_with_role(initial, OutputRole::Authored, None)
    }

    fn selector_events_with_role(
        &mut self,
        initial: Event<'_>,
        mut role: OutputRole,
        namespaces: Option<&CssNamespaceContext>,
    ) -> Result<()> {
        let mut work = Vec::new();
        push(&mut work, initial)?;
        while let Some(event) = work.pop() {
            match event {
                Event::Text(value) => self.append(value)?,
                Event::OutputRole(value) => role = value,
                Event::LiteralMember(value, leading, nested) => {
                    if role == OutputRole::Authored {
                        if let Some(combinator) = leading {
                            self.node()?;
                            self.combinator(combinator, true)?;
                        }
                    } else if leading.is_some()
                        || (nested
                            && !self
                                .has_anchor(value, crate::selector_anchors::AnchorKind::Either)?)
                    {
                        self.context.charge_projection(1)?;
                        self.append("&")?;
                        self.combinator(
                            leading.unwrap_or(CssSelectorCombinator::Descendant),
                            false,
                        )?;
                        if leading.is_some() {
                            self.node()?;
                        }
                    }
                    push(
                        &mut work,
                        Event::Selector(value, CssSelectorGrammarContext::ORDINARY),
                    )?;
                }
                Event::Selector(value, grammar) => {
                    if !grammar.admits_selector(value) {
                        return Err(CssSpecifiedValueSerializationError::new(
                            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
                        ));
                    }
                    match value {
                        CssSelector::Tag(value) => {
                            self.node()?;
                            if role != OutputRole::Authored
                                && let Some(namespaces) = namespaces
                            {
                                self.check_name_context(
                                    &CssQualifiedNamePrefix::Unqualified,
                                    &CssNamespaceConstraint::Any,
                                    true,
                                    namespaces,
                                )?;
                            }
                            self.selector_identifier(value)?;
                        }
                        CssSelector::Key(value) => {
                            self.node()?;
                            self.append("#")?;
                            self.selector_identifier(value)?;
                        }
                        CssSelector::Class(value) => {
                            self.node()?;
                            self.append(".")?;
                            self.selector_identifier(value)?;
                        }
                        CssSelector::PseudoClass(value) => {
                            push(&mut work, Event::Pseudo(value, grammar))?
                        }
                        CssSelector::Compound(value) => {
                            push(&mut work, Event::Compound(value, grammar))?
                        }
                        CssSelector::Complex(value) => {
                            push(&mut work, Event::Complex(value, grammar))?;
                        }
                    }
                }
                Event::Complex(value, grammar) => {
                    self.node()?;
                    push(&mut work, Event::ComplexParts(value.rest(), 0, grammar))?;
                    push(&mut work, Event::Compound(value.first(), grammar))?;
                }
                Event::Compound(value, grammar) => {
                    if !grammar.admits_compound(value) {
                        return Err(CssSpecifiedValueSerializationError::new(
                            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
                        ));
                    }
                    self.node()?;
                    if let Some(name) = value.type_selector() {
                        self.node()?;
                        // The real type-name visit above remains charged even when
                        // its star contributes no bytes under the selected literal policy.
                        if role != OutputRole::Authored
                            && let Some(namespaces) = namespaces
                        {
                            self.check_name_context(
                                name.prefix(),
                                name.namespace(),
                                true,
                                namespaces,
                            )?;
                        }
                        let omit = role == OutputRole::LiteralStyle
                            && name.local_name().is_none()
                            && matches!(name.prefix(), CssQualifiedNamePrefix::Unqualified)
                            && matches!(
                                name.namespace(),
                                CssNamespaceConstraint::Any | CssNamespaceConstraint::Default
                            )
                            && value.nesting_selectors() == 0
                            && value.scope_anchors() == 0
                            && (!value.ids().is_empty()
                                || !value.classes().is_empty()
                                || !value.attributes().is_empty());
                        if !omit {
                            self.output_name_prefix(
                                name.prefix(),
                                name.namespace(),
                                true,
                                role,
                                namespaces,
                            )?;
                            if let Some(local) = name.local_name() {
                                self.selector_identifier(local)?;
                            } else {
                                self.append("*")?;
                            }
                        }
                    }
                    for _ in 0..value.nesting_selectors() {
                        self.keyword("&")?;
                    }
                    for _ in 0..value.scope_anchors() {
                        self.keyword("&")?;
                    }
                    for id in value.ids() {
                        self.node()?;
                        self.append("#")?;
                        self.selector_identifier(id)?;
                    }
                    for class in value.classes() {
                        self.node()?;
                        self.append(".")?;
                        self.selector_identifier(class)?;
                    }
                    push(
                        &mut work,
                        Event::CompoundTail(value, 0, 0, grammar, CssPseudoSuffixContext::Generic),
                    )?;
                }
                Event::ComplexParts(parts, index, grammar) => {
                    if let Some(part) = parts.get(index) {
                        self.combinator(part.combinator(), false)?;
                        push(&mut work, Event::ComplexParts(parts, index + 1, grammar))?;
                        push(&mut work, Event::Compound(part.selector(), grammar))?;
                    }
                }
                Event::CompoundTail(value, phase, index, grammar, mut suffix) => {
                    let event = match phase {
                        0 => value.attributes().get(index).map(Event::Attribute),
                        1 => value
                            .pseudo_classes()
                            .get(index)
                            .map(|p| Event::Pseudo(p, grammar)),
                        2 => value
                            .pseudo_elements()
                            .and_then(|s| s.segments().get(index))
                            .map(|s| match s {
                                CssPseudoElementSegment::PseudoElement(e) => {
                                    suffix = e.suffix_context();
                                    Event::Element(e, grammar)
                                }
                                CssPseudoElementSegment::PseudoClass(p) => {
                                    Event::Pseudo(p, grammar.suffix(suffix))
                                }
                            }),
                        _ => None,
                    };
                    if let Some(event) = event {
                        push(
                            &mut work,
                            Event::CompoundTail(value, phase, index + 1, grammar, suffix),
                        )?;
                        push(&mut work, event)?;
                    } else if phase < 2 {
                        push(
                            &mut work,
                            Event::CompoundTail(value, phase + 1, 0, grammar, suffix),
                        )?;
                    }
                }

                Event::List(values, index, grammar) => {
                    if let Some(value) = values.get(index) {
                        if index != 0 {
                            self.append(", ")?;
                        }
                        push(&mut work, Event::List(values, index + 1, grammar))?;
                        push(&mut work, Event::Selector(value, grammar))?;
                    }
                }
                Event::PseudoList(values, index, grammar, forgiving) => {
                    if let Some(value) = values.get(index) {
                        if index != 0 {
                            self.append(
                                if matches!(value, CssPseudoSelectorListItem::InvalidNesting(_)) {
                                    ","
                                } else {
                                    ", "
                                },
                            )?;
                        }
                        push(
                            &mut work,
                            Event::PseudoList(values, index + 1, grammar, forgiving),
                        )?;
                        match value {
                            CssPseudoSelectorListItem::Selector(value) => {
                                push(&mut work, Event::Selector(value, grammar))?
                            }
                            CssPseudoSelectorListItem::InvalidNesting(value) => {
                                self.node()?;
                                if !forgiving || !value.admitted_in(grammar) {
                                    return Err(CssSpecifiedValueSerializationError::new(CssSpecifiedValueSerializationErrorKind::UnrepresentableValue));
                                }
                                value
                                    .components()
                                    .append_original_region(value.origin(), self)?;
                            }
                        }
                    }
                }
                Event::RelativeList(values, index, grammar) => {
                    if let Some(value) = values.get(index) {
                        if index != 0 {
                            self.append(", ")?;
                        }
                        push(&mut work, Event::RelativeList(values, index + 1, grammar))?;
                        push(&mut work, Event::Relative(value, grammar))?;
                    }
                }
                Event::StyleList(values, index, nested) => {
                    if let Some(value) = values.get(index) {
                        if index != 0 {
                            self.append(", ")?;
                        }
                        push(&mut work, Event::StyleList(values, index + 1, nested))?;
                        push(
                            &mut work,
                            match value {
                                CssStyleSelector::Selector(value) => {
                                    Event::LiteralMember(value, None, nested)
                                }
                                CssStyleSelector::Relative(value) => Event::LiteralMember(
                                    value.selector(),
                                    Some(value.combinator()),
                                    nested,
                                ),
                            },
                        )?;
                    }
                }
                Event::ScopedStyleList(values, index, nested) => {
                    if let Some(value) = values.get(index) {
                        if index != 0 {
                            self.append(", ")?;
                        }
                        push(&mut work, Event::ScopedStyleList(values, index + 1, nested))?;
                        push(
                            &mut work,
                            match value {
                                CssScopedStyleSelector::Selector(value) => {
                                    Event::LiteralMember(value, None, nested)
                                }
                                CssScopedStyleSelector::Relative(value) => Event::LiteralMember(
                                    value.selector(),
                                    Some(value.combinator()),
                                    nested,
                                ),
                            },
                        )?;
                    }
                }
                Event::ScopeList(values, index, nested) => {
                    if let Some(value) = values.get(index) {
                        if index != 0 {
                            self.append(", ")?;
                        }
                        push(&mut work, Event::ScopeList(values, index + 1, nested))?;
                        push(
                            &mut work,
                            match value {
                                CssScopeSelector::Selector(value) => {
                                    Event::LiteralMember(value, None, nested)
                                }
                                CssScopeSelector::Relative(value) => Event::LiteralMember(
                                    value.selector(),
                                    Some(value.combinator()),
                                    nested,
                                ),
                            },
                        )?;
                    }
                }
                Event::Relative(value, grammar) => {
                    self.node()?;
                    self.combinator(value.combinator(), true)?;
                    push(&mut work, Event::Selector(value.selector(), grammar))?;
                }
                Event::Attribute(value) => {
                    self.node()?;
                    self.append("[")?;
                    self.output_name_prefix(
                        value.qualified_name().prefix(),
                        value.qualified_name().namespace(),
                        false,
                        role,
                        namespaces,
                    )?;
                    self.selector_identifier(value.name().as_str())?;
                    let (operator, string) = match value.matcher() {
                        CssAttributeMatcher::Exists => ("", None),
                        CssAttributeMatcher::Equals(s) => ("=", Some(s)),
                        CssAttributeMatcher::Includes(s) => ("~=", Some(s)),
                        CssAttributeMatcher::DashMatch(s) => ("|=", Some(s)),
                        CssAttributeMatcher::Prefix(s) => ("^=", Some(s)),
                        CssAttributeMatcher::Suffix(s) => ("$=", Some(s)),
                        CssAttributeMatcher::Substring(s) => ("*=", Some(s)),
                    };
                    self.append(operator)?;
                    if let Some(string) = string {
                        self.append_string(string)?;
                    }
                    self.append(match value.case_sensitivity() {
                        CssAttributeCaseSensitivity::DocumentDefault => "",
                        CssAttributeCaseSensitivity::AsciiCaseInsensitive => " i",
                        CssAttributeCaseSensitivity::ExplicitSensitive => " s",
                    })?;
                    self.append("]")?;
                }
                Event::Nth(value) => {
                    self.node()?;
                    match value {
                        CssNthPattern::Odd => self.append("2n+1")?,
                        CssNthPattern::Even => self.append("2n")?,
                        CssNthPattern::Integer(value) => self.append(&value.to_string())?,
                        CssNthPattern::AnPlusB(value) => {
                            let (a, b) = (value.a(), value.b());
                            if a == 0 {
                                self.append(&b.to_string())?;
                            } else {
                                if a == -1 {
                                    self.append("-")?;
                                } else if a != 1 {
                                    self.append(&a.to_string())?;
                                }
                                self.append("n")?;
                                if b != 0 {
                                    self.append(if b < 0 { "-" } else { "+" })?;
                                    self.append(&i64::from(b).abs().to_string())?;
                                }
                            }
                        }
                    }
                }
                Event::Pseudo(value, grammar) => {
                    if !grammar.admits_pseudo(value) {
                        return Err(CssSpecifiedValueSerializationError::new(
                            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
                        ));
                    }
                    self.node()?;
                    match value {
                        CssPseudoClass::HostFunction(arg) | CssPseudoClass::HostContext(arg) => {
                            self.append(if matches!(value, CssPseudoClass::HostFunction(_)) {
                                ":host("
                            } else {
                                ":host-context("
                            })?;
                            push(&mut work, Event::Text(")"))?;
                            push_argument(
                                &mut work,
                                Event::Compound(arg.compound(), grammar.compound_arguments()),
                                role,
                            )?;
                        }
                        CssPseudoClass::Not(list)
                        | CssPseudoClass::Is(list)
                        | CssPseudoClass::Where(list) => {
                            self.append(match value {
                                CssPseudoClass::Not(_) => ":not(",
                                CssPseudoClass::Is(_) => ":is(",
                                _ => ":where(",
                            })?;
                            push(&mut work, Event::Text(")"))?;
                            push_argument(
                                &mut work,
                                Event::PseudoList(
                                    list.items(),
                                    0,
                                    grammar.logical_arguments(),
                                    !matches!(value, CssPseudoClass::Not(_)),
                                ),
                                role,
                            )?;
                        }
                        CssPseudoClass::Has(list) => {
                            self.append(":has(")?;
                            push(&mut work, Event::Text(")"))?;
                            push_argument(
                                &mut work,
                                Event::RelativeList(
                                    list.selectors(),
                                    0,
                                    grammar.relative_arguments(),
                                ),
                                role,
                            )?;
                        }
                        CssPseudoClass::NthChild(pattern)
                        | CssPseudoClass::NthLastChild(pattern) => {
                            self.append(if matches!(value, CssPseudoClass::NthChild(_)) {
                                ":nth-child("
                            } else {
                                ":nth-last-child("
                            })?;
                            push(&mut work, Event::Text(")"))?;
                            if let Some(list) = pattern.selector_list() {
                                push_argument(
                                    &mut work,
                                    Event::PseudoList(
                                        list.items(),
                                        0,
                                        grammar.independent_arguments(),
                                        false,
                                    ),
                                    role,
                                )?;
                                push(&mut work, Event::Text(" of "))?;
                            }
                            push(&mut work, Event::Nth(pattern.pattern()))?;
                        }
                        CssPseudoClass::NthOfType(pattern)
                        | CssPseudoClass::NthLastOfType(pattern) => {
                            self.append(if matches!(value, CssPseudoClass::NthOfType(_)) {
                                ":nth-of-type("
                            } else {
                                ":nth-last-of-type("
                            })?;
                            push(&mut work, Event::Text(")"))?;
                            push(&mut work, Event::Nth(*pattern))?;
                        }
                        CssPseudoClass::Dir(dir) => {
                            self.append(":dir(")?;
                            self.selector_identifier(dir.as_str())?;
                            self.append(")")?;
                        }
                        CssPseudoClass::Lang(list) => {
                            self.append(":lang(")?;
                            for (i, range) in list.ranges().iter().enumerate() {
                                if i != 0 {
                                    self.append(", ")?;
                                }
                                self.node()?;
                                self.append_string(range.as_str())?;
                            }
                            self.append(")")?;
                        }
                        CssPseudoClass::Host => self.append(":host")?,
                        CssPseudoClass::Root => self.append(":root")?,
                        CssPseudoClass::Scope => self.append(":scope")?,
                        CssPseudoClass::Defined => self.append(":defined")?,
                        CssPseudoClass::AnyLink => self.append(":any-link")?,
                        CssPseudoClass::Link => self.append(":link")?,
                        CssPseudoClass::Visited => self.append(":visited")?,
                        CssPseudoClass::Target => self.append(":target")?,
                        CssPseudoClass::Current => self.append(":current")?,
                        CssPseudoClass::Hover => self.append(":hover")?,
                        CssPseudoClass::Active => self.append(":active")?,
                        CssPseudoClass::Focus => self.append(":focus")?,
                        CssPseudoClass::FocusVisible => self.append(":focus-visible")?,
                        CssPseudoClass::FocusWithin => self.append(":focus-within")?,
                        CssPseudoClass::Disabled => self.append(":disabled")?,
                        CssPseudoClass::Enabled => self.append(":enabled")?,
                        CssPseudoClass::Checked => self.append(":checked")?,
                        CssPseudoClass::Unchecked => self.append(":unchecked")?,
                        CssPseudoClass::Required => self.append(":required")?,
                        CssPseudoClass::Optional => self.append(":optional")?,
                        CssPseudoClass::Valid => self.append(":valid")?,
                        CssPseudoClass::Invalid => self.append(":invalid")?,
                        CssPseudoClass::UserValid => self.append(":user-valid")?,
                        CssPseudoClass::UserInvalid => self.append(":user-invalid")?,
                        CssPseudoClass::PlaceholderShown => self.append(":placeholder-shown")?,
                        CssPseudoClass::Autofill => self.append(":autofill")?,
                        CssPseudoClass::FirstChild => self.append(":first-child")?,
                        CssPseudoClass::LastChild => self.append(":last-child")?,
                        CssPseudoClass::OnlyChild => self.append(":only-child")?,
                        CssPseudoClass::Empty => self.append(":empty")?,
                        CssPseudoClass::FirstOfType => self.append(":first-of-type")?,
                        CssPseudoClass::LastOfType => self.append(":last-of-type")?,
                        CssPseudoClass::OnlyOfType => self.append(":only-of-type")?,
                        CssPseudoClass::Playing => self.append(":playing")?,
                        CssPseudoClass::Paused => self.append(":paused")?,
                        CssPseudoClass::Seeking => self.append(":seeking")?,
                        CssPseudoClass::Buffering => self.append(":buffering")?,
                        CssPseudoClass::Stalled => self.append(":stalled")?,
                        CssPseudoClass::Muted => self.append(":muted")?,
                        CssPseudoClass::VolumeLocked => self.append(":volume-locked")?,
                        CssPseudoClass::Open => self.append(":open")?,
                        CssPseudoClass::Modal => self.append(":modal")?,
                        CssPseudoClass::Fullscreen => self.append(":fullscreen")?,
                        CssPseudoClass::PictureInPicture => self.append(":picture-in-picture")?,
                        CssPseudoClass::PopoverOpen => self.append(":popover-open")?,
                        CssPseudoClass::Default => self.append(":default")?,
                        CssPseudoClass::Indeterminate => self.append(":indeterminate")?,
                        CssPseudoClass::ReadOnly => self.append(":read-only")?,
                        CssPseudoClass::ReadWrite => self.append(":read-write")?,
                        CssPseudoClass::InRange => self.append(":in-range")?,
                        CssPseudoClass::OutOfRange => self.append(":out-of-range")?,
                    }
                }
                Event::Element(value, grammar) => {
                    self.node()?;
                    match value {
                        CssPseudoElement::Slotted(arg) => {
                            self.append("::slotted(")?;
                            push(&mut work, Event::Text(")"))?;
                            push_argument(
                                &mut work,
                                Event::Compound(arg.compound(), grammar.compound_arguments()),
                                role,
                            )?;
                        }
                        CssPseudoElement::Part(list) => {
                            self.append("::part(")?;
                            for (i, name) in list.names().iter().enumerate() {
                                if i != 0 {
                                    self.append(" ")?;
                                }
                                self.node()?;
                                self.selector_identifier(name.as_str())?;
                            }
                            self.append(")")?;
                        }
                        CssPseudoElement::Highlight(name) => {
                            self.append("::highlight(")?;
                            self.node()?;
                            self.selector_identifier(name.as_str())?;
                            self.append(")")?;
                        }
                        CssPseudoElement::ViewTransition => self.append("::view-transition")?,
                        CssPseudoElement::ViewTransitionGroup(argument)
                        | CssPseudoElement::ViewTransitionImagePair(argument)
                        | CssPseudoElement::ViewTransitionOld(argument)
                        | CssPseudoElement::ViewTransitionNew(argument) => {
                            self.append(match value {
                                CssPseudoElement::ViewTransitionGroup(_) => {
                                    "::view-transition-group("
                                }
                                CssPseudoElement::ViewTransitionImagePair(_) => {
                                    "::view-transition-image-pair("
                                }
                                CssPseudoElement::ViewTransitionOld(_) => "::view-transition-old(",
                                CssPseudoElement::ViewTransitionNew(_) => "::view-transition-new(",
                                _ => unreachable!("named transition arm"),
                            })?;
                            self.node()?;
                            match argument {
                                CssViewTransitionNameSelector::Wildcard => self.append("*")?,
                                CssViewTransitionNameSelector::Name(name) => {
                                    self.selector_identifier(name.as_str())?
                                }
                            }
                            self.append(")")?;
                        }
                        CssPseudoElement::Before => self.append("::before")?,
                        CssPseudoElement::After => self.append("::after")?,
                        CssPseudoElement::FirstLine => self.append("::first-line")?,
                        CssPseudoElement::FirstLetter => self.append("::first-letter")?,
                        CssPseudoElement::Prefix => self.append("::prefix")?,
                        CssPseudoElement::Suffix => self.append("::suffix")?,
                        CssPseudoElement::Marker => self.append("::marker")?,
                        CssPseudoElement::Selection => self.append("::selection")?,
                        CssPseudoElement::SearchText => self.append("::search-text")?,
                        CssPseudoElement::TargetText => self.append("::target-text")?,
                        CssPseudoElement::SpellingError => self.append("::spelling-error")?,
                        CssPseudoElement::GrammarError => self.append("::grammar-error")?,
                        CssPseudoElement::Placeholder => self.append("::placeholder")?,
                        CssPseudoElement::Backdrop => self.append("::backdrop")?,
                        CssPseudoElement::FileSelectorButton => {
                            self.append("::file-selector-button")?
                        }
                        CssPseudoElement::DetailsContent => self.append("::details-content")?,
                        CssPseudoElement::UnknownWebkit(name) => {
                            self.append("::")?;
                            self.selector_identifier(name.as_str())?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod invalid_nesting_context_tests {
    use super::*;

    #[test]
    fn incomparable_receiver_cannot_trade_compound_restriction_for_has_restriction() {
        let source = ":host(:is(.a &))";
        let host = crate::parse_selector(source, &CssNamespaceContext::default())
            .syntax()
            .clone()
            .unwrap();
        let CssSelector::PseudoClass(CssPseudoClass::HostFunction(argument)) = &host else {
            panic!("Host")
        };
        let [CssPseudoClass::Is(list)] = argument.compound().pseudo_classes() else {
            panic!("logical compound argument")
        };
        let detached = CssSelector::PseudoClass(CssPseudoClass::Is(list.clone()));
        let before = detached.clone();
        // This is the actual provider receiving grammar. Relative arguments
        // forbid Has but re-enable complexes, incomparable to the original proof.
        let grammar = CssSelectorGrammarContext::ORDINARY.relative_arguments();
        let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::default());
        assert_eq!(
            writer
                .selector_events(Event::Selector(&detached, grammar))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
        );
        assert_eq!(detached, before);
        // Keeping the compound owner while tightening Has preserves both proofs.
        let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::default());
        writer
            .selector_events(Event::Selector(&host, grammar))
            .unwrap();
        assert_eq!(writer.css, source);
    }
}
