//! Authored named supports definitions and their feature-test candidates.

use crate::component_values::{CssCanonicalBuilder, CssCanonicalToken};
use crate::*;
use std::{fmt, ops::Range};

/// A checked, decoded extension name with its original identifier token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSupportsConditionName {
    component: CssComponentValue,
    name: String,
}

impl CssSupportsConditionName {
    pub fn try_new(name: impl Into<String>) -> Result<Self, CssNamedSupportsConstructionError> {
        Self::try_from_component(CssComponentValue::try_ident(name)?)
    }

    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssNamedSupportsConstructionError> {
        let Some(name) = crate::custom_media::extension_name(&component) else {
            return Err(CssNamedSupportsConstructionError::InvalidName {
                origin: component.origin().clone(),
            });
        };
        Ok(Self {
            name: name.to_owned(),
            component,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }

    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }

    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
}

/// A structurally valid feature-test declaration. Its value need not be a known CSS property.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsTestDeclaration {
    pub(crate) components: Vec<CssComponentValue>,
    pub(crate) property_index: usize,
    pub(crate) value_range: Range<usize>,
    pub(crate) importance_range: Option<Range<usize>>,
    pub(crate) name: String,
    pub(crate) terminator: CssValueOrigin,
    pub(crate) recovery: Vec<CssValueOrigin>,
}

impl CssSupportsTestDeclaration {
    pub fn property(&self) -> &str {
        &self.name
    }

    pub fn property_component(&self) -> &CssComponentValue {
        &self.components[self.property_index]
    }

    pub fn value_components(&self) -> &[CssComponentValue] {
        &self.components[self.value_range.clone()]
    }

    pub fn components(&self) -> &[CssComponentValue] {
        &self.components
    }

    pub fn importance_components(&self) -> Option<&[CssComponentValue]> {
        self.importance_range
            .as_ref()
            .map(|range| &self.components[range.clone()])
    }

    pub fn importance(&self) -> CssImportance {
        if self.importance_range.is_some() {
            CssImportance::Important
        } else {
            CssImportance::Normal
        }
    }

    pub fn origin(&self) -> &CssValueOrigin {
        self.property_component().origin()
    }

    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
}

/// A nonempty, ordered declaration run within a feature-test body.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsTestDeclarations {
    pub(crate) declarations: Vec<CssSupportsTestDeclaration>,
}

impl CssSupportsTestDeclarations {
    pub fn declarations(&self) -> &[CssSupportsTestDeclaration] {
        &self.declarations
    }
}

/// A generic qualified rule tested by a named supports definition.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsQualifiedRuleTest {
    pub(crate) data: Box<SupportsQualifiedRuleTestData>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SupportsQualifiedRuleTestData {
    pub(crate) prelude: CssComponentValues,
    pub(crate) body: Box<CssSupportsTestBody>,
    pub(crate) opening: CssValueOrigin,
    pub(crate) closing: CssValueOrigin,
    pub(crate) recovery: Vec<CssValueOrigin>,
}

impl CssSupportsQualifiedRuleTest {
    pub(crate) fn new(
        prelude: CssComponentValues,
        body: Box<CssSupportsTestBody>,
        opening: CssValueOrigin,
        closing: CssValueOrigin,
    ) -> Self {
        let recovery = retained_component_recovery(prelude.items());
        Self {
            data: Box::new(SupportsQualifiedRuleTestData {
                prelude,
                body,
                opening,
                closing,
                recovery,
            }),
        }
    }

    pub fn prelude(&self) -> &CssComponentValues {
        &self.data.prelude
    }

    pub fn body(&self) -> &CssSupportsTestBody {
        &self.data.body
    }

    pub fn origin(&self) -> &CssValueOrigin {
        self.data
            .prelude
            .items()
            .iter()
            .find(|value| !crate::supports::trivia(value))
            .map_or(&self.data.opening, CssComponentValue::origin)
    }
}

/// A generic at-rule test, including unknown future at-rules.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsAtRuleTest {
    pub(crate) data: Box<SupportsAtRuleTestData>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SupportsAtRuleTestData {
    name: String,
    at_keyword: CssComponentValue,
    prelude: CssComponentValues,
    body: Option<Box<CssSupportsTestBody>>,
    pub(crate) opening: Option<CssValueOrigin>,
    pub(crate) closing: CssValueOrigin,
    recovery: Vec<CssValueOrigin>,
}

impl CssSupportsAtRuleTest {
    pub(crate) fn new(
        name: String,
        at_keyword: CssComponentValue,
        prelude: CssComponentValues,
        body: Option<Box<CssSupportsTestBody>>,
        opening: Option<CssValueOrigin>,
        closing: CssValueOrigin,
    ) -> Self {
        let recovery = retained_component_recovery(prelude.items());
        Self {
            data: Box::new(SupportsAtRuleTestData {
                name,
                at_keyword,
                prelude,
                body,
                opening,
                closing,
                recovery,
            }),
        }
    }

    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn at_keyword(&self) -> &CssComponentValue {
        &self.data.at_keyword
    }

    pub fn prelude(&self) -> &CssComponentValues {
        &self.data.prelude
    }

    pub fn body(&self) -> Option<&CssSupportsTestBody> {
        self.data.body.as_deref()
    }

    pub fn origin(&self) -> &CssValueOrigin {
        self.data.at_keyword.origin()
    }
}

/// One ordered generic feature-test candidate or nonempty declaration run.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssSupportsTestItem {
    Declarations(CssSupportsTestDeclarations),
    QualifiedRule(CssSupportsQualifiedRuleTest),
    AtRule(CssSupportsAtRuleTest),
}

/// An authored, non-rendering feature-test body.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsTestBody {
    pub(crate) items: Vec<CssSupportsTestItem>,
    pub(crate) local_recovery: Vec<CssValueOrigin>,
    pub(crate) recovery_origin: Option<CssValueOrigin>,
}

impl CssSupportsTestBody {
    pub fn try_from_components(
        values: CssComponentValues,
    ) -> Result<Self, CssNamedSupportsConstructionError> {
        Self::try_from_components_with_limits(values, CssComponentValueLimits::default())
    }

    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssNamedSupportsConstructionError> {
        values.validate_with_limits(limits)?;
        if let Some(origin) = values.first_implicit_origin() {
            return Err(CssNamedSupportsConstructionError::RecoveredInput {
                origin: origin.clone(),
            });
        }
        let body = crate::parser::named_supports::construct_test_body(values, limits)?;
        let output = body.serialize_with_limit(limits.max_css_bytes())?;
        validate_emitted_budget(&output, limits, &CssValueOrigin::Programmatic)?;
        Ok(body)
    }

    pub fn items(&self) -> &[CssSupportsTestItem] {
        &self.items
    }

    pub fn recovery_origin(&self) -> Option<&CssValueOrigin> {
        self.recovery_origin.as_ref()
    }

    /// Recovery owned by this body, independent of its retained children.
    pub fn local_recovery_origins(&self) -> &[CssValueOrigin] {
        &self.local_recovery
    }

    /// Current local-first union; historical report diagnostics are separate.
    pub fn recovery_origins(&self) -> impl Iterator<Item = &CssValueOrigin> {
        RecoveryOrigins {
            pending: vec![RecoveryEvent::Body(self)],
        }
    }

    pub(crate) fn from_parts(
        items: Vec<CssSupportsTestItem>,
        local_recovery: Vec<CssValueOrigin>,
    ) -> Self {
        let recovery_origin = local_recovery.first().cloned().or_else(|| {
            children(&items).find_map(|child| child.recovery_origins().next().cloned())
        });
        Self {
            items,
            local_recovery,
            recovery_origin,
        }
    }

    /// Flatten declaration runs without allocating or changing source occurrences.
    pub fn children(&self) -> impl Iterator<Item = CssSupportsTestChildRef<'_>> {
        children(&self.items)
    }

    /// Rebuilds runs from checked occurrences. Only this receiver's body-local
    /// recovery persists; the descendant union follows the supplied child order.
    pub fn recompose_children(
        &self,
        children: &[CssSupportsTestChildRef<'_>],
    ) -> Result<Self, CssNamedSupportsConstructionError> {
        self.recompose_children_with_limits(children, CssComponentValueLimits::default())
    }
    /// One native component/depth context and counting token sink admit the joint
    /// borrowed payload before cloning. Counting retains no output/map and never
    /// reparses or supplies semantics for reconstruction. Failure is atomic.
    pub fn recompose_children_with_limits(
        &self,
        children: &[CssSupportsTestChildRef<'_>],
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssNamedSupportsConstructionError> {
        crate::named_supports_composition::admit(children, limits)?;
        let mut items = Vec::new();
        items.try_reserve(children.len()).map_err(|_| {
            CssComponentValueError::new(
                crate::CssComponentValueErrorKind::CapacityOverflow,
                children
                    .first()
                    .map_or(CssValueOrigin::Programmatic, |v| v.origin().clone()),
            )
        })?;
        let mut run = Vec::new();
        for child in children {
            match child {
                CssSupportsTestChildRef::Declaration(v) => run.push((*v).clone()),
                _ => {
                    if !run.is_empty() {
                        items.push(CssSupportsTestItem::Declarations(
                            CssSupportsTestDeclarations {
                                declarations: std::mem::take(&mut run),
                            },
                        ));
                    }
                    items.push(match child {
                        CssSupportsTestChildRef::QualifiedRule(v) => {
                            CssSupportsTestItem::QualifiedRule((*v).clone())
                        }
                        CssSupportsTestChildRef::AtRule(v) => {
                            CssSupportsTestItem::AtRule((*v).clone())
                        }
                        CssSupportsTestChildRef::Declaration(_) => unreachable!("run handled"),
                    });
                }
            }
        }
        if !run.is_empty() {
            items.push(CssSupportsTestItem::Declarations(
                CssSupportsTestDeclarations { declarations: run },
            ));
        }
        Ok(Self::from_parts(items, self.local_recovery.clone()))
    }

    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }

    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        self.write_to(&mut out)?;
        out.finish()
    }

    pub(crate) fn write_to(
        &self,
        out: &mut CssCanonicalBuilder,
    ) -> Result<(), CssComponentValueError> {
        enum Event<'a> {
            Body(&'a CssSupportsTestBody),
            Item(&'a CssSupportsTestItem),
            Close(&'a CssValueOrigin),
        }
        let mut pending = vec![Event::Body(self)];
        while let Some(event) = pending.pop() {
            match event {
                Event::Body(body) => pending.extend(body.items.iter().rev().map(Event::Item)),
                Event::Item(CssSupportsTestItem::Declarations(run)) => {
                    for declaration in &run.declarations {
                        out.push_components(&declaration.components)?;
                        out.push_grammar(CssCanonicalToken::Semicolon, &declaration.terminator)?;
                    }
                }
                Event::Item(CssSupportsTestItem::QualifiedRule(rule)) => {
                    out.push_components(rule.data.prelude.items())?;
                    out.push_grammar(CssCanonicalToken::OpenBrace, &rule.data.opening)?;
                    pending.push(Event::Close(&rule.data.closing));
                    pending.push(Event::Body(&rule.data.body));
                }
                Event::Item(CssSupportsTestItem::AtRule(rule)) => {
                    out.push_component(&rule.data.at_keyword)?;
                    out.push_components(rule.data.prelude.items())?;
                    if let Some(body) = &rule.data.body {
                        out.push_grammar(
                            CssCanonicalToken::OpenBrace,
                            rule.data.opening.as_ref().expect("block has opening"),
                        )?;
                        pending.push(Event::Close(&rule.data.closing));
                        pending.push(Event::Body(body));
                    } else {
                        out.push_grammar(CssCanonicalToken::Semicolon, &rule.data.closing)?;
                    }
                }
                Event::Close(origin) => {
                    out.push_grammar(CssCanonicalToken::CloseBrace, origin)?;
                }
            }
        }
        Ok(())
    }
}

/// A checked named supports definition with its non-rendering test body.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsConditionRule {
    data: Box<SupportsConditionRuleData>,
}

#[derive(Clone, Debug, PartialEq)]
struct SupportsConditionRuleData {
    name: CssSupportsConditionName,
    body: CssSupportsTestBody,
    origin: CssValueOrigin,
    opening: CssValueOrigin,
    closing: CssValueOrigin,
}

impl CssSupportsConditionRule {
    pub fn try_new(
        name: CssSupportsConditionName,
        body: CssSupportsTestBody,
    ) -> Result<Self, CssNamedSupportsConstructionError> {
        Self::try_new_with_limits(name, body, CssComponentValueLimits::default())
    }

    pub fn try_new_with_limits(
        name: CssSupportsConditionName,
        body: CssSupportsTestBody,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssNamedSupportsConstructionError> {
        if let Some(origin) = body.recovery_origin() {
            return Err(CssNamedSupportsConstructionError::RecoveredInput {
                origin: origin.clone(),
            });
        }
        let value = Self::parsed(
            name,
            body,
            CssValueOrigin::Programmatic,
            CssValueOrigin::Programmatic,
            CssValueOrigin::Programmatic,
        );
        let output = value.serialize_with_limit(limits.max_css_bytes())?;
        validate_emitted_budget(&output, limits, value.origin())?;
        Ok(value)
    }

    pub(crate) fn parsed(
        name: CssSupportsConditionName,
        body: CssSupportsTestBody,
        origin: CssValueOrigin,
        opening: CssValueOrigin,
        closing: CssValueOrigin,
    ) -> Self {
        Self {
            data: Box::new(SupportsConditionRuleData {
                name,
                body,
                origin,
                opening,
                closing,
            }),
        }
    }

    pub fn name(&self) -> &CssSupportsConditionName {
        &self.data.name
    }

    pub fn body(&self) -> &CssSupportsTestBody {
        &self.data.body
    }

    pub fn origin(&self) -> &CssValueOrigin {
        &self.data.origin
    }

    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }

    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }

    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        out.push_grammar(
            CssCanonicalToken::AtKeyword("supports-condition"),
            self.origin(),
        )?;
        out.push_grammar(CssCanonicalToken::Whitespace, &CssValueOrigin::Programmatic)?;
        out.push_component(self.name().component())?;
        out.push_grammar(CssCanonicalToken::OpenBrace, &self.data.opening)?;
        self.body().write_to(&mut out)?;
        out.push_grammar(CssCanonicalToken::CloseBrace, &self.data.closing)?;
        out.finish()
    }
}

fn validate_emitted_budget(
    output: &CssSerializedValue,
    limits: CssComponentValueLimits,
    fallback: &CssValueOrigin,
) -> Result<(), CssNamedSupportsConstructionError> {
    // Count the emitted syntax, including generated whitespace, semicolons,
    // and boundary comments. Reparse only for validation; the original tree
    // and every supplied or generated origin remain owned by the caller.
    parse_component_values_with_limits(output.as_css(), limits).map_err(|error| {
        let offset = crate::media::parsed_position(error.origin())
            .map_or(0, |position| position.byte_offset().value());
        CssNamedSupportsConstructionError::Component(CssComponentValueError::new(
            error.kind(),
            output
                .value_origin_at(offset)
                .cloned()
                .unwrap_or_else(|| fallback.clone()),
        ))
    })?;
    Ok(())
}

/// A failure to construct an unrecovered named supports definition or test body.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssNamedSupportsConstructionError {
    InvalidName { origin: CssValueOrigin },
    InvalidBodyGrammar { origin: CssValueOrigin },
    RecoveredInput { origin: CssValueOrigin },
    Component(CssComponentValueError),
}

impl CssNamedSupportsConstructionError {
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::InvalidName { origin }
            | Self::InvalidBodyGrammar { origin }
            | Self::RecoveredInput { origin } => origin,
            Self::Component(error) => error.origin(),
        }
    }
}

impl From<CssComponentValueError> for CssNamedSupportsConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        Self::Component(error)
    }
}

impl fmt::Display for CssNamedSupportsConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid named supports construction: {self:?}")
    }
}

impl std::error::Error for CssNamedSupportsConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            _ => None,
        }
    }
}

/// One inert direct child. Nested candidate bodies remain immutable CSS payload.
#[derive(Clone, Debug, PartialEq)]
pub enum CssSupportsTestChild {
    Declaration(CssSupportsTestDeclaration),
    QualifiedRule(CssSupportsQualifiedRuleTest),
    AtRule(CssSupportsAtRuleTest),
}

/// Borrowed direct-child projection, transparent to native serialization costs.
#[derive(Clone, Copy, Debug)]
pub enum CssSupportsTestChildRef<'a> {
    Declaration(&'a CssSupportsTestDeclaration),
    QualifiedRule(&'a CssSupportsQualifiedRuleTest),
    AtRule(&'a CssSupportsAtRuleTest),
}
impl CssSupportsTestChild {
    pub fn as_ref(&self) -> CssSupportsTestChildRef<'_> {
        match self {
            Self::Declaration(v) => CssSupportsTestChildRef::Declaration(v),
            Self::QualifiedRule(v) => CssSupportsTestChildRef::QualifiedRule(v),
            Self::AtRule(v) => CssSupportsTestChildRef::AtRule(v),
        }
    }
}
impl<'a> CssSupportsTestChildRef<'a> {
    pub fn origin(self) -> &'a CssValueOrigin {
        match self {
            Self::Declaration(v) => v.origin(),
            Self::QualifiedRule(v) => v.origin(),
            Self::AtRule(v) => v.origin(),
        }
    }
    pub(crate) fn recovery_origins(self) -> impl Iterator<Item = &'a CssValueOrigin> {
        RecoveryOrigins {
            pending: vec![RecoveryEvent::Child(self)],
        }
    }
}
fn children(items: &[CssSupportsTestItem]) -> impl Iterator<Item = CssSupportsTestChildRef<'_>> {
    items.iter().flat_map(|item| {
        let (declarations, candidate) = match item {
            CssSupportsTestItem::Declarations(v) => (v.declarations(), None),
            CssSupportsTestItem::QualifiedRule(v) => {
                (&[][..], Some(CssSupportsTestChildRef::QualifiedRule(v)))
            }
            CssSupportsTestItem::AtRule(v) => (&[][..], Some(CssSupportsTestChildRef::AtRule(v))),
        };
        declarations
            .iter()
            .map(CssSupportsTestChildRef::Declaration)
            .chain(candidate)
    })
}
/// Capture component closures at their retained candidate, not on its parent body.
pub(crate) fn retained_component_recovery(values: &[CssComponentValue]) -> Vec<CssValueOrigin> {
    let mut pending: Vec<_> = values.iter().rev().collect();
    let mut origins = Vec::new();
    while let Some(value) = pending.pop() {
        if let Some(origin) = value.implicit_termination_origin() {
            origins.push(origin.clone());
        }
        if let Some(children) = value.child_values() {
            pending.extend(children.items().iter().rev());
        }
    }
    origins
}

// Borrowed current union uses only traversal continuations, not cloned ancestor
// copies of every descendant fault. Body/run containers remain transparent.
enum RecoveryEvent<'a> {
    Body(&'a CssSupportsTestBody),
    Items(&'a [CssSupportsTestItem]),
    Declarations(&'a [CssSupportsTestDeclaration]),
    Child(CssSupportsTestChildRef<'a>),
    Origins(&'a [CssValueOrigin]),
}
struct RecoveryOrigins<'a> {
    pending: Vec<RecoveryEvent<'a>>,
}
impl<'a> Iterator for RecoveryOrigins<'a> {
    type Item = &'a CssValueOrigin;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.pending.pop() {
            match event {
                RecoveryEvent::Body(body) => {
                    self.pending.push(RecoveryEvent::Items(&body.items));
                    self.pending
                        .push(RecoveryEvent::Origins(&body.local_recovery));
                }
                RecoveryEvent::Items(items) => {
                    if let Some((first, rest)) = items.split_first() {
                        self.pending.push(RecoveryEvent::Items(rest));
                        match first {
                            CssSupportsTestItem::Declarations(run) => self
                                .pending
                                .push(RecoveryEvent::Declarations(run.declarations())),
                            CssSupportsTestItem::QualifiedRule(v) => self.pending.push(
                                RecoveryEvent::Child(CssSupportsTestChildRef::QualifiedRule(v)),
                            ),
                            CssSupportsTestItem::AtRule(v) => self
                                .pending
                                .push(RecoveryEvent::Child(CssSupportsTestChildRef::AtRule(v))),
                        }
                    }
                }
                RecoveryEvent::Declarations(values) => {
                    if let Some((first, rest)) = values.split_first() {
                        self.pending.push(RecoveryEvent::Declarations(rest));
                        self.pending.push(RecoveryEvent::Origins(&first.recovery));
                    }
                }
                RecoveryEvent::Child(child) => match child {
                    CssSupportsTestChildRef::Declaration(v) => {
                        self.pending.push(RecoveryEvent::Origins(&v.recovery))
                    }
                    CssSupportsTestChildRef::QualifiedRule(v) => {
                        self.pending.push(RecoveryEvent::Body(v.body()));
                        self.pending.push(RecoveryEvent::Origins(&v.data.recovery));
                    }
                    CssSupportsTestChildRef::AtRule(v) => {
                        if let Some(body) = v.body() {
                            self.pending.push(RecoveryEvent::Body(body));
                        }
                        self.pending.push(RecoveryEvent::Origins(&v.data.recovery));
                    }
                },
                RecoveryEvent::Origins(origins) => {
                    if let Some((first, rest)) = origins.split_first() {
                        self.pending.push(RecoveryEvent::Origins(rest));
                        return Some(first);
                    }
                }
            }
        }
        None
    }
}
