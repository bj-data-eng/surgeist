//! Exact authored positioned offsets before writing-mode and layout resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssBoxSideKind, CssLengthPercentage, CssNumeric, CssSerializationError, CssValueOrigin,
};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// One physical or logical inset longhand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssInsetValue {
    /// The unresolved automatic offset.
    Auto,
    /// An exact signed length-percentage, including deferred math.
    LengthPercentage(CssNumeric<CssLengthPercentage>),
}

impl CssInsetValue {
    /// Borrows the checked numeric value, if present.
    pub fn length_percentage(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
        match self {
            Self::Auto => None,
            Self::LengthPercentage(value) => Some(value),
        }
    }

    /// Borrows a retained numeric origin; `auto` has no numeric origin.
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.length_percentage()
            .map(CssNumeric::<CssLengthPercentage>::origin)
    }

    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Auto => {
                context.charge_nodes(1)?;
                context.append(output, "auto")
            }
            Self::LengthPercentage(value) => {
                let text = value.capture_specified(context)?;
                context.append(output, &text)
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssInsetValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssInsetValue);

impl PartialEq for CssInsetValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
            _ => false,
        }
    }
}
impl Eq for CssInsetValue {}

/// A one- or two-value logical inset-axis shorthand.
///
/// # Specified serialization
///
/// Serializes only authored components.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssInsetPair {
    start: CssInsetValue,
    authored_end: Option<CssInsetValue>,
}

impl CssInsetPair {
    /// Retains the authored start and optional end.
    #[must_use]
    pub fn new(start: CssInsetValue, authored_end: Option<CssInsetValue>) -> Self {
        Self {
            start,
            authored_end,
        }
    }

    /// Borrows the start assignment.
    pub fn start(&self) -> &CssInsetValue {
        &self.start
    }
    /// Borrows the explicitly authored end, if any.
    pub fn authored_end(&self) -> Option<&CssInsetValue> {
        self.authored_end.as_ref()
    }
    /// Borrows the effective end, repeating start when omitted.
    pub fn end(&self) -> &CssInsetValue {
        self.authored_end.as_ref().unwrap_or(&self.start)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssInsetPair {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.source_member(0, |writer| {
            self.start
                .serialize_into(&mut writer.context, &mut writer.css)
        })?;
        if let Some(end) = &self.authored_end {
            writer.source_member(1, |writer| {
                writer.append(" ")?;
                end.serialize_into(&mut writer.context, &mut writer.css)
            })?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssInsetPair);

/// A one- to four-value physical or logical inset shorthand.
///
/// # Specified serialization
///
/// Serializes only the authored switch and components.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssInsetShorthand {
    kind: CssBoxSideKind,
    authored_values: Box<[CssInsetValue]>,
}

impl CssInsetShorthand {
    /// Checks the shorthand cardinality and preserves its side kind.
    pub fn try_new(kind: CssBoxSideKind, authored_values: Vec<CssInsetValue>) -> Option<Self> {
        if !(1..=4).contains(&authored_values.len()) {
            return None;
        }
        Some(Self {
            kind,
            authored_values: authored_values.into_boxed_slice(),
        })
    }

    /// Returns the physical or logical side kind.
    pub fn kind(&self) -> CssBoxSideKind {
        self.kind
    }
    /// Borrows the exact authored values in source order.
    pub fn authored_values(&self) -> &[CssInsetValue] {
        &self.authored_values
    }
    /// Returns top/right/bottom/left or block-start/inline-start/block-end/inline-end assignments.
    /// This does not determine the shorthand's full reset membership.
    pub fn assigned_values(&self) -> [&CssInsetValue; 4] {
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
impl crate::specified_rule_serialization::SpecifiedProvider for CssInsetShorthand {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        if self.kind == CssBoxSideKind::Logical {
            writer.source_member(0, |writer| {
                writer.node()?;
                writer.append("logical ")
            })?;
        }
        for (index, value) in self.authored_values.iter().enumerate() {
            writer.source_member(index, |writer| {
                if index > 0 {
                    writer.append(" ")?;
                }
                value.serialize_into(&mut writer.context, &mut writer.css)
            })?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssInsetShorthand);

impl crate::specified_rule_serialization::SpecifiedProvider for crate::CssLayoutPosition {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let text = match self {
            Self::Static => "static",
            Self::Relative => "relative",
            Self::Absolute => "absolute",
            Self::Fixed => "fixed",
            Self::Sticky => "sticky",
        };
        writer.keyword(text)
    }
}
crate::specified_rule_serialization::specified_serialization!(crate::CssLayoutPosition);
