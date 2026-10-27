use crate::CssParseReport;
use crate::component::values::serialize::{CssCanonicalBuilder, CssCanonicalToken};
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValues,
    CssSerializedValue, CssValueOrigin,
};
use crate::declaration::CssImportance;
use crate::query::supports::CssSupportsConstructionError;
use crate::source::{
    CssRetainedSources, CssSourcePosition, CssSourceRetention, CssSourceRetentionError,
};
use std::ops::Range;
impl CssRetainedSources for CssSupportsTestBody {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.local_recovery.as_slice())?;
        sources.visit(&self.recovery_origin)?;
        for value in &self.items {
            match value {
                CssSupportsTestItem::Declarations(value) => {
                    for value in value.declarations() {
                        sources.visit(value)?;
                    }
                }
                CssSupportsTestItem::QualifiedRule(value) => sources.visit(value)?,
                CssSupportsTestItem::AtRule(value) => sources.visit(value)?,
            }
        }
        Ok(())
    }
}
impl CssRetainedSources for CssSupportsTestDeclaration {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.components())?;
        sources.visit(&self.terminator)?;
        sources.visit(self.recovery.as_slice())
    }
}
impl CssRetainedSources for CssSupportsQualifiedRuleTest {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(&self.data.prelude)?;
        sources.visit(&self.data.opening)?;
        sources.visit(&self.data.closing)?;
        sources.visit(self.data.recovery.as_slice())?;
        sources.visit(self.data.body.as_ref())
    }
}
impl CssRetainedSources for CssSupportsAtRuleTest {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(&self.data.at_keyword)?;
        sources.visit(&self.data.prelude)?;
        sources.visit(&self.data.opening)?;
        sources.visit(&self.data.closing)?;
        sources.visit(self.data.recovery.as_slice())?;
        if let Some(value) = &self.data.body {
            sources.visit(value.as_ref())?;
        }
        Ok(())
    }
}
impl CssRetainedSources for CssSupportsTestChild {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Declaration(value) => sources.visit(value),
            Self::QualifiedRule(value) => sources.visit(value),
            Self::AtRule(value) => sources.visit(value),
        }
    }
}

/// A checked, decoded extension name with its original identifier token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSupportsConditionName {
    component: CssComponentValue,
    name: String,
}

impl CssSupportsConditionName {
    /// Constructs a name from a decoded identifier, escaping it as necessary.
    ///
    /// # Errors
    ///
    /// Returns [`CssSupportsConstructionError`] when `name` is not a valid identifier, or is not a custom `--` extension name.
    pub fn try_new(name: impl Into<String>) -> Result<Self, CssSupportsConstructionError> {
        Self::try_from_component(CssComponentValue::try_ident(name)?)
    }

    /// Constructs a name from an identifier component.
    ///
    /// # Errors
    ///
    /// Returns [`CssSupportsConstructionError`] when the component is not an identifier that starts with `--`.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssSupportsConstructionError> {
        let Some(name) = crate::query::media::custom_media::extension_name(&component) else {
            return Err(CssSupportsConstructionError::from(
                crate::query::supports::construction::SupportsConstructionKind::InvalidName {
                    origin: component.origin().clone(),
                },
            ));
        };
        Ok(Self {
            name: name.to_owned(),
            component,
        })
    }

    /// Returns the decoded name, including its leading `--`.
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Returns the original identifier component the name was built from.
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    /// Returns the origin of the identifier component.
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }

    /// Returns the source position of the identifier, or `None` when its origin was not parsed from source.
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
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
    /// Returns the property name text as authored.
    pub fn property(&self) -> &str {
        &self.name
    }

    /// Returns the component holding the property name.
    pub fn property_component(&self) -> &CssComponentValue {
        &self.components[self.property_index]
    }

    /// Returns the components of the declaration value, without the property name, colon or importance.
    pub fn value_components(&self) -> &[CssComponentValue] {
        &self.components[self.value_range.clone()]
    }

    /// Returns all components of the declaration, including the property name and any importance.
    pub fn components(&self) -> &[CssComponentValue] {
        &self.components
    }

    /// Returns the `!important` components, or `None` when the declaration has no importance.
    pub fn importance_components(&self) -> Option<&[CssComponentValue]> {
        self.importance_range
            .as_ref()
            .map(|range| &self.components[range.clone()])
    }

    /// Returns [`CssImportance::Important`] when the declaration carries `!important`, and [`CssImportance::Normal`] otherwise.
    pub fn importance(&self) -> CssImportance {
        if self.importance_range.is_some() {
            CssImportance::Important
        } else {
            CssImportance::Normal
        }
    }

    /// Returns the origin of the property-name component.
    pub fn origin(&self) -> &CssValueOrigin {
        self.property_component().origin()
    }

    /// Returns the source position of the property name, or `None` when its origin was not parsed from source.
    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
    }
}

/// A nonempty, ordered declaration run within a feature-test body.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsTestDeclarations {
    pub(crate) declarations: Vec<CssSupportsTestDeclaration>,
}

impl CssSupportsTestDeclarations {
    /// Returns the declarations of the run in authored order.
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

    /// Returns the prelude of the qualified rule as authored component values.
    pub fn prelude(&self) -> &CssComponentValues {
        &self.data.prelude
    }

    /// Returns the test body inside the rule's braces.
    pub fn body(&self) -> &CssSupportsTestBody {
        &self.data.body
    }

    /// Returns the origin of the first non-trivia prelude component, or of the opening brace when the prelude has none.
    pub fn origin(&self) -> &CssValueOrigin {
        self.data
            .prelude
            .items()
            .iter()
            .find(|value| !crate::query::supports::construction::trivia(value))
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

    /// Returns the decoded at-rule name, without the `@`.
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Returns the at-keyword component with its original spelling and origin.
    pub fn at_keyword(&self) -> &CssComponentValue {
        &self.data.at_keyword
    }

    /// Returns the prelude of the at-rule as authored component values.
    pub fn prelude(&self) -> &CssComponentValues {
        &self.data.prelude
    }

    /// Returns the test body in braces, or `None` when the at-rule ends with a semicolon and has no block.
    pub fn body(&self) -> Option<&CssSupportsTestBody> {
        self.data.body.as_deref()
    }

    /// Returns the origin of the at-keyword.
    pub fn origin(&self) -> &CssValueOrigin {
        self.data.at_keyword.origin()
    }
}

/// One ordered generic feature-test candidate or nonempty declaration run.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssSupportsTestItem {
    /// A nonempty run of declarations.
    Declarations(CssSupportsTestDeclarations),
    /// A generic qualified rule: a prelude and a braced test body.
    QualifiedRule(CssSupportsQualifiedRuleTest),
    /// A generic at-rule, including unknown future at-rules.
    AtRule(CssSupportsAtRuleTest),
}

/// An authored, non-rendering feature-test body.
///
/// # Specified serialization
///
/// Serializes test candidates without selecting feature truth values. A standalone body has direct
/// children at depth zero; containers/runs are transparent. Use a named graph input to include the
/// outer wrapper depth.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsTestBody {
    pub(crate) items: Vec<CssSupportsTestItem>,
    pub(crate) local_recovery: Vec<CssValueOrigin>,
    pub(crate) recovery_origin: Option<CssValueOrigin>,
}

impl CssSupportsTestBody {
    /// Constructs a test body from authored component values.
    ///
    /// # Errors
    ///
    /// Returns [`CssSupportsConstructionError`] when the components contain an implicitly closed component, do not form a test body, or violate `limits`.
    pub fn try_from_components(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssSupportsConstructionError> {
        values.validate_with_limits(limits)?;
        if let Some(origin) = values.first_implicit_origin() {
            return Err(CssSupportsConstructionError::from(
                crate::query::supports::construction::SupportsConstructionKind::RecoveredInput {
                    origin: origin.clone(),
                },
            ));
        }
        let body = crate::parser::named_supports::construct_test_body(values, limits)?;
        let output = body.serialize_tokens(limits)?;
        validate_emitted_budget(&output, limits, &CssValueOrigin::Programmatic)?;
        Ok(body)
    }

    /// Returns the ordered candidates and declaration runs of the body.
    pub fn items(&self) -> &[CssSupportsTestItem] {
        &self.items
    }

    /// Returns the origin of the recovered input that makes this body unfit for a definition, or `None` when none was recovered.
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
    ///
    /// One native component/depth context and counting token sink admit the joint
    /// borrowed payload before cloning. Counting retains no output/map and never
    /// reparses or supplies semantics for reconstruction. Failure is atomic.
    pub fn recompose_children(
        &self,
        children: &[CssSupportsTestChildRef<'_>],
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssSupportsConstructionError> {
        crate::query::supports::named_supports_composition::admit(children, limits)?;
        let mut items = Vec::new();
        items.try_reserve(children.len()).map_err(|_| {
            CssComponentValueError::new(
                crate::component::values::CssComponentValueErrorKind::CapacityOverflow,
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

    /// `limits.max_css_bytes()` caps the output, including inserted syntax; the other component
    /// limits do not apply.
    pub fn serialize_tokens(
        &self,
        limits: crate::component::CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let max_css_bytes = limits.max_css_bytes();
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

pub(crate) fn validate_emitted_budget(
    output: &CssSerializedValue,
    limits: CssComponentValueLimits,
    fallback: &CssValueOrigin,
) -> Result<(), CssSupportsConstructionError> {
    // Count the emitted syntax, including generated whitespace, semicolons,
    // and boundary comments. Reparse only for validation; the original tree
    // and every supplied or generated origin remain owned by the caller.
    crate::component::values::parse_component_values_with_limits(output.as_css(), limits).map_err(
        |error| {
            let offset = crate::query::media::construction::parsed_position(error.origin())
                .map_or(0, |position| position.byte_offset().value());
            CssSupportsConstructionError::from(
                crate::query::supports::construction::SupportsConstructionKind::Component(
                    CssComponentValueError::new(
                        error.kind,
                        output
                            .value_origin_at(offset)
                            .cloned()
                            .unwrap_or_else(|| fallback.clone()),
                    ),
                ),
            )
        },
    )?;
    Ok(())
}

/// One inert direct child. Nested candidate bodies remain immutable CSS payload.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssSupportsTestChild {
    /// A declaration.
    Declaration(CssSupportsTestDeclaration),
    /// A generic qualified rule.
    QualifiedRule(CssSupportsQualifiedRuleTest),
    /// A generic at-rule.
    AtRule(CssSupportsAtRuleTest),
}

/// Borrowed direct-child projection, transparent to native serialization costs.
///
/// # Specified serialization
///
/// Standalone child root is depth zero. Nested payload/components and the complete output share the
/// same native writer allowances.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssSupportsTestChildRef<'a> {
    /// The borrowed declaration.
    Declaration(&'a CssSupportsTestDeclaration),
    /// The borrowed generic qualified rule.
    QualifiedRule(&'a CssSupportsQualifiedRuleTest),
    /// The borrowed generic at-rule.
    AtRule(&'a CssSupportsAtRuleTest),
}
impl CssSupportsTestChild {
    /// Returns the child as a borrowed view.
    pub fn as_ref(&self) -> CssSupportsTestChildRef<'_> {
        match self {
            Self::Declaration(v) => CssSupportsTestChildRef::Declaration(v),
            Self::QualifiedRule(v) => CssSupportsTestChildRef::QualifiedRule(v),
            Self::AtRule(v) => CssSupportsTestChildRef::AtRule(v),
        }
    }
}
impl<'a> CssSupportsTestChildRef<'a> {
    /// Returns the origin of the child.
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
impl CssSupportsTestChild {
    /// Admits exactly one flattened direct child with no discarded candidate or
    /// structural recovery. Empty semicolons/trivia are ignored at every depth.
    ///
    /// The parser supplies the cumulative component limits.
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> CssParseReport<Option<CssSupportsTestChild>> {
        crate::parser::named_supports::parse_supports_test_child_with_limits(
            source.as_ref(),
            parser.component_limits(),
        )
    }
}
impl CssSupportsTestBody {
    /// Admits complete genuine unbraced named-test contents. Recovered syntax is
    /// `Some` with diagnostics; typed resource failure is `None`, never a clearing.
    ///
    /// The parser supplies the cumulative component limits.
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> CssParseReport<Option<CssSupportsTestBody>> {
        crate::parser::named_supports::parse_supports_test_contents_with_limits(
            source.as_ref(),
            parser.component_limits(),
        )
    }
}
