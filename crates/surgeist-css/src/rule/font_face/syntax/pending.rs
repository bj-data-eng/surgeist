//! Checked whole-descriptor environment deferral and strict reentry.

use crate::component::{
    CssComponentValueError, CssComponentValueLimits, CssComponentValues, CssSerializedValue,
    CssValueOrigin,
};
use crate::diagnostic::CssError;
use crate::rule::descriptor_values::DescriptorValueFailure;
use crate::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind, CssFontFaceDescriptorValue,
    CssPendingFontFaceDescriptorValue,
};

/// A checked `@font-face` descriptor or pending-reentry failure mapped to original
/// component provenance.
///
/// The supplied components were not admitted ([`Self::component_error`]), the descriptor
/// grammar rejected them ([`Self::diagnostic`]), or reentry was refused; the component
/// error or diagnostic is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFaceValueError {
    failure: DescriptorValueFailure,
}

crate::rule::counter_style::descriptor_values::descriptor_value_api!(
    CssFontFaceValueError,
    "font-face"
);

impl CssFontFaceValueError {
    fn component(error: CssComponentValueError) -> Self {
        DescriptorValueFailure::component(error).into()
    }

    fn grammar(error: CssError, serialized: &CssSerializedValue) -> Self {
        DescriptorValueFailure::grammar(error, serialized).into()
    }

    fn implicit(origin: CssValueOrigin) -> Self {
        DescriptorValueFailure::recovered(origin).into()
    }

    fn residual(origin: CssValueOrigin) -> Self {
        DescriptorValueFailure::residual(origin).into()
    }
}

impl CssAuthoredFontFaceDescriptorValue {
    /// Checks one complete descriptor without raw stylesheet recovery.
    ///
    /// A well-formed `env()` reference defers the whole descriptor; otherwise
    /// the supplied components must satisfy its ordinary grammar.
    ///
    /// Checks one descriptor with explicit component and serialized-size limits.
    /// Recovered or implicitly closed components are rejected.
    pub fn try_from_components(
        kind: CssFontFaceDescriptorKind,
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFaceValueError> {
        values
            .validate_with_limits(limits)
            .map_err(CssFontFaceValueError::component)?;
        if let Some(origin) = values.first_implicit_origin() {
            return Err(CssFontFaceValueError::implicit(origin.clone()));
        }
        let serialized = values
            .serialize_tokens(limits)
            .map_err(CssFontFaceValueError::component)?;
        let (value, recovered) = crate::parser::font_face::construct_font_face_descriptor_value(
            kind,
            &values,
            &serialized,
        )
        .map_err(|error| CssFontFaceValueError::grammar(error, &serialized))?;
        if let Some(error) = recovered {
            return Err(CssFontFaceValueError::grammar(error, &serialized));
        }
        Ok(value)
    }
}

impl CssPendingFontFaceDescriptorValue {
    /// Reparses a fully substituted descriptor and returns only an ordinary value.
    ///
    /// Callers perform substitution before this step. Residual `var()` or `env()`
    /// references, recovery, and implicit closures are errors.
    ///
    /// Reparses substituted components with explicit resource limits.
    /// The same strict grammar and residual-substitution rules apply.
    pub fn reparse_after_substitution(
        &self,
        replacement: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<CssFontFaceDescriptorValue, CssFontFaceValueError> {
        replacement
            .validate_with_limits(limits)
            .map_err(CssFontFaceValueError::component)?;
        if let Some(origin) = replacement.first_implicit_origin() {
            return Err(CssFontFaceValueError::implicit(origin.clone()));
        }
        if let Some(origin) = crate::parser::variables::first_substitution_origin(&replacement) {
            return Err(CssFontFaceValueError::residual(origin));
        }
        match CssAuthoredFontFaceDescriptorValue::try_from_components(
            self.kind,
            replacement,
            limits,
        )? {
            CssAuthoredFontFaceDescriptorValue::Ordinary(value) => Ok(value),
            CssAuthoredFontFaceDescriptorValue::Pending(_) => {
                unreachable!("residual substitutions were rejected before parsing")
            }
        }
    }
}
