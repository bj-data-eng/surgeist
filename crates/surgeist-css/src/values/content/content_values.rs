//! Checked, symbolic CSS Generated Content 3 values.

use crate::values::common::{CssIdent, CssUrl};
use crate::values::content::CssContentString;
use crate::values::image::CssImage;

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
    /// Checks a decoded identifier as a generic content name.
    ///
    /// Returns `None` when the identifier is a CSS-wide keyword (`inherit`, `initial`, `unset`, `revert`, `revert-layer`) or `default`, compared ASCII case-insensitively.
    #[must_use]
    pub fn try_new(name: CssIdent) -> Option<Self> {
        (!reserved(name.as_str())).then_some(Self(name))
    }

    /// Returns the decoded, case-preserved identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A Lists 3 counter name, excluding `none` in addition to generic reserved names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContentCounterName(CssContentName);

impl CssContentCounterName {
    /// Checks a decoded identifier as a counter name.
    ///
    /// Returns `None` when the identifier is reserved as a generic content name or is `none` (ASCII case-insensitive).
    #[must_use]
    pub fn try_new(name: CssIdent) -> Option<Self> {
        let name = CssContentName::try_new(name)?;
        (!name.as_str().eq_ignore_ascii_case("none")).then_some(Self(name))
    }

    /// Returns the decoded, case-preserved counter name.
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
    /// `cyclic`: the symbols repeat in order, looping back to the first after the last.
    Cyclic,
    /// `numeric`: the symbols are the digits of a place-value numbering system; the first symbol is the digit 0.
    Numeric,
    /// `alphabetic`: the symbols are the digits of an alphabetic (bijective) numbering system, which has no digit for 0.
    Alphabetic,
    /// `symbolic`: the symbols repeat in order, doubling, tripling and so on the symbols on each successive pass.
    Symbolic,
    /// `fixed`: the symbols are used once, in order, and values beyond them fall back.
    Fixed,
}

/// One string or image symbol in an authored `symbols()` style.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterSymbolValue {
    /// A `<string>` symbol.
    String(CssContentString),
    /// An `<image>` symbol.
    Image(CssImage),
}

/// A checked `symbols()` counter style.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSymbolsStyleValue {
    system: Option<CssSymbolsSystem>,
    symbols: Vec<CssCounterSymbolValue>,
}

impl CssSymbolsStyleValue {
    /// Checks a `symbols()` counter style.
    ///
    /// Returns `None` when fewer symbols are supplied than the system needs: at least two for `numeric` and `alphabetic`, at least one otherwise, including when the system was omitted.
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

    /// Returns the authored system keyword, or `None` when `symbols()` omitted it.
    #[must_use]
    pub const fn system(&self) -> Option<CssSymbolsSystem> {
        self.system
    }
    /// Returns the symbols in authored order; the list is never empty.
    #[must_use]
    pub fn symbols(&self) -> &[CssCounterSymbolValue] {
        &self.symbols
    }
}

/// A checked named counter-style reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCounterStyleReference(CssIdent);

impl CssCounterStyleReference {
    /// Checks a decoded identifier as a counter-style name.
    ///
    /// Returns `None` when the identifier is not a valid counter name (see [`CssContentCounterName::try_new`]). A predefined counter-style name is normalized to lowercase; any other name keeps its case.
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
    /// Returns the counter-style name: lowercase for a predefined style, otherwise as authored.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A checked named or functional counter-style reference.
///
/// # Specified serialization
///
/// Serializes one checked named or `symbols()` counter style without resolving counters.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssCounterStyleValue {
    /// A reference to a counter style by name.
    Named(CssCounterStyleReference),
    /// A `symbols()` functional counter style.
    Symbols(CssSymbolsStyleValue),
}

impl CssCounterStyleValue {
    /// Checks `name` as a named counter-style reference.
    ///
    /// Returns `None` when the name is not a valid counter name.
    #[must_use]
    pub fn try_named(name: CssIdent) -> Option<Self> {
        CssCounterStyleReference::try_new(name).map(Self::Named)
    }

    /// Returns the named reference, or `None` when the value is a `symbols()` style.
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
    /// Creates `counter()` arguments from a checked counter name and an optional counter style.
    #[must_use]
    pub const fn new(name: CssContentCounterName, style: Option<CssCounterStyleValue>) -> Self {
        Self { name, style }
    }
    /// Returns the counter name.
    #[must_use]
    pub const fn name(&self) -> &CssContentCounterName {
        &self.name
    }
    /// Returns the authored counter style, or `None` when it was omitted.
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
    /// Creates `counters()` arguments from a checked counter name, the separator string and an optional counter style.
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
    /// Returns the counter name.
    #[must_use]
    pub const fn name(&self) -> &CssContentCounterName {
        &self.name
    }
    /// Returns the string placed between the values of nested counters.
    #[must_use]
    pub const fn separator(&self) -> &CssContentString {
        &self.separator
    }
    /// Returns the authored counter style, or `None` when it was omitted.
    #[must_use]
    pub const fn style(&self) -> Option<&CssCounterStyleValue> {
        self.style.as_ref()
    }
}

/// Target location remains authored; resolution belongs downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContentTarget {
    /// A `<string>` target, authored as text.
    String(CssContentString),
    /// A `<url>` target.
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
    /// Creates `target-counter()` arguments from the target, the counter name and an optional counter style.
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
    /// Returns the link target that selects the element to read the counter from.
    #[must_use]
    pub const fn target(&self) -> &CssContentTarget {
        &self.target
    }
    /// Returns the name of the counter to read.
    #[must_use]
    pub const fn name(&self) -> &CssContentName {
        &self.name
    }
    /// Returns the authored counter style, or `None` when it was omitted.
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
    /// Creates `target-counters()` arguments from the target, the counter name, the separator string and an optional counter style.
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
    /// Returns the link target that selects the element to read the counters from.
    #[must_use]
    pub const fn target(&self) -> &CssContentTarget {
        &self.target
    }
    /// Returns the name of the counters to read.
    #[must_use]
    pub const fn name(&self) -> &CssContentName {
        &self.name
    }
    /// Returns the string placed between the values of nested counters.
    #[must_use]
    pub const fn separator(&self) -> &CssContentString {
        &self.separator
    }
    /// Returns the authored counter style, or `None` when it was omitted.
    #[must_use]
    pub const fn style(&self) -> Option<&CssCounterStyleValue> {
        self.style.as_ref()
    }
}

/// The optional second argument of `target-text()`: which text of the target is retrieved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTargetTextMode {
    /// `content`: the text value of the target element.
    Content,
    /// `before`: the string value of the target's `::before` pseudo-element, as for `content(before)`.
    Before,
    /// `after`: the string value of the target's `::after` pseudo-element, as for `content(after)`.
    After,
    /// `first-letter`: the first letter of the target, as defined for the `::first-letter` pseudo-element.
    FirstLetter,
}

/// `target-text()` arguments, kept symbolic until the link target can be resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTargetText {
    target: CssContentTarget,
    mode: Option<CssTargetTextMode>,
}
impl CssTargetText {
    /// Creates `target-text()` arguments from the target and an optional text selection.
    #[must_use]
    pub const fn new(target: CssContentTarget, mode: Option<CssTargetTextMode>) -> Self {
        Self { target, mode }
    }
    /// Returns the link target that selects the element to read text from.
    #[must_use]
    pub const fn target(&self) -> &CssContentTarget {
        &self.target
    }
    /// Returns the selected text kind, or `None` when the second argument was omitted.
    #[must_use]
    pub const fn mode(&self) -> Option<CssTargetTextMode> {
        self.mode
    }
}

/// The optional second argument of `string()`: which assignment of a named string is used on a page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNamedStringMode {
    /// `first`: the value of the first assignment on the page, or the entry value if there is none; the default when the argument is omitted.
    First,
    /// `start`: the value of the first assignment if the element is the first element on the page, otherwise the entry value.
    Start,
    /// `last`: the exit value of the named string.
    Last,
    /// `first-except`: identical to `first`, except that the empty string is used on the page where the value is assigned.
    FirstExcept,
}

/// `string()` arguments, kept symbolic until a named string is available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNamedString {
    name: CssContentName,
    mode: Option<CssNamedStringMode>,
}
impl CssNamedString {
    /// Creates `string()` arguments from the name and an optional assignment selection.
    #[must_use]
    pub const fn new(name: CssContentName, mode: Option<CssNamedStringMode>) -> Self {
        Self { name, mode }
    }
    /// Returns the name of the named string.
    #[must_use]
    pub const fn name(&self) -> &CssContentName {
        &self.name
    }
    /// Returns the selected assignment, or `None` when the second argument was omitted.
    #[must_use]
    pub const fn mode(&self) -> Option<CssNamedStringMode> {
        self.mode
    }
}

/// The keyword argument of `content()`: which string value of the element is used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContentReferenceMode {
    /// `text`: the string value of the element; the default when the argument is omitted.
    Text,
    /// `before`: the string value of the `::before` pseudo-element.
    Before,
    /// `after`: the string value of the `::after` pseudo-element.
    After,
    /// `first-letter`: the first letter of the element, as defined for the `::first-letter` pseudo-element.
    FirstLetter,
    /// `marker`: the string value of the `::marker` pseudo-element.
    Marker,
}

/// The `<leader-type>` argument of `leader()`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLeaderValue {
    /// `dotted`: equivalent to `leader(".")`.
    Dotted,
    /// `solid`: equivalent to `leader("_")`.
    Solid,
    /// `space`: equivalent to `leader(" ")`.
    Space,
    /// An authored `<string>` leader pattern.
    String(CssContentString),
}

/// One general generated-content item; `attr()` declarations remain pending.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContentValueItem {
    /// A `<string>` item.
    String(CssContentString),
    /// An `<image>` item.
    Image(CssImage),
    /// `contents`: the element's descendants; it can be used only once per element.
    Contents,
    /// `open-quote`: replaced by the opening quotation mark that `quotes` defines for the current quote depth, and increments the depth.
    OpenQuote,
    /// `close-quote`: replaced by the closing quotation mark that `quotes` defines for the current quote depth, and decrements the depth.
    CloseQuote,
    /// `no-open-quote`: inserts nothing, but increments the quote depth.
    NoOpenQuote,
    /// `no-close-quote`: inserts nothing, but decrements the quote depth.
    NoCloseQuote,
    /// A `counter()` function.
    Counter(CssContentCounter),
    /// A `counters()` function.
    Counters(CssContentCounters),
    /// A `leader()` function.
    Leader(CssLeaderValue),
    /// A `target-counter()` function.
    TargetCounter(CssTargetCounter),
    /// A `target-counters()` function.
    TargetCounters(CssTargetCounters),
    /// A `target-text()` function.
    TargetText(CssTargetText),
    /// A `string()` function that copies a named string.
    NamedString(CssNamedString),
    /// A `content()` function; `None` when the keyword argument was omitted.
    Content(Option<CssContentReferenceMode>),
}

/// Alternative text excludes images, quotes, contents, and leaders.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContentAlternativeItem {
    /// A `<string>` item.
    String(CssContentString),
    /// A `counter()` function.
    Counter(CssContentCounter),
    /// A `counters()` function.
    Counters(CssContentCounters),
}

/// The alternative text that follows `/` in a `content` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContentAlternative {
    items: Vec<CssContentAlternativeItem>,
}
impl CssContentAlternative {
    /// Checks the alternative text items.
    ///
    /// Returns `None` when `items` is empty.
    #[must_use]
    pub fn try_new(items: Vec<CssContentAlternativeItem>) -> Option<Self> {
        (!items.is_empty()).then_some(Self { items })
    }
    /// Returns the alternative text items in authored order; the slice is never empty.
    #[must_use]
    pub fn items(&self) -> &[CssContentAlternativeItem] {
        &self.items
    }
}

/// Borrowed classification that cannot mislabel a sole image as a content list.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGeneratedContentBodyRef<'a> {
    /// The content is a single `<image>` that replaces the element's content.
    Replacement(&'a CssImage),
    /// The content is a list of items; this includes a list whose only item is not an image.
    List(&'a [CssContentValueItem]),
}

/// A `content` value that generates content: a nonempty `<content-list>` with optional alternative text.
#[derive(Clone, Debug, PartialEq)]
pub struct CssGeneratedContent {
    items: CssContentList,
    alternative: Option<CssContentAlternative>,
}
impl CssGeneratedContent {
    /// Checks the list and attaches the optional alternative text.
    ///
    /// Returns `None` when `items` is empty.
    #[must_use]
    pub fn try_new(
        items: Vec<CssContentValueItem>,
        alternative: Option<CssContentAlternative>,
    ) -> Option<Self> {
        CssContentList::try_new(items).map(|items| Self { items, alternative })
    }
    /// Classifies the list as a sole replacement image or an ordinary item list.
    #[must_use]
    pub fn body(&self) -> CssGeneratedContentBodyRef<'_> {
        match self.items.items() {
            [CssContentValueItem::Image(image)] => CssGeneratedContentBodyRef::Replacement(image),
            items => CssGeneratedContentBodyRef::List(items),
        }
    }
    /// Returns the content items in authored order; the slice is never empty.
    #[must_use]
    pub fn items(&self) -> &[CssContentValueItem] {
        self.items.items()
    }
    #[must_use]
    pub(crate) const fn list(&self) -> &CssContentList {
        &self.items
    }
    /// Returns the alternative text, or `None` when none followed `/`.
    #[must_use]
    pub const fn alternative(&self) -> Option<&CssContentAlternative> {
        self.alternative.as_ref()
    }
}

/// Current authored `content` grammar. It does not resolve generated boxes.
///
/// # Specified serialization
///
/// Serializes current authored content without resolving counters, targets, or generated boxes.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContentValue {
    /// `normal`: the initial value, whose computed behavior depends on the element or pseudo-element.
    Normal,
    /// `none`: on a pseudo-element, inhibits its creation; on an element, behaves as `normal`.
    None,
    /// A content list with optional alternative text.
    Generated(CssGeneratedContent),
}

/// A nonempty ordered `<content-list>`, without property keywords or alternative text.
/// A sole image remains a list item; only `content` classifies it as replacement.
///
/// # Specified serialization
///
/// Serializes an ordered content list without resolving its context.
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
///
/// # Specified serialization
///
/// Serializes separate named string assignments without concatenation.
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
///
/// # Specified serialization
///
/// Serializes the exact level or its symbolic integer calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBookmarkLevel {
    level:
        Option<crate::numeric::CssNumeric<crate::numeric::CssInteger, crate::numeric::CssPositive>>,
}

impl CssBookmarkLevel {
    /// Constructs the intrinsic `none` value without generating a bookmark.
    #[must_use]
    pub const fn none() -> Self {
        Self { level: None }
    }
    /// Retains a positive level; its type already admits only positive
    /// literals and keeps function math with deferred computed rounding/range.
    #[must_use]
    pub const fn new(
        level: crate::numeric::CssNumeric<crate::numeric::CssInteger, crate::numeric::CssPositive>,
    ) -> Self {
        Self { level: Some(level) }
    }
    /// Borrows the exact integer or deferred calculation; `None` is the CSS keyword.
    #[must_use]
    pub const fn level(
        &self,
    ) -> Option<&crate::numeric::CssNumeric<crate::numeric::CssInteger, crate::numeric::CssPositive>>
    {
        self.level.as_ref()
    }
}

/// Authored bookmark subtree state; interaction and visibility are downstream.
///
/// # Specified serialization
///
/// Serializes the authored bookmark state without toggling a subtree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBookmarkState {
    /// `open`: subsequent bookmarks with a greater `bookmark-level` are displayed, until another bookmark of the same or a lower level.
    Open,
    /// `closed`: subsequent bookmarks with a greater `bookmark-level` are not displayed, until another bookmark of the same or a lower level.
    Closed,
}
