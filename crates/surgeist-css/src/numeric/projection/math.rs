//! CSS Values 4 function semantics, including exceptions to host math behavior.
use super::{Function, Scalar, Unit};
use crate::CssRoundingStrategy as Strategy;

pub(super) fn extreme(a: f64, b: f64, minimum: bool) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if minimum {
            if a.is_sign_negative() || b.is_sign_negative() {
                -0.0
            } else {
                0.0
            }
        } else if a.is_sign_positive() || b.is_sign_positive() {
            0.0
        } else {
            -0.0
        };
    }
    if (a < b) == minimum { a } else { b }
}

/// Fixed operand payload for one already type-checked CSS math kernel.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Operand {
    pub(crate) value: f64,
    pub(crate) degrees: bool,
}

/// A one-operand-per-action fold for the three variadic numeric functions.
/// No borrowed argument slice is scanned inside a fold action.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VariadicMath {
    function: Function,
    value: Option<f64>,
    nan: bool,
}
impl VariadicMath {
    pub(crate) const fn new(function: Function) -> Option<Self> {
        match function {
            Function::Min | Function::Max | Function::Hypot => Some(Self {
                function,
                value: None,
                nan: false,
            }),
            _ => None,
        }
    }
    pub(crate) fn push(&mut self, value: f64) {
        self.nan |= value.is_nan();
        self.value = Some(match self.function {
            Function::Min | Function::Max => self.value.map_or(value, |previous| {
                extreme(previous, value, self.function == Function::Min)
            }),
            Function::Hypot => self.value.unwrap_or(0.0).hypot(value),
            _ => unreachable!("checked variadic function"),
        });
    }
    pub(crate) fn finish(self) -> Option<f64> {
        self.value
            .map(|value| if self.nan { f64::NAN } else { value })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelArguments;

pub(super) fn evaluate(
    function: Function,
    args: &[Option<Scalar>],
    strategy: Option<Strategy>,
) -> f64 {
    if let Some(mut fold) = VariadicMath::new(function) {
        for argument in args.iter().flatten() {
            fold.push(argument.value);
        }
        return fold.finish().expect("checked nonempty variadic function");
    }
    let mut fixed = [None; 3];
    assert!(args.len() <= fixed.len(), "checked fixed arity");
    for (index, argument) in args.iter().enumerate() {
        fixed[index] = argument.as_ref().map(|argument| Operand {
            value: argument.value,
            degrees: argument.unit == Unit::Canonical("deg"),
        });
    }
    evaluate_fixed(function, fixed, args.len(), strategy).expect("checked function arguments")
}

/// One fixed-arity numerical computation, shared by ordinary and bounded owners.
/// Invalid argument headers are refused before a kernel executes.
pub(crate) fn evaluate_fixed(
    function: Function,
    args: [Option<Operand>; 3],
    length: usize,
    strategy: Option<Strategy>,
) -> Result<f64, KernelArguments> {
    let valid = match function {
        Function::Min | Function::Max | Function::Hypot => false,
        Function::Clamp => length == 3 && args[1].is_some(),
        Function::Round | Function::Log => {
            (length == 1 || length == 2) && args[0].is_some() && (length == 1 || args[1].is_some())
        }
        Function::Mod | Function::Rem | Function::Atan2 | Function::Pow => {
            length == 2 && args[0].is_some() && args[1].is_some()
        }
        _ => length == 1 && args[0].is_some(),
    };
    if !valid
        || args
            .get(length..)
            .is_none_or(|tail| tail.iter().any(Option::is_some))
    {
        return Err(KernelArguments);
    }
    let at = |i: usize| args[i].expect("checked present argument").value;
    let a = args
        .iter()
        .flatten()
        .next()
        .expect("checked function arguments")
        .value;
    // Exactly three fixed registers; this is not an arbitrary argument scan.
    if args.iter().flatten().any(|value| value.value.is_nan()) {
        return Ok(f64::NAN);
    }
    Ok(match function {
        Function::Calc => a,
        Function::Min | Function::Max | Function::Hypot => unreachable!("variadic owner"),
        Function::Clamp => {
            let mut value = at(1);
            if let Some(upper) = &args[2] {
                value = extreme(value, upper.value, true);
            }
            if let Some(lower) = &args[0] {
                value = extreme(value, lower.value, false);
            }
            value
        }
        Function::Round => round(
            a,
            if length == 1 { 1.0 } else { at(1) },
            strategy.unwrap_or(Strategy::Nearest),
        ),
        Function::Mod | Function::Rem => remainder(a, at(1), function == Function::Mod),
        Function::Sin | Function::Cos | Function::Tan => {
            let angle = args[0].as_ref().expect("angle argument");
            trig(function, a, angle.degrees)
        }
        Function::Asin => {
            if !(-1.0..=1.0).contains(&a) {
                f64::NAN
            } else {
                a.asin().to_degrees()
            }
        }
        Function::Acos => {
            if !(-1.0..=1.0).contains(&a) {
                f64::NAN
            } else {
                a.acos().to_degrees()
            }
        }
        Function::Atan => {
            if a.is_infinite() {
                90.0f64.copysign(a)
            } else {
                a.atan().to_degrees()
            }
        }
        Function::Atan2 => atan2(a, at(1)),
        Function::Pow => pow(a, at(1)),
        Function::Sqrt => {
            if a < 0.0 {
                f64::NAN
            } else {
                a.sqrt()
            }
        }
        Function::Log => {
            let base = if length == 1 {
                std::f64::consts::E
            } else {
                at(1)
            };
            if base == 1.0 || base < 0.0 || a < 0.0 {
                f64::NAN
            } else if a == 0.0 {
                f64::NEG_INFINITY
            } else if a == 1.0 {
                0.0
            } else if a == f64::INFINITY {
                f64::INFINITY
            } else if length == 1 {
                a.ln()
            } else {
                a.ln() / base.ln()
            }
        }
        Function::Exp => a.exp(),
        Function::Abs => a.abs(),
        Function::Sign => {
            if a == 0.0 {
                a
            } else {
                a.signum()
            }
        }
    })
}

fn round(a: f64, b: f64, strategy: Strategy) -> f64 {
    if b == 0.0 || a.is_infinite() && b.is_infinite() {
        return f64::NAN;
    }
    if a.is_infinite() {
        return a;
    }
    if b.is_infinite() {
        return match strategy {
            Strategy::Nearest | Strategy::ToZero => 0.0f64.copysign(a),
            Strategy::Up => {
                if a > 0.0 {
                    f64::INFINITY
                } else {
                    0.0f64.copysign(a)
                }
            }
            Strategy::Down => {
                if a < 0.0 {
                    f64::NEG_INFINITY
                } else {
                    0.0f64.copysign(a)
                }
            }
        };
    }
    if a == 0.0 {
        return a;
    }
    let step = b.abs();
    let remainder = a % step;
    if remainder == 0.0 {
        return a;
    }
    // Use remainder rather than a/step: its quotient may overflow although the
    // neighboring multiples and the rounded value are finite.
    let lower = if a < 0.0 {
        a - remainder - step
    } else {
        a - remainder
    };
    let upper = if a < 0.0 {
        a - remainder
    } else {
        a - remainder + step
    };
    let lower = if lower == 0.0 { 0.0 } else { lower };
    let upper = if upper == 0.0 { -0.0 } else { upper };
    match strategy {
        Strategy::Up => upper,
        Strategy::Down => lower,
        Strategy::ToZero => {
            if a < 0.0 {
                upper
            } else {
                lower
            }
        }
        Strategy::Nearest => {
            let distance_lower = if a < 0.0 { step + remainder } else { remainder };
            let distance_upper = if a < 0.0 {
                -remainder
            } else {
                step - remainder
            };
            if distance_lower < distance_upper {
                lower
            } else {
                upper
            }
        }
    }
}

fn remainder(a: f64, b: f64, modulus: bool) -> f64 {
    if b == 0.0 || a.is_infinite() {
        return f64::NAN;
    }
    if b.is_infinite() {
        return if modulus && a.is_sign_negative() != b.is_sign_negative() {
            f64::NAN
        } else {
            a
        };
    }
    let r = a % b;
    if !modulus {
        return r;
    }
    if r == 0.0 {
        return 0.0f64.copysign(b);
    }
    if r.is_sign_negative() != b.is_sign_negative() {
        r + b
    } else {
        r
    }
}

fn trig(function: Function, a: f64, degrees: bool) -> f64 {
    if a.is_infinite() {
        return f64::NAN;
    }
    if a == 0.0 {
        return if function == Function::Cos { 1.0 } else { a };
    }
    if degrees && a % 90.0 == 0.0 {
        let quarter = (a % 360.0) / 90.0;
        return match function {
            Function::Sin => match quarter as i32 {
                1 | -3 => 1.0,
                3 | -1 => -1.0,
                _ => 0.0,
            },
            Function::Cos => match quarter as i32 {
                0 => 1.0,
                2 | -2 => -1.0,
                _ => 0.0,
            },
            Function::Tan => match quarter as i32 {
                1 | -3 => f64::INFINITY,
                3 | -1 => f64::NEG_INFINITY,
                _ => 0.0,
            },
            _ => unreachable!("trigonometric function"),
        };
    }
    let radians = if degrees { a.to_radians() } else { a };
    match function {
        Function::Sin => radians.sin(),
        Function::Cos => radians.cos(),
        Function::Tan => radians.tan(),
        _ => unreachable!("trigonometric function"),
    }
}

fn atan2(y: f64, x: f64) -> f64 {
    // The selected draft's exceptional table explicitly gives -180 degrees,
    // conflicting with its general (-180, 180] prose. Listed exceptional cases
    // follow the specific table, also corroborated by frozen WebKit atan2.
    if y.is_infinite() {
        if x.is_infinite() {
            return (if x.is_sign_negative() {
                135.0f64
            } else {
                45.0f64
            })
            .copysign(y);
        }
        return 90.0f64.copysign(y);
    }
    if y == 0.0 {
        return if x.is_sign_negative() {
            180.0f64.copysign(y)
        } else {
            0.0f64.copysign(y)
        };
    }
    if x == 0.0 {
        return 90.0f64.copysign(y);
    }
    if x.is_infinite() {
        return if x.is_sign_negative() {
            180.0f64.copysign(y)
        } else {
            0.0f64.copysign(y)
        };
    }
    y.atan2(x).to_degrees()
}

fn pow(a: f64, b: f64) -> f64 {
    if b.is_infinite() {
        if a.abs() == 1.0 {
            return f64::NAN;
        }
        let grows = a.abs() > 1.0;
        return if grows == b.is_sign_positive() {
            f64::INFINITY
        } else {
            0.0
        };
    }
    if b == 0.0 {
        return 1.0;
    }
    let odd = b.is_finite() && b.fract() == 0.0 && b.abs() % 2.0 == 1.0;
    if a == 0.0 || a.is_infinite() {
        let infinite = if a == 0.0 { b < 0.0 } else { b > 0.0 };
        let magnitude = if infinite { f64::INFINITY } else { 0.0 };
        return if a.is_sign_negative() && odd {
            -magnitude
        } else {
            magnitude
        };
    }
    if a < 0.0 && b.fract() != 0.0 {
        return f64::NAN;
    }
    a.powf(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn variadic_steps_preserve_css_nan_and_signed_zero() {
        let mut min = VariadicMath::new(Function::Min).unwrap();
        min.push(0.0);
        min.push(-0.0);
        assert_eq!(min.finish().unwrap().to_bits(), (-0.0f64).to_bits());
        let mut max = VariadicMath::new(Function::Max).unwrap();
        max.push(-0.0);
        max.push(0.0);
        assert_eq!(max.finish().unwrap().to_bits(), 0.0f64.to_bits());
        let mut hypot = VariadicMath::new(Function::Hypot).unwrap();
        hypot.push(3.0);
        hypot.push(4.0);
        assert_eq!(hypot.finish(), Some(5.0));
        hypot.push(f64::INFINITY);
        hypot.push(f64::NAN);
        assert!(hypot.finish().unwrap().is_nan());
        assert!(VariadicMath::new(Function::Calc).is_none());
        assert_eq!(VariadicMath::new(Function::Min).unwrap().finish(), None);
    }

    #[test]
    fn fixed_kernel_refuses_headers_and_reuses_angle_and_clamp_semantics() {
        let value = |value, degrees| Some(Operand { value, degrees });
        assert_eq!(
            evaluate_fixed(Function::Pow, [value(2.0, false), None, None], 1, None),
            Err(KernelArguments)
        );
        assert_eq!(
            evaluate_fixed(
                Function::Abs,
                [value(-3.0, false), value(4.0, false), None],
                1,
                None
            ),
            Err(KernelArguments)
        );
        assert_eq!(
            evaluate_fixed(
                Function::Clamp,
                [None, value(2.0, false), value(1.0, false)],
                3,
                None
            ),
            Ok(1.0)
        );
        assert_eq!(
            evaluate_fixed(Function::Sin, [value(90.0, true), None, None], 1, None),
            Ok(1.0)
        );
        assert_eq!(
            evaluate_fixed(Function::Sign, [value(-0.0, false), None, None], 1, None)
                .unwrap()
                .to_bits(),
            (-0.0f64).to_bits()
        );
    }
    #[test]
    fn stepped_functions_follow_css_ties_signs_and_infinite_steps() {
        for (strategy, positive, negative) in [
            (Strategy::Nearest, 3.0, -2.0),
            (Strategy::Up, 3.0, -2.0),
            (Strategy::Down, 2.0, -3.0),
            (Strategy::ToZero, 2.0, -2.0),
        ] {
            assert_eq!(round(2.5, 1.0, strategy), positive);
            assert_eq!(round(-2.5, 1.0, strategy), negative);
        }
        assert_eq!(remainder(-18.0, 5.0, true), 2.0);
        assert_eq!(remainder(-18.0, 5.0, false), -3.0);
        assert_eq!(remainder(18.0, -5.0, true), -2.0);
        assert_eq!(remainder(18.0, -5.0, false), 3.0);
        assert!(round(1.0, 0.0, Strategy::Nearest).is_nan());
        assert!(remainder(-0.0, f64::INFINITY, true).is_nan());
        assert!(round(-1.0, f64::INFINITY, Strategy::Nearest).is_sign_negative());
        assert_eq!(round(1.0, f64::INFINITY, Strategy::Up), f64::INFINITY);
        assert_eq!(
            round(-1.0, f64::INFINITY, Strategy::Down),
            f64::NEG_INFINITY
        );
    }
    #[test]
    fn atan2_implements_all_selected_exceptional_quadrants() {
        let values = [f64::NEG_INFINITY, -1.0, -0.0, 0.0, 1.0, f64::INFINITY];
        let rows = [
            [-135.0, -90.0, -90.0, -90.0, -90.0, -45.0],
            [-180.0, -135.0, -90.0, -90.0, -45.0, -0.0],
            [-180.0, -180.0, -180.0, -0.0, -0.0, -0.0],
            [180.0, 180.0, 180.0, 0.0, 0.0, 0.0],
            [180.0, 135.0, 90.0, 90.0, 45.0, 0.0],
            [135.0, 90.0, 90.0, 90.0, 90.0, 45.0],
        ];
        for (y, row) in values.into_iter().zip(rows) {
            for (x, expected) in values.into_iter().zip(row) {
                let actual = atan2(y, x);
                if y.is_finite() && y != 0.0 && x.is_finite() && x != 0.0 {
                    // Normal finite cells use host transcendental arithmetic.
                    assert!((actual - expected).abs() <= 4.0 * f64::EPSILON * expected.abs());
                } else {
                    assert_eq!(actual, expected, "atan2({y}, {x})");
                }
                if expected == 0.0 {
                    assert_eq!(actual.is_sign_negative(), expected.is_sign_negative());
                }
            }
        }
    }
    #[test]
    fn pow_retains_selected_zero_infinity_and_parity_semantics() {
        assert!(pow(-1.0, f64::INFINITY).is_nan());
        assert!(pow(-2.0, 0.5).is_nan());
        assert_eq!(pow(-0.0, -3.0), f64::NEG_INFINITY);
        assert_eq!(pow(-0.0, -2.0), f64::INFINITY);
        assert!(pow(f64::NEG_INFINITY, -3.0).is_sign_negative());
        assert_eq!(pow(f64::NEG_INFINITY, 3.0), f64::NEG_INFINITY);
        assert_eq!(pow(f64::NEG_INFINITY, 2.0), f64::INFINITY);
        assert_eq!(pow(0.5, f64::INFINITY), 0.0);
        assert_eq!(pow(0.5, f64::NEG_INFINITY), f64::INFINITY);
    }
}
