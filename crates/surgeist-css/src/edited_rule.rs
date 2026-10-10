//! Immutable borrowed payload views for CSSOM-owned edits. No live identity is stored.
use crate::{
    CssContainerPrelude, CssLayerName, CssMediaQueryList, CssRule, CssRuleCssomSerializationError,
    CssScopedRule, CssScopedStyleSelectorList, CssSpecifiedDeclarationBlock,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssStyleSelectorList, CssSupportsCondition,
    specified_rule_serialization::SpecifiedRuleWriter,
};

/// A CSS-owned, already checked group prelude. Evaluation stays with its consumer.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssEditedGroupPreludeRef<'a> {
    Media(&'a CssMediaQueryList),
    Supports(&'a CssSupportsCondition),
    Container(&'a CssContainerPrelude),
    Layer(Option<&'a CssLayerName>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum EditedStyleSelectorsRef<'a> {
    Ordinary(&'a CssStyleSelectorList),
    Scoped(&'a CssScopedStyleSelectorList),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum EditedRuleKind<'a> {
    Parsed(&'a CssRule),
    ParsedScoped(&'a CssScopedRule),
    Style(
        EditedStyleSelectorsRef<'a>,
        &'a CssSpecifiedDeclarationBlock,
        &'a [CssEditedRuleView<'a>],
    ),
    Group(CssEditedGroupPreludeRef<'a>, &'a [CssEditedRuleView<'a>]),
    NestedDeclarations(&'a CssSpecifiedDeclarationBlock),
    Page(crate::CssPageRuleView<'a>),
    Keyframes(crate::CssKeyframesRuleView<'a>),
}

/// A checked immutable formatting view over edited payloads and child order.
///
/// Edited wrappers are programmatic and have no source position. Borrowed parsed
/// descendants, selectors, queries and terminals keep their actual origins. CSSOM
/// owns placement, live identity, parent links, revisions and mutation. A retained
/// empty nested-declaration child is expressible and serializes to an empty string.
#[derive(Clone, Copy, Debug)]
pub struct CssEditedRuleView<'a> {
    pub(crate) kind: EditedRuleKind<'a>,
}
impl<'a> CssEditedRuleView<'a> {
    /// Borrows a complete existing rule without changing its provenance.
    #[must_use]
    pub const fn from_rule(rule: &'a CssRule) -> Self {
        Self {
            kind: EditedRuleKind::Parsed(rule),
        }
    }
    /// Borrows an existing scoped rule with its checked selector context intact.
    #[must_use]
    pub const fn from_scoped_rule(rule: &'a CssScopedRule) -> Self {
        Self {
            kind: EditedRuleKind::ParsedScoped(rule),
        }
    }
    pub fn try_style(
        selectors: &'a CssStyleSelectorList,
        declarations: &'a CssSpecifiedDeclarationBlock,
        children: &'a [Self],
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::try_style_with_limits(
            selectors,
            declarations,
            children,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    /// Checks cumulative provider work and complete specified output without reparsing.
    pub fn try_style_with_limits(
        selectors: &'a CssStyleSelectorList,
        declarations: &'a CssSpecifiedDeclarationBlock,
        children: &'a [Self],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::ordinary(declarations)?;
        Self {
            kind: EditedRuleKind::Style(
                EditedStyleSelectorsRef::Ordinary(selectors),
                declarations,
                children,
            ),
        }
        .checked(limits)
    }
    /// Borrows scoped style selectors without converting their symbolic scope
    /// anchors to ordinary nesting references or inventing an authored rule.
    pub fn try_scoped_style(
        selectors: &'a CssScopedStyleSelectorList,
        declarations: &'a CssSpecifiedDeclarationBlock,
        children: &'a [Self],
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::try_scoped_style_with_limits(
            selectors,
            declarations,
            children,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn try_scoped_style_with_limits(
        selectors: &'a CssScopedStyleSelectorList,
        declarations: &'a CssSpecifiedDeclarationBlock,
        children: &'a [Self],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::ordinary(declarations)?;
        Self {
            kind: EditedRuleKind::Style(
                EditedStyleSelectorsRef::Scoped(selectors),
                declarations,
                children,
            ),
        }
        .checked(limits)
    }
    pub fn try_group(
        prelude: CssEditedGroupPreludeRef<'a>,
        children: &'a [Self],
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::try_group_with_limits(
            prelude,
            children,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    /// Checks the shared specified providers. Literal CSSOM availability is checked
    /// separately by serialization, retaining the existing typed unavailable result.
    pub fn try_group_with_limits(
        prelude: CssEditedGroupPreludeRef<'a>,
        children: &'a [Self],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        let view = Self {
            kind: EditedRuleKind::Group(prelude, children),
        };
        view.to_specified_css_with_limits(limits)?;
        Ok(view)
    }
    /// Embeds current selected Page state and its ordered, domain-checked margin children.
    /// This checks formatting, while contextual placement remains with the consumer.
    pub fn try_page(
        page: crate::CssPageRuleView<'a>,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::try_page_with_limits(page, CssSpecifiedValueSerializationLimits::default())
    }
    pub fn try_page_with_limits(
        page: crate::CssPageRuleView<'a>,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self {
            kind: EditedRuleKind::Page(page),
        }
        .checked(limits)
    }
    /// Embeds a checked current whole-keyframes payload without reconstructing authored rules.
    pub fn try_keyframes(
        keyframes: crate::CssKeyframesRuleView<'a>,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::try_keyframes_with_limits(keyframes, CssSpecifiedValueSerializationLimits::default())
    }
    pub fn try_keyframes_with_limits(
        keyframes: crate::CssKeyframesRuleView<'a>,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self {
            kind: EditedRuleKind::Keyframes(keyframes),
        }
        .checked(limits)
    }
    pub fn try_nested_declarations(
        declarations: &'a CssSpecifiedDeclarationBlock,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::try_nested_declarations_with_limits(
            declarations,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }
    pub fn try_nested_declarations_with_limits(
        declarations: &'a CssSpecifiedDeclarationBlock,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::ordinary(declarations)?;
        Self {
            kind: EditedRuleKind::NestedDeclarations(declarations),
        }
        .checked(limits)
    }
    fn ordinary(
        block: &CssSpecifiedDeclarationBlock,
    ) -> Result<(), CssRuleCssomSerializationError> {
        if block.is_ordinary() {
            return Ok(());
        }
        Err(CssRuleCssomSerializationError::new(
            CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::UnserializableBoundary,
            )
            .into(),
            Vec::new(),
            None,
        ))
    }
    fn checked(
        self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        self.to_specified_css_with_limits(limits)?;
        Ok(self)
    }
    /// The original parsed rule, when this view borrows one; edited wrappers return None.
    #[must_use]
    pub const fn parsed_rule(self) -> Option<&'a CssRule> {
        match self.kind {
            EditedRuleKind::Parsed(rule) => Some(rule),
            _ => None,
        }
    }
    /// The original scoped rule when borrowed, absent for edited wrappers.
    #[must_use]
    pub const fn parsed_scoped_rule(self) -> Option<&'a CssScopedRule> {
        match self.kind {
            EditedRuleKind::ParsedScoped(rule) => Some(rule),
            _ => None,
        }
    }
    pub fn serialize_cssom(self) -> Result<String, CssRuleCssomSerializationError> {
        self.serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_cssom_with_limits(
        self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssRuleCssomSerializationError> {
        self.serialize(limits, true)
    }
    pub fn to_specified_css(self) -> Result<String, CssRuleCssomSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn to_specified_css_with_limits(
        self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssRuleCssomSerializationError> {
        self.serialize(limits, false)
    }
    fn serialize(
        self,
        limits: CssSpecifiedValueSerializationLimits,
        cssom: bool,
    ) -> Result<String, CssRuleCssomSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.append_edited_rule_graph(self, cssom)?;
        Ok(writer.css)
    }
}
