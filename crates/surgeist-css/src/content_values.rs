//! Checked, symbolic CSS Generated Content 3 values.

use crate::{
    CssBuiltInCounterStyle, CssContent, CssContentItem, CssContentList, CssContentString,
    CssCounterFunction, CssCounterName, CssCounterStyle, CssCounterStyleName, CssCountersFunction,
    CssIdent, CssImage, CssImageValue, CssUrl,
};

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
    items: Vec<CssContentValueItem>,
    alternative: Option<CssContentAlternative>,
}
impl CssGeneratedContent {
    #[must_use]
    pub fn try_new(
        items: Vec<CssContentValueItem>,
        alternative: Option<CssContentAlternative>,
    ) -> Option<Self> {
        (!items.is_empty()).then_some(Self { items, alternative })
    }
    #[must_use]
    pub fn body(&self) -> CssGeneratedContentBodyRef<'_> {
        match self.items.as_slice() {
            [CssContentValueItem::Image(image)] => CssGeneratedContentBodyRef::Replacement(image),
            items => CssGeneratedContentBodyRef::List(items),
        }
    }
    #[must_use]
    pub fn items(&self) -> &[CssContentValueItem] {
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

pub(crate) fn content_i01(value: &CssContentValue) -> Option<CssContent> {
    match value {
        CssContentValue::Normal => Some(CssContent::Normal),
        CssContentValue::None => Some(CssContent::None),
        CssContentValue::Generated(generated) => {
            if generated.alternative().is_some() {
                return None;
            }
            let items = generated
                .items()
                .iter()
                .map(legacy_item)
                .collect::<Option<Vec<_>>>()?;
            Some(CssContent::Items(CssContentList::try_new(items)?))
        }
    }
}

fn legacy_item(item: &CssContentValueItem) -> Option<CssContentItem> {
    Some(match item {
        CssContentValueItem::String(value) => CssContentItem::String(value.clone()),
        CssContentValueItem::Image(image) => match image.value() {
            CssImageValue::Url(url) => CssContentItem::Url(url.clone()),
            _ => return None,
        },
        CssContentValueItem::OpenQuote => CssContentItem::OpenQuote,
        CssContentValueItem::CloseQuote => CssContentItem::CloseQuote,
        CssContentValueItem::NoOpenQuote => CssContentItem::NoOpenQuote,
        CssContentValueItem::NoCloseQuote => CssContentItem::NoCloseQuote,
        CssContentValueItem::Counter(value) => CssContentItem::Counter(CssCounterFunction::new(
            CssCounterName::try_new(value.name().as_str())?,
            legacy_optional_style(value.style())?,
        )),
        CssContentValueItem::Counters(value) => CssContentItem::Counters(CssCountersFunction::new(
            CssCounterName::try_new(value.name().as_str())?,
            value.separator().clone(),
            legacy_optional_style(value.style())?,
        )),
        _ => return None,
    })
}

pub(crate) fn legacy_style(style: &CssCounterStyleValue) -> Option<CssCounterStyle> {
    let CssCounterStyleValue::Named(name) = style else {
        return None;
    };
    let builtin = match name.as_str() {
        "disc" => Some(CssBuiltInCounterStyle::Disc),
        "circle" => Some(CssBuiltInCounterStyle::Circle),
        "square" => Some(CssBuiltInCounterStyle::Square),
        "decimal" => Some(CssBuiltInCounterStyle::Decimal),
        "decimal-leading-zero" => Some(CssBuiltInCounterStyle::DecimalLeadingZero),
        "lower-alpha" => Some(CssBuiltInCounterStyle::LowerAlpha),
        "upper-alpha" => Some(CssBuiltInCounterStyle::UpperAlpha),
        "lower-latin" => Some(CssBuiltInCounterStyle::LowerLatin),
        "upper-latin" => Some(CssBuiltInCounterStyle::UpperLatin),
        "lower-roman" => Some(CssBuiltInCounterStyle::LowerRoman),
        "upper-roman" => Some(CssBuiltInCounterStyle::UpperRoman),
        _ => None,
    };
    match builtin {
        Some(builtin) => Some(CssCounterStyle::BuiltIn(builtin)),
        None => Some(CssCounterStyle::Named(CssCounterStyleName::try_new(
            name.as_str(),
        )?)),
    }
}

fn legacy_optional_style(style: Option<&CssCounterStyleValue>) -> Option<Option<CssCounterStyle>> {
    match style {
        None => Some(None),
        Some(style) => Some(Some(legacy_style(style)?)),
    }
}
