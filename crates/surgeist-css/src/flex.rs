//! Checked specified flex components before contextual flex layout.

use crate::specified_rule_serialization::SpecifiedProvider as _;
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssBoxSize, CssCalcSize, CssSerializationError, CssSizeValue,
    CssSpecifiedNonNegativeLengthPercentage, CssSpecifiedNonNegativeNumber,
};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// Main-axis direction of an authored flex container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl CssFlexDirection {
    const fn keyword(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Column => "column",
            Self::RowReverse => "row-reverse",
            Self::ColumnReverse => "column-reverse",
        }
    }

    fn serialize_specified_into(
        self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        context.keyword(output, self.keyword())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFlexDirection {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFlexDirection);

/// Cross-axis wrapping of an authored flex container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl CssFlexWrap {
    const fn keyword(self) -> &'static str {
        match self {
            Self::NoWrap => "nowrap",
            Self::Wrap => "wrap",
            Self::WrapReverse => "wrap-reverse",
        }
    }

    fn serialize_specified_into(
        self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        context.keyword(output, self.keyword())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFlexWrap {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFlexWrap);

/// The specified direction and wrapping of `flex-flow`.
///
/// Omitted components take their specified initial values; this value does not
/// perform flex layout or resolve either component against layout context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFlexFlow {
    direction: CssFlexDirection,
    wrap: CssFlexWrap,
}

impl CssFlexFlow {
    pub(crate) fn append_cssom_inverse_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        let emit_direction =
            self.direction != CssFlexDirection::Row || self.wrap == CssFlexWrap::NoWrap;
        writer.source_member(0, |writer| {
            if emit_direction {
                self.direction.append_to_rule_writer(writer)
            } else {
                writer.without_output(|writer| self.direction.append_to_rule_writer(writer))
            }
        })?;
        writer.source_member(1, |writer| {
            if self.wrap == CssFlexWrap::NoWrap {
                writer.without_output(|writer| self.wrap.append_to_rule_writer(writer))
            } else {
                if emit_direction {
                    writer.append(" ")?;
                }
                self.wrap.append_to_rule_writer(writer)
            }
        })
    }

    /// Constructs a valid pair of specified direction and wrapping values.
    pub const fn new(direction: CssFlexDirection, wrap: CssFlexWrap) -> Self {
        Self { direction, wrap }
    }

    /// Returns the authored direction component, or `row` when it was omitted.
    #[must_use]
    pub const fn direction(&self) -> CssFlexDirection {
        self.direction
    }

    /// Returns the authored wrapping component, or `nowrap` when it was omitted.
    #[must_use]
    pub const fn wrap(&self) -> CssFlexWrap {
        self.wrap
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFlexFlow {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.direction.serialize_specified_into(context, output)?;
        context.append(output, " ")?;
        self.wrap.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFlexFlow);

#[derive(Clone, Debug)]
enum Basis {
    Content,
    Size(CssSizeValue),
    CalcSize(CssCalcSize),
}

/// A flex basis with width values and the flex-only `content` size.
#[derive(Clone, Debug)]
pub struct CssFlexBasisValue(Basis);

/// Borrowed canonical flex basis.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssFlexBasisRef<'a> {
    Content,
    Size(&'a CssSizeValue),
    CalcSize(&'a CssCalcSize),
}

impl CssFlexBasisValue {
    pub const fn content() -> Self {
        Self(Basis::Content)
    }

    pub fn view(&self) -> CssFlexBasisRef<'_> {
        match &self.0 {
            Basis::Content => CssFlexBasisRef::Content,
            Basis::Size(value) => CssFlexBasisRef::Size(value),
            Basis::CalcSize(value) => CssFlexBasisRef::CalcSize(value),
        }
    }

    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match &self.0 {
            Basis::Content => {
                context.charge_nodes(1)?;
                context.append(output, "content")
            }
            Basis::Size(value) => value.serialize_specified_into(context, output),
            Basis::CalcSize(value) => value.serialize_specified_into(context, output),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFlexBasisValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFlexBasisValue);

impl PartialEq for CssFlexBasisValue {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Basis::Content, Basis::Content) => true,
            (Basis::Size(a), Basis::Size(b)) => a == b,
            (Basis::CalcSize(a), Basis::CalcSize(b)) => a == b,
            _ => false,
        }
    }
}

impl From<CssSizeValue> for CssFlexBasisValue {
    fn from(value: CssSizeValue) -> Self {
        match value {
            CssSizeValue::BoxSize(CssBoxSize::CalcSize(calc)) => {
                Self(Basis::CalcSize(calc.into_calc_size()))
            }
            _ => Self(Basis::Size(value)),
        }
    }
}

impl From<CssCalcSize> for CssFlexBasisValue {
    fn from(value: CssCalcSize) -> Self {
        Self(Basis::CalcSize(value))
    }
}

/// Authored presence of each flex shorthand component.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFlexComponents {
    grow: Option<CssSpecifiedNonNegativeNumber>,
    shrink: Option<CssSpecifiedNonNegativeNumber>,
    basis: Option<CssFlexBasisValue>,
}

impl CssFlexComponents {
    pub fn try_new(
        grow: Option<CssSpecifiedNonNegativeNumber>,
        shrink: Option<CssSpecifiedNonNegativeNumber>,
        basis: Option<CssFlexBasisValue>,
    ) -> Option<Self> {
        if grow.is_none() && basis.is_none() || grow.is_none() && shrink.is_some() {
            return None;
        }
        Some(Self {
            grow,
            shrink,
            basis,
        })
    }

    pub fn grow(&self) -> Option<&CssSpecifiedNonNegativeNumber> {
        self.grow.as_ref()
    }

    pub fn shrink(&self) -> Option<&CssSpecifiedNonNegativeNumber> {
        self.shrink.as_ref()
    }

    pub fn basis(&self) -> Option<&CssFlexBasisValue> {
        self.basis.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFlexValue {
    None,
    Auto,
    Components(CssFlexComponents),
}

impl CssFlexValue {
    pub(crate) fn append_cssom_inverse_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        let Self::Components(value) = self else {
            return self.append_to_rule_writer(writer);
        };
        let Some(grow) = value.grow() else {
            return self.append_to_rule_writer(writer);
        };
        let emit_shrink = value.shrink().is_some_and(|v| !ordinary_factor(v, "1"));
        let emit_basis = value.basis().is_some_and(|v| !inverse_default_basis(v));
        writer.source_member(0, |writer| grow.append_to_rule_writer(writer))?;
        if let Some(shrink) = value.shrink() {
            writer.source_member(1, |writer| {
                if emit_shrink {
                    writer.append(" ")?;
                    shrink.append_to_rule_writer(writer)
                } else {
                    writer.without_output(|writer| shrink.append_to_rule_writer(writer))
                }
            })?;
        }
        if let Some(basis) = value.basis() {
            writer.source_member(2, |writer| {
                if emit_basis {
                    writer.append(" ")?;
                    basis.append_to_rule_writer(writer)
                } else {
                    writer.without_output(|writer| basis.append_to_rule_writer(writer))
                }
            })?;
        }
        Ok(())
    }

    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::None => context.keyword(output, "none"),
            Self::Auto => context.keyword(output, "auto"),
            Self::Components(components) => {
                if let Some(grow) = &components.grow {
                    let captured = grow.capture_specified(context)?;
                    context.append(output, &captured)?;
                } else {
                    context.keyword(output, "1")?;
                }
                context.append(output, " ")?;
                if let Some(shrink) = &components.shrink {
                    let captured = shrink.capture_specified(context)?;
                    context.append(output, &captured)?;
                } else {
                    context.keyword(output, "1")?;
                }
                context.append(output, " ")?;
                if let Some(basis) = &components.basis {
                    basis.serialize_specified_into(context, output)
                } else {
                    context.keyword(output, "0")
                }
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFlexValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFlexValue);

fn factor(value: &str) -> CssSpecifiedNonNegativeNumber {
    let component = crate::CssComponentValue::try_number(value).expect("fixed flex factor token");
    CssSpecifiedNonNegativeNumber::try_from_component(component)
        .expect("fixed nonnegative flex factor")
}

pub(crate) fn initial_grow() -> CssSpecifiedNonNegativeNumber {
    factor("0")
}

pub(crate) fn initial_shrink() -> CssSpecifiedNonNegativeNumber {
    factor("1")
}

fn default_basis() -> CssFlexBasisValue {
    let zero = CssSpecifiedNonNegativeLengthPercentage::zero();
    CssFlexBasisValue::from(CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(zero)))
}

// The owning shorthand default is an ordinary unitless zero length. Compare
// borrowed exact coefficients with its implicit Px unit, without allocating a
// second checked default just to inspect it. Percentage and math stay distinct.
fn inverse_default_basis(value: &CssFlexBasisValue) -> bool {
    let CssFlexBasisRef::Size(CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(value))) =
        value.view()
    else {
        return false;
    };
    let Some(component) = value.literal_component() else {
        return false;
    };
    match component.view() {
        crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Number(number)) => {
            crate::exact_decimal::LexicalDecimal::new(number.representation()).is_zero()
        }
        crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension { number, unit }) => {
            unit.eq_ignore_ascii_case("px")
                && crate::exact_decimal::LexicalDecimal::new(number.representation()).is_zero()
        }
        _ => false,
    }
}

pub(crate) fn effective_grow(value: &CssFlexValue) -> Option<CssSpecifiedNonNegativeNumber> {
    Some(match value {
        CssFlexValue::None => initial_grow(),
        CssFlexValue::Auto => initial_shrink(),
        CssFlexValue::Components(value) => value.grow.clone().unwrap_or_else(initial_shrink),
    })
}

pub(crate) fn effective_shrink(value: &CssFlexValue) -> Option<CssSpecifiedNonNegativeNumber> {
    Some(match value {
        CssFlexValue::None => initial_grow(),
        CssFlexValue::Auto => initial_shrink(),
        CssFlexValue::Components(value) => value.shrink.clone().unwrap_or_else(initial_shrink),
    })
}

pub(crate) fn effective_basis(value: &CssFlexValue) -> Option<CssFlexBasisValue> {
    Some(match value {
        CssFlexValue::None | CssFlexValue::Auto => CssFlexBasisValue::from(CssSizeValue::Auto),
        CssFlexValue::Components(value) => value.basis.clone().unwrap_or_else(default_basis),
    })
}

pub(crate) fn ordinary_factor(value: &CssSpecifiedNonNegativeNumber, expected: &str) -> bool {
    value.literal_component().is_some_and(|value| matches!(value.view(),
        crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Number(number))
        if crate::exact_decimal::LexicalDecimal::new(number.representation()).value_eq(&crate::exact_decimal::LexicalDecimal::new(expected))))
}
