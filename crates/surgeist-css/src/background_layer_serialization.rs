//! Bounded specified text for Backgrounds 3 size, repeat, box and attachment layers.

use crate::{
    CssBackgroundAttachment, CssBackgroundAttachmentList, CssBackgroundBox, CssBackgroundBoxList,
    CssBackgroundRepeat, CssBackgroundRepeatList, CssBackgroundRepeatStyle, CssBackgroundSize,
    CssBackgroundSizeComponent, CssBackgroundSizeList, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn charge(writer: &mut SpecifiedRuleWriter, nodes: usize) -> Result<()> {
    writer.context.charge_input(nodes)?;
    writer.context.charge_projection(nodes)
}

macro_rules! scalar_serialization {
    ($($owner:ty),* $(,)?) => {$(
        impl $owner {
            /// Serializes one layer to canonical specified CSS without resolving context.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }

            /// Uses one cumulative input, projection and byte budget, returning no partial text.
            pub fn serialize_specified_with_limits(
                &self,
                limits: CssSpecifiedValueSerializationLimits,
            ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {


                self.append_specified(writer)?;
                Ok(())

    }
        }
    )*};
}

scalar_serialization!(
    CssBackgroundSize,
    CssBackgroundRepeat,
    CssBackgroundBox,
    CssBackgroundAttachment
);

macro_rules! list_serialization {
    ($($owner:ty => $accessor:ident),* $(,)?) => {$(
        impl $owner {
            /// Serializes the nonempty authored layers in comma order.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }

            /// Shares one input, projection and byte budget across every layer and numeric child.
            /// Failure returns no partial text and leaves the authored list unchanged.
            pub fn serialize_specified_with_limits(
                &self,
                limits: CssSpecifiedValueSerializationLimits,
            ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {


                charge(writer, 1)?;
                for (index, value) in self.$accessor().iter().enumerate() {
                    if index != 0 { writer.append(", ")?; }
                    value.append_specified(writer)?;
                }
                Ok(())

    }
        }
    )*};
}

list_serialization!(
    CssBackgroundSizeList => sizes,
    CssBackgroundRepeatList => repeats,
    CssBackgroundBoxList => boxes,
    CssBackgroundAttachmentList => attachments,
);

impl CssBackgroundSize {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        match self {
            Self::Cover => writer.append("cover"),
            Self::Contain => writer.append("contain"),
            Self::Explicit { width, height } => {
                append_size_component(width, writer)?;
                match height {
                    Some(CssBackgroundSizeComponent::Auto)
                        if matches!(width, CssBackgroundSizeComponent::Auto) =>
                    {
                        // Only auto/auto collapses; the authored height still counts.
                        charge(writer, 1)?;
                    }
                    Some(height) => {
                        writer.append(" ")?;
                        append_size_component(height, writer)?;
                    }
                    None if !matches!(width, CssBackgroundSizeComponent::Auto) => {
                        // The effective default is generated output, not authored input.
                        writer.context.charge_projection(1)?;
                        writer.append(" auto")?;
                    }
                    None => {}
                }
                Ok(())
            }
        }
    }
}

fn append_size_component(
    value: &CssBackgroundSizeComponent,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    charge(writer, 1)?;
    match value {
        CssBackgroundSizeComponent::Auto => writer.append("auto"),
        CssBackgroundSizeComponent::Length(value) => {
            value.append_specified(&mut writer.context, &mut writer.css)
        }
    }
}

impl CssBackgroundRepeat {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        match self {
            Self::RepeatX => writer.append("repeat-x"),
            Self::RepeatY => writer.append("repeat-y"),
            Self::Axes { x, y } => {
                charge(writer, 2)?;
                if x == y {
                    writer.append(repeat_style(*x))
                } else if *x == CssBackgroundRepeatStyle::Repeat
                    && *y == CssBackgroundRepeatStyle::NoRepeat
                {
                    writer.append("repeat-x")
                } else if *x == CssBackgroundRepeatStyle::NoRepeat
                    && *y == CssBackgroundRepeatStyle::Repeat
                {
                    writer.append("repeat-y")
                } else {
                    writer.append(repeat_style(*x))?;
                    writer.append(" ")?;
                    writer.append(repeat_style(*y))
                }
            }
        }
    }
}

fn repeat_style(style: CssBackgroundRepeatStyle) -> &'static str {
    match style {
        CssBackgroundRepeatStyle::Repeat => "repeat",
        CssBackgroundRepeatStyle::Space => "space",
        CssBackgroundRepeatStyle::Round => "round",
        CssBackgroundRepeatStyle::NoRepeat => "no-repeat",
    }
}

impl CssBackgroundBox {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        writer.append(match self {
            Self::BorderBox => "border-box",
            Self::PaddingBox => "padding-box",
            Self::ContentBox => "content-box",
        })
    }
}

impl CssBackgroundAttachment {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        writer.append(match self {
            Self::Scroll => "scroll",
            Self::Fixed => "fixed",
            Self::Local => "local",
        })
    }
}
