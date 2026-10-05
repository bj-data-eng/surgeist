//! Checked authored individual rotation, before contextual transform execution.

use crate::{CssAngleValue, CssSpecifiedNumber, CssValueOrigin};

/// The authored individual rotation; `none` is distinct from an identity rotation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssRotate {
    None,
    Value(CssRotateValues),
}

/// An optional authored rotation axis, retaining exact symbolic vector operands.
/// A zero vector is valid syntax; canonical output preserves its identity effect.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssRotateAxis {
    X,
    Y,
    Z,
    Vector([CssSpecifiedNumber; 3]),
}

impl PartialEq for CssRotateAxis {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::X, Self::X) | (Self::Y, Self::Y) | (Self::Z, Self::Z) => true,
            (Self::Vector(left), Self::Vector(right)) => left
                .iter()
                .zip(right)
                .all(|(left, right)| left.structural_eq(right)),
            _ => false,
        }
    }
}

/// One checked angle and optional axis for the individual `rotate` property.
/// Unlike transform-function angles, this property's angle does not admit bare zero.
#[derive(Clone, Debug)]
pub struct CssRotateValues {
    angle: CssAngleValue,
    axis: Option<CssRotateAxis>,
    keyword_axis_origin: Option<CssValueOrigin>,
}

impl PartialEq for CssRotateValues {
    fn eq(&self, other: &Self) -> bool {
        self.angle.structural_eq(&other.angle) && self.axis == other.axis
    }
}

impl CssRotateValues {
    /// Retains checked operands without normalizing or resolving the axis.
    /// Keyword axes receive programmatic provenance; vector operands keep their origins.
    pub fn new(angle: CssAngleValue, axis: Option<CssRotateAxis>) -> Self {
        let keyword_axis_origin = match &axis {
            Some(CssRotateAxis::X | CssRotateAxis::Y | CssRotateAxis::Z) => {
                Some(CssValueOrigin::Programmatic)
            }
            Some(CssRotateAxis::Vector(_)) | None => None,
        };
        Self {
            angle,
            axis,
            keyword_axis_origin,
        }
    }

    pub(crate) fn from_keyword_axis(
        angle: CssAngleValue,
        axis: CssRotateAxis,
        origin: CssValueOrigin,
    ) -> Self {
        assert!(matches!(
            axis,
            CssRotateAxis::X | CssRotateAxis::Y | CssRotateAxis::Z
        ));
        Self {
            angle,
            axis: Some(axis),
            keyword_axis_origin: Some(origin),
        }
    }

    /// Borrows the original checked angle, including literal or math provenance.
    #[must_use]
    pub const fn angle(&self) -> &CssAngleValue {
        &self.angle
    }

    /// Borrows the authored axis; absence denotes the implicit positive z axis.
    #[must_use]
    pub const fn axis(&self) -> Option<&CssRotateAxis> {
        self.axis.as_ref()
    }

    /// Borrows keyword-axis provenance; omitted and vector axes return `None`.
    /// Vector origins remain available on each exact numeric operand.
    #[must_use]
    pub const fn keyword_axis_origin(&self) -> Option<&CssValueOrigin> {
        self.keyword_axis_origin.as_ref()
    }
}

use crate::{
    CssAngleLiteral, CssAngleUnit, CssComponentValueRef, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits,
    CssValueTokenRef, exact_decimal::LexicalDecimal, numeric::DeferredNumericProjection,
    specified_rule_serialization::SpecifiedRuleWriter,
    specified_serialization::SpecifiedSerializationContext,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

impl CssRotate {
    /// Serializes the individual rotation in canonical axis-before-angle order.
    /// `none` remains distinct from a proved identity, which emits `0deg`.
    /// Directed parallel axes reduce to keywords (or an omitted z axis), with
    /// angle sign inversion for negative vectors. Authored operands are unchanged.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses one cumulative input, projection, and byte context for every child.
    /// Ordinary axis classification is exact; calculation classification uses
    /// the existing numeric projection arithmetic before formatting. Surviving
    /// numbers retain six-place rounding and can approximate the direction.
    /// A general vector's proved nonzero component rounding to zero returns
    /// `UnrepresentableValue`, an owning capability policy rather than invalid CSS.
    /// Failure returns no partial CSS and preserves operands and provenance.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        // One rotation aggregate; an explicit axis is a separate authored child.
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        let values = match self {
            Self::None => return writer.append("none"),
            Self::Value(values) => values,
        };
        let axis = match values.axis() {
            None => PreparedAxis::Parallel {
                keyword: None,
                negate: false,
            },
            Some(axis) => {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                match axis {
                    CssRotateAxis::X => PreparedAxis::Parallel {
                        keyword: Some("x"),
                        negate: false,
                    },
                    CssRotateAxis::Y => PreparedAxis::Parallel {
                        keyword: Some("y"),
                        negate: false,
                    },
                    CssRotateAxis::Z => PreparedAxis::Parallel {
                        keyword: None,
                        negate: false,
                    },
                    CssRotateAxis::Vector(vector) => PreparedAxis::Vector([
                        PreparedNumber::new(&vector[0], &mut writer.context)?,
                        PreparedNumber::new(&vector[1], &mut writer.context)?,
                        PreparedNumber::new(&vector[2], &mut writer.context)?,
                    ]),
                }
            }
        };
        let angle = PreparedAngle::new(values.angle(), &mut writer.context)?;
        let axis = axis.reduce();
        // All children have been projected once, including those omitted here.
        if matches!(axis, PreparedAxis::Zero) || angle.is_identity() {
            writer.context.charge_projection(1)?;
            return writer.append("0deg");
        }
        match axis {
            PreparedAxis::Zero => unreachable!("identity emitted above"),
            PreparedAxis::Parallel { keyword, negate } => {
                if let Some(keyword) = keyword {
                    writer.append(keyword)?;
                    writer.append(" ")?;
                }
                angle.append(writer, negate)
            }
            PreparedAxis::Vector(vector) => {
                for number in vector {
                    number.append(writer)?;
                    writer.append(" ")?;
                }
                angle.append(writer, false)
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum NumberClass {
    Zero,
    Positive,
    Negative,
    Unknown,
}
impl NumberClass {
    fn projected(value: Option<f64>) -> Self {
        match value {
            Some(value) if value.is_finite() => {
                if value == 0.0 {
                    Self::Zero
                } else if value < 0.0 {
                    Self::Negative
                } else {
                    Self::Positive
                }
            }
            _ => Self::Unknown,
        }
    }
    fn nonzero(self) -> bool {
        matches!(self, Self::Positive | Self::Negative)
    }
}

enum PreparedNumber<'a> {
    Literal { text: &'a str, class: NumberClass },
    Calculation(DeferredNumericProjection),
}
impl<'a> PreparedNumber<'a> {
    fn new(
        number: &'a CssSpecifiedNumber,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<Self> {
        if let Some(component) = number.literal_component() {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            let CssComponentValueRef::Token(CssValueTokenRef::Number(token)) = component.view()
            else {
                unreachable!("checked number literal");
            };
            let text = token.representation();
            let decimal = LexicalDecimal::new(text);
            let class = if decimal.len == 0 {
                NumberClass::Zero
            } else if decimal.negative {
                NumberClass::Negative
            } else {
                NumberClass::Positive
            };
            Ok(Self::Literal { text, class })
        } else {
            crate::numeric::prepare_calculation_specified(
                crate::numeric::SpecifiedCalculationRef::Number(
                    number.calculation().expect("checked number calculation"),
                ),
                context,
            )
            .map(Self::Calculation)
        }
    }
    fn class(&self) -> NumberClass {
        match self {
            Self::Literal { class, .. } => *class,
            Self::Calculation(value) => NumberClass::projected(value.outcome().scalar_value),
        }
    }
    fn append(self, writer: &mut SpecifiedRuleWriter) -> SerializationResult<()> {
        // Suppressed children already consumed input and projection work above.
        if writer.context.output_suppressed() {
            return Ok(());
        }
        match self {
            Self::Literal { text, class } => {
                let number = crate::numeric_formatting::format_css_number_with_sign(
                    text,
                    0,
                    false,
                    writer.context.remaining_bytes(),
                )?;
                if class.nonzero() && number.is_zero {
                    return Err(unrepresentable());
                }
                writer.append(&number.text)
            }
            Self::Calculation(value) => {
                if let Some(scalar) = value.outcome().scalar_value
                    && scalar.is_finite()
                    && scalar != 0.0
                    && crate::numeric_formatting::projected_number_rounds_to_zero(scalar)
                {
                    return Err(unrepresentable());
                }
                value.append(&mut writer.context, &mut writer.css, false)
            }
        }
    }
}

fn unrepresentable() -> CssSpecifiedValueSerializationError {
    CssSpecifiedValueSerializationError::new(
        CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
    )
}

enum PreparedAxis<'a> {
    Zero,
    Parallel {
        keyword: Option<&'static str>,
        negate: bool,
    },
    Vector([PreparedNumber<'a>; 3]),
}
impl PreparedAxis<'_> {
    fn reduce(self) -> Self {
        let Self::Vector(vector) = self else {
            return self;
        };
        let classes = vector.each_ref().map(PreparedNumber::class);
        if classes.iter().all(|class| *class == NumberClass::Zero) {
            return Self::Zero;
        }
        for index in 0..3 {
            if classes[index].nonzero()
                && (0..3).all(|other| other == index || classes[other] == NumberClass::Zero)
            {
                return Self::Parallel {
                    keyword: match index {
                        0 => Some("x"),
                        1 => Some("y"),
                        _ => None,
                    },
                    negate: classes[index] == NumberClass::Negative,
                };
            }
        }
        Self::Vector(vector)
    }
}

enum PreparedAngle<'a> {
    Literal(&'a CssAngleLiteral),
    Zero,
    Calculation(DeferredNumericProjection),
}
impl<'a> PreparedAngle<'a> {
    fn from_transform_angle(
        angle: &'a crate::CssAngleOrZero,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<Self> {
        match angle {
            crate::CssAngleOrZero::Angle(value) => Self::new(value, context),
            crate::CssAngleOrZero::Zero(_) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                Ok(Self::Zero)
            }
        }
    }

    fn new(
        angle: &'a CssAngleValue,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<Self> {
        if let Some(literal) = angle.literal() {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            Ok(Self::Literal(literal))
        } else {
            crate::numeric::prepare_calculation_specified(
                crate::numeric::SpecifiedCalculationRef::Angle(
                    angle.calculation().expect("checked angle calculation"),
                ),
                context,
            )
            .map(Self::Calculation)
        }
    }
    fn is_identity(&self) -> bool {
        match self {
            Self::Zero => true,
            Self::Literal(literal) => {
                let decimal = LexicalDecimal::new(literal.numeric().representation());
                match literal.unit() {
                    CssAngleUnit::Degrees => decimal.is_angle_full_turn(360),
                    CssAngleUnit::Gradians => decimal.is_angle_full_turn(400),
                    CssAngleUnit::Turns => decimal.is_angle_full_turn(1),
                    CssAngleUnit::Radians => decimal.len == 0,
                }
            }
            Self::Calculation(value) => value
                .outcome()
                .scalar_value
                .is_some_and(|value| value.is_finite() && value % 360.0 == 0.0),
        }
    }
    fn append(self, writer: &mut SpecifiedRuleWriter, negate: bool) -> SerializationResult<()> {
        match self {
            Self::Zero => writer.append("0"),
            Self::Literal(literal) => {
                if negate {
                    writer.context.charge_projection(1)?;
                }
                if writer.context.output_suppressed() {
                    return Ok(());
                }
                let suffix = crate::angle::suffix(literal.unit());
                let coefficient_limit = writer
                    .context
                    .remaining_bytes()
                    .checked_sub(suffix.len())
                    .ok_or_else(|| {
                        CssSpecifiedValueSerializationError::new(
                            CssSpecifiedValueSerializationErrorKind::ByteLimit,
                        )
                    })?;
                let number = crate::numeric_formatting::format_css_number_with_sign(
                    literal.numeric().representation(),
                    0,
                    negate,
                    coefficient_limit,
                )?;
                writer.append(&number.text)?;
                writer.append(suffix)
            }
            Self::Calculation(value) => value.append(&mut writer.context, &mut writer.css, negate),
        }
    }
}

impl crate::CssTransformRotate3d {
    /// The rotation owner prepares all operands once and emits mandatory function
    /// arguments. Parallel/identity reduction shares the individual rotation's
    /// policy; general vectors retain its documented six-place capability guard.
    pub(crate) fn append_rotation_arguments_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        let axis = PreparedAxis::Vector([
            PreparedNumber::new(self.x(), &mut writer.context)?,
            PreparedNumber::new(self.y(), &mut writer.context)?,
            PreparedNumber::new(self.z(), &mut writer.context)?,
        ]);
        let angle = PreparedAngle::from_transform_angle(self.angle(), &mut writer.context)?;
        let axis = axis.reduce();
        if matches!(axis, PreparedAxis::Zero) || angle.is_identity() {
            // Three replacement axis components and one identity angle.
            writer.context.charge_projection(4)?;
            return writer.append("0, 0, 1, 0deg");
        }
        match axis {
            PreparedAxis::Zero => unreachable!("identity emitted above"),
            PreparedAxis::Parallel { keyword, negate } => {
                writer.context.charge_projection(3)?;
                writer.append(match keyword {
                    Some("x") => "1, 0, 0, ",
                    Some("y") => "0, 1, 0, ",
                    _ => "0, 0, 1, ",
                })?;
                angle.append(writer, negate)
            }
            PreparedAxis::Vector(vector) => {
                for number in vector {
                    number.append(writer)?;
                    writer.append(", ")?;
                }
                angle.append(writer, false)
            }
        }
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;
    use crate::{CssComponentValue, CssComponentValues};
    type Limits = CssSpecifiedValueSerializationLimits;
    type Kind = CssSpecifiedValueSerializationErrorKind;

    fn number(text: &str) -> CssSpecifiedNumber {
        CssSpecifiedNumber::try_from_component(CssComponentValue::try_number(text).unwrap())
            .unwrap()
    }
    fn rotation(axis: [&str; 3], angle: CssAngleValue) -> CssRotate {
        CssRotate::Value(CssRotateValues::new(
            angle,
            Some(CssRotateAxis::Vector(axis.map(number))),
        ))
    }
    fn angle(text: &str) -> CssAngleValue {
        CssAngleValue::from_literal(CssAngleLiteral::try_new(text, CssAngleUnit::Degrees).unwrap())
    }
    fn math_angle() -> CssAngleValue {
        let components: CssComponentValues =
            crate::parse_component_values("calc(15deg + 15deg)").unwrap();
        CssAngleValue::try_from_calculation(
            crate::CssAngleCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn successive_rotations_share_all_three_budgets() {
        let x = rotation(["2", "0", "0"], angle("45"));
        let y = rotation(["0", "2", "0"], angle("30"));
        let mut writer = SpecifiedRuleWriter::new(Limits::new(12, 12, 14));
        x.append_to_rule_writer(&mut writer).unwrap();
        y.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "x 45degy 30deg");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(writer.append(" ").unwrap_err().kind(), Kind::ByteLimit);
        for (limits, kind) in [
            (Limits::new(11, 12, 14), Kind::InputNodeLimit),
            (Limits::new(12, 11, 14), Kind::ProjectionNodeLimit),
            (Limits::new(12, 12, 13), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            x.append_to_rule_writer(&mut writer).unwrap();
            assert_eq!(
                y.append_to_rule_writer(&mut writer).unwrap_err().kind(),
                kind
            );
        }
    }

    #[test]
    fn existing_consumption_is_not_reset_by_the_leaf() {
        let value = rotation(["2", "0", "0"], angle("45"));
        let mut writer = SpecifiedRuleWriter::new(Limits::new(7, 8, 10));
        writer.context.charge_input(1).unwrap();
        writer.context.charge_projection(2).unwrap();
        writer.append("pre").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "prex 45deg");
        for (limits, kind) in [
            (Limits::new(6, 8, 10), Kind::InputNodeLimit),
            (Limits::new(7, 7, 10), Kind::ProjectionNodeLimit),
            (Limits::new(7, 8, 9), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer.context.charge_input(1).unwrap();
            writer.context.charge_projection(2).unwrap();
            writer.append("pre").unwrap();
            assert_eq!(
                value.append_to_rule_writer(&mut writer).unwrap_err().kind(),
                kind
            );
        }
    }

    #[test]
    fn discarded_general_vector_consumes_work_without_formatting_huge_children() {
        let value = rotation(["1e400", "2", "0"], angle("45"));
        let mut writer = SpecifiedRuleWriter::new(Limits::new(7, 7, 4));
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        CssRotate::None.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "none");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
    }

    #[test]
    fn suppression_is_restored_after_a_nested_rotate_resource_error() {
        let value = rotation(["2", "0", "0"], angle("45"));
        let mut writer = SpecifiedRuleWriter::new(Limits::new(5, 6, 7));
        assert_eq!(
            writer
                .without_output(|writer| value.append_to_rule_writer(writer))
                .unwrap_err()
                .kind(),
            Kind::InputNodeLimit
        );
        assert!(!writer.context.output_suppressed());
        writer.append("visible").unwrap();
        assert_eq!(writer.css, "visible");
    }

    #[test]
    fn zero_axis_angle_math_is_visited_once_in_an_already_used_context() {
        // Three literal vector leaves, one Calc node, one Sum node and two
        // angle leaves: 9 input nodes including rotate and axis aggregates.
        // Literal projections plus three angle scalar nodes and identity: 9.
        let value = rotation(["0", "0", "0"], math_angle());
        let mut writer = SpecifiedRuleWriter::new(Limits::new(11, 12, 6));
        writer.context.charge_input(2).unwrap();
        writer.context.charge_projection(3).unwrap();
        writer.append("p:").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "p:0deg");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
    }

    #[test]
    fn negative_axis_angle_math_charges_one_derived_negation_without_retraversal() {
        let value = rotation(["-2", "0", "0"], math_angle());
        let mut writer = SpecifiedRuleWriter::new(Limits::new(9, 9, 14));
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "x calc(-30deg)");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(9, 8, 14))
                .unwrap_err()
                .kind(),
            Kind::ProjectionNodeLimit
        );
    }
}
