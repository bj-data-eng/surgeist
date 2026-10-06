use super::{CssNamespacePrefix, CssValueOrigin};

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssSelector {
    Tag(String),
    Key(String),
    Class(String),
    PseudoClass(CssPseudoClass),
    Compound(CssCompoundSelector),
    Complex(CssComplexSelector),
}

impl CssSelector {
    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        match self {
            Self::Tag(_) | Self::Key(_) | Self::Class(_) => false,
            Self::PseudoClass(pseudo_class) => pseudo_class.has_pseudo_elements(),
            Self::Compound(selector) => selector.has_pseudo_elements(),
            Self::Complex(selector) => selector.has_pseudo_elements(),
        }
    }

    fn into_compound_selector(self) -> CssCompoundSelector {
        match self {
            Self::Tag(tag) => {
                CssCompoundSelector::new(Some(tag), None, Vec::new(), Vec::new(), Vec::new())
            }
            Self::Key(key) => {
                CssCompoundSelector::new(None, Some(key), Vec::new(), Vec::new(), Vec::new())
            }
            Self::Class(class) => {
                CssCompoundSelector::new(None, None, vec![class], Vec::new(), Vec::new())
            }
            Self::PseudoClass(pseudo_class) => {
                CssCompoundSelector::new(None, None, Vec::new(), Vec::new(), vec![pseudo_class])
            }
            Self::Compound(selector) => selector,
            Self::Complex(_) => {
                unreachable!("complex selectors are handled before compound conversion")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssComplexSelector {
    first: CssCompoundSelector,
    rest: Vec<CssComplexSelectorPart>,
}

impl CssComplexSelector {
    /// Checks complete ordinary assembly with the default cumulative output limits.
    pub fn try_new(
        first: CssCompoundSelector,
        rest: Vec<CssComplexSelectorPart>,
    ) -> Result<Self, CssSelectorConstructionError> {
        Self::try_new_with_limits(
            first,
            rest,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }

    /// Checks placement and the complete graph within one shared budget.
    pub fn try_new_with_limits(
        first: CssCompoundSelector,
        rest: Vec<CssComplexSelectorPart>,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        if rest.is_empty() {
            return Err(CssSelectorConstructionError::new(
                CssSelectorConstructionErrorKind::EmptyComplexRest,
            ));
        }
        if let Some(compound_index) = non_terminal_pseudo_element_index(&first, &rest) {
            return Err(CssSelectorConstructionError::new(
                CssSelectorConstructionErrorKind::NonTerminalPseudoElement { compound_index },
            ));
        }
        let value = Self::new(first, rest);
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer
            .complex_selector(&value)
            .map_err(CssSelectorConstructionError::specified)?;
        Ok(value)
    }

    #[must_use]
    pub(crate) fn new(first: CssCompoundSelector, rest: Vec<CssComplexSelectorPart>) -> Self {
        debug_assert!(!rest.is_empty());
        debug_assert!(!complex_selector_has_non_terminal_pseudo_elements(
            &first, &rest
        ));
        Self { first, rest }
    }

    #[must_use]
    pub const fn first(&self) -> &CssCompoundSelector {
        &self.first
    }

    #[must_use]
    pub fn rest(&self) -> &[CssComplexSelectorPart] {
        &self.rest
    }

    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.first.has_pseudo_elements()
            || self
                .rest
                .iter()
                .any(|part| part.selector().has_pseudo_elements())
    }
}

pub(crate) fn complex_selector_has_non_terminal_pseudo_elements(
    first: &CssCompoundSelector,
    rest: &[CssComplexSelectorPart],
) -> bool {
    non_terminal_pseudo_element_index(first, rest).is_some()
}

fn non_terminal_pseudo_element_index(
    first: &CssCompoundSelector,
    rest: &[CssComplexSelectorPart],
) -> Option<usize> {
    if first.has_pseudo_elements() {
        return Some(0);
    }
    rest.iter()
        .take(rest.len().saturating_sub(1))
        .position(|part| part.selector().has_pseudo_elements())
        .map(|index| index + 1)
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssComplexSelectorPart {
    combinator: CssSelectorCombinator,
    selector: CssCompoundSelector,
}

impl CssComplexSelectorPart {
    /// Checks an ordinary compound and its relationship with default output limits.
    /// Admission here does not prove that a pseudo-element is terminal in a complex.
    pub fn try_new(
        combinator: CssSelectorCombinator,
        selector: CssCompoundSelector,
    ) -> Result<Self, CssSelectorConstructionError> {
        Self::try_new_with_limits(
            combinator,
            selector,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }

    /// Charges the compound and canonical relationship bytes cumulatively.
    pub fn try_new_with_limits(
        combinator: CssSelectorCombinator,
        selector: CssCompoundSelector,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        let value = Self::new(combinator, selector);
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer
            .complex_selector_part(&value)
            .map_err(CssSelectorConstructionError::specified)?;
        Ok(value)
    }

    #[must_use]
    pub(crate) const fn new(
        combinator: CssSelectorCombinator,
        selector: CssCompoundSelector,
    ) -> Self {
        Self {
            combinator,
            selector,
        }
    }

    #[must_use]
    pub const fn combinator(&self) -> CssSelectorCombinator {
        self.combinator
    }

    #[must_use]
    pub const fn selector(&self) -> &CssCompoundSelector {
        &self.selector
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSelectorCombinator {
    Descendant,
    Child,
    NextSibling,
    SubsequentSibling,
    /// The authored `||` relationship between a column and its cells.
    Column,
}

/// Why checked authored selector construction could not return a complete value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSelectorConstructionErrorKind {
    /// A compound must contain at least one simple member.
    EmptyCompound,
    /// A strict selector list must contain at least one member.
    EmptyList,
    /// A complex must contain at least one relationship and following compound.
    EmptyComplexRest,
    /// A pseudo-element-bearing compound precedes the final compound.
    NonTerminalPseudoElement { compound_index: usize },
    /// A type or universal selector is duplicated or follows another member.
    InvalidTypePosition { member_index: usize },
    /// An attribute modifier was supplied without an attribute value.
    ModifierWithoutValue,
    /// A decoded attribute operand cannot preserve its string identity.
    InvalidAttributeValue(crate::CssComponentValueError),
    /// Intrinsic grammar or cumulative specified-output admission failed.
    Specified(crate::CssSpecifiedValueSerializationError),
}

/// Atomic construction failure, with no invented parsed-source coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSelectorConstructionError {
    kind: CssSelectorConstructionErrorKind,
}

impl CssSelectorConstructionError {
    const fn new(kind: CssSelectorConstructionErrorKind) -> Self {
        Self { kind }
    }
    const fn empty_list() -> Self {
        Self {
            kind: CssSelectorConstructionErrorKind::EmptyList,
        }
    }

    fn specified(error: crate::CssSpecifiedValueSerializationError) -> Self {
        Self {
            kind: CssSelectorConstructionErrorKind::Specified(error),
        }
    }

    /// Returns the complete typed construction cause.
    #[must_use]
    pub const fn kind(&self) -> &CssSelectorConstructionErrorKind {
        &self.kind
    }
}

impl std::fmt::Display for CssSelectorConstructionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            CssSelectorConstructionErrorKind::EmptyCompound => {
                formatter.write_str("a compound selector must contain at least one member")
            }
            CssSelectorConstructionErrorKind::EmptyList => {
                formatter.write_str("a selector list must contain at least one member")
            }
            CssSelectorConstructionErrorKind::Specified(error) => {
                std::fmt::Display::fmt(error, formatter)
            }
            CssSelectorConstructionErrorKind::EmptyComplexRest => {
                formatter.write_str("a complex selector must contain a following compound")
            }
            CssSelectorConstructionErrorKind::NonTerminalPseudoElement { compound_index } => {
                write!(
                    formatter,
                    "pseudo-element compound at index {compound_index} must be terminal"
                )
            }
            CssSelectorConstructionErrorKind::InvalidTypePosition { member_index } => write!(
                formatter,
                "type or universal selector at member index {member_index} must be unique and first"
            ),
            CssSelectorConstructionErrorKind::ModifierWithoutValue => {
                formatter.write_str("an attribute modifier requires a value")
            }
            CssSelectorConstructionErrorKind::InvalidAttributeValue(error) => {
                std::fmt::Display::fmt(error, formatter)
            }
        }
    }
}

impl std::error::Error for CssSelectorConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            CssSelectorConstructionErrorKind::EmptyCompound
            | CssSelectorConstructionErrorKind::EmptyList
            | CssSelectorConstructionErrorKind::EmptyComplexRest
            | CssSelectorConstructionErrorKind::NonTerminalPseudoElement { .. }
            | CssSelectorConstructionErrorKind::InvalidTypePosition { .. }
            | CssSelectorConstructionErrorKind::ModifierWithoutValue => None,
            CssSelectorConstructionErrorKind::InvalidAttributeValue(error) => Some(error),
            CssSelectorConstructionErrorKind::Specified(error) => Some(error),
        }
    }
}

/// A nonempty ordered list admitted under ordinary authored selector grammar.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSelectorList {
    selectors: Vec<CssSelector>,
}

impl CssSelectorList {
    /// Checks every ordinary member with the default cumulative output limits.
    pub fn try_new(selectors: Vec<CssSelector>) -> Result<Self, CssSelectorConstructionError> {
        Self::try_new_with_limits(
            selectors,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }

    /// Checks the complete graph and canonical output within one shared budget.
    /// No invalid member is dropped and no partially admitted list escapes.
    pub fn try_new_with_limits(
        selectors: Vec<CssSelector>,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        if selectors.is_empty() {
            return Err(CssSelectorConstructionError::empty_list());
        }
        let value = Self::new(selectors);
        value
            .to_specified_css_with_limits(limits)
            .map_err(CssSelectorConstructionError::specified)?;
        Ok(value)
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssSelector] {
        &self.selectors
    }
}

/// Complex-real selector arguments. Parsed and checked forgiving construction
/// can retain zero members; strict construction still requires a nonempty list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPseudoSelectorList {
    items: Vec<CssPseudoSelectorListItem>,
}

/// One ordered argument of a logical pseudo selector.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPseudoSelectorListItem {
    /// An admitted typed selector, checked in its receiving grammar.
    Selector(CssSelector),
    /// A parser-retained invalid member containing a delimiter `&`.
    InvalidNesting(CssInvalidNestingSelectorItem),
}

/// Invalid forgiving syntax retained exactly in its original parse environment.
///
/// This is a match-nothing, zero-specificity authored state, not an admitted
/// selector. Only Is/Where can receive it, in the original or a more restrictive
/// intrinsic grammar. Clone/output never rebinds namespaces: explicitly parsing
/// its emitted text with different bindings is a new operation and may admit a
/// different state. Source coordinates are metadata, not structural identity.
#[derive(Clone, Debug)]
pub struct CssInvalidNestingSelectorItem {
    origin: crate::CssParsedOrigin,
    components: crate::CssComponentValues,
    grammar: CssSelectorGrammarContext,
}

impl CssInvalidNestingSelectorItem {
    pub(crate) const fn new(
        origin: crate::CssParsedOrigin,
        components: crate::CssComponentValues,
        grammar: CssSelectorGrammarContext,
    ) -> Self {
        Self {
            origin,
            components,
            grammar,
        }
    }

    /// Complete original member, excluding its comma and including edge trivia.
    #[must_use]
    pub fn authored(&self) -> &str {
        &self.origin.source().as_str()[self.origin.span().start().byte_offset().value()
            ..self.origin.span().end().byte_offset().value()]
    }

    #[must_use]
    pub const fn origin(&self) -> &crate::CssParsedOrigin {
        &self.origin
    }

    #[must_use]
    pub const fn components(&self) -> &crate::CssComponentValues {
        &self.components
    }

    pub(crate) fn admitted_in(&self, grammar: CssSelectorGrammarContext) -> bool {
        grammar.at_least_as_restrictive_as(self.grammar)
    }
}

impl PartialEq for CssInvalidNestingSelectorItem {
    fn eq(&self, other: &Self) -> bool {
        self.authored() == other.authored() && self.grammar == other.grammar
    }
}

impl CssPseudoSelectorList {
    /// Checks a nonempty complex-real list with the default cumulative limits.
    pub fn try_new(selectors: Vec<CssSelector>) -> Result<Self, CssSelectorConstructionError> {
        Self::try_new_with_limits(
            selectors,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }

    /// Rejects an empty list or any invalid supplied member without filtering.
    pub fn try_new_with_limits(
        selectors: Vec<CssSelector>,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        if selectors.is_empty() {
            return Err(CssSelectorConstructionError::empty_list());
        }
        Self::try_new_forgiving_with_limits(selectors, limits)
    }

    /// Admits empty complex-real arguments for Is/Where. Invalid typed members
    /// fail construction; parsed member forgiveness is a separate operation.
    pub fn try_new_forgiving(
        selectors: Vec<CssSelector>,
    ) -> Result<Self, CssSelectorConstructionError> {
        Self::try_new_forgiving_with_limits(
            selectors,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }

    /// Checks every supplied member with one aggregate and cumulative budget,
    /// including the aggregate node of a valid empty forgiving list.
    pub fn try_new_forgiving_with_limits(
        selectors: Vec<CssSelector>,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer
            .pseudo_selector_members(&selectors)
            .map_err(CssSelectorConstructionError::specified)?;
        let mut items = Vec::new();
        items.try_reserve(selectors.len()).map_err(|_| {
            CssSelectorConstructionError::specified(
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                ),
            )
        })?;
        items.extend(
            selectors
                .into_iter()
                .map(CssPseudoSelectorListItem::Selector),
        );
        Ok(Self { items })
    }

    // Owning private writer fixtures may deliberately bypass public admission.
    #[cfg(test)]
    pub(crate) fn new(selectors: Vec<CssSelector>) -> Self {
        Self {
            items: selectors
                .into_iter()
                .map(CssPseudoSelectorListItem::Selector)
                .collect(),
        }
    }

    pub(crate) const fn from_parsed_items(items: Vec<CssPseudoSelectorListItem>) -> Self {
        Self { items }
    }

    /// Every argument in source order, including explicit retained invalid state.
    #[must_use]
    pub fn items(&self) -> &[CssPseudoSelectorListItem] {
        &self.items
    }

    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.items.iter().any(|item| match item {
            CssPseudoSelectorListItem::Selector(selector) => selector.has_pseudo_elements(),
            CssPseudoSelectorListItem::InvalidNesting(_) => false,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeSelector {
    combinator: CssSelectorCombinator,
    selector: CssSelector,
}

impl CssRelativeSelector {
    #[must_use]
    pub const fn new(combinator: CssSelectorCombinator, selector: CssSelector) -> Self {
        Self {
            combinator,
            selector,
        }
    }

    #[must_use]
    pub const fn combinator(&self) -> CssSelectorCombinator {
        self.combinator
    }

    #[must_use]
    pub const fn selector(&self) -> &CssSelector {
        &self.selector
    }
}

/// A nonempty general relative list. Has attachment applies its additional
/// no-pseudo-element and no-nested-Has restrictions at the consuming boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeSelectorList {
    selectors: Vec<CssRelativeSelector>,
}

impl CssRelativeSelectorList {
    /// Checks every general relative member with default cumulative limits.
    pub fn try_new(
        selectors: Vec<CssRelativeSelector>,
    ) -> Result<Self, CssSelectorConstructionError> {
        Self::try_new_with_limits(
            selectors,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }

    /// Checks all leading carriers and their complete ordinary child graphs.
    /// A valid terminal pseudo-element or Has child is admitted here; attaching
    /// this list as Has arguments independently enforces the narrower context.
    pub fn try_new_with_limits(
        selectors: Vec<CssRelativeSelector>,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        if selectors.is_empty() {
            return Err(CssSelectorConstructionError::empty_list());
        }
        let value = Self::new(selectors);
        value
            .to_specified_css_with_limits(limits)
            .map_err(CssSelectorConstructionError::specified)?;
        Ok(value)
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssRelativeSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssRelativeSelector] {
        &self.selectors
    }

    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.selectors
            .iter()
            .any(|selector| selector.selector().has_pseudo_elements())
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPseudoClass {
    /// The shadow host, without an argument.
    Host,
    /// The shadow host constrained by one compound selector.
    HostFunction(CssCompoundSelectorArgument),
    /// A host whose context satisfies one compound selector.
    HostContext(CssCompoundSelectorArgument),
    Root,
    Scope,
    Link,
    Visited,
    Target,
    Dir(CssDirectionality),
    Lang(CssLanguageRangeList),
    Hover,
    Active,
    Focus,
    FocusVisible,
    FocusWithin,
    Disabled,
    Enabled,
    Checked,
    Required,
    Optional,
    Valid,
    Invalid,
    PlaceholderShown,
    FirstChild,
    LastChild,
    OnlyChild,
    Empty,
    NthChild(CssNthChildPattern),
    NthLastChild(CssNthChildPattern),
    FirstOfType,
    LastOfType,
    OnlyOfType,
    NthOfType(CssNthPattern),
    NthLastOfType(CssNthPattern),
    Not(CssPseudoSelectorList),
    Is(CssPseudoSelectorList),
    Where(CssPseudoSelectorList),
    Has(CssRelativeSelectorList),
    Modal,
    Fullscreen,
    PopoverOpen,
    Default,
    Indeterminate,
    ReadOnly,
    ReadWrite,
    InRange,
    OutOfRange,
}

impl CssPseudoClass {
    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        match self {
            Self::NthChild(pattern) | Self::NthLastChild(pattern) => pattern.has_pseudo_elements(),
            Self::Not(selectors) | Self::Is(selectors) | Self::Where(selectors) => {
                selectors.has_pseudo_elements()
            }
            Self::Has(selectors) => selectors.has_pseudo_elements(),
            Self::Host
            | Self::HostFunction(_)
            | Self::HostContext(_)
            | Self::Root
            | Self::Scope
            | Self::Link
            | Self::Visited
            | Self::Target
            | Self::Dir(_)
            | Self::Lang(_)
            | Self::Hover
            | Self::Active
            | Self::Focus
            | Self::FocusVisible
            | Self::FocusWithin
            | Self::Disabled
            | Self::Enabled
            | Self::Checked
            | Self::Required
            | Self::Optional
            | Self::Valid
            | Self::Invalid
            | Self::PlaceholderShown
            | Self::FirstChild
            | Self::LastChild
            | Self::OnlyChild
            | Self::Empty
            | Self::FirstOfType
            | Self::LastOfType
            | Self::OnlyOfType
            | Self::NthOfType(_)
            | Self::NthLastOfType(_)
            | Self::Modal
            | Self::Fullscreen
            | Self::PopoverOpen
            | Self::Default
            | Self::Indeterminate
            | Self::ReadOnly
            | Self::ReadWrite
            | Self::InRange
            | Self::OutOfRange => false,
        }
    }
}

/// A checked decoded directionality identifier; unknown values remain authored syntax.
/// Equality retains component spelling and provenance, without evaluating direction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssDirectionality {
    component: Box<crate::CssComponentValue>,
}

impl CssDirectionality {
    /// Constructs a decoded identifier, escaping it as needed. Empty and NUL fail.
    pub fn try_new(value: impl Into<String>) -> Result<Self, crate::CssComponentValueError> {
        Self::from_component(crate::CssComponentValue::try_ident(value)?)
    }

    pub(crate) fn from_component(
        component: crate::CssComponentValue,
    ) -> Result<Self, crate::CssComponentValueError> {
        if matches!(
            component.view(),
            crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Ident(_))
        ) {
            Ok(Self {
                component: Box::new(component),
            })
        } else {
            Err(crate::CssComponentValueError::new(
                crate::CssComponentValueErrorKind::InvalidIdentifier,
                component.origin().clone(),
            ))
        }
    }

    /// Returns the decoded, case-preserving identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self.component.view() {
            crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Ident(value)) => value,
            _ => unreachable!("checked directionality identifier"),
        }
    }

    /// Returns the original token origin or explicit programmatic provenance.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }

    /// Serializes the argument as a canonical identifier, without the pseudo-class.
    #[must_use]
    pub fn to_css_string(&self) -> String {
        cssparser::ToCss::to_css_string(&cssparser::Token::Ident(self.as_str().into()))
    }
}

/// The authored token form of a language range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssLanguageRangeKind {
    Identifier,
    String,
}

/// One checked identifier or string in a Selectors 4 language list.
/// Equality includes exact token spelling and provenance; it is not language matching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssLanguageRange {
    component: crate::CssComponentValue,
}

impl CssLanguageRange {
    /// Constructs a decoded identifier, escaping punctuation or spaces as needed.
    pub fn try_ident(value: impl Into<String>) -> Result<Self, crate::CssComponentValueError> {
        Self::from_component(crate::CssComponentValue::try_ident(value)?)
    }

    /// Constructs a decoded string, including empty strings; NUL is rejected.
    pub fn try_string(value: impl Into<String>) -> Result<Self, crate::CssComponentValueError> {
        Self::from_component(crate::CssComponentValue::try_string(value)?)
    }

    pub(crate) fn from_component(
        component: crate::CssComponentValue,
    ) -> Result<Self, crate::CssComponentValueError> {
        if matches!(
            component.view(),
            crate::CssComponentValueRef::Token(
                crate::CssValueTokenRef::Ident(_) | crate::CssValueTokenRef::String(_)
            )
        ) {
            Ok(Self { component })
        } else {
            Err(crate::CssComponentValueError::new(
                crate::CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ))
        }
    }

    /// Returns the decoded range, without case folding or language validation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self.component.view() {
            crate::CssComponentValueRef::Token(
                crate::CssValueTokenRef::Ident(value) | crate::CssValueTokenRef::String(value),
            ) => value,
            _ => unreachable!("checked language range token"),
        }
    }

    /// Returns the identifier or string token form.
    #[must_use]
    pub fn kind(&self) -> CssLanguageRangeKind {
        match self.component.view() {
            crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Ident(_)) => {
                CssLanguageRangeKind::Identifier
            }
            crate::CssComponentValueRef::Token(crate::CssValueTokenRef::String(_)) => {
                CssLanguageRangeKind::String
            }
            _ => unreachable!("checked language range token"),
        }
    }

    /// Returns the original token origin or explicit programmatic provenance.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }

    /// Serializes the argument token canonically, preserving its token form.
    #[must_use]
    pub fn to_css_string(&self) -> String {
        let token = match self.kind() {
            CssLanguageRangeKind::Identifier => cssparser::Token::Ident(self.as_str().into()),
            CssLanguageRangeKind::String => cssparser::Token::QuotedString(self.as_str().into()),
        };
        cssparser::ToCss::to_css_string(&token)
    }
}

/// Construction failed because a language range list was empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssEmptyLanguageRangeList;

impl std::fmt::Display for CssEmptyLanguageRangeList {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a language range list must contain at least one range")
    }
}
impl std::error::Error for CssEmptyLanguageRangeList {}

/// A nonempty authored list, preserving range order, duplicates and token origins.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssLanguageRangeList {
    ranges: Vec<CssLanguageRange>,
}

impl CssLanguageRangeList {
    /// Constructs a nonempty list of independently checked ranges.
    pub fn try_new(ranges: Vec<CssLanguageRange>) -> Result<Self, CssEmptyLanguageRangeList> {
        if ranges.is_empty() {
            Err(CssEmptyLanguageRangeList)
        } else {
            Ok(Self { ranges })
        }
    }

    /// Constructs a singleton list without discarding its token origin.
    #[must_use]
    pub fn single(range: CssLanguageRange) -> Self {
        Self {
            ranges: vec![range],
        }
    }

    /// Returns the complete ordered, immutable range list.
    #[must_use]
    pub fn ranges(&self) -> &[CssLanguageRange] {
        &self.ranges
    }

    /// Serializes the arguments with comma-space separators, without `:lang()`.
    #[must_use]
    pub fn to_css_string(&self) -> String {
        self.ranges
            .iter()
            .map(CssLanguageRange::to_css_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// One checked compound argument for `:host()`, `:host-context()` or `::slotted()`.
/// Logical functions inherit the compound-only restriction; `:has()` retains its
/// relative-selector grammar. Pseudo-elements and recursively nested `:has()` fail.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCompoundSelectorArgument {
    compound: Box<CssCompoundSelector>,
}

impl CssCompoundSelectorArgument {
    #[must_use]
    pub fn try_new(selector: CssSelector) -> Option<Self> {
        if !selector_is_valid_argument(&selector, true, true) {
            return None;
        }
        Some(Self {
            compound: Box::new(selector.into_compound_selector()),
        })
    }

    /// Borrows the complete argument, retaining namespaces and symbolic anchors.
    #[must_use]
    pub fn compound(&self) -> &CssCompoundSelector {
        &self.compound
    }
}

/// A case-preserving decoded `::part()` identifier with token provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPartName {
    component: crate::CssComponentValue,
}

impl CssPartName {
    /// Constructs a decoded identifier. CSS-wide words and duplicates are allowed.
    pub fn try_new(value: impl Into<String>) -> Result<Self, crate::CssComponentValueError> {
        Self::from_component(crate::CssComponentValue::try_ident(value)?)
    }

    pub(crate) fn from_component(
        component: crate::CssComponentValue,
    ) -> Result<Self, crate::CssComponentValueError> {
        if matches!(
            component.view(),
            crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Ident(_))
        ) {
            Ok(Self { component })
        } else {
            Err(crate::CssComponentValueError::new(
                crate::CssComponentValueErrorKind::InvalidIdentifier,
                component.origin().clone(),
            ))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        match self.component.view() {
            crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Ident(value)) => value,
            _ => unreachable!("checked part identifier"),
        }
    }

    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }

    #[must_use]
    pub fn to_css_string(&self) -> String {
        cssparser::ToCss::to_css_string(&cssparser::Token::Ident(self.as_str().into()))
    }
}

/// A nonempty ordered list of `::part()` identifiers, without name resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPartNameList {
    names: Vec<CssPartName>,
}

impl CssPartNameList {
    #[must_use]
    pub fn try_new(names: Vec<CssPartName>) -> Option<Self> {
        (!names.is_empty()).then_some(Self { names })
    }
    #[must_use]
    pub fn names(&self) -> &[CssPartName] {
        &self.names
    }
    /// Serializes canonical identifiers separated by spaces, without `::part()`.
    #[must_use]
    pub fn to_css_string(&self) -> String {
        self.names
            .iter()
            .map(CssPartName::to_css_string)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPseudoElement {
    Before,
    After,
    FirstLine,
    FirstLetter,
    Marker,
    Selection,
    Backdrop,
    Slotted(CssCompoundSelectorArgument),
    Part(CssPartNameList),
}

impl CssPseudoElement {
    pub(crate) fn is_element_backed(&self) -> bool {
        matches!(self, Self::Part(_))
    }

    fn permits_child(&self, child: &Self) -> bool {
        match self {
            Self::Part(_) => true,
            Self::Slotted(_) => matches!(
                child,
                Self::Before | Self::After | Self::Marker | Self::Part(_) | Self::Backdrop
            ),
            Self::Before | Self::After => matches!(child, Self::Marker),
            _ => false,
        }
    }
}

/// One ordered segment following the originating element's compound selector.
/// Each pseudo-class applies to the most recent pseudo-element segment.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPseudoElementSegment {
    PseudoElement(CssPseudoElement),
    PseudoClass(CssPseudoClass),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssPseudoElementSequence {
    segments: Vec<CssPseudoElementSegment>,
}

impl CssPseudoElementSequence {
    /// Constructs an element-only sequence, subject to the same transition rules.
    #[must_use]
    pub fn try_new(pseudo_elements: Vec<CssPseudoElement>) -> Option<Self> {
        Self::try_from_segments(
            pseudo_elements
                .into_iter()
                .map(CssPseudoElementSegment::PseudoElement)
                .collect(),
        )
    }

    /// Checks a nonempty sequence beginning with a pseudo-element. Logical suffix
    /// arguments inherit the restrictions of the pseudo-element they qualify.
    #[must_use]
    pub fn try_from_segments(segments: Vec<CssPseudoElementSegment>) -> Option<Self> {
        let mut current: Option<&CssPseudoElement> = None;
        for segment in &segments {
            match segment {
                CssPseudoElementSegment::PseudoElement(element) => {
                    if current.is_some_and(|previous| !previous.permits_child(element)) {
                        return None;
                    }
                    current = Some(element);
                }
                CssPseudoElementSegment::PseudoClass(pseudo) => {
                    let element = current?;
                    if !pseudo_is_valid_suffix(pseudo, element.is_element_backed(), true) {
                        return None;
                    }
                }
            }
        }
        current?;
        Some(Self { segments })
    }

    /// Returns every segment in authored attachment order, without projections.
    #[must_use]
    pub fn segments(&self) -> &[CssPseudoElementSegment] {
        &self.segments
    }
}

/// Intrinsic authored grammar inherited by selector-function arguments. Matching
/// and namespace resolution are separate from these structural restrictions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CssSelectorGrammarContext {
    allow_pseudo_elements: bool,
    allow_has: bool,
    compound_only: bool,
    pseudo_suffix: Option<bool>,
}

impl CssSelectorGrammarContext {
    pub(crate) const ORDINARY: Self = Self {
        allow_pseudo_elements: true,
        allow_has: true,
        compound_only: false,
        pseudo_suffix: None,
    };

    pub(crate) const fn from_parser_restrictions(
        allow_pseudo_elements: bool,
        allow_has: bool,
        compound_only: bool,
        pseudo_suffix: Option<bool>,
    ) -> Self {
        Self {
            allow_pseudo_elements,
            allow_has,
            compound_only,
            pseudo_suffix,
        }
    }

    fn at_least_as_restrictive_as(self, original: Self) -> bool {
        (!self.allow_pseudo_elements || original.allow_pseudo_elements)
            && (!self.allow_has || original.allow_has)
            && (!original.compound_only || self.compound_only)
            && match original.pseudo_suffix {
                None => true,
                Some(true) => self.pseudo_suffix.is_some(),
                Some(false) => self.pseudo_suffix == Some(false),
            }
    }

    pub(crate) fn logical_arguments(self) -> Self {
        Self {
            allow_pseudo_elements: false,
            ..self
        }
    }

    pub(crate) fn independent_arguments(self) -> Self {
        Self {
            allow_pseudo_elements: false,
            compound_only: false,
            pseudo_suffix: None,
            ..self
        }
    }

    pub(crate) fn compound_arguments(self) -> Self {
        Self {
            compound_only: true,
            ..self.independent_arguments()
        }
    }

    pub(crate) fn relative_arguments(self) -> Self {
        Self {
            allow_has: false,
            ..self.independent_arguments()
        }
    }

    pub(crate) fn suffix(self, element_backed: bool) -> Self {
        Self {
            pseudo_suffix: Some(element_backed),
            ..self.compound_arguments()
        }
    }

    pub(crate) fn admits_selector(self, selector: &CssSelector) -> bool {
        match selector {
            CssSelector::Tag(value) | CssSelector::Key(value) | CssSelector::Class(value) => {
                self.pseudo_suffix.is_none() && !value.is_empty() && !value.contains('\0')
            }
            CssSelector::PseudoClass(_) => true,
            CssSelector::Compound(compound) => self.admits_compound(compound),
            CssSelector::Complex(_) => !self.compound_only && self.pseudo_suffix.is_none(),
        }
    }

    pub(crate) fn admits_compound(self, compound: &CssCompoundSelector) -> bool {
        (self.allow_pseudo_elements || !compound.has_pseudo_elements())
            && (self.pseudo_suffix.is_none()
                || (compound.type_selector().is_none()
                    && compound.ids().is_empty()
                    && compound.classes().is_empty()
                    && compound.attributes().is_empty()
                    && !compound.has_scope_anchor()
                    && compound.nesting_selectors() == 0
                    && !compound.has_pseudo_elements()
                    && !compound.pseudo_classes().is_empty()))
    }

    pub(crate) fn admits_pseudo(self, pseudo: &CssPseudoClass) -> bool {
        let logical = matches!(
            pseudo,
            CssPseudoClass::Not(_) | CssPseudoClass::Is(_) | CssPseudoClass::Where(_)
        );
        let suffix = self.pseudo_suffix != Some(false)
            || logical
            || matches!(
                pseudo,
                CssPseudoClass::Hover
                    | CssPseudoClass::Active
                    | CssPseudoClass::Focus
                    | CssPseudoClass::FocusVisible
                    | CssPseudoClass::FocusWithin
            );
        suffix
            && match pseudo {
                CssPseudoClass::Not(list) => !list.items().is_empty(),
                CssPseudoClass::Has(_) => self.allow_has,
                CssPseudoClass::NthChild(pattern) | CssPseudoClass::NthLastChild(pattern) => {
                    pattern
                        .selector_list()
                        .is_none_or(|list| !list.items().is_empty())
                }
                _ => true,
            }
    }
}

pub(crate) fn selector_is_valid_pseudo_suffix(
    selector: &CssSelector,
    element_backed: bool,
    allow_has: bool,
) -> bool {
    selector_is_valid_in_context(
        selector,
        CssSelectorGrammarContext {
            allow_has,
            ..CssSelectorGrammarContext::ORDINARY.suffix(element_backed)
        },
    )
}

fn pseudo_is_valid_suffix(pseudo: &CssPseudoClass, element_backed: bool, allow_has: bool) -> bool {
    pseudo_is_valid_in_context(
        pseudo,
        CssSelectorGrammarContext {
            allow_has,
            ..CssSelectorGrammarContext::ORDINARY.suffix(element_backed)
        },
    )
}

fn selector_is_valid_argument(
    selector: &CssSelector,
    allow_has: bool,
    compound_only: bool,
) -> bool {
    selector_is_valid_in_context(
        selector,
        CssSelectorGrammarContext {
            allow_has,
            compound_only,
            ..CssSelectorGrammarContext::ORDINARY.logical_arguments()
        },
    )
}

fn selector_is_valid_in_context(
    selector: &CssSelector,
    context: CssSelectorGrammarContext,
) -> bool {
    context.admits_selector(selector)
        && match selector {
            CssSelector::Tag(_) | CssSelector::Key(_) | CssSelector::Class(_) => true,
            CssSelector::PseudoClass(pseudo) => pseudo_is_valid_in_context(pseudo, context),
            CssSelector::Compound(compound) => compound_is_valid_in_context(compound, context),
            CssSelector::Complex(complex) => {
                compound_is_valid_in_context(complex.first(), context)
                    && complex
                        .rest()
                        .iter()
                        .all(|part| compound_is_valid_in_context(part.selector(), context))
            }
        }
}

fn compound_is_valid_in_context(
    compound: &CssCompoundSelector,
    context: CssSelectorGrammarContext,
) -> bool {
    context.admits_compound(compound)
        && compound
            .pseudo_classes()
            .iter()
            .all(|pseudo| pseudo_is_valid_in_context(pseudo, context))
}

fn pseudo_is_valid_in_context(pseudo: &CssPseudoClass, context: CssSelectorGrammarContext) -> bool {
    context.admits_pseudo(pseudo)
        && match pseudo {
            CssPseudoClass::HostFunction(argument) | CssPseudoClass::HostContext(argument) => {
                compound_is_valid_in_context(argument.compound(), context.compound_arguments())
            }
            CssPseudoClass::Not(list) | CssPseudoClass::Is(list) | CssPseudoClass::Where(list) => {
                list.items().iter().all(|item| match item {
                    CssPseudoSelectorListItem::Selector(selector) => {
                        selector_is_valid_in_context(selector, context.logical_arguments())
                    }
                    CssPseudoSelectorListItem::InvalidNesting(item) => {
                        !matches!(pseudo, CssPseudoClass::Not(_))
                            && item.admitted_in(context.logical_arguments())
                    }
                })
            }
            CssPseudoClass::Has(list) => list.selectors().iter().all(|relative| {
                selector_is_valid_in_context(relative.selector(), context.relative_arguments())
            }),
            CssPseudoClass::NthChild(pattern) | CssPseudoClass::NthLastChild(pattern) => {
                pattern.selector_list().is_none_or(|list| {
                    list.items().iter().all(|item| match item {
                        CssPseudoSelectorListItem::Selector(selector) => {
                            selector_is_valid_in_context(selector, context.independent_arguments())
                        }
                        CssPseudoSelectorListItem::InvalidNesting(_) => false,
                    })
                })
            }
            _ => true,
        }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssNthChildPattern {
    pattern: CssNthPattern,
    selector_list: Option<CssPseudoSelectorList>,
}

impl CssNthChildPattern {
    #[must_use]
    pub const fn new(pattern: CssNthPattern, selector_list: Option<CssPseudoSelectorList>) -> Self {
        Self {
            pattern,
            selector_list,
        }
    }

    #[must_use]
    pub const fn pattern(&self) -> CssNthPattern {
        self.pattern
    }

    #[must_use]
    pub const fn selector_list(&self) -> Option<&CssPseudoSelectorList> {
        self.selector_list.as_ref()
    }

    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.selector_list
            .as_ref()
            .is_some_and(CssPseudoSelectorList::has_pseudo_elements)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNthPattern {
    Odd,
    Even,
    Integer(i32),
    AnPlusB(CssNthAnPlusB),
}

/// The namespace selected by one authored selector name.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssNamespaceConstraint {
    /// The active default namespace declared for the stylesheet.
    Default,
    /// No namespace, authored with a leading `|` or implied for an attribute.
    ExplicitNone,
    /// Any namespace, authored with `*|` or implied when no default is active.
    Any,
    /// The active binding for one exact, case-sensitive decoded prefix.
    Named(CssNamespacePrefix),
}

/// The authored prefix of a qualified name, independent of its effective namespace.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssQualifiedNamePrefix {
    /// No prefix or namespace separator was authored.
    Unqualified,
    /// A bare `|` was authored.
    ExplicitNone,
    /// The wildcard prefix `*|` was authored.
    Any,
    /// One exact, case-sensitive decoded prefix was authored.
    Named(CssNamespacePrefix),
}

/// A checked qualified name cannot refer to an undeclared named prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssQualifiedNameError {
    UndeclaredNamespacePrefix(CssNamespacePrefix),
}

impl std::fmt::Display for CssQualifiedNameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndeclaredNamespacePrefix(prefix) => write!(
                formatter,
                "undeclared namespace prefix `{}`",
                prefix.as_str()
            ),
        }
    }
}

impl std::error::Error for CssQualifiedNameError {}

impl CssQualifiedNamePrefix {
    fn admits_constraint(&self, namespace: &CssNamespaceConstraint, apply_default: bool) -> bool {
        match (self, namespace) {
            (Self::Unqualified, CssNamespaceConstraint::Default | CssNamespaceConstraint::Any) => {
                apply_default
            }
            (Self::Unqualified, CssNamespaceConstraint::ExplicitNone) => !apply_default,
            (Self::ExplicitNone, CssNamespaceConstraint::ExplicitNone)
            | (Self::Any, CssNamespaceConstraint::Any) => true,
            (Self::Named(authored), CssNamespaceConstraint::Named(active)) => authored == active,
            _ => false,
        }
    }

    fn namespace(
        &self,
        context: &crate::CssNamespaceContext,
        apply_default: bool,
    ) -> Result<CssNamespaceConstraint, CssQualifiedNameError> {
        Ok(match self {
            Self::Unqualified if !apply_default => CssNamespaceConstraint::ExplicitNone,
            Self::Unqualified if context.default_namespace().is_some() => {
                CssNamespaceConstraint::Default
            }
            Self::Unqualified | Self::Any => CssNamespaceConstraint::Any,
            Self::ExplicitNone => CssNamespaceConstraint::ExplicitNone,
            Self::Named(prefix) => {
                if context.named_namespace(prefix).is_none() {
                    return Err(CssQualifiedNameError::UndeclaredNamespacePrefix(
                        prefix.clone(),
                    ));
                }
                CssNamespaceConstraint::Named(prefix.clone())
            }
        })
    }

    fn serialize_name(
        &self,
        local_name: Option<&str>,
        max_css_bytes: usize,
    ) -> Result<crate::CssSerializedValue, crate::CssComponentValueError> {
        use crate::component_values::{CssCanonicalBuilder, CssCanonicalToken};
        let mut writer = CssCanonicalBuilder::new(max_css_bytes);
        let origin = CssValueOrigin::Programmatic;
        match self {
            Self::Unqualified => {}
            Self::ExplicitNone => writer.push_grammar(CssCanonicalToken::Delim('|'), &origin)?,
            Self::Any => {
                writer.push_grammar(CssCanonicalToken::Delim('*'), &origin)?;
                writer.push_grammar(CssCanonicalToken::Delim('|'), &origin)?;
            }
            Self::Named(prefix) => {
                writer.push_grammar(CssCanonicalToken::Ident(prefix.as_str()), &origin)?;
                writer.push_grammar(CssCanonicalToken::Delim('|'), &origin)?;
            }
        }
        writer.push_grammar(
            match local_name {
                Some(name) => CssCanonicalToken::Ident(name),
                None => CssCanonicalToken::Delim('*'),
            },
            &origin,
        )?;
        writer.finish()
    }
}

/// One authored namespace-qualified type or universal selector name.
///
/// Local identifiers are decoded values; wildcard identity and authored prefix
/// remain distinct from the namespace constraint selected by the parse context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssQualifiedSelectorName {
    namespace: CssNamespaceConstraint,
    prefix: CssQualifiedNamePrefix,
    local_name: Option<super::CssIdent>,
}

impl CssQualifiedSelectorName {
    /// Checks a decoded type name against explicit namespace bindings.
    pub fn try_new(
        prefix: CssQualifiedNamePrefix,
        local_name: super::CssIdent,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, CssQualifiedNameError> {
        let namespace = prefix.namespace(context, true)?;
        Ok(Self::new(namespace, prefix, local_name))
    }

    /// Checks a universal selector against explicit namespace bindings.
    pub fn try_universal(
        prefix: CssQualifiedNamePrefix,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, CssQualifiedNameError> {
        let namespace = prefix.namespace(context, true)?;
        Ok(Self::universal(namespace, prefix))
    }

    pub(crate) fn new(
        namespace: CssNamespaceConstraint,
        prefix: CssQualifiedNamePrefix,
        local_name: super::CssIdent,
    ) -> Self {
        debug_assert!(prefix.admits_constraint(&namespace, true));
        Self {
            namespace,
            prefix,
            local_name: Some(local_name),
        }
    }

    pub(crate) fn universal(
        namespace: CssNamespaceConstraint,
        prefix: CssQualifiedNamePrefix,
    ) -> Self {
        debug_assert!(prefix.admits_constraint(&namespace, true));
        Self {
            namespace,
            prefix,
            local_name: None,
        }
    }

    /// Returns the namespace constraint active for this selector name.
    #[must_use]
    pub const fn namespace(&self) -> &CssNamespaceConstraint {
        &self.namespace
    }

    /// Returns the authored prefix, including the absence of a prefix.
    #[must_use]
    pub const fn prefix(&self) -> &CssQualifiedNamePrefix {
        &self.prefix
    }

    /// Returns the decoded local identifier, or `None` for universal `*`.
    #[must_use]
    pub fn local_name(&self) -> Option<&str> {
        self.local_name.as_ref().map(super::CssIdent::as_str)
    }

    /// Reports whether this name is the universal selector.
    #[must_use]
    pub const fn is_universal(&self) -> bool {
        self.local_name.is_none()
    }

    /// Serializes this name with a bound on generated UTF-8 bytes.
    ///
    /// Escaping preserves decoded identity and authored prefix form. Generated
    /// tokens have programmatic origins; enclosing parsed rule origins remain
    /// available separately. This does not serialize a complete selector or rule.
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<crate::CssSerializedValue, crate::CssComponentValueError> {
        self.prefix.serialize_name(self.local_name(), max_css_bytes)
    }
}

/// One authored qualified attribute name; attribute local names cannot be universal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssQualifiedAttributeName {
    namespace: CssNamespaceConstraint,
    prefix: CssQualifiedNamePrefix,
    local_name: CssAttributeName,
}

impl CssQualifiedAttributeName {
    /// Checks a decoded attribute name against explicit namespace bindings.
    /// The default namespace never applies to an unqualified attribute.
    pub fn try_new(
        prefix: CssQualifiedNamePrefix,
        local_name: CssAttributeName,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, CssQualifiedNameError> {
        let namespace = prefix.namespace(context, false)?;
        Ok(Self::new(namespace, prefix, local_name))
    }

    pub(crate) fn new(
        namespace: CssNamespaceConstraint,
        prefix: CssQualifiedNamePrefix,
        local_name: CssAttributeName,
    ) -> Self {
        debug_assert!(prefix.admits_constraint(&namespace, false));
        Self {
            namespace,
            prefix,
            local_name,
        }
    }

    #[must_use]
    pub const fn namespace(&self) -> &CssNamespaceConstraint {
        &self.namespace
    }

    #[must_use]
    pub const fn prefix(&self) -> &CssQualifiedNamePrefix {
        &self.prefix
    }

    #[must_use]
    pub const fn local_name(&self) -> &CssAttributeName {
        &self.local_name
    }

    /// Serializes this qualified name with bounded bytes and programmatic origins.
    /// This name grammar is for attribute selectors, not `attr()` host admission.
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<crate::CssSerializedValue, crate::CssComponentValueError> {
        self.prefix
            .serialize_name(Some(self.local_name.as_str()), max_css_bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssNthAnPlusB {
    a: i32,
    b: i32,
}

impl CssNthAnPlusB {
    #[must_use]
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    #[must_use]
    pub const fn a(self) -> i32 {
        self.a
    }

    #[must_use]
    pub const fn b(self) -> i32 {
        self.b
    }
}

/// One authored simple member, before checked compound assembly.
/// Type includes the existing qualified universal-selector identity. Pseudo-elements
/// and relative or complex selectors belong to separate construction boundaries.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssSimpleSelector {
    Type(CssQualifiedSelectorName),
    Id(super::CssIdent),
    Class(super::CssIdent),
    Attribute(CssAttributeSelector),
    PseudoClass(CssPseudoClass),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssCompoundSelector {
    scope_anchors: usize,
    nesting_selectors: usize,
    type_selector: Option<Box<CssQualifiedSelectorName>>,
    ids: Vec<String>,
    classes: Vec<String>,
    attributes: Vec<CssAttributeSelector>,
    pseudo_classes: Vec<CssPseudoClass>,
    pseudo_elements: Option<CssPseudoElementSequence>,
}

impl CssCompoundSelector {
    /// Assembles nonempty simple members, admitting a unique type/universal first.
    /// Repeated IDs and classes remain ordered within their canonical groups.
    ///
    /// ```
    /// use surgeist_css::{CssComplexSelector, CssComplexSelectorPart, CssCompoundSelector,
    ///     CssIdent, CssSelector, CssSelectorCombinator, CssSimpleSelector};
    /// let first = CssCompoundSelector::try_new(vec![
    ///     CssSimpleSelector::Class(CssIdent::try_new("Parent").unwrap()),
    /// ]).unwrap();
    /// let child = CssCompoundSelector::try_new(vec![
    ///     CssSimpleSelector::Class(CssIdent::try_new("Child").unwrap()),
    /// ]).unwrap();
    /// let part = CssComplexSelectorPart::try_new(CssSelectorCombinator::Child, child).unwrap();
    /// let selector = CssSelector::Complex(CssComplexSelector::try_new(first, vec![part]).unwrap());
    /// assert_eq!(selector.to_specified_css().unwrap(), ".Parent > .Child");
    /// ```
    pub fn try_new(members: Vec<CssSimpleSelector>) -> Result<Self, CssSelectorConstructionError> {
        Self::try_new_with_limits(
            members,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }

    /// Checks member order, then admits the complete ordinary graph and output
    /// with one cumulative budget. No partially assembled compound escapes.
    pub fn try_new_with_limits(
        members: Vec<CssSimpleSelector>,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        if members.is_empty() {
            return Err(CssSelectorConstructionError::new(
                CssSelectorConstructionErrorKind::EmptyCompound,
            ));
        }
        let mut type_selector = None;
        let mut ids = Vec::new();
        let mut classes = Vec::new();
        let mut attributes = Vec::new();
        let mut pseudo_classes = Vec::new();
        for (member_index, member) in members.into_iter().enumerate() {
            match member {
                CssSimpleSelector::Type(name) => {
                    if member_index != 0 {
                        return Err(CssSelectorConstructionError::new(
                            CssSelectorConstructionErrorKind::InvalidTypePosition { member_index },
                        ));
                    }
                    type_selector = Some(name);
                }
                CssSimpleSelector::Id(value) => ids.push(value.as_str().to_owned()),
                CssSimpleSelector::Class(value) => classes.push(value.as_str().to_owned()),
                CssSimpleSelector::Attribute(value) => attributes.push(value),
                CssSimpleSelector::PseudoClass(value) => pseudo_classes.push(value),
            }
        }
        let value = Self::new_with_qualified_type_and_pseudo_elements(
            0,
            type_selector,
            ids,
            classes,
            attributes,
            pseudo_classes,
            None,
        );
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer
            .compound_selector(&value)
            .map_err(CssSelectorConstructionError::specified)?;
        Ok(value)
    }

    #[must_use]
    pub(crate) fn new(
        tag: Option<String>,
        key: Option<String>,
        classes: Vec<String>,
        attributes: Vec<CssAttributeSelector>,
        pseudo_classes: Vec<CssPseudoClass>,
    ) -> Self {
        Self::new_with_scope_anchor(false, tag, key, classes, attributes, pseudo_classes)
    }

    #[must_use]
    pub(crate) fn new_with_scope_anchor(
        scope_anchor: bool,
        tag: Option<String>,
        key: Option<String>,
        classes: Vec<String>,
        attributes: Vec<CssAttributeSelector>,
        pseudo_classes: Vec<CssPseudoClass>,
    ) -> Self {
        Self::new_with_scope_anchor_and_pseudo_elements(
            scope_anchor,
            tag,
            key,
            classes,
            attributes,
            pseudo_classes,
            None,
        )
    }

    #[must_use]
    pub(crate) fn new_with_scope_anchor_and_pseudo_elements(
        scope_anchor: bool,
        tag: Option<String>,
        key: Option<String>,
        classes: Vec<String>,
        attributes: Vec<CssAttributeSelector>,
        pseudo_classes: Vec<CssPseudoClass>,
        pseudo_elements: Option<CssPseudoElementSequence>,
    ) -> Self {
        let type_selector = tag.map(|tag| {
            CssQualifiedSelectorName::new(
                CssNamespaceConstraint::Any,
                CssQualifiedNamePrefix::Unqualified,
                super::CssIdent::new(tag),
            )
        });
        let ids = key.into_iter().collect();
        Self::new_with_qualified_type_and_pseudo_elements(
            usize::from(scope_anchor),
            type_selector,
            ids,
            classes,
            attributes,
            pseudo_classes,
            pseudo_elements,
        )
    }

    #[must_use]
    pub(crate) fn new_with_qualified_type_and_pseudo_elements(
        scope_anchors: usize,
        type_selector: Option<CssQualifiedSelectorName>,
        ids: Vec<String>,
        classes: Vec<String>,
        attributes: Vec<CssAttributeSelector>,
        pseudo_classes: Vec<CssPseudoClass>,
        pseudo_elements: Option<CssPseudoElementSequence>,
    ) -> Self {
        Self {
            scope_anchors,
            nesting_selectors: 0,
            type_selector: type_selector.map(Box::new),
            ids,
            classes,
            attributes,
            pseudo_classes,
            pseudo_elements,
        }
    }

    pub(crate) fn with_nesting_selectors(mut self, count: usize) -> Self {
        self.nesting_selectors = count;
        self
    }

    /// Returns the number of symbolic nesting selectors in this compound.
    ///
    /// With an enclosing parent selector list, every anchor refers to that complete
    /// list and uses its maximum specificity. Without a parent list, each anchor
    /// matches the context's scope elements and contributes zero specificity.
    /// This count preserves authored anchors; it does not resolve their context or
    /// replace them with a `:scope` pseudo-class.
    #[must_use]
    pub const fn nesting_selectors(&self) -> usize {
        self.nesting_selectors
    }

    #[must_use]
    pub const fn has_scope_anchor(&self) -> bool {
        self.scope_anchors != 0
    }

    /// Returns the number of symbolic scope anchors in this compound.
    ///
    /// These anchors belong to the selector's scope context, independently of
    /// ordinary nesting selectors and explicit `:scope` pseudo-classes.
    /// The count preserves authored multiplicity without evaluating specificity.
    #[must_use]
    pub const fn scope_anchors(&self) -> usize {
        self.scope_anchors
    }

    /// Returns the authored namespace-aware type or universal selector.
    #[must_use]
    pub fn type_selector(&self) -> Option<&CssQualifiedSelectorName> {
        self.type_selector.as_deref()
    }

    /// Whether the authored unqualified form has Any namespace; a simple tag also needs a local name.
    #[must_use]
    pub(crate) fn has_unqualified_any_namespace(&self) -> bool {
        self.type_selector.as_deref().is_some_and(|name| {
            matches!(name.prefix(), CssQualifiedNamePrefix::Unqualified)
                && matches!(name.namespace(), CssNamespaceConstraint::Any)
        })
    }

    /// Returns the parser-retained IDs in authored order.
    #[must_use]
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    #[must_use]
    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    #[must_use]
    pub fn attributes(&self) -> &[CssAttributeSelector] {
        &self.attributes
    }

    #[must_use]
    pub fn pseudo_classes(&self) -> &[CssPseudoClass] {
        &self.pseudo_classes
    }

    #[must_use]
    pub const fn pseudo_elements(&self) -> Option<&CssPseudoElementSequence> {
        self.pseudo_elements.as_ref()
    }

    #[must_use]
    pub const fn has_pseudo_elements(&self) -> bool {
        self.pseudo_elements.is_some()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAttributeSelector {
    name: CssQualifiedAttributeName,
    matcher: CssAttributeMatcher,
    case_sensitivity: CssAttributeCaseSensitivity,
}

impl CssAttributeSelector {
    /// Checks the coupled qualified name, decoded operand and optional modifier.
    /// Empty or whitespace-containing values retain their syntax identity; matching
    /// behavior belongs to consumers. Exists admits only DocumentDefault.
    pub fn try_new(
        name: CssQualifiedAttributeName,
        matcher: CssAttributeMatcher,
        case_sensitivity: CssAttributeCaseSensitivity,
    ) -> Result<Self, CssSelectorConstructionError> {
        Self::try_new_with_limits(
            name,
            matcher,
            case_sensitivity,
            crate::CssSpecifiedValueSerializationLimits::default(),
        )
    }

    /// Checks intrinsic operand identity and charges canonical attribute output
    /// within the supplied semantic-node and UTF-8 byte limits.
    pub fn try_new_with_limits(
        name: CssQualifiedAttributeName,
        matcher: CssAttributeMatcher,
        case_sensitivity: CssAttributeCaseSensitivity,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        match &matcher {
            CssAttributeMatcher::Exists
                if case_sensitivity != CssAttributeCaseSensitivity::DocumentDefault =>
            {
                return Err(CssSelectorConstructionError::new(
                    CssSelectorConstructionErrorKind::ModifierWithoutValue,
                ));
            }
            CssAttributeMatcher::Exists => {}
            CssAttributeMatcher::Equals(value)
            | CssAttributeMatcher::Includes(value)
            | CssAttributeMatcher::DashMatch(value)
            | CssAttributeMatcher::Prefix(value)
            | CssAttributeMatcher::Suffix(value)
            | CssAttributeMatcher::Substring(value) => {
                crate::CssComponentValue::try_string(value.clone()).map_err(|error| {
                    CssSelectorConstructionError::new(
                        CssSelectorConstructionErrorKind::InvalidAttributeValue(error),
                    )
                })?;
            }
        }
        let value = Self::new_qualified(name, matcher, case_sensitivity);
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer
            .attribute_selector(&value)
            .map_err(CssSelectorConstructionError::specified)?;
        Ok(value)
    }

    #[must_use]
    pub(crate) const fn new_qualified(
        name: CssQualifiedAttributeName,
        matcher: CssAttributeMatcher,
        case_sensitivity: CssAttributeCaseSensitivity,
    ) -> Self {
        Self {
            name,
            matcher,
            case_sensitivity,
        }
    }

    /// Returns the namespace constraint for the attribute name.
    #[must_use]
    pub const fn namespace(&self) -> &CssNamespaceConstraint {
        self.name.namespace()
    }

    /// Returns the qualified authored name, including its prefix form.
    #[must_use]
    pub const fn qualified_name(&self) -> &CssQualifiedAttributeName {
        &self.name
    }

    #[must_use]
    pub const fn name(&self) -> &CssAttributeName {
        self.name.local_name()
    }

    #[must_use]
    pub const fn matcher(&self) -> &CssAttributeMatcher {
        &self.matcher
    }

    #[must_use]
    pub const fn case_sensitivity(&self) -> CssAttributeCaseSensitivity {
        self.case_sensitivity
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssAttributeName {
    name: String,
}

impl CssAttributeName {
    /// Constructs a decoded attribute identifier whose CSS spelling may require escapes.
    /// Empty values and NUL cannot preserve identifier identity.
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        crate::CssComponentValue::try_ident(name.clone()).ok()?;
        Some(Self { name })
    }

    #[must_use]
    pub(crate) fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(Self::try_new(name.clone()).is_some());
        Self { name }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAttributeMatcher {
    Exists,
    Equals(String),
    Includes(String),
    DashMatch(String),
    Prefix(String),
    Suffix(String),
    Substring(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAttributeCaseSensitivity {
    DocumentDefault,
    AsciiCaseInsensitive,
    ExplicitSensitive,
}
