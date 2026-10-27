//! Authored descriptor phases with original component provenance.

use crate::component::{CssComponentValueError, CssSerializedValue, CssValueOrigin};
use crate::component::{CssComponentValueLimits, CssComponentValues};
use crate::declaration::CssSubstitutionDependentValue;
use crate::diagnostic::CssError;
use crate::rule::CssDescriptorOccurrence;
use crate::rule::counter_style::syntax::CssCounterStyleDescriptor;
use crate::rule::counter_style::{
    CssCounterAdditiveSymbols, CssCounterStyleDescriptorKind, CssCounterStyleName,
    CssCounterStyleNegative, CssCounterStylePad, CssCounterStyleRange, CssCounterStyleSpeakAs,
    CssCounterStyleSystem, CssCounterSymbol, CssCounterSymbols,
};
use crate::rule::descriptor_values::DescriptorValueFailure;
use crate::source::CssParsedOrigin;

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
    /// A `system` value.
    System(&'a CssCounterStyleSystem),
    /// A `negative` value.
    Negative(&'a CssCounterStyleNegative),
    /// A `symbols` value.
    Symbols(&'a CssCounterSymbols),
    /// A `prefix` value.
    Prefix(&'a CssCounterSymbol),
    /// A `suffix` value.
    Suffix(&'a CssCounterSymbol),
    /// A `range` value.
    Range(&'a CssCounterStyleRange),
    /// A `pad` value.
    Pad(&'a CssCounterStylePad),
    /// A `fallback` value: the name of the fallback counter style.
    Fallback(&'a CssCounterStyleName),
    /// An `additive-symbols` value.
    AdditiveSymbols(&'a CssCounterAdditiveSymbols),
    /// A `speak-as` value.
    SpeakAs(&'a CssCounterStyleSpeakAs),
    /// A value whose grammar depends on later substitution and is still symbolic.
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

/// Generates the public accessors of a descriptor-value error that holds a
/// [`DescriptorValueFailure`] in `failure`.
macro_rules! descriptor_value_api {
    ($ty:ident, $rule:literal) => {
        impl $ty {
            /// Returns the responsible input origin: a supplied token, a generated boundary or
            /// EOF.
            #[must_use]
            pub fn origin(&self) -> &crate::component::CssSerializedOrigin {
                self.failure.origin()
            }

            /// Returns the component failure when the supplied components were not admitted.
            #[must_use]
            pub fn component_error(&self) -> Option<&crate::component::CssComponentValueError> {
                self.failure.component_error()
            }

            /// Returns the grammar diagnostic when the descriptor grammar rejected the
            /// components. Its position indexes the serialized component input; [`Self::origin`]
            /// locates the responsible supplied component.
            #[must_use]
            pub fn diagnostic(&self) -> Option<&crate::diagnostic::CssError> {
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

        impl From<crate::rule::descriptor_values::DescriptorValueFailure> for $ty {
            fn from(failure: crate::rule::descriptor_values::DescriptorValueFailure) -> Self {
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
        if let Some(origin) = crate::parser::variables::first_substitution_origin(&replacement) {
            return Err(CssCounterStyleValueError::residual(origin));
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
impl CssCounterStyleDescriptorValue {
    /// Parses one complete raw Counter Styles 3 descriptor value.
    ///
    /// The supplied kind selects the existing descriptor grammar without a rule name
    /// or descriptor-name occurrence. Components and origin cover the complete input,
    /// including trivia; exact numeric values retain their original token origins.
    /// Cross-descriptor system/symbol requirements remain with the enclosing rule.
    /// Empty, invalid, annotated or trailing input returns `None` with whole-input
    /// rejection. The fixed 256-depth ceiling retains typed resource diagnostics.
    /// Implicit lexical EOF recovery may retain a value with an unclean report;
    /// generic clean-report validation accepts exactly reports without diagnostics.
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
        kind: crate::rule::counter_style::CssCounterStyleDescriptorKind,
    ) -> crate::CssParseReport<Option<CssCounterStyleDescriptorValue>> {
        crate::parser::fragments::parse_counter_style_descriptor_value(source.as_ref(), kind)
    }
}
