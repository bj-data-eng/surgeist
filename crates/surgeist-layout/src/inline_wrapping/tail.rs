//! Optional paragraph-tail quality over actual natural line measurements.

use super::balance::{BalanceGoal, NaturalResidual};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TailLine {
    advance: f64,
    fullness: f64,
}

impl TailLine {
    pub(crate) fn from_components(available: f64, indent: f64, advance: f64) -> Option<Self> {
        if ![available, indent, advance].into_iter().all(f64::is_finite) {
            return None;
        }
        let capacity = available - indent;
        if !capacity.is_finite() || capacity <= 0.0 || advance < 0.0 || advance > capacity {
            return None;
        }
        // Both operands are finite and 0 <= advance <= capacity. Division is
        // bounded by one; it never creates a geometry width or overflowing
        // product. Rounded ties merely miss an optional improvement.
        Some(Self {
            advance,
            fullness: advance / capacity,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TailPair {
    penultimate: TailLine,
    last: TailLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TailAdmission {
    NoLastFullnessGain,
    NoPairGain,
    NoNaturalTailGain,
    Improved,
}

impl TailPair {
    pub(crate) fn new(penultimate: TailLine, last: TailLine) -> Self {
        Self { penultimate, last }
    }

    pub(crate) fn eligible_auto(self) -> bool {
        self.last.fullness < self.penultimate.fullness
    }

    fn minimum(self) -> f64 {
        self.penultimate.fullness.min(self.last.fullness)
    }

    pub(crate) fn admission(self, reference: Self) -> TailAdmission {
        if self.last.fullness <= reference.last.fullness {
            TailAdmission::NoLastFullnessGain
        } else if self.minimum() <= reference.minimum() {
            TailAdmission::NoPairGain
        } else if self.last.advance <= reference.last.advance {
            TailAdmission::NoNaturalTailGain
        } else {
            TailAdmission::Improved
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TailGoal(f64);

impl TailGoal {
    pub(crate) fn compare_distance(
        self,
        left: Option<TailLine>,
        right: Option<TailLine>,
    ) -> Ordering {
        match (left, right) {
            (Some(left), Some(right)) => (left.fullness - self.0)
                .abs()
                .total_cmp(&(right.fullness - self.0).abs()),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

/// The goal and its key belong to one actual endpoint walk. Tail's invalid
/// optional domain remains an ordinary candidate, never a geometry error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LineQualityGoal {
    Balance(BalanceGoal),
    Tail(TailGoal),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LineQualityKey {
    Balance(NaturalResidual),
    Tail(Option<TailLine>),
}

impl LineQualityGoal {
    pub(crate) fn key(self, available: f64, indent: f64, advance: f64) -> Option<LineQualityKey> {
        match self {
            Self::Balance(_) => NaturalResidual::from_components(available, indent, advance)
                .map(LineQualityKey::Balance),
            Self::Tail(_) => Some(LineQualityKey::Tail(TailLine::from_components(
                available, indent, advance,
            ))),
        }
    }

    pub(crate) fn compare(self, left: LineQualityKey, right: LineQualityKey) -> Ordering {
        match (self, left, right) {
            (
                Self::Balance(goal),
                LineQualityKey::Balance(left),
                LineQualityKey::Balance(right),
            ) => goal.compare_distance(left, right),
            (Self::Tail(goal), LineQualityKey::Tail(left), LineQualityKey::Tail(right)) => {
                goal.compare_distance(left, right)
            }
            _ => unreachable!("one endpoint walk retains its actual quality goal"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TailTrial<T> {
    pub(crate) pair: TailPair,
    pub(crate) value: T,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TailWinner<T> {
    pub(crate) goal: TailGoal,
    pub(crate) trial: TailTrial<T>,
}

pub(crate) fn choose_tail<T, E>(
    reference: TailPair,
    mut run: impl FnMut(TailGoal) -> Result<Option<TailTrial<T>>, E>,
) -> Result<Option<TailWinner<T>>, E> {
    if !reference.eligible_auto() {
        return Ok(None);
    }
    let mut winner: Option<TailWinner<T>> = None;
    let mut previous = None;
    // A finite search portfolio, not a shortness cutoff or a group-size cap.
    for step in 0..=16 {
        let fraction = f64::from(step) / 16.0;
        let goal = TailGoal(
            (1.0 - fraction) * reference.last.fullness + fraction * reference.penultimate.fullness,
        );
        if previous == Some(goal) {
            continue;
        }
        previous = Some(goal);
        let Some(trial) = run(goal)? else { continue };
        if trial.pair.admission(reference) == TailAdmission::Improved
            && winner
                .as_ref()
                .is_none_or(|prior| trial.pair.minimum() > prior.trial.pair.minimum())
        {
            winner = Some(TailWinner { goal, trial });
        }
    }
    Ok(winner)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(widths: [f64; 2], advances: [f64; 2]) -> TailPair {
        TailPair::new(
            TailLine::from_components(widths[0], 0.0, advances[0]).unwrap(),
            TailLine::from_components(widths[1], 0.0, advances[1]).unwrap(),
        )
    }

    #[test]
    fn natural_tail_and_weaker_pair_both_improve_without_transferring_the_defect() {
        let reference = pair([60.0; 2], [60.0, 10.0]);
        let winner = choose_tail::<_, ()>(reference, |_| {
            Ok(Some(TailTrial {
                pair: pair([60.0; 2], [30.0, 40.0]),
                value: "repair",
            }))
        })
        .unwrap()
        .unwrap();
        assert_eq!(winner.trial.value, "repair");
        assert!(
            choose_tail::<_, ()>(reference, |_| Ok(Some(TailTrial {
                pair: pair([60.0; 2], [10.0, 60.0]),
                value: (),
            })))
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn a_narrower_band_with_the_same_natural_tail_is_not_a_repair() {
        let reference = pair([60.0, 80.0], [60.0, 10.0]);
        assert!(
            choose_tail::<_, ()>(reference, |_| Ok(Some(TailTrial {
                pair: pair([60.0; 2], [60.0, 10.0]),
                value: (),
            })))
            .unwrap()
            .is_none()
        );
        assert!(
            choose_tail::<_, ()>(reference, |_| Ok(Some(TailTrial {
                pair: pair([60.0, 20.0], [30.0, 9.0]),
                value: (),
            })))
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn optional_domain_is_finite_positive_capacity_and_a_fitting_natural_measure() {
        for components in [
            [f64::NAN, 0.0, 0.0],
            [60.0, f64::INFINITY, 10.0],
            [60.0, 0.0, f64::INFINITY],
            [f64::MAX, -f64::MAX, 10.0],
            [60.0, 60.0, 0.0],
            [60.0, 70.0, 0.0],
            [60.0, 0.0, -1.0],
            [60.0, 0.0, 61.0],
        ] {
            assert!(
                TailLine::from_components(components[0], components[1], components[2]).is_none()
            );
        }
        assert_eq!(
            TailLine::from_components(80.0, 20.0, 30.0)
                .unwrap()
                .fullness,
            0.5
        );
        assert_eq!(
            TailLine::from_components(f64::MAX, 0.0, f64::MAX)
                .unwrap()
                .fullness,
            1.0
        );
    }

    #[test]
    fn equal_improved_scores_keep_the_first_trial_and_owner_errors_propagate() {
        let reference = pair([60.0; 2], [60.0, 10.0]);
        let mut index = 0;
        let winner = choose_tail::<_, ()>(reference, |_| {
            index += 1;
            Ok(Some(TailTrial {
                pair: pair([60.0; 2], [30.0, 40.0]),
                value: index,
            }))
        })
        .unwrap()
        .unwrap();
        assert_eq!(winner.trial.value, 1);
        assert_eq!(
            choose_tail::<(), _>(reference, |_| Err("owner")),
            Err("owner")
        );
        let mut called = false;
        assert!(
            choose_tail::<(), ()>(pair([60.0; 2], [30.0, 40.0]), |_| {
                called = true;
                Ok(None)
            })
            .unwrap()
            .is_none()
        );
        assert!(!called);
    }
}
