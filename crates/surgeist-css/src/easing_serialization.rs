//! CSS Easing Functions Level 1 specified serialization, before timing evaluation.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssCubicBezier, CssEasing, CssEasingKeyword, CssEasingList,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits, CssStepPosition,
    CssSteps,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! specified_methods {
    () => {
        /// Serializes the authored easing value using CSS Easing Level 1 canonical syntax.
        /// Calculations retain specified-value semantics without timing evaluation or clamping.
        pub fn serialize_specified(&self) -> Result<String> {
            self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
        }

        /// Serializes atomically under cumulative input, projection, and CSS byte limits.
        /// Aggregates and their numeric children share one serialization context.
        pub fn serialize_specified_with_limits(
            &self,
            limits: CssSpecifiedValueSerializationLimits,
        ) -> Result<String> {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut output = String::new();
            self.append_specified(&mut context, &mut output)?;
            Ok(output)
        }
    };
}

impl CssEasing {
    specified_methods!();

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Keyword(keyword) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
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
        }
    }
}

impl CssCubicBezier {
    specified_methods!();

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
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

impl CssSteps {
    specified_methods!();

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
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

impl CssEasingList {
    specified_methods!();

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        for (index, value) in self.values().iter().enumerate() {
            if index != 0 {
                context.append(output, ", ")?;
            }
            value.append_specified(context, output)?;
        }
        Ok(())
    }
}
