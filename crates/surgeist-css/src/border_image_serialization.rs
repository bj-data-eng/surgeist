//! Canonical specified CSS Backgrounds 3 border-image values, before contextual resolution.

use crate::{
    CssBorderImage, CssBorderImageOutset, CssBorderImageOutsetComponent, CssBorderImageRepeat,
    CssBorderImageRepeatKeyword, CssBorderImageSlice, CssBorderImageSliceComponent,
    CssBorderImageWidth, CssBorderImageWidthComponent, CssComponentValue, CssComponentValueRef,
    CssImageValue, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    CssValueTokenRef,
    exact_decimal::LexicalDecimal,
    numeric::{CapturedNumericComponent, length_components_equal, numeric_components_equal},
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn charge(writer: &mut SpecifiedRuleWriter, count: usize) -> Result<()> {
    writer.context.charge_input(count)?;
    writer.context.charge_projection(count)
}

macro_rules! serialization {
    ($($owner:ty),* $(,)?) => {$(
        impl $owner {
            /// Serializes canonical specified CSS without resolving contextual values.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }

            /// Shares cumulative input, projection and final byte limits across all children.
            /// Every stored edge is visited, even when compressed or omitted. Failure returns
            /// no partial text and leaves the authored input unchanged.
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

serialization!(
    CssBorderImage,
    CssBorderImageSlice,
    CssBorderImageWidth,
    CssBorderImageOutset,
    CssBorderImageRepeat,
);

// Suppression is entered only after proving an ordinary literal initial. The
// provider still visits the token, without formatting discarded numeric text.
macro_rules! capture_numeric {
    ($value:expr, $writer:expr) => {{
        let value = $value;
        let writer = $writer;
        if writer.context.output_suppressed() {
            value.append_specified(&mut writer.context, &mut writer.css)?;
            Ok(CapturedNumericComponent::Plain(String::new()))
        } else if let Some(calculation) = value.calculation() {
            CapturedNumericComponent::capture_calculation(
                &calculation.expression,
                &mut writer.context,
            )
        } else {
            value
                .capture_specified(&mut writer.context)
                .map(CapturedNumericComponent::Plain)
        }
    }};
}

fn append_sides<T>(
    values: &[T; 4],
    writer: &mut SpecifiedRuleWriter,
    capture: impl Fn(&T, &mut SpecifiedRuleWriter) -> Result<CapturedNumericComponent>,
    component_equal: impl Fn(&T, &T, &CapturedNumericComponent, &CapturedNumericComponent) -> bool,
) -> Result<()> {
    // Capture before group output, preserving every visit, scratch limit and
    // failure order. Ordinary compression compares exact component meaning;
    // calculations compare canonical syntax and unrounded finite components.
    let css = [
        capture(&values[0], writer)?,
        capture(&values[1], writer)?,
        capture(&values[2], writer)?,
        capture(&values[3], writer)?,
    ];
    let equal = |left: usize, right: usize| {
        std::mem::discriminant(&values[left]) == std::mem::discriminant(&values[right])
            && component_equal(&values[left], &values[right], &css[left], &css[right])
    };
    let count = if equal(0, 1) && equal(0, 2) && equal(0, 3) {
        1
    } else if equal(0, 2) && equal(1, 3) {
        2
    } else if equal(1, 3) {
        3
    } else {
        4
    };
    for (index, text) in css[..count].iter().enumerate() {
        if index != 0 {
            writer.append(" ")?;
        }
        writer.append(text.as_css())?;
    }
    Ok(())
}

fn slice_equal(
    left: &CssBorderImageSliceComponent,
    right: &CssBorderImageSliceComponent,
    left_css: &CapturedNumericComponent,
    right_css: &CapturedNumericComponent,
) -> bool {
    match (left, right) {
        (
            CssBorderImageSliceComponent::HintedNumberCalculation(_),
            CssBorderImageSliceComponent::HintedNumberCalculation(_),
        ) => numeric_components_equal(None, None, left_css, right_css),
        (
            CssBorderImageSliceComponent::Number(left),
            CssBorderImageSliceComponent::Number(right),
        ) => numeric_components_equal(
            left.literal_component(),
            right.literal_component(),
            left_css,
            right_css,
        ),
        (
            CssBorderImageSliceComponent::Percentage(left),
            CssBorderImageSliceComponent::Percentage(right),
        ) => numeric_components_equal(
            left.literal_component(),
            right.literal_component(),
            left_css,
            right_css,
        ),
        _ => false,
    }
}

fn width_equal(
    left: &CssBorderImageWidthComponent,
    right: &CssBorderImageWidthComponent,
    left_css: &CapturedNumericComponent,
    right_css: &CapturedNumericComponent,
) -> bool {
    match (left, right) {
        (
            CssBorderImageWidthComponent::HintedNumberCalculation(_),
            CssBorderImageWidthComponent::HintedNumberCalculation(_),
        ) => numeric_components_equal(None, None, left_css, right_css),
        (CssBorderImageWidthComponent::Auto, CssBorderImageWidthComponent::Auto) => true,
        (
            CssBorderImageWidthComponent::Number(left),
            CssBorderImageWidthComponent::Number(right),
        ) => numeric_components_equal(
            left.literal_component(),
            right.literal_component(),
            left_css,
            right_css,
        ),
        (
            CssBorderImageWidthComponent::LengthPercentage(left),
            CssBorderImageWidthComponent::LengthPercentage(right),
        ) => length_components_equal(
            left.literal_component(),
            right.literal_component(),
            left_css,
            right_css,
        ),
        _ => false,
    }
}

fn outset_equal(
    left: &CssBorderImageOutsetComponent,
    right: &CssBorderImageOutsetComponent,
    left_css: &CapturedNumericComponent,
    right_css: &CapturedNumericComponent,
) -> bool {
    match (left, right) {
        (
            CssBorderImageOutsetComponent::Number(left),
            CssBorderImageOutsetComponent::Number(right),
        ) => numeric_components_equal(
            left.literal_component(),
            right.literal_component(),
            left_css,
            right_css,
        ),
        (
            CssBorderImageOutsetComponent::Length(left),
            CssBorderImageOutsetComponent::Length(right),
        ) => length_components_equal(
            left.literal_component(),
            right.literal_component(),
            left_css,
            right_css,
        ),
        _ => false,
    }
}

fn capture_slice(
    value: &CssBorderImageSliceComponent,
    writer: &mut SpecifiedRuleWriter,
) -> Result<CapturedNumericComponent> {
    charge(writer, 1)?;
    match value {
        CssBorderImageSliceComponent::HintedNumberCalculation(value) => {
            CapturedNumericComponent::capture_calculation(&value.expression, &mut writer.context)
        }
        CssBorderImageSliceComponent::Number(value) => capture_numeric!(value, writer),
        CssBorderImageSliceComponent::Percentage(value) => capture_numeric!(value, writer),
    }
}

fn unitless_length_zero(component: Option<&CssComponentValue>) -> bool {
    matches!(component.map(CssComponentValue::view),
        Some(CssComponentValueRef::Token(CssValueTokenRef::Number(number)))
            if LexicalDecimal::new(number.representation()).len == 0)
}

fn capture_width(
    value: &CssBorderImageWidthComponent,
    writer: &mut SpecifiedRuleWriter,
) -> Result<CapturedNumericComponent> {
    charge(writer, 1)?;
    match value {
        CssBorderImageWidthComponent::HintedNumberCalculation(value) => {
            CapturedNumericComponent::capture_calculation(&value.expression, &mut writer.context)
        }
        CssBorderImageWidthComponent::Auto => {
            let mut css = String::new();
            if !writer.context.output_suppressed() {
                writer.context.append_temporary(&mut css, "auto")?;
            }
            Ok(CapturedNumericComponent::Plain(css))
        }
        CssBorderImageWidthComponent::Number(value) => capture_numeric!(value, writer),
        CssBorderImageWidthComponent::LengthPercentage(value) => {
            let mut css = capture_numeric!(value, &mut *writer)?;
            if !writer.context.output_suppressed()
                && unitless_length_zero(value.literal_component())
            {
                // This slot also accepts numbers. Preserve its length branch on
                // grammar reentry; the suffix belongs to the same numeric token.
                let CapturedNumericComponent::Plain(text) = &mut css else {
                    unreachable!("ordinary unitless zero has plain capture")
                };
                writer.context.append_temporary(text, "px")?;
            }
            Ok(css)
        }
    }
}

fn capture_outset(
    value: &CssBorderImageOutsetComponent,
    writer: &mut SpecifiedRuleWriter,
) -> Result<CapturedNumericComponent> {
    charge(writer, 1)?;
    match value {
        CssBorderImageOutsetComponent::Number(value) => capture_numeric!(value, writer),
        CssBorderImageOutsetComponent::Length(value) => {
            let mut css = capture_numeric!(value, &mut *writer)?;
            if !writer.context.output_suppressed()
                && unitless_length_zero(value.literal_component())
            {
                let CapturedNumericComponent::Plain(text) = &mut css else {
                    unreachable!("ordinary unitless zero has plain capture")
                };
                writer.context.append_temporary(text, "px")?;
            }
            Ok(css)
        }
    }
}

impl CssBorderImageSlice {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        append_sides(self.values(), writer, capture_slice, slice_equal)?;
        if self.fill() {
            charge(writer, 1)?;
            writer.append(" fill")?;
        }
        Ok(())
    }
}

impl CssBorderImageWidth {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        append_sides(self.values(), writer, capture_width, width_equal)
    }
}

impl CssBorderImageOutset {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        append_sides(self.values(), writer, capture_outset, outset_equal)
    }
}

fn repeat_keyword(value: CssBorderImageRepeatKeyword) -> &'static str {
    match value {
        CssBorderImageRepeatKeyword::Stretch => "stretch",
        CssBorderImageRepeatKeyword::Repeat => "repeat",
        CssBorderImageRepeatKeyword::Round => "round",
        CssBorderImageRepeatKeyword::Space => "space",
    }
}

impl CssBorderImageRepeat {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 3)?;
        writer.append(repeat_keyword(self.horizontal()))?;
        if self.horizontal() != self.vertical() {
            writer.append(" ")?;
            writer.append(repeat_keyword(self.vertical()))?;
        }
        Ok(())
    }
}

fn literal_initial(component: Option<&CssComponentValue>, exponent: Option<i128>) -> bool {
    let Some(CssComponentValueRef::Token(
        CssValueTokenRef::Number(number) | CssValueTokenRef::Percentage(number),
    )) = component.map(CssComponentValue::view)
    else {
        return false;
    };
    let decimal = LexicalDecimal::new(number.representation());
    match exponent {
        None => decimal.len == 0,
        Some(exponent) => {
            !decimal.negative
                && decimal.len == 1
                && decimal.digits().next() == Some(1)
                && decimal.exponent == Some(exponent)
        }
    }
}

fn initial_slice(value: &CssBorderImageSlice) -> bool {
    !value.fill()
        && value.values().iter().all(|value| {
            matches!(value,
            CssBorderImageSliceComponent::Percentage(number)
                if literal_initial(number.literal_component(), Some(2)))
        })
}

fn initial_width(value: &CssBorderImageWidth) -> bool {
    value.values().iter().all(|value| {
        matches!(value,
        CssBorderImageWidthComponent::Number(number)
            if literal_initial(number.literal_component(), Some(0)))
    })
}

fn initial_outset(value: &CssBorderImageOutset) -> bool {
    value.values().iter().all(|value| {
        matches!(value,
        CssBorderImageOutsetComponent::Number(number)
            if literal_initial(number.literal_component(), None))
    })
}

fn before_component(writer: &mut SpecifiedRuleWriter, emitted: &mut bool) -> Result<()> {
    if *emitted {
        writer.append(" ")?;
    }
    *emitted = true;
    Ok(())
}

impl CssBorderImage {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        let mut emitted = false;
        if let Some(source) = self.source() {
            writer.source_member(0, |writer| {
                if matches!(source, CssImageValue::None) {
                    writer.without_output(|writer| source.append_specified(writer))?;
                } else {
                    before_component(writer, &mut emitted)?;
                    source.append_specified(writer)?;
                }
                Ok(())
            })?;
        }
        let retained_width = self.width().is_some_and(|value| !initial_width(value));
        let retained_outset = self.outset().is_some_and(|value| !initial_outset(value));
        if let Some(slice) = self.slice() {
            writer.source_member(1, |writer| {
                if !retained_width && !retained_outset && initial_slice(slice) {
                    writer.without_output(|writer| slice.append_specified(writer))?;
                } else {
                    before_component(writer, &mut emitted)?;
                    slice.append_specified(writer)?;
                }
                Ok(())
            })?;
        }
        if let Some(width) = self.width() {
            writer.source_member(2, |writer| {
                if retained_width {
                    writer.append(" / ")?;
                    width.append_specified(writer)?;
                } else {
                    writer.without_output(|writer| width.append_specified(writer))?;
                }
                Ok(())
            })?;
        }
        if let Some(outset) = self.outset() {
            writer.source_member(3, |writer| {
                if retained_outset {
                    writer.append(if retained_width { " / " } else { " / / " })?;
                    outset.append_specified(writer)?;
                } else {
                    writer.without_output(|writer| outset.append_specified(writer))?;
                }
                Ok(())
            })?;
        }
        if let Some(repeat) = self.repeat() {
            writer.source_member(4, |writer| {
                if repeat.horizontal() == CssBorderImageRepeatKeyword::Stretch
                    && repeat.vertical() == CssBorderImageRepeatKeyword::Stretch
                {
                    writer.without_output(|writer| repeat.append_specified(writer))?;
                } else {
                    before_component(writer, &mut emitted)?;
                    repeat.append_specified(writer)?;
                }
                Ok(())
            })?;
        }
        if !emitted {
            writer.source_member(0, |writer| {
                writer.context.charge_projection(1)?;
                writer.append("none")
            })?;
        }
        Ok(())
    }
}
