//! Authored font palettes and checked recursive mixes, without palette lookup.

use std::fmt;

use crate::{CssColorInterpolation, CssColorMixWeight, CssFontPaletteName};

/// A keyword, symbolic palette name, or checked authored palette mix.
/// Palette availability and equivalence depend on the selected font downstream.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontPalette {
    Normal,
    Light,
    Dark,
    Named(CssFontPaletteName),
    Mix(Box<CssFontPaletteMix>),
}

/// One palette and its optional literal or symbolic percentage.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteMixComponent {
    palette: CssFontPalette,
    weight: Option<CssColorMixWeight>,
}

impl CssFontPaletteMixComponent {
    #[must_use]
    pub const fn new(palette: CssFontPalette, weight: Option<CssColorMixWeight>) -> Self {
        Self { palette, weight }
    }

    #[must_use]
    pub const fn palette(&self) -> &CssFontPalette {
        &self.palette
    }

    #[must_use]
    pub const fn weight(&self) -> Option<&CssColorMixWeight> {
        self.weight.as_ref()
    }
}

/// Why a checked authored palette mix could not be constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontPaletteMixConstructionError {
    EmptyComponents,
    NestingLimit,
    CapacityOverflow,
}

impl fmt::Display for CssFontPaletteMixConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyComponents => "palette-mix requires at least one component",
            Self::NestingLimit => "palette-mix exceeds the structural nesting limit",
            Self::CapacityOverflow => "palette-mix structural capacity overflow",
        })
    }
}

impl std::error::Error for CssFontPaletteMixConstructionError {}

/// A nonempty ordered mix, retaining duplicate palettes and omitted interpolation.
/// Calculated percentages stay symbolic; this value does not mix font colors.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontPaletteMix {
    interpolation: Option<CssColorInterpolation>,
    components: Vec<CssFontPaletteMixComponent>,
    nesting_depth: u32,
}

impl CssFontPaletteMix {
    /// Checks the complete palette and percentage-calculation nesting depth.
    pub fn try_new(
        interpolation: Option<CssColorInterpolation>,
        components: Vec<CssFontPaletteMixComponent>,
    ) -> Result<Self, CssFontPaletteMixConstructionError> {
        if components.is_empty() {
            return Err(CssFontPaletteMixConstructionError::EmptyComponents);
        }
        let mut nesting_depth = 1;
        for component in &components {
            let palette_depth = match component.palette() {
                CssFontPalette::Mix(value) => value.nesting_depth,
                _ => 0,
            };
            let weight_depth = component
                .weight()
                .and_then(CssColorMixWeight::calculation)
                .map_or(0, |value| value.components().nesting_depth());
            let depth = palette_depth
                .max(weight_depth)
                .checked_add(1)
                .ok_or(CssFontPaletteMixConstructionError::CapacityOverflow)?;
            nesting_depth = nesting_depth.max(depth);
            if nesting_depth > crate::STRUCTURAL_NESTING_LIMIT {
                return Err(CssFontPaletteMixConstructionError::NestingLimit);
            }
        }
        Ok(Self {
            interpolation,
            components,
            nesting_depth,
        })
    }

    #[must_use]
    pub const fn interpolation(&self) -> Option<&CssColorInterpolation> {
        self.interpolation.as_ref()
    }

    #[must_use]
    pub fn components(&self) -> &[CssFontPaletteMixComponent] {
        &self.components
    }
}
