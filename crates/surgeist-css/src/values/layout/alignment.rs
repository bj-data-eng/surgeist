//! Checked authored values for the nine Box Alignment 3 properties.

use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;

type SerializationResult<T> = Result<T, CssSerializationError>;

/// The `<overflow-position>` keyword that controls alignment when the alignment subject overflows its alignment container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflowPosition {
    /// `safe`: if the subject overflows the container, it is aligned as if the alignment mode were `flex-start`.
    Safe,
    /// `unsafe`: the given alignment is honored regardless of whether the subject overflows.
    Unsafe,
}

/// The `<baseline-position>` forms: an optional `first` or `last` preference followed by `baseline`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBaselinePosition {
    /// `baseline`: the bare keyword, aligning to the first baseline set; it serializes as `baseline`.
    Baseline,
    /// `first baseline`: the explicit first-baseline form; it serializes as `baseline`.
    First,
    /// `last baseline`: aligns to the last baseline set.
    Last,
}

/// The `<self-position>` keywords that align the subject to a position in its alignment container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAlignmentPosition {
    /// `center`: centers the subject in the container.
    Center,
    /// `start`: aligns the subject flush with the container's start edge in the appropriate axis; the keyword is flow-relative, so the writing mode decides which side that is.
    Start,
    /// `end`: aligns the subject flush with the container's end edge in the appropriate axis; the keyword is flow-relative, so the writing mode decides which side that is.
    End,
    /// `self-start`: aligns the subject flush with the container edge that corresponds to the subject's own start side in the appropriate axis.
    SelfStart,
    /// `self-end`: aligns the subject flush with the container edge that corresponds to the subject's own end side in the appropriate axis.
    SelfEnd,
    /// `flex-start`: aligns to the flex container's main-start or cross-start side, as appropriate; outside flex layout it behaves as `start`.
    FlexStart,
    /// `flex-end`: aligns to the flex container's main-end or cross-end side, as appropriate; outside flex layout it behaves as `end`.
    FlexEnd,
    /// `left`: aligns the subject flush with the container's line-left or physical left edge, whichever lies in the axis; where the axis is not parallel to left and right it behaves as `start`.
    Left,
    /// `right`: aligns the subject flush with the container's line-right or physical right edge, whichever lies in the axis; where the axis is not parallel to left and right it behaves as `start`.
    Right,
}

/// The optional direction keyword of a `legacy` alignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLegacyAlignment {
    /// `legacy left`: the legacy alignment with a left direction.
    Left,
    /// `legacy right`: the legacy alignment with a right direction.
    Right,
    /// `legacy center`: the legacy alignment with a center direction.
    Center,
}

/// A syntactically meaningful alignment component, before property admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAlignmentValue {
    /// `normal`, with an optional preceding overflow keyword that only `align-self` and `justify-self` admit.
    Normal {
        /// The authored overflow keyword, if any.
        overflow: Option<CssOverflowPosition>,
    },
    /// `auto`: behaves as the parent's computed `align-items` or `justify-items` value (without any `legacy` keyword), or as `normal` when the box has no parent or when the actual position of an absolutely positioned box is determined.
    Auto,
    /// `stretch`: sizes an automatically sized subject to fill the container as far as its size constraints allow.
    Stretch,
    /// An authored baseline alignment.
    Baseline(CssBaselinePosition),
    /// An optional overflow keyword followed by a self-position keyword.
    Position {
        /// The authored overflow keyword, if any.
        overflow: Option<CssOverflowPosition>,
        /// The self-position keyword.
        position: CssAlignmentPosition,
    },
    /// `space-between`: distributes free space between items, with the first and last flush with the container edges.
    SpaceBetween,
    /// `space-around`: distributes free space around items, with half-size spaces at the container edges.
    SpaceAround,
    /// `space-evenly`: distributes free space so that all spaces, including those at the container edges, are equal.
    SpaceEvenly,
    /// `legacy`, with an optional `left`, `right` or `center` direction; admitted only by `justify-items`.
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
        ///
        /// # Specified serialization
        ///
        /// An explicit `first baseline` component serializes as `baseline` per
        /// Box Alignment 3.
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
        }
        impl crate::serialization::specified_rule_serialization::SpecifiedProvider for $name {
            fn append_to_rule_writer(
                &self,
                writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
                let context = &mut writer.context;
                let output = &mut writer.css;
                serialize_value_into(self.0, context, output)?;
                Ok(())
            }
        }
        crate::serialization::specified_rule_serialization::specified_serialization!($name);
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
        ///
        /// # Specified serialization
        ///
        /// Serializes both effective components in align-then-justify order.
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

            pub(crate) fn append_cssom_inverse_to_rule_writer(
                &self,
                writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> SerializationResult<()> {
                writer.source_member(0, |writer| self.align.append_to_rule_writer(writer))?;
                writer.source_member(1, |writer| {
                    if self.justify == Self::from_align(self.align).justify {
                        writer.without_output(|writer| self.justify.append_to_rule_writer(writer))
                    } else {
                        writer.append(" ")?;
                        self.justify.append_to_rule_writer(writer)
                    }
                })
            }
        }
        impl crate::serialization::specified_rule_serialization::SpecifiedProvider for $name {
            fn append_to_rule_writer(
                &self,
                writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
                let context = &mut writer.context;
                let output = &mut writer.css;
                serialize_value_into(self.align.value(), context, output)?;
                context.append(output, " ")?;
                serialize_value_into(self.justify.value(), context, output)?;
                Ok(())
            }
        }
        crate::serialization::specified_rule_serialization::specified_serialization!($name);
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

fn prefix(
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    overflow: Option<CssOverflowPosition>,
) -> SerializationResult<()> {
    if let Some(overflow) = overflow {
        context.keyword(
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
            context.keyword(output, "normal")
        }
        V::Auto => context.keyword(output, "auto"),
        V::Stretch => context.keyword(output, "stretch"),
        V::Baseline(CssBaselinePosition::Baseline | CssBaselinePosition::First) => {
            context.keyword(output, "baseline")
        }
        V::Baseline(CssBaselinePosition::Last) => {
            context.keyword(output, "last")?;
            context.append(output, " ")?;
            context.keyword(output, "baseline")
        }
        V::Position { overflow, position } => {
            prefix(context, output, overflow)?;
            context.keyword(
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
        V::SpaceBetween => context.keyword(output, "space-between"),
        V::SpaceAround => context.keyword(output, "space-around"),
        V::SpaceEvenly => context.keyword(output, "space-evenly"),
        V::Legacy(direction) => {
            context.keyword(output, "legacy")?;
            if let Some(direction) = direction {
                context.append(output, " ")?;
                context.keyword(
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
