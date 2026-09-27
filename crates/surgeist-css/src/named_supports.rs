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
    pub(crate) prelude: CssComponentValues,
    pub(crate) body: Box<CssSupportsTestBody>,
    pub(crate) opening: CssValueOrigin,
    pub(crate) closing: CssValueOrigin,
}

impl CssSupportsQualifiedRuleTest {
    pub fn prelude(&self) -> &CssComponentValues {
        &self.prelude
    }

    pub fn body(&self) -> &CssSupportsTestBody {
        &self.body
    }

    pub fn origin(&self) -> &CssValueOrigin {
        self.prelude
            .items()
            .iter()
            .find(|value| !crate::supports::trivia(value))
            .map_or(&self.opening, CssComponentValue::origin)
    }
}

/// A generic at-rule test, including unknown future at-rules.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsAtRuleTest {
    data: Box<SupportsAtRuleTestData>,
}

#[derive(Clone, Debug, PartialEq)]
struct SupportsAtRuleTestData {
    name: String,
    at_keyword: CssComponentValue,
    prelude: CssComponentValues,
    body: Option<Box<CssSupportsTestBody>>,
    opening: Option<CssValueOrigin>,
    closing: CssValueOrigin,
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
        Self {
            data: Box::new(SupportsAtRuleTestData {
                name,
                at_keyword,
                prelude,
                body,
                opening,
                closing,
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
                    out.push_components(rule.prelude.items())?;
                    out.push_grammar(CssCanonicalToken::OpenBrace, &rule.opening)?;
                    pending.push(Event::Close(&rule.closing));
                    pending.push(Event::Body(&rule.body));
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
