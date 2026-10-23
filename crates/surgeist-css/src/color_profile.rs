//! Authored color-profile definitions; resource loading and substitution belong to hosts.
use crate::{
    CssColorProfileComponentName, CssColorProfileName, CssComponentValueError,
    CssComponentValueLimits, CssComponentValues, CssError, CssSerializedValue, CssSourcePosition,
    CssSubstitutionDependentValue, CssUrl, CssValueOrigin,
    descriptor_values::DescriptorValueFailure,
};

/// The custom case-sensitive name or reserved device profile identity.
///
/// External matches require a wildcard under the crate's public enum policy:
///
/// ```compile_fail,E0004
/// use surgeist_css::CssColorProfileRuleName;
///
/// fn classify(name: CssColorProfileRuleName) -> usize {
///     match name {
///         CssColorProfileRuleName::Custom(_) => 0,
///         CssColorProfileRuleName::DeviceCmyk => 1,
///     }
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorProfileRuleName {
    Custom(CssColorProfileName),
    DeviceCmyk,
}

/// The four authored intents. The default is the specified initial, not an occurrence.
///
/// External matches require a wildcard under the crate's public enum policy:
///
/// ```compile_fail,E0004
/// use surgeist_css::CssColorProfileRenderingIntent;
///
/// fn classify(intent: CssColorProfileRenderingIntent) -> usize {
///     match intent {
///         CssColorProfileRenderingIntent::RelativeColorimetric => 0,
///         CssColorProfileRenderingIntent::AbsoluteColorimetric => 1,
///         CssColorProfileRenderingIntent::Perceptual => 2,
///         CssColorProfileRenderingIntent::Saturation => 3,
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
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
        return Err(DescriptorValueFailure::recovered(origin.clone()).into());
    }
    Ok(())
}

/// A checked component-native `@color-profile` descriptor construction or reentry failure.
///
/// The supplied components were not admitted ([`Self::component_error`]), the descriptor
/// grammar rejected them ([`Self::diagnostic`]), or reentry was refused; the component
/// error or diagnostic is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorProfileValueError {
    failure: DescriptorValueFailure,
}

crate::descriptor_values::descriptor_value_api!(CssColorProfileValueError, "color-profile");

impl CssColorProfileValueError {
    fn from_component(error: CssComponentValueError) -> Self {
        DescriptorValueFailure::component(error).into()
    }

    fn from_grammar(error: CssError, serialized: &CssSerializedValue) -> Self {
        DescriptorValueFailure::grammar(error, serialized).into()
    }

    fn residual(origin: CssValueOrigin) -> Self {
        DescriptorValueFailure::residual(origin).into()
    }

    fn not_pending() -> Self {
        DescriptorValueFailure::not_pending().into()
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
///
/// # Specified serialization
///
/// Emits ordinary checked grammar or symbolic pending tokens without substitution.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorProfileDescriptorValue {
    kind: CssColorProfileDescriptorKind,
    components: CssComponentValues,
    data: CssColorProfileDescriptorData,
}

impl CssColorProfileDescriptorValue {
    /// Checks the selected grammar while retaining mixed or programmatic origins.
    ///
    /// Uses the same component and output-byte limits as shared component serialization.
    pub fn try_new(
        kind: CssColorProfileDescriptorKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssColorProfileValueError> {
        components
            .validate_with_limits(limits)
            .map_err(CssColorProfileValueError::from_component)?;
        reject_recovered(&components)?;
        let serialized = components
            .serialize_tokens(limits)
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
        Self::try_new(self.kind, replacement, limits)
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
///
/// # Specified serialization
///
/// Emits one name and value, without the enclosing list's semicolon.
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
///
/// # Specified serialization
///
/// Emits every retained occurrence in authored order, including empty definitions.
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
