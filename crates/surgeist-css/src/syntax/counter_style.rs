//! Authored Counter Styles models and descriptor identity.

use super::{CssContentString, CssDescriptorOccurrence, CssIdent, CssImage, CssIntegerValue};
use crate::source::{CssParsedOrigin, CssSourcePosition};

/// One syntactically valid parser-produced Counter Styles 3 rule.
/// A rule with missing or insufficient symbols remains authored syntax without
/// necessarily defining a usable counter style for downstream execution.
///
/// The private fields couple a checked name, a valid effective descriptor
/// combination, and the authored at-keyword position. This authored model does
/// not register, resolve, inherit, or evaluate a counter style.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStyleRule {
    name: CssCounterStyleName,
    descriptors: Box<CssCounterStyleDescriptors>,
    position: CssSourcePosition,
    parsed_name: Option<CssParsedOrigin>,
}

impl CssCounterStyleRule {
    #[must_use]
    pub(crate) fn new(
        name: CssCounterStyleName,
        descriptors: CssCounterStyleDescriptors,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            descriptors: Box::new(descriptors),
            position,
            parsed_name: None,
        }
    }

    pub(crate) fn from_parsed(
        name: CssCounterStyleName,
        descriptors: CssCounterStyleDescriptors,
        position: CssSourcePosition,
        parsed_name: CssParsedOrigin,
    ) -> Self {
        Self {
            parsed_name: Some(parsed_name),
            ..Self::new(name, descriptors, position)
        }
    }

    /// Borrows the genuine original name token, before predefined-name normalization.
    /// Semantic construction does not invent a parsed source origin.
    #[must_use]
    pub const fn parsed_name(&self) -> Option<&CssParsedOrigin> {
        self.parsed_name.as_ref()
    }

    /// Returns the checked, case-sensitive authored counter-style name.
    #[must_use]
    pub const fn name(&self) -> &CssCounterStyleName {
        &self.name
    }

    /// Returns the valid authored descriptor collection.
    #[must_use]
    pub const fn descriptors(&self) -> &CssCounterStyleDescriptors {
        &self.descriptors
    }

    /// Returns the semantic source position of the rule's at-keyword.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

/// The valid effective core descriptors of an authored counter-style rule.
///
/// Every valid occurrence remains available in source order. Named accessors
/// select the last valid occurrence. An omitted `system` has the authored CSS
/// default `symbolic`; this default is documented rather than represented by a
/// forged source occurrence.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStyleDescriptors {
    system: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    negative: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    symbols: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    prefix: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    suffix: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    range: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    pad: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    fallback: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    additive_symbols: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    speak_as: Option<CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>>,
    occurrences: Vec<CssCounterStyleDescriptor>,
}

impl CssCounterStyleDescriptors {
    pub(crate) fn from_occurrences(
        occurrences: Vec<CssCounterStyleDescriptor>,
    ) -> Result<Self, CssCounterStyleCombinationIssue> {
        let mut system = None;
        let mut negative = None;
        let mut symbols = None;
        let mut prefix = None;
        let mut suffix = None;
        let mut range = None;
        let mut pad = None;
        let mut fallback = None;
        let mut additive_symbols = None;
        let mut speak_as = None;

        for descriptor in &occurrences {
            match descriptor {
                CssCounterStyleDescriptor::System(value) => system = Some(value.clone()),
                CssCounterStyleDescriptor::Negative(value) => negative = Some(value.clone()),
                CssCounterStyleDescriptor::Symbols(value) => symbols = Some(value.clone()),
                CssCounterStyleDescriptor::Prefix(value) => prefix = Some(value.clone()),
                CssCounterStyleDescriptor::Suffix(value) => suffix = Some(value.clone()),
                CssCounterStyleDescriptor::Range(value) => range = Some(value.clone()),
                CssCounterStyleDescriptor::Pad(value) => pad = Some(value.clone()),
                CssCounterStyleDescriptor::Fallback(value) => fallback = Some(value.clone()),
                CssCounterStyleDescriptor::AdditiveSymbols(value) => {
                    additive_symbols = Some(value.clone());
                }
                CssCounterStyleDescriptor::SpeakAs(value) => speak_as = Some(value.clone()),
            }
        }

        // Counter Styles 3 §§3.1 and 3.8 distinguish a syntactically valid
        // rule from a rule that defines a usable counter style. Missing or
        // insufficient symbols remain authored syntax; definition selection
        // and integer-to-representation execution belong downstream. §3.1.7
        // alone invalidates a rule that combines extends with symbol descriptors.
        if let Some(system) = &system
            && matches!(
                system.value().view(),
                crate::CssCounterStyleDescriptorValueRef::System(CssCounterStyleSystem::Extends(_))
            )
            && (symbols.is_some() || additive_symbols.is_some())
        {
            let mut conflicting = Vec::new();
            if symbols.is_some() {
                conflicting.push("symbols");
            }
            if additive_symbols.is_some() {
                conflicting.push("additive-symbols");
            }
            return Err(CssCounterStyleCombinationIssue::new(
                system.position(),
                "system",
                conflicting,
            ));
        }

        Ok(Self {
            system,
            negative,
            symbols,
            prefix,
            suffix,
            range,
            pad,
            fallback,
            additive_symbols,
            speak_as,
            occurrences,
        })
    }

    /// Returns the effective last valid authored `system` occurrence.
    #[must_use]
    pub const fn system(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.system.as_ref()
    }

    /// Returns the effective last valid authored `negative` occurrence.
    #[must_use]
    pub const fn negative(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.negative.as_ref()
    }

    /// Returns the effective last valid authored `symbols` occurrence.
    #[must_use]
    pub const fn symbols(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.symbols.as_ref()
    }

    /// Returns the effective last valid authored `prefix` occurrence.
    #[must_use]
    pub const fn prefix(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.prefix.as_ref()
    }

    /// Returns the effective last valid authored `suffix` occurrence.
    #[must_use]
    pub const fn suffix(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.suffix.as_ref()
    }

    /// Returns the effective last valid authored `range` occurrence.
    #[must_use]
    pub const fn range(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.range.as_ref()
    }

    /// Returns the effective last valid authored `pad` occurrence.
    #[must_use]
    pub const fn pad(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.pad.as_ref()
    }

    /// Returns the effective last valid authored `fallback` occurrence.
    #[must_use]
    pub const fn fallback(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.fallback.as_ref()
    }

    /// Returns the effective last valid authored `additive-symbols` occurrence.
    #[must_use]
    pub const fn additive_symbols(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.additive_symbols.as_ref()
    }

    /// Returns the effective last valid authored `speak-as` occurrence.
    #[must_use]
    pub const fn speak_as(
        &self,
    ) -> Option<&CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>> {
        self.speak_as.as_ref()
    }

    /// Returns every valid authored core descriptor occurrence in source order.
    pub fn occurrences(&self) -> impl ExactSizeIterator<Item = CssCounterStyleDescriptorRef<'_>> {
        self.occurrences
            .iter()
            .map(CssCounterStyleDescriptor::as_ref)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssCounterStyleDescriptor {
    System(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Negative(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Symbols(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Prefix(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Suffix(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Range(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Pad(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Fallback(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    AdditiveSymbols(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    SpeakAs(CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
}

impl CssCounterStyleDescriptor {
    fn as_ref(&self) -> CssCounterStyleDescriptorRef<'_> {
        match self {
            Self::System(value) => CssCounterStyleDescriptorRef::System(value),
            Self::Negative(value) => CssCounterStyleDescriptorRef::Negative(value),
            Self::Symbols(value) => CssCounterStyleDescriptorRef::Symbols(value),
            Self::Prefix(value) => CssCounterStyleDescriptorRef::Prefix(value),
            Self::Suffix(value) => CssCounterStyleDescriptorRef::Suffix(value),
            Self::Range(value) => CssCounterStyleDescriptorRef::Range(value),
            Self::Pad(value) => CssCounterStyleDescriptorRef::Pad(value),
            Self::Fallback(value) => CssCounterStyleDescriptorRef::Fallback(value),
            Self::AdditiveSymbols(value) => CssCounterStyleDescriptorRef::AdditiveSymbols(value),
            Self::SpeakAs(value) => CssCounterStyleDescriptorRef::SpeakAs(value),
        }
    }
}

/// A borrowed valid authored core counter-style descriptor occurrence.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleDescriptorRef<'a> {
    System(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Negative(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Symbols(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Prefix(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Suffix(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Range(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Pad(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    Fallback(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    AdditiveSymbols(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
    SpeakAs(&'a CssDescriptorOccurrence<crate::CssCounterStyleDescriptorValue>),
}

/// One authored Counter Styles 3 system choice.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleSystem {
    Cyclic,
    Numeric,
    Alphabetic,
    Symbolic,
    Additive,
    Fixed(CssCounterStyleFixedSystem),
    Extends(CssCounterStyleName),
}

/// The optional starting integer of an authored `fixed` system.
/// Calculation graphs remain distinct from literal tokens and omitted defaults.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStyleFixedSystem {
    first_symbol_value: Option<CssIntegerValue>,
}

impl CssCounterStyleFixedSystem {
    #[must_use]
    pub const fn new(first_symbol_value: Option<CssIntegerValue>) -> Self {
        Self { first_symbol_value }
    }

    #[must_use]
    pub const fn first_symbol_value(&self) -> Option<&CssIntegerValue> {
        self.first_symbol_value.as_ref()
    }
}

/// One or two authored symbols applied around negative counter representations.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStyleNegative {
    prefix: CssCounterSymbol,
    suffix: Option<CssCounterSymbol>,
}

impl CssCounterStyleNegative {
    /// Preserves one required checked symbol and an optional second symbol in order.
    /// No counter representation, applicability, or resource is resolved here.
    #[must_use]
    pub const fn new(prefix: CssCounterSymbol, suffix: Option<CssCounterSymbol>) -> Self {
        Self { prefix, suffix }
    }

    /// Returns the required symbol prepended to a negative representation.
    #[must_use]
    pub const fn prefix(&self) -> &CssCounterSymbol {
        &self.prefix
    }

    /// Returns the optional symbol appended to a negative representation.
    #[must_use]
    pub const fn suffix(&self) -> Option<&CssCounterSymbol> {
        self.suffix.as_ref()
    }
}

/// One authored `range` descriptor value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleRange {
    Auto,
    Ranges(CssCounterStyleRanges),
}

/// A nonempty authored comma-separated list of inclusive counter ranges.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStyleRanges {
    ranges: Vec<CssCounterStyleRangeInterval>,
}

impl CssCounterStyleRanges {
    #[must_use]
    pub fn try_new(ranges: Vec<CssCounterStyleRangeInterval>) -> Option<Self> {
        (!ranges.is_empty()).then_some(Self { ranges })
    }

    /// Returns the valid inclusive ranges in authored order.
    #[must_use]
    pub fn ranges(&self) -> &[CssCounterStyleRangeInterval] {
        &self.ranges
    }
}

/// One inclusive authored range. Literal bounds are checked for reversed order;
/// calculation bounds remain symbolic until downstream integer computation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStyleRangeInterval {
    lower: CssCounterStyleRangeBound,
    upper: CssCounterStyleRangeBound,
}

impl CssCounterStyleRangeInterval {
    #[must_use]
    pub fn try_new(
        lower: CssCounterStyleRangeBound,
        upper: CssCounterStyleRangeBound,
    ) -> Option<Self> {
        if let (
            CssCounterStyleRangeBound::Integer(CssIntegerValue::Literal(lower)),
            CssCounterStyleRangeBound::Integer(CssIntegerValue::Literal(upper)),
        ) = (&lower, &upper)
            && lower.compare_value(upper).is_gt()
        {
            return None;
        }
        Some(Self { lower, upper })
    }

    /// Returns the inclusive lower bound, where `Infinite` means negative infinity.
    #[must_use]
    pub const fn lower(&self) -> &CssCounterStyleRangeBound {
        &self.lower
    }

    /// Returns the inclusive upper bound, where `Infinite` means positive infinity.
    #[must_use]
    pub const fn upper(&self) -> &CssCounterStyleRangeBound {
        &self.upper
    }
}

/// One authored finite integer or contextual `infinite` range bound.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleRangeBound {
    Integer(CssIntegerValue),
    Infinite,
}

/// One valid authored zero-padding descriptor.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStylePad {
    minimum_length: CssIntegerValue,
    symbol: CssCounterSymbol,
}

impl CssCounterStylePad {
    #[must_use]
    pub fn try_new(minimum_length: CssIntegerValue, symbol: CssCounterSymbol) -> Option<Self> {
        (!matches!(&minimum_length, CssIntegerValue::Literal(value) if value.is_negative()))
            .then_some(Self {
                minimum_length,
                symbol,
            })
    }

    /// Returns the authored minimum length. Literals are nonnegative; calculations
    /// retain their specified graph before computed integer rounding and clamping.
    #[must_use]
    pub const fn minimum_length(&self) -> &CssIntegerValue {
        &self.minimum_length
    }

    /// Returns the authored padding symbol.
    #[must_use]
    pub const fn symbol(&self) -> &CssCounterSymbol {
        &self.symbol
    }
}

/// A nonempty authored `additive-symbols` list. Adjacent literal weights must
/// strictly descend; comparisons involving calculations remain downstream.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterAdditiveSymbols {
    tuples: Vec<CssCounterAdditiveTuple>,
}

impl CssCounterAdditiveSymbols {
    #[must_use]
    pub fn try_new(tuples: Vec<CssCounterAdditiveTuple>) -> Option<Self> {
        (!tuples.is_empty() && Self::weights_strictly_descend(&tuples)).then_some(Self { tuples })
    }

    pub(crate) fn weights_strictly_descend(tuples: &[CssCounterAdditiveTuple]) -> bool {
        // Counter Styles 3 §3.8 compares integer weights. Imported Values4
        // §11.12 defers calculation rounding and clamping to computed/used
        // values. Only adjacent ordinary literals can be compared here;
        // downstream must check the complete list after resolving all weights.
        tuples
            .windows(2)
            .all(|pair| match (&pair[0].weight, &pair[1].weight) {
                (CssIntegerValue::Literal(left), CssIntegerValue::Literal(right)) => {
                    left.compare_value(right).is_gt()
                }
                _ => true,
            })
    }

    /// Returns the additive tuples in authored order without evaluating weights.
    #[must_use]
    pub fn tuples(&self) -> &[CssCounterAdditiveTuple] {
        &self.tuples
    }
}

/// One authored weight and counter symbol. Ordinary integer weights must be
/// nonnegative; symbolic calculations defer integer rounding and range clamping.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterAdditiveTuple {
    weight: CssIntegerValue,
    symbol: CssCounterSymbol,
}

impl CssCounterAdditiveTuple {
    #[must_use]
    pub fn try_new(weight: CssIntegerValue, symbol: CssCounterSymbol) -> Option<Self> {
        (!matches!(&weight, CssIntegerValue::Literal(value) if value.is_negative()))
            .then_some(Self { weight, symbol })
    }

    /// Returns the authored weight, retaining symbolic integer calculations.
    #[must_use]
    pub const fn weight(&self) -> &CssIntegerValue {
        &self.weight
    }

    /// Returns the counter symbol associated with the weight.
    #[must_use]
    pub const fn symbol(&self) -> &CssCounterSymbol {
        &self.symbol
    }
}

/// One authored Counter Styles 3 speech-synthesis choice.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleSpeakAs {
    Auto,
    Bullets,
    Numbers,
    Words,
    SpellOut,
    CounterStyle(CssCounterStyleName),
}

/// A nonempty authored `symbols` descriptor value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterSymbols {
    symbols: Vec<CssCounterSymbol>,
}

impl CssCounterSymbols {
    #[must_use]
    pub(crate) fn new(symbols: Vec<CssCounterSymbol>) -> Self {
        debug_assert!(!symbols.is_empty());
        Self { symbols }
    }

    #[must_use]
    pub fn symbols(&self) -> &[CssCounterSymbol] {
        &self.symbols
    }
}

/// One typed authored counter symbol.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterSymbol {
    String(CssContentString),
    Ident(CssCounterSymbolIdent),
    Image(CssImage),
}

/// One checked `<custom-ident>` used as an authored counter symbol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterSymbolIdent {
    value: crate::CssContentName,
}

impl CssCounterSymbolIdent {
    /// Constructs a counter symbol from one exact decoded CSS custom identifier.
    /// CSS-wide keywords, `default`, empty values and NUL are rejected.
    /// Other identifiers retain their decoded case and punctuation.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let ident = CssIdent::try_new(value).ok()?;
        crate::CssContentName::try_new(ident).map(|value| Self { value })
    }

    /// Returns the exact, case-sensitive decoded identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CssCounterStyleCombinationIssue {
    position: Option<CssSourcePosition>,
    responsible: &'static str,
    conflicting: Vec<&'static str>,
}

impl CssCounterStyleCombinationIssue {
    fn new(
        position: CssSourcePosition,
        responsible: &'static str,
        conflicting: Vec<&'static str>,
    ) -> Self {
        Self::new_optional(Some(position), responsible, conflicting)
    }

    fn new_optional(
        position: Option<CssSourcePosition>,
        responsible: &'static str,
        conflicting: Vec<&'static str>,
    ) -> Self {
        Self {
            position,
            responsible,
            conflicting,
        }
    }

    pub(crate) const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }

    pub(crate) const fn responsible(&self) -> &'static str {
        self.responsible
    }

    pub(crate) fn conflicting(&self) -> &[&'static str] {
        &self.conflicting
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterStyleName {
    name: crate::CssCounterStyleReference,
}

impl CssCounterStyleName {
    /// Constructs a decoded counter-style name, including names requiring CSS escapes.
    /// Predefined names normalize to lowercase; arbitrary custom names retain case.
    /// CSS-wide keywords, `default`, `none`, empty values and NUL are rejected.
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let ident = CssIdent::try_new(name).ok()?;
        crate::CssCounterStyleReference::try_new(ident).map(|name| Self { name })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

/// One of the ten represented Counter Styles 3 descriptor fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleDescriptorKind {
    System,
    Negative,
    Prefix,
    Suffix,
    Range,
    Pad,
    Fallback,
    Symbols,
    AdditiveSymbols,
    SpeakAs,
}

impl CssCounterStyleDescriptorKind {
    /// Returns the literal CSS descriptor name for this grammar.
    #[must_use]
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Negative => "negative",
            Self::Prefix => "prefix",
            Self::Suffix => "suffix",
            Self::Range => "range",
            Self::Pad => "pad",
            Self::Fallback => "fallback",
            Self::Symbols => "symbols",
            Self::AdditiveSymbols => "additive-symbols",
            Self::SpeakAs => "speak-as",
        }
    }
}
