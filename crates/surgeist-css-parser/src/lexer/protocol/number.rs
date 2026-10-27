//! The canonical CSS Syntax consume-a-number spelling continuation.
//! Decimal conversion and unit/type admission are separate owners.

/// One piece of a number's spelling that a [`NumberStep::Consume`] took.
///
/// The set is closed: it mirrors the parts of a number in section 4.3.13, and
/// a new edition of the specification would be a deliberate breaking change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumberPart {
    /// The leading sign.
    Sign,
    /// One mantissa digit with its value (0 to 9), integer or fraction.
    Mantissa(u8),
    /// The decimal point that introduces a fraction.
    Point,
    /// The exponent marker, U+0065 (e) or U+0045 (E).
    Exponent,
    /// The sign of the exponent.
    ExponentSign,
    /// One exponent digit with its value (0 to 9).
    ExponentDigit(u8),
}

/// The spelling counts of a complete number, fixed once the number ends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumberLayout {
    /// Whether the number's sign was U+002D HYPHEN-MINUS.
    pub negative: bool,
    /// The count of mantissa digits, integer and fraction together.
    pub mantissa_digits: usize,
    /// The count of those digits after the decimal point.
    pub fraction_digits: usize,
    /// Whether the exponent's sign was U+002D HYPHEN-MINUS.
    pub exponent_negative: bool,
    /// The count of exponent digits; zero when there is no exponent.
    pub exponent_digits: usize,
}

/// One transition of a [`NumberCursor`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumberStep {
    /// The caller consumes one input code point, which is this part of the
    /// number.
    Consume(NumberPart),
    /// A fixed grammar transition that consumes nothing.
    Transition,
    /// The number ended before the first unconsumed code point.
    Done(NumberLayout),
}

/// A spelling counter does not fit `usize`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumberOverflow;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Sign,
    Integer,
    Point,
    Fraction,
    Exponent,
    ExponentSign,
    ExponentDigits,
    Done,
}

/// The finite state of consuming one number's spelling (section 4.3.13),
/// driven by a window of the next three code points.
#[derive(Clone, Copy, Debug)]
pub struct NumberCursor {
    state: State,
    layout: NumberLayout,
}
impl Default for NumberCursor {
    fn default() -> Self {
        Self::new()
    }
}
impl NumberCursor {
    /// Creates a cursor before the number's first code point.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: State::Sign,
            layout: NumberLayout {
                negative: false,
                mantissa_digits: 0,
                fraction_digits: 0,
                exponent_negative: false,
                exponent_digits: 0,
            },
        }
    }

    /// One fixed grammar transition over the next three code points (`None`
    /// is the EOF code point). Only a [`NumberStep::Consume`] advances input.
    /// Counters are checked before mutation; overflow retains the old state.
    ///
    /// # Errors
    ///
    /// [`NumberOverflow`] when a digit counter would exceed `usize`.
    pub fn step(&mut self, [a, b, c]: [Option<char>; 3]) -> Result<NumberStep, NumberOverflow> {
        let digit = a
            .and_then(|value| value.to_digit(10).filter(|_| value.is_ascii_digit()))
            .map(|value| value as u8);
        Ok(match self.state {
            State::Sign => {
                self.state = State::Integer;
                if matches!(a, Some('+' | '-')) {
                    self.layout.negative = a == Some('-');
                    NumberStep::Consume(NumberPart::Sign)
                } else {
                    NumberStep::Transition
                }
            }
            State::Integer | State::Fraction if digit.is_some() => {
                let mantissa = self
                    .layout
                    .mantissa_digits
                    .checked_add(1)
                    .ok_or(NumberOverflow)?;
                let fraction = if self.state == State::Fraction {
                    self.layout
                        .fraction_digits
                        .checked_add(1)
                        .ok_or(NumberOverflow)?
                } else {
                    self.layout.fraction_digits
                };
                self.layout.mantissa_digits = mantissa;
                self.layout.fraction_digits = fraction;
                NumberStep::Consume(NumberPart::Mantissa(digit.expect("fixed digit check")))
            }
            State::Integer => {
                self.state = State::Point;
                NumberStep::Transition
            }
            State::Point => {
                if a == Some('.') && b.is_some_and(|value| value.is_ascii_digit()) {
                    self.state = State::Fraction;
                    NumberStep::Consume(NumberPart::Point)
                } else {
                    self.state = State::Exponent;
                    NumberStep::Transition
                }
            }
            State::Fraction => {
                self.state = State::Exponent;
                NumberStep::Transition
            }
            State::Exponent => {
                if matches!(a, Some('e' | 'E'))
                    && (b.is_some_and(|value| value.is_ascii_digit())
                        || matches!(b, Some('+' | '-'))
                            && c.is_some_and(|value| value.is_ascii_digit()))
                {
                    self.state = State::ExponentSign;
                    NumberStep::Consume(NumberPart::Exponent)
                } else {
                    self.state = State::Done;
                    NumberStep::Done(self.layout)
                }
            }
            State::ExponentSign => {
                self.state = State::ExponentDigits;
                if matches!(a, Some('+' | '-')) {
                    self.layout.exponent_negative = a == Some('-');
                    NumberStep::Consume(NumberPart::ExponentSign)
                } else {
                    NumberStep::Transition
                }
            }
            State::ExponentDigits if digit.is_some() => {
                self.layout.exponent_digits = self
                    .layout
                    .exponent_digits
                    .checked_add(1)
                    .ok_or(NumberOverflow)?;
                NumberStep::Consume(NumberPart::ExponentDigit(digit.expect("fixed digit check")))
            }
            State::ExponentDigits => {
                self.state = State::Done;
                NumberStep::Done(self.layout)
            }
            State::Done => NumberStep::Done(self.layout),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_number_boundary_preserves_dimension_and_incomplete_exponent() {
        for (text, length, expected) in [
            (
                "-12.30e+04px",
                10,
                NumberLayout {
                    negative: true,
                    mantissa_digits: 4,
                    fraction_digits: 2,
                    exponent_negative: false,
                    exponent_digits: 2,
                },
            ),
            (
                ".5e-2%",
                5,
                NumberLayout {
                    negative: false,
                    mantissa_digits: 1,
                    fraction_digits: 1,
                    exponent_negative: true,
                    exponent_digits: 1,
                },
            ),
            (
                "1e+px",
                1,
                NumberLayout {
                    negative: false,
                    mantissa_digits: 1,
                    fraction_digits: 0,
                    exponent_negative: false,
                    exponent_digits: 0,
                },
            ),
            (
                "0.0.2",
                3,
                NumberLayout {
                    negative: false,
                    mantissa_digits: 2,
                    fraction_digits: 1,
                    exponent_negative: false,
                    exponent_digits: 0,
                },
            ),
        ] {
            let mut cursor = NumberCursor::new();
            let mut index = 0;
            loop {
                let window = [
                    text.as_bytes().get(index),
                    text.as_bytes().get(index + 1),
                    text.as_bytes().get(index + 2),
                ]
                .map(|value| value.copied().map(char::from));
                match cursor.step(window).unwrap() {
                    NumberStep::Consume(_) => index += 1,
                    NumberStep::Transition => {}
                    NumberStep::Done(layout) => {
                        assert_eq!(index, length, "{text}");
                        assert_eq!(layout, expected);
                        break;
                    }
                }
            }
        }
    }
}
