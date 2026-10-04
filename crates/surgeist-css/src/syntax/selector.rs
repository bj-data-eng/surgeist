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
    #[must_use]
    pub fn try_new(first: CssCompoundSelector, rest: Vec<CssComplexSelectorPart>) -> Option<Self> {
        if rest.is_empty() || complex_selector_has_non_terminal_pseudo_elements(&first, &rest) {
            None
        } else {
            Some(Self::new(first, rest))
        }
    }

    #[must_use]
    fn new(first: CssCompoundSelector, rest: Vec<CssComplexSelectorPart>) -> Self {
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

fn complex_selector_has_non_terminal_pseudo_elements(
    first: &CssCompoundSelector,
    rest: &[CssComplexSelectorPart],
) -> bool {
    first.has_pseudo_elements()
        || rest
            .iter()
            .take(rest.len().saturating_sub(1))
            .any(|part| part.selector().has_pseudo_elements())
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssComplexSelectorPart {
    combinator: CssSelectorCombinator,
    selector: CssCompoundSelector,
}

impl CssComplexSelectorPart {
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssSelectorList {
    selectors: Vec<CssSelector>,
}

impl CssSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
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

#[derive(Clone, Debug, PartialEq)]
pub struct CssPseudoSelectorList {
    selectors: Vec<CssSelector>,
}

impl CssPseudoSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    #[must_use]
    pub(crate) const fn new_forgiving(selectors: Vec<CssSelector>) -> Self {
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssSelector] {
        &self.selectors
    }

    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.selectors.iter().any(CssSelector::has_pseudo_elements)
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

#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeSelectorList {
    selectors: Vec<CssRelativeSelector>,
}

impl CssRelativeSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssRelativeSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
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

pub(crate) fn selector_is_valid_pseudo_suffix(
    selector: &CssSelector,
    element_backed: bool,
    allow_has: bool,
) -> bool {
    match selector {
        CssSelector::PseudoClass(pseudo) => {
            pseudo_is_valid_suffix(pseudo, element_backed, allow_has)
        }
        CssSelector::Compound(compound) => {
            compound.type_selector().is_none()
                && compound.ids().is_empty()
                && compound.classes().is_empty()
                && compound.attributes().is_empty()
                && !compound.has_scope_anchor()
                && compound.nesting_selectors() == 0
                && !compound.has_pseudo_elements()
                && !compound.pseudo_classes().is_empty()
                && compound
                    .pseudo_classes()
                    .iter()
                    .all(|pseudo| pseudo_is_valid_suffix(pseudo, element_backed, allow_has))
        }
        _ => false,
    }
}

fn pseudo_is_valid_suffix(pseudo: &CssPseudoClass, element_backed: bool, allow_has: bool) -> bool {
    match pseudo {
        CssPseudoClass::Not(list) | CssPseudoClass::Is(list) | CssPseudoClass::Where(list) => {
            list.selectors().iter().all(|selector| {
                selector_is_valid_pseudo_suffix(selector, element_backed, allow_has)
            }) && (!matches!(pseudo, CssPseudoClass::Not(_)) || !list.selectors().is_empty())
        }
        _ => {
            (element_backed
                || matches!(
                    pseudo,
                    CssPseudoClass::Hover
                        | CssPseudoClass::Active
                        | CssPseudoClass::Focus
                        | CssPseudoClass::FocusVisible
                        | CssPseudoClass::FocusWithin
                ))
                && pseudo_is_valid_argument(pseudo, allow_has, false)
        }
    }
}

fn selector_is_valid_argument(
    selector: &CssSelector,
    allow_has: bool,
    compound_only: bool,
) -> bool {
    match selector {
        CssSelector::Tag(value) | CssSelector::Key(value) | CssSelector::Class(value) => {
            !value.is_empty() && !value.contains('\0')
        }
        CssSelector::PseudoClass(pseudo) => {
            pseudo_is_valid_argument(pseudo, allow_has, compound_only)
        }
        CssSelector::Compound(compound) => {
            compound_is_valid_argument(compound, allow_has, compound_only)
        }
        CssSelector::Complex(complex) => {
            !compound_only
                && compound_is_valid_argument(complex.first(), allow_has, false)
                && complex
                    .rest()
                    .iter()
                    .all(|part| compound_is_valid_argument(part.selector(), allow_has, false))
        }
    }
}

fn compound_is_valid_argument(
    compound: &CssCompoundSelector,
    allow_has: bool,
    compound_only: bool,
) -> bool {
    !compound.has_pseudo_elements()
        && compound
            .pseudo_classes()
            .iter()
            .all(|pseudo| pseudo_is_valid_argument(pseudo, allow_has, compound_only))
}

fn pseudo_is_valid_argument(pseudo: &CssPseudoClass, allow_has: bool, compound_only: bool) -> bool {
    match pseudo {
        CssPseudoClass::HostFunction(argument) | CssPseudoClass::HostContext(argument) => {
            compound_is_valid_argument(argument.compound(), allow_has, true)
        }
        CssPseudoClass::Not(list) | CssPseudoClass::Is(list) | CssPseudoClass::Where(list) => {
            list.selectors()
                .iter()
                .all(|selector| selector_is_valid_argument(selector, allow_has, compound_only))
                && (!matches!(pseudo, CssPseudoClass::Not(_)) || !list.selectors().is_empty())
        }
        CssPseudoClass::Has(list) => {
            allow_has
                && list
                    .selectors()
                    .iter()
                    .all(|relative| selector_is_valid_argument(relative.selector(), false, false))
        }
        CssPseudoClass::NthChild(pattern) | CssPseudoClass::NthLastChild(pattern) => {
            pattern.selector_list().is_none_or(|list| {
                !list.selectors().is_empty()
                    && list
                        .selectors()
                        .iter()
                        .all(|selector| selector_is_valid_argument(selector, allow_has, false))
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

/// One parser-produced namespace-qualified type or universal selector name.
///
/// The private representation admits either one decoded CSS identifier or the
/// universal selector, never an empty or otherwise invalid local name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssQualifiedSelectorName {
    namespace: CssNamespaceConstraint,
    local_name: Option<String>,
}

impl CssQualifiedSelectorName {
    #[must_use]
    pub(crate) fn new(namespace: CssNamespaceConstraint, local_name: String) -> Self {
        debug_assert!(!local_name.is_empty());
        Self {
            namespace,
            local_name: Some(local_name),
        }
    }

    #[must_use]
    pub(crate) const fn universal(namespace: CssNamespaceConstraint) -> Self {
        Self {
            namespace,
            local_name: None,
        }
    }

    /// Returns the namespace constraint active for this selector name.
    #[must_use]
    pub const fn namespace(&self) -> &CssNamespaceConstraint {
        &self.namespace
    }

    /// Returns the decoded local identifier, or `None` for universal `*`.
    #[must_use]
    pub fn local_name(&self) -> Option<&str> {
        self.local_name.as_deref()
    }

    /// Reports whether this name is the universal selector.
    #[must_use]
    pub const fn is_universal(&self) -> bool {
        self.local_name.is_none()
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

#[derive(Clone, Debug, PartialEq)]
pub struct CssCompoundSelector {
    scope_anchors: usize,
    nesting_selectors: usize,
    type_selector: Option<Box<(CssQualifiedSelectorName, bool)>>,
    ids: Vec<String>,
    classes: Vec<String>,
    attributes: Vec<CssAttributeSelector>,
    pseudo_classes: Vec<CssPseudoClass>,
    pseudo_elements: Option<CssPseudoElementSequence>,
}

impl CssCompoundSelector {
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
            (
                CssQualifiedSelectorName::new(CssNamespaceConstraint::Any, tag),
                true,
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
        type_selector: Option<(CssQualifiedSelectorName, bool)>,
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
        match self.type_selector.as_deref() {
            Some((type_selector, _)) => Some(type_selector),
            None => None,
        }
    }

    /// Whether the authored unqualified form has Any namespace; a simple tag also needs a local name.
    #[must_use]
    pub(crate) fn has_unqualified_any_namespace(&self) -> bool {
        matches!(self.type_selector.as_deref(), Some((_, true)))
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
    namespace: CssNamespaceConstraint,
    name: CssAttributeName,
    matcher: CssAttributeMatcher,
    case_sensitivity: CssAttributeCaseSensitivity,
}

impl CssAttributeSelector {
    #[must_use]
    pub(crate) const fn new_qualified(
        namespace: CssNamespaceConstraint,
        name: CssAttributeName,
        matcher: CssAttributeMatcher,
        case_sensitivity: CssAttributeCaseSensitivity,
    ) -> Self {
        Self {
            namespace,
            name,
            matcher,
            case_sensitivity,
        }
    }

    /// Returns the namespace constraint for the attribute name.
    #[must_use]
    pub const fn namespace(&self) -> &CssNamespaceConstraint {
        &self.namespace
    }

    #[must_use]
    pub const fn name(&self) -> &CssAttributeName {
        &self.name
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
