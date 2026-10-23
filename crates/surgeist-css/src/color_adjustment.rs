//! Authored Color Adjustment 1 hints, without contextual negotiation or execution.

use std::fmt;

use crate::CssIdent;

/// Why a checked scheme name or list cannot express the selected property grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorSchemeConstructionError {
    /// A custom name is a CSS-wide keyword, `default`, or a property keyword.
    ReservedName,
    /// A scheme list needs at least one light, dark, or custom entry.
    EmptySchemes,
}

impl fmt::Display for CssColorSchemeConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ReservedName => "reserved color-scheme custom identifier",
            Self::EmptySchemes => "color-scheme list requires at least one scheme",
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
        if crate::syntax::is_css_wide_keyword(value)
            || ["default", "normal", "light", "dark", "only"]
                .iter()
                .any(|keyword| value.eq_ignore_ascii_case(keyword))
        {
            return Err(CssColorSchemeConstructionError::ReservedName);
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
    Light,
    Dark,
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
    ) -> Result<usize, crate::CssRetainedPayloadError<E>> {
        use crate::CssRetainedPayloadError as Error;
        admit().map_err(Error::Admission)?;
        let mut bytes = self
            .schemes
            .capacity()
            .checked_mul(std::mem::size_of::<CssColorSchemeKeyword>())
            .ok_or(Error::CapacityOverflow)?;
        for scheme in &self.schemes {
            admit().map_err(Error::Admission)?;
            if let CssColorSchemeKeyword::Custom(name) = scheme {
                admit().map_err(Error::Admission)?;
                bytes = bytes
                    .checked_add(name.0.backing_capacity())
                    .ok_or(Error::CapacityOverflow)?;
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
            return Err(CssColorSchemeConstructionError::EmptySchemes);
        }
        Ok(Self { schemes, only })
    }

    #[must_use]
    pub fn is_normal(&self) -> bool {
        self.schemes.is_empty()
    }

    /// Returns the ordered list, or an empty slice for the normal branch.
    #[must_use]
    pub fn schemes(&self) -> &[CssColorSchemeKeyword] {
        &self.schemes
    }

    #[must_use]
    pub const fn only(&self) -> bool {
        self.only
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssColorScheme {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
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
crate::specified_rule_serialization::specified_serialization!(CssColorScheme);

/// An inherited, auto-initial forced-color hint; computed as specified and not animatable.
///
/// Applies to all elements and text, without percentages. `preserve-parent-color`
/// behaves like `none`, except downstream may use the parent's used color when
/// forced colors mode is active and `color` inherits from the parent. This value
/// does not perform that contextual operation or apply it outside forced colors mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssForcedColorAdjust {
    Auto,
    None,
    PreserveParentColor,
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssForcedColorAdjust {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::PreserveParentColor => "preserve-parent-color",
        })
    }
}
crate::specified_rule_serialization::specified_serialization!(CssForcedColorAdjust);

/// An inherited economy-initial output-device hint, retaining the specified keyword.
///
/// Applies to all elements, accepts no percentages, and animates discretely.
/// User preferences and output-device decisions remain downstream. The genuine
/// deprecated `color-adjust` shorthand uses this same grammar and sets only
/// `print-color-adjust`, while retaining a distinct authored property identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPrintColorAdjust {
    Economy,
    Exact,
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssPrintColorAdjust {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(match self {
            Self::Economy => "economy",
            Self::Exact => "exact",
        })
    }
}
crate::specified_rule_serialization::specified_serialization!(CssPrintColorAdjust);
