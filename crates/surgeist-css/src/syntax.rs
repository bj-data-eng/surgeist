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

use crate::CssFontFeatureValuesRule;
use crate::CssValueOrigin;
pub(crate) use crate::media::*;
pub(crate) use crate::media_features::*;
pub(crate) use crate::numeric::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::component_values::{CssComponentValues, CssParsedOrigin};

pub(crate) use crate::properties::CssKnownDeclaration;
use crate::properties::CssKnownProperty;
use crate::source::CssSourcePosition;

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
    Keyframes(CssKeyframesRule),
    Style(CssStyleRule),
    NestedDeclarations(CssNestedDeclarationsRule),
    Media(CssMediaRule),
    Supports(CssSupportsRule),
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

/// One valid parser-produced Counter Styles 3 definition.
///
/// The private fields couple a checked name, a valid effective descriptor
/// combination, and the authored at-keyword position. This authored model does
/// not register, resolve, inherit, or evaluate a counter style.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterStyleRule {
    name: CssCounterStyleName,
    descriptors: Box<CssCounterStyleDescriptors>,
    position: CssSourcePosition,
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
        }
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

        let effective_system = system
            .as_ref()
            .map(CssDescriptorOccurrence::value)
            .unwrap_or(&CssCounterStyleSystem::Symbolic);
        match effective_system {
            CssCounterStyleSystem::Extends(_)
                if symbols.is_some() || additive_symbols.is_some() =>
            {
                let mut conflicting = Vec::new();
                if symbols.is_some() {
                    conflicting.push("symbols");
                }
                if additive_symbols.is_some() {
                    conflicting.push("additive-symbols");
                }
                return Err(CssCounterStyleCombinationIssue::new(
                    system.as_ref().expect("extends is authored").position(),
                    "system",
                    conflicting,
                ));
            }
            CssCounterStyleSystem::Extends(_) => {}
            CssCounterStyleSystem::Additive => {
                if additive_symbols.is_none() {
                    return Err(CssCounterStyleCombinationIssue::new(
                        system.as_ref().expect("additive is authored").position(),
                        "system",
                        vec!["additive-symbols"],
                    ));
                }
            }
            CssCounterStyleSystem::Numeric | CssCounterStyleSystem::Alphabetic => {
                let Some(symbols) = symbols.as_ref() else {
                    return Err(CssCounterStyleCombinationIssue::new(
                        system.as_ref().expect("system is authored").position(),
                        "system",
                        vec!["symbols"],
                    ));
                };
                if symbols.value().symbols().len() < 2 {
                    return Err(CssCounterStyleCombinationIssue::new(
                        symbols.position(),
                        "symbols",
                        vec!["system"],
                    ));
                }
            }
            CssCounterStyleSystem::Cyclic
            | CssCounterStyleSystem::Symbolic
            | CssCounterStyleSystem::Fixed(_) => {
                if symbols.is_none() {
                    let (position, responsible) = system.as_ref().map_or_else(
                        || {
                            occurrences.first().map_or((None, "symbols"), |descriptor| {
                                (Some(descriptor.position()), "symbols")
                            })
                        },
                        |system| (Some(system.position()), "system"),
                    );
                    return Err(CssCounterStyleCombinationIssue::new_optional(
                        position,
                        responsible,
                        vec!["symbols"],
                    ));
                }
            }
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

    const fn position(&self) -> CssSourcePosition {
        match self {
            Self::System(value) => value.position(),
            Self::Negative(value) => value.position(),
            Self::Symbols(value) => value.position(),
            Self::Prefix(value) => value.position(),
            Self::Suffix(value) => value.position(),
            Self::Range(value) => value.position(),
            Self::Pad(value) => value.position(),
            Self::Fallback(value) => value.position(),
            Self::AdditiveSymbols(value) => value.position(),
            Self::SpeakAs(value) => value.position(),
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssCounterStyleFixedSystem {
    first_symbol_value: Option<i32>,
}

impl CssCounterStyleFixedSystem {
    #[must_use]
    pub(crate) const fn new(first_symbol_value: Option<i32>) -> Self {
        Self { first_symbol_value }
    }

    #[must_use]
    pub const fn first_symbol_value(self) -> Option<i32> {
        self.first_symbol_value
    }
}

/// One or two authored symbols applied around negative counter representations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterStyleNegative {
    prefix: CssCounterSymbol,
    suffix: Option<CssCounterSymbol>,
}

impl CssCounterStyleNegative {
    #[must_use]
    pub(crate) const fn new(prefix: CssCounterSymbol, suffix: Option<CssCounterSymbol>) -> Self {
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
    pub(crate) fn new(ranges: Vec<CssCounterStyleRangeInterval>) -> Self {
        debug_assert!(!ranges.is_empty());
        Self { ranges }
    }

    /// Returns the valid inclusive ranges in authored order.
    #[must_use]
    pub fn ranges(&self) -> &[CssCounterStyleRangeInterval] {
        &self.ranges
    }
}

/// One intrinsically valid inclusive range from a `range` descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssCounterStyleRangeInterval {
    lower: CssCounterStyleRangeBound,
    upper: CssCounterStyleRangeBound,
}

impl CssCounterStyleRangeInterval {
    #[must_use]
    pub(crate) const fn new(
        lower: CssCounterStyleRangeBound,
        upper: CssCounterStyleRangeBound,
    ) -> Self {
        Self { lower, upper }
    }

    /// Returns the inclusive lower bound, where `Infinite` means negative infinity.
    #[must_use]
    pub const fn lower(self) -> CssCounterStyleRangeBound {
        self.lower
    }

    /// Returns the inclusive upper bound, where `Infinite` means positive infinity.
    #[must_use]
    pub const fn upper(self) -> CssCounterStyleRangeBound {
        self.upper
    }
}

/// One authored finite integer or contextual `infinite` range bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleRangeBound {
    Integer(i32),
    Infinite,
}

/// One valid authored zero-padding descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterStylePad {
    minimum_length: u32,
    symbol: CssCounterSymbol,
}

impl CssCounterStylePad {
    #[must_use]
    pub(crate) const fn new(minimum_length: u32, symbol: CssCounterSymbol) -> Self {
        Self {
            minimum_length,
            symbol,
        }
    }

    /// Returns the nonnegative minimum representation length.
    #[must_use]
    pub const fn minimum_length(&self) -> u32 {
        self.minimum_length
    }

    /// Returns the authored padding symbol.
    #[must_use]
    pub const fn symbol(&self) -> &CssCounterSymbol {
        &self.symbol
    }
}

/// A nonempty, strictly descending authored `additive-symbols` list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterAdditiveSymbols {
    tuples: Vec<CssCounterAdditiveTuple>,
}

impl CssCounterAdditiveSymbols {
    #[must_use]
    pub(crate) fn new(tuples: Vec<CssCounterAdditiveTuple>) -> Self {
        debug_assert!(!tuples.is_empty());
        debug_assert!(
            tuples
                .windows(2)
                .all(|pair| pair[0].weight > pair[1].weight)
        );
        Self { tuples }
    }

    /// Returns the additive tuples in strictly descending authored weight order.
    #[must_use]
    pub fn tuples(&self) -> &[CssCounterAdditiveTuple] {
        &self.tuples
    }
}

/// One nonnegative integer weight and counter symbol in an additive list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterAdditiveTuple {
    weight: u32,
    symbol: CssCounterSymbol,
}

impl CssCounterAdditiveTuple {
    #[must_use]
    pub(crate) const fn new(weight: u32, symbol: CssCounterSymbol) -> Self {
        Self { weight, symbol }
    }

    /// Returns the nonnegative additive weight.
    #[must_use]
    pub const fn weight(&self) -> u32 {
        self.weight
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterSymbol {
    String(CssContentString),
    Ident(CssCounterSymbolIdent),
    Url(CssUrl),
}

/// One checked `<custom-ident>` used as an authored counter symbol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterSymbolIdent {
    value: String,
}

impl CssCounterSymbolIdent {
    /// Constructs a counter symbol from one exact decoded CSS custom identifier.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if is_exact_css_identifier(&value)
            && !is_css_wide_keyword(&value)
            && !value.eq_ignore_ascii_case("default")
        {
            Some(Self { value })
        } else {
            None
        }
    }

    /// Returns the exact, case-sensitive decoded identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
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

/// One valid authored Namespaces 3 declaration.
///
/// The declaration retains its optional decoded prefix, literal namespace name,
/// and parser-produced source position. It does not resolve namespace names,
/// load resources, or perform selector matching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNamespaceRule {
    prefix: Option<CssNamespacePrefix>,
    name: CssNamespaceName,
    position: CssSourcePosition,
}

impl CssNamespaceRule {
    #[must_use]
    pub(crate) const fn new(
        prefix: Option<CssNamespacePrefix>,
        name: CssNamespaceName,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            prefix,
            name,
            position,
        }
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

    /// Returns the semantic source position of the rule's at-keyword.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
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
    /// Constructs a prefix from one decoded CSS identifier.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        is_exact_css_identifier(&value).then(|| Self::new(value))
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        debug_assert!(is_exact_css_identifier(&value));
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
    value: String,
}

impl CssImportUrl {
    /// Preserves a decoded authored URL, including empty or whitespace values.
    /// Resource resolution and usability belong to the importing layer.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        Some(Self::new(value))
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self { value }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
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
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        Some(Self::new(value))
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
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

#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceRule {
    descriptors: CssFontFaceDescriptors,
    position: CssSourcePosition,
}

impl CssFontFaceRule {
    #[must_use]
    pub(crate) const fn new(
        descriptors: CssFontFaceDescriptors,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            descriptors,
            position,
        }
    }

    #[must_use]
    pub const fn descriptors(&self) -> &CssFontFaceDescriptors {
        &self.descriptors
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

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
    Ident(CssCustomIdent),
    String(CssKeyframesString),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssKeyframesString {
    value: String,
}

impl CssKeyframesString {
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            None
        } else {
            Some(Self::new(value))
        }
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        debug_assert!(!value.trim().is_empty());
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

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssKeyframeSelector {
    From,
    To,
    Percent(CssKeyframePercent),
}

impl CssKeyframeSelector {
    #[must_use]
    pub fn offset(self) -> CssKeyframePercent {
        match self {
            Self::From => CssKeyframePercent::new(0.0),
            Self::To => CssKeyframePercent::new(100.0),
            Self::Percent(percent) => percent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssKeyframePercent {
    value: CssFiniteNumber,
}

impl CssKeyframePercent {
    #[must_use]
    pub fn try_new(value: f32) -> Option<Self> {
        if (0.0..=100.0).contains(&value) {
            CssFiniteNumber::try_new(value).map(|value| Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(value: f32) -> Self {
        debug_assert!((0.0..=100.0).contains(&value));
        debug_assert!(value.is_finite());
        Self {
            value: CssFiniteNumber::new_unchecked(value),
        }
    }

    #[must_use]
    pub const fn value(self) -> CssFiniteNumber {
        self.value
    }
}

/// Selects an authored `@font-face` descriptor-value grammar.
///
/// This context has no source position: its name is not part of a raw value input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceDescriptorKind {
    FontFamily,
    Src,
    FontWeight,
    FontStyle,
    FontStretch,
    FontDisplay,
    UnicodeRange,
    FontFeatureSettings,
}

impl CssFontFaceDescriptorKind {
    /// Returns the canonical CSS descriptor name.
    #[must_use]
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::FontFamily => "font-family",
            Self::Src => "src",
            Self::FontWeight => "font-weight",
            Self::FontStyle => "font-style",
            Self::FontStretch => "font-stretch",
            Self::FontDisplay => "font-display",
            Self::UnicodeRange => "unicode-range",
            Self::FontFeatureSettings => "font-feature-settings",
        }
    }

    pub(crate) fn from_css_name(name: &str) -> Option<Self> {
        [
            Self::FontFamily,
            Self::Src,
            Self::FontWeight,
            Self::FontStyle,
            Self::FontStretch,
            Self::FontDisplay,
            Self::UnicodeRange,
            Self::FontFeatureSettings,
        ]
        .into_iter()
        .find(|kind| kind.css_name().eq_ignore_ascii_case(name))
    }
}

/// An owned validated descriptor value without parsed occurrence provenance.
///
/// These values contain no descriptor-name position. Raw parsing reports source
/// positions separately in diagnostics; stylesheet occurrences retain their real
/// authored name positions in [`CssDescriptorOccurrence`]. Values remain authored:
/// they do not perform font matching, loading, or contextual usability checks.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceDescriptorValue {
    FontFamily(CssFontFaceFamily),
    Src(CssFontFaceSourceList),
    FontWeight(CssFontFaceWeight),
    FontStyle(CssFontFaceStyle),
    FontStretch(CssFontFaceStretch),
    FontDisplay(CssFontDisplay),
    UnicodeRange(CssUnicodeRangeList),
    FontFeatureSettings(CssAuthoredFontFeatureSettings),
}

impl CssFontFaceDescriptorValue {
    /// Returns the descriptor grammar represented by this typed value.
    #[must_use]
    pub const fn kind(&self) -> CssFontFaceDescriptorKind {
        match self {
            Self::FontFamily(_) => CssFontFaceDescriptorKind::FontFamily,
            Self::Src(_) => CssFontFaceDescriptorKind::Src,
            Self::FontWeight(_) => CssFontFaceDescriptorKind::FontWeight,
            Self::FontStyle(_) => CssFontFaceDescriptorKind::FontStyle,
            Self::FontStretch(_) => CssFontFaceDescriptorKind::FontStretch,
            Self::FontDisplay(_) => CssFontFaceDescriptorKind::FontDisplay,
            Self::UnicodeRange(_) => CssFontFaceDescriptorKind::UnicodeRange,
            Self::FontFeatureSettings(_) => CssFontFaceDescriptorKind::FontFeatureSettings,
        }
    }

    pub(crate) fn into_occurrence(self, position: CssSourcePosition) -> CssFontFaceDescriptor {
        match self {
            Self::FontFamily(value) => {
                CssFontFaceDescriptor::FontFamily(CssDescriptorOccurrence::new(value, position))
            }
            Self::Src(value) => {
                CssFontFaceDescriptor::Src(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontWeight(value) => {
                CssFontFaceDescriptor::FontWeight(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontStyle(value) => {
                CssFontFaceDescriptor::FontStyle(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontStretch(value) => {
                CssFontFaceDescriptor::FontStretch(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontDisplay(value) => {
                CssFontFaceDescriptor::FontDisplay(CssDescriptorOccurrence::new(value, position))
            }
            Self::UnicodeRange(value) => {
                CssFontFaceDescriptor::UnicodeRange(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontFeatureSettings(value) => CssFontFaceDescriptor::FontFeatureSettings(
                CssDescriptorOccurrence::new(value, position),
            ),
        }
    }
}

/// The validated semantic aggregate of authored `@font-face` descriptor occurrences.
///
/// Every valid occurrence retains its authored order, typed value, and descriptor-name position.
/// Typed accessors expose the effective last valid occurrence of each descriptor. Construction is
/// crate-private, so callers cannot forge descriptor provenance. Every descriptor is optional
/// in authored syntax. Font matching later requires effective `font-family` and `src` values;
/// this aggregate does not match or load fonts.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceDescriptors {
    font_family: Option<CssDescriptorOccurrence<CssFontFaceFamily>>,
    src: Option<CssDescriptorOccurrence<CssFontFaceSourceList>>,
    font_weight: Option<CssDescriptorOccurrence<CssFontFaceWeight>>,
    font_style: Option<CssDescriptorOccurrence<CssFontFaceStyle>>,
    font_stretch: Option<CssDescriptorOccurrence<CssFontFaceStretch>>,
    font_display: Option<CssDescriptorOccurrence<CssFontDisplay>>,
    unicode_range: Option<CssDescriptorOccurrence<CssUnicodeRangeList>>,
    font_feature_settings: Option<CssDescriptorOccurrence<CssAuthoredFontFeatureSettings>>,
    occurrences: Vec<CssFontFaceDescriptor>,
}

impl CssFontFaceDescriptors {
    #[must_use]
    #[cfg(test)]
    pub(crate) fn new(
        font_family: Option<CssDescriptorOccurrence<CssFontFaceFamily>>,
        src: Option<CssDescriptorOccurrence<CssFontFaceSourceList>>,
        font_weight: Option<CssDescriptorOccurrence<CssFontFaceWeight>>,
        font_style: Option<CssDescriptorOccurrence<CssFontFaceStyle>>,
        font_stretch: Option<CssDescriptorOccurrence<CssFontFaceStretch>>,
        font_display: Option<CssDescriptorOccurrence<CssFontDisplay>>,
        unicode_range: Option<CssDescriptorOccurrence<CssUnicodeRangeList>>,
    ) -> Self {
        let mut occurrences = Vec::new();
        if let Some(value) = font_family {
            occurrences.push(CssFontFaceDescriptor::FontFamily(value));
        }
        if let Some(value) = src {
            occurrences.push(CssFontFaceDescriptor::Src(value));
        }
        if let Some(value) = font_weight {
            occurrences.push(CssFontFaceDescriptor::FontWeight(value));
        }
        if let Some(value) = font_style {
            occurrences.push(CssFontFaceDescriptor::FontStyle(value));
        }
        if let Some(value) = font_stretch {
            occurrences.push(CssFontFaceDescriptor::FontStretch(value));
        }
        if let Some(value) = font_display {
            occurrences.push(CssFontFaceDescriptor::FontDisplay(value));
        }
        if let Some(value) = unicode_range {
            occurrences.push(CssFontFaceDescriptor::UnicodeRange(value));
        }
        Self::from_occurrences(occurrences)
    }

    #[must_use]
    pub(crate) fn from_occurrences(occurrences: Vec<CssFontFaceDescriptor>) -> Self {
        let mut font_family = None;
        let mut src = None;
        let mut font_weight = None;
        let mut font_style = None;
        let mut font_stretch = None;
        let mut font_display = None;
        let mut unicode_range = None;
        let mut font_feature_settings = None;

        for descriptor in &occurrences {
            match descriptor {
                CssFontFaceDescriptor::FontFamily(value) => font_family = Some(value.clone()),
                CssFontFaceDescriptor::Src(value) => src = Some(value.clone()),
                CssFontFaceDescriptor::FontWeight(value) => font_weight = Some(value.clone()),
                CssFontFaceDescriptor::FontStyle(value) => font_style = Some(value.clone()),
                CssFontFaceDescriptor::FontStretch(value) => font_stretch = Some(value.clone()),
                CssFontFaceDescriptor::FontDisplay(value) => font_display = Some(value.clone()),
                CssFontFaceDescriptor::UnicodeRange(value) => unicode_range = Some(value.clone()),
                CssFontFaceDescriptor::FontFeatureSettings(value) => {
                    font_feature_settings = Some(value.clone());
                }
            }
        }

        Self {
            font_family,
            src,
            font_weight,
            font_style,
            font_stretch,
            font_display,
            unicode_range,
            font_feature_settings,
            occurrences,
        }
    }

    #[must_use]
    /// Returns the effective last valid authored `font-family` occurrence, if present.
    pub const fn font_family(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceFamily>> {
        self.font_family.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `src` occurrence, if present.
    pub const fn src(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceSourceList>> {
        self.src.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-weight` occurrence.
    pub const fn font_weight(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceWeight>> {
        self.font_weight.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-style` occurrence.
    pub const fn font_style(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceStyle>> {
        self.font_style.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-stretch` occurrence.
    pub const fn font_stretch(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceStretch>> {
        self.font_stretch.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-display` occurrence.
    pub const fn font_display(&self) -> Option<&CssDescriptorOccurrence<CssFontDisplay>> {
        self.font_display.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `unicode-range` occurrence.
    pub const fn unicode_range(&self) -> Option<&CssDescriptorOccurrence<CssUnicodeRangeList>> {
        self.unicode_range.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-feature-settings` occurrence.
    pub const fn font_feature_settings(
        &self,
    ) -> Option<&CssDescriptorOccurrence<CssAuthoredFontFeatureSettings>> {
        self.font_feature_settings.as_ref()
    }

    /// Returns every valid authored descriptor occurrence in source order.
    pub fn occurrences(&self) -> impl ExactSizeIterator<Item = CssFontFaceDescriptorRef<'_>> {
        self.occurrences.iter().map(CssFontFaceDescriptor::as_ref)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssFontFaceDescriptor {
    FontFamily(CssDescriptorOccurrence<CssFontFaceFamily>),
    Src(CssDescriptorOccurrence<CssFontFaceSourceList>),
    FontWeight(CssDescriptorOccurrence<CssFontFaceWeight>),
    FontStyle(CssDescriptorOccurrence<CssFontFaceStyle>),
    FontStretch(CssDescriptorOccurrence<CssFontFaceStretch>),
    FontDisplay(CssDescriptorOccurrence<CssFontDisplay>),
    UnicodeRange(CssDescriptorOccurrence<CssUnicodeRangeList>),
    FontFeatureSettings(CssDescriptorOccurrence<CssAuthoredFontFeatureSettings>),
}

impl CssFontFaceDescriptor {
    fn as_ref(&self) -> CssFontFaceDescriptorRef<'_> {
        match self {
            Self::FontFamily(value) => CssFontFaceDescriptorRef::FontFamily(value),
            Self::Src(value) => CssFontFaceDescriptorRef::Src(value),
            Self::FontWeight(value) => CssFontFaceDescriptorRef::FontWeight(value),
            Self::FontStyle(value) => CssFontFaceDescriptorRef::FontStyle(value),
            Self::FontStretch(value) => CssFontFaceDescriptorRef::FontStretch(value),
            Self::FontDisplay(value) => CssFontFaceDescriptorRef::FontDisplay(value),
            Self::UnicodeRange(value) => CssFontFaceDescriptorRef::UnicodeRange(value),
            Self::FontFeatureSettings(value) => {
                CssFontFaceDescriptorRef::FontFeatureSettings(value)
            }
        }
    }
}

/// A borrowed valid authored `@font-face` descriptor occurrence.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceDescriptorRef<'a> {
    FontFamily(&'a CssDescriptorOccurrence<CssFontFaceFamily>),
    Src(&'a CssDescriptorOccurrence<CssFontFaceSourceList>),
    FontWeight(&'a CssDescriptorOccurrence<CssFontFaceWeight>),
    FontStyle(&'a CssDescriptorOccurrence<CssFontFaceStyle>),
    FontStretch(&'a CssDescriptorOccurrence<CssFontFaceStretch>),
    FontDisplay(&'a CssDescriptorOccurrence<CssFontDisplay>),
    UnicodeRange(&'a CssDescriptorOccurrence<CssUnicodeRangeList>),
    FontFeatureSettings(&'a CssDescriptorOccurrence<CssAuthoredFontFeatureSettings>),
}

/// A parser-produced authored `@font-face` descriptor value and its semantic name position.
///
/// The private fields preserve the coupling between a validated descriptor value and the source
/// position of its descriptor-name start. Construction is parser-owned, so callers cannot forge
/// provenance. This occurrence does not apply descriptor matching or load font resources.
///
/// ```compile_fail
/// use surgeist_css::{CssDescriptorOccurrence, CssFontDisplay};
/// let _ = CssDescriptorOccurrence { value: CssFontDisplay::Swap, position: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssDescriptorOccurrence<T> {
    value: T,
    position: CssSourcePosition,
}

impl<T> CssDescriptorOccurrence<T> {
    pub(crate) const fn new(value: T, position: CssSourcePosition) -> Self {
        Self { value, position }
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

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceSource {
    Url(CssFontFaceUrlSource),
    Local(CssFontLocalName),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceUrlSource {
    url: String,
    format: Option<CssFontFormatHint>,
    formats: Option<CssFontFormatList>,
    tech: Vec<CssFontTechHint>,
}

impl CssFontFaceUrlSource {
    /// Preserves an authored URL string and its format and technology hints.
    ///
    /// Every URL string is accepted, including empty and whitespace-only strings.
    /// Resource resolution and loading belong to a later phase. The optional return
    /// retains the existing signature after removal of the nonempty restriction.
    #[must_use]
    pub fn try_new(
        url: impl Into<String>,
        format: Option<CssFontFormatHint>,
        tech: Vec<CssFontTechHint>,
    ) -> Option<Self> {
        let formats =
            format.map(|format| CssFontFormatList::new(CssFontFormatString::new(format.as_str())));
        Some(Self::new_with_formats(url, formats, tech))
    }

    /// Preserves an authored URL, a checked single format argument, and technologies.
    ///
    /// Construction is infallible because the format wrapper already enforces its
    /// cardinality. Empty URLs and empty or unrecognized format strings are valid
    /// authored values. Strings are decoded values, not CSS source to parse again.
    #[must_use]
    pub fn new_with_formats(
        url: impl Into<String>,
        formats: Option<CssFontFormatList>,
        tech: Vec<CssFontTechHint>,
    ) -> Self {
        let format = formats.as_ref().and_then(CssFontFormatList::recognized);
        Self {
            url: url.into(),
            format,
            formats,
            tech,
        }
    }

    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the recognized base format, including the four legacy variation strings.
    ///
    /// `None` can mean either an absent hint or an unrecognized string; use
    /// [`Self::formats`] to distinguish them. Format recognition does not determine
    /// resource support. TrueType and OpenType retain distinct authored identities;
    /// [`CssFontFormatHint::is_equivalent_to`] compares their compatibility meaning.
    #[must_use]
    pub const fn format(&self) -> Option<&CssFontFormatHint> {
        self.format.as_ref()
    }

    /// Returns the single authored `format()` argument, when present.
    ///
    /// The list-named wrapper preserves the original inspection API. A present
    /// hint always contains exactly one string, including empty or unrecognized
    /// strings; `None` means that the source has no `format()` hint.
    #[must_use]
    pub const fn formats(&self) -> Option<&CssFontFormatList> {
        self.formats.as_ref()
    }

    /// Returns technology hints in authored order, including repetitions.
    ///
    /// A legacy format string's implied technology is exposed separately by
    /// [`Self::required_technologies`].
    #[must_use]
    pub fn tech(&self) -> &[CssFontTechHint] {
        &self.tech
    }

    /// Returns the distinct technologies that the source requires together.
    ///
    /// Authored technologies retain first-occurrence order. The `variations`
    /// requirement implied by a legacy variation format string follows them when
    /// it was not already authored. This projection leaves [`Self::tech`] unchanged
    /// and does not decide whether a resource loader supports those technologies.
    pub fn required_technologies(&self) -> impl Iterator<Item = CssFontTechHint> + '_ {
        let implied = self.formats.as_ref().and_then(|formats| {
            formats.formats[0]
                .legacy_variation_format()
                .map(|_| CssFontTechHint::Variations)
        });
        let mut seen = Vec::new();
        self.tech
            .iter()
            .copied()
            .chain(implied)
            .filter(move |hint| {
                if seen.contains(hint) {
                    false
                } else {
                    seen.push(*hint);
                    true
                }
            })
    }
}

/// One authored string from a font source `format()` hint.
///
/// Empty and unrecognized strings remain authored values. This type does not
/// determine whether a resource loader recognizes or supports the format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFormatString {
    value: String,
}

impl CssFontFormatString {
    /// Preserves an authored string, including the empty string.
    ///
    /// Every string is accepted. The optional return retains this constructor's
    /// existing signature after removal of the former nonempty restriction.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        Some(Self::new(value))
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    fn recognized(&self) -> Option<CssFontFormatHint> {
        CssFontFormatHint::from_ascii_name(self.value.as_bytes())
            .or_else(|| self.legacy_variation_format())
    }

    fn legacy_variation_format(&self) -> Option<CssFontFormatHint> {
        // Fonts4 section 4.3.1 defines these string equivalents. Keep this table
        // separate from the keyword grammar: bare legacy names are not keywords.
        if self.value.eq_ignore_ascii_case("woff2-variations") {
            Some(CssFontFormatHint::Woff2)
        } else if self.value.eq_ignore_ascii_case("woff-variations") {
            Some(CssFontFormatHint::Woff)
        } else if self.value.eq_ignore_ascii_case("truetype-variations") {
            Some(CssFontFormatHint::TrueType)
        } else if self.value.eq_ignore_ascii_case("opentype-variations") {
            Some(CssFontFormatHint::OpenType)
        } else {
            None
        }
    }
}

/// Exactly one authored string from a font source `format()` hint.
///
/// The name and slice accessor originate in the earlier Fonts3 list model.
/// The selected Fonts4 grammar permits exactly one argument, so empty and
/// multiple-element inputs are rejected by [`Self::try_new`]. The one string
/// itself may be empty or unrecognized; resource support is a later concern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFormatList {
    formats: [CssFontFormatString; 1],
}

impl CssFontFormatList {
    #[must_use]
    pub fn try_new(formats: Vec<CssFontFormatString>) -> Option<Self> {
        let [format]: [CssFontFormatString; 1] = formats.try_into().ok()?;
        Some(Self::new(format))
    }

    #[must_use]
    pub(crate) fn new(format: CssFontFormatString) -> Self {
        Self { formats: [format] }
    }

    #[must_use]
    pub fn formats(&self) -> &[CssFontFormatString] {
        &self.formats
    }

    #[must_use]
    fn recognized(&self) -> Option<CssFontFormatHint> {
        self.formats[0].recognized()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFaceFamily {
    name: String,
}

impl CssFontFaceFamily {
    /// Preserves a decoded literal name, including empty and whitespace-only names.
    /// Reserved spellings remain literal names and can be serialized quoted.
    /// NUL is rejected because CSS replaces it with a different character.
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        (!name.contains('\0')).then_some(Self { name })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontLocalName {
    name: String,
}

impl CssFontLocalName {
    /// Preserves a decoded literal name, including empty and whitespace-only names.
    /// Reserved spellings remain literal names and can be serialized quoted.
    /// NUL is rejected because CSS replaces it with a different character.
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        (!name.contains('\0')).then_some(Self { name })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceSourceList {
    sources: Vec<CssFontFaceSource>,
}

impl CssFontFaceSourceList {
    #[must_use]
    pub fn try_new(sources: Vec<CssFontFaceSource>) -> Option<Self> {
        if sources.is_empty() {
            None
        } else {
            Some(Self::new(sources))
        }
    }

    #[must_use]
    pub(crate) fn new(sources: Vec<CssFontFaceSource>) -> Self {
        debug_assert!(!sources.is_empty());
        Self { sources }
    }

    #[must_use]
    pub fn sources(&self) -> &[CssFontFaceSource] {
        &self.sources
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFontFaceWeight {
    start: CssFontFaceWeightValue,
    end: Option<CssFontFaceWeightValue>,
    keyword: Option<CssFontFaceWeightKeyword>,
}

impl CssFontFaceWeight {
    #[must_use]
    pub fn normal() -> Self {
        Self::from_keyword(CssFontFaceWeightKeyword::Normal)
    }

    #[must_use]
    pub fn bold() -> Self {
        Self::from_keyword(CssFontFaceWeightKeyword::Bold)
    }

    #[must_use]
    pub(crate) fn from_keyword(keyword: CssFontFaceWeightKeyword) -> Self {
        let value = match keyword {
            CssFontFaceWeightKeyword::Normal => 400.0,
            CssFontFaceWeightKeyword::Bold => 700.0,
        };
        Self {
            start: CssFontFaceWeightValue {
                value: CssFiniteNumber::new_unchecked(value),
            },
            end: None,
            keyword: Some(keyword),
        }
    }

    #[must_use]
    pub fn try_single(value: f32) -> Option<Self> {
        Some(Self {
            start: CssFontFaceWeightValue::try_new(value)?,
            end: None,
            keyword: None,
        })
    }

    #[must_use]
    pub fn try_range(start: f32, end: f32) -> Option<Self> {
        let start = CssFontFaceWeightValue::try_new(start)?;
        let end = CssFontFaceWeightValue::try_new(end)?;
        if start.value().value() <= end.value().value() {
            Some(Self {
                start,
                end: Some(end),
                keyword: None,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn start(self) -> CssFontFaceWeightValue {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> Option<CssFontFaceWeightValue> {
        self.end
    }

    #[must_use]
    pub const fn keyword(self) -> Option<CssFontFaceWeightKeyword> {
        self.keyword
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceWeightKeyword {
    Normal,
    Bold,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFontFaceWeightValue {
    value: CssFiniteNumber,
}

impl CssFontFaceWeightValue {
    #[must_use]
    pub fn try_new(value: f32) -> Option<Self> {
        if (1.0..=1000.0).contains(&value) {
            CssFiniteNumber::try_new(value).map(|value| Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> CssFiniteNumber {
        self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceStyle {
    Normal,
    Italic,
    Oblique(Option<CssFontFaceObliqueRange>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFontFaceObliqueRange {
    start_degrees: CssFiniteNumber,
    end_degrees: Option<CssFiniteNumber>,
}

impl CssFontFaceObliqueRange {
    #[must_use]
    pub fn try_new(start_degrees: f32, end_degrees: Option<f32>) -> Option<Self> {
        if !(-90.0..=90.0).contains(&start_degrees) {
            return None;
        }

        let start_degrees = CssFiniteNumber::try_new(start_degrees)?;
        let end_degrees = match end_degrees {
            Some(end_degrees)
                if (-90.0..=90.0).contains(&end_degrees)
                    && start_degrees.value() <= end_degrees =>
            {
                Some(CssFiniteNumber::try_new(end_degrees)?)
            }
            Some(_) => return None,
            None => None,
        };

        Some(Self {
            start_degrees,
            end_degrees,
        })
    }

    #[must_use]
    pub const fn start_degrees(self) -> CssFiniteNumber {
        self.start_degrees
    }

    #[must_use]
    pub const fn end_degrees(self) -> Option<CssFiniteNumber> {
        self.end_degrees
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFontFaceStretch {
    start: CssFontFaceStretchValue,
    end: Option<CssFontFaceStretchValue>,
    keyword: Option<CssFontFaceStretchKeyword>,
}

impl CssFontFaceStretch {
    #[must_use]
    pub fn from_keyword(keyword: CssFontFaceStretchKeyword) -> Self {
        Self {
            start: CssFontFaceStretchValue {
                percent: CssFiniteNumber::new_unchecked(keyword.percent()),
            },
            end: None,
            keyword: Some(keyword),
        }
    }

    #[must_use]
    pub fn try_single_percent(percent: f32) -> Option<Self> {
        Some(Self {
            start: CssFontFaceStretchValue::try_new_percent(percent)?,
            end: None,
            keyword: None,
        })
    }

    #[must_use]
    pub fn try_range_percent(start: f32, end: f32) -> Option<Self> {
        let start = CssFontFaceStretchValue::try_new_percent(start)?;
        let end = CssFontFaceStretchValue::try_new_percent(end)?;
        if start.percent().value() <= end.percent().value() {
            Some(Self {
                start,
                end: Some(end),
                keyword: None,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn start(self) -> CssFontFaceStretchValue {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> Option<CssFontFaceStretchValue> {
        self.end
    }

    #[must_use]
    pub const fn keyword(self) -> Option<CssFontFaceStretchKeyword> {
        self.keyword
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceStretchKeyword {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl CssFontFaceStretchKeyword {
    const fn percent(self) -> f32 {
        match self {
            Self::UltraCondensed => 50.0,
            Self::ExtraCondensed => 62.5,
            Self::Condensed => 75.0,
            Self::SemiCondensed => 87.5,
            Self::Normal => 100.0,
            Self::SemiExpanded => 112.5,
            Self::Expanded => 125.0,
            Self::ExtraExpanded => 150.0,
            Self::UltraExpanded => 200.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFontFaceStretchValue {
    percent: CssFiniteNumber,
}

impl CssFontFaceStretchValue {
    #[must_use]
    pub fn try_new_percent(percent: f32) -> Option<Self> {
        if percent >= 0.0 {
            CssFiniteNumber::try_new(percent).map(|percent| Self { percent })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn percent(self) -> CssFiniteNumber {
        self.percent
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontDisplay {
    Auto,
    Block,
    Swap,
    Fallback,
    Optional,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssUnicodeRangeList {
    ranges: Vec<CssUnicodeRange>,
}

impl CssUnicodeRangeList {
    #[must_use]
    pub fn try_new(ranges: Vec<CssUnicodeRange>) -> Option<Self> {
        if ranges.is_empty() {
            None
        } else {
            Some(Self::new(ranges))
        }
    }

    #[must_use]
    pub(crate) fn new(ranges: Vec<CssUnicodeRange>) -> Self {
        debug_assert!(!ranges.is_empty());
        Self { ranges }
    }

    #[must_use]
    pub fn ranges(&self) -> &[CssUnicodeRange] {
        &self.ranges
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssUnicodeRange {
    start: u32,
    end: u32,
}

impl CssUnicodeRange {
    #[must_use]
    pub const fn try_new(start: u32, end: u32) -> Option<Self> {
        if start <= end && end <= 0x10ffff {
            Some(Self { start, end })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn start(self) -> u32 {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> u32 {
        self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFormatHint {
    Woff,
    Woff2,
    TrueType,
    OpenType,
    Collection,
    EmbeddedOpenType,
    Svg,
}

impl CssFontFormatHint {
    /// Compares format compatibility without changing authored equality.
    ///
    /// Fonts4 section 11.2 makes TrueType and OpenType synonymous. Other formats
    /// are equivalent only to themselves. This does not compare technology hints
    /// or determine resource support.
    #[must_use]
    pub fn is_equivalent_to(self, other: Self) -> bool {
        self == other
            || matches!(
                (self, other),
                (Self::TrueType, Self::OpenType) | (Self::OpenType, Self::TrueType)
            )
    }

    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Woff => "woff",
            Self::Woff2 => "woff2",
            Self::TrueType => "truetype",
            Self::OpenType => "opentype",
            Self::Collection => "collection",
            Self::EmbeddedOpenType => "embedded-opentype",
            Self::Svg => "svg",
        }
    }

    pub(crate) fn from_ascii_name(value: &[u8]) -> Option<Self> {
        if value.eq_ignore_ascii_case(b"woff") {
            Some(Self::Woff)
        } else if value.eq_ignore_ascii_case(b"woff2") {
            Some(Self::Woff2)
        } else if value.eq_ignore_ascii_case(b"truetype") {
            Some(Self::TrueType)
        } else if value.eq_ignore_ascii_case(b"opentype") {
            Some(Self::OpenType)
        } else if value.eq_ignore_ascii_case(b"collection") {
            Some(Self::Collection)
        } else if value.eq_ignore_ascii_case(b"embedded-opentype") {
            Some(Self::EmbeddedOpenType)
        } else if value.eq_ignore_ascii_case(b"svg") {
            Some(Self::Svg)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontTechHint {
    Variations,
    Palettes,
    ColorCOLRv0,
    ColorCOLRv1,
    ColorSVG,
    ColorSbix,
    ColorCBDT,
    FeaturesOpenType,
    FeaturesAAT,
    FeaturesGraphite,
    Incremental,
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
    position: CssSourcePosition,
}

impl CssLayerBlockRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @layer parser construction.
    pub(crate) const fn new(
        name: Option<CssLayerName>,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            rules,
            position,
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
    pub const fn position(&self) -> CssSourcePosition {
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
    position: CssSourcePosition,
}

impl CssMediaRule {
    #[must_use]
    pub(crate) const fn new(
        query: CssMediaQueryList,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            query,
            rules,
            position,
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
    pub const fn position(&self) -> CssSourcePosition {
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
    position: CssSourcePosition,
}

impl CssSupportsRule {
    #[must_use]
    pub(crate) const fn new(
        condition: CssSupportsCondition,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            condition,
            rules,
            position,
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
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

/// One authored supports condition with original lexical backing.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsCondition {
    kind: Box<CssSupportsConditionKind>,
    lexical: crate::supports::SupportsLexical,
    origin: CssValueOrigin,
    bare_declaration: bool,
}
impl CssSupportsCondition {
    pub(crate) fn new(
        kind: CssSupportsConditionKind,
        lexical: crate::supports::SupportsLexical,
        bare_declaration: bool,
    ) -> Self {
        let origin = lexical.first_origin().clone();
        Self {
            kind: Box::new(kind),
            lexical,
            origin,
            bare_declaration,
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
    pub(crate) const fn bare_declaration(&self) -> bool {
        self.bare_declaration
    }
}

/// The complete authored shape of one supports condition.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssSupportsConditionKind {
    Declaration(Box<CssSupportsDeclaration>),
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
    name: Option<CssContainerName>,
    condition: CssContainerCondition,
    rules: Vec<CssRule>,
    position: CssSourcePosition,
}

impl CssContainerRule {
    #[must_use]
    pub(crate) const fn new(
        name: Option<CssContainerName>,
        condition: CssContainerCondition,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            condition,
            rules,
            position,
        }
    }

    #[must_use]
    pub const fn name(&self) -> Option<&CssContainerName> {
        self.name.as_ref()
    }

    #[must_use]
    pub const fn condition(&self) -> &CssContainerCondition {
        &self.condition
    }

    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopeRule {
    root: Option<CssScopeSelectorList>,
    limit: Option<CssScopeSelectorList>,
    rules: CssScopedRuleList,
    position: CssSourcePosition,
}

impl CssScopeRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @scope parser construction.
    pub(crate) const fn new(
        root: Option<CssScopeSelectorList>,
        limit: Option<CssScopeSelectorList>,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            root,
            limit,
            rules,
            position,
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
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopeSelectorList {
    selectors: Vec<CssSelector>,
}

impl CssScopeSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssSelector>) -> Option<Self> {
        if selectors.is_empty() || selectors.iter().any(CssSelector::has_pseudo_elements) {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        debug_assert!(!selectors.iter().any(CssSelector::has_pseudo_elements));
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssSelector] {
        &self.selectors
    }
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
    CustomMedia(crate::CssCustomMediaRule),
    FontFeatureValues(CssFontFeatureValuesRule),
    Style(CssScopedStyleRule),
    Media(CssScopedMediaRule),
    Supports(CssScopedSupportsRule),
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

    /// Returns leading declarations before the first retained child rule.
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
    position: CssSourcePosition,
}

impl CssScopedMediaRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @scope parser construction.
    pub(crate) const fn new(
        query: CssMediaQueryList,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            query,
            rules,
            position,
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
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

/// A parser-produced supports group inside a scoped rule list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssScopedSupportsRule {
    condition: CssSupportsCondition,
    rules: CssScopedRuleList,
    position: CssSourcePosition,
}

impl CssScopedSupportsRule {
    #[must_use]
    pub(crate) const fn new(
        condition: CssSupportsCondition,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            condition,
            rules,
            position,
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
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScopedContainerRule {
    name: Option<CssContainerName>,
    condition: CssContainerCondition,
    rules: CssScopedRuleList,
    position: CssSourcePosition,
}

impl CssScopedContainerRule {
    #[must_use]
    #[allow(dead_code)] // Staged for @scope parser construction.
    pub(crate) const fn new(
        name: Option<CssContainerName>,
        condition: CssContainerCondition,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            condition,
            rules,
            position,
        }
    }

    #[must_use]
    pub const fn name(&self) -> Option<&CssContainerName> {
        self.name.as_ref()
    }

    #[must_use]
    pub const fn condition(&self) -> &CssContainerCondition {
        &self.condition
    }

    #[must_use]
    pub const fn rules(&self) -> &CssScopedRuleList {
        &self.rules
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
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
    position: CssSourcePosition,
}

impl CssScopedLayerBlockRule {
    #[must_use]
    #[allow(dead_code)] // Staged for scoped @layer parser construction.
    pub(crate) const fn new(
        name: Option<CssLayerName>,
        rules: CssScopedRuleList,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            rules,
            position,
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
    pub const fn position(&self) -> CssSourcePosition {
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
    /// Every supplied member is retained in order. This returns `Some` for all
    /// lists; the `Option` return type is preserved for compatibility.
    #[must_use]
    pub fn try_new(queries: Vec<CssMediaQuery>) -> Option<Self> {
        Some(Self::new(queries))
    }

    #[must_use]
    pub(crate) fn new(queries: Vec<CssMediaQuery>) -> Self {
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
            | "style"
    )
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerCondition {
    Feature(CssContainerFeatureQuery),
    Style(CssContainerStyleQuery),
    Not(Box<CssContainerCondition>),
    And(CssContainerConditionList),
    Or(CssContainerConditionList),
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
    Width(CssRangeFeature<CssQueryLength>),
    Height(CssRangeFeature<CssQueryLength>),
    InlineSize(CssRangeFeature<CssQueryLength>),
    BlockSize(CssRangeFeature<CssQueryLength>),
    AspectRatio(CssRangeFeature<CssRatio>),
    Orientation(CssOrientation),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerStyleQuery {
    CustomPropertyPresence(CssCustomPropertyName),
    CustomPropertyValue {
        name: CssCustomPropertyName,
        value: CssAuthoredDeclarationValue,
    },
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
    pub(crate) fn new(comparison: Option<CssQueryComparison>, value: T) -> Self {
        Self { comparison, value }
    }

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
pub struct CssNonNegativeInteger {
    value: u32,
}

impl CssNonNegativeInteger {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { value }
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.value
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssResolution {
    value: CssFiniteNumber,
    unit: CssResolutionUnit,
}

impl CssResolution {
    #[must_use]
    pub fn try_new(value: f32, unit: CssResolutionUnit) -> Option<Self> {
        if value > 0.0 {
            CssFiniteNumber::try_new(value).map(|value| Self { value, unit })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> CssFiniteNumber {
        self.value
    }

    #[must_use]
    pub const fn unit(self) -> CssResolutionUnit {
        self.unit
    }
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

    /// Returns the explicit unit, or the `px`-equivalent compatibility projection for unitless
    /// zero. Use [`Self::authored_unit`] when exact authored-unit presence matters.
    #[must_use]
    pub const fn unit(self) -> CssLengthUnit {
        match self.unit {
            Some(unit) => unit,
            None => CssLengthUnit::Px,
        }
    }

    /// Returns the exact authored unit, or `None` for a valid unitless zero.
    #[must_use]
    pub const fn authored_unit(self) -> Option<CssLengthUnit> {
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

    /// Returns the leading declarations, before the first retained child rule.
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

    /// Returns the leading declarations, before the first retained child rule.
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
/// The private fields retain property/value coupling and the property-name position. Keyframe
/// grammar rejects declaration importance, so this type intentionally has no importance field or
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
    position: CssSourcePosition,
}

impl CssKeyframeDeclaration {
    pub(crate) const fn new(body: CssDeclarationBody, position: CssSourcePosition) -> Self {
        Self { body, position }
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
    reason = "the stable declaration enum keeps values inline while typed properties retain current and I01 projections"
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

// Kept crate-private for the existing overflow shorthand parser's two-result helper. This is not
// a declaration value bag and cannot be paired with a property.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssOverflowParsedValue {
    Overflow(CssOverflow),
    OverflowAxes(CssOverflowAxes),
}

pub(crate) use CssOverflowParsedValue as CssValue;

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
    pub(crate) const fn new_unchecked(value: f32) -> Self {
        Self { value }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssNonNegativeNumber {
    value: CssFiniteNumber,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssNonNegativeNumberValue {
    Literal(CssNonNegativeNumber),
    Calculation(CssNumberCalculation),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssPositiveNumber {
    value: CssFiniteNumber,
}

impl CssPositiveNumber {
    #[must_use]
    pub fn try_new(value: f32) -> Option<Self> {
        if value > 0.0 {
            CssFiniteNumber::try_new(value).map(|value| Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPositiveNumberValue {
    Literal(CssPositiveNumber),
    Calculation(CssNumberCalculation),
}

impl CssNonNegativeNumber {
    #[must_use]
    pub fn try_new(value: f32) -> Option<Self> {
        if value >= 0.0 {
            CssFiniteNumber::try_new(value).map(|value| Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssOpacity {
    value: CssFiniteNumber,
}

impl CssOpacity {
    #[must_use]
    pub fn try_new(value: f32) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            let value = CssFiniteNumber::try_new(value)?;
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOpacityValue {
    Literal(CssOpacity),
    Calculation(CssNumberCalculation),
    Number(CssFiniteNumber),
    Percentage(CssFiniteNumber),
    PercentageCalculation(CssPercentageCalculation),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFlexFactor {
    value: CssNonNegativeNumber,
}

impl CssFlexFactor {
    #[must_use]
    pub fn try_new(value: f32) -> Option<Self> {
        CssNonNegativeNumber::try_new(value).map(|value| Self { value })
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssAspectRatio {
    value: CssFiniteNumber,
}

impl CssAspectRatio {
    #[must_use]
    pub fn try_new(value: f32) -> Option<Self> {
        if value > 0.0 {
            let value = CssFiniteNumber::try_new(value)?;
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAspectRatioValue {
    Literal(CssAspectRatio),
    Calculation(CssNumberCalculation),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollbarWidth {
    Auto,
    Thin,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssDisplay {
    Block,
    Flex,
    Grid,
    InlineBlock,
    InlineGrid,
    GridLanes,
    InlineGridLanes,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssOverflowAxes {
    pub x: CssOverflow,
    pub y: CssOverflow,
}

impl CssOverflowAxes {
    #[must_use]
    pub const fn new(x: CssOverflow, y: CssOverflow) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// The authored component values of the `flex-flow` shorthand.
///
/// Omitted components use their specified initial values. This syntax value does not perform
/// flex layout or resolve either component against layout context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFlexFlow {
    direction: CssFlexDirection,
    wrap: CssFlexWrap,
}

impl CssFlexFlow {
    pub(crate) const fn new(direction: CssFlexDirection, wrap: CssFlexWrap) -> Self {
        Self { direction, wrap }
    }

    /// Returns the authored direction component, or `row` when it was omitted.
    #[must_use]
    pub const fn direction(&self) -> CssFlexDirection {
        self.direction
    }

    /// Returns the authored wrapping component, or `nowrap` when it was omitted.
    #[must_use]
    pub const fn wrap(&self) -> CssFlexWrap {
        self.wrap
    }
}

/// A checked positive authored integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssPositiveInteger {
    value: i32,
}

impl CssPositiveInteger {
    #[must_use]
    pub const fn try_new(value: i32) -> Option<Self> {
        if value > 0 {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

/// A positive integer whose calculated range remains authored and symbolic.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPositiveIntegerValue {
    Literal(CssPositiveInteger),
    Calculation(CssIntegerCalculation),
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

/// A checked non-negative authored length without a percentage component.
///
/// Calculations remain symbolic because their range is enforced during computed-value
/// processing, outside this crate's authored-syntax boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct CssNonNegativeLength {
    value: CssLength,
}

impl CssNonNegativeLength {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        let valid = match &value {
            CssLength::Px(value) => value.value() >= 0.0,
            CssLength::Dimension(value) => value.value() >= 0.0,
            CssLength::Zero => true,
            CssLength::Calc(value) => !value.uses_percentage(),
            CssLength::Percent(_)
            | CssLength::Auto
            | CssLength::MinContent
            | CssLength::MaxContent
            | CssLength::FitContent
            | CssLength::Normal
            | CssLength::Thin
            | CssLength::Medium
            | CssLength::Thick => false,
        };
        valid.then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// The shared authored `<line-style>` domain used by borders and column rules.
pub type CssLineStyle = CssBorderStyle;

/// The shared authored `<line-width>` domain.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssLineWidth {
    Thin,
    Medium,
    Thick,
    Length(CssNonNegativeLength),
}

/// The authored components present in a `column-rule` shorthand.
#[derive(Clone)]
pub struct CssColumnRule {
    width: Option<CssLineWidth>,
    style: Option<CssLineStyle>,
    color: Option<Box<CssParsedColor>>,
}

impl std::fmt::Debug for CssColumnRule {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssColumnRule")
            .field("width", &self.width)
            .field("style", &self.style)
            .field("color", &self.current_color())
            .finish()
    }
}

impl PartialEq for CssColumnRule {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.style == other.style
            && parsed_color_options_equal(self.color.as_deref(), other.color.as_deref())
    }
}

impl CssColumnRule {
    pub(crate) fn new(
        width: Option<CssLineWidth>,
        style: Option<CssLineStyle>,
        color: Option<CssParsedColor>,
    ) -> Self {
        Self {
            width,
            style,
            color: color.map(Box::new),
        }
    }

    #[must_use]
    pub const fn width(&self) -> Option<&CssLineWidth> {
        self.width.as_ref()
    }

    #[must_use]
    pub const fn style(&self) -> Option<CssLineStyle> {
        self.style
    }

    /// Returns the exact authored color component, when present.
    #[must_use]
    pub const fn current_color(&self) -> Option<&CssAuthoredColor> {
        match self.color.as_ref() {
            Some(color) => Some(color.current()),
            None => None,
        }
    }

    /// Returns the frozen I01 color representation when one exists.
    #[must_use]
    pub const fn color(&self) -> Option<&CssColor> {
        match self.color.as_ref() {
            Some(color) => color.i01_subset(),
            None => None,
        }
    }
}

/// The authored `column-span` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColumnSpan {
    None,
    All,
}

/// The authored `column-width` value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColumnWidth {
    Auto,
    Length(CssNonNegativeLength),
}

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
    pub(crate) const fn new(width: CssColumnWidth, count: CssColumnCount) -> Self {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssClear {
    Left,
    Right,
    Both,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAlignment {
    Normal,
    Start,
    End,
    SafeEnd,
    FlexStart,
    FlexEnd,
    SafeFlexEnd,
    Center,
    SafeCenter,
    Baseline,
    FirstBaseline,
    LastBaseline,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAlignItems {
    Normal,
    Start,
    End,
    SafeEnd,
    FlexStart,
    FlexEnd,
    SafeFlexEnd,
    Center,
    SafeCenter,
    Baseline,
    FirstBaseline,
    LastBaseline,
    Stretch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPlaceAlignment {
    Content(CssPlaceContentAlignment),
    Items(CssPlaceItemsAlignment),
}

impl CssPlaceAlignment {
    #[must_use]
    pub const fn content(first: CssAlignment, second: CssAlignment) -> Self {
        Self::Content(CssPlaceContentAlignment::new(first, second))
    }

    #[must_use]
    pub const fn content_all(value: CssAlignment) -> Self {
        Self::content(value, value)
    }

    #[must_use]
    pub const fn items(first: CssAlignItems, second: CssAlignItems) -> Self {
        Self::Items(CssPlaceItemsAlignment::new(first, second))
    }

    #[must_use]
    pub const fn items_all(value: CssAlignItems) -> Self {
        Self::items(value, value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssPlaceContentAlignment {
    first: CssAlignment,
    second: CssAlignment,
}

impl CssPlaceContentAlignment {
    #[must_use]
    pub const fn new(first: CssAlignment, second: CssAlignment) -> Self {
        Self { first, second }
    }

    #[must_use]
    pub const fn all(value: CssAlignment) -> Self {
        Self::new(value, value)
    }

    #[must_use]
    pub const fn first(self) -> CssAlignment {
        self.first
    }

    #[must_use]
    pub const fn second(self) -> CssAlignment {
        self.second
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssPlaceItemsAlignment {
    first: CssAlignItems,
    second: CssAlignItems,
}

impl CssPlaceItemsAlignment {
    #[must_use]
    pub const fn new(first: CssAlignItems, second: CssAlignItems) -> Self {
        Self { first, second }
    }

    #[must_use]
    pub const fn all(value: CssAlignItems) -> Self {
        Self::new(value, value)
    }

    #[must_use]
    pub const fn first(self) -> CssAlignItems {
        self.first
    }

    #[must_use]
    pub const fn second(self) -> CssAlignItems {
        self.second
    }
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
#[non_exhaustive]
pub enum CssContent {
    Normal,
    None,
    Items(CssContentList),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContentList {
    items: Vec<CssContentItem>,
}

impl CssContentList {
    #[must_use]
    pub fn try_new(items: Vec<CssContentItem>) -> Option<Self> {
        if items.is_empty() {
            None
        } else {
            Some(Self { items })
        }
    }

    #[must_use]
    pub fn items(&self) -> &[CssContentItem] {
        &self.items
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContentItem {
    String(CssContentString),
    Url(CssUrl),
    Counter(CssCounterFunction),
    Counters(CssCountersFunction),
    Attr(CssAttributeName),
    OpenQuote,
    CloseQuote,
    NoOpenQuote,
    NoCloseQuote,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssCounterName {
    name: String,
}

impl CssCounterName {
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if is_valid_counter_name(&name) {
            Some(Self { name })
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterStyleName {
    name: String,
}

impl CssCounterStyleName {
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if is_valid_counter_style_name(&name) {
            Some(Self::new(name))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

fn is_valid_counter_name(name: &str) -> bool {
    is_css_ident(name) && !is_css_wide_keyword(name) && !name.eq_ignore_ascii_case("none")
}

fn is_valid_counter_style_name(name: &str) -> bool {
    is_css_ident(name)
        && !is_css_wide_keyword(name)
        && !name.eq_ignore_ascii_case("default")
        && !name.eq_ignore_ascii_case("none")
}

fn is_css_ident(value: &str) -> bool {
    let mut input = cssparser::ParserInput::new(value);
    let mut parser = cssparser::Parser::new(&mut input);
    let Ok(parsed) = parser.expect_ident_cloned() else {
        return false;
    };
    parser.expect_exhausted().is_ok() && parsed.as_ref() == value
}

fn is_css_wide_keyword(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterFunction {
    name: CssCounterName,
    style: Option<CssCounterStyle>,
}

impl CssCounterFunction {
    #[must_use]
    pub const fn new(name: CssCounterName, style: Option<CssCounterStyle>) -> Self {
        Self { name, style }
    }

    #[must_use]
    pub const fn name(&self) -> &CssCounterName {
        &self.name
    }

    #[must_use]
    pub const fn style(&self) -> Option<&CssCounterStyle> {
        self.style.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCountersFunction {
    name: CssCounterName,
    separator: CssContentString,
    style: Option<CssCounterStyle>,
}

impl CssCountersFunction {
    #[must_use]
    pub const fn new(
        name: CssCounterName,
        separator: CssContentString,
        style: Option<CssCounterStyle>,
    ) -> Self {
        Self {
            name,
            separator,
            style,
        }
    }

    #[must_use]
    pub const fn name(&self) -> &CssCounterName {
        &self.name
    }

    #[must_use]
    pub const fn separator(&self) -> &CssContentString {
        &self.separator
    }

    #[must_use]
    pub const fn style(&self) -> Option<&CssCounterStyle> {
        self.style.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyle {
    BuiltIn(CssBuiltInCounterStyle),
    Named(CssCounterStyleName),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBuiltInCounterStyle {
    Disc,
    Circle,
    Square,
    Decimal,
    DecimalLeadingZero,
    LowerAlpha,
    UpperAlpha,
    LowerLatin,
    UpperLatin,
    LowerRoman,
    UpperRoman,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssListStyleType {
    None,
    CounterStyle(CssCounterStyle),
    String(CssContentString),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssListStylePosition {
    Inside,
    Outside,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssListStyleImage {
    None,
    Url(CssUrl),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssListStyle {
    style_type: Option<CssListStyleType>,
    position: Option<CssListStylePosition>,
    image: Option<CssListStyleImage>,
}

impl CssListStyle {
    #[must_use]
    pub fn try_new(
        style_type: Option<CssListStyleType>,
        position: Option<CssListStylePosition>,
        image: Option<CssListStyleImage>,
    ) -> Option<Self> {
        if style_type.is_none() && position.is_none() && image.is_none() {
            None
        } else {
            Some(Self {
                style_type,
                position,
                image,
            })
        }
    }

    #[must_use]
    pub const fn style_type(&self) -> Option<&CssListStyleType> {
        self.style_type.as_ref()
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssListStylePosition> {
        self.position
    }

    #[must_use]
    pub const fn image(&self) -> Option<&CssListStyleImage> {
        self.image.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterChanges {
    None,
    Changes(CssCounterChangeList),
}

impl CssCounterChanges {
    #[must_use]
    pub fn try_changes(changes: Vec<CssCounterChange>) -> Option<Self> {
        CssCounterChangeList::try_new(changes).map(Self::Changes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterChangeList {
    changes: Vec<CssCounterChange>,
}

impl CssCounterChangeList {
    #[must_use]
    pub fn try_new(changes: Vec<CssCounterChange>) -> Option<Self> {
        if changes.is_empty() {
            None
        } else {
            Some(Self { changes })
        }
    }

    #[must_use]
    pub fn changes(&self) -> &[CssCounterChange] {
        &self.changes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterChange {
    name: CssCounterName,
    value: Option<i32>,
}

impl CssCounterChange {
    #[must_use]
    pub const fn new(name: CssCounterName, value: Option<i32>) -> Self {
        Self { name, value }
    }

    #[must_use]
    pub const fn name(&self) -> &CssCounterName {
        &self.name
    }

    #[must_use]
    pub const fn value(&self) -> Option<i32> {
        self.value
    }
}

/// A checked authored `flow-tolerance`, before contextual used-value resolution.
///
/// Grid3 permits `normal`, `infinite`, or a signed length-percentage. `normal`
/// remains symbolic because its used value depends on the downstream layout mode.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFlowTolerance {
    value: FlowToleranceValue,
}

#[derive(Clone, Debug, PartialEq)]
enum FlowToleranceValue {
    Normal,
    Infinite,
    LengthPercentage(CssLength),
}

/// Borrows the checked authored branch without resolving relative units or `normal`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFlowToleranceRef<'a> {
    Normal,
    Infinite,
    LengthPercentage(&'a CssLength),
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

    /// Checks a signed authored length-percentage without resolving it.
    ///
    /// Keyword-bearing lengths are rejected; use [`Self::normal`] for `normal`.
    /// Legacy [`CssCalcLength::Sum`] inputs must be nonempty and begin with
    /// [`CssCalcOperator::Add`] at every nested sum. A leading `Subtract` cannot
    /// be serialized faithfully by that legacy model. Signed first operands,
    /// later subtraction, and checked typed calculations remain supported; this
    /// restriction does not prohibit negative authored CSS mathematics.
    #[must_use]
    pub fn try_length_percentage(value: CssLength) -> Option<Self> {
        let valid = match &value {
            CssLength::Px(_)
            | CssLength::Dimension(_)
            | CssLength::Percent(_)
            | CssLength::Zero => true,
            CssLength::Calc(calc) => calc_has_valid_legacy_shape(calc),
            _ => false,
        };
        valid.then_some(Self {
            value: FlowToleranceValue::LengthPercentage(value),
        })
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

/// Checks only legacy calculation shape, without changing symbolic values or origins.
pub(crate) fn length_has_valid_calc_shape(value: &CssLength) -> bool {
    match value {
        CssLength::Calc(calc) => calc_has_valid_legacy_shape(calc),
        _ => true,
    }
}

fn calc_has_valid_legacy_shape(calc: &CssCalcLength) -> bool {
    let mut pending = vec![calc];
    while let Some(calc) = pending.pop() {
        if let CssCalcLength::Sum(terms) = calc {
            if !terms
                .first()
                .is_some_and(|term| term.operator() == CssCalcOperator::Add)
            {
                return false;
            }
            pending.extend(terms.iter().map(CssCalcLengthTerm::value));
        }
    }
    true
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomIdent {
    value: String,
}

impl CssCustomIdent {
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if is_valid_custom_ident(&value) {
            Some(Self::new(value))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

fn is_valid_custom_ident(value: &str) -> bool {
    !value.is_empty()
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "span" | "auto"
        )
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridTrackBreadth {
    Length(CssLength),
    Fraction(CssNonNegativeNumber),
    MinContent,
    MaxContent,
    Auto,
}

impl CssGridTrackBreadth {
    #[must_use]
    pub const fn length(length: CssLength) -> Self {
        Self::Length(length)
    }

    #[must_use]
    pub fn try_fraction(value: f32) -> Option<Self> {
        CssNonNegativeNumber::try_new(value).map(Self::Fraction)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridTrackSize {
    Breadth(CssGridTrackBreadth),
    MinMax {
        min: CssGridTrackBreadth,
        max: CssGridTrackBreadth,
    },
    FitContent(CssLength),
}

impl CssGridTrackSize {
    #[must_use]
    pub const fn breadth(breadth: CssGridTrackBreadth) -> Self {
        Self::Breadth(breadth)
    }

    #[must_use]
    pub const fn minmax(min: CssGridTrackBreadth, max: CssGridTrackBreadth) -> Self {
        Self::MinMax { min, max }
    }

    #[must_use]
    pub const fn fit_content(limit: CssLength) -> Self {
        Self::FitContent(limit)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridLineNames {
    names: Vec<CssCustomIdent>,
}

impl CssGridLineNames {
    /// Creates an authored line-name group, including the valid empty group `[]`.
    #[must_use]
    pub fn try_new(names: Vec<CssCustomIdent>) -> Option<Self> {
        Some(Self::new(names))
    }

    #[must_use]
    pub(crate) fn new(names: Vec<CssCustomIdent>) -> Self {
        Self { names }
    }

    #[must_use]
    pub fn names(&self) -> &[CssCustomIdent] {
        &self.names
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridTrackComponent {
    LineNames(CssGridLineNames),
    TrackSize(CssGridTrackSize),
    Repeat(CssGridRepeat),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssGridTrackList {
    components: Vec<CssGridTrackComponent>,
}

fn grid_tracks_have_valid_calc_shape(components: &[CssGridTrackComponent]) -> bool {
    let valid_breadth = |breadth: &CssGridTrackBreadth| match breadth {
        CssGridTrackBreadth::Length(value) => length_has_valid_calc_shape(value),
        _ => true,
    };
    let mut pending = vec![components];
    while let Some(components) = pending.pop() {
        for component in components {
            let valid = match component {
                CssGridTrackComponent::LineNames(_) => true,
                CssGridTrackComponent::Repeat(repeat) => {
                    pending.push(repeat.tracks().components());
                    true
                }
                CssGridTrackComponent::TrackSize(size) => match size {
                    CssGridTrackSize::Breadth(breadth) => valid_breadth(breadth),
                    CssGridTrackSize::MinMax { min, max } => {
                        valid_breadth(min) && valid_breadth(max)
                    }
                    CssGridTrackSize::FitContent(value) => length_has_valid_calc_shape(value),
                },
            };
            if !valid {
                return false;
            }
        }
    }
    true
}

impl CssGridTrackList {
    #[must_use]
    pub fn try_new(components: Vec<CssGridTrackComponent>) -> Option<Self> {
        if !grid_tracks_have_valid_calc_shape(&components) {
            return None;
        }
        if components.is_empty() {
            None
        } else {
            Some(Self::new(components))
        }
    }

    #[must_use]
    pub(crate) fn new(components: Vec<CssGridTrackComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssGridTrackComponent] {
        &self.components
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridRepeatCount {
    Integer(CssGridRepeatInteger),
    AutoFill,
    AutoFit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssGridRepeatInteger {
    value: i32,
}

impl CssGridRepeatInteger {
    #[must_use]
    pub const fn try_new(value: i32) -> Option<Self> {
        if value > 0 {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

impl CssGridRepeatCount {
    #[must_use]
    pub const fn try_integer(value: i32) -> Option<Self> {
        match CssGridRepeatInteger::try_new(value) {
            Some(value) => Some(Self::Integer(value)),
            None => None,
        }
    }

    #[must_use]
    pub(crate) const fn integer(value: i32) -> Self {
        match Self::try_integer(value) {
            Some(value) => value,
            None => panic!("grid repeat integer must be positive"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssGridRepeat {
    count: CssGridRepeatCount,
    tracks: CssGridTrackList,
}

impl CssGridRepeat {
    #[must_use]
    pub fn try_new(count: CssGridRepeatCount, tracks: CssGridTrackList) -> Option<Self> {
        if tracks.components().is_empty() {
            None
        } else {
            Some(Self::new(count, tracks))
        }
    }

    #[must_use]
    pub(crate) const fn new(count: CssGridRepeatCount, tracks: CssGridTrackList) -> Self {
        Self { count, tracks }
    }

    #[must_use]
    pub const fn count(&self) -> CssGridRepeatCount {
        self.count
    }

    #[must_use]
    pub const fn tracks(&self) -> &CssGridTrackList {
        &self.tracks
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplateAreaCell {
    Empty,
    Named(CssCustomIdent),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridTemplateAreaRow {
    cells: Vec<CssGridTemplateAreaCell>,
}

impl CssGridTemplateAreaRow {
    #[must_use]
    pub fn try_new(cells: Vec<CssGridTemplateAreaCell>) -> Option<Self> {
        if cells.is_empty() {
            None
        } else {
            Some(Self::new(cells))
        }
    }

    #[must_use]
    pub(crate) fn new(cells: Vec<CssGridTemplateAreaCell>) -> Self {
        Self { cells }
    }

    #[must_use]
    pub fn cells(&self) -> &[CssGridTemplateAreaCell] {
        &self.cells
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplateAreas {
    None,
    Rows(CssGridTemplateAreaRows),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridTemplateAreaRows {
    rows: Vec<CssGridTemplateAreaRow>,
}

impl CssGridTemplateAreaRows {
    #[must_use]
    pub fn try_new(rows: Vec<CssGridTemplateAreaRow>) -> Option<Self> {
        if grid_template_area_rows_are_valid(&rows) {
            Some(Self { rows })
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new_unchecked(rows: Vec<CssGridTemplateAreaRow>) -> Self {
        Self { rows }
    }

    #[must_use]
    pub fn rows(&self) -> &[CssGridTemplateAreaRow] {
        &self.rows
    }
}

impl CssGridTemplateAreas {
    #[must_use]
    pub fn try_rows(rows: Vec<CssGridTemplateAreaRow>) -> Option<Self> {
        CssGridTemplateAreaRows::try_new(rows).map(Self::Rows)
    }

    #[must_use]
    pub(crate) fn rows(rows: Vec<CssGridTemplateAreaRow>) -> Self {
        Self::Rows(CssGridTemplateAreaRows::new_unchecked(rows))
    }
}

fn grid_template_area_rows_are_valid(rows: &[CssGridTemplateAreaRow]) -> bool {
    if rows.is_empty() {
        return false;
    }
    let width = rows[0].cells().len();
    if width == 0 || rows.iter().any(|row| row.cells().len() != width) {
        return false;
    }

    let mut bounds = HashMap::<String, GridAreaBounds>::new();
    for (row_index, row) in rows.iter().enumerate() {
        for (col_index, cell) in row.cells().iter().enumerate() {
            let CssGridTemplateAreaCell::Named(name) = cell else {
                continue;
            };
            bounds
                .entry(name.as_str().to_owned())
                .and_modify(|bounds| {
                    bounds.min_row = bounds.min_row.min(row_index);
                    bounds.max_row = bounds.max_row.max(row_index);
                    bounds.min_col = bounds.min_col.min(col_index);
                    bounds.max_col = bounds.max_col.max(col_index);
                    bounds.count += 1;
                })
                .or_insert(GridAreaBounds {
                    min_row: row_index,
                    max_row: row_index,
                    min_col: col_index,
                    max_col: col_index,
                    count: 1,
                });
        }
    }

    bounds.into_values().all(|bounds| {
        let rectangle_area =
            (bounds.max_row - bounds.min_row + 1) * (bounds.max_col - bounds.min_col + 1);
        rectangle_area == bounds.count
    })
}

#[derive(Clone, Copy)]
struct GridAreaBounds {
    min_row: usize,
    max_row: usize,
    min_col: usize,
    max_col: usize,
    count: usize,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGridTemplate {
    None,
    RowsColumns {
        rows: CssGridTrackList,
        columns: Option<CssGridTrackList>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridAutoFlowAxis {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssGridAutoFlow {
    axis: CssGridAutoFlowAxis,
    dense: bool,
}

impl CssGridAutoFlow {
    #[must_use]
    pub const fn new(axis: CssGridAutoFlowAxis, dense: bool) -> Self {
        Self { axis, dense }
    }

    #[must_use]
    pub const fn axis(self) -> CssGridAutoFlowAxis {
        self.axis
    }

    #[must_use]
    pub const fn dense(self) -> bool {
        self.dense
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridLine {
    Auto,
    Integer(CssGridLineInteger),
    CustomIdent(CssCustomIdent),
    Span(CssGridLineSpan),
}

impl CssGridLine {
    #[must_use]
    pub fn try_integer(value: i32) -> Option<Self> {
        CssGridLineInteger::try_new(value).map(Self::Integer)
    }

    #[must_use]
    pub(crate) fn integer(value: i32) -> Self {
        match Self::try_integer(value) {
            Some(value) => value,
            None => panic!("grid line integer must be non-zero"),
        }
    }

    #[must_use]
    pub fn try_span(integer: Option<i32>, name: Option<CssCustomIdent>) -> Option<Self> {
        CssGridLineSpan::try_new(integer, name).map(Self::Span)
    }

    #[must_use]
    pub(crate) fn span(integer: Option<i32>, name: Option<CssCustomIdent>) -> Self {
        Self::Span(CssGridLineSpan::new(integer, name))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssGridLineInteger {
    value: i32,
}

impl CssGridLineInteger {
    #[must_use]
    pub const fn try_new(value: i32) -> Option<Self> {
        if value != 0 {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridLineSpan {
    integer: Option<CssGridSpanInteger>,
    name: Option<CssCustomIdent>,
}

impl CssGridLineSpan {
    #[must_use]
    pub fn try_new(integer: Option<i32>, name: Option<CssCustomIdent>) -> Option<Self> {
        let integer = match integer {
            Some(value) => Some(CssGridSpanInteger::try_new(value)?),
            None => None,
        };
        if integer.is_none() && name.is_none() {
            None
        } else {
            Some(Self { integer, name })
        }
    }

    #[must_use]
    pub(crate) fn new(integer: Option<i32>, name: Option<CssCustomIdent>) -> Self {
        match Self::try_new(integer, name) {
            Some(value) => value,
            None => panic!("grid span must include a positive integer or name"),
        }
    }

    #[must_use]
    pub const fn integer(&self) -> Option<i32> {
        match self.integer {
            Some(value) => Some(value.value()),
            None => None,
        }
    }

    #[must_use]
    pub const fn name(&self) -> Option<&CssCustomIdent> {
        self.name.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssGridSpanInteger {
    value: i32,
}

impl CssGridSpanInteger {
    #[must_use]
    pub const fn try_new(value: i32) -> Option<Self> {
        if value > 0 {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridLineRange {
    start: CssGridLine,
    end: Option<CssGridLine>,
}

impl CssGridLineRange {
    #[must_use]
    pub const fn new(start: CssGridLine, end: Option<CssGridLine>) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn start(&self) -> &CssGridLine {
        &self.start
    }

    #[must_use]
    pub const fn end(&self) -> Option<&CssGridLine> {
        self.end.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGridArea {
    row_start: CssGridLine,
    column_start: Option<CssGridLine>,
    row_end: Option<CssGridLine>,
    column_end: Option<CssGridLine>,
}

impl CssGridArea {
    #[must_use]
    pub const fn new(
        row_start: CssGridLine,
        column_start: Option<CssGridLine>,
        row_end: Option<CssGridLine>,
        column_end: Option<CssGridLine>,
    ) -> Self {
        Self {
            row_start,
            column_start,
            row_end,
            column_end,
        }
    }

    #[must_use]
    pub const fn row_start(&self) -> &CssGridLine {
        &self.row_start
    }

    #[must_use]
    pub const fn column_start(&self) -> Option<&CssGridLine> {
        self.column_start.as_ref()
    }

    #[must_use]
    pub const fn row_end(&self) -> Option<&CssGridLine> {
        self.row_end.as_ref()
    }

    #[must_use]
    pub const fn column_end(&self) -> Option<&CssGridLine> {
        self.column_end.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGrid {
    Template(CssGridTemplate),
    AutoFlow {
        flow: CssGridAutoFlow,
        auto_tracks: Option<CssGridTrackList>,
        explicit_tracks: CssGridTrackList,
    },
}

/// The semantic branch of a parser-owned current Grid track breadth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridTrackBreadthKind {
    Length,
    Fraction,
    MinContent,
    MaxContent,
    Auto,
}

/// A parser-owned current authored Grid track breadth.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackBreadth {
    representation: CssAuthoredGridTrackBreadthRepresentation,
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridTrackBreadthRepresentation {
    Length(CssLength),
    Fraction(CssNonNegativeNumber),
    MinContent,
    MaxContent,
    Auto,
}

impl CssAuthoredGridTrackBreadth {
    pub(crate) const fn from_length(value: CssLength) -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::Length(value),
        }
    }

    pub(crate) const fn from_fraction(value: CssNonNegativeNumber) -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::Fraction(value),
        }
    }

    pub(crate) const fn min_content() -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::MinContent,
        }
    }

    pub(crate) const fn max_content() -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::MaxContent,
        }
    }

    pub(crate) const fn auto() -> Self {
        Self {
            representation: CssAuthoredGridTrackBreadthRepresentation::Auto,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssAuthoredGridTrackBreadthKind {
        match self.representation {
            CssAuthoredGridTrackBreadthRepresentation::Length(_) => {
                CssAuthoredGridTrackBreadthKind::Length
            }
            CssAuthoredGridTrackBreadthRepresentation::Fraction(_) => {
                CssAuthoredGridTrackBreadthKind::Fraction
            }
            CssAuthoredGridTrackBreadthRepresentation::MinContent => {
                CssAuthoredGridTrackBreadthKind::MinContent
            }
            CssAuthoredGridTrackBreadthRepresentation::MaxContent => {
                CssAuthoredGridTrackBreadthKind::MaxContent
            }
            CssAuthoredGridTrackBreadthRepresentation::Auto => {
                CssAuthoredGridTrackBreadthKind::Auto
            }
        }
    }

    #[must_use]
    pub const fn length(&self) -> Option<&CssLength> {
        match &self.representation {
            CssAuthoredGridTrackBreadthRepresentation::Length(value) => Some(value),
            CssAuthoredGridTrackBreadthRepresentation::Fraction(_)
            | CssAuthoredGridTrackBreadthRepresentation::MinContent
            | CssAuthoredGridTrackBreadthRepresentation::MaxContent
            | CssAuthoredGridTrackBreadthRepresentation::Auto => None,
        }
    }

    #[must_use]
    pub const fn fraction(&self) -> Option<CssNonNegativeNumber> {
        match self.representation {
            CssAuthoredGridTrackBreadthRepresentation::Fraction(value) => Some(value),
            CssAuthoredGridTrackBreadthRepresentation::Length(_)
            | CssAuthoredGridTrackBreadthRepresentation::MinContent
            | CssAuthoredGridTrackBreadthRepresentation::MaxContent
            | CssAuthoredGridTrackBreadthRepresentation::Auto => None,
        }
    }

    pub(crate) fn i01_projection(&self) -> Option<CssGridTrackBreadth> {
        Some(match &self.representation {
            CssAuthoredGridTrackBreadthRepresentation::Length(value) => {
                if matches!(value, CssLength::Calc(CssCalcLength::Typed(_))) {
                    return None;
                }
                CssGridTrackBreadth::length(value.clone())
            }
            CssAuthoredGridTrackBreadthRepresentation::Fraction(value) => {
                CssGridTrackBreadth::Fraction(*value)
            }
            CssAuthoredGridTrackBreadthRepresentation::MinContent => {
                CssGridTrackBreadth::MinContent
            }
            CssAuthoredGridTrackBreadthRepresentation::MaxContent => {
                CssGridTrackBreadth::MaxContent
            }
            CssAuthoredGridTrackBreadthRepresentation::Auto => CssGridTrackBreadth::Auto,
        })
    }

    pub(crate) const fn is_fixed(&self) -> bool {
        matches!(
            self.representation,
            CssAuthoredGridTrackBreadthRepresentation::Length(_)
        )
    }

    pub(crate) const fn is_inflexible(&self) -> bool {
        !matches!(
            self.representation,
            CssAuthoredGridTrackBreadthRepresentation::Fraction(_)
        )
    }
}

/// The semantic branch of a current authored Grid track size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridTrackSizeKind {
    Breadth,
    MinMax,
    FitContent,
}

/// A parser-owned current authored Grid track size.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackSize {
    representation: CssAuthoredGridTrackSizeRepresentation,
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridTrackSizeRepresentation {
    Breadth(CssAuthoredGridTrackBreadth),
    MinMax {
        min: CssAuthoredGridTrackBreadth,
        max: CssAuthoredGridTrackBreadth,
    },
    FitContent(CssLength),
}

impl CssAuthoredGridTrackSize {
    pub(crate) const fn from_breadth(value: CssAuthoredGridTrackBreadth) -> Self {
        Self {
            representation: CssAuthoredGridTrackSizeRepresentation::Breadth(value),
        }
    }

    pub(crate) const fn from_minmax(
        min: CssAuthoredGridTrackBreadth,
        max: CssAuthoredGridTrackBreadth,
    ) -> Self {
        Self {
            representation: CssAuthoredGridTrackSizeRepresentation::MinMax { min, max },
        }
    }

    pub(crate) const fn from_fit_content(value: CssLength) -> Self {
        Self {
            representation: CssAuthoredGridTrackSizeRepresentation::FitContent(value),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CssAuthoredGridTrackSizeKind {
        match self.representation {
            CssAuthoredGridTrackSizeRepresentation::Breadth(_) => {
                CssAuthoredGridTrackSizeKind::Breadth
            }
            CssAuthoredGridTrackSizeRepresentation::MinMax { .. } => {
                CssAuthoredGridTrackSizeKind::MinMax
            }
            CssAuthoredGridTrackSizeRepresentation::FitContent(_) => {
                CssAuthoredGridTrackSizeKind::FitContent
            }
        }
    }

    #[must_use]
    pub const fn breadth(&self) -> Option<&CssAuthoredGridTrackBreadth> {
        match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::Breadth(value) => Some(value),
            CssAuthoredGridTrackSizeRepresentation::MinMax { .. }
            | CssAuthoredGridTrackSizeRepresentation::FitContent(_) => None,
        }
    }

    #[must_use]
    pub const fn minmax(
        &self,
    ) -> Option<(&CssAuthoredGridTrackBreadth, &CssAuthoredGridTrackBreadth)> {
        match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::MinMax { min, max } => Some((min, max)),
            CssAuthoredGridTrackSizeRepresentation::Breadth(_)
            | CssAuthoredGridTrackSizeRepresentation::FitContent(_) => None,
        }
    }

    #[must_use]
    pub const fn fit_content(&self) -> Option<&CssLength> {
        match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::FitContent(value) => Some(value),
            CssAuthoredGridTrackSizeRepresentation::Breadth(_)
            | CssAuthoredGridTrackSizeRepresentation::MinMax { .. } => None,
        }
    }

    pub(crate) fn i01_projection(&self) -> Option<CssGridTrackSize> {
        Some(match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::Breadth(value) => {
                CssGridTrackSize::breadth(value.i01_projection()?)
            }
            CssAuthoredGridTrackSizeRepresentation::MinMax { min, max } => {
                CssGridTrackSize::minmax(min.i01_projection()?, max.i01_projection()?)
            }
            CssAuthoredGridTrackSizeRepresentation::FitContent(value) => {
                if matches!(value, CssLength::Calc(CssCalcLength::Typed(_))) {
                    return None;
                }
                CssGridTrackSize::fit_content(value.clone())
            }
        })
    }

    pub(crate) const fn is_fixed(&self) -> bool {
        match &self.representation {
            CssAuthoredGridTrackSizeRepresentation::Breadth(value) => value.is_fixed(),
            CssAuthoredGridTrackSizeRepresentation::MinMax { min, max } => {
                min.is_fixed() || (min.is_inflexible() && max.is_fixed())
            }
            CssAuthoredGridTrackSizeRepresentation::FitContent(_) => false,
        }
    }
}

/// A fixed size admitted around automatic repetition or in an integer fixed-repeat.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridFixedSize {
    size: CssAuthoredGridTrackSize,
}

impl CssAuthoredGridFixedSize {
    pub(crate) const fn new(size: CssAuthoredGridTrackSize) -> Self {
        Self { size }
    }

    #[must_use]
    pub const fn size(&self) -> &CssAuthoredGridTrackSize {
        &self.size
    }
}

/// One non-recursive member of integer or automatic track-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridTrackRepeatComponent {
    LineNames(CssGridLineNames),
    TrackSize(CssAuthoredGridTrackSize),
}

/// Non-empty, non-recursive integer or automatic track-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackRepeatContent {
    components: Vec<CssAuthoredGridTrackRepeatComponent>,
}

impl CssAuthoredGridTrackRepeatContent {
    pub(crate) const fn new(components: Vec<CssAuthoredGridTrackRepeatComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssAuthoredGridTrackRepeatComponent] {
        &self.components
    }
}

/// One non-recursive member of fixed-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridFixedRepeatComponent {
    LineNames(CssGridLineNames),
    FixedSize(CssAuthoredGridFixedSize),
}

/// Non-empty, non-recursive fixed-repeat content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridFixedRepeatContent {
    components: Vec<CssAuthoredGridFixedRepeatComponent>,
}

impl CssAuthoredGridFixedRepeatContent {
    pub(crate) const fn new(components: Vec<CssAuthoredGridFixedRepeatComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssAuthoredGridFixedRepeatComponent] {
        &self.components
    }
}

/// A positive-integer repetition whose content may use any track size.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridIntegerTrackRepeat {
    count: CssGridRepeatInteger,
    content: CssAuthoredGridTrackRepeatContent,
}

impl CssAuthoredGridIntegerTrackRepeat {
    pub(crate) const fn new(
        count: CssGridRepeatInteger,
        content: CssAuthoredGridTrackRepeatContent,
    ) -> Self {
        Self { count, content }
    }

    #[must_use]
    pub const fn count(&self) -> CssGridRepeatInteger {
        self.count
    }

    #[must_use]
    pub const fn content(&self) -> &CssAuthoredGridTrackRepeatContent {
        &self.content
    }
}

/// A positive-integer repetition constrained to fixed-size content.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridIntegerFixedRepeat {
    count: CssGridRepeatInteger,
    content: CssAuthoredGridFixedRepeatContent,
}

impl CssAuthoredGridIntegerFixedRepeat {
    pub(crate) const fn new(
        count: CssGridRepeatInteger,
        content: CssAuthoredGridFixedRepeatContent,
    ) -> Self {
        Self { count, content }
    }

    #[must_use]
    pub const fn count(&self) -> CssGridRepeatInteger {
        self.count
    }

    #[must_use]
    pub const fn content(&self) -> &CssAuthoredGridFixedRepeatContent {
        &self.content
    }
}

/// The automatic repetition mode in an authored Grid track list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridAutoRepeatKind {
    AutoFill,
    AutoFit,
}

/// The single automatic repetition admitted by an auto track list.
///
/// Its non-recursive body admits general track sizes under CSS Grid 3. Surrounding
/// tracks and integer repetitions in the same list remain constrained to fixed sizes.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridAutoRepeat {
    kind: CssAuthoredGridAutoRepeatKind,
    content: CssAuthoredGridTrackRepeatContent,
}

impl CssAuthoredGridAutoRepeat {
    pub(crate) const fn new(
        kind: CssAuthoredGridAutoRepeatKind,
        content: CssAuthoredGridTrackRepeatContent,
    ) -> Self {
        Self { kind, content }
    }

    #[must_use]
    pub const fn kind(&self) -> CssAuthoredGridAutoRepeatKind {
        self.kind
    }

    #[must_use]
    pub const fn content(&self) -> &CssAuthoredGridTrackRepeatContent {
        &self.content
    }
}

/// One component of a general track list, which never contains automatic repetition.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridGeneralTrackComponent {
    LineNames(CssGridLineNames),
    TrackSize(CssAuthoredGridTrackSize),
    Repeat(CssAuthoredGridIntegerTrackRepeat),
}

/// A non-empty general Grid track list.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridGeneralTrackList {
    components: Vec<CssAuthoredGridGeneralTrackComponent>,
}

impl CssAuthoredGridGeneralTrackList {
    pub(crate) const fn new(components: Vec<CssAuthoredGridGeneralTrackComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssAuthoredGridGeneralTrackComponent] {
        &self.components
    }
}

/// One component of an auto track list.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredGridAutoTrackComponent {
    LineNames(CssGridLineNames),
    FixedSize(CssAuthoredGridFixedSize),
    Repeat(CssAuthoredGridIntegerFixedRepeat),
    AutoRepeat(CssAuthoredGridAutoRepeat),
}

/// A Grid track list containing exactly one automatic repetition.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridAutoTrackList {
    components: Vec<CssAuthoredGridAutoTrackComponent>,
}

impl CssAuthoredGridAutoTrackList {
    pub(crate) const fn new(components: Vec<CssAuthoredGridAutoTrackComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssAuthoredGridAutoTrackComponent] {
        &self.components
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridTrackListRepresentation {
    General(CssAuthoredGridGeneralTrackList),
    Auto(CssAuthoredGridAutoTrackList),
}

/// A parser-owned current Grid track list, classified as general or automatic.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackList {
    representation: CssAuthoredGridTrackListRepresentation,
}

impl CssAuthoredGridTrackList {
    pub(crate) const fn general(value: CssAuthoredGridGeneralTrackList) -> Self {
        Self {
            representation: CssAuthoredGridTrackListRepresentation::General(value),
        }
    }

    pub(crate) const fn auto(value: CssAuthoredGridAutoTrackList) -> Self {
        Self {
            representation: CssAuthoredGridTrackListRepresentation::Auto(value),
        }
    }

    #[must_use]
    pub const fn general_list(&self) -> Option<&CssAuthoredGridGeneralTrackList> {
        match &self.representation {
            CssAuthoredGridTrackListRepresentation::General(value) => Some(value),
            CssAuthoredGridTrackListRepresentation::Auto(_) => None,
        }
    }

    #[must_use]
    pub const fn auto_list(&self) -> Option<&CssAuthoredGridAutoTrackList> {
        match &self.representation {
            CssAuthoredGridTrackListRepresentation::Auto(value) => Some(value),
            CssAuthoredGridTrackListRepresentation::General(_) => None,
        }
    }
}

/// A non-empty authored list for `grid-auto-rows` or `grid-auto-columns`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTrackSizeList {
    sizes: Vec<CssAuthoredGridTrackSize>,
}

impl CssAuthoredGridTrackSizeList {
    pub(crate) const fn new(sizes: Vec<CssAuthoredGridTrackSize>) -> Self {
        Self { sizes }
    }

    #[must_use]
    pub fn sizes(&self) -> &[CssAuthoredGridTrackSize] {
        &self.sizes
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridTemplateRepresentation {
    None,
    RowsColumns {
        rows: CssAuthoredGridTrackList,
        columns: Option<CssAuthoredGridTrackList>,
    },
}

/// The parser-owned current authored `grid-template` aggregate.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridTemplateValue {
    representation: CssAuthoredGridTemplateRepresentation,
}

impl CssAuthoredGridTemplateValue {
    pub(crate) const fn none() -> Self {
        Self {
            representation: CssAuthoredGridTemplateRepresentation::None,
        }
    }

    pub(crate) const fn rows_columns(
        rows: CssAuthoredGridTrackList,
        columns: Option<CssAuthoredGridTrackList>,
    ) -> Self {
        Self {
            representation: CssAuthoredGridTemplateRepresentation::RowsColumns { rows, columns },
        }
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(
            self.representation,
            CssAuthoredGridTemplateRepresentation::None
        )
    }

    #[must_use]
    pub const fn rows(&self) -> Option<&CssAuthoredGridTrackList> {
        match &self.representation {
            CssAuthoredGridTemplateRepresentation::RowsColumns { rows, .. } => Some(rows),
            CssAuthoredGridTemplateRepresentation::None => None,
        }
    }

    #[must_use]
    pub const fn columns(&self) -> Option<&CssAuthoredGridTrackList> {
        match &self.representation {
            CssAuthoredGridTemplateRepresentation::RowsColumns { columns, .. } => columns.as_ref(),
            CssAuthoredGridTemplateRepresentation::None => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredGridRepresentation {
    Template(CssAuthoredGridTemplateValue),
    AutoFlow {
        flow: CssGridAutoFlow,
        auto_tracks: Option<CssAuthoredGridTrackSizeList>,
        explicit_tracks: CssAuthoredGridTrackList,
    },
}

/// The parser-owned current authored `grid` aggregate.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CssAuthoredGridValue {
    representation: CssAuthoredGridRepresentation,
}

impl CssAuthoredGridValue {
    pub(crate) const fn template(value: CssAuthoredGridTemplateValue) -> Self {
        Self {
            representation: CssAuthoredGridRepresentation::Template(value),
        }
    }

    pub(crate) const fn from_auto_flow(
        flow: CssGridAutoFlow,
        auto_tracks: Option<CssAuthoredGridTrackSizeList>,
        explicit_tracks: CssAuthoredGridTrackList,
    ) -> Self {
        Self {
            representation: CssAuthoredGridRepresentation::AutoFlow {
                flow,
                auto_tracks,
                explicit_tracks,
            },
        }
    }

    #[must_use]
    pub const fn template_value(&self) -> Option<&CssAuthoredGridTemplateValue> {
        match &self.representation {
            CssAuthoredGridRepresentation::Template(value) => Some(value),
            CssAuthoredGridRepresentation::AutoFlow { .. } => None,
        }
    }

    #[must_use]
    pub const fn auto_flow(&self) -> Option<CssGridAutoFlow> {
        match self.representation {
            CssAuthoredGridRepresentation::AutoFlow { flow, .. } => Some(flow),
            CssAuthoredGridRepresentation::Template(_) => None,
        }
    }

    #[must_use]
    pub const fn auto_tracks(&self) -> Option<&CssAuthoredGridTrackSizeList> {
        match &self.representation {
            CssAuthoredGridRepresentation::AutoFlow { auto_tracks, .. } => auto_tracks.as_ref(),
            CssAuthoredGridRepresentation::Template(_) => None,
        }
    }

    #[must_use]
    pub const fn explicit_tracks(&self) -> Option<&CssAuthoredGridTrackList> {
        match &self.representation {
            CssAuthoredGridRepresentation::AutoFlow {
                explicit_tracks, ..
            } => Some(explicit_tracks),
            CssAuthoredGridRepresentation::Template(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedGridTrackList {
    current: CssAuthoredGridTrackList,
    i01_subset: Option<CssGridTrackList>,
}

impl CssParsedGridTrackList {
    pub(crate) const fn new(
        current: CssAuthoredGridTrackList,
        i01_subset: Option<CssGridTrackList>,
    ) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredGridTrackList, Option<CssGridTrackList>) {
        (self.current, self.i01_subset)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedGridTrackSizeList {
    current: CssAuthoredGridTrackSizeList,
    i01_subset: Option<CssGridTrackList>,
}

impl CssParsedGridTrackSizeList {
    pub(crate) const fn new(
        current: CssAuthoredGridTrackSizeList,
        i01_subset: Option<CssGridTrackList>,
    ) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredGridTrackSizeList, Option<CssGridTrackList>) {
        (self.current, self.i01_subset)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedGridTemplate {
    current: CssAuthoredGridTemplateValue,
    i01_subset: Option<CssGridTemplate>,
}

impl CssParsedGridTemplate {
    pub(crate) const fn new(
        current: CssAuthoredGridTemplateValue,
        i01_subset: Option<CssGridTemplate>,
    ) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredGridTemplateValue, Option<CssGridTemplate>) {
        (self.current, self.i01_subset)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedGrid {
    current: CssAuthoredGridValue,
    i01_subset: Option<CssGrid>,
}

impl CssParsedGrid {
    pub(crate) const fn new(current: CssAuthoredGridValue, i01_subset: Option<CssGrid>) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredGridValue, Option<CssGrid>) {
        (self.current, self.i01_subset)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOrder {
    Integer(i32),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFlex {
    None,
    Auto,
    Components {
        grow: CssFlexFactor,
        shrink: Option<CssFlexFactor>,
        basis: Option<CssLength>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssFlexComponents {
    grow: CssNonNegativeNumberValue,
    shrink: Option<CssNonNegativeNumberValue>,
    basis: Option<CssLength>,
}

impl CssFlexComponents {
    #[must_use]
    pub(crate) const fn new(
        grow: CssNonNegativeNumberValue,
        shrink: Option<CssNonNegativeNumberValue>,
        basis: Option<CssLength>,
    ) -> Self {
        Self {
            grow,
            shrink,
            basis,
        }
    }

    #[must_use]
    pub const fn grow(&self) -> &CssNonNegativeNumberValue {
        &self.grow
    }

    #[must_use]
    pub const fn shrink(&self) -> Option<&CssNonNegativeNumberValue> {
        self.shrink.as_ref()
    }

    #[must_use]
    pub const fn basis(&self) -> Option<&CssLength> {
        self.basis.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFlexValue {
    None,
    Auto,
    Components(CssFlexComponents),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssIntegerValue {
    Literal(i32),
    Calculation(CssIntegerCalculation),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssZIndexValue {
    Auto,
    Integer(CssIntegerValue),
}

impl CssFlex {
    #[must_use]
    pub const fn components(
        grow: CssFlexFactor,
        shrink: Option<CssFlexFactor>,
        basis: Option<CssLength>,
    ) -> Self {
        Self::Components {
            grow,
            shrink,
            basis,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssZIndex {
    Auto,
    Integer(i32),
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
    Color(Box<CssAuthoredColor>),
}

/// A checked authored `outline-offset` length.
#[derive(Clone, Debug, PartialEq)]
pub struct CssOutlineOffset {
    value: CssLength,
}

impl CssOutlineOffset {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        is_absolute_length(&value).then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// The authored `resize` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssResize {
    None,
    Both,
    Horizontal,
    Vertical,
}

/// One independently authored Containment 1 containment kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContainComponent {
    Size,
    Layout,
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

/// The authored Containment 1 `contain` value.
///
/// This is syntax only and does not apply size, layout, or paint containment.
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

/// A checked, non-negative authored `border-spacing` length.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderSpacingLength {
    value: CssLength,
}

impl CssBorderSpacingLength {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        is_non_negative_absolute_length(&value).then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// The checked horizontal and vertical authored `border-spacing` lengths.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderSpacing {
    horizontal: CssBorderSpacingLength,
    vertical: CssBorderSpacingLength,
}

impl CssBorderSpacing {
    #[must_use]
    pub fn try_new(horizontal: CssLength, vertical: CssLength) -> Option<Self> {
        Some(Self {
            horizontal: CssBorderSpacingLength::try_new(horizontal)?,
            vertical: CssBorderSpacingLength::try_new(vertical)?,
        })
    }

    #[must_use]
    pub const fn horizontal(&self) -> &CssBorderSpacingLength {
        &self.horizontal
    }

    #[must_use]
    pub const fn vertical(&self) -> &CssBorderSpacingLength {
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

/// A checked authored CSS2 `clip` rectangle length.
#[derive(Clone, Debug, PartialEq)]
pub struct CssClipLength {
    value: CssLength,
}

impl CssClipLength {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        is_absolute_length(&value).then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// One authored edge of a CSS2 `clip: rect(...)` value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssClipEdge {
    Auto,
    Length(CssClipLength),
}

/// The four authored edges of a CSS2 clipping rectangle.
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

/// The authored CSS2 `clip` value.
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

#[derive(Clone, Debug, PartialEq)]
enum CssPageLineMinimumValue {
    Literal(i32),
    Calculation(CssIntegerCalculation),
}

/// A positive authored `orphans` or `widows` count, or a symbolic integer calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageLineMinimum {
    value: CssPageLineMinimumValue,
}

impl CssPageLineMinimum {
    #[must_use]
    pub const fn try_literal(value: i32) -> Option<Self> {
        if value > 0 {
            Some(Self {
                value: CssPageLineMinimumValue::Literal(value),
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn from_calculation(value: CssIntegerCalculation) -> Self {
        Self {
            value: CssPageLineMinimumValue::Calculation(value),
        }
    }

    #[must_use]
    pub const fn literal(&self) -> Option<i32> {
        match self.value {
            CssPageLineMinimumValue::Literal(value) => Some(value),
            CssPageLineMinimumValue::Calculation(_) => None,
        }
    }

    #[must_use]
    pub const fn calculation(&self) -> Option<&CssIntegerCalculation> {
        match &self.value {
            CssPageLineMinimumValue::Literal(_) => None,
            CssPageLineMinimumValue::Calculation(value) => Some(value),
        }
    }
}

/// The authored CSS2 `page-break-before` or `page-break-after` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPageBreak {
    Auto,
    Always,
    Avoid,
    Left,
    Right,
}

/// The authored CSS2 `page-break-inside` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPageBreakInside {
    Auto,
    Avoid,
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

/// The authored CSS2 `quotes` value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssQuotes {
    None,
    Pairs(CssQuotePairList),
}

/// The authored `table-layout` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTableLayout {
    Auto,
    Fixed,
}

/// A checked authored `word-spacing` length.
#[derive(Clone, Debug, PartialEq)]
pub struct CssWordSpacingLength {
    value: CssLength,
}

impl CssWordSpacingLength {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        is_absolute_length(&value).then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// The authored CSS2 `word-spacing` value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssWordSpacing {
    Normal,
    Length(CssWordSpacingLength),
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

/// The authored `text-combine-upright` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextCombineUpright {
    None,
    All,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextAlignLast {
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTextIndent {
    length: CssLength,
    hanging: bool,
    each_line: bool,
}

impl CssTextIndent {
    #[must_use]
    pub fn try_new(length: CssLength, hanging: bool, each_line: bool) -> Option<Self> {
        if !length_has_valid_calc_shape(&length) {
            return None;
        }
        if is_text_length(&length) {
            Some(Self::new(length, hanging, each_line))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(length: CssLength, hanging: bool, each_line: bool) -> Self {
        Self {
            length,
            hanging,
            each_line,
        }
    }

    #[must_use]
    pub const fn length(&self) -> &CssLength {
        &self.length
    }

    #[must_use]
    pub const fn hanging(&self) -> bool {
        self.hanging
    }

    #[must_use]
    pub const fn each_line(&self) -> bool {
        self.each_line
    }
}

#[derive(Clone, Debug, PartialEq)]
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
    Length(CssVerticalAlignLength),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssVerticalAlignLength {
    length: CssLength,
}

impl CssVerticalAlignLength {
    #[must_use]
    pub fn try_new(length: CssLength) -> Option<Self> {
        if !length_has_valid_calc_shape(&length) {
            return None;
        }
        if is_vertical_align_length(&length) {
            Some(Self::new(length))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(length: CssLength) -> Self {
        Self { length }
    }

    #[must_use]
    pub const fn length(&self) -> &CssLength {
        &self.length
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
pub enum CssFontWeight {
    Normal,
    Bold,
    Bolder,
    Lighter,
    Number(CssFontWeightNumber),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontWeightNumber {
    value: i32,
}

impl CssFontWeightNumber {
    #[must_use]
    pub const fn try_new(value: i32) -> Option<Self> {
        if value >= 1 && value <= 1000 {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(value: i32) -> Self {
        match Self::try_new(value) {
            Some(value) => value,
            None => panic!("font weight number must be between 1 and 1000"),
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontStretch {
    Normal,
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariant {
    Normal,
    SmallCaps,
}

/// The current authored `font-variant-caps` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantCaps {
    Normal,
    SmallCaps,
    AllSmallCaps,
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
}

/// Whether one authored ligature group is enabled or disabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantLigatureState {
    Enabled,
    Disabled,
}

/// A nonempty checked set of authored ligature-group choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontVariantLigatureValues {
    common: Option<CssFontVariantLigatureState>,
    discretionary: Option<CssFontVariantLigatureState>,
    historical: Option<CssFontVariantLigatureState>,
    contextual: Option<CssFontVariantLigatureState>,
}

impl CssFontVariantLigatureValues {
    #[must_use]
    pub const fn try_new(
        common: Option<CssFontVariantLigatureState>,
        discretionary: Option<CssFontVariantLigatureState>,
        historical: Option<CssFontVariantLigatureState>,
        contextual: Option<CssFontVariantLigatureState>,
    ) -> Option<Self> {
        if common.is_some()
            || discretionary.is_some()
            || historical.is_some()
            || contextual.is_some()
        {
            Some(Self {
                common,
                discretionary,
                historical,
                contextual,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn common(self) -> Option<CssFontVariantLigatureState> {
        self.common
    }

    #[must_use]
    pub const fn discretionary(self) -> Option<CssFontVariantLigatureState> {
        self.discretionary
    }

    #[must_use]
    pub const fn historical(self) -> Option<CssFontVariantLigatureState> {
        self.historical
    }

    #[must_use]
    pub const fn contextual(self) -> Option<CssFontVariantLigatureState> {
        self.contextual
    }
}

/// The current authored `font-variant-ligatures` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantLigatures {
    Normal,
    None,
    Values(CssFontVariantLigatureValues),
}

/// The figure form selected by `font-variant-numeric`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumericFigure {
    LiningNums,
    OldstyleNums,
}

/// The figure spacing selected by `font-variant-numeric`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumericSpacing {
    ProportionalNums,
    TabularNums,
}

/// The fraction form selected by `font-variant-numeric`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumericFraction {
    DiagonalFractions,
    StackedFractions,
}

/// A nonempty checked set of authored numeric-variant choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontVariantNumericValues {
    figure: Option<CssFontVariantNumericFigure>,
    spacing: Option<CssFontVariantNumericSpacing>,
    fraction: Option<CssFontVariantNumericFraction>,
    ordinal: bool,
    slashed_zero: bool,
}

impl CssFontVariantNumericValues {
    #[must_use]
    pub const fn try_new(
        figure: Option<CssFontVariantNumericFigure>,
        spacing: Option<CssFontVariantNumericSpacing>,
        fraction: Option<CssFontVariantNumericFraction>,
        ordinal: bool,
        slashed_zero: bool,
    ) -> Option<Self> {
        if figure.is_some() || spacing.is_some() || fraction.is_some() || ordinal || slashed_zero {
            Some(Self {
                figure,
                spacing,
                fraction,
                ordinal,
                slashed_zero,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn figure(self) -> Option<CssFontVariantNumericFigure> {
        self.figure
    }

    #[must_use]
    pub const fn spacing(self) -> Option<CssFontVariantNumericSpacing> {
        self.spacing
    }

    #[must_use]
    pub const fn fraction(self) -> Option<CssFontVariantNumericFraction> {
        self.fraction
    }

    #[must_use]
    pub const fn ordinal(self) -> bool {
        self.ordinal
    }

    #[must_use]
    pub const fn slashed_zero(self) -> bool {
        self.slashed_zero
    }
}

/// The current authored `font-variant-numeric` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumeric {
    Normal,
    Values(CssFontVariantNumericValues),
}

/// The regional glyph form selected by `font-variant-east-asian`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEastAsianVariant {
    Jis78,
    Jis83,
    Jis90,
    Jis04,
    Simplified,
    Traditional,
}

/// The glyph-width form selected by `font-variant-east-asian`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEastAsianWidth {
    FullWidth,
    ProportionalWidth,
}

/// A nonempty checked set of authored East Asian variant choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontVariantEastAsianValues {
    variant: Option<CssFontVariantEastAsianVariant>,
    width: Option<CssFontVariantEastAsianWidth>,
    ruby: bool,
}

impl CssFontVariantEastAsianValues {
    #[must_use]
    pub const fn try_new(
        variant: Option<CssFontVariantEastAsianVariant>,
        width: Option<CssFontVariantEastAsianWidth>,
        ruby: bool,
    ) -> Option<Self> {
        if variant.is_some() || width.is_some() || ruby {
            Some(Self {
                variant,
                width,
                ruby,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn variant(self) -> Option<CssFontVariantEastAsianVariant> {
        self.variant
    }

    #[must_use]
    pub const fn width(self) -> Option<CssFontVariantEastAsianWidth> {
        self.width
    }

    #[must_use]
    pub const fn ruby(self) -> bool {
        self.ruby
    }
}

/// The current authored `font-variant-east-asian` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEastAsian {
    Normal,
    Values(CssFontVariantEastAsianValues),
}

/// The current authored `font-variant-position` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantPosition {
    Normal,
    Sub,
    Super,
}

/// A nonempty checked compatible union of `font-variant` component groups.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontVariantValues {
    ligatures: Option<CssFontVariantLigatureValues>,
    position: Option<CssFontVariantPosition>,
    caps: Option<CssFontVariantCaps>,
    numeric: Option<CssFontVariantNumericValues>,
    east_asian: Option<CssFontVariantEastAsianValues>,
}

impl CssFontVariantValues {
    #[must_use]
    pub const fn try_new(
        ligatures: Option<CssFontVariantLigatureValues>,
        position: Option<CssFontVariantPosition>,
        caps: Option<CssFontVariantCaps>,
        numeric: Option<CssFontVariantNumericValues>,
        east_asian: Option<CssFontVariantEastAsianValues>,
    ) -> Option<Self> {
        if matches!(position, Some(CssFontVariantPosition::Normal))
            || matches!(caps, Some(CssFontVariantCaps::Normal))
            || (ligatures.is_none()
                && position.is_none()
                && caps.is_none()
                && numeric.is_none()
                && east_asian.is_none())
        {
            None
        } else {
            Some(Self {
                ligatures,
                position,
                caps,
                numeric,
                east_asian,
            })
        }
    }

    #[must_use]
    pub const fn ligatures(&self) -> Option<&CssFontVariantLigatureValues> {
        self.ligatures.as_ref()
    }

    #[must_use]
    pub const fn position(self) -> Option<CssFontVariantPosition> {
        self.position
    }

    #[must_use]
    pub const fn caps(self) -> Option<CssFontVariantCaps> {
        self.caps
    }

    #[must_use]
    pub const fn numeric(&self) -> Option<&CssFontVariantNumericValues> {
        self.numeric.as_ref()
    }

    #[must_use]
    pub const fn east_asian(&self) -> Option<&CssFontVariantEastAsianValues> {
        self.east_asian.as_ref()
    }
}

/// The checked current authored `font-variant` shorthand value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantValue {
    Normal,
    None,
    Values(CssFontVariantValues),
}

/// The current authored `font-kerning` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontKerning {
    Auto,
    Normal,
    None,
}

/// The current authored `font-size-adjust` value.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontSizeAdjust {
    None,
    Number(CssNonNegativeNumber),
}

/// A nonempty checked set of `font-synthesis` capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontSynthesisValues {
    weight: bool,
    style: bool,
}

impl CssFontSynthesisValues {
    #[must_use]
    pub const fn try_new(weight: bool, style: bool) -> Option<Self> {
        if weight || style {
            Some(Self { weight, style })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn weight(self) -> bool {
        self.weight
    }

    #[must_use]
    pub const fn style(self) -> bool {
        self.style
    }
}

/// The current authored `font-synthesis` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontSynthesis {
    None,
    Values(CssFontSynthesisValues),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFeatureSettings {
    Normal,
    Features(CssFontFeatureList),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFeatureList {
    features: Vec<CssFontFeature>,
}

impl CssFontFeatureList {
    #[must_use]
    pub fn try_new(features: Vec<CssFontFeature>) -> Option<Self> {
        if features.is_empty() {
            None
        } else {
            Some(Self::new(features))
        }
    }

    #[must_use]
    pub(crate) fn new(features: Vec<CssFontFeature>) -> Self {
        Self { features }
    }

    #[must_use]
    pub fn features(&self) -> &[CssFontFeature] {
        &self.features
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFeature {
    tag: String,
    value: Option<CssFontFeatureValue>,
}

impl CssFontFeature {
    #[must_use]
    pub fn try_new(tag: impl Into<String>, value: Option<CssFontFeatureValue>) -> Option<Self> {
        let tag = tag.into();
        if !is_valid_font_feature_tag(&tag) {
            None
        } else {
            Some(Self::new(tag, value))
        }
    }

    #[must_use]
    pub(crate) fn new(tag: impl Into<String>, value: Option<CssFontFeatureValue>) -> Self {
        Self {
            tag: tag.into(),
            value,
        }
    }

    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    #[must_use]
    pub const fn value(&self) -> Option<CssFontFeatureValue> {
        self.value
    }
}

fn is_valid_font_feature_tag(tag: &str) -> bool {
    tag.chars().count() == 4
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFeatureValue {
    On,
    Off,
    Integer(i32),
}

/// A decoded OpenType feature tag containing exactly four ASCII characters.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssOpenTypeTag {
    value: String,
}

impl CssOpenTypeTag {
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.len() == 4 && value.is_ascii() {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// A non-negative authored OpenType feature index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssFontFeatureIndex {
    value: i32,
}

impl CssFontFeatureIndex {
    #[must_use]
    pub const fn try_new(value: i32) -> Option<Self> {
        if value >= 0 {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

/// The authored value associated with a checked OpenType feature tag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredFontFeatureValue {
    Omitted,
    On,
    Off,
    Index(CssFontFeatureIndex),
}

/// One checked current authored OpenType feature setting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAuthoredFontFeature {
    tag: CssOpenTypeTag,
    value: CssAuthoredFontFeatureValue,
}

impl CssAuthoredFontFeature {
    #[must_use]
    pub const fn new(tag: CssOpenTypeTag, value: CssAuthoredFontFeatureValue) -> Self {
        Self { tag, value }
    }

    #[must_use]
    pub const fn tag(&self) -> &CssOpenTypeTag {
        &self.tag
    }

    #[must_use]
    pub const fn value(&self) -> CssAuthoredFontFeatureValue {
        self.value
    }
}

/// A nonempty checked current authored OpenType feature list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAuthoredFontFeatureList {
    features: Vec<CssAuthoredFontFeature>,
}

impl CssAuthoredFontFeatureList {
    #[must_use]
    pub fn try_new(features: Vec<CssAuthoredFontFeature>) -> Option<Self> {
        if features.is_empty() {
            None
        } else {
            Some(Self { features })
        }
    }

    #[must_use]
    pub fn features(&self) -> &[CssAuthoredFontFeature] {
        &self.features
    }
}

/// The checked current authored `font-feature-settings` value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredFontFeatureSettings {
    Normal,
    Features(CssAuthoredFontFeatureList),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssFontSizeLengthPercentage {
    value: CssLength,
}

impl CssFontSizeLengthPercentage {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        if !length_has_valid_calc_shape(&value) {
            return None;
        }
        is_non_negative_length_percentage(&value).then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontSize {
    XxSmall,
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
    XxLarge,
    Larger,
    Smaller,
    LengthPercentage(CssFontSizeLengthPercentage),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssLineHeightLengthPercentage {
    value: CssLength,
}

impl CssLineHeightLengthPercentage {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        if !length_has_valid_calc_shape(&value) {
            return None;
        }
        is_non_negative_length_percentage(&value).then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssLineHeight {
    Normal,
    Number(CssNonNegativeNumberValue),
    LengthPercentage(CssLineHeightLengthPercentage),
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
    stretch: Option<CssFontStretch>,
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
        stretch: Option<CssFontStretch>,
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
    pub const fn style(&self) -> Option<CssFontStyle> {
        self.style
    }

    #[must_use]
    pub const fn variant(&self) -> Option<CssFontVariant> {
        self.variant
    }

    #[must_use]
    pub const fn weight(&self) -> Option<CssFontWeight> {
        self.weight
    }

    #[must_use]
    pub const fn stretch(&self) -> Option<CssFontStretch> {
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

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssLetterSpacing {
    Normal,
    Length(CssLetterSpacingLength),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssLetterSpacingLength {
    length: CssLength,
}

impl CssLetterSpacingLength {
    #[must_use]
    pub fn try_new(length: CssLength) -> Option<Self> {
        let length = crate::numeric::admit_pure_length(length)?;
        if is_letter_spacing_length(&length) {
            Some(Self::new(length))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(length: CssLength) -> Self {
        Self { length }
    }

    #[must_use]
    pub const fn length(&self) -> &CssLength {
        &self.length
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextWrap {
    Wrap,
    NoWrap,
    Balance,
    Pretty,
    Stable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWhiteSpace {
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWordBreak {
    Normal,
    BreakAll,
    KeepAll,
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

#[derive(Clone)]
pub struct CssTextDecoration {
    line: Option<CssTextDecorationLine>,
    color: Option<Box<CssParsedColor>>,
    style: Option<CssTextDecorationStyle>,
    thickness: Option<CssTextDecorationThickness>,
}

impl std::fmt::Debug for CssTextDecoration {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssTextDecoration")
            .field("line", &self.line)
            .field("color", &self.color())
            .field("style", &self.style)
            .field("thickness", &self.thickness)
            .finish()
    }
}

impl PartialEq for CssTextDecoration {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line
            && parsed_color_options_equal(self.color.as_deref(), other.color.as_deref())
            && self.style == other.style
            && self.thickness == other.thickness
    }
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

    #[must_use]
    pub(crate) fn new(
        line: Option<CssTextDecorationLine>,
        color: Option<CssColor>,
        style: Option<CssTextDecorationStyle>,
        thickness: Option<CssTextDecorationThickness>,
    ) -> Self {
        Self::new_current(line, color.map(CssParsedColor::from_i01), style, thickness)
    }

    pub(crate) fn new_current(
        line: Option<CssTextDecorationLine>,
        color: Option<CssParsedColor>,
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
    pub const fn color(&self) -> Option<&CssColor> {
        match self.color.as_ref() {
            Some(color) => color.i01_subset(),
            None => None,
        }
    }

    /// Returns the exact authored current color in the shorthand, when present.
    #[must_use]
    pub const fn current_color(&self) -> Option<&CssAuthoredColor> {
        match self.color.as_ref() {
            Some(color) => Some(color.current()),
            None => None,
        }
    }

    pub(crate) const fn has_exact_i01_projection(&self) -> bool {
        match self.color.as_ref() {
            Some(color) => color.i01_subset().is_some(),
            None => true,
        }
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

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTextDecorationThickness {
    Auto,
    FromFont,
    Length(CssTextDecorationThicknessLength),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTextDecorationThicknessLength {
    length: CssLength,
}

impl CssTextDecorationThicknessLength {
    #[must_use]
    pub fn try_new(length: CssLength) -> Option<Self> {
        if !length_has_valid_calc_shape(&length) {
            return None;
        }
        if is_text_decoration_thickness_length(&length) {
            Some(Self::new(length))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(length: CssLength) -> Self {
        Self { length }
    }

    #[must_use]
    pub const fn length(&self) -> &CssLength {
        &self.length
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextTransform {
    None,
    Capitalize,
    Uppercase,
    Lowercase,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssLengthDimension {
    value: CssFiniteNumber,
    unit: CssLengthUnit,
}

impl CssLengthDimension {
    #[must_use]
    pub fn try_new(value: f32, unit: CssLengthUnit) -> Option<Self> {
        CssFiniteNumber::try_new(value).map(|value| Self { value, unit })
    }

    #[must_use]
    pub(crate) const fn new(value: f32, unit: CssLengthUnit) -> Self {
        Self {
            value: CssFiniteNumber::new_unchecked(value),
            unit,
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }

    #[must_use]
    pub const fn unit(self) -> CssLengthUnit {
        self.unit
    }

    #[must_use]
    pub fn to_css_string(self) -> String {
        format!(
            "{}{}",
            format_css_number(self.value.value()),
            self.unit.as_css_str()
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssLength {
    Px(CssFiniteNumber),
    Dimension(CssLengthDimension),
    Percent(CssFiniteNumber),
    Zero,
    Auto,
    MinContent,
    MaxContent,
    FitContent,
    Normal,
    Thin,
    Medium,
    Thick,
    Calc(CssCalcLength),
}

impl CssLength {
    #[must_use]
    pub fn try_px(value: f32) -> Option<Self> {
        CssFiniteNumber::try_new(value).map(Self::Px)
    }

    #[must_use]
    pub fn try_percent(value: f32) -> Option<Self> {
        CssFiniteNumber::try_new(value).map(Self::Percent)
    }

    #[must_use]
    pub fn try_dimension(value: f32, unit: CssLengthUnit) -> Option<Self> {
        match unit {
            CssLengthUnit::Px => Self::try_px(value),
            _ => CssLengthDimension::try_new(value, unit).map(Self::Dimension),
        }
    }

    #[must_use]
    pub(crate) const fn px(value: f32) -> Self {
        Self::Px(CssFiniteNumber::new_unchecked(value))
    }

    #[must_use]
    pub(crate) const fn percent(value: f32) -> Self {
        Self::Percent(CssFiniteNumber::new_unchecked(value))
    }

    #[must_use]
    pub(crate) const fn dimension(value: f32, unit: CssLengthUnit) -> Self {
        match unit {
            CssLengthUnit::Px => Self::px(value),
            _ => Self::Dimension(CssLengthDimension::new(value, unit)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssEdges {
    pub top: CssLength,
    pub right: CssLength,
    pub bottom: CssLength,
    pub left: CssLength,
}

impl CssEdges {
    #[must_use]
    pub fn all(value: CssLength) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    #[must_use]
    pub const fn new(top: CssLength, right: CssLength, bottom: CssLength, left: CssLength) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
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

#[derive(Clone)]
pub struct CssBorder {
    width: Option<CssLength>,
    style: Option<CssBorderStyle>,
    color: Option<Box<CssParsedColor>>,
}

impl std::fmt::Debug for CssBorder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssBorder")
            .field("width", &self.width)
            .field("style", &self.style)
            .field("color", &self.color())
            .finish()
    }
}

impl PartialEq for CssBorder {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.style == other.style
            && parsed_color_options_equal(self.color.as_deref(), other.color.as_deref())
    }
}

impl CssBorder {
    #[must_use]
    pub fn try_new(
        width: Option<CssLength>,
        style: Option<CssBorderStyle>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        let width = match width {
            Some(value) => Some(crate::numeric::admit_pure_length(value)?),
            None => None,
        };
        if width.is_none() && style.is_none() && color.is_none()
            || width.as_ref().is_some_and(|width| !is_border_width(width))
        {
            None
        } else {
            Some(Self::new(width, style, color))
        }
    }

    #[must_use]
    pub(crate) fn new(
        width: Option<CssLength>,
        style: Option<CssBorderStyle>,
        color: Option<CssColor>,
    ) -> Self {
        Self::new_current(width, style, color.map(CssParsedColor::from_i01))
    }

    #[must_use]
    pub(crate) fn new_current(
        width: Option<CssLength>,
        style: Option<CssBorderStyle>,
        color: Option<CssParsedColor>,
    ) -> Self {
        Self {
            width,
            style,
            color: color.map(Box::new),
        }
    }

    #[must_use]
    pub const fn width(&self) -> Option<&CssLength> {
        self.width.as_ref()
    }

    #[must_use]
    pub const fn style(&self) -> Option<CssBorderStyle> {
        self.style
    }

    #[must_use]
    pub const fn color(&self) -> Option<&CssColor> {
        match self.color.as_ref() {
            Some(color) => color.i01_subset(),
            None => None,
        }
    }

    /// Returns the exact authored current color in the shorthand, when present.
    #[must_use]
    pub const fn current_color(&self) -> Option<&CssAuthoredColor> {
        match self.color.as_ref() {
            Some(color) => Some(color.current()),
            None => None,
        }
    }

    pub(crate) const fn has_exact_i01_projection(&self) -> bool {
        match self.color.as_ref() {
            Some(color) => color.i01_subset().is_some(),
            None => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssBorderStyles {
    pub top: CssBorderStyle,
    pub right: CssBorderStyle,
    pub bottom: CssBorderStyle,
    pub left: CssBorderStyle,
}

impl CssBorderStyles {
    #[must_use]
    pub const fn new(
        top: CssBorderStyle,
        right: CssBorderStyle,
        bottom: CssBorderStyle,
        left: CssBorderStyle,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub const fn all(value: CssBorderStyle) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssCornerRadius {
    horizontal: CssLength,
    vertical: CssLength,
}

impl CssCornerRadius {
    #[must_use]
    pub fn try_new(horizontal: CssLength, vertical: CssLength) -> Option<Self> {
        if !(length_has_valid_calc_shape(&horizontal) && length_has_valid_calc_shape(&vertical)) {
            return None;
        }
        if is_radius_length(&horizontal) && is_radius_length(&vertical) {
            Some(Self::new(horizontal, vertical))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(horizontal: CssLength, vertical: CssLength) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    #[must_use]
    pub const fn horizontal(&self) -> &CssLength {
        &self.horizontal
    }

    #[must_use]
    pub const fn vertical(&self) -> &CssLength {
        &self.vertical
    }
}

fn is_border_width(length: &CssLength) -> bool {
    match length {
        CssLength::Px(value) => value.value() >= 0.0,
        CssLength::Dimension(length) => length.value() >= 0.0,
        CssLength::Zero | CssLength::Thin | CssLength::Medium | CssLength::Thick => true,
        CssLength::Calc(calc) => !calc.uses_percentage() && !calc_has_negative_component(calc),
        CssLength::Percent(_)
        | CssLength::Auto
        | CssLength::MinContent
        | CssLength::MaxContent
        | CssLength::FitContent
        | CssLength::Normal => false,
    }
}

fn is_radius_length(length: &CssLength) -> bool {
    match length {
        CssLength::Px(value) | CssLength::Percent(value) => value.value() >= 0.0,
        CssLength::Dimension(length) => length.value() >= 0.0,
        CssLength::Zero => true,
        CssLength::Calc(calc) => !calc_has_negative_component(calc),
        CssLength::Auto
        | CssLength::MinContent
        | CssLength::MaxContent
        | CssLength::FitContent
        | CssLength::Normal
        | CssLength::Thin
        | CssLength::Medium
        | CssLength::Thick => false,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderRadii {
    pub top_left: CssCornerRadius,
    pub top_right: CssCornerRadius,
    pub bottom_right: CssCornerRadius,
    pub bottom_left: CssCornerRadius,
}

impl CssBorderRadii {
    #[must_use]
    pub const fn new(
        top_left: CssCornerRadius,
        top_right: CssCornerRadius,
        bottom_right: CssCornerRadius,
        bottom_left: CssCornerRadius,
    ) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBoxShadow {
    None,
    Shadows(CssBoxShadowList),
}

impl CssBoxShadow {
    pub(crate) fn has_exact_i01_projection(&self) -> bool {
        match self {
            Self::None => true,
            Self::Shadows(shadows) => shadows
                .shadows()
                .iter()
                .all(CssShadow::has_exact_i01_projection),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssBoxShadowList {
    shadows: Vec<CssShadow>,
}

impl CssBoxShadowList {
    pub(crate) fn new(shadows: Vec<CssShadow>) -> Option<Self> {
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

#[derive(Clone)]
pub struct CssShadow {
    inset: bool,
    offset_x: CssLength,
    offset_y: CssLength,
    blur_radius: Option<CssLength>,
    spread_radius: Option<CssLength>,
    color: Option<Box<CssParsedColor>>,
}

impl std::fmt::Debug for CssShadow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssShadow")
            .field("inset", &self.inset)
            .field("offset_x", &self.offset_x)
            .field("offset_y", &self.offset_y)
            .field("blur_radius", &self.blur_radius)
            .field("spread_radius", &self.spread_radius)
            .field("color", &self.color())
            .finish()
    }
}

impl PartialEq for CssShadow {
    fn eq(&self, other: &Self) -> bool {
        self.inset == other.inset
            && self.offset_x == other.offset_x
            && self.offset_y == other.offset_y
            && self.blur_radius == other.blur_radius
            && self.spread_radius == other.spread_radius
            && parsed_color_options_equal(self.color.as_deref(), other.color.as_deref())
    }
}

impl CssShadow {
    #[must_use]
    pub fn try_new(
        inset: bool,
        offset_x: CssLength,
        offset_y: CssLength,
        blur_radius: Option<CssLength>,
        spread_radius: Option<CssLength>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        let offset_x = crate::numeric::admit_pure_length(offset_x)?;
        let offset_y = crate::numeric::admit_pure_length(offset_y)?;
        let blur_radius = match blur_radius {
            Some(value) => Some(crate::numeric::admit_pure_length(value)?),
            None => None,
        };
        let spread_radius = match spread_radius {
            Some(value) => Some(crate::numeric::admit_pure_length(value)?),
            None => None,
        };
        if !is_shadow_length(&offset_x)
            || !is_shadow_length(&offset_y)
            || blur_radius
                .as_ref()
                .is_some_and(|blur| !is_shadow_length(blur) || length_has_negative_component(blur))
            || spread_radius
                .as_ref()
                .is_some_and(|spread| !is_shadow_length(spread))
            || blur_radius.is_none() && spread_radius.is_some()
        {
            None
        } else {
            Some(Self::new(
                inset,
                offset_x,
                offset_y,
                blur_radius,
                spread_radius,
                color,
            ))
        }
    }

    #[must_use]
    pub(crate) fn new(
        inset: bool,
        offset_x: CssLength,
        offset_y: CssLength,
        blur_radius: Option<CssLength>,
        spread_radius: Option<CssLength>,
        color: Option<CssColor>,
    ) -> Self {
        Self::new_current(
            inset,
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            color.map(CssParsedColor::from_i01),
        )
    }

    #[must_use]
    pub(crate) fn new_current(
        inset: bool,
        offset_x: CssLength,
        offset_y: CssLength,
        blur_radius: Option<CssLength>,
        spread_radius: Option<CssLength>,
        color: Option<CssParsedColor>,
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

    #[must_use]
    pub const fn inset(&self) -> bool {
        self.inset
    }

    #[must_use]
    pub const fn offset_x(&self) -> &CssLength {
        &self.offset_x
    }

    #[must_use]
    pub const fn offset_y(&self) -> &CssLength {
        &self.offset_y
    }

    #[must_use]
    pub const fn blur_radius(&self) -> Option<&CssLength> {
        self.blur_radius.as_ref()
    }

    #[must_use]
    pub const fn spread_radius(&self) -> Option<&CssLength> {
        self.spread_radius.as_ref()
    }

    #[must_use]
    pub const fn color(&self) -> Option<&CssColor> {
        match self.color.as_ref() {
            Some(color) => color.i01_subset(),
            None => None,
        }
    }

    /// Returns the exact authored current shadow color, when present.
    #[must_use]
    pub const fn current_color(&self) -> Option<&CssAuthoredColor> {
        match self.color.as_ref() {
            Some(color) => Some(color.current()),
            None => None,
        }
    }

    pub(crate) const fn has_exact_i01_projection(&self) -> bool {
        match self.color.as_ref() {
            Some(color) => color.i01_subset().is_some(),
            None => true,
        }
    }
}

fn is_shadow_length(length: &CssLength) -> bool {
    match length {
        CssLength::Px(_) | CssLength::Dimension(_) | CssLength::Zero => true,
        CssLength::Calc(calc) => !calc.uses_percentage(),
        CssLength::Percent(_)
        | CssLength::Auto
        | CssLength::MinContent
        | CssLength::MaxContent
        | CssLength::FitContent
        | CssLength::Normal
        | CssLength::Thin
        | CssLength::Medium
        | CssLength::Thick => false,
    }
}

fn is_text_length(length: &CssLength) -> bool {
    matches!(
        length,
        CssLength::Px(_)
            | CssLength::Dimension(_)
            | CssLength::Percent(_)
            | CssLength::Zero
            | CssLength::Calc(_)
    )
}

fn is_vertical_align_length(length: &CssLength) -> bool {
    is_text_length(length)
}

fn is_letter_spacing_length(length: &CssLength) -> bool {
    is_absolute_length(length)
}

fn is_absolute_length(length: &CssLength) -> bool {
    match length {
        CssLength::Px(_) | CssLength::Dimension(_) | CssLength::Zero => true,
        CssLength::Calc(calc) => !calc.uses_percentage(),
        CssLength::Percent(_)
        | CssLength::Auto
        | CssLength::MinContent
        | CssLength::MaxContent
        | CssLength::FitContent
        | CssLength::Normal
        | CssLength::Thin
        | CssLength::Medium
        | CssLength::Thick => false,
    }
}

fn is_non_negative_absolute_length(length: &CssLength) -> bool {
    match length {
        CssLength::Px(value) => value.value() >= 0.0,
        CssLength::Dimension(value) => value.value() >= 0.0,
        CssLength::Zero => true,
        CssLength::Calc(calc) => !calc.uses_percentage(),
        CssLength::Percent(_)
        | CssLength::Auto
        | CssLength::MinContent
        | CssLength::MaxContent
        | CssLength::FitContent
        | CssLength::Normal
        | CssLength::Thin
        | CssLength::Medium
        | CssLength::Thick => false,
    }
}

fn is_text_decoration_thickness_length(length: &CssLength) -> bool {
    match length {
        CssLength::Px(value) | CssLength::Percent(value) => value.value() >= 0.0,
        CssLength::Dimension(length) => length.value() >= 0.0,
        CssLength::Zero => true,
        CssLength::Calc(calc) => !calc_has_negative_component(calc),
        CssLength::Auto
        | CssLength::MinContent
        | CssLength::MaxContent
        | CssLength::FitContent
        | CssLength::Normal
        | CssLength::Thin
        | CssLength::Medium
        | CssLength::Thick => false,
    }
}

fn is_non_negative_length_percentage(length: &CssLength) -> bool {
    match length {
        CssLength::Px(value) | CssLength::Percent(value) => value.value() >= 0.0,
        CssLength::Dimension(value) => value.value() >= 0.0,
        CssLength::Zero | CssLength::Calc(_) => true,
        CssLength::Auto
        | CssLength::MinContent
        | CssLength::MaxContent
        | CssLength::FitContent
        | CssLength::Normal
        | CssLength::Thin
        | CssLength::Medium
        | CssLength::Thick => false,
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct CssUrl {
    value: String,
    modifiers: Vec<CssUrlModifier>,
}

impl CssUrl {
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.is_empty() {
            None
        } else {
            Some(Self::new(value))
        }
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            modifiers: Vec::new(),
        }
    }

    #[must_use]
    pub(crate) fn with_modifiers(value: impl Into<String>, modifiers: Vec<CssUrlModifier>) -> Self {
        Self {
            value: value.into(),
            modifiers,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Returns the authored modifiers of a quoted `url()` value.
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

impl std::fmt::Debug for CssUrl {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Preserve the established diagnostic/debug projection used by the I01
        // behavioral oracle. Authored modifiers are inspected through the
        // intentional typed accessor above rather than Debug text.
        formatter
            .debug_struct("CssUrl")
            .field("value", &self.value)
            .finish()
    }
}

/// One decoded CSS identifier retained as authored value syntax.
///
/// This parser-owned value does not apply consumer-specific keyword exclusions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssIdent {
    value: String,
}

impl CssIdent {
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

/// One parser-produced functional URL modifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssUrlModifierFunction {
    name: CssIdent,
    arguments: CssAuthoredFunctionArguments,
}

impl CssUrlModifierFunction {
    #[must_use]
    pub(crate) const fn new(name: CssIdent, arguments: CssAuthoredFunctionArguments) -> Self {
        Self { name, arguments }
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTransformArguments {
    authored: CssAuthoredFunctionArguments,
}

impl CssTransformArguments {
    #[must_use]
    pub(crate) const fn new(authored: CssAuthoredFunctionArguments) -> Self {
        Self { authored }
    }

    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFilterArguments {
    authored: CssAuthoredFunctionArguments,
}

impl CssFilterArguments {
    #[must_use]
    pub(crate) const fn new(authored: CssAuthoredFunctionArguments) -> Self {
        Self { authored }
    }

    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBasicShapeArguments {
    authored: CssAuthoredFunctionArguments,
}

impl CssBasicShapeArguments {
    #[must_use]
    pub(crate) const fn new(authored: CssAuthoredFunctionArguments) -> Self {
        Self { authored }
    }

    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssEasingArguments {
    authored: CssAuthoredFunctionArguments,
}

impl CssEasingArguments {
    #[must_use]
    pub(crate) const fn new(authored: CssAuthoredFunctionArguments) -> Self {
        Self { authored }
    }

    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssImageLayer {
    None,
    Url(CssUrl),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssImageLayerList {
    layers: Vec<CssImageLayer>,
}

impl CssImageLayerList {
    #[must_use]
    pub fn try_new(layers: Vec<CssImageLayer>) -> Option<Self> {
        if layers.is_empty() {
            None
        } else {
            Some(Self::new(layers))
        }
    }

    #[must_use]
    pub(crate) fn new(layers: Vec<CssImageLayer>) -> Self {
        Self { layers }
    }

    #[must_use]
    pub fn layers(&self) -> &[CssImageLayer] {
        &self.layers
    }
}

/// A current authored image value accepted by Images 3 consumers.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssImageValue {
    None,
    Url(CssUrl),
    Gradient(CssGradient),
}

/// One non-negative authored `border-image-slice` component.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBorderImageSliceComponent {
    Number(CssNonNegativeNumber),
    Percentage(CssNonNegativeNumber),
    NumberCalculation(CssNumberCalculation),
    PercentageCalculation(CssPercentageCalculation),
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
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBorderImageWidthComponent {
    Auto,
    LengthPercentage(CssBorderImageWidthLengthPercentage),
    Number(CssNonNegativeNumber),
    NumberCalculation(CssNumberCalculation),
}

/// A checked non-negative authored length-percentage used by `border-image-width`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImageWidthLengthPercentage {
    value: CssLength,
}

impl CssBorderImageWidthLengthPercentage {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        if !length_has_valid_calc_shape(&value) {
            return None;
        }
        is_non_negative_length_percentage(&value).then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
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
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBorderImageOutsetComponent {
    Length(CssBorderImageOutsetLength),
    Number(CssNonNegativeNumber),
    NumberCalculation(CssNumberCalculation),
}

/// A checked non-negative authored length used by `border-image-outset`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImageOutsetLength {
    value: CssLength,
}

impl CssBorderImageOutsetLength {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        is_non_negative_absolute_length(&value).then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
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
    pub(crate) const fn new(
        source: Option<CssImageValue>,
        slice: Option<CssBorderImageSlice>,
        width: Option<CssBorderImageWidth>,
        outset: Option<CssBorderImageOutset>,
        repeat: Option<CssBorderImageRepeat>,
    ) -> Self {
        Self {
            source,
            slice,
            width,
            outset,
            repeat,
        }
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

/// A finite authored angle used by `image-orientation`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssImageOrientationAngle {
    Zero,
    Literal(CssAngleLiteral),
    Calculation(CssAngleCalculation),
}

/// The authored Images 3 `image-orientation` syntax.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssImageOrientation {
    FromImage,
    Angle(CssImageOrientationAngle),
    Flip(Option<CssImageOrientationAngle>),
}

/// The authored Images 3 `image-rendering` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImageRendering {
    Auto,
    CrispEdges,
    Pixelated,
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedImageValueList {
    current: CssImageValueList,
    i01_subset: Option<CssImageLayerList>,
}

impl CssParsedImageValueList {
    pub(crate) const fn new(
        current: CssImageValueList,
        i01_subset: Option<CssImageLayerList>,
    ) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn into_parts(self) -> (CssImageValueList, Option<CssImageLayerList>) {
        (self.current, self.i01_subset)
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

/// A finite authored gradient angle, including the Images 3 unitless-zero branch.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGradientAngle {
    Zero,
    Literal(CssAngleLiteral),
    Calculation(CssAngleCalculation),
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

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssLinearGradientDirection {
    Angle(CssGradientAngle),
    SideOrCorner(CssSideOrCorner),
}

/// An authored length-percentage position on a gradient line.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGradientLinePosition {
    value: CssLength,
}

impl CssGradientLinePosition {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        if !length_has_valid_calc_shape(&value) {
            return None;
        }
        matches!(
            value,
            CssLength::Px(_)
                | CssLength::Dimension(_)
                | CssLength::Percent(_)
                | CssLength::Zero
                | CssLength::Calc(_)
        )
        .then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// One authored color stop in a gradient color-stop list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGradientColorStop {
    color: CssParsedColor,
    position: Option<CssGradientLinePosition>,
}

impl CssGradientColorStop {
    pub(crate) const fn new(
        color: CssParsedColor,
        position: Option<CssGradientLinePosition>,
    ) -> Self {
        Self { color, position }
    }

    #[must_use]
    pub const fn color(&self) -> &CssAuthoredColor {
        self.color.current()
    }

    #[must_use]
    pub const fn position(&self) -> Option<&CssGradientLinePosition> {
        self.position.as_ref()
    }
}

/// One authored item in a color-stop list, preserving stop and hint order.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColorStopListItem {
    Stop(Box<CssGradientColorStop>),
    Hint(CssGradientLinePosition),
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

/// A current authored linear or repeating-linear gradient payload.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLinearGradient {
    direction: Option<CssLinearGradientDirection>,
    stops: CssColorStopList,
}

impl CssLinearGradient {
    pub(crate) const fn new(
        direction: Option<CssLinearGradientDirection>,
        stops: CssColorStopList,
    ) -> Self {
        Self { direction, stops }
    }

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

/// A checked non-negative explicit circle radius for a radial gradient.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRadialCircleSize {
    radius: CssLength,
}

impl CssRadialCircleSize {
    #[must_use]
    pub fn try_new(radius: CssLength) -> Option<Self> {
        let radius = crate::numeric::admit_pure_length(radius)?;
        let valid = match &radius {
            CssLength::Px(value) => value.value() >= 0.0,
            CssLength::Dimension(value) => value.value() >= 0.0,
            CssLength::Zero => true,
            CssLength::Calc(calculation) => !calculation.uses_percentage(),
            CssLength::Percent(_)
            | CssLength::Auto
            | CssLength::MinContent
            | CssLength::MaxContent
            | CssLength::FitContent
            | CssLength::Normal
            | CssLength::Thin
            | CssLength::Medium
            | CssLength::Thick => false,
        };
        valid.then_some(Self { radius })
    }

    #[must_use]
    pub const fn radius(&self) -> &CssLength {
        &self.radius
    }
}

/// A checked pair of non-negative radial-gradient ellipse radii.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRadialEllipseSize {
    horizontal: CssLength,
    vertical: CssLength,
}

impl CssRadialEllipseSize {
    #[must_use]
    pub fn try_new(horizontal: CssLength, vertical: CssLength) -> Option<Self> {
        if !(length_has_valid_calc_shape(&horizontal) && length_has_valid_calc_shape(&vertical)) {
            return None;
        }
        let valid = |value: &CssLength| match value {
            CssLength::Px(value) | CssLength::Percent(value) => value.value() >= 0.0,
            CssLength::Dimension(value) => value.value() >= 0.0,
            CssLength::Zero | CssLength::Calc(_) => true,
            CssLength::Auto
            | CssLength::MinContent
            | CssLength::MaxContent
            | CssLength::FitContent
            | CssLength::Normal
            | CssLength::Thin
            | CssLength::Medium
            | CssLength::Thick => false,
        };
        (valid(&horizontal) && valid(&vertical)).then_some(Self {
            horizontal,
            vertical,
        })
    }

    #[must_use]
    pub const fn horizontal(&self) -> &CssLength {
        &self.horizontal
    }

    #[must_use]
    pub const fn vertical(&self) -> &CssLength {
        &self.vertical
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssRadialSize {
    Extent(CssRadialExtent),
    Circle(CssRadialCircleSize),
    Ellipse(CssRadialEllipseSize),
}

/// A current authored radial or repeating-radial gradient payload.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRadialGradient {
    shape: Option<CssRadialShape>,
    size: Option<CssRadialSize>,
    position: Option<CssPositionValue>,
    stops: CssColorStopList,
}

impl CssRadialGradient {
    pub(crate) const fn new(
        shape: Option<CssRadialShape>,
        size: Option<CssRadialSize>,
        position: Option<CssPositionValue>,
        stops: CssColorStopList,
    ) -> Self {
        Self {
            shape,
            size,
            position,
            stops,
        }
    }

    #[must_use]
    pub const fn shape(&self) -> Option<CssRadialShape> {
        self.shape
    }

    #[must_use]
    pub const fn size(&self) -> Option<&CssRadialSize> {
        self.size.as_ref()
    }

    #[must_use]
    pub const fn position(&self) -> Option<&CssPositionValue> {
        self.position.as_ref()
    }

    #[must_use]
    pub const fn stops(&self) -> &CssColorStopList {
        &self.stops
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHorizontalPositionKeyword {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVerticalPositionKeyword {
    Top,
    Center,
    Bottom,
}

/// A checked authored `<length-percentage>` offset in a CSS position.
///
/// The value remains symbolic: percentages and calculations are not resolved against a box.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPositionOffset {
    value: CssLength,
}

impl CssPositionOffset {
    /// Constructs an offset from a position-valid authored length or percentage.
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        if !length_has_valid_calc_shape(&value) {
            return None;
        }
        match value {
            CssLength::Px(_)
            | CssLength::Dimension(_)
            | CssLength::Percent(_)
            | CssLength::Zero
            | CssLength::Calc(_) => Some(Self { value }),
            CssLength::Auto
            | CssLength::MinContent
            | CssLength::MaxContent
            | CssLength::FitContent
            | CssLength::Normal
            | CssLength::Thin
            | CssLength::Medium
            | CssLength::Thick => None,
        }
    }

    /// Returns the authored symbolic length or percentage.
    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// The exact authored horizontal axis of a generic CSS `<position>`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssHorizontalPosition {
    Left,
    Center,
    Right,
    /// An offset from the horizontal start edge without an authored edge keyword.
    Offset(CssPositionOffset),
    LeftOffset(CssPositionOffset),
    RightOffset(CssPositionOffset),
}

/// The exact authored vertical axis of a generic CSS `<position>`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssVerticalPosition {
    Top,
    Center,
    Bottom,
    /// An offset from the vertical start edge without an authored edge keyword.
    Offset(CssPositionOffset),
    TopOffset(CssPositionOffset),
    BottomOffset(CssPositionOffset),
}

/// A parser-produced authored generic CSS `<position>`.
///
/// Both axes are explicit in this model, including axes omitted and therefore centered by the
/// grammar. Construction is parser-owned so invalid cross-axis combinations cannot be forged.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPositionValue {
    horizontal: CssHorizontalPosition,
    vertical: CssVerticalPosition,
}

impl CssPositionValue {
    #[must_use]
    pub(crate) const fn new(
        horizontal: CssHorizontalPosition,
        vertical: CssVerticalPosition,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// Returns the authored horizontal position, including its offset origin.
    #[must_use]
    pub const fn horizontal(&self) -> &CssHorizontalPosition {
        &self.horizontal
    }

    /// Returns the authored vertical position, including its offset origin.
    #[must_use]
    pub const fn vertical(&self) -> &CssVerticalPosition {
        &self.vertical
    }
}

/// One parser-produced authored layer of `background-position`.
///
/// This model is distinct from generic `<position>` because the background grammar admits its
/// specified three-component form. Both axes remain symbolic and retain authored edge origins.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundPosition {
    horizontal: CssHorizontalPosition,
    vertical: CssVerticalPosition,
    legacy: Option<CssPosition>,
}

impl CssBackgroundPosition {
    #[must_use]
    pub(crate) const fn new(
        horizontal: CssHorizontalPosition,
        vertical: CssVerticalPosition,
        legacy: Option<CssPosition>,
    ) -> Self {
        Self {
            horizontal,
            vertical,
            legacy,
        }
    }

    /// Returns the authored horizontal position, including its offset origin.
    #[must_use]
    pub const fn horizontal(&self) -> &CssHorizontalPosition {
        &self.horizontal
    }

    /// Returns the authored vertical position, including its offset origin.
    #[must_use]
    pub const fn vertical(&self) -> &CssVerticalPosition {
        &self.vertical
    }

    #[must_use]
    pub(crate) const fn legacy(&self) -> Option<&CssPosition> {
        self.legacy.as_ref()
    }
}

/// A nonempty authored comma list of `background-position` layers.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundPositionList {
    positions: Vec<CssBackgroundPosition>,
}

impl CssBackgroundPositionList {
    /// Constructs a nonempty list from already validated background-position layers.
    #[must_use]
    pub fn try_new(positions: Vec<CssBackgroundPosition>) -> Option<Self> {
        if positions.is_empty() {
            None
        } else {
            Some(Self::new(positions))
        }
    }

    #[must_use]
    pub(crate) const fn new(positions: Vec<CssBackgroundPosition>) -> Self {
        Self { positions }
    }

    /// Returns the authored layers in comma order.
    #[must_use]
    pub fn positions(&self) -> &[CssBackgroundPosition] {
        &self.positions
    }
}

/// One parser-produced authored layer of `mask-position`.
///
/// Mask layers use generic `<position>` exactly and therefore cannot represent the
/// background-only three-component form.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMaskPosition {
    value: CssPositionValue,
    legacy: Option<CssPosition>,
}

impl CssMaskPosition {
    #[must_use]
    pub(crate) const fn new(value: CssPositionValue, legacy: Option<CssPosition>) -> Self {
        Self { value, legacy }
    }

    /// Returns the exact generic position for this mask layer.
    #[must_use]
    pub const fn value(&self) -> &CssPositionValue {
        &self.value
    }

    #[must_use]
    pub(crate) const fn legacy(&self) -> Option<&CssPosition> {
        self.legacy.as_ref()
    }
}

/// A nonempty authored comma list of generic `mask-position` layers.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMaskPositionList {
    positions: Vec<CssMaskPosition>,
}

impl CssMaskPositionList {
    /// Constructs a nonempty list from already validated mask-position layers.
    #[must_use]
    pub fn try_new(positions: Vec<CssMaskPosition>) -> Option<Self> {
        if positions.is_empty() {
            None
        } else {
            Some(Self::new(positions))
        }
    }

    #[must_use]
    pub(crate) const fn new(positions: Vec<CssMaskPosition>) -> Self {
        Self { positions }
    }

    /// Returns the authored layers in comma order.
    #[must_use]
    pub fn positions(&self) -> &[CssMaskPosition] {
        &self.positions
    }
}

/// A parser-produced authored value of the `object-position` property.
///
/// Object positioning uses generic `<position>` exactly. The value remains symbolic and does not
/// resolve percentages against an object or positioning area.
#[derive(Clone, Debug, PartialEq)]
pub struct CssObjectPosition {
    value: CssPositionValue,
}

impl CssObjectPosition {
    #[must_use]
    pub(crate) const fn new(value: CssPositionValue) -> Self {
        Self { value }
    }

    /// Returns the exact authored generic position.
    #[must_use]
    pub const fn value(&self) -> &CssPositionValue {
        &self.value
    }
}

/// A checked authored `<length>` on the transform-origin z axis.
///
/// Percentages and mixed length-percentage calculations are not valid on this axis. A well-typed
/// length calculation remains symbolic because range evaluation belongs to computed values.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformOriginZ {
    value: CssLength,
}

impl CssTransformOriginZ {
    /// Constructs a z value from an authored length without a percentage component.
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        match &value {
            CssLength::Px(_) | CssLength::Dimension(_) | CssLength::Zero => Some(Self { value }),
            CssLength::Calc(calculation) if !calculation.uses_percentage() => Some(Self { value }),
            CssLength::Percent(_)
            | CssLength::Auto
            | CssLength::MinContent
            | CssLength::MaxContent
            | CssLength::FitContent
            | CssLength::Normal
            | CssLength::Thin
            | CssLength::Medium
            | CssLength::Thick
            | CssLength::Calc(_) => None,
        }
    }

    /// Returns the authored symbolic z length.
    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// A parser-produced authored value of the `transform-origin` property.
///
/// Both 2D axes are explicit, and the optional z axis can contain only a checked authored length.
/// Construction is parser-owned so the directed greedy split cannot be bypassed.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformOrigin {
    horizontal: CssHorizontalPosition,
    vertical: CssVerticalPosition,
    z: Option<CssTransformOriginZ>,
    legacy: Option<CssPosition>,
}

impl CssTransformOrigin {
    #[must_use]
    pub(crate) fn new(
        position: CssPositionValue,
        z: Option<CssTransformOriginZ>,
        legacy: Option<CssPosition>,
    ) -> Self {
        Self {
            horizontal: position.horizontal,
            vertical: position.vertical,
            z,
            legacy,
        }
    }

    /// Returns the authored horizontal position.
    #[must_use]
    pub const fn horizontal(&self) -> &CssHorizontalPosition {
        &self.horizontal
    }

    /// Returns the authored vertical position.
    #[must_use]
    pub const fn vertical(&self) -> &CssVerticalPosition {
        &self.vertical
    }

    /// Returns the optional authored z length.
    #[must_use]
    pub const fn z(&self) -> Option<&CssTransformOriginZ> {
        self.z.as_ref()
    }

    #[must_use]
    pub(crate) const fn legacy(&self) -> Option<&CssPosition> {
        self.legacy.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPositionComponent {
    Horizontal(CssHorizontalPositionKeyword),
    Vertical(CssVerticalPositionKeyword),
    Length(CssLength),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssPosition {
    components: Vec<CssPositionComponent>,
}

impl CssPosition {
    #[must_use]
    pub fn try_new(components: Vec<CssPositionComponent>) -> Option<Self> {
        if !components.iter().all(|component| match component {
            CssPositionComponent::Length(value) => length_has_valid_calc_shape(value),
            _ => true,
        }) {
            return None;
        }
        if components.is_empty()
            || components.len() > 4
            || has_duplicate_axis_side_keywords(&components)
        {
            None
        } else {
            Some(Self::new(components))
        }
    }

    #[must_use]
    pub(crate) fn new(components: Vec<CssPositionComponent>) -> Self {
        Self { components }
    }

    #[must_use]
    pub fn components(&self) -> &[CssPositionComponent] {
        &self.components
    }
}

fn has_duplicate_axis_side_keywords(components: &[CssPositionComponent]) -> bool {
    let mut has_horizontal_side = false;
    let mut has_vertical_side = false;

    for component in components {
        match component {
            CssPositionComponent::Horizontal(
                CssHorizontalPositionKeyword::Left | CssHorizontalPositionKeyword::Right,
            ) => {
                if has_horizontal_side {
                    return true;
                }
                has_horizontal_side = true;
            }
            CssPositionComponent::Vertical(
                CssVerticalPositionKeyword::Top | CssVerticalPositionKeyword::Bottom,
            ) => {
                if has_vertical_side {
                    return true;
                }
                has_vertical_side = true;
            }
            CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center)
            | CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center)
            | CssPositionComponent::Length(_) => {}
        }
    }

    false
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssPositionList {
    positions: Vec<CssPosition>,
}

impl CssPositionList {
    #[must_use]
    pub fn try_new(positions: Vec<CssPosition>) -> Option<Self> {
        if positions.is_empty() {
            None
        } else {
            Some(Self::new(positions))
        }
    }

    #[must_use]
    pub(crate) fn new(positions: Vec<CssPosition>) -> Self {
        Self { positions }
    }

    #[must_use]
    pub fn positions(&self) -> &[CssPosition] {
        &self.positions
    }
}

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

/// One parser-produced authored `background` shorthand layer.
///
/// Construction is parser-owned so a size cannot exist without an immediately
/// preceding position and a color cannot occur outside the final layer.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundLayer {
    image: Option<CssImageValue>,
    position: Option<CssBackgroundPosition>,
    size: Option<CssBackgroundSize>,
    repeat: Option<CssBackgroundRepeat>,
    attachment: Option<CssBackgroundAttachment>,
    boxes: Option<CssBackgroundLayerBoxes>,
    color: Option<CssAuthoredColor>,
}

impl CssBackgroundLayer {
    #[must_use]
    pub(crate) const fn new(
        image: Option<CssImageValue>,
        position: Option<CssBackgroundPosition>,
        size: Option<CssBackgroundSize>,
        repeat: Option<CssBackgroundRepeat>,
        attachment: Option<CssBackgroundAttachment>,
        boxes: Option<CssBackgroundLayerBoxes>,
        color: Option<CssAuthoredColor>,
    ) -> Self {
        Self {
            image,
            position,
            size,
            repeat,
            attachment,
            boxes,
            color,
        }
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
    pub const fn color(&self) -> Option<&CssAuthoredColor> {
        self.color.as_ref()
    }

    pub(crate) const fn has_only_color(&self) -> bool {
        self.image.is_none()
            && self.position.is_none()
            && self.size.is_none()
            && self.repeat.is_none()
            && self.attachment.is_none()
            && self.boxes.is_none()
            && self.color.is_some()
    }
}

/// A nonempty authored comma list of `background` shorthand layers.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackground {
    layers: Vec<CssBackgroundLayer>,
}

impl CssBackground {
    #[must_use]
    pub(crate) const fn new(layers: Vec<CssBackgroundLayer>) -> Self {
        Self { layers }
    }

    /// Returns the authored layers in comma order.
    #[must_use]
    pub fn layers(&self) -> &[CssBackgroundLayer] {
        &self.layers
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedBackground {
    current: CssBackground,
    i01_subset: Option<CssColor>,
}

impl CssParsedBackground {
    #[must_use]
    pub(crate) const fn new(current: CssBackground, i01_subset: Option<CssColor>) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    #[must_use]
    pub(crate) fn into_parts(self) -> (CssBackground, Option<CssColor>) {
        (self.current, self.i01_subset)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundSizeComponent {
    Auto,
    Length(CssLength),
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
        if !sizes.iter().all(|size| match size {
            CssBackgroundSize::Explicit { width, height } => std::iter::once(width)
                .chain(height.iter())
                .all(|component| match component {
                    CssBackgroundSizeComponent::Length(value) => length_has_valid_calc_shape(value),
                    CssBackgroundSizeComponent::Auto => true,
                }),
            _ => true,
        }) {
            return None;
        }
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

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCursor {
    Keyword(CssCursorKeyword),
    Urls(CssCursorUrls),
}

impl CssCursor {
    #[must_use]
    pub fn try_urls(urls: Vec<CssUrl>, fallback: CssCursorKeyword) -> Option<Self> {
        CssCursorUrlList::try_new(urls).map(|urls| Self::Urls(CssCursorUrls::new(urls, fallback)))
    }

    #[must_use]
    pub(crate) fn urls(urls: Vec<CssUrl>, fallback: CssCursorKeyword) -> Self {
        match Self::try_urls(urls, fallback) {
            Some(value) => value,
            None => panic!("cursor URL fallback must include at least one URL"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssCursorUrls {
    urls: CssCursorUrlList,
    fallback: CssCursorKeyword,
}

impl CssCursorUrls {
    #[must_use]
    pub const fn new(urls: CssCursorUrlList, fallback: CssCursorKeyword) -> Self {
        Self { urls, fallback }
    }

    #[must_use]
    pub const fn urls(&self) -> &CssCursorUrlList {
        &self.urls
    }

    #[must_use]
    pub const fn fallback(&self) -> CssCursorKeyword {
        self.fallback
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssCursorUrlList {
    urls: Vec<CssUrl>,
}

impl CssCursorUrlList {
    #[must_use]
    pub fn try_new(urls: Vec<CssUrl>) -> Option<Self> {
        if urls.is_empty() {
            None
        } else {
            Some(Self::new(urls))
        }
    }

    #[must_use]
    pub(crate) fn new(urls: Vec<CssUrl>) -> Self {
        Self { urls }
    }

    #[must_use]
    pub fn urls(&self) -> &[CssUrl] {
        &self.urls
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOutlineStyle {
    Auto,
    Border(CssBorderStyle),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOutlineWidth {
    Thin,
    Medium,
    Thick,
    Length(CssLength),
}

#[derive(Clone)]
pub struct CssOutline {
    width: Option<CssOutlineWidth>,
    style: Option<CssOutlineStyle>,
    color: Option<Box<CssParsedColor>>,
}

impl std::fmt::Debug for CssOutline {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssOutline")
            .field("width", &self.width)
            .field("style", &self.style)
            .field("color", &self.color())
            .finish()
    }
}

impl PartialEq for CssOutline {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.style == other.style
            && parsed_color_options_equal(self.color.as_deref(), other.color.as_deref())
    }
}

impl CssOutline {
    #[must_use]
    pub fn try_new(
        width: Option<CssOutlineWidth>,
        style: Option<CssOutlineStyle>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        if !width.as_ref().is_none_or(|width| match width {
            CssOutlineWidth::Length(value) => length_has_valid_calc_shape(value),
            _ => true,
        }) {
            return None;
        }
        if width.is_none() && style.is_none() && color.is_none() {
            None
        } else {
            Some(Self::new(width, style, color))
        }
    }

    #[must_use]
    pub(crate) fn new(
        width: Option<CssOutlineWidth>,
        style: Option<CssOutlineStyle>,
        color: Option<CssColor>,
    ) -> Self {
        Self::new_current(width, style, color.map(CssParsedColor::from_i01))
    }

    #[must_use]
    pub(crate) fn new_current(
        width: Option<CssOutlineWidth>,
        style: Option<CssOutlineStyle>,
        color: Option<CssParsedColor>,
    ) -> Self {
        Self {
            width,
            style,
            color: color.map(Box::new),
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
    pub const fn color(&self) -> Option<&CssColor> {
        match self.color.as_ref() {
            Some(color) => color.i01_subset(),
            None => None,
        }
    }

    /// Returns the exact authored current color in the shorthand, when present.
    #[must_use]
    pub const fn current_color(&self) -> Option<&CssAuthoredColor> {
        match self.color.as_ref() {
            Some(color) => Some(color.current()),
            None => None,
        }
    }

    pub(crate) const fn has_exact_i01_projection(&self) -> bool {
        match self.color.as_ref() {
            Some(color) => color.i01_subset().is_some(),
            None => true,
        }
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

/// A finite authored transform `<number>`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTransformNumber {
    Literal(CssFiniteNumber),
    Calculation(CssNumberCalculation),
}

/// A finite authored transform `<percentage>`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTransformPercentage {
    Literal(CssFiniteNumber),
    Calculation(CssPercentageCalculation),
}

/// One authored operand in a transform scale function.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTransformScaleComponent {
    Number(CssTransformNumber),
    Percentage(CssTransformPercentage),
}

/// An authored transform angle, including the unitless-zero branch.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTransformAngle {
    Zero,
    Literal(CssAngleLiteral),
    Calculation(CssAngleCalculation),
}

/// A checked authored transform `<length-percentage>`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformLengthPercentage {
    value: CssLength,
}

impl CssTransformLengthPercentage {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        if !length_has_valid_calc_shape(&value) {
            return None;
        }
        if matches!(
            value,
            CssLength::Px(_)
                | CssLength::Dimension(_)
                | CssLength::Percent(_)
                | CssLength::Zero
                | CssLength::Calc(_)
        ) {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// A checked authored transform `<length>`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformLength {
    value: CssLength,
}

impl CssTransformLength {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        let is_length = match &value {
            CssLength::Px(_) | CssLength::Dimension(_) | CssLength::Zero => true,
            CssLength::Calc(calculation) => !calculation.uses_percentage(),
            CssLength::Percent(_)
            | CssLength::Auto
            | CssLength::MinContent
            | CssLength::MaxContent
            | CssLength::FitContent
            | CssLength::Normal
            | CssLength::Thin
            | CssLength::Medium
            | CssLength::Thick => false,
        };
        is_length.then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// A checked non-negative authored transform `<length>` literal or symbolic calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformNonNegativeLength {
    value: CssLength,
}

impl CssTransformNonNegativeLength {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        let is_non_negative_length = match &value {
            CssLength::Px(value) => value.value() >= 0.0,
            CssLength::Dimension(value) => value.value() >= 0.0,
            CssLength::Zero => true,
            CssLength::Calc(calculation) => !calculation.uses_percentage(),
            CssLength::Percent(_)
            | CssLength::Auto
            | CssLength::MinContent
            | CssLength::MaxContent
            | CssLength::FitContent
            | CssLength::Normal
            | CssLength::Thin
            | CssLength::Medium
            | CssLength::Thick => false,
        };
        is_non_negative_length.then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformMatrix {
    components: [CssTransformNumber; 6],
}

impl CssTransformMatrix {
    pub(crate) const fn new(components: [CssTransformNumber; 6]) -> Self {
        Self { components }
    }

    #[must_use]
    pub const fn components(&self) -> &[CssTransformNumber; 6] {
        &self.components
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformMatrix3d {
    components: [CssTransformNumber; 16],
}

impl CssTransformMatrix3d {
    pub(crate) const fn new(components: [CssTransformNumber; 16]) -> Self {
        Self { components }
    }

    #[must_use]
    pub const fn components(&self) -> &[CssTransformNumber; 16] {
        &self.components
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTransformPerspective {
    None,
    Length(CssTransformNonNegativeLength),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformRotate3d {
    x: CssTransformNumber,
    y: CssTransformNumber,
    z: CssTransformNumber,
    angle: CssTransformAngle,
}

impl CssTransformRotate3d {
    pub(crate) const fn new(
        x: CssTransformNumber,
        y: CssTransformNumber,
        z: CssTransformNumber,
        angle: CssTransformAngle,
    ) -> Self {
        Self { x, y, z, angle }
    }

    #[must_use]
    pub const fn x(&self) -> &CssTransformNumber {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &CssTransformNumber {
        &self.y
    }

    #[must_use]
    pub const fn z(&self) -> &CssTransformNumber {
        &self.z
    }

    #[must_use]
    pub const fn angle(&self) -> &CssTransformAngle {
        &self.angle
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformScale {
    x: CssTransformNumber,
    y: Option<CssTransformNumber>,
}

impl CssTransformScale {
    pub(crate) const fn new(x: CssTransformNumber, y: Option<CssTransformNumber>) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> &CssTransformNumber {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> Option<&CssTransformNumber> {
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
    pub(crate) const fn new(
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

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformSkew {
    x: CssTransformAngle,
    y: Option<CssTransformAngle>,
}

impl CssTransformSkew {
    pub(crate) const fn new(x: CssTransformAngle, y: Option<CssTransformAngle>) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> &CssTransformAngle {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> Option<&CssTransformAngle> {
        self.y.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformTranslate {
    x: CssTransformLengthPercentage,
    y: Option<CssTransformLengthPercentage>,
}

impl CssTransformTranslate {
    pub(crate) const fn new(
        x: CssTransformLengthPercentage,
        y: Option<CssTransformLengthPercentage>,
    ) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> &CssTransformLengthPercentage {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> Option<&CssTransformLengthPercentage> {
        self.y.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformTranslate3d {
    x: CssTransformLengthPercentage,
    y: CssTransformLengthPercentage,
    z: CssTransformLength,
}

impl CssTransformTranslate3d {
    pub(crate) const fn new(
        x: CssTransformLengthPercentage,
        y: CssTransformLengthPercentage,
        z: CssTransformLength,
    ) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn x(&self) -> &CssTransformLengthPercentage {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &CssTransformLengthPercentage {
        &self.y
    }

    #[must_use]
    pub const fn z(&self) -> &CssTransformLength {
        &self.z
    }
}

/// A parser-produced authored transform function with an exact typed payload.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTransformFunctionValue {
    Matrix(CssTransformMatrix),
    Matrix3d(Box<CssTransformMatrix3d>),
    Perspective(CssTransformPerspective),
    Rotate(CssTransformAngle),
    Rotate3d(CssTransformRotate3d),
    RotateX(CssTransformAngle),
    RotateY(CssTransformAngle),
    RotateZ(CssTransformAngle),
    Scale(CssTransformScale),
    Scale3d(CssTransformScale3d),
    ScaleX(CssTransformNumber),
    ScaleY(CssTransformNumber),
    ScaleZ(CssTransformScaleComponent),
    Skew(CssTransformSkew),
    SkewX(CssTransformAngle),
    SkewY(CssTransformAngle),
    Translate(CssTransformTranslate),
    Translate3d(CssTransformTranslate3d),
    TranslateX(CssTransformLengthPercentage),
    TranslateY(CssTransformLengthPercentage),
    TranslateZ(CssTransformLength),
}

impl CssTransformFunctionValue {
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

/// A non-empty ordered list of current authored transform functions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformFunctionValueList {
    functions: Vec<CssTransformFunctionValue>,
}

impl CssTransformFunctionValueList {
    #[must_use]
    pub fn try_new(functions: Vec<CssTransformFunctionValue>) -> Option<Self> {
        (!functions.is_empty()).then_some(Self { functions })
    }

    #[must_use]
    pub fn functions(&self) -> &[CssTransformFunctionValue] {
        &self.functions
    }
}

/// The current authored value of the `transform` property.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTransformValue {
    None,
    Functions(CssTransformFunctionValueList),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedTransform {
    current: CssTransformValue,
    legacy: CssTransform,
}

impl CssParsedTransform {
    pub(crate) const fn new(current: CssTransformValue, legacy: CssTransform) -> Self {
        Self { current, legacy }
    }

    pub(crate) fn into_parts(self) -> (CssTransformValue, CssTransform) {
        (self.current, self.legacy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTransformFunction {
    kind: CssTransformFunctionKind,
    arguments: CssTransformArguments,
}

impl CssTransformFunction {
    #[must_use]
    pub const fn new(kind: CssTransformFunctionKind, arguments: CssTransformArguments) -> Self {
        Self { kind, arguments }
    }

    #[must_use]
    pub const fn kind(&self) -> CssTransformFunctionKind {
        self.kind
    }

    #[must_use]
    pub const fn arguments(&self) -> &CssTransformArguments {
        &self.arguments
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTransformFunctionList {
    functions: Vec<CssTransformFunction>,
}

impl CssTransformFunctionList {
    #[must_use]
    pub fn try_new(functions: Vec<CssTransformFunction>) -> Option<Self> {
        if functions.is_empty() {
            None
        } else {
            Some(Self::new(functions))
        }
    }

    #[must_use]
    pub(crate) fn new(functions: Vec<CssTransformFunction>) -> Self {
        Self { functions }
    }

    #[must_use]
    pub fn functions(&self) -> &[CssTransformFunction] {
        &self.functions
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct CssTranslateValues {
    values: Vec<CssLength>,
}

impl CssTranslateValues {
    #[must_use]
    pub fn try_new(values: Vec<CssLength>) -> Option<Self> {
        if !values.iter().all(length_has_valid_calc_shape) {
            return None;
        }
        if values.is_empty() || values.len() > 3 {
            None
        } else {
            Some(Self::new(values))
        }
    }

    #[must_use]
    pub(crate) fn new(values: Vec<CssLength>) -> Self {
        Self { values }
    }

    #[must_use]
    pub fn values(&self) -> &[CssLength] {
        &self.values
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRotate {
    None,
    Value(String),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssScale {
    None,
    Values(CssScaleValues),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssScaleValues {
    values: Vec<f32>,
}

impl CssScaleValues {
    #[must_use]
    pub fn try_new(values: Vec<f32>) -> Option<Self> {
        if values.is_empty() || values.len() > 3 || values.iter().any(|value| !value.is_finite()) {
            None
        } else {
            Some(Self::new(values))
        }
    }

    #[must_use]
    pub(crate) fn new(values: Vec<f32>) -> Self {
        Self { values }
    }

    #[must_use]
    pub fn values(&self) -> &[f32] {
        &self.values
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFilterFunction {
    Blur(CssFilterArguments),
    Brightness(CssFilterArguments),
    Contrast(CssFilterArguments),
    DropShadow(CssFilterArguments),
    Grayscale(CssFilterArguments),
    HueRotate(CssFilterArguments),
    Invert(CssFilterArguments),
    Opacity(CssFilterArguments),
    Saturate(CssFilterArguments),
    Sepia(CssFilterArguments),
    Url(CssUrl),
}

/// A checked authored non-negative filter `<number>`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilterNumber {
    Literal(CssNonNegativeNumber),
    Calculation(CssNumberCalculation),
}

/// A checked authored non-negative filter `<percentage>`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilterPercentage {
    Literal(CssNonNegativeNumber),
    Calculation(CssPercentageCalculation),
}

/// The optional amount accepted by a filter amount function.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilterAmount {
    Default,
    Number(CssFilterNumber),
    Percentage(CssFilterPercentage),
}

/// A checked specified filter blur length, including the omitted radius's 0px default.
///
/// The property's authored components preserve whether the radius was omitted.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFilterBlur {
    length: CssLength,
}

impl CssFilterBlur {
    #[must_use]
    pub fn try_new(length: CssLength) -> Option<Self> {
        let length = crate::numeric::admit_pure_length(length)?;
        if is_shadow_length(&length) && !length_has_negative_component(&length) {
            Some(Self { length })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn length(&self) -> &CssLength {
        &self.length
    }
}

/// A typed authored angle accepted by `hue-rotate()`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilterAngle {
    Zero,
    Literal(CssAngleLiteral),
    Calculation(CssAngleCalculation),
}

/// A filter `drop-shadow()` value, distinct from a box shadow.
#[derive(Clone)]
pub struct CssDropShadow {
    offset_x: CssLength,
    offset_y: CssLength,
    blur_radius: Option<CssLength>,
    color: Option<Box<CssParsedColor>>,
}

impl std::fmt::Debug for CssDropShadow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssDropShadow")
            .field("offset_x", &self.offset_x)
            .field("offset_y", &self.offset_y)
            .field("blur_radius", &self.blur_radius)
            .field("color", &self.color())
            .finish()
    }
}

impl PartialEq for CssDropShadow {
    fn eq(&self, other: &Self) -> bool {
        self.offset_x == other.offset_x
            && self.offset_y == other.offset_y
            && self.blur_radius == other.blur_radius
            && parsed_color_options_equal(self.color.as_deref(), other.color.as_deref())
    }
}

impl CssDropShadow {
    #[must_use]
    pub fn try_new(
        offset_x: CssLength,
        offset_y: CssLength,
        blur_radius: Option<CssLength>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        let offset_x = crate::numeric::admit_pure_length(offset_x)?;
        let offset_y = crate::numeric::admit_pure_length(offset_y)?;
        let blur_radius = match blur_radius {
            Some(value) => Some(crate::numeric::admit_pure_length(value)?),
            None => None,
        };
        if !is_shadow_length(&offset_x)
            || !is_shadow_length(&offset_y)
            || blur_radius
                .as_ref()
                .is_some_and(|blur| !is_shadow_length(blur) || length_has_negative_component(blur))
        {
            None
        } else {
            Some(Self {
                offset_x,
                offset_y,
                blur_radius,
                color: color.map(CssParsedColor::from_i01).map(Box::new),
            })
        }
    }

    #[must_use]
    pub const fn offset_x(&self) -> &CssLength {
        &self.offset_x
    }

    #[must_use]
    pub const fn offset_y(&self) -> &CssLength {
        &self.offset_y
    }

    #[must_use]
    pub const fn blur_radius(&self) -> Option<&CssLength> {
        self.blur_radius.as_ref()
    }

    #[must_use]
    pub const fn color(&self) -> Option<&CssColor> {
        match self.color.as_ref() {
            Some(color) => color.i01_subset(),
            None => None,
        }
    }

    /// Returns the exact authored current drop-shadow color, when present.
    #[must_use]
    pub const fn current_color(&self) -> Option<&CssAuthoredColor> {
        match self.color.as_ref() {
            Some(color) => Some(color.current()),
            None => None,
        }
    }

    pub(crate) fn try_new_current(
        offset_x: CssLength,
        offset_y: CssLength,
        blur_radius: Option<CssLength>,
        color: Option<CssParsedColor>,
    ) -> Option<Self> {
        let offset_x = crate::numeric::admit_pure_length(offset_x)?;
        let offset_y = crate::numeric::admit_pure_length(offset_y)?;
        let blur_radius = match blur_radius {
            Some(value) => Some(crate::numeric::admit_pure_length(value)?),
            None => None,
        };
        if !is_shadow_length(&offset_x)
            || !is_shadow_length(&offset_y)
            || blur_radius
                .as_ref()
                .is_some_and(|blur| !is_shadow_length(blur) || length_has_negative_component(blur))
        {
            None
        } else {
            Some(Self {
                offset_x,
                offset_y,
                blur_radius,
                color: color.map(Box::new),
            })
        }
    }
}

/// A parser-produced authored filter function with an exact typed payload.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilterFunctionValue {
    Blur(CssFilterBlur),
    Brightness(CssFilterAmount),
    Contrast(CssFilterAmount),
    DropShadow(CssDropShadow),
    Grayscale(CssFilterAmount),
    HueRotate(CssFilterAngle),
    Invert(CssFilterAmount),
    Opacity(CssFilterAmount),
    Saturate(CssFilterAmount),
    Sepia(CssFilterAmount),
    Url(CssUrl),
}

/// A non-empty ordered list of current authored filter functions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFilterFunctionValueList {
    functions: Vec<CssFilterFunctionValue>,
}

impl CssFilterFunctionValueList {
    #[must_use]
    pub fn try_new(functions: Vec<CssFilterFunctionValue>) -> Option<Self> {
        (!functions.is_empty()).then_some(Self { functions })
    }

    #[must_use]
    pub fn functions(&self) -> &[CssFilterFunctionValue] {
        &self.functions
    }
}

/// The current authored value of `filter` or `backdrop-filter`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilterValue {
    None,
    Functions(CssFilterFunctionValueList),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedFilter {
    current: CssFilterValue,
    legacy: Option<CssFilter>,
}

impl CssParsedFilter {
    pub(crate) const fn new(current: CssFilterValue, legacy: Option<CssFilter>) -> Self {
        Self { current, legacy }
    }

    pub(crate) fn into_parts(self) -> (CssFilterValue, Option<CssFilter>) {
        (self.current, self.legacy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFilterFunctionList {
    functions: Vec<CssFilterFunction>,
}

impl CssFilterFunctionList {
    #[must_use]
    pub fn try_new(functions: Vec<CssFilterFunction>) -> Option<Self> {
        if functions.is_empty() {
            None
        } else {
            Some(Self::new(functions))
        }
    }

    #[must_use]
    pub(crate) fn new(functions: Vec<CssFilterFunction>) -> Self {
        Self { functions }
    }

    #[must_use]
    pub fn functions(&self) -> &[CssFilterFunction] {
        &self.functions
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFilter {
    None,
    Functions(CssFilterFunctionList),
}

/// A checked non-negative authored shape `<length>`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssShapeLength {
    value: CssLength,
}

impl CssShapeLength {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        let value = crate::numeric::admit_pure_length(value)?;
        let valid = match &value {
            CssLength::Px(value) => value.value() >= 0.0,
            CssLength::Dimension(value) => value.value() >= 0.0,
            CssLength::Zero => true,
            CssLength::Calc(calculation) => !calculation.uses_percentage(),
            CssLength::Percent(_)
            | CssLength::Auto
            | CssLength::MinContent
            | CssLength::MaxContent
            | CssLength::FitContent
            | CssLength::Normal
            | CssLength::Thin
            | CssLength::Medium
            | CssLength::Thick => false,
        };
        valid.then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

/// A checked non-negative authored shape `<length-percentage>`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssShapeLengthPercentage {
    value: CssLength,
}

impl CssShapeLengthPercentage {
    #[must_use]
    pub fn try_new(value: CssLength) -> Option<Self> {
        if !length_has_valid_calc_shape(&value) {
            return None;
        }
        let valid = match &value {
            CssLength::Px(value) | CssLength::Percent(value) => value.value() >= 0.0,
            CssLength::Dimension(value) => value.value() >= 0.0,
            CssLength::Zero | CssLength::Calc(_) => true,
            CssLength::Auto
            | CssLength::MinContent
            | CssLength::MaxContent
            | CssLength::FitContent
            | CssLength::Normal
            | CssLength::Thin
            | CssLength::Medium
            | CssLength::Thick => false,
        };
        valid.then_some(Self { value })
    }

    #[must_use]
    pub const fn value(&self) -> &CssLength {
        &self.value
    }
}

fn is_shape_length_percentage(value: &CssLength) -> bool {
    matches!(
        value,
        CssLength::Px(_)
            | CssLength::Dimension(_)
            | CssLength::Percent(_)
            | CssLength::Zero
            | CssLength::Calc(_)
    )
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

/// The authored radius branch of a current `circle()` value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCircleRadius {
    Default,
    Extent(CssRadialExtent),
    Length(CssShapeLength),
}

/// A checked pair of non-negative authored ellipse radii.
#[derive(Clone, Debug, PartialEq)]
pub struct CssEllipseRadii {
    horizontal: CssShapeLengthPercentage,
    vertical: CssShapeLengthPercentage,
}

impl CssEllipseRadii {
    #[must_use]
    pub fn try_new(horizontal: CssLength, vertical: CssLength) -> Option<Self> {
        Some(Self {
            horizontal: CssShapeLengthPercentage::try_new(horizontal)?,
            vertical: CssShapeLengthPercentage::try_new(vertical)?,
        })
    }

    pub(crate) const fn new(
        horizontal: CssShapeLengthPercentage,
        vertical: CssShapeLengthPercentage,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    #[must_use]
    pub const fn horizontal(&self) -> &CssShapeLengthPercentage {
        &self.horizontal
    }

    #[must_use]
    pub const fn vertical(&self) -> &CssShapeLengthPercentage {
        &self.vertical
    }
}

/// The authored radius branch of a current `ellipse()` value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssEllipseRadius {
    Default,
    Extent(CssRadialExtent),
    Radii(CssEllipseRadii),
}

/// A current authored `circle()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCircleShape {
    radius: CssCircleRadius,
    position: Option<CssPositionValue>,
}

impl CssCircleShape {
    #[must_use]
    pub const fn new(radius: CssCircleRadius, position: Option<CssPositionValue>) -> Self {
        Self { radius, position }
    }

    #[must_use]
    pub const fn radius(&self) -> &CssCircleRadius {
        &self.radius
    }

    #[must_use]
    pub const fn position(&self) -> Option<&CssPositionValue> {
        self.position.as_ref()
    }
}

/// A current authored `ellipse()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssEllipseShape {
    radius: CssEllipseRadius,
    position: Option<CssPositionValue>,
}

impl CssEllipseShape {
    #[must_use]
    pub const fn new(radius: CssEllipseRadius, position: Option<CssPositionValue>) -> Self {
        Self { radius, position }
    }

    #[must_use]
    pub const fn radius(&self) -> &CssEllipseRadius {
        &self.radius
    }

    #[must_use]
    pub const fn position(&self) -> Option<&CssPositionValue> {
        self.position.as_ref()
    }
}

/// One-to-four authored inset `<length-percentage>` offsets.
#[derive(Clone, Debug, PartialEq)]
pub struct CssInsetShapeOffsets {
    values: Vec<CssLength>,
}

impl CssInsetShapeOffsets {
    #[must_use]
    pub fn try_new(values: Vec<CssLength>) -> Option<Self> {
        if !values.iter().all(length_has_valid_calc_shape) {
            return None;
        }
        ((1..=4).contains(&values.len()) && values.iter().all(is_shape_length_percentage))
            .then_some(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[CssLength] {
        &self.values
    }
}

/// A current authored `inset()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssInsetShape {
    offsets: CssInsetShapeOffsets,
    round: Option<CssBorderRadii>,
}

impl CssInsetShape {
    #[must_use]
    pub const fn new(offsets: CssInsetShapeOffsets, round: Option<CssBorderRadii>) -> Self {
        Self { offsets, round }
    }

    #[must_use]
    pub const fn offsets(&self) -> &CssInsetShapeOffsets {
        &self.offsets
    }

    #[must_use]
    pub const fn round(&self) -> Option<&CssBorderRadii> {
        self.round.as_ref()
    }
}

/// The optional authored fill rule of `polygon()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPolygonFillRule {
    Nonzero,
    Evenodd,
}

/// One checked authored point in a current `polygon()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPolygonPoint {
    x: CssLength,
    y: CssLength,
}

impl CssPolygonPoint {
    #[must_use]
    pub fn try_new(x: CssLength, y: CssLength) -> Option<Self> {
        if !(length_has_valid_calc_shape(&x) && length_has_valid_calc_shape(&y)) {
            return None;
        }
        (is_shape_length_percentage(&x) && is_shape_length_percentage(&y)).then_some(Self { x, y })
    }

    pub(crate) const fn new(x: CssLength, y: CssLength) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> &CssLength {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &CssLength {
        &self.y
    }
}

/// A non-empty authored polygon point list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPolygonPointList {
    points: Vec<CssPolygonPoint>,
}

impl CssPolygonPointList {
    #[must_use]
    pub fn try_new(points: Vec<CssPolygonPoint>) -> Option<Self> {
        (!points.is_empty()).then_some(Self { points })
    }

    #[must_use]
    pub fn points(&self) -> &[CssPolygonPoint] {
        &self.points
    }
}

/// A current authored `polygon()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPolygonShape {
    fill_rule: Option<CssPolygonFillRule>,
    round: Option<CssShapeLength>,
    points: CssPolygonPointList,
}

impl CssPolygonShape {
    #[must_use]
    pub const fn new(
        fill_rule: Option<CssPolygonFillRule>,
        round: Option<CssShapeLength>,
        points: CssPolygonPointList,
    ) -> Self {
        Self {
            fill_rule,
            round,
            points,
        }
    }

    #[must_use]
    pub const fn fill_rule(&self) -> Option<CssPolygonFillRule> {
        self.fill_rule
    }

    #[must_use]
    pub const fn round(&self) -> Option<&CssShapeLength> {
        self.round.as_ref()
    }

    #[must_use]
    pub const fn points(&self) -> &CssPolygonPointList {
        &self.points
    }
}

/// A selected current authored basic-shape function.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBasicShapeValue {
    Inset(Box<CssInsetShape>),
    Circle(CssCircleShape),
    Ellipse(CssEllipseShape),
    Polygon(CssPolygonShape),
}

/// The exact current authored subset of `clip-path`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssClipPathValue {
    None,
    Url(CssUrl),
    BasicShape(CssBasicShapeValue),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBasicShape {
    Inset(CssBasicShapeArguments),
    Circle(CssBasicShapeArguments),
    Ellipse(CssBasicShapeArguments),
    Polygon(CssBasicShapeArguments),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssClipPath {
    None,
    Url(CssUrl),
    BasicShape(CssBasicShape),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedClipPath {
    current: Option<CssClipPathValue>,
    legacy: Option<CssClipPath>,
}

impl CssParsedClipPath {
    pub(crate) const fn new(
        current: Option<CssClipPathValue>,
        legacy: Option<CssClipPath>,
    ) -> Self {
        Self { current, legacy }
    }

    pub(crate) fn into_parts(self) -> (Option<CssClipPathValue>, Option<CssClipPath>) {
        (self.current, self.legacy)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssMaskLayer {
    image: Option<CssImageLayer>,
    position: Option<CssPosition>,
    size: Option<CssBackgroundSize>,
    repeat: Option<CssBackgroundRepeat>,
}

impl CssMaskLayer {
    #[must_use]
    pub fn try_new(
        image: Option<CssImageLayer>,
        position: Option<CssPosition>,
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
        image: Option<CssImageLayer>,
        position: Option<CssPosition>,
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

#[derive(Clone, Copy, PartialEq)]
pub struct CssTime {
    value: CssFiniteNumber,
    unit: CssTimeUnit,
}

impl std::fmt::Debug for CssTime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssTime")
            .field("value", &self.value.value())
            .field("unit", &self.unit)
            .finish()
    }
}

impl CssTime {
    #[must_use]
    pub const fn try_new(value: f32, unit: CssTimeUnit) -> Option<Self> {
        match CssFiniteNumber::try_new(value) {
            Some(value) if value.value() >= 0.0 => Some(Self { value, unit }),
            Some(_) | None => None,
        }
    }

    #[must_use]
    pub const fn try_seconds(value: f32) -> Option<Self> {
        Self::try_new(value, CssTimeUnit::Seconds)
    }

    #[must_use]
    pub const fn try_milliseconds(value: f32) -> Option<Self> {
        Self::try_new(value, CssTimeUnit::Milliseconds)
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }

    #[must_use]
    pub const fn unit(self) -> CssTimeUnit {
        self.unit
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTimeList {
    times: Vec<CssTime>,
}

impl CssTimeList {
    #[must_use]
    pub fn try_new(times: Vec<CssTime>) -> Option<Self> {
        if times.is_empty() {
            None
        } else {
            Some(Self::new(times))
        }
    }

    #[must_use]
    pub(crate) fn new(times: Vec<CssTime>) -> Self {
        Self { times }
    }

    #[must_use]
    pub fn times(&self) -> &[CssTime] {
        &self.times
    }
}

/// A finite non-negative authored duration literal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssDurationLiteral {
    value: CssFiniteNumber,
    unit: CssTimeUnit,
}

impl CssDurationLiteral {
    #[must_use]
    pub const fn try_new(value: f32, unit: CssTimeUnit) -> Option<Self> {
        match CssFiniteNumber::try_new(value) {
            Some(value) if value.value() >= 0.0 => Some(Self { value, unit }),
            Some(_) | None => None,
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }

    #[must_use]
    pub const fn unit(self) -> CssTimeUnit {
        self.unit
    }
}

/// An authored transition or animation duration.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssDuration {
    Literal(CssDurationLiteral),
    Calculation(CssTimeCalculation),
}

/// A non-empty authored duration list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssDurationList {
    values: Vec<CssDuration>,
}

impl CssDurationList {
    #[must_use]
    pub fn try_new(values: Vec<CssDuration>) -> Option<Self> {
        if values.is_empty() {
            None
        } else {
            Some(Self { values })
        }
    }

    #[must_use]
    pub fn values(&self) -> &[CssDuration] {
        &self.values
    }
}

/// An authored signed transition or animation delay.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssDelay {
    Literal(CssDelayLiteral),
    Calculation(CssTimeCalculation),
}

/// A non-empty authored delay list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssDelayList {
    values: Vec<CssDelay>,
}

impl CssDelayList {
    #[must_use]
    pub fn try_new(values: Vec<CssDelay>) -> Option<Self> {
        if values.is_empty() {
            None
        } else {
            Some(Self { values })
        }
    }

    #[must_use]
    pub fn values(&self) -> &[CssDelay] {
        &self.values
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssEasing {
    Ease,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    StepStart,
    StepEnd,
    CubicBezier(CssEasingArguments),
    Steps(CssEasingArguments),
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

/// A finite authored easing `<number>` or a symbolic number calculation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssEasingNumber {
    Literal(CssFiniteNumber),
    Calculation(CssNumberCalculation),
}

/// A checked cubic-bezier x coordinate.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCubicBezierX {
    value: CssEasingNumber,
}

impl CssCubicBezierX {
    #[must_use]
    pub fn try_new(value: CssEasingNumber) -> Option<Self> {
        match value {
            CssEasingNumber::Literal(value) if !(0.0..=1.0).contains(&value.value()) => None,
            value => Some(Self { value }),
        }
    }

    #[must_use]
    pub const fn value(&self) -> &CssEasingNumber {
        &self.value
    }
}

/// A checked authored `cubic-bezier()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCubicBezier {
    x1: CssCubicBezierX,
    y1: CssEasingNumber,
    x2: CssCubicBezierX,
    y2: CssEasingNumber,
}

impl CssCubicBezier {
    #[must_use]
    pub fn try_new(
        x1: CssEasingNumber,
        y1: CssEasingNumber,
        x2: CssEasingNumber,
        y2: CssEasingNumber,
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
    pub const fn y1(&self) -> &CssEasingNumber {
        &self.y1
    }

    #[must_use]
    pub const fn x2(&self) -> &CssCubicBezierX {
        &self.x2
    }

    #[must_use]
    pub const fn y2(&self) -> &CssEasingNumber {
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

#[derive(Clone, Debug, PartialEq)]
enum CssStepCountValue {
    Literal(i32),
    Calculation(CssIntegerCalculation),
}

/// A positive authored step count or a symbolic integer calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssStepCount {
    value: CssStepCountValue,
}

impl CssStepCount {
    #[must_use]
    pub const fn try_literal(value: i32) -> Option<Self> {
        if value > 0 {
            Some(Self {
                value: CssStepCountValue::Literal(value),
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn from_calculation(value: CssIntegerCalculation) -> Self {
        Self {
            value: CssStepCountValue::Calculation(value),
        }
    }

    #[must_use]
    pub const fn literal(&self) -> Option<i32> {
        match self.value {
            CssStepCountValue::Literal(value) => Some(value),
            CssStepCountValue::Calculation(_) => None,
        }
    }

    #[must_use]
    pub const fn calculation(&self) -> Option<&CssIntegerCalculation> {
        match &self.value {
            CssStepCountValue::Literal(_) => None,
            CssStepCountValue::Calculation(value) => Some(value),
        }
    }
}

/// A checked authored `steps()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSteps {
    count: CssStepCount,
    position: Option<CssStepPosition>,
}

impl CssSteps {
    #[must_use]
    pub fn try_new(count: CssStepCount, position: Option<CssStepPosition>) -> Option<Self> {
        if matches!(position, Some(CssStepPosition::JumpNone)) && matches!(count.literal(), Some(1))
        {
            None
        } else {
            Some(Self { count, position })
        }
    }

    #[must_use]
    pub const fn count(&self) -> &CssStepCount {
        &self.count
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssStepPosition> {
        self.position
    }
}

/// A parser-produced current authored easing function.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssEasingValue {
    Keyword(CssEasingKeyword),
    CubicBezier(CssCubicBezier),
    Steps(CssSteps),
}

/// A non-empty comma-separated list of current authored easing functions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssEasingValueList {
    values: Vec<CssEasingValue>,
}

impl CssEasingValueList {
    #[must_use]
    pub fn try_new(values: Vec<CssEasingValue>) -> Option<Self> {
        (!values.is_empty()).then_some(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[CssEasingValue] {
        &self.values
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedEasing {
    current: CssEasingValue,
    legacy: Option<CssEasing>,
}

impl CssParsedEasing {
    pub(crate) const fn new(current: CssEasingValue, legacy: Option<CssEasing>) -> Self {
        Self { current, legacy }
    }

    pub(crate) fn into_parts(self) -> (CssEasingValue, Option<CssEasing>) {
        (self.current, self.legacy)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedEasingList {
    current: CssEasingValueList,
    legacy: Option<CssEasingList>,
}

impl CssParsedEasingList {
    pub(crate) const fn new(current: CssEasingValueList, legacy: Option<CssEasingList>) -> Self {
        Self { current, legacy }
    }

    pub(crate) fn into_parts(self) -> (CssEasingValueList, Option<CssEasingList>) {
        (self.current, self.legacy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssEasingList {
    easings: Vec<CssEasing>,
}

impl CssEasingList {
    #[must_use]
    pub fn try_new(easings: Vec<CssEasing>) -> Option<Self> {
        if easings.is_empty() {
            None
        } else {
            Some(Self::new(easings))
        }
    }

    #[must_use]
    pub(crate) fn new(easings: Vec<CssEasing>) -> Self {
        Self { easings }
    }

    #[must_use]
    pub fn easings(&self) -> &[CssEasing] {
        &self.easings
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTransitionProperty {
    All,
    None,
    Custom(CssCustomIdent),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTransitionPropertyList {
    properties: Vec<CssTransitionProperty>,
}

impl CssTransitionPropertyList {
    #[must_use]
    pub fn try_new(properties: Vec<CssTransitionProperty>) -> Option<Self> {
        if properties.is_empty() {
            None
        } else {
            Some(Self::new(properties))
        }
    }

    #[must_use]
    pub(crate) fn new(properties: Vec<CssTransitionProperty>) -> Self {
        Self { properties }
    }

    #[must_use]
    pub fn properties(&self) -> &[CssTransitionProperty] {
        &self.properties
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationName {
    None,
    Custom(CssCustomIdent),
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

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationIterationCount {
    Infinite,
    Number(CssAnimationIterationNumber),
}

impl CssAnimationIterationCount {
    #[must_use]
    pub const fn try_number(value: f32) -> Option<Self> {
        match CssAnimationIterationNumber::try_new(value) {
            Some(value) => Some(Self::Number(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct CssAnimationIterationNumber {
    value: CssFiniteNumber,
}

impl std::fmt::Debug for CssAnimationIterationNumber {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssAnimationIterationNumber")
            .field("value", &self.value.value())
            .finish()
    }
}

impl CssAnimationIterationNumber {
    #[must_use]
    pub const fn try_new(value: f32) -> Option<Self> {
        match CssFiniteNumber::try_new(value) {
            Some(value) if value.value() >= 0.0 => Some(Self { value }),
            Some(_) | None => None,
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimationIterationCountList {
    counts: Vec<CssAnimationIterationCount>,
}

impl CssAnimationIterationCountList {
    #[must_use]
    pub fn try_new(counts: Vec<CssAnimationIterationCount>) -> Option<Self> {
        if counts.is_empty() {
            None
        } else {
            Some(Self::new(counts))
        }
    }

    #[must_use]
    pub(crate) fn new(counts: Vec<CssAnimationIterationCount>) -> Self {
        Self { counts }
    }

    #[must_use]
    pub fn counts(&self) -> &[CssAnimationIterationCount] {
        &self.counts
    }
}

/// A current authored animation iteration-count value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationIterationValue {
    Infinite,
    Number(CssAnimationIterationNumber),
    Calculation(CssNumberCalculation),
}

/// A non-empty list of current authored animation iteration-count values.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimationIterationValueList {
    values: Vec<CssAnimationIterationValue>,
}

impl CssAnimationIterationValueList {
    #[must_use]
    pub fn try_new(values: Vec<CssAnimationIterationValue>) -> Option<Self> {
        if values.is_empty() {
            None
        } else {
            Some(Self { values })
        }
    }

    #[must_use]
    pub fn values(&self) -> &[CssAnimationIterationValue] {
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransition {
    property: Option<CssTransitionProperty>,
    duration: Option<CssTime>,
    delay: Option<CssTime>,
    timing_function: Option<CssEasing>,
}

impl CssTransition {
    #[must_use]
    pub fn try_new(
        property: Option<CssTransitionProperty>,
        duration: Option<CssTime>,
        delay: Option<CssTime>,
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
    pub const fn duration(&self) -> Option<CssTime> {
        self.duration
    }

    #[must_use]
    pub const fn delay(&self) -> Option<CssTime> {
        self.delay
    }

    #[must_use]
    pub const fn timing_function(&self) -> Option<&CssEasing> {
        self.timing_function.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransitionList {
    items: Vec<CssTransition>,
}

impl CssTransitionList {
    #[must_use]
    pub fn try_new(items: Vec<CssTransition>) -> Option<Self> {
        if items.is_empty() {
            None
        } else {
            Some(Self::new(items))
        }
    }

    #[must_use]
    pub(crate) fn new(items: Vec<CssTransition>) -> Self {
        Self { items }
    }

    #[must_use]
    pub fn items(&self) -> &[CssTransition] {
        &self.items
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimation {
    name: Option<CssAnimationName>,
    duration: Option<CssTime>,
    delay: Option<CssTime>,
    timing_function: Option<CssEasing>,
    iteration_count: Option<CssAnimationIterationCount>,
    direction: Option<CssAnimationDirection>,
    fill_mode: Option<CssAnimationFillMode>,
    play_state: Option<CssAnimationPlayState>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CssAnimationComponents {
    pub name: Option<CssAnimationName>,
    pub duration: Option<CssTime>,
    pub delay: Option<CssTime>,
    pub timing_function: Option<CssEasing>,
    pub iteration_count: Option<CssAnimationIterationCount>,
    pub direction: Option<CssAnimationDirection>,
    pub fill_mode: Option<CssAnimationFillMode>,
    pub play_state: Option<CssAnimationPlayState>,
}

impl CssAnimation {
    #[must_use]
    pub fn try_new(components: CssAnimationComponents) -> Option<Self> {
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
    pub const fn duration(&self) -> Option<CssTime> {
        self.duration
    }

    #[must_use]
    pub const fn delay(&self) -> Option<CssTime> {
        self.delay
    }

    #[must_use]
    pub const fn timing_function(&self) -> Option<&CssEasing> {
        self.timing_function.as_ref()
    }

    #[must_use]
    pub const fn iteration_count(&self) -> Option<CssAnimationIterationCount> {
        self.iteration_count
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

#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimationList {
    items: Vec<CssAnimation>,
}

impl CssAnimationList {
    #[must_use]
    pub fn try_new(items: Vec<CssAnimation>) -> Option<Self> {
        if items.is_empty() {
            None
        } else {
            Some(Self::new(items))
        }
    }

    #[must_use]
    pub(crate) fn new(items: Vec<CssAnimation>) -> Self {
        Self { items }
    }

    #[must_use]
    pub fn items(&self) -> &[CssAnimation] {
        &self.items
    }
}

/// A parser-owned current transition value with distinct duration and delay domains.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransitionValue {
    property: Option<CssTransitionProperty>,
    duration: Option<CssDuration>,
    delay: Option<CssDelay>,
    timing_function: Option<CssEasingValue>,
    legacy_timing_function: Option<CssEasing>,
}

impl CssTransitionValue {
    #[must_use]
    pub(crate) fn try_new(
        property: Option<CssTransitionProperty>,
        duration: Option<CssDuration>,
        delay: Option<CssDelay>,
        timing_function: Option<CssParsedEasing>,
    ) -> Option<Self> {
        if property.is_none() && duration.is_none() && delay.is_none() && timing_function.is_none()
        {
            None
        } else {
            let (timing_function, legacy_timing_function) = match timing_function {
                Some(value) => {
                    let (current, legacy) = value.into_parts();
                    (Some(current), legacy)
                }
                None => (None, None),
            };
            Some(Self {
                property,
                duration,
                delay,
                timing_function,
                legacy_timing_function,
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
    pub const fn delay(&self) -> Option<&CssDelay> {
        self.delay.as_ref()
    }

    #[must_use]
    pub const fn timing_function(&self) -> Option<&CssEasing> {
        self.legacy_timing_function.as_ref()
    }

    /// Returns the exact current authored easing value.
    #[must_use]
    pub const fn current_timing_function(&self) -> Option<&CssEasingValue> {
        self.timing_function.as_ref()
    }
}

/// A parser-owned non-empty current transition list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransitionValueList {
    values: Vec<CssTransitionValue>,
}

impl CssTransitionValueList {
    #[must_use]
    pub(crate) fn try_new(values: Vec<CssTransitionValue>) -> Option<Self> {
        if values.is_empty() {
            None
        } else {
            Some(Self { values })
        }
    }

    #[must_use]
    pub fn values(&self) -> &[CssTransitionValue] {
        &self.values
    }
}

/// A parser-owned current animation value with distinct timing domains.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimationValue {
    name: Option<CssAnimationName>,
    duration: Option<CssDuration>,
    delay: Option<CssDelay>,
    timing_function: Option<CssEasingValue>,
    legacy_timing_function: Option<CssEasing>,
    iteration_count: Option<CssAnimationIterationValue>,
    direction: Option<CssAnimationDirection>,
    fill_mode: Option<CssAnimationFillMode>,
    play_state: Option<CssAnimationPlayState>,
}

impl CssAnimationValue {
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "the parser-owned constructor preserves the eight distinct animation components"
    )]
    pub(crate) fn try_new(
        name: Option<CssAnimationName>,
        duration: Option<CssDuration>,
        delay: Option<CssDelay>,
        timing_function: Option<CssParsedEasing>,
        iteration_count: Option<CssAnimationIterationValue>,
        direction: Option<CssAnimationDirection>,
        fill_mode: Option<CssAnimationFillMode>,
        play_state: Option<CssAnimationPlayState>,
    ) -> Option<Self> {
        if name.is_none()
            && duration.is_none()
            && delay.is_none()
            && timing_function.is_none()
            && iteration_count.is_none()
            && direction.is_none()
            && fill_mode.is_none()
            && play_state.is_none()
        {
            None
        } else {
            let (timing_function, legacy_timing_function) = match timing_function {
                Some(value) => {
                    let (current, legacy) = value.into_parts();
                    (Some(current), legacy)
                }
                None => (None, None),
            };
            Some(Self {
                name,
                duration,
                delay,
                timing_function,
                legacy_timing_function,
                iteration_count,
                direction,
                fill_mode,
                play_state,
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
    pub const fn delay(&self) -> Option<&CssDelay> {
        self.delay.as_ref()
    }

    #[must_use]
    pub const fn timing_function(&self) -> Option<&CssEasing> {
        self.legacy_timing_function.as_ref()
    }

    /// Returns the exact current authored easing value.
    #[must_use]
    pub const fn current_timing_function(&self) -> Option<&CssEasingValue> {
        self.timing_function.as_ref()
    }

    #[must_use]
    pub const fn iteration_count(&self) -> Option<&CssAnimationIterationValue> {
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

/// A parser-owned non-empty current animation list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimationValueList {
    values: Vec<CssAnimationValue>,
}

impl CssAnimationValueList {
    #[must_use]
    pub(crate) fn try_new(values: Vec<CssAnimationValue>) -> Option<Self> {
        if values.is_empty() {
            None
        } else {
            Some(Self { values })
        }
    }

    #[must_use]
    pub fn values(&self) -> &[CssAnimationValue] {
        &self.values
    }
}

pub(crate) fn length_has_negative_component(length: &CssLength) -> bool {
    match length {
        CssLength::Px(value) | CssLength::Percent(value) => value.value() < 0.0,
        CssLength::Dimension(length) => length.value() < 0.0,
        CssLength::Calc(calc) => calc_has_negative_component(calc),
        CssLength::Zero
        | CssLength::Auto
        | CssLength::MinContent
        | CssLength::MaxContent
        | CssLength::FitContent
        | CssLength::Normal
        | CssLength::Thin
        | CssLength::Medium
        | CssLength::Thick => false,
    }
}

pub(crate) fn calc_has_negative_component(calc: &CssCalcLength) -> bool {
    match calc {
        CssCalcLength::Px(value) | CssCalcLength::Percent(value) => value.value() < 0.0,
        CssCalcLength::Dimension(length) => length.value() < 0.0,
        CssCalcLength::Sum(terms) => terms
            .iter()
            .any(|term| calc_has_negative_component(term.value())),
        CssCalcLength::Typed(_) => false,
    }
}

/// A parser-owned authored color that preserves its specified Color 4 branch.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredColor {
    representation: CssAuthoredColorRepresentation,
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredColorRepresentation {
    CurrentColor,
    Transparent,
    Hex(CssHexColor),
    Named(CssNamedColor),
    System(CssAuthoredSystemColor),
    Rgb(CssAuthoredRgbColor),
    Hsl(CssAuthoredHslColor),
    Hwb(CssAuthoredHwbColor),
    Lab(CssAuthoredLabColor),
    Lch(CssAuthoredLchColor),
    Oklab(CssAuthoredLabColor),
    Oklch(CssAuthoredLchColor),
    Predefined(CssAuthoredPredefinedColor),
    Relative(CssAuthoredRelativeColor),
    ColorMix(CssAuthoredColorMix),
    PreservedI01(CssColor),
}

impl CssAuthoredColor {
    pub(crate) const fn current_color() -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::CurrentColor,
        }
    }

    pub(crate) const fn transparent() -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Transparent,
        }
    }

    pub(crate) const fn hex(value: CssHexColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Hex(value),
        }
    }

    pub(crate) const fn from_named(value: CssNamedColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Named(value),
        }
    }

    pub(crate) const fn from_system(value: CssAuthoredSystemColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::System(value),
        }
    }

    pub(crate) const fn rgb(value: CssAuthoredRgbColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Rgb(value),
        }
    }

    pub(crate) const fn hsl(value: CssAuthoredHslColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Hsl(value),
        }
    }

    pub(crate) const fn hwb(value: CssAuthoredHwbColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Hwb(value),
        }
    }

    pub(crate) const fn lab(value: CssAuthoredLabColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Lab(value),
        }
    }

    pub(crate) const fn lch(value: CssAuthoredLchColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Lch(value),
        }
    }

    pub(crate) const fn oklab(value: CssAuthoredLabColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Oklab(value),
        }
    }

    pub(crate) const fn oklch(value: CssAuthoredLchColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Oklch(value),
        }
    }

    pub(crate) const fn predefined(value: CssAuthoredPredefinedColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Predefined(value),
        }
    }

    pub(crate) const fn relative(value: CssAuthoredRelativeColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Relative(value),
        }
    }

    pub(crate) const fn color_mix(value: CssAuthoredColorMix) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::ColorMix(value),
        }
    }

    pub(crate) const fn preserved_i01(value: CssColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::PreservedI01(value),
        }
    }

    #[must_use]
    pub const fn is_current_color(&self) -> bool {
        matches!(
            self.representation,
            CssAuthoredColorRepresentation::CurrentColor
        )
    }

    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        matches!(
            self.representation,
            CssAuthoredColorRepresentation::Transparent
        )
    }

    #[must_use]
    pub const fn hex_value(&self) -> Option<&CssHexColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Hex(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn named(&self) -> Option<&CssNamedColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Named(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn system(&self) -> Option<CssAuthoredSystemColor> {
        match self.representation {
            CssAuthoredColorRepresentation::System(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn rgb_value(&self) -> Option<&CssAuthoredRgbColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Rgb(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn hsl_value(&self) -> Option<&CssAuthoredHslColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Hsl(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn hwb_value(&self) -> Option<&CssAuthoredHwbColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Hwb(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn lab_value(&self) -> Option<&CssAuthoredLabColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Lab(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn lch_value(&self) -> Option<&CssAuthoredLchColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Lch(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn oklab_value(&self) -> Option<&CssAuthoredLabColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Oklab(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn oklch_value(&self) -> Option<&CssAuthoredLchColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Oklch(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn predefined_value(&self) -> Option<&CssAuthoredPredefinedColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Predefined(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the typed preserved Color 5 relative-color branch, when present.
    #[must_use]
    pub const fn relative_value(&self) -> Option<&CssAuthoredRelativeColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Relative(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the checked preserved Color 5 `color-mix()` branch, when present.
    #[must_use]
    pub const fn color_mix_value(&self) -> Option<&CssAuthoredColorMix> {
        match &self.representation {
            CssAuthoredColorRepresentation::ColorMix(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match &self.representation {
            CssAuthoredColorRepresentation::CurrentColor => "currentcolor",
            CssAuthoredColorRepresentation::Transparent => "transparent",
            CssAuthoredColorRepresentation::Hex(_) => "hex",
            CssAuthoredColorRepresentation::Named(_) => "named",
            CssAuthoredColorRepresentation::System(_) => "system",
            CssAuthoredColorRepresentation::Rgb(_) => "rgb",
            CssAuthoredColorRepresentation::Hsl(_) => "hsl",
            CssAuthoredColorRepresentation::Hwb(_) => "hwb",
            CssAuthoredColorRepresentation::Lab(_) => "lab",
            CssAuthoredColorRepresentation::Lch(_) => "lch",
            CssAuthoredColorRepresentation::Oklab(_) => "oklab",
            CssAuthoredColorRepresentation::Oklch(_) => "oklch",
            CssAuthoredColorRepresentation::Predefined(_) => "color",
            CssAuthoredColorRepresentation::Relative(_) => "relative",
            CssAuthoredColorRepresentation::ColorMix(_) => "color-mix",
            CssAuthoredColorRepresentation::PreservedI01(value) => value.kind_name(),
        }
    }

    pub(crate) fn has_exact_i01_projection(&self) -> bool {
        match &self.representation {
            CssAuthoredColorRepresentation::Lab(value)
            | CssAuthoredColorRepresentation::Oklab(value) => {
                authored_alpha_has_exact_i01_projection(value.alpha())
            }
            CssAuthoredColorRepresentation::Lch(value)
            | CssAuthoredColorRepresentation::Oklch(value) => {
                authored_alpha_has_exact_i01_projection(value.alpha())
            }
            CssAuthoredColorRepresentation::Predefined(value) => {
                authored_alpha_has_exact_i01_projection(value.alpha())
            }
            CssAuthoredColorRepresentation::Relative(_) => true,
            CssAuthoredColorRepresentation::ColorMix(value) => value.has_exact_i01_projection(),
            CssAuthoredColorRepresentation::CurrentColor
            | CssAuthoredColorRepresentation::Transparent
            | CssAuthoredColorRepresentation::Hex(_)
            | CssAuthoredColorRepresentation::Named(_)
            | CssAuthoredColorRepresentation::System(_)
            | CssAuthoredColorRepresentation::Rgb(_)
            | CssAuthoredColorRepresentation::Hsl(_)
            | CssAuthoredColorRepresentation::Hwb(_)
            | CssAuthoredColorRepresentation::PreservedI01(_) => true,
        }
    }
}

fn authored_alpha_has_exact_i01_projection(alpha: Option<&CssAuthoredColorComponent>) -> bool {
    match alpha {
        None | Some(CssAuthoredColorComponent::None) => true,
        Some(CssAuthoredColorComponent::Number(value)) => (0.0..=1.0).contains(&value.value()),
        Some(CssAuthoredColorComponent::Percentage(value)) => {
            (0.0..=100.0).contains(&value.value())
        }
        Some(
            CssAuthoredColorComponent::NumberCalculation(_)
            | CssAuthoredColorComponent::PercentageCalculation(_),
        ) => false,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssHexColor {
    digits: String,
}

impl CssHexColor {
    pub(crate) fn new(digits: impl Into<String>) -> Self {
        Self {
            digits: digits.into(),
        }
    }

    #[must_use]
    pub fn digits(&self) -> &str {
        &self.digits
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNamedColor {
    name: String,
}

impl CssNamedColor {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredSystemColor {
    Canvas,
    CanvasText,
    LinkText,
    VisitedText,
    ActiveText,
    ButtonFace,
    ButtonText,
    ButtonBorder,
    Field,
    FieldText,
    Highlight,
    HighlightText,
    Mark,
    MarkText,
    GrayText,
    SelectedItem,
    SelectedItemText,
    AccentColor,
    AccentColorText,
    ActiveBorder,
    ActiveCaption,
    AppWorkspace,
    Background,
    ButtonHighlight,
    ButtonShadow,
    CaptionText,
    InactiveBorder,
    InactiveCaption,
    InactiveCaptionText,
    InfoBackground,
    InfoText,
    Menu,
    MenuText,
    Scrollbar,
    ThreeDDarkShadow,
    ThreeDFace,
    ThreeDHighlight,
    ThreeDLightShadow,
    ThreeDShadow,
    Window,
    WindowFrame,
    WindowText,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredColorSyntax {
    Legacy,
    Modern,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredColorComponent {
    None,
    Number(CssFiniteNumber),
    Percentage(CssFiniteNumber),
    NumberCalculation(CssNumberCalculation),
    PercentageCalculation(CssPercentageCalculation),
}

impl CssAuthoredColorComponent {
    pub(crate) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub(crate) const fn domain(&self) -> Option<CssCalculationType> {
        match self {
            Self::None => None,
            Self::Number(_) | Self::NumberCalculation(_) => Some(CssCalculationType::Number),
            Self::Percentage(_) | Self::PercentageCalculation(_) => {
                Some(CssCalculationType::Percentage)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredHue {
    None,
    Number(CssFiniteNumber),
    Angle(CssAngleLiteral),
    NumberCalculation(CssNumberCalculation),
    AngleCalculation(CssAngleCalculation),
}

impl CssAuthoredHue {
    pub(crate) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredRgbColor {
    syntax: CssAuthoredColorSyntax,
    channels: [CssAuthoredColorComponent; 3],
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredRgbColor {
    pub(crate) const fn new(
        syntax: CssAuthoredColorSyntax,
        channels: [CssAuthoredColorComponent; 3],
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            syntax,
            channels,
            alpha,
        }
    }

    #[must_use]
    pub const fn syntax(&self) -> CssAuthoredColorSyntax {
        self.syntax
    }

    #[must_use]
    pub const fn channels(&self) -> &[CssAuthoredColorComponent; 3] {
        &self.channels
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredHslColor {
    syntax: CssAuthoredColorSyntax,
    hue: CssAuthoredHue,
    saturation: CssAuthoredColorComponent,
    lightness: CssAuthoredColorComponent,
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredHslColor {
    pub(crate) const fn new(
        syntax: CssAuthoredColorSyntax,
        hue: CssAuthoredHue,
        saturation: CssAuthoredColorComponent,
        lightness: CssAuthoredColorComponent,
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            syntax,
            hue,
            saturation,
            lightness,
            alpha,
        }
    }

    #[must_use]
    pub const fn syntax(&self) -> CssAuthoredColorSyntax {
        self.syntax
    }

    #[must_use]
    pub const fn hue(&self) -> &CssAuthoredHue {
        &self.hue
    }

    #[must_use]
    pub const fn saturation(&self) -> &CssAuthoredColorComponent {
        &self.saturation
    }

    #[must_use]
    pub const fn lightness(&self) -> &CssAuthoredColorComponent {
        &self.lightness
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredHwbColor {
    hue: CssAuthoredHue,
    whiteness: CssAuthoredColorComponent,
    blackness: CssAuthoredColorComponent,
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredHwbColor {
    pub(crate) const fn new(
        hue: CssAuthoredHue,
        whiteness: CssAuthoredColorComponent,
        blackness: CssAuthoredColorComponent,
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            hue,
            whiteness,
            blackness,
            alpha,
        }
    }

    #[must_use]
    pub const fn hue(&self) -> &CssAuthoredHue {
        &self.hue
    }

    #[must_use]
    pub const fn whiteness(&self) -> &CssAuthoredColorComponent {
        &self.whiteness
    }

    #[must_use]
    pub const fn blackness(&self) -> &CssAuthoredColorComponent {
        &self.blackness
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

/// A parser-owned authored Lab-family color with exact channel kinds.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredLabColor {
    lightness: CssAuthoredColorComponent,
    a: CssAuthoredColorComponent,
    b: CssAuthoredColorComponent,
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredLabColor {
    pub(crate) const fn new(
        lightness: CssAuthoredColorComponent,
        a: CssAuthoredColorComponent,
        b: CssAuthoredColorComponent,
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            lightness,
            a,
            b,
            alpha,
        }
    }

    #[must_use]
    pub const fn lightness(&self) -> &CssAuthoredColorComponent {
        &self.lightness
    }

    #[must_use]
    pub const fn a(&self) -> &CssAuthoredColorComponent {
        &self.a
    }

    #[must_use]
    pub const fn b(&self) -> &CssAuthoredColorComponent {
        &self.b
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

/// A parser-owned authored LCH-family color with an angle-capable hue.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredLchColor {
    lightness: CssAuthoredColorComponent,
    chroma: CssAuthoredColorComponent,
    hue: CssAuthoredHue,
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredLchColor {
    pub(crate) const fn new(
        lightness: CssAuthoredColorComponent,
        chroma: CssAuthoredColorComponent,
        hue: CssAuthoredHue,
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            lightness,
            chroma,
            hue,
            alpha,
        }
    }

    #[must_use]
    pub const fn lightness(&self) -> &CssAuthoredColorComponent {
        &self.lightness
    }

    #[must_use]
    pub const fn chroma(&self) -> &CssAuthoredColorComponent {
        &self.chroma
    }

    #[must_use]
    pub const fn hue(&self) -> &CssAuthoredHue {
        &self.hue
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

/// A parser-owned absolute `color()` value in a predefined Color 4 space.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredPredefinedColor {
    color_space: CssPredefinedColorSpace,
    channels: [CssAuthoredColorComponent; 3],
    alpha: Option<CssAuthoredColorComponent>,
}

/// The closed origin-channel environment for a preserved relative-color family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorEnvironment {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    PredefinedRgb(CssPredefinedColorSpace),
    Xyz(CssPredefinedColorSpace),
}

/// The semantic result slot in which a relative-color expression is authored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorResultDomain {
    NumberPercentage,
    Hue,
    Alpha,
}

/// A channel name made available by one relative-color origin environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorChannel {
    R,
    G,
    B,
    H,
    S,
    L,
    W,
    A,
    C,
    X,
    Y,
    Z,
    Alpha,
}

/// The authored kind of one validated relative-color result expression.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorExpressionValue {
    None,
    Number(CssFiniteNumber),
    Percentage(CssFiniteNumber),
    Angle(CssAngleLiteral),
    Channel(CssRelativeColorChannel),
    Calculation(CssRelativeColorCalculation),
}

/// A validated relative-color calculation retained symbolically without evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssRelativeColorCalculation {
    data: Box<CssRelativeColorCalculationData>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CssRelativeColorCalculationData {
    authored: CssAuthoredDeclarationValue,
    result_type: CssCalculationType,
    references: Vec<CssRelativeColorChannel>,
    expression: CssCalculationExpression,
}

impl CssRelativeColorCalculation {
    pub(crate) fn from_expression(
        authored: CssAuthoredDeclarationValue,
        expression: CssCalculationExpression,
    ) -> Self {
        Self {
            data: Box::new(CssRelativeColorCalculationData {
                authored,
                result_type: expression.result_type(),
                references: expression.references(),
                expression,
            }),
        }
    }

    #[must_use]
    pub const fn authored(&self) -> &CssAuthoredDeclarationValue {
        &self.data.authored
    }

    #[must_use]
    pub const fn result_type(&self) -> CssCalculationType {
        self.data.result_type
    }

    #[must_use]
    pub fn references(&self) -> &[CssRelativeColorChannel] {
        &self.data.references
    }

    /// Returns the shared exact numeric expression, including original leaf origins.
    pub fn expression(&self) -> CssCalculationExpressionRef<'_> {
        self.data.expression.as_ref()
    }
}

/// One result expression checked against a closed relative-color environment and slot domain.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTypedRelativeColorExpression {
    environment: CssRelativeColorEnvironment,
    result_domain: CssRelativeColorResultDomain,
    value: CssRelativeColorExpressionValue,
}

impl CssTypedRelativeColorExpression {
    pub(crate) const fn new(
        environment: CssRelativeColorEnvironment,
        result_domain: CssRelativeColorResultDomain,
        value: CssRelativeColorExpressionValue,
    ) -> Self {
        Self {
            environment,
            result_domain,
            value,
        }
    }

    #[must_use]
    pub const fn environment(&self) -> CssRelativeColorEnvironment {
        self.environment
    }

    #[must_use]
    pub const fn result_domain(&self) -> CssRelativeColorResultDomain {
        self.result_domain
    }

    #[must_use]
    pub const fn value(&self) -> &CssRelativeColorExpressionValue {
        &self.value
    }
}

/// A parser-owned relative color with an exact three-channel typed result environment.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredRelativeColor {
    function: CssRelativeColorFunction,
    environment: CssRelativeColorEnvironment,
    source: Box<CssAuthoredColor>,
    channels: [CssTypedRelativeColorExpression; 3],
    alpha: Option<CssTypedRelativeColorExpression>,
}

impl CssAuthoredRelativeColor {
    pub(crate) fn new(
        function: CssRelativeColorFunction,
        environment: CssRelativeColorEnvironment,
        source: CssAuthoredColor,
        channels: [CssTypedRelativeColorExpression; 3],
        alpha: Option<CssTypedRelativeColorExpression>,
    ) -> Self {
        Self {
            function,
            environment,
            source: Box::new(source),
            channels,
            alpha,
        }
    }

    #[must_use]
    pub const fn function(&self) -> &CssRelativeColorFunction {
        &self.function
    }

    #[must_use]
    pub const fn environment(&self) -> CssRelativeColorEnvironment {
        self.environment
    }

    #[must_use]
    pub const fn source(&self) -> &CssAuthoredColor {
        &self.source
    }

    #[must_use]
    pub const fn channels(&self) -> &[CssTypedRelativeColorExpression; 3] {
        &self.channels
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssTypedRelativeColorExpression> {
        self.alpha.as_ref()
    }
}

impl CssAuthoredPredefinedColor {
    pub(crate) const fn new(
        color_space: CssPredefinedColorSpace,
        channels: [CssAuthoredColorComponent; 3],
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            color_space,
            channels,
            alpha,
        }
    }

    #[must_use]
    pub const fn color_space(&self) -> CssPredefinedColorSpace {
        self.color_space
    }

    #[must_use]
    pub const fn channels(&self) -> &[CssAuthoredColorComponent; 3] {
        &self.channels
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

/// A checked authored percentage trailing one preserved `color-mix()` component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssAuthoredColorMixPercentage {
    value: CssFiniteNumber,
}

impl CssAuthoredColorMixPercentage {
    #[must_use]
    pub const fn try_new(value: f32) -> Option<Self> {
        if value >= 0.0 && value <= 100.0 {
            match CssFiniteNumber::try_new(value) {
                Some(value) => Some(Self { value }),
                None => None,
            }
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }
}

/// One checked authored color and optional trailing percentage in `color-mix()`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredColorMixComponent {
    color: Box<CssAuthoredColor>,
    percentage: Option<CssAuthoredColorMixPercentage>,
}

impl CssAuthoredColorMixComponent {
    #[must_use]
    pub fn new(color: CssAuthoredColor, percentage: Option<CssAuthoredColorMixPercentage>) -> Self {
        Self {
            color: Box::new(color),
            percentage,
        }
    }

    #[must_use]
    pub const fn color(&self) -> &CssAuthoredColor {
        &self.color
    }

    #[must_use]
    pub const fn percentage(&self) -> Option<CssAuthoredColorMixPercentage> {
        self.percentage
    }
}

/// The valid-by-construction preserved Color 5 `color-mix()` subset.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredColorMix {
    interpolation: CssColorInterpolationMethod,
    left: CssAuthoredColorMixComponent,
    right: CssAuthoredColorMixComponent,
}

impl CssAuthoredColorMix {
    #[must_use]
    pub fn try_new(
        interpolation: CssColorInterpolationMethod,
        left: CssAuthoredColorMixComponent,
        right: CssAuthoredColorMixComponent,
    ) -> Option<Self> {
        if interpolation.hue().is_some() && !interpolation.space().is_polar() {
            None
        } else {
            Some(Self {
                interpolation,
                left,
                right,
            })
        }
    }

    #[must_use]
    pub const fn interpolation(&self) -> &CssColorInterpolationMethod {
        &self.interpolation
    }

    #[must_use]
    pub const fn left(&self) -> &CssAuthoredColorMixComponent {
        &self.left
    }

    #[must_use]
    pub const fn right(&self) -> &CssAuthoredColorMixComponent {
        &self.right
    }

    fn has_exact_i01_projection(&self) -> bool {
        self.left.color().has_exact_i01_projection()
            && self.right.color().has_exact_i01_projection()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedColor {
    current: CssAuthoredColor,
    i01_subset: Option<CssColor>,
}

impl CssParsedColor {
    pub(crate) const fn new(current: CssAuthoredColor, i01_subset: Option<CssColor>) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn from_i01(value: CssColor) -> Self {
        Self::new(CssAuthoredColor::preserved_i01(value.clone()), Some(value))
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredColor, Option<CssColor>) {
        (self.current, self.i01_subset)
    }

    pub(crate) const fn current(&self) -> &CssAuthoredColor {
        &self.current
    }

    pub(crate) const fn i01_subset(&self) -> Option<&CssColor> {
        self.i01_subset.as_ref()
    }
}

fn parsed_color_options_equal(
    left: Option<&CssParsedColor>,
    right: Option<&CssParsedColor>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => match (left.i01_subset(), right.i01_subset()) {
            (Some(left), Some(right)) => left == right,
            (None, None) => left.current() == right.current(),
            (Some(_), None) | (None, Some(_)) => false,
        },
        (None, Some(_)) | (Some(_), None) => false,
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColor {
    CurrentColor,
    Rgba(CssRgbaColor),
    Hsl(CssHslColor),
    Hwb(CssHwbColor),
    Lab(CssLabColor),
    Lch(CssLchColor),
    Oklab(CssLabColor),
    Oklch(CssLchColor),
    ColorFunction(CssColorFunction),
    System(CssSystemColor),
    ColorMix(CssColorMix),
    Relative(CssRelativeColor),
}

impl CssColor {
    pub const TRANSPARENT: Self = Self::Rgba(CssRgbaColor {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0.0,
    });
    pub const BLACK: Self = Self::Rgba(CssRgbaColor {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 1.0,
    });
    pub const WHITE: Self = Self::Rgba(CssRgbaColor {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 1.0,
    });

    #[must_use]
    pub const fn as_rgba(&self) -> Option<&CssRgbaColor> {
        match self {
            Self::Rgba(color) => Some(color),
            Self::CurrentColor
            | Self::Hsl(_)
            | Self::Hwb(_)
            | Self::Lab(_)
            | Self::Lch(_)
            | Self::Oklab(_)
            | Self::Oklch(_)
            | Self::ColorFunction(_)
            | Self::System(_)
            | Self::ColorMix(_)
            | Self::Relative(_) => None,
        }
    }

    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::CurrentColor => "currentcolor",
            Self::Rgba(_) => "rgba",
            Self::Hsl(_) => "hsl",
            Self::Hwb(_) => "hwb",
            Self::Lab(_) => "lab",
            Self::Lch(_) => "lch",
            Self::Oklab(_) => "oklab",
            Self::Oklch(_) => "oklch",
            Self::ColorFunction(_) => "color",
            Self::System(_) => "system",
            Self::ColorMix(_) => "color-mix",
            Self::Relative(_) => "relative",
        }
    }

    #[must_use]
    pub fn try_rgba(r: f32, g: f32, b: f32, a: f32) -> Option<Self> {
        if [r, g, b, a]
            .into_iter()
            .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel))
        {
            Some(Self::rgba_unchecked(r, g, b, a))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn rgba_unchecked(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::Rgba(CssRgbaColor {
            red: normalized_color_channel_to_byte(r),
            green: normalized_color_channel_to_byte(g),
            blue: normalized_color_channel_to_byte(b),
            alpha: a,
        })
    }
}

fn normalized_color_channel_to_byte(channel: f32) -> u8 {
    (channel * 255.0).round() as u8
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssRgbaColor {
    red: u8,
    green: u8,
    blue: u8,
    alpha: f32,
}

impl CssRgbaColor {
    #[must_use]
    pub fn try_new(red: u8, green: u8, blue: u8, alpha: f32) -> Option<Self> {
        if alpha.is_finite() && (0.0..=1.0).contains(&alpha) {
            Some(Self {
                red,
                green,
                blue,
                alpha,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn red(&self) -> u8 {
        self.red
    }

    #[must_use]
    pub const fn green(&self) -> u8 {
        self.green
    }

    #[must_use]
    pub const fn blue(&self) -> u8 {
        self.blue
    }

    #[must_use]
    pub const fn alpha(&self) -> f32 {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssHslColor {
    hue: Option<f32>,
    saturation: Option<f32>,
    lightness: Option<f32>,
    alpha: Option<f32>,
}

impl CssHslColor {
    #[must_use]
    pub fn try_new(
        hue: Option<f32>,
        saturation: Option<f32>,
        lightness: Option<f32>,
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite([hue, saturation, lightness]) && color_alpha_is_valid(alpha)
        {
            Some(Self::new(hue, saturation, lightness, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        hue: Option<f32>,
        saturation: Option<f32>,
        lightness: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        Self {
            hue,
            saturation,
            lightness,
            alpha,
        }
    }

    #[must_use]
    pub const fn hue(&self) -> Option<f32> {
        self.hue
    }

    #[must_use]
    pub const fn saturation(&self) -> Option<f32> {
        self.saturation
    }

    #[must_use]
    pub const fn lightness(&self) -> Option<f32> {
        self.lightness
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssHwbColor {
    hue: Option<f32>,
    whiteness: Option<f32>,
    blackness: Option<f32>,
    alpha: Option<f32>,
}

impl CssHwbColor {
    #[must_use]
    pub fn try_new(
        hue: Option<f32>,
        whiteness: Option<f32>,
        blackness: Option<f32>,
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite([hue, whiteness, blackness]) && color_alpha_is_valid(alpha) {
            Some(Self::new(hue, whiteness, blackness, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        hue: Option<f32>,
        whiteness: Option<f32>,
        blackness: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        Self {
            hue,
            whiteness,
            blackness,
            alpha,
        }
    }

    #[must_use]
    pub const fn hue(&self) -> Option<f32> {
        self.hue
    }

    #[must_use]
    pub const fn whiteness(&self) -> Option<f32> {
        self.whiteness
    }

    #[must_use]
    pub const fn blackness(&self) -> Option<f32> {
        self.blackness
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssLabColor {
    lightness: Option<f32>,
    a: Option<f32>,
    b: Option<f32>,
    alpha: Option<f32>,
}

impl CssLabColor {
    #[must_use]
    pub fn try_new(
        lightness: Option<f32>,
        a: Option<f32>,
        b: Option<f32>,
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite([lightness, a, b]) && color_alpha_is_valid(alpha) {
            Some(Self::new(lightness, a, b, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        lightness: Option<f32>,
        a: Option<f32>,
        b: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        Self {
            lightness,
            a,
            b,
            alpha,
        }
    }

    #[must_use]
    pub const fn lightness(&self) -> Option<f32> {
        self.lightness
    }

    #[must_use]
    pub const fn a(&self) -> Option<f32> {
        self.a
    }

    #[must_use]
    pub const fn b(&self) -> Option<f32> {
        self.b
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssLchColor {
    lightness: Option<f32>,
    chroma: Option<f32>,
    hue: Option<f32>,
    alpha: Option<f32>,
}

impl CssLchColor {
    #[must_use]
    pub fn try_new(
        lightness: Option<f32>,
        chroma: Option<f32>,
        hue: Option<f32>,
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite([lightness, chroma, hue]) && color_alpha_is_valid(alpha) {
            Some(Self::new(lightness, chroma, hue, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        lightness: Option<f32>,
        chroma: Option<f32>,
        hue: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        Self {
            lightness,
            chroma,
            hue,
            alpha,
        }
    }

    #[must_use]
    pub const fn lightness(&self) -> Option<f32> {
        self.lightness
    }

    #[must_use]
    pub const fn chroma(&self) -> Option<f32> {
        self.chroma
    }

    #[must_use]
    pub const fn hue(&self) -> Option<f32> {
        self.hue
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPredefinedColorSpace {
    Srgb,
    SrgbLinear,
    DisplayP3,
    DisplayP3Linear,
    A98Rgb,
    ProphotoRgb,
    Rec2020,
    XyzD50,
    XyzD65,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssColorFunction {
    color_space: CssPredefinedColorSpace,
    components: [Option<f32>; 3],
    alpha: Option<f32>,
}

impl CssColorFunction {
    #[must_use]
    pub fn try_new(
        color_space: CssPredefinedColorSpace,
        components: [Option<f32>; 3],
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite(components) && color_alpha_is_valid(alpha) {
            Some(Self::new(color_space, components, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        color_space: CssPredefinedColorSpace,
        components: [Option<f32>; 3],
        alpha: Option<f32>,
    ) -> Self {
        Self {
            color_space,
            components,
            alpha,
        }
    }

    #[must_use]
    pub const fn color_space(&self) -> CssPredefinedColorSpace {
        self.color_space
    }

    #[must_use]
    pub const fn components(&self) -> &[Option<f32>; 3] {
        &self.components
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

fn color_components_are_finite(components: [Option<f32>; 3]) -> bool {
    components
        .into_iter()
        .all(|component| component.is_none_or(f32::is_finite))
}

fn color_alpha_is_valid(alpha: Option<f32>) -> bool {
    alpha.is_none_or(|alpha| alpha.is_finite() && (0.0..=1.0).contains(&alpha))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSystemColor {
    Canvas,
    CanvasText,
    LinkText,
    VisitedText,
    ActiveText,
    ButtonFace,
    ButtonText,
    ButtonBorder,
    Field,
    FieldText,
    Highlight,
    HighlightText,
    Mark,
    MarkText,
    GrayText,
    SelectedItem,
    SelectedItemText,
    AccentColor,
    AccentColorText,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssColorMix {
    interpolation: CssColorInterpolationMethod,
    left: CssColorMixComponent,
    right: CssColorMixComponent,
}

impl CssColorMix {
    #[must_use]
    pub const fn new(
        interpolation: CssColorInterpolationMethod,
        left: CssColorMixComponent,
        right: CssColorMixComponent,
    ) -> Self {
        Self {
            interpolation,
            left,
            right,
        }
    }

    #[must_use]
    pub const fn interpolation(&self) -> &CssColorInterpolationMethod {
        &self.interpolation
    }

    #[must_use]
    pub const fn left(&self) -> &CssColorMixComponent {
        &self.left
    }

    #[must_use]
    pub const fn right(&self) -> &CssColorMixComponent {
        &self.right
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssColorMixComponent {
    color: Box<CssColor>,
    percentage: Option<f32>,
}

impl CssColorMixComponent {
    #[must_use]
    pub fn try_new(color: CssColor, percentage: Option<f32>) -> Option<Self> {
        if color_percentage_is_valid(percentage) {
            Some(Self::new(color, percentage))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(color: CssColor, percentage: Option<f32>) -> Self {
        Self {
            color: Box::new(color),
            percentage,
        }
    }

    #[must_use]
    pub const fn color(&self) -> &CssColor {
        &self.color
    }

    #[must_use]
    pub const fn percentage(&self) -> Option<f32> {
        self.percentage
    }
}

fn color_percentage_is_valid(percentage: Option<f32>) -> bool {
    percentage
        .is_none_or(|percentage| percentage.is_finite() && (0.0..=100.0).contains(&percentage))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorInterpolationMethod {
    space: CssColorInterpolationSpace,
    hue: Option<CssHueInterpolationMethod>,
}

impl CssColorInterpolationMethod {
    #[must_use]
    pub const fn new(
        space: CssColorInterpolationSpace,
        hue: Option<CssHueInterpolationMethod>,
    ) -> Self {
        Self { space, hue }
    }

    #[must_use]
    pub const fn space(&self) -> CssColorInterpolationSpace {
        self.space
    }

    #[must_use]
    pub const fn hue(&self) -> Option<CssHueInterpolationMethod> {
        self.hue
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorInterpolationSpace {
    Predefined(CssPredefinedColorSpace),
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
}

impl CssColorInterpolationSpace {
    /// Reports whether the interpolation space has a polar hue component.
    #[must_use]
    pub const fn is_polar(self) -> bool {
        matches!(self, Self::Hsl | Self::Hwb | Self::Lch | Self::Oklch)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHueInterpolationMethod {
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeColor {
    function: CssRelativeColorFunction,
    source: Box<CssColor>,
    components: Vec<CssColorComponentExpression>,
    alpha: Option<CssColorComponentExpression>,
}

impl CssRelativeColor {
    #[must_use]
    pub fn try_new(
        function: CssRelativeColorFunction,
        source: CssColor,
        components: Vec<CssColorComponentExpression>,
        alpha: Option<CssColorComponentExpression>,
    ) -> Option<Self> {
        if components.len() == function.component_count() {
            Some(Self::new(function, source, components, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(
        function: CssRelativeColorFunction,
        source: CssColor,
        components: Vec<CssColorComponentExpression>,
        alpha: Option<CssColorComponentExpression>,
    ) -> Self {
        Self {
            function,
            source: Box::new(source),
            components,
            alpha,
        }
    }

    #[must_use]
    pub const fn function(&self) -> &CssRelativeColorFunction {
        &self.function
    }

    #[must_use]
    pub const fn source(&self) -> &CssColor {
        &self.source
    }

    #[must_use]
    pub fn components(&self) -> &[CssColorComponentExpression] {
        &self.components
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssColorComponentExpression> {
        self.alpha.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorFunction {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Color(CssPredefinedColorSpace),
}

impl CssRelativeColorFunction {
    #[must_use]
    pub const fn component_count(self) -> usize {
        match self {
            Self::Rgb
            | Self::Hsl
            | Self::Hwb
            | Self::Lab
            | Self::Lch
            | Self::Oklab
            | Self::Oklch
            | Self::Color(_) => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorComponentExpression {
    authored: CssAuthoredDeclarationValue,
    references: Vec<CssVariableReference>,
}

impl CssColorComponentExpression {
    #[must_use]
    pub fn new(
        authored: CssAuthoredDeclarationValue,
        references: Vec<CssVariableReference>,
    ) -> Self {
        Self {
            authored,
            references,
        }
    }

    #[must_use]
    pub const fn authored(&self) -> &CssAuthoredDeclarationValue {
        &self.authored
    }

    #[must_use]
    pub fn references(&self) -> &[CssVariableReference] {
        &self.references
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssSelector {
    Tag(String),
    Key(String),
    Class(String),
    PseudoClass(CssPseudoClass),
    Compound(CssCompoundSelector),
    Complex(CssComplexSelector),
}

#[allow(dead_code)] // Staged for native nesting flattening in the parser.
impl CssSelector {
    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        match self {
            Self::Tag(_) | Self::Key(_) | Self::Class(_) => false,
            Self::PseudoClass(pseudo_class) => pseudo_class.has_pseudo_elements(),
            Self::Compound(selector) => selector.has_pseudo_elements(),
            Self::Complex(selector) => selector.has_pseudo_elements(),
        }
    }

    #[must_use]
    pub(crate) fn combine_descendant(parent: Self, child: Self) -> Option<Self> {
        let (child_first, child_rest) = child.into_complex_parts();
        let combined =
            Self::combine_with_combinator(parent, CssSelectorCombinator::Descendant, child_first)?;
        let (first, mut rest) = combined.into_complex_parts();
        rest.extend(child_rest);
        CssComplexSelector::try_new(first, rest).map(Self::Complex)
    }

    #[must_use]
    pub(crate) fn combine_with_combinator(
        parent: Self,
        combinator: CssSelectorCombinator,
        child: CssCompoundSelector,
    ) -> Option<Self> {
        let (first, mut rest) = parent.into_complex_parts();
        rest.push(CssComplexSelectorPart::new(combinator, child));
        CssComplexSelector::try_new(first, rest).map(Self::Complex)
    }

    #[must_use]
    pub(crate) fn append_to_subject(parent: Self, suffix: CssCompoundSelector) -> Option<Self> {
        if suffix.type_selector().is_some() || !suffix.ids().is_empty() {
            return None;
        }

        match parent {
            Self::Complex(mut selector) => {
                selector.append_to_subject(suffix)?;
                Some(Self::Complex(selector))
            }
            selector => {
                let mut selector = selector.into_compound_selector();
                selector.append_suffix(suffix)?;
                Some(Self::Compound(selector))
            }
        }
    }

    fn into_complex_parts(self) -> (CssCompoundSelector, Vec<CssComplexSelectorPart>) {
        match self {
            Self::Complex(selector) => selector.into_parts(),
            selector => (selector.into_compound_selector(), Vec::new()),
        }
    }

    fn into_compound_selector(self) -> CssCompoundSelector {
        match self {
            Self::Tag(tag) => {
                CssCompoundSelector::new(Some(tag), None, Vec::new(), Vec::new(), Vec::new())
            }
            Self::Key(key) => {
                CssCompoundSelector::new(None, Some(key), Vec::new(), Vec::new(), Vec::new())
            }
            Self::Class(class) => {
                CssCompoundSelector::new(None, None, vec![class], Vec::new(), Vec::new())
            }
            Self::PseudoClass(pseudo_class) => {
                CssCompoundSelector::new(None, None, Vec::new(), Vec::new(), vec![pseudo_class])
            }
            Self::Compound(selector) => selector,
            Self::Complex(_) => {
                unreachable!("complex selectors are handled before compound conversion")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssComplexSelector {
    first: CssCompoundSelector,
    rest: Vec<CssComplexSelectorPart>,
}

impl CssComplexSelector {
    #[must_use]
    pub fn try_new(first: CssCompoundSelector, rest: Vec<CssComplexSelectorPart>) -> Option<Self> {
        if rest.is_empty() || complex_selector_has_non_terminal_pseudo_elements(&first, &rest) {
            None
        } else {
            Some(Self::new(first, rest))
        }
    }

    #[must_use]
    fn new(first: CssCompoundSelector, rest: Vec<CssComplexSelectorPart>) -> Self {
        debug_assert!(!rest.is_empty());
        debug_assert!(!complex_selector_has_non_terminal_pseudo_elements(
            &first, &rest
        ));
        Self { first, rest }
    }

    #[must_use]
    pub const fn first(&self) -> &CssCompoundSelector {
        &self.first
    }

    #[must_use]
    pub fn rest(&self) -> &[CssComplexSelectorPart] {
        &self.rest
    }

    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.first.has_pseudo_elements()
            || self
                .rest
                .iter()
                .any(|part| part.selector().has_pseudo_elements())
    }

    #[allow(dead_code)] // Used by staged selector composition helpers.
    fn into_parts(self) -> (CssCompoundSelector, Vec<CssComplexSelectorPart>) {
        (self.first, self.rest)
    }

    #[allow(dead_code)] // Used by staged selector composition helpers.
    fn append_to_subject(&mut self, suffix: CssCompoundSelector) -> Option<()> {
        let subject = self.rest.last_mut()?;
        subject.selector.append_suffix(suffix)
    }
}

fn complex_selector_has_non_terminal_pseudo_elements(
    first: &CssCompoundSelector,
    rest: &[CssComplexSelectorPart],
) -> bool {
    first.has_pseudo_elements()
        || rest
            .iter()
            .take(rest.len().saturating_sub(1))
            .any(|part| part.selector().has_pseudo_elements())
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssComplexSelectorPart {
    combinator: CssSelectorCombinator,
    selector: CssCompoundSelector,
}

impl CssComplexSelectorPart {
    #[must_use]
    pub(crate) const fn new(
        combinator: CssSelectorCombinator,
        selector: CssCompoundSelector,
    ) -> Self {
        Self {
            combinator,
            selector,
        }
    }

    #[must_use]
    pub const fn combinator(&self) -> CssSelectorCombinator {
        self.combinator
    }

    #[must_use]
    pub const fn selector(&self) -> &CssCompoundSelector {
        &self.selector
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSelectorCombinator {
    Descendant,
    Child,
    NextSibling,
    SubsequentSibling,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssSelectorList {
    selectors: Vec<CssSelector>,
}

impl CssSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssSelector] {
        &self.selectors
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssPseudoSelectorList {
    selectors: Vec<CssSelector>,
}

impl CssPseudoSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    #[must_use]
    pub(crate) const fn new_forgiving(selectors: Vec<CssSelector>) -> Self {
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssSelector] {
        &self.selectors
    }

    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.selectors.iter().any(CssSelector::has_pseudo_elements)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeSelector {
    combinator: CssSelectorCombinator,
    selector: CssSelector,
}

impl CssRelativeSelector {
    #[must_use]
    pub const fn new(combinator: CssSelectorCombinator, selector: CssSelector) -> Self {
        Self {
            combinator,
            selector,
        }
    }

    #[must_use]
    pub const fn combinator(&self) -> CssSelectorCombinator {
        self.combinator
    }

    #[must_use]
    pub const fn selector(&self) -> &CssSelector {
        &self.selector
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeSelectorList {
    selectors: Vec<CssRelativeSelector>,
}

impl CssRelativeSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssRelativeSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssRelativeSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssRelativeSelector] {
        &self.selectors
    }

    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.selectors
            .iter()
            .any(|selector| selector.selector().has_pseudo_elements())
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPseudoClass {
    Root,
    Scope,
    Link,
    Visited,
    Target,
    Lang(CssLanguageRange),
    Hover,
    Active,
    Focus,
    FocusVisible,
    FocusWithin,
    Disabled,
    Enabled,
    Checked,
    Required,
    Optional,
    Valid,
    Invalid,
    PlaceholderShown,
    FirstChild,
    LastChild,
    OnlyChild,
    Empty,
    NthChild(CssNthChildPattern),
    NthLastChild(CssNthChildPattern),
    FirstOfType,
    LastOfType,
    OnlyOfType,
    NthOfType(CssNthPattern),
    NthLastOfType(CssNthPattern),
    Not(CssPseudoSelectorList),
    Is(CssPseudoSelectorList),
    Where(CssPseudoSelectorList),
    Has(CssRelativeSelectorList),
    Modal,
    Fullscreen,
    PopoverOpen,
    Default,
    Indeterminate,
    ReadOnly,
    ReadWrite,
    InRange,
    OutOfRange,
}

impl CssPseudoClass {
    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        match self {
            Self::NthChild(pattern) | Self::NthLastChild(pattern) => pattern.has_pseudo_elements(),
            Self::Not(selectors) | Self::Is(selectors) | Self::Where(selectors) => {
                selectors.has_pseudo_elements()
            }
            Self::Has(selectors) => selectors.has_pseudo_elements(),
            Self::Root
            | Self::Scope
            | Self::Link
            | Self::Visited
            | Self::Target
            | Self::Lang(_)
            | Self::Hover
            | Self::Active
            | Self::Focus
            | Self::FocusVisible
            | Self::FocusWithin
            | Self::Disabled
            | Self::Enabled
            | Self::Checked
            | Self::Required
            | Self::Optional
            | Self::Valid
            | Self::Invalid
            | Self::PlaceholderShown
            | Self::FirstChild
            | Self::LastChild
            | Self::OnlyChild
            | Self::Empty
            | Self::FirstOfType
            | Self::LastOfType
            | Self::OnlyOfType
            | Self::NthOfType(_)
            | Self::NthLastOfType(_)
            | Self::Modal
            | Self::Fullscreen
            | Self::PopoverOpen
            | Self::Default
            | Self::Indeterminate
            | Self::ReadOnly
            | Self::ReadWrite
            | Self::InRange
            | Self::OutOfRange => false,
        }
    }
}

/// One checked, decoded identifier authored as a Selectors 3 `:lang()` range.
///
/// This value preserves selector syntax only. It does not compare languages or
/// resolve inherited language metadata.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssLanguageRange {
    value: String,
}

impl CssLanguageRange {
    /// Constructs a language range from exactly one decoded CSS identifier.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        is_exact_css_identifier(&value).then_some(Self { value })
    }

    /// Returns the exact decoded identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPseudoElement {
    Before,
    After,
    FirstLine,
    FirstLetter,
    Marker,
    Selection,
    Backdrop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPseudoElementSequence {
    pseudo_elements: Vec<CssPseudoElement>,
}

impl CssPseudoElementSequence {
    #[must_use]
    pub fn try_new(pseudo_elements: Vec<CssPseudoElement>) -> Option<Self> {
        if Self::is_supported_sequence(&pseudo_elements) {
            Some(Self::new(pseudo_elements))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(pseudo_elements: Vec<CssPseudoElement>) -> Self {
        debug_assert!(Self::is_supported_sequence(&pseudo_elements));
        Self { pseudo_elements }
    }

    #[must_use]
    pub fn pseudo_elements(&self) -> &[CssPseudoElement] {
        &self.pseudo_elements
    }

    fn is_supported_sequence(pseudo_elements: &[CssPseudoElement]) -> bool {
        matches!(
            pseudo_elements,
            [CssPseudoElement::Before]
                | [CssPseudoElement::After]
                | [CssPseudoElement::FirstLine]
                | [CssPseudoElement::FirstLetter]
                | [CssPseudoElement::Marker]
                | [CssPseudoElement::Selection]
                | [CssPseudoElement::Backdrop]
                | [CssPseudoElement::Before, CssPseudoElement::Marker]
                | [CssPseudoElement::After, CssPseudoElement::Marker]
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssNthChildPattern {
    pattern: CssNthPattern,
    selector_list: Option<CssPseudoSelectorList>,
}

impl CssNthChildPattern {
    #[must_use]
    pub const fn new(pattern: CssNthPattern, selector_list: Option<CssPseudoSelectorList>) -> Self {
        Self {
            pattern,
            selector_list,
        }
    }

    #[must_use]
    pub const fn pattern(&self) -> CssNthPattern {
        self.pattern
    }

    #[must_use]
    pub const fn selector_list(&self) -> Option<&CssPseudoSelectorList> {
        self.selector_list.as_ref()
    }

    #[must_use]
    pub fn has_pseudo_elements(&self) -> bool {
        self.selector_list
            .as_ref()
            .is_some_and(CssPseudoSelectorList::has_pseudo_elements)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNthPattern {
    Odd,
    Even,
    Integer(i32),
    AnPlusB(CssNthAnPlusB),
}

/// The namespace selected by one authored selector name.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssNamespaceConstraint {
    /// The active default namespace declared for the stylesheet.
    Default,
    /// No namespace, authored with a leading `|` or implied for an attribute.
    ExplicitNone,
    /// Any namespace, authored with `*|` or implied when no default is active.
    Any,
    /// The active binding for one exact, case-sensitive decoded prefix.
    Named(CssNamespacePrefix),
}

/// One parser-produced namespace-qualified type or universal selector name.
///
/// The private representation admits either one decoded CSS identifier or the
/// universal selector, never an empty or otherwise invalid local name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssQualifiedSelectorName {
    namespace: CssNamespaceConstraint,
    local_name: Option<String>,
}

impl CssQualifiedSelectorName {
    #[must_use]
    pub(crate) fn new(namespace: CssNamespaceConstraint, local_name: String) -> Self {
        debug_assert!(!local_name.is_empty());
        Self {
            namespace,
            local_name: Some(local_name),
        }
    }

    #[must_use]
    pub(crate) const fn universal(namespace: CssNamespaceConstraint) -> Self {
        Self {
            namespace,
            local_name: None,
        }
    }

    /// Returns the namespace constraint active for this selector name.
    #[must_use]
    pub const fn namespace(&self) -> &CssNamespaceConstraint {
        &self.namespace
    }

    /// Returns the decoded local identifier, or `None` for universal `*`.
    #[must_use]
    pub fn local_name(&self) -> Option<&str> {
        self.local_name.as_deref()
    }

    /// Reports whether this name is the universal selector.
    #[must_use]
    pub const fn is_universal(&self) -> bool {
        self.local_name.is_none()
    }

    #[must_use]
    const fn local_name_string(&self) -> Option<&String> {
        self.local_name.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssNthAnPlusB {
    a: i32,
    b: i32,
}

impl CssNthAnPlusB {
    #[must_use]
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    #[must_use]
    pub const fn a(self) -> i32 {
        self.a
    }

    #[must_use]
    pub const fn b(self) -> i32 {
        self.b
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssCompoundSelector {
    scope_anchor: bool,
    nesting_selectors: usize,
    type_selector: Option<Box<(CssQualifiedSelectorName, bool)>>,
    tag: Option<String>,
    ids: Vec<String>,
    classes: Vec<String>,
    attributes: Vec<CssAttributeSelector>,
    pseudo_classes: Vec<CssPseudoClass>,
    pseudo_elements: Option<CssPseudoElementSequence>,
}

impl CssCompoundSelector {
    #[must_use]
    pub(crate) fn new(
        tag: Option<String>,
        key: Option<String>,
        classes: Vec<String>,
        attributes: Vec<CssAttributeSelector>,
        pseudo_classes: Vec<CssPseudoClass>,
    ) -> Self {
        Self::new_with_scope_anchor(false, tag, key, classes, attributes, pseudo_classes)
    }

    #[must_use]
    pub(crate) fn new_with_scope_anchor(
        scope_anchor: bool,
        tag: Option<String>,
        key: Option<String>,
        classes: Vec<String>,
        attributes: Vec<CssAttributeSelector>,
        pseudo_classes: Vec<CssPseudoClass>,
    ) -> Self {
        Self::new_with_scope_anchor_and_pseudo_elements(
            scope_anchor,
            tag,
            key,
            classes,
            attributes,
            pseudo_classes,
            None,
        )
    }

    #[must_use]
    pub(crate) fn new_with_scope_anchor_and_pseudo_elements(
        scope_anchor: bool,
        tag: Option<String>,
        key: Option<String>,
        classes: Vec<String>,
        attributes: Vec<CssAttributeSelector>,
        pseudo_classes: Vec<CssPseudoClass>,
        pseudo_elements: Option<CssPseudoElementSequence>,
    ) -> Self {
        let type_selector = tag.map(|tag| {
            (
                CssQualifiedSelectorName::new(CssNamespaceConstraint::Any, tag),
                true,
            )
        });
        let ids = key.into_iter().collect();
        Self::new_with_qualified_type_and_pseudo_elements(
            scope_anchor,
            type_selector,
            ids,
            classes,
            attributes,
            pseudo_classes,
            pseudo_elements,
        )
    }

    #[must_use]
    pub(crate) fn new_with_qualified_type_and_pseudo_elements(
        scope_anchor: bool,
        type_selector: Option<(CssQualifiedSelectorName, bool)>,
        ids: Vec<String>,
        classes: Vec<String>,
        attributes: Vec<CssAttributeSelector>,
        pseudo_classes: Vec<CssPseudoClass>,
        pseudo_elements: Option<CssPseudoElementSequence>,
    ) -> Self {
        let tag = type_selector
            .as_ref()
            .and_then(|(type_selector, _)| type_selector.local_name_string())
            .cloned();
        Self {
            scope_anchor,
            nesting_selectors: 0,
            type_selector: type_selector.map(Box::new),
            tag,
            ids,
            classes,
            attributes,
            pseudo_classes,
            pseudo_elements,
        }
    }

    pub(crate) fn with_nesting_selectors(mut self, count: usize) -> Self {
        self.nesting_selectors = count;
        self
    }

    /// Returns the number of symbolic nesting selectors in this compound.
    ///
    /// With an enclosing parent selector list, every anchor refers to that complete
    /// list and uses its maximum specificity. Without a parent list, each anchor
    /// matches the context's scope elements and contributes zero specificity.
    /// This count preserves authored anchors; it does not resolve their context or
    /// replace them with a `:scope` pseudo-class.
    #[must_use]
    pub const fn nesting_selectors(&self) -> usize {
        self.nesting_selectors
    }

    #[must_use]
    pub const fn has_scope_anchor(&self) -> bool {
        self.scope_anchor
    }

    /// Returns the current namespace-aware type or universal selector.
    #[must_use]
    pub fn type_selector(&self) -> Option<&CssQualifiedSelectorName> {
        match self.type_selector.as_deref() {
            Some((type_selector, _)) => Some(type_selector),
            None => None,
        }
    }

    #[must_use]
    pub(crate) fn has_legacy_type_projection(&self) -> bool {
        matches!(self.type_selector.as_deref(), Some((_, true)))
    }

    /// Returns the local type-name compatibility projection.
    ///
    /// Universal selectors have no tag projection.
    #[must_use]
    pub const fn tag(&self) -> Option<&String> {
        self.tag.as_ref()
    }

    /// Returns the last authored ID as the I01 compatibility projection.
    #[must_use]
    pub fn key(&self) -> Option<&String> {
        self.ids.last()
    }

    /// Returns the parser-retained IDs in authored order.
    #[must_use]
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    #[must_use]
    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    #[must_use]
    pub fn attributes(&self) -> &[CssAttributeSelector] {
        &self.attributes
    }

    #[must_use]
    pub fn pseudo_classes(&self) -> &[CssPseudoClass] {
        &self.pseudo_classes
    }

    #[must_use]
    pub const fn pseudo_elements(&self) -> Option<&CssPseudoElementSequence> {
        self.pseudo_elements.as_ref()
    }

    #[must_use]
    pub const fn has_pseudo_elements(&self) -> bool {
        self.pseudo_elements.is_some()
    }

    #[allow(dead_code)] // Used by staged selector composition helpers.
    fn append_suffix(&mut self, suffix: Self) -> Option<()> {
        debug_assert!(suffix.type_selector.is_none());
        debug_assert!(suffix.ids.is_empty());
        debug_assert!(!suffix.scope_anchor);
        if self.pseudo_elements.is_some() {
            return None;
        }
        self.classes.extend(suffix.classes);
        self.attributes.extend(suffix.attributes);
        self.pseudo_classes.extend(suffix.pseudo_classes);
        self.pseudo_elements = suffix.pseudo_elements;
        Some(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAttributeSelector {
    namespace: CssNamespaceConstraint,
    name: CssAttributeName,
    matcher: CssAttributeMatcher,
    case_sensitivity: CssAttributeCaseSensitivity,
}

impl CssAttributeSelector {
    #[must_use]
    pub(crate) const fn new_qualified(
        namespace: CssNamespaceConstraint,
        name: CssAttributeName,
        matcher: CssAttributeMatcher,
        case_sensitivity: CssAttributeCaseSensitivity,
    ) -> Self {
        Self {
            namespace,
            name,
            matcher,
            case_sensitivity,
        }
    }

    /// Returns the namespace constraint for the attribute name.
    #[must_use]
    pub const fn namespace(&self) -> &CssNamespaceConstraint {
        &self.namespace
    }

    #[must_use]
    pub const fn name(&self) -> &CssAttributeName {
        &self.name
    }

    #[must_use]
    pub const fn matcher(&self) -> &CssAttributeMatcher {
        &self.matcher
    }

    #[must_use]
    pub const fn case_sensitivity(&self) -> CssAttributeCaseSensitivity {
        self.case_sensitivity
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssAttributeName {
    name: String,
}

impl CssAttributeName {
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        let is_valid = {
            let mut input = cssparser::ParserInput::new(&name);
            let mut parser = cssparser::Parser::new(&mut input);
            let parsed = parser.expect_ident_cloned().ok()?;
            parser.expect_exhausted().ok()?;
            parsed.as_ref() == name
        };
        if is_valid { Some(Self { name }) } else { None }
    }

    #[must_use]
    pub(crate) fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(Self::try_new(name.clone()).is_some());
        Self { name }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAttributeMatcher {
    Exists,
    Equals(String),
    Includes(String),
    DashMatch(String),
    Prefix(String),
    Suffix(String),
    Substring(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAttributeCaseSensitivity {
    DocumentDefault,
    AsciiCaseInsensitive,
    ExplicitSensitive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAngleUnit {
    Degrees,
    Gradians,
    Radians,
    Turns,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssAngleLiteral {
    value: CssFiniteNumber,
    unit: CssAngleUnit,
}

impl CssAngleLiteral {
    #[must_use]
    pub fn try_new(value: f32, unit: CssAngleUnit) -> Option<Self> {
        CssFiniteNumber::try_new(value).map(|value| Self { value, unit })
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }

    #[must_use]
    pub const fn unit(self) -> CssAngleUnit {
        self.unit
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFrequencyUnit {
    Hertz,
    Kilohertz,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFrequencyLiteral {
    value: CssFiniteNumber,
    unit: CssFrequencyUnit,
}

impl CssFrequencyLiteral {
    #[must_use]
    pub fn try_new(value: f32, unit: CssFrequencyUnit) -> Option<Self> {
        CssFiniteNumber::try_new(value).map(|value| Self { value, unit })
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }

    #[must_use]
    pub const fn unit(self) -> CssFrequencyUnit {
        self.unit
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssDelayLiteral {
    value: CssFiniteNumber,
    unit: CssTimeUnit,
}

impl CssDelayLiteral {
    #[must_use]
    pub fn try_new(value: f32, unit: CssTimeUnit) -> Option<Self> {
        CssFiniteNumber::try_new(value).map(|value| Self { value, unit })
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value.value()
    }

    #[must_use]
    pub const fn unit(self) -> CssTimeUnit {
        self.unit
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCalcLength {
    Px(CssFiniteNumber),
    Dimension(CssLengthDimension),
    Percent(CssFiniteNumber),
    Sum(Vec<CssCalcLengthTerm>),
    Typed(CssLengthPercentageCalculation),
}

impl CssCalcLength {
    #[must_use]
    pub fn try_px(value: f32) -> Option<Self> {
        CssFiniteNumber::try_new(value).map(Self::Px)
    }

    #[must_use]
    pub fn try_percent(value: f32) -> Option<Self> {
        CssFiniteNumber::try_new(value).map(Self::Percent)
    }

    #[must_use]
    pub fn try_dimension(value: f32, unit: CssLengthUnit) -> Option<Self> {
        match unit {
            CssLengthUnit::Px => Self::try_px(value),
            _ => CssLengthDimension::try_new(value, unit).map(Self::Dimension),
        }
    }

    #[must_use]
    pub(crate) const fn px(value: f32) -> Self {
        Self::Px(CssFiniteNumber::new_unchecked(value))
    }

    #[must_use]
    pub(crate) const fn percent(value: f32) -> Self {
        Self::Percent(CssFiniteNumber::new_unchecked(value))
    }

    #[must_use]
    pub(crate) const fn dimension(value: f32, unit: CssLengthUnit) -> Self {
        match unit {
            CssLengthUnit::Px => Self::px(value),
            _ => Self::Dimension(CssLengthDimension::new(value, unit)),
        }
    }

    #[must_use]
    pub fn sum(
        first: CssCalcLengthTerm,
        rest: impl IntoIterator<Item = CssCalcLengthTerm>,
    ) -> Self {
        let mut terms = vec![first];
        terms.extend(rest);
        Self::Sum(terms)
    }

    #[must_use]
    pub fn uses_percentage(&self) -> bool {
        match self {
            Self::Px(_) => false,
            Self::Dimension(_) => false,
            Self::Percent(_) => true,
            Self::Sum(terms) => terms.iter().any(|term| term.value.uses_percentage()),
            Self::Typed(calculation) => {
                calculation.numeric_type().percent_hint().is_some()
                    || calculation
                        .numeric_type()
                        .exponent(CssNumericDimension::Percentage)
                        != 0
            }
        }
    }

    #[must_use]
    pub fn to_css_string(&self) -> String {
        self.to_css_fragment()
    }

    fn to_css_fragment(&self) -> String {
        match self {
            Self::Px(value) => format!("{}px", format_css_number(value.value())),
            Self::Dimension(length) => length.to_css_string(),
            Self::Percent(value) => format!("{}%", format_css_number(value.value())),
            Self::Sum(terms) => {
                let mut css = String::from("calc(");
                for (index, term) in terms.iter().enumerate() {
                    if index == 0 {
                        css.push_str(&term.value.to_css_fragment());
                    } else {
                        css.push(' ');
                        css.push_str(match term.operator {
                            CssCalcOperator::Add => "+",
                            CssCalcOperator::Subtract => "-",
                        });
                        css.push(' ');
                        css.push_str(&term.value.to_css_fragment());
                    }
                }
                css.push(')');
                css
            }
            Self::Typed(calculation) => calculation.expression.to_css_fragment(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssCalcLengthTerm {
    operator: CssCalcOperator,
    value: CssCalcLength,
}

impl CssCalcLengthTerm {
    #[must_use]
    pub const fn add(value: CssCalcLength) -> Self {
        Self {
            operator: CssCalcOperator::Add,
            value,
        }
    }

    #[must_use]
    pub const fn sub(value: CssCalcLength) -> Self {
        Self {
            operator: CssCalcOperator::Subtract,
            value,
        }
    }

    #[must_use]
    pub const fn operator(&self) -> CssCalcOperator {
        self.operator
    }

    #[must_use]
    pub const fn value(&self) -> &CssCalcLength {
        &self.value
    }
    pub(crate) fn value_mut(&mut self) -> &mut CssCalcLength {
        &mut self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCalcOperator {
    Add,
    Subtract,
}

fn format_css_number(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
