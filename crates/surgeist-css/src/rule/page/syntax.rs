//! Authored Page selectors and distinct Page/margin descriptor domains.
use crate::component::CssComponentValues;
use crate::declaration::{CssDeclaration, CssDeclarationList, CssGlobalKeyword, CssImportance};
use crate::numeric::{CssLength, CssNonNegative, CssNumeric};
use crate::rule::CssDescriptorOccurrence;
use crate::source::{CssParsedOrigin, CssSourcePosition};
use crate::values::common::CssAutoOr;

/// A page-selector pseudo-class of an `@page` prelude.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssPagePseudo {
    /// `:left`: matches a page that would be on the left of a pair of facing pages.
    Left,
    /// `:right`: matches a page that would be on the right of a pair of facing pages.
    Right,
    /// `:first`: matches the first printed page of the document.
    First,
    /// `:blank`: matches content-empty pages that appear as a result of forced page breaks.
    Blank,
    /// `:recto`: the flow-relative equivalent of `:right` in left-to-right page progressions and of `:left` in right-to-left ones.
    Recto,
    /// `:verso`: the flow-relative equivalent of `:left` in left-to-right page progressions and of `:right` in right-to-left ones.
    Verso,
}
impl CssPagePseudo {
    /// Returns the specificity the pseudo-class contributes: one `first_or_blank` for `:first` and `:blank`, one `side` for every other pseudo-class.
    #[must_use]
    pub const fn specificity(self) -> CssPageSpecificity {
        match self {
            Self::First | Self::Blank => CssPageSpecificity::new(0, 1, 0),
            _ => CssPageSpecificity::new(0, 0, 1),
        }
    }
    /// Returns the CSS spelling of the pseudo-class without the colon, such as `first`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::First => "first",
            Self::Blank => "blank",
            Self::Recto => "recto",
            Self::Verso => "verso",
        }
    }
}
/// Page specificity (named, first/blank, side); distinct from element specificity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CssPageSpecificity {
    named: usize,
    first_or_blank: usize,
    side: usize,
}
impl CssPageSpecificity {
    /// Creates a specificity from its three counts.
    #[must_use]
    pub const fn new(named: usize, first_or_blank: usize, side: usize) -> Self {
        Self {
            named,
            first_or_blank,
            side,
        }
    }
    /// Returns the count contributed by a page name (zero or one).
    #[must_use]
    pub const fn named(self) -> usize {
        self.named
    }
    /// Returns the count contributed by `:first` and `:blank` pseudo-classes.
    #[must_use]
    pub const fn first_or_blank(self) -> usize {
        self.first_or_blank
    }
    /// Returns the count contributed by the side pseudo-classes `:left`, `:right`, `:recto` and `:verso`.
    #[must_use]
    pub const fn side(self) -> usize {
        self.side
    }
}
/// One nonempty named/pseudo Page selector, retaining compound order and provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageSelector {
    name: Option<crate::values::common::CssIdent>,
    pseudos: Vec<CssPagePseudo>,
    origin: Option<CssParsedOrigin>,
}
impl CssPageSelector {
    /// Creates a selector from an optional page name and its pseudo-classes in order.
    ///
    /// Returns `None` when there is neither a name nor a pseudo-class.
    #[must_use]
    pub fn try_new(
        name: Option<crate::values::common::CssIdent>,
        pseudos: Vec<CssPagePseudo>,
    ) -> Option<Self> {
        (name.is_some() || !pseudos.is_empty()).then_some(Self {
            name,
            pseudos,
            origin: None,
        })
    }
    pub(crate) fn from_parsed(
        name: Option<crate::values::common::CssIdent>,
        pseudos: Vec<CssPagePseudo>,
        origin: CssParsedOrigin,
    ) -> Self {
        Self {
            name,
            pseudos,
            origin: Some(origin),
        }
    }
    /// Returns the page name, or `None` when the selector has only pseudo-classes.
    #[must_use]
    pub const fn name(&self) -> Option<&crate::values::common::CssIdent> {
        self.name.as_ref()
    }
    /// Returns the pseudo-classes in authored order.
    #[must_use]
    pub fn pseudos(&self) -> &[CssPagePseudo] {
        &self.pseudos
    }
    /// Returns the original raw-source window of the selector, or `None` when it was constructed programmatically.
    #[must_use]
    pub const fn origin(&self) -> Option<&CssParsedOrigin> {
        self.origin.as_ref()
    }
    /// Returns the page specificity: one `named` for a name, and the counts of `:first`/`:blank` and of side pseudo-classes.
    #[must_use]
    pub fn specificity(&self) -> CssPageSpecificity {
        CssPageSpecificity {
            named: usize::from(self.name.is_some()),
            first_or_blank: self
                .pseudos
                .iter()
                .filter(|p| matches!(p, CssPagePseudo::First | CssPagePseudo::Blank))
                .count(),
            side: self
                .pseudos
                .iter()
                .filter(|p| !matches!(p, CssPagePseudo::First | CssPagePseudo::Blank))
                .count(),
        }
    }
}
/// The Page prelude; empty is distinct from a nonempty selector list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CssPageSelectorList {
    selectors: Vec<CssPageSelector>,
    origin: Option<CssParsedOrigin>,
}
impl CssPageSelectorList {
    /// Creates a selector list from the given selectors, which may be empty, with no parsed origin.
    #[must_use]
    pub fn new(selectors: Vec<CssPageSelector>) -> Self {
        Self {
            selectors,
            origin: None,
        }
    }
    pub(crate) fn from_parsed(selectors: Vec<CssPageSelector>, origin: CssParsedOrigin) -> Self {
        Self {
            selectors,
            origin: Some(origin),
        }
    }
    /// Returns the selectors in authored order; the slice is empty for an empty prelude.
    #[must_use]
    pub fn selectors(&self) -> &[CssPageSelector] {
        &self.selectors
    }
    /// Returns whether the prelude has no selectors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.selectors.is_empty()
    }
    /// Returns the original raw-source window of the complete prelude, or `None` when it was constructed programmatically.
    #[must_use]
    pub const fn origin(&self) -> Option<&CssParsedOrigin> {
        self.origin.as_ref()
    }
}
/// The descriptors that `@page` admits besides ordinary properties.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssPageDescriptorKind {
    /// The `size` descriptor, which gives the page box size and orientation.
    Size,
    /// The `page-orientation` descriptor, which rotates the page after layout.
    PageOrientation,
    /// The `marks` descriptor, which adds crop and registration marks outside the page box.
    Marks,
    /// The `bleed` descriptor, which gives the extent of the bleed area outside the page box.
    Bleed,
}
impl CssPageDescriptorKind {
    /// Returns the CSS descriptor name, such as `page-orientation`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::PageOrientation => "page-orientation",
            Self::Marks => "marks",
            Self::Bleed => "bleed",
        }
    }
    /// Looks up an exact decoded Page descriptor name, ignoring ASCII case.
    /// Unknown names, whitespace, escapes and punctuation return `None`.
    /// Ordinary Page properties and custom names remain separate domains.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        [Self::Size, Self::PageOrientation, Self::Marks, Self::Bleed]
            .into_iter()
            .find(|kind| name.eq_ignore_ascii_case(kind.as_str()))
    }
}
/// A page orientation keyword of the `size` descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPageOrientation {
    /// `portrait`: the shorter sides of the page box are horizontal.
    Portrait,
    /// `landscape`: the longer sides of the page box are horizontal.
    Landscape,
}
/// The value of the `page-orientation` descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPageOutputOrientation {
    /// `upright`: no special orientation is applied; the page is laid out and shown as usual.
    Upright,
    /// `rotate-left`: the laid-out page is displayed rotated a quarter turn counter-clockwise, if the output medium supports rotation.
    RotateLeft,
    /// `rotate-right`: the laid-out page is displayed rotated a quarter turn clockwise, if the output medium supports rotation.
    RotateRight,
}
/// A `<page-size>` media name of the `size` descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPageSize {
    /// `A5`: ISO A5 media, 148mm by 210mm.
    A5,
    /// `A4`: ISO A4 media, 210mm by 297mm.
    A4,
    /// `A3`: ISO A3 media, 297mm by 420mm.
    A3,
    /// `B5`: ISO B5 media, 176mm by 250mm.
    B5,
    /// `B4`: ISO B4 media, 250mm by 353mm.
    B4,
    /// `JIS-B5`: JIS B5 media, 182mm by 257mm.
    JisB5,
    /// `JIS-B4`: JIS B4 media, 257mm by 364mm.
    JisB4,
    /// `letter`: North American letter media, 8.5 by 11 inches.
    Letter,
    /// `legal`: North American legal media, 8.5 by 14 inches.
    Legal,
    /// `ledger`: North American ledger media, 11 by 17 inches.
    Ledger,
}
impl CssPageSize {
    pub(crate) const fn css_name(self) -> &'static str {
        match self {
            Self::A5 => "a5",
            Self::A4 => "a4",
            Self::A3 => "a3",
            Self::B5 => "b5",
            Self::B4 => "b4",
            Self::JisB5 => "jis-b5",
            Self::JisB4 => "jis-b4",
            Self::Letter => "letter",
            Self::Legal => "legal",
            Self::Ledger => "ledger",
        }
    }
}
/// The authored value of the `size` descriptor.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPageSizeValue {
    /// `auto`: the user agent chooses the page box size and orientation.
    Auto,
    /// One or two non-negative lengths: with one, the page box is a square of that size; with two, the first is the width and the second the height.
    Dimensions(
        CssNumeric<CssLength, CssNonNegative>,
        Option<CssNumeric<CssLength, CssNonNegative>>,
    ),
    /// A `<page-size>` name and/or an orientation keyword.
    Named(CssPageNamedSize),
}
/// A named page size and/or orientation; at least one component is present.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageNamedSize {
    size: Option<CssPageSize>,
    orientation: Option<CssPageOrientation>,
}
impl CssPageNamedSize {
    /// Creates a named size from an optional media name and an optional orientation.
    ///
    /// Returns `None` when both are absent.
    #[must_use]
    pub fn try_new(
        size: Option<CssPageSize>,
        orientation: Option<CssPageOrientation>,
    ) -> Option<Self> {
        (size.is_some() || orientation.is_some()).then_some(Self { size, orientation })
    }
    /// Returns the media name, or `None` when only an orientation was authored.
    #[must_use]
    pub const fn size(&self) -> Option<CssPageSize> {
        self.size
    }
    /// Returns the orientation keyword, or `None` when none was authored.
    #[must_use]
    pub const fn orientation(&self) -> Option<CssPageOrientation> {
        self.orientation
    }
}
/// The value of the `marks` descriptor: whether crop marks and registration marks are printed. `none` is neither flag set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssPageMarks {
    crop: bool,
    cross: bool,
}
impl CssPageMarks {
    pub(crate) const fn new(crop: bool, cross: bool) -> Self {
        Self { crop, cross }
    }
    /// Returns whether crop marks (`crop`) are requested.
    #[must_use]
    pub const fn crop(self) -> bool {
        self.crop
    }
    /// Returns whether registration marks (`cross`) are requested.
    #[must_use]
    pub const fn cross(self) -> bool {
        self.cross
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PageValueData {
    Size(CssPageSizeValue),
    PageOrientation(CssPageOutputOrientation),
    Marks(CssPageMarks),
    Bleed(CssAutoOr<CssNumeric<CssLength>>),
    Global(CssGlobalKeyword),
    Pending(crate::declaration::CssSubstitutionDependentValue),
}
/// A borrowed checked page descriptor value.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPageDescriptorValueRef<'a> {
    /// A `size` value.
    Size(&'a CssPageSizeValue),
    /// A `page-orientation` value.
    PageOrientation(CssPageOutputOrientation),
    /// A `marks` value.
    Marks(CssPageMarks),
    /// A `bleed` value: `auto` or a length.
    Bleed(&'a CssAutoOr<CssNumeric<CssLength>>),
    /// A CSS-wide keyword authored as the whole value.
    Global(CssGlobalKeyword),
    /// A value whose grammar depends on later substitution and is still symbolic.
    Pending(&'a crate::declaration::CssSubstitutionDependentValue),
}
/// Checked domain-specific value. Every constructor is parser-owned; raw edits retain value-only
/// origins.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageDescriptorValue {
    kind: CssPageDescriptorKind,
    pub(crate) data: PageValueData,
    components: CssComponentValues,
    origin: CssParsedOrigin,
}
impl CssPageDescriptorValue {
    pub(crate) fn from_parsed(
        kind: CssPageDescriptorKind,
        data: PageValueData,
        components: CssComponentValues,
        origin: CssParsedOrigin,
    ) -> Self {
        Self {
            kind,
            data,
            components,
            origin,
        }
    }
    /// Returns which descriptor this value was checked as.
    #[must_use]
    pub const fn kind(&self) -> CssPageDescriptorKind {
        self.kind
    }
    /// Returns the component values the descriptor was checked from, with their original provenance.
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }
    /// Returns the original raw-source window of the value.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
    /// Returns the checked semantics as a borrowed view.
    #[must_use]
    pub const fn view(&self) -> CssPageDescriptorValueRef<'_> {
        match &self.data {
            PageValueData::Size(v) => CssPageDescriptorValueRef::Size(v),
            PageValueData::PageOrientation(v) => CssPageDescriptorValueRef::PageOrientation(*v),
            PageValueData::Marks(v) => CssPageDescriptorValueRef::Marks(*v),
            PageValueData::Bleed(v) => CssPageDescriptorValueRef::Bleed(v),
            PageValueData::Global(v) => CssPageDescriptorValueRef::Global(*v),
            PageValueData::Pending(v) => CssPageDescriptorValueRef::Pending(v),
        }
    }
}
/// One page descriptor occurrence: a checked value with its importance and, when parsed or built from a named occurrence, that occurrence.
///
/// Clones are cheap and share their data.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageDescriptor {
    inner: std::sync::Arc<PageDescriptorData>,
}
#[derive(Debug, PartialEq)]
struct PageDescriptorData {
    value: CssPageDescriptorValue,
    importance: CssImportance,
    occurrence: Option<CssDescriptorOccurrence<CssPageDescriptorValue>>,
}
impl CssPageDescriptor {
    /// Creates a descriptor from a checked value and an importance, with no named occurrence.
    #[must_use]
    pub fn new(value: CssPageDescriptorValue, importance: CssImportance) -> Self {
        Self {
            inner: std::sync::Arc::new(PageDescriptorData {
                value,
                importance,
                occurrence: None,
            }),
        }
    }
    pub(crate) fn from_parsed(
        value: CssPageDescriptorValue,
        importance: CssImportance,
        name: CssParsedOrigin,
    ) -> Self {
        let occurrence = CssDescriptorOccurrence::from_parsed(
            value.clone(),
            name,
            value.origin.clone(),
            value.components.clone(),
        );
        Self {
            inner: std::sync::Arc::new(PageDescriptorData {
                value,
                importance,
                occurrence: Some(occurrence),
            }),
        }
    }
    /// Returns the checked value.
    #[must_use]
    pub fn value(&self) -> &CssPageDescriptorValue {
        &self.inner.value
    }
    /// Returns whether the descriptor was declared `!important`.
    #[must_use]
    pub fn importance(&self) -> CssImportance {
        self.inner.importance
    }
    /// Changes selected priority while retaining the original named/value occurrence.
    #[must_use]
    pub fn with_importance(&self, importance: CssImportance) -> Self {
        Self {
            inner: std::sync::Arc::new(PageDescriptorData {
                value: self.inner.value.clone(),
                importance,
                occurrence: self.inner.occurrence.clone(),
            }),
        }
    }
    /// Returns the original named occurrence of the descriptor, or `None` when it was created with [`Self::new`].
    #[must_use]
    pub fn occurrence(&self) -> Option<&CssDescriptorOccurrence<CssPageDescriptorValue>> {
        self.inner.occurrence.as_ref()
    }
}
/// One member of a `@page` declaration block, in authored order.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssPageDeclaration {
    /// An ordinary declaration valid in a page context.
    Property(CssDeclaration),
    /// A page descriptor such as `size`.
    Descriptor(CssPageDescriptor),
}
/// Checked Page declarations; ordered authored inventory and ordinary property projection stay
/// distinct.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageDeclarationBlock {
    occurrences: Vec<CssPageDeclaration>,
    properties: CssDeclarationList,
}
impl CssPageDeclarationBlock {
    pub(crate) fn from_parsed(occurrences: Vec<CssPageDeclaration>) -> Self {
        let properties = CssDeclarationList::new(
            occurrences
                .iter()
                .filter_map(|entry| match entry {
                    CssPageDeclaration::Property(v) => Some(v.clone()),
                    _ => None,
                })
                .collect(),
        );
        Self {
            occurrences,
            properties,
        }
    }
    /// Returns every declaration in authored order, descriptors and properties interleaved.
    #[must_use]
    pub fn occurrences(&self) -> &[CssPageDeclaration] {
        &self.occurrences
    }
    /// Returns the ordinary property declarations, without the descriptors.
    #[must_use]
    pub const fn properties(&self) -> &CssDeclarationList {
        &self.properties
    }
    /// Returns whether the block has no declarations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.occurrences.is_empty()
    }
    /// Returns the descriptor of the given kind that wins: the last `!important` occurrence, otherwise the last occurrence, or `None` when there is none.
    #[must_use]
    pub fn effective_descriptor(&self, kind: CssPageDescriptorKind) -> Option<&CssPageDescriptor> {
        self.occurrences
            .iter()
            .rev()
            .filter_map(|entry| match entry {
                CssPageDeclaration::Descriptor(v) if v.value().kind() == kind => Some(v),
                _ => None,
            })
            .find(|v| v.importance() == CssImportance::Important)
            .or_else(|| {
                self.occurrences.iter().rev().find_map(|entry| match entry {
                    CssPageDeclaration::Descriptor(v) if v.value().kind() == kind => Some(v),
                    _ => None,
                })
            })
    }
}
/// The name of a page-margin box at-rule.
///
/// The page margin area is divided into sixteen boxes: four corners and three boxes along each edge.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssMarginBox {
    /// `@top-left-corner`: the box in the top-left corner.
    TopLeftCorner,
    /// `@top-left`: the left box along the top edge.
    TopLeft,
    /// `@top-center`: the center box along the top edge.
    TopCenter,
    /// `@top-right`: the right box along the top edge.
    TopRight,
    /// `@top-right-corner`: the box in the top-right corner.
    TopRightCorner,
    /// `@bottom-left-corner`: the box in the bottom-left corner.
    BottomLeftCorner,
    /// `@bottom-left`: the left box along the bottom edge.
    BottomLeft,
    /// `@bottom-center`: the center box along the bottom edge.
    BottomCenter,
    /// `@bottom-right`: the right box along the bottom edge.
    BottomRight,
    /// `@bottom-right-corner`: the box in the bottom-right corner.
    BottomRightCorner,
    /// `@left-top`: the top box along the left edge.
    LeftTop,
    /// `@left-middle`: the middle box along the left edge.
    LeftMiddle,
    /// `@left-bottom`: the bottom box along the left edge.
    LeftBottom,
    /// `@right-top`: the top box along the right edge.
    RightTop,
    /// `@right-middle`: the middle box along the right edge.
    RightMiddle,
    /// `@right-bottom`: the bottom box along the right edge.
    RightBottom,
}
impl CssMarginBox {
    /// Returns the CSS at-keyword name without the `@`, such as `top-left-corner`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TopLeftCorner => "top-left-corner",
            Self::TopLeft => "top-left",
            Self::TopCenter => "top-center",
            Self::TopRight => "top-right",
            Self::TopRightCorner => "top-right-corner",
            Self::BottomLeftCorner => "bottom-left-corner",
            Self::BottomLeft => "bottom-left",
            Self::BottomCenter => "bottom-center",
            Self::BottomRight => "bottom-right",
            Self::BottomRightCorner => "bottom-right-corner",
            Self::LeftTop => "left-top",
            Self::LeftMiddle => "left-middle",
            Self::LeftBottom => "left-bottom",
            Self::RightTop => "right-top",
            Self::RightMiddle => "right-middle",
            Self::RightBottom => "right-bottom",
        }
    }
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        [
            Self::TopLeftCorner,
            Self::TopLeft,
            Self::TopCenter,
            Self::TopRight,
            Self::TopRightCorner,
            Self::BottomLeftCorner,
            Self::BottomLeft,
            Self::BottomCenter,
            Self::BottomRight,
            Self::BottomRightCorner,
            Self::LeftTop,
            Self::LeftMiddle,
            Self::LeftBottom,
            Self::RightTop,
            Self::RightMiddle,
            Self::RightBottom,
        ]
        .into_iter()
        .find(|v| name.eq_ignore_ascii_case(v.as_str()))
    }
}
/// The ordinary property declarations of a margin rule; margin rules have no descriptors.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMarginDeclarationBlock {
    properties: CssDeclarationList,
}
impl CssMarginDeclarationBlock {
    pub(crate) fn from_parsed(properties: CssDeclarationList) -> Self {
        Self { properties }
    }
    /// Returns the property declarations in authored order.
    #[must_use]
    pub const fn properties(&self) -> &CssDeclarationList {
        &self.properties
    }
    /// Returns whether the block has no declarations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}
/// Authored margin rule data, including any retained detached input occurrence.
/// Equality includes that optional parsed origin; live CSSOM identity/publication
/// facts remain separate from equality of this CSS-owned authored carrier.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMarginRule {
    name: CssMarginBox,
    declarations: CssMarginDeclarationBlock,
    position: Option<CssSourcePosition>,
    detached_origin: Option<CssParsedOrigin>,
}
impl CssMarginRule {
    /// Creates a margin rule from a box name and its declarations, with no source position or detached origin.
    #[must_use]
    pub const fn new(name: CssMarginBox, declarations: CssMarginDeclarationBlock) -> Self {
        Self {
            name,
            declarations,
            position: None,
            detached_origin: None,
        }
    }
    pub(crate) const fn from_parsed(
        name: CssMarginBox,
        declarations: CssMarginDeclarationBlock,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            declarations,
            position: Some(position),
            detached_origin: None,
        }
    }
    /// Returns which margin box the rule styles.
    #[must_use]
    pub const fn name(&self) -> CssMarginBox {
        self.name
    }
    /// Returns the rule's property declarations.
    #[must_use]
    pub const fn declarations(&self) -> &CssMarginDeclarationBlock {
        &self.declarations
    }
    /// Returns the source position of the rule, or `None` when it was constructed programmatically.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
    /// The complete detached raw occurrence when admitted through the Page
    /// margin-rule boundary, excluding surrounding trivia but owning the full
    /// supplied input snapshot. Programmatic and Page-body child construction
    /// retain their existing positions/value origins and have no detached origin.
    #[must_use]
    pub const fn detached_origin(&self) -> Option<&CssParsedOrigin> {
        self.detached_origin.as_ref()
    }
    pub(crate) fn with_detached_origin(mut self, origin: CssParsedOrigin) -> Self {
        debug_assert_eq!(self.position, Some(origin.span().start()));
        self.detached_origin = Some(origin);
        self
    }
}
/// The contents of a `@page` rule: its declaration block and its margin rules.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageBody {
    declarations: CssPageDeclarationBlock,
    margin_rules: Vec<CssMarginRule>,
}
impl CssPageBody {
    /// Creates a body from a declaration block and margin rules in the given order.
    #[must_use]
    pub const fn new(
        declarations: CssPageDeclarationBlock,
        margin_rules: Vec<CssMarginRule>,
    ) -> Self {
        Self {
            declarations,
            margin_rules,
        }
    }
    /// Returns the declarations of the body.
    #[must_use]
    pub const fn declarations(&self) -> &CssPageDeclarationBlock {
        &self.declarations
    }
    /// Returns the margin rules in authored order.
    #[must_use]
    pub fn margin_rules(&self) -> &[CssMarginRule] {
        &self.margin_rules
    }
}
/// A parser-produced authored `@page` rule: its selector prelude, declarations and margin rules.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageRule {
    selectors: CssPageSelectorList,
    body: CssPageBody,
    position: CssSourcePosition,
}
impl CssPageRule {
    pub(crate) const fn new(
        selectors: CssPageSelectorList,
        body: CssPageBody,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            selectors,
            body,
            position,
        }
    }
    /// Returns the selector prelude, which is empty for a rule without selectors.
    #[must_use]
    pub const fn selectors(&self) -> &CssPageSelectorList {
        &self.selectors
    }
    /// Returns the declarations of the rule body.
    #[must_use]
    pub const fn declarations(&self) -> &CssPageDeclarationBlock {
        self.body.declarations()
    }
    /// Returns the margin rules of the rule body in authored order.
    #[must_use]
    pub fn margin_rules(&self) -> &[CssMarginRule] {
        self.body.margin_rules()
    }
    /// Returns the source position of the rule's `@page` at-keyword.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
    /// Returns a copy of the rule with its body replaced, keeping the selectors and position.
    #[must_use]
    pub fn with_body(&self, body: CssPageBody) -> Self {
        Self {
            body,
            ..self.clone()
        }
    }
    /// Returns a copy of the rule with its selector prelude replaced, keeping the body and position.
    #[must_use]
    pub fn with_selectors(&self, selectors: CssPageSelectorList) -> Self {
        Self {
            selectors,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssPageBlockErrorKind {
    PropertyNotApplicable,
    InvalidPropertyValue,
    Resource(crate::serialization::CssSerializationError),
}
/// Typed failure at the Page/margin domain reconstruction boundary.
///
/// The responsible declaration is located by [`Self::ordinal`]; a resource refusal is
/// exposed by [`Self::resource`] and is the error's source.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPageBlockError {
    kind: CssPageBlockErrorKind,
    ordinal: Option<usize>,
    declaration: Option<CssDeclaration>,
}
impl CssPageBlockError {
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether a declaration's property does not apply in the Page/margin domain.
        is_property_not_applicable => CssPageBlockErrorKind::PropertyNotApplicable,
        /// Returns whether a declaration's value is invalid in the Page/margin domain.
        is_invalid_property_value => CssPageBlockErrorKind::InvalidPropertyValue,
    }
    /// Returns the serialization resource refusal.
    #[must_use]
    pub const fn resource(&self) -> Option<&crate::serialization::CssSerializationError> {
        match &self.kind {
            CssPageBlockErrorKind::Resource(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the zero-based ordinal of the responsible declaration.
    #[must_use]
    pub const fn ordinal(&self) -> Option<usize> {
        self.ordinal
    }
    /// Returns the responsible declaration.
    #[must_use]
    pub const fn declaration(&self) -> Option<&CssDeclaration> {
        self.declaration.as_ref()
    }
}
impl std::fmt::Display for CssPageBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            CssPageBlockErrorKind::PropertyNotApplicable => {
                f.write_str("property does not apply in the Page/margin domain")
            }
            CssPageBlockErrorKind::InvalidPropertyValue => {
                f.write_str("invalid property value in the Page/margin domain")
            }
            CssPageBlockErrorKind::Resource(error) => {
                write!(f, "invalid Page/margin block: {error}")
            }
        }?;
        if let Some(ordinal) = self.ordinal {
            write!(f, " at declaration {ordinal}")?;
        }
        Ok(())
    }
}
impl std::error::Error for CssPageBlockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.resource()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
impl From<crate::serialization::CssSerializationError> for CssPageBlockError {
    fn from(error: crate::serialization::CssSerializationError) -> Self {
        Self {
            kind: CssPageBlockErrorKind::Resource(error),
            ordinal: None,
            declaration: None,
        }
    }
}
fn check_property(
    value: CssDeclaration,
    ordinal: usize,
) -> Result<CssDeclaration, CssPageBlockError> {
    let issue = if value
        .known()
        .is_some_and(|v| !crate::parser::page::is_page_margin_property(v.property()))
        || value.svg_glyph_orientation_vertical().is_some()
    {
        Some(CssPageBlockErrorKind::PropertyNotApplicable)
    } else if crate::parser::page::page_declaration_violation(
        value.body(),
        value.value_components(),
    )
    .is_some()
    {
        Some(CssPageBlockErrorKind::InvalidPropertyValue)
    } else {
        None
    };
    if let Some(kind) = issue {
        return Err(CssPageBlockError {
            kind,
            ordinal: Some(ordinal),
            declaration: Some(value),
        });
    }
    Ok(value.into_page_context())
}
impl CssPageDeclarationBlock {
    /// Checks the occurrences and creates a block, retaining their order.
    ///
    /// The occurrences are traversed under one cumulative budget given by `limits`.
    ///
    /// # Errors
    ///
    /// Returns an error naming the responsible declaration when a property does not apply in a page context or has an invalid value there, or when `limits` are exceeded.
    pub fn try_new(
        occurrences: Vec<CssPageDeclaration>,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<Self, CssPageBlockError> {
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer.node()?;
        let mut admitted = Vec::new();
        for (ordinal, value) in occurrences.into_iter().enumerate() {
            admitted.try_reserve(1).map_err(|_| {
                crate::serialization::CssSerializationError::new(
                    crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            let value = match value {
                CssPageDeclaration::Property(v) => {
                    let v = check_property(v, ordinal)?;
                    crate::declaration::declaration_serialization::append_authored_declaration(
                        v.body(),
                        v.value_components(),
                        v.importance(),
                        &mut writer,
                    )?;
                    CssPageDeclaration::Property(v)
                }
                CssPageDeclaration::Descriptor(v) => {
                    writer.node()?;
                    v.value().append_value(&mut writer)?;
                    CssPageDeclaration::Descriptor(v)
                }
            };
            admitted.push(value);
        }
        Ok(Self::from_parsed(admitted))
    }
}
impl CssMarginDeclarationBlock {
    /// Checks the declarations and creates a margin block, retaining their order.
    ///
    /// The declarations are traversed under one cumulative budget given by `limits`.
    ///
    /// # Errors
    ///
    /// Returns an error naming the responsible declaration when a property does not apply in a margin context or has an invalid value there, or when `limits` are exceeded.
    pub fn try_new(
        properties: Vec<CssDeclaration>,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<Self, CssPageBlockError> {
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer.node()?;
        let mut admitted = Vec::new();
        for (ordinal, value) in properties.into_iter().enumerate() {
            let value = check_property(value, ordinal)?;
            crate::declaration::declaration_serialization::append_authored_declaration(
                value.body(),
                value.value_components(),
                value.importance(),
                &mut writer,
            )?;
            admitted.try_reserve(1).map_err(|_| {
                crate::serialization::CssSerializationError::new(
                    crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            admitted.push(value);
        }
        Ok(Self {
            properties: CssDeclarationList::new(admitted),
        })
    }
    /// Projects the declarations into a selected property block.
    ///
    /// # Errors
    ///
    /// Returns an error when a declaration cannot be selected or `limits` are exceeded.
    pub fn try_specified_properties(
        &self,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<
        crate::declaration::CssSpecifiedDeclarationBlock,
        crate::declaration::CssDeclarationBlockError,
    > {
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        crate::declaration::CssSpecifiedDeclarationBlock::from_margin_sources(
            self.properties.iter(),
            &mut writer,
        )
    }
    /// Serializes the declarations as CSSOM text, without braces.
    ///
    /// Failure publishes no partial text.
    ///
    /// # Errors
    ///
    /// Returns an error when a declaration cannot be selected or `limits` are exceeded.
    pub fn serialize_cssom(
        &self,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<String, crate::declaration::CssDeclarationBlockError> {
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        let block = crate::declaration::CssSpecifiedDeclarationBlock::from_margin_sources(
            self.properties.iter(),
            &mut writer,
        )?;
        block.append_cssom(&mut writer)?;
        Ok(writer.css)
    }
}
impl CssPageSelectorList {
    /// Parses the complete raw Page prelude, including empty, named, compound and comma-list forms.
    /// All source coordinates belong to the original input; no at-rule wrapper is synthesized.
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
    ) -> crate::CssParseReport<Option<CssPageSelectorList>> {
        crate::parser::fragments::parse_page_selector_list(source.as_ref())
    }
}
impl CssMarginRule {
    /// Parses exactly one complete margin at-rule in a Page destination.
    ///
    /// The decoded at-keyword and empty prelude use the actual Page grammar;
    /// declaration recovery retains applicable properties/custom declarations and
    /// priority. Real framing and origins come from the complete raw input, including
    /// escaped names and surrounding trivia. No containing Page or brace is invented.
    /// Extra nontrivia rejects the complete input. EOF closures use existing recovery.
    ///
    /// `None` means failed preparation: typed diagnostics distinguish preliminary
    /// Syntax, placement, unsupported-name and resource failures. A retained child
    /// may have local recovery. This API establishes a Page destination; live CSSOM
    /// validates outside-Page hierarchy and insertion order after classification.
    ///
    /// The parser supplies the document mode and the cumulative component limits, which charge the full original input once.
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> crate::CssParseReport<Option<CssMarginRule>> {
        crate::parser::page_margin_rule::parse_with_context(
            source.as_ref(),
            parser.component_limits(),
            parser.context(),
        )
    }
}
impl CssPageDescriptorValue {
    /// Parses one complete raw Page descriptor value without a descriptor-name origin or priority.
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
        kind: CssPageDescriptorKind,
    ) -> crate::CssParseReport<Option<CssPageDescriptorValue>> {
        crate::parser::fragments::parse_page_descriptor_value(source.as_ref(), kind)
    }
}
