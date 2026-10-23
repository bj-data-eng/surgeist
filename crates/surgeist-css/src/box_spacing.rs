//! Checked specified margins and padding, before writing-mode or layout resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssSerializationError, CssSpecifiedLengthPercentage, CssSpecifiedNonNegativeLengthPercentage,
    CssValueOrigin,
};

type SerializationResult<T> = Result<T, CssSerializationError>;

trait SpacingValue {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()>;
}

/// One physical or flow-relative margin longhand value.
///
/// # Specified serialization
///
/// Serializes the specified value without resolving layout context.
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
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssMarginValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.serialize_into(&mut writer.context, &mut writer.css)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssMarginValue);

impl SpacingValue for CssMarginValue {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Auto => context.keyword(output, "auto"),
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
///
/// # Specified serialization
///
/// Serializes the specified value without resolving layout context.
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
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssPaddingValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.serialize_into(&mut writer.context, &mut writer.css)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssPaddingValue);

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

impl<T: SpacingValue> crate::specified_rule_serialization::SpecifiedProvider for Pair<T> {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
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
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssMarginPair {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.0.append_to_rule_writer(writer)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssMarginPair);

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
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssPaddingPair {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.0.append_to_rule_writer(writer)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssPaddingPair);

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

impl<T: SpacingValue> crate::specified_rule_serialization::SpecifiedProvider for FourSides<T> {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
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
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssMarginShorthand {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.0.append_to_rule_writer(writer)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssMarginShorthand);

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
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssPaddingShorthand {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.0.append_to_rule_writer(writer)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssPaddingShorthand);
