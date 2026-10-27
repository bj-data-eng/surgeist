//! Authored named supports definitions and their feature-test candidates.

use crate::component::values::serialize::{CssCanonicalBuilder, CssCanonicalToken};
use crate::component::{
    CssComponentValueError, CssComponentValueLimits, CssSerializedValue, CssValueOrigin,
};
use crate::query::supports::CssSupportsConstructionError;
use crate::query::supports::named_supports::validate_emitted_budget;
use crate::query::supports::{CssSupportsConditionName, CssSupportsTestBody};
use crate::source::{
    CssRetainedSources, CssSourcePosition, CssSourceRetention, CssSourceRetentionError,
};

impl CssRetainedSources for CssSupportsConditionRule {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.data.name.component())?;
        sources.visit(&self.data.origin)?;
        sources.visit(&self.data.opening)?;
        sources.visit(&self.data.closing)?;
        sources.visit(&self.data.body)
    }
}

/// A checked named supports definition with its non-rendering test body.
///
/// # Specified serialization
///
/// Canonical specified definition text, including an explicit block closure.
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
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssSupportsConstructionError> {
        if let Some(origin) = body.recovery_origin() {
            return Err(CssSupportsConstructionError::from(
                crate::query::supports::construction::SupportsConstructionKind::RecoveredInput {
                    origin: origin.clone(),
                },
            ));
        }
        let value = Self::parsed(
            name,
            body,
            CssValueOrigin::Programmatic,
            CssValueOrigin::Programmatic,
            CssValueOrigin::Programmatic,
        );
        let output = value.serialize_tokens(limits)?;
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
        crate::query::media::construction::parsed_position(self.origin())
    }

    /// `limits.max_css_bytes()` caps the output, including inserted syntax; the other component
    /// limits do not apply.
    pub fn serialize_tokens(
        &self,
        limits: crate::component::CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let max_css_bytes = limits.max_css_bytes();
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
