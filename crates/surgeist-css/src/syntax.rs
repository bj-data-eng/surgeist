//! Authored CSS syntax values produced by this crate's parser.
//!
//! Property-coupled declarations represent CSS-owned authored syntax without a
//! broad property/value cross product. Property-specific parsers decide which
//! value forms are accepted. CSS owns intrinsic declaration expansion; downstream
//! crates own substitution environments, cascade, and contextual resolution.
//!
//! Parsed declarations retain authored source locations, while checked Rust
//! construction preserves supplied token origins without inventing coordinates.
//! Downstream adapters can inspect either without depending on parser internals.

pub(crate) use crate::container_features::*;
use crate::font_variant::CssFontVariant;
pub(crate) use crate::media::*;
pub(crate) use crate::media_features::*;
pub(crate) use crate::numeric::*;
pub use crate::page_line_minimum::CssPageLineMinimum;
pub use crate::rotate::{CssRotate, CssRotateAxis, CssRotateValues};
pub use crate::text_controls::*;
use crate::{
    CssAngleLiteral, CssAngleOrZero, CssAngleValue, CssColorNumberLiteral,
    CssColorPercentageLiteral, CssColorScalarError, CssDuration, CssFontSize, CssFontStyle,
    CssFontWeight, CssFontWidthKeyword, CssLineHeight, CssTimeValue, CssValueOrigin,
};
use crate::{
    CssColorProfileRule, CssContainerScrollQuery, CssContainerStyleQuery, CssFontFeatureValuesRule,
    CssFontPaletteValuesRule,
};
pub(crate) use crate::{
    CssSpecifiedLength, CssSpecifiedLengthPercentage, CssSpecifiedNonNegativeLength,
    CssSpecifiedNonNegativeLengthPercentage, CssSpecifiedNonNegativeNumber,
    CssSpecifiedNonNegativePercentage, CssSpecifiedNumber, CssSpecifiedPercentage,
};
use std::sync::Arc;

use crate::component_values::{CssComponentValues, CssParsedOrigin};

pub(crate) use crate::properties::CssKnownDeclaration;
use crate::properties::CssKnownProperty;
use crate::source::CssSourcePosition;

// Semantic syntax owners compare exact numeric structure while leaves retain
// provenance-sensitive equality. This also applies to symbolic AST branches;
// raw calculations continue to compare their diagnostic origins separately.
fn optional_numeric_eq<T>(
    left: Option<&T>,
    right: Option<&T>,
    equal: impl FnOnce(&T, &T) -> bool,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => equal(left, right),
        (None, None) => true,
        _ => false,
    }
}

macro_rules! numeric_fields_eq {
    ($owner:ident, [$($required:ident),*], [$($optional:ident),*], [$($ordinary:ident),*]) => {
        impl PartialEq for $owner {
            fn eq(&self, other: &Self) -> bool {
                true $( && self.$required.structural_eq(&other.$required) )*
                $( && optional_numeric_eq(self.$optional.as_ref(), other.$optional.as_ref(), |left, right| left.structural_eq(right)) )*
                $( && self.$ordinary == other.$ordinary )*
            }
        }
    };
}

/// A parser-produced authored stylesheet and optional legacy encoding metadata.
///
/// The private fields guarantee that every retained rule is valid authored CSS
/// syntax and that at most one valid leading encoding declaration is recorded.
/// The metadata does not decode the already-UTF-8 Rust input, and the sheet does
/// not apply cascade, substitution, selector matching, or resource loading.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CssSheet {
    encoding: Option<CssEncodingDeclaration>,
    rules: Vec<CssRule>,
}

impl CssSheet {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            encoding: None,
            rules: Vec::new(),
        }
    }

    pub(crate) fn set_encoding(&mut self, encoding: CssEncodingDeclaration) {
        debug_assert!(self.encoding.is_none());
        self.encoding = Some(encoding);
    }

    pub(crate) fn push_rule(&mut self, rule: CssRule) {
        self.rules.push(rule);
    }

    /// Returns the optional valid leading legacy encoding declaration.
    ///
    /// This authored metadata is recorded once and does not perform byte
    /// decoding or change the UTF-8 source supplied to [`crate::parse_sheet`].
    #[must_use]
    pub const fn encoding(&self) -> Option<&CssEncodingDeclaration> {
        self.encoding.as_ref()
    }

    /// Returns the valid authored rules retained in source order.
    ///
    /// Recovery diagnostics remain on the parse report. Reading the rules does
    /// not imply that the source was clean or perform downstream CSS behavior.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }
}

/// Metadata for one valid leading legacy `@charset` declaration.
///
/// The parser-owned private fields preserve a non-empty authored label and its
/// source position. This authored metadata records legacy syntax only: Rust
/// input is already UTF-8, so it neither detects nor decodes bytes and it does
/// not participate in cascade, substitution, matching, or resource loading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssEncodingDeclaration {
    label: String,
    position: CssSourcePosition,
}

impl CssEncodingDeclaration {
    pub(crate) fn new(label: impl Into<String>, position: CssSourcePosition) -> Self {
        let label = label.into();
        debug_assert!(!label.is_empty());
        Self { label, position }
    }

    /// Returns the non-empty authored encoding label.
    ///
    /// The label is metadata and is not normalized, resolved, or used to decode
    /// the already-UTF-8 stylesheet input.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the semantic source position of the declaration's at-keyword.
    ///
    /// This parser-produced position is diagnostic provenance only and does not
    /// load or reinterpret source bytes.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

/// One checked rule in the authored stylesheet phase.
///
/// The non-exhaustive union contains only syntax that passed its rule grammar;
/// discarded source units are represented by report diagnostics instead. A rule
/// does not perform cascade, substitution, selector matching, query evaluation,
/// resource loading, or contextual resolution.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssRule {
    CustomMedia(crate::CssCustomMediaRule),
    Import(CssImportRule),
    Namespace(CssNamespaceRule),
    CounterStyle(CssCounterStyleRule),
    Page(CssPageRule),
    LayerStatement(CssLayerStatementRule),
    LayerBlock(CssLayerBlockRule),
    FontFace(CssFontFaceRule),
    FontFeatureValues(CssFontFeatureValuesRule),
    FontPaletteValues(CssFontPaletteValuesRule),
    ColorProfile(CssColorProfileRule),
    Keyframes(CssKeyframesRule),
    Style(CssStyleRule),
    NestedDeclarations(CssNestedDeclarationsRule),
    Media(CssMediaRule),
    Supports(CssSupportsRule),
    SupportsCondition(crate::CssSupportsConditionRule),
    Container(CssContainerRule),
    Scope(CssScopeRule),
}

/// One valid parser-produced CSS2 page rule.
///
/// The private fields retain the optional authored page pseudo selector, the
/// ordered page-context margin declarations, and the at-keyword position.
/// This authored model does not paginate, cascade, match pages, or resolve
/// lengths and percentages.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageRule {
    selector: Option<CssPageSelector>,
    declarations: CssDeclarationList,
    position: CssSourcePosition,
}

impl CssPageRule {
    #[must_use]
    pub(crate) const fn new(
        selector: Option<CssPageSelector>,
        declarations: CssDeclarationList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            selector,
            declarations,
            position,
        }
    }

    /// Returns the authored page pseudo selector, or `None` for the default
    /// page form with an empty prelude.
    #[must_use]
    pub const fn selector(&self) -> Option<CssPageSelector> {
        self.selector
    }

    /// Returns the valid page-context margin declarations in authored order.
    #[must_use]
    pub const fn declarations(&self) -> &CssDeclarationList {
        &self.declarations
    }

    /// Returns the semantic source position of the rule's at-keyword.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

/// The finite CSS2 pseudo-page selector set accepted by `@page`.
///
/// The default page form is represented by `None` from
/// [`CssPageRule::selector`], keeping absence distinct from every authored
/// pseudo selector.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssPageSelector {
    Left,
    Right,
    First,
}

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
    system: Option<CssDescriptorOccurrence<CssCounterStyleSystem>>,
    negative: Option<CssDescriptorOccurrence<CssCounterStyleNegative>>,
    symbols: Option<CssDescriptorOccurrence<CssCounterSymbols>>,
    prefix: Option<CssDescriptorOccurrence<CssCounterSymbol>>,
    suffix: Option<CssDescriptorOccurrence<CssCounterSymbol>>,
    range: Option<CssDescriptorOccurrence<CssCounterStyleRange>>,
    pad: Option<CssDescriptorOccurrence<CssCounterStylePad>>,
    fallback: Option<CssDescriptorOccurrence<CssCounterStyleName>>,
    additive_symbols: Option<CssDescriptorOccurrence<CssCounterAdditiveSymbols>>,
    speak_as: Option<CssDescriptorOccurrence<CssCounterStyleSpeakAs>>,
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
            && matches!(system.value(), CssCounterStyleSystem::Extends(_))
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
    pub const fn system(&self) -> Option<&CssDescriptorOccurrence<CssCounterStyleSystem>> {
        self.system.as_ref()
    }

    /// Returns the effective last valid authored `negative` occurrence.
    #[must_use]
    pub const fn negative(&self) -> Option<&CssDescriptorOccurrence<CssCounterStyleNegative>> {
        self.negative.as_ref()
    }

    /// Returns the effective last valid authored `symbols` occurrence.
    #[must_use]
    pub const fn symbols(&self) -> Option<&CssDescriptorOccurrence<CssCounterSymbols>> {
        self.symbols.as_ref()
    }

    /// Returns the effective last valid authored `prefix` occurrence.
    #[must_use]
    pub const fn prefix(&self) -> Option<&CssDescriptorOccurrence<CssCounterSymbol>> {
        self.prefix.as_ref()
    }

    /// Returns the effective last valid authored `suffix` occurrence.
    #[must_use]
    pub const fn suffix(&self) -> Option<&CssDescriptorOccurrence<CssCounterSymbol>> {
        self.suffix.as_ref()
    }

    /// Returns the effective last valid authored `range` occurrence.
    #[must_use]
    pub const fn range(&self) -> Option<&CssDescriptorOccurrence<CssCounterStyleRange>> {
        self.range.as_ref()
    }

    /// Returns the effective last valid authored `pad` occurrence.
    #[must_use]
    pub const fn pad(&self) -> Option<&CssDescriptorOccurrence<CssCounterStylePad>> {
        self.pad.as_ref()
    }

    /// Returns the effective last valid authored `fallback` occurrence.
    #[must_use]
    pub const fn fallback(&self) -> Option<&CssDescriptorOccurrence<CssCounterStyleName>> {
        self.fallback.as_ref()
    }

    /// Returns the effective last valid authored `additive-symbols` occurrence.
    #[must_use]
    pub const fn additive_symbols(
        &self,
    ) -> Option<&CssDescriptorOccurrence<CssCounterAdditiveSymbols>> {
        self.additive_symbols.as_ref()
    }

    /// Returns the effective last valid authored `speak-as` occurrence.
    #[must_use]
    pub const fn speak_as(&self) -> Option<&CssDescriptorOccurrence<CssCounterStyleSpeakAs>> {
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
    System(CssDescriptorOccurrence<CssCounterStyleSystem>),
    Negative(CssDescriptorOccurrence<CssCounterStyleNegative>),
    Symbols(CssDescriptorOccurrence<CssCounterSymbols>),
    Prefix(CssDescriptorOccurrence<CssCounterSymbol>),
    Suffix(CssDescriptorOccurrence<CssCounterSymbol>),
    Range(CssDescriptorOccurrence<CssCounterStyleRange>),
    Pad(CssDescriptorOccurrence<CssCounterStylePad>),
    Fallback(CssDescriptorOccurrence<CssCounterStyleName>),
    AdditiveSymbols(CssDescriptorOccurrence<CssCounterAdditiveSymbols>),
    SpeakAs(CssDescriptorOccurrence<CssCounterStyleSpeakAs>),
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
    System(&'a CssDescriptorOccurrence<CssCounterStyleSystem>),
    Negative(&'a CssDescriptorOccurrence<CssCounterStyleNegative>),
    Symbols(&'a CssDescriptorOccurrence<CssCounterSymbols>),
    Prefix(&'a CssDescriptorOccurrence<CssCounterSymbol>),
    Suffix(&'a CssDescriptorOccurrence<CssCounterSymbol>),
    Range(&'a CssDescriptorOccurrence<CssCounterStyleRange>),
    Pad(&'a CssDescriptorOccurrence<CssCounterStylePad>),
    Fallback(&'a CssDescriptorOccurrence<CssCounterStyleName>),
    AdditiveSymbols(&'a CssDescriptorOccurrence<CssCounterAdditiveSymbols>),
    SpeakAs(&'a CssDescriptorOccurrence<CssCounterStyleSpeakAs>),
}

/// One authored Counter Styles 3 system choice.
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterStyleFixedSystem {
    first_symbol_value: Option<crate::CssIntegerLiteral>,
}

impl CssCounterStyleFixedSystem {
    #[must_use]
    pub const fn new(first_symbol_value: Option<crate::CssIntegerLiteral>) -> Self {
        Self { first_symbol_value }
    }

    #[must_use]
    pub const fn first_symbol_value(&self) -> Option<&crate::CssIntegerLiteral> {
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
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleRange {
    Auto,
    Ranges(CssCounterStyleRanges),
}

/// A nonempty authored comma-separated list of inclusive counter ranges.
#[derive(Clone, Debug, Eq, PartialEq)]
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

/// One intrinsically valid inclusive range from a `range` descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
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
            CssCounterStyleRangeBound::Integer(lower),
            CssCounterStyleRangeBound::Integer(upper),
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
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleRangeBound {
    Integer(crate::CssIntegerLiteral),
    Infinite,
}

/// One valid authored zero-padding descriptor.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStylePad {
    minimum_length: crate::CssIntegerLiteral,
    symbol: CssCounterSymbol,
}

impl CssCounterStylePad {
    #[must_use]
    pub fn try_new(
        minimum_length: crate::CssIntegerLiteral,
        symbol: CssCounterSymbol,
    ) -> Option<Self> {
        (!minimum_length.is_negative()).then_some(Self {
            minimum_length,
            symbol,
        })
    }

    /// Returns the nonnegative minimum representation length.
    #[must_use]
    pub const fn minimum_length(&self) -> &crate::CssIntegerLiteral {
        &self.minimum_length
    }

    /// Returns the authored padding symbol.
    #[must_use]
    pub const fn symbol(&self) -> &CssCounterSymbol {
        &self.symbol
    }
}

/// A nonempty, strictly descending authored `additive-symbols` list.
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
        tuples
            .windows(2)
            .all(|pair| pair[0].weight.compare_value(&pair[1].weight).is_gt())
    }

    /// Returns the additive tuples in strictly descending authored weight order.
    #[must_use]
    pub fn tuples(&self) -> &[CssCounterAdditiveTuple] {
        &self.tuples
    }
}

/// One nonnegative integer weight and counter symbol in an additive list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterAdditiveTuple {
    weight: crate::CssIntegerLiteral,
    symbol: CssCounterSymbol,
}

impl CssCounterAdditiveTuple {
    #[must_use]
    pub fn try_new(weight: crate::CssIntegerLiteral, symbol: CssCounterSymbol) -> Option<Self> {
        (!weight.is_negative()).then_some(Self { weight, symbol })
    }

    /// Returns the nonnegative additive weight.
    #[must_use]
    pub const fn weight(&self) -> &crate::CssIntegerLiteral {
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

/// One intrinsically valid authored Namespaces 3 declaration.
///
/// The declaration retains its optional decoded prefix, literal namespace name,
/// and optional parser-produced source position. Placement and redeclaration
/// conformance belong to the declaring stylesheet. It does not resolve namespace
/// names, load resources, or perform selector matching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNamespaceRule {
    prefix: Option<CssNamespacePrefix>,
    name: CssNamespaceName,
    position: Option<CssSourcePosition>,
}

impl CssNamespaceRule {
    /// Constructs a declaration from a checked decoded prefix and a literal name.
    /// An omitted prefix declares the default. Empty names denote the null namespace.
    /// Programmatic construction has no authored source position.
    #[must_use]
    pub const fn new(prefix: Option<CssNamespacePrefix>, name: CssNamespaceName) -> Self {
        Self {
            prefix,
            name,
            position: None,
        }
    }

    pub(crate) const fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Returns the optional exact, case-sensitive decoded namespace prefix.
    #[must_use]
    pub const fn prefix(&self) -> Option<&CssNamespacePrefix> {
        self.prefix.as_ref()
    }

    /// Returns the literal string carried by the authored string or `url()` token.
    #[must_use]
    pub const fn name(&self) -> &CssNamespaceName {
        &self.name
    }

    /// Returns the rule's authored at-keyword position, or `None` for Rust construction.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// One checked, decoded Namespaces 3 prefix.
///
/// Construction accepts exactly one decoded CSS identifier. Prefix equality is
/// exact and case-sensitive.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssNamespacePrefix {
    value: String,
}

impl CssNamespacePrefix {
    /// Constructs a prefix from a decoded CSS identifier, which may require escapes
    /// in its CSS spelling. Empty values and NUL cannot retain identifier identity.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        crate::CssComponentValue::try_ident(value.clone()).ok()?;
        Some(Self { value })
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        debug_assert!(crate::CssComponentValue::try_ident(value.clone()).is_ok());
        Self { value }
    }

    /// Returns the exact, case-sensitive decoded identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// The literal namespace name from a Namespaces 3 declaration.
///
/// Empty values and strings that are not valid URIs remain valid authored
/// syntax. This value is not normalized, resolved, or loaded.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssNamespaceName {
    value: String,
}

impl CssNamespaceName {
    /// Constructs an authored literal namespace name without URI validation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Returns the literal string carried by the authored string or `url()` token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssImportRule {
    target: CssImportTarget,
    layer: Option<CssImportLayer>,
    supports: Option<CssImportSupports>,
    media: Option<CssMediaQueryList>,
    pub(crate) syntax: Box<crate::imports::ImportSyntax>,
}

impl CssImportRule {
    #[must_use]
    pub(crate) fn new(
        target: CssImportTarget,
        layer: Option<CssImportLayer>,
        supports: Option<CssImportSupports>,
        media: Option<CssMediaQueryList>,
        syntax: crate::imports::ImportSyntax,
    ) -> Self {
        Self {
            target,
            layer,
            supports,
            media,
            syntax: Box::new(syntax),
        }
    }

    #[must_use]
    pub const fn target(&self) -> &CssImportTarget {
        &self.target
    }

    #[must_use]
    pub const fn layer(&self) -> Option<&CssImportLayer> {
        self.layer.as_ref()
    }

    /// Returns the optional authored supports condition that follows the import
    /// target and layer clause.
    #[must_use]
    pub const fn supports(&self) -> Option<&CssImportSupports> {
        self.supports.as_ref()
    }

    #[must_use]
    pub const fn media(&self) -> Option<&CssMediaQueryList> {
        self.media.as_ref()
    }

    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        self.syntax.at_keyword.origin()
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImportTarget {
    Url(CssImportUrl),
    String(CssImportString),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssImportUrl {
    url: CssUrl,
}

impl CssImportUrl {
    /// Preserves a shared authored URL, including its function identity and modifiers.
    #[must_use]
    pub fn new(url: CssUrl) -> Self {
        Self { url }
    }

    /// Returns the shared authored URL without losing its function or modifiers.
    #[must_use]
    pub const fn url(&self) -> &CssUrl {
        &self.url
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssImportString {
    value: String,
}

impl CssImportString {
    /// Preserves a decoded authored string, including empty or whitespace values.
    /// Resource resolution and usability belong to the importing layer.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self { value }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImportLayer {
    Anonymous,
    Named(CssLayerName),
}

/// One parser-produced Cascade 4 supports condition on an import rule.
///
/// The condition preserves authored syntax only. It is not evaluated and does
/// not load the imported resource.
#[derive(Clone, Debug, PartialEq)]
pub struct CssImportSupports {
    condition: CssSupportsCondition,
}

impl CssImportSupports {
    #[must_use]
    pub(crate) const fn new(condition: CssSupportsCondition) -> Self {
        Self { condition }
    }

    /// Returns the authored declaration test or full supports condition.
    #[must_use]
    pub const fn condition(&self) -> &CssSupportsCondition {
        &self.condition
    }
}

mod font_face;
pub use font_face::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframesRule {
    name: CssKeyframesName,
    blocks: Vec<CssKeyframeBlock>,
    position: CssSourcePosition,
}

impl CssKeyframesRule {
    #[must_use]
    pub(crate) fn new(
        name: CssKeyframesName,
        blocks: Vec<CssKeyframeBlock>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            blocks,
            position,
        }
    }

    #[must_use]
    pub const fn name(&self) -> &CssKeyframesName {
        &self.name
    }

    #[must_use]
    pub fn blocks(&self) -> &[CssKeyframeBlock] {
        &self.blocks
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssKeyframesName {
    Ident(CssKeyframesIdent),
    String(CssKeyframesString),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssKeyframesString {
    value: String,
}

impl CssKeyframesString {
    /// Preserves an unrestricted decoded animation name, including empty strings.
    /// Unrepresentable string contents fail explicitly during specified serialization.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self { value }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// A parser-produced authored keyframe block with a distinct declaration collection.
///
/// Its private fields couple validated selectors, keyframe-only declarations, and semantic source
/// provenance. It does not apply cascade, resolve substitutions, or interpolate animations.
#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframeBlock {
    selectors: CssKeyframeSelectorList,
    declarations: CssKeyframeDeclarationList,
    position: CssSourcePosition,
}

impl CssKeyframeBlock {
    #[must_use]
    pub(crate) fn new(
        selectors: CssKeyframeSelectorList,
        declarations: CssKeyframeDeclarationList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            selectors,
            declarations,
            position,
        }
    }

    #[must_use]
    pub const fn selectors(&self) -> &CssKeyframeSelectorList {
        &self.selectors
    }

    #[must_use]
    pub const fn declarations(&self) -> &CssKeyframeDeclarationList {
        &self.declarations
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframeSelectorList {
    selectors: Vec<CssKeyframeSelector>,
}

impl CssKeyframeSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssKeyframeSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssKeyframeSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssKeyframeSelector] {
        &self.selectors
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssKeyframeSelector {
    From,
    To,
    Percent(CssKeyframePercent),
}

impl CssKeyframeSelector {
    /// Returns the authored percentage domain, without evaluating calculations.
    #[must_use]
    pub fn offset(&self) -> CssKeyframePercent {
        match self {
            Self::From => CssKeyframePercent {
                value: CssKeyframePercentData::Literal(0.0),
            },
            Self::To => CssKeyframePercent {
                value: CssKeyframePercentData::Literal(100.0),
            },
            Self::Percent(percent) => percent.clone(),
        }
    }
}

/// An authored keyframe percentage: a bounded binary64 literal or symbolic percentage math.
///
/// Literal admission converts to binary64 before checking the inclusive range. Calculations
/// remain authored; their range is checked at computed or used value time downstream.
#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframePercent {
    value: CssKeyframePercentData,
}

#[derive(Clone, Debug, PartialEq)]
enum CssKeyframePercentData {
    Literal(f64),
    Calculation(CssSpecifiedPercentage),
}

impl CssKeyframePercent {
    /// Accepts a finite binary64 percentage within the inclusive range 0 through 100.
    #[must_use]
    pub fn try_new(value: f64) -> Option<Self> {
        if value.is_finite() && (0.0..=100.0).contains(&value) {
            Some(Self {
                value: CssKeyframePercentData::Literal(value),
            })
        } else {
            None
        }
    }

    /// Retains a checked percentage function, or range-checks a bare percentage root.
    ///
    /// The shared percentage owner classifies the already checked root. Bare
    /// roots use the same binary64 literal boundary as [`Self::try_new`]. Function roots retain
    /// exact authored coefficients and provenance without evaluation or clamping.
    #[must_use]
    pub fn try_from_calculation(calculation: CssPercentageCalculation) -> Option<Self> {
        let value = CssSpecifiedPercentage::try_from_calculation(calculation).ok()?;
        if let Some(component) = value.literal_component() {
            let crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Percentage(number)) =
                component.view()
            else {
                return None;
            };
            Self::try_new(number.representation().parse::<f64>().ok()?)
        } else {
            Some(Self {
                value: CssKeyframePercentData::Calculation(value),
            })
        }
    }

    /// Returns the supported literal scalar, or `None` for authored calculations.
    #[must_use]
    pub const fn literal_value(&self) -> Option<f64> {
        match &self.value {
            CssKeyframePercentData::Literal(value) => Some(*value),
            CssKeyframePercentData::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic percentage function without computing its offset.
    #[must_use]
    pub fn calculation(&self) -> Option<&CssPercentageCalculation> {
        match &self.value {
            CssKeyframePercentData::Literal(_) => None,
            CssKeyframePercentData::Calculation(value) => value.calculation(),
        }
    }

    pub(crate) fn specified_calculation(&self) -> Option<&CssSpecifiedPercentage> {
        match &self.value {
            CssKeyframePercentData::Literal(_) => None,
            CssKeyframePercentData::Calculation(value) => Some(value),
        }
    }
}

/// A checked authored descriptor value and its semantic name position.
///
/// The private fields preserve the coupling between a validated descriptor value and the source
/// position of its descriptor-name start. Construction is parser-owned, so callers cannot forge
/// provenance. Parsed counter descriptors also retain their original name/value regions and
/// complete component trees. Semantic construction and descriptor parsers without retained
/// lexical metadata do not invent parsed origins. This occurrence does not apply descriptor
/// matching or load resources.
///
/// ```compile_fail
/// use surgeist_css::{CssDescriptorOccurrence, CssFontDisplay};
/// let _ = CssDescriptorOccurrence { value: CssFontDisplay::Swap, position: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssDescriptorOccurrence<T> {
    value: T,
    position: CssSourcePosition,
    parsed: Option<Arc<CssParsedDescriptor>>,
}

#[derive(Debug, PartialEq)]
struct CssParsedDescriptor {
    name: CssParsedOrigin,
    value: CssParsedOrigin,
    components: CssComponentValues,
}

impl<T> CssDescriptorOccurrence<T> {
    #[cfg(test)]
    pub(crate) const fn new(value: T, position: CssSourcePosition) -> Self {
        Self {
            value,
            position,
            parsed: None,
        }
    }

    pub(crate) fn from_parsed(
        value: T,
        name: CssParsedOrigin,
        value_origin: CssParsedOrigin,
        components: CssComponentValues,
    ) -> Self {
        Self {
            value,
            position: name.span().start(),
            parsed: Some(Arc::new(CssParsedDescriptor {
                name,
                value: value_origin,
                components,
            })),
        }
    }

    /// Borrows the original descriptor-name token when retained by its parser.
    #[must_use]
    pub fn parsed_name(&self) -> Option<&CssParsedOrigin> {
        self.parsed.as_ref().map(|parsed| &parsed.name)
    }

    /// Borrows the original descriptor-value region when retained by its parser.
    #[must_use]
    pub fn parsed_value(&self) -> Option<&CssParsedOrigin> {
        self.parsed.as_ref().map(|parsed| &parsed.value)
    }

    /// Borrows every original value component, including trivia and nested delimiters.
    /// The retained source stays distinct from specified-text projection.
    #[must_use]
    pub fn value_components(&self) -> Option<&CssComponentValues> {
        self.parsed.as_ref().map(|parsed| &parsed.components)
    }

    /// Returns the typed authored descriptor value.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Returns the semantic source position at the descriptor-name start.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

impl<T> std::ops::Deref for CssDescriptorOccurrence<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssLayerName {
    components: Vec<String>,
}

impl CssLayerName {
    #[must_use]
    pub fn try_new(components: impl IntoIterator<Item = impl Into<String>>) -> Option<Self> {
        let components = components.into_iter().map(Into::into).collect::<Vec<_>>();
        if components.is_empty()
            || components
                .iter()
                .any(|component| !is_valid_layer_name_component(component))
        {
            None
        } else {
            Some(Self::new(components))
        }
    }

    #[must_use]
    pub(crate) fn new(components: Vec<String>) -> Self {
        debug_assert!(!components.is_empty());
        debug_assert!(
            components
                .iter()
                .all(|component| is_valid_layer_name_component(component))
        );
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssLayerNameList {
    names: Vec<CssLayerName>,
}

impl CssLayerNameList {
    #[must_use]
    pub fn try_new(names: Vec<CssLayerName>) -> Option<Self> {
        if names.is_empty() {
            None
        } else {
            Some(Self::new(names))
        }
    }

    #[must_use]
    pub(crate) fn new(names: Vec<CssLayerName>) -> Self {
        debug_assert!(!names.is_empty());
        Self { names }
    }

    #[must_use]
    pub fn names(&self) -> &[CssLayerName] {
        &self.names
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssLayerStatementRule {
    names: CssLayerNameList,
    position: CssSourcePosition,
}

impl CssLayerStatementRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @layer parser construction.
    pub(crate) const fn new(names: CssLayerNameList, position: CssSourcePosition) -> Self {
        Self { names, position }
    }

    #[must_use]
    pub const fn names(&self) -> &CssLayerNameList {
        &self.names
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssLayerBlockRule {
    name: Option<CssLayerName>,
    rules: Vec<CssRule>,
    position: Option<CssSourcePosition>,
}

impl CssLayerBlockRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @layer parser construction.
    pub(crate) fn new(
        name: Option<CssLayerName>,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            rules,
            position: Some(position),
        }
    }

    #[must_use]
    pub const fn name(&self) -> Option<&CssLayerName> {
        self.name.as_ref()
    }

    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

fn is_valid_layer_name_component(component: &str) -> bool {
    !is_parser_reserved_layer_name(component) && is_exact_css_identifier(component)
}

fn is_parser_reserved_layer_name(component: &str) -> bool {
    matches!(
        component.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    )
}

fn is_exact_css_identifier(value: &str) -> bool {
    let mut input = cssparser::ParserInput::new(value);
    let mut parser = cssparser::Parser::new(&mut input);
    parser
        .expect_ident_cloned()
        .ok()
        .is_some_and(|parsed| parser.expect_exhausted().is_ok() && parsed.as_ref() == value)
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssMediaRule {
    query: CssMediaQueryList,
    rules: Vec<CssRule>,
    position: Option<CssSourcePosition>,
}

impl CssMediaRule {
    #[must_use]
    pub(crate) fn new(
        query: CssMediaQueryList,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            query,
            rules,
            position: Some(position),
        }
    }

    #[must_use]
    pub const fn query(&self) -> &CssMediaQueryList {
        &self.query
    }

    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// A parser-produced Conditional Rules 3 `@supports` group rule.
///
/// The condition and children preserve authored syntax only. This value does
/// not evaluate support, match selectors, apply cascade, or resolve values.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsRule {
    condition: CssSupportsCondition,
    rules: Vec<CssRule>,
    position: Option<CssSourcePosition>,
}

impl CssSupportsRule {
    #[must_use]
    pub(crate) fn new(
        condition: CssSupportsCondition,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            condition,
            rules,
            position: Some(position),
        }
    }

    #[must_use]
    pub const fn condition(&self) -> &CssSupportsCondition {
        &self.condition
    }

    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// One authored supports condition with original lexical backing.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsCondition {
    kind: Box<CssSupportsConditionKind>,
    lexical: crate::supports::SupportsLexical,
    origin: CssValueOrigin,
    authored_form: SupportsAuthoredForm,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SupportsAuthoredForm {
    Condition,
    BareDeclaration,
    BareName,
}
impl CssSupportsCondition {
    pub(crate) fn new(
        kind: CssSupportsConditionKind,
        lexical: crate::supports::SupportsLexical,
        authored_form: SupportsAuthoredForm,
    ) -> Self {
        let origin = lexical.first_origin().clone();
        Self {
            kind: Box::new(kind),
            lexical,
            origin,
            authored_form,
        }
    }
    #[must_use]
    pub const fn kind(&self) -> &CssSupportsConditionKind {
        &self.kind
    }
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(&self.origin)
    }
    #[must_use]
    pub fn components(&self) -> &[crate::CssComponentValue] {
        self.lexical.items()
    }
    pub(crate) fn into_kind(self) -> CssSupportsConditionKind {
        *self.kind
    }
    pub(crate) const fn authored_form(&self) -> SupportsAuthoredForm {
        self.authored_form
    }
}

/// The complete authored shape of one supports condition.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssSupportsConditionKind {
    Declaration(Box<CssSupportsDeclaration>),
    Named(crate::CssSupportsConditionName),
    Selector(CssSelector),
    GeneralEnclosed(CssGeneralEnclosed),
    Not(Box<CssSupportsCondition>),
    And(CssSupportsConditionList),
    Or(CssSupportsConditionList),
}

/// A checked list used by `and` and `or` supports conditions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsConditionList {
    conditions: Vec<CssSupportsCondition>,
}

impl CssSupportsConditionList {
    #[must_use]
    pub fn try_new(conditions: Vec<CssSupportsCondition>) -> Option<Self> {
        (conditions.len() >= 2).then_some(Self { conditions })
    }

    #[must_use]
    pub(crate) fn new(conditions: Vec<CssSupportsCondition>) -> Self {
        debug_assert!(conditions.len() >= 2);
        Self { conditions }
    }

    #[must_use]
    pub fn conditions(&self) -> &[CssSupportsCondition] {
        &self.conditions
    }
}

/// One syntactically valid declaration test inside a supports condition.
///
/// Unknown properties and values outside the current property grammar remain
/// valid authored tests. `known()` is populated only when the existing schema
/// parser can provide its property-coupled typed declaration view.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsDeclaration {
    authored: Option<String>,
    property: String,
    importance: CssImportance,
    known: Option<CssKnownDeclaration>,
    lexical: crate::supports::SupportsLexical,
    property_index: usize,
    value_range: std::ops::Range<usize>,
}
impl CssSupportsDeclaration {
    pub(crate) fn new(
        authored: Option<String>,
        property: String,
        importance: CssImportance,
        known: Option<CssKnownDeclaration>,
        lexical: crate::supports::SupportsLexical,
        property_index: usize,
        value_range: std::ops::Range<usize>,
    ) -> Self {
        Self {
            authored,
            property,
            importance,
            known,
            lexical,
            property_index,
            value_range,
        }
    }
    #[must_use]
    pub fn authored(&self) -> Option<&str> {
        self.authored.as_deref()
    }
    #[must_use]
    pub fn property(&self) -> &str {
        &self.property
    }
    #[must_use]
    pub const fn importance(&self) -> CssImportance {
        self.importance
    }
    #[must_use]
    pub const fn known(&self) -> Option<&CssKnownDeclaration> {
        self.known.as_ref()
    }
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.property_component().origin()
    }
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
    #[must_use]
    pub fn components(&self) -> &[crate::CssComponentValue] {
        self.lexical.items()
    }
    #[must_use]
    pub fn property_component(&self) -> &crate::CssComponentValue {
        &self.components()[self.property_index]
    }
    #[must_use]
    pub fn value_components(&self) -> &[crate::CssComponentValue] {
        &self.components()[self.value_range.clone()]
    }
    pub(crate) fn lexical(&self) -> &crate::supports::SupportsLexical {
        &self.lexical
    }
}

/// One checked function or parenthesis component, including EOF-implied closure.
///
/// This lexical wrapper does not select a supports or media grammar branch.
/// Equality compares exact token spelling, children and source-text/span origins;
/// it is not semantic equivalence or source snapshot identity.
///
/// ```compile_fail
/// use surgeist_css::{CssComponentValue, CssGeneralEnclosed};
/// let component = CssComponentValue::try_token("x").unwrap();
/// let forged = CssGeneralEnclosed { component: Box::new(component) };
/// ```
///
/// ```compile_fail
/// use surgeist_css::{CssComponentValue, CssGeneralEnclosed, CssParsedOrigin};
/// let parsed = CssParsedOrigin { source: todo!(), span: todo!() };
/// let component = CssComponentValue { data: todo!(), parsed: Some(parsed) };
/// let forged = CssGeneralEnclosed::try_from_component(component);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGeneralEnclosed {
    component: Box<crate::CssComponentValue>,
}

/// Failure to construct one lexically enclosed component.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssGeneralEnclosedError {
    /// The outer component is neither a function nor a parenthesis block.
    WrongOuterComponent {
        /// The rejected component's token or opener origin.
        origin: crate::CssValueOrigin,
    },
    /// The component owner rejected syntax or a resource limit.
    Component(crate::CssComponentValueError),
}

impl CssGeneralEnclosedError {
    /// Returns the responsible real parsed or programmatic origin.
    #[must_use]
    pub const fn origin(&self) -> &crate::CssValueOrigin {
        match self {
            Self::WrongOuterComponent { origin } => origin,
            Self::Component(error) => error.origin(),
        }
    }
}

impl std::fmt::Display for CssGeneralEnclosedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongOuterComponent { .. } => {
                f.write_str("expected a function or parenthesis component")
            }
            Self::Component(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for CssGeneralEnclosedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            Self::WrongOuterComponent { .. } => None,
        }
    }
}

impl CssGeneralEnclosed {
    /// Checks the outer lexical enclosure without changing any supplied origin.
    pub fn try_from_component(
        component: crate::CssComponentValue,
    ) -> Result<Self, CssGeneralEnclosedError> {
        let valid = match component.view() {
            crate::CssComponentValueRef::Function(_) => true,
            crate::CssComponentValueRef::Block(block) => {
                block.kind() == crate::CssBlockKind::Parenthesis
            }
            _ => false,
        };
        if !valid {
            return Err(CssGeneralEnclosedError::WrongOuterComponent {
                origin: component.origin().clone(),
            });
        }
        Ok(Self {
            component: Box::new(component),
        })
    }

    /// Constructs programmatic function delimiters while preserving child origins.
    pub fn try_function(
        name: impl Into<String>,
        values: CssComponentValues,
    ) -> Result<Self, CssGeneralEnclosedError> {
        Self::try_from_component(
            crate::CssComponentValue::try_function(name, values)
                .map_err(CssGeneralEnclosedError::Component)?,
        )
    }

    /// Constructs programmatic parentheses while preserving child origins.
    pub fn try_parenthesized(values: CssComponentValues) -> Result<Self, CssGeneralEnclosedError> {
        Self::try_from_component(
            crate::CssComponentValue::try_block(crate::CssBlockKind::Parenthesis, values)
                .map_err(CssGeneralEnclosedError::Component)?,
        )
    }

    /// Returns the immutable underlying component.
    #[must_use]
    pub const fn component(&self) -> &crate::CssComponentValue {
        &self.component
    }

    /// Returns the opener's real origin.
    #[must_use]
    pub const fn origin(&self) -> &crate::CssValueOrigin {
        self.component.origin()
    }

    /// Returns coordinates only when the opener was parsed from source.
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        match self.origin() {
            crate::CssValueOrigin::Parsed(origin) => Some(origin.span().start()),
            _ => None,
        }
    }

    /// Returns the complete original component slice, including unclosed EOF input.
    /// Programmatic enclosures have no single authored slice, even with parsed children.
    #[must_use]
    pub fn authored(&self) -> Option<&str> {
        let origin = self.component.parsed_origin()?;
        Some(
            &origin.source().as_str()[origin.span().start().byte_offset().value()
                ..origin.span().end().byte_offset().value()],
        )
    }

    /// Serializes tokens and explicit or EOF-implied delimiters with their origins.
    pub fn serialize(&self) -> Result<crate::CssSerializedValue, crate::CssComponentValueError> {
        CssComponentValues::try_new(vec![self.component().clone()])?.serialize()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerRule {
    prelude: CssContainerPrelude,
    rules: Vec<CssRule>,
    position: Option<CssSourcePosition>,
}

impl CssContainerRule {
    #[must_use]
    pub(crate) fn new(
        prelude: CssContainerPrelude,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            prelude,
            rules,
            position: Some(position),
        }
    }

    /// Returns every authored entry and its shared lexical prelude.
    #[must_use]
    pub const fn prelude(&self) -> &CssContainerPrelude {
        &self.prelude
    }

    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopeRule {
    root: Option<CssScopeSelectorList>,
    limit: Option<CssScopeSelectorList>,
    rules: CssScopedRuleList,
    position: Option<CssSourcePosition>,
}

impl CssScopeRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @scope parser construction.
    pub(crate) fn new(
        root: Option<CssScopeSelectorList>,
        limit: Option<CssScopeSelectorList>,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            root,
            limit,
            rules,
            position: Some(position),
        }
    }

    #[must_use]
    pub const fn root(&self) -> Option<&CssScopeSelectorList> {
        self.root.as_ref()
    }

    #[must_use]
    pub const fn limit(&self) -> Option<&CssScopeSelectorList> {
        self.limit.as_ref()
    }

    #[must_use]
    pub const fn rules(&self) -> &CssScopedRuleList {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopeSelectorList {
    selectors: Vec<CssScopeSelector>,
}

impl CssScopeSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssScopeSelector>) -> Option<Self> {
        if selectors.is_empty()
            || selectors
                .iter()
                .any(|member| member.selector().has_pseudo_elements())
        {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssScopeSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        debug_assert!(
            !selectors
                .iter()
                .any(|member| member.selector().has_pseudo_elements())
        );
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssScopeSelector] {
        &self.selectors
    }
}

/// One scope boundary member, retaining a leading combinator without fabricating
/// an explicit anchor. Root relatives require an enclosing style or scope;
/// limit relatives refer to the scope introduced by their rule.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssScopeSelector {
    Selector(CssSelector),
    Relative(CssRelativeSelector),
}

impl CssScopeSelector {
    #[must_use]
    pub const fn selector(&self) -> &CssSelector {
        match self {
            Self::Selector(selector) => selector,
            Self::Relative(relative) => relative.selector(),
        }
    }
}

/// Validation context for checked scope construction, not a complete ancestry
/// record. Final stylesheet or group assembly rechecks actual ancestry. The
/// nearest enclosing style or scope binds implicit relative roots; explicit
/// root `&` and declaration runs use any enclosing style ancestor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssScopeNestingContext {
    /// No enclosing style or scope. Relative roots are invalid.
    None,
    /// A style ancestor supplies explicit root `&` and declaration runs, including
    /// when another scope intervenes. Implicit relatives retain no synthetic anchor.
    Style,
    /// An enclosing scope supplies the root's scope context, without a style ancestor.
    Scope,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CssScopedRuleList {
    rules: Vec<CssScopedRule>,
}

impl CssScopedRuleList {
    #[must_use]
    pub const fn new() -> Self {
        Self { rules: Vec::new() }
    }

    #[must_use]
    #[allow(dead_code)] // Staged for @scope parser construction.
    pub(crate) const fn from_rules(rules: Vec<CssScopedRule>) -> Self {
        Self { rules }
    }

    #[must_use]
    pub fn rules(&self) -> &[CssScopedRule] {
        &self.rules
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssScopedRule {
    /// A nonempty declaration run inheriting the nearest ancestor style selector list.
    NestedDeclarations(CssNestedDeclarationsRule),
    /// A page definition retained in an ordinary group within a scope.
    Page(CssPageRule),
    /// A global counter definition retained in its authored scope context.
    CounterStyle(CssCounterStyleRule),
    /// A global font definition retained in its authored scope context.
    FontFace(CssFontFaceRule),
    /// A global animation definition retained in its authored scope context.
    Keyframes(CssKeyframesRule),
    CustomMedia(crate::CssCustomMediaRule),
    FontFeatureValues(CssFontFeatureValuesRule),
    FontPaletteValues(CssFontPaletteValuesRule),
    ColorProfile(CssColorProfileRule),
    Style(CssScopedStyleRule),
    Media(CssScopedMediaRule),
    Supports(CssScopedSupportsRule),
    SupportsCondition(crate::CssSupportsConditionRule),
    Container(CssScopedContainerRule),
    LayerStatement(CssScopedLayerStatementRule),
    LayerBlock(CssScopedLayerBlockRule),
    Scope(CssScopeRule),
}

/// A scoped authored style rule whose declarations passed the ordinary declaration boundary.
///
/// Its parser-produced position identifies the start of the authored scoped style rule and cannot
/// be forged by callers. The private collection preserves order and importance without applying
/// scope matching, selector matching, cascade, substitution, or contextual resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct CssScopedStyleRule {
    selectors: CssScopedStyleSelectorList,
    declarations: CssDeclarationList,
    rules: Vec<CssRule>,
    position: CssSourcePosition,
}

impl CssScopedStyleRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @scope parser construction.
    pub(crate) fn new(
        selectors: CssScopedStyleSelectorList,
        declarations: CssDeclarationList,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            selectors,
            declarations,
            rules,
            position,
        }
    }

    #[must_use]
    pub const fn selectors(&self) -> &CssScopedStyleSelectorList {
        &self.selectors
    }

    /// Returns leading declarations when the first retained body item is a declaration run.
    /// A rejected rule can end this run without producing a retained child.
    #[must_use]
    pub const fn declarations(&self) -> &CssDeclarationList {
        &self.declarations
    }

    /// Returns nested rules and subsequent declaration runs in authored order.
    ///
    /// These children use ordinary nesting semantics relative to this scoped style rule.
    /// They preserve the parent context without replacing its scoped selectors.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    /// Returns the semantic source position at the authored scoped selector-list start.
    ///
    /// This parser-produced position is diagnostic and ordering provenance only; it does not
    /// perform scope or selector matching and does not participate in cascade.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopedStyleSelectorList {
    selectors: Vec<CssScopedStyleSelector>,
}

impl CssScopedStyleSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssScopedStyleSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssScopedStyleSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssScopedStyleSelector] {
        &self.selectors
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssScopedStyleSelector {
    Selector(CssSelector),
    Relative(CssRelativeSelector),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopedMediaRule {
    query: CssMediaQueryList,
    rules: CssScopedRuleList,
    position: Option<CssSourcePosition>,
}

impl CssScopedMediaRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @scope parser construction.
    pub(crate) fn new(
        query: CssMediaQueryList,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            query,
            rules,
            position: Some(position),
        }
    }

    #[must_use]
    pub const fn query(&self) -> &CssMediaQueryList {
        &self.query
    }

    #[must_use]
    pub const fn rules(&self) -> &CssScopedRuleList {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// A parser-produced supports group inside a scoped rule list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssScopedSupportsRule {
    condition: CssSupportsCondition,
    rules: CssScopedRuleList,
    position: Option<CssSourcePosition>,
}

impl CssScopedSupportsRule {
    #[must_use]
    pub(crate) fn new(
        condition: CssSupportsCondition,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            condition,
            rules,
            position: Some(position),
        }
    }

    #[must_use]
    pub const fn condition(&self) -> &CssSupportsCondition {
        &self.condition
    }

    #[must_use]
    pub const fn rules(&self) -> &CssScopedRuleList {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopedContainerRule {
    prelude: CssContainerPrelude,
    rules: CssScopedRuleList,
    position: Option<CssSourcePosition>,
}

impl CssScopedContainerRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @scope parser construction.
    pub(crate) fn new(
        prelude: CssContainerPrelude,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            prelude,
            rules,
            position: Some(position),
        }
    }

    /// Returns every authored entry and its shared lexical prelude.
    #[must_use]
    pub const fn prelude(&self) -> &CssContainerPrelude {
        &self.prelude
    }

    #[must_use]
    pub const fn rules(&self) -> &CssScopedRuleList {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopedLayerStatementRule {
    names: CssLayerNameList,
    position: CssSourcePosition,
}

impl CssScopedLayerStatementRule {
    #[must_use]
    #[allow(dead_code)] // Staged for scoped @layer parser construction.
    pub(crate) const fn new(names: CssLayerNameList, position: CssSourcePosition) -> Self {
        Self { names, position }
    }

    #[must_use]
    pub const fn names(&self) -> &CssLayerNameList {
        &self.names
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopedLayerBlockRule {
    name: Option<CssLayerName>,
    rules: CssScopedRuleList,
    position: Option<CssSourcePosition>,
}

impl CssScopedLayerBlockRule {
    #[must_use]
    #[allow(dead_code)] // Staged for scoped @layer parser construction.
    pub(crate) fn new(
        name: Option<CssLayerName>,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            rules,
            position: Some(position),
        }
    }

    #[must_use]
    pub const fn name(&self) -> Option<&CssLayerName> {
        self.name.as_ref()
    }

    #[must_use]
    pub const fn rules(&self) -> &CssScopedRuleList {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssMediaQueryList {
    queries: Vec<CssMediaQuery>,
    pub(crate) comma_origins: Vec<CssValueOrigin>,
}

impl CssMediaQueryList {
    /// Constructs a list, including the valid empty media-query list.
    ///
    /// Every supplied checked member is retained in order.
    #[must_use]
    pub fn new(queries: Vec<CssMediaQuery>) -> Self {
        Self {
            queries,
            comma_origins: Vec::new(),
        }
    }

    pub(crate) fn with_comma_origins(
        queries: Vec<CssMediaQuery>,
        comma_origins: Vec<CssValueOrigin>,
    ) -> Self {
        Self {
            queries,
            comma_origins,
        }
    }

    #[must_use]
    pub fn queries(&self) -> &[CssMediaQuery] {
        &self.queries
    }

    pub(crate) fn into_single_query(self) -> Option<CssMediaQuery> {
        if self.queries.len() == 1 {
            self.queries.into_iter().next()
        } else {
            None
        }
    }
}

/// One authored or parser-recovered media-query-list member.
///
/// The `Never` branch is parser-owned recovery syntax for a malformed authored member. It is not
/// publicly constructible and is the only branch for which [`Self::is_guaranteed_false`] is true.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssMediaQuery {
    Condition(CssMediaCondition),
    Typed(CssTypedMediaQuery),
    Never(CssNeverMediaQuery),
}

impl CssMediaQuery {
    /// Returns the first non-trivia position of this authored or recovered query member.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Condition(v) => v.origin(),
            Self::Typed(v) => v.origin(),
            Self::Never(v) => v.origin(),
        }
    }

    /// Returns whether this member is the parser-owned guaranteed-false recovery sentinel.
    #[must_use]
    pub const fn is_guaranteed_false(&self) -> bool {
        matches!(self, Self::Never(_))
    }
}

/// A parser-owned guaranteed-false replacement for one malformed media-query-list member.
///
/// Its position is the member's first non-trivia position, or the member end when it contained no
/// non-trivia token. The complete malformed source unit remains on the paired recovery diagnostic.
/// Callers cannot construct this recovered state.
///
/// ```compile_fail
/// use surgeist_css::{CssMediaQuery, CssNeverMediaQuery, CssSourcePosition};
///
/// fn forge(position: CssSourcePosition) -> CssMediaQuery {
///     CssMediaQuery::Never(CssNeverMediaQuery { position })
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNeverMediaQuery {
    origin: CssValueOrigin,
}
impl CssNeverMediaQuery {
    pub(crate) fn new(origin: crate::CssParsedOrigin) -> Self {
        Self {
            origin: CssValueOrigin::Parsed(origin),
        }
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    pub const fn position(&self) -> CssSourcePosition {
        match &self.origin {
            CssValueOrigin::Parsed(v) => v.span().start(),
            _ => panic!("parser-only recovery origin"),
        }
    }
}

/// A checked authored typed media query with parsed or programmatic provenance.
///
/// Callers can inspect authored semantics but cannot construct or forge parser provenance.
///
/// ```compile_fail
/// use surgeist_css::{CssMediaQueryModifier, CssMediaType, CssSourcePosition, CssTypedMediaQuery};
///
/// fn forge(position: CssSourcePosition) -> CssTypedMediaQuery {
///     CssTypedMediaQuery {
///         modifier: Some(CssMediaQueryModifier::Only),
///         media_type: CssMediaType::Screen,
///         condition: None,
///         position,
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssTypedMediaQuery {
    modifier: Option<CssMediaQueryModifier>,
    media_type: CssMediaType,
    unknown_media_type: Option<CssUnknownMediaType>,
    condition: Option<Box<CssMediaCondition>>,
    origin: CssValueOrigin,
    pub(crate) syntax: Box<MediaTypedSyntax>,
}

impl CssTypedMediaQuery {
    #[must_use]
    pub(crate) fn new(
        modifier: Option<CssMediaQueryModifier>,
        media_type: CssMediaType,
        condition: Option<CssMediaCondition>,
        origin: CssValueOrigin,
        syntax: MediaTypedSyntax,
    ) -> Self {
        debug_assert_ne!(media_type, CssMediaType::Unknown);
        Self {
            modifier,
            media_type,
            unknown_media_type: None,
            condition: condition.map(Box::new),
            origin,
            syntax: Box::new(syntax),
        }
    }

    #[must_use]
    pub(crate) fn new_unknown(
        modifier: Option<CssMediaQueryModifier>,
        media_type: CssUnknownMediaType,
        condition: Option<CssMediaCondition>,
        origin: CssValueOrigin,
        syntax: MediaTypedSyntax,
    ) -> Self {
        Self {
            modifier,
            media_type: CssMediaType::Unknown,
            unknown_media_type: Some(media_type),
            condition: condition.map(Box::new),
            origin,
            syntax: Box::new(syntax),
        }
    }

    #[must_use]
    pub const fn modifier(&self) -> Option<CssMediaQueryModifier> {
        self.modifier
    }

    #[must_use]
    pub const fn media_type(&self) -> CssMediaType {
        self.media_type
    }

    /// Returns the exact authored spelling and token position for an unknown media type.
    ///
    /// This is present exactly when [`Self::media_type`] returns [`CssMediaType::Unknown`].
    #[must_use]
    pub const fn unknown_media_type(&self) -> Option<&CssUnknownMediaType> {
        self.unknown_media_type.as_ref()
    }

    #[must_use]
    pub const fn condition(&self) -> Option<&CssMediaCondition> {
        match &self.condition {
            Some(condition) => Some(condition),
            None => None,
        }
    }

    /// Returns the first non-trivia position of the authored typed media query.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(&self.origin)
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaQueryModifier {
    Not,
    Only,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaType {
    All,
    Screen,
    Print,
    Aural,
    Braille,
    Embossed,
    Handheld,
    Projection,
    Speech,
    Tty,
    Tv,
    /// A syntactically valid unknown MQ3 media type, which is defined to be false.
    Unknown,
}

impl CssMediaType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Screen => "screen",
            Self::Print => "print",
            Self::Aural => "aural",
            Self::Braille => "braille",
            Self::Embossed => "embossed",
            Self::Handheld => "handheld",
            Self::Projection => "projection",
            Self::Speech => "speech",
            Self::Tty => "tty",
            Self::Tv => "tv",
            Self::Unknown => "unknown",
        }
    }
    /// Whether this type is defined not to match in the selected MQ5 profile.
    pub const fn is_defined_nonmatching(self) -> bool {
        !matches!(self, Self::All | Self::Screen | Self::Print)
    }
}

/// A checked unknown media type with authored spelling and parsed or programmatic provenance.
///
/// Unknown types are valid authored syntax with defined-false semantics. Private fields prevent
/// callers from forging parser provenance or pairing this model with a known media type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssUnknownMediaType {
    spelling: String,
    origin: CssValueOrigin,
}

impl CssUnknownMediaType {
    #[must_use]
    pub(crate) fn new(spelling: impl Into<String>, origin: CssValueOrigin) -> Self {
        let spelling = spelling.into();
        debug_assert!(!spelling.is_empty());
        Self { spelling, origin }
    }

    /// Returns the exact authored media-type token spelling, including escapes and casing.
    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.spelling
    }

    /// Returns the defined-false classification for this unknown media type.
    #[must_use]
    pub const fn reason(&self) -> CssDefinedFalseMediaReason {
        CssDefinedFalseMediaReason::UnknownType
    }

    /// Returns the position of the authored unknown media-type token.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(&self.origin)
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
}

/// A checked authored media condition with parsed or programmatic provenance.
///
/// [`Self::kind`] exposes its semantic shape while private fields prevent callers from attaching a
/// forged source position.
///
/// ```compile_fail
/// use surgeist_css::{CssMediaCondition, CssMediaConditionKind, CssSourcePosition};
///
/// fn forge(kind: CssMediaConditionKind, position: CssSourcePosition) -> CssMediaCondition {
///     CssMediaCondition { kind, position }
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssMediaCondition {
    kind: CssMediaConditionKind,
    origin: CssValueOrigin,
    pub(crate) syntax: MediaConditionSyntax,
}
impl CssMediaCondition {
    pub(crate) fn new(
        kind: CssMediaConditionKind,
        origin: CssValueOrigin,
        syntax: MediaConditionSyntax,
    ) -> Self {
        Self {
            kind,
            origin,
            syntax,
        }
    }
    pub const fn kind(&self) -> &CssMediaConditionKind {
        &self.kind
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(&self.origin)
    }
}

/// The non-exhaustive authored semantic shape of a positioned media condition.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssMediaConditionKind {
    CustomMediaReference(crate::CssCustomMediaReference),
    /// An explicitly parenthesized condition, retaining the outer and inner positions.
    Parenthesized(Box<CssMediaCondition>),
    Feature(CssMediaFeatureQuery),
    UnknownFeature(CssUnknownMediaFeature),
    GeneralEnclosed(CssGeneralEnclosed),
    Not(Box<CssMediaCondition>),
    And(CssMediaConditionList),
    Or(CssMediaConditionList),
}

/// Why an unknown media type is defined to be nonmatching.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssDefinedFalseMediaReason {
    UnknownType,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssContainerName {
    name: String,
}

impl CssContainerName {
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if is_valid_container_name(&name) {
            Some(Self::new(name))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(is_valid_container_name(&name));
        Self { name }
    }

    /// Constructs a decoded container name, including names requiring CSS escapes.
    /// Empty, NUL-containing and reserved names are rejected; spelling is not reparsed.
    #[must_use]
    pub fn try_from_decoded(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        Self::from_decoded(name)
    }

    /// Admits the decoded value of one identifier token without tokenizing again.
    pub(crate) fn from_decoded(name: String) -> Option<Self> {
        (!name.is_empty() && !name.contains('\0') && !is_parser_reserved_container_name(&name))
            .then_some(Self { name })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

fn is_valid_container_name(name: &str) -> bool {
    is_exact_css_identifier(name) && !is_parser_reserved_container_name(name)
}

fn is_parser_reserved_container_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "inherit"
            | "initial"
            | "unset"
            | "revert"
            | "revert-layer"
            | "none"
            | "and"
            | "or"
            | "not"
            | "default"
    )
}

/// A nonempty ordered list of independent authored container-query entries.
/// Commas select alternative containers; they are not Boolean query operators.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerPrelude {
    entries: Vec<CssContainerQueryEntry>,
    lexical: crate::supports::SupportsLexical,
    origin: CssValueOrigin,
}

impl CssContainerPrelude {
    pub(crate) fn new(
        entries: Vec<CssContainerQueryEntry>,
        lexical: crate::supports::SupportsLexical,
    ) -> Self {
        debug_assert!(!entries.is_empty());
        let origin = lexical.first_origin().clone();
        Self {
            entries,
            lexical,
            origin,
        }
    }
    #[must_use]
    pub fn entries(&self) -> &[CssContainerQueryEntry] {
        &self.entries
    }
    /// Includes names, commas, comments and whitespace in their authored order.
    #[must_use]
    pub fn components(&self) -> &[crate::CssComponentValue] {
        self.lexical.items()
    }
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(&self.origin)
    }
}

/// One checked entry, with a name, a query, or both, retaining its lexical region.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerQueryEntry {
    kind: CssContainerQueryEntryKind,
    lexical: crate::supports::SupportsLexical,
    origin: CssValueOrigin,
}
#[derive(Clone, Debug, PartialEq)]
enum CssContainerQueryEntryKind {
    NameOnly(CssContainerName),
    Query {
        name: Option<CssContainerName>,
        query: CssContainerCondition,
    },
}
impl CssContainerQueryEntry {
    pub(crate) fn name_only(
        name: CssContainerName,
        lexical: crate::supports::SupportsLexical,
    ) -> Self {
        let origin = lexical.first_origin().clone();
        Self {
            kind: CssContainerQueryEntryKind::NameOnly(name),
            lexical,
            origin,
        }
    }
    pub(crate) fn with_query(
        name: Option<CssContainerName>,
        query: CssContainerCondition,
        lexical: crate::supports::SupportsLexical,
    ) -> Self {
        let origin = lexical.first_origin().clone();
        Self {
            kind: CssContainerQueryEntryKind::Query { name, query },
            lexical,
            origin,
        }
    }
    #[must_use]
    pub const fn name(&self) -> Option<&CssContainerName> {
        match &self.kind {
            CssContainerQueryEntryKind::NameOnly(name) => Some(name),
            CssContainerQueryEntryKind::Query { name, .. } => name.as_ref(),
        }
    }
    #[must_use]
    pub const fn query(&self) -> Option<&CssContainerCondition> {
        match &self.kind {
            CssContainerQueryEntryKind::NameOnly(_) => None,
            CssContainerQueryEntryKind::Query { query, .. } => Some(query),
        }
    }
    /// Excludes the separating comma and other entries; includes entry trivia.
    #[must_use]
    pub fn components(&self) -> &[crate::CssComponentValue] {
        self.lexical.items()
    }
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(&self.origin)
    }
}

/// One checked authored container condition, preserving grouping and lexical origins.
/// Private fields prevent constructing ungrouped mixtures of boolean operators.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerCondition {
    kind: Box<CssContainerConditionKind>,
    lexical: crate::supports::SupportsLexical,
    origin: CssValueOrigin,
}

impl CssContainerCondition {
    pub(crate) fn new(
        kind: CssContainerConditionKind,
        lexical: crate::supports::SupportsLexical,
    ) -> Self {
        let origin = lexical.first_origin().clone();
        Self {
            kind: Box::new(kind),
            lexical,
            origin,
        }
    }
    #[must_use]
    pub const fn kind(&self) -> &CssContainerConditionKind {
        &self.kind
    }
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(&self.origin)
    }
    /// Returns the complete selected lexical region, including authored operators
    /// and grouping, without copying enclosing or sibling subtrees.
    #[must_use]
    pub fn components(&self) -> &[crate::CssComponentValue] {
        self.lexical.items()
    }
}

/// The inspectable shape of a condition admitted by the container grammar.
/// Constructing a kind does not bypass checked condition construction.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerConditionKind {
    GeneralEnclosed(CssContainerGeneralEnclosed),
    Feature(CssContainerFeatureQuery),
    Style(CssContainerStyleQuery),
    ScrollState(CssContainerScrollQuery),
    /// An explicit grouping pair around a condition, including redundant pairs.
    Parenthesized(Box<CssContainerCondition>),
    Not(Box<CssContainerCondition>),
    And(CssContainerConditionList),
    Or(CssContainerConditionList),
}

/// A container operand classified as general-enclosed by the container grammar.
/// Its private lexical region prevents bypassing recognized feature/style admission.
///
/// ```compile_fail
/// use surgeist_css::{CssContainerCondition, CssContainerConditionKind};
/// fn replace_kind(mut condition: CssContainerCondition, kind: CssContainerConditionKind) {
///     condition.kind = Box::new(kind);
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerGeneralEnclosed {
    lexical: crate::supports::SupportsLexical,
}
impl Eq for CssContainerGeneralEnclosed {}

impl CssContainerGeneralEnclosed {
    pub(crate) fn new(lexical: crate::supports::SupportsLexical) -> Self {
        Self { lexical }
    }
    /// Borrows the single admitted enclosure from the shared lexical root.
    #[must_use]
    pub fn component(&self) -> &crate::CssComponentValue {
        self.lexical
            .items()
            .iter()
            .find(|value| !crate::supports::trivia(value))
            .expect("one admitted enclosure")
    }
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.component().origin()
    }
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
    #[must_use]
    pub fn authored(&self) -> Option<&str> {
        let origin = self.component().parsed_origin()?;
        Some(
            &origin.source().as_str()[origin.span().start().byte_offset().value()
                ..origin.span().end().byte_offset().value()],
        )
    }
    /// Serializes the original enclosure, including EOF-implied delimiters.
    pub fn serialize(&self) -> Result<crate::CssSerializedValue, crate::CssComponentValueError> {
        let mut out = crate::component_values::CssCanonicalBuilder::new(usize::MAX);
        out.push_components(std::slice::from_ref(self.component()))?;
        out.finish()
    }
}

/// A grammar or resource failure during checked construction.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssContainerConstructionError {
    InvalidConditionGrammar { origin: CssValueOrigin },
    InvalidPreludeGrammar { origin: CssValueOrigin },
    Component(crate::CssComponentValueError),
    WorkerUnavailable { origin: CssValueOrigin },
}
impl CssContainerConstructionError {
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::InvalidConditionGrammar { origin }
            | Self::InvalidPreludeGrammar { origin }
            | Self::WorkerUnavailable { origin } => origin,
            Self::Component(error) => error.origin(),
        }
    }
}
impl std::fmt::Display for CssContainerConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid authored container construction: {self:?}")
    }
}
impl std::error::Error for CssContainerConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            _ => None,
        }
    }
}
impl From<crate::CssComponentValueError> for CssContainerConstructionError {
    fn from(error: crate::CssComponentValueError) -> Self {
        Self::Component(error)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerConditionList {
    conditions: Vec<CssContainerCondition>,
}

impl CssContainerConditionList {
    #[must_use]
    pub fn try_new(conditions: Vec<CssContainerCondition>) -> Option<Self> {
        if conditions.len() < 2 {
            None
        } else {
            Some(Self::new(conditions))
        }
    }

    #[must_use]
    pub(crate) fn new(conditions: Vec<CssContainerCondition>) -> Self {
        debug_assert!(conditions.len() >= 2);
        Self { conditions }
    }

    #[must_use]
    pub fn conditions(&self) -> &[CssContainerCondition] {
        &self.conditions
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerFeatureQuery {
    Boolean(CssContainerSizeFeatureKind),
    Width(CssMediaRange<CssContainerLength>),
    Height(CssMediaRange<CssContainerLength>),
    InlineSize(CssMediaRange<CssContainerLength>),
    BlockSize(CssMediaRange<CssContainerLength>),
    AspectRatio(CssMediaRange<CssContainerRatio>),
    Orientation(CssContainerOrientation),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssMediaConditionList {
    conditions: Vec<CssMediaCondition>,
}

impl CssMediaConditionList {
    #[must_use]
    pub fn try_new(conditions: Vec<CssMediaCondition>) -> Option<Self> {
        if conditions.len() < 2 {
            None
        } else {
            Some(Self::new(conditions))
        }
    }

    #[must_use]
    pub(crate) fn new(conditions: Vec<CssMediaCondition>) -> Self {
        debug_assert!(conditions.len() >= 2);
        Self { conditions }
    }

    #[must_use]
    pub fn conditions(&self) -> &[CssMediaCondition] {
        &self.conditions
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssQueryComparison {
    LessThan,
    LessThanOrEqual,
    Equal,
    GreaterThanOrEqual,
    GreaterThan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssRangeFeature<T> {
    comparison: Option<CssQueryComparison>,
    value: T,
}

impl<T> CssRangeFeature<T> {
    #[must_use]
    pub const fn comparison(&self) -> Option<CssQueryComparison> {
        self.comparison
    }

    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOrientation {
    Portrait,
    Landscape,
}

/// The authored scanning mode of the Media Queries Level 3 `scan` feature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScanMode {
    Progressive,
    Interlace,
}

/// The authored binary state of the Media Queries Level 3 `grid` feature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridMode {
    Bitmap,
    Grid,
}

impl CssGridMode {
    /// Returns the authored integer meaning of this grid mode.
    #[must_use]
    pub const fn value(self) -> u8 {
        match self {
            Self::Bitmap => 0,
            Self::Grid => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorSchemePreference {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssReducedMotionPreference {
    Reduce,
    NoPreference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssReducedTransparencyPreference {
    Reduce,
    NoPreference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContrastPreference {
    NoPreference,
    More,
    Less,
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssForcedColorsMode {
    None,
    Active,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHoverCapability {
    None,
    Hover,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPointerCapability {
    None,
    Coarse,
    Fine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssDisplayMode {
    Fullscreen,
    Standalone,
    MinimalUi,
    Browser,
    PictureInPicture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssResolutionUnit {
    Dpi,
    Dpcm,
    Dppx,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssRatio {
    numerator: CssFiniteNumber,
    denominator: CssFiniteNumber,
}

impl CssRatio {
    #[must_use]
    pub fn try_new(numerator: f32, denominator: f32) -> Option<Self> {
        if numerator >= 0.0 && denominator > 0.0 {
            Some(Self {
                numerator: CssFiniteNumber::try_new(numerator)?,
                denominator: CssFiniteNumber::try_new(denominator)?,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn numerator(self) -> CssFiniteNumber {
        self.numerator
    }

    #[must_use]
    pub const fn denominator(self) -> CssFiniteNumber {
        self.denominator
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssQueryLength {
    value: CssFiniteNumber,
    unit: Option<CssLengthUnit>,
}

impl CssQueryLength {
    #[must_use]
    pub fn try_new(value: f32, unit: CssLengthUnit) -> Option<Self> {
        if value >= 0.0 {
            CssFiniteNumber::try_new(value).map(|value| Self {
                value,
                unit: Some(unit),
            })
        } else {
            None
        }
    }

    /// Constructs the exact unitless-zero spelling accepted by the CSS length grammar.
    #[must_use]
    pub fn unitless_zero() -> Self {
        Self {
            value: CssFiniteNumber::try_new(0.0).expect("zero is finite"),
            unit: None,
        }
    }

    #[must_use]
    pub const fn value(self) -> CssFiniteNumber {
        self.value
    }

    /// Returns the exact authored unit, or `None` for a valid unitless zero.
    #[must_use]
    pub const fn unit(self) -> Option<CssLengthUnit> {
        self.unit
    }
}

// Shared parser-produced style-body payload. Syntax owns the declaration-run
// invariant; both selector-bearing rules and raw blocks retain this same shape.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StyleContents {
    pub(crate) declarations: CssDeclarationList,
    pub(crate) rules: Vec<CssRule>,
}

impl StyleContents {
    pub(crate) fn into_nested_rules(self) -> Vec<CssRule> {
        let mut rules = Vec::new();
        if !self.declarations.is_empty() {
            rules.push(CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(
                self.declarations,
            )));
        }
        rules.extend(self.rules);
        rules
    }
}

/// One parser-produced style block with its genuine authored brace region.
///
/// Leading declarations precede the first retained child rule. Later declaration
/// runs appear as [`CssRule::NestedDeclarations`] among [`Self::rules`]. Child
/// selectors remain symbolic; no enclosing selector or parent list is fabricated.
/// Equality compares content and origin source text/span, not snapshot identity.
#[derive(Clone, Debug, PartialEq)]
pub struct CssStyleBlock {
    contents: StyleContents,
    origin: CssParsedOrigin,
}

impl CssStyleBlock {
    pub(crate) fn new(contents: StyleContents, origin: CssParsedOrigin) -> Self {
        Self { contents, origin }
    }

    /// Returns leading declarations when the first retained body item is a declaration run.
    /// A rejected rule can end this run without producing a retained child.
    #[must_use]
    pub const fn declarations(&self) -> &CssDeclarationList {
        &self.contents.declarations
    }

    /// Returns child rules and subsequent declaration runs in authored order.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.contents.rules
    }

    /// Returns the opening brace through the explicit closing brace or actual EOF.
    /// Surrounding trivia is excluded; the snapshot retains the entire original input.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}

/// One authored style rule with a complete selector list and ordered children.
///
/// Leading declarations belong to this rule. Declaration runs after its first retained child
/// appear as [`CssRule::NestedDeclarations`] in [`Self::rules`]. Nested selectors retain symbolic
/// parent references; this tree does not expand selectors or select cascade winners.
#[derive(Clone, Debug, PartialEq)]
pub struct CssStyleRule {
    selectors: CssStyleSelectorList,
    contents: StyleContents,
    position: CssSourcePosition,
}

impl CssStyleRule {
    pub(crate) fn new(
        selectors: CssStyleSelectorList,
        declarations: CssDeclarationList,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            selectors,
            contents: StyleContents {
                declarations,
                rules,
            },
            position,
        }
    }

    /// Returns the complete authored selector list, without parent-selector expansion.
    #[must_use]
    pub const fn selectors(&self) -> &CssStyleSelectorList {
        &self.selectors
    }

    /// Returns leading declarations when the first retained body item is a declaration run.
    /// A rejected rule can end this run without producing a retained child.
    #[must_use]
    pub const fn declarations(&self) -> &CssDeclarationList {
        &self.contents.declarations
    }

    /// Returns nested rules and subsequent declaration runs in authored order.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.contents.rules
    }

    /// Returns the source position of the authored selector-list start.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

/// A nonempty parser-validated selector list for an authored style rule.
///
/// Ordinary stylesheet contexts accept only non-relative selectors. Nested style contexts also
/// accept relative selectors; their relationship to the parent stays symbolic until matching.
#[derive(Clone, Debug, PartialEq)]
pub struct CssStyleSelectorList {
    selectors: Vec<CssStyleSelector>,
}

impl CssStyleSelectorList {
    pub(crate) fn new(selectors: Vec<CssStyleSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    pub(crate) fn absolute(selectors: Vec<CssSelector>) -> Self {
        Self::new(
            selectors
                .into_iter()
                .map(CssStyleSelector::Selector)
                .collect(),
        )
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssStyleSelector] {
        &self.selectors
    }
}

/// An authored selector, or an explicitly relative selector in a nested rule.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssStyleSelector {
    Selector(CssSelector),
    Relative(CssRelativeSelector),
}

impl CssStyleSelector {
    /// Returns the selector itself; [`Self::Relative`] also retains its leading combinator.
    #[must_use]
    pub const fn selector(&self) -> &CssSelector {
        match self {
            Self::Selector(selector) => selector,
            Self::Relative(relative) => relative.selector(),
        }
    }
}

/// A nonempty declaration run in a style rule's child list or a nested conditional group.
///
/// It inherits the enclosing style rule's selector context, including pseudo-elements, rather
/// than synthesizing an `&` selector. Declaration positions preserve its original provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct CssNestedDeclarationsRule {
    declarations: CssDeclarationList,
}

impl CssNestedDeclarationsRule {
    pub(crate) fn new(declarations: CssDeclarationList) -> Self {
        debug_assert!(!declarations.is_empty());
        Self { declarations }
    }

    #[must_use]
    pub const fn declarations(&self) -> &CssDeclarationList {
        &self.declarations
    }

    /// Returns the position of the first declaration in this nonempty run.
    #[must_use]
    pub fn position(&self) -> CssSourcePosition {
        self.declarations[0]
            .position()
            .expect("a parsed nested declaration run contains parsed declarations")
    }
}

/// An ordered parser-produced collection of ordinary authored declarations.
///
/// Private construction ensures every element has passed the ordinary declaration boundary and
/// carries semantic source provenance. The collection is read-only and performs no cascade,
/// substitution, selector matching, or contextual resolution.
///
/// ```compile_fail
/// use surgeist_css::CssDeclarationList;
/// let _ = CssDeclarationList { declarations: Vec::new() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssDeclarationList {
    declarations: Vec<CssDeclaration>,
}

impl CssDeclarationList {
    pub(crate) const fn new(declarations: Vec<CssDeclaration>) -> Self {
        Self { declarations }
    }

    /// Returns the declarations in authored order.
    #[must_use]
    pub fn as_slice(&self) -> &[CssDeclaration] {
        &self.declarations
    }

    /// Iterates over declarations in authored order.
    pub fn iter(&self) -> std::slice::Iter<'_, CssDeclaration> {
        self.declarations.iter()
    }

    /// Returns the number of declarations.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.declarations.len()
    }

    /// Returns whether no declarations were retained.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }
}

impl std::ops::Deref for CssDeclarationList {
    type Target = [CssDeclaration];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// An ordered parser-produced collection of keyframe declarations.
///
/// Its distinct element type makes importance unavailable in keyframe syntax while preserving
/// property coupling and semantic source positions. Construction is parser-owned; this collection
/// does not run animation interpolation, cascade, or substitution.
///
/// ```compile_fail
/// use surgeist_css::CssKeyframeDeclarationList;
/// let _ = CssKeyframeDeclarationList { declarations: Vec::new() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframeDeclarationList {
    declarations: Vec<CssKeyframeDeclaration>,
}

impl CssKeyframeDeclarationList {
    pub(crate) const fn new(declarations: Vec<CssKeyframeDeclaration>) -> Self {
        Self { declarations }
    }

    /// Returns keyframe declarations in authored order.
    #[must_use]
    pub fn as_slice(&self) -> &[CssKeyframeDeclaration] {
        &self.declarations
    }

    /// Iterates over keyframe declarations in authored order.
    pub fn iter(&self) -> std::slice::Iter<'_, CssKeyframeDeclaration> {
        self.declarations.iter()
    }

    /// Returns the number of keyframe declarations.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.declarations.len()
    }

    /// Returns whether no keyframe declarations were retained.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }
}

impl std::ops::Deref for CssKeyframeDeclarationList {
    type Target = [CssKeyframeDeclaration];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// A parser-produced declaration in the authored keyframe syntax phase.
///
/// The private fields retain property/value coupling, complete value components and the
/// property-name position. Keyframe grammar omits animation properties except timing-function
/// and rejects declaration importance, so this type intentionally has no importance field or
/// accessor. It does not interpolate, apply, cascade, or resolve the authored value.
///
/// ```compile_fail
/// let report = surgeist_css::parse_sheet("@keyframes x { from { opacity: 0 } }");
/// assert!(report.is_clean());
/// let surgeist_css::CssRule::Keyframes(rule) = &report.syntax().rules()[0] else {
///     unreachable!()
/// };
/// let _ = rule.blocks()[0].declarations().as_slice()[0].importance();
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframeDeclaration {
    body: CssDeclarationBody,
    value_components: CssComponentValues,
    position: CssSourcePosition,
}

impl CssKeyframeDeclaration {
    pub(crate) const fn new(
        body: CssDeclarationBody,
        value_components: CssComponentValues,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            body,
            value_components,
            position,
        }
    }

    pub(crate) const fn value_components(&self) -> &CssComponentValues {
        &self.value_components
    }

    /// Returns the property-coupled authored body.
    #[must_use]
    pub const fn body(&self) -> &CssDeclarationBody {
        &self.body
    }

    /// Returns the known-property declaration, or `None` for a custom declaration.
    #[must_use]
    pub const fn known(&self) -> Option<&CssKnownDeclaration> {
        match &self.body {
            CssDeclarationBody::Known(known) => Some(known),
            CssDeclarationBody::Custom(_) => None,
        }
    }

    /// Returns the custom declaration, or `None` for a known declaration.
    #[must_use]
    pub const fn custom(&self) -> Option<&CssCustomDeclaration> {
        match &self.body {
            CssDeclarationBody::Known(_) => None,
            CssDeclarationBody::Custom(custom) => Some(custom),
        }
    }

    /// Returns a borrowed semantic property-name view derived from the active body.
    #[must_use]
    pub const fn property_name(&self) -> CssPropertyNameRef<'_> {
        match &self.body {
            CssDeclarationBody::Known(known) => CssPropertyNameRef::Known(known.property()),
            CssDeclarationBody::Custom(custom) => CssPropertyNameRef::Custom(custom.name()),
        }
    }

    /// Returns the semantic source position at the property-name start.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }

    #[cfg(test)]
    pub(crate) fn property(&self) -> crate::test_support::CssProperty {
        crate::test_support::declaration_body_property(&self.body)
    }
}

/// The complete importance state of an ordinary authored declaration.
///
/// Importance is recognized at a parsed declaration boundary or supplied separately to
/// [`crate::parse_property_value`]; it is not part of the property value. Its two states form
/// a closed, exhaustively matchable set.
/// Downstream cascade policy may consume it, but this crate does not apply cascade.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CssImportance {
    /// Normal importance; parsed declarations have no terminal importance annotation.
    #[default]
    Normal,
    /// Important precedence; parsed declarations have one valid terminal `!important` annotation.
    Important,
}

/// A checked declaration occurrence in the authored CSS syntax phase.
///
/// Its private fields couple a known property to its schema-selected value type, or a custom name
/// to its authored custom value. Parsed declarations retain original name and value origins;
/// [`crate::parse_property_value`] constructs declarations from checked components without
/// manufacturing a property-name position. Callers cannot create a property/value mismatch.
///
/// ```compile_fail
/// use surgeist_css::{
///     CssKnownPropertyValueRef, CssWidthPropertyValue, parse_style_attribute,
/// };
///
/// let width_report = parse_style_attribute("width: 1px");
/// let opacity_report = parse_style_attribute("opacity: .5");
/// let Some(CssKnownPropertyValueRef::Width(width)) =
///     width_report.syntax()[0].known().unwrap().property_value()
/// else { panic!("expected width") };
/// let Some(CssKnownPropertyValueRef::Opacity(opacity)) =
///     opacity_report.syntax()[0].known().unwrap().property_value()
/// else { panic!("expected opacity") };
/// fn require_width(_: &CssWidthPropertyValue) {}
/// require_width(width);
/// require_width(opacity);
/// ```
///
/// ```compile_fail
/// use surgeist_css::CssDeclaration;
/// let _ = CssDeclaration { body: todo!(), importance: todo!(), position: todo!() };
/// ```
///
/// This authored node records importance but does not apply cascade, substitution, or contextual
/// resolution.
/// Clones refer to the same immutable occurrence. Structural equality compares the body,
/// importance, and optional name position; it does not compare token provenance or occurrence
/// identity. Use [`Self::same_occurrence`] when that identity matters.
#[derive(Clone, Debug)]
pub struct CssDeclaration {
    occurrence: Arc<DeclarationOccurrence>,
}

#[derive(Debug)]
struct DeclarationOccurrence {
    body: CssDeclarationBody,
    importance: CssImportance,
    components: CssComponentValues,
    provenance: DeclarationProvenance,
}

#[derive(Debug)]
enum DeclarationProvenance {
    Parsed {
        name: CssParsedOrigin,
        value: CssParsedOrigin,
    },
    ParsedValue {
        value: CssParsedOrigin,
    },
    Constructed,
}

impl PartialEq for CssDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.body() == other.body()
            && self.importance() == other.importance()
            && self.position() == other.position()
    }
}

impl CssDeclaration {
    #[must_use]
    pub(crate) fn new_parsed(
        body: CssDeclarationBody,
        importance: CssImportance,
        components: CssComponentValues,
        name: CssParsedOrigin,
        value: CssParsedOrigin,
    ) -> Self {
        Self {
            occurrence: Arc::new(DeclarationOccurrence {
                body,
                importance,
                components,
                provenance: DeclarationProvenance::Parsed { name, value },
            }),
        }
    }

    pub(crate) fn new_parsed_value(
        body: CssDeclarationBody,
        importance: CssImportance,
        components: CssComponentValues,
        value: CssParsedOrigin,
    ) -> Self {
        Self {
            occurrence: Arc::new(DeclarationOccurrence {
                body,
                importance,
                components,
                provenance: DeclarationProvenance::ParsedValue { value },
            }),
        }
    }

    pub(crate) fn new_constructed(
        body: CssDeclarationBody,
        importance: CssImportance,
        components: CssComponentValues,
    ) -> Self {
        Self {
            occurrence: Arc::new(DeclarationOccurrence {
                body,
                importance,
                components,
                provenance: DeclarationProvenance::Constructed,
            }),
        }
    }

    /// Returns the property-coupled authored body.
    #[must_use]
    pub fn body(&self) -> &CssDeclarationBody {
        &self.occurrence.body
    }

    /// Returns the known-property declaration, or `None` for a custom declaration.
    #[must_use]
    pub fn known(&self) -> Option<&CssKnownDeclaration> {
        match self.body() {
            CssDeclarationBody::Known(known) => Some(known),
            CssDeclarationBody::Custom(_) => None,
        }
    }

    /// Returns the custom declaration, or `None` for a known declaration.
    #[must_use]
    pub fn custom(&self) -> Option<&CssCustomDeclaration> {
        match self.body() {
            CssDeclarationBody::Known(_) => None,
            CssDeclarationBody::Custom(custom) => Some(custom),
        }
    }

    /// Returns a borrowed semantic property-name view derived from the active body.
    #[must_use]
    pub fn property_name(&self) -> CssPropertyNameRef<'_> {
        match self.body() {
            CssDeclarationBody::Known(known) => CssPropertyNameRef::Known(known.property()),
            CssDeclarationBody::Custom(custom) => CssPropertyNameRef::Custom(custom.name()),
        }
    }

    /// Returns importance recognized from source or supplied by the caller.
    #[must_use]
    pub fn importance(&self) -> CssImportance {
        self.occurrence.importance
    }

    /// Returns the original property-name start, or `None` when the name was supplied.
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        self.parsed_name().map(|origin| origin.span().start())
    }

    /// Returns the consumed property-name token's original source span.
    /// Raw value parsing and checked construction have no parsed property name.
    #[must_use]
    pub fn parsed_name(&self) -> Option<&CssParsedOrigin> {
        match &self.occurrence.provenance {
            DeclarationProvenance::Parsed { name, .. } => Some(name),
            DeclarationProvenance::ParsedValue { .. } | DeclarationProvenance::Constructed => None,
        }
    }

    /// Returns the parsed value region, excluding its annotation and declaration delimiter.
    ///
    /// Raw value parsing retains its original value source without a fabricated name.
    /// Checked construction has no single declaration-source region, even when individual
    /// supplied tokens have parsed origins. Empty parsed custom values retain a zero-width span.
    #[must_use]
    pub fn parsed_value(&self) -> Option<&CssParsedOrigin> {
        match &self.occurrence.provenance {
            DeclarationProvenance::Parsed { value, .. }
            | DeclarationProvenance::ParsedValue { value } => Some(value),
            DeclarationProvenance::Constructed => None,
        }
    }

    /// Returns the owned value components with their original or programmatic provenance.
    #[must_use]
    pub fn value_components(&self) -> &CssComponentValues {
        &self.occurrence.components
    }

    /// Reports whether both handles refer to the same immutable declaration occurrence.
    #[must_use]
    pub fn same_occurrence(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.occurrence, &other.occurrence)
    }

    #[cfg(test)]
    pub(crate) fn property(&self) -> crate::test_support::CssProperty {
        crate::test_support::declaration_property(self)
    }
}

/// The authored body of a declaration, split between known and custom property invariants.
///
/// Known values are coupled to their schema identity; custom values remain attached to their
/// case-sensitive custom name. Parsing and [`crate::parse_property_value`] construct these
/// checked bodies without performing cascade or substitution.
#[non_exhaustive]
#[expect(
    clippy::large_enum_variant,
    reason = "authored declaration variants retain their checked values inline"
)]
#[derive(Clone, Debug, PartialEq)]
pub enum CssDeclarationBody {
    /// A schema-recognized property carrying only its property-specific declared value.
    Known(CssKnownDeclaration),
    /// A case-sensitive custom property carrying its current authored value representation.
    Custom(CssCustomDeclaration),
}

/// A checked authored custom-property declaration.
///
/// Its private fields prevent attaching the custom value to a known property name. Parsing and
/// [`crate::parse_property_value`] preserve custom syntax without substituting references,
/// computing cascade, or validating a post-substitution value.
///
/// ```compile_fail
/// use surgeist_css::CssCustomDeclaration;
/// let _ = CssCustomDeclaration { name: todo!(), value: todo!() };
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomDeclaration {
    name: CssCustomPropertyName,
    value: CssCustomPropertyDeclaredValue,
}

impl CssCustomDeclaration {
    pub(crate) const fn new(
        name: CssCustomPropertyName,
        value: CssCustomPropertyDeclaredValue,
    ) -> Self {
        Self { name, value }
    }

    /// Returns the case-sensitive validated custom property name.
    #[must_use]
    pub const fn name(&self) -> &CssCustomPropertyName {
        &self.name
    }

    /// Returns the preserved authored custom-property value.
    #[must_use]
    pub const fn value(&self) -> &CssCustomPropertyDeclaredValue {
        &self.value
    }
}

/// A borrowed authored property-name view derived from a declaration body.
///
/// The known branch uses canonical generated identity; the custom branch preserves its
/// case-sensitive name. The view stores no parallel identity and performs no cascade lookup.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssPropertyNameRef<'a> {
    /// A canonical schema-generated known property identity.
    Known(CssKnownProperty),
    /// A borrowed case-sensitive custom property name.
    Custom(&'a CssCustomPropertyName),
}

/// A case-sensitive custom-property name in the authored CSS syntax phase.
///
/// [`Self::try_new`] accepts one complete authored CSS identifier token beginning with `--`,
/// including non-ASCII characters and escapes. [`Self::as_str`] returns its decoded semantic
/// identity, matching names produced by the stylesheet parser; it does not retain the name's
/// source escape spelling.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssCustomPropertyName {
    name: String,
}

impl CssCustomPropertyName {
    /// Tokenizes and validates one complete authored custom-property name.
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let authored = name.into();
        authored.strip_prefix("--")?;
        let mut input = cssparser::ParserInput::new(&authored);
        let mut parser = cssparser::Parser::new(&mut input);
        let decoded = parser.expect_ident_cloned().ok()?;
        let token_end = parser.position();
        parser.expect_exhausted().ok()?;
        if token_end.byte_index() != authored.len() {
            return None;
        }
        Self::from_ident_token(decoded.as_ref())
    }

    #[must_use]
    pub(crate) fn from_ident_token(name: &str) -> Option<Self> {
        name.strip_prefix("--")
            .filter(|suffix| !suffix.is_empty())
            .map(|_| Self {
                name: name.to_owned(),
            })
    }

    /// Returns the decoded, case-sensitive semantic custom-property name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAuthoredDeclarationValue {
    css: String,
}

impl CssAuthoredDeclarationValue {
    #[must_use]
    pub fn try_new(css: impl Into<String>) -> Option<Self> {
        let css = css.into();
        if css.trim().is_empty() {
            None
        } else {
            Some(Self::new(css))
        }
    }

    #[must_use]
    pub(crate) fn new(css: impl Into<String>) -> Self {
        Self { css: css.into() }
    }

    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.css
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssVariableReference {
    name: CssCustomPropertyName,
    fallback: Option<CssVariableFallback>,
}

impl CssVariableReference {
    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn new(name: CssCustomPropertyName, fallback: Option<CssVariableFallback>) -> Self {
        Self { name, fallback }
    }

    #[must_use]
    pub const fn name(&self) -> &CssCustomPropertyName {
        &self.name
    }

    #[must_use]
    pub const fn fallback(&self) -> Option<&CssVariableFallback> {
        self.fallback.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssVariableFallback {
    authored: CssAuthoredDeclarationValue,
    references: Vec<CssVariableReference>,
}

impl CssVariableFallback {
    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn new(
        authored: CssAuthoredDeclarationValue,
        references: Vec<CssVariableReference>,
    ) -> Self {
        Self {
            authored,
            references,
        }
    }

    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }

    #[must_use]
    pub fn references(&self) -> &[CssVariableReference] {
        &self.references
    }
}

/// An exact retained custom-property token stream in the authored CSS syntax phase.
///
/// The value can be empty and preserves interior UTF-8 source spelling after parser-owned boundary
/// trivia removal. It does not substitute variables, expose dependency tokens, compute cascade, or
/// validate a computed value.
///
/// ```compile_fail
/// use surgeist_css::CssCustomPropertyValue;
/// fn dependency_tokens(value: &CssCustomPropertyValue) {
///     let _ = value.references();
/// }
/// ```
///
/// ```compile_fail
/// use surgeist_css::CssCustomPropertyValue;
/// let _ = CssCustomPropertyValue { authored: todo!() };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomPropertyValue {
    authored: CssAuthoredDeclarationValue,
}

impl CssCustomPropertyValue {
    #[must_use]
    pub(crate) const fn new(authored: CssAuthoredDeclarationValue) -> Self {
        Self { authored }
    }

    /// Returns the exact retained UTF-8 source after parser-owned boundary trivia removal.
    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }

    /// Returns whether the retained authored token stream is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.authored.as_css().is_empty()
    }
}

/// A custom property's declared value in the authored CSS syntax phase.
///
/// The branches preserve the strict parser's existing distinction between authored token text
/// and a whole-value CSS-wide keyword. This value remains attached to a validated custom name and
/// does not perform substitution, cascade, or post-substitution validation.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssCustomPropertyDeclaredValue {
    /// Preserved authored custom-property token text.
    Value(CssCustomPropertyValue),
    /// A whole-value CSS-wide keyword.
    Global(CssGlobalKeyword),
}

impl CssCustomPropertyDeclaredValue {
    /// Returns the preserved authored custom-property value when present.
    #[must_use]
    pub const fn value(&self) -> Option<&CssCustomPropertyValue> {
        match self {
            Self::Value(value) => Some(value),
            Self::Global(_) => None,
        }
    }

    /// Returns the symbolic CSS-wide keyword when present.
    #[must_use]
    pub const fn global(&self) -> Option<CssGlobalKeyword> {
        match self {
            Self::Value(_) => None,
            Self::Global(keyword) => Some(*keyword),
        }
    }
}

/// A known property's authored value whose grammar depends on later CSS substitution.
///
/// The complete authored value remains symbolic. This value exposes only its retained CSS text: it
/// does not promise post-substitution grammar validity, resolve variables, or expose/build a
/// dependency graph.
///
/// ```compile_fail
/// use surgeist_css::CssSubstitutionDependentValue;
/// fn dependency_graph(value: &CssSubstitutionDependentValue) {
///     let _ = value.references();
/// }
/// ```
///
/// ```compile_fail
/// use surgeist_css::CssSubstitutionDependentValue;
/// let _ = CssSubstitutionDependentValue { authored: todo!() };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSubstitutionDependentValue {
    authored: CssAuthoredDeclarationValue,
}

impl CssSubstitutionDependentValue {
    #[must_use]
    pub(crate) const fn new(authored: CssAuthoredDeclarationValue) -> Self {
        Self { authored }
    }

    /// Returns the complete retained authored declaration value.
    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }
}

/// A CSS-wide keyword retained in the authored declaration phase.
///
/// It remains symbolic and does not apply inheritance, cascade, or revert behavior.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssGlobalKeyword {
    /// The authored `inherit` keyword.
    Inherit,
    /// The authored `initial` keyword.
    Initial,
    /// The authored `unset` keyword.
    Unset,
    /// The authored `revert` keyword.
    Revert,
    /// The authored `revert-layer` keyword.
    RevertLayer,
}

/// A property-specific declared value in the authored CSS syntax phase.
///
/// `Value` contains only the type selected by the active known-declaration variant. Global and
/// substitution-dependent syntax remain symbolic. These views do not apply cascade, perform
/// substitution, or contextually resolve the value.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssDeclaredValue<T> {
    /// A value accepted by the property's typed authored grammar.
    Value(T),
    /// A whole-value CSS-wide keyword.
    Global(CssGlobalKeyword),
    /// Authored syntax whose grammar depends on later substitution.
    SubstitutionDependent(CssSubstitutionDependentValue),
}

/// The authored declared-value domain for the `all` property.
///
/// Unlike ordinary known properties, `all` has no typed-value branch: it can retain only a
/// CSS-wide keyword or syntax whose grammar depends on later substitution. It does not apply
/// cascade, perform substitution, or resolve the resulting values.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssAllDeclaredValue {
    /// A whole-value CSS-wide keyword.
    Global(CssGlobalKeyword),
    /// Authored syntax whose validity depends on later substitution.
    SubstitutionDependent(CssSubstitutionDependentValue),
}

impl CssAllDeclaredValue {
    /// Returns the symbolic CSS-wide keyword when present.
    #[must_use]
    pub const fn global(&self) -> Option<CssGlobalKeyword> {
        match self {
            Self::Global(keyword) => Some(*keyword),
            Self::SubstitutionDependent(_) => None,
        }
    }

    /// Returns the substitution-dependent authored value when present.
    #[must_use]
    pub const fn substitution_dependent(&self) -> Option<&CssSubstitutionDependentValue> {
        match self {
            Self::Global(_) => None,
            Self::SubstitutionDependent(value) => Some(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFiniteNumber {
    value: f32,
}

impl CssFiniteNumber {
    #[must_use]
    pub const fn try_new(value: f32) -> Option<Self> {
        if value.is_finite() {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value
    }
}

/// An exact ordinary scalar or a domain-checked symbolic opacity calculation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOpacityValue {
    Scalar(crate::CssOpacityScalar),
    NumberCalculation(CssNumberCalculation),
    HintedNumberCalculation(CssHintedNumberCalculation),
    PercentageCalculation(CssPercentageCalculation),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAspectRatioValue {
    Auto,
    Ratio(crate::CssSpecifiedRatio),
    AutoRatio(crate::CssSpecifiedRatio),
}

/// Authored scrollbar thickness preference, initially `auto` and noninherited.
///
/// CSS Scrollbars 1 applies this keyword to scroll containers; its computed
/// value is the specified keyword and animation is by computed value. Actual
/// scrollbar thickness and root-to-viewport application belong downstream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollbarWidth {
    Auto,
    Thin,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBoxSizing {
    ContentBox,
    BorderBox,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLayoutPosition {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssDirection {
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflow {
    Visible,
    Clip,
    Hidden,
    Scroll,
    Auto,
}

/// A checked positive literal or a calculation whose range remains authored and symbolic.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssPositiveIntegerValue {
    Literal(CssPositiveIntegerLiteral),
    Calculation(CssIntegerCalculation),
}

impl CssPositiveIntegerValue {
    // Typed function math stays symbolic. A bare Integer calculation token
    // must cross the same positive-literal boundary as ordinary parsed input.
    pub(crate) fn normalized_positive_root(self) -> Option<Self> {
        match self {
            Self::Literal(literal) => Some(Self::Literal(literal)),
            Self::Calculation(calculation) => {
                let root =
                    crate::specified_numeric::significant_root(calculation.components()).ok()?;
                match root.view() {
                    crate::CssComponentValueRef::Token(_) => {
                        let integer =
                            crate::CssIntegerLiteral::try_from_component(root.clone()).ok()?;
                        CssPositiveIntegerLiteral::try_new(integer).map(Self::Literal)
                    }
                    crate::CssComponentValueRef::Function(_) => {
                        Some(Self::Calculation(calculation))
                    }
                    _ => None,
                }
            }
        }
    }
}

/// A positive integer whose ordinary authored magnitude is not machine-bounded.
#[derive(Clone, Debug)]
pub struct CssPositiveIntegerLiteral {
    integer: crate::CssIntegerLiteral,
}

impl CssPositiveIntegerLiteral {
    /// Accepts an exact integer token only when its authored sign and digits are positive.
    #[must_use]
    pub fn try_new(integer: crate::CssIntegerLiteral) -> Option<Self> {
        let text = integer.numeric().representation();
        let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
        (!text.starts_with('-') && digits.bytes().any(|digit| digit != b'0'))
            .then_some(Self { integer })
    }

    /// Borrows the checked exact integer token and its source origin.
    #[must_use]
    pub const fn integer(&self) -> &crate::CssIntegerLiteral {
        &self.integer
    }
}

impl PartialEq for CssPositiveIntegerLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.integer
            .component()
            .structural_eq_ignoring_origin(other.integer.component())
    }
}

impl PartialEq for CssPositiveIntegerValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(left), Self::Literal(right)) => left == right,
            (Self::Calculation(left), Self::Calculation(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

/// The authored `column-count` value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColumnCount {
    Auto,
    Count(CssPositiveIntegerValue),
}

/// The authored `column-fill` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColumnFill {
    Auto,
    Balance,
    BalanceAll,
}

/// The shared authored `<line-style>` domain used by borders and column rules.
pub type CssLineStyle = CssBorderStyle;

/// The shared authored `<line-width>` domain.
pub type CssLineWidth = crate::CssBorderWidth;

pub use crate::column_rule::CssColumnRule;

/// The authored `column-span` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColumnSpan {
    None,
    All,
}

/// The authored `column-width` value, including Sizing 4 box sizes.
pub type CssColumnWidth = crate::CssSizeValue;

/// The authored component values of the `columns` shorthand.
///
/// Omitted components are represented by their specified `auto` initial value. This syntax value
/// does not calculate used column sizes or perform multi-column layout.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColumns {
    width: CssColumnWidth,
    count: CssColumnCount,
}

impl CssColumns {
    /// Constructs both effective values; omitted shorthand components are supplied as `auto`.
    pub const fn new(width: CssColumnWidth, count: CssColumnCount) -> Self {
        Self { width, count }
    }

    #[must_use]
    pub const fn width(&self) -> &CssColumnWidth {
        &self.width
    }

    #[must_use]
    pub const fn count(&self) -> &CssColumnCount {
        &self.count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFloat {
    Left,
    Right,
    None,
    InlineStart,
    InlineEnd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssClear {
    Left,
    Right,
    Both,
    None,
    InlineStart,
    InlineEnd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVisibility {
    Visible,
    Hidden,
    Collapse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContentVisibility {
    Visible,
    Hidden,
    Auto,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContentString {
    value: String,
}

impl CssContentString {
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.contains('\0') {
            None
        } else {
            Some(Self { value })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
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

pub(crate) fn is_css_wide_keyword(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssListStylePosition {
    Inside,
    Outside,
}

/// A checked authored `flow-tolerance`, before contextual used-value resolution.
///
/// Grid3 permits `normal`, `infinite`, or a signed length-percentage. `normal`
/// remains symbolic because its used value depends on the downstream layout mode.
#[derive(Clone, Debug)]
pub struct CssFlowTolerance {
    value: FlowToleranceValue,
}

impl PartialEq for CssFlowTolerance {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (FlowToleranceValue::Normal, FlowToleranceValue::Normal)
            | (FlowToleranceValue::Infinite, FlowToleranceValue::Infinite) => true,
            (
                FlowToleranceValue::LengthPercentage(left),
                FlowToleranceValue::LengthPercentage(right),
            ) => left.structural_eq(right),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum FlowToleranceValue {
    Normal,
    Infinite,
    LengthPercentage(CssSpecifiedLengthPercentage),
}

/// Borrows the checked authored branch without resolving relative units or `normal`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFlowToleranceRef<'a> {
    Normal,
    Infinite,
    LengthPercentage(&'a CssSpecifiedLengthPercentage),
}

impl CssFlowTolerance {
    /// Constructs the symbolic initial value.
    #[must_use]
    pub const fn normal() -> Self {
        Self {
            value: FlowToleranceValue::Normal,
        }
    }

    /// Constructs the symbolic infinite placement tolerance.
    #[must_use]
    pub const fn infinite() -> Self {
        Self {
            value: FlowToleranceValue::Infinite,
        }
    }

    /// Composes a checked signed length-percentage without resolving it.
    ///
    /// The scalar owner excludes keywords and preserves its original origin.
    /// Signed operands and typed math remain symbolic until contextual resolution.
    #[must_use]
    pub fn length_percentage(value: CssSpecifiedLengthPercentage) -> Self {
        Self {
            value: FlowToleranceValue::LengthPercentage(value),
        }
    }

    /// Returns the checked authored branch and its unchanged numeric payload.
    #[must_use]
    pub const fn as_ref(&self) -> CssFlowToleranceRef<'_> {
        match &self.value {
            FlowToleranceValue::Normal => CssFlowToleranceRef::Normal,
            FlowToleranceValue::Infinite => CssFlowToleranceRef::Infinite,
            FlowToleranceValue::LengthPercentage(value) => {
                CssFlowToleranceRef::LengthPercentage(value)
            }
        }
    }
}

impl Default for CssFlowTolerance {
    fn default() -> Self {
        Self::normal()
    }
}

/// A representable decoded Values 4 custom identifier, preserving authored case.
/// CSS-wide keywords and `default` are excluded; consuming grammars own any
/// additional keyword exclusions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomIdent(CssIdent);

impl CssCustomIdent {
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        Self::try_from_ident(CssIdent::try_new(value).ok()?)
    }

    pub(crate) fn try_from_ident(value: CssIdent) -> Option<Self> {
        (!is_reserved_custom_ident(value.as_str())).then_some(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

fn is_reserved_custom_ident(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "default"
    )
}

/// A checked identifier shared by keyframes definitions and animation names.
/// Animations 1 excludes `none` in addition to the Values 4 reserved words.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssKeyframesIdent(CssCustomIdent);

impl CssKeyframesIdent {
    #[must_use]
    pub fn try_new(value: CssCustomIdent) -> Option<Self> {
        (!value.as_str().eq_ignore_ascii_case("none")).then_some(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A checked custom transition property name, retaining unknown names and case.
/// `none` and `all` use their distinct keyword variants instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTransitionPropertyName(CssCustomIdent);

impl CssTransitionPropertyName {
    #[must_use]
    pub fn try_new(value: CssCustomIdent) -> Option<Self> {
        (!value.as_str().eq_ignore_ascii_case("none")
            && !value.as_str().eq_ignore_ascii_case("all"))
        .then_some(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

mod grid;
pub use grid::*;
mod grid_placement;
pub use grid_placement::*;

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssIntegerValue {
    Literal(crate::CssIntegerLiteral),
    Calculation(CssIntegerCalculation),
}

/// An authored `z-index` value before contextual stacking or integer rounding.
///
/// CSS2 §9.9.1 defines `auto` or a signed integer. Integer literals preserve
/// their exact component origins; calculations retain unresolved numeric input.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssZIndexValue {
    Auto,
    Integer(CssIntegerValue),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBoxDecorationBreak {
    Slice,
    Clone,
}

/// An authored CSS Box Model Level 3 `<box>` edge keyword.
///
/// The value names a box edge only. It does not select a layout, paint, or
/// coordinate-space policy until a consuming property is interpreted by its
/// downstream owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssBoxEdgeKeyword {
    ContentBox,
    PaddingBox,
    BorderBox,
    MarginBox,
    FillBox,
    StrokeBox,
    ViewBox,
}

impl CssBoxEdgeKeyword {
    /// Converts one decoded CSS keyword to its typed edge identity.
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Some(match keyword.to_ascii_lowercase().as_str() {
            "content-box" => Self::ContentBox,
            "padding-box" => Self::PaddingBox,
            "border-box" => Self::BorderBox,
            "margin-box" => Self::MarginBox,
            "fill-box" => Self::FillBox,
            "stroke-box" => Self::StrokeBox,
            "view-box" => Self::ViewBox,
            _ => return None,
        })
    }

    /// Returns the canonical CSS keyword spelling.
    #[must_use]
    pub const fn as_css_str(self) -> &'static str {
        match self {
            Self::ContentBox => "content-box",
            Self::PaddingBox => "padding-box",
            Self::BorderBox => "border-box",
            Self::MarginBox => "margin-box",
            Self::FillBox => "fill-box",
            Self::StrokeBox => "stroke-box",
            Self::ViewBox => "view-box",
        }
    }
}

/// The authored `caret-color` value.
///
/// This value preserves color syntax only; it does not choose a rendered caret
/// color or apply contrast adjustments.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCaretColor {
    Auto,
    Color(Box<CssColor>),
}

/// The authored `resize` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssResize {
    None,
    Both,
    Horizontal,
    Vertical,
    /// Allows resizing along the block axis, resolved by the downstream writing mode.
    Block,
    /// Allows resizing along the inline axis, resolved by the downstream writing mode.
    Inline,
}

/// One independently authored Containment 2 containment kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContainComponent {
    Size,
    Layout,
    Style,
    Paint,
}

/// A nonempty, duplicate-free authored list of containment kinds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContainComponentList {
    components: Vec<CssContainComponent>,
}

impl CssContainComponentList {
    #[must_use]
    pub fn try_new(components: Vec<CssContainComponent>) -> Option<Self> {
        if components.is_empty()
            || components
                .iter()
                .enumerate()
                .any(|(index, component)| components[..index].contains(component))
        {
            return None;
        }
        Some(Self { components })
    }

    #[must_use]
    pub fn components(&self) -> &[CssContainComponent] {
        &self.components
    }
}

/// The authored Containment 2 `contain` value.
///
/// This is syntax only and does not apply size, layout, style, or paint containment.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContain {
    None,
    Strict,
    Content,
    Components(CssContainComponentList),
}

/// A checked authored `transform-box` reference-box keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTransformBox {
    edge: CssBoxEdgeKeyword,
}

impl CssTransformBox {
    #[must_use]
    pub const fn try_new(edge: CssBoxEdgeKeyword) -> Option<Self> {
        match edge {
            CssBoxEdgeKeyword::ContentBox
            | CssBoxEdgeKeyword::BorderBox
            | CssBoxEdgeKeyword::FillBox
            | CssBoxEdgeKeyword::StrokeBox
            | CssBoxEdgeKeyword::ViewBox => Some(Self { edge }),
            CssBoxEdgeKeyword::PaddingBox | CssBoxEdgeKeyword::MarginBox => None,
        }
    }

    #[must_use]
    pub const fn edge(&self) -> CssBoxEdgeKeyword {
        self.edge
    }
}

/// One authored Compositing and Blending Level 1 `<blend-mode>` keyword.
///
/// The value identifies authored syntax only; it does not perform painting or
/// compositing calculations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssBlendMode {
    Normal,
    Darken,
    Multiply,
    ColorBurn,
    Lighten,
    Screen,
    ColorDodge,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl CssBlendMode {
    /// Converts one decoded CSS keyword to its typed blend-mode identity.
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Some(match keyword.to_ascii_lowercase().as_str() {
            "normal" => Self::Normal,
            "darken" => Self::Darken,
            "multiply" => Self::Multiply,
            "color-burn" => Self::ColorBurn,
            "lighten" => Self::Lighten,
            "screen" => Self::Screen,
            "color-dodge" => Self::ColorDodge,
            "overlay" => Self::Overlay,
            "soft-light" => Self::SoftLight,
            "hard-light" => Self::HardLight,
            "difference" => Self::Difference,
            "exclusion" => Self::Exclusion,
            "hue" => Self::Hue,
            "saturation" => Self::Saturation,
            "color" => Self::Color,
            "luminosity" => Self::Luminosity,
            _ => return None,
        })
    }

    /// Returns the canonical CSS keyword spelling.
    #[must_use]
    pub const fn as_css_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Darken => "darken",
            Self::Multiply => "multiply",
            Self::ColorBurn => "color-burn",
            Self::Lighten => "lighten",
            Self::Screen => "screen",
            Self::ColorDodge => "color-dodge",
            Self::Overlay => "overlay",
            Self::SoftLight => "soft-light",
            Self::HardLight => "hard-light",
            Self::Difference => "difference",
            Self::Exclusion => "exclusion",
            Self::Hue => "hue",
            Self::Saturation => "saturation",
            Self::Color => "color",
            Self::Luminosity => "luminosity",
        }
    }
}

/// A nonempty ordered authored comma-separated blend-mode list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBlendModeList {
    modes: Vec<CssBlendMode>,
}

impl CssBlendModeList {
    #[must_use]
    pub fn try_new(modes: Vec<CssBlendMode>) -> Option<Self> {
        (!modes.is_empty()).then_some(Self { modes })
    }

    #[must_use]
    pub fn modes(&self) -> &[CssBlendMode] {
        &self.modes
    }
}

/// The authored `isolation` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssIsolation {
    Auto,
    Isolate,
}

/// The authored `border-collapse` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBorderCollapse {
    Collapse,
    Separate,
}

/// The checked horizontal and vertical authored `border-spacing` lengths.
#[derive(Clone, Debug)]
pub struct CssBorderSpacing {
    horizontal: CssSpecifiedNonNegativeLength,
    vertical: CssSpecifiedNonNegativeLength,
}

impl PartialEq for CssBorderSpacing {
    fn eq(&self, other: &Self) -> bool {
        self.horizontal.structural_eq(&other.horizontal)
            && self.vertical.structural_eq(&other.vertical)
    }
}

impl CssBorderSpacing {
    /// Composes checked nonnegative horizontal and vertical lengths.
    pub const fn new(
        horizontal: CssSpecifiedNonNegativeLength,
        vertical: CssSpecifiedNonNegativeLength,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
    pub const fn horizontal(&self) -> &CssSpecifiedNonNegativeLength {
        &self.horizontal
    }
    pub const fn vertical(&self) -> &CssSpecifiedNonNegativeLength {
        &self.vertical
    }
}

/// The authored `caption-side` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCaptionSide {
    Top,
    Bottom,
}

/// One authored edge of a deprecated Masking 1 `clip: rect(...)` value.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssClipEdge {
    Auto,
    Length(CssSpecifiedLength),
}

impl PartialEq for CssClipEdge {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) => true,
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

/// The four authored edges of a deprecated Masking 1 clipping rectangle.
#[derive(Clone, Debug, PartialEq)]
pub struct CssClipRect {
    top: CssClipEdge,
    right: CssClipEdge,
    bottom: CssClipEdge,
    left: CssClipEdge,
}

impl CssClipRect {
    #[must_use]
    pub const fn new(
        top: CssClipEdge,
        right: CssClipEdge,
        bottom: CssClipEdge,
        left: CssClipEdge,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub const fn top(&self) -> &CssClipEdge {
        &self.top
    }

    #[must_use]
    pub const fn right(&self) -> &CssClipEdge {
        &self.right
    }

    #[must_use]
    pub const fn bottom(&self) -> &CssClipEdge {
        &self.bottom
    }

    #[must_use]
    pub const fn left(&self) -> &CssClipEdge {
        &self.left
    }
}

/// The authored deprecated Masking 1 `clip` value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssClip {
    Auto,
    Rect(CssClipRect),
}

/// The authored `empty-cells` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssEmptyCells {
    Show,
    Hide,
}

/// The authored `break-before` or `break-after` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBreakBetween {
    Auto,
    Avoid,
    AvoidPage,
    Page,
    Left,
    Right,
    Recto,
    Verso,
    AvoidColumn,
    Column,
    AvoidRegion,
    Region,
}

/// The authored `break-inside` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBreakInside {
    Auto,
    Avoid,
    AvoidPage,
    AvoidColumn,
    AvoidRegion,
}

/// One authored opening/closing pair in a `quotes` value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssQuotePair {
    open: CssContentString,
    close: CssContentString,
}

impl CssQuotePair {
    #[must_use]
    pub const fn new(open: CssContentString, close: CssContentString) -> Self {
        Self { open, close }
    }

    #[must_use]
    pub const fn open(&self) -> &CssContentString {
        &self.open
    }

    #[must_use]
    pub const fn close(&self) -> &CssContentString {
        &self.close
    }
}

/// A nonempty authored list of quotation-mark pairs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssQuotePairList {
    pairs: Vec<CssQuotePair>,
}

impl CssQuotePairList {
    #[must_use]
    pub fn try_new(pairs: Vec<CssQuotePair>) -> Option<Self> {
        (!pairs.is_empty()).then_some(Self { pairs })
    }

    #[must_use]
    pub fn pairs(&self) -> &[CssQuotePair] {
        &self.pairs
    }
}

/// The authored CSS Generated Content 3 `quotes` value. Language-dependent
/// quotation marks remain unresolved until the element's context is known.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssQuotes {
    Auto,
    None,
    MatchParent,
    Pairs(CssQuotePairList),
}

/// The authored `table-layout` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTableLayout {
    Auto,
    Fixed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
}

/// The authored `text-combine-upright` value, preserving an omitted digit count.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTextCombineUpright {
    None,
    All,
    /// An omitted count computes to two later; specified values retain omission.
    Digits(Option<CssTextCombineDigitCount>),
}

#[derive(Clone, Debug, PartialEq)]
enum CssTextCombineDigitCountValue {
    Literal(i32),
    Calculation(CssIntegerCalculation),
}

/// A literal digit count from two through four, or an authored integer calculation.
/// Calculations retain their specified value without computed rounding or clamping.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTextCombineDigitCount {
    value: CssTextCombineDigitCountValue,
}

impl CssTextCombineDigitCount {
    #[must_use]
    pub const fn try_literal(value: i32) -> Option<Self> {
        if value >= 2 && value <= 4 {
            Some(Self {
                value: CssTextCombineDigitCountValue::Literal(value),
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn from_calculation(value: CssIntegerCalculation) -> Self {
        Self {
            value: CssTextCombineDigitCountValue::Calculation(value),
        }
    }

    #[must_use]
    pub const fn literal(&self) -> Option<i32> {
        match self.value {
            CssTextCombineDigitCountValue::Literal(value) => Some(value),
            CssTextCombineDigitCountValue::Calculation(_) => None,
        }
    }

    #[must_use]
    pub const fn calculation(&self) -> Option<&CssIntegerCalculation> {
        match &self.value {
            CssTextCombineDigitCountValue::Literal(_) => None,
            CssTextCombineDigitCountValue::Calculation(value) => Some(value),
        }
    }
}

/// The authored `text-orientation` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextOrientation {
    Mixed,
    Upright,
    Sideways,
}

/// The authored `unicode-bidi` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssUnicodeBidi {
    Normal,
    Embed,
    Isolate,
    BidiOverride,
    IsolateOverride,
    Plaintext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
    MatchParent,
}

#[derive(Clone, Debug)]
pub struct CssTextIndent {
    length: CssSpecifiedLengthPercentage,
    hanging: bool,
    each_line: bool,
}

impl PartialEq for CssTextIndent {
    fn eq(&self, other: &Self) -> bool {
        self.length.structural_eq(&other.length)
            && self.hanging == other.hanging
            && self.each_line == other.each_line
    }
}

impl CssTextIndent {
    /// Retains the checked indent and independently authored keyword flags.
    pub const fn new(length: CssSpecifiedLengthPercentage, hanging: bool, each_line: bool) -> Self {
        Self {
            length,
            hanging,
            each_line,
        }
    }
    pub const fn length(&self) -> &CssSpecifiedLengthPercentage {
        &self.length
    }
    pub const fn hanging(&self) -> bool {
        self.hanging
    }
    pub const fn each_line(&self) -> bool {
        self.each_line
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssVerticalAlign {
    Baseline,
    Sub,
    Super,
    TextTop,
    TextBottom,
    Middle,
    Top,
    Bottom,
    Length(CssSpecifiedLengthPercentage),
}

impl PartialEq for CssVerticalAlign {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            (Self::Baseline, Self::Baseline)
            | (Self::Sub, Self::Sub)
            | (Self::Super, Self::Super)
            | (Self::TextTop, Self::TextTop)
            | (Self::TextBottom, Self::TextBottom)
            | (Self::Middle, Self::Middle)
            | (Self::Top, Self::Top)
            | (Self::Bottom, Self::Bottom) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFamilyNameKind {
    Quoted,
    IdentSequence,
    Generic,
}

/// A generic family with CSS-defined meaning, including script-specific functions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGenericFontFamily {
    Serif,
    SansSerif,
    Cursive,
    Fantasy,
    Monospace,
    SystemUi,
    Math,
    UiSerif,
    UiSansSerif,
    UiMonospace,
    UiRounded,
    Fangsong,
    Kai,
    KhmerMul,
    Nastaliq,
}

impl CssGenericFontFamily {
    pub(crate) fn from_keyword(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "serif" => Some(Self::Serif),
            "sans-serif" => Some(Self::SansSerif),
            "cursive" => Some(Self::Cursive),
            "fantasy" => Some(Self::Fantasy),
            "monospace" => Some(Self::Monospace),
            "system-ui" => Some(Self::SystemUi),
            "math" => Some(Self::Math),
            "ui-serif" => Some(Self::UiSerif),
            "ui-sans-serif" => Some(Self::UiSansSerif),
            "ui-monospace" => Some(Self::UiMonospace),
            "ui-rounded" => Some(Self::UiRounded),
            _ => None,
        }
    }

    pub(crate) fn from_script_keyword(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "fangsong" => Some(Self::Fangsong),
            "kai" => Some(Self::Kai),
            "khmer-mul" => Some(Self::KhmerMul),
            "nastaliq" => Some(Self::Nastaliq),
            _ => None,
        }
    }

    const fn as_css(self) -> &'static str {
        match self {
            Self::Serif => "serif",
            Self::SansSerif => "sans-serif",
            Self::Cursive => "cursive",
            Self::Fantasy => "fantasy",
            Self::Monospace => "monospace",
            Self::SystemUi => "system-ui",
            Self::Math => "math",
            Self::UiSerif => "ui-serif",
            Self::UiSansSerif => "ui-sans-serif",
            Self::UiMonospace => "ui-monospace",
            Self::UiRounded => "ui-rounded",
            Self::Fangsong => "generic(fangsong)",
            Self::Kai => "generic(kai)",
            Self::KhmerMul => "generic(khmer-mul)",
            Self::Nastaliq => "generic(nastaliq)",
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
enum CssFontFamilyNameRepresentation {
    Quoted(String),
    IdentSequence {
        identifiers: Vec<String>,
        joined: String,
    },
    Generic(CssGenericFontFamily),
}

/// A checked literal family name or a CSS-defined generic family.
///
/// Identifier boundaries participate in authored equality. A single identifier
/// containing an escaped space can have the same joined name as two identifiers
/// while retaining a different authored representation.
#[derive(Clone, Eq, PartialEq)]
pub struct CssFontFamilyName {
    representation: CssFontFamilyNameRepresentation,
}

impl std::fmt::Debug for CssFontFamilyName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssFontFamilyName")
            .field("kind", &self.kind())
            .field("value", &self.as_str())
            .finish()
    }
}

impl CssFontFamilyName {
    /// Constructs a quoted name from decoded string content, including empty or
    /// whitespace-only content. NUL is rejected because CSS would replace it.
    #[must_use]
    pub fn try_quoted(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.contains('\0')).then_some(Self {
            representation: CssFontFamilyNameRepresentation::Quoted(value),
        })
    }

    /// Constructs a name from one or more decoded identifier tokens.
    ///
    /// Tokens are not split, trimmed, or decoded again. Each token must be
    /// nonempty, contain no NUL, and not be a simple generic-family keyword,
    /// CSS-wide keyword, or `default` in any ASCII case. Escaped whitespace,
    /// punctuation, and leading digits inside a decoded token remain valid.
    /// System-font spellings such as `menu` are ordinary names in this position.
    #[must_use]
    pub fn try_ident_sequence(identifiers: Vec<String>) -> Option<Self> {
        if identifiers.is_empty()
            || identifiers.iter().any(|identifier| {
                identifier.is_empty()
                    || identifier.contains('\0')
                    || CssGenericFontFamily::from_keyword(identifier).is_some()
                    || is_css_wide_keyword(identifier)
                    || identifier.eq_ignore_ascii_case("default")
            })
        {
            return None;
        }
        let joined = identifiers.join(" ");
        Some(Self {
            representation: CssFontFamilyNameRepresentation::IdentSequence {
                identifiers,
                joined,
            },
        })
    }

    /// Constructs a name from exactly one decoded identifier token.
    #[must_use]
    pub fn try_ident(value: impl Into<String>) -> Option<Self> {
        Self::try_ident_sequence(vec![value.into()])
    }

    /// Constructs a CSS-defined generic family without a literal-name ambiguity.
    #[must_use]
    pub const fn generic(generic: CssGenericFontFamily) -> Self {
        Self {
            representation: CssFontFamilyNameRepresentation::Generic(generic),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssFontFamilyNameKind {
        match &self.representation {
            CssFontFamilyNameRepresentation::Quoted(_) => CssFontFamilyNameKind::Quoted,
            CssFontFamilyNameRepresentation::IdentSequence { .. } => {
                CssFontFamilyNameKind::IdentSequence
            }
            CssFontFamilyNameRepresentation::Generic(_) => CssFontFamilyNameKind::Generic,
        }
    }

    /// Returns the decoded literal name or the canonical CSS spelling of a generic.
    /// Identifier tokens are joined with one U+0020 SPACE between tokens; spaces
    /// within a token are preserved. This is not a serialization of a literal name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match &self.representation {
            CssFontFamilyNameRepresentation::Quoted(value) => value,
            CssFontFamilyNameRepresentation::IdentSequence { joined, .. } => joined,
            CssFontFamilyNameRepresentation::Generic(generic) => generic.as_css(),
        }
    }

    /// Returns the decoded identifier tokens, preserving their authored boundaries.
    #[must_use]
    pub fn identifier_tokens(&self) -> Option<&[String]> {
        match &self.representation {
            CssFontFamilyNameRepresentation::IdentSequence { identifiers, .. } => Some(identifiers),
            CssFontFamilyNameRepresentation::Quoted(_)
            | CssFontFamilyNameRepresentation::Generic(_) => None,
        }
    }

    #[must_use]
    pub const fn generic_family(&self) -> Option<CssGenericFontFamily> {
        match &self.representation {
            CssFontFamilyNameRepresentation::Generic(generic) => Some(*generic),
            CssFontFamilyNameRepresentation::Quoted(_)
            | CssFontFamilyNameRepresentation::IdentSequence { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFamilyList {
    families: Vec<CssFontFamilyName>,
}

impl CssFontFamilyList {
    #[must_use]
    pub fn try_new(families: Vec<CssFontFamilyName>) -> Option<Self> {
        if families.is_empty() {
            None
        } else {
            Some(Self::new(families))
        }
    }

    #[must_use]
    pub(crate) fn new(families: Vec<CssFontFamilyName>) -> Self {
        Self { families }
    }

    #[must_use]
    pub fn families(&self) -> &[CssFontFamilyName] {
        &self.families
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSystemFont {
    Caption,
    Icon,
    Menu,
    MessageBox,
    SmallCaption,
    StatusBar,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssExplicitFont {
    style: Option<CssFontStyle>,
    variant: Option<CssFontVariant>,
    weight: Option<CssFontWeight>,
    stretch: Option<CssFontWidthKeyword>,
    size: CssFontSize,
    line_height: Option<CssLineHeight>,
    families: CssFontFamilyList,
}

impl CssExplicitFont {
    #[must_use]
    pub fn try_new(
        style: Option<CssFontStyle>,
        variant: Option<CssFontVariant>,
        weight: Option<CssFontWeight>,
        stretch: Option<CssFontWidthKeyword>,
        size: CssFontSize,
        line_height: Option<CssLineHeight>,
        families: CssFontFamilyList,
    ) -> Option<Self> {
        (!families.families().is_empty()).then_some(Self {
            style,
            variant,
            weight,
            stretch,
            size,
            line_height,
            families,
        })
    }

    #[must_use]
    pub const fn style(&self) -> Option<&CssFontStyle> {
        self.style.as_ref()
    }

    #[must_use]
    pub const fn variant(&self) -> Option<CssFontVariant> {
        self.variant
    }

    #[must_use]
    pub const fn weight(&self) -> Option<&CssFontWeight> {
        self.weight.as_ref()
    }

    #[must_use]
    pub const fn stretch(&self) -> Option<CssFontWidthKeyword> {
        self.stretch
    }

    #[must_use]
    pub const fn size(&self) -> &CssFontSize {
        &self.size
    }

    #[must_use]
    pub const fn line_height(&self) -> Option<&CssLineHeight> {
        self.line_height.as_ref()
    }

    #[must_use]
    pub const fn families(&self) -> &CssFontFamilyList {
        &self.families
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontValue {
    Explicit(CssExplicitFont),
    System(CssSystemFont),
}

/// The authored unforced-wrapping mode, independent of the line selection style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextWrapMode {
    Wrap,
    NoWrap,
}

/// The authored line selection style; no wrapping algorithm is executed here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextWrapStyle {
    Auto,
    Balance,
    Stable,
    Pretty,
    AvoidShortLastLine,
}

/// Nonempty authored text-wrap constituents, retaining omitted whole roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTextWrap {
    mode: Option<CssTextWrapMode>,
    style: Option<CssTextWrapStyle>,
}
impl CssTextWrap {
    /// Admits at least one constituent without inserting projection defaults.
    #[must_use]
    pub const fn try_new(
        mode: Option<CssTextWrapMode>,
        style: Option<CssTextWrapStyle>,
    ) -> Option<Self> {
        if mode.is_none() && style.is_none() {
            None
        } else {
            Some(Self { mode, style })
        }
    }
    /// Returns the authored mode, or its original omission.
    #[must_use]
    pub const fn mode(&self) -> Option<CssTextWrapMode> {
        self.mode
    }
    /// Returns the authored style, or its original omission.
    #[must_use]
    pub const fn style(&self) -> Option<CssTextWrapStyle> {
        self.style
    }
}

/// The authored whitespace preservation/collapse choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWhiteSpaceCollapse {
    Collapse,
    Discard,
    Preserve,
    PreserveBreaks,
    PreserveSpaces,
    BreakSpaces,
}

/// Independent trim flags; the empty set is the authored none value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssWhiteSpaceTrim {
    discard_before: bool,
    discard_after: bool,
    discard_inner: bool,
}
impl CssWhiteSpaceTrim {
    /// Constructs any of the eight intrinsically valid trim sets.
    #[must_use]
    pub const fn new(discard_before: bool, discard_after: bool, discard_inner: bool) -> Self {
        Self {
            discard_before,
            discard_after,
            discard_inner,
        }
    }
    /// Constructs the empty none set.
    #[must_use]
    pub const fn none() -> Self {
        Self::new(false, false, false)
    }
    /// Whether leading whitespace is marked for discarding.
    #[must_use]
    pub const fn discard_before(&self) -> bool {
        self.discard_before
    }
    /// Whether trailing whitespace is marked for discarding.
    #[must_use]
    pub const fn discard_after(&self) -> bool {
        self.discard_after
    }
    /// Whether inner whitespace is marked for discarding.
    #[must_use]
    pub const fn discard_inner(&self) -> bool {
        self.discard_inner
    }
    /// Whether this is the empty none set.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        !self.discard_before && !self.discard_after && !self.discard_inner
    }
}

/// The four special white-space spellings, distinct from constituent forms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWhiteSpaceKeyword {
    Normal,
    Pre,
    PreWrap,
    PreLine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WhiteSpaceValue {
    Keyword(CssWhiteSpaceKeyword),
    Components {
        collapse: Option<CssWhiteSpaceCollapse>,
        mode: Option<CssTextWrapMode>,
        trim: Option<CssWhiteSpaceTrim>,
    },
}

/// A special white-space spelling or nonempty authored constituent composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssWhiteSpace {
    value: WhiteSpaceValue,
}
impl CssWhiteSpace {
    /// Retains a special spelling without manufacturing authored constituents.
    #[must_use]
    pub const fn from_keyword(keyword: CssWhiteSpaceKeyword) -> Self {
        Self {
            value: WhiteSpaceValue::Keyword(keyword),
        }
    }
    /// Admits a nonempty constituent composition, retaining each omission.
    #[must_use]
    pub const fn try_new(
        collapse: Option<CssWhiteSpaceCollapse>,
        mode: Option<CssTextWrapMode>,
        trim: Option<CssWhiteSpaceTrim>,
    ) -> Option<Self> {
        if collapse.is_none() && mode.is_none() && trim.is_none() {
            None
        } else {
            Some(Self {
                value: WhiteSpaceValue::Components {
                    collapse,
                    mode,
                    trim,
                },
            })
        }
    }
    /// Returns the special spelling, if this is the special-form branch.
    #[must_use]
    pub const fn keyword(&self) -> Option<CssWhiteSpaceKeyword> {
        match self.value {
            WhiteSpaceValue::Keyword(value) => Some(value),
            WhiteSpaceValue::Components { .. } => None,
        }
    }
    /// Returns only an authored component; special forms have no such field.
    #[must_use]
    pub const fn collapse(&self) -> Option<CssWhiteSpaceCollapse> {
        match self.value {
            WhiteSpaceValue::Components { collapse, .. } => collapse,
            WhiteSpaceValue::Keyword(_) => None,
        }
    }
    /// Returns only an authored component; special forms have no such field.
    #[must_use]
    pub const fn mode(&self) -> Option<CssTextWrapMode> {
        match self.value {
            WhiteSpaceValue::Components { mode, .. } => mode,
            WhiteSpaceValue::Keyword(_) => None,
        }
    }
    /// Returns only an authored component; special forms have no such field.
    #[must_use]
    pub const fn trim(&self) -> Option<CssWhiteSpaceTrim> {
        match self.value {
            WhiteSpaceValue::Components { trim, .. } => trim,
            WhiteSpaceValue::Keyword(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWordBreak {
    Normal,
    BreakAll,
    KeepAll,
    Manual,
    AutoPhrase,
    BreakWord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflowWrap {
    Normal,
    BreakWord,
    Anywhere,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextOverflow {
    Clip,
    Ellipsis,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTextDecoration {
    line: Option<CssTextDecorationLine>,
    color: Option<Box<CssColor>>,
    style: Option<CssTextDecorationStyle>,
    thickness: Option<CssTextDecorationThickness>,
}

impl CssTextDecoration {
    #[must_use]
    pub fn try_new(
        line: Option<CssTextDecorationLine>,
        color: Option<CssColor>,
        style: Option<CssTextDecorationStyle>,
        thickness: Option<CssTextDecorationThickness>,
    ) -> Option<Self> {
        if line.is_none() && color.is_none() && style.is_none() && thickness.is_none() {
            None
        } else {
            Some(Self::new(line, color, style, thickness))
        }
    }

    pub(crate) fn new(
        line: Option<CssTextDecorationLine>,
        color: Option<CssColor>,
        style: Option<CssTextDecorationStyle>,
        thickness: Option<CssTextDecorationThickness>,
    ) -> Self {
        Self {
            line,
            color: color.map(Box::new),
            style,
            thickness,
        }
    }

    #[must_use]
    pub const fn line(&self) -> Option<&CssTextDecorationLine> {
        self.line.as_ref()
    }

    #[must_use]
    pub fn color(&self) -> Option<&CssColor> {
        self.color.as_deref()
    }

    #[must_use]
    pub const fn style(&self) -> Option<CssTextDecorationStyle> {
        self.style
    }

    #[must_use]
    pub const fn thickness(&self) -> Option<&CssTextDecorationThickness> {
        self.thickness.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTextDecorationLine {
    components: Vec<CssTextDecorationLineComponent>,
    none: bool,
}

impl CssTextDecorationLine {
    #[must_use]
    pub fn try_new(components: Vec<CssTextDecorationLineComponent>) -> Option<Self> {
        if components.is_empty() || has_duplicate_decoration_line_components(&components) {
            None
        } else {
            Some(Self::new(components))
        }
    }

    #[must_use]
    pub(crate) fn new(components: Vec<CssTextDecorationLineComponent>) -> Self {
        Self {
            components,
            none: false,
        }
    }

    #[must_use]
    pub(crate) fn none() -> Self {
        Self {
            components: Vec::new(),
            none: true,
        }
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.none
    }

    #[must_use]
    pub fn components(&self) -> &[CssTextDecorationLineComponent] {
        &self.components
    }
}

fn has_duplicate_decoration_line_components(components: &[CssTextDecorationLineComponent]) -> bool {
    components.iter().enumerate().any(|(index, component)| {
        components
            .iter()
            .skip(index + 1)
            .any(|candidate| candidate == component)
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationLineComponent {
    Underline,
    Overline,
    LineThrough,
    Blink,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationStyle {
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTextDecorationThickness {
    Auto,
    FromFont,
    /// A signed authored thickness, before the actual value's device-pixel floor.
    Length(CssSpecifiedLengthPercentage),
}

impl PartialEq for CssTextDecorationThickness {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            (Self::Auto, Self::Auto) | (Self::FromFont, Self::FromFont) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextTransform {
    None,
    MathAuto,
    Transforms(CssTextTransformSet),
}

/// The exclusive authored case transformation, before language-sensitive processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextTransformCase {
    Capitalize,
    Uppercase,
    Lowercase,
}

/// A nonempty authored combination of case, width and kana transformations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTextTransformSet {
    case: Option<CssTextTransformCase>,
    full_width: bool,
    full_size_kana: bool,
}

impl CssTextTransformSet {
    /// Rejects the empty set; `none` and `math-auto` are separate whole values.
    pub const fn try_new(
        case: Option<CssTextTransformCase>,
        full_width: bool,
        full_size_kana: bool,
    ) -> Option<Self> {
        if case.is_none() && !full_width && !full_size_kana {
            None
        } else {
            Some(Self {
                case,
                full_width,
                full_size_kana,
            })
        }
    }
    pub const fn case(&self) -> Option<CssTextTransformCase> {
        self.case
    }
    pub const fn full_width(&self) -> bool {
        self.full_width
    }
    pub const fn full_size_kana(&self) -> bool {
        self.full_size_kana
    }
}

/// An authored preference for breaking within an inline box.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWrapInside {
    Auto,
    Avoid,
}

/// An authored break preference used independently by `wrap-before` and `wrap-after`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWrapBoundary {
    Auto,
    Avoid,
    AvoidLine,
    AvoidFlex,
    Line,
    Flex,
}

/// An authored line-breaking strictness, without language or layout execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLineBreak {
    Auto,
    Loose,
    Normal,
    Strict,
    Anywhere,
}

/// An authored separator transformation with a mandatory base outside `none`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWordSpaceTransform {
    None,
    Space { auto_phrase: bool },
    IdeographicSpace { auto_phrase: bool },
}

/// An authored tab interval: a space-advance multiplier or an exact length.
/// Literal ranges are checked by the child; calculation ranges remain deferred.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTabSize {
    Number(CssSpecifiedNonNegativeNumber),
    Length(CssSpecifiedNonNegativeLength),
}

impl PartialEq for CssTabSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left.structural_eq(right),
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLengthUnit {
    Px,
    Em,
    Rem,
    Ex,
    Rex,
    Cap,
    Rcap,
    Ch,
    Rch,
    Ic,
    Ric,
    Lh,
    Rlh,
    Vw,
    Vh,
    Vi,
    Vb,
    Vmin,
    Vmax,
    Svw,
    Svh,
    Svi,
    Svb,
    Svmin,
    Svmax,
    Lvw,
    Lvh,
    Lvi,
    Lvb,
    Lvmin,
    Lvmax,
    Dvw,
    Dvh,
    Dvi,
    Dvb,
    Dvmin,
    Dvmax,
    Cqw,
    Cqh,
    Cqi,
    Cqb,
    Cqmin,
    Cqmax,
    Cm,
    Mm,
    Q,
    In,
    Pc,
    Pt,
}

impl CssLengthUnit {
    pub(crate) fn from_css_unit(unit: &str) -> Option<Self> {
        Some(match unit.to_ascii_lowercase().as_str() {
            "px" => Self::Px,
            "em" => Self::Em,
            "rem" => Self::Rem,
            "ex" => Self::Ex,
            "rex" => Self::Rex,
            "cap" => Self::Cap,
            "rcap" => Self::Rcap,
            "ch" => Self::Ch,
            "rch" => Self::Rch,
            "ic" => Self::Ic,
            "ric" => Self::Ric,
            "lh" => Self::Lh,
            "rlh" => Self::Rlh,
            "vw" => Self::Vw,
            "vh" => Self::Vh,
            "vi" => Self::Vi,
            "vb" => Self::Vb,
            "vmin" => Self::Vmin,
            "vmax" => Self::Vmax,
            "svw" => Self::Svw,
            "svh" => Self::Svh,
            "svi" => Self::Svi,
            "svb" => Self::Svb,
            "svmin" => Self::Svmin,
            "svmax" => Self::Svmax,
            "lvw" => Self::Lvw,
            "lvh" => Self::Lvh,
            "lvi" => Self::Lvi,
            "lvb" => Self::Lvb,
            "lvmin" => Self::Lvmin,
            "lvmax" => Self::Lvmax,
            "dvw" => Self::Dvw,
            "dvh" => Self::Dvh,
            "dvi" => Self::Dvi,
            "dvb" => Self::Dvb,
            "dvmin" => Self::Dvmin,
            "dvmax" => Self::Dvmax,
            "cqw" => Self::Cqw,
            "cqh" => Self::Cqh,
            "cqi" => Self::Cqi,
            "cqb" => Self::Cqb,
            "cqmin" => Self::Cqmin,
            "cqmax" => Self::Cqmax,
            "cm" => Self::Cm,
            "mm" => Self::Mm,
            "q" => Self::Q,
            "in" => Self::In,
            "pc" => Self::Pc,
            "pt" => Self::Pt,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn as_css_str(self) -> &'static str {
        match self {
            Self::Px => "px",
            Self::Em => "em",
            Self::Rem => "rem",
            Self::Ex => "ex",
            Self::Rex => "rex",
            Self::Cap => "cap",
            Self::Rcap => "rcap",
            Self::Ch => "ch",
            Self::Rch => "rch",
            Self::Ic => "ic",
            Self::Ric => "ric",
            Self::Lh => "lh",
            Self::Rlh => "rlh",
            Self::Vw => "vw",
            Self::Vh => "vh",
            Self::Vi => "vi",
            Self::Vb => "vb",
            Self::Vmin => "vmin",
            Self::Vmax => "vmax",
            Self::Svw => "svw",
            Self::Svh => "svh",
            Self::Svi => "svi",
            Self::Svb => "svb",
            Self::Svmin => "svmin",
            Self::Svmax => "svmax",
            Self::Lvw => "lvw",
            Self::Lvh => "lvh",
            Self::Lvi => "lvi",
            Self::Lvb => "lvb",
            Self::Lvmin => "lvmin",
            Self::Lvmax => "lvmax",
            Self::Dvw => "dvw",
            Self::Dvh => "dvh",
            Self::Dvi => "dvi",
            Self::Dvb => "dvb",
            Self::Dvmin => "dvmin",
            Self::Dvmax => "dvmax",
            Self::Cqw => "cqw",
            Self::Cqh => "cqh",
            Self::Cqi => "cqi",
            Self::Cqb => "cqb",
            Self::Cqmin => "cqmin",
            Self::Cqmax => "cqmax",
            Self::Cm => "cm",
            Self::Mm => "mm",
            Self::Q => "q",
            Self::In => "in",
            Self::Pc => "pc",
            Self::Pt => "pt",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBorderStyle {
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBoxShadow {
    None,
    Shadows(CssBoxShadowList),
}

/// A nonempty ordered list of authored box shadows.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBoxShadowList {
    shadows: Vec<CssShadow>,
}

impl CssBoxShadowList {
    /// Rejects an empty list; retains the authored member order.
    pub fn try_new(shadows: Vec<CssShadow>) -> Option<Self> {
        if shadows.is_empty() {
            None
        } else {
            Some(Self { shadows })
        }
    }

    #[must_use]
    pub fn shadows(&self) -> &[CssShadow] {
        &self.shadows
    }
}

/// An authored box shadow with optional blur, spread, color, and inset.
#[derive(Clone, Debug)]
pub struct CssShadow {
    inset: bool,
    offset_x: CssSpecifiedLength,
    offset_y: CssSpecifiedLength,
    blur_radius: Option<CssSpecifiedNonNegativeLength>,
    spread_radius: Option<CssSpecifiedLength>,
    color: Option<Box<CssColor>>,
}

impl PartialEq for CssShadow {
    fn eq(&self, other: &Self) -> bool {
        self.inset == other.inset
            && self.offset_x.structural_eq(&other.offset_x)
            && self.offset_y.structural_eq(&other.offset_y)
            && match (&self.blur_radius, &other.blur_radius) {
                (Some(left), Some(right)) => left.structural_eq(right),
                (None, None) => true,
                _ => false,
            }
            && match (&self.spread_radius, &other.spread_radius) {
                (Some(left), Some(right)) => left.structural_eq(right),
                (None, None) => true,
                _ => false,
            }
            && self.color == other.color
    }
}

impl CssShadow {
    /// Retains authored omissions and rejects a spread without a blur radius.
    pub fn try_new(
        inset: bool,
        offset_x: CssSpecifiedLength,
        offset_y: CssSpecifiedLength,
        blur_radius: Option<CssSpecifiedNonNegativeLength>,
        spread_radius: Option<CssSpecifiedLength>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        if blur_radius.is_none() && spread_radius.is_some() {
            return None;
        }
        Some(Self::new(
            inset,
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            color,
        ))
    }
    pub(crate) fn new(
        inset: bool,
        offset_x: CssSpecifiedLength,
        offset_y: CssSpecifiedLength,
        blur_radius: Option<CssSpecifiedNonNegativeLength>,
        spread_radius: Option<CssSpecifiedLength>,
        color: Option<CssColor>,
    ) -> Self {
        Self {
            inset,
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            color: color.map(Box::new),
        }
    }
    pub const fn inset(&self) -> bool {
        self.inset
    }
    pub const fn offset_x(&self) -> &CssSpecifiedLength {
        &self.offset_x
    }
    pub const fn offset_y(&self) -> &CssSpecifiedLength {
        &self.offset_y
    }
    pub const fn blur_radius(&self) -> Option<&CssSpecifiedNonNegativeLength> {
        self.blur_radius.as_ref()
    }
    pub const fn spread_radius(&self) -> Option<&CssSpecifiedLength> {
        self.spread_radius.as_ref()
    }
    pub fn color(&self) -> Option<&CssColor> {
        self.color.as_deref()
    }
}

/// The two authored function identities in the selected Values 4 `<url>` grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssUrlFunction {
    Url,
    Src,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssUrl {
    function: CssUrlFunction,
    value: String,
    modifiers: Vec<CssUrlModifier>,
}

impl CssUrl {
    /// Preserves a decoded authored URL, including empty or whitespace values.
    /// Resource resolution and usability belong to downstream consumers.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self::from_parts(CssUrlFunction::Url, value, Vec::new())
    }

    /// Constructs a decoded authored URL with its function identity and ordered modifiers.
    /// Resource resolution and modifier interpretation belong downstream.
    #[must_use]
    pub fn from_parts(
        function: CssUrlFunction,
        value: impl Into<String>,
        modifiers: Vec<CssUrlModifier>,
    ) -> Self {
        Self {
            function,
            value: value.into(),
            modifiers,
        }
    }

    #[must_use]
    pub const fn function(&self) -> CssUrlFunction {
        self.function
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Whether this authored URL has the Values 4 local URL flag.
    /// A fragment-only target remains symbolic; tree-scoped resolution is downstream.
    #[must_use]
    pub fn is_local_url(&self) -> bool {
        self.value.starts_with('#')
    }

    /// Returns the authored modifiers of a quoted `url()` or `src()` value.
    ///
    /// Values and Units Level 3 defines modifier syntax without assigning
    /// resource semantics to any particular modifier. This accessor therefore
    /// preserves only the typed authored identifier/function forms; it does not
    /// interpret a modifier, resolve the URL, or load a resource.
    #[must_use]
    pub fn modifiers(&self) -> &[CssUrlModifier] {
        &self.modifiers
    }
}

/// One decoded CSS identifier retained as authored value syntax.
///
/// Parsed and checked construction do not apply consumer-specific keyword exclusions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssIdent {
    value: String,
}

impl CssIdent {
    /// Constructs a representable decoded CSS identifier, retaining its case and punctuation.
    /// Consumer-specific keyword exclusions are intentionally not applied.
    pub fn try_new(value: impl Into<String>) -> Result<Self, crate::CssComponentValueError> {
        let value = value.into();
        crate::CssComponentValue::try_ident(value.clone())?;
        Ok(Self { value })
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Returns the decoded, case-preserving identifier value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// One authored Values and Units Level 3 URL modifier.
///
/// The source defines the syntax as either an identifier or functional
/// notation, while assigning no modifier meanings. These values therefore stay
/// symbolic and do not trigger URL resolution or resource loading.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssUrlModifier {
    Ident(CssIdent),
    Function(CssUrlModifierFunction),
}

/// One checked functional URL modifier with retained authored argument tokens.
/// Equality remains based on its decoded name and authored argument text,
/// independent of the retained component origins.
#[derive(Clone)]
pub struct CssUrlModifierFunction {
    name: CssIdent,
    arguments: CssAuthoredFunctionArguments,
    argument_components: crate::CssComponentValues,
}

impl CssUrlModifierFunction {
    /// Constructs a modifier only when its name and argument token stream form a CSS function.
    pub fn try_new(
        name: CssIdent,
        arguments: crate::CssComponentValues,
    ) -> Result<Self, crate::CssComponentValueError> {
        crate::CssComponentValue::try_function(name.as_str(), arguments.clone())?;
        let css = arguments
            .serialize_with_limit(crate::CssComponentValueLimits::default().max_css_bytes())?
            .as_css()
            .to_owned();
        Ok(Self {
            name,
            arguments: CssAuthoredFunctionArguments::new(css),
            argument_components: arguments,
        })
    }

    #[must_use]
    pub(crate) const fn from_parsed(
        name: CssIdent,
        arguments: CssAuthoredFunctionArguments,
        argument_components: crate::CssComponentValues,
    ) -> Self {
        Self {
            name,
            arguments,
            argument_components,
        }
    }

    /// Returns the decoded, case-preserving function name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the preserved authored function arguments.
    #[must_use]
    pub const fn arguments(&self) -> &CssAuthoredFunctionArguments {
        &self.arguments
    }

    /// Returns the original immutable argument tokens and their source origins.
    #[must_use]
    pub const fn argument_components(&self) -> &crate::CssComponentValues {
        &self.argument_components
    }
}

impl PartialEq for CssUrlModifierFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments == other.arguments
    }
}

impl Eq for CssUrlModifierFunction {}

impl std::fmt::Debug for CssUrlModifierFunction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssUrlModifierFunction")
            .field("name", &self.name)
            .field("arguments", &self.arguments)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAuthoredFunctionArguments {
    css: String,
}

impl CssAuthoredFunctionArguments {
    #[must_use]
    pub(crate) fn new(css: impl Into<String>) -> Self {
        Self { css: css.into() }
    }

    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.css
    }
}

/// An authored image value accepted by Images 3 consumers.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssImageValue {
    None,
    Url(CssUrl),
    Gradient(CssGradient),
    /// Two authored image/none branches, retaining their order and scheme dependency.
    LightDark(Box<CssLightDarkImage>),
}

/// One non-negative authored `border-image-slice` component.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBorderImageSliceComponent {
    Number(CssSpecifiedNonNegativeNumber),
    /// A Number result retaining a percentage basis; math range handling is deferred.
    HintedNumberCalculation(CssHintedNumberCalculation),
    Percentage(CssSpecifiedNonNegativePercentage),
}

impl PartialEq for CssBorderImageSliceComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left.structural_eq(right),
            (Self::HintedNumberCalculation(left), Self::HintedNumberCalculation(right)) => {
                left.structural_eq(right)
            }
            (Self::Percentage(left), Self::Percentage(right)) => left == right,
            _ => false,
        }
    }
}

/// The expanded four-edge authored `border-image-slice` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImageSlice {
    values: [CssBorderImageSliceComponent; 4],
    fill: bool,
}

impl CssBorderImageSlice {
    #[must_use]
    pub fn try_new(values: Vec<CssBorderImageSliceComponent>, fill: bool) -> Option<Self> {
        expand_border_image_sides(values).map(|values| Self { values, fill })
    }

    #[must_use]
    pub const fn values(&self) -> &[CssBorderImageSliceComponent; 4] {
        &self.values
    }

    #[must_use]
    pub const fn fill(&self) -> bool {
        self.fill
    }
}

/// One authored `border-image-width` component.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBorderImageWidthComponent {
    Auto,
    LengthPercentage(CssSpecifiedNonNegativeLengthPercentage),
    Number(CssSpecifiedNonNegativeNumber),
    /// A contextual Number multiplier, before computed-value range handling.
    HintedNumberCalculation(CssHintedNumberCalculation),
}

impl PartialEq for CssBorderImageWidthComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
            (Self::Number(left), Self::Number(right)) => left.structural_eq(right),
            (Self::HintedNumberCalculation(left), Self::HintedNumberCalculation(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}

/// The expanded four-edge authored `border-image-width` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImageWidth {
    values: [CssBorderImageWidthComponent; 4],
}

impl CssBorderImageWidth {
    #[must_use]
    pub fn try_new(values: Vec<CssBorderImageWidthComponent>) -> Option<Self> {
        expand_border_image_sides(values).map(|values| Self { values })
    }

    #[must_use]
    pub const fn values(&self) -> &[CssBorderImageWidthComponent; 4] {
        &self.values
    }
}

/// One authored `border-image-outset` component.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBorderImageOutsetComponent {
    Length(CssSpecifiedNonNegativeLength),
    Number(CssSpecifiedNonNegativeNumber),
}

impl PartialEq for CssBorderImageOutsetComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            (Self::Number(left), Self::Number(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

/// The expanded four-edge authored `border-image-outset` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImageOutset {
    values: [CssBorderImageOutsetComponent; 4],
}

impl CssBorderImageOutset {
    #[must_use]
    pub fn try_new(values: Vec<CssBorderImageOutsetComponent>) -> Option<Self> {
        expand_border_image_sides(values).map(|values| Self { values })
    }

    #[must_use]
    pub const fn values(&self) -> &[CssBorderImageOutsetComponent; 4] {
        &self.values
    }
}

fn expand_border_image_sides<T: Clone>(values: Vec<T>) -> Option<[T; 4]> {
    match values.as_slice() {
        [all] => Some([all.clone(), all.clone(), all.clone(), all.clone()]),
        [vertical, horizontal] => Some([
            vertical.clone(),
            horizontal.clone(),
            vertical.clone(),
            horizontal.clone(),
        ]),
        [top, horizontal, bottom] => Some([
            top.clone(),
            horizontal.clone(),
            bottom.clone(),
            horizontal.clone(),
        ]),
        [top, right, bottom, left] => {
            Some([top.clone(), right.clone(), bottom.clone(), left.clone()])
        }
        [] | [_, _, _, _, ..] => None,
    }
}

/// One axis repetition keyword in `border-image-repeat`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBorderImageRepeatKeyword {
    Stretch,
    Repeat,
    Round,
    Space,
}

/// The authored horizontal and vertical border-image repetition modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssBorderImageRepeat {
    horizontal: CssBorderImageRepeatKeyword,
    vertical: CssBorderImageRepeatKeyword,
}

impl CssBorderImageRepeat {
    #[must_use]
    pub const fn new(
        horizontal: CssBorderImageRepeatKeyword,
        vertical: CssBorderImageRepeatKeyword,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    #[must_use]
    pub const fn horizontal(self) -> CssBorderImageRepeatKeyword {
        self.horizontal
    }

    #[must_use]
    pub const fn vertical(self) -> CssBorderImageRepeatKeyword {
        self.vertical
    }
}

/// An authored `border-image` shorthand preserving only explicitly supplied components.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImage {
    source: Option<CssImageValue>,
    slice: Option<CssBorderImageSlice>,
    width: Option<CssBorderImageWidth>,
    outset: Option<CssBorderImageOutset>,
    repeat: Option<CssBorderImageRepeat>,
}

impl CssBorderImage {
    #[must_use]
    pub fn try_new(
        source: Option<CssImageValue>,
        slice: Option<CssBorderImageSlice>,
        width: Option<CssBorderImageWidth>,
        outset: Option<CssBorderImageOutset>,
        repeat: Option<CssBorderImageRepeat>,
    ) -> Option<Self> {
        if source.is_none() && slice.is_none() && repeat.is_none()
            || slice.is_none() && (width.is_some() || outset.is_some())
        {
            return None;
        }
        Some(Self {
            source,
            slice,
            width,
            outset,
            repeat,
        })
    }

    #[must_use]
    pub const fn source(&self) -> Option<&CssImageValue> {
        self.source.as_ref()
    }

    #[must_use]
    pub const fn slice(&self) -> Option<&CssBorderImageSlice> {
        self.slice.as_ref()
    }

    #[must_use]
    pub const fn width(&self) -> Option<&CssBorderImageWidth> {
        self.width.as_ref()
    }

    #[must_use]
    pub const fn outset(&self) -> Option<&CssBorderImageOutset> {
        self.outset.as_ref()
    }

    #[must_use]
    pub const fn repeat(&self) -> Option<CssBorderImageRepeat> {
        self.repeat
    }
}

/// The authored Images 3 `image-orientation` syntax.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssImageOrientation {
    FromImage,
    None,
    Angle(CssAngleValue),
    Flip(Option<CssAngleValue>),
}

impl PartialEq for CssImageOrientation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::FromImage, Self::FromImage) => true,
            (Self::None, Self::None) => true,
            (Self::Angle(left), Self::Angle(right)) => left.structural_eq(right),
            (Self::Flip(left), Self::Flip(right)) => {
                optional_numeric_eq(left.as_ref(), right.as_ref(), CssAngleValue::structural_eq)
            }
            _ => false,
        }
    }
}

/// The authored Images 3 `image-rendering` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImageRendering {
    Auto,
    Smooth,
    HighQuality,
    CrispEdges,
    Pixelated,
    /// Deprecated CSS-standard syntax accepted by user agents; authors must not use it.
    /// Its authored identity stays distinct from the downstream crisp-edges behavior.
    OptimizeSpeed,
    /// Deprecated CSS-standard syntax accepted by user agents; authors must not use it.
    /// Its authored identity stays distinct from the downstream smooth behavior.
    OptimizeQuality,
}

/// The authored Images 3 `object-fit` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssObjectFit {
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

/// A nonempty authored comma-separated image list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssImageValueList {
    images: Vec<CssImageValue>,
}

impl CssImageValueList {
    #[must_use]
    pub fn try_new(images: Vec<CssImageValue>) -> Option<Self> {
        (!images.is_empty()).then_some(Self { images })
    }

    #[must_use]
    pub fn images(&self) -> &[CssImageValue] {
        &self.images
    }
}

/// One of the four authored Images 3 gradient functions.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGradient {
    Linear(CssLinearGradient),
    Radial(CssRadialGradient),
    RepeatingLinear(CssLinearGradient),
    RepeatingRadial(CssRadialGradient),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHorizontalGradientSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVerticalGradientSide {
    Top,
    Bottom,
}

/// A checked authored linear-gradient side or corner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssSideOrCorner {
    horizontal: Option<CssHorizontalGradientSide>,
    vertical: Option<CssVerticalGradientSide>,
}

impl CssSideOrCorner {
    #[must_use]
    pub const fn try_new(
        horizontal: Option<CssHorizontalGradientSide>,
        vertical: Option<CssVerticalGradientSide>,
    ) -> Option<Self> {
        if horizontal.is_none() && vertical.is_none() {
            None
        } else {
            Some(Self {
                horizontal,
                vertical,
            })
        }
    }

    #[must_use]
    pub const fn horizontal(self) -> Option<CssHorizontalGradientSide> {
        self.horizontal
    }

    #[must_use]
    pub const fn vertical(self) -> Option<CssVerticalGradientSide> {
        self.vertical
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssLinearGradientDirection {
    Angle(CssAngleOrZero),
    SideOrCorner(CssSideOrCorner),
}

impl PartialEq for CssLinearGradientDirection {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Angle(left), Self::Angle(right)) => left.structural_eq(right),
            (Self::SideOrCorner(left), Self::SideOrCorner(right)) => left == right,
            _ => false,
        }
    }
}

/// One authored color stop in a gradient color-stop list.
#[derive(Clone, Debug)]
pub struct CssGradientColorStop {
    color: CssColor,
    position: Option<CssSpecifiedLengthPercentage>,
}
numeric_fields_eq!(CssGradientColorStop, [], [position], [color]);

impl CssGradientColorStop {
    #[must_use]
    pub const fn color(&self) -> &CssColor {
        &self.color
    }

    #[must_use]
    pub const fn position(&self) -> Option<&CssSpecifiedLengthPercentage> {
        self.position.as_ref()
    }
}

/// One authored item in a color-stop list, preserving stop and hint order.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssColorStopListItem {
    Stop(Box<CssGradientColorStop>),
    Hint(CssSpecifiedLengthPercentage),
}

impl PartialEq for CssColorStopListItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Stop(left), Self::Stop(right)) => left == right,
            (Self::Hint(left), Self::Hint(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

/// A checked authored color-stop list with two or more stops and interleaved hints.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorStopList {
    items: Vec<CssColorStopListItem>,
}

impl CssColorStopList {
    #[must_use]
    pub fn try_new(items: Vec<CssColorStopListItem>) -> Option<Self> {
        let stop_count = items
            .iter()
            .filter(|item| matches!(item, CssColorStopListItem::Stop(_)))
            .count();
        let ordered = matches!(items.first(), Some(CssColorStopListItem::Stop(_)))
            && matches!(items.last(), Some(CssColorStopListItem::Stop(_)))
            && items.windows(2).all(|pair| {
                !matches!(
                    pair,
                    [CssColorStopListItem::Hint(_), CssColorStopListItem::Hint(_)]
                )
            });
        (stop_count >= 2 && ordered).then_some(Self { items })
    }

    #[must_use]
    pub fn items(&self) -> &[CssColorStopListItem] {
        &self.items
    }
}

/// An authored linear or repeating-linear gradient payload.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLinearGradient {
    direction: Option<CssLinearGradientDirection>,
    stops: CssColorStopList,
}

impl CssLinearGradient {
    #[must_use]
    pub const fn direction(&self) -> Option<&CssLinearGradientDirection> {
        self.direction.as_ref()
    }

    #[must_use]
    pub const fn stops(&self) -> &CssColorStopList {
        &self.stops
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRadialShape {
    Circle,
    Ellipse,
}

/// A checked pair of non-negative radial-gradient ellipse radii.
#[derive(Clone, Debug)]
pub struct CssRadialEllipseSize {
    horizontal: CssSpecifiedNonNegativeLengthPercentage,
    vertical: CssSpecifiedNonNegativeLengthPercentage,
}
numeric_fields_eq!(CssRadialEllipseSize, [horizontal, vertical], [], []);

impl CssRadialEllipseSize {
    /// Composes two independently checked nonnegative radii in horizontal/vertical order.
    pub const fn new(
        horizontal: CssSpecifiedNonNegativeLengthPercentage,
        vertical: CssSpecifiedNonNegativeLengthPercentage,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
    pub const fn horizontal(&self) -> &CssSpecifiedNonNegativeLengthPercentage {
        &self.horizontal
    }
    pub const fn vertical(&self) -> &CssSpecifiedNonNegativeLengthPercentage {
        &self.vertical
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssRadialSize {
    Extent(CssRadialExtent),
    Circle(CssSpecifiedNonNegativeLength),
    Ellipse(CssRadialEllipseSize),
}

impl PartialEq for CssRadialSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Extent(left), Self::Extent(right)) => left == right,
            (Self::Circle(left), Self::Circle(right)) => left.structural_eq(right),
            (Self::Ellipse(left), Self::Ellipse(right)) => left == right,
            _ => false,
        }
    }
}

/// A current authored radial or repeating-radial gradient payload.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRadialGradient {
    shape: Option<CssRadialShape>,
    size: Option<CssRadialSize>,
    position: Option<CssPhysicalPosition>,
    stops: CssColorStopList,
}

impl CssRadialGradient {
    #[must_use]
    pub const fn shape(&self) -> Option<CssRadialShape> {
        self.shape
    }

    #[must_use]
    pub const fn size(&self) -> Option<&CssRadialSize> {
        self.size.as_ref()
    }

    #[must_use]
    pub const fn position(&self) -> Option<&CssPhysicalPosition> {
        self.position.as_ref()
    }

    #[must_use]
    pub const fn stops(&self) -> &CssColorStopList {
        &self.stops
    }
}

mod position;
pub use position::*;

mod images;
pub use images::{CssImage, CssImageConstructionError, CssLightDarkImage};

/// The exact authored background box component count for one shorthand layer.
///
/// One box supplies both origin and clip. Two boxes preserve their authored
/// origin-then-clip order without resolving either box against layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundLayerBoxes {
    One(CssBackgroundBox),
    OriginAndClip {
        origin: CssBackgroundBox,
        clip: CssBackgroundBox,
    },
}

impl CssBackgroundLayerBoxes {
    /// Returns the shorthand layer's background-origin component.
    #[must_use]
    pub const fn origin(self) -> CssBackgroundBox {
        match self {
            Self::One(value) | Self::OriginAndClip { origin: value, .. } => value,
        }
    }

    /// Returns the shorthand layer's background-clip component.
    #[must_use]
    pub const fn clip(self) -> CssBackgroundBox {
        match self {
            Self::One(value) | Self::OriginAndClip { clip: value, .. } => value,
        }
    }
}

/// One checked authored `background` shorthand layer.
///
/// Checked construction retains omissions and requires a position with a size.
/// The enclosing background additionally restricts colors to its final layer.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundLayer {
    image: Option<CssImageValue>,
    position: Option<CssBackgroundPosition>,
    size: Option<CssBackgroundSize>,
    repeat: Option<CssBackgroundRepeat>,
    attachment: Option<CssBackgroundAttachment>,
    boxes: Option<CssBackgroundLayerBoxes>,
    color: Option<CssColor>,
}

impl CssBackgroundLayer {
    /// Constructs one nonempty authored layer without filling omitted components.
    /// A size requires a position; final-layer color validity belongs to [`CssBackground`].
    pub fn try_new(
        image: Option<CssImageValue>,
        position: Option<CssBackgroundPosition>,
        size: Option<CssBackgroundSize>,
        repeat: Option<CssBackgroundRepeat>,
        attachment: Option<CssBackgroundAttachment>,
        boxes: Option<CssBackgroundLayerBoxes>,
        color: Option<CssColor>,
    ) -> Result<Self, CssBackgroundConstructionError> {
        if size.is_some() && position.is_none() {
            return Err(CssBackgroundConstructionError::SizeWithoutPosition);
        }
        if image.is_none()
            && position.is_none()
            && size.is_none()
            && repeat.is_none()
            && attachment.is_none()
            && boxes.is_none()
            && color.is_none()
        {
            return Err(CssBackgroundConstructionError::EmptyLayer);
        }
        Ok(Self {
            image,
            position,
            size,
            repeat,
            attachment,
            boxes,
            color,
        })
    }

    /// Returns this layer's authored image, if present.
    #[must_use]
    pub const fn image(&self) -> Option<&CssImageValue> {
        self.image.as_ref()
    }

    /// Returns this layer's authored position, if present.
    #[must_use]
    pub const fn position(&self) -> Option<&CssBackgroundPosition> {
        self.position.as_ref()
    }

    /// Returns the size coupled to this layer's position, if authored.
    #[must_use]
    pub const fn size(&self) -> Option<&CssBackgroundSize> {
        self.size.as_ref()
    }

    /// Returns this layer's authored repeat style, if present.
    #[must_use]
    pub const fn repeat(&self) -> Option<CssBackgroundRepeat> {
        self.repeat
    }

    /// Returns this layer's authored attachment, if present.
    #[must_use]
    pub const fn attachment(&self) -> Option<CssBackgroundAttachment> {
        self.attachment
    }

    /// Returns the exact one- or two-box shorthand component, if present.
    #[must_use]
    pub const fn boxes(&self) -> Option<CssBackgroundLayerBoxes> {
        self.boxes
    }

    /// Returns the authored color on the final layer, if present.
    #[must_use]
    pub const fn color(&self) -> Option<&CssColor> {
        self.color.as_ref()
    }
}

/// Why checked authored background construction cannot represent a valid shorthand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundConstructionError {
    EmptyLayer,
    SizeWithoutPosition,
    EmptyLayers,
    NonFinalColor { layer_index: usize },
}

impl std::fmt::Display for CssBackgroundConstructionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLayer => formatter.write_str("background layer is empty"),
            Self::SizeWithoutPosition => formatter.write_str("background size requires a position"),
            Self::EmptyLayers => formatter.write_str("background layer list is empty"),
            Self::NonFinalColor { layer_index } => write!(
                formatter,
                "background color in nonfinal layer {layer_index}"
            ),
        }
    }
}

impl std::error::Error for CssBackgroundConstructionError {}

/// A nonempty authored comma list of `background` shorthand layers.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackground {
    layers: Vec<CssBackgroundLayer>,
}

impl CssBackground {
    /// Constructs a nonempty authored list with a color only on its final layer.
    pub fn try_new(
        layers: Vec<CssBackgroundLayer>,
    ) -> Result<Self, CssBackgroundConstructionError> {
        if layers.is_empty() {
            return Err(CssBackgroundConstructionError::EmptyLayers);
        }
        for (layer_index, layer) in layers[..layers.len() - 1].iter().enumerate() {
            if layer.color().is_some() {
                return Err(CssBackgroundConstructionError::NonFinalColor { layer_index });
            }
        }
        Ok(Self { layers })
    }

    /// Returns the authored layers in comma order.
    #[must_use]
    pub fn layers(&self) -> &[CssBackgroundLayer] {
        &self.layers
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBackgroundSizeComponent {
    Auto,
    Length(CssSpecifiedNonNegativeLengthPercentage),
}

impl PartialEq for CssBackgroundSizeComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) => true,
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundSize {
    Cover,
    Contain,
    Explicit {
        width: CssBackgroundSizeComponent,
        height: Option<CssBackgroundSizeComponent>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundSizeList {
    sizes: Vec<CssBackgroundSize>,
}

impl CssBackgroundSizeList {
    #[must_use]
    pub fn try_new(sizes: Vec<CssBackgroundSize>) -> Option<Self> {
        if sizes.is_empty() {
            None
        } else {
            Some(Self::new(sizes))
        }
    }

    #[must_use]
    pub(crate) fn new(sizes: Vec<CssBackgroundSize>) -> Self {
        Self { sizes }
    }

    #[must_use]
    pub fn sizes(&self) -> &[CssBackgroundSize] {
        &self.sizes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundRepeatStyle {
    Repeat,
    Space,
    Round,
    NoRepeat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundRepeat {
    RepeatX,
    RepeatY,
    Axes {
        x: CssBackgroundRepeatStyle,
        y: CssBackgroundRepeatStyle,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBackgroundRepeatList {
    repeats: Vec<CssBackgroundRepeat>,
}

impl CssBackgroundRepeatList {
    #[must_use]
    pub fn try_new(repeats: Vec<CssBackgroundRepeat>) -> Option<Self> {
        if repeats.is_empty() {
            None
        } else {
            Some(Self::new(repeats))
        }
    }

    #[must_use]
    pub(crate) fn new(repeats: Vec<CssBackgroundRepeat>) -> Self {
        Self { repeats }
    }

    #[must_use]
    pub fn repeats(&self) -> &[CssBackgroundRepeat] {
        &self.repeats
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundBox {
    BorderBox,
    PaddingBox,
    ContentBox,
}

/// A nonempty authored comma list of background box longhand values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBackgroundBoxList {
    boxes: Vec<CssBackgroundBox>,
}

impl CssBackgroundBoxList {
    /// Constructs a nonempty background box list.
    #[must_use]
    pub fn try_new(boxes: Vec<CssBackgroundBox>) -> Option<Self> {
        (!boxes.is_empty()).then_some(Self { boxes })
    }

    /// Returns the authored boxes in comma order.
    #[must_use]
    pub fn boxes(&self) -> &[CssBackgroundBox] {
        &self.boxes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundAttachment {
    Scroll,
    Fixed,
    Local,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBackgroundAttachmentList {
    attachments: Vec<CssBackgroundAttachment>,
}

impl CssBackgroundAttachmentList {
    #[must_use]
    pub fn try_new(attachments: Vec<CssBackgroundAttachment>) -> Option<Self> {
        if attachments.is_empty() {
            None
        } else {
            Some(Self::new(attachments))
        }
    }

    #[must_use]
    pub(crate) fn new(attachments: Vec<CssBackgroundAttachment>) -> Self {
        Self { attachments }
    }

    #[must_use]
    pub fn attachments(&self) -> &[CssBackgroundAttachment] {
        &self.attachments
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursorKeyword {
    Auto,
    Default,
    None,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    AllScroll,
    ColResize,
    RowResize,
    NResize,
    EResize,
    SResize,
    WResize,
    NeResize,
    NwResize,
    SeResize,
    SwResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ZoomIn,
    ZoomOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPointerEvents {
    Auto,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssUserSelect {
    Auto,
    Text,
    None,
    All,
    Contain,
}

/// Authored UI4 outline styles. Hidden is excluded from this semantic domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOutlineStyle {
    Auto,
    None,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

/// The represented authored UI4 outline-color domain: automatic or shared color.
/// The inherited image-1D branch is not represented. Automatic color stays
/// symbolic; accent-color and currentColor computation belong downstream.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum CssOutlineColor {
    #[default]
    Auto,
    Color(Box<CssColor>),
}

impl CssOutlineColor {
    /// Borrows the shared authored color, when this is not the automatic branch.
    #[must_use]
    pub fn color(&self) -> Option<&CssColor> {
        match self {
            Self::Auto => None,
            Self::Color(color) => Some(color),
        }
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssOutlineWidth {
    Thin,
    Medium,
    Thick,
    Length(CssSpecifiedNonNegativeLength),
}

impl PartialEq for CssOutlineWidth {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            (Self::Thin, Self::Thin)
            | (Self::Medium, Self::Medium)
            | (Self::Thick, Self::Thick) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssOutline {
    width: Option<CssOutlineWidth>,
    style: Option<CssOutlineStyle>,
    color: Option<CssOutlineColor>,
}

impl CssOutline {
    #[must_use]
    pub fn try_new(
        width: Option<CssOutlineWidth>,
        style: Option<CssOutlineStyle>,
        color: Option<CssOutlineColor>,
    ) -> Option<Self> {
        if width.is_none() && style.is_none() && color.is_none() {
            None
        } else {
            Some(Self::new(width, style, color))
        }
    }

    pub(crate) fn new(
        width: Option<CssOutlineWidth>,
        style: Option<CssOutlineStyle>,
        color: Option<CssOutlineColor>,
    ) -> Self {
        Self {
            width,
            style,
            color,
        }
    }

    #[must_use]
    pub const fn width(&self) -> Option<&CssOutlineWidth> {
        self.width.as_ref()
    }

    #[must_use]
    pub const fn style(&self) -> Option<CssOutlineStyle> {
        self.style
    }

    #[must_use]
    pub const fn color(&self) -> Option<&CssOutlineColor> {
        self.color.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTransformFunctionKind {
    Matrix,
    Matrix3d,
    Perspective,
    Rotate,
    Rotate3d,
    RotateX,
    RotateY,
    RotateZ,
    Scale,
    Scale3d,
    ScaleX,
    ScaleY,
    ScaleZ,
    Skew,
    SkewX,
    SkewY,
    Translate,
    Translate3d,
    TranslateX,
    TranslateY,
    TranslateZ,
}

/// One authored operand in a transform scale function.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTransformScaleComponent {
    Number(CssSpecifiedNumber),
    /// A Number result whose dimensional percentage basis remains unresolved.
    HintedNumberCalculation(CssHintedNumberCalculation),
    Percentage(CssSpecifiedPercentage),
}

#[derive(Clone, Debug)]
pub struct CssTransformMatrix {
    components: [CssSpecifiedNumber; 6],
}

impl PartialEq for CssTransformScaleComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left.structural_eq(right),
            (Self::HintedNumberCalculation(left), Self::HintedNumberCalculation(right)) => {
                left.structural_eq(right)
            }
            (Self::Percentage(left), Self::Percentage(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

impl PartialEq for CssTransformMatrix {
    fn eq(&self, other: &Self) -> bool {
        self.components
            .iter()
            .zip(&other.components)
            .all(|(left, right)| left.structural_eq(right))
    }
}

impl CssTransformMatrix {
    pub const fn new(components: [CssSpecifiedNumber; 6]) -> Self {
        Self { components }
    }

    #[must_use]
    pub const fn components(&self) -> &[CssSpecifiedNumber; 6] {
        &self.components
    }
}

#[derive(Clone, Debug)]
pub struct CssTransformMatrix3d {
    components: [CssSpecifiedNumber; 16],
}

impl PartialEq for CssTransformMatrix3d {
    fn eq(&self, other: &Self) -> bool {
        self.components
            .iter()
            .zip(&other.components)
            .all(|(left, right)| left.structural_eq(right))
    }
}

impl CssTransformMatrix3d {
    pub const fn new(components: [CssSpecifiedNumber; 16]) -> Self {
        Self { components }
    }

    #[must_use]
    pub const fn components(&self) -> &[CssSpecifiedNumber; 16] {
        &self.components
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTransformPerspective {
    None,
    Length(CssSpecifiedNonNegativeLength),
}

impl PartialEq for CssTransformPerspective {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CssTransformRotate3d {
    x: CssSpecifiedNumber,
    y: CssSpecifiedNumber,
    z: CssSpecifiedNumber,
    angle: CssAngleOrZero,
}

numeric_fields_eq!(CssTransformRotate3d, [x, y, z, angle], [], []);

impl CssTransformRotate3d {
    pub const fn new(
        x: CssSpecifiedNumber,
        y: CssSpecifiedNumber,
        z: CssSpecifiedNumber,
        angle: CssAngleOrZero,
    ) -> Self {
        Self { x, y, z, angle }
    }

    #[must_use]
    pub const fn x(&self) -> &CssSpecifiedNumber {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &CssSpecifiedNumber {
        &self.y
    }

    #[must_use]
    pub const fn z(&self) -> &CssSpecifiedNumber {
        &self.z
    }

    #[must_use]
    pub const fn angle(&self) -> &CssAngleOrZero {
        &self.angle
    }
}

#[derive(Clone, Debug)]
pub struct CssTransformScale {
    x: CssTransformScaleComponent,
    y: Option<CssTransformScaleComponent>,
}

impl PartialEq for CssTransformScale {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl CssTransformScale {
    pub const fn new(x: CssTransformScaleComponent, y: Option<CssTransformScaleComponent>) -> Self {
        Self { x, y }
    }
    #[must_use]
    pub const fn x(&self) -> &CssTransformScaleComponent {
        &self.x
    }
    #[must_use]
    pub const fn y(&self) -> Option<&CssTransformScaleComponent> {
        self.y.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformScale3d {
    x: CssTransformScaleComponent,
    y: CssTransformScaleComponent,
    z: CssTransformScaleComponent,
}

impl CssTransformScale3d {
    pub const fn new(
        x: CssTransformScaleComponent,
        y: CssTransformScaleComponent,
        z: CssTransformScaleComponent,
    ) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn x(&self) -> &CssTransformScaleComponent {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &CssTransformScaleComponent {
        &self.y
    }

    #[must_use]
    pub const fn z(&self) -> &CssTransformScaleComponent {
        &self.z
    }
}

#[derive(Clone, Debug)]
pub struct CssTransformSkew {
    x: CssAngleOrZero,
    y: Option<CssAngleOrZero>,
}

numeric_fields_eq!(CssTransformSkew, [x], [y], []);

impl CssTransformSkew {
    pub const fn new(x: CssAngleOrZero, y: Option<CssAngleOrZero>) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> &CssAngleOrZero {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> Option<&CssAngleOrZero> {
        self.y.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct CssTransformTranslate {
    x: CssSpecifiedLengthPercentage,
    y: Option<CssSpecifiedLengthPercentage>,
}
numeric_fields_eq!(CssTransformTranslate, [x], [y], []);

impl CssTransformTranslate {
    pub const fn new(
        x: CssSpecifiedLengthPercentage,
        y: Option<CssSpecifiedLengthPercentage>,
    ) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> &CssSpecifiedLengthPercentage {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> Option<&CssSpecifiedLengthPercentage> {
        self.y.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct CssTransformTranslate3d {
    x: CssSpecifiedLengthPercentage,
    y: CssSpecifiedLengthPercentage,
    z: CssSpecifiedLength,
}
numeric_fields_eq!(CssTransformTranslate3d, [x, y, z], [], []);

impl CssTransformTranslate3d {
    pub const fn new(
        x: CssSpecifiedLengthPercentage,
        y: CssSpecifiedLengthPercentage,
        z: CssSpecifiedLength,
    ) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn x(&self) -> &CssSpecifiedLengthPercentage {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &CssSpecifiedLengthPercentage {
        &self.y
    }

    #[must_use]
    pub const fn z(&self) -> &CssSpecifiedLength {
        &self.z
    }
}

/// An authored transform function with an exact typed payload.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTransformFunction {
    Matrix(CssTransformMatrix),
    Matrix3d(Box<CssTransformMatrix3d>),
    Perspective(CssTransformPerspective),
    Rotate(CssAngleOrZero),
    Rotate3d(CssTransformRotate3d),
    RotateX(CssAngleOrZero),
    RotateY(CssAngleOrZero),
    RotateZ(CssAngleOrZero),
    Scale(CssTransformScale),
    Scale3d(CssTransformScale3d),
    ScaleX(CssTransformScaleComponent),
    ScaleY(CssTransformScaleComponent),
    ScaleZ(CssTransformScaleComponent),
    Skew(CssTransformSkew),
    SkewX(CssAngleOrZero),
    SkewY(CssAngleOrZero),
    Translate(CssTransformTranslate),
    Translate3d(CssTransformTranslate3d),
    TranslateX(CssSpecifiedLengthPercentage),
    TranslateY(CssSpecifiedLengthPercentage),
    TranslateZ(CssSpecifiedLength),
}

impl PartialEq for CssTransformFunction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TranslateX(left), Self::TranslateX(right))
            | (Self::TranslateY(left), Self::TranslateY(right)) => left.structural_eq(right),
            (Self::TranslateZ(left), Self::TranslateZ(right)) => left.structural_eq(right),
            (Self::Matrix(left), Self::Matrix(right)) => left == right,
            (Self::Matrix3d(left), Self::Matrix3d(right)) => left == right,
            (Self::Perspective(left), Self::Perspective(right)) => left == right,
            (Self::Rotate(left), Self::Rotate(right))
            | (Self::RotateX(left), Self::RotateX(right))
            | (Self::RotateY(left), Self::RotateY(right))
            | (Self::RotateZ(left), Self::RotateZ(right)) => left.structural_eq(right),
            (Self::Rotate3d(left), Self::Rotate3d(right)) => left == right,
            (Self::Scale(left), Self::Scale(right)) => left == right,
            (Self::Scale3d(left), Self::Scale3d(right)) => left == right,
            (Self::ScaleX(left), Self::ScaleX(right))
            | (Self::ScaleY(left), Self::ScaleY(right)) => left == right,
            (Self::ScaleZ(left), Self::ScaleZ(right)) => left == right,
            (Self::Skew(left), Self::Skew(right)) => left == right,
            (Self::SkewX(left), Self::SkewX(right)) | (Self::SkewY(left), Self::SkewY(right)) => {
                left.structural_eq(right)
            }
            (Self::Translate(left), Self::Translate(right)) => left == right,
            (Self::Translate3d(left), Self::Translate3d(right)) => left == right,
            _ => false,
        }
    }
}

impl CssTransformFunction {
    #[must_use]
    pub const fn kind(&self) -> CssTransformFunctionKind {
        match self {
            Self::Matrix(_) => CssTransformFunctionKind::Matrix,
            Self::Matrix3d(_) => CssTransformFunctionKind::Matrix3d,
            Self::Perspective(_) => CssTransformFunctionKind::Perspective,
            Self::Rotate(_) => CssTransformFunctionKind::Rotate,
            Self::Rotate3d(_) => CssTransformFunctionKind::Rotate3d,
            Self::RotateX(_) => CssTransformFunctionKind::RotateX,
            Self::RotateY(_) => CssTransformFunctionKind::RotateY,
            Self::RotateZ(_) => CssTransformFunctionKind::RotateZ,
            Self::Scale(_) => CssTransformFunctionKind::Scale,
            Self::Scale3d(_) => CssTransformFunctionKind::Scale3d,
            Self::ScaleX(_) => CssTransformFunctionKind::ScaleX,
            Self::ScaleY(_) => CssTransformFunctionKind::ScaleY,
            Self::ScaleZ(_) => CssTransformFunctionKind::ScaleZ,
            Self::Skew(_) => CssTransformFunctionKind::Skew,
            Self::SkewX(_) => CssTransformFunctionKind::SkewX,
            Self::SkewY(_) => CssTransformFunctionKind::SkewY,
            Self::Translate(_) => CssTransformFunctionKind::Translate,
            Self::Translate3d(_) => CssTransformFunctionKind::Translate3d,
            Self::TranslateX(_) => CssTransformFunctionKind::TranslateX,
            Self::TranslateY(_) => CssTransformFunctionKind::TranslateY,
            Self::TranslateZ(_) => CssTransformFunctionKind::TranslateZ,
        }
    }
}

/// A non-empty ordered list of authored transform functions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformFunctionList {
    functions: Vec<CssTransformFunction>,
}

impl CssTransformFunctionList {
    #[must_use]
    pub fn try_new(functions: Vec<CssTransformFunction>) -> Option<Self> {
        (!functions.is_empty()).then_some(Self { functions })
    }

    #[must_use]
    pub fn functions(&self) -> &[CssTransformFunction] {
        &self.functions
    }
}

/// The authored value of the `transform` property.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTransform {
    None,
    Functions(CssTransformFunctionList),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTranslate {
    None,
    Values(CssTranslateValues),
}

#[derive(Clone, Debug)]
pub struct CssTranslateValues {
    x: CssSpecifiedLengthPercentage,
    y: Option<CssSpecifiedLengthPercentage>,
    z: Option<CssSpecifiedLength>,
}
numeric_fields_eq!(CssTranslateValues, [x], [y, z], []);

impl CssTranslateValues {
    /// Retains one to three consecutive axes; an authored Z requires an authored Y.
    pub fn try_new(
        x: CssSpecifiedLengthPercentage,
        y: Option<CssSpecifiedLengthPercentage>,
        z: Option<CssSpecifiedLength>,
    ) -> Option<Self> {
        if y.is_none() && z.is_some() {
            return None;
        }
        Some(Self { x, y, z })
    }
    pub const fn x(&self) -> &CssSpecifiedLengthPercentage {
        &self.x
    }
    pub const fn y(&self) -> Option<&CssSpecifiedLengthPercentage> {
        self.y.as_ref()
    }
    pub const fn z(&self) -> Option<&CssSpecifiedLength> {
        self.z.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssScale {
    None,
    Values(CssScaleValues),
}

#[derive(Clone, Debug)]
pub struct CssScaleValues {
    values: Vec<CssTransformScaleComponent>,
}
impl PartialEq for CssScaleValues {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl CssScaleValues {
    /// Retains one to three checked number/percentage factors in authored axis order.
    /// Symbolic number, percentage and hinted-number calculations remain unresolved.
    #[must_use]
    pub fn try_new(values: Vec<CssTransformScaleComponent>) -> Option<Self> {
        (1..=3).contains(&values.len()).then_some(Self { values })
    }
    #[must_use]
    pub fn values(&self) -> &[CssTransformScaleComponent] {
        &self.values
    }
}

/// Authored perspective; zero is valid and the rendering floor is not applied here.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssPerspective {
    None,
    Length(CssSpecifiedNonNegativeLength),
}
impl PartialEq for CssPerspective {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Length(a), Self::Length(b)) => a.structural_eq(b),
            _ => false,
        }
    }
}
/// Authored transform-style, distinct from contextual used-value flattening.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTransformStyle {
    Flat,
    Preserve3d,
}
/// Authored visibility of the back face, without executing an accumulated matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackfaceVisibility {
    Visible,
    Hidden,
}

/// The optional authored amount accepted by a filter amount function.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssFilterAmount {
    Default,
    Number(CssSpecifiedNonNegativeNumber),
    /// A Number with an unresolved percentage basis; no filter range is applied here.
    HintedNumberCalculation(CssHintedNumberCalculation),
    Percentage(CssSpecifiedNonNegativePercentage),
}

impl PartialEq for CssFilterAmount {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Default, Self::Default) => true,
            (Self::Number(left), Self::Number(right)) => left.structural_eq(right),
            (Self::HintedNumberCalculation(left), Self::HintedNumberCalculation(right)) => {
                left.structural_eq(right)
            }
            (Self::Percentage(left), Self::Percentage(right)) => left == right,
            _ => false,
        }
    }
}

/// A checked specified filter blur length, including the omitted radius's 0px default.
///
/// [`Self::authored_length`] preserves whether the radius was omitted;
/// [`Self::length`] borrows its effective specified value.
#[derive(Clone, Debug)]
pub struct CssFilterBlur {
    length: CssSpecifiedNonNegativeLength,
    omitted: bool,
}
numeric_fields_eq!(CssFilterBlur, [length], [], [omitted]);

impl CssFilterBlur {
    /// Retains an explicitly authored checked radius.
    pub const fn new(length: CssSpecifiedNonNegativeLength) -> Self {
        Self {
            length,
            omitted: false,
        }
    }
    /// The omitted radius has an effective specified value of 0px.
    pub fn omitted() -> Self {
        Self {
            length: CssSpecifiedNonNegativeLength::try_from_component(
                crate::CssComponentValue::try_token("0px").expect("zero pixels token"),
            )
            .expect("zero pixels length"),
            omitted: true,
        }
    }
    /// Borrows the effective nonnegative radius, including an omitted radius's 0px.
    pub const fn length(&self) -> &CssSpecifiedNonNegativeLength {
        &self.length
    }
    /// Borrows only an explicitly authored radius.
    pub const fn authored_length(&self) -> Option<&CssSpecifiedNonNegativeLength> {
        if self.omitted {
            None
        } else {
            Some(&self.length)
        }
    }
}

/// A specified hue rotation, retaining omission separately from its 0deg default.
///
/// [`Self::angle`] borrows the effective specified angle; [`Self::authored_angle`]
/// borrows only an explicitly authored angle. Literal units, unitless zero, and
/// calculation provenance keep the shared [`CssAngleOrZero`] contract.
#[derive(Clone, Debug)]
pub struct CssFilterHueRotate {
    angle: CssAngleOrZero,
    omitted: bool,
}

numeric_fields_eq!(CssFilterHueRotate, [angle], [], [omitted]);

impl CssFilterHueRotate {
    /// Retains an explicitly authored checked angle without normalizing it.
    pub const fn new(angle: CssAngleOrZero) -> Self {
        Self {
            angle,
            omitted: false,
        }
    }

    /// An omitted angle has the effective specified value 0deg.
    pub fn omitted() -> Self {
        Self {
            angle: CssAngleOrZero::Angle(CssAngleValue::from_literal(
                CssAngleLiteral::try_new("0", CssAngleUnit::Degrees).expect("checked zero degrees"),
            )),
            omitted: true,
        }
    }

    /// Borrows the effective specified angle, including an omitted angle's 0deg.
    pub const fn angle(&self) -> &CssAngleOrZero {
        &self.angle
    }

    /// Borrows only an explicitly authored angle.
    pub const fn authored_angle(&self) -> Option<&CssAngleOrZero> {
        if self.omitted {
            None
        } else {
            Some(&self.angle)
        }
    }
}

/// An authored filter `drop-shadow()` with optional nonnegative standard deviation.
/// Unlike a box shadow, it has no inset or spread component.
#[derive(Clone, Debug)]
pub struct CssDropShadow {
    offset_x: CssSpecifiedLength,
    offset_y: CssSpecifiedLength,
    standard_deviation: Option<CssSpecifiedNonNegativeLength>,
    color: Option<Box<CssColor>>,
}
numeric_fields_eq!(
    CssDropShadow,
    [offset_x, offset_y],
    [standard_deviation],
    [color]
);

impl CssDropShadow {
    /// Retains two signed offsets, an optional checked standard deviation, and optional color.
    pub fn new(
        offset_x: CssSpecifiedLength,
        offset_y: CssSpecifiedLength,
        standard_deviation: Option<CssSpecifiedNonNegativeLength>,
        color: Option<CssColor>,
    ) -> Self {
        Self {
            offset_x,
            offset_y,
            standard_deviation,
            color: color.map(Box::new),
        }
    }
    pub const fn offset_x(&self) -> &CssSpecifiedLength {
        &self.offset_x
    }
    pub const fn offset_y(&self) -> &CssSpecifiedLength {
        &self.offset_y
    }
    /// The authored Gaussian standard deviation; omission remains distinct from zero.
    pub const fn standard_deviation(&self) -> Option<&CssSpecifiedNonNegativeLength> {
        self.standard_deviation.as_ref()
    }
    pub fn color(&self) -> Option<&CssColor> {
        self.color.as_deref()
    }
}

/// A parser-produced authored filter function with an exact typed payload.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilterFunction {
    Blur(CssFilterBlur),
    Brightness(CssFilterAmount),
    Contrast(CssFilterAmount),
    DropShadow(CssDropShadow),
    Grayscale(CssFilterAmount),
    HueRotate(CssFilterHueRotate),
    Invert(CssFilterAmount),
    Opacity(CssFilterAmount),
    Saturate(CssFilterAmount),
    Sepia(CssFilterAmount),
    Url(CssUrl),
}

/// A non-empty ordered list of authored filter functions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFilterFunctionList {
    functions: Vec<CssFilterFunction>,
}

impl CssFilterFunctionList {
    #[must_use]
    pub fn try_new(functions: Vec<CssFilterFunction>) -> Option<Self> {
        (!functions.is_empty()).then_some(Self { functions })
    }

    #[must_use]
    pub fn functions(&self) -> &[CssFilterFunction] {
        &self.functions
    }
}

/// The authored value of `filter` or `backdrop-filter`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilter {
    None,
    Functions(CssFilterFunctionList),
}

/// An authored radial extent keyword shared by `circle()` and `ellipse()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRadialExtent {
    ClosestSide,
    FarthestSide,
    ClosestCorner,
    FarthestCorner,
}

mod path_data;
pub use path_data::{CssPathData, CssPathDataConstructionError, CssPathDataConstructionErrorKind};
mod shape_commands;
pub use shape_commands::*;
mod shapes;
pub use shapes::*;
mod mask_serialization;

#[derive(Clone, Debug, PartialEq)]
pub struct CssMaskLayer {
    image: Option<CssImageValue>,
    position: Option<CssPhysicalPosition>,
    size: Option<CssBackgroundSize>,
    repeat: Option<CssBackgroundRepeat>,
}

impl CssMaskLayer {
    /// Constructs a nonempty mask layer with a checked image/none branch.
    #[must_use]
    pub fn try_new(
        image: Option<CssImageValue>,
        position: Option<CssPhysicalPosition>,
        size: Option<CssBackgroundSize>,
        repeat: Option<CssBackgroundRepeat>,
    ) -> Option<Self> {
        if image.is_none() && position.is_none() && size.is_none() && repeat.is_none() {
            None
        } else {
            Some(Self::new(image, position, size, repeat))
        }
    }

    #[must_use]
    pub(crate) const fn new(
        image: Option<CssImageValue>,
        position: Option<CssPhysicalPosition>,
        size: Option<CssBackgroundSize>,
        repeat: Option<CssBackgroundRepeat>,
    ) -> Self {
        Self {
            image,
            position,
            size,
            repeat,
        }
    }

    /// Returns the authored generic position, when present.
    #[must_use]
    pub const fn position(&self) -> Option<&CssPhysicalPosition> {
        self.position.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssMaskList {
    layers: Vec<CssMaskLayer>,
}

impl CssMaskList {
    #[must_use]
    pub fn try_new(layers: Vec<CssMaskLayer>) -> Option<Self> {
        if layers.is_empty() {
            None
        } else {
            Some(Self::new(layers))
        }
    }

    #[must_use]
    pub(crate) fn new(layers: Vec<CssMaskLayer>) -> Self {
        Self { layers }
    }

    #[must_use]
    pub fn layers(&self) -> &[CssMaskLayer] {
        &self.layers
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTimeUnit {
    Seconds,
    Milliseconds,
}

/// A non-empty authored duration list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssDurationList {
    values: Vec<CssDuration>,
}

impl CssDurationList {
    /// Rejects an empty list or retained recovery in any time child.
    #[must_use]
    pub fn try_new(values: Vec<CssDuration>) -> Option<Self> {
        if values.is_empty()
            || values
                .iter()
                .any(|value| value.time().first_implicit_origin().is_some())
        {
            None
        } else {
            Some(Self { values })
        }
    }

    pub(crate) fn from_parser(values: Vec<CssDuration>) -> Self {
        Self { values }
    }

    #[must_use]
    pub fn values(&self) -> &[CssDuration] {
        &self.values
    }
}

/// A non-empty authored delay list.
#[derive(Clone, Debug)]
pub struct CssDelayList {
    values: Vec<CssTimeValue>,
}

impl CssDelayList {
    /// Rejects an empty list or retained recovery in any time child.
    #[must_use]
    pub fn try_new(values: Vec<CssTimeValue>) -> Option<Self> {
        if values.is_empty()
            || values
                .iter()
                .any(|value| value.first_implicit_origin().is_some())
        {
            None
        } else {
            Some(Self { values })
        }
    }

    pub(crate) fn from_parser(values: Vec<CssTimeValue>) -> Self {
        Self { values }
    }

    #[must_use]
    pub fn values(&self) -> &[CssTimeValue] {
        &self.values
    }
}

/// A keyword-authored easing function, including the two step aliases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssEasingKeyword {
    Ease,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    StepStart,
    StepEnd,
}

/// A checked cubic-bezier x coordinate.
#[derive(Clone, Debug)]
pub struct CssCubicBezierX {
    value: CssSpecifiedNumber,
}

numeric_fields_eq!(CssCubicBezierX, [value], [], []);

impl CssCubicBezierX {
    #[must_use]
    pub fn try_new(value: CssSpecifiedNumber) -> Option<Self> {
        if let Some(component) = value.literal_component() {
            let crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Number(number)) =
                component.view()
            else {
                unreachable!("checked number literal")
            };
            let decimal = crate::exact_decimal::LexicalDecimal::new(number.representation());
            if decimal.len != 0 && (decimal.negative || !decimal.absolute_at_most("1", 0)) {
                return None;
            }
        }
        Some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssSpecifiedNumber {
        &self.value
    }
}

/// A checked authored `cubic-bezier()` value.
#[derive(Clone, Debug)]
pub struct CssCubicBezier {
    x1: CssCubicBezierX,
    y1: CssSpecifiedNumber,
    x2: CssCubicBezierX,
    y2: CssSpecifiedNumber,
}

numeric_fields_eq!(CssCubicBezier, [y1, y2], [], [x1, x2]);

impl CssCubicBezier {
    #[must_use]
    pub fn try_new(
        x1: CssSpecifiedNumber,
        y1: CssSpecifiedNumber,
        x2: CssSpecifiedNumber,
        y2: CssSpecifiedNumber,
    ) -> Option<Self> {
        Some(Self {
            x1: CssCubicBezierX::try_new(x1)?,
            y1,
            x2: CssCubicBezierX::try_new(x2)?,
            y2,
        })
    }

    #[must_use]
    pub const fn x1(&self) -> &CssCubicBezierX {
        &self.x1
    }

    #[must_use]
    pub const fn y1(&self) -> &CssSpecifiedNumber {
        &self.y1
    }

    #[must_use]
    pub const fn x2(&self) -> &CssCubicBezierX {
        &self.x2
    }

    #[must_use]
    pub const fn y2(&self) -> &CssSpecifiedNumber {
        &self.y2
    }
}

/// The optional authored position of a `steps()` easing function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssStepPosition {
    JumpStart,
    JumpEnd,
    JumpNone,
    JumpBoth,
    Start,
    End,
}

/// A checked authored `steps()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSteps {
    count: CssPositiveIntegerValue,
    position: Option<CssStepPosition>,
}

impl CssSteps {
    #[must_use]
    pub fn try_new(
        count: CssPositiveIntegerValue,
        position: Option<CssStepPosition>,
    ) -> Option<Self> {
        let count = count.normalized_positive_root()?;
        if matches!(position, Some(CssStepPosition::JumpNone))
            && let CssPositiveIntegerValue::Literal(literal) = &count
            && literal
                .integer()
                .compare_value(&crate::CssIntegerLiteral::from_i32(1))
                .is_eq()
        {
            None
        } else {
            Some(Self { count, position })
        }
    }

    #[must_use]
    pub const fn count(&self) -> &CssPositiveIntegerValue {
        &self.count
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssStepPosition> {
        self.position
    }
}

/// An authored easing function.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssEasing {
    Keyword(CssEasingKeyword),
    CubicBezier(CssCubicBezier),
    Steps(CssSteps),
}

/// A non-empty comma-separated list of authored easing functions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssEasingList {
    values: Vec<CssEasing>,
}

impl CssEasingList {
    #[must_use]
    pub fn try_new(values: Vec<CssEasing>) -> Option<Self> {
        (!values.is_empty()).then_some(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[CssEasing] {
        &self.values
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTransitionProperty {
    All,
    None,
    Custom(CssTransitionPropertyName),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTransitionPropertyList {
    properties: Vec<CssTransitionProperty>,
}

impl CssTransitionPropertyList {
    /// Rejects an empty list or `none` mixed with other items.
    #[must_use]
    pub fn try_new(properties: Vec<CssTransitionProperty>) -> Option<Self> {
        transition_properties_are_valid(properties.iter().map(Some)).then_some(Self { properties })
    }

    #[must_use]
    pub fn properties(&self) -> &[CssTransitionProperty] {
        &self.properties
    }
}

fn transition_properties_are_valid<'a>(
    mut properties: impl ExactSizeIterator<Item = Option<&'a CssTransitionProperty>>,
) -> bool {
    let count = properties.len();
    count > 0
        && (count == 1
            || !properties.any(|property| matches!(property, Some(CssTransitionProperty::None))))
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationName {
    None,
    Custom(CssKeyframesIdent),
    String(CssKeyframesString),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAnimationNameList {
    names: Vec<CssAnimationName>,
}

impl CssAnimationNameList {
    #[must_use]
    pub fn try_new(names: Vec<CssAnimationName>) -> Option<Self> {
        if names.is_empty() {
            None
        } else {
            Some(Self::new(names))
        }
    }

    #[must_use]
    pub(crate) fn new(names: Vec<CssAnimationName>) -> Self {
        Self { names }
    }

    #[must_use]
    pub fn names(&self) -> &[CssAnimationName] {
        &self.names
    }
}

/// An authored animation iteration count, retaining an exact number or symbolic math.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssAnimationIterationCount {
    Infinite,
    Number(CssSpecifiedNonNegativeNumber),
}

impl PartialEq for CssAnimationIterationCount {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Infinite, Self::Infinite) => true,
            (Self::Number(left), Self::Number(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

/// A non-empty list of authored animation iteration counts.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimationIterationCountList {
    values: Vec<CssAnimationIterationCount>,
}

impl CssAnimationIterationCountList {
    #[must_use]
    pub fn try_new(values: Vec<CssAnimationIterationCount>) -> Option<Self> {
        if values.is_empty() {
            None
        } else {
            Some(Self { values })
        }
    }

    #[must_use]
    pub fn values(&self) -> &[CssAnimationIterationCount] {
        &self.values
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAnimationDirectionList {
    directions: Vec<CssAnimationDirection>,
}

impl CssAnimationDirectionList {
    #[must_use]
    pub fn try_new(directions: Vec<CssAnimationDirection>) -> Option<Self> {
        if directions.is_empty() {
            None
        } else {
            Some(Self::new(directions))
        }
    }

    #[must_use]
    pub(crate) fn new(directions: Vec<CssAnimationDirection>) -> Self {
        Self { directions }
    }

    #[must_use]
    pub fn directions(&self) -> &[CssAnimationDirection] {
        &self.directions
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationFillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAnimationFillModeList {
    modes: Vec<CssAnimationFillMode>,
}

impl CssAnimationFillModeList {
    #[must_use]
    pub fn try_new(modes: Vec<CssAnimationFillMode>) -> Option<Self> {
        if modes.is_empty() {
            None
        } else {
            Some(Self::new(modes))
        }
    }

    #[must_use]
    pub(crate) fn new(modes: Vec<CssAnimationFillMode>) -> Self {
        Self { modes }
    }

    #[must_use]
    pub fn modes(&self) -> &[CssAnimationFillMode] {
        &self.modes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationPlayState {
    Running,
    Paused,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAnimationPlayStateList {
    states: Vec<CssAnimationPlayState>,
}

impl CssAnimationPlayStateList {
    #[must_use]
    pub fn try_new(states: Vec<CssAnimationPlayState>) -> Option<Self> {
        if states.is_empty() {
            None
        } else {
            Some(Self::new(states))
        }
    }

    #[must_use]
    pub(crate) fn new(states: Vec<CssAnimationPlayState>) -> Self {
        Self { states }
    }

    #[must_use]
    pub fn states(&self) -> &[CssAnimationPlayState] {
        &self.states
    }
}

/// An authored transition with distinct duration and delay domains.
#[derive(Clone, Debug)]
pub struct CssTransition {
    property: Option<CssTransitionProperty>,
    duration: Option<CssDuration>,
    delay: Option<CssTimeValue>,
    timing_function: Option<CssEasing>,
}

impl CssTransition {
    /// Rejects an empty item or retained recovery in its duration or delay.
    #[must_use]
    pub fn try_new(
        property: Option<CssTransitionProperty>,
        duration: Option<CssDuration>,
        delay: Option<CssTimeValue>,
        timing_function: Option<CssEasing>,
    ) -> Option<Self> {
        if duration
            .as_ref()
            .is_some_and(|v| v.time().first_implicit_origin().is_some())
            || delay
                .as_ref()
                .is_some_and(|v| v.first_implicit_origin().is_some())
        {
            return None;
        }
        Self::from_parser(property, duration, delay, timing_function)
    }
    pub(crate) fn from_parser(
        property: Option<CssTransitionProperty>,
        duration: Option<CssDuration>,
        delay: Option<CssTimeValue>,
        timing_function: Option<CssEasing>,
    ) -> Option<Self> {
        if property.is_none() && duration.is_none() && delay.is_none() && timing_function.is_none()
        {
            None
        } else {
            Some(Self {
                property,
                duration,
                delay,
                timing_function,
            })
        }
    }

    #[must_use]
    pub const fn property(&self) -> Option<&CssTransitionProperty> {
        self.property.as_ref()
    }

    #[must_use]
    pub const fn duration(&self) -> Option<&CssDuration> {
        self.duration.as_ref()
    }

    #[must_use]
    pub const fn delay(&self) -> Option<&CssTimeValue> {
        self.delay.as_ref()
    }

    #[must_use]
    pub const fn timing_function(&self) -> Option<&CssEasing> {
        self.timing_function.as_ref()
    }
}

/// A non-empty authored transition list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransitionList {
    values: Vec<CssTransition>,
}

impl CssTransitionList {
    /// Rejects an empty list, mixed `none`, or retained recovery in any time child.
    #[must_use]
    pub fn try_new(values: Vec<CssTransition>) -> Option<Self> {
        if values.iter().any(|v| {
            v.duration()
                .is_some_and(|d| d.time().first_implicit_origin().is_some())
                || v.delay()
                    .is_some_and(|d| d.first_implicit_origin().is_some())
        }) {
            None
        } else {
            Self::from_parser(values)
        }
    }

    pub(crate) fn from_parser(values: Vec<CssTransition>) -> Option<Self> {
        transition_properties_are_valid(values.iter().map(CssTransition::property))
            .then_some(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[CssTransition] {
        &self.values
    }
}

/// An authored animation with distinct timing domains.
#[derive(Clone, Debug)]
pub struct CssAnimation {
    name: Option<CssAnimationName>,
    duration: Option<CssDuration>,
    delay: Option<CssTimeValue>,
    timing_function: Option<CssEasing>,
    iteration_count: Option<CssAnimationIterationCount>,
    direction: Option<CssAnimationDirection>,
    fill_mode: Option<CssAnimationFillMode>,
    play_state: Option<CssAnimationPlayState>,
}

/// Checked input components for one authored animation.
#[derive(Clone, Debug, Default)]
pub struct CssAnimationComponents {
    pub name: Option<CssAnimationName>,
    pub duration: Option<CssDuration>,
    pub delay: Option<CssTimeValue>,
    pub timing_function: Option<CssEasing>,
    pub iteration_count: Option<CssAnimationIterationCount>,
    pub direction: Option<CssAnimationDirection>,
    pub fill_mode: Option<CssAnimationFillMode>,
    pub play_state: Option<CssAnimationPlayState>,
}

impl CssAnimation {
    /// Rejects an empty item or retained recovery in its duration or delay.
    #[must_use]
    pub fn try_new(components: CssAnimationComponents) -> Option<Self> {
        if components
            .duration
            .as_ref()
            .is_some_and(|v| v.time().first_implicit_origin().is_some())
            || components
                .delay
                .as_ref()
                .is_some_and(|v| v.first_implicit_origin().is_some())
        {
            return None;
        }
        Self::from_parser(components)
    }
    pub(crate) fn from_parser(components: CssAnimationComponents) -> Option<Self> {
        if components.name.is_none()
            && components.duration.is_none()
            && components.delay.is_none()
            && components.timing_function.is_none()
            && components.iteration_count.is_none()
            && components.direction.is_none()
            && components.fill_mode.is_none()
            && components.play_state.is_none()
        {
            None
        } else {
            Some(Self {
                name: components.name,
                duration: components.duration,
                delay: components.delay,
                timing_function: components.timing_function,
                iteration_count: components.iteration_count,
                direction: components.direction,
                fill_mode: components.fill_mode,
                play_state: components.play_state,
            })
        }
    }

    #[must_use]
    pub const fn name(&self) -> Option<&CssAnimationName> {
        self.name.as_ref()
    }

    #[must_use]
    pub const fn duration(&self) -> Option<&CssDuration> {
        self.duration.as_ref()
    }

    #[must_use]
    pub const fn delay(&self) -> Option<&CssTimeValue> {
        self.delay.as_ref()
    }

    #[must_use]
    pub const fn timing_function(&self) -> Option<&CssEasing> {
        self.timing_function.as_ref()
    }

    #[must_use]
    pub const fn iteration_count(&self) -> Option<&CssAnimationIterationCount> {
        self.iteration_count.as_ref()
    }

    #[must_use]
    pub const fn direction(&self) -> Option<CssAnimationDirection> {
        self.direction
    }

    #[must_use]
    pub const fn fill_mode(&self) -> Option<CssAnimationFillMode> {
        self.fill_mode
    }

    #[must_use]
    pub const fn play_state(&self) -> Option<CssAnimationPlayState> {
        self.play_state
    }
}

/// A non-empty authored animation list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimationList {
    values: Vec<CssAnimation>,
}

impl CssAnimationList {
    /// Rejects an empty list or retained recovery in any time child.
    #[must_use]
    pub fn try_new(values: Vec<CssAnimation>) -> Option<Self> {
        if values.is_empty()
            || values.iter().any(|v| {
                v.duration()
                    .is_some_and(|d| d.time().first_implicit_origin().is_some())
                    || v.delay()
                        .is_some_and(|d| d.first_implicit_origin().is_some())
            })
        {
            None
        } else {
            Some(Self { values })
        }
    }

    pub(crate) fn from_parser(values: Vec<CssAnimation>) -> Self {
        Self { values }
    }

    #[must_use]
    pub fn values(&self) -> &[CssAnimation] {
        &self.values
    }
}

pub(crate) mod color;
pub use color::*;

mod selector;
pub(crate) use selector::selector_is_valid_pseudo_suffix;
pub use selector::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAngleUnit {
    Degrees,
    Gradians,
    Radians,
    Turns,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFrequencyUnit {
    Hertz,
    Kilohertz,
}

impl PartialEq for CssDelayList {
    fn eq(&self, other: &Self) -> bool {
        self.values.len() == other.values.len()
            && self
                .values
                .iter()
                .zip(&other.values)
                .all(|(a, b)| a.structural_eq(b))
    }
}
impl PartialEq for CssTransition {
    fn eq(&self, other: &Self) -> bool {
        self.property == other.property
            && self.duration == other.duration
            && optional_numeric_eq(
                self.delay.as_ref(),
                other.delay.as_ref(),
                CssTimeValue::structural_eq,
            )
            && self.timing_function == other.timing_function
    }
}
impl PartialEq for CssAnimation {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.duration == other.duration
            && self.timing_function == other.timing_function
            && self.iteration_count == other.iteration_count
            && self.direction == other.direction
            && self.fill_mode == other.fill_mode
            && self.play_state == other.play_state
            && optional_numeric_eq(
                self.delay.as_ref(),
                other.delay.as_ref(),
                CssTimeValue::structural_eq,
            )
    }
}
impl PartialEq for CssAnimationComponents {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.duration == other.duration
            && self.timing_function == other.timing_function
            && self.iteration_count == other.iteration_count
            && self.direction == other.direction
            && self.fill_mode == other.fill_mode
            && self.play_state == other.play_state
            && optional_numeric_eq(
                self.delay.as_ref(),
                other.delay.as_ref(),
                CssTimeValue::structural_eq,
            )
    }
}

impl CssSheet {
    /// Checks ordered authored rules, deriving bindings from leading namespace declarations.
    /// No encoding or enclosing parsed position is invented.
    pub fn try_from_rules(rules: Vec<CssRule>) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::sheet(&rules)?;
        Ok(Self {
            encoding: None,
            rules,
        })
    }
}

impl CssMediaRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing assembly.
    pub fn try_new(
        query: CssMediaQueryList,
        rules: Vec<CssRule>,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::ordinary_group(&rules, context)?;
        Ok(Self {
            query,
            rules,
            position: None,
        })
    }
}

impl CssSupportsRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing assembly.
    pub fn try_new(
        condition: CssSupportsCondition,
        rules: Vec<CssRule>,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::supports(&condition, context)?;
        crate::rule_construction::ordinary_group(&rules, context)?;
        Ok(Self {
            condition,
            rules,
            position: None,
        })
    }
}

impl CssContainerRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing assembly.
    pub fn try_new(
        prelude: CssContainerPrelude,
        rules: Vec<CssRule>,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::ordinary_group(&rules, context)?;
        Ok(Self {
            prelude,
            rules,
            position: None,
        })
    }
}

impl CssLayerBlockRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing assembly.
    pub fn try_new(
        name: Option<CssLayerName>,
        rules: Vec<CssRule>,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::ordinary_group(&rules, context)?;
        Ok(Self {
            name,
            rules,
            position: None,
        })
    }
}

impl CssScopedMediaRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing assembly.
    pub fn try_new(
        query: CssMediaQueryList,
        rules: Vec<CssScopedRule>,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::scoped_group(&rules, context)?;
        Ok(Self {
            query,
            rules: CssScopedRuleList::from_rules(rules),
            position: None,
        })
    }
}

impl CssScopedSupportsRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing assembly.
    pub fn try_new(
        condition: CssSupportsCondition,
        rules: Vec<CssScopedRule>,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::supports(&condition, context)?;
        crate::rule_construction::scoped_group(&rules, context)?;
        Ok(Self {
            condition,
            rules: CssScopedRuleList::from_rules(rules),
            position: None,
        })
    }
}

impl CssScopedContainerRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing assembly.
    pub fn try_new(
        prelude: CssContainerPrelude,
        rules: Vec<CssScopedRule>,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::scoped_group(&rules, context)?;
        Ok(Self {
            prelude,
            rules: CssScopedRuleList::from_rules(rules),
            position: None,
        })
    }
}

impl CssScopedLayerBlockRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing assembly.
    pub fn try_new(
        name: Option<CssLayerName>,
        rules: Vec<CssScopedRule>,
        context: &crate::CssNamespaceContext,
    ) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::scoped_group(&rules, context)?;
        Ok(Self {
            name,
            rules: CssScopedRuleList::from_rules(rules),
            position: None,
        })
    }
}

impl CssScopeRule {
    /// Assembles boundaries and a direct scope body in the supplied authored
    /// nesting context, checking selectors and the stricter page placement.
    pub fn try_new(
        root: Option<CssScopeSelectorList>,
        limit: Option<CssScopeSelectorList>,
        rules: Vec<CssScopedRule>,
        context: &crate::CssNamespaceContext,
        nesting: CssScopeNestingContext,
    ) -> Result<Self, crate::CssRuleConstructionError> {
        crate::rule_construction::scope(root.as_ref(), limit.as_ref(), &rules, context, nesting)?;
        Ok(Self {
            root,
            limit,
            rules: CssScopedRuleList::from_rules(rules),
            position: None,
        })
    }
}
