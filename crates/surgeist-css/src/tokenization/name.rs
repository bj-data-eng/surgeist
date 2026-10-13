//! Finite CSS name/escape decoding. Inputs have already been preprocessed.

use super::{name_point, valid_escape};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Name,
    Escape,
    Hex { value: u32, digits: u8 },
    EmitHex { value: u32 },
    Done,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Step {
    /// A fixed syntax transition or consumption that emits no decoded unit.
    Consume {
        count: u8,
    },
    /// The caller stores one decoded scalar before acknowledging this step.
    Emit {
        value: char,
        count: u8,
    },
    Done,
}

/// A cached plan cannot mutate its cursor until the actual append succeeds.
pub(crate) struct NameAction<'a> {
    cursor: &'a mut NameCursor,
    step: Step,
    next: State,
}
impl NameAction<'_> {
    pub(crate) const fn step(&self) -> Step {
        self.step
    }
    pub(crate) fn complete(self) {
        self.cursor.state = self.next;
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NameCursor {
    state: State,
}
impl NameCursor {
    pub(crate) const fn new() -> Self {
        Self { state: State::Name }
    }

    /// A fixed two-code-point decision. It neither consumes input nor stores output.
    pub(crate) fn action(&mut self, [a, b]: [Option<char>; 2]) -> NameAction<'_> {
        let (step, next) = match self.state {
            State::Name => match a {
                Some(value) if name_point(value) => (Step::Emit { value, count: 1 }, State::Name),
                Some('\\') if valid_escape(a, b) => (Step::Consume { count: 1 }, State::Escape),
                _ => (Step::Done, State::Done),
            },
            State::Escape => match a {
                Some(value) if value.is_ascii_hexdigit() => (
                    Step::Consume { count: 1 },
                    State::Hex {
                        value: value.to_digit(16).expect("ASCII hex digit"),
                        digits: 1,
                    },
                ),
                Some(value) => (Step::Emit { value, count: 1 }, State::Name),
                None => (
                    Step::Emit {
                        value: '\u{fffd}',
                        count: 0,
                    },
                    State::Done,
                ),
            },
            State::Hex { value, digits } => {
                if digits < 6 && a.is_some_and(|value| value.is_ascii_hexdigit()) {
                    let value = value * 16
                        + a.and_then(|value| value.to_digit(16))
                            .expect("ASCII hex digit");
                    (
                        Step::Consume { count: 1 },
                        State::Hex {
                            value,
                            digits: digits + 1,
                        },
                    )
                } else {
                    let count = u8::from(a.is_some_and(whitespace));
                    (Step::Consume { count }, State::EmitHex { value })
                }
            }
            State::EmitHex { value } => (
                Step::Emit {
                    value: if value == 0 {
                        '\u{fffd}'
                    } else {
                        char::from_u32(value).unwrap_or('\u{fffd}')
                    },
                    count: 0,
                },
                State::Name,
            ),
            State::Done => (Step::Done, State::Done),
        };
        NameAction {
            cursor: self,
            step,
            next,
        }
    }
}

pub(crate) const fn whitespace(value: char) -> bool {
    matches!(value, '\t' | '\n' | ' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(text: &str) -> (String, usize) {
        let points: Vec<_> = text.chars().collect();
        let mut cursor = NameCursor::new();
        let mut output = String::new();
        let mut index = 0;
        loop {
            let action =
                cursor.action([points.get(index).copied(), points.get(index + 1).copied()]);
            match action.step() {
                Step::Consume { count } => index += usize::from(count),
                Step::Emit { value, count } => {
                    output.push(value);
                    index += usize::from(count);
                }
                Step::Done => return (output, index),
            }
            action.complete();
        }
    }

    #[test]
    fn hexadecimal_escapes_retain_optional_terminator_and_six_digit_limit() {
        assert_eq!(decode("\\72 ed)"), ("red".into(), 6));
        assert_eq!(decode("\\000061b)"), ("ab".into(), 8));
        assert_eq!(decode("\\31\tfoo"), ("1foo".into(), 7));
        assert_eq!(decode("\\abcdef0"), ("�0".into(), 8));
    }

    #[test]
    fn invalid_scalar_escapes_and_eof_follow_css_syntax() {
        for text in ["\\0", "\\d800", "\\110000", "\\"] {
            assert_eq!(decode(text), ("�".into(), text.len()));
        }
        assert_eq!(decode("r\\!d"), ("r!d".into(), 4));
        assert_eq!(decode("r\\\nd"), ("r".into(), 1));
    }

    #[test]
    fn dropped_append_plan_preserves_exact_name_transition() {
        let mut cursor = NameCursor::new();
        {
            let action = cursor.action([Some('r'), Some('e')]);
            assert_eq!(
                action.step(),
                Step::Emit {
                    value: 'r',
                    count: 1
                }
            );
        }
        let action = cursor.action([Some('r'), Some('e')]);
        assert_eq!(
            action.step(),
            Step::Emit {
                value: 'r',
                count: 1
            }
        );
        action.complete();
        let action = cursor.action([Some('e'), Some('d')]);
        assert_eq!(
            action.step(),
            Step::Emit {
                value: 'e',
                count: 1
            }
        );
        action.complete();
    }
}
