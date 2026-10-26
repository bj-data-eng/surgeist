//! Checked authored `@font-palette-values` syntax without palette lookup or substitution.

use std::fmt;

use crate::{
    CssAbsoluteColorEligibility, CssAbsoluteColorExclusion, CssColor, CssComponentValue,
    CssComponentValueError, CssComponentValueLimits, CssComponentValues, CssError,
    CssFontFaceFamily, CssInteger, CssNonNegative, CssNumeric, CssSerializedOrigin,
    CssSerializedValue, CssSourcePosition, CssSubstitutionDependentValue, CssValueOrigin,
};

/// A rejected semantic palette component or rule, or a checked component-native palette
/// descriptor failure.
///
/// Semantic failures are read through the `is_*` predicates and
/// [`Self::contextual_color`]. A descriptor-value failure exposes [`Self::origin`] and,
/// when the components were not admitted or the grammar rejected them,
/// [`Self::component_error`] or [`Self::diagnostic`], which is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontPaletteValuesError {
    kind: FontPaletteValuesKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FontPaletteValuesKind {
    InvalidName,
    ContextualColor(CssAbsoluteColorExclusion),
    MissingFontFamily,
    Value(crate::descriptor_values::DescriptorValueFailure),
}

impl CssFontPaletteValuesError {
    const fn new(kind: FontPaletteValuesKind) -> Self {
        Self { kind }
    }

    fn from_component(error: CssComponentValueError) -> Self {
        crate::descriptor_values::DescriptorValueFailure::component(error).into()
    }

    fn from_grammar(error: CssError, serialized: &CssSerializedValue) -> Self {
        crate::descriptor_values::DescriptorValueFailure::grammar(error, serialized).into()
    }

    fn residual(origin: CssValueOrigin) -> Self {
        crate::descriptor_values::DescriptorValueFailure::residual(origin).into()
    }

    fn not_pending() -> Self {
        crate::descriptor_values::DescriptorValueFailure::not_pending().into()
    }

    const fn value(&self) -> Option<&crate::descriptor_values::DescriptorValueFailure> {
        match &self.kind {
            FontPaletteValuesKind::Value(failure) => Some(failure),
            _ => None,
        }
    }

    crate::error::kind_predicates! { kind;
        /// Returns whether a palette name is not a decoded `<dashed-ident>`.
        is_invalid_name => FontPaletteValuesKind::InvalidName,
        /// Returns whether a palette rule has no `font-family` descriptor.
        is_missing_font_family => FontPaletteValuesKind::MissingFontFamily,
    }

    /// Returns why an override color is not an absolute color.
    #[must_use]
    pub const fn contextual_color(&self) -> Option<CssAbsoluteColorExclusion> {
        match self.kind {
            FontPaletteValuesKind::ContextualColor(reason) => Some(reason),
            _ => None,
        }
    }

    /// Returns the responsible input origin of a descriptor-value failure.
    #[must_use]
    pub fn origin(&self) -> Option<&CssSerializedOrigin> {
        self.value()
            .map(crate::descriptor_values::DescriptorValueFailure::origin)
    }

    /// Returns the component failure when the supplied components were not admitted.
    #[must_use]
    pub fn component_error(&self) -> Option<&CssComponentValueError> {
        self.value()?.component_error()
    }

    /// Returns the grammar diagnostic when the descriptor grammar rejected the components.
    /// Its position indexes the serialized component input; [`Self::origin`] locates the
    /// responsible supplied component.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&CssError> {
        self.value()?.diagnostic()
    }

    /// Returns whether strict reentry still contains a substitution function.
    #[must_use]
    pub fn is_residual_substitution(&self) -> bool {
        self.value()
            .is_some_and(crate::descriptor_values::DescriptorValueFailure::is_residual_substitution)
    }

    /// Returns whether reentry was requested for a value that is not pending substitution.
    #[must_use]
    pub fn is_not_pending(&self) -> bool {
        self.value()
            .is_some_and(crate::descriptor_values::DescriptorValueFailure::is_not_pending)
    }
}

impl From<crate::descriptor_values::DescriptorValueFailure> for CssFontPaletteValuesError {
    fn from(failure: crate::descriptor_values::DescriptorValueFailure) -> Self {
        Self::new(FontPaletteValuesKind::Value(failure))
    }
}

impl fmt::Display for CssFontPaletteValuesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match &self.kind {
            FontPaletteValuesKind::Value(failure) => {
                return failure.describe("font-palette-values", formatter);
            }
            FontPaletteValuesKind::InvalidName => {
                "invalid font-palette-values construction: the name is not a dashed identifier"
            }
            FontPaletteValuesKind::ContextualColor(_) => {
                "invalid font-palette-values construction: an override color is not absolute"
            }
            FontPaletteValuesKind::MissingFontFamily => {
                "invalid font-palette-values construction: the font-family descriptor is missing"
            }
        })
    }
}

impl std::error::Error for CssFontPaletteValuesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.value()?.source()
    }
}

/// A decoded, case-sensitive `<dashed-ident>` palette name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontPaletteName(Box<str>);

impl CssFontPaletteName {
    /// Checks the decoded identifier, including the valid bare `--` spelling.
    pub fn try_new(value: &str) -> Result<Self, CssFontPaletteValuesError> {
        if !value.starts_with("--") || CssComponentValue::try_ident(value).is_err() {
            return Err(CssFontPaletteValuesError::new(
                FontPaletteValuesKind::InvalidName,
            ));
        }
        Ok(Self(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The three descriptor grammars in a palette definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontPaletteDescriptorKind {
    FontFamily,
    BasePalette,
    OverrideColors,
}

impl CssFontPaletteDescriptorKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FontFamily => "font-family",
            Self::BasePalette => "base-palette",
            Self::OverrideColors => "override-colors",
        }
    }

    pub(crate) fn from_css_name(name: &str) -> Option<Self> {
        [Self::FontFamily, Self::BasePalette, Self::OverrideColors]
            .into_iter()
            .find(|kind| kind.as_str().eq_ignore_ascii_case(name))
    }
}

/// The authored base palette selection; absence remains distinct on the rule.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontPaletteBase {
    Light,
    Dark,
    Index(CssNumeric<CssInteger, CssNonNegative>),
}

/// An authored override pair. Duplicated indices are retained by its owning list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteOverride {
    index: CssNumeric<CssInteger, CssNonNegative>,
    color: CssColor,
}

impl CssFontPaletteOverride {
    pub fn try_new(
        index: CssNumeric<CssInteger, CssNonNegative>,
        color: CssColor,
    ) -> Result<Self, CssFontPaletteValuesError> {
        if let CssAbsoluteColorEligibility::Contextual(reason) = color.absolute_eligibility() {
            return Err(CssFontPaletteValuesError::new(
                FontPaletteValuesKind::ContextualColor(reason),
            ));
        }
        Ok(Self { index, color })
    }

    #[must_use]
    pub const fn index(&self) -> &CssNumeric<CssInteger, CssNonNegative> {
        &self.index
    }

    #[must_use]
    pub const fn color(&self) -> &CssColor {
        &self.color
    }
}

/// Borrowed checked descriptor semantics. Components retain the exact authored syntax.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontPaletteDescriptorValueRef<'a> {
    FontFamily(&'a [CssFontFaceFamily]),
    BasePalette(&'a CssFontPaletteBase),
    OverrideColors(&'a [CssFontPaletteOverride]),
    Pending(&'a CssSubstitutionDependentValue),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssFontPaletteDescriptorData {
    FontFamily(Vec<CssFontFaceFamily>),
    BasePalette(CssFontPaletteBase),
    OverrideColors(Vec<CssFontPaletteOverride>),
    Pending(CssSubstitutionDependentValue),
}

/// One checked whole descriptor value with original component provenance.
///
/// # Specified serialization
///
/// Canonical specified descriptor text; pending values retain their token stream.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteDescriptorValue {
    kind: CssFontPaletteDescriptorKind,
    components: CssComponentValues,
    data: CssFontPaletteDescriptorData,
}

impl CssFontPaletteDescriptorValue {
    /// Checks the selected grammar while retaining mixed or programmatic origins.
    ///
    /// Uses the same component and output-byte limits as shared component serialization.
    pub fn try_new(
        kind: CssFontPaletteDescriptorKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontPaletteValuesError> {
        components
            .validate_with_limits(limits)
            .map_err(CssFontPaletteValuesError::from_component)?;
        let serialized = components
            .serialize_tokens(limits)
            .map_err(CssFontPaletteValuesError::from_component)?;
        let data = crate::parser::font_palette_values::construct_descriptor_value(
            kind,
            &components,
            &serialized,
        )
        .map_err(|error| CssFontPaletteValuesError::from_grammar(error, &serialized))?;
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
    ) -> Result<Self, CssFontPaletteValuesError> {
        if !matches!(self.data, CssFontPaletteDescriptorData::Pending(_)) {
            return Err(CssFontPaletteValuesError::not_pending());
        }
        replacement
            .validate_with_limits(limits)
            .map_err(CssFontPaletteValuesError::from_component)?;
        if let Some(origin) = crate::parser::first_substitution_origin(&replacement) {
            return Err(CssFontPaletteValuesError::residual(origin));
        }
        Self::try_new(self.kind, replacement, limits)
    }

    pub(crate) fn from_parts(
        kind: CssFontPaletteDescriptorKind,
        components: CssComponentValues,
        data: CssFontPaletteDescriptorData,
    ) -> Self {
        let matching_kind = match &data {
            CssFontPaletteDescriptorData::FontFamily(_) => {
                kind == CssFontPaletteDescriptorKind::FontFamily
            }
            CssFontPaletteDescriptorData::BasePalette(_) => {
                kind == CssFontPaletteDescriptorKind::BasePalette
            }
            CssFontPaletteDescriptorData::OverrideColors(_) => {
                kind == CssFontPaletteDescriptorKind::OverrideColors
            }
            CssFontPaletteDescriptorData::Pending(_) => true,
        };
        debug_assert!(matching_kind);
        Self {
            kind,
            components,
            data,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssFontPaletteDescriptorKind {
        self.kind
    }

    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }

    #[must_use]
    pub fn view(&self) -> CssFontPaletteDescriptorValueRef<'_> {
        match &self.data {
            CssFontPaletteDescriptorData::FontFamily(value) => {
                CssFontPaletteDescriptorValueRef::FontFamily(value)
            }
            CssFontPaletteDescriptorData::BasePalette(value) => {
                CssFontPaletteDescriptorValueRef::BasePalette(value)
            }
            CssFontPaletteDescriptorData::OverrideColors(value) => {
                CssFontPaletteDescriptorValueRef::OverrideColors(value)
            }
            CssFontPaletteDescriptorData::Pending(value) => {
                CssFontPaletteDescriptorValueRef::Pending(value)
            }
        }
    }
}

/// One ordered descriptor occurrence; checked construction invents no source position.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteDescriptor {
    value: CssFontPaletteDescriptorValue,
    position: Option<CssSourcePosition>,
}

impl CssFontPaletteDescriptor {
    #[must_use]
    pub const fn new(value: CssFontPaletteDescriptorValue) -> Self {
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
    pub const fn value(&self) -> &CssFontPaletteDescriptorValue {
        &self.value
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// An authored palette definition retaining every valid descriptor occurrence in order.
///
/// # Specified serialization
///
/// Canonical specified rule text. Descriptor duplicates retain authored order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteValuesRule {
    name: CssFontPaletteName,
    descriptors: Vec<CssFontPaletteDescriptor>,
    position: Option<CssSourcePosition>,
}

impl CssFontPaletteValuesRule {
    pub fn try_new(
        name: CssFontPaletteName,
        descriptors: Vec<CssFontPaletteDescriptor>,
    ) -> Result<Self, CssFontPaletteValuesError> {
        validate_descriptors(&descriptors)?;
        Ok(Self {
            name,
            descriptors,
            position: None,
        })
    }

    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    #[must_use]
    pub const fn name(&self) -> &CssFontPaletteName {
        &self.name
    }

    #[must_use]
    pub fn descriptors(&self) -> &[CssFontPaletteDescriptor] {
        &self.descriptors
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

pub(crate) fn validate_descriptors(
    descriptors: &[CssFontPaletteDescriptor],
) -> Result<(), CssFontPaletteValuesError> {
    if descriptors
        .iter()
        .any(|descriptor| descriptor.value.kind() == CssFontPaletteDescriptorKind::FontFamily)
    {
        Ok(())
    } else {
        Err(CssFontPaletteValuesError::new(
            FontPaletteValuesKind::MissingFontFamily,
        ))
    }
}
