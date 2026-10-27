//! Checked specified margins and padding, before writing-mode or layout resolution.

use crate::component::CssValueOrigin;
use crate::numeric::{CssLengthPercentage, CssNonNegative, CssNumeric};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::specified_inverse::InverseValueEquality as _;
use crate::values::common::common_values::{four_sides_entry, side_element, side_pair_entry};

type SerializationResult<T> = Result<T, CssSerializationError>;

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
    LengthPercentage(CssNumeric<CssLengthPercentage>),
}

impl CssMarginValue {
    /// Borrows the signed length-percentage, if this is not `auto`.
    pub fn length_percentage(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
        match self {
            Self::Auto => None,
            Self::LengthPercentage(value) => Some(value),
        }
    }

    /// Returns the retained origin of a numeric value; `auto` has no stored origin.
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.length_percentage()
            .map(CssNumeric::<CssLengthPercentage>::origin)
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssMarginValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.serialize_into(&mut writer.context, &mut writer.css)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssMarginValue);

impl CssMarginValue {
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
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
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
pub struct CssPaddingValue(CssNumeric<CssLengthPercentage, CssNonNegative>);

impl CssPaddingValue {
    /// Wraps an already checked nonnegative length-percentage.
    #[must_use]
    pub fn new(value: CssNumeric<CssLengthPercentage, CssNonNegative>) -> Self {
        Self(value)
    }

    /// Borrows the checked length-percentage.
    pub fn length_percentage(&self) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        &self.0
    }

    /// Returns the original parsed or programmatic numeric origin.
    pub fn origin(&self) -> &CssValueOrigin {
        self.0.origin()
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssPaddingValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.serialize_into(&mut writer.context, &mut writer.css)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssPaddingValue);

impl CssPaddingValue {
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
        self.0 == other.0
    }
}

impl Eq for CssPaddingValue {}

side_element!(
    CssMarginValue,
    same = |a, b, context| a.inverse_equal(b, context)
);
side_element!(
    CssPaddingValue,
    same = |a, b, context| a.inverse_equal(b, context)
);
four_sides_entry!(CssMarginValue, CssPaddingValue);
side_pair_entry!(CssMarginValue, CssPaddingValue);
