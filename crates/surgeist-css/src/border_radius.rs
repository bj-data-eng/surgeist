//! Exact authored corner radii before box geometry resolution.

use crate::common_values::{comparison_context, side_element, side_pair_entry};
use crate::specified_inverse::InverseValueEquality as _;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssFourSides, CssLengthPercentage, CssNonNegative, CssNumeric, CssSerializationError,
    CssSidePair,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

/// One corner radius length: `<length-percentage [0,∞]>`.
///
/// The schema names each `border-*-radius` pair, `<length-percentage [0,∞]>{1,2}` (Backgrounds 3:
/// a horizontal radius and a vertical radius that copies it when omitted), through this name.
pub(crate) type CornerRadiusLength = CssNumeric<CssLengthPercentage, CssNonNegative>;

// Equal horizontal and vertical radii are written once, like every other `<T>{1,2}` value.
side_element!(
    CornerRadiusLength,
    same = |a, b, context| a.inverse_equal(b, context)
);
side_pair_entry!(CornerRadiusLength);

/// One to four horizontal radii, optionally followed by one to four vertical radii.
///
/// Named corners use top-left, top-right, bottom-right, bottom-left order.
/// Serialization charges one aggregate node plus each authored scalar; the
/// slash is a byte separator, not an extra node.
///
/// Each list follows the four-side rule of `margin` (one value for all corners, two for
/// opposite corner pairs, three, four) and is written in its shortest form; a vertical list
/// that equals the horizontal list is omitted with its slash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBorderRadiusShorthand {
    horizontal: CssFourSides<CornerRadiusLength>,
    authored_vertical: Option<CssFourSides<CornerRadiusLength>>,
}

impl CssBorderRadiusShorthand {
    /// Rejects an empty or overlong horizontal or supplied vertical list.
    #[must_use]
    pub fn try_new(
        horizontal: Vec<CssNumeric<CssLengthPercentage, CssNonNegative>>,
        authored_vertical: Option<Vec<CssNumeric<CssLengthPercentage, CssNonNegative>>>,
    ) -> Option<Self> {
        let horizontal = CssFourSides::try_new(crate::CssBoxSideKind::Physical, horizontal)?;
        let authored_vertical = match authored_vertical {
            Some(values) => Some(CssFourSides::try_new(
                crate::CssBoxSideKind::Physical,
                values,
            )?),
            None => None,
        };
        Some(Self {
            horizontal,
            authored_vertical,
        })
    }
    /// Borrows the one to four authored horizontal values.
    pub fn horizontal_values(&self) -> &[CssNumeric<CssLengthPercentage, CssNonNegative>] {
        self.horizontal.authored()
    }
    /// Borrows the explicitly authored vertical list, if a slash appeared.
    pub fn authored_vertical_values(
        &self,
    ) -> Option<&[CssNumeric<CssLengthPercentage, CssNonNegative>]> {
        self.authored_vertical.as_ref().map(CssFourSides::authored)
    }
    /// Borrows the effective vertical list, repeating the horizontal list without a slash.
    pub fn vertical_values(&self) -> &[CssNumeric<CssLengthPercentage, CssNonNegative>] {
        self.authored_vertical
            .as_ref()
            .unwrap_or(&self.horizontal)
            .authored()
    }

    /// The corner at `index` in top-left, top-right, bottom-right, bottom-left order. The
    /// four-side assignment of `margin` supplies it: its top, right, bottom and left roles
    /// are the four corners in that order.
    fn corner(&self, index: usize) -> CssSidePair<CornerRadiusLength> {
        CssSidePair::from_parts(
            self.horizontal.assigned()[index].clone(),
            self.authored_vertical
                .as_ref()
                .map(|values| values.assigned()[index].clone()),
        )
    }
    /// Returns the checked top-left corner.
    pub fn top_left(&self) -> CssSidePair<CornerRadiusLength> {
        self.corner(0)
    }
    /// Returns the checked top-right corner.
    pub fn top_right(&self) -> CssSidePair<CornerRadiusLength> {
        self.corner(1)
    }
    /// Returns the checked bottom-right corner.
    pub fn bottom_right(&self) -> CssSidePair<CornerRadiusLength> {
        self.corner(2)
    }
    /// Returns the checked bottom-left corner.
    pub fn bottom_left(&self) -> CssSidePair<CornerRadiusLength> {
        self.corner(3)
    }

    /// Writes the radii of a shape function exactly as authored, sharing the shape's cumulative
    /// budget. CSS Shapes 1 §3.3 serializes shape functions "as per their individual grammars"
    /// and limits omitting components to computed values, so the `round` radii of `inset()`,
    /// `xywh()` and `rect()` are not shortened.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        self.horizontal
            .append_values(writer, self.horizontal.authored().len(), "", &mut |_| {})?;
        if let Some(vertical) = &self.authored_vertical {
            vertical.append_values(writer, vertical.authored().len(), " / ", &mut |_| {})?;
        }
        Ok(())
    }

    /// Writes the `border-radius` property value in its shortest form: each list follows the
    /// four-side rule, and a vertical list equal to the horizontal one is omitted with its slash.
    fn append_shortest(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        let horizontal = self.horizontal.shortest_count();
        self.horizontal
            .append_values(writer, horizontal, "", &mut |_| {})?;
        let Some(vertical) = &self.authored_vertical else {
            return Ok(());
        };
        let count = vertical.shortest_count();
        // Equal axes need no vertical list: Backgrounds 3 repeats the horizontal radii.
        let mut context = comparison_context();
        let equal_axes = count == horizontal
            && self.horizontal.authored()[..horizontal]
                .iter()
                .zip(&vertical.authored()[..count])
                .try_fold(true, |equal, (a, b)| -> Result<bool> {
                    Ok(equal && a.inverse_equal(b, &mut context)?)
                })?;
        vertical.append_values(
            writer,
            if equal_axes { 0 } else { count },
            " / ",
            &mut |_| {},
        )
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssBorderRadiusShorthand {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_shortest(writer)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBorderRadiusShorthand);
