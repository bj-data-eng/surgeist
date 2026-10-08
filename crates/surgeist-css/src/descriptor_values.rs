//! Authored descriptor phases with original component provenance.

use crate::syntax::CssCounterStyleDescriptor;
use crate::{
    CssComponentValueError, CssComponentValueErrorKind, CssSerializedOrigin, CssSerializedValue,
    CssValueOrigin, Error, ErrorKind,
};
use crate::{
    CssComponentValueLimits, CssComponentValues, CssCounterAdditiveSymbols,
    CssCounterStyleDescriptorKind, CssCounterStyleName, CssCounterStyleNegative,
    CssCounterStylePad, CssCounterStyleRange, CssCounterStyleSpeakAs, CssCounterStyleSystem,
    CssCounterSymbol, CssCounterSymbols, CssDescriptorOccurrence, CssFontDisplay,
    CssFontFeatureValueIndex, CssFontFeatureValueKind, CssParsedOrigin,
    CssSubstitutionDependentValue,
};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CounterStyleValueData {
    System(CssCounterStyleSystem),
    Negative(CssCounterStyleNegative),
    Symbols(CssCounterSymbols),
    Prefix(CssCounterSymbol),
    Suffix(CssCounterSymbol),
    Range(CssCounterStyleRange),
    Pad(CssCounterStylePad),
    Fallback(CssCounterStyleName),
    AdditiveSymbols(CssCounterAdditiveSymbols),
    SpeakAs(CssCounterStyleSpeakAs),
    Pending(CssSubstitutionDependentValue),
}

/// A borrowed semantic counter descriptor value, without a named occurrence.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleDescriptorValueRef<'a> {
    System(&'a CssCounterStyleSystem),
    Negative(&'a CssCounterStyleNegative),
    Symbols(&'a CssCounterSymbols),
    Prefix(&'a CssCounterSymbol),
    Suffix(&'a CssCounterSymbol),
    Range(&'a CssCounterStyleRange),
    Pad(&'a CssCounterStylePad),
    Fallback(&'a CssCounterStyleName),
    AdditiveSymbols(&'a CssCounterAdditiveSymbols),
    SpeakAs(&'a CssCounterStyleSpeakAs),
    Pending(&'a CssSubstitutionDependentValue),
}

/// One authored counter descriptor with ordinary or whole-value pending semantics.
///
/// Components and origin include surrounding trivia. This value does not impose
/// cross-descriptor system/symbol requirements or resolve counter-style references.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStyleDescriptorValue {
    kind: CssCounterStyleDescriptorKind,
    data: CounterStyleValueData,
    components: CssComponentValues,
    origin: Option<CssParsedOrigin>,
}

impl CssCounterStyleDescriptorValue {
    pub(crate) fn from_parsed(
        kind: CssCounterStyleDescriptorKind,
        data: CounterStyleValueData,
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

    /// Returns the selected descriptor owner in either phase.
    #[must_use]
    pub const fn kind(&self) -> CssCounterStyleDescriptorKind {
        self.kind
    }

    /// Returns every component of the complete value input, including trivia.
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }

    /// The complete raw-source value window, absent for checked components and reentry.
    #[must_use]
    pub const fn origin(&self) -> Option<&CssParsedOrigin> {
        self.origin.as_ref()
    }

    /// Borrows the admitted semantic value without exposing an occurrence.
    #[must_use]
    pub const fn view(&self) -> CssCounterStyleDescriptorValueRef<'_> {
        match &self.data {
            CounterStyleValueData::System(value) => {
                CssCounterStyleDescriptorValueRef::System(value)
            }
            CounterStyleValueData::Negative(value) => {
                CssCounterStyleDescriptorValueRef::Negative(value)
            }
            CounterStyleValueData::Symbols(value) => {
                CssCounterStyleDescriptorValueRef::Symbols(value)
            }
            CounterStyleValueData::Prefix(value) => {
                CssCounterStyleDescriptorValueRef::Prefix(value)
            }
            CounterStyleValueData::Suffix(value) => {
                CssCounterStyleDescriptorValueRef::Suffix(value)
            }
            CounterStyleValueData::Range(value) => CssCounterStyleDescriptorValueRef::Range(value),
            CounterStyleValueData::Pad(value) => CssCounterStyleDescriptorValueRef::Pad(value),
            CounterStyleValueData::Fallback(value) => {
                CssCounterStyleDescriptorValueRef::Fallback(value)
            }
            CounterStyleValueData::AdditiveSymbols(value) => {
                CssCounterStyleDescriptorValueRef::AdditiveSymbols(value)
            }
            CounterStyleValueData::SpeakAs(value) => {
                CssCounterStyleDescriptorValueRef::SpeakAs(value)
            }
            CounterStyleValueData::Pending(value) => {
                CssCounterStyleDescriptorValueRef::Pending(value)
            }
        }
    }
}

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

fn reject_counterstyle_recovered(
    components: &CssComponentValues,
) -> Result<(), CssCounterStyleValueError> {
    if let Some(origin) = components.first_implicit_origin() {
        return Err(CssCounterStyleValueError {
            kind: Box::new(CssCounterStyleValueErrorKind::RecoveredComponent),
            origin: Box::new(CssSerializedOrigin::Token(origin.clone())),
            source: None,
        });
    }
    Ok(())
}

/// A checked component-native descriptor construction or reentry failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleValueErrorKind {
    Component(CssComponentValueErrorKind),
    RecoveredComponent,
    Grammar(ErrorKind),
    ResidualSubstitution,
    NotPending,
}

/// A failure mapped to the supplied component origins, never fabricated source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterStyleValueError {
    kind: Box<CssCounterStyleValueErrorKind>,
    origin: Box<CssSerializedOrigin>,
    source: Option<Box<CounterStyleValueErrorSource>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CounterStyleValueErrorSource {
    Component(CssComponentValueError),
    Grammar(Error),
}

impl CssCounterStyleValueError {
    #[must_use]
    pub fn kind(&self) -> &CssCounterStyleValueErrorKind {
        &self.kind
    }

    #[must_use]
    pub fn origin(&self) -> &CssSerializedOrigin {
        &self.origin
    }

    fn from_component(error: CssComponentValueError) -> Self {
        Self {
            kind: Box::new(CssCounterStyleValueErrorKind::Component(error.kind())),
            origin: Box::new(CssSerializedOrigin::Token(error.origin().clone())),
            source: Some(Box::new(CounterStyleValueErrorSource::Component(error))),
        }
    }

    fn from_grammar(error: Error, serialized: &CssSerializedValue) -> Self {
        let origin = serialized
            .origin_at(error.position().byte_offset().value())
            .expect("descriptor parser reports a cursor inside its serialized input or at EOF")
            .clone();
        Self {
            kind: Box::new(CssCounterStyleValueErrorKind::Grammar(error.kind().clone())),
            origin: Box::new(origin),
            source: Some(Box::new(CounterStyleValueErrorSource::Grammar(error))),
        }
    }

    fn residual(origin: CssValueOrigin) -> Self {
        Self {
            kind: Box::new(CssCounterStyleValueErrorKind::ResidualSubstitution),
            origin: Box::new(CssSerializedOrigin::Token(origin)),
            source: None,
        }
    }

    fn not_pending() -> Self {
        Self {
            kind: Box::new(CssCounterStyleValueErrorKind::NotPending),
            origin: Box::new(CssSerializedOrigin::End(None)),
            source: None,
        }
    }
}

impl fmt::Display for CssCounterStyleValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid counterstyle descriptor value: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for CssCounterStyleValueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.source.as_deref()? {
            CounterStyleValueErrorSource::Component(error) => Some(error),
            CounterStyleValueErrorSource::Grammar(error) => Some(error),
        }
    }
}

fn reject_fontfeature_recovered(
    components: &CssComponentValues,
) -> Result<(), CssFontFeatureValueError> {
    if let Some(origin) = components.first_implicit_origin() {
        return Err(CssFontFeatureValueError {
            kind: Box::new(CssFontFeatureValueErrorKind::RecoveredComponent),
            origin: Box::new(CssSerializedOrigin::Token(origin.clone())),
            source: None,
        });
    }
    Ok(())
}

/// A checked component-native descriptor construction or reentry failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFeatureValueErrorKind {
    Component(CssComponentValueErrorKind),
    RecoveredComponent,
    Grammar(ErrorKind),
    ResidualSubstitution,
    NotPending,
}

/// A failure mapped to the supplied component origins, never fabricated source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFeatureValueError {
    kind: Box<CssFontFeatureValueErrorKind>,
    origin: Box<CssSerializedOrigin>,
    source: Option<Box<FontFeatureValueErrorSource>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FontFeatureValueErrorSource {
    Component(CssComponentValueError),
    Grammar(Error),
}

impl CssFontFeatureValueError {
    #[must_use]
    pub fn kind(&self) -> &CssFontFeatureValueErrorKind {
        &self.kind
    }

    #[must_use]
    pub fn origin(&self) -> &CssSerializedOrigin {
        &self.origin
    }

    fn from_component(error: CssComponentValueError) -> Self {
        Self {
            kind: Box::new(CssFontFeatureValueErrorKind::Component(error.kind())),
            origin: Box::new(CssSerializedOrigin::Token(error.origin().clone())),
            source: Some(Box::new(FontFeatureValueErrorSource::Component(error))),
        }
    }

    fn from_grammar(error: Error, serialized: &CssSerializedValue) -> Self {
        let origin = serialized
            .origin_at(error.position().byte_offset().value())
            .expect("descriptor parser reports a cursor inside its serialized input or at EOF")
            .clone();
        Self {
            kind: Box::new(CssFontFeatureValueErrorKind::Grammar(error.kind().clone())),
            origin: Box::new(origin),
            source: Some(Box::new(FontFeatureValueErrorSource::Grammar(error))),
        }
    }

    fn residual(origin: CssValueOrigin) -> Self {
        Self {
            kind: Box::new(CssFontFeatureValueErrorKind::ResidualSubstitution),
            origin: Box::new(CssSerializedOrigin::Token(origin)),
            source: None,
        }
    }

    fn not_pending() -> Self {
        Self {
            kind: Box::new(CssFontFeatureValueErrorKind::NotPending),
            origin: Box::new(CssSerializedOrigin::End(None)),
            source: None,
        }
    }
}

impl fmt::Display for CssFontFeatureValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid fontfeature descriptor value: {:?}",
            self.kind
        )
    }
}

impl std::error::Error for CssFontFeatureValueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.source.as_deref()? {
            FontFeatureValueErrorSource::Component(error) => Some(error),
            FontFeatureValueErrorSource::Grammar(error) => Some(error),
        }
    }
}

impl CssCounterStyleDescriptorValue {
    /// Admits the selected authored grammar while preserving every supplied component origin.
    pub fn try_from_components(
        kind: CssCounterStyleDescriptorKind,
        components: CssComponentValues,
    ) -> Result<Self, CssCounterStyleValueError> {
        Self::try_from_components_with_limits(kind, components, CssComponentValueLimits::default())
    }
    /// Checks cumulative component and byte limits before grammar or env qualification.
    pub fn try_from_components_with_limits(
        kind: CssCounterStyleDescriptorKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssCounterStyleValueError> {
        components
            .validate_with_limits(limits)
            .map_err(CssCounterStyleValueError::from_component)?;
        reject_counterstyle_recovered(&components)?;
        Self::from_validated_components(kind, components, limits)
    }
    fn from_validated_components(
        kind: CssCounterStyleDescriptorKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssCounterStyleValueError> {
        let serialized = components
            .serialize_with_limit(limits.max_css_bytes())
            .map_err(CssCounterStyleValueError::from_component)?;
        let data = crate::parser::counter_style::construct_descriptor_value(
            kind,
            &components,
            &serialized,
        )
        .map_err(|error| CssCounterStyleValueError::from_grammar(error, &serialized))?;
        Ok(Self {
            kind,
            data,
            components,
            origin: None,
        })
    }
    /// Rechecks a replacement without performing host lookup or substitution.
    pub fn reparse_after_substitution(
        &self,
        replacement: CssComponentValues,
    ) -> Result<Self, CssCounterStyleValueError> {
        self.reparse_after_substitution_with_limits(replacement, CssComponentValueLimits::default())
    }
    /// Rejects recovered closures and residual decoded var/env functions before ordinary grammar.
    pub fn reparse_after_substitution_with_limits(
        &self,
        replacement: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssCounterStyleValueError> {
        if !matches!(self.data, CounterStyleValueData::Pending(_)) {
            return Err(CssCounterStyleValueError::not_pending());
        }
        replacement
            .validate_with_limits(limits)
            .map_err(CssCounterStyleValueError::from_component)?;
        reject_counterstyle_recovered(&replacement)?;
        if let Some(origin) = crate::parser::first_substitution_origin(&replacement) {
            return Err(CssCounterStyleValueError::residual(origin));
        }
        Self::from_validated_components(self.kind, replacement, limits)
    }
}

impl CssFontFeatureDisplayValue {
    /// Admits the selected authored grammar while preserving every supplied component origin.
    pub fn try_from_components(
        components: CssComponentValues,
    ) -> Result<Self, CssFontFeatureValueError> {
        Self::try_from_components_with_limits(components, CssComponentValueLimits::default())
    }
    /// Checks cumulative component and byte limits before grammar or env qualification.
    pub fn try_from_components_with_limits(
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValueError> {
        components
            .validate_with_limits(limits)
            .map_err(CssFontFeatureValueError::from_component)?;
        reject_fontfeature_recovered(&components)?;
        Self::from_validated_components(components, limits)
    }
    fn from_validated_components(
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValueError> {
        let serialized = components
            .serialize_with_limit(limits.max_css_bytes())
            .map_err(CssFontFeatureValueError::from_component)?;
        let data =
            crate::parser::font_feature_values::construct_display_value(&components, &serialized)
                .map_err(|error| CssFontFeatureValueError::from_grammar(error, &serialized))?;
        Ok(Self {
            data,
            components,
            origin: None,
        })
    }
    /// Rechecks a replacement without performing host lookup or substitution.
    pub fn reparse_after_substitution(
        &self,
        replacement: CssComponentValues,
    ) -> Result<Self, CssFontFeatureValueError> {
        self.reparse_after_substitution_with_limits(replacement, CssComponentValueLimits::default())
    }
    /// Rejects recovered closures and residual decoded var/env functions before ordinary grammar.
    pub fn reparse_after_substitution_with_limits(
        &self,
        replacement: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValueError> {
        if !matches!(self.data, FontFeatureDisplayData::Pending(_)) {
            return Err(CssFontFeatureValueError::not_pending());
        }
        replacement
            .validate_with_limits(limits)
            .map_err(CssFontFeatureValueError::from_component)?;
        reject_fontfeature_recovered(&replacement)?;
        if let Some(origin) = crate::parser::first_substitution_origin(&replacement) {
            return Err(CssFontFeatureValueError::residual(origin));
        }
        Self::from_validated_components(replacement, limits)
    }
}

impl CssFontFeatureValue {
    /// Admits the selected authored grammar while preserving every supplied component origin.
    pub fn try_from_components(
        kind: CssFontFeatureValueKind,
        components: CssComponentValues,
    ) -> Result<Self, CssFontFeatureValueError> {
        Self::try_from_components_with_limits(kind, components, CssComponentValueLimits::default())
    }
    /// Checks cumulative component and byte limits before grammar or env qualification.
    pub fn try_from_components_with_limits(
        kind: CssFontFeatureValueKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValueError> {
        components
            .validate_with_limits(limits)
            .map_err(CssFontFeatureValueError::from_component)?;
        reject_fontfeature_recovered(&components)?;
        Self::from_validated_components(kind, components, limits)
    }
    fn from_validated_components(
        kind: CssFontFeatureValueKind,
        components: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValueError> {
        let serialized = components
            .serialize_with_limit(limits.max_css_bytes())
            .map_err(CssFontFeatureValueError::from_component)?;
        let data = crate::parser::font_feature_values::construct_feature_value(
            kind,
            &components,
            &serialized,
        )
        .map_err(|error| CssFontFeatureValueError::from_grammar(error, &serialized))?;
        Ok(Self {
            kind,
            data,
            components,
            origin: None,
        })
    }
    /// Rechecks a replacement without performing host lookup or substitution.
    pub fn reparse_after_substitution(
        &self,
        replacement: CssComponentValues,
    ) -> Result<Self, CssFontFeatureValueError> {
        self.reparse_after_substitution_with_limits(replacement, CssComponentValueLimits::default())
    }
    /// Rejects recovered closures and residual decoded var/env functions before ordinary grammar.
    pub fn reparse_after_substitution_with_limits(
        &self,
        replacement: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssFontFeatureValueError> {
        if !matches!(self.data, FontFeatureValueData::Pending(_)) {
            return Err(CssFontFeatureValueError::not_pending());
        }
        replacement
            .validate_with_limits(limits)
            .map_err(CssFontFeatureValueError::from_component)?;
        reject_fontfeature_recovered(&replacement)?;
        if let Some(origin) = crate::parser::first_substitution_origin(&replacement) {
            return Err(CssFontFeatureValueError::residual(origin));
        }
        Self::from_validated_components(self.kind, replacement, limits)
    }
}

impl CssCounterStyleDescriptorValue {
    pub(crate) fn into_occurrence(
        self,
        name: CssParsedOrigin,
        origin: CssParsedOrigin,
        components: CssComponentValues,
    ) -> CssCounterStyleDescriptor {
        let kind = self.kind;
        let occurrence = CssDescriptorOccurrence::from_parsed(self, name, origin, components);
        match kind {
            CssCounterStyleDescriptorKind::System => CssCounterStyleDescriptor::System(occurrence),
            CssCounterStyleDescriptorKind::Negative => {
                CssCounterStyleDescriptor::Negative(occurrence)
            }
            CssCounterStyleDescriptorKind::Symbols => {
                CssCounterStyleDescriptor::Symbols(occurrence)
            }
            CssCounterStyleDescriptorKind::Prefix => CssCounterStyleDescriptor::Prefix(occurrence),
            CssCounterStyleDescriptorKind::Suffix => CssCounterStyleDescriptor::Suffix(occurrence),
            CssCounterStyleDescriptorKind::Range => CssCounterStyleDescriptor::Range(occurrence),
            CssCounterStyleDescriptorKind::Pad => CssCounterStyleDescriptor::Pad(occurrence),
            CssCounterStyleDescriptorKind::Fallback => {
                CssCounterStyleDescriptor::Fallback(occurrence)
            }
            CssCounterStyleDescriptorKind::AdditiveSymbols => {
                CssCounterStyleDescriptor::AdditiveSymbols(occurrence)
            }
            CssCounterStyleDescriptorKind::SpeakAs => {
                CssCounterStyleDescriptor::SpeakAs(occurrence)
            }
        }
    }
}
