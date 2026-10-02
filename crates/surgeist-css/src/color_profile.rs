//! Authored color-profile definitions; resource loading and substitution belong to hosts.
use crate::{
    CssColorProfileComponentName, CssColorProfileName, CssComponentValueError,
    CssComponentValueErrorKind, CssComponentValueLimits, CssComponentValues, CssSerializedOrigin,
    CssSerializedValue, CssSourcePosition, CssSubstitutionDependentValue, CssUrl, CssValueOrigin,
    Error, ErrorKind,
};
use std::fmt;

/// The custom case-sensitive name or reserved device profile identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssColorProfileRuleName {
    Custom(CssColorProfileName),
    DeviceCmyk,
}

/// The four authored intents. The default is the specified initial, not an occurrence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CssColorProfileRenderingIntent {
    #[default]
    RelativeColorimetric,
    AbsoluteColorimetric,
    Perceptual,
    Saturation,
}
impl CssColorProfileRenderingIntent {
    #[must_use]
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::RelativeColorimetric => "relative-colorimetric",
            Self::AbsoluteColorimetric => "absolute-colorimetric",
            Self::Perceptual => "perceptual",
            Self::Saturation => "saturation",
        }
    }
    pub(crate) fn from_css_name(name: &str) -> Option<Self> {
        [
            Self::RelativeColorimetric,
            Self::AbsoluteColorimetric,
            Self::Perceptual,
            Self::Saturation,
        ]
        .into_iter()
        .find(|intent| intent.css_name().eq_ignore_ascii_case(name))
    }
}

/// The three descriptor productions in Color 5 §5.3.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorProfileDescriptorKind {
    Src,
    RenderingIntent,
    Components,
}
impl CssColorProfileDescriptorKind {
    #[must_use]
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::Src => "src",
            Self::RenderingIntent => "rendering-intent",
            Self::Components => "components",
        }
    }
    pub(crate) fn from_css_name(name: &str) -> Option<Self> {
        [Self::Src, Self::RenderingIntent, Self::Components]
            .into_iter()
            .find(|kind| kind.css_name().eq_ignore_ascii_case(name))
    }
}

fn reject_recovered(components: &CssComponentValues) -> Result<(), CssColorProfileValueError> {
    if let Some(origin) = components.first_implicit_origin() {
        return Err(CssColorProfileValueError {
            kind: Box::new(CssColorProfileValueErrorKind::RecoveredComponent),
            origin: Box::new(CssSerializedOrigin::Token(origin.clone())),
            source: None,
        });
    }
    Ok(())
}

/// A checked component-native descriptor construction or reentry failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorProfileValueErrorKind {
    Component(CssComponentValueErrorKind),
    RecoveredComponent,
    Grammar(ErrorKind),
    ResidualSubstitution,
    NotPending,
}

/// A failure mapped to the supplied component origins, never fabricated source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorProfileValueError {
    kind: Box<CssColorProfileValueErrorKind>,
    origin: Box<CssSerializedOrigin>,
    source: Option<Box<ProfileValueErrorSource>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProfileValueErrorSource {
    Component(CssComponentValueError),
    Grammar(Error),
}

impl CssColorProfileValueError {
    #[must_use]
    pub fn kind(&self) -> &CssColorProfileValueErrorKind {
        &self.kind
    }

    #[must_use]
    pub fn origin(&self) -> &CssSerializedOrigin {
        &self.origin
    }

    fn from_component(error: CssComponentValueError) -> Self {
        Self {
            kind: Box::new(CssColorProfileValueErrorKind::Component(error.kind())),
            origin: Box::new(CssSerializedOrigin::Token(error.origin().clone())),
            source: Some(Box::new(ProfileValueErrorSource::Component(error))),
        }
    }

    fn from_grammar(error: Error, serialized: &CssSerializedValue) -> Self {
        let origin = serialized
            .origin_at(error.position().byte_offset().value())
            .expect("profile parser reports a cursor inside its serialized input or at EOF")
            .clone();
        Self {
            kind: Box::new(CssColorProfileValueErrorKind::Grammar(error.kind().clone())),
            origin: Box::new(origin),
            source: Some(Box::new(ProfileValueErrorSource::Grammar(error))),
        }
    }

    fn residual(origin: CssValueOrigin) -> Self {
        Self {
            kind: Box::new(CssColorProfileValueErrorKind::ResidualSubstitution),
            origin: Box::new(CssSerializedOrigin::Token(origin)),
            source: None,
        }
    }

    fn not_pending() -> Self {
        Self {
            kind: Box::new(CssColorProfileValueErrorKind::NotPending),
            origin: Box::new(CssSerializedOrigin::End(None)),
            source: None,
        }
    }
}

impl fmt::Display for CssColorProfileValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid color-profile descriptor value: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for CssColorProfileValueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.source.as_deref()? {
            ProfileValueErrorSource::Component(error) => Some(error),
            ProfileValueErrorSource::Grammar(error) => Some(error),
        }
    }
}

/// Borrowed checked descriptor semantics. Components retain the exact authored syntax.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColorProfileDescriptorValueRef<'a> {
    Src(&'a CssUrl),
    RenderingIntent(CssColorProfileRenderingIntent),
    Components(&'a [CssColorProfileComponentName]),
    Pending(&'a CssSubstitutionDependentValue),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssColorProfileDescriptorData {
    Src(CssUrl),
    RenderingIntent(CssColorProfileRenderingIntent),
    Components(Vec<CssColorProfileComponentName>),
    Pending(CssSubstitutionDependentValue),
}

/// One checked whole descriptor value with original component provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorProfileDescriptorValue {
    kind: CssColorProfileDescriptorKind,
    components: CssComponentValues,
    data: CssColorProfileDescriptorData,
}

impl CssColorProfileDescriptorValue {
    /// Checks the selected grammar while retaining mixed or programmatic origins.
    pub fn try_new(
        kind: CssColorProfileDescriptorKind,
        components: CssComponentValues,
    ) -> Result<Self, CssColorProfileValueError> {
        Self::try_new_with_limits(kind, components, CssComponentValueLimits::default())
    }

    /// Uses the same component and output-byte limits as shared component serialization.
    pub fn try_new_with_limits(
        kind: CssColorProfileDescriptorKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssColorProfileValueError> {
        components
            .validate_with_limits(limits)
            .map_err(CssColorProfileValueError::from_component)?;
        reject_recovered(&components)?;
        let serialized = components
            .serialize_with_limit(limits.max_css_bytes())
            .map_err(CssColorProfileValueError::from_component)?;
        let data = crate::parser::color_profile::construct_descriptor_value(
            kind,
            &components,
            &serialized,
        )
        .map_err(|error| CssColorProfileValueError::from_grammar(error, &serialized))?;
        Ok(Self {
            kind,
            components,
            data,
        })
    }

    /// Rechecks caller-supplied replacement components without performing substitution.
    /// Residual decoded `var()` or `env()` anywhere in the tree is rejected first.
    pub fn reparse_after_substitution(
        &self,
        replacement: CssComponentValues,
    ) -> Result<Self, CssColorProfileValueError> {
        self.reparse_after_substitution_with_limits(replacement, CssComponentValueLimits::default())
    }

    pub fn reparse_after_substitution_with_limits(
        &self,
        replacement: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssColorProfileValueError> {
        if !matches!(self.data, CssColorProfileDescriptorData::Pending(_)) {
            return Err(CssColorProfileValueError::not_pending());
        }
        replacement
            .validate_with_limits(limits)
            .map_err(CssColorProfileValueError::from_component)?;
        reject_recovered(&replacement)?;
        if let Some(origin) = crate::parser::first_substitution_origin(&replacement) {
            return Err(CssColorProfileValueError::residual(origin));
        }
        Self::try_new_with_limits(self.kind, replacement, limits)
    }

    pub(crate) fn from_parts(
        kind: CssColorProfileDescriptorKind,
        components: CssComponentValues,
        data: CssColorProfileDescriptorData,
    ) -> Self {
        let matching_kind = matches!(
            (&data, kind),
            (
                CssColorProfileDescriptorData::Src(_),
                CssColorProfileDescriptorKind::Src
            ) | (
                CssColorProfileDescriptorData::RenderingIntent(_),
                CssColorProfileDescriptorKind::RenderingIntent
            ) | (
                CssColorProfileDescriptorData::Components(_),
                CssColorProfileDescriptorKind::Components
            ) | (CssColorProfileDescriptorData::Pending(_), _)
        );
        debug_assert!(matching_kind);
        Self {
            kind,
            components,
            data,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssColorProfileDescriptorKind {
        self.kind
    }

    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }

    #[must_use]
    pub fn view(&self) -> CssColorProfileDescriptorValueRef<'_> {
        match &self.data {
            CssColorProfileDescriptorData::Src(value) => {
                CssColorProfileDescriptorValueRef::Src(value)
            }
            CssColorProfileDescriptorData::RenderingIntent(value) => {
                CssColorProfileDescriptorValueRef::RenderingIntent(*value)
            }
            CssColorProfileDescriptorData::Components(value) => {
                CssColorProfileDescriptorValueRef::Components(value)
            }
            CssColorProfileDescriptorData::Pending(value) => {
                CssColorProfileDescriptorValueRef::Pending(value)
            }
        }
    }
}

/// One ordered descriptor occurrence; checked construction invents no source position.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorProfileDescriptor {
    value: CssColorProfileDescriptorValue,
    position: Option<CssSourcePosition>,
}

impl CssColorProfileDescriptor {
    #[must_use]
    pub const fn new(value: CssColorProfileDescriptorValue) -> Self {
        Self {
            value,
            position: None,
        }
    }

    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    #[must_use]
    pub const fn value(&self) -> &CssColorProfileDescriptorValue {
        &self.value
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// One authored global definition, preserving duplicates and absence.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorProfileRule {
    name: CssColorProfileRuleName,
    descriptors: Vec<CssColorProfileDescriptor>,
    position: Option<CssSourcePosition>,
}
impl CssColorProfileRule {
    #[must_use]
    pub const fn new(
        name: CssColorProfileRuleName,
        descriptors: Vec<CssColorProfileDescriptor>,
    ) -> Self {
        Self {
            name,
            descriptors,
            position: None,
        }
    }
    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }
    #[must_use]
    pub const fn name(&self) -> &CssColorProfileRuleName {
        &self.name
    }
    #[must_use]
    pub fn descriptors(&self) -> &[CssColorProfileDescriptor] {
        &self.descriptors
    }
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
    /// Last retained valid occurrence; no initial value or profile selection is fabricated.
    #[must_use]
    pub fn effective(
        &self,
        kind: CssColorProfileDescriptorKind,
    ) -> Option<&CssColorProfileDescriptorValue> {
        self.descriptors
            .iter()
            .rev()
            .find(|descriptor| descriptor.value().kind() == kind)
            .map(CssColorProfileDescriptor::value)
    }
}
