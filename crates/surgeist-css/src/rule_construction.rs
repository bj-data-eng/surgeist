//! Checked authored assembly. One iterative traversal owns placement, namespace,
//! selector and rule-depth checks; no resolution or original namespace URI identity.
use crate::*;

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
    Import(crate::imports::ImportConstructionKind),
}
impl CssRuleConstructionError {
    const fn assembly(&self) -> Option<CssRuleConstructionErrorKind> {
        match &self.kind {
            RuleConstructionKind::Assembly { kind, .. } => Some(*kind),
            RuleConstructionKind::Import(_) => None,
        }
    }
    const fn import(&self) -> Option<&crate::imports::ImportConstructionKind> {
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
            Some(crate::imports::ImportConstructionKind::InvalidRuleGrammar { .. })
        )
    }
    /// Returns whether the `@import` components contain an implicitly closed component.
    #[must_use]
    pub const fn is_recovered_input(&self) -> bool {
        matches!(
            self.import(),
            Some(crate::imports::ImportConstructionKind::RecoveredInput { .. })
        )
    }
    /// Returns whether the deep `@import` construction thread could not be started.
    #[must_use]
    pub const fn is_worker_unavailable(&self) -> bool {
        matches!(
            self.import(),
            Some(crate::imports::ImportConstructionKind::WorkerUnavailable { .. })
        )
    }
    /// Returns the rejected `@import` component.
    #[must_use]
    pub const fn component_error(&self) -> Option<&CssComponentValueError> {
        match self.import() {
            Some(crate::imports::ImportConstructionKind::Component(error)) => Some(error),
            _ => None,
        }
    }
    /// Returns the rejected `@import` supports condition.
    #[must_use]
    pub const fn supports_error(&self) -> Option<&CssSupportsConstructionError> {
        match self.import() {
            Some(crate::imports::ImportConstructionKind::Supports(error)) => Some(error),
            _ => None,
        }
    }
    /// Returns the rejected `@import` media query list.
    #[must_use]
    pub const fn media_error(&self) -> Option<&CssMediaConstructionError> {
        match self.import() {
            Some(crate::imports::ImportConstructionKind::Media(error)) => Some(error),
            _ => None,
        }
    }
    /// Returns the `@import` serialization failure.
    #[must_use]
    pub const fn serialization_error(&self) -> Option<&CssSerializationError> {
        match self.import() {
            Some(crate::imports::ImportConstructionKind::Serialization(error)) => Some(error),
            _ => None,
        }
    }
    /// Returns the responsible original origin of an `@import` failure.
    #[must_use]
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.import()
            .map(crate::imports::ImportConstructionKind::origin)
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
impl From<crate::imports::ImportConstructionKind> for CssRuleConstructionError {
    fn from(kind: crate::imports::ImportConstructionKind) -> Self {
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
        List::Ordinary(rules),
        &CssNamespaceContext::from_bindings(bindings),
        Role::Sheet,
        false,
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
#[derive(Clone, Copy)]
pub(crate) enum List<'a> {
    Ordinary(&'a [CssRule]),
    Scoped(&'a [CssScopedRule]),
}
impl List<'_> {
    fn len(self) -> usize {
        match self {
            Self::Ordinary(rules) => rules.len(),
            Self::Scoped(rules) => rules.len(),
        }
    }
}
struct Frame<'a> {
    list: List<'a>,
    next: usize,
    role: Role,
    style: bool,
    depth: u32,
    previous_conditional: bool,
}

pub(crate) fn conditional_group(
    list: List<'_>,
    context: &CssNamespaceContext,
    style: bool,
) -> Result<(), CssRuleConstructionError> {
    validate(list, context, Role::Group, style, 1, None)
}
pub(crate) fn ordinary_group(
    rules: &[CssRule],
    context: &CssNamespaceContext,
) -> Result<(), CssRuleConstructionError> {
    validate(List::Ordinary(rules), context, Role::Group, false, 1, None)
}
pub(crate) fn scoped_group(
    rules: &[CssScopedRule],
    context: &CssNamespaceContext,
) -> Result<(), CssRuleConstructionError> {
    validate(List::Scoped(rules), context, Role::Group, false, 1, None)
}
pub(crate) fn scope(
    root: Option<&CssScopeSelectorList>,
    limit: Option<&CssScopeSelectorList>,
    rules: &[CssScopedRule],
    context: &CssNamespaceContext,
    nesting: CssScopeNestingContext,
) -> Result<(), CssRuleConstructionError> {
    check_scope_boundaries(root, limit, context, nesting)
        .map_err(|kind| CssRuleConstructionError::new(kind, Vec::new(), None))?;
    validate(
        List::Scoped(rules),
        context,
        Role::Scope,
        nesting == CssScopeNestingContext::Style,
        1,
        None,
    )
}
pub(crate) fn supports(
    condition: &CssSupportsCondition,
    context: &CssNamespaceContext,
) -> Result<(), CssRuleConstructionError> {
    check_supports(condition, context)
        .map_err(|kind| CssRuleConstructionError::new(kind, Vec::new(), condition.position()))
}
fn validate(
    list: List<'_>,
    context: &CssNamespaceContext,
    role: Role,
    style: bool,
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
        list,
        next: 0,
        role,
        style,
        depth,
        previous_conditional: false,
    });
    while let Some(frame) = frames.last_mut() {
        if frame.next == frame.list.len() {
            frames.pop();
            continue;
        }
        let index = frame.next;
        frame.next += 1;
        let (list, role, style, parent_depth) = (frame.list, frame.role, frame.style, frame.depth);
        let node = match list {
            List::Ordinary(rules) => Node::Ordinary(&rules[index]),
            List::Scoped(rules) => Node::Scoped(&rules[index]),
        };
        let free_else = node.is_else() && !frame.previous_conditional;
        frame.previous_conditional = node.is_conditional();
        let path = || {
            frames
                .iter()
                .map(|frame| frame.next - 1)
                .collect::<Vec<_>>()
        };
        let fail = |kind| CssRuleConstructionError::new(kind, path(), node.position());
        if frames.len() == 1
            && let Some(error) = prelude_error.filter(|error| error.path().first() == Some(&index))
        {
            return Err(error.clone());
        }
        if free_else {
            return Err(fail(CssRuleConstructionErrorKind::InvalidPlacement));
        }
        let depth = parent_depth + u32::from(node.block());
        if depth > crate::STRUCTURAL_NESTING_LIMIT {
            return Err(fail(CssRuleConstructionErrorKind::NestingLimit));
        }
        let scoped = matches!(list, List::Scoped(_))
            || frames.iter().any(|frame| matches!(frame.role, Role::Scope));
        let children = node.check(context, role, style, scoped).map_err(fail)?;
        if let Some((list, role, style)) = children {
            frames.try_reserve(1).map_err(|_| {
                CssRuleConstructionError::new(
                    CssRuleConstructionErrorKind::CapacityOverflow,
                    Vec::new(),
                    node.position(),
                )
            })?;
            frames.push(Frame {
                list,
                next: 0,
                role,
                style,
                depth,
                previous_conditional: false,
            });
        }
    }
    Ok(())
}
#[derive(Clone, Copy)]
enum Node<'a> {
    Ordinary(&'a CssRule),
    Scoped(&'a CssScopedRule),
}
impl<'a> Node<'a> {
    fn is_else(self) -> bool {
        matches!(
            self,
            Self::Ordinary(CssRule::Else(_)) | Self::Scoped(CssScopedRule::Else(_))
        )
    }
    fn is_conditional(self) -> bool {
        matches!(
            self,
            Self::Ordinary(
                CssRule::Media(_)
                    | CssRule::Supports(_)
                    | CssRule::Container(_)
                    | CssRule::When(_)
                    | CssRule::Else(_)
            ) | Self::Scoped(
                CssScopedRule::Media(_)
                    | CssScopedRule::Supports(_)
                    | CssScopedRule::Container(_)
                    | CssScopedRule::When(_)
                    | CssScopedRule::Else(_)
            )
        )
    }

    fn block(self) -> bool {
        !matches!(
            self,
            Self::Ordinary(
                CssRule::Import(_)
                    | CssRule::Namespace(_)
                    | CssRule::CustomMedia(_)
                    | CssRule::LayerStatement(_)
                    | CssRule::SupportsCondition(_)
                    | CssRule::NestedDeclarations(_),
            ) | Self::Scoped(
                CssScopedRule::CustomMedia(_)
                    | CssScopedRule::LayerStatement(_)
                    | CssScopedRule::SupportsCondition(_)
                    | CssScopedRule::NestedDeclarations(_),
            )
        )
    }
    fn position(self) -> Option<CssSourcePosition> {
        match self {
            Self::Ordinary(rule) => ordinary_position(rule),
            Self::Scoped(rule) => scoped_position(rule),
        }
    }
    fn check(
        self,
        context: &CssNamespaceContext,
        role: Role,
        style: bool,
        scoped: bool,
    ) -> Result<Option<(List<'a>, Role, bool)>, CssRuleConstructionErrorKind> {
        use CssRuleConstructionErrorKind::InvalidPlacement;
        match self {
            Self::Ordinary(rule) => match rule {
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
                            CssStyleSelector::Selector(selector) => {
                                check_selector(selector, context)?
                            }
                            CssStyleSelector::Relative(selector) => {
                                if !style {
                                    return Err(InvalidPlacement);
                                }
                                check_selector(selector.selector(), context)?;
                            }
                        }
                    }
                    return Ok(Some((List::Ordinary(rule.rules()), Role::Group, true)));
                }
                CssRule::Media(rule) => {
                    return Ok(Some((List::Ordinary(rule.rules()), Role::Group, style)));
                }
                CssRule::Supports(rule) => {
                    check_supports(rule.condition(), context)?;
                    return Ok(Some((List::Ordinary(rule.rules()), Role::Group, style)));
                }
                CssRule::Container(rule) => {
                    return Ok(Some((List::Ordinary(rule.rules()), Role::Group, style)));
                }
                CssRule::When(rule) => {
                    return Ok(Some((List::Ordinary(rule.rules()), Role::Group, style)));
                }
                CssRule::Else(rule) => {
                    return Ok(Some((List::Ordinary(rule.rules()), Role::Group, style)));
                }
                CssRule::LayerBlock(rule) => {
                    return Ok(Some((List::Ordinary(rule.rules()), Role::Group, style)));
                }
                CssRule::Scope(rule) => return scope_children(rule, context, style, scoped),
                _ => {}
            },
            Self::Scoped(rule) => match rule {
                CssScopedRule::FontFace(_)
                | CssScopedRule::FontFeatureValues(_)
                | CssScopedRule::FontPaletteValues(_)
                | CssScopedRule::ColorProfile(_)
                | CssScopedRule::CounterStyle(_)
                | CssScopedRule::Keyframes(_)
                | CssScopedRule::CustomMedia(_)
                | CssScopedRule::SupportsCondition(_)
                    if style =>
                {
                    return Err(InvalidPlacement);
                }
                CssScopedRule::Page(_) if style || matches!(role, Role::Scope) => {
                    return Err(InvalidPlacement);
                }
                CssScopedRule::NestedDeclarations(_) if !style => return Err(InvalidPlacement),
                CssScopedRule::Style(rule) => {
                    for selector in rule.selectors().selectors() {
                        match selector {
                            CssScopedStyleSelector::Selector(selector) => {
                                check_selector(selector, context)?
                            }
                            CssScopedStyleSelector::Relative(selector) => {
                                check_selector(selector.selector(), context)?
                            }
                        }
                    }
                    return Ok(Some((List::Ordinary(rule.rules()), Role::Group, true)));
                }
                CssScopedRule::Media(rule) => {
                    return Ok(Some((
                        List::Scoped(rule.rules().rules()),
                        Role::Group,
                        style,
                    )));
                }
                CssScopedRule::Supports(rule) => {
                    check_supports(rule.condition(), context)?;
                    return Ok(Some((
                        List::Scoped(rule.rules().rules()),
                        Role::Group,
                        style,
                    )));
                }
                CssScopedRule::Container(rule) => {
                    return Ok(Some((
                        List::Scoped(rule.rules().rules()),
                        Role::Group,
                        style,
                    )));
                }
                CssScopedRule::When(rule) => {
                    return Ok(Some((
                        List::Scoped(rule.rules().rules()),
                        Role::Group,
                        style,
                    )));
                }
                CssScopedRule::Else(rule) => {
                    return Ok(Some((
                        List::Scoped(rule.rules().rules()),
                        Role::Group,
                        style,
                    )));
                }
                CssScopedRule::LayerBlock(rule) => {
                    return Ok(Some((
                        List::Scoped(rule.rules().rules()),
                        Role::Group,
                        style,
                    )));
                }
                CssScopedRule::Scope(rule) => return scope_children(rule, context, style, scoped),
                _ => {}
            },
        }
        Ok(None)
    }
}
fn scope_children<'a>(
    rule: &'a CssScopeRule,
    context: &CssNamespaceContext,
    style: bool,
    scoped: bool,
) -> Result<Option<(List<'a>, Role, bool)>, CssRuleConstructionErrorKind> {
    let nesting = if style {
        CssScopeNestingContext::Style
    } else if scoped {
        CssScopeNestingContext::Scope
    } else {
        CssScopeNestingContext::None
    };
    check_scope_boundaries(rule.root(), rule.limit(), context, nesting)?;
    Ok(Some((
        List::Scoped(rule.rules().rules()),
        Role::Scope,
        style,
    )))
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
                if matches!(member, CssScopeSelector::Relative(_)) && !allow_relative {
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
fn scoped_position(rule: &CssScopedRule) -> Option<CssSourcePosition> {
    match rule {
        CssScopedRule::FontFace(value) => value.position(),
        CssScopedRule::FontFeatureValues(value) => value.position(),
        CssScopedRule::FontPaletteValues(value) => value.position(),
        CssScopedRule::ColorProfile(value) => value.position(),
        CssScopedRule::CustomMedia(value) => value.position(),
        CssScopedRule::SupportsCondition(value) => value.position(),
        CssScopedRule::Media(value) => value.position(),
        CssScopedRule::Supports(value) => value.position(),
        CssScopedRule::Container(value) => value.position(),
        CssScopedRule::When(value) => value.position(),
        CssScopedRule::Else(value) => value.position(),
        CssScopedRule::LayerBlock(value) => value.position(),
        CssScopedRule::Scope(value) => value.position(),
        CssScopedRule::Style(value) => Some(value.position()),
        CssScopedRule::Page(value) => Some(value.position()),
        CssScopedRule::CounterStyle(value) => Some(value.position()),
        CssScopedRule::Keyframes(value) => Some(value.position()),
        CssScopedRule::LayerStatement(value) => Some(value.position()),
        CssScopedRule::NestedDeclarations(value) => Some(value.position()),
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
    condition: &CssSupportsCondition,
    context: &CssNamespaceContext,
) -> Result<(), CssRuleConstructionErrorKind> {
    let mut stack = CheckedStack::new(condition)?;
    while let Some(condition) = stack.pop() {
        match condition.kind() {
            CssSupportsConditionKind::Selector(selector) => check_selector(selector, context)?,
            CssSupportsConditionKind::Not(condition) => stack.push(condition)?,
            CssSupportsConditionKind::And(list) | CssSupportsConditionKind::Or(list) => {
                stack.extend(list.conditions().iter().rev())?
            }
            _ => {}
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
