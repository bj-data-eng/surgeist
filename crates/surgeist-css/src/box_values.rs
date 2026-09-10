//! Checked authored values for box and border property grammars.

use crate::syntax::{CssAuthoredColor, CssColor, CssParsedColor};

/// The four authored side colors expanded from a `border-color` shorthand.
///
/// Colors retain their specified branches, including symbolic `currentcolor`;
/// constructing this value does not resolve them against an element's style.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderColors {
    sides: [CssAuthoredColor; 4],
}

impl CssBorderColors {
    /// Expands one through four checked colors in top, right, bottom, left order.
    ///
    /// One color applies to all sides. Two set the vertical and horizontal
    /// sides. Three set the top, horizontal sides, and bottom. Empty inputs and
    /// inputs with more than four colors are rejected.
    #[must_use]
    pub fn try_new(colors: Vec<CssAuthoredColor>) -> Option<Self> {
        let sides = match colors.as_slice() {
            [all] => [all.clone(), all.clone(), all.clone(), all.clone()],
            [vertical, horizontal] => [
                vertical.clone(),
                horizontal.clone(),
                vertical.clone(),
                horizontal.clone(),
            ],
            [top, horizontal, bottom] => [
                top.clone(),
                horizontal.clone(),
                bottom.clone(),
                horizontal.clone(),
            ],
            [top, right, bottom, left] => {
                [top.clone(), right.clone(), bottom.clone(), left.clone()]
            }
            _ => return None,
        };
        Some(Self { sides })
    }

    #[must_use]
    pub const fn top(&self) -> &CssAuthoredColor {
        &self.sides[0]
    }

    #[must_use]
    pub const fn right(&self) -> &CssAuthoredColor {
        &self.sides[1]
    }

    #[must_use]
    pub const fn bottom(&self) -> &CssAuthoredColor {
        &self.sides[2]
    }

    #[must_use]
    pub const fn left(&self) -> &CssAuthoredColor {
        &self.sides[3]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedBorderColors {
    current: CssBorderColors,
    i01_subset: Option<CssColor>,
}

impl CssParsedBorderColors {
    pub(crate) fn try_new(colors: Vec<CssParsedColor>) -> Option<Self> {
        let i01_subset = match colors.as_slice() {
            [color] => color.i01_subset().cloned(),
            _ => None,
        };
        let current = CssBorderColors::try_new(
            colors
                .into_iter()
                .map(|color| color.into_parts().0)
                .collect(),
        )?;
        Some(Self {
            current,
            i01_subset,
        })
    }

    pub(crate) fn into_parts(self) -> (CssBorderColors, Option<CssColor>) {
        (self.current, self.i01_subset)
    }
}
