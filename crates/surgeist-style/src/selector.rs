use super::{Error, ErrorCode, Result, Traversal, Tree};
use crate::{StateFlag, StyleAttributeName, StyleAttributeValue, StyleClass, StyleKey, StyleTag};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PrimaryKey {
    Universal,
    Key(StyleKey),
    Class(StyleClass),
    Tag(StyleTag),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SelectorSpecificity {
    ids: u16,
    classes: u16,
    elements: u16,
}

impl SelectorSpecificity {
    #[must_use]
    pub const fn new(ids: u16, classes: u16, elements: u16) -> Self {
        Self {
            ids,
            classes,
            elements,
        }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    #[must_use]
    pub const fn ids(self) -> u16 {
        self.ids
    }

    #[must_use]
    pub const fn classes(self) -> u16 {
        self.classes
    }

    #[must_use]
    pub const fn elements(self) -> u16 {
        self.elements
    }

    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self::new(
            self.ids.saturating_add(other.ids),
            self.classes.saturating_add(other.classes),
            self.elements.saturating_add(other.elements),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Selector {
    Any,
    Tag(StyleTag),
    Class(StyleClass),
    Key(StyleKey),
    State(StateFlag),
    Attribute(AttributeSelector),
    Position(PositionSelector),
    Pseudo(PseudoClassSelector),
    Compound(Compound),
    Complex(ComplexSelector),
    List(SelectorList),
}

impl Selector {
    #[must_use]
    pub const fn any() -> Self {
        Self::Any
    }

    pub fn tag(tag: impl AsRef<str>) -> Result<Self> {
        Ok(Self::Tag(tag_from_str(tag.as_ref())?))
    }

    pub fn class(class: impl AsRef<str>) -> Result<Self> {
        Ok(Self::Class(class_from_str(class.as_ref())?))
    }

    pub fn key(key: impl AsRef<str>) -> Result<Self> {
        Ok(Self::Key(key_from_str(key.as_ref())?))
    }

    #[must_use]
    pub const fn state(state: StateFlag) -> Self {
        Self::State(state)
    }

    pub fn attribute_exists(name: impl AsRef<str>) -> Result<Self> {
        Ok(Self::Attribute(AttributeSelector::exists(name)?))
    }

    pub fn attribute_equals(name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        Ok(Self::Attribute(AttributeSelector::equals(name, value)?))
    }

    #[must_use]
    pub const fn position(position: PositionSelector) -> Self {
        Self::Position(position)
    }

    #[must_use]
    pub const fn pseudo(pseudo_class: PseudoClassSelector) -> Self {
        Self::Pseudo(pseudo_class)
    }

    #[must_use]
    pub fn compound() -> Compound {
        Compound::new()
    }

    pub fn complex(parts: impl IntoIterator<Item = ComplexSelectorPart>) -> Result<Self> {
        Self::try_complex(parts)
    }

    pub fn try_complex(parts: impl IntoIterator<Item = ComplexSelectorPart>) -> Result<Self> {
        Ok(Self::Complex(ComplexSelector::try_new(parts)?))
    }

    #[must_use]
    pub const fn complex_selector(selector: ComplexSelector) -> Self {
        Self::Complex(selector)
    }

    #[must_use]
    pub const fn list(list: SelectorList) -> Self {
        Self::List(list)
    }

    pub fn matches<T: Tree>(&self, tree: &T, id: T::Id, traversal: Traversal) -> Result<bool> {
        self.matches_with_context(tree, SelectorMatchContext::new(id, traversal))
    }

    pub fn matches_with_context<T: Tree>(
        &self,
        tree: &T,
        context: SelectorMatchContext<T::Id>,
    ) -> Result<bool> {
        let id = context.subject();
        let traversal = context.traversal();
        match self {
            Self::Any => Ok(true),
            Self::Tag(tag) => Ok(tree.node(id)?.tag.as_ref() == Some(tag)),
            Self::Class(class) => Ok(tree.node(id)?.classes.contains(class)),
            Self::Key(key) => Ok(tree.node(id)?.key.as_ref() == Some(key)),
            Self::State(state) => runtime_state_matches(tree, id, *state),
            Self::Attribute(attribute) => attribute.matches(tree, id),
            Self::Position(position) => position.matches(tree, id, traversal),
            Self::Pseudo(pseudo_class) => pseudo_class.matches(tree, context),
            Self::Compound(compound) => compound.matches_with_context(tree, context),
            Self::Complex(complex) => complex.matches_with_context(tree, context),
            Self::List(list) => list.matches(tree, context),
        }
    }

    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        match self {
            Self::Any => SelectorSpecificity::zero(),
            Self::Tag(_) => SelectorSpecificity::new(0, 0, 1),
            Self::Class(_) | Self::State(_) | Self::Attribute(_) | Self::Position(_) => {
                SelectorSpecificity::new(0, 1, 0)
            }
            Self::Pseudo(pseudo_class) => pseudo_class.specificity(),
            Self::Key(_) => SelectorSpecificity::new(1, 0, 0),
            Self::Compound(compound) => compound.specificity(),
            Self::Complex(complex) => complex.specificity(),
            Self::List(list) => list.max_specificity(),
        }
    }

    pub(crate) fn primary_key(&self) -> PrimaryKey {
        match self {
            Self::Tag(tag) => PrimaryKey::Tag(tag.clone()),
            Self::Class(class) => PrimaryKey::Class(class.clone()),
            Self::Key(key) => PrimaryKey::Key(key.clone()),
            Self::Compound(compound) => compound.primary_key(),
            Self::Complex(complex) => complex
                .parts()
                .last()
                .map(|part| part.selector().primary_key())
                .unwrap_or(PrimaryKey::Universal),
            Self::Any
            | Self::State(_)
            | Self::Attribute(_)
            | Self::Position(_)
            | Self::Pseudo(_)
            | Self::List(_) => PrimaryKey::Universal,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorList {
    selectors: Vec<Selector>,
}

impl SelectorList {
    pub fn try_new(selectors: impl IntoIterator<Item = Selector>) -> Result<Self> {
        let selectors: Vec<_> = selectors.into_iter().collect();
        if selectors.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidSelector,
                "selector list must not be empty",
            ));
        }
        Ok(Self { selectors })
    }

    #[must_use]
    pub fn selectors(&self) -> &[Selector] {
        &self.selectors
    }

    pub fn matches<T: Tree>(&self, tree: &T, context: SelectorMatchContext<T::Id>) -> Result<bool> {
        for selector in &self.selectors {
            if selector.matches_with_context(tree, context)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[must_use]
    pub fn max_specificity(&self) -> SelectorSpecificity {
        self.selectors
            .iter()
            .map(Selector::specificity)
            .max()
            .unwrap_or_else(SelectorSpecificity::zero)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectorListPseudoClass {
    Not(SelectorList),
    Is(SelectorList),
    Where(SelectorList),
}

impl SelectorListPseudoClass {
    pub fn matches<T: Tree>(&self, tree: &T, context: SelectorMatchContext<T::Id>) -> Result<bool> {
        match self {
            Self::Not(list) => Ok(!list.matches(tree, context)?),
            Self::Is(list) | Self::Where(list) => list.matches(tree, context),
        }
    }

    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        match self {
            Self::Not(list) | Self::Is(list) => list.max_specificity(),
            Self::Where(_) => SelectorSpecificity::zero(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelativeSelector {
    combinator: Combinator,
    selector: Box<Selector>,
}

impl RelativeSelector {
    #[must_use]
    pub fn new(combinator: Combinator, selector: Selector) -> Self {
        Self {
            combinator,
            selector: Box::new(selector),
        }
    }

    #[must_use]
    pub const fn combinator(&self) -> Combinator {
        self.combinator
    }

    #[must_use]
    pub fn selector(&self) -> &Selector {
        &self.selector
    }

    pub fn matches<T: Tree>(&self, tree: &T, context: SelectorMatchContext<T::Id>) -> Result<bool> {
        for candidate in relative_candidates(self.combinator, tree, context)? {
            if relative_selector_matches_at_candidate(self.selector(), tree, context, candidate)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        self.selector.specificity()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelativeSelectorList {
    selectors: Vec<RelativeSelector>,
}

impl RelativeSelectorList {
    pub fn try_new(selectors: impl IntoIterator<Item = RelativeSelector>) -> Result<Self> {
        let selectors: Vec<_> = selectors.into_iter().collect();
        if selectors.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidSelector,
                "relative selector list must not be empty",
            ));
        }
        Ok(Self { selectors })
    }

    #[must_use]
    pub fn selectors(&self) -> &[RelativeSelector] {
        &self.selectors
    }

    pub fn matches<T: Tree>(&self, tree: &T, context: SelectorMatchContext<T::Id>) -> Result<bool> {
        for selector in &self.selectors {
            if selector.matches(tree, context)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[must_use]
    pub fn max_specificity(&self) -> SelectorSpecificity {
        self.selectors
            .iter()
            .map(RelativeSelector::specificity)
            .max()
            .unwrap_or_else(SelectorSpecificity::zero)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectorMatchContext<Id> {
    subject: Id,
    traversal: Traversal,
    root: Option<Id>,
    scope: Option<Id>,
}

impl<Id: Copy> SelectorMatchContext<Id> {
    #[must_use]
    pub const fn new(subject: Id, traversal: Traversal) -> Self {
        Self {
            subject,
            traversal,
            root: None,
            scope: None,
        }
    }

    #[must_use]
    pub const fn for_subject(subject: Id) -> Self {
        Self::new(subject, Traversal::Canonical)
    }

    #[must_use]
    pub const fn with_root(mut self, root: Id) -> Self {
        self.root = Some(root);
        self
    }

    #[must_use]
    pub const fn with_scope(mut self, scope: Id) -> Self {
        self.scope = Some(scope);
        self
    }

    #[must_use]
    pub const fn with_subject(mut self, subject: Id) -> Self {
        self.subject = subject;
        self
    }

    #[must_use]
    pub const fn subject(self) -> Id {
        self.subject
    }

    #[must_use]
    pub const fn traversal(self) -> Traversal {
        self.traversal
    }

    #[must_use]
    pub const fn root(self) -> Option<Id> {
        self.root
    }

    #[must_use]
    pub const fn scope(self) -> Option<Id> {
        self.scope
    }
}

fn validate_complex_parts(parts: &[ComplexSelectorPart]) -> Result<()> {
    let Some((first, rest)) = parts.split_first() else {
        return Err(Error::new(
            ErrorCode::InvalidSelector,
            "complex selector must contain at least one part",
        ));
    };
    if first.combinator.is_some() {
        return Err(Error::new(
            ErrorCode::InvalidSelector,
            "complex selector must start with a root part",
        ));
    }
    for (index, part) in rest.iter().enumerate() {
        if part.combinator.is_none() {
            return Err(Error::new(
                ErrorCode::InvalidSelector,
                format!("complex selector part {} must be related", index + 1),
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Compound {
    tag: Option<StyleTag>,
    key: Option<StyleKey>,
    classes: Vec<StyleClass>,
    states: Vec<StateFlag>,
    attributes: Vec<AttributeSelector>,
    pseudo_classes: Vec<PseudoClassSelector>,
    position: Option<PositionSelector>,
    scope_anchor: bool,
}

impl Compound {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tag(mut self, tag: impl AsRef<str>) -> Result<Self> {
        self.tag = Some(tag_from_str(tag.as_ref())?);
        Ok(self)
    }

    pub fn key(mut self, key: impl AsRef<str>) -> Result<Self> {
        self.key = Some(key_from_str(key.as_ref())?);
        Ok(self)
    }

    pub fn class(mut self, class: impl AsRef<str>) -> Result<Self> {
        self.classes.push(class_from_str(class.as_ref())?);
        Ok(self)
    }

    #[must_use]
    pub fn state(mut self, state: StateFlag) -> Self {
        self.states.push(state);
        self
    }

    pub fn attribute_exists(mut self, name: impl AsRef<str>) -> Result<Self> {
        self.attributes.push(AttributeSelector::exists(name)?);
        Ok(self)
    }

    pub fn attribute_equals(
        mut self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<Self> {
        self.attributes
            .push(AttributeSelector::equals(name, value)?);
        Ok(self)
    }

    #[must_use]
    pub const fn position(mut self, position: PositionSelector) -> Self {
        self.position = Some(position);
        self
    }

    #[must_use]
    pub const fn scope_anchor(mut self) -> Self {
        self.scope_anchor = true;
        self
    }

    #[must_use]
    pub fn pseudo(mut self, pseudo_class: PseudoClassSelector) -> Self {
        self.pseudo_classes.push(pseudo_class);
        self
    }

    #[must_use]
    pub fn runtime_pseudo(mut self, pseudo_class: RuntimePseudoClass) -> Self {
        self.pseudo_classes
            .push(PseudoClassSelector::runtime(pseudo_class));
        self
    }

    #[must_use]
    pub fn pseudo_classes(&self) -> &[PseudoClassSelector] {
        &self.pseudo_classes
    }

    #[must_use]
    pub fn selector(self) -> Selector {
        Selector::Compound(self)
    }

    pub fn matches<T: Tree>(&self, tree: &T, id: T::Id, traversal: Traversal) -> Result<bool> {
        self.matches_with_context(tree, SelectorMatchContext::new(id, traversal))
    }

    pub fn matches_with_context<T: Tree>(
        &self,
        tree: &T,
        context: SelectorMatchContext<T::Id>,
    ) -> Result<bool> {
        let id = context.subject();
        let traversal = context.traversal();
        if self.scope_anchor && !matches_scope(context) {
            return Ok(false);
        }
        let node = tree.node(id)?;
        if self
            .tag
            .as_ref()
            .is_some_and(|tag| node.tag.as_ref() != Some(tag))
        {
            return Ok(false);
        }
        if self
            .key
            .as_ref()
            .is_some_and(|key| node.key.as_ref() != Some(key))
        {
            return Ok(false);
        }
        if !self
            .classes
            .iter()
            .all(|class| node.classes.contains(class))
        {
            return Ok(false);
        }
        for state in &self.states {
            if !runtime_state_matches(tree, id, *state)? {
                return Ok(false);
            }
        }
        for attribute in &self.attributes {
            if !attribute.matches(tree, id)? {
                return Ok(false);
            }
        }
        if let Some(position) = self.position
            && !position.matches(tree, id, traversal)?
        {
            return Ok(false);
        }
        for pseudo_class in &self.pseudo_classes {
            if !pseudo_class.matches(tree, context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) fn primary_key(&self) -> PrimaryKey {
        if let Some(key) = &self.key {
            PrimaryKey::Key(key.clone())
        } else if let Some(class) = self.classes.last() {
            PrimaryKey::Class(class.clone())
        } else if let Some(tag) = &self.tag {
            PrimaryKey::Tag(tag.clone())
        } else {
            PrimaryKey::Universal
        }
    }

    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        let mut specificity = SelectorSpecificity::zero();
        if self.key.is_some() {
            specificity = specificity.saturating_add(SelectorSpecificity::new(1, 0, 0));
        }
        if self.tag.is_some() {
            specificity = specificity.saturating_add(SelectorSpecificity::new(0, 0, 1));
        }
        for _ in &self.classes {
            specificity = specificity.saturating_add(SelectorSpecificity::new(0, 1, 0));
        }
        for _ in &self.states {
            specificity = specificity.saturating_add(SelectorSpecificity::new(0, 1, 0));
        }
        for _ in &self.attributes {
            specificity = specificity.saturating_add(SelectorSpecificity::new(0, 1, 0));
        }
        for pseudo_class in &self.pseudo_classes {
            specificity = specificity.saturating_add(pseudo_class.specificity());
        }
        if self.position.is_some() {
            specificity = specificity.saturating_add(SelectorSpecificity::new(0, 1, 0));
        }
        specificity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplexSelector {
    parts: Vec<ComplexSelectorPart>,
}

impl ComplexSelector {
    pub fn try_new(parts: impl IntoIterator<Item = ComplexSelectorPart>) -> Result<Self> {
        let parts: Vec<_> = parts.into_iter().collect();
        validate_complex_parts(&parts)?;
        Ok(Self { parts })
    }

    #[must_use]
    pub fn parts(&self) -> &[ComplexSelectorPart] {
        &self.parts
    }

    pub fn matches<T: Tree>(&self, tree: &T, id: T::Id, traversal: Traversal) -> Result<bool> {
        self.matches_with_context(tree, SelectorMatchContext::new(id, traversal))
    }

    pub fn matches_with_context<T: Tree>(
        &self,
        tree: &T,
        context: SelectorMatchContext<T::Id>,
    ) -> Result<bool> {
        complex_matches(&self.parts, tree, context)
    }

    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        self.parts
            .iter()
            .map(|part| part.selector.specificity())
            .fold(
                SelectorSpecificity::zero(),
                SelectorSpecificity::saturating_add,
            )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplexSelectorPart {
    combinator: Option<Combinator>,
    selector: Compound,
}

impl ComplexSelectorPart {
    #[must_use]
    pub const fn root(selector: Compound) -> Self {
        Self {
            combinator: None,
            selector,
        }
    }

    #[must_use]
    pub const fn related(combinator: Combinator, selector: Compound) -> Self {
        Self {
            combinator: Some(combinator),
            selector,
        }
    }

    #[must_use]
    pub const fn combinator(&self) -> Option<Combinator> {
        self.combinator
    }

    #[must_use]
    pub const fn selector(&self) -> &Compound {
        &self.selector
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Combinator {
    Descendant,
    Child,
    Adjacent,
    Sibling,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PseudoClassSelector {
    Root,
    Scope,
    Runtime(RuntimePseudoClass),
    Structural(StructuralSelector),
    SelectorList(SelectorListPseudoClass),
    Has(RelativeSelectorList),
}

impl PseudoClassSelector {
    #[must_use]
    pub const fn runtime(pseudo_class: RuntimePseudoClass) -> Self {
        Self::Runtime(pseudo_class)
    }

    #[must_use]
    pub const fn structural(selector: StructuralSelector) -> Self {
        Self::Structural(selector)
    }

    #[must_use]
    pub const fn selector_list(selector: SelectorListPseudoClass) -> Self {
        Self::SelectorList(selector)
    }

    #[must_use]
    pub const fn has(selectors: RelativeSelectorList) -> Self {
        Self::Has(selectors)
    }

    pub fn matches<T: Tree>(&self, tree: &T, context: SelectorMatchContext<T::Id>) -> Result<bool> {
        let id = context.subject();
        match self {
            Self::Root => matches_root(tree, context),
            Self::Scope => Ok(matches_scope(context)),
            Self::Runtime(pseudo_class) => {
                runtime_state_matches(tree, id, pseudo_class.state_flag())
            }
            Self::Structural(selector) => selector.matches(tree, context),
            Self::SelectorList(selector) => selector.matches(tree, context),
            Self::Has(selectors) => selectors.matches(tree, context),
        }
    }

    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        match self {
            Self::Root | Self::Scope => SelectorSpecificity::new(0, 1, 0),
            Self::Runtime(_) => SelectorSpecificity::new(0, 1, 0),
            Self::Structural(selector) => selector.specificity(),
            Self::SelectorList(selector) => selector.specificity(),
            Self::Has(selectors) => selectors.max_specificity(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructuralSelector {
    FirstChild,
    LastChild,
    OnlyChild,
    Empty,
    FirstOfType,
    LastOfType,
    OnlyOfType,
    NthChild(NthSelector),
    NthLastChild(NthSelector),
    NthOfType(NthPattern),
    NthLastOfType(NthPattern),
}

impl StructuralSelector {
    pub fn matches<T: Tree>(&self, tree: &T, context: SelectorMatchContext<T::Id>) -> Result<bool> {
        let id = context.subject();
        let traversal = context.traversal();
        Ok(match self {
            Self::FirstChild => child_position(tree, context, None)?
                .is_some_and(|position| NthPattern::integer(1).matches(position)),
            Self::LastChild => reverse_child_position(tree, context, None)?
                .is_some_and(|position| NthPattern::integer(1).matches(position)),
            Self::OnlyChild => child_position(tree, context, None)?
                .zip(reverse_child_position(tree, context, None)?)
                .is_some_and(|(from_start, from_end)| from_start == 1 && from_end == 1),
            Self::Empty => node_is_empty(tree, id, traversal)?,
            Self::FirstOfType => type_position(tree, id, traversal)?
                .is_some_and(|position| NthPattern::integer(1).matches(position)),
            Self::LastOfType => reverse_type_position(tree, id, traversal)?
                .is_some_and(|position| NthPattern::integer(1).matches(position)),
            Self::OnlyOfType => type_position(tree, id, traversal)?
                .zip(reverse_type_position(tree, id, traversal)?)
                .is_some_and(|(from_start, from_end)| from_start == 1 && from_end == 1),
            Self::NthChild(selector) => child_position(tree, context, selector.filter())?
                .is_some_and(|position| selector.pattern().matches(position)),
            Self::NthLastChild(selector) => {
                reverse_child_position(tree, context, selector.filter())?
                    .is_some_and(|position| selector.pattern().matches(position))
            }
            Self::NthOfType(pattern) => type_position(tree, id, traversal)?
                .is_some_and(|position| pattern.matches(position)),
            Self::NthLastOfType(pattern) => reverse_type_position(tree, id, traversal)?
                .is_some_and(|position| pattern.matches(position)),
        })
    }

    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        let base = SelectorSpecificity::new(0, 1, 0);
        match self {
            Self::NthChild(selector) | Self::NthLastChild(selector) => {
                base.saturating_add(selector.filter_specificity())
            }
            Self::FirstChild
            | Self::LastChild
            | Self::OnlyChild
            | Self::Empty
            | Self::FirstOfType
            | Self::LastOfType
            | Self::OnlyOfType
            | Self::NthOfType(_)
            | Self::NthLastOfType(_) => base,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuntimePseudoClass {
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

impl RuntimePseudoClass {
    #[must_use]
    pub const fn state_flag(self) -> StateFlag {
        match self {
            Self::Hover => StateFlag::Hovered,
            Self::Active => StateFlag::Active,
            Self::Focus => StateFlag::Focused,
            Self::FocusVisible => StateFlag::FocusVisible,
            Self::FocusWithin => StateFlag::FocusWithin,
            Self::Disabled => StateFlag::Disabled,
            Self::Enabled => StateFlag::Enabled,
            Self::Checked => StateFlag::Checked,
            Self::Required => StateFlag::Required,
            Self::Optional => StateFlag::Optional,
            Self::Valid => StateFlag::Valid,
            Self::Invalid => StateFlag::Invalid,
            Self::PlaceholderShown => StateFlag::PlaceholderShown,
            Self::Modal => StateFlag::Modal,
            Self::Fullscreen => StateFlag::Fullscreen,
            Self::PopoverOpen => StateFlag::PopoverOpen,
            Self::Default => StateFlag::Default,
            Self::Indeterminate => StateFlag::Indeterminate,
            Self::ReadOnly => StateFlag::ReadOnly,
            Self::ReadWrite => StateFlag::ReadWrite,
            Self::InRange => StateFlag::InRange,
            Self::OutOfRange => StateFlag::OutOfRange,
        }
    }
}

fn runtime_state_matches<T: Tree>(tree: &T, id: T::Id, flag: StateFlag) -> Result<bool> {
    Ok(tree.node(id)?.has_state(flag))
}

fn matches_root<T: Tree>(tree: &T, context: SelectorMatchContext<T::Id>) -> Result<bool> {
    let id = context.subject();
    if let Some(root) = context.root() {
        return Ok(id == root);
    }
    let mut root = id;
    while let Some(parent) = tree.parent(root, context.traversal())? {
        root = parent;
    }
    Ok(id == root)
}

fn matches_scope<Id: Copy + Eq>(context: SelectorMatchContext<Id>) -> bool {
    let id = context.subject();
    context.scope().is_none_or(|scope| id == scope)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributeSelector {
    Exists {
        name: StyleAttributeName,
    },
    Matcher {
        name: StyleAttributeName,
        matcher: AttributeMatcher,
        case_sensitivity: AttributeCaseSensitivity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributeMatcher {
    Equals(StyleAttributeValue),
    Includes(StyleAttributeValue),
    DashMatch(StyleAttributeValue),
    Prefix(StyleAttributeValue),
    Suffix(StyleAttributeValue),
    Substring(StyleAttributeValue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AttributeCaseSensitivity {
    DocumentDefault,
    AsciiCaseInsensitive,
    ExplicitSensitive,
}

impl AttributeSelector {
    pub fn exists(name: impl AsRef<str>) -> Result<Self> {
        Ok(Self::Exists {
            name: attribute_name_from_str(name.as_ref())?,
        })
    }

    pub fn equals(name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        Self::equals_with_case(name, value, AttributeCaseSensitivity::DocumentDefault)
    }

    pub fn includes(name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        Self::matcher(
            name,
            AttributeMatcher::Includes(attribute_value_from_str(value.as_ref())?),
        )
    }

    pub fn dash_match(name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        Self::matcher(
            name,
            AttributeMatcher::DashMatch(attribute_value_from_str(value.as_ref())?),
        )
    }

    pub fn prefix(name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        Self::matcher(
            name,
            AttributeMatcher::Prefix(attribute_value_from_str(value.as_ref())?),
        )
    }

    pub fn suffix(name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        Self::matcher(
            name,
            AttributeMatcher::Suffix(attribute_value_from_str(value.as_ref())?),
        )
    }

    pub fn substring(name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        Self::matcher(
            name,
            AttributeMatcher::Substring(attribute_value_from_str(value.as_ref())?),
        )
    }

    pub fn equals_with_case(
        name: impl AsRef<str>,
        value: impl AsRef<str>,
        case_sensitivity: AttributeCaseSensitivity,
    ) -> Result<Self> {
        Ok(Self::Matcher {
            name: attribute_name_from_str(name.as_ref())?,
            matcher: AttributeMatcher::Equals(attribute_value_from_str(value.as_ref())?),
            case_sensitivity,
        })
    }

    fn matcher(name: impl AsRef<str>, matcher: AttributeMatcher) -> Result<Self> {
        Ok(Self::Matcher {
            name: attribute_name_from_str(name.as_ref())?,
            matcher,
            case_sensitivity: AttributeCaseSensitivity::DocumentDefault,
        })
    }

    pub fn matcher_with_case(
        name: impl AsRef<str>,
        matcher: AttributeMatcher,
        case_sensitivity: AttributeCaseSensitivity,
    ) -> Result<Self> {
        Ok(Self::Matcher {
            name: attribute_name_from_str(name.as_ref())?,
            matcher,
            case_sensitivity,
        })
    }

    pub fn matches<T: Tree>(&self, tree: &T, id: T::Id) -> Result<bool> {
        let node = tree.node(id)?;
        Ok(match self {
            Self::Exists { name } => node
                .attributes
                .iter()
                .any(|attribute| attribute.name() == name),
            Self::Matcher {
                name,
                matcher,
                case_sensitivity,
            } => node.attributes.iter().any(|attribute| {
                attribute.name() == name
                    && attribute_matcher_matches(attribute.value(), matcher, *case_sensitivity)
            }),
        })
    }
}

fn attribute_matcher_matches(
    actual: &StyleAttributeValue,
    matcher: &AttributeMatcher,
    case_sensitivity: AttributeCaseSensitivity,
) -> bool {
    match matcher {
        AttributeMatcher::Equals(expected) => {
            compare_attribute_value(actual, expected, case_sensitivity)
        }
        AttributeMatcher::Includes(expected) => {
            let expected = expected.as_str();
            !expected.is_empty()
                && actual
                    .as_str()
                    .split_ascii_whitespace()
                    .any(|token| compare_attribute_str(token, expected, case_sensitivity))
        }
        AttributeMatcher::DashMatch(expected) => {
            compare_attribute_value(actual, expected, case_sensitivity)
                || attribute_starts_with(actual.as_str(), expected.as_str(), case_sensitivity)
                    && actual
                        .as_str()
                        .as_bytes()
                        .get(expected.as_str().len())
                        .is_some_and(|byte| *byte == b'-')
        }
        AttributeMatcher::Prefix(expected) => {
            let expected = expected.as_str();
            !expected.is_empty()
                && attribute_starts_with(actual.as_str(), expected, case_sensitivity)
        }
        AttributeMatcher::Suffix(expected) => {
            let expected = expected.as_str();
            !expected.is_empty() && attribute_ends_with(actual.as_str(), expected, case_sensitivity)
        }
        AttributeMatcher::Substring(expected) => {
            let expected = expected.as_str();
            !expected.is_empty() && attribute_contains(actual.as_str(), expected, case_sensitivity)
        }
    }
}

fn compare_attribute_value(
    actual: &StyleAttributeValue,
    expected: &StyleAttributeValue,
    case_sensitivity: AttributeCaseSensitivity,
) -> bool {
    compare_attribute_str(actual.as_str(), expected.as_str(), case_sensitivity)
}

fn compare_attribute_str(
    actual: &str,
    expected: &str,
    case_sensitivity: AttributeCaseSensitivity,
) -> bool {
    match case_sensitivity {
        AttributeCaseSensitivity::DocumentDefault | AttributeCaseSensitivity::ExplicitSensitive => {
            actual == expected
        }
        AttributeCaseSensitivity::AsciiCaseInsensitive => actual.eq_ignore_ascii_case(expected),
    }
}

fn attribute_starts_with(
    actual: &str,
    expected: &str,
    case_sensitivity: AttributeCaseSensitivity,
) -> bool {
    match case_sensitivity {
        AttributeCaseSensitivity::DocumentDefault | AttributeCaseSensitivity::ExplicitSensitive => {
            actual.starts_with(expected)
        }
        AttributeCaseSensitivity::AsciiCaseInsensitive => actual
            .to_ascii_lowercase()
            .starts_with(&expected.to_ascii_lowercase()),
    }
}

fn attribute_ends_with(
    actual: &str,
    expected: &str,
    case_sensitivity: AttributeCaseSensitivity,
) -> bool {
    match case_sensitivity {
        AttributeCaseSensitivity::DocumentDefault | AttributeCaseSensitivity::ExplicitSensitive => {
            actual.ends_with(expected)
        }
        AttributeCaseSensitivity::AsciiCaseInsensitive => actual
            .to_ascii_lowercase()
            .ends_with(&expected.to_ascii_lowercase()),
    }
}

fn attribute_contains(
    actual: &str,
    expected: &str,
    case_sensitivity: AttributeCaseSensitivity,
) -> bool {
    match case_sensitivity {
        AttributeCaseSensitivity::DocumentDefault | AttributeCaseSensitivity::ExplicitSensitive => {
            actual.contains(expected)
        }
        AttributeCaseSensitivity::AsciiCaseInsensitive => actual
            .to_ascii_lowercase()
            .contains(&expected.to_ascii_lowercase()),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionSelector {
    First,
    Last,
    Nth(Nth),
}

impl PositionSelector {
    pub fn matches<T: Tree>(&self, tree: &T, id: T::Id, traversal: Traversal) -> Result<bool> {
        let Some(parent) = tree.parent(id, traversal)? else {
            return Ok(false);
        };
        let children: Vec<_> = tree.children(parent, traversal)?.collect();
        let Some(index) = children.iter().position(|child| *child == id) else {
            return Ok(false);
        };
        let position = Position::new(index, children.len());
        Ok(match self {
            Self::First => position.is_first(),
            Self::Last => position.is_last(),
            Self::Nth(nth) => position.matches(*nth),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub index: usize,
    pub sibling_count: usize,
}

impl Position {
    #[must_use]
    pub const fn new(index: usize, sibling_count: usize) -> Self {
        Self {
            index,
            sibling_count,
        }
    }

    #[must_use]
    pub const fn is_first(self) -> bool {
        self.index == 0
    }

    #[must_use]
    pub const fn is_last(self) -> bool {
        self.index + 1 == self.sibling_count
    }

    #[must_use]
    pub fn matches(self, nth: Nth) -> bool {
        nth.matches(self.index + 1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Nth {
    pub step: usize,
    pub offset: usize,
}

impl Nth {
    #[must_use]
    pub const fn new(step: usize, offset: usize) -> Self {
        Self { step, offset }
    }

    #[must_use]
    pub const fn odd() -> Self {
        Self::new(2, 1)
    }

    #[must_use]
    pub const fn even() -> Self {
        Self::new(2, 0)
    }

    #[must_use]
    pub fn matches(self, position: usize) -> bool {
        let step = i32::try_from(self.step).unwrap_or(i32::MAX);
        let offset = i32::try_from(self.offset).unwrap_or(i32::MAX);
        NthPattern::new(step, offset).matches(position)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NthPattern {
    a: i32,
    b: i32,
}

impl NthPattern {
    #[must_use]
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    #[must_use]
    pub const fn odd() -> Self {
        Self::new(2, 1)
    }

    #[must_use]
    pub const fn even() -> Self {
        Self::new(2, 0)
    }

    #[must_use]
    pub const fn integer(position: i32) -> Self {
        Self::new(0, position)
    }

    #[must_use]
    pub fn matches(self, one_based_position: usize) -> bool {
        let position = i32::try_from(one_based_position).unwrap_or(i32::MAX);
        if position <= 0 {
            return false;
        }
        if self.a == 0 {
            return position == self.b;
        }
        let delta = position - self.b;
        if self.a > 0 {
            delta >= 0 && delta % self.a == 0
        } else {
            delta <= 0 && delta % self.a == 0
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NthSelector {
    pattern: NthPattern,
    filter: Option<SelectorList>,
}

impl NthSelector {
    #[must_use]
    pub const fn new(pattern: NthPattern, filter: Option<SelectorList>) -> Self {
        Self { pattern, filter }
    }

    #[must_use]
    pub const fn pattern(&self) -> NthPattern {
        self.pattern
    }

    #[must_use]
    pub const fn filter(&self) -> Option<&SelectorList> {
        self.filter.as_ref()
    }

    #[must_use]
    fn filter_specificity(&self) -> SelectorSpecificity {
        self.filter()
            .map_or_else(SelectorSpecificity::zero, SelectorList::max_specificity)
    }
}

fn child_position<T: Tree>(
    tree: &T,
    context: SelectorMatchContext<T::Id>,
    filter: Option<&SelectorList>,
) -> Result<Option<usize>> {
    let id = context.subject();
    let traversal = context.traversal();
    let Some(parent) = tree.parent(id, traversal)? else {
        return Ok(None);
    };
    position_in_siblings(tree, context, tree.children(parent, traversal)?, filter)
}

fn reverse_child_position<T: Tree>(
    tree: &T,
    context: SelectorMatchContext<T::Id>,
    filter: Option<&SelectorList>,
) -> Result<Option<usize>> {
    let id = context.subject();
    let traversal = context.traversal();
    let Some(parent) = tree.parent(id, traversal)? else {
        return Ok(None);
    };
    reverse_position_in_siblings(tree, context, tree.children(parent, traversal)?, filter)
}

fn position_in_siblings<T: Tree>(
    tree: &T,
    context: SelectorMatchContext<T::Id>,
    siblings: impl IntoIterator<Item = T::Id>,
    filter: Option<&SelectorList>,
) -> Result<Option<usize>> {
    let id = context.subject();
    let mut position = 0;
    for sibling in siblings {
        if sibling_matches_filter(tree, context.with_subject(sibling), filter)? {
            position += 1;
            if sibling == id {
                return Ok(Some(position));
            }
        }
    }
    Ok(None)
}

fn reverse_position_in_siblings<T: Tree>(
    tree: &T,
    context: SelectorMatchContext<T::Id>,
    siblings: impl IntoIterator<Item = T::Id>,
    filter: Option<&SelectorList>,
) -> Result<Option<usize>> {
    let id = context.subject();
    let mut filtered = Vec::new();
    for sibling in siblings {
        if sibling_matches_filter(tree, context.with_subject(sibling), filter)? {
            filtered.push(sibling);
        }
    }
    Ok(filtered
        .iter()
        .rev()
        .position(|sibling| *sibling == id)
        .map(|index| index + 1))
}

fn sibling_matches_filter<T: Tree>(
    tree: &T,
    context: SelectorMatchContext<T::Id>,
    filter: Option<&SelectorList>,
) -> Result<bool> {
    match filter {
        Some(filter) => filter.matches(tree, context),
        None => Ok(true),
    }
}

fn type_position<T: Tree>(tree: &T, id: T::Id, traversal: Traversal) -> Result<Option<usize>> {
    let Some(tag) = tree.node(id)?.tag else {
        return Ok(None);
    };
    let Some(parent) = tree.parent(id, traversal)? else {
        return Ok(None);
    };
    let mut position = 0;
    for sibling in tree.children(parent, traversal)? {
        if tree.node(sibling)?.tag.as_ref() == Some(&tag) {
            position += 1;
            if sibling == id {
                return Ok(Some(position));
            }
        }
    }
    Ok(None)
}

fn reverse_type_position<T: Tree>(
    tree: &T,
    id: T::Id,
    traversal: Traversal,
) -> Result<Option<usize>> {
    let Some(tag) = tree.node(id)?.tag else {
        return Ok(None);
    };
    let Some(parent) = tree.parent(id, traversal)? else {
        return Ok(None);
    };
    let siblings: Vec<_> = tree.children(parent, traversal)?.collect();
    let mut position = 0;
    for sibling in siblings.into_iter().rev() {
        if tree.node(sibling)?.tag.as_ref() == Some(&tag) {
            position += 1;
            if sibling == id {
                return Ok(Some(position));
            }
        }
    }
    Ok(None)
}

fn node_is_empty<T: Tree>(tree: &T, id: T::Id, traversal: Traversal) -> Result<bool> {
    let node = tree.node(id)?;
    if node.text {
        return Ok(false);
    }
    Ok(tree.children(id, traversal)?.next().is_none())
}

fn complex_matches<T: Tree>(
    parts: &[ComplexSelectorPart],
    tree: &T,
    context: SelectorMatchContext<T::Id>,
) -> Result<bool> {
    let Some(last) = parts.last() else {
        return Ok(false);
    };
    let id = context.subject();
    if !last.selector.matches_with_context(tree, context)? {
        return Ok(false);
    }

    complex_prefix_matches(parts, parts.len() - 1, tree, context.with_subject(id))
}

fn complex_prefix_matches<T: Tree>(
    parts: &[ComplexSelectorPart],
    index: usize,
    tree: &T,
    context: SelectorMatchContext<T::Id>,
) -> Result<bool> {
    if index == 0 {
        return Ok(true);
    }
    let combinator = parts[index].combinator.ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidSelector,
            "complex selector part is missing a combinator",
        )
    })?;
    for candidate in related_candidates(combinator, &parts[index - 1].selector, tree, context)? {
        if complex_prefix_matches(parts, index - 1, tree, context.with_subject(candidate))? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn related_candidates<T: Tree>(
    combinator: Combinator,
    selector: &Compound,
    tree: &T,
    context: SelectorMatchContext<T::Id>,
) -> Result<Vec<T::Id>> {
    let id = context.subject();
    let traversal = context.traversal();
    match combinator {
        Combinator::Child => {
            let Some(parent) = tree.parent(id, traversal)? else {
                return Ok(Vec::new());
            };
            if selector.matches_with_context(tree, context.with_subject(parent))? {
                Ok(vec![parent])
            } else {
                Ok(Vec::new())
            }
        }
        Combinator::Descendant => {
            let mut parent = tree.parent(id, traversal)?;
            let mut candidates = Vec::new();
            while let Some(candidate) = parent {
                if selector.matches_with_context(tree, context.with_subject(candidate))? {
                    candidates.push(candidate);
                }
                parent = tree.parent(candidate, traversal)?;
            }
            Ok(candidates)
        }
        Combinator::Adjacent => {
            let Some(previous) = tree.previous_sibling(id, traversal)? else {
                return Ok(Vec::new());
            };
            if selector.matches_with_context(tree, context.with_subject(previous))? {
                Ok(vec![previous])
            } else {
                Ok(Vec::new())
            }
        }
        Combinator::Sibling => {
            let Some(parent) = tree.parent(id, traversal)? else {
                return Ok(Vec::new());
            };
            let siblings: Vec<_> = tree.children(parent, traversal)?.collect();
            let Some(index) = siblings.iter().position(|sibling| *sibling == id) else {
                return Ok(Vec::new());
            };
            let mut candidates = Vec::new();
            for candidate in siblings[..index].iter().rev().copied() {
                if selector.matches_with_context(tree, context.with_subject(candidate))? {
                    candidates.push(candidate);
                }
            }
            Ok(candidates)
        }
    }
}

fn relative_selector_matches_at_candidate<T: Tree>(
    selector: &Selector,
    tree: &T,
    context: SelectorMatchContext<T::Id>,
    candidate: T::Id,
) -> Result<bool> {
    let candidate_context = context.with_subject(candidate);
    match selector {
        Selector::Complex(complex) => {
            complex_matches_from_anchor(complex.parts(), tree, candidate_context)
        }
        _ => selector.matches_with_context(tree, candidate_context),
    }
}

fn complex_matches_from_anchor<T: Tree>(
    parts: &[ComplexSelectorPart],
    tree: &T,
    context: SelectorMatchContext<T::Id>,
) -> Result<bool> {
    let Some(first) = parts.first() else {
        return Ok(false);
    };
    if !first.selector.matches_with_context(tree, context)? {
        return Ok(false);
    }
    complex_suffix_matches(parts, 1, tree, context)
}

fn complex_suffix_matches<T: Tree>(
    parts: &[ComplexSelectorPart],
    index: usize,
    tree: &T,
    context: SelectorMatchContext<T::Id>,
) -> Result<bool> {
    let Some(part) = parts.get(index) else {
        return Ok(true);
    };
    let combinator = part.combinator.ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidSelector,
            "complex selector part is missing a combinator",
        )
    })?;
    for candidate in forward_related_candidates(combinator, &part.selector, tree, context)? {
        if complex_suffix_matches(parts, index + 1, tree, context.with_subject(candidate))? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn forward_related_candidates<T: Tree>(
    combinator: Combinator,
    selector: &Compound,
    tree: &T,
    context: SelectorMatchContext<T::Id>,
) -> Result<Vec<T::Id>> {
    let mut candidates = Vec::new();
    for candidate in relative_candidates(combinator, tree, context)? {
        if selector.matches_with_context(tree, context.with_subject(candidate))? {
            candidates.push(candidate);
        }
    }
    Ok(candidates)
}

fn relative_candidates<T: Tree>(
    combinator: Combinator,
    tree: &T,
    context: SelectorMatchContext<T::Id>,
) -> Result<Vec<T::Id>> {
    let id = context.subject();
    let traversal = context.traversal();
    match combinator {
        Combinator::Child => Ok(tree.children(id, traversal)?.collect()),
        Combinator::Descendant => {
            let mut candidates = Vec::new();
            collect_descendants(tree, id, traversal, &mut candidates)?;
            Ok(candidates)
        }
        Combinator::Adjacent => Ok(next_sibling(tree, id, traversal)?.into_iter().collect()),
        Combinator::Sibling => {
            let Some(parent) = tree.parent(id, traversal)? else {
                return Ok(Vec::new());
            };
            let siblings: Vec<_> = tree.children(parent, traversal)?.collect();
            let Some(index) = siblings.iter().position(|sibling| *sibling == id) else {
                return Ok(Vec::new());
            };
            Ok(siblings[index + 1..].to_vec())
        }
    }
}

fn collect_descendants<T: Tree>(
    tree: &T,
    id: T::Id,
    traversal: Traversal,
    descendants: &mut Vec<T::Id>,
) -> Result<()> {
    let children: Vec<_> = tree.children(id, traversal)?.collect();
    for child in children {
        descendants.push(child);
        collect_descendants(tree, child, traversal, descendants)?;
    }
    Ok(())
}

fn next_sibling<T: Tree>(tree: &T, id: T::Id, traversal: Traversal) -> Result<Option<T::Id>> {
    let Some(parent) = tree.parent(id, traversal)? else {
        return Ok(None);
    };
    let children: Vec<_> = tree.children(parent, traversal)?.collect();
    Ok(children
        .iter()
        .position(|child| *child == id)
        .and_then(|index| children.get(index + 1))
        .copied())
}

fn tag_from_str(value: &str) -> Result<StyleTag> {
    StyleTag::new(value).map_err(|error| Error::new(ErrorCode::InvalidSelector, error.to_string()))
}

fn class_from_str(value: &str) -> Result<StyleClass> {
    StyleClass::new(value)
        .map_err(|error| Error::new(ErrorCode::InvalidSelector, error.to_string()))
}

fn key_from_str(value: &str) -> Result<StyleKey> {
    StyleKey::new(value).map_err(|error| Error::new(ErrorCode::InvalidSelector, error.to_string()))
}

fn attribute_name_from_str(value: &str) -> Result<StyleAttributeName> {
    StyleAttributeName::new(value)
        .map_err(|error| Error::new(ErrorCode::InvalidSelector, error.to_string()))
}

fn attribute_value_from_str(value: &str) -> Result<StyleAttributeValue> {
    StyleAttributeValue::new(value)
        .map_err(|error| Error::new(ErrorCode::InvalidSelector, error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ErrorCode, Node, Result, Sheet, StyleAttribute, StyleAttributeName, StyleAttributeValue,
        StyleClass, StyleKey, StyleRole, StyleState, StyleTag, Traversal, Tree,
    };

    #[test]
    fn selector_matches_style_owned_tree_facts() {
        let tree = TestTree::new(vec![
            TestNode::new(0)
                .tag("window")
                .children([1])
                .state(StyleState::default().with_focus_within(true)),
            TestNode::new(1)
                .tag("button")
                .key("primary")
                .class("accent")
                .attribute("data-mode", "submit")
                .state(StyleState::default().with_hovered(true)),
        ]);

        let selector = Selector::compound()
            .tag("button")
            .unwrap()
            .key("primary")
            .unwrap()
            .class("accent")
            .unwrap()
            .attribute_equals("data-mode", "submit")
            .unwrap()
            .state(crate::StateFlag::Hovered)
            .selector();

        assert!(
            selector
                .matches(&tree, 1, Traversal::Canonical)
                .expect("selector should evaluate")
        );
        assert_eq!(
            Selector::tag("bad name").unwrap_err().code(),
            ErrorCode::InvalidSelector
        );
    }

    #[test]
    fn sheet_candidates_use_style_owned_index_keys() {
        let tree = TestTree::new(vec![
            TestNode::new(0)
                .tag("button")
                .key("primary")
                .class("accent"),
        ]);
        let sheet = Sheet::new()
            .rule(Selector::tag("button").unwrap(), crate::Declarations::new())
            .rule(
                Selector::class("accent").unwrap(),
                crate::Declarations::new(),
            )
            .rule(
                Selector::key("primary").unwrap(),
                crate::Declarations::new(),
            );

        assert_eq!(sheet.candidate_rule_count(&tree, 0).unwrap(), 3);
    }

    #[test]
    fn selector_lists_and_has_rules_are_not_dropped_by_rule_index() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("section").children([1]),
            TestNode::new(1).tag("button").class("primary"),
        ]);
        let list = Selector::list(
            SelectorList::try_new([
                Selector::tag("label").unwrap(),
                Selector::class("primary").unwrap(),
            ])
            .unwrap(),
        );
        let has = Selector::pseudo(PseudoClassSelector::has(
            RelativeSelectorList::try_new([RelativeSelector::new(
                Combinator::Child,
                Selector::class("primary").unwrap(),
            )])
            .unwrap(),
        ));
        let list_sheet = Sheet::new().rule(list, crate::Declarations::new());
        let has_sheet = Sheet::new().rule(has, crate::Declarations::new());

        assert_eq!(list_sheet.candidate_rule_count(&tree, 1).unwrap(), 1);
        assert_eq!(has_sheet.candidate_rule_count(&tree, 0).unwrap(), 1);
    }

    #[test]
    fn selector_list_matches_any_selector_and_rejects_empty_lists() {
        let tree = TestTree::new(vec![TestNode::new(0).tag("button").class("primary")]);
        let list = SelectorList::try_new([
            Selector::tag("label").unwrap(),
            Selector::class("primary").unwrap(),
        ])
        .unwrap();

        assert!(
            list.matches(&tree, SelectorMatchContext::for_subject(0))
                .unwrap()
        );
        assert_eq!(
            SelectorList::try_new([]).unwrap_err().code(),
            ErrorCode::InvalidSelector
        );
    }

    #[test]
    fn selector_list_pseudo_classes_match_over_nested_selector_lists() {
        let tree = TestTree::new(vec![TestNode::new(0).tag("button").class("primary")]);
        let primary = SelectorList::try_new([Selector::class("primary").unwrap()]).unwrap();
        let danger = SelectorList::try_new([Selector::class("danger").unwrap()]).unwrap();

        assert!(
            Selector::pseudo(PseudoClassSelector::selector_list(
                SelectorListPseudoClass::Is(primary.clone()),
            ))
            .matches(&tree, 0, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::selector_list(
                SelectorListPseudoClass::Where(primary),
            ))
            .matches(&tree, 0, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::selector_list(
                SelectorListPseudoClass::Not(danger,)
            ))
            .matches(&tree, 0, Traversal::Canonical)
            .unwrap()
        );
    }

    #[test]
    fn compound_selectors_can_combine_class_and_selector_list_pseudo_classes() {
        let tree = TestTree::new(vec![TestNode::new(0).tag("button").class("item")]);
        let disabled = SelectorList::try_new([Selector::class("disabled").unwrap()]).unwrap();
        let selector = Selector::compound()
            .class("item")
            .unwrap()
            .pseudo(PseudoClassSelector::selector_list(
                SelectorListPseudoClass::Not(disabled),
            ))
            .selector();

        assert!(selector.matches(&tree, 0, Traversal::Canonical).unwrap());
        assert_eq!(selector.specificity(), SelectorSpecificity::new(0, 2, 0));
    }

    #[test]
    fn root_and_scope_pseudo_classes_use_match_context() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("root").children([1]),
            TestNode::new(1).tag("section").children([2]),
            TestNode::new(2).tag("button"),
        ]);

        let context = SelectorMatchContext::new(2, Traversal::Canonical)
            .with_root(0)
            .with_scope(1);

        assert!(
            Selector::pseudo(PseudoClassSelector::Root)
                .matches_with_context(&tree, context.with_subject(0))
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::Root)
                .matches_with_context(&tree, context.with_subject(1))
                .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::Scope)
                .matches_with_context(&tree, context.with_subject(1))
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::Scope)
                .matches_with_context(&tree, context.with_subject(2))
                .unwrap()
        );
    }

    #[test]
    fn root_and_scope_pseudo_classes_use_fallbacks_without_match_context() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("root").children([1]),
            TestNode::new(1).tag("section").children([2]),
            TestNode::new(2).tag("button"),
        ]);
        let root = Selector::pseudo(PseudoClassSelector::Root);
        let scope = Selector::pseudo(PseudoClassSelector::Scope);

        assert!(root.matches(&tree, 0, Traversal::Canonical).unwrap());
        assert!(!root.matches(&tree, 1, Traversal::Canonical).unwrap());
        assert!(scope.matches(&tree, 2, Traversal::Canonical).unwrap());
    }

    #[test]
    fn compound_scope_anchor_matches_scope_node() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("root").children([1]),
            TestNode::new(1).tag("section").class("scope"),
        ]);
        let selector = Selector::compound()
            .scope_anchor()
            .class("scope")
            .unwrap()
            .selector();

        assert!(
            selector
                .matches_with_context(
                    &tree,
                    SelectorMatchContext::new(1, Traversal::Canonical).with_scope(1),
                )
                .unwrap()
        );
        assert!(
            !selector
                .matches_with_context(
                    &tree,
                    SelectorMatchContext::new(0, Traversal::Canonical).with_scope(1),
                )
                .unwrap()
        );
    }

    #[test]
    fn selector_specificity_uses_css_lowering_contract() {
        let key = Selector::key("submit").unwrap();
        let class = Selector::class("primary").unwrap();
        let attr = Selector::attribute_exists("data-mode").unwrap();
        let tag = Selector::tag("button").unwrap();

        assert_eq!(key.specificity(), SelectorSpecificity::new(1, 0, 0));
        assert_eq!(class.specificity(), SelectorSpecificity::new(0, 1, 0));
        assert_eq!(attr.specificity(), SelectorSpecificity::new(0, 1, 0));
        assert_eq!(tag.specificity(), SelectorSpecificity::new(0, 0, 1));
    }

    #[test]
    fn selector_specificity_sums_compound_and_complex_and_uses_list_max() {
        let compound = Selector::compound()
            .tag("button")
            .unwrap()
            .key("submit")
            .unwrap()
            .class("primary")
            .unwrap()
            .attribute_exists("data-mode")
            .unwrap()
            .selector();
        let complex = Selector::complex([
            ComplexSelectorPart::root(Selector::compound().tag("form").unwrap()),
            ComplexSelectorPart::related(
                Combinator::Descendant,
                Selector::compound().class("primary").unwrap(),
            ),
        ])
        .unwrap();
        let list = Selector::list(
            SelectorList::try_new([
                Selector::tag("button").unwrap(),
                Selector::key("submit").unwrap(),
            ])
            .unwrap(),
        );

        assert_eq!(compound.specificity(), SelectorSpecificity::new(1, 2, 1));
        assert_eq!(complex.specificity(), SelectorSpecificity::new(0, 1, 1));
        assert_eq!(list.specificity(), SelectorSpecificity::new(1, 0, 0));
    }

    #[test]
    fn selector_list_pseudo_class_specificity_uses_argument_rules() {
        let key_list = SelectorList::try_new([Selector::key("primary").unwrap()]).unwrap();
        let class_list = SelectorList::try_new([Selector::class("primary").unwrap()]).unwrap();
        let relative_key = RelativeSelectorList::try_new([RelativeSelector::new(
            Combinator::Descendant,
            Selector::key("target").unwrap(),
        )])
        .unwrap();

        assert_eq!(
            Selector::pseudo(PseudoClassSelector::selector_list(
                SelectorListPseudoClass::Where(key_list.clone()),
            ))
            .specificity(),
            SelectorSpecificity::zero()
        );
        assert_eq!(
            Selector::pseudo(PseudoClassSelector::selector_list(
                SelectorListPseudoClass::Is(key_list),
            ))
            .specificity(),
            SelectorSpecificity::new(1, 0, 0)
        );
        assert_eq!(
            Selector::pseudo(PseudoClassSelector::selector_list(
                SelectorListPseudoClass::Not(class_list),
            ))
            .specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );
        assert_eq!(
            Selector::pseudo(PseudoClassSelector::has(relative_key)).specificity(),
            SelectorSpecificity::new(1, 0, 0)
        );
    }

    #[test]
    fn attribute_selector_supports_css_matcher_variants() {
        let tree = TestTree::new(vec![
            TestNode::new(0)
                .attribute("data-tags", "primary featured")
                .attribute("lang", "en-US")
                .attribute("data-id", "Card-Primary"),
        ]);

        assert!(
            AttributeSelector::includes("data-tags", "featured")
                .unwrap()
                .matches(&tree, 0)
                .unwrap()
        );
        assert!(
            AttributeSelector::dash_match("lang", "en")
                .unwrap()
                .matches(&tree, 0)
                .unwrap()
        );
        assert!(
            AttributeSelector::prefix("data-id", "Card")
                .unwrap()
                .matches(&tree, 0)
                .unwrap()
        );
        assert!(
            AttributeSelector::suffix("data-id", "Primary")
                .unwrap()
                .matches(&tree, 0)
                .unwrap()
        );
        assert!(
            AttributeSelector::substring("data-id", "rd-P")
                .unwrap()
                .matches(&tree, 0)
                .unwrap()
        );
        assert!(
            !AttributeSelector::prefix("data-id", "")
                .unwrap()
                .matches(&tree, 0)
                .unwrap()
        );
        assert!(
            !AttributeSelector::suffix("data-id", "")
                .unwrap()
                .matches(&tree, 0)
                .unwrap()
        );
        assert!(
            !AttributeSelector::substring("data-id", "")
                .unwrap()
                .matches(&tree, 0)
                .unwrap()
        );
        assert!(
            AttributeSelector::equals_with_case(
                "data-id",
                "card-primary",
                AttributeCaseSensitivity::AsciiCaseInsensitive,
            )
            .unwrap()
            .matches(&tree, 0)
            .unwrap()
        );
        assert!(
            !AttributeSelector::equals_with_case(
                "data-id",
                "card-primary",
                AttributeCaseSensitivity::ExplicitSensitive,
            )
            .unwrap()
            .matches(&tree, 0)
            .unwrap()
        );
    }

    #[test]
    fn runtime_pseudo_classes_use_explicit_style_state_facts() {
        let tree = TestTree::new(vec![
            TestNode::new(0).state(
                StyleState::default()
                    .with_enabled(Some(true))
                    .with_focus_visible(true)
                    .with_valid(Some(false))
                    .with_read_write(Some(true)),
            ),
            TestNode::new(1).state(StyleState::default()),
            TestNode::new(2).state(StyleState::default().with_enabled(Some(false))),
        ]);

        assert!(
            Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Enabled))
                .matches(&tree, 0, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Invalid))
                .matches(&tree, 0, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Disabled))
                .matches(&tree, 0, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Enabled))
                .matches(&tree, 1, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Disabled))
                .matches(&tree, 1, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Enabled))
                .matches(&tree, 2, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Disabled))
                .matches(&tree, 2, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Required))
                .matches(&tree, 1, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::Optional))
                .matches(&tree, 1, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::InRange))
                .matches(&tree, 1, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::runtime(RuntimePseudoClass::OutOfRange))
                .matches(&tree, 1, Traversal::Canonical)
                .unwrap()
        );
    }

    #[test]
    fn compound_selectors_can_combine_tag_class_attribute_and_runtime_pseudo_classes() {
        let tree = TestTree::new(vec![
            TestNode::new(0)
                .tag("button")
                .class("primary")
                .attribute("data-mode", "submit")
                .state(StyleState::default().with_hovered(true)),
        ]);
        let selector = Selector::compound()
            .tag("button")
            .unwrap()
            .class("primary")
            .unwrap()
            .attribute_exists("data-mode")
            .unwrap()
            .runtime_pseudo(RuntimePseudoClass::Hover)
            .selector();

        assert!(selector.matches(&tree, 0, Traversal::Canonical).unwrap());
        assert_eq!(selector.specificity(), SelectorSpecificity::new(0, 3, 1));
    }

    #[test]
    fn structural_selectors_match_child_and_type_positions() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("root").children([1, 2, 3, 4]),
            TestNode::new(1).tag("button"),
            TestNode::new(2).tag("label"),
            TestNode::new(3).tag("button"),
            TestNode::new(4).tag("button"),
        ]);

        assert!(
            Selector::pseudo(PseudoClassSelector::structural(
                StructuralSelector::FirstChild,
            ))
            .matches(&tree, 1, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::structural(
                StructuralSelector::LastChild
            ))
            .matches(&tree, 4, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::structural(
                StructuralSelector::NthOfType(NthPattern::integer(2)),
            ))
            .matches(&tree, 3, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::structural(
                StructuralSelector::NthLastOfType(NthPattern::integer(1)),
            ))
            .matches(&tree, 4, Traversal::Canonical)
            .unwrap()
        );
    }

    #[test]
    fn compound_selectors_can_combine_tag_and_structural_pseudo_classes() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("ul").children([1, 2]),
            TestNode::new(1).tag("li"),
            TestNode::new(2).tag("li"),
        ]);
        let selector = Selector::compound()
            .tag("li")
            .unwrap()
            .pseudo(PseudoClassSelector::structural(
                StructuralSelector::NthChild(NthSelector::new(NthPattern::integer(2), None)),
            ))
            .selector();

        assert!(selector.matches(&tree, 2, Traversal::Canonical).unwrap());
        assert_eq!(selector.specificity(), SelectorSpecificity::new(0, 1, 1));
    }

    #[test]
    fn has_matches_relative_descendant_child_and_sibling_selectors() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("section").children([1, 2, 3, 4]),
            TestNode::new(1).tag("button").class("primary"),
            TestNode::new(2).tag("label"),
            TestNode::new(3).tag("button").class("later"),
            TestNode::new(4).tag("input").class("adjacent-target"),
        ]);

        let descendant =
            RelativeSelector::new(Combinator::Descendant, Selector::class("primary").unwrap());
        let child = RelativeSelector::new(Combinator::Child, Selector::tag("label").unwrap());
        let sibling = RelativeSelector::new(Combinator::Sibling, Selector::class("later").unwrap());
        let adjacent = RelativeSelector::new(
            Combinator::Adjacent,
            Selector::class("adjacent-target").unwrap(),
        );
        let wrong_adjacent =
            RelativeSelector::new(Combinator::Adjacent, Selector::class("later").unwrap());

        assert!(
            Selector::pseudo(PseudoClassSelector::has(
                RelativeSelectorList::try_new([descendant]).unwrap(),
            ))
            .matches(&tree, 0, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::has(
                RelativeSelectorList::try_new([child]).unwrap(),
            ))
            .matches(&tree, 0, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::has(
                RelativeSelectorList::try_new([sibling]).unwrap(),
            ))
            .matches(&tree, 1, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::has(
                RelativeSelectorList::try_new([adjacent]).unwrap(),
            ))
            .matches(&tree, 3, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::has(
                RelativeSelectorList::try_new([wrong_adjacent]).unwrap(),
            ))
            .matches(&tree, 3, Traversal::Canonical)
            .unwrap()
        );
    }

    #[test]
    fn has_matches_complex_relative_selectors_from_child_and_adjacent_anchors() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("section").children([1, 4, 3]),
            TestNode::new(1).tag("div").class("card").children([2]),
            TestNode::new(2).tag("span").class("target"),
            TestNode::new(3).tag("aside").class("panel").children([5]),
            TestNode::new(4).tag("div").class("before-panel"),
            TestNode::new(5).tag("span").class("target"),
        ]);
        let child_complex = Selector::complex([
            ComplexSelectorPart::root(Selector::compound().class("card").unwrap()),
            ComplexSelectorPart::related(
                Combinator::Descendant,
                Selector::compound().class("target").unwrap(),
            ),
        ])
        .unwrap();
        let adjacent_complex = Selector::complex([
            ComplexSelectorPart::root(Selector::compound().class("panel").unwrap()),
            ComplexSelectorPart::related(
                Combinator::Descendant,
                Selector::compound().class("target").unwrap(),
            ),
        ])
        .unwrap();

        assert!(
            Selector::pseudo(PseudoClassSelector::has(
                RelativeSelectorList::try_new([RelativeSelector::new(
                    Combinator::Child,
                    child_complex,
                )])
                .unwrap(),
            ))
            .matches(&tree, 0, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::has(
                RelativeSelectorList::try_new([RelativeSelector::new(
                    Combinator::Adjacent,
                    adjacent_complex,
                )])
                .unwrap(),
            ))
            .matches(&tree, 4, Traversal::Canonical)
            .unwrap()
        );
    }

    #[test]
    fn structural_selectors_cover_only_empty_reverse_filtered_and_type_edge_cases() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("root").children([1, 2, 3, 4, 5]),
            TestNode::new(1).tag("button").class("candidate"),
            TestNode::new(2).tag("button").class("candidate"),
            TestNode::new(3).tag("label"),
            TestNode::new(4).tag("button").class("candidate"),
            TestNode::new(5),
            TestNode::new(6).tag("empty"),
            TestNode::new(7).tag("text").text(),
            TestNode::new(8).tag("single-parent").children([9]),
            TestNode::new(9).tag("only"),
        ]);

        assert!(
            Selector::pseudo(PseudoClassSelector::structural(
                StructuralSelector::OnlyChild,
            ))
            .matches(&tree, 9, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::structural(StructuralSelector::Empty))
                .matches(&tree, 6, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::structural(StructuralSelector::Empty))
                .matches(&tree, 7, Traversal::Canonical)
                .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::structural(
                StructuralSelector::FirstOfType,
            ))
            .matches(&tree, 1, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::structural(
                StructuralSelector::LastOfType,
            ))
            .matches(&tree, 4, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            Selector::pseudo(PseudoClassSelector::structural(
                StructuralSelector::OnlyOfType,
            ))
            .matches(&tree, 3, Traversal::Canonical)
            .unwrap()
        );
        assert!(
            !Selector::pseudo(PseudoClassSelector::structural(
                StructuralSelector::FirstOfType,
            ))
            .matches(&tree, 5, Traversal::Canonical)
            .unwrap()
        );

        let filter = SelectorList::try_new([Selector::class("candidate").unwrap()]).unwrap();
        let nth_last = Selector::pseudo(PseudoClassSelector::structural(
            StructuralSelector::NthLastChild(NthSelector::new(
                NthPattern::integer(2),
                Some(filter),
            )),
        ));
        assert!(nth_last.matches(&tree, 2, Traversal::Canonical).unwrap());
        assert!(!nth_last.matches(&tree, 1, Traversal::Canonical).unwrap());
    }

    #[test]
    fn nth_child_can_filter_siblings_by_selector_list() {
        let tree = TestTree::new(vec![
            TestNode::new(0).tag("root").children([1, 2, 3, 4]),
            TestNode::new(1).tag("button").class("candidate"),
            TestNode::new(2).tag("button"),
            TestNode::new(3).tag("button").class("candidate"),
            TestNode::new(4).tag("button").class("candidate"),
        ]);
        let filter = SelectorList::try_new([Selector::class("candidate").unwrap()]).unwrap();
        let selector = Selector::pseudo(PseudoClassSelector::structural(
            StructuralSelector::NthChild(NthSelector::new(NthPattern::integer(2), Some(filter))),
        ));

        assert!(selector.matches(&tree, 3, Traversal::Canonical).unwrap());
        assert!(!selector.matches(&tree, 4, Traversal::Canonical).unwrap());
    }

    #[test]
    fn nth_patterns_match_signed_css_an_plus_b_positions() {
        assert!(NthPattern::odd().matches(1));
        assert!(NthPattern::odd().matches(3));
        assert!(!NthPattern::odd().matches(2));

        assert!(NthPattern::even().matches(2));
        assert!(NthPattern::even().matches(4));
        assert!(!NthPattern::even().matches(3));

        assert!(NthPattern::integer(3).matches(3));
        assert!(!NthPattern::integer(3).matches(2));

        let two_n_plus_one = NthPattern::new(2, 1);
        assert!(two_n_plus_one.matches(5));
        assert!(!two_n_plus_one.matches(6));

        let negative_n_plus_three = NthPattern::new(-1, 3);
        assert!(negative_n_plus_three.matches(1));
        assert!(negative_n_plus_three.matches(3));
        assert!(!negative_n_plus_three.matches(4));

        assert!(!NthPattern::integer(0).matches(1));
        assert!(!NthPattern::integer(-1).matches(1));
    }

    #[test]
    fn nth_child_filter_specificity_adds_filter_max_specificity() {
        let filter = SelectorList::try_new([
            Selector::tag("button").unwrap(),
            Selector::key("primary").unwrap(),
        ])
        .unwrap();
        let selector = Selector::pseudo(PseudoClassSelector::structural(
            StructuralSelector::NthChild(NthSelector::new(NthPattern::odd(), Some(filter))),
        ));

        assert_eq!(selector.specificity(), SelectorSpecificity::new(1, 1, 0));
    }

    #[test]
    fn complex_selector_rejects_invalid_part_ordering() {
        assert_eq!(
            ComplexSelector::try_new([]).unwrap_err().code(),
            ErrorCode::InvalidSelector
        );
        assert_eq!(
            ComplexSelector::try_new([ComplexSelectorPart::related(
                Combinator::Child,
                Selector::compound().tag("button").unwrap(),
            )])
            .unwrap_err()
            .code(),
            ErrorCode::InvalidSelector
        );
        assert_eq!(
            ComplexSelector::try_new([
                ComplexSelectorPart::root(Selector::compound().tag("form").unwrap()),
                ComplexSelectorPart::root(Selector::compound().tag("button").unwrap()),
            ])
            .unwrap_err()
            .code(),
            ErrorCode::InvalidSelector
        );
    }

    #[derive(Clone, Debug)]
    struct TestNode {
        id: usize,
        tag: Option<StyleTag>,
        key: Option<StyleKey>,
        classes: Vec<StyleClass>,
        attributes: Vec<StyleAttribute>,
        role: StyleRole,
        state: StyleState,
        text: bool,
        children: Vec<usize>,
    }

    impl TestNode {
        fn new(id: usize) -> Self {
            Self {
                id,
                tag: None,
                key: None,
                classes: Vec::new(),
                attributes: Vec::new(),
                role: StyleRole::default(),
                state: StyleState::default(),
                text: false,
                children: Vec::new(),
            }
        }

        fn tag(mut self, tag: &str) -> Self {
            self.tag = Some(StyleTag::new(tag).unwrap());
            self
        }

        fn key(mut self, key: &str) -> Self {
            self.key = Some(StyleKey::new(key).unwrap());
            self
        }

        fn class(mut self, class: &str) -> Self {
            self.classes.push(StyleClass::new(class).unwrap());
            self
        }

        fn attribute(mut self, name: &str, value: &str) -> Self {
            self.attributes.push(StyleAttribute::new(
                StyleAttributeName::new(name).unwrap(),
                StyleAttributeValue::new(value).unwrap(),
            ));
            self
        }

        fn state(mut self, state: StyleState) -> Self {
            self.state = state;
            self
        }

        fn text(mut self) -> Self {
            self.text = true;
            self
        }

        fn children(mut self, children: impl IntoIterator<Item = usize>) -> Self {
            self.children = children.into_iter().collect();
            self
        }
    }

    struct TestTree {
        nodes: Vec<TestNode>,
    }

    impl TestTree {
        fn new(nodes: Vec<TestNode>) -> Self {
            Self { nodes }
        }
    }

    impl Tree for TestTree {
        type Id = usize;

        fn version_hint(&self) -> Option<u64> {
            Some(1)
        }

        fn node(&self, id: Self::Id) -> Result<Node<Self::Id>> {
            let node = self.nodes.get(id).ok_or_else(|| {
                crate::Error::new(crate::ErrorCode::MissingNode, "missing test node")
            })?;
            Ok(Node {
                id: node.id,
                tag: node.tag.clone(),
                key: node.key.clone(),
                classes: node.classes.clone(),
                attributes: node.attributes.clone(),
                role: node.role.clone(),
                state: node.state.clone(),
                text: node.text,
            })
        }

        fn parent(&self, id: Self::Id, _traversal: Traversal) -> Result<Option<Self::Id>> {
            Ok(self
                .nodes
                .iter()
                .find(|node| node.children.contains(&id))
                .map(|node| node.id))
        }

        fn children(
            &self,
            id: Self::Id,
            _traversal: Traversal,
        ) -> Result<impl Iterator<Item = Self::Id> + '_> {
            Ok(self.nodes[id].children.iter().copied())
        }

        fn previous_sibling(&self, id: Self::Id, traversal: Traversal) -> Result<Option<Self::Id>> {
            let Some(parent) = self.parent(id, traversal)? else {
                return Ok(None);
            };
            let siblings = &self.nodes[parent].children;
            Ok(siblings
                .iter()
                .position(|sibling| *sibling == id)
                .and_then(|index| index.checked_sub(1))
                .map(|index| siblings[index]))
        }
    }
}
