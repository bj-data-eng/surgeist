//! Reverse evaluation of validated tone curves under ICC.1:2022 Annex F.1.

use std::cmp::Ordering;

use super::{
    Curve, Extent, ParametricCurve, ProfileError, ProfileResult, native::IccProfileError,
    sample_invariant, sample_raw, sample_value,
};
use crate::{Result, accounting::Account};

pub(super) struct InverseCurve(Inverse);

enum Inverse {
    Identity,
    Gamma(f64),
    Sampled {
        data: Extent,
        intervals: u32,
        range: MonotonicRange,
    },
    Parametric {
        curve: ParametricCurve,
        range: MonotonicRange,
    },
}

impl InverseCurve {
    pub(super) fn prepare(curve: &Curve, bytes: &[u8], account: &Account) -> ProfileResult<Self> {
        let inverse = match *curve {
            Curve::Identity => Inverse::Identity,
            Curve::Gamma(gamma) => Inverse::Gamma(account.coordinate(|| gamma.recip())?),
            Curve::Sampled { data, intervals } => Inverse::Sampled {
                data,
                intervals,
                range: sampled_range(data.payload(bytes)?, intervals, account)?,
            },
            Curve::Parametric(curve) => Inverse::Parametric {
                curve,
                range: parametric_range(&curve, account)?,
            },
        };
        Ok(Self(inverse))
    }

    pub(super) fn evaluate(&self, target: f64, bytes: &[u8], account: &Account) -> Result<f64> {
        let target = account.coordinate(|| -> Result<f64> {
            if !target.is_finite() {
                return Err(IccProfileError::NumericOutput.into());
            }
            Ok(target.clamp(0.0, 1.0))
        })??;
        match &self.0 {
            Inverse::Identity => Ok(target),
            Inverse::Gamma(exponent) => account.coordinate(|| target.powf(*exponent)),
            Inverse::Sampled {
                data,
                intervals,
                range,
            } => {
                let samples = bytes
                    .get(data.start..data.end)
                    .ok_or_else(sample_invariant)?;
                invert_sampled(samples, *intervals, *range, target, account)
            }
            Inverse::Parametric { curve, range } => {
                invert_parametric(curve, *range, target, account)
            }
        }
    }
}

#[derive(Clone, Copy)]
struct MonotonicRange {
    first: f64,
    last: f64,
    direction: Ordering,
}

impl MonotonicRange {
    fn new(first: f64, last: f64) -> ProfileResult<Self> {
        match first.partial_cmp(&last) {
            Some(direction @ (Ordering::Less | Ordering::Greater)) => Ok(Self {
                first,
                last,
                direction,
            }),
            _ => Err(ProfileError::Unsupported),
        }
    }

    fn bounds(self, target: f64) -> f64 {
        target.clamp(self.first.min(self.last), self.first.max(self.last))
    }

    fn precedes(self, a: f64, b: f64) -> bool {
        a.partial_cmp(&b) == Some(self.direction)
    }

    fn accepts_branch(self, slope: f64, first: f64, last: f64) -> bool {
        first == last || (self.precedes(0.0, slope) && self.precedes(first, last))
    }

    // For a terminal plateau Annex F.1 selects its first coordinate, which is
    // the position already found by the first search. Otherwise use its last.
    fn plateau_end(
        self,
        first: u64,
        end: u64,
        terminal: bool,
        mut compare: impl FnMut(u64) -> Result<Ordering>,
    ) -> Result<u64> {
        if terminal || first == end || compare(first + 1)? != Ordering::Equal {
            return Ok(first);
        }
        let after_plateau = first_matching(first + 2, end, |index| {
            Ok(compare(index)? == self.direction.reverse())
        })?;
        Ok(after_plateau - 1)
    }
}

fn sampled_range(
    samples: &[u8],
    intervals: u32,
    account: &Account,
) -> ProfileResult<MonotonicRange> {
    let first = sample_raw(samples, 0, account)?;
    let mut previous = first;
    let mut direction = Ordering::Equal;
    for index in 1..=u64::from(intervals) {
        let sample = sample_raw(samples, index, account)?;
        let change = previous.cmp(&sample);
        if change != Ordering::Equal {
            if direction != Ordering::Equal && direction != change {
                return Err(ProfileError::Unsupported);
            }
            direction = change;
        }
        previous = sample;
    }
    account.coordinate(|| {
        MonotonicRange::new(f64::from(first) / 65535.0, f64::from(previous) / 65535.0)
    })?
}

fn invert_sampled(
    samples: &[u8],
    intervals: u32,
    range: MonotonicRange,
    target: f64,
    account: &Account,
) -> Result<f64> {
    let output = |index| sample_value(samples, index, account);
    let end = u64::from(intervals);
    let target = account.coordinate(|| range.bounds(target))?;
    let upper = first_matching(0, end, |index| Ok(!range.precedes(output(index)?, target)))?;
    let upper_value = output(upper)?;
    if upper_value == target {
        let index = range.plateau_end(upper, end, target == range.last, |index| {
            Ok(output(index)?
                .partial_cmp(&target)
                .unwrap_or(Ordering::Equal))
        })?;
        return account.coordinate(|| index as f64 / f64::from(intervals));
    }
    // The clamped target lies strictly between two samples, even when the
    // serialized samples descend. Interpolate in full precision.
    let lower = upper.saturating_sub(1);
    let lower_value = output(lower)?;
    account.coordinate(|| {
        let fraction = (target - lower_value) / (upper_value - lower_value);
        (lower as f64 + fraction) / f64::from(intervals)
    })
}

fn parametric_range(curve: &ParametricCurve, account: &Account) -> ProfileResult<MonotonicRange> {
    let first = account.coordinate(|| curve.evaluate(0.0))?;
    let last = account.coordinate(|| curve.evaluate(1.0))?;
    let (range, upper_slope) = account.coordinate(|| {
        Ok::<_, ProfileError>((MonotonicRange::new(first, last)?, curve.scale * curve.gamma))
    })??;
    let valid = if curve.threshold <= 0.0 {
        account.coordinate(|| range.accepts_branch(upper_slope, first, last))?
    } else if curve.threshold > 1.0 {
        account.coordinate(|| range.accepts_branch(curve.lower_scale, first, last))?
    } else {
        // Only the upper branch includes the threshold. Account for the jump
        // between the last admissible f64 in the lower branch and the threshold.
        let lower_end = account.coordinate(|| curve.evaluate(curve.threshold.next_down()))?;
        let upper_start = account.coordinate(|| curve.evaluate(curve.threshold))?;
        account.coordinate(|| {
            range.accepts_branch(curve.lower_scale, first, lower_end)
                && !range.precedes(upper_start, lower_end)
                && range.accepts_branch(upper_slope, upper_start, last)
        })?
    };
    if !valid {
        return Err(ProfileError::Unsupported);
    }
    Ok(range)
}

fn invert_parametric(
    curve: &ParametricCurve,
    range: MonotonicRange,
    target: f64,
    account: &Account,
) -> Result<f64> {
    // Nonnegative f64 bit patterns are ordered by value. Searching them gives
    // adjacent representable inputs in at most 62 evaluations, including
    // the neighbors of a discontinuity and values near zero.
    let output = |index| account.coordinate(|| curve.evaluate(f64::from_bits(index)));
    let end = 1.0_f64.to_bits();
    let target = account.coordinate(|| range.bounds(target))?;
    let level = account.coordinate(|| ParametricLevel::new(curve, target))?;
    let compare = |index| account.coordinate(|| level.compare(curve, f64::from_bits(index)));
    let upper = first_matching(0, end, |index| Ok(compare(index)? != range.direction))?;
    let upper_value = output(upper)?;
    if upper != 0 && upper_value != target {
        let lower = upper - 1;
        let lower_value = output(lower)?;
        // Resolve an equal-distance tie in favor of the smaller input.
        if account.coordinate(|| (target - lower_value).abs() <= (upper_value - target).abs())? {
            return Ok(f64::from_bits(lower));
        }
    }
    let level = if upper_value == target {
        level
    } else {
        account.coordinate(|| ParametricLevel::new(curve, upper_value))?
    };
    let compare = |index| account.coordinate(|| level.compare(curve, f64::from_bits(index)));
    let terminal = upper_value == range.last && compare(end)? == Ordering::Equal;
    let index = if compare(upper)? == Ordering::Equal {
        range.plateau_end(upper, end, terminal, compare)?
    } else {
        upper
    };
    Ok(f64::from_bits(index))
}

struct ParametricLevel {
    output: f64,
    // The power base where the unclipped upper branch would attain `output`.
    // None means the positive power term keeps that entire branch above it.
    power_base: Option<f64>,
}

impl ParametricLevel {
    fn new(curve: &ParametricCurve, output: f64) -> Self {
        let power = output - curve.upper_shift;
        let power_base = if power > 0.0 && curve.gamma != 0.0 {
            Some(power.powf(curve.gamma.recip()))
        } else if power == 0.0 && curve.gamma > 0.0 {
            Some(0.0)
        } else {
            None
        };
        Self { output, power_base }
    }

    fn compare(&self, curve: &ParametricCurve, input: f64) -> Ordering {
        match curve.evaluate(input).partial_cmp(&self.output) {
            Some(Ordering::Equal) => (),
            Some(order) => return order,
            None => return Ordering::Equal,
        }

        // Rounding can make a strictly monotone branch look flat. For
        // equality, compare in the affine domain before the power and shift.
        // For example, x^32767 underflows for much of the unit interval, but
        // an input greater than zero still does not belong to a zero plateau.
        let order = if input < curve.threshold {
            if curve.lower_scale == 0.0 {
                return Ordering::Equal;
            }
            affine_order(curve.lower_scale, input, curve.lower_shift - self.output)
        } else {
            if curve.scale == 0.0 || curve.gamma == 0.0 {
                return Ordering::Equal;
            }
            match self.power_base {
                None => Ordering::Greater,
                Some(0.0) if curve.gamma > 0.0 => {
                    affine_order(curve.scale, input, curve.offset).max(Ordering::Equal)
                }
                Some(base) => {
                    let order = affine_order(curve.scale, input, curve.offset - base);
                    if curve.gamma < 0.0 {
                        order.reverse()
                    } else {
                        order
                    }
                }
            }
        };
        if (self.output == 0.0 && order == Ordering::Less)
            || (self.output == 1.0 && order == Ordering::Greater)
        {
            Ordering::Equal
        } else {
            order
        }
    }
}

fn affine_order(scale: f64, input: f64, offset: f64) -> Ordering {
    let difference = scale.mul_add(input, offset);
    if difference == 0.0 && offset == 0.0 && input > 0.0 {
        // A nonzero slope times a subnormal input can itself underflow.
        return scale.total_cmp(&0.0);
    }
    difference.partial_cmp(&0.0).unwrap_or(Ordering::Equal)
}

fn first_matching(
    mut first: u64,
    mut last: u64,
    mut matches: impl FnMut(u64) -> Result<bool>,
) -> Result<u64> {
    while first < last {
        let middle = first + (last - first) / 2;
        if matches(middle)? {
            last = middle;
        } else {
            first = middle + 1;
        }
    }
    Ok(first)
}
