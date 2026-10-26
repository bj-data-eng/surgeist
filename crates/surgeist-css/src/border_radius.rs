//! Exact authored corner radii before box geometry resolution.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssLengthPercentage, CssNonNegative, CssNumeric, CssSerializationError};

type Result<T> = std::result::Result<T, CssSerializationError>;

fn append_scalar(
    value: &CssNumeric<CssLengthPercentage, CssNonNegative>,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> Result<()> {
    let captured = value.capture_specified(context)?;
    context.append(output, &captured)
}

/// Checked horizontal and optional authored vertical radii for one corner.
///
/// Missing vertical radius repeats the horizontal radius. Both numeric origins
/// remain inspectable through the borrowed scalar values. Serialization charges
/// one aggregate node plus each authored scalar under one cumulative budget.
///
/// # Specified serialization
///
/// Serializes the authored corner without resolving box geometry.
#[derive(Clone, Debug)]
pub struct CssCornerRadiusValue {
    horizontal: CssNumeric<CssLengthPercentage, CssNonNegative>,
    authored_vertical: Option<CssNumeric<CssLengthPercentage, CssNonNegative>>,
}

impl CssCornerRadiusValue {
    /// Retains the checked horizontal radius and optional authored vertical radius.
    #[must_use]
    pub fn new(
        horizontal: CssNumeric<CssLengthPercentage, CssNonNegative>,
        authored_vertical: Option<CssNumeric<CssLengthPercentage, CssNonNegative>>,
    ) -> Self {
        Self {
            horizontal,
            authored_vertical,
        }
    }

    /// Borrows the horizontal radius.
    pub fn horizontal(&self) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        &self.horizontal
    }
    /// Borrows the explicitly authored vertical radius, if present.
    pub fn authored_vertical(&self) -> Option<&CssNumeric<CssLengthPercentage, CssNonNegative>> {
        self.authored_vertical.as_ref()
    }
    /// Borrows the effective vertical radius, repeating horizontal when omitted.
    pub fn vertical(&self) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        self.authored_vertical.as_ref().unwrap_or(&self.horizontal)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssCornerRadiusValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;

        context.charge_nodes(1)?;
        append_scalar(&self.horizontal, context, output)?;
        if let Some(vertical) = &self.authored_vertical {
            context.append(output, " ")?;
            append_scalar(vertical, context, output)?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssCornerRadiusValue);

impl PartialEq for CssCornerRadiusValue {
    fn eq(&self, other: &Self) -> bool {
        self.horizontal == other.horizontal
            && match (&self.authored_vertical, &other.authored_vertical) {
                (None, None) => true,
                (Some(left), Some(right)) => left == right,
                _ => false,
            }
    }
}
impl Eq for CssCornerRadiusValue {}

/// One to four horizontal radii, optionally followed by one to four vertical radii.
///
/// Named corners use top-left, top-right, bottom-right, bottom-left order.
/// Serialization charges one aggregate node plus each authored scalar; the
/// slash is a byte separator, not an extra node.
#[derive(Clone, Debug)]
pub struct CssBorderRadiusShorthand {
    horizontal: Box<[CssNumeric<CssLengthPercentage, CssNonNegative>]>,
    authored_vertical: Option<Box<[CssNumeric<CssLengthPercentage, CssNonNegative>]>>,
}

impl CssBorderRadiusShorthand {
    /// Rejects an empty or overlong horizontal or supplied vertical list.
    #[must_use]
    pub fn try_new(
        horizontal: Vec<CssNumeric<CssLengthPercentage, CssNonNegative>>,
        authored_vertical: Option<Vec<CssNumeric<CssLengthPercentage, CssNonNegative>>>,
    ) -> Option<Self> {
        if !(1..=4).contains(&horizontal.len())
            || authored_vertical
                .as_ref()
                .is_some_and(|values| !(1..=4).contains(&values.len()))
        {
            return None;
        }
        Some(Self {
            horizontal: horizontal.into_boxed_slice(),
            authored_vertical: authored_vertical.map(Vec::into_boxed_slice),
        })
    }
    /// Borrows the one to four authored horizontal values.
    pub fn horizontal_values(&self) -> &[CssNumeric<CssLengthPercentage, CssNonNegative>] {
        &self.horizontal
    }
    /// Borrows the explicitly authored vertical list, if a slash appeared.
    pub fn authored_vertical_values(
        &self,
    ) -> Option<&[CssNumeric<CssLengthPercentage, CssNonNegative>]> {
        self.authored_vertical.as_deref()
    }
    /// Borrows the effective vertical list, repeating the horizontal list without a slash.
    pub fn vertical_values(&self) -> &[CssNumeric<CssLengthPercentage, CssNonNegative>] {
        self.authored_vertical
            .as_deref()
            .unwrap_or(&self.horizontal)
    }

    fn assigned(
        values: &[CssNumeric<CssLengthPercentage, CssNonNegative>],
        corner: usize,
    ) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        let index = match (values.len(), corner) {
            (1, _) => 0,
            (2, 0 | 2) => 0,
            (2, _) => 1,
            (3, 0) => 0,
            (3, 2) => 2,
            (3, _) => 1,
            (4, index) => index,
            _ => unreachable!("checked corner and arity"),
        };
        &values[index]
    }
    fn corner(&self, index: usize) -> CssCornerRadiusValue {
        CssCornerRadiusValue::new(
            Self::assigned(&self.horizontal, index).clone(),
            self.authored_vertical
                .as_ref()
                .map(|values| Self::assigned(values, index).clone()),
        )
    }
    /// Returns the checked top-left corner.
    pub fn top_left(&self) -> CssCornerRadiusValue {
        self.corner(0)
    }
    /// Returns the checked top-right corner.
    pub fn top_right(&self) -> CssCornerRadiusValue {
        self.corner(1)
    }
    /// Returns the checked bottom-right corner.
    pub fn bottom_right(&self) -> CssCornerRadiusValue {
        self.corner(2)
    }
    /// Returns the checked bottom-left corner.
    pub fn bottom_left(&self) -> CssCornerRadiusValue {
        self.corner(3)
    }

    /// Shares the owning shape's cumulative budget without changing authored radii.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        for (index, value) in self.horizontal.iter().enumerate() {
            writer.source_member(index, |writer| {
                if index > 0 {
                    writer.append(" ")?;
                }
                append_scalar(value, &mut writer.context, &mut writer.css)
            })?;
        }
        if let Some(vertical) = &self.authored_vertical {
            for (index, value) in vertical.iter().enumerate() {
                writer.source_member(index, |writer| {
                    writer.append(if index == 0 { " / " } else { " " })?;
                    append_scalar(value, &mut writer.context, &mut writer.css)
                })?;
            }
        }
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssBorderRadiusShorthand {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBorderRadiusShorthand);

impl PartialEq for CssBorderRadiusShorthand {
    fn eq(&self, other: &Self) -> bool {
        fn equal(
            left: &[CssNumeric<CssLengthPercentage, CssNonNegative>],
            right: &[CssNumeric<CssLengthPercentage, CssNonNegative>],
        ) -> bool {
            left.len() == right.len() && left.iter().zip(right).all(|(a, b)| a == b)
        }
        equal(&self.horizontal, &other.horizontal)
            && match (&self.authored_vertical, &other.authored_vertical) {
                (None, None) => true,
                (Some(left), Some(right)) => equal(left, right),
                _ => false,
            }
    }
}
impl Eq for CssBorderRadiusShorthand {}
