use crate::values::font::CssFontVariant;
use crate::values::font::{CssFontSize, CssFontStyle, CssFontWeight, CssFontWidthKeyword};
use crate::values::text::CssLineHeight;

pub(crate) fn is_css_wide_keyword(value: &str) -> bool {
    ["inherit", "initial", "unset", "revert", "revert-layer"]
        .iter()
        .any(|keyword| value.eq_ignore_ascii_case(keyword))
}

/// How a `<family-name>` was authored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFamilyNameKind {
    /// A quoted `<string>` name.
    Quoted,
    /// An unquoted sequence of one or more identifiers.
    IdentSequence,
    /// A CSS-defined generic family keyword or `generic()` function.
    Generic,
}

/// A generic family with CSS-defined meaning, including script-specific functions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGenericFontFamily {
    /// `serif`: faces whose glyphs have finishing strokes, flared or tapering ends, or serifed endings (including slab serifs).
    Serif,
    /// `sans-serif`: faces whose glyphs are generally low contrast and have plain stroke endings, without flaring, cross strokes or other ornamentation.
    SansSerif,
    /// `cursive`: faces whose glyphs generally use a script style that looks more like handwritten pen or brush writing than printed letterwork.
    Cursive,
    /// `fantasy`: primarily decorative or expressive faces.
    Fantasy,
    /// `monospace`: faces whose glyphs all have the same fixed width.
    Monospace,
    /// `system-ui`: the default user interface font of the platform on which the user agent is running.
    SystemUi,
    /// `math`: a face intended for mathematical expressions.
    Math,
    /// `ui-serif`: the serif variant of the system's user interface font.
    UiSerif,
    /// `ui-sans-serif`: the sans-serif variant of the system's user interface font.
    UiSansSerif,
    /// `ui-monospace`: the monospaced variant of the system's user interface font.
    UiMonospace,
    /// `ui-rounded`: the rounded variant of the system's user interface font.
    UiRounded,
    /// `generic(fangsong)`: Fang Song typefaces for Chinese, a relaxed intermediate form between Song (serif) and Kai.
    Fangsong,
    /// `generic(kai)`: Kai typefaces for Simplified and Traditional Chinese, a calligraphic style with notable handwriting features.
    Kai,
    /// `generic(khmer-mul)`: the Khmer style used for titles and headings, characterized by heavier, more rounded letter shapes.
    KhmerMul,
    /// `generic(nastaliq)`: the Nastaliq style, the standard way of writing Urdu and Kashmiri and often a preferred style for Persian and other languages.
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
///
/// # Specified serialization
///
/// Emits the retained quoted, identifier-sequence, or generic form.
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

    /// Returns how the name was authored.
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

    /// Returns the generic family, or `None` for a quoted or identifier-sequence name.
    #[must_use]
    pub const fn generic_family(&self) -> Option<CssGenericFontFamily> {
        match &self.representation {
            CssFontFamilyNameRepresentation::Generic(generic) => Some(*generic),
            CssFontFamilyNameRepresentation::Quoted(_)
            | CssFontFamilyNameRepresentation::IdentSequence { .. } => None,
        }
    }
}

/// # Specified serialization
///
/// Emits the nonempty family list in retained order and authored forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFamilyList {
    families: Vec<CssFontFamilyName>,
}

impl CssFontFamilyList {
    /// Checks a family list.
    ///
    /// Returns `None` when `families` is empty.
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

    /// Returns the family names in authored order; the slice is never empty.
    #[must_use]
    pub fn families(&self) -> &[CssFontFamilyName] {
        &self.families
    }
}

/// A system font keyword accepted by the `font` shorthand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSystemFont {
    /// `caption`: the font used for captioned controls such as buttons and drop-downs.
    Caption,
    /// `icon`: the font used to label icons.
    Icon,
    /// `menu`: the font used in menus.
    Menu,
    /// `message-box`: the font used in dialog boxes.
    MessageBox,
    /// `small-caption`: the font used for labeling small controls.
    SmallCaption,
    /// `status-bar`: the font used in window status bars.
    StatusBar,
}

/// # Specified serialization
///
/// Serializes the authored components in Fonts 4 grammar order. Optional `normal` components are
/// omitted because they have the same shorthand expansion as an omitted component.
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
    /// Checks the components of an explicit `font` shorthand value.
    ///
    /// Returns `None` when `families` is empty. The size and family list are required; the other components are `None` when omitted.
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

    /// Returns the authored `font-style` component, or `None` when it was omitted.
    #[must_use]
    pub const fn style(&self) -> Option<&CssFontStyle> {
        self.style.as_ref()
    }

    /// Returns the authored `font-variant` keyword, or `None` when it was omitted.
    #[must_use]
    pub const fn variant(&self) -> Option<CssFontVariant> {
        self.variant
    }

    /// Returns the authored `font-weight` component, or `None` when it was omitted.
    #[must_use]
    pub const fn weight(&self) -> Option<&CssFontWeight> {
        self.weight.as_ref()
    }

    /// Returns the authored width keyword (the `font-stretch` component), or `None` when it was omitted.
    #[must_use]
    pub const fn stretch(&self) -> Option<CssFontWidthKeyword> {
        self.stretch
    }

    /// Returns the required `font-size` component.
    #[must_use]
    pub const fn size(&self) -> &CssFontSize {
        &self.size
    }

    /// Returns the authored line height written after `/`, or `None` when it was omitted.
    #[must_use]
    pub const fn line_height(&self) -> Option<&CssLineHeight> {
        self.line_height.as_ref()
    }

    /// Returns the required `font-family` list.
    #[must_use]
    pub const fn families(&self) -> &CssFontFamilyList {
        &self.families
    }
}

/// # Specified serialization
///
/// Serializes a complete authored shorthand without resolving system fonts.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontValue {
    /// An explicit `[ <font-style> || <font-variant-css2> || <font-weight> || <font-width-css3> ]? <font-size> [ / <'line-height'> ]? <font-family>#` value.
    Explicit(CssExplicitFont),
    /// A system font keyword.
    System(CssSystemFont),
}
