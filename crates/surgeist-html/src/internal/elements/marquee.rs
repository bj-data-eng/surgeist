//! Marquee's normative attributes and native state.

use crate::{
    internal::microsyntax::numeric::try_parse_nonnegative_integer_capped_steps,
    values::DocumentString,
};

const MAX_REFLECTED_INTEGER: u32 = 2_147_483_647;

/// Default policies for Marquee attributes reflected as nonnegative unsigned longs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMarqueeUnsignedReflection {
    /// The horizontal space, defaulting to zero.
    Hspace,
    /// The vertical space, defaulting to zero.
    Vspace,
    /// The scroll amount, defaulting to six.
    ScrollAmount,
    /// The scroll delay, defaulting to 85.
    ScrollDelay,
}

impl HtmlMarqueeUnsignedReflection {
    const fn default_value(self) -> u32 {
        match self {
            Self::Hspace | Self::Vspace => 0,
            Self::ScrollAmount => 6,
            Self::ScrollDelay => 85,
        }
    }

    /// Read the reflected value, admitting each reached integer parser step.
    ///
    /// Missing, invalid and above-range input use the attribute's default.
    pub fn read_with<E>(
        self,
        raw: Option<&DocumentString>,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<u32, E> {
        Ok(read_nonnegative_with(raw, step)?
            .and_then(|value| u32::try_from(value).ok())
            .map_or(self.default_value(), |value| self.setter_value(value)))
    }

    /// Select the content value after Web IDL unsigned-long conversion.
    pub const fn setter_value(self, value: u32) -> u32 {
        if value <= MAX_REFLECTED_INTEGER {
            value
        } else {
            self.default_value()
        }
    }
}

#[derive(Clone, Copy)]
enum LoopCount {
    Indefinite,
    Finite(i32),
    AboveLong,
}

impl LoopCount {
    fn read_with<E>(
        raw: Option<&DocumentString>,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, E> {
        Ok(match read_nonnegative_with(raw, step)? {
            None | Some(0) => Self::Indefinite,
            Some(value) => i32::try_from(value).map_or(Self::AboveLong, Self::Finite),
        })
    }

    const fn matches_long(self, value: i32) -> bool {
        match self {
            Self::Indefinite => value == -1,
            Self::Finite(current) => value == current,
            Self::AboveLong => false,
        }
    }
}

/// Read Marquee's signed-long loop count, admitting reached parser steps.
///
/// Missing, indefinite and above-range positive counts return -1.
pub fn marquee_loop_value_with<E>(
    raw: Option<&DocumentString>,
    step: impl FnMut() -> Result<(), E>,
) -> Result<i32, E> {
    Ok(match LoopCount::read_with(raw, step)? {
        LoopCount::Finite(value) => value,
        LoopCount::Indefinite | LoopCount::AboveLong => -1,
    })
}

/// Decide whether to serialize a converted Web IDL long to Marquee's loop attribute.
///
/// The current raw count retains positive overflow for the equality decision.
/// Disallowed values require no parser steps.
pub fn marquee_loop_setter_value_with<E>(
    value: i32,
    raw: Option<&DocumentString>,
    step: impl FnMut() -> Result<(), E>,
) -> Result<Option<i32>, E> {
    if value < -1 || value == 0 {
        return Ok(None);
    }
    Ok((!LoopCount::read_with(raw, step)?.matches_long(value)).then_some(value))
}

fn read_nonnegative_with<E>(
    raw: Option<&DocumentString>,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<Option<usize>, E> {
    let Some(raw) = raw else { return Ok(None) };
    // One above range preserves the distinction without accumulating more digits.
    Ok(try_parse_nonnegative_integer_capped_steps(
        raw.code_units(),
        MAX_REFLECTED_INTEGER as usize + 1,
        &mut step,
    )?
    .ok())
}

/// Marquee's actual native state, independent of layout or animation.
///
/// HTML 877d6146487f [#the-marquee-element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-marquee-element)
/// creates marquees turned on with current loop index zero. Start and stop
/// change only the turned-on slot, preserving the current loop index.
///
/// The supported native domain has no rendering-layer increment operation, so
/// its index remains actually zero. This is not an unavailable observation or
/// a general counter with a machine-integer magnitude bound. Installing a
/// rendering increment capability must extend this closed domain first.
/// Copying this value retains actual state; fresh native creation is separate.
/// No renderer, timing, execution or event capability is established here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlMarqueeState {
    turned_on: bool,
    current_loop_index: NativeLoopIndex,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeLoopIndex {
    Zero,
}

impl HtmlMarqueeState {
    /// A newly created marquee: turned on, current loop index zero.
    pub const fn fresh() -> Self {
        Self {
            turned_on: true,
            current_loop_index: NativeLoopIndex::Zero,
        }
    }

    /// Whether the marquee is turned on. Only [`start`](Self::start) and [`stop`](Self::stop)
    /// change it.
    pub const fn turned_on(self) -> bool {
        self.turned_on
    }

    /// Actual current loop index in the supported native zero-index domain.
    pub const fn current_loop_index(self) -> u128 {
        match self.current_loop_index {
            NativeLoopIndex::Zero => 0,
        }
    }

    /// Turns the marquee on, leaving the current loop index unchanged. Idempotent.
    pub fn start(&mut self) {
        self.turned_on = true;
    }

    /// Turns the marquee off, leaving the current loop index unchanged. Idempotent.
    pub fn stop(&mut self) {
        self.turned_on = false;
    }
}

#[cfg(test)]
mod reflection_tests {
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct WorkRefused;

    #[test]
    fn unsigned_marquee_reflections_keep_zero_and_default_above_long_max() {
        let always = || Ok::<_, WorkRefused>(());
        for (rule, default) in [
            (HtmlMarqueeUnsignedReflection::Hspace, 0),
            (HtmlMarqueeUnsignedReflection::Vspace, 0),
            (HtmlMarqueeUnsignedReflection::ScrollAmount, 6),
            (HtmlMarqueeUnsignedReflection::ScrollDelay, 85),
        ] {
            assert_eq!(rule.read_with(None, always), Ok(default));
            for (raw, expected) in [
                (" 	+17tail", 17),
                ("-0", 0),
                ("0", 0),
                ("-1", default),
                ("invalid", default),
                ("2147483647", 2_147_483_647),
                ("2147483648", default),
            ] {
                let raw = DocumentString::from(raw);
                assert_eq!(rule.read_with(Some(&raw), always), Ok(expected));
            }
            assert_eq!(rule.setter_value(0), 0);
            assert_eq!(rule.setter_value(2_147_483_647), 2_147_483_647);
            assert_eq!(rule.setter_value(2_147_483_648), default);
            assert_eq!(rule.setter_value(u32::MAX), default);
        }
    }

    #[test]
    fn loop_equality_distinguishes_positive_overflow_from_indefinite() {
        let always = || Ok::<_, WorkRefused>(());
        assert_eq!(marquee_loop_value_with(None, always), Ok(-1));
        assert_eq!(marquee_loop_setter_value_with(-1, None, always), Ok(None));
        for (raw, read, write, next) in [
            ("bad", -1, -1, None),
            ("-2", -1, -1, None),
            ("-0", -1, -1, None),
            (" +003tail", 3, 3, None),
            (" +003tail", 3, 4, Some(4)),
            ("2147483647", 2_147_483_647, 2_147_483_647, None),
            ("2147483648", -1, -1, Some(-1)),
            ("9999999999999999999999999999999999999999", -1, -1, Some(-1)),
        ] {
            let raw = DocumentString::from(raw);
            assert_eq!(marquee_loop_value_with(Some(&raw), always), Ok(read));
            assert_eq!(
                marquee_loop_setter_value_with(write, Some(&raw), always),
                Ok(next)
            );
        }
        let raw = DocumentString::from("2147483648");
        for value in [i32::MIN, -2, 0] {
            assert_eq!(
                marquee_loop_setter_value_with(value, Some(&raw), || Err(WorkRefused)),
                Ok(None)
            );
        }
    }

    #[test]
    fn reached_marquee_parser_steps_preserve_the_first_refusal() {
        let always = || Ok::<_, WorkRefused>(());
        let raw = DocumentString::from("12tail");
        // Two digits and the terminator are inspected, then two digits projected.
        for limit in [4_usize, 5] {
            let mut left = limit;
            let mut step = || {
                left = left.checked_sub(1).ok_or(WorkRefused)?;
                Ok(())
            };
            let result = marquee_loop_setter_value_with(12, Some(&raw), &mut step);
            assert_eq!(left, 0);
            assert_eq!(
                result,
                if limit == 5 {
                    Ok(None)
                } else {
                    Err(WorkRefused)
                }
            );
        }
        let raw = DocumentString::from_utf16(vec![b'8' as u16, 0xD800, b'9' as u16]);
        let mut reached = 0;
        assert_eq!(
            HtmlMarqueeUnsignedReflection::ScrollAmount.read_with(Some(&raw), || {
                reached += 1;
                if reached > 2 {
                    Err(WorkRefused)
                } else {
                    Ok(())
                }
            }),
            Err(WorkRefused)
        );
        assert_eq!(reached, 3);
        assert_eq!(
            HtmlMarqueeUnsignedReflection::ScrollAmount.read_with(Some(&raw), always),
            Ok(8)
        );
    }
}
