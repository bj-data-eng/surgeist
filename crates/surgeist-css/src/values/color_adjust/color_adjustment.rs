//! Authored Color Adjustment 1 hints, without contextual negotiation or execution.

use std::fmt;

use crate::values::common::CssIdent;

/// Why a checked scheme name or list cannot express the selected property grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorSchemeConstructionError {
    kind: ColorSchemeConstructionKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ColorSchemeConstructionKind {
    /// A custom name is a CSS-wide keyword, `default`, or a property keyword.
    ReservedName,
    /// A scheme list needs at least one light, dark, or custom entry.
    EmptySchemes,
}
impl From<ColorSchemeConstructionKind> for CssColorSchemeConstructionError {
    fn from(kind: ColorSchemeConstructionKind) -> Self {
        Self { kind }
    }
}
impl CssColorSchemeConstructionError {
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether a custom name is a CSS-wide keyword, `default`, or a property keyword.
        is_reserved_name => ColorSchemeConstructionKind::ReservedName,
        /// Returns whether a scheme list needs at least one light, dark, or custom entry.
        is_empty_schemes => ColorSchemeConstructionKind::EmptySchemes,
    }
}
impl fmt::Display for CssColorSchemeConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match &self.kind {
            ColorSchemeConstructionKind::ReservedName => "reserved color-scheme custom identifier",
            ColorSchemeConstructionKind::EmptySchemes => {
                "color-scheme list requires at least one scheme"
            }
        })
    }
}

impl std::error::Error for CssColorSchemeConstructionError {}

/// One case-preserving unknown scheme name; it is assigned no contextual meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorSchemeName(CssIdent);

impl CssColorSchemeName {
    pub(crate) fn from_admitted_ident(name: CssIdent) -> Option<Self> {
        Self::try_new(name).ok()
    }
    /// Checks a representable decoded identifier against generic and local exclusions.
    /// `none`, `auto`, and `span` remain valid; escaped punctuation is retained.
    pub fn try_new(name: CssIdent) -> Result<Self, CssColorSchemeConstructionError> {
        let value = name.as_str();
        if crate::values::font::syntax::is_css_wide_keyword(value)
            || ["default", "normal", "light", "dark", "only"]
                .iter()
                .any(|keyword| value.eq_ignore_ascii_case(keyword))
        {
            return Err(CssColorSchemeConstructionError::from(
                ColorSchemeConstructionKind::ReservedName,
            ));
        }
        Ok(Self(name))
    }

    /// Returns the decoded spelling without folding its case or interpreting it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// One authored list entry, distinct from the media-query color-scheme preference.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorSchemeKeyword {
    /// `light`: the element supports a light color scheme.
    Light,
    /// `dark`: the element supports a dark color scheme.
    Dark,
    /// An authored custom `<custom-ident>` scheme name, retained without interpretation.
    Custom(CssColorSchemeName),
}

/// `normal`, or a checked nonempty ordered list with an optional `only` modifier.
///
/// The inherited initial is `normal`. Repeated entries and unknown names remain
/// authored data. This property applies to all elements and text, accepts no
/// percentages, has a computed normal keyword or ordered specified scheme list,
/// and animates discretely. Negotiation and used palettes belong downstream.
///
/// # Specified serialization
///
/// Writes canonical specified CSS with `only` last, without negotiation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorScheme {
    schemes: Vec<CssColorSchemeKeyword>,
    only: bool,
}

impl CssColorScheme {
    pub(crate) fn retained_backing_bytes_with<E>(
        &self,
        admit: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<usize, crate::source::CssRetainedPayloadError<E>> {
        use crate::source::CssRetainedPayloadError as Error;
        admit().map_err(Error::admission)?;
        let mut bytes = self
            .schemes
            .capacity()
            .checked_mul(std::mem::size_of::<CssColorSchemeKeyword>())
            .ok_or_else(Error::capacity_overflow)?;
        for scheme in &self.schemes {
            admit().map_err(Error::admission)?;
            if let CssColorSchemeKeyword::Custom(name) = scheme {
                admit().map_err(Error::admission)?;
                bytes = bytes
                    .checked_add(name.0.backing_capacity())
                    .ok_or_else(Error::capacity_overflow)?;
            }
        }
        Ok(bytes)
    }
    /// Constructs the normal branch without a list or modifier.
    #[must_use]
    pub const fn normal() -> Self {
        Self {
            schemes: Vec::new(),
            only: false,
        }
    }

    /// Constructs a nonempty list, preserving order and duplicates.
    pub fn try_new(
        schemes: Vec<CssColorSchemeKeyword>,
        only: bool,
    ) -> Result<Self, CssColorSchemeConstructionError> {
        if schemes.is_empty() {
            return Err(CssColorSchemeConstructionError::from(
                ColorSchemeConstructionKind::EmptySchemes,
            ));
        }
        Ok(Self { schemes, only })
    }

    /// Returns whether this is the `normal` branch, which has no scheme list.
    #[must_use]
    pub fn is_normal(&self) -> bool {
        self.schemes.is_empty()
    }

    /// Returns the ordered list, or an empty slice for the normal branch.
    #[must_use]
    pub fn schemes(&self) -> &[CssColorSchemeKeyword] {
        &self.schemes
    }

    /// Returns whether the `only` modifier was authored, which is always `false` for `normal`.
    #[must_use]
    pub const fn only(&self) -> bool {
        self.only
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssColorScheme {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.node()?;
        if self.is_normal() {
            writer.append("normal")?;
        } else {
            for (index, scheme) in self.schemes.iter().enumerate() {
                writer.node()?;
                if index != 0 {
                    writer.append(" ")?;
                }
                match scheme {
                    CssColorSchemeKeyword::Light => writer.append("light")?,
                    CssColorSchemeKeyword::Dark => writer.append("dark")?,
                    CssColorSchemeKeyword::Custom(name) => {
                        writer.append_identifier(name.as_str())?
                    }
                }
            }
            if self.only {
                writer.node()?;
                writer.append(" only")?;
            }
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssColorScheme);

/// An inherited, auto-initial forced-color hint; computed as specified and not animatable.
///
/// Applies to all elements and text, without percentages. `preserve-parent-color`
/// behaves like `none`, except downstream may use the parent's used color when
/// forced colors mode is active and `color` inherits from the parent. This value
/// does not perform that contextual operation or apply it outside forced colors mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssForcedColorAdjust {
    /// `auto`: the element's colors are automatically adjusted by the user agent in forced-colors mode.
    Auto,
    /// `none`: the element's colors are not automatically adjusted by the user agent in forced-colors mode.
    None,
    /// `preserve-parent-color`: like `none`, except that in forced-colors mode a `color` that inherits from its parent computes to the parent's used color.
    PreserveParentColor,
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssForcedColorAdjust
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::PreserveParentColor => "preserve-parent-color",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssForcedColorAdjust);

/// An inherited economy-initial output-device hint, retaining the specified keyword.
///
/// Applies to all elements, accepts no percentages, and animates discretely.
/// User preferences and output-device decisions remain downstream. The genuine
/// deprecated `color-adjust` shorthand uses this same grammar and sets only
/// `print-color-adjust`, while retaining a distinct authored property identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPrintColorAdjust {
    /// `economy`: the user agent should make the adjustments to the page's styling that it deems necessary and prudent for the output device.
    Economy,
    /// `exact`: the page uses color and styling on the element in a way that is important and significant and should not be changed except at the user's request.
    Exact,
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssPrintColorAdjust {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Economy => "economy",
            Self::Exact => "exact",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssPrintColorAdjust);
