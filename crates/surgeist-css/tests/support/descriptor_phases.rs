//! Test-only phase extraction for preserved ordinary descriptor assertions.
#![allow(dead_code)]
use surgeist_css::*;
pub trait OrdinaryCounterValue {
    fn ordinary_system(&self) -> &CssCounterStyleSystem;
    fn ordinary_negative(&self) -> &CssCounterStyleNegative;
    fn ordinary_symbols(&self) -> &CssCounterSymbols;
    fn ordinary_prefix(&self) -> &CssCounterSymbol;
    fn ordinary_suffix(&self) -> &CssCounterSymbol;
    fn ordinary_range(&self) -> &CssCounterStyleRange;
    fn ordinary_pad(&self) -> &CssCounterStylePad;
    fn ordinary_fallback(&self) -> &CssCounterStyleName;
    fn ordinary_additive_symbols(&self) -> &CssCounterAdditiveSymbols;
    fn ordinary_speak_as(&self) -> &CssCounterStyleSpeakAs;
}
impl OrdinaryCounterValue for CssCounterStyleDescriptorValue {
    fn ordinary_system(&self) -> &CssCounterStyleSystem {
        let CssCounterStyleDescriptorValueRef::System(value) = self.view() else {
            panic!("ordinary system phase expected")
        };
        value
    }
    fn ordinary_negative(&self) -> &CssCounterStyleNegative {
        let CssCounterStyleDescriptorValueRef::Negative(value) = self.view() else {
            panic!("ordinary negative phase expected")
        };
        value
    }
    fn ordinary_symbols(&self) -> &CssCounterSymbols {
        let CssCounterStyleDescriptorValueRef::Symbols(value) = self.view() else {
            panic!("ordinary symbols phase expected")
        };
        value
    }
    fn ordinary_prefix(&self) -> &CssCounterSymbol {
        let CssCounterStyleDescriptorValueRef::Prefix(value) = self.view() else {
            panic!("ordinary prefix phase expected")
        };
        value
    }
    fn ordinary_suffix(&self) -> &CssCounterSymbol {
        let CssCounterStyleDescriptorValueRef::Suffix(value) = self.view() else {
            panic!("ordinary suffix phase expected")
        };
        value
    }
    fn ordinary_range(&self) -> &CssCounterStyleRange {
        let CssCounterStyleDescriptorValueRef::Range(value) = self.view() else {
            panic!("ordinary range phase expected")
        };
        value
    }
    fn ordinary_pad(&self) -> &CssCounterStylePad {
        let CssCounterStyleDescriptorValueRef::Pad(value) = self.view() else {
            panic!("ordinary pad phase expected")
        };
        value
    }
    fn ordinary_fallback(&self) -> &CssCounterStyleName {
        let CssCounterStyleDescriptorValueRef::Fallback(value) = self.view() else {
            panic!("ordinary fallback phase expected")
        };
        value
    }
    fn ordinary_additive_symbols(&self) -> &CssCounterAdditiveSymbols {
        let CssCounterStyleDescriptorValueRef::AdditiveSymbols(value) = self.view() else {
            panic!("ordinary additive_symbols phase expected")
        };
        value
    }
    fn ordinary_speak_as(&self) -> &CssCounterStyleSpeakAs {
        let CssCounterStyleDescriptorValueRef::SpeakAs(value) = self.view() else {
            panic!("ordinary speak_as phase expected")
        };
        value
    }
}
pub trait OrdinaryFeatureIndexes {
    fn ordinary_indexes(&self) -> &[CssFontFeatureValueIndex];
}
impl OrdinaryFeatureIndexes for CssFontFeatureValue {
    fn ordinary_indexes(&self) -> &[CssFontFeatureValueIndex] {
        let CssFontFeatureValueRef::Indexes(value) = self.view() else {
            panic!("ordinary feature indexes expected")
        };
        value
    }
}
impl OrdinaryFeatureIndexes for CssFontFeatureValueDefinition {
    fn ordinary_indexes(&self) -> &[CssFontFeatureValueIndex] {
        self.value().ordinary_indexes()
    }
}
pub trait OrdinaryFeatureDisplay {
    fn ordinary_display(&self) -> CssFontDisplay;
}
impl OrdinaryFeatureDisplay for CssFontFeatureDisplayValue {
    fn ordinary_display(&self) -> CssFontDisplay {
        let CssFontFeatureDisplayValueRef::Ordinary(value) = self.view() else {
            panic!("ordinary font display expected")
        };
        value
    }
}

pub fn checked_feature_value(
    kind: CssFontFeatureValueKind,
    spellings: &[&str],
) -> Result<CssFontFeatureValue, CssFontFeatureValueError> {
    let mut components = Vec::new();
    for spelling in spellings {
        if !components.is_empty() {
            components.push(CssComponentValue::try_token(" ").unwrap());
        }
        components.push(CssComponentValue::try_number(spelling).unwrap());
    }
    CssFontFeatureValue::try_from_components(kind, CssComponentValues::try_new(components).unwrap())
}
