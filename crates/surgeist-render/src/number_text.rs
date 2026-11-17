//! The one text form of a floating-point number in a diagnostic message.
//!
//! `f64` and `f32` print through `Display` without an exponent, so `f64::MAX` takes 309 characters
//! and `f64::MIN_POSITIVE` 326. A message that carries a rejected number prints it through
//! [`Number`]: the plain form for ordinary magnitudes and the exponent form beyond them.

use std::{borrow::Cow, fmt};

/// Smallest magnitude printed in plain form; smaller non-zero magnitudes use the exponent form.
const PLAIN_MINIMUM: f64 = 1e-5;
/// Magnitude from which the exponent form is used.
const PLAIN_LIMIT: f64 = 1e16;
/// Longest run of digits that an integer type can print (`i128::MIN` has 39 digits and a sign).
const LONGEST_INTEGER_DIGITS: usize = 40;

/// A number printed in the diagnostic form.
///
/// Zero, NaN and the infinities print as `Display` prints them, a magnitude from 1e-5 up to
/// but excluding 1e16 prints in plain form, and every other magnitude prints in exponent form
/// with the shortest digits that identify the value, such as `1.7976931348623157e308`.
pub(crate) struct Number<T>(pub(crate) T);

macro_rules! number {
    ($float:ty) => {
        impl fmt::Display for Number<$float> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let magnitude = self.0.abs();
                if magnitude == 0.0
                    || !magnitude.is_finite()
                    || (magnitude >= PLAIN_MINIMUM as $float && magnitude < PLAIN_LIMIT as $float)
                {
                    fmt::Display::fmt(&self.0, f)
                } else {
                    fmt::LowerExp::fmt(&self.0, f)
                }
            }
        }
    };
}

number!(f64);
number!(f32);

/// A rectangle as `x, y, width by height`, each number printed through [`Number`].
pub(crate) struct RectText(pub(crate) crate::Rect);

impl fmt::Display for RectText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "x {}, y {}, {} by {}",
            Number(self.0.x()),
            Number(self.0.y()),
            Number(self.0.width()),
            Number(self.0.height())
        )
    }
}

/// A size as `width by height`, each number printed through [`Number`].
pub(crate) struct SizeText(pub(crate) crate::Size);

impl fmt::Display for SizeText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} by {}",
            Number(self.0.width()),
            Number(self.0.height())
        )
    }
}

/// The six affine coefficients in Kurbo order, each printed through [`Number`].
pub(crate) struct CoefficientsText(pub(crate) [f64; 6]);

impl fmt::Display for CoefficientsText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, coefficient) in self.0.iter().enumerate() {
            if index != 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", Number(*coefficient))?;
        }
        Ok(())
    }
}

/// The text of a caller-supplied value in a message.
///
/// A generic value reaches a message only as its `Display` text, so the text of a float is
/// recognised by its form: digits with a decimal point, or more digits than any integer type
/// prints. Such a text is parsed and printed through [`Number`], which does not change a value
/// of ordinary magnitude. Any other text is returned as it is; in particular an integer text is
/// never rewritten.
pub(crate) fn message_text(text: &str) -> Cow<'_, str> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let mut parts = digits.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next();
    let plain = parts.next().is_none()
        && !whole.is_empty()
        && whole.bytes().all(|byte| byte.is_ascii_digit())
        && fraction
            .is_none_or(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
    if !plain || (fraction.is_none() && whole.len() <= LONGEST_INTEGER_DIGITS) {
        return Cow::Borrowed(text);
    }
    match text.parse::<f64>() {
        Ok(value) if value.is_finite() => {
            let rendered = Number(value).to_string();
            if rendered == text {
                Cow::Borrowed(text)
            } else {
                Cow::Owned(rendered)
            }
        }
        _ => Cow::Borrowed(text),
    }
}

#[cfg(test)]
mod tests {
    use super::{Number, message_text};

    #[test]
    fn ordinary_magnitudes_print_in_plain_form() {
        for (value, text) in [
            (0.0, "0"),
            (-0.0, "-0"),
            (1.0, "1"),
            (-2.5, "-2.5"),
            (0.1, "0.1"),
            (0.00001, "0.00001"),
            (123456789.125, "123456789.125"),
            (9_999_999_999_999_998.0, "9999999999999998"),
        ] {
            assert_eq!(Number(value).to_string(), text);
        }
    }

    #[test]
    fn magnitudes_beyond_the_plain_range_print_in_exponent_form() {
        for (value, text) in [
            (1e16, "1e16"),
            (-1e16, "-1e16"),
            (f64::MAX, "1.7976931348623157e308"),
            (f64::MIN, "-1.7976931348623157e308"),
            (f64::MIN_POSITIVE, "2.2250738585072014e-308"),
            (f64::from_bits(1), "5e-324"),
            (0.000001, "1e-6"),
            (0.000009999, "9.999e-6"),
        ] {
            assert_eq!(Number(value).to_string(), text);
        }
        assert_eq!(Number(f32::MAX).to_string(), "3.4028235e38");
        assert_eq!(Number(f32::MIN_POSITIVE).to_string(), "1.1754944e-38");
        assert_eq!(Number(1e-6f32).to_string(), "1e-6");
        assert_eq!(Number(0.3f32).to_string(), "0.3");
        assert_eq!(Number(16384.0f32).to_string(), "16384");
    }

    #[test]
    fn non_finite_values_print_as_display_prints_them() {
        assert_eq!(Number(f64::NAN).to_string(), "NaN");
        assert_eq!(Number(f64::INFINITY).to_string(), "inf");
        assert_eq!(Number(f64::NEG_INFINITY).to_string(), "-inf");
        assert_eq!(Number(f32::NAN).to_string(), "NaN");
        assert_eq!(Number(f32::INFINITY).to_string(), "inf");
    }

    #[test]
    fn a_printed_float_text_is_rewritten_only_beyond_the_plain_range() {
        let max = f64::MAX.to_string();
        assert_eq!(max.len(), 309);
        assert_eq!(message_text(&max), "1.7976931348623157e308");
        let min = f64::MIN.to_string();
        assert_eq!(message_text(&min), "-1.7976931348623157e308");
        let tiny = f64::MIN_POSITIVE.to_string();
        assert_eq!(message_text(&tiny), "2.2250738585072014e-308");
        assert_eq!(message_text("0.000001"), "1e-6");
        for unchanged in [
            "0.5", "-2.5", "100", "0.00001", "NaN", "inf", "-inf", "abc", "", "-", "1.2.3", "1.",
            ".5",
        ] {
            assert_eq!(message_text(unchanged), unchanged);
        }
    }

    #[test]
    fn a_rectangle_a_size_and_coefficients_print_every_number_through_the_form() {
        use super::{CoefficientsText, RectText, SizeText};
        let rect = crate::Rect::try_new(1.5, -2.0, 1e300, 0.000001).unwrap();
        assert_eq!(RectText(rect).to_string(), "x 1.5, y -2, 1e300 by 1e-6");
        let size = crate::Size::try_new(f64::MAX, 0.25).unwrap();
        assert_eq!(SizeText(size).to_string(), "1.7976931348623157e308 by 0.25");
        assert_eq!(
            CoefficientsText([1.0, 0.0, 0.0, 1.0, f64::MIN_POSITIVE, -3.5]).to_string(),
            "1, 0, 0, 1, 2.2250738585072014e-308, -3.5"
        );
    }

    #[test]
    fn an_integer_text_is_never_rewritten() {
        for text in [
            u64::MAX.to_string(),
            i64::MIN.to_string(),
            u128::MAX.to_string(),
            i128::MIN.to_string(),
        ] {
            assert_eq!(message_text(&text), text);
        }
    }
}
