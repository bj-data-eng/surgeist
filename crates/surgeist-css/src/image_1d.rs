//! Reusable Images4 one-dimensional stripes, imported by selected UI4 Outline.
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssColor, CssComponentValue, CssComponentValueError, CssComponentValueRef,
    CssLengthPercentageCalculation, CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssSpecifiedFlex, CssSpecifiedNonNegativeLengthPercentage, CssValueOrigin, CssValueTokenRef,
};

/// A checked stripe thickness length-percentage. Literal percentages are 0..100;
/// lengths are nonnegative. Calculations retain the shared deferred range phase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssStripeLengthPercentage {
    value: CssSpecifiedNonNegativeLengthPercentage,
}
impl CssStripeLengthPercentage {
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssNumericConstructionError> {
        if let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) = component.view()
            && !crate::exact_decimal::LexicalDecimal::new(number.representation())
                .in_percentage_range()
        {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component)
            .map(|value| Self { value })
    }
    pub fn try_from_calculation(
        calculation: CssLengthPercentageCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(origin) = calculation.components().first_implicit_origin() {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Self::from_parser_calculation(calculation)
    }
    pub(crate) fn from_parser_calculation(
        calculation: CssLengthPercentageCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        let root = crate::specified_numeric::significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(calculation)
            .map(|value| Self { value })
    }
    pub const fn value(&self) -> &CssSpecifiedNonNegativeLengthPercentage {
        &self.value
    }
    pub fn origin(&self) -> &CssValueOrigin {
        self.value.origin()
    }
}
/// Distinct unrestricted flex and range-constrained length-percentage alternatives.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssStripeThickness {
    LengthPercentage(CssStripeLengthPercentage),
    Flex(CssSpecifiedFlex),
}
impl PartialEq for CssStripeThickness {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.value.structural_eq(&right.value)
            }
            (Self::Flex(left), Self::Flex(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}
impl Eq for CssStripeThickness {}
impl CssStripeThickness {
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::LengthPercentage(value) => value.origin(),
            Self::Flex(value) => value.origin(),
        }
    }
    pub(crate) fn first_implicit_origin(&self) -> Option<&CssValueOrigin> {
        match self {
            Self::LengthPercentage(value) => value
                .value
                .calculation()
                .and_then(|value| value.components().first_implicit_origin()),
            Self::Flex(value) => value
                .calculation()
                .and_then(|value| value.components().first_implicit_origin()),
        }
    }
    pub(crate) fn nesting_depth(&self) -> u32 {
        match self {
            Self::LengthPercentage(value) => value
                .value
                .calculation()
                .map_or(0, |v| v.components().nesting_depth()),
            Self::Flex(value) => value
                .calculation()
                .map_or(0, |v| v.components().nesting_depth()),
        }
    }
    fn is_default(&self) -> bool {
        let Self::Flex(value) = self else {
            return false;
        };
        let Some(component) = value.literal_component() else {
            return false;
        };
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) =
            component.view()
        else {
            unreachable!("checked flex")
        };
        crate::exact_decimal::LexicalDecimal::new(number.representation())
            .value_eq(&crate::exact_decimal::LexicalDecimal::new("1"))
    }
}
/// Typed checked construction failures retaining original component or numeric provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImage1DConstructionError {
    EmptyStripes,
    Component(CssComponentValueError),
    Numeric(CssNumericConstructionError),
    Grammar(crate::CssPropertyValueParseError),
    RecoveredComponent(CssValueOrigin),
    NestingLimit,
}
impl std::fmt::Display for CssImage1DConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CSS image-1D construction {self:?}")
    }
}
impl std::error::Error for CssImage1DConstructionError {}
/// A required color and optional thickness, retaining complete original color syntax.
#[derive(Clone, Debug)]
pub struct CssStripe {
    color: CssColor,
    color_component: CssComponentValue,
    thickness: Option<CssStripeThickness>,
}
impl PartialEq for CssStripe {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color && self.thickness == other.thickness
    }
}
impl CssStripe {
    /// Checks one complete color component and every retained child's closure.
    pub fn try_new(
        color: CssComponentValue,
        thickness: Option<CssStripeThickness>,
    ) -> Result<Self, CssImage1DConstructionError> {
        let values = crate::CssComponentValues::try_new(vec![color.clone()])
            .map_err(CssImage1DConstructionError::Component)?;
        if let Some(origin) = values.first_implicit_origin() {
            return Err(CssImage1DConstructionError::RecoveredComponent(
                origin.clone(),
            ));
        }
        if let Some(origin) = thickness
            .as_ref()
            .and_then(CssStripeThickness::first_implicit_origin)
        {
            return Err(CssImage1DConstructionError::RecoveredComponent(
                origin.clone(),
            ));
        }
        Self::check_component_depth(&values, thickness.as_ref())?;
        let color_value = crate::parser::image_1d::checked_color(&values)?;
        Ok(Self {
            color: color_value,
            color_component: color,
            thickness,
        })
    }
    pub(crate) fn from_parser(
        color: CssColor,
        color_component: CssComponentValue,
        thickness: Option<CssStripeThickness>,
    ) -> Self {
        Self {
            color,
            color_component,
            thickness,
        }
    }
    fn check_depth(&self) -> Result<(), CssImage1DConstructionError> {
        let color = crate::CssComponentValues::try_new(vec![self.color_component.clone()])
            .map_err(CssImage1DConstructionError::Component)?;
        Self::check_component_depth(&color, self.thickness.as_ref())
    }
    fn check_component_depth(
        color: &crate::CssComponentValues,
        thickness: Option<&CssStripeThickness>,
    ) -> Result<(), CssImage1DConstructionError> {
        let depth = color
            .nesting_depth()
            .max(thickness.map_or(0, CssStripeThickness::nesting_depth));
        if depth >= crate::STRUCTURAL_NESTING_LIMIT {
            return Err(CssImage1DConstructionError::NestingLimit);
        }
        Ok(())
    }
    pub const fn color(&self) -> &CssColor {
        &self.color
    }
    pub const fn color_component(&self) -> &CssComponentValue {
        &self.color_component
    }
    pub const fn color_origin(&self) -> &CssValueOrigin {
        self.color_component.origin()
    }
    pub const fn thickness(&self) -> Option<&CssStripeThickness> {
        self.thickness.as_ref()
    }
}
/// Nonempty ordered stripes; omitted thickness remains absent and means 1fr downstream.
#[derive(Clone, Debug)]
pub struct CssStripes {
    stripes: Vec<CssStripe>,
    original: Option<CssComponentValue>,
}
impl PartialEq for CssStripes {
    fn eq(&self, other: &Self) -> bool {
        self.stripes == other.stripes
    }
}
impl CssStripes {
    pub fn try_new(stripes: Vec<CssStripe>) -> Result<Self, CssImage1DConstructionError> {
        if stripes.is_empty() {
            return Err(CssImage1DConstructionError::EmptyStripes);
        }
        for stripe in &stripes {
            stripe.check_depth()?;
            let values = crate::CssComponentValues::try_new(vec![stripe.color_component.clone()])
                .map_err(CssImage1DConstructionError::Component)?;
            if let Some(origin) = values.first_implicit_origin() {
                return Err(CssImage1DConstructionError::RecoveredComponent(
                    origin.clone(),
                ));
            }
            if let Some(origin) = stripe
                .thickness
                .as_ref()
                .and_then(CssStripeThickness::first_implicit_origin)
            {
                return Err(CssImage1DConstructionError::RecoveredComponent(
                    origin.clone(),
                ));
            }
        }
        Ok(Self {
            stripes,
            original: None,
        })
    }
    pub(crate) fn from_parser(stripes: Vec<CssStripe>, original: CssComponentValue) -> Self {
        Self {
            stripes,
            original: Some(original),
        }
    }
    pub fn stripes(&self) -> &[CssStripe] {
        &self.stripes
    }
    /// Borrows the parsed complete stripes function, including recovered closing provenance.
    pub const fn original_component(&self) -> Option<&CssComponentValue> {
        self.original.as_ref()
    }
    pub fn origin(&self) -> &CssValueOrigin {
        self.original
            .as_ref()
            .map_or(&CssValueOrigin::Programmatic, CssComponentValue::origin)
    }
}
/// A one-dimensional image. This is distinct from Color and the Images3 2D image domain.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssImage1D {
    Stripes(CssStripes),
}
impl CssImage1D {
    /// Checks the reusable image-1D grammar from closed original components.
    pub fn try_from_components(
        values: crate::CssComponentValues,
    ) -> Result<Self, CssImage1DConstructionError> {
        if let Some(origin) = values.first_implicit_origin() {
            return Err(CssImage1DConstructionError::RecoveredComponent(
                origin.clone(),
            ));
        }
        crate::parser::image_1d::checked_image(&values)
    }
    pub fn stripes(&self) -> &CssStripes {
        match self {
            Self::Stripes(value) => value,
        }
    }
}

type Output<T> = Result<T, crate::CssSpecifiedValueSerializationError>;
macro_rules! provider {
    ($ty:ty) => {
        impl $ty {
            pub fn serialize_specified(&self) -> Output<String> {
                self.serialize_specified_with_limits(
                    crate::CssSpecifiedValueSerializationLimits::default(),
                )
            }
            /// Visits the complete retained graph under cumulative budgets; failure is atomic.
            pub fn serialize_specified_with_limits(
                &self,
                limits: crate::CssSpecifiedValueSerializationLimits,
            ) -> Output<String> {
                let mut writer = SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }
        }
    };
}
provider!(CssImage1D);
provider!(CssStripes);
provider!(CssStripeThickness);
provider!(CssStripeLengthPercentage);
impl CssStripeLengthPercentage {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Output<()> {
        self.value.append_to_rule_writer(writer)
    }
}
impl CssStripeThickness {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Output<()> {
        match self {
            Self::LengthPercentage(value) => value.append_to_rule_writer(writer),
            Self::Flex(value) => value.append_to_rule_writer(writer),
        }
    }
}
impl CssImage1D {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Output<()> {
        self.stripes().append_to_rule_writer(writer)
    }
}
impl CssStripes {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Output<()> {
        writer.node()?;
        writer.append("stripes(")?;
        for (index, stripe) in self.stripes.iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            writer.node()?;
            stripe
                .color
                .append_specified(&mut writer.context, &mut writer.css)?;
            if let Some(thickness) = &stripe.thickness {
                if thickness.is_default() {
                    writer.without_output(|writer| thickness.append_to_rule_writer(writer))?;
                } else {
                    writer.append(" ")?;
                    thickness.append_to_rule_writer(writer)?;
                }
            }
        }
        writer.append(")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stripes_share_cumulative_work_for_default_and_math_thicknesses() {
        for (text, expected, input, projection) in [
            ("stripes(red 1fr)", "stripes(red)", 4, 4),
            (
                "stripes(red 2px, blue calc(1fr + 2fr))",
                "stripes(red 2px, blue calc(3fr))",
                10,
                9,
            ),
        ] {
            let value =
                CssImage1D::try_from_components(crate::parse_component_values(text).unwrap())
                    .unwrap();
            let before = value.stripes().original_component().unwrap().clone();
            crate::ui_serialization::provider_tests::shared(
                expected,
                input,
                projection,
                |writer| value.append_to_rule_writer(writer),
            );
            assert_eq!(value.stripes().original_component(), Some(&before));
            assert!(
                value
                    .stripes()
                    .stripes()
                    .iter()
                    .all(|stripe| stripe.thickness().is_some())
            );
        }
    }
}
