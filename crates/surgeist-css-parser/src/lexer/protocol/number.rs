//! The canonical CSS Syntax consume-a-number spelling continuation.
//! Decimal conversion and unit/type admission are separate owners.

use std::fmt;

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
///
/// The cursor is unchanged by the error. Ask which counter overflowed with the
/// `is_*` predicates. The fraction digits are counted among the mantissa digits
/// and never outnumber them, so the mantissa counter is the one that overflows
/// first. Reaching one takes more digits than a `usize` counts, so
/// only a host that feeds the cursor an astronomically long number sees it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumberOverflow {
    kind: NumberOverflowKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NumberOverflowKind {
    Mantissa,
    Exponent,
}

impl NumberOverflow {
    kind_predicates! { kind;
        /// Whether the count of mantissa digits, integer and fraction
        /// together, does not fit `usize`.
        is_mantissa_digits => NumberOverflowKind::Mantissa,
        /// Whether the count of exponent digits does not fit `usize`.
        is_exponent_digits => NumberOverflowKind::Exponent,
    }
}

impl fmt::Display for NumberOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.kind {
            NumberOverflowKind::Mantissa => "the count of mantissa digits does not fit usize",
            NumberOverflowKind::Exponent => "the count of exponent digits does not fit usize",
        })
    }
}

impl std::error::Error for NumberOverflow {}

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

    /// Applies `count` more digit steps at once, for a driver that has counted
    /// the digits that follow one it just consumed. It is the same counting the
    /// step does one digit at a time, so the cursor ends in the same state.
    ///
    /// # Errors
    ///
    /// [`NumberOverflow`] when a digit counter would exceed `usize`; the cursor
    /// is then unchanged.
    pub(crate) fn add_digits(&mut self, count: usize) -> Result<(), NumberOverflow> {
        let layout = &mut self.layout;
        match self.state {
            State::Integer => {
                layout.mantissa_digits =
                    layout
                        .mantissa_digits
                        .checked_add(count)
                        .ok_or(NumberOverflow {
                            kind: NumberOverflowKind::Mantissa,
                        })?;
            }
            State::Fraction => {
                let mantissa = layout
                    .mantissa_digits
                    .checked_add(count)
                    .ok_or(NumberOverflow {
                        kind: NumberOverflowKind::Mantissa,
                    })?;
                // The fraction digits are among the mantissa digits, so they
                // cannot overflow when the mantissa count did not.
                let fraction = layout.fraction_digits + count;
                layout.mantissa_digits = mantissa;
                layout.fraction_digits = fraction;
            }
            State::ExponentDigits => {
                layout.exponent_digits =
                    layout
                        .exponent_digits
                        .checked_add(count)
                        .ok_or(NumberOverflow {
                            kind: NumberOverflowKind::Exponent,
                        })?;
            }
            _ => {}
        }
        Ok(())
    }

    /// When the digits just consumed end the number because `next` starts
    /// neither a fraction nor an exponent, applies at once the transitions that
    /// would consume nothing and return the number's layout. Otherwise leaves
    /// the cursor unchanged and returns `None`.
    ///
    /// It is the steps of the integer and fraction states for a `next` that is
    /// not a digit and not a full stop (integer only) or an `e` (either): the
    /// same ones [`step`](Self::step) takes, one at a time, without consuming.
    pub(crate) fn finish_without_more(&mut self, next: Option<char>) -> Option<NumberLayout> {
        let digit_or_e = matches!(next, Some('0'..='9' | 'e' | 'E'));
        let plain = match self.state {
            State::Integer => !digit_or_e && next != Some('.'),
            State::Fraction => !digit_or_e,
            _ => false,
        };
        if plain {
            self.state = State::Done;
            Some(self.layout)
        } else {
            None
        }
    }

    /// One fixed grammar transition over the next three code points (`None`
    /// is the EOF code point). Only a [`NumberStep::Consume`] advances input.
    /// Counters are checked before mutation; overflow retains the old state.
    ///
    /// # Errors
    ///
    /// [`NumberOverflow`] when a digit counter would exceed `usize`.
    #[inline]
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
                let mantissa =
                    self.layout
                        .mantissa_digits
                        .checked_add(1)
                        .ok_or(NumberOverflow {
                            kind: NumberOverflowKind::Mantissa,
                        })?;
                // The fraction digits are among the mantissa digits, so they
                // cannot overflow when the mantissa count did not.
                let fraction = if self.state == State::Fraction {
                    self.layout.fraction_digits + 1
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
                self.layout.exponent_digits =
                    self.layout
                        .exponent_digits
                        .checked_add(1)
                        .ok_or(NumberOverflow {
                            kind: NumberOverflowKind::Exponent,
                        })?;
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

    /// Steps `cursor` over `text` until a step consumes a digit of the wanted
    /// kind, and returns with that digit consumed.
    fn until_digit(cursor: &mut NumberCursor, text: &str, exponent: bool) {
        let bytes = text.as_bytes();
        let mut index = 0;
        loop {
            let at = |offset: usize| bytes.get(index + offset).copied().map(char::from);
            match cursor.step([at(0), at(1), at(2)]).unwrap() {
                NumberStep::Consume(NumberPart::ExponentDigit(_)) if exponent => return,
                NumberStep::Consume(NumberPart::Mantissa(_)) if !exponent => return,
                NumberStep::Consume(_) => index += 1,
                NumberStep::Transition => {}
                NumberStep::Done(_) => panic!("{text:?} ended before the digit"),
            }
        }
    }

    #[test]
    fn a_digit_count_that_overflows_names_its_counter_and_leaves_the_cursor_unchanged() {
        // Section 4.3.13 counts digits; a count past usize::MAX is not an input
        // any text reaches, so the cursor is driven to the state directly.
        let mut cursor = NumberCursor::new();
        until_digit(&mut cursor, "12", false);
        let before = cursor.layout;
        let error = cursor.add_digits(usize::MAX).unwrap_err();
        assert!(error.is_mantissa_digits());
        assert!(!error.is_exponent_digits());
        assert_eq!(cursor.layout, before);
        // The count still works afterwards: one digit consumed plus two.
        cursor.add_digits(2).unwrap();
        assert_eq!(cursor.layout.mantissa_digits, 3);

        // In the fraction the mantissa counter overflows first.
        let mut cursor = NumberCursor::new();
        until_digit(&mut cursor, ".5", false);
        let before = cursor.layout;
        assert!(
            cursor
                .add_digits(usize::MAX)
                .unwrap_err()
                .is_mantissa_digits()
        );
        assert_eq!(cursor.layout, before);
        cursor.add_digits(3).unwrap();
        assert_eq!(
            (cursor.layout.mantissa_digits, cursor.layout.fraction_digits),
            (4, 4)
        );

        let mut cursor = NumberCursor::new();
        until_digit(&mut cursor, "1e5", true);
        let before = cursor.layout;
        let error = cursor.add_digits(usize::MAX).unwrap_err();
        assert!(error.is_exponent_digits());
        assert!(!error.is_mantissa_digits());
        assert_eq!(cursor.layout, before);
        cursor.add_digits(4).unwrap();
        assert_eq!(cursor.layout.exponent_digits, 5);
    }

    #[test]
    fn a_number_overflow_describes_itself_and_is_an_error() {
        let mut cursor = NumberCursor::new();
        until_digit(&mut cursor, "1", false);
        let mantissa = cursor.add_digits(usize::MAX).unwrap_err();
        let mut cursor = NumberCursor::new();
        until_digit(&mut cursor, "1e1", true);
        let exponent = cursor.add_digits(usize::MAX).unwrap_err();
        assert_eq!(
            mantissa.to_string(),
            "the count of mantissa digits does not fit usize"
        );
        assert_eq!(
            exponent.to_string(),
            "the count of exponent digits does not fit usize"
        );
        assert_ne!(mantissa, exponent);
        let as_error: &dyn std::error::Error = &mantissa;
        assert_eq!(as_error.to_string(), mantissa.to_string());
    }

    #[test]
    fn add_digits_outside_a_digit_state_changes_nothing() {
        let mut cursor = NumberCursor::new();
        let before = cursor.layout;
        cursor.add_digits(5).unwrap();
        assert_eq!(cursor.layout, before);
    }

    #[test]
    fn a_number_ends_at_once_only_when_nothing_can_continue_it() {
        // After integer digits, a full stop, an e or a digit may continue it.
        for (next, ends) in [
            (None, true),
            (Some('x'), true),
            (Some('%'), true),
            (Some('-'), true),
            (Some('.'), false),
            (Some('e'), false),
            (Some('E'), false),
        ] {
            let mut cursor = NumberCursor::new();
            until_digit(&mut cursor, "12", false);
            let layout = cursor.layout;
            assert_eq!(
                cursor.finish_without_more(next),
                ends.then_some(layout),
                "{next:?}"
            );
        }
        // After fraction digits only an e may.
        let mut cursor = NumberCursor::new();
        until_digit(&mut cursor, ".5", false);
        assert!(cursor.finish_without_more(Some('.')).is_some());
        let mut cursor = NumberCursor::new();
        until_digit(&mut cursor, ".5", false);
        assert!(cursor.finish_without_more(Some('e')).is_none());
    }

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
