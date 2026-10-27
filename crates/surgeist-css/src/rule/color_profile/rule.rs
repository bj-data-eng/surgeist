//! Authored color-profile definitions; resource loading and substitution belong to hosts.
use crate::color::{CssColorProfileComponentName, CssColorProfileName};
use crate::component::{
    CssComponentValueError, CssComponentValueLimits, CssComponentValues, CssSerializedValue,
    CssValueOrigin,
};
use crate::declaration::CssSubstitutionDependentValue;
use crate::diagnostic::CssError;
use crate::rule::descriptor_values::DescriptorValueFailure;
use crate::source::CssSourcePosition;
use crate::values::common::CssUrl;

/// The custom case-sensitive name or reserved device profile identity.
///
/// External matches require a wildcard under the crate's public enum policy:
///
/// ```compile_fail,E0004
/// use surgeist_css::rule::color_profile::CssColorProfileRuleName;
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
    /// A custom `<dashed-ident>` profile name; names are case-sensitive.
    Custom(CssColorProfileName),
    /// The reserved `device-cmyk` name, which defines the profile used to resolve `device-cmyk()` colors.
    DeviceCmyk,
}

/// The four authored intents. The default is the specified initial, not an occurrence.
///
/// External matches require a wildcard under the crate's public enum policy:
///
/// ```compile_fail,E0004
/// use surgeist_css::rule::color_profile::CssColorProfileRenderingIntent;
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
    /// `relative-colorimetric`: leaves in-gamut source colors unchanged relative to the media white points (the initial value).
    #[default]
    RelativeColorimetric,
    /// `absolute-colorimetric`: leaves in-gamut source colors unchanged relative to the adopted white, without white-point or black-point matching.
    AbsoluteColorimetric,
    /// `perceptual`: re-optimizes the appearance of colors for the destination medium, preserving the overall artistic intent.
    Perceptual,
    /// `saturation`: preserves the relative saturation (chroma) of the source colors.
    Saturation,
}
impl CssColorProfileRenderingIntent {
    /// Returns the CSS spelling of the intent, such as `relative-colorimetric`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
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
        .find(|intent| intent.as_str().eq_ignore_ascii_case(name))
    }
}

/// The three descriptor productions in Color 5 §5.3.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorProfileDescriptorKind {
    /// The `src` descriptor, which gives the URL of the profile data.
    Src,
    /// The `rendering-intent` descriptor, which chooses one of the intents contained in the profile.
    RenderingIntent,
    /// The `components` descriptor, which names the profile's color components in order.
    Components,
}
impl CssColorProfileDescriptorKind {
    /// Returns the CSS descriptor name, such as `rendering-intent`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Src => "src",
            Self::RenderingIntent => "rendering-intent",
            Self::Components => "components",
        }
    }
    pub(crate) fn from_css_name(name: &str) -> Option<Self> {
        [Self::Src, Self::RenderingIntent, Self::Components]
            .into_iter()
            .find(|kind| kind.as_str().eq_ignore_ascii_case(name))
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

crate::rule::counter_style::descriptor_values::descriptor_value_api!(
    CssColorProfileValueError,
    "color-profile"
);

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
    /// A `src` value: the profile URL.
    Src(&'a CssUrl),
    /// A `rendering-intent` value.
    RenderingIntent(CssColorProfileRenderingIntent),
    /// A `components` value: the component names, one per `<ident>` of the comma-separated list.
    Components(&'a [CssColorProfileComponentName]),
    /// A value whose grammar depends on later substitution and is still symbolic.
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
        if let Some(origin) = crate::parser::variables::first_substitution_origin(&replacement) {
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

    /// Returns which descriptor this value was checked as.
    #[must_use]
    pub const fn kind(&self) -> CssColorProfileDescriptorKind {
        self.kind
    }

    /// Returns the component values the descriptor was checked from, with their original provenance.
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }

    /// Returns the checked semantics as a borrowed view.
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
    /// Creates a descriptor occurrence from a checked value, with no source position.
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

    /// Returns the checked descriptor value.
    #[must_use]
    pub const fn value(&self) -> &CssColorProfileDescriptorValue {
        &self.value
    }

    /// Returns the source position of the descriptor, or `None` when it was constructed programmatically.
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
    /// Creates a rule from its name and descriptor occurrences, with no source position.
    ///
    /// The descriptors are retained in the given order, including duplicates.
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
    /// Returns the profile name.
    #[must_use]
    pub const fn name(&self) -> &CssColorProfileRuleName {
        &self.name
    }
    /// Returns every retained descriptor occurrence in authored order, including duplicates.
    #[must_use]
    pub fn descriptors(&self) -> &[CssColorProfileDescriptor] {
        &self.descriptors
    }
    /// Returns the source position of the rule, or `None` when it was constructed programmatically.
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
impl CssColorProfileDescriptorValue {
    /// Parses one complete authored color-profile descriptor with env-only deferral.
    ///
    /// The parser is not read: neither its document mode nor its component limits apply to this grammar, and only the fixed nesting ceiling bounds the input.
    #[expect(
        unused_variables,
        reason = "this grammar does not read the parser configuration"
    )]
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
        kind: crate::rule::color_profile::CssColorProfileDescriptorKind,
    ) -> crate::CssParseReport<Option<CssColorProfileDescriptorValue>> {
        crate::parser::fragments::parse_color_profile_descriptor_value(source.as_ref(), kind)
    }
}
