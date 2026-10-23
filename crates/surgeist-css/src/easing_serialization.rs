//! Authored easing specified serialization, before timing evaluation.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssCubicBezier, CssEasing, CssEasingKeyword, CssEasingList, CssLinearEasing, CssLinearStop,
    CssSerializationError, CssStepPosition, CssSteps,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

macro_rules! specified_methods {
    ($ty:ty) => {
        impl crate::specified_rule_serialization::SpecifiedProvider for $ty {
            fn append_to_rule_writer(
                &self,
                writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> std::result::Result<(), crate::CssSerializationError> {
                let context = &mut writer.context;
                let output = &mut writer.css;
                self.append_specified(context, output)?;
                Ok(())
            }
        }
        crate::specified_rule_serialization::specified_serialization!($ty);
    };
}

impl CssEasing {
    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Keyword(keyword) => {
                context.charge_nodes(1)?;
                context.append(
                    output,
                    match keyword {
                        CssEasingKeyword::Linear => "linear",
                        CssEasingKeyword::Ease => "ease",
                        CssEasingKeyword::EaseIn => "ease-in",
                        CssEasingKeyword::EaseOut => "ease-out",
                        CssEasingKeyword::EaseInOut => "ease-in-out",
                        CssEasingKeyword::StepStart => "steps(1, start)",
                        CssEasingKeyword::StepEnd => "steps(1)",
                    },
                )
            }
            Self::CubicBezier(value) => value.append_specified(context, output),
            Self::Steps(value) => value.append_specified(context, output),
            Self::Linear(value) => value.append_specified(context, output),
        }
    }
}
specified_methods!(CssEasing);

impl CssCubicBezier {
    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_nodes(1)?;
        context.append(output, "cubic-bezier(")?;
        for (index, value) in [self.x1().value(), self.y1(), self.x2().value(), self.y2()]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                context.append(output, ", ")?;
            }
            let captured = value.capture_specified(context)?;
            context.append(output, &captured)?;
        }
        context.append(output, ")")
    }
}
specified_methods!(CssCubicBezier);

impl CssLinearEasing {
    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_nodes(1)?;
        context.append(output, "linear(")?;
        for (index, stop) in self.stops().iter().enumerate() {
            if index != 0 {
                context.append(output, ", ")?;
            }
            stop.append_specified(context, output)?;
        }
        context.append(output, ")")
    }
}
specified_methods!(CssLinearEasing);

impl CssLinearStop {
    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        // The authored stop is one node; scalar providers retain their own work.
        context.charge_nodes(1)?;
        let captured = self.output().capture_specified(context)?;
        context.append(output, &captured)?;
        for input in self.inputs() {
            context.append(output, " ")?;
            let captured = input.capture_specified(context)?;
            context.append(output, &captured)?;
        }
        Ok(())
    }
}
specified_methods!(CssLinearStop);

impl CssSteps {
    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_nodes(1)?;
        context.append(output, "steps(")?;
        self.count().serialize_specified_into(context, output)?;
        if let Some(position) = self.position() {
            context.charge_input(1)?;
            let keyword = match position {
                CssStepPosition::End | CssStepPosition::JumpEnd => None,
                CssStepPosition::Start => Some("start"),
                CssStepPosition::JumpStart => Some("jump-start"),
                CssStepPosition::JumpNone => Some("jump-none"),
                CssStepPosition::JumpBoth => Some("jump-both"),
            };
            if let Some(keyword) = keyword {
                context.charge_projection(1)?;
                context.append(output, ", ")?;
                context.append(output, keyword)?;
            }
        }
        context.append(output, ")")
    }
}
specified_methods!(CssSteps);

impl CssEasingList {
    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_nodes(1)?;
        for (index, value) in self.values().iter().enumerate() {
            if index != 0 {
                context.append(output, ", ")?;
            }
            value.append_specified(context, output)?;
        }
        Ok(())
    }
}
specified_methods!(CssEasingList);
