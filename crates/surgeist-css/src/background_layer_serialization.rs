//! Bounded specified text for Backgrounds 3 size, repeat, box and attachment layers.

use crate::{
    CssBackgroundAttachment, CssBackgroundAttachmentList, CssBackgroundBox, CssBackgroundBoxList,
    CssBackgroundRepeat, CssBackgroundRepeatList, CssBackgroundRepeatStyle, CssBackgroundSize,
    CssBackgroundSizeComponent, CssBackgroundSizeList, CssSerializationError,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

macro_rules! scalar_serialization {
    ($($owner:ty),* $(,)?) => {$(
        impl crate::specified_rule_serialization::SpecifiedProvider for $owner {
            fn append_to_rule_writer(
                &self,
                writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> std::result::Result<(), crate::CssSerializationError> {
                self.append_specified(writer)
            }
        }
        crate::specified_rule_serialization::specified_serialization!($owner);
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
        impl crate::specified_rule_serialization::SpecifiedProvider for $owner {
            fn append_to_rule_writer(
                &self,
                writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> std::result::Result<(), crate::CssSerializationError> {
                writer.node()?;
                for (index, value) in self.$accessor().iter().enumerate() {
                    if index != 0 { writer.append(", ")?; }
                    value.append_specified(writer)?;
                }
                Ok(())
            }
        }
        crate::specified_rule_serialization::specified_serialization!($owner);
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
        writer.node()?;
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
                        writer.node()?;
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
    writer.node()?;
    match value {
        CssBackgroundSizeComponent::Auto => writer.append("auto"),
        CssBackgroundSizeComponent::Length(value) => {
            value.append_specified(&mut writer.context, &mut writer.css)
        }
    }
}

impl CssBackgroundRepeat {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        match self {
            Self::RepeatX => writer.append("repeat-x"),
            Self::RepeatY => writer.append("repeat-y"),
            Self::Axes { x, y } => {
                writer.nodes(2)?;
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
        writer.node()?;
        writer.append(match self {
            Self::BorderBox => "border-box",
            Self::PaddingBox => "padding-box",
            Self::ContentBox => "content-box",
        })
    }
}

impl CssBackgroundAttachment {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append(match self {
            Self::Scroll => "scroll",
            Self::Fixed => "fixed",
            Self::Local => "local",
        })
    }
}
