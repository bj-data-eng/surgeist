//! Exact authored Fonts 4 style values, before matching or computed angle ranges.
//! Ordinary emitted coefficients round to six fractional places (nearest, ties
//! away from zero); finite calculated text rounds the projected binary64 value with the
//! same policy. Retained coefficients/ranges and calculation arithmetic stay unchanged.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssAngle, CssAngleUnit, CssComponentValue, CssNumeric, CssNumericLiteral, CssRange,
    CssSerializationError,
};

type SerializationResult<T> = Result<T, CssSerializationError>;

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

/// The Fonts 4 oblique angle range `[-90deg,90deg]` of a [`CssAngle`] root.
///
/// Every unit compares its exact authored decimal with the quarter-turn bound:
/// 90deg, 100grad, 0.25turn, or π/2 radians. The irrational radian bound is its
/// first 1000 significant digits, so the decision is exact for every
/// coefficient of at most that many significant digits; a longer coefficient
/// that matches the whole prefix is rejected. Math defers the range to
/// computed-value handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssObliqueAngleRange {}

impl crate::sealed::RangeBounds<CssAngle> for CssObliqueAngleRange {
    fn admits(literal: &CssNumericLiteral<CssAngle>) -> bool {
        let decimal = crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation());
        match literal.unit() {
            CssAngleUnit::Degrees => decimal.absolute_at_most("9", 1),
            CssAngleUnit::Gradians => decimal.absolute_at_most("1", 2),
            CssAngleUnit::Turns => decimal.absolute_at_most("25", -2),
            CssAngleUnit::Radians => {
                decimal.absolute_at_most(HALF_PI_DIGITS, -(HALF_PI_DIGITS.len() as i128 - 1))
            }
        }
    }
}
impl CssRange<CssAngle> for CssObliqueAngleRange {}

/// An authored `font-style` property value.
///
/// # Specified serialization
///
/// Serializes the authored keyword or optional oblique angle.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontStyle {
    Keyword(CssFontStyleKeyword),
    Oblique {
        angle: Option<CssNumeric<CssAngle, CssObliqueAngleRange>>,
    },
}

/// One or two authored oblique descriptor endpoints, in source order.
///
/// # Specified serialization
///
/// Serializes the exact authored one- or two-endpoint range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFaceObliqueRange {
    start: CssNumeric<CssAngle, CssObliqueAngleRange>,
    end: Option<CssNumeric<CssAngle, CssObliqueAngleRange>>,
}

impl CssFontFaceObliqueRange {
    /// Retains already-checked endpoints, including descending pairs.
    #[must_use]
    pub const fn new(
        start: CssNumeric<CssAngle, CssObliqueAngleRange>,
        end: Option<CssNumeric<CssAngle, CssObliqueAngleRange>>,
    ) -> Self {
        Self { start, end }
    }

    #[must_use]
    /// Borrows the required first authored endpoint.
    pub const fn start(&self) -> &CssNumeric<CssAngle, CssObliqueAngleRange> {
        &self.start
    }

    #[must_use]
    /// Borrows an explicitly authored second endpoint, if present.
    pub const fn end(&self) -> Option<&CssNumeric<CssAngle, CssObliqueAngleRange>> {
        self.end.as_ref()
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
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontFaceObliqueRange {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontFaceObliqueRange);

/// An authored `@font-face` style descriptor, before font matching.
///
/// # Specified serialization
///
/// Serializes `auto`, a keyword, or the authored oblique range.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceStyle {
    Auto,
    Keyword(CssFontStyleKeyword),
    Oblique {
        range: Option<CssFontFaceObliqueRange>,
    },
}

impl CssFontStyle {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Keyword(keyword) => context.keyword(output, keyword.as_css())?,
            Self::Oblique { angle } => {
                context.charge_nodes(1)?;
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
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontStyle {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontStyle);

impl CssFontFaceStyle {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Auto => context.keyword(output, "auto")?,
            Self::Keyword(keyword) => context.keyword(output, keyword.as_css())?,
            Self::Oblique { range } => {
                context.keyword(output, "oblique")?;
                if let Some(range) = range {
                    context.append(output, " ")?;
                    range.append_specified(context, output)?;
                }
            }
        }
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontFaceStyle {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontFaceStyle);

impl CssNumeric<CssAngle, CssObliqueAngleRange> {
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
