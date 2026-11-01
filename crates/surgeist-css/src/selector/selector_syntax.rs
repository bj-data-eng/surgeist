use crate::component::CssValueOrigin;
use crate::rule::CssNamespacePrefix;
use crate::source::{CssRetainedSources, CssSourceRetention, CssSourceRetentionError};
use crate::values::common::CssCustomIdent;
use crate::values::text::CssDirectionality;
use crate::values::view_transition::CssViewTransitionNameSelector;

impl CssRetainedSources for CssSelector {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Tag(_) | Self::Key(_) | Self::Class(_) => Ok(()),
            Self::PseudoClass(value) => sources.visit(value),
            Self::Compound(value) => sources.visit(value),
            Self::Complex(value) => {
                sources.visit(&value.first)?;
                for part in &value.rest {
                    sources.visit(&part.selector)?;
                }
                Ok(())
            }
        }
    }
}
impl CssRetainedSources for CssCompoundSelector {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.pseudo_classes.as_slice())?;
        if let Some(value) = &self.pseudo_elements {
            for segment in &value.segments {
                sources.charge_step()?;
                match segment {
                    CssPseudoElementSegment::PseudoClass(value) => sources.visit(value)?,
                    CssPseudoElementSegment::PseudoElement(value) => match value {
                        CssPseudoElement::Slotted(value) => {
                            sources.visit(value.compound.as_ref())?
                        }
                        CssPseudoElement::Part(value) => {
                            for name in &value.names {
                                sources.visit(&name.component)?;
                            }
                        }
                        CssPseudoElement::Before
                        | CssPseudoElement::After
                        | CssPseudoElement::FirstLine
                        | CssPseudoElement::FirstLetter
                        | CssPseudoElement::Prefix
                        | CssPseudoElement::Suffix
                        | CssPseudoElement::Marker
                        | CssPseudoElement::Selection
                        | CssPseudoElement::SearchText
                        | CssPseudoElement::TargetText
                        | CssPseudoElement::SpellingError
                        | CssPseudoElement::GrammarError
                        | CssPseudoElement::Highlight(_)
                        | CssPseudoElement::Placeholder
                        | CssPseudoElement::Backdrop
                        | CssPseudoElement::FileSelectorButton
                        | CssPseudoElement::DetailsContent
                        | CssPseudoElement::ViewTransition
                        | CssPseudoElement::ViewTransitionGroup(_)
                        | CssPseudoElement::ViewTransitionImagePair(_)
                        | CssPseudoElement::ViewTransitionOld(_)
                        | CssPseudoElement::ViewTransitionNew(_)
                        | CssPseudoElement::UnknownWebkit(_) => {}
                    },
                }
            }
        }
        Ok(())
    }
}
impl CssRetainedSources for CssPseudoSelectorList {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        for item in &self.items {
            match item {
                CssPseudoSelectorListItem::Selector(value) => sources.visit(value)?,
                CssPseudoSelectorListItem::InvalidNesting(value) => {
                    sources.visit(&value.origin)?;
                    sources.visit(&value.components)?;
                }
            }
        }
        Ok(())
    }
}
impl CssRetainedSources for CssPseudoClass {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::HostFunction(value) | Self::HostContext(value) => {
                sources.visit(value.compound.as_ref())
            }
            Self::Dir(value) => sources.visit(&value.component),
            Self::Lang(value) => {
                for range in &value.ranges {
                    sources.visit(&range.component)?;
                }
                Ok(())
            }
            Self::Not(value) | Self::Is(value) | Self::Where(value) => sources.visit(value),
            Self::Has(value) => {
                for selector in &value.selectors {
                    sources.visit(selector.selector())?;
                }
                Ok(())
            }
            Self::NthChild(value) | Self::NthLastChild(value) => {
                sources.visit(&value.selector_list)
            }
            Self::Host
            | Self::Root
            | Self::Scope
            | Self::Defined
            | Self::AnyLink
            | Self::Link
            | Self::Visited
            | Self::Target
            | Self::Current
            | Self::Hover
            | Self::Active
            | Self::Focus
            | Self::FocusVisible
            | Self::FocusWithin
            | Self::Disabled
            | Self::Enabled
            | Self::Checked
            | Self::Unchecked
            | Self::Required
            | Self::Optional
            | Self::Valid
            | Self::Invalid
            | Self::UserValid
            | Self::UserInvalid
            | Self::PlaceholderShown
            | Self::Autofill
            | Self::FirstChild
            | Self::LastChild
            | Self::OnlyChild
            | Self::Empty
            | Self::FirstOfType
            | Self::LastOfType
            | Self::OnlyOfType
            | Self::NthOfType(_)
            | Self::NthLastOfType(_)
            | Self::Playing
            | Self::Paused
            | Self::Seeking
            | Self::Buffering
            | Self::Stalled
            | Self::Muted
            | Self::VolumeLocked
            | Self::Open
            | Self::Modal
            | Self::Fullscreen
            | Self::PictureInPicture
            | Self::PopoverOpen
            | Self::Default
            | Self::Indeterminate
            | Self::ReadOnly
            | Self::ReadWrite
            | Self::InRange
            | Self::OutOfRange => Ok(()),
        }
    }
}

/// # Specified serialization
///
/// Emits an authored selector, without resolving its symbolic ancestry.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssSelector {
    /// A type selector, carrying the decoded tag name.
    Tag(String),
    /// An ID selector (`#key`), carrying the decoded identifier without the `#`.
    Key(String),
    /// A class selector (`.class`), carrying the decoded identifier without the `.`.
    Class(String),
    /// A lone pseudo-class selector.
    PseudoClass(CssPseudoClass),
    /// A compound selector.
    Compound(CssCompoundSelector),
    /// A complex selector: compound selectors joined by combinators.
    Complex(CssComplexSelector),
}

impl CssSelector {
    /// Returns whether the selector contains a pseudo-element, including inside the arguments of functional pseudo-classes.
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

/// A complex selector: a first compound selector followed by one or more combinator-and-compound parts.
///
/// A pseudo-element may appear only in the last compound selector.
#[derive(Clone, Debug, PartialEq)]
pub struct CssComplexSelector {
    first: CssCompoundSelector,
    rest: Vec<CssComplexSelectorPart>,
}

impl CssComplexSelector {
    /// Checks complete ordinary assembly with the default cumulative output limits.
    ///
    /// Checks placement and the complete graph within one shared budget.
    pub fn try_new(
        first: CssCompoundSelector,
        rest: Vec<CssComplexSelectorPart>,
        limits: crate::serialization::CssSerializationLimits,
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
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
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

    /// Returns the first compound selector.
    #[must_use]
    pub const fn first(&self) -> &CssCompoundSelector {
        &self.first
    }

    /// Returns the combinator-and-compound parts that follow the first compound selector; the slice is never empty.
    #[must_use]
    pub fn rest(&self) -> &[CssComplexSelectorPart] {
        &self.rest
    }

    /// Returns whether any compound selector contains a pseudo-element.
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

/// One part of a complex selector: a combinator and the compound selector that follows it.
#[derive(Clone, Debug, PartialEq)]
pub struct CssComplexSelectorPart {
    combinator: CssSelectorCombinator,
    selector: CssCompoundSelector,
}

impl CssComplexSelectorPart {
    /// Checks an ordinary compound and its relationship with default output limits.
    /// Admission here does not prove that a pseudo-element is terminal in a complex.
    ///
    /// Charges the compound and canonical relationship bytes cumulatively.
    pub fn try_new(
        combinator: CssSelectorCombinator,
        selector: CssCompoundSelector,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        let value = Self::new(combinator, selector);
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
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

    /// Returns the combinator that relates this part to the compound selector before it.
    #[must_use]
    pub const fn combinator(&self) -> CssSelectorCombinator {
        self.combinator
    }

    /// Returns the compound selector of this part.
    #[must_use]
    pub const fn selector(&self) -> &CssCompoundSelector {
        &self.selector
    }
}

/// A selector combinator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSelectorCombinator {
    /// The descendant combinator, written as whitespace.
    Descendant,
    /// The child combinator `>`.
    Child,
    /// The next-sibling combinator `+`.
    NextSibling,
    /// The subsequent-sibling combinator `~`.
    SubsequentSibling,
    /// The authored `||` relationship between a column and its cells.
    Column,
}

/// Why checked authored selector construction could not return a complete value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CssSelectorConstructionErrorKind {
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
    InvalidAttributeValue(crate::component::CssComponentValueError),
    /// The decoded name is not a genuine identifier beginning with `-webkit-`.
    InvalidUnknownWebkitPseudoElementName,
    /// Intrinsic grammar or cumulative specified-output admission failed.
    Specified(crate::serialization::CssSerializationError),
    /// A checked qualified name refers to an undeclared named prefix.
    UndeclaredNamespacePrefix(CssNamespacePrefix),
}

/// Atomic selector or qualified-name construction failure, with no invented
/// parsed-source coordinates.
///
/// The cause is read through the predicates and accessors; a rejected attribute value
/// or specified-output admission failure is also the error's source.
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

    fn specified(error: crate::serialization::CssSerializationError) -> Self {
        Self {
            kind: CssSelectorConstructionErrorKind::Specified(error),
        }
    }

    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether a compound has no simple member.
        is_empty_compound => CssSelectorConstructionErrorKind::EmptyCompound,
        /// Returns whether a strict selector list has no member.
        is_empty_list => CssSelectorConstructionErrorKind::EmptyList,
        /// Returns whether a complex selector has no relationship and following compound.
        is_empty_complex_rest => CssSelectorConstructionErrorKind::EmptyComplexRest,
        /// Returns whether an attribute modifier was supplied without an attribute value.
        is_modifier_without_value => CssSelectorConstructionErrorKind::ModifierWithoutValue,
        /// Returns whether an unknown WebKit pseudo-element name is not a genuine identifier
        /// beginning with `-webkit-`.
        is_invalid_unknown_webkit_pseudo_element_name =>
            CssSelectorConstructionErrorKind::InvalidUnknownWebkitPseudoElementName,
    }

    /// Returns the index of a pseudo-element-bearing compound that precedes the final
    /// compound.
    #[must_use]
    pub const fn non_terminal_pseudo_element(&self) -> Option<usize> {
        match self.kind {
            CssSelectorConstructionErrorKind::NonTerminalPseudoElement { compound_index } => {
                Some(compound_index)
            }
            _ => None,
        }
    }

    /// Returns the index of a type or universal selector that is duplicated or follows
    /// another member.
    #[must_use]
    pub const fn invalid_type_position(&self) -> Option<usize> {
        match self.kind {
            CssSelectorConstructionErrorKind::InvalidTypePosition { member_index } => {
                Some(member_index)
            }
            _ => None,
        }
    }

    /// Returns why a decoded attribute operand cannot preserve its string identity.
    #[must_use]
    pub const fn attribute_value_error(&self) -> Option<&crate::component::CssComponentValueError> {
        match &self.kind {
            CssSelectorConstructionErrorKind::InvalidAttributeValue(error) => Some(error),
            _ => None,
        }
    }

    /// Returns the intrinsic grammar or cumulative specified-output admission failure.
    #[must_use]
    pub const fn serialization_error(
        &self,
    ) -> Option<&crate::serialization::CssSerializationError> {
        match &self.kind {
            CssSelectorConstructionErrorKind::Specified(error) => Some(error),
            _ => None,
        }
    }

    /// Returns the undeclared named prefix a checked qualified name refers to.
    #[must_use]
    pub const fn undeclared_namespace_prefix(&self) -> Option<&CssNamespacePrefix> {
        match &self.kind {
            CssSelectorConstructionErrorKind::UndeclaredNamespacePrefix(prefix) => Some(prefix),
            _ => None,
        }
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
            CssSelectorConstructionErrorKind::InvalidUnknownWebkitPseudoElementName => formatter
                .write_str(
                    "an unknown WebKit pseudo-element requires a valid decoded -webkit- identifier",
                ),
            CssSelectorConstructionErrorKind::UndeclaredNamespacePrefix(prefix) => write!(
                formatter,
                "undeclared namespace prefix `{}`",
                prefix.as_str()
            ),
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
            | CssSelectorConstructionErrorKind::ModifierWithoutValue
            | CssSelectorConstructionErrorKind::InvalidUnknownWebkitPseudoElementName
            | CssSelectorConstructionErrorKind::UndeclaredNamespacePrefix(_) => None,
            CssSelectorConstructionErrorKind::InvalidAttributeValue(error) => Some(error),
            CssSelectorConstructionErrorKind::Specified(error) => Some(error),
        }
    }
}

/// A nonempty ordered list admitted under ordinary authored selector grammar.
///
/// # Specified serialization
///
/// Emits the complete ordinary list without resolving symbolic ancestry.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSelectorList {
    selectors: Vec<CssSelector>,
}

impl CssSelectorList {
    /// Checks every ordinary member with the default cumulative output limits.
    ///
    /// Checks the complete graph and canonical output within one shared budget.
    /// No invalid member is dropped and no partially admitted list escapes.
    pub fn try_new(
        selectors: Vec<CssSelector>,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        if selectors.is_empty() {
            return Err(CssSelectorConstructionError::empty_list());
        }
        let value = Self::new(selectors);
        value
            .serialize_specified(limits)
            .map_err(CssSelectorConstructionError::specified)?;
        Ok(value)
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    /// Returns the members in authored order; the slice is never empty.
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
    origin: crate::source::CssParsedOrigin,
    components: crate::component::CssComponentValues,
    grammar: CssSelectorGrammarContext,
}

impl CssInvalidNestingSelectorItem {
    pub(crate) const fn new(
        origin: crate::source::CssParsedOrigin,
        components: crate::component::CssComponentValues,
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

    /// Returns the original raw-source window of the member.
    #[must_use]
    pub const fn origin(&self) -> &crate::source::CssParsedOrigin {
        &self.origin
    }

    /// Returns the component values of the member, including edge trivia.
    #[must_use]
    pub const fn components(&self) -> &crate::component::CssComponentValues {
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
    ///
    /// Rejects an empty list or any invalid supplied member without filtering.
    pub fn try_new(
        selectors: Vec<CssSelector>,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        if selectors.is_empty() {
            return Err(CssSelectorConstructionError::empty_list());
        }
        Self::try_new_forgiving(selectors, limits)
    }

    /// Admits empty complex-real arguments for Is/Where. Invalid typed members
    /// fail construction; parsed member forgiveness is a separate operation.
    ///
    /// Checks every supplied member with one aggregate and cumulative budget,
    /// including the aggregate node of a valid empty forgiving list.
    pub fn try_new_forgiving(
        selectors: Vec<CssSelector>,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer
            .pseudo_selector_members(&selectors)
            .map_err(CssSelectorConstructionError::specified)?;
        let mut items = Vec::new();
        items.try_reserve(selectors.len()).map_err(|_| {
            CssSelectorConstructionError::specified(crate::serialization::CssSerializationError::new(
                crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ))
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

    /// Returns whether any selector item contains a pseudo-element; retained invalid items do not.
    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.items.iter().any(|item| match item {
            CssPseudoSelectorListItem::Selector(selector) => selector.has_pseudo_elements(),
            CssPseudoSelectorListItem::InvalidNesting(_) => false,
        })
    }
}

/// A relative selector: a leading combinator and the selector it relates to its anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeSelector {
    combinator: CssSelectorCombinator,
    selector: CssSelector,
}

impl CssRelativeSelector {
    /// Creates a relative selector from its leading combinator and selector.
    #[must_use]
    pub const fn new(combinator: CssSelectorCombinator, selector: CssSelector) -> Self {
        Self {
            combinator,
            selector,
        }
    }

    /// Returns the leading combinator, which is the descendant combinator when none was authored.
    #[must_use]
    pub const fn combinator(&self) -> CssSelectorCombinator {
        self.combinator
    }

    /// Returns the selector that follows the leading combinator.
    #[must_use]
    pub const fn selector(&self) -> &CssSelector {
        &self.selector
    }
}

/// A nonempty general relative list. Has attachment applies its additional
/// no-pseudo-element and no-nested-Has restrictions at the consuming boundary.
///
/// # Specified serialization
///
/// Emits general relatives with their explicit or implicit leading relation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeSelectorList {
    selectors: Vec<CssRelativeSelector>,
}

impl CssRelativeSelectorList {
    /// Checks every general relative member with default cumulative limits.
    ///
    /// Checks all leading carriers and their complete ordinary child graphs.
    /// A valid terminal pseudo-element or Has child is admitted here; attaching
    /// this list as Has arguments independently enforces the narrower context.
    pub fn try_new(
        selectors: Vec<CssRelativeSelector>,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        if selectors.is_empty() {
            return Err(CssSelectorConstructionError::empty_list());
        }
        let value = Self::new(selectors);
        value
            .serialize_specified(limits)
            .map_err(CssSelectorConstructionError::specified)?;
        Ok(value)
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssRelativeSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    /// Returns the relative selectors in authored order; the slice is never empty.
    #[must_use]
    pub fn selectors(&self) -> &[CssRelativeSelector] {
        &self.selectors
    }

    /// Returns whether any relative selector contains a pseudo-element.
    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.selectors
            .iter()
            .any(|selector| selector.selector().has_pseudo_elements())
    }
}

/// A pseudo-class selector, such as `:hover` or `:nth-child(2n+1)`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPseudoClass {
    /// `:host`: the host element of the shadow tree it is evaluated in.
    Host,
    /// `:host()`: the host element of the shadow tree it is evaluated in, when the host element matches one compound selector.
    HostFunction(CssCompoundSelectorArgument),
    /// `:host-context()`: the host element of the shadow tree it is evaluated in, when the host element or one of its ancestors matches one compound selector.
    HostContext(CssCompoundSelectorArgument),
    /// `:root`: the root element of the document.
    Root,
    /// `:scope`: the scoping root of the selector's context.
    Scope,
    /// `:defined`: an element that is fully defined, as dictated by the host language.
    Defined,
    /// `:any-link`: an element that acts as the source anchor of a hyperlink, equivalent to `:is(:link, :visited)`.
    AnyLink,
    /// `:link`: a hyperlink that has not been visited.
    Link,
    /// `:visited`: a hyperlink that has been visited.
    Visited,
    /// `:target`: the element that is the target of the document's URL fragment.
    Target,
    /// `:current`: the currently focused match instance, when combined with `::search-text`.
    Current,
    /// `:dir()`: an element whose directionality is the given one.
    Dir(CssDirectionality),
    /// `:lang()`: an element whose content language matches one of the given language ranges.
    Lang(CssLanguageRangeList),
    /// `:hover`: an element the user is pointing at with a pointing device.
    Hover,
    /// `:active`: an element that is being activated by the user.
    Active,
    /// `:focus`: the element that has input focus.
    Focus,
    /// `:focus-visible`: a focused element for which the user agent shows a focus indication.
    FocusVisible,
    /// `:focus-within`: an element that has focus or has a descendant that has focus.
    FocusWithin,
    /// `:disabled`: a user interface element that is in a disabled state.
    Disabled,
    /// `:enabled`: a user interface element that is in an enabled state.
    Enabled,
    /// `:checked`: an element that is toggled on, such as a checked checkbox or radio button or a selected option.
    Checked,
    /// `:unchecked`: an element that is toggled off, such as an unchecked checkbox or radio button or an unselected option.
    Unchecked,
    /// `:required`: a form element for which a value is required before its form can be validly submitted.
    Required,
    /// `:optional`: a form element for which a value is optional before its form can be validly submitted.
    Optional,
    /// `:valid`: an element whose contents or value is valid under the document language's data validity semantics.
    Valid,
    /// `:invalid`: an element whose contents or value is invalid under the document language's data validity semantics.
    Invalid,
    /// `:user-valid`: a valid form control after the user has interacted with it.
    UserValid,
    /// `:user-invalid`: an invalid form control after the user has interacted with it.
    UserInvalid,
    /// `:placeholder-shown`: an input element that is currently showing placeholder text.
    PlaceholderShown,
    /// The standard input autofill state; the required WebKit alias has this identity.
    Autofill,
    /// `:first-child`: an element that is the first child of its parent.
    FirstChild,
    /// `:last-child`: an element that is the last child of its parent.
    LastChild,
    /// `:only-child`: an element that is the only child of its parent.
    OnlyChild,
    /// `:empty`: an element with no children other than the exceptions the specification allows.
    Empty,
    /// `:nth-child()`: an element at a position matching the pattern among its siblings, optionally restricted to siblings matching a selector list.
    NthChild(CssNthChildPattern),
    /// `:nth-last-child()`: like `:nth-child()`, counting from the last sibling.
    NthLastChild(CssNthChildPattern),
    /// `:first-of-type`: an element that is the first of its type among its siblings.
    FirstOfType,
    /// `:last-of-type`: an element that is the last of its type among its siblings.
    LastOfType,
    /// `:only-of-type`: an element that is the only one of its type among its siblings.
    OnlyOfType,
    /// `:nth-of-type()`: an element at a position matching the pattern among siblings of its type.
    NthOfType(CssNthPattern),
    /// `:nth-last-of-type()`: like `:nth-of-type()`, counting from the last sibling of the type.
    NthLastOfType(CssNthPattern),
    /// `:not()`: an element that matches none of the argument selectors.
    Not(CssPseudoSelectorList),
    /// `:is()`: an element that matches any of the argument selectors.
    Is(CssPseudoSelectorList),
    /// `:where()`: like `:is()`, but with zero specificity.
    Where(CssPseudoSelectorList),
    /// `:has()`: an element for which any of the relative selectors matches relative to it.
    Has(CssRelativeSelectorList),
    /// `:playing`: a media element that is playing.
    Playing,
    /// `:paused`: a media element that is paused.
    Paused,
    /// `:seeking`: a media element that is seeking.
    Seeking,
    /// `:buffering`: a media element that cannot continue playing because it is obtaining media data.
    Buffering,
    /// `:stalled`: a media element that is obtaining media data but has received none for some time.
    Stalled,
    /// `:muted`: an element capable of making sound that is currently muted.
    Muted,
    /// `:volume-locked`: an element capable of making sound whose volume is locked by the user agent or the user.
    VolumeLocked,
    /// `:open`: an element that has open and closed states and is currently open.
    Open,
    /// `:modal`: an element in a state that excludes interaction with elements outside it until it is dismissed.
    Modal,
    /// `:fullscreen`: an element displayed in a mode that takes up most or all of the screen.
    Fullscreen,
    /// `:picture-in-picture`: an element displayed in a Picture-in-Picture mode, confined to part of the screen over other content.
    PictureInPicture,
    /// `:popover-open`: a popover element that is currently showing.
    PopoverOpen,
    /// `:default`: a user interface element that is the default among a set of similar elements.
    Default,
    /// `:indeterminate`: an element that could be checked or unchecked but is neither, or whose value is indeterminate, such as a progress meter with unknown progress.
    Indeterminate,
    /// `:read-only`: an element that the user cannot edit.
    ReadOnly,
    /// `:read-write`: an element that the user can edit.
    ReadWrite,
    /// `:in-range`: an element with range limits whose value is within them.
    InRange,
    /// `:out-of-range`: an element with range limits whose value is outside them.
    OutOfRange,
}

impl CssPseudoClass {
    /// Returns whether the arguments of the pseudo-class contain a pseudo-element.
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
            | Self::Defined
            | Self::AnyLink
            | Self::Link
            | Self::Visited
            | Self::Target
            | Self::Current
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
            | Self::Unchecked
            | Self::Required
            | Self::Optional
            | Self::Valid
            | Self::Invalid
            | Self::UserValid
            | Self::UserInvalid
            | Self::PlaceholderShown
            | Self::Autofill
            | Self::FirstChild
            | Self::LastChild
            | Self::OnlyChild
            | Self::Empty
            | Self::FirstOfType
            | Self::LastOfType
            | Self::OnlyOfType
            | Self::NthOfType(_)
            | Self::NthLastOfType(_)
            | Self::Playing
            | Self::Paused
            | Self::Seeking
            | Self::Buffering
            | Self::Stalled
            | Self::Muted
            | Self::VolumeLocked
            | Self::Open
            | Self::Modal
            | Self::Fullscreen
            | Self::PictureInPicture
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

/// The authored token form of a language range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLanguageRangeKind {
    /// The range was written as an identifier.
    Identifier,
    /// The range was written as a quoted string.
    String,
}

/// One checked identifier or string in a Selectors 4 language list.
/// Equality includes exact token spelling and provenance; it is not language matching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssLanguageRange {
    component: crate::component::CssComponentValue,
}

impl CssLanguageRange {
    /// Constructs a decoded identifier, escaping punctuation or spaces as needed.
    pub fn try_ident(
        value: impl Into<String>,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        Self::from_component(crate::component::CssComponentValue::try_ident(value)?)
    }

    /// Constructs a decoded string, including empty strings; NUL is rejected.
    pub fn try_string(
        value: impl Into<String>,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        Self::from_component(crate::component::CssComponentValue::try_string(value)?)
    }

    pub(crate) fn from_component(
        component: crate::component::CssComponentValue,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        if matches!(
            component.view(),
            crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Ident(_)
                    | crate::component::CssValueTokenRef::String(_)
            )
        ) {
            Ok(Self { component })
        } else {
            Err(crate::component::CssComponentValueError::new(
                crate::component::values::CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ))
        }
    }

    /// Returns the decoded range, without case folding or language validation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self.component.view() {
            crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Ident(value)
                | crate::component::CssValueTokenRef::String(value),
            ) => value,
            _ => unreachable!("checked language range token"),
        }
    }

    /// Returns the identifier or string token form.
    #[must_use]
    pub fn kind(&self) -> CssLanguageRangeKind {
        match self.component.view() {
            crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Ident(_),
            ) => CssLanguageRangeKind::Identifier,
            crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::String(_),
            ) => CssLanguageRangeKind::String,
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
        match self.kind() {
            CssLanguageRangeKind::Identifier => {
                surgeist_css_parser::Token::Ident(self.as_str()).to_string()
            }
            CssLanguageRangeKind::String => {
                surgeist_css_parser::Token::String(self.as_str()).to_string()
            }
        }
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
/// relative-selector grammar. Pseudo-elements and a `:has()` nested inside `:has()` fail.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCompoundSelectorArgument {
    compound: Box<CssCompoundSelector>,
}

impl CssCompoundSelectorArgument {
    /// Creates an argument from a selector, or returns `None` when `selector` is not a valid compound argument.
    ///
    /// Pseudo-elements, a `:has()` nested inside another `:has()`, and any complex selector (one with a combinator) are rejected.
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
    component: crate::component::CssComponentValue,
}

impl CssPartName {
    /// Constructs a decoded identifier. CSS-wide words and duplicates are allowed.
    pub fn try_new(
        value: impl Into<String>,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        Self::from_component(crate::component::CssComponentValue::try_ident(value)?)
    }

    pub(crate) fn from_component(
        component: crate::component::CssComponentValue,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        if matches!(
            component.view(),
            crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Ident(_)
            )
        ) {
            Ok(Self { component })
        } else {
            Err(crate::component::CssComponentValueError::new(
                crate::component::values::CssComponentValueErrorKind::InvalidIdentifier,
                component.origin().clone(),
            ))
        }
    }

    /// Returns the decoded identifier text, preserving case.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self.component.view() {
            crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Ident(value),
            ) => value,
            _ => unreachable!("checked part identifier"),
        }
    }

    /// Returns the original token origin, or explicit programmatic provenance.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }

    /// Serializes the identifier canonically, with CSS escapes where needed.
    #[must_use]
    pub fn to_css_string(&self) -> String {
        surgeist_css_parser::Token::Ident(self.as_str()).to_string()
    }
}

/// A nonempty ordered list of `::part()` identifiers, without name resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPartNameList {
    names: Vec<CssPartName>,
}

impl CssPartNameList {
    /// Creates a list, or returns `None` when `names` is empty.
    #[must_use]
    pub fn try_new(names: Vec<CssPartName>) -> Option<Self> {
        (!names.is_empty()).then_some(Self { names })
    }
    /// Returns the part names in authored order; the slice is never empty.
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

/// A parse-valid, nonfunctional unknown WebKit pseudo-element name.
///
/// This symbolic identity matches nothing. The complete selector retains its
/// originating compound and contextual restrictions; no matching is performed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssUnknownWebkitPseudoElement {
    name: String,
}

impl CssUnknownWebkitPseudoElement {
    /// Checks decoded identifier content and preserves non-ASCII code points.
    /// ASCII letters are canonicalized to lowercase for selector emission.
    pub fn try_new(name: impl Into<String>) -> Result<Self, CssSelectorConstructionError> {
        let name = name.into();
        let invalid = || {
            CssSelectorConstructionError::new(
                CssSelectorConstructionErrorKind::InvalidUnknownWebkitPseudoElementName,
            )
        };
        if !name
            .as_bytes()
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"-webkit-"))
        {
            return Err(invalid());
        }
        crate::component::CssComponentValue::try_ident(name.clone()).map_err(|_| invalid())?;
        Ok(Self {
            name: name.to_ascii_lowercase(),
        })
    }

    /// Returns the canonical decoded name, without colons or CSS escaping.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

/// A pseudo-element selector, such as `::before` or `::part(label)`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPseudoElement {
    /// `::before`: generated content before the element's content.
    Before,
    /// `::after`: generated content after the element's content.
    After,
    /// `::first-line`: the first formatted line of the element.
    FirstLine,
    /// `::first-letter`: the first letter of the first formatted line of the element.
    FirstLetter,
    /// `::prefix`: the punctuation preceding the first letter, available on `::first-letter`.
    Prefix,
    /// `::suffix`: the punctuation following the first letter, available on `::first-letter`.
    Suffix,
    /// `::marker`: the list marker of a list item.
    Marker,
    /// `::selection`: the portion of a document selected as the target of some possible future user-agent operation.
    Selection,
    /// `::search-text`: text matched by the user agent's find-in-page search.
    SearchText,
    /// `::target-text`: text directly targeted by the document URL's fragment.
    TargetText,
    /// `::spelling-error`: text that the user agent has marked as misspelled.
    SpellingError,
    /// `::grammar-error`: text that the user agent has marked as grammatically incorrect.
    GrammarError,
    /// A case-preserving custom highlight name, without registry lookup.
    Highlight(CssCustomIdent),
    /// `::placeholder`: the placeholder text of a form control.
    Placeholder,
    /// `::backdrop`: the backdrop rendered behind an element shown in the top layer.
    Backdrop,
    /// `::file-selector-button`: the button of a file input.
    FileSelectorButton,
    /// `::details-content`: the content wrapper of a `details` element.
    DetailsContent,
    /// The tree-abiding root of an authored view transition pseudo tree.
    ViewTransition,
    /// `::view-transition-group()`: the group of a named view transition participant, with a name or the `*` wildcard.
    ViewTransitionGroup(CssViewTransitionNameSelector),
    /// `::view-transition-image-pair()`: the image pair of a named view transition participant, with a name or the `*` wildcard.
    ViewTransitionImagePair(CssViewTransitionNameSelector),
    /// `::view-transition-old()`: the old-state snapshot of a named view transition participant, with a name or the `*` wildcard.
    ViewTransitionOld(CssViewTransitionNameSelector),
    /// `::view-transition-new()`: the new-state snapshot of a named view transition participant, with a name or the `*` wildcard.
    ViewTransitionNew(CssViewTransitionNameSelector),
    /// `::slotted()`: elements assigned to a slot that match the compound selector argument.
    Slotted(CssCompoundSelectorArgument),
    /// `::part()`: shadow parts whose `part` attribute lists all the given names.
    Part(CssPartNameList),
    /// An unknown `-webkit-` pseudo-element, which parses but matches nothing.
    UnknownWebkit(CssUnknownWebkitPseudoElement),
}

impl CssPseudoElement {
    pub(crate) fn suffix_context(&self) -> CssPseudoSuffixContext {
        match self {
            Self::Part(_) | Self::FileSelectorButton | Self::DetailsContent => {
                CssPseudoSuffixContext::ElementBacked
            }
            Self::SearchText => CssPseudoSuffixContext::SearchText,
            Self::ViewTransitionGroup(_)
            | Self::ViewTransitionImagePair(_)
            | Self::ViewTransitionOld(_)
            | Self::ViewTransitionNew(_) => CssPseudoSuffixContext::ViewTransitionDescendant,
            _ => CssPseudoSuffixContext::Generic,
        }
    }

    fn permits_child(&self, child: &Self) -> bool {
        match self {
            Self::Part(_) | Self::FileSelectorButton | Self::DetailsContent => true,
            Self::Slotted(_) => matches!(
                child,
                Self::Before
                    | Self::After
                    | Self::Marker
                    | Self::Placeholder
                    | Self::Part(_)
                    | Self::Backdrop
                    | Self::FileSelectorButton
                    | Self::DetailsContent
                    | Self::ViewTransition
                    | Self::ViewTransitionGroup(_)
                    | Self::ViewTransitionImagePair(_)
                    | Self::ViewTransitionOld(_)
                    | Self::ViewTransitionNew(_)
            ),
            Self::Before | Self::After => matches!(child, Self::Marker),
            Self::FirstLetter => matches!(child, Self::Prefix | Self::Suffix),
            Self::ViewTransition => matches!(child, Self::ViewTransitionGroup(_)),
            Self::ViewTransitionGroup(_) => matches!(child, Self::ViewTransitionImagePair(_)),
            Self::ViewTransitionImagePair(_) => {
                matches!(
                    child,
                    Self::ViewTransitionOld(_) | Self::ViewTransitionNew(_)
                )
            }
            _ => false,
        }
    }
}

/// One ordered segment following the originating element's compound selector.
/// Each pseudo-class applies to the most recent pseudo-element segment.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPseudoElementSegment {
    /// A pseudo-element.
    PseudoElement(CssPseudoElement),
    /// A pseudo-class that qualifies the most recent pseudo-element.
    PseudoClass(CssPseudoClass),
}

/// A nonempty ordered sequence of pseudo-elements and of the pseudo-classes that qualify them, following the originating element's compound selector.
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
                    if !pseudo_is_valid_suffix(pseudo, element.suffix_context(), true) {
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

/// The pseudo-class set available at the most recent pseudo-element segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CssPseudoSuffixContext {
    Generic,
    SearchText,
    ViewTransitionDescendant,
    ElementBacked,
}

impl CssPseudoSuffixContext {
    fn admits(self, pseudo: &CssPseudoClass) -> bool {
        self == Self::ElementBacked
            || matches!(
                pseudo,
                CssPseudoClass::Not(_)
                    | CssPseudoClass::Is(_)
                    | CssPseudoClass::Where(_)
                    | CssPseudoClass::Hover
                    | CssPseudoClass::Active
                    | CssPseudoClass::Focus
                    | CssPseudoClass::FocusVisible
                    | CssPseudoClass::FocusWithin
            )
            || (self == Self::SearchText && matches!(pseudo, CssPseudoClass::Current))
            || (self == Self::ViewTransitionDescendant
                && matches!(pseudo, CssPseudoClass::OnlyChild))
    }

    /// Ordinary classes exclude Current, so ordinary and SearchText permissions
    /// are incomparable. View-transition descendants add OnlyChild independently
    /// of SearchText's Current. Has and compound restrictions remain separate proofs.
    fn is_subset_of(receiver: Option<Self>, original: Option<Self>) -> bool {
        matches!(
            (receiver, original),
            (Some(Self::Generic), _)
                | (None, None | Some(Self::ElementBacked))
                | (
                    Some(Self::SearchText),
                    Some(Self::SearchText | Self::ElementBacked)
                )
                | (
                    Some(Self::ViewTransitionDescendant),
                    None | Some(Self::ViewTransitionDescendant | Self::ElementBacked)
                )
                | (Some(Self::ElementBacked), Some(Self::ElementBacked))
        )
    }
}

/// Intrinsic authored grammar inherited by selector-function arguments. Matching
/// and namespace resolution are separate from these structural restrictions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CssSelectorGrammarContext {
    allow_pseudo_elements: bool,
    allow_has: bool,
    compound_only: bool,
    pseudo_suffix: Option<CssPseudoSuffixContext>,
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
        pseudo_suffix: Option<CssPseudoSuffixContext>,
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
            // Every suffix also requires pseudo-class-only selector shape,
            // independently of which pseudo-classes its receiver permits.
            && (original.pseudo_suffix.is_none() || self.pseudo_suffix.is_some())
            && CssPseudoSuffixContext::is_subset_of(self.pseudo_suffix, original.pseudo_suffix)
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

    pub(crate) fn suffix(self, suffix: CssPseudoSuffixContext) -> Self {
        Self {
            pseudo_suffix: Some(suffix),
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
        let suffix = match self.pseudo_suffix {
            None => !matches!(pseudo, CssPseudoClass::Current),
            Some(context) => context.admits(pseudo),
        };
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
    suffix: CssPseudoSuffixContext,
    allow_has: bool,
) -> bool {
    selector_is_valid_in_context(
        selector,
        CssSelectorGrammarContext {
            allow_has,
            ..CssSelectorGrammarContext::ORDINARY.suffix(suffix)
        },
    )
}

fn pseudo_is_valid_suffix(
    pseudo: &CssPseudoClass,
    suffix: CssPseudoSuffixContext,
    allow_has: bool,
) -> bool {
    pseudo_is_valid_in_context(
        pseudo,
        CssSelectorGrammarContext {
            allow_has,
            ..CssSelectorGrammarContext::ORDINARY.suffix(suffix)
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

/// The argument of `:nth-child()` or `:nth-last-child()`: a position pattern with an optional `of <selector-list>` filter.
#[derive(Clone, Debug, PartialEq)]
pub struct CssNthChildPattern {
    pattern: CssNthPattern,
    selector_list: Option<CssPseudoSelectorList>,
}

impl CssNthChildPattern {
    /// Creates an argument from a pattern and an optional selector list.
    #[must_use]
    pub const fn new(pattern: CssNthPattern, selector_list: Option<CssPseudoSelectorList>) -> Self {
        Self {
            pattern,
            selector_list,
        }
    }

    /// Returns the position pattern.
    #[must_use]
    pub const fn pattern(&self) -> CssNthPattern {
        self.pattern
    }

    /// Returns the `of <selector-list>` filter, or `None` when none was authored.
    #[must_use]
    pub const fn selector_list(&self) -> Option<&CssPseudoSelectorList> {
        self.selector_list.as_ref()
    }

    /// Returns whether the filter selector list contains a pseudo-element.
    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.selector_list
            .as_ref()
            .is_some_and(CssPseudoSelectorList::has_pseudo_elements)
    }
}

/// A position pattern, the `<an+b>` micro-syntax of the `:nth-*` pseudo-classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNthPattern {
    /// `odd`: positions 1, 3, 5, ... (equivalent to `2n+1`).
    Odd,
    /// `even`: positions 2, 4, 6, ... (equivalent to `2n`).
    Even,
    /// A plain integer: exactly that position.
    Integer(i32),
    /// An explicit `an+b` pattern.
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
        context: &crate::selector::CssNamespaceContext,
        apply_default: bool,
    ) -> Result<CssNamespaceConstraint, CssSelectorConstructionError> {
        Ok(match self {
            Self::Unqualified if !apply_default => CssNamespaceConstraint::ExplicitNone,
            Self::Unqualified if context.default_namespace().is_some() => {
                CssNamespaceConstraint::Default
            }
            Self::Unqualified | Self::Any => CssNamespaceConstraint::Any,
            Self::ExplicitNone => CssNamespaceConstraint::ExplicitNone,
            Self::Named(prefix) => {
                if context.named_namespace(prefix).is_none() {
                    return Err(CssSelectorConstructionError::new(
                        CssSelectorConstructionErrorKind::UndeclaredNamespacePrefix(prefix.clone()),
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
    ) -> Result<crate::component::CssSerializedValue, crate::component::CssComponentValueError>
    {
        use crate::component::values::serialize::{CssCanonicalBuilder, CssCanonicalToken};
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
    local_name: Option<crate::values::common::CssIdent>,
}

impl CssQualifiedSelectorName {
    /// Checks a decoded type name against explicit namespace bindings.
    pub fn try_new(
        prefix: CssQualifiedNamePrefix,
        local_name: crate::values::common::CssIdent,
        context: &crate::selector::CssNamespaceContext,
    ) -> Result<Self, CssSelectorConstructionError> {
        let namespace = prefix.namespace(context, true)?;
        Ok(Self::new(namespace, prefix, local_name))
    }

    /// Checks a universal selector against explicit namespace bindings.
    pub fn try_universal(
        prefix: CssQualifiedNamePrefix,
        context: &crate::selector::CssNamespaceContext,
    ) -> Result<Self, CssSelectorConstructionError> {
        let namespace = prefix.namespace(context, true)?;
        Ok(Self::universal(namespace, prefix))
    }

    pub(crate) fn new(
        namespace: CssNamespaceConstraint,
        prefix: CssQualifiedNamePrefix,
        local_name: crate::values::common::CssIdent,
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
        self.local_name
            .as_ref()
            .map(crate::values::common::CssIdent::as_str)
    }

    /// Reports whether this name is the universal selector.
    #[must_use]
    pub const fn is_universal(&self) -> bool {
        self.local_name.is_none()
    }

    /// Serializes this name as tokens, bounding the generated UTF-8 bytes by
    /// `limits.max_css_bytes()`; the other component limits do not apply.
    ///
    /// Escaping preserves decoded identity and authored prefix form. Generated
    /// tokens have programmatic origins; enclosing parsed rule origins remain
    /// available separately. This does not serialize a complete selector or rule.
    pub fn serialize_tokens(
        &self,
        limits: crate::component::CssComponentValueLimits,
    ) -> Result<crate::component::CssSerializedValue, crate::component::CssComponentValueError>
    {
        self.prefix
            .serialize_name(self.local_name(), limits.max_css_bytes())
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
        context: &crate::selector::CssNamespaceContext,
    ) -> Result<Self, CssSelectorConstructionError> {
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

    /// Returns the namespace constraint of the attribute name.
    #[must_use]
    pub const fn namespace(&self) -> &CssNamespaceConstraint {
        &self.namespace
    }

    /// Returns the authored prefix form of the name.
    #[must_use]
    pub const fn prefix(&self) -> &CssQualifiedNamePrefix {
        &self.prefix
    }

    /// Returns the local name without the prefix.
    #[must_use]
    pub const fn local_name(&self) -> &CssAttributeName {
        &self.local_name
    }

    /// Serializes this qualified name as tokens with programmatic origins,
    /// bounding the bytes by `limits.max_css_bytes()`; the other component
    /// limits do not apply. This name grammar is for attribute selectors, not
    /// `attr()` host admission.
    pub fn serialize_tokens(
        &self,
        limits: crate::component::CssComponentValueLimits,
    ) -> Result<crate::component::CssSerializedValue, crate::component::CssComponentValueError>
    {
        self.prefix
            .serialize_name(Some(self.local_name.as_str()), limits.max_css_bytes())
    }
}

/// The `an+b` pattern: matches positions `a*n + b` for every integer `n >= 0`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssNthAnPlusB {
    a: i32,
    b: i32,
}

impl CssNthAnPlusB {
    /// Creates the pattern `an+b` from its two coefficients.
    #[must_use]
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    /// Returns the step `a`.
    #[must_use]
    pub const fn a(self) -> i32 {
        self.a
    }

    /// Returns the offset `b`.
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
    /// A type or universal selector, possibly namespace-qualified.
    Type(CssQualifiedSelectorName),
    /// An ID selector, carrying the decoded identifier.
    Id(crate::values::common::CssIdent),
    /// A class selector, carrying the decoded identifier.
    Class(crate::values::common::CssIdent),
    /// An attribute selector.
    Attribute(CssAttributeSelector),
    /// A pseudo-class selector.
    PseudoClass(CssPseudoClass),
}

/// One simple selector after a compound's type selector, in authored order.
/// Category members index their position in that category's list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CssCompoundMember {
    Nesting,
    ScopeAnchor,
    Id(usize),
    Class(usize),
    Attribute(usize),
    PseudoClass(usize),
}

/// Ordered simple-selector members of one compound under assembly. Each push
/// records its category entry and its authored position together, so the
/// grouped lists and the member order cannot disagree.
#[derive(Default)]
pub(crate) struct CssCompoundMembers {
    scope_anchors: usize,
    nesting_selectors: usize,
    ids: Vec<String>,
    classes: Vec<String>,
    attributes: Vec<CssAttributeSelector>,
    pseudo_classes: Vec<CssPseudoClass>,
    order: Vec<CssCompoundMember>,
}

impl CssCompoundMembers {
    pub(crate) fn push_nesting(&mut self) {
        self.nesting_selectors += 1;
        self.order.push(CssCompoundMember::Nesting);
    }

    pub(crate) fn push_scope_anchor(&mut self) {
        self.scope_anchors += 1;
        self.order.push(CssCompoundMember::ScopeAnchor);
    }

    pub(crate) fn push_id(&mut self, value: String) {
        self.order.push(CssCompoundMember::Id(self.ids.len()));
        self.ids.push(value);
    }

    pub(crate) fn push_class(&mut self, value: String) {
        self.order
            .push(CssCompoundMember::Class(self.classes.len()));
        self.classes.push(value);
    }

    pub(crate) fn push_attribute(&mut self, value: CssAttributeSelector) {
        self.order
            .push(CssCompoundMember::Attribute(self.attributes.len()));
        self.attributes.push(value);
    }

    pub(crate) fn push_pseudo_class(&mut self, value: CssPseudoClass) {
        self.order
            .push(CssCompoundMember::PseudoClass(self.pseudo_classes.len()));
        self.pseudo_classes.push(value);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub(crate) fn into_compound(
        self,
        type_selector: Option<CssQualifiedSelectorName>,
        pseudo_elements: Option<CssPseudoElementSequence>,
    ) -> CssCompoundSelector {
        CssCompoundSelector {
            scope_anchors: self.scope_anchors,
            nesting_selectors: self.nesting_selectors,
            type_selector: type_selector.map(Box::new),
            ids: self.ids,
            classes: self.classes,
            attributes: self.attributes,
            pseudo_classes: self.pseudo_classes,
            order: self.order,
            pseudo_elements,
        }
    }
}

/// A compound selector: an optional leading type selector, its other simple
/// selectors in authored order, and an optional pseudo-element sequence.
///
/// The per-category accessors expose each category's members in their
/// relative authored order; serialization emits every simple selector at
/// its authored position (CSSOM "serialize a selector").
#[derive(Clone, Debug, PartialEq)]
pub struct CssCompoundSelector {
    scope_anchors: usize,
    nesting_selectors: usize,
    type_selector: Option<Box<CssQualifiedSelectorName>>,
    ids: Vec<String>,
    classes: Vec<String>,
    attributes: Vec<CssAttributeSelector>,
    pseudo_classes: Vec<CssPseudoClass>,
    order: Vec<CssCompoundMember>,
    pseudo_elements: Option<CssPseudoElementSequence>,
}

impl CssCompoundSelector {
    /// Assembles nonempty simple members, admitting a unique type/universal first.
    /// Every member, including repeated IDs and classes, keeps its given order.
    ///
    /// ```
    /// use surgeist_css::selector::{CssComplexSelector, CssComplexSelectorPart, CssCompoundSelector, CssSelector, CssSelectorCombinator, CssSimpleSelector};
    /// use surgeist_css::values::common::CssIdent;
    /// let limits = Default::default();
    /// let first = CssCompoundSelector::try_new(vec![
    ///     CssSimpleSelector::Class(CssIdent::try_new("Parent").unwrap()),
    /// ], limits).unwrap();
    /// let child = CssCompoundSelector::try_new(vec![
    ///     CssSimpleSelector::Class(CssIdent::try_new("Child").unwrap()),
    /// ], limits).unwrap();
    /// let part =
    ///     CssComplexSelectorPart::try_new(CssSelectorCombinator::Child, child, limits).unwrap();
    /// let selector =
    ///     CssSelector::Complex(CssComplexSelector::try_new(first, vec![part], limits).unwrap());
    /// assert_eq!(selector.serialize_specified(Default::default()).unwrap(), ".Parent > .Child");
    /// ```
    ///
    /// Checks member order, then admits the complete ordinary graph and output
    /// with one cumulative budget. No partially assembled compound escapes.
    pub fn try_new(
        members: Vec<CssSimpleSelector>,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<Self, CssSelectorConstructionError> {
        if members.is_empty() {
            return Err(CssSelectorConstructionError::new(
                CssSelectorConstructionErrorKind::EmptyCompound,
            ));
        }
        let mut type_selector = None;
        let mut ordered = CssCompoundMembers::default();
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
                CssSimpleSelector::Id(value) => ordered.push_id(value.as_str().to_owned()),
                CssSimpleSelector::Class(value) => ordered.push_class(value.as_str().to_owned()),
                CssSimpleSelector::Attribute(value) => ordered.push_attribute(value),
                CssSimpleSelector::PseudoClass(value) => ordered.push_pseudo_class(value),
            }
        }
        let value = ordered.into_compound(type_selector, None);
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
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
                crate::values::common::CssIdent::new(tag),
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

    /// Assembles category lists in category order: scope anchors, IDs,
    /// classes, attributes, then pseudo-classes.
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
        let mut ordered = CssCompoundMembers::default();
        for _ in 0..scope_anchors {
            ordered.push_scope_anchor();
        }
        ids.into_iter().for_each(|value| ordered.push_id(value));
        classes
            .into_iter()
            .for_each(|value| ordered.push_class(value));
        attributes
            .into_iter()
            .for_each(|value| ordered.push_attribute(value));
        pseudo_classes
            .into_iter()
            .for_each(|value| ordered.push_pseudo_class(value));
        ordered.into_compound(type_selector, pseudo_elements)
    }

    /// The simple selectors after the type selector, in authored order.
    #[must_use]
    pub(crate) fn members(&self) -> &[CssCompoundMember] {
        &self.order
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

    /// Returns whether the compound contains at least one symbolic scope anchor.
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

    /// Whether the authored unqualified form has Any namespace; a simple tag also needs a local
    /// name.
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

    /// Returns the class names in authored order.
    #[must_use]
    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    /// Returns the attribute selectors in authored order.
    #[must_use]
    pub fn attributes(&self) -> &[CssAttributeSelector] {
        &self.attributes
    }

    /// Returns the pseudo-classes of the originating element in authored order, excluding those that qualify a pseudo-element.
    #[must_use]
    pub fn pseudo_classes(&self) -> &[CssPseudoClass] {
        &self.pseudo_classes
    }

    /// Returns the pseudo-element sequence, or `None` when the compound has no pseudo-element.
    #[must_use]
    pub const fn pseudo_elements(&self) -> Option<&CssPseudoElementSequence> {
        self.pseudo_elements.as_ref()
    }

    /// Returns whether the compound has a pseudo-element.
    #[must_use]
    pub const fn has_pseudo_elements(&self) -> bool {
        self.pseudo_elements.is_some()
    }
}

/// An attribute selector: a qualified attribute name, a matcher and a case-sensitivity mode.
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
    ///
    /// Checks intrinsic operand identity and charges canonical attribute output
    /// within the supplied semantic-node and UTF-8 byte limits.
    pub fn try_new(
        name: CssQualifiedAttributeName,
        matcher: CssAttributeMatcher,
        case_sensitivity: CssAttributeCaseSensitivity,
        limits: crate::serialization::CssSerializationLimits,
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
                crate::component::CssComponentValue::try_string(value.clone()).map_err(
                    |error| {
                        CssSelectorConstructionError::new(
                            CssSelectorConstructionErrorKind::InvalidAttributeValue(error),
                        )
                    },
                )?;
            }
        }
        let value = Self::new_qualified(name, matcher, case_sensitivity);
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
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

    /// Returns the local attribute name without its namespace prefix.
    #[must_use]
    pub const fn name(&self) -> &CssAttributeName {
        self.name.local_name()
    }

    /// Returns how the attribute value is compared.
    #[must_use]
    pub const fn matcher(&self) -> &CssAttributeMatcher {
        &self.matcher
    }

    /// Returns the case-sensitivity mode.
    #[must_use]
    pub const fn case_sensitivity(&self) -> CssAttributeCaseSensitivity {
        self.case_sensitivity
    }
}

/// A decoded attribute name, which may need escapes when written in CSS.
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
        crate::component::CssComponentValue::try_ident(name.clone()).ok()?;
        Some(Self { name })
    }

    #[must_use]
    pub(crate) fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(Self::try_new(name.clone()).is_some());
        Self { name }
    }

    /// Returns the decoded attribute name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

/// How an attribute selector compares the attribute value; the operand is the decoded string.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAttributeMatcher {
    /// `[attr]`: the attribute is present.
    Exists,
    /// `[attr=value]`: the attribute value is exactly the operand.
    Equals(String),
    /// `[attr~=value]`: the attribute value is a whitespace-separated list containing the operand.
    Includes(String),
    /// `[attr|=value]`: the attribute value is exactly the operand or begins with the operand followed by `-`.
    DashMatch(String),
    /// `[attr^=value]`: the attribute value begins with the operand.
    Prefix(String),
    /// `[attr$=value]`: the attribute value ends with the operand.
    Suffix(String),
    /// `[attr*=value]`: the attribute value contains the operand.
    Substring(String),
}

/// The case-sensitivity modifier of an attribute selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAttributeCaseSensitivity {
    /// No modifier: case sensitivity follows the document language.
    DocumentDefault,
    /// `i`: the value is compared ASCII case-insensitively.
    AsciiCaseInsensitive,
    /// `s`: the value is compared case-sensitively.
    ExplicitSensitive,
}
impl CssSelector {
    /// Parses exactly one ordinary selector using the supplied namespace bindings.
    ///
    /// Rejected outer syntax is `None` with diagnostics. Forgiving inner lists retain
    /// admitted members and diagnose invalid members. Invalid delimiter-`&` members
    /// remain explicit match-nothing items; other invalid members are discarded.
    /// Positions refer directly to `source`;
    /// implicit EOF closures are reported only when the outer syntax is retained.
    /// Explicit `&` anchors remain symbolic; without a parent selector list they match
    /// the context's scope elements and contribute zero specificity. Leading relative
    /// combinators are not admitted by this front door.
    ///
    /// The supplied namespace bindings apply to qualified names.
    ///
    /// The parser is not read: neither its document mode nor its component limits apply to this grammar, and only the fixed nesting ceiling bounds the input.
    #[expect(
        unused_variables,
        reason = "this grammar does not read the parser configuration"
    )]
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
        namespaces: &crate::selector::CssNamespaceContext,
    ) -> crate::CssParseReport<Option<CssSelector>> {
        crate::parser::fragments::parse_selector(source.as_ref(), namespaces)
    }
}
impl CssRelativeSelectorList {
    /// Parses a complete nonempty general relative selector list atomically.
    ///
    /// Each member retains its explicit leading `>`, `+`, `~` or `||` relationship,
    /// or an implicit descendant relation. Namespace bindings and diagnostics use
    /// the caller's original source. One invalid outer member rejects the whole list;
    /// forgiving inner Is/Where recovery retains its existing diagnostics. Use
    /// `into_validation_result` when only a clean report is acceptable.
    ///
    /// General relative grammar permits legal terminal pseudo-elements and independent
    /// Has functions. Attaching the resulting list as Has's own argument imposes its
    /// narrower pseudo-element and nested-Has restrictions. Detached anchors remain
    /// symbolic; parsing does not bind or match them. Specified-output limits apply
    /// to later serialization or checked readmission, separately from parsing.
    ///
    /// ```
    /// use surgeist_css::selector::{CssNamespaceContext, CssSelectorCombinator, CssRelativeSelectorList};
    /// use surgeist_css::CssParser;
    /// let list = CssRelativeSelectorList::parse(&CssParser::default(), "> .Child, .Descendant", &CssNamespaceContext::default()).into_validation_result().unwrap().unwrap();
    /// assert_eq!(list.selectors()[0].combinator(), CssSelectorCombinator::Child);
    /// assert_eq!(list.selectors()[1].combinator(), CssSelectorCombinator::Descendant);
    /// assert_eq!(list.serialize_specified(Default::default()).unwrap(), "> .Child, .Descendant");
    /// ```
    ///
    /// An ordinary checked list cannot take a relative carrier:
    ///
    /// ```compile_fail
    /// use surgeist_css::selector::{CssRelativeSelector, CssSelector, CssSelectorCombinator, CssSelectorList};
    /// let relative = CssRelativeSelector::new(CssSelectorCombinator::Child, CssSelector::Class("One".into()));
    /// let _ = CssSelectorList::try_new(vec![relative]);
    /// ```
    ///
    /// The parser is not read: neither its document mode nor its component limits apply to this grammar, and only the fixed nesting ceiling bounds the input.
    #[expect(
        unused_variables,
        reason = "this grammar does not read the parser configuration"
    )]
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
        namespaces: &crate::selector::CssNamespaceContext,
    ) -> crate::CssParseReport<Option<CssRelativeSelectorList>> {
        crate::parser::fragments::parse_relative_selector_list(source.as_ref(), namespaces)
    }
}
