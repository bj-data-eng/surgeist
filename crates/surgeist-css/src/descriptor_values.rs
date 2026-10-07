//! Source-produced descriptor values without invented descriptor occurrences.

use crate::syntax::CssCounterStyleDescriptor;
use crate::{
    CssComponentValues, CssCounterAdditiveSymbols, CssCounterStyleDescriptorKind,
    CssCounterStyleName, CssCounterStyleNegative, CssCounterStylePad, CssCounterStyleRange,
    CssCounterStyleSpeakAs, CssCounterStyleSystem, CssCounterSymbol, CssCounterSymbols,
    CssDescriptorOccurrence, CssFontDisplay, CssFontFeatureValueIndex, CssFontFeatureValueKind,
    CssParsedOrigin,
};

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
}

impl CounterStyleValueData {
    pub(crate) fn into_occurrence(
        self,
        name: CssParsedOrigin,
        origin: CssParsedOrigin,
        components: CssComponentValues,
    ) -> CssCounterStyleDescriptor {
        macro_rules! occurrence {
            ($value:expr) => {
                CssDescriptorOccurrence::from_parsed($value, name, origin, components)
            };
        }
        match self {
            Self::System(value) => CssCounterStyleDescriptor::System(occurrence!(value)),
            Self::Negative(value) => CssCounterStyleDescriptor::Negative(occurrence!(value)),
            Self::Symbols(value) => CssCounterStyleDescriptor::Symbols(occurrence!(value)),
            Self::Prefix(value) => CssCounterStyleDescriptor::Prefix(occurrence!(value)),
            Self::Suffix(value) => CssCounterStyleDescriptor::Suffix(occurrence!(value)),
            Self::Range(value) => CssCounterStyleDescriptor::Range(occurrence!(value)),
            Self::Pad(value) => CssCounterStyleDescriptor::Pad(occurrence!(value)),
            Self::Fallback(value) => CssCounterStyleDescriptor::Fallback(occurrence!(value)),
            Self::AdditiveSymbols(value) => {
                CssCounterStyleDescriptor::AdditiveSymbols(occurrence!(value))
            }
            Self::SpeakAs(value) => CssCounterStyleDescriptor::SpeakAs(occurrence!(value)),
        }
    }
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
}

/// One grammar-valid counter descriptor parsed from its complete raw value source.
///
/// Components and origin include surrounding trivia. This value does not impose
/// cross-descriptor system/symbol requirements or resolve counter-style references.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStyleDescriptorValue {
    data: CounterStyleValueData,
    components: CssComponentValues,
    origin: CssParsedOrigin,
}

impl CssCounterStyleDescriptorValue {
    pub(crate) fn from_parsed(
        data: CounterStyleValueData,
        components: CssComponentValues,
        origin: CssParsedOrigin,
    ) -> Self {
        Self {
            data,
            components,
            origin,
        }
    }

    /// Returns the selected descriptor grammar, derived from its semantic value.
    #[must_use]
    pub const fn kind(&self) -> CssCounterStyleDescriptorKind {
        match &self.data {
            CounterStyleValueData::System(_) => CssCounterStyleDescriptorKind::System,
            CounterStyleValueData::Negative(_) => CssCounterStyleDescriptorKind::Negative,
            CounterStyleValueData::Symbols(_) => CssCounterStyleDescriptorKind::Symbols,
            CounterStyleValueData::Prefix(_) => CssCounterStyleDescriptorKind::Prefix,
            CounterStyleValueData::Suffix(_) => CssCounterStyleDescriptorKind::Suffix,
            CounterStyleValueData::Range(_) => CssCounterStyleDescriptorKind::Range,
            CounterStyleValueData::Pad(_) => CssCounterStyleDescriptorKind::Pad,
            CounterStyleValueData::Fallback(_) => CssCounterStyleDescriptorKind::Fallback,
            CounterStyleValueData::AdditiveSymbols(_) => {
                CssCounterStyleDescriptorKind::AdditiveSymbols
            }
            CounterStyleValueData::SpeakAs(_) => CssCounterStyleDescriptorKind::SpeakAs,
        }
    }

    /// Returns every component of the complete value input, including trivia.
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }

    /// Returns the complete original input's parsed origin.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
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
        }
    }
}

/// The outer `@font-feature-values` font-display value and its complete source.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFeatureDisplayValue {
    value: CssFontDisplay,
    components: CssComponentValues,
    origin: CssParsedOrigin,
}

impl CssFontFeatureDisplayValue {
    pub(crate) fn from_parsed(
        value: CssFontDisplay,
        components: CssComponentValues,
        origin: CssParsedOrigin,
    ) -> Self {
        Self {
            value,
            components,
            origin,
        }
    }

    /// Returns the admitted font-display keyword.
    #[must_use]
    pub const fn value(&self) -> CssFontDisplay {
        self.value
    }

    /// Returns every component of the complete value input, including trivia.
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }

    /// Returns the complete original input's parsed origin.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}

/// Exact nonnegative index tokens admitted by one subsidiary feature-value grammar.
///
/// Indexes retain their individual Number origins; components and carrier origin
/// retain the complete input. No friendly definition name is fabricated.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFeatureValueIndexes {
    kind: CssFontFeatureValueKind,
    indexes: Vec<CssFontFeatureValueIndex>,
    components: CssComponentValues,
    origin: CssParsedOrigin,
}

impl CssFontFeatureValueIndexes {
    pub(crate) fn from_parsed(
        kind: CssFontFeatureValueKind,
        indexes: Vec<CssFontFeatureValueIndex>,
        components: CssComponentValues,
        origin: CssParsedOrigin,
    ) -> Self {
        Self {
            kind,
            indexes,
            components,
            origin,
        }
    }

    /// Returns the selected subsidiary grammar.
    #[must_use]
    pub const fn kind(&self) -> CssFontFeatureValueKind {
        self.kind
    }

    /// Borrows the exact normalized indexes in authored order.
    #[must_use]
    pub fn indexes(&self) -> &[CssFontFeatureValueIndex] {
        &self.indexes
    }

    /// Returns every component of the complete value input, including trivia.
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }

    /// Returns the complete original input's parsed origin.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}
