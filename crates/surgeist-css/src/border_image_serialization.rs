//! Canonical specified CSS Backgrounds 3 border-image values, before contextual resolution.

use crate::{
    CssBorderImage, CssBorderImageOutset, CssBorderImageOutsetComponent, CssBorderImageRepeat,
    CssBorderImageRepeatKeyword, CssBorderImageSlice, CssBorderImageSliceComponent,
    CssBorderImageWidth, CssBorderImageWidthComponent, CssComponentValue, CssComponentValueRef,
    CssImageValue, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    CssValueTokenRef, exact_decimal::LexicalDecimal,
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
                let mut writer = SpecifiedRuleWriter::new(limits);
                self.append_specified(&mut writer)?;
                Ok(writer.css)
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
            Ok(String::new())
        } else {
            value.capture_specified(&mut writer.context)
        }
    }};
}

fn append_sides<T>(
    values: &[T; 4],
    writer: &mut SpecifiedRuleWriter,
    capture: impl Fn(&T, &mut SpecifiedRuleWriter) -> Result<String>,
) -> Result<()> {
    // Capture before group output: every distinct retained text must occur in
    // the compressed result, so each scratch value fits remaining final space.
    // Four captures retain all visits and their cumulative projection costs.
    let css = [
        capture(&values[0], writer)?,
        capture(&values[1], writer)?,
        capture(&values[2], writer)?,
        capture(&values[3], writer)?,
    ];
    let equal = |left: usize, right: usize| {
        std::mem::discriminant(&values[left]) == std::mem::discriminant(&values[right])
            && css[left] == css[right]
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
        writer.append(text)?;
    }
    Ok(())
}

fn capture_slice(
    value: &CssBorderImageSliceComponent,
    writer: &mut SpecifiedRuleWriter,
) -> Result<String> {
    charge(writer, 1)?;
    match value {
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
) -> Result<String> {
    charge(writer, 1)?;
    match value {
        CssBorderImageWidthComponent::Auto => {
            let mut css = String::new();
            if !writer.context.output_suppressed() {
                writer.context.append_temporary(&mut css, "auto")?;
            }
            Ok(css)
        }
        CssBorderImageWidthComponent::Number(value) => capture_numeric!(value, writer),
        CssBorderImageWidthComponent::LengthPercentage(value) => {
            let mut css = capture_numeric!(value, &mut *writer)?;
            if !writer.context.output_suppressed()
                && unitless_length_zero(value.literal_component())
            {
                // This slot also accepts numbers. Preserve its length branch on
                // grammar reentry; the suffix belongs to the same numeric token.
                writer.context.append_temporary(&mut css, "px")?;
            }
            Ok(css)
        }
    }
}

fn capture_outset(
    value: &CssBorderImageOutsetComponent,
    writer: &mut SpecifiedRuleWriter,
) -> Result<String> {
    charge(writer, 1)?;
    match value {
        CssBorderImageOutsetComponent::Number(value) => capture_numeric!(value, writer),
        CssBorderImageOutsetComponent::Length(value) => {
            let mut css = capture_numeric!(value, &mut *writer)?;
            if !writer.context.output_suppressed()
                && unitless_length_zero(value.literal_component())
            {
                writer.context.append_temporary(&mut css, "px")?;
            }
            Ok(css)
        }
    }
}

impl CssBorderImageSlice {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        append_sides(self.values(), writer, capture_slice)?;
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
        append_sides(self.values(), writer, capture_width)
    }
}

impl CssBorderImageOutset {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        append_sides(self.values(), writer, capture_outset)
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
            if matches!(source, CssImageValue::None) {
                writer.without_output(|writer| source.append_specified(writer))?;
            } else {
                before_component(writer, &mut emitted)?;
                source.append_specified(writer)?;
            }
        }
        let retained_width = self.width().is_some_and(|value| !initial_width(value));
        let retained_outset = self.outset().is_some_and(|value| !initial_outset(value));
        if let Some(slice) = self.slice() {
            if !retained_width && !retained_outset && initial_slice(slice) {
                writer.without_output(|writer| slice.append_specified(writer))?;
            } else {
                before_component(writer, &mut emitted)?;
                slice.append_specified(writer)?;
            }
        }
        if let Some(width) = self.width() {
            if retained_width {
                writer.append(" / ")?;
                width.append_specified(writer)?;
            } else {
                writer.without_output(|writer| width.append_specified(writer))?;
            }
        }
        if let Some(outset) = self.outset() {
            if retained_outset {
                writer.append(if retained_width { " / " } else { " / / " })?;
                outset.append_specified(writer)?;
            } else {
                writer.without_output(|writer| outset.append_specified(writer))?;
            }
        }
        if let Some(repeat) = self.repeat() {
            if repeat.horizontal() == CssBorderImageRepeatKeyword::Stretch
                && repeat.vertical() == CssBorderImageRepeatKeyword::Stretch
            {
                writer.without_output(|writer| repeat.append_specified(writer))?;
            } else {
                before_component(writer, &mut emitted)?;
                repeat.append_specified(writer)?;
            }
        }
        if !emitted {
            writer.context.charge_projection(1)?;
            writer.append("none")?;
        }
        Ok(())
    }
}
