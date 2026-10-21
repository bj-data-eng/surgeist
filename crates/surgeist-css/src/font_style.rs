//! Exact authored Fonts 4 style values, before matching or computed angle ranges.
//! Ordinary emitted coefficients round to six fractional places (nearest, ties
//! away from zero); finite calculated text rounds the projected binary64 value with the
//! same policy. Retained coefficients/ranges and calculation arithmetic stay unchanged.

use crate::{
    CssAngleCalculation, CssAngleUnit, CssComponentValue, CssComponentValueRef,
    CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
};
use crate::{
    numeric_formatting::format_css_number, specified_serialization::SpecifiedSerializationContext,
};

type ConstructionResult<T> = Result<T, CssNumericConstructionError>;
type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// The first 1000 significant digits of π/2 = 1.5707963267948966192…, the
/// Fonts 4 oblique radian bound. π/2 is irrational, so this truncation lies
/// strictly below it and no shorter authored decimal can equal it.
const HALF_PI_DIGITS: &str = concat!(
    "1570796326794896619231321691639751442098584699687552910487472296153908203143104499314017412671058533",
    "9910740432566411533235469223047752911158626797040642405587251420513509692605527798223114744774651909",
    "8221440548783296672306423782411689339158263560095457282428346173017430522716332410669680363012457063",
    "6862293503303157794087440760460481414627045857682183946295180005665265274410233260692073475970755804",
    "7165286351828797959765460930586909663058965525592740372311899813747836759428763624456139690915059745",
    "6491683668122032832154301069747319761236859535108993047185138526960858814658837619233740923383470256",
    "6000284063572631780413892885671378894804586818589360734220450612476715073274792685525396139844629461",
    "7710099780560645109804320172090799068148873856549802593536056749999991864890249755298658664080481592",
    "9751222972767345415132126115412667234251763096559408550500156891937644329376660419071030858883457365",
    "1799126745214377734365579781431941176893796875978890928890266085613403306500963938305597954608210099",
);

/// A keyword shared by the `font-style` property and its font-face descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontStyleKeyword {
    Normal,
    Italic,
    Left,
    Right,
}

impl CssFontStyleKeyword {
    const fn as_css(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Italic => "italic",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// An authored oblique angle with exact literal spelling or symbolic angle math.
#[derive(Clone, Debug)]
pub struct CssFontObliqueAngle {
    value: ObliqueAngleValue,
}

#[derive(Clone, Debug)]
enum ObliqueAngleValue {
    Literal(Box<CssComponentValue>),
    Calculation(CssAngleCalculation),
}

impl PartialEq for CssFontObliqueAngle {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (ObliqueAngleValue::Literal(a), ObliqueAngleValue::Literal(b)) => {
                a.structural_eq_ignoring_origin(b)
            }
            (ObliqueAngleValue::Calculation(a), ObliqueAngleValue::Calculation(b)) => {
                a.expression.structural_eq(&b.expression)
            }
            _ => false,
        }
    }
}
impl Eq for CssFontObliqueAngle {}

impl CssFontObliqueAngle {
    /// Checks one literal angle against the inclusive Fonts 4 oblique range.
    /// Every unit compares its exact authored decimal with the quarter-turn
    /// bound: 90deg, 100grad, 0.25turn, or π/2 radians. The irrational radian
    /// bound is its first 1000 significant digits, so the
    /// decision is exact for every coefficient of at most that many
    /// significant digits; a longer coefficient that matches the whole prefix
    /// is rejected. The accepted component keeps its original spelling and
    /// source origin.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
            component.view()
        else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        };
        let spelling = number.representation();
        let Some(crate::CssNumericUnit::Angle(unit)) = crate::numeric::units::parse(unit) else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        };
        let in_range = match unit {
            CssAngleUnit::Degrees => {
                crate::exact_decimal::LexicalDecimal::new(spelling).absolute_at_most("9", 1)
            }
            CssAngleUnit::Gradians => {
                crate::exact_decimal::LexicalDecimal::new(spelling).absolute_at_most("1", 2)
            }
            CssAngleUnit::Turns => {
                crate::exact_decimal::LexicalDecimal::new(spelling).absolute_at_most("25", -2)
            }
            CssAngleUnit::Radians => crate::exact_decimal::LexicalDecimal::new(spelling)
                .absolute_at_most(HALF_PI_DIGITS, -(HALF_PI_DIGITS.len() as i128 - 1)),
        };
        if !in_range {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        Ok(Self {
            value: ObliqueAngleValue::Literal(Box::new(component)),
        })
    }

    /// Retains already-admitted symbolic angle math and its recovery provenance;
    /// a bare dimension root reenters literal bounds.
    pub fn try_from_calculation(calculation: CssAngleCalculation) -> ConstructionResult<Self> {
        // This domain still retains recovered syntax from checked construction.
        crate::specified_numeric::admit_calculation(
            calculation,
            CssAngleCalculation::components,
            crate::specified_numeric::CalculationAdmission::Recovering,
            Self::try_from_component,
            |calculation| Self {
                value: ObliqueAngleValue::Calculation(calculation),
            },
        )
    }

    /// Borrows the original literal, including its unit and source origin.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            ObliqueAngleValue::Literal(component) => Some(component),
            ObliqueAngleValue::Calculation(_) => None,
        }
    }

    /// Borrows symbolic angle math, if authored.
    pub fn calculation(&self) -> Option<&CssAngleCalculation> {
        match &self.value {
            ObliqueAngleValue::Literal(_) => None,
            ObliqueAngleValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            ObliqueAngleValue::Literal(component) => component.origin(),
            ObliqueAngleValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the specified angle without computing font matching.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under the supplied resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
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
        self.append_specified(context, output)?;
        Ok(())
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match &self.value {
            ObliqueAngleValue::Literal(component) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                if context.output_suppressed() {
                    return Ok(());
                }
                let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
                    component.view()
                else {
                    unreachable!("checked literal angle")
                };
                let suffix = match crate::numeric::units::parse(unit) {
                    Some(unit @ crate::CssNumericUnit::Angle(_)) => {
                        crate::numeric::units::suffix(unit)
                    }
                    _ => unreachable!("checked angle unit"),
                };
                let coefficient_limit = context
                    .remaining_bytes()
                    .checked_sub(suffix.len())
                    .ok_or_else(|| {
                        CssSpecifiedValueSerializationError::new(
                            CssSpecifiedValueSerializationErrorKind::ByteLimit,
                        )
                    })?;
                let coefficient = format_css_number(number.representation(), 0, coefficient_limit)?;
                context.append(output, &coefficient)?;
                context.append(output, suffix)
            }
            ObliqueAngleValue::Calculation(calculation) => {
                let (captured, _) =
                    crate::numeric::capture_specified(&calculation.expression, context)?;
                context.append(output, &captured)
            }
        }
    }
}

/// An authored `font-style` property value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontStyle {
    Keyword(CssFontStyleKeyword),
    Oblique { angle: Option<CssFontObliqueAngle> },
}

/// One or two authored oblique descriptor endpoints, in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFaceObliqueRange {
    start: CssFontObliqueAngle,
    end: Option<CssFontObliqueAngle>,
}

impl CssFontFaceObliqueRange {
    /// Retains already-checked endpoints, including descending pairs.
    #[must_use]
    pub const fn new(start: CssFontObliqueAngle, end: Option<CssFontObliqueAngle>) -> Self {
        Self { start, end }
    }

    #[must_use]
    /// Borrows the required first authored endpoint.
    pub const fn start(&self) -> &CssFontObliqueAngle {
        &self.start
    }

    #[must_use]
    /// Borrows an explicitly authored second endpoint, if present.
    pub const fn end(&self) -> Option<&CssFontObliqueAngle> {
        self.end.as_ref()
    }

    /// Serializes the exact authored one- or two-endpoint range.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges both endpoints cumulatively; equal retained endpoints emit only the first.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
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
        self.append_specified(context, output)?;
        Ok(())
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        self.start.append_specified(context, output)?;
        if let Some(end) = &self.end {
            let omit_end = if context.output_suppressed() {
                true
            } else {
                self.start.specified_semantically_eq(end, context)?
            };
            let previous = context.replace_output_suppression(omit_end);
            let result = (|| {
                context.append(output, " ")?;
                end.append_specified(context, output)
            })();
            context.replace_output_suppression(previous);
            result?;
        }
        Ok(())
    }
}

/// An authored `@font-face` style descriptor, before font matching.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceStyle {
    Auto,
    Keyword(CssFontStyleKeyword),
    Oblique {
        range: Option<CssFontFaceObliqueRange>,
    },
}

fn append_keyword(
    keyword: CssFontStyleKeyword,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> SerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, keyword.as_css())
}

impl CssFontStyle {
    /// Serializes the authored keyword or optional oblique angle.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges the keyword and any explicit angle to one cumulative budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
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
        self.append_specified(context, output)?;
        Ok(())
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Keyword(keyword) => append_keyword(*keyword, context, output)?,
            Self::Oblique { angle } => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "oblique")?;
                if let Some(angle) = angle {
                    // Fonts 4 §2.4: a missing angle represents 14deg, so CSSOM's
                    // shortest serialization omits an exactly equal literal. The
                    // omitted angle is still visited under the cumulative budget.
                    let omit = context.output_suppressed() || angle.is_default_oblique(context)?;
                    let previous = context.replace_output_suppression(omit);
                    let result = (|| {
                        context.append(output, " ")?;
                        angle.append_specified(context, output)
                    })();
                    context.replace_output_suppression(previous);
                    result?;
                }
            }
        }
        Ok(())
    }
}

impl CssFontFaceStyle {
    /// Serializes `auto`, a keyword, or the authored oblique range.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges the descriptor keyword and each explicit endpoint cumulatively.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
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
        self.append_specified(context, output)?;
        Ok(())
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Auto => append_text("auto", context, output)?,
            Self::Keyword(keyword) => append_keyword(*keyword, context, output)?,
            Self::Oblique { range } => {
                append_text("oblique", context, output)?;
                if let Some(range) = range {
                    context.append(output, " ")?;
                    range.append_specified(context, output)?;
                }
            }
        }
        Ok(())
    }
}

fn append_text(
    text: &str,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> SerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, text)
}

impl CssFontObliqueAngle {
    /// Whether this literal is exactly the 14deg that an omitted angle represents.
    /// Symbolic math stays explicit: its specified form is not evaluated here.
    fn is_default_oblique(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<bool> {
        let Some(literal) = self.literal_component() else {
            return Ok(false);
        };
        let default = CssComponentValue::try_token("14deg").expect("static 14deg dimension");
        Ok(
            crate::numeric::exact_dimension_identity(Some(literal), Some(&default), context)?
                .unwrap_or(false),
        )
    }

    fn specified_semantically_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<bool> {
        if let (Some(a), Some(b)) = (self.calculation(), other.calculation()) {
            return a.expression.specified_identity_eq(&b.expression, context);
        }
        Ok(crate::numeric::exact_dimension_identity(
            self.literal_component(),
            other.literal_component(),
            context,
        )?
        .unwrap_or_else(|| self == other))
    }
}
