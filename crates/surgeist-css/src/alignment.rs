//! Checked authored values for the nine Box Alignment 3 properties.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflowPosition {
    Safe,
    Unsafe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBaselinePosition {
    Baseline,
    First,
    Last,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAlignmentPosition {
    Center,
    Start,
    End,
    SelfStart,
    SelfEnd,
    FlexStart,
    FlexEnd,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLegacyAlignment {
    Left,
    Right,
    Center,
}

/// A syntactically meaningful alignment component, before property admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAlignmentValue {
    Normal {
        overflow: Option<CssOverflowPosition>,
    },
    Auto,
    Stretch,
    Baseline(CssBaselinePosition),
    Position {
        overflow: Option<CssOverflowPosition>,
        position: CssAlignmentPosition,
    },
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Legacy(Option<CssLegacyAlignment>),
}

#[derive(Clone, Copy)]
enum Domain {
    AlignContent,
    JustifyContent,
    AlignItems,
    JustifyItems,
    AlignSelf,
    JustifySelf,
}

const fn admits(value: CssAlignmentValue, domain: Domain) -> bool {
    use CssAlignmentValue as V;
    match value {
        V::Normal { overflow: None } => true,
        V::Normal { overflow: Some(_) } | V::Auto => {
            matches!(domain, Domain::AlignSelf | Domain::JustifySelf)
        }
        V::Stretch => true,
        V::Baseline(_) => !matches!(domain, Domain::JustifyContent),
        V::SpaceBetween | V::SpaceAround | V::SpaceEvenly => {
            matches!(domain, Domain::AlignContent | Domain::JustifyContent)
        }
        V::Legacy(_) => matches!(domain, Domain::JustifyItems),
        V::Position { position, .. } => match position {
            CssAlignmentPosition::Center
            | CssAlignmentPosition::Start
            | CssAlignmentPosition::End
            | CssAlignmentPosition::FlexStart
            | CssAlignmentPosition::FlexEnd => true,
            CssAlignmentPosition::SelfStart | CssAlignmentPosition::SelfEnd => {
                !matches!(domain, Domain::AlignContent | Domain::JustifyContent)
            }
            CssAlignmentPosition::Left | CssAlignmentPosition::Right => matches!(
                domain,
                Domain::JustifyContent | Domain::JustifyItems | Domain::JustifySelf
            ),
        },
    }
}

macro_rules! checked_alignment {
    ($name:ident, $domain:ident) => {
        /// A property-admitted authored alignment value. Invalid members of the
        /// shared union cannot be stored in this private-field wrapper.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(CssAlignmentValue);

        impl $name {
            /// Checks the property grammar before constructing the value.
            pub const fn try_new(value: CssAlignmentValue) -> Option<Self> {
                if admits(value, Domain::$domain) {
                    Some(Self(value))
                } else {
                    None
                }
            }

            #[must_use]
            /// Returns the admitted specified component without layout resolution.
            pub const fn value(&self) -> CssAlignmentValue {
                self.0
            }

            /// Serializes the canonical specified value. An explicit `first
            /// baseline` component serializes as `baseline` per Box Alignment 3.
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
                serialize_value_into(self.0, context, output)?;
                Ok(())

    }
        }
    };
}

checked_alignment!(CssAlignContentValue, AlignContent);
checked_alignment!(CssJustifyContentValue, JustifyContent);
checked_alignment!(CssAlignItemsValue, AlignItems);
checked_alignment!(CssJustifyItemsValue, JustifyItems);
checked_alignment!(CssAlignSelfValue, AlignSelf);
checked_alignment!(CssJustifySelfValue, JustifySelf);

macro_rules! checked_pair {
    ($name:ident, $align:ident, $justify:ident, $default:expr) => {
        /// Two independently admitted axis values of a place shorthand.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name {
            align: $align,
            justify: $justify,
        }

        impl $name {
            /// Forms an explicit pair from two already checked property values.
            pub const fn new(align: $align, justify: $justify) -> Self {
                Self { align, justify }
            }

            /// Applies the shorthand's specified second-value omission rule.
            pub fn from_align(align: $align) -> Self {
                let value = ($default)(align.value());
                let Some(justify) = $justify::try_new(value) else {
                    unreachable!("align component must be valid on justify axis");
                };
                Self { align, justify }
            }

            #[must_use]
            /// Returns the checked align-axis value.
            pub const fn align(&self) -> $align {
                self.align
            }

            #[must_use]
            /// Returns the checked justify-axis value.
            pub const fn justify(&self) -> $justify {
                self.justify
            }

            /// Serializes both effective components in align-then-justify order.
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
                serialize_value_into(self.align.value(), context, output)?;
                context.append(output, " ")?;
                serialize_value_into(self.justify.value(), context, output)?;
                Ok(())

    }
        }
    };
}

const fn content_default(value: CssAlignmentValue) -> CssAlignmentValue {
    if matches!(value, CssAlignmentValue::Baseline(_)) {
        CssAlignmentValue::Position {
            overflow: None,
            position: CssAlignmentPosition::Start,
        }
    } else {
        value
    }
}

const fn copied(value: CssAlignmentValue) -> CssAlignmentValue {
    value
}

checked_pair!(
    CssPlaceContentValue,
    CssAlignContentValue,
    CssJustifyContentValue,
    content_default
);
checked_pair!(
    CssPlaceItemsValue,
    CssAlignItemsValue,
    CssJustifyItemsValue,
    copied
);
checked_pair!(
    CssPlaceSelfValue,
    CssAlignSelfValue,
    CssJustifySelfValue,
    copied
);

fn keyword(
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    text: &'static str,
) -> SerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, text)
}

fn prefix(
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    overflow: Option<CssOverflowPosition>,
) -> SerializationResult<()> {
    if let Some(overflow) = overflow {
        keyword(
            context,
            output,
            match overflow {
                CssOverflowPosition::Safe => "safe",
                CssOverflowPosition::Unsafe => "unsafe",
            },
        )?;
        context.append(output, " ")?;
    }
    Ok(())
}

fn serialize_value_into(
    value: CssAlignmentValue,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> SerializationResult<()> {
    use CssAlignmentValue as V;
    match value {
        V::Normal { overflow } => {
            prefix(context, output, overflow)?;
            keyword(context, output, "normal")
        }
        V::Auto => keyword(context, output, "auto"),
        V::Stretch => keyword(context, output, "stretch"),
        V::Baseline(CssBaselinePosition::Baseline | CssBaselinePosition::First) => {
            keyword(context, output, "baseline")
        }
        V::Baseline(CssBaselinePosition::Last) => {
            keyword(context, output, "last")?;
            context.append(output, " ")?;
            keyword(context, output, "baseline")
        }
        V::Position { overflow, position } => {
            prefix(context, output, overflow)?;
            keyword(
                context,
                output,
                match position {
                    CssAlignmentPosition::Center => "center",
                    CssAlignmentPosition::Start => "start",
                    CssAlignmentPosition::End => "end",
                    CssAlignmentPosition::SelfStart => "self-start",
                    CssAlignmentPosition::SelfEnd => "self-end",
                    CssAlignmentPosition::FlexStart => "flex-start",
                    CssAlignmentPosition::FlexEnd => "flex-end",
                    CssAlignmentPosition::Left => "left",
                    CssAlignmentPosition::Right => "right",
                },
            )
        }
        V::SpaceBetween => keyword(context, output, "space-between"),
        V::SpaceAround => keyword(context, output, "space-around"),
        V::SpaceEvenly => keyword(context, output, "space-evenly"),
        V::Legacy(direction) => {
            keyword(context, output, "legacy")?;
            if let Some(direction) = direction {
                context.append(output, " ")?;
                keyword(
                    context,
                    output,
                    match direction {
                        CssLegacyAlignment::Left => "left",
                        CssLegacyAlignment::Right => "right",
                        CssLegacyAlignment::Center => "center",
                    },
                )?;
            }
            Ok(())
        }
    }
}
