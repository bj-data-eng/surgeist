//! The canonical CSS Syntax consume-a-number spelling continuation.
//! Decimal conversion and unit/type admission are separate owners.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Part {
    Sign,
    Mantissa(u8),
    Point,
    Exponent,
    ExponentSign,
    ExponentDigit(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Layout {
    pub(crate) negative: bool,
    pub(crate) mantissa_digits: usize,
    pub(crate) fraction_digits: usize,
    pub(crate) exponent_negative: bool,
    pub(crate) exponent_digits: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Step {
    Consume(Part),
    Transition,
    Done(Layout),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Overflow;

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

#[derive(Clone, Copy)]
pub(crate) struct NumberCursor {
    state: State,
    layout: Layout,
}
impl NumberCursor {
    pub(crate) const fn new() -> Self {
        Self {
            state: State::Sign,
            layout: Layout {
                negative: false,
                mantissa_digits: 0,
                fraction_digits: 0,
                exponent_negative: false,
                exponent_digits: 0,
            },
        }
    }

    /// One fixed grammar transition. Only a Consume step advances input.
    /// Counters are checked before mutation; overflow retains the old state.
    pub(crate) fn step(&mut self, [a, b, c]: [Option<char>; 3]) -> Result<Step, Overflow> {
        let digit = a
            .and_then(|value| value.to_digit(10).filter(|_| value.is_ascii_digit()))
            .map(|value| value as u8);
        Ok(match self.state {
            State::Sign => {
                self.state = State::Integer;
                if matches!(a, Some('+' | '-')) {
                    self.layout.negative = a == Some('-');
                    Step::Consume(Part::Sign)
                } else {
                    Step::Transition
                }
            }
            State::Integer | State::Fraction if digit.is_some() => {
                let mantissa = self.layout.mantissa_digits.checked_add(1).ok_or(Overflow)?;
                let fraction = if self.state == State::Fraction {
                    self.layout.fraction_digits.checked_add(1).ok_or(Overflow)?
                } else {
                    self.layout.fraction_digits
                };
                self.layout.mantissa_digits = mantissa;
                self.layout.fraction_digits = fraction;
                Step::Consume(Part::Mantissa(digit.expect("fixed digit check")))
            }
            State::Integer => {
                self.state = State::Point;
                Step::Transition
            }
            State::Point => {
                if a == Some('.') && b.is_some_and(|value| value.is_ascii_digit()) {
                    self.state = State::Fraction;
                    Step::Consume(Part::Point)
                } else {
                    self.state = State::Exponent;
                    Step::Transition
                }
            }
            State::Fraction => {
                self.state = State::Exponent;
                Step::Transition
            }
            State::Exponent => {
                if matches!(a, Some('e' | 'E'))
                    && (b.is_some_and(|value| value.is_ascii_digit())
                        || matches!(b, Some('+' | '-'))
                            && c.is_some_and(|value| value.is_ascii_digit()))
                {
                    self.state = State::ExponentSign;
                    Step::Consume(Part::Exponent)
                } else {
                    self.state = State::Done;
                    Step::Done(self.layout)
                }
            }
            State::ExponentSign => {
                self.state = State::ExponentDigits;
                if matches!(a, Some('+' | '-')) {
                    self.layout.exponent_negative = a == Some('-');
                    Step::Consume(Part::ExponentSign)
                } else {
                    Step::Transition
                }
            }
            State::ExponentDigits if digit.is_some() => {
                self.layout.exponent_digits =
                    self.layout.exponent_digits.checked_add(1).ok_or(Overflow)?;
                Step::Consume(Part::ExponentDigit(digit.expect("fixed digit check")))
            }
            State::ExponentDigits => {
                self.state = State::Done;
                Step::Done(self.layout)
            }
            State::Done => Step::Done(self.layout),
        })
    }
}

/// CSS Syntax's fixed three-code-point would-start-a-number decision.
pub(crate) fn number_start([a, b, c]: [Option<char>; 3]) -> bool {
    match a {
        Some('+' | '-') => {
            b.is_some_and(|value| value.is_ascii_digit())
                || b == Some('.') && c.is_some_and(|value| value.is_ascii_digit())
        }
        Some('.') => b.is_some_and(|value| value.is_ascii_digit()),
        Some(value) => value.is_ascii_digit(),
        None => false,
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
                Layout {
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
                Layout {
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
                Layout {
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
                Layout {
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
                    Step::Consume(_) => index += 1,
                    Step::Transition => {}
                    Step::Done(layout) => {
                        assert_eq!(index, length, "{text}");
                        assert_eq!(layout, expected);
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn starts_are_ascii_and_require_actual_lookahead() {
        for (window, expected) in [
            ([Some('-'), Some('.'), Some('1')], true),
            ([Some('+'), Some('2'), None], true),
            ([Some('.'), Some('1'), None], true),
            ([Some('١'), None, None], false),
            ([Some('-'), Some('.'), None], false),
        ] {
            assert_eq!(number_start(window), expected);
        }
    }
}
