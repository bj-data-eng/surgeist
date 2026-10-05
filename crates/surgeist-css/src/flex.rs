//! Checked specified flex components before contextual flex layout.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssBoxSize, CssCalcSize, CssSizeValue, CssSpecifiedNonNegativeLengthPercentage,
    CssSpecifiedNonNegativeNumber, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

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

    pub fn serialize_specified(self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        self,
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
        self.serialize_specified_into(context, output)?;
        Ok(())
    }

    fn serialize_specified_into(
        self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        append_keyword(context, output, self.keyword())
    }
}

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

    pub fn serialize_specified(self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        self,
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
        self.serialize_specified_into(context, output)?;
        Ok(())
    }

    fn serialize_specified_into(
        self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        append_keyword(context, output, self.keyword())
    }
}

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

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

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
        self.direction.serialize_specified_into(context, output)?;
        context.append(output, " ")?;
        self.wrap.serialize_specified_into(context, output)?;
        Ok(())
    }
}

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

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

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
        self.serialize_specified_into(context, output)?;
        Ok(())
    }

    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match &self.0 {
            Basis::Content => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "content")
            }
            Basis::Size(value) => value.serialize_specified_into(context, output),
            Basis::CalcSize(value) => value.serialize_specified_into(context, output),
        }
    }
}

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
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

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
        self.serialize_specified_into(context, output)?;
        Ok(())
    }

    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::None => append_keyword(context, output, "none"),
            Self::Auto => append_keyword(context, output, "auto"),
            Self::Components(components) => {
                if let Some(grow) = &components.grow {
                    let captured = grow.capture_specified(context)?;
                    context.append(output, &captured)?;
                } else {
                    append_keyword(context, output, "1")?;
                }
                context.append(output, " ")?;
                if let Some(shrink) = &components.shrink {
                    let captured = shrink.capture_specified(context)?;
                    context.append(output, &captured)?;
                } else {
                    append_keyword(context, output, "1")?;
                }
                context.append(output, " ")?;
                if let Some(basis) = &components.basis {
                    basis.serialize_specified_into(context, output)
                } else {
                    append_keyword(context, output, "0")
                }
            }
        }
    }
}

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

fn append_keyword(
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    keyword: &'static str,
) -> SerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, keyword)
}
