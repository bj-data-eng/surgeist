//! Checked specified margins and padding, before writing-mode or layout resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssSpecifiedLengthPercentage, CssSpecifiedNonNegativeLengthPercentage,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits, CssValueOrigin,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

trait SpacingValue {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()>;
}

fn append_auto(
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> SerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, "auto")
}

/// One physical or flow-relative margin longhand value.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssMarginValue {
    /// The deferred `auto` keyword.
    Auto,
    /// An exact signed length-percentage, including deferred math.
    LengthPercentage(CssSpecifiedLengthPercentage),
}

impl CssMarginValue {
    /// Borrows the signed length-percentage, if this is not `auto`.
    pub fn length_percentage(&self) -> Option<&CssSpecifiedLengthPercentage> {
        match self {
            Self::Auto => None,
            Self::LengthPercentage(value) => Some(value),
        }
    }

    /// Returns the retained origin of a numeric value; `auto` has no stored origin.
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.length_percentage()
            .map(CssSpecifiedLengthPercentage::origin)
    }

    /// Serializes the specified value without resolving layout context.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under explicit resource limits.
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
        self.serialize_into(&mut writer.context, &mut writer.css)
    }
}

impl SpacingValue for CssMarginValue {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Auto => append_auto(context, output),
            Self::LengthPercentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    }
}

impl PartialEq for CssMarginValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}

impl Eq for CssMarginValue {}

/// One nonnegative padding longhand value.
#[derive(Clone, Debug)]
pub struct CssPaddingValue(CssSpecifiedNonNegativeLengthPercentage);

impl CssPaddingValue {
    /// Wraps an already checked nonnegative length-percentage.
    #[must_use]
    pub fn new(value: CssSpecifiedNonNegativeLengthPercentage) -> Self {
        Self(value)
    }

    /// Borrows the checked length-percentage.
    pub fn length_percentage(&self) -> &CssSpecifiedNonNegativeLengthPercentage {
        &self.0
    }

    /// Returns the original parsed or programmatic numeric origin.
    pub fn origin(&self) -> &CssValueOrigin {
        self.0.origin()
    }

    /// Serializes the specified value without resolving layout context.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under explicit resource limits.
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
        self.serialize_into(&mut writer.context, &mut writer.css)
    }
}

impl SpacingValue for CssPaddingValue {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        let captured = self.0.capture_specified(context)?;
        context.append(output, &captured)
    }
}

impl PartialEq for CssPaddingValue {
    fn eq(&self, other: &Self) -> bool {
        self.0.structural_eq(&other.0)
    }
}

impl Eq for CssPaddingValue {}

/// Identifies whether four shorthand positions name physical or flow-relative sides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssBoxSideKind {
    /// Top, right, bottom, left.
    Physical,
    /// Block-start, inline-start, block-end, inline-end.
    Logical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Pair<T> {
    start: T,
    authored_end: Option<T>,
}

impl<T> Pair<T> {
    fn start(&self) -> &T {
        &self.start
    }

    fn authored_end(&self) -> Option<&T> {
        self.authored_end.as_ref()
    }

    fn end(&self) -> &T {
        self.authored_end.as_ref().unwrap_or(&self.start)
    }
}

impl<T: SpacingValue> SpacingValue for Pair<T> {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        self.start.serialize_into(context, output)?;
        if let Some(end) = &self.authored_end {
            context.append(output, " ")?;
            end.serialize_into(context, output)?;
        }
        Ok(())
    }
}

/// One- or two-value logical margin-axis shorthand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMarginPair(Pair<CssMarginValue>);

impl CssMarginPair {
    /// Retains the authored start and optional end values.
    #[must_use]
    pub fn new(start: CssMarginValue, authored_end: Option<CssMarginValue>) -> Self {
        Self(Pair {
            start,
            authored_end,
        })
    }

    /// Borrows the authored start value.
    pub fn start(&self) -> &CssMarginValue {
        self.0.start()
    }

    /// Borrows the explicitly authored end value, if any.
    pub fn authored_end(&self) -> Option<&CssMarginValue> {
        self.0.authored_end()
    }

    /// Borrows the effective end assignment, repeating start for one value.
    pub fn end(&self) -> &CssMarginValue {
        self.0.end()
    }

    /// Serializes only authored values with default limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes both authored values under one resource budget.
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
        self.0.serialize_into(&mut writer.context, &mut writer.css)
    }
}

/// One- or two-value logical padding-axis shorthand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPaddingPair(Pair<CssPaddingValue>);

impl CssPaddingPair {
    /// Retains the authored start and optional end values.
    #[must_use]
    pub fn new(start: CssPaddingValue, authored_end: Option<CssPaddingValue>) -> Self {
        Self(Pair {
            start,
            authored_end,
        })
    }

    /// Borrows the authored start value.
    pub fn start(&self) -> &CssPaddingValue {
        self.0.start()
    }

    /// Borrows the explicitly authored end value, if any.
    pub fn authored_end(&self) -> Option<&CssPaddingValue> {
        self.0.authored_end()
    }

    /// Borrows the effective end assignment, repeating start for one value.
    pub fn end(&self) -> &CssPaddingValue {
        self.0.end()
    }

    /// Serializes only authored values with default limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes both authored values under one resource budget.
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
        self.0.serialize_into(&mut writer.context, &mut writer.css)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FourSides<T> {
    kind: CssBoxSideKind,
    authored_values: Box<[T]>,
}

impl<T> FourSides<T> {
    fn try_new(kind: CssBoxSideKind, authored_values: Vec<T>) -> Option<Self> {
        if !(1..=4).contains(&authored_values.len()) {
            return None;
        }
        Some(Self {
            kind,
            authored_values: authored_values.into_boxed_slice(),
        })
    }

    fn assigned_values(&self) -> [&T; 4] {
        let values = &self.authored_values;
        match values.len() {
            1 => [&values[0], &values[0], &values[0], &values[0]],
            2 => [&values[0], &values[1], &values[0], &values[1]],
            3 => [&values[0], &values[1], &values[2], &values[1]],
            4 => [&values[0], &values[1], &values[2], &values[3]],
            _ => unreachable!("checked shorthand cardinality"),
        }
    }
}

impl<T: SpacingValue> SpacingValue for FourSides<T> {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        if self.kind == CssBoxSideKind::Logical {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            context.append(output, "logical ")?;
        }
        for (index, value) in self.authored_values.iter().enumerate() {
            if index > 0 {
                context.append(output, " ")?;
            }
            value.serialize_into(context, output)?;
        }
        Ok(())
    }
}

/// A checked one- to four-value margin shorthand with explicit side kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMarginShorthand(FourSides<CssMarginValue>);

impl CssMarginShorthand {
    /// Checks the authored cardinality and retains physical or logical intent.
    pub fn try_new(kind: CssBoxSideKind, authored_values: Vec<CssMarginValue>) -> Option<Self> {
        FourSides::try_new(kind, authored_values).map(Self)
    }

    /// Returns the explicit physical or logical side kind.
    pub fn kind(&self) -> CssBoxSideKind {
        self.0.kind
    }

    /// Borrows the exact one to four authored values in source order.
    pub fn authored_values(&self) -> &[CssMarginValue] {
        &self.0.authored_values
    }

    /// Returns assignments in top/right/bottom/left order for physical values,
    /// or block-start/inline-start/block-end/inline-end for logical values.
    /// This does not determine the shorthand's full reset membership.
    pub fn assigned_values(&self) -> [&CssMarginValue; 4] {
        self.0.assigned_values()
    }

    /// Serializes the authored shorthand with default limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the `logical` switch and authored values under one budget.
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
        self.0.serialize_into(&mut writer.context, &mut writer.css)
    }
}

/// A checked one- to four-value padding shorthand with explicit side kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPaddingShorthand(FourSides<CssPaddingValue>);

impl CssPaddingShorthand {
    /// Checks the authored cardinality and retains physical or logical intent.
    pub fn try_new(kind: CssBoxSideKind, authored_values: Vec<CssPaddingValue>) -> Option<Self> {
        FourSides::try_new(kind, authored_values).map(Self)
    }

    /// Returns the explicit physical or logical side kind.
    pub fn kind(&self) -> CssBoxSideKind {
        self.0.kind
    }

    /// Borrows the exact one to four authored values in source order.
    pub fn authored_values(&self) -> &[CssPaddingValue] {
        &self.0.authored_values
    }

    /// Returns assignments in top/right/bottom/left order for physical values,
    /// or block-start/inline-start/block-end/inline-end for logical values.
    /// This does not determine the shorthand's full reset membership.
    pub fn assigned_values(&self) -> [&CssPaddingValue; 4] {
        self.0.assigned_values()
    }

    /// Serializes the authored shorthand with default limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the `logical` switch and authored values under one budget.
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
        self.0.serialize_into(&mut writer.context, &mut writer.css)
    }
}
