use crate::component::{CssComponentValueLimits, CssComponentValues};
use crate::declaration::CssSubstitutionDependentValue;
use crate::rule::descriptor_values::DescriptorValueFailure;
use crate::rule::font_face::CssFontDisplay;
use crate::rule::font_feature_values::CssFontFeatureValuesError;
use crate::rule::font_feature_values::{CssFontFeatureValueIndex, CssFontFeatureValueKind};
use crate::source::CssParsedOrigin;

/// A borrowed outer font-display phase.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFeatureDisplayValueRef<'a> {
    Ordinary(CssFontDisplay),
    Pending(&'a CssSubstitutionDependentValue),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FontFeatureDisplayData {
    Ordinary(CssFontDisplay),
    Pending(CssSubstitutionDependentValue),
}

/// The authored outer font-display descriptor with retained complete components.
///
/// # Specified serialization
///
/// Serializes the value without manufacturing a declaration occurrence.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFeatureDisplayValue {
    data: FontFeatureDisplayData,
    components: CssComponentValues,
    origin: Option<CssParsedOrigin>,
}
impl CssFontFeatureDisplayValue {
    pub(crate) fn from_parsed(
        data: FontFeatureDisplayData,
        components: CssComponentValues,
        origin: CssParsedOrigin,
    ) -> Self {
        Self {
            data,
            components,
            origin: Some(origin),
        }
    }
    #[must_use]
    pub const fn view(&self) -> CssFontFeatureDisplayValueRef<'_> {
        match &self.data {
            FontFeatureDisplayData::Ordinary(value) => {
                CssFontFeatureDisplayValueRef::Ordinary(*value)
            }
            FontFeatureDisplayData::Pending(value) => CssFontFeatureDisplayValueRef::Pending(value),
        }
    }
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }
    /// The complete original raw-source window, absent for checked components/reentry.
    #[must_use]
    pub const fn origin(&self) -> Option<&CssParsedOrigin> {
        self.origin.as_ref()
    }
}

/// A borrowed subsidiary value phase; pending input has no completed index count.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFeatureValueRef<'a> {
    Indexes(&'a [CssFontFeatureValueIndex]),
    Pending(&'a CssSubstitutionDependentValue),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FontFeatureValueData {
    Indexes(Vec<CssFontFeatureValueIndex>),
    Pending(CssSubstitutionDependentValue),
}
/// One authored subsidiary feature value in its selected grammar.
///
/// Ordinary indexes retain exact normalized magnitude and supplied token origins.
/// Pending input retains its whole component stream without fabricated indexes.
///
/// # Specified serialization
///
/// Serializes completed exact indexes or the pending whole component stream.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFeatureValue {
    kind: CssFontFeatureValueKind,
    data: FontFeatureValueData,
    components: CssComponentValues,
    origin: Option<CssParsedOrigin>,
}
impl CssFontFeatureValue {
    pub(crate) fn from_parsed(
        kind: CssFontFeatureValueKind,
        data: FontFeatureValueData,
        components: CssComponentValues,
        origin: CssParsedOrigin,
    ) -> Self {
        Self {
            kind,
            data,
            components,
            origin: Some(origin),
        }
    }
    #[must_use]
    pub const fn kind(&self) -> CssFontFeatureValueKind {
        self.kind
    }
    #[must_use]
    pub fn view(&self) -> CssFontFeatureValueRef<'_> {
        match &self.data {
            FontFeatureValueData::Indexes(value) => CssFontFeatureValueRef::Indexes(value),
            FontFeatureValueData::Pending(value) => CssFontFeatureValueRef::Pending(value),
        }
    }
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }
    /// The complete original raw-source window, absent for checked components/reentry.
    #[must_use]
    pub const fn origin(&self) -> Option<&CssParsedOrigin> {
        self.origin.as_ref()
    }
}

fn reject_fontfeature_recovered(
    components: &CssComponentValues,
) -> Result<(), CssFontFeatureValuesError> {
    if let Some(origin) = components.first_implicit_origin() {
        return Err(DescriptorValueFailure::recovered(origin.clone()).into());
    }
    Ok(())
}

impl CssFontFeatureDisplayValue {
    /// Admits the selected authored grammar while preserving every supplied component origin.
    ///
    /// Checks cumulative component and byte limits before grammar or env qualification.
    pub fn try_from_components(
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValuesError> {
        components
            .validate_with_limits(limits)
            .map_err(CssFontFeatureValuesError::from_component)?;
        reject_fontfeature_recovered(&components)?;
        Self::from_validated_components(components, limits)
    }
    fn from_validated_components(
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValuesError> {
        let serialized = components
            .serialize_tokens(limits)
            .map_err(CssFontFeatureValuesError::from_component)?;
        let data =
            crate::parser::font_feature_values::construct_display_value(&components, &serialized)
                .map_err(|error| CssFontFeatureValuesError::from_grammar(error, &serialized))?;
        Ok(Self {
            data,
            components,
            origin: None,
        })
    }
    /// Rechecks a replacement without performing host lookup or substitution.
    ///
    /// Rejects recovered closures and residual decoded var/env functions before ordinary grammar.
    pub fn reparse_after_substitution(
        &self,
        replacement: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValuesError> {
        if !matches!(self.data, FontFeatureDisplayData::Pending(_)) {
            return Err(CssFontFeatureValuesError::not_pending());
        }
        replacement
            .validate_with_limits(limits)
            .map_err(CssFontFeatureValuesError::from_component)?;
        reject_fontfeature_recovered(&replacement)?;
        if let Some(origin) = crate::parser::variables::first_substitution_origin(&replacement) {
            return Err(CssFontFeatureValuesError::residual(origin));
        }
        Self::from_validated_components(replacement, limits)
    }
}

impl CssFontFeatureValue {
    /// Admits the selected authored grammar while preserving every supplied component origin.
    ///
    /// Checks cumulative component and byte limits before grammar or env qualification.
    pub fn try_from_components(
        kind: CssFontFeatureValueKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValuesError> {
        components
            .validate_with_limits(limits)
            .map_err(CssFontFeatureValuesError::from_component)?;
        reject_fontfeature_recovered(&components)?;
        Self::from_validated_components(kind, components, limits)
    }
    fn from_validated_components(
        kind: CssFontFeatureValueKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValuesError> {
        let serialized = components
            .serialize_tokens(limits)
            .map_err(CssFontFeatureValuesError::from_component)?;
        let data = crate::parser::font_feature_values::construct_feature_value(
            kind,
            &components,
            &serialized,
        )
        .map_err(|error| CssFontFeatureValuesError::from_grammar(error, &serialized))?;
        Ok(Self {
            kind,
            data,
            components,
            origin: None,
        })
    }
    /// Rechecks a replacement without performing host lookup or substitution.
    ///
    /// Rejects recovered closures and residual decoded var/env functions before ordinary grammar.
    pub fn reparse_after_substitution(
        &self,
        replacement: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValuesError> {
        if !matches!(self.data, FontFeatureValueData::Pending(_)) {
            return Err(CssFontFeatureValuesError::not_pending());
        }
        replacement
            .validate_with_limits(limits)
            .map_err(CssFontFeatureValuesError::from_component)?;
        reject_fontfeature_recovered(&replacement)?;
        if let Some(origin) = crate::parser::variables::first_substitution_origin(&replacement) {
            return Err(CssFontFeatureValuesError::residual(origin));
        }
        Self::from_validated_components(self.kind, replacement, limits)
    }
}
impl CssFontFeatureDisplayValue {
    /// Parses the outer `@font-feature-values` font-display descriptor's complete value.
    ///
    /// The five font-display keywords admit decoded escaped and ASCII-insensitive
    /// spellings; valid env defers the whole value. There is no invented
    /// descriptor-name position. Components and origin retain all original trivia.
    /// Invalid or trailing input and root annotations reject the complete fragment;
    /// resource failures preserve the fixed 256-depth ceiling. Lexical EOF recovery
    /// remains visible in diagnostics and fails generic clean-report validation.
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
    ) -> crate::CssParseReport<Option<CssFontFeatureDisplayValue>> {
        crate::parser::fragments::parse_font_feature_display_value(source.as_ref())
    }
}
impl CssFontFeatureValue {
    /// Parses one authored subsidiary feature value, including whole-value env deferral.
    ///
    /// Historical-forms/styleset admit one or more indexes, character-variant one or
    /// two, and the other kinds exactly one. Integer token spelling is retained in
    /// components; normalized exact indexes have no machine-integer upper bound.
    /// Valid env defers the completed index grammar; plain var grants no exception.
    /// No friendly definition name is created. Invalid, empty, annotated or trailing
    /// input rejects the whole fragment; token failures use their original origins.
    /// Components and carrier origin include all trivia. The fixed 256-depth ceiling
    /// and lexical EOF diagnostics retain their existing owners; clean validation
    /// accepts exactly reports without diagnostics and performs no font activation.
    /// Name-free token/EOF diagnostics describe the selected integer grammar; root
    /// `!` is rejected as its actual delimiter rather than a named annotation.
    /// An overfull list reports its first surplus integer; empty input reports EOF.
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
        kind: crate::rule::font_feature_values::CssFontFeatureValueKind,
    ) -> crate::CssParseReport<Option<CssFontFeatureValue>> {
        crate::parser::fragments::parse_font_feature_value(source.as_ref(), kind)
    }
}
