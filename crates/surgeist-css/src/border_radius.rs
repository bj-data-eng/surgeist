//! Exact authored corner radii before box geometry resolution.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssSpecifiedNonNegativeLengthPercentage, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn append_scalar(
    value: &CssSpecifiedNonNegativeLengthPercentage,
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
#[derive(Clone, Debug)]
pub struct CssCornerRadiusValue {
    horizontal: CssSpecifiedNonNegativeLengthPercentage,
    authored_vertical: Option<CssSpecifiedNonNegativeLengthPercentage>,
}

impl CssCornerRadiusValue {
    /// Retains the checked horizontal radius and optional authored vertical radius.
    #[must_use]
    pub fn new(
        horizontal: CssSpecifiedNonNegativeLengthPercentage,
        authored_vertical: Option<CssSpecifiedNonNegativeLengthPercentage>,
    ) -> Self {
        Self {
            horizontal,
            authored_vertical,
        }
    }

    /// Borrows the horizontal radius.
    pub fn horizontal(&self) -> &CssSpecifiedNonNegativeLengthPercentage {
        &self.horizontal
    }
    /// Borrows the explicitly authored vertical radius, if present.
    pub fn authored_vertical(&self) -> Option<&CssSpecifiedNonNegativeLengthPercentage> {
        self.authored_vertical.as_ref()
    }
    /// Borrows the effective vertical radius, repeating horizontal when omitted.
    pub fn vertical(&self) -> &CssSpecifiedNonNegativeLengthPercentage {
        self.authored_vertical.as_ref().unwrap_or(&self.horizontal)
    }

    /// Serializes the authored corner without resolving box geometry.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Serializes all authored components under one resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;

        context.charge_input(1)?;
        context.charge_projection(1)?;
        append_scalar(&self.horizontal, context, output)?;
        if let Some(vertical) = &self.authored_vertical {
            context.append(output, " ")?;
            append_scalar(vertical, context, output)?;
        }
        Ok(())
    }
}

impl PartialEq for CssCornerRadiusValue {
    fn eq(&self, other: &Self) -> bool {
        self.horizontal.structural_eq(&other.horizontal)
            && match (&self.authored_vertical, &other.authored_vertical) {
                (None, None) => true,
                (Some(left), Some(right)) => left.structural_eq(right),
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
    horizontal: Box<[CssSpecifiedNonNegativeLengthPercentage]>,
    authored_vertical: Option<Box<[CssSpecifiedNonNegativeLengthPercentage]>>,
}

impl CssBorderRadiusShorthand {
    /// Rejects an empty or overlong horizontal or supplied vertical list.
    #[must_use]
    pub fn try_new(
        horizontal: Vec<CssSpecifiedNonNegativeLengthPercentage>,
        authored_vertical: Option<Vec<CssSpecifiedNonNegativeLengthPercentage>>,
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
    pub fn horizontal_values(&self) -> &[CssSpecifiedNonNegativeLengthPercentage] {
        &self.horizontal
    }
    /// Borrows the explicitly authored vertical list, if a slash appeared.
    pub fn authored_vertical_values(&self) -> Option<&[CssSpecifiedNonNegativeLengthPercentage]> {
        self.authored_vertical.as_deref()
    }
    /// Borrows the effective vertical list, repeating the horizontal list without a slash.
    pub fn vertical_values(&self) -> &[CssSpecifiedNonNegativeLengthPercentage] {
        self.authored_vertical
            .as_deref()
            .unwrap_or(&self.horizontal)
    }

    fn assigned(
        values: &[CssSpecifiedNonNegativeLengthPercentage],
        corner: usize,
    ) -> &CssSpecifiedNonNegativeLengthPercentage {
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

    /// Serializes only authored values with default resource limits.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Serializes all authored values under one cumulative resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }

    /// Shares the owning shape's cumulative budget without changing authored radii.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        for (index, value) in self.horizontal.iter().enumerate() {
            if index > 0 {
                writer.append(" ")?;
            }
            append_scalar(value, &mut writer.context, &mut writer.css)?;
        }
        if let Some(vertical) = &self.authored_vertical {
            writer.append(" / ")?;
            for (index, value) in vertical.iter().enumerate() {
                if index > 0 {
                    writer.append(" ")?;
                }
                append_scalar(value, &mut writer.context, &mut writer.css)?;
            }
        }
        Ok(())
    }
}

impl PartialEq for CssBorderRadiusShorthand {
    fn eq(&self, other: &Self) -> bool {
        fn equal(
            left: &[CssSpecifiedNonNegativeLengthPercentage],
            right: &[CssSpecifiedNonNegativeLengthPercentage],
        ) -> bool {
            left.len() == right.len() && left.iter().zip(right).all(|(a, b)| a.structural_eq(b))
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
