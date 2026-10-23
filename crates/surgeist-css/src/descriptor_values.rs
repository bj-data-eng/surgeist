//! Authored descriptor phases with original component provenance.

use crate::syntax::CssCounterStyleDescriptor;
use crate::{
    CssComponentValueError, CssError, CssFontFeatureValuesError, CssSerializedOrigin,
    CssSerializedValue, CssValueOrigin,
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

/// The shared failure of one checked component-native descriptor construction or
/// pending reentry, mapped to the supplied component origins and never to
/// fabricated source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DescriptorValueFailure {
    kind: DescriptorFailureKind,
    origin: Box<CssSerializedOrigin>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DescriptorFailureKind {
    Component(Box<CssComponentValueError>),
    Grammar(Box<CssError>),
    RecoveredComponent,
    ResidualSubstitution,
    NotPending,
}

impl DescriptorValueFailure {
    pub(crate) fn component(error: CssComponentValueError) -> Self {
        Self {
            origin: Box::new(CssSerializedOrigin::Token(error.origin().clone())),
            kind: DescriptorFailureKind::Component(Box::new(error)),
        }
    }

    pub(crate) fn grammar(error: CssError, serialized: &CssSerializedValue) -> Self {
        Self {
            origin: Box::new(error.component_input_origin(serialized)),
            kind: DescriptorFailureKind::Grammar(Box::new(error)),
        }
    }

    pub(crate) fn recovered(origin: CssValueOrigin) -> Self {
        Self {
            kind: DescriptorFailureKind::RecoveredComponent,
            origin: Box::new(CssSerializedOrigin::Token(origin)),
        }
    }

    pub(crate) fn residual(origin: CssValueOrigin) -> Self {
        Self {
            kind: DescriptorFailureKind::ResidualSubstitution,
            origin: Box::new(CssSerializedOrigin::Token(origin)),
        }
    }

    pub(crate) fn not_pending() -> Self {
        Self {
            kind: DescriptorFailureKind::NotPending,
            origin: Box::new(CssSerializedOrigin::End(None)),
        }
    }

    pub(crate) const fn origin(&self) -> &CssSerializedOrigin {
        &self.origin
    }

    pub(crate) fn component_error(&self) -> Option<&CssComponentValueError> {
        match &self.kind {
            DescriptorFailureKind::Component(error) => Some(error),
            _ => None,
        }
    }

    pub(crate) fn diagnostic(&self) -> Option<&CssError> {
        match &self.kind {
            DescriptorFailureKind::Grammar(error) => Some(error),
            _ => None,
        }
    }

    pub(crate) const fn is_recovered_component(&self) -> bool {
        matches!(self.kind, DescriptorFailureKind::RecoveredComponent)
    }

    pub(crate) const fn is_residual_substitution(&self) -> bool {
        matches!(self.kind, DescriptorFailureKind::ResidualSubstitution)
    }

    pub(crate) const fn is_not_pending(&self) -> bool {
        matches!(self.kind, DescriptorFailureKind::NotPending)
    }

    /// One summary sentence naming the owning rule, without the source's text.
    pub(crate) fn describe(&self, rule: &str, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid {rule} descriptor value: ")?;
        formatter.write_str(match self.kind {
            DescriptorFailureKind::Component(_) => "a supplied component is invalid",
            DescriptorFailureKind::Grammar(_) => "the descriptor grammar rejected it",
            DescriptorFailureKind::RecoveredComponent => {
                "an implicitly closed component is not admitted"
            }
            DescriptorFailureKind::ResidualSubstitution => "it still contains a substitution",
            DescriptorFailureKind::NotPending => "it is not pending substitution",
        })
    }

    pub(crate) fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            DescriptorFailureKind::Component(error) => Some(&**error),
            DescriptorFailureKind::Grammar(error) => Some(&**error),
            _ => None,
        }
    }
}

/// Generates the public accessors of a descriptor-value error that holds a
/// [`DescriptorValueFailure`] in `failure`.
macro_rules! descriptor_value_api {
    ($ty:ident, $rule:literal) => {
        impl $ty {
            /// Returns the responsible input origin: a supplied token, a generated boundary or
            /// EOF.
            #[must_use]
            pub fn origin(&self) -> &crate::CssSerializedOrigin {
                self.failure.origin()
            }

            /// Returns the component failure when the supplied components were not admitted.
            #[must_use]
            pub fn component_error(&self) -> Option<&crate::CssComponentValueError> {
                self.failure.component_error()
            }

            /// Returns the grammar diagnostic when the descriptor grammar rejected the
            /// components. Its position indexes the serialized component input; [`Self::origin`]
            /// locates the responsible supplied component.
            #[must_use]
            pub fn diagnostic(&self) -> Option<&crate::CssError> {
                self.failure.diagnostic()
            }

            /// Returns whether an implicitly closed component was supplied.
            #[must_use]
            pub fn is_recovered_component(&self) -> bool {
                self.failure.is_recovered_component()
            }

            /// Returns whether strict reentry still contains a substitution function.
            #[must_use]
            pub fn is_residual_substitution(&self) -> bool {
                self.failure.is_residual_substitution()
            }

            /// Returns whether reentry was requested for a value that is not pending
            /// substitution.
            #[must_use]
            pub fn is_not_pending(&self) -> bool {
                self.failure.is_not_pending()
            }
        }

        impl From<crate::descriptor_values::DescriptorValueFailure> for $ty {
            fn from(failure: crate::descriptor_values::DescriptorValueFailure) -> Self {
                Self { failure }
            }
        }

        impl std::fmt::Display for $ty {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.failure.describe($rule, formatter)
            }
        }

        impl std::error::Error for $ty {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.failure.source()
            }
        }
    };
}
pub(crate) use descriptor_value_api;

fn reject_counterstyle_recovered(
    components: &CssComponentValues,
) -> Result<(), CssCounterStyleValueError> {
    if let Some(origin) = components.first_implicit_origin() {
        return Err(DescriptorValueFailure::recovered(origin.clone()).into());
    }
    Ok(())
}

/// A checked component-native `@counter-style` descriptor construction or reentry failure.
///
/// The supplied components were not admitted ([`Self::component_error`]), the descriptor
/// grammar rejected them ([`Self::diagnostic`]), or reentry was refused; the component
/// error or diagnostic is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterStyleValueError {
    failure: DescriptorValueFailure,
}

descriptor_value_api!(CssCounterStyleValueError, "counter-style");

impl CssCounterStyleValueError {
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

fn reject_fontfeature_recovered(
    components: &CssComponentValues,
) -> Result<(), CssFontFeatureValuesError> {
    if let Some(origin) = components.first_implicit_origin() {
        return Err(DescriptorValueFailure::recovered(origin.clone()).into());
    }
    Ok(())
}

impl CssCounterStyleDescriptorValue {
    /// Admits the selected authored grammar while preserving every supplied component origin.
    ///
    /// Checks cumulative component and byte limits before grammar or env qualification.
    pub fn try_from_components(
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
            .serialize_tokens(limits)
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
    ///
    /// Rejects recovered closures and residual decoded var/env functions before ordinary grammar.
    pub fn reparse_after_substitution(
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
        if let Some(origin) = crate::parser::first_substitution_origin(&replacement) {
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
        if let Some(origin) = crate::parser::first_substitution_origin(&replacement) {
            return Err(CssFontFeatureValuesError::residual(origin));
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
