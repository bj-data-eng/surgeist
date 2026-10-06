//! Checked, symbolic CSS Generated Content 3 values.

use crate::{CssContentString, CssIdent, CssImage, CssUrl};

fn reserved(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "default"
    )
}

/// A generic decoded `<custom-ident>` for Content 3 functions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContentName(CssIdent);

impl CssContentName {
    #[must_use]
    pub fn try_new(name: CssIdent) -> Option<Self> {
        (!reserved(name.as_str())).then_some(Self(name))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A Lists 3 counter name, excluding `none` in addition to generic reserved names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContentCounterName(CssContentName);

impl CssContentCounterName {
    #[must_use]
    pub fn try_new(name: CssIdent) -> Option<Self> {
        let name = CssContentName::try_new(name)?;
        (!name.as_str().eq_ignore_ascii_case("none")).then_some(Self(name))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// Counter Styles 3's predefined counter-style definitions supply these names. They normalize to
// lowercase when used as counter styles; arbitrary custom names retain case.
const PREDEFINED_STYLES: &[&str] = &[
    "decimal",
    "decimal-leading-zero",
    "arabic-indic",
    "armenian",
    "upper-armenian",
    "lower-armenian",
    "bengali",
    "cambodian",
    "khmer",
    "cjk-decimal",
    "devanagari",
    "georgian",
    "gujarati",
    "gurmukhi",
    "hebrew",
    "kannada",
    "lao",
    "malayalam",
    "mongolian",
    "myanmar",
    "oriya",
    "persian",
    "lower-roman",
    "upper-roman",
    "tamil",
    "telugu",
    "thai",
    "tibetan",
    "lower-alpha",
    "lower-latin",
    "upper-alpha",
    "upper-latin",
    "lower-greek",
    "hiragana",
    "hiragana-iroha",
    "katakana",
    "katakana-iroha",
    "disc",
    "circle",
    "square",
    "disclosure-open",
    "disclosure-closed",
    "cjk-earthly-branch",
    "cjk-heavenly-stem",
    "japanese-informal",
    "japanese-formal",
    "korean-hangul-formal",
    "korean-hanja-informal",
    "korean-hanja-formal",
    "simp-chinese-informal",
    "simp-chinese-formal",
    "trad-chinese-informal",
    "trad-chinese-formal",
    "cjk-ideographic",
    "ethiopic-numeric",
];

/// One of the five `symbols()` systems permitted as a counter style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSymbolsSystem {
    Cyclic,
    Numeric,
    Alphabetic,
    Symbolic,
    Fixed,
}

/// One string or image symbol in an authored `symbols()` style.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterSymbolValue {
    String(CssContentString),
    Image(CssImage),
}

/// A checked `symbols()` counter style.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSymbolsStyleValue {
    system: Option<CssSymbolsSystem>,
    symbols: Vec<CssCounterSymbolValue>,
}

impl CssSymbolsStyleValue {
    #[must_use]
    pub fn try_new(
        system: Option<CssSymbolsSystem>,
        symbols: Vec<CssCounterSymbolValue>,
    ) -> Option<Self> {
        let minimum = if matches!(
            system,
            Some(CssSymbolsSystem::Numeric | CssSymbolsSystem::Alphabetic)
        ) {
            2
        } else {
            1
        };
        (symbols.len() >= minimum).then_some(Self { system, symbols })
    }

    #[must_use]
    pub const fn system(&self) -> Option<CssSymbolsSystem> {
        self.system
    }
    #[must_use]
    pub fn symbols(&self) -> &[CssCounterSymbolValue] {
        &self.symbols
    }
}

/// A checked named counter-style reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterStyleReference(CssIdent);

impl CssCounterStyleReference {
    #[must_use]
    pub fn try_new(name: CssIdent) -> Option<Self> {
        let checked = CssContentCounterName::try_new(name)?;
        let spelling = checked.as_str();
        if PREDEFINED_STYLES
            .iter()
            .any(|value| value.eq_ignore_ascii_case(spelling))
        {
            return Some(Self(CssIdent::try_new(spelling.to_ascii_lowercase()).ok()?));
        }
        Some(Self(checked.0.0))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A checked named or functional counter-style reference.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleValue {
    Named(CssCounterStyleReference),
    Symbols(CssSymbolsStyleValue),
}

impl CssCounterStyleValue {
    #[must_use]
    pub fn try_named(name: CssIdent) -> Option<Self> {
        CssCounterStyleReference::try_new(name).map(Self::Named)
    }

    #[must_use]
    pub fn named(&self) -> Option<&CssCounterStyleReference> {
        match self {
            Self::Named(name) => Some(name),
            Self::Symbols(_) => None,
        }
    }
}

/// `counter()` arguments, kept symbolic until a counter tree is available.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContentCounter {
    name: CssContentCounterName,
    style: Option<CssCounterStyleValue>,
}

impl CssContentCounter {
    #[must_use]
    pub const fn new(name: CssContentCounterName, style: Option<CssCounterStyleValue>) -> Self {
        Self { name, style }
    }
    #[must_use]
    pub const fn name(&self) -> &CssContentCounterName {
        &self.name
    }
    #[must_use]
    pub const fn style(&self) -> Option<&CssCounterStyleValue> {
        self.style.as_ref()
    }
}

/// `counters()` arguments with a required string separator.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContentCounters {
    name: CssContentCounterName,
    separator: CssContentString,
    style: Option<CssCounterStyleValue>,
}

impl CssContentCounters {
    #[must_use]
    pub const fn new(
        name: CssContentCounterName,
        separator: CssContentString,
        style: Option<CssCounterStyleValue>,
    ) -> Self {
        Self {
            name,
            separator,
            style,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &CssContentCounterName {
        &self.name
    }
    #[must_use]
    pub const fn separator(&self) -> &CssContentString {
        &self.separator
    }
    #[must_use]
    pub const fn style(&self) -> Option<&CssCounterStyleValue> {
        self.style.as_ref()
    }
}

/// Target location remains authored; resolution belongs downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContentTarget {
    String(CssContentString),
    Url(CssUrl),
}

/// `target-counter()` arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTargetCounter {
    target: CssContentTarget,
    name: CssContentName,
    style: Option<CssCounterStyleValue>,
}
impl CssTargetCounter {
    #[must_use]
    pub const fn new(
        target: CssContentTarget,
        name: CssContentName,
        style: Option<CssCounterStyleValue>,
    ) -> Self {
        Self {
            target,
            name,
            style,
        }
    }
    #[must_use]
    pub const fn target(&self) -> &CssContentTarget {
        &self.target
    }
    #[must_use]
    pub const fn name(&self) -> &CssContentName {
        &self.name
    }
    #[must_use]
    pub const fn style(&self) -> Option<&CssCounterStyleValue> {
        self.style.as_ref()
    }
}

/// `target-counters()` arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTargetCounters {
    target: CssContentTarget,
    name: CssContentName,
    separator: CssContentString,
    style: Option<CssCounterStyleValue>,
}
impl CssTargetCounters {
    #[must_use]
    pub const fn new(
        target: CssContentTarget,
        name: CssContentName,
        separator: CssContentString,
        style: Option<CssCounterStyleValue>,
    ) -> Self {
        Self {
            target,
            name,
            separator,
            style,
        }
    }
    #[must_use]
    pub const fn target(&self) -> &CssContentTarget {
        &self.target
    }
    #[must_use]
    pub const fn name(&self) -> &CssContentName {
        &self.name
    }
    #[must_use]
    pub const fn separator(&self) -> &CssContentString {
        &self.separator
    }
    #[must_use]
    pub const fn style(&self) -> Option<&CssCounterStyleValue> {
        self.style.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTargetTextMode {
    Content,
    Before,
    After,
    FirstLetter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTargetText {
    target: CssContentTarget,
    mode: Option<CssTargetTextMode>,
}
impl CssTargetText {
    #[must_use]
    pub const fn new(target: CssContentTarget, mode: Option<CssTargetTextMode>) -> Self {
        Self { target, mode }
    }
    #[must_use]
    pub const fn target(&self) -> &CssContentTarget {
        &self.target
    }
    #[must_use]
    pub const fn mode(&self) -> Option<CssTargetTextMode> {
        self.mode
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNamedStringMode {
    First,
    Start,
    Last,
    FirstExcept,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNamedString {
    name: CssContentName,
    mode: Option<CssNamedStringMode>,
}
impl CssNamedString {
    #[must_use]
    pub const fn new(name: CssContentName, mode: Option<CssNamedStringMode>) -> Self {
        Self { name, mode }
    }
    #[must_use]
    pub const fn name(&self) -> &CssContentName {
        &self.name
    }
    #[must_use]
    pub const fn mode(&self) -> Option<CssNamedStringMode> {
        self.mode
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContentReferenceMode {
    Text,
    Before,
    After,
    FirstLetter,
    Marker,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLeaderValue {
    Dotted,
    Solid,
    Space,
    String(CssContentString),
}

/// One general generated-content item; `attr()` declarations remain pending.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContentValueItem {
    String(CssContentString),
    Image(CssImage),
    Contents,
    OpenQuote,
    CloseQuote,
    NoOpenQuote,
    NoCloseQuote,
    Counter(CssContentCounter),
    Counters(CssContentCounters),
    Leader(CssLeaderValue),
    TargetCounter(CssTargetCounter),
    TargetCounters(CssTargetCounters),
    TargetText(CssTargetText),
    NamedString(CssNamedString),
    Content(Option<CssContentReferenceMode>),
}

/// Alternative text excludes images, quotes, contents, and leaders.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContentAlternativeItem {
    String(CssContentString),
    Counter(CssContentCounter),
    Counters(CssContentCounters),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssContentAlternative {
    items: Vec<CssContentAlternativeItem>,
}
impl CssContentAlternative {
    #[must_use]
    pub fn try_new(items: Vec<CssContentAlternativeItem>) -> Option<Self> {
        (!items.is_empty()).then_some(Self { items })
    }
    #[must_use]
    pub fn items(&self) -> &[CssContentAlternativeItem] {
        &self.items
    }
}

/// Borrowed classification that cannot mislabel a sole image as a content list.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGeneratedContentBodyRef<'a> {
    Replacement(&'a CssImage),
    List(&'a [CssContentValueItem]),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssGeneratedContent {
    items: CssContentList,
    alternative: Option<CssContentAlternative>,
}
impl CssGeneratedContent {
    #[must_use]
    pub fn try_new(
        items: Vec<CssContentValueItem>,
        alternative: Option<CssContentAlternative>,
    ) -> Option<Self> {
        CssContentList::try_new(items).map(|items| Self { items, alternative })
    }
    #[must_use]
    pub fn body(&self) -> CssGeneratedContentBodyRef<'_> {
        match self.items.items() {
            [CssContentValueItem::Image(image)] => CssGeneratedContentBodyRef::Replacement(image),
            items => CssGeneratedContentBodyRef::List(items),
        }
    }
    #[must_use]
    pub fn items(&self) -> &[CssContentValueItem] {
        self.items.items()
    }
    #[must_use]
    pub(crate) const fn list(&self) -> &CssContentList {
        &self.items
    }
    #[must_use]
    pub const fn alternative(&self) -> Option<&CssContentAlternative> {
        self.alternative.as_ref()
    }
}

/// Current authored `content` grammar. It does not resolve generated boxes.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContentValue {
    Normal,
    None,
    Generated(CssGeneratedContent),
}

/// A nonempty ordered `<content-list>`, without property keywords or alternative text.
/// A sole image remains a list item; only `content` classifies it as replacement.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContentList {
    items: Vec<CssContentValueItem>,
}
impl CssContentList {
    /// Checks the list's nonempty invariant; every supplied child is already checked.
    #[must_use]
    pub fn try_new(items: Vec<CssContentValueItem>) -> Option<Self> {
        (!items.is_empty()).then_some(Self { items })
    }
    /// Borrows items in authored order, including duplicates and explicit defaults.
    #[must_use]
    pub fn items(&self) -> &[CssContentValueItem] {
        &self.items
    }
}

/// One string-set assignment, preserving a nonempty list of separate string leaves.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssStringSetEntry {
    name: CssContentName,
    strings: Vec<CssContentString>,
}
impl CssStringSetEntry {
    /// Rejects an assignment with no strings; empty individual strings remain valid.
    #[must_use]
    pub fn try_new(name: CssContentName, strings: Vec<CssContentString>) -> Option<Self> {
        (!strings.is_empty()).then_some(Self { name, strings })
    }
    /// Borrows the decoded, case-sensitive generic Content name.
    #[must_use]
    pub const fn name(&self) -> &CssContentName {
        &self.name
    }
    /// Borrows separate string leaves without computed concatenation.
    #[must_use]
    pub fn strings(&self) -> &[CssContentString] {
        &self.strings
    }
}

/// `none` or nonempty string-set assignments under the selected property-table grammar.
/// The conflicting enclosing Content 3 prose remains an unresolved source question.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssStringSet {
    entries: Option<Vec<CssStringSetEntry>>,
}
impl CssStringSet {
    /// Constructs the intrinsic `none` value.
    #[must_use]
    pub const fn none() -> Self {
        Self { entries: None }
    }
    /// Checks nonempty assignment order without merging repeated names.
    #[must_use]
    pub fn try_entries(entries: Vec<CssStringSetEntry>) -> Option<Self> {
        (!entries.is_empty()).then_some(Self {
            entries: Some(entries),
        })
    }
    /// `None` denotes the whole-value keyword; an assignment list is always nonempty.
    #[must_use]
    pub fn entries(&self) -> Option<&[CssStringSetEntry]> {
        self.entries.as_deref()
    }
}

/// `none` or an exact positive ordinary integer / deferred integer calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBookmarkLevel {
    level: Option<crate::CssPositiveIntegerValue>,
}
impl CssBookmarkLevel {
    /// Constructs the intrinsic `none` value without generating a bookmark.
    #[must_use]
    pub const fn none() -> Self {
        Self { level: None }
    }
    /// Normalizes bare calculation tokens through the shared positive literal boundary.
    /// Genuine function math retains its graph and defers computed rounding/range.
    #[must_use]
    pub fn try_new(level: crate::CssPositiveIntegerValue) -> Option<Self> {
        level
            .normalized_positive_root()
            .map(|level| Self { level: Some(level) })
    }
    /// Borrows the exact integer or deferred calculation; `None` is the CSS keyword.
    #[must_use]
    pub const fn level(&self) -> Option<&crate::CssPositiveIntegerValue> {
        self.level.as_ref()
    }
}

/// Authored bookmark subtree state; interaction and visibility are downstream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBookmarkState {
    Open,
    Closed,
}
