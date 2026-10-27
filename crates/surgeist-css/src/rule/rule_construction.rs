//! Checked authored assembly. One iterative traversal owns placement, namespace,
//! selector and rule-depth checks; no resolution or original namespace URI identity.
use crate::component::{CssComponentValueError, CssValueOrigin};
use crate::query::media::CssMediaConstructionError;
use crate::query::supports::{CssSupportsConstructionError, CssSupportsLeaf};
use crate::query::{CssCondition, CssConditionKind};
use crate::rule::{CssRule, CssScopeNestingContext, CssScopeRule};
use crate::selector::{
    CssCompoundSelector, CssNamespaceConstraint, CssNamespaceContext, CssPseudoClass,
    CssPseudoElement, CssPseudoElementSegment, CssPseudoSelectorListItem, CssQualifiedNamePrefix,
    CssScopeSelectorList, CssSelector, CssStyleSelector,
};
use crate::serialization::CssSerializationError;
use crate::source::CssSourcePosition;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CssRuleConstructionErrorKind {
    InvalidPlacement,
    InvalidPreludeOrder,
    DuplicateNamespace,
    NamespaceMismatch,
    InvalidSelectorIdentifier,
    InvalidBoundaryAnchor,
    NestingLimit,
    SelectorNestingLimit,
    CapacityOverflow,
}
impl CssRuleConstructionErrorKind {
    const fn description(self) -> &'static str {
        match self {
            Self::InvalidPlacement => "a rule or selector is not permitted where it appears",
            Self::InvalidPreludeOrder => "an @import or @namespace rule follows a later rule kind",
            Self::DuplicateNamespace => "a namespace prefix is declared twice",
            Self::NamespaceMismatch => {
                "a selector namespace does not match the sheet's declarations"
            }
            Self::InvalidSelectorIdentifier => "a selector identifier is invalid",
            Self::InvalidBoundaryAnchor => "a scope boundary anchor is invalid",
            Self::NestingLimit => "rules nest deeper than the structural limit",
            Self::SelectorNestingLimit => "selectors nest deeper than the structural limit",
            Self::CapacityOverflow => "a structural size calculation overflowed",
        }
    }
}
/// The first authored-order rule assembly failure, or a failed `@import` construction.
///
/// Assembly failures are read through their `is_*` predicates and located by
/// [`Self::path`] and [`Self::position`]. An import failure is read through its
/// predicates and accessors, located by [`Self::origin`], and a rejected component,
/// condition, media query or serialization is its source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssRuleConstructionError {
    kind: RuleConstructionKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum RuleConstructionKind {
    Assembly {
        kind: CssRuleConstructionErrorKind,
        path: Vec<usize>,
        position: Option<CssSourcePosition>,
    },
    Import(crate::rule::imports::ImportConstructionKind),
}
impl CssRuleConstructionError {
    const fn assembly(&self) -> Option<CssRuleConstructionErrorKind> {
        match &self.kind {
            RuleConstructionKind::Assembly { kind, .. } => Some(*kind),
            RuleConstructionKind::Import(_) => None,
        }
    }
    const fn import(&self) -> Option<&crate::rule::imports::ImportConstructionKind> {
        match &self.kind {
            RuleConstructionKind::Import(kind) => Some(kind),
            RuleConstructionKind::Assembly { .. } => None,
        }
    }
    /// Returns whether a rule or selector is not permitted where it appears.
    #[must_use]
    pub const fn is_invalid_placement(&self) -> bool {
        matches!(
            self.assembly(),
            Some(CssRuleConstructionErrorKind::InvalidPlacement)
        )
    }
    /// Returns whether an `@import` or `@namespace` rule follows a later rule.
    #[must_use]
    pub const fn is_invalid_prelude_order(&self) -> bool {
        matches!(
            self.assembly(),
            Some(CssRuleConstructionErrorKind::InvalidPreludeOrder)
        )
    }
    /// Returns whether a namespace prefix or default namespace is declared twice.
    #[must_use]
    pub const fn is_duplicate_namespace(&self) -> bool {
        matches!(
            self.assembly(),
            Some(CssRuleConstructionErrorKind::DuplicateNamespace)
        )
    }
    /// Returns whether a selector namespace does not match the sheet's declarations.
    #[must_use]
    pub const fn is_namespace_mismatch(&self) -> bool {
        matches!(
            self.assembly(),
            Some(CssRuleConstructionErrorKind::NamespaceMismatch)
        )
    }
    /// Returns whether a selector identifier is invalid in its rule.
    #[must_use]
    pub const fn is_invalid_selector_identifier(&self) -> bool {
        matches!(
            self.assembly(),
            Some(CssRuleConstructionErrorKind::InvalidSelectorIdentifier)
        )
    }
    /// Returns whether a scope boundary anchor is invalid.
    #[must_use]
    pub const fn is_invalid_boundary_anchor(&self) -> bool {
        matches!(
            self.assembly(),
            Some(CssRuleConstructionErrorKind::InvalidBoundaryAnchor)
        )
    }
    /// Returns whether rules nest deeper than the structural limit.
    #[must_use]
    pub const fn is_nesting_limit(&self) -> bool {
        matches!(
            self.assembly(),
            Some(CssRuleConstructionErrorKind::NestingLimit)
        )
    }
    /// Returns whether selectors nest deeper than the structural limit.
    #[must_use]
    pub const fn is_selector_nesting_limit(&self) -> bool {
        matches!(
            self.assembly(),
            Some(CssRuleConstructionErrorKind::SelectorNestingLimit)
        )
    }
    /// Returns whether a structural size calculation overflowed.
    #[must_use]
    pub const fn is_capacity_overflow(&self) -> bool {
        matches!(
            self.assembly(),
            Some(CssRuleConstructionErrorKind::CapacityOverflow)
        )
    }
    /// Returns the child-index path of the responsible assembled rule; empty for an
    /// import failure.
    #[must_use]
    pub fn path(&self) -> &[usize] {
        match &self.kind {
            RuleConstructionKind::Assembly { path, .. } => path,
            RuleConstructionKind::Import(_) => &[],
        }
    }
    /// Returns the genuine parsed rule or condition position of an assembly failure,
    /// absent for source-free construction.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        match &self.kind {
            RuleConstructionKind::Assembly { position, .. } => *position,
            RuleConstructionKind::Import(_) => None,
        }
    }
    /// Returns whether the `@import` components do not form an import rule.
    #[must_use]
    pub const fn is_invalid_rule_grammar(&self) -> bool {
        matches!(
            self.import(),
            Some(crate::rule::imports::ImportConstructionKind::InvalidRuleGrammar { .. })
        )
    }
    /// Returns whether the `@import` components contain an implicitly closed component.
    #[must_use]
    pub const fn is_recovered_input(&self) -> bool {
        matches!(
            self.import(),
            Some(crate::rule::imports::ImportConstructionKind::RecoveredInput { .. })
        )
    }
    /// Returns whether the deep `@import` construction thread could not be started.
    #[must_use]
    pub const fn is_worker_unavailable(&self) -> bool {
        matches!(
            self.import(),
            Some(crate::rule::imports::ImportConstructionKind::WorkerUnavailable { .. })
        )
    }
    /// Returns the rejected `@import` component.
    #[must_use]
    pub const fn component_error(&self) -> Option<&CssComponentValueError> {
        match self.import() {
            Some(crate::rule::imports::ImportConstructionKind::Component(error)) => Some(error),
            _ => None,
        }
    }
    /// Returns the rejected `@import` supports condition.
    #[must_use]
    pub const fn supports_error(&self) -> Option<&CssSupportsConstructionError> {
        match self.import() {
            Some(crate::rule::imports::ImportConstructionKind::Supports(error)) => Some(error),
            _ => None,
        }
    }
    /// Returns the rejected `@import` media query list.
    #[must_use]
    pub const fn media_error(&self) -> Option<&CssMediaConstructionError> {
        match self.import() {
            Some(crate::rule::imports::ImportConstructionKind::Media(error)) => Some(error),
            _ => None,
        }
    }
    /// Returns the `@import` serialization failure.
    #[must_use]
    pub const fn serialization_error(&self) -> Option<&CssSerializationError> {
        match self.import() {
            Some(crate::rule::imports::ImportConstructionKind::Serialization(error)) => Some(error),
            _ => None,
        }
    }
    /// Returns the responsible original origin of an `@import` failure.
    #[must_use]
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.import()
            .map(crate::rule::imports::ImportConstructionKind::origin)
    }
    fn new(
        kind: CssRuleConstructionErrorKind,
        path: Vec<usize>,
        position: Option<CssSourcePosition>,
    ) -> Self {
        Self {
            kind: RuleConstructionKind::Assembly {
                kind,
                path,
                position,
            },
        }
    }
}
impl From<crate::rule::imports::ImportConstructionKind> for CssRuleConstructionError {
    fn from(kind: crate::rule::imports::ImportConstructionKind) -> Self {
        Self {
            kind: RuleConstructionKind::Import(kind),
        }
    }
}
impl std::fmt::Display for CssRuleConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            RuleConstructionKind::Assembly { kind, path, .. } => {
                write!(
                    f,
                    "invalid authored rule composition: {}",
                    kind.description()
                )?;
                for (index, child) in path.iter().enumerate() {
                    f.write_str(if index == 0 { " at rule " } else { "." })?;
                    write!(f, "{child}")?;
                }
                Ok(())
            }
            RuleConstructionKind::Import(kind) => kind.describe(f),
        }
    }
}
impl std::error::Error for CssRuleConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.import()?.source()
    }
}

pub(crate) fn sheet(rules: &[CssRule]) -> Result<(), CssRuleConstructionError> {
    let mut phase = 0; // initial, imports, namespaces, body: parser prelude phases
    let mut bindings = Vec::new();
    let mut prelude_error = None;
    for (index, rule) in rules.iter().enumerate() {
        let fail = |kind| CssRuleConstructionError::new(kind, vec![index], ordinary_position(rule));
        match rule {
            CssRule::Import(_) => {
                if phase > 1 {
                    if prelude_error.is_none() {
                        prelude_error =
                            Some(fail(CssRuleConstructionErrorKind::InvalidPreludeOrder));
                    }
                    continue;
                }
                phase = 1;
            }
            CssRule::Namespace(rule) => {
                if phase > 2 {
                    if prelude_error.is_none() {
                        prelude_error =
                            Some(fail(CssRuleConstructionErrorKind::InvalidPreludeOrder));
                    }
                    continue;
                }
                if bindings
                    .iter()
                    .any(|(prefix, _)| prefix == &rule.prefix().cloned())
                {
                    if prelude_error.is_none() {
                        prelude_error =
                            Some(fail(CssRuleConstructionErrorKind::DuplicateNamespace));
                    }
                    continue;
                }
                bindings
                    .try_reserve(1)
                    .map_err(|_| fail(CssRuleConstructionErrorKind::CapacityOverflow))?;
                bindings.push((rule.prefix().cloned(), rule.name().clone()));
                phase = 2;
            }
            CssRule::SupportsCondition(_) => {}
            CssRule::LayerStatement(_) if phase == 0 => {}
            _ => phase = 3,
        }
    }
    validate(
        rules,
        &CssNamespaceContext::from_bindings(bindings),
        Role::Sheet,
        Context::DETACHED,
        0,
        prelude_error.as_ref(),
    )
}

#[derive(Clone, Copy)]
pub(crate) enum Role {
    Sheet,
    Group,
    Scope,
}
/// What a rule list inherits from its owner: whether a style rule encloses it, and
/// which anchor domain a style rule's explicit `&` belongs to. A scope body (and the
/// groups in it) holds scope anchors; every other list, including the body of a style
/// rule inside a scope, holds nesting anchors (CSS Cascade 6 section 2.5).
#[derive(Clone, Copy)]
struct Context {
    style: bool,
    scope_anchors: bool,
}
impl Context {
    const DETACHED: Self = Self {
        style: false,
        scope_anchors: false,
    };
}
struct Frame<'a> {
    rules: &'a [CssRule],
    next: usize,
    role: Role,
    inherited: Context,
    depth: u32,
    previous_conditional: bool,
}

pub(crate) fn conditional_group(
    rules: &[CssRule],
    context: &CssNamespaceContext,
    style: bool,
) -> Result<(), CssRuleConstructionError> {
    let inherited = Context {
        style,
        scope_anchors: false,
    };
    validate(rules, context, Role::Group, inherited, 1, None)
}
pub(crate) fn ordinary_group(
    rules: &[CssRule],
    context: &CssNamespaceContext,
) -> Result<(), CssRuleConstructionError> {
    validate(rules, context, Role::Group, Context::DETACHED, 1, None)
}
pub(crate) fn scope(
    root: Option<&CssScopeSelectorList>,
    limit: Option<&CssScopeSelectorList>,
    rules: &[CssRule],
    context: &CssNamespaceContext,
    nesting: CssScopeNestingContext,
) -> Result<(), CssRuleConstructionError> {
    check_scope_boundaries(root, limit, context, nesting)
        .map_err(|kind| CssRuleConstructionError::new(kind, Vec::new(), None))?;
    validate(
        rules,
        context,
        Role::Scope,
        Context {
            style: nesting == CssScopeNestingContext::Style,
            scope_anchors: true,
        },
        1,
        None,
    )
}
pub(crate) fn supports(
    condition: &CssCondition<CssSupportsLeaf>,
    context: &CssNamespaceContext,
) -> Result<(), CssRuleConstructionError> {
    check_supports(condition, context)
        .map_err(|kind| CssRuleConstructionError::new(kind, Vec::new(), condition.position()))
}
fn validate(
    rules: &[CssRule],
    context: &CssNamespaceContext,
    role: Role,
    inherited: Context,
    depth: u32,
    prelude_error: Option<&CssRuleConstructionError>,
) -> Result<(), CssRuleConstructionError> {
    let mut frames = Vec::new();
    frames.try_reserve(1).map_err(|_| {
        CssRuleConstructionError::new(
            CssRuleConstructionErrorKind::CapacityOverflow,
            Vec::new(),
            None,
        )
    })?;
    frames.push(Frame {
        rules,
        next: 0,
        role,
        inherited,
        depth,
        previous_conditional: false,
    });
    while let Some(frame) = frames.last_mut() {
        if frame.next == frame.rules.len() {
            frames.pop();
            continue;
        }
        let index = frame.next;
        frame.next += 1;
        let (rules, role, inherited, parent_depth) =
            (frame.rules, frame.role, frame.inherited, frame.depth);
        let node = &rules[index];
        let free_else = matches!(node, CssRule::Else(_)) && !frame.previous_conditional;
        frame.previous_conditional = is_conditional(node);
        let path = || {
            frames
                .iter()
                .map(|frame| frame.next - 1)
                .collect::<Vec<_>>()
        };
        let fail = |kind| CssRuleConstructionError::new(kind, path(), ordinary_position(node));
        if frames.len() == 1
            && let Some(error) = prelude_error.filter(|error| error.path().first() == Some(&index))
        {
            return Err(error.clone());
        }
        if free_else {
            return Err(fail(CssRuleConstructionErrorKind::InvalidPlacement));
        }
        let depth = parent_depth + u32::from(is_block(node));
        if depth > crate::STRUCTURAL_NESTING_LIMIT {
            return Err(fail(CssRuleConstructionErrorKind::NestingLimit));
        }
        let scoped = frames.iter().any(|frame| matches!(frame.role, Role::Scope));
        let children = check(node, context, role, inherited, scoped).map_err(fail)?;
        if let Some((rules, role, inherited)) = children {
            frames.try_reserve(1).map_err(|_| {
                CssRuleConstructionError::new(
                    CssRuleConstructionErrorKind::CapacityOverflow,
                    Vec::new(),
                    ordinary_position(node),
                )
            })?;
            frames.push(Frame {
                rules,
                next: 0,
                role,
                inherited,
                depth,
                previous_conditional: false,
            });
        }
    }
    Ok(())
}
fn is_conditional(rule: &CssRule) -> bool {
    matches!(
        rule,
        CssRule::Media(_)
            | CssRule::Supports(_)
            | CssRule::Container(_)
            | CssRule::When(_)
            | CssRule::Else(_)
    )
}
fn is_block(rule: &CssRule) -> bool {
    !matches!(
        rule,
        CssRule::Import(_)
            | CssRule::Namespace(_)
            | CssRule::CustomMedia(_)
            | CssRule::LayerStatement(_)
            | CssRule::SupportsCondition(_)
            | CssRule::NestedDeclarations(_)
    )
}
/// The list a rule contains, with the role and style ancestry of that list.
type Children<'a> = (&'a [CssRule], Role, Context);
/// Checks one rule's placement in its list and returns the list it contains.
fn check<'a>(
    rule: &'a CssRule,
    context: &CssNamespaceContext,
    role: Role,
    inherited: Context,
    scoped: bool,
) -> Result<Option<Children<'a>>, CssRuleConstructionErrorKind> {
    use CssRuleConstructionErrorKind::InvalidPlacement;
    let style = inherited.style;
    match rule {
        CssRule::Import(_) | CssRule::Namespace(_) if !matches!(role, Role::Sheet) => {
            return Err(InvalidPlacement);
        }
        CssRule::FontFace(_)
        | CssRule::FontFeatureValues(_)
        | CssRule::FontPaletteValues(_)
        | CssRule::ColorProfile(_)
        | CssRule::CounterStyle(_)
        | CssRule::Keyframes(_)
        | CssRule::CustomMedia(_)
        | CssRule::SupportsCondition(_)
            if style =>
        {
            return Err(InvalidPlacement);
        }
        CssRule::Page(_) if style || matches!(role, Role::Scope) => {
            return Err(InvalidPlacement);
        }
        CssRule::NestedDeclarations(_) if !style => return Err(InvalidPlacement),
        CssRule::Import(rule) => {
            if let Some(supports) = rule.supports() {
                check_supports(supports.condition(), context)?;
            }
        }
        CssRule::Style(rule) => {
            for selector in rule.selectors().selectors() {
                match selector {
                    CssStyleSelector::Selector(selector) => check_selector_with_anchors(
                        selector,
                        context,
                        Some(inherited.scope_anchors),
                    )?,
                    CssStyleSelector::Relative(selector) => {
                        // A scope body is relative to its scope root; a nested style
                        // rule is relative to its style ancestor.
                        if !style && !scoped {
                            return Err(InvalidPlacement);
                        }
                        check_selector_with_anchors(
                            selector.selector(),
                            context,
                            Some(inherited.scope_anchors),
                        )?;
                    }
                }
            }
            // The body of a style rule is relative to that rule, in any domain.
            let inside = Context {
                style: true,
                scope_anchors: false,
            };
            return Ok(Some((rule.rules(), Role::Group, inside)));
        }
        CssRule::Media(rule) => return Ok(Some((rule.rules(), Role::Group, inherited))),
        CssRule::Supports(rule) => {
            check_supports(rule.condition(), context)?;
            return Ok(Some((rule.rules(), Role::Group, inherited)));
        }
        CssRule::Container(rule) => return Ok(Some((rule.rules(), Role::Group, inherited))),
        CssRule::When(rule) => return Ok(Some((rule.rules(), Role::Group, inherited))),
        CssRule::Else(rule) => return Ok(Some((rule.rules(), Role::Group, inherited))),
        CssRule::LayerBlock(rule) => return Ok(Some((rule.rules(), Role::Group, inherited))),
        CssRule::Scope(rule) => return scope_children(rule, context, inherited, scoped),
        _ => {}
    }
    Ok(None)
}
fn scope_children<'a>(
    rule: &'a CssScopeRule,
    context: &CssNamespaceContext,
    inherited: Context,
    scoped: bool,
) -> Result<Option<Children<'a>>, CssRuleConstructionErrorKind> {
    let nesting = if inherited.style {
        CssScopeNestingContext::Style
    } else if scoped {
        CssScopeNestingContext::Scope
    } else {
        CssScopeNestingContext::None
    };
    check_scope_boundaries(rule.root(), rule.limit(), context, nesting)?;
    let inside = Context {
        style: inherited.style,
        scope_anchors: true,
    };
    Ok(Some((rule.rules().rules(), Role::Scope, inside)))
}

fn check_scope_boundaries(
    root: Option<&CssScopeSelectorList>,
    limit: Option<&CssScopeSelectorList>,
    context: &CssNamespaceContext,
    nesting: CssScopeNestingContext,
) -> Result<(), CssRuleConstructionErrorKind> {
    for (list, allow_relative, scope_anchors) in [
        (
            root,
            nesting != CssScopeNestingContext::None,
            nesting == CssScopeNestingContext::Scope,
        ),
        (limit, true, true),
    ] {
        if let Some(list) = list {
            for member in list.selectors() {
                if matches!(member, CssStyleSelector::Relative(_)) && !allow_relative {
                    return Err(CssRuleConstructionErrorKind::InvalidPlacement);
                }
                check_selector_with_anchors(member.selector(), context, Some(scope_anchors))?;
            }
        }
    }
    Ok(())
}
fn ordinary_position(rule: &CssRule) -> Option<CssSourcePosition> {
    match rule {
        CssRule::FontFace(value) => value.position(),
        CssRule::FontFeatureValues(value) => value.position(),
        CssRule::FontPaletteValues(value) => value.position(),
        CssRule::ColorProfile(value) => value.position(),
        CssRule::Import(value) => value.position(),
        CssRule::Namespace(value) => value.position(),
        CssRule::CustomMedia(value) => value.position(),
        CssRule::SupportsCondition(value) => value.position(),
        CssRule::Media(value) => value.position(),
        CssRule::Supports(value) => value.position(),
        CssRule::Container(value) => value.position(),
        CssRule::When(value) => value.position(),
        CssRule::Else(value) => value.position(),
        CssRule::LayerBlock(value) => value.position(),
        CssRule::Scope(value) => value.position(),
        CssRule::Style(value) => Some(value.position()),
        CssRule::Page(value) => Some(value.position()),
        CssRule::CounterStyle(value) => Some(value.position()),
        CssRule::Keyframes(value) => Some(value.position()),
        CssRule::LayerStatement(value) => Some(value.position()),
        CssRule::NestedDeclarations(value) => Some(value.position()),
    }
}

pub(crate) fn check_namespace(
    prefix: &CssQualifiedNamePrefix,
    constraint: &CssNamespaceConstraint,
    context: &CssNamespaceContext,
    default: bool,
) -> Result<(), CssRuleConstructionErrorKind> {
    let expected = match prefix {
        CssQualifiedNamePrefix::Unqualified if !default => CssNamespaceConstraint::ExplicitNone,
        CssQualifiedNamePrefix::Unqualified if context.default_namespace().is_some() => {
            CssNamespaceConstraint::Default
        }
        CssQualifiedNamePrefix::Unqualified | CssQualifiedNamePrefix::Any => {
            CssNamespaceConstraint::Any
        }
        CssQualifiedNamePrefix::ExplicitNone => CssNamespaceConstraint::ExplicitNone,
        CssQualifiedNamePrefix::Named(prefix) => {
            if context.named_namespace(prefix).is_none() {
                return Err(CssRuleConstructionErrorKind::NamespaceMismatch);
            }
            CssNamespaceConstraint::Named(prefix.clone())
        }
    };
    if &expected == constraint {
        Ok(())
    } else {
        Err(CssRuleConstructionErrorKind::NamespaceMismatch)
    }
}
fn identifier(value: &str) -> Result<(), CssRuleConstructionErrorKind> {
    if value.is_empty() || value.contains('\0') {
        Err(CssRuleConstructionErrorKind::InvalidSelectorIdentifier)
    } else {
        Ok(())
    }
}
struct CheckedStack<T>(Vec<T>);
impl<T> CheckedStack<T> {
    fn new(value: T) -> Result<Self, CssRuleConstructionErrorKind> {
        let mut stack = Self(Vec::new());
        stack.push(value)?;
        Ok(stack)
    }
    fn push(&mut self, value: T) -> Result<(), CssRuleConstructionErrorKind> {
        self.0
            .try_reserve(1)
            .map_err(|_| CssRuleConstructionErrorKind::CapacityOverflow)?;
        self.0.push(value);
        Ok(())
    }
    fn extend(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<(), CssRuleConstructionErrorKind> {
        for value in values {
            self.push(value)?;
        }
        Ok(())
    }
    fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}
fn check_supports(
    condition: &CssCondition<CssSupportsLeaf>,
    context: &CssNamespaceContext,
) -> Result<(), CssRuleConstructionErrorKind> {
    let mut stack = CheckedStack::new(condition)?;
    while let Some(condition) = stack.pop() {
        match condition.kind() {
            CssConditionKind::Leaf(CssSupportsLeaf::Selector(selector)) => {
                check_selector(selector, context)?
            }
            CssConditionKind::Not(condition) | CssConditionKind::Parenthesized(condition) => {
                stack.push(condition)?
            }
            CssConditionKind::And(list) | CssConditionKind::Or(list) => {
                stack.extend(list.conditions().iter().rev())?
            }
            CssConditionKind::Leaf(_) => {}
        }
    }
    Ok(())
}
enum SelectorWork<'a> {
    Selector(&'a CssSelector, u32),
    Compound(&'a CssCompoundSelector, u32),
    Pseudo(&'a CssPseudoClass, u32),
}
fn check_selector(
    selector: &CssSelector,
    context: &CssNamespaceContext,
) -> Result<(), CssRuleConstructionErrorKind> {
    check_selector_with_anchors(selector, context, None)
}
fn check_selector_with_anchors(
    selector: &CssSelector,
    context: &CssNamespaceContext,
    scope_anchors: Option<bool>,
) -> Result<(), CssRuleConstructionErrorKind> {
    let mut stack = CheckedStack::new(SelectorWork::Selector(selector, 0))?;
    while let Some(work) = stack.pop() {
        match work {
            SelectorWork::Selector(selector, depth) => match selector {
                CssSelector::Tag(value) => {
                    identifier(value)?;
                    if context.default_namespace().is_some() {
                        return Err(CssRuleConstructionErrorKind::NamespaceMismatch);
                    }
                }
                CssSelector::Key(value) | CssSelector::Class(value) => identifier(value)?,
                CssSelector::PseudoClass(pseudo) => {
                    stack.push(SelectorWork::Pseudo(pseudo, depth))?
                }
                CssSelector::Compound(compound) => {
                    stack.push(SelectorWork::Compound(compound, depth))?
                }
                CssSelector::Complex(complex) => {
                    for part in complex.rest().iter().rev() {
                        stack.push(SelectorWork::Compound(part.selector(), depth))?;
                    }
                    stack.push(SelectorWork::Compound(complex.first(), depth))?;
                }
            },
            SelectorWork::Compound(compound, depth) => {
                if scope_anchors.is_some_and(|scope| {
                    if scope {
                        compound.nesting_selectors() != 0
                    } else {
                        compound.has_scope_anchor()
                    }
                }) {
                    return Err(CssRuleConstructionErrorKind::InvalidBoundaryAnchor);
                }
                if let Some(name) = compound.type_selector() {
                    if let Some(local) = name.local_name() {
                        identifier(local)?;
                    }
                    check_namespace(name.prefix(), name.namespace(), context, true)?;
                }
                for value in compound.ids().iter().chain(compound.classes()) {
                    identifier(value)?;
                }
                for attr in compound.attributes() {
                    let name = attr.qualified_name();
                    check_namespace(name.prefix(), name.namespace(), context, false)?;
                }
                if let Some(elements) = compound.pseudo_elements() {
                    for segment in elements.segments().iter().rev() {
                        match segment {
                            CssPseudoElementSegment::PseudoClass(pseudo) => {
                                stack.push(SelectorWork::Pseudo(pseudo, depth))?
                            }
                            CssPseudoElementSegment::PseudoElement(CssPseudoElement::Slotted(
                                argument,
                            )) => {
                                if depth == crate::STRUCTURAL_NESTING_LIMIT {
                                    return Err(CssRuleConstructionErrorKind::SelectorNestingLimit);
                                }
                                stack
                                    .push(SelectorWork::Compound(argument.compound(), depth + 1))?;
                            }
                            CssPseudoElementSegment::PseudoElement(
                                CssPseudoElement::Part(_) | CssPseudoElement::Highlight(_),
                            ) if depth == crate::STRUCTURAL_NESTING_LIMIT => {
                                return Err(CssRuleConstructionErrorKind::SelectorNestingLimit);
                            }
                            _ => {}
                        }
                    }
                }
                for pseudo in compound.pseudo_classes().iter().rev() {
                    stack.push(SelectorWork::Pseudo(pseudo, depth))?;
                }
            }
            SelectorWork::Pseudo(pseudo, depth) => {
                let next = depth + 1;
                let functional = matches!(
                    pseudo,
                    CssPseudoClass::Not(_)
                        | CssPseudoClass::Is(_)
                        | CssPseudoClass::Where(_)
                        | CssPseudoClass::Has(_)
                        | CssPseudoClass::HostFunction(_)
                        | CssPseudoClass::HostContext(_)
                        | CssPseudoClass::NthChild(_)
                        | CssPseudoClass::NthLastChild(_)
                        | CssPseudoClass::NthOfType(_)
                        | CssPseudoClass::NthLastOfType(_)
                        | CssPseudoClass::Dir(_)
                        | CssPseudoClass::Lang(_)
                );
                if functional && next > crate::STRUCTURAL_NESTING_LIMIT {
                    return Err(CssRuleConstructionErrorKind::SelectorNestingLimit);
                }
                match pseudo {
                    CssPseudoClass::Not(list)
                    | CssPseudoClass::Is(list)
                    | CssPseudoClass::Where(list) => {
                        for item in list.items().iter().rev() {
                            match item {
                                CssPseudoSelectorListItem::Selector(selector) => {
                                    stack.push(SelectorWork::Selector(selector, next))?
                                }
                                CssPseudoSelectorListItem::InvalidNesting(item) => {
                                    if matches!(pseudo, CssPseudoClass::Not(_)) {
                                        return Err(CssRuleConstructionErrorKind::InvalidPlacement);
                                    }
                                    if next + item.components().nesting_depth()
                                        > crate::STRUCTURAL_NESTING_LIMIT
                                    {
                                        return Err(
                                            CssRuleConstructionErrorKind::SelectorNestingLimit,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    CssPseudoClass::Has(list) => {
                        for selector in list.selectors().iter().rev() {
                            stack.push(SelectorWork::Selector(selector.selector(), next))?;
                        }
                    }
                    CssPseudoClass::HostFunction(argument)
                    | CssPseudoClass::HostContext(argument) => {
                        stack.push(SelectorWork::Compound(argument.compound(), next))?
                    }
                    CssPseudoClass::NthChild(pattern) | CssPseudoClass::NthLastChild(pattern) => {
                        if let Some(list) = pattern.selector_list() {
                            for item in list.items().iter().rev() {
                                match item {
                                    CssPseudoSelectorListItem::Selector(selector) => {
                                        stack.push(SelectorWork::Selector(selector, next))?
                                    }
                                    CssPseudoSelectorListItem::InvalidNesting(_) => {
                                        return Err(CssRuleConstructionErrorKind::InvalidPlacement);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
