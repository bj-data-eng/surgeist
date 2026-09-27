//! Checked whole-descriptor environment deferral and strict reentry.

use std::fmt;

use super::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind, CssFontFaceDescriptorValue,
    CssPendingFontFaceDescriptorValue,
};
use crate::{
    CssComponentValueError, CssComponentValueErrorKind, CssComponentValueLimits,
    CssComponentValues, CssSerializedOrigin, CssSerializedValue, CssValueOrigin, Error, ErrorKind,
};

/// The first reason a checked descriptor or pending reentry failed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceValueErrorKind {
    Component(CssComponentValueErrorKind),
    Grammar(ErrorKind),
    RecoveredComponent,
    ResidualSubstitution,
}

/// A font-face descriptor failure mapped to original component provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFaceValueError {
    kind: Box<CssFontFaceValueErrorKind>,
    origin: Box<CssSerializedOrigin>,
    source: Option<Box<FontFaceValueErrorSource>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FontFaceValueErrorSource {
    Component(CssComponentValueError),
    Grammar(Error),
}

impl CssFontFaceValueError {
    #[must_use]
    pub fn kind(&self) -> &CssFontFaceValueErrorKind {
        &self.kind
    }

    #[must_use]
    pub fn origin(&self) -> &CssSerializedOrigin {
        &self.origin
    }

    fn component(error: CssComponentValueError) -> Self {
        Self {
            kind: Box::new(CssFontFaceValueErrorKind::Component(error.kind())),
            origin: Box::new(CssSerializedOrigin::Token(error.origin().clone())),
            source: Some(Box::new(FontFaceValueErrorSource::Component(error))),
        }
    }

    fn grammar(error: Error, serialized: &CssSerializedValue) -> Self {
        let origin = serialized
            .origin_at(error.position().byte_offset().value())
            .expect("font-face parser position stays inside serialized input or at EOF")
            .clone();
        Self {
            kind: Box::new(CssFontFaceValueErrorKind::Grammar(error.kind().clone())),
            origin: Box::new(origin),
            source: Some(Box::new(FontFaceValueErrorSource::Grammar(error))),
        }
    }

    fn implicit(origin: CssValueOrigin) -> Self {
        Self {
            kind: Box::new(CssFontFaceValueErrorKind::RecoveredComponent),
            origin: Box::new(CssSerializedOrigin::Token(origin)),
            source: None,
        }
    }

    fn residual(origin: CssValueOrigin) -> Self {
        Self {
            kind: Box::new(CssFontFaceValueErrorKind::ResidualSubstitution),
            origin: Box::new(CssSerializedOrigin::Token(origin)),
            source: None,
        }
    }
}

impl fmt::Display for CssFontFaceValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid font-face descriptor value: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for CssFontFaceValueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.source.as_deref()? {
            FontFaceValueErrorSource::Component(error) => Some(error),
            FontFaceValueErrorSource::Grammar(error) => Some(error),
        }
    }
}

impl CssAuthoredFontFaceDescriptorValue {
    /// Checks one complete descriptor without raw stylesheet recovery.
    ///
    /// A well-formed `env()` reference defers the whole descriptor; otherwise
    /// the supplied components must satisfy its ordinary grammar.
    pub fn try_from_components(
        kind: CssFontFaceDescriptorKind,
        values: CssComponentValues,
    ) -> Result<Self, CssFontFaceValueError> {
        Self::try_from_components_with_limits(kind, values, CssComponentValueLimits::default())
    }

    /// Checks one descriptor with explicit component and serialized-size limits.
    /// Recovered or implicitly closed components are rejected.
    pub fn try_from_components_with_limits(
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
            .serialize_with_limit(limits.max_css_bytes())
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
    pub fn reparse_after_substitution(
        &self,
        replacement: CssComponentValues,
    ) -> Result<CssFontFaceDescriptorValue, CssFontFaceValueError> {
        self.reparse_after_substitution_with_limits(replacement, CssComponentValueLimits::default())
    }

    /// Reparses substituted components with explicit resource limits.
    /// The same strict grammar and residual-substitution rules apply.
    pub fn reparse_after_substitution_with_limits(
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
        if let Some(origin) = crate::parser::first_substitution_origin(&replacement) {
            return Err(CssFontFaceValueError::residual(origin));
        }
        match CssAuthoredFontFaceDescriptorValue::try_from_components_with_limits(
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
